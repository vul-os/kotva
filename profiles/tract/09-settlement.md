# 9. Settlement

> **Drafting status: partially normative.** The payment seam (§9.2), the rail-class type (§9.3),
> the `PaymentAttestation` rules (§9.4.1), the `EscrowScope` fail-closed intersection at checkout
> (§9.4.2), the operator-class and tax-facilitator constraints (§9.5), and the measured-cost and
> honest-limits reasoning (§9.5a, §9.6) are authored to normative RFC 2119 text, aligned exactly to
> the frozen `PaymentAttestation` (§16.6), `EscrowScope` and `RailClass` (§16.5.4) grammar and to
> the escrow state machine in §18.5. What remains **scoped, not normative** is the wire
> representation of an escrow *state transition* and of an escrow *ruling* — including a partial
> release / **split** (§9.4.3, §9.7): §16 today carries no signed object for either, so those
> parts are marked inline as **PROVISIONAL — pending decision** and logged as required §16 changes.
> The key words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are to be interpreted as in BCP 14
> (RFC 2119, RFC 8174) wherever they appear below.

## 9.1 Scope

The payment seam, rail classes, escrow, and the gateway's money role. This section owns the reasoning
behind `PaymentAttestation` and `EscrowScope` and the rules for eliciting an escrow operator's cover
at checkout; the byte authority is §16.5.4 and §16.6, and this section MUST NOT contradict it. The
escrow **state machine** is owned by §18.5; this section references it and MUST NOT restate its
transitions. The party-responsibility and tax consequences of a gateway settling are owned by §11.3
and §11.2a; this section references them.

## 9.2 The seam names no provider

TRACT specifies where money crosses the boundary and what must be verified. It specifies no rail,
no currency, no token and no ledger. Naming a provider in a protocol exports that provider's
jurisdiction and licensing to every implementor.

Therefore:

- A settlement object MUST NOT name, require, or hardcode any specific payment provider, bank,
  card network, chain, token, or ledger. The only settlement-typed distinction the wire carries is
  the `RailClass` of §9.3 — two classes, no more.
- The external settlement **reference** on a `PaymentAttestation` (§16.6, key 6) is an opaque
  `tstr`. A decoder MUST treat it as opaque and MUST NOT parse it as a provider-specific
  identifier or derive routing, credentials, or provider identity from it.
- Currency is carried per §16.3 `money` (ISO 4217 minor-units integers), and language, region and
  schedule per the standards named in the brief (C4). These are existing standards profiled by
  reference, not TRACT vocabulary and not a provider.

**The protocol carries attestations, never funds** (§16.6, §9.4.1). Nothing in this section moves,
custodies, or converts money.

## 9.3 Rail class is part of the type

Settlement happens on one of exactly two classes, carried on the wire as `RailClass` (§16.5.4):

| Value | Class | What it means for recourse |
|---|---|---|
| `0` | `CustodialReversible` | Chargebacks exist; the card network or custodian is already the adjudicator. A wronged buyer has an external reversal path. |
| `1` | `NonCustodialFinal` | Nobody custodies, nothing reverses. There is no external reversal path; recourse is only what the parties and any escrow operator arrange in-protocol. |

`RailClass` is part of the type because it **changes the buyer's recourse**, and the grammar carries
it on both `PaymentAttestation` (§16.6, key 5) and `EscrowScope` (§16.5.4, key 6) so that neither a
settlement record nor an operator's declared cover can be silent about it. An implementation MUST
NOT flatten `RailClass` to a boolean or drop it (§16.6, §16.7).

An implementation MUST NOT substitute one rail class for the other — a `CustodialReversible` rail
for a `NonCustodialFinal` rail, or the reverse — without a fresh, explicit agreement by both parties
recorded on the order, because the buyer's recourse differs between them. A substitution absent that
recorded agreement MUST be rejected as `ERR_TRACT_RAIL_CLASS_SUBSTITUTED` (§17, `0x0802`,
fail-closed-block).

## 9.4 What this section specifies

Three things: what a payment attestation is and is not (§9.4.1); how an escrow operator's cover is
matched against a concrete trade, fail-closed (§9.4.2); and the escrow lifecycle, which is owned by
§18.5 and whose per-transition wire object is a §16 gap this section does not invent (§9.4.3).

