# 18. Appendix A: Wire Format (CDDL)

This appendix is the **machine-checkable, normative** rendering of every DMTAP wire object
defined in prose in §1–§13. Where prose and this appendix appear to differ, the intent of the
prose section governs and the divergence is a spec bug to reconcile (§10.4); the known
divergences are catalogued in §18.11. Independent implementations MUST be able to encode and
decode every object below, byte-for-byte identically, from this text alone.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174.

## 18.1 Conventions

### 18.1.1 Serialization — deterministic CBOR

All wire objects are encoded as **CBOR** (RFC 8949) using **Core Deterministic Encoding**
(RFC 8949 §4.2). An encoder MUST, and a verifier of a signed object MUST re-check that:

1. Integers, string lengths, array/map counts, and tags use the **shortest possible** encoding
   of the argument (RFC 8949 §4.2.1, "preferred serialization"); no indefinite-length items are
   permitted.
2. **Map keys are sorted by their deterministic CBOR encoding, compared bytewise, ascending**
   (RFC 8949 §4.2.1). Because every object below is keyed by small unsigned integers, this
   reduces to ascending numeric key order (`0, 1, 2, …`) with `0x00–0x17` encoded in one byte,
   `0x18–0xff` in two, and so on — but implementations MUST compare the *encoded key bytes*, not
   the abstract integer, so the two agree for all keys used here.
3. No duplicate map keys appear. A decoder MUST reject a map containing a duplicate key.
4. Floating-point values do not appear anywhere in DMTAP wire objects. Booleans appear only where
   a rule below admits `bool`.
5. NaN/Infinity, CBOR tags other than those explicitly defined here, and the CBOR `undefined`
   (major-type 7, value 23) MUST NOT appear and MUST be rejected on decode.

CBOR `null` (major-type 7, value 22, the byte `0xf6`) is the canonical representation of an
**absent optional field** *only inside a signing preimage* (§18.9); on the wire an absent
optional field is simply **omitted from the map**, never present with a `null` value. A decoder
MUST reject a wire map that carries an optional key whose value is `null`.

### 18.1.2 Integer-key convention (COSE/CWT-style)

Every DMTAP object that is a CBOR **map** is keyed by **small unsigned integers**, exactly as
COSE (RFC 9052) and CWT (RFC 8392) key their maps, for compactness and to make the deterministic
key order trivial. Keys are assigned **per object type**, starting at `1`, and are *never*
reused with a different meaning across versions of the same object. Key `0` is reserved as a
**variant discriminator** in the tagged-choice objects (`DeliveryTag`, `ChallengeResponse`,
`RecoveryMethod`); no non-choice object uses key `0`.

Integer keys **≥ 64 (`0x40`)** are RESERVED for future/extension fields, mirroring the reserved
message-kind range (§2.3). A decoder processing a **signed** object MUST reject any key it does
not recognize (fail closed) so that the signing preimage is unambiguous; a decoder processing an
**unsigned** object MAY ignore unknown keys ≥ 64. Text-keyed extensibility is confined to
`Headers.ext` (§18.3.6).

**How the `≥ 64` range is used on a *signed* object (normative reconciliation).** The fail-closed
rule above means an old peer cannot "ignore-but-preserve" an unknown `≥ 64` key in a signed
object — it rejects the whole object — so a signed object MUST NOT simply be extended in place and
sent to arbitrary peers. Structural extension of a signed object is instead done through
**capability negotiation (§10.2)**: a sender includes a `≥ 64` extension field in a signed object
**only toward a peer whose capability announcement advertises support** for that extension
(carried, versioned and rollback-protected, in a `system` MOTE, §10.2). A peer that never
advertised the extension never receives the key it would reject, so extension is additive and
flag-day-free while the fail-closed preimage guarantee is preserved. The alternative
"ignore-but-preserve unknown keys" affordance applies **only** to **unsigned** objects and to the
text-keyed `Headers.ext` map (§18.3.6, §21.20), where there is no signing preimage to keep
unambiguous. New `≥ 64` fields intended to be portable follow the extension procedure (§21.25) and are paired
with a capability token (§21.22); private-use extensions use an `x-`-prefixed `Headers.ext` key
instead.

### 18.1.3 Endianness

CBOR encodes all integers big-endian by construction (RFC 8949 §3), so map/field integers carry
no separate endianness rule. Where a **signing/hashing preimage** concatenates raw fixed-width
integers *outside* CBOR (only `Envelope.sender_sig`, §18.9.1), each such integer is encoded
**big-endian, fixed width**: a `u8` as 1 byte, a `u64` as 8 bytes. All other preimages are
deterministic CBOR and inherit CBOR's encoding.

### 18.1.4 Algorithm-suite tagging

Every object carries a **version discriminator** of some form; signed/encrypted objects that
**select an algorithm** carry a `suite` field (a `u8`, §16.7). The suite selects, as a **set**,
the signature algorithm, the KEM/PKE, the AEAD, and the content-hash for that object:

| `suite` | Sign | KEM/PKE | AEAD | Hash | Status |
|--------:|------|---------|------|------|--------|
| `0x01` | Ed25519 | X25519 (HPKE, RFC 9180) | ChaCha20-Poly1305 | BLAKE3-256 | LEGACY — verify only, MUST NOT originate (§1.1) |
| `0x02` | Ed25519 + ML-DSA-65 | X-Wing (X25519 + ML-KEM-768) | ChaCha20-Poly1305 | BLAKE3-256 | **v0 REQUIRED originating suite** (§1.1) |
| `0x03` | Ed25519 + ML-DSA-65 | X-Wing (X25519 + ML-KEM-768) | **AES-256-GCM** | BLAKE3-256 | RESERVED (AEAD-diverse emergency target, §16.7, §21.15) |
| `0x04` | Ed25519 + SLH-DSA-128s | X-Wing (X25519 + ML-KEM-768) | ChaCha20-Poly1305 | BLAKE3-256 | RESERVED; the intended **anchor** profile (§1.2.0, §16.7) |
| `0x05` | Ed25519 + ML-DSA-65 | X-Wing (X25519 + ML-KEM-768) | ChaCha20-Poly1305 | **SHA3-256** | RESERVED (hash-diverse emergency target, §1.1, §16.7, §21.15) |

**Normative status and implementation status are different axes — do not conflate them
(normative).** The Status column above is the **normative** one: what a conformant node MUST
originate, per §1.1. It is *not* a statement about what any implementation supports today, and
earlier drafts of this section conflated the two — labelling `0x01` "v0 REQUIRED" and `0x02`
"RESERVED" because that was the state of the reference implementation, which flatly contradicted
§1.1 and, read literally, instructed implementers to reject the very suite they are required to
originate.

The honest separation:

- **Normative (§1.1, governing):** originate `0x02`; accept `0x01` only to verify historical or
  constrained-peer objects; reject an unknown suite fail-closed (`ERR_UNKNOWN_SUITE`, §21.3);
  refuse to originate below the floor (`ERR_SUITE_BELOW_FLOOR`, `0x0125`).
- **Implementation status (disclosed, not normative):** the frozen conformance vectors
  (`conformance/vectors/`) are **all suite `0x01`**, because ML-DSA-65 and ML-KEM-768 are not yet
  implemented in the reference core. That is a **gap in the corpus, not a licence to originate
  `0x01`** — it means the `0x02` path is currently specified-but-unvectored, exactly as
  `conformance/README.md` records. A conformant implementation is measured against §1.1, and the
  corpus catches up to it; the requirement does not retreat to meet the corpus.

`0x03` shares `0x02`'s byte layout exactly — every suite-governed length is identical (§18.2) —
differing **only** in the AEAD selector (AES-256-GCM in place of ChaCha20-Poly1305; the AEAD
content key remains 32 B). `0x05` likewise shares `0x02`'s byte layout exactly, differing **only**
in the content-hash selector (SHA3-256 in place of BLAKE3-256; the digest remains 32 B, and travels
under multihash prefix `0x16` rather than `0x1e`, §18.1.5).

A decoder MUST reject an object whose `suite` it does not implement (fail closed, §1.1); it MUST
NOT guess. The `suite` value governs the **length and structure** of every `ik-pub`, `sig-val`,
and encapsulated-key byte string in the same object (§18.2). The DMTAP `suite` (`u8`) is
**distinct** from the MLS ciphersuite (a `u16`, e.g. `0x0001`, negotiated *inside*
`Envelope.ciphertext` per RFC 9420, §5.1); the two never share a field.

**Which hook each object carries (normative index).** Not every top-level object carries `suite`
directly; each has exactly one governing versioning/agility hook, as follows — a decoder MUST
apply the fail-closed unknown-value rule to whichever hook the object defines:

| Object(s) | Hook |
|---|---|
| `Envelope`, `Identity`, `DeviceCert`, `RecoveryPolicy`, `KeyRotation`, `MoveRecord`, `DomainDirectory`, `Profile`, `DeniablePrekeyBundle`, `MixNodeDescriptor`, `MixDirectory`, `GroupState`, `PostageStamp`, `CapabilityToken`, `KeyPackageRef`/`KeyPackageBundleRef` | explicit `suite` field (this section) |
| `SignedTreeHead` | explicit `suite`, plus `tree_size` as its monotonic version (§18.4.9) |
| `GatewayAttestation` | `disc` (key 0, §18.3.11) selects the attestation kind; the signature algorithm is that of the published `_dmtap-gw` key (`v=` scheme version, §7.2a) — **no `suite` field** |
| `LocationRecord` | `seq` (monotonic rollback defense) + `substrate` (transport agility, §21.24); `sig` is governed by the signing device key's `Identity` suite — **no `suite` field** |
| `Assertion` | inheritance: `sig` is verified under the pinned identity of `from` (§3.4, §13.3), whose `Identity.suites` governs algorithm and lengths — **no `suite` field** |
| `PushSubscription` | `provider` (key 1, §4.9.3); `sig` is governed by the `device_key`'s `DeviceCert`/`Identity` suite — **no `suite` field** |
| `GroupEvent` | inheritance: the group's `GroupState.suite` governs `committer_sig`; the opaque `mls` blob is versioned by its own RFC 9420 ciphersuite — **no `suite` field** |
| `WakePing`, `SphinxCell`, cluster-sync objects (§18.6.3), `InclusionProof`/`ConsistencyProof`, `ProvenanceRecord`, `GatewayAliasMap` | no signature of their own — secured by a different proof system or by the referencing signed object (§18.9) |

### 18.1.5 Hash-agility prefix

Every content-address and every inter-object hash reference (`hash` in the grammar) is a
**multihash-style, single-byte algorithm prefix followed by the raw digest**. The prefix values
are drawn from the multicodec registry, truncated to one byte for v0 (all values below fit in
one byte):

| Prefix byte | Algorithm | Digest length | Status |
|------------:|-----------|--------------:|--------|
| `0x1e` | BLAKE3-256 | 32 B | v0 REQUIRED (default) — the hash of suites `0x01`–`0x04` (§18.1.4) |
| `0x12` | SHA2-256 | 32 B | RESERVED (compliance migration) — no suite selects it in v0 |
| `0x16` | SHA3-256 | 32 B | RESERVED — the hash of suite `0x05` (§1.1, §18.1.4) |

Thus a v0 `hash` is exactly **33 bytes**: `0x1e ‖ BLAKE3-256(preimage)`. The prefix lets an
implementation migrate the digest algorithm (e.g. to SHA-256 where FIPS compliance requires it,
§2.2) **without changing the address format**. A verifier MUST reject a `hash` whose prefix byte
it does not implement, and MUST reject a digest whose length does not match the prefix's fixed
length.

**The suite is authoritative; the prefix is never an independent selector (normative).** §18.1.4
makes the `suite` select the content-hash *as part of a set*, and this subsection gives every
`hash` its own algorithm byte. Read alone, each is complete — and together they leave the
algorithm of a given 33-byte field **doubly determined**, with no rule saying which determination
wins. That is not merely an interoperability gap (two conformant implementations reaching
different verdicts on the same bytes); it is a **downgrade channel inside the agility mechanism**,
because whoever writes the prefix would otherwise choose which hash an object's integrity rests
on, independently of the suite the parties negotiated. The precedence is therefore fixed:

- Where the containing object carries a `suite` (§18.1.4), **the suite decides.** Every `hash`
  inside that object MUST carry the prefix byte of that suite's hash, and a verifier MUST reject
  the object if any prefix disagrees (`ERR_HASH_ALG_MISMATCH`, `0x0127`, §21.3) — it MUST NOT
  verify the digest under the algorithm the prefix names, and MUST NOT "try both".
