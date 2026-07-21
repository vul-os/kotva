# 17. Errors & registries

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 17.1 Scope

The `ERR_TRACT_*` code block, the responder-action vocabulary, an initial error table, and the
extensible registries — everything a conformant implementation needs to name a failure the same
way another implementation would, without either one inventing its own vocabulary for it.

## 17.2 Code format and the subsystem table

Every code is a 16-bit value `0xSSNN`: `SS` is the subsystem byte, `NN` is the code point within
it — the layout the sibling DMTAP specification uses for its own registry (`21-errors-iana.md`
§21.1), reused here for form only. TRACT allocates its own subsystem bytes rather than sharing
DMTAP's table; a TRACT code and a DMTAP code that happen to carry the same `SSNN` value name
unrelated conditions in two different registries, never the same one.

This appendix introduces no new validation logic. Every code below traces to a rule some
numbered section already states in prose — §0.5.1, §2.4, §5.3, §6.2, §6.4, §8.2, §9.3, §9.4,
§10.2, §11.2, §12.3, §16.4 and §16.7 between them. Naming the rule and giving it a code does not
make the rule normative before its own section is; the whole table is as provisional as those
sections are, and expect renumbering as they firm up — not because the layout is arbitrary, but
because the rule underneath a code might still change shape before the section stating it does.

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

The grouping is not one byte per section number: §3 and §4 share a subsystem because the four
axes (§2.4) are inspected together at the same points (an offer is rejected or accepted as a
whole, never one axis at a time), and §13 and §15 have no subsystem byte yet because neither has
a checkable object shape to raise a code against.

## 17.3 Responder action vocabulary

Every code below resolves to exactly one of a closed set of five actions. The set is closed
deliberately — a sixth action invented ad hoc the day someone needs one is exactly how a
registry like this drifts into having as many behaviours as it has codes.

| Action | Meaning | When it applies |
|---|---|---|
| **fail-closed-block** | Refuse to proceed, and refuse to silently degrade into a lesser but still-plausible outcome. The object, order or trade stops here until the condition is fixed by the party who caused it. | The default for structural violations — a public object carrying personal data, a type-confused decode, an offer missing an axis — where continuing at all, in any reduced form, is the mistake. |
| **drop-silent** | Discard without notifying the sender; to the sender this is indistinguishable from packet loss. | Reserved for conditions where notifying would itself leak information the sender should not have. TRACT's own table has no entry using it yet — the substrate already disposes of the case that would need it (an unrecognised message reaching a node under the substrate's own cold-contact handling, §14.3), and TRACT does not re-litigate that disposition. |
| **rotate-retry** | An internal recovery action — try a different path, a different replica, a refreshed copy of an object — with no user-visible failure unless the retries are exhausted. | Transient conditions, not structural ones. A rate-card fetch failing over a flaky connection is a substrate-level reachability condition (inherited, not a TRACT code); TRACT's own conditions are checks against content already received, which is why none of §17.4's rows use it today. |
| **deny-policy** | Reject per a deliberate, non-adversarial policy decision: the object is well-formed and could succeed elsewhere, but this responder's own declared scope or standard does not cover it. | An `EscrowScope` intersection that comes up empty, or a review an index declines to serve for want of attestation — both are facts about this responder's stated terms, not defects in what was presented. |
| **halt-alert** | Halt the affected process and raise a visible alert. | Reserved for conditions that indicate tampering or equivocation, not ordinary refusal. TRACT's own table has none yet: every condition §17.4 currently names is a bounded, explainable refusal, not evidence of an attack. The substrate's own halt-alert conditions (a broken identity chain, a split-view key-transparency log) already apply beneath TRACT unchanged and are not reproduced here. |

## 17.4 Initial error table

| Code | Name | Operation | Meaning | Retryable | Action |
|---|---|---|---|:---:|---|
| `0x0101` | `ERR_TRACT_TYPE_CONFUSION` | decode of any object (§16.4) | a sealed object is presented under a public-object schema, or a public object under a sealed one | No | fail-closed-block |
| `0x0102` | `ERR_TRACT_PERSONAL_DATA_PUBLIC` | decode or publish of `ProductRecord`, `Offer`, `RateCard`, `CapacityRecord`, or a storefront render bundle (§0.5.1, §16.4) | a public-quadrant object carries a field that identifies, or is linkable to, a natural person | No | fail-closed-block |
| `0x0201` | `ERR_TRACT_OFFER_AXIS_MISSING` | `Offer` decode (§2.4, §16.5.2) | one or more of the four axes — Item, Availability, Fulfilment, Consideration — is absent | No | fail-closed-block |
| `0x0401` | `ERR_TRACT_CURRENCY_MISMATCH` | consideration or route-total arithmetic (§5.3, §16.7) | two `money` values of different currencies are combined without an explicit, disclosed conversion | Conditional — a disclosed conversion supplies a single currency | fail-closed-block |
| `0x0501` | `ERR_TRACT_OVERSELL_PREVENTED` | cart checkout against partitioned inventory (§6.2, §6.4) | a sale would exceed the combined quota remaining across a seller's replicas | No | fail-closed-block |
| `0x0701` | `ERR_TRACT_ROUTE_TOTAL_OVERFLOW` | route-total computation (§8.2, §16.7) | summing leg prices would exceed the representable range of a `money` minor-units integer | No | fail-closed-block |
| `0x0702` | `ERR_TRACT_RATE_CARD_STALE` | rate-card use for local quoting (§8.2, §19.6) | a `RateCard` older than `RATE_CARD_MAX_AGE` is used to compute a quote without disclosing its age | Yes — refetch a current card | fail-closed-block |
| `0x0801` | `ERR_TRACT_ESCROW_SCOPE_EMPTY` | checkout scope intersection (§9.4) | the buyer's declared `EscrowScope` and the gateway's offered scope intersect on no valid combination of country, currency, rail class, value ceiling or category | No | deny-policy |
| `0x0802` | `ERR_TRACT_RAIL_CLASS_SUBSTITUTED` | rail selection at checkout (§9.3) | a `CustodialReversible` rail is replaced with `NonCustodialFinal`, or the reverse, without a fresh agreement recorded on the order | No | fail-closed-block |
| `0x0901` | `ERR_TRACT_REVIEW_UNATTESTED` | `Review` intake by an index (§10.2, §16.5.5) | the review carries no valid `PurchaseAttestation` from the seller or an escrow operator | No | deny-policy |
| `0x0A01` | `ERR_TRACT_PLACE_OF_SUPPLY_UNRESOLVED` | tax-anchor derivation (§11.2, §16.5.2 `Fulfilment`) | the Fulfilment variant does not yet resolve to a single place of supply — most often a multi-variant offer whose buyer choice is not yet recorded on the order | Conditional — resolves once the buyer's choice is recorded | fail-closed-block |
| `0x0B01` | `ERR_TRACT_ORIGIN_NOT_ISOLATED` | render-bundle serving (§12.3) | two stores would be served from the same normalised origin | No | fail-closed-block |

