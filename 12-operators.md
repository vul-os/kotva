# 12. Operators, the Seam & User Protection

Somebody other than the user may end up running part of the path — a gateway for legacy mail, a
buffer while a phone is asleep, a box someone else owns. This section is about **what that person
can and cannot do to the user**. It is partly *informative* (what the seam is for) and mostly
*normative* (the inviolable rule, the never-chargeable list, and the auditability guarantee).

DMTAP has no control plane, no vendor, and nothing to sell (§12.4). Third parties run roles because
they want the network to exist. That makes this section **consumer protection**, not a business
model: the question it answers is not "how does the operator get paid" but "what is the operator
structurally unable to take from you."

## 12.1 Two deployment shapes

| Shape | Who runs the box | What it costs | What it limits |
|-------|------------------|---------------|----------------|
| **Self-host** | the user, on their own hardware | no money to anyone; the hardware, the connection, and the operator's own ongoing time (see the honest limit below) | nothing — every protocol, client, and privacy feature, unrestricted |
| **Someone else hosts** | a peer, a friend, a community, a third-party operator | whatever those two parties agree, including nothing | conveniences only — never capability, never privacy, never access to your own data |

This is a distinction between **whose machine it is**, not between a free tier and a paid tier.
There is no paid tier in DMTAP because there is no seller. A user whose box is run by someone else
has handed over *operation*, never *authority*: the keys are theirs (§1.2), the mailbox decrypts
only to their devices on the native path (§8.1), migration is a DNS edit (§7.7), and the naming
ladder's floor is derived from their own key and cannot be revoked by anyone (§3.9.6).

**The normative consequence, stated plainly: a self-hosting user with no legacy correspondents pays
nobody, ever, and loses no capability.** Not a reduced feature set, not a "community edition," not
a rate limit — the full protocol, including metadata privacy, group messaging, files, recovery, and
web login. There is no party in that user's path to charge them, which is a stronger guarantee than
a promise not to (§12.3).

**Honest limit: "pays nobody" is a claim about money, not about effort.** Running a box is ongoing
systems administration — upgrades, backups, key custody, certificate renewal, storage growth, and
diagnosing failures with nobody on call. Matrix is the closest deployed precedent for self-hosted
infrastructure of this shape, and its operators report exactly this burden accreting over time
until many abandon their servers; no protocol design legislates that away, and this one does not
claim to. What the paragraph above asserts is that **no party can charge a self-hoster or withhold
capability from them**, and that holds. It is not an assertion that self-hosting is effortless, and
a reader should not take it as one. The "someone else hosts" row exists precisely because trading
*operation* for someone else's time is a legitimate and expected choice — it costs authority
nothing (§12.1), which is the property this specification actually defends.

## 12.2 The operator seam

The seam is the small boundary an implementation exposes so that *someone else's* box can enforce
*its own* policy without forking the protocol. It is four capabilities, and they are not equals —
two are load-bearing, two are optional conveniences that default to nothing (reference crate:
`crates/dmtap-seam`, contract: its `CONTRACT.md`):

**Load-bearing (a conformant implementation implements these):**

- **GatewayAuthz** — authorise legacy egress with per-identity-token accountability (§9),
  preserving sealed sender. This is a **security** control, not a commercial one: it is what stops
  the gateway role becoming an open relay (§7.11.2), and it **MUST NOT fail open** (below). Its
  wire form — the gateway-local authorisation record, and the `CapabilityToken`-based per-address/
  per-rail grant extension — is defined at §18.8a.3.
- **Policy** — per-deployment limits on *operations* (storage caps, send caps, domain counts, rate
  limits). It exists because a box has finite resources, not because someone is selling them; the
  self-host default is unlimited.

**OPTIONAL, with no-op defaults (an implementation MAY omit them entirely and remain fully
conformant):**

- **Metering** — emit usage events at real cost centres. **Default: no-op.** DMTAP does not require
  usage to be counted, and an implementation that never counts anything is conformant. Metering
  exists for the case where a third party carries a genuinely scarce cost — legacy egress (§7.1a) —
  and wants to know its size.
- **Provisioning** — create/suspend accounts and `@`-addresses across onboarding tiers A/B/C
  (§3.8). **Default: no-op.** A self-hosting identity provisions itself by generating a keypair
  (§1.2); an org uses this hook to administer its own domain (§12.6).

