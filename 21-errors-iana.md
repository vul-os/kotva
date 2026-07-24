# 21. Appendix D: Error Codes & IANA Considerations

This appendix is normative. It has two parts: **Part 1** (§21.1–§21.12) is the exhaustive
error/status registry — every failure condition defined anywhere in this specification, given a
stable code, a required responder action, and a retryability classification, so that **no
condition in DMTAP has undefined behaviour**. **Part 2** (§21.13–§21.26) states the IANA
considerations: the registries DMTAP requires, their initial contents, their allocation
policies, and the procedure for extending the protocol without fragmenting it. The key words
"MUST", "MUST NOT", "SHOULD", "SHOULD NOT", and "MAY" are used as in RFC 2119, consistent with
the rest of this specification.

Every table entry in Part 1 is a contract: given the stated condition, the stated action is
the *only* conformant behaviour. An implementation that reaches one of these conditions and does
something not listed in its Action column is non-conformant.

---

# Part 1 — Error / Status Registry

## 21.1 Conventions and code format

Every code is a 16-bit value `0xSSNN`: **`SS`** is the subsystem byte, **`NN`** is the code
point within that subsystem. This is a wire-level registry identifier for logging,
conformance-suite assertions (§10.3), and cross-implementation diagnostics — it is **not**
itself a new envelope field; each error is detected at the point in the spec cited in its
"Operation(s)" column, using the checks already normatively defined there. This appendix does
not introduce new validation logic; it catalogues and names the logic that §§1–17 already
require, and closes every gap where those sections left an outcome implicit.

```
0xSSNN
  SS  subsystem byte   (this appendix, §21.14)
  NN  code point       (within-subsystem, §21.14)
```

Subsystem bytes:

| `SS` | Subsystem |
|-----:|-----------|
| `0x00` | Reserved |
| `0x01` | Identity, Naming & Key Transparency (§1, §3) |
| `0x02` | Delivery & Validation — the MOTE object (§2) |
| `0x03` | Transport & Reachability — mesh, DHT, mixnet (§4) |
| `0x04` | Messaging & Group — MLS, committer (§5) |
| `0x05` | Auth — DMTAP-Auth (§13) |
| `0x06` | Gateway — legacy SMTP bridge (§7) |
| `0x07` | Anti-Abuse & Postage (§9) |
| `0x08` | Files (§5.5, §6.7) |
| `0x09` | DMTAP-PUB extension (§22, ROADMAP) + DMTAP-PUBSUB (§25) — `ERR_PUB_*`; individual code points defined in §22 (§21.24b) and §25 (§21.24d) |
| `0x0A` | DMTAP-SYNC substrate capability (proposed, `substrate/SYNC.md`) — `ERR_SYNC_*`; individual code points defined in `substrate/SYNC.md` §12 (§21.24c) |
| `0x0B` | DMTAP Legacy Adapters extension (§26) — `ERR_ADAPTER_*`; individual code points defined in this appendix's Part 1 (§21.11a), subsystem byte reserved by §21.24g |
| `0x0C`–`0xEF` | Unassigned — reserved for future subsystems (§21.25) |
| `0xF0`–`0xFE` | Private Use (experimental/vendor subsystems) |
| `0xFF` | Reserved |

Two code prefixes are used in the tables below:

- **`ERR_*`** — a rejected/failed condition: the operation did not succeed and the recipient
  took a defensive or corrective action.
- **`STATUS_*`** — a defined **non-error** outcome that still requires a specific, mandated
  responder action (e.g. deduplication). Listing these alongside errors is deliberate: a
  status the spec doesn't name explicitly is exactly where undefined behaviour creeps in.

## 21.2 Responder action vocabulary (normative)

Every code in §21.3–§21.11 resolves to exactly one of the following actions. Definitions are
fixed here once so the tables can cite them tersely without re-explaining each time.

