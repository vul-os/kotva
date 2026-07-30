# Designing the primitives — ATTEST · REPUTATION · ESCROW · RESERVE · MATCH · OFFER

> **Status: design brief (non-normative).** This document derives *how* each of the DIRECTION §2
> waist primitives should be built — which existing standard it binds to, its minimal wire shape
> over the substrate, the normative rules a future binding/profile MUST carry, and its honest
> ceiling. It is design reasoning in the [`docs/research/`](README.md) sense: it settles the
> *direction* per primitive so a binding ([`bindings/README.md`](../../bindings/README.md)) or a
> profile (TRACT, WRAP) can ratify the bytes. Where it uses BCP-14 keywords it states the rule the
> ratifying binding MUST adopt, not a byte this document freezes. No new object is invented here;
> every wire sketch is an existing substrate object (MOTE §2.3, Feeds & Blobs / DMTAP-PUB §22,
> SYNC) or an already-frozen profile object, cited so an implementer can find the authority.

The one rule governs all six ([DIRECTION §0](../../DIRECTION.md)): **the primitive works offline
against local trust; a coordinator only adds reach and never gates function.** Each primitive below
is stated at both anchors of the scale-invariance slider ([DIRECTION §6](../../DIRECTION.md)).

---

## 0. The map at a glance

| Primitive | Bind to (DIRECTION §3) | Substrate carrier | Coordinator kind (if any) | Content-visibility |
|---|---|---|---|---|
| **OFFER** | content-addressed signed listing (no registry) | Feeds & Blobs / DMTAP-PUB (§22) | `indexer` (discovery only) | listing public by design; indexer `blind`/attested |
| **MATCH** | order-book / auction assignment (no standard to bind — it is a coordinator) | two OFFERs in, signed Assignment out | `matcher` | sees offers → `terminating`; TEE-`attested` preferred |
| **RESERVE** | single-owner bounded-counter CRDT | SYNC single-writer object (§5.6, TRACT §6) | **none** (self-serving by construction) | owner-held; no intermediary |
| **REPUTATION** | OpenRank (EigenTrust, TEE) over the attestation graph — **no global score** | Review / attestation feed objects (§22) | `indexer` (derived, rebuildable) | public objects; index `blind`/attested |
| **ATTEST** | EAS + W3C Verifiable Credentials for credential claims, else a profile-defined map (e.g. a reputation polarity, a measurement); personhood → World ID / Human Passport | signed feed object (§22) or sealed `identity` MOTE (kind 0x09) | `oracle` (only for physical-fact attestation) | issuer-signed, public or sealed |
| **ESCROW** | HTLC / smart-contract escrow on a stablecoin rail; dispute → Kleros-class | EscrowScope (public) + PaymentAttestation (sealed) + escrow state machine | `arbiter` / escrow operator | evidence `terminating`, disclosed |

Two invariants cut across the whole table: **the protocol carries attestations, never funds**
([TRACT §9.2](../../profiles/tract/09-settlement.md)), and **no object carries a network-wide
published score or a protocol token** ([DIRECTION §5](../../DIRECTION.md),
[CONTRACT §2.1](../../coordinator/CONTRACT.md)).

---

## 1. OFFER — a signed listing, not a registry

- **Bind to:** the substrate's own **content-addressed signed public object** (Feeds & Blobs /
  DMTAP-PUB, §22). There is no external "listing standard" worth importing; the correct move is
  *don't reinvent a registry* — an OFFER is a `PubAnnounce`/`FeedEntry` on the author's feed. TRACT
  (`Offer`, §16.5.2) and WRAP (`Offer`) are the two worked profiles.
- **Wire shape:** an OFFER rides the author feed as a signed public object addressed by
  `(author IK, content-address)`. TRACT structures its body along four axes (catalogue /
  availability / consideration / fulfilment); WRAP carries `mode` + `pool` + terms. Sealed variants
  (a private quote to one buyer) ride a normal MOTE (`mail`/`chat`, §2.3) instead of a public feed.
