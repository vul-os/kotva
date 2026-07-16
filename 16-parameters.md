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

## 16.3 Mixnet & privacy

| Parameter | Default | Notes |
|-----------|---------|-------|
| Mix path length | 3 hops | Sphinx onion, stratified (§4.4.1, §4.4.3) |
| Per-hop mix delay | exp, mean 5 s | Poisson mixing; `private` tier (§4.4.5) |
| Cover-traffic rate | Poisson, mean 30 s/msg | loop + drop + recipient-side loop; tunable per node; higher = more privacy, more bandwidth (§4.4.5) |
| Sphinx cell size (`δ`) | 2 KiB | Sphinx constant-length payload cell after padding (§4.4.1) |
| Payload bucket ladder | {2, 8, 32, 64} KiB = {1, 4, 16, 32} cells | a MOTE is padded up to the next rung, then fragmented into that many 2 KiB cells (§4.4.1); only ladder sizes appear on the wire |
| Mix key epoch | 24 h | Sphinx mix-key rotation; advertise current+next, delete old key at `valid_until` (§4.4.4) |
| Mix-directory freshness window | ≤ 1 mix-key epoch (24 h) | a `MixDirectory` older than this is stale (freeze-attack defense, `0x0311`); MUST be refreshed before building a `private` path, else fail closed (§4.4.2, §4.4.9). Mirrors the KT STH freshness window (§16.2) |
| Sphinx group / `β` stream / header MAC (v0) | X25519 / ChaCha20 / Poly1305 | header DH group, `β` stream cipher, per-hop header MAC over `β` (§4.4.1); PQ variant §4.4.12 |
| Sphinx `δ` payload transform (v0) | LIONESS (wide-block PRP) | keyed permutation over the whole 2 KiB cell — payload tagging-resistance; NOT a stream cipher / AEAD (§4.4.1, §4.4.6) |
| Mix replay-cache window | lifetime of every still-usable mix key + skew (current epoch + next-key overlap, ≈ up to 2 epochs + 120 s) | per-mix Sphinx-tag replay cache; retain until each key's `valid_until` passes — **no hard flush at the epoch boundary** while the overlap key is still usable (§4.4.6, `0x030E`) |
| Loop-cover rate (λ_loop) | Poisson, mean 30 s | client + mix loops for active-attack detection (§4.4.7) |
| Loop-loss detection threshold | > 20% loss (sliding window) or latency ≫ delay budget | infer active drop/delay attack → `0x030F`, rotate + `HALT_ALERT` + fail-closed (§4.4.7). Note: **sub-threshold** selective dropping (< 20% loss) is **bounded, not eliminated** — an adversary dropping a small fraction stays under detection but also achieves little; the **High-security profile's** faster loop rate (mean 5 s, §4.4.10) **tightens** this floor (detects smaller/faster loss) |
| Entry-guard set size (G) | 2 | pinned entry-layer mixes per sender (§4.4.8) |
| Entry-guard rotation period | 30 days | Tor-style guard rotation; intersection-attack bound (§4.4.8) |
| Path operator-diversity | ≥ 3 disjoint operators (one per hop) | no two hops share `operator` (§4.4.8); relaxed only while single-operator (§4.4.11) |
| Minimum viable `private` path | 3 hops, 1/layer, current-epoch keys | below this the sender fails closed, never downgrades (§4.4.9, `0x0310`) |
| High-security profile | 5 hops, exp mean 30 s/hop, constant-rate cover mean 5 s, 3 guards / 7 d, 5 disjoint operators | user-selectable maximal anonymity (§4.4.10); capability-negotiated (§10.2) |
| `private`-tier end-to-end latency | seconds–minutes (Standard); tens of minutes (High-security) | consequence of mixing (§6, §4.4.10) |

## 16.4 File tiers & transfer

| Parameter | Default | Notes |
|-----------|---------|-------|
| Inline attachment threshold | ≤ 64 KiB | rides the message (§2.5); = top bucket-ladder rung (32 Sphinx cells, §16.3) |
| Normal-file threshold | ≤ 4 MiB (≤ 4 chunks) | chunks via mixnet — full privacy (§6.5) |
| Chunk size | 1 MiB | fixed; content-addressed (§5.5) |
| Large-file bulk | > 4 MiB | fast/onion bulk path — weaker privacy (§6.5) |
| Swarm parallelism | ≤ 8 sources | per-file concurrent fetch |

## 16.5 Anti-abuse

| Parameter | Default | Notes |
|-----------|---------|-------|
| Cold-sender PoW | memory-hard (Argon2id), adaptive | last-resort tier (§9.4) |
| PoW puzzle scope | `id ‖ recipient ‖ nonce(epoch)` | fresh epoch nonce to prevent precompute |
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
| `0x02` | Ed25519+ML-DSA-65 | X-Wing (X25519+ML-KEM-768) | ChaCha20-Poly1305 | BLAKE3-256 | PQ target |

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

All numeric values here are v0 defaults; a future protocol version MAY revise them, and
capability negotiation (§10.2) carries any non-default profile.