Each capability has a **self-host default that is unlimited/no-op**, so with no operator the
software runs unrestricted — and that default is the *normal* case, not a fallback.

**Payment rails are out of scope, explicitly and permanently.** How a third party might collect
money — card processors, invoices, bank transfer, cryptocurrency, a tip jar, or nothing at all — is
**operator policy**, has no protocol surface, and MUST NOT acquire one. **DMTAP defines no token
and never will** (§3.12.5(d)): it does not mint, require, endorse, or presuppose any coin, and no
seam capability, registry, or extension may introduce a protocol-level payment path. Postage (§9.5)
is a signed real-money voucher, not a currency, and is an anti-abuse mechanism a recipient may
choose to accept — never a rail the protocol depends on.

**Fail-open to function, fail-safe on security (MUST):** if the party behind the seam is
unreachable, the implementation MUST NOT break user-facing mail/chat/files. Metering (where
implemented at all) queues and retries — an undercount is a rounding error, a stalled mailbox is a
failure — and quota **Policy** falls back to allow, because a resource limit is a housekeeping
concern, never a security one. **GatewayAuthz is different and MUST NOT fail open to "allow."** GatewayAuthz is a security
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

Privacy, cryptography, metadata privacy, and recovery MUST NOT be behind the seam, a quota, or a
charge. There MUST be no seam hook, quota, plan gate, or operator control capable of disabling
encryption, weakening the mixnet, reducing metadata privacy, or denying a user access to their own
keys or mailbox. The seam limits **operations and organisational concerns only**.

### 12.3.1 The never-chargeable list (normative, exhaustive by category)

No operator, and no conformant implementation, may condition, meter, gate, degrade, or charge for
any of the following. This is not a pricing recommendation — it is a statement about what the
protocol makes impossible to sell, and each entry has a structural reason, not merely a promise:

1. **Privacy and cryptography.** End-to-end encryption, signing, suite selection, the suite
   high-water-mark (§1.3), key rotation and PQ migration (§1.5, §1.1). *Structural reason:* these
   run entirely between endpoints; no operator is a participant.
2. **Metadata privacy.** The opt-in, research-tier `private` tier, mixnet path building, entry
   guards, cover traffic, the profile in force
   ([docs/research/mixnet.md §4.4](docs/research/mixnet.md)), the no-silent-downgrade rule
   ([docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)) — where an implementation offers
   the mixnet at all (§4.6). *Structural reason:* the mix role is a permissionless, opt-in
   capability of the node binary, not a default behaviour
   ([docs/research/mixnet.md §4.4.2a](docs/research/mixnet.md)); there is no fleet owner to bill
   for it, and §12.2's Policy hook MUST NOT be consulted for any privacy decision.
3. **Recovery and continuity.** The recovery policy and its factors (§1.4), name migration (§1.6),
   the portable encrypted backup (§1.4). *Structural reason:* a user locked out of recovery is a
   user whose identity has been captured; charging for it would make capture a business model.
4. **Native node-to-node delivery.** Every DMTAP↔DMTAP message, group message, file transfer, and
   control MOTE (§4, §5, §7.7). *Structural reason:* **no operator is on the path, so there is
   nothing to bill.** This is not forbearance; there is no party in a position to invoice, and any
   claim that a pure-mesh message incurred a charge is falsifiable by the recipient (§7.8.1(b),
   §12.7).
5. **Access to your own keys, mailbox, and data.** Reading, exporting, and migrating your own MOTE
   store; the JMAP surface over your own node (§8.1); the export/backup path (§1.4). *Structural
   reason:* the store is at the edge and encrypted to keys only the user's devices hold; an
   operator that could withhold it would have to hold it, which the native architecture does not
   permit.
6. **Reachability by key-name and the zero-relationship delivery floor** (§3.9.6, §9.7a). *Structural
   reason:* the floor is a conformance requirement on *recipients*, not a service; a keypair with a
   few seconds of work always reaches the requests area, with no issuer, no postage, and no
   operator.

