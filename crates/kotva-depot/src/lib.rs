//! **DEPOT** — the cloud elementals and the control-plane vocabulary that provisions them.
//!
//! The reference implementation of [`profiles/cloud.md`](https://github.com/vul-os/kotva/blob/main/profiles/cloud.md).
//!
//! # What this crate is, and what it deliberately is not
//!
//! DEPOT mints **no wire object**. Every schema here is an integer-keyed deterministic-CBOR map
//! (§18.1.2) that rides a carrier that already exists:
//!
//! | Schema | Rides |
//! |---|---|
//! | [`policy::DepotServicePolicy`] | the §18.8a `CoordinatorDescriptor.policy` opaque blob |
//! | [`image::DepotImage`] | a §22 `PubAnnounce` under `meta["depot-image"]` |
//! | [`formula::DepotFormula`] | a §22 `PubAnnounce` under `meta["depot-formula"]` |
//! | [`site::DepotSite`] | a §22 `PubAnnounce` under `meta["depot-site"]` |
//! | [`measurement::DepotMeasurement`] | an ATTEST claim body, schema `kotva-depot/measurement/v0` |
//!
//! and the control plane ([`control`]) is a **vocabulary** scoping the existing
//! [`kotva_core::capability::CapabilityToken`], not a new protocol. So this crate has no transport,
//! no async runtime, and no I/O. It is codecs, closed registries, and the three derivations the spec
//! insists are "computable rather than rhetorical":
//!
//! 1. **Visibility honesty** — [`service::check_visibility`] rejects an operator declaring itself
//!    more blind than its own mechanism permits (DEPOT-2, the profile's one misrepresentation rule).
//! 2. **Formula inheritance** — [`formula::DepotFormula::derive`] computes a composed service's
//!    visibility and portability as the least-blind and least-portable of its parts (§3.6), so a
//!    client can *check* rather than trust a claim.
//! 3. **Control attenuation** — [`control::ResourceRef::covers`] decides whether one capability
//!    scope contains another, and [`control::Ability`] is a **closed** registry whose unknown values
//!    fail closed (§5.2) — the fix for `Capability`'s free-text `resource`/`ability`, which would
//!    otherwise let two conformant gateways diverge *silently*.
//!
//! # The four elementals
//!
//! [`service::Service`] is `bucket` / `volume` / `edge-fn` / `box` and is meant to stay four.
//! A database, a queue, a CDN, an image registry, a static site and a hosted-inference endpoint are
//! all [`formula::DepotFormula`]s composing them — never variants (§3.6, §3.7).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod control;
pub mod formula;
pub mod image;
pub mod measurement;
pub mod policy;
pub mod service;
pub mod site;

pub use control::{
    check_coordinator_binding, check_operator_offering, Ability, ResourceRef, CAVEAT_COORDINATOR,
};
pub use formula::{DepotFormula, DerivedProperties, Part};
pub use image::{DepotImage, ImageFormat, ImageTarget};
pub use measurement::{DepotMeasurement, EvidenceKind, Method, Metric};
pub use policy::{Capacity, DepotServicePolicy};
pub use service::{
    check_visibility, Assurance, Backing, Portability, Service, Visibility, VisibilityClass,
};
pub use site::DepotSite;

/// Order two text map keys the way canonical CBOR does (§18.1.1, RFC 8949 §4.2.1).
///
/// Deterministic CBOR sorts map entries by their **encoded key bytes**, and a text string's head
/// encodes its length monotonically — so the effective order is **length first, then bytewise**, not
/// lexicographic. `kotva_cbor` re-sorts on encode regardless; matching it here is what keeps a
/// decoded struct's `Vec` order identical to the one that was encoded, so `T == decode(encode(T))`
/// holds for the text maps (`attributes`, `resources`, `boot`) as well as for the bytes.
pub(crate) fn canonical_key_cmp(a: &str, b: &str) -> core::cmp::Ordering {
    a.len().cmp(&b.len()).then_with(|| a.as_bytes().cmp(b.as_bytes()))
}

