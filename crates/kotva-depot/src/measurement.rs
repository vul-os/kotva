//! [`DepotMeasurement`] — the ATTEST claim body for a service measurement (§7).
//!
//! DEPOT mints **no new wire object, DS-tag, or signature** for reputation. A measurement is an
//! ATTEST public `Attestation` whose `schema` is [`crate::MEASUREMENT_SCHEMA`]; this module defines
//! only the claim body carried inside it. The carrier's own signature authenticates it, `issuer` is
//! the rater's `IK`, `subject` is the rated coordinator's, and a **self-measurement** is exactly
//! `issuer == subject`.
//!
//! Three value sets here are **closed** — `metric`, `method`, and evidence `kind`. A rater MUST NOT
//! coin a value and an aggregator MUST ignore an unknown one rather than guess: silently treating an
//! unrecognised metric as meaningful is how a reputation market gets poisoned by vocabulary alone.

use kotva_core::cbor::{self, as_bool, as_text, as_u64, Cv, Fields};
use kotva_core::TimestampMs;

use crate::service::Service;
use crate::DepotError;

/// What was measured (§7, CLOSED).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Metric {
    /// Availability, per-mille (`0…1000`).
    Uptime,
    /// A conformance-vector pass.
    Conformance,
    /// An honest-visibility audit (DEPOT-2, DEPOT-12).
    VisibilityAudit,
    /// Round-trip latency in milliseconds.
    LatencyMs,
    /// Whether the operator honoured the ceilings its own signed policy declared (§3.3).
    CapacityConformance,
    /// Whether a DEPOT-4 export actually round-tripped into a **different** operator.
    ExportConformance,
    /// Whether the coordinator accepts the §5.2 verb set without coinage or aliasing.
    ///
    /// The cheapest of the three conformance metrics to test, and the one that makes the control
    /// plane's interoperability falsifiable rather than assumed.
    AbilityConformance,
}

impl Metric {
    /// The wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            Metric::Uptime => "uptime",
            Metric::Conformance => "conformance",
            Metric::VisibilityAudit => "visibility-audit",
            Metric::LatencyMs => "latency-ms",
            Metric::CapacityConformance => "capacity-conformance",
            Metric::ExportConformance => "export-conformance",
            Metric::AbilityConformance => "ability-conformance",
        }
    }

    /// Parse; CLOSED, fails closed.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "uptime" => Metric::Uptime,
            "conformance" => Metric::Conformance,
            "visibility-audit" => Metric::VisibilityAudit,
            "latency-ms" => Metric::LatencyMs,
            "capacity-conformance" => Metric::CapacityConformance,
            "export-conformance" => Metric::ExportConformance,
            "ability-conformance" => Metric::AbilityConformance,
            _ => return None,
        })
    }

    /// Whether this metric's `value` is a `uint` (vs a `bool`) — the typing is **by metric** (§7).
    pub fn is_numeric(self) -> bool {
        matches!(self, Metric::Uptime | Metric::LatencyMs)
    }
}

/// How the observation was produced (§7, CLOSED).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Method {
    /// An active probe.
    Probe,
    /// A conformance vector run.
    ConformanceVector,
    /// A human or tooling audit.
    Audit,
    /// The operator's own report.
    SelfReport,
}

impl Method {
    /// The wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Probe => "probe",
            Method::ConformanceVector => "conformance-vector",
            Method::Audit => "audit",
            Method::SelfReport => "self-report",
        }
    }

    /// Parse; CLOSED, fails closed.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "probe" => Method::Probe,
            "conformance-vector" => Method::ConformanceVector,
            "audit" => Method::Audit,
            "self-report" => Method::SelfReport,
            _ => return None,
        })
    }

    /// Whether an independent party can **re-run** this and get the same answer (§7, §8).
    ///
    /// This is the profile's central reputation bound — *reproducibility over reputation* — and it
    /// has a hole the spec names as the cheapest attack: `self-report` is not reproducible by
    /// construction, and nothing stops an operator minting fresh pseudonymous rater keys and
    /// publishing praise from each. A consumer MUST NOT treat a corpus of `self-report` claims as
    /// evidence.
    pub fn is_reproducible(self) -> bool {
        matches!(self, Method::Probe | Method::ConformanceVector)
    }
}

/// The kind of evidence reference attached to a measurement (§7, CLOSED).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EvidenceKind {
    /// A reproducible recipe an aggregator can re-run.
    Recipe,
    /// A conformance-vector identifier.
    VectorId,
    /// A transcript — **not** reproducible by construction (§8).
    Transcript,
}

impl EvidenceKind {
    /// The wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            EvidenceKind::Recipe => "recipe",
            EvidenceKind::VectorId => "vector-id",
            EvidenceKind::Transcript => "transcript",
        }
    }

    /// Parse; CLOSED, fails closed.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "recipe" => EvidenceKind::Recipe,
            "vector-id" => EvidenceKind::VectorId,
            "transcript" => EvidenceKind::Transcript,
            _ => return None,
        })
    }

    /// Whether this evidence supports independent re-running (§7).
    pub fn is_reproducible(self) -> bool {
        matches!(self, EvidenceKind::Recipe | EvidenceKind::VectorId)
    }
}

