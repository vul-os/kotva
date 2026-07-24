# The Coordinator Contract

**Status:** draft (the keystone new spec of the KOTVA family). Normative once ratified.

A **coordinator** is any party that provides a function the peer-to-peer substrate cannot
provide reciprocally — a global view, a scarce resource, or a legal/real-world anchor. This
document is the single contract every coordinator honours, so that "some centralisation, done
safely" is a *checkable property* and not a hope. It generalises DMTAP's legacy-mail gateway
(§7) and legacy adapters (§26) into one reusable shape; both are instances of it.

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, MAY are BCP 14.

---

## 1. Why coordinators exist (and why they are fenced)

The peer substrate handles custody, transport, and single-writer-per-resource state without a
coordinator. Four jobs it cannot do reciprocally:

- **Global-view optimisation** — matching, ranking, search.
- **Scarce-resource egress** — a reputable IP + unblocked port 25 (mail), a public address.
- **Real-world / legal anchoring** — licensing, liability, physical-event attestation.
- **Cross-party judgement** — moderation labels, dispute arbitration.

Each is a place centralisation can regrow. The contract confines the damage: a coordinator is
**hired, not depended-on**. No coordinator is load-bearing, so none can become a gatekeeper.
The sole exception is the **custodial escrow operator**, which holds the float for a trade
window and is therefore structurally load-bearing and does not fade — disclosed here rather
than hidden (see primitives/ESCROW.md).

---

## 2. The four conformance clauses (normative)

Every coordinator MUST satisfy all four.

### 2.1 Accountable
A coordinator MUST have an **attested identity** (a substrate keypair,
[substrate/IDENTITY.md](../substrate/IDENTITY.md)) and MUST
publish a **signed descriptor** carrying its kind, its policy, and — where it charges — a
signed tariff. The descriptor is **discovery-only and self-asserted**: it carries no global
reputation score, no price ranking, and no stake field. Reputation is **locally measured** by
each client from its own results, never a globally published number (DMTAP §7.5, generalised).
Stake is kept out of the descriptor for the same reason price-ranking is — see §6 for where and
how a relying party verifies it instead. **Wire form:** the signed descriptor is
`CoordinatorDescriptor`, the signed tariff is `Tariff`, and a metered coordinator's usage receipt
(§6) is `UsageReceipt` — deterministic-CBOR CDDL, a DS-tag, and a signing preimage for each, at
18-wire-format.md §18.8a. This clause was, for several revisions, checkable only by reading prose;
it is now byte-checkable by two independent implementations.

### 2.2 Swappable
Leaving a coordinator MUST be a **configuration change with zero data migration and zero
identity change**. A user's keys, mailbox, listings, and history live at the edge; a
coordinator holds only rebuildable operational state. Switching or dropping one MUST NOT
require re-telling correspondents or re-establishing identity. Lock-in is a conformance
violation, not a business model.

### 2.3 Self-hostable
For every coordinator kind there MUST exist a **self-host backstop**: a user who can meet the
kind's requirement can always run it for themselves and depend on no third party. **Two** honest
exception **classes** are disclosed rather than papered over, each a distinct kind of scarcity
controlled by a third party, never the user:

1. **A scarce network-reachability class**, with two members: a reputable IP + unblocked port 25
   for legacy SMTP egress (the `gateway`), and a public reachable ingress for the
   `reachability-adapter`. Both members are a network resource a third party (ISP/host) allocates,
   not something a user can always self-provision.
2. **A regulatory-licensing class**, with one member: `custodial-escrow` (§5, §6;
   [primitives/ESCROW.md](../primitives/ESCROW.md) SEC-6a). Holding a stranger's money for a trade
   window is, in most jurisdictions, a licensed and bonded activity — a wall an unlicensed
   individual cannot self-provision **at any technical skill level**, unlike every other kind's
   requirement, which a sufficiently motivated user can always meet with hardware, bandwidth, or
   software alone. Disclosed here rather than hidden: an ordinary user cannot self-host custodial
   escrow, so the self-host backstop's honest limit for this one kind is a **competitive market of
   licensed operators**, never a solo option. This is a distinct fact from §1's disclosure that
   `custodial-escrow` is also the family's one structurally load-bearing exception (§2.2's
   swappability, not this clause's self-hostability) — related, but not the same MUST.

