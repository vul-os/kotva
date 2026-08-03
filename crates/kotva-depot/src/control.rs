//! The control plane — the `resource`/`ability` vocabulary for `CapabilityToken` (§5).
//!
//! DEPOT-11 requires the control plane to be a capability rather than an API key. That is necessary
//! and **not sufficient**: [`kotva_core::capability::Capability`] carries `resource` and `ability` as
//! free **text**, so two conformant gateways could name the same operation differently and fail to
//! interoperate *silently* — unlike an unrecognised **caveat**, which fails closed by construction
//! (§18.7.3). This module closes that hole. It adds no wire object: it fixes what goes *into* the
//! existing token.

use crate::{DepotError, Service};

/// A DEPOT resource reference (§5.1).
///
/// ```text
/// depot:<service>/<instance-id>     one instance, e.g. depot:box/7f3a91c2
/// depot:<service>/*                 every instance of one service at this coordinator
/// depot:*                           every DEPOT resource at this coordinator
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceRef {
    /// `depot:*` — every DEPOT resource at this coordinator.
    AllServices,
    /// `depot:<service>/*` — every instance of one elemental.
    AllInstances(Service),
    /// `depot:<service>/<instance-id>` — one instance. The id is operator-assigned and opaque.
    Instance(Service, String),
}

impl ResourceRef {
    /// Render to the wire string.
    pub fn to_wire(&self) -> String {
        match self {
            ResourceRef::AllServices => "depot:*".to_string(),
            ResourceRef::AllInstances(s) => format!("depot:{}/*", s.as_str()),
            ResourceRef::Instance(s, id) => format!("depot:{}/{}", s.as_str(), id),
        }
    }

    /// Parse a wire string, failing closed on anything malformed or on an unknown service.
    pub fn parse(s: &str) -> Result<Self, DepotError> {
        let rest = s
            .strip_prefix("depot:")
            .ok_or_else(|| DepotError::MalformedResource(s.to_string()))?;
        if rest == "*" {
            return Ok(ResourceRef::AllServices);
        }
        let (svc, inst) = rest
            .split_once('/')
            .ok_or_else(|| DepotError::MalformedResource(s.to_string()))?;
        let service = Service::from_str(svc).ok_or_else(|| DepotError::UnknownRegistryValue {
            registry: "service",
            value: svc.to_string(),
        })?;
        if inst == "*" {
            return Ok(ResourceRef::AllInstances(service));
        }
        if inst.is_empty() || inst.contains('/') {
            return Err(DepotError::MalformedResource(s.to_string()));
        }
        Ok(ResourceRef::Instance(service, inst.to_string()))
    }

    /// The elemental this reference names, if it names one.
    pub fn service(&self) -> Option<Service> {
        match self {
            ResourceRef::AllServices => None,
            ResourceRef::AllInstances(s) | ResourceRef::Instance(s, _) => Some(*s),
        }
    }

    /// Whether `self` **contains** `other` — the attenuation predicate (§5.1, §18.7.3).
    ///
    /// A child token may only narrow, so a delegated grant is valid iff the parent's resource
    /// `covers` the child's. Ordering is `depot:*` ⊃ `depot:<svc>/*` ⊃ `depot:<svc>/<id>`.
    pub fn covers(&self, other: &ResourceRef) -> bool {
        match (self, other) {
            (ResourceRef::AllServices, _) => true,
            (ResourceRef::AllInstances(a), ResourceRef::AllInstances(b)) => a == b,
            (ResourceRef::AllInstances(a), ResourceRef::Instance(b, _)) => a == b,
            (ResourceRef::Instance(a, ia), ResourceRef::Instance(b, ib)) => a == b && ia == ib,
            // A narrower scope never covers a broader one.
            (ResourceRef::AllInstances(_), ResourceRef::AllServices) => false,
            (ResourceRef::Instance(..), _) => false,
        }
    }
}