/// A measurement's value, typed **by its metric** — never a float (§18.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementValue {
    /// `uptime` (per-mille, `0…1000`) or `latency-ms`.
    Uint(u64),
    /// Every conformance/audit metric.
    Bool(bool),
}

/// The claim body for schema [`crate::MEASUREMENT_SCHEMA`] (§7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepotMeasurement {
    /// Which elemental was measured.
    pub service: Service,
    /// What was measured.
    pub metric: Metric,
    /// The observation, typed by `metric`.
    pub value: MeasurementValue,
    /// How it was produced.
    pub method: Method,
    /// Milliseconds since the Unix epoch (§18.1).
    pub observed_at: TimestampMs,
    /// Optional evidence reference.
    pub evidence: Option<(EvidenceKind, String)>,
}

impl DepotMeasurement {
    /// Whether `value`'s type matches what `metric` requires (§7).
    ///
    /// A `uptime` carrying a bool, or a `conformance` carrying an integer, is malformed — and a
    /// malformed measurement is **ignored** by aggregators, never a fail-closed event and never a
    /// new error code (§7).
    pub fn is_wellformed(&self) -> bool {
        let typed = matches!(
            (self.metric.is_numeric(), &self.value),
            (true, MeasurementValue::Uint(_)) | (false, MeasurementValue::Bool(_))
        );
        // `uptime` is per-mille: a value above 1000 is not a stronger claim, it is a broken one.
        let ranged = !matches!(
            (self.metric, &self.value),
            (Metric::Uptime, MeasurementValue::Uint(v)) if *v > 1000
        );
        typed && ranged
    }

    /// Whether a consumer can independently re-run this rather than trusting the reported value.
    ///
    /// True only when **both** the method and the supplied evidence support it. A consumer SHOULD
    /// weight a measurement by this, and MUST NOT treat a corpus of non-reproducible claims as
    /// evidence (§8).
    pub fn is_independently_checkable(&self) -> bool {
        self.method.is_reproducible()
            && self
                .evidence
                .as_ref()
                .is_some_and(|(k, _)| k.is_reproducible())
    }

    /// Encode to deterministic CBOR (§18.1.2).
    pub fn det_cbor(&self) -> Vec<u8> {
        let value = match self.value {
            MeasurementValue::Uint(v) => Cv::U64(v),
            MeasurementValue::Bool(b) => Cv::Bool(b),
        };
        let mut m: Vec<(u64, Cv)> = vec![
            (1, Cv::Text(self.service.as_str().to_string())),
            (2, Cv::Text(self.metric.as_str().to_string())),
            (3, value),
            (4, Cv::Text(self.method.as_str().to_string())),
            (5, Cv::U64(self.observed_at)),
        ];
        if let Some((k, r)) = &self.evidence {
            m.push((
                6,
                Cv::Map(vec![
                    (1, Cv::Text(k.as_str().to_string())),
                    (2, Cv::Text(r.clone())),
                ]),
            ));
        }
        cbor::encode(&Cv::Map(m))
    }

    /// Decode from deterministic CBOR, failing closed on any unknown closed-registry value.
    pub fn from_det_cbor(bytes: &[u8]) -> Result<Self, DepotError> {
        let mut f = Fields::from_cv(cbor::decode(bytes)?)?;

        let s = as_text(f.req(1)?)?;
        let service = Service::from_str(&s).ok_or(DepotError::UnknownRegistryValue {
            registry: "service",
            value: s,
        })?;

        let m_s = as_text(f.req(2)?)?;
        let metric = Metric::from_str(&m_s).ok_or(DepotError::UnknownRegistryValue {
            registry: "metric",
            value: m_s,
        })?;

        let raw = f.req(3)?;
        let value = match raw {
            Cv::Bool(b) => MeasurementValue::Bool(b),
            other => MeasurementValue::Uint(as_u64(other)?),
        };

        let mt_s = as_text(f.req(4)?)?;
        let method = Method::from_str(&mt_s).ok_or(DepotError::UnknownRegistryValue {
            registry: "method",
            value: mt_s,
        })?;

        let observed_at = as_u64(f.req(5)?)?;

        let mut evidence = None;
        if let Some(cv) = f.take(6) {
            let mut ef = Fields::from_cv(cv)?;
            let k_s = as_text(ef.req(1)?)?;
            let kind = EvidenceKind::from_str(&k_s).ok_or(DepotError::UnknownRegistryValue {
                registry: "evidence-kind",
                value: k_s,
            })?;
            let r = as_text(ef.req(2)?)?;
            ef.deny_unknown()?;
            evidence = Some((kind, r));
        }

        f.deny_unknown()?;
        Ok(DepotMeasurement {
            service,
            metric,
            value,
            method,
            observed_at,
            evidence,
        })
    }
}

