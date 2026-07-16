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

Every top-level signed or encrypted object carries a `suite` field (a `u8`, §16.7). The suite
selects, as a **set**, the signature algorithm, the KEM/PKE, the AEAD, and the content-hash for
that object:

| `suite` | Sign | KEM/PKE | AEAD | Hash | Status |
|--------:|------|---------|------|------|--------|
| `0x01` | Ed25519 | X25519 (HPKE, RFC 9180) | ChaCha20-Poly1305 | BLAKE3-256 | v0 REQUIRED |
| `0x02` | Ed25519 + ML-DSA-65 | X-Wing (X25519 + ML-KEM-768) | ChaCha20-Poly1305 | BLAKE3-256 | RESERVED (PQ) |

A decoder MUST reject an object whose `suite` it does not implement (fail closed, §1.1); it MUST
NOT guess. The `suite` value governs the **length and structure** of every `ik-pub`, `sig-val`,
and encapsulated-key byte string in the same object (§18.2). The DMTAP `suite` (`u8`) is
**distinct** from the MLS ciphersuite (a `u16`, e.g. `0x0001`, negotiated *inside*
`Envelope.ciphertext` per RFC 9420, §5.1); the two never share a field.

### 18.1.5 Hash-agility prefix

Every content-address and every inter-object hash reference (`hash` in the grammar) is a
**multihash-style, single-byte algorithm prefix followed by the raw digest**. The prefix values
are drawn from the multicodec registry, truncated to one byte for v0 (all values below fit in
one byte):

| Prefix byte | Algorithm | Digest length | Status |
|------------:|-----------|--------------:|--------|
| `0x1e` | BLAKE3-256 | 32 B | v0 REQUIRED (default) |
| `0x12` | SHA2-256 | 32 B | RESERVED (compliance migration) |
| `0x16` | SHA3-256 | 32 B | RESERVED |

Thus a v0 `hash` is exactly **33 bytes**: `0x1e ‖ BLAKE3-256(preimage)`. The prefix lets an
implementation migrate the digest algorithm (e.g. to SHA-256 where FIPS compliance requires it,
§2.2) **without changing the address format**. A verifier MUST reject a `hash` whose prefix byte
it does not implement, and MUST reject a digest whose length does not match the prefix's fixed
length.

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

For `suite = 0x02` (hybrid PQ), a `sig-val` is the **concatenation** `Ed25519_sig ‖ ML-DSA-65_sig`
and BOTH component signatures MUST verify; `Identity.sig` additionally carries one `sig-val` per
suite in `suites` (§18.4.1).

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

The PQ (`0x02`) lengths are RESERVED and given for forward planning; a v0 implementation
implements only `0x01` and MUST reject `0x02` fail-closed until it supports it.

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
| `v` | 1 | `u8` | MUST | Format version. MUST equal `0` in v0; a decoder MUST reject any other value (fail closed, §10.1). |
| `suite` | 2 | `suite` | MUST | Algorithm suite actually used for this MOTE (§18.1.4). Governs `sender_sig` length and the crypto inside `ciphertext`. MUST be a suite both parties support (§1.3); unknown ⇒ reject. |
| `id` | 3 | `hash` | MUST | Content address of the exact bytes of field 10 (`ciphertext`), computed per §18.9.4. A verifier MUST recompute it and drop the MOTE on mismatch (§2.7 step 2) *before* any decryption. |
| `to` | 4 | `DeliveryTag` | MUST | Routing target: recipient key, group id, or blinded tag (§18.3.2). If it does not resolve to this node/group, the MOTE is dropped (§2.7 step 4). |
| `epoch` | 5 | `bytes` | OPTIONAL | MLS epoch / group-context reference; present **iff** the MOTE targets an MLS group, so the recipient selects the right epoch key (§5.1). Absent for 1:1/HPKE-sealed MOTEs. Opaque; length set by MLS. |
| `ts` | 6 | `ts` | MUST | Sender wall-clock timestamp, ms since Unix epoch. Subject to clock-skew tolerance ±120 s (§16.1). Used only for ordering/expiry, never for correctness. |
| `kind` | 7 | `u8` | MUST | Message kind (§2.3): `0x00` mail … `0x0a` system; `0x40–0x7f` reserved extensions. A node MUST NOT `ack` a kind it cannot validate (§10.1). |
| `keypkg` | 8 | `KeyPackageRef` | OPTIONAL | Present **iff** this MOTE initiates an MLS session against one of the recipient's published KeyPackages (async join, §5.3). Identifies the consumed KeyPackage (§18.3.4). |
| `challenge` | 9 | `ChallengeResponse` | OPTIONAL | Anti-abuse proof for a **cold** sender (§9), evaluated *without decrypting* (§2.7 step 6). Known contacts omit it (fast path). One of ARC token / PoW / stamp / vouch (§18.3.3). |
| `ciphertext` | 10 | `bytes` | MUST | The MLS `PrivateMessage` or HPKE-sealed `Payload` (§18.3.5). Opaque to every intermediary. Its bytes are the sole input to `id`. |
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
| | `token` | 2 | `bytes` | MUST | The ARC presentation, **per-origin-scoped** (`draft-yun-privacypass-arc`), giving per-recipient rate-limiting *and* cross-recipient unlinkability. Its request context MUST bind the envelope's `sender_key` (§9.2a) so a stripped presentation cannot be replayed under a different ephemeral key; a verifier MUST reject one whose context does not. |
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

