//! The §18.9 signing seam shared by all three §18.8a objects: DS-tags, the suite-aware message
//! representative, the suite-governed byte-string lengths of §18.2, and verification.
//!
//! # The family form (§18.9, normative, governs all of §18.9)
//!
//! ```text
//! single-component (0x01)        : M  = DS-tag ‖ 0x00 ‖ body
//! composite (0x02/0x03/0x04/0x05): M' = DS-tag ‖ 0x00 ‖ u8(suite) ‖ body
//! ```
//!
//! where `body` is `det_cbor(object ∖ {sig})` — the signature field **removed from the map
//! entirely**, never set to `null`. Every §18.8a object carries `suite` at key 1 precisely so the
//! two implementations of a composite suite can agree on which representative was signed:
//! "without it two implementations of a composite suite cannot agree on the preimage" (§18.8a.1).
//!
//! A composite signer MUST sign `M'` with **both** components over the **same** `M'`, and a
//! verifier MUST reconstruct the representative matching the object's `suite` and MUST NOT accept a
//! component that verifies only against the other form (§18.1.6). [`preimage_body`] is the single
//! implementation of that rule in this crate; sign and verify both go through it, so they cannot
//! drift into signing one form and checking the other.
//!
//! # Suites this crate can originate
//!
//! `0x01` (Ed25519) and `0x02` (Ed25519 ∧ ML-DSA-65, AND-composed, no intra-suite strip). `0x03`,
//! `0x04` and `0x05` are **registered-but-unimplemented reserved code points** across the whole
//! family — `kotva_core::Suite` recognises them so they round-trip a decoder, but no verifier
//! exists for any of them, so this crate rejects them fail-closed (§18.2: "An implementation that
//! does not yet support a suite MUST reject objects in it fail-closed… rather than guessing").
//!
//! **§1.1/§18.1.4 mark `0x01` LEGACY — verify-only, MUST NOT originate; `0x02` is the v0 REQUIRED
//! originating suite.** This crate implements both paths, so an originator can and should use
//! [`Signer::PqHybrid`]; the `0x01` path exists to verify what already exists on the wire. Nothing
//! here prevents a caller originating `0x01`, and that choice is the caller's non-conformance
//! (`ERR_SUITE_BELOW_FLOOR`, `0x0125`), not something this crate can decide for it.

use kotva_core::cbor::Cv;
use kotva_core::identity::{verify_domain, IdentityKey};
use kotva_core::pq::{verify_hybrid_domain, HybridSigningKey};
use kotva_core::Suite;

use crate::CoordinatorError;

/// DS-tag for `CoordinatorDescriptor.sig` (§18.9 preimage table, §21.24h). ASCII, terminated by one
/// `0x00` byte — `sign_domain`/`verify_domain` prepend it, so the `msg` they receive is exactly the
/// preimage body.
pub const DESCRIPTOR_DS: &[u8] = b"DMTAP-COORD-v0/descriptor\x00";
/// DS-tag for `Tariff.sig` (§18.9 preimage table, §21.24h).
pub const TARIFF_DS: &[u8] = b"DMTAP-COORD-v0/tariff\x00";
/// DS-tag for `UsageReceipt.sig` (§18.9 preimage table, §21.24h).
pub const USAGE_RECEIPT_DS: &[u8] = b"DMTAP-COORD-v0/usage-receipt\x00";

/// The §18.9 message-representative body for `suite`.
///
/// Under a **composite** suite the `u8(suite)` byte is prepended to `body` **inside** the preimage,
/// after the DS-tag that `sign_domain` adds. Under the single-component suite `0x01` there is no
/// suite byte and the two forms coincide — which is exactly why the omission went unnoticed for so
/// long: every frozen vector in the family is `0x01` (§18.1.4), so nothing arbitrated.
pub fn preimage_body(suite: Suite, body: &[u8]) -> Vec<u8> {
    let mut p = Vec::with_capacity(1 + body.len());
    if suite != Suite::Classical {
        p.push(suite.as_u8());
    }
    p.extend_from_slice(body);
    p
}

