# kotva-cbor

**Canonical deterministic CBOR — a strict encoder and a fail-closed decoder, with zero dependencies.**

RFC 8949 §4.2.1 core deterministic encoding, as profiled by KOTVA/DMTAP §18.1.1.

```toml
kotva-cbor = "0.1"
```

## Why this exists

A signature is a statement about *bytes*. If two honest implementations can serialize the same value
to different bytes, the signature says nothing and a content address is not an address.

So the codec that signed and content-addressed objects flow through must be **byte-identical across
independent implementations**, and its decoder must **reject** non-canonical input rather than
normalize it — a decoder that silently re-canonicalizes lets a sender hand two different verifiers
two different byte strings for the same object.

`encode_canonical` emits exactly one encoding per value. `decode_canonical` accepts *only* that
encoding, so

```text
encode_canonical(decode_canonical(b)?)? == b     for every accepted b
```

which is the malleability guarantee the whole crate exists to provide.

## Rules the decoder enforces

1. **Definite lengths only** — indefinite-length items and the `break` code are rejected.
2. **Shortest-form arguments** — a widened head (`24` spelled with a 2-byte argument, a length of 5
   spelled `0x59 0x00 0x05`) is rejected.
3. **Map keys strictly ascending** by the bytewise order of their own canonical encodings, with no
   duplicates.
4. **No floating point**, and no simple values beyond `false` / `true` / `null`.
5. **No tags.**
6. **Exactly one top-level item** — trailing bytes are an error.

Two further checks are resource guards rather than canonical-encoding rules, and are documented as
such because they are the one place a conforming implementation may legitimately differ: nesting
depth is capped at `MAX_DEPTH` (64), and every length header is checked against the bytes actually
remaining *before* anything is allocated, so a crafted 8-byte length cannot force a huge allocation
out of a nine-byte input.

## Zero dependencies, deliberately

```console
$ cargo tree -p kotva-cbor --all-features
kotva-cbor v0.1.0
```

Not `serde`, not `thiserror`, not `hex`. std only.

That is a load-bearing property, not a preference. The sibling crate `kotva-core` is a heavy leaf —
linking it pulls `hpke`, `x-wing`, `ml-dsa`, `chacha20poly1305`, `ciborium`, `hkdf`, `sha2` and
`unicode-normalization`, 218 transitive crates in total, none of which canonical CBOR needs. A
consumer that needs only the wire codec must pay for only the wire codec; otherwise it hand-rolls a
copy, which is how this family ended up with four of them. **Adding a dependency here is a decision
to be argued in the commit that does it.**

## Provenance — a consolidation, not a new implementation

The Vulos family independently grew **four** canonical-CBOR codecs, all claiming the same rules,
with no cross-check between any two: `kotva_core::cbor`, `kotva_sync::detcbor`,
`evermesh_kernel::codec` and `magnetite_seams::cbor`.

This crate is seeded from **evermesh's**, which was the most tested of the four and the only one
whose module docs already treated cross-implementation byte identity as consensus-critical.

`tests/evermesh_conformance.rs` replays a frozen corpus of **183 vectors** extracted from evermesh's
189-vector conformance suite, and asserts that every canonical vector re-encodes to *itself* and
every non-canonical one is refused. The corpus is a deliberate **snapshot** with its source revision
recorded in the file header — a live mirror would silently track upstream and so could never catch
the drift it exists to catch.

## Scope of the value space

`Value` spans unsigned and negative integers, byte and text strings, arrays, maps, `bool` and
`null`. It has **no float and no tag arm at all**, so those are unrepresentable rather than merely
rejected.

Map keys may be **any** `Value`. That is inherited from evermesh deliberately, so this crate accepts
exactly what evermesh accepts. It has one consequence worth stating plainly: a map keyed by a
`Bool`, `Null`, `Array` or `Map` is accepted here but has **no faithful JSON interchange form**.
Callers with a schema should reject key types their schema does not name — `kotva_core::cbor::Cv`
does exactly that, admitting integer-keyed maps plus one text-keyed site and nothing else.

## Optional `json` feature (default-off)

The JSON interchange mapping of the canonical-CBOR value space (evermesh spec 001 §11): integer map
keys as decimal strings, byte strings as `"hex:…"`, with a `txt:` escape that makes the mapping a
bijection.

Default-off because the mapping is normative to a consumer that is not KOTVA — `kotva-core` and the
DMTAP wire never touch it, and code nobody links should not be code everybody compiles. It ships
here rather than staying downstream so the byte-level codec and its JSON projection cannot drift
apart in separate repos.

## Choosing an encoder

`encode_canonical` refuses a map holding duplicate keys, because the bytes it would otherwise emit
are non-canonical and this crate's own decoder would reject them.
`encode_canonical_unchecked` skips that check and is infallible; it exists only for callers whose
published API is an infallible `fn encode(&V) -> Vec<u8>` and cannot start returning a `Result`
without a breaking change. Prefer the checked one.

## License

MIT.
