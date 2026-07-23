# 5. Consideration

> **Drafting status: partially normative.** The six pricing variants and the money rules
> (§5.3–§5.9, §5.11) are authored to normative RFC 2119 text, aligned exactly to the frozen
> §16.5.2 `Consideration` grammar. §5.9a (competitive bidding and auctions reuse WRAP) and §5.10
> (tax treatment categories, never rates) are founder-settled (2026-07-23) and preserved verbatim.
> What remains **scoped, not normative** is every item in §5.13: each is marked inline as
> **PROVISIONAL — pending decision**, because settling it would either require a §16 MAJOR grammar
> change (a shared-currency rule inside a `Consideration`, a tax-category slot, a sealed
> quote-request object, a metered usage-attestation object) or a founder call this section must not
> invent. The key words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are to be interpreted as in
> BCP 14 (RFC 2119, RFC 8174) wherever they appear below.

## 5.1 Scope

The fourth offer axis (§2.3): what is paid, on what schedule, and how tax attaches to it. This
section owns the reasoning behind each `Consideration` variant and the rules for money; the byte
authority is §16.5.2, and this section MUST NOT contradict it.

## 5.2 Variants

An `Offer` carries exactly one `Consideration` (§16.5.2, key 4), and a `Consideration` is exactly
one of the six variants below. A decoder MUST reject a `Consideration` that matches none of them,
or more than one.

| Variant | Grammar (§16.5.2) | Covers |
|---|---|---|
| `fixed` | `{ 0 => money }` | a single price |
| `tiered` / `volume` | `{ 1 => [+ PriceTier] }` (min_qty, unit_price) | cheaper per unit above a threshold |
| `recurring` | `{ 2 => money, 3 => tstr }` — amount, RFC 5545 `RRULE` | subscriptions |
| `metered` | `{ 4 => tstr, 5 => money }` — dimension, unit price | usage billed after the fact |
| `deposit + balance` | `{ 6 => money, 7 => money }` | part now, remainder on delivery or completion |
| `quote-required` | `{ 8 => null }` | RFQ, and B2B contract pricing |

## 5.3 Money is minor units and a currency code, never a float

Every monetary field of a `Consideration` MUST be a `money` value (§16.3):

```
money = { 1 => int, 2 => currency }   ; minor_units (ISO 4217), currency code
```

A `money` value MUST NOT be a floating-point number, and an implementation MUST NOT introduce one
at any point on the path from a UI to a signed object. This restates §16.7 at the point it bites
hardest: a `Consideration` value is either signed directly inside a published `Offer` (§16.5.2) or
carried into a sealed `Order` whose transitions are themselves signed (§18.3). A float invites a
value like 19.999999999998 to enter through a UI slider or a percentage calculation, and once that
value is signed, the signature covers the wrong number — there is no later step at which it can be
corrected, only a new object that supersedes the old one and a gap in between where the wrong figure
was the authoritative one. Integer minor units foreclose the error at the type: 1999 either is or is
not what was meant, with nothing in between for a rounding step to introduce.

## 5.4 Cross-currency arithmetic is refused, not coerced

A `money` value carries exactly one currency. An implementation MUST refuse — and MUST NOT silently
coerce — any arithmetic (summation, comparison of magnitude, netting) across two `money` values
whose currency codes differ. A rate is a claim about a moment in time that neither party necessarily
agreed was authoritative — unlike a price, which the seller signed directly. Converting one side and
adding therefore produces a total that looks exact and is not one either party actually committed
to. A sum is defined only across values sharing one currency.

This is why an `Order`'s `total` (§16.6) is a single `money`: one order names one seller (§16.6,
"one order per seller is a grammar-level property"), and therefore **one order settles in exactly
one currency**. An implementation that receives lines it would have to sum across currencies MUST
refuse to place the order rather than convert.