/// The **closed** ability registry (§5.2).
///
/// Extended by registry addition, never by an operator coining a verb. An unknown ability MUST be
/// refused rather than mapped onto a similar-sounding one — that refusal is what makes a single
/// open-source client able to drive any conformant gateway.
///
/// **`#[non_exhaustive]` here, unlike [`Service`], and the asymmetry is deliberate.** The elementals
/// are closed at four and a fifth would be a breaking change that should read as one. The verb set
/// is closed *to operators* but genuinely **extensible by registry addition** (§5.2) — a new verb is
/// expected over time — so a consumer must not assume today's list is final. Closed to coinage,
/// open to the registry: those are different properties and they get different attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Ability {
    // ---- common lifecycle, valid for all four elementals ----
    /// Create an instance from a declared shape (and, for `box`/`edge-fn`, a `DepotImage`, §4.2).
    Provision,
    /// Read one instance's configuration and state.
    Inspect,
    /// Enumerate instances in scope.
    List,
    /// Mutate declared configuration of an existing instance.
    Reconfigure,
    /// Read logs and metrics (§4.3, DEPOT-14).
    Observe,
    /// Obtain the DEPOT-4 portable export.
    Export,
    /// Delete an instance and release its resources.
    Destroy,

    // ---- box ----
    /// Start a stopped box.
    Start,
    /// Stop a running box.
    Stop,
    /// Restart a box.
    Restart,
    /// Capture a snapshot (`box` or `volume`, §4.1).
    Snapshot,
    /// **Interactive access to a box — the privilege cliff (§5.2).** Subsumes nearly every other
    /// ability: it can read secrets, alter state, and forge the evidence of having done so.
    Console,

    // ---- volume ----
    /// Attach a volume to a box.
    Attach,
    /// Detach a volume from a box.
    Detach,
    /// Grow a volume.
    Resize,

    // ---- bucket ----
    /// Read objects.
    Read,
    /// Write objects.
    Write,
    /// Delete objects.
    Delete,
    /// Toggle public serving (the CDN mode, §3.7).
    Serve,

    // ---- edge-fn ----
    /// Publish a new artefact version.
    Deploy,
    /// Call the function.
    Invoke,
    /// Point back at a previously deployed artefact.
    Rollback,
}

impl Ability {
    /// The wire verb.
    pub fn as_str(self) -> &'static str {
        use Ability::*;
        match self {
            Provision => "provision",
            Inspect => "inspect",
            List => "list",
            Reconfigure => "reconfigure",
            Observe => "observe",
            Export => "export",
            Destroy => "destroy",
            Start => "start",
            Stop => "stop",
            Restart => "restart",
            Snapshot => "snapshot",
            Console => "console",
            Attach => "attach",
            Detach => "detach",
            Resize => "resize",
            Read => "read",
            Write => "write",
            Delete => "delete",
            Serve => "serve",
            Deploy => "deploy",
            Invoke => "invoke",
            Rollback => "rollback",
        }
    }

    /// Parse a wire verb. **Fails closed** on anything outside the registry (§5.2, DEPOT-9): a
    /// coordinator receiving `terminate` MUST refuse rather than infer `destroy`.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        use Ability::*;
        Some(match s {
            "provision" => Provision,
            "inspect" => Inspect,
            "list" => List,
            "reconfigure" => Reconfigure,
            "observe" => Observe,
            "export" => Export,
            "destroy" => Destroy,
            "start" => Start,
            "stop" => Stop,
            "restart" => Restart,
            "snapshot" => Snapshot,
            "console" => Console,
            "attach" => Attach,
            "detach" => Detach,
            "resize" => Resize,
            "read" => Read,
            "write" => Write,
            "delete" => Delete,
            "serve" => Serve,
            "deploy" => Deploy,
            "invoke" => Invoke,
            "rollback" => Rollback,
            _ => return None,
        })
    }

    /// Every verb in the §5.2 registry, in table order: the seven common lifecycle verbs, then
    /// `box`, `volume`, `bucket`, `edge-fn`.
    ///
    /// This is the enumeration a probe iterates ([`crate::probe`]). It is deliberately a hand-written
    /// array rather than a derive: adding a variant without adding it here leaves the new verb
    /// unprobed, and [`crate::probe::ABILITY_REGISTRY_SIZE`] is the assertion that catches that.
    pub const ALL: [Ability; 22] = [
        Ability::Provision,
        Ability::Inspect,
        Ability::List,
        Ability::Reconfigure,
        Ability::Observe,
        Ability::Export,
        Ability::Destroy,
        Ability::Start,
        Ability::Stop,
        Ability::Restart,
        Ability::Snapshot,
        Ability::Console,
        Ability::Attach,
        Ability::Detach,
        Ability::Resize,
        Ability::Read,
        Ability::Write,
        Ability::Delete,
        Ability::Serve,
        Ability::Deploy,
        Ability::Invoke,
        Ability::Rollback,
    ];

    /// The seven abilities every elemental accepts (§5.2).
    pub const COMMON: [Ability; 7] = [
        Ability::Provision,
        Ability::Inspect,
        Ability::List,
        Ability::Reconfigure,
        Ability::Observe,
        Ability::Export,
        Ability::Destroy,
    ];

    /// Whether this verb is meaningful for `service` (§5.2).
    ///
    /// `snapshot` is shared by `box` and `volume`; everything else outside [`Ability::COMMON`]
    /// belongs to exactly one elemental.
    pub fn is_valid_for(self, service: Service) -> bool {
        use Ability::*;
        if Ability::COMMON.contains(&self) {
            return true;
        }
        match service {
            Service::Box => matches!(self, Start | Stop | Restart | Snapshot | Console),
            Service::Volume => matches!(self, Attach | Detach | Resize | Snapshot),
            Service::Bucket => matches!(self, Read | Write | Delete | Serve),
            Service::EdgeFn => matches!(self, Deploy | Invoke | Rollback),
        }
    }

    /// Whether this ability is the **privilege cliff** that MUST be separately delegated (§5.2).
    ///
    /// A token granting [`Ability::Console`] MUST NOT be issued as an implicit consequence of
    /// granting `provision` or `reconfigure`.
    pub fn requires_separate_delegation(self) -> bool {
        matches!(self, Ability::Console)
    }
}