### 9.4.1 `PaymentAttestation` attests only — it never moves funds

A `PaymentAttestation` (§16.6) records that a settlement occurred between a payer and a payee for an
order:

```cddl
PaymentAttestation = {
  1 => identity-key,   ; payer
  2 => identity-key,   ; payee
  3 => content-address,; order — the sealed order's address
  4 => money,          ; amount
  5 => RailClass,      ; §9.3
  6 => tstr,           ; external settlement reference, opaque
  7 => ts,
}
```

- A `PaymentAttestation` MUST carry a **reference** only (key 6), never funds and never card or
  account data — no PAN, no account number, no credential of any kind (§16.6, §9.2). The protocol
  conveys *that* a payment happened; it does not move money and it is not a payment instrument.
- The `amount` (key 4) MUST be a single `money` value denominated in the order's currency
  (§16.6 `Order.total`, §5.4). An implementation MUST NOT place a converted figure in a
  `PaymentAttestation`: a currency conversion is a presentation estimate (§5.11) and never what
  gets signed (§5.4, §16.7). `PaymentAttestation.amount` attests the value that actually settled,
  in the currency the order settled in — one order, one seller, one currency (§16.6, §5.4).
- The `RailClass` (key 5) MUST equal the class the parties agreed for this settlement (§9.3).
- The attestation references the sealed order by its content address only (key 3); it MUST NOT
  reproduce order contents, buyer identity, or line items into any published object (§16.4, §9.2).
  A `PaymentAttestation` is itself a sealed object (§16.6), held at the endpoints.

Where an escrow operator mediates settlement, the operator is the party that observes the movement
of value and so is the natural issuer of the corresponding attestation(s); the escrow *state* those
attestations accompany is governed by §18.5 (§9.4.3). The separate public proof that an author
transacted — used to gate reviews — is `PurchaseAttestation` (§16.5.5), not this object; a
`PurchaseAttestation` names an escrow operator as attestor via its `attestor` field (`1 = escrow`)
and likewise references the sealed order by address only.

### 9.4.2 `EscrowScope` and the fail-closed intersection at checkout

An escrow operator publishes what it can lawfully serve as an `EscrowScope` (§16.5.4):

```cddl
EscrowScope = {
  1 => identity-key,   ; operator
  2 => [* country],    ; buyer_countries
  3 => [* country],    ; seller_countries
  4 => [* country],    ; supply_countries — checked against place of supply, not the parties
  5 => [* currency],
  6 => [* RailClass],
  7 => money,          ; max_order_value
  8 => [* tstr],       ; excluded_categories
  9 => [* tstr],       ; authorities claimed — prose
}
```

When both parties elect an escrow operator for a trade, an implementation MUST verify, before the
order is placed, that **every** one of the following holds for that operator's `EscrowScope`:

- the order's `buyer_residence` anchor (§11.2, §16.6 `Anchors`) is in `buyer_countries` (key 2);
- the order's `seller_establishment` anchor is in `seller_countries` (key 3);
- the order's `place_of_supply` anchor is in `supply_countries` (key 4). This is checked against
  **place of supply**, derived from the Fulfilment axis (§4, §16.5.2), and MUST NOT be checked
  against the party countries instead: two trades with identical buyers, sellers, currency and rail
  can differ only in where supply happens, and that difference alone can put one outside an
  operator's licence (§16.5.4 note, §9.5);
- the order's currency is in `currencies` (key 5);
- the elected `RailClass` (§9.3) is in `rail_classes` (key 6);
- the order `total` does not exceed `max_order_value` (key 7), compared in the same currency only
  (§5.4) — a comparison across currencies MUST be refused, not coerced (§16.7);
- none of the order's tax-treatment or product categories is listed in `excluded_categories`
  (key 8).

The check is **fail-closed**: a missing, unparseable, or currency-incomparable field means the
operator does not cover the trade, not that it does. If no elected operator covers the trade, an
implementation MUST NOT silently downgrade the arrangement — for example by dropping to an
unescrowed trade, or by substituting a different rail class (§9.3). It MUST surface the empty
intersection as `ERR_TRACT_ESCROW_SCOPE_EMPTY` (§17, `0x0801`, deny-policy) and disclose the
resulting unescrowed outcome explicitly to both parties before they commit (§9.5a). An empty
intersection is a fact about the operator's declared terms, not a defect in what was presented,
which is why §17 classes it as a deny-policy outcome rather than a fail-closed-block.

