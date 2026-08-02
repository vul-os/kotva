//! **The §18.8a coordinator-layer wire objects** — [`CoordinatorDescriptor`], [`Tariff`],
//! [`UsageReceipt`].
//!
//! The reference implementation of [§18.8a of the wire
//! format](https://github.com/vul-os/kotva/blob/main/18-wire-format.md), which is itself the wire
//! form of the two MUSTs in
//! [`coordinator/CONTRACT.md`](https://github.com/vul-os/kotva/blob/main/coordinator/CONTRACT.md):
//! §2.1 ("publish a **signed descriptor** carrying its kind, its policy, and — where it charges — a
//! signed tariff") and §6 ("issue… signed usage receipts delivered directly to the paying party").
//!
//! # Why this crate exists
//!
//! Both objects were normative for several spec revisions with no implementation in this repo. What
//! implementations existed lived *inside a consumer* — an economics crate in a broker — and
//! `CoordinatorDescriptor`, the object the CDDL names, existed as no struct anywhere: it was
//! modelled as an unsigned body type plus a separate signed wrapper, so the wire object had no
//! single name in code. A spec-defined object with exactly one implementation is how "conformant"
//! degenerates into "behaves like that implementation" (`profiles/cloud.md` §8), because a second
//! implementer has nothing to check against except the first one's source.
//!
//! This crate is therefore the *reference* encoding, in the repo that holds the prose, over the
//! frozen deterministic-CBOR corpus (`kotva-cbor`) that the rest of the object family already
//! shares. It is codecs and closed registries — no transport, no async runtime, no I/O, **no
//! clock**: expiry is a function of a caller-supplied `now`, so a verification verdict is
//! reproducible rather than a function of when the test ran.
//!
//! # One object family for every coordinator kind
//!
//! §18.8a is emphatic that this is **one** family keyed by [`CoordinatorKind`] (key 2), not one
//! shape per kind. A `gateway`'s domain/modes/attestation-selector (§7.5) and a legacy adapter's
//! rail/mode/initiation-class (§26.3.1) are kind-specific facts and live in the **opaque**
//! [`CoordinatorDescriptor::policy`] blob. `kotva-depot`'s `DepotServicePolicy` is exactly that
//! blob for `kind = "infra-service"`; `tests/vectors.rs` round-trips a real one through a real
//! descriptor rather than leaving the claim as prose in two crates that never met.
//!
//! # What is signed
//!
//! | Object | sig key | DS-tag | Preimage body |
//! |---|---:|---|---|
//! | [`CoordinatorDescriptor`] | 7 | `DMTAP-COORD-v0/descriptor` | `det_cbor(descriptor ∖ {7})` |
//! | [`Tariff`] | 5 | `DMTAP-COORD-v0/tariff` | `det_cbor(tariff ∖ {5})` |
//! | [`UsageReceipt`] | 4 | `DMTAP-COORD-v0/usage-receipt` | `det_cbor(receipt ∖ {4})` |
//!
//! Every one carries `suite` at key 1, so the §18.9 **family form** applies: the representative is
//! `DS-tag ‖ 0x00 ‖ body` under the single-component suite `0x01` and
//! `DS-tag ‖ 0x00 ‖ u8(suite) ‖ body` under a composite suite (`0x02`–`0x05`). [`sig`] implements
//! that rule once, for all three objects — see [`sig::preimage_body`].
//!
//! [`Tariff`] and [`UsageReceipt`] are **self-certifying**: each carries its own signer identity
//! and verifies standalone, without a live descriptor fetch, and a client MUST attribute a tariff
//! to `Tariff.identity` rather than to the enclosing descriptor's (§18.8a.1).
//!
//! # Decode is not verify (deliberate, and the sharp edge)
//!
//! [`CoordinatorDescriptor::from_det_cbor`] decodes and shape-checks; it does **not** check a
//! signature, so a caller can inspect and report on an object that fails verification instead of
//! silently dropping it. Untrusted bytes MUST go through
//! [`CoordinatorDescriptor::from_det_cbor_verified`] (and the [`Tariff`]/[`UsageReceipt`]
//! equivalents), which decode **and** verify and are the only entry points that may be treated as
//! authentic. Every error in [`CoordinatorError`] is a hard reject; none is a degraded accept.
//!
//! # Honest residual, restated from CONTRACT §6 (normative disclosure)
//!
//! A verified [`UsageReceipt`] proves the coordinator signed a claim about one real operation. It
//! is **one-directional**: it cannot disconfirm an operation the coordinator fabricated or silently
//! omitted, and a client MUST NOT present the absence of a disputed receipt as proof that the
//! operation never happened (§18.8a.2).
//!
//! # Not implemented here
//!
//! §18.8a.3 `GatewayAuthz` is **not** in this crate. It is gateway-local state, carries no
//! signature of its own, is never mesh-transmitted, and its authenticity comes from the
//! `Assertion`/`CapabilityToken` that populated it — a different thing from the three
//! mesh-published, self-certifying objects here. Stated so its absence reads as a scope boundary
//! rather than as coverage.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod blob;
pub mod descriptor;
pub mod kind;
pub mod receipt;
pub mod sig;
pub mod tariff;
pub mod visibility;

pub use blob::DetCbor;
pub use descriptor::CoordinatorDescriptor;
pub use kind::CoordinatorKind;
pub use receipt::UsageReceipt;
pub use sig::{Signer, DESCRIPTOR_DS, TARIFF_DS, USAGE_RECEIPT_DS};
pub use tariff::Tariff;
pub use visibility::{AssuranceLevel, Visibility, VisibilityClass};

