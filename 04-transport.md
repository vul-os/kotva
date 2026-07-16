# 4. Transport: Mesh, Mixnet, Delivery

The mesh finds you and moves ciphertext to you; the mixnet hides who is talking to whom.
The node **is** the mesh — relay and mix roles are capabilities of the node binary, not
separate services.

## 4.1 Substrate: libp2p

DMTAP builds on **libp2p** rather than reinventing P2P:

- **Kademlia DHT** — peer routing and the `key → location` record store (see §4.2 for its
  security limits — it is the weakest link).
- **Circuit relay v2** — reachability for NAT'd nodes (the "relay" role).
- **AutoNAT v2 + DCUtR (hole punching)** — direct connections where possible.
- **Noise** (or TLS 1.3) — hop-by-hop channel security (in addition to end-to-end MOTE
  encryption; QUIC carries TLS 1.3 natively).
- **QUIC / TCP / WebSocket / WebRTC / WebTransport** transports.

A node dials **outbound** and holds connections, so CGNAT/dynamic-IP nodes are reachable.

**Substrate seam — libp2p is v0, not a flag day (normative).** libp2p is DMTAP's **v0 substrate**,
but the protocol MUST NOT be *permanently* wedded to it: the `LocationRecord` (§4.2, §18.5.1)
carries an explicit **`substrate` discriminator** — a tag from the **Transport Substrates
registry** (§21.24) — and its `peer_id` / `addrs` are interpreted **relative to that substrate**.
v0 defines exactly one value, `0x01 = libp2p` (peer id = libp2p PeerId, addrs = multiaddrs), and
an absent `substrate` field means libp2p for backward compatibility. A future non-libp2p overlay
is introduced by **registering a new substrate tag** and advertising support via capability
negotiation (§10.2) — the **same additive, dual-stack migration mechanism as a new crypto suite**
(§1.1, §21.25): a resolver dials a record only on a substrate it implements, nodes bridge during
the transition, and the old substrate is retired only once no pinned relationship needs it. A
record on a substrate a resolver does not implement is simply unreachable to it (`0x0303`), never
a parse failure. This keeps `multiaddr` + the registered substrate tag as the abstraction seam so
moving off libp2p is an incremental migration, not a flag day.

**Roaming — honest note:** identity is a persistent keypair, not an IP. When a node's address
changes, roaming is carried primarily by **re-publishing the location record (§4.2) and peers
re-dialing**, not by QUIC connection migration. QUIC migration (RFC 9000) *can* preserve some
live connections, but the Rust QUIC stack (`quinn`) has no multipath and address-change
handling is imperfect; do not rely on it for seamless roaming.

## 4.2 The `key → location` record (DHT)

```
LocationRecord {
  ik:        bytes,        // identity key (DHT key = hash(ik))
  peer_id:   bytes,        // node id, interpreted per `substrate` (v0 libp2p PeerId; may be per-epoch / unlinkable, §6)
  addrs:     [* multiaddr],// current reachability hints (may be relay circuits, mix addrs)
  seq:       u64,          // monotonic sequence number; reject older-or-equal (rollback defense, §16.2)
  ttl:       u64,
  ts:        u64,
  substrate: u8,           // OPTIONAL transport-substrate tag (§21.24); absent ⇒ 0x01 libp2p (§4.1)
  sig:       bytes,        // signed by a device key
}
```

The record follows the **IPNS pattern**: a self-certifying **value record** (DHT key =
hash(ik)), signed, with a **monotonic sequence number + EOL/TTL** to defeat rollback/replay,
stored at the **K closest peers**, and **aggressively republished** (DHT record lifetimes are
short — hours — so staleness is a real failure mode). Use value records, not provider records.

### CAUTION — signing does NOT stop eclipse attacks (the DHT is the weakest link)

Signing authenticates record *content*, not *routing*. Because a libp2p PeerId is
`hash(pubkey)`, an attacker can cheaply generate IDs closest (XOR-distance) to a target key,
fill honest routing tables, and control all lookups for that key — returning nothing or an
*old, still-valid signed* record (censorship / rollback) **without forging anything**. This is
a Sybil/eclipse attack at the routing layer, and it is the single most attackable part of
DMTAP. Mitigations DMTAP REQUIRES/RECOMMENDS:

- **S/Kademlia disjoint-path lookups** (parallel, node-disjoint) and **IP-diversity caps** per
  k-bucket.
- **Aggressive republish** + accept only records with a newer sequence number (rollback
  defense).
- **The DHT is one discovery mechanism, not the root of trust.** Root of trust = the user's
  long-term keypair. Resolution order: (1) cached direct addresses, (2) **relay-reservation /
  rendezvous ("home relay") addresses**, (3) DHT as fallback. A fresh contact SHOULD have a
  non-DHT path (rendezvous/introduction) so it is not 100% dependent on a hostile public-DHT
  lookup.
- Closed/organizational deployments SHOULD use a **private DHT** (own protocol prefix) to
  shrink the Sybil surface.

## 4.3 Reachability ladder

Try in order; fall down only as needed:

1. **Direct** — IPv6, or IPv4 with port-forward/UPnP. No relay.
2. **Hole-punch** — DCUtR between two nodes (both online; always true for always-on boxes).
3. **Circuit relay** — a public-IP node relays; ciphertext-only, content-blind.

As IPv6 spreads, rungs 1–2 dominate and the relay role fades. Downtime is covered by
sender-retry (§2.6) and optional **peer buffering** (buddy nodes hold ciphertext during an
outage) — no central buffer required.

## 4.4 The mixnet (metadata privacy)

