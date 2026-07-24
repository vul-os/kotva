# Primitive — RESERVE (atomic single-owner reservation)

> **Status:** additive normative primitive spec. It defines **no new substrate bytes**: RESERVE is a
> *composition* — a single-owner bounded-counter over the [`SYNC.md`](../substrate/SYNC.md) op algebra,
> a single-writer LWW receipt register, and a MOTE-carried request. Its wire objects are ordinary
> substrate objects (common header of [`profiles/wrap § 3.2`](../profiles/wrap/02-objects.md); byte
> home is a concrete booking profile). This document owns the **object roles, the single-writer
> invariant, and the honest ceiling** for the RESERVE cell of [`DIRECTION § 2`](../DIRECTION.md).
> Where a byte home and this document disagree, the byte home governs.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are BCP 14 (RFC 2119 / RFC 8174).

---

## 1. Purpose

A **reservation** is a hold on a scarce, single-owned resource against a moment or an interval — a
calendar slot, a table sitting, a room-night, a ticket, an appointment. It is the one gig-cluster
service that needs **no matcher** ([`DIRECTION § 2`](../DIRECTION.md)): there is nothing to match,
because the resource has exactly **one owner**, and the owner's box is the **only writer** of the
hold. This collapses the hardest distributed-systems problem in booking — *two parties must not both
get the last seat* — into a non-problem: a contended write with a **single authorised author** is
not a consensus question (the WRAP [`§ 3.6`](../profiles/wrap/02-objects.md) argument, restated for
holds). Double-booking is therefore **structurally impossible under honest single-writer operation**
(§9 bounds "impossible" precisely). A reservation resolves on the owner's `placed → accepted /
declined` step, exactly as an offer's availability resolves on the seller's placement
([`tract § 3.9`](../profiles/tract/03-availability.md)); RESERVE is that step made a first-class,
signed, self-authenticating object. RESERVE has, uniquely among the primitives, **no coordinator**
([`DIRECTION § 2`](../DIRECTION.md), primitives table): §6 states why, and what that costs.

---

## 2. Objects / MOTE-kinds

Three roles. All carry the substrate common header (`v`, `kind`, `id`, `author`, `ts=Hlc`; keys 1–5,
[`wrap § 3.2`](../profiles/wrap/02-objects.md)); `id = 0x1e ‖ BLAKE3-256(canonical body)` and each is
COSE-signed by its `author`. A concrete profile assigns the `kind` bytes; the roles below are the
normative shape. `Hold` state is **not a wire object between parties** — it is the owner's internal
single-owner sync state (§3.2).

### 2.1 `Reservable` — the resource (owner-authored)

The single-owner descriptor of what may be reserved. Immutable once published; changing terms is a
new `Reservable` superseding the old (never an in-place edit — [`tract § 3`](../profiles/tract/03-availability.md)).

```cddl
Reservable = {
  ; ... common header (keys 1-5) ...
  6  => uint,        ; grain     0 = time-slots (claimed whole) | 1 = capacity-per-interval (shared to a cap)
  7  => tstr,        ; schedule  RFC 5545 VFREEBUSY / RRULE + RFC 7953 VAVAILABILITY (profiled, not re-specified)
  ? 8 => uint,       ; slotlen   minutes per slot            (grain 0 only)
  ? 9 => uint,       ; cap       units available per interval (grain 1 only; the bounded-counter bound, §3.2)
  11 => uint,        ; expires   unix seconds; no request valid after
  ? 12 => map,       ; policy    hold TTL default, cancellation window, lead time — owner policy, never protocol
}
```

`grain` reuses the [`tract § 3.5`](../profiles/tract/03-availability.md) distinction exactly: a time
slot is claimed whole by one hold; a capacity interval is shared by many holds up to `cap`. A
`Reservable` MUST carry exactly one grain.

`Reservable` carries no separate owner field: the sole authorised writer of holds/receipts against it
is its own common-header `author` (key 4). A `Receipt`'s `author` MUST equal that same key (§3.1);
RESERVE has no delegated-authoring mode — the same key that lists the resource is the only key ever
authorised to write a `Receipt` against it.

### 2.2 `ReservationRequest` — the intent (requester-authored)

A would-be reserver's signed intent, addressed to the owner as a MOTE (§6). The requester **cannot**
grant it; it is an offer to hold, resolved only by a `Receipt`.

```cddl
ReservationRequest = {
  ; ... common header (keys 1-5) — author = the requester IK ...
  6  => bstr,        ; reservable   Reservable.id
  7  => tstr,        ; slot         RFC 5545 DTSTART / period selecting the slot or interval
  ? 8 => uint,       ; qty          units requested (grain 1 only; default 1)
  ? 9 => uint,       ; hold_ttl     seconds the requester is willing to hold before auto-release
  ? 10 => tstr,      ; note         free text
}
```

### 2.3 `Receipt` — the decision + the signed reservation receipt (owner-authored)

The owner's authoritative, single-writer decision on one request. It **is** the reservation: a
`Receipt` in state `accepted` is the signed proof the requester holds the slot. Realized as a
substrate **LWW register** (SYNC [`§ 4.4`](../substrate/SYNC.md)) keyed by `(reservable, slot,
request)`; because the register's only admissible author is the `Reservable`'s own common-header
`author` (§3.1), last-writer-wins is unambiguous — the WRAP [`§ 3.6`](../profiles/wrap/02-objects.md) `Assignment` argument.

```cddl
Receipt = {
  ; ... common header (keys 1-5) — author MUST equal the Reservable's common-header author (key 4, §3.1) ...
  6  => bstr,        ; request      ReservationRequest.id it answers
  7  => bstr,        ; reservable   Reservable.id
  8  => tstr,        ; slot         the slot/interval held
  9  => uint,        ; state        0 placed | 1 accepted | 2 declined | 3 released | 4 expired
  ? 10 => uint,      ; qty          units granted (grain 1)
  ? 11 => tstr,      ; reason       decline/release reason (free text)
}
```

`released` and `expired` are terminal states of a prior `accepted`, so cancellation needs no separate
object — it is a later `Receipt` on the same `(reservable, slot, request)` register with a strictly
greater `ts`.

---

## 3. Normative rules

### 3.1 Single writer
A `Receipt` whose `author` is **not** the `Reservable`'s common-header `author` (key 4) MUST be
rejected (`ERR_NOT_OWNER`-class, FAIL_CLOSED). No party but the owner may author a hold, an
acceptance, a decline, or a release.
A requester MUST NOT self-issue any object in state `accepted`. This is the whole primitive: the
contended write has one authorised author, so *"two conflicting accepted holds on one unit"* is not a
state the protocol can reach **between honest participants** (§8).

### 3.2 The hold is a bounded counter across the owner's own devices
Where an owner accepts on more than one of its own devices (a device cluster, SYNC
[`§ 5.6`](../substrate/SYNC.md)), the resource count MUST be enforced by a **bounded counter** of the
escrow/demarcation family (§5), not a read-check-decrement of a PN-counter. The bound (`cap` for
grain 1, or `1` per slot for grain 0) is pre-partitioned into transferable **rights**; a device may
accept only from rights it holds, so under arbitrary loss/delay/reorder/partition the invariant
*accepted ≤ cap* holds **even from stale state** ([`tract § 6.2a`](../profiles/tract/06-cart.md)).
A device with no local right MUST fail closed (`ERR_RESERVE_EXHAUSTED`) — decline or queue — and MUST
NOT oversell. A plain counter measurably *does* oversell under partition and MUST NOT be used.

This invariant holds provided rights are never reclaimed from a device that is only *presumed* dead.
[`Tract § 6.2a` constraint 3](../profiles/tract/06-cart.md) states the residual precisely: reclaiming
rights stranded on a permanently-departed device is an explicit, fallible decision, never a silent
background sweep, because if the liveness call is wrong and the "dead" device returns, its stranded
rights are double-issued and *accepted ≤ cap* breaks. RESERVE MUST NOT reclaim a device's rights
automatically; a booking profile that reclaims stranded rights at all MUST treat the decision as
explicit, disclosed, and fallible, and MUST state this double-issue failure mode to its operators
(tract §6.2a constraint 3) rather than presenting reclamation as safe.

### 3.3 Resolution is `placed → accepted / declined`
On receiving a `ReservationRequest`, the owner MUST emit a `Receipt`. `placed` acknowledges receipt
without committing a right; `accepted` commits a right from the bounded counter (§3.2) and is the
signed reservation receipt; `declined` commits none. A request MAY be declined for any reason or
none — availability was a signal, never a promise ([`tract § 3.9`](../profiles/tract/03-availability.md)).
An `accepted` `Receipt` SHOULD be delivered to the requester (§6); the hold exists whether or not
delivery has happened yet.

### 3.4 Receipts bind to exactly one request/slot
A `Receipt` MUST cite the `request` content-address and the `slot` it grants; a verifier MUST treat a
`Receipt` as authoritative **only** for that `(reservable, slot, request)` triple. A `Receipt` MUST
NOT be honoured against a different slot, request, or `Reservable` (§7 replay-inertness).

### 3.5 Expiry and release
An `accepted` hold not confirmed/consumed within `hold_ttl` (or `policy` default) SHOULD be released
by the owner (`state = released/expired`), returning the right to the bounded counter. Only the owner
writes the release; a requester "cancels" by asking, and the owner records it. `expires` on the
`Reservable` bounds new requests, never existing holds.

### 3.6 No money, no matcher
A `Receipt` MUST NOT carry, move, or settle funds; consideration is a separate settlement leg (§4,
[`OFFLINE § 5`](../substrate/OFFLINE.md)). RESERVE MUST NOT invoke a matcher — the owner *is* the
assignment authority — and MUST NOT depend on any coordinator to produce a hold (§6).

---

## 4. Composition with the other primitives

- **OFFER** ([`22-public-objects`](../22-public-objects.md), tract catalogue) — a `Reservable` is
  discovered as a public signed listing; its `schedule` is the OFFER availability axis
  ([`tract § 3`](../profiles/tract/03-availability.md)). OFFER advertises; RESERVE holds. Discovery
  MAY use an `indexer` coordinator; the **hold never does** (§6).
- **MATCH** — deliberately **absent**. RESERVE is the [`DIRECTION § 2`](../DIRECTION.md) "no matcher"
  branch: single owner ⇒ nothing to match. A service that needs supply↔demand assignment is MATCH,
  not RESERVE.
- **ATTEST** — the `Receipt` is a narrow ATTEST ("owner IK grants this hold"); reputation over kept
  vs. broken reservations is ordinary attestation (`REPUTATION`), never a protocol score.
- **ESCROW / PAY** — a paid booking pairs an `accepted` `Receipt` with a settlement attestation on an
  existing rail ([`bindings`](../bindings/README.md), x402/stablecoin). The hold and the payment are
  **separate legs**; substituting one for the other, or presenting an unpaid hold as paid, is the
  rail-class-substitution defect ([`OFFLINE § 5.2`](../substrate/OFFLINE.md)).
- **DISPUTE / ORACLE** — "was the reservation honoured?" is the physical-event ceiling
  ([`DIRECTION § 8`](../DIRECTION.md)): confirm-plus-dispute via a staked `arbiter`, never provable
  by the protocol.

---

## 5. Binding adopted

Per [`DIRECTION § 3`](../DIRECTION.md) and [`bindings/README.md`](../bindings/README.md), RESERVE
reinvents nothing:

| Need | Bind to | Not |
|---|---|---|
| No-oversell hold under partition | **bounded counter** — escrow/demarcation family (Preguiça/Balegas et al., SRDS 2015), in production as AntidoteDB `antidote_crdt_counter_b` (via SYNC op algebra) | a new inventory CRDT |
| Schedule / availability | **RFC 5545** `VFREEBUSY` / `RRULE` + **RFC 7953** `VAVAILABILITY` | a bespoke schedule grammar |
| Convergent multi-replica state | **SYNC** LWW register + single-owner device cluster ([`§ 4.4`](../substrate/SYNC.md), [`§ 5.6`](../substrate/SYNC.md)) | a new merge algebra |
| Request/receipt transport | **MOTE** store-and-forward ([`OFFLINE § 3.2`](../substrate/OFFLINE.md)) | a booking API |
| Settlement (if paid) | **x402 + stablecoin** ([`bindings`](../bindings/README.md)) | **a protocol token (none exists, none will)** |

---

## 6. Scale-invariance — and why RESERVE never grows a coordinator

The [`DIRECTION § 6`](../DIRECTION.md) trust anchor slides for the other primitives; for RESERVE the
authority **does not slide at any scale** — it is always the owner's box. This is the primitive's
distinguishing property, not an omission:

| Function | Small / mesh (offline) | Global |
|---|---|---|
| **The hold** | owner's box (single writer) | **owner's box (single writer) — unchanged** |
| Discovery of the `Reservable` | following-graph / local cache | a swappable `indexer` (OFFER's, never the hold's) |
| Reach of the request to the owner | mesh mailbox / sneakernet | relay / mailbox coordinator (blind carrier) |

Only **reach** (discovery, transport) has a coordinator, and those coordinators are OFFER's and the
substrate's, obeying [`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md) — content-blind,
swappable, non-load-bearing. The **hold itself** is coordinator-free from a village to a planet,
because a single-owned resource never needs a global view: there is no cross-owner invariant to
enforce, only each owner's own concurrency (§3.2). A world of a billion independent calendars is a
billion independent single-writer registers, and they never need to agree on anything.

