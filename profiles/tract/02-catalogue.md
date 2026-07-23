# 2. Catalogue

> **Drafting status — partially normative.** The object shapes (`ProductRecord`, `Offer`), the
> identity ladder, variants and product groups, bundles and kits, the *structural* canonicalisation
> rules, and the index rule are now **normative** and aligned to the frozen v0 grammar
> (`16-wire-format.md §16.5.1`/`§16.5.2`). The RFC 2119 keywords MUST, MUST NOT, SHOULD, SHOULD NOT
> and MAY are used with their BCP 14 meaning only where the design is settled.
>
> **What is still scoped, not normative.** Three things below carry no MUSTs and say why:
> (a) §2.3a's language-neutral identity is a **resolved decision that the frozen v0 bytes do not yet
> implement** — it depends on a pending §16 localization-slot change (a MAJOR version bump), so
> until that change lands the addressed record still includes display text; (b) the concrete
> Unicode normalisation applied to coded attribute *values* is unspecified; (c) near-duplicate
> resolution and manufacturer-signature bootstrapping remain open (§2.5). The two weakest
> load-bearing claims — cross-publisher product identity (§2.2a, §21.2) and index re-centralisation
> (§2.6, §21.3/§21.9) — remain **marked as weakest and are not defended**.

## 2.1 Scope

Product records, offers, the product-identity ladder, variants, bundles, and the rules that keep
indexes from becoming authorities. Wire bytes are not defined here: every shape this section names
is encoded exactly as `16-wire-format.md` freezes it, and this section MUST NOT contradict that
grammar. The DMTAP substrate governs the byte machinery a public catalogue object stands on —
content-addressed plaintext blobs and signed append-only feeds — and is referenced, never
re-specified: `github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md` and
`github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md`.

## 2.2 The split, and why it is mechanical rather than conventional

A **product record** describes what a thing is; an **offer** is one seller's claim to supply it.
Because the substrate content-addresses public blobs over plaintext, two sellers publishing the
same record converge on the same address by construction, and the swarm stores it once. The
global product view is therefore an emergent consequence of hashing, not a registry.

The split is normative, not stylistic. A `ProductRecord` (§16.5.1) and an `Offer` (§16.5.2) are
**separate objects**: a record belongs to nobody and carries no seller, no price, and no
availability; an offer names its seller (by feed position) and references a record. An
implementation MUST NOT fold price, stock, or seller identity into a `ProductRecord`, and MUST NOT
treat an `Offer` as a description that could be shared by convergence — two sellers supplying the
identical product publish **one** shared record and **two** distinct offers. The whole-cart and
per-seller order machinery that consumes these objects lives in §6 and §7 and is referenced there.

## 2.2a How strong the convergence claim actually is (§21.2)

Weaker than §2.2 sounds, and §21.9 obliges this section to say so rather than let a reader infer
otherwise.

Convergence is trivially true for identical bytes and says nothing about the real case: two shops
describing the same shoe. A July 2026 literature pass found **no deployed system achieving
cross-publisher product identity without a licensed registry**, and the one candidate for
permissionless crawl-derived resolution was refuted 0-3 under adversarial verification. The two
models that exist in the field are the only two: a permissioned monopoly namespace (GS1 GTIN —
licensed rather than sold, fee-bearing, issued through national member organisations) and a purely
nominal string with no issuer or uniqueness guarantee (schema.org `productGroupID`). Nothing in
between is deployed.

So the content address is a sound **mechanism** carrying an **unproven** claim. The canonicalisation
rules of §2.3 — not the hashing — are the load-bearing part of this section, and §2.5 keeps
near-duplicate resolution listed as open rather than implied-solved.

## 2.3 The object shapes (normative)

The bytes are frozen in `16-wire-format.md §16.5.1` and `§16.5.2`; this subsection states the
normative rules that govern how they are populated. It does not restate the CDDL.

