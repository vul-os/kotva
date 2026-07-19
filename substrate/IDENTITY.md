# Substrate Capability ① — Identity

> **Status:** additive profile of the core specification. This document **profiles §1 (Identity
> Lifecycle) and §3 (Naming & Directory)** for use by any DMTAP product, mail or not. It restates no
> normative bytes: every wire structure, algorithm, and fail-closed rule remains defined in §1/§3/§18,
> and **those sections govern.** What follows selects the identity capability out of the mail spec,
> names the minimal subset a non-mail product needs, and states the invariants a product MUST preserve.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174.

---

## 1. The one idea

**A keypair is the identity. A name is a pointer to it.** Everything a DMTAP product needs to know
about "who is this" reduces to a public key it can verify signatures against; everything about "what do
I call them" is a separable, optional convenience layered on top. A product that adopts this capability
gets sovereign, portable, provider-independent identity for free, and inherits the core's tamper-evident
naming without having to touch a line of mail code.

Concretely, the Identity capability is four things, in dependency order:

1. an **Ed25519 root identity key (`IK`)** whose public half *is* the identity (§1.2);
2. **`DeviceCert` subkeys** that let day-to-day devices sign without exposing `IK` (§1.2);
3. a **`name → key` binding** published in DNS and made tamper-evident by a **key-transparency (KT)
   log** (§3.2, §3.5);
4. an **8-word key-name** derived from `IK` — the zero-authority floor that needs no DNS, no
   registration, and no `@` (§3.9.6).

A product **MAY** adopt only rungs 1–2 (verify signatures, no human names at all) and still be a
conformant identity user; the key-name (rung 4) is always available at zero cost because it is a pure
function of the key; DNS + KT (rung 3) are opt-in discovery on top.

---

## 2. Keys: `IK` and `DeviceCert` (profile of §1.2)

The normative definitions are §1.2 (keys and hierarchy), §1.2a (hardware-backed keys), and the
`DeviceCert`/`Identity` CBOR in §1.3 and §18. This section states only what a non-mail product must
carry.

### 2.1 The root identity key

- The identity is rooted in `IK`, an **Ed25519** (RFC 8032) long-term signing keypair; its **public
  half is the identity** (§1.2). `IK` signs everything authoritative but is used rarely and SHOULD be
  held cold / in recovery custody (§1.4).
- The `suite` byte (§1.1) makes the algorithm negotiable: v0 is Ed25519 (`0x01`); a PQ hybrid
  (Ed25519 + ML-DSA, `0x02`) slots in per-object without changing any name or address (§1.1). **Unknown
  suites MUST be rejected, never guessed** (`ERR_UNKNOWN_SUITE`, `0x0101`, §1.1). A product that adopts
  Identity inherits this agility rule unchanged — it MUST NOT hard-wire Ed25519 in a way that cannot
  reject or migrate a suite.

### 2.2 Device certificates

Day-to-day signing is done by **device keys**, each a signing subkey certified by `IK` via a
`DeviceCert` (§1.2). The structure (normative in §1.2; reproduced here for orientation only):

```
DeviceCert {
  suite:          u8,
  ik:             bytes,     // root identity public key
  device_key:     bytes,     // device signing public key
  label:          tstr,      // "phone", "home-box", ...
  created:        u64,
  expires:        ?u64,
  caps:           [+ tstr],  // "send","recv","relay","mix","gateway","admin"  — see note
  key_protection: ?tstr,     // "software"|"tpm"|"secure-enclave"|"strongbox"|"tee" (§1.2a)
  attestation:    ?bytes,    // OPTIONAL platform key-attestation over device_key (§1.2a)
  sig:            bytes,     // IK over the CBOR-encoded fields above
}
```

For a non-mail product, `DeviceCert` is exactly the seam that makes **multi-device sync safe** (see
[`SYNC.md`](SYNC.md)): a Sync replica authorizes an incoming op iff its origin device presents a
`DeviceCert` chaining to the pinned `IK` and is not revoked. That is *the* interoperation point between
Identity and Sync — the Sync layer never re-invents authorization, it checks a `DeviceCert` chain.

