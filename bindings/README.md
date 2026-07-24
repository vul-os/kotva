# Bindings — what KOTVA adopts instead of reinventing

KOTVA "collects it all" the way a narrow waist does: it defines the thin shared core and
**binds** to existing, proven standards for everything else. A binding is a *thin mapping*
document — "how substrate identity/objects map onto this external standard" — never a
re-derivation. The rule ([DIRECTION §3](../DIRECTION.md)): specify new bytes only where
nothing exists; adopt everywhere else.

Each row is a slot. When the frontier improves, swap the binding — the substrate and profiles
never change ([DIRECTION §9](../DIRECTION.md), "future-proof by seams").

---

## The index

| Primitive / need | Bind to | What it gives us | Honest maturity note | Status |
|---|---|---|---|---|
| **Identity recovery** | Account abstraction (ERC-4337, EIP-7702), passkeys, MPC (Turnkey/Privy/Web3Auth/Lit) | Kills irreversible key-loss: social recovery, passkey backup, MPC shares. ~200M smart wallets in 2026. | Solid, mainstream. Recovery finality is real; UX is web2-grade. | **adopt** |
| **Attestation** | EAS (Ethereum Attestation Service), W3C Verifiable Credentials | Signed claims about identities — "X vouches Y is licensed/over-18/KYC'd". | Plumbing is mature; the *portable-reputation* layer on top is nascent. | **adopt** |
| **Reputation** | OpenRank (EigenTrust, TEE-verified over restaking) | Sybil-resistant, context-specific reputation compute over the attestation graph. | Real, but **does not compose cleanly over KOTVA's own objects** — the disjoint truster/trustee keys (REP-1/REP-2) degrade the transitive engine to a near-non-transitive tally, and distrust is dropped, not aggregated ([`primitives/REPUTATION.md § 9`](../primitives/REPUTATION.md)). Global reputation quality also still depends on the personhood anchor below. | **adopt (degraded — see REPUTATION §9)** |
| **Proof-of-personhood** | World ID **and** a structurally-different second anchor (e.g. a zk-passport binding such as Human Passport / OpenPassport) — **interop with ≥2 structurally-different personhood bindings is a v0 target**, not a future swap; see below | The anti-Sybil anchor under reputation. | **Imperfect, and single-vendor today for the primary anchor** — World ID is one commercial vendor operating under active, multi-jurisdiction biometric-privacy regulatory pressure. Every method trades off (biometrics+operator excludes differently than passport-zk, which excludes the undocumented). Local scale uses web-of-trust instead. | **adopt (ceiling)** |
| **Payments / settlement** | x402 (HTTP-402 stablecoin, Coinbase+Cloudflare); USDC on Base/Solana | Money rail + micropayments + agentic payments. | Infra is real (165M+ txns); *genuine* demand still thin (~$28k/day real). | **adopt** |
| **Streaming / subscriptions** | Payment channels (Lightning/state channels), Superfluid-class streams | Recurring + streaming money over the settlement rail. | Works; recurring-pull UX still rough. | **adopt** |
| **Storage — hot** | Walrus (Sui; erasure-coded, CDN-like, ~1/5 cloud cost) | Fast retrieval for media/blobs — the Evermesh CDN answer. | Newer; behaves like a CDN, not an archive. | **adopt** |
| **Storage — permanent** | Arweave | Permanent data layer (records, provenance). | Mature for permanence. | **adopt** |
| **Storage — scale** | Filecoin (+ Storj for erasure-coded speed) | Bulk durability market. | Mature; retrieval historically slow (compose with Walrus). | **adopt** |
| **Dispute / arbitration** | Kleros-class (staked schelling-point juries) | Neutral judge for commerce/gig disputes. | Real but small/unproven at scale; re-centralizes toward pro jurors. | **adopt (ceiling)** |
| **Media transport** | WebRTC (RFC 8825 stack) | Real-time voice/video byte transport. | Mature, ubiquitous. | **adopt** |
| **Media E2E encryption** | SFrame (RFC 9605), keyed from the MLS epoch | End-to-end encrypted media → media relays are `blind`/structural. | Standardised; DMTAP §27 keys it from MLS. | **adopt** |
| **NAT-relay for media** | TURN (RFC 5766 / 8656); coturn | Media relay when direct/STUN fails — content-blind over SFrame. | Mature standard. Served by the `media-relay` role. | **adopt** |
| **Distributed SFU (scale calls)** | LiveKit (Pion) / Jitsi Videobridge + Octo / mediasoup | Multi-provider, cascaded media relay so the host isn't the size limit. | Mature open-source. Bound, not built. | **adopt** |
| **Mesh reachability** | libp2p — Circuit Relay v2, DCUtR, Kademlia, Noise, QUIC | Key-addressed reachability behind CGNAT; content-blind relay. | Mature; DMTAP §4 profiles it directly. | **adopt** |
| **Messaging crypto** | MLS (RFC 9420) | Group ratchet for 1:1, chat, files, and (via SFrame) media keys. | Standards-track, mature. | **adopt** |
| **Verifiable coordination** | TEEs (Intel SGX / AMD SEV / ARM) | Lets a coordinator hold a global view *without* seeing plaintext or being able to cheat — the `attested` assurance level. | **New trust dependency** — trades operator-trust for chip-vendor-trust; side-channel history. Disclosed, not trustless. | **adopt (disclosed)** |

