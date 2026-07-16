# 10. Versioning, Conformance, Governance

## 10.1 Protocol versioning

- Every wire object is **version-discriminable**, but not all via an in-band field. `Envelope`
  carries an explicit format-version field (`v`, §18.3.1). The other top-level signed objects
  (`Identity`, `DeviceCert`, `RecoveryPolicy`, `KeyRotation`, `MoveRecord`, `LocationRecord`, and
  the group/auth objects) derive their format version from their **domain-separation tag**
  (`"DMTAP-v0/…"`, §18.9) together with the `v=dmtap<N>` anchor published in DNS (§3.2): the
  DS-tag is part of every signing preimage, so a verifier reconstructs the preimage under a
  definite structural version and a mismatch simply fails signature verification (fail closed) —
  it never guesses. A future structural revision increments **both** the DS-tag (`"DMTAP-v1/…"`)
  and the DNS `v=` anchor in lockstep, giving an unambiguous discriminator without bloating every
  object with a redundant `v` byte.
- Every wire object also carries an algorithm `suite` (or inherits the enclosing object's, per
  §18.1.4).
- Unknown `v`/`suite` MUST be rejected (fail closed), never guessed.
- New message kinds use the reserved `0x40–0x7f` range (§2.3); a node ignores unknown kinds it
  is not required to process, but MUST NOT ack a kind it cannot validate.

## 10.2 Capability negotiation (dual-stack)

- A sender discovers a recipient's capabilities via the recipient's `Identity`/DNS record
  (native DMTAP vs legacy-only) and picks the path per-recipient (§3, §7.6).
- `system` MOTEs (kind `0x0a`) carry capability announcements between nodes (supported suites,
  privacy tiers, MLS ciphersuites, the optional `deniable-1:1` mode (§5.2.1), extensions, KT
  log-types, transport substrates).
- **Anti-rollback: capability announcements are monotonic (normative).** A capability
  announcement is authenticated (it rides inside a `system` MOTE authenticated to the recipient,
  §2.7) but, without a version, a replayed *older* announcement could **suppress a capability the
  peer has since advertised** (e.g. hide that a peer now supports a stronger suite or the KT-v1
  log-type, forcing a downgrade). Every capability announcement MUST therefore carry a
  **monotonic `caps_version` (`u64`)**, and a receiver MUST **reject any announcement whose
  `caps_version` is older-than-or-equal-to** the last one it has accepted from that peer
  (`ERR_CAPABILITY_ANNOUNCE_ROLLBACK`, `0x030A`, §21.5) — the identical rollback-defense rule
  the spec applies to `Identity.version` (§1.3), `LocationRecord.seq` (§4.2, §16.2), and
  `GroupState.version` (§5.8.2). A receiver retains the highest `caps_version` seen per peer;
  capabilities are only ever added or upgraded across increasing versions, never silently rolled
  back by a stale replay. Absence of a recognized capability token in the *current* (highest)
  announcement is still inconclusive, not a negative assertion (§21.22).
- **Structural extension of signed objects rides capability negotiation.** Because a decoder of
  a **signed** object fails closed on any unknown integer key (§18.1.2, so the signing preimage
  stays unambiguous), a sender MUST include a reserved `≥ 64` extension field in a signed object
  **only toward a peer that has advertised support** for that extension in its capability
  announcement — old peers therefore never receive a key they would reject. This is the
  normative path for extending signed objects; the "ignore-unknown" affordance is confined to
  **unsigned** objects and to `Headers.ext` (§18.1.2, §21.20).
- **Opportunistic upgrade**: a legacy correspondent is reached via the gateway; if they later
  publish a DMTAP key, native delivery is used automatically. No flag day.

## 10.3 Conformance levels

An implementation is **DMTAP-conformant** at a level if it passes the corresponding suite:

