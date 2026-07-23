# 8. Pools and discovery

## 8.1. There is no discovery without infrastructure

State the uncomfortable thing first: **open discovery cannot be done with zero
infrastructure.** A plumber's phone cannot learn that a homeowner four suburbs
away needs a geyser replaced unless something reachable at a known address told
it so. Every system that claims otherwise has merely hidden the infrastructure
— BitTorrent has bootstrap nodes and trackers, Nostr has relays, Matrix has
homeservers, ActivityPub has instances, IPFS has bootstrap peers.

WRAP therefore does not pursue "no nodes". It pursues **no privileged nodes** —
which is precisely the substrate's Infrastructure-Roles stance
([`ROLES.md`](https://github.com/vul-os/dmtap/blob/main/substrate/ROLES.md) §1, "roles, not node types"):

- anyone MAY run one, with no permission and no registration;
- they are interchangeable — no special keys, no protocol authority;
- a participant MAY use several simultaneously;
- losing one costs *reachability*, never identity, history, or data.

This is a weaker claim than "fully decentralized" and it is the honest one. It
is also the property that actually protects participants, because it is what
makes an operator replaceable when they start behaving badly. A WRAP **pool is a
user of substrate roles**, not a new kind of infrastructure: it is discovered
through the substrate's key-addressed announce/resolve (ROLES §2) and it MAY lean
on a substrate mailbox (ROLES §5) to hold offers for an offline performer and a
cache/pin (ROLES §6) to serve work orders and attestation feeds. WRAP invents no
transport of its own for any of this.

## 8.2. Three paths

**Direct (no infrastructure).** The issuer knows the performer's key and offers
straight to it — a shop and its own drivers, a homeowner and their regular
plumber, a landlord and their contractor. This is `Offer.mode = 0` with `pool`
set to the issuer's own key. It requires no third party at all, and it covers
the large majority of real working relationships.

**Pool (the open market).** A pool is a Principal that accepts offers and
distributes them to a membership. A co-op, a union local, a trade association,
a neighbourhood group, a municipality, or a business running a dispatch desk
may each operate one. A performer MAY belong to many. A pool announces its
location under its key through the substrate's announce/resolve
([`ROLES.md`](https://github.com/vul-os/dmtap/blob/main/substrate/ROLES.md) §2), so a performer resolves a pool by
its `IK`, never through a WRAP-specific directory.

**Key-addressed discovery is the substrate's, not a future WRAP mechanism.** An
earlier draft reserved a "DHT, optional, future" for pool discovery; that is
removed. The substrate already defines exactly that — key-addressed
announce/resolve over a Kademlia DHT (ROLES §2), degrading to an HTTPS
announce/resolve where no mesh is present. WRAP discovers pools through it and
specifies no discovery mechanism of its own, which is the rule-2 way to avoid a
spec drifting from an unused invention.

## 8.3. What a pool does and does not do

A pool holds **no authority over the work**. It cannot assign, cannot complete,
cannot alter a work order, and cannot forge a bid — every object is signed by
its author and a pool has no way to produce a signature it does not hold.

A pool provides exactly two services:

1. **Distribution** — performers learn that a work order exists.
2. **Membership** — the pool decides who receives offers and who may bid (§9).

A pool is therefore a *rendezvous point, not a database of record*. If a pool
vanishes tomorrow, every participant retains every work order, assignment,
progress event and attestation they hold. They lose the ability to find new
counterparties through that pool, and nothing else. Participants SHOULD belong
to more than one pool for precisely this reason.

## 8.4. Membership

Pool membership is expressed as an `Offer` reaching a performer and a `Bid`
being accepted from one. WRAP does not define a membership object, an
application flow, or an eviction mechanism in v0.

This is a deliberate scope boundary, not an oversight. How a pool decides who
belongs — open registration, vouching, licensing checks, union membership,
paying dues, geographic residency — is *governance*, and governance is the part
of a labour platform that most deserves to be decided by the people affected
rather than by a protocol author. WRAP's job is to ensure that whatever a pool
decides, it cannot use that position to seize a worker's identity or history.

## 8.5. Privacy

An `Offer` carries a work order to a pool, and work orders contain customer
addresses and commercial terms. A pool operator therefore sees the work it
distributes. This is the one place a WRAP role is **not** content-blind: a
substrate mailbox or circuit relay (ROLES §4, §5) forwards ciphertext it cannot
read, but a pool matches work to performers and so reads the offer by definition.
Implementations MUST make this visible to users and MUST NOT describe pool
distribution as private.

Two mitigations are available and both are RECOMMENDED where the deployment
allows:

- **Coarse offers.** Publish only what is needed to bid — an area, a window, a
  compensation range — and disclose exact addresses in the `Assignment`, which
  goes only to the assigned performer.
- **Direct offers.** Where the relationship already exists, `mode = 0` bypasses
  pools entirely and no third party sees anything.

Sealed-bid mode (`mode = 2`) additionally hides bids from other bidders until
`closes`; it does not hide them from the pool.

## 8.6. Why not gossip

Broadcasting every work order to every participant would publish customer
addresses and commercial volumes to strangers, create an unbounded spam
surface, and scale poorly for a workload whose natural scope is a few
kilometres and a few hours. A mesh also removes the one place where membership
policy can live, which does not eliminate gatekeeping — it merely relocates it
to whoever writes the client's filtering rules.

Point-to-point plus pools keeps the sensitive data narrow, keeps the trust
decision explicit, and keeps the failure domain small.
