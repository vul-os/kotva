# 1. Actors & identity

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 1.1 Scope

Who the parties are, how they are identified, and how they are reached. TRACT adopts substrate
capability ① (Identity) unchanged and adds only the commerce-specific roles and disclosures.

## 1.2 What this section will specify

- The five participant roles — **seller, buyer, courier, distributor, gateway** — as roles of one
  identity type, never as separate account kinds. A single key may be all five.
- **Seller disclosure**: the fields an offer must carry to satisfy trader-traceability duties
  (legal name or trading name, contact route, establishment country, registration where held).
  This is the field set several consumer-protection regimes assume exists; §11 governs which apply.
- **Buyer disclosure tiers**: what the buyer's node attaches at browse, on grant, and at order.
- **Per-store pseudonymous subkeys**: derived so a buyer's activity is linkable *within* a seller
  (repeat-customer recognition, reviews) but not *across* sellers.
- **Reachability**: how a seller behind CGNAT is reached by key, and how an order is delivered to a
  node that is asleep or offline — the ladder, the mailbox, and wake.

## 1.3 Standards profiled

Ed25519 (RFC 8032), deterministic CBOR (RFC 8949 §4.2), COSE (RFC 9052), and the substrate's
`DeviceCert`, key-transparency and key-name constructions. No new key type is introduced.

## 1.4 Open

- Whether a seller's legal-disclosure block belongs in the offer, the feed head, or both.
- Subkey derivation: per-store deterministic derivation is convenient but makes the link
  recomputable by anyone who learns the root key. A stored random mapping avoids that at the cost
  of recovery complexity.
