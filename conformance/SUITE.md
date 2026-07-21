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
| **input** | A **vector id** in `vectors.json` (or, for the `PUB`/`CAD` categories, `vectors/pub_vectors.json` — see those sections), an inline **self-contained construction** (bytes given here), or a **construction** described from other vectors. |
| **expect** | The required outcome: `match` (recompute a KAT and get the committed answer), `accept` (validation passes), or `reject` + the **§21 error code** the reject maps to. |
| **status** | `vectored` (byte-backed by `vectors.json` or `pub_vectors.json`), `self-contained` (bytes given inline, reference-independent), `construction-todo` (recipe given; byte-exact vector still to be generated), or `manual-attestation` (a client-UX, in-product-disclosure or deployment/process MUST with **no wire bytes to recompute** — §22.7's publish-consent disclosures, §4.4.10a's Bootstrap degradation disclosure and no-anonymity-claim rule, §7.11.4/§9.11's gateway posture, §7.1b's process/privilege separation; verified by implementer/deployment review, not by a runner. A vector is **not** invented for these: fabricating bytes would assert a fact the protocol does not carry). |

**Outcome vocabulary.** `match` = the operation is a deterministic known-answer test; recompute
it over the fixed input and the result MUST equal the committed `expected`. `accept` / `reject` =
the operation is a validation predicate; a conforming node MUST accept (resp. reject) the input,
and on `reject` MUST map it to the named §21 error code with that code's `Action` disposition
(DROP_SILENT / DEFER_REQUESTS / FAIL_CLOSED_BLOCK / HALT_ALERT / ACK_DEDUP …).

## Coverage summary

| Level | Cases | Vectored | Self-contained | Construction-todo | Manual |
|-------|------:|---------:|---------------:|-------------------:|-------:|
| **Core** — canonical CBOR (`CBOR`) | 12 | 4 | 6 | 2 | 0 |
| **Core** — content address (`ADDR`) | 6 | 6 | 0 | 0 | 0 |
| **Core** — Ed25519 (`SIG`) | 6 | 6 | 0 | 0 | 0 |
| **Core** — signing preimages (`PRE`) | 3 | 3 | 0 | 0 | 0 |
| **Core** — key-name (`NAME`) | 6 | 6 | 0 | 0 | 0 |
| **Core** — safety number (`SAFE`) | 2 | 2 | 0 | 0 | 0 |
| **Core** — suite fail-closed (`SUITE`) | 7 | 7 | 0 | 0 | 0 |
| **Core** — §2.7 validation pipeline (`VAL`) | 15 | 0 (2 reuse ADDR/PRE) | 0 | 15 | 0 |
| **Core** — identity / KT / naming (`IDENT`) | 6 | 0 | 0 | 6 | 0 |
| **Core** — aliases (`ALIAS`) | 3 | 0 | 0 | 3 | 0 |
| **Core** — resolver framework (`RESOLVE`) | 3 | 0 | 0 | 3 | 0 |
| **Private** (`PRIV`) | 7 | 0 | 0 | 7 | 0 |
| **Groups & Files** (`GRP`, `FILE`) | 12 | 0 | 0 | 12 | 0 |
| **Groups & Files** — device-cluster sync (`SYNC`) | 5 | 0 | 0 | 5 | 0 |
| **Legacy** (`LEG`) | 3 | 0 | 0 | 3 | 0 |
| **Legacy** — gateway alias mapping (`GWALIAS`) | 3 | 0 | 0 | 3 | 0 |
| **Clients** (`CLI`) | 1 | 0 | 0 | 1 | 0 |
| **Auth** (`AUTH`) | 5 | 0 | 0 | 5 | 0 |
| **Private** — deniable 1:1 mode (`DENIABLE`) | 5 | 0 | 0 | 5 | 0 |
| **Core** — org administration (`ORG`) | 5 | 0 | 0 | 5 | 0 |
| **Private** — KT-v1 hardening (`KTV1`) | 4 | 0 | 0 | 4 | 0 |
| **Core** — device attestation (`ATTEST`) | 2 | 0 | 0 | 2 | 0 |
| **Core** — profile / avatar (`PROFILE`) | 2 | 0 | 0 | 2 | 0 |
| **Optional** — push wake-signaling (`PUSH`) | 2 | 0 | 0 | 2 | 0 |
| **Private** — Bootstrap mix profile (`MIXPROF`) | 5 | 0 | 0 | 3 | 2 |
| **Private** — derived mix-fleet view (`FLEET`) | 3 | 0 | 0 | 3 | 0 |
| **Private** — guards & path diversity (`GUARD`) | 3 | 0 | 0 | 3 | 0 |
| **Core** — location & resolution order (`LOC`) | 2 | 0 | 0 | 2 | 0 |
| **Core** — zero-relationship delivery floor (`FLOOR`) | 4 | 0 | 0 | 4 | 0 |
| **Core** — §10.7.0 failure classes (`FAILCLASS`) | 2 | 0 | 0 | 2 | 0 |
| **Legacy** — gateway role boundaries (`GWROLE`) | 3 | 0 | 0 | 1 | 2 |
| **Core** — DMTAP-PUB extension, optional `pub-1` (`PUB`) | 21 | 12 | 0 | 8 | 1 |
| **Core** — CAD/artifact profile, optional `pub-1` (`CAD`) | 11 | 0 | 0 | 11 | 0 |
| **Core** — Video/Media profile, optional `pub-1` (`VIDEO`) | 15 | 0 | 0 | 15 | 0 |
| **Legacy** — operator prerequisites, discovery & modes (`GWOPS`) | 7 | 0 | 0 | 4 | 3 |
| **Legacy** — SMTP legs: TLS, 8-bit/EAI, DSNs, response codes (`GWSMTP`) | 10 | 0 | 0 | 10 | 0 |
| **Legacy** — attestation binding & chaining (`GWATT`) | 6 | 0 | 0 | 5 | 1 |
| **Legacy** — alias minting, parsing & safety tiers (`GWNAME`) | 6 | 0 | 0 | 5 | 1 |
| **Legacy** — inbound anti-abuse floor (`GWFLOOR`) | 3 | 0 | 0 | 3 | 0 |
| **Legacy** — legacy client surfaces (`GWLEG`) | 3 | 0 | 0 | 2 | 1 |
| **Total** | **229** | **46** | **6** | **166** | **11** |

The 46 vectored + 6 self-contained cases (**52**) are fully machine-runnable **today** from
`vectors.json` / `pub_vectors.json` + the inline bytes here, with **no reference implementation
required**. They pin the entire deterministic, security-critical Core spine — canonical CBOR,
content addressing, the two MOTE signature preimages (§18.9.1/§18.9.2), Ed25519 (with RFC 8032
cross-checks), the 8-word key-name, safety numbers, suite fail-closed — **and the full DMTAP-PUB
manifest/announce/feed KAT set** (§22.2/§22.3/§22.4: plaintext chunk hashing + DS-tagged Merkle
root, the announce and feed-head signing preimages, `announce_id`, the prev-chain,
type-incompatibility with sealed manifests, the same-author supersede rule, and feed anti-rollback
incl. the idempotent-refetch and fork/equivocation branches).

The 166 `construction-todo` cases give the exact recipe and expected §21 error for every remaining
normative branch — the full §2.7 pipeline, identity/KT fail-closed, the higher levels, the
hardening families (`DENIABLE`/`ORG`/`KTV1`/`ATTEST`), the `PROFILE` display-data guards, the
pluggable-resolver guards (`RESOLVE`), the optional `PUSH` wake-signaling guards, the `FILE`
durability guards, the anti-drift families
(`MIXPROF`/`FLEET`/`GUARD`/`LOC`/`FLOOR`/`FAILCLASS`/`GWROLE`), the gateway families
(`GWOPS`/`GWSMTP`/`GWATT`/`GWNAME`/`GWFLOOR`/`GWLEG`), the remaining `PUB` fail-closed rows not yet
vectored, and the profile-level `CAD` and `VIDEO` checklists. Each becomes byte-backed when the
corresponding subsystem gains a fixed-input KAT (see README "Coverage vs. deferred").

The 11 `manual-attestation` cases are the MUSTs with **no wire bytes to recompute**: an in-product
disclosure, a share sheet, a process boundary or the population a deployment actually serves. They
are identified by `manual-attestation` in the **status** column, and each names the review that
settles it — client-UX review, operator-copy review, or deployment review.
**Fabricating byte vectors for them would assert a fact the protocol does not carry**; attestation
is the honest status, not a placeholder for a vector that could exist.

**Sync status:** `SUITE.md` and [`suite.json`](suite.json) are **in sync** — both carry the same
**229** case ids, and `make lint` (check C5) fails the build if they ever disagree, or if any
document states a different count. The changed deniable objects (§5.2.1 dedicated-`idk`) are still
to be re-vectored when the reference regenerates `vectors.json`.

> All 46 vectored cases correspond one-for-one to entries in `vectors.json` (34 cases / 33 of its
> 68 vectors — several vectors drive more than one case; the remaining 35 are pre-generated for
> construction-todo families not yet wired to a case) or `pub_vectors.json` (12 cases / all 15 of
> its vectors — several `PUB` cases reference more than one `pub_vectors.json` entry). No case
> references a vector entry that does not exist in its file; see
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
| DMTAP-PRIV-01 | MUST | §4.4.1, §16.3, §2.5 | Sphinx packet is constant-length on the bucket ladder **{16, 64} KiB**; wrong length rejected (neither the 2 KiB nor the 8 KiB rung can hold a conformant PQ envelope — under suite `0x02` a minimal MOTE is **11 967 B** before any body, §4.4.1) | reject → `ERR_MIX_PACKET_MALFORMED` (0x0307) | construction-todo |
| DMTAP-PRIV-02 | MUST | §4.4.2, §18.5.3 | a **cached** `MixDirectory` is verified against the client's pinned KT log quorum (`> n/2`), never under a directory authority — there is none (§4.4.2); an absent authority signature is not a failure, an unverifiable descriptor is (the descriptor-level MUST NOT is `DMTAP-FLEET-02`) | reject → `ERR_MIX_DIRECTORY_SIG_INVALID` (0x030B), FAIL_CLOSED_BLOCK | construction-todo |
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

## Bootstrap mix profile (§4.4.10a) — `MIXPROF`

