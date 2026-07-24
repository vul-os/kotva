# Adoption status — what each product implements today

> **Status:** informative, descriptive, non-normative. This is a **snapshot** (2026-07-20) of what each
> product's code actually does, read directly from each repository — not a conformance certification and
> not a roadmap commitment. It fixes no wire bytes and defines nothing; [`IDENTITY.md`](IDENTITY.md),
> [`FEEDS.md`](FEEDS.md), [`SYNC.md`](SYNC.md), and [`ROLES.md`](ROLES.md) remain the sole normative
> statements of what each capability *is*. Where this document says a product is behind or diverges from
> them, the capability document is right and this document is describing a gap, not a disagreement. See
> [`BINDINGS.md`](BINDINGS.md) for the engineering plan to close these gaps by binding to one shared core
> rather than by five more independent rewrites.

**Status legend** (one per cell): **to-spec** = speaks the capability's actual bytes/wire, verified or
directly inspected; **independent** = implements the *function* with its own bytes — not interoperable with
another to-spec implementation of the same capability; **partial** = a real but incomplete attempt (a
stub, a subset of the algebra, a missing sub-role); **minimal** = a narrow building block exists (e.g. a
bare keypair) but not enough of the capability to call it adopted; **n/a** = not attempted, and there is no
particular reason it should be, given what the product is; **not built** = the capability is meaningfully
relevant to the product but nothing exists yet.

## 1. The matrix

| Product | Identity ① | Feeds & Blobs ② | Sync ③ | Roles ④ | Wake ⑤ |
|---|---|---|---|---|---|
| **envoir** (`/Users/pc/code/vulos/envoir`) | to-spec | to-spec | to-spec | partial | not built |
| **vulos** (`/Users/pc/code/vulos/vulos`) | independent (×2) | independent | independent (×2) | independent | independent** |
| **vulos-relay** (`/Users/pc/code/vulos/vulos-relay`) | minimal | to-spec | not built | independent | not built |
| **ofisi** (`/Users/pc/code/vulos/ofisi`) | minimal | n/a | **partial**† | independent | n/a |
| **flowstock** (`/Users/pc/code/vulos/flowstock`) | minimal | n/a | independent (partial algebra) | n/a | n/a |
| **vidmesh** (`/Users/pc/code/vulos/vidmesh`) | independent*** | independent (founder-gated convergence) | n/a | partial | not built |
| **whatsacc** (`/Users/pc/code/whatsacc`) | minimal | n/a | n/a | n/a | n/a |
| **kerf-pub** (`/Users/pc/code/exo/kerf/packages/kerf-pub`) | minimal | to-spec | n/a | to-spec (cache/pin only) | independent |

† **The substrate's first real product adoption.** ofisi's **Sheets** grid now runs on the shared
`dmtap-sync` engine (vendored as `dmtap-sync-wasm`) — the same compiled implementation, not a
reimplementation — **behind a build-time flag (`VITE_SUBSTRATE_SYNC`), off by default**. Docs and
Whiteboard remain Yjs *by design*, and Slides is blocked on a substrate gap the adoption itself found.
See §2 for the accurate breakdown; the adoption is what surfaced [`SYNC.md`](SYNC.md) §14 corrections
**C-08**, **C-09** and **C-10**.

\** Uses the correct open standard (Web Push/VAPID) but the payload carries a title/body, so it is not the
content-free wake the spec requires — see §2.

\*** vidmesh's own key-rotation-with-recovery-precedence design was independently built, then adopted
*into* [`IDENTITY.md § 5.1`](IDENTITY.md#51-informative--contest-window-finality-for-the-zero-dns--zero-kt-floor)
as an informative note — convergent by influence, not by wire compatibility.

---

## 2. Per-product detail

### envoir — `/Users/pc/code/vulos/envoir`

The reference core (see [`BINDINGS.md`](BINDINGS.md) for the full crate layout).

- **Identity — to-spec.** `dmtap-core::identity` implements `IdentityKey`/`Identity`/`DeviceCert`/
  `RecoveryPolicy`/`KeyRotation`/`MoveRecord` directly; `keyname` implements the 8-word key-name; `kt`
  implements the RFC 6962 transparency objects. This crate *is* one of the documents' own grounding
  references.
- **Feeds & Blobs — to-spec, now vector-verified.** `dmtap-core::pubobj` implements
  `PubManifest`/`PubAnnounce`/`FeedEntry`/`FeedHead`/`verify_feed_chain`/`check_anti_rollback`.
  `conformance-runner` now merges `dmtap/conformance/vectors/pub_vectors.json` into its run (`pub_vectors_path()`
  in `crates/conformance-runner/src/main.rs`) and, as re-run for this survey, reproduces 20 of the 21 PUB
  vectors (the 21st, `DMTAP-PUB-21`, is `suite.json`'s own `manual-attestation` case — no bytes for a
  runner to recompute, an honest skip, not a failure) — this closes the previous asterisk.
  `conformance/vectors/pub_vectors.json`'s `generated_by` metadata string (and its generator,
  `gen_pub_vectors.py`) previously described the Rust core as not implementing the PUB extension yet;
  that was corrected in `7ad7d4f` to say what is actually true — the bytes are produced by the script
  from the spec text, never by the crate, so the vectors remain an INDEPENDENT check on the
  implementation rather than a restatement of it. No stale artifact remains.
