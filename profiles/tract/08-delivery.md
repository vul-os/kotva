# 8. Delivery

> **Drafting status: partially normative.** The delivery mode TRACT **owns** — the **rate-card
> (offer-pull)** mode — is authored to normative RFC 2119 text, aligned exactly to the frozen §16.5.3
> grammar (`RateCard`, `Zone`, `WeightBracket`, `CapacityRecord`): published-not-quoted rate cards
> (§8.2), the volumetric-weight divisor floor (§8.3), leg/consignment/custody as **references**
> (§8.4), three-candidate consolidation routing (§8.5), and the distributor role (§8.6). **§8.9 (the
> WRAP seam) is founder-settled (2026-07-23) and preserved verbatim**: the **on-demand** dispatch
> mode and the custody-handoff lifecycle are **WRAP's**, referenced and never re-specified here. The
> custody lifecycle of §8.4 is likewise WRAP's — §8.4 references §18.4 and WRAP, and restates only the
> single receiver-signs property. What remains **scoped, not normative** is every item in §8.8 — the
> peer-courier rate-card shape, whether `RATE_CARD_MAX_AGE` splits per object, and whether
> consolidation route choice must be reproducible across implementations — each marked inline as
> **PROVISIONAL — pending decision** and collected in §8.8 for the founder-decision list. This section
> is **unevidenced** (§21.1): everything below is design reasoning checked for internal consistency,
> and it MUST NOT be read as validated against how carriers, distributors and couriers actually
> behave. The key words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are to be interpreted as in BCP 14
> (RFC 2119, RFC 8174) wherever they appear below.

## 8.1 Scope

Rate cards, legs, consignments, consolidation, and local route computation — the **rate-card
(offer-pull)** delivery mode. The **on-demand** dispatch mode and the custody-handoff lifecycle are
WRAP's, not this section's (§8.9, §8.4). The byte authority is §16.5.3, and this section MUST NOT
contradict it.

## 8.2 `RateCard`: published, not quoted

A `RateCard` (§16.5.3) is a signed public object: a zone table, weight brackets, a dimensional
divisor, a surcharge, served countries, excluded categories, and a `published` timestamp. A leg's
price **MUST be computed by the buyer's node from a locally held copy** of the card, and MUST NOT
require a call to the carrier. That is a substitution of architecture, not just of API, and it has
consequences beyond convenience:

| A quote API implies | Publishing removes |
|---|---|
| a rate limit, so the carrier can throttle callers | none — the buyer's node reads its own copy |
| an API key, so the carrier knows who is asking | no key; the object is public and unauthenticated to read |
| a cost per quote, since carriers meter API calls | zero marginal cost once the card is fetched |
| the carrier — or whoever operates the quoting endpoint — learning what is being bought, by whom, in what quantity, and how often | nobody outside the buyer's own device runs the computation |

The object type carries no privilege tier. A `RateCard` is keyed to whoever published it — a
national carrier or a neighbour with a bicycle, identically typed (§0.4.1's courier and
distributor rows) — so the grammar (§16.5.3) cannot express a "professional" rate card distinct
from an amateur one, and an implementation MUST NOT treat a card differently by any publisher class
the bytes cannot carry. A route comparison that mixes a multinational's zone table with a peer
courier's flat local rate is one computation over two instances of the same type, not two code
paths.

> **PROVISIONAL — pending decision (peer-courier rate-card shape, §8.8).** Whether a peer courier
> needs a distinct rate-card shape — a flat price per kilometre — or should be expressed inside the
> same zone-table `RateCard` model as a national carrier, at the cost of a zone table with one
> entry, is unresolved. The frozen §16.5.3 grammar carries only the zone-table shape today; a
> per-kilometre variant would be a §16 **MAJOR** grammar change. This section specifies no
> per-kilometre production and an implementation MUST NOT assume one. Recorded for the
> founder-decision list.

Cards drift: surcharges change, zones get renumbered. `published` exists so a stale card is not
replayed silently, and `RATE_CARD_MAX_AGE` (30 days, §19.6) is the point past which a locally
computed price is stale enough to mislead. A price computed from a card older than
`RATE_CARD_MAX_AGE` MUST be surfaced to the buyer as stale rather than presented as a confident
figure computed from it.

