# 15. Conformance

> **Status: normative, with open items scoped.** §15.2 (the four profiles and their floors),
> §15.3 (the fail-closed obligation), §15.4 (profile advertisement), and §15.5.1's vector-scope
> rule are normative. The key words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are to be
> interpreted as in BCP 14 (RFC 2119, RFC 8174). The freeze of §16 at v0 (`16-wire-format.md`,
> normative) lifted the blocker this section previously recorded — there are now committed object
> shapes for a vector to check against — and a first slice of commerce-spine vectors already
> exists on disk (§15.5.3). What remains genuinely undecided — a possible fifth index-building
> profile, whether profile composition is enforced in the capability-token grammar or only by the
> suite, whether a signed announcement suffices without a live probe, and how a both-halves
> gateway that honours one half is judged — stays in §15.6 and is **not** normative.

## 15.1 Scope

The four profiles, the auditable fail-closed set, how a profile is advertised, and the scope and
derivation discipline of the conformance vectors — everything a second implementation needs to
prove it matches this document rather than matching a reference one.

## 15.2 The four profiles

A node need not implement everything to be conformant. The profile it claims is a floor on what
it must do — not a hint about what else it probably does — and what a profile deliberately
excludes is as much a part of its definition as what it requires.

A node **MUST NOT** advertise (§15.4) a profile whose *Requires* floor below it does not
implement, and a conformance harness tests a node **only** against the profiles it advertises,
never against those it does not. The *builds on* column is a hard dependency, not a menu
convenience: a node **MUST NOT** advertise `routing` without also satisfying `transacting`, and
**MUST NOT** advertise `transacting` without also satisfying `catalogue-only`. `gateway` stands
alone and **MAY** be advertised together with any of the other three.

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

The "builds on" dependency is a dependency, not a menu restriction. A node cannot claim routing
without transacting — computing a route only matters once an order exists to attach it to — but
a node can claim catalogue-only and gateway together (a storefront in front of its own
catalogue, settling nothing) without ever claiming transacting.

## 15.3 The fail-closed set

Every security-relevant failure in this document **MUST** be refused outright, or surfaced to
both parties as an explicit choice; nothing **MUST** degrade silently into an outcome that
merely looks like success. Four of these are singled out because their violation is evidence of a
defect in the implementation, not a business outcome an operator chose: escrow-scope
intersection failure (§9.4), rail-class substitution without both parties agreeing (§9.3), origin
isolation (§12.3), and the public-quadrant personal-data prohibition (§0.5.1).

The table indexes every failure of this kind that §17.4 has assigned a code to. **The owning
clause is authoritative** for what the rule requires; this table is a pointer to it, not a
restatement — a change to what a rule actually requires happens at the clause cited, never here.
§17.4 is the registry that names the code. Where §17.4 or an owning section is not yet itself
normative, the code *value* is provisional and may be renumbered as that section firms up, but
the fail-closed *obligation* is not provisional: it is inherent in each cited clause, and a
conformant implementation **MUST NOT** proceed past any of these conditions into a plausible-looking
reduced outcome.

| Violation | Owning clause | Code (§17.4) |
|---|---|---|
| Personal data present in a public-quadrant object | §0.5.1 | `0x0102` |
| Sealed and public object type confusion at decode | §16.4 | `0x0101` |
| Offer missing a required axis | §2.3 | `0x0201` |
| Two `money` values of different currencies combined without a disclosed conversion | §5.3 | `0x0401` |
| A sale that would exceed a seller's combined quota across partitioned replicas | §6.2, §6.4 | `0x0501` |
| A route total exceeding the representable range of a `money` minor-units integer | §8.2 | `0x0701` |
| A rate card used to compute a quote past `RATE_CARD_MAX_AGE` without disclosing its age | §8.2, §19.6 | `0x0702` |
| `EscrowScope` intersection between buyer and gateway is empty | §9.4 | `0x0801` |
| Rail class substituted without a fresh party agreement recorded on the order | §9.3 | `0x0802` |
| `Review` lacking a valid purchase attestation | §10.2 | `0x0901` |
| Place of supply unresolved at the point tax treatment is computed | §11.2 | `0x0A01` |
| A regime-required in-region responsible person absent from an order policy says must name one | §11.3.4 | `0x0A02` |
| Two stores served from one gateway origin | §12.3 | `0x0B01` |

## 15.4 Advertising and negotiating a profile

Profile advertisement **MUST** ride the substrate's capability-token mechanism (§0.3, capability
④) rather than a bespoke TRACT-side negotiation. A node's advertised profile set is a signed,
versioned announcement, and a stale, lower announcement **MUST** be rejected on the same
monotonic-version basis the substrate already defines for every capability token
(`github.com/vul-os/dmtap/blob/main/substrate/ROLES.md`, referenced not re-specified) — a node
cannot be tricked into believing a peer has silently dropped back to a narrower profile than it
last advertised. TRACT introduces no new anti-rollback construction for this; it inherits the
substrate's.

