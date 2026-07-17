# Roadmap — DMTAP-PUB (public objects) & profiles

DMTAP v0 expresses one quadrant of the confidentiality × authenticity space: **sealed to
members**. This roadmap adds the missing public quadrant — **signed by a sovereign identity,
readable by anyone** — as a thin, additive extension: a one-page hook in core, the mechanics in
a standalone extension document, and application profiles on top. Nothing here bumps the
DS-tag generation or the DNS `v=` anchor; everything rides the §10.2 capability-negotiation and
§21.25 registry machinery — dual-stack, no flag day.

Motivating parity gaps (§17): public mailing-list archives, software release announcements,
newsletters, published datasets, open-hardware part libraries. Motivating deployment: the kerf
Workshop (decentralized CAD part sharing) as the first production application.

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
5. **Serving** — gateway HTTP profile (feed endpoint, manifest/chunk fetch) and native mesh
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
- Appendix: mapping to the kerf Workshop (publish flow, gateway endpoints, LFS/sha256
  coexistence via the multihash agility prefix).

## Track 4 — Downstream (not in this repo)

- **kerf**: ADR + roadmap phases — P0 adopt object model (sign, announce, public manifests) over
  plain HTTPS gateways; P1 mirrors + swarm chunk fetch; P2 native mesh transport + MLS private
  team folders once the Envoir node ships. Clean seam preserved: cloud = billing + provisioning
  + fleet only.
- **conformance/**: PUB + profile suites (follow-up wave).
- **Envoir node/gateway**: implement `pub-1` serving (follow-up wave).

## Status

- [x] Roadmap + allocation contract (this document)
- [ ] Track 1 core hook
- [ ] Track 2 `22-public-objects.md`
- [ ] Track 3 `23-cad-artifact-profile.md`
- [ ] Cross-document consistency verify (allocations, links, README/PDF build list)
- [ ] Track 4 kerf ADR + roadmap
