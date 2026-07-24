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

Nodes MUST NOT rely on synchronised clocks for correctness — only for the skew-bounded windows
above and for ordering hints.

## 16.2 Naming, KT & DHT

| Parameter | Default | Notes |
|-----------|---------|-------|
| Key-name length | 8 words (80 bits) | §3.9.6; 12 words (**120** bits) in adversary-proof mode (§16.2.1). At 1024 words the list carries **10 bits per word**, so 12 words is 120 bits, not the 128 an earlier draft of this row asserted — 128 is not reachable at a whole number of words |
| Key-name wordlist size | 1024 | language-agnostic; +1 checksum word |
| KT signed-tree-head poll | ≤ 6 h | client re-checks its own entry (self-monitoring) |
| KT gossip interval (v1) | ≤ 1 h | STH gossip for equivocation detection (§3.5.2(a)) |
| KT maximum merge delay (v1) | ≤ 24 h | a `0x02` log MUST issue a new STH, and include every accepted entry, within this bound (§3.5.2(a)) |
| KT STH freshness window (v1) | ≤ 24 h | an STH older than this is stale (freeze-attack defence, `0x0112`); MUST be refreshed before the view is trusted (§3.5.2(a)) |
| KT log-set consistency quorum (v1) | > n/2 (⌈(n+1)/2⌉ of the pinned log set) | a `name → ik` binding is accepted only on a strict-majority quorum of logs, so a minority cannot forge or suppress it (§3.5.2(b); mirrors the §16.8 committer roster quorum) |
| DHT location-record TTL | 2 h | signed; republish before expiry (§4.2) |
| DHT republish interval | 45 min | < TTL, with jitter |
| DHT lookup redundancy (K) | 20 | store at K closest; S/Kademlia disjoint paths ≥ 3 |
| Location seq-number | monotonic u64 | rollback defence; reject older-or-equal |
| Max names per Identity (soft) | 32 | recommended cap on `Identity.names` aliases (§3.9.4, §3.11.3) to bound Identity size; tunable by policy, not a security gate |

### 16.2.1 Adversary-proof mode (key-name rendering) — normative

Three sections cite a 12-word key-name "for the adversary-proof mode" (§3.9.6,
`substrate/IDENTITY.md` §5, and the table above). This subsection is what they cite; it was a
dangling forward reference until it was written down, which is its own small lesson about
parameters that exist only as a column note.

**Adversary-proof mode is the 12-word (120-bit) rendering of the §18.9.17 key-name digest**, over
the same wordlist and folded checksum as the 8-word form (§3.9.6, §3.4.1). It changes nothing but
the truncation length: the derivation, the anchor-key input and the checksum construction are
identical, and the truncation takes **leading** bits, so a 12-word name and an 8-word name of the
same identity **share their first eight data words** and either can be checked against the other by
prefix. Their trailing **checksum** words differ — the folded checksum is computed over the rendered
payload, so a longer payload yields a different check word. A renderer MUST NOT compare the
checksum words of two different-length forms and conclude they name different keys.

**When it is REQUIRED.** A renderer MUST use the 12-word form whenever either condition holds:

1. **The key-name is the only verification the parties will perform** — no safety-number
   comparison (§3.4.1), no KT-audited resolution (§3.5), no out-of-band key transfer. The 8-word
   form's ≈ 2⁴⁰ chosen-collision margin (§3.9.6) is a *confirmation* margin: it is sound when it
   confirms a key obtained by some other means and unsound when it *is* the means.
2. **The rendering outlives the session that produced it** — anything printed, engraved, etched
   into a business card, published on a web page, carved into a monument, or otherwise fixed in a
   medium the identity holder cannot revise. An attacker grinding a chosen collision against a
   published name has unbounded time and a fixed target, which is the exact shape 2⁴⁰ does not
   survive.

**Elsewhere it is a SHOULD-offer.** A client SHOULD make the 12-word form available on request in
any context where a user is comparing key-names by eye or voice, and MUST label which form is
shown, because a truncated comparison of a longer name against a shorter one is a silent downgrade
to the shorter one's margin. A client MUST NOT present the 8-word form as adversary-proof, and
MUST NOT treat either form as a discriminator (§3.9.6) — mode selection changes the margin, never
the rule that identities are discriminated by key.

## 16.3 Mixnet & privacy

