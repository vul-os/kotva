# Substrate Capability ③ — Sync

> **Status:** this is the **one genuinely new normative specification** in the substrate. Unlike
> [`IDENTITY.md`](IDENTITY.md) and [`FEEDS.md`](FEEDS.md), which *profile* existing sections, this
> document specifies machinery that no single numbered section fully covers: a **signed, deterministic,
> multi-author CRDT operation algebra with range-Merkle reconciliation, first-class signed snapshots,
> and sparse namespace sync.** Its **semantics are grounded in** the existing §5.6 device-cluster CRDTs
> (the `dmtap-clustersync` reference), and its **wire shape is grounded in** the flowstock stateless
> sync protocol (`GET /sync/vector` · `POST /sync/pull` · `POST /sync/ops`). Where this document and
> §5.6 agree, §5.6 remains the normative home of the *single-owner device-cluster* profile and governs
> it; this document is the normative home of the **multi-author generalization** and everything below
> that §5.6 does not specify.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174.

**Proposed registries.** This document proposes a new error-subsystem byte **`0x0A`** (`0x0A00`–`0x0AFF`,
"SYNC substrate") and a capability token `sync-1`, registered additively under the §21.14
new-subsystem-byte policy and §10.2 — exactly as §22 (DMTAP-PUB) registered subsystem `0x09` and `pub-1`.
No existing code, token, or top-level version is changed; a node that does not advertise `sync-1` is
never expected to speak it.

---

## 1. The one idea, and the flowstock test

A product that keeps structured state — inventory, a document, a settings tree, a task board — on more
than one replica needs those replicas to **converge without a coordinator**, tolerate offline edits, and
**merge deterministically** so that every replica that has seen the same operations computes byte-identical
state. That is a CRDT. DMTAP already has one (§5.6), but §5.6 is scoped to **a single owner's device
cluster**, where all writers are devices of one identity and authenticity is supplied by an ambient MLS
group (ops ride *unsigned* inside an encrypted, membership-authenticated group).

The substrate generalizes this to **multi-author** sync — any set of independent identities converging on
shared state — by making the **operation itself the unit of authenticity**: each op is **COSE-signed**
(RFC 9052) by an author key that chains, via a `DeviceCert`, to an `IK` ([`IDENTITY.md`](IDENTITY.md)).
No shared secret group is required, so two products built by different parties can converge on any
namespace they both subscribe to.

