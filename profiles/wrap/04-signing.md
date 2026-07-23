# 5. Signing

## 5.1. Sign the object, not the frame — the substrate discipline

Every WRAP object is signed by its author over a canonical preimage derived from
the object itself, **never over a transport frame**. This is not a WRAP invention;
it is the substrate's signing discipline
([`substrate/README.md`](https://github.com/vul-os/dmtap/blob/main/substrate/README.md) §3, rule 4): Ed25519 over a
domain-separated preimage of the deterministic-CBOR bytes, in a COSE-style
envelope where one is called for.

Because the signature covers the object bytes and not the frame, the same signed
object is valid whether it arrived as a Sync op in a `POST /sync/ops` batch, a
feed entry fetched over plain HTTPS, a line in a file on a USB stick, or a mesh
frame. A binding is a framing detail, not a re-signing exercise, and an untrusted
intermediary can relay an object without touching its authenticity.

## 5.2. Preimage and domain separation

```
preimage = "WRAP-v0/object" ‖ 0x00 ‖ canonical_bytes
```

where `canonical_bytes` is exactly the bytes hashed for `id` (§4.3) — the
deterministic-CBOR encoding of the object with key 3 and the signature omitted.

The domain-separation tag `"WRAP-v0/object"` and the `0x00` terminator are
REQUIRED, per the substrate's DS-tag rule (§18.1.6): the tag is the **only**
WRAP-specific part of the signing construction, and it exists so a signature
produced for a WRAP object can never be replayed as a valid signature in another
protocol that signs raw CBOR, or vice versa.

## 5.3. Signature envelope — COSE_Sign1

A signed WRAP object is a `COSE_Sign1` envelope (RFC 9052) over `preimage`,
produced by the private key corresponding to key 4 (`author`), exactly as the
substrate signs a `SyncOp` or a `FeedEntry`. This is what lets a WRAP object move
into the substrate Sync engine (§7.2) or be published as a substrate feed entry
without re-signing.

An earlier draft made a bespoke two-element `[bytes, sig]` array the REQUIRED form
and COSE merely OPTIONAL; that is inverted. Carrying two signature stacks is the
duplication rule 4 forbids, so WRAP signs the way the substrate signs, and nothing
else.

## 5.4. Verification

A receiver MUST, in this order, aborting at the first failure:

1. Decode `canonical_bytes` as deterministic CBOR (§4.1). Reject a non-canonical
   encoding with `ERR_NOT_CANONICAL` — do **not** re-encode and continue. (Accepting
   a non-canonical encoding and re-encoding before hashing would let two distinct
   byte strings map to one `id`, breaking dedup and content-address integrity.)
2. Reject key 0 with `ERR_FORBIDDEN_KEY` (§4.5).
3. Check `v` is supported (§4.2).
4. Recompute `id` and compare with key 3. Reject a mismatch with `ERR_BAD_ID`.
5. Verify the `COSE_Sign1` signature against key 4, per the substrate's verify
   rules ([`IDENTITY.md`](https://github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md) §2, including `DeviceCert`
   chain and revocation where the author signs under a device subkey). Reject with
   `ERR_BAD_SIG`.
6. Apply the WRAP **authorship rule** for the object's kind (§5.5). Reject with
   `ERR_NOT_AUTHORIZED`.

Only after all six does an object enter local state. An implementation MUST NOT
display, index, or relay an object that has not passed verification. Steps 1–5 are
the substrate's; step 6 is WRAP's, and it is the reason WRAP is a distinct spec at
all.

## 5.5. Authorship rules (WRAP-specific admission)

Signature validity is necessary but not sufficient. Each kind constrains *who* may
author it — and, because WRAP objects are substrate Sync ops and feed entries,
this is enforced as the substrate's **admission** decision
([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §9), applied before an op enters state,
not as a merge-time judgement:

| Kind | Valid author |
|---|---|
| `WorkOrder` | Any Principal |
| `Offer` | The `WorkOrder`'s author |
| `Bid` | Any Principal |
| `Assignment` | **The `WorkOrder`'s author only** |
| `Progress` | The assigned performer, or the issuer |
| `Attestation` | Any Principal — but see §9.3 on weighting |

Pool admission (§8.4) constrains who *receives* an `Offer` and so who is
positioned to `Bid`, but v0 defines no membership mechanism a verifier can
check — it is an out-of-band distribution and social matter, not a
verify-step authorship check. `ERR_NOT_AUTHORIZED` therefore never applies
to a `Bid` in v0.

An object that is cryptographically valid but violates its authorship rule MUST be
rejected, not down-ranked. In particular an `Assignment` signed by anyone other
than the issuer is **not** a competing LWW op to be resolved by the register's HLC
order (§7.2); it is inadmissible and never enters state. This is what prevents a
performer from assigning work to themselves, and it is why the `Assignment`
register can safely use last-writer-wins: admission has already reduced its writers
to one.

## 5.6. Referential validation

An object referring to a `WorkOrder` the receiver does not hold MUST be retained as
**pending**, not rejected. Out-of-order arrival is normal: a `Bid` routinely
overtakes the `Offer` that provoked it when they travel by different paths, and the
substrate's reconciliation makes no ordering promise (SYNC §5).

A pending object MUST be re-validated when its referent arrives, and MUST be
discarded once the referent's `expires` has passed. Implementations SHOULD cap
pending storage and evict oldest-first; an unbounded pending queue is a
denial-of-service surface.
