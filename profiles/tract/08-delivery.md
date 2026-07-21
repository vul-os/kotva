# 8. Delivery

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 8.1 Scope

Rate cards, legs, consignments, consolidation, and local route computation.

## 8.2 What this section will specify

- `RateCard` as a signed public object: zone table, weight brackets, dimensional divisor,
  surcharges, transit estimates, served countries, excluded categories.
- The **local computation** rule: a buyer's node computes leg prices from published rate cards
  rather than calling a quote API — so there is no rate limit, no API key, and no third party
  learning what is being bought.
- `Leg` and `Consignment` objects, and signed **custody handoff** attestations.
- **Consolidation**: the candidate routings (direct, hub-near-buyer, hub-near-sellers) and the
  comparison `Σ legs + storage × wait + handling`. The candidate set is small enough to evaluate
  exhaustively; no optimiser is required or specified.
- Distributor capacity objects.

## 8.3 Standards profiled

Incoterms 2020 for transfer of risk. GS1 SSCC for logistic-unit identification where the parties
already use it. ISO 3166 for served territories. UPU conventions for cross-border postal legs.

## 8.4 Honest limits this section must state

- Some carriers restrict redistribution of negotiated rates. Published list rates and a seller's
  own rates they choose to publish are fine; republishing confidential rates is the publisher's
  compliance problem. Offers may declare that a live quote is required instead.
- Transit estimates are estimates. `wait_days` for consolidation is the least reliable input in
  the whole computation and must be presented as an estimate.
- Custody attestations prove **transfer**, not **recoverability**. Loss and damage are §9's
  problem, or nobody's.

## 8.5 Open

- Whether a peer courier needs a distinct rate-card shape (flat per-km) or should express that
  within the zone-table model.
