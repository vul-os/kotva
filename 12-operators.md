# 12. Operators, the Seam & Sustainability

DMTAP is deployed in one of two modes, and the boundary between them is a small, explicit
**operator seam**. This section is partly *informative* (business/licensing intent) and partly
*normative* (the inviolable rule and the seam's guarantees).

## 12.1 Two deployment modes

| Mode | Who runs it | Cost | Limits |
|------|-------------|------|--------|
| **Self-host** | The user, on their own box | $0 | none — fully functional, unrestricted |
| **Hosted operator** | A provider (e.g. a commercial control-plane) | paid | plan quotas on *operations* only |

The OSS is a **complete product on its own**. Self-host is not a crippled tier: it has every
protocol, client, and privacy feature. A hosted operator adds *convenience* (someone else runs
the box, warms the IPs, manages the domain), not *capability*.

## 12.2 The operator seam

The seam is the clean boundary the OSS exposes so an operator can add billing and multi-tenant
management **without forking or patching the protocol**. It is four capabilities (reference
crate: `crates/dmtap-seam`, contract: its `CONTRACT.md`):

- **Metering** — the OSS emits usage events at the real cost centers (gateway egress, storage,
  relay bytes, message counts, vanity domains).
- **Provisioning** — create/suspend accounts and `@`-addresses across onboarding tiers A/B/C
  (§3.8).
- **Policy** — per-account quotas/entitlements on operations (storage caps, send caps, domain
  counts, rate limits).
- **GatewayAuthz** — authorize legacy egress with per-identity-token accountability (§9),
  preserving sealed sender.

Each capability has a **self-host default that is unlimited/no-op**, so with no operator the
OSS runs unrestricted. A hosted operator implements the same capabilities — in-process (Rust
traits) or out-of-process (HTTP/events) — to add billing and quotas.

**Fail-open to function, fail-safe on security (MUST):** if the operator is unreachable, the OSS
MUST NOT break user-facing mail/chat/files. Metering queues and retries (usage may under-count
during an outage), and quota **Policy** falls back to allow (a billing concern, not a security
one). **GatewayAuthz is different and MUST NOT fail open to "allow."** GatewayAuthz is a security
control (§9): the accountability it enforces is exactly what prevents unattributable legacy
egress — the open-relay failure mode §7.7 exists to avoid. On operator-unreachable, GatewayAuthz
MUST fall back to a **safe default**: permit legacy egress only to **already-established
contacts** and to senders carrying a valid **proof-of-work solution** (§9.4) — the one anti-abuse
proof that is genuinely **verifiable without any operator** — and **deny cold/unproven legacy
egress** for the outage window. **Postage is deliberately excluded from this fallback:** a stamp
requires an **online issuer redemption check** against the issuer's endpoint or signed spent-list
(§9.5.1), and is treated as *unverified* whenever that issuer is unreachable, so it is **not**
operator-independent and MUST NOT be accepted on faith during an outage (it is only usable here if
the postage issuer happens to be independently reachable at redemption time). Established contacts
plus operator-independent PoW are therefore the genuinely operator-independent egress path; a
generic "fail-open to allow" for this capability is prohibited.

## 12.3 The inviolable rule (normative)

Privacy, cryptography, metadata privacy, and recovery MUST NEVER be behind the seam or a
paywall. There MUST be no seam hook, quota, or plan gate capable of disabling encryption,
weakening the mixnet, reducing metadata privacy, or denying a user access to their own keys or
mailbox. The seam meters and limits **operations and organizational concerns only**. Premium is
for *running it*, never for *protection*.

A conformant operator implementation MUST NOT expose any control that violates this rule; a
conformant OSS build MUST NOT consult the seam for any privacy/crypto decision.

## 12.4 Business & licensing model (informative)

DMTAP follows an **open-software + paid-operations** model — the GitLab-style split done cleanly,
without a crippled free tier:

- **Everything a user touches, and everything trust depends on, is open** — protocol, spec,
  node, gateway, client, crypto. Shipped under the **MIT license** (Apache-2.0 dual-licensing is
  under consideration for its explicit patent grant, relevant to novel mechanisms such as
  anti-abuse postage/tokens). Libraries other implementations embed are permissively licensed
  to maximize adoption.
- **The paid layer is a thin, private control-plane** (a *hosted operator*) that implements the
  seam contract and bills **operations**: hosted nodes, storage, legacy egress / IP reputation,
  vanity domains, org/SLA. It withholds no protocol, client, or privacy feature.
- **The moat is reputation + network + being the best operator**, not the code. Because openness
  is what lets the network grow large enough to be worth hosting, full openness costs little and
  is itself a trust requirement for a privacy product (a closed client cannot credibly claim
  "we can't read your mail"). Competitors hosting DMTAP grow the network the operator is central
  to.

## 12.5 Honest limits

- **Free-rider economics:** paying (convenience) customers subsidize free self-hosters. Intended
  and viable (the convenience majority pays; the sovereign minority self-hosts), provided the
  paid tier genuinely covers cost + margin.
- **Commoditization:** open software means thin margins on the software itself; margin comes
  from operations and scale, not licenses.
- **No protocol control:** required for adoption, but it means an operator can be out-executed —
  the defense is execution and trust, not a license.
- **The open-core temptation recurs:** pressure to move "just one" feature behind the paywall
  will appear. The inviolable rule (§12.3) is the bright line; drift there is fatal to the
  brand and is prohibited for privacy/crypto features.

## 12.6 Organization administration & the seam (normative)

Org / domain administration (§3.10) is an **organizational concern**, so it lives squarely on the
operator seam (§12.2) — and the inviolable rule (§12.3) draws the honest line through it:

- **Provisioning maps to the seam's Provisioning capability (§12.2).** Creating, suspending, and
  offboarding `name@abc.com` and org groups (§5.8.7) is exactly the seam's **Provisioning** hook
  across onboarding tiers A/B/C (§3.8). A hosted operator running the domain supplies the admin
  console; a self-hosted domain authority (§3.10.1) uses the same operations against the
  unlimited/no-op self-host default (§12.2) — an org can self-administer its own domain with no
  operator at all.
- **Directory, roles, and quotas are behind the seam; keys and crypto are NOT.** Curating the GAL
  (§3.10.3), assigning admin roles (§13.5.1), and per-member quotas are org/operations concerns the
  seam MAY meter and gate (**Policy**, §12.2). But the inviolable rule (§12.3) forbids any seam
  hook that would disable a member's encryption, weaken metadata privacy, or deny a member access
  to **their own keys or mailbox**. In particular, a **sovereign** member's key (§3.10.2(a)) is never
  a seam-controllable object: no operator plan, quota, or admin action can read it, escrow it, or
  lock the member out of it. The org controls the **name and the operations**, never a sovereign
  member's key.
- **Org-managed escrow is a disclosed member arrangement, not a seam backdoor.** The org-managed
  model (§3.10.2(b)) lets the *org* hold a member's key for compliance — but that is an explicit,
  disclosed, per-account arrangement visible to the user (`custody = "org-managed"`, §18.4.7),
  **not** a hidden operator hook and **not** applied to sovereign accounts. It does not violate
  §12.3 because it disables nothing in the protocol and hides nothing: the member is told, at
  provisioning, that this account's key is org-held. A conformant operator MUST NOT offer a control
  that escrows a **sovereign** account's key, or that presents an org-managed account as sovereign.

This keeps a real management console fully expressible on the seam while the sovereignty ethic
holds: **admin power and premium are for running the domain and its operations, never for reaching
into a member's keys** (§12.3).

## 12.7 Gateway billing is auditable via transport-path provenance (normative)

The seam meters **gateway operations** — legacy sends/receives — as a real cost center (§12.2
Metering; §7.9). Transport-path provenance (§7.8) makes that billing **auditable to the user**,
closing the gap between "the operator says you used the gateway" and "you can verify you did."

- **Billing attaches to gateway operations only, never to native mesh delivery.** A message that
  crossed a legacy gateway carries (inbound) or produced (outbound) the mandatory §7.2a
  attestation; a **pure-mesh** DMTAP↔DMTAP message carries none (§7.8.1(b)) and is, by
  construction, **not a gateway operation**. Per the inviolable rule (§12.3), native delivery and
  every privacy/crypto path are **never** metered or gated. A self-hoster reaching only DMTAP
  correspondents therefore incurs **zero** gateway billing (§7.9).
- **Every billable legacy operation is provenance-backed.** Because each gateway-touched message
  bears a verifiable `GatewayAttestation` (§18.3.11) naming the gateway `domain` and receipt time,
  a user can match each metered charge to a real message that **actually used the gateway** — the
  message's own `ProvenanceRecord` (§18.8.1) is the receipt. A charge with no corresponding
  attested message, or a **pure-mesh** message appearing on a gateway bill, is a **detectable
  billing error**, not something the user must take on faith.
- **Self-host authorization is a `GatewayAuthz` policy matter, disclosed.** A self-hoster's use of
  a *third-party* gateway is governed by the operator's `GatewayAuthz` policy (§12.2) plus the
  DKIM-delegation / MX pointing the user performs (§7.3, §7.9), all of which are visible to the
  user (a DNS act they take, an accountable identity token they present, §9). No hidden operator
  hook can bill a user for traffic that did not cross the operator's gateway, and the attestation
  chain is what makes that guarantee **checkable**, not merely promised.

This is metering as **transparency**: the operator bills exactly the gateway operations the
protocol makes cryptographically visible, and the user can independently audit the bill against the
messages' own provenance — consistent with §12.3 and the honest-limits ethic (§12.5).

## 12.8 Operational & security procedures (considerations)

The clauses above (and §1–§11) specify the *mechanisms*; a serious security protocol also needs the
**operational layer** that turns "we believe it is safe" into "here is what is checkable when
something goes wrong." This subsection is DMTAP's RFC-style **Security / Operational
Considerations**: coordinated disclosure, incident-response runbooks, key-ceremony guidance, the
audit gate, deprecation, and operator lifecycle. It is normative where it says MUST/SHOULD and
otherwise guidance. Two repository-root companion documents restate the human-facing parts for
discoverability: **[`SECURITY.md`](../SECURITY.md)** (how to report a vulnerability) and
**[`GOVERNANCE.md`](../GOVERNANCE.md)** (who decides, and the audit gate). Where they and this
section overlap, this section governs (§10.4).

### 12.8.1 Coordinated vulnerability disclosure (CVD)

DMTAP is a cryptographic protocol; a flaw can silently defeat SP-1–SP-11 (§6.9) for every user.
Disclosure is therefore **coordinated**, not full-drop, and is governed by `SECURITY.md`:

- **Private report channel.** Security reports go to **`security@envoir.org`** (PGP key published in
  `SECURITY.md`), or via the repository's **private security-advisory** facility — **never** a
  public issue for an unfixed vulnerability. A reporter SHOULD include an affected §/clause, a
  reproduction or proof, and the SP-*n* property (§6.9) or §10.7 invariant they believe is broken.
- **Acknowledgement + triage.** The maintainers **SHOULD acknowledge within 3 business days** and
  give an initial severity assessment within **10 business days**, mapping the report to a spec
  clause and to an SP-property/§10.7 row.
- **Embargo window.** The default coordinated embargo is **90 days** from acknowledgement, extendable
  by mutual agreement for a hard-to-fix protocol/wire change (a longer window than a single codebase
  needs, because independent implementers must ship in lockstep — §10.4). A fix MAY be released
  earlier; the embargo ends at the **earlier** of the fix's public release or the agreed date.
- **CVE + disclosure.** For a confirmed vulnerability the maintainers **request a CVE** and publish a
  **security advisory** at disclosure naming the affected versions/clauses, the SP-property broken,
  the fix (or spec erratum + capability-negotiated mitigation, §12.8.5), and credit to the reporter.
  A **spec-level** defect is filed and corrected per §10.4 (the spec governs; the reference is a
  proof, not the authority) and, if it changes crypto or wire, triggers the §12.8.4 audit gate.
- **Safe harbour.** Good-faith research — testing against **your own** identity/node/deployment,
  no access to or exfiltration of others' data, no service degradation, honoring the embargo — will
  **not** be pursued by the project. This is a research-safe-harbour statement, not legal advice, and
  is restated in `SECURITY.md`.
- **Bug bounty is POST-deployment only.** There is **no monetary bounty pre-launch** — there is no
  live production target to attack, and a bounty against a spec/reference under active hardening
  would mis-price the work. A bounty is established **only once a production deployment exists**, and
  is **distinct** from the pre-deployment external audit gate (§12.8.4): the audit is a *gate the
  project pays for before shipping*; the bounty is *continuous crowd-sourced review after shipping*.
  Until then, CVD above is the channel, and coordinated research is welcomed under the safe harbour.

### 12.8.2 Incident-response runbooks

Three protocol events are **detected-and-halted by design** (they raise `HALT_ALERT`, §21.2); each
has a defined operator + user response. The protocol does the detection; these runbooks do the
recovery.

- **Detected KT equivocation (§3.5.2(d), `0x0107`/`0x0110`).** The verifier has already HALTed and
  MUST NOT pin on the log's say-so. Response: (1) **preserve evidence** — the two conflicting
  signed STHs (or contradicting inclusion proofs) are self-authenticating, transferable proof
  (§3.5.2(d) step 3); (2) **gossip the evidence** to peers/auditors so the equivocation is globally
  attributable; (3) **recover on the honest quorum** — if a `> n/2` quorum of the pinned log set
  still agrees, proceed with the offending log **evicted** and its operator reputation-flagged
  (§7.5); if quorum breaks, fail closed (`0x0111`) and fall back to OOB verification (§3.4.1); (4)
  for DMTAP-Auth RPs this is a potential silent account-takeover — require multi-log consistency or
  an OOB pin (§13.7). On **v0-minimal** KT, where equivocation is only tamper-evident-after-the-fact
  (§6.6 item 6), the response is OOB re-verification of the affected binding plus operator escalation
  — the honest v0 limit, disclosed.
- **Committer / co-committer fork (§5.1, §5.1.1, `0x0404`).** The group has HALTed. Response: members
  compare hash-chained log heads, identify the **last common epoch**, and an `admin`/`owner`
  proposes a recovery Commit on top of it that is canonical **only** with the `> n/2` (or, for a
  2-member group, the trivial both-parties) member-signature quorum (§5.1 fork recovery); members on
  the losing fork roll back and re-apply; senders re-submit application messages stranded on the
  abandoned fork (§2.6 sender retry). This is the v0 manual stopgap pending Decentralized MLS.
- **Key or device compromise (§6.7).** Run the normative lost/stolen-device sequence from **any**
  surviving cluster device (or after recovery, §1.4): (1) **MLS Remove** the device from every group
  and the personal-device group (§5.8.2, §5.6); (2) **device-key rotation** re-keying any
  identity/recovery material it held (§1.5); (3) **session revocation** of all its auth sessions
  (§13.4, which a device-key rotation triggers wholesale); (4) **deniable-session teardown** —
  withdraw/rotate the deniable prekeys the device could hold and re-establish live deniable
  conversations (§5.2.1(f)), because the pairwise ratchet is outside MLS. Steps (1)–(2) advance every
  affected epoch and (4) reboots each ratchet, so the evicted key has **post-compromise** (decrypts
  nothing sent after eviction). If `IK` itself is suspected, additionally rotate `IK` (§1.5) and
  **immediately publish a `RecoveryPolicy` that evicts the compromised factors** (§1.4 reactive path)
  — a quorum-backed recovery **overrides any veto** (§1.4 rule 4), so a stolen not-yet-removed factor
  cannot block its own eviction. Confidential shared folders re-key on removal (§6.7).

- **Steady-state monitoring cadence (what *produces* the detections above).** The three runbooks
  are triggered by continuous monitoring an operator MUST run, not by chance observation. A
  conformant deployment operates, at the pinned cadences (§16.2, §16.3): (1) **KT STH gossip +
  consistency checks** at least once per gossip interval (≤ 1 h) and treats any STH older than the
  freshness window (≤ 24 h) as stale — the split-view/freeze detector (§3.5.2(a), `0x0112`); (2)
  **owner self-monitoring** of every log in the identity's pinned set (STH poll ≤ 6 h) so an
  unauthorized entry raises `HALT_ALERT` (§3.5.2(c)); (3) **≥ 2 independent auditors per log**
  checking append-only extension name-agnostically (§3.5.2(c)); (4) **mix-directory freshness**
  checks (≤ one mix-key epoch) so a frozen fleet view fails closed (§4.4.2, `0x0311`); and (5)
  **loop-return telemetry** feeding the active-attack detector (§4.4.7, `0x030F`). These cadences
  are the operational half of the trust-minimization triad — a trusted party (KT log, mix directory
  authority) is *minimized* by quorum, made *detectable* by this monitoring, and *fails closed* on
  the codes above. An operator that runs the mechanisms but not this cadence has the detectors and
  never looks at them; the cadence is what makes "detectable" real.

### 12.8.3 Key-ceremony guidance

The strength of SP-1/SP-2/SP-10 (§6.9) rests on how the long-lived keys are generated and held.
DMTAP does not invent ceremony crypto; it composes §1/§3/§5 primitives, and RECOMMENDS:

- **Identity key (`IK`) generation & custody.** Generate `IK` on a device and, where the platform
  allows, **inside a hardware keystore** (Secure Enclave / TPM 2.0 / StrongBox / TEE) as a
  **non-exportable** key (§1.2a). Because the deniable mode keeps `IK` **sign-only / DH-free**
  (§5.2.1(a)), a usage-fixed sign-only keystore slot suffices — provision it that way. `IK` is used
  **rarely** (it certifies device keys and recovery policy, §1.2); hold it in **cold / recovery
  custody** (§1.4) for day-to-day operation, signing with device subkeys. Record the protection
  class in `DeviceCert.key_protection` and, for attestation-gated contexts, attach platform
  key-attestation evidence (§1.2a), refreshed within the re-attestation cadence (**≤ 90 days**).
- **Recovery-quorum setup (§1.4).** Provision **multiple independent, redundant, rotatable** factors
  (phrase + devices + social guardians) so no single loss is fatal, and set `rotate_threshold` **>**
  `recover_threshold` so a single recovered factor cannot rewrite the policy. Use **Verifiable**
  secret sharing (Feldman/Pedersen VSS), **SLIP-0039** for the mnemonic⊕Shamir encoding, and
  **strongly prefer FROST (RFC 9591)** so guardians *authorize* recovery without ever reassembling
  the key in one place. Publish the policy to KT and confirm the owner's monitor devices alert on
  changes (§1.4 rule 6, §3.5).
- **Domain-authority threshold ceremony (§3.10.1, §5.8.6).** For an org controlling `@domain`, the
  authority key SHOULD be **threshold-held by the domain-owner/admin set** (FROST-style over the §1.4
  machinery, §5.8.6), so rotating the domain anchor or the directory-signing key is a
  **threshold act**, never one admin's unilateral power (§3.10.4, §13.5.1). Perform the initial
  ceremony offline with the guardian set present; anchor the resulting authority `IK` at
  `_dmtap.domain` and in KT (§3.2, §3.10.1); rehearse the recovery/rotation path before it is needed.

### 12.8.4 Audit cadence (the disclosed pre-deployment gate)

DMTAP composes standards rather than inventing crypto (§10.5, §11), but **composition and transport
are where the novelty — and the risk — live** (the mixnet integration, sealed sender, the deniable
side-channel, KT federation). Therefore:

- **An independent external cryptographic and code audit MUST precede any production deployment.**
  This is a **disclosed gate**, not aspirational: a deployment carrying real user mail before a
  qualified third party has reviewed the crypto/protocol and the reference implementation is
  operating outside the project's stated posture. The gate covers the protocol (this spec) and the
  reference `node`/`gateway`/libraries.
- **Re-audit on any major crypto or wire change.** Adding or retiring an algorithm suite (§1.1),
  changing a signing preimage (§18.9), altering the Sphinx/mixnet construction (§4.4), or changing
  the deniable handshake (§5.2.1) re-opens the gate for the affected surface before that change ships
  to production. A CVD fix that touches crypto/wire (§12.8.1) triggers this.
- **Pre-deployment-reachable, distinct from the bounty.** This audit is **reachable before launch**
  (there is a spec and a reference to audit) and is **paid for by the project**; it is explicitly
  **distinct** from the **post-deployment** bug bounty (§12.8.1), which only begins once a live
  target exists. Stating both, and their ordering, is the honest gate (§6.6 voice): review *before*
  shipping, continuous crowd-review *after*.

### 12.8.5 Deprecation procedure (retiring a mechanism without a flag day)

DMTAP retires suites and mechanisms **via capability negotiation** (§10.2), never a coordinated
flag day — the same dual-stack machinery that lets it *add* them:

1. **Announce** the successor as a new suite/capability token (§1.1, §21.15, §21.22) and let it
   spread dual-stack; both old and new coexist while adoption grows (§21.25).
2. **Ratchet.** As peers advertise the successor, the **suite high-water-mark** (§1.3) and
   **monotonic capability version** (§10.2, `0x030A`) make the upgrade *stick* per contact — a peer
   cannot be silently rolled back to the retiring mechanism (SP-8, §10.7.1).
3. **Retire.** The owner performs an explicit `IK`-authorized retirement (§1.5) — e.g. a
   `classical_retired` marker in `Identity` (§1.3) — which is the **only** way a high-water-mark
   lowers, making rejection of the retired mechanism unconditional. A verifier that cannot validate a
   peer's highest offered suite fails closed rather than downgrading (§1.3, §10.1).

The retirement is thus **monotone and per-contact**: no global cutover, no window in which an
adversary can force the old mechanism, and no user stranded because their counterpart has not yet
upgraded (they interoperate on the highest mutual capability until the owner retires the old one).

### 12.8.6 Operator onboarding & offboarding (gateway + mix)

Gateway (§7) and mix (§4.4.8) operators are **accountable, attested identities**, not anonymous
infrastructure. Their lifecycle:

- **Onboarding — attestation.** A **gateway** operator publishes its attestation key under the served
  domain (`<sel>._dmtap-gw.domain`, §7.2a) and its directory entry `{pubkey, reputation, region,
  price, stake}` (§7.5); a **mix** operator publishes a `MixNodeDescriptor` under a KT-auditable
  `node_ik` **and** a `_dmtap-mix` operator attestation under its domain (§4.4.8) — and **only an
  attested operator counts toward path operator-diversity** (§4.4.8, §10.7.2), which is what stops a
  single party minting *N* fake operators. Both SHOULD **post stake/bond** (§9.6, §4.4.8), making
  Sybil fleets costly.
- **Reputation.** Selection is reputation-weighted: gateways by measured deliverability-to-destination
  (§7.5, §9.6), mixes by loop-return reliability feeding a selection weight (§4.4.7–§4.4.8). Neither
  reputation model is a trust root — a gateway cannot forge identity (DKIM delegation separates
  deliverability reputation from the user's key, §7.3), and a mix directory **indexes, it does not
  forge** (§4.4.2). Misbehavior (KT/attestation failures, dropped loops, spam vouching) **down-scores
  and slashes** rather than merely warns.
- **Offboarding / revocation — zero lock-in.** A gateway is swapped by a **DNS/DKIM change** with no
  data migration (§7.7) — the box is the authority, so a user drops or switches an operator freely; a
  self-host backstop is always available (§7.7). A mix is removed by the directory authority
  (**detectably**, since the directory is KT-anchored and rollback-defended, §4.4.2) and, on proven
  misbehavior, has its stake slashed and its operator flagged (§9.6). An equivocating KT log operator
  is **evicted from the pinned set** on self-authenticating evidence (§3.5.2(d), §12.8.2). In every
  case revocation is a *reputation + configuration* act, never a protocol entitlement the operator
  can veto — the same non-lock-in property that makes open service survivable (§7.7).