The `private` tier (the production default for all mail and every control MOTE, §4.6, §10.3) is a
**mixnet**: MOTEs travel as **Sphinx-format**, constant-length, onion-wrapped packets through a
sequence of **mix nodes**, mixed with Poisson delays and cover traffic in the **Loopix/Nym**
operational style. This section specifies it **normatively and by reference** — DMTAP does **not**
invent a mix format. It **profiles** two finalized designs and pins their parameters to §16.3:

- the **Sphinx** packet format (Danezis & Goldberg, *"Sphinx: A Compact and Provably Secure Mix
  Format,"* IEEE S&P 2009) for the on-wire onion packet (§4.4.1); and
- the **Loopix** anonymity system (Piotrowska et al., USENIX Security 2017) and its production
  descendant **Nym** for the operational design — stratified topology, Poisson mixing, loop/drop
  cover, and continuous-time mixing (§4.4.3, §4.4.5).

DMTAP's own contribution is confined to the **binding**: how mixes are discovered and keyed
against the existing DNS/KT trust (§4.4.2), how the tiers compose (§4.6, §10.3), and an honest
low-adoption model (§4.4.11), and the **anti-active-adversary hardening of §4.4.6–§4.4.10**
(replay caches, tagging resistance, Poisson mixing, loop-cover attack detection, entry guards +
operator-diversity, and fail-closed no-downgrade). Mix nodes are the same permissionless,
content-blind contributor
model as relays (the mix role is a capability of the node binary, §4); incentive and
Sybil-resistance are in §6.4 and §9.8. Email's asynchrony is what makes full-strength mixing
affordable: minutes of latency are acceptable for the `private` tier (§16.3).

Baseline guarantees (each detailed below): each mix peels exactly one layer so **no single mix
sees both sender and recipient**; mixes add **randomized (Poisson) delays** and reorder,
defeating timing correlation; nodes emit **loop + drop cover traffic** so an observer cannot tell
when real messages flow; **sealed sender** (§6.2) keeps the sender identity inside the payload,
never in the outer packet; and **size padding** to the bucket ladder (§4.4.1, §16.3) means length
leaks nothing.

### 4.4.1 Sphinx packet format (profiled; parameters pinned)

DMTAP uses the **Sphinx packet format** unchanged; this subsection states the profile a
conformant implementation MUST follow and pins every free parameter. A Sphinx packet is a
**fixed-length** structure `(α, β, γ, δ)`:

- **`α` — the header group element** (an elliptic-curve point). v0 uses **X25519 / Curve25519**
  (the KEM group of `suite = 0x01`, §16.7), so `α` is **32 bytes**. Each hop derives a shared
  secret by Diffie–Hellman between `α` and its **Sphinx mix public key** for the target epoch
  (§4.4.4), then **re-randomizes `α`** (multiplies by the derived blinding factor) before
  forwarding, so `α` is unlinkable hop-to-hop.
- **`β` — the encrypted routing information**, a fixed-length onion of per-hop routing commands
  (next-hop address, per-hop Poisson delay, and padding), each layer encrypted with a key derived
  from that hop's shared secret using a **stream cipher** (v0 ChaCha20, matching the suite AEAD
  primitive). `β` is padded so it stays **constant length at every hop** (this constant length is
  what makes Sphinx resistant to length-based tracing).
- **`γ` — the per-hop MAC** over `β`, a **Poly1305** tag (v0) keyed from the hop's shared secret;
  a hop that fails the MAC drops the packet (`ERR_MIX_PACKET_MALFORMED`, `0x0307`) — this is
  Sphinx's integrity guarantee against tagging attacks.
