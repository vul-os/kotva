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
- **Structural-version migration (normative — additive, never a flag day).** Incrementing the DS-tag
  and the `v=dmtap<N>` anchor is the *discriminator*; the *migration* follows the same additive,
  capability-negotiated discipline as a new algorithm suite (§1.3, §21.15) or a new transport
  substrate (§21.25), never a coordinated flag day:
  1. **Dual-stack verification.** During a transition a conformant node MUST be able to **verify**
     preimages under both the outgoing and the incoming DS-tag version, while **originating** under
     exactly one — the version its own DNS `v=dmtap<N>` anchor advertises. A verifier reconstructs the
     preimage under the version the signer's anchor names; a mismatch fails signature verification
     (fail closed) — never a guess, and never "try both" (§18.1.6's rule for digests applies here in
     spirit: one definite reconstruction, never an oracle over candidates).
  2. **Announcement.** Supported structural versions are announced like any other capability (§10.2,
     `system` MOTE) and published in the identity's DNS anchor (§3.2), so a sender learns which
     version a recipient can verify *before* it originates.
  3. **Anti-downgrade ratchet.** The per-contact high-water-mark rule (§1.3) applies to the structural
     version exactly as it does to the suite: once a peer has been observed originating at `v(N)`, a
     later object from that peer under `v(N-1)` MUST be rejected — an attacker cannot walk an
     established relationship back onto a retired structural version.
  4. **Retirement.** The outgoing version is retired only once no pinned relationship still needs it; a
     node MAY continue to **verify** a retired version for archival objects long after it stops
     **originating** under it.

  **Honest residual.** Unlike `suite` (an in-band field, §18.1.4), the structural version is bound into
  the DS-tag *inside* the signing preimage, so a verifier must learn it out-of-band (the DNS anchor or
  a capability announcement) rather than reading it off the object. Two consequences are disclosed
  rather than solved: an identity whose anchor and DS-tag fall out of lockstep is simply **unverifiable**
  (fail-closed, never silently downgraded); and long-lived **public objects** (§22) remain verifiable
  only under the structural version they were minted at — the same archival-floor problem §22's
  origination floor (`ERR_PUB_SUITE_BELOW_FLOOR`, `0x0914`) already names for suites. This is the
  deliberate cost of not spending a redundant version byte on every object.
- Unknown `v`/`suite` MUST be rejected (fail closed), never guessed.
- New message kinds use the reserved `0x40–0x7f` range (§2.3); a node ignores unknown kinds it
  is not required to process, but MUST NOT ack a kind it cannot validate.

## 10.2 Capability negotiation (dual-stack)

- A sender discovers a recipient's capabilities via the recipient's `Identity`/DNS record
  (native DMTAP vs legacy-only) and picks the path per-recipient (§3, §7.6).
- `system` MOTEs (kind `0x0a`) carry capability announcements between nodes (supported suites,
  privacy tiers, MLS ciphersuites, the optional `deniable-1:1` mode (§5.2.1), extensions, KT
  log-types, transport substrates, `pub-1` — public-object serving opt-in, §22).
- **Wire discriminator.** `kind = 0x0A` is shared with `UsageReceipt` (§18.8a.2) and bounce/DSN
  notices (§7.10.3a), so the `Body` shape alone does not say which of the three a given `0x0A`
  MOTE carries. A capability announcement's `Headers.mime` MUST be
  `application/vnd.dmtap.capability-announcement+cbor`, and a receiver MUST inspect `Headers.mime`
  before parsing a `0x0A` `Body`, exactly as §18.8a.2 requires for `UsageReceipt`.
