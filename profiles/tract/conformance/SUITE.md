# TRACT Conformance — Planned Case Catalogue

Every row below is **PLANNED**. None is backed by a vector, none is runnable, and no count in
this file should be read as a claim about coverage — see `conformance/README.md` for why: §16
(wire format) is not yet normative, so there are no committed bytes for any case to check against.
This catalogue exists to grow the case list section-by-section, alongside the normative text, so
cases are not invented in bulk after the fact from whatever an implementation happens to do.

Cases are grouped by the TRACT section they pin, in the order of §0.8's document map. Each
targets an invariant that would fail **silently** if violated — a wrong answer that looks like a
right one, rather than a crash — because those are the ones a case catalogue is worth having.
Loud failures mostly catch themselves.

## How to read a row

| Field | Meaning |
|---|---|
| **id** | `TRACT-<CATEGORY>-<NN>`, stable once assigned, never reused. |
| **profile** | The §15.2 profile the case belongs to: catalogue-only, transacting, routing, or gateway. A case may name more than one when the invariant applies across profiles. |
| **clause** | The section (and subsection, where the stub prose already commits to one) the case pins. |
| **will assert** | What a conforming implementation must do, in the language of a future case, not a MUST — nothing here is normative text and none of it should be read as if it were. |

A **status** column is deliberately omitted: every row in this file has the same status,
PLANNED, and repeating it 60 times would just be noise. A row moves out of this file and into a
vectored suite when — and only when — the clause it cites has gone normative and a vector has
been derived from that clause's text per the discipline in `conformance/README.md`.

---

## Public/sealed type confusion and the personal-data floor (§0.5.1, §16.4)

The split between the public and sealed quadrants (§0.5) is the single rule most of the rest of
the document leans on. A case family exists here because the failure mode is not "the object
is malformed" — it is "the object is well-formed, and wrong."

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-PUBSEAL-01 | catalogue-only | §0.5.1, §16.4 | a `ProductRecord`, `Offer`, `RateCard`, `CapacityRecord` or storefront render bundle carrying a field that identifies or is linkable to a natural person is rejected on decode, not merely flagged for review — the prohibition is structural, not a linting pass over otherwise-valid bytes |
| TRACT-PUBSEAL-02 | transacting | §16.4 | a decoder presented with a sealed `Order` object under a public-object schema (or vice versa) rejects it as a type mismatch, never as a validation failure inside the wrong schema — the two families are non-confusable by construction, not distinguished only by a `sealed: true` flag an attacker could omit |
| TRACT-PUBSEAL-03 | catalogue-only | §0.5.1, §10.4 | a `Review` — the one object that is public and person-signed by design (§10.3) — is rejected if it carries any field outside the bounded set §10.4 permits (buyer name, address or contact in particular), so the one deliberate exception to §0.5.1 cannot be used to smuggle the rest of a personal-data record in beside it |
| TRACT-PUBSEAL-04 | gateway | §12.2, §0.5.1 | a storefront render bundle that embeds buyer-identifying data supplied at browse time (rather than data the seller published) is refused by the gateway rather than rendered, because rendering it would republish sealed-quadrant data through the public-quadrant serving path |

## Catalogue (§2)

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-CAT-01 | catalogue-only | §2.2, §2.3 | two independently-published `ProductRecord` objects that canonicalise to identical bytes converge to the identical content address; two records differing before canonicalisation — including differing only in field order before canonical CBOR encoding — do not silently collide |
| TRACT-CAT-02 | catalogue-only | §2.6 | a client presented with an index's claim about an offer that disagrees with the seller's own feed for that offer prefers the feed, in every case, with no configuration that inverts the precedence — an index is authoritative over nothing |
| TRACT-CAT-03 | catalogue-only | §2.4 | an `Offer` object is rejected if it does not declare all four axes (Item, Availability, Fulfilment, Consideration) — a partially-specified offer is not accepted with the missing axis defaulted, because a silent default here is a silent price or terms change |
| TRACT-CAT-04 | catalogue-only | §2.3 (identity ladder) | a claimed external identifier (GTIN, MPN) on a product record is treated as advisory and unverified, never as proof of authority — a client MUST NOT rank or dedupe two records more confidently than the content-address floor supports merely because they share a claimed identifier |

## Availability and fulfilment (§3, §4)

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-AVAIL-01 | transacting | §3.2, §6.2 | a buyer's node re-evaluates a cart line's availability against the seller's live signal before checkout, and checkout on a line whose availability has gone to zero since it was added fails closed rather than proceeding on the cart's stale snapshot |
| TRACT-FULF-01 | catalogue-only, transacting | §4.3, §11.2 | the place-of-supply anchor derived from an offer's Fulfilment variant is independent of seller establishment and buyer residence — the forcing case: an event held in country C, sold by a seller established in country A, to a buyer resident in country B, derives place of supply as C from the Fulfilment object alone, and a client that derives it from seller or buyer location instead is non-conformant |
| TRACT-FULF-02 | transacting | §4.3 | a multi-variant offer (e.g. collect *or* ship) binds its place-of-supply and delivery-destination anchors only once the buyer's choice is recorded on the order — computing tax or customs treatment before that choice is made is treated as a defect, not an optimisation |