Retryable means the same condition, presented again with nothing else changed, can succeed —
"Conditional" marks the cases where success depends on an action outside the responder's own
control (the buyer recording a choice, a fresher object being fetched), not on retrying the
identical request. `0x0401`'s and `0x0701`'s reasoning is the same reasoning, run in two
directions: a currency silently coerced, or a total silently wrapped, is a wrong number that
looks exactly like a right one, and by the time it is carried into a signed order there is no
downstream step that can tell the difference. Refusing both at the point of computation is
cheaper than auditing every order for one afterward.

## 17.5 Extensible registries

| Registry | What it enumerates | Where defined | Extension policy |
|---|---|---|---|
| Item kinds | product / variant-of-group / service / right-or-licence / capacity | §2, §16.5.2 `Item` | tolerant to store, strict to act (§17.6) |
| Availability variants | stock-signal / time-slots / capacity-per-interval / unlimited / made-to-order | §3, §16.5.2 `Availability` | tolerant to store, strict to act |
| Fulfilment variants | ship / collect / digital-grant / perform-at-place / perform-remote / access-grant / return-required | §4, §16.5.2 `Fulfilment` | tolerant to store, strict to act |
| Consideration variants | fixed / tiered / recurring / metered / deposit-and-balance / quote-required | §5, §16.5.2 `Consideration` | tolerant to store, strict to act |
| Rail classes | custodial-reversible / non-custodial-final | §9.3, §16.5.4 `RailClass` | a closed set of two, not an ordinary extensible registry — a third class is a specification change, because §9.3 makes the class part of what determines a buyer's recourse, and a registry addition here would be a substitution by another name |
| Tax treatment categories | — not yet enumerated | §11 (§11.3 commits to the field; its values are undecided) | tolerant to store, strict to act, once populated |
| Excluded-category vocabulary | free-text category strings a rate card, capacity record or escrow scope may exclude | §8, §9, §16.5.3, §16.5.4 | advisory strings today, not a controlled list — see §17.7 |
| External-identifier schemes | `gtin`, `mpn`, and others a publisher may claim | §2.3, §16.5.1 `ClaimedExternalRung` | advisory join keys only, never authority (§2.3); a scheme absent from this registry is preserved and surfaced as unknown, never treated as a match |

## 17.6 Tolerant to store, strict to act

Every registry above except rail classes is extensible, and an unrecognised value must not be
fatal to a generic index: the object is preserved, and the unrecognised field is surfaced as
unknown rather than causing the object to be dropped. An index that refused to hold an offer
merely because it used a Fulfilment variant from next year's registry update would make every
extension to this specification a backward-compatibility break for every index already deployed.

A client that *renders or transacts* against a variant it does not implement is a different
actor making a different decision, and it refuses rather than guesses. The two behaviours are
independently required because a case that only checked one could pass an implementation that
guesses exactly where it should refuse.

Guessing is worse than refusing here for the same reason a silently coerced currency or a
silently wrapped total is worse than a refusal (§17.4): a guess produces an answer that looks
exactly as valid as a correct one, and nothing downstream carries a marker saying it was a
guess. A client that renders an unrecognised Fulfilment variant as if it were `collect`, because
`collect` is the closest shape it understands, has just told a buyer a wrong place of supply
with the same confidence it would state a right one (§11.2) — and the buyer has no way to tell
the difference until a tax authority does. A client that refuses instead produces a visible
boundary: the buyer learns their client does not yet understand this offer, which is a true and
actionable fact, rather than a false and confident one.

## 17.7 Open

- Whether the registries of §17.5 live as tables in this document, or as a separate
  machine-readable file this document points to. A table reviewed alongside the prose that
  defines each variant is easier to keep honest; a separate file is easier for an implementation
  to consume without parsing markdown.
- The allocation policy for the `0x0D`–`0xEF` unassigned range and the `0xF0`–`0xFE` private-use
  range — Specification Required, first-come-first-served, or something else — is undecided.
  Mirroring DMTAP's policy is one option, not a commitment.
- Whether the excluded-category vocabulary should graduate from free text to a controlled list.
  Free text is expressive and needs no maintainer; a controlled list is machine-actionable across
  implementations but needs one, which is exactly the centralising pressure §2.6's index rule
  exists to keep out of this specification.
- Whether tax treatment categories can be enumerated at all before jurisdiction is researched.
  §11.3 commits to the field; populating it with real category values may have to wait on work
  this document does not yet consider done.
