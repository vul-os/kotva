# TRACT Authoring Brief — shared instructions for all section authors

> **Read this before writing a line of any section.** It exists so ~24 parallel authors produce
> one coherent specification instead of 24 drifting ones. It is not itself a numbered section and
> ships no normative bytes. Where it summarizes another document, **that document governs**:
> `16-wire-format.md` governs wire bytes (frozen v0), the DMTAP substrate governs substrate
> behaviour, and `21-grounding.md` governs what the evidence does and does not support.

---

## 0. The one-paragraph orientation

TRACT is a protocol for commerce between self-sovereign identities — no marketplace operator, no
registrar, no token. It **stands on the DMTAP substrate** (identity, feeds/blobs, sync, infra
roles, wake) and adds only a commerce spine: catalogue, offer, cart, order, delivery, settlement,
trust, jurisdiction. It splits every object into two families: **public** (signed,
content-addressed, irrevocable — product records, offers, rate cards, reviews) and **sealed**
(encrypted, per-party, deletable — orders and everything identifying a person). There is exactly
**one** operator class, the **gateway** (storefront rendering and/or settlement+escrow), and
confining it is the structural claim of the whole document. Work-coordination (bid/assign,
dispatch, custody) belongs to **WRAP**, not TRACT. Tax is **facts at the edge**, never computation.

---

## 1. The non-negotiable design constraints (restated crisply)

Every section MUST obey all seven. They come from the 2026-07-23 WRAP/TRACT substrate-consolidation
work.

### C1 — Substrate adoption, à la carte, bytes not ideas
TRACT adopts the five DMTAP substrate capabilities under the substrate's **rule 2**: *if a section
implements a capability's function, it MUST speak that capability's spec (its bytes), never a
parallel one.* Concretely:
- **Identity** → `IDENTITY.md`: `IK` keypair is the identity, `DeviceCert` subkeys, `name→key` over
  DNS+KT, 8-word key-name floor. Seller/buyer/courier/distributor/gateway are all just `IK`s.
- **Feeds & Blobs** → `FEEDS.md`: signed append-only author feeds (`FeedHead`/`FeedEntry`/
  `PubAnnounce`) + plaintext content-addressed blobs (`PubManifest`). Catalogues, offers, rate
  cards, capacity records, reviews are public feed/blob objects.
- **Sync** → `SYNC.md`: the signed CRDT op algebra (`SyncOp`, HLC total order, the 9 op kinds).
  Carts across a buyer's devices and inventory across a seller's replicas use **this**, not a
  bespoke CRDT.
- **Infra Roles** → `ROLES.md`: announce/resolve (`LocationRecord`), signaling, circuit relay,
  short-TTL content-blind mailbox, cache/pin. Reachability for NAT'd stores/buyers.
- **Wake** → `ROLES.md §8`: content-free sender-blind push (`PushSubscription`/`WakePing`) to wake
  a sleeping seller node when an order arrives.

TRACT introduces **NO** new cryptography, hash construction, signature framing, encoding, CRDT, or
transport. Reference the substrate by full link, never re-specify it:
`github.com/vul-os/dmtap/blob/main/substrate/{IDENTITY,FEEDS,SYNC,ROLES,README}.md`.

### C2 — The WRAP seam: work-coordination is not TRACT's
Request→bid→assign, dispatch, custody handoffs, and both auction directions live in **WRAP**
(`github.com/vul-os/wrap`), on the same substrate and same `IK` identity. TRACT **defines no
bidding, dispatch, or custody object of its own** and **references** WRAP for:
- **on-demand delivery dispatch** — a leg with no standing rate card (same-hour courier, gig
  carriage) is a WRAP `WorkOrder → Bid → Assignment`, issuer-assigns (08-delivery §8.9).
- **the custody-handoff lifecycle** — `created → accepted → in-custody → handed-off → delivered`
  lives **once** in WRAP's `Progress`/`Attestation`, referenced from TRACT §8.4 and §18.4, written
  a second time nowhere (08-delivery §8.9).