- **Normative rules a binding MUST carry:**
  - The subject/identifier MUST be **operator-independent** — the author `IK` and the offer's
    content-address, identical at every index and gateway (so reputation and history follow the
    key, not a storefront; TRACT §10.2b).
  - **Withdrawal is supersession, never deletion** — a later signed feed entry supersedes; a
    byte-holder that kept the old bytes is not violating anything (§22.6, TRACT §10.2d).
  - An OFFER's availability is a **signal, not a hold** — reading it as a reservation is a category
    error; the hold lives in RESERVE (TRACT §3.9).
  - **No canonical catalogue.** Indexes over public offers are *derived, rebuildable, and never
    authoritative* (§22.4.3); different indexes reaching different sets is the design.
- **Small scale / offline:** an offer sent point-to-point (`mode = direct`, WRAP §8.2) needs no
  infrastructure at all and covers most real relationships. **Global:** an `indexer` coordinator
  makes offers discoverable.
- **Honest ceiling:** *there is no open discovery without infrastructure* (WRAP §8.1) — global reach
  requires an indexer, which can re-rank/omit (fenced by swappability + "authorise, never classify",
  CONTRACT §4). Freshness is bounded only by feed-poll cadence (TRACT §3.8). Listing spam/Sybil is
  unsolved at the protocol layer and is index-local policy.

## 2. MATCH — the one engine, differing only in the assignment rule

- **Bind to:** nothing standardised cleanly maps (order books and auction engines are venue-internal),
  so MATCH is a **`matcher` coordinator** under the contract, not a new primitive. Its only variation
  across services is the **assignment rule** — nearest (rides), highest-bid (auctions), best-fit
  (freelance) — which is *matcher policy*, not protocol (DIRECTION §2, §6). Prefer a **TEE-`attested`**
  matcher so a global view is possible without an operator reading plaintext (bindings: TEEs).
- **Wire shape:** two OFFERs in — a demand OFFER and supply OFFERs (or bids: WRAP `Bid`) — and one
  **signed Assignment out** (WRAP `Assignment`), authored by the matcher and delivered to the
  matched parties as a MOTE. The matcher holds only rebuildable operational state.
- **Normative rules:**
  - The matcher **authorises, never classifies** (CONTRACT §4): it checks *who and rate*, never
    *whether content is wanted*.
  - **Swappable + self-hostable** — the local fallback is a *dumb local order book* the parties run
    themselves; the assignment rule and the party set MUST survive dropping the matcher (CONTRACT
    §2.2–2.3, DIRECTION §6).
  - The matcher **cannot forge a match**: every OFFER/Bid/Assignment is author-signed; it distributes
    and proposes, it does not produce signatures it lacks (WRAP §8.3).
- **Content-visibility:** a matcher **reads the offers by definition** — this is the one role that is
  *not* content-blind unless run in a TEE (WRAP §8.5). It MUST declare `terminating` (or `attested`
  if TEE), and clients MUST surface that (CONTRACT §3). Mitigations: coarse offers (area/window/range)
  with exact detail only in the sealed Assignment.
- **Honest ceiling:** global-view optimisation is exactly where centralisation regrows; the contract
  confines but does not remove it. Blind matching needs a TEE (chip-vendor trust, side-channel
  history — `attested`, never trustless, bindings note). Offline, MATCH degrades to a local order
  book with no global optimality.

## 3. RESERVE — single owner, so double-booking is structurally impossible between honest participants

- **Bind to:** a **single-writer bounded-counter CRDT** — the host's box is the *only* writer for its
  own calendar/inventory, so concurrent overselling is impossible by construction **between honest participants** — a dishonest owner can still oversell, leaving signed attributable evidence rather than being prevented (RESERVE §9) (DIRECTION §2). This
  is the substrate's §5.6 device-cluster CRDT / SYNC single-owner profile, realized as TRACT's
  bounded-counter inventory (§6) grounded in AntidoteDB `antidote_crdt_counter_b` (SRDS 2015). **No
  coordinator, ever** — RESERVE is the primitive that needs none.
- **Wire shape:** a SYNC object in the owner's namespace whose value is a bounded counter, mutated by
  owner-signed ops; or TRACT §6's counter beneath a published `Availability` band. A booking is the
  owner's authoritative `placed → accepted / declined` step (TRACT §3.9, §18.3), not a client-side read.
