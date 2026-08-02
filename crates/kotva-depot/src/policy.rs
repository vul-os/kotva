//! [`DepotServicePolicy`] — the `policy` blob of an `infra-service` `CoordinatorDescriptor` (§3.3).
//!
//! §18.8a.1 defines `policy` as an **opaque, kind-specific det_cbor blob**. DEPOT fixes its shape for
//! this kind, which is what makes "a new limit is a DEPOT registry change, never a §18 wire change"
//! true rather than aspirational.

use kotva_core::cbor::{self, as_bool, as_text, as_u64, Cv, Fields};

use crate::control::{check_operator_offering, Ability};
use crate::service::{Backing, Service};
use crate::{canonical_key_cmp, DepotError};

/// Declared ceilings (§3.3). Every value is a `uint` — no floats anywhere (§18.1).
///
/// **Absent means undeclared, never unlimited.** A client MUST NOT infer a ceiling from an omitted
/// field, and an operator MUST NOT read an omission as permission to refuse arbitrarily.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capacity {
    /// Storage ceiling in bytes.
    pub total_bytes: Option<u64>,
    /// Largest single object or volume, in bytes.
    pub max_object_bytes: Option<u64>,
    /// Sustained throughput ceiling, bits per second.
    pub egress_bps: Option<u64>,
    /// Concurrent streams or instances.
    pub max_concurrent: Option<u64>,
    /// Latency tier — `"cold"` / `"warm"` / `"commit-path"`.
    pub class: Option<String>,
    /// Per-mille availability **intent** (0…1000).
    ///
    /// An aim, **not** an SLA and not evidence. What was *achieved* is the §7 `uptime` measurement,
    /// published by observers rather than by the operator.
    pub uptime_target: Option<u64>,
    /// The open `resources` quantity vocabulary (§3.1) — `cpu-millicores`, `mem-bytes`,
    /// `gpu-count`, `gpu-mem-bytes`, `ipv4-count`, `accel-<class>`, …
    ///
    /// Open by construction: a new accelerator class is a registry name, never a spec change.
    pub resources: Vec<(String, u64)>,
}

impl Capacity {
    /// Look up a resource quantity by key.
    pub fn resource(&self, key: &str) -> Option<u64> {
        self.resources
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| *v)
    }

    /// Whether `uptime_target` is a well-formed per-mille value. A value above 1000 is not a
    /// stronger promise, it is a malformed one.
    pub fn uptime_target_is_wellformed(&self) -> bool {
        self.uptime_target.map_or(true, |t| t <= 1000)
    }

    fn to_cv(&self) -> Cv {
        let mut m: Vec<(u64, Cv)> = Vec::new();
        if let Some(v) = self.total_bytes {
            m.push((1, Cv::U64(v)));
        }
        if let Some(v) = self.max_object_bytes {
            m.push((2, Cv::U64(v)));
        }
        if let Some(v) = self.egress_bps {
            m.push((3, Cv::U64(v)));
        }
        if let Some(v) = self.max_concurrent {
            m.push((4, Cv::U64(v)));
        }
        if let Some(v) = &self.class {
            m.push((5, Cv::Text(v.clone())));
        }
        if let Some(v) = self.uptime_target {
            m.push((6, Cv::U64(v)));
        }
        if !self.resources.is_empty() {
            let mut r: Vec<(String, Cv)> = self
                .resources
                .iter()
                .map(|(k, v)| (k.clone(), Cv::U64(*v)))
                .collect();
            r.sort_by(|a, b| canonical_key_cmp(&a.0, &b.0));
            m.push((7, Cv::TextMap(r)));
        }
        Cv::Map(m)
    }

    fn from_cv(cv: Cv) -> Result<Self, DepotError> {
        let mut f = Fields::from_cv(cv)?;
        let mut c = Capacity {
            total_bytes: f.take(1).map(as_u64).transpose()?,
            max_object_bytes: f.take(2).map(as_u64).transpose()?,
            egress_bps: f.take(3).map(as_u64).transpose()?,
            max_concurrent: f.take(4).map(as_u64).transpose()?,
            class: f.take(5).map(as_text).transpose()?,
            uptime_target: f.take(6).map(as_u64).transpose()?,
            resources: Vec::new(),
        };
        if let Some(Cv::TextMap(pairs)) = f.take(7) {
            for (k, v) in pairs {
                c.resources.push((k, as_u64(v)?));
            }
        }
        f.deny_unknown()?;
        Ok(c)
    }
}

