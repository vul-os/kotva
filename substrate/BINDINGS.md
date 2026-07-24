# One core, many surfaces — the bindings plan

> **Status:** informative planning document, not normative. It fixes no new wire bytes and defines no
> new object, capability token, or error code — [`IDENTITY.md`](IDENTITY.md), [`FEEDS.md`](FEEDS.md), and
> [`SYNC.md`](SYNC.md) remain the sole normative source for what a conformant implementation must do. This
> document states the *engineering* plan for getting from "five products, five hand-rolled
> implementations" to "one compiled core, many thin surfaces" — which crate is the core, which language
> surfaces consume it and how, and what it honestly costs. Where this document and a substrate capability
> document disagree about bytes or behaviour, the capability document governs; this document only governs
> which binary produces those bytes.

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are used here in their plain
RFC 2119 sense, but nothing below is a conformance requirement on third parties — it is Vulos's own
engineering plan for its own products. An independent implementation remains free to implement the
substrate documents in any language, from scratch, per [`README.md § 6`](README.md#6-reference-implementations-informative-non-normative).

---

## 1. Why this document exists

The suite currently has **~5 hand-rolled sync/identity/feed implementations**, surveyed in
[`ADOPTION.md`](ADOPTION.md): flowstock's Go HLC+oplog engine, ofisi's Yjs CRDT, vulos's fabric
LWW/OR-set and its separate peering identity/feed/mailbox stack, vidmesh's independent content-addressing
and rotation log, and kerf-pub's from-scratch Python re-implementation of §22. Each is a good-faith,
independently-correct design for its own product. None of them can talk to each other, because "converge
deterministically" and "verify a signature" are exactly the kind of logic that silently diverges the
moment two implementations encode a float, sort a map, or fold a hash differently — the whole reason
[`SYNC.md § 2.2`](SYNC.md#22-determinism-is-the-contract) makes canonical encoding a MUST.

The fix is not "rewrite five products in Rust." It is: **write the algebra once, in one audited crate,
and give every product a thin binding to that same compiled artifact** — so that byte-identical behaviour
is a property of the toolchain, not of five teams independently re-reading a spec correctly. This is the
same logic that made SQLite, BoringSSL, and libsodium single-core-many-language projects rather than
five-language-five-implementations projects: the hard, security-relevant part gets built once and pinned
down by conformance vectors; every language gets a binding, not a rewrite.

---

## 2. The core: envoir's Rust crates (as they exist today)

`/Users/pc/code/vulos/envoir` is a Cargo workspace. The substrate-relevant crates, read directly from the
workspace (not assumed):

| Crate | Path | Capability | Notes |
|---|---|---|---|
| `dmtap-core` | `crates/dmtap-core` | **Identity** (①), **Feeds & Blobs** (②) | `identity` module: `IdentityKey`, `Identity`, `DeviceCert`, `RecoveryPolicy`, `KeyRotation`, `MoveRecord`. `keyname` module: the 8-word key-name encode/verify. `kt` module: RFC 6962 key-transparency objects. `pubobj` module: `PubManifest`, `PubAnnounce`, `FeedEntry`, `FeedHead`, `verify_feed_chain`, `check_anti_rollback`. Also `cbor`, `id::ContentId`, `capability::CapabilityToken`, `mote`, `cad`, `pq` (X-Wing hybrid). Deps: `ed25519-dalek`, `x25519-dalek`, `hpke`, `blake3`, `ciborium`, `hkdf`, `sha2`, `chacha20poly1305`, `ml-dsa` — pure-Rust crypto, no tokio, no filesystem, no native TLS. |
| `dmtap-clustersync` | `crates/dmtap-clustersync` | **Sync** (③) — §5.6 single-owner profile | `Cluster`, `Replica`, `ClusterState`, `OrSet`, `LwwMap`, `DeathReg`, `validate_op`, `Journal`, `range_fingerprint`/`reconcile`/`verify_range`. Depends on **`dmtap-core` and `std` only** — zero third-party crates, `#![forbid(unsafe_code)]`. The cleanest possible binding target in the whole workspace. |
| `dmtap-sync` | `crates/dmtap-sync` | **Sync** (③) — the substrate's multi-author generalisation | The six-kind op algebra (OR-Set/LWW/death-cert/PN-counter/RGA/movable-tree), `COSE_Sign1`-signed ops, HLC total order, snapshots, range-Merkle. Depends on `dmtap-core` only (dev-dep on `dmtap-clustersync` for cross-checking parity). This is the crate [`SYNC.md § 13`](SYNC.md#13-grounding) grounds itself in. |
| `dmtap-p2p` | `crates/dmtap-p2p` | **Roles** (④), partial | Real libp2p mesh transport: Kademlia (announce/resolve), Circuit Relay v2 + DCUtR (circuit relay, signalling). MSRV 1.83, depends on `tokio` + `libp2p` 0.56 — the heaviest, least binding-portable crate that still touches a substrate capability. Mailbox and cache/pin surfaces were not confirmed to exist as a standalone servable role in this survey. |
| `node` (lib name `dmtap`) | `node/` | consumer, not core | The full client daemon; depends on the above plus `tokio`. Not a binding target itself — it is what a *native Rust* product looks like when it just depends on the crates directly. |

Everything else in the workspace (`dmtap-mail`, `dmtap-send`, `dmtap-mls`, `dmtap-deniable`, `dmtap-auth`,
`gateway`) builds *on top of* the substrate for the mail profile and DMTAP-Auth; it is out of scope for
this document, which is about **five of the six** waist capabilities
([DIRECTION §1](../DIRECTION.md)) — Identity, MOTE, PUB, SYNC and Roles & Wake. **Transport** has no
binding surface of its own here: it is exercised through MOTE delivery and the Roles, exactly as
[OFFLINE §3](OFFLINE.md) records for the same reason, so a language binding never calls it directly.
The capability numbering used in the table below is local to this document; the canonical numbering
is [`substrate/README.md` §2](README.md).

**No binding infrastructure exists yet.** A direct search of the envoir repository found **zero**
`wasm-bindgen`, **zero** `#[no_mangle] extern "C"` FFI exports, **zero** `uniffi`/`.udl`/`uniffi.toml`, and
**zero** `cbindgen` configuration. The one `extern "C"` hit in the whole tree
(`gateway/src/main.rs:26`, a POSIX signal handler) is unrelated to cross-language bindings. There is no
`.github` CI at all, so no wasm32 target has ever been built in this codebase. `desktop/src-tauri/Cargo.toml`
declares `crate-type = ["staticlib", "cdylib", "rlib"]` for its Tauri wrapper crate with a comment that
this "keeps the door open for `tauri android`/`ios` later" — a placeholder, not working bindings. The
top-level `README.md` states the intent outright: *"a real client compiles the Rust core to WASM and
speaks to a real mesh; today's web client simulates the network."* This document is the plan for making
that sentence true, generalised past the web client to every product in the suite.

**Design rule: bindings live in wrapper crates, not in the core.** `dmtap-clustersync`'s zero-dependency,
`forbid(unsafe_code)` posture is valuable on its own terms (it is the easiest crate in the tree to audit
and the easiest to port to a constrained target) and MUST NOT be given up to accommodate a binding. The
`unsafe` that any FFI boundary needs (`#[no_mangle] extern "C"` for cgo, `#[wasm_bindgen]` glue, UniFFI's
generated scaffolding) belongs in a **thin wrapper crate** per surface (e.g. `dmtap-ffi`, `dmtap-wasm`,
`dmtap-uniffi`) that depends on `dmtap-core`/`dmtap-clustersync`/`dmtap-sync` and re-exports their
functions across the boundary — never in the algebra crates themselves.

---

## 3. Target surfaces and the mechanism for each

| Surface | Mechanism | Consumers | Why |
|---|---|---|---|
| **Native Rust** | direct crate dependency (`Cargo.toml` path/registry dep) | envoir `node`/`gateway`, and any future Rust server or desktop backend | Zero binding cost — this is the surface that already works today. |
| **Go** | **C-ABI** (`#[no_mangle] extern "C"` + a generated header) consumed via **cgo** | vulos's OS/control-plane backend, flowstock, whatsacc — all Go | Go has no native Rust FFI story other than cgo; a `dmtap-ffi` crate built as a `cdylib`/`staticlib` gives Go a C header (`cbindgen`-generated) it links against. This is the surface with a real, disclosed cost — see §5. |
| **WASM (browser/JS)** | `wasm-bindgen` compiling to `wasm32-unknown-unknown`, loaded by a JS/TS host | ofisi (editor frontend), kerf's frontend, vidmesh's browser-side `kernel-ts` layer | Every one of these products already ships a JS/TS frontend; a WASM build gives them the *same compiled core* in-process in the browser, no server round-trip, no cgo. This is also the surface envoir's own README already commits to for its web client. |
| **UniFFI** | `.udl`-declared interface, generating Swift/Kotlin bindings | a future native (non-WebView) mobile client, **if** one is built | Conditional, not mandatory. Vulos's current mobile story is Tauri-webview + JS, which is served by the WASM surface, not UniFFI. UniFFI becomes relevant only if a native iOS/Android app bypasses the WebView — `desktop/src-tauri`'s existing `cdylib`/`staticlib` crate-type is the toehold this would extend. Do not build this surface speculatively; build it when a native mobile client is actually scoped. |

A fifth surface — a **Python binding** (e.g. via PyO3) for kerf-pub — is deliberately **not** in this plan.
kerf-pub's value as a from-scratch, independently-written Python re-implementation of §22 is that it is a
second pen holding the spec accountable, per the repository's implementation-neutral stance
([`FEEDS.md § 6`](FEEDS.md#6-reference-implementation--kerf-pub-proves-the-http-test), and see
[`ADOPTION.md`](ADOPTION.md)). Converting kerf-pub into a thin binding over the Rust core would remove that
independent check. Its correct job going forward is to keep validating against the same frozen vectors the
Rust core validates against (§4), not to *become* the Rust core in another language.

---

## 4. The non-negotiable rule: byte-identical merge semantics

**Every surface above MUST produce byte-identical results for byte-identical inputs, because every
surface is a thin binding over the same compiled `dmtap-core`/`dmtap-clustersync`/`dmtap-sync` code —
never an independent re-implementation of the algebra.** A Go product calling through cgo, a browser
calling through WASM, and a native Rust server all execute the *same machine code path* inside the core
crate; the binding layer only marshals arguments in and results out. This is what makes "two independently
built products interoperate on any capability they both adopt" ([`README.md § 3` rule 2](README.md#3-adoption-rules-normative))
actually true rather than aspirational: it is not two teams reading [`SYNC.md`](SYNC.md) carefully and
hoping their CBOR encoders agree bit-for-bit — it is one encoder, called from five languages.

**The frozen conformance vectors are the cross-surface proof, not a formality.** `sync_vectors.json`
(20/20 frozen, [`SYNC.md § 10`](SYNC.md#10-conformance-vectors)) and `pub_vectors.json`
(`../conformance/vectors/`) already exist as the byte-exact known-answer tests every implementation must
reproduce. The discipline this plan adds: **every binding surface's build MUST run the same vector suite
against its own binding path before it ships**, not just against the native Rust unit tests. Concretely:

- The native Rust crate runs the vectors directly (already the intended use).
- The Go cgo binding runs the vectors through the C-ABI boundary — proving marshaling doesn't corrupt a
  signature, a CBOR byte, or a hash.
- The WASM binding runs the vectors through `wasm-bindgen` in a real JS test runner — proving the
  `wasm32` target and the JS↔WASM boundary don't change a single output byte.
- UniFFI, if built, runs them through the generated Swift/Kotlin glue.

**kerf-pub already proves this discipline works cross-language, today.** kerf-pub's Python
implementation vendors `pub_vectors.json` verbatim from this repository and its test suite diffs
byte-identical against it (confirmed: `tests/vectors/pub_vectors.json` in kerf-pub is byte-identical to
`conformance/vectors/pub_vectors.json` here). That is the exact pattern this plan generalises: a vector
suite frozen once in the spec repo, reproduced byte-for-byte by every consuming implementation, in
whatever language it's written in — whether that implementation is an independent rewrite (kerf-pub today)
or a thin binding over the compiled core (the plan for Go/WASM going forward). One honest gap to close
alongside this plan: `dmtap-core`'s own `pubobj` module is not yet confirmed to run against
`pub_vectors.json` — the vector generator's own metadata still describes the Rust core as not implementing
the PUB extension. Wiring that check up is a prerequisite for treating `dmtap-core` as the source of truth
the other four surfaces bind to.

A surface that cannot reproduce the frozen vectors is not a conformant binding of the core — it is a bug in
the binding, full stop, because there is only one implementation of the algebra to disagree with.

---

## 5. The honest cost: cgo and the pure-Go decision

Vulos's OS/control-plane repository has an explicit architecture decision (**D23/J**) to ship a **pure-Go**
binary — no `cgo`, so `CGO_ENABLED=0` static, cross-compilable, no C toolchain dependency at build or
deploy time. A cgo-based binding directly contradicts that decision for any Go product that adopts it
(the OS backend, flowstock, whatsacc). This is a **real constraint, not a rounding error**, and this plan
does not paper over it. Three options, honestly costed:

1. **Accept cgo for that one binary/build target.** Simplest, fastest to build, and the standard way Go
   projects consume Rust (e.g. via `cgo` + a `cdylib`). Cost: loses `CGO_ENABLED=0`, requires a C toolchain
   and the target's Rust `cdylib` at build time, complicates cross-compilation (a separate `cdylib` per
   target OS/arch), and reintroduces a class of memory-safety-at-the-boundary bugs cgo is known for
   (though the `unsafe` surface can be kept small and audited, per §2's wrapper-crate rule). This is
   acceptable where a product already builds natively per-platform and doesn't need a single static
   cross-compiled binary — flowstock and whatsacc, which don't carry D23/J's pure-Go constraint themselves,
   are the more natural first adopters of this option.

2. **A separate sidecar process.** Ship the Rust core as its own small binary (or reuse `envoir-node`'s
   existing daemon pattern) and have the Go product talk to it over a local socket/loopback HTTP —
   exactly the pattern envoir's own desktop app already uses for its Tauri↔`envoir-node` boundary. This
   preserves `CGO_ENABLED=0` and a pure-Go binary completely; the cost is a second process to supervise
   (start/stop/health-check/restart), IPC latency (small, for a loopback call), and a wire protocol between
   Go and Rust that itself needs to be simple and stable (a local, unauthenticated-by-default loopback
   surface is a different trust boundary than the public network — treat it accordingly). This is the
   **lowest-risk option for the OS/control-plane repo specifically**, since it changes nothing about D23/J.

3. **A pure-Go WASM runtime, embedding the core compiled to `wasm32-wasi` (or `wasm32-unknown-unknown`).**
   Runtimes like `wazero` execute WASM entirely in Go with no cgo and no C toolchain, so `CGO_ENABLED=0`
   and static cross-compilation both survive intact — while still running the *same compiled core binary*
   the browser surface uses (§3), which is the strongest version of §4's "same compiled code" guarantee of
   any of the three options here (the Go product and the browser would, in this option, load the literal
   same `.wasm` artifact). Cost: WASM-in-Go has real overhead relative to native cgo calls (host-function
   marshaling, no direct memory sharing without copying), and the core crates would need to build cleanly
   for a WASM target with no OS-level assumptions — true today for `dmtap-clustersync` and `dmtap-core`
   (§2's dependency audit), not yet verified for anything pulling in `dmtap-p2p`. This is the option that
   best resolves the D23/J tension **without giving up the single-binary guarantee**, and is the
   recommended default for any pure-Go product (OS/control-plane, and by extension flowstock/whatsacc if
   they want to stay cgo-free) once a WASM build of the relevant core crates exists.

No option is free. The plan does not pick one globally — it is a per-product decision gated on whether that
product already carries a pure-Go constraint (option 3, or option 1 if the constraint is dropped for that
binary) or can freely add a sidecar (option 2).

---

## 6. Migration path per product (optional, per-capability, nothing forced)

Per [`README.md § 3` rule 1](README.md#3-adoption-rules-normative), **adoption of any waist capability is
always optional and always per-capability** — nothing below is a mandate, and a product may adopt Sync via
a binding while never touching Feeds, or vice versa. This section states what each product's *existing*
hand-rolled logic would be replaced by, if and when that product's owners choose to adopt the capability
through a core binding, per the current state recorded in [`ADOPTION.md`](ADOPTION.md).

- **flowstock** (Go, Sync only). Replaces: its own HLC type (`backend/internal/store/hlc.go`), its
  JSON+hex-Ed25519-signature op format, and its two ad hoc merge strategies (whole-row LWW, insert-only
  ledger) in `backend/internal/sync/sync.go` / `store.go`. Keeps: its SQLite storage and its
  `GET/POST /api/sync/*` HTTP handlers as thin wrappers that marshal into/out of `dmtap-sync`/
  `dmtap-clustersync` calls. Binding: cgo (option 1, §5) or a sidecar (option 2) — flowstock carries no
  pure-Go constraint of its own, so cgo is the lower-effort path. Gains: COSE-signed CBOR ops, the full
  six-kind CRDT algebra (it currently has 2 of 6), DeviceCert-based multi-device authorisation in place of
  its bearer-secret-primary transport auth.
- **vulos OS/control-plane "fabric"** (Go, Sync only, if adopted). Replaces: `internal/fabric/fabric.go`'s
  LAN-only mDNS+JSON transport and `internal/multiinstance/appsync.go`'s single-table LWW+OR-set logic.
  This repo **does** carry the D23/J pure-Go constraint, so this is the product the §5 cgo-vs-sidecar-vs-WASM
  tradeoff is written for; a sidecar (option 2) or a Go-WASM runtime (option 3) is the fit, not raw cgo.
  Gains: sync stops being LAN-only and app-registry-only — the same binding could, if adopted, extend to
  any structured state the OS wants synced across a user's boxes, not just installed-app records.
- **vulos peering subsystem** (Go, Identity + Feeds + Roles, if adopted). Replaces: `peering/identity.go`'s
  own Ed25519+rotation cert format, `peering/feeds.go`'s signed-but-uncontent-addressed hash-chained feed,
  and `peering/relay.go`'s bespoke mailbox auth header — each independently designed and each already doing
  roughly the right *thing*, just not the spec's *bytes*. Same Go binding tradeoff as above applies.
- **vulos-relay** (Go). Its `tunnel/pubcache/*` (Feeds & Blobs) is **already spec-conformant**
  (`/.well-known/dmtap-pub/*`, BLAKE3 DS-tagged addressing) — nothing to migrate there; it is a candidate
  to eventually *become* a binding consumer purely to stop maintaining a parallel Go implementation of
  logic the core crate also has, not because it is behaviorally wrong today. Its `tunnel/rendezvous/*`
  (Roles) is a structurally faithful but independently-wired-protocol implementation of announce/resolve/
  signal/mailbox; a binding would let it speak the core's `LocationRecord` bytes directly instead of its
  own record shape.
- **ofisi** (JS/TS frontend, Sync — partially, by design). Yjs is load-bearing through the editor
  (ProseMirror binding, undo manager, whiteboard binding); replacing its CRDT engine outright is not a
  binding exercise, it's a rewrite of the editor's data model, and is **out of scope** for this plan. The
  realistic adoption path is narrower and still valuable: wrap Yjs's binary update as an opaque payload
  inside a `dmtap-sync` `COSE_Sign1` envelope via the WASM surface (§3), gaining per-op author attribution
  and DeviceCert-based authorisation at the transport layer without touching Yjs's merge algorithm. This is
  adopting Sync's *authenticity envelope*, not its *algebra* — a legitimate, disclosed partial adoption, not
  a stepping-stone to full conformance.
- **kerf-pub** (Python, Feeds & Blobs). Deliberately **not** a binding consumer (§3) — it stays an
  independent implementation, validated against the same frozen vectors a WASM/cgo binding would also be
  validated against. Its remaining gap to full conformance (DeviceCert chains; currently `signer == pub`
  only) is a Python-side implementation task, not a binding-adoption task.
- **vidmesh** (Rust, Feeds & Blobs, if the founder-gated convergence proceeds). This is the one product
  where "migration" is Rust-to-Rust, not cross-language: `vidmesh-kernel`'s own BLAKE3 hashing, `Hint`
  tuple, and `Record` envelope would be replaced by a **direct crate dependency on `dmtap-core`**
  (`PubManifest`/`PubAnnounce`/`FeedHead`/`FeedEntry`, the DS-tagged multihash addressing, the 4-variant
  `Hint` enum) — no cgo, no WASM, no wrapper crate needed, because both sides are already Rust. The one
  genuinely new thing vidmesh's own Feeds primitive lacks today (a `seq`/`prev` append-only feed chain)
  would come for free from adopting `dmtap-core::pubobj` rather than needing to be built from scratch. Once
  that convergence lands, vidmesh's browser-side `kernel-ts` layer would consume the **same WASM build**
  ofisi and kerf's frontend use (§3), instead of hand-porting hashing/proof logic to TypeScript a second
  time. This entire item is gated on the founder decision recorded in vidmesh's own
  `docs/DMTAP-CONVERGENCE.md` and `DECISIONS.md` — nothing here changes that gate, this document only
  describes the binding mechanics *if* it fires.
- **whatsacc** (Go, low priority). Its own `ARCHITECTURE.md` marks a DMTAP adapter as "sketch only — no
  seam interface, no wire format, no schedule," and its Ed25519 command-envelope
  (`gateway/internal/keys/envelope.go`) is point-to-point RPC, not CRDT sync — the closest fit if pursued at
  all is Identity (its device keypairs), not Sync. Not scheduled; included here only for completeness since
  it is one of the products the task named as a target Go surface.

---

## 7. Summary

One core (`dmtap-core` + `dmtap-clustersync` + `dmtap-sync`, all in `/Users/pc/code/vulos/envoir`), four
possible surfaces (native Rust, cgo/C-ABI for Go, `wasm-bindgen` for JS, UniFFI for mobile if a native app
is ever scoped), one proof obligation (the frozen conformance vectors run through *every* binding, not just
the native crate), one real disclosed cost (cgo vs. D23/J's pure-Go decision, with a sidecar or a Go-native
WASM runtime as the ways out), and zero forced migrations — each product adopts what it wants, when it
wants, per capability, per [`README.md § 3`](README.md#3-adoption-rules-normative). See
[`ADOPTION.md`](ADOPTION.md) for exactly what each product does today, capability by capability, which this
plan is the answer to.
