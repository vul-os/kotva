# 19. Parameters

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 19.1 Why this is its own section

Parameters scattered through prose drift out of agreement with each other, and nobody notices until
two sections cite different values for the same thing. Collecting them lets the linter check that a
figure quoted in §N matches the table, and lets an implementer see the whole configuration surface
at once instead of discovering it a section at a time.

The values below are **proposed defaults, not measurements**. Where a value is a genuine trade-off
rather than an obvious floor, the trade-off is stated beside it — a number with no reasoning
attached is a number the next person changes arbitrarily.

## 19.2 Order timeouts (§18.3)

| Parameter | Proposed | Expires into | The trade-off |
|---|---:|---|---|
| `PLACE_TIMEOUT` | 72 h | `cancelled` | Long enough for a one-person seller to be away for a weekend; short enough that a buyer is not left guessing. A silent seller is never treated as having accepted. |
| `COUNTER_TIMEOUT` | 72 h | `cancelled` | Symmetric with the above, since a counter puts the buyer in the seller's former position. |
| `FULFIL_TIMEOUT` | offer-declared | `cancelled`, escrow refunds | Cannot be one number: made-to-order declares lead days (§3), a download is instant. The availability axis already carries the expected duration, so this parameter is a *floor* on how long a buyer must wait before they may cancel — not the duration itself. |
| **`CONFIRM_TIMEOUT`** | **14 days** | **`closed`, escrow releases** | **The most consequential value in this table.** Too short and a slow buyer with a real complaint loses recourse by being slow. Too long and every seller carries the float on every order. 14 days is proposed because it sits near the cooling-off windows several consumer regimes use, which makes it defensible rather than arbitrary — but §11's research gap means that alignment is *asserted, not verified*. |

## 19.3 Consignment timeouts (§18.4)

| Parameter | Proposed | Expires into | Note |
|---|---:|---|---|
| `PICKUP_TIMEOUT` | 48 h | `created` | Returns to bookable rather than failing: a courier that never collected is a re-book, not a lost order. |
| `HOLD_TIMEOUT` | offer-declared | alert only | Consolidation waits on the slowest seller (§8.3). Raises an alert; does not cancel, because the buyer chose to wait. |
| `TRANSIT_TIMEOUT` | carrier estimate × 3 | `lost` | Derived from the rate card's own published transit figure rather than a fixed number, since a same-city courier and a cross-border leg have nothing in common. The multiplier is the parameter; the estimate is the carrier's claim. |

## 19.4 Escrow timeouts (§18.5)

| Parameter | Proposed | Expires into | Note |
|---|---:|---|---|
| `FUND_TIMEOUT` | 24 h | order `cancelled` | An unfunded escrow holds nothing, so expiry costs nobody anything. |
| `DISPUTE_TIMEOUT` | operator-declared | operator ruling | **Custodial rails only.** A non-custodial rail has no ruling available, and §18.5 requires the resulting behaviour — default-to-one-party, or indefinite lock — to be disclosed before the trade rather than discovered at dispute time. |

## 19.5 Object and serving limits

| Parameter | Proposed | Rationale |
|---|---:|---|
| `MAX_PRODUCT_RECORD` | 64 KiB | A record describes a thing; images are separate content-addressed blobs. |
| `MAX_OFFER` | 16 KiB | An offer is four axes and a few territories. |
| `MAX_RATE_CARD` | 1 MiB | Zone tables are genuinely large. The one object where a big number is expected. |
| `MAX_REVIEW_BODY` | 8 KiB | Also a **privacy** bound, not only a resource one: the smaller this is, the less room there is to type personal data into a public irrevocable object (§10.4). |
| `MIN_DIM_DIVISOR` | 1000 | Real carriers publish 5000 or 6000. A divisor between 1 and 1000 is not one any carrier uses, and it inflates the billable weight of every parcel computed from that card (§8.2). Zero is permitted and means "this carrier does not apply volumetric weight". |
| `MAX_CART_LINES` | 500 | A cart is buyer-held (§6.1), so this bounds what a *seller* must be prepared to receive, not what a buyer may collect. |

## 19.6 Rate and freshness

| Parameter | Proposed | Rationale |
|---|---:|---|
| `RATE_CARD_MAX_AGE` | 30 days | Cards drift as surcharges change. Past this, a locally computed quote is stale enough to mislead, and the buyer is told rather than quietly given an old price. |
| `FEED_POLL_MIN` | 60 s | A floor on how often a client re-fetches a seller's feed, so an eager implementation does not become the load problem. |
| `GRANT_LIFETIME` | 24 h | Analytics disclosure grants (§13) expire by default. A grant that never expires is consent that cannot be withdrawn by inaction. |
| `CLOCK_SKEW` | ±5 min | Inherited from the substrate. Timestamps are for display and ordering; where order must be authoritative it comes from a feed sequence, never a clock (§16.7). |

## 19.7 Open

- **Whether `CONFIRM_TIMEOUT` is a protocol floor or an offer-level term** (§18.7). Listed here too,
  because the answer decides whether this table is normative for it or merely suggests a default.
- **Whether `MAX_REVIEW_BODY` should be very much smaller.** Its privacy purpose argues for
  something nearer a sentence than eight kilobytes; its usefulness argues the other way. The
  current value was chosen for the second reason and should be revisited for the first.
- **Every value in §19.2 and §19.3 is a proposal with no measurement behind it.** They are
  plausible and internally consistent, and they have never been tested against how real sellers and
  couriers behave. Recorded as such rather than presented with confidence they have not earned.