The contract confines the scarcity to these two disclosed classes instead of letting either spread
to custody, naming, or moderation. Every other coordinator kind, including every non-custodial
member of `arbiter`/`oracle`, clears the ordinary self-host bar **at the scope its profile actually
requires**.

**Honest limit on that claim, at global scope.** For `indexer` and `matcher` the bar is
scope-dependent, and the blanket reading oversells it. Crawling and serving *the whole network's*
public objects is a bandwidth-and-storage burden closer to §2.3's scarce resources than to
something "a sufficiently motivated user can always meet with hardware, bandwidth, or software
alone". The AT Protocol relay is the deployed precedent: running an independent full-network
instance required real dedicated infrastructure investment rather than ordinary self-hosting,
despite a protocol that nominally permits anyone to run one. KOTVA's mitigation is genuine but
partial — SEARCH is following-graph-first and indexer-optional
([profiles/search.md](../profiles/search.md)), so the expensive global index is never load-bearing
and a user always retains the local-scope version, which is a real architectural difference from a
design where the relay is closer to required infrastructure. The self-host backstop for
`indexer`/`matcher` is therefore guaranteed **at local / following-graph scope**, and is *not*
claimed at whole-network scope. This is the self-hostability face of the discovery
re-centralisation open problem ([DIRECTION §8](../DIRECTION.md)), stated here because §2.3's
blanket sentence would otherwise read as covering a case the evidence says it does not.

### 2.4 Content-visibility declared
Every coordinator MUST declare, in its descriptor, exactly one **visibility class** at one
**assurance level** (§3), and MUST surface it to users. Advertising one class while operating
another is non-conformant misrepresentation, not policy.

---

## 3. The content-visibility property (normative)

This formalizes what §4/§7/§26 already gesture at, into a first-class, declared, checkable
property of every intermediary.

### 3.1 Visibility class — declare exactly one

| Class | Meaning | Examples |
|---|---|---|
| **`blind`** | Forwards/holds ciphertext, holds no key that decrypts the payload, reads neither content nor routing beyond what the wire exposes. | mesh relay (Circuit Relay v2), mix, TURN-over-SFrame |
| **`blind-routing`** | Cannot read the payload; sees routing metadata (envelope, SNI, addresses, size, timing). | SNI-passthrough ingress, buffer/mailbox, SFU media-relay (RFC 9605 — reads per-frame metadata, RTP routing, stream sizes, speaker timing, and participant graph to make forwarding decisions; media payload stays sealed by SFrame) |
| **`terminating`** | Terminates encryption and sees plaintext — a deliberate, disclosed trust boundary. | legacy mail gateway, TLS-terminating ingress |

### 3.2 The four rules
- **MUST declare** its class (no silent default).
- **No silent downgrade** — a coordinator that *can* run `blind`/`blind-routing` MUST NOT run
  `terminating` without explicit disclosure (cf. the TLS no-downgrade rule, DMTAP §7.2).
- **Client MUST surface** the class — the user sees which trust boundary a path crosses.
- **Prefer-blind default** — where a function can be served blind (SNI-passthrough vs.
  TLS-terminating), `blind` is RECOMMENDED and `terminating` requires opt-in + disclosure.

### 3.3 Assurance level — how blindness is guaranteed
"Blind" is not one strength. A coordinator SHOULD state which level it offers:

| Level | Guarantee | Strength |
|---|---|---|
| **`structural`** | The role *has no key* — E2E encryption makes reading impossible. | strongest; provable |
| **`attested`** | The role runs in a **TEE** that proves the code only forwards and holds no key. | hardware-trust |
| **`declared`** | The operator *promises* it is blind; nothing structurally prevents cheating. | honest-trust |

### 3.4 Honest residual (normative disclosure)
The contract can mandate the *architecture* that makes blindness possible (E2E encryption so
the coordinator **cannot** read) and the *declaration* rules above. It **cannot**
cryptographically prove a `declared`-level operator is not secretly logging — only
`structural` and `attested` are verifiable. A visibility declaration proves a coordinator's
architecture and intent, **never** that a `declared`-level operator is honest (DMTAP §7.10.4,
generalised). Clients MUST NOT present a `declared`-level `blind` claim as if it were verified.

---

## 4. Authorize, never classify (normative)

