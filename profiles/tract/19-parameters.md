# 19. Parameters

> **Drafting status. Partially normative.** The parameter *values*, their *class* (Fixed protocol
> constant / Declared wire-carried tunable / Substrate-inherited), and the validation rules over
> them are now normative. The key words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are to be
> interpreted as in BCP 14 (RFC 2119, RFC 8174) where they appear in capitals.
>
> Four entries remain **PROVISIONAL — pending a founder decision** and are marked inline where they
> occur: `CONFIRM_TIMEOUT`'s floor-vs-offer-term status (§18.7); `MAX_REVIEW_BODY`'s size (§19.8);
> whether `RATE_CARD_MAX_AGE` is a single Fixed constant or a per-card Declared value (§8.8); and
> `DISPUTE_TIMEOUT`, which has no machine-readable wire slot. `CLOCK_SKEW` is **not** set here — it
> is defined by the DMTAP substrate and referenced, never redefined.
>
> Every numeric value below is a **chosen default, not a field measurement.** The logistics (§19.2
> `FULFIL_TIMEOUT`, §19.3, `MIN_DIM_DIVISOR`, `RATE_CARD_MAX_AGE`), trust (`MAX_REVIEW_BODY`) and
> analytics (`GRANT_LIFETIME`) parameters rest on design reasoning only, because those areas
> returned nothing verified across the grounding passes (§21.1). Making a value normative fixes
> *what implementations must agree on*, not *that the value is right*; this section MUST NOT be read
> as evidence for the latter (§21.1, §21.9), and MUST NOT be cited as support for §8, §10 or §13.

## 19.1 Why this is its own section

Parameters scattered through prose drift out of agreement with each other, and nobody notices until
two sections cite different values for the same thing. Collecting them lets the linter check that a
figure quoted in §N matches the table, and lets an implementer see the whole configuration surface
at once instead of discovering it a section at a time.

The values below are **normative defaults, not measurements**. Where a value is a genuine trade-off
rather than an obvious floor, the trade-off is stated beside it — a number with no reasoning
attached is a number the next person changes arbitrarily.

## 19.1a How to read this registry

Every parameter carries a **class**, and the class decides who may change it and how:

- **Fixed** — a protocol constant. An implementation MUST use the exact value stated, because both
  parties to a trade MUST compute the same expiry or limit from it. A party that substitutes its
  own value will disagree with its counterparty about when an order cancelled or whether an object
  is valid, and the protocol has no supervisor to reconcile them (§18.1). Changing a Fixed value is
  a specification change under `GOVERNANCE.md`, not a configuration option.
