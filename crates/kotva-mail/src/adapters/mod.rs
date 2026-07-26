//! §26 Legacy Adapters — the framework every rail adapter implements.
//!
//! An **adapter** (spec §26) bridges one legacy communication rail (SMS, WhatsApp, Telegram,
//! Discord, Slack, …) to DMTAP MOTEs. This module implements the *spec-dictated* parts that are
//! pure data and logic — testable with no network:
//!
//!   * the **four fields** every adapter declares (§26.3): initiation, inbound-transport, price,
//!     exposure — stated **per direction** where the rail is asymmetric (§26.3, §26.4.1);
//!   * the **two deployment modes** (§26.2) node vs gateway, and the **sanctioning** rule we adopt:
//!     a rail with no sanctioned API (an unofficial/ToS-violating bridge) is **node-mode only** and
//!     MUST NOT be offered operator-hosted in gateway mode;
//!   * the **authenticity model** (§26.5): email is the only cryptographically-verifiable rail;
//!     every platform rail is *platform-asserted* and unverifiable;
//!   * the canonical **per-rail property table** (§26.4) as data, with tests that pin it to the spec.
//!
//! The **live-network boundary** is the [`RailTransport`] trait; a real deployment supplies an
//! implementation (Slack/Discord/Telegram official APIs, the WhatsApp **Business** Cloud API, an SMS
//! aggregator, an attached modem). Nothing here holds credentials or makes live calls, so it runs in
//! CI; the per-rail modules bind the transport.

use kotva_core::mote::Payload;

/// §26.3 field 1 — *can it initiate?* The load-bearing field: can this rail reach a stranger cold?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitiationClass {
    /// Can contact a party who has never interacted before (email, SMS).
    FreelyInitiating,
    /// The other party MUST open the conversation first, or only a constrained class of message may
    /// be sent (WhatsApp templates; Telegram/Discord/Slack bots that cannot DM first, §26.4.2).
    InboundTriggered,
}

/// §26.3 field 2 — *how does it receive?* Determines whether it works behind CGNAT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboundTransportClass {
    /// Inbound arrives on physical hardware the adapter owns (an attached SMS modem). No network.
    HardwareLocal,
    /// The adapter holds an outbound connection open and receives over it (Telegram long-polling,
    /// Discord Gateway WebSocket, Slack Socket Mode). Works behind CGNAT.
    OutboundPersistent,
    /// The platform calls the adapter back over HTTPS (WhatsApp Cloud API, most SMS aggregators).
    /// Needs a reachable HTTPS endpoint.
    Webhook,
    /// The adapter runs its own server the outside world dials (SMTP on 25). Needs a public address.
    Listener,
}

/// §26.3 field 3 — *price shape.*
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceShape {
    /// Real marginal cost passed through (SMS aggregator, WhatsApp outbound-cold).
    Metered,
    /// A fixed fee unrelated to per-message volume.
    Flat,
    /// No adapter fee.
    Free,
}

/// §26.2 — the deployment mode is a property of the *deployment*, never of the adapter's code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentMode {
    /// The user runs the adapter for themselves, with their own credentials/hardware.
    Node,
    /// An operator (e.g. Ephor) runs the adapter for others — taking on the plaintext exposure and,
    /// for a platform rail, the platform's terms.
    Gateway,
}

/// §26.5 — the authenticity a recipient can actually get from this rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailAuthenticity {
    /// Email only: SPF/DKIM/DMARC in the signed `GatewayAttestation` (§7.2c) — *verifiable*.
    CryptographicLegacy,
    /// Every platform rail: "the platform's API told me this came from X" — trusted, not verified.
    PlatformAsserted,
}

/// Whether the rail has an operator-hostable sanctioned integration. Drives the gateway-mode rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sanctioning {
    /// An open standard the node itself speaks (email, hardware SMS). Always hostable.
    NativeStandard,
    /// An official, terms-compliant API exists (Slack API, Discord/Telegram bots, WhatsApp Business
    /// Cloud API). May run in gateway mode, honouring the API's own terms.
    SanctionedApi,
    /// No sanctioned bridge exists (consumer WhatsApp, iMessage, Signal). An adapter here would
    /// violate the platform's ToS and break its E2E; **node-mode only**, never operator-hosted.
    Unsanctioned,
}

