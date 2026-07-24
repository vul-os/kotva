# 11. Grounding & References

This section is **informative**; where it restates requirements, the owning section governs.

This section records the standards each design choice rests on, the corrections applied after
verification, and a consolidated statement of honest limits. DMTAP deliberately **composes
existing standards**; the novelty is the composition and transport, not new cryptography.

## 11.1 Verified building blocks

| Layer | Choice | Standard / source | Status |
|-------|--------|-------------------|--------|
| PKE | HPKE | RFC 9180 (DHKEM(X25519)/HKDF-SHA256/ChaCha20Poly1305 = `0x0020`/`0x0001`/`0x0003`) | confirmed |
| Signatures | Ed25519 (PureEdDSA) | RFC 8032 | confirmed |
| Key agreement | X25519 | RFC 7748 | confirmed |
| PQ KEM | ML-KEM-768 (cat-3) | FIPS 203 | confirmed |
| PQ signatures | ML-DSA-65 (cat-3) | FIPS 204 (SLH-DSA/FIPS 205 as backup) | confirmed |
| Hybrid HPKE KEM | **X-Wing** (ML-KEM-768 ⊕ X25519) | `draft-connolly-cfrg-xwing-kem-10` (HPKE KEM `0x647a`) | **Independent Submission draft — NOT CFRG-adopted** (§1.3) |
| Hashing | BLAKE3-256 + agility prefix | (BLAKE3 not FIPS/IETF-standardized; SHA-256 fallback) | confirmed w/ caveat |
| Serialization | Deterministic CBOR (bytewise/core key order) | RFC 8949 §4.2 (dCBOR draft for floats) | confirmed |
| Group/session | MLS | RFC 9420; architecture RFC 9750; suites `0x0001`/`0x0003` | confirmed |
| PQ-MLS | ML-KEM ciphersuites + combiner | `draft-ietf-mls-pq-ciphersuites`, `draft-ietf-mls-combiner` | adopted |
| Async join | MLS KeyPackages + external commits | RFC 9420 | adopted (replaces PQXDH) |
| Sealed sender | payload-embedded sender | Signal (2018) | confirmed w/ caveat |
| Mixnet packet | Sphinx | Danezis & Goldberg, IEEE S&P 2009 | confirmed |
| Mixnet design | Loopix (Poisson mix, loop/drop/mix cover) | Piotrowska et al., USENIX Sec 2017; Nym | confirmed |
| Client sync | JMAP | RFC 8620 / RFC 8621 | confirmed |
| Mesh | libp2p (Kad, Relay v2, DCUtR, AutoNAT v2, Noise, QUIC) | libp2p specs | confirmed |
| Location record | IPNS-style signed value record | IPFS/IPNS | confirmed |
| Naming | DNS TXT (key) + SVCB (service) | RFC 9460; cf. DKIM RFC 6376, OPENPGPKEY RFC 7929 | confirmed |
| Key transparency | Merkle log + STH + inclusion/consistency/absence | CT RFC 6962; CONIKS; Google KT; Apple IMCKV; WhatsApp AKD | confirmed |
| Anti-abuse tokens | Privacy Pass + **ARC** scoping | RFC 9576/9577/9578; `draft-ietf-privacypass-arc-protocol` | RFCs confirmed; **ARC is now Privacy Pass WG-adopted** (Standards Track draft, formerly the individual `draft-yun-privacypass-arc`) — still a draft, not an RFC, so kept behind the token-profile seam (§9.3) |
| PoW fallback | memory-hard hashcash | Back 1997; Argon2 | confirmed w/ caveat |
| Recovery | VSS + SLIP-0039 + **FROST** | Feldman/Pedersen VSS; SLIP-0039; RFC 9591 (FROST) | upgraded |
| Self-sovereign naming (opt) | name-chain (ENS-style) | Zooko's Triangle; ENS | optional |

## 11.2 Corrections applied after verification

1. **MLS handshake ordering** — Commit/Proposal/Welcome MUST use an ordered channel, **not**
   the reordering mixnet (§5.1). This is the hardest part of MLS-over-mesh; track
   `draft-kohbrok-mls-dmls`.
2. **Async init** — use MLS-native **KeyPackages + external commits**, not a bolted-on PQXDH
   (§5.3).
3. **Deniability** — the default MLS path is **non-repudiable** (DMTAP does not swap MLS
   signatures for MACs, §5.2); a **normative, optional, capability-negotiated deniable 1:1 mode**
   (Signal-style X3DH/PQXDH + Double Ratchet with shared-key-MAC authentication) is specified in
   §5.2.1 for users who need repudiation. Groups stay MLS/non-deniable; the residual is honest
   (repudiation of the cryptographic transcript, not protection against a compromised endpoint).
