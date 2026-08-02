//! The four elementals, the backing modes, and the two derived properties
//! (`profiles/cloud.md` §1.2, §3, §3.6).
//!
//! The registry is **four rows and is meant to stay four**. Anything that composes from them is a
//! [`crate::formula::DepotFormula`], never a variant added here.

use crate::DepotError;

/// A DEPOT elemental — one of exactly four irreducible resources (§3).
///
/// The split that generates these and no others is **bytes vs code × cold vs hot**:
///
/// |          | stateless / cold | stateful / hot |
/// |----------|------------------|----------------|
/// | bytes    | [`Service::Bucket`] | [`Service::Volume`] |
/// | code     | [`Service::EdgeFn`] | [`Service::Box`] |
///
/// Deliberately **not** `#[non_exhaustive]`: the set is closed at four, and forcing consumers into a
/// wildcard match arm would invite exactly the silent catch-all handling this profile rejects. A
/// fifth mechanism, if one ever existed, is a breaking change and should read as one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Service {
    /// Object storage; also serves public objects at the edge (the CDN fold, §3.7). Adopts the S3
    /// API and CID content-addressing, plus HTTP caching when public-serving.
    Bucket,
    /// Block storage — the **hot tier**, not a slow bucket (§3.3). Adopts virtio-blk / NVMe-oF /
    /// iSCSI with a guest-owned filesystem.
    Volume,
    /// Serverless compute. Adopts WASI / OCI. A hosted-inference endpoint is this, with
    /// `artifact-source = operator` (§3.7).
    EdgeFn,
    /// A managed node. Adopts an OS plus cloud-init. A machine with an accelerator is still a box
    /// (§3.1) — `gpu`/`tpu` is never a service.
    Box,
}

impl Service {
    /// The wire string (§3.3 `DepotServicePolicy` key 1).
    pub fn as_str(self) -> &'static str {
        match self {
            Service::Bucket => "bucket",
            Service::Volume => "volume",
            Service::EdgeFn => "edge-fn",
            Service::Box => "box",
        }
    }

    /// Parse a wire string. An unrecognised service **fails closed** (`None`) — a consumer MUST NOT
    /// guess at a near-match, because "there are exactly four" is the property the whole profile
    /// rests on (§3).
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "bucket" => Service::Bucket,
            "volume" => Service::Volume,
            "edge-fn" => Service::EdgeFn,
            "box" => Service::Box,
            _ => return None,
        })
    }

    /// Every elemental, in registry order.
    pub const ALL: [Service; 4] = [
        Service::Bucket,
        Service::Volume,
        Service::EdgeFn,
        Service::Box,
    ];

    /// Whether this elemental can be **zero-migration** portable at all (§3, DEPOT-4).
    ///
    /// `bucket` (re-pin) and `edge-fn` (redeploy the artefact) can; `volume` and `box` carry
    /// single-writer state and are export/import **always** — there is no configuration under which
    /// they become zero-migration, which is why this is a property of the service and not of an
    /// instance. A `detachable` volume is **not** a counter-example (§3.3).
    pub fn can_be_zero_migration(self) -> bool {
        matches!(self, Service::Bucket | Service::EdgeFn)
    }
}

/// Who owns the implementation behind a coordinator (§1.2) — a **declared, first-class fact**,
/// because it changes both the trust story and the exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backing {
    /// The gateway's own hardware, or a cloud it rents and resells (DEPOT-6). The gateway bills the
    /// user; the exit is an export/import or a re-pin.
    Operator,
    /// **Bring-your-own** — the user's own account at some provider, operated by the gateway under a
    /// delegated, attenuated, unilaterally revocable credential (DEPOT-7). The underlying provider
    /// bills the user directly; the exit is **revoking the credential**, because the bytes never
    /// left the user's account.
    Customer,
    /// Some parts each; a formula's `Part` keys say which (§3.6).
    Mixed,
}

impl Backing {
    /// The wire string (§3.3 `DepotServicePolicy` key 2).
    pub fn as_str(self) -> &'static str {
        match self {
            Backing::Operator => "operator",
            Backing::Customer => "customer",
            Backing::Mixed => "mixed",
        }
    }

    /// Parse a wire string. CLOSED set — an unrecognised value fails closed (§3.3).
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "operator" => Backing::Operator,
            "customer" => Backing::Customer,
            "mixed" => Backing::Mixed,
            _ => return None,
        })
    }

    /// Whether the **exit** is a credential revocation rather than a data migration (§1.2).
    ///
    /// True only for [`Backing::Customer`]. This is the one structural advantage of BYO backing —
    /// and note it says nothing about *visibility*: a gateway holding a read-capable credential is
    /// `terminating` regardless of who owns the account (DEPOT-7).
    pub fn exit_is_revocation(self) -> bool {
        matches!(self, Backing::Customer)
    }
}