> **These parameters serve the opt-in, research-tier mixnet only** — see
> [docs/research/mixnet.md](docs/research/mixnet.md) for the mechanism they parameterize.
> The default transport tier is `fast`/direct (§4.6) and does not use any of them; a
> conformant node MAY implement the mixnet and, if it does, these are its pinned v0
> defaults.

| Parameter | Default | Notes |
|-----------|---------|-------|
| Mix path length | 3 hops | Sphinx onion, stratified ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md), [docs/research/mixnet.md §4.4.3](docs/research/mixnet.md)) |
| Per-hop mix delay | exp, mean 5 s | Poisson mixing; `private` tier ([docs/research/mixnet.md §4.4.5](docs/research/mixnet.md)) |
| Cover-traffic rate | **constant-rate, 30 s/cell (always-on nodes — MUST)**; Poisson mean 30 s/msg (battery/metered devices only) | loop + drop + recipient-side loop ([docs/research/mixnet.md §4.4.5](docs/research/mixnet.md)). Constant-rate ≈ **5.6 MB/day** — negligible for a mains-powered node, and it makes the traffic envelope **activity-independent** rather than merely activity-blurred, which is what defeats a patient long-horizon observer |
| Sphinx cell size (`δ`) | 2 KiB | Sphinx constant-length payload cell after padding ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md)) |
| Payload bucket ladder | **{16, 64} KiB = {8, 32} cells** | a MOTE is padded up to the next rung, then fragmented into that many 2 KiB cells ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md)); only ladder sizes appear on the wire. Two rungs only — an observer learns ≤ 1 bit of size per message (a third rung would make it log₂3 ≈ 1.58, [docs/research/mixnet.md §4.4.1](docs/research/mixnet.md)). The **16 KiB** floor is forced by the PQ envelope under suite `0x02`: **11 967 B** minimum with empty headers and body — 2 × 3 373 B `sig-val` + 2 × 1 984 B `ik-pub`/`sig-pub` + 1 120 B X-Wing encapsulated key + 133 B framing (§18.2, [docs/research/mixnet.md §4.4.1](docs/research/mixnet.md)) — leaving **4 417 B** of headroom. An 8 KiB rung is short by 3 775 B and can hold no conformant MOTE; a 2 KiB rung is short by an order more. Anchor-suite (`0x04`) announcements are ordinary **top-rung** MOTEs at ≈ 26 kB ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md)) |
| Multi-cell reassembly timeout | ≤ 15 min (≈ 3× the `private`-tier delivery latency budget) | a partial multi-cell MOTE held in the bounded reassembly cache is discarded if not completed within this window ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md) fragment reliability); bounds recipient memory against half-MOTE flooding |
| Fragment-recovery method | per-cell SURB-ARQ **or** FEC (`n > k` erasure code) | sender's choice, capability-negotiated (§10.2); recovers missing cells at the lost-fraction cost, never full re-send ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md)). Retransmitted/parity cells are ordinary constant-length Sphinx cells |
| Mix key epoch | 24 h | Sphinx mix-key rotation; advertise current+next, delete old key at `valid_until` ([docs/research/mixnet.md §4.4.4](docs/research/mixnet.md)) |
| Mix-directory freshness window | ≤ 1 mix-key epoch (24 h) | a `MixDirectory` older than this is stale (freeze-attack defence, `0x0311`); MUST be refreshed before building a `private` path, else fail closed ([docs/research/mixnet.md §4.4.2](docs/research/mixnet.md), [docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)). Mirrors the KT STH freshness window (§16.2) |
| Sphinx group / `β` stream / header MAC (v0) | X25519 / ChaCha20 / Poly1305 | header DH group, `β` stream cipher, per-hop header MAC over `β` ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md)); PQ variant [docs/research/mixnet.md §4.4.12](docs/research/mixnet.md) |
| Sphinx `δ` payload transform (v0) | LIONESS (wide-block PRP) | keyed permutation over the whole 2 KiB cell — payload tagging-resistance; NOT a stream cipher / AEAD ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md), [docs/research/mixnet.md §4.4.6](docs/research/mixnet.md)) |
| Mix replay-cache window | lifetime of every still-usable mix key + skew (current epoch + next-key overlap, ≈ up to 2 epochs + 120 s) | per-mix Sphinx-tag replay cache; retain until each key's `valid_until` passes — **no hard flush at the epoch boundary** while the overlap key is still usable ([docs/research/mixnet.md §4.4.6](docs/research/mixnet.md), `0x030E`) |
| Loop-cover rate (λ_loop) | Poisson, mean 30 s | client + mix loops for active-attack detection ([docs/research/mixnet.md §4.4.7](docs/research/mixnet.md)) |
| Loop-loss detection threshold | > 20% loss (sliding window) or latency ≫ delay budget | infer active drop/delay attack → `0x030F`, rotate + `HALT_ALERT` + fail-closed ([docs/research/mixnet.md §4.4.7](docs/research/mixnet.md)). Note: **sub-threshold** selective dropping (< 20% loss) is **bounded, not eliminated** — an adversary dropping a small fraction stays under detection but also achieves little; the **High-security profile's** faster loop rate (mean 5 s, [docs/research/mixnet.md §4.4.10](docs/research/mixnet.md)) **tightens** this floor (detects smaller/faster loss) |
| Entry-guard set size (G) | 2 | pinned entry-layer mixes per sender ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md)) |
| **Guard sample size** | 20 attested, ASN-disjoint entry mixes (Standard); **as many as exist, floor 3** (Bootstrap, [docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md)) | the **persistent sampled set** guards are drawn from ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md)). Chosen ONCE; rotation moves active guards *within* it and MUST NOT re-sample from the fleet — otherwise repeated independent draws turn the `(1−f)^G` bound into `(1−f)^(G·r)` and guards buy nothing over a decade. A Bootstrap-sized sample bounds nothing meaningful, which is why that profile carries no anonymity claim |
| Guard-sample refresh | only on exhaustion or explicit owner re-sample | a re-sample is a disclosed exposure event, not routine hygiene ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md)) |
| **Guard-sample top-up on Bootstrap→Standard** | retain all existing members; grow toward 20 over **≥ 4 mix-key epochs** | [docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md) constraint 3. The transition necessarily enlarges the sample (floor 3 → 20), and a naive enlargement is a **fresh draw at a moment an adversary chooses** — the re-sampling [docs/research/mixnet.md §4.4.8](docs/research/mixnet.md) forbids, which the `(1−f)^G` bound cannot survive. Spreading the top-up means no single epoch's fleet view determines the sample, so an adversary who inflates the derived view for one epoch cannot capture it |
| **Fleet-view shrinkage alarm** | derived view falling **below a previously observed Standard-satisfying size** ⇒ `HALT_ALERT` | [docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md). Mixes leave networks; they do not un-exist in bulk. A sudden contraction is more likely suppression than attrition, and it is the only signal distinguishing an eclipsed view from a genuinely young one — which are otherwise indistinguishable to the client |
| Entry-guard rotation period | 30 days | Tor-style rotation **inside the sample**; intersection-attack bound ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md)) |
| Path operator-diversity | ≥ 3 disjoint operators (one per hop) | no two hops share `operator` ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md)); relaxed only while single-operator ([docs/research/mixnet.md §4.4.11](docs/research/mixnet.md)) |
| **Path ASN-diversity** | ≥ 3 disjoint announced BGP origin ASNs | MUST; domains are cheap and attestation proves accountability, not independence — ASN is the diversity axis rented capacity cannot cheaply buy ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md)). Jurisdiction-disjointness SHOULD additionally hold |
| Minimum viable `private` path | 3 hops, 1/layer, current-epoch keys | below this the sender fails closed, never downgrades ([docs/research/mixnet.md §4.4.9](docs/research/mixnet.md), `0x0310`) |
| **Bootstrap profile** | 3 hops; **guard sample = all attested entry mixes, floor 3**; **best-effort ASN diversity ≥ 2**; other parameters as Standard | for a network too small to satisfy Standard ([docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md)). **No anonymity claim.** MUST be user-visible, MUST auto-upgrade to Standard once satisfiable, MUST NOT be fallen back to, and **ratchets per contact** like the §1.3 suite high-water-mark. Composing §16.3's 20-mix guard sample with the ≥ 3-ASN path rule would otherwise make a young network fail closed on **every** message — the profile exists so that "too small" degrades honestly instead of silently or fatally |
| Bootstrap → Standard threshold | guard sample reaches **20** attested ASN-disjoint entry mixes **and** ≥ 3 disjoint operator ASNs are path-buildable | evaluated per mix-key epoch from the client's own derived fleet view ([docs/research/mixnet.md §4.4.2](docs/research/mixnet.md)); published human-readably as network status (§14.6.3) |
| High-security profile | 5 hops, exp mean 30 s/hop, constant-rate cover mean 5 s, 3 guards / 7 d, 5 disjoint operators | user-selectable maximal anonymity ([docs/research/mixnet.md §4.4.10](docs/research/mixnet.md)); capability-negotiated (§10.2) |
| `private`-tier end-to-end latency | seconds–minutes (Standard); tens of minutes (High-security) | consequence of mixing (§6, [docs/research/mixnet.md §4.4.10](docs/research/mixnet.md)) |