/// The `meta` key carrying a [`DepotFormula`] on a §22 `PubAnnounce` (§3.6, §21.20 registry).
pub const META_FORMULA: &str = "depot-formula";
/// The `meta` key carrying a [`DepotSite`] on a §22 `PubAnnounce` (§3.6).
pub const META_SITE: &str = "depot-site";
/// The `meta` key carrying a [`DepotImage`] on a §22 `PubAnnounce` (§4.1).
pub const META_IMAGE: &str = "depot-image";
/// The ATTEST `SchemaRef` for a [`DepotMeasurement`] claim body (§7).
pub const MEASUREMENT_SCHEMA: &str = "kotva-depot/measurement/v0";

/// Failures this profile defines. Every variant is a **fail-closed** outcome (DEPOT-9): a refusal,
/// never a silent best-effort or a guessed-at near-match.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum DepotError {
    /// An operator declared itself **more blind** than its data model permits — the one rule in this
    /// profile that is misrepresentation rather than policy (DEPOT-2, CONTRACT §3.2).
    #[error(
        "visibility overclaim: {service:?} declared {declared:?} but its mechanism permits at best \
         {permitted:?} (DEPOT-2)"
    )]
    VisibilityOverclaim {
        /// The elemental whose declaration was checked.
        service: Service,
        /// What the operator claimed.
        declared: Visibility,
        /// The most-blind honest claim available to it.
        permitted: Visibility,
    },

    /// A value outside a **closed** registry — an unknown `service`, `backing`, `ability`,
    /// `format`, `metric`, `method`, or evidence `kind`.
    ///
    /// A consumer MUST NOT map this onto a similar-sounding known value. Silent aliasing is exactly
    /// how two conformant implementations diverge without either noticing (§5.2).
    #[error("unknown {registry} value {value:?} — closed registry, fails closed (§5.2)")]
    UnknownRegistryValue {
        /// Which registry rejected it.
        registry: &'static str,
        /// The offending value.
        value: String,
    },

    /// A malformed resource reference (§5.1). The grammar is `depot:<service>/<instance>`,
    /// `depot:<service>/*`, or `depot:*`.
    #[error("malformed resource reference {0:?} — expected depot:<service>/<instance>, \
             depot:<service>/*, or depot:* (§5.1)")]
    MalformedResource(String),

    /// A [`DepotFormula`] with no parts. A formula is a composition; a composition of nothing has no
    /// visibility and no portability to inherit (§3.6).
    #[error("formula {kind:?} has no parts — nothing to inherit visibility or portability from (§3.6)")]
    EmptyFormula {
        /// The formula's declared `kind`.
        kind: String,
    },

    /// A formula advertising horizontal scaling without naming what provides coordination.
    ///
    /// `box` + `volume` + `bucket` gives the *ingredients* of a scalable database, never the
    /// *coordination* — and no engine gets consensus from object storage for free (§3.6).
    #[error("formula {kind:?} claims horizontal scaling but declares no `consensus` — an absent \
             consensus field means single-writer (§3.6)")]
    ScalingWithoutConsensus {
        /// The formula's declared `kind`.
        kind: String,
    },

    /// An **operator's offering** exposes `destroy` while withholding `export` (§5.2, DEPOT-4).
    ///
    /// Raised only by [`control::check_operator_offering`], never for a delegated grant: a token may
    /// legitimately carry `destroy` alone (a CI job reaping preview environments), and requiring the
    /// pair per-token would turn cleanup credentials into exfiltration credentials.
    #[error("operator offers `destroy` without `export` — the account holder could delete an \
             instance but never extract it (§5.2, DEPOT-4)")]
    DestroyWithoutExport,

    /// A capability lacking a `depot:coordinator` caveat, or carrying one naming a different
    /// coordinator (§5.1).
    ///
    /// A resource string names no operator, so an unbound token is valid at *every* DEPOT
    /// coordinator — the confused-deputy hole this closes.
    #[error("capability is not bound to this coordinator: `depot:coordinator` caveat absent or \
             naming another key (§5.1)")]
    CoordinatorBindingMissing,

    /// A det_cbor decode or shape failure, surfaced from `kotva_core::cbor`.
    #[error("cbor: {0}")]
    Cbor(#[from] kotva_core::cbor::CborError),
}
