# Changelog

All notable changes to the DMTAP specification are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Changed

- **spec: folded §23 (CAD/artifact profile) into §24 as the engineering-artifact facet; §23
  retained as a gap — removes the duplicated scaffolding (scope/§22-recap/licensing/lineage/
  canonical-source/HTTP/privacy/embedding) the two profiles each restated.** §24 is renamed the
  *Published-Artifact Profile* and reorganized as one generic core plus two typed facets: the
  **media facet** (§24.4–§24.17, `meta["video"]`) and the new **engineering-artifact facet**
  (§24.18, `meta["artifact"]`, folded from §23). The generic core now states once, facet-neutrally,
  the metadata-embedding + forward-compat + byte-preservation rule (§24.4), the canonical-source
  principle (§24.4.3), licensing (§24.11, which absorbs §23's CERN-OHL hardware-license table),
  revision lineage/deprecation/forks (§24.7), the derived-index/aggregate posture (§24.8),
  public-object HTTP serving (§24.12) and privacy & security (§24.13); the CAD facet references
  these instead of restating them. Everything CAD-specific — `ArtifactMetadata`, the kind/format/
  role registries, `Units`, the canonical-source specialisation, assemblies-as-Merkle-DAGs
  (pin/track, `AssemblyStructure`, BOM/dedup/cycle-rejection), workshop conventions, the `CAD-1`…
  `CAD-12` checklist, and the kerf mapping (now Appendix B) — moved intact into §24.18. `23-cad-
  artifact-profile.md` is now a stub pointing to §24.18; **the number 23 is retained as a gap and
  MUST NOT be reused or renumbered.** Spec-wide §23.x cross-references were repointed to their
  §24.18.x / §24.7 / §24.11 homes (§22, §25, §26, §27, README, substrate/, conformance/), and
  `conformance/scope.json` was remapped. The `CAD`/`CADASM` conformance families
  (`DMTAP-CAD-01`…`11`, `DMTAP-CADASM-01`) keep their ids and counts; a suite regeneration is noted
  (§24.18.10) to add a `DMTAP-CAD-12` case for the byte-retention MUST and to cover `format_id = 7`.

### Added

- **§4.4 closes H-3: an unauthenticated fragment header let a hostile exit mix inject a
  colliding-`frag_index` cell and poison reassembly, invisible to the one detector built for
  active attacks.** `SphinxFragmentHeader` (§18.5.4) is plaintext once the final hop peels it, and
  nothing binds `msg_id`/`frag_index`/`frag_count`/`total_len` to the true sender — Sphinx's `γ`
  MAC is hop-by-hop only. Any node that delivers a cell can therefore read its `msg_id` and inject
  one extra, self-crafted cell with a colliding `frag_index`; the recipient reassembles corrupt
  bytes, §2.7 step 2's content-address check fails, and the message drops **silently** until the
  sender's retries exhaust at `EXPIRED` (72 h) — for the cost of one 2,336-byte cell per suppressed
  message. §4.4.7's loop-return-fraction detector cannot see this: the attacker *adds* rather than
  *suppresses*, so every loop returns on schedule. Three changes, all in §4.4: (1) **§4.4.1's cache
  is now normatively capped**, per delivering connection/relay, on both the number of concurrently
  -open `msg_id` reassembly slots and the aggregate bytes buffered across them (two new §16.3
  parameters) — closing the cheaper variant of the attack (flooding distinct `msg_id`s at
  `frag_count = 32`, one cell each, to pin slots for up to the 15-minute reassembly timeout) at a
  fixed per-connection ceiling instead of unbounded memory; §18.5.4's own discarding of
  out-of-range `frag_index`/`frag_count` caught neither variant. (2) **A new sliding-window
  reassembly-failure counter** — colliding indices, out-of-range indices, cross-`msg_id` mixing,
  and cap-exceeded rejections, counted separately from ordinary benign-loss timeouts — feeds the
  *same* `ERR_MIX_ACTIVE_ATTACK_SUSPECTED` (`0x030F`) inference and rotate + `HALT_ALERT` +
  fail-closed response §4.4.7 already applies to an anomalous loop-return fraction, wired in at
  §4.4.7 as a second, independent input rather than a new mechanism — the detector's previously
  missing *insertion*-side counterpart. (3) **§4.4.1's text is corrected**: it said the cache is
  keyed by envelope `id`, which cannot exist until reassembly completes (`id` is the BLAKE3-256
  digest of the reassembled `ciphertext`, §2.2, computable only after the fact); it is keyed by
  `msg_id`, agreeing with §18.5.4 as already written. Two changes are reported, not made here (this
  entry does not own §16 or §18): **§18.5.4** needs `SphinxFragmentHeader` bound end-to-end — e.g.
  an authentication tag over `msg_id ‖ frag_index ‖ frag_count ‖ total_len` under a key only the
  true sender and recipient hold (derived alongside the MOTE's own encryption key, never a mix's),
  verified before a cell is admitted to a reassembly slot, so an injected cell fails verification
  instead of entering reassembly at all; and **§16.3** needs the two new cache-cap rows plus the
  reassembly-failure anomaly threshold and counting window, sized off the existing 15-minute
  reassembly-timeout and 20%-loop-loss rows rather than picked independently. A conformance case
  belongs in §10 for this path: inject one extra colliding-`frag_index` cell into an otherwise
  complete multi-cell MOTE and assert (a) the corrupt reassembly is dropped, never delivered, and
  (b) the reassembly-failure counter increments and — at an anomalous rate — drives
  `ERR_MIX_ACTIVE_ATTACK_SUSPECTED` exactly as a suppressed loop would.

- **New `26-legacy-adapters.md` generalises the legacy-bridging pattern §7 specifies for SMTP to
  every other rail a DMTAP user's correspondents live on** — SMS (hardware and aggregator),
  WhatsApp, Telegram, Discord, and Slack — without touching §7's own text or the single-sense
  meaning of "gateway" (§0's glossary). §7 gains one new pointer section, **§7.16**, that names the
  new document and states explicitly that nothing above it changes. The new document specifies:
  (1) **two deployment modes, one adapter** (§26.2) — node mode (own credentials, one identity, no
  billing, no authorisation layer) and gateway mode, which adds exactly four things: per-rail
  authorisation scope (generalizing §7.11.2 step 2's per-address claim check), a signed published
  tariff, signed usage receipts (generalizing §7.9's audit model), and a content-visibility
  disclosure (generalizing §7.15.3/§7.15.4); (2) **four fields every adapter declares** (§26.3) —
  can it initiate (freely-initiating vs. inbound-triggered), inbound transport class
  (hardware-local/outbound-persistent/webhook/listener), price shape (metered/flat/free), and
  exposure (who sees plaintext) — populated per rail **and per direction**, since WhatsApp's
  inbound/in-window-reply and outbound-cold legs differ on three of the four fields (§26.4,
  §26.4.1); (3) an **authenticity model** distinguishing email's cryptographic SPF/DKIM/DMARC
  verdict (already carried in `AuthResults`, §7.2c) from every other rail's **platform-asserted,
  cryptographically unverifiable** claim, which needs a structurally distinct `AuthResults` entry
  a client MUST NOT render with the visual weight of `dmarc=pass` (§26.5); (4) a **sovereignty
  disclosure** — node-mode numbers/handles are the user's own and portable; gateway-mode ones die
  on departure and are reassignable to a stranger, worse than mail's alias residual, and a client
  MUST be able to tell a user which applies (§26.6); (5) **reply routing as gateway-mode-only
  state** — the (rail, remote party, number/bot) → identity mapping is the per-rail generalisation
  of mail's `GatewayAliasMap` (§7.10.2), corruptible and leakable, disclosed as a gateway-mode
  concern, not adapter code (§26.7); (6) **WhatsApp credentials default to bring-your-own**, with
  BSP-backed access MAY-and-labelled, and unofficial libraries / ban-evasion number rotation ruled
  out explicitly, by the same argument §7.7 already makes for why open SMTP relays died (§26.8);
  (7) **adapters are opt-in and mostly out-of-tree** — the node core ships only the hardware-SMS
  reference adapter (off by default), the one rail that is both freely-initiating and needs no
  platform in the path at all; every other adapter versions independently so platform API churn
  never touches the node's release cycle (§26.9); (8) **economics** — only SMS has genuine
  marginal cost; free adapters still carry the exposure statement; settlement is delegated exactly
  as postage settlement already is (§9.5), with no protocol token, no advertising, and no
  payment-split or payout-fairness mechanism added (§26.10). §26.11 records the wire-format
  additions this implies for §12/§16/§18/§21 (a `GatewayAuthz` per-rail grant type, a
  platform-asserted `AuthResults` entry, an `AdapterDescriptor`/`SignedTariff` pair reusing §22's
  `pub_announce` transport where formalized, a `system`-kind usage-receipt body, and a proposed
  fourth extension registration mirroring §21.24b–d) and flags that §12.3.1's closing claim ("that
  is the entire chargeable surface of DMTAP") needs a forward reference or rephrase once §26's
  credential-gated (not resource-gated) SMS/WhatsApp charges exist alongside the SMTP gateway's
  resource-gated one — reported, not made here, since this change does not own §12.

- **§7 (the legacy gateway) closes seven normative holes an adversarial audit found in the one
  role that handles plaintext in both directions.** Each gap let the gateway's own signature
  vouch for something it never checked:
  (1) **§7.11.2 gains a per-address claim check.** Admitting an outbound relay on *authenticated
  sender* alone (§7.11.2 step 1) never bound that identity to the `From:`/`MAIL FROM` address the
  gateway then DKIM-signs as, and the delegation check a gateway holds for a domain (§7.3,
  §19.7.2) tests only that *the gateway* may sign for the domain, never that *the submitter* may
  claim an address in it — so any identity registered at a gateway could send fully
  DMARC-aligned mail as any address the gateway's domains serve. New step 2 requires the address
  be resolved (§3.3) to the submitter's own `IK`, or covered by an explicit per-address grant in
  `GatewayAuthz` (§12.2, field addition needed there); refusal is
  `ERR_GATEWAY_SENDER_ADDRESS_UNAUTHORIZED` (proposed `0x060A`, §21 registration needed).
  (2) **§7.2c is new: gateways must convey the SPF/DKIM/DMARC verdict, not just act on it, and
  must strip untrusted trust-boundary headers before signing.** §7.11.1 required evaluating
  SPF/DKIM/DMARC but nothing conveyed the verdict, and the byte-exactness rule (§7.2b) plus the
  no-annotation floor (§7.11.4) meant an attacker-supplied `Authentication-Results` header rode
  the wrapped bytes into `msg_digest` and came out laundered by the gateway's own signature. §7.2c
  requires an `AuthResults` map inside the signed attestation (`GatewayAttestation` field
  addition needed, §18.3.11), requires stripping every `Authentication-Results`/`ARC-*` header
  the gateway did not itself add before hashing (RFC 8601 §5's own rule, clarified as hygiene, not
  the content classification §7.11.4 forbids), and requires a client withhold "authenticated"
  styling from `legacy_from` absent recorded DMARC alignment.
  (3) **§7.2/§7.2a close a Payload-shape ambiguity that made two conformant gateways
  non-interoperable on the most-used inbound path.** §18.3.5 makes `Payload.from`/`sig` MUST, but
  the §19.7.1 worked example showed a MOTE with no `Payload` at all. §7.2 step 4 now requires the
  gateway construct a full `Payload` with `from` = its own `IK`; §7.2a requires that `IK` be the
  same key the domain publishes as its `_dmtap-gw` attestation key (no second signing key), and
  requires a recipient reject a `provenance` chain whose `from` isn't that published `IK`
  (reusing `ERR_GATEWAY_ATTESTATION_KEY_UNTRUSTED`, `0x0602`, since the two checks are now one).
  (4) **§7.11.1 makes `legacy_from` MUST (not optional) for the recipient-domain attestation
  entry, and requires per-sender policy key on it.** Once gateway-injected `Payload.from` is
  always the gateway's own `IK` (item 3), gating the §9.2 cold-sender standing on `Payload.from`
  alone means one accepted legacy stranger silently extends established-contact standing to every
  other stranger behind the same gateway. §9.2's `Policy`/`ContactRef` model needs a
  DMARC-aligned-address key alongside `IK` to actually enforce this — a §9 change reported
  separately.
  (5) **§7.10.4/§7.9 add the integrity residual the confidentiality-only disclosure was missing.**
  A gateway that sees the legacy leg in the clear can alter or fabricate it, in either direction,
  and still produce a `ProvenanceRecord` that verifies — it authors the bytes and signs
  `msg_digest` over them. §7.9's "cryptographically confirm" claim is now scoped: it lets a user
  confirm a real usage claim, it does not and cannot let them disconfirm a fabrication.
  (6) **§7.11.1 disambiguates ARC from AR-Chain and stops mailing lists dying to the hard-fail
  rule.** §7 used "ARC" only for the §9.3 anti-abuse credential, but RFC 8617's Authenticated
  Received Chain is exactly the SMTP-territory term an implementer reaches for by that name;
  §7.11.1 now writes "AR-Chain" throughout. The pre-existing MUST-reject-on-hard-fail rule broke
  mailing-list and forwarded mail (SPF breaks on forwarding, DKIM on footer rewriting); AR-Chain
  is now evaluated *before* the hard-fail rule, recorded as `arc=pass`, and never promoted to
  `dmarc=pass`.
  (7) **§7.2a requires the `_dmtap-gw` record name an algorithm, closing the PQ-hybrid-floor gap
  `GatewayAttestation` had no field for.** The record published only `v=`/`k=`, forcing a verifier
  to infer the algorithm from key length; it now also publishes `suite=` against the §1.1
  registry (a matching `GatewayAttestation.suite` field and a §21.21 registry update are needed
  separately), which composes with fix (3) so the gateway's `IK` — now doing double duty as the
  attestation key — carries the same floor as every other signed object.

- **The hash got the agility hooks every other primitive already had, and the honest limit that
  they still do not buy a migration.** An adversarial crypto review found that all four suites
  carried **BLAKE3-256** while §1.1 reserved `0x03` and `0x04` in advance against an AEAD break and
  a signature break — the doctrine was never applied to the one primitive everything else rests on.
  Six coupled changes:
  (1) **§1.1/§16.7/§18.1.4/§18.2/§21.15 reserve suite `0x05`**, `0x02`'s signature/KEM/AEAD with
  **SHA3-256** — a Keccak *sponge*, sharing no design lineage with the BLAKE/ChaCha ARX family — so
  the standing hash-diverse target exists before the incident, per §1.1's own "primitive-family
  diversity, not merely primitive agility" rule.
  (2) **§11.3 states the limit plainly:** every content address, Merkle root, `prev` link, KT leaf
  and signing preimage is a BLAKE3-256 digest; §18.1.5's prefix makes a *new* address expressible
  but re-anchors no *existing* graph, re-anchoring needs each author's own key, so migration is
  per-author, forward-only, and **never completes**. Content whose integrity must outlive
  BLAKE3-256 MUST be re-published by its author while that author still holds their key.
  (3) **§18.1.5 fixes precedence:** the `suite` is authoritative where present, a prefix that
  disagrees is a rejection (`ERR_HASH_ALG_MISMATCH`, `0x0127`, new in §21.3) and never a selection.
  Two conformant implementations previously disagreed on the same bytes, and an attacker who picked
  the prefix picked which hash the object's integrity rested on — a downgrade channel *inside* the
  agility mechanism.
  (4) **§18.1.6/§18.9.2 label pre-hashed preimages.** `Payload.sig` signed a bare 32-byte digest
  ("no prefix"), which is byte-indistinguishable across algorithms, handing a dual-algorithm
  verifier min(BLAKE3, SHA3) during exactly the transition `0x05` exists to make routine. The
  digest now appears in its §18.1.5 multihash form, as §18.9.1 already did with `id_bytes`. The
  `mote_payload_sig` vector is **regenerated**.
  (5) **§18.9.17 binds the key-name to its digest algorithm** — `BLAKE3-256(0x01 ‖ 0x1e ‖ ik_pub)`.
  The zero-authority floor was the one hash with no hook at all, so a migration would have changed
  every key-name **without any key rotating**, leaving the signed `KeyRotation`/`MoveRecord` chain —
  the mechanism correspondents follow a naming event by — unfired. **This invalidates the committed
  `keyname_*` vectors**, which are withdrawn (`vectors.json`, `withdrawn_vectors`) with
  `DMTAP-NAME-01`…`-05` reverted to construction-todo; regeneration needs the reference core's
  wordlist. `DMTAP-NAME-06` is unaffected.
  (6) **§18.9's preamble and §18.9.1/§18.9.2 state the composite form explicitly.** §18.9.1 spelled
  its preimage out under "Exactly:" with no `suite` byte while §18.1.6 required one for `0x02` — the
  v0 REQUIRED originating suite — so two implementations would have failed each other's
  `sender_sig` on the only suite either may originate, with every frozen vector at `0x01` where the
  forms coincide and nothing to arbitrate. `DMTAP-PRE-04` carries the `0x02` case as a
  construction-todo with an exact recipe; a real KAT needs ML-DSA-65, which the corpus still lacks.

- **§3.9.6/§16.2.1: the key-name's real margins, and "adversary-proof mode" actually defined.**
  §3.9.6 called an 80-bit truncation a "collision-resistant hash", borrowing BLAKE3-256's
  untruncated property; the true margins are ≈ **2⁴⁰** chosen-collision and ≈ **2⁸⁰**
  second-preimage, and are now stated. A client **MUST NOT** use a key-name as a security-relevant
  discriminator — allowlist entry, dedup key, or sole basis for a trust decision — because 2⁴⁰ is
  commodity work; identities are discriminated by **key**, and the human-comparable verification
  artifact is the full-`Identity` safety number (§3.4.1). The 12-word "adversary-proof mode" that
  three sections forward-referenced existed nowhere: **§16.2.1 now defines it normatively**,
  including its trigger — REQUIRED when the key-name is the only verification the parties will
  perform, and for printed/engraved/published renderings that outlive the session.

- **§22.3.3 step 1a: an origination floor for archived public objects
  (`ERR_PUB_SUITE_BELOW_FLOOR`, `0x0914`).** §22 verification is deliberately offline and zero-DNS,
  so §1.3's per-pinned-contact high-water-mark cannot apply — a first-contact archive fetch has no
  pin — and step 1 rejected only *unknown* suites, not *below-floor* ones. A post-quantum adversary
  who recovered a classical key could therefore mint a `PubAnnounce` at `0x01` with a self-asserted
  `ts`, `supersedes` the genuine final announcement, and have it verify **permanently** under §22.7
  irrevocability. The new step enforces a **local absolute floor** defaulting to the §1.1
  originating floor rather than "accept anything registered", with a narrow exception where a
  pinned `Identity` the verifier already holds establishes that the object predates that suite's
  retirement. §22.9 adds the governing disclosure: **§22's offline guarantee is about *names*, not
  *time*** — nothing in a bare announce proves when it was made.

- **§24 rescoped from a video profile to a media profile — video *and* audio — and the one signed
  structure in it given an unambiguous encoding for absent dimensions (§24.17 C-03).** Audio was
  literally unrepresentable: `Media` (§24.4.2) and `Rendition` (§24.4.3) both hard-required `width`
  (key 5) and `height` (key 6), so a song, a podcast episode or an audio-only rendition of a video
  could not be described at all, while §24.1 had claimed "video and time-based media" since the
  profile was written. Keys 5/6 become OPTIONAL in both maps under a **both-present-or-both-absent**
  rule, and **absence is the audio-only signal** — no media-kind discriminator is defined, because
  one would duplicate (and could therefore contradict) a fact the dimensions already carry, because
  the correct granularity is per-encoding rather than per-work (a video work carries audio-only
  renditions), and because the both-or-neither rule already makes the inference total. The
  security-relevant half is §24.4.4: the **signed** rendition-derivation statement is now pinned to a
  **fixed six-element array** with an absent dimension encoded as CBOR `null` (`0xf6`) at its fixed
  position — never omitted, never a `0` sentinel — reusing §18.1.1's existing preimage-`null`
  convention (as §18.9.1 already does for an absent `challenge`) rather than inventing a second one;
  a `0` sentinel was rejected because it would make "no video track" and "0 × 0 pixels" the same
  signed statement. Audio affordances stay minimal and structural: `Caption.format` gains `"lrc"`
  and the rule that an unrecognized token is skipped rather than fatal (**lyrics are captions**), and
  a **release (album/EP/single) is a `Playlist`**, which gains one OPTIONAL `cover` key (key 4); no
  album object and no album/playlist type token are defined, since no protocol behaviour depends on
  the distinction and a self-declared one would be an unverifiable claim (§24.8). **No previously
  valid object or signature changes** — because keys 5/6 were REQUIRED before, every derivation
  statement producible under the old text is byte-identical under the new one — the DS-tag
  `"DMTAP-VID-v0/derivation"` is unchanged, the profile still defines exactly one signed structure,
  and no core section, §21 registry entry, error code or wire object is touched. Conformance: §24.15
  gains VID-16/VID-17/VID-18, carried as variants of the existing `DMTAP-VIDEO-02`/`-05` and
  `DMTAP-VIDMIG-01` cases rather than as new case ids, so the catalogue count is unchanged; the
  derivation KAT, when generated, must include an audio-only (`0xf6`-dimension) vector. The file
  name, section number, `meta["video"]` key and `VideoManifest` schema name are deliberately kept as
  historical spellings — renaming any of them breaks already-published objects, every §24
  cross-reference and the conformance catalogue for no protocol gain.

- **§25 DMTAP-PUBSUB: a signed `Subscription`/`SubscriptionRevoke`, topic addressing, and
  push-hint delivery, layered on DMTAP-PUB (§22).** Closes the gap that §22's author feeds (public,
  pull-only), MLS channels (private, closed-membership, TreeKEM-churn-bound) and JMAP push
  (client-to-own-node only) leave open: machine-oriented event distribution with a real, revocable
  subscription object. Four additions, all additive and capability-negotiated (`pubsub-1`): a
  signed, mandatorily-expiring `Subscription` (§25.4) and same-author-only `SubscriptionRevoke`
  (§25.5); topic addressing at the serving/locator layer only, zero wire-object change (§25.3); new
  entries pushed as ordinary sealed MOTEs (`FeedHint`, kind `0x41`) riding the existing §2.6
  deliver/ack/retry path — no new reliability machinery — with `seq`/`tip` advisory-only and never a
  substitute for verified pull (§25.6); and fan-out explicitly governed by §9.9's existing
  group-address rules rather than a new anti-abuse model (§25.7). Default is pull-with-push-hint,
  not true push, because a publisher tracking per-subscriber delivery state is exactly the durable
  middle-state §0.5's architecture exists to avoid; a bounded inline-announce optimisation is the
  one deliberately-scoped exception (§25.6.3). Stated honestly: encrypted broadcast to a large open
  subscriber set remains out of scope for v1 (§25.11 item 1) — MLS gives confidentiality with known
  membership, §22/§25 give scale with plaintext, and wanting both at once is unsolved, not
  overlooked. No core wire change: no existing object gains a field, no `Envelope.v`/DNS `v=` bump,
  no flag day. New allocations: message kinds `0x41`–`0x43` (§21.16), capability `pubsub-1`
  (§21.22), six error codes `0x090E`–`0x0913` *within* the existing DMTAP-PUB subsystem byte `0x09`
  (§21.24d — an extension of an extension, not a new subsystem), and two DS-tags. Conformance: 15
  new `PUBSUB` cases (328 → **343**; partition 56 + 6 + 263 + 18 = 343); no new vectors
  (`conformance/vectors/` unchanged).
- **Fixed a latent one-case undercount that had propagated into five prose statements.** The
  `SUITE` family's own coverage-table row (`conformance/SUITE.md`) had not been updated when
  `DMTAP-SUITE-11` was added in the previous commit, undercounting the catalogue by exactly one
  vectored case everywhere that row's total fed into: `SUITE.md`'s own `Total` row and two
  paragraphs (327/55/61/68/42/43 → 328/56/62/69/43/44), and `conformance/README.md`'s
  byte-runnable-count sentence (which additionally already disagreed with its *own* next
  paragraph two lines down before this fix). `conformance/suite.json`'s top-level
  `vectors_count`/`referenced_vectors_count` fields carried the same stale 68/42. All now agree
  with the ground truth in `vectors.json`/`suite.json` (69 vectors, 43 driven by cases). Also
  softened the unreproducible "157 of the 183 reject cases..." breakdown in `README.md` to a
  claim actually verifiable from `suite.json` — the original split predates this fix and could not
  be reconciled with either the pre- or post-fix case set under any counting rule tried.
- **Conformance: 22 cases for the new normative requirements** (172 → **194**), closing the gap the
  hardening and one-binary-with-roles commits left open — a MUST with no case is unenforceable, and
  §10.3 makes the suite the operational definition of compatibility. New families: `MIXPROF`
  (§4.4.10a Bootstrap-profile anti-drift constraints), `FLEET` (§4.4.2 derived fleet view), `GUARD`
  (§4.4.8 persistent guard sample + ASN/attested-operator diversity), `LOC` (§4.2 per-epoch
  `peer_id`, §4.2.1 resolution order), `FLOOR` (§9.7a zero-relationship delivery floor, §9.4.1
  memory-hard-PoW floor), `FAILCLASS` (§10.7.0 failure classes) and `GWROLE` (§7.11.4/§9.11
  authorise-never-classify, §7.1b privilege separation). Partition: 46 vectored + 6 self-contained
  + 137 construction-todo + 5 manual-attestation.
- **§21.10 `0x070F` `ERR_POLICY_BELOW_FLOOR`** — referenced by §9.7a since the hardening pass but
  never allocated. The one code in the anti-abuse block whose fault is the recipient's *own* policy
  (`N_floor = 0`, or a VDF-only cold-contact requirement) rather than an inbound object. Registry:
  140 → 141 codes.

### Changed

- **Bucket ladder floor 8 KiB → 16 KiB; inline attachment cap 64 KiB → 48 KiB** (§4.4.1, §16.3,
  §16.4, §2.5, §5.5.1). The 8 KiB floor was arithmetically unsound: it was sized against *one*
  ML-DSA-65 signature and *one* public key, but a MOTE carries **two** of each (`Envelope.sender_sig`
  + `Payload.sig`, `sender_key` + `Payload.from`) plus the X-Wing encapsulated key, so the minimum
  conformant suite-`0x02` MOTE is **11 967 B** with an empty body — 3 775 B *over* the rung it was
  supposed to fit in. §4.4.1 now states the byte arithmetic explicitly, from the §18.2 lengths, so
  it cannot drift again. Two rungs are kept (a third would take the per-message size leak a pinned
  guard observes from 1 bit to log₂3 ≈ 1.58); anchor-suite (`0x04`) announcements are ordinary
  **top-rung** MOTEs at ≈ 26 kB and are *not* excluded from the inline path. The inline attachment
  cap follows: 64 KiB top rung − 11 967 B envelope ⇒ 48 KiB of content.
- **VDF demoted SHOULD → MAY** (§9.4.1, §16.5, `DMTAP-FLOOR-03`). Memory-hard PoW remains the
  interoperable MUST floor and VDF-only remains non-conformant, both unchanged. Three disclosures
  added: sequentiality is a **conjecture** defined only *relatively*, against a `p(t)`-processor
  bound (the foundational definition permits `Eval` up to poly log(t) parallelism); a VDF bounds
  **aggregate parallelism only**, leaving a **10–100×** per-gate latency advantage; and it is
  **not post-quantum** — a quantum adversary computes the group order and collapses the delay.
  The asymmetry that makes this tolerable is stated rather than hidden: a broken VDF is a *future
  spam-cost* problem, repairable locally, not a retroactive confidentiality loss like a broken KEM.
- **X-Wing's standing described accurately** (§1.3, §11.1, §11.3, §15, §16.7, README). It is
  `draft-connolly-cfrg-xwing-kem-10` on the **Independent Submission** stream, **not CFRG-adopted**,
  and **FIPS 203 standardizes no combiner**, warning that a combined KEM containing ML-KEM "might
  not meet IND-CCA2 security" and deferring to SP 800-227. Still pinned — on analysis and a fixed
  HPKE code point, not on standing. `draft-yun-privacypass-arc` likewise relabelled an individual
  draft rather than WG work (§9.3, §11.1).
- **Hybrid signatures: composite message representative, and the exact assurance level** (§1.3,
  §18.1.6, §10.7.1). AND-composition stands, but the components do **not** independently sign the
  object preimage: following the IETF LAMPS composite PQ/T construction both sign
  `M' = DS-tag ‖ 0x00 ‖ suite ‖ body`, which is what makes a component non-separable from the
  composite. Assurance stated as **EUF-CMA, not SUF-CMA** — no composite variant achieves strong
  unforgeability against a quantum adversary — with the note that DMTAP derives no identifier from
  a signature (`Envelope.id` is the content address of `ciphertext`), so it never needed it.
  Suite `0x01` signing is unchanged; the frozen vectors are all `0x01` and are byte-identical.
- **§16.7 gains the `0x04` row and §18.2 the `0x04` lengths** (`sig-val` 7 920 B, `ik-pub` 64 B) —
  the anchor suite was normative in §1.1/§1.2.0 but absent from both length registries, which is
  where the ladder arithmetic reads its numbers from.
- **§4.4.2a's growth argument labelled a design bet**, not a result: volunteer take-up of the mix
  role at scale is unmeasured, and §4.4.10a/§11.3 are what make being wrong about it survivable.

### Fixed

- **§2.6/§19.3.2 close H-6: an `ack` required no signature, so any relay, exit mix or offline-buffer
  holder could forge one from the cleartext envelope `id` alone, silently and deniably suppressing a
  message.** `Envelope.id` travels unsealed at every hop; with `ack_sig` OPTIONAL, an intermediary
  needed no key to produce a fully acceptable `ack(id)` — the sender's retry queue (§4.7, the
  system's *only* durability mechanism, §0.5) would transition straight to `ACKED`, cancel its
  deadline timer, and report delivered for a MOTE that was never received, with no signal at either
  endpoint distinguishing this from a genuine receipt. `ack_sig` is now **MUST** (§19.3.2), verified
  by the sender before the `IN_FLIGHT`/`RETRY` → `ACKED` transition (§20.1) against a key currently
  authorised under the recipient's pinned identity (`IK` or a non-revoked `DeviceCert`-chained
  device key, §1.2, reusing the §5.6.1 cluster-sync authorisation test rather than inventing a
  second one); an unsigned, wrongly-signed, or unauthorised-key ack MUST be ignored — not delivery
  evidence, no state change. A new `tier` field additionally MUST match the tier of the acknowledged
  MOTE (a `private`-tier send MUST NOT be acked over a `fast`-tier shortcut, mirroring §4.4.9's
  no-silent-downgrade rule applied to the return path). The disclosed residual: requiring `ack_sig`
  makes an ack a *specific device's* signature, which a SURB-path/exit-mix observer on the return
  leg can now use to learn that some device of the pinned recipient identity replied — a narrow,
  honestly-disclosed reduction in return-path deniability, not eliminated (some durable
  authentication of "delivery happened" is inherent to an unforgeable ack), and distinct from
  sender anonymity (SP-3/SP-4), which this does not touch. Two changes are reported, not made here
  (this entry does not own §6/§18/§21): **§18** needs the `Ack = {id, tier, ack_sig}` CDDL and a
  `DMTAP-v0/ack` DS-tag registration (preimage `DS-tag ‖ 0x00 ‖ det_cbor({id, tier})`); **§21** needs
  two new `0x03xx` codes for an invalid/unauthorised `ack_sig` and a tier-mismatched `ack`; **§6.9**
  SP-2's residual bullet needs the SURB-path disclosure above folded in. Conformance: two `FSM` cases
  belong in §10 — an unsigned/wrongly-keyed `ack` is ignored and the retry-queue entry stays
  `IN_FLIGHT`/`RETRY` (the H-6 proof-of-fix), and a genuinely-signed but tier-mismatched `ack` is
  likewise ignored.

- **§2.7/§19.3.1 close H-7: no freshness check bounded how old an accepted `ts` could be, so a
  captured, validly-signed 1:1 MOTE replayed after ageing out of dedup would pass every remaining
  check.** Three retentions for what is conceptually one cache disagreed — §16.1's "≥ 300 s" replay
  cache, §16.10's 20-day durable seen-id horizon, and §2.6's dedup-by-previously-acked set, which as
  written had no upper bound at all — while §2.7's ordered validation list had no `ts` check
  whatsoever and §18.3.1 stated `ts` is "used only for ordering/expiry, never for correctness."
  Store-and-forward made this exploitable, not merely theoretical: a MOTE genuinely retries for 72 h
  (§16.1) and can arrive from a 20-day offline buffer (§16.6), so a naively short freshness window
  would reject legitimate late delivery, and a naively long or absent one leaves replay open for as
  long as an attacker can hold a captured ciphertext. New §2.7 step 3a (mirrored at §19.3.1 step 3a)
  rejects a `ts` more than the clock-skew tolerance (§16.1, ±120 s) ahead of the receiver's clock —
  wiring up the pre-existing but previously unreachable `ERR_TIMESTAMP_OUT_OF_SKEW` (`0x020C`) — or
  more than the **durable seen-id horizon** (§16.10, currently 20 days) in the past, with **no**
  known-contact leniency on the past bound (unlike the future direction): freshness and dedup
  lifetime become the same bound by construction, not by cache-retention luck. The deniable 1:1
  mode's first-message defence (§5.2.1(a)) already had an equivalent bound; the default
  (non-deniable) path previously had none. Reported, not made here (this entry does not own
  §16/§18/§21): **§16.1 and §16.10 need reconciling into one cited parameter** for this purpose
  (currently three figures for one cache); **§18.3.1's "used only for ordering/expiry, never for
  correctness" sentence needs amending** — `ts` now also gates a correctness/security check; **§21**
  needs a new `0x02xx` code (`ERR_TS_TOO_STALE` proposed) distinct from `0x020C`, since the past
  bound is a different failure class (likely replay, not clock drift) with a different — no-leniency
  — disposition; and **the `0x0201`-block "Content-addressed dedup as replay defence" note (§21.4)
  overclaims** that dedup-by-`id` alone "absorbs" replay structurally, which was exactly the gap this
  fix closes. Conformance: two `VAL` cases belong in §10 — a `ts` older than the durable seen-id
  horizon is rejected even though `sender_sig`/`Payload.sig`/decryption all pass (the H-7
  proof-of-fix), and a `ts` just inside that horizon (a legitimately late retry or buffer-drain) is
  still accepted, proving the fix does not regress `DMTAP-VAL-14`'s existing future-skew case.

- The **class-group immaturity argument is removed** from §9.4.1/§16.5. It did not survive
  scrutiny — 2018/2019 silence on class-group performance is not evidence about 2026 — and the
  trusted-setup objection is weaker than its strong form (the literature offers a sufficiently
  large random `N`, at a disclosed cost, and class groups). What keeps a VDF out of the floor is
  the absence of a standard, an interoperable parameter set and a pinned proof encoding.
- `0x0311` (`ERR_MIX_DIRECTORY_STALE`) is **FAIL-QUEUED** per §10.7.0/§10.7.2, not
  `FAIL_CLOSED_BLOCK` — the registry still carried the pre-reclassification disposition, which is
  the exact "liveness failure handed a denial-of-service surface" error §10.7.0 exists to forbid.
- `0x030D` (`ERR_MIX_PATH_UNBUILDABLE`) now names the diversity-unmet case, not only the
  empty-layer one, and is scoped to the in-force profile's bar.
- Catalogue rows that outlived their clauses: `DMTAP-PRIV-01` still declared the `{2,8,32,64}` KiB
  bucket ladder (cut to `{8,64}`), `DMTAP-PRIV-02` and the §21.12 condition matrix still spoke of a
  mix "directory authority" (deleted — the fleet view is derived).
- `conformance/README.md` stated 157 cases / 104 construction-todo, two waves behind.

## [0.1.0] — 2026-07-21

First versioned cut of the DMTAP specification — sovereign, end-to-end-encrypted, metadata-private mail/chat/files/identity over a peer-to-peer mesh. 22 numbered sections plus conformance vectors. Spec text is CC BY 4.0.
