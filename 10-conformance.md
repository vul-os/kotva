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
  log-types, transport substrates, `pub-1` — public-object serving opt-in, §22).
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
| **Private** | Core + mixnet (Sphinx packet + derived fleet view + 3-hop stratified paths + key-epoch rotation, §4.4.1–§4.4.4) + sealed sender + cover traffic + the **anti-active-adversary mechanisms** (per-epoch replay caches, Poisson mixing, loop-cover attack detection, entry guards + operator diversity, and **fail-closed no-downgrade**, §4.4.6–§4.4.9) + privacy tiers (§4, §6) + the **Bootstrap-profile rules** (§4.4.10a: user-visible degradation, no anonymity claim, auto-upgrade-never-fall-back, per-contact ratchet) — a network small enough to need Bootstrap is the normal early case, so its constraints are part of the level, not an extra. The user-selectable **high-security profile** (§4.4.10) and **PQ-Sphinx** (§4.4.12) are OPTIONAL. |
| **Groups & Files** | Core + MLS groups + content-addressed file transfer (§5) |
| **Legacy** | Core + gateway inbound/outbound + DKIM delegation (§7); gateway legacy-client surfaces (IMAP/POP/SMTP-submission + CalDAV/CardDAV) + the reachability ingress + operator modes RECOMMENDED (§7.15) |
| **Clients** | Core + **JMAP** — the node's native client surface (§8.1). Legacy client protocols (IMAP/POP/SMTP-submission, CalDAV/CardDAV) are a **gateway** capability (RECOMMENDED, §7.15), **not** a node one; a conformant node runs no legacy protocol server (§8) |
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

- **`conformance/SUITE.md`** — the normative test-case catalog: 348 numbered cases
  (`DMTAP-<category>-<NN>`) grouped by the levels above, each pinning its spec clause, input,
  expected result (accept / reject + the §21 error code), and MUST/SHOULD.
- **`conformance/suite.json`** — the machine-readable mirror of those cases, so a runner in **any
  language** can drive them. It mirrors all 348 (SUITE.md and suite.json are in sync — the wave-2
  deniable-1:1 and KT-v1-hardening families, the `PROFILE` display-data cases, the optional
  `PUSH` wake-signaling cases, the `FILE` durability cases, the wave-3 device-cluster `SYNC`,
  `ALIAS`, and gateway-alias `GWALIAS` families, the pluggable-resolver `RESOLVE` family, the
  `PUB` public-object family, the wave-6 anti-drift families — Bootstrap profile
  (`MIXPROF`, §4.4.10a), derived fleet view (`FLEET`, §4.4.2), guards and path diversity
  (`GUARD`, §4.4.8), location/resolution order (`LOC`, §4.2), the zero-relationship delivery
  floor (`FLOOR`, §9.7a/§9.4.1), failure classes (`FAILCLASS`, §10.7.0) and gateway role
  boundaries (`GWROLE`, §7.1b/§7.11.4), the gateway families (`GWOPS`/`GWSMTP`/`GWATT`/`GWNAME`/
  `GWFLOOR`/`GWLEG`, §7), the auth families (`AUTHORIG`/`AUTHSESS`/`AUTHBRIDGE`, §13), the
  transport, privacy, client, naming, group, anti-abuse, seam, scale, state-machine,
  forward-compatibility and wire-object families, and the DMTAP-PUBSUB family (`PUBSUB`, §25) —
  are mirrored).
- **`conformance/scope.json`** — the **curated MUST denominator** the coverage metric measures
  against: every one of the specification's MUST-bearing sections classified `IMPL` / `ENCODING` /
  `RESTATEMENT` / `PROCEDURE` / `NARRATIVE` with a one-line reason, because not every capitalised
  MUST is an implementation requirement a runner could check (the §21 registry's Action column and
  the §19/§20 appendices restate rules this document's narrative owns, per §10.4; IANA allocation
  policies bind a future registrant). Inclusion is the default: a section is `IMPL` unless the
  reason names what owns the requirement instead.
- **`conformance/vectors/`** — the byte-exact known-answer vectors the cases dispatch on:
  `vectors.json` holds 69 core vectors (derived from the §18 canonical CBOR; 43 of them are
  driven by cases today, the other 26 are pre-generated for construction-todo families not yet
  wired to a case) and `pub_vectors.json` holds 15 vectors for the §22 DMTAP-PUB profile (all 15
  driven by `PUB` cases). DMTAP-PUBSUB (§25) generates no new vectors of its own (§10.3 below).

