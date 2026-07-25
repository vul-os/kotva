# DIRECTION — the rules KOTVA is built to

This file is the **general direction** for the whole family. It is not a wire spec; it is the
set of principles every profile, every binding, and every coordinator must stay consistent
with. When a design decision is unclear, it is resolved *here* first, then in the relevant
spec. The reasoning behind these rules is in [`docs/research/`](docs/research/README.md).

---

## 0. The one rule

> **Decentralise the substrate and the exit. Every unavoidable coordinator is _accountable,
> swappable, and self-hostable_, and _never load-bearing_. Coordinators add reach; they never
> gate function.**

Everything below is a consequence of this one rule. It is DMTAP's legacy-mail-gateway model
(§7 — one accountable operator class, swappable with a DNS change, with a self-host backstop)
generalised from mail to the whole system.

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

Every real-world service is the same small set of primitives rearranged. The primitive set is
exactly six:

```
OFFER · MATCH · RESERVE · REPUTATION · ESCROW · ATTEST
```

The **composite roles** that appear in service recipes — ORACLE, DISPUTE, PAY — are *not*
standalone primitives: **ORACLE** is the oracle coordinator kind (a physical-fact attestation,
i.e. ORACLE ⊂ ATTEST), **DISPUTE** is the arbiter coordinator kind, and **PAY** is the
payment-rail binding (fiat / stablecoin / token, §5). The recipe shorthand `OFFER · MATCH/RESERVE · REPUTATION · ESCROW · ORACLE
· DISPUTE · PAY` remains a useful *recipe* mnemonic, but it names roles, not the primitive set.

- **Uber, delivery, freelance, auctions** = OFFER + MATCH + REPUTATION + ESCROW + ORACLE +
  DISPUTE + PAY, differing only in MATCH's **assignment rule** (nearest / best-fit /
  highest-bid). *One matching engine, not one per service.*
- **Bookings** need no matcher at all — RESERVE against a single-owner calendar (the host's
  box is the only writer, so double-booking is structurally impossible **between honest
  participants** — against a dishonest owner it yields signed, attributable *evidence* rather than
  impossibility, [`primitives/RESERVE.md` §9](primitives/RESERVE.md)).
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
| Attestation | **ATTEST** is ours (a primitive); its claim body binds EAS / W3C Verifiable Credentials | a new credential format |
| Reputation | OpenRank (EigenTrust, TEE-verified) | a global score we compute |
| Personhood | World ID / Human Passport | our own biometrics |
| Payments | x402 + stablecoins, **fiat rails**, or any existing token — operator's choice (§5) | **a *protocol* token (there is none, and none will be added — but accepting an existing token or fiat is fine)** |
| Storage | Walrus (hot) / Arweave (permanent) / Filecoin | a bespoke durability market |
| Dispute | Kleros-class arbitration | our own court |
| Media transport | WebRTC + SFrame (RFC 9605) + TURN | a new media stack |
| Mesh / messaging crypto | libp2p + MLS (RFC 9420) | a new transport or ratchet |

The only genuinely new normative writing KOTVA owns is: the **substrate**, the **coordinator
contract** (§4), the **primitives** (the primitive specs — SYNC and MATCH's assignment-rule
vocabulary are the new normative ground), and the thin **profiles**. Everything else is a
pointer.

---

## 4. Coordinators — where centralisation is allowed, and fenced

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

A coordinator **authorises; it never classifies**: it may check *who you are and your rate*,
never *whether your content is wanted*. That judgement is the recipient's, on the recipient's
device. This is the rule that stops anti-abuse from re-centralising (DMTAP §7.11.4,
generalised).

Coordinator kinds are all instances of the same contract: **gateway** (legacy bridge),
**relay** (content-blind forwarding) / **media-relay** (`blind-routing` — SFrame-sealed
payload, but routing metadata visible, RFC 9605), **reachability-adapter** (ngrok-style
subdomains), **indexer** (search / discovery), **labeler** (moderation), **matcher**
(real-time matching), **compute** (rented blind inference, provisional), **arbiter** (dispute),
**oracle** (physical-world attestation), and **custodial-escrow** (the one load-bearing exception, confined to the commerce extension — Core v1 has none; [SPEC.md ratification tiers](SPEC.md)).
Canonical, exhaustive list:
[`coordinator/CONTRACT § 5`](coordinator/CONTRACT.md).

**`indexer` and `matcher` carry the sharpest version of this risk, and it is a named open
problem, not a solved one.** Both are "authorise, never classify" by contract — but search,
ranking, and match-assignment are exactly the functions through which real markets
re-concentrate power even under low switching costs; a fenced role does not, on its own, prove
the fence holds. See §8.

---

## 5. Money and trust — no *protocol* token, ever (but any settlement rail)

- **The settlement rail is the operator's generic choice; KOTVA mints and brokers none.** Money is
  whatever existing rail the paying and paid parties agree on — a **fiat rail** (card, bank/SEPA/ACH,
  a payment processor), a **stablecoin**, or an **existing token**. The protocol carries only signed
  payment *attestations* over that rail (TRACT §9, `UsageReceipt` §18.8a); it holds no funds, names no
  provider, and takes no cut. This is deliberately generic: a gateway runs **its own economic model**
  and settles however it likes — the protocol constrains *that a price exists, is signed, and is
  metered honestly*, never *what the price is or how it is paid*.