Every gate a coordinator applies **on a delivery path or a canonical/authoritative path** MUST
be an **authorisation** question answered from **sender identity and rate** — *is this party who
they claim, and within their limits?* On that path, a coordinator MUST NOT run content
classification (spam scoring, ML filters, keyword/URL reputation) and MUST NOT drop, quarantine,
re-rank, or annotate on a content basis. "Wanted" is a property of a relationship, judged by the
recipient, on the recipient's device, against the recipient's own corpus.

This is structural, not a preference: a coordinator that classifies content on a delivery or
authoritative path becomes permanent by construction (classification improves with corpus size,
so it centralizes, and it never "finishes"). The rule is what stops anti-abuse from forming a
second centralised tier (DMTAP §7.11.4, generalised to every coordinator).

**Anti-abuse that _is_ allowed:** authenticated-by-default identity, anonymous-but-accountable
rate-limit tokens, optional postage/proof-of-work for cold contact, and — for moderation — a
**market of opt-in labelers** each of which is itself a coordinator under this contract (it
labels; you subscribe to the ones you trust; you can leave).

**Derived-view carve-out (labelers, indexers, matchers):** a coordinator MAY classify, annotate,
rank, or re-rank content within its **own derived, non-authoritative, opt-in, subscribable
view** — this is exactly what the `labeler` kind does (it classifies content into labels; you
subscribe to the ones you trust) and what `indexer`/`matcher` do (they rank their own corpus or
match set). The §4 prohibition is scoped to a **delivery path** or a **canonical/authoritative
path**: a coordinator MUST NOT classify, annotate, drop, gate, quarantine, or re-rank content
that reaches — or is withheld from — a recipient by default. "Classify/rank my own opt-in view"
is permitted; "gate what reaches you" is not.

---

## 5. Coordinator kinds (all instances of the contract)

**This table is the single canonical, authoritative list of coordinator kinds for the entire
KOTVA family.** It names **twelve** kinds — ten fully-specified, one (`compute`) provisional, and
one (`infra-service`) draft ([profiles/cloud.md](../profiles/cloud.md)) — and no other document may
enumerate a different count or add a kind not listed here; a document that needs to talk about "how
many coordinator kinds exist" cites this table rather than re-deriving its own tally.

| Kind | Provides | Typical visibility |
|---|---|---|
| **gateway** | Legacy-mail bridge (MX, DKIM egress, legacy client surfaces) | `terminating` (legacy leg is plaintext) |
| **relay** | Mesh reachability for NAT'd peers | `blind` / structural |
| **media-relay** | Forwards SFrame-encrypted call/stream media (scales calls) | `blind-routing` / structural — media payload sealed by SFrame; per-frame metadata, RTP routing, size, timing, participant graph are visible to the SFU (RFC 9605) |
| **reachability-adapter** | ngrok-style public subdomains for arbitrary box services | `blind-routing` (SNI-passthrough) preferred |
| **infra-service** *(draft, [profiles/cloud.md](../profiles/cloud.md))* | Managed infrastructure — `box` / `bucket` / `edge-fn` / `database` / `cdn` via the DEPOT service registry (`compute` is the general provisional case; DEPOT `edge-fn` is its managed-serverless profiling) | **per service** — `bucket` `blind`/structural, public `cdn` `blind-routing`, `edge-fn`/`database`/`box` `terminating` (→ `attested` in a TEE) |
| **indexer** | Search / discovery / global product-and-price view | corpus is public plaintext (nothing to be blind about); query-channel `terminating` unless `attested` |
| **labeler** | Moderation labels, opt-in, subscribable | n/a (labels public objects) |
| **matcher** | Real-time supply↔demand matching (rides, delivery) | **terminating** (default) / **attested** (TEE) |
| **compute** *(provisional)* | Hosted/outsourced computation (e.g. private-AI inference on rented GPU) | `terminating` (default) / `attested` (TEE, for blind compute) |
| **arbiter** | Dispute resolution (staked jury) | `terminating` for evidence, disclosed |
| **oracle** | Physical-world / real-fact attestation (delivered? ride done?) | `terminating`, disclosed |
| **custodial-escrow** | Holds the trade float for a trade window ([primitives/ESCROW.md](../primitives/ESCROW.md) SEC-6a) | `terminating` for evidence, disclosed — the family's **one load-bearing exception** (§1) |

