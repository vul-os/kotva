# 13. Analytics

> **Drafting status — partially normative.** The *posture* (the four-stage disclosure tiering,
> §13.2), the *semantic* requirements on the disclosure grant and its revocation (§13.3), and the
> structural rule that the aggregate telemetry object carries no visitor identifier (§13.4) are now
> **normative**. The RFC 2119 key words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are used with
> their BCP 14 (RFC 2119, RFC 8174) meaning **only** where the design is settled.
>
> **What is still scoped, not normative.** Two classes of thing below carry no wire-level MUSTs and
> say why. (a) **The byte encoding of the grant and its revocation is PROVISIONAL.** The frozen v0
> grammar (`16-wire-format.md`) defines **no** `Grant` or `GrantRevocation` object; these are
> sealed-family objects with no CDDL yet, so this section specifies their *meaning* normatively and
> flags every byte-level claim as pending a §16 change (a MAJOR version bump — recorded, never
> invented here). (b) **Five substantive questions remain open** (§13.8) — the non-linkability proof
> for the per-store subject key, the `GRANT_LIFETIME` value for analytics, formal noise on top of
> bucketing, revocation's obligation over already-folded counts, and a machine-readable purpose
> field. None is decided here.
>
> **Honesty (do not read as defended).** This whole section is **unevidenced** (§13.7): three
> research passes targeting privacy-preserving analytics and fraud returned **nothing verified**
> (§21.1, §21.10). It MUST NOT cite §21 as support. Separately, the claim that a per-store
> pseudonymous key resists cross-store linkage holds only in the absence of a dominant correlating
> **index** — and index re-centralisation is one of the two weakest load-bearing claims in the whole
> document (§2.6, §21.3/§21.9), **marked as weakest and not defended**.

## 13.1 Scope

Measurement for sellers that does not require per-visitor surveillance: what a seller may learn
about traffic to their store, and by what mechanism, without a hosted analytics platform sitting
between them and every visitor.

This section defines **no new cryptography, no new identity construction, and no new transport**.
The per-store pseudonymous key of §13.3 is a substrate subkey
(`github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md`, and §1.5/§10.2 for the per-subject
subkey pattern this reuses); anonymous catalogue reads are public-object pulls over feeds and blobs
(`github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md`).

## 13.2 Posture (normative)

Tiered and buyer-granted, with an aggregate floor. There are four stages, each with a different
default — and the default is the point, since a merchant cannot be handed more than the stage it
sits in discloses.

| Stage | Default | Why |
|---|---|---|
| **Browse** | anonymous | catalogue reads are pulls of public objects (§2); a fetch against a signed feed carries no session for an identity to attach to |
| **Grant** | opt-in, scoped, revocable, expiring | the buyer's node decides what to disclose and to which store; a seller cannot be handed more than the grant states |
| **Order** | full detail | the seller needs an address to fulfil (§4), and the buyer knowingly provides it as part of placing the order (§7) |
| **Aggregate** | counts, with no visitor-level record | folded before it leaves the point of collection; the object type has no field capable of holding one (§13.4) |

The stages are enforced, not advisory:

- **Browse.** A catalogue read MUST be serveable as an anonymous pull of public objects. An
  implementation MUST NOT condition catalogue access on the reader disclosing a persistent identity,
  a session, or any grant. Whatever a seller learns at this stage is bounded by what the substrate's
  feed/blob fetch itself reveals (transport metadata, governed by `ROLES.md`), not by anything TRACT
  adds.
- **Grant.** Any disclosure beyond an anonymous browse MUST be carried by an explicit,
  buyer-issued **grant** (§13.3): opt-in, scoped to one store, revocable, and time-bounded. A seller
  MUST NOT record, infer, or act on a disclosure wider than the grant enumerates.
- **Order.** At order placement the buyer discloses fulfilment-necessary detail — name, address,
  contact — inside the **sealed** `Order` (§7, `16-wire-format.md §16.6`). This detail lives in the
  sealed family and has no public production at all (§0.5.1, §16.4).
- **Aggregate.** Telemetry a seller derives for its own use MUST be folded to counts with no
  visitor-level record (§13.4).

Both grants (§13.3) and orders (§7) are **sealed**, never published (§0.5, §0.5.1): a disclosure a
buyer chooses to make to one seller is not a public object any node can read, and it is not the
seller's to republish. An implementation MUST NOT publish, plaintext-address, or serve a grant or an
order.

## 13.3 The disclosure grant (semantics normative; bytes PROVISIONAL)

A grant is a small signed object a buyer's node sends directly to one seller — scoped to a single
store, carrying an enumerated field list, and time-bounded — rather than a blanket permission the
seller then interprets. It is a **sealed** object.

> **PROVISIONAL — pending a §16 change.** The frozen v0 grammar defines no `Grant` object. The field
> semantics below are normative; their CBOR encoding is not specified until a sealed-family `Grant`
> production is added to `16-wire-format.md §16.6` (a MAJOR version bump, recorded in
> `grammar_changes_needed`, not invented here). Until it lands, no byte-level conformance vector for
> this object can exist (§15).

