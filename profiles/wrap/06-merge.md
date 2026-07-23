# 7. Merge

## 7.1. WRAP does not define a merge algebra — it adopts the substrate's

WRAP holds no CRDT of its own. Every convergent, multi-author part of a work
order's state is expressed as **substrate Sync operations**
([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md)), and every immutable part as a
**substrate Feeds/Blob object** ([`FEEDS.md`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md)).
This is the substrate's load-bearing adoption rule
([`substrate/README.md`](https://github.com/vul-os/dmtap/blob/main/substrate/README.md) §3, rule 2): a product
that merges structured state across replicas **MUST** use the Sync op algebra
and **MUST NOT** invent a parallel one and call it its own.

An earlier draft of WRAP defined its own set-union algebra, its own hybrid
logical clock, its own tie-break, and its own state root. All four were the
substrate's, re-derived by hand — the exact "parallel CRDT format" rule 2
forbids. They are removed. What remains WRAP's own is the *mapping* — which
object is which CRDT primitive — and the two rules that follow from what the
work *is* rather than from how it merges: **retention** (§7.3) and **partition**
(§7.4).

## 7.2. The object → primitive mapping (normative)

Each work order is one Sync **namespace**, `ns = "wrap:" ‖ WorkOrder.id` (§3.2),
so a participant **MAY** sparse-sync exactly the jobs it cares about
([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §7) and dropping interest in a job
costs nothing but reachability to it. Within that namespace, every object maps
to one substrate primitive:

| WRAP object | Substrate primitive | Address ([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §4.1) | Resolves by |
|---|---|---|---|
| `WorkOrder` | Immutable content object (SYNC §4.9; FEEDS §3) | the namespace anchor; `id` per §3.2 | never changes |
| `Offer` | OR-Set add (SYNC §4.3) | `target = "offers"` | union |
| `Bid` | OR-Set, add-wins observed-remove (SYNC §4.3) | `target = "bids"` | union; withdraw = observed-remove |
| `Assignment` | LWW register (SYNC §4.4) | `target = "assignment"`, `field = ""` | highest HLC among *authorized* ops |
| `Progress` | OR-Set add / append (SYNC §4.3) | `target = "progress"` | union; current state folded at read (§6.3) |
| `Attestation` | Author-feed entry (FEEDS §4) | the subject's feed | append-only, anti-rollback (FEEDS §4.3) |

Three consequences are worth stating, because each **replaces** something the
old draft hand-rolled with the substrate primitive that already existed for it:

- **Bid withdrawal is the substrate's observed-remove, not a second object.** A
  withdrawal is an OR-Set *remove* op citing the add-tags it cancels
  ([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §4.3) — not a new `Bid` with a
  `withdrawn = true` flag resolved at read time. Concurrent bids from performers
  who cannot see each other all survive; a withdraw races only its own add,
  never another performer's. (§3.5 no longer carries a `withdrawn` field.)

- **Assignment is the substrate LWW register**, resolved by the substrate HLC's
  `(wall, counter, author)` total order ([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md)
  §3). WRAP adds only the admission constraint that the sole authorized author
  is the issuer (§5.5) — enforced as a Sync *admission* rule (SYNC §9), never as
  a merge tie-break. Last-writer-wins is safe here **only because** the protocol
  has already guaranteed one legitimate writer (§3.6); WRAP never resolves a
  two-party conflict by picking a timestamp winner.

- **Attestations are a substrate author feed.** That is what makes a worker's
  reputation portable and self-verifying, servable over plain HTTPS with no mesh
  ([`FEEDS.md`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md) §5) and anti-rollback / equivocation
  protected (FEEDS §4.3) — the same properties WRAP's old text asked for by hand,
  now inherited rather than re-specified.

Time (the HLC), the total order and its tie-break, convergence, the state root,
and snapshots/compaction are **all the substrate's**
([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §3, §5.1, §6) and are not restated here.
WRAP's earlier remark that its tie-break "is the substrate's rule" is now true by
construction rather than by coincidence: there is one rule, defined in one place,
and WRAP moves state into and out of the substrate engine without reordering it.

## 7.3. Retention — attestations are never pruned (WRAP-specific)

The substrate permits an implementation to compact superseded ops behind a
stability cut ([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §6.2). WRAP adds one rule
the substrate cannot know to make: **an `Attestation` MUST NOT be compacted,
snapshotted away, or discarded on the basis of age.** It is the portable record
of a worker's history (§9.4) and the one object in WRAP whose value *increases*
with time. A participant that prunes attestations is destroying somebody's
employment record.

An implementation compacting a work order that reached a terminal state (§6.1)
longer ago than its local retention period **MAY** discard superseded `Offer`,
`Bid`, and `Progress` ops, provided it retains:

- the `WorkOrder`;
- the winning `Assignment` register value;
- every `Attestation` feed entry.

## 7.4. Partition — the one thing that cannot be made safe, and why WRAP is

The substrate converges under arbitrary partition, reordering, duplication, and
delay ([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §2.2). WRAP inherits that for
free. The one operation that no CRDT can make safe is **assignment of a work
order whose issuer is unreachable** — and WRAP resolves it by *construction*
rather than by protocol: only the issuer may author an `Assignment` (§3.6, §5.5),
so a partitioned performer simply has no assignment to make. When the partition
heals, the union includes whatever the issuer decided, and the performer's bids
are still there.

The failure mode this avoids is the one that makes naive decentralized dispatch
unusable: two partitioned dispatchers both assigning the same job to different
couriers, both couriers driving to the same restaurant, and no principled way to
decide which was wrong. A general CRDT would "resolve" that by timestamp and
silently discard one courier's trip. WRAP cannot reach the state at all, because
the contended decision has exactly one authorized author.
