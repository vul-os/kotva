# 24. Published-Artifact Profile — media and engineering artifacts (over DMTAP-PUB)

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174.

## 24.1 Scope & goals

This document is the **Published-Artifact Profile**: one **application profile** over the DMTAP-PUB
extension (§22) for publishing durable, signed, content-addressed **artifacts** as public objects. It
is organized as a **generic core** plus typed **facets**. The generic core — stated once, facet-neutrally
— owns everything the facets share: the "application profile over §22" framing (§24.1), the §22
relationship (§24.2), the metadata-embedding + forward-compatibility + byte-preservation rules for
carrying a facet's metadata map as signed CBOR `bytes` under a facet-named `meta` key (§24.4), the
canonical-source-vs-derived-rendition principle (§24.4.3), licensing (§24.11), revision lineage /
deprecation / forks (§24.7), the "aggregates and indexes are derived claims, never truth" posture
(§24.8), public-object HTTP serving (§24.12), and privacy & security (§24.13). A **facet** is a typed
metadata schema plus the conventions specific to one artifact domain; it names its own `meta` key,
adds only what is domain-specific, and inherits the generic core by reference rather than restating it.
Two facets are defined:

- the **media facet** (§24.4–§24.17, `meta["video"]`) — time-based media, video *and* audio; and
- the **engineering-artifact facet** (§24.18, `meta["artifact"]`) — CAD parts, assemblies, PCBs,
  schematics, drawings, datasets and supporting documents. (This facet was §23 in earlier revisions;
  §23 is now a retained gap that points here — see the §23 stub and §24.18.)

The remainder of §24.1 describes the media facet; §24.18 describes the engineering-artifact facet.

**Media facet.** A set of metadata schemas and publishing conventions for sharing **time-based media** — video *and* audio alike:
uploaded videos and their transcoded renditions, songs, albums, podcast episodes and radio sets,
captions and lyrics, channels, comments, reactions, playlists, and live streams — as public
objects. It is the **convergence path** for the *vidmesh* protocol, an
independently-designed video-sharing protocol that reinvented §22's ground (signed
content-addressed records, per-author append-only history, trustless serving, plaintext blobs)
with **incompatible bytes**. This profile carries vidmesh's *application semantics* — its manifest
shape, its verifiable-derivation rule, its channel/social/live model, its "aggregates are claims"
and "no protocol takedown" postures — onto DMTAP-PUB's *wire substrate*, so that one object model,
one identity system, and one serving surface underlie both. Where vidmesh and this profile describe
the same fact, this profile's bytes are the DMTAP-PUB bytes; §24.14 states exactly what a
vidmesh-format record must change to become a conforming public object.

It defines:

- one CBOR metadata schema, `VideoManifest` (§24.4), carried inside a `pub_announce`'s metadata
  (§22.3), describing an original media file — with or without a video track (§24.4.2) — plus
  **signed rendition derivations**, caption/lyric tracks, a thumbnail or cover image, a license,
  and fetch hints;
- a **channel** convention (§24.5) mapping vidmesh channels onto author feeds (§22.4);
- **social objects** — comment, reaction, follow, playlist (§24.6) — each an ordinary
  `pub_announce` on the actor's own feed referencing a subject by content address (or, where the
  subject is an identity, by key — §24.3.1); a **release** (album/EP/single) is the playlist
  convention, not a separate object (§24.6.4);
- **lineage** conventions (§24.7) mapping vidmesh's `supersede` / `retract` / `mirror` / `derived_from`
  onto §22's existing supersede-chain, deprecation-as-successor, cache/pin, and cross-identity
  provenance;
- the **aggregates-are-claims** posture for view/reaction counts (§24.8);
- **segmented serving** — HLS/DASH — as a *serving-layer* concern where segments are
  content-addressed blob ranges and playlists are server-local, unsigned, regenerable (§24.9);
- **live streaming** (§24.10) as an *optional capability* (`vid-live-1`): rolling signed segment
  batches that close into an ordinary VOD `VideoManifest`;
- **licensing** (§24.11), **PUB-server usage** (§24.12), the **content-moderation posture** (§24.13),
  the **vidmesh migration note** (§24.14), the **profile conformance checklist** (§24.15), and the
  **migration-guidance change log** (§24.17), corrected against a real convergence implementation.

**What this profile does and does not add to the core.** Both facets allocate **no core message kind and
no §21 error block**: every metadata map either facet defines is *unsigned application data* embedded
in an already-signed `pub_announce.meta` (§22.3.1), and every file either references is an ordinary §22
public blob (§22.2). A node implementing only §22 with no facet awareness already stores, serves, and
swarms every object this profile defines; it simply does not parse the metadata maps. **One deliberate
exception, disclosed up front:** the media facet defines **one profile-local signed structure** — the
*rendition-derivation statement* (§24.4.4) — so
an untrusted third party can transcode a video and sign accountability for the result. That statement
reuses the core's signing discipline unchanged — and *unchanged* means the whole of §18.1.6: the
DS-tag with its `0x00` separator, the suite byte inside the composite representative on every hybrid
suite, and the §18.1.5-prefixed digest wherever a signature is taken over a digest (§24.4.4) — and it
is verified by exactly the same `IK`/`DeviceCert` authorisation chain as a `pub_announce` signer
(§22.3.3); it is a *profile-scoped* signing context (`"DMTAP-VID-v0/derivation"`), not a new core
primitive. It does allocate **one** identifier in the core registries: that DS-tag string, registered
in **§21.24e** together with the `vid-live-1` capability token (§24.10). §24.4.4 states why a
construction argument is not a substitute for a registration.

**Video and audio are one object model, not two.** Audio-only media — a song, an album track, a
podcast episode, a live radio set — is published with the *same* manifest, the *same* renditions,
the *same* channel/social/lineage conventions and the *same* signed derivation statement as video.
Four consequences carry the whole audio story, and none of them is a new object: an encoding with
**no video track omits `width`/`height`** (§24.4.2), and that absence *is* the audio-only signal —
the profile defines no media-kind discriminator, for the reasons given there; **lyrics are
captions** (§24.4.2), a timed text track alongside the media, not a structure of their own; a
**release** (album, EP, single) is a `Playlist` (§24.6.4); and the derivation statement encodes the
absent dimensions explicitly so that the one signed structure in this profile stays unambiguous
(§24.4.4). The metadata key `meta["video"]` and the schema name `VideoManifest` are kept **as
historical spellings for wire and catalogue compatibility** — they name the media manifest of this
profile whatever the media kind, exactly as `endorse.gateway` (Appendix A) keeps its historical
spelling; renaming either would break every already-published object and every
conformance citation for no protocol gain.

**Definitions.** A **media work** — a *video* in this profile's historical vocabulary, and the word
this document keeps using where the video case is meant — is a piece of time-based media, with or
without a video track, published as one or more §22 public blobs referenced from a `pub_announce`
(§22.3) carrying a `VideoManifest` map — the canonical identity of the work, to which every comment,
reaction, receipt, and claim refers (never a raw blob). A **rendition** is a transcoded encoding of
that work, referenced from the same manifest, carrying a signed derivation statement (§24.4.4). A
**channel** is a named grouping of one publisher's works; the publisher's **author feed** (§22.4) is
the channel-of-record (§24.5).

## 24.2 Relationship to §22 and to vidmesh (informative recap)

**Built on §22 primitives, redefining none:** **`pub_announce`** (kind `0x40`, §22.3, a signed
plaintext CBOR announcement referencing manifests + structured metadata, with `supersedes` for
same-identity revision chains); **public-blob manifests** (§22.2, plaintext-addressed under DS-tag
`DMTAP-PUB-v0/manifest`); **author feeds** (§22.4, per-identity append-only monotonic-`seq` logs with
a signed `FeedHead` and `prev` hash-chain); **public-object HTTP serving** (§22.5.1); and **irrevocability**
(§22.7 — a published object cannot be unpublished; deprecation is a new fact, never a deletion). See
also [`substrate/FEEDS.md`](substrate/FEEDS.md), which extracts this machinery for non-mail products.

**Convergence with vidmesh.** vidmesh is a signed-record video protocol whose kernel independently
arrived at DMTAP-PUB's design — CBOR records, a signing identity that *is* a keypair, content-addressed
blobs with a BLAKE3 chunk tree, dumb relays, edge-selected moderation — but expressed it in bytes
incompatible with §22 (a different envelope, a different signing preimage, bare `b3-256` blob ids with
no multihash prefix, and **no signed feed head**). This profile is not a translation layer bolted
between two live protocols; it is the statement that vidmesh's *semantics* are correct and its *wire*
should be DMTAP-PUB's. A publisher that adopts this profile publishes DMTAP-PUB bytes and is
interoperable with every other §22 reader; §24.14 is the one-time record-shape migration.

## 24.3 Object model — the mapping

Everything a publisher emits is a `pub_announce` on **its own** author feed (§22.4); an object refers
to another by that other's content address (announce id or manifest root). The profile's objects, and
the vidmesh kind each converges from:

| Profile object | Carried in | vidmesh kind | Maps onto §22 mechanism |
|----------------|-----------|--------------|--------------------------|
| **VideoManifest** (§24.4) | `meta["video"]` | `manifest` (16) | signed `pub_announce` + public-blob manifest roots; video **or** audio-only (§24.4.2) |
| **Channel** (§24.5) | `meta["channel"]` | `channel` (36) | a `pub_announce`; the author feed is the channel-of-record |
| **Comment** (§24.6.1) | `meta["comment"]` | `comment` (32) | `pub_announce` on the commenter's feed; `subject` = target announce id |
| **Reaction** (§24.6.2) | `meta["reaction"]` | `reaction` (33) | `pub_announce`; later same-author reaction `supersedes` earlier (for counting) |
| **Follow** (§24.6.3) | `meta["follow"]` | `follow` (34) | `pub_announce`; `subject` = followed identity's `IK` — an `ik-pub`, not a content address (§24.3.1); portable social graph |
| **Playlist** (§24.6.4) | `meta["playlist"]` | `playlist` (35) | `pub_announce`; ordered list of manifest announce ids in the body — also the **release** (album/EP/single) convention |
| **Rendition** (§24.4.3) | inside `VideoManifest` | (manifest field) | derived §22 public blob + signed derivation statement (§24.4.4) |
| **Mirror** (§24.7) | `meta["mirror"]` | `mirror` (19) | a serving *assertion*; maps to the cache/pin role ([`substrate/ROLES.md § 6`](substrate/ROLES.md)) |
| **LiveManifest** (§24.10) | `meta["live"]` | `live.manifest` (112) | rolling series of `pub_announce`s; optional `vid-live-1` |
| **LiveChat** (§24.10) | `meta["live_chat"]` | `live.chat` (113) | `pub_announce`; `subject` = stream id; MAY be expired aggressively |
| **RightsClaim** (§24.11) | `meta["claim"]` | `claim.*` (48–51) | `pub_announce` asserting a claim; presented with provenance, never as truth |
| **Delegate grant** (§24.4.4) | `meta["delegate"]` | `delegate` (3) | a `pub_announce` authorising a `rendition` producer; revoked by successor |

Two structural notes that hold throughout:

- **A publisher only ever writes its own feed.** A comment on someone else's video is not appended to
  the video author's feed — it is a `pub_announce` on the *commenter's* feed that names the video's
  announce id as its `subject`. This is exactly §22.4's single-author-feed model, and it is why the
  social graph needs no shared mutable object: threads, reactions, and follows are reassembled by any
  index that crawls the participating feeds (§24.8). vidmesh's relay-`refs` filtering (its
  `["REQ", …, {refs:[…]}]` subscription) is the crawl input; the ground truth is the signed feeds.
- **`subject` is a content address — except where the field names an identity.** Where a schema below
  carries a `subject` field naming a *record* or a *blob*, it is the `hash`-typed content address
  (§18.1.5 multihash) of the referenced object — an announce id for a record subject, a
  `manifest_root` for a blob subject — inheriting §22's integrity checking unchanged. **Three fields
  in this profile name an identity rather than an object** — `Follow.subject` (§24.6.3),
  `Rendition.produced_by` (§24.4.3) and `Delegate.grantee` (§24.4.6) — and those are **not** content
  addresses and are not `hash`-typed: they are `ik-pub` values carried exactly as `PubAnnounce.pub`
  carries one, each accompanied by the `suite` that governs its length. §24.3.1 states the rule, the
  comparison, and why the alternative was rejected.

### 24.3.1 Identity references are keys, not digests (normative)

Three fields in this profile name a **DMTAP identity** rather than an object: `Rendition.produced_by`
(§24.4.3 key 8), `Delegate.grantee` (§24.4.6 key 1) and `Follow.subject` (§24.6.3 key 1). Each is
typed **`ik-pub`** (§18.1.7) and carries, in the same map, the **`suite`** (§18.1.4) whose row of
§18.2 governs its length — `Rendition` key 11, `Delegate` key 5, `Follow` key 3. The value is exactly
`Identity.iks[suite]` for that identity (§18.4.1), taken as raw public-key bytes with no CBOR head, no
length prefix and no derivation: the same bytes `PubAnnounce.pub`, `PubAnnounce.signer` and
`DeviceCert.ik` already carry.

**Why not `hash`.** These three fields were previously typed `hash`, which cannot hold an identity key
at either end of its range. §18.1.7 fixes `hash = bytes .size (33..129)` — a 1-byte §18.1.5 algorithm
prefix followed by a digest — while an `ik-pub` is **32 B** under suite `0x01` and **1 984 B** under
suite `0x02` (§18.2). A `hash`-typed `IK` is therefore not loose typing but an *unrepresentable*
value, and three encoders would have resolved it three ways — the raw key, `0x1e ‖ BLAKE3-256(IK)`,
`0x1e ‖ IK` — producing three different `VideoManifest` byte strings, three different embedded
`meta["video"]` values, three different `announce_id`s for one logical work, and three mutually
unresolvable follow graphs. It also left VID-4 unimplementable, since that MUST compares
`produced_by` against the manifest's author and the author is `PubAnnounce.pub`, an `ik-pub`:
comparing a `hash` to an `ik-pub` has no defined result.

**Why the key and not a pinned digest of it.** A digest form — specifically §18.9.17's key-name digest
`H(0x01 ‖ 0x1e ‖ Identity.iks[anchor_suite])` carried in §18.1.5 prefixed form — was considered and
rejected on three grounds:

- **The comparisons this profile actually performs are comparisons of keys.** VID-4 tests
  `produced_by` against `PubAnnounce.pub`; §24.4.4 tests the signing `DeviceCert`'s `ik` (key 2)
  against `produced_by`. Both operands are `ik-pub` on the other side, so as keys these are the
  bytewise comparisons §22.3.3 steps 4–5 already perform, with no new machinery — which is precisely
  what §24.4.4 claims the profile does. As digests, neither test is defined without a derivation this
  profile would have to invent and pin.
- **A digest breaks the offline verification §22.3.3 guarantees.** The obvious chaining rule —
  recompute the digest over `DeviceCert.ik` and compare — does not hold in general: §18.9.17's digest
  is taken over `Identity.iks[anchor_suite]`, while `DeviceCert.ik` is `Identity.iks[cert.suite]`
  (§18.4.2), and §1.2.0 explicitly contemplates an anchor suite that differs from the operational
  suite. Wherever they differ the recomputation yields a different digest, so a verifier would have to
  fetch the producer's whole `Identity` to obtain `iks[anchor_suite]` before it could decide
  authorisation — converting a zero-DNS, zero-lookup check (§22.3.3, §3.13) into an online one, for a
  third party the verifier may never have heard of. §24.4.4 designs `produced_by` to be exactly such a
  third party, so this is the common case, not the edge.
- **The stability the digest was argued for does not survive §18.9.17's own text.** §18.9.17 states
  that an **anchor-suite migration changes every key-name digest**, with no key-rotation event to
  signal it and no way to tell an old digest from a new one — so the digest is *not* stable across the
  one event a permanent rendition record most needs to survive. It is stable across re-signings of the
  `Identity` object, but so is the raw key, because the raw key is not `Identity_id`. The stability
  comparison that motivated the digest was drawn against the wrong alternative.

**Cross-suite comparison (normative).** Identity equality in this profile is **bytewise equality of an
`ik-pub` at one suite**. The same identity has *different* key bytes under different suites (§18.4.1),
and relating them requires that identity's `Identity` object. Therefore:

- **The primary test is same-suite and bytewise.** A `Rendition` is author-produced iff
  `Rendition.suite` equals the enclosing announce's `suite` **and** `produced_by` equals
  `PubAnnounce.pub` byte-for-byte. A `Delegate` grant matches a `Rendition` iff
  `Delegate.suite == Rendition.suite` **and** `Delegate.grantee == Rendition.produced_by`
  byte-for-byte. A publisher that produces its own renditions therefore signs them at the announce's
  suite — a one-line implementation rule, not a restriction on which suites an identity may hold.
- A verifier that **already holds** the referenced `Identity` MAY additionally accept a cross-suite
  match (`produced_by == Identity.iks[Rendition.suite]` for the `Identity` anchored by `pub`), but
  MUST NOT **fetch** one in order to do so. A verifier that does not hold it MUST fall through to
  §24.4.4's third-party outcome — a validly-signed but unauthorised rendition, labelled as such — and
  MUST NOT reject the manifest on that ground.
- An index MUST NOT merge two `Follow` objects naming the same identity under different suites unless
  it holds that identity's `Identity`. Left unmerged they are two edges, which over-counts one
  follower — the safe direction — rather than mis-attributing a follow to the wrong identity.

## 24.4 `VideoManifest` metadata schema

`VideoManifest` is this profile's **media** manifest: it describes a video work, an audio-only work,
or a work with renditions of both kinds, under one schema (§24.1, §24.4.2). The name and the `meta`
key are historical spellings kept for wire and catalogue compatibility, and a reader MUST NOT infer
from either that a manifest carrying no video track is malformed.

Carried in a `pub_announce`'s `meta` map (§22.3.1) under the profile-named text key **`"video"`**, as
a `bytes` value containing the deterministically encoded (§18.1.1, §18.1.2) `VideoManifest` map below.

