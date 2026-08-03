//! The `ability-conformance` probe (§7) — a reusable check that a **live coordinator** speaks the
//! §5.2 verb set exactly, without coinage and without aliasing.
//!
//! # What this is for
//!
//! §5.2 is a **closed** registry of exact strings, and the property it buys is stated plainly in the
//! profile: *"this is what makes one open-source client able to drive any conformant gateway."* That
//! property is only real if the refusal half is real. A gateway that quietly accepts `terminate` as a
//! synonym for `destroy`, or maps an unrecognised verb onto its nearest match, has diverged from the
//! vocabulary while every one of its own tests stays green — and the client that meets it later
//! fails for reasons neither side can see. §7 defines the `ability-conformance` metric for exactly
//! this, and this module is its probe.
//!
//! It is **not** a test. It is a library function: give it a callable that answers *"do you accept
//! this ability string for this elemental?"* — a real request over the coordinator's Noise channel, a
//! dry-run authorisation call, an in-process handler — and it returns a structured
//! [`AbilityProbeReport`] that maps onto a [`DepotMeasurement`] with
//! [`Metric::AbilityConformance`].
//!
//! # It probes both directions, because half a probe is worse than none
//!
//! Two distinct defects have to be caught, and a probe that catches only one certifies the other:
//!
//! - **Coinage / aliasing (over-acceptance).** A near-miss string is accepted. §5.2: a coordinator
//!   receiving an ability outside the registry MUST refuse and MUST NOT map it onto a
//!   similar-sounding one.
//! - **Under-implementation (under-acceptance).** A registry verb is refused. A gateway that does
//!   not answer to `export` is not driveable by the common client either, and it also breaks
//!   DEPOT-4's exit obligation the moment it offers `destroy` (§5.2, [`crate::check_operator_offering`]).
//!
//! # Coverage is reported, and a probe that examined nothing FAILS
//!
//! [`AbilityProbeReport::passed`] is false unless the probed counts equal the **hard-coded**
//! expectations in [`EXPECTED_VERBS`] — not counts derived from whatever the loop happened to visit.
//! An oracle that was never called, a registry that silently shrank, or an early `return` inside the
//! loop all read as a failure rather than as a clean pass.
//!
//! # The vacuity caveat survives (§7, normative for aggregators)
//!
//! §7 is explicit that with **one** implementation in existence this metric is vacuous, and that an
//! aggregator MUST NOT treat it as evidence of interoperability in that state: where every gateway
//! runs the same code there is nothing to diverge from, so it passes for reasons unrelated to
//! conformance. This module does not change that and does not claim to. It gives the metric teeth
//! where teeth are possible — against an *independently written* verb table, it is a real check;
//! against a re-export of [`crate::control::Ability`] it is a tautology. [`VACUITY_CAVEAT`] carries
//! the sentence, and every [`core::fmt::Display`] rendering here prints it, so a passing result
//! cannot be quoted without it.

use core::fmt;

use kotva_core::TimestampMs;

use crate::control::Ability;
use crate::measurement::{DepotMeasurement, EvidenceKind, MeasurementValue, Method, Metric};
use crate::service::Service;

/// The §7 caveat that must travel with every passing `ability-conformance` result.
pub const VACUITY_CAVEAT: &str = "CAVEAT (cloud.md §7): with one implementation in existence \
     `ability-conformance` is VACUOUS — an aggregator MUST NOT treat a pass as evidence of \
     interoperability. Below two independent implementations the binding check is the schema \
     vector corpus (conformance/SUITE.md), not this probe.";

/// The number of verbs in the §5.2 registry.
///
/// Hard-coded on purpose. A registry addition is meant to be a deliberate act (§5.2: "extended by
/// registry addition, never by coinage"), so a new verb must break this constant and be added to the
/// probe's expectations by hand — rather than silently widening what a gateway is allowed to accept
/// while the probe reports the same green.
pub const ABILITY_REGISTRY_SIZE: usize = 22;

