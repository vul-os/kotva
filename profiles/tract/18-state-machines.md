# 18. State machines

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 18.1 Scope

The normative state machines, collected so they can be checked against §7's prose rather than
inferred from it.

## 18.2 Machines to be specified

- **Offer**: draft → published → superseded / withdrawn.
- **Order**: draft → placed → accepted / declined / countered → fulfilling → delivered → closed,
  with cancel, return and refund paths.
- **Consignment**: created → accepted → in-custody → handed-off → delivered / lost.
- **Escrow**: funded → held → released / refunded / split, with the timeout edges.

## 18.3 What each transition must carry

Which party signs, what evidence the transition produces, the timeout, and the behaviour when a
timeout expires with no counterparty response. **Every timeout must have a defined expiry
behaviour** — an undefined one is where funds and goods go to die.
