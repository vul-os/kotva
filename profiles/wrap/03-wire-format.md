# 4. Wire format

## 4.1. Encoding is the substrate's, not WRAP's

WRAP objects are serialized with the substrate's shared primitives, unchanged
([`substrate/README.md`](https://github.com/vul-os/dmtap/blob/main/substrate/README.md) §3, rule 4 — "the shared
primitives are non-negotiable"). Specifically:

- **Deterministic CBOR** (RFC 8949 §4.2): map keys sorted ascending, shortest-form
  integers, definite-length arrays and maps only, no duplicate keys, no
  indefinite-length strings. Two implementations encoding the same object produce
  byte-identical output.
- **Integer keys.** WRAP objects use unsigned-integer map keys, the same discipline
  the substrate uses (SYNC `SyncOp`, FEEDS `FeedEntry`): compact on constrained
  links, stable under refactoring, and forcing a visible registry act for every new
  field.

WRAP does **not** define its own canonical form. An earlier draft restated the
RFC 8949 §4.2 rules verbatim; that text is removed, because a product that picks
its own canonical form is exactly what rule 4 forbids, and any drift between a
restatement and the substrate is a silent interop break.

This section therefore defines only what is genuinely WRAP's: its **field
registry** (§4.5), its **kind space** and the forward-compatibility rules over it
(§4.4), and its **size floor** (§4.6).

## 4.2. Versioning

Key 1 (`v`) is the format version and MUST be present. This document defines
version `0`. An implementation receiving an object with a `v` it does not support
MUST ignore it, and MUST NOT attempt best-effort interpretation: a work order
whose semantics have changed is worse than one that was never seen.

## 4.3. Object identifier — the substrate content address

```
id = 0x1e ‖ BLAKE3-256( canonical_bytes )
```

This is the substrate's content-address construction, unchanged
([`FEEDS.md`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md) §3.2): `0x1e` is the multihash prefix
for BLAKE3-256, and `canonical_bytes` is the deterministic-CBOR encoding of the
object map with key 3 (`id`) and the signature omitted. A receiver recomputes it
and rejects any mismatch with `ERR_BAD_ID` (§13). Content addressing gives
deduplication for free — the same object relayed by three peers is stored once —
and it is the same address a WorkOrder carries when it is held as an immutable
substrate blob (§7.2).

## 4.4. Unknown fields and unknown kinds

Two rules, both load-bearing for forward compatibility and both consistent with
the substrate's unknown-kind rule ([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §4.2.1):

**Unknown keys MUST be preserved and ignored.** An implementation receiving an
object with keys it does not understand MUST NOT drop them when re-encoding or
relaying — doing so invalidates the signature and silently corrupts the object for
everyone downstream — and MUST NOT reject the object.

**Unknown kinds MUST be ignored silently.** Not acknowledged, not rejected, not
errored. This lets a deployment introduce a new object kind without breaking older
peers, which is what makes profiles (§12) additive rather than forking.

## 4.5. WRAP field registry

WRAP allocates the object kinds `0x01`–`0x06` (§3.1) and, within each, the field
keys 6–15. The common-header keys 1–5 are the substrate object frame (§3.2). The
registry discipline:

- **Key 0 is FORBIDDEN in every object**; an object containing it MUST be rejected
  with `ERR_FORBIDDEN_KEY` (§13). Key 0 is the value an integer-keyed encoder
  produces when a field name fails to resolve to a registered number — reserving it
  as an error makes that silent-corruption bug loud on the first message.
- **Kinds `0x40`–`0x7f`** are reserved for profile-specific objects (§12).
- **Kinds `0x80` and above** are reserved for future core use.
- **Field keys 16–31** are reserved for future core use in all kinds; **keys 32 and
  above** are available to profiles (§12.1) and MUST NOT be assigned by this
  document.

## 4.6. Sizes

An implementation MUST reject any WRAP object larger than **65 536 bytes** with
`ERR_TOO_LARGE`. Work orders describe work; they do not carry payloads. Photos,
signatures, and documents referenced by an attestation MUST be carried by
reference — a URL or a substrate content address (FEEDS §3) — never inline.