**`ProductRecord` (§16.5.1).** A record MUST be encoded as deterministic CBOR (§16.2) and MUST
carry: a `name` and `description` (slots 1–2), an `attributes` array (slot 3), and an
`identity` ladder (slot 4); it MAY carry a `group` reference (slot 5, §2.3d) and a `components`
list (slot 6, §2.3e). A `ProductRecord` is a **public** object and therefore MUST NOT carry any
personal data — no name of a person, address, or contact detail — a prohibition the §16.4 grammar
enforces structurally rather than by reviewer discipline. A record belongs to nobody: it names no
seller and its authenticity comes from convergence, not from a signature over it.

**`Offer` (§16.5.2).** An offer MUST carry all four axes — `Item`, `Availability`, `Fulfilment`,
`Consideration` (slots 1–4) — plus `sell_to` territories (slot 5) and a `published` timestamp
(slot 6). The offer's `Item` MUST reference the supplied product by content address (a plain
product, or a `group`+`variant` pair for a variant, §2.3d). The **detailed semantics of each axis
are owned elsewhere and referenced, not duplicated here**: availability/stock/slots in §3,
fulfilment modes and place-of-supply derivation in §4, pricing and tax category in §5. Whether an
`Offer` carries its own signature or inherits authenticity from its feed position is an open §16
question (§16.8) and is not resolved by this section.

### 2.3a Language and localization — identity is language-neutral, display is localized

TRACT is language-agnostic by construction, and the rule that makes it so also protects §2.2's
convergence. Human-facing text (a product `name`, a `description`) is UTF-8 — CBOR text strings
already are — and carries a **BCP 47** (RFC 5646) language tag; a record MAY carry **several**
localizations of the same field, as a `language-tag → text` map rather than a single string. A
storefront renders the buyer's best-match locale; nothing forces one language into the wire.

The load-bearing half is the split this forces, and it is the same identity-vs-presentation split the
whole section already leans on:

> **A product's content-addressed *identity* MUST derive only from language-neutral fields** —
> identifiers (the §2.3 ladder, GTIN/MPN claims) and *coded* attributes (keys and enumerated values,
> not free prose) — **never from localized display text.** Otherwise the same shoe listed in French
> and in English produces two different content addresses, and §2.2's convergence — already weak
> (§2.2a) — breaks entirely on the most ordinary multilingual case.

So display `name`/`description` are **presentation**, localized and excluded from (or canonicalized
out of) the address; the *identity* is the codes underneath. This is stated here because §16.5.1
currently addresses a bare `tstr` `name`/`description` **inside** the `ProductRecord`; the frozen v0
grammar (§16) must take a localization slot and move display text out of the addressed identity, and
this subsection is the recorded resolution that grammar change implements, not a silent rewrite of
frozen bytes.

### 2.3b Canonicalisation — the load-bearing part

Convergence is only useful to the extent independent publishers can produce identical bytes, so the
normalisation applied before addressing is the hard part of the section, not the hashing (§2.2a,
§21.2). Two layers, one settled today and one pending a grammar change.

**Settled and normative now** (these are already carried by the frozen v0 grammar, so an
implementation MUST apply them):

- The record MUST be encoded as **deterministic CBOR** (RFC 8949 §4.2, per §16.2). No non-canonical
  encoding of the same logical record is permitted, because two encodings would content-address
  differently.
- In `attributes` (slot 3), attribute **keys MUST be casefolded**, and the array MUST be **sorted
  and deduplicated**. Two records asserting the same coded attributes in a different order, casing,
  or with duplicates MUST reduce to the same bytes.
- In `identity` (slot 4), the rungs MUST be ordered **weakest-first** — `ContentAddressRung`, then
  `ClaimedExternalRung`, then `ManufacturerSignedRung` (§2.3c).