| Action | Definition |
|--------|------------|
| **DROP_SILENT** | Discard immediately. No `ack`, no error returned, no user-visible effect. To the sender/network this is indistinguishable from packet loss (§2.7, §2.7a). |
| **ACK_DEDUP** | Acknowledge (`ack`) without re-processing; the `id` was **previously acked** (§2.6). Never applies to an `id` held only in the requests area — a deferred duplicate is not acked (§2.7a, §19.3.2, §20.2). |
| **IGNORE_NO_ACK** | Do not process (cannot validate), but do not acknowledge either — distinct from DROP_SILENT because the object may be validly formed, just unrecognized (§10.1). |
| **DEFER_REQUESTS** | Accept, but route to a "requests" holding area, rate-limited, never the inbox, never silently discarded (§2.7a). |
| **HOLD_RESYNC** | Buffer locally and initiate a resynchronization/fetch (e.g. a fresh Welcome, a fresh KT proof) before resuming normal processing; not a failure to the user. |
| **FALLBACK_LOWER_TIER** | Treat the presented proof as absent and re-evaluate under the next-weaker anti-abuse tier (token → PoW → postage → deny), never accepted on faith (§9.3.1, §9.5.1). |
| **ROTATE_RETRY** | An internal recovery action (retry via a different path, elect a new committer, re-query disjoint DHT paths) with no user-visible failure unless it is exhausted. |
| **FAIL_CLOSED_BLOCK** | Refuse to proceed and refuse to silently degrade (e.g. refuse to pin, refuse to accept a suite) — the "never guess" rule (§1.1, §3.3, §10.1). |
| **HALT_ALERT** | Halt the affected group/session/identity-verification process and raise a user- or operator-visible security alert. Reserved for conditions that indicate active tampering or equivocation, not ordinary transient failure. |
| **DENY_POLICY** | Reject per a configured (recipient or operator) policy decision — a deliberate, non-adversarial deny, distinct from an abuse/security rejection. |
| **REJECT_NOTIFY** | Reject, and surface the failure to the initiating principal (sender's own client, or the RP) so it can be handled by the user — as opposed to DROP_SILENT, which is invisible to the counterpart. |
| **RETURN_SENDER_SMTP** | Gateway-specific: respond to the legacy SMTP transaction with a 4xx (transient, retry) or 5xx (permanent) code per §21.9's mapping table, so the legacy MTA's own queue is the retry mechanism (§7.2, §7.4). |
| **USER_WARN** | Do not block, but surface a non-blocking, explicit security warning to the affected user (e.g. an unchained key change) — used only where the spec explicitly permits a warn-and-continue path rather than a hard block. |

## 21.3 Identity, Naming & Key Transparency errors (`0x01xx`)

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0101` | `ERR_UNKNOWN_SUITE` | Identity/DeviceCert/RecoveryPolicy/KeyRotation/MoveRecord parse (§1.1, §10.1) | Object carries a `suite` this implementation does not recognise. | No | FAIL_CLOSED_BLOCK |
| `0x0102` | `ERR_SUITE_INTERSECTION_EMPTY` | Suite selection for delivery (§1.3) | Sender's and recipient's supported-suite sets do not intersect. | Conditional (recipient publishes an overlapping suite) | REJECT_NOTIFY |
| `0x0103` | `ERR_IDENTITY_SIG_INVALID` | `Identity` verification (§1.3) | One or more `sig` entries fail to validate under the corresponding `iks` entry. | No | FAIL_CLOSED_BLOCK |
| `0x0104` | `ERR_IDENTITY_CHAIN_BROKEN` | `Identity.prev` / `KeyRotation` / `MoveRecord` chain check (§1.3, §1.5, §1.6) | The presented object's hash chain is inconsistent with what this node has previously pinned or observed. | No | HALT_ALERT |
| `0x0105` | `ERR_STALE_ROLLBACK` | `Identity`/`RecoveryPolicy`/`Profile` version check (§1.3, §1.4, §3.5, §3.9.5) | A version number ≤ a version already seen/pinned is presented (rollback/replay of a superseded-but-validly-signed object). | No | FAIL_CLOSED_BLOCK; HALT_ALERT if the target is this node's own pinned identity |
| `0x0106` | `ERR_KT_UNREACHABLE` | KT check at first contact (§3.3) | Key transparency log is unreachable, partitioned, or censored at the moment of first-contact pinning. | Yes (network condition) | FAIL_CLOSED_BLOCK — MUST NOT silently TOFU-pin; block or hard-warn and require explicit acceptance |
| `0x0107` | `ERR_KT_EQUIVOCATION` | KT tree-head gossip cross-check (§3.5) | The log shows different histories to different observers (split-view). | No | HALT_ALERT |
| `0x0108` | `ERR_KT_PROOF_INVALID` | Inclusion/consistency proof verification (§3.5) | A signed tree head or inclusion proof fails to verify against the log's public key. | Yes (fetch a fresh proof) | FAIL_CLOSED_BLOCK |
| `0x0109` | `ERR_NAME_RESOLUTION_FAILED` | `resolve(name)` (§3.3, §3.6) | DNS/self-sovereign name backend returns no binding, or the binding is malformed. | Yes | REJECT_NOTIFY (to the sender attempting to address this name) |
| `0x010A` | `ERR_MOVE_RECORD_INVALID` | `MoveRecord` verification (§1.6) | Signature invalid, or not chained from the pinned `IK`. | No | FAIL_CLOSED_BLOCK — retain the prior name binding |
| `0x010B` | `ERR_RECOVERY_POLICY_UNAUTHENTICATED` | `RecoveryPolicy` publish (§1.4 rule 1) | Not signed by `IK`, nor by a satisfied `rotate_threshold` quorum. | No | FAIL_CLOSED_BLOCK + HALT_ALERT (owner's monitoring devices must alert) |
| `0x010C` | `ERR_RECOVERY_THRESHOLD_INVALID` | `RecoveryPolicy` publish (§1.4 rule 2) | `rotate_threshold` < `recover_threshold`. | No | FAIL_CLOSED_BLOCK — reject the policy object outright |
| `0x010D` | `ERR_DEVICE_CERT_INVALID` | `DeviceCert` validation (§1.2) | Signature invalid, or `caps` claimed exceed what the signing `IK`/quorum authorised. | No | FAIL_CLOSED_BLOCK |
| `0x010E` | `ERR_RECOVERY_WEAKENING_UNQUORUMED` | `RecoveryPolicy` publish (§1.4 rule 3) | A change that removes/weakens a recovery factor is signed by `IK` alone without satisfying `rotate_threshold` (stolen-`IK` takeover defence). | No | FAIL_CLOSED_BLOCK + HALT_ALERT (owner's monitoring devices must alert) |
| `0x010F` | `ERR_RECOVERY_VETO_WINDOW` | `RecoveryPolicy` weakening effect (§1.4 rule 4, §16.8) | A factor-weakening change attempts to take effect before its 72 h veto/delay window elapses, or a non-conforming lesser-bar weakening is observed within the window. | Conditional (takes effect once the window elapses with no valid veto) | FAIL_CLOSED_BLOCK — hold until the window elapses; a `rotate_threshold`-backed veto aborts it |
| `0x0110` | `ERR_KT_STH_INCONSISTENT` | KT v1 STH gossip cross-check (§3.5.2(a),(d)) | Two validly-signed STHs of the **same** log are mutually inconsistent — equal `tree_size` but differing `root_hash`, or no valid consistency proof exists between them (append-only violation). Distinct from `0x0107` (`ERR_KT_EQUIVOCATION`, the split-view *conclusion*): `0x0110` is the specific append-only/consistency-proof failure that evidences it. | No | HALT_ALERT — stop trusting the log; publish the conflicting STHs as transferable evidence (§3.5.2(d)) |
| `0x0111` | `ERR_KT_LOG_QUORUM_UNMET` | KT v1 multi-log federation binding check (§3.5.2(b)) | A `name → ik` binding is not attested by the required `> n/2` quorum of the pinned log set (logs disagree, or too many are unreachable). | Conditional (a later fetch reaching quorum resolves it) | FAIL_CLOSED_BLOCK — MUST NOT pin on a sub-quorum view; fall back to OOB verification (§3.4.1) |
| `0x0112` | `ERR_KT_STH_STALE` | KT v1 STH freshness check (§3.5.2(a), §16.2) | A presented STH is older than the STH freshness window / not refreshed within the maximum merge delay — the freeze/withholding attack, where a log serves an old but self-consistent head. | Yes (fetch a fresher head) | HOLD_RESYNC — buffer and re-fetch a current STH before trusting the view; escalate to HALT_ALERT if it persists past gossip cross-check |
| `0x0113` | `ERR_DOMAIN_DIRECTORY_SIG_INVALID` | `DomainDirectory` verification (§3.10.3, §18.4.7) | The org directory object is not validly signed by the domain's pinned authority key (§3.10.1). | No | FAIL_CLOSED_BLOCK — do not trust the directory; per-name resolution (§3.3) is unaffected |
| `0x0114` | `ERR_DIRECTORY_ENTRY_UNVERIFIED` | `DirEntry` forward-binding check (§3.10.3, §3.9.4) | A directory entry's `name → ik` does not match the forward DNS + KT binding — the directory indexes, it does not attest. | No | FAIL_CLOSED_BLOCK — render the entry unverified; MUST NOT be used to address mail |
| `0x0115` | `ERR_ORG_MANAGED_UNDISCLOSED` | Member-custody disclosure check (§3.10.2, §18.4.7) | An org-managed (escrowed-key) account is presented without its `org-managed` custody marker — undisclosed org access to a member's mailbox. | No | HALT_ALERT — MUST NOT present as a sovereign identity; surface the escrow honestly |
| `0x0116` | `ERR_DEVICE_ATTESTATION_INVALID` | `DeviceCert` hardware-attestation check (§1.2a, §18.4.2) | A context that **requires** a hardware-backed, non-exportable device key finds the `DeviceCert`'s `key_protection`/`attestation` absent or failing to verify against the platform attestation root. Advisory hardening only — never overrides the §1.4 authorisation authority. | Conditional (re-enroll on an attested keystore) | FAIL_CLOSED_BLOCK — reject the device for the attestation-gated context; a non-gated context is unaffected |
| `0x0117` | `ERR_KT_LEAF_HASH_MISMATCH` | KT inclusion-proof leaf check (§3.5, §18.4.9, §18.4.10) | The leaf a KT `InclusionProof` commits to does not equal the leaf hash recomputed by the Identity-entry rule (`0x1e ‖ BLAKE3-256(0x00 ‖ det_cbor([name, ik, version, identity_id]))`) — the log presents a binding whose leaf does not match the resolved identity. | No | FAIL_CLOSED_BLOCK — the log indexes, it does not redefine; MUST NOT pin on the mismatched leaf (escalate to HALT_ALERT if it evidences equivocation, §3.5.2(d)) |
| `0x0118` | `ERR_DEVICE_ATTESTATION_EXPIRED` | `DeviceCert` attestation-freshness check (§1.2a, §18.4.2, §16.9) | An attestation-gated context finds the `DeviceCert.attestation` evidence older than the re-attestation cadence (≤ 90 days), past its own validity window, or chaining only to a **retired** attestation root. Advisory hardening only — never overrides the §1.4 authorisation authority. | Conditional (re-attest over the same non-exportable key under a current root) | FAIL_CLOSED_BLOCK — reject for the attestation-gated context until re-attested; a non-gated context is unaffected |
| `0x0119` | `ERR_PROFILE_SIG_INVALID` | `Profile` verification (§3.9.5, §18.4.12, §18.9.3) | A `Profile`'s `sig` does not verify under the identity's `IK` (or an `IK`-authorised device key). Profile display data is self-asserted (authenticated to the key, never a real-world-identity claim); a bad signature means the data is not authorised by the key. | No | FAIL_CLOSED_BLOCK — reject the object; retain the prior pinned `Profile` or fall to the §3.9.5 avatar/initials fallback ladder |
| `0x011A` | `ERR_PROFILE_AVATAR_HASH_MISMATCH` | `Profile` avatar integrity check (§3.9.5, §18.4.12) | A `Profile` carries `avatar.hash`, but the bytes fetched from `avatar.url` do not content-address (`0x1e ‖ BLAKE3-256`) to it — the owner-hosted image was swapped/tampered. Tamper-evidence, not a hosting guarantee. | Yes (re-fetch; a corrected host resolves it) | USER_WARN — MUST NOT display the fetched image; fall back down the §3.9.5 ladder (key-derived identicon, then initials) and surface a non-blocking warning |
| `0x011B` | `ERR_PROFILE_AVATAR_URL_UNSAFE` | `Profile` avatar URL safety check (§3.9.5, §18.4.12) | `avatar.url` is attacker-chosen data whose scheme is not `https`, or which resolves (incl. after any redirect) to a loopback / private (RFC 1918 / RFC 4193 ULA) / link-local (`169.254.0.0/16`, cloud-metadata `169.254.169.254`) / non-global address — a server-side-request-forgery or internal-probe attempt via a display pointer. | No | FAIL_CLOSED_BLOCK — MUST NOT fetch the URL; fall back down the §3.9.5 ladder (key-derived identicon, then initials) |
| `0x011C` | `ERR_ALIAS_FORWARD_UNVERIFIED` | Self-asserted-name forward-binding check (§3.9.4, §3.11.3) | A name in the identity's own `Identity.names` (a self-asserted alias) whose forward `name → ik` binding (DNS + KT, §3.3–3.5) does not resolve back to this same identity key — an alias claiming an address the key does not control. The self-asserted-name analogue of the org-directory forward-verify (`0x0114`), applied to the identity's own list. | No | FAIL_CLOSED_BLOCK — render the alias **unverified**; MUST NOT display it as authenticated nor use it to address mail |
| `0x011D` | `ERR_ALIAS_REVOKED` | Revoked-alias use check (§3.9.4, §3.11.3, §3.11.5) | An alias used to address the identity has been **revoked** (dropped in a newer signed `Identity` version and its `name → ik` DNS + KT binding retired), while the key and the identity's other aliases remain valid. Independently-revocable aliases: revoking one MUST NOT be usable off a stale cache. | No (this alias) | REJECT_NOTIFY — tell the sender to use a live alias or the key-name (§3.9.1); the key and other aliases are unaffected |
| `0x011E` | `ERR_NAMECHAIN_BINDING_UNVERIFIED` | Name-chain bidirectional binding check (§3.12.5(b)) | A crypto name-chain (`.eth`/`.sol`, resolver-type `name-chain`, §21.18) resolution whose two binding directions **disagree**: the on-chain `name → ik` record names a key that does not claim the name in its signed `Identity.names`, or a claimed name whose chain record resolves to a different key. The chain is a discovery pointer KT audits (§3.3–3.5), never a trust root. | No | FAIL_CLOSED_BLOCK — render the name **unverified**; MUST NOT display it as authenticated nor use it to address mail |
| `0x011F` | `ERR_RESOLVER_TYPE_UNSUPPORTED` | Resolver-type recognition (§3.12.2) | A name in a resolver type (§21.18) the verifier does not implement, or that is unregistered — the "unknown ⇒ reject, never guess" discipline (as for an unknown suite §1.1 or transport substrate §4.1). The name is unresolvable; the identity is unaffected (its other resolvers and the key-name §3.9.6 still resolve it). | No | FAIL_CLOSED_BLOCK — treat the name as unresolvable; MUST NOT guess a binding |
| `0x0120` | `ERR_RESOLVER_DISAGREEMENT` | Multi-resolver cross-check (§3.12.3) | Two independent resolvers, **each having passed step 2 (KT / bidirectional verification, §3.12.1)**, return **different** `ik` for the **same** name (e.g. a `dns` `_dmtap` pointer and a `name-chain` record the owner also publishes disagree). Since a genuine identity has exactly one key, the disagreement is treated as a potential attack, never silently reconciled to one key. A pointer that **fails** its own step 2 is discarded as unresolved (its own code, e.g. `0x011E`/`0x0114`/`0x0111`) and is **not** counted as a disagreeing peer, so one bogus published record cannot force a halt (§3.12.3). Distinct from `0x011E` (`ERR_NAMECHAIN_BINDING_UNVERIFIED`, the *bidirectional* key↔name mismatch **within one** name-chain resolution): `0x0120` is *inter-resolver* disagreement **across** resolver types (§3.12.3), strengthening the anti-equivocation posture of §3.5. | No | HALT_ALERT — MUST NOT pin; raise a security alert and fall back to KT-quorum (§3.5.2(b)) or out-of-band verification (§3.4.1) to decide the true key |
| `0x0121` | `ERR_KEYROTATION_UNAUTHORIZED` | `KeyRotation` authorisation check (§1.5, §18.4.5) | A `KeyRotation` for an identity that has a published `RecoveryPolicy` (§1.4) is signed by `old_ik` **alone** — it carries **neither** a valid `rotate_threshold` co-signature (`rotate_quorum`, path (a)) **nor** has it been published to KT and passed its §16.8 veto/delay window (path (b)). Installing a new authoritative `IK` is at least as powerful as a recovery-weakening change (§1.4 rule 3), so `old_ik` alone MUST NOT effect it: this closes the stolen-`IK` un-vetoable eviction and the `recover_threshold`-only-reconstruct-then-rotate takeover. In fork resolution a `rotate_threshold`-backed branch is preferred over an `old_ik`-alone branch (§1.5). | Conditional (a quorum-backed re-issue, or the same rotation once its published veto window elapses un-aborted, is accepted) | FAIL_CLOSED_BLOCK — reject or hold; MUST NOT advance the pin to `new_ik`; HALT_ALERT if it competes with a quorum-backed branch at the same chain position (via `0x0104`) |
| `0x0122` | `ERR_NAME_LABEL_MIXED_SCRIPT` | Name-label script check at registration/pin (§3.9.7) | A single name label mixes Unicode scripts (the single-script-per-label rule) — a homograph-spoofing vector (e.g. Cyrillic `а` inside a Latin label). | No | FAIL_CLOSED_BLOCK — reject the label; MUST NOT register or pin it |
| `0x0123` | `ERR_NAME_CONFUSABLE_WITH_PIN` | Confusable-skeleton check at pin time (§3.9.7, §3.4) | A name reduces (UTS #39 confusable skeleton) to the **same skeleton** as an already-pinned contact's name — a visually-confusable impersonation of an existing pin. | No | FAIL_CLOSED_BLOCK — reject the pin; surface the collision to the user; prefer OOB verification (§3.4.1) |
| `0x0124` | `ERR_DEVICE_UNAUTHORIZED` | Device authorisation-policy check (§1.4) | An authorisation-**policy** failure: a well-attested device (valid `DeviceCert`, valid attestation) is nonetheless **not authorised** by the identity's §1.4 policy for the attempted act. Distinct from `0x010D` (`ERR_DEVICE_CERT_INVALID`, a cryptographically bad/over-capped cert): here the cert is valid but the policy says no. | No, without a policy change | FAIL_CLOSED_BLOCK — refuse the act; MUST NOT proceed on an unauthorised device |
| `0x0125` | `ERR_SUITE_BELOW_FLOOR` | Originating-suite floor check (§1.1) | The sender selected, or attempted to originate under, an algorithm suite **below the v0 originating floor**. Suite `0x01` is retained for *verification* of historical or constrained-peer objects only; a conformant node MUST NOT originate it, nor select it for a new relationship. Distinct from `0x020F` (`ERR_SUITE_DOWNGRADE`, the recipient-side per-contact high-water-mark ratchet, §1.3): `0x020F` polices a *peer's* regression against a mark, while `0x0125` polices the *absolute* floor every originator owes the network regardless of contact history. | No | FAIL_CLOSED_BLOCK — refuse to originate; the object is not sent |
| `0x0126` | `ERR_VOUCH_SUBJECT_MISMATCH` | Vouch subject binding (§2.7 step 8(b2), §9.2a) | The accepted cold-sender challenge was a **vouch**, but the decrypted `Payload.from` is not the `VouchToken.subject` the voucher named — i.e. the presenter is not the party vouched for. A vouch travels in the cleartext envelope and cannot be bound to the ephemeral `sender_key` at mint time (the voucher cannot know a key the vouchee has not generated, and a cleartext proof-of-possession would break sealed sender, §6.2), so a lifted vouch is otherwise fully usable by whoever copies it. Distinct from `0x0202` (`ERR_PAYLOAD_SIG_INVALID`): the payload signature here is **valid** — it is simply the thief's. | No | DROP_SILENT — discard and do **not** `ack`, matching the step-8 failure posture; the vouch MUST also be counted against `subject`'s §9.7 rate limit at the gate |
| `0x0127` | `ERR_HASH_ALG_MISMATCH` | Hash-agility prefix check (§18.1.5, §18.1.6) | A `hash` field's §18.1.5 multihash prefix does not name the content-hash the object's `suite` selects (§18.1.4), or a signature over a pre-hashed representative was computed against a **bare, unprefixed** digest instead of the multihash form (§18.1.6). The suite is authoritative where present; the prefix is self-description for suite-less objects and a redundancy check everywhere else, **never an independent selector** — otherwise whoever writes the prefix chooses which hash the object's integrity rests on, a downgrade channel inside the agility mechanism itself. | No | FAIL_CLOSED_BLOCK — reject the object; MUST NOT verify under the algorithm the prefix names, and MUST NOT "try both" |

## 21.4 Delivery & Validation — the MOTE object (`0x02xx`)

This table is the authoritative disposition for **every** step of the §2.7 validation order.
Codes are listed in the same order as the steps they correspond to.

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0201` | `ERR_UNKNOWN_VERSION_OR_SUITE` | §2.7 step 1 | Envelope `v` or `suite` unrecognized. | No | DROP_SILENT |
| `0x0202` | `ERR_BAD_CONTENT_ADDRESS` | §2.7 step 2 | `id` does not match the content address of `ciphertext`. | No | DROP_SILENT |
| `0x0203` | `ERR_BAD_SENDER_SIG` | §2.7 step 3 | `sender_sig` fails to verify over `(id‖to‖ts‖kind‖challenge)`. | No | DROP_SILENT |
| `0x0204` | `ERR_UNRESOLVED_TO` | §2.7 step 4 | `to` (DeliveryTag) does not resolve to this node or a group it belongs to. | Conditional (a later group-membership change may resolve it) | DROP_SILENT — the originating sender's own delivery state machine (§2.6, §4.7) governs retry, independent of this node |
| `0x0205` | `ERR_CHALLENGE_INVALID_FORGED` | §2.7 step 6, §2.7a | Cold-sender `challenge` is cryptographically invalid or forged. | No | DROP_SILENT — no `ack` |
| `0x0206` | `ERR_CHALLENGE_ABSENT_INSUFFICIENT` | §2.7 step 6, §2.7a | Cold-sender `challenge` is absent or below the recipient's policy threshold. Canonical disposition detailed in §21.10 (`0x0701`/`0x0702`). | Yes (resend with a sufficient challenge) | DEFER_REQUESTS |
| `0x0207` | `ERR_DECRYPT_FAILURE` | §2.7 step 7 | `ciphertext` fails to decrypt under the resolved MLS epoch key or HPKE recipient key. | No | DROP_SILENT |
| `0x0208` | `ERR_PAYLOAD_SIG_INVALID` | §2.7 step 8 | `Payload.sig` fails to verify under `Payload.from`. | No | DROP_SILENT |
| `0x0209` | `ERR_FROM_PIN_MISMATCH` | §2.7 step 8, §3.4 | Decrypted `from` does not match the pinned identity for a known contact. | No | HALT_ALERT — surface a security warning; MUST NOT silently repin |
| `0x020A` | `ERR_KIND_UNKNOWN` | §2.3, §10.1 | `kind` is outside the implemented set and not a recognised extension. | No (per this MOTE) | IGNORE_NO_ACK |
| `0x020B` | `ERR_EXPIRED_MOTE` | `Payload.expires` (§2.4, §16.1) | Client-requested expiry has passed at receipt time. | No | DROP_SILENT — cooperative hint only (§6.6 item 8), not a security guarantee |
| `0x020C` | `ERR_TIMESTAMP_OUT_OF_SKEW` | `Envelope.ts` vs. receiver clock (§16.1) | `ts` falls outside the ±120 s clock-skew tolerance. | Yes (resend with a fresh `ts`) | DROP_SILENT for cold senders; implementations MAY be lenient toward known contacts |
| `0x020D` | `ERR_MALFORMED_OBJECT` | any CBOR parse (Envelope/Payload/Attachment/Manifest) | Object fails to parse as well-formed CBOR against its schema. | No | DROP_SILENT |
| `0x020E` | `STATUS_DUPLICATE_ID` | §2.6 (deduplication), §2.7a, §19.3.2, §20.2 | Recipient has **previously acked** `id` — a re-delivery of an already-acked (inbox-stored) MOTE. **Not** merely "already holds": an `id` held only in the requests area was never acked, and a re-delivery of it MUST NOT emit this status or any ack (§2.7a, §19.3.2 — acking a deferred `id` reopens the existence oracle; the ordinary cold-sender retry of §20.1 takes that path for up to [§16.1: 72 h]). | N/A | ACK_DEDUP (previously-acked `id` only; a deferred duplicate stays `DEFERRED` with no ack, §20.2) |
| `0x020F` | `ERR_SUITE_DOWNGRADE` | §2.7 step 8, §1.3 (suite ratchet) | `Envelope.suite` is **below** the sender-contact's pinned suite high-water-mark — a downgrade attempt (e.g. a broken classical suite offered after both parties migrated to PQ). | No | DEFER_REQUESTS + USER_WARN — route to requests with a security warning; MUST NOT accept the downgraded MOTE, MUST NOT ratchet the high-water-mark down |
| `0x0210` | `ERR_HYBRID_SUITE_INCOMPLETE` | §1.3 (hybrid composition), §16.7 | A hybrid-suite object (`0x02`) presented to a verifier that **supports** the hybrid suite validates on only **one** component (e.g. Ed25519 passes but the ML-DSA-65 component is absent/fails) — an intra-suite strip of the PQ half. A hybrid verifier MUST require **all** component signatures (AND-composition) and the X-Wing IND-CCA KEM combiner; single-component acceptance is for a genuinely legacy (single-component) verifier only, at that component's lower assurance. | No | FAIL_CLOSED_BLOCK — reject the incomplete/downgraded hybrid; MUST NOT accept it on the classical half |
| `0x0211` | `ERR_ENVELOPE_CONTEXT_MISMATCH` | §2.7 step 8, §18.9.2 | The `Envelope`'s `kind`/`ts`/`to` do not equal the values **bound into `Payload.sig`** (which now covers them, §18.9.2). Because bare `sender_sig` (§18.9.1) is minted by an anyone-can-mint ephemeral key, a re-emitter of the sealed `ciphertext` could otherwise re-mint it over an altered `kind`/`ts`/`to` — rewriting displayed timestamp/causal order, or relabeling `kind` (chat↔mail render/tier change, or → `0x0b` to force a silent decrypt-fail). The identity signature now binds them, so a post-signing envelope edit is detected here. | No | DROP_SILENT — an altered-context MOTE reveals nothing to notify; the untampered original still delivers on the sender's own retry (§20.1) |
| `0x0212` | `ERR_EDIT_REDACT_AUTHOR_MISMATCH` | §2.7 step 9 (same-author gate) | For `kind = edit` (`0x03`) or `kind = redact` (`0x04`), the incoming MOTE's `Payload.from` does not equal the `Payload.from` of every MOTE named in `refs`, **as the recipient itself stored it** — a cross-author edit/redact attempt, or a `refs` target this node does not hold (an unresolvable target is treated as a mismatch, fail closed rather than fail open). Without this gate, combined with the durable cluster-wide remove-wins semantics of §5.6.4, any correspondent could permanently delete or silently rewrite another party's message in the recipient's own view with a validly signed MOTE of their own. | No | DROP_SILENT — discard **without `ack`**, the same disposition regardless of which of the two triggering conditions applied, so the check cannot become an authorship or existence oracle against a `refs` target a prober does not already know is present; mirrors the disposition of a forged `sender_sig` (`0x0203`) |
| `0x0213` | `ERR_TS_TOO_STALE` | §20.1 (`check_freshness` = `fail_past_stale`), §2.6, §16.10 | A validly-signed MOTE whose `ts` is **older than the durable seen-id horizon** (§16.10) — a captured MOTE replayed after ageing out of this node's dedup cache (§2.6), presented with every other check otherwise passing. Distinct from `0x020C` (`ERR_TIMESTAMP_OUT_OF_SKEW`), the symmetric ±120 s clock-skew window at receipt, and from `0x020E` (`STATUS_DUPLICATE_ID`), which absorbs only replays still **within** the horizon: this is the past-direction staleness **beyond** the horizon that closes the aged-replay window a bare dedup no longer covers. | No | DROP_SILENT — no `ack`; MUST NOT be relaxed for known contacts, since leniency here reopens exactly the replay window this step closes (§20.1, H-7) |

**Content-addressed dedup as replay defence.** §2.6/`0x020E`'s dedup-by-`id` is what makes a bare
resend of a previously-processed MOTE a non-event rather than a distinct "replay" failure mode
at this layer — the content address absorbs it structurally. Replay of a **nonce** (as opposed
to a MOTE) is a distinct concept scoped to the Auth ceremony; see `0x0502` (§21.7).

## 21.5 Transport & Reachability errors (`0x03xx`)

> `0x0307`, `0x030B`–`0x0311` below are **mix codes**: they apply only to the opt-in,
> research-tier `private` mixnet tier ([docs/research/mixnet.md](docs/research/mixnet.md)) and
> are unreachable on a node that has not implemented it. Every other code in this table applies
> to the default `fast` tier as well.

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0301` | `ERR_LOCATION_SIG_INVALID` | `LocationRecord` verification (§4.2) | Signature fails to validate under the claimed device key. | No | FAIL_CLOSED_BLOCK — discard the record |
| `0x0302` | `ERR_LOCATION_STALE` | `LocationRecord` sequence check (§4.2) | Sequence number ≤ a previously-seen record for this key (rollback/censorship defence). | No | FAIL_CLOSED_BLOCK — retain the newer cached record |
| `0x0303` | `ERR_LOCATION_UNREACHABLE` | DHT lookup (§4.2, §4.3) | No location record found, or no peer can be dialed via any resolved address. | Yes | ROTATE_RETRY — fall down the reachability ladder (§4.3); ultimately governed by the sender's delivery state machine (§4.7) |
| `0x0304` | `ERR_ECLIPSE_SUSPECTED` | S/Kademlia disjoint-path lookup comparison (§4.2 CAUTION) | Disjoint lookup paths disagree beyond tolerance, suggesting routing-table poisoning. | Yes | ROTATE_RETRY (re-query via disjoint paths / rendezvous); HALT_ALERT if persistent across attempts |
| `0x0305` | `ERR_RELAY_RESERVATION_UNAVAILABLE` | Circuit-relay v2 reservation (§14.5, §16.6) | Target relay has no free reservation slot. | Yes | ROTATE_RETRY — try an alternate relay |
| `0x0306` | `ERR_RELAY_CIRCUIT_CAP_EXCEEDED` | Circuit-relay v2 per-circuit cap (§16.6: 2 min / 128 KiB) | Circuit closed after hitting its duration/byte cap. | Yes | REJECT_NOTIFY — fall back to direct/hole-punch or another relay |
| `0x0307` | `ERR_MIX_PACKET_MALFORMED` | Sphinx onion peel ([docs/research/mixnet.md §4.4](docs/research/mixnet.md)) | A mix hop cannot peel a layer (corrupt or non-conformant packet). | No | DROP_SILENT — a content-blind mix has no channel to notify anyone |
| `0x0308` | `ERR_SENDER_RETRY_DEADLINE_EXCEEDED` | Delivery state machine `EXPIRED` (§2.6, §4.7, §16.1: 72 h) | Sender's retry deadline elapsed without an `ack`. | No | REJECT_NOTIFY — notify the sending user; drop the queued MOTE |
| `0x0309` | `ERR_OFFLINE_BUFFER_UNAVAILABLE` | Peer buffering / relay-mailbox pickup (§4.3, §14.5) | The buffering peer or relay-mailbox is unreachable, or the buffer's TTL has elapsed. | Yes, until TTL | ROTATE_RETRY before TTL; REJECT_NOTIFY after |
| `0x030A` | `ERR_CAPABILITY_ANNOUNCE_ROLLBACK` | `system`-MOTE capability announcement version check (§10.2) | A capability announcement's `caps_version` is older-than-or-equal-to the last accepted from that peer — a stale replay attempting to suppress an advertised capability (downgrade). | No (this announcement) | FAIL_CLOSED_BLOCK — retain the higher-versioned capability set; do not roll back |
| `0x030B` | `ERR_MIX_DIRECTORY_SIG_INVALID` | `MixDirectory` verification ([docs/research/mixnet.md §4.4.2](docs/research/mixnet.md), §18.5.3) | The mix directory is not validly signed by the pinned directory authority (or fails the `> n/2` authority quorum). | No | FAIL_CLOSED_BLOCK — do not use the fleet; a KT split view over the directory is `0x0107` |
| `0x030C` | `ERR_MIX_DESCRIPTOR_STALE` | Sphinx path build / mix key epoch check ([docs/research/mixnet.md §4.4.4](docs/research/mixnet.md), §18.5.2) | A `MixNodeDescriptor` has no key for a usable epoch, or a packet was built to an expired/rotated mix key (`valid_until` passed). | Yes (refresh the `MixDirectory` and rebuild for the current epoch) | ROTATE_RETRY — re-fetch the directory, rebuild the path for the current epoch |
| `0x030D` | `ERR_MIX_PATH_UNBUILDABLE` | Stratified path selection ([docs/research/mixnet.md §4.4.3](docs/research/mixnet.md), [docs/research/mixnet.md §4.4.8](docs/research/mixnet.md)) | Cannot build a path meeting the **in-force profile's** bar ([docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)) from the derived fleet view — a stratified layer has no live/reachable mix, **or** no candidate set satisfies the attested-operator / ASN-disjointness rules of [docs/research/mixnet.md §4.4.8](docs/research/mixnet.md) (an un-attested `operator` claim contributes no diversity). | Yes (a later mix-key epoch may repopulate the view) | ROTATE_RETRY; REJECT_NOTIFY once the sender's retry budget (§4.7) is exhausted — and never by relaxing the bar (`0x0310`) |
| `0x030E` | `ERR_MIX_REPLAY_DETECTED` | Per-epoch mix replay cache ([docs/research/mixnet.md §4.4.6](docs/research/mixnet.md)) | A mix received a Sphinx packet whose per-hop tag is already in its current-epoch replay cache — a replayed packet (correlation / n−1 replay attempt). | No | DROP_SILENT — a content-blind mix has no channel to notify; the duplicate is simply dropped |
| `0x030F` | `ERR_MIX_ACTIVE_ATTACK_SUSPECTED` | Loop-cover detection ([docs/research/mixnet.md §4.4.7](docs/research/mixnet.md)) | A node's loop-cover return fraction fell below the loss threshold (or latency inflated beyond the delay budget), inferring an active drop/delay/flooding attack on its paths. | Yes (after rotation) | HALT_ALERT — rotate away from implicated mixes/guards, alert the user, and **fail closed for `private`** (MUST NOT auto-downgrade to `fast`, [docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)) |
| `0x0310` | `ERR_PRIVATE_TIER_DOWNGRADE_REFUSED` | Minimum-viable-path check ([docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)) | No path meeting the **in-force profile's** bar is buildable (Standard: ≥ 3 hops, 1/layer, ≥ 3 disjoint operators; **High-security: ≥ 5 hops, ≥ 5 disjoint operators**), all current-epoch keys — an adversary DoSing mixes to force a downgrade, or genuine outage. **Covers both a tier downgrade (`private → fast`) and a profile downgrade (High-security → Standard):** a high-security message that can only build a lesser-bar path fails here rather than silently shipping over Standard strength. | Yes (hold + retry until a viable path exists) | FAIL_CLOSED_BLOCK — hold the MOTE in the sender queue (§4.7), never silently route it over `fast`, a shorter/non-diverse path, or a lower profile's bar; surface to the user if it persists past the retry deadline |
| `0x0311` | `ERR_MIX_DIRECTORY_STALE` | Fleet-view freshness check ([docs/research/mixnet.md §4.4.2](docs/research/mixnet.md), §16.3) | The client's **derived fleet view** (or a served cache of it) is older than the mix-directory freshness window (≤ one mix-key epoch) — a stale, possibly frozen fleet view kept on the client to hold its diversity/anonymity set small (freeze attack, analogue of KT STH-freshness `0x0112`). | Yes (re-derive from the log quorum / re-fetch) | **FAIL-QUEUED** (§10.7.0), not fail-closed-auth: refresh before building any `private` path; if no fresh view is obtainable, **hold the MOTE in the retry queue** and keep retrying (ROTATE_RETRY; REJECT_NOTIFY only past the retry deadline). MUST NOT downgrade the tier ([docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)) and MUST NOT refuse to enqueue — a liveness failure delays mail, it never stops it |
| `0x0312` | `ERR_PUSH_SUBSCRIPTION_SIG_INVALID` | `PushSubscription` verification (§4.9.1, §4.9.4, §18.5.5, §18.9.15) | A `PushSubscription`'s signature does not verify under its claimed `device_key`, or that key is not an `IK`-authorised device key of the owner (§1.2) — the subscription is not authenticated to the identity, so acting on it could register/redirect a device's wakes. | No | FAIL_CLOSED_BLOCK — discard the subscription; never wake against it |
| `0x0313` | `ERR_WAKEPING_CONTENT_PRESENT` | `WakePing` decode (§4.9.1, §18.5.6) | A `WakePing` carries any field beyond the opaque sealed token (key `1`), or its opened plaintext decodes to anything bearing sender/subject/recipient/content — a wake must be content-free and sender-blind. | No | FAIL_CLOSED_BLOCK — reject the wake; a `WakePing` MUST carry only the RFC 8291-sealed sync token |
| `0x0314` | `ERR_WAKEPING_AUTH_FAILED` | `WakePing` open (RFC 8291 AEAD, §4.9.4, §18.9.15) | The wake token's `aes128gcm` AEAD fails to open under the subscription's `push_key`/`auth_secret` — a forged or unauthenticated wake (the push relay lacks the auth secret and cannot forge one). | No | DROP_SILENT — drop; MUST NOT be surfaced as a real sync trigger (an unauthenticated wake reveals nothing to notify) |
| `0x0315` | `ERR_WAKEPING_RATE_LIMITED` | Per-device wake rate limit (§4.9.4, §16) | Wakes to this device exceed its rate budget (a wake spends the target's battery); bursts are coalesced into one wake per window. Enforced at the emitting node **and** at the receiving device (the receiver-side budget bounds a push relay that replays/floods wakes the emitter never sent). | Yes, after the window resets | DEFER_REQUESTS — coalesce/hold below the cap; DROP_SILENT beyond it |
| `0x0316` | `ERR_WAKEPING_REPLAY` | `WakePing` replay-cache check (§4.9.1, §4.9.4, §18.5.6, §16) | A `WakePing` whose sealed sync nonce is already in the device's replay cache — a push relay re-delivering a captured ciphertext to re-wake (drain the battery of) the device; the emitting node's rate limiter cannot see it because the replay never traverses the node. | No | DROP_SILENT — drop without re-waking; MUST NOT be surfaced as a real sync trigger |

## 21.6 Messaging & Group errors — MLS (`0x04xx`)

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0401` | `ERR_MLS_CIPHERSUITE_UNSUPPORTED` | KeyPackage/GroupInfo intake (§5.1) | Advertised MLS ciphersuite is not in this implementation's supported set. | No | FAIL_CLOSED_BLOCK |
| `0x0402` | `ERR_KEYPACKAGE_INVALID_OR_EXHAUSTED` | Async join / Add (§5.3) | KeyPackage fails signature verification, or the identity's KeyPackage bundle is exhausted. | Yes (fetch a freshly replenished KeyPackage) | REJECT_NOTIFY |
| `0x0403` | `ERR_COMMIT_ORDERING_VIOLATION` | Committer-log application (§5.1) | A Commit is received out of the committer's hash-chained log order. | Yes | HOLD_RESYNC — re-fetch the log head before applying |
| `0x0404` | `ERR_COMMITTER_FORK_DETECTED` | Committer-log integrity check (§5.1) | Two Commits occupy the same log position with the same predecessor — proof of committer misbehavior. | No | HALT_ALERT — members MUST halt and alert, analogous to KT equivocation (`0x0107`) |
| `0x0405` | `ERR_COMMITTER_UNREACHABLE` | Committer liveness (§5.1) | The group committer has not been reachable past the rotation timeout. | Yes | ROTATE_RETRY — hold pending Proposals, then elect a new committer via a Commit referencing the last agreed log head |
| `0x0406` | `ERR_EPOCH_MISMATCH` | `Envelope.epoch` resolution (§2.2, §5) | The referenced MLS epoch is unknown or superseded at this member. | Yes | HOLD_RESYNC — request a Welcome/state resync |
| `0x0407` | `ERR_WELCOME_INVALID` | Async join via Welcome (§5.3) | Welcome message fails to verify or decrypt against the joining member's KeyPackage. | Yes (request re-issue) | REJECT_NOTIFY |
| `0x0408` | `ERR_EXTERNAL_COMMIT_REJECTED` | Self-join via External Commit (§5.3) | `GroupInfo` referenced by the External Commit is stale or invalid. | Yes | REJECT_NOTIFY |
| `0x0409` | `ERR_GROUP_POLICY_VIOLATION` | Add/Remove/role/policy Commit (§5.8.2); operational failure-mode table §19.5.2 | The proposing member's role, or the requested change, is not permitted. One code spans several conditions (§19.5.2): an ordinary lack of the required role, a `closed` join policy, a `reader` attempting to post, a change not applicable to the current roster — **and** the **rank-rule** variant (an actor acting on, or granting, a role **strictly above its own** — e.g. an `admin` expelling/demoting/minting an `owner`). The action class depends on which: the ordinary denials are `DENY_POLICY`; the **rank-rule variant is `FAIL_CLOSED_BLOCK`**, because §5.8.2 frames it as a group-**takeover** defence ("would let an `admin` seize the group by expelling its owners"), i.e. a security rejection, which §21.2 says `DENY_POLICY` is *not* for (cf. `0x070E`). | No, without a role change | DENY_POLICY for an ordinary role/join/roster deny; **FAIL_CLOSED_BLOCK** for the rank-rule (anti-takeover) variant, per §19.5.2 |
| `0x040A` | `ERR_TREEKEM_STATE_DIVERGENCE` | Post-Commit ratchet-tree integrity check (§5.1) | A member's derived tree/epoch secret disagrees with the group's expected state after applying an agreed Commit sequence. | Yes (resync from Welcome/External Commit) | HALT_ALERT — treated as a potential integrity failure, not silently reconciled |
| `0x040B` | `ERR_DENIABLE_PREKEY_INVALID_OR_EXHAUSTED` | Deniable-mode prekey intake (§5.2.1, §18.4.8) | A `DeniablePrekeyBundle`'s `sig`/`spk_sig` fails to verify, or the identity's bundle is exhausted (no unspent one-time prekey and last-resort rate-limited out). | Yes (fetch a replenished bundle) | REJECT_NOTIFY — surface to the initiating client; fall back to non-deniable MLS 1:1 only with user consent (§5.2.1(d)) |
| `0x040C` | `ERR_DENIABLE_X3DH_FAILED` | Deniable-mode key agreement (§5.2.1(a)) | X3DH/PQXDH shared-secret derivation failed — an unknown/expired `spk_ref`/`opk_ref`/`kem_ref`, or a KEM decapsulation mismatch. | Yes (re-initiate with current prekeys) | REJECT_NOTIFY |
| `0x040D` | `ERR_DENIABLE_RATCHET_AUTH_FAILED` | Double-Ratchet message auth (§5.2.1(b), §18.9.10) | A `DeniableMessage` AEAD tag (the shared-key MAC) fails to verify, or the message is irrecoverably out of ratchet order (beyond MAX_SKIP, §16.9). | No (this message) | DROP_SILENT — a MAC failure reveals nothing to notify; skipped-key exhaustion holds for resync |
| `0x040E` | `ERR_DENIABLE_MODE_UNAVAILABLE` | Deniable-mode capability check (§5.2.1(d), §10.2) | The recipient has not advertised the `deniable-1:1` capability token, so a deniable session cannot be established. | Conditional (recipient may advertise it later) | REJECT_NOTIFY — the client MUST surface the choice (send non-deniable, or not at all); MUST NOT silently downgrade the user's expectation of deniability |
| `0x040F` | `ERR_DENIABLE_SIGNATURE_PRESENT` | `DeniablePayload` decode (§5.2.1(c), §18.3.10) | A `DeniablePayload` carries a signature field — its presence would make the transcript attributable and defeat the mode's whole purpose. | No | FAIL_CLOSED_BLOCK — reject the message; the deniable payload MUST be MAC-authenticated only, never signed |
| `0x0410` | `ERR_CLUSTER_DEVICE_UNAUTHORIZED` | Device-cluster sync membership check (§5.6.1, §18.6.3) | A `ClusterSyncFrame`/`ClusterOp` from a device whose `DeviceCert` is absent, invalid, or **revoked** (KeyRotation-excluded) under the owner's `IK` — not a current, non-revoked cluster member. Replication is mutually authenticated; a non-member cannot inject or pull cluster data. | No | FAIL_CLOSED_BLOCK — refuse the peer; do not exchange objects or ops with it |
| `0x0411` | `ERR_CLUSTER_RECON_SUMMARY_INVALID` | Range-based reconciliation summary check (§5.6.3(a), §18.6.3) | A `ClusterSyncFrame` `recon` summary is malformed, or a `RangeFingerprint.fp` does not recompute over the ids the receiver holds in the claimed `[lo, hi)` range — a peer serving forged Merkle fingerprints to suppress or misrepresent objects. | Yes (re-drive against another peer) | FAIL_CLOSED_BLOCK — MUST NOT trust the summary; reconcile against a different cluster member |
| `0x0412` | `ERR_CLUSTER_JOURNAL_CHAIN_BROKEN` | Journal-replay backfill chain check (§5.6.3(b), §18.6.3) | A per-account append-only journal presented for replay has a `prev` chain that does not verify — a fork or rewrite of the owner's **own** hash-chained log (the cluster analogue of a committer fork, `0x0404`, and KT append-only violation, `0x0110`). | No | HALT_ALERT — stop replaying the forked journal, alert the owner; fall back to range reconciliation (§5.6.3(a)) on the honest peers |
| `0x0413` | `ERR_CLUSTER_CRDT_OP_INVALID` | Cluster CRDT op validation (§5.6.4, §18.6.3) | A `ClusterOp` is malformed — unknown `kind`, an OR-Set remove citing an unknown add-tag, or an HLC whose `wall` is more than the clock-skew bound (§16.10) ahead of the receiver (a "win-forever" clock) — or it embeds a `DeniablePayload`/its plaintext (forbidden in the cluster CRDT, §5.2.1). | No | FAIL_CLOSED_BLOCK — reject the op; MUST NOT apply it to the CRDT state |
| `0x0414` | `ERR_MLS_CIPHERSUITE_DOWNGRADE` | MLS ciphersuite selection — Welcome / GroupInfo / Commit intake (§5.1) | A group handshake selects an MLS ciphersuite **below the group's MLS-ciphersuite high-water-mark**, or keeps/moves the group to a **classical** MLS ciphersuite when **every** current member advertises a PQ identity suite and a supported PQ MLS ciphersuite — a silent message-confidentiality PQ downgrade (harvest-now-decrypt-later), the MLS-ciphersuite analogue of `Envelope.suite` downgrade `0x020F`. Message-PQ rides the MLS ciphersuite (a separate u16), not `Envelope.suite`, so it is policed on its own axis (§5.1). | No | FAIL_CLOSED_BLOCK — reject the downgrading handshake; the high-water-mark ratchets up only, via a member-agreed retirement Commit, never an inbound handshake |
| `0x0415` | `ERR_RTC_SIGNAL_UNAUTHORIZED` | `RtcSignal` intake (§27.4.5) | `Payload.from` is not a current member of the MLS group under whose epoch key the carrying MOTE decrypted — evaluated against the receiver's own applied group state, never a claim in the signal. | Conditional (a pending Add the receiver has not yet applied may resolve it) | FAIL_CLOSED_BLOCK |
| `0x0416` | `ERR_RTC_CAPACITY_EXCEEDED` | SFU admission / renegotiation check (§27.7.4) | Admitting the participant, or accepting the renegotiation, would exceed a published `RtcCapacity` bound (`max_tracks`, `max_aggregate_bps`, `max_tracks_per_participant`, `max_bps_per_track`). A policy deny evaluated against published values, never against a live measurement. | Yes (a later call, a smaller track set, or another operator) | DENY_POLICY |
| `0x0417` | `ERR_RTC_SFRAME_REQUIRED` | Media protection check (§27.5.2, §27.6.5, §27.7.4) | Unprotected media was emitted or forwarded on a call that negotiated SFrame, a renegotiation would remove SFrame from an established call, or media keys were requested in order to process a track server-side. Protection ratchets up only — the media-layer analogue of `0x0414`. | No | FAIL_CLOSED_BLOCK |

DMTAP-RTC (§27) allocates these three points within this existing subsystem byte, under the
lighter-weight Specification-Required-within-an-existing-subsystem policy of §21.14 rather than
the Standards-Action new-subsystem-byte policy — the extension registration at §21.24f.
Individually defined here (rather than left to §27, as §21.24d/§25.12 does for `ERR_PUB_*`)
because they sit within a subsystem `0x04` this appendix already owns outright, unlike `0x09`
which DMTAP-PUB (§21.24b) reserves to itself.

## 21.7 Auth errors — DMTAP-Auth (`0x05xx`)

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0501` | `ERR_ORIGIN_MISMATCH` | Login ceremony origin check (§13.3.1) | The `rp_origin` bound into the signed challenge does not match the origin the trusted client actually observed. | No | FAIL_CLOSED_BLOCK — treat as a phishing attempt |
| `0x0502` | `ERR_NONCE_REPLAYED` | Challenge nonce check (§13.3, §16.1) | The single-use nonce has already been consumed. | No | FAIL_CLOSED_BLOCK |
| `0x0503` | `ERR_CHALLENGE_EXPIRED` | Challenge validity window (§13.3, §16.1: 120 s) | `exp` has passed, or the challenge falls outside the clock-skew-adjusted validity window. | Yes (issue a fresh challenge) | REJECT_NOTIFY |
| `0x0504` | `ERR_SESSION_REVOKED` | DPoP/GNAP session check (§13.4) | The session backing this request has been explicitly revoked. | No | REJECT_NOTIFY — RP denies; client must re-authenticate |
| `0x0505` | `ERR_SESSION_EXPIRED` | Session lifetime check (§13.4) | The session's lifetime has elapsed. | Yes (re-authenticate) | REJECT_NOTIFY |
| `0x0506` | `ERR_DPOP_PROOF_INVALID` | Proof-of-possession check (§13.4) | The DPoP/GNAP proof does not verify against the session's bound key, or is replayed. | No | FAIL_CLOSED_BLOCK |
| `0x0507` | `ERR_UNTRUSTED_APPROVAL_CHANNEL` | Remote-node login approval (§13.3.1) | The node cannot attribute the presented `rp_origin` to an authenticated, attributable request channel. | No | FAIL_CLOSED_BLOCK — the node MUST reject the challenge outright (consent-farming defence) |
| `0x0508` | `ERR_CAPABILITY_DELEGATION_INVALID` | Delegated-capability check (§13.5, §18.7.3) | The UCAN-style token is malformed, expired, forged, over-attenuated (the invoked right exceeds what was granted), or carries a caveat the verifier does not recognise. `FAIL_CLOSED_BLOCK`, **not** `DENY_POLICY`: §18.7.3 requires an over-attenuated grant and an unrecognised caveat key to **fail closed**, and §21.2 reserves `DENY_POLICY` for a non-security deny — a defective or authority-exceeding token is a security-validity rejection. Distinct from `0x050B` (`ERR_CAPABILITY_REVOKED`), which *is* `DENY_POLICY`: a validly-formed grant its issuer deliberately revoked is a non-adversarial policy decision. | No | FAIL_CLOSED_BLOCK — reject the invocation; never best-effort or partial-grant, and an unrecognised caveat key MUST fail closed (§18.7.3) |
| `0x0509` | `ERR_OIDC_ISSUER_MISMATCH` | OIDC bridge / self-issued discovery (§13.6) | The ID Token's issuer does not match the discovered/pinned issuer for the claimed identity. | No | FAIL_CLOSED_BLOCK |
| `0x050A` | `STATUS_SESSIONS_INVALIDATED_ON_RECOVERY` | `IK` recovery completion (§13.4) | All prior session authorisations are invalidated as a consequence of a completed identity recovery. | N/A | REJECT_NOTIFY — force re-authentication on every RP |
| `0x050B` | `ERR_CAPABILITY_REVOKED` | Delegated-capability invocation vs. revocation check (§13.5, §13.5.1, §18.7.3) | A structurally-valid `CapabilityToken` (or a chain ancestor) is covered by a published `CapabilityRevocation` from its issuer or an ancestor issuer — the capability was explicitly revoked. Distinct from `0x0508` (`ERR_CAPABILITY_DELEGATION_INVALID`, the token being malformed/expired/over-attenuated): `0x050B` is a *validly-formed but revoked* grant. | No | DENY_POLICY — deny the invocation; the delegatee must be re-granted. The revocation is KT-logged and owner-visible (§13.5) |

## 21.8 Gateway errors (`0x06xx`)

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0601` | `ERR_GATEWAY_ATTESTATION_INVALID` | Inbound `GatewayAttestation` verification (§7.2a, §18.3.11, §18.9.11; at `deliver` step 8a, §19.3.1) | The `GatewayAttestation` fails to verify: its signature over "received via gateway G at T" does not verify, `msg_digest` does not match the wrapped message bytes, or the `disc` is an unknown attestation kind. | No | DROP_SILENT at the recipient node (see honest limit, §21.9) |
| `0x0602` | `ERR_GATEWAY_ATTESTATION_KEY_UNTRUSTED` | Inbound attestation key lookup (§7.2a, §18.3.11) | The attestation key (`<selector>._dmtap-gw.<domain>`) is not published under the recipient's own domain's `_dmtap-gw` record, nor in an explicitly trusted set (incl. a chained entry under an untrusted `domain`, §7.8.3). | No | DROP_SILENT |
| `0x0603` | `ERR_DKIM_DELEGATION_INVALID` | Outbound DKIM verification at destination MTA (§7.3) | The gateway's delegated-selector DKIM signature fails to verify at the receiving legacy system. | Yes (fix key publication) | REJECT_NOTIFY — the sending node's queue retries per §7.4 |
| `0x0604` | `ERR_MX_TRANSIENT_FAILURE` | Outbound SMTP transaction (§7.3) | The destination MX rejects or times out the transaction transiently. | Yes | REJECT_NOTIFY — the sending node retries; the gateway holds nothing (§7.4) |
| `0x0605` | `ERR_GATEWAY_ALIAS_UNMAPPED` | Inbound legacy→native alias mapping (§7.10.2, §7.10.3, §18.3.12) | An inbound legacy message to a gateway alias cannot be mapped back to a native DMTAP address: a `random`-mode alias with no live `GatewayAliasMap` row (missing / expired / `burned`), or an `encoded`-mode local-part that does not decode to a valid `localpart@nativedomain`. The bridge owns no identity, so an unmappable alias is "no such user," not a silent drop. | No | RETURN_SENDER_SMTP — `550 5.1.1` (identical to the §21.9 non-existent-recipient reply, so it leaks nothing) |
| `0x0606` | `ERR_GATEWAY_ALIAS_ENCODING_INVALID` | Encoded gateway-alias reversibility check (§7.10.2, §18.3.12) | An `encoded` gateway alias (`localpart.nativedomain@gateway.domain`) is malformed: it does not reversibly decode to **exactly one** `(localpart, nativedomain)` (ambiguous/illegal escaping), or it exceeds the RFC 5321 local-part (64 octet) / path (254 octets — RFC 5321 §4.5.3.1.3's 256-octet forward-/reverse-path limit, less the two `<`/`>` bracket octets that DMTAP's bracket-free `alias@gateway.domain` form never carries) limits (§16.11). | No | FAIL_CLOSED_BLOCK — MUST NOT guess a native address from an ambiguous encoding |
| `0x0607` | `ERR_GATEWAY_SENDER_UNAUTHENTICATED` | Outbound legacy-egress admission (§7.3, §7.11.2, §9.10; `GatewayAuthz` §12.2, §18.8a.3, key-auth §7.12) | A DMTAP→legacy relay is attempted by a sender the gateway has **neither authenticated** (no live `GatewayAuthz` record, §18.8a.3 — no open/key-registered relationship, §7.12) **nor been paid by** (no valid redeemable postage, §9.5). A valid mesh `sender_sig` proves *who signed*, not *who may relay*, so it does not authorise egress — the open-relay-prevention floor (§7.7, §7.11.2). Distinct from `0x070E` (`ERR_GATEWAYAUTHZ_DENIED`, the operator-**unreachable** fail-safe): `0x0607` is the steady-state refusal of an unauthenticated/unpaid sender. | Yes (register with the gateway §7.12, or attach postage §9.5) | FAIL_CLOSED_BLOCK — MUST NOT relay; a gateway is never an open relay |
| `0x0608` | `ERR_GATEWAY_TLS_POLICY_UNMET` | SMTP TLS-policy enforcement, either leg (§7.2 inbound TLS, §7.3 outbound TLS) | The TLS posture in force for this leg cannot be met: **outbound**, the destination domain publishes MTA-STS (RFC 8461) in `enforce` mode or DANE TLSA (RFC 7672) and the session would proceed in cleartext or against a peer the policy does not validate; **inbound**, a session would negotiate cleartext where the gateway's own published policy promised TLS. Not a transport nicety — a downgraded leg is the one place a bridged message is readable in transit, and a gateway that proceeds anyway is misrepresenting the protection it applied (§7.3). | Yes (the peer's TLS may recover; the sending node's queue retries per §7.4) | FAIL_CLOSED_BLOCK — MUST NOT deliver the leg, and MUST NOT record or present it as TLS-protected (§7.3) |
| `0x0609` | `ERR_GATEWAY_SMTPUTF8_UNSUPPORTED` | Internationalized-mail capability check, either leg (§7.2b) | A message requires SMTPUTF8 (RFC 6531) — an EAI envelope address, or an 8-bit body where the peer lacks 8BITMIME and no lossless down-conversion exists — and the required capability is absent: **outbound**, the destination MX does not advertise it; **inbound**, this gateway does not advertise it and MUST therefore let the sending MTA bounce rather than accept an envelope it cannot carry faithfully. Never accept-then-mangle: silent corruption of what the bridge exists to carry is worse than a clean failure. | Yes (a peer that gains the capability, or a message that no longer requires it) | REJECT_NOTIFY — permanent for this message, surfaced to the sender via the §7.3/§7.4 failure report; MUST NOT emit a non-conformant 8-bit or EAI envelope |
| `0x060A` | `ERR_GATEWAY_SENDER_ADDRESS_UNAUTHORIZED` | Outbound legacy-egress per-address authorisation, §7.11.2 step 2; §18.8a.3 | A submitter that clears step 1 (authenticated to *this* gateway, §7.11.2 step 1) is nonetheless not authorised to claim the RFC 5322 `From:`/envelope `MAIL FROM` address the outbound message carries: the address does not resolve (§3.3) to an `IK` equal to the submitter's own, and no valid, unrevoked `CapabilityToken` referenced from the submitter's `GatewayAuthz.grants` (§18.8a.3, `Capability.resource = "gw-addr:"+address`) names the address for that `IK`. Distinct from `0x0607` (`ERR_GATEWAY_SENDER_UNAUTHENTICATED`, *who may relay at all*): `0x060A` is *which address an already-authenticated submitter may sign and send as* — a delegated DKIM selector (§7.3) authorises the gateway to sign for a domain, never any submitter to claim any address within it. | Yes (resolve the address to the submitter's own `IK`, or obtain an explicit per-address `CapabilityToken` grant, §18.8a.3) | FAIL_CLOSED_BLOCK — MUST NOT relay; a gateway is never a same-domain open relay for its own served identities |

### 21.8.1 Honest limit on `0x0601`/`0x0602`

Because the gateway is stateless and holds no queue (§7.4), and the attestation is checked by
the **recipient** after mesh delivery (potentially long after the inbound SMTP transaction has
completed), there is **no live channel back to the original legacy sender** when an attestation
fails post-delivery. This is a disclosed limit, not an oversight: the recipient's DROP_SILENT
is the correct action (never surface an unverified legacy-origin message as though it were
authenticated), but it cannot retroactively bounce the SMTP transaction that already closed.
Operators SHOULD treat repeated `0x0601`/`0x0602` from the same purported gateway as a
reputation signal (§7.5, §9.6).

## 21.9 SMTP response-code mapping (inbound, informative reference to RFC 5321 / RFC 3463)

The gateway's inbound leg (§7.2) is an ordinary SMTP transaction and MUST respond using
standard SMTP reply codes plus RFC 3463 enhanced status codes, so unmodified legacy MTAs
retry/bounce correctly. This table is not a new DMTAP registry — it maps DMTAP-level
conditions onto the existing IANA "SMTP Enhanced Status Codes" registry.

| DMTAP condition | SMTP reply | Enhanced status | Retry semantics |
|---|---|---|---|
| Recipient key/node unresolvable within the inbound transaction window | `451` | `4.4.1` | Transient — legacy sender's MTA retries per its own queue policy. |
| Recipient reachable but has **not durably `ack`ed** within the transaction window (best-effort buffer only, no durable custody) | `451` | `4.4.1` | Transient — the gateway MUST NOT reply `250` before a durable `ack` (§19.7.1 step 6); replying `451` keeps durability in the legacy sender's queue and prevents an un-notified post-`250` loss. |
| No `name → key` binding exists for the recipient at all | `550` | `5.1.1` | Permanent — no such user. |
| Pre-`DATA` anti-abuse reject: RBL/DNSBL, SPF/DMARC hard fail (§9, §7.2 step 2) | `550` | `5.7.1` | Permanent — delivery refused on policy grounds. |
| Pre-`DATA` greylisting / rate-limit (§7.2 step 2) | `450` | `4.7.1` | Transient — try again later. |
| Message exceeds size policy | `552` | `5.3.4` | Permanent. |
| Recipient node reachable but declines (recipient-side `Policy.block`, §9.2) | `550` | `5.1.1` | Permanent. **MUST use the identical code+enhanced status as "no such user" above** (`550 5.1.1`), so a blocked sender cannot distinguish a block from a non-existent address — closing the block-membership oracle. A distinct `5.7.x` here would itself leak that the recipient exists and has blocked this sender; the block is enforced downstream and never surfaced as its own SMTP signal. |

### 21.9.1 Why a stateless gateway defers rather than accepting-then-losing (DSN safety)

A store-and-forward MTA that returns `250` becomes responsible for the message and emits a **DSN
(bounce)** if it later fails to deliver. A DMTAP gateway holds **no queue** (§7.4), so it cannot
emit a later DSN — its only signal to the legacy sender is the live SMTP reply. Therefore the
gateway returns `250` **only after the recipient node durably `ack`s** (§19.7.1 step 6); until
then it replies `451` so the **legacy sender's own MTA** retains the message and will itself
bounce after its retry window. This deliberately keeps the accept/bounce responsibility on the
one party that can still act on it, and eliminates the otherwise-silent loss where a `250` is
followed by a mesh-side `EXPIRED` (§19.3.3) with no channel left to notify the sender. (Contrast
§21.8.1: a *post-delivery* attestation failure genuinely has no live channel back — there the
loss window is disclosed and irreducible; here it is fully closed by never `250`-ing early.)

## 21.10 Anti-Abuse & Postage errors (`0x07xx`)

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0701` | `ERR_CHALLENGE_MISSING_COLD_SENDER` | Cold-sender gate (§9.2, §2.7 step 6) | A cold sender's MOTE carries no `challenge` at all. | Yes (resend with a challenge) | DEFER_REQUESTS |
| `0x0702` | `ERR_CHALLENGE_BELOW_THRESHOLD` | Cold-sender gate (§9.2) | A `challenge` is present but does not meet the recipient's policy threshold. | Yes | DEFER_REQUESTS |
| `0x0703` | `ERR_CHALLENGE_FORGED_INVALID` | Challenge cryptographic verification (§2.7a) | The challenge fails cryptographic verification outright (forged/malformed). | No | DROP_SILENT |
| `0x0704` | `ERR_TOKEN_ISSUER_UNTRUSTED` | ARC token issuer trust check (§9.3.1) | The token's issuer is unknown/unvetted at this recipient (default rate budget = 0), including self-issued tokens. | N/A | FALLBACK_LOWER_TIER — re-evaluate under PoW/postage policy |
| `0x0705` | `ERR_TOKEN_INVALID_OR_SPENT` | ARC token verification/redemption (§9.3) | Token signature invalid, or the token's rate budget is already exhausted. | No (this token) | DROP_SILENT or DEFER_REQUESTS per local policy severity |
| `0x0706` | `ERR_POW_INSUFFICIENT` | Proof-of-work check (§9.4) | Attached PoW solution is below the recipient's required difficulty. | Yes (resolve at correct difficulty) | DEFER_REQUESTS |
| `0x0707` | `ERR_POSTAGE_ISSUER_UNTRUSTED` | Postage issuer trust check (§9.5.1, parallel to §9.3.1) | The postage issuer is unvetted/untrusted at this recipient. | N/A | FALLBACK_LOWER_TIER |
| `0x0708` | `ERR_POSTAGE_DOUBLE_SPEND` | Postage redemption (§9.5.1) | The stamp's `serial` has already been redeemed. | No | REJECT_NOTIFY at redemption; the underlying MOTE falls back to remaining anti-abuse policy (token/PoW) |
| `0x0709` | `ERR_POSTAGE_ISSUER_UNREACHABLE` | Postage online redemption check (§9.5.1 failure mode) | The issuer's redemption endpoint (or signed spent-list) is unreachable at redemption time. | Yes | FALLBACK_LOWER_TIER — never accepted on faith |
| `0x070A` | `ERR_VOUCH_INVALID_OR_RATE_LIMITED` | Vouch/introduction check (§9.7) | The vouch token is invalid, or vouch issuance has exceeded its own anti-farming rate limit. | Yes (obtain a valid vouch) | DEFER_REQUESTS |
| `0x070B` | `ERR_POLICY_DENY_BLOCKLIST` | Recipient `Policy.block` (§9.2) | Sender's key/token matches an explicit block entry. | No | DROP_SILENT — blocked senders receive no distinguishing signal, by design |
| `0x070C` | `ERR_RATE_LIMIT_EXCEEDED` | Recipient `Policy.rate` (§9.2) | Per-sender-token rate limit exceeded. | Yes, after the window resets | DEFER_REQUESTS below a hard cap; DROP_SILENT beyond it |
| `0x070D` | `ERR_QUOTA_POLICY_DENY` | Operator `Policy` capability, operations only (§12.2) | A hosted-operator storage/send/domain-count quota is exceeded. Self-host default is unlimited (§12.2, §12.3 — this MUST NOT be a security or crypto gate). | Yes (quota reset / plan change) | DENY_POLICY |
| `0x070E` | `ERR_GATEWAYAUTHZ_DENIED` | Operator `GatewayAuthz`, operator-unreachable safe default (§12.2, §18.8a.3) | Legacy egress for a cold/unproven sender is denied during an operator outage (fail-safe, never fail-open for this specific capability). **A security control (§9), not a housekeeping limit** — §12.2 states `GatewayAuthz` MUST NOT fail open, because denying it is exactly what prevents unattributable open-relay egress (§7.7). This is why the action is `FAIL_CLOSED_BLOCK`, not `DENY_POLICY` (§21.2 reserves `DENY_POLICY` for a non-security policy deny — that is `0x070D`, the quota sibling). It is the operator-**unreachable** half of the open-relay floor whose steady-state half is `0x0607`. | Yes (once the operator is reachable, or with self-contained proof) | FAIL_CLOSED_BLOCK — MUST NOT relay cold/unproven legacy egress during the outage; permit only established contacts + operator-independent PoW (§12.2) |
| `0x070F` | `ERR_POLICY_BELOW_FLOOR` | Recipient anti-abuse **policy validation** (§9.7a, [docs/research/vdf.md §9.4.1](docs/research/vdf.md), §16.5) | The recipient's own policy sits **below the zero-relationship delivery floor**: a standing configuration under which a valid work proof is **never sufficient** (§9.7a — note the floor is a policy constraint, not a per-`sender_key` count, since that key is ephemeral by §2.2 and would bound nothing), or a cold-contact requirement that accepts **only** a VDF — an unstandardized and non-post-quantum construction ([docs/research/vdf.md §9.4.1](docs/research/vdf.md)) — and would therefore refuse a valid memory-hard PoW (§9.4). Unlike every other code in this block, the fault is in the **local configuration**, not in an inbound object — the condition is detected when the policy is applied, not when a MOTE arrives, so a node MUST NOT start (or MUST NOT commit the change) carrying it. Normative because each recipient's local incentive is to set the floor to zero, and the aggregate is a network only the already-connected can enter (§9.7a). | Yes (raise the floor to ≥ the §16.5 minimum, or accept memory-hard PoW) | REJECT_NOTIFY — refuse the policy and surface it to the operator/user; MUST NOT silently clamp, and MUST NOT apply the sub-floor policy while reporting conformance |

## 21.11 Files errors (`0x08xx`)

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0801` | `ERR_MANIFEST_HASH_MISMATCH` | Manifest integrity check (§5.5) | `Manifest.id` does not match the recomputed BLAKE3 Merkle root over `chunks`. | No | DROP_SILENT — do not begin fetch |
| `0x0802` | `ERR_CHUNK_HASH_MISMATCH` | Per-chunk integrity check (§5.5) | A fetched chunk fails to verify against its listed hash. | Yes (re-fetch from a different swarm holder) | ROTATE_RETRY |
| `0x0803` | `ERR_CHUNK_UNAVAILABLE` | Swarm fetch (§5.5.3 repair; §5.5.1 tiers) | No current holder serves a required chunk. Whole-file unavailability is `0x0809`. | Yes (best-effort tier) | ROTATE_RETRY; REJECT_NOTIFY once retry budget is exhausted |
| `0x0804` | `ERR_SIZE_TIER_MISMATCH` | Size-tier classification (§2.5, §6.5) | The declared/observed size does not match the tier (inline/normal/large) the MOTE was routed under. | No | DENY_POLICY — refuse the mis-tiered MOTE; MUST NOT silently downgrade the privacy tier to compensate. (A single action per the §21.2/§21.26 invariant; DENY_POLICY, not DROP_SILENT, because a size/tier disagreement is a policy reject with no metadata-privacy reason to hide it from the sender.) |
| `0x0805` | `ERR_FILE_KEY_MISSING_OR_REVOKED` | Post-removal file access (§6.7) | A former group member's file key has been rotated out after removal from a shared folder. | No | DENY_POLICY — the member must be re-added and re-shared |
| `0x0806` | `ERR_STORAGE_QUOTA_EXCEEDED` | Operator storage `Policy` (§12.2) | Hosted-operator storage cap reached. Self-host default is unlimited. | Yes | DENY_POLICY |
| `0x0807` | `ERR_ATTACHMENT_KEY_INVALID` | Attachment/chunk decryption (§2.5) | The per-file content `key` fails to decrypt the referenced inline blob or chunk. | No | DROP_SILENT |
| `0x0808` | `ERR_MANIFEST_KEY_PRESENT` | Manifest decode (§5.5, §18.3.8) | A received `Manifest` carries an embedded content key (reserved-forbidden field `5`). A manifest is a swarm-distributed blob, so an embedded key would leak to every holder that serves it. | No | FAIL_CLOSED_BLOCK — reject the manifest, MUST NOT use the embedded key; the legitimate key travels only in `Attachment.key` inside the sealed MOTE (§18.3.7) |
| `0x0809` | `ERR_FILE_UNAVAILABLE` | Referenced-file fetch, durability contract (§5.5.2, §5.5.3) | The **whole file** has no reachable holder and no durability contract can be satisfied — the disclosed **origin-hold** residual realized (the origin dropped before the recipient fetched), as distinct from a single missing chunk (`0x0803`). | Yes, while any holder may still appear | REJECT_NOTIFY — surface to the user that a best-effort (origin-hold) referenced file is gone; closed prospectively by pinning/replicating (§5.5.2, §6.6 item 10) |
| `0x080A` | `ERR_FILE_MANIFEST_INVALID` | `ManifestRef.durability` validation (§5.5.2, §18.3.7) | A **Referenced** file's `ManifestRef` is missing its REQUIRED `durability`, carries an **unknown** `class`, a `cluster-replicated` (`class = 2`) with `replicas < 1`/absent, or a `pinned` (`class = 3`) with no `retention` — a malformed/underspecified durability contract. | No | FAIL_CLOSED_BLOCK — reject; MUST NOT treat an unspecified contract as durable, nor silently downgrade to best-effort |
| `0x080B` | `ERR_FILE_RETENTION_EXPIRED` | `pinned(term)` retention check (§5.5.2, §5.5.4, §16.4) | A `pinned` (`class = 3`) contract's `retention` term has elapsed (or a holder is asked to serve past its committed retention); the pin is no longer honoured and the bytes MAY have been GC'd. | Yes, if re-pinned before GC | REJECT_NOTIFY — renew/re-pin before expiry (§5.5.2); a retention-expiry race is closed by renewing ahead of the term |
| `0x080C` | `ERR_SPOOL_OVERFLOW` | Inbound push admission, spool cap (§5.5.5, §16.4) | A **pushed** Inline/Attached file would exceed the recipient's inbound spool cap for that sender — a storage-based DoS (spool-fill) attempt. Refused, never silently accepted or silently dropped. | Yes, after the recipient frees space / the sender proves legitimacy | DENY_POLICY — fail closed; an unproven/cold sender's pushes also spend the requests-area + anti-abuse budget (§2.7a, §9, §16.5) |
| `0x080D` | `ERR_FILE_SIZE_TIER_VIOLATION` | `Attachment` delivery-mechanism validation (§5.5.1, §18.3.7) | An `Attachment`'s declared delivery mechanism/size is internally inconsistent: an **oversize `inline`** (the inlined bytes exceed the §16.4 inline ceiling), **both or neither** of `inline`/`manifest` present (the §18.3.7 "exactly one of {`inline`,`manifest`} MUST be present" invariant broken), or a **`size` that disagrees** with the `inline` byte length / `ManifestRef.size`. Distinct from `0x0804` (`ERR_SIZE_TIER_MISMATCH`), which concerns the **MOTE's routed privacy tier**; `0x080D` is the **attachment record disagreeing with itself**. | No | FAIL_CLOSED_BLOCK — reject the attachment; MUST NOT guess the intended mechanism nor silently re-tier |

## 21.11a Legacy Adapters errors (`0x0Bxx`, §26)

Subsystem byte `0x0B` is reserved to the DMTAP Legacy Adapters extension (§26 — the generalisation
of the §7 SMTP/mail-bridge pattern to other legacy rails: SMS, WhatsApp, Telegram, Discord, Slack)
by the extension registration at §21.24g. Unlike DMTAP-PUB (`0x09`, §21.24b) and the Sync substrate
capability (`0x0A`, §21.24c), whose individual code points are defined in their own owning
documents, the three code points below are defined directly in this Part 1, at the request of §26
itself (§26.11: "these are proposals for the §21 owner to adopt, amend, or reject, not
registrations this document performs"). A legacy adapter is a `gateway`-kind coordinator (CONTRACT
§5), so the objects the three codes below check are the general `CoordinatorDescriptor`-family
objects of §18.8a, not a bespoke adapter-only shape (§21.24h retires the earlier reserved-but-
undefined `DMTAP-ADAPT-v0/…` tags in favour of the `DMTAP-COORD-v0/…` ones §18.8a defines).

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0B01` | `ERR_ADAPTER_TARIFF_INVALID` | Signed tariff verification (`Tariff`, §18.8a.1; §26.10, §26.11) | A `Tariff` fails to verify under the publishing adapter operator's key, or is presented past its stated validity window — the adapter analogue of a stale/forged price list. | Yes (fetch a current, validly-signed `Tariff`) | FAIL_CLOSED_BLOCK — MUST NOT bill, or let a client compare operators, against an unverified or expired tariff |
| `0x0B02` | `ERR_ADAPTER_RECEIPT_INVALID` | `UsageReceipt` verification (§18.8a.2; `system`-kind body, `kind = 0x0A`, §21.16; §26.10, §26.11) | A `UsageReceipt` fails to verify under the billing adapter operator's key, or its claimed accounted usage does not match the metered rail activity it purports to receipt — the paying user's audit trail (§26.2.1 item 3) is broken. | No (this receipt) | FAIL_CLOSED_BLOCK — MUST NOT charge against, or present as auditable, an unverified receipt |
| `0x0B03` | `ERR_ADAPTER_CREDENTIAL_UNAUTHORIZED` | Outbound per-rail egress admission, gateway-mode adapter (§26.2.1 item 1; `GatewayAuthz` §18.8a.3) | An adapter running in gateway mode attempts to relay a mesh MOTE onto a legacy rail (SMS/WhatsApp/Telegram/Discord/Slack) for a sender it has **neither authorised** (no `GatewayAuthz` record, or no valid, unrevoked per-rail `CapabilityToken` grant naming that identity for that remote-facing number/handle/account, §18.8a.3) **nor been paid by** (no valid redeemable postage/usage credit) — the direct per-rail analogue of `ERR_GATEWAY_SENDER_UNAUTHENTICATED` (`0x0607`) and `ERR_GATEWAY_SENDER_ADDRESS_UNAUTHORIZED` (`0x060A`), generalised from mail to every rail §26 covers. | Yes (register a per-rail authorisation grant, or attach valid postage/usage credit) | FAIL_CLOSED_BLOCK — MUST NOT relay; an adapter in gateway mode is never an open relay on any rail |

## 21.12 Traceability matrix

Every condition explicitly named in this appendix's brief maps to a code as follows, for
auditability. **This matrix is the canonical MUST→code linkage (normative):** many codes are
defined here in §21 and cited at their point of detection in §§1–17, but where a normative
condition elsewhere does not name its code inline, **this table is the authoritative cross-index**
between the condition and its `0xSSNN` code, and the conformance suite (§10.3) references it as the
condition↔code index. A registered code is therefore never orphaned: it is reachable either from
its defining clause or from this matrix.

| Named condition | Code(s) |
|---|---|
| Unknown version/suite (fail-closed) | `0x0201`, `0x0101` |
| Suite downgrade / hybrid strip | `0x020F` (below high-water-mark), `0x0210` (intra-suite hybrid strip) |
| MLS ciphersuite downgrade (message-PQ, policed separately from `Envelope.suite`) | `0x0414` |
| Envelope `kind`/`ts`/`to` altered after payload signed | `0x0211` |
| Cross-author `edit`/`redact` (same-author gate, §2.7 step 9) | `0x0212` |
| Bad content-address | `0x0202` |
| Bad `sender_sig` | `0x0203` |
| Decrypt-failure | `0x0207` |
| Unresolved-to | `0x0204` |
| Cold-sender-no-challenge | `0x0701` (canonical), cross-referenced at `0x0206`/§2.7 step 6 |
| Insufficient-challenge | `0x0702` |
| KT-unreachable-fail-closed | `0x0106` |
| KT-equivocation-detected | `0x0107` (split-view conclusion), `0x0110` (append-only/consistency-proof evidence) |
| KT-v1-log-quorum-unmet | `0x0111` |
| KT-v1-STH-stale (freeze attack) | `0x0112` |
| Chain-broken | `0x0104` |
| Key rotation unauthorised (stolen-`IK` / `recover_threshold`-only takeover defence) | `0x0121` |
| Stale/rollback record | `0x0105` (identity), `0x0302` (location record) |
| Org-directory not authority-signed | `0x0113` |
| Directory entry fails forward-verify | `0x0114` |
| Org-managed custody undisclosed | `0x0115` |
| Self-asserted alias fails forward-verify | `0x011C` (identity's own `Identity.names`), `0x0114` (org directory's assertion) |
| Alias revoked (independently-revocable, other aliases unaffected) | `0x011D` |
| Name-chain bidirectional key↔name binding disagrees (within one resolution) | `0x011E` |
| Resolver type unimplemented/unregistered (unknown ⇒ reject, never guess) | `0x011F` |
| Multi-resolver disagreement (two resolvers, different keys, same name) | `0x0120` |
| Name label mixes Unicode scripts (single-script-per-label, homograph defence) | `0x0122` |
| Name confusable-skeleton collides with an already-pinned contact | `0x0123` |
| Device well-attested but not §1.4-policy-authorised | `0x0124` |
| Committer-fork-detected | `0x0404` |
| Committer-unreachable | `0x0405` |
| Deniable prekey invalid/exhausted | `0x040B` |
| Deniable X3DH/PQXDH failed | `0x040C` |
| Deniable ratchet MAC failed | `0x040D` |
| Deniable mode not advertised | `0x040E` |
| Deniable payload carries a signature (forbidden) | `0x040F` |
| Cluster-sync peer is not a non-revoked device | `0x0410` |
| Cluster reconciliation summary forged/malformed | `0x0411` |
| Cluster journal hash-chain broken (own-log fork) | `0x0412` |
| Cluster CRDT op invalid (bad HLC / unknown add-tag / deniable-embed) | `0x0413` |
| Device hardware-attestation invalid | `0x0116` |
| Device attestation evidence expired / root retired | `0x0118` |
| KT inclusion-proof leaf-hash mismatch | `0x0117` |
| Profile signature invalid (self-asserted display data) | `0x0119` |
| Profile avatar bytes mismatch signed content-address | `0x011A` |
| Profile avatar URL unsafe (non-https / SSRF internal target) | `0x011B` |
| Capability delegation invalid (malformed/expired/over-attenuated) | `0x0508` |
| Capability revoked (valid grant, explicitly revoked) | `0x050B` |
| Session-revoked | `0x0504` |
| Session-expired | `0x0505` |
| Origin-mismatch (auth) | `0x0501` |
| Replay-detected | `0x0502` (auth nonce — the canonical replay instance; MOTE-level replay is structurally absorbed by content-addressed dedup, `0x020E`) |
| Expired | `0x020B` (MOTE `expires`), `0x0503` (auth challenge) |
| Quota/policy-deny | `0x070D`, `0x0806` (operator quotas), `0x070B` (recipient blocklist deny) |
| Gateway-attestation-invalid | `0x0601` |
| Gateway alias unmappable (legacy→native, no such user) | `0x0605` |
| Gateway alias encoding non-reversible/over-length | `0x0606` |
| Gateway outbound open-relay refused (unauthenticated/unpaid sender) | `0x0607` (steady-state); `0x070E` (operator-unreachable fail-safe — same open-relay floor, both `FAIL_CLOSED_BLOCK`) |
| Gateway outbound per-address authorisation unmet (authenticated submitter, unauthorised claimed address) | `0x060A` |
| Gateway TLS policy unmet (MTA-STS `enforce` / DANE outbound, own advertised posture inbound) | `0x0608` |
| Gateway internationalized-mail capability absent (SMTPUTF8 / 8BITMIME, never accept-then-mangle) | `0x0609` |
| Gateway inbound cold legacy sender gated (bidirectional floor, §7.11.1, §9.10) | `0x0701`/`0x0702` (cold-sender gate); SPF/DKIM/DMARC hard-fail → SMTP `550 5.7.1` (§21.9) |
| Postage-double-spend | `0x0708` |
| Issuer-untrusted | `0x0704` (token issuer), `0x0707` (postage issuer), `0x0509` (OIDC issuer) |
| Recipient policy below the zero-relationship delivery floor (work proof never sufficient, or VDF-only) | `0x070F` |
| Capability-announcement rollback | `0x030A` |
| Cached mix directory fails independent verification against the KT log quorum | `0x030B` (there is no directory authority, [docs/research/mixnet.md §4.4.2](docs/research/mixnet.md); a log split-view over the mix set is `0x0107`) |
| Mix directory stale / frozen (freeze attack) | `0x0311` |
| Mix descriptor / key epoch stale | `0x030C` |
| Mix path unbuildable / min-viable-path unmet | `0x030D` (no path), `0x0310` (viable-path refused, no downgrade) |
| Mix packet replay | `0x030E` |
| Mixnet active-attack inferred (loop-cover) | `0x030F` |
| Push subscription not authenticated to device | `0x0312` |
| WakePing carries plaintext content/sender (forbidden) | `0x0313` |
| WakePing unauthenticated / AEAD open failed | `0x0314` |
| WakePing rate-limited (battery abuse) | `0x0315` |
| WakePing replayed by relay (nonce already seen) | `0x0316` |
| Referenced file permanently unavailable (origin-hold residual) | `0x0809` (whole file), `0x0803` (single chunk) |
| File durability contract missing/malformed (Referenced tier) | `0x080A` |
| Pinned-term retention expired / retention-expiry race | `0x080B` |
| Spool overflow (pushed-attachment storage DoS) | `0x080C` (push admission), `0x0806` (hosted quota) |
| Attachment delivery mechanism/size self-inconsistent (oversize inline; both/neither of inline+manifest; size mismatch) | `0x080D` |

---

# Part 2 — IANA Considerations

## 21.13 Overview and policy vocabulary

DMTAP intends to pursue IETF standardisation (§10.5). Upon submission of the governing
Internet-Draft, the registries below SHOULD be requested from IANA under "DMTAP Parameters."
Until then, **this appendix is the interim authoritative registry**, maintained per the
extension procedure in §21.25. Allocation policies use the standard terms of RFC 8126:

- **Standards Action** — requires a new RFC on the standards track.
- **Specification Required** — requires a stable, publicly available specification and review
  by a Designated Expert (§21.25), but not a new RFC.
- **First Come First Served (FCFS)** — no review; allocated on request.
- **Private Use** — not registered at all; meaningful only within a closed deployment or as a
  vendor extension, and MUST be namespaced (e.g. `x-` prefix) to avoid accidental collision.

## 21.14 DMTAP Error/Status Code Registry

| | |
|---|---|
| **Registry name** | DMTAP Error/Status Codes |
| **Reference** | §21.1–§21.11 (this document) |
| **Allocation policy** | New subsystem byte (`0x09`–`0xEF`): Standards Action. New code point within an existing subsystem (`NN` = `0x01`–`0x7F`): Specification Required. `NN` = `0x80`–`0xFE` within any subsystem: Private Use (implementation-local diagnostics; MUST map to the nearest standard code's Responder Action, §21.2, for any behaviour visible to another implementation). `SS`/`NN` = `0x00` or `0xFF`: Reserved. |
| **Initial contents** | The 144 codes enumerated in §21.3–§21.11. |
| **Registry discipline** | Append-only. A retired code MUST be marked Deprecated, never deleted or reassigned to a different meaning (mirroring the append-only philosophy of the KT log, §3.5). |

## 21.15 Algorithm Suites Registry (`suite` u8)

| | |
|---|---|
| **Registry name** | DMTAP Algorithm Suites |
| **Reference** | §1.1, §16.7 |
| **Allocation policy** | `0x01`–`0x1F`: Standards Action (a suite changes the network's core interoperability and security floor — every conforming node must be able to reject or accept it correctly). `0x20`–`0xDF`: Specification Required. `0xE0`–`0xFE`: Private Use. `0x00`, `0xFF`: Reserved. |
| **Initial contents** | `0x01` (Ed25519 / X25519-HPKE / ChaCha20-Poly1305 / BLAKE3-256, **LEGACY** — verify only, MUST NOT originate, §1.1); `0x02` (Ed25519+ML-DSA-65 / X-Wing hybrid / ChaCha20-Poly1305 / BLAKE3-256, the **v0 REQUIRED originating suite**, §1.1); `0x03` (Ed25519+ML-DSA-65 / X-Wing hybrid / **AES-256-GCM** / BLAKE3-256, RESERVED, **AEAD-diverse emergency target**, §1.1); `0x04` (Ed25519+**SLH-DSA-128s** / X-Wing hybrid / ChaCha20-Poly1305 / BLAKE3-256, RESERVED, **signature-diverse emergency target** and the intended anchor profile, §1.1, §1.2.0); `0x05` (Ed25519+ML-DSA-65 / X-Wing hybrid / ChaCha20-Poly1305 / **SHA3-256**, RESERVED, **hash-diverse emergency target**, §1.1). The status labels here are §1.1's normative ones; an earlier draft of this row carried the reference implementation's *support* status instead and read as though `0x01` were required and `0x02` reserved (§16.7). |
| **Registration requirements** | A new suite registration MUST specify: signature scheme, KEM/PKE, AEAD, hash function, and a security analysis of the combination; MUST state its MLS-ciphersuite mapping if it is to be usable for group messaging (§5.1); MUST NOT be accepted by any conformant node until published here (unknown suites are rejected fail-closed, §1.1, §10.1 — there is no silent negotiation fallback beyond the documented suite-intersection rule, §1.3). **AEAD agility is whole-suite-granular:** a suite fixes its AEAD alongside its signature/KEM/hash, with no independent AEAD selector, so an AEAD break is answered by migrating to a suite with a different AEAD (e.g. `0x01`/`0x02` ChaCha20-Poly1305 → `0x03` AES-256-GCM) via the multi-suite mechanism (§1.3), not by swapping a primitive in place (§1.1). **Hash agility is whole-suite-granular on the same terms:** a `hash` field's §18.1.5 prefix is self-description and a redundancy check, **not** an independent selector — the suite decides (`ERR_HASH_ALG_MISMATCH`, `0x0127`) — so a hash break is answered by migrating to a suite with a different hash (`0x02` BLAKE3-256 → `0x05` SHA3-256), and a registration proposing a hash from an already-represented design family MUST say why the network gains diversity by it (§1.1). |

## 21.16 Message Kinds Registry (`kind` u8)

| | |
|---|---|
| **Registry name** | DMTAP Message Kinds |
| **Reference** | §2.3, §10.1 |
| **Allocation policy** | `0x00`–`0x0B`: assigned (below). `0x0C`–`0x3F`: Unassigned — Standards Action (reserved for future *core* kinds sharing the wire-format guarantees of the initial set). `0x40`–`0x7F`: Specification Required (the extension range named in §2.3/§10.1). `0x80`–`0xFE`: Private Use. `0xFF`: Reserved. |
| **Initial contents** | `0x00 mail`, `0x01 chat`, `0x02 reaction`, `0x03 edit`, `0x04 redact`, `0x05 file_offer`, `0x06 group_event`, `0x07 receipt`, `0x08 presence`, `0x09 identity`, `0x0A system`, `0x0B deniable` (deniable 1:1 transport frame carrying a `DeniableFrame`, §5.2.1; the real content kind rides inside the `DeniablePayload`, §18.3.10). Extension range: `0x40 pub_announce` (§22, §21.24b); `0x41 feed_hint`, `0x42 feed_subscribe`, `0x43 feed_unsubscribe` (DMTAP-PUBSUB, §25, §21.24d); `0x44 rtc_signal` (DMTAP-RTC, §27, §21.24f) — an **ordinary sealed-MOTE kind**, `Payload`-wrapped, riding the existing deliver/ack/retry path (§2.6), the next point after `0x43`, unlike `0x40`'s bare signed object (§22.3.2). |
| **Forward-compatibility rule** | A node MUST NOT `ack` a `kind` it cannot validate, whether unassigned or merely unimplemented (§10.1, `0x020A`). It MAY ignore such a MOTE without error. This is the single rule that lets new kinds roll out without a flag day. |

## 21.17 Challenge Types Registry

| | |
|---|---|
| **Registry name** | DMTAP Anti-Abuse Challenge Types |
| **Reference** | §9.2 (`ChallengeSpec`), §2.2b |
| **Allocation policy** | A challenge type is identified by a **`u8` type tag** (fixed encoding, not conditional). `0x00`–`0x3F`: assigned / Specification Required — a new challenge type MUST specify its verification procedure (verifiable without decrypting the payload, per §2.2b), its issuer-trust model (per the §9.3.1 zero-default-budget rule, so a new type cannot bypass "cost for cold contact"), and its interaction with the §2.7a disposition (invalid/forged vs. absent/insufficient). `0x40`–`0xFE`: **Private Use** (unconditional). `0x00`/`0xFF` MAY be reserved by a registration; the four initial types occupy the low assigned range. |
| **Initial contents** | `pow(bits)` (§9.4); `token(issuer)` — ARC-style anonymous rate-limited credential (§9.3); `stamp(amount)` — postage (§9.5); `vouch(guardianSet)` (§9.7). |

## 21.18 Identity Resolver Types Registry

| | |
|---|---|
| **Registry name** | DMTAP Identity Resolver Types (the pluggable resolution framework, §3.12; supersedes the earlier "Name Backends" framing by generalizing it to cover the zero-authority `self`/`petname` types as well as the lookup-based backends) |
| **Reference** | §3.12 (framework), §3.1, §3.3, §3.6, §3.9.2, §3.13 |
| **Allocation policy** | Specification Required. Each registration MUST state the four things of §3.12.2 and **nothing that touches identity**: **(i)** the resolver-type tag; **(ii)** the **name form** — how a name in this type is written and whether it carries an `@namespace` (§3.13); **(iii)** discovery — how step 1 obtains the `name → ik` pointer such that only `IK` can update the binding; **(iv)** KT anchoring — how the binding is verified against key transparency (§3.5) and composes with resolution order (§3.3), or, for a derivational type, why verification is vacuous. A resolver type MUST NOT introduce its own trust root (authenticity is always the key, §3.12.1). |
| **Initial contents (name form ⇒ type)** | `self` — the **key-name** (§3.9.6): a bare **8-word** name derived as `BLAKE3-256(ik)`, **no `@`**, no namespace; discovery is a local derivation and KT verification is vacuous (the binding *is* the key); the always-available zero-authority floor. `petname` — a **local**, bare label a user assigns to an already-pinned contact (§3.9.3), **no `@`**, no global lookup; verification vacuous (bound to a pinned key). `dns` — the default, recommended type (§3.2): `local@domain` (`alice@provider.com`), discovered via a `_dmtap` TXT/SVCB record, forward-verified in KT (§3.3, §3.5). `name-chain` — OPTIONAL crypto name (§3.6, §3.12.5): a dotted-TLD `local@.eth` / `.sol` (ENS/SNS registered today; `.hns` etc. by future registration), discovered by reading the chain's on-chain `name → ik` record, bound **bidirectionally** and KT-audited (§3.12.5(b)). `directory` — the opt-in flat `@handle` registry (§3.9.2), where the leading `@` **is** the namespace marker; `handle → ik` in a KT-audited log. |
| **Note** | Onboarding Tiers A/B/C (§3.8) and the provisioning tiers 0–3 (§3.11.2) are deployment postures over the `dns` type, not separate resolver types, and are not separately registered here. `self` and `petname` are the two **zero-authority** types (no `@`, no lookup, no registration) that make identity and delivery need no naming system at all (§3.13). |
| **Registration is not endorsement (normative)** | The **naming ladder** (§3.13.2) is `self` → `name-chain` → `dns`: DMTAP's identity substrate is **DNS + cryptographic names only**. `directory` (`@handle`, §3.9.2) remains **registered and resolvable but is NOT part of the ladder** and is in no conformance level — a DMTAP-created global handle authority is a new arbitrating power this protocol declines to bring into existence. The registry stays **open** precisely so that a naming system DMTAP does not endorse can still be resolved without changing identity, delivery, or verification (§3.12.2); an implementation that does not support a registered type treats such a name as *undiscovered* (`ERR_RESOLVER_TYPE_UNSUPPORTED`, `0x011F`), never as invalid. |

## 21.19 KT Log Identification Registry

| | |
|---|---|
| **Registry name** | DMTAP Key-Transparency Log Types |
| **Reference** | §3.5, DNS `kt=` parameter (§3.2) |
| **Allocation policy** | Specification Required. A log identifies itself by its own signing key and a **log-type tag** from this registry (no central log-ID authority is needed beyond the tag identifying *how to speak to* the log — the log's identity is its key, consistent with the rest of DMTAP). |
| **Initial contents** | `0x01` — **v0-minimal** (§3.5.1, the interoperable Core default): a single append-only Merkle log; signed tree heads (STHs); inclusion proofs; rollback defence (§3.3 step 2); owner self-monitoring (STH poll, §16.2); **no gossip and no federation** — tamper-evident and self-monitorable, but *not* equivocation-proof (§6.6 item 6). `0x02` — **v1-hardening** (§3.5.2, capability-negotiated per §10.2, **not** the default): a **federated set of independent, per-key append-only logs**, each publishing STHs at least every maximum merge delay (§16.2). A verifier pins a *set* and accepts a `name → ik` binding only on a **`> n/2` quorum** of that set (§3.5.2(b), mirroring the §5.1/§16.8 roster quorum), so no single log is authoritative. Verifiers, **monitors** (per-identity), and **auditors** (per-log) **gossip STHs** and cross-check **consistency proofs** (§3.5.2(a),(c)); a split view / append-only violation / stale-frozen head is detected and produces `ERR_KT_EQUIVOCATION` (`0x0107`), `ERR_KT_STH_INCONSISTENT` (`0x0110`), `ERR_KT_LOG_QUORUM_UNMET` (`0x0111`), or `ERR_KT_STH_STALE` (`0x0112`), with the mandated HALT_ALERT / fail-closed response of §3.5.2(d). A verifier MUST implement `0x01` and MAY additionally negotiate `0x02`; a node offered only `0x02` by a peer that itself supports only `0x01` uses `0x01`. |
| **Registration requirements** | A new log type MUST specify its proof format (STH structure, inclusion and consistency proofs — which MUST remain independently auditable per §3.5) and MUST NOT weaken the append-only, tamper-evident guarantee that makes KT meaningful. A log type that claims equivocation resistance MUST specify its gossip and cross-checking procedure and the responder actions for a detected split view (as `0x02` does via §3.5.2). |

## 21.20 Envelope/Payload Extension Header Keys Registry (`Headers.ext`)

| | |
|---|---|
| **Registry name** | DMTAP `Headers.ext` Keys |
| **Reference** | §2.4 (`ext: { * tstr => any }`), §10.1 |
| **Allocation policy** | Two namespaces. **Unprefixed keys** (e.g. a future `priority`, `delegate`): Specification Required — intended for genuinely interoperable, cross-implementation semantics. **`x-`-prefixed keys**: Private Use / FCFS — no registration required, reserved for experimentation and vendor-specific metadata, and MUST NOT be assumed portable across implementations. |
| **Initial contents** | None assigned at v0. Candidate future registrations flagged by the feature-parity audit (§17.6) but **not yet registered**: a delegate-attribution marker ("sent/edited by delegate device X of identity Y," items 23/43) and a cosmetic `priority` hint (item 27) — both Specification-Required candidates for a future revision, listed here for traceability only. |
| **Forward-compatibility rule (normative)** | A receiver MUST ignore any `ext` key it does not recognise; an unrecognized key MUST NOT cause validation failure or message rejection. This is what lets `ext` be extended without breaking older clients. |

## 21.21 DNS Parameter Keys Registry (`_dmtap` / `_dmtap-gw` / `_dmtap-mix`)

| | |
|---|---|
| **Registry name** | DMTAP DNS TXT/SVCB Parameters |
| **Reference** | §3.2, §7.2a |
| **Allocation policy** | Specification Required. |
| **Initial contents** | Under `<name>._dmtap.<domain>` TXT: `v=` (format version), `suite=` (algorithm suite, §1.1), `ik=` (base64url identity public key), `id=` (hash of the current `Identity`, §1.3), `kt=` (KT log URL), `keypkgs=` (KeyPackage bundle locator, §5.3). Under `_dmtap.<domain>` SVCB: reserved service parameters and KT anchors. Under `<sel>._dmtap-gw.<domain>` TXT (§7.2a): `v=` (attestation scheme version), `k=` (gateway attestation public key), `suite=` (the §1.1 algorithm suite the attestation key `k=` signs under, GW-7 — without it a verifier has no way to learn the signature algorithm except inferring it from key length, which cannot express the §1.1 PQ-hybrid floor: a hybrid public key and a legacy single-component key of the same encoded length are otherwise indistinguishable at the DNS layer). Under `<sel>._dmtap-mix.<domain>` TXT ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md), analogous to `_dmtap-gw`): `v=` (attestation scheme version), `k=` (mix-operator attestation public key), binding a `MixNodeDescriptor.operator` to an accountable domain for path operator-diversity ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md)). Under `_dmtap.<domain>` a `mix=` locator ([docs/research/mixnet.md §4.4.2](docs/research/mixnet.md)) points to the `MixDirectory` authority. |
| **Registration requirements** | A new TXT/SVCB key MUST specify its grammar and whether it is required or optional for a conformant resolver; unrecognized keys in an existing record MUST be ignored by resolvers (same forward-compat rule as §21.20). |

## 21.22 Capability Negotiation Registry (§10.2)

| | |
|---|---|
| **Registry name** | DMTAP Capability Tokens |
| **Reference** | §10.2, `system` MOTEs (`kind = 0x0A`) |
| **Allocation policy** | Specification Required for tokens intended to be portable across implementations; `x-`-prefixed tokens are Private Use / FCFS, mirroring §21.20. |
| **Initial contents** | Supported-suite tokens (§1.1); privacy-tier tokens (`private`, `fast`, §4.6); supported MLS ciphersuite tokens (§5.1); the **`mls-ciphersuite-floor`** token — a group's per-group MLS-ciphersuite high-water-mark / required PQ floor (§5.1): members advertise it so a Welcome/GroupInfo/Commit selecting a ciphersuite below the floor is a downgrade (`ERR_MLS_CIPHERSUITE_DOWNGRADE`, `0x0414`), policing **message-PQ** on the MLS-ciphersuite axis independently of the `Envelope.suite` high-water-mark (§1.3); the **`deniable-1:1`** token (advertises support for the optional deniable 1:1 mode, §5.2.1 — both peers MUST advertise it before a deniable session is established); KT log-type tokens (`0x01`/`0x02`, §3.5.2, §21.19); mix-suite tokens ([docs/research/mixnet.md §4.4.12](docs/research/mixnet.md), §21.23); transport-substrate tokens (§4.1, §21.24); the **`push-wake`** token (advertises support for the OPTIONAL push wake-signalling layer of §4.9 — a device/node feature, not required for Core, negotiated device↔node); the **`cluster-sync`** token (advertises the device-cluster backfill method — range-based Merkle reconciliation and/or journal replay, §5.6.3 — negotiated device↔device within one identity's cluster); supported extension-kind/extension-header tokens (cross-referencing §21.16/§21.20 registrations); **signed-object `≥ 64` extension-field tokens** — a peer advertises support for a reserved extension field before any sender may include it in a *signed* object (§18.1.2, §10.2); the **`mix-profile-bootstrap`** token ([docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md)) — advertises support for the **Bootstrap** mixnet profile, the degraded 3-hop/floor-3-guard-sample/best-effort-diversity profile a network too small for Standard operates on; it is **additive** (no object gains a field and no CBOR key changes — the profile constrains sender-side path *construction* only, so the conformance vectors are unaffected), it carries **no anonymity claim**, and per [docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md) a peer MUST auto-upgrade to Standard once satisfiable, MUST NOT fall back, and MUST NOT accept it for a contact whose relationship has already run at Standard; and the **`vid-live-1`** token (§24.10) — a profile-local capability of the Video/Media application profile over DMTAP-PUB, advertising support for the OPTIONAL live-streaming rolling-segment mechanism; a publisher advertises it, a consumer lacking it treats its absence as a fact about that peer, never a fault (the capability-absence rule, this row) — the base video profile (VOD) needs no token beyond `pub-1` (§22, §21.24b), since `vid-live-1` is additive on top; and the **`pubsub-1`** token (§25, §21.24d) — advertises support for the DMTAP-PUBSUB extension (`Subscription`/`SubscriptionRevoke`/`FeedHint`, §25.4–§25.6); a node MUST advertise `pub-1` to meaningfully advertise `pubsub-1`, but topic-scoped feed serving (§25.3) needs only `pub-1`, never `pubsub-1`. |
| **Announcement versioning** | Capability announcements are **monotonic**: each carries a `caps_version` (`u64`) and a receiver rejects an announcement older-or-equal to the last accepted from that peer (`ERR_CAPABILITY_ANNOUNCE_ROLLBACK`, `0x030A`, §10.2), so a stale replay cannot suppress an advertised capability. |
| **Forward-compatibility rule** | A node receiving a capability token it does not recognise MUST ignore that token (not the whole `system` MOTE) and MUST NOT assume the counterpart lacks the capability merely because the token name is unfamiliar — absence of a recognised token (in the current, highest-`caps_version` announcement) is inconclusive, not a negative assertion. |

## 21.23 Mix Parameters Registry (Sphinx / mixnet, [docs/research/mixnet.md §4.4](docs/research/mixnet.md))

| | |
|---|---|
| **Registry name** | DMTAP Mix Parameters |
| **Reference** | [docs/research/mixnet.md §4.4](docs/research/mixnet.md) ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md) packet format, [docs/research/mixnet.md §4.4.3](docs/research/mixnet.md) path, [docs/research/mixnet.md §4.4.4](docs/research/mixnet.md) epochs, [docs/research/mixnet.md §4.4.12](docs/research/mixnet.md) PQ), §16.3 |
| **Allocation policy** | **Mix suite** values (the `suite` on `MixNodeDescriptor`/`MixKeyEntry`, §18.5.2): `0x01`–`0x1F` Standards Action, `0x20`–`0xDF` Specification Required, `0xE0`–`0xFE` Private Use, `0x00`/`0xFF` Reserved — mirroring the Algorithm Suites registry (§21.15), because a mix packet format is a network-wide interoperability and security floor. Individual numeric parameters (path length, cell size, delay/cover means, epoch length) are **profile values fixed in §16.3**, tuned within the protocol version, not separately code-point-allocated. |
| **Initial contents** | **Mix suite `0x01` (v0 REQUIRED):** classical **Sphinx** (Danezis–Goldberg 2009) — header group **X25519**, `β` stream cipher **ChaCha20**, per-hop header MAC **Poly1305** (over `β` only), **`δ` payload wide-block PRP LIONESS** (Anderson–Biham, over the whole cell — **not** a stream cipher or AEAD; this is what gives payload tagging-resistance, [docs/research/mixnet.md §4.4.1](docs/research/mixnet.md)/[docs/research/mixnet.md §4.4.6](docs/research/mixnet.md)), KDF **BLAKE3**, cell payload `δ` **2 KiB**, path length **3**, stratified topology, Poisson per-hop delay ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md), [docs/research/mixnet.md §4.4.3](docs/research/mixnet.md), §16.3). **Mix suite `0x02` (RESERVED, PQ frontier):** a post-quantum / hybrid Sphinx packet format — **not yet specified**, tracked per [docs/research/mixnet.md §4.4.12](docs/research/mixnet.md) and §11.3; reserved so the agility seam (`suite` on descriptors + capability negotiation §10.2) is ready when a PQ mix format is standardised. |
| **Registration requirements** | A new mix suite MUST specify its full packet construction (group element, per-hop KDF, `β` stream cipher, `γ` header MAC, and a **`δ` payload transform that is a wide-block PRP / SPRP over the whole cell** — never a stream cipher or an AEAD-over-`δ`, since a malleable payload transform reintroduces active payload tagging — plus the constant-length invariant) and a security analysis; MUST preserve Sphinx's constant-length, unlinkable-per-hop, tagging-resistant properties (**header** integrity via `γ`, **payload** tagging-resistance via the wide-block `δ` transform); MUST state its cell size and how it composes with the bucket ladder ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md)); and MUST NOT be accepted by a node until published here (unknown mix suites rejected fail-closed, mirroring §1.1). PQ mix formats MUST disclose their metadata-only exposure honestly ([docs/research/mixnet.md §4.4.12](docs/research/mixnet.md)). |

## 21.24 Transport Substrates Registry (§4.1)

| | |
|---|---|
| **Registry name** | DMTAP Transport Substrates |
| **Reference** | §4.1 (substrate seam), `LocationRecord.substrate` / `MixNodeDescriptor.substrate` (§18.5.1, §18.5.2) |
| **Allocation policy** | Specification Required. A new substrate MUST specify how `peer_id` and `addrs` are interpreted and dialed under it, how it composes with the reachability ladder (§4.3) and the mixnet ([docs/research/mixnet.md §4.4](docs/research/mixnet.md)), and its NAT/roaming story. `0x01`–`0xDF` Specification Required; `0xE0`–`0xFE` Private Use (closed deployments); `0x00`/`0xFF` Reserved. |
| **Initial contents** | `0x01` — **libp2p** (v0 REQUIRED, the default; absent `substrate` field ⇒ this value): Kademlia DHT routing, circuit-relay v2, Noise/TLS, QUIC/TCP/WS/WebRTC/WebTransport, PeerId = `multihash(pubkey)`, addresses = multiaddrs (§4.1–§4.3). |
| **Migration** | A new substrate is introduced **additively** and **capability-negotiated** (§10.2), the same dual-stack mechanism as a new algorithm suite (§21.15, §21.25 procedure): nodes advertise supported substrates, publish records under each, bridge during the transition, and retire the old substrate only once no pinned relationship needs it. A resolver treats a record on an unimplemented substrate as unreachable (`0x0303`), never a parse failure — moving off libp2p is an incremental migration, not a flag day. |

## 21.24a Gateway Attestation Discriminators Registry (`GatewayAttestation.disc`)

| | |
|---|---|
| **Registry name** | DMTAP Gateway Attestation Discriminators |
| **Reference** | §7.2a, §18.3.11 (`GatewayAttestation.disc`, a `u8`) |
| **Allocation policy** | **Specification Required** for portable, cross-implementation attestation kinds; **`0xE0`–`0xFE` Private Use**; `0x00`/`0xFF` Reserved. Every extensible field in DMTAP has an IANA registry; the gateway-attestation discriminator is no exception. A new attestation kind MUST specify its signature preimage (extending §18.9.11), what it attests, and how a recipient verifies it — and MUST NOT weaken the honest-provenance guarantee (an accepted message with no `provenance` was never plaintext at a gateway, §18.3.5). |
| **Initial contents** | `1` — **legacy-inbound bridge attestation** (§7.2a): "received via gateway G at T," signed by the recipient-domain `_dmtap-gw` key. |
| **Forward-compatibility rule** | An **unknown** `disc` value MUST be treated as an **unverifiable** attestation (`ERR_GATEWAY_ATTESTATION_INVALID`, `0x0601`, DROP_SILENT at the recipient), **never** silently ignored — an attestation whose kind a verifier cannot check is not a pass. This is the fail-closed analogue of the unknown-suite rule (§1.1), applied to provenance. |

## 21.24b First extension registration — DMTAP-PUB (§22, ROADMAP)

The following allocations are registered per the §21.25 extension procedure, for the DMTAP-PUB
public-objects extension (§22, ROADMAP.md). This is the first exercise of the extension
machinery this appendix defines. The registrations below reserve the identifiers; the objects
themselves, their wire encodings, and the individual `ERR_PUB_*` code points are normatively
defined in §22, not here.

| Registry | Allocation |
|---|---|
| Message Kinds (§21.16) | `0x40 pub_announce` — Specification Required, extension range (§2.3, §21.16); a public signed announcement, plaintext, openly signed by the publisher identity (no sealed sender). A **bare signed object, not a MOTE** — never carried inside an `Envelope` (§22.3.2). |
| Capability Tokens (§21.22) | `pub-1` — Specification Required; node/operator opt-in to serving public objects (§10.2, §22). |
| Error/Status Codes (§21.14) | Subsystem byte `0x09` (`0x0900`–`0x09FF`), allocated under the new-subsystem-byte / Standards Action policy of §21.14, reserved to the DMTAP-PUB extension (`ERR_PUB_*`); individual code points defined in §22. Verified against §21.3–§21.11 at registration time: `0x09` was previously wholly unassigned (formerly part of the `0x09`–`0xEF` reserved range), so this allocation collides with no existing code. |
| Signature DS-tags (§18.9 convention) | `DMTAP-PUB-v0/manifest`, `DMTAP-PUB-v0/announce`, `DMTAP-PUB-v0/feed` — reserved identifiers, distinct from every `DMTAP-v0/…` DS-tag in §18.9. `DMTAP-PUB-v0/manifest` in particular is **type-incompatible** with the sealed `Manifest` root construction (§18.9.5, which uses the bare RFC 6962 `0x00`/`0x01` tree over ciphertext chunk hashes): a verifier fails closed on any mismatch, never branches on a boolean "public" flag carried on one shared object. |

## 21.24c Second extension registration — DMTAP-SYNC substrate (proposed, `substrate/SYNC.md`)

The following allocations are **proposed additively**, exactly as §21.24b registered subsystem `0x09`
and `pub-1` for the DMTAP-PUB extension, for the Sync substrate capability
([`substrate/SYNC.md`](substrate/SYNC.md)) — the substrate's "one genuinely new normative specification"
(`substrate/README.md`). Per the substrate's own status note, this directory does not renumber or amend
any numbered section; these registrations reserve the identifiers in the shared §21 namespace so a
future numbered adoption (or another product implementing the substrate directly) collides with
nothing already allocated. The objects, their wire encodings (where frozen), and the individual
`ERR_SYNC_*` code points are normatively defined in `substrate/SYNC.md`, not here.

| Registry | Allocation |
|---|---|
| Message Kinds (§21.16) | None. A `SyncOp` is never carried as a MOTE `kind` — it rides its own reconciliation transport (`substrate/SYNC.md` §5), analogous to how `pub_announce` (§22) is a bare signed object, not a MOTE. |
| Capability Tokens (§21.22) | `sync-1` — Specification Required; a replica's opt-in to speaking the Sync substrate's op algebra and reconciliation wire protocol (§10.2, `substrate/SYNC.md` §1). Sub-tokens for optional sub-features (frame-signing batching, range-Merkle reconciliation) are negotiated under the same `sync-1` capability, per `substrate/SYNC.md` §4.1/§5.3. |
| Error/Status Codes (§21.14) | Subsystem byte `0x0A` (`0x0A00`–`0x0AFF`), allocated under the new-subsystem-byte / Standards Action policy of §21.14, reserved to the Sync substrate capability (`ERR_SYNC_*`); individual code points defined in `substrate/SYNC.md` §12. Verified against §21.3–§21.11 and §21.24b at registration time: `0x0A` was previously wholly unassigned (the `0x09`–`0xEF` reserved range, less `0x09` already allocated to DMTAP-PUB by §21.24b), so this allocation collides with no existing code. |
| Signature DS-tags (§18.9 convention) | `DMTAP-SYNC-v0/op` (COSE_Sign1 `external_aad` for a `SyncOp`, `substrate/SYNC.md` §4.1), `DMTAP-SYNC-v0/op-id` (op content-address hash preimage, §4.1), `DMTAP-SYNC-v0/snapshot` (signed-snapshot signature preimage, §6.1), `DMTAP-SYNC-v0/snapshot-state` (observable-state root-hash preimage, §6.1/§6.2), `DMTAP-SYNC-v0/recon-fp` (range-Merkle fingerprint fold preimage, §5.3) — reserved identifiers, distinct from every `DMTAP-v0/…` and `DMTAP-PUB-v0/…` DS-tag already registered. Unlike the native `sig-val` DS-tags, `DMTAP-SYNC-v0/op` is applied through RFC 9052 COSE (`external_aad`), not the §18.9 `DS-tag ‖ body` concatenation; the domain-separation guarantee is identical. |

## 21.24d Third extension registration — DMTAP-PUBSUB (§25)

The following allocations are registered per the §21.25 extension procedure, for the DMTAP-PUBSUB
feed-subscription/push-hint extension (§25). Unlike §21.24b (DMTAP-PUB, a new subsystem byte) and
§21.24c (DMTAP-SYNC, another new subsystem byte), this registration allocates **no** new subsystem
byte: DMTAP-PUBSUB is an extension *of* DMTAP-PUB, so its error codes are new points **within** the
subsystem byte `0x09` §21.24b already reserved, under §21.14's lighter-weight
Specification-Required-within-an-existing-subsystem policy rather than the Standards-Action
new-subsystem-byte policy. The registrations below reserve the identifiers; the objects themselves,
their wire encodings, and the individual `ERR_PUB_*` code points in this range are normatively
defined in §25, not here.

| Registry | Allocation |
|---|---|
| Message Kinds (§21.16) | `0x41 feed_hint`, `0x42 feed_subscribe`, `0x43 feed_unsubscribe` — Specification Required, extension range (§2.3/§21.16), the next three points after `0x40` (§21.24b). All three are **ordinary sealed-MOTE kinds** riding the existing `Envelope`/`Payload` path (§2.4, §18.3.5) — unlike `0x40 pub_announce`, which is a bare unsealed signed object (§22.3.2). |
| Capability Tokens (§21.22) | `pubsub-1` — Specification Required; node/operator opt-in to originating and/or honoring `Subscription`/`SubscriptionRevoke`/`FeedHint` (§10.2, §25.8). |
| Error/Status Codes (§21.14) | **Seven** new code points — `0x090E`–`0x0913` and `0x0915` — within the existing subsystem byte `0x09` (`ERR_PUB_*`, reserved to DMTAP-PUB by §21.24b); individual code points defined in §25.12. Non-contiguous by construction: `0x0914` (`ERR_PUB_SUITE_BELOW_FLOOR`) is **not** part of this registration — it was allocated separately to §22 (§22.10) between this appendix's two registration passes, so `0x0915` (`ERR_PUB_FEED_TOPIC_MISMATCH`, §25.3.1/§25.13 C-01) sits immediately after a point this document did not itself allocate. Verified against §21.3–§21.11, §21.24b, and §25.12 at registration time: at the first pass, `0x090E`–`0x0913` were unused within `0x0900`–`0x09FF` (the highest point previously allocated was `0x090D`, §22.10); `0x0914` was then separately allocated to §22; `0x0915` was confirmed free at the second pass — so this allocation collides with no existing code. |
| Signature DS-tags (§18.9 convention) | `DMTAP-PUB-v0/subscription` (`Subscription.sig` preimage, §25.4.1), `DMTAP-PUB-v0/subscription-revoke` (`SubscriptionRevoke.sig` preimage, §25.5.1) — reserved identifiers, distinct from every `DMTAP-v0/…`, `DMTAP-PUB-v0/…` (§22, §21.24b), and `DMTAP-SYNC-v0/…` (§21.24c) DS-tag already registered. `FeedHint` (kind `0x41`) needs no DS-tag of its own: it is ordinary `Payload` content authenticated by the existing `Payload.sig` preimage (§18.9.2), unchanged. |

## 21.24e Fourth extension registration — DMTAP-VID (§24)

The following allocation is registered per the §21.25 extension procedure, for the Video/Media
application profile's rendition-derivation signing context (§24.4.4). Unlike §21.24b/§21.24c (new
subsystem bytes) and §21.24d (new message kinds within an existing subsystem), this registration
reserves **no message kind and no error code**: the profile carries its objects on the existing
`pub_announce`/`FeedHead`/`FeedEntry` machinery of §22 under `meta["video"]`/`meta["live"]`
(§24.2), and defines no `ERR_VID_*` subsystem of its own — a malformed manifest is caught by the
DMTAP-PUB decode/validation errors (§22, §21.24b) its objects already ride. The one identifier
this registration reserves is the profile-scoped signing context §24.4.4 defines and §24.17 C-06
records as pending exactly this companion change.

| Registry | Allocation |
|---|---|
| Message Kinds (§21.16) | None. VOD/live video objects ride the existing `pub_announce`/`FeedHead`/`FeedEntry` objects of §22/§25 under a profile-local `meta` schema (§24.2); no new kind is allocated. |
| Capability Tokens (§21.22) | `vid-live-1` — Specification Required; a profile-local capability of the Video/Media application profile over DMTAP-PUB, advertising support for the OPTIONAL live-streaming rolling-segment mechanism (§24.10; already carried in §21.22's Initial contents). The base (VOD) profile needs no token beyond `pub-1` (§22, §21.24b). |
| Error/Status Codes (§21.14) | None. §24 allocates no subsystem byte and defines no `ERR_VID_*` code; §24's own malformed-object cases resolve to the existing DMTAP-PUB errors (§22, §21.24b). |
| Signature DS-tags (§18.9 convention) | `DMTAP-VID-v0/derivation` — the rendition-derivation statement's signing context (§24.4.4), a **profile-scoped** signing context rather than a core `DMTAP-v0/…` DS-tag, and type-incompatible by construction with every other reserved tag (§18.1.6's prefix-free property, §24.17 C-06). Reserved here so no future extension can allocate the string, distinct from every `DMTAP-v0/…`, `DMTAP-PUB-v0/…` (§21.24b), and `DMTAP-SYNC-v0/…` (§21.24c) DS-tag already registered. |

A node that does not implement DMTAP-VID is unaffected: it never advertises `vid-live-1`, and
reads/writes §24 objects exactly as any other DMTAP-PUB consumer, since no wire byte core to §22
is touched.

## 21.24f Fifth extension registration — DMTAP-RTC (§27)

The following allocations are registered per the §21.25 extension procedure, for the DMTAP-RTC
real-time signalling/media-protection extension (§27). Unlike §21.24b/§21.24c (new subsystem
bytes), this registration allocates **no** new subsystem byte for its three error codes: they are
new points **within** the existing subsystem byte `0x04` (Messaging & Group, §21.6), under
§21.14's lighter-weight Specification-Required-within-an-existing-subsystem policy — the same path
§21.24d took for new points within `0x09`. The registrations below reserve the identifiers; the
objects themselves, their wire encodings, and the full error-code prose are normatively defined in
§27, with the individual code points additionally defined in this Part 1 (§21.6), since — unlike
`ERR_PUB_*`/`ERR_SYNC_*` — they sit within a subsystem byte this appendix already owns outright.

| Registry | Allocation |
|---|---|
| Message Kinds (§21.16) | `0x44 rtc_signal` — Specification Required, extension range (§2.3/§21.16), the next point after `0x43` (§21.24d). An **ordinary sealed-MOTE kind**, `Payload`-wrapped, riding the existing deliver/ack/retry path (§2.6), unlike `0x40`'s bare signed object. Default tier `fast` (§27.4.6). |
| Capability Tokens (§21.22) | `rtc-1` — Specification Required; an endpoint's opt-in to originating and accepting `rtc_signal` (§27.9). `rtc-sfu-1` — Specification Required; an operator's opt-in to the SFU role (§27.7.2), announced together with an `RtcCapacity` (§27.7.4). An operator MUST advertise `rtc-1` to meaningfully advertise `rtc-sfu-1`. |
| Error/Status Codes (§21.14) | Three new points `0x0415`–`0x0417` **within** the existing subsystem byte `0x04`, under §21.14's Specification-Required-within-an-existing-subsystem policy rather than the Standards-Action new-subsystem-byte policy. Verified against §21.6 at registration time: `0x0401`–`0x0414` were the highest points previously allocated within `0x04`, so `0x0415`–`0x0417` collide with no existing code. Individual code points are defined in §21.6 above (this document's own copy) and in §27.12. |
| KDF labels (§18.9 convention) | `DMTAP-RTC-v0/sframe` — an **MLS exporter label** (§27.5.1), **not** a signature DS-tag. Reserved in the same namespace so no future extension can allocate the string; §18.9's signature-preimage inventory gains only a note recording that this profile takes no signature of its own, in the manner §18.9.12 records that `ProvenanceRecord` carries none. |
| Signal types | `RtcSignal.type` (`u8`) values `0x01`–`0x07`, with §27.4.2 authoritative for the space and its allocation policy — the same arrangement §21.24a uses for `GatewayAttestation.disc`. |

A node that does not implement DMTAP-RTC is unaffected in every direction: it never advertises
`rtc-1`, never emits an `rtc_signal`, and ignores one it receives without acking it (§2.3,
`ERR_KIND_UNKNOWN` `0x020A`, IGNORE_NO_ACK).

## 21.24g Sixth extension registration — DMTAP Legacy Adapters (§26)

The following allocations are registered per the §21.25 extension procedure, for the DMTAP Legacy
Adapters generalisation of the §7 SMTP/mail-bridge pattern to other legacy rails — SMS, WhatsApp,
Telegram, Discord, Slack (§26). Unlike §21.24d/§21.24f (extensions within an existing subsystem
byte), this registration allocates a **new** subsystem byte, `0x0B`, under the Standards-Action
new-subsystem-byte policy of §21.14 — the same path §21.24b/§21.24c took for `0x09`/`0x0A`.
Verified against §21.3–§21.11, §21.24b and §21.24c at registration time: `0x0B` was the next unused
byte within the `0x09`–`0xEF` reserved range once `0x09` (DMTAP-PUB) and `0x0A` (DMTAP-SYNC) are
accounted for, so this allocation collides with no existing code — the same byte, and the same
reasoning, §26.11 itself proposed before this registration landed. The registrations below reserve
the identifiers; the objects themselves were informal at the time of this registration and have
**since been formally defined** by §21.24h/§18.8a (see the DS-tag row below), and the individual
`ERR_ADAPTER_*` code points are defined directly in this Part 1 (§21.11a), at §26's own request.

| Registry | Allocation |
|---|---|
| Message Kinds (§21.16) | None. `AdapterDescriptor`/`SignedTariff` ride an existing `pub_announce` (§22.3.1) where formalized, and a `UsageReceipt` rides the existing `system` kind (`0x0A`, §21.16); no new message kind is allocated (§26.11). |
| Capability Tokens (§21.22) | `legadapt-1` — Specification Required; an operator's opt-in to gateway-mode legacy-adapter obligations (the four items of §26.2.1), analogous to `pub-1`/`sync-1`/`pubsub-1`. |
| Error/Status Codes (§21.14) | Subsystem byte `0x0B` (`0x0B00`–`0x0BFF`), allocated under the new-subsystem-byte / Standards Action policy of §21.14, reserved to the DMTAP Legacy Adapters extension (`ERR_ADAPTER_*`); the three initial code points (`0x0B01`–`0x0B03`) are defined in this Part 1, §21.11a. |
| Signature DS-tags (§18.9 convention) | ~~`DMTAP-ADAPT-v0/descriptor`, `DMTAP-ADAPT-v0/tariff`, `DMTAP-ADAPT-v0/receipt`~~ — reserved identifiers (§26.11), distinct from every `DMTAP-v0/…`, `DMTAP-PUB-v0/…` (§21.24b), `DMTAP-SYNC-v0/…` (§21.24c), `DMTAP-VID-v0/…` (§21.24e), and `DMTAP-RTC-v0/…` (§21.24f) DS-tag/label already registered. **Superseded by §21.24h**: `AdapterDescriptor`/`SignedTariff`/`UsageReceipt` are now formally defined — not as a bespoke adapter-only shape, but as the general `CoordinatorDescriptor`/`Tariff`/`UsageReceipt` objects of §18.8a, since a legacy adapter is a `gateway`-kind coordinator (CONTRACT §5). The three `DMTAP-ADAPT-v0/…` strings above are therefore retired **unallocated** — never assigned to any object — rather than left as a second, permanently-informal scheme running alongside the formal one; they remain reserved only so the literal strings are never reused for anything else. |

A node that does not implement any Legacy Adapter is unaffected: it never advertises
`legadapt-1`, never emits or expects `ERR_ADAPTER_*` on subsystem `0x0B`, and every rail §26
covers remains purely optional, off by default beyond the in-tree hardware-SMS reference (§26.9).

## 21.24h Seventh extension registration — Coordinator Contract wire objects (coordinator/CONTRACT.md)

The following allocations are registered per the §21.25 extension procedure, for the coordinator
contract's `CoordinatorDescriptor`/`Tariff`/`UsageReceipt`/`GatewayAuthz` objects (coordinator/
CONTRACT.md §2.1, §2.4, §6; §12.2), normatively defined at §18.8a. Unlike §21.24b/§21.24c/§21.24g
(new subsystem bytes) and §21.24d/§21.24f (new points within an existing subsystem), this
registration allocates **no new error codes at all**: it pays down wire debt on codes already
allocated — `ERR_GATEWAY_SENDER_UNAUTHENTICATED` (`0x0607`), `ERR_GATEWAY_SENDER_ADDRESS_
UNAUTHORIZED` (`0x060A`), `ERR_GATEWAYAUTHZ_DENIED` (`0x070E`, all three within subsystem `0x06`/
`0x07`, §21.8/§21.10) and `ERR_ADAPTER_TARIFF_INVALID`/`ERR_ADAPTER_RECEIPT_INVALID`/
`ERR_ADAPTER_CREDENTIAL_UNAUTHORIZED` (`0x0B01`–`0x0B03`, subsystem `0x0B`, §21.11a, §21.24g) —
each of which cited an object with no CDDL, DS-tag, or signing preimage until now.

| Registry | Allocation |
|---|---|
| Message Kinds (§21.16) | None. `CoordinatorDescriptor`/`Tariff` ride an existing `pub_announce` (§22.3.1) where published, and `UsageReceipt` rides the existing `system` kind (`0x0A`, §21.16) delivered directly to the payer; `GatewayAuthz` is gateway-local state, never mesh-transmitted (§18.8a.3). No new message kind is allocated. |
| Capability Tokens (§21.22) | None. Coordinator conformance is a document-level contract (CONTRACT §7's checklist), not a capability-negotiated wire opt-in; the objects here are read/verified by any conformant client without a prior capability handshake. |
| Error/Status Codes (§21.14) | None allocated by this registration — see above. `0x0607`/`0x060A`/`0x070E`/`0x0B01`–`0x0B03` remain owned by §21.8/§21.10/§21.11a; this entry exists so the objects those codes name are traceable from the extension-registration list, not to claim new code space. |
| Signature DS-tags (§18.9 convention) | `DMTAP-COORD-v0/descriptor` (`CoordinatorDescriptor.sig`), `DMTAP-COORD-v0/tariff` (`Tariff.sig`), `DMTAP-COORD-v0/usage-receipt` (`UsageReceipt.sig`) — defined identifiers (not merely reserved), distinct from every `DMTAP-v0/…`, `DMTAP-PUB-v0/…` (§21.24b), `DMTAP-SYNC-v0/…` (§21.24c), `DMTAP-VID-v0/…` (§21.24e), `DMTAP-RTC-v0/…` (§21.24f), and the now-retired `DMTAP-ADAPT-v0/…` (§21.24g) DS-tag/label already registered. `GatewayAuthz` mints no DS-tag of its own (§18.8a.3: gateway-local, unsigned); its per-address/per-rail grant extension reuses the existing `DMTAP-v0/cap-token` `CapabilityToken` (§18.7.3) rather than allocating a new signing context. |

A node that implements no coordinator role beyond native mesh delivery is unaffected: it never
constructs, signs, or verifies any object in this registration.

## 21.25 Extension & versioning procedure (normative)

This section is the operational answer to "how does DMTAP add something new without
fragmenting."

1. **New algorithm suite.** Standards Action (§21.15). A node encountering an unregistered
   `suite` value in any signed/encrypted object MUST reject it (`0x0101`/`0x0201`) — there is
   no partial trust, no best-effort parse. Suite adoption spreads only via the multi-suite
   `Identity.suites` mechanism (§1.3): a recipient advertises the new suite, senders pick up
   the intersection, and old suites are retired only once no pinned relationship requires them.

2. **New message `kind`.** Specification Required within `0x40`–`0x7F` (Standards Action for a
   `0x0B`–`0x3F` core allocation, §21.16). A node receiving an unassigned or unimplemented
   `kind` MUST NOT `ack` it (`0x020A`) and MAY silently ignore it; this is what lets a new kind
   roll out client-by-client with no coordinated flag day (§10.1).

3. **New `Headers.ext` key.** Specification Required for an unprefixed key intended to be
   portable; no registration for `x`-prefixed experimentation (§21.20). A receiver MUST ignore
   an unrecognized key rather than fail validation.

4. **New challenge type, Identity Resolver Type, DNS parameter, or capability token.** Specification
   Required, reviewed by a Designated Expert (below) for two things specific to DMTAP, beyond
   ordinary interoperability: (a) an anti-abuse extension MUST preserve the issuer-trust /
   zero-default-budget rule (§9.3.1) — it MUST NOT create a path for a sender to manufacture
   its own unlimited-cost-relief proof; (b) **no** extension of any kind MAY be gated behind an
   operator's paid seam in a way that weakens privacy, cryptography, or metadata protection —
   the inviolable rule of §12.3 binds extensions exactly as it binds the base protocol.

5. **Designated Expert.** Pending formal IETF adoption (§10.5), the expert function is
   performed by the specification's maintainers, applying the criteria above. Upon Internet-
   Draft submission, this function SHOULD transition to an IANA-run registry with an
   IETF-appointed Designated Expert, per standard IANA practice for Specification-Required
   registries.

6. **Deprecation, never reassignment.** Once allocated (in any registry in this appendix), a
   code point, key, or tag MUST NOT be reused for a different meaning, even if the original
   allocation is abandoned. Mark it Deprecated and allocate a new point for new semantics. This
   mirrors the append-only discipline that makes key transparency (§3.5) and the committer log
   (§5.1) trustworthy, applied to the registries themselves.

7. **Unknown-value handling — the three forward-compatibility rules, stated together.**
   Precisely because these three differ, they are restated together here to prevent
   conflation:
   - **Unknown `suite`** → **rejected**, fail-closed, no processing (`0x0101`/`0x0201`).
   - **Unknown `kind`** → **not acknowledged**, but MAY be silently ignored rather than
     rejected (`0x020A`).
   - **Unknown `ext` key** → **ignored**, and MUST NOT affect validation of the rest of the
     object at all.

   The differing treatment is deliberate: a suite is a security-critical trust decision (guess
   wrong and you silently downgrade crypto), a kind is a processing-capability question (guess
   wrong and you silently corrupt application semantics), and an `ext` key is inert metadata by
   construction (guess wrong and nothing security-relevant is at stake).

8. **The top-level format version `v` is FROZEN by design — the one axis that does not evolve.**
   Unlike every mechanism above, the top-level `Envelope.v` (§18.1, §18.3.1) has **no** additive,
   dual-stack, or capability-negotiated evolution path, and this is **intentional**, not an
   oversight: all forward evolution routes through the versioned sub-registries (§21.15–§21.24a),
   each with its own IANA range and dual-stack migration. `v` is a **fail-closed tripwire** — it
   MUST equal `0` in v0, and a decoder MUST reject any other value (`0x0201`) rather than negotiate
   or best-effort-parse it. It exists so that a hypothetical wholly-incompatible successor wire
   format could be rejected cleanly instead of mis-parsed; there is deliberately no `v=` capability
   advertisement and no in-band version negotiation.

## 21.26 Summary

- **Error/status codes defined:** 145 (`0x0101`–`0x0126`: 38, incl. the KT-v1 detection codes
  `0x0110`–`0x0112`, the org-administration codes `0x0113`–`0x0115` (§3.10), `0x0116`
  device-attestation and `0x0118` attestation-expired (§1.2a), `0x0117` KT leaf-hash mismatch
  (§3.5, §18.4.9), the `Profile` display-data codes `0x0119` (signature invalid), `0x011A`
  (avatar content-address mismatch) and `0x011B` (avatar URL unsafe / SSRF guard) (§3.9.5,
  §18.4.12), and the alias codes `0x011C` (self-asserted alias fails forward-verify) and `0x011D`
  (independently-revocable alias revoked) (§3.9.4, §3.11), the resolver codes `0x011E` (name-chain bidirectional binding unverified, §3.12.5(b)), `0x011F` (resolver-type unsupported, §3.12.2) and `0x0120` (inter-resolver disagreement, §3.12.3) (§3.12), and `0x0121` key-rotation-unauthorised (stolen-`IK` / `recover_threshold`-only takeover defence, §1.5), the confusable-name codes `0x0122` (mixed-script label) and `0x0123` (confusable-skeleton collision with a pin) (§3.9.7), and `0x0124` device-unauthorised (well-attested but not §1.4-policy-authorised); `0x0201`–`0x0211`: 17, incl. `0x020F` suite-downgrade, `0x0210`
  hybrid-suite-incomplete (intra-suite PQ-strip defence, §1.3), and `0x0211` envelope-context-mismatch (envelope `kind`/`ts`/`to` bound into `Payload.sig`, §18.9.2);
  `0x0301`–`0x0316`: 22, incl. `0x030A` capability-announce
  rollback (§10.2), the mixnet codes `0x030B`–`0x0311` — directory/descriptor/path (`0x030B`–`0x030D`),
  replay (`0x030E`), active-attack detection (`0x030F`), no-downgrade fail-closed covering
  both tier and profile downgrade (`0x0310`, [docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)), and mix-directory freshness/freeze defence
  (`0x0311`, [docs/research/mixnet.md §4.4.2](docs/research/mixnet.md)) — and the OPTIONAL push wake-signalling codes `0x0312`–`0x0316` (§4.9):
  subscription-not-authenticated, WakePing-content-present, WakePing-auth-failed,
  WakePing-rate-limited (emitter + receiver), and WakePing-replay (relay-replay battery-drain);
  `0x0401`–`0x0414`: 20, incl. the deniable-mode codes `0x040B`–`0x040F` (§5.2.1) — prekey
  invalid/exhausted, X3DH/PQXDH failure, ratchet-MAC failure, mode-unavailable, and the
  signature-forbidden guard — the device-cluster sync codes `0x0410`–`0x0413` (§5.6):
  cluster-device-unauthorised, reconciliation-summary-invalid, journal-chain-broken (own-log fork),
  and cluster-CRDT-op-invalid — and `0x0414` MLS-ciphersuite-downgrade (message-PQ policed on the
  MLS ciphersuite, separate from `Envelope.suite`, §5.1);
  `0x0501`–`0x050B`: 11, incl. `0x050B` capability-revoked (§13.5, §18.7.3); `0x0601`–`0x0609`: 9,
  incl. the gateway-alias codes `0x0605` (legacy→native alias unmappable) and `0x0606` (encoded
  alias non-reversible/over-length) (§7.10) and the outbound open-relay-prevention code `0x0607`
  (`ERR_GATEWAY_SENDER_UNAUTHENTICATED`, the §7.11.2/§9.10 authenticated-sender floor), and the
  two bridge-integrity codes `0x0608` (TLS policy unmet on either leg, §7.2/§7.3) and `0x0609`
  (SMTPUTF8/8BITMIME capability absent — fail cleanly rather than accept-then-mangle, §7.2b),
  plus the informative SMTP mapping table of §21.9;
  `0x0701`–`0x070F`: 15, incl. `0x070F` `ERR_POLICY_BELOW_FLOOR` (§9.7a/[docs/research/vdf.md §9.4.1](docs/research/vdf.md) — the only code in the block whose fault is the recipient's **own** policy rather than an inbound object); `0x0801`–`0x080D`: 13, incl. `0x0808` manifest-key-present (§5.5), the
  file-durability codes `0x0809`–`0x080C` (§5.5.1–§5.5.5) — file-unavailable (origin-hold residual),
  manifest-durability-invalid, retention-expired, and spool-overflow (pushed-attachment storage DoS) —
  and `0x080D` file-size-tier-violation (attachment delivery-mechanism/size self-inconsistent, §5.5.1)),
  spanning the 8 requested subsystems, with every code resolving to exactly one of the 13
  defined responder actions (§21.2) — no undefined behaviour remains.
- **IANA registries defined:** **12 registries + 1 extension/versioning procedure** — the 8
  requested registries (Algorithm Suites, Message Kinds, Challenge Types, Identity Resolver Types, KT Log
  Types, `Headers.ext` Keys, DNS Parameters, Capability Tokens), the **Mix Parameters** registry
  (§21.23) and **Transport Substrates** registry (§21.24) added for the mixnet and substrate seams
  ([docs/research/mixnet.md §4.4](docs/research/mixnet.md), §4.1), the **Gateway Attestation Discriminators** registry (§21.24a) added for the
  `GatewayAttestation.disc` extension seam (§7.2a, §18.3.11), and the DMTAP Error/Status Code
  Registry itself (§21.14, needed to make Part 1 durable against future extension) = **12
  registries**; plus the **extension/versioning procedure** (§21.25) that governs all of them — a
  procedure, **not** a registry. (This count includes only genuine registries; the §21.25 procedure
  is deliberately not counted among them.)