/// The `Headers.mime` a `UsageReceipt` MUST carry on its `0x0A` system MOTE (§18.8a.2).
///
/// `kind = 0x0A` is shared with capability announcements (§10.2) and bounce notices (§7.10.3a), so
/// the `Body` shape alone is ambiguous: a receiver MUST inspect `Headers.mime` **before** parsing a
/// `0x0A` body, and MUST treat an unrecognised `mime` as an undecodable system message rather than
/// guessing at one of the three shapes.
pub const USAGE_RECEIPT_MIME: &str = "application/vnd.dmtap.usage-receipt+cbor";

/// Order two text map keys the way canonical CBOR does (§18.1.1, RFC 8949 §4.2.1).
///
/// Deterministic CBOR sorts map entries by their **encoded key bytes**, and a text string's head
/// encodes its length monotonically — so the effective order is **length first, then bytewise**,
/// never plain lexicographic. `"gb"` therefore precedes `"byte"`, which a naive `sort()` gets
/// backwards; the resulting bytes are not merely unusual, they are **rejected** by the strict
/// decoder (`CborError::MapKeyOrder`), so the mistake shows up as an interop failure at a third
/// party rather than as a local test failure.
///
/// This matters here because the three opaque blobs (`policy`, `schedule`, `operation`) are exactly
/// where an implementer hand-builds a text-keyed map. [`DetCbor::from_text_map`] uses this; it is
/// the same rule and the same reasoning as `kotva_depot::canonical_key_cmp`.
pub(crate) fn canonical_key_cmp(a: &str, b: &str) -> core::cmp::Ordering {
    a.len().cmp(&b.len()).then_with(|| a.as_bytes().cmp(b.as_bytes()))
}

/// Failures this crate defines. **Every variant is a hard reject** — a caller MUST treat any error
/// here as "not verified" and MUST NOT present the value as authentic (fail closed, §18.1.2).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CoordinatorError {
    /// A det_cbor decode or shape failure, surfaced from `kotva_core::cbor`. Includes the
    /// unknown-key rejection §18.8a.1 relies on: "a decoder MUST reject an unknown key (§18.1.2)
    /// exactly so a future field cannot smuggle [a score, a price rank, a stake] back in without a
    /// version bump a verifier would notice."
    #[error("cbor: {0}")]
    Cbor(#[from] kotva_core::cbor::CborError),

    /// A value outside a **closed** registry — an unknown `kind`, visibility `class`, or assurance
    /// `level`. A consumer MUST NOT map it onto a similar-sounding known value: §18.8a.1 requires
    /// an unknown `kind` be treated as an *undeclared* coordinator, and an unrecognised visibility
    /// sub-field be "rejected, not defaulted".
    #[error("unknown {registry} value {value:?} — closed registry, fails closed (§18.8a.1)")]
    UnknownRegistryValue {
        /// Which registry rejected it.
        registry: &'static str,
        /// The offending value.
        value: String,
    },

    /// A registered suite id this crate has no verification path for (`0x03`–`0x05` are reserved
    /// and unimplemented across the whole family), or an unregistered byte.
    ///
    /// §18.2 is explicit that an implementation which does not support a suite MUST reject
    /// fail-closed rather than guess — and MUST NOT read that permission as licence to originate
    /// `0x01` (`ERR_SUITE_BELOW_FLOOR`, `0x0125`).
    #[error("unsupported/unknown signature suite {0:#04x} — fail closed (§18.1.4, §18.2)")]
    UnsupportedSuite(u8),

    /// A suite-governed byte string of the wrong length (§18.2): `ik-pub` or `sig-val` whose size
    /// does not match the row for the object's own `suite`. A wrong length under a composite suite
    /// is how a stripped PQ component looks on the wire, which is why it is a decode-boundary
    /// reject and not a verification-time surprise.
    #[error("{field} is {actual} B but suite {suite:#04x} requires {expected} B (§18.2)")]
    BadFieldLength {
        /// `"identity"` or `"sig"`.
        field: &'static str,
        /// The object's declared suite.
        suite: u8,
        /// The length §18.2 requires.
        expected: usize,
        /// What was on the wire.
        actual: usize,
    },

    /// A visibility declaration §18.8a.1 forbids: class `terminating` at assurance `structural`.
    ///
    /// "There is no `structural` assurance for a plaintext-terminating role" — a role that sees the
    /// data cannot also claim the protocol structurally prevents it from seeing the data. See
    /// [`Visibility::check`] for the divergence this rule carries in the spec text and exactly how
    /// far this crate enforces it.
    #[error("visibility {class:?}/{level:?} is not declarable: a terminating role has no \
             structural assurance (§18.8a.1, CONTRACT §3.3)")]
    UndeclarableVisibility {
        /// The declared class.
        class: VisibilityClass,
        /// The declared assurance level.
        level: AssuranceLevel,
    },

    /// The signature did not verify against the object's own `identity`.
    ///
    /// For a `Tariff` presented in an adapter context this is `ERR_ADAPTER_TARIFF_INVALID`
    /// (`0x0B01`); for a `UsageReceipt`, `ERR_ADAPTER_RECEIPT_INVALID` (`0x0B02`) — §21.11a, or the
    /// kind-appropriate equivalent elsewhere.
    #[error("signature does not verify against the object's own identity (§18.9, fail closed)")]
    BadSignature,

    /// A [`Tariff`] presented past its own signed `valid_until` (§18.8a.1, §26.10/§21).
    ///
    /// **Absent `valid_until` means no expiry**, never "expired" and never "valid forever pending a
    /// default" — the field is either signed into the object or it does not constrain it.
    #[error("tariff expired: valid_until={valid_until} ms, now={now} ms (§18.8a.1, §26.10)")]
    TariffExpired {
        /// The signed expiry.
        valid_until: kotva_core::TimestampMs,
        /// The caller-supplied evaluation time.
        now: kotva_core::TimestampMs,
    },
}
