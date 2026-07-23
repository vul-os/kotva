# 9. Anti-Abuse & Postage

DMTAP must stop spam/abuse **without** a central filter and **without** breaking sealed-sender
anonymity. The tension: sealed sender hides who sent a message, but the recipient still needs
to rate-limit and block abusers. The resolution is **anonymous but accountable** tokens.

## 9.1 Principles

1. **Authenticated-by-default.** Every native MOTE is authenticated to the recipient
   (payload signature, §2.4); there is no anonymous unauthenticated injection like open SMTP.
2. **Recipient policy is local.** The recipient node decides what to accept — allow known
   contacts, challenge strangers, block per-identity — *before* surfacing to the user (§2.7).
3. **Cost for cold contact.** Reaching a stranger costs something (a token, proof-of-work, or
   a paid stamp), so bulk unsolicited sending is uneconomic. Known contacts are free.
4. **Anonymity preserved — with one disclosed, structural exception.** Abuse control MUST NOT
   deanonymize the sender or link a sender across recipients. The vouch (§9.7) is the sole,
   structural exception — see §9.7 for its full disclosure. This principle binds ARC (§9.3), PoW
   (§9.4), and postage (§9.5) without exception; it does **not** bind the vouch, and no
   implementation or operator MAY describe vouch-based introduction as anonymity-preserving
   (§6.9 SP-11, SP-5).

## 9.2 Recipient policy

```
Policy {
  allow:      [* ContactRef],   // known keys / group members → free, always accept
  challenge:  ChallengeSpec,    // unknown senders must satisfy this
  block:      [* Token/KeyRef], // per-identity or per-token blocks
  rate:       RateLimit,        // per-sender-token limits
}
ChallengeSpec = pow(bits) / token(issuer) / stamp(amount) / vouch(guardianSet)
```

A stranger's first MOTE MUST carry a valid challenge response or it is dropped/deferred to a
"requests" area, never delivered to the inbox.

### 9.2a Binding proofs to the message (theft/replay prevention — normative)

Every `ChallengeResponse` is carried in the **cleartext** envelope so it can be checked before
decryption (§2.7). That means an on-path observer sees it. A proof that is valid *in isolation*
could therefore be **stripped from a victim's MOTE and re-attached to the attacker's own MOTE**
(the attacker mints a fresh ephemeral `sender_key`, copies the stolen proof into `challenge`, and
signs — both `sender_sig` and the stolen proof then verify). To close this, **a
`ChallengeResponse` MUST be cryptographically bound to the envelope's ephemeral `sender_key`**
(§2.2, field 12), so a stolen proof is worthless under any other ephemeral key:

- **ArcToken** — the ARC presentation MUST be computed over a request context that includes
  `sender_key` (in addition to `origin`). A verifier MUST reject a presentation whose context
  does not bind the `sender_key` of the carrying envelope. This is what makes the token
  single-holder without deanonymizing it (ARC stays cross-recipient-unlinkable, §9.3).