---

## Personhood — v0 requires ≥2 structurally-different bindings

The proof-of-personhood row above names **World ID** first, but a single commercial biometric
vendor — under active, multi-jurisdiction regulatory pressure over biometric data — is not an
anchor KOTVA can responsibly stake reputation portability on alone. **v0 target, not a
future-swap option:** interoperate with **at least two structurally-different personhood
bindings** — for example a biometric anchor (World ID) **and** a zk-passport anchor (Human
Passport / OpenPassport) — so no single vendor's regulatory exposure, outage, or policy change
can strand a subject's reputation. Each method keeps its own honest exclusion tradeoff, and
adding a second binding discloses that rather than resolving it: biometric enrolment excludes
those unwilling or unable to enrol; passport-zk excludes the undocumented. Concretely: a
conformant implementation accepts anchors from at least two structurally-different bindings,
never just the one named first — the normative acceptance rule belongs in the profile that
consumes the anchor, not in this binding index.

A crypto-anchored personhood proof also assumes a wallet-holding, key-managing user. **A
non-crypto, day-one recovery/personhood path is required for mainstream adoption** — most
users hold no wallet and have signed no key. This is a stated v0 requirement, not yet a
specified binding; it slots in as a future binding swap without a rearchitecture
([`DIRECTION § 9`](../DIRECTION.md), "future-proof by seams").

---

## What KOTVA does **not** bind — and never will

- **A protocol token.** Money is a stablecoin; trust is *staked existing value*. No mint.
  ([DIRECTION §5](../DIRECTION.md).)
- **A global reputation *score* we compute.** Reputation is locally measured, or bound to
  OpenRank as compute — never a single authority's published number.
- **A search *ranking* token.** Search is a coordinator (indexer, TEE-preferred), not a token.
- **Surveillance-based advertising.** Rejected by design.

---

## Sources (2026 survey)

- Proof of personhood — [Human Passport](https://passport.human.tech/), [World ID](https://world.org/blog/world/proof-of-personhood-what-it-is-why-its-needed)
- Account abstraction / recovery — [EIP-7702 smart wallets](https://www.altrady.com/blog/cryptocurrency/account-abstraction-eip-7702-smart-wallets-2026), [smart-wallet recovery 2026](https://eco.com/support/en/articles/15254048-smart-wallet-recovery-2026-social-multisig-passkey-options)
- Reputation / attestation — [OpenRank](https://github.com/openrankprotocol/openrank-tee), [EAS](https://attest.org/)
- Payments — [x402 explainer](https://www.rzlt.io/blog/agentic-payments-2026-x402-explainer), [x402 demand reality-check](https://www.coindesk.com/markets/2026/03/11/coinbase-backed-ai-payments-protocol-wants-to-fix-micropayment-but-demand-is-just-not-there-yet)
- Search (TEE) — [DeSearch (USENIX)](https://www.usenix.org/conference/osdi21/presentation/li)
- Storage — [Walrus paper](https://arxiv.org/pdf/2505.05370), [storage comparison 2026](https://www.securities.io/decentralized-storage-filecoin-arweave-storj-comparison/)
