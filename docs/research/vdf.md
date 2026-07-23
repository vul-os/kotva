# Research: Verifiable Delay Function (VDF) as an optional cold-contact cost

> **STATUS — NON-NORMATIVE / EXPERIMENTAL.** This document is quarantined research, not part of
> the KOTVA conformance surface ([DIRECTION §9](../../DIRECTION.md), [docs/research/README.md
> §5](README.md)). It is **not conformance-tested**, and a conformant KOTVA implementation needs
> none of it. It is retained **verbatim, in full** (moved from `09-anti-abuse.md §9.4.1`) so a
> future graduation has a complete, unmodified starting point.
>
> **Why it moved (2026-07 spec-perfection pass).** The VDF construction rests on an unproven
> sequentiality conjecture, a group of unknown order with a real (if not decisive) trusted-setup
> question, and is explicitly **not post-quantum** — three honest limits the text below states
> itself, at length, as reasons this is a `MAY` rather than a `SHOULD`. Carrying an optional,
> unstandardized (no IETF standard, no interoperable parameter set, no pinned proof encoding)
> construction inside the normative spec overstated its maturity relative to everything around it.
>
> **Current normative position (09-anti-abuse.md).** The interoperable cold-contact floor stays
> **memory-hard proof-of-work (Argon2id-style, §9.4)** — unchanged, still a MUST, not touched by
> this move. §9.7a's zero-relationship delivery floor already states, independently of this
> document, that a conformant recipient **MUST** accept a bare memory-hard PoW solution and **MUST
> NOT** require a VDF as the only acceptable proof — that rule lives in `09-anti-abuse.md §9.7a`
> and did not move. The verifier's own memory-hard verification-cost budget (bounding how many
> Argon2id checks a recipient performs per window) is likewise a property of the still-normative
> §9.4 floor, not of the VDF, and it did not move either — it remains in `09-anti-abuse.md`
> immediately after the stub for this section.
>
> **What follows is the VDF specification exactly as it read in the normative spec before this
> demotion** — the scarcity comparison, why sequential time is an attractive cost dimension, the
> two properties a VDF buys, the conformance boundary (VDF MAY / PoW MUST), and the three honest
> limits (conjectural sequentiality, a residual 10–100× per-gate latency advantage, and the
> non-post-quantum / trusted-setup discussion). The internal section number (`§9.4.1`) and every
> cross-reference **within this document** are preserved unchanged from the original
> `09-anti-abuse.md` so a future graduation can move it back with a pure copy, and so an external
> reference to the old `§9.4.1` numbering still resolves to the right place, just in a different
> file. Cross-references that pointed *outside* old §9.4.1 (e.g. `§9.2a`, `§16.5`, `§1.1`) still
> point at those sections in their home documents, unchanged.

---

### 9.4.1 Sequential work (VDF) as an OPTIONAL cold-contact cost (normative)

Memory-hardness narrows the parallel adversary's advantage; it does not remove it. A GPU farm or
a botnet still computes Argon2 far faster in aggregate than a phone, so the cost gradient
continues to run **against** the legitimate low-power sender and **for** the organized one —
which is the wrong way round for a mechanism whose entire purpose is to make bulk sending
uneconomic. Against precisely the adversary this protocol is designed to withstand — one whose
defining advantage is **access to a great deal of compute** — a compute-denominated puzzle is the
weakest possible choice of scarcity.

Rank the available scarcities by how much an adversary's compute budget helps them:

| Scarcity | Does rented compute help? | Mechanism |
|---|---|---|
| **Social proximity** | Not at all — cannot be bought | vouch / introduction (§9.7) |
| **Money** | No — cost is linear, no asymmetry to exploit | postage (§9.5) |
| **Sequential time** | *Aggregate* parallelism buys ≈ nothing; a ≈ **10–100×** per-gate latency advantage remains | **VDF (this section)** |
| **Parallel compute** | Yes — the whole compute budget applies | memory-hard PoW (§9.4) |

A **Verifiable Delay Function** — a Wesolowski- or Pietrzak-style sequential-squaring VDF over the
same scope as §9.4 (`id ‖ recipient ‖ nonce(epoch)`, §16.5), delay parameter set so one legitimate
message costs a few seconds of wall-clock — is therefore an attractive shape of cost, for two
reasons:

- **It bounds the *aggregate-parallelism* advantage.** Repeated squaring is believed not to
  parallelize, so 100 000 rented cores compute one puzzle no faster than one core does: a botnet's
  breadth buys it essentially nothing *per puzzle*. **This is not the same as flattening the
  hardware spread.** What survives untouched is the
  **per-gate latency constant factor** — faster silicon, a tighter multiplier, an ASIC — which the
  VDF literature itself budgets as a routine design parameter (a ≈ 10× faster-evaluator allowance)
  and separately reasons about at ASIC scale (≈ 100×). The honest claim is therefore: a VDF turns
  *"as fast as your whole fleet"* into *"as fast as your single best circuit"*, a **10–100×**
  residual advantage, not a ≈ 10× total spread against ≈ 1000×.
