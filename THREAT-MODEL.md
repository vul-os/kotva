# THREAT-MODEL — the security floor every KOTVA primitive and profile stands on

**Status:** draft, normative once ratified. A **family-level** companion to
[`DIRECTION.md`](DIRECTION.md) (the rules) and [`coordinator/CONTRACT.md`](coordinator/CONTRACT.md)
(the coordinator keystone). It is not a wire spec and defines no new bytes: it states the
**cross-cutting security invariants** that every waist capability ([`substrate/`](substrate/README.md)),
every binding ([`bindings/README.md`](bindings/README.md)), every coordinator, and every profile
([`profiles/`](profiles/)) MUST honor, and names each one's **honest residual** so a profile author
cannot quietly weaken the floor. Where a numbered section already owns the normative bytes, it
governs; this document is the checklist those sections are all instances of.

This complements — does not replace — [`SECURITY.md`](SECURITY.md), which is the disclosure
*policy* (how to report), not the threat *model* (what we defend).

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are BCP 14 (RFC 2119 / RFC 8174).

---

## 1. What this document is for

The waist composes many independent protocols over one core. A security property is only as
strong as the **weakest** primitive that touches a message, and a profile author working inside
one capability cannot see the whole path. So the guarantees cannot live scattered in each spec:
they are stated **once**, here, as invariants every capability inherits by construction, and each
profile restates the ones it must preserve (as [`profiles/wrap/13-security.md`](profiles/wrap/13-security.md)
and the profile SECURITY files already do).

Two framing rules from [`DIRECTION.md`](DIRECTION.md) govern the whole document:

- **The floor is adopted, not invented.** Every hard security job is a
  [binding](bindings/README.md) (recovery → account abstraction; anti-Sybil → proof-of-personhood;
  content crypto → MLS/HPKE; media crypto → SFrame). KOTVA specifies the *seam and the invariant*,
  never the primitive. When the frontier improves, the filling swaps and this floor does not move
  ([DIRECTION §9](DIRECTION.md)).
- **Every claim ships with its residual.** A security section that lists only strengths is
  marketing. Each MUST below carries the boundary it does **not** cross (§6), disclosed rather
  than hidden — the same discipline as [`coordinator/CONTRACT §3.4`](coordinator/CONTRACT.md).

---

## 2. Adversary model (family-level)

Profiles narrow this; none may widen a defended cell into an undefended one without disclosure.

| Adversary | Capability | Family posture |
|---|---|---|
| **Network eavesdropper** | reads links near a node | **defeated** — everything on the wire is signed and end-to-end encrypted (§3, SEC-1/SEC-3). |
| **Content-blind intermediary** (relay, media-relay, mailbox, SNI ingress) | forwards/holds ciphertext it cannot read | **contained by contract** — holds no decrypting key; visibility class declared and surfaced (SEC-4, [CONTRACT §3](coordinator/CONTRACT.md)). |
| **`terminating` intermediary** (legacy gateway, TLS-terminating ingress) | sees plaintext at a disclosed boundary | **bounded, never silent** — declared `terminating`, no downgrade into it (SEC-4, SEC-8). |
| **Malicious coordinator** (indexer, matcher, labeler, arbiter, oracle) | withholds, favours, mis-orders, or lies about its own operation | **hired, not depended-on** — authorizes but never classifies; swappable with zero migration; audit is one-directional (SEC-6). |
| **Sybil / flooding peer** | mints unlimited identities, floods well-formed objects | **raised, not closed** — cost-for-cold-contact + local web-of-trust + bounded pending state (SEC-7); a disclosed ceiling globally (R-7). |
| **Compromised endpoint** | reads/keys one seized, unlocked device | **hardened then bounded** — hardware-backed non-exportable keys, fast revocation, recovery heals all cases *except* a device actively compromised while unlocked (SEC-5, R-5). |
| **Metadata / traffic-analysis adversary** | observes who talks to whom, when, how much | **reduced where a profile buys it, disclosed where it does not** — sealed sender hides the sender from intermediaries; it is a reduction, not elimination; strong graph privacy against a global passive adversary is a **research-tier** profile property, non-normative here (SEC-9, R-9). |