/// How many §5.2 verbs each elemental MUST accept: the seven common lifecycle verbs plus its own.
///
/// Hard-coded rather than derived from [`Ability::is_valid_for`], so that a bug which shrinks the
/// registry cannot also shrink the expectation and leave the probe passing on less than it should.
/// [`expected_verbs`] reads it; a unit test cross-checks it against the enum.
pub const EXPECTED_VERBS: [(Service, usize); 4] = [
    (Service::Bucket, 11), // 7 common + read, write, delete, serve
    (Service::Volume, 11), // 7 common + attach, detach, resize, snapshot
    (Service::EdgeFn, 10), // 7 common + deploy, invoke, rollback
    (Service::Box, 12),    // 7 common + start, stop, restart, snapshot, console
];

/// Near-miss strings a conformant coordinator MUST refuse for **every** elemental (§5.2).
///
/// Curated rather than fuzzed: random noise is refused by any implementation, including a badly
/// broken one, so it certifies nothing. Each entry below is a string a plausible implementation
/// might accept *by accident*, grouped by the mistake that produces it.
///
/// 1. **Provider-vocabulary synonyms** — the coinages an implementer carries over from a commodity
///    cloud's API. `terminate` is the one §5.2 names by hand; the rest are its siblings. This is the
///    failure the metric exists for.
/// 2. **The privilege cliff by another name** — `exec`, `ssh`, `shell`. §5.2 makes `console`
///    separately delegable *because* interactive access subsumes everything else; a gateway that
///    also answers to `exec` has re-opened the cliff under a name no capability was ever scoped to.
/// 3. **Case variants** — §5.2 is a set of exact strings. A case-insensitive comparison is the single
///    most likely way an implementation accidentally leaves the registry, and it is invisible to a
///    test that only ever sends lowercase.
/// 4. **Whitespace, empty, and zero-width** — the products of trimming, of a mis-split query string,
///    and of a copy-paste through a rich-text field. An implementation that trims before matching
///    accepts a verb the client never sent.
/// 5. **Prefix / substring traps** — `destroyx`, `destro`, `provisioning`. These catch a handler
///    written with `starts_with`/`contains` instead of equality, which is the mechanical form of
///    "mapped it onto a similar-sounding one".
/// 6. **Qualified forms** — `depot:destroy`, `box.destroy`. These catch a handler that strips a
///    namespace before matching, i.e. that accepts strings outside the grammar §5.1 fixes.
///
/// A **seventh** family is not listed here because it is computed per-elemental: a verb valid for a
/// *different* elemental (`attach` on a `bucket`, `console` on a `bucket`, `deploy` on a `box`). See
/// [`cross_elemental_near_misses`].
pub const NEAR_MISS_COINAGES: &[&str] = &[
    // 1. provider-vocabulary synonyms
    "terminate",
    "delete-box",
    "destroy-instance",
    "remove",
    "create",
    "launch",
    "spin-up",
    "reboot",
    "shutdown",
    "describe",
    "get",
    "enumerate",
    "ls",
    "update",
    "modify",
    "logs",
    "metrics",
    "backup",
    "purge",
    "publish",
    "run",
    "call",
    "revert",
    "grow",
    "mount",
    "unmount",
    "download",
    "upload",
    // 2. the privilege cliff by another name
    "exec",
    "ssh",
    "shell",
    "terminal",
    "attach-console",
    // 3. case variants — §5.2 is a closed set of EXACT strings
    "DESTROY",
    "Destroy",
    "Provision",
    "LIST",
    "Console",
    "EXPORT",
    "ReAd",
    // 4. whitespace, empty, zero-width
    "",
    " ",
    "destroy ",
    " destroy",
    "destroy\n",
    "destroy\t",
    "destroy\u{200b}",
    // 5. prefix / substring traps (a `starts_with`/`contains` handler)
    "destroyx",
    "destro",
    "de-stroy",
    "provisioning",
    "re-configure",
    "list-all",
    "export-all",
    // 6. qualified forms (a handler that strips a namespace first)
    "depot:destroy",
    "box.destroy",
    "destroy()",
];

