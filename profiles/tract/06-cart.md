# 6. Cart

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 6.1 Scope

Buyer-side cart state, live availability, and reservation semantics without a central lock.

## 6.2 What this section will specify

- The cart as **buyer-held CRDT state**, synced across the buyer's own devices via substrate
  capability ③. No seller and no gateway holds it.
- Live re-evaluation: subscribing to availability feeds for items already in the cart.
- **Reservation and hold** semantics, including what a seller may promise and for how long.
- **Bounded-counter inventory** across a seller's own replicas: stock partitioned into per-replica
  quotas, sold freely within quota, quota transferred between replicas on demand. The invariant is
  that total sales never exceed total stock, without a coordinator.

## 6.2a What the bounded counter actually costs (§21.10)

The mechanism is real and shipped: the escrow/demarcation family (O'Neil 1986 → Barbará-Millá &
Garcia-Molina 1994 → Balegas et al., SRDS 2015), in production as AntidoteDB's
`antidote_crdt_counter_b`. Safety holds under arbitrary message loss, delay, reorder and partition,
because the locally computed right count is conservative — it can under-count but never over-count.
For contrast, a plain read-check-decrement counter measurably *does* oversell: about 200 excess
decrements at ~200 concurrent clients in the published experiment.

What §21.10 surfaced is that **four operator-shaped roles hide inside it**, and this section has to
accept or replace each rather than let "no coordinator" read as "no coordination anywhere":

1. **An intra-replica serialization point is mandatory** — a single-writer node, a compare-and-swap,
   or consensus. A replica therefore cannot itself be a set of mutually uncoordinated nodes. The
   coordination is not removed; it is pushed down one level and made local.
2. **The failure model is crash-recovery with durable state, not Byzantine.** This is the hard
   boundary. A lying or state-losing replica simply oversells, and the literature covers **a
   seller's own replicas, which trust each other** — not stock held across mutually distrusting
   parties. Bounded counters protect a seller from their own concurrency; they do **not** protect a
   buyer from a dishonest seller, and conflating the two would be a safety claim nothing supports.
3. **Reclaiming a dead replica's rights is a fallible decision.** If the failure detector is wrong,
   rights are double-issued and the invariant breaks. Deciding a peer is permanently gone is exactly
   the judgement a coordinator makes, so it needs a stated failure mode rather than a background
   sweep.
4. **At three or more replicas, transfer names a recipient.** It is no longer anonymous, which
   reintroduces an allocation policy — and the source papers suggest centralising it. This section
   has to say who chooses.

## 6.3 Prior art

Escrow-based / bounded CRDT counters from the distributed-systems literature. Saga and
compensating-transaction patterns for the multi-party case.

## 6.4 Honest limits this section must state

- A partitioned replica holding unused quota **strands** that stock until it rejoins.
- There is **no cross-seller atomicity**. A multi-seller cart is a set of independent orders;
  one seller declining does not roll back another's acceptance. Checkout is compensating actions,
  never a distributed transaction — that would need a coordinator with authority over sovereign
  parties. Interfaces must show per-seller status rather than a single "order placed".

## 6.5 Open

- Quota rebalancing policy: how aggressively replicas should transfer, and whether the spec should
  mandate one or leave it to implementations.
