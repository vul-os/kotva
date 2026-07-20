# 2. Identity

## 2.1. The key is the identity

A WRAP participant is an **Ed25519 keypair**. The public key is the identity.
There is no account, no registration, and no issuing authority. A participant
MAY generate a keypair offline and begin issuing or accepting work immediately.

A **Principal** is a 32-byte Ed25519 public key. Wherever this document says
"issuer", "performer", "pool", or "attestor", it means a Principal.

WRAP defines no username, no email address, and no directory. Display names are
a presentation concern and MUST NOT be used for routing, authorization, or
identity comparison. Two Principals are equal if and only if their public keys
are byte-equal.

> **Note.** This makes WRAP identities compatible by construction with any
> system whose root identity is also an Ed25519 key — including DMTAP, whose
> identity key `IK` is Ed25519. A DMTAP participant is already a valid WRAP
> Principal. WRAP does not depend on DMTAP for this to be true (§11.2).

## 2.2. Roles

Roles are not properties of a Principal; they are positions in a relationship.
The same key MAY be an issuer in one work order and a performer in another — a
plumber who subcontracts is both.

| Role | Meaning |
|---|---|
| **Issuer** | Creates a `WorkOrder` and decides its `Assignment`. |
| **Performer** | Executes assigned work and reports `Progress`. |
| **Pool** | Distributes `Offer`s to a membership. Holds no authority over the work itself. |
| **Attestor** | Signs an `Attestation` about an outcome. Usually the issuer, the performer, or a beneficiary. |

A Pool is a Principal like any other. It has no protocol powers: it cannot
assign, cannot complete, and cannot alter a work order. It is a distribution
convenience (§8) and nothing more. This is the property that keeps pools
replaceable.

## 2.3. Key names

Raw public keys are unusable by humans. Implementations SHOULD render a
Principal as an **8-word key name** derived from its public key:

```
words(BLAKE3-256(pubkey))  over a 1024-word list, 8 words
```

Each word contributes 10 bits, for 80 bits of the digest — enough that two
participants reading words aloud can confirm they hold the same key.

Key names are a **display and verification** convention, not a namespace. They
are not looked up, not registered, and not resolved. An implementation MUST NOT
accept a key name as a substitute for a public key on the wire.

Implementations MAY additionally show a local **petname** — a nickname the user
assigns to a key on their own device. Petnames are local, never transmitted,
and never authoritative.

## 2.4. What WRAP does not define

**Key rotation and revocation are out of scope for v0.** This is a known gap
and it is stated plainly rather than hidden: a participant who loses their key
loses their identity and their accumulated attestations, and there is currently
no in-protocol recovery.

Deployments that need rotation SHOULD layer an existing mechanism — a device
certificate chain, a key transparency log, or an out-of-band re-enrollment
through their pool — beneath WRAP. A future version MAY define one. It is not
defined here because a rotation scheme without a matching revocation-checking
rule fails open, and a fail-open identity mechanism is worse than an absent one.

Until then, the practical mitigation is that attestations (§10) are held by
*both* parties and by the pool. A worker who must re-key can present their old
attestations, signed by counterparties who still exist, and ask a pool to vouch
for the continuity. That is a social recovery path, not a cryptographic one,
and implementations SHOULD NOT describe it as more than that.