The single headline: **an intermediary that cannot read is the default; one that can is a
declared exception.**

---

## 3. The security MUSTs

Nine invariants. Every primitive, binding, coordinator, and profile inherits all nine; a profile
MAY add, MUST NOT subtract, and MUST disclose any residual it cannot close.

### SEC-1 — Fail-closed everywhere
A security-relevant failure MUST either be **refused** (fail closed) or surfaced as an **explicit
user choice** — never a silent fallback to an unauthenticated, unencrypted, or unverified path
(the core fail-closed governance, §10.7; [`substrate README §3 rule 5`](substrate/README.md)).
This binds every layer: an unknown suite is rejected, never guessed (`ERR_UNKNOWN_SUITE`, §1.1); a
signature that does not verify blocks the object; an unverifiable `declared`-blind claim MUST NOT
be shown as verified ([CONTRACT §3.4](coordinator/CONTRACT.md)); a clock-skew or membership check
that cannot be evaluated denies, it does not admit (SYNC §3, §9). A capability MUST NOT be silently
degraded. *Residual: R-1.*

### SEC-2 — Authenticity is intrinsic, not conferred by a server
Every object MUST be **self-authenticating** — COSE/Ed25519-signed (or content-addressed) so a
fetcher verifies it identically regardless of which transport delivered it, trusting the server for
nothing (the HTTP test, [`substrate/README.md §4.2`](substrate/README.md); §22.5.1). A signing
preimage MUST be **domain-separated** by a DS-tag ([§18.1.6](18-wire-format.md)) so a signature
minted for one object kind can never verify as another. Authorization for mutable state MUST chain
to an identity key via a non-revoked `DeviceCert` ([`substrate/IDENTITY.md §2.2`](substrate/IDENTITY.md),
SYNC §9) — the substrate never re-invents authorization, it checks a cert chain. Names confer
**discovery, never authority**: a name resolves to a key; the key is the identity; the ladder is
never inverted ([`substrate/README.md §3` rule 6](substrate/README.md)). *Residual: R-2.*

### SEC-3 — Content confidentiality is end-to-end, keyed off adopted crypto
Payload confidentiality MUST be **end-to-end**: encrypted to the recipient(s) with keys no
intermediary holds, using an [adopted](bindings/README.md) primitive — **MLS (RFC 9420)** for
messaging/group state, **HPKE (RFC 9180)** for sealed objects, **SFrame (RFC 9605) keyed from the
MLS epoch** for real-time media ([DIRECTION §7](DIRECTION.md)). KOTVA MUST NOT invent a ratchet or a
media cipher. Deterministic encoding of any signed/encrypted structure is mandatory (deterministic
CBOR, RFC 8949 §4.2) so two implementations produce byte-identical, reproducibly-verifiable
preimages. *Residual: R-3.*

### SEC-4 — Every intermediary declares what it can see
Any party on a message path — coordinator or not — MUST declare exactly one **visibility class**
(`blind` / `blind-routing` / `terminating`) at one **assurance level** (`structural` / `attested` /
`declared`), MUST surface it to the user, and MUST NOT advertise one class while operating another
([CONTRACT §2.4, §3](coordinator/CONTRACT.md)). Where a function can be served blind, `blind` is
RECOMMENDED and `terminating` requires opt-in plus disclosure. A `declared`-level blind claim is an
intent, never a proof, and MUST NOT be presented as verified (only `structural` and `attested` are
checkable). *Residual: R-4.*

### SEC-5 — Key compromise and loss are recoverable, and no single device is load-bearing
Irreversible key-loss MUST NOT be the failure mode. Recovery is **adopted, not invented** — account
abstraction (ERC-4337 / EIP-7702), passkey backup, or MPC shares behind the recovery seam
([`bindings/README.md`](bindings/README.md), [DIRECTION §3](DIRECTION.md)). The root `IK` SHOULD be
held cold / in recovery custody and used rarely; day-to-day signing uses `DeviceCert` subkeys
([`substrate/IDENTITY.md §2`](substrate/IDENTITY.md)). Compromise MUST be **healable**: a
`DeviceCert` is revocable and revocation MUST fail closed at every authorization check. **No single
device — not even `admin` — may unilaterally rewrite recovery policy or re-key the identity**; that
requires `IK` or the `rotate_threshold` quorum ([`substrate/IDENTITY.md §2.2`](substrate/IDENTITY.md),
§1.4). Keys SHOULD be hardware-backed and non-exportable where the platform allows (§1.2a). A
profile MUST NOT expose an API that lets one device silently take over the identity. *Residual: R-5.*

