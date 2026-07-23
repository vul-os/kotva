# DIRECTION — the rules KOTVA is built to

This file is the **general direction** for the whole family. It is not a wire spec; it is the
set of principles every profile, every binding, and every coordinator must stay consistent
with. When a design decision is unclear, it is resolved *here* first, then in the relevant
spec. The reasoning behind these rules is in [`docs/research/`](docs/research/README.md).

---

## 0. The one rule

> **Decentralize the substrate and the exit. Every unavoidable coordinator is _accountable,
> swappable, and self-hostable_, and _never load-bearing_. Coordinators add reach; they never
> gate function.**

Everything below is a consequence of this one rule. It is DMTAP's legacy-mail-gateway model
(§7 — one accountable operator class, swappable with a DNS change, with a self-host backstop)
generalized from mail to the whole system.

---

## 1. The narrow waist

KOTVA is small on purpose. It "covers everything" the way IP or HTTP do — by being a **thin
waist** that composes upward and downward, **not** by containing everything. The waist is six
capabilities:

| Capability | What it is |
|---|---|
| **Identity** | A keypair is the identity. Names (DNS / chain / key-name floor) are swappable pointers. |
| **MOTE** | The universal object: signed, encrypted, content-addressed. Mail, chat, offers, feed entries, credentials are all MOTEs. |
| **Transport** | Reach anyone by key — online, offline, or over a mesh. Store-and-forward at the edge. |
| **PUB** | Signed public objects + author feeds — authenticity without confidentiality. |
| **SYNC** | Multi-author signed CRDT — shared mutable state with no server. |
| **Roles & Wake** | Infrastructure roles any node may take; content-free push to offline nodes. |

Above the waist sit **profiles** (mail, commerce, work, social, media, calling). To one side
sit **coordinators** (§4). Below sit **adopted standards** (§3). Keep the waist thin: a new
capability enters it only if *most* profiles need it.

---

## 2. Primitives, and why services collapse into them

Every real-world service is the same small set of primitives rearranged:

```
OFFER · MATCH / RESERVE · REPUTATION · ESCROW · ORACLE · DISPUTE · PAY
```

- **Uber, delivery, freelance, auctions** = OFFER + MATCH + REPUTATION + ESCROW + ORACLE +
  DISPUTE + PAY, differing only in MATCH's **assignment rule** (nearest / best-fit /
  highest-bid). *One matching engine, not one per service.*
- **Bookings** need no matcher at all — RESERVE against a single-owner calendar (the host's
  box is the only writer, so double-booking is structurally impossible).
- **Commerce / classifieds** = OFFER + ESCROW + REPUTATION + DISPUTE + PAY.

Because the services reduce to the same primitives, building the primitives once builds the
world. This is why "different products" are thin profiles, not separate systems.

---

## 3. Bind, don't reinvent

KOTVA **adopts** existing, proven standards wherever one exists, and specifies new bytes only
where nothing does. Adoption is a thin binding document, never a re-derivation. The full
index is [`bindings/README.md`](bindings/README.md). The short version:

| Need | Adopt | Not |
|---|---|---|
| Identity recovery | Account abstraction (ERC-4337 / EIP-7702), passkeys, MPC | a bespoke recovery scheme |
| Attestation | EAS / W3C Verifiable Credentials | a new credential format |
| Reputation | OpenRank (EigenTrust, TEE-verified) | a global score we compute |
| Personhood | World ID / Human Passport | our own biometrics |
| Payments | x402 + stablecoins | **a protocol token (there is none, and none will be added)** |
| Storage | Walrus (hot) / Arweave (permanent) / Filecoin | a bespoke durability market |
| Dispute | Kleros-class arbitration | our own court |
| Media transport | WebRTC + SFrame (RFC 9605) + TURN | a new media stack |
| Mesh / messaging crypto | libp2p + MLS (RFC 9420) | a new transport or ratchet |

The only genuinely new normative writing KOTVA owns is: the **substrate**, the **coordinator
contract** (§4), and the thin **profiles**. Everything else is a pointer.

---

## 4. Coordinators — where centralization is allowed, and fenced

Some jobs genuinely need a party with a global view or a scarce/legal resource: matching,
search, moderation, legal accountability, physical-world attestation. KOTVA does **not**
pretend these away. It isolates each behind the **coordinator contract**
([`coordinator/CONTRACT.md`](coordinator/CONTRACT.md)), whose four clauses are non-negotiable:

1. **Accountable** — an attested identity and a signed, published policy/tariff.
2. **Swappable** — leaving is a config change with zero data migration and zero lock-in.
3. **Self-hostable** — a user who can meet the requirement can always serve themselves.
4. **Content-visibility declared** — every intermediary declares what it can see
   (`blind` / `blind-routing` / `terminating`) at an assurance level
   (`structural` / `attested` / `declared`).

