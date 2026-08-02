//! [`CoordinatorKind`] — the `kind` field, key 2 of a `CoordinatorDescriptor` (§18.8a.1).
//!
//! The registry is **`coordinator/CONTRACT.md` §5**, which states it is "the single canonical,
//! authoritative list of coordinator kinds for the entire KOTVA family… **eleven** kinds — ten
//! fully-specified and one (`infra-service`) draft — and no other document may enumerate a
//! different count or add a kind not listed here."
//!
//! # Divergence found while writing this (reported and fixed, not silently copied)
//!
//! The §18.8a.1 `kind` table row said "One of the CONTRACT §5 canonical kind strings" and then
//! **enumerated twelve**, including `"compute"`. CONTRACT §5 has no `compute` row: its own design
//! note ("Why there is no separate `compute` kind") records that it folded into `infra-service`,
//! because invoking a model endpoint is "an `edge-fn` with `artifact-source = operator` on a class
//! declaring `gpu-count` — an **attribute wearing a kind's clothes**", and two kinds both meaning
//! "run code on your machine" left a client with nothing to disambiguate them by.
//! `profiles/cloud.md` §5 records the same fold, and the §18.8a preamble paragraph one screen above
//! the table already listed the post-fold ten inherited kinds with no `compute`.
//!
//! The parenthetical enumeration was therefore stale relative to the registry it defers to, and
//! CONTRACT §5 forbids a second document enumerating a different list. **The spec row has been
//! amended** to the eleven and to say explicitly that `"compute"` is not a kind. This crate
//! implements those eleven, and `"compute"` is a decode-time reject — exercised as a corruption
//! control in `tests/vectors.rs`, because a folded kind that still parses is precisely how a retired
//! concept survives in the field.

use crate::CoordinatorError;

/// A coordinator kind — an instance of the one contract (CONTRACT §5).
///
/// Deliberately **not** `#[non_exhaustive]`, matching `kotva_depot::Service`: the list is canonical
/// and exhaustive, and forcing consumers into a wildcard arm would invite exactly the silent
/// catch-all handling §18.8a.1 rejects when it requires an unknown `kind` be treated as an
/// *undeclared* coordinator. A twelfth kind would be a breaking change and should read as one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CoordinatorKind {
    /// Legacy-mail bridge — MX, DKIM egress, legacy client surfaces (§7). The first fully-worked
    /// instance of the contract; a legacy `adapter` (§26) is this kind too, not a parallel scheme.
    Gateway,
    /// Mesh reachability for NAT'd peers (Circuit Relay v2).
    Relay,
    /// Forwards SFrame-encrypted call/stream media; scales calls (RFC 9605, §27).
    MediaRelay,
    /// ngrok-style public subdomains for arbitrary box services (`profiles/reachability.md`).
    ReachabilityAdapter,
    /// Managed infrastructure — the four DEPOT elementals `box`/`bucket`/`volume`/`edge-fn`, with
    /// `database`, `queue`, `cdn`, image `registry`, static `site` and hosted inference as
    /// **formulas** composing them (`profiles/cloud.md` §3.6–§3.7). **Draft**, and deferred out of
    /// Core v1 (CONTRACT §5 ratification tier). Its descriptor `policy` blob is
    /// `kotva_depot::DepotServicePolicy`.
    InfraService,
    /// Search / discovery / global product-and-price view.
    Indexer,
    /// Moderation labels — opt-in, subscribable.
    Labeler,
    /// Real-time supply↔demand matching (rides, delivery).
    Matcher,
    /// Dispute resolution (staked jury).
    Arbiter,
    /// Physical-world / real-fact attestation (delivered? ride done?).
    Oracle,
    /// Holds the trade float for a trade window — the family's **one load-bearing exception**
    /// (CONTRACT §1, `primitives/ESCROW.md` §9–§10, SEC-6a), confined to the commerce extension
    /// and not in Core v1. Disclosed, not hidden.
    CustodialEscrow,
}

