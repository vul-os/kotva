# 14. Anti-abuse

> **Drafting status — partially normative.** The holder- and index-side posture is now normative:
> §14.3 (validate-not-trust and holder admission), §14.4 (index refusal and capability-absence),
> §14.5 (cold-contact policy) and the review-intake rule in §14.6 carry RFC 2119 keywords, to be
> interpreted as in BCP 14 (RFC 2119, RFC 8174). §14.7 (the achievable Sybil-cost floor) remains
> **scoped, not normative**: its blocking research question is unresearched (§21.8), and the
> paragraph is marked **PROVISIONAL** rather than resolved. §14.8–§14.9 record honest limits and
> open decisions. This section defines **no new wire object**: every rule below is enforcement over
> the §16 objects, using the §19.5 limits and the §17.3 responder actions, and introduces no
> cryptography, transport, or CRDT of its own (all inherited from the substrate, §0.3).

## 14.1 Scope

Listing spam, fake stores, review manipulation, Sybil identities, and resource exhaustion — and, as
much as the honesty of the rest, the abuses this design does **not** overcome. The anti-abuse
posture TRACT can require is a posture on **holders and indexes** (§0.4.1), not on publishers: a
seller's own feed is theirs to fill, and every control here is a requirement on what another party
is willing to carry, serve, or act upon.

## 14.2 The structural position

A seller can flood only **their own** feed, which only their own followers and holders pay for and
may stop serving at will. There is no shared feed to spam and no fan-out amplification: a bad actor
publishing garbage grows the cost of following that one seller, not the cost of running the
network.

| | Shared feed (a marketplace) | A seller's own signed feed (TRACT, §2) |
|---|---|---|
| Who pays to store and serve one seller's spam | every holder of the shared feed, whether or not they follow that seller | only holders who chose to hold that seller's feed |
| Blast radius of one bad actor | the whole marketplace's index and storage | that seller's own followers |
| Can a holder opt out of just the bad actor | no — the feed is shared | yes, at any time (§14.3) |

This is inherited from the substrate's public-object model (§0.3, capability ②) and is why
catalogue spam is a genuinely weaker threat here than on a platform with one shared listings feed.
It is worth stating plainly rather than leaving it implied: this is a real structural advantage, not
a rhetorical one, and it holds regardless of how the rest of this section's open questions resolve.

## 14.3 Holder admission policy — validate, do not trust

An unsolicited public object (§16.5) arrives from a stranger and carries only its publisher's own
signature. That signature proves **who wrote it**, and nothing more: it never establishes that what
the object claims is safe to store, serve, index, or price against. A holder (§0.4.1) therefore
decides what it carries, and that decision is where most abuse is actually stopped, before an object
ever reaches an index.

**The validate-not-trust rule.** A holder MUST validate an unsolicited public object against every
structural bound applicable to its type **before** it stores, serves, or acts upon it, and MUST
refuse it — `fail-closed-block` (§17.3) — when it exceeds any of those bounds. A holder MUST NOT
clamp, truncate, or otherwise coerce an out-of-bound value into a plausible in-bound one. `§8.3`'s
`dim_divisor` floor is the template case: a divisor below `MIN_DIM_DIVISOR` (§19.5) is rejected
outright, **not** silently raised to the floor, because a coerced value looks exactly as valid as a
real one and nothing downstream can tell the difference — the same reasoning that refuses a coerced
currency or a wrapped total (§16.7) and that refuses a guessed axis variant rather than rendering it
(§17.6). §14.3 is the home of the "validated, not trusted" property the rest of this document refers
to (§8.3).

The bounds a holder enforces:

- **Object-size ceilings.** A holder MUST enforce the §19.5 ceilings — `MAX_PRODUCT_RECORD`
  (64 KiB), `MAX_OFFER` (16 KiB), `MAX_RATE_CARD` (1 MiB), `MAX_REVIEW_BODY` (8 KiB). An object
  exceeding its type's ceiling MUST be refused (`fail-closed-block`). A holder that enforces these
  bounds cannot be made to store an unbounded object by a publisher who simply declines to stop
  writing. Size limits bound **one object**.

- **Per-publisher storage quota.** A holder MAY cap the total bytes it will keep for one identity
  key, independent of how many individual objects that key publishes — bounding **one publisher**
  writing many small, individually-legal objects. Declining a well-formed object for want of quota
  is `deny-policy` (§17.3), a fact about this holder's own terms, not a defect in the object; it MUST
  NOT be reported as a structural error against the publisher.

