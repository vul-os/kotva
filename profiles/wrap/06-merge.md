# 7. Merge

## 7.1. The algebra

WRAP state is a set of signed objects. Merging two replicas is **set union**,
followed by the read-time fold of §6.3.

Union is commutative, associative and idempotent, so merge is too. Replicas
converge regardless of the order, direction, or number of times objects are
exchanged, and a duplicate delivery is free rather than harmful. There is no
merge *function* to get wrong, because objects are immutable and nothing is
ever overwritten.

Where a single current value is needed — the assignment, the current state —
it is *derived* by the fold, not stored. Only three merge behaviours exist:

| Behaviour | Applies to | Rule |
|---|---|---|
| **Add-only set** | `Offer`, `Bid`, `Progress`, `Attestation` | Union. Nothing is ever removed. |
| **Single-writer LWW** | `Assignment` | Highest `ts` among objects by the authorized author. |
| **Immutable** | `WorkOrder` | First-writer-wins by `id`; identical `id` means identical bytes. |

Note that last-writer-wins appears exactly once, and only where the protocol
has already guaranteed a single legitimate writer (§3.6). WRAP never resolves a
conflict between two parties by picking a timestamp winner, because a timestamp
winner is arbitrary and someone's work would be discarded.

## 7.2. Hybrid logical clock

`ts` (key 5) is a hybrid logical clock stamp, encoded as a lexically sortable
string:

```
{unix_ms:013d} "-" {counter:04x} "-" {author_hex}
```

- `unix_ms` — 13 zero-padded decimal digits of wall-clock milliseconds.
- `counter` — 4 lowercase hex digits, incremented when minting more than one
  stamp within the same millisecond.
- `author_hex` — the full 64-character lowercase hex of the author's Ed25519
  public key.

Because the encoding is lexically sortable, ordinary string comparison is the
total order. No parsing is required to sort.

**Minting.** On each mint, take `max(now_ms, last_ms)`. If it equals `last_ms`,
increment `counter`; otherwise reset `counter` to zero. A node MUST seed
`last_ms` from the highest stamp it has already issued or observed, so a
backwards wall-clock step cannot mint a stamp that sorts below existing state.

**Observing.** On receiving a remote stamp, advance the local clock to
`max(local, remote)` before the next mint. This is what makes causally later
events sort later even across nodes with skewed clocks.

## 7.3. Tie-break

Two stamps with equal `unix_ms` and equal `counter` are broken by **byte
comparison of `author_hex`**, ascending.

This choice is normative and deliberate. The obvious alternative — breaking
ties on a per-node identifier — produces a total order that depends on *which
node* wrote an object rather than *who authored* it. Those differ whenever an
object is relayed, and two implementations that disagree on the rule converge
to different orders while both believing they are correct. Tying on the author
public key keeps the order a property of the object itself, so it survives
relaying, re-encoding, and storage in a different engine.

An implementation MUST NOT introduce a node identifier into the ordering.

## 7.4. Convergence

Two replicas that have exchanged all objects for a work order MUST compute an
identical state. Conformance vectors (§15) fix this, including the tie-break
case, which is the one most likely to be implemented differently and the least
likely to be noticed in testing.

Implementations SHOULD expose a **state root** — a content address over the
sorted `id` set of all objects for a work order:

```
root = 0x1e ‖ BLAKE3-256( id₁ ‖ id₂ ‖ … ‖ idₙ )   ids sorted ascending
```

Two parties comparing roots can prove they hold byte-identical state rather
than merely agreeing on what is currently displayed. This turns "it looks
right" into a checkable claim, and it is the cheapest possible sync diagnostic.

## 7.5. Compaction

An implementation MAY discard objects for work orders that reached a terminal
state (§6.1) longer ago than a local retention period, provided it retains:

- the `WorkOrder`;
- the final `Assignment`;
- all `Attestation`s.

Attestations MUST NOT be discarded on the basis of age. They are the portable
record of a worker's history (§9.4) and are the one thing in WRAP whose value
increases with time. A participant who prunes attestations is destroying
somebody's employment record.

## 7.6. Partition behaviour

While partitioned, a participant continues to issue, bid, assign, progress and
attest on the objects it holds. Nothing blocks.

The one operation that cannot be made safe under partition is **assignment of a
work order whose issuer is unreachable**, and WRAP resolves this by construction
rather than by protocol: only the issuer may assign, so a partitioned performer
simply has no assignment to make. When the partition heals, the union includes
whatever the issuer decided, and the performer's bids are still there.

The failure mode this avoids is worth naming, because it is the one that makes
naive decentralized dispatch unusable: two partitioned dispatchers both
assigning the same job to different couriers, both couriers driving to the same
restaurant, and no principled way to decide which one was wrong.
