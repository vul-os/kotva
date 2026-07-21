# 16. Wire format

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 16.1 Scope

The byte-level definition of every TRACT object.

## 16.2 Conventions inherited, not reinvented

Deterministic CBOR (RFC 8949 §4.2). Integer-keyed maps, keys assigned per object type from 1, keys
≥ 64 reserved for extension. Signed objects reject unknown keys fail-closed; unsigned objects may
ignore unknown keys ≥ 64. Domain-separation tags on every signing preimage. Content addresses carry
a multihash-style agility prefix.

**TRACT introduces no new hash construction, no new signature framing, and no new address scheme.**

## 16.3 Objects to be defined

`ProductRecord` · `Offer` (with the four axes) · `RateCard` · `Leg` · `Consignment` ·
`CapacityRecord` · `Review` · `PurchaseAttestation` · `EscrowScope` · `PaymentAttestation` ·
`Order` (sealed) · `OrderLine` · grant and telemetry objects.

## 16.4 The structural rule

Public and sealed objects are **separate type families**. A public object must be structurally
incapable of carrying a name, address or contact detail (§0.5.1) — the prohibition is enforced by
the grammar, not by reviewer discipline.
