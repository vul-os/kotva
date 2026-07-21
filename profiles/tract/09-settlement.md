# 9. Settlement

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 9.1 Scope

The payment seam, rail classes, escrow, and the gateway's money role.

## 9.2 The seam names no provider

TRACT specifies where money crosses the boundary and what must be verified. It specifies no rail,
currency, token or ledger. Naming a provider in a protocol exports that provider's jurisdiction
and licensing to every implementor.

## 9.3 Rail class is part of the type

`CustodialReversible` (chargebacks exist; the card network is already the adjudicator) versus
`NonCustodialFinal` (nobody custodies, nothing reverses). An implementation must not substitute
one class for the other without an explicit party decision, because the buyer's recourse differs.

## 9.4 What this section will specify

- The payment-attestation object: payer, payee, order, amount, rail class, external settlement
  reference. **The protocol carries attestations, never funds.**
- Escrow lifecycle: fund → hold → release / refund / split, each a signed object.
- `EscrowScope`: buyer countries, seller countries, supply countries, currencies, rail classes,
  value ceiling, excluded categories, claimed authorisations.
- **Fail-closed scope intersection** at checkout, and the requirement that an empty intersection is
  disclosed rather than silently downgraded.

## 9.5 Escrow is the operator class

It requires legal standing, a payment-provider relationship, a float, and licensing — none of it
derivable from a keypair. What bounds it: permissionless entry, competition, per-order choice by
both parties, no access to identity keys, and **every ruling published as a signed object**, so an
operator that rules unfairly accumulates a permanent verifiable record.

## 9.6 Honest limits this section must state

- Physical custody cannot be made trustless.
- Non-custodial programmatic escrow (multi-sig, hashlock+timelock) removes the custodian but
  **deadlocks on genuine disputes** — the exact case it was wanted for.
- Unescrowed trade must remain possible, or scope mismatches would exclude underserved regions.

## 9.7 Open

- Whether partial release (split rulings) needs protocol representation or is an operator concern.
