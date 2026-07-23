# SPEC — the KOTVA family, as one tree

> **Status:** master index (non-normative navigation). This document introduces no bytes and
> governs nothing; it is the map. Every capability, primitive, coordinator kind, binding, and
> profile below has a normative home, linked inline — **that home governs.** Where this index and
> a normative document disagree, the normative document wins. The rules the whole tree is built to
> are [`DIRECTION.md`](DIRECTION.md); the reasoning under them is [`docs/research/`](docs/research/README.md).

The key words **MUST**, **MUST NOT**, **SHOULD**, **MAY** are BCP 14 (RFC 2119 / RFC 8174) wherever
this document restates a normative rule from its home; here they are pointers, not fresh requirements.

---

## The one rule

> **Decentralize the substrate and the exit. Every unavoidable coordinator is _accountable,
> swappable, and self-hostable_, and _never load-bearing_. Coordinators add reach; they never gate
> function.**

Everything else is a consequence ([`DIRECTION § 0`](DIRECTION.md)). It is DMTAP's legacy-mail-gateway
model ([`§7`](07-gateway.md) — one accountable operator class, swappable with a DNS change, with a
self-host backstop) generalized from mail to the whole system.

---

## The layered architecture

Six layers, bottom to top. Each depends only on the layer beneath and a stable seam sideways; a hard
problem is never hard-wired — it is a swappable **binding** or a fenced **coordinator**
([`DIRECTION § 9`](DIRECTION.md)).

```
        ┌──────────────────────────────────────────────────────────────────┐
        │  ⑥ PROFILES   mail · commerce · work · social · media · calls …    │  thin
        ├──────────────────────────────────────────────────────────────────┤
        │  ⑤ BINDINGS   thin maps onto adopted standards (swap the filling)  │  seam →─┐
        ├──────────────────────────────────────────────────────────────────┤        │
        │  ④ COORDINATOR CONTRACT   safe centralization, made checkable      │  ←──────┘ (to the side)
        ├──────────────────────────────────────────────────────────────────┤
        │  ③ PRIMITIVES   OFFER · MATCH/RESERVE · REPUTATION · ESCROW …      │  compose the space
        ├──────────────────────────────────────────────────────────────────┤
        │  ② SUBSTRATE (the narrow waist)   ID · MOTE · TRANS · PUB · SYNC …  │  6 capabilities
        ├──────────────────────────────────────────────────────────────────┤
        │  ① ADOPT   Ed25519 · HPKE · CBOR                                    │  proven, not reinvented
        └──────────────────────────────────────────────────────────────────┘
```

1. **ADOPT** — the foundational wire primitives everything above rides on, proven rather than
   re-derived ([`DIRECTION § 3`](DIRECTION.md)): Ed25519, HPKE, CBOR. (MLS, libp2p, WebRTC,
   SFrame are the higher-level, swappable **bindings** — see layer ⑤, indexed at
   [`bindings/README.md`](bindings/README.md).)
2. **SUBSTRATE** — the six-capability waist; SYNC and MATCH's assignment vocabulary are the only
   genuinely new normative ground: [`substrate/README.md`](substrate/README.md).
3. **PRIMITIVES** — the small set every service rearranges: [`primitives/`](primitives/), design
   brief [`docs/research/PRIMITIVES.md`](docs/research/PRIMITIVES.md).
4. **COORDINATOR CONTRACT** — the keystone that makes "some centralization, done safely" a checkable
   property, not a hope: [`coordinator/CONTRACT.md`](coordinator/CONTRACT.md).
5. **BINDINGS** — one thin mapping document per adopted standard; swap when the frontier improves:
   [`bindings/README.md`](bindings/README.md).
6. **PROFILES** — mail, commerce, work, social, media, calling, discovery, reachability — each a thin
   composition, no new cryptography: [`profiles/`](profiles/) + the numbered DMTAP-mail sections.

---

## ② Substrate — the narrow waist

Six capabilities. A product **MAY** adopt any subset; if it implements a capability's *function* it
**MUST** speak that capability's *bytes* ([`substrate/README § 3`](substrate/README.md)). Each has a
normative byte-home in the numbered spec and a substrate document that re-presents it for non-mail use.