## Cart (§6)

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-CART-01 | transacting | §6.2 | a seller's bounded-counter inventory, partitioned across replicas by quota, never permits total sales across all replicas to exceed total stock — concurrent sales against two replicas that each believe they hold available quota do not both succeed when their combined quota is exhausted |
| TRACT-CART-02 | transacting | §6.4 | a partitioned replica holding unused quota it cannot currently transfer is surfaced as stranded stock, not silently treated as sold out network-wide — the two states must remain distinguishable to an operator |
| TRACT-CART-03 | transacting | §6.4 | a multi-seller checkout where one seller declines does not roll back, retry, or otherwise touch an already-accepted order from a different seller in the same cart — the interface presents independent per-seller status, and an implementation that models checkout as a single atomic transaction across sellers is non-conformant |

## Order (§7)

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-ORDER-01 | transacting | §7.2 | every transition in the order state machine (placed, accepted, declined, countered, fulfilling, delivered, closed, cancelled) is a signed object attributable to the party whose action it represents — a state change with no corresponding signature from the acting party is rejected, not merely unattested |
| TRACT-ORDER-02 | transacting | §7.2, §18.3 | every timeout edge in the order state machine has a defined expiry behaviour, and a node that reaches a timeout with no counterparty response executes that behaviour rather than leaving the order in an undefined state indefinitely |
| TRACT-ORDER-03 | transacting | §7.2 | a sealed `Order` object is never observable by any party other than its two endpoints — a conformance harness that can see a third party (an index, a relay, a cache) holding order content flags it as a violation regardless of whether that third party could decrypt it |