- **Verification is asymmetric by construction, which independently fixes a DoS hole.** A VDF
  proof verifies in milliseconds regardless of how long it took to produce. This removes the
  problem §9.4 is forced to work around above: because Argon2id verification costs the *recipient*
  roughly what it cost the *sender*, a flood of bogus PoW attachments is itself a memory/CPU
  denial-of-service on the recipient, which is why that clause must impose a verification budget
  and defer unverified MOTEs. A VDF has no such symmetry — a recipient can verify every proof it
  is offered, cheaply, and needs no defer-without-verifying escape hatch.

**Conformance — VDF is MAY; memory-hard PoW is the MUST floor (normative).** The interoperable
floor is the older mechanism, deliberately, and the VDF is an option layered above it rather than
a recommendation:

- A recipient's `ChallengeSpec` (§9.2) MAY specify `vdf(delay)` **alongside** `pow(bits)`, and an
  implementation **MAY** prefer `vdf` where both parties support it. It is deliberately **not** a
  `SHOULD`: the disclosures below — a conjectural, processor-bounded sequentiality assumption and
  a construction that is **not post-quantum** — are not the grounds on which this specification
  recommends anything.
- A conformant recipient **MUST accept a valid memory-hard PoW solution** (§9.4) as satisfying its
  cold-contact requirement, subject only to the verification budget below and to the recipient's
  own rate policy. It **MUST NOT** require a VDF as the *only* acceptable proof, because that would
  make reachability conditional on an unstandardized construction (below) — and a sender who cannot
  produce the one proof a recipient will take is simply undeliverable, which §9.7a exists to
  prevent. A VDF-only cold-contact policy is therefore itself non-conformant
  (`ERR_POLICY_BELOW_FLOOR`, `0x070F`, §21.10), exactly as any policy under which a valid work
  proof is never sufficient is.
- The binding rule of §9.2a applies to both: the proof's scope MUST include `sender_key`, so a
  stolen proof is worthless under any other ephemeral key. Parameters are pinned in §16.5.

**Honest limits — the three reasons this is a MAY and not the floor.** Each is a property of the
construction, not a maturity opinion:

1. **Sequentiality is a conjecture, and it is bounded by a processor count — not a theorem.** The
   foundational VDF definition permits `Eval` up to **poly log(t) parallelism**, and observes that
   constructions requiring *no* parallelism are "unlikely to exist (without trusted hardware)".
   Sequentiality is defined only **relatively**: a VDF is `(p, σ)`-sequential against an adversary
   holding at most `p(t)` processors. Both the Wesolowski and Pietrzak constructions then inherit
   their sequentiality from an **unproven belief** about repeated squaring in a group of unknown
   order. "Cannot be parallelized" is therefore shorthand for "is believed not to parallelize,
   against an adversary below a stated processor bound" — which is a perfectly usable engineering
   assumption for a spam cost, and not one to build a MUST on.
2. **It bounds aggregate parallelism only** — see above. A **10–100×** per-gate latency advantage
   for better silicon remains fully intact, so the gradient is improved, not inverted.
3. **It is not post-quantum, and the rest of this suite is.** Both named constructions rest on a
   **group of unknown order**; a quantum adversary computes that order and collapses `t` sequential
   squarings into a **single reduced exponentiation**, destroying exactly the sequentiality the
   anti-abuse use depends on. **No simple post-quantum VDF exists to substitute.** Pinning a
   PQ-hybrid suite as the REQUIRED default (§1.1, justified by harvest-now-decrypt-later) while
   *recommending* an RSA-group or class-group VDF would be internally inconsistent.

   **Why that inconsistency is tolerable rather than fatal — stated plainly rather than hidden.**
   The two exposures are not the same kind of thing. A quantum break of the KEM is
   **retroactive**: traffic recorded today is decrypted later, and nothing done afterwards repairs
   it — which is why §1.1 pays for PQ up front. A quantum break of the VDF is **prospective and
   local**: it makes cold-contact work cheap *from that day forward* for whoever holds the machine,
   costing the recipient some additional junk in the **requests area** (never the inbox, §2.7a),
   and it is repaired by the recipient raising `pow(bits)` or leaning on the vouch (§9.7), postage
   (§9.5) and ARC (§9.3) tiers, none of which depend on it. A future spam-cost problem is
   survivable; a retroactive confidentiality catastrophe is not. The VDF is confined to the tier
   where that asymmetry holds, and is a `MAY` so that no reachability ever depends on it.

**And a setup problem, weaker than the strong form of the objection.** A group of unknown order is
in practice either an **RSA modulus from a trusted setup** — whoever generated it can shortcut
every puzzle — or a **class group** of an imaginary quadratic field, which needs no setup. The
trusted-setup objection is real but not decisive: the same literature offers a **sufficiently
large random `N`** as an alternative (noting the cost — it gives the adversary *more*
parallelization opportunity and increases verifier time) as well as class groups. What actually
keeps a VDF out of the floor is not that class groups are unproven — this specification makes no
such claim — but that there is **no IETF standard, no interoperable parameter set and no pinned
proof encoding** for either choice, so two independent implementations cannot be expected to agree
byte-for-byte. That is the opposite of what a floor is for. Memory-hard PoW is the weaker mechanism
that everyone can implement today, which is exactly what makes it the floor.
