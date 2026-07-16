# 21. Appendix D: Error Codes & IANA Considerations

This appendix is normative. It has two parts: **Part 1** (§21.1–§21.12) is the exhaustive
error/status registry — every failure condition defined anywhere in this specification, given a
stable code, a required responder action, and a retryability classification, so that **no
condition in DMTAP has undefined behavior**. **Part 2** (§21.13–§21.23) states the IANA
considerations: the registries DMTAP requires, their initial contents, their allocation
policies, and the procedure for extending the protocol without fragmenting it. The key words
"MUST", "MUST NOT", "SHOULD", "SHOULD NOT", and "MAY" are used as in RFC 2119, consistent with
the rest of this specification.

Every table entry in Part 1 is a contract: given the stated condition, the stated action is
the *only* conformant behavior. An implementation that reaches one of these conditions and does
something not listed in its Action column is non-conformant.

---

# Part 1 — Error / Status Registry

## 21.1 Conventions and code format

Every code is a 16-bit value `0xSSNN`: **`SS`** is the subsystem byte, **`NN`** is the code
point within that subsystem. This is a wire-level registry identifier for logging,
conformance-suite assertions (§10.3), and cross-implementation diagnostics — it is **not**
itself a new envelope field; each error is detected at the point in the spec cited in its
"Operation(s)" column, using the checks already normatively defined there. This appendix does
not introduce new validation logic; it catalogs and names the logic that §§1–17 already
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
| `0x09`–`0xEF` | Unassigned — reserved for future subsystems (§21.23) |
| `0xF0`–`0xFE` | Private Use (experimental/vendor subsystems) |
| `0xFF` | Reserved |

Two code prefixes are used in the tables below:

- **`ERR_*`** — a rejected/failed condition: the operation did not succeed and the recipient
  took a defensive or corrective action.
- **`STATUS_*`** — a defined **non-error** outcome that still requires a specific, mandated
  responder action (e.g. deduplication). Listing these alongside errors is deliberate: a
  status the spec doesn't name explicitly is exactly where undefined behavior creeps in.

## 21.2 Responder action vocabulary (normative)

Every code in §21.3–§21.11 resolves to exactly one of the following actions. Definitions are
fixed here once so the tables can cite them tersely without re-explaining each time.

