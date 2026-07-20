# 9. Trust

## 9.1. WRAP is not trustless

Anyone can generate an Ed25519 keypair in microseconds. An open pool with no
gatekeeper will therefore accumulate fake performers as fast as someone cares
to script them, and a reputation system built on unweighted attestations can be
inflated by a Sybil signing praise for itself.

No protocol solves this. Proof-of-work prices out the poor before it prices out
the determined; stake-based schemes require capital and exclude exactly the
workers this protocol exists to serve; and "web of trust" reduces, in practice,
to somebody vouching. WRAP does not claim a solution. It makes the trust source
**explicit and replaceable** instead of pretending it is absent.

## 9.2. Curated pools

The RECOMMENDED deployment is a **curated pool**: membership is vouched for by
the pool operator, who is accountable to its members.

This is federated trust, not trustlessness, and it maps onto structures that
already exist and already do this work — a courier co-operative, a union local,
a trade association that verifies plumbing licences, a municipality maintaining
a register, a business dispatching its own staff. These bodies already perform
identity verification for their own reasons; WRAP lets them express it without
also owning the workers' data.

The protocol keeps the operator honest in one specific way: because a pool
holds no authority over work, no ability to forge signatures, and no exclusive
copy of any record (§8.3), a pool that abuses its position can be left. The
worker walks out with every attestation intact. That is the entire safeguard,
and it is a real one — it converts "trust the platform" into "choose a
gatekeeper you can fire".

## 9.3. Weighting attestations

An `Attestation` (§3.8) is meaningful only in the light of who signed it.
Implementations MUST NOT compute a reputation as an unweighted count.

At minimum:

- A **self-attestation** — where `author == subject` — MUST be excluded.
- Attestations from Principals with no independent history SHOULD be weighted
  near zero, since they are the cheapest thing an attacker can manufacture.
- Attestations SHOULD be weighted by the attestor's own standing in the pool
  evaluating them, and pools SHOULD publish how they weight.
- An attestation without a corresponding `Assignment` naming both parties on
  the same work order MUST be ignored. You cannot review work you were never
  party to.

## 9.4. Reputation is inverted

The naive design — each performer publishes their own reputation feed — is
broken, and it is worth being explicit about why, because it is the design most
implementers reach for first. A performer who controls their feed will omit the
jobs that went badly. A feed that can be silently truncated carries no
information.

WRAP inverts it:

> **Issuers publish outcomes about performers. Performers publish outcomes
> about issuers. Neither publishes about itself.**

A performer's reputation is therefore an *aggregate over attestations signed by
counterparties*, which the performer cannot suppress because they do not hold
the signing keys and do not control the copies. The performer carries pointers
to their attestations for convenience; the attestations' authority comes from
their authors.

Symmetry matters here and is not decoration. Issuers can be bad
counterparties — cancelling after a courier has driven across town, disputing
completed work, setting impossible windows. A protocol that lets only one side
rate the other reproduces the power asymmetry it was meant to remove.

## 9.5. Append-only publication

Where attestations are published as a feed, that feed SHOULD be append-only and
hash-chained: each entry carries a sequence number and the hash of its
predecessor, so an omission or reordering is detectable by any reader who has
seen an earlier head.

Readers SHOULD retain the highest sequence number seen per publisher and reject
a later head with a lower sequence, which detects rollback. Without this, a
publisher can serve different histories to different readers.

This is the one place WRAP recommends a specific external mechanism: the
structure is that of an RFC 6962 style log, and the DMTAP-PUB feed object
(§11.2) is a conformant instance of it that requires no mesh and runs over
plain HTTPS. Implementations MAY use any equivalent construction.

## 9.6. Disputes

WRAP records disputes; it does not resolve them. An `Attestation` with
`outcome = 3` marks a contested outcome, and both parties' attestations remain
in the record permanently, each signed by its author.

Resolution is a social and legal matter for the pool, the parties, or a court.
A protocol that adjudicated disputes would be a protocol that could be captured
by whoever controls the adjudication, which is the thing being removed.