| # | Capability | What it is | Byte-home | Substrate doc |
|---|---|---|---|---|
| ① | **Identity** | A keypair *is* the identity; names (DNS / chain / 8-word key-name floor) are swappable pointers. | [`01-identity.md`](01-identity.md), [`03-naming.md`](03-naming.md) | [`substrate/IDENTITY.md`](substrate/IDENTITY.md) |
| ② | **MOTE** | The universal sealed object: signed, encrypted, content-addressed. Mail, chat, offers, credentials are all MOTEs. | [`02-mote.md`](02-mote.md), [`05-messaging.md`](05-messaging.md) | (sealed path; see [`02-mote.md`](02-mote.md)) |
| ③ | **Transport** | Reach anyone by key — online, offline, or over a mesh; store-and-forward at the edge. | [`04-transport.md`](04-transport.md) | [`substrate/ROLES.md`](substrate/ROLES.md) |
| ④ | **PUB** | Signed public objects + append-only author feeds — authenticity without confidentiality. | [`22-public-objects.md`](22-public-objects.md), [`25-pubsub.md`](25-pubsub.md) | [`substrate/FEEDS.md`](substrate/FEEDS.md) |
| ⑤ | **SYNC** | Multi-author signed CRDT — shared mutable state, no coordinator. *Genuinely new normative ground (with MATCH's assignment vocabulary).* | [`substrate/SYNC.md`](substrate/SYNC.md) | [`substrate/SYNC.md`](substrate/SYNC.md) |
| ⑥ | **Roles & Wake** | Open, key-addressed infrastructure roles any node may serve; content-free push to offline nodes. | [`04-transport.md`](04-transport.md) §4.2/§4.9, [`14-scaling.md`](14-scaling.md) | [`substrate/ROLES.md`](substrate/ROLES.md) |

Cross-cutting substrate docs: [`substrate/OFFLINE.md`](substrate/OFFLINE.md) (the apocalypse-proof
degradation vocabulary), [`substrate/ADOPTION.md`](substrate/ADOPTION.md) (per-product status matrix,
informative), [`substrate/BINDINGS.md`](substrate/BINDINGS.md) (one-core/many-surfaces engineering
plan, informative). The waist's own security floor is [`THREAT-MODEL.md`](THREAT-MODEL.md).

---

## ③ Primitives — build these once, build the world

Every real service is the same set rearranged ([`DIRECTION § 2`](DIRECTION.md)):
`OFFER · MATCH/RESERVE · REPUTATION · ESCROW · ORACLE · DISPUTE · PAY`. All ride existing substrate
objects (MOTE/PUB/SYNC) carrying a profile-defined shape; only MATCH adds a new schema + assignment-rule
vocabulary of its own.

| Primitive | What it is | Binds to | Coordinator | Spec |
|---|---|---|---|---|
| **OFFER** | Signed public listing — supply, not a registry (Product ≠ Offer split, four axes). | DMTAP-PUB (§22) | `indexer` (discovery only) | [`primitives/OFFER.md`](primitives/OFFER.md) |
| **MATCH** | The one matching engine; only the **assignment rule** (nearest / highest-bid / best-fit) slides. | existing MOTE/SYNC objects + a new schema + assignment-rule vocabulary (no new wire envelope) | `matcher` (`terminating`/`attested`) | [`primitives/MATCH.md`](primitives/MATCH.md) |
| **RESERVE** | Single-owner bounded counter — double-booking is structurally impossible. **No coordinator, ever.** | SYNC single-writer object | **none** | [`primitives/RESERVE.md`](primitives/RESERVE.md) |
| **REPUTATION** | Portable, wash-resistant trust with **no global score** — locally measured or OpenRank-computed. | OpenRank / EAS-VC | `indexer` (derived, rebuildable) | [`primitives/REPUTATION.md`](primitives/REPUTATION.md) |
| **ESCROW** | Hold value, move it only on a stated condition. Attestations on the wire, funds on the rail. | HTLC / stablecoin; Kleros dispute | `arbiter` / escrow operator | [`primitives/ESCROW.md`](primitives/ESCROW.md) |
| **ATTEST** | A signed claim — *"`I` said `S` about `X`"* — the substrate under reputation, credentials, KYC. | EAS + W3C VC; World ID | `oracle` (physical-fact only) | [`primitives/ATTEST.md`](primitives/ATTEST.md) |

`ORACLE` (physical-event attestation), `DISPUTE` (staked arbitration), and `PAY` (stablecoin
settlement) are **coordinator/binding** roles rather than owned primitive docs: ORACLE ⊂ ATTEST +
the `oracle` kind; DISPUTE = the `arbiter` kind + Kleros binding; PAY = the x402/stablecoin binding.
The design brief for all six is [`docs/research/PRIMITIVES.md`](docs/research/PRIMITIVES.md).

---

## ④ Coordinator kinds — where centralization is allowed, and fenced

Every coordinator is one instance of [`coordinator/CONTRACT.md`](coordinator/CONTRACT.md): it is
**accountable** (§2.1), **swappable** (§2.2), **self-hostable** (§2.3), **declares one
content-visibility class** (§2.4/§3), and **authorizes but never classifies** (§4). It mints no token
(§6). `gateway` ([`§7`](07-gateway.md)) and the legacy `adapter`s ([`§26`](26-legacy-adapters.md)) are
the first fully-worked instances; every kind inherits the four clauses unchanged.

Canonical, exhaustive kind list (11 kinds, incl. `compute` and the load-bearing
`custodial-escrow` exception): [`coordinator/CONTRACT § 5`](coordinator/CONTRACT.md) — that
table governs; it is not reproduced here to avoid drift.

---

## ⑤ Bindings — what KOTVA adopts instead of reinventing

Each need below is a **slot**; swap the filling as the frontier improves, and the substrate and
profiles never change. Canonical list, with maturity notes and 2026 sources:
[`bindings/README.md`](bindings/README.md) — that document governs; this is a summary, not a copy.

Identity recovery · Attestation (**ATTEST is KOTVA's own primitive; only its claim body binds
EAS / W3C Verifiable Credentials** — no new credential format, [`primitives/ATTEST.md`](primitives/ATTEST.md))
· Reputation · Personhood · Payments (no protocol token — none exists, none will) · Storage ·
Dispute · Media transport · Mesh / messaging crypto · Verifiable coordination.

What KOTVA never binds: a protocol token, a global reputation *score*, a search-ranking token,
surveillance advertising ([`bindings/README.md`](bindings/README.md), [`DIRECTION § 5`](DIRECTION.md)).

---

## ⑥ Profiles — thin compositions over the waist

A profile invents no cryptography and no wire capability; it owns a schema over `PubAnnounce.meta` /
MOTE bodies plus the client rules ([`DIRECTION § 1`](DIRECTION.md)).

### DMTAP-mail — the flagship profile (numbered §00–§27)

The mail + messaging + gateway profile: the first and reference profile, standing on all six waist
capabilities and adding the sealed metadata-private path and legacy bridge on top.
*(To be relocated under `profiles/dmtap-mail/` in a later restructure.)*

| § | Section | § | Section |
|---|---|---|---|
| [00](00-overview.md) | Overview & Architecture | [14](14-scaling.md) | Scaling & Deployment |
| [01](01-identity.md) | Identity Lifecycle | [15](15-references.md) | Normative & Informative References |
| [02](02-mote.md) | The MOTE Object | [16](16-parameters.md) | Numeric Parameters (v0) |
| [03](03-naming.md) | Naming & Directory (name → key) | [17](17-parity.md) | Feature-Parity Audit |
| [04](04-transport.md) | Transport: Mesh, Mixnet, Delivery | [18](18-wire-format.md) | Appendix A: Wire Format (CDDL) |
| [05](05-messaging.md) | Messaging & Files (unified substrate) | [19](19-operations.md) | Appendix B: Protocol Operations |
| [06](06-privacy.md) | Privacy & Threat Model | [20](20-state-machines.md) | Appendix C: State Machines |
| [07](07-gateway.md) | The Legacy Gateway Role (optional) | [21](21-errors-iana.md) | Appendix D: Errors & IANA |
| [08](08-clients.md) | Client Access | [22](22-public-objects.md) | DMTAP-PUB: Public Objects |
| [09](09-anti-abuse.md) | Anti-Abuse & Postage | [23](23-cad-artifact-profile.md) | *(folded into §24)* |
| [10](10-conformance.md) | Versioning, Conformance, Governance | [24](24-video-profile.md) | Published-Artifact Profile (media + engineering) |
| [11](11-grounding-and-references.md) | Grounding & References | [25](25-pubsub.md) | DMTAP-PUBSUB: Feed Subscriptions |
| [12](12-operators.md) | Operators, the Seam & User Protection | [26](26-legacy-adapters.md) | Legacy Adapters — beyond SMTP |
| [13](13-identity-auth.md) | DMTAP-Auth — Decentralized Login | [27](27-realtime-media.md) | Real-Time Media Profile |

### TRACT — the commerce profile ([`profiles/tract/`](profiles/tract/README.md))

Goods, services, rentals, subscriptions between sovereign identities — no marketplace operator, no
registrar, no token. `OFFER · MATCH · RESERVE · REPUTATION · ESCROW · DISPUTE · PAY`.
Sections: [00 overview](profiles/tract/00-overview.md) · [01 actors](profiles/tract/01-actors.md) ·
[02 catalogue](profiles/tract/02-catalogue.md) · [03 availability](profiles/tract/03-availability.md) ·
[04 fulfilment](profiles/tract/04-fulfilment.md) · [05 consideration](profiles/tract/05-consideration.md) ·
[06 cart](profiles/tract/06-cart.md) · [07 order](profiles/tract/07-order.md) ·
[08 delivery](profiles/tract/08-delivery.md) · [09 settlement](profiles/tract/09-settlement.md) ·
[10 trust](profiles/tract/10-trust.md) · [11 jurisdiction](profiles/tract/11-jurisdiction.md) ·
[12 gateway](profiles/tract/12-gateway.md) · [13 analytics](profiles/tract/13-analytics.md) ·
[14 anti-abuse](profiles/tract/14-anti-abuse.md) · [15 conformance](profiles/tract/15-conformance.md) ·
[16 wire-format](profiles/tract/16-wire-format.md) · [17 errors](profiles/tract/17-errors-iana.md) ·
[18 state-machines](profiles/tract/18-state-machines.md) · [19 parameters](profiles/tract/19-parameters.md) ·
[20 references](profiles/tract/20-references.md) · [21 grounding](profiles/tract/21-grounding.md) ·
[22 erasure](profiles/tract/22-erasure.md).

### WRAP — the work profile ([`profiles/wrap/`](profiles/wrap/README.md))

Issuing, offering, assigning, tracking and proving **work** — couriers, plumbers, field service,
mutual aid — no central operator, no cut. `OFFER · MATCH · REPUTATION · ORACLE · DISPUTE` (settlement
and escrow are out of scope for WRAP — no escrow, no currency, no blockchain — and compose via TRACT).
Sections: [00 overview](profiles/wrap/00-overview.md) · [01 identity](profiles/wrap/01-identity.md) ·
[02 objects](profiles/wrap/02-objects.md) · [03 wire-format](profiles/wrap/03-wire-format.md) ·
[04 signing](profiles/wrap/04-signing.md) · [05 lifecycle](profiles/wrap/05-lifecycle.md) ·
[06 merge](profiles/wrap/06-merge.md) · [07 pools](profiles/wrap/07-pools.md) ·
[08 trust](profiles/wrap/08-trust.md) · [09 fulfilment](profiles/wrap/09-fulfilment.md) ·
[10 transport](profiles/wrap/10-transport.md) · [11 profiles](profiles/wrap/11-profiles.md) ·
[12 errors](profiles/wrap/12-errors.md) · [13 security](profiles/wrap/13-security.md) ·
[14 conformance](profiles/wrap/14-conformance.md) · [15 references](profiles/wrap/15-references.md).

### The single-file profiles

| Profile | What it is | Primitives / plane | Spec |
|---|---|---|---|
| **SOCIAL** | Microblogging & public feeds — no timeline operator, no ranking authority. | PUB · SYNC · REPUTATION; `labeler` + `indexer` | [`profiles/social.md`](profiles/social.md) |
| **REACH** | Public HTTPS name for any box service; the adapter stays content-blind (SNI-passthrough). | Roles; `reachability-adapter` | [`profiles/reachability.md`](profiles/reachability.md) |
| **SEARCH** | Discovery over public objects — following-graph first, indexer-optional. | PUB · PUBSUB; `indexer` | [`profiles/search.md`](profiles/search.md) |
| **MEDIA** | The Evermesh profile — video & music; box-as-origin, CDN as swappable cache. | PUB · storage · media-relay; `indexer` | [`profiles/media.md`](profiles/media.md) |
| **RTC** | Real-time voice / video / calling — the parallel media plane (family-level view of §27). | ID · MLS→SFrame; `media-relay` | [`profiles/rtc.md`](profiles/rtc.md) |

---

## COVERAGE — products → primitives

The claim the whole tree exists to make checkable: the six-capability waist + the primitives + the
one contract cover the product space, and the covered rows collapse into a handful of mechanisms. Full
audit (22 shapes, visibility per coordinator, minimal fix per gap):
[`docs/research/coverage-matrix.md`](docs/research/coverage-matrix.md). Compressed:

| Product | Waist | Primitives | Coordinators | Profile | Status |
|---|---|---|---|---|---|
| **Mail / chat** | MOTE · TRANS · SYNC · WAKE | — (postage opt) | gateway · relay · mailbox | [DMTAP §5](05-messaging.md) | ✔ |
| **Calls / live video** | ID · MLS→SFrame · TRANS | — | media-relay + SFU | [RTC](profiles/rtc.md) / [§27](27-realtime-media.md) | ✔ |
| **Social feeds** | PUB · SYNC | REPUTATION | indexer · labeler | [SOCIAL](profiles/social.md) | ▲ editorial |
| **Media (video/music)** | PUB · storage · TRANS | PAY | indexer · media-relay · storage | [MEDIA](profiles/media.md) / [§24](24-video-profile.md) | ✔ |
| **Ride / delivery / freelance** | ID · TRANS · SYNC | OFFER · MATCH · REP · ORC · DIS · (ESC · PAY via TRACT) | matcher · oracle · arbiter | [WRAP](profiles/wrap/README.md) + [TRACT](profiles/tract/README.md) | ▲ oracle |
| **Bookings** | PUB · SYNC · MOTE | OFFER · **RESERVE** · REP · ESC · PAY | none (single-writer) | [TRACT §3](profiles/tract/03-availability.md) | ✔ |
| **Auctions** | PUB · SYNC | OFFER · MATCH(highest-bid) · REP · ESC · DIS | matcher · arbiter | [TRACT](profiles/tract/README.md) | ✔ (sealed-bid ✚) |
| **Commerce / classifieds** | PUB · MOTE · SYNC | OFFER · ESC · REP · DIS · PAY | gateway · indexer · arbiter | [TRACT](profiles/tract/README.md) | ▲ editorial |
| **Search / discovery** | PUB | — | indexer (TEE) | [SEARCH](profiles/search.md) | ▲ editorial + Sybil |
| **Storage / files** | MOTE · PUB · SYNC · storage | PAY | storage-provider · pin | [DMTAP §5.5](05-messaging.md) | ✔ |
| **Reachability** | Roles | — | reachability-adapter | [REACH](profiles/reachability.md) | ✔ |
| **Credentials / registries** | ATTEST · PUB · MOTE | REP · PAY | attester · curation | [ATTEST](primitives/ATTEST.md) + TRACT | ▲ authoritative-issuer |
| **Advertising** | PUB · indexer | OFFER · PAY | indexer (direct-buy) | [§22](22-public-objects.md) / TRACT | ✔ direct / ✖ surveillance |

`✔` covered · `▲` covered with a disclosed ceiling · `✚` needs a new seam · `✖` rejected by design.
Rides / delivery / bookings / auctions / freelance / commerce / classifieds are **one system wearing
different UIs** — they differ only in MATCH's assignment rule, or drop MATCH for RESERVE. Building the
primitives once builds the space.

This compressed table omits several product shapes (IoT, healthcare, maps/geo, calendar); the full
ledger is [`docs/research/coverage-matrix.md`](docs/research/coverage-matrix.md), and the five genuinely
uncovered shapes are enumerated as open IOUs under **Honest residual** below.

---

## FUTURE-PROOFING · APOCALYPSE-PROOFING · SECURITY

Three properties, one mechanism each — stated together because every profile inherits all three.

### Future-proof by seams, never by prediction
Every hard problem is a **pluggable slot** — a binding or a coordinator behind a stable interface
([`DIRECTION § 9`](DIRECTION.md), [`research/README § 4`](docs/research/README.md)). As the frontier
improves the fix slots in as a version bump and nothing above it changes: a better personhood method
is a REPUTATION binding swap; a TEE global matcher is a MATCH coordinator swap; a TEE search index is
an `indexer` swap; TEE anti-abuse with a global view but no plaintext is a `labeler` upgrade. The
product **converges on centralized quality while keeping sovereignty, with no rearchitecture.** What
does *not* converge is the small structural set (§below) — because those are not technical problems.

### Apocalypse-proof by graceful degradation + reconcile
Every object is self-authenticating (signed by a key or named by its content hash), so it verifies
identically over a mesh, over HTTPS, or on an SD card — the sneakernet test
([`substrate/OFFLINE.md`](substrate/OFFLINE.md)). Connectivity governs **reach, never function**. Remove
it and every service collapses to its local-trust form — a local order book instead of a global matcher,
web-of-trust instead of a personhood attester, a known local arbiter instead of a staked market — and
**still works** ([`DIRECTION § 6`](DIRECTION.md)). Reconnect and replicas reconcile from the signed CRDT
and the feed chains alone, idempotent and order-independent; convergence never hides a broken invariant
(equivocation surfaces as transferable evidence, not a clean merge). RESERVE is the single-writer case
where offline safety is paid entirely in liveness; the offline-money hard case is disclosed, not faked.

### Security floor, stated once and inherited
[`THREAT-MODEL.md`](THREAT-MODEL.md) is the checklist every capability, binding, coordinator, and
profile is an instance of: **fail closed** on any security-relevant uncertainty; **intrinsic
authenticity** (verify the object, never trust the server, [`§22.5.1`](22-public-objects.md));
**identity ≠ name** ([`§1`](01-identity.md)); **no silent downgrade** blind→terminating or
encrypted→plaintext ([`CONTRACT § 3.2`](coordinator/CONTRACT.md)); **authorize, never classify**, so
anti-abuse cannot re-centralize ([`CONTRACT § 4`](coordinator/CONTRACT.md)); **content-visibility
declared** by every intermediary at a stated assurance level (`structural` provable / `attested`
TEE / `declared` honest-trust); **no protocol token, ever** ([`DIRECTION § 5`](DIRECTION.md)).

---

## Honest residual

- **Five product shapes are genuinely uncovered — open IOUs, not hidden.** (1) **IoT / constrained
  device** — needs a CoAP/MQTT transport binding; (2) **hosted-blind private AI** — needs the
  provisional `compute` kind ([`coordinator/CONTRACT § 5`](coordinator/CONTRACT.md)); (3) **maps /
  geo-proximity** — the waist addresses by content-hash and by key, never by coordinate, so *"what's
  near lat/lng"* needs a geo-index binding / coordinator; (4) **sealed-bid / second-price auctions** —
  need a commit-reveal convention over SYNC; (5) **cross-party calendar negotiation** — composes over
  SYNC + RESERVE but is unspecified ([`coverage-matrix`](docs/research/coverage-matrix.md)).
- **This is a map, not a proof.** Reduction to the primitives is shown; deployment at centralized
  scale is not. A `✔` above means "composes from the primitives," never "shipped and measured." Global
  cross-publisher product identity, unpaid pinning of a departed node's catalogue, and an Uber-quality
  global matcher are all mechanism-present and demonstration-absent
  ([`coverage-matrix § 5`](docs/research/coverage-matrix.md)).
- **Four root ceilings recur; no seam closes them.** Every `▲` traces to one of **global anti-Sybil**,
  **physical-event oracle**, **legal / authoritative-issuer**, or **editorial governance**
  ([`DIRECTION § 8`](DIRECTION.md)). They are consequences of not being a single surveilling company —
  disclosed, not solved; several are the point.
- **Two things are refused, not missing.** Coercion-resistant public-election voting (harder than
  anti-Sybil) and surveillance-based ad markets (rejected by design) — [`DIRECTION § 8`](DIRECTION.md).
- **The `attested` level trades trust, it does not remove it.** TEEs close several gaps by exchanging
  operator-trust for chip-vendor-trust and have a side-channel history; disclosed as `attested`, never
  sold as trustless ([`CONTRACT § 3.4`](coordinator/CONTRACT.md)).
- **ESCROW's coordinator is the one honest exception to non-load-bearing.** Licensed money-holding for
  strangers does not self-extinguish; the design keeps only the weaker guarantee that the operator
  class is permissionless, competing, per-order, replaceable, and never key-holding
  ([`primitives/ESCROW.md`](primitives/ESCROW.md), [`docs/research/PRIMITIVES.md § 6`](docs/research/PRIMITIVES.md)).
- **The maturity claims are a snapshot.** Coverage rests on the 2026-07 bindings survey; every maturity
  and demand claim must be re-checked before any coordinator or binding is relied on in production
  ([`bindings/README.md`](bindings/README.md), [`docs/research/README § 6`](docs/research/README.md)).
- **One core, many hand-rolled surfaces today.** The suite currently has ~5 independent
  sync/identity/feed implementations, none byte-interoperable; the plan to converge them on one audited
  core is [`substrate/BINDINGS.md`](substrate/BINDINGS.md), the honest status is
  [`substrate/ADOPTION.md`](substrate/ADOPTION.md). This index describes the target tree, not a shipped one.