impl CoordinatorKind {
    /// The wire string (§18.8a.1 key 2).
    pub fn as_str(self) -> &'static str {
        match self {
            CoordinatorKind::Gateway => "gateway",
            CoordinatorKind::Relay => "relay",
            CoordinatorKind::MediaRelay => "media-relay",
            CoordinatorKind::ReachabilityAdapter => "reachability-adapter",
            CoordinatorKind::InfraService => "infra-service",
            CoordinatorKind::Indexer => "indexer",
            CoordinatorKind::Labeler => "labeler",
            CoordinatorKind::Matcher => "matcher",
            CoordinatorKind::Arbiter => "arbiter",
            CoordinatorKind::Oracle => "oracle",
            CoordinatorKind::CustodialEscrow => "custodial-escrow",
        }
    }

    /// Parse a wire string. An unrecognised kind **fails closed** (`None`).
    ///
    /// §18.8a.1: "An unknown `kind` MUST be treated as an undeclared coordinator (§2.4) — a client
    /// MUST NOT rely on a descriptor whose kind it does not recognise." Returning `None` rather
    /// than a near-match is what makes that MUST mechanical: `"compute"`, a kind that existed in an
    /// earlier draft and folded into [`CoordinatorKind::InfraService`], does not parse.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "gateway" => CoordinatorKind::Gateway,
            "relay" => CoordinatorKind::Relay,
            "media-relay" => CoordinatorKind::MediaRelay,
            "reachability-adapter" => CoordinatorKind::ReachabilityAdapter,
            "infra-service" => CoordinatorKind::InfraService,
            "indexer" => CoordinatorKind::Indexer,
            "labeler" => CoordinatorKind::Labeler,
            "matcher" => CoordinatorKind::Matcher,
            "arbiter" => CoordinatorKind::Arbiter,
            "oracle" => CoordinatorKind::Oracle,
            "custodial-escrow" => CoordinatorKind::CustodialEscrow,
            _ => return None,
        })
    }

    /// Parse a wire string into the crate's fail-closed error, for use at a decode boundary.
    pub(crate) fn require(s: &str) -> Result<Self, CoordinatorError> {
        CoordinatorKind::from_str(s).ok_or_else(|| CoordinatorError::UnknownRegistryValue {
            registry: "coordinator-kind",
            value: s.to_string(),
        })
    }

    /// Every kind, in CONTRACT §5 table order. **Eleven** — the count CONTRACT §5 fixes.
    pub const ALL: [CoordinatorKind; 11] = [
        CoordinatorKind::Gateway,
        CoordinatorKind::Relay,
        CoordinatorKind::MediaRelay,
        CoordinatorKind::ReachabilityAdapter,
        CoordinatorKind::InfraService,
        CoordinatorKind::Indexer,
        CoordinatorKind::Labeler,
        CoordinatorKind::Matcher,
        CoordinatorKind::Arbiter,
        CoordinatorKind::Oracle,
        CoordinatorKind::CustodialEscrow,
    ];

    /// Whether this kind is the disclosed **load-bearing exception** — the one kind that does not
    /// fade once hired (CONTRACT §1, THREAT-MODEL SEC-6/R-6). Every other kind is
    /// hired-not-depended-on: removing it degrades reach, never function.
    pub fn is_load_bearing_exception(self) -> bool {
        matches!(self, CoordinatorKind::CustodialEscrow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_round_trips_and_the_count_is_eleven() {
        // CONTRACT §5 fixes the count at eleven and forbids any other document tallying its own.
        // Asserting the count — not merely iterating — is what makes a silently-added twelfth kind
        // fail here rather than pass unnoticed.
        assert_eq!(CoordinatorKind::ALL.len(), 11);
        for k in CoordinatorKind::ALL {
            assert_eq!(CoordinatorKind::from_str(k.as_str()), Some(k), "{k:?}");
        }
        // No duplicate wire strings (a copy-paste in `as_str` would otherwise map two variants onto
        // one string and `from_str` would silently pick whichever arm came first).
        let mut seen: Vec<&str> = CoordinatorKind::ALL.iter().map(|k| k.as_str()).collect();
        seen.sort_unstable();
        let n = seen.len();
        seen.dedup();
        assert_eq!(seen.len(), n, "duplicate wire string in the kind registry");
    }

    #[test]
    fn the_folded_compute_kind_does_not_parse() {
        // The one kind §18.8a.1's stale parenthetical still lists. CONTRACT §5 folded it into
        // `infra-service`; a decoder that still accepts it keeps a retired concept alive in the
        // field, and — worse — two kinds both meaning "run code on your machine" leave a client
        // with nothing to disambiguate them by (CONTRACT §5 design note).
        assert_eq!(CoordinatorKind::from_str("compute"), None);
        assert!(CoordinatorKind::require("compute").is_err());
    }

    #[test]
    fn near_misses_fail_closed() {
        for bad in [
            "adapter",       // §26 adapters ARE `gateway`-kind; there is no `adapter` string
            "mediarelay",    // punctuation drift
            "media_relay",   // separator drift
            "Gateway",       // case
            "infra_service", // separator drift
            "infra",         // truncation
            "escrow",        // truncation of the load-bearing exception
            "",
        ] {
            assert_eq!(CoordinatorKind::from_str(bad), None, "{bad:?} must not parse");
        }
    }

    #[test]
    fn exactly_one_kind_is_load_bearing() {
        let load_bearing: Vec<_> = CoordinatorKind::ALL
            .into_iter()
            .filter(|k| k.is_load_bearing_exception())
            .collect();
        assert_eq!(load_bearing, vec![CoordinatorKind::CustodialEscrow]);
    }
}
