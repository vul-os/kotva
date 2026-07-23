# 10. Trust

> **Drafting status.** This section is **normative** for the review and attestation model. Settled
> and carrying RFC 2119 keywords aligned to the frozen [`16-wire-format.md`](16-wire-format.md): the
> `Review` and `PurchaseAttestation` objects and their fields (§10.2, §10.2a) exactly as §16.5.5
> fixes them; per-subject pseudonymous authorship as a construction referenced from §1.5 (§10.2);
> reputation as portable substrate author-feed data with no operator (§10.2b); supersede-based
> retraction and its cooperative-only limit (§10.2d); the prohibition on any network-wide published
> score (§10.3); and the client disclosure requirement for review irrevocability (§10.2d, §22.6).
> **Still scoped, not yet normative and marked PROVISIONAL inline:** the *seller response* — no §16
> production carries one today, so it is a required §16 grammar change and a founder call, not
> invented here (§10.2c, §10.6). The honest-limit subsections (§10.3a, §10.4) state measured
> outcomes and are **not** weakened by the normative text around them. The key words MUST, MUST NOT,
> REQUIRED, SHOULD, SHOULD NOT and MAY are to be interpreted as in BCP 14 (RFC 2119, RFC 8174) where
> they appear.

## 10.1 Scope

Reviews, purchase attestation, and ranking — without creating an authority. TRACT specifies the
review object, the proof that binds it to a real transaction, and the prohibition that keeps
ranking derived rather than authoritative. It does **not** specify a scoring algorithm, a weighting
policy, or a discovery index; those are index-local judgements (§10.3, §21.9), and different
indexes reaching different answers is the design, not a defect.

Reviews are the **single bounded exception** to the rule that no personal data enters the public
quadrant (§0.5.1, §16.4, §22): a review is the one public object signed by a natural person. This
section bounds it; §22.6 works through the erasure residual that boundary leaves. Everything in the
review family is a public feed/blob object under the substrate
([`FEEDS.md`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md)), and TRACT introduces
no new signing, addressing, or feed construction for it — the substrate governs those bytes and
this section MUST NOT be read as re-specifying them.

## 10.2 The `Review` object

A `Review` is a public, signed, content-addressed object attached to a **subject** — a product
address, a seller, a distributor, or a courier. Its wire shape is frozen at §16.5.5 and an
implementation MUST encode exactly that shape:

- **Subject** (`Review[1]`). One of `{0 => content-address}` (a product record), `{1 => identity-key}`
  (a seller), `{2 => identity-key}` (a distributor), or `{3 => identity-key}` (a courier). A review
  MUST name exactly one subject of one of these four kinds; a decoder MUST reject any other subject
  discriminator (§16.4, fail-closed).
- **Author** (`Review[2]`). The author key MUST be a **per-subject pseudonymous subkey and MUST NOT
  be the root `IK`** — this is the identical construction §1.5 mandates for grant-tier recognition,
  and the two ride the same slot (§1.5, §16.5.5). Two subjects comparing their reviews therefore
  cannot join a buyer's reviews across them by author key; only the buyer, holding the root key or
  the stored mapping, can compute the correspondence. Which derivation method is used (deterministic
  vs. stored-random) is the open founder call of §1.5/§1.9 and is **not** re-decided here.
- **Score** (`Review[3]`). A `uint` in the range `0..5`. A decoder MUST reject a score outside that
  range. The protocol assigns no meaning to the number beyond ordering; how an index weights or
  aggregates scores is index-local policy (§10.3).
- **Body** (`Review[4]`). Free text, `tstr`, bounded by `MAX_REVIEW_BODY` (8 KiB, §19.5). The
  grammar **cannot** prevent a person typing a name, address, or contact detail into the body
  (§16.5.5), so the constraint is a client and index requirement, not a grammar one — see the
  personal-data requirements below.
