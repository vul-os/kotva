//! [`Tariff`] — the signed price schedule CONTRACT §6 requires of a coordinator that charges
//! (§18.8a.1).
//!
//! ```cddl
//! Tariff = {
//!   1 => suite,            ; suite       signature suite of `sig`
//!   2 => ik-pub,           ; identity    the signing coordinator's OWN identity (self-certifying)
//!   3 => bytes,            ; schedule    opaque det_cbor price shape/schedule
//!   ? 4 => ts,             ; valid_until OPTIONAL expiry; absent ⇒ no expiry
//!   5 => sig-val,          ; sig         over fields 1-4, DS-tag DMTAP-COORD-v0/tariff
//! }
//! ```
//!
//! **Self-certifying, and that is load-bearing.** `Tariff.identity` is the *signing* coordinator's
//! own key, not the enclosing descriptor's, so a client handed a tariff directly by an operator — or
//! holding one cached from an earlier descriptor — can verify it standalone. It MAY, but need not,
//! equal `CoordinatorDescriptor.identity`. A client MUST attribute the tariff to `Tariff.identity`
//! and MUST surface that distinction when comparing prices across coordinators (§18.8a.1, §26.10);
//! [`Tariff::is_self_issued_by`] exists so that check is one call rather than a convention.

use kotva_core::cbor::{self, as_bytes, as_u64, Cv, Fields};
use kotva_core::{Suite, TimestampMs};

use crate::sig::{self, Which, TARIFF_DS};
use crate::{CoordinatorError, DetCbor};

/// A signed price schedule (§18.8a.1). The **numbers** are operator policy and out of scope for the
/// specification; only the mechanism — a signed, verifiable, comparable object — is normative.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tariff {
    /// Key 1 — the signature suite of [`Tariff::sig`], which pins the §18.9 representative.
    pub suite: Suite,
    /// Key 2 — the **signing** coordinator's own `ik-pub`.
    pub identity: Vec<u8>,
    /// Key 3 — the opaque det_cbor price shape/schedule.
    pub schedule: DetCbor,
    /// Key 4, OPTIONAL — end of this tariff's validity window, covered by `sig`.
    ///
    /// **Absent ⇒ no expiry.** Never "expired", and never a defaulted window: the same signed field
    /// a verifier checks the price shape against also bounds how long the price stands.
    pub valid_until: Option<TimestampMs>,
    /// Key 5 — the detached signature by [`Tariff::identity`].
    pub sig: Vec<u8>,
}

impl Tariff {
    /// The §18.9 preimage body: `det_cbor(Tariff ∖ {5})` — key 5 removed from the map entirely,
    /// never set to `null`.
    pub fn signing_body(&self) -> Vec<u8> {
        cbor::encode(&self.body_cv())
    }

    fn body_cv(&self) -> Cv {
        let mut m: Vec<(u64, Cv)> = vec![
            (1, Cv::U64(self.suite.as_u8() as u64)),
            (2, Cv::Bytes(self.identity.clone())),
            (3, Cv::Bytes(self.schedule.0.clone())),
        ];
        if let Some(t) = self.valid_until {
            m.push((4, Cv::U64(t)));
        }
        Cv::Map(m)
    }

    pub(crate) fn to_cv(&self) -> Cv {
        let mut m = match self.body_cv() {
            Cv::Map(m) => m,
            _ => unreachable!("body_cv always returns a map"),
        };
        m.push((5, Cv::Bytes(self.sig.clone())));
        Cv::Map(m)
    }

    /// Encode to the §18.8a.1 wire bytes.
    pub fn det_cbor(&self) -> Vec<u8> {
        cbor::encode(&self.to_cv())
    }

    /// Build and sign a tariff. `identity` is taken from the signer, because the object is
    /// self-certifying and a mismatch would only ever produce a tariff that fails its own
    /// verification.
    pub fn sign(
        signer: &sig::Signer<'_>,
        schedule: DetCbor,
        valid_until: Option<TimestampMs>,
    ) -> Tariff {
        let mut t = Tariff {
            suite: signer.suite(),
            identity: signer.public(),
            schedule,
            valid_until,
            sig: Vec::new(),
        };
        t.sig = signer.sign(TARIFF_DS, &t.signing_body());
        t
    }

    /// Verify the signature against this tariff's **own** identity (§18.9).
    ///
    /// Deliberately does **not** check expiry: a client may need to verify that a *lapsed* tariff
    /// was genuinely the price at the time, which is a different question from whether it is the
    /// price now. Use [`Tariff::check_valid_at`] for the second.
    pub fn verify(&self) -> Result<(), CoordinatorError> {
        sig::verify(
            self.suite,
            &self.identity,
            TARIFF_DS,
            &self.signing_body(),
            &self.sig,
        )
    }

