# KOTVA spec-perfection — driven by the Wakala session (founder-authorised)

Temporary process file (delete at convergence). Edits land directly in kotva; **no co-author footer**.
The Wakala session is driving this pass per founder; the spec session should HOLD spec edits to avoid
collision (see wakala COORDINATION.md).

## Founder calls — DECIDED (agents obey these)
- **MIXNET = DEMOTE.** Move `04-transport §4.4` (Sphinx/Loopix mixnet, ~883 lines) and `09-anti-abuse
  §9.4.1` (VDF, ~115 lines) to a new `research/` dir as **non-normative / experimental**, leaving short
  stub pointers. **Flip the transport default off the mixnet/`private` tier to `fast`/direct.** Demote
  `06-privacy` SP-3/SP-4/§6.10. **Restate the metadata-privacy claim honestly**: sealed-sender
  *reduction* vs intermediaries — NOT global-passive-adversary immunity (a global observer can still
  recover the graph via IP+timing correlation). Keep the mixnet as an **opt-in tier + a stated roadmap
  goal**. Reconcile DIRECTION §9, THREAT-MODEL SEC-9/R-9, README, 00-overview, substrate/README+ROLES so
  they all agree (they already say "research-tier" — make the flagship match).
- **ECONOMICS = SEAM ONLY.** CONTRACT §6 keeps the *mechanism* normative (signed tariff object, signed
  usage-receipt object, settlement seam over an existing asset, no token, no published price-rank, stake
  verified on-rail, **charge for service — never for deliverability/classification**) and the *numbers*
  out of scope (operator policy). Do NOT add pricing / rails / billing models. Add ONE honest sentence:
  "whether charge-for-service sustainably funds coordinators is an open question."
- **PQ:** classical suite `0x01` is the interoperable floor; `0x02` (X-Wing + ML-DSA composite) is
  **PROVISIONAL**, pinned to a specific draft revision — not hard-mandated.
- **Personhood:** require interop with **≥2 structurally-different** bindings as a v0 target + a non-crypto
  day-one path; disclose the single-vendor (World ID) fragility.
- **Custodial escrow:** disclose + accept (the one honest load-bearing exception); hold it to the same
  MUST-verify-stake bar as other kinds; add it as a disclosed self-host exception class in CONTRACT §2.3.
- **Naming:** keep legacy brands (DMTAP / TRACT / WRAP) primary as aliases; **wakala** = the coordinator
  umbrella (provisional resolved).
