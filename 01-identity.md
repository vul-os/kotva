# 1. Identity Lifecycle

Identity is the foundation everything else hangs off. The governing principle:

> **Your key is your identity. Your domain, your IP, and your provider are all replaceable
> pointers to it.**

This section defines keys, the key hierarchy, the recovery policy (and how recovery
methods are themselves rotated), key rotation, and name/identity migration. All of these
are **signed, versioned objects** published to the key-transparency log (§3), so every
change is authenticated and every unauthorized change is *detectable*.

## 1.1 Algorithm suites & crypto-agility

Every signed/encrypted object carries a `suite` identifier. Implementations MUST reject
unknown suites (fail closed), never guess.

| `suite` | Sign | KEM / PKE | AEAD | Status |
|--------:|------|-----------|------|--------|
| `0x01`  | Ed25519 | X25519 (HPKE) | ChaCha20-Poly1305 | v0 REQUIRED |
| `0x02`  | Ed25519 + ML-DSA-65 | X25519 + ML-KEM-768 (hybrid) | ChaCha20-Poly1305 | RESERVED (PQ) |

Suite `0x02` is the post-quantum migration target. A node MAY hold keys in multiple suites
during migration; the transparency log records which suite is current. Downgrade is
prevented by pinning (§3) and by the transparency log's monotonic history.

## 1.2 Keys and the identity hierarchy

An identity is rooted in a **root identity key** (`IK`), an offline-capable long-term
signing keypair. `IK` signs everything authoritative but is used rarely; it SHOULD be held
in cold form / recovery custody (§1.4).

```
IK  (root identity key, Ed25519[+ML-DSA])         ← the identity; rarely used
 │  signs ↓
 ├── DeviceKey_1  (per-device signing subkey)     ← day-to-day signing/auth
 ├── DeviceKey_2
 ├── KeyPackages   (MLS: identity/signature + HPKE-init + PQ-KEM keys)   ← async join (§5.3)
 └── RecoveryPolicy vN  (§1.4)                     ← how the identity is recovered
```

- **Device keys** authorize a specific device (phone, laptop, the always-on box). Each is a
  signing subkey signed by `IK` (a `DeviceCert`), with a `label`, `created`, and optional
  `expires`. Devices form the owner's **personal cluster** and share the mailbox via the
  encrypted CRDT sync of §5.
- **KeyPackages** (§5.3) let others start an encrypted MLS session with the identity while
  every device is offline. KeyPackages are signed by a device key; one-time KeyPackages are
  consumed per session.
- A DMTAP **address key** presented to correspondents is `IK`'s public half (the stable
  identity). Correspondents pin it on first use (§3).

### DeviceCert (CBOR)

```
DeviceCert {
  suite:      u8,
  ik:         bytes,        // root identity public key
  device_key: bytes,        // device signing public key
  label:      tstr,         // "phone", "home-box", ...
  created:    u64,          // ms epoch
  expires:    ?u64,
  caps:       [+ tstr],     // "send","recv","relay","mix","gateway","admin"
  sig:        bytes,        // IK over the CBOR-encoded fields above
}
```

`caps` gates what a device *may participate in*. **No single device — including an `admin`
device — may unilaterally change the recovery policy.** Changing `RecoveryPolicy` (§1.4) always
requires either `IK` directly or satisfaction of the current `rotate_threshold` quorum; an
`admin` device counts only as *one factor* toward that quorum (it may, e.g., be a required
member of the quorum, but never sufficient alone). This prevents a single stolen `admin` device
from locking the owner out by rewriting recovery. See §1.4 for the authoritative signer rule.

## 1.3 The `Identity` object (published)

The current public identity is a signed, versioned object; its hash is the anchor everyone
pins.

```
Identity {
  suites:   [+ u8],         // algorithm suites this identity supports, preference-ordered
  iks:      { u8 => bytes },// identity public key per suite (Ed25519 for 0x01, +ML-DSA for 0x02)
  version:  u64,            // monotonically increasing
  devices:  [* DeviceCert],
  keypkgs:  KeyPackageBundleRef,  // location + hash of the current KeyPackage bundle (§5.3)
  recovery: RecoveryPolicyRef,   // hash of the current RecoveryPolicy (§1.4)
  names:    [* tstr],            // self-asserted name(s); trust only after forward name→ik verification (§3.9.4)
  prev:     ?bytes,             // hash of the previous Identity version (hash chain)
  ts:       u64,
  sig:      [+ bytes],         // one signature per suite in `suites`, over all of the above
}
```

