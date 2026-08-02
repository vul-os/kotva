//! [`CoordinatorDescriptor`] — the discovery-only, self-asserted descriptor CONTRACT §2.1 requires
//! (§18.8a.1).
//!
//! ```cddl
//! CoordinatorDescriptor = {
//!   1 => suite,             ; suite       signature suite of `sig`
//!   2 => tstr,              ; kind        coordinator kind (CONTRACT §5 canonical table)
//!   3 => ik-pub,            ; identity    the coordinator's attested substrate identity
//!   4 => Visibility,        ; visibility  exactly one declared class + assurance level
//!   5 => bytes,             ; policy      opaque det_cbor operator policy, kind-specific
//!   ? 6 => Tariff,          ; tariff      OPTIONAL; present iff this coordinator charges
//!   7 => sig-val,           ; sig         over fields 1-6, DS-tag DMTAP-COORD-v0/descriptor
//! }
//! ```
//!
//! # What this object deliberately cannot say
//!
//! "By construction it has no field for a global score, a price rank, or a stake amount (§2.1) — a
//! decoder MUST reject an unknown key (§18.1.2) exactly so a future field cannot smuggle one back
//! in without a version bump a verifier would notice." That rejection is
//! [`Fields::deny_unknown`](kotva_core::cbor::Fields::deny_unknown) in [`Self::from_det_cbor`], and
//! it is the whole mechanism: the absence of a reputation field is only meaningful if an added one
//! fails to parse.
//!
//! # One shape for every kind
//!
//! A gateway's domain/modes/attestation-selector (§7.5) and an adapter's rail/mode/initiation-class
//! (§26.3.1) are kind-specific facts and live in the opaque [`Self::policy`] blob, exactly as they
//! do informally today. For `kind = "infra-service"` that blob is
//! `kotva_depot::DepotServicePolicy` (`profiles/cloud.md` §3.3); `tests/vectors.rs` proves a real
//! one survives the trip.
//!
//! # Publication transport is intentionally unspecified
//!
//! An operator's own domain, a directory, or a `pub_announce` (§22.3.1) under the operator's own
//! author feed are all conformant. This object specifies the **bytes**, not the channel — which is
//! why this crate has no I/O.

use kotva_core::cbor::{self, as_bytes, as_text, Cv, Fields};
use kotva_core::Suite;

use crate::sig::{self, Which, DESCRIPTOR_DS};
use crate::{CoordinatorError, CoordinatorKind, DetCbor, Tariff, Visibility};

/// A signed, discovery-only coordinator descriptor (§18.8a.1).
///
/// The signature is a field of this struct rather than of a separate wrapper type: the CDDL names
/// **one** object with `sig` at key 7, and splitting it into "body" and "signed body" types is how
/// the object ended up with no single name in the implementation that preceded this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordinatorDescriptor {
    /// Key 1 — the signature suite of [`Self::sig`], which pins the §18.9 representative.
    pub suite: Suite,
    /// Key 2 — the coordinator kind (CONTRACT §5).
    pub kind: CoordinatorKind,
    /// Key 3 — the coordinator's attested substrate identity. The descriptor is **self-certifying**:
    /// `sig` is verified against this same field, never an external identity.
    pub identity: Vec<u8>,
    /// Key 4 — exactly one declared class at one assurance level.
    pub visibility: Visibility,
    /// Key 5 — the opaque det_cbor operator policy. Self-asserted; §18 does not interpret it, and
    /// it is **never a ranking input**.
    pub policy: DetCbor,
    /// Key 6, OPTIONAL — a signed [`Tariff`], present **iff** this coordinator charges (CONTRACT
    /// §6). Absent ⇒ free.
    pub tariff: Option<Tariff>,
    /// Key 7 — the detached signature by [`Self::identity`].
    pub sig: Vec<u8>,
}

impl CoordinatorDescriptor {
    /// The §18.9 preimage body: `det_cbor(CoordinatorDescriptor ∖ {7})` — key 7 removed from the
    /// map entirely, never set to `null`.
    pub fn signing_body(&self) -> Vec<u8> {
        cbor::encode(&self.body_cv())
    }

    fn body_cv(&self) -> Cv {
        let mut m: Vec<(u64, Cv)> = vec![
            (1, Cv::U64(self.suite.as_u8() as u64)),
            (2, Cv::Text(self.kind.as_str().to_string())),
            (3, Cv::Bytes(self.identity.clone())),
            (4, self.visibility.to_cv()),
            (5, Cv::Bytes(self.policy.0.clone())),
        ];
        if let Some(t) = &self.tariff {
            m.push((6, t.to_cv()));
        }
        Cv::Map(m)
    }