/// The `infra-service` descriptor policy (§3.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepotServicePolicy {
    /// Which elemental this coordinator provides.
    pub service: Service,
    /// Who owns the implementation behind it (§1.2) — a declared, first-class fact.
    pub backing: Backing,
    /// Declared ceilings, if any.
    pub capacity: Option<Capacity>,
    /// §3.2 registry attribute keys — compatibility **predicates** a client filters on
    /// (`arch`, `virt`, `region`, `jurisdiction`, `ipv6`, `persistence`, `attachment`,
    /// `artifact-source`).
    pub attributes: Vec<(String, String)>,
    /// The §5.2 verbs this coordinator accepts. Empty means the common set (§5.2).
    pub abilities: Vec<Ability>,
}

impl DepotServicePolicy {
    /// A minimal policy: one elemental, one backing mode, nothing declared.
    pub fn new(service: Service, backing: Backing) -> Self {
        DepotServicePolicy {
            service,
            backing,
            capacity: None,
            attributes: Vec::new(),
            abilities: Vec::new(),
        }
    }

    /// Sort both text maps into canonical CBOR key order (§18.1.1).
    ///
    /// [`Self::det_cbor`] always emits canonical order regardless of in-memory order, so a decoded
    /// policy is canonically ordered. Call this on a hand-built policy before comparing it to a
    /// decoded one.
    pub fn normalized(mut self) -> Self {
        self.attributes.sort_by(|a, b| canonical_key_cmp(&a.0, &b.0));
        if let Some(c) = &mut self.capacity {
            c.resources.sort_by(|a, b| canonical_key_cmp(&a.0, &b.0));
        }
        self
    }

    /// Run every conformance check this policy can be checked against, in one call (§3.3, §5.2).
    ///
    /// **This exists because the individual `check_*` functions were previously reachable only from
    /// tests.** Decoding and validating are properly separate — [`Self::from_det_cbor`] must be able
    /// to parse a *non-conformant* policy so a client can report on it rather than silently drop it
    /// — but that separation left no obvious entry point, so an integrator got no protection unless
    /// they already knew to call several free functions by name. A gate nothing calls is not a gate.
    ///
    /// Checks applied:
    ///
    /// - every declared ability is meaningful for this elemental (§5.2);
    /// - the offering does not expose `destroy` while withholding `export` (DEPOT-4);
    /// - `uptime_target`, if declared, is a well-formed per-mille value (§3.3).
    ///
    /// Visibility honesty ([`crate::check_visibility`]) is deliberately **not** here: it needs facts
    /// this blob does not carry — whether the client encrypted its bytes, and whether a bucket is
    /// public-serving — so it is the caller's to run at the point those are known.
    pub fn validate(&self) -> Result<(), DepotError> {
        let offered: Vec<Ability> = if self.abilities.is_empty() {
            Ability::COMMON.to_vec()
        } else {
            self.abilities.clone()
        };
        for a in &offered {
            if !a.is_valid_for(self.service) {
                return Err(DepotError::UnknownRegistryValue {
                    registry: "ability-for-service",
                    value: format!("{} on {}", a.as_str(), self.service.as_str()),
                });
            }
        }
        check_operator_offering(&offered)?;
        if let Some(c) = &self.capacity {
            if !c.uptime_target_is_wellformed() {
                return Err(DepotError::UnknownRegistryValue {
                    registry: "uptime-target-per-mille",
                    value: c.uptime_target.unwrap_or_default().to_string(),
                });
            }
        }
        Ok(())
    }