- **Normative rules:**
  - **Exactly one writer per resource.** A reservation against a single-owner calendar MUST resolve
    on the owner's node; a buyer's local availability read MUST NOT be represented as a hold (TRACT
    §3.7, §3.9).
  - The guarantee is **safety-only, paid entirely in liveness** — stranded quota and spurious
    "sold out" are the cost of never overselling from stale state (TRACT §3.9 honest limit).
- **Honest ceiling:** the no-oversell invariant is **non-Byzantine** — it protects a seller from
  their *own* concurrency, not a buyer from a *dishonest* seller (TRACT §21.10.2). RESERVE guarantees
  no oversell, not honest supply; the availability *signal* upstream of it guarantees even less.

## 4. REPUTATION — locally measured or OpenRank-computed, never a published number

- **Bind to:** **OpenRank** (EigenTrust, TEE-verified) as *compute* over the public attestation/review
  graph — and, at small scale, direct web-of-trust (DIRECTION §6). Bind the *algorithm as a service*,
  never a score we mint (bindings: Reputation).
- **Wire shape:** the graph is ordinary public feed data — TRACT `Review` objects (§10.2), ATTEST
  claims (§5 below), WRAP work attestations — all addressed by the **stable subject identifier**
  (product content-address or subject `IK`). Reputation is *derived* by an `indexer` (or computed
  locally) from these feeds; it is never an object on the wire.
- **Normative rules:**
  - **No network-wide published score, and no object carries one** (TRACT §10.3); a coordinator's
    descriptor explicitly carries no global reputation number (CONTRACT §2.1). Reputation is
    *locally measured by each client from its own results* (CONTRACT §2.1) or bound to OpenRank —
    never one authority's number.
  - Reputation is **portable and un-suppressible**: it attaches to the key, any node MAY gather and
    index the feeds without permission, and a subject cannot delete an unfavourable review by leaving
    a platform (there is none) (TRACT §10.2b).
  - Review authorship uses **per-subject pseudonymous subkeys** (not the root `IK`), so subjects
    cannot join a buyer's reviews across them (TRACT §10.2); retraction is supersede, cooperative-only
    (§10.2d).
  - Different indexes reaching different rankings is **the design, not a defect** (TRACT §10.3).
- **Honest ceiling:** the achievable **Sybil-cost floor on a signed-feed substrate is an open
  question** (TRACT §21.8). Attestation-gating (§5) raises manipulation cost but **self-dealing
  produces genuine attestations** and **whitewashing is bounded only by discounting new keys** — a
  purely index-local defence, since no authority may ban (TRACT §10.3a, §10.4). Global reputation
  quality is capped by the personhood anchor, which is imperfect (bindings: Proof-of-personhood).

## 5. ATTEST — a signed claim; the issuer's standing is the whole trust

- **Bind to:** **EAS** (Ethereum Attestation Service) and **W3C Verifiable Credentials** for the
  claim shape; **World ID / Human Passport** for the specific personhood claim that anchors anti-Sybil
  (DIRECTION §3, bindings). Do not invent a credential format.
- **Wire shape:** an attestation is a signed object carrying `{issuer IK, subject, schema/type, claim,
  ts, ?revocation-ref}` — either a **public** feed object (§22) when the claim is meant to be openly
  verifiable, or a **sealed `identity` MOTE** (kind `0x09`, §2.3) when it is a private identity claim.
  Profile-specific attestations already exist: TRACT `PurchaseAttestation` (§10.2a) follows this
  shape. **WRAP's `Attestation` deliberately does not** — it carries the same semantics in WRAP's own
  uniform `COSE_Sign1` object (its own `author`/`id`), not as an unsigned map inside a carrier; see
  [`primitives/ATTEST.md` §5](../../primitives/ATTEST.md). Both address their subject by
  content-address or `IK` only.
- **Normative rules:**
  - An attestation is **verifiable offline by the issuer's signature** — it proves *`IK` said this*,
    and *nothing more*. A client MUST NOT present an attestation as an established fact beyond the
    issuer's standing (cf. TRACT §10.2a "what attestation does not establish").
  - **Revocation is supersession / status-list**, honoured cooperatively (as with reviews, TRACT
    §10.2d); an attestation MUST reference the subject by a stable operator-independent id.
  - For a **physical-fact** attestation (delivered? ride done?) the issuer is an **`oracle`
    coordinator** — accountable, swappable, `terminating`, disclosed (CONTRACT §5).
