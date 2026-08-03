//! [`Visibility`] — key 4 of a `CoordinatorDescriptor`: exactly one declared class at one
//! assurance level (§18.8a.1; CONTRACT §2.4, §3.1, §3.3).
//!
//! ```cddl
//! Visibility = {
//!   1 => tstr,   ; class  "blind" / "blind-routing" / "terminating"
//!   2 => tstr,   ; level  "structural" / "attested" / "declared"
//! }
//! ```
//!
//! Both sub-fields are **closed** registries: §18.8a.1 requires that "an unrecognized value in
//! either sub-field MUST be rejected, not defaulted (fail-closed, mirrors §18.1.2's unknown-key
//! rule)". [`VisibilityClass::from_str`] and [`AssuranceLevel::from_str`] therefore return `Option`
//! and the decoder turns `None` into a hard reject.
//!
//! This type is deliberately **not** a re-export of `kotva_depot::Visibility`. That one is the
//! DEPOT profile's *derivation* vocabulary — `Ord`-ordered so formula inheritance can compute the
//! least-blind of a composition (`profiles/cloud.md` §3.6) — and its ordering is a property of that
//! computation, not of the wire. This one is the §18.8a.1 wire field. They agree string-for-string,
//! and `tests/vectors.rs` pins that agreement so the two cannot drift apart unnoticed.

use kotva_core::cbor::{self, as_text, Cv, Fields};

use crate::CoordinatorError;

/// What a coordinator can see (CONTRACT §3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VisibilityClass {
    /// Holds ciphertext it has no key to read.
    Blind,
    /// Sees which object and when, never a private payload.
    BlindRouting,
    /// Sees the data or the computation.
    Terminating,
}

impl VisibilityClass {
    /// The wire string (§18.8a.1 `Visibility` key 1).
    pub fn as_str(self) -> &'static str {
        match self {
            VisibilityClass::Blind => "blind",
            VisibilityClass::BlindRouting => "blind-routing",
            VisibilityClass::Terminating => "terminating",
        }
    }

    /// Parse a wire string. CLOSED set — an unrecognised value fails closed (`None`), never
    /// defaults to the most-permissive or the most-restrictive class.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "blind" => VisibilityClass::Blind,
            "blind-routing" => VisibilityClass::BlindRouting,
            "terminating" => VisibilityClass::Terminating,
            _ => return None,
        })
    }

    /// Every class, in registry order.
    pub const ALL: [VisibilityClass; 3] = [
        VisibilityClass::Blind,
        VisibilityClass::BlindRouting,
        VisibilityClass::Terminating,
    ];
}

/// What backs the claim that a coordinator sees only that (CONTRACT §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AssuranceLevel {
    /// The role *has no key* — the **protocol**, not the client's discipline, makes reading
    /// impossible. Strongest, and provable.
    Structural,
    /// The role runs in a **TEE** whose remote attestation proves the code only forwards and holds
    /// no key. Hardware-trust. If the client cannot actually check the attestation, the honest
    /// level is [`AssuranceLevel::Declared`].
    Attested,
    /// The operator *promises*; nothing structurally prevents cheating. Honest-trust — the level a
    /// disclosed trust boundary carries.
    Declared,
}

impl AssuranceLevel {
    /// The wire string (§18.8a.1 `Visibility` key 2).
    pub fn as_str(self) -> &'static str {
        match self {
            AssuranceLevel::Structural => "structural",
            AssuranceLevel::Attested => "attested",
            AssuranceLevel::Declared => "declared",
        }
    }

    /// Parse a wire string. CLOSED set — an unrecognised value fails closed (`None`).
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "structural" => AssuranceLevel::Structural,
            "attested" => AssuranceLevel::Attested,
            "declared" => AssuranceLevel::Declared,
            _ => return None,
        })
    }

    /// Every level, in registry order.
    pub const ALL: [AssuranceLevel; 3] = [
        AssuranceLevel::Structural,
        AssuranceLevel::Attested,
        AssuranceLevel::Declared,
    ];
}

/// Exactly one declared class at one assurance level (§18.8a.1 key 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Visibility {
    /// What the operator can see.
    pub class: VisibilityClass,
    /// What backs the claim that it sees only that.
    pub level: AssuranceLevel,
}

impl Visibility {
    /// A visibility from its class and level. Does **not** check declarability — see
    /// [`Visibility::check`].
    pub const fn new(class: VisibilityClass, level: AssuranceLevel) -> Self {
        Visibility { class, level }
    }

