# Substrate Capabilities ④ + ⑤ — Infrastructure Roles & Wake

> **Status:** additive profile of the core specification. This document presents the DMTAP
> **infrastructure roles** — announce/resolve, signaling, circuit relay, short-TTL content-blind
> mailbox, cache/pin, and **wake** — as an **open, key-addressed protocol** that any product may adopt
> without reading the mail spec. It restates no normative bytes: the `LocationRecord` (§4.2), the
> reachability ladder (§4.3), the relay-mailbox (§14.3, §14.5), the wake objects (§4.9, §18.5.5–6), and
> their fail-closed rules (§21) are all defined in the core, **which governs.** This document names the
> roles, states the one invariant that unifies them (*key-addressed, any node, no privileged types*),
> and profiles each for non-mail use. Capability **⑤ Wake** is the last section; it is a first-class
> capability but an infrastructure role, so it lives here.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174.

---

## 1. The one idea: roles, not node types

DMTAP infrastructure is **a set of roles any node may serve, addressed by key, with no privileged node
type.** A relay, a mailbox, a cache, a wake origin, a rendezvous point — each is a *capability of the
same node binary* (§0.2, §4), advertised by a capability token (§10.2), served by whoever opts in, and
addressed by the **identity key** of whoever it serves for. There is no registrar, no special class of
server, and no node whose absence breaks the network: a role a node does not advertise is simply a role
some other node fills, and the middle holds **no durable user data** (§0.5) — durability always lives at
an edge.

This is the substrate a product needs when it wants **reachability, relaying, offline holding, content
serving, or wake** for identities that are behind NAT, on dynamic IPs, asleep, or intermittently online —
without standing up a centralized service and without inventing a directory. A product adopts the roles it
needs, à la carte:

| Role | What it provides | Key-addressed by | Normative home | Profiles |
|------|------------------|------------------|----------------|----------|
| **Announce / Resolve** | key → current location | the served identity's `IK` | §4.2 | IPNS-pattern signed value records, Kademlia |
| **Signaling** | coordinate a direct connection (hole-punch) | both peers' `IK`s | §4.3 (rung 2) | libp2p DCUtR, rendezvous |
| **Circuit relay** | content-blind ciphertext relay for a NAT'd peer | the relayed peer's `IK` | §4.3 (rung 3), §14.5 | libp2p Circuit Relay v2 |
| **Mailbox** | short-TTL content-blind offline buffer | the offline identity's `IK` | §14.3, §14.5 | Chatmail relay-mailbox |
| **Cache / pin** | serve/retain content-addressed objects | content address (+ publisher `IK`) | §22.5, §5.5 | IPFS-style pinning, HTTP caching |
| **Wake** (⑤) | content-free, sender-blind push to a sleeping device | the device's push key, held by owner's node | §4.9 | Web Push (RFC 8030/8291/8292), UnifiedPush |