/// The visibility **class** an operator declares (CONTRACT §3.1, restated per-service in §3).
///
/// Ordered by exposure, least-exposed first, so [`Ord`] gives the "least-blind" comparison the
/// formula inheritance rule needs (§3.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VisibilityClass {
    /// Holds ciphertext it has no key to read.
    Blind,
    /// Sees which object and when, never a private payload — a public-serving bucket.
    BlindRouting,
    /// Sees the data or the computation.
    Terminating,
}

/// The assurance **level** backing a visibility claim (CONTRACT §3.3).
///
/// Ordered by strength, strongest first, so [`Ord`] again gives "least-blind" for inheritance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Assurance {
    /// Excluded by construction — the operator cannot read it even if it wanted to.
    Structural,
    /// A TEE with **verifiable remote attestation** the client can actually check. If the client
    /// cannot check it, this reverts to [`Assurance::Declared`] (DEPOT-2).
    Attested,
    /// A disclosed trust boundary: the operator can read it and says so.
    Declared,
}

/// A declared visibility — one class at one assurance level (CONTRACT §2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Visibility {
    /// What the operator can see.
    pub class: VisibilityClass,
    /// What backs the claim that it sees only that.
    pub assurance: Assurance,
}

impl Visibility {
    /// A visibility from its class and assurance level.
    pub const fn new(class: VisibilityClass, assurance: Assurance) -> Self {
        Visibility { class, assurance }
    }

    /// `blind` / `structural` — client-encrypted bytes the operator cannot read.
    pub const BLIND_STRUCTURAL: Visibility =
        Visibility::new(VisibilityClass::Blind, Assurance::Structural);
    /// `blind` / `declared` — a bucket or volume handed **plaintext**. The operator's declaration
    /// stays truthful; the blindness was the client's to supply and it did not (CONTRACT §3.3).
    pub const BLIND_DECLARED: Visibility =
        Visibility::new(VisibilityClass::Blind, Assurance::Declared);
    /// `blind-routing` / `structural` — a public-serving bucket.
    pub const BLIND_ROUTING: Visibility =
        Visibility::new(VisibilityClass::BlindRouting, Assurance::Structural);
    /// `terminating` / `declared` — the honest default for a box or an edge-fn.
    pub const TERMINATING: Visibility =
        Visibility::new(VisibilityClass::Terminating, Assurance::Declared);
    /// `terminating` / `attested` — a TEE whose attestation the client verified.
    pub const TERMINATING_ATTESTED: Visibility =
        Visibility::new(VisibilityClass::Terminating, Assurance::Attested);

    /// Whether it is honest to describe this as "blind", "private" or "sovereign" to a user.
    ///
    /// **Only** a structurally-blind service qualifies. Advertising anything else that way is
    /// non-conformant misrepresentation, not marketing (DEPOT-2, CONTRACT §3.2) — which is why this
    /// is a function in the reference implementation and not a matter of taste.
    pub fn may_be_called_private(self) -> bool {
        self.class == VisibilityClass::Blind && self.assurance == Assurance::Structural
    }
}

/// How hard it is to leave (§3, DEPOT-4). Ordered least-portable **last**, so [`Ord`] gives the
/// inheritance rule's "least-portable of the parts".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Portability {
    /// Content-addressed or stateless: re-pin or redeploy elsewhere, no migration.
    ZeroMigration,
    /// Stateful: a real export in the adopted standard's own interchange format, ingestible by a
    /// different operator **without the exporting operator's cooperation** (DEPOT-4). May cost real
    /// downtime; that is an acceptable price, a format nobody else can read is not.
    ExportImport,
}

impl Service {
    /// The **most blind** visibility this elemental can honestly declare, given whether the client
    /// encrypted what it handed over and whether a bucket is serving public objects (§3).
    ///
    /// This is the honesty cliff in executable form. `bucket` and `volume` are blind **only for what
    /// the client actually encrypted** — both accept arbitrary bytes, so hand either one plaintext
    /// and it is readable while the operator's declaration stays truthful (CONTRACT §3.3).
    pub fn max_visibility(self, client_encrypted: bool, public_serving: bool) -> Visibility {
        match self {
            Service::Bucket => {
                if public_serving {
                    Visibility::BLIND_ROUTING
                } else if client_encrypted {
                    Visibility::BLIND_STRUCTURAL
                } else {
                    Visibility::BLIND_DECLARED
                }
            }
            Service::Volume => {
                if client_encrypted {
                    Visibility::BLIND_STRUCTURAL
                } else {
                    // An unencrypted volume is terminating, and the operator cannot tell which it
                    // was given (§3, CONTRACT §3.3).
                    Visibility::TERMINATING
                }
            }
            Service::EdgeFn | Service::Box => Visibility::TERMINATING,
        }
    }

    /// The **true** portability of this elemental (§3, DEPOT-4).
    pub fn portability(self) -> Portability {
        if self.can_be_zero_migration() {
            Portability::ZeroMigration
        } else {
            Portability::ExportImport
        }
    }
}