- **Honest ceiling:** the **physical-event oracle** ceiling (DIRECTION §8.2) — an oracle can attest
  *origin-through-itself* but can never prove *non-fabrication*; "did it happen?" reduces to
  dual-confirm + dispute. Issuer trust is exogenous (an attestation is only as good as the issuer),
  and **personhood is imperfect** — every method trades off (biometric+operator, or passport-zk
  excludes the undocumented). Local scale substitutes web-of-trust.

## 6. ESCROW — attestations on the wire, funds on the rail, disputes to a staked jury

- **Bind to:** **HTLC / smart-contract escrow** on an existing **stablecoin** settlement rail for the
  non-custodial case; a **licensed escrow operator** (TRACT §9 operator class) for the custodial case;
  **Kleros-class staked juries** for dispute (bindings: Payments, Dispute). Stake and settlement are
  in *existing* assets — **no protocol token** (DIRECTION §5).
- **Wire shape:** three parts — `EscrowScope` (public, the operator's declared lawful cover, TRACT
  §16.5.4), `PaymentAttestation` (sealed, *attests* a settlement, never moves it, §16.6), and the
  escrow **state machine** `fund → hold → release / refund / split` (TRACT §18.5). The signed
  per-transition object and the signed ruling are a **known §16 gap** (TRACT §9.4.3, marked
  PROVISIONAL) — a binding that needs them adds an escrow-transition object carrying the operator
  signature, from/to state, order address, and evidence ref.
- **Normative rules:**
  - **The protocol carries attestations, never funds** (TRACT §9.2, §9.4.1) — no PAN, no account
    data, no credential; KOTVA custodies and converts nothing and takes no cut (CONTRACT §6).
  - **`RailClass` is part of the type** (`CustodialReversible` vs `NonCustodialFinal`) because it
    changes the buyer's recourse; substituting one for the other without fresh explicit agreement MUST
    be rejected (TRACT §9.3).
  - **Fail-closed scope intersection** at checkout: a missing/unparseable/incomparable `EscrowScope`
    field means *not covered*, not covered; an empty intersection MUST surface explicitly and MUST NOT
    silently downgrade to an unescrowed trade (TRACT §9.4.2).
  - The escrow operator is **permissionless, per-order, competing, replaceable, and never holds
    identity keys** (TRACT §9.5); an unescrowed trade MUST be a disclosed outcome, never a silent
    default (§9.5a). Where a coordinator stakes (arbiter/oracle), the stake is an existing asset sized
    to value-at-risk (DIRECTION §5).
- **Honest ceiling:** **non-custodial programmatic escrow deadlocks on genuine disputes** — multisig /
  hashlock+timelock removes the custodian but has *no move* when neither party acts; the only honest
  options are a timeout defaulting to one party (a policy choice, not neutral) or an indefinite lock
  (TRACT §9.6). **Opt-in escrow is declined by exactly the actors it targets** (OpenBazaar's measured
  outcome, TRACT §9.5a). The **escrow operator is structurally permanent** — holding money for
  strangers is licensed and does not decay (TRACT §9.6, §0.4.3). **Physical custody cannot be made
  trustless** (escrow moves money, not goods).

---

## 7. PAY — recurring instalments as a rail-agnostic pattern, not a seventh primitive