`gateway` (DMTAP §7) and the legacy `adapter`s (§26) are the first, fully-worked instances;
every kind above inherits the four clauses and the visibility property unchanged. `custodial-escrow`
satisfies all four clauses like every other kind but, uniquely, does not fade once hired — see §1
and [primitives/ESCROW.md](../primitives/ESCROW.md) §9–§10 (SEC-6a, "the one honest load-bearing
exception").

---

## 6. Economics (normative boundary)

- **In scope:** the authorisation *mechanism*, the visibility declaration, the descriptor and
  signed tariff, and — where a coordinator meters — **signed usage receipts** delivered
  directly to the paying party (auditable, never merely asserted). Wire form: `CoordinatorDescriptor`,
  `Tariff`, and `UsageReceipt` — deterministic-CBOR CDDL, a DS-tag, and a signing preimage for each
  (18-wire-format.md §18.8a).
- **Out of scope (operator policy):** the *numbers* — quotas, rate limits, prices — and the
  settlement rail (an existing stablecoin or fiat; KOTVA brokers none and takes no cut).
- **Charge for service, never for deliverability or classification.** A signed tariff and a signed
  usage receipt price the *work a coordinator actually does* — relaying, indexing, computing,
  arbitrating, holding a float. §4's authorise-never-classify rule already forbids gating,
  dropping, or re-ranking on a content basis; §6 closes the economic half of the same rule: a
  coordinator MUST NOT price whether a message is delivered, or how it is classified, only the
  service performed. A tariff that varies by content rather than by service is non-conformant
  under §4, not merely a pricing choice.
- **Stake verification (where a kind requires stake).** §2.1 excludes a stake field from the
  descriptor so stake cannot become a ranking signal. Where a kind carries skin-in-the-game
  (`arbiter`, `oracle` — DIRECTION §5, "sized to the value at risk" — and, where bonded per
  [primitives/ESCROW.md](../primitives/ESCROW.md) SEC-6a, `custodial-escrow`'s float bond), the stake
  MUST instead be verifiable **on the settlement/staking rail itself** — e.g. an on-chain stake
  balance or lock a client can query directly (Kleros-class staked arbitration, OpenRank-class
  staked attestation, DIRECTION §3) — never merely asserted in the descriptor or taken on faith. A
  client relying on a staked coordinator MUST verify the on-rail stake meets the value at risk
  before relying on it; an unverifiable stake claim MUST be treated as no stake (SEC-1, fail
  closed). This bar is uniform across every staked kind, `custodial-escrow` included: §2.3's
  disclosed licensing exception excuses the *self-host* requirement, never the *verify-before-
  relying* one — a licensed custodial operator is held to the same fail-closed stake check as an
  `arbiter`/`oracle`, not a lighter one.
- **Never:** a protocol token, an advertising mechanism, or a payout-fairness scheme. Absent
  by decision.

A user's audit is **one-directional** (DMTAP §7.9, generalised): a signed receipt lets a user
confirm a claimed operation was real; it cannot disconfirm an operation the coordinator
fabricated. Disclosed, not hidden.

**Honest residual: whether this funds anyone.** Charge-for-service is the mechanism this contract
makes normative; whether it is *sufficient* — whether a coordinator can sustain itself charging
only for service, at a price a user will pay, without ever charging for deliverability or
classification — is an open question this specification does not resolve and disclosed as such,
not assumed away.

---

## 7. Conformance checklist

| # | A coordinator… | Clause |
|---|---|---|
| COORD-1 | publishes a signed, discovery-only descriptor with no global score/price-rank/stake | §2.1 |
| COORD-2 | imposes zero lock-in — switching is config-only, identity unchanged | §2.2 |
| COORD-3 | has a self-host backstop (or discloses one of the two exception classes: scarce reachability, or — `custodial-escrow` only — regulatory licensing) | §2.3 |
| COORD-4 | declares exactly one visibility class + assurance level, and clients surface it | §2.4, §3 |
| COORD-5 | never silently downgrades from blind to terminating | §3.2 |
| COORD-6 | authorises from identity + rate on any delivery/authoritative path; classifies only within opt-in derived views | §4 |
| COORD-7 | if metered, issues signed usage receipts directly to the payer | §6 |
| COORD-8 | mints no token; stakes/settles only in existing assets | §6, DIRECTION §5 |
| COORD-9 | prices service performed only, never deliverability or classification | §4, §6 |
| COORD-10 | where staked, the stake is verified on-rail before relying on it (an unverifiable stake = no stake) — applies uniformly, `custodial-escrow` included | §6 |
