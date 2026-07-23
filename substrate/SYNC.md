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
   byte-identically. No merge ever depends on wall-clock accuracy or on arrival order. **"Larger" is pure
   bytewise (lexicographic) comparison of the two encoded byte strings — never a length-first comparison**
   (some canonical-CBOR profiles sort *map keys* by length-then-bytes; this is a value tiebreak, not a
   map-key sort, and the two orders disagree on real inputs — e.g. the one-byte string `0x42` compares
   **greater than** the two-byte string `0x41 0x41` under bytewise comparison, first byte deciding, but
   **less than** it under a length-first rule, since the shorter string would sort first regardless of
   content). Concretely: compare byte-by-byte from the start; the first differing byte decides; if one
   string is a strict prefix of the other, the **shorter** (the prefix) is the lesser of the two, so the
   longer wins only via that prefix case, never via length alone. Every place this document says a byte
   string is "larger" (here, §4.4, §6.1.2's table) means exactly this comparison.

### 2.3 The recurring defect class: a maximum is not a completeness cut (normative framing)

Several fields in this document answer *"has this replica seen everything up to some point?"* — a `pull`
predicate, a snapshot's `covers`, a stability cut — and each needs a **completeness watermark**, not a
**maximum**: a maximum is the greatest value ever seen, with no memory of gaps below it, and the two
coincide only when delivery has no gaps, which this document's own model (§4.6) explicitly does not
guarantee. §5.1 draws this line normatively as `contiguous_below` vs. `max_applied` (§14 C-15); apply the
same distinction wherever a "latest"/"maximum" value is used to decide what a peer may safely assume it
already has.

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
- **Skew bound (fail-closed, one-sided — corrected, §14 C-17).** An op whose `wall` is **more than the
  profile skew window** (§16-class value, default **120 s**, from §5.6 `HLC_SKEW_MS`) **ahead of** the
  receiver's clock is rejected (`ERR_SYNC_HLC_SKEW`, `0x0A05`). This is deliberately **one-sided**: it
  bounds how far a malicious or misconfigured author can push ordering **into the future** — the tiebreak
  budget an author would otherwise spend to win every LWW race, and the only direction a fabricated `wall`
  can attack (§3's total order and §16.10). **A past-dated `wall`, however old, is never grounds for
  rejection on skew grounds alone.** The grounding this bound is drawn from (§5.6.4: "MUST NOT be accepted
  more than the clock-skew tolerance ahead of the receiver's clock") is itself one-sided, and a
  two-sided reading — rejecting an op whose `wall` is far in the **past** — directly contradicts §2.2's
  "no merge ever depends on wall-clock accuracy or on arrival order": an ordinary terminal that was offline
  for longer than the skew window, then reconnects and pushes the ops it minted while offline, produces
  exactly such a past-dated `wall` on every one of them, and a two-sided check would refuse the entire
  backlog of legitimate offline edits this document otherwise goes out of its way to support (§1, §11
  item 1). A deployment wanting to bound how *stale* an op may be accepted (a distinct, policy-level
  concern — data retention, not clock forgery) MUST implement it as a separate, disclosed check, never by
  widening this one.
- **Counter overflow is a spill, not a wrap (fail-safe, MUST — corrected, §14 C-25).** Both `Tick` (local
  mint, above) and `Observe` (remote fold, above) MUST treat `counter` reaching its `u32` maximum
  (`0xFFFFFFFF`) as an overflow: the **next** increment MUST instead advance `wall` by one and reset
  `counter` to `0` — the same branch the ordinary `now > wall` case already takes. This holds whether the
  overflow is produced by a **local** tick or by `Observe` folding a **remote** HLC forward: a remote op
  carrying `counter = 0xFFFFFFFF` MUST cause the receiver's own next tick to land at `(wall+1, 0)`, never
  at `(wall, 0)`. The latter is a **wrap**, not a spill, and it sorts **before** the very op that caused
  it in the §3 total order — retroactively inverting causal order (an LWW tie-break, a §4.8 replay
  position, a version-vector advancement) for as long as any op minted at the wrapped position survives.
  Because `Observe` folds a peer's counter forward **unconditionally**, this is **remotely triggerable** —
  any peer, accidentally or by deliberately probing for an ordering-inversion attack, can send
  `counter = 0xFFFFFFFF` — and MUST NOT be left to each implementation's integer-overflow behaviour: an
  engine whose counter silently wraps is non-conformant. (The Grounding note below states the same spill
  for the non-normative *string* encoding as a SHOULD; this bullet is the binding rule for the normative
  CBOR form itself, and is a MUST for exactly the reason the string-form note is only a SHOULD there but
  not here — this is the wire type every implementation actually verifies signatures and orders against.)

> **Grounding.** The flowstock reference encodes the HLC as a lexically-sortable string
> `"{ms:013d}-{counter:04x}-{author}"`; the §5.6 reference encodes it as an integer-keyed CBOR map.
> The substrate normative encoding is the CBOR map above (deterministic CBOR is the substrate primitive,
> §2.2); a string form is an equivalent client-edge convenience and MUST canonicalize to the same
> `(wall, counter, author)` order — fixed field widths with any `counter` overflow spilled into `wall`
> (never wrapped, per the MUST above) and an order-preserving `author` encoding (fixed-width, single-case
> hex, since the comparison is over raw `ik-pub` bytes) — or it is not this HLC.

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

- `value` is **exactly the §18.3.6 `ext-value` type**, no narrower:

  ```cddl
  ext-value = bool / int / bytes / tstr / [* ext-value] / { * tstr => ext-value }
  ```

  — booleans, unsigned/negative integers, byte strings, text strings, **arrays of `ext-value`
  (heterogeneous permitted), and text-keyed maps of `ext-value`, both recursively**. Excluded are
  **floats, tags, `null`/`undefined`, and integer-keyed maps** (`ext-value` has no integer-keyed-map
  arm), so a value can never smuggle an un-canonicalizable or ambiguous encoding (the §5.6
  `is_ext_value` rule). A violating op is rejected (`ERR_SYNC_OP_INVALID`, `0x0A03`).
  - **Nesting is deterministic-safe, and already load-bearing elsewhere.** A text-keyed map is
    canonicalized by §2.2 exactly as an integer-keyed one is — keys sorted ascending by *encoded key
    bytes*, duplicates forbidden, definite lengths — applied recursively at every depth. This is not a
    new risk being taken here: §18.3.6's `Headers.ext` carries the same recursive type **inside a
    signature preimage** on every MOTE, so the encoding has to be reproducible already. A decoder MUST
    validate `ext-value` recursively and MUST reject at the first violating node rather than
    canonicalizing on a guess.
  - **Depth is bounded by the decoder, and the ceiling is REQUIRED (§14 C-14).** `ext-value` is
    defined recursively and therefore places no depth limit of its own. A decoder MUST enforce a
    container-nesting ceiling of **64** levels (`SYNC_MAX_NESTING_DEPTH`; the outermost value is
    depth 0 and each nested array/map adds one) while decoding **any** sync
    object — a `SyncOp` and its `value`, a `PullResponse`, a `SnapshotBody`, a `fingerprint`
    request — and MUST reject an over-deep item as `ERR_SYNC_OP_INVALID` (`0x0A03`). The bound MUST
    be checked **before** recursing, so a hostile deeply-nested item is a *refusal* rather than a
    stack exhaustion; "the decoder happened to survive it" is not conformance. The number is fixed
    rather than left to the implementation for the ordinary interop reason: a ceiling chosen
    per-implementation lets one encoder mint a value a second decoder refuses, which is divergence
    by rejection (§4.1.2) with no error visible at the minting end. 64 is the same ceiling DMTAP's
    deterministic-CBOR decoder applies to every other object, so a sync value is bounded exactly as
    a MOTE header is. It is a **fixed** constant, not a §16-class deployment parameter, for the same
    reason: a tunable ceiling is a per-deployment accept set, and two deployments that sync are one
    namespace. A value's *encoded size* is independently bounded by the op-size limit
    (§16-class); the depth ceiling is not a substitute for it, nor it for the depth ceiling — a
    small input can be deeply nested.
  - **The empty map `{}` is an `ext-value`, and MUST be accepted (§14 C-14).** `{ * tstr =>
    ext-value }` admits zero entries, so `{}` — which encodes as `0xa0` **regardless of what key
    type its entries would have had** — is a legal value. The ambiguity is real but **vacuous**: a
    decoder cannot tell an empty text-keyed map from an empty integer-keyed map, because there are
    no keys to tell apart, and there is correspondingly nothing that could be smuggled through
    them. A decoder MUST therefore treat `0xa0` as the empty text-keyed map and accept it. Refusing
    it — the reflex of an implementation that rejects integer-keyed maps by rejecting "maps whose
    key type it could not confirm" — would make a legal `ext-value` un-decodable, and would do so
    asymmetrically across a deployment. The empty array `[]` (`0x80`) is accepted on the same
    reasoning and has no ambiguity at all. Note that this is a statement about the *empty* map
    only: a map with **any** entry has a determinable key type and an integer-keyed one is
    rejected at that entry, recursively, per the bullet above.
  - **This corrects a narrowing (§14 C-08).** Earlier text of this bullet described `ext-value` as
    "text, byte strings, integers, booleans and **homogeneous arrays** thereof," silently dropping
    §18.3.6's map arm and adding a homogeneity constraint §18.3.6 does not have — while still calling
    the result "the `ext-value` subset (§18.3.6)". Two types under one name is exactly the defect that
    makes a consumer's data un-encodable for no reason; see §4.1.1 for what nesting does and does not
    buy.

### 4.1.1 Value shape — nesting, opacity, and where the merge boundary is (normative guidance)

`ext-value` can now express a nested application object. **It still does not make that object mergeable.**
This subsection exists because those two facts are easy to conflate, and conflating them costs writes.

**The merge unit is the whole `value`.** §4.4 resolves an LWW register by comparing the two ops' HLCs and
keeping the winner's `value` *entire*. Nothing in §4 descends into a value: there is no per-key merge of
two nested maps, no per-element merge of two arrays. A nested `{x: 10, y: 20}` and a concurrent
`{x: 10, y: 99}` do not merge to `{x: 10, y: 99}` by field — one op wins and the other is discarded whole,
identically to two opaque strings. Structure inside a value is **representation**, not semantics.

**Granularity lives in the address space.** The substrate's nesting mechanism is `(ns, target, field)`,
not the value. A product that wants two authors to edit *different parts of the same object*
concurrently without clobbering each other MUST decompose those parts into distinct addresses — one
register per independently-editable leaf — because that is the only decomposition the algebra sees:

```
  slide:s1 / obj:shape-a  →  the shape's state        ; concurrent edits to shape-a and shape-b
  slide:s1 / obj:shape-b  →  the other shape's state  ; both survive: different registers
  slide:s1 / s:title      →  a scalar
```

Collapsing that into a single register holding one big nested map is a *regression* in merge behaviour,
and one the encoder will not warn you about: it converts "both edits survive" into "last writer wins the
whole slide". **Choose the address decomposition to match the concurrency you want to survive; then the
value shape is free.**

**So which representation, when.** Both are conformant; pick on this axis:

| Use | When | Cost |
|---|---|---|
| **Native nested `ext-value`** | The object is genuinely edited as a unit, *and* you want the protocol to canonicalize it, hash it reproducibly, and let a generic tool inspect it without product knowledge. | Encoder must validate recursively; slightly larger validator surface. |
| **Opaque payload** (`tstr` of JSON, or `bstr` of any format) | The value has a shape the protocol should not model at all — a foreign format, a versioned application struct with its own evolution rules, a pre-existing serialization you must round-trip byte-exactly. | **You own canonicalization.** See the obligation below. |

Neither is "the CRDT-native one": there is no such thing at this boundary, because the boundary does not
merge. **Prefer native nesting when the content is yours and structured**, because it makes the bytes
deterministic by construction; **prefer an opaque payload when the content is foreign or must round-trip
unchanged**, and accept that the substrate is then blind to it by design.

- **The opaque-payload obligation (normative).** A producer that carries an opaque payload MUST
  serialize it **canonically** — a fixed, documented, deterministic encoding for a given logical value
  (canonical JSON with sorted keys, or better, `det_cbor` in a `bstr`). This is not tidiness: §4.4's
  exact-HLC-tie rule breaks by comparing `det_cbor(value)` **bytes**, and §6.1.1 sorts sections by
  those bytes. A producer whose serializer varies key order for equal content makes the tie-break
  depend on the serializer rather than on the data, and makes two byte-different ops that mean the
  same thing indistinguishable from two that do not. The substrate cannot check this for you — an
  opaque payload is opaque — which is precisely why it is stated as an obligation on the producer.
- **There is no `null`, deliberately.** `ext-value` excludes `null`, so "this register holds nothing"
  and "this register holds the empty string" are not distinguishable by the value type alone. This is
  not an omission to be worked around with a sentinel encoding chosen ad hoc: a product that needs the
  distinction MUST make it **explicit and self-describing** in its own value shape — a one-byte/one-char
  discriminator prefix on an opaque payload (`"v"+text` for set, `"x"` for cleared), or a native map
  with an explicit presence key (`{"set": true, "v": "…"}`). Both are conformant; what is prohibited is
  inferring emptiness from a value the algebra treats as an ordinary write. See §4.10, which is where
  the *choice of primitive* for "cleared" is decided — and where getting it wrong loses data silently.
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

### 4.1.2 Value-profile observability — asking a peer what it accepts (normative, advisory)

C-08 **widened** the accepted value type, and a widening has a failure mode a narrowing does not:
**a mixed deployment diverges by *rejection*.** An engine on the pre-C-08 profile answers `0x0A03`
to an op an updated engine accepts and applies; the two replicas then hold different op sets and
compute different roots, with an error raised on **only one side**. Nothing at the accepting end can
observe it — a refusal at the far end is indistinguishable from an op that never arrived. §14 C-08
records that hazard and says a product SHOULD keep using opaque payloads until every engine is
updated, but the first implementations of it found the advice **unactionable**: the document defined
no field, token or header by which one replica could *ask* another which profile it is on (§4.1's
`sync-1` sub-tokens cover only the unrelated per-op-vs-frame signing choice), so "until every engine
is updated" had no way to be evaluated except by an operator's out-of-band belief. This subsection
freezes the missing handle.

**The sub-token.** A node whose engine accepts the **whole** §18.3.6 `ext-value` — text-keyed maps
and heterogeneous arrays included, per §4.1 — MAY advertise the `sync-1` sub-token
**`sync-1/ext-value-2`**. (`ext-value-1` names the pre-C-08 narrower prose and exists only so the
two can be discussed; a node MUST NOT advertise it, since profile 1 is non-conformant, §14 C-08.)
It is carried in either or both of:

- the **capability token** granting `sync-1` (resource `sync-1/ext-value-2`, ability `sync`), where
  the deployment gates sync by capability (§5.4); and
- the **`GET /sync/vector` response**, which gains an OPTIONAL `profiles` member — key `4`, an array
  of sub-token `tstr`s — alongside `node`/`ns`/`vector`. This response is unsigned and is the one
  unauthenticated-shaped surface both peers already call, so a node MUST ignore members it does not
  recognize here rather than reject them (§2.2's unknown-key rejection governs **signed** objects,
  and this is not one). Adding key `4` changes no existing key and no frozen byte.

**It is an observation, not a gate — this is the load-bearing part.** A conformant node:

- **MUST NOT** refuse to reconcile with, refuse a capability token from, or downgrade its response
  to a peer that does not advertise `sync-1/ext-value-2`. Absence means **"unknown or unstated"**,
  never "profile 1": the overwhelming majority of deployments carry no nested value at all, and
  gating on the token would break every one of them for a hazard they cannot experience.
- **MUST NOT** narrow its own validation to match a peer. A profile-2 engine accepts everything a
  profile-1 engine accepts; there is no negotiated-down mode, and an op is valid or invalid against
  §4.1 alone, never against who is listening. A per-peer accept set would make validity a function
  of the connection and is precisely the divergence this document exists to prevent.
- **MAY** use the signal in exactly one place: a **producer** deciding whether to *mint* a nested
  value. Until every peer it can observe advertises the token, it SHOULD keep carrying structured
  content as an opaque payload (§4.1.1) — the same advice as C-08, now with something to evaluate.
- **SHOULD** log an unadvertised peer rather than act on it, exactly as §5.2.2 treats the
  `floor.author` consistency signal.

**Why advisory is the correct strength, and not a hedge.** Ops relay **transitively** (§5.2: a round
is symmetric and ops propagate through any topology), so the replicas that will eventually validate
an op you mint are **not** limited to the peers you can observe. A node that polled every direct
peer and saw profile 2 everywhere still has no proof about a replica two hops out. The signal
therefore bounds the risk without discharging it, and any mechanism that *enforced* on it would be
claiming a completeness it cannot have while breaking the deployments that need it least. The real
obligation stays where C-08 put it — a deployment-wide statement that every engine is updated — and
this sub-token is how a node makes its own half of that statement checkable instead of assumed.

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
| `9` | `counter-aggregate` | PN-counter (compacted, §4.6) | `target`, `field`, `subject`=the summarized author, `value`=`[P_cut,N_cut]`, `hlc`=the cut point |

`SyncOp` gains one field, used only by `kind 9`:

```cddl
SyncOp gains:
? 9 => ik-pub,   ; subject   (kind 9 only) the author whose below-cut deltas this aggregate summarizes
```

### 4.2.1 Unknown `kind` (normative — §14 C-28)

No rule previously stated what a replica does with a `SyncOp` whose `kind` is not one of the values above
— the exact §14 C-08 shape (a value outside the currently-understood range) with no vector or sub-token to
manage it, so a mixed deployment's behaviour on an unrecognized `kind` was unspecified rather than merely
undesirable.

- **The rule.** A `SyncOp` whose `kind` is not a value in the table above (currently `1`–`9`) **MUST** be
  rejected as `ERR_SYNC_OP_INVALID` (`0x0A03`) at the point it would be applied. It MUST NOT be silently
  ignored, MUST NOT be applied on a best-effort guess at its shape, and MUST NOT be merely relayed onward
  without being counted as rejected locally — a relay's job is bytes (§5.2's transitive-relay property),
  never CRDT semantics, and anything reaching the apply layer with an unrecognized `kind` is refused there,
  uniformly, by every conformant engine.
- **This is still a divergence hazard, and a disclosed one.** A uniform rejection rule does not make a
  mixed deployment safe by itself: an engine on an older profile of this document that does not yet
  recognize `kind 9` (or a future `kind 10`) refuses an op a newer engine accepts and applies, and the two
  replicas' observable states differ with an error raised on only one side — precisely the §4.1.2
  divergence-by-rejection shape C-08 named, not a new one.
- **The missing handle, closed the same way C-08's was (§4.1.2).** A node whose engine's apply layer
  recognizes `kind` values through some maximum MAY advertise that fact as an additional `sync-1`
  sub-token, `sync-1/kind-max-N` (e.g. `sync-1/kind-max-9`), governed by **exactly** the §4.1.2 rules:
  carried in the capability token and/or `GET /sync/vector`'s `profiles` member; **MUST NOT** be used to
  refuse, downgrade, or gate a peer that does not advertise it (absence means "unknown," never "kind ≤
  8 only"); and its **one** conformant use is a producer deciding whether it is yet safe to mint an op of a
  newly-added `kind`. This gives the same "unactionable advice" gap C-13(b) closed for `ext-value`
  profiles an equivalent handle for `kind`, rather than leaving a second axis of profile drift with no way
  to ask a peer about it.

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
  `Deleted > Live` in the state order and `class ∈ {redact, expires, sensitive}`.
- **`death` (kind 4)** writes `Deleted(class)` (field = the class token) or `Live` (field = `"live"`).
  **Winner = greater HLC; at an exact HLC tie, greater `DeathState` wins ⇒ `Deleted` beats `Live`
  (remove-wins, fail-safe toward deletion).**
- **`class` carries no semantic order (clarification — §14 C-29).** "An ordered enum" in earlier text
  described only the `Live < Deleted` relation above, not an ordering *among* classes — no rule anywhere
  ranks `redact` against `expires` against `sensitive`, and none is needed for the domination rule, which
  cares only whether the state is `Deleted` at all. The one place a class order could matter — an **exact
  HLC tie** between two `Deleted` certificates of *different* classes for the same object — is resolved
  the same way any other exact tie is: the §2.2 general tiebreak, applied here to the encoded `field`
  (§4.1) since `death` carries its class there rather than in `value`. No class-comparison table exists or
  is required; `class` is a token the D3 invariant is indifferent to.
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
  - **Note on `0x0A06`'s reachability.** A `kind 5` op's target entry is `D[hlc.author]` — literally the
    signing author, with no separate field naming a different subject — so a validly-signed,
    wire-well-formed `kind 5` op can never legally reference another author's entry: `0x0A06` guards a
    case the wire format cannot express through §4.1's ordinary signature path, not a gap left open in
    it. It remains specified for an implementation whose internal representation tracks a delta's owner
    via a field distinct from the verified signer (e.g. threading `author` separately from the
    `COSE_Sign1` check as a refactor convenience) — such an implementation MUST assert the two are equal
    before applying and MUST use this code if they are not. `SYNC-PN-02`'s own note that its construction
    is "declarative… not spec-given" is this fact surfacing in the conformance suite; §10's table entry
    is correct as written and this bullet is why.
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
  the above-cut deltas by union. Compaction never changes the observable total (§6.2), and no replica may
  fold a delta it cannot prove is below the cut (fail-closed: no cut ⇒ no folding).
  - **Merging an aggregate correctly (normative — corrects a double-count, §14 C-23).** "Join aggregates
    by max, join above-cut deltas by union" is underspecified in exactly the way that matters: **above
    which side's cut?** If each side unions the loose deltas *it* considers above *its own* cut, a delta
    that is above one side's (lower) cut but at-or-below the *other* side's (higher, winning) cut is
    already folded into the winning aggregate **and** re-added as a loose entry — counted twice.
    **Worked example.** Author `a` produces exactly two deltas ever, `h1: +12` and `h2: +3` (true
    total `P[a] = 15`). Side X compacted only through `h1` (`aggregate_X = 12` at cut `h1`) and still
    holds `h2`'s `+3` as a loose entry (it is, from X's own point of view, "above X's cut"). Side Y
    compacted through `h2` (`aggregate_Y = 15` at cut `h2`, no loose entries — `h2` is *at* Y's cut, not
    above it). Merging X and Y by the naive rule: `max(12, 15) = 15` (correct so far, Y's aggregate
    already **is** the true total) **unioned with** X's loose `{h2: +3}` (since X still calls it "above
    its cut") **= 18** — three more than the true `15`, because `h2`'s `+3` is now counted once inside
    Y's aggregate and once again as X's loose entry.
  - **The correct procedure**, per author `a`, for any merge that involves at least one aggregate:
    1. Let `C = max` of every cut a side attributes to `a` (the cut of the **winning**, highest-cut
       aggregate among the sides being merged); adopt *that* aggregate as `(P_cut[a], N_cut[a])`. This is
       the `max`-of-aggregates step, sound because aggregates are prefix sums and a higher cut's
       aggregate strictly dominates a lower one's.
    2. **Discard, from every side's loose `(op-id, delta)` entries for `a`, any entry whose `hlc ≤ C`** —
       these are exactly what the winning aggregate at step 1 already accounts for. This is the step the
       earlier text omitted, and the worked example is what omitting it costs.
    3. Union the **surviving** (`hlc > C`) loose entries across sides, keyed by `op-id`, as in the
       ordinary (non-aggregate) merge.
    4. `P[a] = P_cut[a] + Σ{ d : (id,d) ∈ union, d ≥ 0 }`, `N[a]` symmetric. Re-running the worked example
       with this procedure: `C = h2`; X's loose `{h2: +3}` has `hlc ≤ C` and is discarded; the union is
       empty; total `= 15`. Correct.
    5. This remains a join: step 1's `max` is idempotent/associative/commutative over cuts, step 2's
       filter is a pure function of the winning `C` (not of merge order), and step 3 is the ordinary
       union — so the composed procedure is still commutative, associative, and idempotent regardless of
       merge grouping (§4.6's own associativity requirement, above).
  - **An aggregate is not a `SyncOp` as this document otherwise defines one, and §6.2 licensed shipping
    it in a `SnapshotBody` without saying what it would look like on the wire (gap closed, §14 C-23).**
    `(P_cut[a], N_cut[a])` has no `kind`, no per-op signature, and names an author (`a`) distinct from
    whoever computed it — it cannot be serialized as a member of `SnapshotBody = [ * (COSE_Sign1(SyncOp)
    / SyncFrame) ]` (§6.1.2) as originally described. **`kind 9`, `counter-aggregate`** (§4.2) is the
    signed form: `target`, `field` name the counter as usual; **`subject`** (the new envelope field,
    §4.2) names the summarized author `a`; `value = [P_cut, N_cut]` (an `ext-value` array of two
    non-negative integers, §4.1); `hlc` is the cut point, whose **`author` is the compacting replica
    doing the folding — not `subject`.** This is a deliberate, disclosed asymmetry: every other kind's
    signer *is* the party whose state the op changes, and its signature is therefore proof the change is
    authentic; a `counter-aggregate`'s signer only **attests** to a summary of a *different* party's
    history. §11 records the resulting honest limit. The op is otherwise ordinary: admitted-author and
    signature checks (§8/§9) apply to its signer as for any op, and it is subject to §4.2.1's unknown-kind
    handling on an engine that predates it.

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

### 4.10 Choosing the right primitive (normative guidance)

Every kind in §4.2 converges. That is not the hard part. The hard part is that **two different kinds can
both look like the operation you are modelling, converge perfectly, and disagree about what the user
sees** — and the wrong choice does not raise an error, it loses a later write and reports success. This
subsection is here because that has now happened to a real consumer, and the trap is not visible from the
per-kind sections when you read them one at a time.

**The trap: "delete" is two different operations.** §4.5's death certificate and a §4.4 LWW write of an
empty value are both spelled "delete" in a product's UI. They are not interchangeable:

| | §4.5 `death` (kind 4) | §4.4 `lww-set` of an empty/cleared value (kind 3) |
|---|---|---|
| Semantics | **Remove-wins. Dominates.** | Ordinary last-writer-wins. |
| A later ordinary write | **Does not revive.** A `set-add`/`lww-set` at *any* HLC, however much later, is outranked (the D3 invariant, §4.5). | **Revives.** Greater HLC wins as usual. |
| Undoing it | Requires an explicit `Live` write with a strictly greater HLC — a distinct, deliberate operation. | Is just the next write. Nothing special. |
| Right when | Removal is **permanent and policy-bearing**: a redaction, an expiry, a deleted document, a revoked member. The whole point is that a concurrent benign edit must not resurrect it. | Removal is an **ordinary edit that happens to be to an empty value**: clearing a field, blanking a cell, unsetting an optional attribute. |

**The worked example (both halves, from one consumer).** A spreadsheet and a slide deck, in the same
product, chose *differently* — correctly:

- **Clearing a spreadsheet cell is NOT a death certificate.** Select a cell, press Delete, type a new
  value: the value comes back. That is plain LWW. Modelling the clear as `death(target=cell)` converges
  fine and looks right in every test that only clears — and then **silently swallows the next edit into
  that cell, forever**, because the death certificate dominates the subsequent `lww-set` no matter how
  much later it is. The user types, the op is minted, signed, replicated, and applied by every replica,
  and the cell stays empty on all of them. There is no error to catch, no divergence to alarm on: every
  replica agrees on the wrong answer. The correct mapping is `lww-set` with an explicit "cleared" value
  (§4.1.1's discriminator rule, since `ext-value` has no `null`).
- **Deleting a slide IS a death certificate.** A deleted slide never comes back; the product has no
  operation that revives one. Here the domination is exactly the property you want: a peer that was
  concurrently *reordering* that slide must not resurrect it, and §4.5 guarantees it cannot. Modelling
  this as an LWW "deleted: true" flag would be the mirror-image bug — a concurrent move at a greater HLC
  would bring the slide back.

**The selection test.** Ask, of the product's own behaviour, not of the wire: *"is there any user action
that restores this thing, using the same ordinary operation that created it?"*

- **Yes → §4.4 LWW** (or §4.3 OR-Set remove). The removal is an edit. Use a discriminated empty value.
- **No — restoring it, if possible at all, is a distinct privileged operation → §4.5 `death`.**

**Why this is stated as a MUST-read rather than left to taste.** Choosing §4.5 where §4.4 belongs is
**silent, permanent, and converged data loss** — the worst failure shape this document can produce,
because every safeguard in it (signatures, canonical encoding, root comparison, divergence alarms) is
working correctly and confirming the loss. Choosing §4.4 where §4.5 belongs is a **resurrection** bug: the
redaction that §4.5 exists to make durable can be undone by a concurrent benign edit, which is the exact
hole §4.5's D3 invariant was written to close. An implementation MUST document, per modelled object,
which of the two it chose and the answer to the selection test above; a product that cannot state the
answer has not made the choice, it has stumbled into one.

**The same shape recurs across the other kinds.** The general rule is that **domination and ordering are
different properties, and only one of them is a clock**: `death` dominates regardless of HLC; LWW obeys
the HLC; the OR-Set is add-wins by *tag observation*, not by clock; and §4.8's cycle rule is decided by
ordered replay rather than by which move is later (which is why §4.8 says so explicitly). If your mental
model is "the later write wins," it is correct for exactly one of the six kinds.

---

## 5. Reconciliation wire protocol

Two replicas reconcile by exchanging ops each lacks. The baseline is the flowstock stateless
version-vector protocol; a **range-based Merkle** mode (grounded in the §5.6 `recon` reference) scales it
to large states. Bodies are deterministic CBOR (§2.2); a JSON edge is permitted only for a
non-interoperating local client. Endpoints are shown as HTTP (the HTTP test,
[`README.md § 4.2`](README.md#42-the-http-test--are-transports-pluggable-with-https-first-class)); the
same three-or-four operations bind equally to a mesh stream (§4.5).

### 5.1 The version vector

A **version vector** is a per-author watermark over the subscribed namespaces — but a replica MUST keep
two distinct per-author values, because §4.6 blesses gapped delivery (below) and they diverge whenever a
gap is open:

- **`contiguous_below[a]`** — the greatest HLC `h` such that the replica holds **every** op author `a`
  produced at or below `h`, with no gap. This answers "have I seen everything up to some point," and it
  is the **only** one of the two permitted on the wire wherever this document requires a completeness
  watermark: a `pull` vector (§5.2) and `Snapshot.covers` (§6.1).
- **`max_applied[a]`** — the greatest HLC applied from `a` at all, gap or no gap. §4.6 explicitly blesses
  gapped, non-prefix delivery of one author's ops ("partition, sparse backfill, snapshot fast-join,
  range-Merkle drill-down that has not yet completed" are its own words for it), so `max_applied[a]` can
  sit strictly above `contiguous_below[a]` while a gap remains below it — e.g. a replica that has applied
  `A@(W,9)` but not `A@(W,1)`. It is a **display/diagnostics value only** ("how current is this replica's
  view of `a`") and has **no wire representation** in this document; see §2.3 for why conflating the two
  is one defect, not two.

```cddl
VersionVector = { * ik-pub => Hlc }   ; author => contiguous_below[author]: the greatest HLC h such that
                                       ; the replica holds every op from author at-or-below h (NO gap
                                       ; below h). NEVER max_applied — see the two definitions above.
```

Grounded in flowstock's `SELECT author, MAX(hlc) GROUP BY author` — sound as the grounding for
`contiguous_below` specifically because flowstock has no gapped-delivery path, so its `MAX(hlc)` already
**is** a contiguity watermark there; this document adds delivery modes (§4.6) where a bare maximum stops
being one, which is why the two names exist here even though flowstock only ever needed one. A vector is
not causal-delivery state; it is a compact summary of "what I already have, with nothing missing beneath
it," used to compute the difference to ship — never a compact summary of "the largest thing I've seen."

**Encoding (normative).** The keys are the authors' raw `ik-pub` **byte strings** — never an ordinal, an
index, or any other stand-in — and the map is deterministic CBOR (§2.2): definite length, entries sorted
**ascending by encoded key bytes** (which, for equal-length `bstr` keys, is ascending by the raw public
key). §2.2's "integer-keyed maps sorted by encoded key" names the common case (`SyncOp`, `Hlc`,
`Snapshot`, COSE headers); the sorting rule is the general RFC 8949 §4.2.1 one and applies to this
`bstr`-keyed map identically. An author absent from the vector means "I hold nothing from this author,"
never "this author has nothing" (§7, absence is not authority); equivalently, an absent author's
`contiguous_below` is the empty prefix, not "unknown."

**`max_applied` has no wire form, deliberately (normative — §14 C-15).** A replica MAY track
`max_applied` internally and surface it on a local status/diagnostics surface, but MUST NOT carry it in a
`pull` request, a `GET /sync/vector` response's `vector` member, or `Snapshot.covers` (§6.1) — every one
of those is a completeness watermark by contract, and giving `max_applied` any wire slot at all recreates
the ambiguity this section exists to remove, because the next implementation under time pressure reaches
for whichever per-author maximum it already has in memory. Where this document says "the version vector"
anywhere past this point — the `pull` vector, `GET /sync/vector`'s `vector`, `Snapshot.covers` — it means
`contiguous_below` and nothing else.

### 5.2 Endpoints (baseline, grounded in flowstock)

```
GET  /sync/vector                → { node: ik-pub, ns: [tstr], vector: VersionVector }
POST /sync/pull   { vector, ns } → { ops: [ COSE_Sign1(SyncOp) | SyncFrame ] }   ; ops the caller lacks
                                 | { fast-join: FastJoin }                       ; caller is below the §6.2 cut
POST /sync/ops    { ops }        → { applied: u32 }                              ; push ops to the peer
GET  /sync/state/<root>          → det_cbor(SnapshotBody)                       ; §6.1.2 body, addressed by `root`
```

- **`GET /sync/vector`** returns the responder's node key, the namespaces it subscribes to, and its
  current `VersionVector`. (flowstock: `{node_id, vector}`.) It carries an OPTIONAL fourth member,
  `profiles` (key `4`, `[* tstr]`) — the `sync-1` sub-tokens this node's engine implements, of which
  `sync-1/ext-value-2` (§4.1.2) is the first. The response is unsigned, so a reader MUST ignore
  members it does not recognize rather than reject them, and MUST NOT treat an absent or unfamiliar
  `profiles` as grounds to refuse the peer (§4.1.2: observational, never a gate).
- **`POST /sync/pull`** sends the caller's vector (+ requested namespaces); the responder returns, oldest
  HLC first, up to a batch limit, every op it holds whose `hlc` exceeds the caller's vector entry for that
  op's author (or whose author is absent from the vector). (flowstock: `OpsAfter(vector, batch)`.) If the
  caller's vector is **below the responder's §6.2 truncation floor**, the responder answers `fast-join`
  instead of `ops` — never a partial suffix (§5.2.1, a MUST). **Every entry in the vector this endpoint
  sends or reads is `contiguous_below`, never `max_applied`** (§5.1, §14 C-15): the predicate ships
  everything *strictly greater than* the caller's entry, so an entry that is a bare maximum with a gap
  beneath it hides that gap from this exact comparison — the op below the gap never again compares as
  "greater than" anything the caller advertises, and it is unreachable from every peer thereafter, by any
  number of rounds. A responder MUST NOT accept a `max_applied`-style vector as a substitute and answer
  from it; there is no way to detect the substitution from the bytes alone, which is why §5.1 forbids
  producing one in the first place.
- **`GET /sync/state/<root>`** returns the `det_cbor(SnapshotBody)` (§6.1.2) — the compacted **op set**
  whose fold reproduces the observable state committed by `<root>`. It is a content-**keyed** fetch:
  immutable and cacheable/pinnable by any intermediary without trusting it, the same posture as a §22
  public object, served for free by the existing relay cache/pin role. Two honest notes on the
  difference from an ordinary §18.1.5 fetch, since the address is *not* the hash of the returned bytes:
  - **Verification is by recomputation, not by direct hashing.** The fetcher ingests the body, derives
    `ObservableState` per §6.1.1, hashes it, and requires equality with `<root>` (§6.1.2). This is
    strictly stronger than hashing the transfer bytes — it proves the ops *produce* the committed state
    — and it is why `<root>` remains the right address: it names the thing being committed to, not an
    encoding of it.
  - **An intermediary can cache but not validate.** A relay keyed on `<root>` serves bytes it cannot
    check, so a corrupt or substituted body is caught at the fetcher rather than at the cache. That is
    already the trust posture of this endpoint — it carries **no** authority of its own, and bytes are
    only adopted when they reproduce the `root` of a snapshot that verified under §6.1 — and it costs
    the fetcher a discarded body, never a corrupted state.
- **`POST /sync/ops`** pushes a batch; the responder applies each op that is new (dedup by op content
  hash / `hlc`), verifying signature (§4.1) and CRDT validity (§4) before apply, and returns the count of
  **newly** applied ops. Apply is idempotent: a re-pushed op is a no-op (matching flowstock's
  `INSERT OR IGNORE` oplog-dedup).
- **Op framing inside a collection (normative, frozen — `SYNC-OP-02`).** Wherever ops cross a wire as a
  collection — `pull`'s `ops` array (§5.2.1 key `1`), `POST /sync/ops`'s `ops` array, and a `fingerprint`
  drill-down's `ops` array (§5.3) — each member is the `COSE_Sign1` (or `SyncFrame`) **as a CBOR item,
  embedded directly in the array**. It is **NOT** wrapped in a `bstr`: the array element is the
  four-element `COSE_Sign1` array itself (`[protected, unprotected, payload, signature]`, §4.1), so a
  decoder sees `[[h'…', {}, h'…', h'…'], …]`, never `[h'82…', …]`. A `bstr`-wrapped member is malformed
  (`ERR_SYNC_OP_INVALID`, `0x0A03`); this is a MUST, in both directions, because a receiver cannot
  reliably tell the two framings apart from bytes alone and a mismatched pair fails as an opaque decode
  error at the worst moment.
  - **Why item-embedded, not `bstr`-wrapped.** The usual reason to `bstr`-wrap a nested object is to
    preserve its exact bytes across a re-encode, because something is hashed or verified over them. Here
    that reason does not apply: **every byte that is hashed or verified is already inside a `bstr`**. The
    signature preimage is the §4.1 `Sig_structure` built from the `protected` and `payload` `bstr`s; the
    `op-id` (§4.1) is computed over `det_cbor(SyncOp)` — the `payload` `bstr`'s contents. Nothing is
    computed over the *outer* `COSE_Sign1` encoding, so wrapping it would buy no integrity property and
    would instead add a second encoding layer, a second length prefix per op, and a second place for an
    encoder to disagree. Item-embedding also keeps the response **uniformly deterministic CBOR** (§2.2)
    and validatable as a single tree, rather than a tree of opaque blobs whose interiors a validator must
    re-enter to check canonicity. It is what RFC 9052 does natively (a `COSE_Sign1` is an item, not a
    blob), and it matches this document's own seam, visible in §5.2.1's `FastJoin`: **signed structures
    ship as items** (key `1`, the `Snapshot`), **opaque hash-addressed bodies ship as `bstr`** (key `3`,
    the `ObservableState`). An op is a signed structure, so it ships as an item.
  - **Discriminating the union.** Both members of `COSE_Sign1(SyncOp) | SyncFrame` are §4.1 `COSE_Sign1`
    arrays — per-op signing signs a `SyncOp`, frame signing signs a frame head — so the *outer* shape is
    uniform by construction and a receiver MUST discriminate on the decoded `payload`, never on the
    element's framing. This is the second reason a `bstr` wrap earns nothing: it cannot serve as the
    discriminator either.
- **A round is symmetric** (push-then-pull), so only one side of a pair need be reachable and ops relay
  transitively through any topology (hub-and-spoke, mesh, chain) — the flowstock property. Fetching is
  **read-only and content-addressable at the op level**, so responses are cacheable and a lying responder
  can withhold or stall (detectable as a vector that never advances) but **cannot forge** an op (that
  needs an author key) — the same trustless posture as [`FEEDS.md § 5.1`](FEEDS.md).

### 5.2.1 Fast-join from a snapshot (normative, frozen — `SYNC-FJ-01`/`SYNC-FJ-02`)

§6.2 permits a replica to **truncate its op-log** below a stability cut once a snapshot stands in for the
discarded prefix. That creates a caller §5.2's baseline response shape cannot answer: a peer whose vector
is *below* the floor is asking for ops that no longer exist here. This subsection is the answer.

**The MUST.** A responder whose §6.2 truncation floor is above the caller's vector — precisely: whose
retained snapshot's `covers` contains any HLC the caller's vector `lacks` — **MUST NOT** return an `ops`
response. It **MUST** return a `fast-join` response instead. Returning the surviving suffix is
prohibited even though it is well-formed and would be applied without error, because it is *indis-
tinguishable to the caller from a complete answer*: the caller would advance its vector, believe itself
converged, and have silently lost every truncated op. That is a lost-write, presented as a successful
sync — the exact failure §6.2's snapshot obligation exists to prevent, and the reason this is stated as a
MUST rather than a SHOULD. Returning an *error* is likewise not permitted as the ordinary answer, because
it strands a peer that has a perfectly good recovery path; an error is reserved for a responder that
cannot honour the path at all (`0x0A0C`, below). **Silence about truncation is the defect; the response
must name it.**

**Response encoding (frozen).** The `pull` response is a deterministic-CBOR **integer-keyed** map (§2.2);
the named labels in §5.2's endpoint sketch are illustrative prose, the integer keys below are the wire:

```cddl
PullResponse = { 1 => [ COSE_Sign1(SyncOp) | SyncFrame ] }   ; ops — the ordinary answer
              / { 2 => FastJoin }                            ; the caller is below the cut

FastJoin = {
  1 => Snapshot,       ; the §6.1 signed snapshot that replaced the truncated prefix
  2 => Hlc,            ; floor   the §6.2 cut in force at the responder (the caller's audit handle)
  ? 3 => bstr,         ; body    OPTIONAL inline det_cbor(SnapshotBody) (§6.1.2); see below
}
```

`PullResponse` key `1`'s members are framed per §5.2's **op-framing** rule: each `COSE_Sign1`/`SyncFrame`
is embedded as a **CBOR item**, never `bstr`-wrapped — and the same rule governs the ops *inside* a
`SnapshotBody`, since it is an ops collection like any other. `FastJoin` key `3` is nonetheless a `bstr`,
and deliberately so: it wraps the **whole** body as one hash-addressed unit, the alternative to a separate
`GET`, so it ships as the opaque body §5.2's seam assigns to a `bstr` rather than as a second op array
spliced into the response. A decoder unwraps it once and then decodes an ordinary ops array inside.

The two keys are **mutually exclusive** — a response carrying both, or neither, is malformed (`0x0A03`). A
responder that never truncates never emits key `2`, so the baseline is unchanged for every replica that
keeps a complete journal.

**Inline vs. by-reference — the decision, and why.** The choice is not either/or once you notice that a
§6.1 `Snapshot` is *already* a reference: it is a small, bounded, signed descriptor (`v`, `suite`, `ns`,
`covers`, `root`, `ts`, `signer`, `sig`, sized by the author count, not the data) whose `root` is the
§18.1.5 content address of the observable state. So the split is made along that seam:

- **The signed descriptor ships inline** (key `1`). It must — it is what carries the signature, the
  `covers` vector and the `root` commitment, and it is what makes the response self-describing. It is
  bounded, so it cannot blow up the response.
- **The body ships by reference** (`Snapshot.root`, fetched from `GET /sync/state/<root>`). This is
  the unbounded part — a namespace's entire live op set, potentially megabytes — and putting it in
  the `pull` response would make an ordinary sync round's response size unbounded and un-resumable, on a
  path whose batch limit exists precisely to bound it. By-reference instead **reuses infrastructure the
  protocol already has for free**: the body is content-addressed (§18.1.5) and immutable, so it is
  cacheable and pinnable by any relay under the §22 public-object rules, servable by *any* holder rather
  than only the responder, fetchable in parallel or resumably by range, and deduplicated across every
  peer fast-joining to the same `covers`. The cost is honest and small: one extra round trip, on a path
  taken only when a peer has fallen behind a truncation cut — a rare, already-expensive event — never on
  the steady-state sync round.
- **Inline is retained as a bounded optimization** (key `3`, OPTIONAL). A responder MAY include the body
  bytes when they are small (a deployment-set ceiling; `64 KiB` is the RECOMMENDED default), collapsing
  the common small-namespace case back to one round trip. A caller **MUST** verify key `3` exactly as it
  would a fetched body — by folding it and recomputing `root` (§6.1.2) — and **MUST** discard it and fetch
  by reference on mismatch. The inline copy is a cache hint, never a second source of truth. This keeps
  one verification path, not two.

**What the caller does (normative).** On receiving `fast-join`, a replica MUST, in order:

1. **Verify the snapshot** under §6.1 — signature (DS-tag `DMTAP-SYNC-v0/snapshot`) and signer admission
   (§9) — and check `Snapshot.ns` is a namespace it subscribes to (§7); otherwise `0x0A02`/`0x0A01`/
   `0x0A0A`. An unverifiable snapshot is discarded; the caller does **not** fall back to the suffix.
2. **Check what is checkable about `covers` and `floor`** — per §5.2.2, which states exactly which
   part of "the snapshot covers what I am missing" a caller can verify and which part it must trust.
   `covers` MUST be a well-formed, non-empty §5.1 `VersionVector`; a malformed one is `0x0A03`.
3. **Obtain and verify the body** — key `3` if present, else `GET /sync/state/<root>` from the responder
   or any holder. The body is a **`SnapshotBody`**: a compacted set of signed ops, **not** a state
   document (§6.1.2). Verify it by **ingesting it and recomputing** — each op through the ordinary §4
   path (signature, `ext-value`, CRDT validity), then derive `ObservableState` per §6.1.1 and require
   `0x1e ‖ BLAKE3-256("DMTAP-SYNC-v0/snapshot-state" ‖ 0x00 ‖ det_cbor(ObservableState))` to equal
   `Snapshot.root` (`0x0A09` on mismatch, and the body is discarded **whole**). Ingest into a
   **provisional** replica state, not the live one, so a body that fails to reproduce `root` leaves
   nothing behind. If no holder can serve it, `0x0A0C` and the round fails **closed** — the caller keeps
   its old vector rather than half-adopting.
4. **Adopt** — promote the provisional state and set the local vector to `covers` — under the
   deployment's declared §6.1 snapshot **trust policy**. This path does not create a new trust posture,
   it uses that one, including the verify-required deployment's later backfill-and-recompute obligation.
   Note that because the body is *ops*, adoption is a **merge**: a caller that already holds some of
   those ops deduplicates them by `op-id` (§5.2) rather than being reset by them, and re-adopting the
   same body is a no-op.
5. **Continue pulling above the floor** — re-issue `POST /sync/pull` with the new vector (`covers`), which
   is now at or above the responder's floor, and the ordinary `ops` answer completes the join. Fast-join
   replaces the *truncated prefix*, never the suffix. **Fast-join MUST make progress:** if that re-pull
   is answered with another `fast-join` carrying the same `Snapshot.root` and `covers`, the responder is
   looping and the caller MUST fail the round `0x0A09` rather than re-adopt. This is the one loop a
   below-floor caller can otherwise spin in forever.

### 5.2.2 The floor/`covers` relationship — what a caller verifies, what it trusts (normative)

`floor` and `covers` are **different kinds of object**, and the single most likely implementation error
here is to compare them as if they were the same kind:

- **`floor` is one `Hlc`** — a single point in the §3 total order, below which the responder retains no
  ops. Its `author` field is the tiebreaker component of *that one timestamp*; it is **not** a claim about
  that author's stream.
- **`covers` is a per-author `VersionVector`** — a high-water mark per author, saying what the snapshot
  folded in.

Consequently **there is no well-defined ordering between them**, and this document deliberately states
**no** "`floor` MUST NOT exceed `covers`" rule: the comparison is a category error, not a rule an
implementation failed to find. In particular, the natural-looking predicate `covers.lacks(floor)` — "does
the covers vector lack the floor HLC?" — is **wrong**, and returns `true` for a perfectly well-formed
fast-join. `SYNC-FJ-01`'s own frozen data is the counterexample: `floor` = `(W,5,A)` while `covers` =
`{A@(W,4), B@(W,7)}`, so `covers[A]` sits *below* the floor HLC — simply because author `A` produced no
op between `(W,4)` and the cut. Nothing was lost; nothing is wrong; a caller applying that check would
reject a conformant responder. An implementation that writes this check will pass its own unit tests and
fail against real peers, which is exactly how it reaches production.

**What the caller can verify (MUST):** the snapshot's signature and signer admission (§6.1/§9), that
`Snapshot.ns` is a namespace it subscribes to, that `covers` is a well-formed non-empty `VersionVector`,
that the state body hashes to `Snapshot.root`, and that the fast-join **makes progress** (step 5). These
are all local, byte-level checks over material the responder actually handed it.

**What the caller MAY additionally check (a consistency signal, not a MUST):** that `covers` carries a
mark for `floor.author`. A responder that truncated below `(W,5,A)` will in the ordinary case have folded
some op of `A`'s into the snapshot that replaced the prefix, so an absent mark is worth logging. It is
**not** a MUST because it is not entailed: if `A`'s only op is *at* the floor, that op is retained rather
than truncated and `covers` need never name `A` at all. A conformance failure must not be inferred from
it.

**What the caller must simply trust:** that **every op the responder truncated was folded into `covers`**.
That is a statement quantified over ops the caller **cannot see** — they are precisely the ops that no
longer exist at the responder — so no comparison of `floor`, `covers` and the caller's own vector can
establish it, and no amount of care at the caller can substitute. The obligation lives at the **responder**
(§6.2: "the snapshot MUST account for every op dropped", and truncation is refused whole rather than
performed partially), and its residual is governed by the §6.1 snapshot **trust policy** — a
verify-required deployment's later backfill-and-recompute of `root` is what eventually detects a responder
that truncated more than its snapshot covered. Saying so plainly is better than a check that looks like
proof and is not.

**Adopting `covers` may move the caller's vector backwards for some author, and that is intended.** The
caller sets its vector to `covers` (step 4); where it already held a *later* HLC from some author, that
entry regresses. It is not a rollback and MUST NOT be treated as an error: step 5's re-pull re-ships every
retained op above `covers`, including those. What a caller MUST do is ensure ops it holds that the
responder does **not** hold are pushed (`POST /sync/ops`) — the symmetric push-then-pull round of §5.2 —
since those are the only ops a fast-join can actually lose.

The §6.1 snapshot semantics (root determinism, `SYNC-SNAP-02`'s fast-join-equals-replay guarantee, the
trust policy and its residual) govern throughout and are **not** restated here; §5.2.1 specifies only how
that mechanism is *reached over the wire* from a `pull`.

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
  4 => VersionVector,    ; covers      per-author contiguous_below (§5.1) — NOT max_applied — of every
                         ;             op folded into `root`
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
- **`covers` is `contiguous_below`, never `max_applied` (normative — §14 C-15).** A producer that folds a
  gapped author prefix into `root` and then reports its own bare maximum HLC for that author as `covers`
  would **launder the gap network-wide**: every replica that fast-joins from the snapshot adopts `covers`
  as its own vector (below), and none of them ever again asks for the missing op — the pull predicate
  (§5.2) only ships ops strictly greater than a vector entry, so the gap is now invisible to every
  fast-joiner at once, not merely to the one replica that first opened it. `covers[a]` MUST therefore be
  `a`'s `contiguous_below`, i.e. the producer MUST NOT include, and MUST NOT have folded into `root`, any
  op from `a` above a gap in `a`'s own delivery — a producer holding `A@(W,9)` without `A@(W,1)` reports
  `covers[A] = (W,0)` (or omits `A`), not `(W,9)`, until `(W,1)` arrives and closes the prefix.
- **Fast join (the point).** A joining replica fetches a `Snapshot`, **ingests its body — a compacted set
  of ops (§6.1.2), not a state document** — sets its local vector to `covers`, and then pulls **only the
  ops after `covers`** (§5.2). It never replays the pre-snapshot history. Because `root` is recomputable,
  a replica that later backfills the pre-snapshot ops **MUST** recompute `root` and confirm it matches — a
  snapshot is **verifiable, not merely trusted**.
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
- **`ObservableState` is a commitment and a comparison object — it is NOT an import format.** It is what
  you hash to get `root`, and what two replicas compare to prove convergence. It is deliberately
  **lossy**: it drops precisely the merge metadata a replica needs in order to keep merging. Adopting it
  as a base state and continuing to apply ops is **unsound**, not merely unsupported — see §6.1.2, which
  specifies what is actually transferred and adopted.

### 6.1.2 The snapshot body is a compacted op set, not a state document (normative — `SYNC-SNAP-03`)

**The rule.** What a replica transfers and adopts at fast-join is a **`SnapshotBody`**: the minimal set of
**canonical, individually-signed ops** whose fold equals the snapshot's observable state.

```cddl
SnapshotBody = [ * ( COSE_Sign1(SyncOp) / SyncFrame ) ]   ; framed per §5.2's op-framing rule
```

It is served by `GET /sync/state/<root>` and MAY ride inline in `FastJoin` key `3` (§5.2.1). A replica
adopts it by **ingesting every member through the ordinary op path of §4** — same signature check, same
`ext-value` validation, same CRDT apply, same `op-id` dedup. There is no second ingest path, no "load
state" entry point, and this document does not define one.

**Why not just ship `ObservableState`.** Because it cannot be resumed from. §6.1.1 serializes only the
*observable projection* — that is exactly what makes `root` a clean equality object, and exactly what
makes it useless as a base state. Every kind loses the field that decides the next merge:

| Kind | Dropped by §6.1.1 | What breaks if you adopt the projection anyway |
|---|---|---|
| §4.4 LWW | the winning cell's **HLC** | A post-`covers` op's HLC has no incumbent to compare against. It is **not** automatically greater: `covers` bounds each author's *own* stream, and the HLC is a **total order across authors**, so an op above `covers[B]` can still be *below* the winning cell written by `A`. A full replayer keeps `A`; a projection-adopter overwrites with `B`. **Silent divergence, and the two replicas' roots differ forever.** |
| §4.5 `death` | the certificate's **HLC** | §4.5 revives only on a `Live` write with a *strictly greater* HLC. With no HLC there is nothing to be greater than: revival becomes either always-allowed or never-allowed, and the D3 invariant is unenforceable. |
| §4.3 OR-Set | the **add-tags** | A post-`covers` `set-remove` cancels *specific* add-tags. If the adopter never had them, the remove cancels nothing and the element stays present — add-wins degenerates into add-always. |
| §4.7 RGA | **element ids** and tombstones | A post-`covers` `seq-insert` names its left-origin by element id, and `seq-remove` names its target the same way. With ids gone, both reference elements the adopter cannot identify; §4.7's causal-readiness buffer then waits forever for an origin that already arrived. |
| §4.6 PN-counter | per-author `op-id`-keyed deltas | The *total* survives, so counters are the one kind a projection-adopt gets right — until a below-`covers` op arrives out of order by any path, at which point it is double-counted, because the `op-id` set that made §4.6's merge idempotent is gone (§14 C-01 is the same bug in a different disguise). |

So the engine is a **fold over ops, and only a fold over ops.** That is not an implementation limitation
that a richer state format would lift; it is what makes the merge functions total and the algebra
associative. A conforming implementation is not required to expose a state-import entry point, and one
that exposes none is **not** thereby incomplete.

**What the body verifies against.** `Snapshot.root` still commits to `det_cbor(ObservableState)` (§6.1.1,
unchanged — the hash, the DS-tag and every frozen byte of it stand). A caller therefore verifies a body by
**folding it and recomputing**, not by hashing the received bytes directly:

```
  ingest every op in SnapshotBody  →  derive ObservableState per §6.1.1  →  hash  ≟  Snapshot.root
```

A mismatch is `ERR_SYNC_SNAPSHOT_ROOT_MISMATCH` (`0x0A09`) and the body MUST be discarded whole. This is
**strictly stronger** than hashing an opaque state blob: it proves the ops actually reproduce the
committed state, rather than proving only that someone shipped the bytes they promised. It also shrinks
the §6.1 *trusted-checkpoint* residual, because every op in the body is independently COSE-signed under
§4.1 — a malicious signer can **omit** ops (a withholding attack, already detectable as a vector that
never advances, §5.2) but **cannot forge** one, so a "false starting state" is now bounded to a *subset*
of the true history rather than an arbitrary fabrication.

**The cost, honestly.** A body is larger than the projection it folds to — it carries an HLC, an author,
and a signature per surviving element, where the projection carries a bare value. That is the price of
resumability and it is not optional: the dropped bytes *are* the merge metadata. Where size matters, the
lever is §4.1's `SyncFrame` batching (one signature amortized over many ops), not a leaner state format.

**Guidance for a consumer whose storage is state-shaped.** Most products persist a *document*, not a
journal, and will reach for "save the state, load the state." The mapping is mechanical, and the
consumer that first hit this arrived at it independently:

1. **Keep the winning op per address, not the winning value.** Maintain, alongside your rendered state,
   the op bytes that produced each live element (one per LWW `(target, field)`; one per live OR-Set
   element; the certificate per deleted object; the insert per live RGA atom; the winning move per tree
   node). This map *is* your snapshot body.
2. **Order it with the engine's comparator, never a re-implemented one.** Deciding "which op is winning"
   with a locally-written HLC comparison reintroduces the divergence the shared engine exists to remove.
3. **"Save" = serialize that op set. "Load" = ingest it.** Loading is not a replace, it is a **merge** —
   ingesting a saved body into a session that already holds ops is safe and idempotent by construction
   (`op-id` dedup, §5.2), which a state-replace would not be.
4. **Your storage layer need not change shape.** A document-shaped store holds the body as one opaque
   blob; the fold happens on load. What changes is only *what you put in the blob* — ops, not values.

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
  Truncation without a retained snapshot at `V` is prohibited, and the snapshot MUST account for every op
  dropped — an op below the cut that `covers` does not fold in would be erased with nothing standing in
  for it, so the truncation is refused whole rather than performed partially. A peer that nevertheless
  arrives with a vector below the floor (a replica that was excluded from the cut as stale, §11 item 4,
  or that simply never advanced) is served by **§5.2.1 fast-join**, which is what makes truncation safe
  rather than merely permitted: the peer recovers via the snapshot, and the responder is forbidden from
  answering it with a bare suffix.
- **Body-retention obligation (normative).** Because a snapshot's body is a set of *ops* (§6.1.2), a
  truncating replica MUST retain every op that body needs — truncation removes **superseded** history,
  never **live** history. Concretely, the following are retained below the floor, and the rest of the
  prefix MAY be dropped:
  - the winning `lww-set` per live `(target, field)`;
  - the winning `death` op per object **whether that winner is `Deleted` or `Live`** — not only per
    *deleted* object (§14 C-12). A `Live` write is a **live cell, not superseded history**: it is what
    a later *lower*-HLC certificate has to lose to. Drop it, and a post-`covers` `death` at a lower HLC
    than the retained `Live` write deletes an object that a full replayer keeps alive, because §4.5's
    comparison has nothing left to be greater than. The omission is silent by construction: §6.1.1's
    projection lists only *deleted* objects, so a `Live` cell and no cell at all project identically —
    the root still matches at snapshot time and the two replicas diverge only at the next certificate.
    (Same shape as the LWW row of §6.1.2's table: an op genuinely above `covers` for its own author can
    still sit **below** an incumbent in the §3 cross-author total order.)
  - **every** uncancelled `set-add` per present OR-Set element — not merely one (corrected from "at
    least one," §14 C-16). A tag cancelled by a tombstone is still droppable *with* its tombstone,
    exactly as the first compaction bullet already says, but every tag that is **not** cancelled MUST
    survive. "Retain one, it folds identically" is true only **at snapshot time**: §4.3 presence is
    *some* uncancelled tag existing, so a body holding any single uncancelled tag reproduces the same
    `ObservableState`, and hence the same `root`, as a body holding all of them. It stops being true one
    op later, because a `set-remove` cancels **specific** add-tags (§4.3), not "presence" in the
    abstract — a `set-remove` that a full replayer would apply against a tag the thinned body already
    discarded cancels nothing there, leaving the element present in one replica's post-`covers` state and
    absent in another's, permanently, with no signature failure and no `root` mismatch to catch it (the
    retained-tag set is invisible in §6.1.1's projection, so `root` cannot police a violation of this
    rule the way it polices most others). This is why the requirement is a **MUST**, not a MAY: a body
    MUST retain every uncancelled add-tag of every present element, and no producer may substitute a
    subset on the reasoning that the subset folds to the same `root` today;
  - the winning `tree-move` per node still in the tree;
  - every live RGA atom's `seq-insert`, **plus, transitively, the `seq-insert` of any tombstoned atom
    that is the left-origin of a retained atom — and that atom's own `seq-remove` with it.** This is the
    one non-obvious case, and it has two halves. *The insert half:* §4.7 already
    requires tombstones to be retained "until GC, so a concurrent insert whose origin is a removed atom
    still resolves," and the same reasoning binds here — dropping a tombstoned origin strands its
    successors in the causal-readiness buffer of every replica that fast-joins from the body, which
    presents as a sequence that silently never converges rather than as an error. *The tombstone half
    is entailed by the insert half* (§14 C-12): an atom's tombstone **is** its `seq-remove` op, so a
    body that retained the insert and dropped the remove folds to a sequence in which that atom is
    **live** — the extra atom appears in §6.1.1's `rga` section and the recomputed root differs from
    `Snapshot.root`. That body fails its own author's fold-then-recompute check (§6.1.2) if that check
    is run, and corrupts every fast-joiner if it is not;
  - **every `counter` op (§4.6), without exception** (§14 C-13). The earlier list did not mention
    PN-counters at all, which reads as permission to drop them, and dropping them is unsound: **no
    counter delta is ever superseded.** Each `(op-id, delta)` pair is live history — §4.6's merge is
    the *union* of the `op-id`-keyed delta set, and §6.1.2's own table names those keys as what makes
    the merge idempotent. Drop one and the total is simply wrong for every fast-joiner; keep the
    *total* instead of the ops and you have shipped the projection, which C-09 forbids for exactly the
    reason that table records (a below-`covers` op arriving by any path is then double-counted). The
    one licensed reduction is §4.6's below-cut **aggregate**: an author's below-cut deltas MAY be
    folded into a single retained `(P_cut[a], N_cut[a])` entry keyed by the cut HLC, because below the
    cut completeness is *proven* — and a replica that cannot prove the cut folds nothing (fail-closed).
    Absent that aggregate, the ops **are** the state.

  A replica that cannot satisfy this retention set MUST refuse the truncation whole rather than perform
  it partially, on the same fail-closed footing as the accounting obligation below. **A replica that
  never truncates has no obligation here** — its complete journal is trivially a superset of any body.
  A truncating replica SHOULD discharge the whole obligation **mechanically** rather than by
  inspection: build the body, **fold it and recompute `root`** (§6.1.2), and refuse the truncation
  unless it matches. That is the same check the receiver will run, executed at the one moment the
  dropped ops are still enumerable — and every rule in the list above is a *consequence* of it, which
  is why each is a correctness requirement and not an optimisation. A list can be read incompletely;
  the fold cannot.
- **The floor is not comparable to `covers`, and the accounting obligation is not a comparison.** The
  floor is a single `Hlc` (a point in the §3 total order); `covers` is a per-author `VersionVector`. "The
  snapshot accounts for every op dropped" is a statement quantified over **ops**, not a relation between
  those two objects, and it is **not** expressible as any ordering test between them — a snapshot may
  legitimately have `covers[a]` *below* the floor for an author `a` that produced nothing in that window
  (`SYNC-FJ-01`'s frozen data is exactly this shape). The obligation is discharged **here**, at the
  truncating replica, which can enumerate the ops it is about to drop; it is **not** re-checkable by the
  peer that receives the fast-join, which by construction cannot see them. §5.2.2 states the peer-side
  half — what a caller verifies and what it trusts — and exists because the natural-looking peer-side
  check is wrong.

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
  subscription bootstraps the new namespace via that namespace's snapshot **body** + post-snapshot ops
  (§6.1.2) — the same op-ingest path as an ordinary sync round, never a state import.
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
document's text alone. **All 24 are now byte-exact**, generated by
[`../conformance/vectors/gen_sync_vectors.py`](../conformance/vectors/gen_sync_vectors.py) into
[`../conformance/vectors/sync_vectors.json`](../conformance/vectors/sync_vectors.json) — the same
throwaway-script provenance model as [`pub_vectors.json`](../conformance/vectors/pub_vectors.json): every
value is a direct, mechanical application of §3/§4/§5/§6/§7 to fixed inputs, no randomness — Ed25519 is
deterministic, so even the one signature vector is a reproducible known answer. An **independent Rust
implementation** of this document now exists and executes these vectors; it is what surfaced the §14
corrections C-01…C-07, and the vectors below are the ones it must reproduce byte-for-byte. Generation
stays in the script, so the vectors remain an independent check on any implementation rather than a
restatement of one.

Five of these were previously **NOT-FROZEN** stubs: `SYNC-OP-02` (the `COSE_Sign1` envelope framing),
`SYNC-TREE-01` (an outright contradiction between the stub's expectation and §4.8's replay algorithm),
`SYNC-SNAP-01`/`SYNC-SNAP-02` (the canonical observable-state schema) and `SYNC-RECON-01` (the
fingerprint fold). Each was resolved by **this document deciding first** — §4.1, §4.8, §6.1.1 and §5.3
now carry the normative frozen text together with the reasoning for the choice — and only then vectored.
Nothing below is frozen *by* the generator script: a vector may only mechanically apply a decision the
specification already states.

`SYNC-SNAP-03` and `SYNC-VAL-01` are **new** (C-09 and C-08), and their provenance is different from
every vector above them: they came from the **first real product adoption** of this engine rather than
from an independent reimplementation of this document. A consumer exercises a specification along an axis
a reimplementer does not — it arrives with data and storage of its own shape and asks the document to
accommodate them — and both gaps are of that kind. Neither is a contradiction a vector could have caught:
`SYNC-VAL-01` exists because §4.1 and §18.3.6 gave one name to two types and nothing compared them, and
`SYNC-SNAP-03` exists because §6.1's prose described an adoption step that §6.1.1's own schema cannot
support. Both were decided in the text first (§4.1/§4.1.1 and §6.1.2) and only then vectored. **No
previously frozen byte changed**; `SYNC-SNAP-02` gains one descriptive input field naming what its
`ObservableState` is *for* (a commitment, not a transferable body) and its expectation stands unaltered.

`SYNC-FJ-01`/`SYNC-FJ-02` are **new** (C-05), added with §5.2.1 in the same discipline: the response shape
and the below-floor MUST NOT were decided in §5.2.1 first, with the inline-vs-by-reference tradeoff
reasoned there, and only then vectored. They exist because an implementation hit a *hole* rather than a
contradiction — §5.2 had no way to express an answer §6.2 requires — which is the case a vector suite is
least likely to catch on its own, since there was no wrong answer to disagree with, only a missing one.

**C-11…C-14 change no count either — the suite stays at 24.** They correct one artifact and extend
another: `SYNC-FJ-01`'s inline `FastJoin` body, which C-09 left as a state document while claiming
the vector was unaffected (C-11), and `SYNC-VAL-01`, which gains the empty-map/empty-array accept
cases, the fixed depth ceiling and the profile sub-token (C-13/C-14). C-12 and C-13(a) are §6.2
retention rules with no vector of their own: they are already entailed by `SYNC-SNAP-03`'s
fold-then-recompute, which is the check that catches a body missing any of them.

They were tightened again by **C-06/C-07**, and the lesson is worth stating where the vectors are defined:
`SYNC-FJ-02`'s `ops` member was originally a bare `SyncOp` **stand-in** rather than a real `COSE_Sign1`,
which is why it could not settle the op-framing ambiguity (C-06) that §5.2's prose also left open. A vector
whose artifact does not have the shape the specification requires cannot discharge the specification's
ambiguity — it merely looks like it does. The `ops` responses here now carry real `COSE_Sign1` envelopes,
item-embedded per §5.2, and `SYNC-FJ-02` additionally freezes the `floor`/`covers` **non**-relationship
(§5.2.2) together with the naive predicate it rejects.

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
| `SYNC-SNAP-02` | Fast-join equals replay | join via snapshot+post-ops vs full replay | byte-identical observable state | **Frozen** — `sync_snapshot_fast_join_equals_replay`. Same §6.1.1 schema: snapshot-body ingest (§6.1.2) + post-`covers` ops yields byte-identical `ObservableState` (hence identical `root`) to a full replay, because only the observable projection is serialized. **Corrected (§14):** the vector's `snapshot_covers_cbor_hex` previously encoded an **integer**-keyed map `{1: Hlc, 2: Hlc}`, contradicting §5.1's `VersionVector = { * ik-pub => Hlc }` — the keys are the authors' 32-byte `ik-pub` **byte strings**, ordered by §2.2 canonical map rules (ascending by encoded key bytes). No expectation exercised it, so nothing failed; it is fixed so no implementer is misled |
| `SYNC-SNAP-03` | Snapshot body is an op set | body = the compacted signed ops folding to `root`; a post-`covers` op at `(W,3,B)` that is *below* the incumbent `(W,4,A)` | body folds to `root`; conformant replica **keeps** the incumbent; a replica that adopted §6.1.1's projection instead lands on the other value with a **different root** | **Frozen** — `sync_snapshot_body_is_op_set`. §6.1.2 (C-09): what is transferred and adopted at fast-join is a `SnapshotBody` — the minimal set of individually-signed ops whose fold equals the observable state — **not** `det_cbor(ObservableState)`, which §6.1.1 deliberately strips of the winning HLCs, add-tags and element ids the next merge needs. Verified by **fold-then-recompute** against `Snapshot.root` (`0x0A09` on mismatch, body discarded whole), which is strictly stronger than hashing the transfer bytes. The second half is why this is normative and not stylistic: `covers` bounds each author's *own* stream while the §3 HLC orders *across* authors, so an op genuinely after `covers` can still lose to the incumbent — invisible to a projection-adopter, which has the value but not its HLC |
| `SYNC-VAL-01` | `ext-value` boundary | **10** accept cases (incl. a depth-2 text-keyed map, a heterogeneous array, and the **empty map `0xa0`** + empty array `0x80`) and 6 reject cases (incl. an integer-keyed map nested at depth 2) | accepts/rejects as listed; every reject is `0x0A03`; validation is **recursive**; the depth ceiling is **64** and is a MUST | **Frozen** — `sync_ext_value_boundary`. **Extended (§14 C-13/C-14):** the empty map is accepted — `0xa0` is key-type-ambiguous but *vacuously* so, and refusing it would make a legal `ext-value` un-decodable; the depth ceiling is fixed at 64 and made a MUST rather than left to each implementation's "ordinary" one (no byte-exact over-deep case is frozen, deliberately: an off-by-one at the boundary would pin an accident rather than the rule); and the advisory `sync-1/ext-value-2` sub-token (§4.1.2) is recorded as the profile handle, with `is_a_gate: false` pinned alongside it. §4.1/§4.1.1 (C-08): a `value` is **exactly** §18.3.6's `ext-value` — `bool / int / bytes / tstr / [* ext-value] / { * tstr => ext-value }`, recursive, heterogeneous arrays and text-keyed maps both admitted. §4.1 previously described a narrower type under the same name, so nested application data was blocked twice over: no tag to *encode* a string-keyed map, and the integer-keyed map CBOR does offer correctly answering `false`. Both refusals are frozen here, on opposite sides of the boundary. The carrier op shows the intended end-to-end shape — one register per `(slide, object)`, nesting for representation while §4.1.1's merge boundary stays at the whole value |
| `SYNC-FJ-01` | Fast-join response encoding | caller's vector below the responder's §6.2 floor; responder holds a signed snapshot at `covers` | byte-exact `det_cbor({2: FastJoin{1: Snapshot, 2: floor}})`; key `1` (`ops`) absent; the body (a §6.1.2 compacted **op set**) is addressed by `Snapshot.root`, fetched from `GET /sync/state/<root>`, and the vector's inline (key `3`) variant carries that **op set** | **Frozen** — `sync_fastjoin_response`. §5.2.1 pins the response as a mutually-exclusive integer-keyed map and the encoding split: signed descriptor inline, unbounded `SnapshotBody` by content address, optional bounded inline copy as a cache hint verified by **fold-then-recompute**. **Regenerated (§14 C-11):** C-09's note here previously read "does not change this vector's bytes — key `3` is absent", which holds for `pull_response_cbor_hex` and **not** for `pull_response_with_inline_state_cbor_hex`, which this vector also carries and froze — so a frozen vector was pinning a value §6.1.2 forbids adopting. Key `3` now carries a real `SnapshotBody`: the ten signed ops that are the §6.2 retention set for this snapshot's state (including **both** PN-counter deltas, C-13), folding to the unchanged `snapshot_root_hex`. The superseded state-document framing is retained as an explicitly-labelled **non**-conformant artifact, as C-06's `bstr`-wrapped `ops` member is. No other byte of this vector changed |
| `SYNC-FJ-02` | Below-floor suffix forbidden | same responder, two callers: one below the floor, one at/above it | below-floor ⇒ `fast-join` **only** — returning `{1: ops}` (the surviving suffix) is non-conformant, the silent-loss case; at/above ⇒ ordinary `{1: ops}`, no `fast-join`. Caller-side: state body unfetchable ⇒ `0x0A0C`, vector unchanged; a repeated `fast-join` at the same `root`/`covers` ⇒ `0x0A09` | **Frozen** — `sync_fastjoin_below_floor_suffix_forbidden`. The predicate is `Snapshot.covers` containing any HLC the caller's vector lacks (§5.2.1). **Corrected (§14 C-07):** the vector now also carries the `floor`/`covers` **non**-relationship — its own `floor` `(W,5,A)` sits *above* `covers[A]` `(W,4)` — and the naive `covers.lacks(floor)` predicate as an explicit **rejected** check (§5.2.2). The `{1: ops}` responses in this vector carry **real `COSE_Sign1` envelopes**, item-embedded per §5.2 (C-06) |
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
6. **A decoder is attack surface before any signature is checked.** Every sync object is decoded
   *before* it can be verified — a `COSE_Sign1` must be parsed to find the signature it is checked
   against — so the CBOR decoder runs on **unauthenticated, attacker-chosen bytes** on every path
   (`pull` response, `POST /sync/ops` push, a fetched `SnapshotBody`, a `fingerprint` request). It
   MUST therefore be bounded independently of authentication: the §4.1 nesting ceiling of 64 is a
   MUST for that reason and not for interop alone, and it MUST be enforced *before* recursing, since
   a recursive-descent parser with no bound is a stack-exhaustion DoS reachable by any peer that can
   open a connection — including, on an open namespace (§9), any peer at all. Size limits (§16-class
   op-size and batch limits, §5.2) bound the other dimension; neither bounds the other. An
   implementation that relies on "no legitimate producer nests that deeply" has stated a property of
   its friends, not of its inputs.
7. **Metadata: authorship and timing are visible to co-authors.** Sync is not sealed-sender — every op
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
| `0x0A0C` | `ERR_SYNC_SNAPSHOT_STATE_UNAVAILABLE` | a `fast-join` snapshot verified, but no holder can serve the `ObservableState` at its `root` (§5.2.1) | FAIL_CLOSED_BLOCK — the caller keeps its old vector; it MUST NOT fall back to the truncated suffix |

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

| **C-05** | **§5.2 `pull` gains a `fast-join` response (new §5.2.1); a bare suffix below the truncation floor is now a MUST NOT.** §6.2 permits op-log truncation and §6.1 gives a stranded peer the recovery mechanism, but §5.2's response shape had no way to *say* "you are below my floor, fast-join from this snapshot" — leaving a truncating responder two bad options: return the surviving suffix, which the caller cannot distinguish from a complete answer and which **silently loses every truncated op** while appearing to succeed; or return an error, which strands a peer that has a working recovery path. §5.2.1 now freezes the response (`PullResponse = {1 => ops} / {2 => FastJoin}`, mutually exclusive), the caller's obligations (verify signature → check `covers` closes the gap → verify the state body against `root` → adopt per §6.1 → resume pulling above `covers`), and the split encoding: the **signed descriptor inline**, the **unbounded state body by content address** via the new `GET /sync/state/<root>`, with an OPTIONAL bounded inline copy as a cache hint. Adds `0x0A0C`. | **NORMATIVE — wire protocol + a new MUST NOT.** A responder that implements §6.2 truncation and answers a below-floor caller with `{1 => ops}` is **non-conformant and loses writes**. A responder that never truncates is unaffected. | The same independent Rust implementation (envoir `dmtap-sync` / `node::syncserve`), while implementing §6.2 truncation: it could not implement the section safely within §5.2's shape, extended the response with an unspecified key `2`, and flagged prominently that no vector froze it. It was right to refuse to work around the gap silently — this entry is the specification catching up. |

| **C-06** | **Op framing inside an `ops` array pinned to item-embedded, never `bstr`-wrapped (§5.2, new bullet; restated at §5.2.1).** §5.2/§5.2.1 wrote `{1 => [ COSE_Sign1(SyncOp) \| SyncFrame ]}`, which two implementations read two ways: a `COSE_Sign1` *is* an array, so embedding it directly as an array item is one defensible reading, and `bstr`-wrapping each op (the habit from formats that must preserve nested bytes) is another. Both readings survived because nothing in the text ruled either out. The decision is **item-embedded**, on the merits: nothing is hashed or verified over the *outer* `COSE_Sign1` encoding — the signature preimage is built from the `protected`/`payload` `bstr`s and the `op-id` is computed over `det_cbor(SyncOp)`, i.e. the `payload` contents — so a wrapper buys no integrity property while adding a second encoding layer, a per-op length prefix, and a second place to disagree; item-embedding keeps the whole response one deterministic-CBOR tree a validator can check without re-entering opaque blobs; it is RFC 9052's own framing; and it matches this document's existing seam (`FastJoin` ships the signed `Snapshot` as an item and the opaque hash-addressed body as a `bstr`). A `bstr`-wrapped member is now malformed (`0x0A03`). The rule is stated once and applies to every place ops cross a wire: `pull`, `POST /sync/ops`, and §5.3 drill-down. The union is discriminated on the decoded `payload`, never on framing — both variants are `COSE_Sign1` arrays. | **NORMATIVE — wire encoding.** An implementation that `bstr`-wraps ops in an `ops` array is **non-conformant**; the two framings do not interoperate and fail as an opaque decode error. | The same independent Rust implementation (envoir), which `bstr`-wrapped its `POST /sync/pull` ops and **declined to guess** when it found the frozen vector doing the opposite. It was right that the text supported both. Aggravating factor recorded here: `SYNC-FJ-02`'s `ops` member was a bare `SyncOp` stand-in rather than a real `COSE_Sign1`, so the vector could not settle the question either — a stand-in that does not have the specified shape is how an ambiguity survives a frozen vector. The vector now carries real `COSE_Sign1` envelopes. |
| **C-07** | **The `floor`/`covers` relationship documented as a non-relationship (new §5.2.2; §6.2 and §5.2.1 step 2 corrected).** §5.2.1 step 2 required that "the responder's `floor` MUST NOT exceed `covers`". That rule is **not expressible**: `floor` is a single `Hlc` (a point in the §3 total order) and `covers` is a per-author `VersionVector`, so there is no ordering between them. An implementer reaching for the nearest available predicate wrote `covers.lacks(floor)`, which returns `true` for a well-formed fast-join — `SYNC-FJ-01`'s own frozen data has `floor` `(W,5,A)` with `covers[A]` `= (W,4)`, because `A` produced no op in that window — and so rejects conformant responders. The unverifiable clause is **removed**, not reworded. §5.2.2 now states the split explicitly: the caller **verifies** the snapshot signature/admission/`ns`, `covers` well-formedness, the state body against `root`, and that fast-join **makes progress** (a repeated `fast-join` at the same `root`/`covers` is `0x0A09`, a loop this document previously allowed); it **MAY** check that `covers` carries a mark for `floor.author` as a logging-grade signal (not a MUST — an author whose only op is *at* the floor is retained, not truncated, so `covers` need never name it); and it **trusts** that every truncated op was folded into `covers`, since that is quantified over ops the caller cannot see. §6.2 gains the matching responder-side statement — the accounting obligation is discharged where the ops are enumerable — and §5.2.2 also records that adopting `covers` may move the caller's vector *backwards* for an author, which is intended and MUST NOT be treated as an error (step 5's re-pull re-ships the retained suffix). | **NORMATIVE — a removed MUST plus a new one.** An implementation enforcing any `floor`-vs-`covers` ordering check rejects conformant peers and must delete it; the new progress MUST (`0x0A09` on a repeated identical `fast-join`) must be added. | The same implementation, writing the caller side of §5.2.1: its first pass implemented the clause literally, and the check failed against `SYNC-FJ-01`'s own frozen data — the vector caught the specification's own sentence. |

| **C-08** | **§4.1's `value` type was silently narrower than the `ext-value` it named.** The bullet described "the `ext-value` subset (§18.3.6)" as "text, byte strings, integers, booleans, and **homogeneous arrays** thereof", which drops §18.3.6's `{ * tstr => ext-value }` arm entirely and adds a homogeneity constraint §18.3.6 does not impose. Two different types were therefore in circulation under one name, and an implementation following §4.1's prose **cannot encode a nested application object at all** — a string-keyed map has no tag, so it fails at the encoder, while the integer-keyed map CBOR does encode is (correctly) not an `ext-value`, so it validates and answers `false`. The same case is blocked twice, for no stated reason: the recursive type is deterministic under §2.2's canonical map rules and **already rides inside a signature preimage** on every MOTE via `Headers.ext`, so no new determinism risk was being avoided. §4.1 now states `ext-value` in full, with recursive validation, a transport-level depth ceiling, and the exclusions that are real (floats, tags, `null`, integer-keyed maps). New §4.1.1 states what nesting does **not** buy: the merge unit is the whole `value`, so nesting is representation, and per-leaf concurrency requires decomposing into `(target, field)` addresses — plus the opaque-payload canonicalization obligation (§4.4's tie-break compares `det_cbor(value)` bytes) and the discriminated-empty rule that stands in for the absent `null`. | **NORMATIVE — a widening of the accepted value type.** An implementation that rejects a text-keyed `ext-value` map, or that rejects a heterogeneous array, is **non-conformant** and must be updated; ops it currently rejects are valid. Because it is a widening, a mixed deployment diverges by *rejection* (old replicas refuse ops new ones accept), so a product SHOULD keep using opaque payloads until every engine in its deployment is updated. No previously-valid op becomes invalid. | A product adopting the substrate for the first time (ofisi's Slides investigation), which recorded both refusals as executable assertions and shipped its content as opaque JSON `tstr` rather than guess. |
| **C-09** | **The snapshot body is a compacted op set, not `ObservableState` (new §6.1.2; §6.1, §5.2, §5.2.1 step 3–4 and §7 corrected).** §6.1 said a joining replica "adopts its observable state" and §5.2 served `det_cbor(ObservableState)` from `GET /sync/state/<root>` — but §6.1.1 serializes only the **observable projection**, deliberately dropping exactly the metadata the next merge needs. Adopting it and continuing is **unsound for five of the six kinds**: LWW loses the winning cell's HLC (and a post-`covers` op is *not* automatically greater than it — `covers` bounds an author's own stream while the HLC totally orders across authors, so a projection-adopter overwrites a winner a full replayer keeps, and the two roots differ forever); `death` loses the certificate's HLC, making the D3 revival test unevaluable; the OR-Set loses add-tags, degenerating add-wins into add-always; RGA loses element ids, stranding every post-`covers` insert/remove in the causal-readiness buffer; only the PN-counter's total survives, and only until a below-`covers` op arrives by any path and is double-counted. §6.1.2 now specifies `SnapshotBody` — the minimal set of signed ops whose fold equals the observable state — served at `GET /sync/state/<root>`, adopted through the **ordinary op path** (no state-import entry point exists, and one is explicitly not required), and verified by **fold-then-recompute** against `Snapshot.root`. §6.2 gains the matching **body-retention obligation**, including the transitive rule that a tombstoned RGA atom which is the left-origin of a retained atom must itself be retained. | **NORMATIVE — what is transferred and how it is verified.** `Snapshot`, `root`, the DS-tags and §6.1.1's schema are **unchanged**; every frozen byte stands. What changes is the *body*: an implementation serving or adopting `det_cbor(ObservableState)` at fast-join is **non-conformant** and silently diverges. Verification changes from hashing the received bytes to folding them and recomputing — strictly stronger, since it proves the ops reproduce the committed state, and it shrinks the trusted-checkpoint residual to *omission* (unforgeable ops), never fabrication. | The same first adopter, from the other end: it built a snapshot as "one op per key" because the engine exposes no import entry point, and asked whether that was a workaround or the design. It is the design; the specification's own prose was the thing that read otherwise. |
| **C-10** | **§4 gains explicit primitive-selection guidance (new §4.10).** No text told an implementer *which* kind to reach for, and two of them look identical from a product's UI: §4.5's death certificate and a §4.4 LWW write of an empty value are both spelled "delete". They are not interchangeable — a death certificate **dominates**, so no ordinary later write revives the key. Mapping a spreadsheet's "clear cell" to §4.5 therefore converges perfectly, passes every clear-only test, and then **silently swallows the next edit into that cell forever**, on every replica, with no error and no divergence to alarm on. The mirror-image error is equally silent: modelling a permanent deletion as an LWW flag lets a concurrent benign edit resurrect redacted content, the exact hole §4.5's D3 invariant exists to close. §4.10 states the distinction, the worked both-halves example (clear-a-cell ⇒ LWW; delete-a-slide ⇒ `death`), the selection test ("is there any user action that restores this using the same ordinary operation that created it?"), and the obligation to record the answer per modelled object. It also records the general shape: domination and ordering are different properties, and "the later write wins" is correct for exactly one of the six kinds. | **Guidance, not a semantics change.** §4.4 and §4.5 are unchanged and were already correct as written. This is a new normative *selection* requirement (an implementation MUST be able to state, per modelled object, which it chose and why) because the failure mode is silent converged data loss rather than an error. | The same first adopter, which initially mapped "clear cell" to §4.5 because it looked like a delete, caught it before shipping, and chose §4.5 **correctly** for slide deletion in the same investigation — the pair is what made the distinction legible enough to specify. |

| **C-11** | **`SYNC-FJ-01` regenerated for C-09; §10's row corrected.** C-09 recorded that making the snapshot body an op set "does not change this vector's bytes — key `3` is absent here". That is true of `pull_response_cbor_hex` and **false** of `pull_response_with_inline_state_cbor_hex`, which the vector also carries — so a *frozen* vector was pinning, as the inline `FastJoin` key `3`, exactly the `det_cbor(ObservableState)` §6.1.2 had just made unadoptable. A frozen wrong value is worse than no value: it is the artifact an implementer copies precisely because it is frozen. Key `3` now carries a real `SnapshotBody` — the ten individually-signed ops that are §6.2's retention set for this snapshot's state, folding to the **unchanged** `snapshot_root_hex` — and verification of it is fold-then-recompute, not a hash of the transferred bytes. The ops are freshly minted rather than lifted from this file's earlier vectors, because those are independent scenarios that reuse HLC counters across each other and composing their literal ops would freeze a journal in which one author minted two ops at the same HLC (§3 forbids it); the state they fold to is identical, which is all `root` commits to. The pre-C-09 state-document framing is retained as an explicitly-labelled **non-conformant** artifact, the same discipline C-06 used for the `bstr`-wrapped op. §10's row is rewritten. | **Vector fix + a §10 correction.** No normative rule changes and no other frozen byte moves; §6.1.2 already governed. An implementation that had copied the old inline value was adopting a projection — the C-09 defect, delivered by the conformance suite itself. | Reported by the C-08/C-09 implementation (envoir `dmtap-sync`) after it landed both corrections and re-read what the suite still froze. Worth stating as a general lesson: a correction MUST be checked against **every** artifact a vector carries, not only the one the correction's prose had in mind. |
| **C-12** | **§6.2's body-retention list was too narrow in two places the document already entailed.** (a) It required "the winning `death` per **deleted** object". A winning **`Live`** cell must be retained too: it is live history, not superseded history, and it is what a later *lower*-HLC certificate must lose to. Drop it and a post-`covers` `death` **below** the retained `Live` write deletes an object a full replayer keeps alive, because §4.5's comparison has nothing left to be greater than. The failure is silent by construction — §6.1.1's projection lists only *deleted* objects, so a `Live` cell and no cell project identically, the root matches at snapshot time, and the divergence surfaces only at the next certificate. (b) It required retaining a tombstoned RGA atom's **`seq-insert`** when that atom is a live atom's left-origin, but not the atom's **`seq-remove`**. The tombstone *is* that op: retain the insert without it and the fold shows the atom **live**, an extra atom appears in the `rga` section, and the recomputed root differs from `Snapshot.root`. §6.2 now states both, and adds the mechanical discharge that entails all of them — build the body, fold it, recompute `root`, and refuse the truncation unless it matches, executed while the dropped ops are still enumerable. | **NORMATIVE — a widened MUST-retain set.** A truncating replica whose retention set omits either case publishes a body that either fails its own fold-then-recompute check or corrupts every fast-joiner. A replica that never truncates is unaffected. Both cases are *entailed* by §6.1.2's existing verification rule, so this tightens the statement of an obligation rather than adding one. | The same implementation, writing §6.2's retention predicate against §6.1.2's fold check: the check rejected bodies its own reading of §6.2's list had produced. The list was the incomplete side. |
| **C-13** | **Two gaps around what §6.2 and §4.1 leave unsaid: PN-counter retention, and a way to ask a peer its `ext-value` profile.** (a) **§6.2 never mentioned counter ops at all** — an omission that reads as permission to drop them — even though §6.1.2's own table names the `op-id`-keyed deltas as what makes §4.6 idempotent. **No counter delta is ever superseded**, so every `counter` op is retained; keeping the *total* instead is shipping the projection, which C-09 forbids for the reason that table gives (a below-`covers` op arriving later is double-counted). The single licensed reduction is §4.6's below-cut aggregate, where completeness is proven, fail-closed on no cut. (b) **C-08's mixed-deployment warning had no mechanism behind it.** It says a product SHOULD wait until every engine is updated, but the document defined no sub-token, header or field by which one replica could *ask* another which profile it runs (§4.1's `sync-1` sub-tokens cover only the frame-signing choice), so the advice could not be evaluated. New §4.1.2 freezes `sync-1/ext-value-2`, carried in the `sync-1` capability token and/or as an OPTIONAL `profiles` member (key `4`) of the `GET /sync/vector` response — and specifies it as **observational, never a gate**: absence means "unknown", never "profile 1"; a node MUST NOT refuse, downgrade, or narrow its own validation on it; its one conformant use is a *producer* deciding whether to mint nested values. That strength is deliberate on two grounds — refusing profile-1 peers would break every deployment that has no nested values at all (most of them), and ops relay transitively, so observing every direct peer still proves nothing about a replica two hops out. The signal bounds the risk; the deployment-wide statement discharges it. | **(a) NORMATIVE — a widened MUST-retain set** on the same footing as C-12. **(b) NORMATIVE but additive:** a new advertisable sub-token and one new OPTIONAL response member; no existing key or byte changes, nothing is required to advertise, and nothing may be refused for not advertising. | (a) The same implementation, which retained counter ops because the fold required it and recorded that §6.2's list did not say so. (b) The same implementation, which shipped `EXT_VALUE_PROFILE`/`sync-1/ext-value-2` as explicitly **local conventions** and said so in its own documentation rather than inventing a wire rule — the correct thing to do, and the reason the gap was legible enough to close here. |
| **C-14** | **The empty map `{}` was outside the `ext-value` boundary as specified, and the depth ceiling was not a number.** (a) `0xa0` encodes an empty map **regardless of key type**, and the empty `{ * tstr => ext-value }` map is a legal `ext-value` — so a validator that rejects "maps whose key type it cannot confirm" (the natural shape of a validator written to reject integer-keyed maps) refuses a legal value, and refuses it *asymmetrically* across a deployment. The ambiguity is real but **vacuous**: no entries, hence nothing that could be smuggled through them. `0xa0` MUST be accepted as the empty text-keyed map; the empty array `0x80` likewise; a map with **any** entry has a determinable key type and the integer-keyed case is still rejected, recursively. `SYNC-VAL-01` gains both cases. (b) §4.1 said only that an implementation "MUST apply its **ordinary** deterministic-CBOR nesting-depth ceiling", which is not a ceiling: a per-implementation number lets one encoder mint a value a second decoder refuses — divergence by rejection again, with no error at the minting end. The ceiling is now **64** container levels, a MUST, matching the ceiling DMTAP's deterministic-CBOR decoder applies to every other object, checked **before** recursing, applying to **all** sync decoding (ops, `PullResponse`, `SnapshotBody`, `fingerprint` requests) rather than to `value` alone. §11 gains the reason it is a security rule and not only an interop one: every sync object is decoded *before* it can be verified, so the decoder runs on attacker-chosen bytes on every path, and an unbounded recursive-descent parser is a stack-exhaustion DoS reachable by any peer that can connect. | **NORMATIVE.** (a) A widening: an implementation rejecting `0xa0` is non-conformant, and no previously-valid value becomes invalid. (b) A tightening in the sense that "whatever ceiling you had" is no longer conformant if it is not 64, and a decoder with **no** bound was never conformant but could previously claim it was. | (a) The C-08 implementation, whose parser accepted `0xa0` on the reasoning above and noted that `SYNC-VAL-01` did not cover the case either way. (b) The same implementation, which found its sync CBOR parser had **no nesting bound at all** — an unbounded-recursion hazard independent of C-08, capped at 64 to match `dmtap_core::cbor`. The document's word "ordinary" was doing work no implementation could see. |

| **C-15** | **§5.1's version vector conflated `max_applied` (the largest HLC ever applied from an author) with the completeness watermark a `pull` predicate and `Snapshot.covers` actually need — the general defect §2.3 now names.** §4.6 explicitly blesses gapped, non-prefix delivery of one author's ops ("partition, sparse backfill, snapshot fast-join, range-Merkle drill-down that has not yet completed" are its own words), so a replica could hold `A@(W,9)` without `A@(W,1)` and still advertise `9` under the old per-author-`max` reading. §5.2's pull predicate ships only ops **strictly greater than** the caller's vector entry, so `A`'s op at `(W,1)` became **permanently unreachable** from every peer, by any number of sync rounds — a silent lost-op, not an error. Because §6.1's `Snapshot.covers` was the same bare maximum, publishing a snapshot **laundered the gap network-wide**: every fast-joining replica adopts `covers` as its own vector and none of them ever again asks for the missing op, the signature and root checks all pass, and the replicas agree on a `root` that is quietly missing inventory. `VersionVector` is now specified as `contiguous_below[a]` only — the greatest HLC below which the replica provably holds every op from `a`, no gaps — and this is the **only** value permitted in a `pull` vector or in `Snapshot.covers` (§5.1, §5.2, §6.1); the old semantics survive as `max_applied`, an explicitly display/diagnostics-only value with **no wire representation**, so it cannot be silently substituted back in. **Alternative considered and rejected: require per-author causal (strictly contiguous) delivery, buffering out-of-prefix ops the way §4.7 already buffers RGA origins.** That would also close the gap, but it is the larger change — every delivery path (fast-join, sparse backfill, range-Merkle drill-down, §5.3) would need a reorder-buffer and an eviction policy, not just the two read sites of a vector — and it forecloses the sparse-backfill and drill-down modes §4.6 and §5.3 treat as ordinary, non-error operation; it also does not by itself fix `Snapshot.covers`, which launders the gap independent of how carefully an individual peer buffers its own delivery. Splitting the mark is the smaller change that is sufficient on its own. | **NORMATIVE — a redefinition of `VersionVector`'s semantics plus two new MUSTs.** An implementation whose `pull` vector or `Snapshot.covers` carries a bare per-author maximum computed without gap-tracking is non-conformant and can permanently strand any op that was ever delivered out of prefix order to it or to an upstream it snapshotted from; the failure is silent by construction (every signature verifies, every merge is a correct join). An implementation that has never delivered gapped — flowstock's own model, and any engine that already buffers to strict per-author causal order — already computes `contiguous_below` whenever it computes `max`, so it needs no code change, only the corrected reading of the field it already has. | Specification self-audit: reading §5.1's per-author `max` against §4.6's own gapped-delivery guarantee and §5.2's strictly-greater-than pull predicate together, per the §2.3 checklist this pass introduced — not by an implementation's test or property check. |

| **C-16** | **§6.2's OR-Set body-retention rule licensed "at least one uncancelled `set-add`" per present element; §5.2 then blessed two producers choosing *different* ones as equally conformant. Both are wrong one op past snapshot time.** §4.3 presence is *some* uncancelled tag existing, so any single retained tag reproduces the same `ObservableState` and `root` **at the snapshot** — the reasoning §6.2 gave ("folds identically, since presence is add-wins over tags") is correct exactly there and nowhere else. It fails the instant a `set-remove` cancels **specific** tags (§4.3), not presence in the abstract: a replica that only ever observed the add now missing from a thinned body emits a legitimate `set-remove` citing the tag it *did* see; a full replayer (holding every tag) finds the *other* tag still uncancelled and keeps the element present, while a fast-joiner from the thinned body has no surviving tag and drops it. Worked failure: element `"widget"` added twice, tag T1 by A and T2 by B, both uncancelled at snapshot time; body **P** retains only T1, body **Q** retains only T2; both fold to the identical `ObservableState` and `root`, both pass every check. A replica that only saw A's add later sends `set-remove("widget", observed=[T1])` — conformant under §4.3. Full replayer: T1 cancelled, T2 survives ⇒ present. Fast-joiner from P: no surviving tag ⇒ absent. Fast-joiner from Q ⇒ present. Three replicas, three individually-verified op sets, two permanently different observable states, and no signature or `root` check ever flags it, because the retained-tag set is invisible in §6.1.1's projection — this is the same class §2.3 names and the same shape as C-10 (a spec-encouraged optimisation that silently deletes converged data), found here in retention rather than in primitive choice. §6.2 now requires retaining **every** uncancelled add-tag of every present element, not "at least one"; §5.2's blessing of non-unique snapshot bodies (the "MAY serialize different (equally valid) bodies … e.g. choosing different uncancelled add-tags" bullet) is deleted outright rather than reworded, since the case it was written to license is exactly the one that loses data. | **NORMATIVE — a MAY-retain tightened to a MUST-retain-every, and a removed blessing.** A truncating replica that retains only one uncancelled add-tag per present OR-Set element is now non-conformant and can silently strand or resurrect elements for a peer whose subsequent `set-remove` cites a different tag than the one it kept; the failure is undetectable by `root` comparison at the time it is introduced. An implementation that already retains every uncancelled tag (the "simpler superset" the old text offered as an equally-valid alternative) needs no change — this correction removes the cheaper, unsound option, not the sound one. | Found by adversarial specification audit — reading §6.2's retention list against §4.3's tag-specific (not presence-specific) cancellation rule and §5.2's non-uniqueness blessing together; not surfaced by an implementation or a conformance vector. |
| **C-17** | **§3's skew bound stated explicitly as one-sided (fail-closed).** An op whose `wall` is more than the §16-class skew window (default 120 s) *ahead* of the receiver's clock is rejected (`ERR_SYNC_HLC_SKEW`, `0x0A05`) — bounding only how far a malicious or misconfigured author can push ordering into the future, the one direction a fabricated `wall` can attack (§3's total order). **A past-dated `wall`, however old, is never grounds for rejection on skew grounds alone**: the §5.6.4 grounding this bound is drawn from is itself one-sided, and a two-sided reading would reject the entire legitimate backlog an ordinary terminal produces after reconnecting from an offline period longer than the skew window — directly contradicting §2.2's "no merge ever depends on wall-clock accuracy or on arrival order" and §1/§11 item 1's offline-edit guarantee. A deployment wanting to bound *staleness* (a policy/retention concern, not clock forgery) MUST implement it as a separate, disclosed check, never by widening this one. | **NORMATIVE — a fail-closed bound clarified as one-sided, not two-sided.** An implementation that also rejects a far-past-dated `wall` is non-conformant and strands legitimate offline edits. | Specification self-audit: reading §5.6.4's one-sided grounding against §2.2's wall-clock-independence rule and the offline-edit guarantee together. |
| **C-23** | **§4.6's aggregate merge corrected to avoid a double-count, and its wire form specified.** "Join aggregates by max, join above-cut deltas by union" did not say *above which side's cut* — so a delta above one side's (lower) cut but at-or-below the other side's (higher, winning) cut was folded into the winning aggregate **and** re-added as a loose entry, counted twice (worked example: true total `15`, naive merge yields `18`). The corrected procedure: take the aggregate at the highest cut among the sides being merged; **discard**, from every side's loose entries, any at or below that cut; union only the survivors. Separately, an aggregate `(P_cut, N_cut)` has no `kind` or per-op signature of its own and could not be serialized as a `SnapshotBody` member as originally described; new **`kind 9`, `counter-aggregate`** (§4.2) is its signed wire form, with a new `subject` envelope field naming the summarized author (distinct from the signer, which is the compacting replica). | **NORMATIVE — corrects a double-count in merge semantics, plus an additive op kind.** An implementation merging aggregates without discarding at-or-below-cut loose entries overcounts a compacted author's total; one lacking `kind 9` cannot represent a compacted counter in a snapshot body. | Specification self-audit: working the aggregate-merge procedure against §4.6's own associativity requirement and finding the counted-twice case. |
| **C-25** | **Counter overflow in the HLC spills into `wall`, never wraps — a MUST for both local `Tick` and remote `Observe`.** When `counter` reaches its `u32` maximum (`0xFFFFFFFF`), the next increment MUST advance `wall` by one and reset `counter` to `0` — the same branch the ordinary `now > wall` case already takes — never land at `(wall, 0)`, which is a **wrap** that sorts *before* the op that caused it, retroactively inverting causal order. Because `Observe` folds a peer's counter forward unconditionally, this is **remotely triggerable** — any peer can send `counter = 0xFFFFFFFF` — so an engine whose counter silently wraps is non-conformant. | **NORMATIVE — fail-safe MUST, closes a remotely-triggerable ordering-inversion.** An implementation relying on its language's default integer-overflow behaviour for `counter` is non-conformant. | Specification self-audit: reading `Observe`'s unconditional forward-fold of a remote counter against §3's total order. |
| **C-28** | **§4.2.1 states the missing rule for an unrecognized `SyncOp.kind`.** No prior text said what a replica does with a `kind` outside the §4.2 table — the same shape as C-08's unrecognized-value gap, now for the op discriminator. An unrecognized `kind` MUST be rejected as `ERR_SYNC_OP_INVALID` (`0x0A03`) at the point it would be applied — never silently ignored, applied on a best-effort guess, or relayed onward without being counted as rejected locally. Because this is still a divergence-by-rejection hazard across profiles (an older engine that does not yet recognize `kind 9` refuses what a newer one accepts), §4.2.1 adds the advisory sub-token `sync-1/kind-max-N`, governed by the same §4.1.2 rules — never a gate; a producer's one conformant use is deciding whether it is safe to mint an op of a newly-added `kind`. | **NORMATIVE — closes an unspecified-behaviour gap, plus an additive advisory sub-token.** An implementation with no rule for an unrecognized `kind` is non-conformant; the sub-token is optional and gates nothing. | Specification self-audit: `kind 9` (`counter-aggregate`, C-23) exercised exactly the unrecognized-kind case C-08 had already named for values, with no rule governing it. |
| **C-29** | **§4.5 clarifies that `death`'s `class` token carries no semantic order among classes.** Earlier text called `class` "an ordered enum," which described only `Live < Deleted`, not an ordering *among* `redact`/`expires`/`sensitive` — no such ordering exists or is needed, since the D3 domination rule cares only whether the state is `Deleted` at all. An exact-HLC tie between two `Deleted` certificates of different classes is resolved the same way any other exact tie is: the §2.2 general tiebreak, applied to the encoded `field` bytes (where `death` carries its class). | **Editorial — clarification, no semantics change.** No class-comparison table exists or is required. | Specification self-audit: reading "ordered enum" against the D3 domination rule, which requires no such ordering. |

**Standing rule.** A defect between this document and a conformance vector is resolved by deciding
**which side is right on the merits** and correcting the other **in the open** (the §10 discipline: a
vector may only mechanically apply a decision this document already states). Where the defect is in this
document's *semantics* — as C-01 was — the correction is recorded as a normative change with its class
stated, so an implementer can tell at a glance whether they must change code.
