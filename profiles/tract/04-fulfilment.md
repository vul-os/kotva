# 4. Fulfilment

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 4.1 Scope

The second axis: how the thing reaches the buyer — and, consequentially, **where the supply
happens** for tax purposes.

## 4.2 Variants

`ship` · `collect` · `digital-grant` · `perform-at-place` · `perform-remote` · `access-grant` ·
`return-required`

## 4.3 What this section will specify

- The fulfilment object per variant, including the place a service is performed and the return
  terms for a rental.
- The **place-of-supply derivation table** — the mapping from fulfilment variant to jurisdictional
  anchor (§11.2). Ship → destination; perform-at-place → the venue; perform-remote and
  digital-grant → buyer residence; return-required → collection point.
- **Handover**: what constitutes delivery for each variant, and what evidence is signed by whom.
- Multi-variant offers (collect *or* ship) and how the buyer's choice binds the anchors.

## 4.4 Standards profiled

Incoterms 2020 for the risk/cost transfer point on shipped goods. ISO 3166 for places.

## 4.5 Open

- Whether digital-grant needs a licence-terms sub-object or should defer entirely to §5.