**This embedding rule is the profile's generic core (normative), stated here once and inherited by
every facet.** It governs how *any* facet carries its metadata map as signed CBOR `bytes` under a
facet-named `meta` key — the media facet's `meta["video"]` here, the engineering-artifact facet's
`meta["artifact"]` (§24.18.1) identically — and the two are the same rule with a different key. Where
the paragraphs below say `meta["video"]` and `VideoManifest`, read `meta[<facet-key>]` and the facet's
metadata map for a facet other than media; the byte-string mechanics, the forward-compatibility
handling, and the byte-preservation obligation are facet-neutral and are not restated per facet.

The embedding keeps both grammars intact: `meta` stays a text-keyed `ext-value` map a generic
§22 reader parses unchanged (ignoring the unrecognized facet key per §21.20), while the facet
schema keeps the compact integer-keyed convention. The embedded bytes ride inside the signed announce
body, so they are covered by the announce's signature like every other `meta` value.

**Forward compatibility (normative).** A facet-aware client MUST ignore unrecognized integer keys in
`VideoManifest`, `Media`, `Rendition`, `Caption`, and every other profile map, MUST NOT treat their
presence as fatal, and MUST preserve them on re-serialize. Keys **≥ 64** are reserved for future
revisions of this profile, as §18.1.2 reserves them generally; currently-unallocated keys **< 64** are
reserved to this profile's own allocation, and a client MUST NOT assign one privately.

**Why ignore-and-preserve is legitimate here, and exactly what "preserve" means (normative).**
§18.1.2 requires a decoder processing a **signed** object to reject an unrecognized key fail-closed,
and confines ignore-and-preserve to *unsigned* objects and the text-keyed `Headers.ext` map. The
embedded bytes of `meta["video"]` **are** covered by the announce's signature — the sentence above
says so — so the justification an earlier revision of this section gave, "these are unsigned
application maps," was false as stated. The rule survives, for a different and narrower reason which
is now the normative one:

- **What the announce signs is a `bytes` string, not a map.** `PubAnnounce.sig` and `announce_id`
  (§22.3.1) are computed over `det_cbor(PubAnnounce)`, in which `meta["video"]` is a single opaque
  CBOR byte string. The profile map inside it has **no signing preimage of its own**, and nothing
  about the announce's preimage becomes ambiguous when a client fails to recognise a key inside that
  string. §18.1.2's fail-closed rule exists to keep a preimage unambiguous; here the preimage is the
  byte string, and its length and content are already fixed by the signature that covers it.
- **Preservation is byte-retention, never decode-and-re-encode (MUST).** A client that re-serializes,
  relays, mirrors, caches, or re-emits an announce MUST carry the `meta["video"]` **`bytes` value
  through unchanged, byte-for-byte**. It MUST NOT decode the profile map and re-encode it, even
  deterministically, and MUST NOT "preserve" unknown keys by copying decoded values into a map it
  rebuilds itself. Any such round-trip yields a new byte string, and a new byte string is a different
  `det_cbor(PubAnnounce)` — a broken signature and a different `announce_id` for an object whose
  signature was over the old bytes. Byte-retention is the only preservation that preserves the object
  (VID-21).
- **The §18.1.1 deterministic-CBOR bans do cross the `bytes` boundary (MUST).** This section requires
  the embedded value to be deterministically encoded (§18.1.1, §18.1.2), and that requirement is
  **not** relaxed for keys a given client does not recognise. Floating-point values, CBOR tags,
  `undefined`, NaN/Infinity, indefinite-length items, non-shortest-form arguments, out-of-order map
  keys and duplicate map keys MUST NOT appear anywhere inside the embedded map, at any depth, under a
  recognised key or an unrecognized one. A profile-aware client that decodes the embedded map and
  finds one MUST reject **the profile object** — the enclosing `pub_announce` is unaffected and
  remains verifiable, and a generic §22 reader that never decodes the value is unaffected in both
  directions (VID-21). Without this rule the preceding bullet would be unenforceable in practice: a
  float under an unknown key has no byte-stable decode/re-encode, so "preserve" and "re-encode" would
  silently disagree at exactly the place the client cannot see them disagreeing.
- **An unrecognized key never alters what a derivation signature attests.** The derivation statement
  (§24.4.4) is a fixed six-element array over fixed `Rendition` keys, so an unrecognized `Rendition`
  key is outside it by construction — it is preserved, and it is not accountable. A client MUST NOT
  let an unrecognized key change how it treats a rendition whose `derivation_sig` verifies, and a
  future revision of this profile that adds a rendition fact which *must* be accountable MUST extend
  the statement (§24.4.4) rather than rely on this ignore rule to carry it.

### 24.4.1 `VideoManifest`

```cddl
VideoManifest = {
  1  => tstr,            ; title          human-readable title (UTF-8), REQUIRED
  ? 2  => tstr,          ; description    free-form; MAY be empty
  ? 3  => [* tstr],      ; tags           ≤ 32 tags, each ≤ 64 bytes; advisory, never authoritative
  ? 4  => tstr,          ; language       BCP 47 tag for the primary audio/subject language
  5  => Media,           ; original       the source encoding, REQUIRED (§24.4.2) — the canonical rendition
  ? 6  => [* Rendition], ; renditions     derived encodings (§24.4.3), each a signed derivation
  ? 7  => [* Caption],   ; captions       subtitle/caption tracks (§24.4.2)
  ? 8  => hash,          ; thumbnail      manifest_root of a still-image public blob
  9  => tstr,            ; license        SPDX expression or a profile consent token (§24.11), REQUIRED
  ? 10 => hash,          ; channel        announce id of a Channel this video joins (§24.5)
  ? 11 => [* Hint],      ; hints          advisory retrieval hints for the referenced blobs (§24.4.5)
  ? 12 => bool,          ; retracted      true iff this revision withdraws the video (§24.7)
  ? 13 => tstr,          ; retract_reason human reason; MUST be present iff retracted = true
  ? 14 => hash,          ; derived_from   announce id of an ancestor video this remixes/reuploads (§24.7)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `title` | 1 | `tstr` | MUST | Human-readable title. UTF-8, ≤ 512 bytes RECOMMENDED. Not unique — disambiguation is the publisher's identity + feed, not the title. |
| `description` | 2 | `tstr` | OPTIONAL | Free-form prose. ≤ 16384 bytes RECOMMENDED. |
| `tags` | 3 | `[* tstr]` | OPTIONAL | ≤ 32 tags, each ≤ 64 bytes. Purely advisory index input (§24.8); carries no protocol meaning. |
| `language` | 4 | `tstr` | OPTIONAL | BCP 47 tag. |
| `original` | 5 | `Media` | MUST | The source encoding — the **canonical rendition** of the work (§24.4.2, §24.4.3). Carries `width`/`height` iff it has a video track; an audio-only work's `original` has neither (§24.4.2). |
| `renditions` | 6 | `[* Rendition]` | OPTIONAL | Derived encodings, each carrying a signed derivation statement (§24.4.3, §24.4.4). |
| `captions` | 7 | `[* Caption]` | OPTIONAL | Caption/subtitle tracks — and, on an audio-only work, lyric or transcript tracks (§24.4.2). |
| `thumbnail` | 8 | `hash` | OPTIONAL | `manifest_root` of a still-image public blob (a video thumbnail, or an audio work's cover art). |
| `license` | 9 | `tstr` | MUST | SPDX expression or one of the profile consent tokens (§24.11). |
| `channel` | 10 | `hash` | OPTIONAL | Announce id of a `Channel` object this video joins (§24.5). The channel's author MUST equal the video's author. |
| `hints` | 11 | `[* Hint]` | OPTIONAL | Advisory retrieval hints (§24.4.5) for every blob this manifest references. |
| `retracted` | 12 | `bool` | OPTIONAL | Present and `true` iff this revision withdraws the video (§24.7). Absent ⇒ `false`. |
| `retract_reason` | 13 | `tstr` | MUST iff `retracted = true` | Human-readable reason. A `retracted = true` manifest with this absent is malformed for this profile (VID-7). |
| `derived_from` | 14 | `hash` | OPTIONAL | Announce id of an ancestor video this one remixes or re-uploads (§24.7). Self-asserted provenance, never endorsement. |

### 24.4.2 `Media` and `Caption` (normative)

```cddl
Media = {
  1 => hash,     ; blob        manifest_root of the encoded file's §22 public blob
  2 => u64,      ; size        total plaintext byte size
  3 => tstr,     ; codec       RFC 6381 codecs string, e.g. "av01.0.08M.08", "avc1.640028",
                 ;             "opus", "mp4a.40.2", "flac"
  4 => u64,      ; duration    milliseconds
  ? 5 => u32,    ; width       pixels; present iff this encoding carries a video track
  ? 6 => u32,    ; height      pixels; present iff this encoding carries a video track
  ? 7 => u64,    ; bitrate     bits/second (advisory; REQUIRED in a Rendition, §24.4.3)
}

Caption = {
  1 => hash,     ; blob        manifest_root of the caption/lyric file's §22 public blob
  2 => tstr,     ; language    BCP 47 tag
  3 => tstr,     ; format      caption format token, e.g. "vtt", "srt", "lrc"
}
```

**Dimensions are optional, and their absence is the audio-only signal (normative).** `width`
(key 5) and `height` (key 6) MUST **both be present or both be absent**, in a `Media` and in a
`Rendition` alike; a map carrying exactly one of them is malformed for this profile and a
conformant client MUST reject it as such (VID-16). Both present ⇒ the encoding carries a **video
track** of those pixel dimensions. Both absent ⇒ the encoding carries **no video track**: an
audio-only work (a song, a podcast episode, a radio set) or an audio-only rendition of a work that
does have video (§24.4.3). `blob`, `size`, `codec` and `duration` are REQUIRED in every case — an
audio-only encoding is described by exactly the same four facts a video encoding is, and `codec`
carries the audio codec string (`"opus"`, `"mp4a.40.2"`, `"flac"`) with no change of rule.

**No media-kind discriminator — a deliberate omission (normative).** This profile defines no
`kind`/`media_type` field. A client MUST determine whether an encoding carries video from the
presence of `width`/`height` and MUST NOT require a separate discriminator field in order to
render an audio-only work. Three reasons this is not an oversight:

- **A discriminator duplicates a fact the dimensions already carry, and a duplicated fact can
  disagree with itself.** A manifest declaring `kind = "audio"` while carrying `width`/`height`
  would have no defined resolution and no remedy — the profile would have to pick a winner, and
  whichever it picked, the other field becomes a lie a publisher can tell for free.
- **The correct granularity is per-encoding, not per-work.** A video work legitimately carries
  **audio-only renditions** (§24.4.3), and an audio-only work may later gain a video rendition
  (a visualiser, a live capture). A work-level kind flag would mis-describe both; the per-`Media`
  rule describes exactly the encoding it appears in.
- **The inference is total.** With the both-or-neither rule above, every well-formed `Media` and
  `Rendition` is on exactly one side of it — there is no third state ("dimensions unknown") for a
  discriminator to disambiguate, because a publisher that cannot state dimensions is publishing an
  encoding whose video track it cannot describe, which this profile treats as audio-only.

**Captions, lyrics and transcripts (normative).** `Caption.format` is a **decode hint, not an
enum** — exactly like `codec`. `"vtt"` (WebVTT), `"srt"` (SubRip) and `"lrc"` (the timed-lyric
format) are the tokens in use at launch; an unrecognized `format` token MUST NOT cause rejection of
the manifest or of any other track (a client skips the track, or hands it to an external handler,
VID-18), so a new caption format is additive exactly as a new `codec` string is. **Lyrics are
structurally captions and get no object of their own:** a lyric track is a `Caption` whose
`language` is the language of the lyric and whose `format` is `"lrc"` — or `"vtt"`/`"srt"`, which
carry lyrics and transcripts as well as they carry subtitles. On an audio-only work (dimensions
absent) a caption track is by construction a lyric or transcript track rather than a subtitle
overlay, so that distinction needs no field either.

`Media.blob` (and `Caption.blob`) is a §22 public-blob `manifest_root` (§22.2.1), fetched exactly as
any §22 public manifest and content-verified per §22.2's chunk hashing, inherited unchanged. There is
**no separate `chunk_root` field** as in vidmesh: the §22 public-blob manifest *is* the chunk-hash
list and its DS-tagged Merkle root is the integrity anchor; per-chunk verification and verified range
reads are §22 machinery (and are strengthened by the optional range-proof endpoint proposed in
§24.16 / [`substrate/FEEDS.md`](substrate/FEEDS.md)). RFC 6381 `codec` strings carry over from vidmesh
unchanged — a display/decode hint, not a protocol enum.

### 24.4.3 `Rendition` and the canonical-original rule (normative)

A **rendition** is a transcoded encoding of the same work — a lower resolution, a lower bitrate, a
different codec, an audio-only extraction of a video. Renditions share the manifest's identity:
byte-different encodings of one work do not fragment into separate "videos" (this is vidmesh §004
§3, carried over).

```cddl
Rendition = {
  1 => hash,     ; blob            manifest_root of this rendition's §22 public blob
  2 => u64,      ; size
  3 => tstr,     ; codec
  4 => u64,      ; duration
  ? 5 => u32,    ; width           present iff this rendition carries a video track (§24.4.2)
  ? 6 => u32,    ; height          present iff this rendition carries a video track (§24.4.2)
  7 => u64,      ; bitrate         REQUIRED for a rendition
  8 => ik-pub,   ; produced_by     IK of the identity that transcoded — a KEY, not a digest
                 ;                 (§24.3.1, §24.4.4); length governed by key 11
  9 => sig-val,  ; derivation_sig  signature over the derivation statement (§24.4.4);
                 ;                 length governed by key 11
  ? 10 => hash,  ; derived_from    manifest_root this rendition was produced from; default = original.blob
  11 => suite,   ; suite           the PRODUCER's algorithm suite (§18.1.4), REQUIRED — governs the
                 ;                 length of keys 8 and 9 and the signing representative of §24.4.4;
                 ;                 it does NOT govern the §18.1.5 prefix of keys 1 and 10 (below)
}
```

**The canonical-original rule (load-bearing, normative).** The `original` media (`VideoManifest` key
5) is the artifact of record; every `Rendition` is a convenience encoding, never the source of truth.
This is the media facet's statement of the profile's **generic canonical-source principle** — one
canonical source of record; a derived rendition is never authoritative; a lossy/derived form MUST NOT
be the canonical one — which the engineering-artifact facet specializes for CAD formats with its
mesh-is-never-canonical guarantee (§24.18.4). Concretely:

- `original` (key 5) MUST be present and is always the canonical rendition. A consumer that needs the
  highest-fidelity source can always find it at `original.blob`, never only a lossy transcode.
- Every `Rendition` MUST carry `produced_by` (key 8), `suite` (key 11) and a valid `derivation_sig`
  (key 9) over the derivation statement (§24.4.4). A `Rendition` missing any of the three, or whose
  signature does not verify under the representative its `suite` selects, is **not an authorised
  rendition**: a conformant client MUST NOT present it as an equivalent encoding of the video (it MAY
  be shown, clearly labelled, as an unverified third-party encoding, or dropped).
- `derived_from` (key 10), when present, is the `manifest_root` the rendition was produced from; when
  absent it defaults to `original.blob`. A rendition MAY be derived from another rendition (a
  transcode chain), but the derivation statement always names the concrete input it was produced from,
  so accountability is never ambiguous.
- `width`/`height` (keys 5, 6) obey §24.4.2's both-or-neither rule and are **independent of the
  original's**: an audio-only rendition of a video omits both (this is the `original`-has-video,
  `Rendition`-has-not case the opening sentence contemplates), and every rendition of an audio-only
  work omits both throughout. Their presence *or* absence is covered by the signed derivation
  statement either way, under the fixed encoding of §24.4.4 — a rendition cannot silently drop or
  acquire a video track without invalidating its signature.

**Which suite governs what (normative).** `Rendition.suite` (key 11) is REQUIRED and is the suite of
the **producer's** keys — the identity that signed the derivation statement, which §24.4.4 designs to
be, in the general case, a party other than the announce's publisher. It governs exactly three things:

- the length of `produced_by` (key 8) and of `derivation_sig` (key 9), by the §18.2 row for that
  suite — §18.2's rule that a decoder "MUST select the length row by the suite of the key that made
  the signature, not by a single per-object suite" is the reason this field exists;
- which representative §24.4.4's signature is computed over (single-component or composite), and the
  §18.1.5 prefix carried on the statement digest inside it;
- the identity comparisons of §24.3.1.

It governs nothing else, and in particular it does **not** make `Rendition` a "containing object
carrying a `suite`" for §18.1.5's precedence rule. Every `hash`-typed field in this profile —
`Rendition.blob` (key 1), `derived_from` (key 10), and every `hash` in `VideoManifest`, `Media`,
`Caption`, `Channel`, `Playlist`, `Mirror`, `LiveManifest` and the social objects — is a content
address of a §22 object and takes its §18.1.5 prefix from the **enclosing `PubAnnounce.suite`**
(§18.1.5, §22.3.1), exactly as it did before key 11 existed. A verifier MUST NOT check those prefixes
against `Rendition.suite`, and MUST NOT treat a `Rendition` whose `suite` differs from the announce's
as malformed on that ground: a third-party producer holding a different suite from the publisher is
the normal case this field exists to describe.

`suite` MUST be present, and is deliberately not OPTIONAL-with-a-default. A verifier cannot fall back
to `PubAnnounce.suite`, because the producer is by design not the publisher; and a default reachable
two ways is the same defect as an absent field — it would let a producer sign under one suite while a
verifier reconstructs under another, which is the precise divergence §18.1.6's composite
representative exists to make impossible. A `suite` value the verifier does not implement is rejected
fail-closed (§1.1, §18.1.4): the rendition is not authorised and MUST NOT be presented as an
equivalent encoding, while the manifest and every other rendition in it are unaffected.

### 24.4.4 The rendition-derivation statement (the one profile-local signed structure)

This is the single place the media facet goes beyond the engineering-artifact facet's (§24.18) "zero new signed structures." It exists so an
**untrusted third party** can transcode a creator's video and publish the rendition with cryptographic
*accountability* — the property vidmesh calls a *verifiable derivation* (§004 §3), which lets PUB servers
serve third-party renditions "without blind trust." A derivation signature proves **accountability,
not fidelity** — a malicious transcoder can sign an unfaithful rendition; the remedy is revocation
plus edge reputation (§24.13), never a protocol guarantee of transcode correctness.

The producer signs the derivation statement:

```
stmt = det_cbor([ derived_from, rendition.blob, codec, width_or_null, height_or_null, bitrate ])
       ; ALWAYS a six-element array, in this order, whatever the media kind
       ; derived_from   = Rendition key 10 if present, else original.blob
       ; width_or_null  = Rendition key 5 if present, else CBOR null (the single byte 0xf6)
       ; height_or_null = Rendition key 6 if present, else CBOR null (the single byte 0xf6)
       ; the other three elements are exactly the Rendition's key-1/3/7 values, copied
       ; byte-for-byte out of the map — the two hash elements keep whatever §18.1.5
       ; prefix they carry there, which is the ANNOUNCE's, never the producer's (§24.4.3)

