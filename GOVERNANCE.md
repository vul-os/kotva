# Governance

DMTAP is an **open protocol specification** intended to be implemented by anyone, with an
independent reference implementation (Envoir). There is no control plane and no hosted component
any implementation must speak to; the only role needing a resource not everyone can get is the
legacy gateway (§0.2.3, §7.1a), and nothing requires you to use one. This document states who
decides, how the specification changes, and the security gates that govern a production deployment. The normative sources are the specification's
**§10** (versioning, conformance, governance) and **§12.8** (operational & security procedures);
where this file and the spec disagree, the spec governs (**§10.4**).

## The specification is authoritative

Independent implementations MUST be buildable from the specification text alone. The Rust reference
in `node/` (the one binary, gateway mode included, §0.2) is a **proof and a set of libraries, not normative** — where the reference
and the spec disagree, **the spec wins** (or the discrepancy is filed as a bug). "DMTAP-compatible"
means **passes the conformance suite** (`conformance/`), not "resembles the reference" (§10.3, §10.4).

## Standards track & licensing

- **Standards track.** The intent is to pursue an **IETF Internet-Draft** for the wire protocol and
  object formats, aiming for RFC status — neutral governance is what lets competitors adopt without
  fearing capture (§10.5).
- **Licensing.** The **specification** (this repository) is licensed **CC BY 4.0**; the **reference
  implementation** (separate repository) is dual-licensed **MIT OR Apache-2.0**. Both © VulOS.
  Everything a user touches and everything trust depends on is open (§10.5, §12.4).
- **No control plane, nothing sold.** There is no hosted component any implementation must talk to,
  no licence server, and no commercial layer; third parties run the gateway role because they want
  the network to exist (§12.4). The **inviolable rule** (§12.3) lists what can never be gated or
  charged for: privacy, cryptography, metadata privacy, recovery, **native node-to-node delivery**
  (no operator is on that path), and **access to your own keys, mailbox, and data**. A
  self-hosting user with no legacy correspondents pays nobody, ever, and loses no capability. This
  bright line is non-negotiable governance, not a product decision (§12.3, §12.5).

## Changing the specification

- **Backwards-compatible evolution** happens through **capability negotiation** (§10.2) and the
  registries' extension procedure (§21.25): new suites, message kinds, KT log-types, and capability
  tokens are added dual-stack and negotiated per-peer — **no flag day**. Mechanisms are retired the
  same way (§12.8.5): announce the successor, let the suite high-water-mark and monotonic capability
  version make the upgrade stick, then retire the old one with an explicit owner-authorized action.
- **Structural revisions** increment **both** the domain-separation tag (`DMTAP-v0/…` → `DMTAP-v1/…`)
  and the DNS `v=dmtap<N>` anchor in lockstep, giving an unambiguous discriminator (§10.1).
- **Errata and defects** are filed against the spec and corrected under §10.4. Security-relevant
  changes follow the disclosure process below and the audit gate.
- **New fail-closed / downgrade rules** added anywhere in the spec MUST be mirrored into the
  **§10.7** auditable set, so the fail-closed posture stays checkable as a whole.

## Security governance

- **Coordinated vulnerability disclosure** is governed by [`SECURITY.md`](SECURITY.md) and §12.8.1:
  private reporting to `security@envoir.org`, a **90-day** default embargo, CVE assignment, and a
  research **safe harbour**.
- **Independent external audit is a pre-deployment gate (§12.8.4).** A qualified third party MUST
  review the cryptography/protocol and the reference implementation **before any production
  deployment**, and again on any **major crypto or wire change** (a new/retired suite, a changed
  signing preimage, a mixnet or deniable-handshake change). This gate is **paid for by the project**
  and is **distinct from** the post-deployment bug bounty.
- **Bug bounty is post-deployment only** — there is no live target to attack before launch
  (`SECURITY.md`, §12.8.1).
- **Honest limits are governance, not marketing.** The project states what the protocol **cannot**
  do (§6.6) and the disclosed residual of every security property (§6.9) rather than overclaim.
  Presenting a documented residual as solved — e.g. "anonymous" against an active adversary, or
  v0 KT as equivocation-proof — is a governance violation, not a nuance.

## Roles

- **Maintainers** steward the specification and reference implementation, triage disclosures, run
  the audit gate, and arbitrate spec-vs-reference discrepancies (§10.4).
- **Implementers** are first-class: conformance is defined by the public suite, so any independent
  implementation that passes it is DMTAP-compatible without maintainer permission (§10.3).
- **Role-takers** run the infrastructure roles (§0.2.2) — relay, mix, buffer, KT log, rendezvous —
  which need no scarce resource and no permission. The two roles that make claims about *who they
  are* (gateway, mix) plus postage/token issuers are **accountable, attested identities** with a
  defined joining/reputation/leaving lifecycle (§12.8.6): never anonymous infrastructure, never
  bonded or slashed (§4.4.8), and never able to gate a privacy/crypto feature (§12.3).
