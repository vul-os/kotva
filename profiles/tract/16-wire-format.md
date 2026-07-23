# 16. Wire format

> **Status: normative, frozen at v0.** This section defines bytes, and unlike the sections around
> it, it is no longer a proposal. The key words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are to
> be interpreted as in BCP 14 (RFC 2119, RFC 8174).
>
> **What freezing commits to.** A change to any shape below changes what an implementation puts on
> the wire, so it is a MAJOR version change under `GOVERNANCE.md` — not a correction. That is the
> point of freezing: §15's conformance vectors cannot exist against a moving target, and a second
> implementation cannot prove it matches this document rather than matching the reference one until
> there is something fixed to match.
>
> **One correction was made after freezing, before anything depended on it.** `Order` carried no
> jurisdictional anchors, while §11.3 stated that every order names them — a contradiction the
> first conformance vectors surfaced within hours of the freeze, which is precisely what a vector
> corpus is for. The fields were added rather than the gap recorded, because freezing something
> known to contradict another section preserves the appearance of stability and nothing else.
>
> **What it does not claim.** Every object here is implemented and exercised end to end in the
> reference implementation, which is evidence the shapes *compose* and is **not** evidence they are
> *right* — an implementation and a grammar derived from each other only prove they agree with one
> another (`conformance/README.md`). Freezing is a commitment to stability, not a claim of
> correctness, and §16.8 still lists what is undecided **above** the byte level.

## 16.1 Scope

The byte-level definition of every TRACT object: what is encoded, in what order, with which keys,
and which of those keys a decoder may not tolerate.

## 16.2 Conventions inherited, not reinvented

Deterministic CBOR (RFC 8949 §4.2). Integer-keyed maps, keys assigned per object type from 1, keys
≥ 64 reserved for extension. A decoder MUST reject an unknown key in a **signed** object, failing closed; it MAY
ignore unknown keys ≥ 64 in an **unsigned** object. Domain-separation tags on every signing preimage. Content addresses carry
a multihash-style agility prefix.

**TRACT introduces no new hash construction, no new signature framing, and no new address scheme.**
All four come from the DMTAP substrate. An implementation MUST NOT invent one; a construction appearing here would be a defect in this
document.

## 16.3 Shared primitives

```cddl
content-address = bytes         ; multihash-style prefix ‖ digest (substrate §18.1.5)
identity-key    = bytes         ; the public half of an IK; also a courier, distributor or gateway
ts              = int           ; milliseconds since the Unix epoch
country         = tstr .size 2  ; ISO 3166-1 alpha-2
currency        = tstr .size 3  ; ISO 4217

money = {
  1 => int,        ; minor_units — NEVER a floating-point value (§16.7)
  2 => currency,
}
```

## 16.4 The structural rule — two type families, not one with a flag

Public and sealed objects are **separate type families**. A public object MUST NOT carry a name, address or contact
detail (§0.5.1), and the grammar below is written so that it cannot: the prohibition is enforced by
the productions, not by reviewer discipline, because published objects are irrevocable and an
erasure right cannot be satisfied against them after the fact.

Two consequences the grammar has to carry, rather than leaving to convention:

- **A locality, never an address.** Public objects that reference a place (`PlaceRef`,
  `CapacityRecord`) carry a country and a coarse locality only. There is no street-address
  production in the public family at all, so "just add a line" is a grammar change a reviewer sees
  rather than a field a contributor adds.
- **No boolean discriminator.** A decoder is always in exactly one mode — expecting a public object
  or a sealed one — and the two are made non-confusable at the *content-address* level by distinct
  domain-separation tags, in the same way the substrate separates its own sealed and public
  manifests. There is no `sealed: true` field an attacker could omit, because a flag can be
  dropped and a domain-separation tag cannot. A decoder expecting one family and handed the other
  MUST reject it rather than coerce.

## 16.5 Public objects

### 16.5.1 `ProductRecord` — what a thing is

Belongs to nobody. Two publishers whose canonicalised records encode identically converge on one
content address (§2.2), with the caveat §2.2a records about how weak that is in the real case.

