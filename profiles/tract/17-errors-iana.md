# 17. Errors & registries

> **Drafting status.** Normative for the registry machinery and the initial code assignments;
> partial above the code level. The code format, the subsystem table, the responder-action
> vocabulary, the initial error table, the extension policy (*tolerant to store, strict to act*),
> and the disjointness of the TRACT and substrate registries are now **normative**. Four things
> remain scoped and are marked `PROVISIONAL` where they appear: the physical representation of the
> registries (in-document tables vs. a machine-readable file), the allocation policy for the
> unassigned and private-use subsystem ranges, whether the excluded-category vocabulary graduates
> to a controlled list, and whether tax-treatment categories can be enumerated at all before the
> jurisdiction research lands. Media types and well-known URIs (§17.7) are likewise `PROVISIONAL`.
>
> The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, MAY
> and OPTIONAL are to be interpreted as in BCP 14 (RFC 2119, RFC 8174) when, and only when, they
> appear in all capitals.

## 17.1 Scope

The `ERR_TRACT_*` code block, the responder-action vocabulary, an initial error table, and the
extensible registries — everything a conformant implementation needs to name a failure the same
way another implementation would, without either one inventing its own vocabulary for it.

This section adds **no new validation logic and no new wire object**. Every code it assigns traces
to a rule a numbered section already states in prose, and the code is a name for that rule's
failure, not a second statement of it. Error codes are a diagnostic and interoperability registry;
they are not themselves TRACT wire objects, and §16 defines no error frame. How a responder
conveys a code to a peer — inline in an application response, in a log, in a substrate-level
rejection — is out of this document's scope. What is in scope is that two implementations naming
the same failure name it identically.

**TRACT codes cover the commerce spine only.** A condition the DMTAP substrate already names — a
transport failure, a signature or content-address validation failure, a feed anti-rollback
violation, an unrecognised substrate message — MUST be raised with the substrate's own code from
the substrate's own registry, never re-coded here. The two registries are disjoint (§17.2), and
duplicating a substrate condition under a TRACT number would create exactly the two-names-for-one-
thing drift this section exists to prevent. The substrate registry is defined by the DMTAP
specification at `github.com/vul-os/dmtap`; TRACT references it and does not reproduce it.

## 17.2 Code format and the subsystem table

A TRACT error code MUST be a 16-bit value `0xSSNN`: `SS` is the subsystem byte and `NN` is the
code point within it — the layout the sibling DMTAP specification uses for its own registry, reused
here for form only. TRACT allocates its own subsystem bytes rather than sharing DMTAP's table. A
TRACT code and a DMTAP code that happen to carry the same `SSNN` value name unrelated conditions in
two different registries, never the same one; an implementation MUST NOT interpret a TRACT code
against the substrate table, nor a substrate code against this one.

This appendix introduces no new validation logic. Every code below traces to a rule some numbered
section already states — §0.5.1, §2.3, §5.3, §6.2, §6.4, §8.2, §9.3, §9.4, §10.2, §11.2, §11.3,
§12.3, §16.4 and §16.7 between them. A code is normative wherever its owning section is normative;
the owning clause is authoritative, and this table is a pointer to it, never a restatement (the
same discipline §15.3 states from the other side). Once a code point is assigned it is stable:
changing the meaning of an assigned `SSNN` value, or reassigning it, is a MAJOR version change under
`GOVERNANCE.md`, on the same footing as a change to a §16 shape, because a second implementation
cannot match a moving registry any more than it can match a moving grammar.