    /// `blind` / `structural` — the relay's declaration: MOTE sealing is mandatory (§2), so the
    /// role provably has no key for anything it will ever carry.
    pub const BLIND_STRUCTURAL: Visibility =
        Visibility::new(VisibilityClass::Blind, AssuranceLevel::Structural);
    /// `blind-routing` / `structural` — an SNI-passthrough reachability adapter, or a media relay
    /// that published `sframe_required = true`.
    pub const BLIND_ROUTING: Visibility =
        Visibility::new(VisibilityClass::BlindRouting, AssuranceLevel::Structural);
    /// `terminating` / `declared` — the honest default wherever a legacy leg, a matcher, an arbiter
    /// or an oracle sees plaintext.
    pub const TERMINATING: Visibility =
        Visibility::new(VisibilityClass::Terminating, AssuranceLevel::Declared);
    /// `terminating` / `attested` — a role that sees the data, running in a TEE the client verified.
    pub const TERMINATING_ATTESTED: Visibility =
        Visibility::new(VisibilityClass::Terminating, AssuranceLevel::Attested);

    /// Whether it is honest to describe this to a user as "blind", "private" or "sovereign".
    ///
    /// **Only** a structurally-blind role qualifies. An attested TEE narrows the exposure; it does
    /// not erase the role's original access to plaintext-in-use, so it still may not be sold as
    /// blindness (CONTRACT §3.2, `profiles/cloud.md` §8, DEPOT-2). A function rather than a matter
    /// of taste, for the same reason it is one in `kotva-depot`.
    pub fn may_be_called_private(self) -> bool {
        self.class == VisibilityClass::Blind && self.level == AssuranceLevel::Structural
    }

    /// Reject a declaration §18.8a.1 forbids: **`terminating` at `structural`**.
    ///
    /// A role that sees the data cannot simultaneously claim the protocol structurally prevents it
    /// from seeing the data — "there is no `structural` assurance for a plaintext-terminating role"
    /// (§18.8a.1, CONTRACT §3.3). Applied at the decode boundary, because unlike DEPOT's
    /// `check_visibility` this needs no facts the wire object lacks: it is a contradiction internal
    /// to the two bytes on the wire, not a comparison against a mechanism.
    ///
    /// # A spec-internal divergence this function decides (disclosed, not papered over)
    ///
    /// §18.8a.1's cell states the constraint twice and the two statements are not the same rule:
    ///
    /// 1. the CDDL comment and prose rationale — "there is no `structural` assurance for a
    ///    plaintext-terminating role, and `declared` is the honest-trust level for it"; and
    /// 2. the literal sentence — "A `terminating` class MUST declare `level = "declared"`".
    ///
    /// Reading (2) literally also forbids **`terminating`/`attested`**, which three other normative
    /// places explicitly permit: CONTRACT §5 gives `matcher` as "**terminating** (always — the
    /// class), optionally **attested** (the assurance level, §3.3, via TEE)" and `indexer` as
    /// "query-channel `terminating` unless `attested`"; `profiles/cloud.md` §5 gives
    /// `edge-fn`/`box` as "`terminating` (→ `attested` in a TEE)"; and `kotva_depot::Visibility`
    /// ships `TERMINATING_ATTESTED` as a first-class constant its own `check_visibility` accepts.
    /// Under the literal reading, a DEPOT `infra-service` coordinator running a box in a verified
    /// TEE could not publish a conformant descriptor at all — the strictly-stricter reading is not
    /// the safer one here, it is an interop break against the profile that carries this crate's
    /// own `policy` blob.
    ///
    /// Rationale (1) is what both readings agree on and is the substance of the rule, so this
    /// function enforces exactly that: `terminating` MUST NOT be `structural`; `declared` and
    /// `attested` are both declarable. **The spec text has been amended to match** (see the
    /// §18.8a.1 `visibility` row) rather than left contradicting its reference implementation —
    /// which is the failure mode this whole crate exists to prevent.
    pub fn check(self) -> Result<(), CoordinatorError> {
        if self.class == VisibilityClass::Terminating && self.level == AssuranceLevel::Structural {
            return Err(CoordinatorError::UndeclarableVisibility {
                class: self.class,
                level: self.level,
            });
        }
        Ok(())
    }

    /// The §18.8a.1 `Visibility` map.
    pub(crate) fn to_cv(self) -> Cv {
        Cv::Map(vec![
            (1, Cv::Text(self.class.as_str().to_string())),
            (2, Cv::Text(self.level.as_str().to_string())),
        ])
    }