- **Declared** — a value carried on the wire in a §16 object and set by the party that signs that
  object (a carrier's `dim_divisor`, an operator's `max_order_value`, an offer's lead days). This
  is a "16-class tunable": its authority is the signed object that carries it, and TRACT fixes only
  the *floor or rule* over it (§19.7), never the value itself.
- **Substrate** — defined by the DMTAP substrate and referenced here, never redefined
  (`github.com/vul-os/dmtap/blob/main/substrate/`). TRACT sets no value for it.

A timeout's *mechanism* — its source state, its destination, and who signs the transition — is
owned by §18 and is not restated here. §19 is normative for the *value* only. Where §18 and §19
both name a parameter, §18 owns the transition and §19 owns the number, and the two MUST agree.

## 19.2 Order timeouts (§18.3)

| Parameter | Value | Class | Expires into | The trade-off |
|---|---:|---|---|---|
| `PLACE_TIMEOUT` | 72 h | Fixed | `cancelled` | Long enough for a one-person seller to be away for a weekend; short enough that a buyer is not left guessing. A silent seller is never treated as having accepted. |
| `COUNTER_TIMEOUT` | 72 h | Fixed | `cancelled` | Symmetric with the above, since a counter puts the buyer in the seller's former position. |
| `FULFIL_TIMEOUT` | offer-declared floor | Declared | `cancelled`, escrow refunds | Cannot be one number: made-to-order declares lead days (§3, `Availability` variant 6, §16.5.2), a download is instant. The availability axis already carries the expected duration, so this parameter is a *floor* on how long a buyer MUST wait before they may cancel — not the duration itself. TRACT fixes the rule (a buyer MUST NOT cancel before the declared duration elapses); the seller's offer declares the duration. |
| **`CONFIRM_TIMEOUT`** | **14 days** *(PROVISIONAL)* | **Fixed floor** *(provisional)* | **`closed`, escrow releases** | **The most consequential value in this table.** Too short and a slow buyer with a real complaint loses recourse by being slow. Too long and every seller carries the float on every order. 14 days is proposed because it sits near the cooling-off windows several consumer regimes use, which makes it defensible rather than arbitrary — but §11's research gap means that alignment is *asserted, not verified* (§21.11). |

`PLACE_TIMEOUT` and `COUNTER_TIMEOUT` are Fixed: an implementation MUST use 72 hours when computing
the §18.3 transitions, so that buyer and seller agree on the instant an unanswered order cancels.
`FULFIL_TIMEOUT` is Declared — it derives from the offer's `Availability`, and only the *rule* that
it is a lower bound on the buyer's right to cancel is Fixed.

**PROVISIONAL — pending decision (§18.7).** Whether `CONFIRM_TIMEOUT` is a Fixed protocol floor or a
Declared offer-level term is unresolved; the standing lean is a Fixed floor with a Declared
extension *upward only*. Until the founder decides, an implementation SHOULD use 14 days as the
default and, if it honours an offer-declared value at all, MUST NOT let that value fall below the
floor. This decision governs whether §19 is normative for `CONFIRM_TIMEOUT` or merely suggests it,
which is why it is recorded rather than settled here. See the open-decisions list.

## 19.3 Consignment timeouts (§18.4)

| Parameter | Value | Class | Expires into | Note |
|---|---:|---|---|---|
| `PICKUP_TIMEOUT` | 48 h | Fixed | `created` | Returns to bookable rather than failing: a courier that never collected is a re-book, not a lost order. |
| `HOLD_TIMEOUT` | consolidation-declared | Declared | alert only | Consolidation waits on the slowest seller (§8.3). Raises an alert; does not cancel, because the buyer chose to wait. Declared by the consolidation arrangement, not fixed. |
| `TRANSIT_TIMEOUT` | estimate × **3** | Fixed multiplier over a Declared estimate | `lost` | The **multiplier `3` is Fixed**; the estimate is the carrier's Declared `transit_days` from the applicable `Zone` (§16.5.3). Derived rather than a flat number, since a same-city courier and a cross-border leg have nothing in common. An implementation MUST compute the deadline as three times the rate card's own published transit figure for the leg. |

`PICKUP_TIMEOUT` is Fixed. `TRANSIT_TIMEOUT` fixes only the ×3 multiplier; both parties MUST derive
the concrete deadline from the same `Zone.transit_days` the leg was booked against, so the estimate
is the carrier's claim and the multiplier is the protocol's.

## 19.4 Escrow timeouts (§18.5)

| Parameter | Value | Class | Expires into | Note |
|---|---:|---|---|---|
| `FUND_TIMEOUT` | 24 h | Fixed | order `cancelled` | An unfunded escrow holds nothing, so expiry costs nobody anything. |
| `DISPUTE_TIMEOUT` | operator-declared *(PROVISIONAL slot)* | Declared | operator ruling | **Custodial rail only.** A non-custodial rail has no ruling available, and §18.5 requires the resulting behaviour — default-to-one-party, or indefinite lock — to be **disclosed before the trade** rather than discovered at dispute time (§21.5, §21.11). |

`FUND_TIMEOUT` is Fixed. `DISPUTE_TIMEOUT` is Declared by the operator, and TRACT sets no default:
on a custodial rail (`RailClass 0`) the operator MUST publish the value it will apply, and on a
non-custodial rail (`RailClass 1`) the operator MUST disclose, before the trade, that no timed
ruling is available and what happens instead. The protocol does not pretend a neutral third option
exists (§18.5, §21.5).

**PROVISIONAL — no machine-readable slot.** `DISPUTE_TIMEOUT` has no dedicated field in
`EscrowScope` (§16.5.4); today the disclosure is prose (`EscrowScope` field 9, "authorities
claimed") or out of band. A machine-checkable field would be a §16 change (MAJOR). Recorded, not
invented. See the open-decisions list.

## 19.5 Object and serving limits

All Fixed. A decoder MUST reject an object whose encoded size exceeds its cap; the caps are
resource and — for reviews — privacy bounds, layered as validity rules over the §16 shapes (the
CDDL itself carries no `.size` bound, §16.5).

| Parameter | Value | Class | Rationale |
|---|---:|---|---|
| `MAX_PRODUCT_RECORD` | 64 KiB | Fixed | A record describes a thing; images are separate content-addressed blobs. |
| `MAX_OFFER` | 16 KiB | Fixed | An offer is four axes and a few territories. |
| `MAX_RATE_CARD` | 1 MiB | Fixed | Zone tables are genuinely large. The one object where a big number is expected. |
| `MAX_REVIEW_BODY` | 8 KiB *(PROVISIONAL)* | Fixed | Also a **privacy** bound, not only a resource one: the smaller this is, the less room there is to type personal data into a public irrevocable object (§10.4, §16.5.5). The grammar cannot stop free text; this cap and §10.4 are what bound it. |
| `MIN_DIM_DIVISOR` | 1000 | Fixed floor over a Declared value | A validity floor on a carrier's Declared `dim_divisor` (`RateCard` field 4, §16.5.3). Real carriers publish 5000 or 6000. A divisor between 1 and 1000 is not one any carrier uses, and it inflates the billable weight of every parcel computed from that card (§8.2). An implementation MUST treat a `dim_divisor` that is neither `0` nor `≥ MIN_DIM_DIVISOR` as an invalid rate card. **Zero is permitted** and means "this carrier does not apply volumetric weight". |
| `MAX_CART_LINES` | 500 | Fixed | A cart is buyer-held (§6.1), so this bounds what a *seller* must be prepared to receive, not what a buyer may collect. |

**PROVISIONAL — pending decision (§19.8).** `MAX_REVIEW_BODY` is normative at 8 KiB, but its size is
an open trade-off: its privacy purpose argues for something nearer a sentence, its usefulness argues
for the current value. It is set at 8 KiB so the field is bounded, and flagged for revision downward.
See the open-decisions list.

## 19.6 Rate and freshness

| Parameter | Value | Class | Rationale |
|---|---:|---|---|
| `RATE_CARD_MAX_AGE` | 30 days *(PROVISIONAL class)* | Fixed default | Cards drift as surcharges change. Past this, a locally computed quote is stale enough to mislead, and the buyer MUST be told rather than quietly given an old price. Staleness is measured against `RateCard.published` (field 7, §16.5.3). Whether a card MAY declare its own longer or shorter age is open (§8.8). |
| `FEED_POLL_MIN` | 60 s | Fixed | A floor on how often a client re-fetches a seller's feed. A client MUST NOT re-fetch a feed more frequently than this, so an eager implementation does not become the load problem. |
| `GRANT_LIFETIME` | 24 h | Fixed default | Analytics disclosure grants (§13) MUST expire; absent an explicit shorter lifetime, they expire 24 h after issue. A grant that never expires is consent that cannot be withdrawn by inaction. (§13 is design reasoning only — nothing in the analytics area was verified, §21.1; do not read this as evidence.) |
| `CLOCK_SKEW` | **substrate-defined** | Substrate | **Not set by TRACT.** Timestamp skew tolerance and the HLC total order are defined by the DMTAP substrate (`SYNC.md`) and referenced, never redefined here. `ts` values are for display and ordering only; where order must be authoritative it comes from a feed's sequence, never a clock (§16.7). |

An implementation MUST NOT redefine `CLOCK_SKEW` locally; it is the substrate's value
(`github.com/vul-os/dmtap/blob/main/substrate/SYNC.md`, HLC), and TRACT inherits it unchanged (C1).

**PROVISIONAL — pending decision (§8.8).** `RATE_CARD_MAX_AGE` is normative at 30 days as a Fixed
default, but whether a rate card MAY carry its own per-object max age (making it Declared) is open.
Until decided, an implementation MUST apply the 30-day default and MUST warn the buyer when a card
older than that is used to compute a quote. See the open-decisions list.

## 19.7 Wire-declared thresholds (Declared, not protocol constants)

These are values that gate trades but are **set per-object on the wire** by the party that signs the
object, not fixed by this document. They are registered here so the linter knows their absence from
the Fixed tables is intentional, and so §19 is the one place a reader can see the whole
configuration surface — Fixed and Declared alike.

| Threshold | §16 home | Set by | Rule TRACT fixes |
|---|---|---|---|
| `max_order_value` | `EscrowScope` field 7 (§16.5.4) | escrow operator | Usually a **KYC ceiling**. A party MUST NOT route a trade whose order `total` exceeds `max_order_value` through that operator, and the operator MUST refuse to fund such a trade (§9.4, §18.5). Like tax (C3), the *threshold is a fact on the wire*; the KYC determination behind it is the operator's edge policy and is **out of scope**. |
| `dim_divisor` | `RateCard` field 4 (§16.5.3) | carrier | MUST be `0` or `≥ MIN_DIM_DIVISOR` (§19.5). |
| `transit_days` | `RateCard` → `Zone` field 3 (§16.5.3) | carrier | The estimate `TRANSIT_TIMEOUT` multiplies by 3 (§19.3). |
| made-to-order lead days | `Availability` variant 6 (§16.5.2) | seller | The duration `FULFIL_TIMEOUT` floors (§19.2). |
| `storage_per_day` / `handling_fee` / `slots` | `CapacityRecord` (§16.5.3) | distributor | Consolidation terms; `HOLD_TIMEOUT` is Declared against them (§19.3). |

No Declared threshold above may be silently defaulted by an implementation: if the signing party
did not set it, it is not in force. A Fixed floor (`MIN_DIM_DIVISOR`, `FULFIL_TIMEOUT`,
`TRANSIT_TIMEOUT`'s multiplier) constrains a Declared value but never supplies one.

## 19.8 Open

- **Whether `CONFIRM_TIMEOUT` is a Fixed protocol floor or a Declared offer-level term** (§18.7).
  Recorded here because the answer decides whether this table is normative for it or merely suggests
  a default. Current lean: Fixed floor with Declared extension upward only.
- **Whether `RATE_CARD_MAX_AGE` is a single Fixed constant or a per-card Declared value** (§8.8). A
  carrier whose surcharges are stable and one whose fuel levy moves weekly want different windows,
  which argues Declared; a buyer comparing cards should not have to read a freshness term to know a
  quote is trustworthy, which argues Fixed. Set Fixed at 30 days for now.
- **Whether `DISPUTE_TIMEOUT` needs a machine-readable slot** in `EscrowScope` (§16.5.4). Today the
  operator's dispute behaviour is disclosed as prose; a dedicated field would make the §18.5
  before-the-trade disclosure machine-checkable, at the cost of a MAJOR §16 change.
- **Whether `MAX_REVIEW_BODY` should be very much smaller.** Its privacy purpose argues for
  something nearer a sentence than eight kilobytes; its usefulness argues the other way. The current
  value was chosen for the second reason and should be revisited for the first (§10.4).
- **Every value in §19.2 and §19.3 is a default with no measurement behind it.** They are plausible
  and internally consistent, and they have never been tested against how real sellers and couriers
  behave (§21.1). Recorded as such rather than presented with confidence they have not earned.
