# SEARCH — discovery over public objects, following-graph first, indexer-optional

> **Status:** profile spec (KOTVA family). Normative once ratified. SEARCH is the **discovery**
> profile: how a reader *finds* the public objects other profiles publish — offers, products,
> reviews, feeds, media, artifacts — without a central index that owns the answer. It defines **no
> new wire bytes**: search reads DMTAP-PUB objects ([§22](../22-public-objects.md)) and subscribes to
> feeds ([§25](../25-pubsub.md)); its coordinator is an `indexer` under
> [`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md). This document owns the profile rules that
> make discovery **coordinator-optional** — a local search over the following-graph that works with
> no infrastructure at all, and an *opt-in* global index that adds reach, never authority.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are BCP 14 (RFC 2119 / RFC 8174).

---

## 1. What this profile is

SEARCH is the answer to *"how do I find things?"* computed **without an authority that owns the
index**. A centralised search engine bundles three things a user cannot separate: the crawl (what
exists), the rank (what surfaces first), and the query log (what you looked for) — and it owns all
three, so it can hide, promote, or sell any of them. SEARCH unbundles them and keeps discovery on the
same slide as the rest of KOTVA ([DIRECTION §6](../DIRECTION.md)): the primitives are identical at
every scale; only the **discovery anchor** moves.

- **The default is your following-graph.** Every object KOTVA discovers is a self-authenticating
  public object ([§22.5](../22-public-objects.md)); a reader already pulls the feeds it follows
  ([§25](../25-pubsub.md)). Searching *that* corpus is a **local** operation over objects you already
  hold — no crawler, no server, no coordinator. This is the floor the design collapses to, not a
  lesser mode ([DIRECTION §6](../DIRECTION.md)).
- **Global reach is an opt-in, hired `indexer`.** To find objects *beyond* your following-graph you
  hire an `indexer` coordinator ([CONTRACT §5](../coordinator/CONTRACT.md)) — one of several that
  compete, that you can fire, and that you can self-host. An indexer adds **reach**, never authority
  (SRCH-2). Whether one is TEE-attested (DeSearch-style, §5) or a plain crawler is a declared,
  surfaced property.
- **There is no ranking token, and no paid network rank.** Rank is not purchasable through any
  protocol mechanism — there is no protocol token to buy it with ([DIRECTION §5](../DIRECTION.md)).
  Ranking is *index-local, disclosed policy* (SRCH-4), exactly as reputation is (REP-7): two indexers
  reaching different orders is the design, not a defect.

SEARCH is **cross-profile by construction.** Offers (TRACT), providers (WRAP), reviews, artifacts
(§24), and feeds are all just `IK`-authored, content-addressed public objects on one substrate. One
index spans them all; the profile invents no per-vertical registry.

---

## 2. Primitives, coordinators, and bindings it composes

SEARCH is a **reader**, not a foundation: it discovers what other primitives already sign, and adds
one coordinator kind.

| Composed with | Role in SEARCH |
|---|---|
| **OFFER** ([`OFFER.md`](../primitives/OFFER.md)) | The primary indexed object. OFFER's **OFR-7** — *indexes are derived, never authoritative* — is the rule SEARCH is built around; on any disagreement between an index and a seller's signed feed, the feed governs. |
| **REPUTATION** ([`REPUTATION.md`](../primitives/REPUTATION.md)) | A ranking *input*, not a stored number. An indexer MAY weight results by reputation computed over public feeds, but **REP-3** forbids a published score, so ranking is derived and disclosed (SRCH-4), never a figure baked into a result. |
| **ATTEST** ([`ATTEST.md`](../primitives/ATTEST.md)) | The anchor an index MAY use to discount unanchored authors (anti-spam, §7) — personhood (World ID / Human Passport) or staked value, never a token. |
| **`indexer`** coordinator ([CONTRACT §5](../coordinator/CONTRACT.md)) | The one coordinator SEARCH adds: a global-view crawl + rank service, `blind`/`attested` (TEE) preferred. It **authorises, never classifies** (CONTRACT §4): it ranks and filters its *own* view; it cannot delist an author from the network. |
| **`labeler`** coordinator ([CONTRACT §5](../coordinator/CONTRACT.md)) | OPTIONAL. Moderation of *results* is an opt-in labeler the reader subscribes to and can leave — never a classification gate the indexer applies for everyone (§7). |

**Bindings adopted** ([DIRECTION §3](../DIRECTION.md), [`bindings/README.md`](../bindings/README.md)):

- **TEE search — DeSearch** ([USENIX OSDI '21](../bindings/README.md)). The reference for an
  `attested` indexer: a global crawl+rank run inside a TEE so results are **verifiably the published
  ranking over the operator's corpus** and queries are shielded from the operator. Adopted as the
  `attested` assurance level ([CONTRACT §3.3](../coordinator/CONTRACT.md)), disclosed as
  chip-vendor-trust, **never** sold as trustless.
- **Federated / local index — YaCy-class.** The precedent for the following-graph-first local index
  and for peer-to-peer index sharing with no central crawler. KOTVA specifies no index format; a
  local index is rebuildable private state over §22/§25 objects.
- **Reputation-weighted ranking — OpenRank** (EigenTrust, TEE-verified). The bound compute an
  indexer MAY use to order results by reputation without publishing a score (REP-4/REP-7).
- **Feed reach — libp2p + DMTAP-PUBSUB** ([§25](../25-pubsub.md)). Following-graph subscription and
  push hints; the substrate that makes the local corpus exist without a crawler.

No binding introduces a protocol token, a global score, or a surveillance signal — forbidden by
[DIRECTION §5](../DIRECTION.md) and [`bindings/README.md`](../bindings/README.md).

---

## 3. MOTE kinds / PUB objects it uses

SEARCH defines **no new wire object** (like OFFER and REPUTATION, it is a shape and a set of rules
over existing bytes). It **reads**:

| Object | Kind | Role in search |
|---|---|---|
| `PubAnnounce` | `0x40` ([§22.3](../22-public-objects.md)) | The unit of discovery — an offer, review, artifact, or record announcement. Indexed by `(pub, content-address)`; verified per-hit (§8). |
| `PubManifest` | ([§22.2](../22-public-objects.md)) | The public blob a `PubAnnounce` points at (product data, media, document body) — the full-text/content an index derives terms from. |
| `FeedHead` / topic feeds | ([§22.4](../22-public-objects.md), [§25.3](../25-pubsub.md)) | The author-feed spine: order, anti-rollback, and the `topic` dimension (`FeedHead` key `64`) an index uses to scope a crawl. |
| `Subscription` | ([§25.4](../25-pubsub.md)) | How a reader assembles its **following-graph** corpus — the local, coordinator-free search domain. |
| `identity` announcement | `0x09` ([§1](../01-identity.md), [§2](../02-mote.md)) | Resolves a hit's author `IK` and its `DeviceCert` chain, so a result is attributable, not just a string. |

The **index itself is derived, rebuildable, non-authoritative state** — never a wire object with
protocol standing (SRCH-2, OFR-7). An indexer's *descriptor* is an ordinary coordinator descriptor
([CONTRACT §2.1](../coordinator/CONTRACT.md)) carrying its kind, corpus scope, ranking policy, and
visibility class. An `attested` indexer MAY additionally publish a **signed corpus-root commitment**
and per-query result attestations (DeSearch-style) so completeness and non-tampering are checkable;
these are OPTIONAL, content-addressed, and themselves rebuildable — they add no authority the signed
feeds do not already carry.

A **query** is not a wire object: it is client↔indexer traffic against the indexer's own API,
outside the substrate object model. Its confidentiality is a declared content-visibility property
(§8), not a signed object.

---

## 4. Normative profile rules

- **SRCH-1 — following-graph first.** A conformant search client MUST be able to search its **local
  corpus** — the feeds it already follows/pulls ([§25](../25-pubsub.md)) — with **zero coordinator**.
  A global `indexer` MUST be strictly opt-in; a client MUST NOT require one to return local results,
  and MUST NOT hard-wire a default index a user cannot remove.
- **SRCH-2 — the index is derived, never authoritative.** Any index, ranking, or category view over
  public objects is rebuildable and MUST NOT be treated as an authority (inherits **OFR-7**). On any
  disagreement between an index and an author's signed feed, resolution MUST prefer the feed
  ([§22.4.3](../22-public-objects.md)). No index can revoke a content-address or delist an author
  from the network; it may only decline to list in its own view.
- **SRCH-3 — competing indexers, zero lock-in.** Switching or dropping an indexer MUST be a
  configuration change with zero data migration and zero identity change
  ([CONTRACT §2.2](../coordinator/CONTRACT.md)). A client SHOULD support **multiple concurrent
  indexers** and MUST NOT bind a user's history, follows, or identity to any one.
- **SRCH-4 — ranking policy is local, and disclosed.** An indexer that ranks MUST publish its
  weighting policy in its signed descriptor ([CONTRACT §2.1](../coordinator/CONTRACT.md)) — which
  reputation, anchoring, freshness, and web-of-trust measures produced the order — and a reader MUST
  be able to substitute its own local ranking (REP-7). Ranking is opinion, disclosed as such.
- **SRCH-5 — no ranking token; paid rank is disclosed operator policy, never protocol.** Rank MUST
  NOT be purchasable through any protocol token; none exists and none will be added
  ([DIRECTION §5](../DIRECTION.md)). An indexer MAY offer paid placement as *operator policy*, but it
  MUST be disclosed in the descriptor and **visibly labelled distinct from organic results** at
  display; it is escapable by switching indexers or self-hosting, and it is never a network-wide
  authority over what surfaces.
- **SRCH-6 — authorise, never classify.** An indexer MUST NOT act as a network content gate. It MAY
  rank and filter *its own* view; it MUST NOT drop, quarantine, or annotate on a content basis as if
  removing an object from the network ([CONTRACT §4](../coordinator/CONTRACT.md)). Result moderation
  is an **opt-in labeler** the reader chooses (§7), not a filter the indexer imposes on everyone.
- **SRCH-7 — network-completeness is unverifiable at every level; corpus-completeness is a
  narrower, checkable claim.** An indexer SHOULD run `attested` (TEE, DeSearch-style) so result
  integrity, query privacy, and completeness **over its own reachable corpus** are verifiable. No
  assurance level — `attested` included — can prove that corpus is the whole network: no global
  corpus exists to check against, so network-completeness is unfalsifiable regardless of level
  (§8). A client MUST NOT present any index's result set as demonstrably complete against the
  network or "everything"; it MAY present an `attested` index's result set as complete against its
  own reachable corpus. A `declared`-level indexer's corpus-completeness and neutrality are
  unverifiable even in that narrower sense, and a client MUST surface that: it MUST NOT present a
  `declared` index's result set as demonstrably complete or unfiltered
  ([CONTRACT §3.4](../coordinator/CONTRACT.md)).
- **SRCH-8 — no new bytes.** SEARCH MUST NOT define a new MOTE kind or reassign an existing wire key.
  It reads §22/§25 objects; any index checkpoint or result attestation it emits MUST be a coordinator
  descriptor or a signed public object, rebuildable and non-authoritative (SRCH-2).
- **SRCH-9 — query visibility declared.** An indexer MUST declare, in its descriptor, its
  query-channel visibility **class** — `terminating` if the operator can read queries in the
  clear, `blind` if they are shielded — and its **assurance level** (§8;
  [CONTRACT §3.3](../coordinator/CONTRACT.md)). A client MUST surface both. Advertising
  query-blindness while operating query-readable is non-conformant misrepresentation
  ([CONTRACT §2.4/§3](../coordinator/CONTRACT.md)).
- **SRCH-10 — cross-profile, no per-vertical registry.** Search MUST span every profile by keying on
  `IK` and content-address alone ([§22](../22-public-objects.md)); it MUST NOT require a
  profile-scoped registry or a per-vertical index authority. A profile MUST NOT redefine discovery;
  it publishes public objects and inherits this profile.

---

## 5. Scale-invariance — following-graph ⇆ competing indexers

The objects and rules are identical at every scale; only the **discovery anchor** slides
([DIRECTION §6](../DIRECTION.md)):

| Function | Small / mesh (offline, no coordinator) | Global (opt-in, swappable coordinator) |
|---|---|---|
| **Corpus** | feeds you already follow / pull ([§25](../25-pubsub.md)) | an `indexer`'s crawl over the public swarm |
| **Rank** | local heuristic over web-of-trust + freshness, on your device | indexer ranking (OpenRank/TEE), disclosed policy (SRCH-4) |
| **Integrity** | you verify every hit yourself (§8) | `attested` TEE result attestation, or unverified `declared` (SRCH-7) |
| **Authority** | you | still you — the index is derived, fireable (SRCH-2/SRCH-3) |

The mesh form is the **fallback the design collapses to**, not a degraded mode: remove connectivity
and search is exactly the transitive following-graph you can reach, ranked locally. Add connectivity
and a global indexer becomes *available* for reach over strangers — never *required* to find the
people and objects you already follow. The same signed object verifies identically whether a hit
arrived over the mesh, HTTPS, or an SD card ([OFFER §6](../primitives/OFFER.md)).

---

## 6. Offline / apocalypse behaviour + reconcile

Per [`substrate/OFFLINE.md`](../substrate/OFFLINE.md), each search action classifies into exactly one
degradation grade, with **no silent degradation** and **no fabricated completion**:

| Action | Grade | Offline behaviour |
|---|---|---|
| Search the local corpus (following-graph) | **`full`** | Local-first: the corpus is objects you already hold; the index is local rebuildable state; results verify from the objects alone. |
| Follow / index a peer's feed you can still reach (mesh) | **`local-trust`** | Discovery over the mesh web-of-trust — reachable feeds only, ranked locally. |
| Global discovery via an `indexer` | **`local-trust` → `blocked`** | Degrades to the local index; with no network and no reachable index, discovery beyond the local corpus is **`blocked`** and MUST say so — an empty or partial result MUST NOT be presented as the whole network's answer. |

**Reconcile on reconnect** is DMTAP-PUB feed catch-up ([OFFLINE §4](../substrate/OFFLINE.md),
[§22.4.2](../22-public-objects.md)): a reader re-pulls feed heads, applies strict monotonic-`seq`
anti-rollback, walks the `prev` chain, and re-derives its local index — idempotent and
order-independent. The index is derived state (SRCH-2), so a stale index is never a correctness
failure, only a staleness one; it rebuilds from the feeds with no coordinator refereeing. Feed
**equivocation** (two heads at one `seq`) surfaces as transferable `ERR_PUB_FEED_CHAIN_BROKEN`
evidence, never swallowed by a clean merge (R-REC-2). Because search only *reads* and *derives*, it
has no cross-replica invariant to violate offline — the hard offline cases (holds, money) belong to
RESERVE and [OFFLINE §5](../substrate/OFFLINE.md), not here.

---

## 7. Security + declared content-visibility

SEARCH inherits [`THREAT-MODEL.md`](../THREAT-MODEL.md) (SEC-1…SEC-9). The invariants that bite here:

- **SEC-2 (intrinsic authenticity).** A result is a *pointer*, not a truth. A client MUST verify each
  hit — signature, `DeviceCert` chain to `IK`, content-address recomputation
  ([§22.5.1](../22-public-objects.md)) — **before** presenting or trusting it; an indexer is a
  convenience, never a trust root. A malicious index can **withhold, re-order, or stall** (SRCH-7,
  detectable at `attested` level; undetectable at `declared`) but can **never forge** a signed object
  or make an unsigned string appear authored.
- **SEC-6 (authorise, never classify).** An `indexer` is hired, swappable, self-hostable, never
  load-bearing, and it authorises from identity + rate only — it does not classify content as a
  network gate (SRCH-6). An index that "finishes" and centralizes by out-classifying rivals is the
  failure mode CONTRACT §4 exists to forbid.
- **SEC-7 (Sybil priced and localized, not solved).** Spam and index-poisoning resistance is
  **index-local** — priced at the indexer (postage / rate-limit / discount unanchored authors via
  ATTEST, §2) — never a network authority. There is no global search registry to poison; a poisoned
  index is one index, swappable (SRCH-2/SRCH-3). A profile MUST NOT describe search-spam resistance
  as solved.
- **SEC-9 (query metadata).** The query is the sensitive stream — it reveals interest, not content
  (the objects are public). It MUST be handled per SRCH-9.

**Declared content-visibility (normative).** Search is the unusual case where the *indexed objects
carry no secret* — they are public by design ([§22.1](../22-public-objects.md)), so there is no
payload confidentiality for the indexer to breach. The declared visibility therefore concerns the
**query channel** and **result integrity**, not the corpus:

| Aspect | What the indexer can see / do | Declared as |
|---|---|---|
| **Indexed corpus** | Public objects, by design — reading them is not a trust breach. | `public` (no secret exists) |
| **Query channel** | *What you search for.* At class `terminating`, the operator reads queries in the clear (`declared` assurance — a promise, not proven). At class `blind`, an `attested` (TEE, DeSearch) indexer shields queries from the operator (hardware-trust, provable). | class `terminating` (`declared`) or class `blind` (`attested`) — MUST be surfaced (SRCH-9) |
| **Result integrity** | *Whether the result set was silently filtered / re-ranked.* Verifiable only at `attested`; a promise at `declared`. | assurance level per [CONTRACT §3.3](../coordinator/CONTRACT.md) |

A client MUST NOT present a `declared`-level query-blindness or completeness claim as if it were
verified ([CONTRACT §3.4](../coordinator/CONTRACT.md)). All nine MUSTs hold offline
([THREAT-MODEL §4](../THREAT-MODEL.md)): local search needs no coordinator, and a property that
required one would be a conformance violation.

---

## 8. Honest residual

Discovery is where the **editorial-governance** ceiling ([DIRECTION §8.4](../DIRECTION.md)) bites
hardest, and this profile **discloses it rather than solving it**:

- **Complete indexing of public objects amplifies targeted harassment, and this profile has no
  answer.** Mastodon deliberately restricted full-text search to a user's own posts and mentions
  precisely because complete searchability is the mechanism by which pile-ons find their targets;
  that limitation was a product decision its operators defended for years, not an implementation
  gap. KOTVA treats corpus completeness as a virtue — an `indexer`'s "corpus is public plaintext
  (nothing to be blind about)" ([CONTRACT §5](../coordinator/CONTRACT.md)) — and publishes no
  discoverability preference on a PUB object, so nothing in this profile lets an author say "public,
  but not indexable". Note that the coordinator contract makes the obvious fix harder than it looks:
  indexers are uncoordinated and swappable, so any such preference would be an author-declared
  request that a conformant indexer MAY honour and a hostile one simply ignores — closer to
  `endorsed-only` (§24.11) than to a structural control, and unenforceable by construction against
  the very actor it needs to bind. **This is a disclosed gap, not a solved problem**; whether the
  substrate should carry such a field at all is an open design question recorded for the maintainer,
  and is deliberately *not* decided here.
- **Discovery re-centralizes first, and SRCH-6 does not stop it.** A content-addressed substrate
  offers no global index, so whichever indexer becomes economically dominant becomes a *de-facto*
  content-policy gatekeeper — able to bury (never delete) an author across most readers' views —
  *regardless* of what SRCH-2/SRCH-6 permit. Multiple competing indexers with verifiable
  completeness is the candidate answer and has **no deployed precedent**
  ([OFFER §9](../primitives/OFFER.md), [TRACT §2.6/§21](tract/02-catalogue.md)). This is the weakest
  claim in the profile, named not defended.
- **"Complete" is unfalsifiable on a permissionless substrate.** Even an `attested` TEE index proves
  only *"I ran the published ranking over my corpus"* — never that its corpus is the whole network,
  because no global corpus exists to compare against (there is no registry, by design). Completeness
  against the reachable swarm is the strongest honest claim; completeness against "everything" is
  not achievable and MUST NOT be asserted (SRCH-7).
- **Query privacy against a `declared` operator is unattainable; the TEE only moves the trust.**
  DeSearch-style `attested` indexing shields queries and proves result integrity, but it substitutes
  chip-vendor trust for operator trust and carries the TEE side-channel history
  ([bindings](../bindings/README.md), `adopt (disclosed)`). DeSearch is research-grade (USENIX '21),
  not a mainstream deployment; the maturity claim is a 2026-07 snapshot.
- **Ranking has no authoritative answer, and that is the cost of sovereignty.** Removing the single
  authoritative index buys freedom from a gatekeeper at a real usability cost: indexes disagree, and
  there is no canonical "top result." That divergence is the intended outcome (SRCH-4, REP-4), not a
  defect to engineer away.
- **Index-spam is priced, not eliminated.** SEC-7 confines poisoning to one index; it does not raise
  the global anti-Sybil floor, which stays imperfect ([DIRECTION §8.1](../DIRECTION.md)). Local scale
  dissolves it into web-of-trust; global scale prices it, and pricing is bounded by imperfect
  personhood.

Every residual traces to a root ceiling ([DIRECTION §8](../DIRECTION.md)): the gatekeeper and
completeness residuals are **editorial governance**; the spam floor is **global anti-Sybil**. None is
a bug in SEARCH; each is a consequence of not being a single surveilling company, and is disclosed
rather than solved. Search/TEE research returned a small verified base (DeSearch, YaCy, OpenRank);
the reasoning here is checked for internal consistency against the anchor docs, not offered as a
demonstrated result, and all maturity claims are a 2026-07 snapshot
([docs/research/README §6](../docs/research/README.md)).