- **`δ` — the payload**, a **constant-length, padded body** transformed at each hop by a
  **wide-block pseudo-random permutation (PRP)** over the entire `δ` block — v0 pins **LIONESS**
  (Anderson–Biham, keyed from the hop's shared secret), exactly as classical Sphinx specifies —
  **not** a stream cipher, and **not** an "AEAD over `δ`". The wide-block property is
  **load-bearing for tagging resistance**: because a LIONESS block is a keyed permutation of the
  whole cell, **any** change to a single ciphertext bit of `δ` at one hop diffuses to the entire
  block on the next unwrap, so a corrupted payload becomes indistinguishable random noise and
  **cannot** be recognised downstream — the payload carries no adversary-recoverable structure to
  tag. (A stream-cipher or AEAD `δ` would be **XOR-affine/malleable**: a controlled bit-flip at
  entry would survive as a *correlatable* mark at the exit, defeating unlinkability. Sphinx's
  tagging resistance for the **payload** comes from this wide-block PRP, **not** from the header
  MAC `γ`, which protects only `β`.) v0 fixes `δ` to the **Sphinx cell size = 2 KiB** (§16.3), the
  constant-length bucket after padding.

**Pinned parameters (v0, §16.3):** path length **ν = 3 hops** (Standard) — but the header `β` is
**sized for the maximum supported path length `r_max = 5`** (the High-security hop count, §4.4.10)
and **zero-padded for shorter paths**, so a 3-hop and a 5-hop packet are **byte-for-byte the same
length** and the packet size **never leaks which profile** a sender uses. Also pinned: cell
payload **`δ` = 2 KiB**; per-hop delay **exp(mean 5 s)**; group **X25519**; **`β` stream cipher
ChaCha20**; per-hop header **MAC Poly1305** (over `β` only); **`δ` payload PRP LIONESS**
(wide-block, over the whole cell); KDF **BLAKE3** (suite hash). These are carried as capabilities
and are versioned with the protocol; a PQ variant is §4.4.12.

**The bucket ladder (reconciles §2.5 inline size with the 2 KiB cell).** A single Sphinx cell
carries **2 KiB** of payload, so a MOTE is not "one packet ≤ 64 KiB" — it is a **whole number of
2 KiB cells**. To keep size from leaking while still allowing multi-KiB inline/normal payloads
(§2.5, §6.5), a sender MUST pad the MOTE up to the next size on a **fixed bucket ladder** and then
fragment it into exactly `bucket / 2 KiB` Sphinx cells, all sent over independently-selected paths
(§4.4.3) and reassembled by the recipient. The v0 ladder (§16.3) is **{2 KiB, 8 KiB, 32 KiB,
64 KiB}** — i.e. **1, 4, 16, 32 cells**. Thus the `inline` file tier's "≤ 64 KiB after padding"
(§2.5) is the **top rung = 32 cells**, not one packet; only sizes on the ladder ever appear on the
wire, so an observer learns only which of four buckets a message fell into, never its true length.
A MOTE that would exceed the top inline rung is a `normal`/`large` file (§2.5) and its bulk travels
per §4.5, not as inline cells.

**Acks and replies.** A delivery `ack` (§2.6) travels the mixnet as its own small `system` MOTE
(one cell); implementations MAY additionally use Sphinx **Single-Use Reply Blocks (SURBs)** so a
recipient can reply/ack without learning the sender's location, and SURBs are the mechanism for
recipient-side loop cover (§4.4.5).

### 4.4.2 Mix directory (discovery + keys, bound to DNS/KT)

Senders need the mix fleet's **identities, addresses, Sphinx public keys, and layers** to build
paths — DMTAP distributes them by **reusing DNS + key transparency**, not a new PKI:

- **`MixNodeDescriptor`** (§18.5.2) — each mix publishes a signed descriptor: its identity key
  `node_ik` (an ordinary DMTAP identity, so operators are accountable and KT-auditable), its
  reachability, its **Sphinx mix public key(s) per epoch** (§4.4.4), and its **stratified layer**
  (§4.4.3).
- **`MixDirectory`** (§18.5.3) — a **directory authority** publishes a signed, versioned,
  hash-chained snapshot of the fleet for an epoch, and **appends its root to key transparency**
  (§3.5) exactly as the org directory does (§3.10.3). The authority is a DMTAP identity whose key
  is pinned via the domain's `_dmtap` anchor (§3.2, a `mix=` locator) — **the same discovery and
  pinning path as any name → key binding** (§3.3). The authority key SHOULD be threshold-held
  (§5.8.6) and the directory MAY be attested by a **set of authorities with a `> n/2` quorum**
  (§3.5.2(b)), so no single authority can unilaterally inject or suppress mixes.

Because the directory is KT-anchored, an authority that shows different fleets to different
clients (a split view over the mix set) is **detectable exactly like KT equivocation** (`0x0107`,
§3.5.2) — **but only to the same degree KT itself is, which depends on the KT profile in force.**
Under **v1-hardening KT** (log-type `0x02`, §3.5.2) a directory split view is gossip-detected and
quorum-bounded like any equivocation; under **v0-minimal KT** (log-type `0x01`) a single,
non-gossiped log can present a split view that is only tamper-evident *after the fact* (§6.6 item
6), so mix-directory equivocation is **deterred, not reliably detected**, in v0. Therefore, for
**high-risk** use while on v0, a deployment SHOULD (i) publish the `MixDirectory` under a **set of
authorities with a `> n/2` quorum** (§3.5.2(b)) rather than a single authority, and (ii) have
verifiers **OOB-pin** the authority key(s) (§3.4.1) rather than trust a single unaudited anchor.
This is the mixnet instance of the same v0/v1 caveat, and it compounds with the honest
single-operator launch-trust disclosure (§4.4.11): early on, one operator running both the fleet
*and* a single-log directory authority is a concentrated trust point, disclosed as such. The
directory **indexes; it does not forge**: each `MixNodeDescriptor` self-verifies
under its own `node_ik`, so a compromised authority can withhold or reorder mixes (a
denial/annoyance, detectable) but cannot make a sender encrypt to a key an honest mix does not
hold — the same "convenience enumeration of independently-verifiable bindings" discipline as the
GAL (§3.10.3). A directory not signed by the pinned authority is rejected
(`ERR_MIX_DIRECTORY_SIG_INVALID`, `0x030B`); an older-or-equal `version` is rejected (rollback
defense); a directory lacking a full stratified layer set makes path-building fail
(`ERR_MIX_PATH_UNBUILDABLE`, `0x030D`).

- **Directory freshness — freeze-attack defense (MUST).** Rollback defense (rejecting an
  *older-or-equal* `version`) stops an adversary *rewinding* a client, but it does **not** by
  itself stop an adversary *freezing* one: an on-path adversary (or a censoring authority) that
  simply **serves the last honest directory forever** presents a validly-signed, non-rolled-back
  snapshot while withholding every newer one. Left unbounded, a freeze pins the victim to a
  **stale fleet view** — it never learns of newly-joined, operator-diverse honest mixes (so its
  effective diversity and anonymity set stay artificially small and adversary-favourable) even
  though nothing it can see is *invalid*. This is the exact analogue of the KT **freeze attack**
  that STH freshness defends (§3.5.2(a), `0x0112`), and the mix directory — being KT-anchored
  (its root is appended to KT) — MUST inherit the same defense. A client therefore MUST treat a
  `MixDirectory` older than the **mix-directory freshness window** (§16.3, ≤ one mix-key epoch)
  as **stale**, MUST refresh it before building any `private`-tier path, and MUST **fail closed**
  (§4.4.9 — hold, never downgrade) if it cannot obtain a fresh directory — raising
  `ERR_MIX_DIRECTORY_STALE` (`0x0311`). Because the authority MUST publish a new directory each
  epoch and append its root to KT (above), a freeze is **detectable**: the authority's own KT log
  will show no fresh directory root within the window (a withholding signal, gossiped exactly as
  an unpublished KT entry is, §3.5.2(a)), and under v1-hardening KT it is attributable to the
  authority. This makes "withhold newer mixes" a **detected, fail-closed** event rather than a
  silent shrinking of the anonymity set, completing the trust-minimization triad for the
  directory authority (minimized §4.4.2 quorum, **detectable** here, fail-closed §4.4.9). In v0,
  as with all KT, a single-log freeze is only tamper-evident-after-the-fact (§6.6 item 6); the
  ≤-one-epoch key rotation (§4.4.4) still bounds the freeze window because a frozen directory's
  keys expire and path-building then fails closed regardless.

### 4.4.3 Path selection (3-hop stratified free-route)

- **Length: 3 hops** (§16.3) — entry, middle, exit — the Loopix/Nym choice: enough that no single
  mix links both ends, short enough to keep email-scale latency and bandwidth bounded (the
  anonymity ↔ latency/bandwidth tradeoff, §6.6 item 1).
- **Topology: stratified (layered), free-route within a layer.** The fleet is partitioned into
  **three layers** by `MixNodeDescriptor.layer` (0=entry/1=middle/2=exit); a path is built by
  drawing **one mix uniformly at random from each layer in order**, weighted by advertised
  capacity/reputation (§9.8). Rationale: a stratified topology (as analyzed for Loopix and
  deployed by Nym) gives a **well-defined anonymity-set analysis and predictable mixing** and
  spreads load evenly, versus an unconstrained free route whose anonymity is harder to reason
  about and whose load concentrates on popular nodes. Layer assignment is part of the directory,
  so all senders draw from the same partition.
- **Fresh path per packet.** A sender MUST select an **independent** path for **each Sphinx cell**
  (including the cells of a multi-cell MOTE, §4.4.1, and each cover packet, §4.4.5); paths are
  never reused across messages, so no persistent circuit exists to correlate.

### 4.4.4 Mix key rotation & epochs

- **Epochs.** Mix keys are **epoch-scoped**; the v0 epoch is **24 h** (a §16.3 parameter). The
  `MixDirectory.epoch` names the current epoch and each `MixKeyEntry` (§18.5.2) binds a Sphinx mix
  public key to an epoch and a `valid_until`.
- **Rotation with overlap.** A mix MUST generate a fresh Sphinx keypair each epoch and SHOULD
  advertise **both the current and the next** epoch key (an overlap window) so senders can
  pre-build paths across an epoch boundary without a gap. Old epoch private keys are **deleted at
  `valid_until`**, giving the mixnet **forward secrecy against later node compromise** (a seized
  mix cannot retroactively peel captured old packets).
- **Sender obligation.** A sender MUST encrypt each hop to the mix key of the **epoch that hop
  will process the packet in**, and MUST NOT build to an expired epoch key; a packet built to an
  expired/rotated key is dropped by the mix (`ERR_MIX_DESCRIPTOR_STALE`, `0x030C`). Clients
  refresh the `MixDirectory` at least once per epoch.

### 4.4.5 Cover traffic & Loopix loops (normative)

Cover traffic is **load-bearing, not optional** (§6.2, §6.4). Every `private`-tier node MUST emit,
independently of user activity, two Poisson streams (rate per §16.3, default mean 30 s/msg,
tunable — higher rate = more privacy, more bandwidth):

- **Loop cover** — a packet the node sends **through a full 3-hop path back to itself** (via a
  SURB, §4.4.1) at Poisson rate **λ_loop** (§16.3). Loops (a) give the node a steady *sending*
  stream indistinguishable from real traffic, and (b) act as **active-attack detection**: a node
  that stops receiving its own loops at the expected rate has evidence its traffic is being
  dropped/delayed (an `(n-1)`/flooding attack). The **full detection rule, threshold, and
  fail-closed response are normative in §4.4.7** — loops are the key lever that makes active
  attacks detectable-and-responded rather than silent.
- **Drop cover** — a packet addressed to a **random mix that discards it at the last hop**,
  providing link cover on the entry segment.
- **Recipient-side loop cover (normative, §6.4).** An always-on recipient node MUST **also**
  receive a steady loop-cover stream so that **real receipts are indistinguishable from cover** on
  its delivery link — closing the receipt-timing exposure of §6.4 item 2. Without this, an
  observer of the recipient's link learns *when* mail arrives even though it cannot read it.

Mixing delay is applied **per hop**: each mix independently holds each packet an exp(mean 5 s,
§16.3) time drawn from the per-hop delay command in `β` (§4.4.1) and forwards in re-randomized
order (continuous-time Poisson mixing, as Loopix specifies), so input and output streams cannot be
timing-correlated. Cover-packet admission is rate-bounded per node (§9.8) so cover cannot be
turned into a flooding vector.

### 4.4.6 Active-attack resistance: replay, tagging, unlinkability, Poisson mixing (normative)

These are the concrete mechanisms that make a **global *active* adversary** (inject/drop/delay,
§6.1) expensive and, where it acts, **detectable** — not an "honest limit," a defense. Each is
profiled from Sphinx/Loopix, made mandatory here.

- **Per-epoch replay cache at every mix (MUST).** Each mix MUST maintain a **replay cache** of the
  Sphinx per-hop **tag** — the value `H(shared_secret)` derived when it peels a packet (a unique,
  unlinkable-across-hops identifier) — for **every packet it has processed in the current mix-key
  epoch** (§4.4.4), and MUST **drop any packet whose tag is already present**
  (`ERR_MIX_REPLAY_DETECTED`, `0x030E`, DROP_SILENT). Because mix keys rotate per epoch and the old
  private key is **deleted at `valid_until`**, a captured packet is replayable only while the key
  it was built to is still usable. The cache MUST therefore cover the **entire lifetime of every
  mix key currently usable at that node — the current epoch AND the overlap window of the next
  epoch's pre-published key (§4.4.4), plus the clock-skew window** (§16.3) — and a per-key cache
  entry may be dropped only once **that** key's `valid_until` has passed. A **hard flush at the
  epoch boundary is forbidden**: because a mix advertises current+next keys so senders can
  pre-build across the boundary, a packet built to a still-valid key must remain replay-protected
  until that specific key expires, not until the nominal epoch ticks over. This stays bounded
  memory (two epochs at most, no permanent log). This is the primary defence against **replay-based
  correlation and (n−1) replay flooding**: an adversary cannot re-inject a target's packet to trace
  it, because the second copy is dropped at the first honest hop.
- **Tagging-attack resistance (MUST) — header AND payload.** Two distinct mechanisms, one per
  packet part, and both are required: **(header)** each hop verifies the per-hop **MAC `γ` over
  `β`** before any processing (§4.4.1); any adversarial bit-flip of the routing header fails the
  MAC and the packet is dropped (`0x0307`). **(payload)** `δ` is transformed by a **wide-block PRP
  (LIONESS)** at every hop (§4.4.1), so any bit-flip of the payload diffuses across the **entire
  cell** on the next unwrap and becomes unrecognisable random noise downstream — the payload
  carries no malleable, correlatable structure to mark. The header MAC alone does **not** protect
  `δ`; the wide-block PRP is what makes **payload** tagging fail. Together an active adversary
  **cannot mark ("tag") a packet at the entry and recognise the mark at the exit** on either part
  — Sphinx's provable integrity guarantee, and the reason DMTAP uses Sphinx (with a wide-block
  payload) rather than a plain stream-cipher/AEAD layered-encryption onion.
