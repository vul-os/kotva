# MATCH — discovery + assignment with a pluggable rule (the one matching engine)

> **Status:** additive normative primitive spec of the KOTVA family. It is one of the very few
> places KOTVA writes **new** bytes rather than binding an external standard: order-book /
> auction assignment has nothing proven to adopt ([`bindings/README.md`](../bindings/README.md),
> [`docs/research/PRIMITIVES.md`](../docs/research/PRIMITIVES.md)). MATCH owns only the
> **assignment vocabulary and its rules**; identity, objects, feeds, CRDT registers, and the
> matcher-as-coordinator all belong to specs it composes over, which govern where they overlap.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are BCP 14 (RFC 2119 / RFC 8174).

---

## 1. Purpose

**MATCH pairs demand with supply and emits a signed, attributable assignment.** It is the
observation from [`DIRECTION § 2`](../DIRECTION.md) made concrete: Uber, delivery, freelance,
auctions, and ride-hail are *the same engine* — they differ only in the **assignment rule**
(nearest / highest-bid / best-fit). Building the rule as a **pluggable seam** builds all of
them at once, so "different products" are thin profiles, not separate systems.

MATCH is deliberately narrow:

- It takes a set of **candidate** supply commitments answering a **demand**, applies a declared
  **rule**, and produces an **assignment**. It carries **no funds** and holds **no key that
  decrypts the parties' payloads** — money is ESCROW+PAY, confirmation is ORACLE, the goods/work
  themselves are the parties'.
- The rule is data, not code baked into the protocol. `nearest`, `highest-bid`, and `best-fit`
  are three registered rules; a profile MAY register more. The engine is one; the rule slides.