/// The verbs that are real §5.2 registry entries but belong to a **different** elemental, and which
/// this elemental MUST therefore refuse (`attach` on a `bucket`, `deploy` on a `box`).
///
/// This family matters more than the invented coinages: these strings are spelled exactly right and
/// are accepted somewhere at this very coordinator, so an implementation with one flat verb table —
/// the obvious first implementation — accepts every one of them everywhere.
pub fn cross_elemental_near_misses(service: Service) -> Vec<&'static str> {
    Ability::ALL
        .iter()
        .filter(|a| !a.is_valid_for(service))
        .map(|a| a.as_str())
        .collect()
}

/// How many §5.2 verbs `service` must accept (from [`EXPECTED_VERBS`]).
pub fn expected_verbs(service: Service) -> usize {
    EXPECTED_VERBS
        .iter()
        .find(|(s, _)| *s == service)
        .map(|(_, n)| *n)
        .expect("EXPECTED_VERBS covers all four elementals")
}

/// How many near-misses `service` must refuse: the shared coinage corpus plus every registry verb
/// that belongs to another elemental.
pub fn expected_near_misses(service: Service) -> usize {
    NEAR_MISS_COINAGES.len() + (ABILITY_REGISTRY_SIZE - expected_verbs(service))
}

/// The thing under test: something that can answer *"do you accept this ability string?"*.
///
/// Deliberately `&str`, not [`Ability`]. A probe that could only pass a parsed [`Ability`] would be
/// unable to ask the only question that matters — what the coordinator does with a string that is
/// **not** in the registry — and would silently certify the caller's own parser instead of the
/// gateway's.
///
/// `accepts` means *"this ability is a recognised verb for this elemental at this coordinator"*, not
/// *"this caller is authorised"*: a probe run with a token that lacks `destroy` must still see
/// `destroy` recognised. An implementation that cannot separate the two should answer with its
/// vocabulary check alone and say so in the measurement's `evidence`.
pub trait AbilityOracle {
    /// Answer for one `(elemental, ability-string)` pair.
    fn accepts(&mut self, service: Service, ability: &str) -> bool;
}

impl<F> AbilityOracle for F
where
    F: FnMut(Service, &str) -> bool,
{
    fn accepts(&mut self, service: Service, ability: &str) -> bool {
        self(service, ability)
    }
}

/// The outcome of probing **one** elemental. Maps onto exactly one [`DepotMeasurement`], whose
/// `service` field names a single elemental.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbilityProbeReport {
    /// The elemental probed.
    pub service: Service,
    /// How many §5.2 verbs were actually put to the oracle.
    pub verbs_probed: usize,
    /// How many near-miss strings were actually put to the oracle.
    pub near_misses_probed: usize,
    /// Registry verbs the coordinator **refused** — under-implementation.
    pub refused_registry_verbs: Vec<&'static str>,
    /// Near-miss strings the coordinator **accepted** — coinage or silent aliasing, the §5.2
    /// failure this metric exists to catch.
    pub accepted_coinages: Vec<String>,
}

impl AbilityProbeReport {
    /// Whether the probe actually examined the whole subject.
    ///
    /// Compared against the hard-coded [`EXPECTED_VERBS`] / [`NEAR_MISS_COINAGES`] sizes, so an
    /// oracle that was never called (zero of each) reports `false` rather than "no findings".
    pub fn coverage_is_complete(&self) -> bool {
        self.verbs_probed == expected_verbs(self.service)
            && self.near_misses_probed == expected_near_misses(self.service)
    }

    /// The metric's boolean value: complete coverage **and** no finding in either direction.
    pub fn passed(&self) -> bool {
        self.coverage_is_complete()
            && self.refused_registry_verbs.is_empty()
            && self.accepted_coinages.is_empty()
    }