- **Feed-append rate.** A holder MAY cap how fast one feed may grow, so a compromised or malicious
  key cannot exhaust the holder's storage or bandwidth faster than the holder can react. This is a
  holder-local policy parameter with **no proposed default in §19** (§14.9); whether it belongs in
  §19 at all is **PROVISIONAL — pending decision** (§14.9, first bullet).

None of these is a protocol requirement on **what a seller may publish** — a seller's own feed is
theirs to fill (§0.4.1). They are requirements on what a *holder* is willing to carry, and a holder
MAY set any of them tighter than any floor this document proposes. A holder MUST NOT be read as a
protocol-level authority over a seller's catalogue: enforcing an admission bound removes an object
from **that holder's** store, never from the seller's feed.

## 14.4 Index admission policy, the refusal rule, and capability-absence

An index (§0.9 glossary; §2.6) decides independently what it aggregates. **Refusal to serve or
index MUST be treated as a policy decision, never a protocol-level takedown.** No holder can compel
another holder to serve an object; no index can compel a holder to stop serving one, or compel
another index to agree with its own admission choices. A disagreement between an index and a feed
MUST resolve in favour of the feed (§0.4.1): an index that refuses to list a seller has removed
itself from that seller's discovery, not removed the seller.

**Capability-absence is not a fault.** TRACT inherits the substrate's adoption rule 3: *a peer that
has not advertised a capability is never expected to serve it, and its silence is never a fault*
(`https://github.com/vul-os/dmtap/blob/main/substrate/README.md`). A node that has not undertaken to
hold, index, pin, or serve a given object is therefore not malfunctioning by declining it. A
holder's or index's refusal MUST NOT be encoded or reported as an error **against that node**; where
a responder must name the outcome, it is `deny-policy` (§17.3) — a fact about the responder's
declared scope, not a defect in what was presented, and never `fail-closed-block`, which §17.3
reserves for structural violations of the object itself.

The corollary is that indexing power concentrates the same way holding power does not: refusing to
index is cheap, individually reversible, and produces no shared record other than the index's own
choice. §0.4.1 already discloses the consequence — a dominant index becomes a de facto
content-policy gatekeeper regardless of what this document permits, the **weakest load-bearing claim
in the document** (§21.3, §21.4) — and that disclosure is §2.6's to own. This section's obligation
is narrower: the *mechanism* of refusal is policy, at every layer, by construction, never a
network-level removal.

## 14.5 Cold-contact economics

A sealed order from a stranger (§7, §16.6) is, structurally, an unsolicited first-contact message —
exactly the case the substrate's own anti-abuse tiers were built to price, not a new problem TRACT
introduces. Two substrate properties, referenced here and re-specified nowhere in TRACT, already
bound the cost of spamming orders at a seller
(`https://github.com/vul-os/dmtap/blob/main/substrate/ROLES.md`):

- the **mailbox** a sleeping seller wakes through is short-TTL and content-blind (§0.3, capability
  ④) — nothing sent to it persists past that TTL, and nothing about its content is visible to
  whoever operates it, so it cannot be used as free bulk storage for junk orders the way an open
  inbox can;
- **wake** delivery is content-free and sender-blind (§0.3, capability ⑤) — a wake signal carries no
  payload and does not identify who sent it to the transport, so it gives an attacker no cheap way
  to fan a single spam campaign out across many sellers' wake paths at once.

A seller MAY require some form of prior relationship — a follow, a previous order — before it accepts
unsolicited contact from an unrecognised key, and MAY refuse such contact outright; how much it
accepts is the seller's policy, enforced the same way holder admission is (§14.3), and **TRACT
defines no object for it**.

> **PROVISIONAL — pending decision.** Whether a seller's cold-contact policy should be a
> machine-readable declaration on the seller's feed — so a buyer's node knows *before* sending
> whether an unsolicited order will even be looked at — or is better left as an undeclared,
> seller-side heuristic, is **open** (§14.9). No §16 grammar slot exists for such a declaration
> today; adding one would be a MAJOR version change (§16), and this document does **not** invent it.
> The recommendation is recorded in the founder-decision list.

## 14.6 Review manipulation: what purchase attestation does and does not prevent