Level **Private**. The Bootstrap profile exists so a network too small to satisfy Standard's bar is
not non-functional by its own rules (§4.4.10a) — and its four normative constraints exist so the
profile cannot become a permanent silent downgrade. Constraint 3 is the load-bearing one: if
Bootstrap were reachable as a *fallback*, an adversary who DoSes or eclipses enough mixes forces
every sender down onto it, which is exactly the §4.4.9 downgrade attack under a friendlier name.
Constraints 1–2 are in-product UX MUSTs with no wire bytes and are `manual-attestation` (the
`DMTAP-PUB-21` precedent); constraints 3–4 are observable state transitions and map onto the
existing `0x0310`. Bootstrap is announced by the additive capability token `mix-profile-bootstrap`
(§10.2, §21.22); no object gains a field and no vector changes.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-MIXPROF-01 | MUST | §4.4.10a(1), §4.4.11 | **User-visible degradation.** A client operating on Bootstrap MUST show that metadata privacy is **degraded** and MUST say **why** — how many attested mixes and how many disjoint ASNs the network currently offers **against the Standard bar** (§16.3). Silence converts a temporary shortfall into an invisible one | manual attestation (client-UX review with the node in a fleet below Standard's bar; no wire bytes to recompute) | non-conformant if the degradation is not shown, or is shown without the current-vs-required mix/ASN counts | manual-attestation |
| DMTAP-MIXPROF-02 | MUST | §4.4.10a(2), §4.4.11 | **No anonymity claim in-product.** While Bootstrap is in force, an implementation or operator MUST NOT present the `private` tier as anonymous, MUST NOT report an anonymity-set size, and MUST NOT describe the traffic as unlinkable — §4.4.11's prohibition as a property of the profile itself | manual attestation (in-product copy, status surfaces, and operator-facing material) | non-conformant if any of the three claims appears while Bootstrap is in force | manual-attestation |
| DMTAP-MIXPROF-03 | MUST | §4.4.10a(3), §4.4.2, §4.4.4 | **Auto-upgrade is mandatory.** A node MUST re-evaluate its derived fleet view **each mix-key epoch** and MUST move to **Standard** as soon as Standard's bar is satisfiable (≥ 3 hops one per layer, ≥ 3 disjoint **attested** operators, ≥ 3 disjoint ASNs, guard sample per §16.3). Remaining on Bootstrap with a satisfiable Standard bar is non-conformant — it is a silent permanent downgrade | construction: derived fleet view below Standard's bar at epoch `e` (Bootstrap in force); grow the view to satisfy Standard at `e+1`; step the node's epoch re-evaluation with no user action | accept (in-force profile = Standard from `e+1`, unprompted); non-conformant if it stays on Bootstrap | construction-todo |
| DMTAP-MIXPROF-04 | MUST | §4.4.10a(3), §4.4.9, §10.7.0 | **Never fall back.** Once a node has operated at Standard it MUST NOT return to Bootstrap. If the fleet later shrinks below Standard's bar the sender **holds and retries** (FAIL-QUEUED) and surfaces the refusal past the retry deadline — it MUST NOT rebuild the path at Bootstrap's bar. This is the case that closes "DoS the mixes to force everyone onto the degraded profile" | construction: node at Standard; shrink the derived fleet below Standard's bar (kill attested mixes / collapse ASN diversity); attempt a `private` send | reject → `ERR_PRIVATE_TIER_DOWNGRADE_REFUSED` (0x0310); message stays queued (§10.7.0 FAIL-QUEUED), MUST NOT be carried at Bootstrap's bar and MUST NOT be refused enqueue | construction-todo |
| DMTAP-MIXPROF-05 | MUST | §4.4.10a(4), §1.3, §4.4.9 | **Per-contact ratchet.** Profile level ratchets **per correspondent**, exactly like the §1.3 suite high-water-mark: once any message in a relationship has been carried at Standard (or High-security), a Bootstrap-tier send to that contact MUST fail closed. The granularity is deliberate — a **new** contact on a still-small network MUST remain reachable at Bootstrap in the same fleet state, so the attacker must defeat one relationship at a time | construction: contact **A** with a Standard-carried message in its history and contact **B** never carried above Bootstrap; with the fleet below Standard's bar, attempt a `private` send to each | reject → `ERR_PRIVATE_TIER_DOWNGRADE_REFUSED` (0x0310) for **A**; accept (Bootstrap path built) for **B**; a global rather than per-contact mark fails one branch or the other | construction-todo |

---

## Derived mix-fleet view (§4.4.2) — `FLEET`

Level **Private**. The mix directory is a **derived view, not an authority-signed artifact**: each
mix publishes its `MixNodeDescriptor` into the KT logs and every client computes the fleet locally
from a `> n/2` quorum of its pinned logs. A signed fleet snapshot would make its signer the most
powerful party in the protocol — it picks the anonymity set, and its silence stops all
`private`-tier mail — so the cases below assert that a served `MixDirectory` is only ever a
**cache**: convenient, verifiable, never load-bearing. `DMTAP-PRIV-02` states the same rule at the
directory level; `-02` here is the per-descriptor MUST NOT.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-FLEET-01 | MUST | §4.4.2, §3.5.2(b) | **Reconstructable from logs alone.** With **no** served `MixDirectory` available, a client MUST still compute its epoch fleet view from its pinned KT log quorum: the set of descriptors that (i) self-verify under `node_ik`, (ii) carry a valid `_dmtap-mix` operator attestation, (iii) name a current-epoch Sphinx key, and (iv) appear with a valid inclusion proof in `> n/2` of the pinned logs. An implementation that cannot build a `private` path without a served directory has re-created the authority §4.4.2 deleted | construction: populate the pinned log set with `n` descriptors (some failing exactly one predicate); withhold every `MixDirectory` cache; ask the client for its epoch fleet view | accept (derived view = exactly the descriptors satisfying all four predicates; identical to the view derived when a valid cache *is* present) | construction-todo |
| DMTAP-FLEET-02 | MUST | §4.4.2, §18.5.3 | **A cache is never an authority.** A client MUST NOT accept a served/cached `MixDirectory` containing a descriptor it cannot independently verify against its own log quorum (any of `DMTAP-FLEET-01`'s four predicates failing), and MUST reject a cache whose `version` is older-or-equal to one already accepted (rollback). A hostile cache may withhold or reorder — never inject | construction: cache of valid descriptors plus one whose `_dmtap-mix` attestation does not validate (variant: one absent from the log quorum); and separately, replay a cache at an older `version` | reject → `ERR_MIX_DIRECTORY_SIG_INVALID` (0x030B), FAIL_CLOSED_BLOCK; the unverifiable descriptor MUST NOT enter path selection | construction-todo |
| DMTAP-FLEET-03 | MUST | §4.4.2, §10.7.2, §10.7.0, §16.3 | **Freeze defense is FAIL-QUEUED, not fail-closed.** A derived view (or cache) older than the mix-directory freshness window (§16.3, ≤ one mix-key epoch) is **stale**: the client MUST refresh before building any `private` path, and if it cannot obtain a fresh one it **queues and retries**. It MUST NOT downgrade the tier and MUST NOT refuse to enqueue — a directory outage delays mail, it must never stop it | construction: freeze the client's log/cache feed at a view older than the freshness window; submit a `private` MOTE from the user | reject (path build) → `ERR_MIX_DIRECTORY_STALE` (0x0311), FAIL-QUEUED per §10.7.0 — **and** accept (enqueue): the MOTE is durably held and retried; emitting a tier downgrade, or refusing the enqueue, is non-conformant | construction-todo |

---

## Entry guards & path diversity (§4.4.8) — `GUARD`

Level **Private**. Guards and diversity defend **different** attacks and neither substitutes for the
other (§4.4.10): guards bound the long-horizon intersection attack, diversity bounds the
both-ends-adversarial placement. Each case below pins the mechanism that makes the stated bound
true rather than merely stating the bound.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-GUARD-01 | MUST | §4.4.8, §16.3 | **Guards come from a persistent sampled set, never a fresh fleet draw.** The active `G` guards MUST be drawn only from the node's persistent **guard sample**, chosen once; rotation moves the active guards *inside* the sample and MUST NOT re-sample from the fleet. The sample is refreshed only on exhaustion (sampled nodes permanently offline) or an explicit owner re-sample (a disclosed exposure event). Re-drawing per rotation turns the `(1−f)^G` bound into `(1−f)^(G·r)` — at `G=2`, 30-day rotation, `f=0.05`, a decade is `0.95^244 ≈ 3×10⁻⁷`, i.e. near-certain eventual exposure. This is Tor prop-271's fix | construction: fleet of `N` attested, ASN-disjoint mixes, unchanged throughout; record the node's initial guard sample; advance `r ≥ 10` guard-rotation periods (§16.3) | accept (every active guard at every rotation ∈ the initial sample, and the sample is byte-identical throughout); any guard outside the initial sample, absent exhaustion or an explicit re-sample, is non-conformant | construction-todo |
| DMTAP-GUARD-02 | MUST | §4.4.8, §4.4.9 | **ASN-disjointness in path selection.** A path MUST traverse mixes under **disjoint announced BGP origin ASNs**; a candidate path whose hops carry three distinct *attested* operators but announce from **one** ASN MUST NOT be built. Domains are cheap — attestation proves accountability, not independence — and ASN is the axis rented capacity cannot cheaply diversify | construction: three mixes with distinct, validly-attested `operator` domains, all reachable via addresses announced by a single origin ASN; request a Standard-profile path | reject → `ERR_MIX_PATH_UNBUILDABLE` (0x030D); the sender holds per §4.4.9 (`0x0310` past the retry deadline) and MUST NOT route over the ASN-colliding path | construction-todo |
| DMTAP-GUARD-03 | MUST | §4.4.8 | **An un-attested `operator` claim confers no diversity.** A mix whose `operator` is absent or not backed by a valid `_dmtap-mix` attestation under that domain MUST NOT count as its own operator for the disjoint-operator rule — it is excluded from selection or counted as one unknown/shared operator. Otherwise a single adversary publishes *N* self-claimed operators and collapses the ≈ *a*² compromised-path bound to ≈ *a* | construction: three mixes, each self-asserting a **different** `operator` value, none carrying a resolvable/valid `_dmtap-mix` record; request a Standard-profile path | reject → `ERR_MIX_PATH_UNBUILDABLE` (0x030D); counting the three as three operators is non-conformant | construction-todo |

---

## Location records & resolution order (§4.2, §4.2.1) — `LOC`

Level **Core**. Two properties of the reachability layer that the mixnet's guarantees rest on: a
routing identifier that does not survive its epoch (so a harvested v0-Sphinx corpus resolves to
expired pseudonyms), and a resolution order in which an established relationship performs **no
lookup at all** (so the eclipse attack, which is an attack on lookup, is off the path that matters).

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-LOC-01 | MUST | §4.2, §16.2, §4.4.12 | **`peer_id` is per-epoch unlinkable.** A node MUST derive its `peer_id` **freshly per location epoch** rather than holding one identifier across its lifetime, so a `peer_id` observed at *T* cannot be linked to the same node at *T+Δ* by the identifier alone. This is what bounds the harvest-now-decrypt-later exposure of the routing layer: a route recovered in 2040 names an identifier that expired in 2026 | construction: publish the node's `LocationRecord` for two consecutive location epochs `e`, `e+1` under the same `ik`; both validly signed, `seq` strictly increasing | match (`peer_id(e) ≠ peer_id(e+1)`, both records verify); a stable lifetime `peer_id` is non-conformant | construction-todo |
| DMTAP-LOC-02 | MUST | §4.2.1 | **Piggybacked location precedes every lookup.** A resolver MUST try, in order: (1) the sender's signed `LocationRecord` carried by the MOTE itself, (2) cached direct addresses, (3) the home rendezvous set (≥ 3 disjoint operators), (4) the DHT — opportunistic only, and a DHT-only record is a **hint** (usable to attempt a connection, never to authenticate; identity is authenticated by the pinned `IK` as always). Replying to a MOTE that carried a fresh, valid record MUST perform **no** rendezvous query and **no** DHT lookup | construction: instrumented resolver counting rendezvous/DHT queries; deliver a MOTE carrying the sender's current signed `LocationRecord`, then send a reply | accept (zero lookups of any kind; the piggybacked record is used); any lookup on this path is non-conformant | construction-todo |

---

## Zero-relationship delivery floor (§9.7a, §9.4.1) — `FLOOR`

Level **Core**. §3.13 promises that a user with no domain, no name-chain and no provider is a
first-class identity; that promise is only true if such a user can **deliver**. Unknown-issuer ARC
budget is zero, postage needs an issuer, vouch needs a mutual contact — compose those and a
sovereign key-name identity is nameable, reachable, verifiable and **silently undeliverable**. The
floor is normative precisely because every recipient's *local* incentive is to set it to zero, so
it is a collective-action problem and the conformance suite is the only place to solve it.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-FLOOR-01 | MUST | §9.7a, §16.5, §2.7a | **The floor delivers.** A recipient MUST accept at least `N_floor` (§16.5, **≥ 5**) cold MOTEs **per sender-key per day** from a sender presenting **only** a valid work proof — a sequential-work `vdf` (§9.4.1) **or** a memory-hard PoW (§9.4) — bound to the recipient's current epoch beacon, with **no** token, **no** postage and **no** vouch. Floor deliveries land in the **requests area** (§2.7a), never the inbox, and are **not** acked (§2.7a, `DMTAP-VAL-12`) | construction: `N_floor` cold MOTEs from one fresh sender key inside 24 h, each carrying a valid proof scoped per §9.2a (incl. `sender_key`) and no other credential; recipient has no relationship with the sender | accept (all `N_floor` durably held in the requests area for the §16.5 retention; none dropped; **no ack** emitted); dropping any of them is non-conformant | construction-todo |
| DMTAP-FLOOR-02 | MUST | §9.7a, §16.5 | **A zero floor is a non-conformant policy.** A recipient MAY grant far more than the floor to trusted issuers, vouched senders or paid postage, but MUST NOT grant less: `N_floor = 0` (or below §16.5's minimum) as a **standing** policy — including a shipped default, and including a "reject all unknown senders" configuration offered without the §9.7a disclosure — MUST be refused, not silently clamped. Under active flood the §9.4 deferral budget MAY be applied to floor traffic as to any other; that is a transient budget, not a standing floor of zero | construction: apply a recipient `Policy` (§9.2) with `N_floor = 0`; variant: a policy UI offering "reject all unknown senders" as a reachable configuration with no disclosure | reject → `ERR_POLICY_BELOW_FLOOR` (0x070F), REJECT_NOTIFY — the node MUST NOT apply the policy while reporting conformance | construction-todo |
| DMTAP-FLOOR-03 | MUST | §9.4.1, §9.7a, §16.5 | **Memory-hard PoW is the interoperable MUST floor.** A conformant recipient MUST accept a valid memory-hard PoW solution (Argon2id at or above its advertised difficulty, scope incl. `sender_key` per §9.2a) as satisfying its cold-contact requirement, subject only to the §9.4 verification budget and its own rate policy. VDF is an *optional* cost (**MAY**, §9.4.1) — it bounds aggregate parallelism but leaves a 10–100× latency advantage, its sequentiality is conjectural and processor-bounded, and it is **not post-quantum**, so it is neither the floor nor a SHOULD | construction: cold MOTE carrying a valid Argon2id solution at the recipient's advertised difficulty and **no** VDF proof, from a sender with no relationship | accept (routed to the requests area exactly as `DMTAP-FLOOR-01`); rejecting it because it is not a VDF is non-conformant | construction-todo |
| DMTAP-FLOOR-04 | MUST | §9.4.1, §9.7a | **VDF-only is a non-conformant policy.** A recipient MUST NOT require a VDF as the *only* acceptable proof. A recipient that would accept only a VDF has made an unstandardized, non-post-quantum construction the price of contacting it, and a sender that cannot produce the one proof a recipient will take is simply undeliverable — the floor failing in a new way rather than holding | construction: apply a `ChallengeSpec`/policy accepting `vdf` only, such that the valid PoW of `DMTAP-FLOOR-03` would be refused | reject → `ERR_POLICY_BELOW_FLOOR` (0x070F), REJECT_NOTIFY | construction-todo |

---

## Failure classes (§10.7.0) — `FAILCLASS`

Level **Core**. "Fail closed" names three different behaviors — FAIL-CLOSED-AUTH, FAIL-QUEUED,
FAIL-DEGRADED — and conflating them produces a protocol that is secure and unusable. A failure of
*authenticity* must never become a delay; a failure of *liveness* must never become a rejection,
because classifying a liveness failure as fail-closed hands a denial-of-service surface to anyone
who can take a service offline. The governing invariant: **an offline-first store-and-forward
protocol must never be unable to queue.** (`DMTAP-FLEET-03` is the individually-identified
FAIL-QUEUED instance; `DMTAP-MIXPROF-04` is the FAIL-QUEUED-without-downgrade instance.)

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-FAILCLASS-01 | MUST | §10.7.0 | **No liveness failure may prevent enqueue.** Whatever is unreachable — KT logs, the derived fleet view, a postage issuer, the rendezvous set, every peer — a node MUST always be able to accept a message from its own user, hold it **durably**, and keep retrying under the guarantees in force. Email's decisive operational property is that it degrades to *late*, never to *blocked* | construction: partition the node from every external dependency simultaneously; submit a `private`-tier message over the node's native client surface (§8.1); restart the node | accept (message enqueued, still present and still retrying after restart); refusing the submission — for any unreachable dependency — is non-conformant | construction-todo |
| DMTAP-FAILCLASS-02 | MUST | §10.7.0, §21.2 | **Each condition takes exactly its class's disposition.** A FAIL-CLOSED-AUTH condition (a signature that does not verify, a pin mismatch, a log equivocation, a suite below the ratchet) is refused **permanently**: retrying cannot help and MUST NOT be offered or silently performed. A FAIL-QUEUED condition (an external service unreachable *right now*) MUST NOT be surfaced as a permanent rejection before the retry deadline (§16), and MUST NOT weaken any guarantee in the meantime | construction: drive one instance of each class against the same node — a tampered `Payload.sig` (§2.7 step 8) and a stale derived fleet view (§4.4.2) — and inspect the retry queue and the user-facing outcome for both | accept (the `0x0208` condition is permanent, never queued and never offered as retryable; the `0x0311` condition is queued and retried, never presented as a rejection before the retry deadline); either mapping inverted is non-conformant | construction-todo |

---

## Gateway role boundaries (§7.1b, §7.11.4, §9.11) — `GWROLE`

Level **Legacy**. The gateway is a **legacy adapter**, and these are the two boundaries that keep it
one. (i) It **authorizes** — SPF/DKIM/DMARC results, IP standing, authenticated sender identity,
cold-sender state, rate counters — and **never classifies content**; a gateway that classifies is
permanent by construction, because classification improves with corpus size, never terminates, and
makes everyone's mail depend on a judgement only the operator can make (the measured evidence is in
§9.11: third-party mail-security vendors rank in the top five by MX share while not being mailbox
providers at all). (ii) One binary must never mean one address space: gateway mode terminates
untrusted port-25 connections and runs the most-exploited parsers in mail, so it MUST be a separate
process with no reach into `IK` or the MOTE store. `-02` and `-03` are `manual-attestation`: a
process boundary and a product's user-facing copy have no wire bytes to recompute.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-GWROLE-01 | MUST | §7.11.4, §9.11 | **Authorize, never classify — differential test.** The gateway's admission decision MUST be a function of its permitted inputs only (SPF/DKIM/DMARC results, sending-IP standing, authenticated sender identity, cold-sender state, rate/volume counters). It MUST NOT run content-based scoring, Bayesian/learned filters, keyword or URL reputation, or attachment-content heuristics, and MUST NOT drop, quarantine, re-rank or annotate on such a basis | construction: two inbound legacy messages from the same authenticated sender inside its rate budget, identical in every authorization input (same envelope sender, same SPF/DKIM/DMARC result, same source IP, same cold-sender state), differing **only** in body/subject content — one carrying canonical spam-corpus text, one benign | accept (byte-identical admission decision, disposition and annotation for both); any divergence between the two is proof of content classification and is non-conformant | construction-todo |
| DMTAP-GWROLE-02 | MUST | §7.11.4, §9.11 | **No filter posture, no decryption for anti-abuse.** A gateway MUST NOT be required to decrypt for anti-abuse purposes and MUST NOT present its authorization checks to users as a spam filter. Classification, where it happens at all, is recipient-side and on-device against the user's own corpus, and a recipient MUST be able to run *no* classifier and still be protected — the §9.1–§9.7a mechanisms are sender-cost and policy mechanisms, not content judgements | manual attestation (review of the gateway's user-facing copy, admin surfaces and operator documentation; no wire bytes to recompute) | non-conformant if the authorization gate is described to users as spam filtering, or if any anti-abuse path requires access to plaintext it would not otherwise hold | manual-attestation |
| DMTAP-GWROLE-03 | MUST | §7.1b | **Privilege separation.** Gateway mode MUST run as a **separate process in a separate privilege domain** (distinct OS user; a distinct container/jail/namespace where the platform provides one) from the node's identity and store processes; it MUST NOT have access to identity key material (`IK`, device private keys, recovery material) and MUST NOT have read access to the local MOTE store. Legacy parsers (SMTP/IMAP/POP3/MIME/iCalendar/vCard) SHOULD be sandboxed further (seccomp/pledge-class filter, per-connection parsing process, or a memory-safe parser). "One binary" is a distribution property, never an isolation property — and the rule holds for a self-operated gateway too, because the threat is remote code execution by a stranger, not the operator | manual attestation (deployment review: process table and uid separation, keystore and MOTE-store access permissions, sandbox profile; no wire bytes to recompute) | non-conformant if gateway mode shares an address space or privilege domain with identity/store, or can read either | manual-attestation |

---

## DMTAP-PUB (§22) — `PUB`

Level **Core**, optional capability **`pub-1`** (§10.2, §21.22, §21.24b) — the identical treatment
as `PUSH` above: `pub-1` guards are MUST **when a node implements DMTAP-PUB**, never required for
bare Core conformance, and a node that never advertises `pub-1` is never expected to serve or
validate these objects. DMTAP-PUB is additive and capability-negotiated (§22, ROADMAP.md); it
bumps no `Envelope.v`, no DNS `v=` anchor, and introduces no flag day.

Byte-exact vectors live in **[`vectors/pub_vectors.json`](vectors/pub_vectors.json)** — a
**separate file** from `vectors/vectors.json`, generated by
**[`vectors/gen_pub_vectors.py`](vectors/gen_pub_vectors.py)** (a throwaway, deterministic Python
script; fixed Ed25519 seeds, fixed timestamps, fixed plaintext — no randomness), because the
`dmtap-core` reference crate that generates `vectors.json` does not yet implement the DMTAP-PUB
extension (ROADMAP.md "Envoir node `pub-1` serving" is still a follow-up wave). See
`README.md`'s Provenance section for why the two files are kept separate rather than merged, and
run `python3 conformance/vectors/gen_pub_vectors.py > conformance/vectors/pub_vectors.json` to
regenerate `pub_vectors.json` byte-for-byte from the script. A second, independent implementation
of the same formulas — [`vectors/verify_pub_vectors.py`](vectors/verify_pub_vectors.py), which does
not import the generator — cross-checks every committed vector (`python3
conformance/vectors/verify_pub_vectors.py`).

All digests below are **BLAKE3-256 with the v0 `0x1e` content-address prefix** (§18.1.5,
§22.2.2) and **Ed25519** (§18.1.6) — the same suite `0x01` primitives `vectors.json` uses — so a
runner that already implements the Core vector dispatch (README "How an implementation runs the
vectors") needs only two new operations (`pub_manifest_root`, the DS-tagged Merkle tree of
§22.2.2) plus the existing `ed25519_sign`/`content_address`/`det_cbor_decode`-family dispatch to
run every `PUB` case below.

### PubManifest / PubAnnounce / FeedEntry·FeedHead — known-answer tests

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-PUB-01 | MUST | §22.2.2 | `PubManifest.id` over a **single** plaintext chunk: `h_0 = 0x1e‖BLAKE3-256(plaintext_0)`, `id = 0x1e‖leaf(h_0)`, `leaf(h)=BLAKE3-256(DS‖0x00‖h)`, `DS="DMTAP-PUB-v0/manifest"‖0x00` | vector `pub_manifest_single_chunk` | match (`id_hex`) | vectored |
| DMTAP-PUB-02 | MUST | §22.2.2 | `PubManifest.id` over **3** ordered plaintext chunks (RFC 6962 split, `k=2`): `MTH(h_0,h_1,h_2) = node(node(leaf(h_0),leaf(h_1)),leaf(h_2))` | vector `pub_manifest_three_chunks` | match (`id_hex`) | vectored |
| DMTAP-PUB-03 | MUST | §22.3.1 | `PubAnnounce.sig` signing preimage: `Ed25519(signer_seed).sign(DS‖det_cbor(PubAnnounce ∖ {9}))`, `DS="DMTAP-PUB-v0/announce"‖0x00` | vector `pub_announce_signing_preimage` | match (`pubkey_hex`,`sig_hex`) | vectored |
| DMTAP-PUB-04 | MUST | §22.3.1, §18.9.4 | `announce_id = 0x1e‖BLAKE3-256(det_cbor(PubAnnounce))` over the **complete, signed** object (the derived-anchor rule, no self-`id` field) | vector `pub_announce_id` | match (`id_hex`) | vectored |
| DMTAP-PUB-05 | MUST | §22.4.1 | `FeedEntry.prev`-chain: genesis (`seq=0`, no `prev`) → entry (`seq=1`, `prev=`genesis id) → entry (`seq=2`, `prev=`prior id); each `FeedEntry_id` is content-addressed (§2.2 generic rule — see the vector's `note` for the inference this makes explicit, since §22.4.1 does not spell the formula out the way §22.3.1 does for `announce_id`) | vector `pub_feed_entry_chain` | match (`entry_ids_hex`), accept (chain valid) | vectored |
| DMTAP-PUB-06 | MUST | §22.4.1 | `FeedHead.sig` signing preimage over `v,suite,pub,seq,tip,ts,signer`: `Ed25519(signer_seed).sign(DS‖det_cbor(FeedHead ∖ {8}))`, `DS="DMTAP-PUB-v0/feed"‖0x00`; because `tip`'s entry chains `prev` back to genesis, signing `tip` transitively commits the whole log | vector `pub_feed_head_signing_preimage` | match (`pubkey_hex`,`sig_hex`) | vectored |

### §22.8 fail-closed table — individually-identified checks

Each row of the normative §22.8 table, as its own case, in the table's own order. A conformant
`pub-1` implementation enforces every one.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-PUB-07 | MUST | §22.2.3 | **Manifest DS-tag confusion (sealed ↔ public):** the *same* ordered chunk-hash list, rooted once under the public DS-tagged tree (§22.2.2) and once under the §18.9.5 bare sealed tree (no DS fold), MUST yield **different** roots — a verifier expecting one type recomputing over the other's bytes can never match | vector `pub_manifest_type_incompatibility` | match; `public_root_hex` ≠ `sealed_style_root_hex` → reject `ERR_PUB_MANIFEST_TYPE_MISMATCH` (0x0903), FAIL_CLOSED_BLOCK | vectored |
| DMTAP-PUB-08 | MUST | §22.2.1 | **Public manifest carries a key field:** a `PubManifest` CBOR map carrying the forbidden key `5` MUST be rejected — a public blob has no key by construction | vector `pub_manifest_key5_forbidden` | reject → `ERR_PUB_MANIFEST_KEY_PRESENT` (0x0902), FAIL_CLOSED_BLOCK | vectored |
| DMTAP-PUB-09 | MUST | §22.2.2 | **Public manifest self-verification:** a recomputed DS-tagged Merkle root that does not equal `PubManifest.id` MUST be rejected **before fetch begins** | construction: flip one byte of a chunk hash in `pub_manifest_three_chunks` and recompute the root | reject → `ERR_PUB_MANIFEST_HASH_MISMATCH` (0x0909), DROP_SILENT | construction-todo |
| DMTAP-PUB-10 | MUST | §22.2.2, §5.5.3 | **Public chunk self-verification:** a fetched plaintext chunk whose recomputed `h_i` disagrees with its listed `PubManifest.chunks` entry MUST be rejected and refetched from another holder | construction: tamper one byte of a chunk from `pub_manifest_three_chunks` and recompute `h_i` | reject → `ERR_PUB_CHUNK_HASH_MISMATCH` (0x090A), ROTATE_RETRY | construction-todo |
| DMTAP-PUB-11 | MUST | §22.3.1, §22.3.3 | **Announce content-address bind:** a recomputed `announce_id` that does not equal the address the object was fetched by MUST be rejected | construction: mutate one byte of the `pub_announce_id` vector's `bytes_hex` and recompute `announce_id` | reject → `ERR_PUB_ANNOUNCE_ID_MISMATCH` (0x0905), DROP_SILENT (retryable — re-fetch) | construction-todo |
| DMTAP-PUB-12 | MUST | §22.3.1, §22.3.3 | **Announce signature + IK chain:** `sig` failing under `signer`, or `signer` not authorized by `pub` (no valid `DeviceCert` chain), MUST be rejected | construction: flip one bit of `pub_announce_signing_preimage`'s `sig_hex` and re-verify against `pubkey_hex` | reject → `ERR_PUB_ANNOUNCE_SIG_INVALID` (0x0904), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-PUB-13 | MUST | §22.3.4, §22.3.3 step 5 | **Supersede is same-author:** a `supersedes` reference to an announce whose `pub` differs from this announce's `pub` MUST be rejected; a publisher may only supersede its own announcements | vectors `pub_announce_supersede_same_author_valid` (accept) + `pub_announce_supersede_cross_author_invalid` (reject) | accept (same author) / reject → `ERR_PUB_SUPERSEDE_INVALID` (0x090B), FAIL_CLOSED_BLOCK (cross author) | vectored |
| DMTAP-PUB-14 | MUST | §22.4.2 | **Feed `seq` anti-rollback:** a `FeedHead.seq` strictly below the highest accepted for that `pub` MUST be rejected; equal `seq` + identical `tip` is an idempotent accept (a cacheable re-fetch, never an error); equal `seq` + different `tip` is equivocation (→ DMTAP-PUB-15), never a rollback | vectors `pub_feed_rollback_strict_less_than` (reject) + `pub_feed_equal_seq_identical_tip_idempotent` (accept) | reject → `ERR_PUB_FEED_ROLLBACK` (0x0907), FAIL_CLOSED_BLOCK (strict-less-than) / accept (equal+identical) | vectored |
| DMTAP-PUB-15 | MUST | §22.4.2 | **Feed hash-chain integrity (fork):** two `FeedEntry`s at one `seq` with the same `prev` (or a `prev` not resolving to `seq-1`) is evidence of a rewritten/equivocated author log — halted on, same posture as a committer fork (0x0404) / cluster-journal break (0x0412) | vector `pub_feed_equal_seq_different_tip_fork` (two distinct entries at `seq=1`, same `prev`) | reject → `ERR_PUB_FEED_CHAIN_BROKEN` (0x0908), HALT_ALERT | vectored |
| DMTAP-PUB-16 | MUST | §22.4.1 | **Feed genesis rule:** a genesis entry (`seq=0`) carrying a `prev` field, or a non-genesis entry (`seq≠0`) lacking one, is malformed | vectors `pub_feed_genesis_carries_prev_malformed` + `pub_feed_nongenesis_missing_prev_malformed` | reject → `ERR_PUB_FEED_CHAIN_BROKEN` (0x0908), HALT_ALERT | vectored |
| DMTAP-PUB-17 | MUST | §22.4.1 | **Feed head signature:** `FeedHead.sig` failing under the `signer`/`pub` chain MUST be rejected | construction: flip one bit of `pub_feed_head_signing_preimage`'s `sig_hex` and re-verify | reject → `ERR_PUB_FEED_SIG_INVALID` (0x0906), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-PUB-18 | MUST | §22.3.1, §22.4.1 | **Unknown PUB version/suite:** a `PubAnnounce`/`PubManifest`/`FeedHead` carrying a `v`/`suite` the implementation does not support MUST be rejected, never guessed | construction: `PubAnnounce` with `v=1` (any value ≠ 0) | reject → `ERR_PUB_UNSUPPORTED_VERSION` (0x0901), FAIL_CLOSED_BLOCK | construction-todo |
| DMTAP-PUB-19 | MUST | §22.6.2 | **Serve refusal is policy, not fault:** a holder declining to serve a requested public object per its own serve policy is a policy deny at the holder, never a correctness error; the fetcher rotates to another holder | construction: holder policy configured to decline a given `announce_id`/`manifest_root` | reject (at holder) → `ERR_PUB_NOT_SERVED` (0x090C), DENY_POLICY; fetcher ROTATE_RETRY | construction-todo |
| DMTAP-PUB-20 | MUST | §22.6.3 | **Serving resource limit:** exceeding a serving node's admission policy (object size / per-publisher quota / feed-append rate) is a policy deny, never a security/crypto gate | construction: publish/append past a configured per-publisher storage quota | reject → `ERR_PUB_SERVE_QUOTA` (0x090D), DENY_POLICY | construction-todo |
| DMTAP-PUB-21 | MUST | §22.7 | **Publish is explicit + irrevocable (client UX, no wire bytes):** a client MUST NOT create/announce/serve a public object except as the direct result of an explicit user publish act; MUST warn, before completing a publish, that publishing is irrevocable; MUST express retraction only as a successor `supersedes` announcement, never as deletion | manual attestation (implementer/UX review — see "How to read a case" `status: manual-attestation`) | non-conformant if any of the three sub-requirements is missing | manual-attestation |

---

## CAD/Artifact profile (§23) — `CAD`

An **application profile** over DMTAP-PUB, not a new wire mechanism (§23.1): it allocates no
message kind, capability token, DS-tag, or error block. Conformance to this profile is therefore
**additive and orthogonal** to §22/§21 conformance — a node can be Core/`pub-1`-conformant without
ever having heard of this document, and a CAD-aware client is `pub-1`-conformant-by-construction
because it is only ever a consumer/producer of ordinary §22 objects (`ArtifactMetadata` rides
inside an already-signed `pub_announce.meta["artifact"]`, §23.3.1). The 11 checks below are the
**§23.10 conformance checklist**, one case per row, in the checklist's own order. None allocates a
§21 error code (the profile has none); "reject" below means a CAD-aware client/index MUST refuse
to treat the artifact as usable/well-formed, not that a §22/§21 wire error is raised — a
non-CAD-aware §22 node stores and serves the same bytes unaffected (§23.2).

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-CAD-01 | MUST | §23.4, §23.10 CAD-1 | every artifact `pub_announce`'s `ArtifactMetadata` carries a `license` field (SPDX expression); an announce omitting it is malformed for this profile | construction: `ArtifactMetadata` with key 7 (`license`) omitted | reject (profile-level; generic §22 node still stores/serves it) | construction-todo |
| DMTAP-CAD-02 | MUST | §23.3.4, §23.10 CAD-2 | `formats` contains **at least one** entry | construction: `ArtifactMetadata` with `formats` (key 4) an empty array | reject | construction-todo |
| DMTAP-CAD-03 | MUST | §23.3.4, §23.10 CAD-3 | exactly one `formats` entry carries `role = canonical-source` (non-`assembly` kinds) or `role = structure` (`assembly` kind); an `assembly` with no `structure` entry is malformed | construction: two `role=1` entries (ambiguous canonical source); and separately, an `assembly`-kind with no `role=3` entry | reject (both variants) | construction-todo |
| DMTAP-CAD-04 | MUST | §23.3.4, §23.10 CAD-4 | no `format_id = 3` (glTF/mesh) entry ever carries `role = 1` (canonical-source) — the profile's central integrity guarantee: a lossy tessellation is never the artifact of record | construction: `ArtifactFormat{format_id:3, role:1, manifest_root:<any hash>}` | reject | construction-todo |
| DMTAP-CAD-05 | MUST | §23.3.4, §23.10 CAD-5 | every `role = 2` (derived-rendition) entry carries `derived_from_format` (key 4) | construction: `ArtifactFormat{format_id:1, role:2}` with `derived_from_format` omitted | reject | construction-todo |
| DMTAP-CAD-06 | MUST | §23.3.3, §23.10 CAD-6 | `units.length_unit` is present and explicit; a client MUST NOT default or infer it | construction: `Units` with key 1 (`length_unit`) omitted | reject (client MUST refuse to interpret geometry; MAY still show name/description/license) | construction-todo |
| DMTAP-CAD-07 | MUST | §23.3.1, §23.10 CAD-7 | `deprecated = true` is always accompanied by `deprecation_reason` (key 9) | construction: `ArtifactMetadata{deprecated: true}` with key 9 absent | reject (flagged by a CAD-aware index) | construction-todo |
| DMTAP-CAD-08 | MUST | §23.5, §23.10 CAD-8 | deprecation/yank is expressed **only** as a successor announcement (`supersedes` + `deprecated=true`), never as a deletion — no operation removes a previously published revision | construction: attempt to model "deletion" of a prior revision (no protocol operation exists; a CAD-aware client MUST NOT present one) | reject (no-such-operation; client MUST NOT imply deletion) | construction-todo |
| DMTAP-CAD-09 | MUST | §23.6.1, §23.10 CAD-9 | assembly children reference exclusively by `pin` (`ref_kind=1`, a `manifest_root`) or `track` (`ref_kind=2`, a `pub_announce` id) | construction: `AssemblyChild.ref_kind` outside `{1,2}` | reject | construction-todo |
| DMTAP-CAD-10 | MUST | §23.6.3, §23.10 CAD-10 | a BOM-walking client MUST detect and reject a cycle in an assembly's resolved DAG (a `track` reference can form one across revisions) rather than recurse indefinitely or silently drop it | construction: assembly A `track`s part B; B's publisher later republishes B as an assembly that `track`s back to A; walk A's BOM | reject (abort the walk at the cycle; surface it to the user — never infinite-recurse, never silently drop) | construction-todo |
| DMTAP-CAD-11 | MUST | §23.7, §23.10 CAD-11 | no client treats any single index (category/search/workshop) as authoritative over the signed announces/feeds it was derived from | construction: two independently-built indexes over the same feed set disagree (different crawl coverage) | accept (neither index is "wrong"; ground truth is always the signed announces, re-derivable by any client) | construction-todo |

---

## Video/Media profile (§24) — `VIDEO`

An **application profile** over DMTAP-PUB (§24.1), the convergence path for the *vidmesh* protocol.
Like `CAD`, it is **additive and orthogonal** to §22/§21 conformance: a node can be Core/`pub-1`-conformant
without parsing any of it, and a video-aware client is `pub-1`-conformant-by-construction because it only
ever produces/consumes ordinary §22 objects (its metadata rides inside an already-signed
`pub_announce.meta[<key>]`, §24.4.1). The 15 checks below are the **§24.15 conformance checklist**, one
case per row, in checklist order. The profile allocates **no §21 error code** (so "reject" means a
video-aware client MUST refuse to treat the object as usable/well-formed — a non-video §22 node stores and
serves the same bytes unaffected). **The one exception is VID-3/VID-5**, the rendition-derivation statement
(§24.4.4): it *does* have a signable preimage (DS-tag `"DMTAP-VID-v0/derivation"`), so those two become
byte-backed KATs once a fixed-input derivation vector is generated — until then they carry a construction
recipe like the rest.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-VIDEO-01 | MUST | §24.11, §24.15 VID-1 | every `VideoManifest` carries a `license` field (SPDX expression or a profile consent token `all-rights-reserved`/`mirror-freely`/`endorsed-only`) | construction: `VideoManifest` with key 9 (`license`) omitted | reject (profile-level; generic §22 node still stores/serves it) | construction-todo |
| DMTAP-VIDEO-02 | MUST | §24.4.3, §24.15 VID-2 | `original` (key 5) is present and is the canonical rendition — a `Rendition` is never the artifact of record | construction: `VideoManifest` with `original` (key 5) omitted; and separately, a client treating a `Rendition.blob` as the source of truth | reject (both variants) | construction-todo |
| DMTAP-VIDEO-03 | MUST | §24.4.3, §24.4.4, §24.15 VID-3 | every `Rendition` carries `produced_by` (key 8) and a `derivation_sig` (key 9) that verifies over the derivation statement | construction: `Rendition` with `derivation_sig` omitted; and separately, a `derivation_sig` that fails to verify over the reconstructed statement | reject (not an authorized rendition; MAY be shown labeled as unverified, or dropped) | construction-todo |
| DMTAP-VIDEO-04 | MUST | §24.4.4, §24.4.6, §24.15 VID-4 | a rendition is treated as *authorized* only if `produced_by` is the manifest author or holds an unrevoked, unexpired `rendition` delegate grant from the author | construction: a validly-signed `Rendition` whose `produced_by` is neither the author nor a valid delegate | reject-as-authorized (present only as a labeled third-party encoding, never as an equivalent authorized rendition) | construction-todo |
| DMTAP-VIDEO-05 | MUST | §24.4.4, §24.15 VID-5 | the derivation statement binds `derived_from`→`rendition.blob` + codec/width/height/bitrate, signed under `"DMTAP-VID-v0/derivation"` by a device key chaining to `produced_by` | construction (signable KAT recipe): `stmt = det_cbor([derived_from, rendition.blob, codec, width, height, bitrate])`; `sig = Sign(dev_key, "DMTAP-VID-v0/derivation" ‖ 0x00 ‖ BLAKE3-256(stmt))`; mutate any tuple element ⇒ signature MUST fail | reject (replayed/mismatched statement); accept (matching) | construction-todo |
| DMTAP-VIDEO-06 | MUST | §24.5, §24.15 VID-6 | a video's `channel` (key 10) reference resolves to an announce with the **same `pub`** as the video's author (a video cannot join another identity's channel) | construction: `VideoManifest.channel` naming a `Channel` announce with a different `pub` | reject | construction-todo |
| DMTAP-VIDEO-07 | MUST | §24.4.1, §24.15 VID-7 | `retracted = true` (key 12) is always accompanied by `retract_reason` (key 13) | construction: `VideoManifest{retracted: true}` with key 13 absent | reject (malformed for this profile) | construction-todo |
| DMTAP-VIDEO-08 | MUST | §24.7, §24.15 VID-8 | retraction/removal is expressed **only** as a successor `supersedes` announcement (`retracted=true`), never as deletion; a client MUST NOT imply deletion of prior bytes | construction: attempt to model "deletion" of a prior revision (no protocol operation exists) | reject (no-such-operation; retracted bytes remain fetchable, only status changes) | construction-todo |
| DMTAP-VIDEO-09 | MUST | §24.6.1, §24.15 VID-9 | a threaded `Comment` with a `parent` (key 3) references the **same `subject`** (key 1) as its parent | construction: `Comment` whose `parent` references a comment with a different `subject` | reject (parent-subject mismatch) | construction-todo |
| DMTAP-VIDEO-10 | MUST | §24.6.2, §24.15 VID-10 | a `Reaction` is counted at most once per identity per subject — a later same-author reaction `supersedes` the earlier | construction: two reactions by one identity to one subject, the later `supersedes` the earlier; an index counts both | reject (count the current one only; superseded reactions are discarded) | construction-todo |
| DMTAP-VIDEO-11 | MUST | §24.9, §24.15 VID-11 | segmented playback verifies segment/range bytes against the signed rendition's Merkle root; the HLS/DASH playlist is unsigned serving output, never authoritative | construction: a PUB-server-supplied `.m3u8` listing a segment whose bytes do not verify against the rendition root; client accepts on the playlist's say-so | reject (verify against the signed root; the playlist is not an object of record) | construction-todo |
| DMTAP-VIDEO-12 | MUST | §24.10, §24.15 VID-12 | live streaming is gated behind the `vid-live-1` capability; a consumer lacking it treats its absence as a fact, not a fault (capability-absence rule §21.22) | construction: a peer that has not advertised `vid-live-1` receives a `LiveManifest`; treats non-support as a parse failure | reject-the-error (absence is unsupported-not-fatal; store the announce, do not low-latency-follow) | construction-todo |
| DMTAP-VIDEO-13 | MUST | §24.8, §24.15 VID-13 | view/reaction/trending aggregates are presented as **per-server claims**, never as network-wide truth | construction: a client renders a PUB server's view count as an authoritative global number | reject (label as "views on this server"; a signed tally is worth only the attester's reputation) | construction-todo |
| DMTAP-VIDEO-14 | MUST | §24.8, §24.13, §24.15 VID-14 | no client treats any single index (search/recommendation/compliance) as authoritative over the signed announces/feeds it was derived from | construction: two independently-built indexes (or compliance feeds) over the same feed set disagree | accept (neither is "wrong"; ground truth is the signed feeds, re-derivable by any client) | construction-todo |
| DMTAP-VIDEO-15 | MUST | §24.13, §24.15 VID-15 | encrypted-media fields (`keygrant`/`encryption`) do not appear in a public `VideoManifest` — the public profile makes no confidentiality claim | construction: a `VideoManifest` carrying an `encryption`/`keygrant` field | reject (category error; encrypted media belongs to the sealed path §5.5, not this public profile) | construction-todo |

---

## Gateway operator prerequisites, discovery & modes (§7.1a, §7.5, §7.9, §7.12.1, §7.15.4) — `GWOPS`

Level **Legacy**. §7.1a states plainly what the gateway role requires, because an unstated
requirement always resolves into "ask the vendor": a static IP with forward-confirmed PTR,
unblocked outbound port 25, and a domain — and **not** a payment method. The cases below pin the
parts of that posture an implementation can get wrong: implying a charge the protocol never
requires, publishing a reputation score that would recreate the directory authority §4.4.2
deleted, assuming a registration mode a gateway never advertised, and declaring one legacy-access
service mode while operating another. `-02`, `-06` and `-07` are `manual-attestation`: product
copy, the population a deployment actually serves, and an operator's usage claim have no wire
bytes to recompute.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-GWOPS-01 | MUST | §7.1a | **Forward-confirmed reverse DNS and a matching HELO.** A gateway MUST publish a PTR for its sending address that forward-resolves back to the same host, and MUST use a HELO/EHLO name matching it. This is not hygiene advice: receiving MTAs check FCrDNS before content is ever considered, so a gateway that fails it is rejected or spam-scored on every leg, and the bridge silently stops bridging | construction: resolve the gateway's sending address → PTR → A/AAAA and compare; open an SMTP session to the gateway's own MX and capture the HELO/EHLO name it presents on egress | accept (PTR forward-resolves to the same host **and** the egress HELO/EHLO equals that name); either half missing is non-conformant | construction-todo |
| DMTAP-GWOPS-02 | MUST | §7.1a, §12.3, §7.14 | **A payment method is not a requirement, and an implementation MUST NOT imply one.** Nothing in DMTAP requires a gateway operator to charge or a user to pay. An install, onboarding or gateway-enablement flow that presents billing details as a prerequisite for running or using the role — or that gates identity, keys, native mesh delivery, the mixnet or recovery behind one — asserts a requirement the protocol does not have | manual attestation (review of the gateway-enablement and onboarding flows, install documentation and the operator's own copy; no wire bytes to recompute) | non-conformant if a payment method is presented as a prerequisite for the role, or if any §12.3.1 never-chargeable function is reachable only after billing details are supplied | manual-attestation |
| DMTAP-GWOPS-03 | MUST | §7.5, §9.6 | **Reputation is locally measured, never a globally published score.** A gateway descriptor is discovery-only and self-asserted: it carries no reputation field, no price and no stake. Each sending node derives its own view from its **own** deliverability results, and a measurement shared by a peer MAY be an input but MUST NOT be treated as authoritative. A global score needs a party to compute it, and that party then decides which gateways receive traffic — the directory-authority problem §4.4.2 removed from the mixnet, reappearing one layer up | construction: node A has measured gateway G as delivering successfully to its own destinations; a peer supplies a signed "G is bad" measurement; separately, a descriptor for G is presented carrying a `reputation`/`price`/`stake` field | accept (A's routing decision remains a function of A's own results — the peer's claim is an input it MAY weigh and MUST NOT be bound by; the extra descriptor fields are ignored, never ranked on); a node that stops routing to G on the peer's say-so alone, or that orders gateways by a published score, is non-conformant | construction-todo |
| DMTAP-GWOPS-04 | MUST | §7.12.1, §7.12 | **A client MUST NOT assume a registration mode.** A gateway advertises `open`, `key-registered`, or both, in its descriptor; a client reads the advertised set and, for `key-registered`, runs the §7.12.2 handshake first. Assuming `open` against a `key-registered` gateway is the client-side half of the open-relay floor | construction: gateway advertising `key-registered` only; a client submits an outbound legacy relay without having run §7.12.2, and a second client that advertises no mode at all is asked to relay | reject → `ERR_GATEWAY_SENDER_UNAUTHENTICATED` (0x0607), FAIL_CLOSED_BLOCK; a gateway advertising neither mode performs no third-party relay at all | construction-todo |
| DMTAP-GWOPS-05 | MUST | §7.15.4, §7.5 | **Exactly one legacy-access service mode is declared.** A gateway operator MUST declare exactly one of `public` / `registered-clients-only` / `private` in its directory descriptor. The mode is what tells a user whether a third party can read the mail it serves (§7.15.3), so a descriptor declaring two modes, or none, is not a richer offer — it is an undisclosed trust boundary | construction: directory descriptors declaring (a) both `public` and `private`, (b) no `operator_mode` at all | reject (the descriptor is unusable for legacy-access selection; the client MUST NOT infer a mode); treating an absent mode as `public`, or as `private`, is non-conformant | construction-todo |
| DMTAP-GWOPS-06 | MUST | §7.15.4, §7.15.3 | **The declared mode MUST be enforced.** Advertising `private` while serving third parties, or `registered-clients-only` while accepting open registration, is misrepresentation rather than a policy choice: the mode is the user's only statement of who can read their mail, and a mode that is not enforced makes the disclosure of §7.15.3 false | manual attestation (deployment review: the declared `operator_mode` in the descriptor against the population the legacy-access surface actually serves — registration policy, provisioned app-passwords, distinct identities with live sessions) | non-conformant if any identity outside the declared mode's population holds legacy access, in particular any second identity on a gateway declaring `private` | manual-attestation |
| DMTAP-GWOPS-07 | MUST | §7.9, §7.8, §18.8.1 | **A pure-mesh message MUST NOT appear on any operator's usage claim.** Every gateway-touched message carries a verifiable §7.2a attestation naming the gateway `domain` and receipt time, so a user can confirm each claimed legacy operation against the message's own `ProvenanceRecord`. A message the client shows as pure-mesh had no operator on its path, so no operator was positioned to observe it — this is the user's evidence, not the operator's accounting mechanism | manual attestation (audit of an operator usage claim against the claimed messages' own `ProvenanceRecord`s: every claimed operation must correspond to a MOTE carrying a valid `GatewayAttestation` for that gateway `domain`) | non-conformant if any claimed operation references a MOTE the client renders pure-mesh, or one whose provenance names a different gateway | manual-attestation |

---

## SMTP legs — TLS, 8-bit/EAI, DSNs and response codes (§7.2, §7.2b, §7.3, §7.10.3a, §21.9) — `GWSMTP`

Level **Legacy**. The bridge's two SMTP legs are where DMTAP touches bytes it did not
create, and every rule here exists because the failure mode is silent: a downgraded leg is
readable in transit while the client still shows a lock, a lossy re-encode corrupts exactly what
the bridge exists to carry, an uncorrelated `MAIL FROM:<>` is the classic backscatter vector, and
a `250` returned before a durable `ack` opens a window in which the message is lost after the
legacy sender has already been told it was delivered. `-01`/`-02` and `-06` use the two codes this
wave registers, `0x0608` and `0x0609` (§21.8).

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-GWSMTP-01 | MUST | §7.2 | **Inbound TLS: offer STARTTLS, never silently downgrade the advertised posture.** The gateway MX MUST offer STARTTLS (RFC 3207) on its SMTP listener and MUST NOT silently downgrade its own advertised TLS posture: a session that negotiates cleartext where the gateway's published policy promised TLS is a policy violation, not a fallback | construction: (a) EHLO the gateway MX and check for `STARTTLS`; (b) with an MTA-STS `enforce` policy published for the gateway's own MX hostname, drive a session that declines STARTTLS | (a) accept (`STARTTLS` advertised); (b) reject → `ERR_GATEWAY_TLS_POLICY_UNMET` (0x0608), FAIL_CLOSED_BLOCK — proceeding in cleartext while a published policy promises TLS is non-conformant | construction-todo |
| DMTAP-GWSMTP-02 | MUST | §7.3 | **Outbound TLS: enforce a published policy, and never present an unprotected leg as protected.** Where the destination domain publishes MTA-STS `enforce` (RFC 8461) or DANE TLSA (RFC 7672), the gateway MUST enforce TLS to that policy with no downgrade to cleartext or to an unvalidated peer. Where neither is published it MUST attempt opportunistic STARTTLS, and a leg delivered opportunistically or in cleartext MUST be **recorded as such** — a gateway MUST NOT present an opportunistic or cleartext leg as a protected one | construction: destination publishing MTA-STS `enforce`, whose MX then offers no STARTTLS (variant: presents a certificate the policy does not validate); separately, a destination publishing neither, delivered opportunistically | reject → `ERR_GATEWAY_TLS_POLICY_UNMET` (0x0608), FAIL_CLOSED_BLOCK for the policy-bearing destination; accept for the opportunistic destination **only if** the leg is recorded as opportunistic/cleartext — recording it as TLS-enforced is non-conformant | construction-todo |
| DMTAP-GWSMTP-03 | MUST | §7.2b | **8-bit transparency, byte-exact.** A gateway MUST advertise **8BITMIME** (RFC 6152) and MUST carry 8-bit message data byte-exact through verification and wrapping — DKIM is computed over the original bytes, and the bytes wrapped into the MOTE are the bytes received. Lossy re-encoding is forbidden: a body that cannot be transcoded to UTF-8 MUST be carried verbatim as opaque bytes under its original MIME type with its original charset declaration preserved | construction: inbound message with an ISO-2022-JP body and a byte sequence that does not transcode cleanly to UTF-8; capture the exact `DATA` octets and compare against the body bytes the wrapped MOTE carries | accept (`8BITMIME` advertised; wrapped body bytes byte-identical to the received `DATA` octets; MIME type and `charset` parameter preserved); any re-encoding, substitution character or normalization is non-conformant | construction-todo |
| DMTAP-GWSMTP-04 | MUST | §7.2b | **Header encoding round-trips.** RFC 2047 encoded-words MUST be decoded to UTF-8 for the native subject/display fields of the wrapped MOTE, and non-ASCII header values MUST be (re-)encoded per RFC 2047 on the outbound leg. The decode and the re-encode are a pair: a bridge that decodes inbound but emits raw UTF-8 outbound produces headers a legacy MUA cannot read | construction: inbound `Subject:` as an RFC 2047 encoded-word (`=?UTF-8?B?…?=`) over a non-ASCII string; relay the resulting MOTE back out to a legacy destination and capture the emitted header | match (the MOTE's native subject field is the decoded UTF-8 string; the outbound `Subject:` is a valid RFC 2047 encoded-word decoding to the same string) | construction-todo |
| DMTAP-GWSMTP-05 | MUST | §7.2b, §3.9.7 | **Internationalized domains go on the wire as A-labels.** Domains MUST be converted to A-label form (RFC 5890) for DNS resolution, dialing and SNI; U-labels are display forms, never wire forms. A U-label reaching the wire is both an interoperability failure and a homograph surface, since the comparison that would catch the spoof (§3.9.7) is defined on the canonical form | construction: relay to a recipient at an IDN domain; capture the name used for the MX lookup, the TLS SNI value and the SMTP envelope | accept (A-label in all three positions); a U-label in any wire position is non-conformant | construction-todo |
| DMTAP-GWSMTP-06 | MUST | §7.2b | **SMTPUTF8: fail cleanly, never accept-then-mangle.** A gateway that does not advertise SMTPUTF8 (RFC 6531) MUST let a conforming EAI sender fail at its own MTA, and MUST NOT accept EAI envelopes it cannot carry faithfully. Outbound, when a message requires SMTPUTF8 and the destination MX does not advertise it, the gateway MUST fail the send permanently and specifically, surfaced to the sender via the §7.3/§7.4 failure report, and MUST NOT emit a non-conformant 8-bit envelope; where the body is 8-bit and the peer lacks 8BITMIME it MUST either down-convert **losslessly** or fail with the same specificity | construction: (a) gateway not advertising SMTPUTF8 receives `RCPT TO:<用户@例子.测试>`; (b) a message requiring SMTPUTF8 relayed to an MX advertising neither SMTPUTF8 nor 8BITMIME, with no lossless down-conversion available | reject → `ERR_GATEWAY_SMTPUTF8_UNSUPPORTED` (0x0609), REJECT_NOTIFY — permanent for this message and reported to the sender; accepting the envelope and down-converting lossily, or emitting a non-conformant 8-bit envelope, is non-conformant | construction-todo |
| DMTAP-GWSMTP-07 | MUST | §7.10.3a, §7.11.1 | **DSNs are exempted only when correlated.** A DSN/NDR (RFC 3464; envelope `MAIL FROM:<>`) inbound to a gateway alias MUST be recognized as such and exempted from the SPF/DMARC hard-fail and cold-sender gates of §7.11.1 **provided** the gateway can correlate it — via `Original-Recipient` and/or the referenced `Message-ID` — to an outbound message this gateway relayed for that identity inside the node's retry window (§7.4). A null-return-path message it cannot correlate is gated like any other inbound: an uncorrelated `MAIL FROM:<>` is the classic backscatter/spoof vector | construction: (a) a DSN whose `Original-Recipient`/`Message-ID` matches an outbound relay this gateway performed inside §7.4's window; (b) a byte-identical DSN referencing a message this gateway never relayed | (a) accept (delivered to the native sender as a system/bounce MOTE, gates exempted); (b) reject → the ordinary cold-sender gate applies, `ERR_CHALLENGE_MISSING_COLD_SENDER` (0x0701), DEFER_REQUESTS — exempting an uncorrelated null-return-path message is non-conformant | construction-todo |
| DMTAP-GWSMTP-08 | MUST | §7.10.3a, §7.4 | **A legacy send never fails silently.** When the node's outbound retry budget (§7.4) exhausts, the node MUST surface a permanent-failure notice to the sender. The gateway holds no queue, so if the node treats exhaustion as a quiet terminal state the user's message has disappeared with no signal anywhere in the system | construction: relay to a destination MX that `4xx`s every attempt; advance past the §7.4 retry budget and inspect the sender-visible outcome | accept (a permanent-failure notice reaches the sender at exhaustion); silently dropping the send, or leaving it displayed as in-flight past the deadline, is non-conformant | construction-todo |
| DMTAP-GWSMTP-09 | MUST | §21.9, §19.7.1, §10.7.4, §7.4 | **No `250` before a durable `ack`.** The gateway's inbound leg is an ordinary SMTP transaction and MUST respond with RFC 5321/RFC 3463 codes. Where the recipient is reachable but has not durably `ack`ed inside the transaction window — a best-effort buffer accepted the packet, or nothing did — the gateway MUST reply `451 4.4.1` and MUST NOT reply `250` on mere hand-off. Replying `250` closes the SMTP transaction and moves durability out of the legacy sender's queue, so a later mesh-side `EXPIRED` loses the message with nobody left to notify | construction: inbound SMTP for a recipient whose node accepts into a best-effort peer buffer but emits no durable `ack` before the transaction window closes | accept (`451 4.4.1`, deferring to the legacy sender's queue); any `2xx` before a durable `ack` is non-conformant | construction-todo |
| DMTAP-GWSMTP-10 | MUST | §21.9, §9.2 | **A block is indistinguishable from a non-existent address.** A recipient that declines via `Policy.block` (§9.2) MUST be answered with the **identical** code and enhanced status as "no such user" — `550 5.1.1`. A distinct `5.7.x` would itself reveal that the recipient exists and has blocked this sender, turning the SMTP reply into a block-membership oracle; the block is enforced downstream and never surfaced as its own signal | construction: two inbound SMTP transactions from the same sender — one to an address that does not exist at the gateway, one to an existing recipient whose `Policy.block` names that sender; compare the replies octet for octet | accept (both replies are `550 5.1.1`, byte-identical including any text); any divergence — code, enhanced status, or wording — is a block-membership oracle and is non-conformant | construction-todo |

---

## Gateway attestation binding & chaining (§7.2a, §7.8.3, §18.3.11, §21.24a) — `GWATT`

Level **Legacy**. An attestation is worthless unless its signing key is provably bound to a
gateway the domain actually authorized — otherwise any operator forges "legitimate legacy origin"
for a domain it does not serve. The binding is a `_dmtap-gw` DNS record, so these cases pin both
halves: that the recipient checks the key under its **own** domain and that the attestation is
bound to **this one message**, and that the honesty of the anchor is not overstated. `-03` is
`manual-attestation`: how strong an assurance a client presents is a UX property with no wire bytes.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-GWATT-01 | MUST | §7.2a, §18.3.11 | **The attestation key is looked up under the recipient's own domain.** A recipient node MUST verify an inbound MOTE's attestation signature against a key published under the recipient's own domain's `_dmtap-gw` record (or an explicitly trusted gateway set), MUST reject attestations that do not verify, and MUST mark accepted ones as *legacy-origin*. A key the gateway publishes under its **own** domain proves only that the gateway signed — the question is whether the recipient's domain authorized it | construction: inbound MOTE whose `GatewayAttestation.domain`/`selector` resolve to a validly-signing key published under the **gateway's** domain, with no `_dmtap-gw` record under the recipient's domain and the gateway absent from the recipient's trusted set | reject → `ERR_GATEWAY_ATTESTATION_KEY_UNTRUSTED` (0x0602), DROP_SILENT; accepting on the strength of a self-published key is non-conformant, and an accepted attestation MUST mark the message legacy-origin rather than end-to-end | construction-todo |
| DMTAP-GWATT-02 | MUST | §7.2a, §18.3.11, §18.9.11 | **`msg_digest` binds the attestation to one message.** `msg_digest` is `0x1e ‖ BLAKE3-256(rfc5322_bytes)` over the exact legacy bytes the gateway wrapped. The recipient MUST recompute it from the decrypted body and reject a mismatch, so a valid attestation cannot be lifted from the message it was issued for and re-presented over different content | construction: take a MOTE carrying a valid `GatewayAttestation`; replace the wrapped RFC 5322 body with different bytes, leaving `domain`, `selector`, `recv_at` and `sig` untouched | reject → `ERR_GATEWAY_ATTESTATION_INVALID` (0x0601), DROP_SILENT; verifying the signature without recomputing `msg_digest` from the decrypted body is non-conformant | construction-todo |
| DMTAP-GWATT-03 | MUST | §7.2a, §13.7 | **Anchor honesty.** The `_dmtap-gw` record is a DNS binding, so absent DNSSEC **and** a KT anchor the attestation inherits the DNS-substitution risk of §13.7 item 6 — a registrar or DNS compromise substitutes the attestation key. A client MUST NOT present such an attestation to users as a stronger assurance than DKIM-class domain authentication | manual attestation (client-UX review with an attestation whose `_dmtap-gw` record is neither DNSSEC-signed nor KT-anchored; no wire bytes to recompute) | non-conformant if the message is presented as verified-origin at an assurance above DKIM-class domain authentication, or if the difference between an anchored and an unanchored attestation is not surfaced | manual-attestation |
| DMTAP-GWATT-04 | MUST | §7.2a | **High-value recipients require the KT-anchored form.** A recipient operating at high value MUST require the KT-anchored `_dmtap-gw` binding rather than the bare DNS one; at that assurance level a registrar compromise is inside the threat model, and the KT anchor is what makes a substituted attestation key detectable | construction: recipient configured high-value; inbound MOTE whose attestation key is published in DNS only, with no KT entry for the `_dmtap-gw` binding | reject → `ERR_GATEWAY_ATTESTATION_KEY_UNTRUSTED` (0x0602), DROP_SILENT; accepting the DNS-only binding at the high-value setting is non-conformant | construction-todo |
| DMTAP-GWATT-05 | MUST | §7.8.3, §18.3.11 | **A chain verifies entry by entry, and the bridging entry under the recipient's own domain.** Where more than one gateway bridged a message, `Payload.provenance` carries an ordered chain of `GatewayAttestation`s. Each entry is verified against the `_dmtap-gw` key published under **its own** `domain`; the entry that bridged mail for the recipient MUST verify under the recipient's own domain, and entries under other domains verify only if that domain is in the recipient's explicitly-trusted set — otherwise they are surfaced as an *unverified hop*, never silently accepted or silently dropped | construction: a two-entry provenance chain in which `seq=1` is under an untrusted third-party domain and `seq=0` (the entry that bridged for the recipient) verifies under the recipient's own domain | accept (message accepted as legacy-origin on the strength of `seq=0`; `seq=1` rendered as an unverified hop); a variant in which the recipient-facing entry verifies only under some other domain rejects → `ERR_GATEWAY_ATTESTATION_KEY_UNTRUSTED` (0x0602) | construction-todo |
| DMTAP-GWATT-06 | MUST | §21.24a, §18.3.11 | **An unknown attestation discriminator is unverifiable, never ignorable.** An unknown `GatewayAttestation.disc` value MUST be treated as an unverifiable attestation, never silently ignored: an attestation whose kind a verifier cannot check is not a pass. This is the fail-closed analogue of the unknown-suite rule (§1.1) applied to provenance — silently ignoring it would let a forger downgrade a message to "no attestation present", which §18.3.5 makes a claim in itself | construction: inbound MOTE carrying a `GatewayAttestation` with `disc = 0x7F` (unassigned) and an otherwise well-formed body | reject → `ERR_GATEWAY_ATTESTATION_INVALID` (0x0601), DROP_SILENT; ignoring the entry and surfacing the message as if it carried no provenance is non-conformant | construction-todo |

---

## Alias minting, parsing & safety tiers (§7.10.1, §7.10.5, §7.10.5a, §7.10.6) — `GWNAME`

Level **Legacy**. `GWALIAS` above tests that an alias *maps*; this family tests what a gateway
is allowed to *mint* and how a client is allowed to *present* it. The rule is one sentence — a
gateway can only alias what already exists and MUST NOT mint a global name — and §7.10.5a makes it
decidable from the local-part alone: a dot means the name belongs to another namespace, its
absence means it is a vanity in the gateway's own. The client half matters as much: a product that
lets a user hand out a tier-3 alias as "their email address" has re-created provider lock-in
through the UI having avoided it in the protocol, which is why `-05` and `-06` are
`manual-attestation` — a share sheet has no wire bytes.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-GWNAME-01 | MUST | §7.10.1 | **Native → legacy rewrites the reply path.** Legacy mail cannot route a reply to a non-MX native address: a legacy MTA replying to `imran@mydomain.com` looks up that domain's MX, finds none, and bounces. So the machine-readable return/reply path (envelope-from and `Reply-To:`) MUST be the gateway alias, which does have an MX; the display MAY still show the friendly native address as an RFC 5322 display name | construction: native user sends to a legacy recipient; capture the emitted envelope-from and `Reply-To:`, then have the legacy recipient reply and follow the route | accept (envelope-from and `Reply-To:` are the gateway alias; the reply reaches the gateway, is mapped back per §7.10.3 and delivered as a MOTE); emitting the bare native address in either machine-readable position is non-conformant | construction-todo |
| DMTAP-GWNAME-02 | MUST | §7.10.5, §3.13.1 | **A gateway MUST NOT mint a global name.** A vanity is a user-chosen local-part in the gateway's own domain: it MUST be dot-free and MUST be valid **only** fully-qualified (`vanity@gatewaydomain`), never as a bare handle. Allocating a globally-unique name from a bare string is the flat-namespace consensus problem DMTAP deliberately does not solve; the key-name (§3.9.6) is the floor instead, and a gateway handing out a bare handle claims an authority it does not have | construction: request the vanity registrations `al.ice` (dotted) and `alice` presented for use as a bare handle with no `@gatewaydomain` suffix | reject (both: a dotted local-part is reserved for foreign namespaces per §7.10.5a, and a bare handle has no anchor); issuing either is non-conformant, and the identity's key-name remains available regardless | construction-todo |
| DMTAP-GWNAME-03 | MUST | §7.10.5 | **A vanity yields to, and never shadows, a real network name.** If a resolvable `name@domain` or name-chain name exists, the anchored name wins; a gateway MUST NOT let a chosen vanity mask or intercept delivery to it. A vanity is first-come and revocable in the gateway's own namespace — the one alias form with ownership semantics — and that ownership stops exactly where an anchor begins | construction: gateway domain `gw.example` also hosts a resolvable native identity `alice@gw.example`; a different user then registers, or already holds, the vanity `alice` there; deliver inbound legacy mail to `alice@gw.example` | accept (delivered to the anchored identity); delivery to the vanity holder is non-conformant, and the vanity registration MUST NOT be issued or MUST yield | construction-todo |
| DMTAP-GWNAME-04 | MUST | §7.10.5a, §21.18 | **The local-part parsing rule is decidable and closed.** A dot means the name is not the gateway's: it MUST be resolved in the namespace it belongs to — through a registered name-chain resolver (§21.18) or by decoding the §7.10.2 packed form — and MUST NOT be treated as a registration in the gateway's own namespace, allocated, sold or squatted. Failure is closed and specific, and the gateway MUST NOT guess which form was meant, nor fall back from a failed chain resolution to a vanity lookup — that fallback lets a squatted vanity intercept mail for a chain name | construction: inbound mail to `alice.sol@gw.example` where the `.sol` chain lookup fails **and** a local vanity `alice.sol` (or `alice`) exists; variant: `imran.mydomain-.com@gw.example` whose encoded form does not decode | reject → `ERR_GATEWAY_ALIAS_UNMAPPED` (0x0605), RETURN_SENDER_SMTP (`550 5.1.1`), or `ERR_GATEWAY_ALIAS_ENCODING_INVALID` (0x0606) for the undecodable variant; any delivery to a local vanity after a failed chain lookup is non-conformant | construction-todo |
| DMTAP-GWNAME-05 | MUST | §7.10.6, §3.13.2, §3.13.4 | **A client MUST NOT present a legacy alias as the user's address.** It MUST NOT appear in the share-address, copy-address or QR-and-invite flows, MUST NOT be offered as the identity's display address, and wherever it does appear MUST be labelled with its provenance — "legacy alias, issued by `gw.example`, not portable". The share flow surfaces the user's own domain if they have one, otherwise their chain name, otherwise the key-name, in that order | manual attestation (client-UX review of the share/copy/QR/invite surfaces and the account's route list, for an identity holding a tier-3 alias and no own domain; no wire bytes to recompute) | non-conformant if a tier-3 alias appears in any share flow, is offered as the display address, appears unlabelled anywhere, or if the share flow prefers it over an available own-domain or chain name — the key-name is the floor, never the alias | manual-attestation |
| DMTAP-GWNAME-06 | MUST | §7.10.6 | **The gateway MUST NOT strip the native-name advertisement, and the client MUST let the user turn it off.** Outbound legacy mail SHOULD advertise the sender's native name in both a `DMTAP-Name:` header field and a human-readable signature line; that advertisement is the mechanism that **drains** aliases rather than hoping they drain, so a gateway MUST NOT strip or rewrite either. It is also an identifier, and some correspondence is deliberately compartmented, so the client MUST offer the user a way to suppress it | construction: relay an outbound legacy message carrying a `DMTAP-Name:` header and the body signature line; capture what the destination receives. Companion check: the client exposes a per-identity (or per-message) control that suppresses both | accept (both advertisements arrive unmodified); stripping or rewriting either at the gateway is non-conformant, as is a client that emits them with no way to turn them off | construction-todo |

---

## Inbound anti-abuse floor (§7.11.1, §7.11.3) — `GWFLOOR`

Level **Legacy**. A bridge that injects unauthenticated legacy mail into the mesh with the
standing of an established contact launders spam into the accountable network, and does it under
the recipient's own domain. §7.11 splits into a **floor** — *that* both directions are gated, *that*
the gates fail closed, and *which* signals gate them — and **numbers**, which are operator policy
(§7.13). These cases pin the floor and, deliberately, do not pin the numbers.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-GWFLOOR-01 | MUST | §7.11.1, §21.9 | **Legacy sender authentication, rejected early.** Before injecting anything into the mesh a gateway MUST apply SPF, DKIM and DMARC and reject on hard fail, pre-`DATA` where possible. The point of pre-`DATA` is that the gateway holds no queue: accepting the body and deciding later means either storing it or dropping it silently, and both are worse than a `550` the sending MTA can act on | construction: inbound transaction from an IP outside the envelope domain's SPF record with a DMARC policy of `reject`; observe at which SMTP verb the rejection occurs | accept (rejected with `550 5.7.1`, at or before `DATA`); accepting the message and dropping it after wrapping, or injecting it, is non-conformant | construction-todo |
| DMTAP-GWFLOOR-02 | MUST | §7.11.1, §9.2, §2.7a | **An injected legacy stranger is a cold contact.** The gateway MUST carry the legacy origin through the recipient's §9 cold-sender gate exactly as a native cold sender is carried: it MUST NOT inject legacy mail with the standing of an established contact, and MUST NOT strip or forge the recipient-facing challenge state. Mail it cannot qualify defers to the requests area, never the inbox, and a gateway MUST NOT emit an inbound MOTE that bypasses either check | construction: SPF/DKIM/DMARC-passing inbound mail from a legacy address with no relationship to the recipient; inspect the emitted MOTE's challenge state and the recipient-side placement. Variant: a gateway that sets the challenge state to that of an established contact | accept (the MOTE presents cold-sender standing and lands in the requests area); the forged-standing variant rejects → `ERR_CHALLENGE_FORGED_INVALID` (0x0703), DROP_SILENT, and delivery to the inbox on the gateway's say-so is non-conformant | construction-todo |
| DMTAP-GWFLOOR-03 | MUST | §7.11.3, §7.13 | **Implement the floor, choose the numbers.** The interoperable requirement is *that* both directions are gated, *that* the gates fail closed, and *which* signals gate them (SPF/DKIM/DMARC plus cold-sender inbound; authenticated-sender plus rate/volume outbound). The specific values — DMARC override handling, RBL choice, challenge thresholds, egress rates, volume caps — are operator policy and out of scope. A suite that tested the numbers would be enforcing one operator's policy as interop; a suite that tested nothing would let a gateway disable a gate and still claim conformance | construction: two gateways with deliberately different thresholds (different RBL sets, different challenge difficulty, different egress rate) drive the same inbound and outbound corpus; a third has one gate disabled entirely | accept (both threshold variants conform — divergence in the numbers is not a conformance signal); the third rejects: a disabled gate fails `DMTAP-GWFLOOR-01`, `-02` or `DMTAP-LEG-03` as applicable | construction-todo |

---

## Legacy client surfaces (§7.15.1, §7.15.2, §7.15.3) — `GWLEG`

Level **Legacy**. To speak IMAP, POP3, SMTP-submission, CalDAV or CardDAV a gateway **must
decrypt** the mailbox — those protocols have no notion of DMTAP's object encryption. DMTAP states
that plainly rather than presenting the legacy path as private, and these cases pin the three
consequences: the surfaces belong to the gateway and never to the node, the legacy ingress is not
the mesh relay (which stays ciphertext-only and content-blind), and a client may not present
gateway-served access as end-to-end. `-03` is `manual-attestation`: what a client claims about a
session has no wire bytes to recompute.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-GWLEG-01 | MUST | §7.15.1, §8.2 | **The legacy surfaces are the gateway's, and app-passwords are the credential.** A gateway serving legacy clients MUST project the identity's one MOTE store through the requested legacy protocol — IMAP/POP3 read access, SMTP-submission outbound, CalDAV/CardDAV as iCalendar/vCard — decrypting as §7.15.3 requires. Authentication is an app-specific password mapped to the identity, so a legacy client never touches the keypair, and revoking that password revokes that client. These are the only legacy client surfaces: the node exposes none of them | construction: authenticate an IMAP client with an app-password, read a folder, then revoke that app-password and retry; separately, attempt the same IMAP connection against the node's own listeners | accept (the projection serves while the app-password is live and the session fails immediately after revocation, with the identity keypair never presented); a node that answers IMAP/POP3/CalDAV/CardDAV itself is non-conformant | construction-todo |
| DMTAP-GWLEG-02 | MUST | §7.15.2, §4.3 | **The legacy ingress is not the mesh relay.** The gateway's legacy-client ingress accepts an inbound legacy connection, terminates TLS and speaks the legacy protocol against the mailbox — so it sees plaintext. It MUST NOT be confused with the node's native mesh relay (Circuit Relay v2 / DCUtR), which carries ciphertext-only, content-blind node↔node traffic and never speaks a legacy protocol. Collapsing the two would silently convert a content-blind role into a plaintext one | construction: drive an IMAPS session at a node acting as a Circuit Relay v2 relay for third parties; separately, verify that the relay path for a `mail` MOTE carries only ciphertext the relay cannot open | accept (the relay refuses the legacy protocol outright and holds no key that opens the traffic it forwards); a mesh relay that terminates a legacy session, or that can read what it forwards, is non-conformant | construction-todo |
| DMTAP-GWLEG-03 | MUST | §7.15.3, §8.2, §8.4 | **Gateway-served legacy access is not end-to-end, and the trust boundary is named.** A legacy client's mail, calendar and contacts are visible in the clear to whatever gateway serves them, so a client MUST NOT present gateway-served legacy access as end-to-end when served by a non-private gateway, and MUST surface **which** gateway — and therefore which trust boundary — serves the session. A private gateway (§7.15.4) means the operator is the user and no third party sees it; a public one is the same honest trade as choosing any hosted provider | manual attestation (client-UX review of the legacy-access settings and any privacy indicator, for an account served by a `public` gateway and again by a `private` one; no wire bytes to recompute) | non-conformant if the `public`-gateway session is presented as end-to-end or zero-access, or if the serving gateway's identity is not surfaced to the user in either case | manual-attestation |

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

Every `vectored` `PUB` case maps to an entry in the **separate** file
[`vectors/pub_vectors.json`](vectors/pub_vectors.json) (**all 15 of its 15 vectors** are referenced
by cases — none orphaned). Cross-check (case → vector):

| pub_vectors.json entry | driven by case(s) |
|-------------------------|-------------------|
| `pub_manifest_single_chunk` | PUB-01 |
| `pub_manifest_three_chunks` | PUB-02 (also the base fixture for PUB-09/-10's construction-todo tamper recipes) |
| `pub_announce_signing_preimage` | PUB-03 (also the base fixture for PUB-12's construction-todo recipe) |
| `pub_announce_id` | PUB-04 (also the base fixture for PUB-11's construction-todo recipe) |
| `pub_feed_entry_chain` | PUB-05 |
| `pub_feed_head_signing_preimage` | PUB-06 (also the base fixture for PUB-17's construction-todo recipe) |
| `pub_manifest_type_incompatibility` | PUB-07 |
| `pub_manifest_key5_forbidden` | PUB-08 |
| `pub_announce_supersede_same_author_valid` / `_cross_author_invalid` | PUB-13 |
| `pub_feed_rollback_strict_less_than` / `pub_feed_equal_seq_identical_tip_idempotent` | PUB-14 |
| `pub_feed_equal_seq_different_tip_fork` | PUB-15 |
| `pub_feed_genesis_carries_prev_malformed` / `pub_feed_nongenesis_missing_prev_malformed` | PUB-16 |

`pub_vectors.json` is generated by [`vectors/gen_pub_vectors.py`](vectors/gen_pub_vectors.py), not
by the `dmtap-core` reference crate (see the `PUB` section above and README.md's Provenance note)
— it is independently re-derivable by anyone with `pip install blake3 cryptography`, no Rust
toolchain required. The `CAD` cases carry no vectors (the profile allocates no wire bytes of its
own, §23.1); all 11 are `construction-todo` recipes over the `ArtifactMetadata`/`AssemblyStructure`
CDDL of §23. The `VIDEO` cases (§24) are likewise `construction-todo` recipes over the
`VideoManifest`/`Rendition`/`Comment`/… CDDL of §24 — with the one exception that
`DMTAP-VIDEO-03`/`-05` (the rendition-derivation statement, §24.4.4) *do* have a signable preimage
(DS-tag `"DMTAP-VID-v0/derivation"`) and become byte-backed KATs once a fixed-input derivation vector
is generated, re-derivable with `blake3` + `ed25519` and no reference implementation.
