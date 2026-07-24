# Governance

KOTVA is a **family of open protocol specifications** — a shared substrate (Identity, MOTE,
Transport, PUB, SYNC, infrastructure roles), the [coordinator contract](coordinator/CONTRACT.md)
that fences centralisation, a small set of primitives, and the profiles built on top of them:
**DMTAP-mail** (mail, messaging, and the legacy gateway — the first and reference profile, §0–§27
at the repository root) plus **TRACT** (commerce, [`profiles/tract/`](profiles/tract/)) and
**WRAP** (work, [`profiles/wrap/`](profiles/wrap/)), with more to follow. Every layer is intended
to be implemented by anyone, with independent reference implementations per profile (Envoir for
DMTAP-mail; Soko for TRACT). There is no control plane and no hosted component any implementation
must speak to; the only roles needing a resource not everyone can get are the two disclosed
exception classes [`coordinator/CONTRACT.md`](coordinator/CONTRACT.md) §2.3 names — a scarce
network-reachability class (the legacy gateway, §0.2.3/§7.1a, and the reachability-adapter) and a
regulatory-licensing class (custodial escrow) — and nothing requires you to use one. This document
states who decides, how each layer of the family changes, and the security gates that govern a
production deployment.

Where this file and a more specific one disagree, **the more specific document governs**: DMTAP-mail's
own **§10** (versioning, conformance, governance) and **§12.8** (operational & security procedures)
for that profile, [`coordinator/CONTRACT.md`](coordinator/CONTRACT.md) for the coordinator
contract (amendment process below), and a profile's own `GOVERNANCE.md` where one exists (e.g.
[`profiles/tract/GOVERNANCE.md`](profiles/tract/GOVERNANCE.md)). This file is the family-wide
default that fills the gaps between them, not a document that overrides them.

## The specification is authoritative

Independent implementations MUST be buildable from each governing document's text alone. Reference
implementations — the Rust `node/` (DMTAP-mail, gateway mode included, §0.2) and, per profile,
implementations such as [Soko](https://github.com/vul-os/soko) for TRACT — are **proofs and sets
of libraries, not normative**: where a reference and its governing document disagree, **the
document wins** (or the discrepancy is filed as a bug). A "KOTVA-compatible" claim (or a
profile-specific one, e.g. "DMTAP-compatible") means **passes the conformance suite**
(`conformance/`), not "resembles a reference" (DMTAP-mail §10.3, §10.4).

## Standards track & licensing

- **Standards track.** The intent is to pursue an **IETF Internet-Draft** for the wire protocol
  and object formats — starting with DMTAP-mail, the first profile far enough along — aiming for
  RFC status; neutral governance is what lets competitors adopt without fearing capture (§10.5).
- **Licensing.** The **specification** (this repository — every substrate, primitive, coordinator,
  and profile document in it) is licensed **CC BY 4.0**; each **reference implementation** (a
  separate repository) is dual-licensed **MIT OR Apache-2.0**. Both © VulOS. Everything a user
  touches and everything trust depends on is open (§10.5, §12.4).
- **No control plane, nothing sold.** There is no hosted component any implementation must talk
  to, no licence server, and no commercial layer; third parties run a coordinator role because
  they want the network to exist (§12.4, [`coordinator/CONTRACT.md`](coordinator/CONTRACT.md) §1).
  The **inviolable rule** (§12.3) lists what can never be gated or charged for: privacy,
  cryptography, metadata privacy, recovery, **native node-to-node delivery** (no operator is on
  that path), and **access to your own keys, mailbox, and data**. A self-hosting user with no
  legacy correspondents pays nobody, ever, and loses no capability. This bright line binds every
  profile, not only DMTAP-mail — non-negotiable governance, not a product decision (§12.3, §12.5).

## Changing the specification

- **Backwards-compatible evolution** happens through **capability negotiation** and each
  registry's own extension procedure (DMTAP-mail §10.2, §21.25 for its wire registries): new
  suites, message kinds, KT log-types, and capability tokens are added dual-stack and negotiated
  per-peer — **no flag day**. New **coordinator kinds** follow the analogous, family-wide
  procedure below. Mechanisms are retired the same way (§12.8.5): announce the successor, let the
  suite high-water-mark / capability version make the upgrade stick, then retire the old one with
  an explicit owner-authorised action.
- **Structural revisions** to a wire-bearing document increment **both** its domain-separation-tag
  prefix and, where one exists, a DNS discriminator anchor, in lockstep — DMTAP-mail's
  `DMTAP-v0/…` → `DMTAP-v1/…` plus `v=dmtap<N>` is the worked instance (§10.1); the coordinator
  contract's own instance is below.
- **Errata and defects** are filed against the governing document and corrected under its own
  authoritative-spec rule (DMTAP-mail §10.4). Security-relevant changes follow the disclosure
  process below and the audit gate.
- **New fail-closed / downgrade rules** added anywhere in the family MUST be mirrored into the
  relevant document's own auditable set (DMTAP-mail's is §10.7), so the fail-closed posture stays
  checkable as a whole, per document.

## Amending the coordinator contract

[`coordinator/CONTRACT.md`](coordinator/CONTRACT.md) is the mechanism that does the family's
anti-centralisation work: its four conformance clauses (§2) and its **§5 canonical coordinator-kind
list** are what stop "some centralisation, done safely" from drifting into ungoverned gatekeeping.
It carries its own `Status: draft… normative once ratified` line — this section is the process that
line refers to.

