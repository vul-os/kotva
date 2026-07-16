# DMTAP Conformance Test-Case Catalog

This is the **normative test-case catalog** referenced by spec §10.3 — the *operational
definition of compatibility*. An implementation, **in any language**, is DMTAP-conformant at a
level (§10.3: Core / Private / Groups&Files / Legacy / Clients / Auth) **iff** it produces the
expected result for every `MUST` case at that level (and the cases of every level it composes).
"DMTAP-compatible" means "passes this suite," not "resembles the reference" (§10.3, §10.4).

The machine-readable form of this catalog is [`suite.json`](suite.json); the byte-exact inputs it
drives are [`vectors/vectors.json`](vectors/vectors.json) (see [`README.md`](README.md) for the
runner contract and provenance). This `.md` is the human-readable, clause-cross-referenced twin of
`suite.json`: the two carry the **same case ids** and MUST stay in sync.

## How to read a case

Each case has:

| Field | Meaning |
|-------|---------|
| **id** | Stable identifier, `DMTAP-<CATEGORY>-<NN>`. Never reused. |
| **level** | The §10.3 level it belongs to (Core, Private, Groups&Files, Legacy, Clients, Auth). |
| **req** | `MUST` or `SHOULD` (RFC 2119). A level passes only if every `MUST` at that level passes. |
| **clause** | The exact normative spec clause (§-ref) the case pins. |
| **checks** | What behaviour is asserted. |
| **input** | A **vector id** in `vectors.json`, an inline **self-contained construction** (bytes given here), or a **construction** described from other vectors. |
| **expect** | The required outcome: `match` (recompute a KAT and get the committed answer), `accept` (validation passes), or `reject` + the **§21 error code** the reject maps to. |
| **status** | `vectored` (byte-backed by `vectors.json`), `self-contained` (bytes given inline, reference-independent), or `construction-todo` (recipe given; byte-exact vector still to be generated). |

**Outcome vocabulary.** `match` = the operation is a deterministic known-answer test; recompute
it over the fixed input and the result MUST equal the committed `expected`. `accept` / `reject` =
the operation is a validation predicate; a conforming node MUST accept (resp. reject) the input,
and on `reject` MUST map it to the named §21 error code with that code's `Action` disposition
(DROP_SILENT / DEFER_REQUESTS / FAIL_CLOSED_BLOCK / HALT_ALERT / ACK_DEDUP …).

## Coverage summary

| Level | Cases | Vectored | Self-contained | Construction-todo |
|-------|------:|---------:|---------------:|------------------:|
| **Core** — canonical CBOR (`CBOR`) | 12 | 4 | 6 | 2 |
| **Core** — content address (`ADDR`) | 6 | 6 | 0 | 0 |
| **Core** — Ed25519 (`SIG`) | 6 | 6 | 0 | 0 |
| **Core** — signing preimages (`PRE`) | 3 | 3 | 0 | 0 |
| **Core** — key-name (`NAME`) | 6 | 6 | 0 | 0 |
| **Core** — safety number (`SAFE`) | 2 | 2 | 0 | 0 |
| **Core** — suite fail-closed (`SUITE`) | 6 | 6 | 0 | 0 |
| **Core** — §2.7 validation pipeline (`VAL`) | 15 | 0 (2 reuse ADDR/PRE) | 0 | 15 |
| **Core** — identity / KT / naming (`IDENT`) | 6 | 0 | 0 | 6 |
| **Private** (`PRIV`) | 7 | 0 | 0 | 7 |
| **Groups & Files** (`GRP`, `FILE`) | 7 | 0 | 0 | 7 |
| **Legacy** (`LEG`) | 2 | 0 | 0 | 2 |
| **Clients** (`CLI`) | 1 | 0 | 0 | 1 |
| **Auth** (`AUTH`) | 5 | 0 | 0 | 5 |
| **Total** | **84** | **33** | **6** | **45** |

