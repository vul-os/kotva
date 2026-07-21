# 21. Grounding — what the evidence actually supports

> **Why this section exists.** Every claim in this specification about what decentralized commerce
> can do is a claim about the future. The claims below are about the past, and they are less
> flattering. This section records what a 107-agent, adversarially-verified literature pass found
> in July 2026, including the findings that **contradict** parts of this document, so that a reader
> can tell the difference between a design decision and a demonstrated result.
>
> Nothing here is normative. Everything here should change what is normative.

## 21.1 Coverage — read this before citing anything below

Seven areas were researched. **Only three produced claims that survived three-vote adversarial
verification:** deployed networks, global product identity, and the goods/variant slice of the data
model.

**Unresearched, and therefore neither supported nor refuted:** logistics standards and consolidation
optimisation (§8); privacy-preserving analytics (§13); live commerce state, bounded-counter CRDTs
and reservation semantics (§6); and the whole of trust, dispute, tax and legal (§9.6, §10, §11).

Those gaps are **absence of evidence, not evidence of absence**, and §8, §10, §11 and §13 must not
cite this section as support. Within the covered areas, several named systems also produced nothing
verified: Particl, Origin Protocol, BOLT12, x402, L402, and ActivityPub-based commerce.

## 21.2 The finding that most threatens this design

> **There is no deployed system achieving cross-publisher product identity without a licensed
> registry, and the one candidate for a permissionless alternative was refuted.**

The two deployed models are the only two:

| Model | Property | Cost / gate |
|---|---|---|
| **GS1 GTIN** | permissioned monopoly namespace | *licensed*, not sold; issued through national member organisations; US$30 per identifier at GS1 US (verified 2026-07-21, US-specific — do not generalise). The free tier is explicitly restricted to closed/local circulation. |
| **schema.org `productGroupID`** | purely nominal | a plain `Text` string — no issuer, no registry, no uniqueness guarantee |

There is nothing in between. A claim that crawl-derived clustering over schema.org identifiers
demonstrates feasible permissionless product entity resolution was **refuted 0-3**. So this
document contains **no positive evidence that cross-publisher product deduplication works at
scale.**

**What this means for §2.** The content-address floor is sound as a *mechanism* — identical bytes
converge, trivially. It is unproven as a *solution*, because independent publishers describing the
same physical product do not produce identical bytes, and nothing here shows that the gap can be
closed adversarially. §2's canonicalisation rules are therefore the load-bearing part of that
section, not the addressing, and §2.5 must keep near-duplicate resolution listed as **open**
rather than implied-solved. Language suggesting a global product view "falls out of" content
addressing overstates what is known.

## 21.3 The OpenBazaar postmortem

OpenBazaar was the closest deployed relative of this design: signed objects, content-addressed
listings, keypair identity, no operator. It shut down in January 2021. Cite it as a postmortem,
never as a live comparison.

| Measure | Result |
|---|---|
| Lifetime participants | ~6,651 |
| Concurrently online | ~80 |
| Credible sales, 14 months | ~US$86,000 |
| Median listing lifetime | ~22 days |
| Share of measured sales value faked by one vendor | **60%** |

Four failure modes, each of which maps onto a section of this document:

1. **Negligible economic activity** — orders of magnitude below centralized equivalents.
2. **Discovery re-centralized first.** A content-addressed substrate offers no global index, so
   search was outsourced to crawler operators, and the default one became a content-policy
   gatekeeper. → §2.6.
3. **Availability was bounded by publisher liveness.** Whole catalogues disappeared when a merchant
   node went offline; third-party indexes went stale against the live network. → §21.5.
4. **Reputation was trivially ballot-stuffed and structurally unenforceable**, and **opt-in escrow
   and moderation went unused precisely where they mattered most** — bad actors simply declined
   them. → §21.6.

*Source caveat:* all four rest on one peer-reviewed, DHS-funded measurement paper and share its
methodology. Sales figures are an author-acknowledged lower bound from voluntary feedback, and
item-survival numbers conflate deliberate delisting with liveness eviction.

## 21.4 What the largest live network chose instead

**Beckn / ONDC** is the biggest deployed "decentralized commerce" network, and it avoids
OpenBazaar's failure modes by doing the opposite of this specification:

- **Registry-mediated, not self-certifying.** Participants register keys via `POST /subscribe`;
  counterparties resolve endpoints and public keys through a registry lookup — not gossip, not a
  DHT, not content addressing.
- **Approval-gated permissioned enrollment.** Portal whitelisting (6–48h), a signed agreement,
  certification, and probation before a participant may transact.
- **Identity anchored to DNS/TLS domain names, not keypairs** — inheriting CA and DNS
  centralization on top of registry centralization.
- **Discovery through operator-hosted, rate-limited lookup endpoints**, giving the operator a
  throttle and deny lever over network-wide discovery.

**This is the honest comparison, and it is uncomfortable.** The network with volume chose an
operator at exactly the three points this document tries to leave operator-free. That is not proof
this design fails; it is evidence that the burden of proof sits here, not with them.

*Time sensitivity:* Beckn developer docs are stamped 2021; ONDC has deprecated `/lookup` and
`/vlookup` in favour of `/v2.0/lookup`, whose rate limit is published as "TBD".