- **Bitwise unlinkability (inherent).** `α` is re-randomised and `β`/`δ` fully re-encrypted at
  every hop (§4.4.1), so a mix's input and output packets share **no correlatable bits**. An
  adversary observing both sides of an honest mix is reduced to **timing** correlation only, which
  the next mechanism defeats.
- **Poisson (exponential, memoryless) mixing (MUST).** Each hop delays each packet by an
  **independent exponential** delay (mean per §16.3), never a fixed or bounded delay. By the
  memoryless property, a packet's **output time is independent of its input time** given the
  exponential hold, so even an adversary that **injects or selectively delays** input cannot use
  inter-arrival timing to correlate a mix's input and output streams (continuous-time mixing, as
  Loopix specifies). Fixed or uniform delays would leak ordering; the exponential distribution is
  **required**, not a tuning choice.

### 4.4.7 Loop cover as active-attack detection & fail-closed response (normative) — the key lever

Cover traffic (§4.4.5) is not only obfuscation: **loop cover is an active-attack *detector***, the
mechanism that converts drop/delay/flooding attacks from **undetectable** to
**detected-and-responded**. This is the single most important lever against an active adversary.

- **Client loops and mix loops (MUST).** Every `private`-tier node emits **client loops** — a
  Sphinx packet sent through a **full 3-hop path back to itself** via a Single-Use Reply Block
  (SURB, §4.4.1) — at Poisson rate **λ_loop** (§16.3); every mix likewise emits **mix loops**
  through the layers back to itself. A node knows exactly which loops it launched, over which
  paths, and the delay budget within which each should return.