- **SA/British English + RFC layout: LAST**, after correctness is frozen (don't re-spell churned text).

## Guardrails (every wave)
Perfect, don't rewrite. PRESERVE all normative content, every honest-residual, every security MUST, every
wire byte/CBOR key/DS-tag/error code, every RFC citation. Never introduce an overclaim. Keep cross-refs
resolving + BCP-14 correct. Commit per wave; `git pull --rebase` before push; one writer per doc-cluster.

## Waves
**W1 (substantive, cross-cutting — file-disjoint, run now):**
- **A1 governing alignment** — DIRECTION, 00-overview, README, THREAT-MODEL, SPEC, substrate/README,
  substrate/ROLES: apply the mixnet-demotion *statements* + canonical **six** waist capabilities (restore
  MOTE + Transport to substrate/README §2) + align every coordinator-kind count/tally to CONTRACT §5.
  Does NOT edit CONTRACT (A2 owns it).
- **A1b mixnet mechanics** — 04-transport §4.4, 06-privacy SP-3/SP-4/§6.10, 09-anti-abuse §9.4.1; create
  `research/mixnet.md` + `research/vdf.md`; move the content, stub pointers, flip the default.
- **A2 contract + wire-debt** — coordinator/CONTRACT (§5 canonical kinds; §6 economics + the honesty note;
  custodial-escrow bar §2.3/§6), 07-gateway, 12-operators, 26-legacy-adapters, 18-wire-format, 21-errors:
  write **GatewayAuthz / CoordinatorDescriptor / SignedTariff / UsageReceipt** CDDL + DS-tags + signing
  preimages (match the Wakala impl's logged descriptor layout where sensible), so "Accountable" is
  wire-checkable.

**W1-ripple (mixnet-demotion cross-file contradictions — fix right after A1b commits, one agent):**
The demotion moved §4.4 to `docs/research/mixnet.md` but these untouched docs still present the mixnet as
default/normative or dangle refs to the moved section — reconcile to opt-in/research + `fast` default:
- `14-scaling.md` §14 table "Mix … **default-on** (§4.4.2a)" + heading "14.6.3 Tie-in to the mixnet profiles
  (**normative**)" → opt-in/non-default + non-normative; repoint §4.4.* refs to `docs/research/mixnet.md`.
- `27-realtime-media.md` §150/§342 "`private` (mixnet) the default for mail and all control messages" →
  restate: default is `fast`; `private`/mixnet is opt-in/research.
- `18-wire-format.md §1740` `min_hops` — repoint moved `§4.4.10`/`§6.8` refs to `docs/research/mixnet.md`;
  soften "the mixnet's anonymity guarantee" to the opt-in/research framing.
- Sweep for any other doc still saying mixnet "default"/"normative"/"guarantee" (grep) and reconcile.

**W2 (substantive residuals — after W1 commits, watch overlaps):**
- [x] RecoveryPolicy §1.4 formal model (01/13) — Table A/D + `is_weakening()`; D0 eviction case `1aad934`.
- [x] HPKE-mode pinned Base (02/05/18) — `712c091`.
- [x] SYNC split minimal-core+extension + open-namespace determinism (substrate/SYNC) — `1ea9ac8`
  (core = OR-Set/LWW/Death-cert; ext = PN-counter/RGA/Movable-tree; vectors tiered, none changed).
  **Round-2 follow-up flagged:** ADOPTION.md calls RGA load-bearing for an editor product while SYNC
  now marks it optional — reconcile the language (a product MAY require an optional extension; not a
  contradiction, but the wording should agree).
- [x] premature-generality cuts — **RE-EVALUATED, mostly REJECTED on inspection (defect doesn't
  reproduce):** suites `0x03`-`0x05` are already `RESERVED` + honestly status-labelled and are the
  *designed crypto-agility / emergency-pivot scaffold* (§1.1 anchor rotation, §16.7, §18.2 byte
  arithmetic, §18.1.5 multihash) — an appendix move would dangle 30+ refs and break the migration
  narrative, i.e. a regression not a simplification. Keep as-is. compute-kind note + media-relay/
  reachability collapse: DEFER to critique round 2 — act only if a fresh lens confirms real
  over-engineering (avoid a risky kind-merge that ripples into the Wakala impl + CONTRACT §5).
- [x] multi-homing RECOMMENDED (substrate/ROLES) — `7c89506`.
- [x] REPUTATION→OpenRank "degraded" label + personhood ≥2 + discovery/indexer + coordinator-funding
  open problems (primitives/REPUTATION, bindings, profiles, DIRECTION/SPEC) — `5d05427`.
- [x] GOVERNANCE.md refresh + coordinator-contract ratification process (+ fail-open→fail-closed `e3328d0`).
- [x] §19.7.1 Payload.from=gateway-IK fix — already correct (step 3 sets gateway IK; CHANGELOG:139).

**Critique round 1 (workflow `wyr7famw7`) — 8 residuals, closing:**
- [x] §1.4a D0 eviction `1aad934` · [x] HPKE Base `712c091` · [x] trivial batch `e3328d0`
- [x] §18.8a wire-object cluster (suite/valid_until/terminating→declared/mime/mail-fast) `2b86e22`
- [x] 06/04 privacy (bulk onion-MUST vs §4.5 MUST-NOT contradiction; dangling §4.4.11) `675aad9`
- [x] cross-doc reconcile CONTRACT/07/26/21/10 to T's layout — `217fccc` (§26.11 false-premise fixed,
  Tariff.identity attribution added, 2 sibling mime values assigned non-colliding; 2 false-positives
  correctly skipped). **Critique round 1 CLOSED 8/8.**

**Critique round 2 (workflow `wp8pff72w`, 5 lenses + adversarial verify) — NOT clean: 6 confirmed, 1 refuted.**
Deferred questions ADJUDICATED as non-findings: media-relay vs reachability-adapter = distinct planes (SFU vs
NAT-ingress, composed in profiles/rtc.md); compute = disclosed provisional slot (not over-engineering);
ADOPTION/RGA = not a contradiction (ADOPTION calls *Yjs* load-bearing, RGA gaps are gaps). Refuted finding
(RecoveryPolicy recover_threshold "weakening") = actually a STRENGTHENING (adds a factor → harder), Table D
correctly silent. The 6 confirmed (each a different file → 2 disjoint fix-agents):
- [x] **F1 (HIGH)** substrate/SYNC.md §6.2 — stability cut redefined to min-of-`contiguous_below` (was
  min-of-`max_applied`); `StabilityMark.hlc` MUST carry contiguous_below; C-30 logged; reused `0x0A09`
  (byte-indistinguishable, so no receipt-time code); no conformance-vector change; min-of-min preserves
  convergence. `9b62798`. **Critique round 2 CLOSED 6/6.**
- [~] **F2 batch** (agent F2 in flight): (MED) coordinator/CONTRACT.md 5 cross-dir links missing `../`;
  (MED) 18-wire-format.md:1803 "kind … key 1" → key 2 (kind is key 2, suite is key 1); (MED) 17-parity.md:113
  stale "full mixnet privacy"/"onion" attachment-tier labels → fast default + never-mixnet bulk; (LOW)
  docs/research/coverage-matrix.md compute "does not exist / add it" → already a provisional kind; (LOW)
  11-grounding-and-references.md:75 unqualified "DMTAP targets a global passive adversary" → mixnet does; default = reduction.
After F1+F2 land → **critique round 3** (different lens mix); two consecutive clean → PERFECTED.

**Critique round 3 (workflow `wfmvmanpz`, fresh 5 lenses: adversarial-crypto · state-machine/liveness ·
implementer-interop · rfc-layout/bcp14 · cross-family) — NOT clean: 8 confirmed** (7 by the workflow + 1
HIGH I hand-verified after its verify-agent died on an API stall — the "1 refuted" was that miscount, not
a real refutation). Fix-agents (file-disjoint):
- [x] **X** {18-wire-format, 13-identity-auth} `8f322d0`: §18.9.8 `auth_hash` → `0x1e ‖ BLAKE3-256(…)`
  multihash form (+ §13.3 step 5); §18.3.3 PostageStamp/Vouch → reference §18.9.7 whole-object rule;
  §18.3.2 BlindedTag KDF pinned. Verified clean. **Conformance follow-up done:** DMTAP-AUTH-01 in
  SUITE.md/suite.json aligned to the multihash preimage `1984175` (auth vectors are construction-todo —
  no frozen bytes to regenerate).
- [x] **Y** {20-state-machines} `17390d6`: §20.1 RETRY totality gap closed — pre-seal RETRY now
  re-resolves (was condemning DNS/KT-lagged MOTEs to EXPIRED). Verified: pre-seal edge in table+diagram.
- [x] **Z** {00-overview, SPEC, STYLE, wrap/*, coverage-matrix, ADOPTION} `f7b0008`: §0.8 (+SPEC+STYLE)
  BCP-14 full set; WRAP four-of-six + canonical numbers; coverage-matrix SFU=`blind-routing`; ADOPTION §1.
- [x] **W** {02-mote, 06-privacy} `bef2927`: BlindedTag prose (§2.2a/§6.4) → reference X's §18.3.2 pin;
  honest limits preserved.

**Round 3 CLOSED 8/8.** → **critique round 4** (fresh lens mix: security-downgrade/composition ·
conformance-testability · xref-integrity · honesty-overclaim · reader-coherence). Two consecutive
fully-clean passes from DIFFERENT lens-sets → PERFECTED.

**Round 4 status — workflow `war2cklt5` INFRA-DEGRADED, NOT a valid pass.** Only 1/5 lenses returned
(security-downgrade: 1 finding, refuted → clean); the other 4 lenses died on API stalls (widespread
mid-stream stalls also hit round 3). The workflow's `overall_clean:true` is a FALSE clean (4/5 of the spec
un-critiqued). **Re-running the 4 missing lenses as resilient individual agents (sonnet, self-verifying,
≤3 concurrent):** conformance-testability `a657e6c7`, xref-integrity `ab9a888b`, honesty-overclaim
`af753942` in flight; reader-coherence queued. Only when ALL 5 lenses return clean does round 4 count as
pass 1 of 2. **Infra note:** stop using the high-concurrency opus workflow for critique; prefer individual
sonnet lens-agents (one stall no longer voids the batch).

**Round 4 = CLEAN (pass 1 of 2), 5/5 lenses.** security-downgrade/composition (surviving workflow agent —
1 finding, refuted); conformance-testability, xref-integrity, honesty-overclaim, reader-coherence
(inline-verified by the driver with evidence, because the re-run agents ALSO stalled mid-stream — a
persistent infra issue). Evidence: BlindedTag conformance row (DMTAP-REST-05) correctly cites §18.3.2 and
carries no stale KDF; no row asserts old postage/vouch preimage or max-applied cut; §20.1 table↔mermaid
agree on the RETRY→QUEUED pre-seal edge; `[§18.3.2](18-wire-format.md)` links resolve; lint 0 errors;
REACH §7 scopes Noise to "control leg only, does not affect the REACH-1a cert residual"; BlindedTag honest
limits intact; WRAP "four of six" + canonical numbers; 11 kinds + six capabilities consistent.
**CAVEAT (honesty):** 4/5 lenses were driver-inline, not independent agents — less adversarially
independent than the bar intends. **Round 5 (pass 2)** MUST attempt genuine independent agents on a
DIFFERENT lens-set (infra may recover); if agents keep stalling, report to the user that the spec is clean
per driver verification but independent-agent confirmation is infra-blocked. Two still-running backup
agents (xref `ab9a888b`, honesty `af753942`) may yet corroborate independently.

**DECISION (founder-overridable, gap-fill) — BlindedTag KDF pinned:** `BT = HKDF-SHA256(IKM=shared_secret,
salt="DMTAP-v0/blinded-tag", info=uint64_be(epoch_day), L=16)` (RFC 5869; the same HKDF-SHA256 as HPKE/RFC
9180 + push-wake/RFC 8291; suite-migration-independent). `BlindedTag.bytes` = exactly 16 B. Chosen as the
least-surprising, interop-safe pin consistent with the stack's existing KDF choice; was unspecified →
non-interoperable.

Round-3 verification calibration: the panel correctly adjudicated media-relay/reachability as distinct and
compute as a disclosed slot (round 2), and this round found real crypto/interop/state-machine defects — the
adversarial-verify layer is holding. After X/Y/Z/W land → **critique round 4** (re-covers REACH + these
fixes; different lens mix) toward the two-consecutive-clean gate.

## PARALLEL TRACK — REACH profile (founder pivot 2026-07-23: "both in parallel" + "spec-first")
The founder redirected to wakala's #1 gap (content-blind ingress) while this spec loop keeps converging in
the background. Decision: **spec-first** — finish the REACH profile in kotva (zone/descriptor + Noise
control-channel handshake + REACH-* error codes) to freeze it, THEN implement the wakala
`reachability-adapter` against it. **Disjointness rule for the background loop:** the REACH thread OWNS
`profiles/reachability.md` (+ any new REACH wire additions to 18-wire-format/21-errors it introduces) —
spec-perfection fix-agents MUST NOT touch those while REACH is being developed; critique may still READ
them. Recon agent mapping REACH draft + adapter status in flight; profile draft follows.

**REACH spec-first: DONE `e9a8adf` (deep research `wf_4ff01382-713` → decisive Option A).** REACH-2 rewritten:
box↔adapter tunnel = **libp2p-Noise `XX`**, each peer's libp2p identity key = its kotva Ed25519 IK;
signed-static-key payload = SIGMA sign-and-bind, channel-bound by construction (retires the §13 DMTAP-Auth
mis-citation that had produced an Asokan-vulnerable `nonce‖name`-with-no-binding impl). Added REACH-2b
control frames (ReachRegister/ReachRegisterAck, session-local, unsigned); reconciled the adapter descriptor
to §18.8a `CoordinatorDescriptor{kind=reachability-adapter}`; **no new §18 wire object, no new §21 code**;
§7 states Noise secures the control leg ONLY (does not close the REACH-1a cert residual) + IK dual-signing-
domain disclosure. Verified: SIGMA framing correct, no overclaim, lint clean. **Impl (wakala) follows the
freeze** — logged to wakala/COORDINATION.md [2026-07-23 reach] with the 1-day libp2p-noise-0.56 API spike +
atomicity MUST. Round 4 will re-cover REACH under the convergence gate.

**W3 (simplify):** compress per-doc SEC-invariant/honest-residual boilerplate to reference tables
(THREAT-MODEL = single argued source, ~15-20% cut, zero normative loss); turn reproduced index tables into
pointers; publish a minimal "Identity + MOTE + one transport" first-implementation conformance tier.

**W4 (cross-doc consistency):** unified terminology, all cross-refs resolve, family "connection" (SPEC
index + inter-profile links), including the previously-skipped profiles/tract/* + profiles/wrap/*.

**W5 (LAST — editorial):** South African/British English (prose only — NOT the `Authorization` header,
`GatewayAuthz`, wire fields, code, RFC proper nouns) + one RFC-grade skeleton per doc (abstract, BCP-14
conventions, stable numbering, normative/informative references, honest-residual section), modelled on
RFC 9051 / RFC 9420.

**W6 (converge — repeated multi-perspective critique, the hard gate):** critique is NOT a one-shot.
- **After EACH substantive wave commits (W1, W2, W3, W4)**, run a fresh **multi-lens read-only critique**
  over the whole spec (not just the changed files — catch regressions + cross-file contradictions the wave
  introduced). Feed every confirmed residual back as the next fix-wave. Verify each finding adversarially
  (a defect must reproduce; a claimed contradiction must be real) so no false-positive churns the spec.
- **Rotate the perspectives each round** so different angles hit: cryptographer · distributed-systems ·
  IETF-protocol/wire · simplicity/over-engineering · red-team/threat-model · adoption/pragmatist ·
  editorial/RFC-style/spelling · cross-reference-integrity. Every round uses ≥4 lenses; vary the mix
  round to round so the same blind spots aren't re-used.
- **Convergence bar (strict):** declare the spec PERFECTED only after **TWO CONSECUTIVE fully-clean
  multi-lens critique passes run from DIFFERENT lens-sets** — both reporting zero substantive
  contradictions, consistent South-African/British English, RFC-grade layout, and all cross-refs
  resolving. One clean pass is not enough. Then delete this file, stop the cron, report DONE with the
  commit range.
- If a round keeps surfacing the same residual, escalate it (it may be a genuine founder-call → COORDINATION).
  If **three successive convergence rounds** still find substantive residuals, STOP and report to the user.