| Level | Requires |
|-------|----------|
| **Core** | Identity (§1), MOTE (§2), naming resolution + TOFU + **v0-minimal KT publish/verify with fail-closed-on-unreachable** (§3), mesh delivery + `deliver`/`ack` (§4), MLS 1:1 (§5), recipient policy incl. cold-sender challenge gating (§9) |
| **Private** | Core + mixnet (Sphinx packet + directory + 3-hop stratified paths + key-epoch rotation, §4.4.1–§4.4.4) + sealed sender + cover traffic + the **anti-active-adversary mechanisms** (per-epoch replay caches, Poisson mixing, loop-cover attack detection, entry guards + operator diversity, and **fail-closed no-downgrade**, §4.4.6–§4.4.9) + privacy tiers (§4, §6). The user-selectable **high-security profile** (§4.4.10) and **PQ-Sphinx** (§4.4.12) are OPTIONAL. |
| **Groups & Files** | Core + MLS groups + content-addressed file transfer (§5) |
| **Legacy** | Core + gateway inbound/outbound + DKIM delegation (§7) |
| **Clients** | Core + JMAP; IMAP/POP/SMTP-submission compat RECOMMENDED (§8) |
| **Auth** | Core + DMTAP-Auth login ceremony with origin binding + key-bound sessions (§13); OIDC bridge RECOMMENDED (§13.6) |

**Core is an interoperability floor, not a production target.** Because the standing default
privacy tier for mail (and for every control MOTE) is `private` (§4.6), a Core-only node cannot
operate at the protocol's own default without also implementing **Private**. Therefore a
**production** node that carries user mail **MUST** implement **Private** as well as Core;
`fast`-tier-only operation is a deliberate, user-surfaced downgrade (§4.6, §6), never the silent
default. Core exists so a minimal or special-purpose implementation (e.g. a gateway, a test
harness) can interoperate on the message spine, not so a shipping mail client can skip metadata
privacy. Privacy is a first-class requirement of the protocol, not an optional add-on level — the
Private level is the *normal* baseline, and the mixnet it depends on is fully specified in §4.4 /
§6.3 (Sphinx/Loopix, with the parameters of §16.3) rather than left to implementations.

The **conformance test suite** is the *operational definition* of compatibility. "DMTAP-compatible"
means "passes the suite," not "resembles the reference." This is the primary defense against
fragmentation. The suite lives in `conformance/` as three coupled artifacts:

- **`conformance/SUITE.md`** — the normative test-case catalog: 104 numbered cases
  (`DMTAP-<category>-<NN>`) grouped by the levels above, each pinning its spec clause, input,
  expected result (accept / reject + the §21 error code), and MUST/SHOULD.
- **`conformance/suite.json`** — the machine-readable mirror of those cases, so a runner in **any
  language** can drive them. It mirrors all 104 (SUITE.md and suite.json are in sync — the wave-2
  deniable-1:1 and KT-v1-hardening families, the `PROFILE` display-data cases, and the optional
  `PUSH` wake-signaling cases are mirrored).
- **`conformance/vectors/vectors.json`** — 32 byte-exact known-answer vectors (derived from the
  §18 canonical CBOR) that the cases dispatch on.

39 cases are byte-runnable today (33 vector-backed + 6 self-contained canonical-CBOR reject cases);
the remaining 65 carry an exact construction recipe and expected §21 error for the branches whose
subsystems are not yet vectored (mixnet/MLS/gateway/auth, plus the wave-2 deniable/KT-v1/org/
device-attestation families, the `PROFILE` display-data guards, and the optional `PUSH`
wake-signaling guards — see `conformance/README.md`). An
implementation conforms at a level iff it passes every `MUST` case of that level and of every level
it composes. The reference `dmtap-core` self-check test drives the vectors, but the spec plus these
three files are authoritative (§10.4), not the reference.

## 10.4 The spec is authoritative

Independent implementations MUST be buildable from this specification alone. The Rust reference
in `node/` and `gateway/` is a proof and a set of libraries, **not** normative. Where reference
and spec disagree, the spec governs (or the discrepancy is filed as a bug).