### SEC-6 — Coordinators authorize; they never classify, gate, or lock in
Every coordinator MUST satisfy all four [CONTRACT](coordinator/CONTRACT.md) clauses — accountable,
swappable (zero data migration, zero identity change), self-hostable, visibility-declared — and
MUST be **never load-bearing**: removing it degrades reach, never function — with **one disclosed
exception**, the **custodial escrow operator**, which holds the float for a trade window and is
therefore structurally load-bearing by design, not oversight ([CONTRACT §1](coordinator/CONTRACT.md);
[ESCROW §10](primitives/ESCROW.md)), bounded by bonding/staking sized to the float and by per-order
swappability, never eliminated. A coordinator's every
gate MUST be an **authorization** answered from *sender identity + rate* — never a content
classification (no spam scoring, no ML filter, no content-basis drop/re-rank/annotate)
([CONTRACT §4](coordinator/CONTRACT.md)). "Wanted" is judged by the recipient, on the recipient's
device. Moderation is a **market of opt-in labelers**, each itself a coordinator you can leave. If a
coordinator meters, it MUST issue **signed usage receipts directly to the payer**; the audit is
**one-directional** (a receipt confirms a real operation, it cannot disconfirm a fabricated one) and
that limit is disclosed, not hidden ([CONTRACT §6](coordinator/CONTRACT.md)). It mints **no token**;
stake and settlement are existing assets only ([DIRECTION §5](DIRECTION.md)). *Residual: R-6.*

### SEC-7 — Abuse is priced and localized, never centrally filtered
Anti-abuse MUST hold **without** a central content filter and **without** deanonymizing the sender.
The permitted tools are: **authenticated-by-default** identity (no anonymous unauthenticated
injection), **anonymous-but-accountable** rate-limit tokens, **cost-for-cold-contact** (proof-of-work,
issued token, or paid postage; known contacts free), and **local recipient policy** applied before
the inbox (§9). Every proof that admits envelope binding — `ArcToken`, `PowSolution`,
`PostageStamp` — MUST be **cryptographically bound to the carrying envelope** (to the ephemeral
`sender_key`) so a copy stripped from a victim and re-attached to an attacker's message is worthless
(§9.2a). A **vouch** cannot be bound at mint time — the voucher cannot know a key the vouchee has
not yet generated — and is instead bound to its **named subject**, verified post-decryption; a
lifted vouch still buys the thief one decryption before rejection, a disclosed residual, not a gap
in this MUST (§9.2a; R-7). Global anti-Sybil is **not solved**: at local scale it dissolves into
web-of-trust; at global scale it binds to proof-of-personhood (World ID / Human Passport) which
**raises the floor, never closes it** ([bindings](bindings/README.md), [DIRECTION §8](DIRECTION.md)).
A profile MUST NOT describe Sybil resistance as solved. *Residual: R-7.*

### SEC-8 — Replay is inert, and downgrade is impossible
**Object replay MUST be harmless by construction**: objects are immutable, content-addressed, and
every merge is an idempotent join, so a re-delivered object changes nothing (SYNC §2.2; WRAP §14.4).
**Transport freshness** rides the substrate wire's own authentication, never a per-profile nonce
cache. Ordering MUST NOT be forgeable: the HLC skew bound is **one-sided and fail-closed** — an op
dated too far into the *future* is rejected, a past-dated offline op is accepted (SYNC §3), so
neither a fast clock nor a stale-clock attacker can win an LWW race or refuse a legitimate offline
backlog. **No silent downgrade, anywhere**: unknown crypto suites are rejected not guessed (SEC-1);
a coordinator that can run `blind` MUST NOT run `terminating` without disclosure
([CONTRACT §3.2](coordinator/CONTRACT.md)); a TLS/mesh path MUST NOT fall back to plaintext
unannounced. *Residual: R-8.*