```cddl
ProductRecord = {
  1 => tstr,                    ; name        canonicalised (§2.3)
  2 => tstr,                    ; description canonicalised; may be empty
  3 => [* Attribute],           ; attributes  sorted, deduplicated, keys casefolded
  4 => [* IdentityRung],        ; identity    weakest first
  ? 5 => content-address,       ; group       the ProductGroup this varies within
  ? 6 => [* content-address],   ; components  bundle members; may reference other sellers' records
}

Attribute = { 1 => tstr, 2 => tstr }    ; key (casefolded), value

IdentityRung = ContentAddressRung / ClaimedExternalRung / ManufacturerSignedRung
ContentAddressRung     = { 0 => content-address }
ClaimedExternalRung    = { 1 => tstr, 2 => tstr }   ; scheme ("gtin", "mpn"), value — UNVERIFIED
ManufacturerSignedRung = { 2 => identity-key }      ; the brand's own key
```

The middle rung is a **claim and nothing more**: anyone can assert any GTIN, so an index MUST treat
it as an advisory join key and MUST NOT treat it as authority (§2.3). Squatting is expected rather
than prevented.

### 16.5.2 `Offer` — one seller's claim to supply

```cddl
Offer = {
  1 => Item,
  2 => Availability,
  3 => Fulfilment,
  4 => Consideration,
  5 => [* country],             ; sell_to — territories this offer may be accepted from
  6 => ts,                      ; published
}

Item = { 0 => content-address }                             ; product
     / { 1 => content-address, 2 => content-address }       ; variant-of-group: group, variant
     / { 3 => content-address }                             ; service
     / { 4 => content-address }                             ; right / licence
     / { 5 => content-address }                             ; capacity

Availability = { 0 => StockSignal }
             / { 1 => tstr, 2 => uint }        ; time-slots: RFC 5545 payload, slot minutes
             / { 3 => uint, 4 => tstr }        ; capacity-per-interval: capacity, RFC 5545 recurrence
             / { 5 => null }                   ; unlimited
             / { 6 => uint }                   ; made-to-order: lead days

StockSignal = { 0 => uint } / { 1 => null } / { 2 => null } / { 3 => null }
            ; exact(n) / in-stock / low / out-of-stock — a band is publishable instead of a number,
            ; because exact stock is commercially sensitive and browsing does not require it

Fulfilment = { 0 => [* country] }              ; ship: destinations served
           / { 1 => PlaceRef }                 ; collect
           / { 2 => null }                     ; digital-grant
           / { 3 => PlaceRef }                 ; perform-at-place — the venue
           / { 4 => null }                     ; perform-remote
           / { 5 => PlaceRef / null }          ; access-grant, physical or not
           / { 6 => PlaceRef, 7 => uint }      ; return-required: place, term days

PlaceRef = { 1 => country, 2 => tstr }         ; country, LOCALITY — never a street address (§16.4)

Consideration = { 0 => money }                          ; fixed
              / { 1 => [+ PriceTier] }                  ; tiered / volume
              / { 2 => money, 3 => tstr }               ; recurring: amount, RFC 5545 RRULE
              / { 4 => tstr, 5 => money }               ; metered: dimension, unit price
              / { 6 => money, 7 => money }              ; deposit + balance
              / { 8 => null }                           ; quote-required (RFQ; also B2B contract pricing)

PriceTier = { 1 => uint, 2 => money }          ; min_qty, unit_price
```

**The load-bearing detail.** `Fulfilment` is not merely logistics: it is the only object that knows
where a supply happens. §4 derives the tax anchor from this object ([§4.3](04-fulfilment.md)); an
implementation MUST derive it from this object and MUST NOT accept it as a separate argument that
could disagree — one resolving place of supply from the parties' countries instead will be
plausibly and consistently wrong about every event held abroad.

### 16.5.3 `RateCard` and `CapacityRecord` — published, so routing is computed not quoted