/// The caveat key binding a DEPOT capability to the coordinator it is meant for (§5.1).
///
/// A resource string names no operator — instance ids are operator-assigned and opaque, so
/// `depot:box/7f3a91c2` denotes different machines at different coordinators — and
/// `CapabilityToken.aud` binds the **delegatee**, not the target. Without this caveat a credential
/// delegated to something that talks to two gateways is presentable at both: a confused deputy.
///
/// §18.7.3 evaluates every caveat on every link of the chain and **fails closed on any it does not
/// recognise**, so this costs no new verification step: a non-DEPOT coordinator rejects the token
/// automatically, and a DEPOT one performs one key comparison.
pub const CAVEAT_COORDINATOR: &str = "depot:coordinator";

/// Check that a capability is bound to *this* coordinator (§5.1, normative).
///
/// `caveat_value` is the token's `depot:coordinator` caveat, absent if the token carries none.
/// A missing binding fails — a token that names no coordinator is valid everywhere, which is the
/// whole defect this closes.
pub fn check_coordinator_binding(
    caveat_value: Option<&[u8]>,
    own_ik: &[u8],
) -> Result<(), DepotError> {
    match caveat_value {
        Some(v) if v == own_ik => Ok(()),
        _ => Err(DepotError::CoordinatorBindingMissing),
    }
}

/// Check that every ability is meaningful for the elemental it is scoped to (§5.2).
///
/// **This deliberately does not require `export` alongside `destroy`.** DEPOT-4's exit obligation
/// binds what an *operator offers* — see [`check_operator_offering`] — not what an individual grant
/// carries. Requiring the pair on every token would turn a cleanup credential (a CI job reaping
/// preview environments) into a data-exfiltration credential, inverting least privilege. Delegated
/// grants narrow freely and independently.
pub fn check_grant(service: Service, abilities: &[Ability]) -> Result<(), DepotError> {
    for a in abilities {
        if !a.is_valid_for(service) {
            return Err(DepotError::UnknownRegistryValue {
                registry: "ability-for-service",
                value: format!("{} on {}", a.as_str(), service.as_str()),
            });
        }
    }
    Ok(())
}