**PROVISIONAL — pending decision.** The grammar does not yet enforce a shared currency **within** a
single `Consideration` value: `deposit + balance`'s two `money` fields (§5.8), and each
`PriceTier`'s `unit_price` (§5.5), each carry their own independent currency code. Nothing in the
frozen grammar today rejects a deposit denominated in one currency against a balance denominated in
another, even though the two are instalments of one price rather than two independent prices. Making
this a hard rule requires a §16 shape change (a MAJOR version bump) — recorded in §5.13 and in the
grammar-change list, not invented here. Until then, an implementation SHOULD require all `money`
values inside one `Consideration` to share a currency, and SHOULD reject an offer that mixes them,
but the grammar cannot yet make this MUST.

## 5.5 Tiered / volume pricing

`{ 1 => [+ PriceTier] }` carries at least one `PriceTier`, each a `(min_qty, unit_price)` pair
(§16.5.2). To price a requested quantity `q`, an implementation MUST select the tier with the
greatest `min_qty` not exceeding `q`, and MUST charge `q × unit_price` at that tier's price. This is
standard volume-pricing selection.

**PROVISIONAL — pending decision.** Two properties the frozen grammar does not pin down, and which
this section therefore cannot yet make normative:

- **Ordering and duplicates.** `ProductRecord`'s attribute list is canonicalised — sorted and
  deduplicated (§16.5.1) — but the `PriceTier` array carries no equivalent rule, so two tiers naming
  the same `min_qty` are not excluded and their relative order is undefined. A canonical-ordering and
  duplicate-`min_qty`-rejection rule is a §16 change (§5.13).
- **Coverage below the lowest tier.** `[+ PriceTier]` guarantees at least one entry, not one whose
  `min_qty` is 1, so a quantity below every published tier's threshold has no defined price. Whether
  such a quantity is unpriceable (the offer refuses it) or falls to the lowest published tier is a
  founder call (§5.13). Until it is settled, an implementation MUST NOT invent a price for a quantity
  below every tier; it MUST treat the offer as not applicable to that quantity.

## 5.6 Recurring — reusing §3's calendar machinery

`{ 2 => money, 3 => tstr }` carries a per-period amount and an RFC 5545 `RRULE`, the same standard
§3 profiles for availability, so a seller who already runs billing or calendar software against
RFC 5545 does not need a second schedule format for a subscription period. The amount is fixed per
period: a recurring charge that also varies by usage is `metered` (§5.7), not `recurring`, and an
offer MUST NOT encode usage-varying charges as `recurring`. A trade needing both a periodic fee and
metered usage is two axes' worth of behaviour that one `Consideration` value cannot express, and an
implementation MUST NOT attempt to fold both into a single `recurring` value.

**PROVISIONAL — pending decision.** How a recurring consideration interacts with §18.3's order state
machine is unresolved. §18.3 runs one lifecycle from `draft` to a terminal state and does not loop.
Whether each renewal period is a fresh `Order` — the leaning answer, since `Order` is a lightweight
sealed object and "was this specific charge accepted" is naturally per-period — or one `Order` that
persists across periods, is not decided, and §18.3 as written describes no renewal path either way.
An implementation MUST NOT assume a renewal semantics this document has not specified. Recorded in
§5.13.

## 5.7 Metered

`{ 4 => tstr, 5 => money }` carries a dimension string (calls, kWh, gigabytes) and a unit price,
billed after consumption is known. The unit price and dimension are settled and MUST be encoded as
above.

**PROVISIONAL — pending decision.** Two things the frozen grammar does not resolve, and which block
metered from being fully normative:

- **No usage-attestation object exists.** §16.6 defines no object for reporting, attesting, or
  disputing how much of the metered dimension was actually consumed, so the mechanism by which a
  metered charge's final figure is attested by one party and verified — or contested — by the other
  is unspecified. Adding one is a §16 change (§5.13).
- **`Order.total` is mandatory and a metered total is not knowable up front.** §16.6 defines `total`
  as key 4 with no `?`: it is required on every `Order`, set at `draft`/`placed` time, while a
  metered charge is only known after fulfilment. Whether `total` for a metered order is an estimate,
  a cap, or whether metered consideration needs a different order shape is a founder call (§5.13). An
  implementation MUST NOT silently place a metered `total` as if it were the final settled figure.