## 10.5 Governance & licensing (intent)

- **Standards track:** pursue an **IETF Internet-Draft** for the wire protocol and object
  formats, aiming for RFC status (as JMAP and MLS did). Neutral governance is what lets
  competitors adopt without fearing capture.
- **Licensing:** the **specification** and the **reference implementation** (node, gateway,
  client, libraries) ship under the **MIT license** (Apache-2.0 dual-licensing under
  consideration for its explicit patent grant). Everything a user touches and everything trust
  depends on is open; permissive licensing maximizes adoption by any party, competitors
  included.
- **Open software + paid operations:** commercial sustainability comes from a thin, **private
  control-plane** (a hosted operator) that implements the **operator seam** (`crates/dmtap-seam`)
  to bill *operations* — never gating any protocol or privacy feature. See §12 for the full
  model and the inviolable rule (privacy/crypto are never behind the seam).
- **Reuse over reinvention:** DMTAP deliberately composes existing standards — MLS (RFC 9420),
  HPKE (RFC 9180), JMAP (RFC 8620/8621), libp2p, DNS/DNSSEC/SVCB, key transparency, Privacy
  Pass. The novelty is the *composition and transport*, not new cryptography.

## 10.6 Roadmap markers (non-normative)

- **v0:** Core + Private (minimal KT via TOFU+pinning) + Groups & Files + Legacy gateway.
- **v1 hardening:** **KT-v1 is specified and capability-negotiated** — the full CONIKS/
  key-transparency profile (federated multi-log with `> n/2` quorum-audited bindings, STH
  gossip, monitor/auditor roles, and equivocation halt-alert) is normatively defined as log-type
  `0x02` (§3.5.2, §21.19) and selected per §10.2, with v0-minimal (§3.5.1) remaining the
  interoperable Core default. Also: onion-routed bulk; anonymous tokens at scale; PQ suite `0x02`
  migration; optional self-sovereign naming backend.
- **Later research:** stronger private contact discovery; scalable private retrieval for
  hostile-buffer scenarios; deniable-group properties; metadata-privacy for very large files.

## 10.7 Downgrade & fail-closed invariants (the auditable set)

DMTAP's downgrade-resistance and fail-closed rules are deliberately scattered across the layers
that own them — the suite ratchet lives with identity (§1.3), the tier/profile floor with the
mixnet (§4.4.9), the ack-before-`250` rule with the gateway (§7.4). **Scattering hides gaps:** a
reviewer checking one section cannot see whether the *posture as a whole* is fail-closed, and it was
exactly two such scattered gaps — the recipient-side **suite high-water-mark** (§1.3) and the
**in-force-profile floor** for High-security paths (§4.4.9) — that a hardening review surfaced,
because each looked complete in isolation while the set had a hole. This subsection therefore
collects **every** downgrade and fail-closed invariant into one table so the property "DMTAP never
*silently* accepts a weaker security posture" is checkable as a **set**, not rule-by-rule.

**Reading the table.** Each row is an invariant, the clause that defines it (authoritative — this
table indexes, it does not restate), the trigger that fires it, and the required behavior/error on
violation. Codes are the §21 registry values. Where a row has no code, the failure is a
signature/decoder rejection with no dedicated status. Every "behavior" is a **MUST** unless the
owning clause says otherwise.

### 10.7.1 Version, suite & capability downgrades