/// The per-direction half of an adapter's declaration (§26.3: properties are per-direction, not
/// per-platform — WhatsApp is the clearest case, §26.4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectionProperties {
    pub initiation: InitiationClass,
    pub price: PriceShape,
    /// Who sees plaintext on this leg in **node** mode (§26.5, §26.6).
    pub exposure_node: &'static str,
    /// Who sees plaintext on this leg in **gateway** mode (adds the operator; for a platform rail
    /// neither mode reaches "nobody", §26.5.1).
    pub exposure_gateway: &'static str,
}

/// A rail's full §26.4 declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RailProperties {
    /// Rail label as in §26.4 (e.g. `"telegram"`, `"whatsapp"`).
    pub rail: &'static str,
    pub inbound_transport: InboundTransportClass,
    pub authenticity: RailAuthenticity,
    pub sanctioning: Sanctioning,
    /// Inbound (rail → DMTAP) direction.
    pub inbound: DirectionProperties,
    /// Outbound (DMTAP → rail) direction — where asymmetry lives (§26.4.1).
    pub outbound: DirectionProperties,
}

impl RailProperties {
    /// §26 (the sanctioning rule): an **unsanctioned** rail MUST NOT run in gateway mode — it can
    /// only be self-hosted by a user who accepts their own account's ToS/ban risk. Every other rail
    /// may run in either mode.
    #[must_use]
    pub fn permits_mode(&self, mode: DeploymentMode) -> bool {
        !(self.sanctioning == Sanctioning::Unsanctioned && mode == DeploymentMode::Gateway)
    }

    /// §26.4.2 — a rail that is `InboundTriggered` **cannot** originate a conversation cold. A
    /// conformant adapter MUST refuse to imply otherwise. Returns true iff an outbound-cold send
    /// (no prior inbound, outside any service window) is possible on this rail at all.
    #[must_use]
    pub fn can_initiate_outbound_cold(&self) -> bool {
        self.outbound.initiation == InitiationClass::FreelyInitiating
    }
}

/// Result of attempting to map an outbound MOTE onto a rail send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutboundDisposition {
    /// The send is deliverable now.
    Deliverable(RailSend),
    /// The rail is inbound-triggered and there is no open window / prior inbound — a **functional
    /// wall** (§26.4.1/§26.4.2), not a price. The caller MUST surface this, not silently drop.
    BlockedNoWindow,
    /// The rail can only carry a constrained class (WhatsApp templates) outside the window.
    RequiresTemplate,
}

/// A rail-agnostic outbound send request (the transport binds the platform specifics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RailSend {
    pub to: String,
    pub text: String,
}

/// A rail-agnostic inbound message the transport delivers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RailMessage {
    /// The platform-asserted origin (a phone number, Telegram user id, Discord snowflake, …).
    pub from: String,
    pub text: String,
    /// Whether this inbound opens/refreshes a reply window (WhatsApp's 24h service window).
    pub opens_window: bool,
}

/// The live-network boundary. A real deployment supplies this (official Slack/Discord/Telegram APIs,
/// the WhatsApp Business Cloud API, an SMS aggregator, an attached modem). Errors are the platform's.
pub trait RailTransport {
    fn send(&self, send: RailSend) -> Result<(), TransportError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// The platform rejected the send (rate limit, template not approved, not in window, …).
    Rejected(String),
    /// The transport could not reach the platform.
    Unreachable,
}

/// A legacy-rail adapter (§26). The declaration is pure data; the send/receive binds a transport.
pub trait LegacyAdapter {
    /// The rail's §26.4 declaration.
    fn properties(&self) -> &RailProperties;

    /// Map an inbound rail message → a DMTAP MOTE `Payload`, carrying the platform-asserted origin
    /// **as such** (never as a cryptographically-verified sender — §26.5).
    fn inbound_to_mote(&self, msg: &RailMessage) -> Payload;

