# Substrate — Offline Degradation & Reconcile (the apocalypse-proof property)

> **Status:** additive normative profile of the core specification and the coordinator contract.
> It restates no wire bytes. Its job is to make one property in [`DIRECTION.md § 6`](../DIRECTION.md)
> — *"remove connectivity and every service collapses to its local-trust version and still works"* —
> a **checkable obligation on every primitive and profile**, rather than an aspiration stated once and
> never enforced. Everything here is a *composition* of machinery specified elsewhere: the signed CRDT
> and version-vector reconciliation of [`SYNC.md`](SYNC.md), the store-and-forward roles of
> [`ROLES.md`](ROLES.md) (mailbox, relay, wake), the self-authenticating objects of [`FEEDS.md`](FEEDS.md)
> and §22, and the payment *seam* (attestations, never funds) of the profiles. Where this document and a
> normative-byte home disagree, the byte home governs; this document owns only the **degradation
> vocabulary, the reconcile obligations, and the offline-money framing**.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174.

---

## 1. The one idea, and the sneakernet test

A KOTVA object is self-authenticating: it is signed by a key or named by its content hash, so it verifies
identically no matter how it arrived — over a mesh, over HTTPS, or on an SD card carried by hand (Secure
Scuttlebutt's property; §11). Connectivity therefore governs **reach**, never **function**. A coordinator
([`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md)) *adds* reach — a global view, a scarce egress, a
real-world anchor — and by the contract's own **non-load-bearing** clause its absence must never gate
function. This document is what turns that clause into per-primitive requirements.

> **The sneakernet test** — *"does it still work with the network unplugged, and heal when it returns?"*
> A capability passes iff (a) every action a user can take offline is either **completed locally** or
> **captured as a signed intent that completes on reconnect** — never silently dropped and never silently
> treated as if it completed; and (b) on reconnect the capability **reconciles deterministically** from
> the two replicas' state alone, with no coordinator required to referee. A capability that needs a live
> server to make progress, or that *loses* an offline action, or that *fabricates* a completion that did
> not happen, fails the test.

Passing this test is not free (§10). What it buys is stated plainly: the substrate is **coordinator-
optional**, so an internet shutdown, a censored region, a basement, a tunnel, a ship, or a collapse
degrades KOTVA to its local-trust form and it keeps running.

---

## 2. The degradation grades (normative vocabulary)

Every offline-reachable action a primitive or profile exposes MUST be classifiable into **exactly one** of
four grades, and an implementation SHOULD surface the grade to the user at the point of action (as the
coordinator contract requires visibility to be surfaced, §CONTRACT 3.2).

| Grade | Meaning | Offline behaviour |
|---|---|---|
| **`full`** | Local-first; identical online and offline. No coordinator was ever in the path. | Proceeds normally. |
| **`local-trust`** | Works offline, but with the **local / web-of-trust anchor** substituted for a coordinator's global view ([`DIRECTION § 6`](../DIRECTION.md) table). | Proceeds at reduced assurance; the substituted anchor is disclosed. |
| **`deferred`** | The action is captured **now** as a signed intent and **completes/settles on reconnect** against an external rail or coordinator. | Captured locally; **MUST NOT be presented as complete** until it is. |
| **`blocked`** | Cannot proceed offline (needs a live scarce resource — settlement finality, port-25 egress, a fresh personhood proof). | **MUST fail closed** and say why — never a silent no-op, never a silent downgrade. |

Two rules bind the vocabulary (both inherited from [`README`](README.md)'s §3 fail-closed invariant —
*"a capability MUST NOT be silently degraded"* — and §CONTRACT 3.2 no-silent-downgrade):

- **R-GRADE-1 — No silent degradation.** A capability MUST NOT silently drop from a higher grade to a
  lower one. Moving from `full`/`local-trust` operation to `deferred` or `blocked` because connectivity
  was lost MUST be surfaced as an explicit state, not hidden behind a spinner or a fake success.
- **R-GRADE-2 — No fabricated completion.** A `deferred` action MUST carry, and display, its
  *unsettled* status until reconcile confirms it. Rendering a captured intent as a finished fact is the
  same defect as a coordinator fabricating a receipt (§CONTRACT 6) and is non-conformant.

---

## 3. Per-primitive degradation rules (five of the six waist capabilities)

Each of the five waist capabilities below ([`DIRECTION § 1`](../DIRECTION.md)) MUST degrade as follows.
The grade in each heading is the **best** grade the capability reaches offline; specific actions may be
lower. **Transport** has no subsection of its own: its offline behaviour is *MOTE delivery* (§3.2,
`deferred` store-and-forward) carried by the roles in §3.5.

### 3.1 Identity — `full`
A keypair is the identity; verifying a `DeviceCert` chain to an `IK` needs no network
([`IDENTITY.md`](IDENTITY.md)). Signing, verifying, and encrypting are `full` offline.

- **`local-trust`** for the parts that reach outward: **name resolution** (DNS/KT `name→key`, §3.3) is
  unavailable offline, so an implementation MUST fall back to keys already pinned/cached and to the
  8-word **key-name floor** (§3.9.6), which is verifiable offline by construction, and MUST NOT treat an
  unresolvable name as an unknown or untrusted key when a pinned binding exists.
- **`blocked`** for actions that need a live external anchor: a **fresh proof-of-personhood** (World ID /
  Human Passport) and **coordinator-brokered key recovery** (account-abstraction social recovery,
  MPC-share reassembly) require reachability; offline they fail closed. A previously-issued personhood
  **attestation** (an EAS/VC credential already held) is `full` — it is a signed object like any other.
- **R-ID-1 — Monotonic counters survive power loss.** Every rollback-defended counter that must not
  regress offline — `Identity.version` (§1.3), `LocationRecord.seq` (§4.2), feed `seq`, and the Sync HLC
  wall/counter (§3 of [`SYNC.md`](SYNC.md)) — MUST be persisted to non-volatile storage **before** the
  signed object that bears it is emitted, so an offline crash-and-restart can never re-issue a lower
  value. (The HLC absorbs wall-clock skew across offline replicas; the *counter* must still be durable.)

### 3.2 MOTE (transport object) — `full` create / `deferred` deliver
Composing, signing, sealing, and content-addressing a MOTE is `full` offline (§2). **Delivery** is
`deferred`: an outbound MOTE is queued and dispatched when a path appears. This is exactly DTN
store-carry-forward (RFC 9171) and Briar's mailbox pattern — the sender's obligation ends when a carrier
accepts custody, not when the recipient is online (§11).

- **R-MOTE-1 — Custody is store-and-forward at the edge, self-authenticating end to end.** Any node MAY
  carry a queued MOTE toward its recipient (mesh hop, mailbox drop, sneakernet); because the MOTE is
  sealed and signed, a carrier is `blind`/`blind-routing` (§CONTRACT 3.1) and is **trusted for nothing**
  — it can neither read nor forge. An implementation MUST NOT require the recipient to be simultaneously
  online (the anti-pattern §ROLES's mailbox exists to remove).
- **R-MOTE-2 — Self-contained cold-contact proofs are offline-verifiable; issuer-redemption proofs are
  not.** Of the `challenge` proofs gating a cold MOTE (§2.2b, §9 — PoW, postage stamp, ARC token, vouch),
  only the **self-contained** ones — PoW and vouch — MUST be verifiable by the recipient **without a
  coordinator** (checkable from the envelope alone, before decryption). This is what lets anti-abuse
  survive an internet shutdown for those two: authorisation is local, so the coordinator contract's
  *authorise-never-classify* rule (§CONTRACT 4) holds for PoW/vouch even with every coordinator
  unreachable. **Postage stamp** is the opposite case: its issuer signature verifies offline, but its
  single-use/unspent check needs the issuer's redemption endpoint or signed spent-list (§9.5.1,
  normative) — offline, a recipient MUST treat a stamp as `deferred`/`blocked` for admission, MUST NOT
  accept it on signature validity alone, and MUST fall back to the PoW/vouch tier, matching §9.5.1's "no
  offline bearer acceptance of real money." **ARC token**'s per-recipient rate budget is locally
  enforceable offline, but its issuer-level trust and revocation are not, so an offline recipient honours
  only the local budget check, never a claim of un-revoked issuer standing.
- **`deferred`/`blocked`** for reach that needs infrastructure: an outbound MOTE to a peer whose
  `LocationRecord` has expired (§3.1, TTL staleness) is `deferred` on the mailbox/relay; a **legacy-gateway
  egress** (SMTP, the one scarce-resource exception, §CONTRACT 2.3) is `blocked` offline.

### 3.3 PUB (public objects & feeds) — `full` read-cached / `full` author / `deferred` distribute
An author appends to a signed feed offline (`full`); a reader verifies any cached feed entry or blob
offline (`full`, self-verifying, §22.5). **Distribution** to the swarm is `deferred`. Indexes over public
objects are derived and rebuildable (§22.4.3), so their staleness offline is a `local-trust` view, never a
correctness failure.

### 3.4 SYNC (multi-author CRDT) — `full`
This is the capability built for the case: leaderless, offline-tolerant, deterministic merge
([`SYNC.md § 1`](SYNC.md)). Offline edits are `full`; reconcile is §4 below. The one caveat is not
availability but *semantics*:

- **R-SYNC-1 — Convergence is not invariant preservation.** A CRDT guarantees the two replicas reach
  **byte-identical state** on reconnect; it does **not** guarantee that state satisfies an application
  invariant that neither replica could check while partitioned (Automerge's own model-checking result,
  §11). Any invariant that two offline writers can jointly violate — non-negative stock, a unique
  single-winner assignment, a spend not exceeding a balance — MUST be enforced by a **single-writer
  authority** for that resource (the [`DIRECTION § 2`](../DIRECTION.md) RESERVE pattern: bookings against
  a single-owner calendar are structurally un-double-bookable) **or** be reconciled as a detectable
  **conflict** at merge time (§4.3), **never** assumed away by the merge. A profile that needs a
  cross-replica invariant offline MUST state which of the two it uses.

### 3.5 Roles & Wake — `local-trust` (mesh) / `blocked` (needs infrastructure)
Roles are the infrastructure primitives, so their offline story is the most conditional. On a **local
mesh** with no internet (Meshtastic/LoRa, Bluetooth, LAN), announce/resolve, relay, mailbox and cache
degrade to their `local-trust` forms — reachability is whoever is in range (§11). With **no network at
all**, roles that reach outward are `blocked`; roles that hold-and-carry (mailbox, cache/pin) still serve
locally.

- **R-ROLE-1 — Wake is `blocked` and MUST NOT be relied on offline.** Content-free push (Web Push /
  UnifiedPush, §ROLES ⑤) needs a live push service. A profile MUST NOT design a flow whose *correctness*
  depends on a wake arriving; wake is an optimisation over polling, and offline it degrades to "the peer
  finds out on next contact." (WRAP already assumes this — it converges on reconnect, §profiles/wrap 1.4.)

---

## 4. Reconcile-on-reconnect (normative)

When connectivity returns, a replica MUST heal from the two endpoints' state alone. There is exactly one
reconcile mechanism per primitive; all are already specified, and this section only makes their invocation
on reconnect mandatory.

| What was `deferred`/diverged offline | Reconcile mechanism (specified in) | On conflict |
|---|---|---|
| Divergent CRDT state | version-vector diff → range-Merkle drill-down → op exchange ([`SYNC § 4`](SYNC.md)) | deterministic join; ties broken by HLC then value-bytes (§SYNC 2.2) |
| Missed feed entries | fetch entries newer than last-seen `seq`, walk `prev` chain (§22.4.2) | append-only; a fork in an author feed is a **detectable equivocation**, surfaced not merged |
| Queued outbound MOTEs | drain mailbox / dispatch on fresh `LocationRecord` (§ROLES, §14.5) | idempotent by content-address `id`; duplicates dedup, they do not double-apply |
| Captured payment **intents** | settle against the external rail (§5) | double-commitment detected, not prevented (§5.2) |

- **R-REC-1 — Reconcile is idempotent and order-independent.** Replaying the drain twice, or draining a
  duplicate a Meshtastic-style flood produced (§11), MUST converge to the same state (content-address and
  op-id dedup already give this; a profile MUST NOT add a non-idempotent side effect to delivery).
- **R-REC-2 — Equivocation and over-commitment are surfaced, not swallowed.** Where reconnect reveals
  something a partition allowed that an online replica would have refused — a forked feed, a resource
  committed twice, an assignment claimed by two writers — the implementation MUST surface it as a
  **conflict for the single-writer authority or a dispute coordinator to resolve** (§CONTRACT 5 arbiter),
  and MUST NOT let the CRDT's "successful merge" mask it. Convergence hid nothing about *availability*; it
  must not be allowed to hide a *broken invariant* (§3.4, R-SYNC-1).

---

## 5. The hard case: offline money and double-spend

This is where the property is genuinely difficult and where the temptation to overclaim is greatest. KOTVA
holds the line it holds everywhere: **the protocol carries attestations, never funds, and mints no token**
([`DIRECTION § 5`](../DIRECTION.md), profiles/tract §9.2). So "offline money" is never KOTVA moving value
offline; it is one of **three named strategies for what happens to a payment when the settlement rail is
unreachable**, each a swappable seam, none inventing a currency. A profile that supports offline trade
MUST declare which strategy a given trade uses, and MUST surface it before the parties commit.

### 5.1 The trilemma (the fact that forces a choice)
Offline, double-spend is *trivial* to attempt: with no ledger to consult, a payer can promise the same
value twice (§11, the CBDC-offline literature). There are only three honest responses, and each pays a
different price:

| Strategy | Grade | Double-spend is… | Price paid | KOTVA mapping |
|---|---|---|---|---|
| **A. Hardware ecash** | `full` for the transfer itself; load/redeem/reconcile are `blocked` offline | **prevented** at the moment of transfer — value lives in tamper-resistant hardware that cannot be spent twice offline | trust moves to a chip vendor **and** an issuer: value must be loaded and redeemed with the issuer online, and real schemes cap offline spend/holding and require periodic online reconciliation to reset double-spend counters | a **binding** to an external offline-CBDC/SE ecash system, with an issuer in the load/redeem path; visibility `structural` (SE) or `attested` (TEE) |
| **B. IOU-reconcile** | `deferred` | **not prevented; bounded** — a signed obligation that may over-commit | becomes uncollateralized **credit**, capped by counterparty trust/reputation | a `deferred` obligation object that nets and settles on reconnect |
| **C. Settle-on-reconnect** | `deferred` → `blocked` settlement | **avoided** — no value moves offline at all | the trade's *object* completes but its *money* waits for the rail | the substrate default: a `PaymentAttestation` (tract §9.4.1) is simply `deferred` |

- **R-MONEY-1 — Only hardware prevents; the other two detect or avoid.** An implementation MUST NOT claim
  a strategy **B** or **C** offline payment is double-spend-*proof*. B *detects* over-commitment on
  reconnect and resolves it as a dispute/credit loss; C *avoids* the problem by not moving value offline.
  Prevention offline exists **only** under **A**, and **A**'s guarantee is exactly as strong as the
  tamper-resistance of the hardware and no stronger — disclosed at the `structural`/`attested` assurance
  level, never as trustless (§CONTRACT 3.3, bindings TEE row).

### 5.2 Rules that hold across all three
- **R-MONEY-2 — Strategy is declared, like a rail class.** The chosen strategy MUST be carried and
  surfaced with the same weight as `RailClass` (tract §9.3), because — like rail class — it **changes the
  buyer's recourse**. A `deferred` (B/C) outcome MUST NOT be presented as settled (R-GRADE-2). Silently
  substituting one strategy for another is the rail-class-substitution defect (tract §9.3) and MUST be
  refused.
- **R-MONEY-3 — Settle-on-reconnect is the honest default.** Because the substrate already separates the
  *trade* (objects that merge offline: order placed, work done, delivery confirmed — all `full`/`deferred`
  via §3–§4) from the *settlement* (an attestation against an external rail), strategy **C** falls out for
  free and requires no new machinery: the trade completes offline, `RailClass` finality is `blocked` until
  a rail is reachable, and the `PaymentAttestation` is written on reconnect. A profile SHOULD default to
  **C** and treat **A**/**B** as opt-in.
- **R-MONEY-4 — Reconcile resolves over-commitment through the existing dispute/escrow path, not a new
  one.** When strategy **B** reconnect reveals an over-commitment, resolution is the *already-specified*
  arbiter/escrow coordinator (§CONTRACT 5, tract §9), the reputation hit (OpenRank / local), and the
  credit write-off — **never** a protocol-level clawback and never a minted make-good. This keeps
  offline money consistent with the standing rule that KOTVA custodies nothing.

### 5.3 What this does not solve (carried, not hidden)
- **Uncollateralized credit stays unsolved.** Strategy **B** *is* credit, and credit is one of the
  root-blocked services ([`docs/research § 1`](../docs/research/README.md)); offline capability does not
  change that, it just names where the loss lands.
- **Hardware trust is a real dependency, and so is the issuer.** Strategy **A** trades operator-trust
  for chip-vendor-trust and inherits the side-channel history disclosed for every TEE binding
  ([`bindings`](../bindings/README.md), `attested` row) — and it depends on an **issuer** to load value
  into the hardware and to honour redemption/deposit, the same coordinator-class dependency Strategy
  **C**'s settlement leg carries (§5.1). The real SE/offline-CBDC constructions §9 cites additionally
  impose **offline spend/holding caps** and require **periodic online reconciliation** to reset
  double-spend counters, so offline behaviour is more constrained than online, not identical to it. It
  is not trustless, coordinator-free offline cash; it is hardware-trusted, issuer-dependent offline
  cash, `full` only for the transfer moment itself (§5.1).
- **Physical custody is still not trustless** (tract §9.6): none of the three strategies changes that the
  goods leg's "did it arrive?" reduces to confirm-plus-dispute (the physical-event oracle ceiling,
  [`DIRECTION § 8`](../DIRECTION.md)), which is *more* exposed offline because the oracle is unreachable.

---

## 6. Per-profile obligations

A profile MUST document, in one place, the grade of every user-facing action offline and its reconcile
mechanism. The two shipped profiles set the pattern:

- **TRACT (commerce).** Catalogue/offer authoring and reading = `full` (signed feeds, §3.3). Cart across
  sellers = `full` (buyer-held CRDT, tract §6). Order placement = `deferred` delivery (§3.2). Settlement =
  §5, defaulting to strategy **C**; escrow election is `blocked` offline (needs the operator reachable),
  so an offline order MUST disclose that escrow will attach on reconnect or not at all, never silently
  drop to unescrowed (tract §9.4.2 fail-closed, restated for the offline case).
- **WRAP (work).** Already designed for it (§profiles/wrap 1.4): offer/bid/assign/progress are CRDT ops
  that converge on reconnect (`full`), issuer-assigns removes the race so no coordinator referees, and
  wake is treated as optimisation not correctness (R-ROLE-1). WRAP's offline story is the reference for
  R-SYNC-1's single-writer form: the assignment register is single-writer by the issuer-assigns rule, so
  two couriers cannot both hold the job even if both were offline when they bid.

New profiles inherit these rules; a profile that cannot honour the sneakernet test for an action MUST mark
that action `blocked` and say why, rather than pretend it degrades.

---

## 7. Conformance checklist

| # | An implementation… | Rule |
|---|---|---|
| OFF-1 | classifies every offline-reachable action into exactly one degradation grade and surfaces it | §2 |
| OFF-2 | never silently degrades grade and never fabricates a `deferred` completion | R-GRADE-1/2 |
| OFF-3 | persists every rollback-defended counter before emitting the object that bears it | R-ID-1 |
| OFF-4 | queues outbound MOTEs store-and-forward; carriers stay blind and are trusted for nothing | R-MOTE-1 |
| OFF-5 | verifies self-contained cold-contact proofs (PoW, vouch) offline with no coordinator in the path; treats issuer-redemption proofs (postage, ARC issuer trust/revocation) as `deferred`/`blocked` offline and never accepts them on faith | R-MOTE-2 |
| OFF-6 | enforces cross-replica invariants by single-writer authority or detectable conflict, never by merge | R-SYNC-1 |
| OFF-7 | does not depend on wake for correctness | R-ROLE-1 |
| OFF-8 | reconciles idempotently and surfaces equivocation/over-commitment for resolution | R-REC-1/2 |
| OFF-9 | declares the offline-money strategy; never claims B/C prevents double-spend | R-MONEY-1/2 |
| OFF-10 | defaults to settle-on-reconnect; resolves over-commitment via existing dispute/escrow, no clawback/mint | R-MONEY-3/4 |

---

## 8. Honest residual

- **Offline is a reach property, not a magic one.** The substrate makes *function* survive a partition; it
  cannot make a **scarce live resource** appear where there is no network. Settlement finality, legacy-SMTP
  egress, a fresh personhood proof, and coordinator-brokered recovery are `blocked` offline by nature, and
  this document's honesty is in marking them so rather than faking a degraded version.
- **Double-spend prevention offline requires hardware, full stop.** Everything software-only can do is
  *detect and reconcile* (strategy B) or *avoid* (strategy C). Naming the trilemma does not dissolve it;
  it relocates the price to a place the user can see before paying it.
- **Convergence can hide a broken invariant.** A CRDT's clean merge is a statement about *availability*,
  not *correctness*. R-SYNC-1/R-REC-2 force the invariant question into the open, but the underlying truth
  remains: two parties who could not talk cannot have jointly respected a rule neither could check. Some
  offline conflicts are only *resolvable*, never *preventable*, and a profile that pretends otherwise is
  the defect this document exists to catch.
- **Local-trust is genuinely weaker.** Degrading personhood to web-of-trust, reputation to direct-local,
  and dispute to a known local arbiter ([`DIRECTION § 6`](../DIRECTION.md)) keeps the service *running*; it
  does not keep it at *global-coordinator quality*. The gap is the honest cost of not needing the internet,
  and it is disclosed, not papered over.

---

## 9. Grounding (informative)

The patterns this document composes, and where the primary sources sit (a 2026-07 snapshot; re-check
before relying on any of them):

- **Delay/disruption-tolerant networking — store-carry-forward, custody transfer.** IETF
  [RFC 9171 (Bundle Protocol v7)](https://datatracker.ietf.org/doc/rfc9171/);
  [DTN7 implementation](https://arxiv.org/pdf/1908.10237). KOTVA's MOTE delivery (§3.2) *is* this pattern,
  but with end-to-end self-authenticating objects, so a carrier needs no custody-signaling trust.
- **Self-authenticating feeds over any carrier, including sneakernet.**
  [Secure Scuttlebutt](https://ssbc.github.io/ssb-db/) — signed append-only logs replicated by gossip or
  SD card. This is the §22 feed model and the §1 sneakernet property.
- **Delay-tolerant mesh + offline mailbox for messaging.**
  [Briar / Bramble](https://briarproject.org/how-it-works/) — Bluetooth/Wi-Fi mesh with a store-and-forward
  mailbox for when a contact is offline. This is the §3.2 / §3.5 degradation of MOTE + roles.
- **LoRa mesh with store-and-forward and flood dedup.**
  [Meshtastic store-and-forward](https://meshtastic.org/docs/configuration/module/store-and-forward-module/)
  and [mesh broadcast algorithm](https://meshtastic.org/docs/overview/mesh-algo/) — the reference for
  `local-trust` reachability with no internet and for R-REC-1's flood-duplicate idempotency.
- **Local-first CRDT reconcile, and the invariant caveat.**
  [Automerge](https://automerge.org/docs/hello/) and its
  [conflict model](https://automerge.org/docs/reference/documents/conflicts/) — offline edits merge
  deterministically, but invariant preservation is a level above the merge operator (R-SYNC-1). This is the
  [`SYNC.md`](SYNC.md) reconcile engine.
- **Offline digital cash — the hardware-vs-detection dichotomy.**
  [Secure/privacy-preserving CBDC offline payments via a Secure Element](https://eprint.iacr.org/2024/1746/);
  [offline digital euro (Groth-Sahai)](https://arxiv.org/pdf/2407.13776). These establish §5.1's trilemma:
  offline double-spend is *prevented* only by tamper-resistant hardware; software-only schemes *detect* it
  at deposit and de-anonymize the double-spender. KOTVA binds to such a system (strategy A) rather than
  building one, and defaults to not moving value offline at all (strategy C).