// Symmetry with the other schema modules; `as_bool` is reached via the Cv::Bool arm above.
const _: fn(Cv) -> Result<bool, cbor::CborError> = as_bool;

#[cfg(test)]
mod tests {
    use super::*;

    fn uptime(v: u64) -> DepotMeasurement {
        DepotMeasurement {
            service: Service::Box,
            metric: Metric::Uptime,
            value: MeasurementValue::Uint(v),
            method: Method::Probe,
            observed_at: 1_775_000_000_000,
            evidence: Some((EvidenceKind::Recipe, "ipfs://recipe".into())),
        }
    }

    #[test]
    fn round_trips_numeric_and_bool_metrics() {
        let a = uptime(995);
        assert_eq!(DepotMeasurement::from_det_cbor(&a.det_cbor()).unwrap(), a);

        let b = DepotMeasurement {
            service: Service::Bucket,
            metric: Metric::ExportConformance,
            value: MeasurementValue::Bool(true),
            method: Method::Audit,
            observed_at: 1,
            evidence: None,
        };
        assert_eq!(DepotMeasurement::from_det_cbor(&b.det_cbor()).unwrap(), b);
    }

    #[test]
    fn every_closed_registry_rejects_coinage() {
        for (idx, bad, reg) in [(2u64, "availability", "metric"), (4, "vibes", "method")] {
            let mut m: Vec<(u64, Cv)> = vec![
                (1, Cv::Text("box".into())),
                (2, Cv::Text("uptime".into())),
                (3, Cv::U64(1000)),
                (4, Cv::Text("probe".into())),
                (5, Cv::U64(1)),
            ];
            m[(idx - 1) as usize] = (idx, Cv::Text(bad.into()));
            let enc = cbor::encode(&Cv::Map(m));
            match DepotMeasurement::from_det_cbor(&enc) {
                Err(DepotError::UnknownRegistryValue { registry, .. }) => {
                    assert_eq!(registry, reg)
                }
                other => panic!("{bad:?} should fail closed on {reg}, got {other:?}"),
            }
        }
    }

    #[test]
    fn unknown_evidence_kind_fails_closed() {
        let enc = cbor::encode(&Cv::Map(vec![
            (1, Cv::Text("box".into())),
            (2, Cv::Text("uptime".into())),
            (3, Cv::U64(999)),
            (4, Cv::Text("probe".into())),
            (5, Cv::U64(1)),
            (
                6,
                Cv::Map(vec![
                    (1, Cv::Text("screenshot".into())),
                    (2, Cv::Text("x".into())),
                ]),
            ),
        ]));
        assert!(matches!(
            DepotMeasurement::from_det_cbor(&enc),
            Err(DepotError::UnknownRegistryValue {
                registry: "evidence-kind",
                ..
            })
        ));
    }

    #[test]
    fn value_typing_is_by_metric() {
        assert!(uptime(995).is_wellformed());
        // per-mille, so >1000 is broken not better.
        assert!(!uptime(1001).is_wellformed());

        let mistyped = DepotMeasurement {
            metric: Metric::Conformance,
            value: MeasurementValue::Uint(1),
            ..uptime(1)
        };
        assert!(!mistyped.is_wellformed());

        let mistyped2 = DepotMeasurement {
            metric: Metric::LatencyMs,
            value: MeasurementValue::Bool(true),
            ..uptime(1)
        };
        assert!(!mistyped2.is_wellformed());
    }

    #[test]
    fn self_report_is_never_independently_checkable() {
        // The cheapest attack the profile names (§8): self-report sits entirely outside the
        // re-run-the-probe mitigation.
        let m = DepotMeasurement {
            method: Method::SelfReport,
            evidence: Some((EvidenceKind::Recipe, "r".into())),
            ..uptime(1000)
        };
        assert!(!m.is_independently_checkable());

        // A transcript is not reproducible by construction either, even from a probe.
        let t = DepotMeasurement {
            method: Method::Probe,
            evidence: Some((EvidenceKind::Transcript, "t".into())),
            ..uptime(1000)
        };
        assert!(!t.is_independently_checkable());

        // No evidence at all is not checkable, however good the method.
        let n = DepotMeasurement {
            evidence: None,
            ..uptime(1000)
        };
        assert!(!n.is_independently_checkable());

        // The one combination that is.
        assert!(uptime(1000).is_independently_checkable());
    }

    #[test]
    fn ability_conformance_is_a_bool_metric() {
        assert!(!Metric::AbilityConformance.is_numeric());
        let m = DepotMeasurement {
            metric: Metric::AbilityConformance,
            value: MeasurementValue::Bool(false),
            method: Method::ConformanceVector,
            evidence: Some((EvidenceKind::VectorId, "depot-abilities-v0".into())),
            ..uptime(0)
        };
        assert!(m.is_wellformed());
        assert!(m.is_independently_checkable());
        assert_eq!(DepotMeasurement::from_det_cbor(&m.det_cbor()).unwrap(), m);
    }
}
