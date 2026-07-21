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
| **Auth** — origin binding & the remote-node hazard (`AUTHORIG`) | 4 | 0 | 0 | 4 | 0 |
| **Auth** — key-bound sessions, re-validation & recovery (`AUTHSESS`) | 6 | 0 | 0 | 6 | 0 |
| **Auth** — OIDC bridge & honest limits (`AUTHBRIDGE`) | 5 | 0 | 0 | 4 | 1 |
| **Core** — bootstrap & substrate seam (`BOOT`) | 4 | 0 | 0 | 4 | 0 |
| **Private** — cover traffic, active-attack detection & the mix default (`COVER`) | 6 | 0 | 0 | 6 | 0 |
| **Private** — tier boundaries & push provider choice (`TIER`) | 2 | 0 | 0 | 2 | 0 |
| **Private** — data at rest, provenance limits & residuals (`REST`) | 5 | 0 | 0 | 4 | 1 |
| **Clients** — surfaces, decentralization invariant & UX obligations (`CLIUX`) | 5 | 0 | 0 | 2 | 3 |
| **Core** — anchor suite, DNS pointers & cold start (`ANCHOR`) | 6 | 0 | 0 | 6 | 0 |
| **Groups & Files** — group custody, fan-out & membership privacy (`GRPGOV`) | 6 | 0 | 0 | 6 | 0 |
| **Core** — issuer trust, vouch, postage & mixnet admission (`ABUSE`) | 5 | 0 | 0 | 5 | 0 |
| **Core** — the operator seam & the inviolable rule (`SEAM`) | 4 | 0 | 0 | 4 | 0 |
| **Core** — roles at scale: fleet, thin clients, buffers & status pages (`SCALE`) | 5 | 0 | 0 | 5 | 0 |
| **Core** — hybrid-suite composition (`HYBRID`) | 1 | 0 | 0 | 1 | 0 |
| **Core** — state-machine totality, the §20 [fill] rules (`FSM`) | 5 | 0 | 0 | 5 | 0 |
| **Core** — forward compatibility, the three unknown-value rules (`FWDCOMPAT`) | 3 | 0 | 0 | 3 | 0 |
| **Core** — DMTAP-PUB publication guards, optional `pub-1` (`PUBGUARD`) | 3 | 0 | 0 | 2 | 1 |
| **Core** — CAD assembly structure, optional `pub-1` (`CADASM`) | 1 | 0 | 0 | 1 | 0 |
| **Core** — video hints, migration & attestation badges, optional `pub-1` (`VIDMIG`) | 3 | 0 | 0 | 3 | 0 |
| **Core** — wire objects with no vector: decode & cross-field rules (`WIRE`) | 10 | 0 | 0 | 10 | 0 |
| **Core** — §18 KATs: manifest, mix descriptor, Sphinx framing (`WIREKAT`) | 9 | 9 | 0 | 0 | 0 |
| **Total** | **327** | **55** | **6** | **249** | **17** |

The 55 vectored + 6 self-contained cases (**61**) are fully machine-runnable **today** from
`vectors.json` / `pub_vectors.json` + the inline bytes here, with **no reference implementation
required**. They pin the entire deterministic, security-critical Core spine — canonical CBOR,
content addressing, the two MOTE signature preimages (§18.9.1/§18.9.2), Ed25519 (with RFC 8032
cross-checks), the 8-word key-name, safety numbers, suite fail-closed — **and the full DMTAP-PUB
manifest/announce/feed KAT set** (§22.2/§22.3/§22.4: plaintext chunk hashing + DS-tagged Merkle
root, the announce and feed-head signing preimages, `announce_id`, the prev-chain,
type-incompatibility with sealed manifests, the same-author supersede rule, and feed anti-rollback
incl. the idempotent-refetch and fork/equivocation branches).

The 249 `construction-todo` cases give the exact recipe and expected §21 error for every remaining
normative branch — the full §2.7 pipeline, identity/KT fail-closed, the higher levels, the
hardening families (`DENIABLE`/`ORG`/`KTV1`/`ATTEST`), the `PROFILE` display-data guards, the
pluggable-resolver guards (`RESOLVE`), the optional `PUSH` wake-signaling guards, the `FILE`
durability guards, the anti-drift families
(`MIXPROF`/`FLEET`/`GUARD`/`LOC`/`FLOOR`/`FAILCLASS`/`GWROLE`), the gateway families
(`GWOPS`/`GWSMTP`/`GWATT`/`GWNAME`/`GWFLOOR`/`GWLEG`), the remaining `PUB` fail-closed rows not yet
vectored, and the profile-level `CAD` and `VIDEO` checklists. Each becomes byte-backed when the
corresponding subsystem gains a fixed-input KAT (see README "Coverage vs. deferred").

The 17 `manual-attestation` cases are the MUSTs with **no wire bytes to recompute**: an in-product
disclosure, a share sheet, a process boundary or the population a deployment actually serves. They
are identified by `manual-attestation` in the **status** column, and each names the review that
settles it — client-UX review, operator-copy review, or deployment review.
**Fabricating byte vectors for them would assert a fact the protocol does not carry**; attestation
is the honest status, not a placeholder for a vector that could exist.

### Normative coverage, and what it is not

`make coverage` reports **100% of IMPL MUSTs sit in a section some case cites**, measured against
the curated denominator in [`scope.json`](scope.json). That sentence is doing exact work and is
easy to over-read, so:

- It is **section-level, not MUST-level.** A section counts as covered if *any* case cites it, not
  if every MUST in it is exercised. The metric is a deliberately generous floor: it says "nothing
  in the implementable spec is entirely unattended", never "everything is checked".
- It counts cases that **exist**, not cases that **pass.** Of 327 cases, 61 are byte-runnable
  today; the rest carry a construction recipe or are settled by review. **No implementation has
  been run against this suite**, so the suite is a specification of tests, not a test result.
- The denominator is **curated.** [`scope.json`](scope.json) classifies all 333 MUST-bearing
  sections and states a reason for each; the 89 non-IMPL sections are excluded because their MUSTs
  are owned by another clause (the §21 registry Action column, the §19/§20 appendices), bind a
  future registrant or an operator's process, or are not requirements at all. Inclusion is the
  default and every exclusion names its owner — but the classification is a judgement, and the
  intended response to disagreeing with one is to reclassify it `IMPL` and write the case.

The raw figure — every capitalised MUST in the specification, unclassified — is **84%**. Both
numbers are printed by `make coverage`, deliberately, so the curation can be checked rather than
trusted.

**Sync status:** `SUITE.md` and [`suite.json`](suite.json) are **in sync** — both carry the same
**327** case ids, and `make lint` (check C5) fails the build if they ever disagree, or if any
document states a different count. The changed deniable objects (§5.2.1 dedicated-`idk`) are still
to be re-vectored when the reference regenerates `vectors.json`.

