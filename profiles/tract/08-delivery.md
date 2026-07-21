# 8. Delivery

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 8.1 Scope

Rate cards, legs, consignments, consolidation, and local route computation.

## 8.2 `RateCard`: published, not quoted

A `RateCard` (§16.5.3) is a signed public object: a zone table, weight brackets, a dimensional
divisor, surcharges, served countries, excluded categories, and a `published` timestamp. A leg's
price is **computed by the buyer's node from a locally held copy**, not returned by a call to the
carrier. That is a substitution of architecture, not just of API, and it has consequences beyond
convenience:

| A quote API implies | Publishing removes |
|---|---|
| a rate limit, so the carrier can throttle callers | none — the buyer's node reads its own copy |
| an API key, so the carrier knows who is asking | no key; the object is public and unauthenticated to read |
| a cost per quote, since carriers meter API calls | zero marginal cost once the card is fetched |
| the carrier — or whoever operates the quoting endpoint — learning what is being bought, by whom, in what quantity, and how often | nobody outside the buyer's own device runs the computation |

The object type carries no privilege tier. A `RateCard` is keyed to whoever published it — a
national carrier or a neighbour with a bicycle, identically typed (§0.4.1's courier and
distributor rows) — so the grammar cannot express a "professional" rate card distinct from an
amateur one. A route comparison that mixes a multinational's zone table with a peer courier's flat
local rate is one computation over two instances of the same type, not two code paths.

Cards drift: surcharges change, zones get renumbered. `published` exists so a stale card is not
replayed silently, and `RATE_CARD_MAX_AGE` (30 days, §19.6) is the point past which a locally
computed price is stale enough to mislead — the buyer is told the figure is old rather than shown
a confident number computed from it.

## 8.3 Volumetric weight and the divisor floor

`billable = max(actual, L·W·H / dim_divisor)`. Every major carrier prices this way, because a
router that used actual weight alone would under-quote large, light parcels — the buyer would
discover the shortfall at the counter, which is exactly the surprise this design exists to remove
by computing the number before checkout rather than after.

`dim_divisor` is the field a hostile or careless publisher can use to distort every quote computed
from their card: the smaller the divisor, the larger the volumetric figure, and volumetric weight
already dominates actual weight for anything bulky and light. `MIN_DIM_DIVISOR` is 1000 (§19.5).
Real carriers publish 5000 or 6000; nothing between 1 and 1000 corresponds to a divisor any carrier
actually uses, so a card in that range is treated as unusable rather than as an aggressive but
legitimate rate. Zero is the one exception and means something different — "this carrier does not
apply volumetric weight at all" — not an unset field, so it is accepted rather than rejected.

| `dim_divisor` | Meaning | Accepted |
|---:|---|---|
| 0 | carrier does not apply volumetric weight | yes |
| 1–999 | not a divisor any carrier publishes; inflates every parcel priced against the card | no |
| ≥ 1000 | ordinary range; 5000 and 6000 are the common real-world values | yes |

This is checked before the card is used to price anything, because a rate card is a claim by its
publisher and arrives from a stranger — it is validated, not trusted, the same way any other public
object arriving unsolicited is (§14.3).

## 8.4 Legs, consignments, and custody

A **leg** (§0.9 glossary) is one movement of goods between two places, priced by one rate card. A
**consignment** is physical goods in someone else's custody — a courier leg or a distributor hold —
and moving through several legs means moving through several consignments in sequence, each with
its own custodian.

Custody changing hands is attested by a **signed custody handoff**, and the full lifecycle —
`created → accepted → in-custody → handed-off → delivered`, who signs each transition, and what
each timeout expires into — is specified once, in §18.4, and is not repeated here. The one property
worth restating because it shapes everything else in this section: a handoff is signed by the party
**taking** custody, not the party giving it up, because a chain attested only by senders proves
someone tried to hand something over, and a chain attested by receivers proves the goods actually
moved.

## 8.5 Consolidation: three candidate routings

A cart with lines from several sellers can reach the buyer by more than one shape, and the three
worth comparing exhaust the useful design space:

| Shape | Route | Wins when |
|---|---|---|
| **Direct** | every seller ships straight to the buyer | speed per item matters more than total cost |
| **Hub-near-buyer** | sellers ship to a distributor near the buyer, who combines and sends one parcel | the last mile dominates cost; the buyer accepts waiting for the slowest seller |
| **Hub-near-sellers** | sellers who are geographically close consolidate first, then one long leg carries the combined parcel | the long haul dominates cost |

Each candidate is scored the same way:

```
total = Σ(leg costs) + storage_per_day × wait_days + handling_fee
```

`storage_per_day`, `handling_fee`, and `wait_days` come from the hub's `CapacityRecord` (§16.5.3)
when a hub is involved, and are zero for `Direct`. Comparing three fully-priced totals is the whole
routing decision.

**No optimiser is specified or needed.** The candidate set is small by construction — a handful of
hub choices, not a combinatorial search over carriers and paths — so a buyer's own device evaluates
all three exhaustively and picks the lowest total. A specification that reached for a solver here
would be adding machinery to a problem that does not have enough variables to need one.

## 8.6 Distributors: the consolidation role

A **distributor** publishes a `CapacityRecord` (§16.5.3) — country, coarse locality, storage per
item per day, a handling fee, available slots, and excluded categories — and holds goods in transit
on that basis. Entry is permissionless: a keypair and space is the whole requirement, the same bar
as every other role in §0.4.1.

Economically this is the freight-forwarder pattern that already runs worldwide. What differs is
that the terms are a signed object anyone can read and compute against, rather than a bilaterally
negotiated contract a buyer's node has no way to see. A distributor competes on the same published
terms a carrier competes on rate cards, and switching between them costs a routing recomputation,
not a renegotiation.

## 8.7 Honest limits this section must state

- **Redistribution of negotiated rates may be restricted.** Some carriers' API terms forbid
  republishing negotiated rates. Published list rates, and a seller's own rates they choose to
  publish, are unaffected; republishing a carrier's confidential negotiated rate is the
  *publisher's* compliance problem, not the protocol's. An offer may declare that a live quote is
  required (§16.5.3) instead of pointing at a card, precisely for this case.
- **Transit figures are estimates, not commitments.** `transit_days` on a `Zone` (§16.5.3) is the
  carrier's own claim, and `TRANSIT_TIMEOUT` (carrier estimate × 3, §19.3) is built around that
  being unreliable rather than around it being wrong by a fixed margin.
- **`wait_days` is the least reliable input in the whole computation.** It depends on the slowest
  seller in a consolidation actually dispatching on time, which is outside any rate card's or
  distributor's control, and it must be presented to a buyer as an estimate rather than folded into
  a total that reads as a promise.
- **Custody attestations prove transfer, not recoverability.** A signed handoff establishes who
  held the goods and when. It establishes nothing about getting them back. `lost` is a terminal
  state in §18.4 with a signed history attached and no remedy attached — where the goods went is
  answerable; being made whole for them is §9's problem, or nobody's.
- **This section is unevidenced.** Three research passes over the literature returned no verified
  findings on logistics standards, carrier API terms, or consolidation optimisation (§21.1); the
  gap is recorded there as absence of evidence, not evidence of absence. Everything above is design
  reasoning checked for internal consistency, not a claim checked against how carriers, distributors
  and couriers actually behave, and it should be read with that confidence level and no more.

## 8.8 Open

- Whether a peer courier needs a distinct rate-card shape — flat price per kilometre — or should be
  expressed inside the same zone-table model as a national carrier, at the cost of a zone table with
  one entry.
- Whether `RATE_CARD_MAX_AGE` should differ between a `RateCard` and a `CapacityRecord`. Zone tables
  and storage rates plausibly drift at different speeds, and today one parameter (§19.6) covers
  both.
- Whether consolidation route choice needs to be reproducible across independent implementations
  given the same inputs, or whether two conformant buyer nodes may legitimately land on different
  totals because their copies of the same rate card are at different points within
  `RATE_CARD_MAX_AGE`.