```cddl
RateCard = {
  1 => identity-key,            ; carrier — a multinational and a neighbour with a van, identically typed
  2 => [* country],             ; serves
  3 => [* Zone],
  4 => uint,                    ; dim_divisor — billable = max(actual, L*W*H / divisor)
  5 => uint,                    ; surcharge_pct
  6 => [* tstr],                ; excluded_categories
  7 => ts,                      ; published — cards drift; a stale one is not replayed silently
}

Zone          = { 1 => uint, 2 => [+ WeightBracket], 3 => uint }  ; id, brackets, transit_days
WeightBracket = { 1 => uint, 2 => money }                          ; max_grams, price

CapacityRecord = {                                 ; a distributor's published consolidation offer
  1 => identity-key,            ; distributor
  2 => country,
  3 => tstr,                    ; locality — coarse (§16.4)
  4 => money,                   ; storage_per_day
  5 => money,                   ; handling_fee
  6 => uint,                    ; slots
  7 => [* tstr],                ; excluded_categories
}
```

A rate card is a **claim by its publisher**, and a transit figure inside it is an estimate. §8.4
records that some carriers restrict redistribution of negotiated rates, which is why an offer may
instead declare that a live quote is required.

### 16.5.4 `EscrowScope` — what an operator can lawfully serve

```cddl
EscrowScope = {
  1 => identity-key,            ; operator
  2 => [* country],             ; buyer_countries
  3 => [* country],             ; seller_countries
  4 => [* country],             ; supply_countries — checked against place of supply, not the parties
  5 => [* currency],
  6 => [* RailClass],
  7 => money,                   ; max_order_value — usually a KYC threshold
  8 => [* tstr],                ; excluded_categories
  9 => [* tstr],                ; authorities claimed — prose, because regulators share no schema
}

RailClass = 0 / 1              ; 0 = custodial-reversible, 1 = non-custodial-final
```

`supply_countries` is a separate field from the party countries deliberately: two trades with
identical buyers, sellers, currency and rail can differ only in where the supply happens, and that
difference alone can put one of them outside an operator's licence (§9.4).

### 16.5.5 `Review` and `PurchaseAttestation` — the bounded exception

```cddl
Review = {
  1 => Subject,
  2 => identity-key,            ; author — a PER-SUBJECT pseudonymous subkey, never the root IK
  3 => uint,                    ; score, 0..5
  4 => tstr,                    ; body
  ? 5 => PurchaseAttestation,
  6 => ts,
}

Subject = { 0 => content-address }   ; product
        / { 1 => identity-key }      ; seller
        / { 2 => identity-key }      ; distributor
        / { 3 => identity-key }      ; courier

PurchaseAttestation = {
  1 => 0 / 1,                   ; attestor: 0 = seller, 1 = escrow operator
  2 => identity-key,            ; issuer
  3 => content-address,         ; order — the SEALED order's address only; never its contents
  4 => ts,
}
```

A review is the one public object signed by a natural person, and §10.4 bounds it rather than
exempting it. Two things the grammar carries: the author is a per-subject subkey so reviews are not
trivially linkable across sellers, and the attestation references a sealed order **by address
only** — nothing about what was bought crosses into the public quadrant.

`body` remains free text a person could type an address into. The grammar cannot prevent that;
§10.4 and the client requirements have to, and pretending otherwise would be exactly the kind of
overclaim §16.4 exists to avoid.

## 16.6 Sealed objects

Never published. Held at the two endpoints, where deletion is meaningful.

