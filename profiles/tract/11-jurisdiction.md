# 11. Jurisdiction

> **Drafting status.** Partially normative. The key words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY
> are to be interpreted as in BCP 14 (RFC 2119, RFC 8174). **§11.3** (disclosure fields,
> geo-availability, and fail-closed construction on a missing in-region role) and **§11.5** (erasure,
> resolved structurally) are now normative and align byte-for-byte to the frozen §16 grammar —
> `Order`, `Anchors`, `Responsible`, `Representative`, and `Offer.sell_to`. **§11.2 / §11.2a / §11.2b**
> state settled conclusions and defer every scrap of case-law reasoning to [§21.11](21-grounding.md);
> they carry no RFC 2119 keywords and ship no bytes. **Still provisional**, pending founder decisions
> recorded below: the tax-treatment **category** field (no §16 slot exists yet and its value set is
> not enumerable — §5.10, §17); whether an empty `sell_to` means "unrestricted" or "malformed"
> (§16.8); and whether `Anchors.place_of_supply` is recomputable or authoritative-as-recorded
> (§16.8). Those parts are flagged **PROVISIONAL** inline and are not yet implementable.

## 11.1 Scope

Making the legally responsible party explicit, and getting tax anchors right by construction.

## 11.2 The four anchors

The most common commerce-tax error is conflating party location with where a supply happens. These
are four distinct fields, derived from four different places:

| Anchor | Derived from | Governs |
|---|---|---|
| seller establishment | seller identity | licensing, seller-side registration |
| buyer residence | buyer disclosure at order | consumer-protection rights (generally non-waivable) |
| **place of supply** | **the Fulfilment axis (§4)** | VAT/GST, especially services and events |
| delivery destination | the shipping leg (§8) | customs, duty, product-safety regimes |

The forcing example: an event held in one country, sold by a seller in another, to a buyer in a
third. Admission to events is generally taxed where the event physically takes place, so only the
Fulfilment object knows the answer.

## 11.2a What the evidence says — and why it belongs in §21, not here

An earlier draft argued the case law inline — US marketplace-facilitator statutes, EU Council
Implementing Regulation 282/2011 Art 5b, the Commission's Explanatory Notes. That analysis is
**evidence for a design choice, not protocol text**, and it now lives where evidence belongs and is
not duplicated: [§21.11](21-grounding.md), adversarially checked across three passes. This section
keeps only the conclusions that shape the wire, and defers to §21.11 for the reasoning and its
substantial caveats:

- **There is a marketplace; the argument available is that there is no *facilitator*** — and only
  where no seller has contracted with the gateway (§21.11.1–2).
- **Escrow is the trigger.** A gateway that settles or routes payment is the actor most likely deemed
  a facilitator, and in some US states escrow alone suffices (§21.11.2) — which is why escrow is an
  operator class (§9).
- **"Render-only" is not a universal safe posture, and "the contract is between two keypairs" does
  not defeat a test that measures economic influence** rather than declarations; the EU rule
  anticipates exactly that claim (§21.11.3).
- **None of this is settled law** — no court has tested it, four US states diverge, and DAC7 and the
  GDPR erasure conflict remain open (§21.11.4–5). TRACT is shaped to be *defensible*, not *compliant
  by construction*, and must not be read as the latter.

## 11.2b The line: the protocol carries facts; computation happens at the edge

The one rule that keeps this section small and keeps TRACT agnostic across ~200 jurisdictions:

> **TRACT carries the *facts* a tax authority asks for. It never computes, collects, or remits the
> tax.** Rate lookup, threshold tracking, registration, invoicing, and remittance are the
> **seller's** or, where it settles, the **gateway's** job — policy at the edge, not bytes on the
> wire.

What the protocol therefore defines is a small, closed set of *disclosure* fields, most of which the
grammar already produces for other reasons:

- the four anchors of §11.2 — three of which come free from identity, buyer disclosure, and the
  Fulfilment axis;
- a **tax treatment category** (standard / zero / reduced / exempt, per whatever taxonomy applies) —
  never a **rate** (§5.10);
- the responsible parties an order names: seller of record, facilitator, importer of record,
  in-region responsible person, escrow/rail (§11.3).

That is the entire tax surface. Everything past it — *which* rate, *whether* a threshold is crossed,
*how* it is remitted — changes by jurisdiction and by week, so encoding it would ship stale law the
day a legislature changes a number. It is deliberately out of scope; a conformant implementation that
computes tax does so as **local policy over these fields**, interchangeably with any commercial rate
service or published schedule it trusts. This is the same discipline the substrate applies to itself
(policy at the edge, a narrow waist of facts in the middle), and the same one §5.10 already states
for rates.

## 11.3 Disclosure fields, geo-availability, and fail-closed construction (normative)

This subsection binds the tax surface of §11.2b to the frozen bytes of §16. It defines **no new
object**: `Order`, `Anchors`, `Responsible`, `Representative`, and `Offer.sell_to` are specified in
[§16.5.2](16-wire-format.md) and [§16.6](16-wire-format.md), and this text states the construction
rules over them. It carries facts only; nothing here computes, collects, or remits tax (§11.2b).

