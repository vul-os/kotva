# 3. Availability

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 3.1 Scope

The first of the four offer axes: when, and how much, is on offer.

## 3.2 Variants

`count` · `time-slots` · `capacity-per-interval` · `unlimited` · `made-to-order`

## 3.3 What this section will specify

- The availability object per variant, and how a buyer's node evaluates it locally.
- **Slot availability** as a profile of iCalendar, so a seller can publish from calendar software
  they already run rather than maintaining a second source of truth.
- **Lead time** for made-to-order, distinguished from stock-out.
- How availability signals are published without leaking commercially sensitive exact stock — a
  seller may publish a band ("in stock", "low") rather than a number.

## 3.4 Standards profiled

RFC 5545 `VAVAILABILITY` and `VFREEBUSY` for time-slot and capacity availability. Time zones per
RFC 5545 / IANA tzdata.

## 3.5 Open

- Whether published availability bands need a normative vocabulary or stay seller-defined.