4. **No-PIR claim** — reframed honestly: push delivery avoids PIR's problem but leaves
   last-hop visibility, receipt-timing, and intersection exposure — mitigated by node/identity
   decoupling + recipient-side cover traffic (§6.4).
5. **DHT security** — record signing ≠ eclipse resistance; added S/Kademlia disjoint paths,
   IP-diversity buckets, seq#/EOL rollback defense, and a **non-DHT rendezvous fallback**; the
   DHT is one discovery mechanism, not the root of trust (§4.2).
6. **QUIC roaming** — carried by location re-publish + re-dial, not seamless QUIC migration
   (`quinn` has no multipath) (§4.1).
7. **Sealed sender scope** — hides sender from intermediaries, not the IP (mixnet does that);
   metadata-*reduction*, not elimination (NDSS 2021 side-channel) (§6.2).
8. **Bulk-file tier** — explicitly weaker: large-tier chunks transfer **direct** and swarmed and
   **never** traverse the mixnet (§4.5), so the fact and approximate size of a transfer stay
   observable to a well-positioned observer; swarming/padding raise cost, not guarantees (§6.5).
9. **Recovery** — VSS over plain Shamir; FROST to avoid key reassembly; SLIP-0039 encoding;
   "redistribution" vs "proactive refresh" distinguished (§1.4).
10. **Anti-abuse tokens** — need **ARC** (per-origin rate-limited + cross-recipient unlinkable),
    not vanilla Privacy Pass; PoW must be **memory-hard**, last-resort (§9).
11. **Hash agility** — content-address digests carry a multihash-style prefix (§2.2).
12. **DNS is discovery, not trust** — a DNS binding is only trustworthy via KT; SVCB is for
    service params, a dedicated TXT for the key (§3).
13. **DTN / radio** — store-and-forward is a DMTAP-built layer (not libp2p); Bluetooth/Wi-Fi
    Direct/LoRa are future work, not v0 (§4.8).

## 11.3 Consolidated honest limits

- **Global active adversary** with unlimited resources is not fully defeated — the
  **Anonymity Trilemma** (Das et al., IEEE S&P 2018) makes strong anonymity cost latency
  and/or bandwidth. The opt-in, research-tier mixnet (`docs/research/mixnet.md`) targets a
  **global passive adversary** and *bounds* (not eliminates) partial-active exposure; DMTAP's
  default (`fast`) guarantee is a metadata *reduction* vs intermediaries, not GPA immunity.
- **Intersection / statistical-disclosure attacks** against always-on nodes are inherited;
  cover traffic bounds, does not eliminate, them.
- **First-contact MITM** is possible until KT gossip or out-of-band verification (§3.4).
- **KT without gossip** catches later tampering and enables self-monitoring, but a single
  equivocating log is not caught until independent auditors/gossip exist (§3.5).
- **Large-file bulk** metadata is weaker than message metadata by necessity — bulk never traverses
  the mixnet (§4.5), so the fact and approximate size of a transfer remain observable.
- **Persistent-file forward secrecy** cannot match ephemeral-message FS (files must stay
  readable); mitigated by per-file keys + FS-delivered keys + at-rest encryption (§6.7).
- **Key loss** of `IK` plus all recovery factors is unrecoverable (the bottom turtle, §1.4).
- **PQ onion (Sphinx)** as published is not PQ-safe; PQ mix packet formats are open research.
- **The hybrid KEM combiner is not standardized.** X-Wing is an **Independent Submission**
  Internet-Draft, not a CFRG-adopted document, and **FIPS 203 standardizes no combiner**, warning
  that a combined KEM containing ML-KEM "might not meet IND-CCA2 security" and deferring to
  SP 800-227. DMTAP pins X-Wing on analysis and a fixed HPKE code point, not on standing; the
  choice is confined to one suite byte (§1.3, §21.15).
- **Hybrid signatures give EUF-CMA, not SUF-CMA.** No composite PQ/T variant is known to achieve
  strong unforgeability against a quantum adversary, so DMTAP treats signatures as malleable and
  derives no identifier from one (§1.3, §2.2).