**The one thing outside this list** is legacy SMTP egress and ingress (§7.1a) — the only function
requiring a resource that cannot be self-provisioned — together with the two conveniences that ride
on it (a vanity local-part in someone else's domain, and legacy-client service, §7.14). That is the
entire chargeable surface of **the in-tree SMTP/mail gateway**, and it is exactly co-extensive with
the one operator class the architecture admits (§0.5). It is not the entire chargeable surface of
DMTAP: the Legacy Adapters extension (§26) generalises the gateway pattern to other legacy rails
and adds a second, smaller, **credential-gated rather than resource-gated** category of legitimate
charge — see §26.10 for that surface.

### 12.3.2 Conformance

A conformant implementation MUST NOT expose any control that violates §12.3.1, MUST NOT consult the
seam for any privacy/crypto decision, and MUST NOT ship a build in which any §12.3.1 item is
disabled, degraded, or delayed by the absence of an operator relationship. A user with no operator
relationship at all MUST observe no functional difference on any of the six categories above.

## 12.4 Licensing and the absence of a control plane (informative)

There is no business model here to describe, and this section exists to say so precisely enough
that the absence cannot be quietly filled in later:

- **The specification and the reference implementation are MIT-licensed** (Apache-2.0
  dual-licensing under consideration for its explicit patent grant). Everything a user touches, and
  everything trust depends on, is open — protocol, spec, node, gateway role, client, crypto. A
  closed client cannot credibly claim "we cannot read your mail," so openness is a correctness
  requirement here, not a marketing posture.
- **There is no control plane.** No registry of users, no central provisioning service, no
  license server, no telemetry endpoint, and no hosted component any implementation must talk to.
  The seam of §12.2 is a *local* interface for a box's own policy, not a client of anything.
- **Operators run gateways because they want the network to exist.** That is the whole incentive
  model, and it is the same one under every other role (§0.2.2) — you run what you want to use.
  Whether an operator asks anyone for money is between that operator and its users (§7.13, §7.14);
  the protocol neither knows nor cares, and provides no rail for it (§12.2).
- **DMTAP has no token and never will** (§3.12.5(d), §12.2).

## 12.5 Honest limits

- **Volunteer infrastructure may not materialise.** The mix fleet, the KT log set, the rendezvous
  layer and the buffer roles are all provided reciprocally by whoever shows up. Nothing guarantees
  anyone shows up. The disclosed consequence — degradation to `fast`-tier direct delivery, still
  encrypted and authenticated but without default metadata privacy — is stated in §6.6 item 13, and
  it is the honest price of having no commercial engine behind the infrastructure.
- **Legacy egress may be scarce even when nobody is charging for it.** The requirement is an IP
  with reputation and unblocked port 25 (§7.1a); goodwill does not produce those. A period with few
  gateways looks, to a user with legacy correspondents, exactly like a period with expensive ones.
- **"Someone else hosts" is still someone else.** A third party running a box for a user can go
  away, lose data it never should have had, or be compelled. DMTAP bounds what that costs (keys
  stay at the edge, migration is a DNS edit, the native path is content-blind) but does not make it
  free — and the legacy-client surfaces are explicitly *not* content-blind (§7.15.3).
- **The open-core temptation recurs.** Pressure to move "just one" feature behind a gate will
  appear the moment anyone tries to fund operations. §12.3.1 is the bright line, and it is written
  as a list precisely so that crossing it requires an argument in public.

## 12.6 Organisation administration & the seam (normative)

Org / domain administration (§3.10) is an **organisational concern**, so it lives squarely on the
operator seam (§12.2) — and the inviolable rule (§12.3) draws the honest line through it:

- **Provisioning maps to the seam's Provisioning capability (§12.2).** Creating, suspending, and
  offboarding `name@abc.com` and org groups (§5.8.7) is exactly the seam's **Provisioning** hook
  across onboarding tiers A/B/C (§3.8). A third party running the domain supplies the admin
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
holds: **admin power is for running the domain and its operations, never for reaching into a
member's keys** (§12.3).

## 12.7 A user can audit any usage claim made against them (normative)

Where a third party *does* carry legacy egress for someone, transport-path provenance (§7.8) gives
the **user** — not the operator — the evidence. This is a protection against the operator, and it
is stated in that direction deliberately: the guarantee is not "the operator can prove its
invoice," it is "**the user can disprove a false one**."

- **Only gateway operations are observable to any operator; native mesh delivery is not.** A message
  that crossed a legacy gateway carries (inbound) or produced (outbound) the mandatory §7.2a
  attestation; a **pure-mesh** DMTAP↔DMTAP message carries none (§7.8.1(b)) and involved no
  operator at all. Per the inviolable rule (§12.3), native delivery and every privacy/crypto path
  are **never** metered or gated. A self-hoster reaching only DMTAP correspondents therefore has
  nothing any operator could even count (§7.9).
- **Every claimable legacy operation is provenance-backed.** Because each gateway-touched message
  bears a verifiable `GatewayAttestation` (§18.3.11) naming the gateway `domain` and receipt time,
  a user can match each claimed charge to a real message that **actually used the gateway** — the
  message's own `ProvenanceRecord` (§18.8.1) is the receipt. A charge with no corresponding
  attested message, or a **pure-mesh** message appearing on a gateway bill, is a **detectable
  billing error**, not something the user must take on faith.
- **Self-host authorisation is a `GatewayAuthz` policy matter, disclosed.** A self-hoster's use of
  a *third-party* gateway is governed by the operator's `GatewayAuthz` policy (§12.2) plus the
  DKIM-delegation / MX pointing the user performs (§7.3, §7.9), all of which are visible to the
  user (a DNS act they take, an accountable identity token they present, §9). No hidden operator
  hook can bill a user for traffic that did not cross the operator's gateway, and the attestation
  chain is what makes that guarantee **checkable**, not merely promised.

This is the user's protection made checkable: the only operations any operator can even see are the
ones the protocol marks cryptographically, and the user holds the marks. Consistent with §12.3 and
the honest-limits ethic (§12.5).

## 12.8 Operational & security procedures (considerations)

The clauses above (and §1–§11) specify the *mechanisms*; a serious security protocol also needs the
**operational layer** that turns "we believe it is safe" into "here is what is checkable when
something goes wrong." This subsection is DMTAP's RFC-style **Security / Operational
Considerations**: coordinated disclosure, incident-response runbooks, key-ceremony guidance, the
audit gate, deprecation, and operator lifecycle. It is normative where it says MUST/SHOULD and
otherwise guidance. Two repository-root companion documents restate the human-facing parts for
discoverability: **[`SECURITY.md`](SECURITY.md)** (how to report a vulnerability) and
**[`GOVERNANCE.md`](GOVERNANCE.md)** (who decides, and the audit gate). Where they and this
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
  abandoned fork (§2.6 sender retry). This is the v0 manual stopgap pending Decentralised MLS.
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
  unauthorised entry raises `HALT_ALERT` (§3.5.2(c)); (3) **≥ 2 independent auditors per log**
  checking append-only extension name-agnostically (§3.5.2(c)); (4) **mix-directory freshness**
  checks (≤ one mix-key epoch) so a frozen fleet view fails closed
  ([docs/research/mixnet.md §4.4.2](docs/research/mixnet.md), `0x0311`); and (5)
  **loop-return telemetry** feeding the active-attack detector
  ([docs/research/mixnet.md §4.4.7](docs/research/mixnet.md), `0x030F`) — items (4) and (5) apply
  only to an operator whose deployment offers the opt-in, research-tier mixnet. These cadences
  are the operational half of the trust-minimization triad — a trusted party (KT log, mix directory
  authority) is *minimised* by quorum, made *detectable* by this monitoring, and *fails closed* on
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
  **strongly prefer FROST (RFC 9591)** so guardians *authorise* recovery without ever reassembling
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
  reference node binary (all roles, §0.2) and its libraries.
- **Re-audit on any major crypto or wire change.** Adding or retiring an algorithm suite (§1.1),
  changing a signing preimage (§18.9), altering the Sphinx/mixnet construction
  ([docs/research/mixnet.md §4.4](docs/research/mixnet.md), for implementations that offer it), or
  changing
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
3. **Retire.** The owner performs an explicit `IK`-authorised retirement — a `classical_retired`
   marker in `Identity` (**§1.3**, the suite-ratchet floor; the marker travels in the signed
   `Identity` object, not the key-rotation path §1.5) — which is the **only** way a high-water-mark
   lowers, making rejection of the retired mechanism unconditional. A verifier that cannot validate a
   peer's highest offered suite fails closed rather than downgrading (§1.3, §10.1).

The retirement is thus **monotone and per-contact**: no global cutover, no window in which an
adversary can force the old mechanism, and no user stranded because their counterpart has not yet
upgraded (they interoperate on the highest mutual capability until the owner retires the old one).

**Honest limit — retirement is per-owner and slow; there is no global kill-switch (by design).**
Because a high-water-mark lowers only through **each owner's own `IK`-authorised** action (§1.3), a
compromised primitive is retired **identity-by-identity, at the pace owners act** — not
network-wide by fiat. This is deliberate **anti-capture**: a global "disable suite X everywhere"
lever would itself be a single point an adversary, or a coerced authority, could pull to **force** a
downgrade. The residual is disclosed, not hidden: a slow retirement tail during which un-upgraded
owners still accept a weakened primitive (the same reason suite `0x03` is **pre-reserved** as a
standing AEAD-diverse target, §1.1, so the migration itself is ready before it is needed). As an
**OPTIONAL, advisory-only** mitigation, a quorum of independent, KT-anchored operators
(mix/gateway/log authorities, §12.8.6) MAY publish a **KT-logged "suite-compromise advisory"** that
clients MAY surface to prompt owners to retire faster. It is **never mandatory, never automatic, and
never itself lowers a high-water-mark** — only the owner's `IK` can — so it informs without becoming
the very central kill-switch the design refuses.

### 12.8.6 Role lifecycle — joining and leaving (gateway + mix)

Anyone may take a role (§0.2.2); what the specification requires is that the two roles making
**claims about who they are** — gateway and mix — be **accountable, attested identities** rather
than anonymous infrastructure. Their lifecycle:

- **Joining — attestation.** A **gateway** publishes its attestation key under the served domain
  (`<sel>._dmtap-gw.domain`, §7.2a) and, optionally, a self-signed **discovery descriptor** carrying
  no score, price, or bond (§7.5); a **mix** — an opt-in, research-tier role that exists only for
  an implementation offering the opt-in mixnet ([docs/research/mixnet.md](docs/research/mixnet.md))
  — publishes a `MixNodeDescriptor` under a KT-auditable
  `node_ik` **and** a `_dmtap-mix` operator attestation under its domain
  ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md)) — and **only an
  attested operator counts toward path operator-diversity**
  ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md), §10.7.2), which is what stops a
  single party minting *N* fake operators. **No bond or stake is required, offered, or implied**
  ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md)'s normative note, §9.6): enforcing one
  would need an adjudicator empowered to seize funds,
  a more powerful authority than anything else in this document. Sybil cost comes from attestation
  plus ASN/jurisdiction diversity instead.