```cddl
Order = {
  1 => identity-key,            ; buyer
  2 => identity-key,            ; seller
  3 => [+ OrderLine],           ; THIS seller's lines only — never the whole cart
  4 => money,                   ; total
  5 => OrderState,
  6 => ts,                      ; placed
  7 => Anchors,                 ; the four jurisdictional anchors (§11.2)
  8 => Responsible,             ; who is answerable, and for what (§11.3)
  ; buyer name, delivery address and contact details are carried here, in the sealed family,
  ; and have no production in the public family at all (§16.4)
}

; §11.2's four anchors, carried on the order because they are the facts a regulator asks for and
; because three of them cannot be recomputed later: buyer residence is disclosed at order time and
; nowhere else, and place of supply depends on the fulfilment variant as it stood when the order
; was placed, not as the offer reads today.
Anchors = {
  1 => country,                 ; seller_establishment
  2 => country,                 ; buyer_residence
  3 => country,                 ; place_of_supply — derived from Fulfilment (§4), never supplied beside it
  ? 4 => country,               ; delivery_destination — absent when nothing moves
}

; §11.3. `facilitator` present iff a gateway settled the payment: that presence is the
; marketplace-facilitator hook, and its absence is equally load-bearing, because a self-hosted
; seller taking direct payment is never a facilitator (§21.11.2).
Responsible = {
  1 => identity-key,            ; seller_of_record
  ? 2 => identity-key,          ; facilitator — the gateway, if it settled
  ? 3 => identity-key,          ; importer_of_record
  ? 4 => Representative,        ; in-region responsible person, where a regime requires one
}

Representative = { 1 => country, 2 => identity-key }   ; region covered, who

OrderLine  = { 1 => content-address, 2 => identity-key, 3 => uint }   ; offer, seller, quantity
OrderState = 0 / 1 / 2 / 3 / 4 / 5 / 6 / 7 / 8
           ; draft / placed / accepted / declined / countered / fulfilling / delivered /
           ; closed / cancelled  (§18)

PaymentAttestation = {
  1 => identity-key,            ; payer
  2 => identity-key,            ; payee
  3 => content-address,         ; order
  4 => money,
  5 => RailClass,               ; determines the buyer's recourse — never flattened to a boolean
  6 => tstr,                    ; external settlement reference, opaque
  7 => ts,
}
```

**One order per seller is a grammar-level property, not a client convention.** `Order` names a
single seller and MUST carry only that seller's lines, so a cross-seller order is not expressible.
The whole-cart view exists on the buyer's device and nowhere else (§6.1).

`OrderLine.seller` (key 2) MUST equal `Order.seller` (key 2) for every line; the field names the
line's seller for local convenience, not a second seller the order could bind to. A decoder MUST
reject, failing closed, an `Order` containing any `OrderLine` whose `seller` differs from
`Order.seller` — this MUST is what keeps a cross-seller order not expressible, since the field
alone does not forbid one at the grammar level.

`PaymentAttestation` MUST carry a *reference* only — never funds, and never card data. The protocol conveys
that a payment happened; it does not move money (§9.2).

## 16.7 Encoding rules that exist because of a specific failure

- **Money MUST be minor units and a currency code, never a float.** A rounding error in a signed object
  cannot be corrected after the fact — the signature covers the wrong number.
- **Arithmetic across currencies MUST be refused, never coerced.** A silently converted total is a wrong
  total that looks right, and it gets carried into an order (§5.3).
- **A decoder that finds an unrecognised field in a signed object MUST reject it**, rather than
  ignoring it. The alternative lets a signer and a verifier disagree about what was signed.
- **`ts` MUST be milliseconds since the Unix epoch**, subject to the substrate's clock-skew
  tolerance. A
  timestamp is for display and ordering; where order must be authoritative it comes from a feed's
  sequence, never from a clock.

## 16.8 Open

- **Whether `Offer` carries its own signature or inherits authenticity from its position in a
  signed feed.** The substrate's feed head transitively commits to every entry, which makes a
  per-object signature redundant — and makes an offer quoted out of its feed unverifiable. The
  trade is real and undecided.
- **Whether `sell_to` being empty means "unrestricted" or "malformed."** Unrestricted is
  convenient and is almost never what a seller means, given §11's in-region representative
  requirements. Leaning malformed.
- **Whether `Anchors.place_of_supply` should be recomputable or is authoritative as recorded.** It
  is stored because the fulfilment variant it derives from may change in a later offer revision,
  and a tax position should reflect the terms at order time. That makes it a fact a party asserts
  rather than one a verifier can check, which is a real weakening and the reason it is listed here.
- **Extension-key policy for the axis unions.** Each axis is a small closed set today. What a
  decoder does with an axis variant it has never seen — refuse, or preserve and refuse only on
  *acting* — is §17.4's tolerant-to-store / strict-to-act rule, and it needs stating here in
  grammar terms rather than only in prose.
