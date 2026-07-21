# 15. Conformance

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 15.1 Scope

The four profiles, the auditable fail-closed set, how a profile is advertised, and the current
state of the conformance vectors.

## 15.2 The four profiles

A node need not implement everything to be conformant. The profile it claims is a floor on what
it must do — not a hint about what else it probably does — and what a profile deliberately
excludes is as much a part of its definition as what it requires.

| Profile | Builds on | Requires | Explicitly does not require |
|---|---|---|---|
| **catalogue-only** | — | publish and/or serve `ProductRecord`, `Offer`, `RateCard` and `CapacityRecord` objects (§2, §8.2) over the public quadrant; honour feed-head ordering, so an older signed head is never accepted over a newer one already observed (§2); serve derived index queries without asserting authority over a seller's own feed (§2.6) | accepting a sealed `Order` (§7); computing a delivery total (§8); holding or brokering funds (§9); rendering a storefront (§12) |
| **transacting** | catalogue-only | send and receive sealed `Order` objects and drive every transition of §18.3's state machine, including its timeouts; re-evaluate a cart line's live availability before checkout (§6.2); apply the substrate's cold-contact tiers to an unsolicited order (§14.3) | computing a route or a shipping total locally — a transacting node may present an order without one, or delegate the computation to a routing-profile peer |
| **routing** | transacting | compute a leg's price and transit estimate from a published `RateCard` object locally, without a live call to the carrier (§8.2); evaluate the small consolidation candidate set (§8.3) | being a carrier or a distributor itself — those are roles (§0.4.1), independent of which profile a node advertises |
| **gateway** | — (may stand alone, or combine with any of the above) | storefront rendering (§12) or settlement and escrow (§9.5), or both — the two bundle only because they share the same commercial and legal standing (§0.4.2), not because one implies the other; whichever it does, the matching fail-closed obligations of §15.3 apply | the half it does not do — a storefront-only gateway need not settle, and an escrow-only gateway needs no storefront, because two TRACT-native parties never need a gateway to transact at all (§0.4.2) |

None of the four is "complete TRACT," and none is meant to be. This mirrors §0.4.1's roles: a
seller needs a keypair and a box that is up, not a delivery-routing engine or a
payment-provider relationship, and a profile boundary exists precisely so a catalogue-only node
never has to build — or fake — the parts of the protocol it has no reason to run. Forcing every
implementation to speak every profile would recreate the single do-everything platform the rest
of this document exists to avoid.

The "builds on" column is a dependency, not a menu restriction. A node cannot claim routing
without transacting — computing a route only matters once an order exists to attach it to — but
a node can claim catalogue-only and gateway together (a storefront in front of its own
catalogue, settling nothing) without ever claiming transacting.

## 15.3 The fail-closed set

Every security-relevant failure in this document is refused outright, or surfaced to both
parties as an explicit choice; nothing degrades silently into an outcome that merely looks like
success. Four of these are singled out because their violation is evidence of a defect in the
implementation, not a business outcome an operator chose: escrow-scope intersection failure
(§9.4), rail-class substitution without both parties agreeing (§9.3), origin isolation (§12.3),
and the public-quadrant personal-data prohibition (§0.5.1).

The table indexes every failure of this kind that §17.4 has already assigned a code to. The
owning clause is authoritative; this table is a pointer to it, not a restatement — a change to
what a rule actually requires happens at the clause cited, never here.