| Invariant | Clause | Trigger | Behavior / error on violation |
|-----------|--------|---------|-------------------------------|
| Unknown `v`/`suite` → reject | §10.1, §2.7 step 1, §1.1 | any object decoded under an unknown format version or algorithm suite | fail closed — reject, **never guess**; a signed object's DS-tag/`suite` mismatch simply fails verification |
| Highest-mutual-suite send | §1.3 | empty suite intersection with recipient | delivery fails closed — no silent weak-suite fallback |
| **Suite high-water-mark ratchet** | §1.3, §2.7 step 8 | inbound `Envelope.suite` **below** the pinned contact's high-water-mark | reject to requests + security warning, `0x020F`; the mark ratchets **up** only, lowered solely by an `IK`-authorized retirement (§1.5) |
| **Hybrid suite: no intra-suite strip** | §1.3, §16.7 | a hybrid-suite object (`0x02`) whose PQ signature component is missing/fails while only the classical component validates, presented to a verifier that **supports** the hybrid suite | reject as incomplete/downgraded hybrid, `0x0210`; a hybrid verifier MUST require **every** component signature (AND-composition) and MUST use the X-Wing IND-CCA KEM combiner — single-component acceptance is for a genuinely legacy verifier only, at that component's lower assurance |
| Capability-announce anti-rollback | §10.2 | `caps_version` older-than-or-equal-to the last accepted from that peer | reject the announcement, retain the higher set, `0x030A` |
| Signed-object extension gating | §10.2, §18.1.2 | an unknown integer key in a **signed** object | decoder fails closed; a reserved `≥ 64` field is sent **only** toward a peer that advertised support |

### 10.7.2 Metadata-privacy (tier/profile/mixnet) downgrades

| Invariant | Clause | Trigger | Behavior / error on violation |
|-----------|--------|---------|-------------------------------|
| **No `private → fast` / in-force-profile floor** | §4.4.9 | no path meeting the **in-force profile's** bar (Standard ≥ 3 hops/≥ 3 operators; High-security ≥ 5/≥ 5) is buildable | hold in the retry queue; **never** silently route over `fast`, fewer hops, fewer operators, or a lower profile's bar; `0x0310`; surface to user past the retry deadline |
| Active-attack fail-closed | §4.4.7 | loop-cover return fraction below the loss threshold (§16.3) | rotate away + `HALT_ALERT` + **fail closed** for the `private` tier — never silently continue on a weaker path, `0x030F` |
| Per-epoch mix replay drop | §4.4.6 | a Sphinx per-hop tag already in the epoch replay cache | `DROP_SILENT`, `0x030E` (cache spans every still-usable key, no hard epoch-boundary flush) |
| Mix operator-diversity **MUST be attested** | §4.4.8 | a mix whose `operator` is absent/un-attested is counted as fresh diversity | it MUST NOT count as its own operator — excluded or counted shared; keeps the ≈ *a*² compromised-path bound (else one Sybil operator collapses it to ≈ *a*) |
| Mix directory authority-signed + rollback | §4.4.2 | directory not signed by the pinned authority, or an older-or-equal `version` | reject, `0x030B` (a directory split-view is a KT equivocation, `0x0107`) |
| **Mix directory freshness (freeze defense)** | §4.4.2 | a served `MixDirectory` older than the freshness window (§16.3, ≤ one mix-key epoch) — an adversary freezing the client on a stale, adversary-favourable fleet view | treat as stale; MUST refresh before building any `private` path and **fail closed** if none is obtainable, `0x0311`; a withheld fresh directory is KT-detectable (no new root within the window), completing the directory authority's detectable-if-misbehaves property |
| Cover traffic is not optional | §4.4.5, §6.2 | (posture) a `private`-tier node omitting loop/drop/recipient cover | non-conformant — cover is load-bearing, MUST be emitted |

### 10.7.3 Trust-binding (KT / identity / group) fail-closed

