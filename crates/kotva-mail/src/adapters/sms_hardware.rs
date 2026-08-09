//! §26.9 legacy adapter — **hardware SMS**, the in-tree reference adapter.
//!
//! Unlike the platform rails (Telegram/Slack/Discord/WhatsApp), hardware SMS is the *freely-
//! initiating, hardware-local* case (§26.4): it can reach a stranger cold given only a number, and
//! it receives on an attached modem's SIM — no network reachability, no public endpoint, nobody but
//! the user (and the carrier, inherent to SMS) on the plaintext leg (§26.4/§26.9). It is the rail the
//! node ships in-tree precisely because it depends on no third party's API or terms.
//!
//! Authenticity is still *platform-asserted* (§26.5): a sender number in an SMS PDU is what the
//! carrier put there, spoofable and unverifiable, so the origin rides the ONE canonical
//! platform-asserted `Headers.ext` entry (§26.5.1), never `Payload.from`. The hardware boundary — AT
//! commands to a modem — is the [`Modem`] trait; nothing here talks to real hardware, so it runs in
//! CI.

use kotva_core::mote::{Headers, Payload};

use super::{
    platform_asserted_cv, platform_asserted_origin, LegacyAdapter, PlatformAsserted, RailMessage,
    RailProperties, RailSend, RailTransport, TransportError, PLATFORM_ASSERTED_EXT_KEY,
    SMS_HARDWARE,
};

/// The rail label carried in the platform-asserted origin (§26.5.1). `"sms"` — the transport, not
/// the deployment: node-mode hardware and gateway-mode aggregator are the same rail (§26.4).
const RAIL: &str = "sms";

/// The in-tree reference SMS adapter (§26.9): freely-initiating, hardware-local, free.
#[derive(Debug, Default, Clone, Copy)]
pub struct SmsHardwareAdapter;

impl LegacyAdapter for SmsHardwareAdapter {
    fn properties(&self) -> &RailProperties {
        &SMS_HARDWARE
    }

    /// Map an inbound SMS → a MOTE [`Payload`]. The sender number is **carrier-asserted** (§26.5):
    /// it rides the canonical platform-asserted ext entry with `verifiable = false`, and `from`/`sig`
    /// are left empty — an SMS sender id is not a cryptographic identity key and is trivially spoofed.
    fn inbound_to_mote(&self, msg: &RailMessage) -> Payload {
        Payload {
            from: Vec::new(),
            sig: Vec::new(),
            headers: Headers {
                mime: Some("text/plain; charset=utf-8".to_string()),
                ext: vec![(
                    PLATFORM_ASSERTED_EXT_KEY.to_string(),
                    platform_asserted_cv(RAIL, &msg.from),
                )],
                ..Default::default()
            },
            body: msg.text.clone().into_bytes(),
            refs: Vec::new(),
            attach: Vec::new(),
            expires: None,
        }
    }

    // `outbound_disposition` uses the trait default. Because SMS is FREELY-INITIATING (§26.4), a cold
    // send with no prior inbound resolves to `Deliverable` — the deliberate contrast to the
    // platform rails, whose cold sends hit the §26.4.2 wall.
}

/// The hardware boundary: send an SMS via an attached modem (e.g. AT+CMGS). Mocked in tests; a real
/// deployment drives a serial modem. No network, no third-party API, no credentials beyond the SIM.
pub trait Modem {
    /// Submit an SMS to `to` with `text`; `Err` carries the modem/network failure.
    fn submit_sms(&self, to: &str, text: &str) -> Result<(), TransportError>;
}

/// A [`RailTransport`] over a [`Modem`].
pub struct SmsHardwareTransport<M: Modem> {
    modem: M,
}

impl<M: Modem> SmsHardwareTransport<M> {
    pub fn new(modem: M) -> Self {
        Self { modem }
    }
}

impl<M: Modem> RailTransport for SmsHardwareTransport<M> {
    fn send(&self, send: RailSend) -> Result<(), TransportError> {
        self.modem.submit_sms(&send.to, &send.text)
    }
}