> All 55 vectored cases correspond one-for-one to entries in `vectors.json` (43 cases / 42 of its
> 68 vectors — several vectors drive more than one case; the remaining 26 are pre-generated for
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

## Origin binding & the remote-node hazard (§13.3.1, §13.1) — `AUTHORIG`

Level **Auth**. Phishing resistance in DMTAP-Auth comes **entirely** from binding the assertion
to the *true* RP origin, and that binding must be injected by a trusted client on the user's side —
never by the signer trusting a value the RP handed it. The hazard the cases below exist for is
specific and structural: an always-on node that signs relayed challenges has no proximity channel
to fall back on, so origin binding evaporates and the node becomes a consent-farming oracle. The
defence is **intent matching** — a node signs only a challenge matching a nonce *it* minted when the
user actively started a login on its own authenticated client — and it is testable exactly because
an unsolicited challenge has nothing to match.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-AUTHORIG-01 | MUST | §13.3.1, §13.1, §13.7 | **A relayed challenge has no pending intent and MUST be refused.** For any node-signed login the node MUST bind the challenge to a pending, user-initiated login intent — a nonce the node itself minted at the moment the user started a login on the node's own authenticated client. An unsolicited challenge relayed by a third party has nothing to match, which is precisely what makes the phisher's relay attack detectable: the phisher can reproduce the challenge, but cannot cause the victim's node to have minted an intent for it | construction: a valid, unexpired `Challenge` for a real RP, presented to the user's node over a channel the node did not originate a request for (the phishing relay); compare against the same challenge delivered against a live intent the node minted | reject → `ERR_UNTRUSTED_APPROVAL_CHANNEL` (0x0507), FAIL_CLOSED_BLOCK for the relayed challenge; accept only the intent-matched one — a node that signs both is a consent-farming oracle | construction-todo |
| DMTAP-AUTHORIG-02 | MUST | §13.3.1, §10.7.3 | **Bare node-signed login is FORBIDDEN, and "approve any challenge" is not a mode.** A remote node signing on its own authority, without a local user-verification bound to the origin, is forbidden: the only permitted remote-node design is one where the passkey/WebAuthn ceremony happens in the user's client and the node's key is invoked only afterwards. Nodes MUST NOT offer an "approve any challenge" setting — a mode that pre-approves is indistinguishable, to the protocol, from the attack the intent binding exists to stop | construction: (a) drive a login against a node configured for bare node-signed operation with no client-side user verification; (b) enumerate the node's approval settings for any option that approves without per-login user action | reject → `ERR_UNTRUSTED_APPROVAL_CHANNEL` (0x0507), FAIL_CLOSED_BLOCK; a node exposing an "approve any challenge" mode at all is non-conformant, whether or not it is enabled | construction-todo |
| DMTAP-AUTHORIG-03 | MUST | §13.3.1, §13.7 | **A companion client TOFU-pins the RP origin and fails closed on mismatch.** A user without a passkey MAY use an authenticated paired companion client as the trusted client, but the companion receives `rp_origin` as *data* and can only display it — degrading to the user-verified mode §13.7 limit 1 calls weaker. To restore a machine-enforced comparison, a companion client MUST pin the RP's origin on first login and fail closed on any origin mismatch thereafter, so an established RP cannot later be confused with a look-alike | construction: complete a first login to `https://rp.example` through the companion path; then present a challenge from a look-alike origin (`https://rp-example.evil`, and an IDN homograph variant) for the same pinned RP identifier | reject → `ERR_ORIGIN_MISMATCH` (0x0501), FAIL_CLOSED_BLOCK; displaying the look-alike origin and relying on the user to notice it is exactly the weaker mode §13.7 forbids as the only mode | construction-todo |
| DMTAP-AUTHORIG-04 | MUST | §13.3.1 | **Approvals are rate-limited and logged.** The consent-farming defence is not only the intent binding: a node MUST rate-limit and log approvals, so a campaign that induces a user to approve repeatedly is bounded and, afterwards, visible. A node that approves without limit leaves no evidence that the attack happened | construction: drive N intent-matched approval prompts in rapid succession past the node's configured budget, then inspect the node's approval log | accept (approvals beyond the budget are refused, and every approval — granted or refused — appears in the log with its `rp_origin` and time); unlimited approvals, or approvals that leave no record, are non-conformant | construction-todo |

---

## Key-bound sessions, re-validation & recovery (§13.4) — `AUTHSESS`

Level **Auth**. §13.4 carries 15 MUSTs, more than any other clause in §13, and they exist because
an RP validates the login-time delegation **once**. Without the rules below, `IK` rotation, device
revocation and recovery — the three events that are supposed to end an attacker's access — never
reach a live RP session at all. The subtle one is `-04`: a bare revocation list cannot express
recovery, because a recovering owner on a fresh device does not know which RPs hold live sessions
and so cannot populate it. Keying the epoch to `Identity.version` is what makes "recovery
invalidates everything" true without the owner enumerating anybody.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-AUTHSESS-01 | MUST | §13.4 | **A native session is key-bound, never bearer.** Every request on a native session carries a fresh proof of possession — a DPoP JWT (RFC 9449) signed by the session key bound at login, or GNAP (RFC 9635) continuation. A bearer session on the native path is non-conformant: the whole "a stolen token is useless without the key" property (§13.1 goal 3) is exactly this rule and nothing else | construction: capture a valid native-session request, strip the DPoP proof and replay the token alone; variant: replay a captured DPoP proof on a second request | reject → `ERR_DPOP_PROOF_INVALID` (0x0506), FAIL_CLOSED_BLOCK for both the missing proof and the replayed one; a native session that authorizes on the token alone is non-conformant | construction-todo |
| DMTAP-AUTHSESS-02 | MUST | §13.4, §1.5 | **Revocation costs a session, not an identity.** Revoking one app or device session publishes a revocation to the transparency log and/or a short-lived status endpoint and MUST NOT require rotating `IK`; rotating a device key (§1.5) revokes all of that device's sessions at once. Session keys are per-RP, per-device ephemeral keys authorized by a device key, never `IK` itself — which is what bounds the blast radius and makes granular revocation possible in the first place | construction: establish two sessions on device D (RPs X and Y) and one on device E; revoke the D↔X session, then rotate D's device key; inspect `Identity.version`, and each session's liveness at its RP | accept (D↔X terminates and `Identity` is unchanged — no `IK` rotation; after the device-key rotation both of D's sessions terminate and E's survives); reject → `ERR_SESSION_REVOKED` (0x0504), REJECT_NOTIFY on any request from a terminated session | construction-todo |
| DMTAP-AUTHSESS-03 | MUST | §13.4, §16 | **Bounded re-validation.** Session authorizations MUST be short-lived, and an RP MUST NOT treat a login-time delegation as valid indefinitely: it MUST re-validate the delegation against the user's status endpoint or KT head at a bounded interval (§16), or bind the session to a revocation-list epoch it refreshes at that interval, and MUST terminate a session whose delegation no longer validates at the next check | construction: establish a session; revoke the authorizing delegation at the user's status endpoint; advance the clock past the §16 re-validation interval and issue a request with a valid, fresh DPoP proof | reject → `ERR_SESSION_REVOKED` (0x0504), REJECT_NOTIFY; an RP that keeps honouring the session past the interval because the proof itself is well-formed is non-conformant, and a session with no bounded re-validation at all fails this case by construction | construction-todo |
| DMTAP-AUTHSESS-04 | MUST | §13.4, §1.4, §10.7.3 | **Recovery invalidates every prior session authorization, and the epoch is keyed to `Identity.version`.** Losing `IK` and recovering MUST invalidate all prior session authorizations. A bare revocation list cannot deliver that: it enumerates *explicitly revoked* sessions, and a recovering owner on a fresh device with an empty store does not know which RPs hold live sessions, so a pre-recovery session survives and the guarantee silently fails. The revocation-list epoch MUST therefore be keyed to `Identity.version` (bumped by any `KeyRotation`/recovery), a session MUST carry the version it was authorized under, and re-validation MUST terminate it when the current version is higher. An RP that cannot bind its epoch to `Identity.version` MUST use the delegation-re-chain option instead | construction: RP holds a session authorized at `Identity.version = n`; the owner performs a full §1.4 recovery on a fresh device with no record of that RP, advancing the published version to `n+1`; the RP re-validates | reject → `STATUS_SESSIONS_INVALIDATED_ON_RECOVERY` (0x050A) — the session terminates on the version bump alone, with no revocation entry naming it; an RP whose epoch is not version-keyed and honours the pre-recovery session is non-conformant | construction-todo |
| DMTAP-AUTHSESS-05 | MUST | §13.4, §16, §3.3 | **Unreachable status endpoint: a bounded grace window, then closed.** If the status endpoint or KT head is unreachable at a re-validation check, the RP MUST NOT honour the session indefinitely — fail-open lets an attacker who partitions the endpoint keep a revoked session alive — and SHOULD NOT hard-fail instantly, which would log everyone out on a transient outage. It MUST honour the last successfully-validated delegation only until a bounded grace window (2× the re-validation interval, §16), then fail closed and require re-authentication | construction: partition the RP from the status endpoint immediately after a successful validation; issue requests at intervals spanning the grace window and past it, with valid DPoP proofs throughout | accept inside the grace window; reject → `ERR_SESSION_EXPIRED` (0x0505), REJECT_NOTIFY past it, requiring re-authentication. Both failure modes are non-conformant: honouring the session past the window (fail-open) and terminating at the first unreachable check (needless outage sensitivity) | construction-todo |
| DMTAP-AUTHSESS-06 | MUST | §13.4, §13.6 | **No proof-of-possession on the bridge path, and implementers MUST NOT assume otherwise.** A bridged login mints a classical OIDC ID Token — a *bearer* token — so §13.1 goal 3 is forfeited there. This is a disclosed exception, not a gap: what is non-conformant is a product that carries the native path's key-bound guarantee across to bridged sessions in its documentation, its UI, or its RP-side session handling | construction: complete a bridged login and inspect the resulting session's requests for any proof of possession; alongside, review what the product tells the user and the RP about that session's security properties | accept (the bridged session is a bearer session and is presented as one); presenting a bridged session as key-bound, or an RP-side implementation that assumes a DPoP proof will be present on the bridge path, is non-conformant | construction-todo |

---

## OIDC bridge & honest limits (§13.6, §13.7) — `AUTHBRIDGE`

Level **Auth**. The bridge exists because mainstream RP libraries assume a fixed issuer
allowlist, and it signs ID Tokens with its **own** key — so a compromised bridge could forge logins
for its RPs exactly as any classical IdP could. DMTAP-Auth bounds that two ways, and both bounds
are testable: the token embeds the user's own assertion so an RP can verify the user's key
directly, and every minted token is appended to a log the user's node monitors. `-02` is the one
that is easy to get wrong and fatal to get wrong: one bridge-audienced assertion replayed into
several RPs' tokens would authenticate to any of the bridge's RPs and collapse the per-RP `cnf`
unlinkability of §13.7 limit 7. `-05` is `manual-attestation`: a disclosure has no wire bytes.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-AUTHBRIDGE-01 | MUST | §13.6, §3.4 | **`did.json` is a pointer, not a proof.** A user's `did:web` document MUST be byte-consistent with the §3 DNS `name → key` binding and its KT entry — same `IK`, same current `Identity` hash — and an RP MUST cross-check the `did.json` key against DNS + KT and pin it, never trusting the DID document alone. It is the same discovery-not-proof pointer as the DNS record itself | construction: publish a `did.json` at `did:web:yourdomain:users:alice` whose key differs from the `_dmtap` DNS binding and the identity's KT entry; drive an RP login that resolves the DID document | reject → `ERR_RESOLVER_DISAGREEMENT` (0x0120), HALT_ALERT; authenticating on the DID document's key alone is non-conformant, and pinning it is what the cross-check exists to prevent | construction-todo |
| DMTAP-AUTHBRIDGE-02 | MUST | §13.6, §10.7.3, §13.7 | **One ceremony per consuming RP, one audience per assertion.** Every ID Token the bridge mints MUST embed the user's own §13.3 signed assertion and its `cnf`. To make direct verification meaningful and prevent cross-RP assertion reuse, the bridge MUST run a distinct §13.3 ceremony per consuming RP, with the assertion's `rp_origin`/`aud` set to the **target RP's** identifier and a fresh per-RP `cnf` — never one bridge-audienced assertion replayed into several tokens. An RP verifying the embedded assertion directly MUST check `assertion.aud` equals its own identifier and fail closed on mismatch | construction: bridge mints tokens for RPs X and Y from a single ceremony audienced to the bridge; RP X verifies the embedded assertion directly. Companion: compare the `cnf` values in the two tokens | reject → `ERR_ORIGIN_MISMATCH` (0x0501), FAIL_CLOSED_BLOCK — `assertion.aud` is the bridge, not X; and an identical `cnf` across X's and Y's tokens is independently non-conformant (it collapses the per-RP unlinkability of §13.7 limit 7) | construction-todo |
| DMTAP-AUTHBRIDGE-03 | MUST | §13.6, §3.5 | **Every minted token is logged, and the log is what makes a forged login detectable.** Every token the bridge mints MUST be appended to a bridge-transparency log (KT-style, §3.5) that the user's node monitors, so a bridge minting a login the user never performed is detectable. Without the append, the bridge's compromise is not merely possible but silent — and the whole bound DMTAP-Auth claims over a classical IdP is detection | construction: have the bridge mint a token for a login the user never initiated; run the node's monitor against the bridge log; variant: the bridge omits the append entirely for that token | accept (the monitor surfaces the unrecognized login for the appended case); reject → `ERR_KT_PROOF_INVALID` (0x0108), HALT_ALERT where the log cannot produce an inclusion proof for a token an RP presents — a token that is not in the log is not a token the user's node can be expected to have seen | construction-todo |
| DMTAP-AUTHBRIDGE-04 | MUST | §13.7, §10.7.3, §3.5.2 | **High-value login RPs require multi-log consistency or an OOB-verified pin.** A v0 single KT log that can present a split view is, for auth specifically, a silent per-RP account takeover: the RP is shown a key the owner never published and has no second opinion to contradict it. DMTAP-Auth therefore REQUIRES — even in v0 — that high-value login RPs verify the `name → key` binding against multiple independent KT logs, or against an out-of-band-verified pin, never a single unaudited log | construction: high-value RP; present a `name → key` binding attested by one log only (variant: by a set of logs that does not reach the §3.5.2(b) quorum), with no OOB-verified pin on file | reject → `ERR_KT_LOG_QUORUM_UNMET` (0x0111), FAIL_CLOSED_BLOCK; authenticating a high-value login on a single log's say-so is non-conformant even though v0's default KT profile is single-log | construction-todo |
| DMTAP-AUTHBRIDGE-05 | MUST | §13.7, §6 | **Login is a deliberate, per-RP disclosure — and a global `sub` MUST say so.** Authenticating to an RP intentionally reveals the user's identity to that RP; it is opt-in and per-RP and MUST NOT be conflated with, or allowed to weaken, the mail/messaging metadata-privacy guarantees of §6. Pairwise subject identifiers stay a SHOULD because some RP ecosystems need a stable portable `sub`, but a bridge that issues a **global** `sub` MUST disclose to the user that its RPs can correlate them across sites by it | manual attestation (review of the bridge's consent screen and documentation for a deployment issuing a global `sub`; and of the client's copy, for any claim that logging in preserves the §6 metadata-privacy properties) | non-conformant if a global `sub` is issued without the cross-site-correlation disclosure, or if login is presented as inheriting the messaging layer's metadata privacy | manual-attestation |

---

## Bootstrap & the substrate seam (§4.1, §4.2.2) — `BOOT`

Level **Core**. Whatever answers "how does a node with nothing find its first peer" is the most
centralizing component in any P2P system: it is consulted by every node at exactly the moment the
node can verify nothing. §4.2.2 specifies it rather than leaving it to each vendor's hardcoded
addresses, and these cases pin the three properties that keep it from becoming permanent
infrastructure — contacts first, ASN-disjointness in the shipped list, and a role that ends after
first contact. `-04` pins the other seam in §4: the substrate discriminator that stops libp2p from
becoming a flag day.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-BOOT-01 | MUST | §4.2.2, §3.4 | **Contacts are the bootstrap set, and the shipped list is a last resort.** A node that has ever pinned a contact already holds signed, verifiable peers and MUST attempt those first. Once a node has peers its `BootstrapSet` role **ends permanently**: it MUST prefer path 1 and MUST NOT re-consult the `BootstrapSet` while any known peer is reachable. A design where nodes periodically return to a well-known set has a permanent central dependency however that set is owned | construction: instrumented node with ≥ 1 pinned contact whose last-known address is reachable; restart it cold and count outbound connection attempts by category; variant: restart with the contact reachable but stale-addressed | accept (zero `BootstrapSet` contacts while any known peer is reachable, and the pinned contacts are attempted first); any consultation of the shipped list while a known peer answers is non-conformant | construction-todo |
| DMTAP-BOOT-02 | MUST | §4.2.2 | **≥ 3 nodes under ≥ 3 disjoint announced BGP origin ASNs.** A `BootstrapSet` whose entries all sit in one network is non-conformant, because it is indistinguishable from a central server. The axis is deliberately **ASNs, not attested operators**: running a bootstrap entry needs no scarce resource, so requiring a credential here would impose one on the single path a brand-new network must have working on day one, and would be unsatisfiable at launch. ASN-disjointness delivers the property that matters — no single network, datacenter or legal order controls every entry point — and is checkable from the addresses in the list itself | construction: signed `BootstrapSet`s with (a) 3 entries whose addresses all announce from one origin ASN, (b) 2 entries under 2 ASNs, (c) 3 entries under 3 disjoint ASNs | reject (a) and (b); accept (c). Rejecting (c) for lack of operator attestation is also non-conformant — attestation is deliberately **not** the axis here | construction-todo |
| DMTAP-BOOT-03 | MUST | §4.2.2, §3.3 | **Inspectable, overridable, and discovery-only.** The `BootstrapSet` MUST be user-inspectable and user-overridable in the client, and the client MUST surface which entry it actually used. It is discovery, never trust: a bootstrap peer can introduce a node to the network and can withhold, but every identity learned through it is verified against KT and pinned exactly as any other — a hostile bootstrap peer yields **no** peers, never wrong ones | construction: a hostile bootstrap entry that returns a peer whose claimed `name → ik` binding fails KT verification; alongside, inspect the client for the list, an override control, and the used-entry indicator | reject → `ERR_KT_PROOF_INVALID` (0x0108), FAIL_CLOSED_BLOCK for the mis-bound identity (the node ends with no peer rather than a wrong one); non-conformant if the list is not inspectable, not overridable, or the entry used is not surfaced | construction-todo |
| DMTAP-BOOT-04 | MUST | §4.1, §18.5.1, §21.24 | **libp2p is v0's substrate, not a flag day.** A `LocationRecord` carries an explicit `substrate` discriminator from the Transport Substrates registry (§21.24), and its `peer_id`/`addrs` are interpreted **relative to that substrate**; an absent field means libp2p for backward compatibility. A resolver dials a record only on a substrate it implements, and a record on a substrate it does not implement is simply unreachable to it — never guessed at, and never assumed to be libp2p because that is the only value v0 defines | construction: `LocationRecord`s carrying (a) `substrate` absent, (b) `substrate = 0x01`, (c) `substrate = 0xE0` (Private Use, unimplemented), each with well-formed `addrs` for its own substrate | accept (a) and (b) as libp2p; (c) is unreachable → `ERR_LOCATION_UNREACHABLE` (0x0303), ROTATE_RETRY — interpreting (c)'s `addrs` as multiaddrs and dialing them is non-conformant | construction-todo |

---

## Cover traffic, active-attack detection & the mix default (§4.4.2a, §4.4.3, §4.4.5, §4.4.7) — `COVER`

Level **Private**. Cover traffic is load-bearing, not decoration, and loop cover is the single
lever that converts drop/delay/flooding attacks from **undetectable** to
**detected-and-responded**. The property that makes it work is that loops are Sphinx packets
indistinguishable from real traffic on the same paths, so an adversary cannot suppress messages
while sparing loops. `-04` is the one an implementation is most tempted to get wrong under
pressure: on a detected active attack, falling back to `fast` "to get the message through" is
precisely the adversary's goal.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-COVER-01 | MUST | §4.4.5, §16.3, §14.1 | **Constant-rate cover is the default for always-on nodes.** Every `private`-tier node emits two Poisson streams — loop cover through a full 3-hop path back to itself via a SURB, and drop cover to a random mix that discards at the last hop. But an always-on node MUST emit **constant-rate** cover: Poisson cover blurs the relationship between activity and traffic, whereas constant-rate cover yields nothing to traffic analysis however long the observation. At one cell per 30 s that is ≈ 5.6 MB/day, so the defence is free for exactly the device class that holds the mailbox; Poisson remains permitted **only** for battery- or metered-constrained devices | construction: always-on mains-powered node with a public address; record its egress cell timestamps over a window spanning long idle and heavy-activity periods; repeat on a battery-constrained intermittent device | accept (the always-on node's envelope is flat and uncorrelated with activity; the intermittent device's may be Poisson); an always-on node whose emission rate tracks user activity is non-conformant | construction-todo |
| DMTAP-COVER-02 | MUST | §4.4.5, §6.4 | **Recipient-side loop cover closes the receipt-timing leak.** An always-on recipient node MUST **also** receive a steady loop-cover stream, so real receipts are indistinguishable from cover on its delivery link. Without it an observer of the recipient's link learns *when* mail arrives even though it cannot read it — and §6.4 item 2 is the clause that owns this, not the honest-limits prose | construction: observe an always-on recipient node's inbound link across a period containing real deliveries and a period containing none; compare the arrival distributions | accept (the two periods are indistinguishable at the link); an inbound stream that is quiet when no mail arrives is non-conformant — the node's sending cover does not substitute for it | construction-todo |
| DMTAP-COVER-03 | MUST | §4.4.7, §16.3 | **The detection rule is a measurement, not an intuition.** A node MUST track, over a sliding window, the fraction of its loops that return within their expected delay budget and their latency distribution; if the return fraction drops below the loop-loss threshold (§16.3), or latencies inflate beyond what the exponential budget explains, it MUST infer an active drop/delay attack on its paths and raise `ERR_MIX_ACTIVE_ATTACK_SUSPECTED` | construction: hold the fleet and paths fixed and drop a controlled fraction of the node's traffic at one mix, sweeping the fraction across the §16.3 threshold; separately, add latency beyond the exponential budget with no loss at all | reject → `ERR_MIX_ACTIVE_ATTACK_SUSPECTED` (0x030F), HALT_ALERT once either signal crosses its bound; a node that only counts total delivery failures, and never measures its own loops, cannot pass this case | construction-todo |
| DMTAP-COVER-04 | MUST | §4.4.7, §4.4.9 | **Rotate, alert, fail closed — never silently continue.** On an inferred active attack the node MUST (1) rotate away from the implicated mixes and entry guards and rebuild over alternate, operator-diverse paths, (2) raise a `HALT_ALERT` to the user, and (3) **fail closed for the `private` tier**: it MUST NOT silently fall back to `fast`, or to a shorter or less diverse path, to get the message through. That fallback is exactly the adversary's goal — an attacker who can degrade a path gets to choose the tier the message travels on | construction: drive `DMTAP-COVER-03`'s detection to fire while a `private`-tier MOTE is queued; record the tier, hop count and guard set of every subsequent dispatch attempt, and the user-visible state | accept (paths rebuilt away from the implicated mixes and guards, a `HALT_ALERT` surfaced, and the MOTE still `private` — held rather than downgraded); any dispatch at `fast`, or on a path below the in-force profile's bar, is non-conformant → `ERR_PRIVATE_TIER_DOWNGRADE_REFUSED` (0x0310) is the correct refusal | construction-todo |
| DMTAP-COVER-05 | MUST | §4.4.3, §4.4.6 | **A fresh, independent path per Sphinx cell.** A sender MUST select an independent path for **each** cell — including every cell of a multi-cell MOTE and every cover packet — so no persistent circuit exists to correlate. This is also what makes `private` retry work at all: §4.7 requires a re-onion-wrap before re-dispatch, because identical Sphinx bytes are dropped at the first honest hop as a per-hop-tag replay | construction: send a multi-cell `private` MOTE and a burst of cover packets; extract the hop sequence of each cell. Companion: force a `RETRY → IN_FLIGHT` transition and compare the re-dispatched bytes with the original | accept (no two cells share a path, and the retry carries fresh `α`/per-hop tags under a stable envelope `id`); re-dispatching identical `private` bytes rejects at the first hop → `ERR_MIX_REPLAY_DETECTED` (0x030E), so an implementation that skips the re-wrap can never deliver a retried `private` MOTE | construction-todo |
| DMTAP-COVER-06 | MUST | §4.4.2a, §4.4.2 | **The mix role is default-on for always-on public nodes, and default-off for everything else.** A mixnet with no mixes protects nobody and DMTAP has no launching operator whose fleet it could borrow, so a node that is always-on and has a public address MUST default to serving the mix role and MUST publish a `MixNodeDescriptor`. The operator MAY turn it off — the requirement is on the **default**. Symmetrically, intermittent, battery- or metered-connection devices and nodes without a public address MUST NOT default to it: a mix that disappears mid-epoch degrades everyone's paths | construction: first-run configuration on (a) a mains-powered node with a public address, (b) a phone on a metered connection, (c) a node behind a NAT with no public address — inspect the default mix-role setting and whether a `MixNodeDescriptor` is published | accept ((a) defaults on and publishes a descriptor; (b) and (c) default off and publish none); a build that ships (a) opt-in, or (b)/(c) opt-out, is non-conformant | construction-todo |

---

## Tier boundaries & push provider choice (§4.5, §4.9.3, §6.5) — `TIER`

Level **Private**. Two places where a convenient default would quietly change what the protocol
claims: routing bulk transfer over the mixnet (impractical, so it does not happen — and the client
must not claim mixnet-grade privacy for it anyway), and reaching for a platform push service on a
platform that did not require one.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-TIER-01 | MUST | §4.5, §6.5, §2.5 | **Large-tier bulk does not traverse the mixnet, and is not described as if it did.** Blobs above the large-tier boundary MUST NOT traverse the mixnet — the bandwidth and latency are impractical — while normal-tier chunks do ride it like messages. The control MOTE carrying the manifest and key travels `private`; the bytes do not. The honest consequence is that a well-positioned observer learns the fact and approximate size of a large transfer, and an implementation MUST NOT claim mixnet-grade metadata privacy for it | construction: offer a large-tier file; trace the control MOTE and the chunk fetches separately, and inspect what the client tells the user about the transfer's privacy | accept (control MOTE on `private`, chunk fetch on the fast/bulk path); routing the chunks over the mixnet is non-conformant, and so is presenting the chunk fetch as carrying the `private` tier's metadata protection | construction-todo |
| DMTAP-TIER-02 | MUST | §4.9.3, §6.6 | **Prefer the open push provider wherever the platform allows.** A conforming node MUST prefer an open provider — UnifiedPush or Web Push — wherever the platform allows, and MUST fall back to APNs or FCM **only** on a platform that mandates them. The wake payload is the same RFC 8291-sealed content-free token either way, so the provider choice changes only who is in the path, which is exactly why the preference is normative rather than advisory | construction: enable push on (a) a desktop/browser platform where Web Push is available, (b) a de-Googled Android with UnifiedPush available, (c) iOS; inspect the provider tag in the resulting `PushSubscription` | accept ((a) Web Push, (b) UnifiedPush, (c) APNs); selecting FCM on (b), or any closed bridge on (a), is non-conformant even though the sealed token is identical | construction-todo |

---

## Data at rest, provenance limits & the residuals (§6.4, §6.7, §6.8, §6.9) — `REST`

Level **Private**. §6.7 is the clause that decides what a seized device yields, and it draws the
line sharply on purpose: an offline-seized device gives up inert ciphertext, a live unlocked device
reads what its user reads, and the specification says so rather than blurring the two. §6.8 draws
the matching line for provenance — the recipient learns which trust boundaries a message crossed
and never which nodes carried it, because the private path is unknown even to the recipient's own
node. `-05` is `manual-attestation`: what a client claims about a blinded tag has no wire bytes.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-REST-01 | MUST | §6.7, §1.2a, §16.9 | **The at-rest key is unlock-gated and evicted on relock.** The at-rest key MUST be released only on device unlock — wrapped by a key the hardware keystore yields after a successful biometric/PIN authentication — and evicted from memory on relock or timeout (§16.9); implementations MUST NOT keep it resident indefinitely across a locked device. The single exception is a platform with **no unlock signal** (a headless always-on box), which MUST instead seal the key to the strongest available boot-time protection; the eviction it cannot observe does not apply and everything else does | construction: unlock a device, read the store, relock, wait past the §16.9 timeout and inspect process memory for the at-rest key; repeat on a headless node with no lock concept and inspect how the key is sealed at boot | accept (key absent from memory after relock+timeout, and store reads fail until the next unlock; on the headless node the key is keystore- or full-disk-sealed at boot); a key that survives relock, or a headless node holding it unsealed, is non-conformant | construction-todo |
| DMTAP-REST-02 | MUST | §6.7, §18.3.6 | **`sensitive` is MAY-send, MUST-honor.** A sender MAY mark a message `sensitive` (a `Headers` flag); the receiving client **MUST NOT persist it at rest** — it is held in memory for an ephemeral view and dropped, never written to the durable MOTE store — so a later seizure reveals nothing of it. Like `redact`/`expires` this is cooperative and a least-persistence reduction, not a guarantee against a live endpoint, and it MUST NOT be presented as one | construction: deliver a `sensitive`-flagged MOTE, view it, then power-cycle and inspect the durable store, any search index, any thumbnail or preview cache, and the device-cluster sync journal (§5.6) | accept (no trace of the message body in any durable artifact, including derived indexes and the cluster journal); persisting it anywhere at rest is non-conformant, and so is presenting the flag as protection against a compromised recipient | construction-todo |
| DMTAP-REST-03 | MUST | §6.7, §5.5, §5.8.2 | **Removal from a shared folder re-keys the files.** MLS removal blocks a removed member's *future* messages but does not revoke file keys they already hold, so when a member is removed from a shared file folder the node MUST re-key and re-encrypt every file that member had access to **by default**, and `confidential` folders MUST always re-key with no opt-out. Without it a removed member keeps indefinite read access to everything already shared, and clients MUST surface that pre-removal copies already taken cannot be reached | construction: share a folder with member M, let M fetch the chunks, remove M from the group; inspect whether the stored chunks are re-encrypted under fresh per-file keys and whether M's held keys still open them. Variant: a `confidential` folder with re-keying disabled by policy | accept (files re-keyed and re-encrypted, M's retained keys no longer open the current chunks); reject → `ERR_FILE_KEY_MISSING_OR_REVOKED` (0x0805) on M's post-removal fetch; a `confidential` folder that allows the opt-out is non-conformant | construction-todo |
| DMTAP-REST-04 | MUST | §6.8, §18.8.1, §6.2 | **Provenance names boundaries, never nodes.** A `ProvenanceRecord` MUST NOT contain — and a node MUST NOT synthesize — anything from which the path could be reconstructed. For the `private` tier the recipient learns only the profile floor the path satisfied, never a mix-node identity, address, exact hop count, path descriptor or per-hop timing: the private path is by design unknown even to the recipient's own node. The record is node-local, served only to the owner's own cluster, and never attached to a forwarded MOTE | construction: receive a `private`-tier MOTE and dump the full `ProvenanceRecord`; then forward that MOTE and inspect the emitted object for any provenance field | accept (a profile-floor statement and nothing more; no node identity, address, exact hop count or timing anywhere in it, and no provenance field on the forwarded MOTE); a node that reports an exact hop count, or invents a hop list it does not know, is non-conformant | construction-todo |
| DMTAP-REST-05 | MUST | §6.9, §6.4, §2.2a, §18.3.2 | **Blinded tags are not recipient anonymity, and v0's single KT log is not equivocation-proof.** A `BlindedTag` is unlinkable to the persistent key across time and observers but does **not** hide last-hop delivery, and implementations MUST NOT present it as full recipient anonymity. In the same spirit, sealed sender plus mixnet hides receipt *timing* but does not erase last-hop observability — and every high-value contact, and every DMTAP-Auth login RP, MUST require multi-log consistency rather than relying on a single v0 log | manual attestation (review of the client's privacy copy and any per-message or per-contact privacy indicator, on a deployment using blinded tags and the v0 single-log KT profile; no wire bytes to recompute) | non-conformant if a blinded tag is presented as recipient anonymity, if last-hop observability is not disclosed where the product makes an anonymity claim, or if a high-value contact is verified against a single unaudited log | manual-attestation |

---

## Client surfaces, the decentralization invariant & UX obligations (§8.2, §8.4, §8.5, §8.6, §8.7) — `CLIUX`

Level **Clients**. Most of §8 is guidance, but four rules in it are MUSTs because getting them
wrong silently converts an honest design into a dishonest product: presenting a gateway-served
legacy session as end-to-end, over-claiming on `private` while the fleet is small, letting a silent
auto-forward rule redirect a mailbox, and offering deniable mode as a default rather than a choice.
Three of the four are `manual-attestation` — a claim a client makes has no wire bytes — and `-02`
is deliberately not, because a silent grant is an observable event on the owner's cluster.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-CLIUX-01 | MUST | §8.5, §8.1 | **Native surfaces only on the node.** Every data class — mail, chat, files, calendar, contacts, identity and login — lives on the user's node, end-to-end encrypted, synced across the device cluster over JMAP, shared through the same MLS groups and routed over the same mesh, with no central server on the native path. The node runs **native surfaces only** (JMAP + the mesh) and no legacy protocol server of any kind: IMAP/POP/SMTP/CalDAV/CardDAV live only on the gateway (§7.15) as edge surfaces, and the OIDC bridge is likewise an edge surface | construction: port-scan and protocol-probe a node for IMAP/POP3/SMTP/CalDAV/CardDAV/OIDC listeners; separately, exercise calendar and contacts end to end and confirm they resolve through JMAP and the mesh with no third-party service in the path | accept (no legacy protocol server on the node, and every data class resolvable with the node as the only authority); a node answering any legacy protocol itself is non-conformant, as is any data class that silently depends on a central service on the native path | construction-todo |
| DMTAP-CLIUX-02 | MUST | §8.5, §13.5, §13.4, §3.5, §17.6 | **No silent grant.** Silent grants that redirect or delegate the user's data — **auto-forward rules**, new **capability delegations** (§13.5) and new **RP-session authorizations** (§13.4) — MUST be surfaced to the owner's device cluster and logged to KT self-monitoring, so a business-email-compromise-style silent redirect is owner-visible rather than covert. Silent forwarding-rule injection is a live real-world account-takeover technique, and the defence needs no new cryptography: it is an existing replication path plus a rule that it MUST be used | construction: install an auto-forward rule, mint a capability delegation, and authorize an RP session — each through whatever surface the implementation exposes, including an admin or recovery path; then inspect every other device in the owner's cluster and the KT self-monitoring feed | accept (all three appear on every cluster device and in the KT self-monitoring path); any grant installable without appearing on both is non-conformant, and a path that bypasses the log is exactly the takeover vector this rule closes | construction-todo |
| DMTAP-CLIUX-03 | MUST | §8.6, §6.6, §4.4.11, §4.4.10a, §7.9 | **Do not over-claim on `private`, and never invent a hop.** While the mix fleet is small the client MUST honour the §6.6/§4.4.11 disclosure and MUST NOT present `private` as absolute anonymity; the transport-path graph shows boundary crossings, never a node-by-node trace, and individual mix nodes MUST NOT be drawn because the node does not know them (§6.8). On the Bootstrap profile the client MUST additionally show that metadata privacy is degraded **and why**, and MUST NOT state an anonymity set at all. And it MUST NOT show a pure-mesh message as a gateway operation — no operator was on that path to observe it | manual attestation (client-UX review of the transport-path surface on (a) a Standard-profile fleet, (b) a Bootstrap-profile fleet, for a `private` message, a `fast` message, a gateway-touched message and a pure-mesh message) | non-conformant if `private` is rendered as absolute anonymity, if any individual mix node appears in the graph, if the Bootstrap profile is shown without its degradation reason or with an anonymity-set figure, or if a pure-mesh message is attributed to a gateway | manual-attestation |
| DMTAP-CLIUX-04 | MUST | §8.7, §5.2.1, §10.7.4 | **Deniable mode is an explicit per-conversation choice, and its absence is never a silent downgrade.** A client offering deniable 1:1 MUST present it as an explicit per-conversation user choice, never a silent default, and MUST show that a deniable transcript is unattributable. Where the recipient has not advertised the capability the client MUST **surface the choice** — send non-deniable, or don't send — and MUST NOT silently downgrade the user's *expectation* of deniability, which is the one thing the mode exists to provide | manual attestation (client-UX review: the deniable-mode selector, what it says a deniable transcript is, and the flow taken when the recipient has not advertised `deniable-1:1` — the `ERR_DENIABLE_MODE_UNAVAILABLE` (0x040E) path) | non-conformant if deniable mode is a silent default, if the unattributability property is not shown, or if an unavailable recipient results in a non-deniable send without an explicit user decision | manual-attestation |
| DMTAP-CLIUX-05 | MUST | §8.7, §3.10.2, §3.10.5, §1.2a | **Org custody is disclosed, and attestation is advisory hardening.** A client MUST render an org-managed (escrowed-key) account distinguishably and MUST NOT present it as equivalent to a sovereign identity (`ERR_ORG_MANAGED_UNDISCLOSED`, `0x0115`), and MUST show every administrative capability an org holds over the account — including the offboarding path. Separately, it MUST make clear that device attestation is **advisory hardening**: an unattested device is not thereby untrusted, and presenting attestation as a security guarantee misstates what a platform attestation proves | manual attestation (client-UX review of an org-managed account's rendering, the org-capability list and offboarding disclosure, and the copy shown for an attested vs. unattested device) | non-conformant if an org-managed account is rendered indistinguishably from a sovereign one, if the org's capabilities over it are not shown, or if attestation is presented as more than advisory hardening | manual-attestation |

---

## Anchor suite, DNS pointers & cold start (§1.2.0, §3.1, §3.2, §3.8, §3.9.3, §3.13.5) — `ANCHOR`

Level **Core**. `IK` and the device keys under it have **opposite** requirements — decades versus
months, rare versus constant — so §1.2.0 lets an identity carry an anchor suite independent of its
operational one, and requires every verifier to check a signature under the suite of the key that
made it. The rest of this family pins what DNS is and is not: a discovery pointer that MUST NOT
hold location and is never proof, plus the one path that works when a brand-new identity has
literally nothing.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-ANCHOR-01 | MUST | §1.2.0, §1.3, §21.15 | **Verify each signature under the suite of the key that made it.** An `Identity` carries an `anchor_suite` governing `IK` and every signature `IK` itself makes, independent of the operational `suite` governing everything below. A conformant implementation MUST accept an anchor suite that differs from the operational suite, and MUST NOT verify under a single per-object suite assumed to cover both — the intended anchor profile is SLH-DSA (`0x04`), whose ~7.9 KB signatures are irrelevant at anchor-signing frequency and whose security rests on no algebraic structure | construction: an `Identity` with `anchor_suite = 0x04` and operational `suite = 0x02`, carrying an `IK`-made `DeviceCert` signature under `0x04` and a device-made `Payload.sig` under `0x02` | accept (both verify, each under its own key's suite); rejecting the object because the two suites differ, or attempting the `IK` signature under `0x02`, is non-conformant → `ERR_IDENTITY_SIG_INVALID` (0x0103) would be the wrong answer here | construction-todo |
| DMTAP-ANCHOR-02 | MUST | §3.1, §3.2, §4.2 | **DNS points; it never locates and never proves.** DNS holds the stable `name → key` pointer and MUST NOT hold location — that is the mesh's job (§4) — and it is never the proof: KT plus pinning are. A resolver that accepts an address out of a `_dmtap` record has folded the dynamic layer into the static, cacheable one and has made a stale or hostile zone into a routing authority | construction: a `_dmtap` TXT carrying an address/multiaddr-shaped parameter alongside `ik`; and a valid `ik` binding presented with no KT verification available | accept the pointer while ignoring any location-shaped parameter (the mesh `LocationRecord` remains the only source of `addrs`); reject → `ERR_KT_UNREACHABLE` (0x0106), FAIL_CLOSED_BLOCK for the unverifiable binding — a DNS record alone MUST NOT be TOFU-pinned silently | construction-todo |
| DMTAP-ANCHOR-03 | MUST | §3.2, §13.6, §3.4 | **A `did.json` is byte-consistent with DNS + KT, or it is nothing.** Where an identity is also published as a `did:web` document, that document's key MUST be byte-consistent with the DNS `name → key` binding and its KT entry — same `IK`, same `Identity` hash — and a verifier MUST cross-check the two and pin. The DID document is the same discovery-only pointer as DNS, never proof on its own | construction: `did.json` whose `IK` matches DNS but whose pinned `Identity` hash names an older version; variant: a `did.json` published for an identity with no DNS binding at all | reject → `ERR_STALE_ROLLBACK` (0x0105), FAIL_CLOSED_BLOCK for the stale hash; the DNS-less variant is simply unresolved (`ERR_NAME_RESOLUTION_FAILED`, 0x0109) — a DID document cannot substitute for the binding it is supposed to mirror | construction-todo |
| DMTAP-ANCHOR-04 | MUST | §3.8, §3.10.1 | **DNS is a generated projection, never hand-authored.** The identity is a key and DNS is an auto-managed projection of it; a conformant client SHOULD default to Tier B and **never require a user to author DNS**. For a Tier C own-domain user the domain's own DNS must carry MX/SPF/DKIM/DMARC (DMARC alignment is defined on the `From:` domain — unavoidable while legacy exists), but that requirement MUST NOT be pushed onto the user as manual record editing | construction: complete onboarding at each tier and record every step that requires the user to type a DNS record by hand; for Tier C, verify the generated zone carries MX/SPF/DKIM/DMARC | accept (no tier requires hand-authored DNS, and Tier C's generated projection is complete); an onboarding flow that hands the user a record to paste into a registrar as a required step is non-conformant | construction-todo |
| DMTAP-ANCHOR-05 | MUST | §3.13.5, §3.4.1, §4.2.2 | **Out-of-band introduction is the primary first-contact path, and it is optional.** A brand-new identity has no key-name anyone can search, no contacts, and no measurements — every mechanism that makes DMTAP graceful is downstream of a first relationship. A client MUST implement at least one out-of-band exchange (QR, invite link or contact card carrying the identity key, a peer hint, and key verification for free), and MUST present it as optional: nothing in DMTAP requires it, and a user who declines still reaches the network by §4.2.2's remaining rungs and is still reachable cold by key-name | construction: first-run a node with no contacts; exercise the out-of-band exchange end to end (scan → pinned, verified contact with a peer hint), then first-run a second node and decline every introduction offer | accept (the exchange yields a pinned, OOB-verified contact with zero infrastructure — no DNS, chain, KT, gateway, rendezvous or bootstrap set; and the declining node still reaches the network and still receives cold contact at its key-name); a client with no such exchange, or one that blocks setup until an introduction happens, is non-conformant | construction-todo |
| DMTAP-ANCHOR-06 | MUST | §3.9.3, §6.7 | **Petnames are a social-graph artifact and are stored encrypted at rest.** A petname is local-scope only and never leaves the device cluster; it MUST be stored encrypted at rest with the mailbox. The reason is not tidiness: a petname table is a labelled edge list of the user's social graph, which is exactly the thing §6 spends the rest of the protocol hiding on the wire | construction: assign petnames, power off, and inspect the on-disk store, any search index and any backup/export artifact for petname strings in the clear | accept (no petname recoverable from a powered-off device without the at-rest key, per `DMTAP-REST-01`); a plaintext petname table, index or export is non-conformant even though petnames never touch the wire | construction-todo |

---

## Group custody, fan-out & membership privacy (§5.1.1, §5.6.5, §5.8.3, §5.8.4, §5.8.6, §5.8.7) — `GRPGOV`

Level **Groups & Files**. A group is an addressable identity with its own keypair, so it needs the
same custody discipline as a person — otherwise one admin holding the group key can hijack
`team@company.com`. §5.8.6 is honest about the limit: a FROST signature is bit-for-bit
indistinguishable from a single-signer signature, so threshold custody is a SHOULD and only the
KT-anchored DKG/VSS commitment makes the claim checkable. `-01` tests exactly the part that *is*
enforceable — where the commitment object is carried, an uncommitted group key MUST be rejected.
`-03` and `-04` pin the amplification rules, which are the difference between a group address and
an open spam relay.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-GRPGOV-01 | MUST | §5.8.6, §5.8.7, §1.4 | **Where the commitment is carried, an uncommitted group key is rejected.** Threshold custody is a SHOULD because a threshold Schnorr signature is indistinguishable from a single-signer one, so the signature alone proves nothing. What *is* normative: where the §18 DKG/VSS commitment object is carried, members MUST reject an uncommitted group key — a group `Identity` whose signing key is not accompanied by a valid commitment MUST NOT be treated as threshold-held. Separately, changes to the group `Identity`, its recovery methods or its key MUST satisfy the group's `rotate_threshold` under §1.4's weakening-quorum and veto rules, and group `Identity`/`KeyRotation`/`RecoveryPolicy` events MUST be KT-logged | construction: a group whose deployment carries commitment objects presents (a) a group `Identity` with a valid VSS commitment over ≥ `rotate_threshold` distinct admin keys, (b) the same `Identity` with the commitment absent, (c) a group `KeyRotation` signed by one admin where `rotate_threshold = 2` | accept (a); reject (b) — it MUST NOT be presented or treated as threshold-held; reject (c) → `ERR_KEYROTATION_UNAUTHORIZED` (0x0121), FAIL_CLOSED_BLOCK, and an event absent from KT is independently non-conformant | construction-todo |
| DMTAP-GRPGOV-02 | MUST | §5.1.1, §5.1 | **A 1:1 is two symmetric co-committers, and equivocation still halts.** The committer-takeover machinery cannot work at n = 2: takeover needs a both-of-two quorum a live peer can never assemble when its counterpart is offline or censoring. So each member serializes its own handshakes under its own strictly-increasing `hs_seq` chained by `prev`, concurrent Commits are resolved by the §18.9 committer-rank preimage (folding in the epoch, so neither peer can grind a key that wins every race), and a peer presenting two different handshakes at the same `(originator_ik, hs_seq)` produces self-signed fork evidence the other member MUST `HALT_ALERT` on | construction: drive concurrent Commits from both members at one epoch and observe convergence across several epochs; then have one member present two distinct handshakes at the same `(originator_ik, hs_seq)` | accept (both peers converge on the identical epoch chain with no vote, and the winner varies across epochs); reject → `ERR_COMMITTER_FORK_DETECTED` (0x0404), HALT_ALERT on the duplicated `hs_seq` | construction-todo |
| DMTAP-GRPGOV-03 | MUST | §5.8.4, §9.9, §9.2 | **No accountability laundering behind a group address.** Fan-out is an amplification vector: one post becomes N deliveries, so each recipient's per-sender policy MUST be applied to the **original poster**, never to the list identity. The poster's proof MUST be carried on each per-member delivery, and *which* proof depends on the membership model, because ARC is per-origin scoped and does not compose with fan-out: member-visible channels MAY use per-member ARC, while hidden-membership lists MUST use postage or PoW scoped to the list address, since per-member ARC would require the poster to know the members | construction: post to (a) a member-visible channel carrying per-member ARC, (b) a hidden-membership list carrying a list-scoped postage/PoW proof, (c) the same hidden list carrying nothing but the committer's own standing; inspect each per-member delivery for the poster's proof and each recipient's policy evaluation | accept (a) and (b) — every delivery carries the poster's proof and is evaluated against the poster; reject (c) → `ERR_CHALLENGE_ABSENT_INSUFFICIENT` (0x0206); evaluating any of them against the *list* identity is non-conformant | construction-todo |
| DMTAP-GRPGOV-04 | MUST | §5.8.4, §9.9 | **Fan-out is rate-limited per poster, `open` amplification is capped, and large lists cost.** A list MUST rate-limit fan-out per poster and cap amplification for `open` (anyone-can-post) lists, and posting to a large list MUST require postage or PoW commensurate with the fan-out size. Without all three, a single accepted post is an unbounded multiplier, and the cheapest attack on the network is to find one open list | construction: from one poster, drive posts past the per-poster fan-out budget on an `open` list of N members; separately, post to a large list carrying no postage or PoW | reject → `ERR_RATE_LIMIT_EXCEEDED` (0x070C), DEFER_REQUESTS past the per-poster budget; reject → `ERR_CHALLENGE_ABSENT_INSUFFICIENT` (0x0206) for the unpaid large-list post; an `open` list with no amplification cap is non-conformant even before either limit is reached | construction-todo |
| DMTAP-GRPGOV-05 | MUST | §5.8.3, §5.8.1 | **Hidden membership is a supported model, not a setting that leaks anyway.** Broadcast lists MUST support hidden membership: members receive via per-member sealed delivery and do not learn each other's keys. MLS's ratchet tree exposes members to one another by default, so a hidden-membership list uses relay/committer fan-out with the list identity re-sealing to each member individually — paying the shared-group efficiency to get the property. Channels use the normal member-visible tree, and the model is chosen per group and disclosed | construction: create a hidden-membership broadcast list with ≥ 3 members; from one member's node, enumerate everything reachable about the roster — ratchet tree, `GroupInfo`, delivery metadata — and compare against the same enumeration on a member-visible channel | accept (the hidden list yields no other member's key or count to a member, while the channel does); a "hidden" list implemented over a shared member-visible tree is non-conformant however it is labelled | construction-todo |
| DMTAP-GRPGOV-06 | MUST | §5.6.5, §16.10, §5.6.1 | **A tombstone is never GC'd before the stability cut — and a dead device cannot stall GC forever.** A delete tombstone MAY be reclaimed only once every **live** cluster member has acknowledged an HLC ≥ the tombstone's and the §16.10 retention floor has elapsed; reclaiming early lets a partitioned device's stale add resurrect a deleted object. The cut is computed over live members only — a device that has not advanced its `StabilityMark` within the cluster-member-liveness timeout is excluded — so an unrevoked dead device cannot hold the tombstone store open indefinitely. Exclusion affects only **when** tombstones may be reclaimed, never **whether** an op is authorized | construction: three-device cluster; delete an object, hold device C partitioned with a stale add for that object, and attempt GC before the cut and after C is excluded by the liveness timeout; then return C and let it backfill | accept (no GC before the cut and the retention floor; GC proceeds once C is excluded as stale; the returning C backfills current state — including the `deleted` flag — before it may push, so it cannot resurrect the object); GC before the cut is non-conformant, and so is treating C's exclusion as de-authorizing its ops | construction-todo |

---

## Issuer trust, vouch, postage settlement & mixnet admission (§9.3, §9.3.1, §9.5.1, §9.7, §9.8) — `ABUSE`

Level **Core**. The load-bearing rule of the whole anti-abuse chapter is §9.3.1's: a token is worth
exactly its issuer's standing, and an unknown issuer — including the sender's own node — carries a
budget of **zero**. Without it a spammer self-issues unlimited free tokens and every other
mechanism in §9 is decoration. `-03` pins the matching rule for real money: no offline bearer
acceptance, ever. `-04` is the one worth reading twice — the vouch is the only anti-abuse tier an
adversary cannot buy with compute or money, so it is normatively a primary tier and not a
curiosity.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-ABUSE-01 | MUST | §9.3.1, §9.3 | **An unknown issuer's token is no token.** A token's granted rate budget is a function of **issuer trust at the recipient**, not of the token alone, and a token from an unknown or unvetted issuer — explicitly including the sender's own node — carries a default budget of **ZERO**, forcing fallback to PoW or postage. Self-issuance buys anonymity and no cost relief. This is the rule that makes ARC anti-abuse real rather than bypassable | construction: cold MOTE carrying a cryptographically valid ARC presentation from (a) an issuer the recipient has never seen, (b) the sender's own node as issuer, (c) an issuer in the recipient's trusted set | reject (a) and (b) → `ERR_TOKEN_ISSUER_UNTRUSTED` (0x0704) — treated as no token, so the §9.4/§9.5 fallback applies; accept (c) within its budget. Granting any budget on signature validity alone is non-conformant | construction-todo |
| DMTAP-ABUSE-02 | MUST | §9.3, §9.2a, §18.3.3 | **ARC-style per-origin binding, not plain Privacy Pass tokens.** Vanilla Privacy Pass tokens are issuance-unlinkable but do not, alone, give per-recipient rate-limiting **and** cross-recipient unlinkability simultaneously; that combination requires per-origin-scoped issuance. DMTAP tokens MUST use ARC-style per-origin binding, the presentation's `origin` MUST match the verifying node's origin, and its request context MUST bind the envelope's `sender_key` — so a stripped presentation cannot be replayed under a different ephemeral key | construction: (a) an unscoped Privacy Pass token presented as a `ChallengeResponse`; (b) a valid ARC presentation whose `origin` names a different recipient; (c) a valid presentation lifted onto an envelope with a different `sender_key` | reject all three → `ERR_TOKEN_INVALID_OR_SPENT` (0x0705) for (a) and (c), `ERR_TOKEN_ISSUER_UNTRUSTED`/origin mismatch for (b); accepting an unscoped token is the single change that collapses the anonymity-plus-accountability guarantee | construction-todo |
| DMTAP-ABUSE-03 | MUST | §9.5.1, §18.3.3 | **No offline bearer acceptance of real money.** Postage is a bearer instrument for real money, so the redeeming party MUST check the stamp's `serial` against the issuer's redemption endpoint, or accept the issuer's signed short-lived spent-list; the issuer marks the serial spent atomically and a stamp presented twice is rejected on the second redeem. If the issuer is unreachable the stamp is treated as **unverified** — falling back to token/PoW policy — never accepted on faith | construction: (a) redeem one stamp twice against a live issuer; (b) present a valid, unexpired stamp with the issuer's redemption endpoint unreachable; (c) present an expired stamp | reject (a) on the second redeem → `ERR_POSTAGE_DOUBLE_SPEND` (0x0708); (b) → `ERR_POSTAGE_ISSUER_UNREACHABLE` (0x0709) with the message falling back to the recipient's token/PoW policy rather than being accepted; (c) rejected as expired. Accepting (b) on faith is non-conformant | construction-todo |
| DMTAP-ABUSE-04 | MUST | §9.7, §18.3.3 | **Vouch is a primary tier, and it is itself rate-limited.** A vouch is the only anti-abuse mechanism an adversary cannot buy with compute or money — it requires someone the recipient already trusts to spend their own reputation — so implementations MUST offer it wherever a mutual contact exists rather than treating it as an exotic fallback, and it MUST itself be rate-limited to prevent vouch farming. The `voucher` must be a contact the recipient has pinned, and the vouch is scoped to that recipient | construction: cold sender with a mutual contact — check whether the client offers the vouch path; then drive vouch issuance past the voucher's anti-farming budget; and present a vouch whose `recipient` names a different node | accept the first vouch (cold contact admitted with no PoW or postage); reject the over-budget and mis-scoped ones → `ERR_VOUCH_INVALID_OR_RATE_LIMITED` (0x070A). A client that never surfaces the vouch path where a mutual contact exists is non-conformant | construction-todo |
| DMTAP-ABUSE-05 | MUST | §9.8, §4.4.8, §2.2b | **An entry mix rate-limits blind; it never inspects a proof it cannot read.** The recipient-facing anti-abuse proof is sealed inside the encrypted `ciphertext` and bound to the recipient, so a mix cannot verify it. Mixnet flood-abuse is therefore bounded by what a content-blind node can actually apply: a per-connection **and** per-operator rate budget at the entry mix, plus operator-level limits and the attested-operator/ASN-diversity requirements of §4.4.8. A mix that admits on a proof is either decrypting or pretending | construction: inject Sphinx cells past the per-connection budget from one source, and past the per-operator budget across several connections under one upstream operator; separately, probe whether admission varies with the sealed proof carried inside otherwise-identical cells | accept (both budgets bind, and admission is byte-identical across cells differing only in their sealed contents); a mix whose admission varies with the sealed payload is non-conformant, and one enforcing only a per-connection limit is trivially bypassed by opening more connections | construction-todo |

---

## The operator seam & the inviolable rule (§12.2, §12.3.2, §12.6) — `SEAM`

Level **Core**. The seam is the whole surface a third party's box gets, and §12.3's rule is what
keeps it small: privacy, cryptography, metadata privacy and recovery are never behind it. The
asymmetry in `-01` is the design's core move and is easy to invert by accident — the *functional*
capabilities fail **open** so an unreachable operator never breaks someone's mail, while
`GatewayAuthz` is a **security** control and fails **safe**, because failing it open turns the
gateway into the open relay §7.11.2 exists to prevent.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-SEAM-01 | MUST | §12.2, §7.11.2 | **Fail open to function, fail safe on security.** If the party behind the seam is unreachable, the implementation MUST NOT break user-facing mail, chat or files — metering and quota decisions are availability concerns, never security ones. `GatewayAuthz` is the exception and MUST NOT fail open to "allow": on operator unreachability it falls back to a **safe default**, permitting legacy egress only to already-established relationships, and a cached authorization is operator-independent and MUST NOT be accepted on faith during an outage | construction: partition the node from the seam party, then (a) send and receive native mail, chat and files, (b) attempt legacy egress for a cold/unproven sender, (c) attempt legacy egress for an already-established relationship, (d) present a cached `GatewayAuthz` for an unestablished sender | accept (a) and (c) — user-facing function is unaffected and established egress continues; reject (b) and (d) → `ERR_GATEWAYAUTHZ_DENIED` (0x070E), FAIL_CLOSED_BLOCK. Blocking (a) is non-conformant, and allowing (b) is the open relay | construction-todo |
| DMTAP-SEAM-02 | MUST | §12.2, §9.5, §3.12.5 | **Payment rails have no protocol surface, and never will.** How a third party collects money is operator policy: no seam capability, registry or extension may introduce a protocol-level payment path, and DMTAP defines no token. Postage (§9.5) is a signed real-money voucher and an anti-abuse mechanism a recipient MAY choose to accept — never a rail the protocol depends on. The check is structural: two implementations must interoperate fully with **no** agreement on payment of any kind | construction: two independently-configured deployments — one whose operator charges for legacy egress, one with no operator at all — exchange native mail, chat and files in both directions and complete an identity/KT/recovery cycle; separately, scan the seam surface and the §21 registries for any payment-, currency- or token-typed field | accept (full interop with no shared payment configuration, and no payment-typed surface anywhere in the protocol); any capability whose absence blocks native interop is a payment rail by another name and is non-conformant | construction-todo |
| DMTAP-SEAM-03 | MUST | §12.3.2, §12.3, §12.3.1, §7.14 | **No operator relationship, no functional difference.** A conformant implementation MUST NOT expose any control that violates §12.3.1, MUST NOT consult the seam for any privacy or crypto decision, and MUST NOT ship a build in which any §12.3.1 item is disabled, degraded or delayed by the absence of an operator relationship. A user with no operator relationship at all MUST observe **no functional difference** on any of the six never-chargeable categories | construction: differential — the same build run (a) with a fully-configured operator and (b) with none; exercise every §12.3.1 category (identity and keys, encryption, metadata privacy, recovery, access to one's own mailbox and export, native delivery) and compare behaviour and timing. Companion: trace every seam call made during a privacy or crypto decision | accept (byte-identical capability across (a) and (b), with no delay or degradation attributable to the missing operator, and **zero** seam calls on any privacy/crypto path); any divergence, or any seam consultation during a crypto decision, is non-conformant | construction-todo |
| DMTAP-SEAM-04 | MUST | §12.6, §3.10.2, §18.4.7 | **The org controls the name and the operations, never a sovereign member's key.** Org administration rides the seam's Provisioning and Policy hooks, but a conformant operator MUST NOT offer a control that escrows a **sovereign** account's key, or that presents an org-managed account as sovereign. Org-managed escrow is a disclosed per-account arrangement carrying the `custody = "org-managed"` marker — it hides nothing and disables nothing — whereas an escrow hook reaching a sovereign account would be exactly the backdoor §12.3 forbids | construction: enumerate every admin control the deployment exposes over (a) a sovereign member and (b) an org-managed member; attempt key escrow, key read and lockout against (a); and present an org-managed account's directory entry with the `custody` marker stripped | accept (admin power over (a) covers the name and operations only — no control reaches the key); reject the marker-stripped entry → `ERR_ORG_MANAGED_UNDISCLOSED` (0x0115), HALT_ALERT. Any exposed control that escrows or reads a sovereign key is non-conformant regardless of whether it is used | construction-todo |

---

## Roles at scale — gateway fleet, thin clients, buffers & status pages (§14.2, §14.3, §14.3a, §14.5, §14.6.2, §14.6.3) — `SCALE`

Level **Core**. §14's normative content is mostly about what must **not** become load-bearing. A
buffer must not be one volunteer's uptime — the Mastodon measurement study puts mean instance
downtime at 10.95% and permanent death within 15 months at 21.3%, so a single-holder buffer is
roughly an order of magnitude worse than the centralized service it replaces. A push must not be
delivery. A status page must not be the directory authority §4.4.2 deleted, and must not be a
telemetry funnel: aggregate counts of *infrastructure*, never of *behaviour*.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-SCALE-01 | MUST | §14.3, §4.9 | **Push is a latency optimization, never delivery.** The client MUST still poll and reconcile on foreground, and DMTAP MUST NOT treat a push as delivery confirmation — platform push is best-effort on both APNs and FCM, so a design that treats a wake as an ack has adopted a delivery guarantee neither platform makes. Push carries no content: it is wake-and-fetch, never deliver-in-push | construction: deliver to a push-woken thin client with the push path silently dropped; bring the client to foreground; separately, inspect whether any sender-side state advances on the wake alone, and inspect the wake payload for content | accept (the message is reconciled on foreground despite the lost wake, no sender-side delivery state advanced on the wake, and the wake carries only the sealed content-free token); treating a wake as delivery is non-conformant, and content in the wake rejects → `ERR_WAKEPING_CONTENT_PRESENT` (0x0313) | construction-todo |
| DMTAP-SCALE-02 | MUST | §14.3a, §14.1, §16.6 | **The buffer is an `n`-of-`m` role, not a hosted service.** A conformant node offers its ciphertext to `m` holders — peers, **the owner's own other devices**, optionally a third party — and treats delivery as buffered only when `n` of them acknowledge custody. No single holder is load-bearing, and a holder that vanishes costs redundancy rather than mail. Holders are content-blind, so raising `m` adds no trust; single-holder peer buffering is the degenerate `n = m = 1` case, which is exactly the measured failure mode | construction: configure `m ≥ 3` holders including one of the owner's own devices, require `n = 2` acknowledgements, then kill one holder permanently and one temporarily and drain the queue on the recipient's return | accept (buffered only after `n` custody acknowledgements; no mail lost to either holder failure); reject → `ERR_OFFLINE_BUFFER_UNAVAILABLE` (0x0309) only when fewer than `n` holders can be reached at all — an implementation that treats a single holder's ack as buffered cannot pass | construction-todo |
| DMTAP-SCALE-03 | MUST | §14.5, §7.4, §7.1 | **A relay is a hop, a gateway queue is not a store, and durability lands at the edge.** The gateway's short queue is only the legacy-translation hop and MUST NOT become a store; the buffer role is not a gateway function. Relay-mailbox TTL loses undelivered mail, so durability MUST land at the recipient's edge once fetched, and senders own the retry queue. The honest limit MUST be disclosed: the public libp2p/IPFS DHT and relay path is designed for brief hole-punch assistance, not sustained mailbox sync, and is unsuitable as the sole production discovery/relay path | construction: inspect the gateway for any persistent per-message store beyond the §7.4 window; fetch buffered ciphertext at a recipient and then destroy the buffer, checking the message survives at the edge; and review the deployment's documentation for the DHT-suitability disclosure | accept (no gateway store, message durable at the recipient after fetch); a gateway holding a long-lived retry queue is non-conformant, as is a deployment relying solely on the public DHT without disclosing the limit | construction-todo |
| DMTAP-SCALE-04 | MUST | §14.6.2, §14.6.3, §4.4.2, §4.4.9 | **A status page is a derived observation, never a registry — and never telemetry.** No protocol behaviour may depend on trusting one: a client MUST derive its own profile determination from its own derived fleet view, and MAY use a status page only as human-facing corroboration. A status page that becomes load-bearing has become the directory authority §4.4.2 deleted. It MUST NOT deanonymize a user or reveal any part of the social graph, message-volume and latency series are out of scope — they are exactly the timing corpus the mixnet exists to deny — and an implementation MUST NOT emit per-user telemetry to any such service, nor make participation in measurement a condition of anything | construction: present a status page asserting a Standard-capable fleet while the client's own derived view supports only Bootstrap (and the inverse); separately, trace every outbound report an implementation makes to a measurement service, and disable measurement entirely | accept (the client's profile determination follows its **own** derived view in both directions, and disabling measurement changes nothing else); a client that adopts the status page's answer is non-conformant, and any per-user or per-mailbox datum in an outbound report is non-conformant on its own | construction-todo |
| DMTAP-SCALE-05 | MUST | §14.2, §7.3 | **The gateway fleet is shared-nothing, and its one piece of shared state is disclosed.** An operator running the gateway role at volume MUST scale as a shared-nothing worker fleet: identical interchangeable instances with no whole-cluster coordination on the hot path, egress IPs owned by an egress layer rather than by instances, DKIM keys distributed read-only and never generated per instance. IP reputation and rate counters are the one thing a shared-nothing fleet cannot make node-local — and that irreducibly shared mutable state MUST be disclosed rather than presented as if the fleet were stateless throughout | construction: add and remove instances under load and observe whether any hot-path operation requires cluster coordination; inspect whether any instance generates its own DKIM key or owns its egress IP; review the operator documentation for the IP-reputation shared-state disclosure | accept (capacity scales by adding instances, no per-instance DKIM generation, egress IPs managed independently of instance count, and the shared reputation tier disclosed); a fleet whose instances generate DKIM keys or own egress IPs is non-conformant | construction-todo |

---

## Hybrid-suite composition (§10.7.1, §1.3, §16.7) — `HYBRID`

Level **Core**. Suite `0x02` is the v0 originating suite and it is hybrid, so the rule that keeps
it meaningful is AND-composition: a verifier that **supports** the hybrid suite must require every
component signature. Accepting an object whose PQ component is missing while the classical one
validates is an intra-suite strip — the downgrade that needs no negotiation and leaves no trace in
the suite byte.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-HYBRID-01 | MUST | §10.7.1, §1.3, §16.7 | **A hybrid verifier requires every component.** A hybrid-suite (`0x02`) object whose PQ signature component is missing or fails, while only the classical component validates, presented to a verifier that **supports** the hybrid suite, is an incomplete/downgraded hybrid: a hybrid verifier MUST require every component signature (AND-composition) and MUST use the X-Wing KEM combiner. Single-component acceptance is for a genuinely legacy verifier only, at that component's lower assurance | construction: a `0x02` object with its PQ signature component stripped (variant: present but invalid), presented to (a) a verifier supporting `0x02`, (b) a verifier supporting only `0x01` | reject at (a) → `ERR_HYBRID_SUITE_INCOMPLETE` (0x0210), FAIL_CLOSED_BLOCK; (b) may accept at the classical component's lower assurance, and MUST NOT report that as hybrid-strength verification | construction-todo |

---

## State-machine totality — the [fill] rules §1–§16 leave open (§20.1, §20.3, §20.5.2, §20.6, §20.7) — `FSM`

Level **Core**. §20 restates the protocol's dynamics as **total** machines: for every machine,
every state and every applicable event, the next state and required action are defined. Where the
body of the spec settles a transition, §20 restates it and the owning clause governs (§10.4). Where
it does **not**, §20 makes a conservative normative choice and marks it **[fill]** — and those
choices are owned here, tested nowhere else, and are exactly the transitions two implementations
would otherwise resolve differently. The cases below pin the [fill] rows that have observable
consequences.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-FSM-01 | MUST | §20.1, §4.7, §2.6 | **A late `ack` corrects the display and reverses nothing.** §4.7 and §2.6 do not address an `ack` arriving after the sender has already given up — a peer-buffer drain (§14.5) can deliver long after `EXPIRED`. §20.1 makes the choice: the sender MUST treat it idempotently, surfacing a "delivered late" correction, and MUST NOT re-send and MUST NOT re-open the deadline. The protocol outcome does not reverse; only the user-facing status corrects | construction: drive a MOTE to `EXPIRED` at the §16.1 deadline, then deliver a valid `ack` for its `id` from a draining buffer; observe the state, any outbound dispatch, and the timer | accept (state remains `EXPIRED`, no re-send, no re-opened deadline, and a "delivered late" correction surfaced); re-entering `RETRY`, re-arming the deadline, or discarding the late `ack` without correcting the display are all non-conformant | construction-todo |
| DMTAP-FSM-02 | MUST | §20.3, §3.3, §3.4 | **Transient and definitive resolution failures are different states.** §3.3's three named outcomes are about *KT* unreachability, not DNS itself, so §20.3 fills the gap: a transient failure (timeout, SERVFAIL, no connectivity) is **not an answer** and bounces to `UNRESOLVED` for the caller's retry schedule, while a definitive negative (authoritative NXDOMAIN, or a zone publishing no `_dmtap` record) is terminal for the attempt and surfaces "name not found" — it MUST NOT be auto-retried as though it were a fault. Separately, a chain that does not validate goes straight to `FAIL_CLOSED_BLOCKED` rather than being retried, and a dismissed `SECURITY_ALERT` keeps routing to the **old** pinned key | construction: resolve a name under (a) SERVFAIL, (b) authoritative NXDOMAIN, (c) an `Identity` whose signature chain does not validate, (d) a key change with an invalid chain that the user then dismisses; observe the state, the retry behaviour and which key subsequent messages route to | (a) retries on the caller's schedule; (b) reject → `ERR_NAME_RESOLUTION_FAILED` (0x0109), terminal for this attempt and not auto-retried; (c) `FAIL_CLOSED_BLOCKED`, not retried; (d) the **old** pinned key remains in use and the new key is never adopted without explicit action | construction-todo |
| DMTAP-FSM-03 | MUST | §20.6, §13.4, §16.8 | **The grace timer runs monotonically, and one bad proof is one bad request.** A re-validation attempt during the grace window that finds the endpoint still unreachable MUST NOT restart the grace timer: it runs from the *first* entry into `REVALIDATION_GRACE`, so `grace_window_elapsed` fires on schedule however many unreachable re-checks occur. Otherwise an attacker who keeps the endpoint down extends a revoked session indefinitely — by making the check fail more often. §20.6 also fills what §13.4 leaves open: a single failed DPoP proof rejects that request and keeps the session alive, because DPoP is per-request by construction | construction: partition the RP from the status endpoint and issue re-validation attempts at short intervals across the whole grace window, recording when `grace_window_elapsed` fires; separately, send one request with a bad DPoP proof followed by one with a good proof | accept (the window expires exactly 2× the re-validation interval after **first** entry, regardless of the number of failed re-checks; the bad proof rejects → `ERR_DPOP_PROOF_INVALID` (0x0506) while the following good-proof request succeeds); a restarted grace timer is non-conformant, and so is tearing down the session on one bad proof | construction-todo |
| DMTAP-FSM-04 | MUST | §20.5.2, §16.8, §5.1 | **A quorum below `> n/2` MUST NOT rotate the committer.** A takeover Commit promoting the deterministic successor is applied only when it references the last agreed log head **and** carries a `> n/2` roster quorum of member signatures. Below that, rotation is split-brain: two partitions each electing their own successor produce two committers, which is the fork the whole ordering layer exists to prevent | construction: partition a group so that a minority observes the committer-liveness timeout (§16.8) twice; have it assemble a takeover Commit carrying exactly `⌊n/2⌋` signatures against the last agreed head | reject → `ERR_GROUP_POLICY_VIOLATION` (0x0409); the rotation does not occur and the minority holds. Applying a sub-quorum takeover is non-conformant, and a competing Commit at the same log position is fork evidence → `ERR_COMMITTER_FORK_DETECTED` (0x0404), HALT_ALERT | construction-todo |
| DMTAP-FSM-05 | MUST | §20.7, §14.3, §4.9 | **A wake without connectivity is a benign event, and a thin client still reconciles.** For a mobile thin client `ONLINE` is transient: the client MUST still poll and reconcile rather than treating reachability as continuous. A push wake arriving with no connectivity — airplane mode — returns to `OFFLINE` and is expected, not an error, because push is wake-and-fetch and never delivery confirmation; and repeated futile wakes SHOULD be coalesced rather than driving repeated reconnect attempts | construction: deliver a wake to a thin client in airplane mode, then restore connectivity without a further wake; observe whether the queue drains on foreground and whether any error state or delivery confirmation was produced by the failed wake | accept (the failed wake yields no error and no delivery signal; the queue drains on the client's own foreground reconcile); a client that only fetches on a successful wake will lose mail whenever the platform drops one, which both APNs and FCM do by design | construction-todo |

---

## Forward compatibility — the three unknown-value rules (§21.20, §21.21, §21.25) — `FWDCOMPAT`

Level **Core**. §21.25 restates the three unknown-value rules together **precisely because they
differ**, and conflating them is how a protocol acquires a flag day. An unknown `suite` is a
security-critical trust decision and is rejected outright; an unknown `kind` is not acknowledged
but may be ignored; an unknown `ext` key is ignored and MUST NOT affect validation of the rest of
the object at all. An implementation that hardens all three into "reject" is as non-conformant as
one that softens all three into "ignore" — the first cannot roll out a new kind without a flag day,
the second will accept a suite it cannot verify.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-FWDCOMPAT-01 | MUST | §21.25, §10.1, §21.20 | **Three unknown values, three different dispositions.** Unknown `suite` → **rejected**, fail-closed, no processing (`0x0101`/`0x0201`); unknown `kind` → **not acknowledged**, but MAY be silently ignored rather than rejected (`0x020A`); unknown `Headers.ext` key → **ignored**, and MUST NOT affect validation of the rest of the object. Getting any one of the three wrong in either direction is a conformance failure, which is why they are tested as one differential rather than three isolated rejects | construction: three otherwise-valid objects differing only in one unknown value — (a) `Envelope.suite = 0x7E` (unregistered), (b) `kind = 0x5A` (unassigned in the extension range), (c) a `Headers.ext` key `future-priority` the receiver does not implement | (a) reject → `ERR_UNKNOWN_VERSION_OR_SUITE` (0x0201), DROP_SILENT; (b) not acked → `ERR_KIND_UNKNOWN` (0x020A), IGNORE_NO_ACK, and MUST NOT be rejected as malformed; (c) accept — the object validates and delivers with the unknown key ignored. Rejecting (c), acking (b), or processing (a) are each independently non-conformant | construction-todo |
| DMTAP-FWDCOMPAT-02 | MUST | §21.20, §21.21 | **Unknown keys are ignored in `ext` and in DNS alike, and `x-` keys are never assumed portable.** A receiver MUST ignore any `Headers.ext` key it does not recognize, and an unrecognized key MUST NOT cause validation failure or message rejection; the same forward-compatibility rule applies to `_dmtap`/`_dmtap-gw`/`_dmtap-mix` TXT/SVCB parameters, which resolvers MUST ignore when unrecognized. `x-`-prefixed `ext` keys are Private Use and MUST NOT be assumed portable across implementations — an implementation that depends on a peer honouring its own `x-` key has invented a private protocol and called it DMTAP | construction: (a) a `_dmtap` TXT carrying `v=`, `ik=`, and an unrecognized `futureparam=`; (b) a MOTE carrying `x-vendor-hint` in `Headers.ext` sent to an implementation that does not know it; (c) the same implementation's behaviour when its own `x-` key is absent from a peer's reply | accept all three: (a) resolves normally with the unknown parameter ignored, (b) validates and delivers with the key ignored, (c) proceeds without the `x-` key. Failing resolution on (a), rejecting (b), or degrading function on (c) are each non-conformant | construction-todo |
| DMTAP-FWDCOMPAT-03 | MUST | §21.25, §9.3.1, §12.3 | **Two things an extension may never do.** Beyond ordinary interoperability review, every new challenge type, resolver type, DNS parameter or capability token is checked for two DMTAP-specific properties: (a) it MUST preserve the issuer-trust / zero-default-budget rule and MUST NOT create a path for a sender to manufacture its own unlimited cost relief; (b) **no** extension may be gated behind an operator's paid seam in a way that weakens privacy, cryptography or metadata protection — §12.3 binds extensions exactly as it binds the base protocol. And once allocated, a code point, key or tag MUST NOT be reused for a different meaning, even if the original allocation is abandoned | construction: three candidate extensions — (a) a challenge type whose proof is self-issued and self-scored, (b) a resolver type whose privacy-relevant verification step is reachable only through a seam capability, (c) a registration re-using a deprecated code point with new semantics | reject all three (extension review fails): (a) manufactures unlimited cost relief and voids §9.3.1, (b) puts a privacy decision behind the seam and voids §12.3, (c) breaks the append-only discipline that makes the registries — like KT (§3.5) and the committer log (§5.1) — trustworthy | construction-todo |

---

## Publication consent, fetcher verification & the not-blind holder (§22.2.4, §22.5.1, §22.6.1, §22.9) — `PUBGUARD`

Level **Core**, optional capability **`pub-1`**. DMTAP-PUB's whole risk is that plaintext content
addressing is a **confirmation oracle**: anyone holding a candidate file can test whether it exists
in the public set. §22.2.4 answers that head-on — a `PubManifest` is derived only from content the
user explicitly published, and nothing is plaintext-addressed, announced or served except as the
result of that act. A node that content-addresses the whole store for deduplication has built the
oracle without ever publishing anything. `-03` is `manual-attestation`: whether serving was an
operator's choice or a default is a deployment fact, not a wire fact.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-PUBGUARD-01 | MUST | §22.2.4, §22.9, §22.7 | **Nothing is plaintext-addressed except as the result of an explicit publish act.** A `PubManifest` MUST NOT be derived from content the user has not explicitly published, and an implementation MUST NOT plaintext-address, announce or serve any object except as the result of that act. The attack this closes is confirmation: plaintext content addressing lets anyone holding a candidate file test whether it is in the public set, so deriving public addresses over private content — for deduplication, indexing or convenience — hands out that oracle for objects the user never chose to publish | construction: a store holding private objects and one published object; enumerate every plaintext content address the node computes, holds or serves, including any dedup index, thumbnail pipeline or convenience cache | accept (exactly one plaintext-addressed object — the published one); any plaintext address derived over unpublished content is non-conformant even if it is never served, because holding it is what makes the oracle answerable | construction-todo |
| DMTAP-PUBGUARD-02 | MUST | §22.5.1, §22.2.2, §22.3.3 | **Verification is the fetcher's job, always.** A fetcher MUST verify signatures and content addresses itself. The PUB HTTP endpoint is a convenience for serving bytes, not an authority over them: a server that returns a well-formed response has said nothing about whether the bytes are the ones the author signed, and a client that trusts the transport has made the serving node into exactly the trusted third party the design removes | construction: a PUB endpoint serving (a) chunk bytes that do not hash to the listed `h_i`, (b) a `PubAnnounce` whose `sig` does not verify under `signer`, (c) a `PubManifest` whose recomputed DS-tagged Merkle root differs from its `id` — each returned with a `200` and correct content type | reject → `ERR_PUB_CHUNK_HASH_MISMATCH` (0x090A), `ERR_PUB_ANNOUNCE_SIG_INVALID` (0x0904) and `ERR_PUB_MANIFEST_HASH_MISMATCH` (0x0909) respectively; accepting any of them on the server's `200` is non-conformant | construction-todo |
| DMTAP-PUBGUARD-03 | MUST | §22.6.1, §6.6 | **Serving public objects is an explicit operator choice, never automatic.** A node advertising `pub-1` and serving public objects is **not content-blind** for what it serves — unlike every other holder role in DMTAP, which handles ciphertext it cannot read. Because that changes what the operator knows and what they may be asked about, it MUST be an explicit operator choice and MUST NOT be switched on by default or as a side effect of enabling something else | manual attestation (deployment review: whether `pub-1` serving is off until an operator turns it on, whether enabling any adjacent feature turns it on implicitly, and whether the operator is told they are no longer content-blind for what they serve) | non-conformant if public-object serving is on by default, is enabled as a side effect of another setting, or is presented without the not-content-blind disclosure | manual-attestation |

---

## Assembly structure (§23.6.2) — `CADASM`

Level **Core**, optional capability **`pub-1`**. `AssemblyStructure` is the one CAD object with
structural invariants an implementation can get wrong quietly: an assembly with no children is a
part wearing the wrong kind, and a child with `quantity = 0` is a child that should have been
omitted. The `ref_kind` distinction is the load-bearing one — `pin` names a `manifest_root` and is
immutable, `track` names a `pub_announce` id and follows the author's revision chain.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-CADASM-01 | MUST | §23.6.2, §23.6.1 | **An assembly has children, each with a `ref_kind`, a `ref` and a `quantity ≥ 1`.** `children` is REQUIRED with at least one entry — an assembly with zero children is malformed for this profile and should have been a `part`-kind artifact. Every `AssemblyChild` carries `ref_kind` (`1` pin, `2` track), a `ref` that is a `manifest_root` when pinned and a `pub_announce` id when tracked, and a `quantity ≥ 1`: a quantity of `0` is expressed by omitting the child, never by a zero count | construction: `AssemblyStructure`s with (a) `children = []`, (b) a child with `quantity = 0`, (c) a child with `ref_kind = 1` whose `ref` is a `pub_announce` id, (d) a child missing `ref_kind`, (e) a well-formed assembly with two children at quantities 1 and 4 | reject (a)–(d) → `ERR_MALFORMED_OBJECT` (0x020D), DROP_SILENT; accept (e). Coercing `quantity = 0` to `1`, or guessing `ref_kind` from the shape of `ref`, is non-conformant — the two reference modes have different revision semantics and a guess silently changes which bytes the assembly names | construction-todo |

---

## Retrieval hints, migration & attestation badges (§24.4.5, §24.14, §24.17) — `VIDMIG`

Level **Core**, optional capability **`pub-1`**. Two rules here, and both are about not letting a
server's convenience become a claim about authorship. A retrieval `Hint` is advisory — bytes fetched
from an unlisted source verify the same way, because verification is against the signed root and
never against where the bytes came from. And migration is per-author and consensual by
construction: re-signing an existing record needs the **author's** key, which a PUB server holding
only plaintext never has, so a server re-attesting old content is vouching for it, not proving
authorship — and must be rendered as visibly weaker.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-VIDMIG-01 | MUST | §24.4.5, §22.5.1 | **Hints are advisory, and unrecognized hint types are ignored.** A client MUST NOT treat a blob fetched from an unlisted source differently from one fetched from a listed hint — verification is against the signed rendition root, so provenance of the *bytes* is irrelevant once they verify. And new transports are new hint types: an unrecognized type MUST be ignored, never rejected, or every new transport becomes a flag day | construction: fetch identical, correctly-verifying rendition bytes from (a) a listed hint and (b) an unlisted source, comparing the client's acceptance and any trust marking; and supply a `Hint` of an unrecognized type alongside a usable one | accept (identical treatment for (a) and (b), and the unknown hint type ignored while the usable one is used); marking (b) as less trusted, or rejecting the object because one hint type is unknown, is non-conformant | construction-todo |
| DMTAP-VIDMIG-02 | MUST | §24.14, §24.8, §22.3.3 | **No laundering authorship through attestation, and history stays dual-format.** Migration is per-author and consensual: re-signing an existing record requires the author's key, which a PUB server, archive or migration tool holding only plaintext never has for a self-custodied identity — so there is no bulk operator-run rewrite. Pre-migration history stays valid in its original bytes and a reader's client MUST retain the ability to verify **both** formats for as long as pre-migration content exists. A server that re-attests old content under its own key is asserting **server reputation, not authorship**, and MUST render it with a visibly weaker badge; a UI surfacing attestation provenance MUST distinguish the two cases visibly, not only in metadata a reader has to go looking for | construction: a feed mixing author-signed records and server-attested pre-migration records; inspect whether both formats still verify and how each is rendered; variant: a server-attested record presented with the same badge as an author-signed one | accept (both formats verify and the two provenances are visibly distinct in the surface itself); rendering a server attestation identically to an author signature is a security misrepresentation and is non-conformant, as is dropping the ability to verify the pre-migration format | construction-todo |
| DMTAP-VIDMIG-03 | MUST | §24.17, §24.8 | **A similarity relation is evidence, never truth.** The `similarity` relation type (`20`) carries a near-duplicate **claim**, and a PUB server MUST NOT auto-merge on it alone. Near-duplicate detection is a heuristic over bytes an adversary chooses; merging two authors' records on it collapses two identities into one on the strength of a guess, and the §24.8 rule that a signed tally is worth exactly the attester's reputation applies here in its sharpest form | construction: two independently-authored records with a `similarity` claim asserted between them by a third party; drive the server's indexing and inspect whether the records remain distinct objects of record | accept (both records remain distinct and independently addressable; the claim is presented with its provenance); auto-merging, deduplicating or suppressing either record on the claim alone is non-conformant | construction-todo |

---

## Wire objects with no vector — decode & cross-field rules (§18.3.4, §18.4.3–§18.4.6, §18.6.1, §18.6.2, §18.7.1, §18.7.2) — `WIRE`

Level **Core**. Most of §18 is exercised byte-exactly by `vectors.json` — `Identity`,
`DeviceCert`, `Payload`, `Envelope`, `Manifest`, `MixNodeDescriptor`, `MixDirectory`,
`DomainDirectory`, the deniable objects, the KT objects, `CapabilityToken` and the Sphinx framing
all have committed KATs, and `conformance/scope.json` classes those sections **ENCODING** for
exactly that reason. Nine objects have **no** vector, so nothing checks their decoders at all;
these are their cases.

Each one pins two things: that a decoder rejects an instance missing any REQUIRED field
(`ERR_MALFORMED_OBJECT`, `0x020D` — the §18.1 canonical-CBOR schema check the `CBOR` family
already exercises for its own objects), and — more interestingly — the **cross-field** rules that
a per-field schema check cannot express, because those are the ones a hand-written decoder gets
wrong: a `suite` that must equal the enclosing `Envelope.suite`, a threshold that must be ≥
another threshold, an echoed field that must equal the value it echoes, a `scope` that must be
`[]` rather than absent inside a signed preimage. Each becomes a KAT the moment its object gains a
fixed-input vector.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-WIRE-01 | MUST | §18.3.4, §5.3, §1.3 | **`KeyPackageRef.suite` MUST equal `Envelope.suite`.** The reference names the specific one-time KeyPackage consumed to initiate the session, and per-message suite is negotiated at KeyPackage granularity — so a reference advertising one suite inside an envelope declaring another is a negotiation the two ends did not agree on. `ref` and `suite` are both REQUIRED; `loc` is an informational hint and its absence changes nothing | construction: `Envelope` at `suite = 0x02` carrying a `KeyPackageRef` with (a) `suite = 0x01`, (b) `ref` absent, (c) `loc` absent and everything else well-formed | reject (a) → `ERR_SUITE_DOWNGRADE` (0x020F) where the referenced suite is below the pinned mark, otherwise `ERR_MALFORMED_OBJECT` (0x020D) for the mismatch; reject (b) → `0x020D`; accept (c) | construction-todo |
| DMTAP-WIRE-02 | MUST | §18.4.3, §5.3, §1.3 | **A fetched bundle MUST hash to `KeyPackageBundleRef.id`, and its `suites` MUST be a subset of `Identity.suites`.** `loc` and `id` are REQUIRED: the locator says where to fetch and the content address says what counts as the answer, so a reference without the pin is an invitation to serve anything. Where the OPTIONAL `suites` is present it MUST be a subset of the identity's own advertised suites — a bundle claiming a suite the identity does not is claiming a capability the identity never signed for | construction: (a) fetch a bundle whose bytes do not hash to `id`; (b) a `KeyPackageBundleRef` with `id` absent; (c) `suites = [0x02, 0x03]` against an `Identity.suites = [0x02]` | reject (a) → `ERR_BAD_CONTENT_ADDRESS` (0x0202), DROP_SILENT; (b) → `ERR_MALFORMED_OBJECT` (0x020D); (c) → `ERR_MALFORMED_OBJECT` (0x020D) — the superset is not a richer offer, it is an unsigned claim | construction-todo |
| DMTAP-WIRE-03 | MUST | §18.4.4, §1.4 | **`RecoveryPolicy` required fields, and `rotate_threshold ≥ recover_threshold`.** `suite`, `ik`, `version`, `methods`, `recover_threshold`, `rotate_threshold`, `ts` and `sig` are all REQUIRED, and the ordering constraint is the load-bearing one: a policy whose `rotate_threshold` is **below** its `recover_threshold` lets a single recovered factor rewrite the policy that governs recovery — the escalation §1.4 rule 2 exists to prevent. `version` is monotonic and a policy at or below the pinned version is a rollback | construction: `RecoveryPolicy` instances with (a) `rotate_threshold` = 1-of-phrase and `recover_threshold` = 2-of-device, (b) `methods` absent, (c) `version` equal to the pinned version, (d) `sig` by `IK` alone over a change that removes a guardian | reject (a) → `ERR_RECOVERY_THRESHOLD_INVALID` (0x010C), FAIL_CLOSED_BLOCK; (b) → `ERR_MALFORMED_OBJECT` (0x020D); (c) → `ERR_STALE_ROLLBACK` (0x0105); (d) → `ERR_RECOVERY_WEAKENING_UNQUORUMED` (0x010E) | construction-todo |
| DMTAP-WIRE-04 | MUST | §18.4.4, §1.4 | **`RecoveryMethod` and `Threshold` shapes.** Each method carries its discriminator plus its own required fields — `PhraseMethod` a `recovery_key`, `DeviceMethod` a `device_key` and a `label`, `SocialMethod` a `guardians` list and a `threshold`. A `Threshold` is a disjunction: `any_of` is REQUIRED and each `MethodPredicate` carries a `method` drawn from `"phrase"`/`"device"`/`"social"`/`"ik"` and a `count ≥ 1` (exactly `1` for `"ik"`, which names no `RecoveryMethod`). Changing a `SocialMethod`'s guardian set MUST trigger redistribution/resharing rather than a proactive refresh, and rotating a method out MUST re-key the underlying secret rather than merely editing the list | construction: (a) a `SocialMethod` with `guardians` present and `threshold` absent; (b) a `MethodPredicate` with `method = "ik"` and `count = 2`; (c) an unknown `method` string; (d) a policy edit that drops a `PhraseMethod` and re-adds it with the same underlying secret | reject (a), (b) and (c) → `ERR_MALFORMED_OBJECT` (0x020D); (d) is non-conformant at the §1.4 rule-3 level — the list changed while the secret did not, so the evicted factor still opens the account | construction-todo |
| DMTAP-WIRE-05 | MUST | §18.4.5, §1.5 | **`KeyRotation` required fields, and `old_ik` MUST be the currently-pinned `IK`.** `suite`, `old_ik`, `new_ik`, `reason`, `ts` and `sig` are REQUIRED; `prev` and `rotate_quorum` are OPTIONAL. `sig` is by **`old_ik`** over the body, which is what proves continuity — a rotation signed by `new_ik` proves only that someone holds the key they are trying to install. Where the identity has a published `RecoveryPolicy`, `old_ik` alone does not suffice: the rotation needs either the `rotate_quorum` co-signature (path (a)) or publication plus an elapsed veto window (path (b)) | construction: (a) `KeyRotation` whose `old_ik` is not the pinned key; (b) `sig` made by `new_ik`; (c) `reason` absent; (d) an identity with a published `RecoveryPolicy` receiving a rotation with `rotate_quorum` absent and no elapsed veto window | reject (a) and (b) → `ERR_IDENTITY_CHAIN_BROKEN` (0x0104), FAIL_CLOSED_BLOCK; (c) → `ERR_MALFORMED_OBJECT` (0x020D); (d) → `ERR_KEYROTATION_UNAUTHORIZED` (0x0121), FAIL_CLOSED_BLOCK — the pin MUST NOT advance to `new_ik` | construction-todo |
| DMTAP-WIRE-06 | MUST | §18.4.6, §1.6 | **`MoveRecord` rebinds a name and never a key.** `suite`, `ik`, `from`, `to`, `ts` and `sig` are REQUIRED; `prev` is OPTIONAL. The key is unchanged by a move — that is the whole point — so contacts route by key and verify the move against the pinned `IK`, and a forged move cannot redirect them. A `MoveRecord` presenting a different `ik`, or signed by anything other than the pinned key, is not a move; it is an attempt to substitute an identity behind a name change | construction: (a) `MoveRecord` whose `ik` differs from the pinned identity key; (b) `sig` by a device key rather than `IK`; (c) `to` absent; (d) a valid move followed by a message from the pinned key at the new name | reject (a) and (b) → `ERR_MOVE_RECORD_INVALID` (0x010A), FAIL_CLOSED_BLOCK; (c) → `ERR_MALFORMED_OBJECT` (0x020D); accept (d) — the pin follows the key, and the name follows the pin | construction-todo |
| DMTAP-WIRE-07 | MUST | §18.6.1, §5.8.1, §5.8.2 | **`GroupState` is a committer-signed projection, not the authority.** All of `group_id`, `suite`, `epoch`, `committer`, `posting_model`, `membership_visibility`, `join_policy`, `roster`, `log_head`, `version`, `ts`, `committer_sig` and `group_identity` are REQUIRED; only `name` is OPTIONAL. The enumerated fields are closed sets — `posting-model`, `visibility`, `join-policy` and `role` each admit exactly the listed strings — the roster carries ≥ 1 entry including ≥ 1 `owner`, and `version` is monotonic so a state at or below the pinned version is a rollback. The authoritative ordering remains the hash-chained log: `GroupState` is what members pin, `log_head` is what it points at | construction: `GroupState`s with (a) an unlisted `join_policy` string, (b) an empty `roster`, (c) a roster with no `owner`, (d) `version` below the pinned value, (e) `committer_sig` by a member that is not `committer`, (f) `name` absent and everything else well-formed | reject (a)–(c) → `ERR_MALFORMED_OBJECT` (0x020D); (d) → `ERR_STALE_ROLLBACK` (0x0105); (e) → `ERR_GROUP_POLICY_VIOLATION` (0x0409); accept (f) | construction-todo |
| DMTAP-WIRE-08 | MUST | §18.6.2, §5.1 | **`GroupEvent` carries MLS bytes verbatim and orders them.** Every field is REQUIRED. `mls` is the opaque TLS-encoded `MLSMessage`, carried byte-for-byte — DMTAP does not re-encode it, and every handshake stays member-signed inside that blob, so the committer signs ordering only and can stall but never forge. `log_seq` is strictly increasing and `prev` chains to the previous entry (genesis uses an all-zero digest with the v0 prefix): a gap or a duplicate is a fork signal | construction: (a) a re-encoded `mls` blob that is semantically equal but not byte-identical to what was received; (b) a `log_seq` gap; (c) two events at one `log_seq` with the same `prev`; (d) a genesis event with `prev` absent; (e) `committer_sig` valid but made over a different `log_seq` | reject (a) — the member signature inside no longer verifies (`ERR_PAYLOAD_SIG_INVALID`, 0x0208); (b) → `ERR_COMMIT_ORDERING_VIOLATION` (0x0403), HOLD_RESYNC; (c) → `ERR_COMMITTER_FORK_DETECTED` (0x0404), HALT_ALERT; (d) and (e) → `ERR_MALFORMED_OBJECT` (0x020D) and a failed ordering signature respectively | construction-todo |
| DMTAP-WIRE-09 | MUST | §18.7.1, §13.3, §16.1 | **`Challenge` required fields and its validity window.** `rp_origin`, `nonce`, `issued_at`, `exp` and `aud` are REQUIRED; `scope` is OPTIONAL. `rp_origin` is the RP's true origin — phishing resistance depends entirely on binding to it and having it injected by a trusted client, never trusted from the RP by the signer. The `nonce` is single-use and valid ≤ 120 s with a replay cache of ≥ 300 s, and an assertion presented after `exp` MUST be rejected | construction: `Challenge`s with (a) `aud` absent, (b) `exp` before `issued_at`, (c) a `nonce` already in the replay cache, (d) an assertion presented 121 s after issue | reject (a) and (b) → `ERR_MALFORMED_OBJECT` (0x020D); (c) → `ERR_NONCE_REPLAYED` (0x0502), FAIL_CLOSED_BLOCK; (d) → `ERR_CHALLENGE_EXPIRED` (0x0503) | construction-todo |
| DMTAP-WIRE-10 | MUST | §18.7.2, §13.3, §18.9.8 | **`Assertion` echoes exactly, and `scope` is `[]` rather than absent inside the preimage.** `rp_origin`, `nonce`, `issued_at`, `exp`, `aud`, `from`, `sig` and `cnf` are REQUIRED; `scope` is an OPTIONAL echo that is the **empty array** when the `Challenge` omitted it. The echoed fields MUST equal the `Challenge` values and the RP MUST verify `rp_origin` is its own origin and `aud` matches it. `from` is the identity-revealing login signer — an `IK`-authorized device key that MUST resolve to the pinned `name → key` identity — and is **not** the session key: `cnf = H(session_pubkey)` is inside the signed preimage, so a captured assertion cannot be replayed with an attacker-chosen session key, and the RP MUST bind the session **only** to `cnf` | construction: (a) an `Assertion` whose `rp_origin` differs from the `Challenge`; (b) `cnf` absent; (c) `cnf` replaced after signing; (d) a `Challenge` with no `scope` answered by an `Assertion` that omits `scope` entirely rather than sending `[]`; (e) `from` a device key that does not resolve to the pinned identity | reject (a) → `ERR_ORIGIN_MISMATCH` (0x0501); (b) → `ERR_MALFORMED_OBJECT` (0x020D); (c) and (d) → the reconstructed §18.9.8 preimage differs, so the signature fails (`ERR_PAYLOAD_SIG_INVALID`, 0x0208) — an implementation that omits an empty `scope` cannot interoperate at all; (e) → `ERR_IDENTITY_CHAIN_BROKEN` (0x0104) | construction-todo |

---

## Byte-exact KATs for §18.3.8, §18.5.2 and §18.5.4 — `WIREKAT`

Level **Core**. Nine entries in [`vectors/vectors.json`](vectors/vectors.json) were generated by
the reference crate and committed, and no case dispatched on them — so three §18 objects with
byte-exact known answers sitting in the repository were, formally, untested. These cases wire them
up. **No vector is added or changed here**; the recipe, seeds and expected bytes are exactly what
was already committed, which is why every case below is `vectored` rather than `construction-todo`
and is machine-runnable today with no reference implementation.

`-01` is the one worth singling out: `Manifest.id` MUST **self-verify** — recompute the DS-tagged
Merkle root over the ordered chunk list and it must equal `id` — so a manifest is checkable
against itself before a single chunk is fetched.

| id | req | clause | checks | input | expect | status |
|----|-----|--------|--------|-------|--------|--------|
| DMTAP-WIREKAT-01 | MUST | §18.3.8, §18.9.5, §5.5 | **`Manifest.id` self-verifies.** `id` is the domain-separated BLAKE3 Merkle root over the **ordered** `chunks` hashes: leaf = `BLAKE3(0x00 ‖ h)`, node = `BLAKE3(0x01 ‖ l ‖ r)`, RFC 6962 split, prefixed `0x1e`. A recomputed root MUST equal `id`, so a manifest is falsifiable against itself before any chunk is fetched — and reordering the chunk list changes the root, which is what makes the order part of the content address rather than a convention | vector `manifest_root` (3 chunk hashes, fixed) | match `id_hex` — recompute the root over `chunks` in order and it equals `Manifest.id` | vectored |
| DMTAP-WIREKAT-02 | MUST | §18.3.8, §18.1.1 | **Canonical encoding of a `Manifest`.** Integer-keyed deterministic CBOR over the REQUIRED `id`, `size`, `chunk_sz`, `chunks` and `suite`, with key `5` **absent** — a `Manifest` is a swarm-distributed blob, so the content key travels only in the sealed `Attachment.key` (§18.3.7) and a manifest carrying key `5` is rejected (`ERR_MANIFEST_KEY_PRESENT`, `0x0808`, tested by `DMTAP-FILE-04`) | vector `cbor_manifest` | match `cbor_hex` | vectored |
| DMTAP-WIREKAT-03 | MUST | §18.5.2, §18.1.1, §4.4.2 | **Canonical encoding of a `MixNodeDescriptor`.** Integer-keyed deterministic CBOR of the IK-signed descriptor a mix publishes — the object from which every client derives its own fleet view (§4.4.2), which is why it is signed by the node and not by any directory authority | vector `cbor_mix_descriptor` | match `cbor_hex` | vectored |
| DMTAP-WIREKAT-04 | MUST | §18.9.9, §18.5.2 | **`MixNodeDescriptor.sig` preimage.** DS-tag `"DMTAP-v0/mix-descriptor\0"` ‖ `det_cbor(descriptor ∖ {7})` — the signature covers the descriptor with its own signature field removed, so the preimage is reconstructible by any verifier from the object it received | vector `mix_descriptor_sig` (node_ik seed `0x11`×32) | match `pubkey_hex` and `sig_hex` | vectored |
| DMTAP-WIREKAT-05 | MUST | §18.5.2, §18.9.9 | **The descriptor signature verifies under `node_ik`.** The companion to `-04`: the signature verifies under the descriptor's own key (§18.5.2 key 2), which is the whole trust story for a mix — a descriptor is self-asserted and its `operator` claim is what §4.4.8 requires an independent `_dmtap-mix` attestation for | vector `mix_descriptor_sig_verify` | accept (`valid = true`) | vectored |
| DMTAP-WIREKAT-06 | MUST | §18.5.4, §4.4.1 | **`SphinxCell` framing is constant-length.** 2336 B = `α`(32) ‖ `β`(240) ‖ `γ`(16) ‖ `δ`(2048). Constant length is not a layout detail: a variable-length cell would let an observer distinguish cells by size, which is precisely the correlation the mixnet exists to deny (§4.4.1's bucket ladder is the same argument one level up) | vector `sphinx_cell` | match `bytes_hex` (and `len` = 2336) | vectored |
| DMTAP-WIREKAT-07 | MUST | §18.5.4, §4.4.4 | **`RoutingCommand` framing.** 48 B = `cmd`(1) ‖ `flags`(1) ‖ `delay_ms`(4, big-endian) ‖ `next_hop`(32) ‖ `reserved`(10 = 0). The per-hop delay is inside the encrypted `β` and is what makes the mix a mix rather than a proxy; the fixed reserved tail keeps the command length constant across command types | vector `sphinx_routing_command` | match `bytes_hex` (and `len` = 48) | vectored |
| DMTAP-WIREKAT-08 | MUST | §18.5.4, §4.4.5 | **`SURB` framing.** 352 B = `first_hop`(32) ‖ `header`(288 = `α` ‖ `β` ‖ `γ`) ‖ `key_seed`(32). The SURB is what lets a node send a loop through a full path **back to itself** without revealing a return address — the mechanism `DMTAP-COVER-03`'s active-attack detection is built on | vector `sphinx_surb` | match `bytes_hex` (and `len` = 352) | vectored |
| DMTAP-WIREKAT-09 | MUST | §18.5.4, §4.4.1 | **`SphinxFragmentHeader` framing.** 16 B = `msg_id`(8) ‖ `frag_index`(2, BE) ‖ `frag_count`(2, BE) ‖ `total_len`(4, BE). Multi-cell MOTEs fragment across cells, and each cell still takes an independent path (§4.4.3) — the fragment header is what reassembles them at the far end without any hop learning the message size | vector `sphinx_fragment_header` | match `bytes_hex` (and `len` = 16) | vectored |

---

## Vector cross-reference

Every `vectored` case above maps to an existing entry in `vectors/vectors.json`
(**42 of the 68 vectors** in the file are referenced by cases). Cross-check (case → vector):

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
| `manifest_root` / `cbor_manifest` | WIREKAT-01 / WIREKAT-02 |
| `cbor_mix_descriptor` | WIREKAT-03 |
| `mix_descriptor_sig` / `mix_descriptor_sig_verify` | WIREKAT-04 / WIREKAT-05 |
| `sphinx_cell` / `sphinx_routing_command` | WIREKAT-06 / WIREKAT-07 |
| `sphinx_surb` / `sphinx_fragment_header` | WIREKAT-08 / WIREKAT-09 |

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
