# REPUTATION — portable, wash-resistant trust without a global score

> **Status:** draft, normative once ratified. One of the KOTVA **primitives**
> ([`DIRECTION § 2`](../DIRECTION.md)) — the small set every real service rearranges. This document
> owns the *generic* reputation shape and its rules; the two shipped profiles instantiate it
> ([`profiles/tract/10-trust.md`](../profiles/tract/10-trust.md) for commerce reviews,
> [`profiles/wrap/08-trust.md`](../profiles/wrap/08-trust.md) for provider reputation) and freeze
> the exact bytes. Reputation invents **no new cryptography and no new wire capability**: its objects
> are ordinary public author-feed objects ([`substrate/FEEDS.md`](../substrate/FEEDS.md), §22), its
> anchor and its compute are [bindings](../bindings/README.md), and its coordinator is an `indexer`
> under [`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md). Where this document and a
> normative-byte home (§16 of a profile, §22) disagree, the byte home governs.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are BCP 14 (RFC 2119 / RFC 8174).

---

## 1. Purpose

Reputation is the answer to *"should I trust this counterparty?"* computed **without an authority
that owns the answer**. A centralized platform bundles three things a user cannot separate: the
identities, the feedback graph, and the single number it publishes about you — and it owns all
three, so it can suppress, inflate, or sell them. REPUTATION unbundles them:

- **Trust is locally measured, or computed by a hired `indexer` — never a globally published
  number.** There is no `Reputation` object anywhere in KOTVA carrying a network-wide score
  ([`bindings/README.md`](../bindings/README.md) "does not bind"; [`DIRECTION § 5`](../DIRECTION.md);
  [`CONTRACT § 2.1`](../coordinator/CONTRACT.md)). What exists on the wire is only signed **input**:
  vouches and feedback attached to a **stable subject key**. The score is *derived data*, rebuildable
  by anyone, authoritative for no one.
- **Trust is portable and un-suppressible.** Feedback attaches to the subject's identity key, not to
  a storefront, so a subject carries its reputation across every index and every gateway and cannot
  delete an unfavourable entry by leaving a platform — there is no platform to leave
  (TRACT §10.2b).
- **Trust is anchored, so it is wash-resistant, not wash-proof.** The only defence against a fresh
  key with no history ("whitewashing") is to **discount unanchored keys**. The anchor is
  **proof-of-personhood** (World ID / Human Passport) or **staked existing value** — never a minted
  token (§5). Anchoring raises the Sybil-cost floor; it does not close it (§9).

The wedge is honesty about scale: at village scale reputation *is* web-of-trust and needs no
coordinator at all; at planet scale it binds to a Sybil-resistant compute service you choose and can
fire. The primitive is the same at both ends; only the trust anchor slides
([`DIRECTION § 6`](../DIRECTION.md)).

---

## 2. Objects it defines

Reputation defines **two signed public input objects** and, deliberately, **no output object**. Both
are public author-feed objects under §22 / [`FEEDS.md`](../substrate/FEEDS.md): signed in the clear
(authenticity, not confidentiality), content-addressed, servable over plain HTTPS, self-verifying
(§22.5). The CDDL below is a **generic sketch**; a profile freezes the exact integer-keyed encoding
(TRACT §16.5.5 is the reference freeze). Both inherit §22's map conventions and its origination floor
(§22.3.3 step 1a).

### 2.1 The trust-edge attestation — the web-of-trust primitive (an ATTEST claim, no wire object)

A directed, signed vouch from a **truster** to a **subject**, in a **context**. It is
trust-matrix input to the adopted EigenTrust/OpenRank compute (§5) — with the transitivity and
polarity caveats disclosed in §9.

REPUTATION mints **no wire object of its own** for a trust edge. A profile MAY define a schema
and constrain fields; it MUST NOT invent a parallel attestation object (the waist adoption rule,
[`ATTEST.md`](ATTEST.md) §2, `substrate/README` §3.2) — so a trust edge **is** an ATTEST
`Attestation` ([`ATTEST.md`](ATTEST.md) §2), carried on the **public** carrier (`det_cbor(Attestation)`
embedded in a `PubAnnounce`, ATTEST §2), constrained by a registered **trust-edge schema**:

| retired `TrustEdge` field | `Attestation` field (ATTEST §2) | mapping |
|---|---|---|
| `subject` (ik-pub) | `subject` (key 4) | the vouched-for identity key — unchanged: stable, operator-independent |
| `context` (tstr) | `schema` (key 5) | a registered trust-context schema URI/UID, e.g. `kotva-reputation/trust-edge/commerce/v0`, `…/courier/v0`, `…/code-review/v0`; a new trust context is a **new schema registration**, never a new wire object |
| `polarity` + `evidence` | `claim` (key 6) | the claim body: `{ polarity: -1/0/+1 (a sign, NOT a magnitude, §2.3), ? evidence: hash of a backing ATTEST/PurchaseAttestation }` |
| — (feed author key) | `issuer` (key 3) | the vouching key; MUST equal the carrier's authenticated identity (`PubAnnounce.pub`, ATTEST §2) |
| `ts` | `ts` (key 7) | display/ordering only; feed seq is authoritative (§22.3.1) |
| `supersedes` | *(no `Attestation` field — rides the carrier's `PubAnnounce.supersedes`)* | retract = supersede at the carrier level (ATTEST §4.3), never a claim field — ATTEST retired its own key-9 `supersedes` for the identical reason (ATTEST §2) |

The **author** (the truster) is, as before, not a claim field: it is the `PubAnnounce`'s
authenticated `pub`, which MUST equal `Attestation.issuer` (ATTEST §2) and **MUST be a
per-context pseudonymous subkey, MUST NOT be the root `IK`** (REP-2, §3; the §1.5 grant-tier
construction). A trust-edge attestation names its own author only through the carrier that signs
it.

Nothing that guaranteed portability, wash-resistance, or the no-score-object rule (§2.3) depends
on the retired bespoke shape: those are properties of the *subject-keyed, signed, unauthoritative*
pattern, which `Attestation` already provides natively. REPUTATION now names a **schema** over
ATTEST rather than a competing wire object; no guarantee is weakened.

### 2.2 `Feedback` — subject-attached rating (a REVIEW/attestation input)

A public rating attached to a subject, optionally bound to a real interaction. This is the
family-generic form of TRACT's frozen `Review` (§16.5.5); a profile MUST encode its own frozen shape
rather than re-deriving these bytes.

```cddl
Feedback = {
  1 => subject,       ; subject   one of: content-address (an OFFER/product) OR ik-pub (a party)
  2 => uint,          ; score     0..5 ordinal; the protocol assigns NO meaning beyond order (§2.3)
  3 => tstr,          ; body      bounded free text (profile MAX; the one natural-person public field)
  ? 4 => hash,        ; proof     OPTIONAL content-address of a PurchaseAttestation / ATTEST (ATTEST.md §2)
  5 => ts,            ; ts        display/ordering only
  ? 6 => hash,        ; supersedes prior Feedback this revises (§2.4)
}
subject = ik-pub / hash
```

### 2.3 The non-object: there is no score object (normative)

**No KOTVA object carries an aggregated, network-wide, or per-subject *reputation number*.** A
`Feedback` carries a per-interaction `score`; a trust-edge attestation carries a per-edge `polarity`. Neither is
a reputation. **Aggregation is forbidden as an object and mandated as a derived computation** (§5):
publishing a single authoritative number requires a party that ranks every subject — exactly the
operator this primitive removes (TRACT §10.3). `polarity` is a sign, not a weight, precisely so that
no publisher can encode "how much" trust into the object and invite it to be read as a score; *how
much* is the reader's or the indexer's judgement, never the author's assertion.

### 2.4 Retraction is supersession, never deletion

An author retracts a trust-edge attestation or `Feedback` by publishing a later feed entry that
supersedes it (§22.3.4; for the trust-edge attestation this is the carrier's `PubAnnounce.supersedes`,
ATTEST §4.3). There is no tombstone byte and no delete. Retraction is **cooperative-only**: it binds a
reader who checks the author's feed head before displaying, and nothing obliges an independent
byte-holder or a pre-snapshot index to notice it (TRACT §10.2d). A client **MUST disclose
irrevocability to the author before the object is published** (§8, SEC-9 residual; TRACT §10.2).

---

## 3. Normative rules

- **REP-1 — Subject identifier is stable and operator-independent.** Every reputation object MUST
  name its subject by a stable identifier that is the same at every index and every gateway: a
  content-address (an OFFER / product) or an `identity-key` (a party). It MUST NOT name a subject by
  a storefront, an account handle, or any operator-scoped id. Portability is a consequence of this
  rule, not of any bespoke mechanism (TRACT §10.2b).
- **REP-2 — Author is a per-context pseudonymous subkey, never the root `IK`.** A truster/reviewer
  MUST sign with a per-subject-or-per-context subkey (§1.5), so two subjects cannot join an author's
  history across them by key, and only the author (holding the root key or the stored mapping) can.
  A profile MUST NOT require the root `IK` to sign reputation input.
- **REP-3 — No published score (the prohibition).** No object this primitive or any profile defines
  MAY carry an aggregated reputation number (§2.3). A conformant encoder MUST reject any attempt to
  add one; a network-wide score is a conformance violation, not a feature.
- **REP-4 — Aggregation is derived and non-authoritative.** Any node MAY compute any aggregate,
  weighting, or ranking over the public reputation feeds. The result is rebuildable and MUST NOT be
  treated as authoritative; the authoritative state is always the set of signed feeds. Two indexes
  reaching different answers is **the design, not a defect** (TRACT §10.3, §21.9).
- **REP-5 — Anchoring is disclosed, and unanchored keys are discountable.** A reputation object MAY
  be anchored by a personhood or stake ATTEST (§5); an index or a local computation MAY discount an
  unanchored author. The anchor and its assurance level MUST be surfaced, never presented as closing
  the Sybil gap (§9). Discounting a fresh, unanchored key is the *only* whitewashing defence and MUST
  be available; it MUST NOT be sold as a solution.
- **REP-6 — Feedback binds to evidence where it claims to.** Where a `Feedback` or a trust-edge
  attestation asserts it reflects a real interaction, it MUST carry the content-address of an ATTEST /
  `PurchaseAttestation` (ATTEST.md §2; TRACT §10.2a/§16.5.5) that a verifier can independently
  check. An index MAY gate on the presence of such a proof (`deny-policy`, TRACT §10.2a); whether
  it does is **index-local policy, never a protocol mandate** — mandating it network-wide would
  require the authority REP-3 removes.
- **REP-7 — Ranking policy is local, and disclosed.** An index that ranks MUST publish its weighting
  policy in its signed descriptor ([`CONTRACT § 2.1`](../coordinator/CONTRACT.md)) so a reader knows
  which discounting, anchoring, and web-of-trust measures produced the order. A reader MUST be able to
  substitute its own local computation for the index's (§6).
- **REP-8 — Reputation is shared across profiles, not bridged.** Because sellers, couriers, workers,
  and reviewers are all just `IK`s on one substrate (§1.2), a subject's reputation is a property of
  its key across TRACT, WRAP, and every profile — one shared, unsolved Sybil surface, never two
  separately-solved ones (TRACT §10.2b, §10.3a; WRAP §9). A profile MUST NOT redefine another
  profile's reputation record; it names its own input objects and shares the key.

---

## 4. Composition with the other primitives

Reputation is a **consumer**, not a foundation — it reads what the other primitives already sign.

- **IDENTITY** — the subject key and the author subkey are ordinary substrate identities
  ([`substrate/IDENTITY.md`](../substrate/IDENTITY.md), §1). Portability (REP-1) and pseudonymous
  authorship (REP-2) are IDENTITY properties, not new ones.
- **ATTEST** ([`ATTEST.md`](ATTEST.md)) — the trust-edge vouch is not a REPUTATION wire object at
  all: it **is** an ATTEST `Attestation` constrained by the trust-edge schema (an issuer-signed
  directed claim about a subject, narrowed to trust polarity, §2.1); the personhood and stake anchors
  (§5) are ATTEST objects; the `proof` binding a `Feedback` to a real interaction is an ATTEST /
  `PurchaseAttestation`. Reputation is largely ATTEST rearranged around a subject.
- **OFFER** ([`OFFER.md`](OFFER.md)) — a product/listing content-address is a valid `Feedback`
  subject; "how good is this offer/seller" is reputation computed over OFFER subjects.
- **ESCROW / DISPUTE** — an escrow-issued `PurchaseAttestation` is the strongest `Feedback.proof`
  (independent of the party being reviewed), and a `DISPUTE` outcome is a high-weight input an index
  MAY favour. Both are hired coordinators, never load-bearing (TRACT §9, §10.2a).
- **FEEDS / PUB** ([`substrate/FEEDS.md`](../substrate/FEEDS.md), §22) — the carrier. Reputation adds
  no signing, addressing, or feed construction; a subject cannot suppress feedback because reviews are
  ordinary public author-feed objects any node may gather without permission (TRACT §10.2b).

---

## 5. Binding adopted

Per [`DIRECTION § 3`](../DIRECTION.md) / [`bindings/README.md`](../bindings/README.md), reputation
binds rather than reinvents. Two slots, both swappable:

- **Sybil-resistant compute — OpenRank (EigenTrust, TEE-verified).** The global-view computation over
  the trust/attestation graph binds to [OpenRank](../bindings/README.md) (the *"adopt (evolving)"*
  row): context-specific EigenTrust, optionally run inside a TEE so the `indexer` holds a global view
  **without** being able to forge the result — the `attested` assurance level
  ([`CONTRACT § 3.3`](../coordinator/CONTRACT.md)), disclosed as chip-vendor-trust, never trustless.
  KOTVA computes **no** reputation of its own; it feeds signed edges to an adopted engine — an
  engine whose transitivity these objects only partly deliver, and whose polarity handling drops
  distrust; both disclosed in §9.
- **Anchor — proof-of-personhood or stake.** The Sybil anchor binds to World ID / Human Passport (the
  *"adopt (ceiling)"* row) **or** to staked existing value ([`DIRECTION § 5`](../DIRECTION.md), sized
  to value at risk). A protocol token is **forbidden** as an anchor — it is either a financing scheme
  or a coordination problem a token cannot solve ([`bindings`](../bindings/README.md) "does not
  bind").

When the frontier improves (better personhood, better TEE), the filling swaps as a binding version
bump; §2's objects, §3's rules, and the profiles do not change
([`DIRECTION § 9`](../DIRECTION.md), "future-proof by seams").

---

## 6. Scale-invariance — mesh web-of-trust ⇆ global coordinator

The objects and rules are identical at every scale; only the **trust anchor** slides
([`DIRECTION § 6`](../DIRECTION.md) table). A reader always computes reputation as a function it
controls; the difference is only *whose* edges it weighs and *who* runs the sum.

| | Small / mesh (no coordinator) | Global (hired coordinator) |
|---|---|---|
| **Anchor** | web-of-trust: trust-edge attestations from keys you already pin | a personhood attester you choose (§5) |
| **Compute** | local EigenTrust over the edges you hold, on your device | an `indexer` running OpenRank/TEE over public feeds |
| **Subject id** | same stable key (REP-1) | same stable key (REP-1) |
| **Authority** | you | still you — the index is derived, fireable (REP-4/REP-7) |

The mesh form is the **fallback the design collapses to**, not a lesser mode: remove connectivity and
reputation is exactly the transitive vouch graph you can reach, computed locally. Add connectivity and
a global indexer becomes *available* for reach over strangers — never *required* to trust the people
you already know. This is the difference between REPUTATION and every platform score: the platform's
number is authoritative and yours is not; here yours is authoritative and the index's is a convenience
([`CONTRACT § 2.1`](../coordinator/CONTRACT.md), locally-measured reputation).

---

## 7. Offline / apocalypse behaviour + reconcile

Reputation inherits the substrate's offline profile ([`substrate/OFFLINE.md`](../substrate/OFFLINE.md)):

- **Authoring feedback is `full` offline.** Signing a trust-edge attestation / `Feedback` and appending it to
  your own feed needs no network (OFFLINE §3.3). **Distribution** to the swarm is `deferred`.
- **Local computation is `full`; global indexer view is `local-trust`.** Computing reputation over the
  feeds you already hold is `full`; an index's global aggregate is a `local-trust` view whose staleness
  offline is never a correctness failure — indexes are derived and rebuildable (REP-4, OFFLINE §3.3).
- **The personhood anchor is `blocked` offline.** A *fresh* personhood proof needs a live attester and
  fails closed offline; a *previously issued* personhood/stake ATTEST already held is `full` — it is a
  signed object like any other (OFFLINE §3.1). A profile MUST NOT gate offline trust computation on a
  fresh proof it cannot obtain.
- **Reconcile is feed catch-up.** On reconnect a reader fetches feed entries newer than its last-seen
  `seq` and walks the `prev` chain (OFFLINE §4; §22.4.2). Reconcile is idempotent (content-address
  dedup). A **forked author feed** — two histories under one key — is a **detectable equivocation
  surfaced for dispute, never swallowed by a merge** (OFFLINE R-REC-2; §22.4.2 `HALT_ALERT`): a
  publisher cannot honestly present two reputations, and a reader that sees both holds transferable
  evidence.

An action that cannot proceed offline (a fresh anchor) MUST be marked `blocked` and say why, never
silently degraded to unanchored (OFFLINE R-GRADE-1).

---

## 8. Security MUSTs

Reputation inherits [`THREAT-MODEL.md`](../THREAT-MODEL.md) unchanged; the invariants that bite here:

- **SEC-2 (intrinsic authenticity).** Every reputation object is self-authenticating (signed,
  content-addressed) and verifies identically over mesh, HTTPS, or SD card, trusting no server. A
  malicious index can **withhold or stall** (detectable via `seq`/chain discontinuity, §22.4.3) but
  can **never forge** an edge (that needs the author's key) or **hide** one without a reader noticing.
- **SEC-6 (authorize, never classify).** An `indexer` is a hired coordinator: swappable with zero
  migration, self-hostable, never load-bearing, and it **authorizes, it does not classify** — it MAY
  weight and rank as disclosed policy (REP-7) but MUST NOT be the authority on a subject's trust
  (REP-4). Ranking that "finishes" and centralizes is exactly what REP-4 forbids.
- **SEC-7 (abuse priced and localized; anti-Sybil not solved).** A profile MUST NOT describe Sybil
  resistance as solved. Ballot-stuffing costs real interactions where `Feedback.proof` is required
  (REP-6), the personhood/stake anchor raises the floor (§5), and web-of-trust dissolves it at local
  scale (§6) — but a funded, patient self-dealer defeats all three (§9). Every anchoring/gating claim
  MUST ship with this residual.
- **SEC-9 (metadata / irrevocability).** A `Feedback.body` is the one public field a natural person
  authors; a client MUST disclose, **before** publishing, that the object is public and irrevocable
  and that retraction is cooperative-only (§2.4, TRACT §10.2d, §22.6). It SHOULD warn on a body that
  appears to carry a third party's personal data.

---

## 9. Honest residual

Reputation is where three of the four root ceilings ([`DIRECTION § 8`](../DIRECTION.md)) converge, and
this primitive **discloses them rather than solving them**:

- **The adopted EigenTrust/OpenRank engine does not run transitively over these objects.**
  EigenTrust's Sybil-resistance depends on every party being identifiable as both truster and
  trustee, so trust can flow *through* it. REP-1 keys the trustee to a stable root `IK`; REP-2
  requires the truster to sign under a per-context pseudonym unlinkable to any root `IK` by a
  third party. A party's incoming trust (into its root `IK`) and outgoing vouches (from an
  unlinkable pseudonym) therefore attach to disjoint keys, and no indexer can compose them: the
  global-view compute degenerates to a near-non-transitive tally over directly-known edges, not
  the transitive engine §5 binds to. Closing this needs either an author-held mapping linking a
  truster's pseudonym to its subject `IK` (a privacy cost REP-2 exists to avoid) or scoping both
  truster and trustee to the same per-context key (a portability cost to REP-1) — KOTVA has
  chosen neither and discloses the gap instead.
- **Distrust does not reach the global-view compute.** Standard EigenTrust clips negative input
  to zero (`c_ij = max(s_ij,0) / Σ_k max(s_ik,0)`), and OpenRank inherits this. A trust-edge
  attestation with `polarity = -1` is therefore discarded, not aggregated, by the bound compute — it carries
  no weight against a bad actor's score there — and remains usable only as local, reader-side
  policy, never as input the adopted engine consumes.
- **The Sybil-cost floor is an open question, not a solved one.** The achievable floor on a
  signed-feed substrate is unresearched (TRACT §21.8), shared identically with WRAP. Personhood raises
  it; it does not close it, and every method trades off (biometrics + operator, or zk-passport that
  excludes the undocumented). Stake raises it in proportion to value at risk; it prices out the poor,
  not the funded.
- **Self-dealing produces *genuine* attestations.** A party transacting with itself generates real
  proofs and real feedback. `Feedback.proof` (REP-6) raises the cost and leaves a public trail; it
  establishes *that* an interaction happened, never that the counterparty was **independent**, and
  nothing on the wire can (the physical-event/independence ceiling, [`DIRECTION § 8`](../DIRECTION.md)).
  Escrow-issued proof is stronger but scarcest exactly where it would matter most, because escrow is
  opt-in and declined by the actors it constrains (TRACT §10.3a).
- **Whitewashing is bounded, not eliminated.** A fresh key has no history; discounting unanchored new
  keys (REP-5) is the *only* defence, and it is purely local — the protocol names no authority to ban,
  by design. A determined abuser re-anchors and starts over at the anchor's cost.
- **Portability implies cross-vertical linkability.** REP-1/REP-8 make a subject's reputation one
  shared surface keyed to the stable root `IK` across TRACT, WRAP, and every profile — one unsolved
  Sybil surface, never two. The unstated cost: a reputation-bearing key is a cross-context correlation
  handle, so any identity that accrues reputation in *any* vertical is thereby linkable across *all* of
  them. The only unlinkability escape — separate keys per vertical — forfeits the very portability
  REP-1 exists to provide. KOTVA chooses portability and **discloses the linkage** rather than hiding
  it; a subject that needs cross-vertical unlinkability MUST run distinct keys and accept no carried
  reputation.
- **The reader loses the convenience of one number.** Removing the global score buys sovereignty at a
  real usability cost: indexes disagree, and there is no single authoritative "4.8 stars." That
  divergence is the intended outcome (REP-4), and the cost is disclosed, not engineered away.

Trust/dispute research returned nothing verified across passes (TRACT §21.1); the reasoning here is
checked for internal consistency against the anchor docs, not offered as a demonstrated result.
Maturity claims for OpenRank, World ID, and Human Passport are a 2026-07 snapshot
([`bindings/README.md`](../bindings/README.md)) and MUST be re-checked before production reliance.