| Field | Carries | Note |
|---|---|---|
| `store` | the seller's public key the grant is scoped to | one store per grant; a grant issued to one seller discloses nothing to another — there is no field meaning "all stores" |
| `subject` | a per-store pseudonymous key, distinct from the buyer's persistent identity | reused across visits to *this* store only, so a seller can recognise a returning visitor without the grant doubling as a cross-store identifier (§13.6) |
| `fields` | which of {coarse geography, referrer class, session-continuity} the buyer discloses | enumerated; a field absent from the list is not disclosed, not disclosed-as-empty |
| `issued_at` / `expires_at` | timestamps (`ts`, ms since epoch, §16.3) | defaults to `GRANT_LIFETIME` (24 h, §19.6) unless the buyer's node states a longer value explicitly |
| `signature` | over the above, by the subject key | proves the grant came from whoever holds that pseudonymous key, not from a seller recording consent after the fact |

Normatively:

- A grant MUST name exactly **one** `store` (one seller `identity-key`, §16.3). No value meaning
  "all stores" exists; a grant issued to one seller MUST NOT be relied on by another.
- `subject` MUST be a **per-store pseudonymous key** distinct from the buyer's root `IK`
  (`IDENTITY.md`; the per-subject subkey pattern of §10.2). A seller MAY use it to recognise a
  returning visitor to *its own* store; the construction's resistance to *cross-store* correlation
  is **not proven here** and is qualified in §13.6 and §13.8.
- `fields` MUST be an enumerated list. A field absent from `fields` MUST be treated as **not
  disclosed** — never as disclosed-with-an-empty-value. A seller MUST NOT record, infer, or derive a
  field the grant does not list.
- A grant MUST be time-bounded. Where the buyer's node does not set `expires_at` explicitly,
  `expires_at` MUST be taken as `issued_at + GRANT_LIFETIME` (§19.6). A grant is never unbounded:
  unlimited-duration consent is not expressible.
- `signature` MUST be produced by the `subject` key over the grant's canonical contents.

**Expiry is not a courtesy.** A grant with no expiry is a permission that can only be withdrawn by an
affirmative act, and a buyer who forgets, moves on, or replaces the device that issued it never
performs that act. `GRANT_LIFETIME` makes silence the buyer's default rather than the seller's — the
same asymmetry §18.3 states for order timeouts, applied to consent rather than money: a grant that
never expires is consent that cannot be withdrawn by inaction.

**Revocation** is a further signed object naming the grant it withdraws, sent the same way (sealed,
directly to the one seller). On receipt, a seller MUST cease relying on the named grant from that
point forward. It does not, and structurally cannot, unwind an aggregate count the grant already
contributed to before revocation (§13.8); whether it *should be obliged to* is an open question
(§13.8), not settled here.

> **PROVISIONAL — pending a §16 change.** As with the grant, the frozen v0 grammar defines no
> `GrantRevocation` object. Its meaning (name the grant, cease reliance on receipt) is normative; its
> encoding awaits a sealed-family production in `16-wire-format.md §16.6`.

## 13.4 The aggregate telemetry object (normative; local, not a wire object)

What a seller's own node folds its request log and its received grants into, before treating the
result as a number to look at rather than a record to query per visitor. This object is **local to
the seller's node**: TRACT defines **no** wire production for it, and nothing here specifies
publishing it anywhere. It is therefore not a §16 grammar object.

| Field | Carries |
|---|---|
| `window` | a bucketed time period (hour or day) — never a timestamp finer than the bucket |
| `stage_counts` | funnel-stage counts (viewed / carted / ordered) for the window |
| `geography` | counts by country or region (ISO 3166-1, §16.3), never finer than the seller declares |
| `referrer_class` | counts by coarse referrer category, not a full referrer URL |

Normatively, for any implementation that derives such telemetry:

- The object MUST NOT contain a visitor-identifier field. This is a **structural** absence, not an
  unpopulated field: a field that is merely left empty can be filled in later by a different code
  path; a field absent from the type cannot. An implementation MUST NOT extend the object with one.
- `window` MUST be a bucketed period no finer than one hour, and MUST NOT carry a timestamp finer
  than the bucket.
- `geography` MUST use ISO 3166-1 codes and MUST NOT be finer than the granularity the seller
  declares.
- `referrer_class` MUST be a coarse category, never a full referrer URL.
- The object MUST NOT be published to the public quadrant (§0.5.1). By construction it carries no
  personal data, but TRACT still specifies no publication of it; a seller keeps it locally.

## 13.5 What a seller genuinely loses (honest limit)

Stated plainly, because a section claiming parity with hosted analytics platforms would be false.
This subsection is design reasoning, not a measured result (§13.7, §21.1):

- **No cross-site retargeting.** There is no shared identifier to follow one visitor between stores —
  the pseudonymous subject key of §13.3 is scoped to a single store by construction.
- **Campaign-level, not person-level, attribution.** A seller learns which channel a cohort arrived
  from, not which visitor did what before converting.
- **No individual session replay.** Nothing records one visitor's path through a store; there is no
  object type that could be queried for it.