- Where the object carries **no** `suite` hook (§18.1.4's index names them), the prefix is the
  object's **self-description** and does select the algorithm — this is the case the multihash
  form exists for, and the reason it is carried at all.
- Everywhere else the prefix is a **redundancy check**, not a choice: it makes a mis-parsed or
  cross-suite field fail loudly instead of being silently reinterpreted.

The one exception is a field this specification **explicitly designates** as an external
reference — a `hash` naming an object produced under a different suite, or by a system outside
DMTAP, where the containing object's suite says nothing about how the referenced bytes were
addressed. Such a field MUST be marked as such where it is defined, and its prefix selects; no
v0 field currently claims the exception, and a future one that does MUST state why the suite
cannot govern it. Absent that explicit designation, a prefix that disagrees with the suite is a
rejection, never a selection.

BLAKE3 is used in its **default (unkeyed) hashing mode with 256-bit (32-byte) output**. The
`derive_key` and `keyed_hash` modes are NOT used for content addresses. Where BLAKE3 is used as a
Merkle construction over chunk hashes, the domain-separated tree of §18.9.5 applies, not BLAKE3's
internal tree.

### 18.1.6 Domain separation & signing (general rule)

Every signature in DMTAP is computed over a **domain-separated preimage**:

```
preimage = DS-tag ‖ body
signature = Sign(sk_suite, preimage)
```

where `DS-tag` is the object-specific ASCII string from §18.9 terminated by a single `0x00`
byte, and `body` is defined per object (usually the deterministic CBOR of the object **with its
signature field(s) omitted**). Domain separation guarantees that a signature valid for one object
type can never be replayed as a signature for another. A verifier MUST reconstruct the exact
`DS-tag ‖ body` and MUST reject any signature that does not verify against it under the key
selected by `suite`.

**Hybrid suites sign a combined message representative (normative).** For a hybrid suite
(`0x02`/`0x03`/`0x04`/`0x05`) a `sig-val` is the **concatenation** of the component signatures
(e.g. `Ed25519_sig ‖ ML-DSA-65_sig`) and BOTH component signatures MUST verify (AND-composition,
§1.3). The two components do **not** sign the object preimage independently: following the IETF
LAMPS composite PQ/T construction, each signs one **composite message representative**

```
M' = DS-tag ‖ 0x00 ‖ suite ‖ body        ; suite = the 1-byte composite algorithm id (§18.1.4)
```

so the `suite` byte is inside what both components sign. This is what makes the composite
**non-separable**: a component signature over `M'` is not a valid standalone signature over the
single-algorithm preimage `DS-tag ‖ 0x00 ‖ body`, so it cannot be stripped out of a hybrid object
and replayed as an `0x01` signature, nor can an `0x01` signature be promoted into a hybrid
`sig-val`. A single-suite `sig-val` (`0x01`) signs the plain preimage above and is unchanged. A
verifier MUST reconstruct the representative that matches the object's `suite` and MUST NOT accept
a component verified against the other form. `Identity.sig` additionally carries one `sig-val` per
suite in `suites` (§18.4.1) — each computed under **its own** suite's representative, which is why
a legacy verifier's single-suite validation and a hybrid verifier's AND-validation can coexist on
one object without either being a downgrade of the other.

**A signature over a digest MUST label the digest (normative).** Nearly every preimage in §18.9 is
computed over the object's `body`. A few are computed over a **pre-hashed representative** instead —
a fixed-length digest standing in for a variable-length body, so the signer hashes once and the
signature covers a constant-size message (§18.9.2 is the v0 example). A bare digest is
**byte-indistinguishable across hash algorithms**: 32 bytes of BLAKE3-256 output and 32 bytes of
SHA3-256 output are the same field with nothing to tell them apart. A verifier that implements
both — precisely the situation a hash migration to suite `0x05` (§1.1) creates, and the whole
point of reserving it — would then accept a signature that is valid under *either*, and the
security of the signed representative collapses to **min**(BLAKE3, SHA3) at exactly the moment the
migration was supposed to deliver **max**. The rule is therefore general and applies wherever the
pattern occurs:

> Where a signature is computed over a **digest** rather than over the body, that digest MUST
> appear in the preimage in its §18.1.5 **multihash form** (`prefix ‖ digest`), never as a bare
> digest. A verifier MUST reconstruct the prefixed form and **MUST NOT** accept a signature that
> verifies only against an unprefixed representative.

§18.9.1 already does this correctly: it feeds `id_bytes`, the full `Envelope.id` byte string with
its `0x1e` prefix, not the raw 32-byte digest. §18.9.2 is amended to match. A preimage that omits
the prefix is non-conformant, and a prefix that disagrees with the object's `suite` is a rejection
under §18.1.5's precedence rule (`ERR_HASH_ALG_MISMATCH`, `0x0127`, §21.3), never a second thing to
try. The rule binds every future preimage as well: a new pre-hashed construction in this section
inherits it without needing to restate it.

### 18.1.7 CDDL prelude (shared productions)

The following CDDL rules (RFC 8610) are referenced by every object subsection and are collected
once here. `.size` constraints marked "v0" hold for `suite = 0x01`; under `suite = 0x02` the
suite-governed lengths change and the constraint is relaxed to the ranges noted in comments.

```cddl
; ── scalar aliases ────────────────────────────────────────────────
u8      = uint .size 1                 ; 0 .. 255
u16     = uint .size 2                 ; 0 .. 65535
u32     = uint .size 4                 ; 0 .. 4294967295
u64     = uint                         ; 0 .. 2^64-1 (CBOR shortest form)
ts      = u64                          ; ms since Unix epoch (§16.1)
suite   = 0x01..0xff                   ; DMTAP algorithm-suite id (§18.1.4)

; ── crypto byte strings (lengths suite-governed, §18.1.4) ─────────
hash    = bytes .size (33..129)        ; 1-byte alg prefix ‖ digest; v0 = 33
ik-pub  = bytes                        ; identity/device public key; v0 Ed25519 = 32 B
sig-pub = bytes                        ; ephemeral sender_sig verification key; v0 Ed25519 = 32 B
sig-val = bytes                        ; detached signature;         v0 Ed25519 = 64 B
enc-key = bytes                        ; HPKE/AEAD content key;      v0 = 32 B
peer-id = bytes                        ; libp2p PeerId = multihash(pubkey)
maddr   = tstr                         ; multiaddr, textual form (e.g. "/ip6/…/quic-v1")
```

**Integer-domain governance (normative).** `u8`/`u16`/`u32`/`u64`/`ts` above are not merely
comments on a shared alias — each names a **domain**, and a decoder MUST admit a field declared
against one of them into exactly that domain **at the decode boundary**: reject a value that is
**negative**, that **exceeds the declared width**, or that is **non-finite**, before the value
ever reaches a comparison, an ordering check, or arithmetic. §18.2 states this MUST for
suite-governed **byte-string** lengths; this is its counterpart for **integer** domains, and the
omission was accidental, not a narrower scope — nothing about a `u64` field makes out-of-domain
admission a smaller hazard than an out-of-length byte string.

This binds with particular force on every monotonic counter reachable from the network —
`Identity.version` (§1.3), `LocationRecord.seq` (§4.2), `caps_version` (§10.2),
`GroupState.version` (§5.8.2), `FeedHead.seq` (§22.4.2) — because an anti-rollback rule is a claim
about a **total order**, and a value that is well-formed to one engine and unrepresentable to
another makes an order-dependent rule return two different answers while both engines believe
they are conformant: a bare admit-then-compare implementation in a language with signed or
arbitrary-precision integers accepts a negative or oversized counter a strictly-`u64` engine
cannot represent at all, and a language with IEEE-754 `NaN` turns a failed decode into a value
against which every `<`/`>` comparison is silently false rather than an error. None of the five
sections above states this rule locally; each MUST be read together with this one. The same
requirement governs any **textual rendering** of such a counter (`substrate/SYNC.md` §3, for the
HLC's fixed-width string form).

---

## 18.2 Length & type governance by suite (normative)

For every byte-string field whose length is "suite-governed," the exact length under each suite
is fixed by the table below. An implementation MUST reject a field whose length does not match
the object's `suite`.

| Field class | `suite = 0x01` | `suite = 0x02` |
|-------------|---------------:|---------------:|
| `ik-pub` (signing) | 32 B (Ed25519) | 32 B ‖ 1952 B (Ed25519 ‖ ML-DSA-65) |
| `sig-val` | 64 B (Ed25519) | 64 B ‖ 3309 B (Ed25519 ‖ ML-DSA-65) |
| `enc-key` (AEAD content key) | 32 B (ChaCha20) | 32 B |
| HPKE encapsulated key (inside `ciphertext`) | 32 B (X25519) | 32 B ‖ 1088 B (X-Wing) |
| `hash` digest | 32 B (BLAKE3-256) | 32 B |

**Suite `0x03`** (Ed25519+ML-DSA-65 / X-Wing / AES-256-GCM / BLAKE3-256, §16.7/§21.15) has the
**identical byte layout to `0x02`** in every row above — including the 32 B `enc-key` — differing
only in the AEAD selector (AES-256-GCM), so no separate column is needed.

**Suite `0x04`** (Ed25519+SLH-DSA-128s, the intended **anchor** profile, §1.2.0/§16.7) differs
from `0x02` in exactly the two signature rows: `ik-pub` = **64 B** (Ed25519 32 ‖ SLH-DSA-128s
public key **32**) and `sig-val` = **7 920 B** (Ed25519 64 ‖ SLH-DSA-128s signature **7 856**,
FIPS 205 Table 2, identical for the SHA2 and SHAKE instantiations). Its KEM, AEAD-key and hash
rows are `0x02`'s.

**Suite `0x05`** (the **hash-diverse** target, §1.1/§16.7) has the **identical byte layout to
`0x02`** in every row above, including the 32 B `hash` digest — SHA3-256 and BLAKE3-256 are both
256-bit — differing only in which function produced it and in the §18.1.5 prefix byte it travels
under (`0x16`, not `0x1e`). A length check therefore cannot distinguish them, which is exactly why
the prefix is mandatory and why §18.1.5 makes the suite authoritative over it.

Because an `Identity` may carry an anchor suite differing from its operational
suite (§1.2.0), a decoder MUST select the length row by **the suite of the key that made the
signature**, not by a single per-object suite. The 7 920 B `sig-val` is why an anchor-signed
announcement is a **top-rung** MOTE on the bucket ladder (§4.4.1).

The PQ (`0x02`/`0x03`/`0x04`) lengths are **normative**, not forward planning: `0x02` is the v0
originating suite (§1.1) and the bucket ladder is dimensioned from these numbers (§4.4.1). An
implementation that does not yet support a suite MUST reject objects in it fail-closed
(`ERR_UNKNOWN_SUITE`, §21.3) rather than guessing — but it is **not conformant while it cannot
originate `0x02`**, and MUST NOT read that rejection rule as permission to originate `0x01`
(`ERR_SUITE_BELOW_FLOOR`, `0x0125`, §1.1).

---

## 18.3 Message-layer objects (§2)

### 18.3.1 `Envelope` (§2.2)

```cddl
Envelope = {
  1  => u8,               ; v          format version, = 0 in v0
  2  => suite,            ; suite      algorithm suite (§18.1.4)
  3  => hash,             ; id         content address of field 10 (ciphertext)
  4  => DeliveryTag,      ; to         routing target (§18.3.2)
  ? 5  => bytes,          ; epoch      MLS epoch / group-context ref (present iff group)
  6  => ts,               ; ts         sender timestamp (ms epoch)
  7  => u8,               ; kind       message kind (§2.3)
  ? 8  => KeyPackageRef,  ; keypkg     present iff this initiates an MLS session (§18.3.4)
  ? 9  => ChallengeResponse, ; challenge  anti-abuse proof (§18.3.3)
  10 => bytes,            ; ciphertext MLS/HPKE sealed Payload (§18.3.5)
  11 => sig-val,          ; sender_sig detached sig by an EPHEMERAL per-message key
  12 => sig-pub,          ; sender_key ephemeral public key that verifies field 11
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `v` | 1 | `u8` | MUST | Format version. MUST equal `0` in v0; a decoder MUST reject any other value (fail closed, §10.1, `0x0201`). **The top-level `v` is FROZEN by design (normative):** it is deliberately the **one** axis with **no** additive/dual-stack evolution path. *All* forward evolution of DMTAP routes through the versioned **sub-registries** — algorithm suites (§21.15), message kinds (§21.16), `Headers.ext` keys (§21.20), capability tokens (§21.22), DNS parameters (§21.21), and the rest — each carrying its own IANA range and dual-stack migration (§21.25). `v` exists solely as a **fail-closed tripwire**: it lets a wholly-incompatible successor wire format, should one ever be needed, be **rejected cleanly** rather than mis-parsed. A conformant v0 node MUST NOT attempt to negotiate, dual-stack, or best-effort-parse a non-zero `v`; there is intentionally no `v=` capability advertisement. (This is the honest reading: DMTAP evolves through sub-registries, not through the top-level version byte. See §21.25 item 8.) |
| `suite` | 2 | `suite` | MUST | Algorithm suite actually used for this MOTE (§18.1.4). Governs `sender_sig` length and the crypto inside `ciphertext`. MUST be a suite both parties support (§1.3); unknown ⇒ reject. |
| `id` | 3 | `hash` | MUST | Content address of the exact bytes of field 10 (`ciphertext`), computed per §18.9.4. A verifier MUST recompute it and drop the MOTE on mismatch (§2.7 step 2) *before* any decryption. |
| `to` | 4 | `DeliveryTag` | MUST | Routing target: recipient key, group id, or blinded tag (§18.3.2). If it does not resolve to this node/group, the MOTE is dropped (§2.7 step 4). |
| `epoch` | 5 | `bytes` | OPTIONAL | MLS epoch / group-context reference; present **iff** the MOTE targets an MLS group, so the recipient selects the right epoch key (§5.1). Absent for 1:1/HPKE-sealed MOTEs. Opaque; length set by MLS. |
| `ts` | 6 | `ts` | MUST | Sender wall-clock timestamp, ms since Unix epoch. Subject to clock-skew tolerance ±120 s (§16.1). Used only for ordering/expiry, never for correctness. |
| `kind` | 7 | `u8` | MUST | Message kind (§2.3): `0x00` mail … `0x0a` system, `0x0b` **deniable** (§5.2.1); `0x40–0x7f` reserved extensions. A node MUST NOT `ack` a kind it cannot validate (§10.1). |
| `keypkg` | 8 | `KeyPackageRef` | OPTIONAL | Present **iff** this MOTE initiates an MLS session against one of the recipient's published KeyPackages (async join, §5.3). Identifies the consumed KeyPackage (§18.3.4). |
| `challenge` | 9 | `ChallengeResponse` | OPTIONAL | Anti-abuse proof for a **cold** sender (§9), evaluated *without decrypting* (§2.7 step 6). Known contacts omit it (fast path). One of ARC token / PoW / stamp / vouch (§18.3.3). |
| `ciphertext` | 10 | `bytes` | MUST | The MLS `PrivateMessage`, the HPKE-sealed `Payload` (§18.3.5), **or** — when `kind = 0x0b` (deniable, §5.2.1) — the deterministic CBOR of a `DeniableFrame` (§18.3.9), whose payload is Double-Ratchet-encrypted. Opaque to every intermediary (the recipient node decrypts). Its bytes are the sole input to `id`. |
| `sender_sig` | 11 | `sig-val` | MUST | Detached signature by an **ephemeral, per-message** signing key (unlinkable) over the preimage of §18.9.1 `(DS ‖ id ‖ to ‖ ts ‖ kind ‖ challenge)`. Gates abuse; reveals no identity (identity is inside `Payload`). Verified at §2.7 step 3. |
| `sender_key` | 12 | `sig-pub` | MUST | The **ephemeral, per-message** public key that verifies `sender_sig`. A verifier has no persistent key to look up for an unknown cold sender, so the verification key MUST travel with the message; it is fresh per message (hence unlinkable) and asserts no identity. `sig-pub` is a raw public key whose length is fixed by `suite` (§18.1.4), matching the signature algorithm of `sender_sig`. The `challenge` (field 9) MUST be cryptographically bound to this key (§9.4). |

### 18.3.2 `DeliveryTag` (§2.2a)

A tagged choice; key `0` is the variant discriminator.

```cddl
DeliveryTag = KeyTag / GroupTag / BlindedTag

KeyTag     = { 0 => 1, 1 => ik-pub }   ; deliver to a recipient identity key (default)
GroupTag   = { 0 => 2, 1 => bytes }    ; deliver to an MLS group id (§5)
BlindedTag = { 0 => 3, 1 => bytes }    ; blinded per-contact tag BT = HKDF(ss, epoch_day)
```

| Variant | Disc. (key 0) | Value (key 1) | Meaning & constraints |
|---------|:-------------:|---------------|-----------------------|
| `KeyTag` | `1` | `ik-pub` | The recipient's persistent identity public key. Simplest; links the packet to the persistent key for any observer. |
| `GroupTag` | `2` | `bytes` (group id) | The stable group identifier (§5.8). Resolves to a group the receiving node is a member of. |
| `BlindedTag` | `3` | `bytes` (BT) | A per-contact blinded tag `BT = HKDF(shared_secret, epoch_day)` (§2.2a). RECOMMENDED for the `private` tier. Unlinkable to the persistent key across time/observers, but does NOT hide last-hop delivery (§6.4); implementations MUST NOT present it as full recipient anonymity. |

### 18.3.3 `ChallengeResponse` (§2.2b, §9)

A tagged choice; key `0` selects the proof type. All four are verifiable **without decrypting**
the payload (§2.7).

```cddl
ChallengeResponse = ArcToken / PowSolution / PostageStamp / Vouch

ArcToken = {
  0 => 1,               ; discriminator: ARC anonymous rate-limited credential (§9.3)
  1 => bytes,           ; issuer     issuer identifier / public key ref
  2 => bytes,           ; token      ARC presentation (per-origin-scoped)
  3 => bytes,           ; origin     recipient-origin scope the token is bound to
  ? 4 => bytes,         ; nonce      optional freshness nonce
}

PowSolution = {
  0 => 2,               ; discriminator: memory-hard proof-of-work (§9.4)
  1 => tstr,            ; algo       MUST be "argon2id" in v0
  2 => [ u32, u32, u32 ], ; params   Argon2id (m_KiB, t_iters, p_lanes)
  3 => bytes,           ; epoch_nonce  recipient-issued epoch nonce (anti-precompute, §16.5)
  4 => bytes,           ; solution   value satisfying the difficulty target
  5 => u8,              ; difficulty leading-zero-bit target set by recipient policy
}

PostageStamp = {
  0 => 3,               ; discriminator: prepaid real-money stamp (§9.5)
  1 => bytes,           ; issuer     postage issuer public key ref
  2 => bytes,           ; serial     unique serial (double-spend key, §9.5.1)
  3 => u64,             ; amount     value in minor currency units
  4 => tstr,            ; currency   ISO-4217 code, e.g. "USD"
  5 => ts,              ; expiry     stamp expiry (ms epoch)
  ? 6 => bytes,         ; audience   OPTIONAL recipient/gateway scope
  7 => sig-val,         ; sig        issuer signature (§18.9.7)
}

Vouch = {
  0 => 4,               ; discriminator: social introduction (§9.7)
  1 => ik-pub,          ; voucher    IK of the vouching contact (trusted by recipient)
  2 => ik-pub,          ; subject    key of the cold sender being vouched for
  3 => ik-pub,          ; recipient  the recipient this vouch is scoped to
  4 => ts,              ; exp        vouch expiry (ms epoch); MUST be rate-limited
  5 => sig-val,         ; sig        signature by `voucher` (§18.9.7)
}
```

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `ArcToken` | disc | 0 | `1` | MUST | Selects ARC. |
| | `issuer` | 1 | `bytes` | MUST | Issuer identity/key ref. An **unknown/unvetted** issuer (incl. the sender's own node) carries a rate budget of **ZERO** (§9.3.1) — treated as no token. |
| | `token` | 2 | `bytes` | MUST | The ARC presentation, **per-origin-scoped** (`draft-ietf-privacypass-arc-protocol`), giving per-recipient rate-limiting *and* cross-recipient unlinkability. Its request context MUST bind the envelope's `sender_key` (§9.2a) so a stripped presentation cannot be replayed under a different ephemeral key; a verifier MUST reject one whose context does not. |
| | `origin` | 3 | `bytes` | MUST | The recipient-origin the credential is scoped to; MUST match the verifying node's origin. |
| | `nonce` | 4 | `bytes` | OPTIONAL | Freshness nonce within the 120 s validity window (§16.1). |
| `PowSolution` | disc | 0 | `2` | MUST | Selects PoW (last-resort tier, §9.4). |
| | `algo` | 1 | `tstr` | MUST | MUST be `"argon2id"` (memory-hard) in v0; a plain-hashcash proof MUST be rejected (§9.4). |
| | `params` | 2 | `[u32,u32,u32]` | MUST | Argon2id parameters `(memory KiB, iterations, lanes)`. MUST meet the recipient's minimum. |
| | `epoch_nonce` | 3 | `bytes` | MUST | Recipient-issued epoch nonce; scope is `id ‖ recipient ‖ nonce(epoch)` (§16.5). Prevents precomputation. |
| | `solution` | 4 | `bytes` | MUST | The value whose Argon2id digest meets `difficulty`. |
| | `difficulty` | 5 | `u8` | MUST | Leading-zero-bit target; SHOULD be adaptive (§9.4). |
| `PostageStamp` | disc | 0 | `3` | MUST | Selects postage. |
| | `issuer` | 1 | `bytes` | MUST | Issuer key ref; subject to reputation scoring (§9.5.1). |
| | `serial` | 2 | `bytes` | MUST | Unique serial; the double-spend key. The redeemer MUST check it against the issuer's redemption endpoint (online), never accept offline (§9.5.1). |
| | `amount` | 3 | `u64` | MUST | Face value in minor units of `currency`. |
| | `currency` | 4 | `tstr` | MUST | ISO-4217 currency code. |
| | `expiry` | 5 | `ts` | MUST | Expiry; an expired stamp MUST be rejected. |
| | `audience` | 6 | `bytes` | OPTIONAL | Scopes the stamp to a specific recipient/gateway. |
| | `sig` | 7 | `sig-val` | MUST | Issuer signature over `(serial, amount, currency, expiry, audience)` (§18.9.7). |
| `Vouch` | disc | 0 | `4` | MUST | Selects vouch. |
| | `voucher` | 1 | `ik-pub` | MUST | IK of the introducing contact; the recipient MUST trust it (pinned contact) for the vouch to count. |
| | `subject` | 2 | `ik-pub` | MUST | The cold sender's key being introduced. |
| | `recipient` | 3 | `ik-pub` | MUST | The recipient the vouch is scoped to; MUST match the verifying node. |
| | `exp` | 4 | `ts` | MUST | Expiry; vouches MUST be rate-limited to prevent farming (§9.7). |
| | `sig` | 5 | `sig-val` | MUST | Signature by `voucher` over `(subject, recipient, exp)` (§18.9.7). |

### 18.3.4 `KeyPackageRef` (§2.2, §5.3)

Reference to a **single** recipient KeyPackage consumed to initiate an MLS session. Distinct from
`KeyPackageBundleRef` (§18.4.3), which locates the recipient's *whole* published bundle.

```cddl
KeyPackageRef = {
  1 => hash,            ; ref    content address of the consumed KeyPackage
  2 => suite,           ; suite  suite advertised by that KeyPackage
  ? 3 => tstr,          ; loc    OPTIONAL locator hint where it was fetched
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `ref` | 1 | `hash` | MUST | Content address of the specific KeyPackage consumed. One-time KeyPackages are consumed per session (§5.3); the recipient marks it spent. |
| `suite` | 2 | `suite` | MUST | Suite the KeyPackage advertises; MUST equal `Envelope.suite`. Per-message suite is negotiated at KeyPackage granularity (§1.3). |
| `loc` | 3 | `tstr` | OPTIONAL | Locator hint (mesh/relay address) where the KeyPackage bundle was retrieved. Informational. |

### 18.3.5 `Payload` (§2.4)

The plaintext sealed into `Envelope.ciphertext`. Only the recipient (or group members) can
decrypt it; all sender identity, subject, recipients, threading, and content live here.

```cddl
Payload = {
  1 => ik-pub,          ; from        sender identity key (IK); revealed only to recipient
  2 => sig-val,         ; sig         IK (or device key) over the canonical payload hash
  3 => Headers,         ; headers     (§18.3.6)
  4 => Body,            ; body        (§18.3.6)
  5 => [* hash],        ; refs        ids of MOTEs replied-to/referenced (threading)
  6 => [* Attachment],  ; attach      (§18.3.7)
  ? 7 => ts,            ; expires     requested client-enforced expiry
  ? 8 => bytes,         ; fs_ratchet  forward-secrecy ratchet material (§5.2)
  ? 9 => [+ GatewayAttestation], ; provenance  sealed gateway-attestation chain (§18.3.11, §7.8); absent ⇒ pure-mesh
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `from` | 1 | `ik-pub` | MUST | Sender identity key (IK). For a known contact it MUST match the pinned key; on first contact it is TOFU-pinned (§2.7 step 8, §3.4). |
| `sig` | 2 | `sig-val` | MUST | Signature by `from` (IK) or an IK-authorized device key over the canonical payload hash (§18.9.2). Verified at §2.7 step 8. |
| `headers` | 3 | `Headers` | MUST | Message headers (§18.3.6). MAY be an empty map. |
| `body` | 4 | `Body` | MUST | Message body: text (`tstr`) or opaque MIME (`bytes`). MAY be empty. |
| `refs` | 5 | `[* hash]` | MUST (MAY be empty) | Content addresses of MOTEs this one replies to / references, giving threading and `reaction`/`edit`/`redact` targeting (§2.3, §5.4). Order is preserved. |
| `attach` | 6 | `[* Attachment]` | MUST (MAY be empty) | Attachments (§18.3.7). Small ones inline; large ones by manifest reference. |
| `expires` | 7 | `ts` | OPTIONAL | Requested expiry; client-enforced deletion. MUST NOT exceed 1 year beyond `Envelope.ts` (§16.1); a larger value is clamped/ignored. |
| `fs_ratchet` | 8 | `bytes` | OPTIONAL | Forward-secrecy ratchet material (§5.2). Opaque; interpreted by the MLS/ratchet layer. |
| `provenance` | 9 | `[+ GatewayAttestation]` | OPTIONAL | The **sealed gateway-attestation chain** (§18.3.11, §7.8): one `GatewayAttestation` per legacy gateway that bridged this message, in temporal order. Present **iff** the message is **gateway-touched (legacy-origin)**; a native mesh message carries it **absent**, which is the **provable pure-mesh** signal (a legacy-origin message that lacked a valid attestation would be rejected at delivery, §7.2a / §19.3.1, so an *accepted* message with no `provenance` was never plaintext at a gateway). It rides **inside the sealed `Payload`** so the gateway identity, timing, and legacy-sender address it carries are visible **only to the recipient**, never to any mixnet intermediary (§7.8, §6.8). Covered by `Payload.sig` (the preimage is `Payload ∖ {2}`, §18.9.2) *and* each entry is independently signed by its domain-anchored `_dmtap-gw` key (§18.9.11). A **deniable** message (`DeniablePayload`, §18.3.10) MUST NOT carry it — deniable traffic is native P2P and never transits a gateway. |

### 18.3.6 `Headers` and `Body` (§2.4)

```cddl
Headers = {
  ? 1 => bytes,               ; thread   stable thread/conversation id
  ? 2 => tstr,                ; subject  mail only
  ? 3 => tstr,                ; mime     content type of Body
  4 => [* ik-pub],            ; cc       additional recipient keys (fan-out is per-recipient)
  ? 5 => { * tstr => ext-value }, ; ext   extension headers (§10) — deterministic-safe values only
  ? 6 => bool,                ; sensitive  MUST NOT be persisted at rest by the recipient (§6.7)
}

; ext values are constrained to the deterministic-CBOR-safe subset (§18.1.1): NO floats,
; NO NaN/Infinity, NO undefined, NO tags. Headers rides inside the SIGNED Payload (§18.9.2),
; so a non-canonical value here would break signature reproducibility.
ext-value = bool / int / bytes / tstr / [* ext-value] / { * tstr => ext-value }

Body = tstr / bytes           ; UTF-8 text, or opaque MIME bytes
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `thread` | 1 | `bytes` | OPTIONAL | Stable thread/conversation id for grouping; independent of transport ordering (§2.6). |
| `subject` | 2 | `tstr` | OPTIONAL | Human subject line; meaningful for `mail` kind only. UTF-8. |
| `mime` | 3 | `tstr` | OPTIONAL | Media type of `Body` (e.g. `"text/plain; charset=utf-8"`, `"message/rfc822"`). If absent, `Body` of type `tstr` defaults to `text/plain; charset=utf-8`. |
| `cc` | 4 | `[* ik-pub]` | MUST (MAY be empty) | Additional recipient identity keys. Delivery fan-out is one sealed MOTE **per recipient** (§2.4); `cc` is informational threading metadata visible only to those who can decrypt. |
| `ext` | 5 | `{* tstr => ext-value}` | OPTIONAL | Text-keyed extension headers (§10). The **only** place text keys are admitted — but values are restricted to `ext-value` (bool/int/bytes/tstr and nestings), **not** arbitrary CBOR: floats, NaN/Infinity, `undefined`, and tags are forbidden (§18.1.1 rules 4–5), because `Headers` is inside the signed `Payload` (§18.9.2) and a non-canonical value would make the signature non-reproducible. A decoder MUST reject an `ext` value outside `ext-value` (fail closed) rather than sign/verify over an ambiguous encoding. Unknown extension *keys* MUST be ignored, never rejected. Keys SHOULD be namespaced (e.g. `"x-vendor-foo"`). |
| `sensitive` | 6 | `bool` | OPTIONAL | If `true`, the receiving client **MUST NOT persist the message at rest** — hold it for an ephemeral view only, never write it to the durable MOTE store (§6.7, endpoint-seizure least-persistence). Cooperative like `expires` (§6.6 item 8): a compliant recipient honors it, a compromised one can still copy what it reads. Absent ⇒ `false` (normal persistence). |
| `Body` | — | `tstr / bytes` | (as `Payload.body`) | `tstr` ⇒ UTF-8 text; `bytes` ⇒ opaque MIME per `mime`. A decoder MUST accept either major type. |

### 18.3.7 `Attachment` and `ManifestRef` (§2.5)

```cddl
Attachment = {
  1 => tstr,            ; name      display file name
  2 => tstr,            ; mime      media type
  3 => u64,             ; size      plaintext size in bytes
  ? 4 => bytes,         ; inline    present iff small (≤ inline threshold)
  ? 5 => ManifestRef,   ; manifest  present iff large
  6 => enc-key,         ; key       per-file content key
}

ManifestRef = {
  1 => hash,            ; id        BLAKE3 Merkle-DAG root of the Manifest (§18.9.5)
  2 => u64,             ; size      total plaintext size
  3 => u32,             ; chunks    NUMBER of chunks (a count)
  ? 4 => Durability,    ; durability delivery/retention contract (§5.5.2); MUST for Referenced tier
}

; The durability contract for THIS delivery. It rides in the ManifestRef — inside the
; sealed, signed MOTE — NOT in the content-addressed Manifest (§18.3.8), so re-pinning /
; upgrading durability never changes the file's content address, and a holder cannot tamper
; with it (it is covered by Payload.sig, §18.9.2). Advisory locator only for holder_hint.
Durability = {
  1 => uint,            ; class      0=origin-hold 1=recipient-pinned 2=cluster-replicated 3=pinned
  ? 2 => uint,          ; retention  unix seconds; retention term for class 3 (pinned); absent ⇒ indefinite
  ? 3 => uint,          ; replicas   N for class 2 (cluster-replicated), MUST be ≥ 1; absent otherwise
  ? 4 => tstr,          ; holder_hint advisory pull-locator hint; NOT authoritative (§5.5.2)
}
```

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `Attachment` | `name` | 1 | `tstr` | MUST | Display name; MUST be treated as untrusted (path-sanitize on save). UTF-8. |
| | `mime` | 2 | `tstr` | MUST | Declared media type. |
| | `size` | 3 | `u64` | MUST | Plaintext size in bytes. |
| | `inline` | 4 | `bytes` | OPTIONAL | The encrypted-then-inlined content, present **iff** the file is ≤ the inline threshold (v0 ≤ 48 KiB of content; the padded MOTE then occupies the 64 KiB top rung, §16.4, §4.4.1). Mutually exclusive with `manifest`. |
| | `manifest` | 5 | `ManifestRef` | OPTIONAL | Reference to the file's manifest, present **iff** the file exceeds the inline threshold. Mutually exclusive with `inline`. Exactly one of {`inline`,`manifest`} MUST be present. |
| | `key` | 6 | `enc-key` | MUST | Per-file content key; the recipient decrypts chunks (or `inline`) with it. Travels only inside the (private) MOTE. |
| `ManifestRef` | `id` | 1 | `hash` | MUST | Content address = Merkle-DAG root of the `Manifest` over its ordered chunk hashes (§18.9.5). |
| | `size` | 2 | `u64` | MUST | Total plaintext size, equal to `Manifest.size`. |
| | `chunks` | 3 | `u32` | MUST | **Count** of chunks (⚠ note: in `ManifestRef`, `chunks` is a *number*; in `Manifest`, `chunks` is the *list of hashes* — §18.11). |
| | `durability` | 4 | `Durability` | OPTIONAL (**MUST** for the Referenced tier, §16.4) | The delivery/retention contract (§5.5.2). A **Referenced** file (> 25 MiB) MUST carry it with a known `class`; Inline/Attached files are durable by construction and MAY omit it. |
| `Durability` | `class` | 1 | `uint` | MUST | `0` origin-hold (best-effort, disclosed residual §6.6 item 10), `1` recipient-pinned, `2` cluster-replicated, `3` pinned(term). An unknown value ⇒ `ERR_FILE_MANIFEST_INVALID` (§21). |
| | `retention` | 2 | `uint` | MUST **iff** `class = 3` | Unix-seconds retention term for a `pinned` contract; absent ⇒ indefinite. A `class = 3` with no `retention` ⇒ `ERR_FILE_MANIFEST_INVALID`. After it elapses the host MAY GC (`ERR_FILE_RETENTION_EXPIRED`, §21). |
| | `replicas` | 3 | `uint` | MUST **iff** `class = 2` | Replica count N for `cluster-replicated`; **MUST be ≥ 1**. A `class = 2` with `replicas < 1` (or absent) ⇒ `ERR_FILE_MANIFEST_INVALID`. Tolerates N−1 holder loss (§5.5.2). |
| | `holder_hint` | 4 | `tstr` | OPTIONAL | Advisory pull-locator hint; **NOT authoritative** — a fetcher MUST still content-verify every chunk (§18.9.5) regardless of the hint (§5.5.2). |

### 18.3.8 `Manifest` (§5.5)

```cddl
Manifest = {
  1 => hash,            ; id        BLAKE3 Merkle root over chunk hashes (the content address)
  2 => u64,             ; size      total plaintext size
  3 => u32,             ; chunk_sz  fixed chunk size (e.g. 1 MiB, §16.4)
  4 => [+ hash],        ; chunks    ordered chunk hashes (list, ≥ 1)
  ; NOTE: NO key field — the content key travels ONLY in Attachment.key inside the
  ;       sealed MOTE (§18.3.7). A Manifest is a swarm-distributed content-addressed
  ;       blob; embedding the key here would leak it to every holder that serves it.
  6 => suite,           ; suite     suite for chunk encryption + hashing
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `id` | 1 | `hash` | MUST | Content address = domain-separated BLAKE3 Merkle root over the **ordered** `chunks` hashes (§18.9.5). MUST self-verify: a recomputed root MUST equal `id`. |
| `size` | 2 | `u64` | MUST | Total plaintext file size in bytes. |
| `chunk_sz` | 3 | `u32` | MUST | Fixed chunk size; every chunk except possibly the last is exactly this many plaintext bytes. v0 default 1 MiB (§16.4). |
| `chunks` | 4 | `[+ hash]` | MUST | **Ordered list** of per-chunk content addresses `h_i = prefix ‖ BLAKE3-256(encrypted_chunk_i)` (§18.9.5). At least one. Enables resumable/parallel/swarmed/deduplicated transfer (§5.5). |
| ~~`key`~~ | ~~5~~ | — | **FORBIDDEN** | The file content key MUST NOT appear in a `Manifest`. A `Manifest` is a content-addressed blob fetched from the swarm to obtain the chunk list (§5.5, §19.8.2); any holder that serves it would then also learn the key and could decrypt every chunk, defeating blind chunk-serving. The key travels **only** in `Attachment.key` (§18.3.7, key 6) inside the **sealed** MOTE. A decoder that receives a `Manifest` containing key `5` MUST reject it (`ERR_MANIFEST_KEY_PRESENT`, §21) as a leak/malformation, never use the embedded key. Key `5` is reserved-unused for this object so an old sender's leaky manifest is detected, not silently honored. |
| `suite` | 6 | `suite` | MUST | Suite governing chunk AEAD and the digest algorithm of each `h_i` and of `id`. |

### 18.3.9 `DeniableFrame`, `DeniableInit`, `DeniableMessage` (§5.2.1)

The transport frame of the **optional deniable 1:1 mode** (§5.2.1). Carried as the
`Envelope.ciphertext` of a `kind = 0x0b` MOTE. A tagged choice; key `0` is the variant
discriminator. **None of these objects carries a `sig-val`** — authentication is the Double
Ratchet's shared-key MAC (the AEAD tag), which either party could compute; that absence is what
makes the mode repudiable (§18.9.10).

```cddl
DeniableFrame = DeniableInit / DeniableMessage

DeniableInit = {
  0 => 1,               ; discriminator: X3DH/PQXDH first message (§5.2.1(a))
  1 => suite,           ; suite     0x01 ⇒ X3DH; 0x02 ⇒ PQXDH (ML-KEM-768)
  2 => ik-pub,          ; ik_a      initiator Ed25519 IK — for AD binding + to authorize idk_a; NOT a DH input
  9 => bytes,           ; idk_a     initiator DEDICATED deniable-identity DH key (X25519); the X3DH long-term DH input
  10 => sig-val,        ; idk_a_cert IK-authorized device-key sig over idk_a (DS-tag DMTAP-v0/deniable-idk, §18.9.10)
  3 => bytes,           ; ek_a      initiator ephemeral X25519 public key
  4 => hash,            ; spk_ref   content-addr of the responder signed prekey consumed
  ? 5 => hash,          ; opk_ref   content-addr of the responder one-time prekey consumed (absent ⇒ signed-prekey only)
  ? 6 => bytes,         ; kem_ct    PQXDH KEM ciphertext to the responder KEM key (present iff suite = 0x02)
  ? 7 => hash,          ; kem_ref   content-addr of the responder one-time KEM prekey consumed (PQ)
  8 => DeniableMessage, ; msg       the first Double-Ratchet message (embedded)
}

DeniableMessage = {
  0 => 2,               ; discriminator: subsequent Double-Ratchet message (§5.2.1(b))
  1 => bytes,           ; dh        sender's current ratchet X25519 public key
  2 => u32,             ; pn        number of messages in the previous sending chain
  3 => u32,             ; n         message number in the current sending chain
  4 => bytes,           ; ct        AEAD ciphertext of the DeniablePayload; the AEAD tag is the shared-key MAC
}
```

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `DeniableInit` | disc | 0 | `1` | MUST | Selects the X3DH/PQXDH first message. |
| | `suite` | 1 | `suite` | MUST | `0x01` ⇒ classical X3DH; `0x02` ⇒ PQXDH with ML-KEM-768 (§16.7). MUST satisfy the recipient's suite ratchet (§1.3); a below-high-water-mark suite is rejected (`0x020F`). |
| | `ik_a` | 2 | `ik-pub` | MUST | Initiator **Ed25519 `IK`**. Only the **recipient** parses this (sealed sender — the whole `ciphertext` is opaque to intermediaries). It is used for the AD identity binding (`AD = IK_A ‖ IK_B`, oriented initiator‖responder) and to authorize `idk_a`; it is **not** an X3DH DH input. The recipient MUST bind it to the pinned `name → key` identity (§3.4). |
| | `idk_a` | 9 | `bytes` | MUST | Initiator's **dedicated deniable-identity DH key** (X25519) — the X3DH/PQXDH *long-term identity DH input* on the initiator side, carried inline so an offline responder can complete the async handshake without fetching the initiator's own `DeniablePrekeyBundle`. The recipient MUST verify `idk_a_cert` before use; DH1 mixes `idk_a` (not any `IK`-derived key). |
| | `idk_a_cert` | 10 | `sig-val` | MUST | Signature by an `IK`-authorized device key of `ik_a` over the raw `idk_a` bytes (DS-tag `DMTAP-v0/deniable-idk`, §18.9.10) — the same certification carried in the responder's `DeniablePrekeyBundle.idk_sig` (§18.4.8). Authenticates that `idk_a` belongs to the initiator's identity; a failure is `ERR_DENIABLE_X3DH_FAILED` (`0x040C`). It signs a *public DH key*, never content, so it is deniability-neutral. |
| | `ek_a` | 3 | `bytes` | MUST | Initiator ephemeral X25519 public key for the X3DH DH mix. |
| | `spk_ref` | 4 | `hash` | MUST | Content address of the responder **signed prekey** (`spk`) consumed from the responder's `DeniablePrekeyBundle` (§18.4.8). Unknown/expired ⇒ `ERR_DENIABLE_X3DH_FAILED` (`0x040C`). |
| | `opk_ref` | 5 | `hash` | OPTIONAL | Content address of the responder **one-time prekey** consumed. Absent ⇒ the signed prekey alone was used (last-resort, rate-limited, §16.9). A one-time prekey MUST be marked spent on use. |
| | `kem_ct` | 6 | `bytes` | OPTIONAL | PQXDH KEM ciphertext encapsulated to the responder's ML-KEM key; present **iff** `suite = 0x02`. |
| | `kem_ref` | 7 | `hash` | OPTIONAL | Content address of the responder one-time KEM prekey consumed (PQ); absent ⇒ last-resort KEM key. |
| | `msg` | 8 | `DeniableMessage` | MUST | The first Double-Ratchet message, whose `ct` is already ratchet-encrypted under the freshly agreed root key. |
| `DeniableMessage` | disc | 0 | `2` | MUST | Selects a subsequent Double-Ratchet message. |
| | `dh` | 1 | `bytes` | MUST | Sender's current ratchet X25519 public key (the DH-ratchet step, §5.2.1(b)). |
| | `pn` | 2 | `u32` | MUST | Count of messages in the previous sending chain (skipped-key handling, MAX_SKIP §16.9). |
| | `n` | 3 | `u32` | MUST | Message number in the current sending chain. |
| | `ct` | 4 | `bytes` | MUST | AEAD ciphertext of the `DeniablePayload` (§18.3.10) under the derived message key. The **AEAD tag is the shared-key MAC**; a verification failure is `ERR_DENIABLE_RATCHET_AUTH_FAILED` (`0x040D`). The AEAD **associated data** is the standard Double-Ratchet AD — the ratchet header `(dh, pn, n)` concatenated with the X3DH-established context `AD = IK_A ‖ IK_B` (the two parties' Ed25519 identity keys, §5.2.1), **canonically oriented `initiator ‖ responder`** — `IK_A` is the session initiator's `IK` and `IK_B` the responder's, fixed for the lifetime of the session **regardless of which party is currently sending** — so a message cannot be cut-and-pasted across sessions or reattributed and both endpoints derive byte-identical AD. It does **not** and cannot bind `Envelope.id` (which is the hash of the ciphertext that already contains this `ct`). **No signature accompanies it — by design.** |

### 18.3.10 `DeniablePayload` (§5.2.1)

The plaintext sealed into a `DeniableMessage.ct` — the structural twin of `Payload` (§18.3.5)
**with the identity signature removed**, because the mode's authentication is the ratchet MAC,
not a signature. It carries the real content `kind` (the envelope `kind` is the fixed `0x0b`).

```cddl
DeniablePayload = {
  1 => ik-pub,          ; from      sender IK (bound by X3DH, NOT by a signature — repudiable)
  2 => u8,              ; kind      the real content kind (mail/chat/reaction/…, §2.3)
  3 => Headers,         ; headers   (§18.3.6)
  4 => Body,            ; body      (§18.3.6)
  5 => [* hash],        ; refs      threading (§2.3)
  6 => [* Attachment],  ; attach    (§18.3.7)
  ? 7 => ts,            ; expires   requested client-enforced expiry
  ; NO signature field. A DeniablePayload carrying one MUST be rejected (0x040F).
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `from` | 1 | `ik-pub` | MUST | Sender `IK`. It is **authenticated by the X3DH/PQXDH key agreement** (the shared secret can only be derived by the holder of `ik_a`'s identity key and the consumed prekeys), **not** by a signature — so the recipient is convinced of the sender *to itself* while retaining no transferable proof of authorship (repudiation). MUST match the pinned identity for a known contact (§3.4). |
| `kind` | 2 | `u8` | MUST | The real content kind (§2.3), since the envelope `kind` is the fixed transport tag `0x0b`. A node MUST NOT act on a `kind` it cannot validate. |
| `headers` | 3 | `Headers` | MUST | As §18.3.6. MAY be empty. The `sensitive` flag (key 6) is honored as in any MOTE. |
| `body` | 4 | `Body` | MUST | As §18.3.6. MAY be empty. |
| `refs` | 5 | `[* hash]` | MUST (MAY be empty) | Threading references (§2.3). |
| `attach` | 6 | `[* Attachment]` | MUST (MAY be empty) | Attachments (§18.3.7); per-file keys travel in `Attachment.key` as always. |
| `expires` | 7 | `ts` | OPTIONAL | Requested client-enforced expiry (§2.4). |
| ~~`sig`~~ | — | — | **FORBIDDEN** | A `DeniablePayload` MUST NOT carry any signature field. Its presence would make the transcript attributable and defeat the mode; a decoder that finds one MUST reject the message (`ERR_DENIABLE_SIGNATURE_PRESENT`, `0x040F`). |

### 18.3.11 `GatewayAttestation` (§7.2a, §7.8)

The **domain-anchored attestation** a legacy gateway signs when it bridges an inbound legacy
message into the mesh (§7.2 step 4, §7.2a). §7.2a already REQUIRES this attestation and its
DNS/KT-anchored key; this object is its **normative wire form**. One or more travel in
`Payload.provenance` (§18.3.5 key 9), sealed inside the encrypted payload so they are visible
**only to the recipient** (§7.8, §6.8). It is the seed of the whole transport-path provenance
model: its **presence** proves the message was **gateway-touched / legacy-origin** (plaintext at
a gateway before the mesh); its **absence** proves **pure-mesh** (never plaintext at a gateway,
§7.8).

```cddl
GatewayAttestation = {
  0 => 1,               ; disc       1 = legacy-inbound bridge attestation (§7.2a); other values reserved
  1 => tstr,            ; domain     the domain whose `_dmtap-gw` key signs (§7.2a); the recipient-domain
                        ;            entry MUST be the recipient's own domain
  2 => tstr,            ; selector   `_dmtap-gw` selector naming the attestation key in DNS/KT
  3 => ts,              ; recv_at    gateway receipt time T ("received via gateway G at T")
  4 => hash,            ; msg_digest binds THIS attestation to the wrapped legacy message (§18.9.11)
  ? 5 => tstr,          ; legacy_from  SMTP MAIL FROM, recipient-visible (sealed); informational
  ? 6 => u8,            ; seq        position in a multi-gateway chain, 0-based; absent ⇒ 0 (single gateway)
  7 => sig-val,         ; sig        signature by the `<selector>._dmtap-gw.<domain>` attestation key (§18.9.11)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `disc` | 0 | `1` | MUST | Selects the legacy-inbound bridge attestation. Other discriminator values are reserved for future attestation kinds; an unknown value MUST be treated as an unverifiable attestation (`ERR_GATEWAY_ATTESTATION_INVALID`, `0x0601`), never silently ignored. |
| `domain` | 1 | `tstr` | MUST | The domain whose `_dmtap-gw` attestation key (§7.2a) signs this entry. For the entry that bridged mail addressed to the recipient, `domain` MUST equal the **recipient's own domain** and is verified against **that** domain's `_dmtap-gw` record (§7.2a). Chained entries under other domains (§7.8) are verified only if their domain is in the recipient's explicitly-trusted gateway set, else surfaced as an *unverified* hop. |
| `selector` | 2 | `tstr` | MUST | The `_dmtap-gw` selector naming the attestation public key: the verifier fetches `<selector>._dmtap-gw.<domain>` TXT (`v=dmtapgw1; k=…`, §7.2a), optionally KT-anchored, and checks `sig` against `k`. A key not published there is untrusted (`ERR_GATEWAY_ATTESTATION_KEY_UNTRUSTED`, `0x0602`). |
| `recv_at` | 3 | `ts` | MUST | Gateway receipt time `T`. Part of the signed statement "received via gateway `domain` at `T`". |
| `msg_digest` | 4 | `hash` | MUST | `0x1e ‖ BLAKE3-256(rfc5322_bytes)` over the exact legacy bytes the gateway wrapped (§18.9.11). The recipient MUST recompute it from the decrypted body and reject a mismatch (`0x0601`): this **binds the attestation to this one message**, so a valid attestation cannot be lifted onto different content. |
| `legacy_from` | 5 | `tstr` | OPTIONAL | The SMTP `MAIL FROM` of the legacy sender. Recipient-visible (it is the recipient's own inbound mail) but sealed inside `Payload` so no intermediary sees it. Informational — display only; authenticity of the legacy leg is DKIM/SPF/DMARC at the gateway (§7.2), not this field. |
| `seq` | 6 | `u8` | OPTIONAL | 0-based position in a multi-gateway chain (§7.8); absent ⇒ `0`. Entries in `Payload.provenance` are in temporal (ascending `seq`) order; the recipient-domain bridge is the last entry. |
| `sig` | 7 | `sig-val` | MUST | Signature by the domain-anchored `_dmtap-gw` attestation key over the preimage of §18.9.11. A failure to verify ⇒ `0x0601`; the attestation MUST NOT be accepted, and an inbound legacy message with no valid attestation MUST NOT be surfaced as legacy-origin-verified (§7.2a, §19.3.1). |

The gateway's `_dmtap-gw` attestation key is **distinct from** the gateway's delegated **DKIM**
key (§7.3): DKIM authenticates the *outbound* legacy leg to the legacy world, while this
attestation authenticates the *inbound* bridge to the mesh recipient. Neither is the user's DMTAP
identity key, which the gateway never holds (§7.3).

### 18.3.12 `GatewayAliasMap` (§7.10)

The gateway-held **native ↔ legacy alias mapping** for the **random** alias mode (§7.10.2). It is
**gateway-local state** (persisted/optionally synced within the gateway), **not** mesh-transmitted
and **not** part of any user's identity — the alias is a rotatable bridge pointer, the native
address is the anchor (§7.10.4). The **encoded** alias mode needs **no** row: the alias
`localpart.nativedomain@gateway.domain` is a self-describing reversible transform (§7.10.2), so its
"mapping" is the decode function, not a table.

```cddl
GatewayAliasMap = {
  1 => tstr,        ; alias         the gateway-local localpart (<rand>, or the encoded localpart for logging)
  2 => tstr,        ; native        the native DMTAP address "localpart@nativedomain" it maps to
  3 => u8,          ; mode          1 = encoded (self-describing), 2 = random (this table row)
  ? 4 => tstr,      ; correspondent legacy peer this alias is scoped/burnable to (mode 2)
  5 => ts,          ; created
  ? 6 => ts,        ; expires       absent ⇒ no expiry (§16.11)
  ? 7 => bool,      ; burned        true ⇒ retired; no longer maps (mode 2)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `alias` | 1 | `tstr` | MUST | The gateway-local local-part the legacy world addresses. For `mode = 2` a high-entropy random string; a lookup miss / expired / `burned` row on inbound ⇒ `ERR_GATEWAY_ALIAS_UNMAPPED` (`0x0605`, RETURN_SENDER_SMTP `550 5.1.1`, §7.10.3). |
| `native` | 2 | `tstr` | MUST | The native `localpart@nativedomain` this alias resolves to. The gateway rewrites the legacy reply-path to `alias@gateway.domain` (§7.10.1) and maps back to `native` on inbound (§7.10.3). |
| `mode` | 3 | `u8` | MUST | `1` encoded (near-stateless; the row is advisory/logging — the truth is the reversible encoding, §7.10.2; a non-decodable encoded local-part ⇒ `ERR_GATEWAY_ALIAS_ENCODING_INVALID` `0x0606`), `2` random (this row **is** the mapping). |
| `correspondent` | 4 | `tstr` | OPTIONAL | The single legacy peer a `mode = 2` alias is scoped to — a per-correspondent, burnable "Hide-My-Email"-style alias (§7.10.2). |
| `created` | 5 | `ts` | MUST | Row creation time. |
| `expires` | 6 | `ts` | OPTIONAL | Expiry; absent ⇒ no expiry (§16.11). A lookup after `expires` ⇒ `0x0605`. |
| `burned` | 7 | `bool` | OPTIONAL | `true` retires the alias (user burned it); a burned alias no longer maps (`0x0605`) but is retained to avoid reuse. |

---

## 18.4 Identity-layer objects (§1)

### 18.4.1 `Identity` (§1.3)

The published, signed, versioned identity; its `id` (a `hash` of this object, §18.9.4) is the
anchor everyone pins.

```cddl
Identity = {
  1  => [+ u8],                 ; suites    supported suites, preference-ordered (a SET)
  2  => { + u8 => ik-pub },     ; iks       identity public key per suite
  3  => u64,                    ; version   monotonically increasing
  4  => [* DeviceCert],         ; devices
  5  => KeyPackageBundleRef,    ; keypkgs   location+hash of current KeyPackage bundle (§18.4.3)
  6  => hash,                   ; recovery  hash of the current RecoveryPolicy (§18.4.4)
  7  => [* tstr],               ; names     canonical human name(s) (§3)
  ? 8  => hash,                 ; prev      hash of previous Identity version (hash chain)
  9  => ts,                     ; ts
  10 => [+ sig-val],            ; sig       ONE signature per suite in `suites`, over the body
  ? 11 => KeyPackageBundleRef,  ; deniable_prekeys  OPTIONAL: X3DH/PQXDH prekey bundle (§5.2.1, §18.4.8)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `suites` | 1 | `[+ u8]` | MUST | The suites this identity supports, **as a preference-ordered set** (§1.3). A sender MUST use the highest suite both parties support; empty intersection ⇒ delivery fails closed (no downgrade). |
| `iks` | 2 | `{+ u8 => ik-pub}` | MUST | Map from each suite in `suites` to that suite's identity public key. Every suite in `suites` MUST have exactly one entry, and vice-versa. |
| `version` | 3 | `u64` | MUST | Monotonic version. A verifier MUST reject a version ≤ the last one it pinned for this identity (rollback defense, §3.3). |
| `devices` | 4 | `[* DeviceCert]` | MUST (MAY be empty) | The identity's device certificates (§18.4.2), each signed by IK. |
| `keypkgs` | 5 | `KeyPackageBundleRef` | MUST | Location + hash of the current KeyPackage bundle (§18.4.3). Named `keypkgs` throughout, matching §1.3 and §5.3 (the earlier `prekeys` name is retired; §18.11 item 1). |
| `recovery` | 6 | `hash` (`RecoveryPolicyRef`) | MUST | Hash of the current `RecoveryPolicy` (§18.4.4). Resolved via the mesh/KT log. |
| `names` | 7 | `[* tstr]` | MUST (MAY be empty) | Human name(s), e.g. `"abc@def.com"`, `"@handle"` (§3.9). A list ⇒ aliases. These are **self-asserted**: an identity MAY list any string, including a victim's address, so a listed name proves **nothing** on its own. A verifier MUST trust/display a name **only after** confirming the forward `name → ik` binding (DNS + KT, §3.3–3.5) resolves back to *this* key (§3.9.4); a name that does not verify back MUST be rendered as unverified, never as an authenticated address. Every *verified* alias resolves to the same key. |
| `prev` | 8 | `hash` | OPTIONAL | Hash of the previous `Identity` version; absent only for the genesis version. Chains versions into the KT-mirrored history (§3.5). |
| `ts` | 9 | `ts` | MUST | Publication timestamp. |
| `sig` | 10 | `[+ sig-val]` | MUST | **One signature per suite in `suites`**, in the same order, each over the body preimage (§18.9.3). A verifier trusting either the classical or PQ key can validate; it MUST reject an Identity whose highest offered suite it cannot validate (§1.3). Note `sig` remains key `10` even though the OPTIONAL `deniable_prekeys` is key `11`; the signing preimage is `Identity ∖ {10}` and so covers key `11` when present (§18.9.3). |
| `deniable_prekeys` | 11 | `KeyPackageBundleRef` | OPTIONAL | Location + hash of the identity's `DeniablePrekeyBundle` (§18.4.8) for the optional deniable 1:1 mode (§5.2.1). Same shape as `keypkgs` (§18.4.3). Absent ⇒ the identity does not offer deniable sessions; the default MLS path is unaffected. |

### 18.4.2 `DeviceCert` (§1.2)

```cddl
DeviceCert = {
  1 => suite,           ; suite
  2 => ik-pub,          ; ik          root identity public key
  3 => ik-pub,          ; device_key  device signing public key
  4 => tstr,            ; label       "phone", "home-box", …
  5 => ts,              ; created
  ? 6 => ts,            ; expires
  7 => [+ tstr],        ; caps        capability strings
  8 => sig-val,         ; sig         IK over the body (§18.9)
  ? 9 => key-protection, ; key_protection  keystore class holding device_key (§1.2a)
  ? 10 => bytes,        ; attestation  OPTIONAL platform key-attestation evidence (§1.2a)
}
key-protection = "software" / "tpm" / "secure-enclave" / "strongbox" / "tee"
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `suite` | 1 | `suite` | MUST | Suite of `device_key` and `sig`. |
| `ik` | 2 | `ik-pub` | MUST | The root identity public key that issued this cert; MUST match `Identity.iks[suite]`. |
| `device_key` | 3 | `ik-pub` | MUST | The device's signing public key (day-to-day signing/auth). |
| `label` | 4 | `tstr` | MUST | Human device label. UTF-8. |
| `created` | 5 | `ts` | MUST | Issuance time. |
| `expires` | 6 | `ts` | OPTIONAL | Expiry; after it, verifiers MUST NOT accept `device_key`. Absent ⇒ no expiry. |
| `caps` | 7 | `[+ tstr]` | MUST | Capability set gating participation: `"send"`, `"recv"`, `"relay"`, `"mix"`, `"gateway"`, `"admin"` (§1.2). An `admin` device counts as only **one factor** toward `rotate_threshold` and MAY NOT unilaterally change recovery (§1.4). |
| `sig` | 8 | `sig-val` | MUST | IK signature over the body (§18.9.3). Covers keys `9`/`10` when present (the preimage is `DeviceCert ∖ {8}`). |
| `key_protection` | 9 | `key-protection` | OPTIONAL | The keystore class holding `device_key`: `"software"` or a hardware class (`"tpm"`/`"secure-enclave"`/`"strongbox"`/`"tee"`, §1.2a). A relying context (group admit, org provisioning) MAY require a hardware class. Absent ⇒ unstated (treat as `"software"` for policy). |
| `attestation` | 10 | `bytes` | OPTIONAL | Platform key-attestation evidence that `device_key` is hardware-resident and **non-exportable** (Android Key Attestation / Apple / TPM `AK` quote / FIDO), §1.2a. Verified against the platform's **attestation root** — a vendor trust anchor (a disclosed TTP, §1.2a) — out of band. A context requiring attestation rejects a device whose evidence is absent/invalid (`ERR_DEVICE_ATTESTATION_INVALID`, `0x0116`). Evidence has a **validity window**: a context MUST treat evidence older than the re-attestation cadence (≤ 90 days, §16.9) or past its own expiry, or chaining only to a retired attestation root, as **expired** (`ERR_DEVICE_ATTESTATION_EXPIRED`, `0x0118`) and require re-attestation over the same key (§1.2a). Advisory hardening — never a substitute for the §1.4 authorization authority. |

### 18.4.3 `KeyPackageBundleRef` (§1.3, §5.3)

Locates and pins the identity's **whole** published KeyPackage bundle (distinct from
`KeyPackageRef`, §18.3.4, which names one consumed package).

```cddl
KeyPackageBundleRef = {
  1 => tstr,            ; loc     mesh/relay locator for the bundle
  2 => hash,            ; id      content address of the bundle
  ? 3 => [+ u8],        ; suites  suites advertised by the bundle
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `loc` | 1 | `tstr` | MUST | Locator (mesh key, relay URL, or DNS `keypkgs=` value, §3.2) where the bundle is fetched. |
| `id` | 2 | `hash` | MUST | Content address pinning the exact bundle bytes; a fetched bundle MUST hash to it. |
| `suites` | 3 | `[+ u8]` | OPTIONAL | Suites the bundle offers; if present, MUST be a subset of `Identity.suites`. |

### 18.4.4 `RecoveryPolicy`, `RecoveryMethod`, `Threshold` (§1.4)

```cddl
RecoveryPolicyRef = hash            ; hash of a RecoveryPolicy (Identity.recovery)

RecoveryPolicy = {
  1 => suite,
  2 => ik-pub,                      ; ik
  3 => u64,                         ; version
  4 => [+ RecoveryMethod],          ; methods
  5 => Threshold,                   ; recover_threshold  what regains access
  6 => Threshold,                   ; rotate_threshold   higher bar to change THIS policy
  ? 7 => hash,                      ; prev  hash chain
  8 => ts,
  9 => sig-val,                     ; sig  by IK, OR by a satisfied rotate_threshold quorum
}

RecoveryMethod = PhraseMethod / DeviceMethod / SocialMethod

PhraseMethod = { 0 => 1, 1 => bytes }               ; recovery_key from a SLIP-0039/BIP39 phrase
DeviceMethod = { 0 => 2, 1 => ik-pub, 2 => tstr }   ; device_key, label
SocialMethod = { 0 => 3, 1 => [+ ik-pub], 2 => u8 } ; guardians, threshold (VSS/FROST shares)

Threshold      = { 1 => [+ MethodPredicate] }       ; any_of — satisfied if ANY predicate holds
MethodPredicate = { 1 => method-type, 2 => uint }   ; method + required count
method-type    = "phrase" / "device" / "social" / "ik"
```

`method-type` maps the §1.4 prose predicates onto factor kinds: `"phrase"` = `Phrase`,
`"device"` = `Devices(n)`, `"social"` = `Guardians(n)` (guardian shares), and `"ik"` = `Ik` (the
root key itself; `count` is `1`). The `"ik"` predicate has **no** corresponding entry in
`methods` — it is satisfied by an `IK` signature directly, so it is the one predicate that does
not name a `RecoveryMethod`.

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `RecoveryPolicy` | `suite` | 1 | `suite` | MUST | Suite of `ik`/`sig`. |
| | `ik` | 2 | `ik-pub` | MUST | The identity this policy governs. |
| | `version` | 3 | `u64` | MUST | Monotonic; reject ≤ last pinned. |
| | `methods` | 4 | `[+ RecoveryMethod]` | MUST | The recovery factors (phrase/device/social). |
| | `recover_threshold` | 5 | `Threshold` | MUST | Predicate that regains access. |
| | `rotate_threshold` | 6 | `Threshold` | MUST | Predicate to change **this** policy; MUST be **≥** `recover_threshold` (§1.4 rule 2) — a single recovered factor MUST NOT be able to rewrite the policy. |
| | `prev` | 7 | `hash` | OPTIONAL | Hash of the previous policy version (chain). |
| | `ts` | 8 | `ts` | MUST | Timestamp. |
| | `sig` | 9 | `sig-val` | MUST | Signed by IK (proactive) OR by a satisfied `rotate_threshold` quorum (reactive, mid-recovery). Unauthenticated changes are invalid (§1.4 rule 1). Rotating a method out MUST re-key the underlying secret (rule 3), not merely edit the list. |
| `PhraseMethod` | disc | 0 | `1` | MUST | Phrase factor. |
| | `recovery_key` | 1 | `bytes` | MUST | Key derived from a mnemonic (SLIP-0039 RECOMMENDED, §1.4). |
| `DeviceMethod` | disc | 0 | `2` | MUST | Device factor. |
| | `device_key` | 1 | `ik-pub` | MUST | The device signing key acting as a recovery factor. |
| | `label` | 2 | `tstr` | MUST | Human label. |
| `SocialMethod` | disc | 0 | `3` | MUST | Social/guardian factor. |
| | `guardians` | 1 | `[+ ik-pub]` | MUST | Guardian keys holding VSS shares. Changing this set MUST trigger **redistribution/resharing** (§1.4 rule 3), not proactive refresh. FROST (RFC 9591) RECOMMENDED so the secret is never reassembled. |
| | `threshold` | 2 | `u8` | MUST | Number of guardian shares required (`M` of `N`). |
| `Threshold` | `any_of` | 1 | `[+ MethodPredicate]` | MUST | Disjunction: satisfied if **any** listed predicate is met (e.g. 1 phrase OR 2 devices OR 2 guardians, §1.4). |
| `MethodPredicate` | `method` | 1 | `method-type` | MUST | One of `"phrase"`, `"device"`, `"social"`, `"ik"` — mapping the §1.4 predicates `Phrase`/`Devices(n)`/`Guardians(n)`/`Ik`. `"ik"` is satisfied by an `IK` signature and names no `RecoveryMethod`. |
| | `count` | 2 | `uint` | MUST | Number of factors of `method` required to satisfy this predicate (≥ 1; `1` for `"ik"`). |

### 18.4.5 `KeyRotation` (§1.5)

Cross-signed record authorizing a new root key. Distributed via the `Identity` chain and the KT
log.

```cddl
KeyRotation = {
  1 => suite,
  2 => ik-pub,          ; old_ik   the retiring root key
  3 => ik-pub,          ; new_ik   the incoming root key
  4 => tstr,            ; reason   free-text reason (e.g. "compromise", "pq-migration")
  5 => ts,
  ? 6 => hash,          ; prev
  7 => sig-val,         ; sig            by OLD_IK over (old_ik, new_ik, reason, ts)
  ? 8 => sig-val,       ; rotate_quorum  OPTIONAL rotate_threshold co-signature (§1.5 path (a))
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `suite` | 1 | `suite` | MUST | Suite of the signature (`old_ik`'s suite). During PQ migration this may differ from the new suite. |
| `old_ik` | 2 | `ik-pub` | MUST | The retiring root key; MUST be the currently-pinned IK. |
| `new_ik` | 3 | `ik-pub` | MUST | The incoming root key. A verifier accepts it **only** via this valid chain from a pinned IK, or by explicit out-of-band re-verification (§1.5). |
| `reason` | 4 | `tstr` | MUST | Human/audit reason string. |
| `ts` | 5 | `ts` | MUST | Timestamp. |
| `prev` | 6 | `hash` | OPTIONAL | Hash chaining into identity history. |
| `sig` | 7 | `sig-val` | MUST | Signature by **`old_ik`** over the body (§18.9.3), proving continuity. |
| `rotate_quorum` | 8 | `sig-val` | OPTIONAL | A **`rotate_threshold` quorum co-signature** over the body (`det_cbor(KeyRotation ∖ {7,8})`, verified under the recovery group per §1.4), authorizing the rotation under §1.5 **path (a)** (immediate effect). **When the identity has a published `RecoveryPolicy`, a `KeyRotation` MUST carry a valid `rotate_quorum` OR be published to KT and take effect only after the §16.8 veto/delay window (path (b))** — an `old_ik`-alone rotation satisfying neither is rejected/held (`ERR_KEYROTATION_UNAUTHORIZED`, `0x0121`, §1.5, §21.3). Absent for an identity with no published `RecoveryPolicy`, where `old_ik` alone suffices. |

### 18.4.6 `MoveRecord` (§1.6)

Rebinds the human name while preserving the key.

```cddl
MoveRecord = {
  1 => suite,
  2 => ik-pub,          ; ik
  3 => tstr,            ; from   "abc@old.com"
  4 => tstr,            ; to     "abc@new.com" or a self-sovereign name (§3.6)
  5 => ts,
  ? 6 => hash,          ; prev
  7 => sig-val,         ; sig    by IK
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `suite` | 1 | `suite` | MUST | Suite of `ik`/`sig`. |
| `ik` | 2 | `ik-pub` | MUST | The identity performing the move; the key is unchanged by a move. |
| `from` | 3 | `tstr` | MUST | The old canonical name. |
| `to` | 4 | `tstr` | MUST | The new canonical name (or self-sovereign name). |
| `ts` | 5 | `ts` | MUST | Timestamp. |
| `prev` | 6 | `hash` | OPTIONAL | Hash chain. |
| `sig` | 7 | `sig-val` | MUST | Signature by IK (§18.9.3). Contacts route by key and verify against IK, so a forged move cannot redirect them (§1.6). |

---

### 18.4.7 `DomainDirectory` and `DirEntry` (§3.10.3)

The signed, versioned, KT-logged enumeration of a domain's member (and group) bindings — the org
directory / GAL (§3.10.3). Signed by the **domain authority** (§3.10.1, threshold-held). It is a
convenience **index**: each entry's `name → ik` MUST still verify forward via DNS + KT (§3.3–3.5,
§3.9.4) before use, so the directory can enumerate but never *forge* a binding.

```cddl
DomainDirectory = {
  1 => suite,
  2 => tstr,             ; domain                 "abc.com"
  3 => ik-pub,           ; authority              domain authority IK (threshold-held, §3.10.1)
  4 => u64,              ; version                monotonic
  5 => dir-visibility,   ; membership_visibility
  6 => [* DirEntry],     ; entries
  ? 7 => hash,           ; prev                   hash chain (KT-logged, §3.5)
  8 => ts,
  9 => sig-val,          ; sig                    by the domain authority (§18.9.3)
}

DirEntry = {
  1 => tstr,             ; name       "alice@abc.com" (or a group name, §5.8.7)
  2 => ik-pub,           ; ik         member/group identity key
  3 => hash,             ; id         content addr of the member's current Identity (§18.4.1)
  4 => member-custody,   ; custody
  ? 5 => [* tstr],       ; roles      org roles / standing-group memberships (informative)
  6 => ts,               ; added
  ? 7 => alloc-tier,     ; alloc      provisioning tier disclosure (§3.11.2), informative
}

dir-visibility = "public" / "members-only"
member-custody = "sovereign" / "org-managed"
alloc-tier     = "random" / "vanity" / "byod"     ; tiers 0 / 1 / 2 (§3.11.2)
```

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `DomainDirectory` | `domain` | 2 | `tstr` | MUST | The administered domain; the `authority` key is pinned via `_dmtap.<domain>` (§3.2). |
| | `authority` | 3 | `ik-pub` | MUST | Domain authority IK (§3.10.1). SHOULD be threshold-held by the domain-owner/domain-admin set (§5.8.6). A verifier MUST reject a directory not signed by the pinned authority (`ERR_DOMAIN_DIRECTORY_SIG_INVALID`, `0x0113`). |
| | `version` | 4 | `u64` | MUST | Monotonic; reject ≤ last pinned (rollback defense, `0x0105`, same rule as `Identity`). |
| | `membership_visibility` | 5 | `dir-visibility` | MUST | `"public"` (world-listable) or `"members-only"` (entries served only to authenticated members) (§3.10.3). |
| | `entries` | 6 | `[* DirEntry]` | MUST (MAY be empty) | Member/group bindings. Each MUST verify forward against DNS + KT (§3.9.4) before use — the directory indexes, it does not attest (`ERR_DIRECTORY_ENTRY_UNVERIFIED`, `0x0114`). MAY be sharded/paged for large orgs; the KT-logged signed root is authoritative over the set. |
| | `prev` | 7 | `hash` | OPTIONAL | Hash chain; the chain is appended to key transparency (§3.5), so directory history is append-only and auditable. |
| | `ts` | 8 | `ts` | MUST | Publication time. |
| | `sig` | 9 | `sig-val` | MUST | Signature by the domain authority (§18.9.3, DS-tag `DMTAP-v0/domain-directory`); a threshold-quorum signature where the authority key is threshold-held (§3.10.1, §5.8.6). |
| `DirEntry` | `name` | 1 | `tstr` | MUST | The member or group name under the domain (§3.10.2, §5.8.7). |
| | `ik` | 2 | `ik-pub` | MUST | The member's/group's identity key; MUST match the forward DNS + KT binding (§3.9.4) or the entry is unverified (`0x0114`). |
| | `id` | 3 | `hash` | MUST | Content address of the member's current `Identity` object (§18.4.1). |
| | `custody` | 4 | `member-custody` | MUST | `"sovereign"` (member holds their own key; the org cannot access, §3.10.2(a)) or `"org-managed"` (org holds/escrows the key — a disclosed §6.6-style limit, §3.10.2(b)). An `"org-managed"` entry MUST be rendered as such; presenting one as sovereign MUST fail closed (`ERR_ORG_MANAGED_UNDISCLOSED`, `0x0115`). |
| | `roles` | 5 | `[* tstr]` | OPTIONAL | Informative org roles / standing-group memberships (§13.5.1, §5.8.7); authority for a role is the capability (§13.5.1), not this hint. |
| | `added` | 6 | `ts` | MUST | When the entry was published. |
| | `alloc` | 7 | `alloc-tier` | OPTIONAL | Informative disclosure of the address's **provisioning tier** (§3.11.2): `"random"` (tier 0, provider-assigned word-combo), `"vanity"` (tier 1, reserved localpart), `"byod"` (tier 2, bring-your-own-domain). Advisory only — it never changes how the binding is verified (still forward DNS + KT, §3.9.4); it lets a client render "how this address was allocated" honestly. |

### 18.4.8 `DeniablePrekeyBundle` (§5.2.1)

The published X3DH/PQXDH prekeys for the optional deniable 1:1 mode (§5.2.1), located via
`Identity.deniable_prekeys` (§18.4.1). Analogous to the MLS KeyPackage bundle (§18.4.3) but for
the Signal-style channel; replenished and rate-limited per §16.9.

```cddl
DeniablePrekeyBundle = {
  1 => suite,           ; suite     0x01 (X3DH) or 0x02 (PQXDH)
  2 => ik-pub,          ; ik        the identity these prekeys belong to (Ed25519 IK; used for AD binding + to authorize idk, NOT for DH)
  11 => bytes,          ; idk       long-term deniable-identity DH key — a DEDICATED X25519 public key, NOT derived from IK (§5.2.1(a))
  12 => sig-val,        ; idk_sig   IK-authorized device-key signature over `idk` (DS-tag DMTAP-v0/deniable-idk, §18.9.10)
  3 => bytes,           ; spk       signed prekey — X25519 DH public key
  4 => sig-val,         ; spk_sig   device-key signature over `spk` (§5.2.1(a))
  5 => [* bytes],       ; opks      one-time prekeys (X25519 DH publics), consumed per session
  ? 6 => bytes,         ; last_kem  (PQ) signed last-resort ML-KEM encapsulation key (suite 0x02)
  ? 7 => [* bytes],     ; okems     (PQ) one-time ML-KEM encapsulation keys
  8 => u64,             ; version   monotonic; reject older-or-equal (rollback defense)
  9 => ts,              ; ts
  10 => sig-val,        ; sig       device-key signature over the whole bundle (§18.9.10)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `suite` | 1 | `suite` | MUST | `0x01` X3DH or `0x02` PQXDH; governs the DH group and (PQ) the ML-KEM parameters. |
| `ik` | 2 | `ik-pub` | MUST | The identity offering deniable sessions (its **Ed25519 `IK`**); MUST match the pinned identity. `IK` is used **only** for the AD identity binding (`AD = IK_A ‖ IK_B`, §18.3.9) and to authorize `idk` — it is **never** the X3DH DH input. This keeps `IK` cold and hardware-buildable (a usage-fixed Secure-Enclave/TPM/StrongBox signing key need never also perform DH). |
| `idk` | 11 | `bytes` | MUST | The **dedicated long-term deniable-identity DH key** — a **standalone X25519 public key**, provisioned once and reused across sessions, that serves as this identity's X3DH/PQXDH *long-term identity DH key*. It is **NOT** derived from `IK` by XEdDSA (the retired construction); a separate DH key means the signing `IK` never needs sign-and-DH on one key material. A verifier MUST reject a bundle whose `idk_sig` does not authorize `idk` under `ik`'s `Identity` (`ERR_DENIABLE_PREKEY_INVALID_OR_EXHAUSTED`, `0x040B`). |
| `idk_sig` | 12 | `sig-val` | MUST | Signature by an `IK`-authorized **device key** over the raw `idk` bytes (DS-tag `DMTAP-v0/deniable-idk`, §18.9.10). This **certifies the long-term deniable-identity DH key to the identity** — it signs a *public DH key*, never any message, so it is exactly as deniability-preserving as `spk_sig`. It is the certification that replaces the old "`IK` *is* the identity DH key via XEdDSA" binding. |
| `spk` | 3 | `bytes` | MUST | The signed prekey — an X25519 DH public key. |
| `spk_sig` | 4 | `sig-val` | MUST | Signature by an `IK`-authorized **device key** over `spk` (DS-tag `DMTAP-v0/deniable-spk`, §18.9.10). Together with `idk_sig`, these are the **only** signatures that participate in a deniable session, and they both sign *public prekeys*, never any message — which is exactly why deniability is preserved (§5.2.1(a)). |
| `opks` | 5 | `[* bytes]` | MUST (MAY be empty) | One-time prekeys (X25519 DH publics). Each is consumed (marked spent) by one `DeniableInit.opk_ref`. Empty ⇒ only the signed prekey / last-resort path is available (rate-limited, §16.9). |
| `last_kem` | 6 | `bytes` | OPTIONAL | PQXDH signed last-resort ML-KEM encapsulation key; present under `suite = 0x02`. |
| `okems` | 7 | `[* bytes]` | OPTIONAL | PQXDH one-time ML-KEM encapsulation keys, consumed by `DeniableInit.kem_ref`. |
| `version` | 8 | `u64` | MUST | Monotonic; a verifier MUST reject a bundle whose `version` is older-or-equal to one already seen (rollback defense, same rule as `Identity.version`). |
| `ts` | 9 | `ts` | MUST | Publication time. |
| `sig` | 10 | `sig-val` | MUST | Signature by an `IK`-authorized device key over the body (§18.9.10, DS-tag `DMTAP-v0/deniable-prekeys`). Authenticates the *bundle*; an invalid signature or an exhausted bundle is `ERR_DENIABLE_PREKEY_INVALID_OR_EXHAUSTED` (`0x040B`). |

### 18.4.9 `SignedTreeHead` (KT, §3.5)

The **signed tree head (STH)** of a key-transparency log — the wire object §3.5 REQUIRES but did
not previously give a CBOR form. DMTAP's KT is an **RFC 6962-profiled** append-only Merkle log;
this is the STH a verifier fetches, gossips (§3.5.2(a)), and checks freshness on. It is the KT
analog of `MixDirectory` (§18.5.3): signed by the log's own key, versioned by `tree_size`.

```cddl
SignedTreeHead = {
  1 => suite,           ; suite       signature/hash suite of this log
  2 => bytes,           ; log_id      the log's identity = its signing public key (no central log registry, §21.19)
  3 => u64,             ; tree_size   number of log entries (the RFC 6962 tree size)
  4 => ts,              ; timestamp   STH issuance time (freshness / MMD, §3.5.2(a))
  5 => hash,            ; root_hash   Merkle tree hash of the first `tree_size` entries (prefix ‖ digest)
  6 => sig-val,         ; sig         the LOG's signature over the head (DS-tag DMTAP-v0/kt-sth, §18.9.13)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `suite` | 1 | `suite` | MUST | Suite governing `sig` and the tree hash. |
| `log_id` | 2 | `bytes` | MUST | The log's public signing key — the log **is** its key (§21.19); a verifier MUST have pinned it (the `kt=` DNS/SVCB anchor, §3.2) and MUST reject an STH not signed by it. |
| `tree_size` | 3 | `u64` | MUST | Count of leaves. Two validly-signed STHs of one `log_id` with **equal `tree_size` but differing `root_hash`** are transferable proof of equivocation (`0x0110`/`0x0107`, §3.5.2(d)). |
| `timestamp` | 4 | `ts` | MUST | Issuance time; an STH older than the STH-freshness window (§16.2) is stale (`ERR_KT_STH_STALE`, `0x0112`) — the freeze-attack defense. |
| `root_hash` | 5 | `hash` | MUST | The RFC 6962 Merkle Tree Hash over the first `tree_size` leaves, using the log's suite hash with the §18.9.5 domain-separated leaf/node prefixes. |
| `sig` | 6 | `sig-val` | MUST | The log's signature over the head (§18.9.13). Signature failure ⇒ `ERR_KT_PROOF_INVALID` (`0x0108`). |

**Identity-entry leaf-hash rule (normative).** A KT log leaf for a DMTAP identity event commits to
the **exact bytes** of that event so a leaf can never be reinterpreted. The leaf **data** for an
`Identity` (§18.4.1) publication is the deterministic CBOR of a fixed 4-tuple
`KTLeaf = [ name, ik, version, identity_id ]` — the primary `name` (tstr), the identity key `ik`
(`ik-pub`), the `Identity.version` (u64), and `identity_id` (the content address of the signed
`Identity`, §18.9.4) — and the **leaf hash** is:

```
leaf_data = det_cbor([ name, ik, version, identity_id ])
leaf_hash = 0x1e ‖ BLAKE3-256( 0x00 ‖ leaf_data )      ; RFC 6962 leaf prefix 0x00 (cf. §18.9.5)
```

A verifier recomputes `leaf_hash` from the pinned/resolved `Identity` and MUST reject a proof whose
committed leaf does not equal it (`ERR_KT_LEAF_HASH_MISMATCH`, `0x0117`) — the log **indexes**
bindings, it does not get to redefine them. `RecoveryPolicy`/`KeyRotation`/`MoveRecord` leaves use
the same rule with their own content address in place of `identity_id`.

### 18.4.10 `InclusionProof` (KT, §3.5)

An RFC 6962 **Merkle audit path** proving a specific `leaf_hash` (§18.4.9) is the `leaf_index`-th
leaf of the tree committed by an STH's `root_hash`. Carries **no signature** — it is verified
*against* the STH root, not signed on its own (like `ProvenanceRecord`, §18.9.12).

```cddl
InclusionProof = {
  1 => u64,             ; tree_size    the STH tree_size this proof is relative to
  2 => u64,             ; leaf_index   0-based index of the leaf in the tree
  3 => hash,            ; leaf_hash    the leaf being proven (§18.4.9 rule)
  4 => [* hash],        ; audit_path   sibling hashes bottom-to-top (RFC 6962 audit path; MAY be empty for a size-1 tree)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `tree_size` | 1 | `u64` | MUST | The `SignedTreeHead.tree_size` this path reconstructs to; MUST match the STH the verifier holds. |
| `leaf_index` | 2 | `u64` | MUST | 0-based leaf position; `< tree_size`. |
| `leaf_hash` | 3 | `hash` | MUST | The leaf being proven, computed by the §18.4.9 Identity-entry rule. |
| `audit_path` | 4 | `[* hash]` | MUST (MAY be empty) | The RFC 6962 sibling hashes; the verifier folds them (with the §18.9.5 node prefix `0x01`) up to a root and MUST equal the STH `root_hash`, else `ERR_KT_PROOF_INVALID` (`0x0108`). |

### 18.4.11 `ConsistencyProof` (KT, §3.5)

An RFC 6962 **consistency proof** that the tree of an earlier STH (`first_size`) is a **prefix
(append-only extension)** of a later STH (`second_size`) of the **same** `log_id` — the object the
gossip cross-check of §3.5.2(a) requests to detect a rewrite/rollback. Unsigned (verified against
the two STH roots).

```cddl
ConsistencyProof = {
  1 => u64,             ; first_size    tree_size of the earlier STH
  2 => u64,             ; second_size   tree_size of the later STH (>= first_size)
  3 => [* hash],        ; proof_path    RFC 6962 consistency proof nodes
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `first_size` | 1 | `u64` | MUST | Earlier tree size; `≤ second_size`. |
| `second_size` | 2 | `u64` | MUST | Later tree size. |
| `proof_path` | 3 | `[* hash]` | MUST (MAY be empty when `first_size ∈ {0, second_size}`) | The RFC 6962 consistency nodes; the verifier checks that the `first_size` root is a prefix of the `second_size` root (both from the two STHs). A proof that does not verify — or the demonstrable **absence** of any valid proof between two heads of one log — is an append-only violation, `ERR_KT_STH_INCONSISTENT` (`0x0110`) → equivocation response (§3.5.2(d)). |

### 18.4.12 `Profile` (§3.9.5)

An identity's **self-asserted, signed human display data** — display name, optional structured
name parts, and an optional avatar pointer (§3.9.5). It is a **replaceable pointer**, authenticated
to the key exactly like `Identity.names` (§3.9.4): the signature proves the key asserts this data,
never a real-world identity. Signed by `IK` (or an `IK`-authorized device key, §1.2); versioned
and rollback-protected like `Identity`; published and pinned via the directory / DNS / KT path
(§3.3–3.5). The avatar is **owner-hosted** — DMTAP stores no image — with an OPTIONAL content
address giving tamper-evidence for the exact bytes the owner signed.

```cddl
Profile = {
  1 => suite,           ; suite         suite of ik/sig
  2 => ik-pub,          ; ik            the identity this profile describes
  3 => u64,             ; version       monotonic; reject <= last pinned (rollback defense)
  4 => tstr,            ; display_name  primary human-shown string (UTF-8, NFC)
  ? 5 => tstr,          ; given_name    OPTIONAL structured name part
  ? 6 => tstr,          ; family_name   OPTIONAL structured name part
  ? 7 => Avatar,        ; avatar        OPTIONAL owner-set avatar pointer
  ? 8 => hash,          ; prev          OPTIONAL hash of previous Profile version (chain)
  9 => ts,              ; ts            publication time
  10 => sig-val,        ; sig           IK (or IK-authorized device key) over the body (§18.9.3)
}

Avatar = {
  1 => tstr,            ; url           owner-set public URL of the image (https RECOMMENDED)
  ? 2 => hash,          ; hash          OPTIONAL 0x1e ‖ BLAKE3-256 content address of the image bytes
}
```

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `Profile` | `suite` | 1 | `suite` | MUST | Suite of `ik`/`sig`. |
| | `ik` | 2 | `ik-pub` | MUST | The identity this profile describes; MUST match the pinned `Identity.iks[suite]`. |
| | `version` | 3 | `u64` | MUST | Monotonic. A verifier MUST reject a `version` ≤ the last pinned for this identity (rollback defense, `ERR_STALE_ROLLBACK` `0x0105`, same rule as `Identity.version`). |
| | `display_name` | 4 | `tstr` | MUST | The primary human-shown string, UTF-8 (NFC). **Self-asserted** — proves only that this key chose it, never a real-world identity (§3.9.5); two keys MAY assert the same string. Confusable/spoofing handling is a rendering concern, not an authenticity claim. |
| | `given_name` | 5 | `tstr` | OPTIONAL | Structured given-name part (legacy interop / sorting). |
| | `family_name` | 6 | `tstr` | OPTIONAL | Structured family-name part. |
| | `avatar` | 7 | `Avatar` | OPTIONAL | Owner-set avatar pointer. Absent ⇒ the client uses the §3.9.5 fallback ladder (key-derived identicon, then initials). |
| | `prev` | 8 | `hash` | OPTIONAL | Hash of the previous `Profile` version; absent for the first. Chains profile history for KT audit (§3.5), same shape as `Identity.prev`. |
| | `ts` | 9 | `ts` | MUST | Publication time. |
| | `sig` | 10 | `sig-val` | MUST | `IK` (or an `IK`-authorized device key, §1.2) signature over the body (§18.9.3, DS-tag `DMTAP-v0/profile`). A `Profile` whose `sig` fails MUST be rejected (`ERR_PROFILE_SIG_INVALID` `0x0119`, FAIL_CLOSED_BLOCK) and the prior pinned profile / fallback ladder used. |
| `Avatar` | `url` | 1 | `tstr` | MUST | Owner-set public URL of the avatar image. It is **attacker-chosen data** (any key may publish any `Profile` about itself): a fetcher MUST require scheme `https` and MUST NOT fetch a URL that resolves to a loopback / private / link-local / ULA / cloud-metadata (`169.254.169.254`) address — re-checked after any redirect — else `ERR_PROFILE_AVATAR_URL_UNSAFE` (`0x011B`, FAIL_CLOSED_BLOCK) and fall back (§3.9.5). DMTAP does **not** host the image. Fetching discloses the viewer's IP/timing to the owner-chosen host (a read-beacon): a client MUST NOT fetch eagerly on message arrival, and when `avatar.hash` is present MUST cache by content address and SHOULD NOT re-fetch on later renders (§3.9.5, §6 metadata caveat). |
| | `hash` | 2 | `hash` | OPTIONAL | Content address (`0x1e ‖ BLAKE3-256`, §18.1.5) of the exact image bytes. When present, a client MUST verify the fetched bytes address to this value **before display**; on mismatch it MUST NOT display them, MUST fall back (§3.9.5), and SHOULD warn (`ERR_PROFILE_AVATAR_HASH_MISMATCH` `0x011A`, USER_WARN). Absent ⇒ the URL is best-effort with no integrity guarantee. |

---

## 18.5 Transport-layer object (§4)

### 18.5.1 `LocationRecord` (§4.2)

The self-certifying `key → location` value record stored in the DHT (IPNS pattern).

```cddl
LocationRecord = {
  1 => ik-pub,          ; ik        identity key (DHT key = multihash(ik))
  2 => peer-id,         ; peer_id   node id per `substrate` (v0 libp2p PeerId; MAY be per-epoch/unlinkable, §6)
  3 => [* maddr],       ; addrs     current reachability hints
  4 => u64,             ; seq       monotonic sequence number (rollback defense, §16.2)
  5 => u64,             ; ttl       record lifetime in seconds
  6 => ts,              ; ts
  ? 8 => u8,            ; substrate transport-substrate tag (§21.24); absent ⇒ 0x01 libp2p (§4.1)
  7 => sig-val,         ; sig       signed by a device key
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `ik` | 1 | `ik-pub` | MUST | Identity key; the DHT key is `multihash(ik)`. |
| `peer_id` | 2 | `peer-id` | MUST | libp2p PeerId; MAY be a per-epoch, unlinkable id to decouple node from identity (§6.4). |
| `addrs` | 3 | `[* maddr]` | MUST (MAY be empty) | Reachability hints (multiaddrs): direct, relay-circuit, or mix addresses. Order = preference. |
| `seq` | 4 | `u64` | MUST | Monotonic sequence number; a resolver MUST reject a record whose `seq` is **older or equal** to one already seen (rollback/replay defense, §4.2, §16.2). |
| `ttl` | 5 | `u64` | MUST | Record lifetime in seconds (v0 default 2 h, §16.2); republished before expiry (default 45 min). |
| `ts` | 6 | `ts` | MUST | Publication time. |
| `substrate` | 8 | `u8` | OPTIONAL | Transport-substrate tag from the Transport Substrates registry (§21.24). **Absent ⇒ `0x01` (libp2p)**, the v0 default (§4.1). Governs how `peer_id` and `addrs` are interpreted and dialed; a resolver that does not implement the tagged substrate treats the record as unreachable (`0x0303`), never a parse error. Introducing a new substrate is the additive, capability-negotiated (§10.2) migration analogous to a new suite (§4.1, §21.25) — not a flag day. |
| `sig` | 7 | `sig-val` | MUST | Signed by a **device key** (not necessarily IK) authorized in the current `Identity` (§18.9.3). Signing authenticates content only — it does NOT stop eclipse (§4.2). Covers `substrate` when present. |

> **Reconciled (§18.11 item 3):** `seq` (key 4) is REQUIRED here, per §16.2's "Location
> seq-number | monotonic u64" rollback defense. §4.2's inline CBOR now carries the same `seq`
> field, so appendix and prose agree; a resolver MUST reject any record whose `seq` is older-or-
> equal to one already seen.

### 18.5.2 `MixNodeDescriptor` (§4.4.2)

A signed self-descriptor a mix node publishes so senders can route Sphinx packets (§4.4.1)
through it. It advertises the node's **Sphinx mix public key(s)** per epoch (§4.4.4), its
reachability, and its **stratified layer** (§4.4.3). It is discovered via the `MixDirectory`
(§18.5.3), which is bound to the existing DNS/KT trust (§4.4.2); a descriptor is otherwise an
ordinary signed identity-style object.

```cddl
MixNodeDescriptor = {
  1 => suite,           ; suite      signature/KEM/hash suite of this descriptor and its mix keys
  2 => ik-pub,          ; node_ik    the mix node's long-term identity key (its DMTAP identity)
  3 => [* maddr],       ; addrs      reachability hints for the mix role
  4 => [+ MixKeyEntry], ; mix_keys   current + next Sphinx mix public keys, keyed by epoch (§4.4.4)
  5 => mix-layer,       ; layer      stratified position: 0=entry, 1=middle, 2=exit (§4.4.3)
  ? 9 => ik-pub,        ; operator   operator identity for diversity (§4.4.8); absent ⇒ node_ik
  ? 8 => u8,            ; substrate  transport-substrate tag (§21.24); absent ⇒ 0x01 libp2p
  6 => ts,              ; ts
  7 => sig-val,         ; sig        signed by an IK-authorized device key of `node_ik` (§18.9.9)
}
MixKeyEntry = { 1 => u64, 2 => enc-key, 3 => ts }   ; epoch, Sphinx mix public key, valid-until
mix-layer   = 0..2
```

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `MixNodeDescriptor` | `suite` | 1 | `suite` | MUST | Suite of `sig` and of `mix_keys` (v0 `0x01`: the Sphinx mix key is an X25519 public key; §4.4.1). A PQ mix suite is a future registration (§4.4.12). |
| | `node_ik` | 2 | `ik-pub` | MUST | The mix node's long-term identity key — an ordinary DMTAP identity (§1.3), so a mix operator is accountable and KT-auditable (§4.4.2, §9.8). |
| | `addrs` | 3 | `[* maddr]` | MUST (MAY be empty) | Reachability hints for the mix role, interpreted per `substrate`. |
| | `mix_keys` | 4 | `[+ MixKeyEntry]` | MUST | The node's **Sphinx mix public keys** by epoch (§4.4.4): at least the current epoch, SHOULD also carry the next (overlap window) so senders can pre-build. A sender MUST encrypt each hop to the key of the **epoch it targets** and MUST reject a descriptor with no key for a usable epoch. |
| | `layer` | 5 | `mix-layer` | MUST | Stratified layer the node serves (`0` entry / `1` middle / `2` exit); the directory authority assigns/accepts it, and path selection (§4.4.3) draws one node per layer in order. |
| | `operator` | 9 | `ik-pub` | OPTIONAL | The **operator identity** this mix is under, for the **operator-diversity** path rule (§4.4.8): a path MUST NOT reuse an `operator` across hops. **Absent ⇒ the mix is its own operator (`node_ik`).** SHOULD be corroborated by a `_dmtap-mix` operator attestation (§4.4.8, analogous to `_dmtap-gw`, §7.2a) so it cannot be spoofed to defeat diversity. |
| | `substrate` | 8 | `u8` | OPTIONAL | Transport-substrate tag (§21.24); absent ⇒ `0x01` libp2p (§4.1). |
| | `ts` | 6 | `ts` | MUST | Descriptor publication time. |
| | `sig` | 7 | `sig-val` | MUST | Signature by an `IK`-authorized device key of `node_ik` (§18.9.9). Authenticates the descriptor's content; trust in the *set* of mixes comes from the KT-anchored `MixDirectory`, §18.5.3. |
| `MixKeyEntry` | `epoch` | 1 | `u64` | MUST | Monotonic mix-key epoch number (§4.4.4). |
| | `mix_key` | 2 | `enc-key` | MUST | Sphinx per-hop public key for this epoch (v0 X25519). |
| | `valid_until` | 3 | `ts` | MUST | End of this epoch's validity; a packet built to an expired epoch key is rejected (`ERR_MIX_DESCRIPTOR_STALE`, `0x030C`). |

### 18.5.3 `MixDirectory` (§4.4.2)

The signed, versioned, **KT-anchored** snapshot of the mix fleet for an epoch — the mixnet analog
of the `DomainDirectory` (§18.4.7) and subject to the same "indexes, does not forge" discipline.
It is published by a **directory authority** (a DMTAP identity whose key is pinned via DNS/KT,
§4.4.2) and its root is appended to key transparency (§3.5) so the fleet's history is append-only
and auditable; the authority key SHOULD be threshold-held (§5.8.6) and MAY be a set with a
`> n/2` quorum (§3.5.2(b)) so no single authority can unilaterally inject mixes.

```cddl
MixDirectory = {
  1 => suite,                  ; suite
  2 => ik-pub,                 ; authority   directory-authority identity key (pinned via DNS/KT)
  3 => u64,                    ; epoch       directory epoch (§4.4.4)
  4 => u64,                    ; version     monotonic; reject older-or-equal (rollback defense)
  5 => [+ MixNodeDescriptor],  ; mixes       the fleet for this epoch, each independently signed
  6 => hash,                   ; prev        content-address of the previous MixDirectory (chain)
  7 => ts,                     ; ts
  8 => sig-val,                ; sig         signed by `authority` (§18.9.9)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `suite` | 1 | `suite` | MUST | Suite of the authority signature. |
| `authority` | 2 | `ik-pub` | MUST | Directory-authority identity key; a verifier MUST have pinned it via DNS/KT (§4.4.2) and MUST reject a directory not signed by it (`ERR_MIX_DIRECTORY_SIG_INVALID`, `0x030B`). |
| `epoch` | 3 | `u64` | MUST | The mix-key epoch this directory describes (§4.4.4). |
| `version` | 4 | `u64` | MUST | Monotonic; a resolver MUST reject a directory whose `version` is older-or-equal to one already accepted (rollback defense, mirrors `LocationRecord.seq`). |
| `mixes` | 5 | `[+ MixNodeDescriptor]` | MUST | Each mix's own signed descriptor (§18.5.2); the authority attests **membership of the set**, not the descriptors' content (each self-verifies under its own `node_ik`). A directory MUST contain ≥ 1 node per stratified layer or path-building fails (`ERR_MIX_PATH_UNBUILDABLE`, `0x030D`). |
| `prev` | 6 | `hash` | MUST | Content-address of the previous `MixDirectory`, chaining the fleet history (genesis = all-zero digest with the v0 prefix); the root is KT-anchored (§3.5) so equivocation over the fleet is detectable exactly like a split-view (`0x0107`). |
| `ts` | 7 | `ts` | MUST | Publication time. |
| `sig` | 8 | `sig-val` | MUST | Directory-authority signature (§18.9.9). |

### 18.5.4 `SphinxCell` — packet, β routing-command, SURB & fragment framing (§4.4.1)

Sphinx is a **fixed-length binary** packet, **not** deterministic CBOR — so, unlike every object
above, `SphinxCell` and its sub-structures are pinned as **byte layouts** (they are on the mixnet
wire, not in a CBOR MOTE, and MUST be constant-length at every hop). This subsection gives the
DMTAP-specific field pinning §4.4.1 requires: the per-hop routing/delay command in `β`, the SURB
layout, and the cell fragment/reassembly framing across the `{16, 64}` KiB ladder (§16.3). All
multi-byte integers are **big-endian** (§18.1.3). Cryptographic construction (`α` re-randomization,
`β` stream-cipher onion, `γ` header MAC, `δ` LIONESS wide-block PRP) is per §4.4.1 and is not
re-specified here.

**`SphinxCell` (fixed length).**

| Part | Bytes | Meaning |
|------|------:|---------|
| `α` | 32 | header group element (X25519 point, v0), re-randomized per hop (§4.4.1). |
| `β` | `r_max · 48` = **240** | the routing onion: `r_max = 5` per-hop `RoutingCommand` blocks (48 B each, below), zero-padded for shorter paths so a 3-hop and 5-hop cell are byte-identical (§4.4.1). |
| `γ` | 16 | Poly1305 header MAC over `β` for this hop (§4.4.1). |
| `δ` | **2048** | the constant-length payload cell (LIONESS-permuted per hop); its plaintext at the exit is a `SphinxFragmentHeader` (below) followed by fragment bytes. |

The total on-wire cell is `32 + 240 + 16 + 2048 = 2336` bytes, **identical for every cell of every
profile** (padding hides both path length and true payload length).

**`RoutingCommand` (per-hop, fixed 48 bytes, inside `β`).** Each hop peels exactly one:

| Field | Offset | Bytes | Meaning |
|-------|-------:|------:|---------|
| `cmd` | 0 | 1 | `0x00` forward-to-mix; `0x01` deliver-to-recipient (exit); `0x02` SURB-reply hop. Unknown ⇒ drop (`ERR_MIX_PACKET_MALFORMED`, `0x0307`). |
| `flags` | 1 | 1 | bit0 = last-hop; other bits reserved (MUST be 0). |
| `delay_ms` | 2 | 4 | the hop's **Poisson-sampled** hold in milliseconds (the sender draws it `exp(mean)` per §16.3 and writes it here; the hop MUST honor it — memoryless mixing, §4.4.6). |
| `next_hop` | 6 | 32 | for `cmd=0x00`/`0x02`, the next node's routing id (`peer-id`/mix id per `substrate`, §21.24); for `cmd=0x01`, all-zero (the recipient is the local node). |
| `reserved` | 38 | 10 | MUST be zero; reserved for a future substrate/PQ field, capability-gated (§10.2). |

**`SURB` (Single-Use Reply Block, §4.4.1/§4.4.5).** Lets a recipient (or a loop, §4.4.7) reply
without learning the sender's location. Layout:

| Field | Bytes | Meaning |
|-------|------:|---------|
| `first_hop` | 32 | routing id of the SURB's first mix (where the replier injects). |
| `header` | 288 | a **pre-built** Sphinx header `(α ‖ β ‖ γ)` = `32 + 240 + 16` for the return path, opaque to the replier. |
| `key_seed` | 32 | the seed the replier uses to LIONESS-wrap its `δ` so only the SURB's creator can peel the reply (the creator holds the matching per-hop keys). |

A replier treats `SURB` as opaque: it sets `δ` to its (padded, single-cell) reply, wraps it under
`key_seed`, prepends `header`, and sends to `first_hop`. A SURB is **single-use** — reusing one is
a replay and is dropped by the per-epoch replay cache (§4.4.6).

**`SphinxFragmentHeader` (fixed 16 bytes, at the front of each cell's `δ` plaintext).** A MOTE is
padded to a bucket rung and split into `frag_count` cells (§4.4.1); each cell's payload begins with:

| Field | Offset | Bytes | Meaning |
|-------|-------:|------:|---------|
| `msg_id` | 0 | 8 | random per-MOTE id linking this MOTE's fragments at the recipient (unlinkable to identity; fresh per MOTE). |
| `frag_index` | 8 | 2 | 0-based fragment number. |
| `frag_count` | 10 | 2 | total fragments for this MOTE (`∈ {1,4,16,32}`, the ladder). |
| `total_len` | 12 | 4 | true `Envelope` length in bytes before bucket padding, so the recipient strips padding after reassembly. |

The remaining `2048 − 16 = 2032` bytes of each cell carry fragment data. The recipient buffers by
`msg_id`, reassembles in `frag_index` order once all `frag_count` cells arrive (each over an
independent path, §4.4.3), truncates to `total_len`, and then runs `deliver` (§19.3.1) on the
recovered `Envelope`. A reassembly that never completes within the sender's retry window is simply
dropped (the sender re-sends, §4.7); a fragment with an out-of-range `frag_index`/`frag_count`, or
mixing `msg_id`s, is discarded (`0x0307`).

### 18.5.5 `PushSubscription` (§4.9)

The signed registration a device publishes **to its own node** so the node can wake it (§4.9.1). It
is an ordinary signed object, held only within the device cluster (§5.6) — never in the DHT, a
directory, or a relay. The device signature (an `IK`-authorized device key, §1.2) authenticates it
to the identity, so no other party can register or redirect a device's wakes.

```cddl
PushSubscription = {
  1 => u8,          ; provider    push-provider tag (§4.9.3): 1 UnifiedPush, 2 Web Push, 3 APNs, 4 FCM
  2 => tstr,        ; endpoint    provider endpoint URL (Web Push / UnifiedPush) or opaque device token (APNs / FCM)
  3 => bytes,       ; push_key    device public push key (Web Push: P-256 uncompressed point, 65 B, RFC 8291)
  4 => bytes,       ; auth_secret RFC 8291 auth secret (16 B), shared only with the user's own node
  5 => ik-pub,      ; device_key  the IK-authorized device key that signs this subscription (§1.2)
  6 => ts,          ; ts
  7 => sig-val,     ; sig         signed by `device_key` (§18.9.15)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `provider` | 1 | `u8` | MUST | Push-provider tag (§4.9.3). A node MUST prefer an open provider (`1`/`2`) where the platform allows and fall back to `3`/`4` only where mandated (§6.6 item 9). An unrecognized tag ⇒ unsupported provider (capability-negotiated, §10.2), never a parse failure. |
| `endpoint` | 2 | `tstr` | MUST | The provider endpoint: an RFC 8030 push-resource URL for Web Push / UnifiedPush, or the opaque platform device token for APNs / FCM. |
| `push_key` | 3 | `bytes` | MUST | The device's public push key the wake token is sealed to under RFC 8291 Web Push encryption (Web Push: an uncompressed P-256 point). |
| `auth_secret` | 4 | `bytes` | MUST | The RFC 8291 auth secret (16 B) mixed into the wake's HKDF key derivation; held **only** by the device and the user's own node, so only the node can produce a wake the device will open (§18.9.15). |
| `device_key` | 5 | `ik-pub` | MUST | The device signing key (an `IK`-authorized device key, §1.2) that signs this subscription; a verifier MUST confirm it is authorized by a current `DeviceCert` under the owner's `Identity`. |
| `ts` | 6 | `ts` | MUST | Registration time. |
| `sig` | 7 | `sig-val` | MUST | Signature by `device_key` over the body (§18.9.15). A signature that does not verify is `ERR_PUSH_SUBSCRIPTION_SIG_INVALID` (`0x0312`, FAIL_CLOSED_BLOCK); the subscription MUST NOT be acted on. |

### 18.5.6 `WakePing` (§4.9)

The **content-free, sender-blind** wake signal a node emits to a sleeping device (§4.9.1). It
carries **only** the opaque, RFC 8291-sealed "sync now" token — no sender, subject, recipient, or
content, and no field beyond key `1`. It bears **no** DMTAP `sig-val`: its authentication is the RFC
8291 AEAD tag under the device push key + `auth_secret` (§18.9.15), so the push relay can neither
read nor forge one.

```cddl
WakePing = {
  1 => bytes,       ; token   RFC 8291 (aes128gcm) sealed wake token; the sealed plaintext is an
                    ;         opaque fixed-form sync nonce ONLY — no sender/subject/recipient/content (§4.9.1)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `token` | 1 | `bytes` | MUST | The RFC 8291 `aes128gcm` ciphertext of an opaque sync nonce. The sealed plaintext MUST be a **fresh, unpredictable nonce of ≥ 16 bytes** minted per wake (never fixed/reused), so the device can dedup replays (§4.9.1). A `WakePing` carrying **any** other map key, or whose opened plaintext decodes to anything carrying sender/subject/recipient/content, MUST be rejected fail-closed (`ERR_WAKEPING_CONTENT_PRESENT`, `0x0313`). A token whose AEAD fails to open under the subscription's `push_key`/`auth_secret` is a forged/unauthenticated wake and is dropped (`ERR_WAKEPING_AUTH_FAILED`, `0x0314`, DROP_SILENT). A token whose nonce is already in the device's replay cache (§16) is a **replayed** wake — dropped without re-waking (`ERR_WAKEPING_REPLAY`, `0x0316`, DROP_SILENT). Wakes are rate-limited per device at **emitter and receiver** (`ERR_WAKEPING_RATE_LIMITED`, `0x0315`, §4.9.4). |

---

## 18.6 Group-layer objects (§5.8)

### 18.6.1 `GroupState`

The signed, versioned snapshot of a group's roster and policy, derived from the group's
hash-chained MLS handshake log (§5.1). The **authoritative** ordering is the log; `GroupState` is
the committer-signed projection members pin.

```cddl
GroupState = {
  1  => bytes,          ; group_id
  2  => suite,
  3  => bytes,          ; epoch        MLS epoch / group-context ref
  ? 4  => tstr,         ; name         group's human name (§5.8), if any
  5  => ik-pub,         ; committer    the node serializing handshakes (§5.1)
  6  => posting-model,  ; posting_model
  7  => visibility,     ; membership_visibility
  8  => join-policy,    ; join_policy
  9  => [+ RosterEntry],; roster
  10 => hash,           ; log_head     hash-chained handshake-log head
  11 => u64,            ; version      monotonic
  12 => ts,             ; ts
  13 => sig-val,        ; committer_sig  by `committer` over the body (§18.9.6)
  14 => hash,           ; group_identity  content addr of the group's own Identity (§5.8.6)
}

RosterEntry = {
  1 => ik-pub,          ; member
  2 => [+ role],        ; roles
  3 => ts,              ; joined
}

posting-model = "broadcast" / "collaborative"
visibility    = "hidden" / "visible"
join-policy   = "closed" / "request" / "open" / "vouch"
role          = "owner" / "admin" / "member" / "poster" / "reader"
```

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `GroupState` | `group_id` | 1 | `bytes` | MUST | Stable group identifier; the group's own key-derived id (a group is an addressable identity, §5.8). |
| | `suite` | 2 | `suite` | MUST | Suite of `committer_sig` and member keys. |
| | `epoch` | 3 | `bytes` | MUST | Current MLS epoch reference; ties `GroupState` to a point in the ratchet history. |
| | `name` | 4 | `tstr` | OPTIONAL | Group human name (`@team`, `team@company.com`), if named (§5.8). |
| | `committer` | 5 | `ik-pub` | MUST | The member serializing handshake messages into the append-only log (§5.1). Rotatable by Commit; every member knows it. A committer can stall but not forge. |
| | `posting_model` | 6 | `posting-model` | MUST | `"broadcast"` (list: post→all; membership typically hidden) or `"collaborative"` (channel: shared ordered state; membership typically visible) (§5.8.1). Switchable by a Commit. |
| | `membership_visibility` | 7 | `visibility` | MUST | `"hidden"` (per-member sealed fan-out; members don't learn each other, §5.8.3) or `"visible"` (normal MLS tree). |
| | `join_policy` | 8 | `join-policy` | MUST | `"closed"` (invite only), `"request"` (admin approval), `"open"` (anyone with the address, rate-limited + §9), or `"vouch"` (member introduces) (§5.8.2). |
| | `roster` | 9 | `[+ RosterEntry]` | MUST | Current members with roles; ≥ 1 (≥ 1 `owner`). |
| | `log_head` | 10 | `hash` | MUST | Hash-chained head of the group's handshake log; two Commits at the same position with the same predecessor is proof of committer misbehavior → members MUST halt and alert (§5.1). |
| | `version` | 11 | `u64` | MUST | Monotonic; reject ≤ last pinned. |
| | `ts` | 12 | `ts` | MUST | Snapshot time. |
| | `committer_sig` | 13 | `sig-val` | MUST | Signature by `committer` over the body (§18.9.6). Members MUST also independently verify each underlying handshake is member-signed (a committer cannot forge membership changes). |
| | `group_identity` | 14 | `hash` | MUST | Content address of the group's **own** `Identity` object (§18.4.1) — the group is an addressable identity with its own keypair (§5.8). That `Identity` carries the group's **threshold-held signing key** (`iks`, FROST-style over the `owner`/`admin` set) and its `recovery` → group `RecoveryPolicy` (§18.4.4), per §5.8.6. Changing the group's identity key or recovery is a **threshold act** requiring the group's `rotate_threshold` (weakening-quorum + veto rules of §1.4 apply) and MUST appear in key transparency (§3.5); the `committer` orders *handshakes* only and is **NOT** authorized to change it. `group_id` (key 1) is derived from this identity. |
| `RosterEntry` | `member` | 1 | `ik-pub` | MUST | Member identity key. |
| | `roles` | 2 | `[+ role]` | MUST | ≥ 1 role from `{owner, admin, member, poster, reader}` (§5.8.2). `owner`/`admin` gate management ops; `poster` may send, `reader` may not. |
| | `joined` | 3 | `ts` | MUST | Join timestamp. |

### 18.6.2 `GroupEvent` (kind `0x06`)

A thin wrapper carrying an opaque MLS handshake PDU with the committer's ordering. The MLS
`MLSMessage` (Proposal/Commit/Welcome) is defined by RFC 9420 and is **opaque** to DMTAP framing.

```cddl
GroupEvent = {
  1 => bytes,           ; group_id
  2 => bytes,           ; epoch
  3 => bytes,           ; mls       opaque MLSMessage (RFC 9420 TLS-encoded)
  4 => u64,             ; log_seq   position in the hash-chained handshake log
  5 => hash,            ; prev      previous log entry's hash (chain)
  6 => sig-val,         ; committer_sig  committer's ordering signature (§18.9.6)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `group_id` | 1 | `bytes` | MUST | Target group. |
| `epoch` | 2 | `bytes` | MUST | Epoch the handshake applies to. |
| `mls` | 3 | `bytes` | MUST | The opaque, TLS-encoded MLS `MLSMessage` (RFC 9420). DMTAP does not re-encode MLS bytes; it carries them verbatim. Every handshake is **member-signed** inside this blob. |
| `log_seq` | 4 | `u64` | MUST | Strictly increasing position in the per-group log. A gap or duplicate is a fork signal (§5.1). |
| `prev` | 5 | `hash` | MUST | Hash of the previous log entry (`GroupEvent`), chaining the log. Genesis uses an all-zero digest with the v0 prefix. |
| `committer_sig` | 6 | `sig-val` | MUST | Committer signature over `(group_id, epoch, mls, log_seq, prev)` (§18.9.6), asserting ordering only. |

### 18.6.3 Device-cluster sync objects (§5.6)

The objects the owner's devices exchange to converge the personal cluster (§5.6). They ride
**inside the encrypted, authenticated MLS cluster group** (§5.6.1) — like a `DeniableMessage`
(§18.3.9) they carry **no separate DMTAP signature**: the MLS group provides confidentiality and
membership authentication, and each op/frame names its origin `device_id` whose non-revoked
`DeviceCert` (§1.2) a receiver MUST check before acting (`ERR_CLUSTER_DEVICE_UNAUTHORIZED`,
`0x0410`). A `RangeFingerprint` is additionally **self-verifying** by recomputation (§5.6.3(a)).

```cddl
; The top-level cluster-sync frame. `type` selects which optional body fields are present.
ClusterSyncFrame = {
  1 => u8,                     ; type      1=announce 2=recon 3=fetch-request 4=journal 5=stability
  2 => ik-pub,                 ; device    origin device key (a non-revoked DeviceCert subject, §5.6.1)
  ? 3 => [* hash],             ; ids       object content-addresses (type 1 announce / 3 fetch-request)
  ? 4 => [* RangeFingerprint], ; ranges    Merkle range summary (type 2 recon, §5.6.3(a))
  ? 5 => [* ClusterOp],        ; ops       CRDT metadata ops (§5.6.4)
  ? 6 => [* JournalEntry],     ; journal   append-only hash-chained segment (type 4, §5.6.3(b))
  ? 7 => [* StabilityMark],    ; stability per-device max-applied HLC (type 5, tombstone GC §5.6.5)
}

; One id-range and the sender's fingerprint over the ids it holds there. Self-verifying:
; the receiver recomputes `fp` over ITS ids in [lo, hi) and compares (§5.6.3(a)).
RangeFingerprint = {
  1 => hash,   ; lo    inclusive low id bound of the range
  2 => hash,   ; hi    exclusive high id bound
  3 => u64,    ; count number of ids the sender holds in [lo, hi)
  4 => hash,   ; fp    suite hash over the sender's SORTED ids in [lo, hi)
}

; Hybrid logical clock. Total order = lexicographic (wall, counter, device) (§5.6.4).
HLC = { 1 => u64, 2 => u32, 3 => ik-pub }   ; wall(ms), counter, device_id

; A unique OR-Set add-tag: which device added an element, and when (§5.6.4).
AddTag = { 1 => ik-pub, 2 => HLC }

; A CRDT metadata op. kind 1/2 = OR-Set add/remove (membership, folders, labels, deletes);
; kind 3 = per-field LWW-Register write (read/unread, star, current folder), keyed by `hlc`.
ClusterOp = {
  1 => u8,           ; kind     1=set-add 2=set-remove 3=lww-set
  2 => tstr,         ; target   object / folder / label id the op applies to
  ? 3 => tstr,       ; field    LWW field name (kind 3), e.g. "read" / "folder" / "star"
  ? 4 => ext-value,  ; value    LWW value (kind 3)
  5 => HLC,          ; hlc      add-tag time (kind 1) / LWW key (kind 3) / remove time (kind 2)
  ? 6 => [+ AddTag], ; observed add-tags this remove tombstones (kind 2)
}

; One append-only, hash-chained per-account journal entry (§5.6.3(b)).
JournalEntry = { 1 => u64, 2 => hash, 3 => hash }   ; seq, prev(hash of prior entry), ref(object id or op hash)

; A device's advertised stability point: the max HLC it has durably applied (tombstone GC, §5.6.5).
StabilityMark = { 1 => ik-pub, 2 => HLC }           ; device, max-applied HLC
```

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `ClusterSyncFrame` | `type` | 1 | `u8` | MUST | `1` announce new object ids, `2` recon (range summary), `3` fetch-request (pull ids), `4` journal segment, `5` stability marks. |
| | `device` | 2 | `ik-pub` | MUST | Origin device key; MUST be a non-revoked cluster member (§5.6.1), else `0x0410` (FAIL_CLOSED_BLOCK). |
| | `ids` | 3 | `[* hash]` | OPTIONAL | Content-addresses announced (type 1) or requested (type 3). |
| | `ranges` | 4 | `[* RangeFingerprint]` | OPTIONAL | The reconciliation summary (type 2). A malformed summary, or a `fp` that does not recompute over the claimed range, ⇒ `ERR_CLUSTER_RECON_SUMMARY_INVALID` (`0x0411`, FAIL_CLOSED_BLOCK). |
| | `ops` | 5 | `[* ClusterOp]` | OPTIONAL | CRDT metadata ops to merge (§5.6.4). |
| | `journal` | 6 | `[* JournalEntry]` | OPTIONAL | Append-only journal segment for replay-backfill (type 4). A broken `prev` chain ⇒ `ERR_CLUSTER_JOURNAL_CHAIN_BROKEN` (`0x0412`, HALT_ALERT). |
| | `stability` | 7 | `[* StabilityMark]` | OPTIONAL | Per-device max-applied HLC for the tombstone-GC stability cut (§5.6.5). |
| `RangeFingerprint` | `lo`/`hi` | 1/2 | `hash` | MUST | Inclusive-low / exclusive-high id bounds of the range. |
| | `count` | 3 | `u64` | MUST | Ids the sender holds in `[lo, hi)`. |
| | `fp` | 4 | `hash` | MUST | Suite hash over the sender's **sorted** ids in `[lo, hi)`; the receiver recomputes and compares (§5.6.3(a)). |
| `HLC` | `wall`/`counter`/`device` | 1/2/3 | `u64`/`u32`/`ik-pub` | MUST | Hybrid logical clock; total order lexicographic `(wall, counter, device)` (§5.6.4). A `wall` more than the skew bound (§16.10) ahead of the receiver ⇒ `ERR_CLUSTER_CRDT_OP_INVALID` (`0x0413`). |
| `ClusterOp` | `kind` | 1 | `u8` | MUST | `1` set-add, `2` set-remove, `3` lww-set. An unknown kind ⇒ `0x0413`. |
| | `target` | 2 | `tstr` | MUST | Object/folder/label id the op applies to. |
| | `field`/`value` | 3/4 | `tstr`/`ext-value` | MUST **iff** `kind = 3` | The LWW field and its value. |
| | `hlc` | 5 | `HLC` | MUST | The op's hybrid logical clock (add-tag time / LWW key / remove time). |
| | `observed` | 6 | `[+ AddTag]` | MUST **iff** `kind = 2` | The add-tags this remove tombstones; a remove citing an unknown add-tag ⇒ `0x0413`. An op embedding a `DeniablePayload`/its plaintext (forbidden, §5.2.1) ⇒ `0x0413`. |
| `JournalEntry` | `seq`/`prev`/`ref` | 1/2/3 | `u64`/`hash`/`hash` | MUST | Strictly-increasing seq; hash of the prior entry (genesis = all-zero v0-prefixed digest); the object id or op hash this entry records. |
| `StabilityMark` | `device`/`hlc` | 1/2 | `ik-pub`/`HLC` | MUST | A device and the max HLC it has durably applied (§5.6.5). |

---

## 18.7 Auth-layer objects (§13)

### 18.7.1 `Challenge` (§13.3)

Created by the relying party (RP); presented to a trusted client, which binds and displays the
verified origin.

```cddl
Challenge = {
  1 => tstr,            ; rp_origin   the RP's true web origin (scheme://host[:port])
  2 => bytes,           ; nonce       single-use server nonce
  3 => ts,              ; issued_at
  4 => ts,              ; exp
  5 => tstr,            ; aud         intended RP audience identifier
  ? 6 => [* tstr],      ; scope       OPTIONAL requested scopes / capabilities
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `rp_origin` | 1 | `tstr` | MUST | The RP's true origin. Phishing resistance depends **entirely** on binding to this and having it injected/enforced by a trusted client (WebAuthn), never a value trusted from the RP by the signer (§13.3.1). |
| `nonce` | 2 | `bytes` | MUST | Single-use nonce; valid ≤ 120 s (§16.1). Reuse MUST be rejected (replay cache ≥ 300 s). |
| `issued_at` | 3 | `ts` | MUST | Issue time. |
| `exp` | 4 | `ts` | MUST | Expiry; an assertion after `exp` MUST be rejected. |
| `aud` | 5 | `tstr` | MUST | Audience binding the assertion to the intended RP. |
| `scope` | 6 | `[* tstr]` | OPTIONAL | Requested login scopes / delegated capabilities (§13.5). |

### 18.7.2 `Assertion` (§13.3)

The user's signed response. The signature is over the **canonical hash of the origin-bound
fields** (§13.3 step 5), so a look-alike origin cannot produce a valid assertion for the real RP.

```cddl
Assertion = {
  1 => tstr,            ; rp_origin   echoed; MUST equal the Challenge value
  2 => bytes,           ; nonce       echoed
  3 => ts,              ; issued_at   echoed
  4 => ts,              ; exp         echoed
  5 => tstr,            ; aud         echoed
  6 => ik-pub,          ; from        the identity-revealing login signer (IK-authorized device key)
  7 => sig-val,         ; sig         over the origin-bound preimage incl. scope + cnf (§18.9.8)
  8 => hash,            ; cnf         H(session_pubkey): binds the fresh per-RP session key (§13.3)
  ? 9 => [* tstr],      ; scope       echoed Challenge scope ([] if absent); inside the signed preimage
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `rp_origin` | 1 | `tstr` | MUST | Echo of `Challenge.rp_origin`; the RP MUST verify it equals its own origin. |
| `nonce` | 2 | `bytes` | MUST | Echo of `Challenge.nonce`; MUST be unused and within its validity window. |
| `issued_at` | 3 | `ts` | MUST | Echo. |
| `exp` | 4 | `ts` | MUST | Echo; RP MUST reject if now > `exp`. |
| `aud` | 5 | `tstr` | MUST | Echo; MUST match the RP. |
| `from` | 6 | `ik-pub` | MUST | The user's **login signer**: an `IK`-authorized **device key** (or `IK` itself, §1.2) that the RP MUST verify resolves to the pinned `name → key` identity (§3.4, §13.3 step 6). This is the identity-revealing login signature; it is **NOT** the session key. The fresh per-RP session key is committed by `cnf` (key 8), not carried here; per-RP session keys (used for DPoP/GNAP thereafter) are what give cross-site unlinkability (§13.4, §13.7 limit 7), not this field. |
| `sig` | 7 | `sig-val` | MUST | Signature by `from` over the origin-bound preimage **including `cnf`** (§18.9.8). A captured assertion cannot be replayed with an attacker-chosen session key because `cnf` is inside the signed preimage (session-hijack defense, §13.3). |
| `cnf` | 8 | `hash` | MUST | Confirmation key = `H(session_pubkey)`, carried in the `cnf` claim (RFC 7800) as a DPoP-style (RFC 9449 §6.1) `jkt` SHA-256 key-hash confirmation (§13.3 step 4). The client generates the per-RP, per-device session keypair **before** signing and commits it here; the RP MUST bind the session **only** to `cnf` (proof-of-possession, §13.4). Present on every native assertion, and embedded verbatim in a bridged ID Token (§13.6). |
| `scope` | 9 | `[* tstr]` | OPTIONAL (echo) | Echo of `Challenge.scope` (§18.7.1 key 6); the **empty array `[]`** when the Challenge omits it. It is **inside the signed preimage** (§18.9.8), so the granted scope is cryptographically bound to the user's consent: the RP MUST reconstruct the preimage with exactly the scope it will grant and MUST NOT grant any scope broader than the signed value (a broader grant fails signature verification; an over-attenuated delegation surfaces as `0x0508`). Closes the OAuth-style scope-elevation where a scope the user never signed is granted on the login assertion. |

### 18.7.3 `CapabilityToken` and `CapabilityRevocation` (§13.5, §3.10.4)

The normative wire form of the delegated **capability token** §13.5/§13.5.1 previously described
only informally ("UCAN-style"). A `CapabilityToken` is a **profile of UCAN v1.0** with the fields
below: a signed, offline-verifiable, **attenuable** grant of a *specific, least-privilege* right,
from an issuer key to an audience key, chainable so each link may only **narrow** its parent. It is
the object minted/checked by `delegate-capability`/`revoke-capability` (§19.6.6) and by the org-
admin roles (§13.5.1). Delegation is verified **without contacting any server**; revocation is a
separately published, KT-logged object (below).

```cddl
CapabilityToken = {
  1 => suite,           ; suite       signature/hash suite
  2 => ik-pub,          ; iss         issuer (delegator) key — IK, device key, or a parent token's `aud`
  3 => ik-pub,          ; aud         audience (delegatee) key this grant is FOR
  4 => [+ Capability],  ; caps        the granted capabilities (resource + ability + caveats)
  5 => u64,             ; nbf         not-before (ms epoch)
  6 => u64,             ; exp         expiry (ms epoch); MUST be present — no non-expiring capability
  7 => bytes,           ; nonce       uniqueness / anti-replay salt
  ? 8 => hash,          ; prnt        content-address of the PARENT CapabilityToken (absent ⇒ this link is rooted at `iss`)
  9 => sig-val,         ; sig         `iss` signature over the token body (DS-tag DMTAP-v0/cap-token, §18.9.14)
}
Capability = {
  1 => tstr,            ; resource    the scoped resource (e.g. "mailbox:calendar", "domain:abc.com/members")
  2 => tstr,            ; ability     the verb (e.g. "read", "send", "provision", "directory/write")
  ? 3 => { * tstr => ext-value },  ; caveats  attenuating conditions (e.g. {"before": ts, "label": "work"})
}

; Published, KT-logged revocation of a previously issued token (§13.4 revocation, §13.5.1).
CapabilityRevocation = {
  1 => suite,           ; suite
  2 => ik-pub,          ; iss         the revoker — MUST be the token's `iss` or an ancestor issuer in its chain
  3 => hash,            ; token       content-address of the revoked CapabilityToken (or a chain root it descends from)
  4 => ts,              ; ts          revocation time
  5 => sig-val,         ; sig         `iss` signature (DS-tag DMTAP-v0/cap-revocation, §18.9.14)
}
```

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `CapabilityToken` | `iss` | 2 | `ik-pub` | MUST | The delegator. For the **root** link (`prnt` absent) `iss` MUST be an authority the verifier already trusts for `caps` — the user's `IK`/device key (§13.5) or the **domain authority** for org roles (§13.5.1). |
| | `aud` | 3 | `ik-pub` | MUST | The delegatee this grant empowers; to **invoke**, a party proves possession of `aud` (via the session/DPoP key, §13.4). |
| | `caps` | 4 | `[+ Capability]` | MUST | The granted rights. **Attenuation invariant:** each `Capability` MUST be **≤** a capability granted by the parent (`prnt`) — same-or-narrower `resource`, same `ability`, and caveats only **added/tightened**. A token granting anything its parent did not is invalid (`ERR_CAPABILITY_DELEGATION_INVALID`, `0x0508`). |
| | `nbf`/`exp` | 5/6 | `u64` | MUST | Validity window; `exp` is REQUIRED (no eternal capability). Outside the window ⇒ `0x0508`. |
| | `nonce` | 7 | `bytes` | MUST | Uniqueness salt so two otherwise-identical grants have distinct content addresses (needed for precise revocation). |
| | `prnt` | 8 | `hash` | OPTIONAL | Content-address of the parent token in the delegation chain. **Absent ⇒ rooted at `iss`.** A verifier MUST validate the **whole chain** to a trusted root, checking the attenuation invariant at every link and that each link's `iss` equals its parent's `aud`. |
| | `sig` | 9 | `sig-val` | MUST | `iss` signature over the body (§18.9.14). Invalid ⇒ `0x0508`. |
| `Capability` | `resource` | 1 | `tstr` | MUST | The scoped object/namespace the ability applies to. |
| | `ability` | 2 | `tstr` | MUST | The permitted verb; an invocation whose requested ability/resource is not covered is rejected (`0x0508`). |
| | `caveats` | 3 | map | OPTIONAL | Attenuating conditions; a child MAY add caveats, never remove a parent's. |
| `CapabilityRevocation` | `iss` | 2 | `ik-pub` | MUST | MUST be the token's own `iss` or an **ancestor** issuer in its chain — only an issuer (or someone it delegated *from*) may revoke; a stranger cannot. |
| | `token` | 3 | `hash` | MUST | Content-address of the revoked `CapabilityToken` (revoking a chain root revokes all descendants). A verified revocation makes any invocation of that token (or descendant) fail `ERR_CAPABILITY_REVOKED` (`0x050B`). |
| | `ts`/`sig` | 4/5 | | MUST | Revocation time and `iss` signature (§18.9.14). Revocations are **published to the transparency log / status endpoint** (§13.4, §13.5.1) and MUST be routed through the owner's/domain's KT self-monitoring path so a silent grant/revoke is owner-visible (§13.5). |

**Verification (normative).** To honor an invocation a verifier MUST: (1) validate the token's
signature and, if `prnt` present, the **entire chain** to a trusted root; (2) check the
**attenuation invariant** at every link (each narrows its parent); (3) check `nbf ≤ now ≤ exp` at
every link; (4) confirm the requested `(resource, ability)` is covered by the leaf `caps` and all
caveats are satisfied; (5) confirm the invoker proves possession of the leaf `aud` key; (6) check
**no** `CapabilityRevocation` (from the token's `iss` or an ancestor) covers the token or any chain
link. Any failure of (1)–(4) is `0x0508`; a revocation hit at (6) is `0x050B`. Verification is
otherwise **offline** — no issuer round-trip — matching §13.5.

## 18.8 Client-facing transport-path provenance (§7.8, §8.6, §19.9)

### 18.8.1 `ProvenanceRecord`

The **client-facing** transport-path record. Unlike every other object in this appendix it is
**not transmitted over the mesh**: the recipient's own node **assembles** it at reception and
serves it to the owner's own devices over the authenticated JMAP / mesh client surface (§8.1,
§8.6, §19.9), so a client can render the transport-path graph (§8.6). It composes two sources of
truth with **different trust origins**, and the split is load-bearing for privacy (§6.8):

- **Observed transport** (`tier`, `profile`, `min_hops`) — what the **recipient node itself
  observed** about how the message arrived. It is **never** taken from a sender claim (a sender
  cannot be trusted to state its own tier) and **never** enumerates or identifies a mix node.
- **Verified gateway origin** (`origin`, `gateways`) — derived **solely** from the verified,
  sealed `Payload.provenance` attestation chain (§18.3.11). Empty chain ⇒ `origin = 0` (pure-mesh);
  ≥ 1 valid attestation ⇒ `origin = 1` (gateway-touched).

```cddl
ProvenanceRecord = {
  1 => u8,                       ; tier      observed arrival tier: 1 = private (mixnet), 2 = fast (direct)
  2 => u8,                       ; profile   mix profile evidenced: 0 = n/a (fast), 1 = standard, 2 = high-security
  3 => u8,                       ; origin    0 = pure-mesh, 1 = gateway-touched (legacy-origin)
  4 => [* GatewayAttestation],   ; gateways  verified attestation chain (§18.3.11), temporal order; empty iff origin = 0
  ? 5 => u8,                     ; min_hops  COARSE guaranteed lower-bound hop count; NEVER node identities (§6.8)
  ? 6 => ts,                     ; observed_at  recipient-node reception time
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `tier` | 1 | `u8` | MUST | The tier the message arrived on **as the recipient node observed it** (§4.6): `1` = `private` (peeled off the mixnet, §4.4), `2` = `fast` (direct/low-hop, §4.5). A recipient node knows this from *how it received the packet*; it is not a sender assertion. |
| `profile` | 2 | `u8` | MUST | The mix profile the arrival is consistent with (§4.4.10): `0` = not applicable (`tier = 2`), `1` = Standard (≥ 3 hops), `2` = High-security (≥ 5 hops). For `private` this states the **minimum-viable-path guarantee that held** (§4.4.9), not a measured path. |
| `origin` | 3 | `u8` | MUST | `0` = **pure-mesh** — no gateway attestation present, so the message was **never plaintext at a gateway** (the soundness of this claim rests on §7.2a making a gateway attestation mandatory for legacy-origin mail; §7.8). `1` = **gateway-touched / legacy-origin** — ≥ 1 verified attestation. |
| `gateways` | 4 | `[* GatewayAttestation]` | MUST (MAY be empty) | The **verified** attestation chain copied from `Payload.provenance` (§18.3.11) **after** each entry's signature has been checked (§18.9.11); entries that failed verification are **excluded** (a message with an unverifiable required attestation is rejected upstream, §19.3.1, and never reaches this record). Empty **iff** `origin = 0`. Temporal order (ascending `seq`). |
| `min_hops` | 5 | `u8` | OPTIONAL | A **coarse, privacy-safe lower bound** on hop count. For `private` it MUST equal the profile floor (`3` Standard / `5` High-security, §4.4.10) — the **guaranteed** minimum-viable-path length, **not** a measured or exact path, and it MUST NOT enumerate, identify, or count-beyond-the-floor the mixes traversed: the recipient node **cannot** know the full private path (that is precisely the mixnet's anonymity guarantee, §6.8) and MUST NOT synthesize one. For `fast` it MAY reflect the directly-observed hop (`1`), which exposes nothing beyond what `fast` already reveals (§6.5). |
| `observed_at` | 6 | `ts` | OPTIONAL | The recipient node's reception time. Local; never leaves the owner's device cluster. |

**Privacy invariants (normative, §6.8).** A `ProvenanceRecord`:

1. MUST be delivered **only** over the owner's authenticated client surface (§8.1) to the owner's
   own device cluster (§8.3); it MUST NOT be attached to, embedded in, or forwarded on any MOTE
   sent to a third party.
2. MUST NOT contain any mix-node identity, address, per-hop timing, path descriptor, or any datum
   from which the private-tier path could be reconstructed. `min_hops` is a **profile floor**, not
   a path. This keeps provenance a statement about **which trust boundaries a message crossed**,
   never **which nodes** carried it (§6.8) — it therefore reveals nothing an honest recipient
   could not already infer, and cannot weaken sealed sender or mixnet anonymity (§12.3, §6.2).

---

## 18.8a Coordinator-layer objects (coordinator/CONTRACT.md §2.1, §2.4, §6; §12.2)

CONTRACT §2.1 requires every coordinator to "publish a **signed descriptor** carrying its kind,
its policy, and — where it charges — a signed tariff," and CONTRACT §6 requires that a metered
coordinator "issue... signed usage receipts delivered directly to the paying party." Both MUSTs,
and the operator seam's `GatewayAuthz` (§12.2), were cited normatively for several revisions with
no wire form: an implementation had a security requirement it could not byte-exactly interop on.
This subsection is that wire form — the CONTRACT-level counterpart to §18.3.11/§18.3.12 for the
gateway kind specifically. `ERR_GATEWAY_SENDER_UNAUTHENTICATED` (`0x0607`),
`ERR_GATEWAY_SENDER_ADDRESS_UNAUTHORIZED` (`0x060A`), `ERR_GATEWAYAUTHZ_DENIED` (`0x070E`),
`ERR_ADAPTER_TARIFF_INVALID` (`0x0B01`), `ERR_ADAPTER_RECEIPT_INVALID` (`0x0B02`), and
`ERR_ADAPTER_CREDENTIAL_UNAUTHORIZED` (`0x0B03`, §21.8, §21.10, §21.11a) now each resolve to a
defined object below.

**One object family for every coordinator kind, not one per kind.** CONTRACT §5 states that
`gateway` (§7) and the legacy `adapter`s (§26) are the first, fully-worked instances of the
contract, and that every other kind (`relay`, `media-relay`, `reachability-adapter`, `indexer`,
`labeler`, `matcher`, `compute`, `arbiter`, `oracle`, `custodial-escrow`) inherits the same four
clauses unchanged. `CoordinatorDescriptor`/`Tariff`/`UsageReceipt` are correspondingly **one**
object family, keyed by the `kind` field (§18.8a.1, key 1) rather than one bespoke shape per kind
— a `gateway`'s domain/modes/attestation-selector (§7.5) and an adapter's rail/mode/initiation-
class (§26.3.1) are kind-specific facts and live in the opaque `policy` field (key 4) exactly as
they do informally today; nothing about §7.5's or §26.3.1's own field list changes. §26's
previously-**reserved-but-undefined** `DMTAP-ADAPT-v0/…` DS-tags (§21.24g) are **retired, unused**
in favor of the tags below (§21.24h) — an adapter, being a `gateway`-kind coordinator (CONTRACT
§5), signs the same object a mail gateway would, never a second parallel scheme.

### 18.8a.1 `CoordinatorDescriptor`, `Visibility`, `Tariff` (CONTRACT §2.1, §2.4, §6)

The **discovery-only, self-asserted** descriptor CONTRACT §2.1 requires. By construction it has no
field for a global score, a price rank, or a stake amount (§2.1) — a decoder MUST reject an
unknown key (§18.1.2) exactly so a future field cannot smuggle one back in without a version bump
a verifier would notice.

```cddl
CoordinatorDescriptor = {
  1 => tstr,             ; kind        coordinator kind string (CONTRACT §5 canonical table)
  2 => ik-pub,           ; identity    the coordinator's attested substrate identity (§1, CONTRACT §2.1)
  3 => Visibility,        ; visibility  exactly one declared class + assurance level (CONTRACT §2.4, §3)
  4 => bytes,            ; policy      opaque det_cbor operator policy — self-asserted, kind-specific
  ? 5 => Tariff,          ; tariff      OPTIONAL; present iff this coordinator charges (CONTRACT §6)
  6 => sig-val,           ; sig         signature over fields 1-5 (DS-tag DMTAP-COORD-v0/descriptor, §18.9)
}
Visibility = {
  1 => tstr,             ; class       "blind" / "blind-routing" / "terminating" (CONTRACT §3.1)
  2 => tstr,             ; level       "structural" / "attested" / "declared" (CONTRACT §3.3)
}
Tariff = {
  1 => ik-pub,           ; identity    the signing coordinator's OWN identity (self-certifying, below)
  2 => bytes,            ; schedule    opaque det_cbor price schedule; the numbers are operator policy (CONTRACT §6)
  3 => sig-val,           ; sig         signature over fields 1-2 (DS-tag DMTAP-COORD-v0/tariff, §18.9)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `kind` | 1 | `tstr` | MUST | One of the CONTRACT §5 canonical kind strings (`"gateway"`, `"relay"`, `"media-relay"`, `"reachability-adapter"`, `"indexer"`, `"labeler"`, `"matcher"`, `"compute"`, `"arbiter"`, `"oracle"`, `"custodial-escrow"`). An unknown `kind` MUST be treated as an undeclared coordinator (§2.4) — a client MUST NOT rely on a descriptor whose kind it does not recognize. |
| `identity` | 2 | `ik-pub` | MUST | The coordinator's attested substrate identity (CONTRACT §2.1). The descriptor is self-certifying — `sig` (key 6) is verified against this same field, never an external identity. |
| `visibility` | 3 | `Visibility` | MUST | Exactly one declared class at one assurance level (CONTRACT §2.4, §3.1, §3.3); `class` ∈ `{"blind","blind-routing","terminating"}`, `level` ∈ `{"structural","attested","declared"}`. An unrecognized value in either sub-field MUST be rejected, not defaulted (fail-closed, mirrors §18.1.2's unknown-key rule for the enclosing choice). |
| `policy` | 4 | `bytes` | MUST | Opaque deterministic-CBOR operator policy (region, capabilities, contact, and every kind-specific field §7.5/§26.3.1 already enumerate for `gateway`). This document does not interpret it; it exists so §2.1's "no reputation/price/stake field" rule has exactly one escape hatch (self-declared, never a ranking input) rather than a slow accretion of new top-level descriptor keys. |
| `tariff` | 5 | `Tariff` | OPTIONAL | Present **iff** this coordinator charges (CONTRACT §6); absent ⇒ free. |
| `sig` | 6 | `sig-val` | MUST | Signature by `identity` over `det_cbor(CoordinatorDescriptor ∖ {6})` under DS-tag `DMTAP-COORD-v0/descriptor` (§18.9). |
| `Tariff.identity` | 1 | `ik-pub` | MUST | The **signing** coordinator's own identity. `Tariff` is self-certifying (carries its own signer) rather than relying on the enclosing descriptor's, so a client that already holds a `Tariff` (e.g. handed one directly by an operator, or cached from an earlier descriptor) can verify it standalone; it MAY, but need not, equal the enclosing `CoordinatorDescriptor.identity`. |
| `Tariff.schedule` | 2 | `bytes` | MUST | Opaque deterministic-CBOR price shape/schedule. The **numbers** are operator policy and out of scope for this specification (CONTRACT §6); only the mechanism — a signed, verifiable, comparable object — is normative. |
| `Tariff.sig` | 3 | `sig-val` | MUST | Signature by `Tariff.identity` over `det_cbor(Tariff ∖ {3})` under DS-tag `DMTAP-COORD-v0/tariff` (§18.9). Fails ⇒ `ERR_ADAPTER_TARIFF_INVALID` (`0x0B01`, §21.11a) where the tariff is presented in an adapter context, or the kind-appropriate equivalent elsewhere. |

**Publication transport is intentionally unspecified**, exactly as §7.5/§26.3.1 already leave it
for the gateway/adapter descriptor today: an operator's own domain, a directory, or a
`pub_announce` (§22.3.1) under the operator's own author feed are all conformant; this object
specifies the **bytes**, not the channel.

### 18.8a.2 `UsageReceipt` (CONTRACT §6)

The signed usage receipt CONTRACT §6 requires a metering coordinator to deliver **directly to the
paying party**. Like `Tariff`, it is **independently self-certifying**: it carries its own signer
`identity` rather than depending on an enclosing descriptor, because — CONTRACT §6's own framing —
"a signed receipt lets a user confirm a claimed operation was real," and that check must hold up
standalone, at whatever later point the payer re-examines it, without a live descriptor fetch.

```cddl
UsageReceipt = {
  1 => ik-pub,           ; identity    the issuing coordinator's own identity (self-certifying)
  2 => bytes,            ; operation   opaque det_cbor metered operation this receipt attests to
  3 => sig-val,           ; sig         signature over fields 1-2 (DS-tag DMTAP-COORD-v0/usage-receipt, §18.9)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `identity` | 1 | `ik-pub` | MUST | The issuing coordinator's own identity; verified against `sig`, never an enclosing descriptor's. |
| `operation` | 2 | `bytes` | MUST | Opaque deterministic-CBOR description of the metered operation the receipt attests to (kind-specific — e.g. a legacy-adapter send, §26.10). |
| `sig` | 3 | `sig-val` | MUST | Signature by `identity` over `det_cbor(UsageReceipt ∖ {3})` under DS-tag `DMTAP-COORD-v0/usage-receipt` (§18.9). Fails to verify ⇒ `ERR_ADAPTER_RECEIPT_INVALID` (`0x0B02`, §21.11a) in an adapter context, or the kind-appropriate equivalent elsewhere. |

**Transport is the existing `system` MOTE** (`kind = 0x0A`, §21.16), delivered directly to the
paying identity, never published — the same carriage §26.10/§26.11 already specified informally.
No new message kind is allocated.

**Honest residual, restated from CONTRACT §6 (normative disclosure).** A verified `UsageReceipt`
proves the coordinator signed a claim about one real operation; it is **one-directional** — it
cannot disconfirm an operation the coordinator fabricated or silently omitted, and a client MUST
NOT present the absence of a disputed receipt as proof the operation never happened. Disclosed, not
hidden (CONTRACT §6, DMTAP §7.9 generalized).

### 18.8a.3 `GatewayAuthz` (§12.2, §7.11.2, §7.12, §26.2.1)

The operator-side record of a legacy-egress authorization (§12.2): who a gateway (or, generalized,
a gateway-mode adapter, §26.2.1 item 1) will relay outbound for, and — where granted — which
address(es) or rail identifier(s) that identity may claim. §7.11.2 step 2 and §26.2.1 item 1 each
cited a "planned... not yet defined on wire" per-address/per-rail grant type; this closes both from
the same object rather than minting two.

`GatewayAuthz` is **gateway-local state**, in the same sense `GatewayAliasMap` is (§18.3.12): it is
**not mesh-transmitted** and carries **no signature of its own**. Its authenticity is not a
property of its own bytes but of how it was **populated** — one of:

1. **key-registered admission** (§7.12.2): the record exists because the gateway verified a §13.3
   `Assertion` (§18.7.2, DS-tag `DMTAP-v0/auth-assertion`, §18.9.8) from `identity`, and the
   `Assertion`'s own signature is the authenticity evidence, retained or re-derivable by the
   operator, not re-signed into this record;
2. **an explicit per-address/per-rail grant**: a `CapabilityToken` (§18.7.3, DS-tag
   `DMTAP-v0/cap-token`) issued by the domain authority (mail) or the adapter operator (a rail) to
   `identity`, with `Capability.resource` = `"gw-addr:" ++ address` (the RFC 5322 address `identity`
   may claim, §7.11.2 step 2) or `"gw-rail:" ++ rail ++ ":" ++ remote_id` (the rail + remote-facing
   number/handle/account `identity` may claim, §26.2.1 item 1) and `Capability.ability` =
   `"send-as"`. This reuses the existing, fully-specified delegation/attenuation/revocation
   machinery (§18.7.3) instead of inventing a second one: the grant is offline-verifiable, narrows
   under the usual attenuation invariant, and revokes through the existing `CapabilityRevocation`
   path — no new DS-tag, no new signature scheme, no new revocation channel.

```cddl
GatewayAuthz = {
  1 => ik-pub,            ; identity     the (would-be) authorized sender's IK
  2 => u8,                ; mode         1 = open (operator-chosen admission, e.g. postage-only), 2 = key-registered (§7.12.1)
  3 => ts,                ; granted_at
  ? 4 => [* hash],        ; grants       content-addresses (§18.9.4) of CapabilityToken per-address/per-rail grants naming this identity
  ? 5 => ts,               ; expires
  ? 6 => bool,             ; revoked
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `identity` | 1 | `ik-pub` | MUST | The authorized sender's `IK`. Absence of a live, unexpired, unrevoked record for a given `identity` is exactly the condition behind `ERR_GATEWAY_SENDER_UNAUTHENTICATED` (`0x0607`) / `ERR_ADAPTER_CREDENTIAL_UNAUTHORIZED` (`0x0B03`). |
| `mode` | 2 | `u8` | MUST | `1` open (§7.12.1: the operator's own admission means, e.g. postage-only), `2` key-registered (§7.12.2, verified via a retained `Assertion`). |
| `granted_at` | 3 | `ts` | MUST | When this record was created. |
| `grants` | 4 | `[* hash]` | OPTIONAL | Content-addresses of the `CapabilityToken`(s) (§18.7.3, §18.9.4 content-address formula) that authorize `identity` for a **specific** address or rail identifier. Absence ⇒ `identity` is authenticated for egress (§7.11.2 step 1 / `0x0607`) but has **no** per-address/per-rail grant — the ordinary case, where the submitter's own `IK` already resolves to the address it claims (§7.11.2 step 2, first bullet), so no separate grant is needed. Where an outbound message claims an address/rail-identity the ordinary resolution does not cover, the gateway MUST find a covering, valid, unrevoked `CapabilityToken` referenced here or refuse with `ERR_GATEWAY_SENDER_ADDRESS_UNAUTHORIZED` (`0x060A`) / `ERR_ADAPTER_CREDENTIAL_UNAUTHORIZED` (`0x0B03`). |
| `expires` | 5 | `ts` | OPTIONAL | Absent ⇒ no expiry. A lookup after `expires` MUST be treated as no authorization. |
| `revoked` | 6 | `bool` | OPTIONAL | `true` retires the record (operator-initiated); retained rather than deleted so a repeat attempt is distinguishable from one that was never authorized, for audit purposes only — it confers nothing. |

**Operator-unreachable fail-safe (`ERR_GATEWAYAUTHZ_DENIED`, `0x070E`, §12.2).** This object is
consulted, not transmitted, when the operator behind the seam is unreachable: §12.2's safe default
(permit only already-established contacts + operator-independent PoW, deny cold/unproven egress)
is a **local policy fallback**, not a wire condition — there is nothing about `GatewayAuthz`'s own
bytes that changes during an outage, only which records the gateway can still consult.

---

## 18.9 Canonical signing & hashing preimages (normative)

Two independent implementations MUST produce **identical** signatures and content addresses. This
requires an exact, shared definition of the bytes fed to `Sign` / `Hash`. Except where noted,
every signature is over `DS-tag ‖ det_cbor(object∖sig)`, where:

- `DS-tag` is the object's ASCII domain-separation string from the table below, **terminated by
  one `0x00` byte**;
- `det_cbor(object∖sig)` is the RFC 8949 §4.2 deterministic encoding of the object **with its
  signature field(s) removed from the map entirely** (not set to `null`) and all other fields
  present exactly as they appear on the wire.

**Every preimage below is written in its single-component form; under a composite suite the
`suite` byte is inserted (normative, governs all of §18.9).** §18.1.6 defines two message
representatives, and which one applies is a property of the object's `suite`, not of the
subsection:

```
single-component (0x01) : M  = DS-tag ‖ 0x00 ‖ body
composite (0x02/0x03/0x04/0x05) : M' = DS-tag ‖ 0x00 ‖ u8(suite) ‖ body
```

Every preimage in this section — whether spelled out field by field, as in §18.9.1, or given as
`DS-tag ‖ det_cbor(object ∖ sig)` in the table above — is the **`body`** of those two forms. A
composite signer MUST sign `M'`, both components over the **same** `M'`, and a verifier MUST
reconstruct the representative matching the object's `suite` and MUST NOT accept a component that
verifies only against the other form (§18.1.6). This is stated once here because it was previously
implicit: §18.9.1 spelled its preimage out under the heading "Exactly:" with no `suite` byte, and
`Envelope.suite` is not one of the concatenated fields either, so the suite was not smuggled in
through a whole-object body. Meanwhile §18.1.6 required a `suite` byte for `0x02` — the **v0
REQUIRED originating suite** (§1.1) — so two conformant implementations would have failed each
other's `sender_sig` on the only suite either is allowed to originate. Nothing arbitrated, because
every frozen vector is `0x01` (§18.1.4), where the two forms coincide. Where a subsection restates
both forms explicitly (§18.9.1, §18.9.2) it is illustrating this rule, not creating an exception
to it.

| Object | Signed field(s) | DS-tag (ASCII, + `0x00`) | Preimage body |
|--------|-----------------|--------------------------|---------------|
| `Envelope` | `sender_sig` (k11) | `DMTAP-v0/envelope-sender` | §18.9.1 (concatenation, not whole-object CBOR) |
| `Payload` | `sig` (k2) | `DMTAP-v0/payload` | §18.9.2 (over payload hash) |
| `Identity` | `sig` (k10, per suite) | `DMTAP-v0/identity` | `det_cbor(Identity ∖ {10})` |
| `DeviceCert` | `sig` (k8) | `DMTAP-v0/device-cert` | `det_cbor(DeviceCert ∖ {8})` |
| `RecoveryPolicy` | `sig` (k9) | `DMTAP-v0/recovery-policy` | `det_cbor(RecoveryPolicy ∖ {9})` |
| `KeyRotation` | `sig` (k7) | `DMTAP-v0/key-rotation` | `det_cbor(KeyRotation ∖ {7})` |
| `MoveRecord` | `sig` (k7) | `DMTAP-v0/move-record` | `det_cbor(MoveRecord ∖ {7})` |
| `DomainDirectory` | `sig` (k9) | `DMTAP-v0/domain-directory` | `det_cbor(DomainDirectory ∖ {9})` |
| `Profile` | `sig` (k10) | `DMTAP-v0/profile` | `det_cbor(Profile ∖ {10})` (§18.4.12) |
| `LocationRecord` | `sig` (k7) | `DMTAP-v0/location-record` | `det_cbor(LocationRecord ∖ {7})` |
| `MixNodeDescriptor` | `sig` (k7) | `DMTAP-v0/mix-descriptor` | `det_cbor(MixNodeDescriptor ∖ {7})` |
| `MixDirectory` | `sig` (k8) | `DMTAP-v0/mix-directory` | `det_cbor(MixDirectory ∖ {8})` |
| `PushSubscription` | `sig` (k7) | `DMTAP-v0/push-subscription` | `det_cbor(PushSubscription ∖ {7})` (§18.9.15) |
| `WakePing` | — (none) | — | **no DMTAP sig** — authenticated by the RFC 8291 AEAD tag under the device push key + `auth_secret` (§18.9.15) |
| `GroupState` | `committer_sig` (k13) | `DMTAP-v0/group-state` | `det_cbor(GroupState ∖ {13})` |
| `GroupEvent` | `committer_sig` (k6) | `DMTAP-v0/group-event` | `det_cbor(GroupEvent ∖ {6})` |
| `PostageStamp` | `sig` (k7) | `DMTAP-v0/postage-stamp` | `det_cbor(PostageStamp ∖ {7})` |
| `Vouch` | `sig` (k5) | `DMTAP-v0/vouch` | `det_cbor(Vouch ∖ {5})` |
| `GatewayAttestation` | `sig` (k7) | `DMTAP-v0/gateway-attest` | §18.9.11 (`det_cbor(GatewayAttestation ∖ {7})`) |
| `Assertion` | `sig` (k7) | `DMTAP-v0/auth-assertion` | §18.9.8 |
| `SignedTreeHead` | `sig` (k6) | `DMTAP-v0/kt-sth` | `det_cbor(SignedTreeHead ∖ {6})`, signed by `log_id` (§18.9.13) |
| `InclusionProof`/`ConsistencyProof` | — (none) | — | **no signature** — verified against the STH `root_hash` (§18.9.13) |
| `CapabilityToken` | `sig` (k9) | `DMTAP-v0/cap-token` | `det_cbor(CapabilityToken ∖ {9})` (§18.9.14) |
| `CapabilityRevocation` | `sig` (k5) | `DMTAP-v0/cap-revocation` | `det_cbor(CapabilityRevocation ∖ {5})` (§18.9.14) |
| `SphinxCell` & sub-structures | — (none) | — | **no DMTAP sig** — Sphinx per-hop MAC / wide-block PRP (§18.5.4, §18.9.9) |
| `DeniablePrekeyBundle` | `sig` (k10) | `DMTAP-v0/deniable-prekeys` | `det_cbor(DeniablePrekeyBundle ∖ {10})` (§18.9.10) |
| `DeniablePrekeyBundle` | `spk_sig` (k4) | `DMTAP-v0/deniable-spk` | the raw `spk` bytes (field 3) (§18.9.10) |
| `DeniablePrekeyBundle` / `DeniableInit` | `idk_sig` (k12) / `idk_a_cert` (k10) | `DMTAP-v0/deniable-idk` | the raw `idk` / `idk_a` bytes (§18.9.10) |
| `DeniableInit`/`DeniableMessage`/`DeniablePayload` | — (none) | — | **no signature** — authenticated by the Double-Ratchet AEAD MAC (§18.9.10) |
| `ClusterSyncFrame` / `ClusterOp` / `RangeFingerprint` / `JournalEntry` / `StabilityMark` | — (none) | — | **no DMTAP sig** — carried inside the encrypted, membership-authenticated **MLS cluster group** (§5.6.1, §18.6.3); the origin `device_id`'s non-revoked `DeviceCert` authenticates the sender (`0x0410`) and a `RangeFingerprint` is self-verifying by recomputation (§5.6.3(a)). Referenced objects (MOTEs/manifests) remain self-signed/content-addressed. |
| `GatewayAliasMap` | — (none) | — | **no DMTAP sig** — gateway-local state, not mesh-transmitted (§18.3.12, §7.10) |
| `CoordinatorDescriptor` | `sig` (k6) | `DMTAP-COORD-v0/descriptor` | `det_cbor(CoordinatorDescriptor ∖ {6})` (§18.8a.1) |
| `Tariff` | `sig` (k3) | `DMTAP-COORD-v0/tariff` | `det_cbor(Tariff ∖ {3})` (§18.8a.1); self-certifying — signed by `Tariff.identity`, not the enclosing descriptor's |
| `UsageReceipt` | `sig` (k3) | `DMTAP-COORD-v0/usage-receipt` | `det_cbor(UsageReceipt ∖ {3})` (§18.8a.2); self-certifying — signed by `UsageReceipt.identity` |
| `GatewayAuthz` | — (none) | — | **no DMTAP sig** — gateway-local state, not mesh-transmitted; authenticity derives from the verified `Assertion` or `CapabilityToken`(s) that populated it (§18.8a.3) |

For `Identity` (`sig` is a **list**, one entry per suite): the **same** preimage
`DS-tag ‖ det_cbor(Identity ∖ {10})` is signed once per suite in `suites`, and the results are
placed in `sig` in `suites` order. Each entry is a `sig-val` per its suite (§18.2).

### 18.9.1 `Envelope.sender_sig`

Per §2.2/§2.7 the ephemeral sender signature is over the concatenation
`(id ‖ to ‖ ts ‖ kind ‖ challenge)`, **not** the whole envelope. The signed **body** is exactly:

```
body = id_bytes                       ; field 3, the full hash byte string (prefix ‖ digest)
     ‖ det_cbor(to)                   ; field 4, deterministic CBOR of the DeliveryTag map
     ‖ u64be(ts)                      ; field 6, 8 bytes big-endian
     ‖ u8(kind)                       ; field 7, 1 byte
     ‖ challenge_enc                  ; field 9: det_cbor(ChallengeResponse) if present,
                                      ;          else the single byte 0xf6 (CBOR null)
```

and the preimage is that body under the representative selected by `Envelope.suite` (field 2),
per §18.1.6 — **both forms spelled out, because the difference between them is the whole
interoperability question** (§18.9 preamble):

```
suite = 0x01 (single component, LEGACY — verify only):
  preimage = "DMTAP-v0/envelope-sender" ‖ 0x00 ‖ body
  sender_sig = Ed25519_Sign(sk_ephemeral, preimage)                       ; 64 B

suite = 0x02 / 0x03 / 0x04 / 0x05 (composite — 0x02 is what a v0 node originates, §1.1):
  preimage = "DMTAP-v0/envelope-sender" ‖ 0x00 ‖ u8(suite) ‖ body        ; the suite byte is INSIDE
  sender_sig = Ed25519_Sign(sk_e_classical, preimage) ‖ PQ_Sign(sk_e_pq, preimage)
             ; PQ_Sign = ML-DSA-65 under 0x02/0x03/0x05, SLH-DSA-128s under 0x04 (§16.7)
```

Under a composite suite **both** components sign the **same** `preimage`, both MUST verify
(AND-composition, §1.3/§18.1.6), and a verifier MUST NOT accept a component that verifies only
against the single-component form — that is what makes the composite non-separable. The `u8(suite)`
byte is the **only** difference between the two preimages; omitting it under `0x02` produces a
signature no conformant implementation will accept.

Here `(sk_ephemeral, sender_key)` is a **fresh keypair generated for this one message** and
`sender_key` (field 12) is the public half. `id_bytes` is the raw byte string of `Envelope.id`
(its CBOR head is *not* included). `to` and `challenge` are included as their deterministic CBOR
encodings so structured tags are covered unambiguously. When `challenge` is absent, exactly one
byte `0xf6` is appended (this is the only place a `null` appears in a preimage, §18.1.1). A
verifier MUST reconstruct this preimage and check `sender_sig` under **`sender_key`** (field 12,
carried in the same envelope) **before** any decryption (§2.7 step 3). Because the keypair is
ephemeral, the abuse proof in `challenge` MUST commit to `sender_key` (§9.4); otherwise a valid
proof observed on the wire could be re-attached to a forged envelope under a new ephemeral key.

### 18.9.2 `Payload.sig`

```
payload_hash = 0x1e ‖ BLAKE3-256(                ; 33 bytes — the §18.1.5 MULTIHASH form,
                 det_cbor(Payload ∖ {2})         ; prefix INCLUDED, not a bare digest
               ‖ u8(kind)                        ; Envelope field 7, 1 byte big-endian
               ‖ u64be(ts)                       ; Envelope field 6, 8 bytes big-endian
               ‖ det_cbor(to)                    ; Envelope field 4, the DeliveryTag map
               )
body         = payload_hash                      ; the §18.1.6 `body` for this object

suite = 0x01 (single component):
  preimage   = "DMTAP-v0/payload" ‖ 0x00 ‖ body
  Payload.sig = Ed25519_Sign(sk_from, preimage)                     ; sk_from = IK or device key

suite = 0x02 / 0x03 / 0x04 / 0x05 (composite):
  preimage   = "DMTAP-v0/payload" ‖ 0x00 ‖ u8(suite) ‖ body
  Payload.sig = Ed25519_Sign(sk_from_classical, preimage) ‖ PQ_Sign(sk_from_pq, preimage)
             ; PQ_Sign = ML-DSA-65 under 0x02/0x03/0x05, SLH-DSA-128s under 0x04 (§16.7)
```

**The digest is prefixed, and this is load-bearing (normative).** `payload_hash` carries its
§18.1.5 algorithm prefix — `0x1e` for the BLAKE3-256 of suites `0x01`–`0x04`, `0x16` for the
SHA3-256 of suite `0x05` — because this is one of the few preimages computed over a *digest*
rather than over the body, and a bare 32-byte digest names no algorithm. §18.1.6's general rule
governs: a verifier MUST reconstruct the prefixed form and MUST NOT accept a signature that
verifies only against an unprefixed 32-byte representative, and the prefix MUST be the one the
object's `suite` selects (§18.1.5, `ERR_HASH_ALG_MISMATCH`, `0x0127`). Earlier drafts specified
"32 bytes, **no prefix**" here, which would have handed a dual-algorithm verifier
min(BLAKE3, SHA3) collision resistance during exactly the transition suite `0x05` exists to make
routine. Regenerating the frozen `mote_payload_sig` vector was the cost of the correction, and it
was paid (`conformance/vectors/vectors.json`).

`from` (key 1) is included in the hashed body, binding the signature to the claimed sender. A
device-key signature is valid only if that device key is authorized by a current `DeviceCert`
under `from`'s `Identity` (§1.2). Verified at §2.7 step 8.

**Envelope-context binding (normative — envelope `kind`/`ts`/`to` under the *identity* signature).**
The `Envelope`'s `kind`, `ts`, and `to` are authenticated on the wire only by the **ephemeral,
anyone-can-mint** `sender_sig` (§18.9.1); for a **known contact** the abuse gate is skipped and no
challenge binds them, so an attacker who can re-emit the (already-sealed) `ciphertext` could
**re-mint** `sender_sig` under a **new** ephemeral key over an **altered** `kind`/`ts`/`to` — rewriting
the displayed timestamp and causal order, or relabeling `kind` (e.g. chat↔mail to change tier/render,
or → `0x0b` to force a Double-Ratchet decrypt-fail and thus **silently suppress** the message). The
deniable path already binds `kind` inside the ratchet AD (§18.9.10); this rule closes the same gap for
the **non-deniable** path by folding those three envelope fields into the **`Payload.sig` preimage**
(the identity signature the ephemeral key cannot forge). At **§2.7 step 8** the recipient MUST
recompute `payload_hash` **using the values in the received `Envelope`** and reject any MOTE whose
envelope `kind`/`ts`/`to` do not equal the signed context (**`ERR_ENVELOPE_CONTEXT_MISMATCH`**,
`0x0211`, §21.4) — an envelope field was altered after the payload was signed.

### 18.9.3 Identity-family objects

`Identity`, `DeviceCert`, `RecoveryPolicy`, `KeyRotation`, `MoveRecord`, `LocationRecord`,
`DomainDirectory`, `Profile` all use the general rule: `Sign(sk, DS-tag ‖ 0x00 ‖ det_cbor(object ∖ {sig-key}))`
with the DS-tags in the table above. The signing key is: IK for `Identity`/`DeviceCert`/`MoveRecord`;
IK **or** a satisfied `rotate_threshold` quorum for `RecoveryPolicy`; **`old_ik`** for `KeyRotation`;
an authorized **device key** for `LocationRecord`; the **domain authority IK** (threshold-held,
§3.10.1/§5.8.6) for `DomainDirectory`; IK **or** an `IK`-authorized device key for `Profile`
(§3.9.5, §18.4.12).

### 18.9.4 Object content addresses (`Envelope.id`, `Identity` anchor)

`Envelope.id` is the content address of the **exact bytes** of `Envelope.ciphertext`, with **no**
domain separation (so identical ciphertext dedups regardless of context, §2.8):

```
Envelope.id = 0x1e ‖ BLAKE3-256( ciphertext_bytes )           ; v0 prefix 0x1e = BLAKE3-256
```

The `Identity` anchor everyone pins (the `id` referenced from DNS/KT, §3.2) is the content
address of the identity, computed **with `sig` (key 10) excluded** — the same signature-excluded
body the DS-tagged `sig` itself already covers (§18.9.3, `Identity ∖ {10}`), **not** the complete
object:

```
Identity_id = 0x1e ‖ BLAKE3-256( det_cbor(Identity ∖ {10}) )   ; key 10 (sig) EXCLUDED
```

**Why: §1.3 forbids deriving an identifier from a signature — the same correction as
`announce_id` (§22.3.1), applied here.** An earlier draft of this formula hashed the complete,
signed `Identity`, `sig` included. §1.3 states the rule plainly: "no identifier, dedup key, or
replay-cache key in this protocol is derived from a signature … An implementation MUST NOT
introduce a construction that depends on signature uniqueness or non-malleability" — and §1.3
also concedes hybrid AND-composition gives EUF-CMA, not SUF-CMA (malleable). Hashing `sig` into
`Identity_id` did exactly what §1.3 forbids: two byte-distinct but both-valid signatures over the
same `Identity ∖ {10}` body (a re-signing under the same key, a second authorized signer under
§18.9.3's quorum rule, or a malleated hybrid component) would produce two different `Identity_id`s
for what is semantically **one** identity version — splitting the very pin (§3.2), safety-number
input (§3.4.1, below), and KT leaf (§18.4.9) this anchor exists to make singular. Excluding `sig`
closes this: the id is now a pure function of the signed content, exactly as `sig` already treats
it (§18.9.3). As with `announce_id` (§22.3.1), this is a **restoration of §1.3's invariant, not an
exception carved out of it** — §1.3 already forbade a signature-derived identifier, and the
sig-included formula was the violation the invariant already ruled out, not a case it left open.

**Consequence, stated rather than left implicit: the id is now stable across re-signing.** Because
`sig` is excluded, re-signing the identical `Identity ∖ {10}` body — a second authorized signature
added under the multi-suite `sig` array (§18.4.1 key 10), or a signature refreshed with no content
change — yields the **same** `Identity_id` with different/additional valid `sig` entries. This is
benign and intentional, and is recorded here rather than left to be discovered by surprise.

**Conformance-vector impact.** No committed vector in `conformance/vectors/vectors.json` derives
`Identity_id` from a full `Identity` object under test (the one place `identity_id_hex` appears,
the `kt_identity_leaf_hash` vector, takes it as an opaque input to the leaf-hash formula, not as
something the vector itself computes from `det_cbor(Identity)`) — so, unlike `pub_announce_id`
(§22.3.1), this correction does not invalidate a frozen vector. Any future `Identity_id` vector
MUST be generated under the `∖ {10}` formula above.

Other `hash` references to a whole **signed** object (`recovery`, `keypkgs.id`, `prev`,
`KeyPackageRef.ref`, `GroupState.log_head`) are, as a general matter, computed over the complete
referenced object — `prefix ‖ BLAKE3-256(det_cbor(referenced object))` — with no domain
separation, since a content address is a pure function of the bytes. **`Envelope.id` and
`Identity_id` above are the two exceptions carved out by this subsection**, because each is used
as a §1.3-governed identifier (a fetch/dedup/pin address) rather than a mere reference hash;
`Envelope.id` addresses ciphertext that carries no `sig` field at all, and `Identity_id` is
corrected above. Whether any of the remaining referenced-object hashes are themselves used in a
way §1.3 would also reach (each such referent does carry its own `sig` field) is not resolved by
this correction and is flagged as a follow-up audit, not silently assumed clear.

**`0x1e ‖ BLAKE3-256` above is the v0 instantiation, not the rule.** The rule is
`suite_hash_prefix ‖ suite_hash(...)`: the hash and its prefix are both selected by the object's
`suite` (§18.1.4), so under suite `0x05` every address in this subsection reads
`0x16 ‖ SHA3-256(…)`. The literals are written out because `0x02` — the v0 originating suite —
selects BLAKE3-256 and every address on the wire today carries `0x1e`. A prefix that disagrees with
the suite is a rejection, never a selection (§18.1.5, `ERR_HASH_ALG_MISMATCH`, `0x0127`).

**MLS-native referents (carve-out from the `det_cbor` rule).** Where the referent is an
**MLS-native object** — a KeyPackage referenced by `KeyPackageRef.ref`, or the bundle referenced
by `KeyPackageBundleRef.id` — the content address is computed over its **MLS/TLS wire
serialization** (RFC 9420 `tls_serialize`), not `det_cbor`:

```
mls_ref = prefix ‖ BLAKE3-256( tls_serialized_bytes )         ; v0 prefix 0x1e
```

MLS objects are **never re-encoded as DMTAP CBOR** — the address is a pure function of the bytes
RFC 9420 puts on the wire, so both sides hash the identical serialization. By contrast,
`GroupState.log_head` references the **DMTAP CBOR committer-log head** (a DMTAP object, §5.1), so
the generic `det_cbor` rule above applies to it. A **`ClusterOp` op-hash** (as recorded in a
`JournalEntry.ref`, §18.6.3) is likewise `prefix ‖ BLAKE3-256(det_cbor(ClusterOp))`, with no
DS-tag — a content address, not a signature preimage.

### 18.9.5 Merkle-DAG manifest root (`Manifest.id`, `ManifestRef.id`)

The manifest root is an **RFC 6962-style binary Merkle tree** over the ordered chunk hashes, using
the object's suite hash (v0 BLAKE3-256) with **domain-separated leaf/node prefixes** so a leaf can
never be reinterpreted as an internal node (second-preimage defense):

```
; 1. Chunking & per-chunk hash (chunks are encrypted, then hashed)
enc_i  = AEAD_encrypt(Attachment.key, plaintext_chunk_i)       ; key from the sealed MOTE, NOT the Manifest (§5.5)
h_i    = 0x1e ‖ BLAKE3-256( enc_i )                            ; the value stored in Manifest.chunks

; 2. Merkle tree over the ORDERED chunk hashes h_0 … h_{n-1}
leaf(h_i)          = BLAKE3-256( 0x00 ‖ h_i )                  ; 32-byte digest, no prefix
node(left, right)  = BLAKE3-256( 0x01 ‖ left ‖ right )         ; 32-byte digest, no prefix

MTH([h_0])                 = leaf(h_0)
MTH(h_0 … h_{n-1}), n > 1  = node( MTH(h_0 … h_{k-1}),         ; k = largest power of 2 < n
                                   MTH(h_k … h_{n-1}) )        ; (RFC 6962 split rule)

; 3. Manifest content address
Manifest.id = 0x1e ‖ MTH(h_0 … h_{n-1})
```

`ManifestRef.id` (in an `Attachment`) MUST equal the referenced `Manifest.id`. The tree uses the
RFC 6962 non-power-of-two split (no padding; unpaired subtrees carried up), so the root is
deterministic for any chunk count `n ≥ 1`. Each `h_i` self-verifies a fetched chunk; the tree
self-verifies the manifest against `id` (§5.5).

### 18.9.6 Group objects

`GroupState.committer_sig` and `GroupEvent.committer_sig` use the general rule with tags
`DMTAP-v0/group-state` / `DMTAP-v0/group-event`. These signatures assert **ordering by the
committer only**; they do NOT substitute for the member signatures inside each MLS handshake
(`GroupEvent.mls`), which every member MUST verify independently (§5.1, §5.8.2).

### 18.9.7 Anti-abuse objects

`PostageStamp.sig` and `Vouch.sig` use the general rule
(`Sign(sk_issuer|voucher, DS-tag ‖ 0x00 ‖ det_cbor(object ∖ {sig}))`) with tags
`DMTAP-v0/postage-stamp` / `DMTAP-v0/vouch`. `ArcToken` and `PowSolution` carry no DMTAP
signature of their own — an ARC presentation is verified by the ARC protocol
(`draft-ietf-privacypass-arc-protocol`) and a PoW by recomputing the Argon2id digest over
`id ‖ recipient ‖ epoch_nonce` (§16.5).

### 18.9.8 `Assertion.sig` (auth)

Per §13.3 step 5 the signature is over the hash of the origin-bound fields **including `cnf`**:

```
auth_hash    = BLAKE3-256( det_cbor([ rp_origin, nonce, issued_at, exp, aud, scope, cnf ]) )
preimage     = "DMTAP-v0/auth-assertion" ‖ 0x00 ‖ auth_hash
Assertion.sig = Sign(sk_device, preimage)          ; sk_device = the IK-authorized login signer
```

The hashed array is a fixed 7-element CBOR array in exactly that order — the five echoed
`Challenge` fields, then `scope` (`Assertion` key 9; the **empty array `[]`** when the Challenge
carries no `scope`), then `cnf` (`Assertion` key 8) — matching §13.3 step 5's
`H(rp_origin ‖ nonce ‖ issued_at ‖ exp ‖ aud ‖ scope ‖ cnf)`. The RP reconstructs it from its own
issued `Challenge` plus the assertion's `cnf`, **using exactly the scope it will grant**, and MUST
reject any mismatch of `rp_origin`/`aud`, MUST NOT grant a scope broader than the signed value
(a broader grant simply fails verification), and MUST bind the session **only** to `cnf`. The signing key is the user's **`IK`-authorized device key**
(`Assertion.from`), verified against the pinned `name → key` identity (§3.4) — **not** the session
key (which `cnf` merely commits) and not `IK` used directly for routine logins.

### 18.9.9 Mixnet objects

`MixNodeDescriptor.sig` and `MixDirectory.sig` use the general rule
(`Sign(sk, DS-tag ‖ 0x00 ‖ det_cbor(object ∖ {sig}))`) with tags `DMTAP-v0/mix-descriptor` /
`DMTAP-v0/mix-directory`. The `MixNodeDescriptor` signing key is an **`IK`-authorized device key**
of the descriptor's `node_ik` (verified via that node's `Identity`, §1.2); the `MixDirectory`
signing key is the **directory-authority IK** pinned via DNS/KT (§4.4.2, threshold-held per
§5.8.6 where a set/quorum is used). The **Sphinx per-hop wrapping** of a MOTE is **not** a DMTAP
CBOR signature — it is the Sphinx packet construction of §4.4.1 (per-hop MAC + re-randomized
group element), verified by each mix peeling its layer, and is out of scope for this preimage
table (it carries no `sig-val` field).

### 18.9.10 Deniable-mode objects (§5.2.1)

The deniable 1:1 mode has a **deliberately asymmetric** signing story, because signatures are
what would destroy deniability:

- **`DeniablePrekeyBundle.sig`** (key 10) uses the general rule
  `Sign(sk_device, "DMTAP-v0/deniable-prekeys" ‖ 0x00 ‖ det_cbor(DeniablePrekeyBundle ∖ {10}))`
  with an `IK`-authorized device key. This signs the *bundle of public prekeys*, not any message.
- **`DeniablePrekeyBundle.spk_sig`** (key 4) is the standard X3DH **signed-prekey signature**:
  `Sign(sk_device, "DMTAP-v0/deniable-spk" ‖ 0x00 ‖ spk_bytes)` over the raw `spk` public key
  (field 3). It proves the prekey was published by the identity; it signs **no** content and
  binds **no** transcript, so it does not make any conversation attributable (§5.2.1(a)).
- **`DeniablePrekeyBundle.idk_sig`** (key 12) and the identical **`DeniableInit.idk_a_cert`**
  (key 10) are the **deniable-identity DH-key certification**:
  `Sign(sk_device, "DMTAP-v0/deniable-idk" ‖ 0x00 ‖ idk_bytes)` over the raw X25519 `idk` /
  `idk_a` public key. This is what **replaces the retired XEdDSA-from-`IK` derivation**: instead
  of `IK` *being* the long-term identity DH key, a **dedicated X25519 `idk`** is certified once by
  an `IK`-authorized device key. Like `spk_sig` it signs a *public DH key*, never any message, so
  deniability is unchanged — no long-term signature ever covers content or a transcript. Keeping
  the DH key separate from `IK` also lets `IK` live in a usage-fixed hardware keystore that only
  signs (Secure Enclave P-256 / TPM / StrongBox), which cannot perform both signing and DH on one
  key.
- **`DeniableInit`, `DeniableMessage`, and `DeniablePayload` carry NO DMTAP signature at all.**
  Their integrity/authentication is the **Double Ratchet AEAD tag** — a **shared-key MAC** under a
  per-message key that *both* parties can derive from the X3DH/PQXDH shared secret. Because either
  party could have produced any such tag, no tag is a transferable proof of authorship — this is
  the cryptographic root of participation + message repudiation. A `DeniablePayload` presented
  with any signature field MUST be rejected (`ERR_DENIABLE_SIGNATURE_PRESENT`, `0x040F`); a
  `DeniableMessage` whose AEAD tag fails is `ERR_DENIABLE_RATCHET_AUTH_FAILED` (`0x040D`). The AEAD
  associated data is the standard Double-Ratchet AD — the ratchet header `(dh, pn, n)` with the
  X3DH context `AD = IK_A ‖ IK_B` — binding the message to the session (it does **not** bind
  `Envelope.id`, which is the hash of the ciphertext that already contains this tag). This mirrors
  the `ArcToken`/`PowSolution` case (§18.9.7): a wire object whose security comes from a *different*
  proof system, not a DMTAP `sig-val`.

The envelope that carries a deniable frame still bears `Envelope.sender_sig` (§18.9.1), but that
is a **fresh per-message ephemeral** signature over routing metadata that binds **no long-term
identity** — it gates abuse (§9) without attributing the transcript, so it is deniability-neutral.

### 18.9.11 `GatewayAttestation.sig` and `msg_digest`

The attestation signature uses the general rule under the **domain-anchored** `_dmtap-gw` key
(§7.2a), **not** any DMTAP identity key and **not** the gateway's DKIM key (§7.3):

```
msg_digest  = 0x1e ‖ BLAKE3-256( rfc5322_bytes )     ; the EXACT legacy bytes the gateway wrapped
preimage    = "DMTAP-v0/gateway-attest" ‖ 0x00 ‖ det_cbor(GatewayAttestation ∖ {7})
sig         = Sign(sk_gw_attest, preimage)           ; sk_gw_attest ↔ <selector>._dmtap-gw.<domain> "k="
```

`msg_digest` (key 4) is inside the signed body, so the signature binds the attestation to **this
specific message**: a verifier recomputes `0x1e ‖ BLAKE3-256` over the decrypted RFC 5322/MIME
body and MUST reject a mismatch (`ERR_GATEWAY_ATTESTATION_INVALID`, `0x0601`) — an attestation
lifted onto other content fails this bind. The verifying key is looked up at
`<selector>._dmtap-gw.<domain>` (DNS + optional KT anchor, §7.2a); a key not published there, or
under a `domain` the recipient does not trust for the recipient-domain entry, is untrusted
(`ERR_GATEWAY_ATTESTATION_KEY_UNTRUSTED`, `0x0602`). Verification runs **after** decryption
(the attestation is sealed in `Payload`), at `deliver` step 8a (§19.3.1).

### 18.9.12 `ProvenanceRecord` carries no signature

`ProvenanceRecord` (§18.8.1) is **not signed and not transmitted over the mesh**: it is a
node-local, client-facing assembly served only to the owner's own devices (§8.6, §19.9). Its
`gateways` field consists of `GatewayAttestation` entries **already** verified per §18.9.11; the
record itself needs no signature because it never crosses a trust boundary — it is authenticated
by the authenticated client channel it is delivered on (§8.1), exactly like a `Mailbox` CRDT view
(§5.6). Adding a signature would serve nothing and is not defined.

### 18.9.13 Key-transparency STH (`SignedTreeHead.sig`)

The STH is signed **by the log's own key** (`log_id`, field 2), not by any DMTAP identity, using
the general rule:

```
preimage = "DMTAP-v0/kt-sth" ‖ 0x00 ‖ det_cbor(SignedTreeHead ∖ {6})
SignedTreeHead.sig = Sign(sk_log, preimage)          ; sk_log ↔ log_id (field 2)
```

A verifier reconstructs the preimage and checks `sig` under `log_id`; failure is
`ERR_KT_PROOF_INVALID` (`0x0108`). `InclusionProof` and `ConsistencyProof` (§18.4.10/§18.4.11)
carry **no** signature — they are verified *arithmetically against* an STH's `root_hash` (folding
the RFC 6962 audit/consistency path with the §18.9.5 domain-separated node prefix), so their
integrity derives from the signed head, not a signature of their own (like `ProvenanceRecord`,
§18.9.12). The **leaf hash** they commit to is computed by the Identity-entry rule of §18.4.9
(`0x1e ‖ BLAKE3-256(0x00 ‖ det_cbor([name, ik, version, identity_id]))`); a leaf that does not
recompute is `ERR_KT_LEAF_HASH_MISMATCH` (`0x0117`).

### 18.9.14 Capability token & revocation (`CapabilityToken.sig`, `CapabilityRevocation.sig`)

Both use the general rule under the **issuer** key:

```
CapabilityToken.sig       = Sign(sk_iss, "DMTAP-v0/cap-token"      ‖ 0x00 ‖ det_cbor(CapabilityToken ∖ {9}))
CapabilityRevocation.sig  = Sign(sk_iss, "DMTAP-v0/cap-revocation" ‖ 0x00 ‖ det_cbor(CapabilityRevocation ∖ {5}))
```

`sk_iss` is the issuer's `IK` or an `IK`-authorized device key (for a root link, §13.5), or — for a
delegated link — the key that is the parent token's `aud` (the chain binds each link's `iss` to its
parent's `aud`, §18.7.3). A signature/attenuation/expiry failure anywhere in the chain is
`ERR_CAPABILITY_DELEGATION_INVALID` (`0x0508`); a covering revocation is `ERR_CAPABILITY_REVOKED`
(`0x050B`). The `Capability` sub-map has no signature of its own — it is covered by the enclosing
token's `sig`. **`SphinxCell` and its sub-structures (§18.5.4) carry no DMTAP `sig-val`** — their
integrity is the Sphinx per-hop MAC / wide-block PRP (§4.4.1, §18.9.9), the same
"security-from-a-different-proof-system" case as `ArcToken`/deniable frames.

### 18.9.15 Push wake-signaling objects (`PushSubscription.sig`, `WakePing`) (§4.9)

`PushSubscription.sig` (key 7) uses the general rule
`Sign(sk_device, "DMTAP-v0/push-subscription" ‖ 0x00 ‖ det_cbor(PushSubscription ∖ {7}))` with an
`IK`-authorized device key (§1.2) — the same signing discipline as `LocationRecord` (§18.9.3). It
authenticates the subscription to the identity so no other party can register or redirect a device's
wakes; a signature that does not verify is `ERR_PUSH_SUBSCRIPTION_SIG_INVALID` (`0x0312`),
fail-closed. The verifier MUST also confirm `device_key` (field 5) is authorized by a current
`DeviceCert` under the owner's `Identity`.

`WakePing` carries **no** DMTAP `sig-val`. Its authentication is the **RFC 8291 `aes128gcm` AEAD
tag**, computed under a key derived (RFC 8291 HKDF) from the device `push_key` and the
subscription's `auth_secret` — secrets held **only** by the device and the user's own node — so only
that node can produce a `WakePing` the device will open, and the push relay (which lacks the auth
secret) can neither read nor forge one. A `WakePing` whose AEAD fails to open is a
forged/unauthenticated wake and is dropped (`ERR_WAKEPING_AUTH_FAILED`, `0x0314`, DROP_SILENT); a
`WakePing` (or its opened plaintext) carrying any field beyond the opaque sync token is
`ERR_WAKEPING_CONTENT_PRESENT` (`0x0313`), fail-closed; wakes are rate-limited per device
(`ERR_WAKEPING_RATE_LIMITED`, `0x0315`, §4.9.4). This mirrors the deniable-mode objects (§18.9.10)
and `SphinxCell` (§18.9.9): a wire object whose security rests on a different, well-specified proof
system — here RFC 8291 Web Push encryption — not on a DMTAP signature.

### 18.9.16 Committer-rank preimage (§5.1)

The deterministic committer-election rank (§5.1) is a domain-separated hash, computed
identically by every member so the election needs no extra round-trips:

```
rank = BLAKE3-256( "DMTAP-v0/committer-rank" ‖ 0x00
                 ‖ member_signing_key_bytes            ; raw MLS leaf signature public key
                 ‖ group_id_bytes                      ; GroupState.group_id (field 1), raw bytes
                 ‖ u64be(epoch) )                      ; the MLS epoch number, 8 bytes big-endian
```

`member_signing_key_bytes` is the member's **raw MLS leaf signature public key** exactly as it
appears in the ratchet tree (RFC 9420), with no CBOR head; `group_id_bytes` is the raw byte
string of `GroupState.group_id`. This is a **ranking hash, not a signature** — it carries no
`sig-val` and proves nothing by itself; the DS-tag only ensures a rank can never collide with any
other DMTAP preimage. Members order candidates by `rank` (lowest first) per §5.1.

**Unsigned and unlabelled — disclosed, not fixed (v0).** `rank` carries neither a signature nor a
§18.1.5 algorithm prefix, so §18.1.6's prefixed-digest rule does not reach it: it is not a
signature over a digest, it is a bare ordering value. The consequence of the missing label is
narrow but real — two members running **different** content-hash suites (§1.1) would compute
different ranks over the same inputs and could elect different committers, which the group's own
fork detection then reports as a committer fork (`0x0404`, §5.1). Because a group already agrees
on one `GroupState.suite` (§18.1.4) before any election happens, this cannot arise inside a
conformant group, and the correct time to bind the algorithm is a `GroupState.suite` migration,
which is a group-wide event with a rekey attached. It is recorded here rather than left as an
omission (the same posture as §18.9.17's).

### 18.9.17 Key-name digest (§3.9.6)

The **key-name** — the zero-authority floor of the naming ladder (§3.13) — is the one hash in the
protocol §18.9 did not pin. §3.9.6 says only "a word encoding of `BLAKE3-256(ik)`", and `ik` is
not a value: `Identity.iks` is a **map** of one identity public key *per suite* (§18.4), whose
entries differ in length between suites (§16.7). Left unpinned, one implementation could hash
`iks[anchor_suite]`, another `iks[suite]`, another the deterministic CBOR of the whole map — and
the same identity would have **three different 8-word key-names**. Since the key-name is read
aloud and compared by humans, that is a spoofing surface as well as an interoperability break, and
it is load-bearing for §3.13, §9.7a and §12.3.1.

Pinned:

```
keyname_digest = H( u8(0x01)          ; keyname derivation VERSION byte
                  ‖ u8(hash_prefix)   ; the §18.1.5 prefix of H itself: 0x1e for BLAKE3-256
                  ‖ ik_pub_bytes )    ; ik_pub_bytes = Identity.iks[anchor_suite]

; v0: H = BLAKE3-256, hash_prefix = 0x1e, so the input is 0x01 ‖ 0x1e ‖ ik_pub_bytes
```

- **The hash names itself (normative — the agility hook the floor previously lacked).** The
  key-name is the **zero-authority floor** (§3.13): it is derived, not published, so it is the one
  identifier in the protocol that no signed object carries and no `suite` byte governs. Before this
  correction it had **no** agility hook of any kind — no DS-tag, no multihash prefix, no suite
  byte — which meant a hash migration (suite `0x05`, §1.1) would silently change **every key-name
  in existence without any key rotating**. That is the worst possible shape for a naming event: the
  signed `KeyRotation`/`MoveRecord` chain that lets correspondents follow a name change (§1.5–§1.6)
  is driven by a *key* changing, so it would not fire at all, and the old and new key-names of the
  same unchanged key would be indistinguishable — both 8 words, both valid, neither carrying any
  evidence of which digest produced it. Committing the algorithm **inside the hashed input** fixes
  this at the only place available: the input. A migration to a different `H` yields a *different*
  and therefore **distinguishable** key-name, which a client can recognise as a derivation change
  rather than mistake for a different identity, and the version byte leaves room for a future
  derivation change that is not a hash change.
- **Why a version byte as well as a prefix.** `u8(0x01)` versions the *derivation*; the prefix
  versions the *primitive*. They are separate axes for the same reason §18.1.4 and §18.1.5 are
  (a suite is a set; a hash is one member of it), and conflating them is how an agility hook ends
  up unable to express the change that actually arrives.
- **Which key: the ANCHOR.** `ik_pub_bytes` is the entry of `Identity.iks` at
  `Identity.anchor_suite` (§1.2.0), taken as **raw public-key bytes with no CBOR head, no length
  prefix and no suite byte**. The anchor is the correct input because the key-name must be as
  durable as the identity itself: operational suites are expected to migrate (§1.1), and hashing an
  operational key would change a user's zero-authority name every time they rotated one.
- **Still no domain-separation tag** — deliberately, and this remains the one §18.9 preimage
  without one. The DS-tag collision class does not arise: a key-name is never compared as raw
  digest bytes against another DMTAP digest; it is rendered to words and compared *as words*. The
  algorithm-binding above is a **different** argument and does not carry over to domain separation:
  it exists because a hash migration changes the derivation with no key-rotation event to signal
  it, which is a live failure mode, whereas cross-preimage collision here is not. The two were
  previously conflated — an earlier draft cited the cost of invalidating the committed `keyname_*`
  conformance vectors as the reason for having **no** hook at all. That cost is real and has now
  been paid once (see below); it was never a reason to leave the floor unmigratable.
- **This change invalidates the committed `keyname_*` vectors — stated plainly.** Every
  `keyname_encode` known-answer vector in `conformance/vectors/vectors.json` was generated under
  the bare `BLAKE3-256(ik_pub_bytes)` derivation and is **wrong** under this one. They are removed
  rather than silently retained, and the affected cases (`DMTAP-NAME-01`…`-05`) revert to
  construction-todo with the recipe above until the reference core regenerates them. The
  checksum-failure case (`DMTAP-NAME-06`) is unaffected: a mistyped word fails the folded checksum
  regardless of which digest produced the name.
- **Consequence, stated because §1.2.0 implies it without saying it: an anchor-suite migration
  changes every key-name.** Rotating the anchor — the emergency `0x04` pivot (§1.1) — is therefore
  a **network-wide naming event**, not merely a cryptographic one. Existing correspondents follow
  by pinned key and the signed chain (§1.5–§1.6) and are unaffected; only a new contact who knows
  solely the old key-name is. This is the same residual §3.9.6 already discloses for `IK` rotation,
  and the reason the anchor should be the most conservative primitive available.

The word encoding, wordlist size and folded checksum are §3.9.6's; this subsection pins only the
digest that feeds them.

---

## 18.10 Collected CDDL grammar (copy-paste block)

The following is the complete, self-contained CDDL for every DMTAP wire object. An implementer
MAY copy this single block into a CDDL tool (RFC 8610) as the normative schema. It is internally
consistent and consistent with §1, §2, §5, §13, coordinator/CONTRACT.md.

```cddl
; ═══════════════════════════════════════════════════════════════════
;  DMTAP v0 — Wire Format (Appendix A, §18)
;  Deterministic CBOR (RFC 8949 §4.2); integer-keyed maps (COSE/CWT).
; ═══════════════════════════════════════════════════════════════════

; ── scalar aliases ─────────────────────────────────────────────────
u8      = uint .size 1
u16     = uint .size 2
u32     = uint .size 4
u64     = uint
ts      = u64                          ; ms since Unix epoch
suite   = 0x01..0xff                   ; DMTAP algorithm suite

; ── crypto byte strings (lengths suite-governed, §18.2) ────────────
hash    = bytes .size (33..129)        ; 1-byte alg prefix ‖ digest; v0 = 33
ik-pub  = bytes                        ; v0 Ed25519 = 32 B
sig-pub = bytes                        ; ephemeral sender_key; v0 Ed25519 = 32 B
sig-val = bytes                        ; v0 Ed25519 = 64 B
enc-key = bytes                        ; v0 = 32 B
peer-id = bytes
maddr   = tstr

; ── message layer (§2) ─────────────────────────────────────────────
Envelope = {
  1 => u8, 2 => suite, 3 => hash, 4 => DeliveryTag,
  ? 5 => bytes, 6 => ts, 7 => u8,
  ? 8 => KeyPackageRef, ? 9 => ChallengeResponse,
  10 => bytes, 11 => sig-val, 12 => sig-pub,
}

DeliveryTag = KeyTag / GroupTag / BlindedTag
KeyTag      = { 0 => 1, 1 => ik-pub }
GroupTag    = { 0 => 2, 1 => bytes }
BlindedTag  = { 0 => 3, 1 => bytes }

ChallengeResponse = ArcToken / PowSolution / PostageStamp / Vouch
ArcToken     = { 0 => 1, 1 => bytes, 2 => bytes, 3 => bytes, ? 4 => bytes }
PowSolution  = { 0 => 2, 1 => tstr, 2 => [u32, u32, u32],
                 3 => bytes, 4 => bytes, 5 => u8 }
PostageStamp = { 0 => 3, 1 => bytes, 2 => bytes, 3 => u64, 4 => tstr,
                 5 => ts, ? 6 => bytes, 7 => sig-val }
Vouch        = { 0 => 4, 1 => ik-pub, 2 => ik-pub, 3 => ik-pub,
                 4 => ts, 5 => sig-val }

KeyPackageRef = { 1 => hash, 2 => suite, ? 3 => tstr }

Payload = {
  1 => ik-pub, 2 => sig-val, 3 => Headers, 4 => Body,
  5 => [* hash], 6 => [* Attachment], ? 7 => ts, ? 8 => bytes,
  ? 9 => [+ GatewayAttestation],   ; 9 = sealed gateway-attestation chain (§18.3.11, §7.8); absent ⇒ pure-mesh
}

; Sealed inside Payload; signed by the domain-anchored _dmtap-gw key (§7.2a, §18.9.11), NOT a
; DMTAP identity or DKIM key. Present iff gateway-touched (legacy-origin); absent ⇒ pure-mesh.
GatewayAttestation = {
  0 => 1, 1 => tstr, 2 => tstr, 3 => ts, 4 => hash,
  ? 5 => tstr, ? 6 => u8, 7 => sig-val,
}

Headers = {
  ? 1 => bytes, ? 2 => tstr, ? 3 => tstr,
  4 => [* ik-pub], ? 5 => { * tstr => ext-value }, ? 6 => bool,   ; 6 = sensitive (no-persist, §6.7)
}
ext-value = bool / int / bytes / tstr / [* ext-value] / { * tstr => ext-value }
Body = tstr / bytes

; ── deniable 1:1 mode (§5.2.1); kind = 0x0b; NO sig-val anywhere (repudiable) ───────
DeniableFrame   = DeniableInit / DeniableMessage
DeniableInit    = {
  0 => 1, 1 => suite, 2 => ik-pub, 9 => bytes, 10 => sig-val,   ; 9=idk_a (X25519 identity DH key), 10=idk_a_cert
  3 => bytes, 4 => hash,
  ? 5 => hash, ? 6 => bytes, ? 7 => hash, 8 => DeniableMessage,
}
DeniableMessage = { 0 => 2, 1 => bytes, 2 => u32, 3 => u32, 4 => bytes }   ; 4 = AEAD ct; tag IS the shared-key MAC
DeniablePayload = {
  1 => ik-pub, 2 => u8, 3 => Headers, 4 => Body,
  5 => [* hash], 6 => [* Attachment], ? 7 => ts,             ; NO signature field (§18.3.10)
}

Attachment = {
  1 => tstr, 2 => tstr, 3 => u64,
  ? 4 => bytes, ? 5 => ManifestRef, 6 => enc-key,   ; key lives HERE (sealed MOTE), never in Manifest
}
ManifestRef = { 1 => hash, 2 => u64, 3 => u32 }

; Manifest has NO key field (§5.5/§18.3.8): it is a swarm-distributed content-addressed blob,
; so an embedded key would leak to every holder that serves it. Key 5 is reserved-FORBIDDEN;
; a Manifest carrying key 5 MUST be rejected (ERR_MANIFEST_KEY_PRESENT, 0x0808).
Manifest = {
  1 => hash, 2 => u64, 3 => u32, 4 => [+ hash], 6 => suite,
}

; ── gateway alias mapping (§7.10) ──────────────────────────────────
; Gateway-LOCAL state for the RANDOM alias mode; NOT mesh-transmitted, NO signature. The ENCODED
; mode needs no row (the alias localpart.nativedomain@gw is a reversible transform, §7.10.2).
GatewayAliasMap = {
  1 => tstr, 2 => tstr, 3 => u8, ? 4 => tstr, 5 => ts, ? 6 => ts, ? 7 => bool,
  ; 1=alias 2=native 3=mode(1 encoded/2 random) 4=correspondent 6=expires 7=burned
}

; ── identity layer (§1) ────────────────────────────────────────────
Identity = {
  1 => [+ u8], 2 => { + u8 => ik-pub }, 3 => u64,
  4 => [* DeviceCert], 5 => KeyPackageBundleRef, 6 => hash,
  7 => [* tstr], ? 8 => hash, 9 => ts, 10 => [+ sig-val],
  ? 11 => KeyPackageBundleRef,                              ; 11 = deniable_prekeys (§5.2.1); sig stays key 10
}

DeviceCert = {
  1 => suite, 2 => ik-pub, 3 => ik-pub, 4 => tstr,
  5 => ts, ? 6 => ts, 7 => [+ tstr], 8 => sig-val,
  ? 9 => key-protection, ? 10 => bytes,                    ; 9 = keystore class, 10 = attestation (§1.2a)
}
key-protection = "software" / "tpm" / "secure-enclave" / "strongbox" / "tee"

KeyPackageBundleRef = { 1 => tstr, 2 => hash, ? 3 => [+ u8] }

; Published X3DH/PQXDH prekeys for the deniable mode (§5.2.1); located via Identity.deniable_prekeys.
; idk (key 11) is a DEDICATED long-term X25519 identity-DH key (NOT derived from IK), certified by
; idk_sig (key 12); spk_sig (key 4) signs the PUBLIC prekey only; sig (key 10) signs the bundle.
; No message is ever signed. ik (key 2) is the Ed25519 IK, used for AD binding only, never for DH.
DeniablePrekeyBundle = {
  1 => suite, 2 => ik-pub, 11 => bytes, 12 => sig-val,          ; 11=idk (X25519), 12=idk_sig (DMTAP-v0/deniable-idk)
  3 => bytes, 4 => sig-val, 5 => [* bytes],
  ? 6 => bytes, ? 7 => [* bytes], 8 => u64, 9 => ts, 10 => sig-val,
}

RecoveryPolicyRef = hash
RecoveryPolicy = {
  1 => suite, 2 => ik-pub, 3 => u64, 4 => [+ RecoveryMethod],
  5 => Threshold, 6 => Threshold, ? 7 => hash, 8 => ts, 9 => sig-val,
}
RecoveryMethod = PhraseMethod / DeviceMethod / SocialMethod
PhraseMethod   = { 0 => 1, 1 => bytes }
DeviceMethod   = { 0 => 2, 1 => ik-pub, 2 => tstr }
SocialMethod   = { 0 => 3, 1 => [+ ik-pub], 2 => u8 }
Threshold      = { 1 => [+ MethodPredicate] }
MethodPredicate = { 1 => method-type, 2 => uint }
method-type    = "phrase" / "device" / "social" / "ik"

KeyRotation = {
  1 => suite, 2 => ik-pub, 3 => ik-pub, 4 => tstr,
  5 => ts, ? 6 => hash, 7 => sig-val,
}

MoveRecord = {
  1 => suite, 2 => ik-pub, 3 => tstr, 4 => tstr,
  5 => ts, ? 6 => hash, 7 => sig-val,
}

DomainDirectory = {
  1 => suite, 2 => tstr, 3 => ik-pub, 4 => u64,
  5 => dir-visibility, 6 => [* DirEntry], ? 7 => hash, 8 => ts, 9 => sig-val,
}
DirEntry       = { 1 => tstr, 2 => ik-pub, 3 => hash, 4 => member-custody, ? 5 => [* tstr], 6 => ts, ? 7 => alloc-tier }
dir-visibility = "public" / "members-only"
member-custody = "sovereign" / "org-managed"
alloc-tier     = "random" / "vanity" / "byod"     ; provisioning tier disclosure (§3.11.2)

; Self-asserted, signed human display data (§3.9.5); a REPLACEABLE pointer, not an identity claim.
; sig (key 10) by IK or an IK-authorized device key (DMTAP-v0/profile). Avatar is owner-hosted;
; avatar.hash (0x1e ‖ BLAKE3-256 of the image) gives tamper-evidence without DMTAP hosting the image.
Profile = {
  1 => suite, 2 => ik-pub, 3 => u64, 4 => tstr,
  ? 5 => tstr, ? 6 => tstr, ? 7 => Avatar, ? 8 => hash, 9 => ts, 10 => sig-val,
}
Avatar = { 1 => tstr, ? 2 => hash }

; ── key transparency (§3.5); RFC 6962-profiled. STH signed by the LOG's key (log_id).
; InclusionProof/ConsistencyProof are UNSIGNED — verified against an STH root_hash (§18.9.13).
; Identity leaf-hash = 0x1e ‖ BLAKE3-256(0x00 ‖ det_cbor([name, ik, version, identity_id])) (§18.4.9).
SignedTreeHead = { 1 => suite, 2 => bytes, 3 => u64, 4 => ts, 5 => hash, 6 => sig-val }  ; 2=log_id
InclusionProof   = { 1 => u64, 2 => u64, 3 => hash, 4 => [* hash] }   ; tree_size, leaf_index, leaf_hash, audit_path
ConsistencyProof = { 1 => u64, 2 => u64, 3 => [* hash] }              ; first_size, second_size, proof_path

; ── transport layer (§4) ───────────────────────────────────────────
LocationRecord = {
  1 => ik-pub, 2 => peer-id, 3 => [* maddr],
  4 => u64, 5 => u64, 6 => ts, ? 8 => u8, 7 => sig-val,   ; 8 = substrate tag (§21.24); absent ⇒ 0x01 libp2p
}

; ── mixnet layer (§4.4) ────────────────────────────────────────────
; A signed descriptor for one mix node, and the signed per-epoch directory of them.
MixKeyEntry     = { 1 => u64, 2 => enc-key, 3 => ts }        ; epoch, Sphinx mix public key, valid-until
MixNodeDescriptor = {
  1 => suite, 2 => ik-pub, 3 => [* maddr], 4 => [+ MixKeyEntry],
  5 => mix-layer, ? 9 => ik-pub, ? 8 => u8, 6 => ts, 7 => sig-val,  ; 9=operator (§4.4.8); 8=substrate (§21.24, absent⇒0x01 libp2p)
}
mix-layer       = 0..2                                       ; stratified position: 0=entry,1=middle,2=exit
MixDirectory = {
  1 => suite, 2 => ik-pub, 3 => u64, 4 => u64, 5 => [+ MixNodeDescriptor],
  6 => hash, 7 => ts, 8 => sig-val,                          ; 3=epoch 4=version 6=prev-directory hash
}
; NOTE: the Sphinx SphinxCell / RoutingCommand / SURB / SphinxFragmentHeader (§18.5.4) are
; FIXED-LENGTH BYTE LAYOUTS on the mixnet wire, NOT deterministic CBOR, so they are specified as
; byte tables in §18.5.4 and are deliberately not encoded as CDDL rules here.

; ── push wake-signaling (§4.9, OPTIONAL) ───────────────────────────
; PushSubscription is held only on the user's OWN node; WakePing is content-free & sender-blind.
PushSubscription = {
  1 => u8, 2 => tstr, 3 => bytes, 4 => bytes, 5 => ik-pub, 6 => ts, 7 => sig-val,
  ; 1=provider(1 UnifiedPush/2 Web Push/3 APNs/4 FCM) 3=push_key(P-256) 4=auth_secret(RFC 8291) 5=device_key
}
WakePing = { 1 => bytes }   ; RFC 8291-sealed opaque sync token ONLY; NO sig-val; any other field ⇒ 0x0313

; ── group layer (§5.8) ─────────────────────────────────────────────
GroupState = {
  1 => bytes, 2 => suite, 3 => bytes, ? 4 => tstr, 5 => ik-pub,
  6 => posting-model, 7 => visibility, 8 => join-policy,
  9 => [+ RosterEntry], 10 => hash, 11 => u64, 12 => ts, 13 => sig-val,
  14 => hash,
}
RosterEntry   = { 1 => ik-pub, 2 => [+ role], 3 => ts }
posting-model = "broadcast" / "collaborative"
visibility    = "hidden" / "visible"
join-policy   = "closed" / "request" / "open" / "vouch"
role          = "owner" / "admin" / "member" / "poster" / "reader"

GroupEvent = {
  1 => bytes, 2 => bytes, 3 => bytes, 4 => u64, 5 => hash, 6 => sig-val,
}

; ── device-cluster sync (§5.6) ─────────────────────────────────────
; Carried INSIDE the encrypted MLS cluster group; NO DMTAP sig (MLS + DeviceCert authenticate,
; §18.6.3). RangeFingerprint is self-verifying by recomputation. dir-visibility/alloc-tier above.
ClusterSyncFrame = {
  1 => u8, 2 => ik-pub, ? 3 => [* hash], ? 4 => [* RangeFingerprint],
  ? 5 => [* ClusterOp], ? 6 => [* JournalEntry], ? 7 => [* StabilityMark],
  ; 1=type(1 announce/2 recon/3 fetch/4 journal/5 stability) 2=device
}
RangeFingerprint = { 1 => hash, 2 => hash, 3 => u64, 4 => hash }   ; lo, hi(excl), count, fp(over sorted ids)
HLC              = { 1 => u64, 2 => u32, 3 => ik-pub }             ; wall, counter, device — order (wall,counter,device)
AddTag           = { 1 => ik-pub, 2 => HLC }                       ; OR-Set add-tag {device, hlc}
ClusterOp = {
  1 => u8, 2 => tstr, ? 3 => tstr, ? 4 => ext-value, 5 => HLC, ? 6 => [+ AddTag],
  ; 1=kind(1 set-add/2 set-remove/3 lww-set) 2=target 3=field 4=value 5=hlc 6=observed add-tags(remove)
}
JournalEntry  = { 1 => u64, 2 => hash, 3 => hash }                 ; seq, prev, ref(object id or op hash)
StabilityMark = { 1 => ik-pub, 2 => HLC }                          ; device, max-applied HLC

; ── auth layer (§13) ───────────────────────────────────────────────
Challenge = {
  1 => tstr, 2 => bytes, 3 => ts, 4 => ts, 5 => tstr, ? 6 => [* tstr],
}
Assertion = {
  1 => tstr, 2 => bytes, 3 => ts, 4 => ts, 5 => tstr,
  6 => ik-pub, 7 => sig-val, 8 => hash,
}

; Delegated capability (§13.5, §13.5.1) — a profile of UCAN v1.0. Chained via `prnt`; each link
; MUST only narrow its parent (attenuation). Verified OFFLINE; revoked by a KT-logged CapabilityRevocation.
CapabilityToken = {
  1 => suite, 2 => ik-pub, 3 => ik-pub, 4 => [+ Capability],
  5 => u64, 6 => u64, 7 => bytes, ? 8 => hash, 9 => sig-val,   ; 2=iss 3=aud 5=nbf 6=exp 8=prnt
}
Capability = { 1 => tstr, 2 => tstr, ? 3 => { * tstr => ext-value } }   ; resource, ability, caveats
CapabilityRevocation = { 1 => suite, 2 => ik-pub, 3 => hash, 4 => ts, 5 => sig-val }  ; 3=revoked token addr

; ── coordinator layer (coordinator/CONTRACT.md §2.1, §2.4, §6; §12.2) ──
; One object family for every coordinator kind (CONTRACT §5), keyed by `kind`; a gateway's/
; adapter's kind-specific fields (§7.5, §26.3.1) live inside the opaque `policy` blob.
CoordinatorDescriptor = {
  1 => tstr, 2 => ik-pub, 3 => Visibility, 4 => bytes, ? 5 => Tariff, 6 => sig-val,
}
Visibility = { 1 => tstr, 2 => tstr }                        ; class, level
Tariff     = { 1 => ik-pub, 2 => bytes, 3 => sig-val }        ; self-certifying: own identity, not the descriptor's
UsageReceipt = { 1 => ik-pub, 2 => bytes, 3 => sig-val }      ; self-certifying, delivered directly to the payer

; GatewayAuthz is gateway-LOCAL state (like GatewayAliasMap): NOT mesh-transmitted, NO signature
; of its own — authenticity derives from the Assertion/CapabilityToken(s) that populated it.
; `grants` entries reuse the existing CapabilityToken (§13.5/§18.7.3) with
; Capability.resource = "gw-addr:"+address or "gw-rail:"+rail+":"+remote_id, ability = "send-as".
GatewayAuthz = {
  1 => ik-pub, 2 => u8, 3 => ts, ? 4 => [* hash], ? 5 => ts, ? 6 => bool,
  ; 1=identity 2=mode(1 open/2 key-registered) 3=granted_at 4=grants 5=expires 6=revoked
}

; ── client-facing provenance (§7.8, §8.6, §19.9) ───────────────────
; NODE-LOCAL: assembled by the recipient node, served only to the owner's own devices; NOT
; mesh-transmitted, NOT signed (§18.9.12). `gateways` are GatewayAttestations already verified
; (§18.9.11). min_hops is a COARSE profile floor, NEVER mix-node identities (§6.8).
ProvenanceRecord = {
  1 => u8, 2 => u8, 3 => u8, 4 => [* GatewayAttestation],
  ? 5 => u8, ? 6 => ts,
}
```

---

## 18.11 Source inconsistencies flagged (not silently diverged)

While formalizing, the following discrepancies among §1/§2/§4/§5/§16 were found. This appendix
resolves each explicitly rather than picking one silently; each SHOULD be reconciled in the prose
(§10.4):

1. **`Identity.keypkgs` vs `Identity.prekeys`.** RECONCILED. §1.3's object names the field
   `keypkgs` (type `KeyPackageBundleRef`) and §5.3's prose now uses `keypkgs` as well; the earlier
   `prekeys` name is retired. This appendix uses **`keypkgs`** — key 5 of `Identity`.

2. **Two distinct KeyPackage reference types.** §2.2's `Envelope.keypkg` is a `KeyPackageRef`
   (a single consumed package); §1.3's `Identity.keypkgs` is a `KeyPackageBundleRef` (the whole
   published bundle). The names are close enough to conflate. This appendix keeps them as two
   separate objects (§18.3.4, §18.4.3) and documents the distinction.

3. **`LocationRecord` missing `seq`.** RECONCILED. §16.2 normatively requires a monotonic
   `Location seq-number (u64)` for rollback defense; §4.2's inline CBOR now carries it and this
   appendix specifies it as a REQUIRED field (key 4, §18.5.1). Appendix and prose agree.

4. **`chunks` means two different things.** In `ManifestRef` (§2.5) `chunks` is a `u32` **count**;
   in `Manifest` (§5.5) `chunks` is a `[+ bytes]` **list of hashes**. Both are retained with their
   source meanings and the collision is called out in §18.3.7/§18.3.8. Recommend renaming
   `ManifestRef.chunks` → `chunk_count` in a future revision to remove the overload.

5. **`Threshold` predicate type.** RECONCILED. §1.4 now defines `MethodPredicate` as naming a
   satisfied factor — `Phrase` / `Devices(n)` / `Guardians(n)` / `Ik`. This appendix encodes it as
   `{ method: ("phrase"/"device"/"social"/"ik"), count: uint }` (§18.4.4): `Guardians` → `"social"`
   and `Ik` → `"ik"` (satisfied by an `IK` signature, naming no `RecoveryMethod`). Earlier drafts
   of this appendix omitted `"ik"`; it is added here so every §1.4 predicate is expressible.

6. **`RecoveryMethod` discriminator representation.** §1.4 tags variants by a string field
   `type:"phrase"|"device"|"social"`. For a uniform integer-keyed choice, this appendix encodes the
   discriminator at **key 0** with integer values `1/2/3` (§18.3.3 convention), preserving the
   variant meaning while keeping deterministic small-integer keys throughout. This is a *format*
   normalization, not a semantic change; noted for consistency review.

7. **DMTAP `suite` (u8) vs MLS ciphersuite (u16).** §1.1/§16.7 define `suite` as a one-byte DMTAP
   id (`0x01`/`0x02`); §5.1 also cites MLS ciphersuites `0x0001`/`0x0003`. These are different
   registries in different layers and MUST NOT share a field; §18.1.4 states this explicitly to
   prevent an implementer from conflating them.

8. **Group-state object was prose-only.** §5.8 describes roster/roles/join-policy/posting-model/
   membership-visibility/committer but gives no CBOR. This appendix constructs `GroupState` and
   `RosterEntry` (§18.6.1) faithfully to that prose, including the §5.8.6 hardening: a
   `group_identity` field (key 14) pins the group's own `Identity` object, whose threshold-held
   key and `recovery` policy govern group-authoritative acts (a `rotate_threshold` bar the
   committer cannot meet). Being newly formalized, they SHOULD get a review pass against any
   reference implementation.

9. **Deniable 1:1 mode & endpoint-hardening fields (newly formalized).** The optional deniable
   mode (§5.2.1) adds `DeniablePrekeyBundle` (§18.4.8), the `DeniableFrame` choice
   (`DeniableInit`/`DeniableMessage`, §18.3.9), and `DeniablePayload` (§18.3.10) — the last three
   deliberately carry **no `sig-val`** (authentication is the Double-Ratchet AEAD MAC, §18.9.10),
   which is the wire-visible root of repudiation. Endpoint hardening (§1.2a, §6.7) adds OPTIONAL
   fields to existing signed objects: `Identity.deniable_prekeys` (key 11; `sig` stays key 10),
   `DeviceCert.key_protection`/`attestation` (keys 9/10), and `Headers.sensitive` (key 6). All are
   OPTIONAL and additive; the signing preimages (`object ∖ {sig}`) already cover them, so no
   existing signature computation changes. These SHOULD get a review pass against a reference
   implementation.

10. **Transport-path provenance (newly formalized).** §7.2a already REQUIRED the gateway
    attestation but gave it no CBOR; this appendix formalizes it as `GatewayAttestation`
    (§18.3.11), carried in the new OPTIONAL `Payload.provenance` (key 9) — additive, and covered
    by the existing `Payload.sig` preimage (`Payload ∖ {2}`), so no existing signature computation
    changes. The client-facing `ProvenanceRecord` (§18.8.1) is **node-local and unsigned** by
    design (§18.9.12): it never crosses a trust boundary and MUST NOT carry mix-node identities
    (§6.8), so it is deliberately *not* a mesh wire object. Both SHOULD get a review pass against a
    reference implementation. The gateway attestation reuses the DNS/KT `_dmtap-gw` anchor already
    registered in §21.21 and the existing errors `0x0601`/`0x0602`; no new error code or registry
    is introduced.

11. **Key-transparency objects were prose-only (newly formalized).** §3.5 REQUIRED signed tree
    heads and inclusion/consistency proofs but gave no CBOR — a core-required object left
    undefined. This appendix adds `SignedTreeHead` (§18.4.9, signed by the log's `log_id`,
    DS-tag `DMTAP-v0/kt-sth`), `InclusionProof` (§18.4.10) and `ConsistencyProof` (§18.4.11) as an
    **RFC 6962 profile**, with the Identity-entry **leaf-hash rule** (§18.4.9) pinning what a leaf
    commits to. The two proofs are **unsigned** (verified against an STH root). One new error code
    `0x0117` (`ERR_KT_LEAF_HASH_MISMATCH`) is added; the existing KT codes `0x0107`/`0x0108`/
    `0x0110`–`0x0112` cover the rest. SHOULD get a KAT vector pass.

12. **Delegated capability token & the Sphinx cell were prose/reference-only (newly formalized).**
    §13.5/§13.5.1 described the capability token as "UCAN-style (informative)" with no grammar;
    this appendix adds the normative `CapabilityToken`/`Capability`/`CapabilityRevocation`
    (§18.7.3, a **profile of UCAN v1.0**) with signing preimages (§18.9.14), the chain-attenuation
    invariant, and the KT-logged revocation object, plus `delegate-capability`/`revoke-capability`
    ops (§19.6.6) and error `0x050B` (`ERR_CAPABILITY_REVOKED`; delegation failures stay `0x0508`).
    Separately, the Sphinx packet — previously "specified by reference" — now has its DMTAP-specific
    **byte layout** pinned as `SphinxCell` with the per-hop `RoutingCommand`, `SURB`, and
    `SphinxFragmentHeader` framing (§18.5.4); these are **fixed-length binary**, not CBOR, so they
    are deliberately **not** added to the CDDL grammar. All SHOULD get KAT vectors.

**Object count:** this appendix gives a normative CDDL rule and a per-field semantics table for
**39 wire objects** — `Envelope`, `DeliveryTag`, `ChallengeResponse`, `KeyPackageRef`, `Payload`,
`Headers`, `Body`, `Attachment`, `ManifestRef`, `Manifest`, `DeniableFrame`, `DeniablePayload`,
`GatewayAttestation`, `Identity`, `DeviceCert`, `KeyPackageBundleRef`, `DeniablePrekeyBundle`,
`SignedTreeHead`, `InclusionProof`, `ConsistencyProof`, `RecoveryPolicy`, `RecoveryMethod`,
`Threshold`, `KeyRotation`, `MoveRecord`, `DomainDirectory`, `DirEntry`, `LocationRecord`,
`MixNodeDescriptor`, `MixDirectory`, `SphinxCell`, `GroupState`, `RosterEntry`, `GroupEvent`,
`Challenge`, `Assertion`, `CapabilityToken`, `CapabilityRevocation`, `ProvenanceRecord` — plus
their tagged sub-variants (`KeyTag`/`GroupTag`/`BlindedTag`;
`ArcToken`/`PowSolution`/`PostageStamp`/`Vouch`; `DeniableInit`/`DeniableMessage`;
`PhraseMethod`/`DeviceMethod`/`SocialMethod`; `MethodPredicate`; `MixKeyEntry`; `Capability`) and
the shared scalar prelude (§18.1.7). Counting every distinct CBOR map/group rule in §18.10 — the
**48 map definitions** plus the **four choice-variant families** (`DeliveryTag`,
`ChallengeResponse`, `DeniableFrame`, `RecoveryMethod`) — gives **52 CDDL-defined structures**, all
collected in §18.10. (`Body` is a scalar type alias, not a structure; `SphinxCell` and its
sub-structures `RoutingCommand`/`SURB`/`SphinxFragmentHeader` are **fixed-length byte layouts**
internal to `SphinxCell` (§18.5.4) — **not** CBOR and **not** separately counted; only `SphinxCell`
itself is among the 39 wire objects, and (being byte-layout, not CBOR) it is **not** among the 52
CDDL structures.) (`GatewayAttestation` is the §7.2a/§7.8 transport-path binding,
sealed in `Payload`; `ProvenanceRecord` is the §8.6/§19.9 client-facing assembly — node-local, not
mesh-transmitted, not signed, §18.9.12; `SignedTreeHead`/`InclusionProof`/`ConsistencyProof` are the
§3.5 RFC-6962 KT binding; `CapabilityToken`/`Capability`/`CapabilityRevocation` are the §13.5
delegation binding.) (The mixnet objects `MixNodeDescriptor`/`MixDirectory`/`MixKeyEntry` plus the
byte-layout `SphinxCell` are the §4.4 mixnet binding; the deniable `DeniableFrame`/`DeniableInit`/
`DeniableMessage`/`DeniablePayload`/`DeniablePrekeyBundle` are the §5.2.1 binding; the
Double-Ratchet/X3DH/PQXDH cryptographic cores are specified by reference — Signal's designs in
§5.2.1 — not re-encoded as CDDL objects here.)
