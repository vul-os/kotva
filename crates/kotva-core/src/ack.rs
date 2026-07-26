//! Signed delivery acknowledgement (`Ack`) — spec §2.6, §18.9.18, §19.3.2, §20.1.
//!
//! The `ack` a recipient returns for a delivered MOTE is the delivery system's **only** durability
//! signal (§0.5, H-6), and the envelope `id` it references travels **unsealed** at every hop
//! (§18.3.1). If the ack were unsigned, any intermediary that read `id` off the wire could forge a
//! receipt and drive the sender's durability FSM straight to `ACKED` (§20.1), silently cancelling
//! the retry that is the sole guarantee of delivery. `ack_sig` is therefore **MUST** (§19.3.2).
//!
//! `Ack.ack_sig` covers the DS-tagged preimage
//! `"DMTAP-v0/ack" ‖ 0x00 ‖ det_cbor(Ack ∖ {3})` (= `det_cbor({1 => id, 2 => tier})`), signed
//! under the acknowledging recipient's `IK` or an `IK`-authorised, non-revoked device key (§1.2) —
//! the identical authorisation test §5.6.1 applies to cluster peers.
//!
//! The **sender** verifying an inbound ack rejects it — *no delivery evidence, no durability-state
//! change, the retry-queue entry stays `IN_FLIGHT`/`RETRY`* (§20.1) — when the signature does not
//! verify or the signing key is not authorised by the pinned recipient identity
//! (`ERR_ACK_SIG_INVALID`, `0x0317`), **or** when `Ack.tier` does not equal the tier the MOTE was
//! sent at (`ERR_ACK_TIER_MISMATCH`, `0x0318`; a return-path tier downgrade, §2.6). Neither is
//! grounds to short-circuit the mixnet-carried delivery the sender requested.

use crate::cbor::{self, as_u64, as_u8, as_bytes, CborError, Cv, Fields};
use crate::id::ContentId;
use crate::identity::{verify_domain, Identity, IdentityKey};
use crate::mote::Tier;
use crate::TimestampMs;

/// §18.9.18 domain-separation tag (ASCII ‖ trailing `0x00`; `sign_domain` prepends it).
pub const ACK_DS: &[u8] = b"DMTAP-v0/ack\x00";

/// A sender-side `ack`-verification failure (spec §18.9.18, §19.3.2, §20.1). Each carries its
/// §21.5 wire error code via [`AckError::code`]. Disposition for both is `DROP_SILENT` — the ack is
/// **not** delivery evidence and MUST NOT change durability state (the retry-queue entry stays
/// `IN_FLIGHT`/`RETRY`), and neither ever short-circuits the requested delivery.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AckError {
    /// The `ack_sig` does not verify, or verifies only under a key **not** currently authorised by
    /// the pinned recipient identity (`IK` or a non-revoked, `IK`-authorised device key). An
    /// unsigned/forged ack from an intermediary that merely read the unsealed `id` lands here.
    /// `ERR_ACK_SIG_INVALID` (`0x0317`).
    #[error("ack signature invalid or signed by an unauthorised key \
             (ERR_ACK_SIG_INVALID, 0x0317)")]
    SigInvalid,
    /// A genuinely-signed ack whose `tier` does not equal the tier the acknowledged MOTE was sent
    /// at — a return-path tier downgrade (e.g. a `private`-tier send acked over `fast`) or an
    /// implementation bug. Treated identically to a signature failure. `ERR_ACK_TIER_MISMATCH`
    /// (`0x0318`).
    #[error("ack tier does not match the tier the MOTE was sent at \
             (ERR_ACK_TIER_MISMATCH, 0x0318)")]
    TierMismatch,
    /// The ack could not be decoded as canonical CBOR (§18.1.1) or its fields were malformed.
    #[error("ack is malformed: {0}")]
    BadEncoding(#[from] CborError),
}

impl AckError {
    /// The normative DMTAP wire error code (§21.5).
    pub fn code(&self) -> u16 {
        match self {
            AckError::SigInvalid | AckError::BadEncoding(_) => 0x0317,
            AckError::TierMismatch => 0x0318,
        }
    }
}

/// A signed delivery acknowledgement (spec §18.9.18): the acknowledged `Envelope.id`, the privacy
/// `tier` the ack MUST travel at (the tier the MOTE was sent at), and `ack_sig`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ack {
    pub id: ContentId, // key 1 — the acknowledged Envelope.id (§18.3.1)
    pub tier: Tier,    // key 2 — the tier the MOTE was sent at (§2.6); wire byte 1=private, 2=fast
    pub ack_sig: Vec<u8>, // key 3 — §18.9.18, over det_cbor(Ack ∖ {3}) under an IK-authorised key
}

impl Ack {
    /// Integer-keyed canonical map (§18.9.18). `include_sig=false` omits key 3 for the signing
    /// preimage `det_cbor(Ack ∖ {3}) = det_cbor({1 => id, 2 => tier})`.
    fn to_cv(&self, include_sig: bool) -> Cv {
        let mut m = vec![
            (1u64, Cv::Bytes(self.id.as_bytes().to_vec())),
            (2, Cv::U64(self.tier.wire() as u64)),
        ];
        if include_sig {
            m.push((3, Cv::Bytes(self.ack_sig.clone())));
        }
        Cv::Map(m)
    }