## 5.8 Deposit + balance

`{ 6 => money, 7 => money }` carries a deposit payable at order and a balance payable at delivery or
completion — the shape a genuine deposit-taking trade has, rather than two unrelated fixed prices.
It pairs naturally with `return-required` fulfilment (§16.5.2, `Fulfilment` key 6; §4.7): the
deposit is the mechanism that makes a late-return or damage claim on a rental whole, and §4.7 is
explicit that condition and lateness are handled as an order-level dispute, not as a fulfilment-axis
field. The deposit and balance are instalments of one price; §5.4's shared-currency requirement
applies (SHOULD today, MUST once the §16 rule lands), and this is the sharpest instance of that gap.

## 5.9 Quote-required — RFQ and B2B contract pricing

`{ 8 => null }` publishes only that no price is published and that a buyer must ask. The published
offer carries no `money` at all.

A seller's response to a quote request needs no new object type: it is an ordinary `Offer` — `fixed`,
`tiered`, or whatever consideration the seller actually wants to charge this buyer — distributed
differently from the rest of the catalogue, handed directly to the requester's key rather than
published in the open feed. B2B contract pricing works under the same mechanism: a per-buyer price is
nothing more than an `Offer` addressed to one key instead of broadcast to all of them. No
pricing-tier object and no buyer-group primitive is required; the existing `Offer` grammar already
expresses "this price, for you specifically" the moment its distribution, not its shape, is
restricted. An implementation MUST NOT introduce a separate per-buyer pricing object for this case.

**PROVISIONAL — pending decision.** Two things this leaves unspecified, both of which would require
a §16 change:

- **The request itself has no defined wire shape.** A buyer asking for a quote is plausibly sealed —
  it may disclose volume or identity a seller would rather not answer about openly — but §16.6
  defines no such object today (§5.13).
- **Nothing links a private per-buyer `Offer` back to the request that produced it.** Without that
  link, a buyer holding such an offer has a signed record of having *received* it, not of having
  *asked* for it (§5.13).

## 5.9a Competitive bidding and auctions reuse WRAP — they are not a new mechanism

`quote-required` (§5.9) is *private pricing*: one buyer asks, one seller answers with an `Offer`
addressed to their key. It is not an auction, and TRACT needs no bidding engine of its own for the
cases that are — because [WRAP](https://github.com/vul-os/wrap) already is one. WRAP's
`Offer → Bid → Assignment` (with `Offer.mode`: direct / open-bid / sealed-bid) is the single bidding
primitive on this substrate, and every auction shape is a *profile* of it with one actor as the
assigner. Two shapes matter for commerce:

- **Competitive procurement / reverse auction** — a buyer posts a need and *many* providers bid; the
  buyer picks. This is WRAP's request-assign direction exactly (the buyer is WRAP's issuer). A TRACT
  buyer wanting competitive quotes rather than one private answer emits a WRAP `WorkOrder`, collects
  `Bid`s, and assigns — TRACT **references** WRAP rather than growing a second bid object. The
  resulting winning offer re-enters TRACT as an ordinary `Offer` addressed to the buyer, so the order,
  settlement, and delivery spine are unchanged.