The rule that follows is inherited unchanged from the substrate rather than restated in TRACT's
own words: a capability absent from a peer's current announcement is a **fact about that peer**,
never a fault to be worked around, retried past, or read as a temporary condition. A buyer's
node that finds a seller advertising catalogue-only **MUST NOT** treat the missing transacting
profile as an error to route around — there is nothing to route around. It is the honest current
shape of that seller: browsable, not yet buyable through this protocol.

The consequence for conformance is symmetric with §15.2's "does not require" column: a
conformance harness tests a node against the profiles it claims, never against the profiles it
does not. `conformance/SUITE.md`'s TRACT-PROFILE-01 and TRACT-PROFILE-02 cases turn exactly this
into assertions — a catalogue-only node **MUST** refuse an inbound sealed order rather than
silently accept and ignore it, and a node advertising transacting without routing **MUST NOT**
compute delivery routing on the buyer's behalf while implying that it does.

## 15.5 Conformance vectors: scope and derivation

Two things changed the moment §16 froze at v0 (`16-wire-format.md`, normative): there are now
committed object shapes for a vector to check against, and the blocker this section previously
recorded — "no vectors until §16 is normative" — is lifted. What survives that change, and is
itself normative, is a **scope rule** (§15.5.1) and a **derivation rule** (§15.5.2). The honest
count of what exists today follows in §15.5.3.

### 15.5.1 Scope — TRACT vectors fix the commerce spine, never the substrate

TRACT conformance vectors **MUST** cover only the commerce spine this document adds on top of the
DMTAP substrate. A TRACT vector **MUST NOT** restate, duplicate, or re-derive substrate
byte-behaviour — the deterministic-CBOR canonicalisation TRACT inherits, the COSE-style signature
framing, the hash and content-address construction, or the HLC total order — because §16.2 adopts
all four from the substrate unchanged and introduces none of its own. An implementation
demonstrates conformance to that byte-behaviour against **the substrate's own conformance
vectors** (`github.com/vul-os/dmtap` — `IDENTITY.md` for keys and signatures, `FEEDS.md` for
content addressing and feed-head ordering, `SYNC.md` for the HLC), not against a TRACT copy of
them. A second copy maintained here could only ever drift out of agreement with the first, and
would make TRACT the accidental authority over bytes it explicitly does not own — the same
closed-loop failure `conformance/README.md` exists to prevent.

This rule is not an aspiration the vectors will grow into; it is already the reason a large
fraction of the case catalogue is **not yet computable** (§15.5.3). A case whose invariant turns
on a real content address (`TRACT-CAT-01`'s address-convergence, `TRACT-PUBSEAL-02`'s
domain-separation tags), a real signature (`TRACT-ORDER-01`), or a feed-sequence rollback
(`TRACT-ABUSE-01`) cannot be vectored inside TRACT without inventing a substrate primitive TRACT
is forbidden from inventing. Those cases are covered where they belong — against the substrate —
and TRACT's vectors stop precisely at the boundary where the substrate's byte-behaviour begins.

Concretely, four families of TRACT vector, and no substrate-level one, are in scope:

- **Catalogue canonicalisation (§2.3).** Two `ProductRecord`s whose canonicalised fields encode
  to identical bytes converge, and two that differ before canonicalisation do not silently
  collide. The vector fixes the *canonicalisation* TRACT specifies and the *precondition* the
  address-convergence claim rests on (canonical CBOR is independent of a publisher's field-order);
  it stops before the address itself, which the substrate's hash construction fixes
  (`TRACT-CAT-01`, marked partial for exactly this reason; `TRACT-CAT-03`'s offer-axis
  completeness is fully in scope, being a structural property of the frozen `Offer` shape).
- **Order lifecycle (§18.3).** Which signed transition is legal from which state, and where each
  timeout edge expires *into* (`TRACT-ORDER-02`, `TRACT-SM-01`). The vector fixes the transition
  graph and the timeout destinations, not the signature bytes beneath a transition — those are
  the substrate's.
- **Pricing selection (§5, §16.5.2 `Consideration`/`PriceTier`; §8.2 `RateCard`).** Which tier a
  quantity selects; the local `billable = max(actual, L*W*H / dim_divisor)` and bracket-price
  lookup a `RateCard` yields with no live call (`TRACT-DELIV-01`); the refusal of cross-currency
  route arithmetic (`TRACT-DELIV-03`, §5.3) and of a route total that overflows a `money`
  minor-units integer (§8.2). These fix the arithmetic TRACT specifies *over* the `money` type,
  never the `money` encoding itself (§16.7).
- **Place-of-supply anchoring (§4 `Fulfilment` → §11.2).** The anchor a `Fulfilment` variant
  derives, independent of buyer and seller country — the forcing case being an event held in
  country C, sold by a seller in A, to a buyer in B, deriving place of supply as C from the
  `Fulfilment` object alone (`TRACT-FULF-01`/`-02`, full) — and the four independent anchor fields
  carried on the sealed order. The `Order` grammar now carries these directly: the post-freeze
  §16.6 correction that added the `Anchors` map (`seller_establishment`, `buyer_residence`,
  `place_of_supply`, optional `delivery_destination`) closes the gap the first vector pass
  recorded against `TRACT-JURIS-02`, which had found `Order` carrying no anchor fields at all.

### 15.5.2 Derivation — read the specification, never dump an implementation

A vector corpus that an implementation passes proves the corpus and the implementation agree with
**each other**; it does not prove either agrees with **this document**. That distinction has
already failed once on the sibling DMTAP specification, where spec text, a frozen vector and three
tests were self-consistently wrong together because the vector had been exported from the same
reference implementation the tests ran against (`conformance/README.md`).

The rule TRACT inherits, and which is normative for the first vector as much as the hundredth: a
vector **MUST** be derived by a person reading the §-numbered specification text of the object it
covers, from the formulas that section states, and **MUST NOT** be exported from, or back-filled
to match, a running implementation (Soko or any other). A generator that computes bytes from the
spec's own formulas, annotated with the exact sentence each vector encodes, is sound; a dump of an
implementation's internal state is not, because it can only ever prove that implementation
self-consistent. Where the cited text is silent on a detail a vector needs — a rounding rule, a
tie-break, the numeric meaning of "intersect" for a non-set field — the vector **MUST** record
that silence in its own `description`/`note` and mark itself `partial`, rather than pick an answer
and present it as derived (as `TRACT-CAT-01` and `TRACT-SETTLE-01` do today). A divergence between
a committed vector and an independent implementation is resolved with the specification text as
the tiebreaker, never by editing the vector to match the implementation.

### 15.5.3 The honest count

`conformance/SUITE.md` catalogues 39 cases across the four profiles, every one marked PLANNED.
`conformance/vectors/` now holds a first slice derived under §15.5.2's discipline: **21 vector
files supporting 8 of those 39 case ids — 6 fully, 2 partially** (`TRACT-CAT-01` and
`TRACT-SETTLE-01`, partial for the silences §15.5.2 requires them to disclose). The remaining 31
ids each carry a stated reason they are not yet computable, and those reasons are §15.5.1's scope
rule in practice: most need a substrate primitive (a real hash, a real signature, a feed-sequence)
this corpus is correctly forbidden from inventing, or describe a runtime/network behaviour rather
than a static byte computation, or await a section (§11's responsible-person field, §18's full
timeout table) firming up.

`make coverage` reports this by counting artefacts on disk — SUITE.md rows against vector files
present — never a hardcoded total or a projected percentage, so it cannot silently go stale as the
corpus grows. A vector moves a SUITE.md row toward "runnable" only when it is derived from
normative §-text by §15.5.2's discipline, and never when it is exported from a reference
implementation and back-filled here.

Two commerce-spine vector families cannot be frozen to full coverage even now, and this is an
honest limit rather than an oversight: catalogue-canonicalisation vectors depend on the §2.3a
localization grammar change (a pending §16 MAJOR change that moves localized display text out of
the content-addressed identity), and pricing/tax-treatment vectors depend on the §5.10
tax-treatment-category slot — both already-logged §16 open items. A vector frozen against the
current `ProductRecord` shape would be invalidated the moment the localization change lands, which
is exactly the silent-invalidation §15.5.2's discipline exists to keep out of this corpus.

## 15.6 Open

The following are **not** normative and are collected for decision; each is a conformance-design
choice this section deliberately does not invent.

- **A fifth, index-building profile.** Whether derived-data (indexing) capability is worth
  advertising as a distinct profile, given §0.4.1 already treats building an index as available to
  any node without registration. The four profiles above describe transactional capability, not
  derived-data capability, and TRACT-CAT-02 and TRACT-TRUST-02 already assume index behaviour that
  no profile currently names. *PROVISIONAL — pending decision. Recommendation: do not add one;
  keep indexing profile-free per §0.4.1, and let the two cases that touch it assert against any
  node that serves an index without claiming authority.*
- **Where profile composition is enforced.** Whether the `builds on` dependency (§15.2) is enforced
  by the capability-token grammar itself, or stays a conformance-suite check layered on an
  announcement mechanism that does not itself understand dependencies. *PROVISIONAL — pending
  decision. Recommendation: enforce in the suite (TRACT-PROFILE-*), not in the substrate
  capability-token grammar, so no §16/substrate grammar change is required for a TRACT-layer rule.*
- **Announcement versus live probe.** Whether a signed capability announcement is sufficient
  evidence of conformance, or whether the suite needs a live-probe component. An announcement
  authenticates to a key, not to the behaviour behind it, and TRACT-PROFILE-02 is written against
  the latter. *PROVISIONAL — pending decision.*
- **The both-halves gateway that honours one.** What "conformant" means for a gateway that
  advertises both storefront and settlement but only actually delivers one. §15.2 permits claiming
  either half alone; the suite does not yet have a case for a gateway that claims both and honours
  one. *PROVISIONAL — pending decision. Recommendation: add a suite case treating a claimed-but-not-honoured
  half as a §15.4 advertisement violation (claiming a profile floor it does not meet), not a new
  fail-closed code.*