- It is **coordinator-optional**. A **local order book** — any node running the same
  deterministic rule over the candidates it can see — is the offline / small-scale form. A
  **matcher** coordinator ([`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md)) adds only
  **reach** (a global candidate pool and a global-view optimisation), never authority: it can
  *recommend* a pairing, never *forge* one.

[`profiles/wrap`](../profiles/wrap/00-overview.md) is the first fully-worked instantiation
(work orders, bids, issuer-assigns); this document is the primitive WRAP is one profile of.

---

## 2. Objects (wire-shape sketch)

MATCH defines four abstract object kinds. They are ordinary substrate objects — signed by their
author, content-addressed, immutable, state expressed by *adding* objects — carried as SYNC ops
and feed entries, never a merge algebra of MATCH's own (exactly [`wrap § 3`](../profiles/wrap/02-objects.md)).
A profile allocates the concrete `kind` bytes in its own namespace; WRAP §3.1 is the reference
allocation (`WorkOrder`/`Bid`/`Assignment` = `MatchDemand`/`Candidate`/`Assignment` here).

Every object carries the substrate **common header** — `v`, `kind`, `id` (`0x1e ‖
BLAKE3-256(canonical body)`, §22.2), `author` (Ed25519 `IK`), `ts` (HLC) — and is COSE/Ed25519
signed with a domain-separated preimage (§18.1.6). The sketch below shows only the payload.

```cddl
; ---- MatchDemand : what is wanted + how to choose (author = requester) ----
MatchDemand = {
  offer:   bstr,          ; content-address of the OFFER being sought against (the demand-side listing)
  rule:    MatchRule,     ; the assignment rule to apply (§2.1) — declared, not hidden
  fields:  MatchFields,   ; the ONLY structured inputs a matcher may read (§3.2); canonicalized per §3.4
  ? pool:  bstr .size 32, ; matcher/indexer principal or issuer's own key for a local order book
  ? window: Window,       ; time bounds for bidding/assignment (wrap §3.10)
  ? closes: uint,         ; unix seconds bidding-close; after this no new Candidate is valid (defaults to `expires`)
  expires: uint,          ; unix seconds demand-death; after this no new Assignment is valid (REQUIRED, no default)
}

; ---- MatchRule : the pluggable seam (data, never protocol-baked) ----
MatchRule = {
  name:    tstr,          ; "nearest" / "highest-bid" / "best-fit" / profile-registered
  ? weights: { * tstr => int },     ; declared coefficients over `fields` keys (best-fit); integer fixed-point, §3.4
  tiebreak: tstr,         ; deterministic final tie-break, e.g. "hlc" then "id-bytes" (§3.4)
}

; ---- Candidate : a supply commitment answering a demand (author = supplier) ----
Candidate = {
  demand:  bstr,          ; MatchDemand.id being answered — see the `demand` note below
  offer:   bstr,          ; content-address of the supplier's OFFER
  fields:  MatchFields,   ; supplier's structured match inputs (location, quote, capabilities…); canonicalized per §3.4
  ? eta:   uint,          ; unix seconds estimate
}                          ; carried as an OR-Set (SYNC §4.3): concurrent bids never lost, withdraw = observed-remove

; ---- MatchProposal : a matcher's ADVISORY recommendation (author = matcher) ----
MatchProposal = {
  demand:  bstr,           ; MatchDemand.id — see the `demand` note below
  candidate: bstr,         ; the Candidate.id the matcher ranks first
  ? ranked: [* bstr],      ; full ranked candidate list, for one-directional audit (§8, CONTRACT §6)
  candidate_set: bstr,     ; BLAKE3-256 over the sorted Candidate.id list the rule was evaluated over — what `proof` binds to
  proof:   bstr,           ; detached COSE/Ed25519 signature over (demand, ranked, rule, candidate_set) — see §8 R-M-5
}                           ; NON-BINDING: a proposal is a recommendation, never the assignment (§3.1)

; ---- Assignment : the binding pairing (author = demand issuer; single-writer) ----
Assignment = {
  demand:  bstr,            ; MatchDemand.id — see the `demand` note below
  supplier: bstr .size 32,  ; assigned principal
  ? candidate: bstr,        ; the Candidate accepted
  ? revoked: bool,          ; true unassigns (no-show / cancellation) — valid until MatchDemand.expires, §3.5
}                            ; LWW register (SYNC §4.4): the sole authorized writer is the demand issuer (§3.1)

MatchFields = { * tstr => ext-value }  ; e.g. {"lat":…, "lon":…, "bid":…, "caps":["vehicle:bicycle"]}; §3.4
```

> `demand` is always a `MatchDemand.id`, in every object that carries it (`Candidate`,
> `MatchProposal`, `Assignment`). The content-address of the OFFER a demand is seeking is a
> separate field, `MatchDemand.offer` — symmetric with `Candidate.offer` — never named `demand`.
> This is a naming fix over an earlier draft that overloaded `demand` for both; profiles MUST NOT
> reintroduce the collision.

### 2.1 The three registered rules
`nearest` minimises a distance metric over `fields.lat`/`fields.lon`; `highest-bid` maximises
`fields.bid` (sealed-bid variants close before revealing); `best-fit` maximises the weighted sum
`Σ weights[k]·fields[k]` over declared keys. All three are **total deterministic functions of the
candidate set and the rule** — given the same canonicalized inputs (§3.4), every honest evaluator
returns the bit-identical winner. That determinism is the entire reason a local order book equals
a matcher (§6).

---

## 3. Normative rules

### 3.1 The assignment is single-writer; the matcher is advisory
The binding `Assignment` MUST be authored by the **demand issuer** (or co-signed by both parties
in a mutual-accept profile). An `Assignment` whose `author` is not the `MatchDemand` issuer MUST
be rejected (`ERR_NOT_ISSUER`, cf. wrap §13). A `MatchProposal` from a matcher is a
**recommendation only** and MUST NOT be treated as, rendered as, or acted on as an assignment. A
matcher therefore **cannot forge a match**: the most a compromised or biased matcher can do is
recommend badly or withhold, both of which are visible (§8) and swappable (`CONTRACT §2.2`). This
is the RESERVE/single-writer discipline of [`substrate/OFFLINE.md` R-SYNC-1](../substrate/OFFLINE.md):
a cross-replica invariant (one demand → one winner) is enforced by a single authorised writer, not
assumed away by a merge.

### 3.2 Match-fields only — the matcher reads structure, never content
A matcher MUST operate **exclusively** over the declared `MatchFields` of demand and candidates.
Everything else — the parties' messages, item detail, identity beyond the match — stays sealed and
is **not** disclosed to the matcher. A matcher's visibility **class** is always `terminating` — it
necessarily reads the match-fields to rank them ([`CONTRACT § 3.1`](../coordinator/CONTRACT.md)).
A matcher MUST declare its **assurance level**: `declared` (default, honest-trust) or `attested`
when it runs the rule inside a TEE that proves the terminated match-fields are confined to the
enclave and not logged ([`CONTRACT § 3.3`](../coordinator/CONTRACT.md), §8 R-M-2). A matcher MUST
NOT require more fields than the rule consumes.

### 3.3 Authorise, never classify
A matcher optimises; it MUST NOT **classify**. It MUST NOT drop, hide, quarantine, or
down-rank an **eligible** candidate on a content basis (who the supplier is beyond identity+rate,
what they "seem like"), and MUST NOT inject candidates the rule does not rank. Eligibility is an
**authorisation** question — is this candidate well-formed, within its rate, answering this demand,
unexpired? — answered from identity and the declared fields, never from a judgement about whether
the candidate is "wanted" ([`CONTRACT § 4`](../coordinator/CONTRACT.md)). "Wanted" is the issuer's
call, made on the issuer's device by *accepting* the assignment. A matcher that classifies is
non-conformant, because classification centralises by construction (§8).

### 3.4 Determinism and ties — canonical arithmetic is mandatory
The rule MUST be a deterministic function of `(candidate-set, rule)`. IEEE-754 floating-point
arithmetic is **not** bit-identical across languages/runtimes, so `MatchFields` and `weights` values
MUST already be fixed-point integers **as published** — the fixed-point form is what gets signed, not
a normalisation the matcher applies afterwards. A float can never appear on the wire at all
([§18.1.1](../18-wire-format.md) rule 4 forbids it in every object, and these are signed objects), so
both maps are typed `ext-value` / `int`: text-keyed like `Headers.ext` because the criteria set is
application-declared, and value-restricted for exactly the same reason — floats, NaN/Infinity,
`undefined` and tags would make the signature non-reproducible. Concretely, the encodings are:
coordinates as integer microdegrees (`round(1e6 × decimal-degrees)`), bid/score/weight terms as
integer minor units, `nearest`'s distance compared as squared-integer (no `sqrt`, no float), and
`best-fit`'s weighted sum accumulated as integer (fixed-point) multiply-then-sum over `weights`
keys taken in ascending key-byte order — no reordering. A `MatchDemand` or `Candidate` whose
`fields` cannot be canonicalized to the rule's required integer encoding MUST be refused
(`ERR_FIELD_SHAPE`, R-M-4). `highest-bid` is integer/ordinal by construction and needs no
canonicalization. Ties MUST be broken by the declared `tiebreak`, ending in a total order
(RECOMMENDED: HLC then `id` bytes, as SYNC §2.2). Under this canonicalization two independent
evaluators — a matcher and a local order book — reach the **bit-identical** winner and a
double-assignment is a *detectable equivocation* (§7), not an ambiguity. A profile or matcher that
skips canonicalization MUST NOT claim cross-evaluator determinism: its result is
locally-recomputable by re-running the same rule over the same candidate set, but not guaranteed
bit-identical to another implementation's.

### 3.5 Bidding-close and expiry are distinct gates
`MatchDemand.expires` is REQUIRED and has no default; it bounds the demand's life — after
`expires`, no `Assignment` for that demand (including a `revoked = true` unassign) is valid.
`closes` (OPTIONAL, defaults to `expires`) bounds bidding — after `closes`, no new `Candidate` for
that demand is valid. An order book whose bidding window never closes is indistinguishable from
stale demands and poisons the pool (wrap §3.3). Separating the two gates keeps a late no-show,
cancellation, or reassignment reachable: between `closes` and `expires` the issuer MAY still
author, revoke, or reassign an `Assignment` among the `Candidate`s already on record, without
reopening the pool to new bids.

### 3.6 No funds, no token
MATCH carries settlement value **never**. A bid amount in `fields.bid` is a *term*, not a payment;
the money moves later over ESCROW+PAY against an existing stablecoin/fiat rail
([`DIRECTION § 5`](../DIRECTION.md)). A matcher that meters MUST issue signed usage receipts to the
payer and MUST price its service in an existing asset — no protocol token, ever
([`CONTRACT § 6`](../coordinator/CONTRACT.md)).

---

## 4. Composition with the other primitives

MATCH is a joint, not a silo. Its neighbours ([`DIRECTION § 2`](../DIRECTION.md),
[`docs/research/PRIMITIVES.md`](../docs/research/PRIMITIVES.md)):

| Primitive | Relationship to MATCH |
|---|---|
| **OFFER** | Demand and supply are both **OFFER** objects (content-addressed signed listings). MATCH *consumes* them and adds no listing model of its own; `MatchDemand.offer` and `Candidate.offer` are OFFER content-addresses. |
| **RESERVE** | When the matched supply is a **bookable** single-owner resource, the hold is a RESERVE (bounded-counter, single-writer), not a MATCH — a booking needs no matcher at all. MATCH assigns; RESERVE holds capacity. |
| **REPUTATION** | A rule MAY read a reputation value as a `fields` input (OpenRank-computed or locally measured). MATCH consumes a score; it never publishes one, and no MATCH object carries a network-wide number ([`DIRECTION § 5`](../DIRECTION.md)). |
| **ESCROW / PAY** | Settlement follows assignment and is out of MATCH entirely. The `Assignment` is the trigger, never the transfer. |
| **ORACLE / DISPUTE** | "Did the matched ride/delivery/work happen?" is an ORACLE + DISPUTE question after the fact — the physical-event ceiling ([`DIRECTION § 8`](../DIRECTION.md)), not MATCH's to close. |
| **ATTEST** | The outcome of an assignment is recorded as an `Attestation` feeding REPUTATION (wrap §3.8); MATCH emits the pairing, ATTEST records how it went. |

---

## 5. Binding adopted

Per [`bindings/README.md`](../bindings/README.md), the assignment engine itself has **nothing to
bind** — no proven external order-book/auction standard exists — so MATCH's ranking bytes are
original normative writing (one of the three things KOTVA owns: substrate, coordinator contract,
thin primitives/profiles). What MATCH *does* bind at its edges:

- **Verifiable coordination → TEEs** (`bindings` TEE row). A matcher raises its **assurance level**
  from `declared` to `attested` by binding to an SGX/SEV/TrustZone TEE that proves the terminated
  match-fields never leave the enclave (§3.2). Disclosed, not trustless: it trades operator-trust
  for chip-vendor-trust.
- **Reputation → OpenRank** (`bindings` reputation row) as a rule input, never recomputed here.
- **Identity / auth → the substrate** ([`substrate/IDENTITY.md`](../substrate/IDENTITY.md)): issuer
  and supplier are substrate `IK`s; the single-writer check is a cert-chain check, not a new scheme.
- **Matcher-as-coordinator → the contract** ([`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md)):
  a matcher is one coordinator kind, inheriting all four clauses unchanged.