stmt_digest = H_prefix ‖ H( stmt )        ; the §18.1.5 PREFIXED form, never a bare digest (§18.1.6)
                                          ; H / H_prefix are those of Rendition.suite (key 11):
                                          ; 0x1e ‖ BLAKE3-256(stmt) for suites 0x01–0x04

; single-component suite (0x01 — verify-only, MUST NOT originate, §1.1):
M  = "DMTAP-VID-v0/derivation" ‖ 0x00 ‖ stmt_digest

; composite (hybrid) suites 0x02 / 0x03 / 0x04 / 0x05:
M' = "DMTAP-VID-v0/derivation" ‖ 0x00 ‖ suite ‖ stmt_digest
     ; suite = Rendition key 11, the 1-byte composite algorithm id (§18.1.4), INSIDE
     ; what both components sign — this is what makes the composite non-separable

derivation_sig = Sign( producer_device_key, M or M' as Rendition.suite selects )
```

- The statement binds the **input→output blob pair together with the codec/resolution/bitrate tuple**,
  so a signature cannot be replayed onto a different quality claim for the same blob pair (vidmesh
  §004 Decisions, carried over).
- The signature reuses the **core signing discipline unchanged** — where *unchanged* means the whole
  of §18.1.6, including the parts an earlier revision of this section elided: per-object suite agility
  (§1.1) carried by an explicit `suite` field on the signed structure (`Rendition` key 11, §24.4.3), a
  DS-tag prefix with the `0x00` separator, the **suite byte inside the composite representative** on
  every hybrid suite, and a **§18.1.5-prefixed digest** wherever a signature is taken over a digest
  rather than over a body. It is signed by a **device key that chains to `produced_by` via a
  `DeviceCert`** exactly as a `pub_announce` signer chains to its `pub` (§22.3.3 step 4), with
  `DeviceCert.ik` compared to `produced_by` bytewise (§24.3.1). No new algorithm and no new core
  primitive; the one identifier this profile allocates is the DS-tag string, registered in §21.24e.
- **Authorization.** A rendition is *authorised* iff `produced_by` is the manifest's author, **or** an
  identity holding an unrevoked, unexpired **`rendition` delegate grant** from the author (§24.4.6).
  Both limbs are the bytewise `ik-pub` comparisons of §24.3.1 — `produced_by` against
  `PubAnnounce.pub`, or `produced_by` against `Delegate.grantee` — never a comparison of a key against
  a digest. A client MUST verify both the derivation signature *and* the authorisation before treating
  a rendition as an equivalent encoding; an unauthorised-but-validly-signed rendition is a labelled
  third-party encoding (the signature still names *who* asserts it), not an authorised one.

**Suite binding — both representatives, stated in full (normative).** §18.1.6 fixes, for the composite
suites `0x02`/`0x03`/`0x04`/`0x05`, a message representative `M' = DS-tag ‖ 0x00 ‖ suite ‖ body` with
the suite byte *inside* what both components sign — "this is what makes the composite
non-separable" — and requires a verifier to reconstruct the form matching the object's suite. An
earlier revision of this section cited §18.1.6 while showing only
`DS-tag ‖ 0x00 ‖ BLAKE3-256(stmt)`: the **separable** single-component form, over a **bare** digest,
on a structure that carried no suite field at all. On suite `0x02` — the v0 REQUIRED originating suite
(§1.1) — that left two conformant implementations free to build different representatives, so every
hybrid rendition failed verification across the pair; and it published exactly the construction
§18.1.6 exists to forbid, in which an Ed25519 component could be stripped out of a hybrid signature
and replayed as a standalone `0x01` signature. This section therefore states both representatives in
full rather than delegating them, and `Rendition` carries `suite` (key 11, §24.4.3):

- A producer MUST compute `stmt_digest` as the §18.1.5 **prefixed** digest of `stmt` under
  `Rendition.suite`'s hash, and MUST NOT sign a bare digest. This is §18.1.6's label-the-digest rule
  applied where it says it applies: 32 bytes of BLAKE3-256 output and 32 bytes of SHA3-256 output are
  byte-indistinguishable, so a verifier implementing both would accept a signature valid under either
  and the security of the signed representative would collapse to the weaker of the two at exactly the
  moment a hash migration (suite `0x05`) was meant to deliver the stronger.
- A producer whose `Rendition.suite` is a **composite** suite MUST sign `M'` — with the one-byte
  `suite` between the `0x00` separator and `stmt_digest` — and MUST emit a `sig-val` that is the
  concatenation of the component signatures in the §18.2 order for that suite. A producer whose suite
  is `0x01` signs `M`; `0x01` MUST NOT be originated (§1.1), and the single-component form is
  specified only so that historical renditions remain verifiable.
- A verifier MUST reconstruct **exactly one** representative — the one `Rendition.suite` selects — and
  MUST reject a signature that verifies only against the other. It MUST NOT "try both", MUST NOT
  accept one component of a composite `sig-val` as a standalone signature, and MUST NOT promote an
  `0x01` signature into a composite `sig-val`. For a composite suite **both** component signatures
  MUST verify (AND-composition, §1.3, §18.1.6). A `derivation_sig` whose length does not match the
  §18.2 row for `Rendition.suite` is rejected before any verification is attempted (VID-20).
- The suite byte is covered by the signature through `M'` only, exactly as in the core; it is **not**
  an element of `stmt`, and `stmt` is byte-identical to what the previous revision of this section
  defined. The residual is the core's own and is disclosed rather than patched over: on the legacy
  single-component suite `0x01` the suite byte is outside the representative, which is one more reason
  `0x01` MUST NOT be originated. On every originatable suite the byte is inside what both components
  sign, so a third party that rewrites `Rendition.suite` in a manifest it relays produces a signature
  that no longer verifies and therefore a rendition that is *unauthorised* — a fail-closed outcome,
  never a silent reinterpretation of what the producer signed.

**The DS-tag is registered, because construction separates it from what exists and only a registry
separates it from what comes next (normative).** An earlier revision of this section claimed the
profile needed "no §21 registry entry — a *profile-scoped* DS-tag string is the only new byte, and it
is domain-separated from every core context by construction." Half of that is true and worth stating
plainly, because it is a real structural strength: every DS-tag is an ASCII string terminated by a
single `0x00` byte (§18.1.6) and `0x00` occurs in none of the tags themselves, so the tag set is
**prefix-free** — no tag's preimage can be a prefix of another's, and no signature under
`"DMTAP-VID-v0/derivation"` can be reinterpreted under `"DMTAP-PUB-v0/announce"`,
`"DMTAP-v0/device-cert"`, or any other tag now in this specification. That guarantee holds without a
registry and does not depend on one.

The other half does not follow. Separation from tags that **do not exist yet** is not a property of
the construction — it is a property of the allocation process, and the identical construction argument
would be available to a future extension that happened to choose the same string. Every prior
extension registered its tags for exactly this reason: §21.24b (`DMTAP-PUB-v0/manifest`,
`…/announce`, `…/feed`), §21.24c (`DMTAP-SYNC-v0/…`), §21.24d (`DMTAP-PUB-v0/subscription`,
`…/subscription-revoke`). §24 was the first extension to opt out, on grounds that were equally true of
all of them. `"DMTAP-VID-v0/derivation"` is therefore **registered in §21.24e**, together with the
`vid-live-1` capability token (§24.10). That registration reserves an identifier and nothing more: it
allocates **no** message kind, **no** error-code block and **no** subsystem byte, so the profile's
core property — no new core message kind, no §21 error block, one profile-local signed structure — is
intact (§24.1).

**Absent dimensions in the signed preimage — fixed arity, CBOR `null` (normative).** Because
`width`/`height` are OPTIONAL (§24.4.2) and this statement is **signed**, the encoding of their
absence is a security property, not a formatting choice: two encodings of "the same" derivation
would mean two byte strings a signature could be over, which is precisely the signature-ambiguity
class a domain-separated preimage exists to close. Therefore, normatively:

- The statement is **always a six-element array**, in the order above. A producer MUST encode an
  absent `width` or `height` as the single byte `0xf6` (CBOR `null`, major type 7 value 22) **at its
  fixed position**; it MUST NOT omit the element, MUST NOT shorten the array, and MUST NOT
  substitute `0` or any other in-band value (VID-17).
- A verifier MUST reconstruct the statement by exactly this rule from the `Rendition` it holds, and
  verify `derivation_sig` over that one reconstruction. It MUST NOT try alternative reconstructions
  (a shortened array, a `0` sentinel) in order to make a signature verify — accepting a second
  encoding re-introduces the ambiguity this rule removes.
- Because §24.4.2 requires `width` and `height` to be both present or both absent, the fourth and
  fifth elements are either **both** integers or **both** `0xf6`. A `Rendition` carrying exactly one
  of them is malformed (§24.4.2, VID-16) and MUST be rejected as malformed *before* signature
  verification, not treated as a merely-unauthorised rendition.

*Why `null`, and why not the alternatives (rationale, normative-by-consequence).*

- **`null` is the core's existing answer, not a new invention.** §18.1.1 already names CBOR `null`
  (`0xf6`) as the canonical representation of an absent optional field **inside a signing preimage**,
  and §18.9.1 already uses it exactly this way for an absent `Envelope.challenge`. Reusing that
  convention keeps one rule in the specification instead of two. The complementary §18.1.1 rule — an
  absent optional field is **omitted** on the wire, never sent as `null` — is untouched: a
  `Rendition` on the wire still simply omits keys 5 and 6, and `0xf6` appears **only** in the
  reconstructed preimage, which is never transmitted and never decoded from the wire.
- **Variable arity was rejected.** A four-element `[derived_from, blob, codec, bitrate]` for an
  audio-only rendition and the six-element form for the same rendition are different byte strings,
  so two conformant implementations could disagree about which one a signature covers and a producer
  could sign whichever it preferred. A signature is *accountability* only if exactly one byte string
  is signable per rendition.
- **A `0` sentinel was rejected.** `0` is a syntactically valid `u32` in both fields, so a
  `0`-sentinel makes "this encoding has no video track" and "this encoding is 0 × 0 pixels" the
  **same signed statement** — the signature would stop distinguishing two claims it exists to bind,
  and a statement signed for a degenerate video rendition would replay onto an audio-only claim for
  the same blob pair. `null` is outside the value space of keys 5/6, so present and absent are
  disjoint by construction.
- **Nothing else in `stmt` changes, and no video statement's `stmt` changes.** The DS-tag string
  remains `"DMTAP-VID-v0/derivation"`, the element order, the hash function, the signature algorithm
  and the authorisation rule are unchanged, and this profile still defines **exactly one** signed
  structure (§24.1). Since `width`/`height` were previously REQUIRED, **every `stmt` that was valid
  under the previous revision of this section is byte-identical under this one** — only the previously
  unrepresentable audio-only case gains an encoding (§24.17, C-03). The *representative* built over
  that `stmt` did subsequently change — it gained the §18.1.5 prefix on the digest and, on a composite
  suite, the suite byte (§24.17, C-05) — but that is a separate correction one level above the array,
  and it leaves this subsection's fixed-arity rule exactly as stated.

### 24.4.5 `Hint` — advisory retrieval hints

```cddl
Hint = [ hint_type: uint, value: tstr ]
```

`hints` (in `VideoManifest`, `Mirror`, and live objects) are **advisory and additive**: any host
answering for a content address is equivalent, because the address proves integrity regardless of
origin (§22.5.1). A client MUST NOT treat a blob fetched from an unlisted source differently, and MUST
NOT treat a hint as authoritative over the content address. The profile-local hint-type registry
(proposed additively for the whole waist in §24.16 / [`substrate/FEEDS.md`](substrate/FEEDS.md)):

| `hint_type` | Name | Value |
|------------:|------|-------|
| `1` | `https` | URL serving the blob with HTTP Range (a §22 public-object `chunk` surface, or any range server) |
| `2` | `torrent-v2` | BitTorrent v2 infohash, lowercase hex |
| `3` | `relay-blob` | base URL of a relay/PUB-server blob surface (§22.5.1) |
| `4` | `bundle` | locator of a self-verifying bundle (§24.14 note) containing the blob |

New transports are new hint types; unrecognized types MUST be ignored, never rejected.

### 24.4.6 `Delegate` grant for `rendition` producers

A `Delegate` object (`meta["delegate"]`) grants the `rendition` capability from the video author to a
producer identity, so a transcoding service can publish *authorised* renditions on the author's behalf.

```cddl
Delegate = {
  1 => ik-pub,     ; grantee     IK of the identity receiving the capability — a KEY, not a digest
                   ;             (§24.3.1); length governed by key 5
  2 => tstr,       ; capability  registered capability name; "rendition" at launch
  ? 3 => u64,      ; expires_at  Unix seconds; absent = until revoked
  ? 4 => bool,     ; revoked     true revokes the grant named by supersedes
  5 => suite,      ; suite       the GRANTEE's algorithm suite (§18.1.4), REQUIRED — governs key 1's
                   ;             length and the bytewise match of §24.3.1
}
```

A grant is a `pub_announce` with `Delegate.revoked` absent. A revocation is a **successor
`pub_announce`** whose `supersedes` names the grant and whose `Delegate.revoked = true`, with
`grantee`, `capability` **and `suite`** matching the grant (irrevocability + supersede-only, §22.3.4);
a revocation that changes `suite` names a different identity reference and revokes nothing. Delegation
never transfers identity; it authorises exactly the `rendition` action §24.4.4 consults. A grant
matches a `Rendition` iff `Delegate.suite == Rendition.suite` **and**
`Delegate.grantee == Rendition.produced_by`, byte-for-byte (§24.3.1) — the same bytewise `ik-pub`
comparison §22.3.3 already performs, never a key-against-digest comparison. This maps vidmesh's
`delegate` (kind 3) onto the §22 supersede mechanism with no new machinery.

## 24.5 Channels are author feeds

A **channel** is a named grouping of one publisher's works — a video channel, an artist's catalogue,
a podcast. The mapping has two layers:

- **The author feed is the channel-of-record.** A publisher's §22 author feed (§22.4) — its
  per-identity, `seq`/`prev`-chained, signed-`FeedHead` log — *is* its channel: to "subscribe to a
  channel" is to follow that feed (§24.6.3), and every video the publisher announces appears on it in
  order, with anti-rollback and equivocation detection for free (§22.4.2). No separate channel object
  is required for the common one-identity-one-channel case.
- **Sub-channels for publishers who want more than one.** A publisher MAY partition its feed into named
  sub-channels by publishing a `Channel` object and having each video reference it via
  `VideoManifest.channel` (key 10). This carries over vidmesh's "an identity MAY have many channels."

```cddl
Channel = {
  1 => tstr,     ; title        REQUIRED
  ? 2 => tstr,   ; description
  ? 3 => hash,   ; avatar       manifest_root of an image blob
  ? 4 => hash,   ; banner       manifest_root of an image blob
}
```

A `Channel` is a `pub_announce` on the publisher's own feed; a video "joins" it by naming its announce
id in `VideoManifest.channel`. A client MUST reject a `channel` reference whose named announce has a
different `pub` than the video's author (a video cannot join another identity's channel) — this is the
same-author constraint vidmesh states in §003 §4.1, enforced here by comparing the two announces' `pub`
(exactly the §22.3.3-style `pub`-match already required for `supersedes`).

## 24.6 Social objects

Each social object is an ordinary `pub_announce` on the **actor's own** feed. None mutates the
subject's feed; a crawler reassembles threads, tallies, and graphs from the participating feeds
(§24.8). `Comment.subject`, `Comment.parent` and `Reaction.subject` are content addresses (§24.3);
`Follow.subject` is the one exception in this section — it names an *identity*, so it is an `ik-pub`
with an accompanying `suite`, not a `hash` (§24.3.1, §24.6.3).

### 24.6.1 `Comment`

```cddl
Comment = {
  1 => hash,     ; subject   announce id of the VideoManifest or LiveManifest commented on
  2 => tstr,     ; text      non-empty, ≤ 8192 bytes
  ? 3 => hash,   ; parent    announce id of a parent Comment, for threading
  ? 4 => [* hash]; media     manifest_roots of attached media blobs
}
```

Threads merge by reference in any arrival order (partition posture). A `parent`, when present, MUST
itself reference the same `subject` (a reply cannot jump subjects) — a client validating a thread MUST
enforce this (VID-9). Editing a comment is a same-author `supersede` (§24.7); the original announce id
remains the stable reference for replies.

### 24.6.2 `Reaction`

```cddl
Reaction = {
  1 => hash,     ; subject    announce id of the reacted-to record
  2 => tstr,     ; reaction   single emoji grapheme cluster or registered token, ≤ 32 bytes
}
```

**Later supersedes earlier, for counting.** An identity's newer reaction to the same subject
`supersedes` (§22.3, §24.7) its earlier one — a same-author supersede chain per (subject, actor), so an
index counts **at most one** current reaction per identity per subject (vidmesh §003 §5.2, carried
over). Removing a reaction is a `retract`-style successor (§24.7).

### 24.6.3 `Follow`

```cddl
Follow = {
  1 => ik-pub,   ; subject   the followed identity's IK (§1.2) — a KEY, not a content address
                 ;           (§24.3.1); length governed by key 3
  ? 2 => tstr,   ; note
  3 => suite,    ; suite     the SUBJECT's algorithm suite (§18.1.4), REQUIRED — governs key 1's
                 ;           length and the graph-merge rule of §24.3.1
}
```

