# 16. Numeric Parameters (v0)

Interoperability requires agreed numbers, not qualitative "hours"/"minutes." This appendix fixes
the v0 defaults. Each is a **parameter** an implementation MAY tune within stated bounds, but the
defaults here are what a conformance run (§10) checks. All are versioned with the protocol.

## 16.1 Time, replay & clocks

| Parameter | Default | Notes |
|-----------|---------|-------|
| Clock-skew tolerance | ±120 s | max accepted difference between `ts` and receiver clock |
| Challenge/nonce validity window | 120 s | server-issued single-use nonce for auth (§13) & anti-abuse |
| Replay cache retention | ≥ 300 s | seen-`id` / seen-nonce cache MUST cover the skew+validity window |
| MOTE `expires` max | 1 year | cap on requested client-side expiry |
| Sender retry deadline | 72 h | after which an undelivered MOTE fails to the user (§2.6) |
| Sender retry backoff | exp, base 30 s, cap 1 h | exponential with jitter |

Nodes MUST NOT rely on synchronized clocks for correctness — only for the skew-bounded windows
above and for ordering hints.

## 16.2 Naming, KT & DHT

| Parameter | Default | Notes |
|-----------|---------|-------|
| Key-name length | 8 words (80 bits) | §3.9.1; 12 words (128 bits) for adversary-proof mode |
| Key-name wordlist size | 1024 | language-agnostic; +1 checksum word |
| KT signed-tree-head poll | ≤ 6 h | client re-checks its own entry (self-monitoring) |
| KT gossip interval (v1) | ≤ 1 h | STH gossip for equivocation detection (§3.5.2(a)) |
| KT maximum merge delay (v1) | ≤ 24 h | a `0x02` log MUST issue a new STH, and include every accepted entry, within this bound (§3.5.2(a)) |
| KT STH freshness window (v1) | ≤ 24 h | an STH older than this is stale (freeze-attack defense, `0x0112`); MUST be refreshed before the view is trusted (§3.5.2(a)) |
| KT log-set consistency quorum (v1) | > n/2 (⌈(n+1)/2⌉ of the pinned log set) | a `name → ik` binding is accepted only on a strict-majority quorum of logs, so a minority cannot forge or suppress it (§3.5.2(b); mirrors the §16.8 committer roster quorum) |
| DHT location-record TTL | 2 h | signed; republish before expiry (§4.2) |
| DHT republish interval | 45 min | < TTL, with jitter |
| DHT lookup redundancy (K) | 20 | store at K closest; S/Kademlia disjoint paths ≥ 3 |
| Location seq-number | monotonic u64 | rollback defense; reject older-or-equal |
| Max names per Identity (soft) | 32 | recommended cap on `Identity.names` aliases (§3.9.4, §3.11.3) to bound Identity size; tunable by policy, not a security gate |

## 16.3 Mixnet & privacy

