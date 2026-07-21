# Help wanted

Everything below is blocked on **expertise this project does not have**, not on effort. Several
items have already had multiple research passes that returned nothing verifiable — those are marked
as such, because "we tried three times and failed" is more useful to a would-be contributor than
"todo".

If you can close any one of these, you will move the specification further than another month of
writing would.

---

## 1. A second implementation — the single most valuable contribution

**What:** build anything that speaks TRACT §16 without reading [Soko](https://github.com/vul-os/soko).

**Why it matters more than anything else here.** §16 is frozen at v0 and 21 conformance vectors
exist, derived by hand from the specification text. What none of that establishes is whether a
competent reader who has *not* seen the reference implementation can build from this document and
interoperate. Right now the spec has exactly one implementation, and an implementation and a
specification derived from each other only prove they agree with one another — which is precisely
the failure `conformance/README.md` records from the sibling DMTAP project, where a spec, a frozen
vector and three tests were all self-consistently wrong together.

**Where to start:** §16 (wire format, normative), then `conformance/vectors/` — every expected value
shows its arithmetic, so you can check your encoder against the spec without running anything.

**What would be most useful:** a report of every place the document was ambiguous, underspecified,
or wrong. Especially wrong. A disagreement between your implementation and the vectors is a finding,
not a bug in your code, until proven otherwise.

**Skills:** any language with a deterministic CBOR encoder and Ed25519.

---

## 2. Data-protection law — the most likely hard blocker

**What:** whether TRACT's erasure model actually satisfies POPIA, GDPR and LGPD.

**Status: three research passes, nothing verified.** This is not an unexplored area; it is one that
resisted being answered by reading. §22 states the conflict exactly and deliberately does not
pretend to solve it.

**The specific questions:**

- Does destroying an encryption key satisfy a right to erasure, or does it merely make data
  inaccessible? Regulator positions appear to differ and we could not establish a settled one.
- When a sealed order exists at two endpoints belonging to different natural persons, who is the
  controller? Is joint controllership the right frame, and where does the household exemption end
  for someone self-hosting a shop?
- A published review is signed by a per-subject pseudonymous subkey. That is still personal data. Is
  a tombstone honoured by conformant holders a defensible answer where deletion is impossible?

**What would help:** a written opinion, or even a clear "this is unsettled and here is why", from
someone qualified in any one of these jurisdictions. A partial answer for one country is worth more
than speculation about all three.

**Skills:** data-protection practitioner. This will not be closed by another literature pass.

---

## 3. EU VAT — a decision made on an engineering reading

**What:** review [DECISION-vat-facilitator.md](DECISION-vat-facilitator.md) and D1 in
[DECISIONS.md](DECISIONS.md).

**Status: decided provisionally, explicitly pending advice.** The finding is that Art 5b of
Implementing Regulation 282/2011 anticipates and rejects the argument that a contract concluded
directly between two parties relieves an interface of deemed-supplier status.

**The specific question we could not settle:** limb (a) — "does not set, directly or indirectly, any
of the terms and conditions" — as applied to a gateway that *renders a seller's signed offer without
authoring it*. Faithful rendering of terms someone else set is arguably not setting them. The
Explanatory Notes address the general argument, not this fact pattern, and no CJEU decision
interprets Art 5b at all.

**Skills:** EU VAT practitioner. Also useful: anyone who has actually operated under IOSS and can
say whether the compliance load matches what D1 assumes.

---

## 4. Logistics — §8 is unevidenced

**Status: targeted directly by research, returned nothing verifiable.**

**The specific questions:**

- **Do carrier developer API terms permit republishing rate data?** §8's entire model is that rate
  cards are *published* so routing can be computed locally. If the major carriers forbid
  redistribution of negotiated rates, that changes what the section can claim. We could not
  establish the terms with confidence.
- How is multi-origin consolidation actually optimised in practice, and how much of it is genuinely
  computable client-side from published inputs versus needing a live quote?
- How are peer couriers classified for employment and liability purposes? §14 flags this as open,
  and it bears on whether the courier role is permissionless in the world as well as in the
  protocol.

**Skills:** anyone who has integrated a carrier API and read the contract, or worked in freight
forwarding. Practitioner knowledge beats literature here.

---

## 5. Privacy-preserving analytics — §13 is unevidenced

**Status: targeted directly, returned nothing verifiable.**

§13 names Prio-style aggregation, browser-vendor attribution work, anonymous credentials and
Oblivious HTTP as *intentions* rather than evaluated fits, and says so.

**The specific questions:** what fidelity is genuinely achievable, what is the minimum volume for a
useful signal, and — the one that matters for this design's actual users — **what happens to a
small seller?** Aggregation needs volume, and the sellers this project exists to serve are the ones
least likely to have it.

**Skills:** anyone who has shipped privacy-preserving measurement in production.

---

## 6. Reputation attack literature — §10's floor is unknown

**Status: unresearched after multiple attempts.**

§10 uses purchase-attested reviews and local-only scoring. §21.6 records that this does **not** stop
self-dealing: a seller transacting with itself produces genuine attestations.

**The specific question:** what is the achievable Sybil-cost floor for reputation on a signed-feed
substrate? Do buyer counter-signatures or transaction-cost binding measurably help, and what does
the attack literature say about the limit?

**Skills:** familiarity with Sybil-resistance and reputation-system research.

---

## 7. Adversarial review — of anything

The two most valuable contributions this project has received were both adversarial: a review that
found six invariants documented but not structurally enforced, and a conformance pass that found
the specification contradicting itself between §11.3 and §16.6.

**If you want to help and none of the above fits, try to break something.** Particularly welcome:

- a claim in a section that the section does not actually support
- a place where §21 or §22 records a limit that a *different* section quietly contradicts
- an invariant asserted in prose with nothing enforcing it

---

## What this project will not accept

Stated so nobody wastes their time:

- **A token, a coin, or a protocol fee.** Not now, not later.
- **A canonical ranking or reputation service.** It would be the authority the design removes.
- **A global product registry.** §2's identity ladder is deliberately permissionless at the floor.
- **Cross-seller distributed transactions.** They need a coordinator with authority over sovereign
  parties; §21.10.3 records that sagas provide neither atomicity nor isolation, which makes
  compensating actions the correct shape rather than a limitation.

## How to raise something

Open an issue on [tract](https://github.com/vul-os/tract) for the specification, or
[soko](https://github.com/vul-os/soko) for the implementation. If you are unsure which, the
specification is usually the right place — several implementation bugs have turned out to be
specification gaps.

If your finding is that something here is **wrong**, that is the most welcome kind. §21 exists
because the evidence contradicted the design and the document says so; a further contradiction is
a contribution, not a complaint.
