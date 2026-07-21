# Decision note — EU VAT deemed-supplier status

**What this document is.** A decision note on TRACT's single most adverse legal finding to date
(§21.11.3, carried into the spec at §11.2b). It lays out the options for a **gateway operator**
(§0.4.2) and recommends one.

**What this document is not.** This is an engineering reading of a regulation and a Commission
explanatory note, produced by the same adversarial-verification process behind §21. It is **not
legal advice**, has not been reviewed by a lawyer, and should not be relied on for a filing
position. Where the finding is genuinely unresolved, this note says so rather than picking a side.

**Date.** 2026-07-21.

**Scope boundary, stated first because it bounds how alarming this is.** This is a question about
what a gateway operator does and discloses. It is not a question about the wire format. Nothing
below requires a change to any TRACT object, message, or state machine. A protocol that carries
signed attestations between two keypairs is unaffected by which party a tax authority treats as
liable for VAT on a subset of transactions that flow through one optional, permissionless,
competing operator role. The finding constrains gateway operators, not implementors of the spec.

---

## 1. The finding, and why it is worse than the US one

§21.11 ran one legal question to a favourable-but-narrow answer (US state marketplace-facilitator
tests are conjunctive and gated on a contract with the seller — a gateway no seller contracted with
is textually outside them) and one to an adverse one. This note is about the adverse one.

Art 5b of Council Implementing Regulation (EU) 282/2011 treats an electronic interface as **not**
facilitating a supply only if, cumulatively, it:

- (a) does not set, directly or indirectly, **any** of the terms and conditions of the supply;
- (b) is not, directly or indirectly, involved in authorising the charge to the customer; and
- (c) is not, directly or indirectly, involved in the ordering or delivery of the goods.

The Commission's Explanatory Notes say carrying out even **one** of those may suffice to make the
interface a deemed supplier. And then, verbatim, they name and reject TRACT's central claim:

> "the indication that the seller … is responsible for the goods sold via a marketplace/platform or
> that the contract is concluded between the underlying supplier and the customer **is not
> sufficient**" to escape deemed-supplier status, because the concept "goes beyond the contractual
> relationship and looks at the **economic reality** and in particular the **influence** exercised".

"Indirectly" and "any" are stated to exist precisely to prevent "artificial splitting of rights and
obligations between the electronic interface and the underlying suppliers".

**Why this is more serious than the US finding.** The US result is a textual gap nobody has tested
in court — favourable, but silent on whether it would survive contact with a regulator motivated to
close it. The EU text is not silent. It was drafted, with named examples, specifically to catch the
move TRACT makes: declaring that the contract runs directly between the two parties and that the
platform is merely infrastructure. Where the US finding is *untested*, the EU finding is *pre-argued
against*. There is no textual gap to stand in; the drafters anticipated this gap and closed it with
the words "indirectly" and "any".

## 2. What the finding does not reach

Art 14a (via 5b) only bites on:

- imported consignments of intrinsic value **at or below EUR 150**, and
- supplies **within the EU** made by sellers **not established in the EU**.

It is not a general rule for all EU commerce. A gateway whose transaction flow never touches either
case is outside Art 14a on its face, for the same reason a purely domestic EU-to-EU sale above €150
is outside it today. That boundary is real, and is the basis for Option B below.

It is also worth restating plainly: escrow is what the literature identifies as the most reliable
trigger for facilitator status generally (§21.11.2, §9.5), and it is the "(b) authorising the
charge" prong of Art 5b specifically. A gateway that never settles has a materially different
position under limb (b), though limbs (a) and (c) can still be tripped by storefront rendering and
order-taking alone — see Option C.

## 3. Options

| | Operationally | Reach preserved | Cost | Residual risk |
|---|---|---|---|---|
| **A. Accept deemed-supplier status where Art 14a applies** | Gateway registers for VAT (IOSS/OSS or local), charges, collects, and remits VAT on the in-scope transactions it facilitates, as if it were the seller **for VAT purposes on those transactions only** | Full — no change to who a gateway can serve | Ongoing: VAT registration, IOSS/OSS or destination-country registration, remittance compliance, per-transaction VAT calculation and evidencing | Compliance execution risk; does not by itself resolve non-EU or DAC7-adjacent reporting questions (§21.11.5, unresolved) |
| **B. Scope gateways out of Art 14a's triggering cases** | Gateway declines to serve imports ≤ €150 and declines to serve non-EU-established sellers selling into the EU (or fences that traffic to a separately-operated, VAT-registered entity) | Reduced — excludes low-value imports and non-EU-established sellers into the EU, a real and growing share of cross-border e-commerce | Low ongoing cost; a policy/geofencing decision, not a compliance programme | Scope creeps if ViDA expands deemed-supplier categories (§21.11.5); must be actively maintained, not set once |
| **C. Separate the roles — render-only, settlement elsewhere** | Gateway operates storefront rendering (§0.4.2, §12) but never holds funds; the seller's own PSP or a third-party escrow settles. Removes limb (b) cleanly | Preserves most of the design's premise — the storefront-optional/settlement-optional split TRACT already has at §0.4.2 | Low — this is closer to §0.4.2's existing bundling-is-a-choice structure than a new build | Does **not** clear limbs (a)/(c): a storefront that lists terms or takes orders can still trip Art 5b on its own. And it is only a **partial** answer in the US — render-only is safe in NY/TX, likely caught in WA/CA (§21.11.2) — so this option helps in the EU and only partly in the US |
| **D. Change nothing, disclose the exposure** | Gateway continues bundling storefront + escrow as §0.4.2 describes today, with the deemed-supplier exposure stated in operator-facing documentation | Full | None beyond the disclosure itself | The exposure is real and uncosted: unregistered VAT liability on in-scope transactions, with the gateway operator — not the protocol, not the seller, not TRACT's authors — carrying it |