    /// Look up an attribute predicate.
    pub fn attribute(&self, key: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Whether this coordinator accepts `ability`.
    ///
    /// An empty `abilities` list means the common seven (§5.2); anything else is an explicit
    /// allow-list. An ability meaningless for the elemental is never accepted regardless.
    pub fn accepts(&self, ability: Ability) -> bool {
        if !ability.is_valid_for(self.service) {
            return false;
        }
        if self.abilities.is_empty() {
            return Ability::COMMON.contains(&ability);
        }
        self.abilities.contains(&ability)
    }

    /// Encode to deterministic CBOR (§18.1.2).
    pub fn det_cbor(&self) -> Vec<u8> {
        let mut m: Vec<(u64, Cv)> = vec![
            (1, Cv::Text(self.service.as_str().to_string())),
            (2, Cv::Text(self.backing.as_str().to_string())),
        ];
        if let Some(c) = &self.capacity {
            m.push((3, c.to_cv()));
        }
        if !self.attributes.is_empty() {
            let mut a: Vec<(String, Cv)> = self
                .attributes
                .iter()
                .map(|(k, v)| (k.clone(), Cv::Text(v.clone())))
                .collect();
            a.sort_by(|x, y| canonical_key_cmp(&x.0, &y.0));
            m.push((4, Cv::TextMap(a)));
        }
        if !self.abilities.is_empty() {
            let mut ab: Vec<Ability> = self.abilities.clone();
            ab.sort();
            ab.dedup();
            m.push((
                5,
                Cv::Array(ab.into_iter().map(|a| Cv::Text(a.as_str().to_string())).collect()),
            ));
        }
        cbor::encode(&Cv::Map(m))
    }

    /// Decode from deterministic CBOR, failing closed on any unknown closed-registry value.
    pub fn from_det_cbor(bytes: &[u8]) -> Result<Self, DepotError> {
        let mut f = Fields::from_cv(cbor::decode(bytes)?)?;

        let svc_s = as_text(f.req(1)?)?;
        let service = Service::from_str(&svc_s).ok_or(DepotError::UnknownRegistryValue {
            registry: "service",
            value: svc_s,
        })?;

        let bk_s = as_text(f.req(2)?)?;
        let backing = Backing::from_str(&bk_s).ok_or(DepotError::UnknownRegistryValue {
            registry: "backing",
            value: bk_s,
        })?;

        let capacity = f.take(3).map(Capacity::from_cv).transpose()?;

        let mut attributes = Vec::new();
        if let Some(Cv::TextMap(pairs)) = f.take(4) {
            for (k, v) in pairs {
                attributes.push((k, as_text(v)?));
            }
        }

        let mut abilities = Vec::new();
        if let Some(cv) = f.take(5) {
            for item in cbor::as_array(cv)? {
                let s = as_text(item)?;
                let a = Ability::from_str(&s).ok_or(DepotError::UnknownRegistryValue {
                    registry: "ability",
                    value: s,
                })?;
                abilities.push(a);
            }
        }

        f.deny_unknown()?;
        Ok(DepotServicePolicy {
            service,
            backing,
            capacity,
            attributes,
            abilities,
        })
    }
}

// Keeps `as_bool` imported for symmetry with the other schema modules without warning; the policy
// blob itself carries no booleans (every Capacity value is a uint, §18.1).
const _: fn(Cv) -> Result<bool, cbor::CborError> = as_bool;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> DepotServicePolicy {
        DepotServicePolicy {
            service: Service::Box,
            backing: Backing::Customer,
            capacity: Some(Capacity {
                total_bytes: Some(2_000_000_000_000),
                max_object_bytes: None,
                egress_bps: Some(50_000_000),
                max_concurrent: Some(4),
                class: Some("commit-path".into()),
                uptime_target: Some(995),
                resources: vec![
                    ("cpu-millicores".into(), 4000),
                    ("gpu-count".into(), 1),
                    ("ipv4-count".into(), 1),
                    ("mem-bytes".into(), 17_179_869_184),
                ],
            }),
            attributes: vec![
                ("arch".into(), "aarch64".into()),
                ("ipv6".into(), "/56".into()),
                ("jurisdiction".into(), "ZA".into()),
                ("region".into(), "za-jhb".into()),
                ("virt".into(), "vm".into()),
            ],
            abilities: vec![
                Ability::Provision,
                Ability::Inspect,
                Ability::Export,
                Ability::Destroy,
                Ability::Console,
            ],
        }
    }

    #[test]
    fn round_trips_byte_identically() {
        let p = sample().normalized();
        let b = p.det_cbor();
        let back = DepotServicePolicy::from_det_cbor(&b).unwrap();
        assert_eq!(p, back);
        // Deterministic: re-encoding the decoded form reproduces the same bytes (§18.1.1).
        assert_eq!(back.det_cbor(), b);
    }

    #[test]
    fn minimal_policy_round_trips() {
        let p = DepotServicePolicy::new(Service::Bucket, Backing::Operator);
        let back = DepotServicePolicy::from_det_cbor(&p.det_cbor()).unwrap();
        assert_eq!(p, back);
        assert!(back.capacity.is_none());
    }

    #[test]
    fn unknown_service_and_backing_fail_closed() {
        // A hand-built map naming a service outside the registry.
        let bad = cbor::encode(&Cv::Map(vec![
            (1, Cv::Text("vm".into())),
            (2, Cv::Text("operator".into())),
        ]));
        assert!(matches!(
            DepotServicePolicy::from_det_cbor(&bad),
            Err(DepotError::UnknownRegistryValue { registry: "service", .. })
        ));

        let bad2 = cbor::encode(&Cv::Map(vec![
            (1, Cv::Text("box".into())),
            (2, Cv::Text("byo".into())),
        ]));
        assert!(matches!(
            DepotServicePolicy::from_det_cbor(&bad2),
            Err(DepotError::UnknownRegistryValue { registry: "backing", .. })
        ));
    }

