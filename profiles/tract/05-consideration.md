# 5. Consideration

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 5.1 Scope

The third axis: what is paid, on what schedule, and how tax attaches to it.

## 5.2 Variants

`fixed` · `tiered/volume` · `recurring` · `metered` · `deposit+balance` · `quote-required (RFQ)`

## 5.3 What this section will specify

- Money representation: **minor units and currency code, never a float**.
- Per-variant pricing objects, including recurrence rules and metering dimensions.
- **Quote/RFQ**: a signed request and a signed counter-offer, which is also the B2B
  contract-pricing mechanism — a per-buyer price is an offer addressed to one key.
- **Tax presentation**: inclusive vs exclusive display, and which anchor determines the rate.
- Currency conversion as a **presentation** concern, with the settlement currency binding.

## 5.4 Standards profiled

ISO 4217 currency codes. RFC 5545 recurrence rules for subscription periods, reusing §3's
machinery rather than inventing a second schedule format.

## 5.5 Open

- Whether tax *rates* belong in the protocol at all. Current position: no — rates change by
  jurisdiction and by week; the offer carries the treatment category and the anchors, and rate
  lookup is an implementation concern. Encoding rates would guarantee the spec ships stale law.