A note on D: "carries it" means the gateway operator, as the entity that in economic reality set
terms, authorised charges, or touched ordering/delivery on the affected transactions, is the one a
tax authority would assess. TRACT the specification carries no VAT liability under any option,
because TRACT specifies no operator and mandates no gateway (§0.2, §9.2). This is squarely an
operator-level exposure.

## 4. What Option A does and does not mean

Worth stating precisely, because "deemed supplier" is easy to over-read:

- **It does mean**: for VAT purposes, on the in-scope transactions, the gateway is treated as if it
  bought the goods from the seller and sold them to the customer — a legal fiction confined to VAT
  collection and remittance. The gateway registers, charges the correct rate, collects, and remits.
- **It does not mean**: the gateway becomes the seller of record for the transaction generally, the
  counterparty on the sale contract, the party liable for product conformity, consumer-protection
  obligations, or anything outside the VAT fiction. §11.3's "seller of record" field is untouched.
  It also does not retroactively validate the US marketplace-facilitator arguments in §21.11.1 —
  those are governed by separate, conjunctive, contract-gated tests, not by the EU VAT fiction.

## 5. Recommendation

> **Adopted 2026-07-21 as decision D1, with one correction to the reasoning below.** See
> [DECISIONS.md](DECISIONS.md#d1--eu-vat-posture). The adopted posture is **outside Art 14a by
> default, inside by deliberate decision with an IOSS registration** — not "outside permanently".
> And **Option C is hygiene, not a defence**: Art 5b needs all three limbs clear, separating
> settlement clears limb (b) alone, and a storefront where a buyer clicks buy is involved in the
> ordering (limb (c)). The analysis below stands; the weight it gives C does not.

**B, with C as the default posture for gateways that don't take Option B.**

Reasoning:

- Option B costs the least to implement and preserves the cleanest legal position: a gateway that
  never serves ≤€150 imports or non-EU-established sellers into the EU is outside Art 14a on the
  same textual grounds as the US argument in §21.11.1 — a real boundary, not a hope. It is the
  option most consistent with "permissionless to enter and competing" (§0.4.2): a gateway operator
  can choose this scope without asking the protocol for anything.
- Option A is the right answer for a gateway operator who *wants* to serve the excluded traffic
  (low-value imports, non-EU sellers) — that is a large and legitimate market — but it is a
  compliance programme, not a design choice, and should be a deliberate business decision by
  whoever runs that gateway, made with actual legal and tax advice, not inferred from this note.
- Option C is complementary, not a substitute: it clears one of three limbs (authorising the
  charge) and is worth doing regardless of A or B, because it also improves the US position in two
  states (§21.11.2) and is close to free given §0.4.2 already treats storefront and escrow as
  separable functions bundled by choice, not by necessity. It does not clear the EU exposure alone.
- Option D is not a recommendation, it is a fallback for a gateway operator who has weighed A/B/C
  and still wants to bundle storefront and escrow into the excluded traffic. It should be a
  conscious decision with the exposure written down, not silence.

**What preserves the most of the design's premise:** Option D, trivially, because it changes
nothing — but that is exactly why it is not the recommendation; it preserves the premise by leaving
the exposure uncosted. Among the options that actually change the operator's risk, **C** preserves
the most of the "storefront and escrow are one bundled-by-choice role, not two" structure in §0.4.2,
because it keeps both functions available, just not simultaneously mandatory for the same legal
entity on the same transaction.

**What is cheapest to implement:** **B**. It is a scoping decision, not a build. A gateway operator
who simply declines the two triggering transaction classes needs no new compliance surface at all.

## 6. What would change this answer

- **A CJEU interpretation of Art 5b.** None exists. The Explanatory Notes disclaim binding force,
  and no court has tested how "indirectly" and "any" apply to a protocol where the platform asserts
  — truthfully, on TRACT's own object model — that it never sets prices, only renders a seller's
  signed offer as published. Whether a court would treat faithful rendering of seller-set terms as
  "indirectly setting terms" is exactly the open question; the Explanatory Notes address the
  argument, not this specific fact pattern.
- **ViDA.** The EU's VAT in the Digital Age package is expected to expand deemed-supplier scope.
  §21.11.5 already flags this as pending; it would very likely narrow, not widen, the safe harbour
  Option B relies on, and should be re-checked against this note when it lands.
- **Actual legal advice.** This note is an engineering reading of primary and Commission-published
  text. A qualified EU VAT practitioner may read Art 5b differently, particularly on limb (a) as
  applied to a gateway that renders but does not author terms. None of the options above should be
  acted on by a real gateway operator without that advice.

None of the three exists yet. Until one does, this note's recommendation stands as the best
available reading, not a settled position.