- **Sync — to-spec.** `dmtap-clustersync` (§5.6 single-owner profile: `Cluster`/`Replica`/`ClusterState`/
  `OrSet`/`LwwMap`/`DeathReg`/`Journal`/`range_fingerprint`) and `dmtap-sync` (the substrate's multi-author
  six-kind algebra, COSE-signed ops) are the crates [`SYNC.md`](SYNC.md) grounds itself in.
- **Roles — partial.** `dmtap-p2p` gives real libp2p implementations of announce/resolve (Kademlia),
  signaling (DCUtR), and circuit relay (Circuit Relay v2) — to-spec for those three sub-roles. Mailbox and
  a servable cache/pin surface were not confirmed to exist in this survey (`node/src/pubserve.rs` exists
  and its name suggests a pub-serving path, but its wire shape was not verified). **What would close the
  gap:** confirm or build the mailbox role and verify `pubserve.rs` actually serves the
  `/.well-known/dmtap-pub/*` surface per [`FEEDS.md § 5.1`](FEEDS.md#51-the-public-object-http-endpoint-2251).
- **Wake — not built.** No VAPID/Web Push/UnifiedPush code found anywhere in the workspace. **What would
  close the gap:** implement `PushSubscription`/`WakePing` per [`ROLES.md § 8`](ROLES.md#8-wake--content-free-sender-blind-push-capability-⑤-profile-of-49) — envoir has no wake path today, so this is greenfield, not a rewrite.

### vulos — `/Users/pc/code/vulos/vulos`

The main suite (control plane, OS, apps). Two largely disconnected identity/sync stories live in this one
repo — the ordinary product auth path, and a separately-designed peering subsystem that is far closer to
the substrate's shape without speaking its bytes.

- **Identity — independent, two layers.** (1) The primary auth path is session/OAuth-based, not
  keypair-rooted, though device-level Ed25519 + management-CA `DeviceCert`-like certs exist for offline
  first-boot claiming (`INTEG-SEC-01`) — own format, not the DMTAP wire. (2) `backend/services/peering/
  identity.go`/`identity_lifecycle.go` implements a real Ed25519 keypair identity (`vula:ed25519:<pubkey>`)
  with rotation/revocation/recovery and TOFU/anchor trust — conceptually a close cousin of §1/§3, wire-
  incompatible (own cert format, not COSE/DMTAP CBOR). **What would move it:** re-encode the peering
  identity objects as DMTAP `Identity`/`DeviceCert` CBOR and adopt the 8-word key-name as the durable
  handle instead of the bespoke `vula:` scheme.
- **Feeds & Blobs — independent.** `backend/services/peering/feeds.go` implements a real signed,
  hash-chained (`prev_hash`/`content_hash`), append-only feed with access-level gating — structurally a
  cousin of `FeedHead`/`FeedEntry`, but its own JSON wire, no content-addressed chunking, no §22 manifest.
  **What would move it:** adopt `PubManifest`/`PubAnnounce`/`FeedHead`/`FeedEntry` CDDL and BLAKE3
  DS-tagged addressing in place of the bespoke JSON chain.
- **Sync — independent, two unrelated engines.** (1) `internal/multiinstance/appsync.go` +
  `internal/fabric/fabric.go`: syncs exactly one table (`app_registry`) — LWW on version/timestamp + OR-set
  on installed flag — over **LAN-only** mDNS-discovered JSON changesets with a shared-secret auth header.
  No HLC, no version-vector/Merkle reconciliation, no RGA/tree. (2) `services/peering/collab.go`: real-time
  document collaboration via **Yjs**, a third-party CRDT unrelated to DMTAP's op algebra, relayed as opaque
  binary over WebSocket. Neither is DMTAP-SYNC; they are not even the same engine as each other. **What
  would move fabric:** widen it past a single table and swap its custom changeset format for
  `dmtap-sync` ops (see [`BINDINGS.md § 6`](BINDINGS.md#6-migration-path-per-product-optional-per-capability-nothing-forced)
  for the pure-Go binding tradeoff this repo specifically carries).
- **Roles — independent.** `peering/relay.go` is a genuine Ed25519-signed deposit/pickup/ack mailbox with
  TTL — structurally the mailbox role, own auth-header wire format (`Vula-Relay <id>.<ts>.<sig>`), not
  `LocationRecord`/DMTAP's mailbox wire. `peering/discovery.go` is a plain REST directory lookup against
  vulos.org, not key-addressed announce/resolve. No circuit-relay/signaling in this repo (that lives in
  vulos-relay). **What would move it:** re-key `discovery.go` to resolve by `IK` instead of an account
  directory lookup, and re-wire `relay.go`'s auth header as a `DeviceCert`-chained signature.
- **Wake — independent, and not content-free.** `internal/webpush/webpush.go` (`PUSH-CELL-01`) is real,
  sovereign, RFC 8291-compliant Web Push/VAPID — the box holds its own VAPID keys, SSRF-guarded, correctly
  using an open standard. But the payload (`Payload{Title, Body, Tag, Source, URL}`) carries a
  human-visible title/body — it is a **push notification**, not the spec's content-free wake-and-fetch
  hint. This is a legitimate, different design goal (rendering a notification vs. silently triggering a
  sync), not a bug, but it is not what [`ROLES.md § 8`](ROLES.md#8-wake--content-free-sender-blind-push-capability-⑤-profile-of-49)
  defines as Wake. **What would move it:** add a second, genuinely content-free `WakePing` path
  alongside the existing notification push, rather than repurposing the notification payload.

### vulos-relay — `/Users/pc/code/vulos/vulos-relay`

- **Identity — minimal.** The commercial tunnel core is bearer-token/`AccountID`-based, not key-addressed
  at all. The separate rendezvous subsystem (below) does Ed25519-sign its writes, but that is Roles usage,
  not a general Identity capability (no `DeviceCert`, no naming ladder). **What would move it:** adopt
  `IK`/`DeviceCert` as the rendezvous subsystem's identity rather than a bare signing key.
- **Feeds & Blobs — to-spec.** `tunnel/pubcache/*` is a literal §22 implementation: BLAKE3 domain-separated
  Merkle addressing (`"DMTAP-PUB-v0/manifest"`), served at `/.well-known/dmtap-pub/{announce,manifest,
  chunk,feed}` with correct immutable-vs-mutable caching. This is the most spec-faithful code found in
  either vulos repo. It has since also picked up the OPTIONAL [`FEEDS.md § 5.3`](FEEDS.md#53-optional--chunk-tree-range-proofs-additive-proposal)
  chunk-tree range-proof endpoint (`tunnel/pubcache/proof.go`, flag-gated behind
  `-pubcache-serve-proofs`) — encode/decode/generate/verify for `[chunk_index, [sibling_hashes…]]`, correctly
  taking `nChunks` out-of-band from the manifest header rather than the proof response (the precision
  [`FEEDS.md § 5.3`](FEEDS.md#53-optional--chunk-tree-range-proofs-additive-proposal) itself was just
  tightened to require, per this survey — see `docs/PUBCACHE.md` §5 in this repo, which independently
  reached the same conclusion). Nothing to close here.
- **Sync — not built.** No CRDT/HLC code found in this repo.
- **Roles — independent.** `tunnel/rendezvous/*` implements key-addressed announce/resolve/signal/mailbox +
  ICE, Ed25519-signed, content-blind, standalone from the OS repo — structurally faithful to the role
  definitions in [`ROLES.md`](ROLES.md) but its own wire protocol, not literal `LocationRecord` CBOR. The
  commercial tunnel core itself (bearer-token WSS+yamux) is a separate product surface, not a spec role.
  **What would move it:** re-encode `rendezvous`'s records as DMTAP `LocationRecord`/mailbox objects; the
  role *shape* is already right, only the bytes differ.
- **Wake — not built.** No VAPID/webpush code found in this repo. (Unrelated honesty gap, noted for
  completeness: SNI-passthrough mail is also still not built here, per the 2026-07-17 audit — this affects
  none of the five substrate capabilities and is not part of this matrix.)

### ofisi — `/Users/pc/code/vulos/ofisi`

- **Identity — minimal.** No per-user keypair is used as sync/session identity; the P2P room model is
  **capability-by-shared-key** (an invite link embeds `{roomId, roomKey, cap}`, HKDF-derived AES-GCM +
  HMAC) — "link holders," explicitly not per-identity, by the code's own doc comment. Ed25519 exists
  elsewhere (`backend/signing/crypto.go`, sharelink signing) but for content-sealing, unrelated to sync
  identity.
- **Feeds & Blobs — n/a.** No content-addressed public object or publish mechanism exists or is implied by
  what ofisi is (a live collaborative editor).
- **Sync — partial (and the substrate's first real adoption anywhere).** This cell is **per-surface**, and
  the honest summary is that one of four editors is on-spec behind a flag while the rest are unchanged.
  - **Sheets — to-spec, flag-gated, OFF by default.** `src/lib/crdt/substrateGrid.js` replaces the
    hand-rolled LWW map in `grid.js` with the **shared `dmtap-sync` engine**, vendored as
    `dmtap-sync-wasm` (`third_party/dmtap-sync-wasm/`) — the *same compiled implementation* a Rust server
    runs, not a JS reimplementation of it. The mapping is [`SYNC.md`](SYNC.md) §4.4 directly: namespace
    `sheet`, one LWW register per cell at `target = cell:<r>,<c>`, `field = "v"`, resolved by the §3 HLC.
    Canonical op bytes are the durable artifact; ofisi's existing JSON update-log and fabric frames carry
    them base64-wrapped as transport only. Convergence is asserted against the engine's own §6.1 **state
    root**, not against a rendered projection.
    - Selected by **`VITE_SUBSTRATE_SYNC` at build time**, default **off**; with the flag off not one byte
      of the substrate loads and `grid.js` behaves exactly as before. The gating is *correct and not
      timidity*: the two engines are each internally convergent but do not share a total order
      (`grid.js` compares `(lamport, replicaId)` and ignores wall-clock; the substrate compares a full
      HLC), so for two concurrent writes to one cell they can pick different winners. Every replica in a
      deployment must run the same path — which a build-time flag guarantees and a gradual rollout would
      not. Load failure falls back to `grid.js` rather than leaving a grid that records nothing.
    - **One real gap, named by the code itself:** ops go in via `ingest_ambient_authenticated`, the §5.6
      path for ops whose authenticity was established out of band. ofisi's grid ops are unsigned (they
      ride an authenticated fabric room / the server's update log), so this is the honest mapping rather
      than a downgrade — and it *is* a hole on a multi-author untrusted transport. Closing it means
      wiring per-device COSE signing (§4.1); that work is not done.
  - **Docs and Whiteboard — Yjs, by design, not by inertia.** Genuine **Yjs** (`yjs`, `y-prosemirror`)
    with its own binary update/state-vector wire, base64-wrapped in `{y:1, u:"..."}` — no CBOR, no COSE,
    no HLC total order. Rich-text with a live ProseMirror binding is not something [`SYNC.md`](SYNC.md)
    §4.7's RGA displaces cheaply, and Yjs is load-bearing through the whole editor stack. The realistic
    move here remains [`BINDINGS.md § 6`](BINDINGS.md#6-migration-path-per-product-optional-per-capability-nothing-forced)'s:
    wrap Yjs's opaque updates in a COSE-signed envelope, adopting the authenticity layer and not the
    algebra.
  - **Slides — blocked, and it is a substrate gap, not an ofisi one.** `src/lib/crdt/__tests__/substrateTree.mapping.test.js`
    measures the boundary by execution rather than asserting it: the **structural** half maps cleanly onto
    §4.8 `tree-move` (ofisi's fractional `ordKey`s carried unchanged as the ordering key, reorder = a
    second move, concurrent moves converging to one identical tree and one identical state root), and
    slide **deletion** maps correctly onto §4.5's death certificate because a deleted slide is never
    revived. The **content** half is what blocked: a slide object is nested JSON, and §4.1's value type as
    written could not express it. That is now [`SYNC.md`](SYNC.md) §14 **C-08** — the section named
    §18.3.6's `ext-value` but described a strictly narrower type — so the block is being lifted in the
    specification rather than worked around in the product. The remaining work is then an ordinary port
    of `tree.js`, which is a rewrite of a working editor's core and deliberately not attempted yet.
  - **What this adoption gave back.** Three [`SYNC.md`](SYNC.md) §14 corrections came out of it, all of a
    kind an independent reimplementation would not have found, because they are only visible to something
    that arrives with *its own data and its own storage shape*: **C-08** (the `ext-value` narrowing above),
    **C-09** (the snapshot *body* is a compacted **op set**, not `ObservableState` — ofisi built exactly
    that, one op per key, because the engine exposes no state-import entry point, and asked whether it was
    a workaround or the design; it is the design, and §6.1.2 now says so), and **C-10** (§4.10's
    death-certificate-vs-LWW selection guidance — ofisi first mapped "clear cell" onto §4.5, which would
    have silently swallowed every subsequent edit to a cleared cell, caught it, and then chose §4.5
    *correctly* for slide deletion in the same investigation).
- **Roles — independent.** A vendored `@vulos/relay-client` (`FabricClient`) provides signaling, rendezvous,
  WebRTC circuit-fallback relay, and presence — conceptually close to the Roles substrate, but its own
  SDK/protocol, not DMTAP key-addressed announce/resolve.
- **Wake — n/a.** No VAPID/web-push/UnifiedPush code found; nothing in ofisi's design implies it needs one
  independent of whatever the host product (Workspace/Meet) already provides.

### flowstock — `/Users/pc/code/vulos/flowstock`

- **Identity — minimal.** `backend/internal/store/identity.go` gives every node a per-node Ed25519 keypair
  used to sign op batches, but there is no naming ladder above it (no DNS, no KT, no `DeviceCert`, no
  8-word key-name) — a bare signing key, not the Identity capability.
- **Feeds & Blobs — n/a.** Not attempted; nothing in flowstock's design (inventory sync) implies it.
- **Sync — independent, partial algebra.** This is the repo [`SYNC.md`](SYNC.md) itself grounds its wire
  *shape* in, and it is a real, working stateless sync protocol: `GET/POST /api/sync/{vector,pull,ops}`,
  an HLC (`"{ms:013d}-{counter:04x}-{node}"`) matching the spec's grounding note exactly, a per-author
  `MAX(hlc)` version vector. But it is **not** byte-conformant: bodies are **JSON**, not deterministic
  CBOR; signatures are a **plain hex Ed25519 signature over `json.Marshal(ops)`**, not a `COSE_Sign1`
  envelope; transport auth is **bearer-secret-primary** with key-based auth as an optional add-on the
  code's own comments call "a documented next step, not forced here." It supports **2 of the 6** CRDT
  kinds — whole-row LWW and an insert-only/set-union ledger (explicitly *not* a PN-counter, by the schema's
  own comment) — with no OR-Set, death-certificate, RGA, or movable-tree. **What would move it:** switch
  the wire body to deterministic CBOR with `COSE_Sign1` envelopes, make key-based auth the only path
  (retire the bearer-secret fallback), and add the four missing CRDT kinds — see
  [`BINDINGS.md § 6`](BINDINGS.md#6-migration-path-per-product-optional-per-capability-nothing-forced) for
  the binding path that would deliver all of this as a swap-in rather than a rewrite.
- **Roles — n/a.** Not attempted; flowstock has no reachability/relay/mailbox problem of its own (it syncs
  over a network it doesn't need to traverse NAT for in the same sense a P2P messaging product does).
- **Wake — n/a.** No push/wake mechanism, and nothing in flowstock's design implies it needs one.

### vidmesh — `/Users/pc/code/vulos/vidmesh`

A substantial, independently-specified Rust protocol (its own `spec/000-007*.md`) that is *convergent in
design* with the substrate in several places, and explicitly aware of it.

- **Identity — independent (design already adopted upstream).** Real Ed25519 keypair identity plus a
  rotation log with recovery-over-signing contest-window finality (`crates/vidmesh-kernel/src/identity.rs`,
  its own `spec/002` §4). No `DeviceCert` chain, DNS binding, key transparency, or 8-word name. Notably,
  this exact fork-resolution design was independently built here *first* and then adopted into
  [`IDENTITY.md § 5.1`](IDENTITY.md#51-informative--contest-window-finality-for-the-zero-dns--zero-kt-floor)
  as an informative note — convergent by influence on the spec, not by wire compatibility with it.
- **Feeds & Blobs — independent, convergence founder-gated.** Hash is **BLAKE3-256**, matching the spec's
  algorithm choice — but the raw blob hash and record id carry **no DS-tag and no multihash prefix**
  (unlike DMTAP-PUB's `0x1e`-prefixed, `"DMTAP-PUB-v0/manifest"`-tagged tree), and the Merkle split rule
  differs from RFC 6962. A `Hint` type exists (`crates/vidmesh-kernel/src/kinds/content.rs:32`) but as a
  bare `Hint(u64, String)` tuple, not the 4-variant `https`/`torrent-v2`/`relay-blob`/`bundle` enum the
  substrate's advisory hint registry defines (the enumeration exists only in vidmesh's own prose spec, not
  its types). A `GET /blob/{id}/proof?chunk=i` endpoint returns a CBOR Merkle audit path — same idea as
  [`FEEDS.md § 5.3`](FEEDS.md#53-optional--chunk-tree-range-proofs-additive-proposal), own encoder and own
  array shape. Content is signed, but through one universal `Record` envelope, not a dedicated
  `PubManifest`/`PubAnnounce` — and critically, **there is no Feeds primitive at all**: no monotonic `seq`,
  no `prev` hash-chain, no fork detection (vidmesh's own docs note this as a gap). A full decision document
  exists in-repo (`docs/DMTAP-CONVERGENCE.md`, mirrored at `apps/site/docs/dmtap-convergence.md`)
  recommending re-basing vidmesh's video layer as the [§24 Video/Media profile](../24-video-profile.md);
  `DECISIONS.md` marks this **founder-gated** on three preconditions (founder confirms direction, §24 is
  targetable, envoir's §22 Rust implementation is a consumable dependency) — nothing has been actioned yet.
  **What would move it:** per [`BINDINGS.md § 6`](BINDINGS.md#6-migration-path-per-product-optional-per-capability-nothing-forced),
  fold in the DS-tag + multihash prefix, switch to the RFC-6962 split, adopt `PubManifest`/`PubAnnounce`/
  `FeedHead`/`FeedEntry`, and serve at `/.well-known/dmtap-pub/*` — all Rust-to-Rust, no cross-language
  binding needed, since this would be a direct `dmtap-core` dependency.
- **Sync — n/a.** No CRDT/HLC/vector-clock code or dependency anywhere; "sync" here is flood-fill gossip
  dedup over WebSocket, a different problem (event propagation, not convergent state) than
  [`SYNC.md`](SYNC.md) addresses.
- **Roles — partial.** Relay is a full, working crate. Cache/pin is explicitly marked
  `SCAFFOLD(phase-8)` and non-functional in the code itself
  (`crates/vidmesh-node/src/pinning.rs`). Mailbox has zero hits anywhere in the repo. **What would move
  it:** finish the scaffolded cache/pin path and build a mailbox role, or fold both into the §24
  convergence work if/when it proceeds.
- **Wake — not built.** No wake/webpush/APNs/FCM code anywhere in the repo.

### whatsacc — `/Users/pc/code/whatsacc`

(Note: not under `/Users/pc/code/vulos/` as originally expected — it lives at `/Users/pc/code/whatsacc`. A
similarly-named `/Users/pc/code/exo/whatsacc-mono` is an unrelated project.) A mature chat-driven physical
gate access-control product (Go gateway + controller agent + Tauri app). Its own `ARCHITECTURE.md` §8 marks
a DMTAP channel adapter as **"Sketch only — no seam interface, no wire format, no schedule."** Its authors'
own assessment is the right one to record here: this product has essentially no decentralised-substrate
surface today, by design, not by oversight.

- **Identity — minimal.** Real per-device Ed25519 keypairs (`controller/internal/identity/identity.go`,
  `gateway/internal/keys/keys.go`) with TOFU gateway-key pinning — a real building block, but no
  `DeviceCert`, DNS binding, transparency, or naming ladder above it.
- **Feeds & Blobs — n/a.** Its "audit log" is now a **SHA-256 `prev`-hash chain with append-only
  triggers and a verify endpoint** — tamper-evident in the same shape as a §22 feed's `prev` chain, though
  it is a private operational log, not a content-addressed public feed, and there's no reason a gate-access
  log needs to be one. (Earlier snapshots of this document described it as a plain, unsigned, unchained SQL
  table; that is stale.)
- **Sync — n/a.** It has a well-built Ed25519+JCS signed-command envelope with nonce/replay protection
  (`gateway/internal/keys/envelope.go`) — adjacent in spirit to a COSE-signed op, but it is point-to-point
  RPC (open a gate, revoke a badge), not CRDT state convergence. There is no shared state to converge here.
- **Roles — n/a.** Its "relay" is a GPIO hardware relay (the physical gate latch) and its "mdns" is plain
  LAN discovery — name collisions with DMTAP terms, not related implementations.
- **Wake — n/a.** Notifications ride third-party WhatsApp/Slack push, which is out of scope for a
  DMTAP-native wake mechanism and not a gap this product needs closed.

### kerf-pub — `/Users/pc/code/exo/kerf/packages/kerf-pub`

The named reference implementation of §22 ([`FEEDS.md § 6`](FEEDS.md#6-reference-implementation--kerf-pub-proves-the-http-test)).
Its actual current state is **better than that document's text**, which had gone stale.

- **Identity — minimal.** A local Ed25519 keypair signs announces/heads, but `signer == pub` is
  hard-enforced everywhere (`identity.py`, `objects.py`) — no `DeviceCert` chain support yet, so it is not
  adopting the Identity capability beyond a bare signer key.
- **Feeds & Blobs — to-spec.** The wire encoding is genuine deterministic **CBOR** matching the CDDL key
  numbers exactly (not JSON), and all five `/.well-known/dmtap-pub/*` endpoints exist in `router.py`. The
  hash scheme has **moved to BLAKE3-256** (`0x1e`) as the sole write path — `SHA2-256` (`0x12`) is kept
  only as a legacy read path for pre-cutover objects, selected by the prefix byte each address already
  carries — with a dependency-free BLAKE3 fallback (`blake3_pure.py`, ~1700x slower than the compiled
  wheel, measured, so it keeps a node verifying rather than serving) when the compiled wheel isn't
  available. This migration was **uncommitted at the time `FEEDS.md § 6` was first written; it has since
  been committed and pushed** (`f23c8576` migrates the hashing + replays all 15 vendored §22 vectors
  through the ordinary public API, `cedcddf9` documents the conformance status and corrects the fallback
  cost estimate) — kerf-pub's `main` now passes **15 of 15** shared `pub_vectors.json` cases with no
  skips, and `main`/`origin/main` are in sync. **`DeviceCert` chains remain unimplemented, deliberately,
  as the stricter of the two arms §22.3.3 step 4 permits**: kerf-pub hard-enforces `signer == pub` rather
  than accepting a `DeviceCert` chain, because a chain-verification path that checked the certificate's
  signature but not its revocation status (§1.5) would be a **fail-open** — accepting a revoked signer as
  though it were still authorised. `signer == pub` has no revocation surface to get wrong, so it is
  correctly the safer of the two conformant postures, not an outstanding defect. **What would close the
  remaining gap:** implement `DeviceCert` chain verification **with** §1.5 revocation checking (not
  without it) per [`FEEDS.md § 4.2`](FEEDS.md#42-the-pubannounce-kind-0x40-223) step 4 — a strict
  superset of the current behaviour, never a loosening of it.
  **Fixed-width `seq`/`ts` decode guards — closed 2026-07-21.** kerf-pub compared feed `seq`
  **numerically** (Python `int`), so it never had the lexical-vs-numeric ordering bug that the
  flowstock HLC counter had — but it decoded `seq`, `ts`, `size` and `chunk_sz` with a bare
  `int(m[k])`, admitting **negative** values (CBOR major type 1) and, for `chunk_sz`, values
  outside `u32`. Verified reachable: `FeedEntry.from_cbor` accepted `seq = -1` and
  `FeedHead.from_cbor` accepted `seq = -5`, both spec-illegal against the `u64` of §22.4.1. The
  values were caught downstream *incidentally* (the tip lookup in `_walk_and_verify` misses, so
  the chain walk raises `0x0908`) rather than by any range check, so the safety depended on an
  unrelated lookup and would not have survived a refactor. Now enforced at the decode boundary by
  a shared `_require_uint(value, what, bits, err)` helper, with the boundary value `2⁶⁴−1` still
  accepted — the guard rejects *outside* `u64`, not *at* it. This is the cross-capability
  ordered-domain invariant of [`FEEDS.md § 4.3`](FEEDS.md#43-anti-rollback-and-equivocation-2242),
  the Feeds twin of [`SYNC.md § 3`](SYNC.md)'s fixed-width HLC rule.
- **Sync — n/a.** Not attempted; kerf-pub is a Feeds/Blobs (+ Wake) server, not a state-sync engine.
  The ordered-domain invariant above is therefore **not** inherited via Sync — it binds kerf-pub
  directly as a §22 Feeds adopter, which is exactly why it is stated in `FEEDS.md` rather than
  left as a Sync-specific footnote.
- **Roles — to-spec, cache/pin only.** The `/.well-known/dmtap-pub/*` endpoint itself **is** the cache/pin
  role ([`ROLES.md § 6`](ROLES.md#6-cache--pin--serve-content-addressed-objects-profile-of-225-55)),
  correctly implemented (immutable `Cache-Control`/`ETag` on the four content-addressed endpoints). No
  other role (announce/resolve, signaling, circuit relay, mailbox) is attempted, and none is implied by
  what kerf-pub is.
- **Wake — independent.** kerf-pub ships its own Wake extension (VAPID keys, `router.py`'s
  `wake-key`/`subscribe` routes) — a real, working use of the correct open standard (Web Push/VAPID), but
  with its own subscription/registration endpoint shapes rather than the `PushSubscription`/`WakePing`
  object forms [`ROLES.md § 8.1`](ROLES.md#81-the-two-objects-profile-of-491-18556) defines. **What would
  move it:** adopt the `IK`-signed `PushSubscription` and content-free, RFC-8291-sealed `WakePing` object
  shapes precisely, in place of its own subscribe endpoint's payload.

---

## 3. What this matrix says about the strategic goal

The premise this survey was run to test — "~5 hand-rolled sync engines, one shared implementation should
retire them" — holds up, with more nuance than the premise implied:

- **Sync is genuinely fragmented**, and each fragment is a real, working, independently-correct design:
  flowstock's HLC+oplog (the spec's own wire-shape grounding, yet not byte-conformant itself), ofisi's Yjs,
  vulos's fabric LWW/OR-set (LAN-only, one table) and its separate Yjs-based collab path. No two of these
  four can talk to each other today. **The first fragment has now actually been retired** rather than
  planned away: ofisi's hand-rolled Sheets grid CRDT — the fifth such engine — runs on the shared
  `dmtap-sync` core behind a flag, which is the premise's first real evidence. It also shows what the
  retirement costs and returns: the cost is a build-time flag (two engines cannot share a deployment,
  because they do not share a total order), and the return is three specification corrections
  ([`SYNC.md`](SYNC.md) §14 C-08…C-10) that no amount of reimplementing the document from the document
  would have surfaced. This is exactly the gap [`BINDINGS.md`](BINDINGS.md) is written to
  close — not by asking four teams to rewrite four times, but by giving Sync one compiled core and four
  thin bindings.
- **Feeds & Blobs is closer to converged than expected.** vulos-relay's `pubcache`, kerf-pub, and envoir's
  `dmtap-core::pubobj` are now all to-spec, independently and vector-verified. The gap here is not
  fragmentation so much as reach: three products have it, most don't need it, and vidmesh's convergence
  is a real, drafted, founder-gated plan rather than an open question.
- **Roles is the most consistently "right shape, wrong bytes" capability** — vulos's peering relay,
  vulos-relay's rendezvous, and ofisi's `FabricClient` are all structurally faithful, independent
  reinventions of announce/resolve/mailbox/relay. This is the capability where a shared binding would
  probably require the least behavioural change per product, only a wire-format swap.
- **Wake is the least attempted and the most subtly wrong where it exists.** vulos's cellpush is a correct,
  sovereign VAPID implementation for the wrong job (it wants to render a notification, not fire a
  content-free hint) — a good reminder that "uses the right open standard" and "implements this capability"
  are different claims, worth keeping distinct in future audits.
- **The ordered-domain invariant is a measured defect class, not a hypothetical.** A follow-up audit of
  every adopter against [`FEEDS.md § 4.3`](FEEDS.md#43-anti-rollback-and-equivocation-2242) found it
  **five times, in five languages, by five different mechanisms** — each network-reachable, each invisible
  to that repo's own tests because the local engine agreed with itself:

  | Repo | Language | Mechanism | Fixed |
  |------|----------|-----------|-------|
  | kerf-pub | Python | bare `int()` decode accepted a **negative `seq`** | exo/kerf `66ea6e33` |
  | vulos-relay | Go | `PubManifest.size`/`chunk_sz` presence-checked but **never value-decoded** — a negative `size` or over-`u32` `chunk_sz` passed as "verified" and would be re-served, while a `u64`/`u32`-typed peer cannot parse it at all (the cross-engine well-formedness gap §4.3 names) | `d0f7b3a` |
  | ofisi | JavaScript | **`NaN` comparison** — `parseInt` on a malformed counter yields `NaN`, and `NaN < x`, `x < NaN`, `NaN >= x` are *all* false, so the obvious comparator returns "not less than" and the caller applies the op; one hostile peer could overwrite any cell | `0b3fd70` |
  | flowstock | Go | two entry points **re-implemented the HLC string format** with a bare `Sprintf`, bypassing the width guard | `9e431a3` |
  | vidmesh | Rust | `contest_window as i64` silently reinterprets any `u64 ≥ 2^63` as **negative**, making a finality check trivially true and instantly finalizing a stolen-key rotation | `ad04112` |

  The JavaScript case was a genuinely new variant and is now named in the invariant itself
  ([`FEEDS.md § 4.3`](FEEDS.md#43-anti-rollback-and-equivocation-2242) item 3): width and sign were
  already covered, `NaN` was not. **whatsacc was audited and found clean of this class** (random-unique
  nonces rather than ordered ones; IAT/EXP decoded into signed `int64` with fail-closed bounds checks at
  every arithmetic edge; no key-rotation counter at all — TOFU-replace only; its audit log orders by
  SQLite rowid, never by client input). The general lesson for future audits: a monotonic counter is a
  **cross-engine** contract, so the place to look is every decode boundary, in every language, not the
  comparison itself.
