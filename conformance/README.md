# DMTAP Conformance

The conformance suite is the **operational definition** of "DMTAP-compatible": an
implementation conforms at a level (Core / Private / Groups&Files / Legacy / Clients / Auth,
see spec §10) if and only if it passes the corresponding cases here.

The suite has two coupled artifacts plus this guide:

| File | Role |
|------|------|
| [`SUITE.md`](SUITE.md) | The **normative test-case catalog**: every case numbered (`DMTAP-<CAT>-<NN>`), grouped by §10.3 level, with its spec clause, input, expected result (accept / reject + §21 error code), and MUST/SHOULD. |
| [`suite.json`](suite.json) | The **machine-readable** form of the same cases (same ids) — a runner in any language drives it. |
| [`vectors/vectors.json`](vectors/vectors.json) | The **byte-exact known-answer inputs** the vectored cases dispatch on. |

`SUITE.md` and `suite.json` carry the **same 91 case ids** and MUST stay in sync. 39 cases are
byte-runnable today (33 backed by a `vectors.json` entry, 6 self-contained CBOR-reject cases whose
bytes are inline); 45 are `construction-todo` — each carries an exact construction recipe and the
expected §21 error, and becomes byte-backed as its subsystem gains a fixed-input KAT (see
[Coverage vs. deferred](#coverage-vs-deferred)).

### How a third-party implementer uses the suite

1. Parse `suite.json`. For each case, read `level`, `operation`, `expect.outcome`
   (`match` / `accept` / `reject`), and — for `reject` — `expect.error_code`/`error_name`.
2. If the case has a `vector` (or `reuses_vector`), load that entry from `vectors.json` and dispatch
   on its `operation` (see [How an implementation runs the vectors](#how-an-implementation-runs-the-vectors)).
   If it has an inline `input.cbor_hex`, feed those bytes to your deterministic-CBOR decoder.
   If it is `construction-todo`, build the fixture from the `construction` recipe.
3. Assert the outcome: a `match` case recomputes to the committed value; an `accept` case validates;
   a `reject` case fails **and** maps to the named §21 code with that code's disposition
   (DROP_SILENT / DEFER_REQUESTS / FAIL_CLOSED_BLOCK / HALT_ALERT / ACK_DEDUP).
4. You conform at a level iff every `MUST` case at that level (and every level it composes) passes.

### Level → capability mapping (spec §10.3)

| Level | Case prefixes | Requires (spec) |
|-------|---------------|-----------------|
| **Core** | `CBOR`, `ADDR`, `SIG`, `PRE`, `NAME`, `SAFE`, `SUITE`, `VAL`, `IDENT` | Identity (§1), MOTE (§2), naming + TOFU + fail-closed KT (§3), delivery + `deliver`/`ack` (§4), MLS 1:1 (§5), cold-sender challenge gating (§9) |
| **Private** | `PRIV` (+ Core) | Core + mixnet + sealed sender + cover traffic + anti-active-adversary + fail-closed no-downgrade + privacy tiers (§4.4, §6). A production mail node MUST implement Private. |
| **Groups & Files** | `GRP`, `FILE` (+ Core) | Core + MLS groups + content-addressed file transfer (§5) |
| **Legacy** | `LEG` (+ Core) | Core + gateway inbound/outbound + DKIM delegation (§7) |
| **Clients** | `CLI` (+ Core) | Core + JMAP; IMAP/POP/SMTP-submission compat RECOMMENDED (§8) |
| **Auth** | `AUTH` (+ Core) | Core + DMTAP-Auth login ceremony + origin binding + key-bound sessions (§13); OIDC bridge RECOMMENDED |

### Reference runner

The reference runner is the `dmtap-core` crate's self-checking test
(`crates/dmtap-core/tests/conformance_vectors.rs`, run with `cargo test -p dmtap-core`): it is
exactly the vector-dispatch loop below plus the drift guard of
[Provenance & drift protection](#provenance--drift-protection). It is a **proof**, not the
definition — the spec + `SUITE.md` + `suite.json` + `vectors.json` are authoritative (§10.4). A
third-party runner in any language is written directly against those four; it does not need the
reference crate.

## Byte-exact known-answer vectors

[`vectors/vectors.json`](vectors/vectors.json) contains **byte-for-byte known-answer vectors**
for DMTAP's deterministic, security-critical operations. They make the spec
**machine-verifiable**: any independent implementation can compute the same operation over the
same fixed input and check that it gets the same committed answer.

The vectors are **not hand-written** — they are computed by the reference crate `dmtap-core`
(`../../envoir/crates/dmtap-core/`) and are proven correct against it by a self-checking test
(see [Provenance & drift protection](#provenance--drift-protection)). Once published, **the
vectors — not the reference — are normative** (spec §10): a divergence between the reference and
these vectors is a bug to reconcile.

### Fixed-seed methodology

Every value is derived from a **fixed seed or fixed input** so it is exactly reproducible:

- **Ed25519** is deterministic per **RFC 8032** — a given secret seed + message always yields the
  same signature. Keys are reconstructed from fixed 32-byte seeds (`IdentityKey::from_seed`),
  never generated randomly.
- **BLAKE3-256** (content addresses, key-names, safety fingerprints) and **CBOR** (RFC 8949) are
  deterministic functions of their input.
- Two of the sign vectors reuse the **RFC 8032 §7.1 Test 1 / Test 2** secret seeds, so an
  implementation gets a free cross-check against a published external standard (the committed
  public keys `d75a9801…` / `3d4017c3…` and signatures `e5564300…` / `92a009a9…` are exactly the
  RFC's values).

**Nothing that depends on fresh randomness is vectored as a full object.** HPKE encapsulation
uses an ephemeral key from the OS CSPRNG, so a sealed `Envelope.ciphertext` is not reproducible;
instead the *deterministic sub-operations* it decomposes into — the content-address check, the
`sender_sig` preimage/signature, and the `payload_sig` preimage/signature — are each vectored as
known-answer tests.

### Vector file format

`vectors.json` is a single object: `{ format, suite, generated_by, methodology, vectors: [...] }`.
Each entry in `vectors` is:

```json
{ "name": "…", "operation": "…", "input": { … }, "expected": { … }, "note": "…" }
```

`name` is unique. `operation` names the algorithm a runner dispatches on. Byte strings are
lowercase hex. `input` fully determines the operation (for the input-driven operations); `expected`
is what a conforming implementation MUST produce.

## What the vectors prove

| # | Operation(s) | Vectors | What it proves |
|---|--------------|---------|----------------|
| 1 | `content_address` | `content_address_{empty,small,phrase,multi_kib}` | Content address = `0x1e ‖ BLAKE3-256(bytes)` (spec §2.2, §18.1.5) for empty, small, phrase, and a 4096-byte input. |
| 2 | `content_address_verify` | `mote_content_address_{ok,tampered}` | The §2.7 step-2 check: `id` recomputes from `ciphertext`, and a tampered ciphertext is rejected **before any decryption**. |
| 3 | `keyname_encode` / `keyname_verify` | `keyname_{zero_key,key_ones,key_twos,real_pubkey}`, `keyname_typo_rejected` | The zero-authority 8-word + checksum key-name (§3.9.1, §16.2): determinism, distinctness (`key_ones` ≠ `key_twos`), and that a mistyped word fails the folded checksum (fail-closed). |
| 4 | `safety_number` | `safety_number_pair_ab`, `safety_number_order_independent` | The §3.4.1 OOB safety number: a deterministic fingerprint of a **pair** of identity keys, **order-independent** (swapping the two keys yields the identical value). |
| 5 | `ed25519_sign` / `ed25519_verify` | `ed25519_rfc8032_test{1,2}`, `ed25519_domain_separated`, `ed25519_verify_{ok,tampered_msg,tampered_sig}` | Deterministic Ed25519 signatures (incl. two RFC 8032 cross-checks and DMTAP's domain-separated `sign_domain`), and that verification fails closed on a tampered message or signature. |
| 6 | `cbor_encode` | `cbor_{identity,device_cert,payload,envelope}` | Exact deterministic CBOR bytes of a signed `Identity` (+ its content address), a `DeviceCert`, a `Payload`, and an `Envelope`. The self-check additionally round-trips each (`decode(encode(x)) == x`, and re-encode is byte-identical). |
| 7 | `suite_decode` | `suite_{reject_0x00,reject_0x03,reject_0x05,reject_0xff,accept_0x01,accept_0x02}` | Suite fail-closed (§1.1, §18.1.4): an unknown suite byte MUST be rejected on decode; `0x01`/`0x02` are known ids. |
| 8 | `ed25519_sign` / `ed25519_verify` | `mote_sender_sig`, `mote_sender_sig_verify`, `mote_payload_sig` | The two MOTE signature KATs (§2.7 steps 3 & 8): the `sender_sig` preimage `id ‖ to ‖ ts_be64 ‖ kind ‖ challenge` under the ephemeral key, and the `payload_sig` preimage `BLAKE3-256(CBOR(payload with sig cleared))` under the IK — both with their exact domain-separation labels. |

The **MOTE validation ordering** (§2.7) is expressed as KATs #2 (content-address, checked before
decryption) and #8 (the two signature checks), which is the deterministic core of the ordered
recipient validation.

## How an implementation runs the vectors

1. Parse `vectors/vectors.json`.
2. For each vector, dispatch on `operation`, feed it the hex/field values in `input`, and compare
   your result to `expected`:
   - `content_address` → `0x1e ‖ BLAKE3-256(bytes_hex)` == `id_hex`.
   - `content_address_verify` → recompute the address of `ciphertext_hex`; it matches `id_hex` iff `valid`.
   - `keyname_encode` → your key-name of `pubkey_hex` == `name` (and it checksum-verifies).
   - `keyname_verify` → checksum-verifying `name` == `checksum_verifies`.
   - `safety_number` → your safety number / fingerprint of `(ik_a_hex, ik_b_hex)` == `safety_number` / `fingerprint_hex`.
   - `ed25519_sign` → `Ed25519(seed_hex).sign(domain_hex ‖ msg_hex)` == `sig_hex`, and the public key == `pubkey_hex`.
   - `ed25519_verify` → verifying `sig_hex` over `domain_hex ‖ msg_hex` under `pubkey_hex` == `valid`.
   - `suite_decode` → decoding the CBOR in `cbor_hex` as a suite id succeeds iff `accepted`.
   - `cbor_encode` → decode `cbor_hex`; re-encoding MUST be byte-identical (deterministic CBOR).

The reference runner is the crate's own self-check test, which is exactly this loop plus a drift
guard (below); use it as the executable specification of the dispatch.

## Provenance & drift protection

The vectors are generated and continuously verified by `dmtap-core`:

- **Generate:** `cargo run -p dmtap-core --example gen_vectors` recomputes `vectors.json` from the
  reference crate.
- **Self-check:** `cargo test -p dmtap-core` (test file `tests/conformance_vectors.rs`) proves,
  on every test run, that
  1. every input-determined vector **re-derives from `dmtap-core`** to its committed `expected`
     value (the vectors are correct against the reference, not guessed), and
  2. the committed `vectors.json` **byte-for-byte matches** what the current reference generates —
     so if a primitive ever changes, the test fails until the vectors are regenerated (**drift is
     caught**),

  plus CBOR round-trips and the suite fail-closed property directly.

The generator and the checker share one source of truth (`crates/dmtap-core/vectors_gen.rs.inc`),
so they cannot silently disagree.

## Coverage vs. deferred

**Covered (deterministic, known-answer):** content addressing; the key-name (encode + checksum);
safety numbers; Ed25519 sign/verify (incl. RFC 8032 cross-checks + domain separation); canonical
CBOR of `Identity` / `DeviceCert` / `Payload` / `Envelope` (+ round-trip); suite fail-closed; and
the MOTE content-address and signature KATs. **32 vectors** across 9 operations.

**Deferred, and why:**

- **HPKE seal/open (suite 0x01) and any full sealed `Envelope.ciphertext`** — RFC 9180 HPKE uses
  an ephemeral encapsulation key from the CSPRNG, so the output is non-deterministic. `dmtap-core`
  exposes no fixed-randomness HPKE path, so a byte-exact seal vector is not reproducible. Its
  deterministic decomposition (content-address + `sender_sig` + `payload_sig`) is covered instead.
  A fixed-KAT HPKE vector could be added if the reference gains a deterministic-RNG test seam.
- **MLS group operations (`GroupState`, `GroupEvent`, Welcome/Commit, §5)** — MLS handshake bytes
  (RFC 9420) are carried opaquely by DMTAP and are not implemented in `dmtap-core`.
- **Suite `0x02` (PQ: ML-DSA-65 / X-Wing)** — reserved and unimplemented; it correctly fails
  closed, which *is* vectored (`suite_reject_*` covers unknown-suite rejection; a positive
  `0x02` object KAT awaits an implementation).
- **Argon2id PoW, ARC tokens, postage, vouches (§9), key-transparency proofs (§3.5), Merkle-DAG
  manifests (§5.5), DMTAP-Auth challenge/response (§13), name→key DNS/KT resolution (§3.3)** —
  either not yet implemented in the reference or dependent on external/interactive state; to be
  added as those subsystems land.

### A note on the CBOR encoding vectored here

The `cbor_encode` vectors are the **§18-canonical, integer-keyed (COSE/CWT-style) deterministic
encoding** (RFC 8949 §4.2; spec §18.1.1 / §18.1.2 / §18.3.x / §18.4.x), matching the
`vectors.json` `methodology` field and each vector's own note. This is directly visible in the
bytes: e.g. `cbor_identity` begins `a9 01 81 01 02 a1 …` — a 9-entry map keyed by the small
unsigned integers `1, 2, 3, …` in ascending order, exactly as §18.4.1 specifies, **not** a
text-keyed map. A second implementer following §18 alone reproduces these bytes; the self-check
additionally round-trips each object (`decode(encode(x)) == x`, byte-identical) and recomputes the
`Identity` content address (§18.9.4). Cases `DMTAP-CBOR-01…04` in `SUITE.md` pin these four
objects; `DMTAP-CBOR-05…12` pin the canonical-form **reject** rules (non-preferred integers,
indefinite lengths, unsorted/duplicate keys, floats, `undefined`, null-valued optionals, and
unknown `≥ 64` keys in signed objects).
