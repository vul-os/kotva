//! [`UsageReceipt`] — the signed usage receipt CONTRACT §6 requires a metering coordinator to
//! deliver **directly to the paying party** (§18.8a.2).
//!
//! ```cddl
//! UsageReceipt = {
//!   1 => suite,            ; suite      signature suite of `sig`
//!   2 => ik-pub,           ; identity   the issuing coordinator's own identity (self-certifying)
//!   3 => bytes,            ; operation  opaque det_cbor metered operation
//!   4 => sig-val,          ; sig        over fields 1-3, DS-tag DMTAP-COORD-v0/usage-receipt
//! }
//! ```
//!
//! **Independently self-certifying**, for the same reason as [`crate::Tariff`] and one more:
//! CONTRACT §6's framing is that "a signed receipt lets a user confirm a claimed operation was
//! real", and that check must hold up standalone at whatever later point the payer re-examines it,
//! **without a live descriptor fetch**. So it carries its own signer rather than depending on an
//! enclosing descriptor.
//!
//! # Transport
//!
//! The existing `system` MOTE (`kind = 0x0A`, §21.16), delivered directly to the paying identity and
//! **never published**. No new message kind is allocated. `kind = 0x0A` is shared with capability
//! announcements (§10.2) and bounce notices (§7.10.3a), so the `Body` shape alone is ambiguous: the
//! MOTE's `Headers.mime` MUST be [`crate::USAGE_RECEIPT_MIME`], and a receiver MUST inspect
//! `Headers.mime` **before** parsing a `0x0A` body — an unrecognised `mime` is an undecodable
//! system message, never a guess between the three shapes.
//!
//! # Honest residual (normative disclosure, CONTRACT §6)
//!
//! A verified receipt proves the coordinator signed a claim about one real operation. It is
//! **one-directional**: it cannot disconfirm an operation the coordinator fabricated or silently
//! omitted, and a client MUST NOT present the absence of a disputed receipt as proof the operation
//! never happened. Disclosed, not hidden.

use kotva_core::cbor::{self, as_bytes, Cv, Fields};
use kotva_core::Suite;

use crate::sig::{self, Which, USAGE_RECEIPT_DS};
use crate::{CoordinatorError, DetCbor};

/// A signed usage receipt (§18.8a.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageReceipt {
    /// Key 1 — the signature suite of [`UsageReceipt::sig`].
    pub suite: Suite,
    /// Key 2 — the **issuing** coordinator's own `ik-pub`; verified against `sig`, never an
    /// enclosing descriptor's.
    pub identity: Vec<u8>,
    /// Key 3 — the opaque det_cbor description of the metered operation (kind-specific: a
    /// legacy-adapter send, §26.10; a DEPOT `edge-fn` invocation; …).
    pub operation: DetCbor,
    /// Key 4 — the detached signature by [`UsageReceipt::identity`].
    pub sig: Vec<u8>,
}

impl UsageReceipt {
    /// The §18.9 preimage body: `det_cbor(UsageReceipt ∖ {4})`.
    pub fn signing_body(&self) -> Vec<u8> {
        cbor::encode(&self.body_cv())
    }

    fn body_cv(&self) -> Cv {
        Cv::Map(vec![
            (1, Cv::U64(self.suite.as_u8() as u64)),
            (2, Cv::Bytes(self.identity.clone())),
            (3, Cv::Bytes(self.operation.0.clone())),
        ])
    }

    fn to_cv(&self) -> Cv {
        let mut m = match self.body_cv() {
            Cv::Map(m) => m,
            _ => unreachable!("body_cv always returns a map"),
        };
        m.push((4, Cv::Bytes(self.sig.clone())));
        Cv::Map(m)
    }

    /// Encode to the §18.8a.2 wire bytes.
    pub fn det_cbor(&self) -> Vec<u8> {
        cbor::encode(&self.to_cv())
    }

    /// Issue and sign a receipt.
    pub fn sign(signer: &sig::Signer<'_>, operation: DetCbor) -> UsageReceipt {
        let mut r = UsageReceipt {
            suite: signer.suite(),
            identity: signer.public(),
            operation,
            sig: Vec::new(),
        };
        r.sig = signer.sign(USAGE_RECEIPT_DS, &r.signing_body());
        r
    }

    /// Verify the signature against this receipt's **own** identity (§18.9).
    pub fn verify(&self) -> Result<(), CoordinatorError> {
        sig::verify(
            self.suite,
            &self.identity,
            USAGE_RECEIPT_DS,
            &self.signing_body(),
            &self.sig,
        )
    }

    /// Whether this receipt was issued by `identity` — the check a payer makes against the
    /// coordinator it actually transacted with, since a valid signature by *someone* proves nothing
    /// about *who*.
    pub fn was_issued_by(&self, identity: &[u8]) -> bool {
        self.identity == identity
    }

    fn from_cv(cv: Cv) -> Result<Self, CoordinatorError> {
        let mut f = Fields::from_cv(cv)?;
        let suite = sig::require_suite(f.req(1)?)?;
        let identity = as_bytes(f.req(2)?)?;
        sig::require_len("identity", suite, &identity, Which::IkPub)?;
        let operation = DetCbor(as_bytes(f.req(3)?)?);
        let sig_bytes = as_bytes(f.req(4)?)?;
        sig::require_len("sig", suite, &sig_bytes, Which::SigVal)?;
        f.deny_unknown()?;
        Ok(UsageReceipt {
            suite,
            identity,
            operation,
            sig: sig_bytes,
        })
    }

