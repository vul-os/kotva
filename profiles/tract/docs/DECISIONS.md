# Decisions

Decisions that have been taken, with the reasoning and the date. One line per decision; the long
form lives in its own note where there is one.

This exists because the alternative is re-litigating the same question every few weeks with worse
recall each time, and because a decision whose reasoning was never written down is indistinguishable
later from an accident.

**Status vocabulary.** *Adopted* — decided, and the spec and implementation reflect it. *Provisional*
— decided on current evidence, expected to be revisited when something specific changes.
*Open* — genuinely undecided, and named here so it is not mistaken for settled.

| # | Decision | Status | Date | Detail |
|---|---|---|---|---|
| D1 | **EU VAT posture: start outside Art 14a; register IOSS deliberately when entering it** | Adopted (provisional on legal advice) | 2026-07-21 | [DECISION-vat-facilitator.md](DECISION-vat-facilitator.md) |
| D2 | **Erasure is not absolute; refusals name their statutory basis and expiry** | Adopted | 2026-07-21 | §22, `soko-erasure` |
| D3 | **§16 wire format frozen at v0** | Adopted | 2026-07-21 | §16, `conformance/` |
| D4 | **One operator class (gateway), and it does not self-extinguish** | Adopted | 2026-07-21 | §0.4.2, §0.4.3 |
| D5 | **No personal data in the public quadrant, ever** | Adopted | 2026-07-21 | §0.5.1, §16.4, §22 |
| D6 | **No network-wide published reputation score** | Adopted | 2026-07-21 | §10.3 |
| D7 | **Settlement names no provider** | Adopted | 2026-07-21 | §9.2, `soko-seam` |

---

## D1 — EU VAT posture

**Decided:** operate where Art 14a does not reach, and treat entering its scope as a deliberate,
separately-taken business step with an IOSS registration behind it. Do not rely on separating
storefront from settlement as an EU defence.

**Why, in the order the reasoning actually runs:**

**Role separation is hygiene, not a defence.** Art 5b requires *all three* limbs to be clear — sets
none of the terms, not involved in authorising the charge, not involved in ordering or delivery.
Separating settlement clears limb (b) alone. A storefront where a buyer clicks buy is involved in
the ordering, which is limb (c), so a rendering gateway is plausibly caught whoever holds the money.
This is the correction to the earlier framing in the note, which treated separation as a
complementary defence; it is worth doing for other reasons (it improves the US position in two
states, and §0.4.2 already treats the bundle as a choice) but it does not clear Art 5b.

**The first market is already outside the rule.** Art 14a bites on imported consignments of
intrinsic value at or below €150, and on supplies within the EU by sellers not established in the
EU. A South African seller, a South African buyer and a South African gateway are outside it
entirely, as is EU-to-EU trade above €150. Operating there is not a defensive contortion — it is
where the wedge market already is, and it costs nothing.

**Entering the EU is a registration, not a compliance programme.** The earlier note treated
accepting deemed-supplier status as heavy. IOSS is a **single registration covering all 27 member
states**, built precisely for this case. Low-value cross-border is the fastest-growing segment in
e-commerce, and permanently fencing it off to avoid one registration is a bad trade — so the
posture is "outside by default, in by decision", not "outside permanently".

**Never simply carry the exposure silently.** An operator that bundles storefront and escrow into
in-scope traffic without registering is running an unregistered VAT liability. If that is chosen, it
is chosen out loud and written down.

**What the protocol changes:** nothing. This constrains what a gateway operator serves and
discloses. No TRACT object, message or state machine differs under any option, and TRACT itself
carries no VAT liability because it specifies no operator and mandates no gateway (§0.2, §9.2).

**What would reopen it:** a CJEU interpretation of Art 5b; ViDA expanding deemed-supplier scope
(expected, and likely to narrow the boundary D1 relies on); or actual advice from an EU VAT
practitioner, particularly on limb (a) as applied to a gateway that renders but does not author
terms. None exists yet. **This is an engineering reading, not legal advice, and no real operator
should act on it without counsel.**

---

## D2 — Erasure is not absolute

**Decided:** a holder refusing an erasure request names the statutory basis and the date it lapses.
Erasure outcomes are not a boolean.

**Why:** POPIA section 14 requires retention to stop once the purpose is served — *unless retention
is required by law* — and tax law then requires exactly that, five years under South Africa's Tax
Administration Act and comparable periods across the EU. A system that silently deletes has broken
the seller's tax position; one that silently refuses has misinformed the subject. Both failures are
avoidable by making the refusal legible.

A single request routinely produces several dispositions at once — some records deleted, some held
until a tax period closes, some published irrevocably and only tombstonable — so "did it work?"
cannot express the answer. `soko-erasure`'s `Outcome` carries each disposition separately, and a
`Tombstone` is structurally incapable of being reported as an `Erased`.

**The residual, unchanged:** for anything published, a tombstone honoured by conformant holders is
all there is. That is why D5 exists — the defence is publishing nothing personal, not building a
deletion mechanism that cannot deliver.