---

## 7. Offline / apocalypse behaviour + reconcile

RESERVE passes the sneakernet test ([`OFFLINE § 1`](../substrate/OFFLINE.md)); it is the reference
case for **R-SYNC-1** (single-writer authority enforces the cross-replica invariant, never the
merge).

| Action | Grade ([`OFFLINE § 2`](../substrate/OFFLINE.md)) | Behaviour |
|---|---|---|
| Author / read a `Reservable` | `full` | local-first, self-verifying |
| Make a `ReservationRequest` | `deferred` | captured as a signed intent MOTE; store-carry-forward to the owner; **MUST NOT be shown as held** until an `accepted` `Receipt` returns (R-GRADE-2) |
| Owner `accept` / `decline` | `full` | the owner *is* the authority — needs no network to decide; the bounded counter (§3.2) enforces *accepted ≤ cap* across the owner's own offline devices |
| Deliver the `Receipt` | `deferred` | returns to the requester on reconnect |

- **R-RESERVE-1 (single-writer invariant).** The no-double-book invariant MUST be carried by the
  single-writer authority (§3.1) and the bounded counter (§3.2) — **never assumed away by the CRDT
  merge** ([`OFFLINE § 3.4`](../substrate/OFFLINE.md), R-SYNC-1). A clean merge is a statement about
  availability, not about the invariant.