- **Anti-rollback: capability announcements are monotonic (normative).** A capability
  announcement is authenticated (it rides inside a `system` MOTE authenticated to the recipient,
  §2.7) but, without a version, a replayed *older* announcement could **suppress a capability the
  peer has since advertised** (e.g. hide that a peer now supports a stronger suite or the KT-v1
  log-type, forcing a downgrade). Every capability announcement MUST therefore carry a
  **monotonic `caps_version` (`u64`)**, and a receiver MUST **reject any announcement whose
  `caps_version` is older-than-or-equal-to** the last one it has accepted from that peer
  (`ERR_CAPABILITY_ANNOUNCE_ROLLBACK`, `0x030A`, §21.5) — the identical rollback-defence rule
  the spec applies to `Identity.version` (§1.3), `LocationRecord.seq` (§4.2, §16.2), and
  `GroupState.version` (§5.8.2). A receiver retains the highest `caps_version` seen per peer;
  capabilities are only ever added or upgraded across increasing versions, never silently rolled
  back by a stale replay. Absence of a recognised capability token in the *current* (highest)
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
| **Private** (OPTIONAL, research-tier — not required of any conformant node, §4.6) | Core + the opt-in mixnet (Sphinx packet + derived fleet view + 3-hop stratified paths + key-epoch rotation, [docs/research/mixnet.md §4.4.1–§4.4.4](docs/research/mixnet.md)) + sealed sender + cover traffic + the **anti-active-adversary mechanisms** (per-epoch replay caches, Poisson mixing, loop-cover attack detection, entry guards + operator diversity, and **fail-closed no-downgrade**, [docs/research/mixnet.md §4.4.6–§4.4.9](docs/research/mixnet.md)) + privacy tiers (§4, §6) + the **Bootstrap-profile rules** ([docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md): user-visible degradation, no anonymity claim, auto-upgrade-never-fall-back, per-contact ratchet) — a network small enough to need Bootstrap is the normal early case for an implementation that offers this level, so its constraints are part of the level, not an extra. The user-selectable **high-security profile** ([docs/research/mixnet.md §4.4.10](docs/research/mixnet.md)) and **PQ-Sphinx** ([docs/research/mixnet.md §4.4.12](docs/research/mixnet.md)) are OPTIONAL within this already-optional level. This level is **non-normative and not conformance-required**: it exists so an implementation that *chooses* to offer the opt-in `private` tier ([docs/research/mixnet.md](docs/research/mixnet.md)) has a byte-exact interoperability target, not because any conformant node must reach it. |
| **Groups & Files** | Core + MLS groups + content-addressed file transfer (§5) |
| **Legacy** | Core + gateway inbound/outbound + DKIM delegation (§7); gateway legacy-client surfaces (IMAP/POP/SMTP-submission + CalDAV/CardDAV) + the reachability ingress + operator modes RECOMMENDED (§7.15) |
| **Clients** | Core + **JMAP** — the node's native client surface (§8.1). Legacy client protocols (IMAP/POP/SMTP-submission, CalDAV/CardDAV) are a **gateway** capability (RECOMMENDED, §7.15), **not** a node one; a conformant node runs no legacy protocol server (§8) |
| **Auth** | Core + DMTAP-Auth login ceremony with origin binding + key-bound sessions (§13); OIDC bridge RECOMMENDED (§13.6) |

**Core is the interoperability floor, and — because the standing default privacy tier for mail and
every control MOTE is `fast` (direct/low-hop, §4.6), not `private` — Core alone is also sufficient
for a production node to operate at the protocol's own default.** A conformant node needs none of
the **Private** level: `private`/mixnet is an **opt-in, research-tier** capability
([docs/research/mixnet.md](docs/research/mixnet.md)), never a conformance requirement, and
operating a node that only ever uses `fast` is not a downgrade from anything — it is the ordinary,
undisclosed-nothing default the protocol itself specifies. An implementation that wants to *offer*
its users the opt-in `private` tier SHOULD additionally implement the **Private** level so that
choice interoperates byte-exactly with other implementations that made the same choice; the mixnet
that level depends on is fully specified, for that purpose, in
[docs/research/mixnet.md §4.4](docs/research/mixnet.md) / §6.3 (Sphinx/Loopix, with the parameters
of §16.3). Core exists so a minimal or special-purpose implementation (e.g. a gateway, a test
harness) can interoperate on the message spine — and so, equally, can a full-featured node that
has simply chosen not to offer the opt-in mixnet.

The **conformance test suite** is the *operational definition* of compatibility. "DMTAP-compatible"
means "passes the suite," not "resembles the reference." This is the primary defence against
fragmentation. The suite lives in `conformance/` as three coupled artifacts:

- **`conformance/SUITE.md`** — the normative test-case catalogue: 358 numbered cases
  (`DMTAP-<category>-<NN>`) grouped by the levels above, each pinning its spec clause, input,
  expected result (accept / reject + the §21 error code), and MUST/SHOULD.
