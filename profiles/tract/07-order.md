# 7. Order

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 7.1 Scope

The sealed order object, its lifecycle, and the evidence each transition produces.

## 7.2 What this section will specify

- `Order` as a **sealed message**, one per seller, carrying only that seller's lines. It is never
  published; it lives at the two endpoints and is deletable there.
- The state machine: draft → placed → accepted / declined / countered → fulfilling → delivered →
  closed, with cancellation and return paths, and the timeout at each edge.
- **Signed transitions**: which party signs what, so that acceptance, dispatch, custody handoff and
  receipt are each provable after the fact.
- Order amendment, partial fulfilment, and partial refund.
- **Returns and exchanges**, including who pays return carriage under which fulfilment variant.

## 7.3 Standards profiled

The substrate's sealed message object for transport. Where an order must be exported to a legacy
counterparty (an ERP, a 3PL), the mapping targets are UN/CEFACT and EDIFACT order/despatch/invoice
messages; that mapping is informative, not required.

## 7.4 Open

- Whether order amendment is a new sealed object superseding the prior, or an operation log. The
  supersede model matches the rest of the design; the log model is easier to audit.