When any of these frontiers improves, the filling swaps and the MATCH bytes do not move
([`DIRECTION § 9`](../DIRECTION.md)).

---

## 6. Scale-invariance — order book to planet on one engine

The engine is identical at every scale; only the **candidate reach and trust anchor** slide
([`DIRECTION § 6`](../DIRECTION.md)):

| | Mesh / small (no coordinator) | Global (swappable matcher) |
|---|---|---|
| Candidate pool | whoever is in the local following-graph / mesh range | a global matcher-as-a-service pool |
| Who applies the rule | any node — a **dumb local order book** running the same deterministic rule | the matcher, `terminating` at `attested` assurance preferred |
| Personhood / Sybil floor | web-of-trust (you know these suppliers) | a personhood attester the issuer chooses |
| Who signs the assignment | the issuer, always — *unchanged across scales* |

Because the rule (§2.1) is a deterministic function of its canonicalized inputs (§3.4), the
**local order book and the global matcher compute the same winner over the same candidate set**.
The matcher adds *candidates* the issuer could not otherwise reach; it never adds *authority*.
Removing it shrinks the pool, never breaks the function — the coordinator-optional property, held.

---

## 7. Offline / apocalypse behaviour + reconcile

Graded per [`substrate/OFFLINE.md § 2`](../substrate/OFFLINE.md):

