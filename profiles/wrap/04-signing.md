# 5. Signing

## 5.1. Sign the object, not the frame

Every WRAP object is signed by its author over a canonical preimage derived
from the object itself, **never over a transport frame**.

This is the single decision that makes transport bindings cheap. Because the
signature covers the object bytes, the same signed object is valid whether it
arrived as an HTTP request body, a row in a sync batch, a line in a file on a
USB stick, or an operation in a CRDT log. A binding becomes a framing detail
rather than a re-signing exercise, and an object can be relayed by an untrusted
intermediary without losing its authenticity.

## 5.2. Preimage

```
preimage = "WRAP-v0/object" ‖ 0x00 ‖ canonical_bytes
```

where `canonical_bytes` is exactly the bytes hashed for `id` (§4.3) — the
deterministic CBOR encoding of the object with key 3 and the signature omitted.

The domain-separation tag and the `0x00` terminator are REQUIRED. Without
domain separation, a signature produced for a WRAP object could be replayed as
a valid signature in another protocol that happens to sign raw CBOR, and vice
versa.

## 5.3. Signature envelope

A signed object is transmitted as a two-element CBOR array:

```
[ canonical_bytes : bstr, signature : bstr(64) ]
```

`signature` is Ed25519 (RFC 8032) over `preimage`, produced by the private key
corresponding to key 4 (`author`).

Implementations MAY additionally accept a `COSE_Sign1` envelope (RFC 9052) with
the same preimage; this exists so that deployments already using COSE — the
DMTAP binding among them (§11.2) — need not carry two signature stacks. The
bare array form is REQUIRED; COSE support is OPTIONAL.

## 5.4. Verification

A receiver MUST, in this order, and MUST abort at the first failure:

1. Decode `canonical_bytes` as deterministic CBOR (§4.1). Reject non-canonical
   encodings with `ERR_NOT_CANONICAL` — do **not** re-encode and continue.
2. Reject key 0 with `ERR_FORBIDDEN_KEY` (§4.5).
3. Check `v` is supported (§4.2).
4. Recompute `id` and compare with key 3. Reject mismatch with `ERR_BAD_ID`.
5. Verify the Ed25519 signature against key 4. Reject with `ERR_BAD_SIG`.
6. Apply the **authorship rule** for the object kind (§5.5). Reject with
   `ERR_NOT_AUTHORIZED`.

Only after all six does an object enter local state. An implementation MUST NOT
display, index, or relay an object that has not passed verification.

> Step 1 matters more than it looks. Accepting a non-canonical encoding and
> re-encoding it before hashing would let two distinct byte strings map to one
> `id`, which breaks deduplication and lets an attacker produce two objects
> that a naive implementation treats as the same one.

## 5.5. Authorship rules

Signature validity is necessary but not sufficient. Each kind constrains *who*
may author it:

| Kind | Valid author |
|---|---|
| `WorkOrder` | Any Principal |
| `Offer` | The `WorkOrder`'s author |
| `Bid` | Any Principal the offer's pool admits (§8.4) |
| `Assignment` | **The `WorkOrder`'s author only** |
| `Progress` | The assigned performer, or the issuer |
| `Attestation` | Any Principal — but see §9.3 on weighting |

An object that is cryptographically valid but violates its authorship rule MUST
be rejected, not merely down-ranked. In particular, an `Assignment` signed by
anyone other than the issuer is not a competing opinion to be resolved by the
merge algebra; it is invalid and never enters state. This is what prevents a
performer from assigning work to themselves.

## 5.6. Referential validation

An object referring to a `WorkOrder` the receiver does not hold MUST be
retained as **pending**, not rejected. Out-of-order arrival is normal: a `Bid`
routinely overtakes the `Offer` that provoked it when they travel by different
paths.

A pending object MUST be re-validated when its referent arrives, and MUST be
discarded once the referent's `expires` has passed. Implementations SHOULD cap
pending storage and evict oldest-first; unbounded pending queues are a
denial-of-service surface.