    /// Decide whether an outbound MOTE can be sent now (§26.4.1/§26.4.2 initiation constraints).
    /// `window_open` reflects whether a reply window is open (only meaningful for WhatsApp-class).
    fn outbound_disposition(&self, to: &str, text: &str, window_open: bool) -> OutboundDisposition {
        let p = self.properties();
        if p.can_initiate_outbound_cold() || window_open {
            OutboundDisposition::Deliverable(RailSend { to: to.to_string(), text: text.to_string() })
        } else if p.rail == "whatsapp" {
            OutboundDisposition::RequiresTemplate
        } else {
            OutboundDisposition::BlockedNoWindow
        }
    }
}

// ── The canonical §26.4 per-rail property table, as data (tests pin it to the spec) ────────────

/// SMS over an attached modem — the in-tree reference adapter (§26.9).
pub const SMS_HARDWARE: RailProperties = RailProperties {
    rail: "sms-hardware",
    inbound_transport: InboundTransportClass::HardwareLocal,
    authenticity: RailAuthenticity::PlatformAsserted,
    sanctioning: Sanctioning::NativeStandard,
    inbound: DirectionProperties {
        initiation: InitiationClass::FreelyInitiating,
        price: PriceShape::Free,
        exposure_node: "nobody but the user (own hardware, own SIM) + the user's own carrier",
        exposure_gateway: "the operator + the carrier",
    },
    outbound: DirectionProperties {
        initiation: InitiationClass::FreelyInitiating,
        price: PriceShape::Free,
        exposure_node: "nobody but the user + the destination carrier",
        exposure_gateway: "the operator + the destination carrier",
    },
};

/// WhatsApp via the **Business** Cloud API (§26.4, §26.4.1). Asymmetric: inbound / in-window reply is
/// free; outbound-cold is metered + template-walled.
pub const WHATSAPP_BUSINESS: RailProperties = RailProperties {
    rail: "whatsapp",
    inbound_transport: InboundTransportClass::Webhook,
    authenticity: RailAuthenticity::PlatformAsserted,
    sanctioning: Sanctioning::SanctionedApi,
    inbound: DirectionProperties {
        initiation: InitiationClass::InboundTriggered,
        price: PriceShape::Free, // within the service window
        exposure_node: "Meta, always + the user (self-hosted WABA)",
        exposure_gateway: "Meta, always + the operator",
    },
    outbound: DirectionProperties {
        // Outside the window this leg cannot originate at all — template-walled (§26.4.1).
        initiation: InitiationClass::InboundTriggered,
        price: PriceShape::Metered,
        exposure_node: "Meta, always + the user",
        exposure_gateway: "Meta, always + the operator",
    },
};

/// Telegram bot (§26.4, §26.4.2): inbound-triggered, cannot DM first.
pub const TELEGRAM: RailProperties = platform_rail("telegram", "Telegram");
/// Discord bot (§26.4, §26.4.2): a shared guild is required.
pub const DISCORD: RailProperties = platform_rail("discord", "Discord");
/// Slack app (§26.4, §26.4.2): a workspace install is required.
pub const SLACK: RailProperties = platform_rail("slack", "Slack");

/// The Telegram/Discord/Slack shape (§26.4.2): sanctioned bot API, outbound-persistent, free,
/// platform-asserted, and — critically — **cannot initiate at all** in either direction.
const fn platform_rail(rail: &'static str, platform: &'static str) -> RailProperties {
    let dir = DirectionProperties {
        initiation: InitiationClass::InboundTriggered,
        price: PriceShape::Free,
        exposure_node: platform, // the platform always sees plaintext
        exposure_gateway: platform,
    };
    RailProperties {
        rail,
        inbound_transport: InboundTransportClass::OutboundPersistent,
        authenticity: RailAuthenticity::PlatformAsserted,
        sanctioning: Sanctioning::SanctionedApi,
        inbound: dir,
        outbound: dir,
    }
}