- **Detection rule (MUST).** A node MUST track, over a sliding window, the **fraction of its loops
  that return within their expected delay budget** and their **latency distribution**. If the
  return fraction drops below the **loop-loss threshold** (§16.3) or latencies inflate beyond what
  the exponential budget explains, the node MUST **infer an active drop/delay attack on its paths**
  and raise `ERR_MIX_ACTIVE_ATTACK_SUSPECTED` (`0x030F`). Because loops are **Sphinx packets
  indistinguishable from real traffic** on the same paths, an adversary **cannot** selectively
  drop real messages while sparing loops — suppressing traffic on a path necessarily suppresses
  that path's loops, which is exactly what the node measures.
- **Response — rotate, alert, and FAIL CLOSED (MUST; never silently continue).** On an inferred
  active attack the node MUST: (1) **rotate away** from the implicated mixes and **entry guards**
  (§4.4.8) and rebuild over alternate, operator-diverse paths; (2) raise a **`HALT_ALERT`**
  (§21.2) to the user — the private tier is under active attack; and (3) **fail closed for the
  `private` tier** — it MUST NOT silently fall back to `fast` or to a shorter/less-diverse path to
  "get the message through" (that is precisely the adversary's goal, §4.4.9). Silent continuation
  under detected active attack is prohibited.
- **(n−1) / flooding defence (MUST).** To isolate a target packet an adversary must flush all
  other traffic from a mix (the classic n−1 attack). The **steady loop + drop cover from every
  honest node** (§4.4.5) means a mix is **never empty of indistinguishable cover**, so the
  adversary must inject vastly more traffic to dominate a mix (raising cost sharply), and the
  honest nodes whose loops then fail to return **detect** the flush (above). Cover admission is
  **rate-bounded per node** (§9.8) so cover itself cannot be turned into the flood. Loops thus both
  **raise the cost** of and **expose** flooding.

### 4.4.8 Entry guards, operator diversity & mix Sybil resistance (normative)

Long-term **statistical-disclosure / intersection** attacks — an adversary who controls some mixes
correlating a persistent user across many messages — are bounded **structurally**, not merely
disclosed.

- **Entry guards (MUST).** A sender MUST NOT choose a fresh entry mix per packet; instead it pins a
  **small, rotating entry-guard set** of **G = 2** entry-layer mixes (§16.3), reuses them across
  packets, and rotates only every **guard-rotation period** (§16.3, default **30 days**) —
  Tor-style. Rationale: with a fresh random entry per packet, an adversary owning a fraction *f* of
  entry mixes appears on *some* of a victim's paths eventually and, across many messages, mounts an
  intersection attack; pinning guards means the victim is **either persistently clear of the
  adversary's entries (probability ≈ (1−f)^G) or persistently on a known guard** — bounding
  long-term exposure to a one-time draw instead of a cumulative certainty (§6.6, §11.3).
  **Why G = 2 (not Tor's single primary guard).** Tor pins **one** primary guard (with sampled
  backups) to *minimise* the chance of ever touching an adversarial guard; DMTAP pins **two**
  because a mail node's entry mix is also its **cover-loop origin** and reachability anchor, and a
  single guard is a single availability/rotation point whose outage would either stall the node or
  force an unplanned re-draw (itself an exposure event). G = 2 trades a marginally higher
  one-time draw probability (`1−(1−f)²` vs `1−(1−f)¹` of touching *an* adversarial entry) for
  resilience and steadier cover, while still bounding the *long-term* intersection to that
  one-time draw. A deployment MAY instead follow the Tor **1-primary + sampled-backup** shape
  under the same guard-rotation period; both are conformant, the invariant being *persistent* (not
  per-packet-fresh) entry selection. The value is a §16.3 profile parameter.
  **Disclosed guard observation (NIT).** A pinned entry guard, if adversarial, **necessarily sees**
  each of the sender's packets it carries — including the **bucket size** (which of the {2,8,32,64}
  KiB rungs, i.e. the exact **cell count**, §4.4.1) and the **send time**. This is inherent to
  being the first hop and is **not** hidden by the mixnet (padding hides *true* length, not which
  *rung*; cover traffic blurs *rate*, not that *this guard* is used). Constant-rate cover
  (High-security, §4.4.10) is the mitigation that most flattens the send-time signal; the residual
  — an adversarial guard learns "this sender sent a bucket-*k* message now" — is a disclosed
  boundary, bounded (not eliminated) by guards being few and rotated.
