<div align="center">

# WRAP

### Work Request & Assignment Protocol

**Coordinating work without a middleman.** One open protocol for issuing,
offering, assigning, tracking and proving **work** — couriers, plumbers, field
service, mutual aid — between independent parties, with no central operator, no
platform account, and no cut taken.

*Your key is your identity. Your work history belongs to you. Any pool you join, you can leave.*

</div>

---

## The one idea

Every platform that coordinates work does it the same way: put one company in
the middle, let it hold every identity, every job record and every reputation,
and let it take a cut. The coordination is not the hard part. The
centralization is a business model wearing the costume of an architecture.

Look at what actually *forces* a central operator and it comes down to one
thing: **somebody must decide who gets the job.** Almost every platform runs
first-come-claim — broadcast the job, workers race, first tap wins. A race
needs an arbiter. An arbiter is a central server. And once it exists,
everything else gets absorbed into it.

WRAP removes the race:

```mermaid
flowchart LR
  I["<b>Issuer</b><br/><i>wants work done</i>"]
  P["<b>Pool</b><br/><i>distribution only</i><br/>no authority"]
  W["<b>Performers</b><br/><i>bid</i>"]
  A["<b>Assignment</b><br/><i>issuer decides</i>"]
  I -->|Offer| P --> W -->|Bid| I
  I --> A
```

> **The issuer assigns.** The only contended decision is made by the party who
> is already its natural authority — the one who wants the work done. Whoever
> cooked the food decides who carries it.

That single choice removes consensus, leader election and distributed locking
from the protocol. What's left is a set of signed objects that merge
deterministically — exchangeable over any transport, in any order, with
arbitrary delay, and they still converge.

## Properties

- **Offline-first.** A courier in a tunnel, a plumber in a basement, a café
  whose connection dies mid-service. Everyone keeps working; state converges
  afterwards. Nothing blocks.
- **No privileged nodes.** Some infrastructure is unavoidable for open
  discovery — the spec is explicit about why. WRAP requires only that it be
  *replaceable*: anyone can run a pool, you can join several, and losing one
  costs reachability, never identity, history or data.
- **Portable reputation.** Attestations are signed by counterparties and held
  by both sides. Leave a pool and your record comes with you.
- **Domain-neutral.** Delivery and skilled trades ship as profiles in v0.
  Field service, mutual aid, municipal reporting, medical courier and remote
  freelance work are all expressible — geography is an optional field.
- **Implementable in an afternoon.** Ed25519, CBOR, HTTP. Nothing else is
  required.

## What it deliberately isn't

- **Not a payment system.** Work orders carry compensation *terms*; settlement
  happens out of band. No escrow, no currency, no blockchain.
- **Not a mesh.** Broadcasting work orders would publish customer addresses to
  strangers and invite spam. Point-to-point plus pools.
- **Not trustless.** Open participation admits Sybils. WRAP makes the trust
  source explicit and replaceable — curated pools — rather than pretending it
  isn't there.
- **Not anonymous.** It protects integrity and ownership of history, not
  metadata privacy.
- **Not a governance framework.** How a pool decides who belongs is for the
  people affected, not for a protocol author.

## The spec

| § | Document | |
|---|---|---|
| 1 | [Overview](00-overview.md) | The idea, scope, constraints |
| 2 | [Identity](01-identity.md) | Keypairs, roles, key names |
| 3 | [Objects](02-objects.md) | WorkOrder, Offer, Bid, Assignment, Progress, Attestation |
| 4 | [Wire format](03-wire-format.md) | CBOR, versioning, forbidden keys |
| 5 | [Signing](04-signing.md) | Sign the object, not the frame |
| 6 | [Lifecycle](05-lifecycle.md) | States, transitions, computed expiry |
| 7 | [Merge](06-merge.md) | CRDT algebra, HLC, the tie-break |
| 8 | [Pools](07-pools.md) | Discovery without privileged nodes |
| 9 | [Trust](08-trust.md) | Sybils, curated pools, inverted reputation |
| 10 | [Fulfilment](09-fulfilment.md) | Handoff codes and honest weak proofs |
| 11 | [Transport](10-transport.md) | HTTP binding, DMTAP binding, USB sticks |
| 12 | [Profiles](11-profiles.md) | Delivery, trades, and other domains |
| 13 | [Errors](12-errors.md) | Codes, and what is deliberately not an error |
| 14 | [Security](13-security.md) | Including what it does *not* protect |
| 15 | [Conformance](14-conformance.md) | Vectors are normative |
| 16 | [References](15-references.md) | Prior art and debts |

Build the PDF:

```bash
cd build && npm install && npm run build   # -> wrap.pdf
```

## Status

**Version 0. No compatibility guarantee.** This describes an implementation in
progress. It is written with the rigour of a standards document because that
discipline produces better designs — not because stability is being claimed.

It is not an RFC and does not seek to be one yet. A specification with a single
implementation is a description, not a standard; promoting it is a question for
the day a second implementer appears.

## License

Specification: [CC BY 4.0](LICENSE.md) — implement, quote and build on it
freely, with attribution.

<div align="center">
<sub>Part of <strong><a href="https://vulos.org">VulOS</a></strong> — the open, self-hostable web OS &amp; app suite.</sub><br>
<sub><em>Vulos — rooted in <strong>vula</strong>, the Zulu and Xhosa word for <strong>open</strong>.</em></sub>
</div>