> **PROVISIONAL — pending decision (`RATE_CARD_MAX_AGE` split, §8.8).** Whether `RATE_CARD_MAX_AGE`
> (§19.6) should differ between a `RateCard` and a `CapacityRecord` (§8.6) is unresolved: zone
> tables and storage rates plausibly drift at different speeds, and today one §19.6 parameter covers
> both. Until this is settled, an implementation MUST apply the single §19.6 value to both object
> types and MUST NOT assume a per-object staleness horizon this document has not specified. Recorded
> for the founder-decision list.

## 8.3 Volumetric weight and the divisor floor

An implementation MUST compute billable weight as `billable = max(actual, L·W·H / dim_divisor)`.
Every major carrier prices this way, because a router that used actual weight alone would
under-quote large, light parcels — the buyer would discover the shortfall at the counter, which is
exactly the surprise this design exists to remove by computing the number before checkout rather
than after.

`dim_divisor` (§16.5.3 key 4) is the field a hostile or careless publisher can use to distort every
quote computed from their card: the smaller the divisor, the larger the volumetric figure, and
volumetric weight already dominates actual weight for anything bulky and light. `MIN_DIM_DIVISOR` is
1000 (§19.5). Real carriers publish 5000 or 6000; nothing between 1 and 1000 corresponds to a
divisor any carrier actually uses, so a card in that range MUST be treated as unusable and MUST NOT
be used to price anything, rather than accepted as an aggressive but legitimate rate. Zero is the
one exception and means something different — "this carrier does not apply volumetric weight at
all" — not an unset field, so it MUST be accepted rather than rejected.

| `dim_divisor` | Meaning | Accepted |
|---:|---|---|
| 0 | carrier does not apply volumetric weight | yes |
| 1–999 | not a divisor any carrier publishes; inflates every parcel priced against the card | no |
| ≥ 1000 | ordinary range; 5000 and 6000 are the common real-world values | yes |

This check MUST be performed before the card is used to price anything, because a rate card is a
claim by its publisher and arrives from a stranger — it is validated, not trusted, the same way any
other public object arriving unsolicited is (§14.3).

## 8.4 Legs, consignments, and custody

A **leg** (§0.9 glossary) is one movement of goods between two places, priced by one rate card. A
**consignment** is physical goods in someone else's custody — a courier leg or a distributor hold —
and moving through several legs means moving through several consignments in sequence, each with
its own custodian.

