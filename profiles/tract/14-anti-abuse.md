# 14. Anti-abuse

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 14.1 Scope

Listing spam, fake stores, review manipulation, Sybil identities, and resource exhaustion.

## 14.2 The structural position

A seller can flood only **their own** feed, which only their own followers and holders pay for and
may stop serving at will. There is no shared feed to spam and no fan-out amplification. This is
inherited from the substrate's public-object model and is why catalogue spam is a weaker threat
here than on a shared marketplace.

## 14.3 What this section will specify

- Holder admission policy: object-size ceilings, per-publisher storage quota, feed-append rate.
- Index admission policy, and the requirement that refusal to serve or index is a **policy
  decision**, never a protocol-level takedown.
- Cold-contact economics for unsolicited sealed messages (an order from a stranger is exactly the
  case the substrate's anti-abuse tiers were designed for).
- Review manipulation: what purchase attestation does and does not prevent.

## 14.4 Honest limits

- Attestation makes fake reviews expensive, not impossible.
- A new identity has no history; discounting new keys is the only defence, and it penalises
  legitimate newcomers. There is no resolution to this that does not create an authority.
