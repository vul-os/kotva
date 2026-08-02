//! [`DepotFormula`] — everything that is not one of the four elementals (§3.6).
//!
//! A database, a queue, a CDN, an image registry, a static site, a hosted-inference endpoint: each
//! is a **recipe that composes the four**, never a new mechanism. A formula adds **no** `service`
//! value and **no** coordinator kind — it is a named schema over a content-addressed §22 object that
//! anyone may publish, fork and compete on. The protocol never learns what "Postgres" is.
//!
//! # The derivation this module exists for
//!
//! > *"A formula's visibility and portability are inherited, not declared."*
//!
//! [`DepotFormula::derive`] makes that **computable rather than rhetorical**: a client resolves each
//! part's signed descriptor and takes the least-blind visibility and least-portable class across
//! them. There is a concrete object to check, not a promise — and no way for a formula to be more
//! blind or more portable than its most-exposed, most-stuck part.

use kotva_core::cbor::{self, as_bytes, as_text, Cv, Fields};
use kotva_core::ContentId;

use crate::service::{Portability, Service, Visibility};
use crate::DepotError;

/// One primitive coordinator a formula composes (§3.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    /// Which elemental this part supplies.
    pub service: Service,
    /// The `infra-service` coordinator supplying it — its `IK` public key.
    ///
    /// Parts spanning **different** providers is legitimate and is exactly how a user avoids one
    /// operator holding the whole database; parts all naming **one** provider is a single-operator
    /// managed offering, and a client can see which from these keys.
    pub provider: Vec<u8>,
    /// Content address of that provider's `CoordinatorDescriptor` (§18.8a.1).
    pub descriptor: Option<ContentId>,
}

/// What a formula inherits from its parts (§3.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DerivedProperties {
    /// The **least-blind** visibility across the parts.
    pub visibility: Visibility,
    /// The **least-portable** class across the parts.
    pub portability: Portability,
    /// Whether every part names the same provider — a single-operator managed offering.
    pub single_operator: bool,
}

/// A composed service (§3.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepotFormula {
    /// Formula identity — `"postgres"`, `"redis"`, `"kafka-queue"`, … Free text on purpose: the
    /// protocol never learns what these mean, and competing formulas for one `kind` are the market.
    pub kind: String,
    /// The primitive coordinators it composes (at least one).
    pub parts: Vec<Part>,
    /// Opaque engine-defined provisioning/wiring.
    pub recipe: Option<Vec<u8>>,
    /// What provides coordination if it scales. **Absent means single-writer** (§3.6).
    pub consensus: Option<String>,
}

impl DepotFormula {
    /// A formula over the given parts.
    pub fn new(kind: impl Into<String>, parts: Vec<Part>) -> Self {
        DepotFormula {
            kind: kind.into(),
            parts,
            recipe: None,
            consensus: None,
        }
    }

    /// Derive inherited visibility and portability from the parts' declared properties (§3.6).
    ///
    /// `part_visibility` supplies each part's declared visibility, positionally. The caller obtains
    /// these by resolving each `Part.provider`'s signed descriptor — this crate does no I/O.
    ///
    /// Returns [`DepotError::EmptyFormula`] for a formula with no parts: a composition of nothing
    /// has nothing to inherit.
    pub fn derive(&self, part_visibility: &[Visibility]) -> Result<DerivedProperties, DepotError> {
        if self.parts.is_empty() || part_visibility.is_empty() {
            return Err(DepotError::EmptyFormula {
                kind: self.kind.clone(),
            });
        }
        // `Ord` on Visibility runs least-exposed -> most-exposed, so the max is the least blind.
        let visibility = *part_visibility
            .iter()
            .max()
            .expect("non-empty checked above");
        // Likewise Portability orders ZeroMigration < ExportImport, so max is least portable.
        let portability = self
            .parts
            .iter()
            .map(|p| p.service.portability())
            .max()
            .expect("non-empty checked above");
        let first = &self.parts[0].provider;
        let single_operator = self.parts.iter().all(|p| &p.provider == first);
        Ok(DerivedProperties {
            visibility,
            portability,
            single_operator,
        })
    }

    /// Check a horizontal-scaling claim (§3.6).
    ///
    /// `box` + `volume` + `bucket` gives the *ingredients* of a scalable database, never the
    /// *coordination*: two boxes cannot share one volume, and two boxes sharing a bucket still need
    /// something to decide which is primary. No engine gets consensus from object storage for free.
    /// A formula advertising horizontal scaling MUST name what provides the coordination; an absent
    /// `consensus` means single-writer and MUST NOT be advertised as scaling.
    pub fn check_scaling_claim(&self, advertises_horizontal_scaling: bool) -> Result<(), DepotError> {
        if advertises_horizontal_scaling && self.consensus.is_none() {
            return Err(DepotError::ScalingWithoutConsensus {
                kind: self.kind.clone(),
            });
        }
        Ok(())
    }