- **R-RESERVE-2 (durable rights).** A bounded-counter right and every rollback-defended counter
  (`Reservable`/feed `seq`, HLC) MUST be persisted to non-volatile storage **before** the object that
  spends it is emitted ([`OFFLINE`](../substrate/OFFLINE.md) R-ID-1), so an offline crash-and-restart
  cannot re-spend a right and oversell.

**Reconcile on reconnect.** Divergent state heals by version-vector diff → range-Merkle drill-down →
op exchange (SYNC [`§ 4`](../substrate/SYNC.md)); requests dedup by content-address (idempotent,
order-independent, R-REC-1). Two of the owner's devices that *both* accepted while partitioned is the
one interesting case: the bounded counter **prevents** joint over-acceptance up to `cap` (safety, but
see the reclamation residual, §3.2), so reconcile finds no oversell to fix; a device that ran out of
local rights merely **stranded quota** — declining requests it could have served — which surfaces as
spurious "sold out," never as an oversell (§8). A `Receipt` is a private point-to-point object, not a
feed entry: §2.3 keys it `(reservable, slot, request)`, so each victim's `accepted` `Receipt` lives on
a *different* register and is delivered as a sealed MOTE to that one requester (SEC-R5) — no replica
ever compares two victims' receipts, so R-REC-1's dedup-by-content-address sees no conflict to fork;
the receipts remain signed and durable evidence regardless. Whether a genuinely over-committed
*dishonest* owner's evidence is then detected — and how that differs by grain — is the honest scoping
§9 owns, not a reconcile property.

