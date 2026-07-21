# 13. Analytics

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 13.1 Scope

Measurement for sellers that does not require per-visitor surveillance: what a seller may learn
about traffic to their store, and by what mechanism, without a hosted analytics platform sitting
between them and every visitor.

## 13.2 Posture

Tiered and buyer-granted, with an aggregate floor. Four stages, each with a different default —
and the default is the point, since a merchant cannot be handed more than the stage it sits in
discloses:

| Stage | Default | Why |
|---|---|---|
| **Browse** | anonymous | catalogue reads are pulls of public objects (§2); a fetch against a signed feed carries no session for an identity to attach to |
| **Grant** | opt-in, scoped, revocable, expiring | the buyer's node decides what to disclose and to which store; a seller cannot be handed more than the grant states |
| **Order** | full detail | the seller needs an address to fulfil (§4), and the buyer knowingly provides it as part of placing the order (§7) |
| **Aggregate** | counts, with no visitor-level record | folded before it leaves the point of collection; the object type has no field capable of holding one (§13.4) |

Both grants (§13.3) and orders (§7) are sealed, never published (§0.5): a disclosure a buyer
chooses to make to one seller is not a public object any node can read, and it is not the
seller's to republish.

## 13.3 The grant object

A grant is a small signed object a buyer's node sends directly to one seller — scoped to a
single store, carrying an enumerated field list, and time-bounded — rather than a blanket
permission the seller then interprets.

| Field | Carries | Note |
|---|---|---|
| `store` | the seller's public key the grant is scoped to | one store per grant; a grant issued to one seller discloses nothing to another — there is no field meaning "all stores" |
| `subject` | a per-store pseudonymous key, distinct from the buyer's persistent identity | reused across visits to *this* store only, so a seller can recognise a returning visitor without the grant doubling as a cross-store identifier (§13.6) |
| `fields` | which of {coarse geography, referrer, session-continuity} the buyer discloses | enumerated; a field absent from the list is not disclosed, not disclosed-as-empty |
| `issued_at` / `expires_at` | timestamps | defaults to `GRANT_LIFETIME` (24 hours, §19.6) unless the buyer's node states a longer value explicitly |
| `signature` | over the above, by the subject key | proves the grant came from whoever holds that pseudonymous key, not from a seller recording consent after the fact |

**Expiry is not a courtesy.** A grant with no expiry is a permission that can only be withdrawn
by an affirmative act, and a buyer who forgets, moves on, or replaces the device that issued it
never performs that act. `GRANT_LIFETIME` makes silence the buyer's default rather than the
seller's — the same asymmetry §18.3 states for order timeouts, applied to consent rather than
money: a grant that never expires is consent that cannot be withdrawn by inaction.

**Revocation** is a further signed object naming the grant it withdraws, sent the same way. A
seller that receives one stops relying on the grant from that point forward; it does not, and
structurally cannot, unwind an aggregate count the grant already contributed to before
revocation (§13.8).

## 13.4 The aggregate telemetry object

What a seller's own node folds its request log and its received grants into before treating the
result as a number to look at, rather than a record to query per visitor. This object stays on
the seller's own node; nothing here specifies publishing it anywhere.

| Field | Carries |
|---|---|
| `window` | a bucketed time period (hour or day) — never a timestamp finer than the bucket |
| `stage_counts` | funnel-stage counts (viewed / carted / ordered) for the window |
| `geography` | counts by country or region (ISO 3166-1), never finer than the seller declares |
| `referrer_class` | counts by coarse referrer category, not a full referrer URL |

There is no visitor-identifier field in this object's definition — not one left empty, one that
does not exist. A field that is merely unpopulated can be filled in later by a different code
path; a field absent from the type cannot.

## 13.5 What a seller genuinely loses

Stated plainly, because a section claiming parity with hosted analytics platforms would be
false:

- **No cross-site retargeting.** There is no shared identifier to follow one visitor between
  stores — the pseudonymous subject key of §13.3 is scoped to a single store by construction.
- **Campaign-level, not person-level, attribution.** A seller learns which channel a cohort
  arrived from, not which visitor did what before converting.
