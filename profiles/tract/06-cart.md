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