- **Authoring** a `MatchDemand`, `Candidate`, or `Assignment` is **`full`** offline — sign and
  content-address locally (§2).
- **Applying the rule over the locally-visible candidate set** is **`full`** — the order book is
  local and deterministic.
- **Applying it over the *global* pool** is **`local-trust`** (only the reachable candidates are
  seen; the substituted local anchor is disclosed) degrading to **`deferred`** for a matcher
  proposal that settles when the matcher is reachable. A `MatchProposal` is never load-bearing, so
  its absence blocks nothing (R-ROLE-1: **wake/proposal MUST NOT be relied on for correctness**).

**Reconcile on reconnect.** Candidates are an OR-Set and heal by version-vector diff (SYNC §4);
the binding `Assignment` is an LWW register with a **single authorised writer** (§3.1), so two
partitioned replicas cannot both hold a valid winning assignment for one demand from honest
participants — the WRAP issuer-assigns reference for [`OFFLINE.md` R-SYNC-1](../substrate/OFFLINE.md).
If a **malicious** issuer signs two assignments, that is not a merge failure to be smoothed over: it
is a signed, permanent, attributable equivocation that MUST be **surfaced** for single-writer /
dispute resolution (`OFFLINE.md` R-REC-2), never masked by the CRDT's clean merge. Reconcile is
idempotent by content-address (R-REC-1): a re-delivered assignment changes nothing.