## 16.4 File tiers & transfer

**Size / privacy sub-tiers** (metadata-privacy axis, §6.5 — normal vs. bulk):

| Parameter | Default | Notes |
|-----------|---------|-------|
| Inline attachment threshold | ≤ **48 KiB** | rides the message (§2.5). This is the **top bucket-ladder rung (64 KiB) minus the PQ envelope**: the envelope costs 11 967 B before any content ([docs/research/mixnet.md §4.4.1](docs/research/mixnet.md), opt-in research-tier mixnet sizing), so 65 536 − 11 967 − 49 152 leaves **4 417 B** for headers, `refs` and framing — the same headroom the floor rung reserves. NOT larger: a bigger inline cap forces a MOTE above the top bucket → cannot ride the opt-in `private` mixnet even when selected (§5.5.1) |
| Normal-file threshold | ≤ 4 MiB (≤ 4 chunks) | chunks route via whichever tier the message uses — default `fast`, or the opt-in mixnet's full privacy if selected (§6.5) |
| Chunk size | 1 MiB | fixed; content-addressed over **ciphertext** (§5.5, §18.9.5) |
| Large-file bulk | > 4 MiB | fast/direct bulk path — weaker privacy (§6.5) |
| Swarm parallelism | ≤ 8 sources | per-file concurrent fetch; bounds swarm-poisoning wasted bandwidth (§5.5.3) |