**The custody-handoff lifecycle is WRAP's, not TRACT's, and is referenced not re-specified here.**
Custody changing hands is attested by a **signed custody handoff**, and the full lifecycle —
`created → accepted → in-custody → handed-off → delivered`, with `lost` terminal, who signs each
transition, and what each timeout expires into — is specified **once** as WRAP's delivery profile
(`Progress` + `Attestation`, [WRAP](https://github.com/vul-os/wrap) §3.7–§3.8) and is stated for
TRACT as a *state machine* in §18.4. TRACT defines **no** custody object, consignment record, or
handoff transition of its own; an implementation MUST use WRAP's objects for the custody lifecycle
and MUST NOT introduce a parallel TRACT-native one (§8.9, §18.4). Where this section and WRAP would
disagree about the custody machine, **WRAP governs**.

The one property worth restating because it shapes everything else in this section: a handoff is
signed by the party **taking** custody, not the party giving it up, because a chain attested only by
senders proves someone tried to hand something over, and a chain attested by receivers proves the
goods actually moved. The authority for this rule is WRAP's `Attestation` model (§18.4); it is
restated here for orientation and written a third time nowhere.

## 8.5 Consolidation: three candidate routings

A cart with lines from several sellers can reach the buyer by more than one shape, and the three
worth comparing exhaust the useful design space:

| Shape | Route | Wins when |
|---|---|---|
| **Direct** | every seller ships straight to the buyer | speed per item matters more than total cost |
| **Hub-near-buyer** | sellers ship to a distributor near the buyer, who combines and sends one parcel | the last mile dominates cost; the buyer accepts waiting for the slowest seller |
| **Hub-near-sellers** | sellers who are geographically close consolidate first, then one long leg carries the combined parcel | the long haul dominates cost |

Each candidate MUST be scored the same way:

```
total = Σ(leg costs) + storage_per_day × wait_days + handling_fee
```

`storage_per_day`, `handling_fee`, and `wait_days` MUST come from the hub's `CapacityRecord`
(§16.5.3) when a hub is involved, and MUST be zero for `Direct`. Comparing three fully-priced totals
is the whole routing decision.

**No optimiser is specified or needed.** The candidate set is small by construction — a handful of
hub choices, not a combinatorial search over carriers and paths — so a buyer's own device evaluates
all three exhaustively and picks the lowest total. A specification that reached for a solver here
would be adding machinery to a problem that does not have enough variables to need one. Which
candidate a buyer ultimately selects is the buyer's own call — the lowest total, or a costlier one
they judge faster — and TRACT does not compel the choice.

> **PROVISIONAL — pending decision (route-choice reproducibility, §8.8).** Whether consolidation
> route choice must be **reproducible across independent implementations** given the same inputs, or
> whether two conformant buyer nodes may legitimately land on different totals because their copies
> of the same rate card are at different points within `RATE_CARD_MAX_AGE`, is unresolved. The
> scoring formula above is normative; the requirement that two nodes reach the *same* total from the
> *same* nominal inputs is not. Until this is settled, an implementation MUST NOT rely on another
> node having computed an identical total, and MUST NOT treat a divergent total as evidence of
> non-conformance. Recorded for the founder-decision list.

## 8.6 Distributors: the consolidation role

A **distributor** publishes a `CapacityRecord` (§16.5.3) — country, coarse locality, storage per
item per day, a handling fee, available slots, and excluded categories — and holds goods in transit
on that basis. A distributor MUST publish a `CapacityRecord` to be selectable as a consolidation
hub; a hub with no published record cannot be scored by §8.5 and MUST NOT be routed through. Entry
is permissionless: a keypair and space is the whole requirement, the same bar as every other role
in §0.4.1. A `CapacityRecord` is stale past `RATE_CARD_MAX_AGE` on the same terms as a `RateCard`
(§8.2's staleness rule and its PROVISIONAL split).

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
  required instead of pricing against a card, precisely for this case — but note that the frozen §16
  carries **no production** for an `Offer` to point at a specific `RateCard`, nor a delivery-specific
  "live quote required" flag distinct from the `Consideration` `quote-required` variant (§16.5.2 key
  8, which prices the goods, not the carriage). The base offer-pull flow does not need one — the
  buyer selects any carrier whose card serves the lane — but pinning a carrier or flagging
  shipping-needs-a-quote both require a §16 grammar slot that does not exist yet, recorded as a
  needed §16 change (§8.8, §16.8).
- **Transit figures are estimates, not commitments.** `transit_days` on a `Zone` (§16.5.3) is the
  carrier's own claim, and `TRANSIT_TIMEOUT` (carrier estimate × 3, §19.3) is built around that
  being unreliable rather than around it being wrong by a fixed margin. A `transit_days` figure MUST
  be presented to a buyer as an estimate, never as a delivery commitment.
- **`wait_days` is the least reliable input in the whole computation.** It depends on the slowest
  seller in a consolidation actually dispatching on time, which is outside any rate card's or
  distributor's control, and it MUST be presented to a buyer as an estimate rather than folded into
  a total that reads as a promise.
- **Custody attestations prove transfer, not recoverability.** A signed handoff establishes who
  held the goods and when. It establishes nothing about getting them back. `lost` is a terminal
  state in §18.4 (WRAP's machine) with a signed history attached and no remedy attached — where the
  goods went is answerable; being made whole for them is §9's problem, or nobody's.
- **This section is unevidenced.** Three research passes over the literature returned no verified
  findings on logistics standards, carrier API terms, or consolidation optimisation (§21.1); the
  gap is recorded there as absence of evidence, not evidence of absence. Everything above is design
  reasoning checked for internal consistency, not a claim checked against how carriers, distributors
  and couriers actually behave, and it MUST be read with that confidence level and no more. This
  section MUST NOT cite §21 as *support* for any of its logistics reasoning (§21.1, C6).

## 8.9 The seam with WRAP — on-demand dispatch and custody are not TRACT's to re-specify

> **Drafting decision (2026-07-23).** Recorded here so this section's normative text, when written,
> does not duplicate a sibling spec. Delivery has **two** matching modes, and only one of them is
> TRACT's.

TRACT owns the **rate-card** mode above: a carrier or distributor publishes a standing signed offer
(`RateCard`, `CapacityRecord`), and the buyer's node *pulls* — computes a price and picks. That is
offer-pull commerce, the same shape as the rest of TRACT, and it stays here.

The **on-demand** mode is different and is [WRAP](https://github.com/vul-os/wrap)'s, not TRACT's: a
leg with no standing rate card — a same-hour courier, a peer with a bicycle, a gig dispatched to
whoever will take it — is a *request* posted to a pool, bid on, and **assigned by the issuer**. That
is WRAP's `WorkOrder → Bid → Assignment` exactly (WRAP §3), including its "the issuer assigns"
race-free dispatch. TRACT **references** WRAP for this leg rather than re-deriving a dispatch model:
a TRACT order that needs on-demand carriage emits a WRAP `WorkOrder` under WRAP's delivery profile,
whose assigned performer is the courier, and folds the result back as the leg's custodian. Both
specs sit on the same DMTAP substrate and the same `IK` identity, so the courier's key, signatures,
and reputation are shared, not bridged.

**The custody-handoff lifecycle is shared, and lives in WRAP.** The chain
`created → accepted → in-custody → handed-off → delivered`, signed by the party *taking* custody
(§8.4, §18.4), is the same signed-progress-and-attestation model WRAP already defines
(`Progress` + `Attestation`, WRAP §3.7–§3.8). It MUST be specified **once** — as WRAP's delivery
profile — and referenced from TRACT §8.4 and §18.4, not written a second time here. Whichever mode
carried the leg (rate-card or on-demand), the physical custody chain is identical WRAP objects, so a
handoff proves the goods moved regardless of how the carrier was chosen. This is the single largest
redundancy the two specs could fall into, and this decision closes it before the normative text is
written.

## 8.8 Open — collected for the founder-decision list

Each item below is marked **PROVISIONAL** in the subsection that raises it. None may be resolved by
inventing a founder call; two interact with a §16 grammar change.

- **Peer-courier rate-card shape (§8.2).** Whether a peer courier needs a distinct rate-card shape —
  flat price per kilometre — or should be expressed inside the same zone-table `RateCard` model as a
  national carrier, at the cost of a zone table with one entry. A per-kilometre variant is a §16
  **MAJOR** grammar change (§16.5.3); the single-entry zone table needs none.
- **Per-object `RATE_CARD_MAX_AGE` (§8.2, §8.6).** Whether `RATE_CARD_MAX_AGE` should differ between
  a `RateCard` and a `CapacityRecord`. Zone tables and storage rates plausibly drift at different
  speeds, and today one parameter (§19.6) covers both.
- **Reproducibility of consolidation route choice (§8.5).** Whether consolidation route choice needs
  to be reproducible across independent implementations given the same inputs, or whether two
  conformant buyer nodes may legitimately land on different totals because their copies of the same
  rate card are at different points within `RATE_CARD_MAX_AGE`.

**Needed §16 grammar change (§8.7).** The frozen §16 carries no production linking an `Offer`'s ship
`Fulfilment` (§16.5.2) to a specific `RateCard`, and no delivery-specific "live quote required" flag
distinct from the `Consideration` `quote-required` variant. The base offer-pull flow does not need
one; pinning a carrier to an offer or flagging that carriage needs a live quote does. Recorded as a
required §16 open item (§16.8), not invented here.
