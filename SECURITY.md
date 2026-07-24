# Security Policy

DMTAP is a cryptographic protocol: a single flaw can silently defeat a security property for every
user. Reports are handled under **coordinated vulnerability disclosure (CVD)**. This document is the
human-facing summary; the normative version lives in the specification at **§12.8.1** (with the
falsifiable security properties in **§6.9** and the fail-closed invariant set in **§10.7**). Where
this file and the spec disagree, the spec governs (§10.4).

## Reporting a vulnerability

**Do not open a public issue for an unfixed vulnerability.** Instead, use one of:

- **Email:** `security@envoir.org` — encrypt with the project PGP key (fingerprint published
  alongside this file / on the project site; request it at the same address if not yet posted).
- **Private advisory:** the repository's private security-advisory facility (GitHub/GitLab
  "report a vulnerability").

A good report includes:

- the affected specification **§/clause**, or the reference file/function;
- the **security property** you believe is broken — an `SP-n` from §6.9, or a `§10.7` fail-closed
  invariant — and **without invoking its disclosed residual** (a counterexample that only exercises
  a stated residual is expected behaviour, not a defect);
- a reproduction, proof-of-concept, or symbolic-model trace;
- the version / commit / deployment tested.

## What to expect

| Stage | Target |
|-------|--------|
| Acknowledgement | within **3 business days** |
| Initial severity + clause mapping | within **10 business days** |
| Default coordinated embargo | **90 days** from acknowledgement (extendable by mutual agreement for a protocol/wire fix that independent implementations must ship in lockstep) |
| CVE + advisory | requested for confirmed vulnerabilities; published at disclosure |

The embargo ends at the **earlier** of the fix's public release or the agreed date. A confirmed
**spec-level** defect is corrected per §10.4 (the spec is authoritative; the reference implementation
is a proof, not the authority). A fix that changes cryptography or the wire format re-opens the
**pre-deployment audit gate** (§12.8.4).

## Safe harbour

Good-faith security research is welcome and **will not be pursued by the project** when it:

- tests only against **your own** identity, node, or deployment;
- does **not** access, modify, or exfiltrate other users' data;
- does **not** degrade or disrupt service for others;
- **honours the embargo** above and reports through the private channel.

This is a research safe-harbour statement, not legal advice.

## Bug bounty — post-deployment only

There is **no monetary bug bounty before launch.** There is no live production target to attack
pre-deployment, and a bounty against a spec and reference implementation under active hardening would
mis-price the work. A bounty is established **only once a production deployment exists**, and is
**distinct from** the independent external audit that **must precede** production and any major
crypto/wire change (§12.8.4). Until a bounty exists, coordinated disclosure above — under the safe
harbour — is the way to contribute security findings.

## Scope

- **In scope:** the protocol specification (this repository) and the Envoir reference
  implementation (node, gateway, clients, libraries).
- **Out of scope for CVD-as-vulnerability:** the disclosed honest limits (§6.6) and the stated
  residuals of each security property (§6.9) — these are documented boundaries, not defects.
  Reports that *tighten* a residual, or show a residual is worse than disclosed, are in scope.