**Delivery / durability tiers** (durability axis, §5.5.1–§5.5.5 — orthogonal to the privacy axis
above; inline/push/pull governs durability, normal/bulk governs metadata privacy):

| Parameter | Default | Notes |
|-----------|---------|-------|
| Inline tier cap (durable-by-delivery) | ≤ **48 KiB** | bytes in the sealed MOTE (`Attachment.inline`); = the inline attachment threshold above (§5.5.1) |
| Attached tier cap (pushed, durable recipient copy) | ≤ 25 MiB | chunks **pushed** into the recipient's store on delivery — survives the sender dropping (§5.5.1) |
| Referenced tier (pull-on-demand) | > 25 MiB | `ManifestRef` + key in the MOTE; chunks pulled from a holder; MUST carry a `durability` class (§5.5.2) |
| Auto-pull-to-durable threshold | ≤ 256 MiB | a client SHOULD auto-pull-and-pin a Referenced file below this on receipt, converting origin-hold → recipient-pinned (§5.5.2) |
| Default `cluster-replicated` N | 3 | replicas across a box-cluster (§5.6, §14); tolerates N−1 holder loss (§5.5.2) |
| Default `pinned(term)` retention | 90 days | minimum paid-pin retention term; renew before expiry (mirrors inactive-account purge §16.6); after it the host MAY GC (`0x080B`, §5.5.4) |
| Inbound spool cap (per unproven sender) | ≤ 64 MiB aggregate | a pushed Inline/Attached file exceeding this for that sender is refused fail-closed (`0x080C`, storage-DoS / spool-fill defence, §5.5.5); tunable by operator policy, generous on self-host |

## 16.5 Anti-abuse

