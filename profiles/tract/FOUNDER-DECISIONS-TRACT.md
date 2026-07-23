# TRACT — Founder Decision Register

This file aggregates every open decision surfaced by the section authors that requires a
**founder call** rather than a mechanical/editorial resolution. A decision lands here when it
is one or more of: a privacy/trust-posture trade with no reversal path, a scope/positioning
choice about what the protocol blesses, or a change to the **frozen** `16-wire-format.md`
grammar (a MAJOR version bump under `GOVERNANCE.md`).

Each entry records the **question**, **why it is a founder call** (not something an author may
invent under the frozen-grammar constraint), and the **author's recommendation**.

The final section, [Grammar changes needed](#grammar-changes-needed-frozen-16-sign-off),
consolidates every change that touches frozen bytes, because those need explicit sign-off
before any author may write them.

> **Coverage note.** This register now covers sections **§1–§7** and **§9–§22**, extracted directly
> from the authored section files. **§8 (Delivery)** is authored separately and its open decisions are
> appended last when that section lands; **§11 (Jurisdiction)** was already represented. Every open
> decision each covered section marks PROVISIONAL, founder-call, or Open is entered below, with the
> frozen-§16 grammar changes consolidated in the final table.

---

## §1 — Actors & identity
*Status: partial-normative*

### 1.1 Placement of the seller legal-disclosure block (§1.3)
- **Question:** Where does the seller legal-disclosure block live on the wire — on the Offer,
  on the seller's FeedHead, or both?
- **Why it is a founder call:** No §16 production exists for it, so authoring a placement would
  invent conflicting bytes against a frozen grammar. And §21.11 returned nothing verified on the
  trader-traceability regimes (DSA / INFORM / GPSR) these fields serve, so the placement carries
  unquantified legal weight.
- **Recommendation:** Carry it **once on the seller's FeedHead** (one identity, one disclosure)
  plus an **optional per-Offer override slot** for sellers trading under distinct names, so the
  common case is not duplicated per offer. Requires the §16 grammar addition (see G1).

### 1.2 Derivation of the per-store pseudonymous subkey (§1.5)
- **Question:** Is the per-store pseudonymous subkey derived deterministically from the buyer's
  root key + seller key, or generated randomly and stored in the buyer's synced cart state?
- **Why it is a founder call:** A privacy-vs-recoverability trade with **no reversal path** once
  chosen: deterministic derivation makes cross-seller purchase history retroactively joinable by
  anyone who ever learns the root key. No wire shape changes either way, so it is a policy
  mandate, not a byte question.
- **Recommendation:** Mandate the **stored random mapping** as default (structural unlinkability
  that survives root-key compromise); permit deterministic derivation only as an explicit,
  disclosed opt-in for buyers who prioritise seedphrase-only recovery.

### 1.3 Does the grant tier (§1.4) need its own signed disclosure object?
- **Question:** Does the grant tier need a dedicated signed disclosure object, or is it fully
  satisfied by the per-seller subkey with no separate protocol object?
- **Why it is a founder call:** Adding a protocol object is a §16 grammar decision with permanent
  wire-surface cost; declining one is a scope call the founder-decision list should ratify.
- **Recommendation:** **No new object** — the per-seller subkey plus the existing Review/cart
  constructions already carry every grant-tier need; a new object adds wire surface for no
  capability.

---

## §2 — Catalogue
*Status: partial-normative*

### 2.1 Near-duplicate product resolution
- **Question:** Should the spec recommend one canonical matching heuristic, or deliberately leave
  it to differ between indexes?
- **Why it is a founder call:** Trades interoperability against the core no-authority stance;
  blessing a matcher is a structural/positioning decision, not a mechanical one.
- **Recommendation:** **Leave it to differ**; optionally publish a NON-normative reference
  heuristic. Endorsing one canonical matcher would make that matcher a de facto authority — the
  exact re-centralisation §2.6 warns against.

### 2.2 Unicode value-normalisation for coded attribute VALUES
- **Question:** What concrete normalisation form + casefolding applies to coded attribute *values*
  before addressing? Frozen v0 casefolds keys but leaves value normalisation unspecified.
- **Why it is a founder call:** Without a fixed value normalisation, cross-publisher convergence
  cannot work in practice; the choice binds the frozen grammar's real behaviour and the §15 vector
  corpus, and is effectively frozen for convergence once chosen.
- **Recommendation:** Adopt **Unicode NFC + Unicode default casefold** for enumerated values —
  **but confirm before writing**, since it touches every conformance vector.

### 2.3 Manufacturer-signature bootstrapping
- **Question:** How does the top identity rung bootstrap when most brands will not publish an IK
  early?
- **Why it is a founder call:** Adoption/GTM strategy question, not a byte-level one.
- **Recommendation:** Ship v1 with the ladder **tolerant of an absent ManufacturerSignedRung** and
  no bootstrap mechanism; treat brand-key adoption as an ecosystem effort, not a protocol feature.

### 2.4 ProductRecord localization slot (§2.3a)
- **Question:** Schedule the §16 change moving display name/description out of the
  content-addressed identity into a language-tag→text map — a MAJOR version bump.
- **Why it is a founder call:** Any change to a frozen §16 shape is a MAJOR version bump under
  `GOVERNANCE.md`; timing and version discipline is a founder call.
- **Recommendation:** Schedule as the **v1 MAJOR change**; until it lands, document explicitly that
  frozen v0 does **NOT** yet achieve language-neutral identity (§2.3b PROVISIONAL note). Requires
  the §16 grammar change (see G4).

### 2.5 Explicit ProductGroup / variesBy production?
- **Question:** Add an explicit ProductGroup / variesBy production to §16, or keep recovering
  varying axes from differing coded Attributes?
- **Why it is a founder call:** Adds grammar surface and a version bump for marginal schema.org
  fidelity; a scope/complexity decision (cf. §21.7's NIP-15 "too complicated" caution).
- **Recommendation:** **Keep the attribute-based reading** (no grammar change) — §2.3d already
  makes it normative and it needs no new bytes.

---

## §3 — Availability
*Status: authored-normative*

### 3.1 Is the four-arm StockSignal set final?
- **Question:** Is the four-arm StockSignal set (exact / in-stock / low / out-of-stock) the final
  stock vocabulary, or should a richer seller-defined band namespace be added?
- **Why it is a founder call:** Changes the frozen §16 grammar (MAJOR bump) and the public
  commercial-privacy surface; not resolvable by an author under C5/C7.
- **Recommendation:** **Keep the four frozen arms.** Band thresholds (what count maps to `low`)
  are already seller policy and unspecifiable by protocol; adding arms is a MAJOR change for
  marginal gain.

### 3.2 Should Availability carry its own freshness parameter?
- **Question:** Should `Availability` carry its own freshness parameter, distinct from the Offer's
  `published` timestamp and from RateCard's `RATE_CARD_MAX_AGE`, since stock and slots move faster
  than price?
- **Why it is a founder call:** Requires a frozen-grammar change (MAJOR bump) and trades wire size
  against freshness honesty; a founder call per C5/C7.
- **Recommendation:** **Add a per-variant optional max-age hint** so a stale stock band is flagged
  separately from a day-old price, rather than overloading the Offer's single `published`.
  Requires the §16 grammar change (see G5).

### 3.3 `made-to-order` lead days — calendar or working days?
- **Question:** Are `made-to-order` lead days (§16.5.2 key 6, bare uint) counted in calendar days
  or working days?
- **Why it is a founder call:** The grammar cannot encode the distinction and it feeds §19
  `FULFIL_TIMEOUT` / cancellation rights; picking a convention is a normative behaviour decision
  the author must not invent.
- **Recommendation:** Fix the wire meaning as **CALENDAR days** (unambiguous across
  jurisdictions/holidays); leave working-day presentation to the seller's display layer. Note:
  the reference implementation currently documents working days, so this is a **reconciliation**,
  not a greenfield choice.

### 3.4 `capacity-per-interval` — total cap or live remaining?
- **Question:** Is the `capacity-per-interval` number the published total cap or a live remaining
  count?
- **Why it is a founder call:** The draft explicitly listed it as open; although §3.9's
  signal-not-reservation rule entails the total reading, the author flags it for confirmation
  rather than silently closing a listed-open item.
- **Recommendation:** **Confirm TOTAL / published-cap**, as normatively stated in §3.5. §3.9's
  signal-not-reservation rule already forces this (a live remainder would need per-interval
  booking state the public object cannot carry).

---

## §4 — Fulfilment
*Status: partial-normative*

### 4.1 Buyer counter-signature for carrier-less fulfilment variants?
- **Question:** Must the carrier-less fulfilment variants (collect, perform-at-place,
  perform-remote) require the buyer's counter-signature to reach the delivered state, rather than
  accepting the seller's signature alone?
- **Why it is a founder call:** Changes the trust posture of an order state transition and the
  buyer's recourse; §18.3 currently signs "carrier or seller", so resolving it re-touches the
  order state machine. Not derivable from the grammar alone.
- **Recommendation:** **Require buyer counter-signature** (or a `CONFIRM_TIMEOUT` auto-close) for
  these three variants; a seller-only "delivered" is the exact unilateral assertion the
  signed-transition model exists to avoid. `digital-grant` and `access-grant` stay seller-signed
  (the artefact is the seller's own, no third party could counter-sign). Coordinate with §18.3's
  transition signer.

### 4.2 Dedicated Incoterm 2020 wire field for the ship variant?
- **Question:** Does the ship fulfilment variant need a dedicated Incoterm 2020 wire field, or
  does the Incoterm remain a leg/order-level convention outside the offer?
- **Why it is a founder call:** A "yes" requires changing the frozen §16.5.2 ship variant (MAJOR
  version bump); the grammar is founder-frozen at v0.
- **Recommendation:** **Keep it a convention out of band for v0**; the risk/cost-transfer point is
  order/leg-level, not an offer-axis fact. Revisit only if a concrete regime forces the Incoterm
  onto the offer. (Conditional grammar change G-C1 if resolved "yes".)

### 4.3 Does digital-grant need a licence-terms sub-object?
- **Question:** Does `digital-grant` need a licence-terms sub-object, or does it defer entirely to
  the §5 consideration axis?
- **Why it is a founder call:** Introducing the sub-object is a new frozen-grammar shape (MAJOR
  bump) and re-opens the §4/§5 axis boundary.
- **Recommendation:** **Defer to §5 for v0**; keep the grammar frozen. A licence-terms sub-object
  duplicates pricing/term expression that §5 already owns. (Conditional grammar change G-C2 if
  resolved "yes".)

### 4.4 First-class grouping for repeated same-item / different-fulfilment offers (§4.8)?
- **Question:** Do repeated same-item, different-fulfilment offers warrant a first-class grouping
  construct, given the grammar currently expresses the case only as separate independently signed
  Offers?
- **Why it is a founder call:** A grouping construct is a new frozen-grammar object (MAJOR bump)
  with signing/convergence implications across §16 and §18.2.
- **Recommendation:** **No grouping construct for v0**; the two-offers pattern is correct and
  independently withdrawable. Leave to a later MAJOR version if the repetition proves costly.
  (Conditional grammar change G-C3 if resolved "yes".)

---

## §5 — Consideration
*Status: partial-normative*

### 5.1 Shared-currency rule inside a single Consideration?
- **Question:** Should a shared-currency rule be enforced inside a single Consideration value
  (deposit+balance's two money fields, and each PriceTier's `unit_price`)?
- **Why it is a founder call:** Changes the frozen §16.5.2 Consideration shape — a MAJOR version
  bump under `GOVERNANCE.md`, not a spec correction.
- **Recommendation:** **Add a §16 grammar-level rule** requiring one currency per Consideration;
  until then the section states SHOULD-refuse and MUST-not-coerce. Mirrors the settled
  one-currency-per-order rule. (Grammar change G6.)

### 5.2 PriceTier ordering, duplicate rejection, and below-lowest-tier behaviour
- **Question:** Does PriceTier need canonical ordering + duplicate-`min_qty` rejection, and what
  does a quantity below every published tier resolve to?
- **Why it is a founder call:** Canonicalisation is a §16 MAJOR change; the below-lowest-tier
  behaviour is a pricing-policy call the evidence does not settle.
- **Recommendation:** **Add sorted/deduplicated canonicalisation** to the PriceTier array (matching
  ProductRecord attributes), and rule that a quantity below every tier is **unpriceable** (offer
  not applicable) rather than defaulting to the lowest tier. (Grammar change G7.)

### 5.3 Recurring-consideration renewal vs §18.3 single-shot lifecycle
- **Question:** How do a recurring consideration's renewal periods map onto §18.3's single-shot
  order lifecycle?
- **Why it is a founder call:** Defines new lifecycle semantics across §5/§18 that do not exist
  today; picking fresh-Order vs persistent-Order is an architectural decision, not a byte
  alignment.
- **Recommendation:** **Treat each renewal period as a fresh sealed Order** (per-period
  accept/decline), since Order is lightweight and "was this charge accepted" is naturally
  per-period; add a renewal path note to §18.3. (Grammar/lifecycle change G10.)

### 5.4 Metered consideration — usage-attestation object and Order.total
- **Question:** Does metered consideration need a usage-attestation object, and how does a metered
  Order satisfy the mandatory `Order.total` when the true amount is only known after consumption?
- **Why it is a founder call:** Requires a new §16 sealed object (MAJOR change) and a decision on
  whether total is estimate/cap/other — neither derivable from the frozen grammar.
- **Recommendation:** **Add a sealed usage-attestation object** to §16.6 and define metered
  `Order.total` as an **estimate-or-cap** placed at order time, reconciled by the attestation at
  close. (Grammar change G8.)

### 5.5 Quote-request wire shape and back-reference on private Offers
- **Question:** Does a quote request need a defined sealed wire shape, and does a private per-buyer
  Offer need a field linking it back to the request that produced it?
- **Why it is a founder call:** Both are new §16 shapes (MAJOR change); the current mechanism works
  without them, so adding them is a scope decision.
- **Recommendation:** **Add a sealed quote-request object** and an **optional request-reference
  field on Offer** so a buyer holds a signed record of having asked, not just of having received.
  (Grammar change G9.)

### 5.6 Where does the tax treatment category live on the wire?
- **Question:** Where is the tax treatment category carried, given neither Consideration nor Offer
  has a slot for it today?
- **Why it is a founder call:** §5.10 is founder-settled that the category belongs here, but the
  grammar has no slot; adding one is a §16 MAJOR change that must be placed deliberately, not
  invented.
- **Recommendation:** **Add a treatment-category slot** to Offer (or Consideration) carrying a
  **coded category string only — never a rate** — per the §5.10 founder-settled policy.
  (Grammar change G-tax, listed as part of the §5 grammar set.)

### 5.7 Shape of the optional forward-auction profile
- **Question:** What is the exact shape of the optional forward-auction profile over WRAP's Bid
  (close-time semantics, sealed-vs-open visibility, seller-assigned winner re-entering TRACT as an
  Order)?
- **Why it is a founder call:** Lives in WRAP (`github.com/vul-os/wrap`), reintroduces a bounded
  close-time race, and requires per-auction visibility choices — a cross-repo design decision
  beyond this section's authority.
- **Recommendation:** **Specify it in WRAP's namespace** as an optional profile (seller-assigns at
  close), **not in TRACT**; TRACT only references it. Competitive procurement needs no new shape
  (WRAP unchanged). No TRACT grammar change.

---

## §6 — Cart
*Status: authored-normative*

### 6.1 Recipient selection for bounded-counter rights transfer (N ≥ 3 replicas)
- **Question:** When a bounded-counter rights transfer must name a recipient, who chooses the
  recipient and by what allocation policy (round-robin, demand-weighted, seller-elected
  coordinating replica)?
- **Why it is a founder call:** §21.10.2 item 4 flags this as an operator-shaped allocation policy
  the source papers suggest centralising; whether the spec mandates one or defers to
  implementations is a design-purity call, not a mechanically derivable one.
- **Recommendation:** **Do not mandate a single algorithm.** Leave the mechanism to the seller but
  require a normative floor: the policy MUST be **deterministic per replica set and disclosed** in
  the seller's operational configuration, so it cannot violate the intra-replica serialization
  constraint. **No §16 change** (seller-internal SYNC state).

### 6.2 Replica rebalancing aggressiveness — mandate a policy?
- **Question:** How aggressively should a seller's replicas rebalance/transfer quota to reduce
  stranding, and should the spec mandate one rebalancing policy or leave it to implementations?
- **Why it is a founder call:** Carried over from §6.5's existing open item; whether the spec
  normatively constrains this is a scope/standardisation call reserved for the founder.
- **Recommendation:** **Leave it to implementations.** Rebalancing aggressiveness is a pure
  liveness/overhead trade-off with no effect on the safety invariant; a mandated policy would be a
  proposal with no measurement behind it (§19.8). No §16 change.

---

## §7 — Order
*Status: partial-normative*

### 7.1 Order amendment — superseding object or append-only op-log?
- **Question:** Is order amendment a superseding sealed Order object or an append-only operation
  log against the original?
- **Why it is a founder call:** Undecided at the wire-format level (§16.8); picking either shape is
  a MAJOR §16 grammar commitment (a superseding-order predecessor pointer vs an op-log object), not
  a §7 editorial call.
- **Recommendation:** **Supersession** — it reuses the pattern already in the document (offer
  withdrawal §18.2, review retraction §10.2) rather than inventing an order-only mechanism; the
  audit-trail advantage of a log can be reconstructed by walking the successor chain. (Grammar
  change G11.)

### 7.2 Partial fulfilment — per-line status or order split?
- **Question:** Is partial fulfilment represented by a per-line status field on OrderLine, or by an
  amendment that splits one order into a closed successor and a still-fulfilling successor?
- **Why it is a founder call:** The frozen grammar has no per-line status (OrderLine is
  `{offer, seller, qty}`); a per-line status is a MAJOR §16 change. The choice is coupled to the
  amendment decision (7.1) and must be made after it.
- **Recommendation:** **The split**, to stay consistent with §16.8's supersession lean — but only
  after the 7.1 amendment decision is made, since it depends on it. (Grammar change G12,
  conditional on 7.1 = supersession.)

---

## §8 — Delivery
*Status: partially normative (rate-card mode); on-demand dispatch + custody are WRAP's (§8.9)*

### 8.1 Peer-courier rate-card shape (§8.2, §8.8)
- **Question:** Does a peer courier need a distinct rate-card shape — a flat price per kilometre —
  or should it be expressed inside the same zone-table `RateCard` model as a national carrier, as a
  one-entry zone table?
- **Why it is a founder call:** A per-kilometre variant is a change to the frozen §16.5.3 `RateCard`
  grammar (a MAJOR version bump); the author must not invent a production the bytes do not carry.
- **Recommendation:** Prefer the **single-entry zone-table** expression (no grammar change) unless a
  concrete peer-courier deployment proves the one-entry table too lossy; the identical-type property
  (a bicycle and a multinational priced by one computation) is worth preserving. (Conditional grammar
  change G-C7 if resolved toward a per-km variant.)

### 8.2 Consolidation route-choice reproducibility (§8.5, §8.8)
- **Question:** Must two conformant buyer nodes reach the same consolidation total from the same
  nominal inputs, or may they legitimately differ because their held `RateCard` copies are at
  different points within `RATE_CARD_MAX_AGE`?
- **Why it is a founder call:** This is a conformance-semantics decision (does §15 assert route-choice
  determinism?), not a byte alignment — and it trades interoperability against the offline,
  local-computation model.
- **Recommendation:** **Do not require cross-node reproducibility** of the total; require only that
  each node's computation over *its own* held cards is deterministic and that a stale card is flagged
  (§8.2). Fabricating agreement across nodes holding different-age cards would misrepresent the
  offline model. Reference the §19 `RATE_CARD_MAX_AGE`-class decision (below) — the two are linked.

*(§8's `RATE_CARD_MAX_AGE`-split question is de-duplicated to the §19 "RATE_CARD_MAX_AGE class"
decision, per this register's cross-section convention.)*

---

## §9 — Settlement
*Status: partially normative*

### 9.1 Escrow state-transition and ruling wire objects (§9.4.3, §9.7)
- **Question:** Should the escrow lifecycle carry, on the wire, a signed **state-transition** object
  (`funded` / `held` / `released` / `refunded` / `split`) and a signed **ruling** object (its
  disposition, including a split's per-party amounts), or is the lifecycle settled off-protocol by
  the operator with only the resulting `PaymentAttestation`s observable?
- **Why it is a founder call:** The frozen §16 carries neither object — it has `EscrowScope` (public)
  and `PaymentAttestation` (sealed) but nothing that carries a transition's signer/from-to state or a
  ruling's disposition. Making §9.5's "every ruling is published as a signed object" and §18.5's
  "each step is a signed object" normative therefore requires a §16 **MAJOR** grammar change, which
  no author may write unsigned. The same gap is raised independently by §18.5 and §12.2b. Until it
  lands, the *published-ruling* guarantee (a badly-ruling operator accumulating a verifiable record,
  §9.5) is an intended property not yet expressible on the wire.
- **Recommendation:** **Add** the escrow-transition and ruling objects to §16.6 so the
  verifiable-record guarantee becomes real, carrying the split's per-party amounts on the ruling;
  until then §18.5's transitions remain a state machine whose published-ruling property is stated but
  unexpressible. Recorded as a required §16 change (see G13).

---

## §10 — Trust
*Status: normative (review/attestation model)*

### 10.1 Seller-response object (§10.2c, §10.6)
- **Question:** Does a subject a review names get a first-class wire object to respond in public,
  attached to the same subject — and if so, what shape?
- **Why it is a founder call:** No §16 production carries a seller response today: `Review` can only
  name a product/seller/distributor/courier as subject, not another review, and there is no
  `ReviewResponse` object. A response therefore requires a **§16 grammar change** (a public object
  referencing a review's `content-address`, authored by the review's subject key), which is a MAJOR
  bump no author may invent.
- **Recommendation:** Add a minimal **`ReviewResponse`** public object — subject-key author, target
  review `content-address`, bounded body reusing `MAX_REVIEW_BODY`, `ts` — under the same
  public-quadrant personal-data prohibition, so it adds no new privacy surface; keep it optional and
  index-displayed alongside the review, never merged into the score. Requires the §16 grammar change
  (see G16).

### 10.2 Is a valid `PurchaseAttestation` a protocol floor or index-local policy?
- **Question:** Should an index requiring a valid `PurchaseAttestation` before serving a `Review` be
  a network-wide protocol floor, or remain index-local policy?
- **Why it is a founder call:** A scope/positioning choice about what the protocol blesses: mandating
  the gate network-wide would require the aggregating authority §10.3 exists to remove, so declining
  to mandate it is a stance the founder-decision list should ratify, not an editorial call.
- **Recommendation:** **Keep it index-local** (`ERR_TRACT_REVIEW_UNATTESTED` stays a policy deny, not
  a decode failure); the honest cost of optionality (§10.3a) is disclosed rather than engineered
  away. No §16 change.

---

## §12 — Gateway
*Status: normative, one item scoped*

### 12.1 Canonical render, or expose the objects only? (§12.6)
- **Question:** Should the spec fix a **canonical render** — a deterministic, reproducible mapping
  from a store's signed objects to bytes — so the byte-for-byte comparison the §12.5 trust mitigation
  relies on is well-defined, or require only that the underlying signed objects be exposed for
  independent re-rendering, leaving presentation bytes unconstrained?
- **Why it is a founder call:** A scope/positioning trade with a permanent cost either way: a
  canonical render makes the mitigation crisp but freezes presentation and is fragile against every
  browser/CSS revision; exposing the objects keeps presentation free but makes "compared
  byte-for-byte" an assertion about the objects, not the rendered page. If canonical render is
  chosen, §16.5 has no production for a TRACT-native render bundle, so it would additionally force a
  MAJOR §16 grammar change (§12.4a).
- **Recommendation:** For v0, keep the §12.5 mitigation stated over the **underlying objects** and
  add no canonical-render requirement (the current default); only if a canonical render is later
  required does the render-bundle-as-native-object grammar change arise (conditional G-C5).

---

## §13 — Analytics
*Status: partially normative (semantics), bytes PROVISIONAL*

### 13.1 Wire encoding of the disclosure `Grant` and `GrantRevocation` (§13.3)
- **Question:** How are the analytics disclosure `Grant` and its `GrantRevocation` encoded on the
  wire, given the frozen v0 grammar defines neither?
- **Why it is a founder call:** Both are new sealed-family objects with no CDDL in §16.6; the field
  *semantics* are already normative, but committing their bytes is a §16 **MAJOR** version bump no
  author may write unsigned, and until it lands no byte-level conformance vector for them can exist.
- **Recommendation:** **Add** sealed `Grant` and `GrantRevocation` productions to §16.6 matching the
  normative semantics of §13.3 (one `store`, per-store pseudonymous `subject`, enumerated `fields`,
  bounded lifetime, `subject`-key signature). Requires the §16 grammar change (see G14, G15).

### 13.2 Does the per-store subject key need a non-linkability proof? (§13.8)
- **Question:** Does the per-store pseudonymous subject key need a proof of non-linkability beyond
  "it is a different key per store," given a seller who also runs or buys an index could correlate a
  naive derivation across stores?
- **Why it is a founder call:** This is the same index re-centralisation risk that is one of the two
  weakest load-bearing claims in the whole document (§2.6, §21.3/§21.9) — a privacy claim with no
  reversal if it turns out false, which an author must not silently upgrade from "intended" to
  "proven."
- **Recommendation:** **Keep it marked as weakest and undefended** rather than assert a proof the
  evidence does not support; treat closing it as a flagged research dependency, not a claim the spec
  may make today.

### 13.3 Is 24 h the right `GRANT_LIFETIME` for analytics? (§13.8, §19.6)
- **Question:** Is the 24-hour `GRANT_LIFETIME` default right for analytics specifically, or carried
  over from the same round-number instinct that set `FUND_TIMEOUT` to 24 h for an unrelated reason?
- **Why it is a founder call:** The value binds a consent-expiry default with no measurement behind
  it (§21.1); picking it is a privacy-posture default, not a mechanical one.
- **Recommendation:** Revisit against analytics-specific reasoning before v1; no field measurement
  exists to confirm the current value.

### 13.4 Formal noise on top of bucketing? (§13.8)
- **Question:** Does aggregate telemetry (§13.4) need formal noise (e.g. differential privacy) on top
  of bucketing, or are coarse buckets sufficient at the traffic volumes real stores see?
- **Why it is a founder call:** A privacy-vs-utility trade unresolved because the volumes themselves
  are unmeasured (§13.5, §21.1); not derivable without data an author does not have.
- **Recommendation:** Unresolved; recorded pending measurement of real store traffic volumes.

### 13.5 Must revocation unwind already-folded counts? (§13.8)
- **Question:** Does a revoked grant oblige a seller to remove its already-folded contribution to a
  past aggregate, given folding is one-way by construction and "stop disclosing" may not be the same
  operation as "undisclose"?
- **Why it is a founder call:** A privacy-obligation call with no clean mechanism — the aggregate
  cannot structurally unwind one contribution — so whether to *require* it is a posture decision, not
  an implementation detail.
- **Recommendation:** Unresolved; recorded for the founder-decision list.

### 13.6 A machine-readable purpose field on the grant? (§13.8)
- **Question:** Does the grant object need a machine-readable purpose field (marketing, fraud,
  fulfilment-adjacent measurement) so a buyer's node can grant narrowly, or is the `fields`
  enumeration alone granular enough?
- **Why it is a founder call:** It changes the shape of the (still-provisional) `Grant` object and
  the consent surface a buyer reasons about; a byte-shape decision folded into the §16 Grant
  production (G14), not an editorial one.
- **Recommendation:** Decide alongside the `Grant` encoding (G14); if added, it is an optional field
  on the sealed Grant, never a value meaning "all purposes."

---

## §14 — Anti-abuse
*Status: partially normative*

### 14.1 Do feed-append rate and per-publisher storage quota belong in §19? (§14.3, §14.9)
- **Question:** Should feed-append rate and per-publisher storage quota carry suggested defaults in
  §19, or are they inherently holder-local policy that a shared table would misrepresent as
  protocol-level?
- **Why it is a founder call:** A scope/positioning choice — listing a default in §19 reads as a
  protocol-level floor, which conflicts with the "requirements on what a holder is willing to carry,
  not on what a seller may publish" posture (§0.4.1); an author cannot pick which framing the spec
  blesses.
- **Recommendation:** Left open in §14.9; lean toward keeping them holder-local (no §19 default),
  since a shared table would imply a network-wide floor these deliberately are not.

### 14.2 The achievable Sybil-cost floor on a signed-feed substrate (§14.7, §21.8)
- **Question:** What Sybil-cost floor is achievable on a signed-feed substrate — do buyer
  counter-signatures, transaction-cost binding, or purely local web-of-trust scoring actually raise
  it above "discount new keys"?
- **Why it is a founder call:** It is listed as entirely **unresearched** by both literature passes
  (§21.8) and is shared with WRAP; no normative floor stronger than "discount new keys" may be stated
  until the research lands, and §14.7 marks it PROVISIONAL and records it here so it is not lost. (The
  same gap surfaces in §10.6, where it is framed as a research gap rather than a decision.)
- **Recommendation:** No floor stronger than "discount new keys" is stateable today; recorded as a
  blocking research dependency, not a decision that can be closed by drafting.

### 14.3 A machine-readable cold-contact policy declaration on the seller's feed? (§14.5, §14.9)
- **Question:** Should a seller's cold-contact policy be a machine-readable declaration on its feed —
  so a buyer's node knows before sending whether an unsolicited order will even be looked at — or is
  it better left an undeclared, seller-side heuristic?
- **Why it is a founder call:** No §16 grammar slot exists for such a declaration; adding one is a
  MAJOR §16 change (§16), which no author may invent.
- **Recommendation:** Recorded for the founder-decision list without a firm lean; if resolved toward
  a declaration it forces the conditional §16 change (see G-C4).

---

## §15 — Conformance
*Status: normative, open items scoped*

### 15.1 A fifth, index-building profile? (§15.6)
- **Question:** Is derived-data (indexing) capability worth advertising as a distinct fifth profile,
  given §0.4.1 already treats building an index as available to any node without registration?
- **Why it is a founder call:** A scope/positioning choice about what capability the profile
  vocabulary blesses; naming an index profile risks implying an authority §0.4.1 denies.
- **Recommendation:** **Do not add one;** keep indexing profile-free per §0.4.1, and let the two
  cases that touch it (TRACT-CAT-02, TRACT-TRUST-02) assert against any node that serves an index
  without claiming authority.

### 15.2 Where is profile composition enforced? (§15.6)
- **Question:** Is the `builds on` dependency (§15.2) enforced by the capability-token grammar itself,
  or by a conformance-suite check layered on an announcement mechanism that does not understand
  dependencies?
- **Why it is a founder call:** The two answers differ in whether a substrate/§16 grammar change is
  required — a positioning choice about where the rule lives, not a mechanical one.
- **Recommendation:** **Enforce in the suite** (TRACT-PROFILE-*), not in the substrate
  capability-token grammar, so no §16/substrate grammar change is required for a TRACT-layer rule.

### 15.3 Signed announcement, or live probe? (§15.6)
- **Question:** Is a signed capability announcement sufficient evidence of conformance, or does the
  suite need a live-probe component, since an announcement authenticates to a key, not to the
  behaviour behind it?
- **Why it is a founder call:** A conformance-model choice that sets how much the suite trusts a
  signature over observed behaviour; not resolvable by an author without founder direction.
- **Recommendation:** Left open in §15.6; TRACT-PROFILE-02 is already written against the behaviour,
  not the announcement.

### 15.4 What does "conformant" mean for a both-halves gateway that honours one? (§15.6)
- **Question:** How is a gateway judged that advertises both storefront and settlement but actually
  delivers only one?
- **Why it is a founder call:** A conformance-classification choice — whether this is an advertisement
  violation or a new failure class — that the suite deliberately does not yet decide.
- **Recommendation:** Add a suite case treating a **claimed-but-not-honoured half as a §15.4
  advertisement violation** (claiming a profile floor it does not meet), not a new fail-closed code.

---

## §17 — Errors & registries
*Status: normative for machinery, partial above the code level*

### 17.1 Registries as in-document tables, or a machine-readable file? (§17.8)
- **Question:** Do the §17.5 registries live as tables in this document, or as a separate
  machine-readable file the document points to?
- **Why it is a founder call:** A single-source-of-authority choice with drift consequences across
  the whole toolchain; positioning, not mechanical.
- **Recommendation:** Keep the normative source **in this document** and **generate** the
  machine-readable file from it in the build, so there is one authority and no drift.

### 17.2 Allocation policy for the unassigned and private-use subsystem ranges (§17.2, §17.8)
- **Question:** What allocation policy governs the `0x0D`–`0xEF` unassigned range and the
  `0xF0`–`0xFE` private-use range — Specification Required, first-come-first-served, or something
  else?
- **Why it is a founder call:** A registry-governance choice that binds how the code space evolves
  under `GOVERNANCE.md`; mirroring DMTAP is one option, not a commitment an author may assume.
- **Recommendation:** **Specification Required** for the unassigned range (a code is only
  interoperable if the rule it names is written down); **no registration** for private use
  (non-interoperable by definition).

### 17.3 Does the excluded-category vocabulary graduate to a controlled list? (§17.8)
- **Question:** Should the excluded-category vocabulary graduate from free text to a controlled list?
- **Why it is a founder call:** A controlled list is machine-actionable but needs a maintainer — the
  exact centralising pressure §2.6's index rule exists to keep out; whether to create that role is a
  positioning decision.
- **Recommendation:** **Keep it free text;** a controlled category list is a maintainer-shaped role
  the design should not create without a demonstrated need.

### 17.4 Can tax-treatment categories be enumerated before the jurisdiction research lands? (§17.8)
- **Question:** Can tax-treatment categories (§11.3.5) be enumerated at all before the jurisdiction
  research is done?
- **Why it is a founder call:** Populating the vocabulary with real category values carries
  unquantified legal weight (§21.11 records the research as unresearched), and the field's own §16
  slot is still a logged open item (§5.10) — a legal/scope call, not editorial.
- **Recommendation:** **Do not enumerate** values in this document until the research lands.

### 17.5 Does TRACT register a media type or reserve a well-known URI? (§17.7, §17.8)
- **Question:** Does TRACT register a media type and/or reserve a well-known URI in v0?
- **Why it is a founder call:** A `.well-known/` path is a host-centric discovery point that would
  reintroduce the de-facto-gatekeeper surface §21.3/§21.4 records as decentralized commerce's first
  re-centralization failure — a positioning trade, not a mechanical registration.
- **Recommendation:** **Neither in v0.** If a later non-substrate transport forces content-type
  negotiation, register a **single** `application/tract+cbor` type with the object family carried as
  a parameter (never a per-object-type family), and reserve **no** well-known URI — discovery stays
  index-mediated.

---

## §18 — State machines
*Status: partially normative*

*(The recurring-renewal, metered-total, and escrow-object questions surfaced in §18.3/§18.5 are the
§18-side of decisions **5.3**, **5.4**, and **9.1** respectively, and are not re-listed here.)*

### 18.1 Is `CONFIRM_TIMEOUT` a protocol floor or an offer-level term? (§18.3, §18.7, §19.2)
- **Question:** Is `CONFIRM_TIMEOUT` — the most consequential parameter in the document — a Fixed
  protocol floor, or a Declared offer-level term?
- **Why it is a founder call:** It sets a buyer's recourse window, and the answer decides whether §19
  is normative for the value or merely suggests it. A seller of perishables and a seller of furniture
  want very different values (argues offer); a buyer comparing offers should not have to read a
  timeout to know their recourse (argues protocol). Not derivable by an author.
- **Recommendation:** **Fixed protocol floor with a Declared offer-level extension upward only** —
  an offer may lengthen the window but never drop below the floor (14 days proposed).

### 18.2 Does a dispute pause `CONFIRM_TIMEOUT`, or run alongside it? (§18.7)
- **Question:** When a dispute is raised, does `CONFIRM_TIMEOUT` pause, or keep running?
- **Why it is a founder call:** A trust-posture trade with no neutral answer: pausing lets a bad-faith
  buyer stall indefinitely by disputing; not pausing lets a slow operator time a real dispute out into
  an automatic release (§18.5).
- **Recommendation:** Left open in §18.7; stated as a genuine trade-off rather than resolved.

### 18.3 Does `lost` need a distinct `disputed-lost` state? (§18.4, §18.7)
- **Question:** Does the custody lifecycle need a distinct `disputed-lost` state, since the single
  terminal `lost` cannot distinguish an agreed loss from a custodian/recipient disagreement about
  whether delivery happened at all?
- **Why it is a founder call:** The custody machine is **WRAP's**, so adding the state is a **WRAP
  profile change, not a §16 grammar change** — a cross-repo decision beyond this section's authority
  (hence no §16 grammar-table row).
- **Recommendation:** Recorded for the founder-decision list; resolve in WRAP if adopted, not in
  TRACT's §16.

---

## §19 — Parameters
*Status: partially normative*

### 19.1 Does `DISPUTE_TIMEOUT` need a machine-readable slot on `EscrowScope`? (§19.4, §19.8)
- **Question:** Should `DISPUTE_TIMEOUT` — and the non-custodial-rail dispute behaviour §18.5
  requires be disclosed before the trade — have a dedicated machine-checkable field on `EscrowScope`,
  rather than living in prose (field 9, "authorities claimed") or out of band?
- **Why it is a founder call:** A dedicated field is a §16 **MAJOR** change (`EscrowScope` is frozen
  at §16.5.4); making the before-the-trade disclosure machine-enforceable versus prose-only is a
  trust-posture-vs-wire-cost trade an author may not invent.
- **Recommendation:** Recorded with its cost noted; a machine-checkable field would make the §18.5
  disclosure enforceable at the price of a MAJOR §16 change (see G17).

### 19.2 Is `RATE_CARD_MAX_AGE` a single Fixed constant or a per-card Declared value? (§8.8, §19.8)
- **Question:** Is `RATE_CARD_MAX_AGE` a single Fixed protocol constant, or a per-card Declared value
  a carrier may set on its own rate card?
- **Why it is a founder call:** A carrier with stable surcharges and one whose fuel levy moves weekly
  want different windows (argues Declared); a buyer comparing cards should not have to read a
  freshness term to know a quote is trustworthy (argues Fixed). The decision is **owned by §8.8**
  (authored separately) and mirrored in §19.8.
- **Recommendation:** **Set Fixed at 30 days for now**, with an implementation warning the buyer when
  a card older than that is used to quote.

### 19.3 Should `MAX_REVIEW_BODY` be very much smaller? (§19.5, §19.8, §22.8.4)
- **Question:** Should `MAX_REVIEW_BODY` shrink well below 8 KiB — nearer a sentence — trading review
  usefulness for a smaller personal-data residual in a public irrevocable object?
- **Why it is a founder call:** A privacy-vs-usefulness trade with a permanent residual (the smaller
  the field, the less identifying detail a public irrevocable object can carry); the current value was
  chosen for usefulness and §22.7 endorses revisiting it downward. Not an editorial pick.
- **Recommendation:** **Revisit downward** toward the privacy purpose; the exact value is §19's call
  and is left Open.

---

## §20 — References
*Status: normative for the tables; maintenance questions open*

### 20.1 Version-pinning for revisable standards (§20.7)
- **Question:** Do the informative reference tables need version pinning the way RFCs are immutably
  numbered — e.g. what happens to a signed offer naming "Incoterms 2020" once Incoterms 2030 exists?
- **Why it is a founder call:** The recommended fix (pin the edition in the offer's own fields) is a
  **§16 / §4.6 shape question** — a frozen-grammar change — not a §20 editorial one, and interacts
  with the conditional Incoterm slot (G-C1).
- **Recommendation:** Pin the version in the offer's own fields (so a signed offer commits to the
  edition it was priced against) and let the reference table name the current default only; deferred
  to §16 / §4.6 as a shape question.

### 20.2 Extending the T5 completeness check beyond RFCs (§20.7)
- **Question:** Should the machine-checked T5 completeness rule extend to ISO standards, schema.org
  terms and GS1 identifiers, which have identical drift risk and none of the automated protection?
- **Why it is a founder call:** A specification-maintenance/governance choice about which references
  the build guards, recorded for the founder-decision pass rather than silently changed.
- **Recommendation:** **Extend it** — the marginal linter cost is low and the manual-drift surface is
  the larger half of the section.

### 20.3 A "checked as of" date on the legal table (§20.4, §20.7)
- **Question:** Should each §20.4 legal-instrument row be stamped with the date its reading was
  verified, since a statute list is not append-only the way an RFC list is (Washington amended its
  facilitator definition in 2026; ViDA will change EU scope)?
- **Why it is a founder call:** A maintenance-discipline choice about how the legal evidence table
  ages; it governs how the list is maintained, not which references are normative.
- **Recommendation:** **Stamp each §20.4 row** with the date its reading was verified, mirroring
  §21.11's own dating.

### 20.4 Pinning the substrate rows to a commit (§20.7)
- **Question:** Should the §20.2.1 substrate rows name a specific substrate version or commit once the
  substrate specification stabilises, rather than a bare `main` reference?
- **Why it is a founder call:** A floating `main` under a frozen wire format is the moving-target
  problem freezing was meant to remove; when to pin is a version-discipline call tied to TRACT's own
  freeze, not an author's to time.
- **Recommendation:** **Pin to a tagged substrate release at TRACT's own v1 freeze**, holding the
  bare reference only until the substrate cuts that tag.

---

## §22 — Erasure
*Status: normative for the structural resolution; legal questions scoped*

*(Whether `MAX_REVIEW_BODY` should shrink, raised at §22.8.4, is **owned by §19** and listed as
decision 19.3.)*

### 22.1 Does destroying a crypto-shredding key satisfy a statutory erasure right? (§22.4.1, §22.8.1)
- **Question:** Does destroying a crypto-shredding key satisfy an erasure right, or only make the
  referenced data permanently inaccessible without deleting it?
- **Why it is a founder call:** A legal question with no reversal once a deployment relies on the
  answer; all three research passes returned nothing verified (§21.11), and it needs qualified counsel
  per jurisdiction, not more protocol text.
- **Recommendation:** **Take no position;** TRACT v0 defines no crypto-shredding mechanism. Whether it
  ever has a place is an open founder-and-counsel call (see 22.5).

### 22.2 Is joint controllership the right frame for a sealed order at two self-hosted nodes? (§22.5, §22.8.2)
- **Question:** Is joint controllership the correct legal frame for a sealed order held at two
  independently self-hosted nodes, and if so, how could its arrangement duties ever be discharged
  between two parties who transact once and have no other relationship?
- **Why it is a founder call:** A legal-characterisation question the protocol cannot answer in bytes,
  unresolved by the pass that studied the gateway's "facilitator" position; a deployment-shaping call
  for counsel, not an author.
- **Recommendation:** Unresolved; a question for qualified counsel per jurisdiction (§22.8.8).

### 22.3 Does a household-style exemption apply to a self-hosted seller? (§22.5, §22.8.3)
- **Question:** Where, if anywhere, does a personal/household-activity exemption apply to a
  keypair-identified self-hosted seller with no business registration, no employees, and no
  infrastructure beyond their own node?
- **Why it is a founder call:** Unresearched (not merely unsettled) and jurisdiction-specific; the
  boundary determines whether large parts of a regime even apply, and only counsel can place it.
- **Recommendation:** Unresolved; for qualified counsel per jurisdiction (§22.8.8).

### 22.4 Should a supersede/tombstone mechanism ever be given protocol-level teeth? (§22.4.2, §22.8.5)
- **Question:** Should a supersede/tombstone ever carry a propagation obligation on indexes and caches
  — protocol-level teeth — or is that permanently out of reach?
- **Why it is a founder call:** Enforcing propagation requires exactly the coordinating authority this
  design refuses to introduce anywhere else; blessing it would reverse a core no-authority stance.
- **Recommendation:** Leans **permanently out of reach;** keep retraction cooperative-only and
  disclosed as such, never presented as erasure enforced by construction.

### 22.5 Does crypto-shredding have any place in TRACT's model? (§22.4.1, §22.8.6)
- **Question:** Does crypto-shredding have any legitimate place in TRACT at all — given the public
  quadrant exists to be served and computed over in plaintext — or does it only ever apply as
  voluntary defence-in-depth on the sealed side, where both endpoints can already delete their copy?
- **Why it is a founder call:** Adopting it for the **public family** would require a §16 grammar
  change (a public ciphertext-blob production that does not exist in frozen v0) — a MAJOR bump — and
  the underlying erasure-equivalence question (22.1) is unresolved.
- **Recommendation:** Do **not** adopt it for the public family; if used at all, only as voluntary
  defence-in-depth on the sealed side, where the problem it would solve does not exist. Adoption for
  the public family is the conditional §16 change (see G-C6).

### 22.6 When trader-traceability duties collide with minimising public data, which bends? (§22.8.7, §11.4)
- **Question:** When trader-traceability duties that require identifying a seller (§11.4) collide with
  minimising what is public, which obligation should this document bend?
- **Why it is a founder call:** A direct privacy-vs-legal-transparency conflict with no reversal once
  a deployment chooses a side; named, not resolved, and beyond what an author may settle.
- **Recommendation:** Unresolved; the tension is named for the founder-decision list, not decided.

---

## Grammar changes needed (frozen §16 sign-off)

Every item below touches the **frozen** `16-wire-format.md` and therefore requires a **MAJOR
version bump** under `GOVERNANCE.md` and explicit founder sign-off before any author writes it.
None may be added silently. Items marked **CONDITIONAL** are needed only if the linked decision
resolves "yes".

| ID | Section | Change | Trigger |
|----|---------|--------|---------|
| **G1** | §1.3 | Add a production to carry the seller legal-disclosure block (legal/trading name, contact route, registration reference) in the PUBLIC family. §16 today has no slot — neither Offer (§16.5.2) nor ProductRecord (§16.5.1) carries these. Recommended shape: an optional disclosure map on **FeedHead** (one per seller identity) with an optional **per-Offer override** slot. Blocks §1.3 from becoming fully normative. | Decision 1.1 |
| **G4** | §2.3a | **ProductRecord localization slot** (already logged in §16 known-pending list): move display name/description (slots 1, 2) OUT of the content-addressed identity into a **BCP47 language-tag→text map**, so identity derives only from language-neutral fields. Required to make §2.3a real; frozen v0 does not yet implement it. | Decision 2.4 |
| **G5** | §3 / §16.5.2 | Add an optional **per-variant freshness/max-age field** to Availability so a fast-moving stock band or slot set can be aged independently of the Offer's single `published` (key 6). Today a stale Offer and a stale Availability are indistinguishable. | Decision 3.2 |
| **G6** | §5 / §16.5.2 | Add a **shared-currency constraint** so deposit+balance's two money fields and PriceTier `unit_price`s must carry one currency. | Decision 5.1 |
| **G7** | §5 / §16.5.2 | Add **canonical ordering (sorted) and duplicate-`min_qty` rejection** to the `[+ PriceTier]` array, matching ProductRecord attribute canonicalisation; define below-lowest-tier as unpriceable. | Decision 5.2 |
| **G8** | §5 / §16.6 | Add a **sealed usage-attestation object** for metered consumption reporting/dispute, and resolve how metered `Order.total` is set (estimate/cap) given key 4 is mandatory. | Decision 5.4 |
| **G9** | §5 / §16.6 | Add a **sealed quote-request object**, plus an optional **back-reference field on Offer** linking a private per-buyer Offer to the request that produced it. | Decision 5.5 |
| **G-tax** | §5 / §16.5.2 | Add a **tax treatment-category slot** to Offer (or Consideration) carrying a coded category string, **never a rate**, to place the §5.10 policy on the wire. | Decision 5.6 |
| **G10** | §5 / §18.3 | Define a **renewal path** (or fresh-Order-per-period rule) for recurring consideration, which the single-shot draft→terminal lifecycle does not currently describe. | Decision 5.3 |
| **G11** | §7 / §16.8 | Commit the **order-amendment shape**: a superseding-order **predecessor pointer** (recommended) vs an append-only op-log object. | Decision 7.1 |
| **G12** *(conditional on G11 = supersession)* | §7 / §16.5.x | Add a **per-line status** to OrderLine, OR (recommended) represent partial fulfilment via the amendment/split mechanism with no new per-line field. | Decision 7.2 |
| **G-C1** *(CONDITIONAL)* | §4.6 / §16.5.2 | Add an **Incoterm slot** on the ship variant. Add only if decision 4.2 resolves "yes". | Decision 4.2 |
| **G-C2** *(CONDITIONAL)* | §4.10 / §16 | Add a **licence-terms sub-object** on digital-grant. Add only if decision 4.3 resolves "yes". | Decision 4.3 |
| **G-C3** *(CONDITIONAL)* | §4.8 / §16 + §18.2 | Add a **first-class multi-fulfilment grouping construct**. Add only if decision 4.4 resolves "yes". | Decision 4.4 |
| **G13** | §9.4.3, §9.7 / §16.6 + §18.5 | Add a **sealed escrow state-transition object** (operator signature, from/to state, order address, evidence reference) **and an escrow ruling object** (its disposition, including a `split`'s per-party amounts). §16 today carries neither, so §9.5's "every ruling is published" and §18.5's "each step is signed" are intended guarantees not yet expressible on the wire. | Decision 9.1 |
| **G16** | §10.2c, §10.6 / §16.5 | Add a **public `ReviewResponse` object** — subject-key author, target review `content-address`, bounded body (`MAX_REVIEW_BODY`), `ts` — so a review's subject can respond in public attached to the same subject. `Review` cannot name another review, and no response production exists today. | Decision 10.1 |
| **G14** | §13.3 / §16.6 | Add a **sealed `Grant` object** for analytics disclosure (one `store`, per-store pseudonymous `subject`, enumerated `fields`, bounded lifetime, `subject`-key signature). Semantics are normative in §13.3; the byte encoding does not exist in frozen v0. Optional machine-readable **purpose field** (decision 13.6) is folded into this production if adopted. | Decisions 13.1, 13.6 |
| **G15** | §13.3 / §16.6 | Add a **sealed `GrantRevocation` object** naming the grant it withdraws. Meaning is normative (name the grant, cease reliance on receipt); the encoding awaits this production. | Decision 13.1 |
| **G17** | §19.4, §19.8 / §16.5.4 | Add a **machine-readable `DISPUTE_TIMEOUT` slot** to `EscrowScope`, so the §18.5 before-the-trade dispute-behaviour disclosure is machine-checkable rather than prose (field 9) or out of band. | Decision 19.1 |
| **G-C4** *(CONDITIONAL)* | §14.5 / §16 | Add a **machine-readable cold-contact-policy declaration** on the seller's feed, so a buyer's node knows before sending whether an unsolicited order will be looked at. No slot exists today. Add only if decision 14.3 resolves toward declaring it. | Decision 14.3 |
| **G-C5** *(CONDITIONAL)* | §12.4a, §12.6 / §16.5 | Make **storefront render bundles TRACT-native signed public objects** (today they are substrate content-addressed blobs with no TRACT production). Add only if decision 12.1 resolves toward a **canonical render** whose byte-for-byte comparison needs a fixed shape. | Decision 12.1 |
| **G-C6** *(CONDITIONAL)* | §22.4.1, §22.8.6 / §16.5 | Add a **public ciphertext-blob production** to support crypto-shredding of public-family objects (does not exist in frozen v0). Add only if decision 22.5 resolves toward adopting crypto-shredding for the public family — which §22 recommends against. | Decision 22.5 |

*(WRAP-side, not a §16 change: decision 18.3's `disputed-lost` custody state is a **WRAP profile
change**, not a frozen-§16 grammar change, and so carries no G-row here.)*

**Also pending confirmation (not a byte-shape change, but binds frozen behaviour):**

- **§2.2 value normalisation** — fixing Unicode **NFC + default casefold** for coded attribute
  *values* is not a new production, but it binds the frozen grammar's real convergence behaviour
  and the §15 vector corpus, and is effectively frozen once chosen. Confirm before writing.
- **§3.3 lead-day convention** — fixing lead days as **calendar days** reconciles the wire meaning
  against a reference implementation that currently documents working days. A normative behaviour
  decision, not a grammar change.

---

*Coverage: §1–§7 and §9–§22, extracted from the authored section files. §8 (Delivery) is authored
separately and appended last; §11 (Jurisdiction) already represented.*
