# Roadmap — DMTAP-PUB (public objects) & profiles

DMTAP v0 expresses one quadrant of the confidentiality × authenticity space: **sealed to
members**. This roadmap adds the missing public quadrant — **signed by a sovereign identity,
readable by anyone** — as a thin, additive extension: a one-page hook in core, the mechanics in
a standalone extension document, and application profiles on top. Nothing here bumps the
DS-tag generation or the DNS `v=` anchor; everything rides the §10.2 capability-negotiation and
§21.25 registry machinery — dual-stack, no flag day.

Motivating parity gaps (§17): public mailing-list archives, software release announcements,
newsletters, published datasets, open-hardware part libraries. Motivating deployment: the kerf
Workshop (decentralised CAD part sharing) as the first production application.

## Design contract (allocations — normative for the documents below)

| Item | Allocation |
|---|---|
| Message kind | `0x40 pub_announce` — public signed announcement (extension range, Specification Required, §2.3/§21) |
| Capability token | `pub-1` — node/operator opts into serving public objects (§10.2) |
| DS-tags | `DMTAP-PUB-v0/manifest`, `DMTAP-PUB-v0/announce`, `DMTAP-PUB-v0/feed` |
| Error block | `0x0900`–`0x09FF` — PUB extension errors (`ERR_PUB_*`), registered in §21 |
| New documents | `22-public-objects.md` (DMTAP-PUB), `23-cad-artifact-profile.md` (CAD/artifact profile) |
| Public-blob addressing | plaintext chunk hashing: `h_i = prefix ‖ BLAKE3-256(plaintext_i)`; manifest carries **no key field by construction**; distinct DS-tag makes public manifests type-incompatible with §5.5 sealed manifests (fail closed, never a boolean flag) |

## Track 1 — Core hook (edits to existing sections; ~one page total)