**`caps` and non-mail products (normative).** The `caps` token list gates *what a device may participate
in*. The tokens `send`/`recv`/`gateway`/`mix` are mail/mesh roles; a non-mail product **MAY** ignore
tokens it does not implement and **MUST NOT** treat an unrecognized cap token as a parse failure
(the same forward-compat discipline as §10.2 capability tokens). A product that defines a new
device-scoped capability **SHOULD** register a `caps` token rather than inventing a parallel
authorization field, so one `DeviceCert` serves every capability the device holds.

**No single device rewrites recovery (inherited invariant).** Even an `admin` device cannot
unilaterally change the recovery policy; that always requires `IK` or the `rotate_threshold` quorum
(§1.2, §1.4). A product that adopts Identity inherits this — it MUST NOT expose an API that lets one
device silently re-key the identity.

### 2.3 Hardware backing and revocation (profile of §1.2a)

The three §1.2a mechanisms apply unchanged and are RECOMMENDED for any product that holds keys on a
user device:

- **Non-exportable hardware-backed keys** (Secure Enclave / TPM 2.0 / StrongBox / TEE), recorded in
  `key_protection`, optionally attested via `attestation` — a compromise can *use* the key only while
  unlocked, never exfiltrate it (§1.2a). Attestation is advisory; it never overrides the owner's §1.4
  authorization, and it expires (re-attest ≤ 90 days, `ERR_DEVICE_ATTESTATION_EXPIRED`, `0x0118`).
- **Compromise healing via revocation (MUST, PCS).** Removing/rotating a compromised device
  (`IK`-authorized rotation, §1.5) heals every capability forward: an evicted device key verifies
  nothing further. For a Sync-adopting product this is what makes device eviction safe — the revoked
  device drops out of the authorized member set and its future ops are rejected (`ERR_DEVICE_
  UNAUTHORIZED`-class, see [`SYNC.md`](SYNC.md)).

---

## 3. The published `Identity` object (profile of §1.3)

The current public identity is a signed, versioned `Identity` object (§1.3) whose hash is the anchor
everyone pins. Its `version` is monotonic and guards against stale-replay (the same anti-rollback family
as feed `seq`, §22.4.2, and Sync watermarks). For a non-mail product the load-bearing fields are `iks`
(the key per suite), `version`, `devices` (the `DeviceCert` set — the authorized signer set), and `prev`
(the hash chain that carries rotations and migrations). Fields specific to messaging (`keypkgs`,
`deniable_prekeys`) are **OPTIONAL to a non-mail product** and MAY be absent — a product that does no
MLS messaging simply does not publish KeyPackage references. The `Identity` object's authenticity and
chain rules (§1.3, §1.5, §1.6) apply unchanged regardless.

---

## 4. `name → key` over DNS + KT (profile of §3.2–§3.5)

A human name is resolved to a key by DNS, and the binding is made tamper-evident by a KT log. This is
opt-in discovery: a product that never displays a human name never needs it.

### 4.1 DNS binding (§3.2)

For `abc@def.com`, a resolver reads a TXT record binding the name to the key (§3.2):

```
abc._dmtap.def.com.  IN  TXT  "v=dmtap1; suite=1; ik=<base64url IK>; id=<hash of Identity §1.3>;
                                kt=<KT log URL>; keypkgs=<KeyPackage bundle locator §5.3>"
```

`keypkgs` is mail/MLS-specific and MAY be omitted by a non-mail product; `ik`, `id`, and `kt` are the
identity-capability core. Resolution (§3.3) is: DNS lookup → **KT verification at first contact** →
fetch and verify the full `Identity` → **pin `(name → ik, id)` (TOFU)** → thereafter route by key, never
consulting DNS again unless the pinned chain says to. **DNS is the front door, used once** (§3.3).

### 4.2 Key transparency (§3.5)