    /// Encode to the §18.8a.1 wire bytes.
    pub fn det_cbor(&self) -> Vec<u8> {
        let mut m = match self.body_cv() {
            Cv::Map(m) => m,
            _ => unreachable!("body_cv always returns a map"),
        };
        m.push((7, Cv::Bytes(self.sig.clone())));
        cbor::encode(&Cv::Map(m))
    }

    /// Build and sign a descriptor.
    ///
    /// Rejects an undeclarable [`Visibility`] up front rather than minting a signed object no
    /// conformant verifier would accept — an operator finds out at publish time, not at a relying
    /// party. `identity` comes from the signer because the object is self-certifying.
    pub fn sign(
        signer: &sig::Signer<'_>,
        kind: CoordinatorKind,
        visibility: Visibility,
        policy: DetCbor,
        tariff: Option<Tariff>,
    ) -> Result<Self, CoordinatorError> {
        visibility.check()?;
        let mut d = CoordinatorDescriptor {
            suite: signer.suite(),
            kind,
            identity: signer.public(),
            visibility,
            policy,
            tariff,
            sig: Vec::new(),
        };
        d.sig = signer.sign(DESCRIPTOR_DS, &d.signing_body());
        Ok(d)
    }

    /// Verify the descriptor's own signature against its own declared identity (§18.9).
    ///
    /// **Does not verify an embedded [`Tariff`]** — that object is self-certifying and signed by a
    /// key that may differ from this descriptor's, so folding the two checks into one would let a
    /// caller believe a third party's tariff had been attributed to this coordinator. Use
    /// [`Self::verify_all`] to check both, and [`Tariff::is_self_issued_by`] to ask whose it is.
    pub fn verify(&self) -> Result<(), CoordinatorError> {
        sig::verify(
            self.suite,
            &self.identity,
            DESCRIPTOR_DS,
            &self.signing_body(),
            &self.sig,
        )
    }

    /// Verify this descriptor **and** its embedded tariff, if any.
    ///
    /// Still says nothing about *who* signed the tariff — that stays a separate, explicit question
    /// ([`Tariff::is_self_issued_by`]) because §18.8a.1 requires a client to surface the
    /// distinction when comparing prices, and a combined "all good" verdict is exactly what would
    /// hide it.
    pub fn verify_all(&self) -> Result<(), CoordinatorError> {
        self.verify()?;
        if let Some(t) = &self.tariff {
            t.verify()?;
        }
        Ok(())
    }

    /// Whether this coordinator charges — i.e. carries a tariff (CONTRACT §6). Absent ⇒ free.
    pub fn charges(&self) -> bool {
        self.tariff.is_some()
    }

    /// Decode the §18.8a.1 wire bytes. **Does not verify any signature** — see
    /// [`Self::from_det_cbor_verified`], which is what untrusted input must go through.
    ///
    /// Decoding is deliberately separable from verification so a client can *report on* a
    /// non-verifying descriptor rather than silently dropping it. It is not lenient: an unknown
    /// key, an unknown kind, an unrecognised visibility sub-field, an undeclarable
    /// class/level pair, an unsupported suite and a wrong-length key or signature are all hard
    /// rejects here, before any signature is considered.
    pub fn from_det_cbor(bytes: &[u8]) -> Result<Self, CoordinatorError> {
        let mut f = Fields::from_cv(cbor::decode(bytes)?)?;
        let suite = sig::require_suite(f.req(1)?)?;
        let kind = CoordinatorKind::require(&as_text(f.req(2)?)?)?;
        let identity = as_bytes(f.req(3)?)?;
        sig::require_len("identity", suite, &identity, Which::IkPub)?;
        let visibility = Visibility::from_cv(f.req(4)?)?;
        let policy = DetCbor(as_bytes(f.req(5)?)?);
        let tariff = f.take(6).map(Tariff::from_cv).transpose()?;
        let sig_bytes = as_bytes(f.req(7)?)?;
        sig::require_len("sig", suite, &sig_bytes, Which::SigVal)?;
        f.deny_unknown()?;
        Ok(CoordinatorDescriptor {
            suite,
            kind,
            identity,
            visibility,
            policy,
            tariff,
            sig: sig_bytes,
        })
    }

