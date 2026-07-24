# Research & rationale behind KOTVA

This is the reasoning the design rests on: the coverage audit (what the substrate covers and
what it doesn't), the drawback analysis versus centralised products, the 2026 tooling survey,
and the future-proofing argument. It exists so the [DIRECTION](../../DIRECTION.md) rules are
*derived*, not asserted.

---

## 1. Coverage — does one substrate really cover the world?

We tested the substrate against ~35 real-world services (communication, money, the gig
cluster, discovery, infrastructure, civic, media, AI). Result:

- **~29 covered** (a handful with disclosed ceilings) — because they reduce to the same
  primitives: `OFFER · MATCH/RESERVE · REPUTATION · ESCROW · ORACLE · DISPUTE · PAY`.
- **~3 hard-blocked** — coercion-resistant **voting**, uncollateralized **credit**,
  programmatic **ad markets** (the last is also a non-goal).
- **~2 governance-gap** — Wikipedia-grade authoritative knowledge and app-store/registry
  curation (the "who decides the canonical version" problem).
- **~1 unbuilt** — a lightweight IoT profile.

The elegant finding: Uber, delivery, bookings, auctions, freelance, and classifieds are the
*same* system wearing different UIs. They differ only in MATCH's assignment rule (nearest /
highest-bid / best-fit), or drop MATCH for RESERVE (single-owner bookings). Building the
primitives once builds the space — which is why KOTVA is a substrate, not a product suite.

---

## 2. The four root ceilings

Every gap and disclosed residual traces to one of four things — not dozens of separate
problems:

1. **Global anti-Sybil.** Reputation, reviews, credit, ads, and the easy half of voting all
   lean on it. Local scale dissolves it into web-of-trust; global scale is imperfect (World
   ID / Human Passport raise the floor, don't close it).
2. **Physical-event oracle.** "Did the delivery / ride / work happen?" reduces to
   dual-confirm + dispute. A coordinator can attest origin-through-itself, never
   non-fabrication.
3. **Legal / authoritative-issuer.** Land title, licensing, money-transmission need a real
   accountable party. The **paid-coordinator model absorbs this** (an operator holds the
   license for pay) — the burden moves, it doesn't vanish.
4. **Editorial governance.** "Who decides the canonical version" for wikis, stores,
   registries. Distinct from anti-Sybil; closest answer is a reputation-weighted curation
   coordinator.

Two genuinely-unsolvable-in-principle, disclosed as such: **coercion-resistant public
voting** and **surveillance ad markets**.

---

## 3. Drawbacks vs. centralised products — and which are real

We catalogued the honest disadvantages, then re-graded them against 2026 tooling.

**Now solved (adopt off the shelf):**
key loss → account abstraction; onboarding friction → passkeys; local consistency → matured
CRDTs (Automerge 3.0 Rust core, Yjs, Jazz, cr-sqlite); storage/CDN → Walrus.

**Credible-but-unfinished:**
anti-abuse global-view → TEE coordinators (a labeler/indexer with a global view that cannot
read plaintext); search → TEE index (DeSearch); reputation → OpenRank; micropayments → x402
(infra real, demand thin).

**Genuinely fundamental (design around, don't fix):**
feature velocity (protocols evolve slower than a company ships); the ad-subsidized free tier
(no surveillance engine, by choice); legal liability sink; cold-start coordination.

**The reframe that resolves the fundamentals:** they are *jobs*, not walls. A centralised
company bundles them, makes them mandatory, and pays for them by owning you. KOTVA unbundles
them into **paid coordinator roles** that compete and that you can fire or self-host. You
re-buy the same services à la carte, from operators who cannot trap you. The only thing
genuinely surrendered is "free because they surveil you."

**The new residual we introduce:** TEEs (which close several gaps) trade operator-trust for
hardware-vendor-trust and have a side-channel history. Disclosed as the `attested` assurance
level, never sold as trustless ([coordinator/CONTRACT §3](../../coordinator/CONTRACT.md)).

---

## 4. Future-proofing — why the design converges on centralised quality

Every hard problem is a **swappable slot** — a binding or a coordinator behind a stable
interface. As the frontier improves, the fix slots in as a version bump, and nothing above it
changes:

| As this improves… | It enters as… | Product moves toward |
|---|---|---|
| Better proof-of-personhood | a REPUTATION binding swap | centralised-grade Sybil resistance |
| TEE global matcher | a MATCH coordinator swap | Uber-grade matching |
| TEE search index | an indexer coordinator swap | Google-grade discovery |
| TEE anti-abuse (global view, no plaintext) | a labeler upgrade | platform-grade T&S, privately |

So the **technical** gaps are future-proofed — they converge on centralised quality without
rearchitecture, while keeping sovereignty. What remains is the small **structural** set
(§3, fundamental), which no seam fixes because they aren't technical problems — they're
consequences of not being a single surveilling company, and several are the point.

---

## 5. What the substrate does and doesn't reinvent (grounding)

KOTVA's novelty is **composition and the coordinator contract**, not new cryptography. It
profiles libp2p (it does *not* reinvent P2P), applies MLS, adopts SFrame/TURN/WebRTC for
media, and binds identity/reputation/payments/storage/dispute to existing standards
([bindings](../../bindings/README.md)). The genuinely original writing is: the **substrate
waist**, the **coordinator contract** (safe centralisation as a checkable property), and thin
**profiles**. Far-future cryptographic research (mixnet/Sphinx, VDF, PQ envelope tuning) is
**quarantined to `docs/research/` as non-normative** — its assurance is not yet deployment-grade,
and keeping it out of the normative surface stops the spec overclaiming.

---

## 6. Primary-source survey (2026)

- **Personhood:** [Human Passport](https://passport.human.tech/) · [World](https://world.org/blog/world/proof-of-personhood-what-it-is-why-its-needed)
- **Recovery / account abstraction:** [EIP-7702](https://www.altrady.com/blog/cryptocurrency/account-abstraction-eip-7702-smart-wallets-2026) · [recovery patterns](https://eco.com/support/en/articles/15254048-smart-wallet-recovery-2026-social-multisig-passkey-options) · [invisible wallet / passkeys](https://cryptonium.cloud/articles/invisible-wallet-passkeys-account-abstraction-crypto-super-app-2026)
- **Reputation / attestation:** [OpenRank (TEE)](https://github.com/openrankprotocol/openrank-tee) · [EAS](https://attest.org/)
- **Payments:** [x402 explainer](https://www.rzlt.io/blog/agentic-payments-2026-x402-explainer) · [demand reality-check](https://www.coindesk.com/markets/2026/03/11/coinbase-backed-ai-payments-protocol-wants-to-fix-micropayment-but-demand-is-just-not-there-yet)
- **Search (TEE):** [DeSearch (USENIX)](https://www.usenix.org/conference/osdi21/presentation/li) · [YaCy](https://www.glukhov.org/post/2025/06/yacy-search-engine/)
- **Local-first / CRDT:** [local-first 2026](https://appscale.blog/en/blog/local-first-architecture-crdts-sync-engines-offline-first-2026) · [Automerge](https://github.com/automerge/automerge) · [Jazz](https://jazz.tools/blog/what-is-jazz)
- **Storage:** [Walrus paper](https://arxiv.org/pdf/2505.05370) · [comparison 2026](https://www.securities.io/decentralized-storage-filecoin-arweave-storj-comparison/)

*Caveat on all of the above: a survey is a snapshot. Maturity and demand claims are dated to
2026-07 and must be re-checked before any of these bindings is relied on in production.*
