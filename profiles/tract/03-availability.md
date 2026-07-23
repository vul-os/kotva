# 3. Availability

> **Drafting status.** Normative. The five availability variants, the band vocabulary, the rule for
> evaluating one locally, and the signal-not-reservation boundary are settled and stated below with
> RFC 2119 keywords aligned exactly to the `Availability` and `StockSignal` grammar frozen in
> §16.5.2. Four decisions remain open; each is marked **PROVISIONAL — pending decision** at the
> point it bites (collected in §3.11): an `Availability`-specific freshness field, whether lead days
> count calendar or working days, whether the four-band set admits a richer seller-defined
> vocabulary, and confirmation that a capacity number is a published total rather than a live
> remainder. The key words MUST, MUST NOT, REQUIRED, SHALL, SHOULD, SHOULD NOT and MAY are to be
> interpreted as in BCP 14 (RFC 2119, RFC 8174).

## 3.1 Scope

This section owns the second of the four offer axes (§2.3): **when a thing is available, and how
much of it there is.** §16.5.2 is the byte authority — it freezes the `Availability` and
`StockSignal` shapes — and this section supplies the reasoning behind each variant and the
normative rule for evaluating one. It defines no new bytes; every field named here is the field
§16.5.2 already carries.

An `Availability` value is a **signal a seller publishes**, not a reservation. It commits no stock.
The only place stock is actually held is §6's bounded-counter inventory (§6.2, §6.2a); §3.9 states
that boundary and MUST be read alongside this section, because reading `Availability` as a hold is a
category error the rest of this section is written to prevent.

## 3.2 Variants

An `Offer` (§16.5.2, key 2) MUST carry exactly one `Availability` value, and that value MUST be
exactly one of the five variants below. The variants are the five arms of the `Availability` CDDL
union; a decoder is always in exactly one arm, distinguished by the integer key.

| Variant | §16.5.2 shape | Carries | Covers |
|---|---|---|---|
| `count` | `{ 0 => StockSignal }` | a `StockSignal` band | physical or digital stock, exact or banded |
| `time-slots` | `{ 1 => tstr, 2 => uint }` | an RFC 5545 payload, slot length in minutes | bookable appointments |
| `capacity-per-interval` | `{ 3 => uint, 4 => tstr }` | a capacity number, an RFC 5545 recurrence | seats, tables, room-nights |
| `unlimited` | `{ 5 => null }` | nothing | digital goods with no bound |
| `made-to-order` | `{ 6 => uint }` | lead days | built or provisioned after the order, not held in stock |

Five variants, one grammar. A haircut and a restaurant table are both bookable, but a haircut
consumes an appointment slot whole and a table seats a party against a nightly cap — `time-slots`
and `capacity-per-interval` are distinct variants because those are different questions, not the
same question asked twice (§3.5).

A decoder that encounters an `Availability` key it does not recognise MUST NOT present the item as
buyable; whether an unrecognised variant is tolerated on store and rejected only on act, or rejected
on receipt, is the axis-union extension-key policy still open at §16.8(d) — this section defers to
that decision and MUST NOT resolve it independently.

## 3.3 The stock signal is a band, not a number

```
StockSignal = { 0 => uint }   ; exact(n)
            / { 1 => null }   ; in-stock
            / { 2 => null }   ; low
            / { 3 => null }   ; out-of-stock
```

These four arms are the **complete** stock vocabulary the wire admits; no other band value is
expressible, and an implementation MUST NOT invent one. A seller MAY publish `exact(n)`, but the
`count` variant MUST NOT require it, for two reasons that both matter:

- **Exact stock is commercially sensitive.** A competitor watching a public feed can infer
  sell-through rate, restock cadence, and even unit economics from a bare number changing over time.
  Browsing a catalogue does not need that number; it needs to know whether the thing can be bought.
- **A band degrades honestly; an exact number degrades into a lie.** Stock moves between publishes.
  `in-stock` stays true across a wide range of underlying counts, so it survives the gap between one
  publish and the next. `exact(4)` is precisely correct at the moment it is signed and increasingly
  wrong afterwards — and it is wrong with the same apparent confidence throughout. A band's
  imprecision is disclosed by its shape; a stale exact count's imprecision is not disclosed at all.

`out-of-stock` and `low` are distinct from each other for the same reason `made-to-order` is
distinct from `out-of-stock` (§3.6): a buyer deciding whether to add something to a cart needs to
know whether waiting a moment might work, not just whether it works right now.

The protocol assigns **no numeric threshold** to `low` or `in-stock`: the count at which a seller
publishes `low` rather than `in-stock` is seller policy and the protocol MUST NOT prescribe it. A
buyer's node MUST treat the band as a qualitative signal and MUST NOT infer an exact count from
`low`, `in-stock`, or `out-of-stock`.

> **PROVISIONAL — pending decision.** Whether the four-arm set is the final stock vocabulary or
> whether a richer seller-defined band namespace is wanted is recorded in §3.11 and the
> founder-decision list. Adding a band value is a §16 grammar change (MAJOR bump), so nothing here
> may add one silently.

## 3.4 Time slots — profiling RFC 5545