### SEC-9 — Metadata exposure is minimized where bought, and disclosed where not
Where a profile carries it, the sender's identity and authenticating signature MUST live **inside**
the encrypted payload (**sealed sender**), so intermediaries see only ciphertext to an opaque
destination (§6.2). Honest scope, which a profile MUST state and MUST NOT overstate: sealed sender
hides the sender **from intermediaries**, **not** the sender's IP, and is a metadata *reduction*, not
elimination — timing/receipt side channels statistically erode it. **Strong metadata privacy against
a global passive adversary** (mixnet / onion routing / cover traffic) is **research-tier and
non-normative** in the KOTVA family: it is quarantined to `docs/research/` because its assurance is not
deployment-grade ([DIRECTION §9](DIRECTION.md), [`docs/research/README.md §5`](docs/research/README.md)).
A profile MUST NOT claim graph/timing privacy it does not implement, and MUST declare the
`blind-routing` metadata a buffer/relay/ingress sees ([CONTRACT §3.1](coordinator/CONTRACT.md)).
*Residual: R-9.*

---

## 4. Apocalypse-proof: the invariants survive going offline

Every MUST above MUST hold with **no coordinator and no connectivity** ([DIRECTION §6](DIRECTION.md)).
Concretely: SEC-2 authenticity and SEC-3 confidentiality are intrinsic to the object and need no
server; SEC-7 anti-abuse collapses to web-of-trust and local policy; SEC-6 coordinators are absent,
so nothing they did was load-bearing — except a custodial-escrow trade window, the disclosed
exception, whose settlement is `blocked` offline by design ([ESCROW §8](primitives/ESCROW.md)),
leaving only the trade objects to reconcile; SEC-8 replay-inertness and deterministic merge let a partitioned
replica **reconcile on reconnect** with byte-identical convergence (SYNC §2.2). A security property
that only holds while a coordinator is reachable is a conformance violation of this document, not a
degraded mode. Graceful offline degradation plus reconcile-on-reconnect is the test.

---

## 5. Conformance checklist

| # | Every primitive / binding / coordinator / profile… | Anchor |
|---|---|---|
| SEC-1 | fails **closed** — refuse or explicit choice, never silent unauthenticated/unencrypted fallback | §10.7; substrate README §3 rule 5 |
| SEC-2 | ships **self-authenticating**, domain-separated, cert-chained objects; names point, never authorize | §18.1.6; IDENTITY §2 |
| SEC-3 | encrypts payloads **end-to-end** with adopted crypto (MLS/HPKE/SFrame), deterministic encoding | bindings; DIRECTION §7 |
| SEC-4 | **declares** one visibility class + assurance level, surfaces it, never misrepresents | CONTRACT §2.4, §3 |
| SEC-5 | makes key-loss/compromise **recoverable** (adopted); no single device rewrites recovery | IDENTITY §2.2; bindings |
| SEC-6 | is a coordinator that **authorizes, never classifies**; swappable, never load-bearing (except custodial escrow, disclosed — R-6); no token | CONTRACT §2, §4, §6 |
| SEC-7 | prices **cold contact**, binds proofs to the envelope, never central-filters or deanonymizes | §9, §9.2a |
| SEC-8 | makes replay **inert** and downgrade **impossible** (suite/visibility/TLS no-downgrade) | SYNC §2.2, §3; CONTRACT §3.2 |
| SEC-9 | **minimizes** metadata (sealed sender where bought), **discloses** what it does not hide | §6.2; DIRECTION §9 |
| SEC-A | preserves every property above **offline**, reconciling on reconnect | §4 above; DIRECTION §6 |

---

## 6. Honest residual (normative disclosure)

Per house rule, the boundaries this floor does **not** cross — one per invariant, disclosed so a
profile author cannot silently assume them away:

- **R-1 (fail-closed).** Fail-closed defends **availability of correctness**, not availability of
  service: a node that correctly refuses is a node that, to an attacker, can be *made* to refuse.
  Denial-of-service is bounded (size limits, bounded pending state, mandatory expiry — WRAP §14.5),
  never eliminated.