A coordinator **authorizes; it never classifies**: it may check *who you are and your rate*,
never *whether your content is wanted*. That judgement is the recipient's, on the recipient's
device. This is the rule that stops anti-abuse from re-centralizing (DMTAP §7.11.4,
generalized).

Coordinator kinds are all instances of the same contract: **gateway** (legacy bridge),
**relay** / **media-relay** (content-blind forwarding), **reachability-adapter** (ngrok-style
subdomains), **indexer** (search / discovery), **labeler** (moderation), **matcher**
(real-time matching), **arbiter** (dispute), **oracle** (physical-world attestation).

---

## 5. Money and trust — no token, ever

- **Money is an existing stablecoin.** Custody and canonical settlement are the one thing
  KOTVA adopts wholesale. It mints nothing.
- **Trust is _staked existing value_,** never a native token. Where a coordinator needs
  skin-in-the-game (arbiter, oracle), the stake is denominated in an existing asset and sized
  to the value at risk.
- A **new protocol token is forbidden.** It is either a financing scheme in disguise or a
  coordination problem a token cannot actually solve. Free tiers, subsidies, and even ads are
  *operator policy* a coordinator may offer — optional, swappable, escapable by self-hosting —
  never a protocol requirement.

---

## 6. Scale-invariance — village to planet on the same primitives

The primitives are the same at every scale; only the **trust anchor** slides:

| Function | Small / mesh (offline, no coordinator) | Global (swappable coordinator) |
|---|---|---|
| Personhood | web-of-trust (you know these people) | a personhood attester you choose |
| Reputation | direct, local | indexer over the attestation graph |
| Matching | dumb local order book | global matcher-as-a-service |
| Discovery | following-graph + local index | competing indexers |
| Dispute | a known local arbiter | staked arbitration market |

The system is **coordinator-optional**: remove connectivity and every service collapses to
its local-trust version and *still works* (the offline / apocalypse-proof property). Add
connectivity and coordinators become *available* for global reach — never *required*.

---

## 7. Two modalities — async substrate, real-time parallel plane

- The **async world** (mail, chat, social, commerce, files) composes from MOTE + PUB + SYNC.
- The **real-time world** (voice, video, live streaming) rides a **parallel media plane**
  (WebRTC) that reuses the substrate's identity, keys (MLS→SFrame), roles, coordination, and
  signaling — but **not** the store-and-forward object model. Only the media *bytes* are on
  their own track, which is correct: real-time must not be forced through MOTE delivery.
- Scaling calls is a **media-relay role** (content-blind, because SFrame E2E-encrypts the
  media), a pool anyone can provide, coordinated by an existing distributed SFU. The host's
  hardware is not the size limit.

---

## 8. Honest ceilings — what we disclose rather than solve

KOTVA covers every service's **mechanism**; it **discloses** (never hides) each service's
trust and legal residual. Four root ceilings recur, and everything hard traces to one:

1. **Global anti-Sybil** — imperfect; local scale dissolves it into web-of-trust.
2. **Physical-event oracle** — "did the delivery/ride/work happen?" reduces to confirm +
   dispute; a coordinator can attest but cannot prove non-fabrication.
3. **Legal / authoritative-issuer** — land title, licensing, money-transmission need a real
   accountable party. The **paid-coordinator model absorbs this** (an operator holds the
   license for pay), but the burden does not vanish.
4. **Editorial governance** — "who decides the canonical version" for wikis, app stores,
   registries. Distinct from anti-Sybil; closest answer is a reputation-weighted curation
   coordinator.

Two things KOTVA genuinely **cannot** give, even in principle, and says so: **coercion-
resistant public-election voting** (harder than anti-Sybil) and **surveillance-based ad
markets** (rejected by design). Everything else composes.

---

## 9. The perfection rules — how this spec stays simple and future-proof

Direction for the ongoing work, so "perfect the spec" never becomes a sprawling project:

- **The stop rule.** Before any spec edit ask: *does an implementer need this to build or
  interoperate with what we actually ship?* If not — if it polishes prose, adds a far-future
  layer, or re-litigates a settled decision — don't write it.
- **Quarantine research.** Far-future cryptography that is unproven or unsound (mixnet,
  VDF, PQ envelope tuning) lives in `research/` as **non-normative**. This removes it from the
  critical path *and* stops the spec overclaiming guarantees the implementation doesn't meet.
- **Pay wire debt before prose.** Normative MUSTs must be backed by wire definitions. Write
  the missing CDDL (e.g. `GatewayAuthz`) before writing more requirements that reference it.
- **Future-proof by seams, not by prediction.** Every hard problem is a *pluggable slot*
  (a binding or a coordinator). When the frontier improves (better personhood, TEE matching),
  swap the filling — the substrate and profiles never change. The product converges on
  centralized quality while keeping sovereignty, with no rearchitecture.
- **Simple by subtraction.** Prefer adopting a standard over specifying one; prefer a profile
  over a new waist capability; prefer a coordinator role over a new primitive. The smallest
  design that composes is the correct one.