The `time-slots` variant carries an **opaque RFC 5545 payload** (`tstr` at key 1 — a `VAVAILABILITY`
/ `VFREEBUSY` document) plus a slot length in minutes (`uint` at key 2). It MUST NOT carry a
bespoke schedule grammar. The reasoning is the one §5.6 makes for recurring consideration: a seller
who already runs calendar software SHOULD publish from it directly, rather than maintain a second,
TRACT-specific source of truth that drifts out of sync with the first.

The buyer's node MUST parse the payload locally against the slot length to enumerate candidate slots;
there is no slot-listing endpoint, and nothing in this variant contacts the seller. Time zones and
recurrence within the payload are interpreted per RFC 5545 and IANA tzdata (§3.10).

What the payload cannot express is which slots other buyers have since taken. That is a staleness
property, not a grammar gap: §3.8 states why, and §3.9 states that the authoritative resolution of a
taken slot is the seller's `placed → accepted / declined` step (§18.3), never the published payload.

## 3.5 Capacity per interval

The `capacity-per-interval` variant carries a unit count (`uint` at key 3) and an RFC 5545 recurrence
(`tstr` at key 4) describing the intervals the count applies to — 40 covers per sitting, three
sittings a night. This MUST NOT be modelled as `time-slots` with a number attached: a time slot is
claimed whole by one booking, whereas a capacity interval is shared by many bookings up to its cap.
Modelling a restaurant table as a time slot would force one party to occupy the whole sitting;
modelling a haircut as capacity-per-interval would allow two people to book the same ten minutes. A
seller MUST choose the variant that matches the question its supply actually answers.

The capacity number is the **published cap per interval**. It is a signal, consistent with §3.9, and
MUST NOT be read as a live remaining count — the published object carries no per-interval booking
state, and a buyer's node has no way to observe how much of a cap is already taken (§3.7). Reading it
as a remainder would credit the signal with reservation semantics only §6's counter provides.

> **PROVISIONAL — pending decision.** The draft raised whether this number is total or remaining
> capacity. §3.9's signal-not-reservation rule forces the total/published-cap reading — a
> per-interval *remainder* would require live booking state the public object does not and cannot
> carry — so the normative text above resolves it in that direction. §3.11 records this as a
> confirmation item for the founder, not an invented call.

## 3.6 Made-to-order is not out-of-stock

The `made-to-order` variant carries a lead-time figure (`uint` at key 6, in days) and nothing else —
no stock number, no slot, no capacity — because none of those describe it. The thing is available. It
simply is not sitting in a warehouse yet, and that is a different fact from having none. An
implementation MUST NOT map `made-to-order` onto `in-stock` or `out-of-stock`.

Conflating the two is why lead-time products display badly on retail platforms built around a stock
count: a made-to-measure suit or a build-to-order machine gets forced into `in stock` (wrong — it
does not exist yet) or `out of stock` (wrong — it is buyable right now, just not instantly). Neither
label is honest, and a buyer who wanted to know "how long" was never asked the right question.
`made-to-order` exists so that question has a field.

§19's `FULFIL_TIMEOUT` reads this figure directly: it cannot be one protocol-wide number because a
made-to-order lead time and an instant digital grant have nothing in common, so the availability
axis supplies the floor a buyer may expect to wait before cancelling becomes reasonable, rather than
the protocol asserting a duration it cannot know. A buyer's node MUST use this figure, not a
protocol constant, when computing that floor (§3.7, §19).

> **PROVISIONAL — pending decision.** Whether the lead-time figure is counted in calendar days or
> working days is not settled by the grammar: §16.5.2 carries a bare `uint` and does not encode the
> distinction, and the reference implementation documents working days. Until this is decided, an
> implementation MUST NOT silently assume one convention; §3.11 records the decision.

## 3.7 Evaluating availability locally

The buyer's node does this work; nothing is queried live. For a given `Availability` value an
implementation MUST evaluate it as follows.

| Variant | Local evaluation |
|---|---|
| `count` | Compare the requested quantity against the band. `exact(n)` supports an exact check; `in-stock` / `low` support only "probably yes, confirm on placement"; `out-of-stock` MUST block the add. |
| `time-slots` | Expand the RFC 5545 payload against the slot length into candidate slots and present them; MUST NOT present a slot as reserved. |
| `capacity-per-interval` | Expand the recurrence into intervals and show the published cap per interval; MUST NOT display it as remaining capacity (§3.5). |
| `unlimited` | Nothing to evaluate; the item is always addable. |
| `made-to-order` | Add the lead days to the order date to show an expected dispatch date, and feed that as the `FULFIL_TIMEOUT` floor (§3.6, §19). |

None of this evaluation contacts the seller — that is the point of publishing the object at all. It
also means every row above is read from whatever copy of the offer the buyer's node last fetched,
which is the subject of §3.8. An implementation MUST NOT represent a local evaluation as a live or
confirmed availability; the only authoritative confirmation is the seller's placement step (§3.9,
§18.3).

## 3.8 What happens when published availability is stale

