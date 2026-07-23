# 4. Fulfilment

> **Drafting status: normative, with four decisions still open.** The key words MUST, MUST NOT,
> SHOULD, SHOULD NOT and MAY are to be interpreted as in BCP 14 (RFC 2119, RFC 8174). The seven
> fulfilment variants (§4.2), the place-of-supply derivation (§4.3–§4.4), handover semantics for
> the carrier-borne variants (§4.5), return terms (§4.7), and the multi-mode expression rule (§4.8)
> are normative and align to the frozen grammar in [§16.5.2](16-wire-format.md). Four peripheral
> questions remain open and are marked **PROVISIONAL — pending decision** where they arise:
> whether carrier-less variants require the buyer's counter-signature to reach `delivered` (§4.5);
> whether `ship` needs an Incoterm wire field (§4.6); whether `digital-grant` needs a licence-terms
> sub-object (§4.10); and whether repeated same-item offers warrant a grouping construct (§4.8).
> None of the four block the settled core, and none may be filled in by an implementation ahead of
> the decision.

## 4.1 Scope

The third offer axis: how the thing reaches the buyer. This is not only a logistics question — the
`Fulfilment` object is the **only** object that knows where a supply happens, and
[§11.2](11-jurisdiction.md) derives the place-of-supply tax anchor from it and from nothing else.

This section therefore carries the *input* to that anchor and stops there. It **MUST NOT** compute,
look up, or assert a tax rate, a threshold, or a liability; that computation is the seller's or
gateway's edge policy and is out of scope ([§11.2b](11-jurisdiction.md), constraint C3). The
protocol carries the fact — which variant, and where — and the derivation of §4.3 turns that fact
into an anchor country. Whether that anchor is the *legally correct* place of supply in any given
regime is design reasoning that no court has tested against a permissionless no-operator protocol
([§21.11](21-grounding.md)); §4 does not claim otherwise, and carrying the fact honestly is the
whole of its job.

## 4.2 Variants

An `Offer` MUST carry exactly one `Fulfilment` value ([§16.5.2](16-wire-format.md), key 3). The
union is a choice, not a set; the seven variants and what each carries on the wire are fixed by the
frozen grammar:

| Variant | Grammar (§16.5.2) | Carries | Covers |
|---|---|---|---|
| `ship` | `{ 0 => [* country] }` | destination countries served | carried to an address |
| `collect` | `{ 1 => PlaceRef }` | a `PlaceRef` | buyer picks up |
| `digital-grant` | `{ 2 => null }` | nothing | download, licence key |
| `perform-at-place` | `{ 3 => PlaceRef }` | a `PlaceRef` | the venue for a haircut, an event |
| `perform-remote` | `{ 4 => null }` | nothing | consulting over video |
| `access-grant` | `{ 5 => PlaceRef / null }` | a `PlaceRef` **or** nothing | a gym membership, or a streaming subscription — same variant |
| `return-required` | `{ 6 => PlaceRef, 7 => uint }` | a `PlaceRef`, a term in days | a rental or hire |

A `PlaceRef` is `{ 1 => country, 2 => tstr }` — a country and a coarse locality, and **never** a
street address ([§16.4](16-wire-format.md)). No fulfilment variant carries, or MAY carry, a
street-level destination in the public family; the buyer's actual delivery address lives only in the
sealed `Order` ([§16.6](16-wire-format.md)).

§4.8 covers what the one-of union means for a seller who genuinely offers more than one fulfilment
mode for the same item.

## 4.3 The place-of-supply derivation

An implementation MUST derive the place-of-supply anchor as a **pure function of the `Fulfilment`
value** — the same rule the frozen grammar states in [§16.5.2](16-wire-format.md) and that
[§11.2](11-jurisdiction.md) relies on. The function is total over the seven variants:

| Fulfilment variant | Anchor country is | Sourced from | Why |
|---|---|---|---|
| `ship` | the delivery destination | the shipping leg (§8) / the sealed order's `delivery_destination` | the goods physically arrive there; import VAT/GST and duty regimes key off where they land |
| `collect` | the stated place | `PlaceRef.country` (key 1) | the buyer takes possession there, regardless of where either party is established |
| `perform-at-place` | the stated place | `PlaceRef.country` (key 3) | admission and physically-performed services are generally taxed where the performance happens |
| `return-required` | the stated place | `PlaceRef.country` (key 6) | the same reasoning as `collect` — the item changes hands there |
| `perform-remote` | buyer residence | `Anchors.buyer_residence` (§16.6, key 2) | there is no physical venue to anchor to |
| `digital-grant` | buyer residence | `Anchors.buyer_residence` | nothing physical happens anywhere |
| `access-grant` | the stated place **if** the variant carries a `PlaceRef`; otherwise buyer residence | `PlaceRef.country` when key 5 is a `PlaceRef`, else `Anchors.buyer_residence` | the field name is shared by two economically different cases, and the anchor MUST follow whichever one *this* instance actually is |