    /// The exact wire bytes of this ack: §18-canonical integer-keyed CBOR.
    pub fn det_cbor(&self) -> Vec<u8> {
        cbor::encode(&self.to_cv(true))
    }

    /// The §18.9.18 signing body: `det_cbor(Ack ∖ {3})`. The [`ACK_DS`] tag (with its trailing
    /// `0x00`) is prepended by `sign_domain`, so the full preimage is
    /// `"DMTAP-v0/ack" ‖ 0x00 ‖ det_cbor({1 => id, 2 => tier})`.
    pub fn signing_body(&self) -> Vec<u8> {
        cbor::encode(&self.to_cv(false))
    }

    /// Decode an ack from its canonical CBOR (§18.9.18), failing closed on any violation (including
    /// an unknown `tier` byte — only `1`/`2` are defined).
    pub fn from_det_cbor(bytes: &[u8]) -> Result<Self, CborError> {
        let mut f = Fields::from_cv(cbor::decode(bytes)?)?;
        let id = ContentId(as_bytes(f.req(1)?)?);
        let tier = Tier::from_wire(as_u8(f.req(2)?)?).ok_or(CborError::TypeMismatch)?;
        let ack_sig = as_bytes(f.req(3)?)?;
        f.deny_unknown()?;
        Ok(Ack { id, tier, ack_sig })
    }

    /// Produce a signed ack for `id` at `tier` under the recipient's key `sk` (§18.9.18). `sk` is
    /// the recipient's `IK` or an `IK`-authorised device key.
    pub fn sign(id: ContentId, tier: Tier, sk: &IdentityKey) -> Ack {
        let mut ack = Ack { id, tier, ack_sig: Vec::new() };
        ack.ack_sig = sk.sign_domain(ACK_DS, &ack.signing_body());
        ack
    }

    /// **Sender-side verification** (§18.9.18, §19.3.2): does this ack authentically acknowledge a
    /// MOTE sent at `sent_tier` to the pinned recipient `recipient`?
    ///
    /// Fail closed, in order:
    /// 1. the `ack_sig` MUST verify under a key **currently authorised** by `recipient` — one of
    ///    the recipient's per-suite `IK`s, or the `device_key` of an embedded [`DeviceCert`] that
    ///    itself validly chains to that identity's `IK` (§1.2). Otherwise
    ///    [`AckError::SigInvalid`] (`0x0317`) — an intermediary that merely read the unsealed `id`
    ///    cannot forge a receipt;
    /// 2. `Ack.tier` MUST equal `sent_tier`; otherwise [`AckError::TierMismatch`] (`0x0318`) — a
    ///    return-path tier downgrade is refused (no-silent-downgrade, §2.6).
    ///
    /// A rejection is **not** delivery evidence: the caller MUST leave the durability FSM
    /// (`IN_FLIGHT`/`RETRY`) untouched (§20.1). The `id` this ack references is **not** checked
    /// here — the caller matches it against its own in-flight set.
    pub fn verify(&self, recipient: &Identity, sent_tier: Tier) -> Result<(), AckError> {
        // (1) Authorisation: the signature must verify under an IK or an IK-authorised device key.
        if !self.signed_by_authorised_key(recipient) {
            return Err(AckError::SigInvalid);
        }
        // (2) Tier match: a genuinely-signed ack at the wrong tier is still rejected (§2.6).
        if self.tier != sent_tier {
            return Err(AckError::TierMismatch);
        }
        Ok(())
    }

