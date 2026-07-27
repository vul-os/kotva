# 2. Identity

## 2.1. The key is the identity

A WRAP participant is an **Ed25519 keypair**. The public key is the identity.
There is no account, no registration, and no issuing authority. A participant
MAY generate a keypair offline and begin issuing or accepting work immediately.

A **Principal** is a 32-byte Ed25519 public key. Wherever this document says
"issuer", "performer", "pool", or "attestor", it means a Principal.

WRAP defines no username, no email address, and no directory. Display names are
a presentation concern and MUST NOT be used for routing, authorisation, or
identity comparison. Two Principals are equal if and only if their public keys
are byte-equal.

A WRAP Principal **is** a substrate Identity `IK`
([`IDENTITY.md`](https://github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md) §2): WRAP adopts the substrate's
Identity capability (①) unchanged rather than defining a keypair scheme of its
own. A DMTAP mail participant, a flowstock node, or any other substrate identity
is already a valid WRAP Principal, and the same key that receives someone's mail
can issue and perform their work.

**Classical-only, and the limit that follows (normative disclosure).** A WRAP
Principal is specifically the substrate's **classical (suite `0x01`) key** — a bare
32-byte Ed25519 public key. The substrate `Identity` is *multi-suite* (a `suites`
set and an `iks` map, one public key per suite): its v0 REQUIRED originating suite
`0x02` carries a **1 984-byte** hybrid IK, and suite `0x01` is marked **legacy**
(substrate §1.1, §18.2). So the "adopts unchanged / already a valid Principal"
equivalence holds **exactly for a single-suite `0x01` identity**; a multi-suite
substrate identity's IK is not a bare 32-byte key and is not a WRAP Principal as-is,
and a WRAP-native keypair originates under `0x01`, which the substrate treats as
legacy. WRAP is therefore a **classical-only** profile until a future revision
carries the substrate's `suite`/`iks` — disclosed, not hidden.

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

Raw public keys are unusable by humans. WRAP renders a Principal as the
substrate's **8-word key-name** ([`IDENTITY.md`](https://github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md)
§5) — `BLAKE3-256(0x01 ‖ 0x1e ‖ ik_pub)` over the curated ~1024-word list with a
folded checksum, 80 bits — **not** a scheme of its own. An earlier draft defined a
bespoke `words(BLAKE3-256(pubkey))`; that produced a *different name for the same
key* (no algorithm-commit prefix, no checksum), which is exactly the naming
divergence the substrate's rule 6 forbids. WRAP uses the substrate's bytes so a
worker's key-name is the same handle every product shows.

This "same handle" identity holds **only for a single-suite classical (`0x01`)
identity**, where `ik_pub = iks[0x01] = iks[anchor_suite]`. For a **multi-suite**
substrate identity the key-name is derived over `iks[anchor_suite]` (substrate
§18.9.17) — a *different* key than the bare Ed25519 Principal — so WRAP's name and
the substrate's name **differ**. The "same handle every product shows" guarantee is
therefore classical-only, consistent with §2.1's disclosure above.

Two substrate rules WRAP inherits and MUST honour:

- **A key-name is a display hint, never a key.** It MUST NOT be accepted as a
  substitute for a public key on the wire, and — because the 8-word form carries
  only 80 bits (≈2⁴⁰ chosen-collision margin) — MUST NOT be used as an allowlist
  entry, a dedup or database key for identities, or the sole basis for a trust
  decision. Attestation aggregation (§9.3) keys on the full `IK`, never the
  key-name. Where a key-name is the *only* verification or is printed/engraved, the
  substrate's 12-word form is REQUIRED (IDENTITY §5).
- A local **petname** MAY additionally be shown — a nickname the user assigns on
  their own device, never transmitted and never authoritative.

## 2.4. Key rotation, recovery, and revocation — the substrate's, not WRAP's

An earlier draft declared key rotation and revocation "out of scope for v0" and a
known gap. Adopting the substrate closes it: WRAP inherits the substrate's
identity lifecycle whole ([`IDENTITY.md`](https://github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md) §2.2,
§2.3, §5.1), and defines none of its own.

- **Operational keys rotate under `IK` without changing identity.** A performer
  signs work under a `DeviceCert` subkey (IDENTITY §2.2); losing or rotating that
  device key does not lose the identity or its attestations, because existing
  contacts follow the pinned `IK` through the signed certificate chain.
- **Recovery precedence and revocation are defined.** A stolen signing key that
  rotates to an attacker is beaten by the owner's recovery key, resolved by the
  substrate's contest-window finality even at the zero-DNS/zero-KT key-name floor
  (IDENTITY §5.1). Verification MUST honour `DeviceCert` revocation (IDENTITY §2.3)
  — a chain check that skips revocation is a fail-open, which is why WRAP's verify
  step (§5.4 step 5) defers to the substrate's rule rather than restating it.
- **Honest residual (disclosed).** The one event that *does* change a worker's
  key-name is an `IK` rotation itself — the rare migration case, not the common
  device-key case. Existing counterparties follow by pinned key; only a brand-new
  contact who knew *only* the old key-name is affected. Here WRAP's portable
  attestations help socially: a re-keyed worker presents old attestations signed by
  counterparties who still exist and asks a pool to vouch for continuity. That is a
  social bridge over a disclosed cryptographic residual, not a recovery mechanism,
  and implementations SHOULD NOT describe it as more.