- **Attestation** (`Review[5]`, optional). An optional `PurchaseAttestation` (§10.2a). Its presence
  is optional **on the wire**; whether an index accepts an unattested review is index-local policy
  (§10.2a, §10.3).
- **Timestamp** (`Review[6]`). `ts` in milliseconds; display and ordering only, never authoritative
  (§16.7). Authoritative order comes from the author feed's sequence
  ([`FEEDS.md`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md)).

**Personal-data requirements (the bounded exception, made operational).** Because `Review.body` is
the one public free-text field a natural person authors, a conformant client:

- MUST, before publishing a review, disclose to the author that the review is public, irrevocable,
  and cannot be fully taken back once published — at the moment of the action, not only in response
  to a later erasure request (§10.2d, §22.6, §22.7).
- SHOULD warn the author before publishing a body that appears to contain a name, street address,
  phone number, or e-mail address, since none of these can be recalled once the object exists
  (§22.7).
- SHOULD keep the body as small as the review needs; the smaller the field, the less identifying
  detail a public irrevocable object can carry (§19.5, §22.7).

An index MAY refuse to display or ingest a review whose body carries data that identifies a natural
person other than the author; this is index-local policy, consistent with §10.3, and no protocol
object compels a byte-holder to notice it.

## 10.2a Purchase attestation

A `PurchaseAttestation` is a signed proof that the review's author actually transacted. Its wire
shape is frozen at §16.5.5 and an implementation MUST encode exactly that shape:

- **Attestor** (`PurchaseAttestation[1]`). `0` = the seller, `1` = an escrow operator. The value
  states who is vouching, because the two carry different independence properties (see below).
- **Issuer** (`PurchaseAttestation[2]`). The `identity-key` that signed the attestation.
- **Order** (`PurchaseAttestation[3]`). The **content-address of the sealed order, and only its
  address**. An attestation MUST NOT carry any field of the order's contents — not the lines, not
  the total, not the buyer's detail. Nothing about *what* was bought crosses into the public
  quadrant; the address proves *that* a sealed order exists without disclosing it (§16.5.5, §22.4.3
  records that the address itself is not nothing).
- **Timestamp** (`PurchaseAttestation[4]`). `ts`, display and ordering only.

An attestation is verifiable by anyone who can check the issuer's signature — unlike a centralized
platform's "verified purchase" badge, which is an assertion only that platform can make. Because a
review's attestation binds it to a real transaction, ballot-stuffing costs actual trades rather
than free key generation.

**Index gating is policy, not protocol.** The grammar makes the attestation optional (§16.5.5).
Whether an index requires one is the index's own weighting decision: `ERR_TRACT_REVIEW_UNATTESTED`
(§17, `0x0901`, responder action `deny-policy`) exists precisely for an index that chooses to
reject unattested reviews at intake. The protocol neither mandates that gate nor forbids it —
mandating it network-wide would require the network-wide authority §10.3 removes.

**What attestation does not establish (kept honest, not papered over).** A seller-issued
attestation (attestor `0`) is issued by exactly the party a review may be about, so a seller
transacting with itself produces a **genuine** attestation; an escrow-issued attestation (attestor
`1`) is stronger but is scarcest where it would matter most, because escrow is opt-in and declined
by exactly the actors it constrains (§9.5a, §10.3a). This is a measured outcome (§21.6), stated
here rather than resolved.

## 10.2b Reputation is portable

A subject's reputation is the set of public reviews naming it, and it is **not held by any
operator**. It MUST NOT require one:

- A review names its subject by a **stable, operator-independent identifier** — a product
  `content-address` or the subject's `identity-key` (§16.5.5) — which is the same identifier at
  every index and every gateway. No index owns it, and a subject that changes which gateway renders
  its storefront carries its reviews with it, because the reviews attach to its key, not to a
  storefront.
- Reviews are ordinary public author-feed objects
  ([`FEEDS.md`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md), substrate capability
  ②/④). Any node MAY gather, cache, and index them without permission from anyone, and a subject
  cannot suppress an unfavourable one by leaving a platform — there is no platform to leave.