| Parameter | Default | Notes |
|-----------|---------|-------|
| Mix path length | 3 hops | Sphinx onion, stratified (§4.4.1, §4.4.3) |
| Per-hop mix delay | exp, mean 5 s | Poisson mixing; `private` tier (§4.4.5) |
| Cover-traffic rate | **constant-rate, 30 s/cell (always-on nodes — MUST)**; Poisson mean 30 s/msg (battery/metered devices only) | loop + drop + recipient-side loop (§4.4.5). Constant-rate ≈ **5.6 MB/day** — negligible for a mains-powered node, and it makes the traffic envelope **activity-independent** rather than merely activity-blurred, which is what defeats a patient long-horizon observer |
| Sphinx cell size (`δ`) | 2 KiB | Sphinx constant-length payload cell after padding (§4.4.1) |
| Payload bucket ladder | **{8, 64} KiB = {4, 32} cells** | a MOTE is padded up to the next rung, then fragmented into that many 2 KiB cells (§4.4.1); only ladder sizes appear on the wire. Two rungs only — an observer learns ≤ 1 bit of size per message. The 8 KiB floor is set by the PQ envelope (ML-DSA-65 sig ~3.3 KB + ML-KEM-768 ct ~1.1 KB, §1.1), which cannot fit a 2 KiB rung |
| Multi-cell reassembly timeout | ≤ 15 min (≈ 3× the `private`-tier delivery latency budget) | a partial multi-cell MOTE held in the bounded reassembly cache is discarded if not completed within this window (§4.4.1 fragment reliability); bounds recipient memory against half-MOTE flooding |
| Fragment-recovery method | per-cell SURB-ARQ **or** FEC (`n > k` erasure code) | sender's choice, capability-negotiated (§10.2); recovers missing cells at the lost-fraction cost, never full re-send (§4.4.1). Retransmitted/parity cells are ordinary constant-length Sphinx cells |
| Mix key epoch | 24 h | Sphinx mix-key rotation; advertise current+next, delete old key at `valid_until` (§4.4.4) |
| Mix-directory freshness window | ≤ 1 mix-key epoch (24 h) | a `MixDirectory` older than this is stale (freeze-attack defense, `0x0311`); MUST be refreshed before building a `private` path, else fail closed (§4.4.2, §4.4.9). Mirrors the KT STH freshness window (§16.2) |
| Sphinx group / `β` stream / header MAC (v0) | X25519 / ChaCha20 / Poly1305 | header DH group, `β` stream cipher, per-hop header MAC over `β` (§4.4.1); PQ variant §4.4.12 |
| Sphinx `δ` payload transform (v0) | LIONESS (wide-block PRP) | keyed permutation over the whole 2 KiB cell — payload tagging-resistance; NOT a stream cipher / AEAD (§4.4.1, §4.4.6) |
| Mix replay-cache window | lifetime of every still-usable mix key + skew (current epoch + next-key overlap, ≈ up to 2 epochs + 120 s) | per-mix Sphinx-tag replay cache; retain until each key's `valid_until` passes — **no hard flush at the epoch boundary** while the overlap key is still usable (§4.4.6, `0x030E`) |
| Loop-cover rate (λ_loop) | Poisson, mean 30 s | client + mix loops for active-attack detection (§4.4.7) |
| Loop-loss detection threshold | > 20% loss (sliding window) or latency ≫ delay budget | infer active drop/delay attack → `0x030F`, rotate + `HALT_ALERT` + fail-closed (§4.4.7). Note: **sub-threshold** selective dropping (< 20% loss) is **bounded, not eliminated** — an adversary dropping a small fraction stays under detection but also achieves little; the **High-security profile's** faster loop rate (mean 5 s, §4.4.10) **tightens** this floor (detects smaller/faster loss) |
| Entry-guard set size (G) | 2 | pinned entry-layer mixes per sender (§4.4.8) |
| **Guard sample size** | 20 attested, ASN-disjoint entry mixes (Standard); **as many as exist, floor 3** (Bootstrap, §4.4.10a) | the **persistent sampled set** guards are drawn from (§4.4.8). Chosen ONCE; rotation moves active guards *within* it and MUST NOT re-sample from the fleet — otherwise repeated independent draws turn the `(1−f)^G` bound into `(1−f)^(G·r)` and guards buy nothing over a decade. A Bootstrap-sized sample bounds nothing meaningful, which is why that profile carries no anonymity claim |
| Guard-sample refresh | only on exhaustion or explicit owner re-sample | a re-sample is a disclosed exposure event, not routine hygiene (§4.4.8) |
| Entry-guard rotation period | 30 days | Tor-style rotation **inside the sample**; intersection-attack bound (§4.4.8) |
| Path operator-diversity | ≥ 3 disjoint operators (one per hop) | no two hops share `operator` (§4.4.8); relaxed only while single-operator (§4.4.11) |
| **Path ASN-diversity** | ≥ 3 disjoint announced BGP origin ASNs | MUST; domains are cheap and attestation proves accountability, not independence — ASN is the diversity axis rented capacity cannot cheaply buy (§4.4.8). Jurisdiction-disjointness SHOULD additionally hold |
| Minimum viable `private` path | 3 hops, 1/layer, current-epoch keys | below this the sender fails closed, never downgrades (§4.4.9, `0x0310`) |
| **Bootstrap profile** | 3 hops; **guard sample = all attested entry mixes, floor 3**; **best-effort ASN diversity ≥ 2**; other parameters as Standard | for a network too small to satisfy Standard (§4.4.10a). **No anonymity claim.** MUST be user-visible, MUST auto-upgrade to Standard once satisfiable, MUST NOT be fallen back to, and **ratchets per contact** like the §1.3 suite high-water-mark. Composing §16.3's 20-mix guard sample with the ≥ 3-ASN path rule would otherwise make a young network fail closed on **every** message — the profile exists so that "too small" degrades honestly instead of silently or fatally |
| Bootstrap → Standard threshold | guard sample reaches **20** attested ASN-disjoint entry mixes **and** ≥ 3 disjoint operator ASNs are path-buildable | evaluated per mix-key epoch from the client's own derived fleet view (§4.4.2); published human-readably as network status (§14.6.3) |
| High-security profile | 5 hops, exp mean 30 s/hop, constant-rate cover mean 5 s, 3 guards / 7 d, 5 disjoint operators | user-selectable maximal anonymity (§4.4.10); capability-negotiated (§10.2) |
| `private`-tier end-to-end latency | seconds–minutes (Standard); tens of minutes (High-security) | consequence of mixing (§6, §4.4.10) |