### 18.3.6 `Headers` and `Body` (§2.4)

```cddl
Headers = {
  ? 1 => bytes,               ; thread   stable thread/conversation id
  ? 2 => tstr,                ; subject  mail only
  ? 3 => tstr,                ; mime     content type of Body
  4 => [* ik-pub],            ; cc       additional recipient keys (fan-out is per-recipient)
  ? 5 => { * tstr => ext-value }, ; ext   extension headers (§10) — deterministic-safe values only
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
}
```

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `Attachment` | `name` | 1 | `tstr` | MUST | Display name; MUST be treated as untrusted (path-sanitize on save). UTF-8. |
| | `mime` | 2 | `tstr` | MUST | Declared media type. |
| | `size` | 3 | `u64` | MUST | Plaintext size in bytes. |
| | `inline` | 4 | `bytes` | OPTIONAL | The encrypted-then-inlined content, present **iff** the file is ≤ the inline threshold (v0 ≤ 64 KiB after padding, §16.4). Mutually exclusive with `manifest`. |
| | `manifest` | 5 | `ManifestRef` | OPTIONAL | Reference to the file's manifest, present **iff** the file exceeds the inline threshold. Mutually exclusive with `inline`. Exactly one of {`inline`,`manifest`} MUST be present. |
| | `key` | 6 | `enc-key` | MUST | Per-file content key; the recipient decrypts chunks (or `inline`) with it. Travels only inside the (private) MOTE. |
| `ManifestRef` | `id` | 1 | `hash` | MUST | Content address = Merkle-DAG root of the `Manifest` over its ordered chunk hashes (§18.9.5). |
| | `size` | 2 | `u64` | MUST | Total plaintext size, equal to `Manifest.size`. |
| | `chunks` | 3 | `u32` | MUST | **Count** of chunks (⚠ note: in `ManifestRef`, `chunks` is a *number*; in `Manifest`, `chunks` is the *list of hashes* — §18.11). |

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
| `sig` | 10 | `[+ sig-val]` | MUST | **One signature per suite in `suites`**, in the same order, each over the body preimage (§18.9.3). A verifier trusting either the classical or PQ key can validate; it MUST reject an Identity whose highest offered suite it cannot validate (§1.3). |

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
}
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
| `sig` | 8 | `sig-val` | MUST | IK signature over the body (§18.9.3). |

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
  7 => sig-val,         ; sig      by OLD_IK over (old_ik, new_ik, reason, ts)
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
}

dir-visibility = "public" / "members-only"
member-custody = "sovereign" / "org-managed"
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
| | `custody` | 4 | `member-custody` | MUST | `"sovereign"` (member holds their own key; the org cannot access, §3.10.2a) or `"org-managed"` (org holds/escrows the key — a disclosed §6.6-style limit, §3.10.2b). An `"org-managed"` entry MUST be rendered as such; presenting one as sovereign MUST fail closed (`ERR_ORG_MANAGED_UNDISCLOSED`, `0x0115`). |
| | `roles` | 5 | `[* tstr]` | OPTIONAL | Informative org roles / standing-group memberships (§13.5.1, §5.8.7); authority for a role is the capability (§13.5.1), not this hint. |
| | `added` | 6 | `ts` | MUST | When the entry was published. |

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
  7 => sig-val,         ; sig         over the origin-bound preimage incl. cnf (§18.9.8)
  8 => hash,            ; cnf         H(session_pubkey): binds the fresh per-RP session key (§13.3)
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
| `cnf` | 8 | `hash` | MUST | Confirmation key = `H(session_pubkey)` (RFC 7800 style, §13.3 step 4). The client generates the per-RP, per-device session keypair **before** signing and commits it here; the RP MUST bind the session **only** to `cnf` (proof-of-possession, §13.4). Present on every native assertion, and embedded verbatim in a bridged ID Token (§13.6). |

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
| `LocationRecord` | `sig` (k7) | `DMTAP-v0/location-record` | `det_cbor(LocationRecord ∖ {7})` |
| `MixNodeDescriptor` | `sig` (k7) | `DMTAP-v0/mix-descriptor` | `det_cbor(MixNodeDescriptor ∖ {7})` |
| `MixDirectory` | `sig` (k8) | `DMTAP-v0/mix-directory` | `det_cbor(MixDirectory ∖ {8})` |
| `GroupState` | `committer_sig` (k13) | `DMTAP-v0/group-state` | `det_cbor(GroupState ∖ {13})` |
| `GroupEvent` | `committer_sig` (k6) | `DMTAP-v0/group-event` | `det_cbor(GroupEvent ∖ {6})` |
| `PostageStamp` | `sig` (k7) | `DMTAP-v0/postage-stamp` | `det_cbor(PostageStamp ∖ {7})` |
| `Vouch` | `sig` (k5) | `DMTAP-v0/vouch` | `det_cbor(Vouch ∖ {5})` |
| `Assertion` | `sig` (k7) | `DMTAP-v0/auth-assertion` | §18.9.8 |