    /// Whether `ack_sig` verifies under some key authorised by `recipient`: an `IK` (any suite's
    /// entry in `iks`) or a `device_key` of a [`DeviceCert`] that validly chains to this identity's
    /// `IK` for its suite (§1.2, the §5.6.1 authorisation test). Verification is Ed25519 (the
    /// reference's only `Identity`-layer signature algorithm).
    fn signed_by_authorised_key(&self, recipient: &Identity) -> bool {
        let body = self.signing_body();
        // (a) the identity keys themselves.
        for ik in recipient.iks.values() {
            if verify_domain(ik, ACK_DS, &body, &self.ack_sig).is_ok() {
                return true;
            }
        }
        // (b) IK-authorised device keys: the cert must itself verify AND bind to the identity's IK
        // for its suite — an unbound or forged cert confers no authority (mirrors Identity::verify).
        for cert in &recipient.devices {
            let bound = recipient
                .iks
                .get(&cert.suite.as_u8())
                .is_some_and(|ik| ik.as_slice() == cert.ik.as_slice());
            if !bound || cert.verify().is_err() {
                continue;
            }
            if verify_domain(&cert.device_key, ACK_DS, &body, &self.ack_sig).is_ok() {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{Cap, DeviceCert, KeyPackageBundleRef};

    fn bundle(b: &[u8]) -> KeyPackageBundleRef {
        KeyPackageBundleRef::new("/mesh/keypkgs", ContentId::of(b))
    }

    fn recipient_identity(ik: &IdentityKey, devices: Vec<DeviceCert>) -> Identity {
        Identity::create_classical(
            ik,
            0,
            devices,
            bundle(b"kp"),
            ContentId::of(b"rec"),
            vec!["bob@example.com".into()],
            None,
            1_700_000_000_000,
        )
    }

    #[test]
    fn ack_signs_verifies_and_round_trips() {
        let ik = IdentityKey::generate();
        let recip = recipient_identity(&ik, vec![]);
        let ack = Ack::sign(ContentId::of(b"envelope-id"), Tier::Private, &ik);
        // Round-trips through canonical CBOR byte-identically.
        let bytes = ack.det_cbor();
        assert_eq!(bytes[0] & 0xe0, 0xa0, "ack is a CBOR map");
        assert_eq!(bytes[1], 0x01, "first key is integer 1 (id), not a text key");
        let back = Ack::from_det_cbor(&bytes).unwrap();
        assert_eq!(ack, back);
        assert_eq!(bytes, back.det_cbor());
        // Verifies under the recipient's IK at the matching tier.
        assert!(ack.verify(&recip, Tier::Private).is_ok());
    }

    #[test]
    fn ack_signed_by_authorised_device_key_verifies() {
        let ik = IdentityKey::generate();
        let device = IdentityKey::generate();
        let cert = DeviceCert::issue(&ik, device.public(), "phone", 1, None, vec![Cap::Recv]);
        let recip = recipient_identity(&ik, vec![cert]);
        // The device key (IK-authorised via its cert) may sign the ack.
        let ack = Ack::sign(ContentId::of(b"env"), Tier::Fast, &device);
        assert!(ack.verify(&recip, Tier::Fast).is_ok());
    }

    #[test]
    fn forged_ack_from_unauthorised_key_is_rejected() {
        // The victim recipient.
        let ik = IdentityKey::generate();
        let recip = recipient_identity(&ik, vec![]);
        // An intermediary that read the unsealed envelope `id` signs an ack with ITS OWN key —
        // authentic signature, but not authorised by the pinned recipient identity.
        let attacker = IdentityKey::generate();
        let forged = Ack::sign(ContentId::of(b"env"), Tier::Private, &attacker);
        let err = forged.verify(&recip, Tier::Private).unwrap_err();
        assert_eq!(err, AckError::SigInvalid);
        assert_eq!(err.code(), 0x0317);
    }

    #[test]
    fn tampered_ack_signature_is_rejected() {
        let ik = IdentityKey::generate();
        let recip = recipient_identity(&ik, vec![]);
        let mut ack = Ack::sign(ContentId::of(b"env"), Tier::Private, &ik);
        ack.ack_sig[0] ^= 0xff; // corrupt the signature
        assert_eq!(ack.verify(&recip, Tier::Private), Err(AckError::SigInvalid));
    }

    #[test]
    fn tier_mismatch_is_rejected_even_when_signature_is_valid() {
        let ik = IdentityKey::generate();
        let recip = recipient_identity(&ik, vec![]);
        // Genuinely signed at `private`, but the MOTE was sent at `fast` — a return-path downgrade.
        let ack = Ack::sign(ContentId::of(b"env"), Tier::Private, &ik);
        assert!(ack.verify(&recip, Tier::Private).is_ok());
        let err = ack.verify(&recip, Tier::Fast).unwrap_err();
        assert_eq!(err, AckError::TierMismatch);
        assert_eq!(err.code(), 0x0318);
    }

    #[test]
    fn unbound_device_cert_confers_no_ack_authority() {
        // A device cert issued by a DIFFERENT identity, dropped into the recipient's `devices`,
        // must not authorise its device key to ack for the recipient.
        let ik = IdentityKey::generate();
        let foreign_ik = IdentityKey::generate();
        let device = IdentityKey::generate();
        let foreign_cert =
            DeviceCert::issue(&foreign_ik, device.public(), "phone", 1, None, vec![Cap::Recv]);
        let recip = recipient_identity(&ik, vec![foreign_cert]);
        let ack = Ack::sign(ContentId::of(b"env"), Tier::Private, &device);
        assert_eq!(ack.verify(&recip, Tier::Private), Err(AckError::SigInvalid));
    }

    #[test]
    fn unknown_tier_byte_fails_closed_on_decode() {
        let ik = IdentityKey::generate();
        let ack = Ack::sign(ContentId::of(b"env"), Tier::Private, &ik);
        // Re-encode with a bogus tier byte (3) and confirm the decoder rejects it.
        let cv = Cv::Map(vec![
            (1u64, Cv::Bytes(ack.id.as_bytes().to_vec())),
            (2, Cv::U64(3)),
            (3, Cv::Bytes(ack.ack_sig.clone())),
        ]);
        let bytes = cbor::encode(&cv);
        assert!(Ack::from_det_cbor(&bytes).is_err());
    }
}