    /// Decode the §18.8a.2 wire bytes. **Does not verify the signature** — see
    /// [`UsageReceipt::from_det_cbor_verified`].
    pub fn from_det_cbor(bytes: &[u8]) -> Result<Self, CoordinatorError> {
        UsageReceipt::from_cv(cbor::decode(bytes)?)
    }

    /// Decode **and** verify. The only entry point whose result may be treated as authentic; a
    /// failure here is `ERR_ADAPTER_RECEIPT_INVALID` (`0x0B02`) in an adapter context (§21.11a).
    pub fn from_det_cbor_verified(bytes: &[u8]) -> Result<Self, CoordinatorError> {
        let r = UsageReceipt::from_det_cbor(bytes)?;
        r.verify()?;
        Ok(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kotva_core::identity::IdentityKey;

    fn op() -> DetCbor {
        DetCbor::from_cv(&Cv::Map(vec![(1, Cv::U64(7))]))
    }

    #[test]
    fn signs_verifies_and_round_trips() {
        let ik = IdentityKey::from_seed(&[11u8; 32]);
        let r = UsageReceipt::sign(&sig::Signer::Classical(&ik), op());
        r.verify().unwrap();
        let b = r.det_cbor();
        let back = UsageReceipt::from_det_cbor_verified(&b).unwrap();
        assert_eq!(back, r);
        assert_eq!(back.det_cbor(), b);
        assert!(back.was_issued_by(&ik.public()));
    }

    #[test]
    fn a_tampered_operation_fails_verification() {
        let ik = IdentityKey::from_seed(&[12u8; 32]);
        let mut r = UsageReceipt::sign(&sig::Signer::Classical(&ik), op());
        // The metering claim itself — "you sent 1 message", rewritten to 700.
        r.operation = DetCbor::from_cv(&Cv::Map(vec![(1, Cv::U64(700))]));
        assert_eq!(r.verify(), Err(CoordinatorError::BadSignature));
    }

    #[test]
    fn a_receipt_signature_cannot_be_replayed_as_a_tariff_or_descriptor() {
        // The DS-tag is what makes this true; asserting it here keeps the property attached to the
        // object rather than only to `sig.rs`'s own unit test.
        let ik = IdentityKey::from_seed(&[13u8; 32]);
        let r = UsageReceipt::sign(&sig::Signer::Classical(&ik), op());
        let body = r.signing_body();
        assert!(sig::verify(r.suite, &r.identity, USAGE_RECEIPT_DS, &body, &r.sig).is_ok());
        for other in [crate::TARIFF_DS, crate::DESCRIPTOR_DS] {
            assert_eq!(
                sig::verify(r.suite, &r.identity, other, &body, &r.sig),
                Err(CoordinatorError::BadSignature)
            );
        }
    }

    #[test]
    fn a_receipt_from_the_wrong_coordinator_verifies_but_is_not_attributed() {
        let mine = IdentityKey::from_seed(&[14u8; 32]);
        let theirs = IdentityKey::from_seed(&[15u8; 32]);
        let r = UsageReceipt::sign(&sig::Signer::Classical(&theirs), op());
        // A perfectly valid signature — by the wrong party. `verify` alone is not enough.
        r.verify().unwrap();
        assert!(!r.was_issued_by(&mine.public()));
    }

    #[test]
    fn the_pq_hybrid_suite_round_trips() {
        let hk = kotva_core::pq::HybridSigningKey::generate();
        let r = UsageReceipt::sign(&sig::Signer::PqHybrid(&hk), op());
        assert_eq!(r.suite, Suite::PqHybrid);
        let back = UsageReceipt::from_det_cbor_verified(&r.det_cbor()).unwrap();
        assert_eq!(back, r);
        // Strip the ML-DSA half of the signature: AND-composition means the surviving Ed25519 half
        // must NOT be accepted (`ERR_HYBRID_SUITE_INCOMPLETE`, 0x0210).
        let mut stripped = r.clone();
        stripped.sig.truncate(64);
        assert!(matches!(
            UsageReceipt::from_det_cbor(&stripped.det_cbor()),
            Err(CoordinatorError::BadFieldLength { .. })
        ));
        assert_eq!(stripped.verify(), Err(CoordinatorError::BadSignature));
    }

    #[test]
    fn every_field_is_required() {
        let ik = IdentityKey::from_seed(&[16u8; 32]);
        let r = UsageReceipt::sign(&sig::Signer::Classical(&ik), op());
        for drop in [1u64, 2, 3, 4] {
            let m: Vec<(u64, Cv)> = match r.to_cv() {
                Cv::Map(m) => m.into_iter().filter(|(k, _)| *k != drop).collect(),
                _ => unreachable!(),
            };
            assert!(
                UsageReceipt::from_det_cbor(&cbor::encode(&Cv::Map(m))).is_err(),
                "key {drop} is MUST-present and its absence must be rejected"
            );
        }
    }
}