- **Operator diversity in path selection (MUST) — and only *attested* operators count.** A 3-hop
  path MUST traverse mixes under **three disjoint operators** — no two hops may share a
  `MixNodeDescriptor.operator` (§18.5.2) — mirroring the disjoint-operator discipline of KT log
  sets (§3.5.2(b)) and S/Kademlia disjoint paths (§4.2). **Crucially, a mix contributes a distinct
  operator to this rule *only* if its `operator` claim is backed by a valid `_dmtap-mix` operator
  attestation** under that operator's domain (below). A mix whose `operator` is **absent** or
  **un-attested** MUST **NOT** be treated as its own separate operator for diversity purposes — it
  is either excluded from path selection or counted as an **unknown/shared** operator, never as
  fresh diversity. This is essential: if an absent/self-asserted `operator` counted as diverse, a
  **single** adversary could publish *N* mixes each self-claiming a different operator and defeat
  the whole rule — collapsing the ≈ *a*² fully-compromised-path bound back to ≈ *a* (one adversary
  faking *N* operators). Requiring attestation to count keeps the bound at ≈ *a*² for an adversary
  operating a fraction *a* of the **attested-diverse** fleet, and denies any single operator a
  whole-path view. (While the fleet is single-operator at launch, diversity is necessarily relaxed
  to that operator and **disclosed** per §4.4.11; the rule binds as independent, *attested*
  operators join.)
- **Mix Sybil resistance — attested identity, NO anonymous token (MUST).** Mix admission MUST NOT
  use an anonymous token (that would let one party mint unlimited Sybil mixes). A mix's `node_ik`
  is a **KT-auditable DMTAP identity** (§4.4.2), and its **operator control MUST be attested by a
  DNS/KT record under the operator's domain** — a `_dmtap-mix` attestation directly analogous to
  the gateway attestation `_dmtap-gw` (§7.2a) — so every mix is bound to an **accountable,
  rate-limited real-world operator**; the directory authority (§4.4.2) admits only attested mixes,
  and — per the operator-diversity rule above — an **un-attested `operator` claim confers no
  diversity** (a mix counts as a distinct operator toward the disjoint-operator requirement **iff**
  its `_dmtap-mix` attestation validates), which is what stops one adversary minting *N* fake
  operators to defeat the a² bound.
  Operators **SHOULD post stake/bond** as gateway operators do (§9.6), making Sybil fleets costly,
  and path selection **weights mixes by measured reliability/reputation**. **The exact weight
  function is an implementation choice, not pinned (NIT):** the "measurement hook" is a concrete
  input — each node's loop-return statistics (§4.4.7) feed a reliability score (§9.8) that
  **down-weights mixes that drop or delay** — but the mapping from score to selection probability
  is left to the implementation, so the path distribution is **non-deterministic across
  implementations**; only the hard constraints (one-per-layer, ≥ the in-force profile's disjoint
  **attested** operators, current-epoch keys) are normative. A misbehaving or Sybil mix loses path
  share under any conforming weighting.

### 4.4.9 Fail-closed: minimum viable path, no silent downgrade (normative)

A global active adversary can **DoS mixes specifically to force `private → fast`** — collapsing a
metadata-private message onto a correlatable tier. DMTAP treats this as an attack and **refuses**.