| `SS` | Subsystem |
|-----:|-----------|
| `0x00` | Reserved |
| `0x01` | Wire format — public/sealed structure (§16) |
| `0x02` | Catalogue (§2) |
| `0x03` | Availability & fulfilment (§3, §4) |
| `0x04` | Consideration (§5) |
| `0x05` | Cart (§6) |
| `0x06` | Order (§7) |
| `0x07` | Delivery (§8) |
| `0x08` | Settlement (§9) |
| `0x09` | Trust (§10) |
| `0x0A` | Jurisdiction (§11) |
| `0x0B` | Gateway (§12) |
| `0x0C` | Anti-abuse & feed continuity (§14) |
| `0x0D`–`0xEF` | Unassigned — reserved for future subsystems |
| `0xF0`–`0xFE` | Private use (experimental/vendor subsystems) |
| `0xFF` | Reserved |

The grouping is deliberately not one byte per section number: §3 and §4 share a subsystem because
the four axes (§2.3) are inspected together at the same points — an offer is rejected or accepted as
a whole, never one axis at a time — and §13 and §15 have no subsystem byte because neither has a
checkable object shape to raise a code against. A subsystem byte marked *Unassigned* MUST NOT be
used by a conformant implementation; a byte in the *Private use* range MAY be used for
experimental or vendor conditions, which by definition do not interoperate and MUST NOT appear in a
conformance vector (§15).

> **PROVISIONAL — pending decision.** The allocation policy for the `0x0D`–`0xEF` unassigned range
> and the `0xF0`–`0xFE` private-use range (Specification Required, first-come-first-served, or
> something else) is not yet decided. Mirroring DMTAP's policy is one option, not a commitment; see
> §17.8.

## 17.3 Responder action vocabulary

Every code below resolves to exactly one of a closed set of five actions. A conformant
implementation MUST map each code it raises to exactly one of these five, and MUST NOT invent a
sixth: a new action added ad hoc the day someone needs one is exactly how a registry like this
drifts into having as many behaviours as it has codes. Extending this vocabulary is a MAJOR version
change, not an implementation liberty.

| Action | Meaning | When it applies |
|---|---|---|
| **fail-closed-block** | Refuse to proceed, and refuse to silently degrade into a lesser but still-plausible outcome. The object, order or trade stops here until the condition is fixed by the party who caused it. | The default for structural violations — a public object carrying personal data, a type-confused decode, an offer missing an axis — where continuing at all, in any reduced form, is the mistake. |
| **drop-silent** | Discard without notifying the sender; to the sender this is indistinguishable from packet loss. | Reserved for conditions where notifying would itself leak information the sender should not have. TRACT's own table has no entry using it — the substrate already disposes of the case that would need it (an unrecognised message reaching a node under the substrate's own cold-contact handling, §14.3), and TRACT does not re-litigate that disposition. |
| **rotate-retry** | An internal recovery action — try a different path, a different replica, a refreshed copy of an object — with no user-visible failure unless the retries are exhausted. | Transient conditions, not structural ones. A rate-card fetch failing over a flaky connection is a substrate-level reachability condition (inherited, not a TRACT code); TRACT's own conditions are checks against content already received, which is why no §17.4 row uses it. |
| **deny-policy** | Reject per a deliberate, non-adversarial policy decision: the object is well-formed and could succeed elsewhere, but this responder's own declared scope or standard does not cover it. | An `EscrowScope` intersection that comes up empty, or a review an index declines to serve for want of attestation — both are facts about this responder's stated terms, not defects in what was presented. |
| **halt-alert** | Halt the affected process and raise a visible alert. | Reserved for conditions that indicate tampering or equivocation, not ordinary refusal. TRACT's own table has none: every condition §17.4 names is a bounded, explainable refusal, not evidence of an attack. The substrate's own halt-alert conditions (a broken identity chain, a split-view key-transparency log) already apply beneath TRACT unchanged and are not reproduced here. |

## 17.4 Initial error table

The codes below are normative. Each row's condition is owned by the cited clause; where that clause
is normative, the code is normative with it.