    /// Whether this tariff is still within its signed validity window at `now` (ms since the Unix
    /// epoch, supplied by the caller — this crate reads no clock).
    ///
    /// A tariff presented past `valid_until` MUST be treated as expired and fails closed (§18.8a.1,
    /// §26.10/§21). The boundary is inclusive: at exactly `valid_until` the tariff still stands, and
    /// it lapses strictly after.
    pub fn check_valid_at(&self, now: TimestampMs) -> Result<(), CoordinatorError> {
        match self.valid_until {
            Some(t) if now > t => Err(CoordinatorError::TariffExpired {
                valid_until: t,
                now,
            }),
            _ => Ok(()),
        }
    }

    /// Verify the signature **and** the validity window in one call — the entry point a client
    /// quoting a live price should use.
    pub fn verify_at(&self, now: TimestampMs) -> Result<(), CoordinatorError> {
        self.verify()?;
        self.check_valid_at(now)
    }

    /// Whether this tariff was signed by the same identity as the descriptor carrying it.
    ///
    /// `false` is **not** an error — §18.8a.1 explicitly permits a tariff signed by a different
    /// key. It is a fact a client MUST surface when comparing prices across coordinators, which is
    /// why it is a named predicate rather than a comparison a caller might forget to make.
    pub fn is_self_issued_by(&self, descriptor_identity: &[u8]) -> bool {
        self.identity == descriptor_identity
    }

    pub(crate) fn from_cv(cv: Cv) -> Result<Self, CoordinatorError> {
        let mut f = Fields::from_cv(cv)?;
        let suite = sig::require_suite(f.req(1)?)?;
        let identity = as_bytes(f.req(2)?)?;
        sig::require_len("identity", suite, &identity, Which::IkPub)?;
        let schedule = DetCbor(as_bytes(f.req(3)?)?);
        let valid_until = f.take(4).map(as_u64).transpose()?;
        let sig_bytes = as_bytes(f.req(5)?)?;
        sig::require_len("sig", suite, &sig_bytes, Which::SigVal)?;
        f.deny_unknown()?;
        Ok(Tariff {
            suite,
            identity,
            schedule,
            valid_until,
            sig: sig_bytes,
        })
    }

    /// Decode the §18.8a.1 wire bytes. **Does not verify the signature** — see
    /// [`Tariff::from_det_cbor_verified`], which is what untrusted input must go through.
    pub fn from_det_cbor(bytes: &[u8]) -> Result<Self, CoordinatorError> {
        Tariff::from_cv(cbor::decode(bytes)?)
    }