- **The in-force profile's bar IS the fail-closed floor (MUST).** The minimum-viable-path bar is
  **the bar of the profile actually in force for this message**, not a fixed 3-hop base. Under the
  Standard profile that is **≥ 3 hops, one per stratified layer, under ≥ 3 disjoint operators**
  (§4.4.8); under the **High-security profile it is ≥ 5 hops and ≥ 5 disjoint operators**
  (§4.4.10) — and that higher bar is **itself the floor**. A high-security packet that can only
  build a shorter or less-diverse path (e.g. only a 3-hop / 3-operator path is currently buildable)
  MUST **fail closed exactly as if no path existed** — it MUST **NOT** silently satisfy the lesser
  base bar, because delivering a message a user marked high-security over a Standard-strength path
  is precisely the covert downgrade an adversary DoSes the fleet to force. All paths use
  **current-epoch** mix keys (§4.4.4). While the fleet is single-operator the operator-diversity
  component is relaxed **and disclosed** (§4.4.11), but the **hop count and per-layer requirement
  are never relaxed**, at whichever profile is in force.
- **Fail closed, never auto-downgrade — across tiers AND across profiles (MUST).** If no path
  meeting the **in-force profile's** bar is buildable from the current `MixDirectory` (too many
  mixes down/attacked, or diversity unmet), the sender MUST **NOT** silently route the MOTE over
  `fast`, over fewer hops, over fewer operators, **or over a lower profile's bar**. It MUST **hold
  the MOTE in its retry queue** (§4.7) and retry, surfacing `ERR_PRIVATE_TIER_DOWNGRADE_REFUSED`
  (`0x0310`) to the user if the condition persists to the retry deadline — the same fail-closed
  stance as unreachable KT (§3.3) and unreachable auth status (§13.4). This error covers **both**
  the tier downgrade (`private → fast`) **and** the profile downgrade (High-security → Standard):
  either is a refused silent demotion. Downgrading `private → fast`, **or High-security →
  Standard**, is **only ever a deliberate, user-surfaced choice** (§4.6, §4.4.10), **never** an
  automatic reaction to mix unavailability. This closes the "DoS the mixnet to strip anonymity"
  vector: an adversary can **delay** delivery but cannot silently **demote** it — not to a weaker
  tier and not to a weaker profile.
- **Triggered by the detector.** The condition is raised either by an empty minimum-viable-path
  build (§4.4.3) or by the loop-cover active-attack inference (§4.4.7); both lead to
  hold-rotate-alert, never to a silent weaker tier.

### 4.4.10 High-security profile (normative, user-selectable)

The Anonymity Trilemma (§6.6, Das et al. 2018) proves strong anonymity **must** cost latency
and/or bandwidth; DMTAP therefore exposes that tradeoff as a **selectable profile**, not a fixed
point — the concrete lever a high-risk user pulls. Two profiles are normative (parameters §16.3):

| Profile | Hops | Per-hop delay | Cover / loop rate | Entry guards | Operator diversity | Intended use |
|---------|:----:|---------------|-------------------|:------------:|--------------------|--------------|
| **Standard** (default) | 3 | exp, mean 5 s | Poisson, mean 30 s; λ_loop mean 30 s | 2 guards / 30 d | 3 disjoint (all hops) | all mail by default |
| **High-security** | **5** | exp, **mean 30 s** | **constant-rate** (Poisson) mean **5 s**; λ_loop mean **5 s** | **3 guards / 7 d** | **5 disjoint** (all hops) | high-risk users / messages |

The high-security profile trades minutes → tens-of-minutes latency and higher bandwidth for a
larger effective anonymity set, **constant-rate cover** (a flat, activity-independent traffic
envelope that yields nothing to traffic analysis), longer memoryless delays, faster loop-based
detection, and stricter guard/diversity. It is **negotiated as a capability** (§10.2); both parties
MUST support the profile in force, and a recipient **MAY require** it for its inbound `private`
traffic. Profiles are the lever that lets a deployment climb toward the trilemma bound; the
irreducible residual **after** the maximal profile is stated honestly in §6.6.

**What more hops do — and do not — buy (honest scope).** Raising the hop count (5 vs 3) and the
per-hop delay hardens DMTAP against **timing correlation by a network observer**: more independent
memoryless hops (§4.4.6) mean an observer watching the links must defeat more layers of Poisson
reordering to link a mix's input to its output, so the passive-correlation advantage falls toward
the chance floor as hops and cover grow (measured, §6.10). It does **not**, however, reduce the
probability that the **entry and exit mixes are both adversarial** — the one placement from which a
*colluding* pair correlates a flow regardless of how many honest hops sit between them. That
probability is bounded **only** by entry guards + attested operator diversity (§4.4.8), which hold
it near ≈ *a*² for an adversary operating a fraction *a* of the attested-diverse fleet; adding
middle hops leaves it unchanged. Hop count and operator diversity therefore defend **different**
attacks and neither substitutes for the other — the High-security profile raises **both** (5 hops
*and* 5 disjoint operators / 3 tighter guards) precisely because each closes a gap the other
cannot. This distinction is stated as measured evidence in §6.10.

### 4.4.11 Honest low-adoption model (disclosed, §6.6 style)

A mixnet's privacy **is** its anonymity set, and DMTAP does **not** overclaim a day-one one:

- **Launch = an operator-run mix fleet.** At launch the three stratified layers are staffed by a
  **small fleet the launching operator runs** (the same operator that runs gateways/relays, §12).
  With few nodes under one operator, the guarantee is closer to **Tor-with-few-relays / a trusted
  VPN with mixing** than to a strong global mixnet: it defeats a **network-local passive
  observer** and hides the graph from **any single mix**, but a **small set of colluding
  first-and-last mixes, or the operator itself if it ran an entire path, could correlate** — this
  is disclosed, not hidden.