§10.2 specifies purchase attestation: a proof, issued by the seller or an escrow operator, that a
review's author actually transacted — verifiable by anyone, making ballot-stuffing cost a real
transaction rather than a keystroke. As **admission control**, an index MAY require a valid
`PurchaseAttestation` before it serves a `Review` and MUST, where it does, treat an unattested
review as `ERR_TRACT_REVIEW_UNATTESTED` → `deny-policy` (§17.4) — a refusal under the index's own
declared standard, not a defect in the review. An index that requires attestation MUST NOT, however,
treat its presence as proof the counterparty was independent; attestation is admission control on
*who may credibly claim to have transacted*, not a guarantee that the transaction was arm's-length.

This edge is not hypothetical: the closest deployed relative of this design measured it.
OpenBazaar's reputation system was self-published, unweighted reviews with no banning authority,
and **vendor-scale ballot-stuffing of measured sales value** (§21.3, §21.6). Purchase attestation
is strictly stronger — it binds a review to a signed transaction record — but two of OpenBazaar's
failure conditions are **measured outcomes** (§21.6), not hypotheticals, and survive unchanged in
this design as currently written:

1. **Self-dealing produces genuine attestations.** A seller transacting with a key it also controls
   generates a real, verifiable proof of purchase. Attestation raises the cost of faking a review and
   leaves a signed trail; it does not, and cannot, establish that the counterparty was an independent
   party.
2. **Escrow-issued attestations are scarcest exactly where they would matter most.** Escrow is
   opt-in (§9.5), and the measured OpenBazaar outcome was that bad actors simply declined the
   protection that would have constrained them — declining it was free, and the actors most worth
   constraining had the clearest incentive to decline. §9.5a and §10.3a carry this as a measured cost
   of optionality; this section inherits it as an anti-abuse limit rather than re-deriving it.

Both are stated in full in §10.3a and §10.4, which this section defers to rather than duplicates.

## 14.7 Sybil and whitewashing

A new identity key has no history. Discounting new keys — weighting an unfamiliar seller, courier,
distributor or reviewer down until it has accumulated some track record — is the only defence this
design has against a bad actor simply generating a fresh key after being caught, and it is a blunt
one: it penalises every legitimate newcomer exactly as much as it penalises a whitewasher, because
the two are indistinguishable at the point a key first appears.

The one thing that **is** settled here is what TRACT MUST NOT do about it: TRACT introduces no party
empowered to vouch for or gate new identity keys. Any such party is an authority, and reintroducing
one to solve Sybil resistance would concede the point the rest of this document is built to avoid
(§0.4). The only defence available is local: an index, or a counterparty, MAY weight an unfamiliar
key down until it accumulates a track record, and an index's weighting policy is where whatever
judgement is currently applied actually lives — different indexes reaching different answers is the
design (§10.3), not a gap in it.

> **PROVISIONAL — pending decision.** **The achievable Sybil-cost floor on a signed-feed substrate
> is an open question, not a solved one** (§21.8): it is listed as entirely unresearched by either
> literature pass behind this document. Whether buyer counter-signatures, transaction-cost binding,
> or purely local web-of-trust scoring actually raise that floor is **unknown rather than assumed**.
> No normative floor stronger than "discount new keys" can be stated until that research lands, and
> this paragraph MUST NOT be read as understating a solved problem. Recorded in the founder-decision
> list.

## 14.8 Honest limits

- Attestation makes a fake review or a fake sale expensive and leaves a signed trail. It does not
  make either impossible, and §14.6 states exactly where it stops. The Sybil-cost floor beneath it
  is unresearched (§14.7, §21.8).
- Holder and index admission policy (§14.3, §14.4) is enforced locally and voluntarily; nothing
  compels any two holders or indexes to agree on where the line sits, and a seller excluded
  everywhere still has a valid, undeleted feed. A refusal is `deny-policy`, never a takedown.
- Feed-append rate and per-publisher storage quota are described here as concepts a holder MAY
  apply; neither has a proposed default in §19 yet (§14.9).
- Sybil resistance is an **honest limit, not a trustless guarantee** (§14.7). The cold-contact and
  admission bounds price abuse; they do not make it impossible, and no claim here is that they do.

## 14.9 Open

- Whether feed-append rate and per-publisher storage quota belong in §19 as suggested defaults, or
  are inherently holder-local policy that a shared table would misrepresent as protocol-level.
- The achievable Sybil-cost floor on a signed-feed substrate (§21.8) — unresearched, and blocking a
  more specific answer than "discount new keys" for §14.7.
- Whether cold-contact policy (§14.5) needs a machine-readable declaration on a seller's feed — so a
  buyer's node knows before sending whether an unsolicited order will even be looked at — or is
  better left as an undeclared, seller-side heuristic. A declaration would require a new §16 grammar
  slot.