The `name → key` binding is only as trustworthy as the log that records it. KT (§3.5) appends the
owner's identity events to an **append-only Merkle log** (RFC 6962 discipline), serves **signed tree
heads + inclusion/consistency proofs**, and lets the owner's own devices **monitor** the log as identity
intrusion detection. Two profiles (§3.5.1/§3.5.2): **v0-minimal** (single log, tamper-evident,
Core-required) and **v1-hardening** (federated, gossiped, equivocation-detecting). A product that
adopts human names **MUST** adopt at least v0-minimal KT and **MUST fail closed on an unreachable KT at
first contact** — it MUST NOT silently TOFU-pin an unverified key (§3.3, `0x0106`), because that reopens
the replay-an-old-`Identity` attack precisely under the network conditions that make KT unreachable.

### 4.3 First-contact MITM and its closure (honest limit)

A MITM at the *very first* contact, before KT is consulted or before out-of-band verification, can
substitute a key (§3.4 honest limit). KT closes it after the fact; **safety-number / QR out-of-band
verification** (§3.4.1) closes it immediately for high-value contacts. A product that surfaces identity
to users SHOULD offer the safety-number comparison, which compares the **key**, not the name, and is the
strongest trust upgrade.

### 4.4 Confusable-name defenses (inherited, §3.9.7)

Any product that displays or accepts human names inherits the two §3.9.7 spoofing defenses, both
fail-closed: the **single-script-per-label rule** at registration/parse/pin
(`ERR_NAME_LABEL_MIXED_SCRIPT`, `0x0122`) and the **pin-time confusable-skeleton check** against
existing pins (`ERR_NAME_CONFUSABLE_WITH_PIN`, `0x0123`). These protect the human comparison step; they
never weaken the invariant that authenticity is the key.

---

## 5. The key-name — the zero-authority floor (profile of §3.9.6)

The **key-name** is the one name that needs no resolver, no DNS, no registration, and no `@`: it is a
deterministic word-encoding of `BLAKE3-256(IK)` rendered as an **8-word** sequence (80 bits) over the
same curated ~1024-word list and folded checksum as the safety number (§3.4.1), so a single mistyped
word fails closed. A **12-word** (128-bit) form exists for the adversary-proof mode (§16.2).

For a non-mail product the key-name is the most valuable single thing this capability provides for free:

- **Globally unique with zero infrastructure.** Distinct keys yield distinct key-names with
  overwhelming probability (collision-resistant hash), so a product gets a globally-unique handle for
  every identity **without any registry, consensus, or network** — the one Zooko corner (global +
  authority-free) a derived name can occupy (§3.9.6, §15.5).
- **`self`-resolving.** In the resolver framework (§3.12) the key-name is resolver-type `self`:
  "resolution" is a local derivation from the key, not a lookup — nothing to KT-audit, because the
  binding *is* the key. It carries no `@` because it belongs to no authority's namespace.
- **The escape hatch.** Whatever else fails — no domain, a seized domain, a dropped provider — a user
  remains nameable and reachable by key-name (§3.9.6, §3.11.5). A product **SHOULD** expose the key-name
  wherever it needs a durable, provider-independent identifier for an identity.
- **Honest residual (disclosed).** The key-name changes if `IK` rotates (§1.5) — the rare migration
  event, not the common case (device/operational keys rotate *under* `IK` without changing it). Existing
  contacts follow by the pinned key and the signed chain; only a new contact who knows only the old
  key-name is affected (§3.9.6). This is the disclosed price of a coin-free global-unique name.

### 5.1 Informative — contest-window finality for the zero-DNS / zero-KT floor

> **Status: informative, additive.** This note records a rotation-log **fork-resolution rule** proposed
> for the **zero-DNS, zero-KT floor** — the case where an identity is used purely by key-name (§5) with
> no DNS binding and no KT log to anchor rotation ordering. It changes no normative core byte; it is a
> deterministic tiebreak a resolver **MAY** apply when it holds a rotation history but has no transparency
> log to order it. It is drawn from the *vidmesh* protocol's identity model (its rotation-log fork
> resolution), surfaced here as the convergence work of the [Video/Media profile (§24)](../24-video-profile.md)
> turned up a genuinely useful primitive that fills a gap in this exact floor case.