**Multi-suite support & PQ transition (normative).** `suites` is a *set*, preference-ordered,
so an identity can hold classical and PQ keys simultaneously during migration. Rules:

- A **sender MUST use the highest suite both parties support** (intersection of the sender's
  supported suites and the recipient's `Identity.suites`); if the intersection is empty, delivery
  fails closed (no silent downgrade).
- **Recipient-side downgrade defense — the suite ratchet (normative).** The sender rule above is
  not self-enforcing: a recipient that still publishes a weaker suite would otherwise accept a
  MOTE encrypted/signed under it even after both parties can do better, so a future break of the
  weaker primitive (e.g. quantum against a classical suite) would defeat the migration. A
  recipient therefore MUST maintain, **per pinned contact**, a **suite high-water-mark** = the
  highest suite it has seen that contact use or advertise (in a validated `Identity`/KeyPackage).
  A subsequent MOTE from that contact using a suite **below** the high-water-mark MUST be rejected
  (`ERR_SUITE_DOWNGRADE`, §21.4) — routed to the requests area with a security warning, never
  silently accepted. The high-water-mark only ratchets **up**; it lowers solely through an
  explicit, `IK`-authorized rotation the owner performs (a genuine suite retirement, §1.5), never
  through an inbound message. An owner MAY additionally publish a signed **`classical_retired`**
  marker in `Identity` that makes rejection of the retired suite unconditional (not merely
  below-high-water-mark). This is the analogue of TLS's downgrade-sentinel: migration to a PQ
  suite delivers protection at **receipt**, not only once the old keys are globally unpublished.
- During transition the `Identity` MUST carry a signature under **every** suite in `suites`
  (`sig` is a list), so a verifier trusting either the classical or the PQ key can validate the
  object — this is what lets the network migrate without a flag day.
- Per-message suite is negotiated at **KeyPackage granularity** (§5.3): a KeyPackage advertises
  its suite, and the sender selects a recipient KeyPackage of the chosen suite. `Envelope.suite`
  records the suite actually used.
- A verifier MUST reject an `Identity` whose highest offered suite it cannot validate rather
  than fall back silently.

`prev` chains versions into a tamper-evident history mirrored in the transparency log (§3).
A verifier accepts an `Identity` only if every `sig` validates under the corresponding key in
`iks` and the chain is consistent with what it has previously pinned/seen.

## 1.4 Recovery policy (and rotating the recovery methods)

Recovery is **not** baked in at setup; it is a first-class, versioned, signed object so the
recovery methods themselves can be rotated.

```
RecoveryPolicy {
  suite:     u8,
  ik:        bytes,
  version:   u64,
  methods: [+ RecoveryMethod],
  recover_threshold: Threshold,   // what regains access
  rotate_threshold:  Threshold,   // higher bar to change THIS policy
  prev:      ?bytes,             // hash chain
  ts:        u64,
  sig:       bytes,             // by IK, OR satisfied rotate_threshold quorum (reactive)
}

RecoveryMethod = PhraseMethod / DeviceMethod / SocialMethod
PhraseMethod { type:"phrase", recovery_key: bytes }            // key derived from a BIP39-style phrase
DeviceMethod { type:"device", device_key: bytes, label: tstr }
SocialMethod { type:"social", guardians: [+ bytes], threshold: u8 }  // Shamir shares

Threshold = { any_of: [+ MethodPredicate] }   // e.g. 1 phrase OR 2 devices OR 2 guardians
```

A **`MethodPredicate`** names a satisfied factor: `Phrase` (the phrase-derived key), `Devices(n)`
(any *n* device factors), `Guardians(n)` (any *n* social shares), or `Ik` (the root key itself).
A `Threshold`'s `any_of` is met when **any** listed predicate is satisfied.

Rules:

1. **Authenticated.** Every version is signed by `IK` (proactive) or by satisfying the
   current `rotate_threshold` (reactive, mid-recovery). Unauthenticated policy changes are
   invalid.
2. **`rotate_threshold` ≥ `recover_threshold`.** A single compromised factor may (perhaps)
   recover access but MUST NOT be able to unilaterally rewrite the policy and lock the owner
   out.
3. **Weakening changes need the quorum, not `IK` alone (compromise defense).** A policy change
   that **removes or weakens any recovery factor** (drops a method, lowers a threshold, evicts a
   guardian/device) MUST satisfy `rotate_threshold` **even when signed by `IK`** — `IK` alone MUST
   NOT be sufficient to weaken recovery. This closes the *stolen-`IK`* takeover where an attacker
   proactively rewrites recovery to evict the owner and install their own factors. **Additive,
   non-weakening** changes (adding a redundant factor) MAY be signed by `IK` alone.
4. **Asymmetric veto window on weakening changes.** A recovery-weakening change MUST be published
   to the transparency log and take effect only **after a veto/delay window** (§16), during which
   the owner's monitoring devices (§3.5) can detect it and publish a **counter-signed veto/abort**.
   **The veto is deliberately asymmetric** to avoid a deadlock in which a compromised factor
   vetoes its *own* eviction:
   - A veto MUST itself satisfy the **`rotate_threshold`** quorum — a *single* prior factor
     (e.g. the very factor being removed) CANNOT veto. This ensures a stolen single factor cannot
     block its own removal.
   - A recovery change that itself satisfies `rotate_threshold` (a genuine quorum-backed
     recovery, §1.4 reactive path) **overrides any veto** and is NOT blockable — so legitimate
     recovery from compromise always wins over an attacker holding one not-yet-removed factor.

   Because rule 3 already requires *every* weakening change to satisfy `rotate_threshold`, a
   rule-conforming weakening is inherently quorum-backed and thus veto-proof; the veto window is
   therefore **defense-in-depth** — it exists to detect and block a *non-conforming* (lesser-bar)
   weakening that slipped through a buggy/malicious implementation, not to gate normal recovery.
   A weakening change MUST NOT take effect instantly; non-weakening changes are not delayed.
5. **Real revocation.** Rotating a method out MUST re-key the underlying secret, not merely
   edit a list:
   - rotate **phrase** → re-wrap the identity secret under the new phrase-derived key; old
     phrase becomes useless;
   - change **guardians / threshold** → **redistribution / resharing** (a fresh access
     structure, Desmedt–Jajodia style), not merely proactive refresh; old shares fall below
     threshold and cannot reconstruct;
   - remove **device** → rotate any identity/recovery material it held.

   **Recommended primitives (grounded):**
   - Use **Verifiable Secret Sharing** (Feldman or Pedersen VSS), **not** plain Shamir, so
     guardians can detect a corrupt share or a cheating dealer at reconstruction (hostile-
     guardian threat model). Plain Shamir has no cheating detection.
   - **Proactive Secret Sharing** refreshes shares under the *same* (M,N); **redistribution**
     changes the access structure (add/remove guardians, change threshold). Use the correct
     one per operation.
   - Prefer **SLIP-0039** for the mnemonic⊕Shamir encoding (purpose-built: two-level groups,
     checksums, passphrase) over hand-rolling BIP39 + Shamir.
   - **Strongly consider FROST (RFC 9591)** threshold Ed25519 signatures so guardians
     *authorize* recovery **without ever reassembling the secret key in one place** —
     eliminating the single-point-of-compromise moment that Shamir reconstruction creates.
     This is the preferred design when the recovery secret is a signing key.
6. **Logged & detectable.** Every version is published to the transparency log; the owner's
   other devices monitor it and MUST alert on a change they did not initiate (intrusion
   detection, §3.5).

### Recovery flows

- **Proactive rotation** (owner has access): publish `version+1` signed by `IK`. Routine
  hygiene; do it whenever a factor is suspected compromised.
- **Reactive recovery** (owner lost access): satisfy `recover_threshold` to reconstruct
  `IK` (or a fresh `IK` authorized by the old under `rotate_threshold`), then immediately
  publish a new policy that invalidates the compromised factor.

### The bottom turtle

There is a foundational anchor (root phrase + social quorum). It is rotatable but requires
the strongest `rotate_threshold`. Two worst cases bound it:

- **Loss.** Losing `IK` **and** enough recovery factors simultaneously is unrecoverable.
- **Compromise (the sharper case).** An attacker who obtains `IK` **plus** enough factors to
  satisfy `rotate_threshold` can rewrite recovery and effect an **owner-unrecoverable takeover** —
  *worse* than loss, because the owner is actively **evicted**, not merely locked out.

Both are mitigated by the same discipline: multiple independent, redundant, rotatable factors so
no single loss is fatal; the weakening-quorum + veto-window rules above so a *partial* compromise
cannot silently escalate to takeover; and KT monitoring (§3.5) so any takeover attempt is at
least detectable and vetoable within the window.

### Backup & restore (content continuity)

Recovery restores the **key**, not the **mailbox**: reconstructing `IK` onto a fresh device
yields an authoritative identity over an **empty store**. Message/file history returns only from a
surviving cluster device (§5.6) or from a **portable encrypted backup**. DMTAP therefore defines a
normative backup primitive: an owner's node MUST be able to export an **encrypted,
integrity-protected backup** of the mailbox/CRDT state (§5.6) and file blobs (§5.5), sealed under
a key derived from the recovery phrase (or a separate backup key held in `RecoveryPolicy`) and
restorable onto an empty store after key recovery. Without a surviving device or such a backup,
recovery preserves the **identity and all relationships** (§1.6) but **not prior content**. See
§14.5 for how this differs from a relay/peer buffer (a buffer is not a backup).

## 1.5 Key rotation

To rotate `IK` (compromise, or scheduled PQ migration):

1. Generate `IK'`.
2. Publish an `Identity` version whose `sig` is by the **old** `IK` and whose body
   authorizes `IK'` (a cross-signed `KeyRotation` record: `old_ik` signs `new_ik` +
   `reason` + `ts`).
3. From then on `IK'` is authoritative; the transparency log's chain proves continuity.
4. Re-issue device certs and KeyPackages under `IK'`. Correspondents update the pin by following
   the signed chain (§3.4).

A verifier MUST accept `IK'` only via a valid chain from a previously-pinned `IK`, or via an
explicit out-of-band re-verification.

## 1.6 Identity (name) migration — never losing your address

Because **key is identity**, losing a domain is a change of *name*, not of identity.

A **MoveRecord** rebinds the human name while preserving the key:

```
MoveRecord {
  suite:   u8,
  ik:      bytes,
  from:    tstr,          // "abc@old.com"
  to:      tstr,          // "abc@new.com"  (or a self-sovereign name, §3.6)
  ts:      u64,
  prev:    ?bytes,
  sig:     bytes,         // by IK
}
```

Distribution (all three):

1. **Mesh/DHT** — the key→location record and `Identity.names` carry the new canonical name.
2. **Transparency log** — an auditable, ordered record of the move (and of the key
   discontinuity at the old name, defeating a squatter who later registers `old.com`).
3. **Push to contacts** — a signed MOTE announcing the move.

Because existing contacts **route by key via the mesh** and verify the `MoveRecord` against
`IK`, they follow the owner automatically and cannot be redirected by a forged move. Only
*new* contacts who know only the abandoned name are affected — the same, unavoidable
tradeoff as giving up any domain, but the *identity and all existing relationships survive*.

**Optional maximal durability:** to make the literal address itself un-loseable and
un-seizable, bind the name in a self-sovereign name backend instead of (or in addition to)
DNS (§3.6). This is the only place DMTAP admits a name-chain, and it is confined to the name
layer.

## 1.7 Object summary

| Object | Signed by | Chained | In transparency log |
|--------|-----------|---------|---------------------|
| `Identity` | IK | yes (`prev`) | yes |
| `DeviceCert` | IK | via Identity | via Identity |
| `RecoveryPolicy` | IK or rotate-quorum | yes | yes |
| `KeyRotation` | old IK → new IK | via Identity chain | yes |
| `MoveRecord` | IK | yes | yes |

All identity-lifecycle operations share one machinery: **signed, versioned objects,
threshold-gated changes, and the transparency log as the audit trail** (§3).