    /// Render this report as the §7 ATTEST claim body, `metric = ability-conformance`,
    /// `method = probe`, `value = passed()`.
    ///
    /// `evidence` SHOULD be a [`EvidenceKind::Recipe`] naming how to re-run this — §7 says a consumer
    /// SHOULD re-run a reproducible probe rather than trust the reported value, and a measurement
    /// with no recipe is not independently checkable
    /// ([`DepotMeasurement::is_independently_checkable`]).
    pub fn into_measurement(
        self,
        observed_at: TimestampMs,
        evidence: Option<(EvidenceKind, String)>,
    ) -> DepotMeasurement {
        DepotMeasurement {
            service: self.service,
            metric: Metric::AbilityConformance,
            value: MeasurementValue::Bool(self.passed()),
            method: Method::Probe,
            observed_at,
            evidence,
        }
    }
}

impl fmt::Display for AbilityProbeReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "ability-conformance[{}]: {} — {}/{} registry verbs probed, {}/{} near-misses probed",
            self.service.as_str(),
            if self.passed() { "PASS" } else { "FAIL" },
            self.verbs_probed,
            expected_verbs(self.service),
            self.near_misses_probed,
            expected_near_misses(self.service),
        )?;
        if !self.coverage_is_complete() {
            writeln!(
                f,
                "  INCOMPLETE COVERAGE — this is a failure, not an absence of findings"
            )?;
        }
        for v in &self.refused_registry_verbs {
            writeln!(f, "  REFUSED a §5.2 registry verb: {v:?}")?;
        }
        for v in &self.accepted_coinages {
            writeln!(
                f,
                "  ACCEPTED a non-registry verb: {v:?} — coinage or silent aliasing (§5.2)"
            )?;
        }
        write!(f, "  {VACUITY_CAVEAT}")
    }
}

/// The result of probing all four elementals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbilityConformance {
    /// One report per elemental, in [`Service::ALL`] order.
    pub reports: [AbilityProbeReport; 4],
}

impl AbilityConformance {
    /// True only if every elemental passed with complete coverage.
    pub fn passed(&self) -> bool {
        self.reports.iter().all(AbilityProbeReport::passed)
    }

    /// Total §5.2 verbs put to the oracle across all four elementals.
    pub fn verbs_probed(&self) -> usize {
        self.reports.iter().map(|r| r.verbs_probed).sum()
    }

    /// Total near-miss strings put to the oracle across all four elementals.
    pub fn near_misses_probed(&self) -> usize {
        self.reports.iter().map(|r| r.near_misses_probed).sum()
    }

    /// One [`DepotMeasurement`] per elemental — §7's schema carries a single `service`, so a
    /// whole-coordinator result is four claims, not one averaged one.
    pub fn measurements(
        &self,
        observed_at: TimestampMs,
        evidence: Option<(EvidenceKind, String)>,
    ) -> Vec<DepotMeasurement> {
        self.reports
            .iter()
            .map(|r| r.clone().into_measurement(observed_at, evidence.clone()))
            .collect()
    }
}

impl fmt::Display for AbilityConformance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "ability-conformance: {} — {} registry verbs and {} near-misses probed across {} \
             elementals",
            if self.passed() { "PASS" } else { "FAIL" },
            self.verbs_probed(),
            self.near_misses_probed(),
            self.reports.len(),
        )?;
        for r in &self.reports {
            writeln!(f, "{r}")?;
        }
        Ok(())
    }
}