> **Backlog C5, answered 2026-07-30.** `PAY` is named in [`SPEC.md § ③`](../../SPEC.md) as one of
> the seven waist terms (`OFFER · MATCH/RESERVE · REPUTATION · ESCROW · ORACLE · DISPUTE · PAY`) but
> — unlike OFFER/MATCH/RESERVE/REPUTATION/ATTEST/ESCROW above — it is explicitly **not** an owned
> primitive doc: `SPEC.md` states plainly that `PAY = the x402/stablecoin binding`, i.e. its whole
> content already lives in [`bindings/README.md`](../../bindings/README.md). This section is written
> at that same level — a binding-adjacent design brief — **not** as a new `primitives/PAY.md`, and
> **not** inside a profile. The reasoning for landing one level above "profile" rather than inside
> one, since the backlog item's own title proposed "profile-level":
>
> - A profile is where **wire bytes get frozen for one domain** (TRACT's `Offer`/`Review`, WRAP's
>   `Assignment`). Recurring settlement is not domain bytes — it is a *pattern of PAY usage* that
>   TRACT, WRAP, media, and any future profile all want unchanged, exactly the way ESCROW's shape (§6
>   above) is written once, above every profile, and only its concrete `EscrowTransition`/
>   `EscrowRuling` bytes are left PROVISIONAL for whichever profile freezes them first.
> - PAY has no owned primitive doc to attach this to (unlike ESCROW/RESERVE), so the semantics belong
>   at PAY's own existing home — the binding index — generalised as a design brief the way this whole
>   document generalised ATTEST/ESCROW/RESERVE **before** any profile had frozen their bytes.
> - Concretely: **actual wire bytes for a recurring schedule object remain unspecified here, on
>   purpose**, and **no wire id is allocated** — that is a profile-and-registry decision
>   (`003-kinds-registry.md`-style coordination is an owner call for wire ids in general; see
>   `substrate/ADOPTION.md` and `18-wire-format.md` for the existing allocation discipline). If a
>   profile later needs `RecurringPlan` on the wire, it freezes the bytes there, citing this section
>   for the rules those bytes must satisfy — same relationship ESCROW already has with TRACT.

### 7.1 What triggered this section

`patala` (a separate product, read for its **design**, never as a dependency — KOTVA does not import
from products) built and demonstrated, once, on Stellar **testnet**: N pre-signed, time-bounded
transactions on one dedicated source account, using Stellar's `PreconditionsV2`
(`min_seq_age`/`min_seq_ledger_gap`) to make early redemption **network-rejected**, cancellable by one
`BUMP_SEQUENCE` that permanently invalidates every remaining instalment
(`patala-stellar/src/recurring.rs`, `patala-stellar/README.md` §"The recurring primitive"). Real
Horizon rejected an early resubmission with `tx_bad_minseq_age_or_gap` and a post-cancellation
instalment with `tx_bad_seq` — the network enforced both properties, not just the client library.
**This is the first working reference for what "recurring" can mean without a smart contract or a
custodial standing mandate**, so it is worth generalising from — carefully, and only as far as the
evidence reaches (§7.4).

### 7.2 Stellar-specific vs genuinely general

The crux this section exists to get right. Reading `recurring.rs` line by line:

| In patala's implementation | Rail-specific (Stellar) | The general property it demonstrates |
|---|---|---|
| `PreconditionsV2.min_seq_age` / `min_seq_ledger_gap` | Yes — a field that exists **only** inside Stellar's `PreconditionsV2`; `Preconditions::None`/`Time` (an absolute calendar window) cannot express it | **Enforced spacing**: the payee cannot redeem instalment *i+1* before a stated interval has elapsed since instalment *i* actually settled, and this is checked by the network/rail itself, not by client-side trust |
| Sequence-number chaining (`min_seq_num: None` ⇒ "exactly `seqNum − 1`") | Yes — Stellar's account-sequence model specifically | **Independent per-instalment authorisation**: each instalment is a separately pre-signed transfer decided in full at authorisation time; no instalment grants the payee any right to another one, and none may be redeemed out of order or ahead of schedule |
| `BUMP_SEQUENCE` as the cancellation op | Yes — a Stellar-specific operation | **Unilateral, payer-side cancellation**: the payer can, in one action and without the payee's cooperation or signature, permanently void every not-yet-settled instalment |
| "Dedicated source account" (never used for anything else) | An implementation precaution specific to how Stellar sequence numbers are shared across all of an account's activity | Not general at all — it is a workaround for a Stellar-specific coupling (ordinary wallet activity would otherwise desynchronise the chain). A rail without a single shared per-account sequence counter has no analogous need |
| `min_seq_num` deliberately left `None`, not the looser shared-window variant | Stellar-specific spec knob | General **lesson**, not mechanism: a "relaxed ordering" mode that tolerates a skipped instalment is real and tempting, but here it also lets whoever holds the signed set jump straight to a later instalment and burn every one between — patala declined it for exactly this reason. Any binding offering "skip-tolerant" recurring MUST show it cannot be exploited the same way, not merely assume the temptation is rail-specific |
| Non-custodial, no contract, no standing mandate | **General.** This is the property the mechanism exists to prove, not an accident of Stellar | — |