- Because sellers, couriers, and distributors are all just `IK`s on the same substrate (§1.2), a
  courier's reputation is **shared, not bridged**, across TRACT and WRAP
  ([`github.com/vul-os/wrap`](https://github.com/vul-os/wrap)): the same key that a TRACT `Review`
  with a courier subject names is the key that signs WRAP's work attestations (08-delivery §8.9,
  founder decision 4). TRACT defines the courier-review object; it does not redefine WRAP's work
  record, and MUST NOT.

Portability is a property of the substrate feed and the stable subject identifier, not of any
TRACT-specific mechanism; this section adds none.

## 10.2c Seller response — PROVISIONAL

A subject that a review names has a legitimate interest in responding in public, attached to the
same subject, so that a reader sees both. This is settled as an intent; the **wire shape is not**.

> **PROVISIONAL — pending decision.** No §16 production carries a seller response today. `Review`
> requires a `Subject` that is a product, seller, distributor, or courier (§16.5.5) — it cannot name
> another review, so a response is not expressible as a `Review`, and there is no `ReviewResponse`
> object in the frozen grammar. A response therefore requires a **§16 grammar change** (a public
> object referencing a review's content-address, authored by the review's subject key) **and** a
> founder call on its shape. Until that lands, an implementation MUST NOT synthesise a response
> object the frozen grammar does not define (§16.4); a subject's recourse in the interim is to
> publish an ordinary public statement on its own feed, which no index is obliged to correlate with
> the review. Logged as a required §16 open item and a founder decision (§10.6).

## 10.2d Retraction

A review's author MAY retract it. Retraction is **supersession, never deletion**, and follows the
same pattern as a withdrawn offer (§18.2, "withdrawal is not deletion"):

- The author publishes a later signed entry in the same author feed that supersedes the review; a
  conformant client or index that checks the author's feed head before displaying the review MUST
  show the retraction rather than the original
  ([`FEEDS.md`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md), §18.2). TRACT adds no
  new tombstone byte shape for this; it is the substrate's own supersede mechanism over a public
  feed.
- Retraction is **cooperative-only**, and this limit is real: it binds only participants who look
  for the later entry and choose to honour it. A byte-holder that serves whatever is at the review's
  address, an index that snapshotted before the supersede, or a cache pinned independently of the
  feed is not violating the protocol — nothing in the wire format obliges a byte-holder to notice a
  later message, and nothing can make it (§22.4.2).
- Because of that residual, a client MUST disclose to the author, **before** the original review is
  published, that its bytes may persist wherever an independent holder kept them and that retraction
  is honoured by cooperation rather than enforced (§10.2, §22.6). Disclosure of irrevocability is
  before the act, not after a request to remove it.

## 10.3 The prohibition — no network-wide published score

**There MUST be no network-wide published score, and no TRACT object carries one.** Computing a
single authoritative number requires a party that aggregates and ranks every subject's reviews —
which is exactly the operator this design removes. The grammar reflects this: a `Review` carries a
per-review `score`, and there is no aggregate-score object anywhere in §16.

Ranking is **derived data**. Any node MAY build an index over the public review feeds and compute
whatever aggregate, weighting, or ranking it chooses; an index is rebuildable and MUST NOT be
treated as authoritative (§0.9 "index"; §2.6). Indexes will differ — one may discount unattested
reviews, another may weight escrow-attested reviews above seller-attested ones, another may apply a
web-of-trust measure local to the querying party — and **that divergence is the intended outcome,
not a defect** (§10.4, §21.9). A reader loses the convenience of one authoritative number; the
design buys, in exchange, that no single party decides what a subject's reputation is.

This is also where the honest limits below live: an index's weighting policy is the only place the
Sybil/self-dealing judgement of §10.3a can be made, because the protocol deliberately provides no
central place to make it.

## 10.3a What attestation does not fix (§21.6)

§21.9 obliges this section to state the measured outcome rather than the hypothetical one.

OpenBazaar's reputation failure was self-published, unweighted reviews with no banning authority,
and **one vendor faked 60% of measured sales value**. A purchase attestation is strictly stronger
than what OpenBazaar had — it binds a review to a real transaction, so ballot-stuffing costs actual
trades. Two of the failure conditions nonetheless survive unchanged:

1. **Self-dealing produces genuine attestations.** A seller transacting with itself generates real
   proofs. Attestation raises the cost and leaves a public trail; it does not establish that the
   counterparty was independent, and nothing in this document does.
2. **Escrow-issued attestations are scarcest where they would matter most**, because escrow is
   opt-in and declined by exactly the actors it constrains (§9.5a).

The achievable Sybil-cost floor on a signed-feed substrate is an **open question** (§21.8), not a
solved one. An index's weighting policy is where that judgement lives, and different indexes
reaching different answers is the design rather than a defect. TRACT holds the same honest posture
here as WRAP does for provider reputation on the same substrate
([`github.com/vul-os/wrap`](https://github.com/vul-os/wrap)): shared identity and shared feeds mean
a shared, unsolved Sybil problem, not two separately-solved ones.

Per §21.1, this subsection cites §21 only for the honesty it obliges; it is **not** offered as
support for any trust claim — trust/dispute research returned nothing verified across passes.

## 10.4 Honest limits this section must state

- Rankings disagree between indexes; buyers lose the convenience of one authoritative number.
- Attestation raises manipulation cost but does not eliminate it — a seller can transact with
  itself, expensively and visibly (§10.3a).
- Whitewashing is bounded, not solved: a new key has no history, and discounting new keys is the
  only available defence — a purely index-local one, since the protocol names no authority to ban.
- **Erasure**: a review is public, irrevocable, and signed by a person. Retraction is a superseding
  tombstone honoured by conformant clients and gateways (§10.2d); the residual — bytes persist if
  any independent holder keeps them — MUST be disclosed to the author before publishing (§10.2,
  §22.6). Reviews are the one bounded personal-data exception in the public quadrant, and §22.6
  works the residual through in full.

## 10.5 Prior art

Sybil, whitewashing and ballot-stuffing literature on decentralized reputation; web-of-trust and
locally-measured reputation as the alternatives to a global score. The load-bearing postmortem is
OpenBazaar's (§21.3, §21.6); the cautionary scope note is Nostr's own NIP-15 "unrecommended: too
complicated" (§21.7). None of this literature was reached with verified findings for trust
specifically (§21.1) — it is design reasoning checked for internal consistency, not a demonstrated
result.

## 10.6 Open

Collected for the founder-decision list:

- **The seller-response object (§10.2c).** No §16 production carries one, so a response is not
  expressible today. This needs a §16 grammar slot that does not exist yet (a public object
  referencing a review's `content-address`, authored by the review's subject key) **and** a founder
  call on its shape. Recommendation: add a minimal `ReviewResponse` public object — subject-key
  author, target review `content-address`, bounded body, `ts` — reusing `MAX_REVIEW_BODY` and the
  same public-quadrant personal-data prohibition, so a response adds no new privacy surface beyond a
  review's own; keep it optional and index-displayed alongside the review, never merged into the
  score. Recorded as a required §16 change, not invented here.
- **Whether an index requiring a valid `PurchaseAttestation` should be a protocol floor or remain
  index-local policy.** Today it is index-local (`ERR_TRACT_REVIEW_UNATTESTED` is a policy deny, not
  a decode failure; §17). Recommendation: keep it index-local — a network-wide mandate would require
  the aggregating authority §10.3 removes, and the honest cost of optionality (§10.3a) is already
  disclosed rather than engineered away.
- **The achievable Sybil-cost floor on a signed-feed substrate (§21.8).** Unresearched, shared with
  WRAP. Not a founder call so much as a flagged research gap that no normative text here may pretend
  is closed; recorded so it is not lost.