    /// Decode **and** verify the descriptor's own signature plus any embedded tariff's. The only
    /// entry point whose result may be treated as authentic.
    pub fn from_det_cbor_verified(bytes: &[u8]) -> Result<Self, CoordinatorError> {
        let d = CoordinatorDescriptor::from_det_cbor(bytes)?;
        d.verify_all()?;
        Ok(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kotva_core::identity::IdentityKey;

    fn policy() -> DetCbor {
        DetCbor::from_cv(&Cv::Map(vec![(1, Cv::Text("za-jhb".into()))]))
    }

    fn signed(seed: u8, kind: CoordinatorKind, v: Visibility) -> (IdentityKey, CoordinatorDescriptor) {
        let ik = IdentityKey::from_seed(&[seed; 32]);
        let d = CoordinatorDescriptor::sign(
            &sig::Signer::Classical(&ik),
            kind,
            v,
            policy(),
            None,
        )
        .unwrap();
        (ik, d)
    }

    #[test]
    fn signs_verifies_and_round_trips() {
        let (ik, d) = signed(21, CoordinatorKind::Relay, Visibility::BLIND_STRUCTURAL);
        d.verify().unwrap();
        let b = d.det_cbor();
        let back = CoordinatorDescriptor::from_det_cbor_verified(&b).unwrap();
        assert_eq!(back, d);
        assert_eq!(back.det_cbor(), b, "re-encode must reproduce the bytes");
        assert_eq!(back.identity, ik.public());
        assert!(!back.charges());
    }

    #[test]
    fn a_tariff_rides_in_key_6_and_stays_self_certifying() {
        let operator = IdentityKey::from_seed(&[22u8; 32]);
        let reseller = IdentityKey::from_seed(&[23u8; 32]);
        let t = Tariff::sign(
            &sig::Signer::Classical(&reseller),
            DetCbor::from_cv(&Cv::U64(42)),
            Some(9_000),
        );
        let d = CoordinatorDescriptor::sign(
            &sig::Signer::Classical(&operator),
            CoordinatorKind::Gateway,
            Visibility::TERMINATING,
            policy(),
            Some(t),
        )
        .unwrap();
        assert!(d.charges());
        d.verify_all().unwrap();
        let back = CoordinatorDescriptor::from_det_cbor_verified(&d.det_cbor()).unwrap();
        assert_eq!(back, d);
        // §18.8a.1: the tariff MAY, but need not, be the descriptor's own identity — and a client
        // MUST attribute it to the tariff's signer.
        let tariff = back.tariff.as_ref().unwrap();
        assert!(!tariff.is_self_issued_by(&back.identity));
        assert!(tariff.is_self_issued_by(&reseller.public()));
    }

    #[test]
    fn a_tampered_tariff_inside_a_valid_descriptor_is_caught_by_verify_all() {
        let operator = IdentityKey::from_seed(&[24u8; 32]);
        let t = Tariff::sign(
            &sig::Signer::Classical(&operator),
            DetCbor::from_cv(&Cv::U64(1)),
            None,
        );
        let mut d = CoordinatorDescriptor::sign(
            &sig::Signer::Classical(&operator),
            CoordinatorKind::Gateway,
            Visibility::TERMINATING,
            policy(),
            Some(t),
        )
        .unwrap();
        // Rewrite the price and re-sign the DESCRIPTOR only — the enclosing signature is then
        // perfectly valid over a tariff whose own signature is not.
        d.tariff.as_mut().unwrap().schedule = DetCbor::from_cv(&Cv::U64(999));
        d.sig = sig::Signer::Classical(&operator).sign(DESCRIPTOR_DS, &d.signing_body());
        d.verify().unwrap();
        assert_eq!(d.verify_all(), Err(CoordinatorError::BadSignature));
        assert_eq!(
            CoordinatorDescriptor::from_det_cbor_verified(&d.det_cbor()),
            Err(CoordinatorError::BadSignature)
        );
    }

    #[test]
    fn an_unknown_key_is_rejected_so_a_score_cannot_be_smuggled_in() {
        // The precise mechanism §2.1 relies on: with no reputation field defined, an added one must
        // fail to parse or the absence means nothing.
        let (_, d) = signed(25, CoordinatorKind::Indexer, Visibility::TERMINATING);
        let mut m = match cbor::decode(&d.det_cbor()).unwrap() {
            Cv::Map(m) => m,
            _ => unreachable!(),
        };
        m.push((8, Cv::U64(97))); // "reputation: 97"
        assert!(matches!(
            CoordinatorDescriptor::from_det_cbor(&cbor::encode(&Cv::Map(m))),
            Err(CoordinatorError::Cbor(kotva_core::cbor::CborError::UnknownKey(8)))
        ));
    }

    #[test]
    fn an_unknown_kind_is_rejected_rather_than_treated_as_a_near_match() {
        let (_, d) = signed(26, CoordinatorKind::InfraService, Visibility::BLIND_STRUCTURAL);
        let m: Vec<(u64, Cv)> = match cbor::decode(&d.det_cbor()).unwrap() {
            Cv::Map(m) => m
                .into_iter()
                .map(|(k, v)| if k == 2 { (k, Cv::Text("compute".into())) } else { (k, v) })
                .collect(),
            _ => unreachable!(),
        };
        assert!(matches!(
            CoordinatorDescriptor::from_det_cbor(&cbor::encode(&Cv::Map(m))),
            Err(CoordinatorError::UnknownRegistryValue { registry: "coordinator-kind", .. })
        ));
    }

    #[test]
    fn an_undeclarable_visibility_is_refused_at_signing_and_at_decoding() {
        let ik = IdentityKey::from_seed(&[27u8; 32]);
        let bad = Visibility::new(
            crate::VisibilityClass::Terminating,
            crate::AssuranceLevel::Structural,
        );
        assert!(matches!(
            CoordinatorDescriptor::sign(
                &sig::Signer::Classical(&ik),
                CoordinatorKind::Matcher,
                bad,
                policy(),
                None
            ),
            Err(CoordinatorError::UndeclarableVisibility { .. })
        ));
        // And a hand-built one on the wire, so refusing at signing is not the only defence.
        let (_, d) = signed(27, CoordinatorKind::Matcher, Visibility::TERMINATING);
        let m: Vec<(u64, Cv)> = match cbor::decode(&d.det_cbor()).unwrap() {
            Cv::Map(m) => m
                .into_iter()
                .map(|(k, v)| if k == 4 { (k, bad.to_cv()) } else { (k, v) })
                .collect(),
            _ => unreachable!(),
        };
        assert!(matches!(
            CoordinatorDescriptor::from_det_cbor(&cbor::encode(&Cv::Map(m))),
            Err(CoordinatorError::UndeclarableVisibility { .. })
        ));
    }

    #[test]
    fn every_field_the_cddl_marks_must_is_required() {
        let (_, d) = signed(28, CoordinatorKind::Labeler, Visibility::BLIND_ROUTING);
        for drop in [1u64, 2, 3, 4, 5, 7] {
            let m: Vec<(u64, Cv)> = match cbor::decode(&d.det_cbor()).unwrap() {
                Cv::Map(m) => m.into_iter().filter(|(k, _)| *k != drop).collect(),
                _ => unreachable!(),
            };
            assert!(
                CoordinatorDescriptor::from_det_cbor(&cbor::encode(&Cv::Map(m))).is_err(),
                "key {drop} is MUST-present"
            );
        }
        // Key 6 is the one that may be absent, and its absence is not an error.
        assert!(d.tariff.is_none());
        CoordinatorDescriptor::from_det_cbor(&d.det_cbor()).unwrap();
    }

    #[test]
    fn an_empty_policy_is_valid_nothing_declared() {
        let ik = IdentityKey::from_seed(&[29u8; 32]);
        let d = CoordinatorDescriptor::sign(
            &sig::Signer::Classical(&ik),
            CoordinatorKind::Oracle,
            Visibility::TERMINATING,
            DetCbor::empty(),
            None,
        )
        .unwrap();
        let back = CoordinatorDescriptor::from_det_cbor_verified(&d.det_cbor()).unwrap();
        assert!(back.policy.is_empty());
    }

    #[test]
    fn the_pq_hybrid_suite_round_trips_a_full_descriptor_with_a_tariff() {
        let hk = kotva_core::pq::HybridSigningKey::generate();
        let t = Tariff::sign(&sig::Signer::PqHybrid(&hk), DetCbor::from_cv(&Cv::U64(5)), None);
        let d = CoordinatorDescriptor::sign(
            &sig::Signer::PqHybrid(&hk),
            CoordinatorKind::ReachabilityAdapter,
            Visibility::BLIND_ROUTING,
            policy(),
            Some(t),
        )
        .unwrap();
        assert_eq!(d.suite, Suite::PqHybrid);
        let back = CoordinatorDescriptor::from_det_cbor_verified(&d.det_cbor()).unwrap();
        assert_eq!(back, d);
    }

    #[test]
    fn decode_is_not_verify() {
        let (_, mut d) = signed(30, CoordinatorKind::Arbiter, Visibility::TERMINATING_ATTESTED);
        d.sig[0] ^= 0xff;
        let bytes = d.det_cbor();
        assert_eq!(CoordinatorDescriptor::from_det_cbor(&bytes).unwrap(), d);
        assert_eq!(
            CoordinatorDescriptor::from_det_cbor_verified(&bytes),
            Err(CoordinatorError::BadSignature)
        );
    }
}