/// Read the carrier-asserted origin back out of an inbound SMS MOTE (delegates to the shared reader).
#[must_use]
pub fn sms_origin(payload: &Payload) -> Option<PlatformAsserted> {
    platform_asserted_origin(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{DeploymentMode, InitiationClass, OutboundDisposition, Sanctioning};
    use std::cell::RefCell;

    /// §26.4/§26.9: hardware SMS is freely-initiating in both directions, hardware-local, free,
    /// native-standard (hostable in either mode), and platform-asserted.
    #[test]
    fn properties_match_spec_26_9() {
        let p = SmsHardwareAdapter.properties();
        assert_eq!(p.inbound.initiation, InitiationClass::FreelyInitiating);
        assert_eq!(p.outbound.initiation, InitiationClass::FreelyInitiating);
        assert!(
            p.can_initiate_outbound_cold(),
            "SMS can reach a stranger cold (§26.4)"
        );
        assert_eq!(p.sanctioning, Sanctioning::NativeStandard);
        assert!(p.permits_mode(DeploymentMode::Node) && p.permits_mode(DeploymentMode::Gateway));
    }

    /// The carrier-asserted origin rides the ONE canonical ext entry; `from` stays empty (§26.5).
    #[test]
    fn inbound_carries_carrier_asserted_origin_never_from() {
        let msg = RailMessage {
            from: "+27821234567".to_string(),
            text: "hello over sms".to_string(),
            opens_window: true,
        };
        let mote = SmsHardwareAdapter.inbound_to_mote(&msg);
        assert_eq!(mote.body, b"hello over sms");
        assert!(
            mote.from.is_empty(),
            "an SMS number must never masquerade as a verified sender IK"
        );
        let origin = sms_origin(&mote).expect("carrier-asserted origin present");
        assert_eq!(origin.rail, "sms");
        assert_eq!(origin.claim, "+27821234567");
        assert!(
            !origin.verifiable,
            "an SMS sender id is unverifiable (§26.5)"
        );
        // Survives the canonical wire round-trip.
        let back = Payload::from_det_cbor(&mote.det_cbor()).unwrap();
        assert_eq!(sms_origin(&back), Some(origin));
    }

    /// The contrast to the platform rails: a COLD SMS (no prior inbound, no window) is Deliverable,
    /// because SMS is freely-initiating (§26.4) — no §26.4.2 wall.
    #[test]
    fn cold_send_is_deliverable_freely_initiating() {
        let a = SmsHardwareAdapter;
        assert!(matches!(
            a.outbound_disposition("+27821234567", "cold hello", false),
            OutboundDisposition::Deliverable(_)
        ));
    }

    /// The transport drives the modem; a modem failure surfaces as `TransportError`, never as success.
    #[test]
    fn transport_drives_the_modem_and_surfaces_failure() {
        struct MockModem {
            sent: RefCell<Vec<(String, String)>>,
            fail: bool,
        }
        impl Modem for MockModem {
            fn submit_sms(&self, to: &str, text: &str) -> Result<(), TransportError> {
                if self.fail {
                    return Err(TransportError::Unreachable);
                }
                self.sent
                    .borrow_mut()
                    .push((to.to_string(), text.to_string()));
                Ok(())
            }
        }
        let ok = SmsHardwareTransport::new(MockModem {
            sent: RefCell::new(vec![]),
            fail: false,
        });
        ok.send(RailSend {
            to: "+27821234567".to_string(),
            text: "hi".to_string(),
        })
        .unwrap();
        assert_eq!(
            ok.modem.sent.borrow().as_slice(),
            &[("+27821234567".to_string(), "hi".to_string())]
        );

        let bad = SmsHardwareTransport::new(MockModem {
            sent: RefCell::new(vec![]),
            fail: true,
        });
        assert_eq!(
            bad.send(RailSend {
                to: "x".to_string(),
                text: "y".to_string()
            }),
            Err(TransportError::Unreachable)
        );
    }
}