/// Probe one elemental's §5.2 vocabulary (§7 `ability-conformance`).
///
/// Every registry verb valid for `service` MUST be accepted; every entry of
/// [`NEAR_MISS_COINAGES`] and every [`cross_elemental_near_misses`] verb MUST be refused. The
/// returned report carries the counts, so an oracle that was never reached fails rather than passes.
pub fn probe_service<O: AbilityOracle + ?Sized>(
    service: Service,
    oracle: &mut O,
) -> AbilityProbeReport {
    let mut verbs_probed = 0usize;
    let mut near_misses_probed = 0usize;
    let mut refused_registry_verbs = Vec::new();
    let mut accepted_coinages = Vec::new();

    for ability in Ability::ALL {
        if !ability.is_valid_for(service) {
            continue;
        }
        verbs_probed += 1;
        if !oracle.accepts(service, ability.as_str()) {
            refused_registry_verbs.push(ability.as_str());
        }
    }

    for coinage in NEAR_MISS_COINAGES.iter().copied() {
        near_misses_probed += 1;
        if oracle.accepts(service, coinage) {
            accepted_coinages.push(coinage.to_string());
        }
    }

    for verb in cross_elemental_near_misses(service) {
        near_misses_probed += 1;
        if oracle.accepts(service, verb) {
            accepted_coinages.push(verb.to_string());
        }
    }

    AbilityProbeReport {
        service,
        verbs_probed,
        near_misses_probed,
        refused_registry_verbs,
        accepted_coinages,
    }
}

