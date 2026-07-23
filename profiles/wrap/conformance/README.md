# WRAP conformance vectors

`wrap_vectors.json` is the file `14-conformance.md` §15.1 says is **normative
and takes precedence over the prose of that document** wherever the two
disagree. This document explains its format and how to drive it against an
implementation. It is not itself normative — if this README and the JSON
disagree, the JSON wins.

> **Regenerated for v0.2.0 (2026-07-23).** This corpus was regenerated against
> the DMTAP substrate byte formats after WRAP's rebase (`CHANGELOG.md` [0.2.0],
> `14-conformance.md` §15.2/§15.3). The **retired** groups are gone — the
> bespoke two-element `[canonical_bytes, signature]` array envelope, the
> string-HLC, and the `encode`/`id`/`sign`/`hlc`/`tiebreak` groups, which are
> now the **substrate's** vectors, not WRAP's. The corpus now carries only the
> WRAP-specific groups of §15.3: `map`, `authorship`, `withdraw`, `fold`,
> `expiry`, `retain`, `forward`, `proof`. It is produced by
> [`gen_wrap_vectors.py`](gen_wrap_vectors.py) (`pip install blake3 pynacl`),
> which is reproducible and byte-deterministic (re-running emits an identical
> file).

## What a vector corpus does and does not prove (read this first)

A vector corpus proves that two engines agree **with each other**. It does not
prove either agrees with the **spec** — a shared bug in a value both the
generator and an implementation compute the same wrong way is invisible to the
corpus. The vectors are a cross-implementation interoperability net, not a
proof of correctness against the prose. Where a byte here and the spec disagree,
that is a bug to reconcile (§15.5), and the *prose intent* governs the fix even
though §15.1 makes the JSON win at the point of an interop dispute. Keep both
facts in mind: the JSON is the tie-breaker for "which bytes", the prose is the
authority for "which behaviour".

## Why hex, not a language's native types

Every byte value in the file — canonical CBOR encodings, ids, public keys,
seeds, commitments — is a lowercase hex string with no `0x` prefix. Vectors must
be reproducible by an implementation in any language, so nothing here depends on
Go's `[]byte`, Rust's `Vec<u8>`, or any other host representation.

## What the substrate fixes, and is therefore NOT here (§15.2)

WRAP objects are substrate Sync ops, Feed entries, and content objects (§7). The
byte-level behaviour they share with every other substrate product is fixed by
the **substrate's** conformance vectors, and WRAP MUST NOT re-vector it:

- deterministic-CBOR encoding proper (RFC 8949 §4.2, DMTAP §18.1.1);
- the `COSE_Sign1` envelope framing **and its signature values** (RFC 9052,
  DMTAP §18.1.6);
- the HLC and its `(wall, counter, author)` tie-break (SYNC §3);
- OR-Set / LWW / feed merge *convergence* at the byte level (SYNC §4).

Consequently this corpus emits **no COSE_Sign1 bytes and no signature values.**
It records, per object, the WRAP signing *preimage*
(`"WRAP-v0/object" ‖ 0x00 ‖ canonical_bytes`, §5.2) in `wrap_preimage_hex` for
reference, because the DS-tag is the one WRAP-specific part of the signing
construction — but the signature over it is the substrate's vector, not WRAP's.
See "gaps" at the foot of this file for the honest consequence: the exact
COSE_Sign1 framing is **not exercised** by this corpus.

## Top-level shape

```jsonc
{
  "wrap_version": 0,
  "conformance_vectors_version": "0.2.0",
  "description": "...",
  "value_typing_convention": { ... },
  "hlc_encoding_note": "...",
  "signing_note": "...",
  "keys":    { "issuer": {...}, "performer": {...}, ... },
  "objects": { "wo1": {...}, "offer1": {...}, ... },
  "vectors": [ {...}, {...}, ... ]
}
```

### `keys`

Fixed Ed25519 keypairs, each derived from a 32-byte **seed** via the standard
Ed25519 key-derivation (RFC 8032 §5.1.5; `ed25519.NewKeyFromSeed` in Go,
`SigningKey(seed)` in PyNaCl). Every seed is one byte repeated 32× (`issuer` =
`0x11`×32, `performer` = `0x22`×32, `imposter` `0x33`, `pool` `0x44`, `attestor`
`0x55`, `performer2` `0x66`) so a reader can eyeball which key is which. **Do not
reuse these keys for anything but reproducing these vectors** — the whole point
of a fixed public seed is that anyone can derive the private key. Each entry is
`{"seed": "<hex>", "pub": "<hex>"}`; `pub` is the 32-byte raw Ed25519 public key
that appears as `author`, `performer`, `subject`, `pool`, etc.

### `objects`

Fully-specified WRAP objects referenced by name from `vectors[]`. Each carries:

- `kind` / `kind_hex` — the object kind name and value (§3.1).
- `author` / `author_pub_hex` — which `keys` entry signs it, and its public key.
- `ts` — the substrate HLC as `{wall, counter, author_hex}` (SYNC §3). The
  retired string-HLC is gone; the **encoded** HLC map `{1,2,3}` lives inside
  `canonical_bytes_hex`.
- `fields` — the kind-specific fields (keys 6+) in the typed-value convention
  below, sufficient to reconstruct the object from scratch.
- `canonical_bytes_hex` — the deterministic-CBOR encoding of the full object
  (common-header keys 1,2,4,5 plus `fields`), with **key 3 (`id`) and the
  signature both excluded** (§4.3, §5.2).
- `id_hex` — the substrate content address `0x1e ‖ BLAKE3-256(canonical_bytes)`
  (§4.3; DMTAP §18.1.5).
- `wrap_preimage_hex` — the WRAP signing preimage
  `"WRAP-v0/object" ‖ 0x00 ‖ canonical_bytes` (§5.2). Reference only; no
  signature over it is emitted (§15.2).

An implementation should reconstruct `canonical_bytes_hex` from `fields` +
`author` + `ts` **independently** — that reconstruction, byte-for-byte, is the
real encode test. Do not feed `canonical_bytes_hex` straight to a decoder and
call it done; that tests decode, never encode.

### `value_typing_convention`

CBOR has more scalar types than JSON, so every typed value is a small tagged
object `{"t": ..., "v": ...}`:

| `t` | Meaning | `v` |
|---|---|---|
| `uint` | CBOR major type 0 | JSON number ≥ 0 |
| `int` | CBOR major type 0 or 1 (by sign) | JSON number, may be negative |
| `bool` | CBOR major type 7 (`0xf4`/`0xf5`) | JSON boolean |
| `tstr` | CBOR major type 3 | JSON string |
| `bstr` | CBOR major type 2 | lowercase hex, `""` for empty |
| `array` | CBOR major type 4, definite-length | JSON array of typed values |
| `map` | CBOR major type 5, definite-length, **unsigned-integer keys** | JSON object; keys are decimal strings of the uint key, values typed |
| `refmap` | The one WRAP map with **text** keys — `WorkOrder.refs` (key 14, §3.3) | plain JSON object string:string |

There is **no `float64`** in this corpus: the objects are float-free by
construction (no `Place` lat/lon), to sidestep an unresolved question — see gaps.

### `vectors`

Each vector has a stable `id`, a `group` (one of the eight in §15.3), a
`description` that says what property it pins, and an `expect` block; some also
carry `object`, `input`, or `context`. **Read `description` first.** The groups:

| Group | Fixes | Shape of `expect` |
|---|---|---|
| `map` | Each object → substrate primitive + address (§7.2) | `primitive`, `ns`, `target`, `field`, `address_hex` |
| `authorship` | The §5.5 admission table — Assignment by a non-issuer is inadmissible (never enters the LWW register, even with a higher HLC); Bid by any admitted principal accepted | `admissible`, `enters_register`/`enters_set`, `error_code`/`error_name` |
| `withdraw` | Bid withdrawal is the OR-Set observed-remove (SYNC §4.3), not a `withdrawn` flag; a withdraw cancels only its own add-tag | `present_element_ids_hex`, `cancelled_element_ids_hex` |
| `fold` | Lifecycle state computed from a work order's op set, including unreachable-state discard (§6.3) | `state`, optional `discarded_object_ids_hex` |
| `expiry` | Computed expiry with no message; and that an Assignment beats expiry (§6.2/§6.3) | `state`, `message_required` |
| `retain` | Attestations survive compaction of a terminal work order (§7.3) | `must_retain_ids_hex`, `may_discard_ids_hex` |
| `forward` | Unknown kind ignored; unknown field preserved through re-encode; unknown profile stored (§4.4, §13.3) | `action`, `is_error`, optional `reencode_must_equal_hex` |
| `proof` | Handoff commitment verifies; a wrong code fails (§10) | `commit_hex`, `verified` |

For `authorship`, `error_code` / `error_name` are the four-hex-digit code and
name from `12-errors.md` §13.1 (e.g. `"0x0202"` / `"ERR_NOT_ISSUER"`).

## Running the vectors against an implementation

There is no vector-runner binary here — WRAP is a wire format, not a library. To
validate an implementation:

1. **`map` / encode.** For every `objects[]` entry, reconstruct the object from
   `kind` + `author` + `ts` + `fields` and confirm your encoder produces
   `canonical_bytes_hex` byte-for-byte; confirm
   `id_hex = 0x1e ‖ BLAKE3-256(canonical_bytes)`; confirm each `map` vector's
   `primitive`/`target`/`field` matches how your engine routes that object into
   the substrate (SYNC §4).