## 16.4 File tiers & transfer

**Size / privacy sub-tiers** (metadata-privacy axis, §6.5 — mixnet vs. bulk):

| Parameter | Default | Notes |
|-----------|---------|-------|
| Inline attachment threshold | ≤ 64 KiB | rides the message (§2.5); = top bucket-ladder rung (32 Sphinx cells, §16.3). NOT larger: a bigger inline cap forces a MOTE above the top bucket → cannot ride the `private` mixnet (§5.5.1) |
| Normal-file threshold | ≤ 4 MiB (≤ 4 chunks) | chunks via mixnet — full privacy (§6.5) |
| Chunk size | 1 MiB | fixed; content-addressed over **ciphertext** (§5.5, §18.9.5) |
| Large-file bulk | > 4 MiB | fast/onion bulk path — weaker privacy (§6.5) |
| Swarm parallelism | ≤ 8 sources | per-file concurrent fetch; bounds swarm-poisoning wasted bandwidth (§5.5.3) |

**Delivery / durability tiers** (durability axis, §5.5.1–§5.5.5 — orthogonal to the privacy axis
above; inline/push/pull governs durability, mixnet/bulk governs metadata privacy):

| Parameter | Default | Notes |
|-----------|---------|-------|
| Inline tier cap (durable-by-delivery) | ≤ 64 KiB | bytes in the sealed MOTE (`Attachment.inline`); = the inline attachment threshold above (§5.5.1) |
| Attached tier cap (pushed, durable recipient copy) | ≤ 25 MiB | chunks **pushed** into the recipient's store on delivery — survives the sender dropping (§5.5.1) |
| Referenced tier (pull-on-demand) | > 25 MiB | `ManifestRef` + key in the MOTE; chunks pulled from a holder; MUST carry a `durability` class (§5.5.2) |
| Auto-pull-to-durable threshold | ≤ 256 MiB | a client SHOULD auto-pull-and-pin a Referenced file below this on receipt, converting origin-hold → recipient-pinned (§5.5.2) |
| Default `cluster-replicated` N | 3 | replicas across a box-cluster (§5.6, §14); tolerates N−1 holder loss (§5.5.2) |
| Default `pinned(term)` retention | 90 days | minimum paid-pin retention term; renew before expiry (mirrors inactive-account purge §16.6); after it the host MAY GC (`0x080B`, §5.5.4) |
| Inbound spool cap (per unproven sender) | ≤ 64 MiB aggregate | a pushed Inline/Attached file exceeding this for that sender is refused fail-closed (`0x080C`, storage-DoS / spool-fill defense, §5.5.5); tunable by operator policy, generous on self-host |