    /// Encode to deterministic CBOR (§18.1.2).
    pub fn det_cbor(&self) -> Vec<u8> {
        let parts: Vec<Cv> = self
            .parts
            .iter()
            .map(|p| {
                let mut pm: Vec<(u64, Cv)> = vec![
                    (1, Cv::Text(p.service.as_str().to_string())),
                    (2, Cv::Bytes(p.provider.clone())),
                ];
                if let Some(d) = &p.descriptor {
                    pm.push((3, Cv::Bytes(d.0.clone())));
                }
                Cv::Map(pm)
            })
            .collect();

        let mut m: Vec<(u64, Cv)> = vec![(1, Cv::Text(self.kind.clone())), (2, Cv::Array(parts))];
        if let Some(r) = &self.recipe {
            m.push((3, Cv::Bytes(r.clone())));
        }
        if let Some(c) = &self.consensus {
            m.push((4, Cv::Text(c.clone())));
        }
        cbor::encode(&Cv::Map(m))
    }

    /// Decode from deterministic CBOR, failing closed on an unknown part service.
    pub fn from_det_cbor(bytes: &[u8]) -> Result<Self, DepotError> {
        let mut f = Fields::from_cv(cbor::decode(bytes)?)?;
        let kind = as_text(f.req(1)?)?;

        let mut parts = Vec::new();
        for item in cbor::as_array(f.req(2)?)? {
            let mut pf = Fields::from_cv(item)?;
            let s = as_text(pf.req(1)?)?;
            let service = Service::from_str(&s).ok_or(DepotError::UnknownRegistryValue {
                registry: "service",
                value: s,
            })?;
            let provider = as_bytes(pf.req(2)?)?;
            let descriptor = pf.take(3).map(as_bytes).transpose()?.map(ContentId);
            pf.deny_unknown()?;
            parts.push(Part {
                service,
                provider,
                descriptor,
            });
        }
        if parts.is_empty() {
            return Err(DepotError::EmptyFormula { kind });
        }

        let recipe = f.take(3).map(as_bytes).transpose()?;
        let consensus = f.take(4).map(as_text).transpose()?;
        f.deny_unknown()?;
        Ok(DepotFormula {
            kind,
            parts,
            recipe,
            consensus,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::Visibility;

    fn part(service: Service, provider: &[u8]) -> Part {
        Part {
            service,
            provider: provider.to_vec(),
            descriptor: None,
        }
    }

    /// `database` = box + volume + bucket (§3.7).
    fn database(provider_box: &[u8]) -> DepotFormula {
        DepotFormula::new(
            "postgres",
            vec![
                part(Service::Box, provider_box),
                part(Service::Volume, b"op-a"),
                part(Service::Bucket, b"op-a"),
            ],
        )
    }

    #[test]
    fn database_inherits_terminating_from_its_box() {
        let f = database(b"op-a");
        // The bucket and volume are structurally blind; the box is not.
        let d = f
            .derive(&[
                Visibility::TERMINATING,
                Visibility::BLIND_STRUCTURAL,
                Visibility::BLIND_STRUCTURAL,
            ])
            .unwrap();
        assert_eq!(d.visibility, Visibility::TERMINATING);
        // And it must NOT be describable as private, however blind two of its three parts are.
        assert!(!d.visibility.may_be_called_private());
        assert_eq!(d.portability, Portability::ExportImport);
        assert!(d.single_operator);
    }

    #[test]
    fn queue_is_at_best_blind_and_inherits_conditionally() {
        // queue = bucket + a claim mechanism (§3.7). Client-encrypted payload.
        let f = DepotFormula::new("kafka-queue", vec![part(Service::Bucket, b"op-b")]);
        let d = f.derive(&[Visibility::BLIND_STRUCTURAL]).unwrap();
        assert_eq!(d.visibility, Visibility::BLIND_STRUCTURAL);
        assert_eq!(d.portability, Portability::ZeroMigration);

        // Same formula, plaintext handed to the bucket: not blind, and calling it a queue does not
        // make it so (§3.7).
        let d2 = f.derive(&[Visibility::BLIND_DECLARED]).unwrap();
        assert!(!d2.visibility.may_be_called_private());
    }

    #[test]
    fn least_blind_wins_regardless_of_part_order() {
        let f = database(b"op-a");
        let a = f
            .derive(&[
                Visibility::BLIND_STRUCTURAL,
                Visibility::BLIND_STRUCTURAL,
                Visibility::TERMINATING,
            ])
            .unwrap();
        let b = f
            .derive(&[
                Visibility::TERMINATING,
                Visibility::BLIND_STRUCTURAL,
                Visibility::BLIND_STRUCTURAL,
            ])
            .unwrap();
        assert_eq!(a.visibility, b.visibility);
        assert_eq!(a.visibility, Visibility::TERMINATING);
    }

    #[test]
    fn attested_is_less_blind_than_structural_but_more_than_declared() {
        let f = DepotFormula::new("inference", vec![part(Service::EdgeFn, b"op-c")]);
        let d = f.derive(&[Visibility::TERMINATING_ATTESTED]).unwrap();
        assert_eq!(d.visibility, Visibility::TERMINATING_ATTESTED);
        // A TEE narrows exposure; it never makes the formula sellable as blindness (§8).
        assert!(!d.visibility.may_be_called_private());
        // And a formula mixing an attested part with a plain terminating one inherits the worse.
        let f2 = DepotFormula::new(
            "inference-2",
            vec![part(Service::EdgeFn, b"op-c"), part(Service::Box, b"op-c")],
        );
        let d2 = f2
            .derive(&[Visibility::TERMINATING_ATTESTED, Visibility::TERMINATING])
            .unwrap();
        assert_eq!(d2.visibility, Visibility::TERMINATING);
    }

    #[test]
    fn multi_operator_formula_is_detectable() {
        let f = database(b"op-z"); // box at op-z, volume+bucket at op-a
        let d = f
            .derive(&[
                Visibility::TERMINATING,
                Visibility::BLIND_STRUCTURAL,
                Visibility::BLIND_STRUCTURAL,
            ])
            .unwrap();
        assert!(!d.single_operator);
    }

    #[test]
    fn stateless_only_formula_stays_zero_migration() {
        let f = DepotFormula::new(
            "static-site",
            vec![part(Service::Bucket, b"op-a"), part(Service::EdgeFn, b"op-a")],
        );
        let d = f
            .derive(&[Visibility::BLIND_ROUTING, Visibility::TERMINATING])
            .unwrap();
        assert_eq!(d.portability, Portability::ZeroMigration);
    }

    #[test]
    fn empty_formula_has_nothing_to_inherit() {
        let f = DepotFormula::new("nothing", vec![]);
        assert!(matches!(
            f.derive(&[]),
            Err(DepotError::EmptyFormula { .. })
        ));
    }

    #[test]
    fn scaling_claim_requires_consensus() {
        let f = database(b"op-a");
        assert!(matches!(
            f.check_scaling_claim(true),
            Err(DepotError::ScalingWithoutConsensus { .. })
        ));
        // Not advertising scaling is fine — it is a single-writer database, and may be described
        // as one.
        f.check_scaling_claim(false).unwrap();

        let mut scaled = database(b"op-a");
        scaled.consensus = Some("raft".into());
        scaled.check_scaling_claim(true).unwrap();
    }

    #[test]
    fn round_trips() {
        let mut f = database(b"op-a");
        f.recipe = Some(vec![0xa1, 0x01, 0x02]);
        f.consensus = Some("single-writer-lease".into());
        f.parts[0].descriptor = Some(ContentId::of(b"desc"));
        let back = DepotFormula::from_det_cbor(&f.det_cbor()).unwrap();
        assert_eq!(f, back);
        assert_eq!(back.det_cbor(), f.det_cbor());
    }

    #[test]
    fn formula_naming_an_unknown_service_fails_closed() {
        let bad = cbor::encode(&Cv::Map(vec![
            (1, Cv::Text("postgres".into())),
            (
                2,
                Cv::Array(vec![Cv::Map(vec![
                    (1, Cv::Text("database".into())), // a formula is never a part service
                    (2, Cv::Bytes(b"op".to_vec())),
                ])]),
            ),
        ]));
        assert!(matches!(
            DepotFormula::from_det_cbor(&bad),
            Err(DepotError::UnknownRegistryValue { registry: "service", .. })
        ));
    }

    #[test]
    fn decoding_a_partless_formula_fails_closed() {
        let bad = cbor::encode(&Cv::Map(vec![
            (1, Cv::Text("empty".into())),
            (2, Cv::Array(vec![])),
        ]));
        assert!(matches!(
            DepotFormula::from_det_cbor(&bad),
            Err(DepotError::EmptyFormula { .. })
        ));
    }
}