**The gap.** At rung 4 (DNS + KT, §4) the KT log is what makes rotation ordering tamper-evident and
gives two parties a shared, gossiped, append-only history to agree on. At the **key-name floor** there is
by definition *no* such log — the whole point is zero infrastructure. So when a resolver holds two
*validly-signed* rotations of one identity (e.g. a thief who captured a signing key rotates to their own
key, while the legitimate owner rotates using a **recovery key**, §1.4), it needs a rule to pick the
**active branch** that (a) requires no external log, (b) is deterministic given a record set, and (c) is
**partition-safe** — two resolvers with the same records eventually agree.

**The rule (as an option, not a mandate).** Evaluate the held rotations as a tree rooted at genesis and
select the active branch:

1. **Validity.** Discard any rotation not authorized under the chain state at its parent (§1.5 rules).
2. **Recovery-over-signing, until finality.** Where a parent has multiple valid children, a
   **recovery-key-authorized** child supersedes a **signing-key-authorized** child — *unless* the
   signing-key child is already **final**. A signing-key rotation becomes **final** once the resolver has
   held it (first observed it) for longer than a declared **contest window** (a per-identity duration
   published in the rotation itself). Before finality it is provisional and a recovery-key sibling
   displaces it; after finality it stands. This is the **theft-recovery** property: a thief holding a
   stolen signing key cannot outrun the legitimate recovery-key holder within the window, while the
   recovery key cannot rewrite history that has already outlived the window.
3. **Deterministic tiebreak.** Between two children of the **same** authorization class, the one with the
   **bytewise-lower record id** wins — arbitrary but deterministic and partition-safe.
4. **Depth.** Along the selected edges, the deepest node is the current state.

**Why it is partition-safe and infrastructure-free.** First-observation time is *verifier-local*, so two
resolvers that received the same rotations at different moments MAY transiently disagree about finality —
but this is inherent to partition tolerance, and they **converge** once both have held the records for the
window. No shared clock, no log, and no network round-trip is required; the rule is a pure function of
(record set, local first-seen times). A `contest_window = 0` opts the identity out of recovery precedence
entirely (rule 2 never fires). Where portable, human/legal-reviewable ordering evidence is wanted on top,
an external-timestamp anchor (the vidmesh `anchor` pattern, carried as metadata in §24) gives it without
becoming a trust root — the deterministic rule above is still what resolvers *compute*.

**Relationship to the core.** This does not replace KT (§4.2), which remains the Core-required mechanism
wherever human `name@domain` is used and gives *tamper-evidence*, not just fork-resolution. It is offered
for the floor case KT does not cover — an identity that never touches DNS — so that even there, theft of a
day-to-day signing key is **recoverable** by a recovery key rather than permanent. A product **MAY** adopt
it for zero-DNS identities; a product that uses KT does not need it.

---

## 6. The naming ladder is not inverted

This is the invariant the substrate exists to protect (see [`README.md § 3` rule 6](README.md)). The
naming ladder (§3.13.2) runs **key-name (floor) → petname (local) → `name@namespace` (convenience)**, and
**every rung resolves to the same key.** The direction of authority is fixed:

- **A name resolves to a key; a key never resolves to a name.** DNS is the root of *discovery*; the key
  is the root of *authority*. Possessing a name never confers identity (§3.13).
- **`@` is a namespace marker, not a delivery requirement** (§3.13.1). Native addressing is always to a
  key; `@` appears only for a name that lives in an authority's namespace (a DNS domain, a name-chain,
  the opt-in handle registry). Key-name and petname carry no `@`.
- **A waist product MUST NOT introduce a scheme in which a name is authoritative over a key**, or in
  which a name lookup is required for identity, delivery, or verification — all three are complete at
  rung 1 (the key-name) with zero DNS and zero name-chain (§3.13.2). Climbing the ladder adds
  discoverability and human-friendliness; it never adds anything to identity itself.
- **Name changes are non-events** (§3.13.3): a new domain, provider, or handle — even a rotated `IK`
  that changes the key-name — is a change of *label*, not of identity. Existing correspondents route by
  the pinned key and follow a signed `MoveRecord` (§1.6).

