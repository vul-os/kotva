# 2. Catalogue

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 2.1 Scope

Product records, offers, the product-identity ladder, variants, and the rules that keep indexes
from becoming authorities.

## 2.2 The split, and why it is mechanical rather than conventional

A **product record** describes what a thing is; an **offer** is one seller's claim to supply it.
Because the substrate content-addresses public blobs over plaintext, two sellers publishing the
same record converge on the same address by construction, and the swarm stores it once. The
global product view is therefore an emergent consequence of hashing, not a registry.

## 2.3 What this section will specify

- `ProductRecord` and `Offer` object shapes.
- **Canonicalisation**: the normalisation applied before addressing, since convergence is only
  useful to the extent independent publishers can actually produce identical bytes. This is the
  hard part of the section.
- The **identity ladder** — content address (floor, zero authority) → claimed external identifiers
  (advisory, unverified, squattable) → manufacturer-signed record (authority = the brand).
- **Variants**: product groups, varies-by axes, and per-variant records.
- **Bundles and kits**, including components published by other sellers.
- The **index rule**: derived, rebuildable, never authoritative; disagreement resolves toward the
  feed; no protocol mechanism exists by which an index can delist a seller from the network.

## 2.4 Standards profiled

schema.org product vocabulary (`Product`, `Offer`, `ProductGroup`, `hasVariant`, `variesBy`) for
the data model, so existing merchant feeds map in by translation. GS1 identifiers (GTIN, MPN) are
supported as **claims only** — issuance is gated and fee-bearing, so a spec that depended on them
would import a centralization point and a cost barrier.

## 2.5 Open

- Near-duplicate resolution. The address floor is exact-match; merging almost-identical records is
  an index-side heuristic, and heuristics differ between indexes. Whether the spec should
  recommend one, or deliberately leave it to differ, is undecided.
- Bootstrapping manufacturer signatures when most brands will not participate early.
