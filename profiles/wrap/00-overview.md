# 1. Overview

## 1.1. The one idea

Every platform that coordinates work — delivery apps, gig marketplaces, trade
directories, field-service dispatchers — solves the same problem the same way:
put one company in the middle, let it hold every identity, every job record and
every reputation, and let it take a cut for the privilege. The coordination is
not the hard part. The *centralization* is not a technical necessity; it is a
business model that arrived wearing the costume of an architecture.

Look at what actually forces a central operator, and it comes down to one
thing: **somebody must decide who gets the job.** Nearly every platform runs
first-come-claim — the job is broadcast, workers race, and the first tap wins.
A race needs an arbiter. An arbiter is a central server. Everything else —
identity, reputation, payments, history — gets absorbed into that server
because it is already there.

WRAP removes the race. The party who wants the work done decides who does it:

> **The issuer assigns.** The only contended decision in the protocol is
> resolved by the party who is already its natural authority — the one who
> issued the work. Whoever cooked the food decides who carries it.

That single choice eliminates consensus, leader election, and distributed
locking from the protocol. What remains is a set of signed objects that merge
deterministically, which any two participants can exchange over any transport,
in any order, with arbitrary delay, and still converge.

## 1.2. WRAP adopts the DMTAP substrate

WRAP is **not a new stack.** Identity, signed content-addressed objects,
deterministic encoding, the CRDT merge algebra, key-addressed discovery, offline
transport — none of these are about *work*, and WRAP invents none of them. It
adopts four of the [DMTAP substrate](https://github.com/vul-os/dmtap)'s six capabilities
à la carte, under that directory's adoption rules
([`substrate/README.md`](https://github.com/vul-os/dmtap/blob/main/substrate/README.md)
§3): a product that implements a capability's function **MUST** speak that
capability's bytes, never a parallel format. Concretely WRAP adopts:

- **Identity** (①) — a Principal *is* a substrate `IK`, with the substrate's
  `DeviceCert`, key-name, rotation, and recovery (§2);
- **Sync** (⑤) — coordination state (`Offer`/`Bid`/`Assignment`/`Progress`) is
  substrate Sync ops, merged by the substrate's CRDT algebra and HLC (§7);
- **Feeds & Blobs** (④) — a `WorkOrder` is an immutable content object and an
  `Attestation` is an author-feed entry, so reputation is portable and
  HTTPS-servable (§7, §9);
- **Infrastructure Roles** (⑥) — pools are discovered and work distributed
  through the substrate's announce/resolve, relay, and mailbox (§8).

An earlier draft of WRAP defined its own encoding, its own signing envelope, its
own CRDT, and its own transport bindings. Every one of those was the substrate's,
re-derived — the "parallel format" rule 2 forbids — and all four are removed. What
is **genuinely WRAP**, and all this document now specifies, is the work spine on
top:

- the **object model** — `WorkOrder`, `Offer`, `Bid`, `Assignment`, `Progress`,
  `Attestation` (§3) — and the field registry over it (§4);
- the **issuer-assigns** rule and the authorship/admission constraints that make
  the assignment register single-writer (§5);
- the **lifecycle** state machine (§6) and the object → substrate-primitive
  mapping (§7);
- the **pool authority model** — a pool distributes but never assigns (§8);
- the **trust** model, honest about Sybil resistance (§9);
- **fulfilment** — how completion is proven to a party who was not present (§10);
- **profiles** — delivery, trades, and how to define others (§12).

## 1.3. What WRAP is not

- **Not a payment system.** A work order carries compensation *terms* as data.
  Settlement happens out of band, by whatever means the parties already use.
  There is no escrow, no currency, and no blockchain anywhere in this document.
- **Not a mesh.** Broadcasting work orders to every participant would publish
  customer addresses and commercial volumes to strangers, invite spam, and
  scale badly for something whose scope is usually a few kilometres. WRAP is
  point-to-point plus pools (§8).
- **Not a substrate.** WRAP does not define identity, encoding, merge, discovery,
  or transport — it adopts the DMTAP substrate for all of them (§1.2). This
  document is only the work-coordination spine on top.
- **Not a commerce protocol.** WRAP coordinates *work* (request → bid → assign);
  it does not model catalogues, carts, or orders. Its sibling
  [TRACT](https://github.com/vul-os/tract) is the commerce spec on the same
  substrate, and it references WRAP for the delivery/dispatch leg of an order
  rather than re-specifying work coordination.
- **Not trustless.** Open participation without a gatekeeper admits Sybils.
  WRAP does not claim to have solved this; it makes the trust source explicit
  and replaceable (§9).
- **Not a governance framework.** WRAP says nothing about how a pool decides
  its rules, prices, or membership. That is deliberate and it is the most
  important thing WRAP leaves to its users.
- **Not a scheduler or optimiser.** Route planning, batching, and matching
  heuristics are implementation concerns, not wire concerns.
- **Not anonymous.** Participants are identified by public key to their
  counterparties. WRAP protects *integrity* and *ownership of history*, not
  metadata privacy.

## 1.4. Design constraints

These constraints produced the design and explain most of its choices.

**It must work offline.** A courier goes through a tunnel; a plumber works in a
basement; a café's connection fails during the dinner rush. Participants MUST
be able to make progress while partitioned and converge afterwards without
losing work. The substrate's Sync algebra provides exactly this (§7); WRAP adds
no synchronous coordination of its own.

**It must run on modest hardware.** The reference deployment is a low-cost
Android tablet and a phone. This forbids heavyweight consensus and rules out
requiring a always-on server per participant.

**It must be implementable in an afternoon.** A protocol that requires adopting
a novel messaging stack acquires zero independent implementations. WRAP rides the
substrate, which is itself a narrow waist of Ed25519, deterministic CBOR, and an
ordinary HTTP client (SYNC's three endpoints, the Feeds HTTP surface). WRAP adds
only six object kinds and one assignment rule on top.

**The worker's history must be portable.** If reputation lives in a platform's
database, the platform owns the worker. WRAP makes attestations signed objects
held by the parties, so a worker who leaves a pool takes their record with them.

**No privileged nodes.** Some infrastructure is unavoidable for open discovery
(§8.1 is explicit about why). WRAP requires only that any such node be
*replaceable*: anyone may run one, participants may use several, and losing one
costs reachability, never identity, history, or data.

## 1.5. Document status

This is **version 0**. It carries no compatibility guarantee and describes an
implementation in progress. It is written with the rigour of a standards
document because that discipline produces better designs, not because
stability is being claimed.

The conformance vectors (§15) are normative where they disagree with the prose.
This is deliberate: prose cannot tell an implementer that they got a tie-break
wrong, and the parts of comparable protocols that actually achieved
independent implementations are precisely those that shipped vectors.
