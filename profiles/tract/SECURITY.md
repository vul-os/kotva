# Security

## Reporting

Specification defects with security consequences — an under-specified check, a mechanism that fails
open, a privacy leak implied by a required field — should be reported as an issue on this
repository, or privately to the maintainers if disclosure would put deployments at risk before a
fix lands.

Vulnerabilities in an *implementation* go to that implementation. For Soko, see
<https://github.com/vul-os/soko>.

## What counts as a specification security defect

- **A failure that is not fail-closed.** Every security-relevant failure must be refused or
  surfaced as an explicit choice, never silently degraded (§15.3).
- **A field that leaks more than its purpose requires**, particularly anything that would place
  personal data in the public quadrant, which is irrevocable (§0.5.1).
- **A trust assumption that is not stated.** An unstated assumption is a defect even when the
  mechanism is otherwise sound, because implementers cannot mitigate what they were not told.
- **A downgrade path** — anywhere a party can be moved to a weaker guarantee without both parties
  deciding to. Rail-class substitution (§9.3) and escrow-scope mismatch (§9.4) are the two live
  examples.

## Known structural exposures

These are design consequences, not bugs, and they are documented rather than fixed:

- **Storefront gateways are trusted by browsers.** A shopper with no keypair cannot verify a
  signature. Mitigated by universal re-renderability — any node can produce the same page from the
  same signed objects — which makes dishonest rendering *detectable*, not impossible (§12.5).
- **Escrow operators hold funds.** They never hold identity keys, they are chosen per order, and
  every ruling is published as a signed object — but they are custodians (§9.5).
- **Published objects are irrevocable.** This is why nothing personal may be published (§0.5.1),
  and why reviews are the single bounded exception (§10.4).
- **Reputation is manipulable at a cost.** Purchase attestation raises the price of ballot-stuffing
  and leaves a trail; it does not establish counterparty independence. The achievable Sybil-cost
  floor on this substrate is an open question (§21.8).
- **Discovery may re-centralise.** No protocol rule prevents one index becoming dominant, and this
  happened to the closest deployed relative of this design (§21.3).

## Cryptography

TRACT introduces none. Every primitive is inherited from the DMTAP substrate, which profiles
existing RFCs. A cryptographic construction invented in this document would be a defect.
