# 15. Conformance

## 15.1. Vectors are normative

The conformance vectors in `conformance/wrap_vectors.json` are **normative and
take precedence over the prose of this document** wherever the two disagree.

This inversion is deliberate. Prose cannot tell an implementer that they broke
a tie the wrong way, sorted a map incorrectly, or omitted a domain-separation
tag — all three produce code that passes every test the author thought to
write, interoperates with itself perfectly, and fails silently against a second
implementation. Vectors catch exactly this class of error, and they are the
difference between protocols that acquire independent implementations and
protocols that acquire admirers.

A claim of WRAP v0 conformance means: all vectors pass, with no skips. An
implementation that skips vectors MUST say which and why.

## 15.2. Required coverage

| Group | What it fixes |
|---|---|
| `encode` | Deterministic CBOR for every object kind; key ordering; shortest-form integers |
| `id` | `id` computation over canonical bytes, including omission of key 3 |
| `sign` | Preimage construction with the domain tag; signature verify pass and fail |
| `reject` | Key 0; non-canonical encoding; oversize; bad id; bad signature; unsupported version |
| `authorship` | `Assignment` by a non-issuer rejected; `Bid` by any principal accepted |
| `hlc` | Mint monotonicity; observe advancement; backwards-clock seeding |
| `tiebreak` | Equal `unix_ms` and `counter`, resolved by author public key |
| `merge` | Union convergence; idempotent redelivery; reordered delivery; three-way relay |
| `fold` | State computation from an object set, including unreachable-state discard |
| `expiry` | Computed expiry with no message exchanged |
| `forward` | Unknown kind ignored; unknown field preserved through re-encode; unknown profile stored |
| `proof` | Handoff commitment verifies; wrong code fails |

## 15.3. The cases most likely to be got wrong

Implementers SHOULD run these first.

1. **Tie-break on author key (§7.3).** The most likely divergence in the whole
   document, and invisible until a second implementation exists.
2. **Preserving unknown fields through a re-encode (§4.4).** Dropping them
   invalidates the signature for everyone downstream, and the implementation
   that dropped them will not be the one that reports the failure.
3. **Rejecting rather than repairing non-canonical encodings (§5.4 step 1).**
   Repairing lets two byte strings share one `id`.
4. **Pending rather than rejecting unknown referents (§5.6).** Rejecting makes
   out-of-order delivery lossy, and out-of-order delivery is normal.
5. **Computed expiry (§6.2).** Implementations that wait for an expiry message
   will hold work orders forever when a peer is unreachable.

## 15.4. Interoperability

Two implementations are interoperable at v0 if, given the same object set
delivered in any order, they compute the same state (§6.3) and the same state
root (§7.4).

State root equality is the RECOMMENDED interoperability test because it fails
loudly on divergence that a state comparison would hide — two replicas can
display identical current state while holding different histories, and the
difference surfaces only later, usually during a dispute.
