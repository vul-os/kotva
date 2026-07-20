# 4. Wire format

## 4.1. Encoding

Objects are encoded as **CBOR** (RFC 8949) maps with **unsigned integer keys**.

Integer keys are chosen over text keys for three reasons: they are compact on
constrained links, they are stable under translation and refactoring, and they
force a registry discipline — a new field requires allocating a number, which
is a visible act, rather than inventing a string, which is not.

Implementations MUST use **deterministic encoding** as defined in RFC 8949
§4.2.1:

- map keys sorted in ascending numeric order;
- shortest-form integer encoding;
- definite-length arrays and maps only;
- no duplicate keys;
- no indefinite-length strings.

Two implementations encoding the same object MUST produce byte-identical
output. This is not a stylistic preference: the object identifier and the
signature are both computed over these bytes, so any encoding freedom is a
correctness bug.

## 4.2. Versioning

Key 1 (`v`) is the format version and MUST be present. This document defines
version `0`.

An implementation receiving an object with a `v` it does not support MUST
ignore the object. It MUST NOT attempt best-effort interpretation: a work order
whose semantics have changed is worse than a work order that was never seen.

## 4.3. Object identifier

```
id = 0x1e ‖ BLAKE3-256( canonical_bytes )
```

where `canonical_bytes` is the deterministic CBOR encoding of the object map
with key 3 (`id`) and the signature omitted, and `0x1e` is the multihash prefix
for BLAKE3-256.

The identifier is therefore self-verifying: a receiver recomputes it and
rejects any mismatch with `ERR_BAD_ID` (§13). Content addressing gives
deduplication for free — the same object relayed by three peers is stored once.

## 4.4. Unknown fields and unknown kinds

Two rules, both load-bearing for forward compatibility:

**Unknown keys MUST be preserved and ignored.** An implementation that receives
an object containing keys it does not understand MUST NOT drop them when
re-encoding or relaying, because doing so would invalidate the signature and
silently corrupt the object for everyone downstream. It MUST NOT reject the
object either.

**Unknown kinds MUST be ignored silently.** Not acknowledged, not rejected, not
errored. This lets a deployment introduce a new object kind without breaking
older peers, which is what makes profiles (§12) additive rather than forking.

## 4.5. Reserved and forbidden keys

**Key 0 is FORBIDDEN in every object.** An object containing key 0 MUST be
rejected with `ERR_FORBIDDEN_KEY` (§13).

This is a deliberate trap rather than a gap. Key 0 is the value an encoder
produces when a field name fails to resolve to a registered number — the most
likely silent-corruption bug in an integer-keyed format. Reserving it as an
error makes that bug loud on the first message instead of subtly wrong forever.

Keys 16–31 are reserved for future core use in all object kinds. Keys 32 and
above are available to profiles (§12.1) and MUST NOT be assigned by this
document.

## 4.6. Sizes

An implementation MUST reject any object larger than **65 536 bytes** with
`ERR_TOO_LARGE`. Work orders describe work; they do not carry payloads. Photos,
signatures and documents referenced by an attestation MUST be carried by
reference (a URL or content address), never inline.