**Resolved but not yet implemented — PROVISIONAL, pending a §16 change.** §2.3a settles that the
addressed *identity* MUST derive only from language-neutral fields and never from localized display
text. **The frozen v0 grammar does not yet do this:** slots 1 and 2 (`name`, `description`) are
bare `tstr` inside the addressed `ProductRecord`, so today two localizations of the same product
still produce two addresses. Closing this requires moving display text into a localization slot
excluded from the address — a MAJOR version bump logged as a §16 open item (§16 known-pending
changes; §2.5). Until it lands, this section MUST NOT claim language-neutral convergence as an
achieved property; it is the recorded target, not the current behaviour.

**Underspecified — the concrete value normalisation.** The grammar casefolds attribute *keys* but
says nothing about the Unicode normalisation of enumerated *values*. Real convergence across
independent publishers needs a fixed value-normalisation (a Unicode normalisation form plus a
defined casefolding), and none is specified. This is recorded as open (§2.5); this section does not
invent one.

Even with all of the above, exact-byte convergence is an exact-match floor. **Near-duplicate**
records — the same physical product described slightly differently — do not converge, and merging
them is an index-side heuristic that §2.5 keeps open rather than implied-solved (§21.2).

### 2.3c The identity ladder (normative)

`ProductRecord.identity` (slot 4) is a ladder of rungs ordered weakest-first (§2.3b). Each rung
carries a strictly different amount of authority, and an index MUST treat them accordingly:

1. **`ContentAddressRung` — the floor, zero authority.** Every record has a content address by
   construction; this rung asserts nothing beyond "these bytes." It cannot be squatted because it
   *is* the bytes.
2. **`ClaimedExternalRung` — a claim and nothing more.** A GTIN or MPN carried here is
   **UNVERIFIED**: anyone MAY assert any external identifier for any record. An index MUST treat it
   as an **advisory join key only** and MUST NOT treat it as authority (§16.5.1, §2.6). Squatting is
   **expected rather than prevented** — GS1 issuance is gated and fee-bearing, so depending on it as
   authority would import exactly the centralization point and cost barrier TRACT refuses to require.
3. **`ManufacturerSignedRung` — authority is the brand.** This rung names the brand's own `IK`
   (substrate `IDENTITY.md`). Its weight is exactly the weight a verifier assigns that key: a
   consumer of the record verifies the manufacturer's signature against the named `IK` and decides
   for itself whether that brand identity is one it trusts. The protocol confers no authority here
   beyond a checkable signature by a named key; there is no registry that blesses the key.

The ladder is additive: a record MAY carry any subset, and higher rungs strengthen — never
replace — the floor. Bootstrapping the top rung when most brands will not publish a key early is an
adoption problem left open (§2.5), not a protocol gap.

### 2.3d Variants and product groups (normative)

A **product group** and its **variants** are both `ProductRecord`s, linked through slot 5:

- A group is a `ProductRecord` carrying the **shared, language-neutral** attributes common to every
  variant (§2.3a).
- A variant is a `ProductRecord` whose `group` reference (slot 5) is the content address of its
  group, and whose **distinguishing** attributes — size, colour, material — are carried as **coded
  `Attribute`s** (keys and enumerated values, not free prose), so they are part of the variant's
  language-neutral identity and converge across publishers exactly as any other coded attribute
  does.
- An `Offer` supplying a specific variant uses the `Item` variant-of-group form (`{1 => group,
  2 => variant}`, §16.5.2), naming both the group and the specific variant by content address.

The schema.org `variesBy` axis enumeration is **recovered** as the set of `Attribute` keys that
differ between sibling variants under one group; it is not carried as a separate first-class field
in the frozen v0 grammar. Whether an explicit `variesBy`/`ProductGroup` production is worth adding
is recorded as a possible §16 change (§2.5) — the attribute-based reading above needs **no** grammar
change and is the normative interpretation today.

### 2.3e Bundles and kits (normative)

A bundle or kit is a `ProductRecord` whose `components` list (slot 6) names its member records by
content address (§16.5.1). Two rules:

- **Members MAY be published by other sellers.** A `components` entry is a content address of some
  other `ProductRecord`; nothing requires the bundle's publisher to also own the members. A bundle
  is a description, not a supply arrangement — supplying it is a separate `Offer`.