- **No individual session replay.** Nothing records one visitor's path through a store; there is
  no object type that could be queried for it.
- **Coarser fraud signals on non-reversible rails.** §13.6.
- **Noisier numbers for small sellers.** Aggregation needs volume to be both private and
  accurate; a store with ten visitors a day gets a bucketed count that swings hard on small
  changes. This cost is real, and it falls hardest on exactly the sellers this design exists to
  serve — a one-person store cannot buy its way to a bigger sample the way a hosted platform's
  shared infrastructure effectively lets its tenants do.

What a seller keeps: funnel counts, traffic sources at campaign granularity, geography
sufficient for shipping and tax, product-level performance, and full order detail for customers
who actually bought.

## 13.6 Fraud without raw IP

Most of what a raw IP address is used for in fraud scoring is a proxy for three separable
questions:

| Question | Answered by | Without identity because |
|---|---|---|
| Is this a bot? | anonymous rate-limiting credentials | a visitor proves they are within a rate budget without revealing who they are |
| Is this the same actor as last time? | the per-store pseudonymous key (§13.3) | linkage holds within one store and does not cross to any other, so it cannot become a cross-store tracking identifier by accretion |
| Is this location consistent with the payment method? | the payment provider | this is data the provider already holds to run its own rail; the protocol does not duplicate it |

The honest residual: a merchant on a final, non-reversible rail with no provider-side fraud
tooling has weaker fraud protection than one settling behind a card network's chargebacks and
issuer risk scoring. That gap is real, nothing in this section closes it, and it is a reason to
choose the rail class deliberately (§9.3) rather than by default.

## 13.7 This section is unevidenced

Three research passes across this document's drafting targeted privacy-preserving analytics and
fraud specifically, and returned no verified findings on any of it. Everything above — the grant
shape, the aggregate object, the claim that per-store pseudonymous keys resist cross-store
linkage, the claim that small-seller noise is the dominant cost — is design reasoning: a shape
argued from the substrate's own properties, not a measured result, not a deployed precedent at
this scale, and not a citation of prior art shown to hold against an adversarial analyst. Read
every fidelity claim in this section as a hypothesis this document has not tested, not a finding
it has confirmed.

**What §13.3 and §13.6 intend to profile**, named as intentions rather than as evaluated fits:

- **Prio-style aggregation** — client-side secret-sharing so a collecting node learns only the
  aggregate, never an individual contribution — for `stage_counts` and `geography` in §13.4.
- **Privacy-preserving attribution work from the browser vendors** — the aggregate-and-noise
  pattern behind the various post-third-party-cookie attribution proposals — for campaign-level
  attribution without a cross-site identifier.
- **Anonymous credential schemes** for the rate-limiting credential of §13.6, so a visitor can
  prove eligibility without a name attached to the proof.
- **Oblivious HTTP (RFC 9458)** where IP-level unlinkability is wanted at the transport, ahead of
  whatever the application-layer grant already withholds.

None of these has been evaluated against this section's specific object shapes. Naming them
states what this section intends to reuse rather than invent; it is not a claim that the fit has
been checked.

## 13.8 Open

- Whether the per-store pseudonymous subject key (§13.3) needs a proof of non-linkability beyond
  "it is a different key per store" — a naive derivation (store key plus a buyer secret, hashed)
  could still be correlated by a seller who also runs, or buys data from, an index (§2.6).
- Whether `GRANT_LIFETIME`'s 24-hour default (§19.6) is the right value for analytics
  specifically, or was carried over from the same round-number instinct that set `FUND_TIMEOUT`
  to 24 hours for an unrelated reason.
- Whether aggregate telemetry needs formal noise on top of bucketing, or coarse buckets are
  sufficient protection at the traffic volumes real stores see — unresolved because those
  volumes are themselves unmeasured (§13.5).
- Whether a revoked grant obliges a seller to remove its already-folded contribution to a past
  aggregate, given that folding is one-way by construction and "stop disclosing" may not be the
  same operation as "undisclose".
- Whether the grant object needs a machine-readable purpose field (marketing, fraud,
  fulfilment-adjacent measurement) so a buyer's node can grant narrowly, or whether `fields`
  alone is granular enough.