- **A hash break is not migratable, and this is the largest single exposure in the document.**
  Every content address, every Merkle root, every `prev` link, every key-transparency leaf and
  every signing preimage is a **BLAKE3-256** digest (§18.9). Suite `0x05` (§1.1) and the §18.1.5
  multihash prefix make a **new** address *expressible* under a successor hash — an encoding seam,
  not a migration path — neither **re-anchors an existing graph**. Re-anchoring means recomputing
  every address and **re-signing** every object that names one, which requires each author's own
  key — the spec proves this against its own hardest case in §24.14's change log: re-signing an
  existing record needs the author's key, which no PUB server, archive or migration tool ever
  holds for a self-custodied identity. So migration is **per-author and consensual, never a bulk
  operator-run rewrite**, proceeds only as fast as authors act, and for keys that are lost,
  retired, or whose holders are gone it **never completes**. There is no network-wide re-hash, and
  there cannot be one without an authority able to re-sign other people's history, which DMTAP
  does not have and does not want. Combined with §22.7 irrevocability — you cannot un-publish, only
  publish a correction — every un-migrated object stays authoritative indefinitely, not merely
  until its author gets around to re-signing it. Hash migration is therefore **forward-only**:
  new objects move to the successor suite, old objects keep whatever assurance BLAKE3-256 still
  has. **Content whose integrity must outlive BLAKE3-256 MUST be re-published under the successor
  suite by its author, while that author still holds their key.** The one derived identifier with no signed object behind it — the key-name
  (§3.9.6) — now at least commits its algorithm inside the hashed input (§18.9.17), so a migration
  produces a distinguishable name instead of a silent replacement; that makes the event *visible*,
  not *reversible*.
- **The optional cold-contact cost (VDF) is not post-quantum.** Both named constructions rest on
  a group of unknown order, which a quantum adversary computes; it is a `MAY` for the opt-in,
  research-tier mixnet, memory-hard PoW is the MUST floor, and the exposure is future spam cost,
  never retroactive confidentiality
  ([docs/research/vdf.md §9.4.1](docs/research/vdf.md)).
- **Volunteer-provisioned infrastructure may not materialise.** Mixes, KT logs, rendezvous nodes
  and buffers are reciprocal roles, not funded services; the default (`fast`) does not depend on
  the mixnet fleet at all, so it never degrades — but if too few nodes take the mix role, the
  **opt-in** `private` tier simply stays **unbuilt**, and a client MUST NOT present it as
  available (§6.6 item 13,
  [docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md)).
- **No free, human-shareable, user-owned name exists.** Key-names are unwieldy; domains and chain
  names cost money; a free user's shareable name is a non-portable gateway alias (§3.13.4).

## 11.4 Implementation-stack notes (reference, Rust)

- **Prefer formally-verified PQ crates** (`libcrux-ml-kem`, `libcrux-ml-dsa`) over the
  unaudited RustCrypto `ml-kem`/`ml-dsa`; pin past the `ml-dsa` advisory GHSA-5x2r-hc65-25f9.
- **openmls** — independently audited (SRLabs, 2025) — is the MLS choice.
- **hpke** (rozbb) or **hpke-rs** (Cryspen) — not `ed25519-dalek-hpke` (experimental).
- **ed25519-dalek ≥2.x**, **x25519-dalek**, **blake3**, **ciborium** — mature.
- **Mixnet**: `sphinx-packet`/`nym-sphinx-*` are low-cadence standalone; track Nym directly.
- **rust-libp2p** is production-usable but churns its API (pin versions); go-libp2p is the more
  mature interop reference — a deliberate memory-safety-vs-maturity trade.

## 11.5 Selected bibliography

§15 is the authoritative reference list; this subsection is a reading guide, not a registry.

- Danezis & Goldberg, *Sphinx: A Compact and Provably Secure Mix Format*, IEEE S&P 2009.
- Piotrowska, Hayes, Elahi, Meiser, Danezis, *The Loopix Anonymity System*, USENIX Sec 2017.
- Das, Meiser, Mohammadi, Kate, *Anonymity Trilemma*, IEEE S&P 2018.
- van den Hooff et al., *Vuvuzela*, SOSP 2015; Angel & Setty, *Pung*, OSDI 2016; Cheng et al.,
  *Talek*, ACSAC 2020.
- Dingledine, Mathewson, Syverson, *Tor*, USENIX Sec 2004; Murdoch & Danezis, *Low-Cost Traffic
  Analysis of Tor*, IEEE S&P 2005.
- Melara et al., *CONIKS*, USENIX Sec 2015; RFC 6962 (Certificate Transparency).
- Raman, Joglekar, De Cristofaro, Sastry & Tyson, *Challenges in the Decentralised Web: The
  Mastodon Case*, ACM IMC 2019 — volunteer availability and replication measurements behind the
  `n`-of-`m` buffer role (§14.3a).
- Liu, Fass, Hong et al., *Who's Got Your Mail? Characterizing Mail Service Provider Usage*, ACM
  IMC 2021 — the anti-abuse centralization tier behind the authorize-never-classify rule (§9.11).
- Martiny et al., *Improving Signal's Sealed Sender*, NDSS 2021.
- RFCs 9180, 8032, 7748, 8949, 9420, 9750, 8620, 8621, 9460, 6376, 7929, 9576/9577/9578, 9591.
- FIPS 203, 204, 205; `draft-connolly-cfrg-xwing-kem`; `draft-ietf-mls-pq-ciphersuites`;
  `draft-ietf-mls-combiner`; `draft-kohbrok-mls-dmls`; `draft-ietf-privacypass-arc-protocol`.