- **The bundle converges independently.** A bundle record content-addresses over its own
  canonicalised bytes (§2.3b), including the ordered `components` addresses, so two publishers
  describing the identical kit converge on one bundle address, subject to the same near-duplicate
  caveat as any other record (§2.2a).

A bundle `ProductRecord` remains a public object and MUST NOT carry personal data (§2.3, §16.4).

## 2.4 Standards profiled

schema.org product vocabulary (`Product`, `Offer`, `ProductGroup`, `hasVariant`, `variesBy`) for
the data model, so existing merchant feeds map in by translation. GS1 identifiers (GTIN, MPN) are
supported as **claims only** (§2.3c) — issuance is gated and fee-bearing, so a spec that depended on
them would import a centralization point and a cost barrier. **BCP 47** (RFC 5646) language tags for
all human-facing text (§2.3a), UTF-8 throughout, and **ISO 4217** for currency (§5.3) — so language,
region, and currency are all carried by boring existing standards rather than TRACT vocabulary.

## 2.5 Open

- **Near-duplicate resolution.** The address floor is exact-match; merging almost-identical records
  is an index-side heuristic, and heuristics differ between indexes. Whether the spec should
  recommend one, or deliberately leave it to differ, is undecided (§21.2, §21.8).
- **The concrete value-normalisation for coded attributes** (§2.3b) — the Unicode normalisation form
  plus casefolding applied to enumerated attribute *values* before addressing. The grammar fixes
  key casefolding; value normalisation is unspecified and is a prerequisite for real convergence.
- **Bootstrapping manufacturer signatures** (§2.3c) when most brands will not participate early.
- **The §16 localization-slot grammar change** for display text and its exclusion from the
  content-addressed identity (§2.3a) — a frozen-v0 grammar change to `ProductRecord`, a MAJOR
  version bump, with the canonicalisation consequence that two same-product listings in different
  languages must still converge on one address. Until it lands, §2.3a is the recorded target and not
  the frozen behaviour.
- **Whether to add an explicit `ProductGroup`/`variesBy` production** (§2.3d) rather than recovering
  the varying axes from differing coded `Attribute`s. Leaning "no change" — the attribute-based
  reading works today.

## 2.6 The index rule (normative), and the gap between permission and practice

**The rule (settled, normative).** An **index** is a *derived, rebuildable* view over the public
catalogue. It is **never authoritative**:

- An index MUST NOT be treated as an authority on what a product is or who supplies it; it is a
  convenience over feeds and blobs that any node MAY build, and any node MAY build a competing one.
- On any disagreement between an index and a seller's own signed feed, resolution MUST **prefer the
  feed**. The feed and the content-addressed record are the source of truth; the index is a cache of
  them.
- An index MUST treat a `ClaimedExternalRung` as an advisory join key only, never authority
  (§2.3c).
- **No protocol mechanism exists by which an index can delist a seller from the network.** An index
  can decline to *list* a record in its own view, but it cannot remove the record from the swarm,
  cannot revoke a content address, and cannot prevent another index from listing it.

**The gap (honest, marked — do not read as defended).** "Any node MAY build an index" does not mean
many will, and this is the weakest load-bearing claim in the specification alongside cross-publisher
identity (§2.2a). A content-addressed substrate offers no global index, so discovery is the *first*
function to re-centralize: whichever index becomes economically dominant becomes a de facto
content-policy gatekeeper **regardless of what this document permits**. That is precisely what
happened to the closest deployed relative of this design — OpenBazaar's default crawler became the
gatekeeper (§21.3) — and the largest live decentralized-commerce network avoided it only by adopting
a central approval-gating registry (§21.4). **No rule in this document prevents it.** Multiple
competing indexers with verifiable completeness or censorship proofs is the candidate answer, and it
has **no deployed precedent** (§21.8). This is marked as the weakest claim rather than defended
(§21.9).