- **`conformance/suite.json`** — the machine-readable mirror of those cases, so a runner in **any
  language** can drive them. It mirrors all 358 (SUITE.md and suite.json are in sync — the wave-2
  deniable-1:1 and KT-v1-hardening families, the `PROFILE` display-data cases, the optional
  `PUSH` wake-signaling cases, the `FILE` durability cases, the wave-3 device-cluster `SYNC`,
  `ALIAS`, and gateway-alias `GWALIAS` families, the pluggable-resolver `RESOLVE` family, the
  `PUB` public-object family, the wave-6 anti-drift families — Bootstrap profile
  (`MIXPROF`, [docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md), opt-in mixnet only),
  derived fleet view (`FLEET`, [docs/research/mixnet.md §4.4.2](docs/research/mixnet.md), opt-in
  mixnet only), guards and path diversity
  (`GUARD`, [docs/research/mixnet.md §4.4.8](docs/research/mixnet.md), opt-in mixnet only),
  location/resolution order (`LOC`, §4.2), the zero-relationship delivery
  floor (`FLOOR`, §9.7a/[docs/research/vdf.md §9.4.1](docs/research/vdf.md)), failure classes
  (`FAILCLASS`, §10.7.0) and gateway role
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

58 cases are byte-runnable today (52 vector-backed against `vectors.json`/`pub_vectors.json` +
6 self-contained canonical-CBOR reject cases); 17 further cases are verified by implementer or
deployment attestation, having **no wire bytes to recompute at all** — an in-product disclosure
(§22.7 publish consent, [docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md)'s Bootstrap
degradation notice for an implementation offering the opt-in mixnet, §25.9's bounded-lifetime /
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
58 + 19 + 281 = 358. An implementation conforms at a level iff it passes every `MUST` case of that
level and of every level it composes.

**On the coverage figure.** `make coverage` reports that **100%** of `IMPL` MUSTs sit in a section
some case cites. That is a **floor and a section-level measure**: a section counts as covered if
*any* case cites it, not if every MUST in it is exercised; it counts cases that **exist**, not
cases that **pass** (58 of 358 are byte-runnable today, and no implementation has yet been run
against the suite); and it is measured against a **curated** denominator whose classification is a
judgement, auditable in `conformance/scope.json` and re-checkable by `make lint` (check C10, which
fails the build if a MUST-bearing section is left unclassified). The uncurated figure over every
capitalised MUST is **84%**. Read the number as "nothing implementable is entirely
unattended", never as a pass mark. The reference `dmtap-core` self-check test drives the vectors,
but the spec plus these three artifacts are authoritative (§10.4), not the reference.

## 10.3a Minimum viable implementation (the falsifiable Core boundary)

**Core is not a prose aspiration — it has a mechanical acceptance test.** The **"Core level" case
block of [`conformance/SUITE.md`](conformance/SUITE.md)** is that test: passing exactly the cases
listed there is **necessary and sufficient** to claim Core conformance. That block governs; it is
**not reproduced here**, to avoid the drift a second hand-maintained list would introduce (the same
discipline the §5 kind table and the SPEC index use).

- **Derived scope (normative).** A Core-only implementation MUST implement exactly the §18 wire
  objects and §21 error codes that the Core case block exercises, and **MAY omit every other** §18
  object and §21 code — including all coordinator machinery
  ([`coordinator/CONTRACT.md` §5](coordinator/CONTRACT.md)), every non-Core profile
  ([`profiles/`](profiles/)), the opt-in `private`/mixnet tier (§4.6), PUB and PUBSUB (§22, §25), and
  the escrow / dispute / matching primitives. On receiving an object, kind, `suite`, or code outside
  its implemented scope a Core node **fails closed** (§10.1) — it never guesses, and never `ack`s
  what it cannot validate.
- **Disambiguation — two senses of "Core" (normative).** `conformance/SUITE.md`'s coverage-summary
  table prefixes many category rows with "**Core** — …". That prefix denotes the **spec area** a
  category derives from (the numbered core chapters, as opposed to a `Legacy` / `Auth` / `Private` /
  `Clients` profile); it does **NOT** mean the category is required at the Core conformance *level*.
  The table proves this on its own face: rows such as "**Core** — DMTAP-PUB extension, **optional
  `pub-1`**" and "**Core** — DMTAP-PUBSUB extension, **optional `pubsub-1`**" are area-Core and
  simultaneously optional. The Core *level* boundary is, and is only, the `## Core level` case block
  above. An implementer or verifier MUST NOT infer a Core-level obligation from the area prefix.
- **Scale of the floor (informative, this revision).** Core is ≈ **71 of ~354** conformance cases,
  ≈ **28 of ~166** registered error codes, and ≈ **300 of ~2 144** normative MUSTs across the
  numbered chapters — roughly a **15–20 % slice**. These counts are informative and drift as the
  suite grows; the Core case block, not these numbers, is the boundary.

**Honest residual.** Two costs are disclosed rather than hidden. (a) **§18 and §21 are not
tier-partitioned** — a Core implementer derives their subset from the Core case block rather than
reading a pre-filtered chapter, so the narrow-waist framing of §0/§10.3 does not extend to the two
reference chapters an implementer opens most. (b) **Core is the interoperability floor, not the
minimum conceivable effort**: its naming/KT obligations (§3, fail-closed-on-unreachable) and its
**MLS 1:1** requirement (§5) are each heavier than a two-box demo strictly needs — Core buys
byte-exact interoperability and fail-closed safety, and charges for them.

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
opt-in mixnet ([docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)), the ack-before-`250`
rule with the gateway (§7.4). **Scattering hides gaps:** a
reviewer checking one section cannot see whether the *posture as a whole* is fail-closed, and it was
exactly two such scattered gaps — the recipient-side **suite high-water-mark** (§1.3) and the
**in-force-profile floor** for High-security paths
([docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)) — that a hardening review surfaced,
because each looked complete in isolation while the set had a hole. This subsection therefore
collects **every** downgrade and fail-closed invariant into one table so the property "DMTAP never
*silently* accepts a weaker security posture" is checkable as a **set**, not rule-by-rule.

**Reading the table.** Each row is an invariant, the clause that defines it (authoritative — this
table indexes, it does not restate), the trigger that fires it, and the required behaviour/error on
violation. Codes are the §21 registry values. Where a row has no code, the failure is a
signature/decoder rejection with no dedicated status. Every "behaviour" is a **MUST** unless the
owning clause says otherwise.

### 10.7.0 Failure classes — "fail closed" names three different behaviours (normative)

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

| Invariant | Clause | Trigger | Behaviour / error on violation |
|-----------|--------|---------|-------------------------------|
| Unknown `v`/`suite` → reject | §10.1, §2.7 step 1, §1.1 | any object decoded under an unknown format version or algorithm suite | fail closed — reject, **never guess**; a signed object's DS-tag/`suite` mismatch simply fails verification |
| Highest-mutual-suite send | §1.3 | empty suite intersection with recipient | delivery fails closed — no silent weak-suite fallback |
| **Suite high-water-mark ratchet** | §1.3, §2.7 step 8 | inbound `Envelope.suite` **below** the pinned contact's high-water-mark | reject to requests + security warning, `0x020F`; the mark ratchets **up** only, lowered solely by an `IK`-authorised retirement (§1.5) |
| **Hybrid suite: no intra-suite strip** | §1.3, §16.7 | a hybrid-suite object (`0x02`) whose PQ signature component is missing/fails while only the classical component validates, presented to a verifier that **supports** the hybrid suite | reject as incomplete/downgraded hybrid, `0x0210`; a hybrid verifier MUST require **every** component signature (AND-composition) and MUST use the X-Wing KEM combiner — single-component acceptance is for a genuinely legacy verifier only, at that component's lower assurance |
| **Hybrid components are non-separable** | §1.3, §18.1.6 | a hybrid `sig-val` whose components were signed over the single-algorithm preimage rather than the suite-bound composite representative `M'`, or a component lifted out of a `0x02` object and presented as an `0x01` signature | verification fails: the two forms of the representative are distinct preimages, so a stripped or promoted component simply does not verify. An implementation that signs `M' = DS-tag ‖ 0x00 ‖ body` under a hybrid suite is non-conformant (§18.1.6) |
| **Hash prefix does not select — the suite does** | §18.1.5, §18.1.4 | a `hash` whose §18.1.5 multihash prefix disagrees with the content-hash the object's `suite` selects | reject, `0x0127` — MUST NOT verify under the algorithm the prefix names and MUST NOT try both. The prefix is self-description for suite-less objects and a redundancy check elsewhere; leaving it as an independent selector puts a downgrade channel **inside** the agility mechanism |
| **Pre-hashed preimages are algorithm-labelled** | §18.1.6, §18.9.2 | a signature computed over a **bare** digest rather than its `prefix ‖ digest` multihash form | non-conformant signer; a verifier MUST NOT accept a signature that verifies only against the unprefixed representative, `0x0127`. A bare 32-byte digest names no algorithm, so a dual-algorithm verifier would otherwise get **min**(BLAKE3, SHA3) during exactly the migration that was supposed to deliver **max** |
| **Composite suites sign the suite byte** | §18.1.6, §18.9 preamble | a hybrid-suite signature computed over `DS-tag ‖ 0x00 ‖ body` with no `u8(suite)` | verification fails — the representatives are distinct preimages. Every §18.9 preimage is the `body`; `0x02` (the v0 REQUIRED originating suite) takes `M' = DS-tag ‖ 0x00 ‖ u8(suite) ‖ body` |
| Capability-announce anti-rollback | §10.2 | `caps_version` older-than-or-equal-to the last accepted from that peer | reject the announcement, retain the higher set, `0x030A` |
| Signed-object extension gating | §10.2, §18.1.2 | an unknown integer key in a **signed** object | decoder fails closed; a reserved `≥ 64` field is sent **only** toward a peer that advertised support |

### 10.7.2 Metadata-privacy (tier/profile/mixnet) downgrades — applies only to an implementation offering the opt-in `private` tier

Every invariant below governs the **opt-in, research-tier** `private`/mixnet layer
([docs/research/mixnet.md](docs/research/mixnet.md)); a conformant node that never offers
`private` (the default tier is `fast`, §4.6) has nothing here to satisfy. For an implementation
that *does* offer `private`, these remain the correct invariants once offered:

| Invariant | Clause | Trigger | Behaviour / error on violation |
|-----------|--------|---------|-------------------------------|
| **No `private → fast` / in-force-profile floor** | [docs/research/mixnet.md §4.4.9](docs/research/mixnet.md) | no path meeting the **in-force profile's** bar (Bootstrap ≥ 3 hops/best-effort ≥ 2 ASNs; Standard ≥ 3 hops/≥ 3 operators; High-security ≥ 5/≥ 5) is buildable | hold in the retry queue; **never** silently route over `fast`, fewer hops, fewer operators, or a lower profile's bar; `0x0310`; surface to user past the retry deadline |
| **Bootstrap profile ratchets up only** | [docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md) | a contact whose relationship has already run at Standard (or above) is offered a **Bootstrap**-tier path | fail closed, `0x0310` — the profile ratchets **per contact** like the §1.3 suite high-water-mark; auto-upgrade is mandatory and fallback is forbidden, else DoSing mixes forces the whole network onto the degraded profile (the [docs/research/mixnet.md §4.4.9](docs/research/mixnet.md) attack under a friendlier name) |
| **Bootstrap profile is disclosed, never claimed as anonymity** | [docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md) | client operating on Bootstrap presents `private` as anonymous, or hides the degradation | non-conformant: the client MUST show that metadata privacy is degraded **and why** (current mixes/ASNs vs the Standard bar), and MUST NOT state an anonymity set |
| Active-attack fail-closed | [docs/research/mixnet.md §4.4.7](docs/research/mixnet.md) | loop-cover return fraction below the loss threshold (§16.3) | rotate away + `HALT_ALERT` + **fail closed** for the `private` tier — never silently continue on a weaker path, `0x030F` |
| Per-epoch mix replay drop | [docs/research/mixnet.md §4.4.6](docs/research/mixnet.md) | a Sphinx per-hop tag already in the epoch replay cache | `DROP_SILENT`, `0x030E` (cache spans every still-usable key, no hard epoch-boundary flush) |
| Mix operator-diversity **MUST be attested** | [docs/research/mixnet.md §4.4.8](docs/research/mixnet.md) | a mix whose `operator` is absent/un-attested is counted as fresh diversity | it MUST NOT count as its own operator — excluded or counted shared; keeps the ≈ *a*² compromised-path bound (else one Sybil operator collapses it to ≈ *a*) |
| Mix fleet view is **derived**, cache verified + rollback | [docs/research/mixnet.md §4.4.2](docs/research/mixnet.md) | a **cached** `MixDirectory` containing a descriptor the client cannot independently verify against its KT log quorum, or an older-or-equal `version` | reject, `0x030B` — a cache is a convenience, never an authority; a log split-view over the mix set is a KT equivocation, `0x0107` |
| **Mix fleet-view freshness (freeze defence)** | [docs/research/mixnet.md §4.4.2](docs/research/mixnet.md) | a derived view or cache older than the freshness window (§16.3, ≤ one mix-key epoch) — an adversary freezing the client on a stale, adversary-favourable fleet view | **FAIL-QUEUED** (§10.7.0): treat as stale; MUST refresh before building any `private` path; if none is obtainable the sender **queues and retries**, `0x0311` — it MUST NOT downgrade the tier and MUST NOT refuse to enqueue. A directory outage delays mail; it must never stop it. Withheld fresh descriptors are KT-detectable (no new current-epoch entry within the window), and with the authority removed no single party's silence achieves this network-wide |
| Cover traffic is not optional | [docs/research/mixnet.md §4.4.5](docs/research/mixnet.md), §6.2 | (posture) a `private`-tier node omitting loop/drop/recipient cover | non-conformant for a `private`-offering implementation — cover is load-bearing, MUST be emitted |

### 10.7.3 Trust-binding (KT / identity / group) fail-closed

| Invariant | Clause | Trigger | Behaviour / error on violation |
|-----------|--------|---------|-------------------------------|
| KT fail-closed on unreachable | §3.3, §3.5.1 | KT log unreachable/partitioned/censored at first-contact pinning | MUST NOT silently TOFU-pin — block or hard-warn + explicit acceptance, `0x0106` |
| KT equivocation → HALT | §3.5.2(d) | split view / append-only violation / stale-frozen head / sub-quorum | `HALT_ALERT`, publish evidence, evict the log, fail closed below quorum — `0x0107` / `0x0110` / `0x0111` / `0x0112` |
| Committer / co-committer fork → HALT | §5.1, §5.1.1 | two Commits at one log position with the same predecessor | `HALT_ALERT`; out-of-band fork recovery on the `> n/2` (or 2-party) quorum, `0x0404` |
| `from`-pin mismatch | §2.7 step 8, §3.4 | decrypted `Payload.from` ≠ the pinned identity for a known contact | `HALT_ALERT`, **never silently re-pin**, `0x0209` |
| Keyset-change re-verification | §3.4.1 | a verified pin's `iks`/`Identity` content-address changes | downgrade the pin to **unverified**, prompt OOB re-verify — never carry verified status across the change |
| Device attestation freshness (gated only) | §1.2a | attestation absent/invalid/expired/retired-root in an **attestation-gated** context | `FAIL_CLOSED_BLOCK` for that context, `0x0116` / `0x0118`; advisory — **never** overrides §1.4 authorisation |
| **Org-managed-undisclosed** fail-closed | §3.10.2, §18.4.7 | an escrowed-key account presented **without** its `custody = "org-managed"` marker | `HALT_ALERT`; MUST NOT present as a sovereign identity, `0x0115` |
| **Recovery-weakening quorum + veto** | §1.4 rules 3–4 | a `RecoveryPolicy` change that drops/weakens a factor | requires `rotate_threshold` **even when signed by `IK`**; takes effect only after the **72 h** asymmetric veto window (§16); a non-conforming weakening is vetoable |
| **Auth: bare-node-signed login FORBIDDEN** | §13.3.1 | a login assertion presented as node-signed / from an unattributable channel | reject outright, `0x0507`; a node MUST NOT sign a login on a user's behalf (consent-farming defence) |
| **Auth: login scope is signed** | §13.3, §18.7.2, §18.9.8 | RP would grant a scope broader than the assertion's signed `scope` | fail closed — the broader-scope preimage fails signature verification; RP MUST NOT grant beyond the signed scope (`0x0508` if surfaced as over-attenuated) |
| **Auth: delegation re-chains through recovery** | §13.4, §1.4 | a live RP session whose authorising delegation predates an `Identity.version` bump (IK rotation / recovery) | terminate at next re-validation; the revocation-list-epoch option MUST be keyed to `Identity.version`, else the recovery-invalidation guarantee silently fails to reach that RP |
| **Auth: unreachable status/KT ⇒ bounded grace, then fail closed** | §13.4 | RP's status/KT head unreachable at re-validation | honour the last-validated delegation only to a **2× grace window** (§16), then fail closed — never honour indefinitely (bounds post-revocation persistence) |
| **Auth: high-value RP multi-log / OOB** | §13.7 item 6 | high-value login against v0 single-KT-log | MUST require multi-log consistency or an OOB-verified pin — a single log can equivocate, `0x0111`/`0x0107` |
| **Auth bridge: per-RP audience** | §13.6 | bridge embeds a login assertion audienced to the bridge, reused across its RPs | the bridge MUST run a per-RP §13.3 ceremony (`aud` = target RP, fresh per-RP `cnf`); an RP verifying the key directly MUST check `assertion.aud == own identifier`, `0x0501` on mismatch |

### 10.7.4 Delivery, gateway & anti-abuse fail-closed

| Invariant | Clause | Trigger | Behaviour / error on violation |
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

| Invariant | Clause | Trigger | Behaviour / error on violation |
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
