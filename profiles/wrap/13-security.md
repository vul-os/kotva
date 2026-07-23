# 14. Security considerations

## 14.1. What WRAP protects

- **Authorship.** Every object is signed; nobody can put words in another
  party's mouth. In particular no performer can assign work to themselves
  (§5.5) and no pool can forge a bid (§8.3).
- **Integrity.** Objects are content-addressed and immutable; tampering changes
  the `id` and fails verification.
- **Non-repudiation.** A signed attestation cannot be disowned later, by either
  party.
- **Ownership of history.** Attestations are held by their authors and their
  subjects, not by an operator, so leaving a pool costs nothing but reach
  (§9.2).

## 14.2. What WRAP does not protect

Stated plainly, because a security section that lists only strengths is
marketing.

- **Confidentiality against a pool.** A pool operator sees the work orders it
  distributes, including addresses and terms (§8.5). WRAP does not encrypt
  offers in v0.
- **Metadata privacy.** Who works for whom, how often, and where is visible to
  counterparties and to any pool in the path. WRAP is not an anonymity system
  and MUST NOT be described as one.
- **Location integrity.** Device geolocation is trivially falsified. It is
  operational data, never proof (§10.4).
- **Key loss.** There is no rotation or recovery in v0 (§2.4). A lost key is a
  lost identity and a lost work history.
- **Sybil resistance.** Not solved; made explicit and delegated to pool
  curation (§9.1).
- **Availability.** A pool can refuse to distribute a participant's work. WRAP
  guarantees only that this costs them nothing but that pool.

## 14.3. Misbehaving parties

**A malicious issuer** can sign two `Assignment` objects for one work order
with the same `ts`. The merge algebra will pick one deterministically, but two
performers may already have started. This is not a distributed-systems failure
— it is a party misbehaving in a way that is signed, permanent and
attributable. Both performers hold the signed evidence and can attest
accordingly (§9.4). WRAP's answer to bad behaviour is durable evidence, not
prevention.

**A malicious performer** can report `completed` without doing the work. The
handoff commitment (§10.2) defeats this whenever a beneficiary is present, and
`kind = 4` honestly marks the cases where it cannot.

**A malicious pool** can withhold offers, favour members, or leak commercial
data. It cannot forge, assign, complete, or destroy history. The mitigation is
exit, and it is only real if participants belong to more than one pool —
implementations SHOULD encourage this in the interface, not merely permit it.

**A malicious peer** can flood a node with well-formed objects. Implementations
MUST bound pending storage (§5.6), MUST enforce the size limit (§4.6), and
SHOULD rate-limit per authenticated peer key.

## 14.4. Replay

Object replay is harmless by construction: objects are immutable and merge is
idempotent, so a re-delivered object changes nothing.

Transport replay is prevented by the substrate wire's authentication (§11.2;
[`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §5.4), not
by a rule of WRAP's own. Object replay being harmless (above) is what lets WRAP
delegate transport freshness entirely to the substrate rather than carry a second
nonce cache.

## 14.5. Denial of service

The `expires` field (§3.3) is REQUIRED for a security reason as well as a
usability one: without mandatory expiry, an attacker can permanently inflate
every pool with work orders that never resolve, and no participant can
distinguish a stale pool from a busy one.

Implementations MUST discard expired work orders and their pending objects, and
MUST NOT allow retention policy to be set to unbounded.

## 14.6. Privacy of beneficiaries

The most sensitive data in a WRAP deployment does not belong to any
participant: it is the customer's home address, and the customer never
consented to a protocol.

Implementations SHOULD publish coarse offers and disclose exact addresses only
in the `Assignment`, which reaches one performer (§8.5). Implementations SHOULD
compact location trails after completion (§7.5). Implementations MUST NOT
publish beneficiary addresses in attestation feeds, which are long-lived,
widely replicated, and designed never to be deleted.