/// The `(ik-pub, sig-val)` byte lengths §18.2 fixes for `suite` — normative, not forward planning.
///
/// | suite | `ik-pub` | `sig-val` |
/// |---|---:|---:|
/// | `0x01` Ed25519 | 32 | 64 |
/// | `0x02` Ed25519 ∧ ML-DSA-65 | 1984 | 3373 |
/// | `0x03` (identical layout to `0x02`; differs only in AEAD) | 1984 | 3373 |
/// | `0x04` Ed25519 ∧ SLH-DSA-128s (anchor) | 64 | 7920 |
/// | `0x05` (identical layout to `0x02`; differs only in hash) | 1984 | 3373 |
pub fn field_lengths(suite: Suite) -> (usize, usize) {
    match suite {
        Suite::Classical => (32, 64),
        Suite::ReservedAnchorSlhDsa => (64, 7920),
        // 0x02, and 0x03/0x05 which share its byte layout exactly.
        _ => (1984, 3373),
    }
}

/// Whether this crate has a real verification path for `suite`.
///
/// Only `0x01` and `0x02`. A `true` here is a claim that a forged signature under that suite would
/// be **rejected**, not merely that the bytes parse.
pub fn is_verifiable(suite: Suite) -> bool {
    matches!(suite, Suite::Classical | Suite::PqHybrid)
}

/// Decode a `suite` field (key 1 on every §18.8a object), failing closed on an unregistered byte or
/// on a registered suite this crate cannot verify.
pub(crate) fn require_suite(cv: Cv) -> Result<Suite, CoordinatorError> {
    let b = kotva_core::cbor::as_u8(cv)?;
    match Suite::from_u8(b) {
        Some(s) if is_verifiable(s) => Ok(s),
        // A registered-but-unimplemented reserved code point is a real gap, not a decode error —
        // but the outcome is the same reject, because accepting it would mean carrying an object
        // whose signature nothing in this process can check.
        _ => Err(CoordinatorError::UnsupportedSuite(b)),
    }
}

/// Enforce §18.2's suite-governed length on an `ik-pub` / `sig-val` field at the **decode
/// boundary**.
///
/// A wrong length under a composite suite is what a stripped PQ component looks like on the wire
/// (`ERR_HYBRID_SUITE_INCOMPLETE`, `0x0210`), so checking it here rather than at verification time
/// means the object never exists in a half-trusted state in the first place.
pub(crate) fn require_len(
    field: &'static str,
    suite: Suite,
    bytes: &[u8],
    which: Which,
) -> Result<(), CoordinatorError> {
    let (ik, sig) = field_lengths(suite);
    let expected = match which {
        Which::IkPub => ik,
        Which::SigVal => sig,
    };
    if bytes.len() != expected {
        return Err(CoordinatorError::BadFieldLength {
            field,
            suite: suite.as_u8(),
            expected,
            actual: bytes.len(),
        });
    }
    Ok(())
}

/// Which §18.2 length row a field is governed by.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Which {
    /// An `ik-pub` identity key.
    IkPub,
    /// A `sig-val` detached signature.
    SigVal,
}

/// A signing key together with the suite it originates under.
///
/// A borrowing enum rather than a trait object: the two suites have genuinely different key types
/// (32 B Ed25519 vs. a 1984 B hybrid pair) and there are exactly two, so a closed enum keeps the
/// "which suite did this object get signed under" question answerable by reading the call site.
pub enum Signer<'a> {
    /// Suite `0x01`, Ed25519. **LEGACY — verify-only per §1.1**; originating under it is
    /// `ERR_SUITE_BELOW_FLOOR` (`0x0125`) and the caller's choice, not this crate's.
    Classical(&'a IdentityKey),
    /// Suite `0x02`, Ed25519 ∧ ML-DSA-65 — the v0 REQUIRED originating suite (§1.1).
    PqHybrid(&'a HybridSigningKey),
}

impl Signer<'_> {
    /// The suite this signer originates under — what lands in key 1 of the signed object.
    pub fn suite(&self) -> Suite {
        match self {
            Signer::Classical(_) => Suite::Classical,
            Signer::PqHybrid(_) => Suite::PqHybrid,
        }
    }

    /// The public key this signer's signatures verify against — what lands in the object's
    /// `identity` field, since all three objects are self-certifying.
    pub fn public(&self) -> Vec<u8> {
        match self {
            Signer::Classical(k) => k.public(),
            Signer::PqHybrid(k) => k.public(),
        }
    }

    /// Sign the §18.9 representative for this signer's suite: `ds ‖ 0x00 ‖ [u8(suite)] ‖ body`.
    pub fn sign(&self, ds: &[u8], body: &[u8]) -> Vec<u8> {
        let pre = preimage_body(self.suite(), body);
        match self {
            Signer::Classical(k) => k.sign_domain(ds, &pre),
            Signer::PqHybrid(k) => k.sign_domain(ds, &pre),
        }
    }
}