| Violation | Owning clause | Code (§17.4) |
|---|---|---|
| Personal data present in a public-quadrant object | §0.5.1 | `0x0102` |
| Sealed and public object type confusion at decode | §16.4 | `0x0101` |
| Offer missing a required axis | §2.4 | `0x0201` |
| Two `money` values of different currencies combined without a disclosed conversion | §5.3 | `0x0401` |
| A sale that would exceed a seller's combined quota across partitioned replicas | §6.2, §6.4 | `0x0501` |
| A route total exceeding the representable range of a `money` minor-units integer | §8.2 | `0x0701` |
| A rate card used to compute a quote past `RATE_CARD_MAX_AGE` without disclosing its age | §8.2, §19.6 | `0x0702` |
| `EscrowScope` intersection between buyer and gateway is empty | §9.4 | `0x0801` |
| Rail class substituted without a fresh party agreement recorded on the order | §9.3 | `0x0802` |
| `Review` lacking a valid purchase attestation | §10.2 | `0x0901` |
| Place of supply unresolved at the point tax treatment is computed | §11.2 | `0x0A01` |
| Two stores served from one gateway origin | §12.3 | `0x0B01` |

## 15.4 Advertising and negotiating a profile

Profile advertisement rides the substrate's capability-token mechanism (§0.3, capability ④)
rather than a bespoke TRACT-side negotiation. A node's advertised profile set is a signed,
versioned announcement, and a stale, lower announcement is rejected on the same monotonic
version basis the substrate already defines for every capability token — a node cannot be
tricked into believing a peer has silently dropped back to a narrower profile than it last
advertised.

The rule that follows is inherited unchanged from the substrate rather than restated in TRACT's
own words: a capability absent from a peer's current announcement is a **fact about that peer**,
never a fault to be worked around, retried past, or read as a temporary condition. A buyer's
node that finds a seller advertising catalogue-only does not treat the missing transacting
profile as an error to route around — there is nothing to route around. It is the honest current
shape of that seller: browsable, not yet buyable through this protocol.

The consequence for conformance is symmetric with §15.2's "does not require" column: a
conformance harness tests a node against the profiles it claims, never against the profiles it
does not. `conformance/SUITE.md`'s TRACT-PROFILE-01 and TRACT-PROFILE-02 cases turn exactly this
into assertions — a catalogue-only node must refuse an inbound sealed order rather than silently
accept and ignore it, and a node advertising transacting without routing must not compute
delivery routing on the buyer's behalf while implying that it does.

## 15.5 Vectors: the honest status

There are zero conformance vectors in this repository. `conformance/SUITE.md` lists 39 planned
cases across the four profiles above, organised by the section each pins; every one is marked
PLANNED, and none is backed by a byte-exact vector, because §16 (wire format) is not yet
normative — there are no committed object shapes for a vector to check against. Freezing one
early would invent bytes no numbered section actually specifies, or get silently invalidated the
moment §16 lands with a different shape and nobody notices, because a vector nothing runs cannot
fail a build (`conformance/README.md`).

`make coverage` reports this honestly rather than projecting a percentage:

```
conformance: 39 case(s) planned, 0 runnable (no vectors until §16 is normative — see conformance/README.md)
```

That number moves only when a case is added to or removed from `conformance/SUITE.md`. It does
not become "0 runnable" less true until a vector is derived from normative §16 text by the
discipline `conformance/README.md` states — never exported from a reference implementation and
back-filled here.

## 15.6 Open

- Whether a fifth, distinct profile for index-building behaviour is worth advertising, given
  §0.4.1 already treats building one as available to any node without registration. The four
  profiles above describe transactional capability, not derived-data capability, and
  TRACT-CAT-02 and TRACT-TRUST-02 already assume index behaviour that no profile currently names.
- Whether profile composition (transacting implying catalogue-only, routing implying transacting)
  needs to be enforced by the capability-token grammar itself, or stays a conformance-suite check
  layered on top of an announcement mechanism that does not itself understand dependencies.
- Whether a signed capability announcement is sufficient evidence of conformance, or whether the
  suite needs a live-probe component. An announcement authenticates to a key, not to the
  behaviour behind it, and TRACT-PROFILE-02 is written against the latter.
- What "conformant" means for a gateway that advertises both storefront and settlement but only
  actually delivers one. §15.2 permits claiming either half alone; the suite does not yet have a
  case for a gateway that claims both and only honours one.