The 33 vectored + 6 self-contained cases (**39**) are fully machine-runnable **today** from
`vectors.json` + the inline bytes here, with **no reference implementation required**. They pin the
entire deterministic, security-critical Core spine — canonical CBOR, content addressing, the two
MOTE signature preimages (§18.9.1/§18.9.2), Ed25519 (with RFC 8032 cross-checks), the 8-word
key-name, safety numbers, and suite fail-closed. The 45 `construction-todo` cases give the exact
recipe and expected §21 error for every remaining normative branch (the full §2.7 pipeline,
identity/KT fail-closed, and the higher levels); each becomes byte-backed when the corresponding
subsystem gains a fixed-input KAT in `vectors.json` (see README "Coverage vs. deferred").

> All 39 byte-backed cases correspond one-for-one to entries in `vectors.json`
> (**32 vectors**, several driving more than one case). No case references a `vectors.json`
> entry that does not exist; see [Vector cross-reference](#vector-cross-reference).

---

## Core level

Core is the interoperability floor (§10.3): Identity (§1), MOTE (§2), naming + TOFU + fail-closed
KT (§3), delivery + `deliver`/`ack` (§4), MLS 1:1 (§5), recipient policy incl. cold-sender
challenge gating (§9). A production mail node MUST also implement **Private** (§10.3). Every Core
crypto/encoding case below is a prerequisite the higher levels inherit.

### CBOR — deterministic canonical encoding (§18.1.1, §18.1.2)

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-CBOR-01 | MUST | §18.1.1, §18.4.1 | `decode(encode(Identity))` is byte-identical; integer-keyed map, keys ascending | vector `cbor_identity` | match (`cbor_hex`) | vectored |
| DMTAP-CBOR-02 | MUST | §18.1.1, §18.4.2 | `DeviceCert` deterministic re-encode is byte-identical | vector `cbor_device_cert` | match | vectored |
| DMTAP-CBOR-03 | MUST | §18.1.1, §18.3.5 | `Payload` deterministic re-encode is byte-identical | vector `cbor_payload` | match | vectored |
| DMTAP-CBOR-04 | MUST | §18.1.1, §18.3.1 | `Envelope` deterministic re-encode is byte-identical | vector `cbor_envelope` | match | vectored |
| DMTAP-CBOR-05 | MUST | §18.1.1(1) | reject non-shortest (non-preferred) integer encoding | inline `0x1817` (uint 23 in 2 bytes; MUST be `0x17`) | reject → `ERR_MALFORMED_OBJECT` (0x020D) | self-contained |
| DMTAP-CBOR-06 | MUST | §18.1.1(1) | reject indefinite-length item | inline `0x9fff` (indefinite array + break) | reject → `ERR_MALFORMED_OBJECT` (0x020D) | self-contained |
| DMTAP-CBOR-07 | MUST | §18.1.1(2) | reject map whose keys are not bytewise-ascending | inline `0xa2020001 00`(`{2:0,1:0}` descending) | reject → `ERR_MALFORMED_OBJECT` (0x020D) | self-contained |
| DMTAP-CBOR-08 | MUST | §18.1.1(3) | reject map with a duplicate key | inline `0xa201010102` (`{1:1, 1:2}`) | reject → `ERR_MALFORMED_OBJECT` (0x020D) | self-contained |
| DMTAP-CBOR-09 | MUST | §18.1.1(4) | reject any floating-point value | inline `0xf93e00` (half-float 1.5) | reject → `ERR_MALFORMED_OBJECT` (0x020D) | self-contained |
| DMTAP-CBOR-10 | MUST | §18.1.1(5) | reject CBOR `undefined` (and NaN/Inf/undefined-tag) | inline `0xf7` (simple(23) = undefined) | reject → `ERR_MALFORMED_OBJECT` (0x020D) | self-contained |
| DMTAP-CBOR-11 | MUST | §18.1.1 (last ¶) | reject a wire map carrying an optional key whose **value is `null`** (absent ⇒ omitted, never `null`) | construction: take `cbor_envelope`, insert key `5` (epoch) `=> 0xf6`, re-sort keys | reject → `ERR_MALFORMED_OBJECT` (0x020D) | construction-todo |
| DMTAP-CBOR-12 | MUST | §18.1.2 | a decoder of a **signed** object rejects any unknown integer key `≥ 64` (fail closed, so the signing preimage stays unambiguous) | construction: take `cbor_payload`, insert key `64` (`0x1840`) `=> 0`, re-sort keys | reject → `ERR_MALFORMED_OBJECT` (0x020D) | construction-todo |

> Note: an **unsigned** object MAY ignore unknown keys `≥ 64` (§18.1.2); DMTAP-CBOR-12 asserts the
> *signed*-object fail-closed rule specifically. `Headers.ext` text-keyed extension is the only
> text-key surface (§18.3.6) and is out of scope for the integer-key rules above.

### ADDR — content addressing (§2.2, §18.1.5, §18.9.4)

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-ADDR-01 | MUST | §18.1.5, §18.9.4 | `id = 0x1e ‖ BLAKE3-256(bytes)` over empty input | vector `content_address_empty` | match (`id_hex`) | vectored |
| DMTAP-ADDR-02 | MUST | §18.1.5, §18.9.4 | content address of a 5-byte input | vector `content_address_small` | match | vectored |
| DMTAP-ADDR-03 | MUST | §18.1.5, §18.9.4 | content address of a human phrase | vector `content_address_phrase` | match | vectored |
| DMTAP-ADDR-04 | MUST | §18.1.5, §18.9.4 | content address of a 4096-byte input | vector `content_address_multi_kib` | match | vectored |
| DMTAP-ADDR-05 | MUST | §2.7 step 2 | `Envelope.id` recomputes from `ciphertext` (checked **before** decryption) | vector `mote_content_address_ok` | accept | vectored |
| DMTAP-ADDR-06 | MUST | §2.7 step 2 | a tampered ciphertext fails the address check and is dropped before decrypt | vector `mote_content_address_tampered` | reject → `ERR_BAD_CONTENT_ADDRESS` (0x0202), DROP_SILENT | vectored |

### SIG — Ed25519 sign/verify (§18.1.6)

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-SIG-01 | MUST | §18.1.6 | deterministic Ed25519 (RFC 8032 §7.1 Test 1 cross-check) | vector `ed25519_rfc8032_test1` | match (`pubkey_hex`,`sig_hex`) | vectored |
| DMTAP-SIG-02 | MUST | §18.1.6 | deterministic Ed25519 (RFC 8032 §7.1 Test 2 cross-check) | vector `ed25519_rfc8032_test2` | match | vectored |
| DMTAP-SIG-03 | MUST | §18.1.6 | Ed25519 over the domain-separated preimage `DS ‖ msg` | vector `ed25519_domain_separated` | match | vectored |
| DMTAP-SIG-04 | MUST | §18.1.6 | a valid signature verifies | vector `ed25519_verify_ok` | accept | vectored |
| DMTAP-SIG-05 | MUST | §18.1.6 | a one-bit message tamper fails verification (fail closed) | vector `ed25519_verify_tampered_msg` | reject | vectored |
| DMTAP-SIG-06 | MUST | §18.1.6 | a one-bit signature tamper fails verification (fail closed) | vector `ed25519_verify_tampered_sig` | reject | vectored |

### PRE — MOTE signing preimages (§18.9.1, §18.9.2; §2.7 steps 3 & 8)

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-PRE-01 | MUST | §18.9.1 | `sender_sig` over `DS ‖ id ‖ det_cbor(to) ‖ u64be(ts) ‖ u8(kind) ‖ challenge_enc` (challenge absent ⇒ `0xf6`), under the ephemeral `sender_key` | vector `mote_sender_sig` | match (`sig_hex`,`pubkey_hex`) | vectored |
| DMTAP-PRE-02 | MUST | §18.9.1, §2.7 step 3 | that `sender_sig` verifies under `sender_key` before any decryption | vector `mote_sender_sig_verify` | accept | vectored |
| DMTAP-PRE-03 | MUST | §18.9.2, §2.7 step 8 | `Payload.sig` over `DS ‖ BLAKE3-256(det_cbor(Payload ∖ {sig}))` under the IK | vector `mote_payload_sig` | match | vectored |

### NAME — zero-authority 8-word key-name (§3.9.1, §16.2)

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-NAME-01 | MUST | §3.9.1, §16.2 | key-name is deterministic (+ checksum verifies) — all-zero key | vector `keyname_zero_key` | match (`name`), accept (checksum) | vectored |
| DMTAP-NAME-02 | MUST | §3.9.1 | key-name of all-`0x01` key | vector `keyname_key_ones` | match | vectored |
| DMTAP-NAME-03 | MUST | §3.9.1 | key-name of all-`0x02` key | vector `keyname_key_twos` | match | vectored |
| DMTAP-NAME-04 | MUST | §3.9.1 | key-name of a real Ed25519 public key | vector `keyname_real_pubkey` | match | vectored |
| DMTAP-NAME-05 | MUST | §3.9.1 | distinct keys ⇒ distinct names (`keyname_key_ones` ≠ `keyname_key_twos`) | derived from NAME-02 / NAME-03 | accept (names differ) | vectored |
| DMTAP-NAME-06 | MUST | §3.9.1, §16.2 | a single mistyped word fails the folded checksum (fail closed) | vector `keyname_typo_rejected` | reject (checksum) | vectored |

### SAFE — out-of-band safety number (§3.4.1)

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-SAFE-01 | MUST | §3.4.1 | deterministic pair fingerprint / safety number of two identity keys | vector `safety_number_pair_ab` | match (`fingerprint_hex`,`safety_number`) | vectored |
| DMTAP-SAFE-02 | MUST | §3.4.1 | the safety number is **order-independent** (swapping the two keys yields the identical value) | vector `safety_number_order_independent` | match; MUST equal SAFE-01 | vectored |

### SUITE — algorithm-suite fail-closed (§1.1, §18.1.4)

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-SUITE-01 | MUST | §1.1, §18.1.4 | unknown suite `0x00` MUST be rejected on decode (never guess) | vector `suite_reject_0x00` | reject → `ERR_UNKNOWN_SUITE` (0x0101) / `ERR_UNKNOWN_VERSION_OR_SUITE` (0x0201) | vectored |
| DMTAP-SUITE-02 | MUST | §1.1, §18.1.4 | unknown suite `0x03` rejected | vector `suite_reject_0x03` | reject → 0x0101 / 0x0201 | vectored |
| DMTAP-SUITE-03 | MUST | §1.1, §18.1.4 | unknown suite `0x05` rejected | vector `suite_reject_0x05` | reject → 0x0101 / 0x0201 | vectored |
| DMTAP-SUITE-04 | MUST | §1.1, §18.1.4 | unknown suite `0xff` rejected | vector `suite_reject_0xff` | reject → 0x0101 / 0x0201 | vectored |
| DMTAP-SUITE-05 | MUST | §1.1, §18.1.4 | known suite id `0x01` (classical) decodes | vector `suite_accept_0x01` | accept | vectored |
| DMTAP-SUITE-06 | MUST | §1.1, §18.1.4, §18.2 | suite id `0x02` (reserved PQ) is a **known id** and decodes; note an object whose crypto actually **uses** `0x02` MUST still fail closed until the PQ suite is implemented (§18.2) | vector `suite_accept_0x02` | accept (id) | vectored |

### VAL — the §2.7 ordered recipient-validation pipeline

The full ordered pipeline (§2.7), each step mapped to its authoritative §21.4 disposition. Steps
2, 3 and 8's positive path are already byte-backed by ADDR-05/06, PRE-02 and PRE-03; the remaining
branches give the exact reject code and disposition and are `construction-todo` (they need a full
sealed `Envelope`/`Payload` fixture, which is non-deterministic to *seal* — see README "deferred").

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-VAL-01 | MUST | §2.7 step 1, §10.1 | unknown `v`/`suite` rejected **first**, before any crypto | construction: `Envelope` with `v=1` (or unknown `suite`) | reject → `ERR_UNKNOWN_VERSION_OR_SUITE` (0x0201), DROP_SILENT | construction-todo |
| DMTAP-VAL-02 | MUST | §2.7 step 2 | `id` mismatch dropped before decryption | (byte-backed by DMTAP-ADDR-06) | reject → `ERR_BAD_CONTENT_ADDRESS` (0x0202), DROP_SILENT | construction-todo |
| DMTAP-VAL-03 | MUST | §2.7 step 3 | `sender_sig` failure dropped (cheap, pre-decryption) | construction: `mote_sender_sig` fixture with one sig bit flipped | reject → `ERR_BAD_SENDER_SIG` (0x0203), DROP_SILENT | construction-todo |
| DMTAP-VAL-04 | MUST | §2.7 step 4 | `to` that does not resolve to this node/group is dropped | construction: `Envelope.to = KeyTag(other key)` | reject → `ERR_UNRESOLVED_TO` (0x0204), DROP_SILENT | construction-todo |
| DMTAP-VAL-05 | MUST | §2.7 step 6, §2.7a, §9 | a cold sender's **forged/invalid** `challenge` is discarded silently, **not** acked | construction: `Envelope` from cold sender, tampered `ChallengeResponse` | reject → `ERR_CHALLENGE_INVALID_FORGED` (0x0205 → §21.10 0x0703), DROP_SILENT, no ack | construction-todo |
| DMTAP-VAL-06 | MUST | §2.7 step 6, §2.7a | a cold sender with **absent/under-threshold** proof is **deferred to the requests area**, rate-limited, and **NOT acked** (deferred = no ack) | construction: cold-sender `Envelope`, `challenge` absent | reject → `ERR_CHALLENGE_ABSENT_INSUFFICIENT` (0x0206 → §21.10 0x0701/0x0702), DEFER_REQUESTS, no ack | construction-todo |
| DMTAP-VAL-07 | MUST | §2.7 step 7 | ciphertext that fails to decrypt is dropped | construction: `Envelope` with corrupt `ciphertext` (address recomputed to keep step 2 valid) | reject → `ERR_DECRYPT_FAILURE` (0x0207), DROP_SILENT | construction-todo |
| DMTAP-VAL-08 | MUST | §2.7 step 8 | `Payload.sig` failure ⇒ **discard silently and do NOT ack** (fail closed) | construction: sealed `Payload` with tampered `sig` | reject → `ERR_PAYLOAD_SIG_INVALID` (0x0208), DROP_SILENT, no ack | construction-todo |
| DMTAP-VAL-09 | MUST | §2.7 step 8, §3.4 | decrypted `from` that mismatches the pinned identity ⇒ security warning, MUST NOT silently repin | construction: known-contact `Envelope`, `Payload.from` ≠ pinned | reject → `ERR_FROM_PIN_MISMATCH` (0x0209), HALT_ALERT | construction-todo |
| DMTAP-VAL-10 | MUST | §2.7 step 8, §1.3 | **suite-ratchet**: `Envelope.suite` **below** the contact's pinned high-water-mark is a downgrade ⇒ requests area + warning; MUST NOT accept, MUST NOT ratchet the high-water-mark down | construction: pinned contact HWM=`0x02`, inbound `Envelope.suite=0x01` | reject → `ERR_SUITE_DOWNGRADE` (0x020F), DEFER_REQUESTS + USER_WARN | construction-todo |
| DMTAP-VAL-11 | MUST | §2.6, §21.4 | a duplicate `id` already held is **acked** immediately without re-processing | construction: re-deliver an already-stored `id` | accept → `STATUS_DUPLICATE_ID` (0x020E), ACK_DEDUP | construction-todo |
| DMTAP-VAL-12 | MUST | §2.7a, §19.3.1 | ack is owed **only** for inbox delivery (step 9); a deferred cold MOTE is durably held for the requests-area retention (30 d, §16.5) but **no ack is sent** | construction: cold MOTE deferred at step 6 | accept (held), **no ack emitted** | construction-todo |
| DMTAP-VAL-13 | MUST | §2.3, §10.1 | a `kind` the node cannot validate is ignored and **MUST NOT be acked** | construction: `Envelope.kind = 0x40` (reserved) unimplemented | reject → `ERR_KIND_UNKNOWN` (0x020A), IGNORE_NO_ACK | construction-todo |
| DMTAP-VAL-14 | SHOULD | §16.1 | `ts` outside ±120 s skew is dropped for cold senders (MAY be lenient for known contacts) | construction: `Envelope.ts` = now + 10 min | reject → `ERR_TIMESTAMP_OUT_OF_SKEW` (0x020C), DROP_SILENT (cold) | construction-todo |
| DMTAP-VAL-15 | SHOULD | §2.4, §16.1 | an expired MOTE (`Payload.expires` past) is dropped (cooperative hint, not a security guarantee) | construction: `Payload.expires` in the past | reject → `ERR_EXPIRED_MOTE` (0x020B), DROP_SILENT | construction-todo |

### IDENT — identity / KT / naming fail-closed (§1.3, §3.3, §3.5)

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-IDENT-01 | MUST | §1.3 | an `Identity` whose `sig` (any suite entry) fails is rejected | construction: `cbor_identity` with tampered `sig` | reject → `ERR_IDENTITY_SIG_INVALID` (0x0103), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-IDENT-02 | MUST | §1.3 (anti-rollback) | an `Identity.version` `≤` the pinned version is rejected (rollback/replay of a superseded object) | construction: pin `version=n`, present validly-signed `version=n-1` | reject → `ERR_STALE_ROLLBACK` (0x0105), FAIL_CLOSED_BLOCK (HALT_ALERT if own identity) | construction-todo |
| DMTAP-IDENT-03 | MUST | §1.3 | a broken `prev` hash chain is rejected | construction: `Identity.prev` ≠ hash of pinned prior | reject → `ERR_IDENTITY_CHAIN_BROKEN` (0x0104), HALT_ALERT | construction-todo |
| DMTAP-IDENT-04 | MUST | §3.3, §3.5.1 | **KT fail-closed**: if the transparency log is unreachable at first-contact pinning, the node MUST NOT silently TOFU-pin — block or hard-warn | construction: first contact with KT endpoint unreachable | reject → `ERR_KT_UNREACHABLE` (0x0106), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-IDENT-05 | MUST | §1.2 | a `DeviceCert` with an invalid signature, or `caps` exceeding what the IK authorized, is rejected | construction: `cbor_device_cert` with tampered `sig` | reject → `ERR_DEVICE_CERT_INVALID` (0x010D), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-IDENT-06 | MUST | §1.3 | empty suite intersection ⇒ delivery fails closed, **no silent downgrade** | construction: sender suites ∩ recipient `Identity.suites` = ∅ | reject → `ERR_SUITE_INTERSECTION_EMPTY` (0x0102), REJECT_NOTIFY | construction-todo |

---

## Private level (§4.4, §6)

Core + mixnet (Sphinx + directory + 3-hop stratified paths + key-epoch rotation) + sealed sender +
cover traffic + anti-active-adversary mechanisms + fail-closed no-downgrade + privacy tiers. A
**production** mail node MUST implement Private (§10.3): `private` is the standing default tier
(§4.6). No byte-exact vectors exist yet (Sphinx uses fresh randomness; see README "deferred");
each case gives the normative check and error.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-PRIV-01 | MUST | §4.4.1, §16.3 | Sphinx packet is constant-length on the bucket ladder {2,8,32,64} KiB; wrong length rejected | reject → `ERR_MIX_PACKET_MALFORMED` (0x0307) | construction-todo |
| DMTAP-PRIV-02 | MUST | §4.4.2, §18.5.3 | `MixDirectory` verifies under the pinned authority (`> n/2` quorum) | reject → `ERR_MIX_DIRECTORY_SIG_INVALID` (0x030B), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-PRIV-03 | MUST | §4.4.6 | per-epoch replay cache drops a replayed mix packet | reject → `ERR_MIX_REPLAY_DETECTED` (0x030E) | construction-todo |
| DMTAP-PRIV-04 | MUST | §4.4.9 | fail-closed **no-downgrade**: a forced drop from `private` to `fast` is refused, never silent | reject → `ERR_PRIVATE_TIER_DOWNGRADE_REFUSED` (0x0310) | construction-todo |
| DMTAP-PRIV-05 | MUST | §4.4.8 | loop-cover / active-attack suspicion is surfaced, not ignored | reject → `ERR_MIX_ACTIVE_ATTACK_SUSPECTED` (0x030F) | construction-todo |
| DMTAP-PRIV-06 | MUST | §4.4.2 | a stale `MixNodeDescriptor` is not used | reject → `ERR_MIX_DESCRIPTOR_STALE` (0x030C) | construction-todo |
| DMTAP-PRIV-07 | SHOULD | §4.4.10, §4.4.12 | high-security profile and PQ-Sphinx are OPTIONAL (negotiated, not required at Private) | accept (optional) | construction-todo |

---

## Groups & Files level (§5)

Core + MLS groups + content-addressed file transfer. MLS handshake bytes (RFC 9420) are carried
opaquely by DMTAP and are not vectored (README "deferred"); the DMTAP-native Merkle-DAG manifest
(§18.9.5) is deterministic and SHOULD be vectored next.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-GRP-01 | MUST | §5.1, §18.9.6 | `GroupEvent.committer_sig` / `GroupState.committer_sig` verify under the committer's IK-authorized device key | reject → `ERR_PAYLOAD_SIG_INVALID`-class / group check | construction-todo |
| DMTAP-GRP-02 | MUST | §5.1 | two Commits at the same log position with the same predecessor ⇒ committer fork | reject → `ERR_COMMITTER_FORK_DETECTED` (0x0404), HALT_ALERT | construction-todo |
| DMTAP-GRP-03 | MUST | §5 | wrong MLS epoch key selection is rejected | reject → `ERR_EPOCH_MISMATCH` (0x0406) | construction-todo |
| DMTAP-FILE-01 | MUST | §18.9.5, §5.5 | `Manifest.id` = RFC 6962 Merkle root over ordered chunk hashes with domain-separated leaf/node prefixes | match (root hash) | construction-todo |
| DMTAP-FILE-02 | MUST | §5.5 | a fetched chunk whose hash ≠ its `Manifest.chunks` entry is rejected | reject → `ERR_CHUNK_HASH_MISMATCH` (0x0802) | construction-todo |
| DMTAP-FILE-03 | MUST | §2.5 | a file routed on the wrong size-tier path is rejected | reject → `ERR_SIZE_TIER_MISMATCH` (0x0804) | construction-todo |
| DMTAP-FILE-04 | MUST | §5.5 | a `Manifest` MUST NOT carry the file key (key rides the sealed MOTE, not the manifest) | reject → `ERR_MANIFEST_KEY_PRESENT` (0x0808) | construction-todo |

---

## Legacy level (§7)

Core + gateway inbound/outbound + DKIM delegation.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-LEG-01 | MUST | §7 | a gateway attestation that fails to verify under a trusted key is rejected | reject → `ERR_GATEWAY_ATTESTATION_INVALID` (0x0601) | construction-todo |
| DMTAP-LEG-02 | MUST | §7 | invalid DKIM delegation is rejected | reject → `ERR_DKIM_DELEGATION_INVALID` (0x0603) | construction-todo |

---

## Clients level (§8)

Core + JMAP; IMAP/POP/SMTP-submission compat RECOMMENDED.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-CLI-01 | MUST | §8 | a MOTE renders to/from the JMAP object model without loss of the fields §8 requires | accept (round-trip) | construction-todo |

---

## Auth level (§13)

Core + DMTAP-Auth login ceremony with origin binding + key-bound sessions; OIDC bridge RECOMMENDED.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-AUTH-01 | MUST | §18.9.8, §13.3 | `Assertion.sig` over `DS ‖ BLAKE3-256(det_cbor([rp_origin,nonce,issued_at,exp,aud,cnf]))` under the IK-authorized device key | match (sig) | construction-todo |
| DMTAP-AUTH-02 | MUST | §13.3 | an assertion whose `rp_origin`/`aud` mismatch the issued `Challenge` is rejected | reject → `ERR_ORIGIN_MISMATCH` (0x0501) | construction-todo |
| DMTAP-AUTH-03 | MUST | §13.3 | a replayed `nonce` is rejected | reject → `ERR_NONCE_REPLAYED` (0x0502) | construction-todo |
| DMTAP-AUTH-04 | MUST | §13.3 | an expired `Challenge` is rejected | reject → `ERR_CHALLENGE_EXPIRED` (0x0503) | construction-todo |
| DMTAP-AUTH-05 | MUST | §18.9.8, §13.3 | the session is bound **only** to `cnf` (not the signing key), and MUST reject on `cnf` mismatch | accept (bound to `cnf`) | construction-todo |

---

## Vector cross-reference

Every `vectored` case above maps to an existing entry in `vectors/vectors.json`
(**32 vectors**). Cross-check (case → vector):

| vectors.json entry | driven by case(s) |
|--------------------|-------------------|
| `content_address_empty` / `_small` / `_phrase` / `_multi_kib` | ADDR-01 / -02 / -03 / -04 |
| `mote_content_address_ok` / `_tampered` | ADDR-05 / ADDR-06 (= VAL-02) |
| `ed25519_rfc8032_test1` / `_test2` | SIG-01 / SIG-02 |
| `ed25519_domain_separated` | SIG-03 |
| `ed25519_verify_ok` / `_tampered_msg` / `_tampered_sig` | SIG-04 / SIG-05 / SIG-06 |
| `mote_sender_sig` / `mote_sender_sig_verify` | PRE-01 / PRE-02 (= VAL-03 base) |
| `mote_payload_sig` | PRE-03 (= VAL-08 base) |
| `keyname_zero_key` / `_key_ones` / `_key_twos` / `_real_pubkey` | NAME-01 / -02 / -03 / -04 (-05 derives from -02/-03) |
| `keyname_typo_rejected` | NAME-06 |
| `safety_number_pair_ab` / `_order_independent` | SAFE-01 / SAFE-02 |
| `suite_reject_0x00` / `_0x03` / `_0x05` / `_0xff` | SUITE-01 / -02 / -03 / -04 |
| `suite_accept_0x01` / `_0x02` | SUITE-05 / SUITE-06 |
| `cbor_identity` / `cbor_device_cert` / `cbor_payload` / `cbor_envelope` | CBOR-01 / -02 / -03 / -04 |

No case references a vector that is absent from `vectors.json`. The six `self-contained` CBOR
reject cases (CBOR-05…CBOR-10) carry their bytes inline here and in `suite.json` and need no
reference implementation. The `construction-todo` cases carry a byte-exact recipe and the expected
§21 error; they are wired byte-for-byte as their subsystems gain fixed-input KATs (README
"Coverage vs. deferred").