- **Reputation is measured locally, by each participant.** Gateways are ranked by the *sending
  node's own* measured deliverability-to-destination (§7.5, §9.6) — never by a published network
  score, which would need the very authority [docs/research/mixnet.md §4.4.2](docs/research/mixnet.md)
  removed. Mixes (where offered) are weighted by each client's own
  loop-return statistics ([docs/research/mixnet.md §4.4.7–§4.4.8](docs/research/mixnet.md)). Neither
  model is a trust root: a gateway cannot forge
  identity (DKIM delegation separates deliverability reputation from the user's key, §7.3), and the
  mix fleet view is **derived from the KT logs, not signed by anyone**
  ([docs/research/mixnet.md §4.4.2](docs/research/mixnet.md)). Misbehavior loses
  path share and traffic automatically under any conforming weighting; there is nothing to slash and
  nobody to do the slashing.
- **Leaving / revocation — zero lock-in.** A gateway is swapped by a **DNS/DKIM change** with no
  data migration (§7.7) — the box is the authority, so a user drops or switches freely; the
  self-host backstop applies to anyone who can meet §7.1a. A mix leaves by ceasing to publish a
  current-epoch descriptor, and is dropped from every client's derived view at the next epoch
  ([docs/research/mixnet.md §4.4.2, §4.4.4](docs/research/mixnet.md)) — no authority evicts it,
  because there is no authority. An equivocating KT log
  operator is **evicted from the pinned set** on self-authenticating evidence (§3.5.2(d), §12.8.2).
  In every case revocation is a **local decision each participant makes for itself**, never a
  protocol entitlement anyone can veto — the same non-lock-in property that makes open service
  survivable (§7.7).