---

## 8. Security MUSTs

Inherits the family posture of [`THREAT-MODEL.md`](../THREAT-MODEL.md); each MUST names its residual.

- **SEC-R1 — Owner-only authority (fail-closed).** A `Receipt` not authored by the `Reservable`'s
  common-header `author` (key 4) MUST be rejected (§3.1); an unverifiable or missing `Receipt` is
  **not a hold** (SEC-1 fail-closed).
- **SEC-R2 — Intrinsic authenticity.** `Reservable`, `ReservationRequest`, and `Receipt` are
  COSE-signed and content-addressed; verification is offline and carrier-independent (SEC-2). A hold
  is exactly a valid signed `accepted` `Receipt` — no server assertion counts.
- **SEC-R3 — Replay-inert / no substitution.** A `Receipt` binds `(reservable, slot, request)` and an
  HLC (§3.4); it MUST NOT be replayable to another slot, request, or resource, nor a `declined`
  flipped to `accepted` without a strictly-greater-HLC owner op (SEC-8 replay/downgrade).
- **SEC-R4 — No self-grant.** A requester MUST NOT author any `accepted` object (§3.1); the buyer
  cannot mint its own hold, which is *why* two racing requesters cannot both self-confirm.
- **SEC-R5 — Content-visibility declared.** Request and receipt travel as MOTEs; carriers are
  `blind`/`blind-routing` and trusted for nothing ([`CONTRACT § 3`](../coordinator/CONTRACT.md)). The
  owner's box necessarily reads the reservation (it is the authority) — a `terminating` boundary that
  is the resource owner, disclosed, not a third party.