---

## 8. Security MUSTs

Inheriting [`THREAT-MODEL.md`](../THREAT-MODEL.md); a profile MAY add, MUST NOT subtract.

- **R-M-1 — A matcher cannot forge a binding match (SEC-2).** Only an issuer-signed (or mutually
  co-signed) `Assignment` binds; a `MatchProposal` is advisory and MUST NOT be honoured as an
  assignment (§3.1). Every object is self-authenticating and domain-separated, so a proposal can
  never verify as an assignment.
- **R-M-2 — Visibility is declared; match-fields only (SEC-3, SEC-4).** A matcher reads only
  `MatchFields` and holds no key to the parties' sealed payloads outside them. Its visibility
  **class** is always `terminating` (it reads the fields to rank them); it MUST declare exactly
  one **assurance level** — `declared` (default) or `attested` (TEE, proves the terminated
  plaintext never leaves the enclave) — surfaced to users. No silent downgrade into `declared`
  where `attested` was advertised.
- **R-M-3 — Authorise, never classify (SEC-6).** Eligibility is identity+rate+well-formedness; a
  matcher MUST NOT content-drop or content-rank an eligible candidate, and MUST NOT be
  load-bearing — removing it degrades reach, never function.
- **R-M-4 — Fail-closed and replay-inert (SEC-1, SEC-8).** A candidate that fails signature,
  expiry, rate, or field-shape validation is **refused**, never guessed. Assignments are immutable
  content-addressed LWW ops, so re-delivery is an idempotent join — replay changes nothing.
- **R-M-5 — One-directional audit (SEC-6, `CONTRACT § 6`).** A matcher SHOULD emit `ranked` and a
  `proof`: a detached COSE/Ed25519 signature, by the matcher, over the tuple (`demand`, ordered
  `ranked` candidate-id list, `rule` bytes, `candidate_set` digest — §2). `candidate_set` names the
  exact evaluated Candidate-id set, so a verifier can re-run the declared deterministic rule (§3.4)
  over the named candidates and confirm the order. This lets a client **confirm** a claimed
  ranking was real over the candidates named in `candidate_set`; it cannot **disconfirm** a
  candidate the matcher silently left out of that set. Disclosed, not hidden.
- **R-M-6 — No token, no funds (`DIRECTION § 5`).** `fields.bid` is a term; MATCH moves no value
  and mints nothing; a metering matcher prices in an existing asset with signed receipts.

---

## 9. Honest residual

Per house rule, the boundaries MATCH does **not** cross:

- **Global reach is where centralisation regrows.** A matcher's value is candidate **liquidity**,
  and liquidity has network-effect gravity: the matcher with the deepest pool wins, and that pull
  toward one operator is real even though the contract keeps it swappable and self-hostable. The
  engine stays sovereign; the *market* still tends to concentrate. Disclosed
  ([`docs/research/PRIMITIVES.md`](../docs/research/PRIMITIVES.md), MATCH ceiling).
- **A matcher's visibility class is always `terminating`.** By definition it reads the match-fields
  to rank them; there is no blind matcher class. `attested` raises the **assurance level** that the
  terminated plaintext stays confined to a TEE, but trades operator-trust for **chip-vendor-trust**
  with a side-channel history — never sold as trustless ([`bindings` TEE row](../bindings/README.md),
  `CONTRACT § 3.4`).
- **Rule-fairness is policy, audited one-directionally.** The protocol enforces that the *binding*
  assignment is single-writer and that a matcher *declares* its rule; it cannot prove a matcher
  actually ran the declared rule (only `attested` execution can). A biased ranking is detectable if
  the rule is verifiable, and otherwise only inferable from results — the same one-directional-audit
  ceiling as every coordinator (R-M-5).
- **Sybil in the candidate pool is the anti-Sybil ceiling, unchanged.** At local scale it dissolves
  into web-of-trust; at global scale it binds to imperfect proof-of-personhood, which raises the
  floor and does not close it ([`DIRECTION § 8`](../DIRECTION.md)).
- **Whether the matched thing actually happened is the physical-event ceiling**, ORACLE+DISPUTE's to
  bound and no one's to prove non-fabrication — MATCH ends at the assignment, not the outcome.

Maturity claims (TEE matching, OpenRank inputs) are a 2026-07 snapshot and must be re-checked
before any binding is relied on in production.
