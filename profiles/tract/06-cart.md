# 6. Cart

> **Drafting status.** Normative. The buyer-held nature of the cart, its transport over the
> substrate sync op algebra, the signal-not-reservation boundary against §3, one-order-per-seller
> checkout with no coordinator, and the four operator-shaped constraints the bounded counter carries
> are settled and stated below with RFC 2119 keywords. This section introduces **no new bytes**: the
> cart has no object in §16 by design (it is buyer-private substrate state), and the bounded-counter
> inventory rides the substrate sync op algebra, not a TRACT-defined CRDT. Two decisions remain
> open, each marked **PROVISIONAL — pending decision** at the point it bites and collected in §6.5:
> who chooses the transfer recipient when a bounded-counter transfer names one (N ≥ 3 replicas), and
> how aggressively replicas rebalance quota. The key words MUST, MUST NOT, REQUIRED, SHALL, SHOULD,
> SHOULD NOT and MAY are to be interpreted as in BCP 14 (RFC 2119, RFC 8174).

## 6.1 Scope

This section owns two things that live on opposite sides of the trade and never touch each other on
the wire: the **buyer's cart**, and the **seller's oversell prevention**.

- The cart is **buyer-held state**. It is where a multi-seller selection exists, and it is the only
  place that view exists anywhere. No seller and no gateway holds a cart or learns its whole shape.
- Oversell prevention is the seller's **bounded-counter inventory** across the seller's own
  replicas, which guarantees that concurrent sales never exceed real stock **without a coordinator**
  (§6.2, §6.2a).

This section defines no new bytes. The cart has no object in §16 — that is deliberate, not an
omission (§6.2) — and the bounded counter is expressed over the substrate sync op algebra
(`github.com/vul-os/dmtap/blob/main/substrate/SYNC.md`), never a bespoke CRDT. A cart's line count
is bounded by `MAX_CART_LINES` (§19.5), which bounds what a **seller** must be prepared to receive
at checkout, not what a buyer may collect.

## 6.2 The cart is buyer-held state

The cart is state the buyer holds and synchronises **across the buyer's own devices**, using the
substrate sync op algebra (capability ③, `SyncOp` / HLC total order,
`github.com/vul-os/dmtap/blob/main/substrate/SYNC.md`). An implementation MUST use that op algebra
and MUST NOT define a parallel cart CRDT. No seller, no gateway, and no index holds cart state or
receives it; a buyer's node MUST NOT publish cart contents to any party before checkout.

**The cart has no object in §16, and this is by design.** A cart is buyer-private device state, not
a public feed object and not a sealed order; there is nothing about it to put on the wire between
parties before checkout, so there is no cart production for a §16 slot to hold. An implementation
therefore MUST NOT synthesise a cart wire object, and MUST NOT smuggle cart state into any public
object — the cart cannot become a route by which a buyer's selection, quantities, or identity leak
into the irrevocable public quadrant (§16.4, §0.5.1).

A cart line references an `Offer` by its content address (§16.5.2) and a quantity. Because an offer
is a public content-addressed object, a cart carries **no personal data**: the buyer's name, address
and contact details appear for the first time only in the sealed `Order` produced at checkout
(§16.6), never before and never in public.

**Live re-evaluation.** A buyer's node SHOULD re-evaluate a cart line's availability against the
current published offer before checkout (§15, transacting profile). Re-evaluation is the local
availability computation of §3.7 over the latest fetched copy of the seller's feed; it contacts no
seller and confirms nothing. A cart line's availability is a **signal, never a reservation** (§3.9):
adding a line to a cart holds no stock, and an implementation MUST NOT present a cart line as
reserved, held, or guaranteed. The only authoritative resolution of whether stock exists is the
seller's `placed → accepted / declined` step (§18.3); the browsing-time and cart-time signals are
not authoritative and MUST NOT be represented as such.

**Oversell prevention lives on the seller's side.** A seller MAY run its inventory across N replicas
of its own node. To guarantee that concurrent sales never exceed real stock without a coordinator,
the seller's replicas MUST use a **bounded counter** from the escrow/demarcation family — the
counter's bound is pre-partitioned into transferable **rights**, a replica may decrement only from
rights it holds locally, and the locally computed right count is conservative (it may under-count
but never over-count), so the no-oversell invariant holds even from stale state and survives
arbitrary message loss, delay, reorder and partition (§21.10.1). The bounded counter MUST be
expressed over the substrate sync op algebra; a seller MUST NOT invent a parallel inventory CRDT. A
checkout that would exceed the combined quota remaining across a seller's replicas MUST fail closed
with `ERR_TRACT_OVERSELL_PREVENTED` (`0x0501`, §17) rather than oversell.