An `Offer` carries one freshness signal: its own `published` timestamp (§16.5.2, key 6).
`Availability` has no timestamp of its own — a stale `Offer` and a stale `Availability` are the same
event, because they are the same object. There is no push notification for an availability change
specifically: substrate wake (`ROLES.md §8`,
`github.com/vul-os/dmtap/blob/main/substrate/ROLES.md`) wakes a **seller's** node for incoming
orders, not a **buyer's** node for outgoing price or stock changes. Freshness is therefore bounded
only by how often the buyer's client re-fetches the seller's feed — no more often than
`FEED_POLL_MIN` (§19) but with no upper bound the protocol enforces.

The consequence: a buyer MAY add something to a cart that sold out between the last fetch and now.
This MUST NOT be treated as a protocol defect to be engineered away — §3.3's whole argument is that a
band already discloses this uncertainty rather than hiding it. What resolves the gap is not a tighter
poll interval; it is that the seller's `placed → accepted / declined` step (§18.3) is authoritative
and the browsing-time signal is not. An offer that turned out to be wrong MUST be resolvable by a
decline there, explicitly, rather than silently honoured against stock that no longer exists.

> **PROVISIONAL — pending decision.** Whether `Availability` should carry its own freshness
> parameter — distinct from the offer's `published` and from `RateCard`'s `RATE_CARD_MAX_AGE`
> (§19), since stock and slots plausibly move much faster than price — is unresolved and would
> require a §16 grammar change (a freshness field on `Availability`). It is recorded in §3.11, the
> founder-decision list, and the §16 open-items list; this section MUST NOT add such a field.

## 3.9 A signal, not a reservation

Nothing in this section commits stock. `Availability` is what a seller publishes and what a buyer's
node reads to decide whether adding something to a cart is worth attempting — it is input to a
decision, not a hold on the outcome. An implementation MUST NOT treat any `Availability` value as a
reservation, a hold, or a guarantee of supply.

§6 is where a hold actually exists: a seller's bounded-counter inventory (§6.2, §6.2a) is what
guarantees that concurrent sales never exceed real stock, and it operates beneath what gets published
here. A buyer's node never sees counter state directly, only the band this section defines. Reading
`Availability` as a reservation would be a category error in both directions — it would credit the
band with a guarantee only the counter provides, and it would blame the band for a staleness gap
(§3.8) that the counter, not the signal, is responsible for closing.

**Honest limit (§21.10.1, §21.10.2).** The no-coordinator oversell guarantee that backs §6 is real
and measured — the escrow/demarcation bounded counter (AntidoteDB `antidote_crdt_counter_b`, SRDS
2015) holds the no-oversell invariant even from stale state, without any coordinator — but that
guarantee is **safety-only, paid entirely in liveness** (stranded quota, spurious "sold out"), it
lives in §6 and not in this section, and its failure model is explicitly **non-Byzantine**: it
protects a seller from their own concurrency, not a buyer from a dishonest seller (§21.10.2). The
`Availability` signal is weaker still — it is not even a hold — so this section makes no oversell or
supply guarantee of any kind, and MUST NOT be cited as one. §21 is honest evidence, not support:
this section leans on §21.10.1 only to disclaim, never to claim.

## 3.10 Standards profiled

- **RFC 5545** `VAVAILABILITY` / `VFREEBUSY` for the `time-slots` payload (§3.4).
- **RFC 5545** recurrence rules for `capacity-per-interval`'s interval definition (§3.5).
- **Time zones** per RFC 5545 / IANA tzdata.

These are profiled via existing standards per the brief's C4 agnosticism rule; TRACT defines no
schedule vocabulary of its own. The substrate governs the bytes that carry these payloads on a feed
(`github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md`); this section never re-specifies them.

## 3.11 Open

Four decisions remain open. Each is written as **PROVISIONAL — pending decision** at its point of use
above and is collected here for the founder-decision list. None is invented in this section.

- **Stock band vocabulary (§3.3).** Whether the four-arm `StockSignal` set is final or whether a
  richer seller-defined band namespace is wanted. Recommendation: keep the four frozen arms; band
  thresholds are already seller policy, and adding arms is a §16 MAJOR change for marginal gain.
- **`Availability` freshness (§3.8).** Whether `Availability` needs its own freshness parameter,
  distinct from the offer's `published` and from `RateCard`'s `RATE_CARD_MAX_AGE` (§19), because
  stock and slots plausibly move much faster than price. Requires a §16 grammar change. Recommendation:
  add a per-variant optional max-age hint so a stale stock band can be flagged separately from a
  day-old price, rather than overloading the offer's single `published`.
- **Made-to-order day convention (§3.6).** Whether `made-to-order` lead days are calendar days or
  working days; the grammar carries a bare `uint`. Recommendation: fix the wire meaning as **calendar
  days** (unambiguous across jurisdictions and holidays) and leave working-day presentation to the
  seller's display layer.
- **Capacity number reading (§3.5).** Confirmation that the `capacity-per-interval` number is the
  published total cap, not a live remainder. §3.9 already forces the total reading; this is a
  confirmation, not an open mechanism. Recommendation: confirm total/published-cap as normatively
  stated in §3.5.