**The general pattern, stated rail-agnostically:** a **recurring instalment plan** is *N
separately-authorised transfers of a fixed amount and asset from payer to payee, spaced by a stated
interval that the payee cannot shorten unilaterally, established once and cancellable in one payer-side
action, with no protocol-held or contract-held custody of undelivered funds at any point.* Stellar's
`PreconditionsV2` is **one way** to realise "enforced spacing" and "unilateral cancellation"; it is not
what those properties mean. A payment-channel/state-channel rail (the existing
[`bindings/README.md`](../../bindings/README.md) "Streaming / subscriptions" row — Lightning, a
Superfluid-class stream) realises the same two properties by a structurally different mechanism (a
unilateral-close timeout instead of a sequence precondition); a custodial fiat "subscription" API
realises neither cryptographically at all (§7.3).

### 7.3 Normative rules a binding/profile ratifying this pattern MUST carry

Following this document's own convention (§0): these are the rules a future binding entry or a
profile that freezes recurring's wire bytes MUST adopt, not bytes this document freezes.

- **MUST fix amount, asset, count (or an explicit renewal point), and spacing at authorisation time.**
  A recurring plan is not an open-ended standing mandate the payee can redefine; every instalment's
  terms are decided once, up front, by the payer (patala's `RecurringPlan` — count, amount, asset,
  spacing all fixed fields, refused rather than built if the count is unbounded/absurd).
- **MUST make every instalment independently authorised, never a shared-window grant.** A binding MUST
  NOT let holding one instalment's authorisation confer any right to a different instalment,
  out of order or ahead of its own spacing floor (§7.2's `min_seq_num` lesson — declined by patala, not
  yet proven safe by anyone).
- **MUST give the payer a unilateral cancellation action.** Cancellation MUST NOT require the payee's
  signature, cooperation, or acknowledgement, and MUST be effective in one step against every
  not-yet-settled instalment at once — never a per-instalment revocation list the payer has to work
  through under time pressure.
- **MUST NOT let the protocol or any coordinator custody funds in flight for a recurring schedule** —
  the same rule ESCROW states as N-1 (§3 above), restated for PAY: funds move directly between the
  payer's and payee's own accounts on the rail; a recurring schedule is never a reason to introduce a
  holding party.
- **MUST disclose, per instalment, whether spacing and cancellation are *rail-enforced* or
  *cooperative-only*** — never present the weaker as the stronger. Rail-enforced means the network
  itself rejects an early or post-cancellation redemption attempt (proven, for Stellar, by real
  Horizon `tx_bad_minseq_age_or_gap`/`tx_bad_seq` responses in §7.1 — not merely by client-side logic).
  Cooperative-only means the payee could technically attempt it and something else (contract law,
  chargeback, reputational cost) is the only deterrent.

### 7.4 A rail that cannot provide this MUST be refused, not silently degraded

Exactly the pattern patala already applies to atomic multi-party settlement
(`RailCapabilities::atomic_multi_party` / `require_atomic_multi_party` — `patala-core/src/capabilities.rs`,
returning `Error::Unsupported` rather than quietly executing N independent operations with no shared
guarantee): **a binding or profile that offers "recurring" as a feature MUST expose a capability a
caller can check, and MUST refuse a request for *rail-enforced* recurring on a rail that cannot
provide §7.3's spacing/cancellation guarantees — never silently downgrade to the cooperative-only
form while claiming the stronger one.**