- **Strengthening with the network.** Privacy **increases as independent operators contribute
  mixes** and the layers come under **disjoint operational control** (the directory SHOULD prefer
  layer assignments that spread operators, analogous to S/Kademlia disjoint paths, §4.2, and to KT
  logs under distinct operators, §3.5.2(b)). The design target is many-operator layers where no
  single party controls a whole path; the launch state is the honest floor, not the end state.
- **No false anonymity-set claims.** Clients MUST NOT present the `private` tier as "anonymous"
  in absolute terms while the fleet is small; the in-product disclosure follows §6.6. This is the
  mixnet-specific instance of the weakest-link and global-active-adversary boundaries (§6.6 items
  1 and 5, §11.3).

### 4.4.12 Post-quantum Sphinx (tracked frontier, agility hook)

v0 Sphinx uses **X25519** for `α` (§4.4.1), so the **onion packet is not post-quantum**: a
"harvest-now-decrypt-later" adversary who records `private`-tier packets and later has a quantum
computer could recover **routing metadata** (not content — content is separately MLS/HPKE-sealed
and PQ-migratable, §5.1). **This is not "just metadata":** recovering per-hop routing across a
corpus of recorded traffic is **retroactive social-graph deanonymization** — a future quantum
adversary could reconstruct *who communicated with whom, when* over all `private` traffic it
recorded today, which is precisely the graph the mixnet exists to hide. The exposure is bounded to
the *routing layer* (content stays sealed) but is a serious, disclosed harvest-now risk, not a
cosmetic one. This is an **openly tracked frontier**: standardized PQ mix-packet
formats (lattice-based / hybrid Sphinx constructions) are **active research, not yet finalized**
(§11.3), and DMTAP does **not** invent one.

The **agility seam** is already in place, mirroring the crypto-suite mechanism (§1.1): the
`MixNodeDescriptor` and each `MixKeyEntry` carry a **`suite`** (§18.5.2), and the **Mix Parameters
registry** (§21.23) is where a future PQ-Sphinx packet format + its group/KEM, delay and MAC
primitives are registered. When a PQ mix format is standardized it is added as a **new mix suite**,
advertised in descriptors, negotiated via capability announcements (§10.2), and adopted
dual-stack — old and new packet formats coexisting until the classical format is retired, exactly
as a new crypto suite spreads (§21.25). Classical X25519 Sphinx is the v0 baseline; PQ-Sphinx is
disclosed as the frontier, not silently deferred.

## 4.5 Bulk / file transfer

Large blobs (§5.5) MUST NOT traverse the mixnet (impractical bandwidth/latency). Instead:

- The **control MOTE** (`file_offer`: manifest + key) travels the **private** tier.
- The **chunks** transfer **direct** (fast tier), content-encrypted, **swarmed** from any
  holder (BitTorrent-style), resumable per chunk.

Honest tradeoff: the *fact and size* of a bulk transfer between two nodes is observable
(direct), though the content is encrypted and the control message is private. See §6.

## 4.6 Privacy tiers

| Tier | Path | Latency | Metadata privacy | Default for |
|------|------|---------|------------------|-------------|
| `private` | mixnet + cover traffic | minutes | strong (global passive) — quantified, see §6.4/§4.4.11 | mail, all control messages |
| `fast` | direct / low-hop mesh | sub-second | content only; graph observable | live chat (both online), bulk chunks |

Default is `private`. `fast` is opt-in per conversation/message. Choosing `fast` is itself
metadata; clients SHOULD keep `private` as the standing default and cover it with cover
traffic (§6).

## 4.7 Delivery state machine (sender)

```
QUEUED → SEALED (sealed sender + onion, §6) → IN_FLIGHT (mixnet/direct)
       → ACKED (done)  |  RETRY (backoff)  |  EXPIRED (drop, notify user)
```

Durability lives entirely in this sender-side queue. The middle is stateless.

## 4.8 Local, isolated, and delay-tolerant networks

DMTAP works in **remote environments with their own networks** — often *more easily than
email*, because email needs an SMTP/IMAP server + MX + DNS, whereas two DMTAP nodes on the same
network need no infrastructure at all.

- **Local discovery (mDNS).** Nodes on the same LAN discover each other via libp2p **mDNS**
  (`_p2p._udp.local`) with **zero configuration, no internet, and no central server**. A ship,
  a remote site, an air-gapped office, or a home LAN can run a fully functional local DMTAP
  mesh with nothing but the nodes. Private networks use a fingerprinted service name so they
  do not cross-discover.
- **Scope of the "easier than email" claim.** True specifically for the **local / same-network
  / isolated** case. Across the open internet, DMTAP still depends on the DHT/relays (§4.2–4.3),
  which have their own fragility.
- **Delay-tolerant store-and-forward is a DMTAP layer, not a libp2p feature.** libp2p is
  connection-oriented and provides no bundle/epidemic DTN routing. DMTAP REQUIRES its own
  store-and-forward: the sender-side retry queue (§4.7), **peer buffering**, and
  **sync-on-reconnect** of the device-cluster CRDT (§5.6). This is what lets an intermittently
  connected site queue locally and reconcile with the wider mesh when connectivity returns.
- **Radio transports (Bluetooth / Wi-Fi Direct / LoRa) are out of scope for v0.** libp2p ships
  no such transport (Briar's offline radio transports are its own Bramble code, not libp2p).
  Supporting them would mean writing a custom libp2p `Transport` — flagged as future work, not
  a v0 claim.