| Invariant | Clause | Trigger | Behavior / error on violation |
|-----------|--------|---------|-------------------------------|
| KT fail-closed on unreachable | §3.3, §3.5.1 | KT log unreachable/partitioned/censored at first-contact pinning | MUST NOT silently TOFU-pin — block or hard-warn + explicit acceptance, `0x0106` |
| KT equivocation → HALT | §3.5.2(d) | split view / append-only violation / stale-frozen head / sub-quorum | `HALT_ALERT`, publish evidence, evict the log, fail closed below quorum — `0x0107` / `0x0110` / `0x0111` / `0x0112` |
| Committer / co-committer fork → HALT | §5.1, §5.1.1 | two Commits at one log position with the same predecessor | `HALT_ALERT`; out-of-band fork recovery on the `> n/2` (or 2-party) quorum, `0x0404` |
| `from`-pin mismatch | §2.7 step 8, §3.4 | decrypted `Payload.from` ≠ the pinned identity for a known contact | `HALT_ALERT`, **never silently re-pin**, `0x0209` |
| Keyset-change re-verification | §3.4.1 | a verified pin's `iks`/`Identity` content-address changes | downgrade the pin to **unverified**, prompt OOB re-verify — never carry verified status across the change |
| Device attestation freshness (gated only) | §1.2a | attestation absent/invalid/expired/retired-root in an **attestation-gated** context | `FAIL_CLOSED_BLOCK` for that context, `0x0116` / `0x0118`; advisory — **never** overrides §1.4 authorization |
| **Org-managed-undisclosed** fail-closed | §3.10.2, §18.4.7 | an escrowed-key account presented **without** its `custody = "org-managed"` marker | `HALT_ALERT`; MUST NOT present as a sovereign identity, `0x0115` |
| **Recovery-weakening quorum + veto** | §1.4 rules 3–4 | a `RecoveryPolicy` change that drops/weakens a factor | requires `rotate_threshold` **even when signed by `IK`**; takes effect only after the **72 h** asymmetric veto window (§16); a non-conforming weakening is vetoable |
| **Auth: bare-node-signed login FORBIDDEN** | §13.3.1 | a login assertion presented as node-signed / from an unattributable channel | reject outright, `0x0507`; a node MUST NOT sign a login on a user's behalf (consent-farming defense) |
| **Auth: login scope is signed** | §13.3, §18.7.2, §18.9.8 | RP would grant a scope broader than the assertion's signed `scope` | fail closed — the broader-scope preimage fails signature verification; RP MUST NOT grant beyond the signed scope (`0x0508` if surfaced as over-attenuated) |
| **Auth: delegation re-chains through recovery** | §13.4, §1.4 | a live RP session whose authorizing delegation predates an `Identity.version` bump (IK rotation / recovery) | terminate at next re-validation; the revocation-list-epoch option MUST be keyed to `Identity.version`, else the recovery-invalidation guarantee silently fails to reach that RP |
| **Auth: unreachable status/KT ⇒ bounded grace, then fail closed** | §13.4 | RP's status/KT head unreachable at re-validation | honor the last-validated delegation only to a **2× grace window** (§16), then fail closed — never honor indefinitely (bounds post-revocation persistence) |
| **Auth: high-value RP multi-log / OOB** | §13.7 item 6 | high-value login against v0 single-KT-log | MUST require multi-log consistency or an OOB-verified pin — a single log can equivocate, `0x0111`/`0x0107` |
| **Auth bridge: per-RP audience** | §13.6 | bridge embeds a login assertion audienced to the bridge, reused across its RPs | the bridge MUST run a per-RP §13.3 ceremony (`aud` = target RP, fresh per-RP `cnf`); an RP verifying the key directly MUST check `assertion.aud == own identifier`, `0x0501` on mismatch |

### 10.7.4 Delivery, gateway & anti-abuse fail-closed