/// Check a declared visibility against what the elemental's data model actually permits (DEPOT-2).
///
/// Returns [`DepotError::VisibilityOverclaim`] when an operator declares itself **more blind** than
/// its own mechanism allows — the one rule in this profile that is misrepresentation rather than
/// policy. Declaring *less* blindness than permitted is always allowed: an operator may be more
/// conservative about itself than the protocol requires.
pub fn check_visibility(
    service: Service,
    declared: Visibility,
    client_encrypted: bool,
    public_serving: bool,
) -> Result<(), DepotError> {
    let permitted = service.max_visibility(client_encrypted, public_serving);
    // `Ord` on (class, assurance) runs least-exposed -> most-exposed, so "more blind than permitted"
    // is strictly less than the permitted bound.
    if declared < permitted {
        return Err(DepotError::VisibilityOverclaim {
            service,
            declared,
            permitted,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_round_trips_and_unknown_fails_closed() {
        for s in Service::ALL {
            assert_eq!(Service::from_str(s.as_str()), Some(s));
        }
        // Near-misses a catalogue-minded implementer might coin. Each MUST fail closed rather than
        // resolve to something plausible (§3).
        for bad in ["boxes", "vm", "gpu", "cdn", "database", "queue", "Box", ""] {
            assert_eq!(Service::from_str(bad), None, "{bad} must not parse");
        }
    }

    #[test]
    fn backing_unknown_fails_closed() {
        assert_eq!(Backing::from_str("customer"), Some(Backing::Customer));
        assert_eq!(Backing::from_str("byo"), None);
        assert!(Backing::Customer.exit_is_revocation());
        assert!(!Backing::Operator.exit_is_revocation());
        assert!(!Backing::Mixed.exit_is_revocation());
    }

    #[test]
    fn only_structural_blindness_may_be_called_private() {
        assert!(Visibility::BLIND_STRUCTURAL.may_be_called_private());
        // The sharpest self-deception risk in the profile: a bucket handed plaintext is still
        // declared honestly, and is still not private (§8).
        assert!(!Visibility::BLIND_DECLARED.may_be_called_private());
        assert!(!Visibility::BLIND_ROUTING.may_be_called_private());
        assert!(!Visibility::TERMINATING.may_be_called_private());
        // A TEE narrows the exposure; it does not erase the operator's original access to
        // plaintext-in-use, so it still may not be sold as blindness (§8).
        assert!(!Visibility::TERMINATING_ATTESTED.may_be_called_private());
    }

    #[test]
    fn bucket_blindness_is_the_clients_property_not_the_operators() {
        assert_eq!(
            Service::Bucket.max_visibility(true, false),
            Visibility::BLIND_STRUCTURAL
        );
        assert_eq!(
            Service::Bucket.max_visibility(false, false),
            Visibility::BLIND_DECLARED
        );
        // Public-serving wins regardless of encryption: it sees which object and when.
        assert_eq!(
            Service::Bucket.max_visibility(true, true),
            Visibility::BLIND_ROUTING
        );
    }

    #[test]
    fn unencrypted_volume_is_terminating() {
        assert_eq!(
            Service::Volume.max_visibility(true, false),
            Visibility::BLIND_STRUCTURAL
        );
        assert_eq!(
            Service::Volume.max_visibility(false, false),
            Visibility::TERMINATING
        );
    }

    #[test]
    fn box_and_edge_fn_are_never_blind() {
        for s in [Service::Box, Service::EdgeFn] {
            for enc in [true, false] {
                assert_eq!(s.max_visibility(enc, false), Visibility::TERMINATING);
            }
        }
    }

    #[test]
    fn overclaiming_visibility_is_rejected() {
        // A box advertised as blind — the exact misrepresentation DEPOT-2 names.
        assert!(matches!(
            check_visibility(Service::Box, Visibility::BLIND_STRUCTURAL, false, false),
            Err(DepotError::VisibilityOverclaim { .. })
        ));
        // A plaintext bucket advertised as structurally blind.
        assert!(matches!(
            check_visibility(Service::Bucket, Visibility::BLIND_STRUCTURAL, false, false),
            Err(DepotError::VisibilityOverclaim { .. })
        ));
        // Honest declarations pass.
        check_visibility(Service::Box, Visibility::TERMINATING, false, false).unwrap();
        check_visibility(Service::Bucket, Visibility::BLIND_STRUCTURAL, true, false).unwrap();
        // Declaring *less* blindness than permitted is always allowed.
        check_visibility(Service::Bucket, Visibility::TERMINATING, true, false).unwrap();
    }

    #[test]
    fn stateful_elementals_are_never_zero_migration() {
        assert_eq!(Service::Bucket.portability(), Portability::ZeroMigration);
        assert_eq!(Service::EdgeFn.portability(), Portability::ZeroMigration);
        assert_eq!(Service::Volume.portability(), Portability::ExportImport);
        assert_eq!(Service::Box.portability(), Portability::ExportImport);
    }
}
