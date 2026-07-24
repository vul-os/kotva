# Primitive — ESCROW (conditional value transfer)

> **Status:** additive normative primitive spec of the KOTVA family. It restates no profile
> bytes and mints no currency. It abstracts the **generic escrow primitive** — *hold value, then
> move it only when a stated condition is met* — out of its first fully-worked instance, TRACT
> ([`profiles/tract/09-settlement.md`](../profiles/tract/09-settlement.md),
> [`§18.5`](../profiles/tract/18-state-machines.md)), so that any profile (commerce, gig,
> bookings, marketplaces) reuses one shape rather than re-deriving one. It sits under
> [`DIRECTION.md § 2`](../DIRECTION.md) (`OFFER · MATCH/RESERVE · REPUTATION · ESCROW · ORACLE ·
> DISPUTE · PAY`) and inherits [`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md),
> [`THREAT-MODEL.md`](../THREAT-MODEL.md), and [`substrate/OFFLINE.md`](../substrate/OFFLINE.md)
> unchanged. Where a profile owns the frozen bytes, the profile governs; this document owns only
> the **primitive's shape, its rules, and its disclosed ceiling**.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are BCP 14 (RFC 2119 / RFC 8174).

---

## 1. Purpose

ESCROW is the primitive that lets two parties who do not trust each other **transact anyway**: a
payer commits value, the value is **held** against a stated release condition, and it moves to the
payee only when that condition is satisfied — or is refunded, or split, if it is not. It is the
`ESCROW` term every gig, delivery, freelance, auction, and classifieds service composes
([`DIRECTION § 2`](../DIRECTION.md)); the services differ only in *which condition* releases.

Two rules from the family govern before any mechanism:

- **The protocol carries attestations, never funds** (TRACT §9.2, [`DIRECTION § 5`](../DIRECTION.md)).
  KOTVA moves, custodies, and converts nothing. ESCROW is a set of **signed attestations about a
  hold that lives on an external rail** — never a KOTVA-held balance and never a token.
- **The hold's assurance is a binding, not our invention** (§7). Whether double-spend or premature
  release is *prevented* or merely *detected* is exactly the strength of the adopted mechanism
  (multisig / HTLC / smart-contract / a custodial operator), disclosed at its assurance level.

---

## 2. Objects it defines (wire-shape sketch)

**Illustrative, not frozen.** The bytes below are a family-level sketch. `EscrowScope`,
`PaymentAttestation`, `money`, `RailClass`, `identity-key`, and `content-address` are **already
frozen by a profile** (TRACT §16.5.4 / §16.6 / §16.3) and are **referenced, never redefined**. The
`EscrowTransition` and `EscrowRuling` shapes fill a **known profile gap** — TRACT §9.4.3 / §18.5
record that §16 today carries *no* signed object for an escrow state transition or a ruling — so
they are marked **PROVISIONAL**: a profile freezes the keys and the DS-tag; this primitive fixes
only their *content* and their *rules*. Encoding is deterministic CBOR (RFC 8949 §4.2, SEC-3).

```cddl
; ---- referenced from the profile, not redefined ----
;   EscrowScope        TRACT §16.5.4  — operator's published, per-order cover
;   PaymentAttestation TRACT §16.6    — "a settlement happened"; a reference, never funds
;   RailClass          TRACT §16.5.4  — 0 CustodialReversible | 1 NonCustodialFinal  (part of the type)

EscrowState = &( funded: 0, held: 1, released: 2, refunded: 3, split: 4 )   ; = TRACT §18.5

Mechanism = &(
  multisig:       0,   ; k-of-n signatures over the settlement transaction
  htlc:           1,   ; hashlock + timelock on the rail
  smart-contract: 2,   ; on-rail programmatic escrow
  custodial:      3,   ; a licensed operator holds the float and can rule (the only one that breaks a true deadlock, §10)
)

ReleaseCondition = {          ; what must be true for `held` → `released`
  1 => Mechanism,             ; how the hold is enforced on the rail (§7)
  2 => uint,                  ; k — confirmations required to release; dual-confirm ⇒ k = 2 (§5)
  3 => [* identity-key],      ; the confirmers whose attestations count (buyer, seller, an ORACLE)
  4 => ? identity-key,        ; arbiter to fall back to on dispute — a DISPUTE coordinator (§5, §6)
  5 => ts,                    ; confirm / dispute deadline; expiry has a *named* destination, never bare (§4)
  6 => RailClass,             ; carried here too, so the condition cannot be silent about recourse
}

EscrowTransition = {          ; PROVISIONAL — the object TRACT §16 does not yet carry (§9.4.3)
  1 => identity-key,          ; signer: the operator, or a `ReleaseCondition` confirmer
  2 => content-address,       ; the order / trade this escrow secures (sealed order, by address only)
  3 => EscrowState,           ; from
  4 => EscrowState,           ; to
  5 => ? content-address,     ; evidence: an ORACLE/confirm attestation, a ruling, a dispatch proof
  6 => ts,
  7 => ? RailClass,           ; elected rail class — REQUIRED when `to` is released/refunded/split (N-2)
  8 => bstr,                  ; signature over the DS-tagged preimage of keys 1..7 (SEC-2)
}

EscrowRuling = {              ; PROVISIONAL — the DISPUTE-fallback disposition; likewise a §16 gap
  1 => identity-key,          ; arbiter (a DISPUTE coordinator, Kleros-class §7)
  2 => content-address,       ; the order / trade
  3 => &( release: 0, refund: 1, split: 2 ),
  4 => ? [* [identity-key, money]],  ; split ⇒ per-party amounts, one order, one currency (TRACT §5.4)
  5 => ? content-address,     ; the evidence bundle the ruling rests on
  6 => ts,
  7 => bstr,                  ; arbiter signature over keys 1..6
}
```

These are **not** new MOTE kinds in the core registry (§2.3). An `EscrowScope` is a **public**
object (like an offer); a `PaymentAttestation`, an `EscrowTransition`, and an `EscrowRuling` are
**sealed** per-party evidence (TRACT §18.6) — except a ruling, which a profile MAY additionally
publish so an operator that rules badly accumulates a permanent record (TRACT §9.5). The concrete
kind / key numbers are a profile-and-registry concern, deliberately not fixed here.

---

## 3. Normative rules

- **N-1 — Attests, never holds.** An ESCROW object MUST NOT carry funds, a card PAN, an account
  number, or any credential. It carries a `content-address` of the order and an **opaque** external
  settlement reference only (TRACT §9.2, §9.4.1). An implementation that puts value *in* a KOTVA
  object is non-conformant.
- **N-2 — `RailClass` is part of the type.** Every `ReleaseCondition`, `EscrowTransition`
  producing a `released`/`refunded`/`split`, and `PaymentAttestation` MUST carry the elected
  `RailClass`, and an implementation MUST NOT flatten or drop it. It MUST NOT substitute one rail
  class for the other mid-trade without a fresh recorded agreement by both parties
  (`ERR_TRACT_RAIL_CLASS_SUBSTITUTED`, TRACT §9.3), because it changes the buyer's recourse.
- **N-3 — Fail-closed scope, never a silent downgrade to unescrowed.** Escrow attaches **only**
  when both parties elected an operator whose `EscrowScope` covers the trade under the fail-closed
  intersection (TRACT §9.4.2). A missing, unparseable, or incomparable scope field means *not
  covered*. On an empty intersection an implementation MUST surface the unescrowed outcome
  **explicitly to both parties before they commit** (`ERR_TRACT_ESCROW_SCOPE_EMPTY`) and MUST NOT
  reach unescrowed as a silent default (TRACT §9.4.2, §9.5a; SEC-1).
- **N-4 — Every transition is signed and lock-stepped.** Each `EscrowState` transition MUST be a
  signed `EscrowTransition` whose signer is determinate (TRACT §18.6). It MUST advance in lock-step
  with the order machine: reaching order `closed` releases a `held` escrow, reaching order
  `cancelled` refunds it; an implementation MUST NOT release a `held` escrow while its order is open
  and undisputed, nor leave one `held` after the order closed or cancelled (TRACT §18.5).
- **N-5 — Expiry has a named destination.** `ReleaseCondition.deadline` MUST expire *into* a named
  state (release / refund / a ruling), never "expire" alone (TRACT §18.1, §18.6). On a custodial
  rail a dispute deadline expires into the operator's ruling; on an arbiter-cosigned non-custodial
  rail (§4) it expires into the arbiter's ruling, enforced by co-signature; only where no signer,
  including the ruling's beneficiary, will act does the disclosed default (§10) govern.
- **N-6 — The operator never holds identity keys.** An escrow operator holds a float; it MUST NOT
  hold, recover, or sign as a party's `IK` or `DeviceCert` (TRACT §9.5; SEC-5). It authorises and
  settles; it is not the identity substrate.
- **N-7 — No token.** Any stake an arbiter or operator posts, and all settlement, MUST be an
  **existing asset** ([`DIRECTION § 5`](../DIRECTION.md), CONTRACT §6). ESCROW mints nothing.
- **N-8 — A split ruling conserves the amount held.** For a `split` `EscrowRuling`, the per-party
  `money` amounts MUST be in the order's single currency (§2) and MUST sum exactly to the amount
  recorded as held for that escrow. An implementation MUST reject a split whose amounts do not
  conserve the held total (fail-closed, SEC-1).

---

## 4. Release, and the DISPUTE fallback

The release condition is a **dual-confirm** by default: `held` → `released` requires `k = 2`
agreeing confirmations from `ReleaseCondition.confirmers` — typically the buyer's confirmation and
either the seller's dispatch proof or an **ORACLE** attestation that the physical event happened
(delivery/ride/work — the physical-event anchor, CONTRACT §5 `oracle`, [`DIRECTION § 8`](../DIRECTION.md)).
Dual-confirm is why the primitive lists ORACLE as a sibling: the oracle supplies the second
confirmation the payee alone cannot self-certify.

When confirmations **disagree or are absent** at the deadline, the condition routes to
`ReleaseCondition.arbiter` — a **DISPUTE** coordinator — which emits a signed `EscrowRuling`
(`release` / `refund` / `split`) that the escrow machine applies (§2, TRACT §18.5). This is the
fallback path, not the happy path: a well-behaved trade never reaches it.

---

## 5. Composition with the other primitives

| Primitive | How ESCROW composes with it |
|---|---|
| **OFFER** | The offer names the price and MAY name an acceptable escrow operator set; ESCROW secures the trade an accepted offer becomes. |
| **MATCH / RESERVE** | Matching/booking decides *who* trades; ESCROW secures *the* trade. RESERVE (single-owner calendar) needs no escrow when nothing is prepaid. |
| **REPUTATION** | A `PurchaseAttestation` from a settled escrow gates reviews (TRACT §16.5.5, §10); an operator's ruling record feeds its own reputation (§9). |
| **ORACLE** | Supplies the second, non-self-certifiable confirmation for dual-confirm release (§4). The physical-event ceiling ([`DIRECTION § 8`](../DIRECTION.md)) is ORACLE's, inherited here. |
| **DISPUTE** | The fallback when confirmations fail (§4); emits the `EscrowRuling`. A staked arbiter is a coordinator (CONTRACT §5). |
| **PAY** | Settlement itself — the external rail. ESCROW brackets PAY with a hold; the `PaymentAttestation` (TRACT §9.4.1) is PAY's evidence, ESCROW's release trigger. |

ESCROW is the **only** primitive whose coordinator is structurally load-bearing-adjacent, and that
is disclosed (§9), not hidden.

---

## 6. Binding adopted

ESCROW **binds**, it does not build ([`bindings/README.md`](../bindings/README.md),
[`DIRECTION § 3`](../DIRECTION.md)). The `Mechanism` slot is the seam:

- **The hold** — an **HTLC**, an **on-rail smart contract**, or **k-of-n multisig** over a
  **stablecoin** settlement (bindings: *Payments / settlement* → x402 + stablecoins on the rail).
  These are the non-custodial mechanisms; their assurance is `structural` (the chain enforces the
  lock). An arbiter-cosigned multisig (§4) resolves a genuine dispute without a custodian; only a
  **true deadlock** — no signer, including the ruling's beneficiary, will act — has no move (§10).
- **The dispute** — **Kleros-class** staked arbitration (bindings: *Dispute / arbitration*), the
  DISPUTE coordinator of §4.
- **The physical confirmation** — the `oracle` coordinator (CONTRACT §5), itself resting on the
  physical-event ceiling.
- **The custodial alternative** — a licensed escrow **operator** (TRACT §0.4.2, §9.5): the only
  `Mechanism` that can *break* a true deadlock, at the cost of being structurally permanent (§10).

When the frontier improves (better on-chain dispute primitives, TEE-attested oracles), the filling
swaps and this primitive does not change ([`DIRECTION § 9`](../DIRECTION.md)). ESCROW owns the
seam and the rules; it owns no cryptography.

---

## 7. Scale-invariance

The primitive is identical from a village mesh to a planetary marketplace; only the **trust anchor**
slides ([`DIRECTION § 6`](../DIRECTION.md)):

| Function | Mesh / local (no coordinator) | Global (swappable coordinator) |
|---|---|---|
| The hold | multisig among people you know, or nothing (goods-on-handover) | HTLC / smart-contract / a licensed custodial operator |
| Dual-confirm oracle | a mutually-known neighbour vouches the event | a chosen `oracle` coordinator |
| Dispute fallback | a known local arbiter both parties accept | a staked Kleros-class arbitration market |

At local scale ESCROW collapses to *"a person we both trust holds it, or we hand over on delivery"*
— the web-of-trust form. At global scale each anchor becomes a swappable, per-order coordinator. The
mechanism is one; the anchor is what you choose.

---

## 8. Offline / apocalypse behaviour

Per [`substrate/OFFLINE.md § 5–§6`](../substrate/OFFLINE.md), ESCROW degrades as follows, and MUST
surface the grade before the parties commit (OFF-1, R-GRADE-1):

- **Escrow election is `blocked` offline.** Attaching escrow needs the operator reachable
  (`EscrowScope` fetch, float lock). An offline order MUST disclose that escrow will attach **on
  reconnect or not at all**, and MUST NOT silently drop to unescrowed (OFFLINE §6, N-3).
- **The trade objects still complete; settlement is `deferred`.** The order, dispatch, and confirm
  objects merge offline (`full`/`deferred`); the `PaymentAttestation` and any `EscrowTransition` are
  written on reconnect. This is strategy **C** (settle-on-reconnect), the honest default, and it
  falls out free from the substrate already separating the trade from its settlement (OFFLINE §5,
  R-MONEY-3).
- **IOU is the opt-in `deferred` alternative.** Where parties accept counterparty credit, a
  `deferred` obligation nets and settles on reconnect (strategy **B**). It does **not** prevent
  double-spend; it **bounds** it by counterparty trust, with over-commitment *detected* on reconnect
  (R-MONEY-1). An implementation MUST NOT present an IOU escrow as double-spend-proof.
- **Reconcile.** On reconnect, `EscrowTransition`s dedup idempotently by content-address (R-REC-1);
  an over-commitment or a two-signer race surfaced by reconnect is resolved through the **existing**
  DISPUTE/arbiter path and a reputation/credit write-off — **never** a protocol clawback and never a
  minted make-good (R-MONEY-4, R-REC-2, N-1).

---

## 9. Security MUSTs

ESCROW inherits all nine invariants of [`THREAT-MODEL.md § 3`](../THREAT-MODEL.md); the load-bearing
ones here:

- **SEC-1 (fail-closed).** The scope intersection (N-3), an unresolvable evidence `refs` (treat as
  mismatch, not release), an unparseable `RailClass`, and a non-conserving split ruling (N-8) all
  fail **closed** — deny, never admit.
- **SEC-2 (intrinsic authenticity).** Every `EscrowTransition`/`EscrowRuling` MUST be
  self-authenticating, DS-tag-domain-separated, and chain to an `IK` via a non-revoked `DeviceCert`.
  An unsigned transition is not evidence of anything (TRACT §18.6).
- **SEC-6 (coordinator).** The escrow operator and the arbiter MUST satisfy all four CONTRACT
  clauses — accountable, swappable (per-order, zero migration), self-hostable at local scale (§7),
  visibility-declared (`terminating` for evidence, disclosed, CONTRACT §5). They **authorise and
  rule; they never classify content**. Their audit is one-directional (R-6).
- **SEC-6a (custody risk).** A *custodial* escrow coordinator holds the trade float for the trade
  window and is a live counterparty: it can abscond, become insolvent, or freeze funds. It MUST
  disclose this custody risk before the trade and SHOULD be bonded / staked in an **existing** asset
  sized to the float (N-7) — never a protocol token.
- **SEC-8 (no downgrade).** No silent `RailClass` substitution (N-2); no silent downgrade of a
  disputed release into an automatic one.
- **DIRECTION § 5 (no token).** Restated as N-7.

---

## 10. Honest residual

Every disclosure here traces to two of the four root ceilings ([`DIRECTION § 8`](../DIRECTION.md)) —
the **physical-event oracle** and the **legal/authoritative-issuer** — and to a measured commerce
outcome. None is a bug this primitive can close; each is disclosed rather than solved.

- **A true deadlock has no non-custodial move.** An arbiter-cosigned k-of-n multisig (§4) — the
  OpenBazaar 2-of-3 model cited below — resolves a genuine two-party dispute without a custodian:
  the arbiter is a rail signer, and its `EscrowRuling` is enforced by its co-signature. The residual is
  narrower — a **true deadlock**, where no signer, including the ruling's beneficiary, will act.
  There, plain multisig / HTLC / smart-contract have **no move**. On a `NonCustodialFinal` rail the
  only honest options are a timeout that defaults to one party (a policy choice, not a neutral
  mechanism) or an indefinite lock. There is no third option; it MUST be disclosed before the trade
  (TRACT §9.6, §18.5). Only a **custodial** operator can rule its way out of a true deadlock —
  which is why one exists.
- **Opt-in escrow is declined by exactly the actors it targets.** A *measured* outcome
  (OpenBazaar, TRACT §9.5a, §21.6): mandatory escrow would exclude regions no operator serves, so
  escrow stays optional — and bad actors simply decline it. The primitive pays this cost by
  disclosing the unescrowed outcome, not by pretending optionality is free.
- **The escrow operator is structurally permanent.** Unlike DMTAP's self-extinguishing gateway,
  holding money for strangers is licensed and does **not** decay (TRACT §9.6, §0.4.3). What is
  preserved is only that the class is one, permissionless, competing, per-order, replaceable, and
  never holding identity keys. ESCROW is the one primitive whose coordinator does not fade, and
  says so.
- **The custodial operator holds the float — and could run, freeze, or fail.** The bigger dependency
  is not that it holds no identity key (N-6) but that it *does* hold your money for the trade window:
  it can abscond, become insolvent, or freeze funds — the counterparty risk real-world escrow custody
  is licensed and bonded against. This custodial operator is the family's **one honest load-bearing
  exception** (CONTRACT §1, [`DIRECTION § 0`](../DIRECTION.md)): every other coordinator is swappable
  scaffolding, but the party holding the float for the window genuinely holds it. Bounded by bonding /
  staking sized to the float (SEC-6a) and by per-order swappability, never eliminated.
- **Physical custody is not trustless.** ESCROW moves money, not goods; the goods leg's *"did it
  arrive?"* reduces to dual-confirm + dispute (the physical-event ceiling), which is *more* exposed
  offline where the oracle is unreachable (OFFLINE §5.3).
- **The transition and ruling objects are a live gap.** Their bytes are **PROVISIONAL** (§2): the
  first profile (TRACT §16) does not yet freeze them, so the *"every ruling is a published signed
  object"* guarantee is intended, not yet expressible on the wire (TRACT §9.4.3, §18.5). Recorded,
  not papered over.
- **Nothing in the trust/dispute/tax literature returned verified support.** The escrow reasoning
  is design checked for internal consistency; it MUST NOT be read as evidenced (TRACT §21.9, C6).