| Invariant | Clause | Trigger | Behavior / error on violation |
|-----------|--------|---------|-------------------------------|
| **Deferred cold MOTE = no-ack** | §2.7a, §19.3.1 step 9 | cold sender with absent/below-threshold challenge | hold in the rate-limited **requests** area, never the inbox, never silent-drop, and **not acked** (an ack would confirm existence + falsely signal delivered) |
| Invalid/forged proof or bad `sender_sig`/`id` | §2.7, §2.7a | forged challenge, failed ephemeral signature, or `id` mismatch | discard **silently**, do not ack (except a duplicate `id`, which is acked) |
| Payload-signature fail-closed | §2.7 step 8 | `Payload.sig` fails under `from` | discard silently, do **not** ack (fail closed, matching steps 1–3) |
| **Gateway ack-before-`250`** | §7.4, §19.7.1 | inbound SMTP; recipient node has not durably `ack`ed within the transaction window | return SMTP **`451`** (defer to the legacy sender's queue); MUST NOT return `250` on mere hand-off — no post-`250` silent-loss window |
| Gateway attestation mandatory + key-bound | §7.2a, §19.3.1 step 8a | a legacy-origin MOTE is unattested, or its key is not under the recipient's own `_dmtap-gw` | `DROP_SILENT`, `0x0601` / `0x0602`; every *accepted* message is thus either attested (gateway-touched) or attestation-free (**provably** pure-mesh) — no unmarked third state |
| GatewayAuthz fail-**safe**, not fail-open | §12.2 | hosted operator unreachable | permit legacy egress only to **established contacts** + senders with operator-independent **PoW**; **deny** cold/unproven egress; postage excluded (needs online redemption) |
| Postage issuer-unreachable = unverified | §9.5.1 | stamp redemption endpoint unreachable | treat the stamp as *unverified*, fall back to token/PoW policy — **never** accept real-money bearer value on faith |
| Unvetted token = zero budget | §9.3.1 | token from an unknown/self issuer | counts as **no token** — forces fallback to PoW/postage; self-issuance buys anonymity, not cost relief |
| **Deniable-mode no-silent-downgrade** | §5.2.1(d) | recipient has not advertised the `deniable-1:1` capability | `REJECT_NOTIFY` — the client MUST surface the choice (non-deniable send, or not at all); **never** silently downgrade the user's *expectation* of deniability, `0x040E` |
| Deniable payload signature forbidden | §5.2.1(c) | a `DeniablePayload` carries a signature field | `FAIL_CLOSED_BLOCK`, `0x040F` — the missing signature is the property |
| **Deniable content off the signed cluster tree** | §5.2.1(d), §5.6 | a personal-cluster MLS CRDT entry embeds a `DeniablePayload` / deniable plaintext | `FAIL_CLOSED_BLOCK`, `0x040F` — no member-signed record may cover deniable plaintext (else owner-side non-repudiable evidence is manufactured) |
| **Deniable bundle first-contact rollback** | §5.2.1(f), §5.8.6 | a fetched `DeniablePrekeyBundle` `version` is older than the KT-anchored current version | fail closed, `0x040B` — withdrawal must be detectable at first contact, not only on re-fetch by a pinned peer |
| **Push-wake gates fail closed** | §4.9.1, §4.9.4 | unverifiable subscription / content-bearing wake / unauthenticated wake / replayed wake / over-budget wake (a wake spends the target's battery) | reject/drop — `0x0312` (FAIL_CLOSED_BLOCK), `0x0313` (FAIL_CLOSED_BLOCK), `0x0314` (DROP_SILENT), `0x0316` (DROP_SILENT), `0x0315` (rate-limit at emitter **and** receiver); a wake is never surfaced on faith |

### 10.7.5 The one governing rule

Across all four groups the invariant is identical: **a security-relevant downgrade is either
*refused* (fail closed) or an *explicit, user-surfaced choice* — never an automatic, silent
reaction to adversary pressure or component unavailability.** The seam's `GatewayAuthz` is the sole
place this interacts with the operator model, and even there it fails **safe**, not open (§12.2,
distinct from the fail-*open*-to-function stance of metering/quota). A conformant implementation
satisfies §10.7 iff it enforces **every** MUST row above; the conformance suite (§10.3) carries a
case per row, and a new downgrade-resistance or fail-closed rule added anywhere in the spec MUST be
mirrored here so the set stays complete.