- **Coarser fraud signals on non-reversible rails.** §13.6.
- **Noisier numbers for small sellers.** Aggregation needs volume to be both private and accurate; a
  store with ten visitors a day gets a bucketed count that swings hard on small changes. This cost is
  real, and it falls hardest on exactly the sellers this design exists to serve — a one-person store
  cannot buy its way to a bigger sample the way a hosted platform's shared infrastructure effectively
  lets its tenants do.

What a seller keeps: funnel counts, traffic sources at campaign granularity, geography sufficient for
shipping and tax, product-level performance, and full order detail for customers who actually bought.

## 13.6 Fraud without raw IP (honest limit)

Most of what a raw IP address is used for in fraud scoring is a proxy for three separable questions.
This mapping is design reasoning, not a verified fraud result (§13.7, §21.1):

| Question | Answered by | Without identity because |
|---|---|---|
| Is this a bot? | anonymous rate-limiting credentials (anti-abuse, §14) | a visitor proves they are within a rate budget without revealing who they are |
| Is this the same actor as last time? | the per-store pseudonymous key (§13.3) | linkage is *intended* to hold within one store and not cross to another — an intention this document has not proven against an adversary who also runs an index (§13.8, §21.3) |
| Is this location consistent with the payment method? | the payment provider (§9) | this is data the provider already holds to run its own rail; the protocol does not duplicate it |

The honest residual: a merchant on a final, non-reversible rail (`RailClass` 1, §16.5.4) with no
provider-side fraud tooling has weaker fraud protection than one settling behind a card network's
chargebacks and issuer risk scoring. That gap is real, nothing in this section closes it, and it is a
reason to choose the rail class deliberately (§9.3) rather than by default.

## 13.7 This section is unevidenced (honest limit)

Three research passes across this document's drafting targeted privacy-preserving analytics and fraud
specifically, and returned no verified findings on any of it (§21.1, §21.10). Everything above — the
grant shape, the aggregate object, the claim that per-store pseudonymous keys resist cross-store
linkage, the claim that small-seller noise is the dominant cost — is design reasoning: a shape argued
from the substrate's own properties, not a measured result, not a deployed precedent at this scale,
and not a citation of prior art shown to hold against an adversarial analyst. Read every fidelity
claim in this section as a hypothesis this document has not tested, not a finding it has confirmed.
**This section MUST NOT be cited as, and does not rest on, §21 as support** (§21.1, §21.9).

**What §13.3 and §13.6 intend to profile**, named as intentions rather than as evaluated fits:

- **Prio-style aggregation** — client-side secret-sharing so a collecting node learns only the
  aggregate, never an individual contribution — for `stage_counts` and `geography` in §13.4.
- **Privacy-preserving attribution work from the browser vendors** — the aggregate-and-noise pattern
  behind the various post-third-party-cookie attribution proposals — for campaign-level attribution
  without a cross-site identifier.
- **Anonymous credential schemes** for the rate-limiting credential of §13.6, so a visitor can prove
  eligibility without a name attached to the proof.
- **Oblivious HTTP (RFC 9458)** where IP-level unlinkability is wanted at the transport, ahead of
  whatever the application-layer grant already withholds.

None of these has been evaluated against this section's specific object shapes. Naming them states
what this section intends to reuse rather than invent; it is not a claim that the fit has been
checked.

## 13.8 Open

Each item below is an open decision recorded for the founder-decision list (Brief §5). None is
settled here; the settled parts are in §13.2–§13.4.

- **Per-store subject-key non-linkability (weakest-claim territory, do not read as defended).**
  Whether the per-store pseudonymous subject key (§13.3) needs a proof of non-linkability beyond "it
  is a different key per store." A naive derivation (store key plus a buyer secret, hashed) could
  still be correlated by a seller who **also runs, or buys data from, an index** (§2.6). Index
  dominance becoming a de facto correlation point is the same re-centralisation risk that is one of
  the two weakest load-bearing claims in the whole document (§2.6, §21.3/§21.9) — **kept marked as
  weakest, not defended.**
- **`GRANT_LIFETIME` for analytics.** Whether the 24-hour default (§19.6) is the right value for
  analytics specifically, or was carried over from the same round-number instinct that set
  `FUND_TIMEOUT` to 24 hours for an unrelated reason.
- **Formal noise on top of bucketing.** Whether aggregate telemetry (§13.4) needs formal noise
  (e.g. differential privacy) on top of bucketing, or coarse buckets are sufficient protection at the
  traffic volumes real stores see — unresolved because those volumes are themselves unmeasured
  (§13.5, §21.1).
- **Revocation vs already-folded counts.** Whether a revoked grant obliges a seller to remove its
  already-folded contribution to a past aggregate, given that folding is one-way by construction and
  "stop disclosing" may not be the same operation as "undisclose."
- **A machine-readable purpose field.** Whether the grant object needs a machine-readable purpose
  field (marketing, fraud, fulfilment-adjacent measurement) so a buyer's node can grant narrowly, or
  whether `fields` alone is granular enough.