## 16.5 Anti-abuse

| Parameter | Default | Notes |
|-----------|---------|-------|
| **Cold-sender sequential work (VDF)** | Wesolowski/Pietrzak VDF, delay ≈ **3 s** single-core, adaptive | **SHOULD** — the preferred cold-contact cost where both parties support it (§9.4.1). Sequential ⇒ rented parallel compute buys ≈ nothing (≈10× spread across all real hardware, vs ≈1000× for a parallelizable puzzle), so the cost gradient finally runs *against* the spam farm instead of against the phone. **Not the interoperable floor:** VDFs need a group of unknown order (RSA trusted setup, or less-mature class groups) and have no IETF standard, so a recipient MUST NOT require one as the only acceptable proof |
| VDF verification cost | milliseconds, independent of delay | **asymmetric by construction** — this is why the VDF path needs no verification budget and no defer-without-verifying escape hatch, unlike Argon2id below (§9.4.1) |
| VDF puzzle scope | `id ‖ recipient ‖ nonce(epoch) ‖ sender_key` | `sender_key` binding per §9.2a — a stolen proof is worthless under any other ephemeral key |
| **Zero-relationship delivery floor (`N_floor`)** | **≥ 5 cold MOTEs / sender-key / day** | MUST (§9.7a). A stranger with only a keypair and a valid work proof — **VDF *or* memory-hard PoW**, recipient's preference but both acceptable — always reaches the **requests area**, never the inbox. Without this floor the §3.13 key-name promise is false: a sovereign identity would be nameable, reachable, verifiable, and silently undeliverable |
| Cold-sender PoW | memory-hard (Argon2id), adaptive | **the interoperable MUST floor** a recipient has to accept (§9.4, §9.4.1) — the weaker mechanism everyone can implement today, which is precisely what makes it the floor; preference for VDF is a SHOULD layered above it |
| PoW puzzle scope | `id ‖ recipient ‖ nonce(epoch)` | fresh epoch nonce to prevent precompute |
| Memory-hard PoW verification budget | bounded per window **per delivering connection/relay** (e.g. ≈ a few / s / source, operator-tunable) | a recipient MUST bound how many symmetric-cost Argon2id verifications it performs; beyond budget, **defer to the requests area WITHOUT verifying** (never spend unbounded memory-hard work on unauthenticated input, never fail open) (§9.4) |
| Unknown-issuer token budget | 0 | self-issued/unvetted → no rate budget (§9.3.1) |
| Requests-area retention | 30 days | unproven cold MOTEs held here, not inbox (§2.7a) |
| Postage redemption check | online (issuer) | no offline bearer acceptance (§9.5.1) |

## 16.6 Transport & relay

| Parameter | Default | Notes |
|-----------|---------|-------|
| Relay reservation TTL | 1 h | libp2p circuit-relay v2 (§14.5) |
| Relay per-circuit cap | 2 min / 128 KiB | brief hole-punch assist only — not sustained sync |
| Relay per-peer reservations | ≈ 128 | libp2p relay-v2 abuse cap (§14.5) |
| Relay circuits / peer | ≈ 16 | libp2p relay-v2 abuse cap (§14.5) |
| Reachability ladder | IPv6 → hole-punch → relay | prefer direct (§4.3) |
| Offline-buffer TTL | 20 days | relay-mailbox message hold (§14.5) |
| Peer-buffer TTL | 20 days | buddy-node ciphertext hold (§4.3, §14.5); not an archive |
| Inactive-account purge | 90 days | relay-mailbox (§14.5) |
| Push payload | ≤ 4 KiB, content-free | wake-and-fetch only (§14.3) |
| Push wake rate limit | ≤ 1 wake / 60 s per device (burst-coalesced), budget ≈ 30 wakes / h | emitter **and** receiver enforce (§4.9.4); bounds battery-drain / relay-replay |
| Push wake replay cache | ≥ recent 512 nonces or 24 h, whichever larger | device-side dedup of replayed wake nonces (§4.9.1, §4.9.4, `0x0316`) |