- **§5.5 carve-out**: plaintext content addressing remains forbidden for sealed files; it is
  permitted **only** under the DMTAP-PUB public-blob profile's distinct DS-tag, for content
  published by an explicit, irrevocable act. The CAS-confirmation attack is acknowledged and
  accepted-by-design for published-public content (the publisher's holding is public on purpose).
- **§2.3**: note `0x40 pub_announce` allocated to DMTAP-PUB (pointer to §22).
- **§10.2**: `pub-1` capability token listed among negotiable capabilities.
- **§6.6 honest limits**: two new entries — (a) publishing a public object is **irrevocable**
  (content-addressed + swarmed = un-unpublishable); (b) holders of public objects are **not
  blind** — public serving shifts the operator's moderation/liability posture, hence opt-in.
- **§21 registries**: kind `0x40`, capability `pub-1`, error block `0x0900`–`0x09FF`, DS-tag
  registrations, document references.

## Track 2 — `22-public-objects.md` (DMTAP-PUB extension)

1. Goals & non-goals (authenticity without confidentiality; not a CDN contract — durability
   remains the §5.5.1 tiered contract).
2. **Public blob profile** — plaintext-addressed Merkle-DAG manifest (chunking, streaming,
   swarm, resumability inherited from §5.5); no AEAD, no key field; global cross-user dedup as
   the explicit purpose.
3. **`pub_announce` object** — signed plaintext CBOR; the publisher's identity key signs openly
   (no sealed sender — publisher identity is the point); references a public manifest root +
   structured metadata; `supersedes` for revision chains (borrowing `edit` semantics).
4. **Author feeds** — per-identity append-only signed log of announcements; monotonic `seq`
   with the standard anti-rollback rule (same pattern as `caps_version`/`Identity.version`);
   any node can serve any feed; indexes are derived, rebuildable, never authoritative.
5. **Serving** — public-object HTTP endpoint (feed endpoint, manifest/chunk fetch) and native mesh
   fetch; swarm rules for popular objects.
6. **Operator opt-in & anti-abuse** — `pub-1` capability; per-holder serve policy; no
   protocol-level takedown (a holder chooses what it serves); §9 interaction.
7. **Client requirements** — explicit publish act, irrevocability warning (normative UX).
8. Conformance additions + §10.7-style fail-closed table (DS-tag confusion, feed rollback,
   manifest-with-key rejection).

## Track 3 — `23-cad-artifact-profile.md` (CAD/artifact profile over PUB)

- Artifact metadata schema (CBOR): name, artifact kind, formats (STEP, native, glTF, ECAD),
  units, parametric-source vs derived-mesh distinction, provenance.
- Licensing: SPDX identifiers incl. CERN-OHL-S/-W/-P; license is announce-level metadata.
- Revision lineage via `supersedes` chains; deprecation/yank as a successor announcement
  (never deletion — see irrevocability).
- **Assemblies as Merkle DAGs of parts**: an assembly references sub-parts by content address;
  BOM extraction is a tree walk; dedup composes across assemblies automatically.
- Workshop conventions: a "workshop" is a set of followed feeds; category/search indexes are
  derived data any node can rebuild.
- Appendix: mapping to the kerf Workshop (publish flow, public-object endpoints, LFS/sha256
  coexistence via the multihash agility prefix).

## Track 4 — Downstream (not in this repo)

- **kerf**: ADR + roadmap phases — P0 adopt object model (sign, announce, public manifests) over
  plain HTTPS PUB servers; P1 mirrors + swarm chunk fetch; P2 native mesh transport + MLS private
  team folders once the Envoir node ships. Clean seam preserved: cloud = billing + provisioning
  + fleet only.
- **conformance/**: PUB + profile suites (follow-up wave).
- **Envoir node**: implement `pub-1` serving (follow-up wave).

## Status

- [x] Roadmap + allocation contract (this document)
- [x] Track 1 core hook
- [x] Track 2 `22-public-objects.md`
- [x] Track 3 `23-cad-artifact-profile.md`
- [x] Cross-document consistency verify (allocations, links; build script auto-globs `NN-*.md`, so §22, §23 and §24 enter the HTML/PDF build with no list edit — rebuild pending)
- [x] Track 4 kerf ADR + roadmap (landed in the kerf repo)
- [x] `conformance/` suites for PUB + CAD profile (follow-up wave)
- [ ] Envoir node `pub-1` serving (follow-up wave)

## Spec perfection — what's left (paused 2026-07-23)

A perfection pass reality-verified every external citation against primary sources (all groundings
applied), folded §23 (CAD) into §24 as the engineering-artifact facet (§24.18), and added the README
honest-limits node-count math. Lint is 0-error. Remaining, **paused for founder review**:

1. **KEYTRANS partial delegation (§3.5).** The IETF KEYTRANS WG (`draft-ietf-keytrans-architecture`,
   `draft-ietf-keytrans-protocol`) now standardizes the monitor/auditor role taxonomy + the log+prefix-tree
   data structure that §3.5.2 hand-derives. Cite them (with the same "still a draft" disclosure X-Wing
   gets) and narrow §3.5.2 to what KEYTRANS does *not* cover and DMTAP genuinely adds: the multi-log
   `>n/2` quorum-of-a-pinned-set federation, the concrete gossip/freshness parameters, and the
   HALT/ALERT/evict incident-response.
2. **Redundancy collapses** (verified duplicates — state once + reference): §9.10/§9.11 restate §7.11's
   gateway anti-abuse MUSTs → collapse to a pointer; §4.4.10's mixnet parameter table duplicates §16.3
   (now `docs/research/mixnet.md` §4.4.10) → pointer; §7.9 ↔ §12.7 (usage-audit) → §12.7 canonical;
   §6.8 ↔ §7.8 (transport-provenance) → §7.8
   canonical; the replay/dedup cache's three divergent figures (§16.1 ≥300 s / §16.10 20-day / §2.6
   unbounded) → reconcile to one parameter (the spec itself flags this at §2.6).
3. **Relay-scope trim** — move relay from a *mechanism* DMTAP re-specifies to a *property* it requires:
   §4 states "the substrate MUST provide content-blind, direct-first reachability; v0 = libp2p (Circuit
   Relay v2 / DCUtR)" + an informative binding note; drop the vestigial, never-triggered relay codes
   `0x0305`/`0x0306`/`0x0309` from §21.
4. **Conformance-vector gap** — the new `FeedHead.topic` (key 64) and `SubscriptionRevoke` keys 5/6/7 have
   **no spec-derived vectors yet** (`gen_pub_vectors.py` / `gen_pubsub_vectors.py` don't generate them).
   Add them from spec text — never hand-matched to the implementation.
5. **Minor groundings** — §27.5.1 should state a RECOMMENDED SFrame KID epoch-window `E`; note the
   MLS-combiner / MLS-pq-ciphersuites current pre-RFC (WG-adopted-but-unpublished) status where cited.

> The §23 references in the "Status" block above are historical — §23 (CAD/artifact profile) is folded
> into **§24.18** (the engineering-artifact facet); §23 is a retained gap, not renumbered.

## Future candidates (not scheduled)

- **Sealed-media profile** — a private-content counterpart to §24 (Video/Media over DMTAP-PUB) for the
  vidmesh `keygrant`/encryption path (access-controlled, non-public video), analogous to how §5.5 sealed
  files sit beside the §22 public-blob profile; not designed here, noted only as a future candidate.