- **R-2 (authenticity).** Self-authentication proves *who signed*, never *that the signer is
  truthful*. A signed lie is still a lie; the substrate gives durable, attributable evidence, not
  prevention (WRAP §14.3). Non-repudiation is also non-deniability — a property some contexts do not
  want; deniable messaging is a research-tier layer, not a family MUST.
- **R-3 (confidentiality).** End-to-end encryption protects content **in transit and at rest between
  endpoints**, never a **compromised endpoint** that holds the keys (§2.7 adversary; R-5). Group
  confidentiality is only as strong as MLS membership hygiene, which is the profile's to run.
- **R-4 (visibility).** The contract can mandate the *architecture* that makes blindness possible and
  the *declaration* rules; it **cannot** cryptographically prove a `declared`-level operator is not
  secretly logging. Only `structural` (no key) and `attested` (TEE) are verifiable — and `attested`
  trades operator-trust for **chip-vendor-trust** with a side-channel history, disclosed not trustless
  ([CONTRACT §3.4](coordinator/CONTRACT.md); [bindings TEE row](bindings/README.md)).
- **R-5 (key compromise).** Recovery and revocation heal a **lost** or **seized-and-locked** device;
  they do **not** heal a device **actively compromised while unlocked and in use** — that endpoint's
  live plaintext and signing capability are exposed for the compromise window (§6.6). Recovery adds a
  new trust dependency (guardians / passkey vault / MPC operators) in exchange for killing
  irreversible loss.
- **R-6 (coordinators).** The audit is **one-directional**: a signed receipt confirms a real
  operation, it cannot disconfirm one the coordinator fabricated or silently omitted
  ([CONTRACT §6](coordinator/CONTRACT.md)). One honest scarcity exception is disclosed rather than
  papered over — the one scarce-network-reachability exception class has two members: legacy SMTP
  egress (a reputable IP + unblocked port 25) and the reachability-adapter's public ingress, either
  of which an ISP/host may deny ([CONTRACT §2.3](coordinator/CONTRACT.md)). A second, structural exception to "never load-bearing"
  itself: the **custodial escrow operator** is the family's one honest load-bearing coordinator — it
  holds the trade-window float and can abscond, become insolvent, or freeze funds, a counterparty
  risk bonding/staking bounds but does not eliminate ([CONTRACT §1](coordinator/CONTRACT.md);
  [ESCROW §10](primitives/ESCROW.md)).
- **R-7 (anti-abuse).** **Global anti-Sybil is imperfect** — every personhood method trades off
  (biometrics + operator, or zk-passport that excludes the undocumented); it raises the floor and does
  not close it ([DIRECTION §8](DIRECTION.md)). Cost-for-cold-contact deters bulk sending; it does not
  stop a funded, patient, low-volume abuser. **Vouch is bound post-decryption, not to the envelope**:
  a lifted vouch still buys the thief one decryption before `ERR_VOUCH_SUBJECT_MISMATCH`, and because
  a replayed vouch is charged against the *subject's* budget, the replay MUST be surfaced to the
  recipient rather than silently rate-limited into invisibility — otherwise the mechanism becomes a
  way to frame the vouched-for party (§9.2a).
- **R-8 (replay/downgrade).** Object-replay inertness holds **only because** merges are idempotent
  joins; a profile that adds non-idempotent side effects on receipt reintroduces replay and MUST carry
  its own freshness. Downgrade is closed against **protocol** paths; a user who manually overrides a
  disclosed `terminating` opt-in has made a choice the protocol cannot unmake.
- **R-9 (metadata).** Sealed sender does **not** hide IP, and is eroded by timing/receipt side
  channels; the **buffered/offline** path is a polled shared store an observer can watch (delivery tag,
  time, volume) — the untrusted-store access-pattern problem PIR exists for, **not** solved here (§6.4
  item, §6.6). Strong graph/timing privacy is quarantined as non-normative and MUST NOT be claimed by a
  normative profile ([DIRECTION §9](DIRECTION.md)).

Two things KOTVA **cannot** give even in principle, and says so plainly ([DIRECTION §8](DIRECTION.md)):
**coercion-resistant public-election voting** (strictly harder than anti-Sybil) and
**surveillance-based ad markets** (rejected by design, not a gap). Everything else composes on the
floor above.