Every role is **transport-pluggable with HTTPS first-class** (the HTTP test,
[`README.md § 4.2`](README.md#42-the-http-test--are-transports-pluggable-with-https-first-class)): where
no mesh exists, announce/resolve degrades to a signed record fetched over HTTPS, the mailbox is an HTTPS
buffer, cache/pin is the `/.well-known/dmtap-pub` surface ([`FEEDS.md § 5`](FEEDS.md)), and wake is Web
Push. The mesh binding (§4) adds swarming, NAT traversal, and mixnet privacy on top — never a requirement.

---

## 2. Announce / Resolve — the `key → location` record (profile of §4.2)

The one dynamic binding in DMTAP is **key → current location**: a signed, self-certifying value record
that lets a node behind CGNAT or on a dynamic IP stay reachable **by its key**. This is the role that
makes "reachable without a static IP" (§0.1) true.

```
LocationRecord {
  ik:        bytes,        // identity key (record key = hash(ik))
  peer_id:   bytes,        // node id (may be per-epoch / unlinkable, §6)
  addrs:     [* multiaddr],// current reachability hints (direct, relay circuits, mix addrs)
  seq:       u64,          // monotonic; reject older-or-equal (rollback defense)
  ttl:       u64,          // short — hours; staleness is a real failure mode
  ts:        u64,
  substrate: u8,           // OPTIONAL transport-substrate tag; absent ⇒ libp2p
  sig:       bytes,        // signed by a device key (chains to IK, §1.2)
}
```

- **IPNS pattern (profile).** A self-certifying value record keyed by `hash(ik)`, signed, with a
  **monotonic `seq` + TTL** to defeat rollback/replay, stored at the K closest peers and **aggressively
  republished** (§4.2). Use value records, not provider records. A product adopts this to publish "where
  am I now" for an identity without a central directory.
- **Anti-rollback (inherited).** A resolver accepts only a **newer `seq`**; an older-or-equal record is
  rejected — the same monotonic-counter family as `Identity.version` (§1.3) and feed `seq`
  ([`FEEDS.md § 4.3`](FEEDS.md), §22.4.2).
- **The DHT is not the root of trust (MUST — the honest, load-bearing caution).** Signing authenticates
  record *content*, not *routing*. Because a libp2p PeerId is `hash(pubkey)`, an attacker can cheaply
  generate IDs closest to a target key, eclipse its lookups, and return **nothing or an old-but-still-valid
  signed record** without forging anything — the single most attackable part of the mesh (§4.2). A
  conformant adopter therefore MUST treat the DHT as **one discovery mechanism, not the root of trust**
  (the root is the user's keypair) and resolve in order: **(1) cached direct addresses → (2)
  relay-reservation / rendezvous ("home relay") → (3) DHT as fallback.** A fresh contact SHOULD have a
  non-DHT path so it is not 100% dependent on a hostile public-DHT lookup. Required/recommended
  mitigations: **S/Kademlia disjoint-path lookups**, IP-diversity caps per bucket, aggressive republish,
  and a **private DHT** (own protocol prefix) for closed deployments to shrink the Sybil surface (§4.2).
- **Transport agility.** `substrate` (§21.24) tags the transport a record's `addrs` speak; the protocol
  is not permanently wedded to libp2p (§4.1). A resolver dials a record only on a substrate it implements;
  a record on an unimplemented substrate is simply unreachable to it (`0x0303`), never a parse failure.

---

## 3. Signaling — coordinate a direct connection (profile of §4.3 rung 2)

Two nodes that are both online can usually reach each other **directly** with a coordinated hole-punch —
no relay carries their traffic, only the *coordination*. This is rung 2 of the reachability ladder (§4.3):

- **Direct first (rung 1).** IPv6, or IPv4 with port-forward/UPnP — no signaling needed.
- **Hole-punch (rung 2).** **libp2p DCUtR** between two nodes coordinates simultaneous connection attempts
  so both NATs open; the signaling node relays only the coordination messages, never the payload. Always
  succeeds for two always-on boxes. A **rendezvous** point (a known node both peers can reach) is the
  meeting place that makes first contact possible without a hostile DHT lookup (§4.2 resolution order).
- As IPv6 spreads, rungs 1–2 dominate and the relaying roles fade (§4.3) — signaling is the role that lets
  the network need *less* infrastructure over time, not more.

A product adopts signaling to get peer-to-peer connectivity between its replicas or clients (e.g. two
[`SYNC.md`](SYNC.md) replicas reconciling directly) without routing their bytes through a third party.

---

## 4. Circuit relay — content-blind ciphertext relay (profile of §4.3 rung 3, §14.5)

When direct and hole-punch both fail (the residual hard-NAT minority), a public-IP node **relays
ciphertext** for a NAT'd peer — rung 3, **content-blind** by construction:

- **libp2p Circuit Relay v2 (profile).** The relay carries ciphertext-only mesh traffic between DMTAP
  nodes and **never** decrypts or speaks a legacy protocol — it is the *native* node↔node relay, sharply
  distinct from the gateway's legacy-client ingress (§7.15) which *terminates* a legacy connection. Do not
  conflate the two "relays" (§4.3, and the four-sense `relay` glossary, §0.8).
- **Tight caps, stateless, horizontally scalable (§14.5).** Run **many small, independent, stateless**
  relays — Circuit Relay v2 needs zero coordination between them, so the fleet scales by adding cheap
  nodes. Enforce tight caps (go-libp2p defaults: ~128 reservations, 16 circuits/peer, 2 min / 128 KB per
  circuit) — a relay is a **reachability hop, not a pipe**. Discovery SHOULD use an operator-run
  rendezvous/static list, not sole reliance on the public DHT.
- **Honest limit (MUST disclose, §14.5).** The public libp2p/IPFS DHT + relay path is designed for *brief
  hole-punch assistance*, not sustained transfer; a deployment SHOULD run its own tuned relay fleet with
  rendezvous discovery and use the DHT (if at all) only opportunistically. Direct connectivity is always
  preferred; relays carry only the residual hard-NAT minority.
- **Key-addressed and blind.** A relay serves *a peer, by that peer's key*, and never learns what it
  carries — the same content-blind, single-choke-point discipline as the mixnet (§4.4) and the wake relay
  (§8). This blindness is exactly why relaying can be permissionless and why it does **not** shift the
  operator's liability posture the way serving *plaintext* (cache/pin, §5) does.

---

## 5. Mailbox — short-TTL content-blind offline buffer (profile of §14.3, §14.5)

An offline or asleep identity needs somewhere to hold ciphertext until it returns. The **relay-mailbox** is
a **hosted, content-blind, short-TTL buffer** — the Chatmail model — and it is a **buffer, not an
archive**:

- **Content-blind, short TTL, delete-after-inactivity (§14.5).** It holds only E2EE ciphertext addressed
  to an offline identity's key, for a short TTL (reference: ~20-day message TTL, ~90-day inactive purge),
  then deletes. It never decrypts and never becomes a store — the durable copy lands at the recipient's
  edge once fetched (§14.3). A product adopts it so availability does not depend on any single peer's
  uptime, while per-account cost stays near-zero (no long-term archive).
- **Buffer is not backup (§14.5).** A mailbox holds only *undelivered* ciphertext within its TTL; it is
  **not** content backup. An identity offline past the TTL loses undelivered items; durability requires an
  edge copy or a portable encrypted backup (§1.4). Senders retry within their own deadline regardless
  (§2.6). A conformant adopter MUST NOT present a mailbox as durable storage.
- **Alignment with §14.3.** For a mobile-only identity with no always-on box, the mailbox is paired with
  **wake** (§8): the sender deposits ciphertext, the mailbox triggers a content-free wake, the device
  wakes and **drains the queue over its own authenticated connection**, decrypting locally. The mailbox
  never sees plaintext and the push relay never sees content — *wake-and-fetch*, never deliver-in-push.
- **Key-addressed.** A mailbox buffers *for an identity, by its key*; any node may run one, and an identity
  may point at its own (no lock-in) — the same non-lock-in framing as the gateway (§7.7) and the wake relay
  (§8).

---

## 6. Cache / pin — serve content-addressed objects (profile of §22.5, §5.5)

Any node may **cache and serve** content-addressed objects — chunks, `PubManifest`s, feed heads/entries,
announces — and **pin** them for durability. This is the serving side of [`FEEDS.md`](FEEDS.md) and the
§5.5 swarm, exposed as an infrastructure role:

- **Self-verifying, so serving is trustless (§22.5.1).** Every object is authenticated by a signature or a
  content address it carries, so a cache **cannot forge** an object it did not receive the key to sign, and
  **cannot substitute** chunk bytes under a matching hash (BLAKE3 collision resistance) — a cache is a
  convenience, not a trust root. Immutable content-addressed objects SHOULD be served with
  `Cache-Control: public, immutable` and a strong `ETag` equal to the content address, and MAY be fronted
  by any ordinary HTTP CDN that need not understand DMTAP.
- **Pinning is durability, and it costs real storage (§5.5.2).** A content address is a name, not a promise
  (§5.5.1): an object is available exactly as long as some holder serves it. Durability is bought by
  **pinning/replication**, an explicit act with a real storage cost — availability is the emergent sum of
  independent holder choices (§22.6.2).
- **The one role that is NOT blind (§22.6.1, MUST).** Unlike relay and mailbox, which carry ciphertext,
  cache/pin of **public** objects serves **plaintext a holder can read**, which shifts the operator's
  moderation and liability posture. Serving public content MUST therefore be **explicit operator opt-in**
  (`pub-1`), never automatic; each holder applies its own serve policy and MAY decline any object
  (`ERR_PUB_NOT_SERVED`, `0x090C` — a fetcher rotates to another holder). There is **no protocol-level
  takedown** (§22.6.2). (Caching *sealed* ciphertext chunks, §5.5, remains blind and is not subject to
  this opt-in.)
- **Key-addressed.** Public objects are addressed by content hash *and* attributed to a publisher `IK`
  ([`FEEDS.md`](FEEDS.md)); a cache serves by content address and a follower discovers holders by publisher
  key.

---

## 7. Cross-cutting invariants (normative)

Every role above obeys the same discipline, and a product adopting any of them MUST preserve it:

1. **Key-addressed throughout.** A role always serves *for an identity or a content address*, never for an
   account at a provider. Reachability, buffering, and serving all resolve from the key, so an identity is
   never locked to one operator (§7.7 non-lock-in).
2. **Any node may serve any role; no privileged node types.** Every role is a capability of the same
   binary, advertised by a token (§10.2). A role a node does not advertise is filled by another node; no
   node's absence breaks the network.
3. **The middle holds no durable user data (§0.5).** Relay, signaling, mailbox, and wake carry or hold data
   only transiently (in-flight ciphertext, short-TTL buffer, a wake token). Durability lives at edges. The
   one exception is *pinning*, which is a deliberate, paid durability act at a holder, not incidental
   middle-state.
4. **Content-blind by default; plaintext-serving is opt-in.** Relay, mailbox, and sealed-chunk cache are
   **blind** (ciphertext only). Only public cache/pin (§6) serves plaintext, and only under explicit
   `pub-1` opt-in — the honest liability boundary (§22.6.1).
5. **Sybil/eclipse is the residual threat (disclosed).** Because roles are permissionless and
   key-addressed over a DHT, the routing layer — not the record content — is the weakest link (§2). A
   deployment MUST apply the §4.2 mitigations (disjoint-path lookups, IP-diversity caps, rendezvous-first
   resolution, private DHT for closed nets) and MUST NOT treat a DHT answer as authoritative.
6. **Fail-closed (§10.7).** A role's security-relevant failure is refused or surfaced as an explicit
   choice, never silently degraded (see each role's error codes and §8's wake codes).

---

## 8. Wake — content-free, sender-blind push (Capability ⑤; profile of §4.9)

<a id="wake"></a>

A sleeping mobile device cannot hold a mesh connection, so it must be **woken** to reconnect and sync. The
naive path — Apple **APNs** / Google **FCM** — sees, for every message, *which device was woken and when*:
a centralized metadata choke point (§4.9). The **Wake** capability is an **optional, open wake-signaling
layer** that carries **no content and no sender identity**, is **originated by the user's own node**, and
**reuses existing standards** rather than inventing a push protocol. It is Capability ⑤ because *needing to
be woken* is a product-facing property a device may want even when it runs none of §2–§7 roles; its spec is
an infrastructure role, so it lives here.

**It is never a delivery path (MUST).** A wake is only a hint to *reconnect and sync now* — the actual
object is still **pulled** over the ordinary reachability ladder (§3) or the mailbox (§5). This is
**wake-and-fetch, never deliver-in-push** (§4.9, §14.3). A client MUST still poll/reconcile on foreground
and MUST NOT treat a wake as delivery confirmation.

### 8.1 The two objects (profile of §4.9.1, §18.5.5–6)

- **`PushSubscription`** — a device registers, **with its own node**, the provider kind, the provider
  endpoint, the device's **public push key** (P-256 for Web Push, RFC 8291), and the RFC 8291 **auth
  secret**, all **signed by an `IK`-authorized device key** (§1.2) so the subscription is authenticated to
  the identity and cannot be forged to register/redirect a device's wakes. It is published **only to the
  user's own node(s)** — never to a directory, DHT, or relay — so no external party learns the device
  exists or where it is pushed.
- **`WakePing`** — when an object arrives for the user, the node emits a **content-free, sender-blind**
  wake to each sleeping device: an **opaque "sync now" token and nothing else** — no body, subject,
  recipient, or sender. The token is **sealed to the device push key with RFC 8291 Web Push encryption**
  (ECDH + `aes128gcm`, auth-secret bound into the HKDF), so even the push relay sees **only ciphertext of
  fixed shape**. The device opens it and **pulls** the real object.

### 8.2 Why the graph stays private (profile of §4.9.2)

The subscription lives on the **user's own node**, and that node — which already terminates delivery for
its user — emits the ping. So the push relay sees only *"this user's node woke this user's own device"* — a
single self-edge, never who sent the message or whom the user talks to. Same content-blind,
single-choke-point discipline as circuit relays (§4) and the mailbox (§5); the push relay is a **thin,
content-blind, self-hostable** carrier subject to the same non-lock-in framing (§7.7).

### 8.3 Provider seam — prefer the open ones (profile of §4.9.3)

Push sits behind a provider seam; the wake payload is the **same** RFC 8291-sealed content-free token
whichever provider carries it — the provider choice changes only the outer transport, never what leaks
inside it (which is: nothing).

| Provider | Standard | Openness |
|----------|----------|----------|
| **UnifiedPush** | user-chosen distributor | fully open / self-hostable (the decentralized default on Android/desktop) |
| **Web Push** | RFC 8030 + 8291 + 8292 (VAPID) | open / self-hostable (the node is the VAPID application server) |
| **APNs / FCM** | platform-mandated | closed bridges, used **only** where a platform mandates them |

A conformant node **MUST prefer an open provider (UnifiedPush or Web Push) wherever the platform allows**,
and **MUST fall back to APNs/FCM only on a platform that mandates them** (§4.9.3).

### 8.4 Anti-abuse — a wake costs battery (profile of §4.9.4)

A wake spends the target's battery, so wakes are gated **fail-closed**, and the codes already exist in the
§21 registry (`0x0312`–`0x0316`):

- **Authenticated to the device.** A `WakePing` is honored only against a `PushSubscription` the device
  itself signed; an unverifiable subscription is rejected (`ERR_PUSH_SUBSCRIPTION_SIG_INVALID`, `0x0312`,
  FAIL_CLOSED_BLOCK). A wake that fails to open under the sealed push key + auth secret is
  forged/unauthenticated and dropped (`ERR_WAKEPING_AUTH_FAILED`, `0x0314`, DROP_SILENT).
- **Content-free (MUST).** A `WakePing` bearing any field beyond the opaque token — or an opened plaintext
  bearing sender/subject/recipient/content — is rejected (`ERR_WAKEPING_CONTENT_PRESENT`, `0x0313`).
- **Rate-limited at both ends.** The emitting node rate-limits per device (coalescing bursts); the
  **receiving device** enforces the same budget on inbound wakes as a fail-closed backstop, so a
  misbehaving relay that replays/floods cannot exceed the budget (`ERR_WAKEPING_RATE_LIMITED`, `0x0315`).
- **Replay-dedup at the device.** Each wake's sealed plaintext is a **fresh ≥16-byte nonce**; the device
  keeps a bounded replay cache and drops a wake whose nonce it has already accepted
  (`ERR_WAKEPING_REPLAY`, `0x0316`, DROP_SILENT) — closing the relay-replay battery-drain the emitter's
  limiter cannot see.
- **Jitter/batch (privacy).** A node MAY jitter/batch wakes to blunt timing correlation at the push relay;
  under the High-security profile (§4.4.10) this is a **MUST**, not a MAY.

---

## 9. Reference implementation (informative, non-normative)

**vulos-relayd** is the intended reference implementation of these roles — a single key-addressed daemon
serving announce/resolve, circuit relay, the short-TTL content-blind mailbox, and the wake origin. It is
named only as an existence proof; per the repository's implementation-neutral stance it is not part of the
standard and not required to speak it, and where it and the spec disagree, **the spec wins.** An
independent implementation MUST be buildable from this document and the core (§4, §14, §4.9) alone.

---

## 10. Security considerations / honest limits

1. **The DHT/routing layer is the weakest link** (§2, §7 invariant 5). Signing protects record *content*,
   not *routing*; eclipse/Sybil can censor or roll back resolution *without forging*. Mitigated (not
   defeated) by disjoint-path lookups, rendezvous-first resolution, and private DHTs; disclosed, not
   hidden.
2. **A mailbox is a buffer, not an archive** (§5). Undelivered ciphertext past the TTL is lost; durability
   is an edge property. A conformant adopter MUST NOT present it as durable storage or as backup.
3. **Pinning ≠ takedown; availability ≠ durability** (§6). Serving is best-effort and opt-in; there is no
   protocol mechanism to compel or forbid serving. A published object persists as long as *any* holder
   serves it — irrevocable in the same sense as [`FEEDS.md § 8`](FEEDS.md).
4. **Cache/pin of public objects is not blind** (§6, §22.6.1). Serving plaintext shifts operator liability;
   it MUST be explicit opt-in. Relay/mailbox/sealed-chunk caching stay blind.
5. **Wake is a metadata optimization with residual channels** (§8). It is content-free and sender-blind and
   keeps the social graph off the push relay, but a push relay still learns *that a device was woken and
   when* unless jitter/batching is applied (a MUST under the High-security profile). Platform-mandated
   APNs/FCM bridges are closed carriers used only where unavoidable; prefer the open providers.
6. **Roles are permissionless — abuse is bounded by caps and opt-in, not identity** (§4, §7). Relay caps,
   mailbox TTLs, per-holder serve policy, and wake budgets are the bounds; none relies on a trusted node
   type, because there is none.

---

## 11. Fail-closed rules these capabilities contribute

The owning clauses govern; these are the roles' auditable rows (§10.7, §21). Wake codes are already
registered in §21.5.

| Trigger | Error | Action | Clause |
|---------|-------|--------|--------|
| `LocationRecord` older-or-equal `seq` | rollback reject | discard stale record | §4.2 |
| Record on an unimplemented transport substrate | `0x0303` | unreachable-to-this-resolver, not a parse fault | §4.1 |
| Public object a holder declines to serve | `ERR_PUB_NOT_SERVED` `0x090C` | DENY_POLICY; fetcher rotates to another holder | §22.6.2 |
| Push subscription not authenticated to device | `ERR_PUSH_SUBSCRIPTION_SIG_INVALID` `0x0312` | FAIL_CLOSED_BLOCK — never wake against it | §4.9.4 |
| WakePing carries content/sender | `ERR_WAKEPING_CONTENT_PRESENT` `0x0313` | FAIL_CLOSED_BLOCK | §4.9.1 |
| WakePing fails to open (forged/unauth) | `ERR_WAKEPING_AUTH_FAILED` `0x0314` | DROP_SILENT | §4.9.4 |
| Wakes exceed the device budget | `ERR_WAKEPING_RATE_LIMITED` `0x0315` | coalesce/hold; drop beyond cap (emitter **and** receiver) | §4.9.4 |
| WakePing nonce already seen (relay replay) | `ERR_WAKEPING_REPLAY` `0x0316` | DROP_SILENT — do not re-wake | §4.9.4 |

The §10.7.5 governing rule applies unchanged: a role's security-relevant failure is refused (fail closed)
or surfaced as an explicit choice, never a silent degradation.