The `access-grant` split is decided by the grammar itself and MUST be read from it: a value of
`{ 5 => PlaceRef }` anchors to the place, a value of `{ 5 => null }` anchors to buyer residence.
There is no separate flag and no default; the presence or absence of the `PlaceRef` is the whole
signal.

The derived country MUST be written to `Anchors.place_of_supply` ([§16.6](16-wire-format.md),
key 3) on the sealed order. For a `ship` fulfilment the order MUST also carry
`Anchors.delivery_destination` (key 4); for every non-shipping variant that key is absent, "absent
when nothing moves" being exactly what the optional slot encodes. The remaining two anchors —
`seller_establishment` and `buyer_residence` — come from identity and buyer disclosure and are not
this section's to derive; §4 owns only place of supply and, for `ship`, the destination that both
place of supply and `delivery_destination` read from.

This is the forcing example [§0.1](00-overview.md) and [§11.2](11-jurisdiction.md) both cite: an
event held in one country, sold by a seller established in a second, to a buyer resident in a third.
Only the venue in the `Fulfilment` object answers the question; neither party's country does, and
averaging or defaulting to either one is simply wrong.

## 4.4 Why the anchor is derived, never supplied

[§11.2](11-jurisdiction.md) states the rule this section has to honour: place of supply is
**computed from** the `Fulfilment` object, and it is not a separate field an implementation fills in
alongside it. An implementation MUST derive place of supply from the `Fulfilment` value per §4.3 and
MUST NOT accept it as a separate argument that could disagree with the fulfilment details. This is
stricter than it sounds, and it is stricter on purpose.

An earlier implementation took place of supply as its own parameter, populated independently of the
fulfilment details rather than derived from them. Once the two could be set independently, nothing
checked that they agreed — and in practice the anchor defaulted to the seller's own establishment,
because that is the natural default in ordinary billing code, not because anyone decided a haircut
performed abroad should be taxed at home. The system returned a country. It was wrong, and nothing
about the output looked wrong: no error, no missing field, just a confidently incorrect
jurisdiction.

Deriving the anchor as a pure function of the `Fulfilment` value removes the second parameter
entirely. There is nothing left for the anchor to disagree with, because there is nothing else it
could be computed from. This is also why neither `Offer` nor `Order` carries a place-of-supply field
*separate from* the derivation ([§16.5.2](16-wire-format.md), [§16.6](16-wire-format.md)): adding
one back would reopen exactly this failure mode.

One consequence must be stated rather than hidden. `Anchors.place_of_supply` is nonetheless
*stored* on the sealed order, because the fulfilment variant it derives from may change in a later
offer revision and a tax position should reflect the terms as they stood at order time. That makes
the stored anchor a fact a party asserts rather than one a verifier can recompute against a
now-different offer. Whether the anchor is authoritative-as-recorded or should be recomputable is a
known open grammar-level question ([§16.8](16-wire-format.md), item (c)); §4 records the derivation
function and does not re-decide the storage question here.

## 4.5 Handover: what counts, and who signs it

[§18.4](18-state-machines.md) states the general custody rule, and the custody-handoff lifecycle
itself is specified **once, in WRAP** (`Progress` + `Attestation`,
`https://github.com/vul-os/wrap`), referenced from [§8.4](08-delivery.md) and
[§18.4](18-state-machines.md) and written a second time nowhere (constraint C2). A handoff is signed
by the party **taking** custody, not the party giving it up, because a chain attested only by the
sender proves someone tried, and attested by the receiver proves something actually moved. §4 does
not restate those transitions; it states only which variants produce a custody chain at all, and
what constitutes handover for each.

| Variant | What constitutes handover | Signer of the `delivered` transition (§18.3) |
|---|---|---|
| `ship` | the carrier takes custody at first-leg pickup, beginning the WRAP consignment chain; final handover is the last leg's `delivered` transition | carrier proof-of-delivery, or recipient, per [§18.4](18-state-machines.md) |
| `collect` | the buyer takes physical possession at the stated place — no consignment leg exists | seller's claim (see PROVISIONAL below) |
| `digital-grant` | the grant (download, licence key, credential) is transmitted | the seller, at transmission |
| `perform-at-place` | the service is completed at the venue | seller's claim (see PROVISIONAL below) |
| `perform-remote` | the remote session or deliverable is completed | seller's claim (see PROVISIONAL below) |
| `access-grant` | access is issued, and for a term remains valid until it lapses | the seller, at issuance |
| `return-required` | two handovers: outbound at the start of the term, inbound return at or before its end — each a custody transition in the WRAP chain | the party taking custody at each leg (§18.4) |

For `digital-grant` and `access-grant` the seller's signature at transmission/issuance is
sufficient to reach `delivered`, because the thing handed over is the seller's own artefact and
there is no third party who could counter-sign; these are normative today.

**PROVISIONAL — pending decision.** For `collect`, `perform-at-place` and `perform-remote` there is
no carrier and no WRAP consignment chain, so the only signature available for the `fulfilling →
delivered` transition is the seller's own claim that handover happened — exactly the kind of
unilateral assertion the signed-transition model exists elsewhere to avoid.
[§18.3](18-state-machines.md) currently signs that transition "carrier or seller". Whether these
carrier-less variants MUST instead require the buyer's counter-signature to reach `delivered` is
**not resolved**; an implementation MUST NOT unilaterally impose a counter-signature requirement
ahead of the decision, and MUST NOT treat a seller-only `delivered` as dispute-proof. Recorded in
the founder-decision list (§4.10, first bullet).