This document **MUST pass the flowstock test** ([`README.md § 4.1`](README.md#41-the-flowstock-test--can-a-product-sync-without-reading-mail)):
a product that syncs structured state **MUST be able to implement it from this document plus
[`IDENTITY.md`](IDENTITY.md) alone** — the op algebra, the signing rule, the reconciliation wire
protocol, snapshots, and sparse sync are all specified here, with no forward reference into §2/§5/§7/§8
that a reader must chase to get the bytes right. References into §5.6 for *grounding* are informative.

---

## 2. Model

### 2.1 Replicas, authors, objects, namespaces

- A **replica** is a store that holds sync state and exchanges ops with peer replicas. Any node may be a
  replica; there is **no primary and no coordinator** (leaderless).
- An **author** is an identity (an `IK`, §1.2) that produces ops. Each op is signed by an operational
  **author key** certified to that `IK` by a `DeviceCert` (§1.2). *Multi-device of one owner* is the
  special case where every author key chains to the *same* `IK` — that is the §5.6 device cluster.
  *Multi-owner* is the general case where author keys chain to *different* `IK`s admitted by policy (§9).
- An **object** is a keyed piece of state (`target`, a string id) whose value is governed by exactly one
  **CRDT type** (§4). A **field** further keys a register or counter within an object.
- A **namespace** (`ns`, a short string) partitions objects into independently-syncable collections. A
  replica **subscribes** to a set of namespaces and exchanges ops only within them (§7, sparse sync). The
  empty namespace `""` is the default single-collection case.

### 2.2 Determinism is the contract

Every merge in this document is a **join** — commutative, associative, and idempotent — over
deterministically-encoded state, so **any two replicas that have applied the same set of ops compute the
same bytes.** Two rules make this hold across independent implementations:

1. **Deterministic CBOR everywhere.** Every op, snapshot, and reconciliation message is encoded as
   deterministic CBOR (RFC 8949 §4.2, §18.1.1): shortest-form integers, integer-keyed maps sorted by
   encoded key, no floats/tags/indefinite lengths, `null` rejected on the wire, unknown keys rejected in
   signed objects (§18.1.2). JSON appears **only** at a client edge (as JMAP does for mail, §8), never on
   the sync wire.
2. **Total order on every tie.** The tiebreak for any "latest wins" decision is the **Hybrid Logical
   Clock** total order (§3), and where two ops carry the *identical* HLC the tiebreak descends to the
   **encoded-CBOR value bytes** (larger byte string wins) — so even "same HLC, different value" converges
   byte-identically. No merge ever depends on wall-clock accuracy or on arrival order.

---

## 3. Time: Hybrid Logical Clocks (HLC)

Causality and ordering use an HLC, grounded in the §5.6 cluster HLC and the flowstock HLC.

```cddl
Hlc = {
  1 => u64,      ; wall     unix milliseconds (ordering hint + skew bound, NOT relied on for correctness)
  2 => u32,      ; counter  logical tick within a wall-ms
  3 => ik-pub,   ; author   the author key producing this timestamp (globally unique tiebreak)
}
```

- **Total order (normative).** HLCs compare **lexicographically by `(wall, counter, author)`**. Because
  `author` is a public key, no two distinct authors ever tie, so the order is total across all replicas.
- **Tick (local mint).** On producing an op: `now = wall_clock_ms()`; if `now > wall` then
  `wall = now, counter = 0`; else `counter += 1`. Strictly monotonic per author.
- **Observe (on receiving a remote op).** Fold the remote HLC forward: if `remote.wall > wall`, or
  (`remote.wall == wall` and `remote.counter >= counter`), advance `wall`/`counter` past it, so a future
  local tick always sorts after every op already seen. This makes a backwards or fast wall clock unable
  to mint a stale-ordering timestamp.
- **Skew bound (fail-closed).** An op whose `wall` is more than the profile skew window (§16-class value,
  default **±120 s**, from §5.6 `HLC_SKEW_MS`) from the receiver's clock is rejected
  (`ERR_SYNC_HLC_SKEW`, `0x0A05`). This bounds how far a malicious author can push ordering into the
  future or past.

> **Grounding.** The flowstock reference encodes the HLC as a lexically-sortable string
> `"{ms:013d}-{counter:04x}-{author}"`; the §5.6 reference encodes it as an integer-keyed CBOR map.
> The substrate normative encoding is the CBOR map above (deterministic CBOR is the substrate primitive,
> §2.2); a string form is an equivalent client-edge convenience and MUST canonicalize to the same order.

---

## 4. The CRDT operation algebra

An operation is the atomic, signed mutation. All six CRDT types share one op envelope; `kind` selects the
type and dictates which fields are meaningful.

### 4.1 The `SyncOp` envelope

```cddl
SyncOp = {
  1  => u8,          ; kind    CRDT/op discriminator (§4.2–§4.7 table below)
  2  => tstr,        ; ns      namespace (§7); "" = default
  3  => tstr,        ; target  object id
  ? 4 => tstr,       ; field   register/counter field, RGA/tree edge selector — per-kind
  ? 5 => cv,         ; value   ext-value payload (LWW value, counter delta, RGA atom) — per-kind
  6  => Hlc,         ; hlc     the op's HLC (§3); its `author` is the producing key
  ? 7 => [+ AddTag], ; observed OR-Set: the specific add-tags a remove cancels — per-kind
  ? 8 => OpRef,      ; ref     RGA left-origin / tree parent / prev-chain reference — per-kind
}
AddTag = { 1 => ik-pub, 2 => Hlc }   ; a globally-unique add identity: (author, hlc)
OpRef  = { 1 => tstr, ? 2 => Hlc }   ; a reference to another element by (target[, hlc])
cv     = ext-value                    ; the deterministic-safe value subset of §18.3.6
```

- `value` is restricted to the **`ext-value` subset** (§18.3.6): text, byte strings, unsigned/negative
  integers, booleans, and homogeneous arrays thereof — **no** integer-keyed maps, floats, tags, or
  `null`, so a value can never smuggle an un-canonicalizable or ambiguous encoding (the §5.6
  `is_ext_value` rule). A violating op is rejected (`ERR_SYNC_OP_INVALID`, `0x0A03`).
- **Authenticity: each `SyncOp` is COSE-signed.** The wire object is a **`COSE_Sign1`** (RFC 9052) whose
  payload is `det_cbor(SyncOp)` and whose protected header binds the signer's author key; the DS-tag
  `DMTAP-SYNC-v0/op ‖ 0x00` domain-separates the signing preimage (§18.1.6). The signature MUST verify
  under the `hlc.author` key, and that key MUST chain to an admitted `IK` via a non-revoked `DeviceCert`
  (§9). A failure is `ERR_SYNC_OP_SIG_INVALID` (`0x0A02`, FAIL_CLOSED_BLOCK); an unauthorized author is
  `ERR_SYNC_AUTHOR_UNAUTHORIZED` (`0x0A01`).
  - **The `COSE_Sign1` envelope (normative, frozen — `SYNC-OP-02`).** The wire object is the RFC 9052
    `COSE_Sign1` four-element array `[protected, unprotected, payload, signature]`, itself encoded as
    deterministic CBOR (§2.2):
    - **`protected`** is a `bstr` wrapping `det_cbor` of the protected-header map `{1: alg, 4: kid}`:
      label **`1` (`alg`)** = the COSE algorithm of the op's suite — **`-8` (EdDSA)** for suite `0x01`
      (Ed25519); for suite `0x02` the classical **and** PQ algorithms are each present and **both**
      signatures MUST verify under the §18.1.6 AND-composition. Label **`4` (`kid`)** = the raw
      `hlc.author` public-key bytes. `kid` lives in the **protected** (integrity-covered) header on
      purpose: the asserted signer key is folded into the signature, so a key substitution is a
      *verification failure*, never a silent mis-attribution. No other protected-header labels are
      emitted, and the map is deterministic-CBOR (ascending integer keys, shortest-form).
    - **`unprotected`** is the **empty map** `0xA0`. Nothing is carried outside the signature — every
      byte that matters to a `SyncOp` is either in the signed `protected` header or in `payload`.
    - **`payload`** is the `bstr` `det_cbor(SyncOp)` (never `nil`): a `SyncOp` is always carried inline,
      not detached.
    - **`signature`** = `Sign(sk_author, det_cbor(Sig_structure))`, where the **signable preimage** is the
      RFC 9052 §4.4 `Sig_structure` array `["Signature1", protected, external_aad, payload]` with
      **`external_aad` = the DS-tag `DMTAP-SYNC-v0/op` ‖ `0x00`** (a `bstr`). Carrying the DS-tag in
      `external_aad` is the RFC-9052-idiomatic realization of the §18.1.6 rule `preimage = DS-tag ‖ body`
      (matching how §22/MOTE domain-separate their native `sig-val` preimages, but expressed through the
      COSE mechanism COSE gives us): the tag is bound into the signature yet never transmitted, so a
      `COSE_Sign1` minted for any other DMTAP object can never verify as a `SyncOp`, and vice-versa,
      **without a discriminator flag a peer could flip**. Verification recomputes this exact
      `Sig_structure` and checks it under `kid` = `hlc.author`; a flipped `payload` byte, a substituted
      `kid`, or any other `external_aad` is `ERR_SYNC_OP_SIG_INVALID` (`0x0A02`).
  - **Op content address (`op-id`).** An op's content address — its dedup key (§5.2), a `SyncFrame`'s
    per-op back-link `ref` target (below), and the reconciliation leaf (§5.3) — is
    `op-id = 0x1e ‖ BLAKE3-256( "DMTAP-SYNC-v0/op-id" ‖ 0x00 ‖ det_cbor(SyncOp) )` (a §18.1.5 v0 `hash`,
    33 bytes). It is computed over the **`SyncOp`**, *not* over the `COSE_Sign1` envelope, so a per-op-
    signed op and the identical op carried inside a `SyncFrame` share **one** identity and dedup/fingerprint
    identically — a necessary property, since the two signing modes are interchangeable on the wire (below).
  - **Batching (OPTIONAL, amortized signatures).** A producer MAY emit a **`SyncFrame`** — a hash-chained
    run of that author's ops with a single `COSE_Sign1` over the frame head — so the signature amortizes
    over many ops, exactly as a signed `FeedHead` transitively commits a chain of unsigned `FeedEntry`s
    ([`FEEDS.md § 4.1`](FEEDS.md), §22.4.1) and a signed cluster-journal segment commits its ops
    (§5.6.3(b)). Within a frame, each op carries `ref` = the content hash of the prior op, genesis
    excepted; a broken back-link is `ERR_SYNC_FRAME_CHAIN_BROKEN` (`0x0A08`, HALT_ALERT). Per-op signing
    (baseline) and frame signing (amortized) are interchangeable on the wire and negotiated by `sync-1`
    sub-tokens.

### 4.2 Op kinds

| `kind` | Name | CRDT type | Uses fields |
|:------:|------|-----------|-------------|
| `1` | `set-add` | OR-Set (add-wins) | `target`, `value`=element, `hlc` (the add-tag) |
| `2` | `set-remove` | OR-Set | `target`, `value`=element, `observed`=cancelled add-tags |
| `3` | `lww-set` | LWW register | `target`, `field`, `value`, `hlc` |
| `4` | `death` | remove-wins death-certificate | `target`, `field`=death class or `"live"`, `hlc` |
| `5` | `counter` | PN-counter | `target`, `field`, `value`=signed delta, `hlc` |
| `6` | `seq-insert` | RGA sequence | `target`, `value`=atom, `ref`=left origin, `hlc`=element id |
| `7` | `seq-remove` | RGA sequence | `target`, `ref`=element to tombstone |
| `8` | `tree-move` | movable tree | `target`=node, `ref`=new parent, `field`=ordering key, `hlc` |

### 4.3 OR-Set — add-wins observed-remove (grounded in §5.6)

- **State.** Per element: a set of **add-tags** `{author, hlc}` and a set of tombstoned add-tags.
- **`set-add`** inserts a fresh, globally-unique add-tag (its own `hlc`). **`set-remove`** tombstones the
  **specific** add-tags listed in `observed` — it cancels only adds the remover had seen, so a concurrent
  unseen add survives (**add-wins**). A remove may legitimately arrive before its add and still converge.
- **Presence.** An element is present iff it has **at least one add-tag not covered by a tombstone**.
- **Merge** is per-element set union of adds and of tombstones (a join).
- **Causal integrity (fail-closed).** A `set-remove` MUST cite ≥1 add-tag, and **no cited add-tag may
  post-date the remove's own HLC** ("you cannot have observed an add from the future") — a violation is
  `ERR_SYNC_OP_INVALID` (`0x0A03`). This validity is state-free, so it never depends on local order.

### 4.4 LWW register — last-writer-wins by HLC (grounded in §5.6)

- **State.** Per `(target, field)`: a single `(hlc, value)` cell.
- **`lww-set`** writes `(hlc, value)`. **Winner = greater HLC.** At an **exact HLC tie** (only possible
  same-author same-tick, or a forged duplicate), the winner is the one whose **`det_cbor(value)` byte
  string is larger** — so convergence is byte-identical even under a tie.
- **Merge** keeps the winning cell (a join).

### 4.5 Death-certificate — remove-wins durable delete (grounded in §5.6)

For deletions that must **not** be silently resurrected by a concurrent benign edit (privacy redactions,
expiries, policy removals), the death dimension **dominates** the OR-Set.

- **State.** Per object: an LWW register over `DeathState ∈ {Live, Deleted(class)}`, where
  `Deleted > Live` in the state order and `class ∈ {redact, expires, sensitive}` (an ordered enum).
- **`death` (kind 4)** writes `Deleted(class)` (field = the class token) or `Live` (field = `"live"`).
  **Winner = greater HLC; at an exact HLC tie, greater `DeathState` wins ⇒ `Deleted` beats `Live`
  (remove-wins, fail-safe toward deletion).**
- **Domination (the D3 invariant).** An object is observably present iff **`!deaths.is_deleted(target)`
  AND the OR-Set says present.** A bare `set-add` never writes the death dimension, so it can **never**
  outrank a death certificate — even with a numerically greater wall clock. Only an explicit `Live`
  write with a **strictly greater** HLC than the certificate revives an object. This closes the
  resurrection hole where a concurrent re-label would revive redacted content.

### 4.6 PN-counter — increment/decrement (NEW; standard positive-negative counter)

No §5.6 analogue exists; specified here for products that count (inventory on-hand, votes, quotas).

- **State (normative).** Per `(target, field)`, per author `a`, the **set of that author's applied
  deltas keyed by `op-id`**: `D[a] : op-id → int`. The per-author aggregates are *derived*, never stored
  as the merge unit:

  ```
  P[a] = Σ { d  : (id, d) ∈ D[a], d ≥ 0 }        N[a] = Σ { |d| : (id, d) ∈ D[a], d < 0 }
  ```
- **`counter` (kind 5)** carries a **signed delta** in `value` and applies to the **author's own**
  entry: applying an op with delta `d` inserts `(op-id, d)` into `D[author]`, so a positive delta `d`
  advances `P[author]` by `d` and a negative delta `−d` advances `N[author]` by `d`. An author may only
  advance its **own** entry (a signed op from author `a` MUST NOT mutate `D[b]`/`P[b]`/`N[b]`), enforced
  by the op signature — `ERR_SYNC_COUNTER_FOREIGN` (`0x0A06`) otherwise.
- **Value.** `Σ_a P[a] − Σ_a N[a]` — equivalently, `Σ_a Σ_{(id,d) ∈ D[a]} d`.
- **Merge (normative — `SYNC-PN-01`).** Per author, the **union of the `op-id`-keyed delta sets**:
  `(D₁ ⊔ D₂)[a] = D₁[a] ∪ D₂[a]`. Because an `op-id` is the content address of the whole `SyncOp`
  (§4.1), the same key always carries the same delta, so the union is an ordinary **set union over
  `(op-id, delta)` pairs**: **commutative, associative, and idempotent** — a join in the §2.2 sense.
  Redelivery of an op re-inserts a key already present and is therefore a no-op (no double-counting);
  the `hlc` still orders an author's own successive deltas for display and for the version vector, but
  the merge does **not** depend on it.
  - **Associativity is a REQUIREMENT, not an incidental property.** Replicas merge in arbitrary
    groupings — a partition heals in one order here and another order there — so `(A ⊔ B) ⊔ C` **MUST**
    equal `A ⊔ (B ⊔ C)` for every partial state `A`, `B`, `C`, including states that hold **different
    subsets of one author's ops**. Set union satisfies this unconditionally.
  - **Why the naive per-author `max` is wrong (correction — see §14).** Earlier text specified the merge
    as per-author `max` of `P` and of `N`, the classical *state-based* PN-counter join. That join is
    sound only when each replica's `P[a]` is derived from the **whole** of `a`'s op prefix — but §4.2
    kind 5 carries a **delta**, so two replicas can legitimately hold *different subsets* of one
    author's deltas (partition, sparse backfill, snapshot fast-join, range-Merkle drill-down that has
    not yet completed). Under `max` those two partial states collapse to the larger subtotal and the
    other subset's deltas are **silently lost**: with `a`'s deltas `+5` and `+3`, a replica holding only
    `+5` and one holding only `+3` merge to `P[a]=5`, and merging the third replica that holds both
    gives `P[a]=8` — so the result depends on the grouping, `max` is **not associative** over
    delta-carrying ops, and the "merge" is not a join at all. This is a *soundness* defect (lost
    writes), not a performance one, and it was found by an implementation's property test rather than
    by review. Keying by `op-id` fixes it without changing anything else: **whenever both replicas hold
    complete op sets — the only case the old text actually described — union and `max` compute
    identical `P`, `N`, and total**, so the corrected rule is a strictly stronger statement of the same
    semantics, not a different CRDT.
- **Compaction (relationship to §6.2).** Retaining one entry per delta forever is unnecessary. Below the
  **stability cut** (§6.2) every live replica is known to hold the *complete* prefix of every author's
  deltas, which is exactly the completeness condition under which the `max` reading is sound; a replica
  MAY therefore fold all of an author's below-cut deltas into a single retained entry — an aggregate
  `(P_cut[a], N_cut[a])` pair keyed by the cut HLC — and join those aggregates by **max** while joining
  the above-cut deltas by union. Aggregates at *different* cuts join by max soundly because each is a
  prefix sum of the same op order. Compaction thus never changes the observable total (§6.2), and no
  replica may fold a delta it cannot prove is below the cut (fail-closed: no cut ⇒ no folding).

> **Alternative for immutable ledgers.** A product whose counter is a sum of immutable facts (flowstock's
> `stock_movements`, summed at read time) MAY instead model each movement as a `set-add` of an immutable
> record and derive the total by a read-side `SUM` — a set-union CRDT with no counter state at all. This
> is the flowstock choice and is often simpler than a PN-counter where every increment is an auditable
> event. The PN-counter is for a *scalar* whose history need not be retained.

### 4.7 RGA sequence — ordered list (NEW; Replicated Growable Array)

For ordered text or lists that multiple authors edit concurrently (a shared document line, an ordered
checklist). No §5.6 analogue; specified here.

- **State.** A set of **atoms**, each with a unique **element id** = its insertion `hlc` (`{author,…}`
  globally unique), a **left-origin** reference (`ref`, the element id it was inserted immediately after;
  the sentinel `⊥` = list head), an `ext-value` payload, and a tombstone flag.
- **`seq-insert` (kind 6)** creates an atom whose id is its `hlc`, inserted after `ref`. **`seq-remove`
  (kind 7)** tombstones the atom named by `ref` (tombstones are retained until GC, §6, so a concurrent
  insert whose origin is a removed atom still resolves).
- **Order (normative RGA rule).** Atoms sharing a left-origin are ordered by **descending element-id HLC**
  (later insertions sort *earlier* among same-origin siblings — the standard RGA "insert-after with
  newer-first" rule), recursively; the sequence is the pre-order walk of this tree skipping tombstoned
  atoms. Because element ids are HLC-total-ordered, the resulting sequence is identical on every replica
  that has applied the same op set. An insert referencing an unknown origin is **buffered** until the
  origin arrives (causal readiness), not rejected — `ERR_SYNC_SEQ_ORIGIN_MISSING` (`0x0A07`) is raised
  only if the buffer is bounded and overflows, never as a convergence fault.
- **Merge** is set union of atoms + union of tombstones (a join); the order is recomputed by the rule
  above, so merge is order-independent.

### 4.8 Movable tree — hierarchy with safe moves (NEW; cycle-safe replicated tree)

For a tree that multiple authors reparent concurrently (a folder/outline/scene graph). No §5.6 analogue.

- **State.** Each node holds an LWW register over `(parent, ordering_key)` — its edge to its parent.
- **`tree-move` (kind 8)** LWW-writes `(ref=new_parent, field=ordering_key)` on `target`; latest HLC
  wins per node (as §4.4), exact-tie by encoded value.
- **The one hard problem — cycles.** Concurrent moves can form a cycle (A→under→B while B→under→A). The
  substrate adopts the **deterministic cycle-resolution rule** (Kleppmann's highly-available replicated
  tree): after applying moves in **HLC order**, any move that *would* create a cycle is **skipped** (its
  effect discarded) deterministically on every replica, because every replica applies the same ops in the
  same total order. Skipped moves are recorded as no-ops, never errors; the tree is always acyclic. A
  replica that receives an out-of-order move **re-evaluates** from the affected subtree in HLC order (the
  standard "undo-move / redo in order" procedure). This is the only kind whose apply is **not** a pure
  per-op join — it is a deterministic *replay in HLC order* — and a conformant implementation MUST
  produce the acyclic result the ordered replay defines.
- **Cycle test and which move loses (normative).** Moves are replayed in **ascending HLC order (oldest
  first)**. A move `(node → new_parent)` **would create a cycle** iff `new_parent == node` **or**
  `new_parent` is a descendant of `node` in the tree formed by all **strictly-earlier-HLC** moves already
  applied; a move that would create a cycle is **skipped** (recorded as a no-op). Because each move is
  evaluated against the state of every lower-HLC move, the **later**-HLC move of a colliding pair is the
  one skipped and the **earlier** applied. Concretely, for the canonical concurrent-swap collision —
  `move(A → under B)` at HLC `h1` and `move(B → under A)` at HLC `h2` with `h1 < h2` — replay applies
  `h1` first (A becomes a child of B; no cycle yet), and when `h2` is evaluated A is *already* a
  descendant of B, so moving B under A **would** close the cycle B→A→B and `h2` is skipped. The observable
  result is therefore **`h1` (A under B) applied, `h2` (B under A) skipped**; B keeps its pre-swap parent.
  This is deliberately **not** last-writer-wins for the colliding *pair*: plain LWW (§4.4) governs only
  repeated moves of the **same** node (where the greater HLC does win); the cycle rule governs the
  *interaction* between moves of *different* nodes, and there the ordered replay — not the clock — decides,
  so that no move is silently lost to a numerically larger wall clock and every replica reaches the
  identical acyclic tree regardless of arrival order (the property `SYNC-TREE-01` pins).

### 4.9 Immutable content needs no CRDT

Content-addressed blobs (chunks, `PubManifest`s, MOTEs) are immutable and **need no merge** — they are
referenced *from* CRDT state (an LWW value or an RGA atom MAY carry a content hash), and the bytes are
fetched and self-verified by the Feeds/blob path ([`FEEDS.md`](FEEDS.md), §5.5). Sync moves the *pointers*
deterministically; the content layer moves the *bytes* by content address.

---

## 5. Reconciliation wire protocol

Two replicas reconcile by exchanging ops each lacks. The baseline is the flowstock stateless
version-vector protocol; a **range-based Merkle** mode (grounded in the §5.6 `recon` reference) scales it
to large states. Bodies are deterministic CBOR (§2.2); a JSON edge is permitted only for a
non-interoperating local client. Endpoints are shown as HTTP (the HTTP test,
[`README.md § 4.2`](README.md#42-the-http-test--are-transports-pluggable-with-https-first-class)); the
same three-or-four operations bind equally to a mesh stream (§4.5).

### 5.1 The version vector

A **version vector** is a per-author high-water-mark of applied HLCs, over the subscribed namespaces:

```cddl
VersionVector = { * ik-pub => Hlc }   ; author => the max HLC applied from that author
```

Grounded in flowstock's `SELECT author, MAX(hlc) GROUP BY author`. A vector is not causal-delivery state;
it is a compact summary of "what I already have," used to compute the difference to ship.

**Encoding (normative).** The keys are the authors' raw `ik-pub` **byte strings** — never an ordinal, an
index, or any other stand-in — and the map is deterministic CBOR (§2.2): definite length, entries sorted
**ascending by encoded key bytes** (which, for equal-length `bstr` keys, is ascending by the raw public
key). §2.2's "integer-keyed maps sorted by encoded key" names the common case (`SyncOp`, `Hlc`,
`Snapshot`, COSE headers); the sorting rule is the general RFC 8949 §4.2.1 one and applies to this
`bstr`-keyed map identically. An author absent from the vector means "I hold nothing from this author,"
never "this author has nothing" (§7, absence is not authority).

### 5.2 Endpoints (baseline, grounded in flowstock)

```
GET  /sync/vector                → { node: ik-pub, ns: [tstr], vector: VersionVector }
POST /sync/pull   { vector, ns } → { ops: [ COSE_Sign1(SyncOp) | SyncFrame ] }   ; ops the caller lacks
POST /sync/ops    { ops }        → { applied: u32 }                              ; push ops to the peer
```

- **`GET /sync/vector`** returns the responder's node key, the namespaces it subscribes to, and its
  current `VersionVector`. (flowstock: `{node_id, vector}`.)
- **`POST /sync/pull`** sends the caller's vector (+ requested namespaces); the responder returns, oldest
  HLC first, up to a batch limit, every op it holds whose `hlc` exceeds the caller's vector entry for that
  op's author (or whose author is absent from the vector). (flowstock: `OpsAfter(vector, batch)`.)
- **`POST /sync/ops`** pushes a batch; the responder applies each op that is new (dedup by op content
  hash / `hlc`), verifying signature (§4.1) and CRDT validity (§4) before apply, and returns the count of
  **newly** applied ops. Apply is idempotent: a re-pushed op is a no-op (matching flowstock's
  `INSERT OR IGNORE` oplog-dedup).
- **A round is symmetric** (push-then-pull), so only one side of a pair need be reachable and ops relay
  transitively through any topology (hub-and-spoke, mesh, chain) — the flowstock property. Fetching is
  **read-only and content-addressable at the op level**, so responses are cacheable and a lying responder
  can withhold or stall (detectable as a vector that never advances) but **cannot forge** an op (that
  needs an author key) — the same trustless posture as [`FEEDS.md § 5.1`](FEEDS.md).

### 5.3 Range-based Merkle reconciliation (scalable mode, grounded in §5.6 `recon`)

The baseline `pull` scans all ops after a vector — fine for small states, but O(history) to *find* the
difference when two large replicas differ in only a few ops. The **range-Merkle** mode finds the
difference in O(log n · divergence) by recursively fingerprinting HLC-ordered ranges:

```
POST /sync/fingerprint { ns, ranges: [ { lo: Hlc, hi: Hlc, fp: hash, count: u64 } ] }
   → { mismatched: [ { lo, hi, split: [subrange fingerprints] | ops: [...] } ] }
```

- Each side computes, over the op ids in a range `[lo, hi)`, a **fingerprint** `fp` plus a `count`. Equal
  `(fp, count)` ⇒ the ranges are identical, **no data exchanged**. On mismatch, the range is **split** (by
  op count, into a small fixed fan-out) and the sub-range fingerprints are exchanged recursively; a range
  that shrinks below a threshold ships its ops directly. This is the standard range-based set-
  reconciliation algorithm (Meyer/`recon`), operating over the HLC total order so ranges are canonical on
  both sides.
- **The fingerprint fold (normative, frozen — `SYNC-RECON-01`).** Let `R` be the multiset of op ids whose
  ops have `hlc ∈ [lo, hi)` (an `op-id` is the §4.1 content address `0x1e ‖ BLAKE3-256("DMTAP-SYNC-v0/`
  `op-id" ‖ 0x00 ‖ det_cbor(SyncOp))`). Sort `R` **ascending by HLC** (the §3 total order `(wall, counter,
  author)`; distinct authors never tie, so the order is total and identical on both sides), and
  `fp = 0x1e ‖ BLAKE3-256( "DMTAP-SYNC-v0/recon-fp" ‖ 0x00 ‖ det_cbor([ * op-id ]) )` — one DS-tagged
  BLAKE3 hash **folding** the range's ordered op ids (their raw 33-byte `hash` bstrs) into a single
  32-byte digest, with `count = |R|`. "Fold" here is exactly this collapse-to-one-hash, matching the §5.6
  `recon` reference (`fp = ContentId::of(det_cbor([* id]))` over the range's sorted ids); it is
  **deliberately not** an incremental/homomorphic combiner (XOR- or addition-of-hashes). A homomorphic
  fold buys O(1) range updates but admits cancellation (an even number of identical insertions vanishes)
  and adds an integer-arithmetic corner to the wire — unnecessary here, since a changed range is simply
  re-hashed, and BLAKE3 over the length-prefixed deterministic-CBOR array is collision-resistant and
  unambiguous across a range boundary. The `count` guards the degenerate empty-vs-empty and duplicate
  cases the digest alone could not distinguish. The DS-tag `DMTAP-SYNC-v0/recon-fp` keeps a fingerprint
  from ever colliding with an `op-id`, a `snapshot-state` root, or any other DMTAP hash.
- Range-Merkle is a **discovery optimization only** — every op it surfaces is applied through the same
  §4 verify+merge path; it changes *how the difference is found*, never *what converges*. A node
  advertises it as a `sync-1` sub-token; a peer that lacks it falls back to §5.2 baseline `pull`.

### 5.4 Auth on the wire

The reconciliation transport itself is authenticated per the deployment (§9): a single-owner cluster
gates peers by MLS group membership or a shared bearer secret over a trusted network (the flowstock model:
constant-time bearer compare, fail-closed on an empty secret); a multi-owner deployment gates by an
Identity-authenticated session ([`ROLES.md`](ROLES.md) announce/resolve + a DPoP/WebAuthn-class bound
token, §13). **Transport auth is orthogonal to op auth:** even over a fully trusted transport, every op
MUST carry its own valid COSE signature (§4.1) — the transport gate controls *who may sync*, the op
signature controls *who authored each change*.

---

## 6. Snapshots & compaction (first-class)

A new replica must not have to replay all history, and an old replica must not grow unbounded. Both are
**first-class** here (the §5.6 reference GCs but does not ship signed portable checkpoints; flowstock does
neither and grows unbounded — this section is the substrate's genuinely new contribution on top of both).

### 6.1 Signed snapshots

```cddl
Snapshot = {
  1 => u8,               ; v = 0
  2 => suite,
  3 => tstr,             ; ns          the namespace this snapshot covers
  4 => VersionVector,    ; covers      the exact set of ops folded into `root` (per-author max HLC)
  5 => hash,             ; root        DS-tagged hash of the canonical observable state (§6.1.1)
  6 => ts,
  7 => ik-pub,           ; signer      author key; DeviceCert chains to an admitted IK (§9)
  8 => sig-val,          ; signer over det_cbor(Snapshot ∖ {8}), DS-tag DMTAP-SYNC-v0/snapshot
}
```

- **`root` is a deterministic function of observable state.** The producer serializes the *observable*
  state of the namespace — present OR-Set members, sorted LWW cells, PN-counter totals per object,
  live RGA sequences, the acyclic tree — as the canonical deterministic CBOR of **§6.1.1** and hashes it
  with the DS-tag `DMTAP-SYNC-v0/snapshot-state ‖ 0x00`: `root = 0x1e ‖ BLAKE3-256("DMTAP-SYNC-v0/`
  `snapshot-state" ‖ 0x00 ‖ det_cbor(ObservableState))` (a §18.1.5 v0 `hash`). This generalizes the §5.6
  `ClusterState::snapshot()` canonical form across all six CRDT types. The **snapshot signature** (key
  `8`) uses the distinct DS-tag `DMTAP-SYNC-v0/snapshot` over `det_cbor(Snapshot ∖ {8})` (§18.1.6) — two
  DS-tags, one for the state-root hash preimage and one for the signature preimage, so the two can never
  be confused. Two replicas at the same `covers` vector **MUST** compute the same `root`; a mismatch is
  `ERR_SYNC_SNAPSHOT_ROOT_MISMATCH` (`0x0A09`) and is evidence of divergence (HALT_ALERT).
- **Fast join (the point).** A joining replica fetches a `Snapshot`, adopts its observable state, sets its
  local vector to `covers`, and then pulls **only the ops after `covers`** (§5.2) — it never replays the
  pre-snapshot history. Because `root` is recomputable, a replica that later backfills the pre-snapshot
  ops **MUST** recompute `root` and confirm it matches — a snapshot is **verifiable, not merely trusted**.
- **Trust posture (honest).** A replica that adopts a snapshot *without* backfilling trusts the `signer`
  for the pre-`covers` history until it verifies. A deployment therefore fixes a **snapshot trust
  policy**: (a) *verify-required* — accept a snapshot only from a signer whose `root` the replica can
  recompute from ops it will fetch (self-hosting, high-assurance); or (b) *trusted-checkpoint* — accept a
  snapshot on the signer's authority for bootstrap speed, with the disclosed residual that a malicious
  signer could present a false starting state (bounded, because every op *after* `covers` is still
  independently signed and verified, so divergence surfaces at the next snapshot boundary). The policy
  MUST be explicit; silently trusting an unverifiable snapshot is prohibited (fail-closed governance,
  §10.7).

### 6.1.1 Canonical observable-state schema (normative, frozen — `SYNC-SNAP-01`/`SYNC-SNAP-02`)

`ObservableState` is the single deterministic-CBOR value a replica hashes to produce `Snapshot.root`. It
is a **fixed six-element array**, one section per CRDT type in **`kind`-ascending order**, generalizing
the §5.6 three-array `ClusterState::snapshot()` form (which covered only OR-Set/LWW/death) to all six
types. Positional sections — not a keyed map — are used deliberately, matching the §5.6 reference and
removing any map-key-scheme choice as a source of divergence.

```cddl
ObservableState = [
  orset,   ; §4.3 present OR-Set members
  lww,     ; §4.4 LWW register cells
  pn,      ; §4.6 PN-counter totals
  death,   ; §4.5 death-certificate deleted objects
  rga,     ; §4.7 live RGA sequences
  tree,    ; §4.8 movable-tree parent edges
]

orset = [ * [ target: tstr, element: cv ] ]              ; one entry per PRESENT (target, element);
                                                          ; sorted ASCENDING by det_cbor of the [target,element] pair
lww   = [ * [ target: tstr, field: tstr, value: cv ] ]   ; winning cell per (target,field);
                                                          ; sorted ASCENDING by det_cbor of the triple
pn    = [ * [ target: tstr, field: tstr, total: int ] ]  ; total = Σ_a P[a] − Σ_a N[a] (a signed integer);
                                                          ; sorted ASCENDING by det_cbor of the triple
death = [ * [ target: tstr, class: tstr ] ]              ; one entry per DELETED object (Live contributes nothing);
                                                          ; class ∈ {redact,expires,sensitive}; sorted by det_cbor
rga   = [ * [ target: tstr, atoms: [ * cv ] ] ]          ; per RGA target, the live (non-tombstoned) atom values
                                                          ; in SEQUENCE order (the §4.7 pre-order walk, NOT re-sorted);
                                                          ; outer array sorted ASCENDING by det_cbor(target)
tree  = [ * [ node: tstr, parent: tstr, ord: tstr ] ]    ; each non-root node's winning (parent, ordering_key) after
                                                          ; the §4.8 acyclic replay; sorted ASCENDING by det_cbor of the triple
```

- **Empty sections are the empty array `[]`**, present in position; a section is never omitted, so the
  array is always length 6.
- **Sort keys are byte comparisons of the deterministic-CBOR encoding** of each entry (or, for `rga`, of
  `det_cbor(target)`), so ordering is implementation-independent — the same rule §5.6 already uses
  (`cells.sort_by(|a,b| cbor(a).cmp(&cbor(b)))`).
- **Only *observable* state appears.** OR-Set add-tags/tombstones, the PN-counter's per-author
  `op-id`-keyed delta sets and their derived `P`/`N` aggregates (§4.6),
  RGA element ids and tombstones, `Live` death cells, and superseded LWW cells are all internal — two
  replicas that converge on the *observable* projection produce byte-identical `ObservableState`, hence a
  byte-identical `root`, regardless of internal bookkeeping or apply order. This is the strong-eventual-
  consistency equality the fast-join guarantee (`SYNC-SNAP-02`) rests on: adopting a snapshot + applying
  post-`covers` ops yields the same `ObservableState` bytes as a full replay.
- **Movable-tree root sentinel.** The reserved tree-root node id is the **empty string `""`**; it never
  appears as a `node` entry (the root has no parent edge), only as the `parent` value of a top-level node.

### 6.2 Compaction via stability cut (grounded in §5.6)

Tombstones (OR-Set, RGA) and superseded LWW/death cells can be dropped once **every live replica has
advanced past them**, so no future op can depend on them:

- The **stability cut** is the **minimum, across every live subscribed replica, of that replica's
  max-applied HLC** (folding in the local max). A replica publishes its watermark as a signed
  `StabilityMark { author, hlc }` (§5.6). A **stale** replica — not seen within the liveness window
  (§16-class) — is **excluded** from the min, so a dead-but-unrevoked replica cannot stall compaction; a
  live replica with *no known* watermark yields **no cut** (fail-closed: never GC on incomplete
  knowledge).
- Below the cut: an OR-Set add-tag present in **both** adds and tombstones is dropped (it can never again
  affect presence, since its `{author, hlc}` is globally unique); an RGA atom that is tombstoned and below
  the cut is dropped; a superseded LWW/death cell below the cut needs no history. Compaction **never
  changes observable state** — it only discards proof-obligations no live replica can still raise.
- **Op-log truncation.** Once a snapshot at vector `V` exists and every live replica has advanced past
  `V`, the op-log prefix below `V` MAY be truncated; the retained suffix plus the snapshot reconstruct the
  state. (The §5.6 journal `truncate_before` with the same "every member replayed past it" obligation.)

---

## 7. Sparse / partial sync (namespaces)

A replica need not hold everything. **Sparse sync** lets a replica subscribe to a subset of namespaces —
a phone that syncs only "recent" while the home box holds "all," or a shop terminal that syncs only its
own branch.

- Every op carries `ns` (§4.1). A replica **subscribes** to a set of namespaces and advertises it in
  `GET /sync/vector` (`ns: [tstr]`). `pull`/`fingerprint`/`ops` are **scoped to the intersection** of the
  two peers' subscriptions; a namespace the caller did not subscribe to is never shipped to it.
- The `VersionVector`, snapshots, and stability cut are all **per-namespace**: convergence, anti-rollback,
  and compaction are sound *within* each namespace independently. A replica that later widens its
  subscription bootstraps the new namespace via its snapshot + post-snapshot ops (§6).
- **Causal soundness across the boundary (normative).** An op MUST be causally self-contained within its
  namespace: an RGA `ref` or tree `parent` MUST name a `target` in the **same** `ns`, and a
  cross-namespace reference is `ERR_SYNC_NS_LEAK` (`0x0A0A`). This keeps a sparse subscriber from needing
  ops outside its subscription to converge its own namespaces — the property that makes partial sync
  correct rather than merely partial.
- **Absence is not authority.** A replica not subscribed to a namespace makes **no** assertion about it
  (the capability-absence rule, §21.22): its silence is never read as "empty" or "deleted."

---

## 8. Multi-author capability (the generalization over §5.6)

§5.6 authorizes an op by **ambient MLS group membership** — every writer is a device of one owner, and
the op rides unsigned inside the encrypted group. The substrate authorizes an op by its **own COSE
signature** (§4.1), which decouples authorship from any shared secret and yields three deployment shapes
from one spec:

1. **Single-owner device cluster** (= §5.6). Every author key chains to the **same** `IK`; the admitted
   author set is exactly that identity's non-revoked `DeviceCert`s (§1.2). Evicting a device (`IK`-signed
   rotation, §1.5) drops it from the set and rejects its future ops — the §5.6 healing path.
2. **Closed multi-owner set.** Author keys chain to **different** `IK`s admitted by an explicit **member
   set** (an OR-Set of admitted `IK`s, itself a synced object governed by an admin-capability, §13.5) —
   a shared workspace of several identities.
3. **Open / permissionless namespace.** Any Identity may author; the replica applies an **admission
   policy** (rate/quota/reputation) rather than a membership gate, exactly as a Feeds holder does
   ([`FEEDS.md § 7`](FEEDS.md)). Conflict resolution is unchanged — determinism comes from the CRDT
   algebra and the HLC total order, not from who is allowed to write.

In all three, the **authorization check is the same**: an op is applied iff its signature verifies under
`hlc.author` (§4.1) **and** that author is admitted by the namespace's policy — `ERR_SYNC_AUTHOR_
UNAUTHORIZED` (`0x0A01`) otherwise. Because the author key is the HLC tiebreak (§3), determinism holds
across any author set: distinct authors never tie, so the merged state is identical everywhere regardless
of how many identities wrote it.

**Deniable content is barred from sync state (inherited, §5.6).** A `SyncOp` value MUST NOT embed a
deniable payload: durable, signed, replicated history is the opposite of repudiable, so a value that
carries deniable material is rejected (`ERR_SYNC_OP_INVALID`, `0x0A03`) — the §5.6 `embeds_deniable`
rule, carried forward.

---

## 9. Authorization and admission (summary)

| Deployment | Admitted authors | Gate | Eviction |
|------------|------------------|------|----------|
| Device cluster (§5.6) | one `IK`'s `DeviceCert`s | MLS membership + op signature | `IK`-signed device rotation (§1.5) |
| Closed multi-owner | a synced member-set of `IK`s | member-set + op signature | admin removes `IK` from member-set (§13.5) |
| Open namespace | any `IK` | admission policy (rate/quota) + op signature | policy deny (`0x0A0B`), never a takedown |

Op signature verification (§4.1) is **mandatory in every row** — the gate differs, the per-op
authenticity does not.

---

## 10. Conformance vectors

These follow the existing suite style ([`../conformance/`](../conformance/), §10, §22 vectors): a
known-answer test with byte-exact inputs/outputs where the wire encoding is fully determined by this
document's text alone. **All 20 are now byte-exact**, generated by
[`../conformance/vectors/gen_sync_vectors.py`](../conformance/vectors/gen_sync_vectors.py) into
[`../conformance/vectors/sync_vectors.json`](../conformance/vectors/sync_vectors.json) — the same
throwaway-script provenance model as [`pub_vectors.json`](../conformance/vectors/pub_vectors.json): every
value is a direct, mechanical application of §3/§4/§5/§6/§7 to fixed inputs, no randomness — Ed25519 is
deterministic, so even the one signature vector is a reproducible known answer. An **independent Rust
implementation** of this document now exists and executes these vectors; it is what surfaced the §14
corrections C-01…C-04, and the vectors below are the ones it must reproduce byte-for-byte. Generation
stays in the script, so the vectors remain an independent check on any implementation rather than a
restatement of one.

Five of these were previously **NOT-FROZEN** stubs: `SYNC-OP-02` (the `COSE_Sign1` envelope framing),
`SYNC-TREE-01` (an outright contradiction between the stub's expectation and §4.8's replay algorithm),
`SYNC-SNAP-01`/`SYNC-SNAP-02` (the canonical observable-state schema) and `SYNC-RECON-01` (the
fingerprint fold). Each was resolved by **this document deciding first** — §4.1, §4.8, §6.1.1 and §5.3
now carry the normative frozen text together with the reasoning for the choice — and only then vectored.
Nothing below is frozen *by* the generator script: a vector may only mechanically apply a decision the
specification already states.

| # | Name | Construction | Expected | Status |
|---|------|--------------|----------|--------|
| `SYNC-OP-01` | Op canonical encoding | `det_cbor(SyncOp{kind:3, ns:"", target:"a", field:"x", value:"v", hlc})` | byte-exact CBOR; re-decode round-trips; non-canonical input rejected | **Frozen** — `sync_op_lww_canonical` |
| `SYNC-OP-02` | Op signature bind | `COSE_Sign1` over `SYNC-OP-01` under author key K, DS-tag `DMTAP-SYNC-v0/op` | verifies under K; a flipped byte ⇒ `0x0A02` | **Frozen** — `sync_op_cose_sign1_bind`. §4.1 now pins the full `COSE_Sign1` profile: `protected = {1: alg=-8 EdDSA, 4: kid=author}`, `unprotected = {}`, `payload = det_cbor(SyncOp)`, and the signable preimage = the RFC 9052 `Sig_structure ["Signature1", protected, external_aad=DMTAP-SYNC-v0/op‖0x00, payload]`. The DS-tag rides in `external_aad` (the RFC-9052 domain-separation slot, consistent with §22/§18.1.6). Verifies under K; a flipped payload byte or substituted `kid` ⇒ `0x0A02` |
| `SYNC-AUTH-01` | Unauthorized author | op signed by K′ whose `DeviceCert` is absent/revoked from the member-set | reject `0x0A01` | **Frozen** — `sync_author_unauthorized` (the admission predicate only, independent of `SYNC-OP-02`'s open envelope question) |
| `SYNC-LWW-01` | LWW HLC winner | two `lww-set` on `(a,x)` with HLC h1<h2 | value of h2, on any apply order | **Frozen** — `sync_lww_hlc_winner` |
| `SYNC-LWW-02` | LWW exact-tie determinism | two `lww-set`, identical HLC, values `"m"`/`"n"` | larger `det_cbor(value)` wins, identical on both replicas | **Frozen** — `sync_lww_exact_tie` |
| `SYNC-ORSET-01` | Add-wins | concurrent `set-add(e)` and `set-remove(e)` where remove's `observed` omits the concurrent add | `e` present | **Frozen** — `sync_orset_add_wins` |
| `SYNC-ORSET-02` | Future-add remove rejected | `set-remove` citing an add-tag with HLC > remove's HLC | reject `0x0A03` | **Frozen** — `sync_orset_future_add_remove_rejected` |
| `SYNC-DEATH-01` | Remove-wins domination | `death(redact)` at h1, concurrent `set-add` at h2>h1 | object **absent** (death dominates regardless of h2) | **Frozen** — `sync_death_domination` |
| `SYNC-DEATH-02` | Tie fail-safe | `death` and `live` at identical HLC | `Deleted` wins (fail-safe) | **Frozen** — `sync_death_tie_failsafe` |
| `SYNC-PN-01` | Counter convergence | authors a,b: `+5(a)`, `−2(b)`, **true replay** of `+5(a)` (byte-identical op ⇒ identical `op-id`) | `P[a]=5`, `N[b]=2`, total `3`; the replay is a no-op (union of `op-id`-keyed deltas) | **Frozen** — `sync_pn_counter_convergence`. **Corrected (§14):** the merge is the §4.6 per-author **union of `op-id`-keyed deltas**, not per-author `max`; and the vector's third op previously carried `hlc.counter=1` where the first carried `0`, making it a *distinct* op (different `det_cbor` ⇒ different `op-id`) whose two `+5` deltas correctly sum to `P[a]=10`, total `8` — contradicting the vector's own stated expectation. The third op now carries the **same** HLC as the first, so it is genuinely the same op and dedup by `op-id` (§5.2) makes it a no-op, reproducing `total=3` |
| `SYNC-PN-02` | Foreign-entry reject | op from `a` mutating `P[b]` | reject `0x0A06` | **Frozen** — `sync_pn_counter_foreign_reject` (declarative author/entry-author fields; the wire shape of a *malformed* op is implementation-internal, not spec-given) |
| `SYNC-RGA-01` | Concurrent insert order | two `seq-insert` same origin, ids h1<h2 | order = [h2, h1] (newer-first), identical on both replicas | **Frozen** — `sync_rga_concurrent_sibling_order` |
| `SYNC-RGA-02` | Insert after tombstone | `seq-remove(x)` then concurrent `seq-insert` (`"Z"`) with `ref=x` | insert resolves; atom order **including** tombstones is `["x(tombstoned)", "Z"]` (Z sorts *after* its left-origin x); visible sequence `["Z"]` | **Frozen** — `sync_rga_insert_after_tombstone`. **Corrected (§14):** the vector's `atom_order_incl_tombstones` array previously listed `["Z", "x(tombstoned)"]`, contradicting both §4.7 (`seq-insert` places an atom **after** its left-origin `ref`) and the vector's own note. The **note was right**; the array was wrong and is now `["x(tombstoned)", "Z"]`. The array is a **human-readable label list**, not normative bytes — the normative artifacts of this vector are the three op encodings and the visible sequence; the label list pins order and membership only |
| `SYNC-TREE-01` | Concurrent-move cycle | `move(A→B)` at `h1` and `move(B→A)` at `h2`, `h1<h2`, HLC-ordered replay | **earlier**-HLC move (`h1`, A under B) applied, **later** (`h2`, B under A) skipped; tree acyclic; identical on both | **Frozen** — `sync_tree_concurrent_move_cycle`. The contradiction is **resolved in §4.8**: replaying oldest-first, `h1` applies first (A becomes a child of B) and `h2` then *would* close the cycle B→A→B, so the **later** move is the one skipped. The stub's prior expected text ("later-HLC move applied, earlier skipped") was the **erroneous side** and is corrected here; §4.8's ordered-replay algorithm was correct and now states the outcome explicitly. This is Kleppmann's result and is **not** LWW for the colliding pair (LWW governs only repeated moves of the *same* node) |
| `SYNC-SNAP-01` | Snapshot root determinism | two replicas at the same `covers` vector | identical `root`; mismatch ⇒ `0x0A09` | **Frozen** — `sync_snapshot_root_determinism`. §6.1.1 now pins the canonical `ObservableState`: a fixed six-element positional array (one section per CRDT kind, `orset`/`lww`/`pn`/`death`/`rga`/`tree`), each a section sorted by `det_cbor` of its entries (RGA inner order is sequence order, not re-sorted), and `root = 0x1e ‖ BLAKE3-256("DMTAP-SYNC-v0/snapshot-state" ‖ 0x00 ‖ det_cbor(ObservableState))`. Two replicas at the same `covers` compute identical `root`; mismatch ⇒ `0x0A09` |
| `SYNC-SNAP-02` | Fast-join equals replay | join via snapshot+post-ops vs full replay | byte-identical observable state | **Frozen** — `sync_snapshot_fast_join_equals_replay`. Same §6.1.1 schema: snapshot-adopt + post-`covers` ops yields byte-identical `ObservableState` (hence identical `root`) to a full replay, because only the observable projection is serialized. **Corrected (§14):** the vector's `snapshot_covers_cbor_hex` previously encoded an **integer**-keyed map `{1: Hlc, 2: Hlc}`, contradicting §5.1's `VersionVector = { * ik-pub => Hlc }` — the keys are the authors' 32-byte `ik-pub` **byte strings**, ordered by §2.2 canonical map rules (ascending by encoded key bytes). No expectation exercised it, so nothing failed; it is fixed so no implementer is misled |
| `SYNC-RECON-01` | Range-Merkle finds diff | two replicas differing in 1 op, range-fingerprint round | exactly the 1 differing op surfaced; equal ranges exchange no ops | **Frozen** — `sync_recon_range_merkle_diff`. §5.3 now pins the fold: `fp = 0x1e ‖ BLAKE3-256("DMTAP-SYNC-v0/recon-fp" ‖ 0x00 ‖ det_cbor([* op-id]))` over the range's op ids in ascending-HLC order, plus `count` — a single DS-tagged BLAKE3 hash (matching the §5.6 `recon` reference), **not** a homomorphic combiner. Equal `(fp,count)` ⇒ range identical, no ops exchanged; the one differing op is surfaced by drill-down |
| `SYNC-NS-01` | Sparse scoping | caller subscribes `{x}`, responder holds `{x,y}` | only `ns=x` ops shipped | **Frozen** — `sync_ns_sparse_scoping` |
| `SYNC-NS-02` | Cross-namespace ref rejected | RGA `ref` naming a target in another `ns` | reject `0x0A0A` | **Frozen** — `sync_ns_cross_namespace_ref_rejected` |
| `SYNC-GC-01` | Stability-cut safety | tombstone below the cut dropped; a stale replica excluded from the cut | observable state unchanged; live replica with no watermark ⇒ no GC | **Frozen** — `sync_gc_stability_cut` |

---

## 11. Security considerations / honest limits

1. **Determinism depends on canonical encoding.** If an implementation emits non-canonical CBOR or a
   non-`ext-value` value, two replicas can diverge. This is why §2.2 is a MUST and every value is
   restricted to `ext-value`; a violating op is rejected (`0x0A03`), never merged on a guess.
2. **A signing-key compromise can author under the identity.** An op is only as authentic as its author
   key and `DeviceCert` chain (§4.1). A compromised operational key can write ops until revoked (§1.5);
   the blast radius is the ops it could author in the interval, bounded by eviction (§8) — the ordinary
   endpoint residual (§6.6 item 3), now with the caveat that replicated CRDT history is durable, so a
   malicious write must be **superseded by a later op** (a compensating `lww-set`/`death`/`seq-remove`),
   not "deleted."
3. **Snapshots trade replay for trust unless verified.** A trusted-checkpoint snapshot (§6.1) trusts the
   signer for pre-`covers` history until backfilled. The deployment policy MUST be explicit; the residual
   is bounded because every post-`covers` op is independently verified and the next snapshot boundary
   surfaces divergence (`0x0A09`).
4. **Stability-cut liveness is a policy tradeoff.** Excluding a stale replica from the cut lets
   compaction proceed, but if that replica returns *after* GC it may lack a tombstone it needed — it then
   heals by snapshot bootstrap (§6.1), not by replaying dropped tombstones. This is the standard CRDT GC
   tradeoff (bounded storage vs. arbitrarily-late rejoin), disclosed, not hidden.
5. **The movable tree is the one non-join kind.** Its convergence relies on deterministic HLC-ordered
   replay (§4.8), not a pure per-op merge; an implementation that applies tree-moves out of order without
   the re-evaluation step will diverge. The conformance vector `SYNC-TREE-01` exists to catch exactly
   this.
6. **Metadata: authorship and timing are visible to co-authors.** Sync is not sealed-sender — every op
   carries its author and HLC, visible to every replica in the namespace (by design: multi-author
   convergence needs attributable ops). A product needing author-blind convergence is out of scope; that
   is a different problem than the one solved here.

---

## 12. Fail-closed rules this capability contributes (proposed `0x0A` block)

Registered additively under §21.14; mirrored into the §10.7 auditable set. The owning clause governs.

| Code | Name | Trigger | Action |
|------|------|---------|--------|
| `0x0A01` | `ERR_SYNC_AUTHOR_UNAUTHORIZED` | op author not admitted by the namespace policy (§8, §9) | FAIL_CLOSED_BLOCK |
| `0x0A02` | `ERR_SYNC_OP_SIG_INVALID` | `COSE_Sign1` fails under `hlc.author` / broken `DeviceCert` chain (§4.1) | FAIL_CLOSED_BLOCK |
| `0x0A03` | `ERR_SYNC_OP_INVALID` | non-`ext-value` value, future-add remove, embedded deniable payload, malformed op (§4) | FAIL_CLOSED_BLOCK |
| `0x0A04` | `ERR_SYNC_UNSUPPORTED_VERSION` | op/snapshot carries a `v`/`suite` unsupported (§4.1, §6.1) | FAIL_CLOSED_BLOCK — never guess |
| `0x0A05` | `ERR_SYNC_HLC_SKEW` | op `wall` outside the skew window (§3) | FAIL_CLOSED_BLOCK |
| `0x0A06` | `ERR_SYNC_COUNTER_FOREIGN` | a PN-counter op mutates another author's P/N entry (§4.6) | FAIL_CLOSED_BLOCK |
| `0x0A07` | `ERR_SYNC_SEQ_ORIGIN_MISSING` | RGA insert origin absent and the causal buffer overflows (§4.7) | DEFER_REQUESTS then ROTATE_RETRY |
| `0x0A08` | `ERR_SYNC_FRAME_CHAIN_BROKEN` | a `SyncFrame` op's `ref` back-link does not resolve to its predecessor (§4.1) | HALT_ALERT — publish conflicting frames as evidence |
| `0x0A09` | `ERR_SYNC_SNAPSHOT_ROOT_MISMATCH` | recomputed observable-state root ≠ `Snapshot.root` at the same `covers` (§6.1) | HALT_ALERT — divergence evidence |
| `0x0A0A` | `ERR_SYNC_NS_LEAK` | an op references a `target` in a different namespace (§7) | FAIL_CLOSED_BLOCK |
| `0x0A0B` | `ERR_SYNC_ADMISSION_QUOTA` | an open-namespace admission limit (rate/quota) exceeded (§9) | DENY_POLICY — a policy deny, never a security gate, never a silent hole |

The §10.7.5 governing rule applies unchanged: a sync security-relevant failure is either refused (fail
closed) or surfaced as an explicit choice, never a silent degradation. New sync fail-closed rules MUST be
mirrored into §10.7.

---

## 13. Grounding

- **CRDT semantics** — `dmtap-clustersync` (§5.6 reference, `/Users/pc/code/vulos/envoir/crates/dmtap-clustersync`):
  OR-Set add-wins, LWW-by-HLC with encoded-value tiebreak, remove-wins death-certificate with the D3
  domination invariant, HLC total order `(wall, counter, author)`, stability-cut GC, hash-chained
  journal. The PN-counter, RGA sequence, and movable tree are **new** here (standard CRDT constructions:
  the PN-counter in its **op/delta** form joined by union of `op-id`-keyed deltas — *not* the textbook
  state-based per-author-`max` form, which is unsound for delta-carrying ops, §4.6 and §14; Roh et al.
  RGA; Kleppmann highly-available replicated tree), added to complete the algebra.
- **Wire protocol** — flowstock stateless sync (`/Users/pc/code/vulos/flowstock`): `GET /sync/vector`,
  `POST /sync/pull`, `POST /sync/ops`, per-author `MAX(hlc)` version vector, symmetric push-then-pull
  round, idempotent oplog dedup, LWW-by-HLC + set-union movements, constant-time bearer auth fail-closed
  on empty secret. The substrate upgrades flowstock's JSON to deterministic CBOR and its trusted-network
  bearer auth to per-op COSE signatures for the multi-author case.
- **Range-based reconciliation** — the §5.6 `recon` module and the standard range-based set-reconciliation
  algorithm, operating over the HLC total order.
- **Snapshots & sparse sync** — new to the substrate; grounded in the §5.6 canonical-snapshot form and
  stability-cut, generalized to signed portable checkpoints and per-namespace scoping.

Independent implementations MUST be buildable from this document and [`IDENTITY.md`](IDENTITY.md) alone
(the flowstock test); the reference code above is an existence proof, not the standard, and where it and
this document disagree, this document governs.

---

## 14. Change log — normative corrections

This document is pre-1.0 and is corrected in the open: a defect found by an implementation is fixed
here **and recorded here**, never silently edited. Each entry states what changed, whether it changes
**normative merge semantics** (an implementation MUST be updated) or only a vector/editorial artifact,
and how it was found.

| # | Change | Class | Found by |
|---|--------|-------|----------|
| **C-01** | **§4.6 PN-counter merge: per-author `max` of `P`/`N` → per-author UNION of `op-id`-keyed deltas.** The `max` join is sound only for *state*-based counters where each replica's `P[a]` covers `a`'s whole op prefix; §4.2 kind 5 carries a **delta**, so replicas holding different *subsets* of one author's deltas merged to the larger subtotal and **silently lost the rest** — a lost-write soundness bug, and non-associative (`(A⊔B)⊔C ≠ A⊔(B⊔C)`). The state is now `D[a]: op-id → delta` with `P`/`N` derived; merge is set union over `(op-id, delta)`, which is commutative, associative, and idempotent. The two rules **agree exactly** whenever both replicas hold complete op sets, so this tightens the intended semantics rather than replacing them; §4.6's compaction note reinstates `max` for the below-stability-cut aggregate, the one place completeness is guaranteed. | **NORMATIVE — merge semantics.** An implementation that joins PN-counters by per-author `max` over deltas is **non-conformant** and can lose writes. | A property test in an independent Rust implementation of this document (envoir `dmtap-sync`), not by review — the reason C-01 is recorded loudly. |
| **C-02** | **`SYNC-PN-01` vector corrected.** Its third op was described as "a replay of author A's own contribution" but carried `hlc.counter=1` where the first carried `0` — a different `det_cbor`, hence a different `op-id`, hence a **distinct** op whose delta §4.6 correctly accumulates (`P[A]=10`, total `8`), contradicting the vector's own expected `P[A]=5`, total `3`. The third op now carries the **same** HLC as the first, so it is genuinely the same op and is deduped by `op-id` (§5.2). | Vector fix; the expectation (`total=3`) was already right, the input was not. | Same implementation: its runner applied the true-replay variant and reproduced the vector's stated expectation exactly. |
| **C-03** | **`SYNC-RGA-02` vector corrected.** Its note ("Z sorts immediately after x's tombstoned position" — the §4.7 insert-after rule) contradicted its `atom_order_incl_tombstones` array `["Z", "x(tombstoned)"]`. §4.7 governs: the array is now `["x(tombstoned)", "Z"]`. The array is clarified as a **human-readable label list**, not normative bytes. | Vector fix; §4.7 unchanged. | Implementation followed §4.7 and diverged from the array. |
| **C-04** | **`SYNC-SNAP-02` vector corrected.** `snapshot_covers_cbor_hex` encoded an integer-keyed map `{1: Hlc, 2: Hlc}` where §5.1 specifies `VersionVector = { * ik-pub => Hlc }` (32-byte `bstr` keys). Re-encoded with `ik-pub` keys in canonical order; §5.1 now states the encoding rule explicitly (RFC 8949 §4.2.1 sorting applies to `bstr`-keyed maps just as to integer-keyed ones). | Vector fix, latent — no expectation exercised the field, so nothing failed. | Read-through during the C-01–C-03 fixes. |

**Standing rule.** A defect between this document and a conformance vector is resolved by deciding
**which side is right on the merits** and correcting the other **in the open** (the §10 discipline: a
vector may only mechanically apply a decision this document already states). Where the defect is in this
document's *semantics* — as C-01 was — the correction is recorded as a normative change with its class
stated, so an implementer can tell at a glance whether they must change code.