62 cases are byte-runnable today (56 vector-backed against `vectors.json`/`pub_vectors.json` +
6 self-contained canonical-CBOR reject cases); 17 further cases are verified by implementer or
deployment attestation, having **no wire bytes to recompute at all** — an in-product disclosure
(§22.7 publish consent, §4.4.10a's Bootstrap degradation notice, §25.9's bounded-lifetime /
cooperative-revoke disclosure), a client's own claims about a session or an address (§7.10.6,
§7.15.3), a process boundary (§7.1b), or the population a deployment actually serves (§7.15.4);
they are the rows marked `manual-attestation` in `conformance/SUITE.md`, each naming the review
that settles it. The remaining 263 carry an exact construction recipe and expected §21 error for the
branches whose subsystems are not yet vectored (mixnet/MLS/gateway/auth, plus the wave-2
deniable/KT-v1/org/device-attestation families, the `FILE` durability guards, the `PROFILE`
display-data guards, the optional `PUSH` wake-signaling guards, the wave-6 anti-drift families
(`MIXPROF`/`FLEET`/`GUARD`/`LOC`/`FLOOR`/`FAILCLASS`/`GWROLE`), the gateway families
(`GWOPS`/`GWSMTP`/`GWATT`/`GWNAME`/`GWFLOOR`/`GWLEG`), the profile-level `CAD`/`VIDEO` checklists,
and the DMTAP-PUBSUB guards (`PUBSUB`, §25) — see `conformance/README.md`). The partition is exact:
62 + 19 + 267 = 348. An implementation conforms at a level iff it passes every `MUST` case of that
level and of every level it composes.

**On the coverage figure.** `make coverage` reports that **100%** of `IMPL` MUSTs sit in a section
some case cites. That is a **floor and a section-level measure**: a section counts as covered if
*any* case cites it, not if every MUST in it is exercised; it counts cases that **exist**, not
cases that **pass** (62 of 348 are byte-runnable today, and no implementation has yet been run
against the suite); and it is measured against a **curated** denominator whose classification is a
judgement, auditable in `conformance/scope.json` and re-checkable by `make lint` (check C10, which
fails the build if a MUST-bearing section is left unclassified). The uncurated figure over every
capitalised MUST is **84%**. Read the number as "nothing implementable is entirely
unattended", never as a pass mark. The reference `dmtap-core` self-check test drives the vectors,
but the spec plus these three artifacts are authoritative (§10.4), not the reference.

## 10.4 The spec is authoritative

Independent implementations MUST be buildable from this specification alone. The Rust reference
in `node/` — the one binary, whose gateway mode is a role rather than a separate program (§0.2) — is a proof and a set of libraries, **not** normative. Where reference
and spec disagree, the spec governs (or the discrepancy is filed as a bug).

## 10.5 Governance & licensing (intent)

- **Standards track:** pursue an **IETF Internet-Draft** for the wire protocol and object
  formats, aiming for RFC status (as JMAP and MLS did). Neutral governance is what lets
  competitors adopt without fearing capture.
- **Licensing:** the **specification** (this repository) is licensed **CC BY 4.0** — anyone may
  implement, quote, translate, and build on it with attribution, which is the licence a *standard*
  wants. The **reference implementation** (a separate repository) is dual-licensed **MIT OR
  Apache-2.0** — the permissive pair, with Apache-2.0's explicit patent grant — so any party,
  competitors included, may embed it. Both are © VulOS. Everything a user touches and everything
  trust depends on is open; a closed client could not credibly claim "we cannot read your mail."
- **There is no control plane, and no commercial layer to describe.** Nothing is sold, no hosted
  component is required by any implementation, and there is no registry, licence server, or
  telemetry endpoint. Third parties run the gateway role because they want the network to exist
  (§0.2.3, §12.4). What sustains the protocol is that every role but one costs its taker almost
  nothing and benefits them directly (§0.2.2) — and the exception, legacy SMTP egress, is
  transitional (§7.1c). See §12 for the user-protection model and the inviolable rule (§12.3:
  privacy, crypto, metadata privacy, recovery, native delivery, and access to your own data are
  never behind a gate).
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

### 10.7.0 Failure classes — "fail closed" names three different behaviors (normative)

"Fail closed" has been used above as though it were one rule. It is three, and conflating them
produces a protocol that is secure and unusable. Every invariant in this section MUST be
classified as exactly one of:

| Class | Meaning | When it applies | User-visible effect |
|---|---|---|---|
| **FAIL-CLOSED-AUTH** | Refuse permanently. Do not proceed on any timescale. | The failure is about **authenticity or confidentiality**: a signature does not verify, a key does not match a pin, a log equivocated, a suite is below the ratchet, resolvers disagree. | An error and an alert. Retrying cannot help and MUST NOT be offered. |
| **FAIL-QUEUED** | Hold, retry, never lose, never weaken. | The failure is about **liveness**: some external service is unreachable *right now* — a directory, a log, an issuer, a peer. Nothing is wrong with the message; it simply cannot be sent yet under the guarantees in force. | Queued. Surfaced only after the retry deadline (§16). |
| **FAIL-DEGRADED** | Proceed with an explicitly reduced guarantee. | Only where the reduced guarantee is **still acceptable**, the reduction is **user-visible**, **time-capped**, and **logged**, and the user has consented to the weaker tier. | A visible indicator naming exactly what is reduced and for how long. |

**The governing distinction.** A failure of *authenticity* must never become a delay, and a
failure of *liveness* must never become a rejection. Classifying a liveness failure as
FAIL-CLOSED-AUTH is not conservative — it is a **denial-of-service surface handed to anyone who
can take a service offline**, and it converts an outage in one component into total loss of
function. This matters especially here: DMTAP depends on several external services (KT logs, the
mix directory, postage issuers, rendezvous nodes), and if each is independently permitted to
block all progress, composite availability is the product of all of them and will be worse than
the SMTP the protocol replaces.

**The invariant that follows from it (MUST).** *An offline-first store-and-forward protocol must
never be unable to queue.* Whatever is unreachable, a node MUST always be able to accept a
message from its user, hold it durably, and keep trying under the guarantees in force. No
liveness failure anywhere in this specification may prevent enqueue. Email's decisive operational
property is that it degrades to **late**, never to **blocked**; DMTAP MUST preserve that, and the
one thing it may never trade for privacy is the ability to hold a message until the privacy is
available.

### 10.7.1 Version, suite & capability downgrades

| Invariant | Clause | Trigger | Behavior / error on violation |
|-----------|--------|---------|-------------------------------|
| Unknown `v`/`suite` → reject | §10.1, §2.7 step 1, §1.1 | any object decoded under an unknown format version or algorithm suite | fail closed — reject, **never guess**; a signed object's DS-tag/`suite` mismatch simply fails verification |
| Highest-mutual-suite send | §1.3 | empty suite intersection with recipient | delivery fails closed — no silent weak-suite fallback |
| **Suite high-water-mark ratchet** | §1.3, §2.7 step 8 | inbound `Envelope.suite` **below** the pinned contact's high-water-mark | reject to requests + security warning, `0x020F`; the mark ratchets **up** only, lowered solely by an `IK`-authorized retirement (§1.5) |
| **Hybrid suite: no intra-suite strip** | §1.3, §16.7 | a hybrid-suite object (`0x02`) whose PQ signature component is missing/fails while only the classical component validates, presented to a verifier that **supports** the hybrid suite | reject as incomplete/downgraded hybrid, `0x0210`; a hybrid verifier MUST require **every** component signature (AND-composition) and MUST use the X-Wing KEM combiner — single-component acceptance is for a genuinely legacy verifier only, at that component's lower assurance |
| **Hybrid components are non-separable** | §1.3, §18.1.6 | a hybrid `sig-val` whose components were signed over the single-algorithm preimage rather than the suite-bound composite representative `M'`, or a component lifted out of a `0x02` object and presented as an `0x01` signature | verification fails: the two forms of the representative are distinct preimages, so a stripped or promoted component simply does not verify. An implementation that signs `M' = DS-tag ‖ 0x00 ‖ body` under a hybrid suite is non-conformant (§18.1.6) |
| Capability-announce anti-rollback | §10.2 | `caps_version` older-than-or-equal-to the last accepted from that peer | reject the announcement, retain the higher set, `0x030A` |
| Signed-object extension gating | §10.2, §18.1.2 | an unknown integer key in a **signed** object | decoder fails closed; a reserved `≥ 64` field is sent **only** toward a peer that advertised support |

### 10.7.2 Metadata-privacy (tier/profile/mixnet) downgrades