The `authorities` field (key 9) is prose, because regulators share no schema; an implementation
MUST NOT parse it as a machine-checkable licence and MUST NOT treat its presence as verification of
anything. It is a claim by the operator, readable by a human.

### 9.4.3 Escrow lifecycle — owned by §18.5

The escrow lifecycle is **fund → hold → release / refund / split**, and its state machine —
every state, its trigger, its signer, and its timeout destination — is specified once in §18.5 and
referenced, not restated, here. Escrow is present only when both parties chose an operator whose
`EscrowScope` covers the trade (§9.4.2, §18.5). The escrow timeouts (`FUND_TIMEOUT`,
`DISPUTE_TIMEOUT`, and the shared `CONFIRM_TIMEOUT`) are §19.4 parameters.

**PROVISIONAL — pending decision.** §9.4 (and the original draft of this section) states that each
escrow lifecycle step is "a signed object," and §9.5 states that "every ruling is published as a
signed object." **The frozen grammar carries neither.** §16 has `EscrowScope` (public) and
`PaymentAttestation` (sealed) but **no object for an escrow state transition** (`funded` / `held` /
`released` / `refunded` / `split`) and **no object for an escrow ruling**. The §18.5 machine names
these states and names a "ruling" as a trigger, but nothing on the wire carries the ruling itself or
its signer. Making the lifecycle-is-signed and rulings-are-published claims normative therefore
requires a §16 MAJOR grammar change (an escrow-transition object carrying the operator's signature,
the from/to state, the order address, and any evidence reference; and, for a ruling, its
disposition). This section does not invent those bytes. Until §16 carries them, §18.5's transitions
are specified as a state machine whose *published-ruling* property (§9.5) is an intended guarantee
not yet expressible on the wire. Recorded as a required §16 change; see §9.7 for the split-specific
part.

## 9.5 Escrow is an operator class — and, when it settles, likely a tax facilitator

Escrow is the operator class of TRACT (§0.4.2). It requires legal standing, a payment-provider
relationship, a float, and licensing — none of it derivable from a keypair (§21.11.2). What bounds
it, and MUST hold for any conformant escrow role, is exactly what bounds the gateway class
generally:

- **Permissionless entry.** Any `IK` MAY publish an `EscrowScope` (§16.5.4) and offer to escrow;
  there is no registrar and no allow-list (§0.4.2).
- **Per-order choice by both parties.** Escrow is elected per order by both buyer and seller
  (§9.4.2, §9.6), never imposed protocol-wide.
- **No access to identity keys.** An escrow operator holds funds, never `IK`s or `DeviceCert`s
  (§0.9 "gateway"; identity is the substrate's,
  `github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md`). It cannot sign as, impersonate, or
  recover a party.
- **Competition and replaceability.** Operators compete on published scope and terms; a party MAY
  choose a different one per order.

**Not yet a MUST — an intended guarantee with no mechanism behind it.** The four bullets above bind
a conformant escrow role. The following does **not**, and is listed separately rather than inside
the MUST set precisely so it is not mistaken for one:

- **A verifiable record of rulings.** An operator that rules unfairly *should* accumulate a
  permanent, verifiable record, so that its conduct is legible to future counterparties. This is
  the intended guarantee of §9.4.3, but it has **no wire representation** — the §16 gap recorded
  there (**PROVISIONAL**). Until that object exists, nothing obliges an operator to emit such a
  record and no party can verify its absence, so an implementer MUST NOT rely on it as a check on
  operator conduct. The real bound on a bad escrow operator today is §9.5's licensing and the
  per-order choice above, not this.