The case this exists for is named explicitly, per [`DIRECTION § 5`](../../DIRECTION.md)'s own
acknowledgement that a **fiat rail is a legitimate operator choice**, not a lesser citizen: a typical
custodial-processor "subscription"/recurring-billing API (Stripe-class, `RailClass::CustodialReversible`
in patala's own vocabulary) is a **payee-initiated pull**, authorised once by the payer. Nothing on that
rail structurally stops the processor (or the payee behind it) from attempting an out-of-schedule
charge before a cancellation is processed — the payer's recourse is a chargeback or dispute *after the
fact*, not a network rejection *before* it happens. That rail can offer real, useful recurring billing;
it cannot honestly claim §7.3's rail-enforced spacing or unilateral, immediately-effective cancellation,
and a spec-conformant implementation MUST say so rather than present card-network "subscriptions" as
equivalent to a Stellar-precondition-enforced schedule or a payment-channel timeout. This is the same
honesty patala already enforces structurally for atomic splits (`atomic_multi_party: false` on every one
of its 20+ fiat rails, grepped and asserted in its own test suite) — recurring earns the identical
treatment here, in the spec, even though nothing in kotva itself implements a capability struct today.

### 7.5 Honest ceiling — what this section is evidence of, and what it is not

- **Demonstrated exactly once**, by a separate product, on Stellar **testnet**: one 3-instalment
  schedule, one dedicated throwaway account, an ~8-second spacing floor. That is real evidence the
  *pattern* (enforced spacing + unilateral cancellation without a contract or custody) is achievable —
  it is not evidence the pattern is proven **across rails**. Mainnet is untouched. No other chain rail,
  no payment-channel binding, and no fiat rail has been exercised against §7.3 at all.
  **kotva itself has implemented nothing here** — this section is a design brief, exactly like the six
  above were before any profile used them.
- **A second, structurally-different proof would strengthen this the way personhood required ≥2
  structurally-different anchors** ([`bindings/README.md`](../../bindings/README.md) "Personhood — v0
  requires ≥2 structurally-different bindings"). A payment-channel/state-channel demonstration of the
  same two properties (spacing + cancellation) by a genuinely different mechanism is the natural next
  data point — not required to write these rules down, but required before treating the *pattern* (as
  opposed to one rail's realisation of it) as validated.
- **No wire bytes, no wire id, no coordinator kind.** Whether a `RecurringPlan`-shaped object ever
  needs its own MOTE kind or rides inside an existing `PaymentAttestation`-style object is left to
  whichever profile freezes it — deliberately unresolved here, the same way ESCROW leaves
  `EscrowTransition`/`EscrowRuling` PROVISIONAL (§2, §10 above).
- **C5 is answered as a spec question, not closed as a shipped capability.** "Where does recurring
  PAY semantics belong" now has a written answer (§7's own header) and a rule set (§7.3–§7.4) a future
  binding/profile can ratify against. Nothing about what any product actually does changed today.

---

## 8. Honest residual

- **Every primitive keeps its coordinator non-load-bearing except one, and it says so.** OFFER,
  REPUTATION, MATCH, ATTEST degrade to local-trust versions offline (web-of-trust, local order book,
  direct offers) and *still function* (DIRECTION §6). RESERVE needs no coordinator by construction.
  ESCROW is the honest exception: **its operator is structurally permanent** (licensed money-holding
  does not self-extinguish), and the design keeps only the weaker guarantee that the class is one,
  permissionless, competing, per-order, replaceable, and never key-holding (TRACT §9.6).
- **The four root ceilings (DIRECTION §8) account for every residual above, and no seam closes them:**
  global anti-Sybil (caps REPUTATION and offer-spam), the physical-event oracle (caps ATTEST and the
  ESCROW dispute), the legal/authoritative-issuer burden (the ESCROW operator absorbs it for pay — the
  burden moves, it does not vanish), and editorial governance (which OFFER/REPUTATION indexes decline
  to centralise, at the cost of one authoritative answer).
- **What is genuinely future-proofed vs. genuinely fundamental.** *Technical* gaps converge on
  centralised quality as the frontier improves — a better personhood method is a REPUTATION binding
  swap, a TEE matcher is a MATCH coordinator swap, a TEE index is an OFFER/REPUTATION indexer swap —
  with no rearchitecture (research/README §4). The *structural* residuals (opt-in escrow declined,
  self-dealing attestations, Sybil-cost floor, non-custodial dispute deadlock) are consequences of not
  being a single surveilling company, and several are the point — disclosed, not solved.
- **Trust/dispute research returned nothing verified** across the grounding passes (TRACT §21.1). The
  reasoning here is checked for internal consistency against the anchor specs; it MUST NOT be read as
  evidenced, and the maturity/demand claims in [`bindings/README.md`](../../bindings/README.md) are a
  2026-07 snapshot to be re-checked before any binding is relied on in production.