| Parameter | Default | Notes |
|-----------|---------|-------|
| **Cold-sender sequential work (VDF)** | Wesolowski/Pietrzak VDF, delay ≈ **3 s** single-core, adaptive | **MAY**, opt-in/research-tier — an optional cold-contact cost where both parties support it ([docs/research/vdf.md §9.4.1](docs/research/vdf.md)). It bounds the **aggregate-parallelism** advantage (a botnet's breadth buys ≈ nothing per puzzle) but leaves a **10–100× per-gate latency** advantage intact; sequentiality is a **conjecture** and is defined only relative to a `p(t)`-processor bound. **Not the interoperable floor, and not a SHOULD:** it needs a group of unknown order (RSA trusted setup, or class groups) with no IETF standard, no interoperable parameter set and no pinned proof encoding — and it is **not post-quantum** (a quantum adversary computes the group order and collapses the delay), unlike the rest of the suite. A recipient MUST NOT require one as the only acceptable proof |
| VDF verification cost | milliseconds, independent of delay | **asymmetric by construction** — this is why the VDF path needs no verification budget and no defer-without-verifying escape hatch, unlike Argon2id below ([docs/research/vdf.md §9.4.1](docs/research/vdf.md)) |
| VDF puzzle scope | `id ‖ recipient ‖ nonce(epoch) ‖ sender_key` | `sender_key` binding per §9.2a — a stolen proof is worthless under any other ephemeral key |
| **Zero-relationship delivery floor** | **a policy constraint, not a per-sender count** (§9.7a) | MUST. A stranger with only a keypair and a valid work proof — **VDF *or* memory-hard PoW**, recipient's preference but both acceptable — always reaches the **requests area**, never the inbox, subject only to the aggregate budget below. Without it the §3.13 key-name promise is false: a sovereign identity would be nameable, verifiable, and silently undeliverable. A policy that makes a valid work proof never sufficient — or accepts **only** a VDF, refusing the interoperable proof — is non-conformant (`ERR_POLICY_BELOW_FLOOR`, `0x070F`). **NOT expressed per `sender_key`:** §2.2 makes that key ephemeral and fresh per message, so a per-key quota is a quota per message (no bound at all), and the recipient has no stable subject to meter at gate time by design — identity appears at §2.7 step 8, after decryption |
| **Requests-area aggregate budget** | node-wide per 24 h: **≤ 2 000 entries** and **≤ 256 MiB**; operator-tunable upward | What makes the floor affordable to hold open. Without an aggregate cap the floor + §9.4's defer-**without**-verifying + §2.7a's no-drop rule compose into an unbounded **durable-storage DoS**: saturate the per-connection verification budget, then send unlimited cold MOTEs carrying **garbage** proofs — no work performed at all — and a conformant node must store every one for 30 days. Past this budget a recipient MAY refuse **unverified** cold MOTEs; that is refusal of unverified input, not the silent dropping of a verified MOTE that §2.7a forbids |
| **Unverified-deferral holding class** | separate from the requests area: **≤ 200 entries**, **≤ 16 MiB**, retention **≤ 24 h**, non-durable | Where a MOTE deferred *without verification* (§9.4's over-budget path) goes. Deliberately small and short-lived so an unverified flood cannot displace verified floor traffic in the 30-day requests area, and it MUST NOT count as having satisfied the floor |
| Cold-sender PoW | memory-hard (Argon2id), adaptive | **the interoperable MUST floor** a recipient has to accept (§9.4, [docs/research/vdf.md §9.4.1](docs/research/vdf.md) for the opt-in VDF alternative) — the weaker mechanism everyone can implement today, which is precisely what makes it the floor, and the **only** cold-contact proof in this table that is not itself broken by a quantum adversary; a VDF is a MAY layered above it |
| PoW puzzle scope | `id ‖ recipient ‖ nonce(epoch)` | fresh epoch nonce to prevent precompute |
| **Accepted `nonce(epoch)` scopes** | the recipient's **published beacon** OR the **UTC date** — both MUST be accepted | §9.4. The fallback is conditioned on the *recipient's* behaviour but chosen by the *sender*, who cannot tell "publishes none" from "cannot reach it"; and a key-name-only identity (`self`, §3.12.4) has no publication surface for a beacon at all. Requiring the beacon would make the §9.7a floor unreachable for exactly the sovereign user it exists for. Cost: UTC bounds precompute to a day rather than the beacon cadence — accepted deliberately |
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
| `0x01` | Ed25519 | X25519 (HPKE) | ChaCha20-Poly1305 | BLAKE3-256 | LEGACY — accept, never originate (§1.1) |
| `0x02` | Ed25519+ML-DSA-65 | X-Wing (X25519+ML-KEM-768) | ChaCha20-Poly1305 | BLAKE3-256 | **v0 REQUIRED originating suite** (§1.1) |
| `0x03` | Ed25519+ML-DSA-65 | X-Wing (X25519+ML-KEM-768) | AES-256-GCM | BLAKE3-256 | AEAD-diverse emergency target (RESERVED, §1.1, §21.15) |
| `0x04` | Ed25519+SLH-DSA-128s | X-Wing (X25519+ML-KEM-768) | ChaCha20-Poly1305 | BLAKE3-256 | signature-diverse emergency target; the intended **anchor** profile (RESERVED, §1.1, §1.2.0) |
| `0x05` | Ed25519+ML-DSA-65 | X-Wing (X25519+ML-KEM-768) | ChaCha20-Poly1305 | **SHA3-256** | hash-diverse emergency target (RESERVED, §1.1, §21.15); digests travel under multihash prefix `0x16` (§18.1.5) |

**Suite-governed lengths that other sections' arithmetic depends on** (§18.2 is authoritative):
`sig-val` = 3 373 B and `ik-pub`/`sig-pub` = 1 984 B under `0x02`/`0x03`/`0x05`; `sig-val` = **7 920 B**
(Ed25519 64 ‖ SLH-DSA-128s **7 856**, FIPS 205 Table 2) and `ik-pub` = **64 B** (32 ‖ 32) under
`0x04`; HPKE encapsulated key = 1 120 B (X-Wing) under all four PQ suites. These are what set the bucket
ladder floor (§16.3, [docs/research/mixnet.md §4.4.1](docs/research/mixnet.md)).

**X-Wing's standards status (honest limit, §1.3).** X-Wing is an **Independent Submission**
Internet-Draft (`draft-connolly-cfrg-xwing-kem-10`), **not CFRG-adopted**, and **FIPS 203
standardizes no KEM combiner**, warning that a combined KEM containing ML-KEM "might not meet
IND-CCA2 security" and deferring to SP 800-227. It is pinned on analysis and a fixed HPKE code
point, not on standing.

> **Resolved.** §18.2 previously labelled `0x01` "v0 REQUIRED" and `0x02` "RESERVED", and
> instructed a v0 implementation to *reject* `0x02` fail-closed — flatly contradicting §1.1 and,
> read literally, telling implementers to refuse the suite they are required to originate. The
> labels were **implementation status** (what the reference core supports, and what the frozen
> vectors were generated under) presented in a **normative** column. §18.2 now separates the two
> axes explicitly: the normative status is §1.1's, and the all-`0x01` vector corpus is disclosed
> as a **gap in the corpus, not a licence to originate `0x01`**. Regenerating the corpus under
> `0x02` remains its own tracked change; it was never what the contradiction depended on.

## 16.8 Auth, sessions & group ordering

| Parameter | Default | Notes |
|-----------|---------|-------|
| DMTAP-Auth session TTL | 24 h | key-bound session lifetime before re-auth (§13.4) |
| DMTAP-Auth session idle-timeout | 30 min | idle expiry of a key-bound session (§13.4) |
| RP delegation re-validation interval | ≤ 15 min | RP re-checks status endpoint / KT head (§13.4) |
| RP re-validation grace window (unreachable status/KT) | 2× re-validation interval (≤ 30 min) | honour last-validated delegation on unreachable status/KT, then fail closed (§13.4, §20.6) |
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
| **Cold-sender OPK consumption cap** | ≤ **10** `opk` reservations per cold sender-key per window, and ≤ **25%** of the bundle held in reservations by cold senders at once | §19.3.1 reserve-then-commit. A one-time prekey is exhaustible and step 7 precedes step 8, so an *unauthenticated* sender could otherwise burn one per message and drain a 100-key bundle, forcing every subsequent legitimate first contact onto the **replayable** last-resort path (§5.2.1). Beyond the cap a cold `DeniableInit` is served from last-resort instead — degrading that one session's forward secrecy rather than depleting a shared resource for everyone |
| **OPK reservation TTL** | ≤ 5 min | an unclaimed reservation (step 8 never reached) is released, so a flood cannot pin the bundle indefinitely; commit happens at step 9 (§19.3.1) |
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
| Gateway-alias full-path max | 254 octets | RFC 5321 §4.5.3.1.3's 256-octet forward-/reverse-path limit, less the two `<`/`>` bracket octets that DMTAP's bracket-free `alias@gateway.domain` form never carries, for the whole address (§7.10.2) |
| Random-alias mapping TTL | indefinite (user-controlled) | a `GatewayAliasMap` row persists until the user burns/expires it; a gateway MAY set an unused-alias reap TTL by policy (§7.10.2, §18.3.12) |

All numeric values here are v0 defaults; a future protocol version MAY revise them, and
capability negotiation (§10.2) carries any non-default profile.