    /// Decode **and** verify. The only entry point whose result may be treated as authentic; a
    /// failure here is `ERR_ADAPTER_TARIFF_INVALID` (`0x0B01`) in an adapter context (§21.11a).
    pub fn from_det_cbor_verified(bytes: &[u8]) -> Result<Self, CoordinatorError> {
        let t = Tariff::from_det_cbor(bytes)?;
        t.verify()?;
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kotva_core::identity::IdentityKey;

    fn schedule() -> DetCbor {
        DetCbor::from_text_map([
            ("gb".to_string(), Cv::U64(3)),
            ("byte".to_string(), Cv::U64(5)),
        ])
    }

    #[test]
    fn signs_verifies_and_round_trips() {
        let ik = IdentityKey::from_seed(&[1u8; 32]);
        let t = Tariff::sign(&sig::Signer::Classical(&ik), schedule(), Some(1_700_000_000_000));
        t.verify().unwrap();
        let b = t.det_cbor();
        let back = Tariff::from_det_cbor_verified(&b).unwrap();
        assert_eq!(back, t);
        assert_eq!(back.det_cbor(), b, "re-encode must reproduce the bytes");
        assert_eq!(back.identity, ik.public());
    }

    #[test]
    fn absent_valid_until_means_no_expiry_not_expired() {
        let ik = IdentityKey::from_seed(&[2u8; 32]);
        let t = Tariff::sign(&sig::Signer::Classical(&ik), schedule(), None);
        assert_eq!(t.valid_until, None);
        // Far-future evaluation still passes: absence is not a zero-valued window.
        t.verify_at(u64::MAX).unwrap();
        // And key 4 is genuinely absent from the wire map, not present-as-zero.
        let decoded = cbor::decode(&t.det_cbor()).unwrap();
        match decoded {
            Cv::Map(m) => assert!(m.iter().all(|(k, _)| *k != 4)),
            _ => panic!("not a map"),
        }
    }

    #[test]
    fn an_expired_tariff_fails_closed_at_the_right_boundary() {
        let ik = IdentityKey::from_seed(&[3u8; 32]);
        let t = Tariff::sign(&sig::Signer::Classical(&ik), schedule(), Some(1_000));
        t.verify_at(999).unwrap();
        t.verify_at(1_000).unwrap(); // inclusive: it still stands AT valid_until
        assert_eq!(
            t.verify_at(1_001),
            Err(CoordinatorError::TariffExpired { valid_until: 1_000, now: 1_001 })
        );
        // Expiry is not a signature failure: the signature over a lapsed tariff still verifies,
        // which is what lets a client prove what the price *was*.
        t.verify().unwrap();
    }

    #[test]
    fn valid_until_is_covered_by_the_signature() {
        let ik = IdentityKey::from_seed(&[4u8; 32]);
        let mut t = Tariff::sign(&sig::Signer::Classical(&ik), schedule(), Some(1_000));
        t.verify().unwrap();
        // Extend the window without re-signing — the classic "make the old price last longer"
        // tamper. It must not verify.
        t.valid_until = Some(u64::MAX);
        assert_eq!(t.verify(), Err(CoordinatorError::BadSignature));
        // Removing the field entirely (unbounded price) must also fail.
        t.valid_until = None;
        assert_eq!(t.verify(), Err(CoordinatorError::BadSignature));
    }

    #[test]
    fn a_tamper_of_the_schedule_fails_verification() {
        let ik = IdentityKey::from_seed(&[5u8; 32]);
        let mut t = Tariff::sign(&sig::Signer::Classical(&ik), schedule(), None);
        t.schedule = DetCbor::from_text_map([("gb".to_string(), Cv::U64(1))]);
        assert_eq!(t.verify(), Err(CoordinatorError::BadSignature));
    }

    #[test]
    fn a_tariff_signed_by_another_key_is_still_verifiable_and_is_flagged_as_third_party() {
        let operator = IdentityKey::from_seed(&[6u8; 32]);
        let reseller = IdentityKey::from_seed(&[7u8; 32]);
        let t = Tariff::sign(&sig::Signer::Classical(&reseller), schedule(), None);
        // Self-certifying: it verifies against its own identity with no descriptor in sight.
        t.verify().unwrap();
        assert!(!t.is_self_issued_by(&operator.public()));
        assert!(t.is_self_issued_by(&reseller.public()));
    }

    #[test]
    fn the_pq_hybrid_suite_signs_and_verifies_over_the_composite_representative() {
        let hk = kotva_core::pq::HybridSigningKey::generate();
        let t = Tariff::sign(&sig::Signer::PqHybrid(&hk), schedule(), None);
        assert_eq!(t.suite, Suite::PqHybrid);
        assert_eq!(t.identity.len(), 1984);
        assert_eq!(t.sig.len(), 3373);
        let back = Tariff::from_det_cbor_verified(&t.det_cbor()).unwrap();
        assert_eq!(back, t);
        // Downgrade attempt: relabel it 0x01 and the length governance alone refuses it, before any
        // signature check gets a chance to be lenient.
        let mut downgraded = t.clone();
        downgraded.suite = Suite::Classical;
        assert!(matches!(
            Tariff::from_det_cbor(&downgraded.det_cbor()),
            Err(CoordinatorError::BadFieldLength { .. })
        ));
    }

    #[test]
    fn decode_is_not_verify_and_the_verified_entry_point_is_the_one_that_rejects() {
        let ik = IdentityKey::from_seed(&[8u8; 32]);
        let mut t = Tariff::sign(&sig::Signer::Classical(&ik), schedule(), None);
        t.sig[0] ^= 0xff;
        let bytes = t.det_cbor();
        // Decoding a forged tariff succeeds *on purpose*, so a client can report on it.
        let decoded = Tariff::from_det_cbor(&bytes).unwrap();
        assert_eq!(decoded, t);
        // The entry point that matters rejects it.
        assert_eq!(
            Tariff::from_det_cbor_verified(&bytes),
            Err(CoordinatorError::BadSignature)
        );
    }

    #[test]
    fn unknown_keys_and_missing_required_keys_fail_closed() {
        let ik = IdentityKey::from_seed(&[9u8; 32]);
        let t = Tariff::sign(&sig::Signer::Classical(&ik), schedule(), None);
        let mut m = match t.to_cv() {
            Cv::Map(m) => m,
            _ => unreachable!(),
        };
        m.push((9, Cv::U64(1)));
        assert!(matches!(
            Tariff::from_det_cbor(&cbor::encode(&Cv::Map(m))),
            Err(CoordinatorError::Cbor(kotva_core::cbor::CborError::UnknownKey(9)))
        ));
        // Drop the required signature.
        let m2: Vec<(u64, Cv)> = match t.to_cv() {
            Cv::Map(m) => m.into_iter().filter(|(k, _)| *k != 5).collect(),
            _ => unreachable!(),
        };
        assert!(matches!(
            Tariff::from_det_cbor(&cbor::encode(&Cv::Map(m2))),
            Err(CoordinatorError::Cbor(kotva_core::cbor::CborError::MissingKey(5)))
        ));
    }
}