## 21.5 Liveness is a first-class problem, not an edge case

§0.7 says durability lives at the edges and the sender's node retries. That is true **for orders**
and false **for catalogues**.

A seller whose node is offline is not merely slow to receive orders — they are **invisible**.
OpenBazaar's ~22-day median listing lifetime and mass catalogue disappearance on node departure are
the measured consequence. §21.1 notwithstanding, this one is directly evidenced.

**Implications this document must carry rather than bury:**

- Documentation MUST NOT imply that an intermittently-online seller is fully functional. A laptop
  that sleeps can *receive orders* through the substrate's mailbox and wake path; it cannot *serve
  a catalogue* while asleep.
- Third-party caching/pinning of public catalogue objects moves from a nice-to-have to a
  requirement for any seller not running an always-on node — and **unpaid replication is exactly
  what OpenBazaar did not get**. Whether pinning needs an incentive, and whether that incentive
  creates another operator, is open.
- A gateway serving a store is, incidentally, a liveness backstop. That is an argument *for* the
  operator class of §0.4.2, and it should be counted honestly as one.

## 21.6 Reputation: the documented failure mode is this design's

OpenBazaar's reputation failure was **self-published, unweighted reviews with no banning
authority**, combined with **opt-in escrow that bad actors declined**. One vendor faked 60% of
measured sales value.

§10 proposes purchase-attested reviews, which is strictly stronger than what OpenBazaar had — an
attestation binds a review to a transaction. But two of OpenBazaar's failure conditions survive
unchanged in this document as currently written:

1. **Self-dealing.** A seller transacting with itself produces genuine attestations. Attestation
   raises cost; it does not establish that the counterparty was independent.
2. **Opt-in escrow is declined by exactly the actors it exists to constrain.** §9.6 makes escrow
   per-order and optional, and §9.6's own reasoning — that mandatory escrow would exclude
   underserved regions — is sound. But the measured consequence of optionality is that it goes
   unused where it matters. Both cannot be true costlessly.

**§10 and §9.6 must state this rather than claiming attestation resolves it.** The sybil-cost floor
achievable on a signed-feed substrate is listed in §21.8 as an open question because the attack
literature was not reached by this pass.

## 21.7 A warning about scope

Nostr — a signed-feed ecosystem structurally similar to this substrate — marked its marketplace
specification **NIP-15 "unrecommended: too complicated"** in-repo (banner added 2026-05-31) and
redirected implementers to NIP-99, a far simpler classified-listing primitive. Neither solves "many
stores, one SKU": product IDs are merchant-generated and merchant-scoped, discovery is free-text
tags, and there is no cross-publisher product identifier.

NIP-15 also moves the entire order and checkout flow off the public feed into encrypted messages
with **the merchant as sole authority** asserting payment and shipment — no escrow, no buyer
counter-signature, no third-party-verifiable order state. This document's §7 signed-transition
model is a deliberate departure from that, and the departure is the point; but the ecosystem's own
verdict on structured-commerce complexity is a caution this specification should not ignore.

*Wording note:* the repo status word is "unrecommended", not "deprecated".

## 21.8 Open questions this pass could not close

1. **Does any deployed system achieve cross-publisher product identity without a licensed
   registry?** The design space between GS1 monopoly and nominal merchant string is currently
   evidence-free. Needs a dedicated pass on entity resolution, blocking strategies and
   probabilistic matching **under adversarial publishers** — including deliberate collision.
2. **What is the achievable sybil-cost floor for reputation on a signed-feed substrate?** Do buyer
   counter-signatures, transaction-cost binding, or local-only web-of-trust scoring actually resist
   ballot-stuffing? Entirely unresearched.
3. **Where does a spec put the operator-shaped functions it cannot eliminate — indexer, registrar,
   arbiter, pinner?** ONDC answers "one central approval-gating registry". OpenBazaar answered
   "nowhere" and got a gatekeeping default search engine plus liveness-bounded availability. **Is
   there a middle design** — multiple competing indexers with verifiable completeness or
   censorship proofs, federated or rotating registrars, paid pinning markets — **with any deployed
   precedent and measured outcomes?** This is the central unanswered question for §0.4.
4. **How do erasure rights and identifiable-seller mandates interact with immutable,
   content-addressed, replicated commerce objects?** Who is the data controller when order data is
   replicated across independent nodes; how is erasure satisfied against irrevocable objects; who
   satisfies trader-traceability duties when identity is a keypair; and who is the "marketplace
   facilitator" for tax purposes when no marketplace operator exists. **These are the most likely
   hard blockers on a no-operator design**, and §0.5.1 and §11 currently answer them by
   construction and by assertion respectively, not by evidence.

## 21.9 What this section obliges

- §2 must not imply that content addressing solves global product identity (§21.2).
- §2.6 must state that "any node MAY build an index" does not prevent one index becoming the de
  facto gatekeeper (§21.3, §21.4).
- §0.7 and all self-hosting guidance must distinguish receiving orders from serving a catalogue,
  and must not present an intermittently-online seller as fully functional (§21.5).
- §9.6 and §10 must state the opt-in-escrow and self-dealing failure modes as measured outcomes,
  not hypotheticals (§21.6).
- No section may cite this one as support for logistics, analytics, live-state or legal claims
  (§21.1).
