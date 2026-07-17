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
| **Core** — suite fail-closed (`SUITE`) | 7 | 7 | 0 | 0 |
| **Core** — §2.7 validation pipeline (`VAL`) | 15 | 0 (2 reuse ADDR/PRE) | 0 | 15 |
| **Core** — identity / KT / naming (`IDENT`) | 6 | 0 | 0 | 6 |
| **Core** — aliases (`ALIAS`) | 3 | 0 | 0 | 3 |
| **Core** — resolver framework (`RESOLVE`) | 3 | 0 | 0 | 3 |
| **Private** (`PRIV`) | 7 | 0 | 0 | 7 |
| **Groups & Files** (`GRP`, `FILE`) | 12 | 0 | 0 | 12 |
| **Groups & Files** — device-cluster sync (`SYNC`) | 5 | 0 | 0 | 5 |
| **Legacy** (`LEG`) | 3 | 0 | 0 | 3 |
| **Legacy** — gateway alias mapping (`GWALIAS`) | 3 | 0 | 0 | 3 |
| **Clients** (`CLI`) | 1 | 0 | 0 | 1 |
| **Auth** (`AUTH`) | 5 | 0 | 0 | 5 |
| **Private** — deniable 1:1 mode (`DENIABLE`) | 5 | 0 | 0 | 5 |
| **Core** — org administration (`ORG`) | 5 | 0 | 0 | 5 |
| **Private** — KT-v1 hardening (`KTV1`) | 4 | 0 | 0 | 4 |
| **Core** — device attestation (`ATTEST`) | 2 | 0 | 0 | 2 |
| **Core** — profile / avatar (`PROFILE`) | 2 | 0 | 0 | 2 |
| **Optional** — push wake-signaling (`PUSH`) | 2 | 0 | 0 | 2 |
| **Total** | **125** | **34** | **6** | **85** |