| Action | Definition |
|--------|------------|
| **DROP_SILENT** | Discard immediately. No `ack`, no error returned, no user-visible effect. To the sender/network this is indistinguishable from packet loss (§2.7, §2.7a). |
| **ACK_DEDUP** | Acknowledge (`ack`) without re-processing; the object is already held (§2.6). |
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
| `0x0101` | `ERR_UNKNOWN_SUITE` | Identity/DeviceCert/RecoveryPolicy/KeyRotation/MoveRecord parse (§1.1, §10.1) | Object carries a `suite` this implementation does not recognize. | No | FAIL_CLOSED_BLOCK |
| `0x0102` | `ERR_SUITE_INTERSECTION_EMPTY` | Suite selection for delivery (§1.3) | Sender's and recipient's supported-suite sets do not intersect. | Conditional (recipient publishes an overlapping suite) | REJECT_NOTIFY |
| `0x0103` | `ERR_IDENTITY_SIG_INVALID` | `Identity` verification (§1.3) | One or more `sig` entries fail to validate under the corresponding `iks` entry. | No | FAIL_CLOSED_BLOCK |
| `0x0104` | `ERR_IDENTITY_CHAIN_BROKEN` | `Identity.prev` / `KeyRotation` / `MoveRecord` chain check (§1.3, §1.5, §1.6) | The presented object's hash chain is inconsistent with what this node has previously pinned or observed. | No | HALT_ALERT |
| `0x0105` | `ERR_STALE_ROLLBACK` | `Identity`/`RecoveryPolicy` version check (§1.3, §1.4, §3.5) | A version number ≤ a version already seen/pinned is presented (rollback/replay of a superseded-but-validly-signed object). | No | FAIL_CLOSED_BLOCK; HALT_ALERT if the target is this node's own pinned identity |
| `0x0106` | `ERR_KT_UNREACHABLE` | KT check at first contact (§3.3) | Key transparency log is unreachable, partitioned, or censored at the moment of first-contact pinning. | Yes (network condition) | FAIL_CLOSED_BLOCK — MUST NOT silently TOFU-pin; block or hard-warn and require explicit acceptance |
| `0x0107` | `ERR_KT_EQUIVOCATION` | KT tree-head gossip cross-check (§3.5) | The log shows different histories to different observers (split-view). | No | HALT_ALERT |
| `0x0108` | `ERR_KT_PROOF_INVALID` | Inclusion/consistency proof verification (§3.5) | A signed tree head or inclusion proof fails to verify against the log's public key. | Yes (fetch a fresh proof) | FAIL_CLOSED_BLOCK |
| `0x0109` | `ERR_NAME_RESOLUTION_FAILED` | `resolve(name)` (§3.3, §3.6) | DNS/self-sovereign name backend returns no binding, or the binding is malformed. | Yes | REJECT_NOTIFY (to the sender attempting to address this name) |
| `0x010A` | `ERR_MOVE_RECORD_INVALID` | `MoveRecord` verification (§1.6) | Signature invalid, or not chained from the pinned `IK`. | No | FAIL_CLOSED_BLOCK — retain the prior name binding |
| `0x010B` | `ERR_RECOVERY_POLICY_UNAUTHENTICATED` | `RecoveryPolicy` publish (§1.4 rule 1) | Not signed by `IK`, nor by a satisfied `rotate_threshold` quorum. | No | FAIL_CLOSED_BLOCK + HALT_ALERT (owner's monitoring devices must alert) |
| `0x010C` | `ERR_RECOVERY_THRESHOLD_INVALID` | `RecoveryPolicy` publish (§1.4 rule 2) | `rotate_threshold` < `recover_threshold`. | No | FAIL_CLOSED_BLOCK — reject the policy object outright |
| `0x010D` | `ERR_DEVICE_CERT_INVALID` | `DeviceCert` validation (§1.2) | Signature invalid, or `caps` claimed exceed what the signing `IK`/quorum authorized. | No | FAIL_CLOSED_BLOCK |
| `0x010E` | `ERR_RECOVERY_WEAKENING_UNQUORUMED` | `RecoveryPolicy` publish (§1.4 rule 3) | A change that removes/weakens a recovery factor is signed by `IK` alone without satisfying `rotate_threshold` (stolen-`IK` takeover defense). | No | FAIL_CLOSED_BLOCK + HALT_ALERT (owner's monitoring devices must alert) |
| `0x010F` | `ERR_RECOVERY_VETO_WINDOW` | `RecoveryPolicy` weakening effect (§1.4 rule 4, §16.8) | A factor-weakening change attempts to take effect before its 72 h veto/delay window elapses, or a non-conforming lesser-bar weakening is observed within the window. | Conditional (takes effect once the window elapses with no valid veto) | FAIL_CLOSED_BLOCK — hold until the window elapses; a `rotate_threshold`-backed veto aborts it |
| `0x0110` | `ERR_KT_STH_INCONSISTENT` | KT v1 STH gossip cross-check (§3.5.2(a),(d)) | Two validly-signed STHs of the **same** log are mutually inconsistent — equal `tree_size` but differing `root_hash`, or no valid consistency proof exists between them (append-only violation). Distinct from `0x0107` (`ERR_KT_EQUIVOCATION`, the split-view *conclusion*): `0x0110` is the specific append-only/consistency-proof failure that evidences it. | No | HALT_ALERT — stop trusting the log; publish the conflicting STHs as transferable evidence (§3.5.2(d)) |
| `0x0111` | `ERR_KT_LOG_QUORUM_UNMET` | KT v1 multi-log federation binding check (§3.5.2(b)) | A `name → ik` binding is not attested by the required `> n/2` quorum of the pinned log set (logs disagree, or too many are unreachable). | Conditional (a later fetch reaching quorum resolves it) | FAIL_CLOSED_BLOCK — MUST NOT pin on a sub-quorum view; fall back to OOB verification (§3.4.1) |
| `0x0112` | `ERR_KT_STH_STALE` | KT v1 STH freshness check (§3.5.2(a), §16.2) | A presented STH is older than the STH freshness window / not refreshed within the maximum merge delay — the freeze/withholding attack, where a log serves an old but self-consistent head. | Yes (fetch a fresher head) | HOLD_RESYNC — buffer and re-fetch a current STH before trusting the view; escalate to HALT_ALERT if it persists past gossip cross-check |
| `0x0113` | `ERR_DOMAIN_DIRECTORY_SIG_INVALID` | `DomainDirectory` verification (§3.10.3, §18.4.7) | The org directory object is not validly signed by the domain's pinned authority key (§3.10.1). | No | FAIL_CLOSED_BLOCK — do not trust the directory; per-name resolution (§3.3) is unaffected |
| `0x0114` | `ERR_DIRECTORY_ENTRY_UNVERIFIED` | `DirEntry` forward-binding check (§3.10.3, §3.9.4) | A directory entry's `name → ik` does not match the forward DNS + KT binding — the directory indexes, it does not attest. | No | FAIL_CLOSED_BLOCK — render the entry unverified; MUST NOT be used to address mail |
| `0x0115` | `ERR_ORG_MANAGED_UNDISCLOSED` | Member-custody disclosure check (§3.10.2, §18.4.7) | An org-managed (escrowed-key) account is presented without its `org-managed` custody marker — undisclosed org access to a member's mailbox. | No | HALT_ALERT — MUST NOT present as a sovereign identity; surface the escrow honestly |

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
| `0x020A` | `ERR_KIND_UNKNOWN` | §2.3, §10.1 | `kind` is outside the implemented set and not a recognized extension. | No (per this MOTE) | IGNORE_NO_ACK |
| `0x020B` | `ERR_EXPIRED_MOTE` | `Payload.expires` (§2.4, §16.1) | Client-requested expiry has passed at receipt time. | No | DROP_SILENT — cooperative hint only (§6.6 item 8), not a security guarantee |
| `0x020C` | `ERR_TIMESTAMP_OUT_OF_SKEW` | `Envelope.ts` vs. receiver clock (§16.1) | `ts` falls outside the ±120 s clock-skew tolerance. | Yes (resend with a fresh `ts`) | DROP_SILENT for cold senders; implementations MAY be lenient toward known contacts |
| `0x020D` | `ERR_MALFORMED_OBJECT` | any CBOR parse (Envelope/Payload/Attachment/Manifest) | Object fails to parse as well-formed CBOR against its schema. | No | DROP_SILENT |
| `0x020E` | `STATUS_DUPLICATE_ID` | §2.6 (deduplication) | Recipient already holds `id`. | N/A | ACK_DEDUP |

**Content-addressed dedup as replay defense.** §2.6/§20E's dedup-by-`id` is what makes a bare
resend of a previously-processed MOTE a non-event rather than a distinct "replay" failure mode
at this layer — the content address absorbs it structurally. Replay of a **nonce** (as opposed
to a MOTE) is a distinct concept scoped to the Auth ceremony; see `0x0502` (§21.7).

## 21.5 Transport & Reachability errors (`0x03xx`)

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0301` | `ERR_LOCATION_SIG_INVALID` | `LocationRecord` verification (§4.2) | Signature fails to validate under the claimed device key. | No | FAIL_CLOSED_BLOCK — discard the record |
| `0x0302` | `ERR_LOCATION_STALE` | `LocationRecord` sequence check (§4.2) | Sequence number ≤ a previously-seen record for this key (rollback/censorship defense). | No | FAIL_CLOSED_BLOCK — retain the newer cached record |
| `0x0303` | `ERR_LOCATION_UNREACHABLE` | DHT lookup (§4.2, §4.3) | No location record found, or no peer can be dialed via any resolved address. | Yes | ROTATE_RETRY — fall down the reachability ladder (§4.3); ultimately governed by the sender's delivery state machine (§4.7) |
| `0x0304` | `ERR_ECLIPSE_SUSPECTED` | S/Kademlia disjoint-path lookup comparison (§4.2 CAUTION) | Disjoint lookup paths disagree beyond tolerance, suggesting routing-table poisoning. | Yes | ROTATE_RETRY (re-query via disjoint paths / rendezvous); HALT_ALERT if persistent across attempts |
| `0x0305` | `ERR_RELAY_RESERVATION_UNAVAILABLE` | Circuit-relay v2 reservation (§14.5, §16.6) | Target relay has no free reservation slot. | Yes | ROTATE_RETRY — try an alternate relay |
| `0x0306` | `ERR_RELAY_CIRCUIT_CAP_EXCEEDED` | Circuit-relay v2 per-circuit cap (§16.6: 2 min / 128 KiB) | Circuit closed after hitting its duration/byte cap. | Yes | REJECT_NOTIFY — fall back to direct/hole-punch or another relay |
| `0x0307` | `ERR_MIX_PACKET_MALFORMED` | Sphinx onion peel (§4.4) | A mix hop cannot peel a layer (corrupt or non-conformant packet). | No | DROP_SILENT — a content-blind mix has no channel to notify anyone |
| `0x0308` | `ERR_SENDER_RETRY_DEADLINE_EXCEEDED` | Delivery state machine `EXPIRED` (§2.6, §4.7, §16.1: 72 h) | Sender's retry deadline elapsed without an `ack`. | No | REJECT_NOTIFY — notify the sending user; drop the queued MOTE |
| `0x0309` | `ERR_OFFLINE_BUFFER_UNAVAILABLE` | Peer buffering / relay-mailbox pickup (§4.3, §14.5) | The buffering peer or relay-mailbox is unreachable, or the buffer's TTL has elapsed. | Yes, until TTL | ROTATE_RETRY before TTL; REJECT_NOTIFY after |

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
| `0x0409` | `ERR_GROUP_POLICY_VIOLATION` | Add/Remove/role/policy Commit (§5.8.2) | The proposing member's role does not satisfy the required `admin`/`owner` authorization for the requested change. | No, without a role change | DENY_POLICY |
| `0x040A` | `ERR_TREEKEM_STATE_DIVERGENCE` | Post-Commit ratchet-tree integrity check (§5.1) | A member's derived tree/epoch secret disagrees with the group's expected state after applying an agreed Commit sequence. | Yes (resync from Welcome/External Commit) | HALT_ALERT — treated as a potential integrity failure, not silently reconciled |

## 21.7 Auth errors — DMTAP-Auth (`0x05xx`)

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0501` | `ERR_ORIGIN_MISMATCH` | Login ceremony origin check (§13.3.1) | The `rp_origin` bound into the signed challenge does not match the origin the trusted client actually observed. | No | FAIL_CLOSED_BLOCK — treat as a phishing attempt |
| `0x0502` | `ERR_NONCE_REPLAYED` | Challenge nonce check (§13.3, §16.1) | The single-use nonce has already been consumed. | No | FAIL_CLOSED_BLOCK |
| `0x0503` | `ERR_CHALLENGE_EXPIRED` | Challenge validity window (§13.3, §16.1: 120 s) | `exp` has passed, or the challenge falls outside the clock-skew-adjusted validity window. | Yes (issue a fresh challenge) | REJECT_NOTIFY |
| `0x0504` | `ERR_SESSION_REVOKED` | DPoP/GNAP session check (§13.4) | The session backing this request has been explicitly revoked. | No | REJECT_NOTIFY — RP denies; client must re-authenticate |
| `0x0505` | `ERR_SESSION_EXPIRED` | Session lifetime check (§13.4) | The session's lifetime has elapsed. | Yes (re-authenticate) | REJECT_NOTIFY |
| `0x0506` | `ERR_DPOP_PROOF_INVALID` | Proof-of-possession check (§13.4) | The DPoP/GNAP proof does not verify against the session's bound key, or is replayed. | No | FAIL_CLOSED_BLOCK |
| `0x0507` | `ERR_UNTRUSTED_APPROVAL_CHANNEL` | Remote-node login approval (§13.3.1) | The node cannot attribute the presented `rp_origin` to an authenticated, attributable request channel. | No | FAIL_CLOSED_BLOCK — the node MUST reject the challenge outright (consent-farming defense) |
| `0x0508` | `ERR_CAPABILITY_DELEGATION_INVALID` | Delegated-capability check (§13.5) | The UCAN-style token is invalid, expired, or the invoked right exceeds what was attenuated. | No | DENY_POLICY |
| `0x0509` | `ERR_OIDC_ISSUER_MISMATCH` | OIDC bridge / self-issued discovery (§13.6) | The ID Token's issuer does not match the discovered/pinned issuer for the claimed identity. | No | FAIL_CLOSED_BLOCK |
| `0x050A` | `STATUS_SESSIONS_INVALIDATED_ON_RECOVERY` | `IK` recovery completion (§13.4) | All prior session authorizations are invalidated as a consequence of a completed identity recovery. | N/A | REJECT_NOTIFY — force re-authentication on every RP |

## 21.8 Gateway errors (`0x06xx`)

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0601` | `ERR_GATEWAY_ATTESTATION_INVALID` | Inbound attestation verification (§7.2a) | The attestation signature over "received via gateway G at T" does not verify. | No | DROP_SILENT at the recipient node (see honest limit, §21.9) |
| `0x0602` | `ERR_GATEWAY_ATTESTATION_KEY_UNTRUSTED` | Inbound attestation key lookup (§7.2a) | The attestation key is not published under the recipient's own domain's `_dmtap-gw` record, nor in an explicitly trusted set. | No | DROP_SILENT |
| `0x0603` | `ERR_DKIM_DELEGATION_INVALID` | Outbound DKIM verification at destination MTA (§7.3) | The gateway's delegated-selector DKIM signature fails to verify at the receiving legacy system. | Yes (fix key publication) | REJECT_NOTIFY — the sending node's queue retries per §7.4 |
| `0x0604` | `ERR_MX_TRANSIENT_FAILURE` | Outbound SMTP transaction (§7.3) | The destination MX rejects or times out the transaction transiently. | Yes | REJECT_NOTIFY — the sending node retries; the gateway holds nothing (§7.4) |

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
| `0x070E` | `ERR_GATEWAYAUTHZ_DENIED` | Operator `GatewayAuthz`, operator-unreachable safe default (§12.2) | Legacy egress for a cold/unproven sender is denied during an operator outage (fail-safe, never fail-open for this specific capability). | Yes (once the operator is reachable, or with self-contained proof) | DENY_POLICY |

## 21.11 Files errors (`0x08xx`)

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0801` | `ERR_MANIFEST_HASH_MISMATCH` | Manifest integrity check (§5.5) | `Manifest.id` does not match the recomputed BLAKE3 Merkle root over `chunks`. | No | DROP_SILENT — do not begin fetch |
| `0x0802` | `ERR_CHUNK_HASH_MISMATCH` | Per-chunk integrity check (§5.5) | A fetched chunk fails to verify against its listed hash. | Yes (re-fetch from a different swarm holder) | ROTATE_RETRY |
| `0x0803` | `ERR_CHUNK_UNAVAILABLE` | Swarm fetch (§5.5 availability tiers) | No current holder serves a required chunk. | Yes (best-effort tier) | ROTATE_RETRY; REJECT_NOTIFY once retry budget is exhausted |
| `0x0804` | `ERR_SIZE_TIER_MISMATCH` | Size-tier classification (§2.5, §6.5) | The declared/observed size does not match the tier (inline/normal/large) the MOTE was routed under. | No | DROP_SILENT / DENY_POLICY — MUST NOT silently downgrade the privacy tier to compensate |
| `0x0805` | `ERR_FILE_KEY_MISSING_OR_REVOKED` | Post-removal file access (§6.7) | A former group member's file key has been rotated out after removal from a shared folder. | No | DENY_POLICY — the member must be re-added and re-shared |
| `0x0806` | `ERR_STORAGE_QUOTA_EXCEEDED` | Operator storage `Policy` (§12.2) | Hosted-operator storage cap reached. Self-host default is unlimited. | Yes | DENY_POLICY |
| `0x0807` | `ERR_ATTACHMENT_KEY_INVALID` | Attachment/chunk decryption (§2.5) | The per-file content `key` fails to decrypt the referenced inline blob or chunk. | No | DROP_SILENT |
| `0x0808` | `ERR_MANIFEST_KEY_PRESENT` | Manifest decode (§5.5, §18.3.8) | A received `Manifest` carries an embedded content key (reserved-forbidden field `5`). A manifest is a swarm-distributed blob, so an embedded key would leak to every holder that serves it. | No | FAIL_CLOSED_BLOCK — reject the manifest, MUST NOT use the embedded key; the legitimate key travels only in `Attachment.key` inside the sealed MOTE (§18.3.7) |

## 21.12 Traceability matrix

Every condition explicitly named in this appendix's brief maps to a code as follows, for
auditability:

| Named condition | Code(s) |
|---|---|
| Unknown version/suite (fail-closed) | `0x0201`, `0x0101` |
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
| Stale/rollback record | `0x0105` (identity), `0x0302` (location record) |
| Org-directory not authority-signed | `0x0113` |
| Directory entry fails forward-verify | `0x0114` |
| Org-managed custody undisclosed | `0x0115` |
| Committer-fork-detected | `0x0404` |
| Committer-unreachable | `0x0405` |
| Session-revoked | `0x0504` |
| Session-expired | `0x0505` |
| Origin-mismatch (auth) | `0x0501` |
| Replay-detected | `0x0502` (auth nonce — the canonical replay instance; MOTE-level replay is structurally absorbed by content-addressed dedup, `0x020E`) |
| Expired | `0x020B` (MOTE `expires`), `0x0503` (auth challenge) |
| Quota/policy-deny | `0x070D`, `0x0806` (operator quotas), `0x070B` (recipient blocklist deny) |
| Gateway-attestation-invalid | `0x0601` |
| Postage-double-spend | `0x0708` |
| Issuer-untrusted | `0x0704` (token issuer), `0x0707` (postage issuer), `0x0509` (OIDC issuer) |

---

# Part 2 — IANA Considerations

## 21.13 Overview and policy vocabulary

DMTAP intends to pursue IETF standardization (§10.5). Upon submission of the governing
Internet-Draft, the registries below SHOULD be requested from IANA under "DMTAP Parameters."
Until then, **this appendix is the interim authoritative registry**, maintained per the
extension procedure in §21.23. Allocation policies use the standard terms of RFC 8126:

- **Standards Action** — requires a new RFC on the standards track.
- **Specification Required** — requires a stable, publicly available specification and review
  by a Designated Expert (§21.23), but not a new RFC.
- **First Come First Served (FCFS)** — no review; allocated on request.
- **Private Use** — not registered at all; meaningful only within a closed deployment or as a
  vendor extension, and MUST be namespaced (e.g. `x-` prefix) to avoid accidental collision.

## 21.14 DMTAP Error/Status Code Registry

| | |
|---|---|
| **Registry name** | DMTAP Error/Status Codes |
| **Reference** | §21.1–§21.11 (this document) |
| **Allocation policy** | New subsystem byte (`0x09`–`0xEF`): Standards Action. New code point within an existing subsystem (`NN` = `0x01`–`0x7F`): Specification Required. `NN` = `0x80`–`0xFE` within any subsystem: Private Use (implementation-local diagnostics; MUST map to the nearest standard code's Responder Action, §21.2, for any behavior visible to another implementation). `SS`/`NN` = `0x00` or `0xFF`: Reserved. |
| **Initial contents** | The 89 codes enumerated in §21.3–§21.11. |
| **Registry discipline** | Append-only. A retired code MUST be marked Deprecated, never deleted or reassigned to a different meaning (mirroring the append-only philosophy of the KT log, §3.5). |

## 21.15 Algorithm Suites Registry (`suite` u8)

| | |
|---|---|
| **Registry name** | DMTAP Algorithm Suites |
| **Reference** | §1.1, §16.7 |
| **Allocation policy** | `0x01`–`0x1F`: Standards Action (a suite changes the network's core interoperability and security floor — every conforming node must be able to reject or accept it correctly). `0x20`–`0xDF`: Specification Required. `0xE0`–`0xFE`: Private Use. `0x00`, `0xFF`: Reserved. |
| **Initial contents** | `0x01` (Ed25519 / X25519-HPKE / ChaCha20-Poly1305 / BLAKE3-256, v0 REQUIRED); `0x02` (Ed25519+ML-DSA-65 / X-Wing hybrid / ChaCha20-Poly1305 / BLAKE3-256, PQ target). |
| **Registration requirements** | A new suite registration MUST specify: signature scheme, KEM/PKE, AEAD, hash function, and a security analysis of the combination; MUST state its MLS-ciphersuite mapping if it is to be usable for group messaging (§5.1); MUST NOT be accepted by any conformant node until published here (unknown suites are rejected fail-closed, §1.1, §10.1 — there is no silent negotiation fallback beyond the documented suite-intersection rule, §1.3). |

## 21.16 Message Kinds Registry (`kind` u8)

| | |
|---|---|
| **Registry name** | DMTAP Message Kinds |
| **Reference** | §2.3, §10.1 |
| **Allocation policy** | `0x00`–`0x0A`: assigned (below). `0x0B`–`0x3F`: Unassigned — Standards Action (reserved for future *core* kinds sharing the wire-format guarantees of the initial set). `0x40`–`0x7F`: Specification Required (the extension range named in §2.3/§10.1). `0x80`–`0xFE`: Private Use. `0xFF`: Reserved. |
| **Initial contents** | `0x00 mail`, `0x01 chat`, `0x02 reaction`, `0x03 edit`, `0x04 redact`, `0x05 file_offer`, `0x06 group_event`, `0x07 receipt`, `0x08 presence`, `0x09 identity`, `0x0A system`. |
| **Forward-compatibility rule** | A node MUST NOT `ack` a `kind` it cannot validate, whether unassigned or merely unimplemented (§10.1, `0x020A`). It MAY ignore such a MOTE without error. This is the single rule that lets new kinds roll out without a flag day. |

## 21.17 Challenge Types Registry

| | |
|---|---|
| **Registry name** | DMTAP Anti-Abuse Challenge Types |
| **Reference** | §9.2 (`ChallengeSpec`), §2.2b |
| **Allocation policy** | Initial four types: assigned. Additional types: Specification Required — a new challenge type MUST specify its verification procedure (verifiable without decrypting the payload, per §2.2b), its issuer-trust model (per the §9.3.1 zero-default-budget rule, so a new type cannot bypass "cost for cold contact"), and its interaction with the §2.7a disposition (invalid/forged vs. absent/insufficient). `0x40`–`0xFE` (if a numeric tag encoding is used): Private Use. |
| **Initial contents** | `pow(bits)` (§9.4); `token(issuer)` — ARC-style anonymous rate-limited credential (§9.3); `stamp(amount)` — postage (§9.5); `vouch(guardianSet)` (§9.7). |

## 21.18 Name Backends Registry

| | |
|---|---|
| **Registry name** | DMTAP Name Backends |
| **Reference** | §3.1, §3.6, §3.9.2 |
| **Allocation policy** | Specification Required. A new backend MUST specify how it maps `name → ik` such that only `IK` can update the binding, and MUST state whether/how it composes with key transparency (§3.5) and with resolution order (§3.3). |
| **Initial contents** | `dns` (the default backend, §3.2 — TXT/SVCB records); `name-chain` (self-sovereign, un-seizable naming, §3.6 — the one place a blockchain is admitted, confined to this layer); `directory` (the opt-in flat `@handle` registry, §3.9.2, itself KT-audited). |
| **Note** | Onboarding Tiers A/B/C (§3.8) are deployment postures over the `dns` backend, not separate backends, and are not separately registered here. |

## 21.19 KT Log Identification Registry

| | |
|---|---|
| **Registry name** | DMTAP Key-Transparency Log Types |
| **Reference** | §3.5, DNS `kt=` parameter (§3.2) |
| **Allocation policy** | Specification Required. A log identifies itself by its own signing key and a **log-type tag** from this registry (no central log-ID authority is needed beyond the tag identifying *how to speak to* the log — the log's identity is its key, consistent with the rest of DMTAP). |
| **Initial contents** | `0x01` — **v0-minimal** (§3.5.1, the interoperable Core default): a single append-only Merkle log; signed tree heads (STHs); inclusion proofs; rollback defense (§3.3 step 2); owner self-monitoring (STH poll, §16.2); **no gossip and no federation** — tamper-evident and self-monitorable, but *not* equivocation-proof (§6.6 item 6). `0x02` — **v1-hardening** (§3.5.2, capability-negotiated per §10.2, **not** the default): a **federated set of independent, per-key append-only logs**, each publishing STHs at least every maximum merge delay (§16.2). A verifier pins a *set* and accepts a `name → ik` binding only on a **`> n/2` quorum** of that set (§3.5.2(b), mirroring the §5.1/§16.8 roster quorum), so no single log is authoritative. Verifiers, **monitors** (per-identity), and **auditors** (per-log) **gossip STHs** and cross-check **consistency proofs** (§3.5.2(a),(c)); a split view / append-only violation / stale-frozen head is detected and produces `ERR_KT_EQUIVOCATION` (`0x0107`), `ERR_KT_STH_INCONSISTENT` (`0x0110`), `ERR_KT_LOG_QUORUM_UNMET` (`0x0111`), or `ERR_KT_STH_STALE` (`0x0112`), with the mandated HALT_ALERT / fail-closed response of §3.5.2(d). A verifier MUST implement `0x01` and MAY additionally negotiate `0x02`; a node offered only `0x02` by a peer that itself supports only `0x01` uses `0x01`. |
| **Registration requirements** | A new log type MUST specify its proof format (STH structure, inclusion and consistency proofs — which MUST remain independently auditable per §3.5) and MUST NOT weaken the append-only, tamper-evident guarantee that makes KT meaningful. A log type that claims equivocation resistance MUST specify its gossip and cross-checking procedure and the responder actions for a detected split view (as `0x02` does via §3.5.2). |

## 21.20 Envelope/Payload Extension Header Keys Registry (`Headers.ext`)

| | |
|---|---|
| **Registry name** | DMTAP `Headers.ext` Keys |
| **Reference** | §2.4 (`ext: { * tstr => any }`), §10.1 |
| **Allocation policy** | Two namespaces. **Unprefixed keys** (e.g. a future `priority`, `delegate`): Specification Required — intended for genuinely interoperable, cross-implementation semantics. **`x-`-prefixed keys**: Private Use / FCFS — no registration required, reserved for experimentation and vendor-specific metadata, and MUST NOT be assumed portable across implementations. |
| **Initial contents** | None assigned at v0. Candidate future registrations flagged by the feature-parity audit (§17.6) but **not yet registered**: a delegate-attribution marker ("sent/edited by delegate device X of identity Y," items 23/43) and a cosmetic `priority` hint (item 27) — both Specification-Required candidates for a future revision, listed here for traceability only. |
| **Forward-compatibility rule (normative)** | A receiver MUST ignore any `ext` key it does not recognize; an unrecognized key MUST NOT cause validation failure or message rejection. This is what lets `ext` be extended without breaking older clients. |

## 21.21 DNS Parameter Keys Registry (`_dmtap` / `_dmtap-gw`)

| | |
|---|---|
| **Registry name** | DMTAP DNS TXT/SVCB Parameters |
| **Reference** | §3.2, §7.2a |
| **Allocation policy** | Specification Required. |
| **Initial contents** | Under `<name>._dmtap.<domain>` TXT: `v=` (format version), `suite=` (algorithm suite, §1.1), `ik=` (base64url identity public key), `id=` (hash of the current `Identity`, §1.3), `kt=` (KT log URL), `keypkgs=` (KeyPackage bundle locator, §5.3). Under `_dmtap.<domain>` SVCB: reserved service parameters and KT anchors. Under `<sel>._dmtap-gw.<domain>` TXT (§7.2a): `v=` (attestation scheme version), `k=` (gateway attestation public key). |
| **Registration requirements** | A new TXT/SVCB key MUST specify its grammar and whether it is required or optional for a conformant resolver; unrecognized keys in an existing record MUST be ignored by resolvers (same forward-compat rule as §21.20). |

## 21.22 Capability Tokens Registry (negotiation, §10.2)

| | |
|---|---|
| **Registry name** | DMTAP Capability Tokens |
| **Reference** | §10.2, `system` MOTEs (`kind = 0x0A`) |
| **Allocation policy** | Specification Required for tokens intended to be portable across implementations; `x-`-prefixed tokens are Private Use / FCFS, mirroring §21.20. |
| **Initial contents** | Supported-suite tokens (§1.1); privacy-tier tokens (`private`, `fast`, §4.6); supported MLS ciphersuite tokens (§5.1); supported extension-kind/extension-header tokens (cross-referencing §21.16/§21.20 registrations). |
| **Forward-compatibility rule** | A node receiving a capability token it does not recognize MUST ignore that token (not the whole `system` MOTE) and MUST NOT assume the counterpart lacks the capability merely because the token name is unfamiliar — absence of a recognized token is inconclusive, not a negative assertion. |

## 21.23 Extension & versioning procedure (normative)

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

4. **New challenge type, name backend, DNS parameter, or capability token.** Specification
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

## 21.24 Summary

- **Error/status codes defined:** 89 (`0x0101`–`0x0115`: 21, incl. the KT-v1 detection codes
  `0x0110`–`0x0112` and the org-administration codes `0x0113`–`0x0115` (§3.10); `0x0201`–`0x020E`: 14; `0x0301`–
  `0x0309`: 9; `0x0401`–`0x040A`: 10; `0x0501`–`0x050A`: 10; `0x0601`–`0x0604`: 4, plus the
  informative SMTP mapping table of §21.9; `0x0701`–`0x070E`: 14; `0x0801`–`0x0807`: 7),
  spanning the 8 requested subsystems, with every code resolving to exactly one of the 13
  defined responder actions (§21.2) — no undefined behavior remains.
- **IANA registries defined:** 10 — the 8 requested (Algorithm Suites, Message Kinds, Challenge
  Types, Name Backends, KT Log Types, `Headers.ext` Keys, DNS Parameters, Capability Tokens),
  plus the DMTAP Error/Status Code Registry itself (§21.14, needed to make Part 1 durable
  against future extension) and the extension/versioning procedure (§21.23) that governs all of
  them.