**Escrow is also, most likely, a *tax* facilitator.** A gateway that settles — that receives buyer
funds and releases them, or routes payment through its own processor — is the actor most likely to
be deemed a marketplace facilitator, and in some US states escrow alone suffices to trigger that
status on its own (§11.2a, §21.11.2). §9.5's "escrow is an operator class" therefore *understates*
the exposure: escrow is also the single thing most likely to make that operator answerable for tax.
Where a gateway settles a payment, that gateway MUST be recorded as the `facilitator` on the
order's `Responsible` object (§16.6, §11.3) — its presence there is the marketplace-facilitator
hook, and its absence (a self-hosted seller taking direct payment) is equally load-bearing, because
such a seller is never a facilitator (§21.11.2, confirmed). TRACT carries this as a **fact** and
computes no tax from it (§11.2b, C3); none of the legal reasoning is settled law (§11.2a, §21.11.5),
and this section MUST NOT be read as making any deployment compliant.

## 9.5a The measured cost of optionality (§21.6)

§9.5 keeps escrow per-order and optional, and §9.6 gives the reason: mandatory escrow would exclude
regions no licensed operator serves. That reasoning is sound and the consequence is still real.

OpenBazaar's escrow was also opt-in, and **bad actors simply declined it** — the protection went
unused precisely where it mattered most (§21.6). Both cannot be true for free, and this
specification pays on the second. This is a **measured outcome, not a hypothetical**, and it is
stated here as one. What follows from it, rather than pretending it away, is normative:

- an unescrowed trade MUST be presented as an explicit, disclosed outcome shown to both parties
  before they commit, and MUST NOT be reached as a silent default or a silent downgrade from a
  failed scope intersection (§9.4.2);
- an interface SHOULD render the absence of escrow as a fact about the trade, not as a missing or
  incidental field;
- a seller declining every escrow provider a buyer trusts is itself information the buyer SHOULD be
  given, since it is the observable form of the failure above.

Attestation-gated reviews (§10) raise the cost of the adjacent failure modes but do not resolve
them: **self-dealing** and **opt-in escrow declined by exactly the actors it targets** both persist
as measured outcomes (§21.6). This section does not claim escrow optionality is free.

## 9.6 Honest limits this section states

- **Physical custody cannot be made trustless.** No settlement construction changes this; escrow
  moves money, not goods, and the goods leg's custody lifecycle lives in WRAP (`github.com/vul-os/wrap`; §8.4, §18.4, C2).
- **Non-custodial programmatic escrow deadlocks on genuine disputes.** Multi-sig or
  hashlock+timelock removes the custodian but has no move available when neither party will act —
  the exact case escrow was wanted for. On a `NonCustodialFinal` rail (§9.3) the only honest
  options are a timeout that defaults to one party (a policy choice favouring whoever it defaults
  to, not a neutral mechanism) or an indefinite lock. There is no third option, and this section
  does not pretend one exists (§18.5, §21.11). The choice MUST be **disclosed before the trade**,
  because a buyer who learns at dispute time that no one can release the funds was mis-sold the
  arrangement (§18.5).
- **Unescrowed trade MUST remain possible**, or scope mismatches would exclude underserved regions
  (§9.4.2, §9.5a).
- **The escrow operator is structurally permanent.** Unlike DMTAP's self-extinguishing operator
  class, holding money for strangers is licensed and does not decay (§0.4.3, §21.11.6). What is
  preserved is only that the class is one, permissionless, competing, per-order, replaceable, and
  never holding identity keys (§9.5). TRACT is structurally less pure than DMTAP here, and says so.
- **The whole of trust, dispute, tax and legal returned nothing verified** across the grounding
  passes (§21.1, §21.10, §21.11). This section is design reasoning checked for internal consistency;
  it MUST NOT be read as evidenced, and no part of it may be cited from §21 as *support* (§21.9, C6).

## 9.7 Open

- **Whether a partial release (split ruling) needs protocol representation or is an operator
  concern.** §18.5 already names a `split` escrow state, but §16 carries no object expressing the
  split — no proportions, no per-party amounts, no ruling that produced it. So the question is not
  whether split exists in the state machine (it does) but whether the **wire** must carry a signed
  split object, or whether the split is settled entirely off-protocol by the operator with only the
  resulting `PaymentAttestation`s (§9.4.1) observable. This is the same §16 gap recorded in §9.4.3
  and is left **PROVISIONAL — pending decision**; resolving it toward on-wire representation is a
  §16 MAJOR change.