§6.2a states what that guarantee costs and the four operator-shaped constraints it carries. §6.4
states the honest limits both the cart and the counter must disclose.

## 6.2a What the bounded counter actually costs (§21.10)

The mechanism is real and shipped: the escrow/demarcation family (O'Neil 1986 → Barbará-Millá &
Garcia-Molina 1994 → Balegas et al., SRDS 2015), in production as AntidoteDB's
`antidote_crdt_counter_b` (§20, §21.10.1). Safety holds under arbitrary message loss, delay, reorder
and partition, because the locally computed right count is conservative — it can under-count but
never over-count. For contrast, a plain read-check-decrement counter measurably *does* oversell, at
production-relevant concurrency, in the published experiment (§21.10.1). The guarantee is
**safety-only and is paid for entirely in liveness** (§6.4).

What §21.10.2 surfaced is that **four operator-shaped roles hide inside the counter**. This section
accepts each explicitly rather than letting "no coordinator" read as "no coordination anywhere."
None is engineered away; three are stated as hard constraints, and the fourth is left open (§6.5).

1. **An intra-replica serialization point is mandatory.** Each replica MUST have a local
   serialization point — a single-writer node, a compare-and-swap, or a local consensus — over its
   own rights. A "replica" therefore MUST NOT itself be a set of mutually uncoordinated nodes. The
   coordination is not removed; it is pushed down one level and made local, and an implementation
   MUST NOT present replica-level operation as coordination-free at every level.

2. **The failure model is crash-recovery with durable state, not Byzantine — this is a hard
   boundary.** The guarantee holds only for a seller's **own** replicas, which trust each other and
   persist their state durably. A lying or state-losing replica simply oversells, and the literature
   covers no case of stock held across mutually distrusting parties (§21.10.2). Accordingly, the
   bounded counter protects **a seller from their own concurrency**; it does **not** protect a buyer
   from a dishonest seller. An implementation and its documentation MUST NOT present the counter as a
   defence against a seller who lies about stock, and MUST NOT conflate the two — doing so would be a
   safety claim the evidence does not support (§6.4, §21.10.2).

3. **Reclaiming a dead replica's rights is an explicit, fallible decision.** Rights stranded on a
   permanently-departed replica may be reclaimed only by an explicit decision (an agreement among
   the surviving replicas, or a failure-detector call the seller configures). This decision MUST NOT
   be a silent background sweep, because if it is wrong — if the "dead" replica returns — rights are
   double-issued and the no-oversell invariant breaks. The failure mode (double-issue on a mistaken
   liveness call) MUST be stated by any implementation that performs reclamation, not buried. The
   detector's aggressiveness is seller policy; the requirement that reclamation be explicit and its
   failure mode disclosed is not.

4. **At three or more replicas, a transfer names its recipient.** With two replicas a rights
   transfer is anonymous; at N ≥ 3 the giver must name the recipient, which reintroduces an
   allocation policy — and the source papers suggest centralising it. Who chooses the recipient is a
   policy this section does not settle.
   > **PROVISIONAL — pending decision.** The recipient-selection (quota-allocation) policy at N ≥ 3
   > is recorded in §6.5 and the founder-decision list. It is a seller-internal policy, not a wire
   > shape, so it requires no §16 change; but the specification does not currently mandate one, and
   > an implementation MUST NOT read the absence of a mandated policy as licence to allocate in a way
   > that violates constraints 1–3 above.

## 6.3 Checkout produces one sealed order per seller, with no coordinator

Checkout is where the cart stops being multi-seller. **One order per seller is a grammar-level
property, not a client convention** (§16.6): `Order` names a single seller and MUST carry only that
seller's lines, so a cross-seller order is not expressible on the wire. A cart spanning K sellers
therefore produces **K independent sealed `Order` objects** (§16.6, §7.2), one per seller, and none
of the K carries any trace that the other K−1 exist — not their sellers, not their lines, not their
totals. Each seller learns only its own lines and this buyer's total for those lines.

**There is no coordinator, and none is added here.** A distributed transaction across sovereign
parties would require a coordinator with authority over all of them, which is precisely the thing
this design does not have and is not growing. Consequently:

- Checkout MUST NOT be presented as atomic across sellers. One seller declining, timing out, or
  cancelling has **no effect** on another seller's order state (§7.8).
- An interface built on this section MUST show **per-seller status**, never a single aggregate
  "order placed" — a single aggregate status would report a guarantee the protocol does not make
  (§6.4).
- The literature is explicit that the standard pattern for this multi-party case, **sagas, provides
  neither atomicity nor isolation** (§21.10.3): other parties observe partial checkouts and there is
  no abort cascade that unwinds an accepted order because a different seller declined. Oversell
  prevention therefore sits in the per-replica bounded-counter invariant (§6.2, §6.2a), **not** in
  the checkout flow above it. §6.4's "no cross-seller atomicity" is the correct shape, not a
  limitation to be engineered away later.

**One currency per order.** Each resulting `Order` carries exactly one `money` total (§16.6). A cart
MAY hold lines priced in different currencies across different sellers, but checkout MUST NOT sum
across currencies into any order total; cross-currency arithmetic is refused, never coerced (§16.7,
§5.3). A cart line whose currency cannot be reconciled into its seller's single-currency order is a
buyer-side presentation matter, not a coercion the protocol performs.

## 6.4 Honest limits this section must state

- **A partitioned replica strands the quota it holds.** The no-oversell guarantee is safety-only and
  is paid for entirely in liveness (§21.10.1): a replica holding unused rights while partitioned
  **strands** that stock — it cannot be sold from elsewhere — until the replica rejoins. Spurious
  "sold out" and aborted decrements while global stock remains are the expected liveness cost, not a
  defect. Documentation MUST NOT present the guarantee as free of this cost.
- **The counter is non-Byzantine (§6.2a item 2, §21.10.2).** It protects a seller from their own
  concurrency, not a buyer from a dishonest seller. This section makes **no** oversell or supply
  guarantee against a seller who lies or loses state, and MUST NOT be cited as one.
- **There is no cross-seller atomicity.** A multi-seller cart is a set of independent orders; one
  seller declining does not roll back another's acceptance (§6.3, §7.8). Checkout is a set of
  independent placements with eventual per-seller amendment, **never** a distributed transaction —
  that would need a coordinator with authority over sovereign parties. Interfaces MUST show
  per-seller status rather than a single "order placed".
- **A cart line is not a reservation (§3.9).** Adding to a cart holds nothing; the browsing-time and
  cart-time availability signals are advisory, and a line MAY have sold out between the last feed
  fetch and checkout. The authoritative resolution is the seller's `placed → accepted / declined`
  step (§18.3) — which, at partitioned inventory, MAY be a decline carrying
  `ERR_TRACT_OVERSELL_PREVENTED` (§17). This is disclosure of uncertainty, not a protocol defect.
- **§21 is honest evidence, not support.** This section leans on §21.10.1/§21.10.2/§21.10.3 only to
  **disclaim** — to state the failure model, the liveness cost, and the absence of checkout
  atomicity — never to claim a result. Trust, dispute and logistics evidence returned nothing
  verified (§21.1), and this section cites §21 for none of them.

## 6.5 Open

Two decisions remain open. Each is written as **PROVISIONAL — pending decision** at its point of use
above and collected here for the founder-decision list. Neither is invented in this section.

- **Quota-recipient allocation at N ≥ 3 (§6.2a item 4, §21.10.2).** When a bounded-counter transfer
  must name a recipient, who chooses it, and by what policy — round-robin, demand-weighted, or a
  seller-elected coordinating replica. Requires no §16 change (it is seller-internal state over the
  substrate sync op algebra). *Recommendation:* leave the mechanism to the seller but require, as a
  normative floor, that the policy be deterministic per replica set and disclosed in the seller's
  own operational configuration, so it cannot silently violate the intra-replica serialization
  constraint (§6.2a item 1); do not mandate a single allocation algorithm in the specification.
- **Quota rebalancing aggressiveness (§6.2a).** How aggressively replicas should transfer rights to
  reduce stranding (§6.4), and whether the specification should mandate one policy or leave it to
  implementations. Requires no §16 change. *Recommendation:* leave it to implementations. Rebalancing
  aggressiveness is a pure liveness/overhead trade-off with no effect on the safety invariant, and a
  mandated policy would be a proposal with no measurement behind it (§19.8); a seller tuning it
  against its own demand is the right altitude for the decision.