## Delivery (§8)

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-DELIV-01 | routing | §8.2 | a buyer's node computes a leg's price and transit estimate from a published `RateCard` object locally, without a network call to the carrier at quote time — a client that requires a live API call to price a route is non-conformant to the routing profile |
| TRACT-DELIV-02 | routing | §8.4 | a signed custody-handoff attestation is treated as proof of **transfer** only — a client or index that represents it as proof of recoverability, insurance, or successful final delivery misstates what the attestation carries |
| TRACT-DELIV-03 | routing | §8.2 | route-total currency mismatches across legs priced in different currencies are surfaced explicitly (as a conversion the buyer's node performed, with the rate and source disclosed) rather than summed as if the units were the same |

## Settlement (§9)

This is the section §15.3's fail-closed set names first — scope-intersection and rail-class
substitution are called out there by clause, and the cases below are that clause turned into
assertions.

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-SETTLE-01 | gateway | §9.4 | at checkout, the buyer's declared `EscrowScope` and the gateway's offered scope are intersected on every field (countries, currencies, rail classes, value ceiling, excluded categories); an **empty intersection is refused and disclosed as a refusal**, never silently narrowed to whichever subset happens to validate |
| TRACT-SETTLE-02 | gateway, transacting | §9.3 | a rail's class (`CustodialReversible` vs `NonCustodialFinal`) is never substituted for the other without both parties' explicit agreement recorded on the order — an implementation that falls back from a declined custodial rail to a non-custodial one without a fresh agreement changes the buyer's recourse without their knowledge, and that is treated as a security defect, not a UX shortcut |
| TRACT-SETTLE-03 | gateway | §9.4 | a `PaymentAttestation` never carries funds and is rejected if it is the only signed object standing in for a settlement event — the protocol only ever carries an attestation *about* a rail-side transfer, never the transfer itself |
| TRACT-SETTLE-04 | gateway | §9.4 | an escrow ruling (release, refund, split) is always a signed object published by the gateway that made it, retrievable by both parties after the fact — a ruling with no corresponding signed record is treated the same as no ruling at all |

## Trust (§10)

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-TRUST-01 | transacting | §10.2 | a `Review` lacking a valid purchase attestation from the seller or an escrow operator is rejected by a conformant index — reviews are never accepted on the strength of the review's own signature alone |
| TRACT-TRUST-02 | catalogue-only | §10.3 | no client, index or gateway computes or serves a single network-wide score for a seller, product, courier or distributor — an aggregation surface that presents itself as *the* rating rather than *an* index's derived rating is non-conformant to the prohibition, regardless of how the number is computed |
| TRACT-TRUST-03 | transacting | §10.4 | a review's supersede-based retraction is honoured by conformant clients and gateways going forward (the tombstone is served in place of the retracted review), while the documentation and UX around retraction do not claim the original bytes are unreachable everywhere — the residual is disclosed, not hidden by the retraction succeeding locally |

## Jurisdiction (§11)

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-JURIS-01 | transacting | §11.3 | when an order requires an in-region responsible person (per the applicable regime) and none is present on the offer or order, construction of the order fails closed rather than completing with the field silently absent |
| TRACT-JURIS-02 | transacting | §11.2 | the four jurisdictional anchors (seller establishment, buyer residence, place of supply, delivery destination) are carried as four independent fields on the order, and a client that collapses any two of them into one value — most commonly place of supply into delivery destination — is non-conformant, per the forcing example in §11.2 |

## Gateway (§12)

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-GW-01 | gateway | §12.3 | two stores rendered by the same gateway are served from two distinct origins; a render bundle for one store that can read another store's cart or session state (via a shared origin) is a conformance failure, not a hardening recommendation, once §12.3 is normative |
| TRACT-GW-02 | gateway | §12.4 | the rendering process has no access to identity keys or the raw object store — a conformance harness that can reach either from the code path serving an untrusted render bundle fails the case regardless of whether an exploit was demonstrated |
| TRACT-GW-03 | gateway | §12.5 | independently re-rendering a store's public objects from a different gateway (or no gateway) produces output that is comparable byte-for-byte to the original gateway's render, so the detectability mitigation §12.5 relies on is actually available, not merely asserted |

## Anti-abuse and feed continuity (§14, §2, §0.3)

Feed continuity is inherited unchanged from the DMTAP substrate's Feeds capability (§0.3,
capability ②) rather than respecified here, but the invariant still needs a TRACT-side case: a
seller's catalogue, rate-card and offer feed is the thing a buyer's node trusts for price and
stock, so a rollback of that feed is a silent lie about what is currently on offer.

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-ABUSE-01 | catalogue-only | §0.3 (capability ②), §2 | a holder presenting an older signed feed head for a seller's catalogue after a newer head has already been observed is rejected as a rollback, not accepted as a slower-arriving update — this covers the case of a price rise or a withdrawn offer being made to silently reappear by replaying a stale head |
| TRACT-ABUSE-02 | catalogue-only | §14.2 | a seller's feed-append rate and per-publisher storage quota are enforced by holders independently of any other seller's feed — one seller's flood does not cause a holder to stop serving or degrade service for an unrelated seller's feed |
| TRACT-ABUSE-03 | transacting | §14.3 | an unsolicited sealed order from a sender with no prior relationship is subject to the substrate's cold-contact economics (challenge/proof-of-work gating) before it reaches a seller's inbox, exactly as an unsolicited message would under the substrate's own anti-abuse tiers — TRACT introduces no separate, weaker path for orders |

## Conformance profiles, registries and state machines (§15, §17, §18)

| id | profile | clause | will assert |
|---|---|---|---|
| TRACT-PROFILE-01 | catalogue-only | §15.2 | a node advertising only the catalogue-only profile rejects an inbound sealed order rather than silently accepting and ignoring it — a profile boundary is enforced at the point of receipt, not left to the application layer to notice the node cannot act on what it received |
| TRACT-PROFILE-02 | transacting, routing | §15.2 | a node advertising the transacting profile without the routing profile does not compute delivery routing on the buyer's behalf and does not claim to — the profile a node advertises is the profile a conformance harness can rely on it actually implementing |
| TRACT-REG-01 | catalogue-only | §17.4 | a generic index encountering an unrecognised value in an extensible registry (item kind, fulfilment variant, rail class, etc.) preserves and surfaces it as unknown rather than dropping the object; a client that would *render or transact* against that same unrecognised value refuses instead of guessing at its meaning — the two behaviours (tolerant to store, strict to act) are independently checked, because a case that only checks one could pass an implementation that guesses when it should refuse |
| TRACT-SM-01 | transacting, routing, gateway | §18.3 | every state machine defined in §18 (offer, order, consignment, escrow) has, for every transition, a defined behaviour for the case where the expected timeout elapses with no counterparty response — a state machine with a reachable state that has no timeout-expiry edge fails this case regardless of whether that state is ever observed in practice |

## What is deliberately not here yet

§13 (analytics) and §16 (wire format) have no cases in this catalogue. §13's stub prose does not
yet commit to a checkable object shape or a testable boundary — "tiered and buyer-granted, with
an aggregate floor" is a posture, not yet a rule a case can assert pass/fail against. §16 has no
cases of its own because every case above that touches wire bytes (TRACT-PUBSEAL-\*,
TRACT-CAT-01, TRACT-ABUSE-01) already cites §16's structural rule where it applies; §16 going
normative is what turns those rows from PLANNED into vectored, not a trigger for a new row.