- **Forward auction** — a seller lists one scarce good and *buyers* bid up; the highest valid bid at a
  close time wins. This is the harder case, because a "who won at close" race is precisely what the
  request-assign model removes elsewhere. It is solvable by the same trick and no other: **the seller
  is the natural arbiter — the seller assigns the good to the winning bid at close** ("whoever owns
  the goods decides who gets them," the mirror of WRAP's "whoever issued the work assigns it"). This
  is specified as an **optional auction profile** over WRAP's `Bid` object, not core, and its residual
  is stated honestly: a partitioned seller cannot close, and sealed-vs-open bid visibility
  (`Offer.mode = 2`) must be chosen per auction. It reintroduces a bounded version of the close-time
  race WRAP otherwise avoids, in exchange for expressing genuine ascending auctions at all.

The design rule: **one bid→assign engine, many profiles, the issuer/seller always the assigner.**
TRACT does not define a competing auction object, and a `Consideration` never carries bidding state —
bidding lives in WRAP's namespace, and its outcome arrives here as a settled `Offer` or `Order`.

## 5.10 Tax treatment categories belong here; tax rates do not

The offer carries a **treatment category** — standard-rated, zero-rated, exempt, reduced, and
whatever taxonomy a given regime uses — plus the anchors §11.2 derives, principally the place of
supply this axis's counterpart, §4, computes. It does not carry a **rate**. Rates change by
jurisdiction and by week; encoding one into a specification guarantees the specification ships stale
law the day a legislature changes a number. Rate lookup, keyed by treatment category, place of
supply, and the applicable date, is deliberately left to whatever source an implementation trusts —
a jurisdiction's published schedule, a commercial rate service — never to a table in this document.

Neither `Consideration` nor `Offer` currently has a field for the treatment category itself
(§16.5.2) — this section states the policy the grammar has not yet been given a slot to carry.

## 5.11 Currency conversion is a presentation concern

An offer's `money` is denominated in one currency, and that is the currency a resulting `Order`'s
`total` and any `PaymentAttestation` (§16.6) MUST be bound to — the **settlement currency**. A
storefront or client MAY show a buyer a converted estimate in a currency they prefer while browsing,
computed live against whatever FX source that display layer trusts, and it MUST mark that figure
visibly as an estimate rather than a price. A converted figure MUST NOT become what gets signed:
§5.4 and §16.7 are the same rule applied at two different points — refused in arithmetic, estimated
only in presentation.

## 5.12 Standards profiled

ISO 4217 currency codes (§16.3 `currency`). RFC 5545 recurrence rules, reusing §3's machinery rather
than inventing a second schedule format. No new pricing, currency, or scheduling vocabulary is
introduced by this section.

## 5.13 Open — collected for the founder-decision list

Each item below is marked PROVISIONAL in the subsection that raises it. None may be resolved by
inventing a founder call; several require a §16 MAJOR grammar change.

- **Shared currency inside a multi-`money` `Consideration`.** Whether `deposit + balance`, and each
  `PriceTier`'s `unit_price` against its neighbours, need a grammar-level rule requiring one currency
  (§5.4, §5.8). Requires a §16 change.
- **`PriceTier` canonicalisation and coverage.** Whether `PriceTier` needs a canonical ordering rule
  and duplicate-`min_qty` rejection the way `ProductRecord`'s attributes have, and what a quantity
  below every published tier resolves to (§5.5). Requires a §16 change for the first two.
- **Recurring vs the single-shot order lifecycle.** How a `recurring` consideration's renewal periods
  map onto §18.3 — fresh `Order` per period, or some other shape the state machine does not currently
  describe (§5.6).
- **Metered attestation and mandatory total.** Whether `metered` needs a usage-attestation object,
  and how a metered order satisfies `Order.total` as a mandatory field when the true amount is only
  known after consumption (§5.7). Requires a §16 change for the attestation object.
- **Quote-request wire shape and back-link.** Whether a quote request needs a defined sealed wire
  shape, and whether a private per-buyer `Offer` needs a field linking it back to the request that
  produced it (§5.9). Requires a §16 change.
- **Where the tax treatment category is carried.** Neither `Consideration` nor `Offer` has a field
  for it today (§5.10). Requires a §16 change.
- **The forward-auction profile shape over WRAP's `Bid`.** Close-time semantics, sealed-vs-open
  visibility, and how a seller-assigned winning bid re-enters TRACT as an `Order` (§5.9a). Competitive
  procurement needs no new shape (it is WRAP unchanged); the forward auction does — and it lives in
  WRAP's namespace, not TRACT's.