/// Every rail this crate declares, for descriptor publication and the capability matrix (§26.4).
#[must_use]
pub fn all_rails() -> &'static [RailProperties] {
    &[SMS_HARDWARE, WHATSAPP_BUSINESS, TELEGRAM, DISCORD, SLACK]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The encoded table matches §26.4 for the load-bearing facts.
    #[test]
    fn rail_table_matches_spec_26_4() {
        // §26.4.2: Telegram/Discord/Slack cannot initiate at all — in EITHER direction.
        for r in [TELEGRAM, DISCORD, SLACK] {
            assert_eq!(r.inbound.initiation, InitiationClass::InboundTriggered);
            assert_eq!(r.outbound.initiation, InitiationClass::InboundTriggered);
            assert!(!r.can_initiate_outbound_cold(), "{} must not initiate cold (§26.4.2)", r.rail);
            assert_eq!(r.inbound_transport, InboundTransportClass::OutboundPersistent);
            assert_eq!(r.outbound.price, PriceShape::Free);
            assert_eq!(r.authenticity, RailAuthenticity::PlatformAsserted);
        }
        // §26.4.1: WhatsApp is asymmetric — outbound-cold is metered + template-walled.
        assert_eq!(WHATSAPP_BUSINESS.inbound.price, PriceShape::Free);
        assert_eq!(WHATSAPP_BUSINESS.outbound.price, PriceShape::Metered);
        assert!(!WHATSAPP_BUSINESS.can_initiate_outbound_cold());
        // §26.9 / §26.4: hardware SMS freely initiates and is free, both directions.
        assert!(SMS_HARDWARE.can_initiate_outbound_cold());
        assert_eq!(SMS_HARDWARE.inbound_transport, InboundTransportClass::HardwareLocal);
        // §26.5: no platform rail is cryptographically verifiable.
        for r in all_rails() {
            assert_eq!(r.authenticity, RailAuthenticity::PlatformAsserted,
                "no legacy rail here has verifiable auth (§26.5): {}", r.rail);
        }
    }

    /// The sanctioning rule: unsanctioned rails are node-mode only; sanctioned/native may gateway.
    #[test]
    fn unsanctioned_rails_are_node_mode_only() {
        let unofficial = RailProperties { sanctioning: Sanctioning::Unsanctioned, ..TELEGRAM };
        assert!(unofficial.permits_mode(DeploymentMode::Node));
        assert!(!unofficial.permits_mode(DeploymentMode::Gateway),
            "an unsanctioned (ToS-violating) rail MUST NOT be operator-hosted (§26)");
        // The sanctioned WhatsApp Business API and native SMS may run in gateway mode.
        assert!(WHATSAPP_BUSINESS.permits_mode(DeploymentMode::Gateway));
        assert!(SMS_HARDWARE.permits_mode(DeploymentMode::Gateway));
    }

    /// §26.4.1/§26.4.2: an outbound-cold attempt on an inbound-triggered rail is a functional wall,
    /// surfaced — never silently deliverable.
    #[test]
    fn outbound_cold_on_inbound_triggered_rail_is_a_wall() {
        struct A(RailProperties);
        impl LegacyAdapter for A {
            fn properties(&self) -> &RailProperties { &self.0 }
            fn inbound_to_mote(&self, _m: &RailMessage) -> Payload { unimplemented!() }
        }
        // Telegram cold with no window → blocked (not deliverable).
        let tg = A(TELEGRAM);
        assert_eq!(tg.outbound_disposition("user", "hi", false), OutboundDisposition::BlockedNoWindow);
        // WhatsApp cold with no window → requires a template (the wall), not a free send.
        let wa = A(WHATSAPP_BUSINESS);
        assert_eq!(wa.outbound_disposition("+27821234567", "hi", false), OutboundDisposition::RequiresTemplate);
        // WhatsApp inside the window → deliverable.
        assert!(matches!(wa.outbound_disposition("+27821234567", "hi", true), OutboundDisposition::Deliverable(_)));
    }
}
