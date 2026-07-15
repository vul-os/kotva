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
| KT gossip interval (v1) | ≤ 1 h | tree-head gossip for equivocation detection |
| DHT location-record TTL | 2 h | signed; republish before expiry (§4.2) |
| DHT republish interval | 45 min | < TTL, with jitter |
| DHT lookup redundancy (K) | 20 | store at K closest; S/Kademlia disjoint paths ≥ 3 |
| Location seq-number | monotonic u64 | rollback defense; reject older-or-equal |

## 16.3 Mixnet & privacy

| Parameter | Default | Notes |
|-----------|---------|-------|
| Mix path length | 3 hops | Sphinx onion (§4.4) |
| Per-hop mix delay | exp, mean 5 s | Poisson mixing; `private` tier |
| Cover-traffic rate | Poisson, mean 30 s/msg | tunable per node; higher = more privacy, more bandwidth |
| Padded packet size (bucket) | 2 KiB | Sphinx constant-length cell after padding |
| `private`-tier end-to-end latency | seconds–minutes | consequence of mixing (§6) |

## 16.4 File tiers & transfer

| Parameter | Default | Notes |
|-----------|---------|-------|
| Inline attachment threshold | ≤ 64 KiB | rides the message (§2.5) |
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
| Reachability ladder | IPv6 → hole-punch → relay | prefer direct (§4.3) |
| Offline-buffer TTL | 20 days | relay-mailbox message hold (§14.5) |
| Inactive-account purge | 90 days | relay-mailbox (§14.5) |
| Push payload | ≤ 4 KiB, content-free | wake-and-fetch only (§14.3) |

## 16.7 Crypto suites

| `suite` | Sign | KEM | AEAD | Hash | Status |
|--------:|------|-----|------|------|--------|
| `0x01` | Ed25519 | X25519 (HPKE) | ChaCha20-Poly1305 | BLAKE3-256 | v0 REQUIRED |
| `0x02` | Ed25519+ML-DSA-65 | X-Wing (X25519+ML-KEM-768) | ChaCha20-Poly1305 | BLAKE3-256 | PQ target |

All numeric values here are v0 defaults; a future protocol version MAY revise them, and
capability negotiation (§10.2) carries any non-default profile.