## 16.7 Crypto suites

| `suite` | Sign | KEM | AEAD | Hash | Status |
|--------:|------|-----|------|------|--------|
| `0x01` | Ed25519 | X25519 (HPKE) | ChaCha20-Poly1305 | BLAKE3-256 | v0 REQUIRED |
| `0x02` | Ed25519+ML-DSA-65 | X-Wing (X25519+ML-KEM-768) | ChaCha20-Poly1305 | BLAKE3-256 | PQ target (RESERVED) |
| `0x03` | Ed25519+ML-DSA-65 | X-Wing (X25519+ML-KEM-768) | AES-256-GCM | BLAKE3-256 | AEAD-diverse emergency target (RESERVED, §1.1, §21.15) |

## 16.8 Auth, sessions & group ordering

| Parameter | Default | Notes |
|-----------|---------|-------|
| DMTAP-Auth session TTL | 24 h | key-bound session lifetime before re-auth (§13.4) |
| DMTAP-Auth session idle-timeout | 30 min | idle expiry of a key-bound session (§13.4) |
| RP delegation re-validation interval | ≤ 15 min | RP re-checks status endpoint / KT head (§13.4) |
| RP re-validation grace window (unreachable status/KT) | 2× re-validation interval (≤ 30 min) | honor last-validated delegation on unreachable status/KT, then fail closed (§13.4, §20.6) |
| Recovery-weakening veto window | 72 h | delay before a factor-weakening `RecoveryPolicy` change takes effect (§1.4) |
| Recovery veto quorum | `rotate_threshold` | a veto MUST satisfy this (asymmetric; a single factor cannot veto its own eviction, §1.4) |
| Committer-liveness timeout | 5 min | pending signed proposal unordered past this → takeover eligible (§5.1) |
| Committer-takeover hysteresis | 2 consecutive misses | takeover only after the timeout is exceeded twice in a row, to avoid churn on transient NAT/relay blips (§5.1) |
| **Committer roster quorum** | **> n/2** (⌈(n+1)/2⌉ of current members) | a takeover Commit is valid only with a strict-majority member-signature quorum, so two partitions CANNOT each elect a rival successor (split-brain prevention, §5.1) |
| Group join-request expiry | 30 days | a `request`-mode join with no admin response is auto-expired/cleaned up (§5.8.2; mirrors requests-area retention §16.5) |

## 16.9 Deniable mode & endpoint hardening