- **competitive procurement / RFQ** — buyer posts a need, many providers bid, buyer assigns; WRAP
  unchanged (05-consideration §5.9a).
- **forward auctions** — an **optional profile over WRAP's `Bid`**, seller-assigns at close
  (05-consideration §5.9a).

Rule of thumb: if a concept is "who does the work and who holds the goods," it is WRAP. If it is
"what is offered and what is owed," it is TRACT.

### C3 — Tax is edge policy; the protocol carries facts only
The protocol carries **facts**: the four jurisdictional anchors, a tax-treatment **CATEGORY** (not
a **RATE**), and the responsible parties. Computation, collection, remittance, rate lookup,
threshold tracking, invoicing = **seller's or gateway's** job, **OUT OF SCOPE**. Legal/case-law
analysis is **evidence** and belongs in `21-grounding.md §21.11`, never in a normative section.
See `11-jurisdiction.md §11.2a/§11.2b`.

### C4 — Agnostic via existing standards, not TRACT vocabulary
- **currency** = ISO 4217, minor-units integers (`money` type). Cross-currency arithmetic is
  **refused**, never coerced; one currency per order.
- **language** = UTF-8 + BCP 47 (RFC 5646) tags, with multi-localization. Content-addressed product
  **identity MUST be language-neutral** (codes/identifiers); display text is presentation, excluded
  from the address (02-catalogue §2.3a).
- **region** = ISO 3166 (alpha-2).
- **schedule/recurrence** = RFC 5545 (RRULE for recurring pricing and availability).
- **product vocabulary** = schema.org (`Product`, `Offer`, `ProductGroup`, `hasVariant`,
  `variesBy`).
- **identifiers** = GS1 (GTIN, MPN) as **claims only** — advisory, unverified, squattable.

### C5 — The frozen grammar is the authority
`16-wire-format.md` is normative and **frozen at v0**. It is the authority on wire bytes. Every
section MUST align to its CDDL and MUST NOT contradict it. If a section needs a grammar slot 16
does not have, **RECORD it as a required §16 change (an open item)** — do not invent conflicting
bytes. A change to any §16 shape is a MAJOR version bump, not a correction. (Known pending §16
changes are already logged: a localization slot on `ProductRecord`, §2.3a; a tax-treatment-category
slot, §5.10; a shared-currency rule inside multi-`money` `Consideration` values, §5.4/§5.13.)

### C6 — Honesty: do not paper over the contradicting evidence
`21-grounding.md` records evidence that **contradicts** parts of the design. Where a section rests
on an unproven or contradicted claim, it MUST say so and cite the specific §21.x. The two weakest
load-bearing claims — **index/discovery re-centralization** and **cross-publisher (near-duplicate)
product identity** — MUST stay marked as weakest, not defended. No section may cite §21 as
*support* for logistics (§8), analytics (§13), trust (§10), or tax/legal (§11) — those areas
returned **nothing verified** across passes (§21.1, §21.10, §21.11).

### C7 — RFC 2119 discipline: normative only where settled
Author MUST/SHOULD/MAY **only where the design is settled**. Where a design decision is **open**
(the drafting-status banner exists for a reason), DO NOT invent a founder call. Write the settled
parts as normative and **collect each open decision explicitly for the founder-decision list**
(§5 below). Any subsection dated or attributed **2026-07-23** is founder-aligned and MUST be
preserved verbatim in intent.

---

## 2. Terminology & object list — extracted from `16-wire-format.md` (use these exact names)

All authors MUST use these type names verbatim. Two disjoint families; a decoder is always in
exactly one mode, made non-confusable by distinct domain-separation tags (no `sealed: true` flag).

### 2.1 Shared primitives (§16.3)
| Name | Definition / purpose |
|---|---|
| `content-address` | `bytes` — multihash-prefixed digest (substrate §18.1.5). The address of a public object. |
| `identity-key` | `bytes` — public half of an `IK`; also names a courier, distributor, gateway. |
| `ts` | `int` — milliseconds since Unix epoch. Display/ordering only; authoritative order comes from a feed's sequence. |
| `country` | `tstr .size 2` — ISO 3166-1 alpha-2. |
| `currency` | `tstr .size 3` — ISO 4217. |
| `money` | `{1 => int (minor_units), 2 => currency}` — **never a float**; cross-currency arithmetic refused. |