/// Verify a §18.8a signature against the object's own declared `suite` and `identity`.
///
/// Fails closed on every path: an unsupported suite, a wrong-length key or signature, a bad
/// signature, and — under `0x02` — a hybrid whose PQ half is missing, stripped or tampered
/// (AND-composition, never accepted on the classical half alone).
pub(crate) fn verify(
    suite: Suite,
    identity: &[u8],
    ds: &[u8],
    body: &[u8],
    sig: &[u8],
) -> Result<(), CoordinatorError> {
    let pre = preimage_body(suite, body);
    match suite {
        Suite::Classical => {
            verify_domain(identity, ds, &pre, sig).map_err(|_| CoordinatorError::BadSignature)
        }
        Suite::PqHybrid => verify_hybrid_domain(identity, ds, &pre, sig)
            .map_err(|_| CoordinatorError::BadSignature),
        other => Err(CoordinatorError::UnsupportedSuite(other.as_u8())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ds_tags_are_the_spec_strings_and_are_nul_terminated() {
        // §18.9's preimage table and §21.24h's registry. A DS-tag typo produces signatures that
        // verify perfectly against themselves and against nothing else in the world, so the exact
        // bytes are asserted rather than assumed.
        assert_eq!(DESCRIPTOR_DS, b"DMTAP-COORD-v0/descriptor\x00");
        assert_eq!(TARIFF_DS, b"DMTAP-COORD-v0/tariff\x00");
        assert_eq!(USAGE_RECEIPT_DS, b"DMTAP-COORD-v0/usage-receipt\x00");
        for ds in [DESCRIPTOR_DS, TARIFF_DS, USAGE_RECEIPT_DS] {
            assert_eq!(*ds.last().unwrap(), 0x00, "DS-tag must be NUL-terminated");
            assert_eq!(
                ds.iter().filter(|b| **b == 0x00).count(),
                1,
                "exactly one NUL, at the end"
            );
        }
        // All three distinct: a signature over one object must never replay as another.
        assert_ne!(DESCRIPTOR_DS, TARIFF_DS);
        assert_ne!(TARIFF_DS, USAGE_RECEIPT_DS);
        assert_ne!(DESCRIPTOR_DS, USAGE_RECEIPT_DS);
    }

    #[test]
    fn the_composite_preimage_inserts_the_suite_byte_and_the_classical_one_does_not() {
        let body = b"body".to_vec();
        assert_eq!(preimage_body(Suite::Classical, &body), b"body".to_vec());
        assert_eq!(preimage_body(Suite::PqHybrid, &body), b"\x02body".to_vec());
        assert_eq!(
            preimage_body(Suite::ReservedAnchorSlhDsa, &body),
            b"\x04body".to_vec()
        );
    }

    #[test]
    fn field_lengths_match_the_normative_18_2_table() {
        // These five rows are §18.2's, transcribed. They are asserted rather than merely written
        // down because 0x03/0x04/0x05 have no reachable code path in this crate, so nothing else
        // would ever notice a transcription error.
        assert_eq!(field_lengths(Suite::Classical), (32, 64));
        assert_eq!(field_lengths(Suite::PqHybrid), (1984, 3373));
        assert_eq!(field_lengths(Suite::ReservedAeadGcm), (1984, 3373));
        assert_eq!(field_lengths(Suite::ReservedAnchorSlhDsa), (64, 7920));
        assert_eq!(field_lengths(Suite::ReservedHashSha3), (1984, 3373));
        // The 0x02 numbers are the component sums, not magic constants.
        assert_eq!(32 + 1952, 1984);
        assert_eq!(64 + 3309, 3373);
        // And 0x04's, from FIPS 205 Table 2.
        assert_eq!(32 + 32, 64);
        assert_eq!(64 + 7856, 7920);
    }

    #[test]
    fn only_0x01_and_0x02_are_verifiable_and_the_rest_fail_closed() {
        assert!(is_verifiable(Suite::Classical));
        assert!(is_verifiable(Suite::PqHybrid));
        for s in [
            Suite::ReservedAeadGcm,
            Suite::ReservedAnchorSlhDsa,
            Suite::ReservedHashSha3,
        ] {
            assert!(
                !is_verifiable(s),
                "{s:?} has no verifier and must not claim one"
            );
            assert_eq!(
                require_suite(Cv::U64(s.as_u8() as u64)),
                Err(CoordinatorError::UnsupportedSuite(s.as_u8()))
            );
        }
        // An unregistered byte, and the reserved-zero byte.
        assert_eq!(
            require_suite(Cv::U64(0x00)),
            Err(CoordinatorError::UnsupportedSuite(0))
        );
        assert_eq!(
            require_suite(Cv::U64(0xff)),
            Err(CoordinatorError::UnsupportedSuite(0xff))
        );
        // A suite field that is not even a u8 is a type error, not a suite error.
        assert!(matches!(
            require_suite(Cv::Text("0x01".into())),
            Err(CoordinatorError::Cbor(_))
        ));
    }

    #[test]
    fn a_classical_signature_does_not_verify_against_the_composite_representative() {
        // §18.1.6: a verifier MUST NOT accept a component that verifies only against the other
        // form. Signing the 0x01 representative and checking it as 0x02 must fail — this is the
        // check that a lazily-shared `preimage_body` would silently defeat.
        let ik = IdentityKey::from_seed(&[7u8; 32]);
        let body = b"the body".to_vec();
        let sig = ik.sign_domain(DESCRIPTOR_DS, &preimage_body(Suite::Classical, &body));
        verify(Suite::Classical, &ik.public(), DESCRIPTOR_DS, &body, &sig).unwrap();
        // Same bytes, re-checked under the composite rule: rejected (also on length grounds, which
        // is itself the §18.2 no-strip property).
        assert_eq!(
            verify(Suite::PqHybrid, &ik.public(), DESCRIPTOR_DS, &body, &sig),
            Err(CoordinatorError::BadSignature)
        );
        // And a signature made over the *composite* representative must not verify as classical.
        let composite_sig = ik.sign_domain(DESCRIPTOR_DS, &preimage_body(Suite::PqHybrid, &body));
        assert_eq!(
            verify(
                Suite::Classical,
                &ik.public(),
                DESCRIPTOR_DS,
                &body,
                &composite_sig
            ),
            Err(CoordinatorError::BadSignature)
        );
    }

    #[test]
    fn a_signature_under_one_ds_tag_does_not_verify_under_another() {
        let ik = IdentityKey::from_seed(&[9u8; 32]);
        let body = b"same body".to_vec();
        let sig = ik.sign_domain(TARIFF_DS, &body);
        verify(Suite::Classical, &ik.public(), TARIFF_DS, &body, &sig).unwrap();
        for other in [DESCRIPTOR_DS, USAGE_RECEIPT_DS] {
            assert_eq!(
                verify(Suite::Classical, &ik.public(), other, &body, &sig),
                Err(CoordinatorError::BadSignature),
                "cross-object replay must fail"
            );
        }
    }

    #[test]
    fn signer_reports_its_own_suite_and_key() {
        let ik = IdentityKey::from_seed(&[3u8; 32]);
        let s = Signer::Classical(&ik);
        assert_eq!(s.suite(), Suite::Classical);
        assert_eq!(s.public(), ik.public());
        assert_eq!(s.public().len(), 32);
        let sig = s.sign(TARIFF_DS, b"x");
        assert_eq!(sig.len(), 64);
        verify(Suite::Classical, &s.public(), TARIFF_DS, b"x", &sig).unwrap();
    }

    #[test]
    fn length_governance_rejects_a_stripped_or_padded_field() {
        require_len("identity", Suite::Classical, &[0u8; 32], Which::IkPub).unwrap();
        require_len("sig", Suite::Classical, &[0u8; 64], Which::SigVal).unwrap();
        assert!(matches!(
            require_len("identity", Suite::Classical, &[0u8; 31], Which::IkPub),
            Err(CoordinatorError::BadFieldLength {
                expected: 32,
                actual: 31,
                ..
            })
        ));
        // A 0x02 identity truncated to its Ed25519 half — the shape of a stripped PQ component.
        assert!(matches!(
            require_len("identity", Suite::PqHybrid, &[0u8; 32], Which::IkPub),
            Err(CoordinatorError::BadFieldLength {
                expected: 1984,
                actual: 32,
                ..
            })
        ));
    }
}