- **Who may propose.** Any implementer or maintainer, by a pull request against CONTRACT.md (a new
  §5 row, or an edit to §1–§4/§6) carrying a written rationale.
- **The bar a proposal must clear.** A **new coordinator kind** MUST show (a) which of §1's four
  jobs — the ones the peer substrate cannot do reciprocally — it fills, and (b) how it satisfies
  all four conformance clauses (§2.1–§2.4), or that it explicitly claims membership in one of
  §2.3's two disclosed self-host exception classes rather than silently adding a third. A change to
  an **existing clause** (§2–§4, §6) or the **removal** of a kind already relied upon carries the
  higher, structural bar below.
- **Review.** Pending IETF adoption of the standards track (DMTAP-mail §10.5), review is performed
  by the specification's maintainers acting as a **Designated-Expert** function — the same interim
  arrangement DMTAP-mail's own wire registries use (§21.25 item 5) — against the criteria above,
  with a minimum **14-day public comment window** on the open pull request before it may merge.
- **Ratification.** A proposal is *ratified* — moves from prose proposal to the normative table —
  when it merges with maintainer sign-off after the comment window **and**, where it adds or
  changes wire-visible behaviour, its CDDL/DS-tag lands in the same change
  (18-wire-format.md §18.8a). A rule does not go live in prose alone; the bytes and the words
  ratify together, so "normative" stays byte-checkable rather than a claim taken on faith.
- **Versioning.** Every ratified CONTRACT change is logged in `CHANGELOG.md` under a "Coordinator
  Contract" entry. **Adding a kind is additive**: `CoordinatorDescriptor.kind` is a free-form
  string and an unrecognized value is already fail-closed by construction — a client treats it as an
  undeclared coordinator (18-wire-format.md §18.8a.1) — so a new row needs no DS-tag bump and no
  flag day. A **structural** revision — redefining or removing a clause, or retiring a kind already
  relied upon — bumps the coordinator-layer DS-tag family (`DMTAP-COORD-v0/…` → `…v1/…`, mirroring
  DMTAP-mail §10.1) across every wire object it touches.
- **Backward compatibility.** A structural revision follows the same discipline as mechanism
  retirement elsewhere in the family (DMTAP-mail §12.8.5): announce the successor, allow a
  transition window, retire only with an explicit maintainer-authorised action. A ratified kind is
  **deprecated, never silently redefined** — the append-only discipline the wire registries already
  use (§21.25 item 6) — and an already-conformant coordinator MUST NOT be retroactively failed
  without notice.

This is deliberately a light process, not a foundation: while the family is pre-standardisation,
"ratified" means "the maintainers, applying the criteria above in the open, said so" — the same
honest, minimal arrangement DMTAP-mail's own registries already run on (§21.25 item 5) — and it is
expected to hand off to a heavier, IETF-run process only if and when the standards track
(DMTAP-mail §10.5) formally begins.

## Security governance

- **Coordinated vulnerability disclosure** is governed by [`SECURITY.md`](SECURITY.md) and
  DMTAP-mail §12.8.1: private reporting to `security@envoir.org`, a **90-day** default embargo,
  CVE assignment, and a research **safe harbour**. This is the family's default disclosure channel
  today; a profile stands up its own only if it later needs one.
- **Independent external audit is a pre-deployment gate** (DMTAP-mail §12.8.4). A qualified third
  party MUST review the cryptography/protocol and the relevant reference implementation **before
  any production deployment**, and again on any **major crypto or wire change** — a new/retired
  suite, a changed signing preimage, a mixnet or deniable-handshake change
  ([docs/research/mixnet.md](docs/research/mixnet.md)), or a **structural** coordinator-contract
  revision (above). This gate is **paid for by the project** and is **distinct from** the
  post-deployment bug bounty.
- **Bug bounty is post-deployment only** — there is no live target to attack before launch
  (`SECURITY.md`, §12.8.1).
- **Honest limits are governance, not marketing.** Every document states what it **cannot** do and
  the disclosed residual of every security property it claims (DMTAP-mail §6.6, §6.9;
  CONTRACT.md §3.4's blindness residual and §6's funding residual are the coordinator-layer
  instances) rather than overclaim. Presenting a documented residual as solved — e.g. "anonymous"
  against an active adversary, v0 KT as equivocation-proof, or a `declared`-level blindness claim
  as verified — is a governance violation, not a nuance.

## Roles

- **Maintainers** steward the specifications and reference implementations, triage disclosures,
  run the audit gate, arbitrate spec-vs-reference discrepancies, and perform the coordinator-
  contract Designated-Expert function above.
- **Implementers** are first-class: conformance is defined by each public suite, so any
  independent implementation that passes it is compatible without maintainer permission
  (DMTAP-mail §10.3).
- **Role-takers** run the infrastructure roles that need no scarce resource and no permission
  (DMTAP-mail §0.2.2: relay, mix, buffer, KT log, rendezvous) and, at the family layer, any
  [`coordinator/CONTRACT.md`](coordinator/CONTRACT.md) §5 kind that clears the ordinary self-host
  bar (§2.3). Every kind in that table is an **accountable, attested identity** with a defined
  joining/reputation/leaving lifecycle (DMTAP-mail §12.8.6): never anonymous infrastructure, and
  never able to gate a privacy/crypto feature (DMTAP-mail §12.3). Most are never bonded or slashed
  ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md) for the mix fleet's operator-diversity
  case) — the disclosed exceptions are the staked kinds CONTRACT §6 itself names (`arbiter`,
  `oracle`, and bonded `custodial-escrow`), verified on-rail per §6, never taken on faith.