### 2.2 Public objects (§16.5) — signed, content-addressed, irrevocable, NO personal data
| Type | Purpose |
|---|---|
| `ProductRecord` | What a thing **is** — name, description, attributes, identity ladder, optional group, optional bundle components. Belongs to nobody; converges by content address. |
| `Attribute` | `{1 => key (casefolded), 2 => value}` — sorted, deduplicated inside a record. |
| `IdentityRung` | One rung of the product-identity ladder; union of the three below, weakest first. |
| `ContentAddressRung` | `{0 => content-address}` — the floor, zero authority. |
| `ClaimedExternalRung` | `{1 => scheme, 2 => value}` — GTIN/MPN **claim, UNVERIFIED**; an index MUST treat as advisory join key, never authority. |
| `ManufacturerSignedRung` | `{2 => identity-key}` — the brand's own key; authority = the brand. |
| `Offer` | **One seller's** claim to supply — the four axes + `sell_to` territories + `published`. |
| `Item` | Axis 1: `product` / `variant-of-group` / `service` / `right/licence` / `capacity`. |
| `Availability` | Axis 2: `StockSignal` / time-slots (RFC 5545) / capacity-per-interval / unlimited / made-to-order(lead days). |
| `StockSignal` | `exact(n)` / `in-stock` / `low` / `out-of-stock` — a publishable band, since exact stock is sensitive. |
| `Fulfilment` | Axis 3: `ship` / `collect` / `digital-grant` / `perform-at-place` / `perform-remote` / `access-grant` / `return-required`. **The only object that knows where a supply happens** — §11.2 derives place of supply from it. |
| `PlaceRef` | `{1 => country, 2 => locality}` — country + coarse locality, **never a street address**. |
| `Consideration` | Axis 4: `fixed` / `tiered` / `recurring`(RRULE) / `metered` / `deposit+balance` / `quote-required`. |
| `PriceTier` | `{1 => min_qty, 2 => unit_price}`. |
| `RateCard` | A carrier's signed public rate object — zones, weight brackets, dim_divisor, surcharge, served countries, excluded categories, `published`. Price is **computed locally**, not quoted. |
| `Zone` | `{1 => id, 2 => [+ WeightBracket], 3 => transit_days}`. |
| `WeightBracket` | `{1 => max_grams, 2 => price}`. |
| `CapacityRecord` | A distributor's published consolidation offer — country, locality, storage/day, handling fee, slots, excluded categories. |
| `EscrowScope` | What an operator can lawfully serve — buyer/seller/**supply** countries, currencies, rail classes, max order value, excluded categories, authorities claimed (prose). |
| `RailClass` | `0` = custodial-reversible, `1` = non-custodial-final. Part of the type because it changes recourse. |
| `Review` | The **one** public object signed by a natural person — subject, per-subject pseudonymous subkey author, score 0..5, body, optional attestation, ts. Bounded by §10.4. |
| `Subject` | What a review is about: `product` / `seller` / `distributor` / `courier`. |
| `PurchaseAttestation` | Signed proof the author transacted — attestor (seller/escrow), issuer, **sealed order's address only**, ts. |

### 2.3 Sealed objects (§16.6) — encrypted, per-party, deletable, held at the two endpoints
| Type | Purpose |
|---|---|
| `Order` | **One seller's** sealed order — buyer, seller, that seller's lines only, total (one `money`), state, placed, `Anchors`, `Responsible`. Buyer name/address/contact live here and have no public production at all. A cross-seller order is not expressible. |
| `Anchors` | The four jurisdictional anchors: `seller_establishment`, `buyer_residence`, `place_of_supply` (derived from Fulfilment), optional `delivery_destination`. |
| `Responsible` | `seller_of_record`, optional `facilitator` (gateway iff it settled), optional `importer_of_record`, optional `Representative`. |
| `Representative` | `{1 => region covered, 2 => identity-key}` — in-region responsible person where a regime requires one. |
| `OrderLine` | `{1 => offer address, 2 => seller, 3 => quantity}`. |
| `OrderState` | `0..8`: draft / placed / accepted / declined / countered / fulfilling / delivered / closed / cancelled (§18). |
| `PaymentAttestation` | payer, payee, order, amount, `RailClass`, opaque external settlement **reference** (never funds, never card data), ts. |

### 2.4 Terms of art (from `00-overview.md §0.9`, use with these meanings)
**product record**, **offer**, **the four axes** (Item/Availability/Fulfilment/Consideration),
**index** (derived, rebuildable, **never** authoritative; the word *marketplace* is not used for
anything TRACT defines), **gateway** (one sense only: storefront and/or settlement+escrow —
never an index, courier, or relay, never holds identity keys), **consignment**, **leg**,
**rate card**, **place of supply**, **rail**, **purchase attestation**, **personal data**.

### 2.5 Substrate object names authors will reference (do not redefine)
From `IDENTITY.md`: `IK`, `DeviceCert`, `Identity`, key-name. From `FEEDS.md`: `PubAnnounce`,
`PubManifest`, `FeedHead`, `FeedEntry`. From `SYNC.md`: `SyncOp`, `Hlc`, the 9 op kinds. From
`ROLES.md`: `LocationRecord`, `PushSubscription`, `WakePing`.

---

## 3. Cross-reference map — who owns what (reference, do not duplicate)

Each concept has exactly **one** home. Every other section that touches it **references** the home.

| Concept | Home | Everyone else |
|---|---|---|
| Identity, keys, `IK`, `DeviceCert`, key-name, `name→key` | substrate `IDENTITY.md` (+ TRACT §1) | reference; never re-specify keys |
| Public feeds/blobs, content addressing, signing, anti-rollback | substrate `FEEDS.md` | catalogues/offers/rate-cards/reviews are instances |
| CRDT sync, merge, HLC, oversell prevention | substrate `SYNC.md` (+ TRACT §6 for cart) | cart/inventory reference the op algebra |
| Reachability, relay, mailbox, cache/pin, wake | substrate `ROLES.md` (+ TRACT §1.5) | reference; the middle holds no durable data |
| **Wire bytes / all CDDL** | **§16 (frozen v0)** | every section aligns; grammar gaps → §16 open items |
| Product identity, canonicalisation, ladder, variants, index rule | §2 catalogue | §5/§8/§10 reference the record & offer |
| The four offer axes, definitionally | §2.4.3 | §3 (Availability), §4 (Fulfilment), §5 (Consideration) each own one axis's detail |
| Availability / stock / slots / made-to-order | §3 | §6 cart reads live availability |
| Fulfilment modes; **place-of-supply derivation** | §4 | §11 derives the tax anchor from §4; §8 delivery consumes ship destinations |
| Money, pricing, tax **category** (not rate), currency | §5 consideration + §16 `money` | §11 names the anchors; §9 settles; §16 is the byte authority |
| **Custody handoff lifecycle** | **WRAP** (`Progress`/`Attestation`), referenced from §8.4 + §18.4 | never written twice in TRACT |
| On-demand dispatch, bidding, auctions | **WRAP** (`WorkOrder→Bid→Assignment`), referenced from §8.9 + §5.9a | TRACT defines no bid/dispatch object |
| Rate cards, legs, consolidation, local route computation | §8 delivery | §11 delivery_destination from the shipping leg |
| Payment seam, rail classes, escrow, gateway money role | §9 settlement | §11 facilitator; §18.5 escrow state machine |
| Reviews, purchase attestation, local ranking, no global score | §10 trust | §16 `Review`/`PurchaseAttestation` bytes |
| **Tax facts, four anchors, responsible parties, scope** | §11 jurisdiction | §5 category, §4 place-of-supply, §9 escrow trigger; **evidence** → §21.11 |
| Gateway storefront, domains, honest trust limits | §12 gateway | §0.4.2 defines the operator class |
| State machines (offer/order/consignment/escrow) | §18 | §7 order prose checks against §18.3 |
| Errors, registries | §17 | every section uses `ERR_TRACT_*`; substrate codes stay in substrate |
| Parameters, timeouts, limits | §19 | §18/§8 reference `PLACE_TIMEOUT`, `CONFIRM_TIMEOUT`, `RATE_CARD_MAX_AGE`, etc. |
| **Grounding / contradicting evidence** | §21 | every section cites it for honesty; **no section cites it as support** |
| Erasure-rights conflict | §22 (+ §0.5.1, §11.5) | no personal data in the public quadrant, ever |

**Custody appears in three places, one owner:** the lifecycle bytes live in **WRAP**; §18.4 states
the *state machine* referencing WRAP; §8.4 states the *one property* (receiver signs the handoff)
and references both. Do not restate the transitions a fourth time.

---

## 4. Settled 2026-07-23 decisions — preserve verbatim in intent

These are founder-aligned. Do not reopen, weaken, or contradict them. Where a section's normative
text touches one, it must encode this intent.

1. **02-catalogue §2.3a — language/localization.** Product content-addressed **identity MUST derive
   only from language-neutral fields** (identifiers + coded attributes), **never from localized
   display text**. Display `name`/`description` are UTF-8 + BCP 47, may carry several localizations
   as a `language-tag → text` map, and are presentation excluded from (or canonicalized out of) the
   address. This **requires a §16 grammar change** to `ProductRecord` (a localization slot; move
   display text out of the addressed identity) — logged as an open §16 item, not a silent rewrite.

2. **05-consideration §5.9a — auctions reuse WRAP.** One bid→assign engine, many profiles, the
   issuer/seller always the assigner. **Competitive procurement / reverse auction** = WRAP
   unchanged (buyer issues, buyer assigns). **Forward auction** = an **optional profile over WRAP's
   `Bid`**, seller-assigns at close; residual disclosed (a partitioned seller cannot close;
   sealed-vs-open visibility chosen per auction; reintroduces a bounded close-time race). TRACT
   defines no competing auction object; a `Consideration` never carries bidding state.

3. **05-consideration §5.10 — tax categories here, rates never.** The offer carries a **treatment
   category** (standard/zero/reduced/exempt per the applicable taxonomy) plus the §11 anchors; it
   carries **no rate**. Rate lookup keyed by (category, place of supply, date) is left to whatever
   source an implementation trusts — never a table in this document. (Note: neither `Consideration`
   nor `Offer` has the category slot yet — a logged §16 open item.)

4. **08-delivery §8.9 — the WRAP seam.** TRACT owns the **rate-card (offer-pull)** delivery mode.
   The **on-demand** mode is WRAP's (`WorkOrder→Bid→Assignment`, issuer-assigns). The
   **custody-handoff lifecycle** is shared and specified **once in WRAP** (`Progress`+`Attestation`),
   referenced from §8.4 and §18.4 — never rewritten in TRACT. Same substrate, same `IK`, so the
   courier's key/signatures/reputation are shared, not bridged.

5. **11-jurisdiction §11.2a — evidence belongs in §21, not §11.** The case-law analysis (US
   marketplace-facilitator statutes, EU 282/2011 Art 5b, Explanatory Notes) is **evidence, not
   protocol text**, and lives in §21.11. §11 keeps only the wire-shaping conclusions and defers:
   *there is a marketplace — the available argument is "no facilitator," and only where no seller
   contracted with the gateway*; *escrow is the trigger* (in TX/NY, enough on its own);
   *render-only is not a universal safe posture*; *"the contract is between two keypairs" does not
   defeat a test that measures economic influence*; *none of this is settled law*. TRACT is shaped
   to be **defensible, not compliant by construction**, and must not be read as the latter.

6. **11-jurisdiction §11.2b — facts on the wire, computation at the edge.** TRACT carries the facts
   a tax authority asks for and **never computes, collects, or remits** tax. The entire tax surface
   is: the four anchors, a treatment **category** (never a rate), and the responsible parties an
   order names. Everything past it is out of scope, done as **local policy over these fields**.

---

## 5. RFC 2119 discipline & the founder-decision list

**Write normative text (MUST/SHOULD/MAY) only where settled.** Where open, write the settled parts
normatively and add the open decision to the list below — do not invent a founder call. Keep every
drafting-status banner until the section is genuinely normative.

**Open decisions to collect for the founder** (already surfaced across the read set; add any new
ones you find, cited to the section that raises them):

- **§16.8** — (a) does `Offer` carry its own signature or inherit authenticity from feed position;
  (b) does empty `sell_to` mean "unrestricted" or "malformed" (leaning malformed);
  (c) is `Anchors.place_of_supply` recomputable or authoritative-as-recorded;
  (d) extension-key policy for the axis unions (tolerant-to-store / strict-to-act).
- **§2.5** — near-duplicate product resolution (recommend one heuristic or leave to differ);
  bootstrapping manufacturer signatures; the §16 localization-slot grammar change.
- **§5.13** — shared-currency rule inside multi-`money` `Consideration` values; `PriceTier`
  canonical ordering + duplicate-`min_qty` + below-lowest-tier coverage; `recurring` renewal vs
  §18.3's single-shot lifecycle; `metered` usage-attestation object + mandatory `total` conflict;
  quote-request wire shape + link back to the private per-buyer `Offer`; where the tax category is
  carried; the exact forward-auction profile shape over WRAP's `Bid`.
- **§8.8** — peer-courier rate-card shape; per-object `RATE_CARD_MAX_AGE`; reproducibility of
  consolidation route choice across implementations.
- **§18.7** — `CONFIRM_TIMEOUT` protocol-parameter vs offer-level term (leaning protocol floor +
  offer-level extension upward); does a dispute pause `CONFIRM_TIMEOUT`; does `lost` need a distinct
  `disputed-lost`.
- **§6 (from §21.10)** — name the intra-replica serialization requirement; state the **non-Byzantine
  failure model as a hard boundary** (bounded counters protect a seller from their own concurrency,
  not a buyer from a dishonest seller); dead-replica right reclamation as an explicit fallible
  decision; who chooses the recipient when transfer names one at N≥3.

---

## 6. Honest-gaps list from `21-grounding.md` — MUST NOT be papered over

Cite the specific §21.x wherever a section leans on one of these. Nothing in §21 is normative;
everything in it should change what is normative.

1. **Cross-publisher product identity is unproven (§21.2).** No deployed system achieves it without
   a licensed registry; the permissionless-clustering candidate was refuted 0-3. Content addressing
   is a sound **mechanism** carrying an **unproven solution**. §2's **canonicalisation** is the
   load-bearing part, not the hashing; §2.5 keeps near-duplicate resolution **open**. Do not say a
   global product view "falls out of" content addressing. **(One of the two weakest claims — keep
   marked.)**

2. **Discovery re-centralizes first (§21.3, §21.4, §0.4.1 gap note).** "Any node MAY build an index"
   does not stop one index becoming the de facto content-policy gatekeeper. OpenBazaar's default
   crawler became one; ONDC (the largest live network) avoided it only with a central
   approval-gating registry. No rule here prevents it; competing indexers with completeness/
   censorship proofs have **no deployed precedent**. **(The other weakest claim — keep marked.)**

3. **Liveness bounds catalogues, not orders (§21.5, §0.7).** A seller whose node is offline is
   **invisible** — they can *receive orders* via mailbox+wake but cannot *serve a catalogue*.
   OpenBazaar: ~22-day median listing lifetime, catalogues vanishing on node departure. Docs MUST
   NOT present an intermittently-online seller as fully functional. Third-party pinning becomes a
   **requirement**, and unpaid replication is exactly what OpenBazaar didn't get; whether pinning
   needs an incentive (and whether that creates another operator) is open.

4. **Reputation failure modes survive (§21.6).** Purchase-attested reviews are stronger than
   OpenBazaar's, but **self-dealing** (a seller transacting with itself yields genuine attestations)
   and **opt-in escrow declined by exactly the bad actors it targets** both persist. §10 and §9.6
   MUST state these as **measured outcomes**, not hypotheticals. The achievable sybil-cost floor on
   a signed-feed substrate is unresearched (§21.8).

5. **Non-custodial escrow deadlocks (§18.5, §21).** A dispute where neither party moves has no good
   answer on a non-custodial rail — a timeout defaulting to one party (a policy choice, not a
   neutral mechanism) or indefinite lock. The choice MUST be **disclosed before the trade**. Do not
   pretend a third option exists.

6. **The gateway is structurally permanent (§0.4.3).** Unlike DMTAP's self-extinguishing operator
   class, TRACT's storefront (browsers are permanent, keyless shoppers trust the render) and escrow
   (holding money for strangers is licensed; physical custody can't be trustless) do not decay.
   TRACT is **structurally less pure than DMTAP** — say so; what is preserved is that the class is
   one, permissionless, competing, per-order, replaceable, never holding identity keys.

7. **Whole research areas are unevidenced (§21.1, §21.10, §21.11).** Logistics/consolidation (§8),
   privacy analytics/fraud (§13), and trust/dispute/tax/legal (§9.6, §10, §11) returned **nothing
   verified** across passes. These sections are design reasoning checked for internal consistency
   only. **No section may cite §21 as support for them.** Absence of evidence ≠ evidence of absence.

8. **Tax/legal is assertion, textually-strong-but-untested (§21.11).** No court has applied any of
   it to a permissionless no-operator protocol. §11 must argue **"no facilitator"** (not "no
   marketplace" — the term explicitly enumerates a catalog and a sales app, so TRACT's feed and cart
   client are within it); **escrow is the trigger**; **render-only is a two-states-of-fifty holding,
   not safe**; and the **EU VAT rule (Art 5b / Art 14a ≤ €150) anticipates and rejects "the contract
   is between two keypairs"** by testing economic reality. GDPR Art 17 erasure vs immutable objects
   is the **most likely hard blocker** and survived three passes unanswered.

9. **No personal data in the public quadrant, ever (§0.5.1, §16.4, §11.5).** Published objects are
   irrevocable; erasure can't be satisfied against them. The prohibition is enforced by the §16
   grammar (no street-address production in the public family), not by reviewer discipline. Reviews
   are the single bounded exception (§10.4): `body` is free text a person could type an address
   into — the grammar can't stop it, §10.4 and client requirements must.

10. **Nostr's own caution (§21.7).** NIP-15 marketplace spec is marked in-repo "unrecommended: too
    complicated." TRACT's signed-transition order model (§7) is a deliberate departure from NIP-15's
    merchant-as-sole-authority flow — the departure is the point — but the complexity caution is
    real; do not ignore it.

---

## 7. Working rules for authors (quick checklist)

- Reference the substrate by full GitHub link; never restate its bytes or behaviour.
- Reference WRAP for anything bid/assign/dispatch/custody; define none of it in TRACT.
- Align every byte to §16; a needed slot §16 lacks is an **open §16 item**, not new bytes.
- Money is minor-units integers; refuse cross-currency arithmetic; one currency per order.
- Product identity is language-neutral; display text is localized presentation.
- Public = signed/irrevocable/no-personal-data; sealed = per-party/deletable. Never blur the seam.
- Tax = facts (four anchors + category + responsible parties); computation is out of scope; legal
  reasoning goes to §21.
- Every state transition names its signer, its evidence (public or sealed), and its timeout →
  destination (never "expires" alone).
- Write MUST/SHOULD/MAY only where settled; log every open decision for the founder.
- Cite §21.x wherever you lean on an unproven/contradicted claim; keep the two weakest claims
  (product identity, discovery re-centralization) marked as weakest.
- Preserve the six 2026-07-23 founder decisions verbatim in intent.
