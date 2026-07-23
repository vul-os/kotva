# The Coordinator Contract

**Status:** draft (the keystone new spec of the KOTVA family). Normative once ratified.

A **coordinator** is any party that provides a function the peer-to-peer substrate cannot
provide reciprocally — a global view, a scarce resource, or a legal/real-world anchor. This
document is the single contract every coordinator honors, so that "some centralization, done
safely" is a *checkable property* and not a hope. It generalizes DMTAP's legacy-mail gateway
(§7) and legacy adapters (§26) into one reusable shape; both are instances of it.

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, MAY are BCP 14.

---

## 1. Why coordinators exist (and why they are fenced)

The peer substrate handles custody, transport, and single-writer-per-resource state without a
coordinator. Four jobs it cannot do reciprocally:

- **Global-view optimization** — matching, ranking, search.
- **Scarce-resource egress** — a reputable IP + unblocked port 25 (mail), a public address.
- **Real-world / legal anchoring** — licensing, liability, physical-event attestation.
- **Cross-party judgement** — moderation labels, dispute arbitration.

Each is a place centralization can regrow. The contract confines the damage: a coordinator is
**hired, not depended-on**. No coordinator is load-bearing, so none can become a gatekeeper.

---

## 2. The four conformance clauses (normative)

Every coordinator MUST satisfy all four.

### 2.1 Accountable
A coordinator MUST have an **attested identity** (a substrate keypair, §Identity) and MUST
publish a **signed descriptor** carrying its kind, its policy, and — where it charges — a
signed tariff. The descriptor is **discovery-only and self-asserted**: it carries no global
reputation score, no price ranking, and no stake field. Reputation is **locally measured** by
each client from its own results, never a globally published number (DMTAP §7.5, generalized).

### 2.2 Swappable
Leaving a coordinator MUST be a **configuration change with zero data migration and zero
identity change**. A user's keys, mailbox, listings, and history live at the edge; a
coordinator holds only rebuildable operational state. Switching or dropping one MUST NOT
require re-telling correspondents or re-establishing identity. Lock-in is a conformance
violation, not a business model.

### 2.3 Self-hostable
For every coordinator kind there MUST exist a **self-host backstop**: a user who can meet the
kind's requirement can always run it for themselves and depend on no third party. Exactly one
honest exception is disclosed rather than papered over — legacy SMTP egress requires a
resource (reputable IP, unblocked port 25) an ISP may deny — and the contract confines that
scarcity to that one kind instead of letting it spread.

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
| **`blind`** | Forwards/holds ciphertext, holds no key that decrypts the payload, reads neither content nor routing beyond what the wire exposes. | mesh relay (Circuit Relay v2), mix, SFrame media-relay, TURN-over-SFrame |
| **`blind-routing`** | Cannot read the payload; sees routing metadata (envelope, SNI, addresses, size, timing). | SNI-passthrough ingress, buffer/mailbox |
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
generalized). Clients MUST NOT present a `declared`-level `blind` claim as if it were verified.

---

## 4. Authorize, never classify (normative)

Every gate a coordinator applies MUST be an **authorization** question answered from **sender
identity and rate** — *is this party who they claim, and within their limits?* A coordinator
MUST NOT run content classification (spam scoring, ML filters, keyword/URL reputation) and
MUST NOT drop, quarantine, re-rank, or annotate on a content basis. "Wanted" is a property of
a relationship, judged by the recipient, on the recipient's device, against the recipient's
own corpus.

This is structural, not a preference: a coordinator that classifies content becomes permanent
by construction (classification improves with corpus size, so it centralizes, and it never
"finishes"). The rule is what stops anti-abuse from forming a second centralized tier (DMTAP
§7.11.4, generalized to every coordinator).

**Anti-abuse that _is_ allowed:** authenticated-by-default identity, anonymous-but-accountable
rate-limit tokens, optional postage/proof-of-work for cold contact, and — for moderation — a
**market of opt-in labelers** each of which is itself a coordinator under this contract (it
labels; you subscribe to the ones you trust; you can leave).

---

## 5. Coordinator kinds (all instances of the contract)

| Kind | Provides | Typical visibility |
|---|---|---|
| **gateway** | Legacy-mail bridge (MX, DKIM egress, legacy client surfaces) | `terminating` (legacy leg is plaintext) |
| **relay** | Mesh reachability for NAT'd peers | `blind` / structural |
| **media-relay** | Forwards SFrame-encrypted call/stream media (scales calls) | `blind` / structural |
| **reachability-adapter** | ngrok-style public subdomains for arbitrary box services | `blind-routing` (SNI-passthrough) preferred |
| **indexer** | Search / discovery / global product-and-price view | `blind` / attested (TEE) preferred |
| **labeler** | Moderation labels, opt-in, subscribable | n/a (labels public objects) |
| **matcher** | Real-time supply↔demand matching (rides, delivery) | `blind` / attested preferred |
| **arbiter** | Dispute resolution (staked jury) | `terminating` for evidence, disclosed |
| **oracle** | Physical-world / real-fact attestation (delivered? ride done?) | `terminating`, disclosed |

`gateway` (DMTAP §7) and the legacy `adapter`s (§26) are the first, fully-worked instances;
every kind above inherits the four clauses and the visibility property unchanged.

---

## 6. Economics (normative boundary)

- **In scope:** the authorization *mechanism*, the visibility declaration, the descriptor and
  signed tariff, and — where a coordinator meters — **signed usage receipts** delivered
  directly to the paying party (auditable, never merely asserted).
- **Out of scope (operator policy):** the *numbers* — quotas, rate limits, prices — and the
  settlement rail (an existing stablecoin or fiat; KOTVA brokers none and takes no cut).
- **Never:** a protocol token, an advertising mechanism, or a payout-fairness scheme. Absent
  by decision.

A user's audit is **one-directional** (DMTAP §7.9, generalized): a signed receipt lets a user
confirm a claimed operation was real; it cannot disconfirm an operation the coordinator
fabricated. Disclosed, not hidden.

---

## 7. Conformance checklist

| # | A coordinator… | Clause |
|---|---|---|
| COORD-1 | publishes a signed, discovery-only descriptor with no global score/price-rank/stake | §2.1 |
| COORD-2 | imposes zero lock-in — switching is config-only, identity unchanged | §2.2 |
| COORD-3 | has a self-host backstop (or discloses the one scarce-resource exception) | §2.3 |
| COORD-4 | declares exactly one visibility class + assurance level, and clients surface it | §2.4, §3 |
| COORD-5 | never silently downgrades from blind to terminating | §3.2 |
| COORD-6 | authorizes from identity + rate only; never classifies content | §4 |
| COORD-7 | if metered, issues signed usage receipts directly to the payer | §6 |
| COORD-8 | mints no token; stakes/settles only in existing assets | §6, DIRECTION §5 |
