# 14. Anti-abuse

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 14.1 Scope

Listing spam, fake stores, review manipulation, Sybil identities, and resource exhaustion.

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

## 14.3 Holder admission policy

A holder (§0.4.1) decides what it stores and serves, and that decision is where most abuse is
actually stopped, before an object ever reaches an index:

- **Object-size ceilings.** `MAX_PRODUCT_RECORD` (64 KiB), `MAX_OFFER` (16 KiB), `MAX_RATE_CARD`
  (1 MiB), `MAX_REVIEW_BODY` (8 KiB) — the full set is §19.5. A holder that enforces these bounds
  cannot be made to store an unbounded object by a publisher who simply declines to stop writing.
- **Per-publisher storage quota.** A ceiling on total bytes a holder will keep for one identity key,
  independent of how many individual objects that key publishes. Size limits bound one object;
  a quota bounds one publisher writing many small, individually-legal ones.
- **Feed-append rate.** A ceiling on how fast one feed may grow, so a compromised or malicious key
  cannot exhaust a holder's storage or bandwidth faster than the holder can react. This is a
  holder-side policy parameter, not yet collected in §19, and doing so is listed at §14.8.

None of these is a protocol requirement on what a seller may publish — a seller's own feed is
theirs to fill (§0.4.1). They are requirements on what a *holder* is willing to carry, and a holder
is free to set them tighter than any floor this document proposes.

## 14.4 Index admission policy, and the refusal rule

An index (§0.9 glossary; §2.6) decides independently what it aggregates. **Refusal to serve or
index is a policy decision, never a protocol-level takedown.** No holder can compel another holder
to serve an object, and no index can compel a holder to stop serving one, or compel another index to
agree with its own admission choices. A disagreement between an index and a feed always resolves in
favour of the feed (§0.4.1): an index that refuses to list a seller has removed itself from that
seller's discovery, not removed the seller.

The corollary is that indexing power concentrates the same way holding power does not: refusing to
index is cheap, individually reversible, and produces no shared record other than the index's own
choice. §0.4.1 already discloses the consequence — a dominant index becomes a de facto
content-policy gatekeeper regardless of what this document permits — and that disclosure is §2.6's
to own; this section's obligation is narrower: the *mechanism* of refusal is policy, at every layer,
by construction, never a network-level removal.

## 14.5 Cold-contact economics

A sealed order from a stranger (§7, §16.6) is, structurally, an unsolicited first-contact message —
exactly the case the substrate's own anti-abuse tiers were built to price, not a new problem TRACT
introduces. Two substrate properties already bound the cost of spamming orders at a seller:

- the **mailbox** a sleeping seller wakes through is short-TTL and content-blind (§0.3, capability
  ④) — nothing sent to it persists past that TTL, and nothing about its content is visible to
  whoever operates it, so it cannot be used as free bulk storage for junk orders the way an open
  inbox can;
- **wake** delivery is content-free and sender-blind (§0.3, capability ⑤) — a wake signal carries no
  payload and does not identify who sent it to the transport, so it gives an attacker no cheap way
  to fan a single spam campaign out across many sellers' wake paths at once.

A seller still decides, per §14.3's admission logic applied to its own inbound path, how much
unsolicited contact from unrecognised keys it accepts before requiring some form of prior
relationship (a follow, a previous order) — that policy is the seller's, not the protocol's, for the
same reason holder admission is (§14.3).

## 14.6 Review manipulation: what purchase attestation does and does not prevent

§10.2 specifies purchase attestation: a proof, issued by the seller or an escrow operator, that a
review's author actually transacted — verifiable by anyone, and it makes ballot-stuffing cost a real
transaction rather than a keystroke. As an anti-abuse mechanism specifically, it is worth being
exact about its edges, because the closest deployed relative of this design measured them.

OpenBazaar's reputation system was self-published, unweighted reviews with no banning authority, and
**one vendor faked 60% of measured sales value** (§21.3, §21.6). Purchase attestation is strictly
stronger than that — it binds a review to a signed transaction record — but two of OpenBazaar's
failure conditions are not addressed by attestation and survive unchanged in this design as
currently written:

1. **Self-dealing produces genuine attestations.** A seller transacting with a key it also controls
   generates a real, verifiable proof of purchase. Attestation raises the cost of faking a review and
   leaves a signed trail; it does not, and cannot, establish that the counterparty was an independent
   party.
2. **Escrow-issued attestations are scarcest exactly where they would matter most.** Escrow is
   opt-in (§9.5), and the measured OpenBazaar outcome was that bad actors simply declined the
   protection that would have constrained them — declining it was free, and the actors most worth
   constraining were the ones with the clearest incentive to decline. §9.5a and §10.3a already carry
   this as a measured cost of optionality rather than a hypothetical; this section inherits it as an
   anti-abuse limit rather than re-deriving it.

Both are stated in full in §10.3a and §10.4, which this section defers to rather than duplicates;
what belongs here is the anti-abuse framing — attestation is admission control on *who may credibly
claim to have transacted*, not a guarantee that the transaction was arm's-length.

## 14.7 Sybil and whitewashing

A new identity key has no history. Discounting new keys — weighting an unfamiliar seller, courier,
distributor or reviewer down until it has accumulated some track record — is the only defence this
design has against a bad actor simply generating a fresh key after being caught, and it is a blunt
one: it penalises every legitimate newcomer exactly as much as it penalises a whitewasher, because
the two are indistinguishable at the point a key first appears. There is no resolution to that which
does not require a party empowered to vouch for or gate new identities — which is an authority, and
reintroducing one to solve Sybil resistance would concede the point the rest of this document is
built to avoid.

**The achievable Sybil-cost floor on a signed-feed substrate is an open question**, not a solved one
this section is understating: §21.8 lists it as entirely unresearched by either literature pass
behind this document. Whether buyer counter-signatures, transaction-cost binding, or purely local
web-of-trust scoring actually raise that floor is unknown rather than assumed, and an index's
weighting policy is where whatever judgement is currently applied actually lives — different indexes
reaching different answers here is the design (§10.3), not a gap in it.

## 14.8 Honest limits

- Attestation makes a fake review or a fake sale expensive and leaves a signed trail. It does not
  make either impossible, and §14.6 states exactly where it stops.
- Holder and index admission policy (§14.3, §14.4) is enforced locally and voluntarily; nothing
  compels any two holders or indexes to agree on where the line sits, and a seller excluded
  everywhere still has a valid, undeleted feed.
- Feed-append rate and per-publisher storage quota are described here as concepts a holder needs;
  neither has a proposed default in §19 yet.

## 14.9 Open

- Whether feed-append rate and per-publisher storage quota belong in §19 as suggested defaults, or
  are inherently holder-local policy that a shared table would misrepresent as protocol-level.
- The achievable Sybil-cost floor on a signed-feed substrate (§21.8) — unresearched, and blocking a
  more specific answer than "discount new keys" for §14.7.
- Whether cold-contact policy (§14.5) needs a machine-readable declaration on a seller's feed — so a
  buyer's node knows before sending whether an unsolicited order will even be looked at — or is
  better left as an undeclared, seller-side heuristic.