    /// Decode the §18.8a.1 `Visibility` map, failing closed on an unrecognised sub-field, an
    /// unknown key, or an undeclarable combination.
    pub(crate) fn from_cv(cv: Cv) -> Result<Self, CoordinatorError> {
        let mut f = Fields::from_cv(cv)?;
        let class_s = as_text(f.req(1)?)?;
        let class =
            VisibilityClass::from_str(&class_s).ok_or(CoordinatorError::UnknownRegistryValue {
                registry: "visibility-class",
                value: class_s,
            })?;
        let level_s = as_text(f.req(2)?)?;
        let level =
            AssuranceLevel::from_str(&level_s).ok_or(CoordinatorError::UnknownRegistryValue {
                registry: "assurance-level",
                value: level_s,
            })?;
        f.deny_unknown()?;
        let v = Visibility { class, level };
        v.check()?;
        Ok(v)
    }

    /// Encode standalone (for tests and for a caller embedding the shape elsewhere).
    pub fn det_cbor(self) -> Vec<u8> {
        cbor::encode(&self.to_cv())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_sub_registries_round_trip_and_fail_closed() {
        for c in VisibilityClass::ALL {
            assert_eq!(VisibilityClass::from_str(c.as_str()), Some(c));
        }
        for l in AssuranceLevel::ALL {
            assert_eq!(AssuranceLevel::from_str(l.as_str()), Some(l));
        }
        // Plausible near-misses an implementer might coin. §18.8a.1: "rejected, not defaulted".
        for bad in [
            "blind_routing",
            "blindrouting",
            "Blind",
            "opaque",
            "terminal",
            "",
        ] {
            assert_eq!(VisibilityClass::from_str(bad), None, "{bad:?}");
        }
        for bad in ["structured", "attest", "Declared", "promised", "tee", ""] {
            assert_eq!(AssuranceLevel::from_str(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn terminating_structural_is_the_one_undeclarable_pair() {
        // Exhaustive over the 3x3 product: exactly one cell is refused, and every other cell is
        // declarable. Enumerating rather than spot-checking is what would catch a `check` that
        // accidentally refused a whole row or column.
        let mut refused = Vec::new();
        for class in VisibilityClass::ALL {
            for level in AssuranceLevel::ALL {
                if Visibility::new(class, level).check().is_err() {
                    refused.push((class, level));
                }
            }
        }
        assert_eq!(
            refused,
            vec![(VisibilityClass::Terminating, AssuranceLevel::Structural)]
        );
    }

    #[test]
    fn terminating_attested_is_permitted_and_that_is_a_decided_divergence() {
        // See `Visibility::check`'s doc comment. The literal §18.8a.1 sentence ("a terminating
        // class MUST declare declared") would refuse this pair, but CONTRACT §5 (matcher, indexer),
        // profiles/cloud.md §5 (edge-fn/box in a TEE) and kotva_depot::Visibility::TERMINATING_
        // ATTESTED all permit it. This test pins the decision so it cannot be quietly reversed.
        Visibility::TERMINATING_ATTESTED.check().unwrap();
        Visibility::TERMINATING.check().unwrap();
        // And the substance both readings share is still enforced.
        assert!(
            Visibility::new(VisibilityClass::Terminating, AssuranceLevel::Structural)
                .check()
                .is_err()
        );
    }

    #[test]
    fn only_structural_blindness_may_be_called_private() {
        assert!(Visibility::BLIND_STRUCTURAL.may_be_called_private());
        assert!(!Visibility::BLIND_ROUTING.may_be_called_private());
        assert!(!Visibility::TERMINATING.may_be_called_private());
        // The sharpest self-deception risk: a TEE is not blindness.
        assert!(!Visibility::TERMINATING_ATTESTED.may_be_called_private());
        assert!(
            !Visibility::new(VisibilityClass::Blind, AssuranceLevel::Declared)
                .may_be_called_private()
        );
    }

    #[test]
    fn unknown_key_in_the_visibility_map_is_rejected() {
        // §18.1.2's unknown-key rule applies to the nested map too, not only the outer object.
        let bad = cbor::encode(&Cv::Map(vec![
            (1, Cv::Text("blind".into())),
            (2, Cv::Text("structural".into())),
            (3, Cv::Text("smuggled".into())),
        ]));
        assert!(matches!(
            Visibility::from_cv(cbor::decode(&bad).unwrap()),
            Err(CoordinatorError::Cbor(
                kotva_core::cbor::CborError::UnknownKey(3)
            ))
        ));
    }

    #[test]
    fn round_trips_through_det_cbor() {
        for v in [
            Visibility::BLIND_STRUCTURAL,
            Visibility::BLIND_ROUTING,
            Visibility::TERMINATING,
            Visibility::TERMINATING_ATTESTED,
        ] {
            let b = v.det_cbor();
            assert_eq!(Visibility::from_cv(cbor::decode(&b).unwrap()).unwrap(), v);
        }
    }
}