| Parameter | Default | Notes |
|-----------|---------|-------|
| Deniable one-time-prekey (OPK) bundle size | 100 | X3DH/PQXDH one-time prekeys published per replenish (§5.2.1, §18.4.8); Signal-scale |
| Deniable OPK replenish threshold | ≤ 20 remaining | owner's node republishes the bundle before exhaustion (§5.2.1) |
| Deniable last-resort prekey use | rate-limited (signed prekey / last-resort KEM) | fallback when OPKs exhausted; reused, so rate-limited to bound reuse (§5.2.1, `0x040B`) |
| Deniable signed-prekey rotation | ≤ 7 days | rotate `spk` (and PQ last-resort KEM key) on this cadence; old kept briefly for in-flight inits (§5.2.1) |
| Deniable-identity DH key (`idk`) rotation | on device revocation / recovery only | the dedicated long-term X25519 identity DH key is otherwise stable; rotated (fresh `idk` + `idk_sig`) when a holding device is lost (§5.2.1(a),(f), §6.7) |
| Deniable last-resort init replay cache | signed-prekey lifetime + overlap (≈ 7 d + skew) | responder cache of consumed initiator `ek_a`/`idk_a` for signed-prekey-only (no-OPK) first messages, to bound X3DH first-message replay (§5.2.1(a), `0x040C`) |
| Double-Ratchet skipped-message keys (MAX_SKIP) | 1000 | max out-of-order message keys cached before a gap is irrecoverable (`0x040D`, §5.2.1) |
| At-rest key relock timeout | ≤ 5 min inactivity (client policy) | evict the unlock-released at-rest key from memory after this idle period (§6.7); shorter on mobile |
| `sensitive` message persistence | never written at rest | ephemeral-view only; MUST NOT be persisted by a compliant recipient (§6.7, §18.3.6) |
| Device re-attestation cadence | ≤ 90 days (or the evidence's own expiry) | attestation-gated contexts treat older evidence as expired (`0x0118`) and require fresh evidence over the same non-exportable key (§1.2a, §18.4.2) |

## 16.10 Device-cluster sync (§5.6)

| Parameter | Default | Notes |
|-----------|---------|-------|
| Reconciliation range fan-out (`b`) | 16 | branching factor of the Merkle range summary; each divergent range re-splits into `b` sub-ranges (§5.6.3(a)) |
| Reconciliation leaf threshold | ≤ 8 ids | drill until a divergent range holds ≤ this many ids, then enumerate them directly (§5.6.3(a)) |
| HLC wall-clock skew bound | ±120 s (= §16.1) | an HLC `wall` more than the clock-skew tolerance ahead of the receiver is rejected (`0x0413`, §5.6.4) — a device cannot "win forever" |
| Tombstone-retention floor | 30 days | minimum a delete tombstone / superseded LWW value is retained **after** the all-member stability cut before GC (§5.6.5); tolerates a briefly-offline device (mirrors requests-area retention §16.5) |
| Durable seen-id / tombstone horizon | ≥ max(72 h retry, 20 d offline-buffer) = **20 days** | a delivered/deleted/expired MOTE's `id` and any durable-delete (`deleted`-flag) tombstone MUST persist in the durable seen-id set at least this long, so a **late duplicate** re-arriving from a retry (§16.1) or an offline/peer buffer (§16.6) is deduplicated against a *deleted* object rather than resurrecting it (§2.6, §5.6.4, §5.6.5). The tombstone-retention floor (above) is pinned **≥** this horizon |
| Cluster-member-liveness timeout | 7 days | a cluster member that has not advanced its `StabilityMark` within this window is **excluded from the stability cut** (and SHOULD be proposed for MLS Remove), so a dead-but-unrevoked device cannot stall tombstone GC forever (§5.6.5, mirrors the §16.8 committer-liveness principle at cluster timescale); a returning device re-syncs via backfill (§5.6.3) before it can push, so exclusion never enables resurrection |
| Cluster gossip / stability interval | ≤ 1 h | cadence of the periodic `ClusterSyncFrame` carrying `StabilityMark`s (max-applied HLC per device) for tombstone GC (§5.6.5) |
| Cluster journal replay batch | ≤ 1024 entries / frame | append-only journal segment size per `ClusterSyncFrame` on replay-backfill (§5.6.3(b)) |
| `cluster-replicated` default N | 3 (= §16.4) | eager chunk replicas across the box-cluster; tolerates N−1 holder loss (§5.5.2, §5.6.6) |

## 16.11 Gateway alias mapping (§7.10)

| Parameter | Default | Notes |
|-----------|---------|-------|
| Encoded-alias local-part max | 64 octets | RFC 5321 local-part limit; a longer encoded `localpart.nativedomain` alias is rejected (`0x0606`, §7.10.2) |
| Gateway-alias full-path max | 254 octets | RFC 5321 path limit for the whole `alias@gateway.domain` (§7.10.2) |
| Random-alias mapping TTL | indefinite (user-controlled) | a `GatewayAliasMap` row persists until the user burns/expires it; a gateway MAY set an unused-alias reap TTL by policy (§7.10.2, §18.3.12) |

All numeric values here are v0 defaults; a future protocol version MAY revise them, and
capability negotiation (§10.2) carries any non-default profile.