/// Probe all four elementals (§3) against one oracle.
pub fn probe_all_elementals<O: AbilityOracle>(mut oracle: O) -> AbilityConformance {
    AbilityConformance {
        reports: [
            probe_service(Service::ALL[0], &mut oracle),
            probe_service(Service::ALL[1], &mut oracle),
            probe_service(Service::ALL[2], &mut oracle),
            probe_service(Service::ALL[3], &mut oracle),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A conformant oracle: the reference registry itself, scoped per elemental.
    fn conformant(service: Service, ability: &str) -> bool {
        Ability::from_str(ability).is_some_and(|a| a.is_valid_for(service))
    }

    // ---- the probe's own expectations must match the registry it claims to cover ----

    #[test]
    fn registry_size_constant_matches_the_enum() {
        assert_eq!(ABILITY_REGISTRY_SIZE, Ability::ALL.len());
        // ...and every entry is distinct, so a duplicated variant cannot pad the count.
        let mut wire: Vec<&str> = Ability::ALL.iter().map(|a| a.as_str()).collect();
        wire.sort_unstable();
        let before = wire.len();
        wire.dedup();
        assert_eq!(before, wire.len(), "Ability::ALL has a duplicate");
        // ...and every entry round-trips, so ALL cannot contain a verb `from_str` does not know.
        for a in Ability::ALL {
            assert_eq!(Ability::from_str(a.as_str()), Some(a), "{a:?}");
        }
    }

    #[test]
    fn hard_coded_verb_counts_match_the_registry() {
        // The hard-coded expectation is what makes an empty iteration a FAIL. It is only safe if it
        // agrees with §5.2 — so a registry addition breaks this test loudly and must be reflected in
        // EXPECTED_VERBS by hand.
        for s in Service::ALL {
            let derived = Ability::ALL.iter().filter(|a| a.is_valid_for(s)).count();
            assert_eq!(expected_verbs(s), derived, "{s:?}");
            assert_eq!(
                expected_near_misses(s),
                NEAR_MISS_COINAGES.len() + (ABILITY_REGISTRY_SIZE - derived),
                "{s:?}"
            );
        }
        assert_eq!(
            EXPECTED_VERBS.iter().map(|(_, n)| n).sum::<usize>(),
            44,
            "44 = 4x7 common + 5 box + 4 volume + 4 bucket + 3 edge-fn"
        );
    }

    #[test]
    fn no_near_miss_is_secretly_a_registry_verb() {
        // The false-positive control. If a coinage in the corpus were actually a §5.2 verb, a
        // CONFORMANT coordinator would fail the probe — and the probe would be worse than useless.
        for c in NEAR_MISS_COINAGES {
            assert_eq!(Ability::from_str(c), None, "{c:?} is a real registry verb");
        }
    }

    #[test]
    fn the_mandated_near_misses_are_all_present() {
        // The floor the profile itself names, pinned so a later edit cannot quietly drop one.
        for required in [
            "terminate",
            "delete-box",
            "remove",
            "create",
            "reboot",
            "exec",
            "ssh",
            "DESTROY",
            "Destroy",
            "destroy ",
            "",
        ] {
            assert!(
                NEAR_MISS_COINAGES.contains(&required),
                "{required:?} must stay in the near-miss corpus"
            );
        }
        assert!(NEAR_MISS_COINAGES.len() >= 50, "corpus shrank unexpectedly");
        // `attach` on a bucket — a verb valid for a DIFFERENT elemental — is the computed family.
        assert!(cross_elemental_near_misses(Service::Bucket).contains(&"attach"));
        assert!(cross_elemental_near_misses(Service::Bucket).contains(&"console"));
        assert!(cross_elemental_near_misses(Service::Box).contains(&"deploy"));
        assert!(!cross_elemental_near_misses(Service::Volume).contains(&"snapshot"));
    }

    // ---- the probe passes a conformant implementation ----

    #[test]
    fn a_conformant_oracle_passes_with_full_coverage() {
        let out = probe_all_elementals(conformant);
        assert!(out.passed(), "{out}");
        assert_eq!(out.verbs_probed(), 44);
        assert_eq!(
            out.near_misses_probed(),
            NEAR_MISS_COINAGES.len() * 4 + (22 - 11) + (22 - 11) + (22 - 10) + (22 - 12)
        );
    }

    // ---- ...and catches BOTH failure directions ----

    #[test]
    fn an_aliasing_oracle_fails() {
        // The §5.2 failure by name: `terminate` accepted as a synonym for `destroy`.
        let out = probe_all_elementals(|s: Service, a: &str| {
            a == "terminate" && s == Service::Box || conformant(s, a)
        });
        assert!(!out.passed());
        assert!(out.reports[3]
            .accepted_coinages
            .contains(&"terminate".to_string()));
        // Coverage was still complete — the failure is a finding, not a hole.
        assert!(out.reports[3].coverage_is_complete());
    }

    #[test]
    fn a_case_insensitive_oracle_fails() {
        // The likeliest accidental exit from the registry, and one a lowercase-only test never sees.
        let out = probe_all_elementals(|s: Service, a: &str| conformant(s, &a.to_lowercase()));
        assert!(!out.passed());
        assert!(out.reports[3]
            .accepted_coinages
            .contains(&"DESTROY".to_string()));
    }

    #[test]
    fn a_trimming_oracle_fails() {
        let out = probe_all_elementals(|s: Service, a: &str| conformant(s, a.trim()));
        assert!(!out.passed());
        assert!(out.reports[0]
            .accepted_coinages
            .contains(&"destroy ".to_string()));
    }

    #[test]
    fn a_prefix_matching_oracle_fails() {
        let out = probe_all_elementals(|s: Service, a: &str| {
            !a.is_empty()
                && Ability::ALL
                    .iter()
                    .any(|k| k.is_valid_for(s) && a.starts_with(k.as_str()))
        });
        assert!(!out.passed());
        assert!(out.reports[0]
            .accepted_coinages
            .contains(&"destroyx".to_string()));
    }

    #[test]
    fn a_flat_verb_table_oracle_fails() {
        // One table for all four elementals: every string is spelled right, and every elemental
        // answers to every other's verbs. This is the obvious first implementation.
        let out = probe_all_elementals(|_s: Service, a: &str| Ability::from_str(a).is_some());
        assert!(!out.passed());
        assert!(out.reports[0]
            .accepted_coinages
            .contains(&"attach".to_string()));
    }

    #[test]
    fn an_under_implementing_oracle_fails() {
        // The other direction. A gateway that does not answer to `export` is not driveable by the
        // common client either — and a probe blind to this certifies half a vocabulary.
        let out = probe_all_elementals(|s: Service, a: &str| a != "export" && conformant(s, a));
        assert!(!out.passed());
        for r in &out.reports {
            assert!(r.refused_registry_verbs.contains(&"export"), "{r}");
            assert!(r.accepted_coinages.is_empty());
            assert!(r.coverage_is_complete());
        }
    }

    #[test]
    fn an_oracle_that_refuses_everything_fails() {
        let out = probe_all_elementals(|_s: Service, _a: &str| false);
        assert!(!out.passed());
        assert_eq!(out.reports[0].refused_registry_verbs.len(), 11);
    }

    #[test]
    fn an_oracle_that_accepts_everything_fails() {
        let out = probe_all_elementals(|_s: Service, _a: &str| true);
        assert!(!out.passed());
        assert_eq!(
            out.reports[0].accepted_coinages.len(),
            expected_near_misses(Service::Bucket)
        );
    }

    // ---- a probe that examined nothing must not read as a pass ----

    #[test]
    fn an_empty_iteration_is_a_failure_not_a_pass() {
        let hollow = AbilityProbeReport {
            service: Service::Box,
            verbs_probed: 0,
            near_misses_probed: 0,
            refused_registry_verbs: Vec::new(),
            accepted_coinages: Vec::new(),
        };
        assert!(!hollow.coverage_is_complete());
        assert!(
            !hollow.passed(),
            "no findings must not be confused with nothing checked"
        );
        // ...and it says so, rather than printing a bare FAIL with no findings listed.
        assert!(hollow.to_string().contains("INCOMPLETE COVERAGE"));
        // One short of the floor also fails.
        let short = AbilityProbeReport {
            verbs_probed: expected_verbs(Service::Box) - 1,
            near_misses_probed: expected_near_misses(Service::Box),
            ..hollow.clone()
        };
        assert!(!short.passed());
        let short_misses = AbilityProbeReport {
            verbs_probed: expected_verbs(Service::Box),
            near_misses_probed: expected_near_misses(Service::Box) - 1,
            ..hollow
        };
        assert!(!short_misses.passed());
    }

    // ---- the measurement mapping ----

    #[test]
    fn a_report_becomes_an_ability_conformance_measurement() {
        let out = probe_all_elementals(conformant);
        let ms = out.measurements(
            1_754_000_000_000,
            Some((
                EvidenceKind::Recipe,
                "cargo test -p kotva-depot probe".into(),
            )),
        );
        assert_eq!(ms.len(), 4);
        for (m, s) in ms.iter().zip(Service::ALL) {
            assert_eq!(m.service, s);
            assert_eq!(m.metric, Metric::AbilityConformance);
            assert_eq!(m.method, Method::Probe);
            assert_eq!(m.value, MeasurementValue::Bool(true));
            assert!(m.is_wellformed(), "§7 types ability-conformance as a bool");
            assert!(m.is_independently_checkable());
            // ...and it survives the wire.
            assert_eq!(&DepotMeasurement::from_det_cbor(&m.det_cbor()).unwrap(), m);
        }
    }

    #[test]
    fn a_failing_probe_measures_false_rather_than_being_omitted() {
        let out = probe_all_elementals(|s: Service, a: &str| a == "terminate" || conformant(s, a));
        let ms = out.measurements(1_754_000_000_000, None);
        assert!(ms
            .iter()
            .all(|m| m.value == MeasurementValue::Bool(false) && m.is_wellformed()));
        // No evidence recipe => §7's re-run advice cannot be followed.
        assert!(!ms[0].is_independently_checkable());
    }

    // ---- §7's vacuity caveat must survive ----

    #[test]
    fn every_rendering_carries_the_vacuity_caveat() {
        let out = probe_all_elementals(conformant);
        assert!(out.passed());
        assert!(VACUITY_CAVEAT.contains("VACUOUS"));
        assert!(VACUITY_CAVEAT.contains("MUST NOT"));
        assert!(VACUITY_CAVEAT.contains("conformance/SUITE.md"));
        // A pass cannot be quoted without the caveat attached to it.
        assert!(out.to_string().contains(VACUITY_CAVEAT));
        for r in &out.reports {
            assert!(r.to_string().contains(VACUITY_CAVEAT));
        }
    }
}