2. **`authorship`.** Run your admission check (SYNC §9 admission, applied
   *before* an op enters state) and confirm the `admissible` verdict; the
   inadmissible Assignment must never reach the LWW register, regardless of its
   HLC.
3. **`withdraw`.** Apply the listed add-tags and the observed-remove to your
   OR-Set and confirm the surviving element-id set.
4. **`fold` / `expiry`.** Fold the object set with `now` and confirm the state
   and any discarded object; expiry is computed, never a message.
5. **`retain`.** Compact the terminal object set and confirm the must-retain and
   may-discard partitions.
6. **`forward`.** Confirm an unknown kind is ignored silently, an unknown field
   survives a re-encode byte-for-byte (`reencode_must_equal_hex`), and an
   unknown profile is stored/relayed/merged.
7. **`proof`.** Recompute `BLAKE3-256(code ‖ order_id)` and confirm it equals
   `commit_hex` for the right code and differs for the wrong one.

## Reproducing and independently checking this corpus

- **Regenerate:** `python gen_wrap_vectors.py wrap_vectors.json`
  (`pip install blake3 pynacl`). The generator hand-rolls its deterministic-CBOR
  encoder rather than importing one, so the bytes are portable and auditable.
- **Determinism:** re-running the generator emits a byte-identical file.
- **Independent addressing:** the corpus was cross-checked with a **second,
  different** CBOR encoder (`cbor2`, `canonical=True`) that re-encoded every
  object and recomputed every `0x1e ‖ BLAKE3-256` address to the same bytes, and
  with a fully-manual (no-CBOR-library) byte-walk of `wo1`'s map header and key
  sequence. Agreement of two independent encoders is what makes the encode bytes
  trustworthy — the same "two engines agree with each other" property this whole
  corpus provides, applied to its own construction.

## A claim of conformance

Per §15.1: **all applicable vectors pass, with no silent skips.** A WRAP
implementation that rides a conformant substrate engine inherits the substrate's
encode/id/sign/hlc/merge behaviour and is measured *here* only on the
WRAP-specific spine above. If your implementation does not cover a group (e.g. it
is a narrow encode/address binding with no fold runtime), **say which and why** —
"we do not implement lifecycle folding" is honest and acceptable; silence is not.

## Gaps — what this corpus does NOT verify (be brutal, not reassuring)

These are stated because a plausible-but-unverified vector is worse than an
honest gap.

1. **COSE_Sign1 framing is not exercised at all.** Per §15.2 the COSE_Sign1
   envelope and its signature value are the *substrate's* vectors, so this corpus
   emits none. That means the corpus does **not** prove a WRAP implementation
   frames or verifies a `COSE_Sign1` correctly — only the substrate's vectors do.
   `wrap_preimage_hex` records the DS-tagged preimage, but no signature over it
   was produced or checked here. If you need to pin the WRAP DS-tag inside a real
   COSE structure, that must be done against a real substrate engine; it was not
   available to this generator.
2. **The namespace concatenation is unpinned.** §7.2 writes
   `ns = "wrap:" ‖ WorkOrder.id`, where `id` is 33 raw bytes but `SyncOp.ns` is a
   `tstr`. Whether the id is hex-encoded, base-something, or raw bytes inside the
   text namespace is not stated by the spec, so the `map` vectors record the two
   parts **structurally** (`{prefix, order_id_hex}`) rather than assert a single
   concatenated string. This is a real spec ambiguity, not a vector choice.
3. **Floats are avoided, not solved.** `Place.lat`/`lon` are `float` in §3.9, but
   DMTAP §18.1.1 rule 4 says floats MUST NOT appear in wire objects, and
   deterministic float encoding (RFC 8949 §4.2.2) is a separate hazard. The
   objects here are float-free so the ambiguity is dodged; a corpus that needs to
   pin `Place` bytes must first resolve whether WRAP content objects may carry
   floats at all. Flagged, not decided.
4. **`proof` order-id width is a choice.** §10.2 says
   `commit = BLAKE3-256(code ‖ order_id)`; the corpus uses the **full 33-byte
   `id`** (the `0x1e`-prefixed value) as `order_id`. The spec does not say whether
   the prefix is included, so an implementation using the bare 32-byte digest
   would compute a different commitment. The vector documents the choice; the spec
   should pin it.
5. **Merge convergence is asserted structurally, not by running a substrate
   engine.** `withdraw`/`fold`/`retain` describe the *expected result* of applying
   ops; they were not produced by executing a real SYNC engine (none was on hand),
   so they verify WRAP's mapping/fold rules but inherit the substrate's actual
   merge only by reference. If a real engine disagrees with a listed present-set,
   the engine and this corpus must be reconciled (§15.5).
6. **Only interop, never correctness.** As stated up top: everything here proves
   engines agree with each other, never that they agree with the prose.