A follow's `subject` is the **followed identity's `IK`**, not an announce id — following an *identity*
(its whole feed / channel-of-record, §24.5), not one video. It is therefore the one `subject` in this
profile that is **not** a content address, and it is typed `ik-pub` rather than `hash` for the reasons
§24.3.1 gives: a `hash` cannot hold an identity key at either end of its 33–129-byte range, and an
index reassembling a follow graph must be able to compare `subject` bytewise against the `pub` of the
feed it names. Two follows naming the same identity under different suites are two edges unless the
index holds that identity's `Identity` (§24.3.1). Publishing the follow as a `pub_announce`
(rather than keeping it client-side as the engineering-artifact facet's "workshop" does, §24.18.9) makes the **social graph portable**: any
index that crawls follows can reconstruct any identity's follower/following graph, and no PUB server owns
it (vidmesh §003 §5.3). Unfollowing is a `retract`-style successor (§24.7). A client that prefers a
private follow set MAY instead keep it client-side (the §24.18.9 workshop model) and simply not publish
`Follow` objects — publishing is opt-in, and an unpublished follow is invisible by construction.

### 24.6.4 `Playlist`

```cddl
Playlist = {
  1 => tstr,        ; title       REQUIRED
  ? 2 => tstr,      ; description
  3 => [+ hash],    ; entries     ordered announce ids of VideoManifests; non-empty
  ? 4 => hash,      ; cover       manifest_root of a still-image public blob (cover / playlist art)
}
```

Entries live in the body (not in refs) so that **reordering is an ordinary same-author `supersede`**
with a replacement `Playlist` body (vidmesh §003 §5.4, carried over). The original announce id remains
the playlist's stable identity across reorderings. `cover` (key 4) is an OPTIONAL still-image blob —
album art for a release, playlist art for a collection — advisory in exactly the way
`VideoManifest.thumbnail` is; a client with no cover falls back to an entry's own `thumbnail` or to
nothing.

**A release (album, EP, single) is a `Playlist` — no new object.** A release is an ordered, titled,
non-empty list of works published by one identity, whose order is meaningful, whose art is one
image, and whose corrections are same-author revisions. That is `Playlist`, field for field: the
track order is `entries` in track order, the release title and liner notes are `title` and
`description`, the cover is `cover`, a re-issue or a corrected track order is an ordinary
`supersede` (§24.7), and the release's stable identity across re-issues is the original announce
id. Nothing a release needs is missing: each track is an ordinary `VideoManifest` with audio-only
`Media` (§24.4.2) carrying its own title, language, license, lyrics (§24.4.2) and renditions, so
per-track facts live on the track where they belong rather than in a release-level table that could
contradict them. A compilation across artists needs nothing extra either — an entry is a content
address, so it MAY name any author's manifest, and a release MAY equally mix audio and video
entries.

The profile deliberately defines **no album/playlist type token**. No protocol behaviour depends on
the distinction — serving, verification, lineage and counting treat every `Playlist` identically —
and a self-declared `type: "album"` would be an unverifiable assertion of exactly the kind §24.8
requires be presented as a claim rather than a fact. Where a presentation layer wants the
distinction it MAY use the fact the objects already carry: a playlist whose entries are all
announces by the playlist's own author is an artist's own collection (the release case), while
mixed authorship is a compilation or a listener's mixtape. That heuristic is a display choice, is
derived from signed authorship rather than from a label, and carries no protocol weight.

## 24.7 Lineage — supersede, retract, mirror, remix

All four map onto §22's **existing** lineage mechanics; the profile adds no new lineage machinery.

- **Revisions = same-author `supersedes` (§22.3, §24.5-style `pub`-match).** Editing any object — a
  video's metadata, a comment's text, a playlist's order — is a fresh signed `pub_announce` naming the
  announce id it supersedes; `supersedes` is strictly *same-identity* history, and a reader following a
  chain to its head resolves "the current version." This is vidmesh's `supersede` (kind 17) with its
  "complete replacement body, not a diff" rule preserved: the new `meta[<key>]` map is the whole new
  state.
- **Retract = deprecation-as-successor, never deletion (§22.3.4, §22.7).** To withdraw a video, the
  publisher issues a **successor** `pub_announce` that `supersedes` the prior one with
  `VideoManifest.retracted = true` and a `retract_reason` (§24.4.1). Consistent with irrevocability,
  the retracted bytes remain fetchable — only their status changes. A conformant client MUST surface a
  retracted head distinctly (e.g. a banner) and MUST NOT silently hide it, and MUST NOT present any
  operation as *deleting* the prior bytes (VID-8). This is vidmesh's `retract` (kind 18) — "a request,
  not an erasure" — realized as §22 supersede-only.
- **Mirror = a serving *assertion*, mapped to the cache/pin role.** A `Mirror` object
  (`meta["mirror"]`) is one identity asserting "I serve these blobs, here":

  ```cddl
  Mirror = {
    1 => [+ hash],   ; subjects   manifest_roots (pin the blob) and/or VideoManifest announce ids
                     ;            (pin every blob that manifest references)
    ? 2 => [* Hint], ; hints      where the author serves them (§24.4.5)
  }
  ```

  A mirror **drives discovery and holder-set membership**; it asserts intent, not an enforceable
  promise — availability is the emergent sum of independent holder choices (§22.6.2). This is exactly
  the **cache/pin infrastructure role** ([`substrate/ROLES.md § 6`](substrate/ROLES.md)): pinning is a
  deliberate, real-storage durability act, and serving public plaintext is explicit opt-in
  (`pub-1`, §22.6.1), never automatic. vidmesh's `mirror` (kind 19) *is* a pin-assertion; here it is a
  discovery hint over the §22 cache/pin role.
- **Remix / re-upload = cross-identity `derived_from` provenance.** A different identity
  republishing or remixing a video sets `VideoManifest.derived_from` (key 14) to the ancestor's announce
  id. This is **self-asserted provenance, not permission or endorsement**: publishing a
  `pub_announce` needs no consent from the original publisher (the video is public), so `derived_from`
  exists only so provenance is *discoverable*, never so it can be gated. A client MUST NOT interpret it
  as authorisation. (Note the distinction from `mirror`: a mirror serves the *same* bytes under the
  *same* manifest; a `derived_from` remix is *new* bytes by a *new* author citing an ancestor.)

## 24.8 Aggregates are claims, never truth

View counts, reaction tallies, "trending," and recommendation rankings are **per-server computed
claims**, not protocol facts — the posture both vidmesh (§006 §7, §009 §6) and DMTAP-PUB (§22.4)
already hold identically. This is the profile's **generic "indexes and aggregates are derived,
never authoritative" rule**: it governs the media facet's tallies here and the engineering-artifact
facet's category/search/workshop indexes (§24.18.9) alike — any node MAY recompute an index or an
aggregate from the signed feeds, and no computed index is authoritative over the announces it was
derived from.

- **Any node can compute an aggregate; none is authoritative.** A PUB server, a client, or a community
  index sums reactions or estimates views by crawling the feeds it knows about. Two indexes MAY
  legitimately disagree (different crawl coverage, staleness, or heuristics) without either being
  "wrong": the ground truth is the signed `pub_announce`s themselves, re-derivable by any client.
- **Published tallies are attestations.** A PUB server MAY publish a signed count as a `RightsClaim`-style
  attestation (`meta["attest"]`, converging vidmesh's `attest` kind 82) — "server G asserts this video
  reached X views on G" — that others may display or sum. Such a tally is worth exactly the attester's
  reputation; a client that shows it MUST label it as a per-server claim ("views on this server"),
  never as a network-wide truth. Fraud-proof global counting is out of scope for both systems by design.
- **Reaction counting specifically** uses the supersede-latest rule of §24.6.2: an index counts each
  identity's current reaction per subject, discarding superseded ones.

## 24.9 Segmented serving (HLS / DASH) — a serving-layer concern

Adaptive streaming (HLS, MPEG-DASH) is realized **entirely at the serving layer**, changing nothing
about what is signed. This design is carried over from vidmesh unchanged, because it is correct.

This is media-kind-agnostic: an audio-only rendition (§24.4.2) streams by exactly the same mechanism —
an audio HLS variant is byte ranges of the same signed blob, verified against the same Merkle root.

- **Segments are content-addressed blob ranges.** A player fetches byte ranges of a signed rendition's
  §22 public blob (via §22's `chunk` surface, or the optional range-proof endpoint of §24.16) and
  verifies each range against the rendition's DS-tagged Merkle root — the same integrity a whole-file
  fetch has, at segment granularity.
- **Playlists are server-local, unsigned, and regenerable.** An `.m3u8` / DASH MPD is **serving-layer
  output a PUB server synthesizes on demand** from the signed manifest — it points at segment/range reads
  of the signed rendition blobs. It is **not an object of record**: it carries no signature, is not
  content-addressed, and MUST NOT be treated as authoritative. A PUB server MAY regenerate a playlist at
  will (different segment durations, different CDN base URLs); two servers' playlists for the same
  video may differ byte-for-byte while both stream the identical signed bytes.
- **Only whole-file renditions are signed.** The unit of signed truth is the rendition's *whole-file*
  §22 public blob + its derivation statement (§24.4.4). Per-segment objects are never separately signed
  (there is nothing to sign — they are byte ranges of an already-signed, already-Merkle-rooted blob).
  A conformant client MUST verify streamed segments against the signed rendition's Merkle root and MUST
  NOT accept a playlist's segment list as a substitute for that verification (VID-11).

## 24.10 Live streaming (optional capability `vid-live-1`)

Live streaming is an **optional capability**, advertised by the profile-local capability token
**`vid-live-1`** using the core capability machinery (§10.2) — a publisher advertises it; a consumer
that has not advertised it simply does not do low-latency live follow, and its silence is never a fault
(capability-absence rule, §21.22). The base video profile (VOD, §24.4–§24.9) rides on `pub-1` alone,
exactly as the engineering-artifact facet (§24.18) rides on `pub-1`; `vid-live-1` is additive on top.

A live stream is a **rolling series of signed `pub_announce`s** on the streamer's own feed, each
carrying a `LiveManifest`:

```cddl
LiveManifest = {
  ? 1 => tstr,          ; title      REQUIRED in the first record of a stream; absent thereafter
  2 => u64,             ; seq        0 in the stream's first record, then strictly +1 (stream-local)
  3 => [+ LiveSegment], ; segments   ordered segment batch appended by this record
  4 => bool,            ; final      true closes the stream
  ? 5 => hash,          ; stream     announce id of the first LiveManifest (the stream id); absent in the first
  ? 6 => [* Hint],      ; hints
}

LiveSegment = [ blob: hash, duration_ms: uint ]   ; blob = manifest_root of a 2–6 s media blob
```

A live **audio** stream — a radio set, a DJ set, a live show — uses this mechanism unchanged: the
segment blobs carry audio, and the VOD published on close (below) is an ordinary `VideoManifest`
whose `original` omits `width`/`height` (§24.4.2). `LiveManifest` carries no dimensions of its own,
so nothing in this section distinguishes the two cases.

- **The first record is the stream id.** The first `LiveManifest` (`seq = 0`) carries `title` and no
  `stream` field; its announce id *is* the stream id. Every later record carries `stream` = that id and
  `seq` = previous + 1. Comments and `LiveChat` reference the stream id as their `subject` (§24.6.1).
- **Every segment self-verifies.** Segments are ordinary §22 public blobs (2–6 s of media each);
  because each `LiveManifest` is a signed `pub_announce`, a viewer verifies each segment's
  `manifest_root` against the signed rolling manifest — live content has the **same integrity as VOD**,
  delayed by one manifest publication (vidmesh §004 §6, carried over).
- **`seq` gaps are tolerated.** Consumers order by `seq`, not arrival (partition posture); the feed's
  own `prev`-chain and signed `FeedHead` (§22.4) give the anti-rollback vidmesh's bare relay-`seq`
  could not.
- **`final: true` → publish a VOD.** After closing the stream, the creator SHOULD publish an ordinary
  `VideoManifest` (§24.4) whose `original` is the concatenation (or a proper re-encode) of the
  stream's segments — the durable VOD record, superseding the ephemeral live chain for archival
  purposes. `LiveChat` (`meta["live_chat"]`, a `pub_announce` with `subject` = stream id and non-empty
  `text` ≤ 2048 bytes) is ephemeral in spirit: a serving node MAY expire `live_chat` announces
  aggressively (a serve-policy choice, not a deletion — other holders and archives are unaffected).

## 24.11 Licensing

Licensing is a **generic-core** concern shared by every facet of this profile: the rules below govern
the media facet's `VideoManifest.license` (key 9) and the engineering-artifact facet's
`ArtifactMetadata.license` (key 7, §24.18.1) alike, and neither facet restates them.

Every artifact MUST carry a `license` field. There is no "no license" state in this profile: an
announce omitting `license` is malformed for the facet (a generic §22 node still stores/serves it —
§22 has no concept of licensing — but a facet-aware client/index SHOULD refuse to index it as usable
and SHOULD surface the omission to the publisher, VID-1 / CAD-1). The admissible value is **any valid
SPDX license expression** (SPDX License List, or a valid SPDX expression for combinations, e.g.
`"MIT OR Apache-2.0"`). The media facet **additionally** admits one of three **serving-consent tokens**
carried over from vidmesh (§004 §4), because its primary case is republication/serving consent rather
than reuse licensing:

| Value | Meaning |
|-------|---------|
| `all-rights-reserved` | No republication consent expressed |
| `mirror-freely` | Anyone may pin and serve the unmodified bytes |
| `endorsed-only` | Serving intended for PUB servers the creator endorses, named by an `endorse.gateway` announce (kind 80, Appendix A — **informative**). Like the notices of §24.13, this expresses the creator's preference and **obligates no one by protocol**: no PUB server is required to check it, and nothing enforces it. |

**Hardware, content and software licenses are all first-class SPDX.** Because the engineering-artifact
facet's primary motivating use case is open-hardware part sharing (ROADMAP), the CERN Open Hardware
Licence family is named explicitly alongside the usual software/content licenses; the list is
illustrative, not a closed enum, since `license` is free-text SPDX and any valid expression is admissible:

| Domain | Representative SPDX identifiers |
|--------|----------------------------------|
| Hardware (CERN OHL v2) | `CERN-OHL-S-2.0` (strong reciprocal), `CERN-OHL-W-2.0` (weak reciprocal), `CERN-OHL-P-2.0` (permissive) |
| Documentation / content | `CC0-1.0`, `CC-BY-4.0`, `CC-BY-SA-4.0` |
| Software (firmware, scripts, generator code shipped alongside an artifact) | `MIT`, `Apache-2.0`, `BSD-3-Clause`, `GPL-3.0-only` |

The field **enforces nothing** — it makes violations legible and gives compliant nodes something to honour
automatically (e.g. a node auto-pinning only `mirror-freely` content, §24.7 mirror). `license` is
metadata of the **whole revision** and is **immutable for that revision**: it rides inside the signed
announce, so it cannot be edited after publication any more than a manifest root can, and a **license
change is expressible only as a new revision** (`supersedes` + the new value, §24.7). The prior
revision's bytes remain published under their original terms forever (irrevocability, §22.7) — a
license change is forward-only and does not retroactively relicense what a holder already has. A `RightsClaim`
(`meta["claim"]`, converging vidmesh's `claim.license` kind 49) MAY assert a later licensing position
of the rights chain; interpreters present the latest position **with provenance, as a claim, never as
verified truth** (§24.8, §24.13).

## 24.12 Public-object HTTP endpoint usage

Nothing in this profile requires a mesh transport for a first deployment: its objects are ordinary §22
objects, served by §22's public-object HTTP endpoint (§22.5.1) — a plain-HTTPS surface, no protocol change.
The normative endpoint grammar is §22.5.1's; this is the profile's **generic-core** serving surface,
used identically by both facets (the engineering-artifact facet, §24.18, references it rather than
restating it), with segmented playback layered on the `chunk` surface for the media facet:

| Purpose | Endpoint (normative grammar: §22.5.1) |
|---------|----------------------------------------|
| Fetch a channel/publisher's feed | `GET /.well-known/dmtap-pub/feed/{pub}/head`, then `…/feed/{pub}/range?from=&to=` |
| Fetch one announce (video, comment, live, …) | `GET /.well-known/dmtap-pub/announce/{id}` — the signed `pub_announce`; its `meta[<key>]` embeds the profile object |
| Fetch a manifest (media/rendition/caption blob) | `GET /.well-known/dmtap-pub/manifest/{id}` |
| Fetch chunk bytes / a playback range | `GET /.well-known/dmtap-pub/chunk/{h}` — raw plaintext chunk bytes, self-verifying against `h` |
| *(optional, proposed §24.16)* verify one chunk out-of-order | `GET /.well-known/dmtap-pub/manifest/{id}/proof?chunk=i` — O(log n) Merkle inclusion proof |

A PUB server serving this surface needs **no video-specific code**: it stores and serves opaque
signed/content-addressed §22 objects. All schema interpretation — `VideoManifest`, renditions,
channels, comments, aggregates, HLS playlist synthesis — happens client-side or as server *product*
(§24.13), not as protocol. A first deployment is one plain-HTTPS PUB server with zero mesh (the HTTP test,
[`substrate/README.md § 4.2`](substrate/README.md)).

```mermaid
sequenceDiagram
  autonumber
  participant Client
  participant GW as PUB server (HTTPS)
  Client->>GW: GET feed head + range (a followed channel)
  GW-->>Client: FeedHead (signed) + FeedEntries — verify sig, walk prev-chain (§22.4)
  Client->>GW: GET announce (a VideoManifest from the feed)
  GW-->>Client: pub_announce (signed) — verify, parse meta["video"]
  Client->>GW: GET manifest for the chosen Rendition.blob
  GW-->>Client: PubManifest (chunk hash list)
  Client->>GW: GET chunk/range bytes — verify each against its hash (§22.2)
  Note over Client: renditions: verify derivation_sig + authorization (§24.4.4)<br/>segmented: verify ranges vs the signed Merkle root, ignore playlist as truth (§24.9)
```

## 24.13 Content moderation, privacy & security

**Moderation posture — edge-selected, no protocol takedown (both systems already agree).** This is the
load-bearing security-relevant design, and vidmesh (§009) and DMTAP-PUB (§22.6.2,
[`substrate/FEEDS.md § 7`](substrate/FEEDS.md)) hold it identically:

- **Selection is the moderation model.** A PUB server indexes and serves a *selection* of the substrate;
  non-serving is a first-class, instant, local operation. Nothing a PUB server does removes anything from
  the substrate — there is **no protocol-level takedown** (§22.6.2), because any mechanism strong
  enough to force one holder's removal is strong enough to censor the network. A publisher's video
  persists exactly as long as *some* holder serves it (availability ≠ durability, §22.9).
- **Serving public plaintext is opt-in and shifts liability.** Unlike a sealed-chunk relay (blind
  ciphertext), a video holder serves **plaintext it can read** (§22.6.1) — so serving MUST be an
  explicit operator choice (`pub-1`), and each holder applies its own serve policy, declining any
  object with `ERR_PUB_NOT_SERVED` (`0x090C`, a fetcher rotates to another holder). This is the cache/pin
  "not blind" rule ([`substrate/ROLES.md § 6`](substrate/ROLES.md)).
- **Compliance is expressed as subscribable claims, applied at the edge.** vidmesh's `notice.takedown`
  / `notice.counter` / `feed.takedown` (kinds 64–66) and its per-server compliance-feed subscription
  converge onto **ordinary `pub_announce`s carrying claim/notice metadata** (`meta["claim"]`,
  `meta["notice"]`, `meta["compliance_feed"]`): a structured legal notice is a *signed record*, a
  compliance feed is one identity's `seq`-ordered author feed of add/remove batches, and a PUB server
  *subscribes per its jurisdiction* and de-indexes matched subjects locally. A notice **obligates no
  one by protocol**; feeds are plural, opt-in, and auditable (an over-blocking feed loses subscribers
  to a competitor). Interpreters MUST present every claim/notice **with provenance, as an assertion,
  never as verified truth** (§24.8). CSAM handling, jurisdictional legal toolkits, and mandatory
  custody-exit are **reference-server / trademark-program obligations** (vidmesh §009 §4–§5), not
  kernel rules the protocol can enforce — carried over as such: they live in a conformant reference
  server, not in these profile bytes.

**Privacy posture (inherited from §22, unchanged).** Publishing under this profile is a **deliberate,
irrevocable** act of making media public; the CAS-confirmation the sealed model of §5.5 sacrifices
dedup to avoid is *accepted by design* here (§22.2.4). `VideoManifest`, every media/rendition/caption
blob, the channel, and every social object are **plaintext by construction**. This profile makes **no
confidentiality claim** — a publisher who needs a private video uses the sealed-file model of §5.5,
not this profile. For the rare encrypted-media case, vidmesh's `keygrant` (kind 97) and `encryption`
manifest field are explicitly **out of scope** for this public profile; they belong to the sealed path
(§5.5, §6.2), and mixing them into a public `VideoManifest` is a category error a conformant
client SHOULD refuse.

**Honest limits (from §22.9,** [`substrate/FEEDS.md § 8`](substrate/FEEDS.md)**):** irrevocability
(a published video cannot be recalled once any holder retains it); publisher metadata is public by
design (who published what, and when, is the verifiable fact); dedup reveals content equality across
publishers; availability is not durability (best-effort, opt-in serving); signing-key compromise scope
(a compromised operational key publishes under the identity until revoked, and what it published is
itself irrevocable — keep `IK` cold, §1.2a); and no read privacy — a holder sees which reader fetched
which public video, so a reader needing to hide *that they watched* must supply their own transport
anonymity.

## 24.14 Migration from vidmesh-format records

vidmesh independently reinvented §22 with incompatible bytes. A vidmesh record becomes a conforming
DMTAP-PUB object by the following **one-time, per-author** changes. Because vidmesh already chose the
*same cryptographic primitives* DMTAP requires at the v0 floor — Ed25519 (RFC 8032) and BLAKE3-256 —
convergence is never a re-choice of algorithm, and the identity layer costs nothing to move (item 7).
It is **not**, however, uniformly cheap or uniformly mechanical: producing the new bytes needs a fresh
signature only the author can make (items 1, 2, 4, 8), and migrating a media blob's content addressing
needs a genuine re-hash of the stored bytes, not just a re-framing of retained digests (item 4,
corrected — this was previously mis-stated as free; see the erratum in §24.17).

1. **Envelope → `pub_announce`.** A vidmesh record is a CBOR map with integer envelope keys 1–7
   (`kind`/`author`/`created_at`/`refs`/`body`/`sig_alg`/`sig`) signed as
   `Sign(k, "vidmesh:record:v1" ‖ id)` where `id = BLAKE3-256(det_cbor(keys 1..6))`. It becomes a §22
   **`pub_announce`** (kind `0x40`): the vidmesh `body` moves into `meta[<profile-key>]` (e.g.
   `meta["video"]` for a `manifest`), `author` becomes the announce's `pub` (root `IK`) + `signer`
   (operational key), and the object is signed under the **§22 announce preimage and its
   `"DMTAP-PUB-v0/..."` DS-tag** (§22.3, §18.9), not `"vidmesh:record:v1"`. The `announce_id` is
   `0x1e ‖ BLAKE3-256(det_cbor(pub_announce))` (§22.3).
2. **DS-tags.** Every signing/hashing context changes from a `"vidmesh:*"` prefix to the corresponding
   `"DMTAP-PUB-v0/*"` DS-tag (announce, feed, manifest) — **except** the rendition-derivation statement,
   which this profile deliberately keeps as a profile-scoped `"DMTAP-VID-v0/derivation"` context
   (§24.4.4), prefix-free against every core tag by construction *and* registered in §21.24e so that a
   future extension cannot allocate the same string. Note that the derivation *representative* is not
   simply the tag concatenated with a digest: it carries the `0x00` separator, the producer's `suite`
   byte on a composite suite, and the §18.1.5-prefixed digest (§24.4.4), so a vidmesh producer porting
   its transcode signer changes more than the tag string.
3. **Multihash prefix on blob ids.** vidmesh blob ids are **bare 32-byte BLAKE3** (`b3-256:<hex>`, no
   prefix) over the **whole blob's bytes** — a plain `BLAKE3-256(bytes)`, unrelated to the chunk tree of
   item 4. DMTAP content addresses carry the **multihash prefix `0x1e ‖`** (BLAKE3-256, §18.1.5). A
   vidmesh whole-blob id maps to `0x1e ‖ <same 32 bytes>` — a **one-byte prepend, no re-hash**, because
   both are BLAKE3-256. (This is the cleanest part of convergence: unlike the kerf-pub reference, which
   ships SHA-256 under prefix `0x12` and must migrate, vidmesh already matches the v0-REQUIRED BLAKE3
   prefix.) **Scope note:** this cheap transform applies to a whole-blob or record content reference used
   for cross-linking (e.g. the traceability address carried in `meta["vidmesh.record"]`, item 6) — it is
   **not** the value any of this profile's `hash`-typed blob fields need. `Media.blob`, `Rendition.blob`,
   `Caption.blob`, and `thumbnail` (§24.4.2, §24.4.1) all require a `manifest_root` — item 4's
   `PubManifest.id` — which is the expensive derivation below, not this one.
4. **Chunk root → §22 `PubManifest` root — an I/O-bound re-read, not a metadata recompute (corrected;
   erratum C-01, §24.17).** vidmesh's `Media.chunk_root` is a BLAKE3 chunk tree whose domain separation
   is folded **into the per-chunk hash itself**: the value vidmesh persists per chunk is
   `BLAKE3-256(0x00 ‖ chunk_i)` (kernel §8) — a tagged leaf digest, not a bare chunk hash. §22's
   `PubManifest.chunks` requires a **different pre-image at that same position**:
   `h_i = 0x1e ‖ BLAKE3-256(chunk_i)`, the *undecorated* plaintext-chunk hash (§22.2.2) — §22 folds its
   own `"DMTAP-PUB-v0/manifest"` DS-tag one level **up**, at the `leaf()`/`node()` tree step, not into
   `h_i` itself. **These are not the same value, and neither is derivable from the other**:
   `BLAKE3-256(0x00 ‖ chunk_i)` cannot be un-mixed into its `0x00` tag and a bare `BLAKE3-256(chunk_i)` —
   a hash with a domain tag folded into its input does not expose an untagged hash as a byproduct. Nor
   does a vidmesh record even retain a per-chunk list to fall back on: `Media.chunk_root` is the single
   32-byte tree **root**, not the ordered `h_0…h_{n-1}` list a `PubManifest` needs. **The chunk leaf
   hashes are therefore not identical, and re-derivation is not a tree recompute over retained digests —
   it requires re-reading every stored media byte and re-hashing each chunk from the plaintext** (the
   same 1 MiB chunk boundaries, but a fresh hash of the bytes, never a reuse of a stored value), exactly
   as if the file were being published for the first time. For a video platform this is the entire
   difference between a metadata migration (cost proportional to record count) and an I/O-bound one (cost
   proportional to total stored media bytes) — a migration plan MUST budget it as the latter. The
   separate `chunk_root` field disappears once migrated — the `PubManifest` *is* the chunk list and its
   root.
5. **FeedHead / anti-rollback adoption (the substantive gap).** vidmesh has **no signed feed head**: it
   orders records by a **relay-local, unauthenticated `seq` receipt counter** (§006 §2), which gives no
   per-author anti-rollback and no equivocation evidence — a relay can silently omit or reorder an
   author's records. DMTAP-PUB **mandates** a per-author **signed `FeedHead`** (`seq` + `tip` + a
   `prev`-chained `FeedEntry` log, §22.4). To converge, a vidmesh identity MUST **adopt a signed feed
   head**: publish each announce as a `FeedEntry` (strictly increasing `seq`, `prev` = the prior
   entry's content address) and sign the `FeedHead` over the tip (DS-tag `"DMTAP-PUB-v0/feed"`). Readers
   then get **anti-rollback** (reject a lower `seq`, `ERR_PUB_FEED_ROLLBACK` `0x0907`) and **fork/
   equivocation detection** (two entries at one `seq`, or a broken `prev`, is `ERR_PUB_FEED_CHAIN_BROKEN`
   `0x0908`, HALT_ALERT) that vidmesh's relay-`seq` model structurally cannot provide. This is the one
   place convergence *hardens* vidmesh rather than merely re-encoding it. (vidmesh's `anchor` kind 96 —
   external-timestamp Merkle batching — remains useful as a portable *ordering-evidence* layer over
   the signed feed and MAY be carried as `meta["anchor"]`, but it is not a substitute for the signed
   head.)
6. **`refs` → typed subjects.** vidmesh positional `refs` (`[ref_type, hash]`, type 0 = record, type 1
   = blob) become the named `subject`/`parent`/`channel`/`derived_from` fields of the profile schemas
   above (each a `hash` content address), so an interpreter no longer relies on ref *position*.
7. **Identity requires no migration (informative, and the one item that is free).** A vidmesh `Keypair`
   and a §22 `IdentityKey` are **both Ed25519 keys built from the same 32-byte RFC 8032 seed** —
   reinterpreting one as the other is a type relabeling, not a key derivation, and the public key is
   **bit-identical** on both sides. An existing vidmesh author therefore publishes §22 objects **under
   the identity they already hold**: no new keypair, no rotation, no re-provisioning of trust (any
   `DeviceCert` chain, delegate grant, or follower relationship rooted in that `IK` carries over
   unchanged). This is the one item on this list that costs nothing.
8. **Records cannot be migrated in bulk — re-signing needs the author's key (informative, corrected).**
   Items 1, 2, and 4 each change the signed pre-image — a new envelope, a new DS-tag, and (per item 4, in
   the general case) a new chunk-tree root. None of these is a transform *on the old signature bytes*:
   each produces a genuinely new object that only the author — or a device holding a valid, unrevoked
   `DeviceCert` chaining to that author's `IK` (§1.2) — can sign. A migration tool, PUB server, or archive
   that holds only the *plaintext* of a vidmesh record (which is everything a public vidmesh relay or
   mirror holds) **cannot** produce a valid `pub_announce` for it, because it does not hold the signing
   key. Concretely: **there is no automatic, bulk, third-party migration of existing vidmesh records to
   §22 bytes.** Migration to §22 is necessarily **per-author and consensual** — each identity re-publishes
   its own catalogue (item 7 makes this cheap on the key side; item 4 is what makes it expensive on the I/O
   side) — never a batch job an operator runs unilaterally over records it merely stores on someone
   else's behalf.

**History stays dual-format; a PUB server MUST NOT launder authorship through attestation.** Migration is
**prospective, not retroactive**: a vidmesh record published before an identity migrates remains a valid,
permanent, irrevocable vidmesh-format object — irrevocability (§22.7, §24.7) applies to whichever
substrate a given record actually was published under, and there is no in-place rewrite of history. A
reader's client MUST retain the ability to verify **both** formats for as long as pre-migration content is
served; "migrated" describes what an identity publishes going forward, never a bulk rewrite of what it
already published. A PUB server MAY choose to **re-attest** old vidmesh-format content it holds under its own
key as a `meta["attest"]`-style claim (§24.8) — "server G vouches this pre-migration record is
authentic" — but that is **server reputation, not authorship**, and MUST be rendered with a **visibly
weaker** verification indicator than an author-signed §22 object. Presenting a server attestation of an
un-migrated record behind the **same verification badge** as a direct author signature is a **security
misrepresentation**: it tells a viewer "the creator signed this" when what actually happened is "a third
party vouches for something the creator never signed under these bytes." This is not a hypothetical edge
case — it is the natural shape a "convenience" migration wrapper reaches for, and it recreates exactly the
authority confusion §22.9's honest-limits discipline exists to keep legible. A conformant client that
surfaces attestation provenance MUST distinguish the two cases visibly in its UI, not only in metadata a
user never inspects.

**Bundles.** vidmesh's `.vmsh` bundle (a magic-prefixed CBOR sequence of records + blobs with an `end`
count, §007) is a useful **partition-tolerant export** for DTN/sneakernet/archival. It carries over as
an informative packaging of DMTAP-PUB objects: a bundle of `pub_announce`s + `PubManifest`s + chunk
blobs, import-verified exactly as §22 verifies each object (signature + content address). It is
referenced by the `bundle` hint type (§24.4.5) and is not a normative part of this profile.

## 24.15 Conformance checklist (profile-level MUSTs)

| # | Requirement | Ref |
|---|-------------|-----|
| VID-1 | Every `VideoManifest` carries a `license` field (SPDX expression or a profile consent token) | §24.11 |
| VID-2 | `original` (key 5) is present and is the canonical rendition — a `Rendition` is never the artifact of record | §24.4.3 |
| VID-3 | Every `Rendition` carries `produced_by` (`ik-pub`), `suite` and a `derivation_sig` (`sig-val`) that verifies over the derivation statement under the representative that `suite` selects | §24.4.3, §24.4.4 |
| VID-4 | A rendition is treated as an *authorised* encoding only if `produced_by` **byte-for-byte equals** the manifest author's `pub` at the same suite, or byte-for-byte equals the `grantee` of an unrevoked, unexpired `rendition` delegate grant at the same suite | §24.3.1, §24.4.4, §24.4.6 |
| VID-5 | The derivation statement binds `derived_from`→`rendition.blob` + codec/width/height/bitrate, signed under the `"DMTAP-VID-v0/derivation"` DS-tag (registered §21.24e) by a device key whose `DeviceCert.ik` equals `produced_by` | §24.4.4, §21.24e |
| VID-6 | A video's `channel` reference resolves to an announce with the same `pub` as the video's author | §24.5 |
| VID-7 | `retracted = true` is always accompanied by `retract_reason` | §24.4.1 |
| VID-8 | Retraction/removal is expressed only as a successor `supersedes` announcement, never as deletion; a client MUST NOT imply deletion of prior bytes | §24.7 |
| VID-9 | A threaded `Comment` with a `parent` references the same `subject` as its parent | §24.6.1 |
| VID-10 | A `Reaction` is counted at most once per identity per subject — a later same-author reaction supersedes the earlier | §24.6.2 |
| VID-11 | Segmented playback verifies segment/range bytes against the signed rendition's Merkle root; the HLS/DASH playlist is unsigned serving output and is never treated as authoritative | §24.9 |
| VID-12 | Live streaming is gated behind the `vid-live-1` capability; a consumer lacking it treats its absence as a fact, not a fault | §24.10 |
| VID-13 | View/reaction/trending aggregates are presented as per-server claims, never as network-wide truth | §24.8 |
| VID-14 | No client treats any single index (search/recommendation/compliance) as authoritative over the signed announces/feeds it was derived from | §24.8, §24.13 |
| VID-15 | Encrypted-media fields (`keygrant`/`encryption`) do not appear in a public `VideoManifest` — the public profile makes no confidentiality claim | §24.13 |
| VID-16 | `width` and `height` are **both present or both absent** in every `Media` and `Rendition`; exactly one of them is malformed. Both absent ⇒ the encoding carries no video track (audio-only); a client infers the media kind from that and MUST NOT require a discriminator field | §24.4.2 |
| VID-17 | The derivation statement is **always a six-element array**; an absent `width`/`height` is encoded as CBOR `null` (`0xf6`) at its fixed position — never omitted, never `0`. A verifier reconstructs exactly that one encoding and MUST NOT accept a signature under any alternative reconstruction | §24.4.4 |
| VID-18 | An unrecognized `Caption.format` token (beyond `"vtt"`/`"srt"`/`"lrc"`) does not cause rejection of the manifest or of any other track — the track is skipped or handed to an external handler | §24.4.2 |
| VID-19 | Every field naming an identity — `Rendition.produced_by`, `Delegate.grantee`, `Follow.subject` — is an `ik-pub` accompanied by the `suite` governing its length, never a `hash`; identity equality is bytewise equality of those key bytes at one suite, and a verifier MUST NOT fetch an `Identity` in order to compare across suites | §24.3.1 |
| VID-20 | `derivation_sig` verifies against exactly one representative — `DS-tag ‖ 0x00 ‖ stmt_digest` on a single-component suite, `DS-tag ‖ 0x00 ‖ suite ‖ stmt_digest` on a composite one — where `stmt_digest` is the §18.1.5 **prefixed** digest of `stmt`, never a bare digest, and both components of a composite `sig-val` MUST verify. A verifier MUST NOT try the other form | §24.4.4, §18.1.6 |
| VID-21 | The embedded `meta["video"]` value is preserved as **bytes**: a relay/mirror/cache MUST carry it through byte-for-byte and MUST NOT decode-and-re-encode it. §18.1.1's deterministic-CBOR bans (no floats, tags, `undefined`, indefinite lengths, non-shortest form, duplicate or out-of-order keys) apply inside the embedded map at every depth, under recognised and unrecognized keys alike; a violation makes the *profile object* malformed, never the announce | §24.4 |

> **Conformance-suite note.** These MUSTs are catalogued as the `VIDEO` category in
> [`conformance/SUITE.md`](conformance/SUITE.md) / [`conformance/suite.json`](conformance/suite.json)
> as `construction-todo` stubs: like the engineering-artifact facet's `CAD` family (§24.18.10), the
> facet allocates no wire bytes of its own, so each case is a construction recipe over the CDDL above. VID-3/VID-5/VID-17/VID-20 (the
> derivation statement) are the exception — they *do* have a signable preimage, so they become
> byte-backed KATs once a fixed-input derivation vector is generated (the `"DMTAP-VID-v0/derivation"`
> context is deterministic and re-derivable with `blake3` + `ed25519`, no reference implementation
> required). When that vector is generated it MUST include an **audio-only** case, so the `0xf6`
> positions of VID-17 are pinned by bytes and not only by prose, and it MUST include a **composite
> suite (`0x02`)** case, so that the suite byte and the `0x1e` digest prefix of VID-20 are pinned by
> bytes rather than by prose — a single-component `0x01`-only vector cannot distinguish the two
> representatives and would freeze exactly the ambiguity VID-20 exists to close. Because that
> `0x02` case needs ML-DSA-65, which the reference core does not yet implement (§18.1.4's disclosed
> implementation-status gap), VID-20 stays `construction-todo` with a fully specified recipe until it
> does; the `0x01` half of the vector is generatable today and pins everything except the suite byte.
> VID-16 and VID-18 are catalogued as variants of the existing `DMTAP-VIDEO-02` (manifest/`Media`
> well-formedness) and `DMTAP-VIDMIG-01` (unrecognized-token forward compatibility) cases rather than
> as new case ids; VID-19 and VID-21 need cases of their own, since neither has an existing case whose
> construction they vary.

## 24.16 Additive proposals contributed upstream (informative)

Three pieces of vidmesh are genuinely superior to what §22 currently offers and are proposed as
**additive** contributions to the waist (not part of this profile's required bytes). They are written
up where they belong so any product — not only video — can use them:

1. **Chunk-tree range-proof endpoint** — an optional PUB-server endpoint returning an O(log n) Merkle
   inclusion proof for one chunk, so a client can fetch and verify a *middle* chunk of a large blob
   (seek into a 2 GB video) without downloading the whole chunk-hash list. §22 PUB servers currently serve
   whole chunks against the full manifest. Proposed in [`substrate/FEEDS.md`](substrate/FEEDS.md) and
   [`substrate/ROLES.md`](substrate/ROLES.md) as `GET …/manifest/{id}/proof?chunk=i`.
2. **Fetch-hint types registry** — the advisory `Hint` type registry of §24.4.5
   (`https`/`torrent-v2`/`relay-blob`/`bundle`) generalised as a waist-wide advisory registry in
   [`substrate/FEEDS.md`](substrate/FEEDS.md); hints are never authoritative over the content address.
3. **Rotation-log contest-window finality** — vidmesh's deterministic theft-recovery rule
   (recovery-authorised rotation beats a signing-key rotation until it is *final*, i.e. first-observed
   more than `contest_window` seconds ago, then a bytewise record-id tiebreak) is proposed as an
   informative note in [`substrate/IDENTITY.md`](substrate/IDENTITY.md) for the **zero-DNS / zero-KT
   key-name floor**, where no transparency log exists to anchor rotation ordering.

## 24.17 Change log — normative corrections

This document is pre-1.0 and is corrected in the open, in the same discipline
[`substrate/SYNC.md § 14`](substrate/SYNC.md) established: a defect found by an implementation is fixed
here **and recorded here**, never silently edited. Each entry states what changed, whether it changes
**wire bytes** (a KAT/vector consumer must be updated) or only **informative guidance** (migration
text, cost estimates, operational advice — no bytes on the wire change), and how it was found.

| # | Change | Class | Found by |
|---|--------|-------|----------|
| **C-01** | **§24.14 item 4 corrected: migrating a vidmesh blob to §22 is an I/O-bound re-read of the media bytes, not a metadata-only tree recompute.** The prior text claimed "the chunk leaf hashes are identical (bare-chunk BLAKE3); only the tree's domain separation differs, so re-derivation is a tree recompute over the existing chunk hashes, not a re-read of the media bytes." That is false: vidmesh's stored leaf value is `BLAKE3-256(0x00 ‖ chunk)` — the `0x00` domain tag is folded *into* the hash, not applied one level above it — while §22's `PubManifest.chunks` needs the *undecorated* `BLAKE3-256(chunk)` at that position. There is no operation that recovers the untagged hash from the tagged one, and vidmesh does not even retain a per-chunk list to begin with (`Media.chunk_root` is the single tree root). Migration therefore requires re-reading and re-hashing every stored media byte — for a video platform, the difference between a cost proportional to record count and a cost proportional to total stored bytes. Item 4's text is corrected to state this plainly; a new item 3 scope note prevents the adjacent (and correct) multihash-prefix transform from being mistaken for a solution to the same problem. | **INFORMATIVE — migration-cost/mechanism correction.** §24.14 is non-normative migration guidance, not the §22 wire format; no CDDL, DS-tag, or `PubManifest`/chunk-hash rule changes, and no conformance vector changes (§22's `pub_vectors.json` and this profile's construction-todo KATs are both unaffected). An implementation planning a vidmesh→§22 migration around the old text would under-provision I/O capacity and timeline by the gap between "recompute a tree" and "re-read every video file." | The vidmesh phase-1 `dmtap-core` adoption (`docs/DMTAP-CONVERGENCE.md`, commits `fa646f2`/`aa27b6a`/`09b5fc1`): its `dmtap_pub` bridge module computes both digests for the same chunk and asserts them unequal (`stored_vidmesh_leaf_is_not_the_pub_chunk_address`), with the §22 side checked byte-for-byte against the frozen `pub_manifest_single_chunk` vector — so the correction is corpus-anchored, not merely argued. |
| **C-02** | **§24.14 gains items 7–8 and a dual-format/attestation paragraph: no key migration is needed, but no bulk record migration is possible either, and old records stay dual-format.** The prior six items described byte-level transforms but never stated the two facts that most affect a migration plan's shape: (a) a vidmesh `Keypair` and a §22 `IdentityKey` are both Ed25519 seeds with a bit-identical public key, so an author needs **no new key** to publish under this profile; (b) precisely because items 1/2/4 change the signed pre-image, re-signing an *existing* record needs the **author's** key, which a PUB server, archive, or migration tool holding only plaintext never has for a self-custodied identity — so migration is **per-author and consensual, never a bulk operator-run rewrite**. The new closing paragraph adds the guidance this implies: pre-migration history remains valid and permanent in its original vidmesh bytes (irrevocability applies per-record, to whichever substrate it was actually published under, not retroactively), and a PUB server that chooses to re-attest old content under its own key MUST render that attestation with a visibly weaker verification badge than an author signature — presenting the two identically is a security misrepresentation (the creator did not sign these bytes; the server is vouching for them). | **Guidance addition, not a semantics change.** Items 1–6 and every §22/§24 normative rule are unchanged. This states a **new consequence** of already-normative rules (§1.2 key structure, §22.3.3 signer authorisation, §22.7/§24.7 irrevocability, §24.8's "attestation is worth exactly the attester's reputation") that §24.14 previously left the reader to infer, and one new expectation (the visibly-weaker attestation badge) stated here because its absence is a legible security failure mode a product would otherwise ship by default. | Same vidmesh phase-1 adoption: `keypair_to_identity_key`'s round-trip test (`identity_key_material_is_shared`) established (a); the same investigation, reasoning forward from what its bridge module can and cannot construct without the author's private key, established (b) and the attestation-wrapper risk this closing paragraph now warns against for the whole profile, not just one product's design doc. |
| **C-03** | **The profile is rescoped from video to time-based media (video *and* audio), and the derivation statement gains an unambiguous encoding for absent dimensions.** Audio was previously *unrepresentable*: `Media` (§24.4.2) and `Rendition` (§24.4.3) both required `width` (key 5) and `height` (key 6), so a song, a podcast episode or an audio-only rendition of a video could not be described at all. Five coupled changes: (a) keys 5 and 6 become OPTIONAL in both maps, with a **both-or-neither** rule — exactly one of them is malformed — and **absence is the audio-only signal** (VID-16); (b) the profile deliberately adds **no media-kind discriminator**, because it would duplicate a fact the dimensions already carry (and could therefore contradict it), because the correct granularity is per-encoding rather than per-work (a video work carries audio-only renditions), and because the both-or-neither rule already makes the inference total; (c) the **signed** derivation statement of §24.4.4 is pinned to a **fixed six-element array** with an absent dimension encoded as CBOR `null` (`0xf6`) at its fixed position — never omitted, never `0` (VID-17); (d) `Caption.format` gains `"lrc"` and the rule that an unrecognized token is skipped rather than fatal, so **lyrics are captions** and need no structure of their own (VID-18); (e) a **release (album/EP/single) is a `Playlist`**, which gains one OPTIONAL `cover` key (key 4) and an explicit statement that no album/playlist type token is defined — no protocol behaviour depends on the distinction, and a self-declared one would be an unverifiable claim (§24.8). The document title and §24.1 are rescoped to match; the file name, the section number, the `meta["video"]` key and the `VideoManifest` schema name are **kept** as historical spellings, because renaming any of them breaks already-published objects, every §24 cross-reference and the conformance catalogue for no protocol gain. | **NORMATIVE — CDDL shape + signed-preimage rule; strictly widening for the existing (video) case.** `Media`/`Rendition` keys 5/6 change from REQUIRED to OPTIONAL, `Playlist` gains OPTIONAL key 4, and §24.15 gains VID-16/VID-17/VID-18. **No previously valid object becomes invalid and no previously valid signature changes:** because `width`/`height` were REQUIRED before, every derivation statement that could have been produced under the previous revision has integers in both positions and is byte-identical under this one. The **DS-tag `"DMTAP-VID-v0/derivation"` is unchanged**, this profile still defines exactly one signed structure, and no core §21 registry entry, error code, DS-tag or wire object is touched. What *is* newly constrained: a producer that would have invented its own encoding for an absent dimension (a shortened array, a `0` sentinel) is now non-conformant, and an existing implementation that hard-requires keys 5/6 will reject audio-only media until it is updated. §24 has no byte-runnable vectors yet, so no committed KAT changes; the audio-only derivation vector required by §24.15's suite note is new work. | Reading §24 against the audio/music case it claims to cover in §24.1 ("video and time-based media") while §24.4.2/§24.4.3 hard-required pixel dimensions — the scope sentence and the schema had disagreed since the profile was written. The signed-preimage consequence (an OPTIONAL field inside a signed statement is a signature-ambiguity bug unless its absence has exactly one encoding) was found by following that change into §24.4.4 rather than stopping at the CDDL, and is resolved by reusing §18.1.1's existing preimage-`null` convention rather than inventing a second one. |
| **C-04** | **Identity references are retyped from `hash` to `ik-pub` + `suite`.** `Rendition.produced_by` (key 8), `Delegate.grantee` (key 1) and `Follow.subject` (key 1) were typed `hash`, which **cannot hold an identity key**: §18.1.7 fixes `hash = bytes .size (33..129)` (a 1-byte §18.1.5 prefix ‖ a digest), while an `ik-pub` is 32 B under suite `0x01` and 1 984 B under suite `0x02` (§18.2) — outside that range at both ends. Three encoders would have resolved the impossibility three ways (the raw key, `0x1e ‖ BLAKE3-256(IK)`, `0x1e ‖ IK`), yielding three `VideoManifest` byte strings, three `meta["video"]` values, three `announce_id`s for one logical work, and three mutually unresolvable follow graphs. It also left **VID-4 unimplementable**, since that MUST compares `produced_by` against the manifest's author and the author is `PubAnnounce.pub`, an `ik-pub`. All three fields are now `ik-pub`, each accompanied by a REQUIRED `suite` (`Rendition` key 11, `Delegate` key 5, `Follow` key 3) governing its §18.2 length; §24.3 gains an explicit carve-out from "`subject` is always a content address," and the new §24.3.1 states the typing, the bytewise same-suite comparison, the bounded cross-suite allowance (a verifier MAY use an `Identity` it already holds, MUST NOT fetch one), and why the digest alternative was rejected. **The digest alternative was rejected on the merits, not for convenience:** §18.9.17's key-name digest is taken over `Identity.iks[anchor_suite]` while a `DeviceCert.ik` is `Identity.iks[cert.suite]`, and §1.2.0 explicitly contemplates those differing — so the proposed "recompute the digest over `DeviceCert.ik` and compare" chaining rule fails whenever they do, forcing a verifier to fetch the producer's whole `Identity` and converting §22.3.3's zero-DNS offline check into an online one for an untrusted third party. Its claimed stability does not survive §18.9.17 either, which states that an anchor-suite migration changes every key-name digest with no key-rotation event to signal it. | **NORMATIVE — CDDL shape; not backward-compatible for any implementation that already emitted these fields.** Three maps change type at one key each and gain one REQUIRED key each. Because the previous typing was *unrepresentable*, no conformant object could have carried these fields correctly, so nothing valid is invalidated — but an implementation that chose one of the three ad-hoc encodings must re-emit its manifests, and re-emitting a manifest is a new `pub_announce` with a new `announce_id` (§22.3.4), not an edit. The **derivation statement `stmt` is unchanged** (it never contained `produced_by`). No core §21 error code, message kind or wire object is touched. | Adversarial audit of §24 against §18.1.7's `hash` production and §18.2's suite-governed lengths: the two audits agreed the field was unrepresentable and split on the remedy, and the split was resolved against §22.3.3's offline-verification guarantee and §18.9.17's own stability disclosure. |
| **C-05** | **The derivation signature gains its suite binding and its digest prefix; `derivation_sig` is retyped `bytes` → `sig-val`.** §24.4.4 asserted it reused "the core signing discipline unchanged … a DS-tag prefix with the `0x00` separator (§18.1.6)" and then showed `Sign(k, "DMTAP-VID-v0/derivation" ‖ 0x00 ‖ BLAKE3-256(stmt))` — which is neither of the two forms §18.1.6 defines. §18.1.6 requires, for suites `0x02`/`0x03`/`0x04`/`0x05`, the composite representative `M' = DS-tag ‖ 0x00 ‖ suite ‖ body` with the suite byte inside what both components sign ("this is what makes the composite non-separable"), and separately requires a signature taken over a **digest** to carry that digest in §18.1.5 prefixed form, never bare. `Rendition` had **no suite field at all**, so a verifier could not even select a representative. On suite `0x02` — the v0 REQUIRED originating suite — two conformant implementations therefore built different representatives and every hybrid rendition failed across the pair; worse, the published form was the **separable** one §18.1.6 exists to forbid, in which an Ed25519 component can be stripped from a composite signature and replayed as a standalone `0x01` signature. Fixed by: adding REQUIRED `Rendition.suite` (key 11, with an explicit statement of what it does and does not govern — notably that it does **not** displace `PubAnnounce.suite` as the §18.1.5 authority for the `hash`-typed fields); retyping key 9 to `sig-val` so §18.2's length governance applies; defining `stmt_digest = H_prefix ‖ H(stmt)` under the producer's suite; and stating **both** representatives in §24.4.4 in full, mirroring §18.1.6's wording, rather than delegating to it. The six-element `0xf6` fixed-arity rule of C-03 is untouched and layered under this change. | **NORMATIVE — signed representative; wire-visible.** `stmt` is byte-identical to C-03's; the bytes actually signed are not — every `derivation_sig` produced under the previous text is invalid under this one, on every suite, because the digest is now prefixed. This is a deliberate break of an unshippable construction: the previous form had no interoperable definition on the v0 originating suite and was cross-suite forgeable, so there is no correct implementation to preserve. `Rendition` gains one REQUIRED key and one retyped key. No core §21 error code, message kind or wire object is touched, and the profile still defines **exactly one** signed structure. §24 still has no committed KATs; §24.15's suite note now requires the derivation vector to include a composite-suite (`0x02`) case as well as an audio-only one. | Adversarial audit of §24.4.4 against §18.1.6 in full. The missing digest prefix was found by following the same section's second rule ("a signature over a digest MUST label the digest") rather than stopping at the composite-representative rule that prompted the review. |
| **C-06** | **The profile's DS-tag is registered in §21.24e; §24 stops claiming registration is unnecessary.** §24.4.4 held that the profile needed "no §21 registry entry — a profile-scoped DS-tag string is the only new byte, and it is domain-separated from every core context by construction." The construction argument is sound *and is now stated explicitly as a strength*: DS-tags are ASCII strings terminated by a single `0x00` (§18.1.6) and contain no `0x00` themselves, so the tag set is **prefix-free** and no signature under one tag can be reinterpreted under another. But that separates the tag only from tags that **already exist**. Separation from **future** tags is a property of the allocation process, not of the construction — the identical argument would be available to a future extension choosing the same string — and every prior extension registered its tags for exactly this reason (§21.24b, §21.24c, §21.24d). §24 was the first to opt out. §24.1, §24.4.4 and §24.14 item 2 are corrected, and §24.15's VID-5 now cites the registration. | **Registry/process, no bytes.** Nothing signed, hashed or encoded changes: the DS-tag string is the same string it always was, and every existing (or, per C-05, future) derivation signature is unaffected. What changes is that the identifier is reserved against future allocation, and that this document no longer asserts a guarantee a registry provides. **This entry depends on a companion change to §21** (a §21.24e block reserving `"DMTAP-VID-v0/derivation"` and recording `vid-live-1`); until that lands, §24's citation of §21.24e is a forward reference. | Adversarial audit noting that §21.24b/c/d each registered their DS-tags and §24 was the sole extension asserting it need not. |
| **C-07** | **§24.4's unknown-key rule is reconciled with §18.1.2, and "preserve" is defined as byte-retention.** §24.4 required clients to ignore and preserve unrecognized integer keys "because these are *unsigned* application maps," which contradicts the adjacent sentence of the same section — "the embedded bytes ride inside the signed announce body, so they are covered by the announce's signature" — and §18.1.2, which requires a decoder processing a **signed** object to reject unknown keys fail-closed and confines ignore-and-preserve to unsigned objects and the text-keyed `Headers.ext` map. The rule is kept, on a narrower and correct premise: what the announce signs is a `bytes` string, and the profile map inside it has **no signing preimage of its own**, so no announce preimage becomes ambiguous when a key inside that string is unrecognized. Three normative consequences are now stated rather than left to inference: preservation MUST be byte-retention of the whole embedded `bytes` value and MUST NOT be decode-and-re-encode (a re-encode is a different `announce_id` for an announce whose signature was over the old bytes); §18.1.1's float/tag/`undefined`/indefinite-length/duplicate-key bans **do** cross the `bytes` boundary and apply inside the embedded map under unrecognized keys as well as recognised ones, with a violation making the *profile object* malformed and never the announce; and an unrecognized `Rendition` key is outside the derivation statement by construction, so it can never alter what a `derivation_sig` attests. | **NORMATIVE — clarification; no encoding changes, one new prohibition.** No CDDL, DS-tag or preimage changes and no previously valid object becomes invalid. What is newly non-conformant is a relay/mirror/cache that "preserves" the embedded map by decoding and re-encoding it — a practice that was already producing broken signatures, now named as the fault — and an emitter that puts a float or a CBOR tag under an unrecognized key. §24.15 gains VID-21. | Adversarial audit of §24.4 against §18.1.2. The float/`bytes`-boundary question was raised as ambiguous and is resolved here in the direction that makes the byte-retention rule enforceable, since a float has no byte-stable decode/re-encode. |

**Standing rule.** A defect between this document and an implementation is resolved by deciding **which
side is right on the merits** and correcting the other in the open, exactly as [`substrate/SYNC.md`](substrate/SYNC.md)
§14 states it. §24.14 is informative migration guidance, so its corrections are Class INFORMATIVE by
default unless a fix also touches a CDDL shape, a DS-tag, or a `VID-*` conformance MUST — neither C-01
nor C-02 does; **C-03, C-04, C-05 and C-07 do**, and are classed NORMATIVE accordingly. **C-06 is
neither**: it changes no bytes and no MUST's content, only where an identifier is reserved and what
this document claims about that — a registry/process class, and the only entry so far whose completion
depends on a change to a file other than this one (§21.24e).

## 24.18 Engineering-artifact facet (CAD, PCB, assemblies)

The **engineering-artifact facet** applies the generic core (§24.1) to **engineering artifacts** — CAD
parts, PCBs, assemblies, schematics, drawings, and the datasets/documents around them — carried in a
`pub_announce`'s `meta` map under the facet key **`"artifact"`**. This facet was §23 (the *CAD / Artifact
Profile*) in earlier revisions and is unchanged in substance; §23 is retained as a gap pointing here.

**Shared concerns are inherited, not restated.** The scope framing (§24.1), the §22 relationship (§24.2),
the metadata-embedding + forward-compatibility + byte-preservation rules (§24.4), the canonical-source
principle (§24.4.3), licensing (§24.11), revision lineage / deprecation / forks (§24.7), the "indexes and
aggregates are derived, never authoritative" posture (§24.8), public-object HTTP serving (§24.12), and
privacy & security (§24.13) are the profile's **generic core** and govern this facet by reference. This
section adds only what is specific to engineering artifacts: the `ArtifactMetadata` schema (§24.18.1), the
kind/format/role registries (§24.18.2), units (§24.18.3), the CAD specialisation of the canonical-source
rule (§24.18.4), assemblies as Merkle DAGs (§24.18.5–§24.18.8), workshop conventions (§24.18.9), and the
facet conformance checklist (§24.18.10).

This facet defines one CBOR metadata schema, `ArtifactMetadata` (§24.18.1), carried inside a
`pub_announce`'s metadata (§22.3) under `meta["artifact"]`; and one CBOR structure schema,
`AssemblyStructure` (§24.18.7), for the parts-DAG of an assembly, itself published as an ordinary §22
public blob.

**This facet introduces zero new wire mechanisms, zero new crypto, and — unlike the media facet — no
profile-local signed structure** (§24.1): it allocates no message kind, no capability token, no DS-tag,
and no error block. Every byte it defines is either (a) unsigned application data that inherits its
authenticity from living inside a signed `pub_announce` (§22.3) or a content-addressed public blob (§22.2),
(b) a plain convention a facet-aware client applies when interpreting such data, or (c) a `track` reference
(`ref_kind = 2`, §24.18.6) — a mutable pointer whose referent is resolved by a third party (an index, or
the author feed itself) at fetch time rather than fixed at publication time. A `track` reference's
authenticity comes from neither (a) nor (b): it is not signed data living inside the referencing announce,
and it is not a content address, so there is no hash to check it against. It comes entirely from the
feed-resolution procedure required of a conformant client by §24.18.6, executed against a verified author
feed before any resolved target is trusted. This does not add a wire mechanism — the procedure is built
from §22.3.3 announce verification and §22.4 feed verification already in scope — but it is a third,
distinct authenticity source, named so the (a)/(b) framing is not read as exhaustive. A node implementing
only §22 with no facet awareness already stores, serves, and swarms every object this facet defines; it
simply does not parse `ArtifactMetadata`. Conformance to this facet is additive and orthogonal to §22/§21
conformance.

**Definitions.** An **artifact** is a versioned engineering design — a part, assembly, PCB, schematic,
drawing, dataset, or supporting document — published as one or more §22 public blobs referenced from a
`pub_announce` (§22.3) that carries an `ArtifactMetadata` map. A **revision** is one such announce. An
artifact's history is the `supersedes` chain (§22.3, §24.7) of its revisions, each independently signed by
the publishing identity.

### 24.18.1 `ArtifactMetadata`

Carried in a `pub_announce`'s `meta` map (§22.3.1) under the facet key **`"artifact"`**, as a `bytes` value
containing the deterministically encoded (§18.1.1, §18.1.2) `ArtifactMetadata` map below. The embedding,
forward-compatibility, and byte-preservation mechanics are the profile's **generic core rule** (§24.4)
applied with facet key `"artifact"` instead of `"video"`: `meta` stays a text-keyed `ext-value` map a
generic §22 reader parses unchanged (ignoring the unrecognized `"artifact"` key per §21.20), the embedded
bytes ride inside the signed announce body and are covered by its signature, "preserve" means
**byte-retention of the whole embedded `bytes` value, never decode-and-re-encode** (any round-trip yields a
new byte string — a broken signature and a different `announce_id`), and §18.1.1's deterministic-CBOR bans
(no floats, CBOR tags, `undefined`, NaN/Infinity, indefinite lengths, non-shortest-form arguments,
duplicate or out-of-order map keys) apply inside the embedded map at every depth, under recognised and
unrecognized keys alike — a violation makes a facet-aware client reject the *profile object*, never the
announce, and a generic §22 reader that never decodes the value is unaffected (§24.18.10, CAD-12). §24.4
states the full rule and its rationale once; it is not restated here.

**Forward compatibility (normative).** A facet-aware client MUST ignore unrecognized integer keys in
`ArtifactMetadata`, `ArtifactFormat`, `AssemblyStructure`, and `AssemblyChild`, MUST NOT treat their
presence as fatal, and MUST preserve them on re-serialize (as byte-retention, §24.4); keys **≥ 64** are
reserved for future revisions of this facet.

**Fail-closed rejection beyond ignore-and-preserve is not added here (facet-specific).** This facet's
stakes make the question worth asking directly: a dropped safety- or compliance-relevant flag (a
hypothetical `safety_recall` or `export_controlled` marker) risks a physical part being manufactured
against stale metadata. But the one case of this kind the facet actually defines already travels a
REQUIRED, non-ignorable channel: `deprecated`/`deprecation_reason` (keys 8/9) MUST be parsed and surfaced
by every conformant client unconditionally (CAD-7), expressed only as a successor announcement (CAD-8) — it
is not an extension key a client is free to not implement. A *future* safety-critical field that must force
an old client to refuse rather than silently proceed cannot be added as one more ignorable key under this
facet's forward-compat rule without contradicting it. The only mechanism this document family uses for "a
client without X MUST treat its absence as a fact, not silently proceed" is a registered capability token
gating the whole object — the pattern §24.10's `vid-live-1` establishes, registered in §21.24e. This facet
defines **no capability token**. Extending fail-closed rejection to a not-yet-defined safety field is
therefore a §21 capability registration this facet does not currently have — reported here, not invented: a
future revision needing mandatory-to-implement safety/compliance gating would register a token analogous to
`vid-live-1` (e.g. under a name like `cad-safety-1`) in §21's capability registry, alongside the
corresponding `ArtifactMetadata` field. Until that registration exists, ignore-and-preserve is this facet's
complete answer, and it is the correct one for every key this document currently defines.

```cddl
ArtifactMetadata = {
  1  => tstr,                 ; name           human-readable artifact name (UTF-8)
  2  => tstr,                 ; description    free-form description; MAY be empty
  3  => uint,                 ; artifact_kind  §24.18.2
  4  => [+ ArtifactFormat],   ; formats        at least one rendition, §24.18.4
  5  => Units,                ; units          explicit unit declaration, §24.18.3
  ? 6  => [* tstr],           ; tags           free-form; index-derived, never authoritative, §24.18.9
  7  => tstr,                 ; license        SPDX license expression, REQUIRED, §24.11
  ? 8  => bool,               ; deprecated     true iff this revision deprecates/yanks the artifact, §24.7
  ? 9  => tstr,               ; deprecation_reason  human reason; MUST be present iff deprecated = true
  ? 10 => hash,               ; derived_from   announce id of the ancestor this artifact forks from, §24.7
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `name` | 1 | `tstr` | MUST | Human-readable artifact name. UTF-8. Not unique — disambiguation is the publisher's identity + feed, not the name. |
| `description` | 2 | `tstr` | MUST (MAY be empty) | Free-form prose description. |
| `artifact_kind` | 3 | `uint` | MUST | One of the values in §24.18.2. An unrecognized value MUST NOT be treated as fatal by a generic index (it is preserved and surfaced as "unknown kind"), but a client that renders/BOM-walks artifacts MUST refuse to do so for a kind it does not implement. |
| `formats` | 4 | `[+ ArtifactFormat]` | MUST (≥ 1) | The artifact's renditions (§24.18.4). |
| `units` | 5 | `Units` | MUST | Explicit unit declaration (§24.18.3). |
| `tags` | 6 | `[* tstr]` | OPTIONAL | Free-form category/search tags. Purely advisory index input (§24.18.9); carries no protocol meaning. |
| `license` | 7 | `tstr` | MUST | SPDX license expression (§24.11). |
| `deprecated` | 8 | `bool` | OPTIONAL | Present and `true` iff this revision's purpose is to mark the artifact deprecated/yanked (§24.7). Absent ⇒ `false`. |
| `deprecation_reason` | 9 | `tstr` | MUST iff `deprecated = true` | Human-readable reason. A `deprecated = true` announce with this field absent is malformed for this facet. Because discarding a malformed deprecation would fail **open** on a safety signal, a conformant client MUST still honour the `deprecated` flag — surfacing the artifact as deprecated with its reason unavailable — rather than ignoring the announce, and a facet-aware index MUST flag it as malformed (§24.18.10, CAD-7). |
| `derived_from` | 10 | `hash` | OPTIONAL | Content address (announce id, §22.3) of the ancestor artifact/revision this one forks from (§24.7). Distinct from `supersedes`, which is same-identity revision history; `derived_from` is cross-identity provenance. |

### 24.18.2 Artifact-kind and format registries (profile-local)

These are conventions of this facet, not entries in the core §21 IANA registries — extending them is a
matter of revising this facet, not the core protocol.

| `artifact_kind` | Meaning |
|----------------:|---------|
| `1` | part — a single-body mechanical/physical component |
| `2` | assembly — a composition of parts/sub-assemblies (§24.18.5) |
| `3` | pcb — a printed-circuit-board design |
| `4` | schematic — an electrical/logical schematic |
| `5` | drawing — a 2D engineering drawing |
| `6` | dataset — non-CAD engineering data (simulation results, test data, material tables) |
| `7` | doc — supporting documentation |

| `format_id` | Meaning | Typical `role` |
|-------------:|---------|-----------------|
| `1` | STEP (AP242) | canonical-source (if no native file is published) or derived-rendition |
| `2` | native parametric source (vendor-specific feature-tree format) | canonical-source |
| `3` | glTF / mesh (tessellated geometry) | derived-rendition, always |
| `4` | ECAD (KiCad and equivalent PCB/schematic formats) | canonical-source (for `pcb`/`schematic` kinds) |
| `5` | PDF drawing | derived-rendition (typically), or canonical-source for a `drawing`-kind artifact authored directly as PDF |
| `6` | assembly-structure (the `AssemblyStructure` CBOR document, §24.18.7) | structure |
| `7` | opaque dataset/document blob (arbitrary bytes with no assumed CAD structure — simulation data, test logs, material tables, prose documents) | canonical-source (for `dataset`/`doc` kinds, §24.18.4) or derived-rendition |

| `role` | Meaning |
|-------:|---------|
| `1` | canonical-source — the authoritative rendition (§24.18.4) |
| `2` | derived-rendition — a convenience rendition generated from a source (§24.18.4) |
| `3` | structure — the assembly BOM graph (§24.18.5); applies only to `artifact_kind = assembly` |

### 24.18.3 Units (normative)

```cddl
Units = {
  1  => tstr,          ; length_unit   REQUIRED, explicit — no implied default
  ? 2  => tstr,         ; angle_unit    default "rad" if absent
  ? 3  => tstr,         ; mass_unit     OPTIONAL, for BOM mass properties
}
```

`length_unit` is an SI or SI-derived token (`"m"`, `"mm"`, `"um"`; non-SI tokens such as `"in"` MAY appear
but MUST be explicit). It **MUST always be present and MUST NOT be defaulted or inferred** by any producer
or consumer — unit ambiguity in interchanged engineering data is a well-documented, catastrophic-failure-class
bug, and this facet closes it structurally rather than by convention: an `ArtifactMetadata` with
`units.length_unit` absent is malformed for this facet, and a conformant client MUST refuse to interpret the
artifact's geometry (it MAY still display name/description/license) until the publisher corrects it in a
superseding revision (§24.7). `angle_unit` defaults to radians when absent; `mass_unit` is
informational — it declares the unit in which masses carried in the artifact's own format-native
payload are to be read. This facet defines no mass field and no mass aggregation: §24.18.8's BOM
walk multiplies `quantity` only.

### 24.18.4 `ArtifactFormat` and the canonical-source specialisation (normative)

This subsection specializes the profile's **generic canonical-source principle** (§24.4.3 — one canonical
source of record; a derived rendition is never authoritative; a lossy/derived form MUST NOT be the
canonical one) for CAD/engineering formats. The generic principle governs; the format-specific role
constraints below are what this facet adds.

```cddl
ArtifactFormat = {
  1  => uint,           ; format_id            §24.18.2
  2  => hash,           ; manifest_root        §22 public-blob manifest root for this rendition
  3  => uint,           ; role                 canonical-source(1) / derived-rendition(2) / structure(3)
  ? 4 => hash,           ; derived_from_format  manifest_root this rendition was generated from
  ? 5 => tstr,           ; format_version       free-form tool/variant string, e.g. "AP242 ED2", "KiCad 8.0"
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `format_id` | 1 | `uint` | MUST | Rendition format (§24.18.2). |
| `manifest_root` | 2 | `hash` | MUST | The §22 public-blob manifest root for this rendition's bytes. Fetched exactly as any §22 public manifest (§24.12); content-verified per §18.9.5-style chunk hashing, inherited unchanged from §22. |
| `role` | 3 | `uint` | MUST | `1` canonical-source, `2` derived-rendition, `3` structure (§24.18.2). |
| `derived_from_format` | 4 | `hash` | MUST iff `role = 2` | The `manifest_root` of the format entry this rendition was generated from — normally the canonical-source entry, or (for an assembly) the `structure` entry when the derived rendition is a composed/baked mesh of the whole assembly. Forms a shallow provenance pointer, not a chain a client is required to walk. |
| `format_version` | 5 | `tstr` | OPTIONAL | Free-form authoring-tool or format-variant string. Display/index hint only. |

**The canonical-source rule (load-bearing, normative).** The parametric source is the canonical artifact;
a derived mesh or tessellation is a convenience rendition, never the artifact of record. Concretely:

- `formats` MUST contain **at least one** entry.
- For `artifact_kind` other than `assembly` (§24.18.2), exactly one entry MUST carry `role = 1`
  (canonical-source), and that entry MUST be the artifact's native parametric/original-authoring format
  (`format_id = 2` native, or `4` ECAD for `pcb`/`schematic` kinds, or a directly-authored `format_id = 5`
  PDF for a `drawing`-kind artifact with no separate source). A STEP (AP242) entry (`format_id = 1`) MAY
  serve as canonical-source **only** when no native parametric source is published for that artifact (a pure
  interchange-only publication); where a native source exists, STEP MUST be `role = 2` (derived-rendition).
  For a `dataset` (`artifact_kind = 6`) or `doc` (`artifact_kind = 7`) artifact — which has no parametric
  source by nature — the canonical-source entry MAY be a `format_id = 7` (opaque dataset/document blob)
  entry: the opaque bytes *are* the artifact of record for these kinds.
- For `artifact_kind = assembly`, exactly one entry MUST carry `role = 3` (structure, `format_id = 6`,
  §24.18.7); a native assembly-authoring file, if published, MAY additionally carry `role = 1`. An assembly
  with no `structure` entry is malformed for this facet.
- **A `format_id = 3` (glTF/mesh) entry MUST always carry `role = 2`.** A mesh/tessellation MUST NOT be
  marked `canonical-source` under any circumstance — this is the facet's central integrity guarantee: a
  consumer that needs to re-derive dimensions, edit features, or verify tolerances can always find the
  source, never only a lossy tessellation of it.
- Every `role = 2` entry MUST carry `derived_from_format`, pointing at the `manifest_root` of the entry it
  was generated from (the canonical-source entry, or, for assemblies, optionally the structure entry). A
  client MAY follow this pointer to reach the canonical rendition; it MUST NOT assume a derived rendition is
  dimensionally authoritative in its absence.

### 24.18.5 Assemblies as Merkle DAGs of parts

An `assembly`-kind artifact's composition is a **content-addressed DAG** for its `pin` children: each `pin`
sub-part or sub-assembly is referenced by content address, so identical children dedup automatically
wherever they recur (within one assembly or across many), and BOM extraction over `pin` edges is an
ordinary tree/DAG walk over already-content-addressed, already-integrity-checked (§22.2) references. A
`track` child is a different kind of edge: it is **not** content-addressed — it names a `pub_announce` id
whose current head is resolved by the feed-based procedure defined in §24.18.6, which can legitimately yield
different bytes on different days. A BOM containing any `track` child MUST be labelled **non-reproducible** by
a conformant client (§24.18.6).

### 24.18.6 Reference modes: pin vs track (normative)

An assembly child references another artifact one of two ways:

| Mode | References | Resolves to |
|------|------------|-------------|
| **pin** | a `manifest_root` (a §22 public-blob manifest root) | exact bytes, forever — the same content address always names the same bytes |
| **track** | a `pub_announce` id | whatever the current head of that announce's `supersedes` chain is (§24.7), resolved at fetch time |

Both are `hash`-typed content addresses (§18.1.5-style multihash addressing, inherited from §22); they
differ in *what* they are addresses of.

**Resolution procedure for `track` references (normative).** `supersedes` (§24.7) points backward, from a
successor to what it replaces, so the current head can never be reached by following `supersedes` forward
from the referenced announce, and an index's bare answer to "what is the head of announce X?" is not by
itself verifiable. A conformant client resolving a `track` reference MUST do so as follows, and MUST treat
any index as a discovery hint only, never as an authority (§24.18.9):

1. Fetch the referenced `pub_announce` by content address and verify it per §22.3.3.
2. Read that announce's `pub`.
3. Fetch the author feed head for that `pub` and verify it per §22.4, including the anti-rollback watermark.
4. Walk the feed forward from the referenced announce, accepting a successor `A′` as the new head only if
   `A′.pub` equals the referenced announce's `pub` **and** `A′.supersedes` transitively reaches the
   referenced announce through a chain of announces that all share that same `pub`. A successor under any
   other `pub` MUST NOT be accepted, regardless of how it was obtained or how validly it verifies in
   isolation under §22.3.3.
5. If resolution cannot be completed against a verified feed under steps 1-4 — the feed is unreachable, its
   head fails §22.4 verification, or no forward path under a uniform `pub` exists — the client MUST fail
   that child. It MUST NOT substitute an index's answer, and MUST NOT silently omit the child from the BOM
   in place of failing.

**Tradeoffs (normative guidance).** `pin` gives an assembly a reproducible, exact BOM — re-resolving the
same structure at any later time yields byte-identical children, because a manifest root cannot change
meaning (§22's content addressing). This is correct for anything that must build/manufacture identically on
re-fetch — a released product BOM, an archival snapshot, a manufacturing hand-off. Its cost: a pinned child
never receives upstream fixes, and if every holder of that exact manifest stops serving it (§22 durability
is best-effort per holder, not a guarantee), the reference can become unresolvable even though the artifact
it came from is alive at a newer revision. `track` always resolves to the live head, so an assembly
automatically picks up upstream fixes without itself being re-published. Its cost: the effective BOM is
**not stable over time** — walking the same `AssemblyStructure` bytes on two different days can yield
different children, since tracking follows whatever the sub-part's publisher has since done, including
deprecating it (§24.7). A consumer needing a frozen record of "what this build actually used" MUST resolve
every `track` reference to a concrete revision at the time that matters and, if reproducibility is required
going forward, republish that resolution as `pin` references — tracking is a live view, not a durable
record. Neither mode is a default; a publisher chooses per child, and nothing requires uniformity within one
assembly.

### 24.18.7 `AssemblyStructure`

Published as an ordinary §22 public blob (its bytes are the content of a `manifest_root` named by an
`ArtifactFormat` entry with `role = 3`, `format_id = 6`, §24.18.4) — it carries no signature of its own; its
authenticity is exactly the authenticity of the manifest that names it (content addressing, §22.2), which is
in turn named from the signed `pub_announce`.

```cddl
AssemblyStructure = {
  1 => [+ AssemblyChild],   ; children   one or more sub-part/sub-assembly references
}

AssemblyChild = {
  1 => uint,             ; ref_kind    pin(1) / track(2), §24.18.6
  2 => hash,             ; ref         manifest_root (pin) or pub_announce id (track)
  3 => uint,             ; quantity    instance count of this child in the parent; MUST be >= 1
  ? 4 => bytes,          ; transform   OPTIONAL placement/orientation data
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `children` | 1 | `[+ AssemblyChild]` | MUST (≥ 1) | The assembly's direct children. An assembly with zero children is malformed for this facet (use a `part`-kind artifact instead). |
| `ref_kind` | 1 (`AssemblyChild`) | `uint` | MUST | `1` pin, `2` track (§24.18.6). |
| `ref` | 2 | `hash` | MUST | A `manifest_root` when `ref_kind = 1`, or a `pub_announce` id when `ref_kind = 2`. |
| `quantity` | 3 | `uint` | MUST | Number of instances of this child in the parent (e.g. `4` for four identical bolts). MUST be `>= 1`; a quantity of `0` is expressed by omitting the child, not by a zero count. |
| `transform` | 4 | `bytes` | OPTIONAL | Placement/orientation data (e.g. a transform matrix) positioning this instance within the parent. **Its byte format is explicitly out of scope for this facet** — left to a future geometry/kinematics profile layered the same way this one is layered on §22. A client that does not understand the transform encoding MAY still perform BOM extraction and dedup (§24.18.8), which do not depend on it. |

### 24.18.8 BOM extraction, dedup, and cycle rejection (normative)

BOM (bill-of-materials) extraction is a walk of the DAG rooted at an `assembly`-kind artifact's `structure`
entry: fetch the `AssemblyStructure`, resolve each child (`pin` directly to its manifest; `track` by
following `supersedes` to the current head, §24.7), and recurse into any child that is itself
`assembly`-kind.

- **Dedup composes automatically.** Because children are content-addressed (`pin`) or resolve to a
  content-addressed manifest (`track`, once resolved to a specific revision), two occurrences of the
  identical part — whether within one assembly or across many different assemblies published by anyone —
  collapse to the same reference during a walk. A BOM tool needs no explicit deduplication step beyond the
  ordinary "have I already visited this content address" check any DAG walk performs; this is the same
  property that makes §22 public-blob storage itself cross-user-deduplicating (§22.2), extended one layer up
  to the artifact graph.
- **Cycles MUST be rejected.** A `pin` reference cannot participate in a cycle — content addressing is
  acyclic by construction, since a manifest root is a hash of content computed after that content exists. A
  `track` reference **can** form a cycle across revisions (assembly A's live head tracks part B, and B's
  publisher — maliciously or by error — later publishes B as an assembly that tracks back to A). A
  conformant BOM-walking client MUST maintain the set of content-addresses (`pin`) and resolved-announce-ids
  at the visited revision (`track`) on the current path, and MUST treat re-encountering one on the same path
  as a fatal structural error for that subtree: abort the walk there, do not extract a BOM through it, and
  surface the cycle to the user. This is a client-side validation rule, not a protocol-level rejection at
  publish time — §22 has no mechanism to prevent a cycle, since `track` references resolve after the fact, at
  fetch time. The facet's integrity guarantee is that no conformant client silently produces a wrong or
  infinitely-recursing BOM, not that a cyclic publication is impossible.
- **Quantity multiplies along the path.** The effective quantity of a leaf part in a BOM is the product of
  `quantity` at every level of the path from the assembly root to that leaf (standard multi-level BOM
  semantics); a walker accumulates this product per distinct content address.

### 24.18.9 Workshop conventions

The "any node MAY recompute an index, and no computed index is authoritative over the signed announces it
was derived from" principle is the profile's **generic §24.8 rule**; it governs this facet's
category/search/workshop indexes and is not restated here. This subsection adds only the facet-specific
*workshop* naming and the kerf publish flow.

A **workshop** is purely client-side state: the set of author feeds (§22.4) a user follows. It is not a
protocol object, has no wire representation, and no node other than the user's own client(s) needs to know
its contents.

- **Category and search indexes are derived data.** Any node — the user's own client, a community index
  service, a search engine — can rebuild a browsable index of artifacts by crawling the feeds it knows about
  and reading each `pub_announce`'s `ArtifactMetadata` (`artifact_kind`, `tags`, `license`,
  `name`/`description`). **No index is authoritative** (§24.8): two indexes MAY disagree (different crawl
  coverage, staleness, tag-derivation heuristics) without either being "wrong" in a protocol sense — the
  ground truth is always the signed announces themselves, fetchable per §24.12, and any client MAY recompute
  an index from scratch.
- **Publishing to a workshop is publishing to your own feed.** There is no separate "workshop publish"
  operation: an artifact is published exactly as any §22 object is — sign and append a `pub_announce` to the
  publisher's own author feed (§22.4). "Adding it to a workshop" from a consumer's perspective is simply
  following that feed. A publisher MAY additionally notify one or more index services out-of-band (e.g. an
  HTTP ping asking a crawler to re-fetch sooner) — this is a convenience integration outside the DMTAP wire
  protocol, not a normative part of this facet, exactly as a website ping-submitting itself to a search
  engine is outside HTTP.

### 24.18.10 Conformance checklist (engineering-artifact facet MUSTs)

| # | Requirement | Ref |
|---|-------------|-----|
| CAD-1 | Every artifact `pub_announce`'s `ArtifactMetadata` carries a `license` field (SPDX expression) | §24.11 |
| CAD-2 | `formats` contains at least one entry | §24.18.4 |
| CAD-3 | Exactly one `formats` entry carries `role = canonical-source` (non-assembly kinds) or `role = structure` (assembly kind) | §24.18.4 |
| CAD-4 | No `format_id = gltf/mesh` entry ever carries `role = canonical-source` | §24.18.4 |
| CAD-5 | Every `role = derived-rendition` entry carries `derived_from_format` | §24.18.4 |
| CAD-6 | `units.length_unit` is present and explicit; a client MUST NOT default it | §24.18.3 |
| CAD-7 | `deprecated = true` is always accompanied by `deprecation_reason` | §24.18.1 |
| CAD-8 | Deprecation/yank is expressed only as a successor announcement, never as deletion | §24.7 |
| CAD-9 | Assembly children reference exclusively by `pin` (manifest root) or `track` (announce id) | §24.18.6 |
| CAD-10 | A BOM-walking client MUST detect and reject a cycle in an assembly's resolved DAG rather than recurse indefinitely or silently drop it | §24.18.8 |
| CAD-11 | No client treats any single index (category/search/workshop) as authoritative over the signed announces it was derived from | §24.18.9, §24.8 |
| CAD-12 | The embedded `meta["artifact"]` value is preserved as **bytes**: a relay/mirror/cache MUST carry it through byte-for-byte and MUST NOT decode-and-re-encode it. §18.1.1's deterministic-CBOR bans apply inside the embedded map at every depth, under recognised and unrecognized keys alike; a violation makes the *profile object* malformed, never the announce | §24.18.1, §24.4 |

> **Conformance-suite note.** The `format_id = 7` (opaque dataset/document blob) admissibility in
> §24.18.2/§24.18.4 widens the admissible canonical-source formats for `dataset`/`doc` kinds, which touches
> the semantics exercised by the CAD-2/CAD-3 cases in `conformance/SUITE.md` / `conformance/suite.json`. The
> `CAD` family there (`DMTAP-CAD-01`…`11`) is maintained by the conformance workstream and is **not** updated
> here; it requires a suite regeneration both to cover `format_id = 7` and to add a `DMTAP-CAD-12` case for
> the byte-retention MUST (CAD-12), whose checklist row is stated here to match the §24.18.1 citations but has
> no corresponding vector yet.

## Appendix A: vidmesh kind → profile object mapping (informative)

Non-normative crosswalk of vidmesh's 27 record kinds (§003) onto this profile. Kinds that converge to
core DMTAP mechanisms (identity, feeds) are noted as such; a few are reference-server obligations, not
profile bytes.

| vidmesh kind | id | Converges to |
|--------------|---:|--------------|
| `rotation` | 1 | core Identity — the identity's rotation log ([`substrate/IDENTITY.md`](substrate/IDENTITY.md), §1); genesis id = identity id |
| `profile` | 2 | a `pub_announce` carrying profile display data (name/avatar/payment); latest-wins by chain order |
| `delegate` | 3 | `Delegate` grant (§24.4.6) — authorises `rendition` producers |
| `manifest` | 16 | **`VideoManifest`** (§24.4) |
| `supersede` | 17 | §22 same-author `supersedes` (§24.7) |
| `retract` | 18 | deprecation-as-successor: `retracted = true` (§24.7) |
| `mirror` | 19 | `Mirror` serving-assertion over the cache/pin role (§24.7) |
| `similarity` | 20 | a near-duplicate *claim* (`meta["claim"]`); evidence, never truth (§24.8) — PUB servers MUST NOT auto-merge on it alone |
| `comment` | 32 | `Comment` (§24.6.1) |
| `reaction` | 33 | `Reaction` (§24.6.2) |
| `follow` | 34 | `Follow` (§24.6.3) — portable social graph |
| `playlist` | 35 | `Playlist` (§24.6.4) |
| `channel` | 36 | `Channel` (§24.5); the author feed is the channel-of-record |
| `claim.author` / `.license` / `.transfer` / `.dispute` | 48–51 | `RightsClaim` (`meta["claim"]`, §24.11); presented with provenance, never as truth |
| `notice.takedown` / `.counter` | 64–65 | `meta["notice"]` signed legal notices; obligate no one by protocol (§24.13) |
| `feed.takedown` | 66 | `meta["compliance_feed"]` — a `seq`-ordered author feed of add/remove batches; PUB servers subscribe per jurisdiction, de-index at the edge (§24.13) |
| `endorse.gateway` | 80 | `meta["endorse"]` — a `pub_announce` naming a creator's designated **PUB server** (`endorsed-only` licensing, §24.11). **The identifier keeps its historical spelling for wire compatibility** (vidmesh kind 80 converges on it); it names a public-object server, **never** the legacy-mail gateway role of §7 (§0.8) |
| `receipt` | 81 | `meta["receipt"]` — a signed payment statement (tip/superchat); rails prove settlement, not the protocol |
| `attest` | 82 | `meta["attest"]` — portable third-party reputation / aggregate tally (§24.8) |
| `anchor` | 96 | `meta["anchor"]` — external-timestamp ordering evidence over the signed feed (§24.14 note 5) |
| `keygrant` | 97 | **out of scope** for this public profile — belongs to the sealed path (§5.5, §24.13) |
| `live.manifest` | 112 | **`LiveManifest`** (§24.10), optional `vid-live-1` |
| `live.chat` | 113 | `LiveChat` (§24.10) — ephemeral, aggressively expirable |

## Appendix B: Mapping to the kerf Workshop (informative)

This appendix illustrates, non-normatively, how the motivating deployment of the engineering-artifact
facet (§24.18, ROADMAP) uses it. It does not define new protocol behaviour.

kerf's Workshop publish flow maps onto this facet as follows:

1. **Build a public manifest from project files.** kerf already content-addresses project files as Git LFS
   objects, keyed by SHA-256. DMTAP's hash-agility prefix (§18.1.5) is a multihash-style byte in front of
   the digest, but it is not a free per-object choice: §18.1.5's precedence rule requires every `hash` inside
   an object that carries a `suite` field to use that suite's prefix, and to **reject** the object
   (`ERR_HASH_ALG_MISMATCH`, `0x0127`) rather than honour the prefix in place of the suite. No v0 suite selects
   SHA2-256, and §22.2.2's `leaf()`/`node()` tree functions are defined only for BLAKE3. **Stated honestly**:
   a §22 public manifest that addresses its chunks under the SHA-256 prefix (`0x12`) using the LFS objects'
   existing digests directly is a **rejected object, not an interoperability surface** — every conformant
   peer rejects it, not merely peers implementing only BLAKE3. kerf therefore MUST re-hash project files
   under the v0-REQUIRED BLAKE3 prefix (`0x1e`) to build a §22 manifest; there is no shortcut through the
   existing LFS digests. The LFS SHA-256 digest MAY be retained out-of-band as a provenance note (e.g. in
   kerf's own project metadata) but MUST NOT appear as a `PubManifest.chunks` value — no new addressing
   scheme, no flag day, BLAKE3 from the start.
2. **Sign a `pub_announce`** carrying an `ArtifactMetadata` map (§24.18.1) built from the kerf project's
   metadata: name, description, `artifact_kind` from the project type, the format list (native kerf/OCCT
   source as canonical, STEP/glTF exports as derived renditions, §24.18.4), units, and the project's declared
   SPDX license (§24.11).
3. **Append to the author feed** (§22.4) — the publishing identity's own feed; there is no separate "kerf
   server feed."
4. **PUB servers serve over plain HTTPS** (§24.12) — kerf's hosted server is **one server among equals**,
   exactly as any self-hosted or third-party DMTAP-PUB server; it holds no special protocol role. This
   preserves the clean seam already established for kerf's architecture: kerf cloud is billing + provisioning
   + fleet, never a required intermediary for the artifact itself to exist or be fetched.