## 4.6 Incoterms 2020: risk and cost transfer for shipped goods

Incoterms 2020 govern a different question from §4.3's anchor, and the two are easy to conflate the
same way seller/buyer country and place of supply are ([§11.2](11-jurisdiction.md)): Incoterms fix
**when risk and cost** pass from seller to buyer on a shipped good, not **where the supply is
taxed**. A `DAP` shipment and an `EXW` shipment to the same destination have the same place of
supply and different points at which the buyer bears loss in transit. An implementation MUST NOT
derive place of supply from an Incoterm, and MUST NOT let an Incoterm override the §4.3 derivation.

The `Fulfilment` `ship` variant carries only destination countries
([§16.5.2](16-wire-format.md)) — the frozen grammar has **no field for which Incoterm applies**.
Where an Incoterm matters, it currently sits at the leg or order level rather than on the offer's
fulfilment axis, as a convention outside the wire.

**PROVISIONAL — pending decision.** Whether `ship` needs a dedicated Incoterm wire field is open
(§4.10, second bullet). Adding one is a change to a frozen §16 shape and therefore a MAJOR version
bump, not a correction; it MUST NOT be introduced as an unrecorded field. Until decided, an
implementation that needs to convey an Incoterm carries it out of band at the leg/order level and
MUST NOT invent a `Fulfilment` extension key for it.

## 4.7 Return terms for rentals

`return-required` carries a `PlaceRef` and a term in days ([§16.5.2](16-wire-format.md), keys 6–7)
and nothing else. What happens if the item comes back damaged, or late, is **not** a fulfilment-axis
question and MUST NOT be encoded as a condition on the offer. It is a dispute over an order already
placed, resolved through the order and escrow machinery ([§7](07-order.md),
[§18.5](18-state-machines.md)); the physical return leg is a custody handoff carried in the WRAP
chain (§4.5, `https://github.com/vul-os/wrap`), not a fulfilment field.

The natural pairing is with `DepositBalance` consideration ([§5.8](05-consideration.md)): a deposit
taken at order time is the mechanism by which a late-return or damage claim is actually made whole.
The fulfilment axis's job stops at stating **where** and **for how long**; it does not, and MUST
not, state what enforces the return.

## 4.8 One item, several ways to get it

A seller who genuinely offers both `collect` and `ship` for the same item — common, not an edge
case — cannot express that as a single `Offer`, because `Fulfilment` is a one-of union and an
`Offer` carries exactly one value of it ([§16.5.2](16-wire-format.md), key 3). Under the frozen
grammar the seller MUST express it as two `Offer` objects: same `Item`, same `Availability`, same
`Consideration`, differing only in `Fulfilment`. Each is independently published and independently
withdrawable ([§18.2](18-state-machines.md)). A buyer's node presents both against the same
underlying product and lets the buyer's fulfilment choice pick which offer — and therefore which
anchor (§4.3) — the resulting order binds to.

There is no offer-level alternative-fulfilment construct today; there are only multiple offers that
happen to share everything but the fulfilment axis.

**PROVISIONAL — pending decision.** Whether that repetition warrants a first-class grouping
construct is open (§4.10, fourth bullet). Such a construct would be a new §16 shape (MAJOR bump);
until decided, an implementation MUST NOT introduce one, and MUST express multi-mode fulfilment as
separate, independently signed offers.

## 4.9 Standards profiled

Incoterms 2020 for the risk/cost transfer point on shipped goods (a convention, not currently a wire
field — §4.6). ISO 3166-1 alpha-2 for the country in every `PlaceRef` and `ship` destination
([§16.3](16-wire-format.md), [§16.5.2](16-wire-format.md)).

## 4.10 Open

Each item below is a founder-decision, restated at the point it arises above. None is filled in by
this section.

- **Licence terms for grants.** Whether `digital-grant` needs a licence-terms sub-object, or defers
  entirely to §5's consideration axis. A sub-object would be a new §16 shape (MAJOR bump); deferring
  keeps the grammar as frozen. Leaning defer to §5, pending decision.
- **Incoterm wire field (§4.6).** Whether an Incoterm needs its own wire field on `ship`, or remains
  a leg/order-level convention outside the offer. A field is a §16 change; the convention is not.
- **Counter-signature for carrier-less delivery (§4.5).** Whether `collect`, `perform-at-place` and
  `perform-remote` MUST require the buyer's counter-signature to reach `delivered`, rather than
  accepting the seller's signature alone. Touches [§18.3](18-state-machines.md)'s transition signer.
- **Grouping construct for multi-mode offers (§4.8).** Whether repeated same-item, different-
  fulfilment offers warrant a first-class grouping construct, given the grammar currently expresses
  the case only as separate, independently signed objects.