    #[test]
    fn unknown_ability_in_policy_fails_closed() {
        let bad = cbor::encode(&Cv::Map(vec![
            (1, Cv::Text("box".into())),
            (2, Cv::Text("operator".into())),
            (5, Cv::Array(vec![Cv::Text("terminate".into())])),
        ]));
        assert!(matches!(
            DepotServicePolicy::from_det_cbor(&bad),
            Err(DepotError::UnknownRegistryValue { registry: "ability", .. })
        ));
    }

    #[test]
    fn absent_capacity_is_undeclared_not_unlimited() {
        let p = DepotServicePolicy::new(Service::Bucket, Backing::Operator);
        // There is deliberately no `total_bytes()` accessor defaulting to u64::MAX — the absence
        // must stay visible to the caller (§3.3).
        assert!(p.capacity.is_none());
        let c = Capacity::default();
        assert_eq!(c.total_bytes, None);
        assert_eq!(c.resource("gpu-count"), None);
    }

    #[test]
    fn uptime_target_wellformedness() {
        let mut c = Capacity { uptime_target: Some(1000), ..Default::default() };
        assert!(c.uptime_target_is_wellformed());
        c.uptime_target = Some(1001);
        assert!(!c.uptime_target_is_wellformed());
        c.uptime_target = None;
        assert!(c.uptime_target_is_wellformed());
    }

    #[test]
    fn accepts_respects_service_scoping_and_allowlist() {
        let p = sample();
        assert!(p.accepts(Ability::Console)); // explicitly listed, valid for box
        assert!(!p.accepts(Ability::Observe)); // valid for box but not in the allow-list
        assert!(!p.accepts(Ability::Attach)); // meaningless for box

        // Empty list = the common seven.
        let q = DepotServicePolicy::new(Service::Bucket, Backing::Operator);
        assert!(q.accepts(Ability::Provision));
        assert!(q.accepts(Ability::Export));
        assert!(!q.accepts(Ability::Read)); // bucket-specific, not in the common set
        assert!(!q.accepts(Ability::Console)); // meaningless for bucket
    }

    #[test]
    fn validate_catches_what_decode_deliberately_lets_through() {
        // A policy offering destroy while withholding export decodes fine (decode != validate) and
        // must be caught by the reachable entry point (DEPOT-4).
        let bad = DepotServicePolicy {
            abilities: vec![Ability::Provision, Ability::Destroy],
            ..DepotServicePolicy::new(Service::Box, Backing::Operator)
        };
        let decoded = DepotServicePolicy::from_det_cbor(&bad.det_cbor()).unwrap();
        assert_eq!(decoded, bad, "decode must not reject a non-conformant policy");
        assert_eq!(decoded.validate(), Err(DepotError::DestroyWithoutExport));

        // An ability meaningless for the elemental.
        let wrong = DepotServicePolicy {
            abilities: vec![Ability::Attach],
            ..DepotServicePolicy::new(Service::Bucket, Backing::Operator)
        };
        assert!(wrong.validate().is_err());

        // A malformed per-mille target.
        let over = DepotServicePolicy {
            capacity: Some(Capacity { uptime_target: Some(1001), ..Default::default() }),
            ..DepotServicePolicy::new(Service::Bucket, Backing::Operator)
        };
        assert!(over.validate().is_err());
    }

    #[test]
    fn validate_accepts_conformant_policies_including_the_default_set() {
        // Empty abilities = the common seven, which contain both destroy and export, so the
        // default offering is conformant.
        DepotServicePolicy::new(Service::Box, Backing::Operator).validate().unwrap();
        DepotServicePolicy::new(Service::Bucket, Backing::Customer).validate().unwrap();
        sample().validate().unwrap();
    }

    #[test]
    fn resources_is_an_open_vocabulary() {
        // A brand-new accelerator class must survive a round-trip with no code change (§3.1).
        let p = DepotServicePolicy {
            capacity: Some(Capacity {
                resources: vec![("accel-photonic-v3".into(), 2)],
                ..Default::default()
            }),
            ..DepotServicePolicy::new(Service::Box, Backing::Operator)
        };
        let back = DepotServicePolicy::from_det_cbor(&p.det_cbor()).unwrap();
        assert_eq!(
            back.capacity.as_ref().unwrap().resource("accel-photonic-v3"),
            Some(2)
        );
    }
}