A product that respects this ladder gets provider-independence and continuity as a structural property,
not a feature it has to build.

---

## 7. The minimal identity a product needs

The smallest conformant adoption, for a product that just wants to verify who signed something and refer
to them durably:

1. Generate or import an **`IK`** (Ed25519, suite `0x01`), hardware-backed where possible (§1.2a).
2. Issue a **`DeviceCert`** per device from `IK` (§1.2); verify a signature by checking the signer's
   `DeviceCert` chains to a pinned `IK` and is not revoked.
3. Derive and display the **key-name** (§3.9.6) as the durable identifier — no DNS, no registry.
4. **Optionally** add DNS + KT (§3.2, §3.5) when human `name@domain` discovery is wanted, fail-closed on
   unreachable KT at first contact, with confusable defenses (§3.9.7) on display.

Rungs 1–3 require **no network at all**; rung 4 is opt-in. This is the flowstock/HTTP-test posture from
[`README.md § 4`](README.md): identity works offline, over any transport, verified from the key alone.

---

## 8. Security considerations

Each is inherited from §1/§3 and re-stated for the substrate reader; none is new ground.

1. **First-contact MITM.** Closed by KT after the fact and by out-of-band safety-number verification
   immediately (§4.3, §3.4.1). Silent TOFU on unreachable KT is prohibited (§4.2).
2. **Signing-key compromise.** An operational device key compromised while unlocked can sign under the
   identity until revoked (§1.5) and the identity heals forward (PCS, §1.2a, §6.6 item 3). Keep `IK`
   cold; sign day-to-day with revocable device keys (§1.2). For a Sync-adopting product, the blast
   radius is the ops that device could have authored in the interval — bounded by eviction, not
   unbounded.
3. **Attestation trusts a vendor root (disclosed TTP).** Verifying key-attestation evidence trusts a
   platform attestation root — the same class of trusted third party as a WebPKI CA (§1.2a). DMTAP
   confines the dependency to the *advisory* gate and never lets a vendor root override the owner's §1.4
   authorization; the dependency is disclosed, not hidden.
4. **Key-name rotates with `IK`.** Disclosed in §5; the durable identifier is stable only as long as
   `IK` is, and `IK` rotation is a deliberate migration, not a routine event.
5. **Name confusables.** The human comparison step is the attack surface for lookalike strings, not the
   key; §3.9.7's two fail-closed defenses (mixed-script block, confusable-skeleton check) apply wherever
   a name enters the system.

---

## 9. Fail-closed rules this capability contributes

Mirrored into the core auditable set (§10.7); the owning clause governs.

| Invariant | Clause | Trigger | Behavior on violation |
|-----------|--------|---------|-----------------------|
| Unknown suite rejected | §1.1 | a signed object carries a `suite` the implementation does not support | reject, never guess — `ERR_UNKNOWN_SUITE` `0x0101`, FAIL_CLOSED_BLOCK |
| KT unreachable at first contact | §3.3 | first-contact resolve with KT partitioned/censored | MUST NOT silent-TOFU; block or hard-warn + OOB — `0x0106` |
| Device attestation expired | §1.2a | `DeviceCert.attestation` older than re-attest cadence or past expiry, in an attestation-gated context | require re-attest — `ERR_DEVICE_ATTESTATION_EXPIRED` `0x0118` |
| Mixed-script name label | §3.9.7 | a name label mixing Unicode scripts (homoglyph) at registration/parse/pin | reject — `ERR_NAME_LABEL_MIXED_SCRIPT` `0x0122`, FAIL_CLOSED_BLOCK |
| Confusable with existing pin | §3.9.7 | a newly-resolved name whose confusable skeleton collides with a different pinned name | reject, steer to OOB — `ERR_NAME_CONFUSABLE_WITH_PIN` `0x0123`, FAIL_CLOSED_BLOCK |
| Recovery not rewritable by one device | §1.2, §1.4 | a single device attempts to change `RecoveryPolicy` without `IK` / quorum | reject — recovery change requires `IK` or `rotate_threshold` quorum |