| Code | Name | Operation | Meaning | Retryable | Action |
|---|---|---|---|:---:|---|
| `0x0101` | `ERR_TRACT_TYPE_CONFUSION` | decode of any object (§16.4) | a sealed object is presented under a public-object schema, or a public object under a sealed one | No | fail-closed-block |
| `0x0102` | `ERR_TRACT_PERSONAL_DATA_PUBLIC` | decode or publish of any public-family object (§22.3's canonical list, §16.4) | a public-quadrant object carries a field that identifies, or is linkable to, a natural person | No | fail-closed-block |
| `0x0201` | `ERR_TRACT_OFFER_AXIS_MISSING` | `Offer` decode (§2.3, §16.5.2) | one or more of the four axes — Item, Availability, Fulfilment, Consideration — is absent | No | fail-closed-block |
| `0x0401` | `ERR_TRACT_CURRENCY_MISMATCH` | consideration or route-total arithmetic (§5.3, §16.7) | two `money` values of different currencies are combined without an explicit, disclosed conversion | Conditional — a disclosed conversion supplies a single currency | fail-closed-block |
| `0x0501` | `ERR_TRACT_OVERSELL_PREVENTED` | cart checkout against partitioned inventory (§6.2, §6.4) | a sale would exceed the combined quota remaining across a seller's replicas | No | fail-closed-block |
| `0x0701` | `ERR_TRACT_ROUTE_TOTAL_OVERFLOW` | route-total computation (§8.2, §16.7) | summing leg prices would exceed the representable range of a `money` minor-units integer | No | fail-closed-block |
| `0x0702` | `ERR_TRACT_RATE_CARD_STALE` | rate-card use for local quoting (§8.2, §19.6) | a `RateCard` older than `RATE_CARD_MAX_AGE` is used to compute a quote without disclosing its age | Yes — refetch a current card | fail-closed-block |
| `0x0801` | `ERR_TRACT_ESCROW_SCOPE_EMPTY` | checkout scope intersection (§9.4) | the buyer's declared `EscrowScope` and the gateway's offered scope intersect on no valid combination of country, currency, rail class, value ceiling or category | No | deny-policy |
| `0x0802` | `ERR_TRACT_RAIL_CLASS_SUBSTITUTED` | rail selection at checkout (§9.3) | a `CustodialReversible` rail is replaced with `NonCustodialFinal`, or the reverse, without a fresh agreement recorded on the order | No | fail-closed-block |
| `0x0901` | `ERR_TRACT_REVIEW_UNATTESTED` | `Review` intake by an index (§10.2, §16.5.5) | the review carries no valid `PurchaseAttestation` from the seller or an escrow operator | No | deny-policy |
| `0x0A01` | `ERR_TRACT_PLACE_OF_SUPPLY_UNRESOLVED` | tax-anchor derivation (§11.2, §16.5.2 `Fulfilment`) | the Fulfilment variant does not yet resolve to a single place of supply — most often a multi-variant offer whose buyer choice is not yet recorded on the order | Conditional — resolves once the buyer's choice is recorded | fail-closed-block |
| `0x0A02` | `ERR_TRACT_RESPONSIBLE_ROLE_MISSING` | `Responsible` completeness check at order placement (§11.3.4) | a responsible role a regime requires — most often an in-region `Representative` — is absent from an order that policy says must name one | Conditional — resolves once the required party is supplied | fail-closed-block |
| `0x0B01` | `ERR_TRACT_ORIGIN_NOT_ISOLATED` | render-bundle serving (§12.3) | two stores would be served from the same normalised origin | No | fail-closed-block |

`0x0A02` is recorded here at the explicit request of §11.3.4, which states the fail-closed behaviour
normatively but had no dedicated code; assigning one touches no §16 bytes, because the condition is
a policy check over the already-defined `Responsible` map (§16.6), not a new field.

Retryable means the same condition, presented again with nothing else changed, can succeed —
"Conditional" marks the cases where success depends on an action outside the responder's own control
(the buyer recording a choice, a required party being supplied, a fresher object being fetched), not
on retrying the identical request. `0x0401`'s and `0x0701`'s reasoning is the same reasoning, run in
two directions: a currency silently coerced, or a total silently wrapped, is a wrong number that
looks exactly like a right one, and by the time it is carried into a signed order there is no
downstream step that can tell the difference (§16.7). Refusing both at the point of computation is
cheaper than auditing every order for one afterward.

## 17.5 Extensible registries

| Registry | What it enumerates | Where defined | Extension policy |
|---|---|---|---|
| Item kinds | product / variant-of-group / service / right-or-licence / capacity | §2, §16.5.2 `Item` | tolerant to store, strict to act (§17.6) |
| Availability variants | stock-signal / time-slots / capacity-per-interval / unlimited / made-to-order | §3, §16.5.2 `Availability` | tolerant to store, strict to act |
| Fulfilment variants | ship / collect / digital-grant / perform-at-place / perform-remote / access-grant / return-required | §4, §16.5.2 `Fulfilment` | tolerant to store, strict to act |
| Consideration variants | fixed / tiered / recurring / metered / deposit-and-balance / quote-required | §5, §16.5.2 `Consideration` | tolerant to store, strict to act |
| Rail classes | custodial-reversible / non-custodial-final | §9.3, §16.5.4 `RailClass` | a closed set of two, **not** an ordinary extensible registry — a third class is a specification change, because §9.3 makes the class part of what determines a buyer's recourse, and a registry addition here would be a substitution by another name |
| Tax treatment categories | — not yet enumerated | §11 (§11.3.5 commits to the field; its values are undecided) | tolerant to store, strict to act, once populated — see §17.8 |
| Excluded-category vocabulary | free-text category strings a rate card, capacity record or escrow scope may exclude | §8, §9, §16.5.3, §16.5.4 | advisory strings today, not a controlled list — see §17.8 |
| External-identifier schemes | `gtin`, `mpn`, and others a publisher may claim | §2.3, §16.5.1 `ClaimedExternalRung` | advisory join keys only, never authority (§2.3); a scheme absent from this registry MUST be preserved and surfaced as unknown, never treated as a match |

Each axis registry above (Item / Availability / Fulfilment / Consideration) enumerates exactly the
variants §16.5.2's CDDL encodes today; a value outside that CDDL is not a registry extension a
current decoder can accept but an unrecognised key, governed by §16.2's signed-object rule and the
extension-key question §16.8 leaves open. Adding a variant is therefore a §16 grammar change (a
MAJOR bump), and this registry tracks the grammar rather than growing independently of it.

## 17.6 Tolerant to store, strict to act

Every registry in §17.5 except rail classes is extensible, and an unrecognised value MUST NOT be
fatal to a **generic index**: the object MUST be preserved and the unrecognised field surfaced as
unknown, rather than causing the object to be dropped. An index that refused to hold an offer merely
because it used a Fulfilment variant from a later registry update would make every extension to this
specification a backward-compatibility break for every index already deployed.

A client that **renders or transacts** against a variant it does not implement is a different actor
making a different decision, and it MUST refuse rather than guess. The two behaviours are
independently required: a conformance case checking only one could pass an implementation that
guesses exactly where it should refuse.

Guessing is worse than refusing here for the same reason a silently coerced currency or a silently
wrapped total is worse than a refusal (§17.4): a guess produces an answer that looks exactly as
valid as a correct one, and nothing downstream carries a marker saying it was a guess. A client that
rendered an unrecognised Fulfilment variant as if it were `collect`, because `collect` is the
closest shape it understands, would tell a buyer a wrong place of supply with the same confidence it
would state a right one (§11.2) — and the buyer would have no way to tell the difference until a tax
authority did. A client that refuses instead produces a visible boundary: the buyer learns their
client does not yet understand this offer, which is a true and actionable fact rather than a false
and confident one.

This is the prose statement §16.8 defers to when it asks for the extension-key policy to be restated
in grammar terms; the grammar-level form of the rule remains a §16 open item, and until it is
settled the two documents MUST be read together — §16.2 for what a decoder does with an unrecognised
key at decode time, §17.6 for what a storing index versus an acting client does with it thereafter.

## 17.7 Media types and well-known URIs

> **PROVISIONAL — pending decision.** TRACT registers no media type and reserves no well-known URI
> in v0. This subsection records why, and what the settled parts already imply, so that a future
> registration does not contradict them.

The settled facts that bound any future registration:

- **The container framing is the substrate's, not TRACT's.** A TRACT public object is a
  content-addressed substrate blob and a `PubManifest` (`FEEDS.md`); a sealed object is held at the
  two endpoints. Neither is fetched through a content-type-negotiating transport in the peer path,
  so the substrate's own framing — not a TRACT media type — governs how bytes are labelled on the
  wire. TRACT MUST NOT restate that framing (`github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md`).
- **Discovery is index-mediated, not host-mediated.** TRACT deliberately has no authoritative host
  a buyer resolves a catalogue *from* (§2.6, §0.4.1); an index is derived, rebuildable and never
  authoritative. A `.well-known/` path is a host-centric discovery point, and minting one would
  reintroduce exactly the de-facto-gatekeeper surface §21.3/§21.4 records as decentralised
  commerce's first re-centralisation failure. No well-known URI is defined, and one SHOULD NOT be
  added without confronting that.
- **The one HTTP surface is the gateway, and it serves HTML.** A gateway (§12) renders storefronts
  to keyless browsers over HTTP; that traffic is presentation HTML, not TRACT CBOR, and needs no
  TRACT media type.

**Recommendation for the founder decision.** If a non-substrate transport ever needs to label TRACT
objects for content-type negotiation, register a **single** structured-suffix type
`application/tract+cbor` with the object family carried as a parameter, rather than minting a
per-object-type family (`application/tract-offer+cbor`, `…-review+cbor`, …). One type keeps the
registry additive-free as new object types are added under §16's MAJOR-bump discipline; a per-type
family turns every §16 addition into an IANA registration too, which is the same sprawl §17.3 closes
off for actions. Do **not** reserve a well-known URI; discovery stays index-mediated. Recorded for
decision in §17.8.

## 17.8 Open

- Whether the registries of §17.5 live as tables in this document, or as a separate machine-readable
  file this document points to. A table reviewed alongside the prose that defines each variant is
  easier to keep honest; a separate file is easier for an implementation to consume without parsing
  markdown. **Recommendation:** keep the normative source in this document and generate the
  machine-readable file from it in the build, so there is one authority and no drift.
- The allocation policy for the `0x0D`–`0xEF` unassigned range and the `0xF0`–`0xFE` private-use
  range — Specification Required, first-come-first-served, or something else — is undecided.
  Mirroring DMTAP's policy is one option, not a commitment. **Recommendation:** Specification
  Required for the unassigned range (a code is only interoperable if the rule it names is written
  down), no registration for private use (by definition non-interoperable).
- Whether the excluded-category vocabulary should graduate from free text to a controlled list. Free
  text is expressive and needs no maintainer; a controlled list is machine-actionable across
  implementations but needs one, which is exactly the centralising pressure §2.6's index rule exists
  to keep out of this specification. **Recommendation:** keep it free text; a controlled category
  list is a maintainer-shaped role the design should not create without a demonstrated need.
- Whether tax treatment categories (§11.3.5) can be enumerated at all before jurisdiction is
  researched. §11.3.5 commits to the field; populating it with real category values may have to wait
  on work §21.11 records as unresearched. **Recommendation:** do not enumerate values in this
  document until the research lands; the field's §16 slot is itself still a logged open item (§5.10).
- Whether TRACT registers a media type and/or reserves a well-known URI (§17.7). **Recommendation:**
  neither in v0; if a later transport forces the question, one `application/tract+cbor` type,
  family-by-parameter, and no well-known URI.