| Invariant | Clause | Trigger | Behavior / error on violation |
|-----------|--------|---------|-------------------------------|
| **No `private → fast` / in-force-profile floor** | §4.4.9 | no path meeting the **in-force profile's** bar (Bootstrap ≥ 3 hops/best-effort ≥ 2 ASNs; Standard ≥ 3 hops/≥ 3 operators; High-security ≥ 5/≥ 5) is buildable | hold in the retry queue; **never** silently route over `fast`, fewer hops, fewer operators, or a lower profile's bar; `0x0310`; surface to user past the retry deadline |
| **Bootstrap profile ratchets up only** | §4.4.10a | a contact whose relationship has already run at Standard (or above) is offered a **Bootstrap**-tier path | fail closed, `0x0310` — the profile ratchets **per contact** like the §1.3 suite high-water-mark; auto-upgrade is mandatory and fallback is forbidden, else DoSing mixes forces the whole network onto the degraded profile (the §4.4.9 attack under a friendlier name) |
| **Bootstrap profile is disclosed, never claimed as anonymity** | §4.4.10a | client operating on Bootstrap presents `private` as anonymous, or hides the degradation | non-conformant: the client MUST show that metadata privacy is degraded **and why** (current mixes/ASNs vs the Standard bar), and MUST NOT state an anonymity set |
| Active-attack fail-closed | §4.4.7 | loop-cover return fraction below the loss threshold (§16.3) | rotate away + `HALT_ALERT` + **fail closed** for the `private` tier — never silently continue on a weaker path, `0x030F` |
| Per-epoch mix replay drop | §4.4.6 | a Sphinx per-hop tag already in the epoch replay cache | `DROP_SILENT`, `0x030E` (cache spans every still-usable key, no hard epoch-boundary flush) |
| Mix operator-diversity **MUST be attested** | §4.4.8 | a mix whose `operator` is absent/un-attested is counted as fresh diversity | it MUST NOT count as its own operator — excluded or counted shared; keeps the ≈ *a*² compromised-path bound (else one Sybil operator collapses it to ≈ *a*) |
| Mix fleet view is **derived**, cache verified + rollback | §4.4.2 | a **cached** `MixDirectory` containing a descriptor the client cannot independently verify against its KT log quorum, or an older-or-equal `version` | reject, `0x030B` — a cache is a convenience, never an authority; a log split-view over the mix set is a KT equivocation, `0x0107` |
| **Mix fleet-view freshness (freeze defense)** | §4.4.2 | a derived view or cache older than the freshness window (§16.3, ≤ one mix-key epoch) — an adversary freezing the client on a stale, adversary-favourable fleet view | **FAIL-QUEUED** (§10.7.0): treat as stale; MUST refresh before building any `private` path; if none is obtainable the sender **queues and retries**, `0x0311` — it MUST NOT downgrade the tier and MUST NOT refuse to enqueue. A directory outage delays mail; it must never stop it. Withheld fresh descriptors are KT-detectable (no new current-epoch entry within the window), and with the authority removed no single party's silence achieves this network-wide |
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
| GatewayAuthz fail-**safe**, not fail-open | §12.2 | the party behind the seam is unreachable | permit legacy egress only to **established contacts** + senders with operator-independent **PoW**; **deny** cold/unproven egress; postage excluded (needs online redemption) |
| Postage issuer-unreachable = unverified | §9.5.1 | stamp redemption endpoint unreachable | treat the stamp as *unverified*, fall back to token/PoW policy — **never** accept real-money bearer value on faith |
| Unvetted token = zero budget | §9.3.1 | token from an unknown/self issuer | counts as **no token** — forces fallback to PoW/postage; self-issuance buys anonymity, not cost relief |
| **Deniable-mode no-silent-downgrade** | §5.2.1(d) | recipient has not advertised the `deniable-1:1` capability | `REJECT_NOTIFY` — the client MUST surface the choice (non-deniable send, or not at all); **never** silently downgrade the user's *expectation* of deniability, `0x040E` |
| Deniable payload signature forbidden | §5.2.1(c) | a `DeniablePayload` carries a signature field | `FAIL_CLOSED_BLOCK`, `0x040F` — the missing signature is the property |
| **Deniable content off the signed cluster tree** | §5.2.1(d), §5.6 | a personal-cluster MLS CRDT entry embeds a `DeniablePayload` / deniable plaintext | `FAIL_CLOSED_BLOCK`, `0x040F` — no member-signed record may cover deniable plaintext (else owner-side non-repudiable evidence is manufactured) |
| **Deniable bundle first-contact rollback** | §5.2.1(f), §5.8.6 | a fetched `DeniablePrekeyBundle` `version` is older than the KT-anchored current version | fail closed, `0x040B` — withdrawal must be detectable at first contact, not only on re-fetch by a pinned peer |
| **Push-wake gates fail closed** | §4.9.1, §4.9.4 | unverifiable subscription / content-bearing wake / unauthenticated wake / replayed wake / over-budget wake (a wake spends the target's battery) | reject/drop — `0x0312` (FAIL_CLOSED_BLOCK), `0x0313` (FAIL_CLOSED_BLOCK), `0x0314` (DROP_SILENT), `0x0316` (DROP_SILENT), `0x0315` (rate-limit at emitter **and** receiver); a wake is never surfaced on faith |

### 10.7.4a Substrate — Sync capability fail-closed (`0x0A`, proposed additive)

Mirrored here per `substrate/SYNC.md` §12's own rule ("registered additively under §21.14; mirrored into
the §10.7 auditable set") and the substrate adoption rule that "each substrate document carries its own
fail-closed table cross-referenced into §10.7" (`substrate/README.md` rule 5). The Sync substrate
capability ([`substrate/SYNC.md`](substrate/SYNC.md)) is the one genuinely new normative specification in
the substrate (not a profile of an existing numbered section); its fail-closed rules are additive and
apply only to a replica that advertises `sync-1` (§21.22, §21.24c) — the identical scoping as `pub-1`'s
optional-capability posture (§10.3).

| Invariant | Clause | Trigger | Behavior / error on violation |
|-----------|--------|---------|-------------------------------|
| Author admission + op signature mandatory | `SYNC.md` §4.1, §8, §9 | op author not admitted by the namespace policy, or its `COSE_Sign1` fails / `DeviceCert` chain broken | FAIL_CLOSED_BLOCK, `0x0A01` / `0x0A02` |
| Op value / causal-integrity validity | `SYNC.md` §4.1, §4.3, §8 | non-`ext-value` value, a `set-remove` citing a future add-tag, an embedded deniable payload, or any malformed op | FAIL_CLOSED_BLOCK, `0x0A03` — never merge on a guess |
| Unsupported op/snapshot version | `SYNC.md` §4.1, §6.1 | a `v`/`suite` this replica does not support | FAIL_CLOSED_BLOCK, `0x0A04` — never guess |
| HLC skew bound | `SYNC.md` §3 | op `wall` outside the fixed skew window | FAIL_CLOSED_BLOCK, `0x0A05` |
| PN-counter foreign-entry write | `SYNC.md` §4.6 | an op mutates another author's `P`/`N` entry | FAIL_CLOSED_BLOCK, `0x0A06` |
| RGA causal-readiness bound | `SYNC.md` §4.7 | an insert's origin is absent and the causal buffer overflows | DEFER_REQUESTS then ROTATE_RETRY, `0x0A07` — buffering, never rejection, unless bounded |
| Frame hash-chain integrity | `SYNC.md` §4.1 | a `SyncFrame` op's back-link does not resolve to its predecessor | HALT_ALERT, `0x0A08` — publish conflicting frames as evidence |
| Snapshot root mismatch | `SYNC.md` §6.1 | recomputed observable-state root ≠ `Snapshot.root` at the same `covers` | HALT_ALERT, `0x0A09` — divergence evidence |
| Cross-namespace reference | `SYNC.md` §7 | an op references a `target` outside its own namespace | FAIL_CLOSED_BLOCK, `0x0A0A` |
| Open-namespace admission quota | `SYNC.md` §9 | an admission rate/quota exceeded | DENY_POLICY, `0x0A0B` — a policy deny, never a silent hole |

### 10.7.5 The one governing rule

Across all four groups the invariant is identical: **a security-relevant downgrade is either
*refused* (fail closed) or an *explicit, user-surfaced choice* — never an automatic, silent
reaction to adversary pressure or component unavailability.** The seam's `GatewayAuthz` is the sole
place this interacts with the operator model, and even there it fails **safe**, not open (§12.2,
distinct from the fail-*open*-to-function stance of metering/quota). A conformant implementation
satisfies §10.7 iff it enforces **every** MUST row above. The conformance suite (§10.3) is the
enforcement vehicle: most rows carry a pinning case, and the rows currently **without** one —
the §7.4 ack-before-`250` rule, the key-bound half of §7.2a (`0x0602`), the §13.3.1
bare-node-signed-login rejection, the §13.6 per-RP-audience check, the §13.7 high-value
multi-log requirement, and the §9.5.1 postage issuer-unreachable fallback — are pending wave-2
vectors. A new downgrade-resistance or fail-closed rule added anywhere in the spec MUST be
mirrored here so the set stays complete.