- **SEC-R6 — Bounded counter, not read-check-decrement (§3.2).** The no-oversell property MUST rest
  on the pre-partitioned bounded counter, which holds under partition; a check-then-write counter
  MUST NOT be claimed to prevent oversell (SEC-1).

---

## 9. Honest residual

- **"Structurally impossible" is scoped to *honest* single-writer operation.** No interleaving of
  honest parties double-books: the buyer cannot self-grant (SEC-R4) and the owner's own devices are
  bounded (§3.2). A **malicious owner can** sign two `accepted` receipts for one unit — that is not a
  distributed-systems failure but a party misbehaving in a way that is **signed, permanent, and
  attributable** (the WRAP [`§ 3.6`](../profiles/wrap/02-objects.md) point). The `Receipt` makes
  double-booking *evidence*, not *impossibility*, against a dishonest owner — and how detectable that
  evidence is **differs by grain**: at **grain 0**, two signed `accepted` receipts for one unit become
  dispute evidence only once the two victims combine them out of band; at **grain 1 (capacity)**, two
  receipts alone prove nothing, because establishing oversell requires summing granted `qty` across
  *every* `accepted` receipt for the interval against `cap` — a computation no third party, and no
  single victim, can perform. Capacity oversell by a dishonest owner is therefore durable evidence but
  **not individually detectable**.
- **Non-Byzantine — protects the seller from itself, not the buyer from the seller.** The bounded
  counter's guarantee is **safety-only, paid entirely in liveness, so long as no stranded rights are
  reclaimed** ([`tract § 6.2a`](../profiles/tract/06-cart.md),
  [`tract § 3.9`](../profiles/tract/03-availability.md)): a partitioned device **strands the quota it
  holds** and returns **spurious "sold out"** rather than ever overselling. The reclamation rule and
  its double-issue failure mode are normative in §3.2; violating it is what breaks this guarantee. It
  shields an owner from its own concurrency; it makes **no** promise to a buyer
  about a dishonest owner. Documentation MUST NOT present it as trust between the parties.
- **The hold does not prove the resource is honoured.** Whether the reserved table, room, or slot is
  actually delivered reduces to confirm-plus-dispute — the physical-event oracle ceiling
  ([`DIRECTION § 8`](../DIRECTION.md)) — and is *more* exposed offline, where the oracle is
  unreachable ([`OFFLINE § 5.3`](../substrate/OFFLINE.md)).
- **Payment is a separate leg with its own residual.** A paid booking inherits ESCROW's and
  [`OFFLINE § 5`](../substrate/OFFLINE.md)'s ceilings; RESERVE moves no money and custodies nothing.
- **No coordinator is a feature with a cost.** RESERVE needs none for the hold (§6), so it cannot be
  centrally censored — but discovery of a `Reservable` inherits OFFER's indexer ceiling (no discovery
  without an indexer; spam is index-local), and reach inherits the transport's. The hold is sovereign;
  finding it is not.