### 11.3.1 Every order names its anchors

`Order` (§16.6) carries `Anchors` at key 7 and `Responsible` at key 8; both are grammar-required, so
a decoder MUST reject an order missing either, failing closed (§16.4, `ERR_TRACT_TYPE_CONFUSION` on a
malformed decode). The construction rules over `Anchors` are:

- `Anchors` MUST carry the three mandatory anchors `seller_establishment`, `buyer_residence`, and
  `place_of_supply`, each an ISO 3166 alpha-2 `country` (§16.3).
- `Anchors.delivery_destination` MUST be present when, and only when, a shipping leg moves goods to a
  destination (§8); it MUST be absent when nothing moves. Its presence is not a proxy for
  place-of-supply — the two answer different questions and can differ (§11.2).
- `place_of_supply` MUST be **derived from the order's `Fulfilment` variant** exactly as §16.5.2 and
  §4 prescribe (ship → destination; collect / perform-at-place / return-required → the stated place;
  perform-remote and digital-grant → buyer residence; access-grant → per whether it names a place).
  An implementation MUST NOT accept `place_of_supply` as an independent argument that could disagree
  with the Fulfilment object.
- Where the Fulfilment variant does not yet resolve to a single place — most often a multi-variant
  offer whose buyer choice is unrecorded — order construction MUST fail closed with
  `ERR_TRACT_PLACE_OF_SUPPLY_UNRESOLVED` (§17, `0x0A01`) until the buyer's choice is recorded.

> **PROVISIONAL — pending decision (§16.8).** `place_of_supply` is *stored* on the sealed order rather
> than recomputed on demand, because the Fulfilment variant it derives from may change in a later
> offer revision and a tax position should reflect the terms as they stood at order time. Whether a
> verifier MAY recompute it from the referenced offer, or MUST accept it as recorded (a fact a party
> asserts, not one a verifier can check), is **undecided** and logged below. The MUST-derive rule
> above governs the *constructor*; the verifier's posture is what remains open.

### 11.3.2 Responsibility follows the money

`Responsible` (§16.6) names who is answerable and for what. Presence and absence are both
load-bearing:

- `Responsible.seller_of_record` (key 1) MUST be present on every order.
- `Responsible.facilitator` (key 2) MUST be present **if and only if** a gateway settled the payment
  for the order, and MUST be absent otherwise. Its presence is the marketplace-facilitator disclosure
  hook; its absence records that a self-hosted seller took direct payment — the **one §11.3 assertion
  §21.11.2 confirms**, because every marketplace-facilitator definition requires sales by a person
  *other than* the operator of the medium. This field discloses a **fact** (a gateway settled), not a
  legal determination; whether that fact makes the gateway a facilitator in any given forum is
  untested edge law, not protocol output (§21.11).
- `Responsible.importer_of_record` (key 3) MUST be present where a cross-border `delivery_destination`
  regime names an importer of record, and MUST be absent otherwise. Which regimes require one is edge
  policy over the anchors (§11.2b), not a protocol table.
- Escrow and rail are named on the settlement objects — `PaymentAttestation.RailClass` and
  `EscrowScope` (§9, §16.5.4, §16.6) — and MUST NOT be duplicated onto `Responsible`. The gateway that
  settles appears here as `facilitator`; its rail class and lawful scope live with the payment.

### 11.3.3 Geo-availability

`Offer.sell_to` (§16.5.2, key 5) enumerates, as ISO 3166 alpha-2 `country` codes, the territories
from which the offer may be accepted. It is the offer-side geo-availability control:

- A constructor MUST NOT build an `Order` against an offer whose `sell_to` does not include the
  buyer's acceptance territory (`Anchors.buyer_residence`). This is a fail-closed refusal, not a
  silent drop of the line.
- `sell_to` restricts *acceptance*, not *visibility*: a public `Offer` remains published and
  content-addressed regardless (§16.4). Geo-availability is enforced at order construction, where the
  buyer's territory is known, never by withholding the public object.

> **PROVISIONAL — pending decision (§16.8).** The meaning of an **empty** `sell_to` list —
> "unrestricted" or "malformed" — is **undecided**, and §16.8 leans malformed given §11's in-region
> representative requirements. Until the founder resolves it, an implementation SHOULD treat an empty
> `sell_to` as malformed and refuse to construct an order against it, which is the conservative
> fail-closed reading consistent with the rest of this document; this SHOULD becomes a MUST or is
> replaced by an explicit "unrestricted" semantics once the decision lands.

### 11.3.4 Fail-closed on a missing in-region responsible person

Some regimes require a named in-region responsible person for a regulated supply or import — for
example the EU's product-safety responsible person, and other traceability regimes enumerated in
§11.4. `Responsible.Representative` (§16.6, `{1 => country, 2 => identity-key}`) carries that person,
naming the region covered and the `IK` answerable within it.

The protocol does **not** decide which regimes require a `Representative`: that determination is edge
policy over the anchors (§11.2b), because the set of triggering regimes changes with the law and must
not be frozen into bytes. What is normative is the *construction discipline* once that policy fires:

- Where a constructor's policy determines that a regime governing one of an order's anchors requires
  an in-region responsible person and **no such person can be named**, the constructor MUST fail
  closed. It MUST NOT construct or place the order with `Representative` omitted, and MUST NOT
  silently degrade into placing the order anyway. This is a `fail-closed-block` (§17): the order stops
  until the required role is supplied by the party who must supply it.
- `Representative` is grammar-optional (§16.6, key `? 4`) precisely because most orders need none;
  the fail-closed rule above governs the case where policy says one is required, which the grammar
  alone cannot express.

> **§17 registry gap.** There is today no dedicated jurisdiction error code for "a required in-region
> responsible person is absent" (§17 lists only `0x0A01`). The behaviour above is normative as a
> `fail-closed-block`; a distinct code (proposed `ERR_TRACT_RESPONSIBLE_ROLE_MISSING` under family
> `0x0A`) is recorded for §17 below. This does not touch §16 bytes.

### 11.3.5 Tax treatment category — the field is committed, its bytes are not

The offer is intended to carry a **tax treatment category** (standard / zero / reduced / exempt, per
the applicable taxonomy) as a disclosure field, never a rate (§5.10, §11.2b). This is the only tax
*classification* the protocol carries; the rate keyed by (category, place of supply, date) is looked
up at the edge against whatever source an implementation trusts, never a table in this document.

> **PROVISIONAL — pending §16 slot and value set.** Neither `Consideration` nor `Offer` has a category
> slot in the frozen grammar yet — a logged §16 open item (§5.10) — and §17 records that the category
> **value set is not yet enumerable** before the underlying jurisdiction work is done. Until that §16
> change lands and the values are settled, **no order or offer carries a machine-readable tax
> category**, and the tax surface actually on the wire is the anchors (§11.3.1) and responsible
> parties (§11.3.2) alone. This subsection commits to the field; it does not yet specify its bytes,
> and MUST NOT be read as though it did.

## 11.4 Regimes this section must accommodate

South Africa (electronic-transaction disclosure and cooling-off, consumer protection, POPIA, VAT,
payment-side KYC); the EU (GDPR, platform trader traceability, consumer rights, in-region
responsible person for product safety, VAT one-stop schemes, platform reporting); other African
markets (national data-protection acts, local VAT registration, regional trade frameworks); New
Zealand (privacy, fair trading, consumer guarantees, GST on low-value imports); the Americas (US
economic nexus and marketplace-facilitator rules, seller-traceability legislation, Canadian and
Brazilian privacy law).

The protocol guarantees the **facts** a regulator asks for are present, signed and attributable: the
grammar makes `Anchors` and `Responsible` mandatory on every order (§16.6), and §11.3 makes their
construction fail closed rather than degrade. It does **not** make any deployment compliant, decide
which regimes bind a given trade, or constitute legal advice, and MUST NOT be read as any of those.
None of the regime-mapping above returned verified findings across the §21 passes (§21.1, §21.11);
it is design reasoning checked for internal consistency, and this section MUST NOT cite §21 as
support for it.

## 11.5 The erasure conflict, resolved structurally (normative)

Published objects are irrevocable; an erasure right cannot be satisfied against them after the fact.
The resolution is structural — keep the data out of the irrevocable family in the first place — and
it is enforced by the §16 grammar, not by reviewer discipline (§16.4):

- **No personal data enters the public quadrant.** A public-family object (`ProductRecord`, `Offer`,
  `RateCard`, `CapacityRecord`, or a storefront render bundle) MUST NOT carry a name, street address,
  contact detail, or any field that identifies or is linkable to a natural person (§0.5.1, §16.4). A
  decode or publish that violates this MUST fail closed with `ERR_TRACT_PERSONAL_DATA_PUBLIC` (§17,
  `0x0102`). Public objects reference places as a country plus a coarse locality only (`PlaceRef`,
  §16.5.2); there is **no street-address production in the public family at all**.
- **Personal data lives sealed and deletable.** Orders and everything identifying a person — buyer
  name, delivery address, contact details — are carried in the sealed family (`Order`, §16.6), held
  at the two endpoints, where deletion is meaningful. A sealed object presented under a public schema,
  or the reverse, MUST be rejected, never coerced (§16.4, `ERR_TRACT_TYPE_CONFUSION`).
- **Reviews are the single bounded exception (§10.4).** A `Review` is the one public object signed by
  a natural person, and its `body` is free text a person *could* type an address into. The grammar
  cannot prevent that; §10.4 and the client requirements MUST, and this section does not pretend the
  grammar closes it.

> **Honest limit (§21.11.4–5, §22).** Structural avoidance is a mitigation, **not** a compliance
> guarantee. GDPR Art 17 (erasure) against immutable published objects is the most likely hard
> blocker in the whole design, and it survived three §21 passes **unanswered**. Keeping personal data
> out of the irrevocable family removes the largest surface of the conflict; it does not prove the
> conflict resolved for every object a regulator might reach. The erasure-rights analysis is owned by
> §22 (with §0.5.1 and this subsection); this section states the wire consequence and cites the open
> question rather than papering over it.