For `Identity` (`sig` is a **list**, one entry per suite): the **same** preimage
`DS-tag ‖ det_cbor(Identity ∖ {10})` is signed once per suite in `suites`, and the results are
placed in `sig` in `suites` order. Each entry is a `sig-val` per its suite (§18.2).

### 18.9.1 `Envelope.sender_sig`

Per §2.2/§2.7 the ephemeral sender signature is over the concatenation
`(id ‖ to ‖ ts ‖ kind ‖ challenge)`, **not** the whole envelope. Exactly:

```
preimage = "DMTAP-v0/envelope-sender" ‖ 0x00
         ‖ id_bytes                       ; field 3, the full hash byte string (prefix ‖ digest)
         ‖ det_cbor(to)                   ; field 4, deterministic CBOR of the DeliveryTag map
         ‖ u64be(ts)                      ; field 6, 8 bytes big-endian
         ‖ u8(kind)                       ; field 7, 1 byte
         ‖ challenge_enc                  ; field 9: det_cbor(ChallengeResponse) if present,
                                          ;          else the single byte 0xf6 (CBOR null)
sender_sig = Sign(sk_ephemeral, preimage)
```

where `(sk_ephemeral, sender_key)` is a **fresh keypair generated for this one message** and
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
payload_hash = BLAKE3-256( det_cbor(Payload ∖ {2}) )          ; 32 bytes, no prefix
preimage     = "DMTAP-v0/payload" ‖ 0x00 ‖ payload_hash
Payload.sig  = Sign(sk_from, preimage)                         ; sk_from = IK or device key
```

`from` (key 1) is included in the hashed body, binding the signature to the claimed sender. A
device-key signature is valid only if that device key is authorized by a current `DeviceCert`
under `from`'s `Identity` (§1.2). Verified at §2.7 step 8.

### 18.9.3 Identity-family objects

`Identity`, `DeviceCert`, `RecoveryPolicy`, `KeyRotation`, `MoveRecord`, `LocationRecord`,
`DomainDirectory` all use the general rule: `Sign(sk, DS-tag ‖ 0x00 ‖ det_cbor(object ∖ {sig-key}))`
with the DS-tags in the table above. The signing key is: IK for `Identity`/`DeviceCert`/`MoveRecord`;
IK **or** a satisfied `rotate_threshold` quorum for `RecoveryPolicy`; **`old_ik`** for `KeyRotation`;
an authorized **device key** for `LocationRecord`; the **domain authority IK** (threshold-held,
§3.10.1/§5.8.6) for `DomainDirectory`.

### 18.9.4 Object content addresses (`Envelope.id`, `Identity` anchor)

`Envelope.id` is the content address of the **exact bytes** of `Envelope.ciphertext`, with **no**
domain separation (so identical ciphertext dedups regardless of context, §2.8):

```
Envelope.id = 0x1e ‖ BLAKE3-256( ciphertext_bytes )           ; v0 prefix 0x1e = BLAKE3-256
```

The `Identity` anchor everyone pins (the `id` referenced from DNS/KT, §3.2) is the content
address of the fully-signed `Identity` object:

```
Identity_id = 0x1e ‖ BLAKE3-256( det_cbor(Identity) )         ; the complete, signed object
```

Other `hash` references to a whole object (`recovery`, `keypkgs.id`, `prev`, `KeyPackageRef.ref`,
`GroupState.log_head`) are computed the same way — `prefix ‖ BLAKE3-256(det_cbor(referenced
object))` — with no domain separation, since a content address is a pure function of the bytes.

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
(`draft-yun-privacypass-arc`) and a PoW by recomputing the Argon2id digest over
`id ‖ recipient ‖ epoch_nonce` (§16.5).

### 18.9.8 `Assertion.sig` (auth)

Per §13.3 step 5 the signature is over the hash of the origin-bound fields **including `cnf`**:

```
auth_hash    = BLAKE3-256( det_cbor([ rp_origin, nonce, issued_at, exp, aud, cnf ]) )
preimage     = "DMTAP-v0/auth-assertion" ‖ 0x00 ‖ auth_hash
Assertion.sig = Sign(sk_device, preimage)          ; sk_device = the IK-authorized login signer
```

The hashed array is a fixed 6-element CBOR array in exactly that order — the five echoed
`Challenge` fields followed by `cnf` (`Assertion` key 8) — matching §13.3 step 5's
`H(rp_origin ‖ nonce ‖ issued_at ‖ exp ‖ aud ‖ cnf)`. The RP reconstructs it from its own issued
`Challenge` plus the assertion's `cnf` and MUST reject any mismatch of `rp_origin`/`aud`, and MUST
bind the session **only** to `cnf`. The signing key is the user's **`IK`-authorized device key**
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

---

## 18.10 Collected CDDL grammar (copy-paste block)

The following is the complete, self-contained CDDL for every DMTAP wire object. An implementer
MAY copy this single block into a CDDL tool (RFC 8610) as the normative schema. It is internally
consistent and consistent with §1, §2, §5, §13.

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
}

Headers = {
  ? 1 => bytes, ? 2 => tstr, ? 3 => tstr,
  4 => [* ik-pub], ? 5 => { * tstr => ext-value },
}
ext-value = bool / int / bytes / tstr / [* ext-value] / { * tstr => ext-value }
Body = tstr / bytes

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

; ── identity layer (§1) ────────────────────────────────────────────
Identity = {
  1 => [+ u8], 2 => { + u8 => ik-pub }, 3 => u64,
  4 => [* DeviceCert], 5 => KeyPackageBundleRef, 6 => hash,
  7 => [* tstr], ? 8 => hash, 9 => ts, 10 => [+ sig-val],
}

DeviceCert = {
  1 => suite, 2 => ik-pub, 3 => ik-pub, 4 => tstr,
  5 => ts, ? 6 => ts, 7 => [+ tstr], 8 => sig-val,
}

KeyPackageBundleRef = { 1 => tstr, 2 => hash, ? 3 => [+ u8] }

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
DirEntry       = { 1 => tstr, 2 => ik-pub, 3 => hash, 4 => member-custody, ? 5 => [* tstr], 6 => ts }
dir-visibility = "public" / "members-only"
member-custody = "sovereign" / "org-managed"

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

; ── auth layer (§13) ───────────────────────────────────────────────
Challenge = {
  1 => tstr, 2 => bytes, 3 => ts, 4 => ts, 5 => tstr, ? 6 => [* tstr],
}
Assertion = {
  1 => tstr, 2 => bytes, 3 => ts, 4 => ts, 5 => tstr,
  6 => ik-pub, 7 => sig-val, 8 => hash,
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

**Object count:** this appendix gives a normative CDDL rule and a per-field semantics table for
**28 wire objects** — `Envelope`, `DeliveryTag`, `ChallengeResponse`, `KeyPackageRef`, `Payload`,
`Headers`, `Body`, `Attachment`, `ManifestRef`, `Manifest`, `Identity`, `DeviceCert`,
`KeyPackageBundleRef`, `RecoveryPolicy`, `RecoveryMethod`, `Threshold`, `KeyRotation`,
`MoveRecord`, `DomainDirectory`, `DirEntry`, `LocationRecord`, `MixNodeDescriptor`,
`MixDirectory`, `GroupState`, `RosterEntry`, `GroupEvent`, `Challenge`, `Assertion` — plus their
tagged sub-variants (`KeyTag`/`GroupTag`/`BlindedTag`;
`ArcToken`/`PowSolution`/`PostageStamp`/`Vouch`; `PhraseMethod`/`DeviceMethod`/`SocialMethod`;
`MethodPredicate`; `MixKeyEntry`) and the shared scalar prelude (§18.1.7). Counting the four
choice-variant families, `MethodPredicate`, and `MixKeyEntry` as distinct encodable structures
brings the total to **41 CDDL-defined structures**, all collected in §18.10. (The two new mixnet
objects `MixNodeDescriptor`/`MixDirectory` plus the `MixKeyEntry` sub-structure are the §4.4
mixnet binding; the Sphinx packet itself is a non-CBOR fixed-length format specified by reference
in §4.4.1, not a CDDL object.)