- **PowSolution** — already bound: the `epoch_nonce` scope is `id ‖ recipient ‖ nonce(epoch)`
  (`nonce(epoch)` is the recipient's published epoch beacon, defined in §9.4; parameters §16.5)
  and `id` is the content address of *this* ciphertext, so a stolen PoW only "works"
  on the identical MOTE, which the recipient de-duplicates by `id`. Implementations SHOULD
  additionally fold `sender_key` into the PoW scope for defense in depth.
- **PostageStamp** — theft is bounded by the online single-use redemption check (§9.5.1): the
  first party to redeem `serial` wins and the stamp is spent, so a stolen stamp cannot be
  redeemed twice. The presentation MUST additionally name `sender_key` in the redemption request
  so the issuer binds the spend to the presenting ephemeral key.
- **Vouch** — a vouch **cannot** be bound to `sender_key` at mint time: the voucher cannot know a
  key the vouchee has not yet generated, and a cleartext proof-of-possession over `sender_key`
  would break sealed sender (§6.2). It is therefore bound to the **subject it names** instead:
  §2.7 step 8(b2) requires `Payload.from == VouchToken.subject`, and a mismatch is discarded
  without an `ack` (`ERR_VOUCH_SUBJECT_MISMATCH`, `0x0126`).
  A stolen vouch is not caught by identity authentication alone: step 8(a) verifies
  `Payload.sig` under `Payload.from`, a field the thief chooses and signs with their own key, so
  it succeeds. Since a vouch travels in the cleartext envelope (this section's own premise), any
  on-path observer could lift one and present it as their own — acquiring, at zero cost, the tier
  §9.7 calls the only one "an adversary cannot buy with either compute or money" and which bypasses
  VDF/PoW/stamp entirely.
  **Honest residual.** The binding is necessarily **post-decryption** — `from` is not visible
  earlier — so a lifted vouch still buys the thief one decryption before rejection. The per-
  `subject` rate limit (§9.7) MUST therefore be applied at the **gate**, and because a replayed
  vouch is charged against the *subject's* budget rather than the thief's, a subject whose vouch is
  being replayed MUST be surfaced to the recipient rather than silently rate-limited into
  invisibility — otherwise the mechanism becomes a way to frame the vouched-for party.

Because `challenge` is inside the `sender_sig` preimage (§18.9.1) and `sender_sig` is made by
`sender_key`, this binding also proves the proof and the signature came from the *same* ephemeral
key for this exact envelope.

## 9.3 Anonymous rate-limit tokens (the core mechanism)

DMTAP uses **Privacy-Pass anonymous tokens** (RFC 9576 architecture, RFC 9577 HTTP scheme,
RFC 9578 issuance) so a recipient can rate-limit/block a sender **without learning who they
are** and **without the sender being linkable across recipients**.

**CAUTION — pick the right variant.** Vanilla Privacy Pass tokens are issuance-unlinkable but
do **not**, alone, give the exact combination DMTAP needs: *per-recipient rate-limiting* AND
*cross-recipient unlinkability*. That combination requires per-origin-scoped issuance —
**Anonymous Rate-limited Credentials (ARC)**, specified in the Privacy Pass space by
`draft-ietf-privacypass-arc-protocol` — a Privacy Pass **WG-adopted** Standards-Track Internet-Draft (formerly the individual `draft-yun-privacypass-arc`), still a draft, not an RFC. DMTAP tokens MUST use ARC-style per-origin binding, not plain
tokens, for the anonymity + accountability guarantee to hold simultaneously.

- A sender obtains blinded tokens from an **issuer** bound to a *rate budget*.
- The sender attaches a token to a cold MOTE; the recipient verifies it is valid and unspent,
  enforces per-token rate limits, and can **block a token/issuer** if it misbehaves — all
  without identity.
- Tokens are **unlinkable across recipients** (blind signatures), so token use does not build
  a cross-recipient graph.

### 9.3.1 Issuer trust is what gives a token value (normative)

A token is only as trustworthy as its **issuer**, and issuance MUST have a cost or scarcity or
the whole "cost for cold contact" model collapses (a spammer would self-issue unlimited free
tokens). Therefore:

- The recipient's policy scores issuers (parallel to gateway reputation, §7.5). A token's
  granted rate budget is a function of **issuer trust at the recipient**, not of the token
  alone.
- A token from an **unknown/unvetted issuer (including the sender's own node) carries a default
  rate budget of ZERO** — i.e. it counts as *no token*, forcing fallback to PoW (§9.4) or
  postage (§9.5). Self-issuance thus buys anonymity but **no cost relief**.
- Trusted issuers (a recipient's own provider, an attested gateway operator, a vouched community
  issuer) make issuance costly/rate-limited on their side and put their own standing behind not
  minting for abusers; a recipient extends budget to their tokens.

This is the load-bearing rule that makes ARC anti-abuse real rather than bypassable.

### 9.3.2 The unlinkability ↔ collective-defense tension (honest)

ARC's cross-recipient unlinkability is in **direct tension** with identifying a repeat spammer
who hits many recipients: the very property that protects a legitimate sender's social graph
also prevents recipients from cross-linking an abuser. DMTAP resolves this at the **issuer
layer, not the recipient layer**: repeat abuse is bounded by the issuer throttling/revoking its
own issuance to a misbehaving client (who is *not* anonymous to its own issuer), and by
recipients down-scoring issuers that mint for abusers. Recipient-side cross-recipient linking is
deliberately unavailable; DMTAP does not claim it. (Signal solves the analogous problem with
delivery tokens; DMTAP generalizes it to open, issuer-scored anonymous tokens.)

## 9.4 Proof-of-work (fallback)

Where no token issuer is available, a stranger MAY attach a **proof-of-work** — a puzzle whose
scope is `id ‖ recipient ‖ nonce(epoch)` (§16.5) — whose difficulty the recipient policy sets.
PoW imposes cost on bulk senders while remaining trivial for a single legitimate message.

**The freshness source (normative — this clause defines `nonce(epoch)`, referenced by §9.2a and
§16.5).** `nonce(epoch)` is the recipient's **current published epoch beacon**: a value
published in the recipient's directory/`Identity` record and rotated on the §16.5 cadence, so a
cold sender fetches it from the resolution path it already uses (§3) **without contacting the
recipient's node**. A recipient that publishes no beacon falls back to the **current UTC date**
— the coarsest permissible beacon. Either way, the epoch scope is what makes precomputed
puzzles stale.

**Acceptance MUST be sender-agnostic (normative).** A recipient **MUST accept a work proof scoped
to *either* its current published beacon *or* the UTC-date fallback** (within the §16.1 clock-skew
window), and **MUST NOT reject a proof solely because it used the coarser scope**.

Without this rule the fallback is unusable, because **it is conditioned on the recipient's
behaviour but the choice is made by the sender**. A sender that cannot *fetch* the beacon cannot
distinguish "this recipient publishes none" from "I cannot reach the place it is published" —
censorship, an eclipsed resolver, an offline directory all look identical. It computes over the
UTC date, a recipient that *does* publish rejects the proof as out-of-scope, and the message is
**silently undeliverable with no error the sender can act on**.

The case that makes this decisive is the one §9.7a exists for. A **key-name-only identity**
(resolver type `self`) has **no publication surface at all**: §3.12.4 defines its discovery as a
local derivation with "no lookup, no authority", so there is no `_dmtap` record and no directory
in which a beacon could live. Requiring the beacon would make the zero-relationship floor
unreachable for precisely the sovereign, no-infrastructure user §3.13 promises it to — the floor
would guarantee acceptance of a proof that user cannot construct.

**The cost, stated plainly:** the UTC-date fallback is **coarser**, so it bounds precomputation to
a day rather than to the §16.5 beacon cadence. That is the correct trade — a day-bounded precompute
window still makes stockpiled puzzles stale, while the alternative is a floor that silently fails
closed on the users it was written for. A recipient MAY additionally *prefer* beacon-scoped proofs
(e.g. grant them a larger budget) and MAY require the beacon from senders that demonstrably
resolved the recipient's `Identity`; it MUST NOT make the beacon the only acceptable scope.

**CAUTION.** PoW is a **last-resort** tier, not primary: there is no live standard, and plain
hashcash asymmetry favors organized spammers (botnets/GPUs) over low-power legitimate senders
(phones). DMTAP PoW MUST use a **memory-hard** function (Argon2/scrypt-style) to flatten that
asymmetry, and difficulty SHOULD be adaptive. Prefer the token/reputation path (§9.3) whenever
available.

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

**Bounding verification cost — the verifier's own memory-hard budget (normative).** A memory-hard
Argon2id verification is **symmetric**: checking a solution costs the recipient roughly what
producing it cost the sender. Because the cold-sender gate runs this check **before** any per-source
cap can apply (§2.7 step 6 precedes identity), a flood of **bogus** PoW attachments is itself a
memory/CPU **DoS on the recipient** — the attacker forces expensive verifications for free. A
recipient therefore MUST **bound the number of memory-hard PoW verifications it performs per time
window, per delivering connection/relay** (§16.5). Beyond that budget the recipient MUST **defer the
MOTE to the requests area WITHOUT verifying** its PoW (`DEFER_REQUESTS`, §2.7a) — never spend
unbounded memory-hard work on unauthenticated input, and never fail *open* by accepting it. This is
one more reason PoW is the **rate-capped last resort**: the ARC token / postage paths (§9.3, §9.5)
verify **cheaply** (a signature check), so they impose no symmetric-cost DoS surface and are always
preferred over PoW.

## 9.5 Postage (economic tier + gateway funding)

**Postage** is a signed, prepaid, real-money credit voucher (NOT a cryptocurrency):

- A sender MAY attach postage to a cold MOTE; heavy senders pay, contacts are free.
- **Postage doubles as the gateway fee**: when a MOTE is bridged to legacy (§7), the delivering
  gateway **redeems the stamp**, so outbound legacy sending is paid-for and revenue-neutral.
- Postage is enforced by the recipient/gateway; it is orthogonal to token/PoW anti-spam and
  MAY be combined.

### 9.5.1 Issuance, redemption & double-spend (normative)

Postage is a **bearer instrument for real money**, so it needs a settlement model, not just a
signature. DMTAP specifies the minimum:

- **Issuance:** a postage **issuer** (a provider/gateway with a real-money float) mints a stamp
  = `sign_issuer(serial, amount, expiry, [audience])`. The issuer debits the buyer's account at
  mint time. `audience` MAY scope a stamp to a specific recipient/gateway.
- **Redemption + double-spend prevention:** a stamp is single-use. The **redeeming party
  (recipient node or gateway) MUST check the stamp's `serial` against the issuer's redemption
  endpoint** (online check) or accept the issuer's signed short-lived spent-list; the issuer
  marks the serial spent atomically. A stamp presented twice is rejected on the second redeem.
- **Cross-operator settlement:** the redeemer claims the amount from the issuer against the
  serial; issuers reconcile out-of-band (standard interchange). Settlement requires an
  issuer-side serial ledger; this section specifies it.
- **Failure mode:** if the issuer is unreachable at redemption, the stamp is treated as
  *unverified* (falls back to token/PoW policy), never accepted on faith. No offline bearer
  acceptance of real money.

Postage issuers are subject to the same reputation/trust scoring as token issuers (§9.3.1);
an untrusted issuer's stamps carry no weight.

## 9.6 Gateway-operator accountability

For decentralized legacy gateways (§7):

- **Per-identity accountability**: every outbound handoff carries the sender's signature +
  postage, so a gateway attributes abuse to a *token/identity*, not an IP — making a shared
  reputation pool safe to decentralize.
- **Structural independence**: operator attestation plus **ASN/jurisdiction diversity**
  (§4.4.8) — the Sybil-resistance mechanisms that need no adjudicator. DMTAP deliberately does
  **not** specify an operator stake/bond or a slashing scheme: enforcing one requires an escrow
  and an adjudicator empowered to seize funds, which is a central authority more powerful than
  anything else in this specification (§4.4.8, normative note). Deployments may run bonds as
  operator policy; the protocol claims no such deterrent.
- **Reputation routing — locally measured, never globally published**: each node routes outbound
  legacy mail by **its own** measured deliverability-to-destination, and bad operators lose that
  node's traffic. This is the enforcement mechanism — automatic, adjudicator-free, and applied by
  each node independently. A **network-wide published score is forbidden** (§7.5): computing one
  requires a party that aggregates and ranks, which is the directory-authority problem §4.4.2
  deleted from the mixnet, re-created for gateways — a single point of censorship over who may
  carry legacy mail. Local measurement is also strictly more accurate, since deliverability is
  per-sender and per-destination rather than a global scalar.

## 9.7 Vouch / introduction (first-class)

A cold sender MAY present a **vouch** — an introduction token signed by a contact the recipient
trusts (a mutual connection) — to bypass VDF/PoW/stamp. This bootstraps first contact through the
social graph without a central authority, and MUST itself be rate-limited to prevent vouch
farming.

**Vouch is a primary tier, not a curiosity (normative) — but not a free one.** Of every anti-abuse
mechanism in this section, the vouch is the **only** one an adversary cannot buy with either
compute or money: it requires that someone the recipient already trusts is willing to spend their
own reputation on the sender. That makes it the most robust cold-contact path against a
well-resourced adversary, and it is also the mechanism with the best user experience — an
introduction is how humans actually make first contact. Usually security and usability pull the
same way here, which is rare enough to exploit; **the vouch is the one place in this section they
do not**, because it is bought with a privacy cost the other three mechanisms do not charge (see
below). Implementations MUST offer it wherever a mutual contact exists **and** MUST disclose that
cost at the point of offer — it MUST NOT be presented, defaulted to, or auto-selected as though it
were privacy-equivalent to an ARC token, a PoW solution, or postage.

**Honest limit — a vouch is a disclosed exception to sealed sender (normative).** A `Vouch`
(§18.3.3) names the **voucher**, the **subject** (the cold sender), and the **recipient** as
identity keys, and §9.2a requires the whole `ChallengeResponse` — vouch included — to ride the
**cleartext** envelope so the recipient's gate can check it before decryption (§2.7 step 6). There
is no sealed variant of a vouch: by the shape of the mechanism it *is* three identity keys plus a
signed, transferable social-graph edge, readable by every exit mix and any on-path observer of
that hop — precisely on the first-contact message whose sender most needs protecting. This is a
**different** exposure from the theft/replay issue §9.2a and §2.7 step 8(b2) close: that fix stops
a stolen vouch from being replayed by someone else; it does nothing about what the *legitimate*
vouch itself discloses to an observer when it is used exactly as designed. It directly narrows
SP-11's claim and §9.1 principle 4's guarantee (see there for the precise scope), and it means
`vouch` is the wrong choice of proof whenever the sender's identity, the voucher's identity, or the
bare fact that the two are mutually connected to the recipient is itself the sensitive fact —
the vouch cannot be used to reach such a recipient privately; only PoW, ARC, or postage can.

Closing this exposure requires a structural change this specification does not make here: either
**(a)** move the vouch inside `ciphertext` and restructure §2.7's cheapest-and-anonymous-first
validation order into a bounded decrypt-then-gate path for the vouch case specifically, or
**(b)** replace the cleartext vouch with a blinded, ARC-style presentation proving "someone the
recipient trusts vouched for the holder of `sender_key`" without revealing the voucher's or the
subject's identity to anyone but the recipient. Both are redesigns of §2.7 and/or §18.3.3, not
editorial fixes — this specification discloses the exposure they would close rather than choosing
between them.

## 9.7a The zero-relationship delivery floor (normative)

§3.13 promises that a user with no domain, no name-chain, and no provider is a **full
first-class DMTAP identity**. That promise is only true if such a user can actually *deliver*.
Without the rule in this section it is not: an identity with no issuer relationship gets a
default ARC budget of **zero** (§9.3.1), postage requires an issuer with a real-money float
(§9.5.1), a vouch requires a pre-existing mutual contact, and PoW is explicitly rate-capped and
last-resort (§9.4). Compose those and a sovereign key-name identity is nameable, reachable, and
verifiable — and **silently undeliverable**. That would reproduce, one layer up, exactly the
IP-reputation gatekeeping that made self-hosted SMTP unusable, while the specification claimed
the opposite.

Therefore: **a conformant recipient MUST NOT operate a policy under which a valid work proof is
never sufficient** — it MUST admit, up to the aggregate resource budget of §16.5, a sender
presenting **only** a
valid work proof — **either** a sequential-work proof (`vdf`, §9.4.1) **or** a memory-hard PoW
solution (§9.4), the recipient's choice of preference but **both** acceptable — bound to the
recipient's current epoch beacon, with **no** token, **no** postage, and **no** vouch. A recipient
that would accept only a VDF has set an unstandardized, non-post-quantum construction as the price
of contacting it (§9.4.1), which is the floor failing in a new way rather than holding. Floor deliveries land in the requests area (§2.7a), not
the inbox — the recipient's surfacing policy is unchanged and the user still sees nothing they
did not ask for. What the floor guarantees is narrower and non-negotiable: that a stranger with
nothing but a keypair and a few seconds of work — memory-hard by default, sequential if the
recipient also offers it — **can always reach the requests area**,
and can therefore be found, replied to, and promoted to a contact by a human decision.

**Why the floor is a policy constraint and NOT a per-sender count (normative).** A per-sender-key
quota — "at least `N_floor` cold MOTEs **per sender-key** per day" — bounds nothing. §2.2 defines
`sender_key` as an **ephemeral, fresh-per-message** key whose entire purpose is unlinkability, and
§9.2a notes an attacker mints them at will — so a per-sender-key quota is a quota *per message*,
which is no quota at all. Nor could it be otherwise: the recipient
has **no stable subject to meter against at gate time by design**, because identity is not
revealed until §2.7 step 8, after decryption. §9.2's `RateLimit` and §16.4's per-sender spool cap
inherit exactly the same vacuity on the cold path and MUST be read as aggregate limits there.

Worse, the old wording composed with §9.4's overflow rule into an unbounded **durable-storage**
denial of service. Beyond the memory-hard verification budget §9.4 requires deferral **without
verifying**; the deferral target is the requests area; and §2.7a forbade dropping from it while
mandating 30-day retention. An attacker could saturate the per-connection budget (cheap, and
multipliable across delivering paths), then send unlimited cold MOTEs carrying **garbage** in
`challenge` — never doing any work at all — and a conformant node was obliged to store every one
of them, durably, for 30 days. The §9.11 claim that native mail needs no filter rested on a
requests area that was in fact an unauthenticated write channel.

The floor is therefore a **guarantee about policy** — that no configuration may make honest
zero-relationship contact impossible — not a licence for unbounded intake. Bounding the
**aggregate** (§16.5) is what makes the guarantee affordable to hold open, and §2.7a now
distinguishes *refusing unverified input* from *silently dropping a well-formed, verified MOTE*.

- The floor is a **minimum**, not a ceiling: a recipient MAY grant far more to trusted issuers,
  vouched senders, or paid postage. It MUST NOT grant less.
- A recipient under active flood MAY apply the §9.4 deferral budget to floor traffic as it does to
  any other, but MUST NOT make a valid work proof insufficient as a standing policy
  (`ERR_POLICY_BELOW_FLOOR`, `0x070F`, §21.10 — the policy is refused, not silently clamped).
- Implementations MUST NOT ship a default policy that violates the floor, and a policy UI MUST
  NOT offer "reject all unknown senders" as a reachable configuration without disclosing that it
  makes the identity uncontactable by anyone not already known.

**Why this is normative rather than advisory.** Every individual recipient has a local incentive
to set their own floor to zero — it is strictly safer for them. The aggregate of everyone doing so
is a network where only the already-connected can participate, which is the precise failure the
naming ladder (§3.13) and the key-name floor (§3.9.6) exist to prevent. This is a
collective-action problem, and the only place to solve it is in the conformance requirements.

## 9.8 Mixnet abuse

Mix nodes are content-blind and cannot filter content — and, crucially, an entry mix **cannot**
verify the recipient-facing anti-abuse proof (ARC token / postage / PoW) either. The proof rides
the **cleartext envelope**, not the payload (§2.2b) — but Sphinx onion-layering (§4.4) hides that
envelope from every mix except the one peeling the final layer: each mix peels exactly one layer
and forwards on its own hop's routing info alone, never seeing the recipient-scoped envelope
underneath. Only the **exit mix** peels that last layer and sees `challenge`; an entry mix has
nothing to check because the proof has not yet been revealed to it, not because it is sealed
inside encrypted payload. Mixnet flood-abuse is therefore bounded by mechanisms a
**content-blind** node can actually apply:

- **Per-connection / per-guard-operator rate limiting (MUST).** An entry mix admits Sphinx cells
  under a **per-connection and per-operator rate budget** — it limits how fast any one source (and
  any one upstream operator) may inject cells — **not** by inspecting a sealed recipient proof it
  cannot read. This is the honest entry-admission control: it caps injection rate blind to content.
- **Optional mix-visible anti-flood PoW (MAY).** A mix MAY additionally require a **small
  proof-of-work bound to the mix hop itself** (a puzzle over the cell + epoch the mix *can* check
  without decrypting) as an anti-flood cost knob under load — distinct from the recipient's §9.4
  cold-sender PoW, which the mix cannot see.
- **Operator rate limits and attested diversity (MUST).** Operator-level rate limits, plus the
  attested-operator and ASN-diversity requirements of §4.4.8, make sustained flooding costly and
  attributable without requiring any adjudicator to seize a bond (§9.6).

The recipient's token/postage/PoW proof (§9.3–§9.5) still gates whether the message is *accepted
into the recipient's inbox* — but it is enforced at the **recipient**, after mix delivery, never
mistaken for something an entry mix verifies. Cover traffic is itself **rate-bounded per node**
(§4.4.7) so cover cannot be turned into the flood.

## 9.9 Group-address amplification

A **group address** (§5.8) is a first-class abuse-amplification vector: a single post fans out to
N recipients, and a naïve design evaluates the anti-abuse challenge **once at list ingress** and
then attributes delivery to the **group**, bypassing each recipient's per-sender policy (§9.2) and
**laundering the poster's accountability**. DMTAP forbids this:

- **Origin accountability carries through fan-out — with the right proof per list type.** The
  **poster's** challenge proof MUST be carried on each fanned-out per-member delivery; each
  recipient applies its per-sender policy to the **original poster**, not the group identity.
  *Which* proof depends on the list's membership model, because ARC tokens are per-origin
  (per-recipient) scoped and cross-recipient-unlinkable (§9.3) and therefore **do not compose**
  with fan-out to many recipients:
  - **Member-visible channels** (§5.8.3): the poster knows the members, so it MAY mint a
    per-member ARC token (one per recipient origin). ARC carry-through applies here.
  - **Hidden-membership lists** (§5.8.3): the poster does *not* know the members (that's the
    point), so per-member ARC is impossible without breaking membership privacy. These lists
    MUST use **postage or PoW scoped to the list address** as the poster's proof (a single
    list-scoped proof the committer/relay verifies at ingress and vouches per-delivery), not
    per-member ARC. The committer attests "this fan-out carried a valid list-scoped proof from
    poster P" so recipients still get origin accountability without learning a cross-recipient
    ARC graph. **Honest limit:** this hands the hidden-list committer a trust power recipients
    must accept — it could false-vouch (launder a spammer), misattribute (frame a poster or evade
    a recipient's block on P), or under-size the "commensurate" proof. It is bounded (the hidden
    list's committer *is* the list operator, which already re-seals the fan-out and knows the full
    membership — roughly the mailing-list trust you already accept), and abuse is detectable
    (recipients down-score a committer/list that vouches for spam, §7.5). Member-visible channels
    avoid this power entirely (each recipient verifies ARC independently), so hidden-membership
    trades a committer-trust assumption for its privacy — disclose it, don't hide it.
- **Per-poster fan-out rate-limits.** A list MUST rate-limit fan-out **per poster** and **cap
  amplification** for `open`-join lists (anyone-can-post), whose amplification is otherwise
  unbounded.
- **Cost to post to large lists.** Posting to a large list MUST require **postage or PoW**
  commensurate with the fan-out size, so mass amplification is not free.
- **Legacy fan-out** (§5.8.5) is bound by the same rules; the gateway attributes the origin
  (§9.6).

## 9.10 Gateway bidirectional anti-abuse (the two-way choke point)

§9.1–§9.9 protect **native** recipients. The **legacy gateway** (§7) is the one component that also
faces the open legacy world, so it MUST apply anti-abuse in **both** directions, fail-closed —
specified normatively in §7.11 and restated here because it belongs to the anti-abuse model:

- **Inbound (legacy → mesh).** The gateway MUST authenticate the legacy sender (SPF/DKIM/DMARC,
  §7.11.1) and MUST carry legacy senders through the **cold-sender gate** (§9.2): a legacy stranger is
  a **cold contact**, subject to the recipient's challenge policy, never injected with contact
  standing. This stops a gateway from laundering legacy spam into the accountable mesh.
- **Outbound (mesh → legacy).** The gateway MUST relay **only for authenticated senders** — an
  authorized `GatewayAuthz`/key-registered relationship (§7.12) **or** valid redeemable postage
  (§9.5) — and MUST apply per-sender **rate limits and volume caps** (§9.6). An unauthenticated
  outbound relay attempt is refused fail-closed with `ERR_GATEWAY_SENDER_UNAUTHENTICATED` (`0x0607`,
  §21.8): a valid `sender_sig` proves *who signed*, not *who may relay*, so signature-validity alone
  MUST NOT authorize egress. This is the open-relay floor (§7.7) that per-identity accountability
  (§9.6) makes affordable to hold open.

As everywhere in §9, only the **floor** — that both directions are gated, fail-closed, on these
signals — is in-spec; the **thresholds, caps, and pricing** are operator policy and out of scope
(§7.13).

## 9.11 Authorization is at the boundary; classification is at the recipient (normative)

Two questions get confused in mail, and keeping them apart is the difference between a bridge and
an institution:

| Question | Who answers it | On what evidence |
|----------|----------------|------------------|
| **"May this sender inject/relay this?"** — *authorization* | the gateway, at the legacy boundary (§7.11) | SPF/DKIM/DMARC results, IP standing, authenticated sender identity, cold-sender state, rate counters |
| **"Is this message wanted?"** — *classification* | the **recipient**, on the recipient's own device | the recipient's own corpus, contacts, and history |

**The rule.** A gateway **MUST NOT classify content** (§7.11.4): no content-based scoring, no
Bayesian or learned filters, no keyword/URL reputation, no content heuristics, and no dropping or
re-ranking on such a basis. Classification, where it happens at all, is **recipient-side and
on-device**, against a corpus that never leaves the user's own machines (§0.7, §6.7). A recipient
MUST be able to run *no* classifier and still be protected, because the §9.1–§9.7a mechanisms are
sender-cost and policy mechanisms, not content judgements.

**Native DMTAP mail needs no filter at all.** This is the part that makes the rule affordable
rather than merely principled. Every native MOTE is **authenticated to the recipient** before it is
surfaced (§9.1 principle 1, §2.7): there is no anonymous injection path, cold senders are gated by
the recipient's own policy into the requests area (§2.7a, §9.2), and per-identity blocking is
exact. The thing legacy filters exist to solve — unlimited unattributable injection by strangers —
**does not exist on the native path**. Filtering is a legacy-shaped answer to a legacy-shaped
problem, and it should leave when legacy does (§7.1c).

**Why this is structural: measured evidence that anti-abuse is how mail re-centralizes.** Mail
centralization is usually told as a story about mailbox providers. Measured evidence says there is
a **second, independently growing centralized tier** made of anti-abuse — why this prohibition is
structural, not a preference — reported in `research/` (IMC 2021 mail-provider study; BitTorrent
tracker persistence).

Both point at the same conclusion: **the classifier is the durable centralizer**, because
classification improves with corpus size, never terminates, and makes everyone's mail depend on a
judgement only the aggregator can make. A DMTAP gateway that classified content would therefore be
**permanent by construction** — the one thing §7.1c is designed to prevent it from becoming — and
would rebuild, at the bridge, precisely the tier these measurements found had already formed
around SMTP. So the prohibition is not a preference about spam handling; it is what stops the
legacy adapter from turning into the next incumbent.