- **"No token" means no *protocol* token — not "no tokens."** KOTVA mints no native asset and none
  will be added: a native protocol token is either a financing scheme in disguise or a coordination
  problem a token cannot solve. An operator **accepting an existing token as payment** is just a
  settlement-rail choice, exactly like accepting fiat — permitted and unremarkable. The forbidden
  thing is the *protocol* minting or requiring one, not a coordinator taking crypto.
- **Trust is _staked existing value_,** never a native token. Where a coordinator needs
  skin-in-the-game (arbiter, oracle), the stake is denominated in an existing asset (fiat-bond,
  stablecoin, or token) and sized to the value at risk.
- **Bootstrapping is operator-layer, not protocol-layer.** Free tiers, subsidies, promotional
  credits, referral incentives — even an operator issuing *its own* loyalty token on its own rail —
  are *operator policy* a coordinator may offer: optional, swappable, escapable by self-hosting, never
  a protocol requirement. So the one cold-start lever deployed markets relied on (paying early supply
  before demand) **is available at the operator layer**; what is forbidden is baking it into the
  protocol as a native token or a mandated subsidy.
- **Open problem: coordinator-funding sustainability.** Whether charge-for-service alone — no
  token, no ads, zero lock-in, no paid classification — sustainably funds coordinators at scale
  is unproven: an open economic question, not a solved one
  ([`coordinator/CONTRACT § 6`](coordinator/CONTRACT.md)).

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
  signalling — but **not** the store-and-forward object model. Only the media *bytes* are on
  their own track, which is correct: real-time must not be forced through MOTE delivery.
- Scaling calls is a **media-relay role** (`blind-routing` — SFrame E2E-encrypts the media
  *payload*, though the relay still reads routing metadata to forward, RFC 9605), a pool anyone
  can provide, coordinated by an existing distributed SFU. The host's hardware is not the size limit.

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

One further item is a **named open problem** — a candidate fix exists but is unproven, unlike
the ceilings above, which have no fix even proposed (a second open problem, coordinator-funding
sustainability, is named at §5):

- **Discovery / indexer / matcher re-centralisation.** `indexer` and `matcher` are
  contractually authorise-only and swappable (§4), but that does not by itself stop the effect
  real markets show repeatedly: whoever runs search, ranking, or match-assignment accrues power
  over what gets found and who gets matched, however low the switching cost. The proposed
  structural answer — **verifiable-completeness indexing**, where an index proves (not merely
  discloses its policy) that it did not selectively omit or bury a result — has **no deployed
  precedent**. Named here as open, not claimed solved.

  **A second mechanism, which the coordinator contract does not reach at all: client defaults.**
  Nostr is the deployed precedent — its relays are genuinely swappable and permissionless, yet every
  major client ships a hardcoded default relay list, and that alone produced real concentration
  without any relay violating anything. The re-centralising actor there is the *client's* choice of
  default, not the coordinator's conduct, so no clause binding coordinators — accountability,
  swappability, self-hostability, declared visibility — constrains it. KOTVA inherits this exposure
  wherever a client must pick an `indexer`, `matcher`, or `relay` on a user's behalf. Client
  conformance is the only surface that could address it, and [§8.6b](08-clients.md) now carries that
  rule: a client selecting a coordinator on the user's behalf SHOULD make the choice inspectable and
  changeable in the running client and SHOULD seed from more than one provider, and MUST NOT hardcode
  a coordinator the user can neither see nor replace. This buys **real, visible exit** and lowers the
  switching cost; it does not equalise default weight — most users never change a default — so the
  *structural* answer (verifiable-completeness indexing above) stays open. Exit is bound; gravity is
  not.

---

## 9. The perfection rules — how this spec stays simple and future-proof

Direction for the ongoing work, so "perfect the spec" never becomes a sprawling project:

- **The stop rule.** Before any spec edit ask: *does an implementer need this to build or
  interoperate with what we actually ship?* If not — if it polishes prose, adds a far-future
  layer, or re-litigates a settled decision — don't write it.
- **Quarantine research.** Far-future cryptography that is unproven or unsound (mixnet,
  VDF, PQ envelope tuning) lives in `docs/research/` as **non-normative**. This removes it from the
  critical path *and* stops the spec overclaiming guarantees the implementation doesn't meet.
- **Pay wire debt before prose.** Normative MUSTs must be backed by wire definitions. Write
  the missing CDDL (e.g. `GatewayAuthz`) before writing more requirements that reference it.
- **Future-proof by seams, not by prediction.** Every hard problem is a *pluggable slot*
  (a binding or a coordinator). When the frontier improves (better personhood, TEE matching),
  swap the filling — the substrate and profiles never change. The product converges on
  centralised quality while keeping sovereignty, with no rearchitecture.
- **Simple by subtraction.** Prefer adopting a standard over specifying one; prefer a profile
  over a new waist capability; prefer a coordinator role over a new primitive. The smallest
  design that composes is the correct one.