/// Check an **operator's declared ability set** against DEPOT-4's exit obligation (§5.2).
///
/// A coordinator that offers `destroy` while withholding `export` has made the exit weaker than the
/// loss: the account holder can delete an instance but can never extract it. This is the rule that
/// belongs to the operator's product, checked against the `abilities` it declares in its policy
/// (§3.3) — never against a delegated token.
pub fn check_operator_offering(offered: &[Ability]) -> Result<(), DepotError> {
    if offered.contains(&Ability::Destroy) && !offered.contains(&Ability::Export) {
        return Err(DepotError::DestroyWithoutExport);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_round_trips() {
        for wire in ["depot:*", "depot:box/*", "depot:box/7f3a91c2", "depot:bucket/photos"] {
            let r = ResourceRef::parse(wire).unwrap();
            assert_eq!(r.to_wire(), wire);
        }
    }

    #[test]
    fn malformed_resources_fail_closed() {
        for bad in ["box/7f3a", "depot:", "depot:box", "depot:vm/*", "depot:box/", "depot:box/a/b", ""] {
            assert!(ResourceRef::parse(bad).is_err(), "{bad:?} must not parse");
        }
    }

    #[test]
    fn attenuation_is_one_directional() {
        let all = ResourceRef::parse("depot:*").unwrap();
        let all_boxes = ResourceRef::parse("depot:box/*").unwrap();
        let one_box = ResourceRef::parse("depot:box/7f3a").unwrap();
        let other_box = ResourceRef::parse("depot:box/aaaa").unwrap();
        let all_buckets = ResourceRef::parse("depot:bucket/*").unwrap();

        // Broader covers narrower.
        assert!(all.covers(&all_boxes));
        assert!(all.covers(&one_box));
        assert!(all_boxes.covers(&one_box));
        assert!(all_boxes.covers(&all_boxes));
        assert!(one_box.covers(&one_box));

        // Narrower NEVER covers broader — the direction that matters, since getting it backwards
        // silently widens every delegated token.
        assert!(!all_boxes.covers(&all));
        assert!(!one_box.covers(&all_boxes));
        assert!(!one_box.covers(&all));

        // Siblings never cover each other.
        assert!(!one_box.covers(&other_box));
        assert!(!all_boxes.covers(&all_buckets));
        assert!(!all_buckets.covers(&one_box));
    }

    #[test]
    fn unknown_ability_fails_closed_and_is_not_aliased() {
        // The exact coinages a catalogue-minded implementer reaches for. Each MUST fail rather than
        // resolve to the nearby real verb (§5.2).
        for bad in ["terminate", "delete-box", "create", "remove", "reboot", "exec", "ssh", "DESTROY", ""] {
            assert_eq!(Ability::from_str(bad), None, "{bad:?} must not parse");
        }
        assert_eq!(Ability::from_str("destroy"), Some(Ability::Destroy));
    }

    #[test]
    fn ability_wire_round_trips() {
        let all = [
            Ability::Provision, Ability::Inspect, Ability::List, Ability::Reconfigure,
            Ability::Observe, Ability::Export, Ability::Destroy, Ability::Start, Ability::Stop,
            Ability::Restart, Ability::Snapshot, Ability::Console, Ability::Attach,
            Ability::Detach, Ability::Resize, Ability::Read, Ability::Write, Ability::Delete,
            Ability::Serve, Ability::Deploy, Ability::Invoke, Ability::Rollback,
        ];
        for a in all {
            assert_eq!(Ability::from_str(a.as_str()), Some(a), "{a:?}");
        }
    }

    #[test]
    fn abilities_are_scoped_to_their_elemental() {
        assert!(Ability::Console.is_valid_for(Service::Box));
        assert!(!Ability::Console.is_valid_for(Service::Bucket));
        assert!(Ability::Attach.is_valid_for(Service::Volume));
        assert!(!Ability::Attach.is_valid_for(Service::EdgeFn));
        assert!(Ability::Deploy.is_valid_for(Service::EdgeFn));
        assert!(!Ability::Deploy.is_valid_for(Service::Box));
        // snapshot is deliberately shared by the two stateful elementals.
        assert!(Ability::Snapshot.is_valid_for(Service::Box));
        assert!(Ability::Snapshot.is_valid_for(Service::Volume));
        assert!(!Ability::Snapshot.is_valid_for(Service::Bucket));
        // The common seven apply everywhere.
        for s in Service::ALL {
            for a in Ability::COMMON {
                assert!(a.is_valid_for(s), "{a:?} must be valid for {s:?}");
            }
        }
    }

    #[test]
    fn console_must_be_separately_delegated() {
        assert!(Ability::Console.requires_separate_delegation());
        assert!(!Ability::Provision.requires_separate_delegation());
        assert!(!Ability::Reconfigure.requires_separate_delegation());
    }

    #[test]
    fn operator_offering_must_not_gate_export_behind_destroy() {
        // The obligation binds the operator's PRODUCT (DEPOT-4).
        assert_eq!(
            check_operator_offering(&[Ability::Destroy, Ability::Inspect]),
            Err(DepotError::DestroyWithoutExport)
        );
        check_operator_offering(&[Ability::Destroy, Ability::Export]).unwrap();
        // Offering export without destroy is fine — you may always leave without being able to
        // delete.
        check_operator_offering(&[Ability::Export]).unwrap();
    }

    #[test]
    fn a_delegated_grant_may_carry_destroy_alone() {
        // The inverse of the operator rule, and the reason the two are separate functions: a CI job
        // that reaps preview environments must NOT be forced to also carry data extraction.
        check_grant(Service::Box, &[Ability::Destroy]).unwrap();
        check_grant(Service::Box, &[Ability::Destroy, Ability::Inspect]).unwrap();
        // ...and least privilege in the other direction still works.
        check_grant(Service::Box, &[Ability::Export]).unwrap();
    }

    #[test]
    fn unbound_or_misbound_capability_is_refused() {
        let me = b"coordinator-a-ik";
        let them = b"coordinator-b-ik";
        check_coordinator_binding(Some(me), me).unwrap();
        // The confused-deputy case: a token minted for another gateway.
        assert_eq!(
            check_coordinator_binding(Some(them), me),
            Err(DepotError::CoordinatorBindingMissing)
        );
        // An unbound token is valid everywhere, which is the defect — so it must fail here.
        assert_eq!(
            check_coordinator_binding(None, me),
            Err(DepotError::CoordinatorBindingMissing)
        );
        // Empty is not a wildcard.
        assert_eq!(
            check_coordinator_binding(Some(b""), me),
            Err(DepotError::CoordinatorBindingMissing)
        );
    }

    #[test]
    fn grant_rejects_ability_meaningless_for_the_service() {
        assert!(check_grant(Service::Bucket, &[Ability::Console]).is_err());
        check_grant(Service::Bucket, &[Ability::Read, Ability::Write]).unwrap();
    }
}
