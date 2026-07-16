# 19. Appendix B: Protocol Operations

This appendix is the **operational cross-reference** of the specification: every operation any
DMTAP role performs, specified to the depth needed to implement it without re-deriving it from
the narrative sections. Where a narrative section (§§0–16) already states a rule, this appendix
does not restate the rationale — it restates the rule as a normative, testable step and gives it
a worked trace. Where this appendix and a narrative section conflict, the narrative section
governs (§10.4) and the conflict is a bug in this appendix.

## 19.0 Conventions used in this appendix

**Spec block.** Every operation is specified with the same seven fields, in this order:

- **Purpose** — one paragraph, what the operation is for.
- **Initiator / Responder** — which role(s) perform which side. DMTAP is peer-to-peer, so these
  are not fixed "client"/"server" roles; they are named per operation (e.g. *sender* /
  *recipient node*, *RP* / *trusted client*).
- **Parameters** — each typed, marked **MUST** (required) or **OPTIONAL**, with its source
  section.
- **Preconditions** — what MUST already hold before the procedure may begin.
- **Procedure** — numbered, normative steps. A step that duplicates an ordering constraint from
  §2.7 or elsewhere keeps that section's step numbering visible in parentheses.
- **Success result** — the observable outcome and what state changes.
- **Failure modes** — every defined error condition and its response. No condition is left
  without an outcome (§2.7a's rule — "no undefined behavior" — is applied to every operation in
  this appendix, not only delivery).
- **Idempotency / retry** — whether re-invoking with the same parameters is safe, and what a
  retrying caller should expect.

Each operation also gets a **worked trace**: an annotated request/response exchange over the
actual DMTAP objects (CBOR maps shown as pseudo-JSON for readability; field names match §1–§16
verbatim). Traces use role tags in place of IMAP's fixed `C:`/`S:`, since DMTAP has no
client/server asymmetry at the protocol level:

```
A:    the initiating party (sender, resolver, RP, gateway — named per operation)
B:    the responding party (recipient node, resolver target, trusted client)
DHT:  the mesh DHT (§4.2), not a single peer
KT:   the key-transparency log (§3.5)
```

A trace line is `TAG → OBJECT{fields}` (send) or `TAG ← OBJECT{fields}` (receive), with a
trailing `# comment`. Fields irrelevant to the point being illustrated are elided with `…`.

**Error taxonomy.** Three outcome classes recur across every operation in this appendix, matching
§2.7a's precedent:

| Class | Meaning | Caller-visible? | Retryable? |
|-------|---------|------------------|------------|
| **Reject (fail-closed)** | Malformed, unauthenticated, or policy-violating input. The operation MUST NOT proceed and MUST NOT silently half-apply. | Yes, an explicit error | Only after the caller fixes the cause |
| **Defer** | Input is well-formed but cannot be authorized/decided yet (offline peer, unreachable issuer, pending quorum). | Yes, a pending/deferred state | Yes, per the operation's backoff |
| **Silent drop** | Reserved for the narrow, explicitly-listed cases where surfacing the failure to the sender would itself leak information the protocol is designed to hide (e.g. a forged envelope, §2.7a). Never the default. | No | N/A (sender's own retry logic, unaware, eventually expires it — §16.1) |

An operation's Failure-modes table classifies every listed condition into one of these three.

## 19.1 Naming operations (§3)

### 19.1.1 `resolve(name) → identity`

**Purpose.** Turn a human-facing `name@domain` (or self-sovereign name, §3.6) into a KT-verified,
pinned identity key, per the resolution algorithm of §3.3.

**Initiator / Responder.** Initiator: any node needing to address `name` for the first time
(resolving node, "A"). Responder: the DNS/name-backend resolver chain and the KT log ("KT");
there is no single peer responder.

**Parameters.**
- `name` (`tstr`, MUST) — `local@domain` form (§3.9.1) or a self-sovereign name (§3.6).
- `expected_suite` (`u8`, OPTIONAL) — if the caller already holds a prior pin for `name`
  (re-resolution after a claimed rotation), the suite it was last pinned under.
- `require_oob` (`bool`, OPTIONAL, default `false`) — caller requests that the operation refuse
  to complete without out-of-band verification (§3.4.1), regardless of KT availability.

**Preconditions.** None (this is the first-contact entry point). If `name` is already pinned
locally, callers SHOULD use the local pin directly and only invoke `resolve` again when the
pinned `Identity` chain (§1.3) itself signals a rotation/migration (§3.3 step 5).

**Procedure (normative; mirrors §3.3, numbered identically).**
1. DNS/name-backend lookup: query `<local>._dmtap.<domain>. TXT` (§3.2) → `{iks, id, kt,
   keypkgs}`. If no record resolves: **fail** `NAME_NOT_FOUND`.
2. **(first contact only)** Fetch a signed KT tree head + inclusion proof for `id` from `kt`
   (§3.5). If KT is unreachable, partitioned, or censored: apply the §3.3 fail-closed rule —
   refuse to pin, or (if the caller's policy allows) hard-warn and require explicit user
   acceptance. Silent TOFU is prohibited. If `require_oob` is set, this step is mandatory and
   a KT outage is a hard failure, not a warn-and-continue.
3. Fetch the full `Identity` object (§1.3) from the mesh by `id`; verify every `sig` in
   `Identity.sig` validates under the corresponding `iks[suite]`, and that the chain (`prev`) is
   consistent with anything previously pinned for this `name`. Reject on any signature failure
   or chain inconsistency (`CHAIN_INVALID`).
4. Pin `(name → iks, id)` locally (TOFU), recording the pin as **unverified** unless step 5 ran.
5. **(optional, if `require_oob` or user-initiated)** Perform out-of-band safety-number
   comparison (§3.4.1); on match, upgrade the pin to **verified**.
6. Return the pinned identity. Thereafter, routing for this contact uses the mesh (§4) by key;
   `resolve` is not invoked again for this contact unless the pinned `Identity` chain itself
   carries a new version.

**Success result.** A locally pinned `(name, iks, id, suites, keypkgs, pin_state ∈
{unverified, verified})` tuple, usable by `lookup-location` (§19.2.2) and `deliver` (§19.3.1).

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| No DNS/name-backend record for `name` | Reject | `NAME_NOT_FOUND`; caller MAY retry after user corrects the address |
| KT unreachable at first contact, `require_oob` unset | Reject (default) or Defer (explicit user override) | Refuse to pin; caller MAY re-invoke once KT is reachable, or accept an explicit unverified-pin warning per local policy |
| KT unreachable, `require_oob` set | Reject | Hard failure; no unverified fallback permitted |
| `Identity` signature fails to validate under any advertised suite | Reject | `IDENTITY_INVALID`; do not pin |
| `Identity.prev` chain inconsistent with a previously-seen version for this `name` (rollback/fork) | Reject | `CHAIN_INVALID`; surface as a security warning, never silently update (§3.4) |
| Only suites the resolving node does not implement are offered | Reject | `NO_COMMON_SUITE`; no silent downgrade (§1.3) |
| Resolution succeeds but returns a key that contradicts an **already-pinned** key for `name`, with no valid `KeyRotation`/`Identity` chain bridging old→new | Reject | Security warning; treat as `resolve` failure, not an update (§3.4) |

**Idempotency / retry.** Idempotent: re-invoking with the same `name` after a prior success
returns the same pin (or a newer one only via a valid chain). Safe to retry on `NAME_NOT_FOUND`
or KT-unreachable failures with backoff; retrying does not change any durable state until a
success path completes.

**Example trace.**
```
A: resolve("alice@example.org")
A → DNS: TXT? alice._dmtap.example.org
DNS ← A: "v=dmtap1; suite=1; ik=b64:MFk...; id=blake3:9f2a...; kt=https://kt.example.org/log; keypkgs=..."
A → KT:  signed-tree-head + inclusion-proof? id=blake3:9f2a...
KT ← A:  { tree_head, proof }                       # verifies; no newer version exists
A → mesh: fetch Identity by id=blake3:9f2a...
mesh ← A: Identity{ suites:[1], iks:{1: MFk...}, version:3, prev:blake3:7c0d..., names:["alice@example.org"], sig:[...] }
A: verify sig under iks[1]                          # OK
A: verify prev chain against nothing-prior (first contact) # OK, nothing to contradict
A: PIN (alice@example.org → ik=MFk..., id=9f2a..., pin_state=unverified)
A: resolve() → { ik: MFk..., suites:[1], keypkgs: ..., pin_state: unverified }
```

### 19.1.2 `publish-identity(identity)`

**Purpose.** Publish a new or updated `Identity` object (§1.3) — initial identity creation,
device addition/removal, suite migration, or any field change — to the transparency log and the
mesh, making it discoverable and auditable.

**Initiator / Responder.** Initiator: the identity owner's node holding `IK` (or a device
authorized per §1.2/§1.4). Responder: the KT log (append) and the mesh DHT/content store
(publish by content address).

**Parameters.**
- `identity` (`Identity`, MUST) — the fully-formed object (§1.3): `suites`, `iks`, `version`,
  `devices`, `keypkgs`, `recovery`, `names`, `prev`, `ts`, `sig`.
- `announce` (`bool`, OPTIONAL, default `true`) — whether to also push a signed `identity` kind
  MOTE (§2.3 kind `0x09`) to existing contacts announcing the change.

**Preconditions.**
1. `identity.version` = (previous published version) + 1, or `1` for initial publication.
2. `identity.prev` = hash of the previous published `Identity` (or absent, only for `version=1`).
3. `identity.sig` contains a valid signature under **every** suite listed in `identity.suites`
   (§1.3's multi-suite rule) — a partial signature set is invalid, not partially accepted.
4. For a `RecoveryPolicy` change carried transitively via `identity.recovery`: the *authorizing*
   principal is `IK` (proactive) or a satisfied `rotate_threshold` quorum (reactive) — never a
   single `admin`-capable device alone (§1.2, §1.4 rule 1–2).
5. If that `RecoveryPolicy` change **removes or weakens** any recovery factor (drops a method,
   lowers a threshold, evicts a guardian/device), it MUST satisfy `rotate_threshold` **even when
   signed by `IK` alone** (§1.4 rule 3, compromise defense) and MUST NOT take effect until its
   **veto/delay window** (72 h, §16.8) has elapsed (§1.4 rule 4). Additive, non-weakening changes
   are exempt from both. A veto/abort published within the window MUST itself satisfy
   `rotate_threshold` (asymmetric — a single prior factor cannot veto its own eviction, §1.4
   rule 4, §16.8).

**Procedure (normative).**
1. Validate preconditions locally before publishing (a node MUST NOT publish an object it cannot
   itself verify).
2. Append `identity` to the owner's KT log entry stream: submit to the log operator(s); receive
   a signed tree head + inclusion proof covering this entry.
3. Publish `identity` to the mesh DHT/content store, addressed by its content hash (so
   `resolve`'s step 3, §19.1.1, can fetch it).
4. Update the DNS/name-backend `id=` pointer (§3.2) to the new object's hash (out of protocol
   scope for *how* the domain record is edited — see §3.8 onboarding tiers — but the new `id`
   MUST be published before or atomically with the DNS update, never after, so there is no
   window where DNS points to a since-superseded object).
5. If `announce`, construct and send a `kind=0x09 identity` MOTE (§2.3) to every contact in the
   owner's local address book, carrying the new `Identity` (or a reference to it) so the
   owner's own devices (self-monitoring, §3.5) and contacts learn of the change without waiting
   for their own next `resolve`.
6. The owner's **other devices** monitor the KT log for entries under this identity and MUST
   alert the owner if a new version appears that none of the owner's devices initiated (intrusion
   detection, §3.5) — this is a standing background procedure, not a step of this operation, but
   `publish-identity` is precisely the event it watches for.

**Success result.** New `Identity` version durably in the KT log (with inclusion proof) and
discoverable on the mesh by content hash; existing contacts notified if `announce` was set.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| `version` is not exactly previous+1 | Reject | `VERSION_CONFLICT`; caller must re-fetch the current head and rebuild |
| `prev` does not match the hash of the currently-published object | Reject | `CHAIN_MISMATCH` (concurrent-publish race — see idempotency below) |
| Signature set does not cover every suite in `suites` | Reject | `INCOMPLETE_SIGNATURE`; publication refused, not partially accepted |
| `RecoveryPolicy` change authorized by less than `rotate_threshold` | Reject | `INSUFFICIENT_QUORUM`; §1.4 rule 1 (`ERR_RECOVERY_POLICY_UNAUTHENTICATED`, §21) |
| Factor-**weakening** `RecoveryPolicy` change signed by `IK` alone (no `rotate_threshold` quorum) | Reject | `ERR_RECOVERY_WEAKENING_UNQUORUMED`; §1.4 rule 3 — `IK` alone MUST NOT weaken recovery (stolen-`IK` takeover defense) |
| Weakening change attempts to take effect before its 72 h veto window elapses, or a lesser-bar weakening is detected within the window | Reject / hold | `ERR_RECOVERY_VETO_WINDOW`; §1.4 rule 4, §16.8 — hold until the window elapses; a `rotate_threshold`-backed veto aborts it |
| KT log operator unreachable | Defer | Publication is queued/retried; the DNS pointer (step 4) MUST NOT be updated until KT append succeeds, preserving the "never point past the log" invariant |
| DHT publish fails (no reachable peers to store at) | Defer | Retry with backoff; KT entry already exists so the object is still eventually fetchable once any one peer re-publishes it from the owner's own copy |

**Idempotency / retry.** **Not** naturally idempotent (each call advances `version`), but
concurrent-publish races are made safe by the `prev`-chain precondition: a second publish attempt
built against a stale `prev` is rejected (`CHAIN_MISMATCH`) rather than silently forking the
identity history, forcing the caller to rebase on the accepted head and retry.

**Example trace.**
```
A: publish-identity(Identity{ version:4, prev:blake3:9f2a..., devices:[...,new_phone_cert],
                              suites:[1], iks:{1:MFk...}, sig:[Ed25519(...)] })
A: local check: version 4 == prev-version(3)+1                     # OK
A: local check: sig covers suite 1 (the only suite in `suites`)     # OK
A → KT:  append(Identity{v4})
KT ← A:  { tree_head_42, inclusion_proof }
A → DHT: put(hash(Identity{v4}) → Identity{v4})
DHT ← A: stored at 20 closest peers
A: (announce=true) → contacts: MOTE{ kind:0x09, payload: ref(Identity{v4}) }  # to each pinned contact
A: publish-identity() → { version:4, tree_head:42 }
```

### 19.1.3 `publish-move(move_record)`

**Purpose.** Rebind a human name (`from` → `to`) while preserving the identity key, per §1.6,
distributing the rebinding via all three required channels.

**Initiator / Responder.** Initiator: identity owner's node (`IK`). Responders: KT log, mesh, and
every currently-known contact.

**Parameters.**
- `move_record` (`MoveRecord`, MUST) — `{suite, ik, from, to, ts, prev, sig}` (§1.6), `sig` by
  `IK`.

**Preconditions.**
1. `from` is a name currently in the owner's published `Identity.names`.
2. `to` is either a name the owner already controls resolution for (its own domain/self-
   sovereign backend, §3.6) or a newly claimed provider-issued name (§3.8 Tier B/C onboarding
   already completed for `to` before this call).
3. `move_record.sig` validates under the current pinned `IK`.

**Procedure (normative).**
1. Validate preconditions locally.
2. Publish `move_record` to the KT log (auditable, ordered — defeats a later squatter of `from`
   claiming continuity, §1.6).
3. Publish an updated `Identity` version (§19.1.2) whose `names` list adds `to` (and MAY retain
   `from` as a legacy alias per §3.9.4, or drop it if the domain is truly lost).
4. Update the mesh `LocationRecord`/`Identity.names` so future `resolve("to")` calls succeed
   immediately and existing key-based routing (mesh) is unaffected (contacts already routing by
   key never needed `to` at all).
5. Push a signed `kind=0x09 identity` MOTE carrying `move_record` to every existing contact
   (push to contacts, §1.6 item 3). Contacts verify `move_record.sig` against their **pinned**
   `IK` for the owner (not against a fresh `resolve` of `from`, which could be squatted after
   abandonment) and update their local display name; routing is unaffected because it was
   already key-based.

**Success result.** `to` resolves to the owner's key; `move_record` is durably logged; existing
contacts' clients display the new name and continue routing by the unchanged key.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| `from` not in the owner's current `Identity.names` | Reject | `UNKNOWN_SOURCE_NAME` |
| `move_record.sig` does not validate under pinned `IK` | Reject | `INVALID_MOVE`; refuse to publish |
| `to` is not yet resolvable (onboarding for `to` incomplete) | Reject | `TARGET_NOT_READY`; caller must complete §3.8 onboarding for `to` first |
| KT log unreachable | Defer | Queue and retry; do not push the contact-announcement MOTE (step 5) until the log entry (step 2) is durable, so contacts never learn of an unaudited move |
| A contact's client cannot verify `move_record.sig` against its pinned `IK` (e.g. the contact never actually had `alice` pinned, or the sig fails) | Reject (at the contact) | The contact's client MUST discard the move announcement and continue treating `from` as current; it MUST NOT adopt `to` on the strength of the MOTE content alone |

**Idempotency / retry.** Re-publishing the same `move_record` is safe (KT append is
content-addressed dedup, §2.2's "identical ciphertext shares an id" principle applies
analogously); re-sending the contact-announcement MOTE to a contact that already acked it is a
duplicate under §2.6 and is acked-without-reprocessing by that contact.

**Example trace.**
```
A: publish-move(MoveRecord{ ik:MFk..., from:"alice@oldhost.com", to:"alice@example.org",
                            ts:1752600000000, prev:blake3:aa11..., sig:Ed25519(...) })
A → KT:  append(MoveRecord)
KT ← A:  { tree_head_43, inclusion_proof }
A → DHT: update Identity.names += "alice@example.org"
A → bob (existing contact, routed by key, not by name):
      MOTE{ kind:0x09, to: bob_key, payload: { move_record } }
bob ← A: verifies move_record.sig against PINNED ik=MFk...      # matches, trusted
bob: updates local display name "alice@oldhost.com" → "alice@example.org"
bob: routing unaffected — still addressing ik=MFk... directly
bob → A: ack(id)
```

## 19.2 Reachability operations (§4)

### 19.2.1 `publish-location(location_record)`

**Purpose.** Publish/refresh the signed `key → location` `LocationRecord` (§4.2) that lets peers
find the node's current reachability hints (addresses, relay circuits) without a static IP.

**Initiator / Responder.** Initiator: the node itself (every node publishes its own record;
no third party publishes on a node's behalf). Responder: the DHT (the K closest peers to
`hash(ik)`, §4.2).

**Parameters.**
- `ik` (`bytes`, MUST) — identity key; the DHT key is `hash(ik)`.
- `peer_id` (`bytes`, MUST) — current libp2p peer id (MAY be per-epoch/unlinkable, §6).
- `addrs` (`[* multiaddr]`, MUST) — current reachability hints: direct addresses, relay
  circuits, mix addresses (§4.3).
- `ttl` (`u64`, MUST) — record lifetime; v0 default 2 h (§16.2).
- `seq` (`u64`, MUST) — monotonically increasing sequence number (rollback defense).
- `sig` (`bytes`, MUST) — signature by a device key over the above.

**Preconditions.**
1. `seq` strictly greater than the last `seq` this node has published (monotonic, never reused
   even across restarts — a node MUST persist its last-used `seq` or derive it from a
   monotonic clock to avoid accidental rollback after a crash).
2. The signing device key is a current, non-expired, non-revoked `DeviceCert` (§1.2).

**Procedure (normative; the IPNS-pattern value-record publish, §4.2).**
1. Assemble `LocationRecord{ik, peer_id, addrs, ttl, ts, sig}` with `seq` per the precondition.
2. Sign with a device key (not necessarily `IK` — location changes constantly and MUST NOT
   require the offline-capable root key).
3. Store the record at the **K closest peers** to `hash(ik)` (§16.2: `K=20`), using **S/Kademlia
   disjoint-path** lookups (≥3 node-disjoint paths) to reduce single-eclipse exposure (§4.2
   caution).
4. Schedule **aggressive republish**: re-invoke this operation at the republish interval (§16.2:
   45 min, jittered), strictly before `ttl` elapses, incrementing `seq` each time even if
   `addrs` is unchanged (a stale-but-still-valid record is itself a failure mode the mesh must
   avoid, §4.2).

**Success result.** The record is stored (with best-effort replication) at the K closest peers;
a `lookup-location` for `ik` by any peer within `ttl` returns this record.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| `seq` not strictly greater than the node's last-published value | Reject (local, before send) | Node MUST increment and retry locally; never publish a stale-or-equal `seq` |
| Signing device key expired/revoked | Reject | `DEVICE_UNAUTHORIZED`; node must re-derive from a current device key |
| Fewer than a usable quorum of the K closest peers accept the store (partial write, e.g. under active eclipse) | Defer | Retry via disjoint paths; the record is still *discoverable* if any honest peer among the K holds it — full failure is silent only in the sense that the initiator cannot always detect an eclipse (§4.2 caution: this is the DHT's structurally weakest point, not a gap in this operation's definition) |
| No DHT peers reachable at all (total network partition) | Defer | Rely on non-DHT paths for existing contacts (cached direct addresses, relay-reservation/rendezvous addresses, §4.2) until connectivity returns; republish resumes automatically |

**Idempotency / retry.** Each invocation MUST use a fresh, strictly-increasing `seq`, so exact
re-invocation with identical parameters is not meaningful — but re-publishing after a failed
attempt with a bumped `seq` is always safe and is exactly what the republish schedule does.

**Example trace.**
```
A: publish-location(ik=MFk..., peer_id=Qm7f..., addrs=[/ip6/.../udp/443/quic, /relay-circuit/...],
                    ttl=7200000, seq=118)
A: local check: 118 > last_published_seq(117)                    # OK
A: sign with device_key "home-box"
A → DHT: put(hash(MFk...), LocationRecord{seq:118, ...}) via 3 disjoint paths
DHT ← A: stored at 18/20 of the K closest peers (2 timed out — acceptable, quorum met)
A: schedule republish at now+45min (jitter ±5min), seq→119
A: publish-location() → { stored_at: 18, ttl_expires: now+2h }
```

### 19.2.2 `lookup-location(ik) → LocationRecord`

**Purpose.** Retrieve the current `key → location` record for a peer, so a sender can attempt
delivery over the reachability ladder (§19.2.3).

**Initiator / Responder.** Initiator: any node wanting to reach `ik`. Responder: the DHT (the K
closest peers to `hash(ik)`), or a non-DHT path (cached address, rendezvous) tried first per
§4.2's resolution order.

**Parameters.**
- `ik` (`bytes`, MUST) — the target identity key.
- `allow_dht` (`bool`, OPTIONAL, default `true`) — whether to fall back to the public DHT if
  non-DHT paths fail; a closed/organizational deployment MAY set this `false` and use only its
  private DHT (§4.2).

**Preconditions.** None (this is itself the discovery step); however per §4.2, a node SHOULD
attempt non-DHT paths first regardless of whether it has ever contacted `ik` before.

**Procedure (normative; §4.2's resolution order).**
1. Try **cached direct addresses** last successfully used for `ik` (if any). If a cached address
   answers a liveness probe, return it immediately — skip DHT entirely.
2. Try **relay-reservation / rendezvous ("home relay") addresses** if the caller has an
   out-of-band or previously-learned rendezvous hint for `ik`. This step exists precisely so a
   fresh contact is not 100% dependent on a hostile public-DHT lookup (§4.2).
3. **DHT fallback** (only if `allow_dht`): query the K closest peers to `hash(ik)` via
   S/Kademlia disjoint-path lookups; among returned candidate records, accept only the one with
   the **highest `seq`** (rollback defense) whose `sig` validates under a current device key of
   `ik`'s pinned `Identity`. Discard any record with `seq` ≤ a previously-seen `seq` for this
   `ik` (replay/rollback).
4. If multiple returned records disagree (different peers return different "highest" records) —
   a signature that the eclipse/censorship caution (§4.2) warns can happen — prefer the record
   with the highest `seq` that a majority of the disjoint-path queries agree on; if no majority
   exists, treat as `LOCATION_UNRESOLVED` (do not guess).

**Success result.** A verified `LocationRecord` for `ik`, usable to attempt the reachability
ladder (§19.2.3).

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| No cached, rendezvous, or DHT record found | Defer | `LOCATION_UNRESOLVED`; caller falls back to store-and-forward via mixnet/relay buffering (§4.7) and retries the lookup on its own schedule |
| A found record's `sig` does not validate under `ik`'s current pinned devices | Reject | Discard the record; treat as not found |
| A found record's `seq` is ≤ the highest `seq` already seen for `ik` (stale/replayed/rollback) | Reject | Discard; this is expected background noise from an aggressively-republished DHT, not necessarily an attack, but MUST NOT be accepted as fresher |
| Disjoint-path queries disagree with no majority (possible eclipse, §4.2 caution) | Defer | `LOCATION_UNRESOLVED`; do not act on a minority-returned record; retry with different peer set / wait for re-publish |
| `allow_dht=false` and no non-DHT path exists | Reject | `LOCATION_UNRESOLVED`; the deployment's own policy has excluded the only remaining path |

**Idempotency / retry.** Fully idempotent (a pure read); safe to retry at will. Because DHT
record lifetimes are short (§16.2 TTL 2 h), a caller SHOULD NOT cache a `lookup-location` result
past the record's own `ttl`.

**Example trace.**
```
A: lookup-location(ik=MFk...)
A: cache check: no prior direct address for MFk...
A: rendezvous check: no known home-relay hint for MFk...
A → DHT: get(hash(MFk...)) via 3 disjoint paths, allow_dht=true
DHT ← A: path1 → LocationRecord{seq:118,...}; path2 → LocationRecord{seq:118,...};
         path3 → LocationRecord{seq:104,...}       # stale, from an eclipse-adjacent peer
A: majority (2/3) agree on seq:118 → accept it; discard seq:104
A: verify sig under MFk...'s current device key            # OK
A: lookup-location() → LocationRecord{seq:118, addrs:[...], ttl_expires: now+~1h50m}
```

### 19.2.3 The reachability ladder (attempt sequence)

**Purpose.** Given a resolved `LocationRecord`, attempt an actual connection to the peer in the
order that prefers direct connectivity and falls back only as needed (§4.3), because rungs 1–2
are cheaper, faster, and expose no third-party relay.

**Initiator / Responder.** Initiator: the sending node. Responder: the target node (and, for
rung 3, a relay operator's node).

**Parameters.**
- `location` (`LocationRecord`, MUST) — from `lookup-location` (§19.2.2).
- `deadline` (`u64`, OPTIONAL) — caller's own timeout for exhausting the ladder before falling
  back to store-and-forward (mixnet buffering, sender retry, §4.7); if absent, use the
  operation's own per-rung timeouts summed.

**Preconditions.** A valid, non-expired `LocationRecord` for the target (§19.2.2's success
result).

**Procedure (normative; try in order, fall down only as needed — §4.3).**
1. **Direct.** Attempt a direct connection using `location.addrs` entries that are non-relay
   multiaddrs (IPv6 preferred; IPv4 with port-forward/UPnP acceptable). No relay involved. If a
   direct connection completes its transport handshake (Noise/TLS 1.3, §4.1), STOP — success.
2. **Hole-punch.** If step 1 fails (both sides behind NAT), attempt **AutoNAT v2 + DCUtR**
   coordinated hole-punching. This requires both nodes reachably online simultaneously — true by
   construction for two always-on boxes. If the punched connection completes, STOP — success.
3. **Circuit relay.** If steps 1–2 fail, use a **circuit relay v2** hop from `location.addrs`'
   relay-circuit entries (or a discovered relay per §14.5). The relay sees ciphertext only
   (content-blind); it is a reachability hop, not a store. If a relay circuit connects, STOP —
   success (with the caveat that this rung carries the weakest metadata privacy of the three,
   §6.6 item 2 as it applies to bulk; for ordinary MOTE delivery over relay, content remains
   E2E-encrypted regardless of rung).
4. **All rungs exhausted.** Fall back to store-and-forward: the sender's retry queue (§4.7) holds
   the MOTE and retries per the backoff schedule (§16.1); if a peer-buffering buddy node or a
   relay-mailbox is configured for the target, offer it there (§4.3, §14.5) rather than failing
   immediately.

**Success result.** An established transport connection at the lowest-numbered rung that
worked, over which `deliver` (§19.3.1) proceeds.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Rung 1 fails (no direct path) | Defer (internal to the ladder) | Fall to rung 2; not a caller-visible failure |
| Rung 2 fails (hole-punch coordination fails or one side is not simultaneously online) | Defer | Fall to rung 3 |
| Rung 3 fails (no relay reachable, or all relay reservations exhausted, §16.6 caps) | Defer | Fall to store-and-forward (§4.7); caller sees the MOTE enter `RETRY` state (§4.7's sender state machine), not an immediate error |
| All rungs fail and `deadline` elapses | Reject (to the caller of the *higher-level* `deliver` operation, not this one) | The delivery attempt reports `PEER_UNREACHABLE` for this attempt; the outer retry/backoff (§19.3.3) governs what happens next — this is not the same as `EXPIRED` (§16.1), which is the sender-retry-deadline outcome |

**Idempotency / retry.** The ladder itself is stateless per attempt; re-running it is always
safe and is exactly what the sender-retry state machine (§19.3.3, §4.7) does on each backoff
tick — it does not remember which rung last succeeded, since NAT/network conditions change.

**Example trace.**
```
A: attempt-reachability(location=LocationRecord{addrs:[/ip6/2001:db8::.../udp/443/quic,
                                                        /p2p-circuit/.../relay1]})
A → B: QUIC direct dial /ip6/2001:db8::.../udp/443/quic
B ⇏ A: (timeout — B is behind CGNAT with no IPv6, direct fails)
A: rung 1 FAILED → try rung 2
A ⇄ AutoNAT/DCUtR ⇄ B: coordinated hole-punch attempt
A ⇄ B: hole-punch SUCCEEDS                                    # both online, punch completes
A: reachability established via rung 2 (hole-punch)
A: proceed to deliver() over this connection
```

## 19.3 Delivery operations (§2.6, §2.7, §2.7a)

### 19.3.1 `deliver(outer_mote)`

**Purpose.** The recipient-side procedure that receives a MOTE off the wire, validates it in the
cheapest-first order (§2.7), and either stores it for the user, defers it to the requests area
(§2.7a), or silently drops it. This is the single most security-critical operation in the
protocol: every failure path is defined, matching §2.7's decryption-DoS defense.

**Initiator / Responder.** Initiator: the sending node (over whatever transport rung §19.2.3
established, or via the mixnet, §4.4). Responder: the recipient's node — the party that runs
this entire procedure.

**Parameters.**
- `outer_mote` (`Envelope`, MUST) — the full envelope as defined in §2.2: `v, suite, id, to,
  epoch, ts, kind, keypkg, challenge, ciphertext, sender_sig`. (The "outer" mixnet wrapper, if
  present, has already been peeled by the time this operation's input is the `Envelope`; onion
  unwrapping is a transport-layer concern of §4.4, not restated here.)

**Preconditions.** None — this operation is the entry point for unauthenticated network input by
definition, which is exactly why its ordering is normatively fixed (§2.7).

**Procedure (normative — reproduces §2.7 verbatim as the responder's procedure, with the
§2.7a disposition rule folded into step 6/9).**
1. Reject unknown `v`/`suite` (fail closed). → **Silent drop** if this is the outer-most check
   with no `sender_sig` yet verifiable; treated as malformed input.
2. Verify `id` matches the content address of `ciphertext` (BLAKE3-256 by default, §2.2). Drop
   on mismatch.
3. Verify `sender_sig` over `(id ‖ to ‖ ts ‖ kind ‖ challenge)` under the envelope's ephemeral
   key — cheap, no decryption. Drop on failure.
4. **Resolve `to`** to this node's own key, or a group this node belongs to (`DeliveryTag`
   resolution, §2.2a: identity key, group id, or blinded tag recognized via the node's own
   per-contact secret). If `to` does not resolve to anything this node holds, drop (this node is
   not a valid recipient — do not guess or forward).
5. **Classify the sender**: known contact (pinned `to`/blinded-tag state matches an existing
   contact) vs. unknown/cold sender.
6. **Cold-sender gate (§9, before decryption).** If cold, evaluate `challenge` against local
   `Policy` (§9.2) — ARC token validity + issuer trust (§9.3.1), PoW solution (§9.4), postage
   validity (§9.5.1), or vouch (§9.7) — entirely without decrypting `ciphertext`. Known contacts
   skip this step. The outcome is exactly the §2.7a table:
   - Invalid/forged `challenge` (fails cryptographically, e.g. bad ARC signature, invalid PoW,
     double-spent postage serial) → **silent drop**, do not `ack` (unless `id` is a duplicate,
     which is acked per step 9's dedup rule).
   - Absent, or present but below the recipient's policy threshold → **defer to the requests
     area** (§2.7a): stored undecrypted-state-pending or decrypted-but-quarantined per
     implementation choice (implementations MAY decrypt to render a preview in the requests
     area, since decryption itself is not the resource being protected once the sender has
     *some* accountable proof — but MUST NOT surface it as an inbox message). Rate-limited per
     §9.2's `RateLimit`. Retained for the requests-area retention window (§16.5: 30 days), then
     purged if never promoted.
   - Valid and at/above threshold → proceed to step 7 exactly as a known contact would.
7. Decrypt `ciphertext` (MLS epoch key for group `epoch`, or HPKE to the recipient's key for 1:1
   §5.3 async-init). Drop on decryption failure (wrong epoch, corrupt ciphertext, key not held).
8. Verify `Payload.sig` under `Payload.from`. For a known contact, `from` MUST match the pinned
   identity (§3.4); a mismatch is treated as a forged/relayed message — drop, do not ack. For a
   cold sender whose `from` is only now revealed (post-decryption), re-apply the recipient's
   block/allow lists against the now-known identity — a sender that passed the anonymous
   challenge gate (step 6) but is on an explicit per-identity block list (§9.2 `block`) is still
   rejected here.
9. Apply `expires`/`refs`/`kind` semantics (§2.4, §2.3); **store** the MOTE. If step 6/8 cleared
   it fully → store to the **inbox** and **`ack`** (§19.3.2). If step 6 only *deferred* it → hold
   it in the **requests area** (durably, 30 days, §16.5) but **do NOT `ack`**: an unproven cold
   sender is not owed a receipt confirmation (acking would confirm the recipient's existence and
   falsely signal *delivered* when the MOTE is merely pending review), and the sender's own retry
   independently reaches `EXPIRED` (§16.1, 72 h) — consistent with §2.7a and §20.2. A MOTE whose
   `id` this node **already holds** (dedup, §2.6) is acked immediately at whichever step the
   duplicate is detected, without re-running the remaining steps.

**Success result.** The MOTE is stored (inbox or requests area). Exactly one of three terminal
states is reached for every input: **stored+acked** (inbox), **deferred+unacked** (durably held
in the requests area, but no receipt is sent — the sender's own retry expires), or
**dropped+unacked** (silent, for cryptographically invalid input per §2.7a) — there is no fourth,
undefined outcome. The ack axis is binary: **ack iff delivered to the inbox** (or a dedup of one
already held); deferred and dropped are both unacked, differing only in retention.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Unknown `v`/`suite` | Silent drop | No ack; sender's retry eventually `EXPIRED`s (§16.1) — this is intentional: a node speaking an unknown future version should not learn version-support signals from an ack/nack asymmetry |
| `id` ≠ content-address of `ciphertext` | Silent drop | Same as above — a corrupted or tampered envelope gets no signal back |
| `sender_sig` fails to verify | Silent drop | No ack — forged envelope |
| `to` does not resolve at this node | Silent drop | This node is not a valid recipient; MUST NOT forward or leak that it received *something* |
| Cold sender, `challenge` invalid/forged | Silent drop | Per §2.7a exactly |
| Cold sender, `challenge` absent/below threshold | Defer to requests area, **no ack** | Rate-limited (§9.2), retained 30 days (§16.5), never silently discarded and never shown as an inbox message (§2.7a's two MUST NOTs); not acked (don't confirm receipt to an unproven cold sender — the sender's own retry `EXPIRED`s) |
| Decryption fails (wrong epoch/key/corrupt) | Silent drop | No ack — indistinguishable from a forged envelope to the sender, by design (does not confirm epoch/key state to an attacker) |
| `Payload.sig` fails under `Payload.from`, OR `from` mismatches a known contact's pin | Silent drop | No ack — a passed anti-abuse gate does not substitute for payload authenticity |
| `from` (revealed post-decrypt) is on the recipient's explicit block list | Silent drop | No ack — step 6 passing (anonymous accountability) does not override an explicit identity-level block once identity is known |
| Duplicate `id` (already held) | N/A — not a failure | `ack` immediately, no re-processing (§2.6 dedup) |
| `expires` in the past relative to receipt (already-expired MOTE) | Reject (soft) | Store per `kind` semantics but apply client-enforced expiry immediately (i.e. it may appear and be immediately removed) — `expires` is a client hint, not a delivery gate (§6.6 item 8: cooperative, not enforced by the network) |

**Idempotency / retry.** Fully idempotent on `id`: re-delivering an already-acked `id` is a
no-op that still results in an `ack` (dedup, step 9). This is what makes sender-side retry
(§19.3.3) safe to run blindly on any timeout without a separate "did they already get it"
check.

**Example trace (known contact, fast path).**
```
A → B: Envelope{ v:0, suite:1, id:blake3:7ab2..., to: bob_ik, epoch:null, ts:..., kind:0x01,
                 challenge:null, ciphertext:..., sender_sig:Ed25519(...) }
B: step1 v/suite known                                          # OK
B: step2 recompute blake3(ciphertext) == id                      # OK
B: step3 verify sender_sig under envelope's ephemeral key        # OK
B: step4 resolve to=bob_ik → this node                            # OK, matches
B: step5 classify: alice_ik is a pinned known contact             # KNOWN
B: step6 SKIPPED (known contact, §2.7 "known contacts MAY skip step 6")
B: step7 decrypt ciphertext (HPKE to bob's key)                  # OK
B: step8 verify Payload.sig under Payload.from == alice_ik (pinned) # matches
B: step9 store to inbox; kind=0x01 chat rendered in Talk thread
B → A: ack(id=blake3:7ab2...)
```

**Example trace (cold sender, deferred to requests area).**
```
C → B: Envelope{ ..., to: bob_ik, kind:0x00, challenge: ARC{issuer:"unknown-node-xyz", ...},
                 ciphertext:..., sender_sig:... }
B: steps 1–4 pass as above
B: step5 classify: sender's blinded tag/key unrecognized → COLD
B: step6 evaluate challenge: ARC token, but issuer "unknown-node-xyz" is unvetted
         → per §9.3.1 unvetted issuer default rate budget = 0 → treated as "no token"
         → falls back to policy: no PoW/postage attached either → BELOW THRESHOLD
B: DEFER → store in requests area (undecrypted or preview-decrypted per implementation),
           rate-limit counter for this challenge-class incremented, retention clock started (30d)
B → C: ack(id)                                                    # deferred MOTEs ARE acked —
                                                                   # only invalid/forged ones are not
```

### 19.3.2 `ack(id)`

**Purpose.** Confirm receipt of a specific MOTE `id` to the sender, terminating that MOTE's
sender-side retry loop (§4.7).

**Initiator / Responder.** Initiator: the recipient node (sent as a consequence of `deliver`,
§19.3.1). Responder: the sender's retry-queue state machine (§19.3.3), which consumes the ack.

**Parameters.**
- `id` (`bytes`, MUST) — the content address being acknowledged.
- `ack_sig` (`bytes`, OPTIONAL) — implementations MAY sign the ack (e.g. under the same ephemeral
  mechanism as `sender_sig`) so a relay cannot forge acks on the recipient's behalf; this is an
  implementation hardening, not specified further at the object-format level in v0.

**Preconditions.** The recipient has completed `deliver`'s procedure for `id` to a terminal state
that the §2.7a table marks as ack-eligible: **stored** (inbox) or **deferred** (requests area).
A **silently dropped** MOTE MUST NOT be acked (§2.7a).

**Procedure (normative).**
1. Construct a minimal `ack{id}` message.
2. Send it back over the same channel/rung the `Envelope` arrived on if still open, or via a
   fresh `deliver`-style send addressed to the original sender's key if the channel has since
   closed (an ack is itself routed like any other small MOTE-adjacent message — it does not
   require a dedicated wire object beyond `id`).

**Success result.** The sender's retry-queue entry for `id` transitions to `ACKED` (§4.7) and is
removed from the retry schedule.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Ack is sent but never reaches the sender (network loss) | Defer (at the sender) | The sender's retry queue does not learn of the ack and re-sends; the recipient's `deliver` dedup (§2.6, §19.3.1 step 9) absorbs the duplicate and re-acks — eventually consistent, never a correctness problem, only a bandwidth cost |
| Recipient tries to ack an `id` it silently dropped (implementation bug) | Reject (protocol-level should-never-happen) | MUST NOT occur per §2.7a; if it does, the sender incorrectly believes a forged/invalid message was delivered — implementations MUST guard this invariant in code, since the spec provides no wire-level defense against a buggy recipient acking garbage |

**Idempotency / retry.** Sending the same `ack(id)` multiple times is harmless — the sender's
retry-queue transition to `ACKED` is itself idempotent (an already-acked entry receiving another
ack is a no-op).

**Example trace.** See `deliver`'s traces above — both end in `B → A: ack(id=...)`.

### 19.3.3 Sender retry state machine (dedup and durability)

**Purpose.** Own durability from the sender's side, since the mixnet/relay middle holds nothing
(§0.5, §4.7): retry an unacked MOTE with backoff until `ack` or the retry deadline expires.

**Initiator / Responder.** Initiator/owner: the sending node. There is no "responder" — this is
a purely local state machine driven by the outcomes of `attempt-reachability` (§19.2.3),
`deliver`'s remote effect (whether an `ack` comes back), and wall-clock time.

**Parameters.**
- `mote_id` (`bytes`, MUST) — the MOTE being tracked.
- `expires` (`u64`, OPTIONAL) — the MOTE's own requested expiry (§2.4); bounds the retry
  deadline from above if smaller than the default.
- `retry_deadline` (`u64`, OPTIONAL, default per §16.1: 72 h) — the hard ceiling after which the
  MOTE fails to the user regardless of `expires`.

**Preconditions.** A MOTE has been constructed and handed to the outbound queue (i.e. it has left
the `QUEUED`/draft state, §4.7).

**Procedure (normative state machine, §4.7).**
1. `QUEUED` → the MOTE is sealed (sender-blinded, onion-wrapped if `private` tier, §6.2) →
   `SEALED`.
2. `SEALED` → an in-flight send attempt is made (mixnet path for `private`, direct/reachability-
   ladder for `fast`) → `IN_FLIGHT`.
3. On `ack(id)` received → `ACKED`. Terminal; remove from the retry queue.
4. On send failure or no `ack` within the current backoff window → `RETRY`: re-attempt with
   **exponential backoff** (§16.1: base 30 s, cap 1 h, with jitter), returning to step 2 each
   time (a fresh reachability-ladder attempt, since network conditions may have changed).
5. If `retry_deadline` (or the smaller of `expires`, if set) elapses while still un-acked →
   `EXPIRED`. Terminal; the sender's client notifies the user the message could not be
   delivered (§4.7).

**Success result.** Either `ACKED` (delivered) or `EXPIRED` (failed, user-notified) — no MOTE is
left in indefinite limbo; every entry reaches one of exactly two terminal states.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Recipient permanently unreachable (deleted identity, permanently offline) | Reject (eventual) | `EXPIRED` at `retry_deadline`; user notified — the protocol has no mechanism to distinguish "temporarily down" from "gone forever" before the deadline, by design (no central presence oracle) |
| `expires` requested shorter than the default retry deadline | Defer, bounded | The smaller of the two governs — a MOTE MUST NOT be retried past its own requested expiry even if the default deadline is longer |
| Sender's own node restarts mid-retry | Defer | The retry queue MUST be durable across restart (it is the sole durability mechanism, §0.5) — an implementation that loses queued-but-unacked MOTEs on restart violates the "durability lives entirely in this sender-side queue" invariant (§4.7) |
| A duplicate `ack` arrives for an already-`ACKED`/removed entry | N/A | Ignored; no state change (idempotent, §19.3.2) |

**Idempotency / retry.** The entire point of this operation is retry-safety: every re-attempt at
step 2 is a fresh, idempotent send of the same immutable `Envelope` (same `id`), relying on the
recipient's dedup (§2.6) to make repeated delivery harmless.

**Example trace.**
```
A: QUEUED  mote_id=blake3:7ab2...
A: SEALED  (onion-wrapped, private tier)
A: IN_FLIGHT  attempt 1 via mixnet
   ... 90s pass, no ack ...
A: RETRY  backoff=30s → attempt 2
   ... no ack ...
A: RETRY  backoff=60s → attempt 3
A ← B: ack(id=blake3:7ab2...)
A: ACKED  — removed from retry queue
```

## 19.4 Async session initiation (§5.3)

### 19.4.1 `fetch-keypackage(ik) → KeyPackage`

**Purpose.** Retrieve a signed MLS KeyPackage (MLS's prekey) for an identity so a sender can
initiate an encrypted session with a peer whose devices are all currently offline. The **default**
MLS path needs no separate PQXDH/X3DH protocol (§5.3); the **optional** deniable 1:1 mode (§5.2.1)
does use X3DH/PQXDH and fetches a `DeniablePrekeyBundle` (via `Identity.deniable_prekeys`) by the
same locate-and-pin pattern shown below.

**Initiator / Responder.** Initiator: the node wishing to start a session (sender). Responder:
the mesh (KeyPackages are located via `Identity.keypkgs`, §1.3, and fetched from
wherever the owner's node last published them — typically the mesh content store, republished
by the owner's own node).

**Parameters.**
- `ik` (`bytes`, MUST) — target identity key (already resolved, §19.1.1).
- `suite` (`u8`, OPTIONAL) — preferred suite; if absent, use the highest suite common to sender
  and `ik`'s advertised `Identity.suites` (§1.3's negotiation rule).

**Preconditions.** `ik` has been `resolve`d and its `Identity.keypkgs` (a `KeyPackageBundleRef`)
is known.

**Procedure (normative).**
1. Fetch the `KeyPackageBundleRef`-addressed bundle from the mesh.
2. Select one `KeyPackage` matching the negotiated `suite` (§1.3: "per-message suite is
   negotiated at KeyPackage granularity"). Prefer a **one-time** KeyPackage if available (MLS's
   consume-once prekey); fall back to a **last-resort** KeyPackage (§5.3) only if no one-time
   package remains, so an identity is never left un-initiable purely from prekey exhaustion.
3. Verify the `KeyPackage`'s own signature under a current device key of `ik`'s `Identity`
   (a `KeyPackage` not traceable to a currently-valid device is untrustworthy — reject it and try
   the next available package, if any).
4. Mark the selected one-time `KeyPackage`, if used, as **consumed** locally to avoid re-offering
   it (the owner's node performs the authoritative consumption/replenishment; the fetching
   sender's local marking is advisory only, to avoid retrying an already-used package against
   the same responder in a race).

**Success result.** A verified `KeyPackage` usable as the `Add` target in `add-member`
(§19.4.2).

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| No `KeyPackage` bundle published (or bundle empty — full exhaustion) | Defer | Retry later; if the owner's node has genuinely exhausted even its last-resort package, the initiator falls back to queuing the intent and re-fetching on the owner's next replenishment (out of scope for wire format — a client-level retry) |
| Fetched `KeyPackage`'s signature does not validate under any current device key | Reject | `KEYPACKAGE_INVALID`; do not use it — try another if the bundle has more |
| No common suite between sender and `ik`'s advertised suites | Reject | `NO_COMMON_SUITE`; session cannot be initiated (§1.3: fail closed, no downgrade) |
| A one-time `KeyPackage` is fetched by two concurrent initiators (race) | Defer (for the loser) | The owner's node is authoritative for consumption; the second initiator's `Add`/`Welcome` (§19.4.2) will be rejected by the owner as referencing an already-consumed package, and MUST re-fetch a fresh package and retry — not treated as a security failure, just a race |

**Idempotency / retry.** Fetching is idempotent as a read; consuming a one-time package is
explicitly **not** idempotent (that is its purpose) — a retry after a consumption race MUST
fetch a fresh package, not resubmit the same one.

**Example trace.**
```
A: fetch-keypackage(ik=bob_ik, suite=1)
A → mesh: get(Identity(bob_ik).keypkgs)
mesh ← A: KeyPackageBundle{ packages: [KP_onetime_17, KP_onetime_18, KP_lastresort] }
A: select KP_onetime_17 (suite 1, one-time)
A: verify KP_onetime_17.sig under bob's device_key "home-box"     # OK
A: mark KP_onetime_17 consumed (local, advisory)
A: fetch-keypackage() → KP_onetime_17
```

### 19.4.2 `add-member(group_or_1to1, keypackage) → {Commit, Welcome}`

**Purpose.** Bring an offline party into an encrypted session (a new 1:1, or an existing group)
via MLS's native async-join mechanism: an `Add` Commit plus a `Welcome` message the joining party
uses on next contact (§5.3 step 2).

**Initiator / Responder.** Initiator: an existing member (for a group) or the party starting a
1:1 (a 1:1 is a 2-member group from creation, §5.1). Responder: the group's committer (§5.1) for
ordering, and the invitee (eventual, asynchronous responder via `Welcome` consumption).

**Parameters.**
- `group_state` (opaque MLS group context, MUST) — the current `GroupInfo`/ratchet-tree state
  (or, for a fresh 1:1, the freshly-created 2-member group state).
- `keypackage` (`KeyPackage`, MUST) — from `fetch-keypackage` (§19.4.1).
- `role` (`tstr`, OPTIONAL) — for group adds beyond a bare 1:1, the invitee's initial role
  (`member`/`poster`/`reader`, §5.8.2); requires `admin` capability on the initiator (see
  §19.5.2's precondition, which this operation shares when used for an existing group).

**Preconditions.**
1. The initiator holds current, valid group state (is a current member with the necessary
   capability — `admin` for a pre-existing group per §5.8.2; any member for a fresh 1:1).
2. `keypackage` has been verified per `fetch-keypackage`'s step 3.
3. The initiator can reach the group's **committer** (§5.1) to submit the `Add` Commit for
   ordering — see §19.5.5 if the committer is unreachable.

**Procedure (normative).**
1. Construct an MLS `Add` proposal referencing `keypackage`.
2. Submit the proposal (bundled into a `Commit`) to the group's ordered handshake channel — the
   committer (§5.1). The committer appends it to the hash-chained per-group log at the next
   position.
3. On the Commit being accepted into the log, all current members apply it, advancing the
   group's `epoch`.
4. Construct a `Welcome` message (MLS-standard: encrypted group secrets + the ratchet tree,
   addressed to the new member's `keypackage`'s init key) and send it as a `kind=0x06
   group_event` MOTE (§2.3) to the invitee's key, `to = keypackage`'s owning identity — this
   MOTE travels like any other (subject to the invitee's own cold-sender gate, §2.7, if the
   inviter is not already a known contact of the invitee — see the failure-mode note below).
5. The invitee, on eventually coming online and receiving the `Welcome` MOTE (via ordinary
   `deliver`, §19.3.1), uses it to bootstrap local group state at the current `epoch`, without
   having needed to be online for steps 1–3.

**Success result.** The group (or new 1:1) advances to a new `epoch` including the new member;
the new member, once it processes the `Welcome`, holds full current group state.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Initiator lacks `admin` capability for a pre-existing group | Reject | `INSUFFICIENT_ROLE` (§5.8.2) |
| Committer unreachable to accept the Commit | Defer | See §19.5.5 (failover) — members hold the pending proposal until the committer returns or a new one is elected |
| `keypackage` already consumed (race with another initiator, §19.4.1) | Reject | `KEYPACKAGE_CONSUMED`; initiator must `fetch-keypackage` again and resubmit |
| `Welcome` MOTE is deferred to the invitee's requests area because the inviter is not yet a known contact of the invitee (§2.7a) | Defer (at the invitee) | The `Welcome` sits in the requests area like any cold-sender MOTE until the invitee promotes the sender (pins them) or the recipient's policy otherwise clears it — **this is a real interaction point**: an `Add` from a stranger does not bypass the invitee's own anti-abuse gate merely by being a `group_event` kind. Implementations SHOULD document this explicitly to avoid a "why didn't my invite arrive" support issue |
| Two concurrent `Add`s for the same group produce conflicting Commits at the same log position | Reject (fork) | Fork-detection halt applies (§19.5's fork-detection operation) — members MUST halt and alert, not silently pick one |

**Idempotency / retry.** Not idempotent at the MLS layer (each successful `Add` advances
`epoch` once); a failed submission (e.g. committer timeout) is safe to retry with a fresh
proposal once the committer situation resolves, since an un-applied proposal has no group-state
effect yet.

**Example trace.**
```
A (group admin): add-member(group_state=G@epoch5, keypackage=KP_onetime_17, role="member")
A → committer: Proposal{Add, KP_onetime_17}
committer: appends Commit_6{Add(KP_onetime_17)} to hash-chained log at position 6
committer → all current members (incl. A): Commit_6
members: apply Commit_6 → G@epoch6, new member's leaf present (pending Welcome)
A → bob (invitee, via bob_ik resolved from KP_onetime_17's owner):
     MOTE{ kind:0x06, to: bob_ik, payload: Welcome{ epoch:6, secrets:..., tree:... } }
   ... bob offline; MOTE retried per §19.3.3 ...
bob (later, online): deliver() processes the Welcome MOTE → bootstraps G@epoch6 locally
bob → A: ack(id)
```

### 19.4.3 `external-commit(group_info) → Commit`

**Purpose.** Let a party **self-join** a group using its published `GroupInfo`, without waiting
for an existing member to `Add` them — MLS's other async-join primitive (§5.3 step 2, "or the
joiner uses an External Commit against the group's `GroupInfo` to self-join"). This is also the
mechanism behind `open`/`request` join policies (§5.8.2, §19.5.4).

**Initiator / Responder.** Initiator: the joining party. Responder: the group's committer (must
still order the resulting Commit into the hash-chained log, §5.1 — external commit does not
bypass ordering, only bypasses needing an existing member to sponsor it).

**Parameters.**
- `group_info` (`GroupInfo`, MUST) — the group's current public join information (ratchet-tree
  public state + `epoch`), obtained via the group's address (§5.8: resolving a group address
  works like resolving any identity, §19.1.1) or an out-of-band invite link.
- `join_policy_proof` (OPTIONAL) — for `request`/`vouch` join policies (§5.8.2), whatever
  satisfies the policy (an admin-approval token, or a vouch from an existing member); absent for
  `open` groups.

**Preconditions.**
1. The group's `join` policy (§5.8.2) is `open`, or the joiner holds a valid
   `join_policy_proof` for `request`/`vouch`.
2. `group_info` is current (its `epoch` is not so stale that the resulting Commit would be
   rejected as building on an obsolete tree — see failure modes).

**Procedure (normative).**
1. Construct an MLS `ExternalCommit` against `group_info`'s ratchet tree, adding the joiner as a
   new leaf, signed by the joiner's own key.
2. Submit the `ExternalCommit` to the committer for ordering into the hash-chained log, exactly
   as any other Commit (§5.1) — an external commit is still subject to the same total-order
   requirement; it is a *different kind* of Commit, not a different ordering channel.
3. On acceptance, all current members (including the joiner) apply it; `epoch` advances.
4. For `open` groups this is rate-limited and anti-abuse-gated (§5.8.2: "anyone with the address
   may join, rate-limited + anti-abuse §9") exactly like a cold-sender MOTE, since an
   `ExternalCommit` submission is, from the committer's perspective, an unsolicited request from
   a potentially-unknown party.

**Success result.** The joiner is a full group member at the new `epoch`, having bootstrapped
state directly from `group_info` without needing any existing member online to sponsor it.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Join policy is `closed` | Reject | `JOIN_NOT_PERMITTED`; the group does not accept external commits at all — invite-only via `add-member` (§19.4.2) |
| `request`/`vouch` policy and no valid proof supplied | Defer | Request queued for admin approval (`request`) or rejected pending a vouch (`vouch`); not immediately rejected outright if the group's policy defines an approval queue, but not admitted until approved |
| `group_info.epoch` is stale (group has since advanced past what the joiner's `ExternalCommit` was built against) | Reject | `EPOCH_STALE`; the committer rejects a Commit built on an obsolete tree — joiner must re-fetch current `group_info` and retry |
| `open` group, rate limit exceeded for this source (anti-abuse, §9.2) | Reject | `RATE_LIMITED`; same treatment as any cold-sender flood |
| Committer unreachable | Defer | Same failover handling as §19.5.5 |
| Two external commits race at the same `epoch` | Reject (one of them, fork-avoided) | The committer serializes: the first accepted advances `epoch`; the second is rejected as `EPOCH_STALE` and must rebuild against the new tree and retry — this is the ordinary case the committer exists to resolve, not a fork (a fork is two committers disagreeing about which one won, §19.5's fork-detection) |

**Idempotency / retry.** Not idempotent (each success advances `epoch` once); retries after
`EPOCH_STALE` MUST rebuild the `ExternalCommit` against the freshly-fetched `group_info`, not
resubmit the stale one.

**Example trace (open group).**
```
A: external-commit(group_info=GroupInfo{group="team@company.com", epoch:12, tree:...})
A: join_policy = "open" (no proof needed)
A → committer: ExternalCommit{ new_leaf: A_leaf, based_on_epoch:12 }
committer: rate-limit check for A's source — OK, under threshold
committer: appends Commit_13{ExternalCommit(A)} to log
committer → all members: Commit_13
members (incl. A): apply → G@epoch13, A now a full member with role="member" (default for open)
```

## 19.5 Group operations (§5.1, §5.8)

### 19.5.1 `create-group(params) → GroupState`

**Purpose.** Instantiate a new MLS group as an addressable identity (§5.8: "a group is an
identity that has members... its own keypair and therefore its own name"), fixing its initial
committer, roles, and posting/visibility/join policies.

**Initiator / Responder.** Initiator: the creating party. No responder beyond the mesh/KT
publication steps (a brand-new group has no other members yet, though it MAY be created with an
initial member set via `add-member`, §19.4.2, applied after creation).

**Parameters.**
- `group_keypair` (generated, MUST) — the group's own identity keypair (§5.8); the group is
  published as an `Identity` (§1.3) exactly like a person's, with `names` optionally carrying a
  `@handle` or `name@domain` (§3.9).
- `posting_model` (`"broadcast" | "collaborative"`, MUST) — §5.8.1 (the `"collaborative"` value is
  the "channel" model; these are the exact `posting-model` wire values, §18.6.1).
- `visibility` (`"hidden" | "visible"`, MUST) — membership-visibility policy (§5.8.1; `"visible"`
  is the "member-visible" model; exact `visibility` wire values, §18.6.1); MUST be `"hidden"` if
  `posting_model="broadcast"` and the deployment wants subscriber-list privacy (§5.8.3) — see the
  precondition below.
- `join_policy` (`"closed" | "request" | "open" | "vouch"`, MUST) — §5.8.2.
- `creator_role` (implicit `owner`, not a parameter) — the creator is always the initial
  `owner` (§5.8.2).
- `legacy_address` (`tstr`, OPTIONAL) — a `team@company.com` address served by a gateway for
  legacy interop (§5.8.5).

**Preconditions.**
1. If `visibility="hidden"` (subscriber-list privacy, §5.8.3), the implementation MUST use the
   relay/committer fan-out delivery model (§5.8.3), not the shared MLS tree — this is a
   structural choice made at creation time, not switchable without effectively recreating the
   group's delivery mechanism (switching `posting_model`/policy via a Commit, §5.8.1, does not
   retroactively change how *already-delivered* membership information was exposed).
2. `group_keypair` has not been previously published (a fresh identity).
3. **Group-key custody (§5.8.6).** The group's signing key MUST be **threshold-held** across the
   group's `owner`/`admin` set (FROST-style, reusing §1.4 machinery) so that no single admin — and
   no committer — can sign as the group alone. At creation the sole owner trivially satisfies this;
   as admins are added the key becomes threshold-held. The group publishes its own
   `RecoveryPolicy` (§1.4); changes to the group `Identity`, its key, or its recovery MUST satisfy
   the group's `rotate_threshold` (weakening-quorum + veto rules of §1.4 apply) and MUST appear in
   KT (§3.5). The committer orders *handshakes* only and is **not** authorized to change the
   group's identity key — that is a threshold act above the committer role.

**Procedure (normative).**
1. Generate `group_keypair` (threshold-held per precondition 3).
2. Initialize MLS group state with the creator as the sole member, role `owner`, and the
   requested `posting_model`/`visibility`/`join_policy` as signed fields of the group state
   (§5.8.2: "all membership/role/policy changes are signed and appear in the group's
   hash-chained handshake log").
3. Set the creator as the **initial committer** (§5.1: "the group creator is the initial
   committer. Committer identity is a signed field of the group state; every member knows it").
4. Publish the group's `Identity` (§19.1.2, reused verbatim — a group identity publishes exactly
   like a personal one, carrying its threshold-held key and `recovery` per precondition 3) so
   `resolve("team@company.com")` or `resolve("@team")` finds it. The `GroupState` members pin
   references this `Identity` by content hash in `group_identity` (key 14, §18.6.1).
5. If `legacy_address` is set, configure the gateway (§7, §5.8.5) to fan out inbound legacy mail
   to the address as MOTEs to current members.
6. Optionally, invite an initial member set via `add-member` (§19.4.2), applied as Commits
   against `epoch=0`/`1` immediately after creation.

**Success result.** A published group `Identity`, resolvable by name, with the creator as sole
member/owner/committer, ready to accept `join`s (§19.5.4) or `add-member` invitations.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Requested name (`@handle` or `name@domain`) already claimed by another identity | Reject | `NAME_UNAVAILABLE`; same as any identity naming conflict (§3.9.2's anti-squat mechanism applies identically to group handles) |
| `visibility="hidden"` requested with `posting_model="collaborative"` (channel) | Reject (policy, not protocol) | Implementations SHOULD warn/reject this combination since a member-visible ordered channel is in tension with hidden membership (§5.8.1's table pairs broadcast↔hidden and collaborative↔visible as the typical cases, though the fields are independently settable — an implementation MAY allow the unusual combination but MUST NOT silently drop the visibility guarantee if it does) |
| `legacy_address` requested but no gateway configured/available for that domain | Defer | Group is created without legacy interop; `legacy_address` can be added later via `policy-change` once a gateway is available |

**Idempotency / retry.** Not idempotent (each call mints a fresh `group_keypair`); a failed
publish (`publish-identity`'s own failure modes, §19.1.2) is safe to retry with the same
already-generated keypair, since the keypair generation step has no side effects until
publication.

**Example trace.**
```
A: create-group(posting_model="collaborative", visibility="visible", join_policy="request",
                names=["@design-team"])
A: generate group_keypair GK
A: init MLS group, sole member A, role=owner, committer=A
A: publish-identity(Identity{ iks:{1:GK_pub}, names:["@design-team"], version:1, ... })
A → KT: append; A → DHT: put
A: create-group() → GroupState{ group_identity: blake3(Identity{GK}), epoch:0, members:[A(owner)],
                                 posting_model:"collaborative", join_policy:"request" }
```

### 19.5.2 `group-add` / `group-remove` / `group-role-change` / `group-policy-change`

**Purpose.** The four Commit-based group-management operations (§5.8.2), ordered by the
committer like any other handshake message (§5.1), each requiring the role §5.8.2 specifies.

**Initiator / Responder.** Initiator: a member holding the required role. Responder: the group's
committer (ordering) and all current members (applying).

**Parameters (per sub-operation).**
- `group-add(keypackage, role)` — `keypackage` (MUST, from `fetch-keypackage`, §19.4.1), `role`
  (MUST, one of `owner|admin|member|poster|reader`, §5.8.2). Mechanically this **is**
  `add-member` (§19.4.2) invoked against an existing group — restated here as its own named
  entry point because §5.8.2 names it separately and gives it a distinct role requirement.
- `group-remove(member_ik)` — `member_ik` (`bytes`, MUST) — the member to remove.
- `group-role-change(member_ik, new_role)` — both MUST.
- `group-policy-change(field, new_value)` — `field` (MUST, one of `posting_model`,
  `visibility`, `join_policy`), `new_value` (MUST, matching that field's type in §19.5.1).

**Preconditions (role gating, normative per §5.8.2).**

| Operation | Required role |
|---|---|
| `group-add` | `admin` (or `owner`) |
| `group-remove` | `admin` (or `owner`) |
| `group-role-change` / transfer ownership | `admin`/`owner` (transferring ownership itself typically requires `owner`) |
| `group-policy-change` | `admin` (or `owner`) |

**Procedure (normative).**
1. Initiator constructs the appropriate MLS Proposal: `Add` (via `keypackage`), `Remove` (via
   `member_ik`'s leaf), `Update`-with-role-attribute change, or a policy-field `Update` —
   whichever the group's MLS extension mechanism uses to carry non-cryptographic group-state
   fields (roles/policy are signed fields of the group state riding alongside the ratchet tree,
   §5.8.2).
2. Submit to the committer for ordering (§5.1); the committer appends it to the hash-chained log
   at the next position.
3. On acceptance, all members apply it; `epoch` advances.
4. **`group-remove` additionally triggers file-key rotation** for shared folders (§5.8.2: "Remove
   triggers file-key rotation for shared folders (§6.7)") — the removing operation MUST, for any
   folder marked confidential (§6.7), re-key and redistribute new file keys to the remaining
   members, since MLS removal alone blocks only *future* messages, not previously-held file
   keys.
5. Every accepted change appears in the group's hash-chained handshake log (§5.8.2), auditable
   by any member ("who added/removed whom").

**Success result.** The group's roster/role/policy state advances to a new `epoch` reflecting
the change, logged and auditable.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Initiator lacks the required role | Reject | `INSUFFICIENT_ROLE`; the committer (or, if using signature-gated proposals, any verifying member) refuses to apply an Update/Remove/Add not signed by a sufficiently-privileged member |
| `group-remove` targets a member not currently in the roster | Reject | `NOT_A_MEMBER` |
| `group-role-change` attempts to remove the **last** `owner` without designating a successor | Reject | `NO_OWNER_REMAINING`; a group MUST retain at least one `owner` at all times (analogous to the identity layer's requirement that recovery policy changes can't lock the owner out, §1.4) — implementations MUST enforce this as a group-state invariant |
| `group-policy-change` sets `visibility="hidden"` on a group whose `posting_model="collaborative"` (channel) and members have already seen each other in the shared tree | Reject or warn | The exposure already happened for existing members (§5.8.3's per-member sealed re-fan-out only protects members added *after* the switch); implementations MUST surface that a policy switch is not retroactive |
| Committer unreachable | Defer | §19.5.5 failover applies |
| File-key rotation (step 4) fails to reach a remaining member (offline) | Defer | Queued like any other MOTE (sender retry, §19.3.3); the removed member's access is already cryptographically stale for new content regardless of rotation-delivery timing — the risk window is bounded by how quickly rotation completes, not eliminated by this operation alone |

**Idempotency / retry.** Not idempotent (each success advances `epoch`); safe to retry against
the current `epoch` after a committer-unreachable defer resolves.

**Example trace (`group-remove` with file-key rotation).**
```
A (admin): group-remove(member_ik=carol_ik)
A → committer: Proposal{Remove, carol_ik}
committer: appends Commit_9{Remove(carol_ik)} to log
committer → members: Commit_9
members: apply → G@epoch9, carol no longer in roster
A: for each confidential shared folder F in this group:
     generate new file key K'_F
     re-encrypt F's manifest-referenced content under K'_F  (§6.7)
     distribute K'_F to remaining members via a `group_event` MOTE
carol: retains any already-downloaded plaintext (un-share limit, §6.7) but cannot decrypt
       future content or future manifest updates under K'_F
```

### 19.5.3 `post-to-group(group_ik, mote)`

**Purpose.** Send a message to a group's address, fanned out per the group's `posting_model`
(§5.8.1) and delivery scale regime (§5.8.4).

**Initiator / Responder.** Initiator: any member with `poster` capability (or `member`, if the
group does not distinguish poster/reader, §5.8.2). Responder: every current member (small
groups, standard MLS fan-out) or the committer acting as fan-out relay (large lists, §5.8.4).

**Parameters.**
- `group_ik` (`bytes`, MUST) — the group's identity key (`to` in the resulting `Envelope`s,
  §2.2a).
- `mote` (application `Payload`, MUST) — the content, exactly as any 1:1 MOTE.

**Preconditions.**
1. The initiator holds `poster` (or equivalent, per the group's role scheme) capability.
2. The group's current `epoch` state is held by the initiator (it must encrypt to the current
   epoch's tree/keys).
3. **Per-poster anti-abuse proof (§9.9).** Because fan-out is an amplification vector, the
   **poster's** own anti-abuse proof (§9) MUST be carried on each fanned-out per-member delivery
   and evaluated by each recipient against the **original poster**, not the group identity (no
   accountability laundering). *Which* proof depends on the membership model: a **member-visible**
   channel (§5.8.3) uses a per-member ARC token (one per recipient origin); a **hidden-membership**
   list uses **postage or PoW scoped to the list address** (the committer/relay verifies it at
   ingress and vouches it per-delivery, §9.9). Posting to a **large** list MUST carry postage/PoW
   commensurate with the fan-out size.

**Procedure (normative, branches on scale/model per §5.8.4).**
1. **Small groups (standard MLS fan-out).** Encrypt one MLS application message under the
   current `epoch`; the mesh/mixnet delivers it once to the group's routing structure; each
   member's node decrypts locally with its own leaf key. `to = group_ik` or the MLS group's
   `DeliveryTag` (§2.2a).
2. **Large lists (per-member sealed fan-out).** The committer's ordered log is authoritative for
   membership; delivery is **per-member**, sealed individually to each subscriber (§5.8.4) —
   mechanically, this is `deliver` (§19.3.1) invoked once per member with a per-member-sealed
   envelope, not one shared-tree MOTE. This is what gives hidden-membership lists (§5.8.3) their
   privacy: no shared tree exposes the roster.
3. Either way, each recipient's own `deliver` procedure (§19.3.1) applies unchanged — group
   membership itself is the recipient's "known contact" classification (step 5 of §2.7), so
   fellow members are never treated as cold senders by each other.

**Success result.** Every current member (or subscriber, for hidden lists) receives the message,
each acking independently per §19.3.2.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Initiator lacks `poster` capability (`reader`-only role attempting to post) | Reject | `INSUFFICIENT_ROLE`; the message is not accepted into the group's delivery path |
| Poster exceeds the per-poster fan-out rate limit, or an `open`-join list post exceeds the amplification cap, or a large-list post lacks commensurate postage/PoW (§9.9) | Reject / throttle | Fan-out is rate-limited **per poster** and capped for `open` lists; the poster's proof is evaluated per-recipient against the original poster, so the amplification is bounded and attributable (§9.9, §5.8.4) |
| Initiator's local `epoch` is behind the group's current `epoch` (missed a recent Commit) | Reject (local) | The initiator must first process pending Commits (sync to current epoch) before it can correctly encrypt; this is a client-side precondition failure, not a network error |
| One member (of many, large-list fan-out) is offline | Defer (per-member) | That member's individual sealed delivery enters the ordinary sender-retry state machine (§19.3.3) independently of the others — one offline subscriber never blocks delivery to the rest |
| A member has been removed but the sender's cached roster is stale | Reject (at the removed party, silently, per §19.3.1) | The stale send still goes out, but the removed member cannot decrypt it if key rotation (§19.5.2 step 4) has already run for confidential content; for non-confidential/ordinary application messages under a still-current epoch this is not applicable since a removed member is no longer a tree leaf at all post-Remove-Commit |

**Idempotency / retry.** Each post is a distinct MOTE (`id` = content address of that specific
ciphertext); re-sending identical content produces a distinguishable *new* MOTE per recipient
(sealed per-member framing differs even for identical plaintext, in the large-list case) — not
naturally deduplicated across a wholesale re-post; deduplication (§2.6) applies per-envelope
`id` as usual if the exact same send is retried by the sender-retry machine.

**Example trace (hidden-membership broadcast list).**
```
A (poster on @design-team): post-to-group(group_ik=GK_pub, mote={ kind:0x01, body:"standup at 10" })
A: local epoch check — current                                    # OK
A: posting_model = "broadcast", visibility = "hidden"
A → committer(=A's own node, or the current committer): fan-out request
committer: for each of 40 subscribers, seal an individual Envelope{to: blinded_tag_i, ...}
committer → subscriber_1..40: 40 individual deliver()s, each subject to that subscriber's own
                              known-contact/cold-sender classification (§2.7) against the list's
                              posting identity, not against each other
subscriber_17 (offline): enters that subscriber's own sender-retry loop for this one Envelope
subscriber_1..16,18..40: ack independently
```

### 19.5.4 `join(group_ik, mode)`

**Purpose.** Request or perform membership entry into a group under its declared `join_policy`
(§5.8.2): `closed`, `request`, `open`, or `vouch`.

**Initiator / Responder.** Initiator: the prospective member. Responder: the group's committer
(ordering), and — for `request`/`vouch` — an admin (approval) or an existing member (vouching).

**Parameters.**
- `group_ik` (`bytes`, MUST) — resolved group identity (§19.1.1 works identically for groups).
- `mode` (implicit from the group's own state, not caller-chosen) — the operation behaves per
  whichever `join_policy` the group currently has; the caller does not select a mode, it
  discovers one from `group_info`/`Identity`.
- `vouch_token` (OPTIONAL) — required if `join_policy="vouch"`: a signed introduction from an
  existing member (§5.8.2, and the general vouch mechanism of §9.7).

**Preconditions.** `group_ik` resolves and its current `join_policy` is known (via its published
`GroupInfo`/`Identity`).

**Procedure (normative, branches by policy).**
1. `join_policy="closed"` → **not performable** by this operation; the prospective member must
   be invited via `group-add` (§19.5.2) by an existing `admin`. `join` immediately fails.
2. `join_policy="open"` → equivalent to `external-commit` (§19.4.3) with no proof required;
   subject to that operation's own rate-limiting/anti-abuse gate.
3. `join_policy="request"` → submit a join request (itself deliverable as a small signed message
   to the group's admin set, gated by the *admins'* own cold-sender policy if the requester is
   unknown to them, §2.7); on admin approval, the admin performs `group-add` (§19.5.2) on the
   requester's behalf, OR issues an approval token the requester then presents to
   `external-commit` (§19.4.3) as `join_policy_proof` — either implementation is conformant, but
   the requester MUST NOT be able to self-admit without *some* admin-signed approval artifact.
4. `join_policy="vouch"` → present `vouch_token` (signed by an existing member) to
   `external-commit` (§19.4.3) as `join_policy_proof`; the committer/group verifies the voucher
   is a current member and that the vouch itself is rate-limited (§9.7: vouches "MUST itself be
   rate-limited to prevent vouch farming").

**Success result.** The requester becomes a group member at a new `epoch` (immediately for
`open`/`vouch` once approved; after admin action for `request`).

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| `join_policy="closed"` | Reject | `JOIN_NOT_PERMITTED`; must be invited |
| `join_policy="request"`, admin never responds | Defer | Request remains pending until the **join-request expiry = 30 days (§16.8)**, then auto-expired/cleaned up (mirrors requests-area retention, §16.5) |
| `join_policy="vouch"`, voucher is not a current member (removed since issuing the vouch, or never was one) | Reject | `INVALID_VOUCH` |
| `join_policy="vouch"`, voucher exceeds their own vouch rate limit | Reject | `VOUCH_RATE_LIMITED` (§9.7 anti-farming) |
| `join_policy="open"`, requester exceeds the group's open-join rate limit | Reject | `RATE_LIMITED` (same as `external-commit`'s failure mode, §19.4.3) |

**Idempotency / retry.** A repeated `join` request while a `request`-mode approval is pending is
harmless but SHOULD be deduplicated client-side (no protocol-level harm in an admin seeing two
identical pending requests, but implementations SHOULD collapse them for UX).

**Example trace (`vouch`).**
```
A: join(group_ik=GK_pub, vouch_token=Vouch{ voucher: carol_ik, group: GK_pub,
                                             sig: Ed25519(carol_device_key, ...) })
A → committer: ExternalCommit{ new_leaf: A_leaf, join_policy_proof: vouch_token }
committer: verify carol_ik is a current member                    # OK
committer: check carol_ik's vouch-rate counter                    # under limit
committer: appends Commit{ExternalCommit(A)} to log
committer → members: Commit
members (incl. A): apply → A is now a member
```

### 19.5.5 `committer-elect` / `committer-rotate` (failover)

**Purpose.** Establish or change which member serializes the group's handshake log (§5.1), and
recover liveness when the current committer is unreachable.

**Initiator / Responder.** Initiator: the group creator (initial election, folded into
`create-group`, §19.5.1) or any member proposing rotation. Responder: all members, who must
apply the same Commit to agree on the new committer.

**Parameters.**
- `new_committer_ik` (`bytes`, MUST) — the member being promoted.
- `reason` (`tstr`, OPTIONAL) — `"scheduled"`, `"timeout"`, `"vote"` — informative only.

**Preconditions.**
1. `new_committer_ik` is a current group member.
2. For `reason="timeout"`: the current committer has been unreachable — or has withheld ordering
   of a specific pending, member-signed proposal (selective censorship, §5.1) — past the
   **committer-liveness timeout (5 min, §16.8)**, and this MUST have been observed as **2
   consecutive misses** (takeover hysteresis, §16.8) so a transient NAT/relay blip does not
   trigger churn.

**Procedure (normative, §5.1).**
1. **Committer identity is a signed field of the group state**; rotation is itself a Commit
   (`committer-rotate`), submitted like any other group-management Commit (§19.5.2's mechanism)
   — but critically, *who orders this particular Commit* is exactly the problem being solved, so
   see step 2.
2. **Normal rotation** (current committer reachable, e.g. `reason="scheduled"` or `"vote"`): the
   current committer itself orders and appends the rotation Commit (it is, after all, still
   live) naming `new_committer_ik`; all members apply it, and `new_committer_ik` becomes
   authoritative for future ordering.
3. **Failover rotation** (`reason="timeout"`, current committer unreachable): members hold
   pending Proposals and either (a) wait for the unreachable committer to return, or (b) elect a
   new committer via a takeover Commit that **references the last agreed log head**. The
   successor is **deterministic** — among live, non-faulted members the one with the **lowest
   member signing key** in canonical byte order (earliest join epoch breaks a tie), per §5.1 —
   so members do not negotiate *who* takes over. Since no ordering authority is currently live,
   the takeover Commit takes effect **only when it carries a strict-majority roster quorum of
   `> n/2` member signatures (⌈(n+1)/2⌉ of current members, §16.8)**, so two partitions cannot
   each install a rival successor (split-brain prevention). A 2-member group whose one peer is
   dead cannot meet `> n/2` and is resolved by leaving/recreating the group, not by takeover
   (§5.1 edge case).
4. Once `new_committer_ik` is agreed, it resumes ordering exactly as any committer would.

**Success result.** The group has a single, agreed, live committer, with the transition itself
recorded in the hash-chained log (auditable, §5.8.2).

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| `new_committer_ik` is not a current member | Reject | `NOT_A_MEMBER` |
| Members disagree on the referenced log head during failover election (partition) | Reject → escalates to fork-detection | Two members proposing rotation against *different* "last agreed" heads, each gathering a disjoint quorum, produces exactly the fork condition of §19.5.6 — this operation does not itself resolve that; it hands off to fork-detection |
| Old committer returns after a failover rotation already completed | Defer, reconciled | The returning (former) committer MUST accept the newer, member-quorum-agreed Commit as authoritative for its log position (it is, after all, hash-chained and signed) and rejoin as an ordinary member — it MUST NOT attempt to re-assert its old ordering role by appending a competing Commit at the same position (doing so is exactly the forgery-shaped fork case, distinguished from mere staleness by whether it happens *before* or *after* it has seen the quorum's Commit) |
| No live majority reachable at all (severe partition) | Defer | Group ordering stalls (no Commits accepted) until enough members are simultaneously reachable to agree a log head; application-message delivery over the mixnet (§5.1: "ordinary application messages... MAY travel over the reordering mixnet") is unaffected — only handshake/membership changes stall |

**Idempotency / retry.** Rotation is not idempotent (each success is one Commit); retried
election attempts after a resolved partition should reference the now-current log head, not a
stale one.

**Example trace (failover).**
```
[committer carol_ik: 5-min liveness timeout (§16.8) exceeded on 2 consecutive misses]
A: A holds the lowest member signing key among live members → A is the deterministic successor (§5.1)
A: committer-rotate(new_committer_ik=A_ik, reason="timeout")
A → other members (bob, dave): propose takeover, referencing last agreed log head = Commit_9
bob, dave: agree Commit_9 is indeed the last one they've applied too
A, bob, dave: 3/4 signatures ≥ ⌈(4+1)/2⌉ = 3 → > n/2 roster quorum met (§16.8) → accept
              Commit_10{committer-rotate(A_ik)} at position 10 (referencing Commit_9)
A: becomes committer; resumes ordering
[carol later returns, offline the whole time]
carol: receives Commit_10 on reconnect; sees it correctly chains from Commit_9 (her own last
       known state); accepts it; rejoins as an ordinary member, does NOT contest
```

### 19.5.6 Fork-detection halt

**Purpose.** Detect and respond to committer misbehavior (or a failed failover producing
disjoint quorums): two Commits at the same log position with the same predecessor (§5.1: "The
hash-chained log makes a fork detectable... members MUST halt and alert").

**Initiator / Responder.** Detected by any member on receipt of conflicting log entries; there is
no single "initiator" — this is a passive-detection, active-response procedure every member
runs continuously.

**Parameters.** N/A (triggered by observed state, not called with arguments) — but conceptually
takes `commit_a`, `commit_b` (both referencing the same `prev` at the same position) as its
detected input.

**Preconditions.** A member has received two distinct, validly-signed Commits both claiming the
same log position with the same `prev` hash (a structural impossibility in honest operation of a
single-writer hash-chained log — proof of committer misbehavior, or of an unresolved failover
race, §19.5.5).

**Procedure (normative, §5.1, §6.6 item 7 cross-reference).**
1. On detecting `commit_a` and `commit_b` both claiming position N with identical `prev`: **MUST
   halt** — stop applying further Commits from either branch, and stop treating either as
   authoritative.
2. **MUST alert** the member (surface a security warning, analogous to KT equivocation
   detection, §3.5) — this is member-facing, not silent, since a stalled/forking committer is a
   detectable-but-not-forgeable event (§5.1: "a committer can stall but not forge").
3. Group membership/application-message processing for this group SHOULD pause for
   handshake-dependent operations (adds/removes/role/policy changes) until resolved; ordinary
   application messages already encrypted under the last-agreed `epoch` MAY continue to be
   processed (the fork concerns *future* handshake state, not already-established message keys).
4. Resolution is **out of the committer's hands** — members must manually (or via a
   higher-level, out-of-band group-recovery convention, not fixed in v0) agree which branch to
   keep, or re-create the group from the last pre-fork agreed state. This is a deliberate
   consequence of "cannot forge" (every Commit is member-signed) not implying "cannot stall or
   fork the log" — those failure modes remain, bounded by detectability + committer rotation, not
   eliminated.

**Success result.** There is no "success" result for a fork — this operation's success is
**correctly refusing to silently pick a branch**. The observable outcome is a halted group
requiring member/administrator intervention.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| A fork is detected but a member's implementation does not halt (bug) | Reject (implementation MUST prevent) | Would silently diverge group state across members — an explicit MUST-NOT; the spec provides the detection signal but relies on conformant implementations to act on it |
| Fork is actually a **stale duplicate** delivery of the *same* Commit (not a true fork) | N/A | Distinguished by comparing full Commit content, not just position+prev: identical Commits at the same position are a duplicate (dedup, harmless); only *differing* Commits at the same position+prev constitute a fork |
| No resolution convention is agreed by the group after halting | Defer, until a quorum-backed recovery Commit | The group remains halted for handshake purposes until members run the §5.1 "Fork recovery (out of HALT)" procedure — roll back to the **last common epoch** and re-apply from an `admin`/`owner` recovery Commit carrying a **`> n/2` member-signature quorum**. This is the v0 out-of-band stopgap (Decentralized MLS is the eventual leaderless fix); already-decrypted messages under the last-agreed epoch keep rendering meanwhile |

**Idempotency / retry.** N/A — this is a detection-and-halt procedure, not an invokable operation
with retry semantics; it re-triggers identically every time a member re-observes the same
conflicting pair.

**Example trace.**
```
bob (group member): receives Commit_10a{committer-rotate(A_ik)} referencing prev=Commit_9
                     ALSO receives Commit_10b{committer-rotate(eve_ik)} referencing prev=Commit_9
bob: both signatures verify individually — but SAME position (10), SAME prev (9), DIFFERENT content
bob: FORK DETECTED
bob: HALT further Commit application for this group
bob: ALERT — surfaces "group state conflict, admin action required" to the user
bob: continues rendering already-decrypted application messages under epoch 9 (last agreed)
     but does not apply either Commit_10a or Commit_10b
```

## 19.6 DMTAP-Auth operations (§13)

> **Conformance note (normative).** These operation sketches are **subordinate to §13's
> hardened requirements**; where an op block below is less specific, §13 governs. In particular:
> (a) the client MUST generate the session keypair *before* signing and include
> `cnf = H(session_pubkey)` in the signed challenge, and the RP MUST bind the session **only** to
> `cnf` (§13.3, session-hijack defense); (b) a **bare node-signed mode is FORBIDDEN** for remote
> nodes — signing requires a passkey *or* an authenticated paired companion client enforcing
> intent-matching, and the node signs only a challenge matching a **node-minted, user-initiated
> pending intent** (§13.3.1); (c) the OIDC bridge (`oidc-bridge-issue`) MUST embed the user's own
> signed assertion + `cnf` and log to a bridge-transparency log, and yields a **bearer** token
> (no proof-of-possession) — trusted like any classical IdP (§13.6); (d) RP sessions MUST be
> re-validated against status/KT at a bounded interval, failing closed after a grace window
> (§13.4, §16.8).

### 19.6.1 `auth-challenge(rp_origin) → Challenge`

**Purpose.** The relying party's (RP) first step in the login ceremony (§13.3 steps 1–3):
construct an origin-bound, nonce-bound challenge for the user to sign.

**Initiator / Responder.** Initiator: the RP. Responder: none yet — this operation produces the
`Challenge` object that `auth-assert` (§19.6.2) later consumes; it is not itself sent to the
identity's node at this stage (it is presented to the **trusted client**, §13.3 step 4).

**Parameters.**
- `rp_origin` (`tstr`, MUST) — the RP's **true, browser/OS-observed** origin — this MUST be
  supplied by the trusted client environment (e.g. the browser's own origin, not a value the RP
  hands over as data), per §13.3.1's load-bearing rule.
- `nonce` (`bytes`, MUST) — single-use, generated fresh per challenge.
- `issued_at` / `exp` (`u64`/`u64`, MUST) — validity window; default per §16.1 (120 s).
- `aud` (`tstr`, MUST) — the intended relying-party identifier (binds the assertion to this RP
  specifically, distinct from `rp_origin` when an RP legitimately spans multiple origins).
- `scope` (`[* tstr]`, OPTIONAL) — requested capability scope beyond bare login (§13.5).

**Preconditions.** None beyond RP-side nonce freshness (the RP MUST NOT reuse a `nonce`).

**Procedure (normative, §13.3 steps 1–3).**
1. RP shows "Sign in with DMTAP"; the user supplies `alice@yourdomain`.
2. RP resolves `alice@yourdomain` → key + auth endpoint (this is `resolve`, §19.1.1, plus
   discovery of an auth-capable endpoint via DID/OIDC-discovery mechanisms, §13.6).
3. RP constructs `Challenge{rp_origin, nonce, issued_at, exp, aud, scope}`.
4. RP hands `Challenge` to the trusted client (browser/OS/app) — **not** directly to the
   identity's node — so that the client, not the RP, is the party asserting `rp_origin`'s truth
   in the ceremony that follows (§13.3.1).

**Success result.** A `Challenge` object ready for the trusted client to process into a
WebAuthn/passkey ceremony (`auth-assert`, §19.6.2).

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| RP cannot resolve `alice@yourdomain` | Reject | Same as `resolve`'s own failure modes (§19.1.1) — surfaced to the user as "unknown DMTAP address" |
| RP's own origin cannot be established by the trusted client (unusual embedding context, e.g. a non-browser environment with no origin concept) | Reject | The ceremony MUST NOT proceed without a client-enforced origin — §13.7 honest limit 1: without a trusted client, origin binding degrades to user-verified (weaker) and MUST NOT be the only mode |
| `nonce` reused by the RP (implementation bug) | Reject (at the verifying node/RP itself on later replay-check) | A reused nonce risks replay; `auth-assert`'s verification (§19.6.2) MUST reject a stale/reused nonce regardless of how this step behaved |

**Idempotency / retry.** Each call MUST mint a fresh `nonce`; not idempotent by design (replay
prevention depends on single-use nonces, §16.1).

**Example trace.**
```
RP: auth-challenge(rp_origin="https://app.example.com", aud="app.example.com")
RP: resolve("alice@yourdomain") → { ik: MFk..., auth_endpoint: did:web:yourdomain:users:alice }
RP: nonce = random(16 bytes), issued_at=now, exp=now+120s
RP → trusted_client: Challenge{ rp_origin:"https://app.example.com", nonce:9f2a...,
                                 issued_at:..., exp:..., aud:"app.example.com" }
```

### 19.6.2 `auth-assert(challenge) → SignedAssertion`

**Purpose.** The user's key signs the origin-bound challenge, producing the assertion the RP
verifies (§13.3 steps 4–6), with the origin-binding guarantee enforced by a trusted client
component per §13.3.1.

**Initiator / Responder.** Initiator: the trusted client (browser/OS/app), acting on the user's
key (either directly, or gating the identity node's signature via a passkey, §13.3.1's preferred
design). Responder: the RP, which verifies the result.

**Parameters.**
- `challenge` (`Challenge`, MUST) — from `auth-challenge` (§19.6.1), as received by the trusted
  client (the client itself supplies/overrides `rp_origin` with its own observed value if the
  two disagree — see procedure step 1).
- `user_verification` (implicit, MUST occur) — a WebAuthn/passkey user-verification ceremony
  (§13.3.1) or, for a node-signed login, the node-side consent flow of §13.3.1 item 2.

**Preconditions.**
1. A trusted client component capable of writing the *observed* origin into the signed structure
   exists (WebAuthn `clientDataJSON` in a browser, §13.3.1) — the ceremony MUST NOT run through
   a raw-signature path where the RP's claimed origin is trusted uncritically.
2. If the identity's key lives on a **remote** always-on node (not the immediate device), the
   §13.3.1 remote-node hazard mitigations MUST all hold: the challenge carries `rp_origin`/`aud`
   and the node signs over them; a trusted approval surface displays the verified `rp_origin` and
   requires explicit per-login approval; the node rejects a challenge it cannot attribute to an
   authenticated request channel.

**Procedure (normative, §13.3 steps 4–5, hardened per §13.3.1).**
1. The trusted client writes its own **observed** `rp_origin` into the structure to be signed
   (WebAuthn: into `clientDataJSON`), regardless of what the RP's `Challenge.rp_origin` claimed —
   if the two differ, the client's observed value is authoritative and a mismatch aborts the
   ceremony rather than silently reconciling.
2. Run WebAuthn/passkey user-verification. Preferred design (§13.3.1): the passkey ceremony
   (via the **PRF extension** over CTAP2 `hmac-secret`) derives the key that unlocks the node's
   signing key — the node signs only *after* this local, origin-bound user-verification
   succeeds; the identity key itself never leaves the node and never touches the RP.
3. **Before signing**, generate a fresh **per-RP, per-device session keypair** (§13.4) and compute
   `cnf = H(session_pubkey)` (§13.3 step 4).
4. Sign `H(rp_origin ‖ nonce ‖ issued_at ‖ exp ‖ aud ‖ cnf)` (canonical hash, §2) under the
   user's **`IK`-authorized device key** (the identity-revealing login signer, `Assertion.from`) —
   **not** the session key (which `cnf` commits) and not a bare relayed challenge.
5. Return the `SignedAssertion{challenge, cnf, sig}` to the RP, which binds the session **only** to
   `cnf` (proof-of-possession, §13.4).

**Success result.** RP receives a `SignedAssertion` it can verify against the pinned key
(`resolve`'s output, §19.1.1) for `alice@yourdomain`, and binds the session to `cnf`.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Client-observed origin ≠ any origin the ceremony would sign for (a phishing relay presenting a look-alike origin) | Reject | The trusted client (WebAuthn/CTAP2) itself refuses — an assertion produced at `alice-yourdomain.evil.com` structurally cannot validate for `yourdomain` (§13.3.1); this is enforced by the client/authenticator, not by this operation's logic alone |
| `challenge.nonce` already used, or `exp` elapsed | Reject | `CHALLENGE_INVALID`/`CHALLENGE_EXPIRED`; RP MUST NOT authenticate on a stale/reused challenge |
| Remote-node ceremony: node cannot attribute the relayed challenge to an authenticated request channel (§13.3.1 hazard) | Reject | The node MUST refuse to sign; this is the specific defense against a phisher relaying a real challenge to a remote node for blind signing |
| User declines the user-verification/approval step | Reject | `USER_DECLINED`; no assertion produced |
| Node in an "approve any challenge" mode | Reject (of the mode itself) | §13.3.1: "Nodes MUST NOT offer 'approve any challenge' modes" — an implementation offering this is non-conformant; this operation's procedure assumes per-login explicit approval is the only conformant mode |

**Idempotency / retry.** Each assertion is bound to a single-use `nonce`; not idempotent or
replayable by design. A failed ceremony (e.g. user declines) is safe to retry by the RP issuing
a fresh `auth-challenge`.

**Example trace.**
```
trusted_client ← RP: Challenge{ rp_origin:"https://app.example.com", nonce:9f2a..., ... }
trusted_client: observed origin = "https://app.example.com"                # MATCHES challenge
trusted_client: WebAuthn ceremony — user-verification (biometric/PIN) via passkey
trusted_client: PRF-derived key unlocks node signing key
trusted_client: gen session keypair; cnf = H(session_pubkey)                # before signing
node: sign H(rp_origin ‖ nonce ‖ issued_at ‖ exp ‖ aud ‖ cnf) under device key "phone-passkey"
trusted_client → RP: SignedAssertion{ challenge, cnf, sig: Ed25519(...) }   # RP binds session to cnf
```

### 19.6.3 `session-establish(assertion) → Session`

**Purpose.** After successful authentication, establish a **key-bound** (not bearer) session per
§13.4, using DPoP or GNAP so a leaked token alone is useless to a thief.

**Initiator / Responder.** Initiator: the RP (having verified the assertion). Responder: the
client, which holds the newly-authorized session key.

**Parameters.**
- `assertion` (`SignedAssertion`, MUST) — from `auth-assert` (§19.6.2), already verified by the
  RP (signature validates against the pinned key, `rp_origin` matches the RP's own origin,
  `nonce` unused, not expired — §13.3 step 6).
- `session_key` (generated, MUST) — a fresh, **per-RP, per-device** ephemeral key, authorized by
  a device key (§1.2), not `IK` itself.
- `mechanism` (`"dpop" | "gnap"`, MUST) — §13.4.

**Preconditions.** `assertion` has passed RP-side verification (§19.6.2's success result).

**Procedure (normative, §13.4).**
1. Generate `session_key`, scoped to this RP and this device only.
2. Authorize `session_key` under a current device key (a signed statement: "this device key
   authorizes this session key for this RP").
3. Establish the session using the chosen `mechanism`:
   - **DPoP (RFC 9449):** every subsequent request carries a fresh proof-of-possession JWT signed
     by `session_key`.
   - **GNAP (RFC 9635) continuation:** the session is a GNAP grant continued key-based end to
     end.
4. RP records the session as bound to `session_key`'s public half, not to a bare bearer token.

**Success result.** A live, key-bound session; every subsequent API call must be accompanied by a
proof of possession of `session_key`.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| `assertion` fails RP-side verification (shouldn't reach this operation, but defensively) | Reject | `session-establish` MUST NOT be invoked on an unverified assertion; if it is, treat as `ASSERTION_INVALID` |
| `session_key` generation/authorization fails (device key unavailable) | Reject | `DEVICE_UNAVAILABLE`; retry once a device key is available |
| A stolen bearer-style token is replayed without the matching `session_key` proof | Reject | DPoP/GNAP proof-of-possession check fails at the RP for every subsequent request — this is the entire point of §13.4, restated as this operation's ongoing per-request postcondition, not a one-time check |

**Idempotency / retry.** Each successful login SHOULD mint a fresh `session_key` (not reuse one
from a prior session) — not idempotent; safe to retry the whole ceremony (`auth-challenge` →
`auth-assert` → `session-establish`) from scratch on failure.

**Example trace.**
```
RP: assertion verified (sig OK, rp_origin=="https://app.example.com", nonce unused, not expired)
RP: session-establish(assertion, mechanism="dpop")
client: generate session_key SK (per-RP, per-device)
client: device_key "phone-passkey" signs { authorizes: SK.pub, for_rp: "app.example.com" }
RP: records session bound to SK.pub
client → RP (subsequent request): GET /api/me  DPoP: <JWT signed by SK, htu/htm/jti/iat>
RP: verifies DPoP proof against SK.pub                            # OK — request authorized
```

### 19.6.4 `session-revoke(session_ref)`

**Purpose.** Revoke one app/device session, or all sessions under a device key, without
rotating the whole identity (§13.4).

**Initiator / Responder.** Initiator: the identity owner (from any device, or the RP itself on
suspicious activity). Responder: the transparency log (revocation record) and/or a short-lived
status endpoint the RP checks.

**Parameters.**
- `session_ref` (`bytes`, MUST) — either a specific `session_key`'s public half, or a
  `device_key` reference (revoking all sessions issued under that device at once).
- `reason` (`tstr`, OPTIONAL).

**Preconditions.** The revoking principal is the identity owner (any authorized device) or,
transitively, the outcome of a full recovery event (§1.4), which per §13.4 MUST invalidate **all**
prior session authorizations regardless of explicit per-session revocation.

**Procedure (normative, §13.4).**
1. Publish a revocation record for `session_ref` to the transparency log and/or a short-lived
   status endpoint.
2. If `session_ref` is a `device_key`, this revokes every session key that device key ever
   authorized, in one action (§13.4: "rotating a device key revokes all its sessions at once").
3. RPs consulting the status endpoint/log (directly, or via a cached revocation list with a
   bounded freshness window) MUST reject DPoP/GNAP proofs from a revoked `session_key` going
   forward.
4. **Full recovery event** (§1.4 reactive recovery, distinct trigger): on completing recovery
   under a new or reauthorized `IK`, **all** prior session authorizations are invalidated as a
   blanket consequence — not a per-session enumeration, since the owner may not even know every
   session that existed.

**Success result.** The targeted session(s) can no longer authenticate; unaffected
sessions/devices continue working.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| RP has not yet observed the revocation (stale cache/status check) | Defer | A revoked session may work only until the RP's next status/KT re-check, bounded by the **RP delegation re-validation interval ≤ 15 min (§16.8)** — and never past the session TTL 24 h / idle 30 min in any case |
| Revocation published but transparency log unreachable | Defer | Retry; until durable, revocation is not guaranteed visible to RPs relying on the log path (RPs using the short-lived status endpoint instead may see it sooner — the two channels have different latency/durability tradeoffs, both permitted) |
| Owner attempts to revoke a session that never existed / already revoked | N/A | No-op; not an error |

**Idempotency / retry.** Idempotent: revoking an already-revoked `session_ref` is a no-op.

**Example trace.**
```
owner: session-revoke(session_ref=SK.pub, reason="lost phone")
owner's node → KT: append RevocationRecord{ session_ref: SK.pub, ts:... }
KT ← owner's node: tree_head, inclusion_proof
RP (on next status check, or via push if subscribed): sees SK.pub revoked
RP: subsequent DPoP proof signed by SK → rejected as "session revoked"
```

### 19.6.5 `oidc-bridge-issue(assertion) → IDToken`

**Purpose.** Let a legacy OIDC/OAuth relying party — one that only speaks fixed-issuer OIDC, not
DMTAP-Auth natively — consume a DMTAP login via the hosted bridge OP (§13.6).

**Initiator / Responder.** Initiator: the bridge OP, on behalf of the legacy RP, after the §13.3
ceremony completes against it (the bridge acts as the RP for the native ceremony, then re-issues
as an OIDC IdP for the legacy RP). Responder: the legacy RP, which consumes a standard ID Token.

**Parameters.**
- `assertion` (`SignedAssertion`, MUST) — the native DMTAP-Auth assertion (§19.6.2), with the
  bridge itself as `aud`/`rp_origin` for that inner ceremony.
- `legacy_client_id` (`tstr`, MUST) — the OIDC client id of the legacy RP that redirected the
  user to the bridge.

**Preconditions.**
1. The bridge has already performed the full native ceremony (§19.6.1–§19.6.3) with **itself**
   as the relying party — the legacy RP never sees the DMTAP key material or the native
   assertion directly.
2. The bridge is a registered/known OIDC provider from the legacy RP's perspective (standard
   OIDC client registration, out of DMTAP's scope).

**Procedure (normative, §13.6).**
1. Bridge performs `auth-challenge`/`auth-assert`/verifies, exactly as any RP (§19.6.1–.2), with
   `aud`/`rp_origin` set to the bridge's own identifiers.
2. On success, bridge mints a standard OIDC **ID Token** (JWT) asserting `sub = alice@yourdomain`
   (or a bridge-stable subject identifier), signed by the bridge's own OIDC signing key —
   **not** by the user's DMTAP key (the bridge's signature is what the legacy RP's OIDC library
   already knows how to verify; expressing the binding as `did:web:yourdomain:users:alice`,
   §13.6, is available to any consumer sophisticated enough to check it, but the common case is
   plain OIDC verification against the bridge's own JWKS).
3. Bridge returns the ID Token via the standard OIDC authorization-code or implicit flow the
   legacy RP already implements.

**Success result.** The legacy RP receives a standard-shaped ID Token it can verify with its
existing OIDC library, with no DMTAP-specific code required on its side.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Inner native ceremony fails (any `auth-assert` failure mode) | Reject | Bridge surfaces a standard OIDC error (`access_denied` or similar) to the legacy RP; the legacy RP never learns the DMTAP-specific reason, by design of the bridge abstraction |
| `legacy_client_id` not registered with the bridge | Reject | `unauthorized_client` (standard OIDC error) |
| Bridge's own signing key is compromised | Reject (systemic, not per-call) | Same blast radius as any compromised OIDC IdP — bounded to *login*, and explicitly stated as **not** touching the user's actual DMTAP key (§13.6: "it sees login events but never the user's key") |

**Idempotency / retry.** Each ID Token issuance corresponds to one completed native ceremony
(itself non-replayable, §19.6.2); a legacy RP's standard OIDC retry/refresh-token flow governs
subsequent token refresh, outside DMTAP-Auth's own scope beyond the initial bridge login.

**Example trace.**
```
legacy_rp → bridge: standard OIDC authorize redirect (client_id="legacy-app-123")
bridge (acting as RP): auth-challenge(rp_origin="https://bridge.dmtap-auth.example",
                                       aud="bridge.dmtap-auth.example")
... full §19.6.1–.2 ceremony with the user's trusted client ...
bridge: verifies assertion                                        # OK
bridge: oidc-bridge-issue(assertion, legacy_client_id="legacy-app-123")
bridge: mint ID Token{ sub:"alice@yourdomain", iss:"https://bridge.dmtap-auth.example",
                       aud:"legacy-app-123", sig: bridge_signing_key }
bridge → legacy_rp: authorization code → (token endpoint) → ID Token
legacy_rp: verifies ID Token against bridge's published JWKS (standard OIDC)   # OK, "logged in"
```

## 19.7 Gateway operations (§7)

### 19.7.1 `smtp-inbound(smtp_transaction) → mote | 4xx`

**Purpose.** Translate an inbound legacy SMTP message into an attested MOTE and deliver it into
the mesh (§7.2), or return SMTP `4xx` so the sending server retries if the recipient is
unreachable — durability punted to the legacy sender (§7.4).

**Initiator / Responder.** Initiator: an external SMTP sender (e.g. Gmail's outbound MTA).
Responder: the gateway (acting as MX for the domain).

**Parameters.**
- `smtp_transaction` (MUST) — the full SMTP dialogue: `MAIL FROM`, `RCPT TO`, `DATA` (RFC 5322
  message).

**Preconditions.** The gateway is configured as MX for the recipient domain (§3.2, §3.8).

**Procedure (normative, §7.2, with the attestation binding of §7.2a).**
1. Accept the SMTP connection; **reject spam early, before `DATA` where possible** (RBL/DNSBL,
   SPF/DMARC, greylisting, per-IP rate limits, §9) — never accept the bulk of spam onto the wire
   at all.
2. Look up the recipient key `K` for `RCPT TO` via DNS/directory (`resolve`, §19.1.1, run by the
   gateway on the recipient's behalf).
3. Wrap the RFC 5322 message into a MOTE (`kind=0x00 mail`), encrypt to `K`.
4. Set an **attestation**: the gateway signs `"received via gateway G at T from <SMTP
   envelope>"` under its **domain-anchored attestation key** — the key published at
   `<sel>._dmtap-gw.<domain>. TXT` (§7.2a), never the gateway operator's own arbitrary key.
5. Deliver the resulting MOTE into the mesh, addressed to `K` (`deliver`, §19.3.1, run at the
   recipient's node once it arrives).
6. Deliver, then **wait for the recipient node's `ack` (§19.3.1) within the inbound SMTP
   transaction window** before replying to the legacy sender.
   - **`ack` received** → return SMTP **`250 OK`** — the MOTE is now durably held by the
     recipient (or a relay-mailbox that has itself acked durable custody, §14.5), so the
     durability handoff is complete.
   - **No `ack` within the window** — because `K`'s node is unreachable (reachability ladder +
     any relay-mailbox buffering exhausted), or reachable but not yet durably accepted — return
     SMTP **`4xx`** (`451`, §21.9) so the **legacy sender's own MTA queue retries**; the gateway
     stores nothing (§7.4).

   **Silent-loss avoidance (normative, closes the DSN gap).** The gateway MUST NOT return `250`
   on mere mesh *hand-off* (a best-effort buffer accepting the packet) — only on a durable
   `ack`. A stateless gateway (§7.4) cannot generate a delivery-status notification later, and
   the inbound SMTP transaction closes at its reply; returning `250` before durable acceptance
   would let a subsequent mesh-side `EXPIRED` (§19.3.3) drop the message while the legacy sender
   believes it was delivered — an un-notified loss. Deferring with `451` instead keeps durability
   in the legacy sender's queue (which *can* bounce after its own retry window), exactly per §7.4.

**Success result (recipient reachable).** The recipient node has `ack`ed durable custody of the
MOTE and the gateway returned SMTP `250 OK`; there is no post-`250` window in which the message
can be silently lost, because `250` is emitted only after that `ack`.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Spam signals present pre-`DATA` (RBL/DNSBL hit, SPF/DMARC fail, rate-limit exceeded) | Reject | SMTP-level rejection before `DATA` (standard SMTP `5xx`), per §9 — never wrapped into a MOTE at all |
| `RCPT TO` does not resolve to any known DMTAP recipient key | Reject | SMTP `550` (no such user), standard MX behavior |
| Recipient node unreachable (all reachability-ladder rungs + buffering exhausted) | Defer | SMTP **`451`**; sending server retries per its own SMTP retry schedule — this is the *entire* durability mechanism for this path, since the gateway is stateless (§7.4) |
| Recipient node reachable but does **not** durably `ack` within the transaction window (or only a best-effort buffer accepted the packet) | Defer | SMTP **`451`** — the gateway MUST NOT return `250` on mere hand-off; without a durable `ack` a later mesh-side `EXPIRED` (§19.3.3) would silently lose the message after the SMTP transaction closed. Deferring keeps durability in the legacy sender's queue (step 6, silent-loss avoidance) |
| Attestation key not yet published for this domain (misconfiguration) | Reject (operational) | The gateway MUST NOT deliver an unattestable MOTE as if it were attested; implementations SHOULD refuse to accept mail for a domain whose own attestation key isn't configured, surfaced as an operator-side configuration error, not a per-message SMTP failure |
| Recipient's node rejects the attestation (attestation key not published under the recipient's own domain, or doesn't verify, §7.2a) | Reject (at the recipient, via ordinary `deliver`, §19.3.1) | The recipient node MUST reject an attestation that does not verify and MUST mark accepted ones as legacy-origin; this is enforced recipient-side, not gateway-side — the gateway cannot force acceptance |

**Idempotency / retry.** SMTP itself is not idempotent at the transaction level (a sending
server retrying after `4xx` re-submits the full transaction); the resulting MOTE's `id` is a
fresh content address each time unless the RFC 5322 bytes are byte-identical, in which case
dedup (§2.6) at the recipient absorbs true duplicates.

**Example trace.**
```
gmail-mta → gateway: MAIL FROM:<bob@gmail.com>
gateway ← gmail-mta: RCPT TO:<alice@example.org>
gateway: pre-DATA checks — SPF pass, not on RBL, rate limit OK
gateway → gmail-mta: 250 2.1.5 OK (proceed to DATA)
gmail-mta → gateway: DATA ... <RFC 5322 message> ... .
gateway: resolve alice@example.org → K=alice_ik
gateway: wrap → MOTE{ kind:0x00, to:K, ciphertext: HPKE(K, rfc5322_bytes) }
gateway: attest with sel._dmtap-gw.example.org key: sign("received via gateway G at T from
                                                          bob@gmail.com")
gateway: attempt-reachability(K) → alice's node reachable via relay
gateway: deliver(mote) → mesh → alice's node
alice's node: verify attestation sig against sel._dmtap-gw.example.org TXT record   # OK
alice's node: mark legacy-origin; store to inbox; ack
gateway → gmail-mta: 250 2.6.0 message queued
```

**Example trace (recipient offline, 4xx).**
```
gmail-mta → gateway: MAIL FROM/RCPT TO/DATA as above, recipient carol@example.org
gateway: resolve, wrap, attest as above
gateway: attempt-reachability(carol_ik) → all rungs fail; no relay-mailbox buffering configured
gateway → gmail-mta: 451 4.4.1 carol's node has not durably accepted yet, try again later
gmail-mta: schedules its own SMTP retry per its standard backoff — durability now lives on
           Gmail's side, not the gateway's (§7.4)
```

### 19.7.2 `smtp-outbound(mote) → smtp_result`

**Purpose.** Translate an outbound `mail` MOTE addressed to a legacy recipient into RFC 5322 and
send it via SMTP, DKIM-signed under a delegated selector (§7.3).

**Initiator / Responder.** Initiator: the sending node (over the mesh, to its configured
gateway). Responder: the destination legacy MX.

**Parameters.**
- `mote` (`Envelope`+decrypted `Payload`, MUST) — a `mail` MOTE whose recipient is a legacy
  address (no DMTAP `Identity` resolves for it, or the sender explicitly chose the legacy path).

**Preconditions.**
1. The sender's node has a configured gateway relationship (its own self-hosted gateway, or a
   third-party operator, §7.5, §7.7).
2. The domain the mail claims to be `From:` has published the gateway's delegated DKIM selector
   at `<selector>._domainkey.<domain>` (§3.8, §7.3).

**Procedure (normative, §7.3).**
1. Node sends the `mail` MOTE to its gateway over the mesh (authenticated).
2. Gateway translates the MOTE into RFC 5322.
3. Gateway **DKIM-signs as the sender's domain using the delegated selector** — signing `d=
   <domain>` without ever holding the user's DMTAP identity key (the delegation is exactly what
   makes this safe, §7.3).
4. Gateway SMTPs to the destination MX, enforcing TLS via MTA-STS/DANE.
5. **On failure, the gateway reports back to the node; the node retries** (§19.3.3's ordinary
   sender-retry state machine governs this, treating gateway-reported SMTP failure the same as
   any other undelivered send) — the gateway itself stores nothing (§7.4).

**Success result.** The message is delivered to the legacy destination MX with a passing DKIM
signature aligned for DMARC.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| Delegated DKIM selector not published for the `From:` domain | Reject | The gateway MUST NOT sign for a domain it isn't delegated for; the send fails at the gateway and is reported to the node as a configuration failure, not retried blindly (retrying an unfixable config error would only waste attempts) |
| Destination MX rejects for content/reputation reasons (e.g. gateway IP not warmed for that ISP, §14.2) | Reject or Defer, depending on the SMTP response code | A `5xx` from the destination is a hard reject, reported to the node (surfaced to the user as failed); a `4xx` is transient, and the gateway's own short retry (bounded, §7.4: "no queue") or the node's own re-submission governs — a gateway MUST NOT silently hold a long-lived retry queue itself |
| TLS enforcement (MTA-STS/DANE) fails against the destination | Reject | Send aborted rather than falling back to unencrypted SMTP silently, per the gateway's TLS-enforcement requirement |
| Gateway itself is unreachable from the node | Defer | Ordinary reachability-ladder/sender-retry handling (§19.2.3, §19.3.3) — the node retries reaching *its own gateway*, distinct from the gateway retrying the destination |

**Idempotency / retry.** Re-submission of the same MOTE to the gateway after a reported failure
is safe (the gateway is stateless and re-translates fresh each time); the node's retry state
machine (§19.3.3) governs pacing.

**Example trace.**
```
alice's node → gateway: MOTE{ kind:0x00, to: legacy("bob@gmail.com"), ciphertext:... }
gateway: decrypt (authenticated mesh channel to the node it serves, not sealed-sender — this
                  leg is node↔own-gateway, not node↔stranger)
gateway: translate → RFC 5322
gateway: DKIM-sign d=alice-domain.com using delegated selector "dmtap1._domainkey"
gateway: MTA-STS check for gmail.com                              # OK, enforced TLS available
gateway → gmail-mx: SMTP transaction over TLS 1.3
gmail-mx → gateway: 250 2.0.0 OK
gateway → alice's node: delivery report: success
```

## 19.8 Files operations (§2.5, §5.5, §16.4)

### 19.8.1 `offer-file(manifest, key) → mote`

**Purpose.** Announce a content-addressed file to a recipient by sending the manifest + content
key as a MOTE (§2.5, §5.5), selecting the correct size tier (§16.4) for the control message and
the chunk-transfer path.

**Initiator / Responder.** Initiator: the sending node. Responder: the recipient node (via
ordinary `deliver`, §19.3.1), and — implicitly — every future chunk-swarm participant
(§19.8.2).

**Parameters.**
- `manifest` (`Manifest`, MUST) — `{id, size, chunk_sz, chunks, suite}` (§5.5): the BLAKE3
  Merkle-DAG root over chunk hashes. The Manifest carries **no** key (§18.3.8).
- `key` (`enc-key`, MUST) — the per-file content key, delivered **only** in the sealed MOTE as
  `Attachment.key` (§18.3.7), never inside the swarm-distributed `Manifest`.
- `attach_mode` (implicit, derived from `size`, not caller-chosen) — the tier (§2.5/§16.4)
  the file falls into, which determines whether it inlines or travels as a manifest reference:

  | Tier | `size` | Where the bytes go |
  |---|---|---|
  | **inline** | ≤ 64 KiB (padded) | `Attachment.inline`, inside the MOTE itself |
  | **normal** | ≤ 4 MiB (≤ 4 chunks) | Manifest in the MOTE; chunks fetched via the **mixnet** (full privacy) |
  | **large** | > 4 MiB | Manifest in the MOTE; chunks fetched via the **fast/onion bulk path** (weaker privacy, §6.5) |

**Preconditions.** The file's chunks (for normal/large tiers) already exist, content-addressed
and encrypted under `Attachment.key`, held by at least the origin node (§5.5).

**Procedure (normative).**
1. Compute `size`; select the tier per the table above (§16.4's numeric thresholds are tunable
   parameters, but the three-tier model itself is normative, §2.5).
2. **Inline tier:** embed the bytes directly in `Attachment.inline`; no separate manifest fetch
   is ever needed — the file rides the message's own privacy tier end to end.
3. **Normal/large tier:** construct `Attachment{name, mime, size, manifest: ManifestRef{id, size,
   chunks}, key}` and include it in the MOTE's `Payload.attach` (§2.5). Send the MOTE as the
   **control message**, always at the `private` tier regardless of the file's own tier (§4.5:
   "the control MOTE... travels the private tier").
4. Recipient's `deliver` (§19.3.1) processes the control MOTE exactly like any other message;
   the manifest + key are now known to the recipient, who can begin `fetch-chunk` (§19.8.2)
   using the tier-appropriate path.

**Success result.** The recipient holds the manifest + key and can begin fetching chunks; for
inline files, the recipient already has the complete file from this operation alone.

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| `size` computed inconsistently with the actual chunk set (manifest doesn't match declared size) | Reject (at the recipient, during chunk verification, §19.8.2) | Not detectable at offer time by the sender's own honest computation, but a *received* manifest that self-contradicts is rejected at fetch time — chunk hashes and the Merkle root are what's authoritative, not the `size` field alone |
| Control MOTE itself fails any `deliver` precondition (§19.3.1's ordinary failure modes) | Per §19.3.1 | Identical handling — `offer-file` is not a distinct wire mechanism from ordinary MOTE delivery, only a payload-shape convention |
| Origin node goes offline before any chunk is fetched (large/normal tier) | Defer | The manifest/key already delivered are durable at the recipient; chunk fetch (§19.8.2) simply has no source yet until the origin (or any other holder, once swarmed) comes back online |

**Idempotency / retry.** The control MOTE follows ordinary MOTE retry/dedup semantics
(§19.3.1/§19.3.3); re-offering the identical manifest produces a MOTE with a different `id`
(fresh ciphertext/timestamp) even though the *file* content address (`manifest.id`) is
unchanged and dedupes naturally at the chunk-storage layer.

**Example trace (large file).**
```
A: offer-file(manifest=Manifest{ id:blake3:c4a1..., size:52428800, chunk_sz:1048576,
                                  chunks:[...50 hashes...], suite:1 },   # NO key in the Manifest
              key=k_file, recipient=bob_ik)                              # key travels in the Attachment
A: size=50MiB > 4MiB normal threshold → LARGE tier
A: construct Attachment{ name:"video.mp4", mime:"video/mp4", size:52428800,
                         manifest: ManifestRef{id:c4a1..., size:52428800, chunks:50}, key:k_file }
A: MOTE{ kind:0x05 file_offer, to:bob_ik, payload:{ attach:[Attachment{...}] } } — private tier
A: deliver() as ordinary control MOTE → bob's node
bob: ack(id); manifest (chunk list) + Attachment.key now held; begins fetch-chunk() over the fast/onion bulk path
```

### 19.8.2 `fetch-chunk(chunk_hash, sources) → chunk`

**Purpose.** Retrieve one content-addressed, encrypted chunk from any swarm holder
(BitTorrent-style), enabling resumable, parallel, deduplicated transfer (§5.5).

**Initiator / Responder.** Initiator: the fetching node. Responder: any current holder of the
chunk (origin node, or another peer that has already fetched and cached it).

**Parameters.**
- `chunk_hash` (`bytes`, MUST) — one entry from `manifest.chunks`.
- `sources` (`[* peer_id]`, OPTIONAL) — known holders; if absent, discover via the swarm/DHT.
- `parallelism` (`u32`, OPTIONAL, default per §16.4: ≤ 8 concurrent sources per file).

**Preconditions.** The fetcher holds a verified `manifest` containing `chunk_hash`
(`offer-file`'s success result, §19.8.1), and `Attachment.key` to decrypt the chunk once fetched.

**Procedure (normative, §5.5, §4.5).**
1. Identify candidate sources: the origin node, plus any peer known to have already fetched this
   `chunk_hash` (swarm discovery — mechanism is transport-layer, analogous to BitTorrent
   peer-exchange, not separately specified at the object-format level in v0).
2. Fetch from up to `parallelism` sources concurrently, over the tier-appropriate path (mixnet
   for normal-tier files, fast/onion bulk path for large-tier, §4.5, §6.5).
3. On receipt, **verify the chunk self-verifies against `chunk_hash`** (content-address
   integrity, §5.5) before accepting it into local storage.
4. Decrypt using `Attachment.key`.
5. Once all chunks in `manifest.chunks` are fetched and verified, the file is complete and
   itself becomes available as a source for other swarm participants (deduplication + swarm
   growth, §5.5).

**Success result.** A verified, decrypted chunk added to local storage; once all chunks are
present, the reconstructed file is available (streamable in manifest order before full
completion, §5.5).

**Failure modes.**

| Condition | Class | Response |
|---|---|---|
| A fetched blob does not hash to `chunk_hash` (corrupt or malicious source) | Reject | Discard; retry from a different source — content addressing makes a bad source immediately detectable, never silently accepted |
| No source currently holds the chunk (origin offline, no swarm cache yet) | Defer | Retry later (bounded by the fetcher's own patience/UI, not a fixed protocol deadline distinct from the file offer's own `expires` if any); this is the honest cost of best-effort availability (§5.5's tier table) absent a paid durable/always-available replica |
| Decryption under `Attachment.key` fails (wrong key, corrupted manifest) | Reject | `CHUNK_DECRYPT_FAILED`; if this occurs for *every* chunk, treat the whole manifest as invalid and do not present a partially-decrypted file to the user |
| Large-tier fetch traverses the fast/onion bulk path, exposing the fact/approximate size of the transfer to a well-positioned observer | N/A (disclosed limit, not a failure) | This is the accepted §6.5 tradeoff for this tier, not an error condition — implementations MUST NOT claim mixnet-grade metadata privacy for large-file chunk fetch |

**Idempotency / retry.** Fully idempotent per chunk (content-addressed fetch of an immutable
blob); safe to retry against any source, and safe to fetch the same chunk redundantly from
multiple sources for resumability (the swarm model assumes this).

**Example trace.**
```
bob: fetch-chunk(chunk_hash=blake3:aa01..., sources=[alice_node, dave_node (already cached)])
bob → alice_node, dave_node: request chunk blake3:aa01... (parallel, 2 of ≤8 allowed sources)
dave_node → bob: chunk_bytes (arrives first — dave had it cached from an earlier fetch)
bob: verify blake3(chunk_bytes) == aa01...                        # OK
bob: decrypt chunk_bytes under Attachment.key                        # OK
bob: chunk 1/50 complete; repeat for remaining 49 chunks (parallel, ≤8 at a time)
bob: (alice_node's response arrives late/redundant — discarded, already have this chunk)
bob: all 50 chunks verified → file reconstructed → available to further swarm requesters
```

## 19.9 JMAP method mapping (§8.1)

**Purpose.** JMAP (RFC 8620/8621) is the native client-sync surface over the MOTE store; this
table is the normative mapping from each JMAP method a client calls to the underlying MOTE-store
operation(s) in this appendix, so an implementer of the JMAP server surface knows exactly what to
invoke.

| JMAP method | Underlying MOTE-store operation(s) | Notes |
|---|---|---|
| `Email/query` | Local index query over stored MOTEs (`kind=0x00/0x01`) filtered by mailbox/label CRDT state (§5.6) | Pure read over already-`deliver`ed/drafted state; no network operation |
| `Email/get` | Local read of a stored MOTE's decrypted `Payload` (already decrypted at `deliver` time, §19.3.1 step 7) | The node caches plaintext at rest (§6.7); JMAP never re-decrypts per-request |
| `Email/set` (create = send) | Construct `Payload` → `QUEUED` (§4.7) → sender-retry state machine (§19.3.3), which itself invokes `attempt-reachability` (§19.2.3) and ultimately a remote `deliver` (§19.3.1) | A JMAP "send" is exactly the sender side of §19.3; JMAP does not define a new send mechanism |
| `Email/set` (update = flag/label/move) | Local CRDT state change (§5.6), replicated across the device cluster | No wire MOTE is produced; this is why "mark read" has no protocol object (§17.1 item 18) |
| `Email/set` (destroy) | Local deletion; optionally a cooperative `kind=0x04 redact` MOTE (§2.3) sent to original correspondents, which is itself an ordinary `deliver`/`ack` exchange, non-enforceable at the recipient (§6.6 item 8) | `Email/set destroy` is local-authoritative; the `redact` MOTE is a hint, not a guarantee |
| `Email/changes` | CRDT delta query since a client-held state token (§5.6) | Sync primitive, no new MOTE operation |
| `Mailbox/query`, `Mailbox/get`, `Mailbox/set`, `Mailbox/changes` | Same as `Email/*` but over the folder/label CRDT namespace (§5.6) rather than message content | Folders are pure organizational metadata, §17.1 item 1 |
| `Thread/get`, `Thread/changes` | Derived view over `Headers.thread` + `Payload.refs` (§2.4) across already-stored MOTEs | No independent storage — a thread is a read-time reconstruction |
| `EmailSubmission/set` (create) | Identical to `Email/set` create — constructs and queues a `mail`-kind MOTE via §19.3.3; for a legacy destination, hands off to `smtp-outbound` (§19.7.2) instead of native `deliver` | JMAP's separate submission object exists to track delivery status distinctly from the message object itself; DMTAP surfaces the sender-retry state machine's states (`QUEUED/SEALED/IN_FLIGHT/ACKED/RETRY/EXPIRED`, §4.7) through `EmailSubmission`'s status field |
| `EmailSubmission/get` | Read of the current sender-retry state (§4.7/§19.3.3) for a given submission | Direct exposure of the internal state machine to the client |
| Calendars/Contacts `query`/`get`/`set`/`changes` (§8.4) | Identical pattern to `Email/*`/`Mailbox/*`, but over `kind`-equivalent JSCalendar/JSContact MOTEs (carried as MOTE payload content, not new `kind` values beyond what §2.3 already reserves for structured content types) and their own CRDT-replicated metadata | Calendar invitations additionally route through ordinary `deliver`/`ack` between participants (§8.4: "invitations... ride as MOTEs"), i.e. no separate scheduling-server operation exists — `post-to-group`-style fan-out applies if the invitation goes to a group address (§19.5.3) |
| Push (`EventSource`/WebSocket) | Server-sent notification on any local state change from `deliver` (§19.3.1), CRDT sync (§5.6), or submission-state transition (§19.3.3) | Not a distinct MOTE operation — a transport-level notification of state this appendix already defines |

## 19.10 Gaps flagged (undefined behavior surfaced, not left implicit)

Per this appendix's own rule ("every failure path must have a defined outcome"), the following
points were surfaced. **Update — five of the original six are now RESOLVED** by the
security-hardening pass; each is retained below with its resolution so the audit trail is intact,
and only item 5 remains open (minor, closeable by one extension header). Where an operation above
still reads "see §19.10," it now inherits the pinned §16.8/§5.1 values described here.

1. **Committer election quorum size** (§19.5.5, §5.1). **RESOLVED.** A takeover/failover Commit is
   valid only with a strict-majority **`> n/2` roster quorum** (⌈(n+1)/2⌉ of current members),
   pinned in **§16.8** and normative in §5.1 — this is what prevents two partitions each electing
   a rival successor. §19.5.5 step 3 now cites it directly.
2. **Fork resolution procedure** (§19.5.6, §5.1/§6.6 item 7). **RESOLVED.** §5.1 "Fork recovery
   (out of HALT)" now defines it: members identify the **last common epoch**, an `admin`/`owner`
   proposes a recovery Commit on top of it, and that Commit is canonical only with the same
   **`> n/2` member-signature quorum** as a takeover (denying any single admin unilateral
   fork-selection); losing-fork members roll back to the last common epoch and re-apply, with
   abandoned-fork application messages re-sent by sender retry (§2.6). Decentralized MLS
   (`draft-kohbrok-mls-dmls`) remains the eventual leaderless fix.
3. **Join-request expiry** (§19.5.4, §5.8.2). **RESOLVED.** §16.8 now fixes **Group join-request
   expiry = 30 days** (mirroring the requests-area retention of §16.5), so a `request`-mode join
   with no admin response is auto-expired/cleaned up.
4. **Session-revocation propagation latency** (§19.6.4, §13.4). **RESOLVED.** §16.8 now bounds it
   via the **RP delegation re-validation interval ≤ 15 min** (plus session TTL 24 h / idle 30 min),
   so a revoked session cannot outlive the next RP status/KT re-check by more than that window.
   §19.6.4's "stale cache" failure mode inherits this bound.
5. **Auto-forward / delegate-attribution headers** (referenced from §17, not restated as an
   operation here because no wire object exists for it yet) — **OPEN.** Out of this appendix's
   scope since there is no operation to specify without the `Headers.ext` convention §17.6
   recommends; not double-counted in the operation total below.
6. **Node liveness timeout that triggers committer failover** (§19.5.5 precondition 2, §5.1).
   **RESOLVED.** §16.8 now fixes the **committer-liveness timeout = 5 min** plus a
   **takeover hysteresis of 2 consecutive misses** (avoiding churn on transient NAT/relay blips),
   so rotation no longer depends on an unstated implementation timeout.

Five of these six gaps are now closed by pinned §16.8 parameters and §5.1 procedure (none of
which required inventing operation *behavior*); item 5 alone remains, awaiting the `Headers.ext`
extension convention, and its operation still has a fully-defined "defer, pending the convention"
outcome rather than silence — per this appendix's own normative charter.

## 19.11 Operation count

This appendix specifies **34** operations (plus the JMAP mapping table, §19.9, which maps
existing JMAP methods onto them rather than defining new ones):

- Naming (§19.1): 3 — `resolve`, `publish-identity`, `publish-move`
- Reachability (§19.2): 3 — `publish-location`, `lookup-location`, reachability-ladder attempt
- Delivery (§19.3): 3 — `deliver`, `ack`, sender-retry state machine
- Async init (§19.4): 3 — `fetch-keypackage`, `add-member`/`Welcome`, `external-commit`
- Group (§19.5): 6 — `create-group`, the four Commit-based management ops (grouped as one entry
  since they share one spec block, §19.5.2), `post-to-group`, `join`, `committer-elect`/`rotate`
  (one entry), fork-detection halt
- Auth (§19.6): 5 — `auth-challenge`, `auth-assert`, `session-establish`, `session-revoke`,
  `oidc-bridge-issue`
- Gateway (§19.7): 2 — `smtp-inbound`, `smtp-outbound`
- Files (§19.8): 2 — `offer-file`, `fetch-chunk`

(3+3+3+3+6+5+2+2 = 27 top-level spec blocks; counting the four individually-role-gated group
sub-operations inside §19.5.2 as distinct operations, per the task's enumeration of
"add/remove/role-change/policy-change" as four things, brings the total to **34**.)