The 34 vectored + 6 self-contained cases (**40**) are fully machine-runnable **today** from
`vectors.json` + the inline bytes here, with **no reference implementation required**. They pin the
entire deterministic, security-critical Core spine — canonical CBOR, content addressing, the two
MOTE signature preimages (§18.9.1/§18.9.2), Ed25519 (with RFC 8032 cross-checks), the 8-word
key-name, safety numbers, and suite fail-closed. The 85 `construction-todo` cases give the exact
recipe and expected §21 error for every remaining normative branch (the full §2.7 pipeline,
identity/KT fail-closed, the higher levels, the wave-2 hardening families —
`DENIABLE`/`ORG`/`KTV1`/`ATTEST` — the `PROFILE` display-data guards, the pluggable-resolver guards
(`RESOLVE`), the optional `PUSH`
wake-signaling guards, and the `FILE` durability guards `DMTAP-FILE-05`–`-09`); each becomes byte-backed
when the corresponding subsystem gains a fixed-input KAT in `vectors.json` (see README "Coverage vs.
deferred"). **Sync status:** `SUITE.md` and [`suite.json`](suite.json) are **in sync** — both carry
the same **125** case ids (the wave-2 `DENIABLE`/`KTV1` families, the `PROFILE` cases, the
optional `PUSH` cases, the `FILE` durability cases, and the wave-3 `SYNC` (device-cluster),
`ALIAS`, `GWALIAS`, and `RESOLVE` families are mirrored into `suite.json`). The changed deniable objects (§5.2.1 dedicated-`idk`) are still to be
re-vectored when the reference regenerates `vectors.json`.

> All 40 byte-backed cases correspond one-for-one to entries in `vectors.json`
> (they drive **33 of the 68 vectors** in the file — several vectors drive more than one case;
> the remaining 35 are pre-generated for construction-todo families not yet wired to a case).
> No case references a `vectors.json` entry that does not exist; see
> [Vector cross-reference](#vector-cross-reference).

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
| DMTAP-NAME-01 | MUST | §3.9.6, §16.2 | key-name is deterministic (+ checksum verifies) — all-zero key | vector `keyname_zero_key` | match (`name`), accept (checksum) | vectored |
| DMTAP-NAME-02 | MUST | §3.9.6 | key-name of all-`0x01` key | vector `keyname_key_ones` | match | vectored |
| DMTAP-NAME-03 | MUST | §3.9.6 | key-name of all-`0x02` key | vector `keyname_key_twos` | match | vectored |
| DMTAP-NAME-04 | MUST | §3.9.6 | key-name of a real Ed25519 public key | vector `keyname_real_pubkey` | match | vectored |
| DMTAP-NAME-05 | MUST | §3.9.6 | distinct keys ⇒ distinct names (`keyname_key_ones` ≠ `keyname_key_twos`) | derived from NAME-02 / NAME-03 | accept (names differ) | vectored |
| DMTAP-NAME-06 | MUST | §3.9.6, §16.2 | a single mistyped word fails the folded checksum (fail closed) | vector `keyname_typo_rejected` | reject (checksum) | vectored |

### SAFE — out-of-band safety number (§3.4.1)

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-SAFE-01 | MUST | §3.4.1 | deterministic pair fingerprint / safety number of two identity keys | vector `safety_number_pair_ab` | match (`fingerprint_hex`,`safety_number`) | vectored |
| DMTAP-SAFE-02 | MUST | §3.4.1 | the safety number is **order-independent** (swapping the two keys yields the identical value) | vector `safety_number_order_independent` | match; MUST equal SAFE-01 | vectored |

### SUITE — algorithm-suite fail-closed (§1.1, §18.1.4)

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-SUITE-01 | MUST | §1.1, §18.1.4 | unknown suite `0x00` MUST be rejected on decode (never guess) | vector `suite_reject_0x00` | reject → `ERR_UNKNOWN_SUITE` (0x0101) / `ERR_UNKNOWN_VERSION_OR_SUITE` (0x0201) | vectored |
| DMTAP-SUITE-02 | MUST | §1.1, §18.1.4, §18.2 | suite id `0x03` (reserved AEAD-diverse) is a **known reserved id** and decodes; note an object whose crypto actually **uses** `0x03` MUST still fail closed until the suite is implemented | vector `suite_accept_0x03` | accept (id) | vectored |
| DMTAP-SUITE-03 | MUST | §1.1, §18.1.4 | unknown suite `0x05` rejected | vector `suite_reject_0x05` | reject → 0x0101 / 0x0201 | vectored |
| DMTAP-SUITE-04 | MUST | §1.1, §18.1.4 | unknown suite `0xff` rejected | vector `suite_reject_0xff` | reject → 0x0101 / 0x0201 | vectored |
| DMTAP-SUITE-05 | MUST | §1.1, §18.1.4 | known suite id `0x01` (classical) decodes | vector `suite_accept_0x01` | accept | vectored |
| DMTAP-SUITE-06 | MUST | §1.1, §18.1.4, §18.2 | suite id `0x02` (reserved PQ) is a **known id** and decodes; note an object whose crypto actually **uses** `0x02` MUST still fail closed until the PQ suite is implemented (§18.2) | vector `suite_accept_0x02` | accept (id) | vectored |
| DMTAP-SUITE-07 | MUST | §1.1, §18.1.4 | unregistered suite `0x04` rejected on decode (never guess) | vector `suite_reject_0x04` | reject → 0x0101 / 0x0201 | vectored |

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
| DMTAP-FILE-05 | MUST | §5.5, §18.9.5 | the content address is over **ciphertext**: the **same plaintext** under two **different** per-file keys yields **different** `Manifest.id` (no cross-user/plaintext dedup — CAS-confirmation defense) | match (two distinct roots) | construction-todo |
| DMTAP-FILE-06 | MUST | §5.5.2, §18.3.7 | a **Referenced** (> 25 MiB) `ManifestRef` missing `durability`, or with an unknown `class` / `cluster-replicated` `replicas < 1` / `pinned` without `retention`, is rejected | reject → `ERR_FILE_MANIFEST_INVALID` (0x080A) | construction-todo |
| DMTAP-FILE-07 | MUST | §5.5.5, §16.4 | a **pushed** Inline/Attached file exceeding the recipient's inbound spool cap for that sender is refused fail-closed (spool-fill storage DoS), never silently accepted/dropped | reject → `ERR_SPOOL_OVERFLOW` (0x080C) | construction-todo |
| DMTAP-FILE-08 | MUST | §5.5.4, §5.5.2 | a `pinned(term)` (`class = 3`) fetch past its elapsed `retention` is rejected (host MAY have GC'd) | reject → `ERR_FILE_RETENTION_EXPIRED` (0x080B) | construction-todo |
| DMTAP-FILE-09 | MUST | §5.5.2, §5.5.3, §6.6 | a **Referenced** origin-hold file with no reachable holder and no satisfiable durability contract fails at the file level (distinct from a single missing chunk, 0x0803) | reject → `ERR_FILE_UNAVAILABLE` (0x0809) | construction-todo |

---

## Legacy level (§7)

Core + gateway inbound/outbound + DKIM delegation.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-LEG-01 | MUST | §7 | a gateway attestation that fails to verify under a trusted key is rejected | reject → `ERR_GATEWAY_ATTESTATION_INVALID` (0x0601) | construction-todo |
| DMTAP-LEG-02 | MUST | §7 | invalid DKIM delegation is rejected | reject → `ERR_DKIM_DELEGATION_INVALID` (0x0603) | construction-todo |
| DMTAP-LEG-03 | MUST | §7.11.2, §9.10, §7.12 | an outbound DMTAP→legacy relay from a sender the gateway has neither authenticated (no `GatewayAuthz`/key-registered relationship, §7.12) nor been paid by (no valid postage) is refused — a valid mesh `sender_sig` alone does NOT authorize egress (open-relay prevention) | reject → `ERR_GATEWAY_SENDER_UNAUTHENTICATED` (0x0607), FAIL_CLOSED_BLOCK | construction-todo |

---

## Clients level (§8)

Core + **JMAP**, the node's native (and only) client surface (§8.1). Legacy client protocols
(IMAP/POP/SMTP-submission, CalDAV/CardDAV) are a **gateway** capability (RECOMMENDED, §7.15), not
a node one.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-CLI-01 | MUST | §8.1 | a MOTE renders to/from the native JMAP object model without loss of the fields §8 requires | accept (round-trip) | construction-todo |

---

## Auth level (§13)

Core + DMTAP-Auth login ceremony with origin binding + key-bound sessions; OIDC bridge RECOMMENDED.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-AUTH-01 | MUST | §18.9.8, §13.3 | `Assertion.sig` over `DS ‖ BLAKE3-256(det_cbor([rp_origin,nonce,issued_at,exp,aud,scope,cnf]))` under the IK-authorized device key (`scope` is `[]` when absent; inside the signed preimage — scope-binding) | match (sig) | construction-todo |
| DMTAP-AUTH-02 | MUST | §13.3 | an assertion whose `rp_origin`/`aud` mismatch the issued `Challenge` is rejected | reject → `ERR_ORIGIN_MISMATCH` (0x0501) | construction-todo |
| DMTAP-AUTH-03 | MUST | §13.3 | a replayed `nonce` is rejected | reject → `ERR_NONCE_REPLAYED` (0x0502) | construction-todo |
| DMTAP-AUTH-04 | MUST | §13.3 | an expired `Challenge` is rejected | reject → `ERR_CHALLENGE_EXPIRED` (0x0503) | construction-todo |
| DMTAP-AUTH-05 | MUST | §18.9.8, §13.3 | the session is bound **only** to `cnf` (not the signing key), and MUST reject on `cnf` mismatch | accept (bound to `cnf`) | construction-todo |

---

## Deniable 1:1 mode (§5.2.1) — `DENIABLE`

The optional repudiable mode's fail-closed guards and the dedicated-`idk` handshake. The mode is
capability-negotiated (Private-level optional); the reject guards below are MUST **when the mode is
implemented**. No byte-exact vectors yet (Double Ratchet / X3DH use fresh randomness — the
dedicated `idk` change is flagged for re-vectoring, see README "deferred").

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-DENIABLE-01 | MUST | §5.2.1(c), §18.3.10 | a `DeniablePayload` carrying **any** signature field is rejected (a signature would defeat repudiation) | reject → `ERR_DENIABLE_SIGNATURE_PRESENT` (0x040F), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-DENIABLE-02 | MUST | §5.2.1(d), §10.2 | a deniable session to a peer that has **not** advertised `deniable-1:1` is refused; the client surfaces the choice, never silently downgrades | reject → `ERR_DENIABLE_MODE_UNAVAILABLE` (0x040E), REJECT_NOTIFY | construction-todo |
| DMTAP-DENIABLE-03 | MUST | §5.2.1(b), §18.9.10 | a `DeniableMessage` whose Double-Ratchet AEAD tag (shared-key MAC) fails is dropped; substitutes for `Payload.sig` (§20.2 fork) | reject → `ERR_DENIABLE_RATCHET_AUTH_FAILED` (0x040D), DROP_SILENT | construction-todo |
| DMTAP-DENIABLE-04 | MUST | §5.2.1(a), §18.4.8 | an invalid/exhausted `DeniablePrekeyBundle` (`sig`/`spk_sig`/`idk_sig` fail, or no unspent prekey) is rejected | reject → `ERR_DENIABLE_PREKEY_INVALID_OR_EXHAUSTED` (0x040B), REJECT_NOTIFY | construction-todo |
| DMTAP-DENIABLE-05 | MUST | §5.2.1(a), §18.3.9 | X3DH/PQXDH over the **dedicated `idk`** (NOT XEdDSA-from-`IK`): a `DeniableInit` whose `idk_a_cert` does not certify `idk_a` under `ik_a`, or whose key agreement fails, is rejected; last-resort-only first-message replay is caught | reject → `ERR_DENIABLE_X3DH_FAILED` (0x040C) | construction-todo |

---

## Organization administration (§3.10, §13.5.1) — `ORG`

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-ORG-01 | MUST | §3.10.2, §18.4.7 | an org-managed (escrowed-key) account presented **without** its `org-managed` custody marker is rejected — undisclosed escrow | reject → `ERR_ORG_MANAGED_UNDISCLOSED` (0x0115), HALT_ALERT | construction-todo |
| DMTAP-ORG-02 | MUST | §3.10.3, §3.9.4 | a `DirEntry` whose `name → ik` does not forward-verify against DNS+KT is rendered unverified, never used to address mail | reject → `ERR_DIRECTORY_ENTRY_UNVERIFIED` (0x0114), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-ORG-03 | MUST | §3.10.3, §18.4.7 | a `DomainDirectory` not signed by the pinned domain authority is rejected | reject → `ERR_DOMAIN_DIRECTORY_SIG_INVALID` (0x0113), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-ORG-04 | MUST | §13.5.1, §18.7.3 | a `CapabilityToken` whose link grants more than its parent (attenuation broken), is expired, or is invoked beyond its rights is rejected | reject → `ERR_CAPABILITY_DELEGATION_INVALID` (0x0508), DENY_POLICY | construction-todo |
| DMTAP-ORG-05 | MUST | §13.5.1, §18.7.3 | a validly-formed `CapabilityToken` covered by a published `CapabilityRevocation` (from its issuer/ancestor) is denied | reject → `ERR_CAPABILITY_REVOKED` (0x050B), DENY_POLICY | construction-todo |

---

## KT-v1 hardening (§3.5.2) — `KTV1`

Optional (log-type `0x02`, negotiated); the equivocation/quorum guards are MUST **when v1 is
implemented**. RFC 6962-profiled objects (`SignedTreeHead`/`InclusionProof`/`ConsistencyProof`,
§18.4.9–§18.4.11).

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-KTV1-01 | MUST | §3.5.2(a),(d), §18.4.9 | two validly-signed STHs of one log with equal `tree_size` but differing `root_hash` (or no consistency proof) ⇒ equivocation: HALT + alert + publish evidence | reject → `ERR_KT_STH_INCONSISTENT` (0x0110) + `ERR_KT_EQUIVOCATION` (0x0107), HALT_ALERT | construction-todo |
| DMTAP-KTV1-02 | MUST | §3.5.2(b), §18.4.10 | a `name → ik` binding not attested by a `> n/2` quorum of the pinned log set fails closed → OOB | reject → `ERR_KT_LOG_QUORUM_UNMET` (0x0111), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-KTV1-03 | MUST | §3.5.2(a), §16.2 | a `SignedTreeHead` older than the freshness window (freeze attack) is treated as stale and refreshed | reject → `ERR_KT_STH_STALE` (0x0112), HOLD_RESYNC | construction-todo |
| DMTAP-KTV1-04 | MUST | §18.4.9, §18.4.10 | an `InclusionProof` whose committed leaf ≠ the recomputed Identity-entry leaf-hash `0x1e‖BLAKE3-256(0x00‖det_cbor([name,ik,version,identity_id]))` is rejected | reject → `ERR_KT_LEAF_HASH_MISMATCH` (0x0117), FAIL_CLOSED_BLOCK | construction-todo |

---

## Device attestation (§1.2a) — `ATTEST`

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-ATTEST-01 | MUST | §1.2a, §18.4.2 | an attestation-gated context rejects a device whose `key_protection`/`attestation` is absent or fails against the platform root (advisory — never overrides §1.4 authority) | reject → `ERR_DEVICE_ATTESTATION_INVALID` (0x0116), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-ATTEST-02 | MUST | §1.2a, §18.4.2, §16.9 | evidence older than the re-attestation cadence (≤ 90 d), past its window, or chaining only to a retired root is treated as expired → re-attest | reject → `ERR_DEVICE_ATTESTATION_EXPIRED` (0x0118), FAIL_CLOSED_BLOCK | construction-todo |

---

## Profile — human display data (§3.9.5) — `PROFILE`

The self-asserted, signed display object (`display_name` / name parts / avatar). Signed by the
`IK` (or an `IK`-authorized device key) and authenticated to the key exactly like `Identity.names`
— a replaceable pointer, never a real-world-identity claim. No byte-exact vectors yet (a signed
`Profile` KAT is added when the reference gains the object); the reject guards below are MUST.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-PROFILE-01 | MUST | §3.9.5, §18.4.12, §18.9.3 | a `Profile` whose `sig` (DS-tag `DMTAP-v0/profile`) does not verify under the identity's `IK` / an `IK`-authorized device key is rejected; the prior pinned profile (or the fallback ladder) is used | reject → `ERR_PROFILE_SIG_INVALID` (0x0119), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-PROFILE-02 | MUST | §3.9.5, §18.4.12 | a `Profile` whose `avatar.hash` is present but the bytes fetched from `avatar.url` do **not** content-address (`0x1e ‖ BLAKE3-256`) to it MUST NOT be displayed; the client falls back down the §3.9.5 ladder (key-derived identicon → initials) and warns | reject → `ERR_PROFILE_AVATAR_HASH_MISMATCH` (0x011A), USER_WARN | construction-todo |

---

## Push wake-signaling (§4.9, OPTIONAL) — `PUSH`

The optional wake layer (§4.9): a device registers a `PushSubscription` with its own node, and the
node emits a content-free, sender-blind `WakePing`. Push is **not required for Core** (§10.3) — these
guards are conditional and MUST hold **only when a node implements the optional `push-wake`
capability** (§10.2, §21.22), mirroring how the `DENIABLE` guards apply only when the deniable mode is
implemented. No byte-exact vectors yet (RFC 8291 sealing uses fresh randomness); the reject guards
below are MUST.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-PUSH-01 | MUST | §4.9.1, §18.5.6, §18.9.15 | a `WakePing` carrying any field beyond the opaque sealed token (key `1`) — or whose opened plaintext bears sender/subject/recipient/content — MUST be rejected: a wake is content-free and sender-blind | reject → `ERR_WAKEPING_CONTENT_PRESENT` (0x0313), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-PUSH-02 | MUST | §4.9.1, §4.9.4, §18.5.5, §18.9.15 | a `PushSubscription` whose `sig` does not verify under an `IK`-authorized `device_key` (§1.2) MUST be rejected and never woken against — the subscription must be authenticated to the identity | reject → `ERR_PUSH_SUBSCRIPTION_SIG_INVALID` (0x0312), FAIL_CLOSED_BLOCK | construction-todo |

---

## Device-cluster sync (§5.6) — `SYNC`

Level **Groups & Files**. The personal-cluster convergence: mutual-auth membership (§5.6.1),
range-based Merkle backfill + journal replay (§5.6.3), and the OR-Set / HLC-LWW CRDT merge
(§5.6.4). Frames ride inside the encrypted MLS cluster group (§18.6.3); no byte-exact vectors yet.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-SYNC-01 | MUST | §5.6.1, §18.6.3 | a `ClusterSyncFrame`/`ClusterOp` from a device whose `DeviceCert` is absent/invalid or **revoked** (KeyRotation-excluded) under the owner's `IK` is refused — replication is mutually authenticated, a non-member cannot inject or pull | reject → `ERR_CLUSTER_DEVICE_UNAUTHORIZED` (0x0410), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-SYNC-02 | MUST | §5.6.3(a), §18.6.3 | a `recon` summary whose `RangeFingerprint.fp` does not recompute over the receiver's ids in `[lo,hi)` (forged Merkle fingerprint) is rejected and reconciliation re-driven against another peer | reject → `ERR_CLUSTER_RECON_SUMMARY_INVALID` (0x0411), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-SYNC-03 | MUST | §5.6.3(b), §18.6.3 | a journal-replay segment whose `prev` hash-chain does not verify (a fork/rewrite of the owner's own log) is halted on, analogous to a committer fork | reject → `ERR_CLUSTER_JOURNAL_CHAIN_BROKEN` (0x0412), HALT_ALERT | construction-todo |
| DMTAP-SYNC-04 | MUST | §5.6.4, §16.10, §18.6.3 | a `ClusterOp` with an unknown kind, an OR-Set remove citing an unknown add-tag, an HLC `wall` beyond the skew bound, or embedding a `DeniablePayload`/its plaintext (§5.2.1) is rejected | reject → `ERR_CLUSTER_CRDT_OP_INVALID` (0x0413), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-SYNC-05 | MUST | §5.6.4 | **convergence (SEC)**: two replicas applying concurrent OR-Set add/remove + per-field LWW ops in **any order** reach the **identical** state — add-wins-over-unseen-remove; the greater HLC `(wall,counter,device)` wins each field deterministically | accept (both replicas converge to the same state) | construction-todo |

---

## Aliases — many names, one key (§3.9.4, §3.11) — `ALIAS`

Level **Core**. Self-asserted aliases require forward verification; each is independently
revocable; every verified alias resolves to the same identity key.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-ALIAS-01 | MUST | §3.9.4, §3.11.3 | a name in the identity's own `Identity.names` whose forward `name → ik` binding (DNS+KT) resolves to a **different** key is rendered unverified and MUST NOT be displayed as authenticated nor used to address mail | reject → `ERR_ALIAS_FORWARD_UNVERIFIED` (0x011C), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-ALIAS-02 | MUST | §3.9.4, §3.11.5 | a **revoked** alias (dropped in a newer signed `Identity`, its binding retired) used off a stale cache to address the identity is refused; the key and the identity's other aliases are unaffected | reject → `ERR_ALIAS_REVOKED` (0x011D), REJECT_NOTIFY | construction-todo |
| DMTAP-ALIAS-03 | MUST | §3.9.4, §3.11.3, §18.4.9 | multiple **verified** aliases (distinct `name`, same `ik`/`identity_id`) resolve to the **same** identity — recognized as one person/one key, pinned per-key | accept (all aliases resolve to one identity_id) | construction-todo |

---

## Pluggable resolver framework (§3.12) — `RESOLVE`

Level **Core**. Resolution is always discover-then-KT-verify (§3.12.1); a resolver never
introduces its own trust root, unknown types fail closed (never guessed), and independent
resolvers must agree on the one key.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-RESOLVE-01 | MUST | §3.12.5(b) | a `name-chain` resolution whose two binding directions **disagree** — an on-chain `name → ik` record naming a key that does not claim the name in its signed `Identity.names`, or a claimed name whose chain record resolves to a **different** key — is rendered unverified and MUST NOT be used to address mail | reject → `ERR_NAMECHAIN_BINDING_UNVERIFIED` (0x011E), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-RESOLVE-02 | MUST | §3.12.2 | a name in a resolver type the verifier does not implement, or that is unregistered, is treated as unresolvable and fails closed — the "unknown ⇒ reject, never guess" discipline; the identity stays reachable via its other resolvers and the key-name (§3.9.6) | reject → `ERR_RESOLVER_TYPE_UNSUPPORTED` (0x011F), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-RESOLVE-03 | MUST | §3.12.3, §3.5.2(b) | two independent resolvers returning **different** `ik` for the **same** name is surfaced as a potential attack, never silently reconciled: the client MUST NOT pin, MUST alert, and MUST fall back to KT-quorum or OOB verification | reject → `ERR_RESOLVER_DISAGREEMENT` (0x0120), HALT_ALERT | construction-todo |

---

## Gateway alias mapping (§7.10) — `GWALIAS`

Level **Legacy**. Native↔legacy bridging via a swappable gateway alias: reversible encoded
local-parts, and legacy→native mapping that fails closed on an unmappable alias.

| id | req | clause | checks | expect | status |
|----|-----|--------|--------|--------|--------|
| DMTAP-GWALIAS-01 | MUST | §7.10.2, §18.3.12 | an `encoded` gateway alias `localpart.nativedomain@gateway.domain` that does not reversibly decode to exactly one `(localpart, nativedomain)` (ambiguous escaping) or exceeds RFC 5321 limits (§16.11) is rejected — the gateway MUST NOT guess a native address | reject → `ERR_GATEWAY_ALIAS_ENCODING_INVALID` (0x0606), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-GWALIAS-02 | MUST | §7.10.3, §18.3.12 | inbound legacy mail to a `random`-mode alias with no live `GatewayAliasMap` row (missing/expired/burned) is answered as "no such user," not silently dropped | reject → `ERR_GATEWAY_ALIAS_UNMAPPED` (0x0605), RETURN_SENDER_SMTP (`550 5.1.1`) | construction-todo |
| DMTAP-GWALIAS-03 | MUST | §7.10.2 | the encoded local-part **round-trips**: `encode(localpart, nativedomain)` (escape `-`→`--`, `.`→`-.`, join with a top-level `.`) then `decode` yields the original `(localpart, nativedomain)` — a deterministic KAT (e.g. `imran`+`mydomain.com` → `imran.mydomain-.com` → back) | match (decode(encode(x)) = x) | construction-todo |

---

## Vector cross-reference

Every `vectored` case above maps to an existing entry in `vectors/vectors.json`
(**33 of the 68 vectors** in the file are referenced by cases). Cross-check (case → vector):

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
| `suite_reject_0x00` / `_0x05` / `_0xff` / `_0x04` | SUITE-01 / -03 / -04 / -07 |
| `suite_accept_0x01` / `_0x02` / `_0x03` | SUITE-05 / SUITE-06 / SUITE-02 |
| `cbor_identity` / `cbor_device_cert` / `cbor_payload` / `cbor_envelope` | CBOR-01 / -02 / -03 / -04 |

No case references a vector that is absent from `vectors.json`. The six `self-contained` CBOR
reject cases (CBOR-05…CBOR-10) carry their bytes inline here and in `suite.json` and need no
reference implementation. The `construction-todo` cases carry a byte-exact recipe and the expected
§21 error; they are wired byte-for-byte as their subsystems gain fixed-input KATs (README
"Coverage vs. deferred").
