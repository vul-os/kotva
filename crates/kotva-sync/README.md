# kotva-sync — the shared sync engine

KOTVA substrate capability ③, **Sync** ([`substrate/SYNC.md`](../../substrate/SYNC.md)): a
**signed, deterministic, multi-author CRDT operation algebra** with range-Merkle reconciliation,
first-class signed snapshots, and sparse namespace sync.

**Not on crates.io yet** — see "Extraction" below for exactly what is still in the way (one thing,
and it is upstream). Adopters no longer have to import a product to get it, which was the larger
half of the problem: this crate lives in the substrate repo now, beside the spec it mechanizes.

Six CRDT types (§4.3–§4.8) — OR-Set, HLC-LWW register, remove-wins death certificate, PN-counter,
RGA sequence, cycle-safe movable tree — plus a hybrid logical clock (§3), per-op RFC 9052
`COSE_Sign1` authenticity (§4.1), canonical six-section observable-state snapshot roots (§6.1), and
the §5.3 range-Merkle fingerprint fold. **std + `kotva-core` only**; this crate adds no third-party
dependency of its own, and `#![forbid(unsafe_code)]` is on the crate root.

The module map lives in the crate docs (`cargo doc -p kotva-sync --open`) rather than being
duplicated here, where it would rot.

---

## What this is *for*

It is the library several products in the suite consume, and it exists because each of them had
written its own HLC, its own op encoder and its own merge — which converge
with each other exactly as well as separate teams reading the same prose carefully do. Adopting a
specified algebra replaces "we believe these agree" with frozen bytes that either match or do not.

The reference consumer is `envoir-node`'s `syncserve`, which now depends on this crate from the
other side of a repo boundary — exactly like any other adopter, with no privileged path.

## What it deliberately is not

* **Transport.** §5.2's pull/push protocol is the host's job. No sockets, no discovery. Fast-join
  tells you what to fetch; you perform the fetch.
* **Persistence.** `SyncState` is in-memory. Bring your own store; replay or fast-join on load.
* **Identity or admission policy.** `check_admitted` tests membership in a list *you* supply. It
  resolves no `DeviceCert` chain, no namespace policy object, no revocation — that is capability ①.

## Honest limits

Carried over from the crate docs, because they belong in front of an adopter:

Sync is **not** sealed-sender: every op carries its author and HLC, visible to every replica in the
namespace — multi-author convergence needs attributable ops. A compromised author key can write ops
until revoked, and because replicated history is durable a malicious write must be *superseded* by a
later op, not "deleted". A trusted-checkpoint snapshot trusts its signer for pre-`covers` history
until backfilled and recomputed.

One more, and it is a deployment obligation rather than a runtime check: this engine is
`ext-value` **profile 2** (`EXT_VALUE_PROFILE`), a widening over §4.1's original narrower prose. A
mixed deployment diverges by *rejection* — an engine still on profile 1 refuses, with `0x0A03`, an
op an updated engine accepts — and nothing here can detect that from the other end. See the crate
docs and `SYNC.md` §4.1.2's `sync-1/ext-value-2` sub-token.

---

## The proof

```sh
cargo test -p kotva-sync                                 # 52 unit + 6 convergence properties
cargo test -p dmtap-clustersync --test sync_parity       # agreement with the §5.6 reference
cargo test -p conformance-runner --test sync_vectors     # the 24 frozen SYNC.md §10 vectors
```

`tests/convergence.rs` asserts the algebraic laws directly — commutativity, associativity and
idempotence of merge over the *observable bytes*, and tree acyclicity under every arrival order.

`sync_parity` lives in envoir's `dmtap-clustersync` rather than here, on purpose — this crate must
not depend on an envoir-local crate even for a dev-dependency, and that discipline is precisely what
made this extraction a manifest edit. See the note at the bottom of this crate's `Cargo.toml`.

The vector gate reads the frozen `sync_vectors.json` from the sibling **KOTVA** spec repo (default
`../kotva`; override with `KOTVA_DIR`, which the JS and Go harnesses read too). Note its posture,
because it still differs from the other two harnesses: without that checkout it **skips**, where
`crates/kotva-sync-wasm/tests/native_trace.rs` and `bindings/go/vectors_test.go` both hard-fail
instead. The skip is no longer silent — it prints that zero of the 24 vectors ran and what that
leaves unverified — but a green `cargo test` on a machine with no sibling checkout still does not
by itself mean 24/24 ran. **In CI it does**: `.github/workflows/ci.yml` checks KOTVA out beside the
repo and fails closed before any test runs if that checkout is incomplete.

Two further surfaces execute the **same** compiled algebra and are diffed byte-for-byte against a
trace recorded from this crate: `crates/kotva-sync-wasm` (browser/JS) and `bindings/go` (pure Go,
via wazero). All three now live in this repo, with the vectors.

---

## Extraction — done, and what it actually cost

This crate **was** `crates/dmtap-sync` inside the envoir product. Every adopter's manifest read
`git = "https://github.com/vul-os/envoir"`, which imported a mail node in order to get a CRDT
library — the suite rule that products never import each other, violated by exactly one line each.
It now lives in the substrate repo, as `kotva-sync`, under the naming ruling (substrate capability →
`kotva-*`, mail-profile capability → `dmtap-*`; Sync is `substrate/SYNC.md`).

The old version of this section was a countdown of five items. All five are closed:

1. **`dmtap-core` was inherited from envoir's workspace** (`{ workspace = true }`). It is now an
   explicit `dmtap-core = { package = "kotva-core", path = "../kotva-core", version = "0.2.0" }`.
2. **Publishing was blocked upstream.** Still is, and it is the ONE thing left: `kotva-core` is not
   on crates.io, and cargo refuses to publish a crate with a version-less dependency. Nothing in
   *this* crate's manifest is missing — see "Publish readiness" below. `kotva-core` must go first.
3. **The 24-vector gate lived in envoir's `conformance-runner`.** There are now gates *here*, beside
   the implementation, reading the vectors *in-tree*: `kotva-sync-wasm/tests/native_trace.rs` and
   `bindings/go/vectors_test.go`. envoir keeps its own consumer-side runner, which is a different
   and legitimate thing: it checks that the crate *as envoir consumes it* still reproduces the
   frozen answers.
4. **Two doc comments named `envoir-node`'s `syncserve`.** They still do. They are cross-references
   to a consumer, not dependencies, and they are accurate — `syncserve` is still where capability
   negotiation happens. They dangle across a repo boundary now, which is the honest description of
   the relationship.
5. **`bindings/go`'s module path.** Now `github.com/vul-os/kotva/bindings/go`, package `kotvasync`.
   This IS a breaking change for every Go adopter, as predicted; there is no way to move a Go module
   without renaming its import path.

### What did NOT change, and why that is checkable

Not one byte of `src/` moved. That is not a claim — it is visible in
`bindings/go/wasm_provenance.json`, which digests every Rust source the committed WASM artifact was
built from: after the move, all fourteen `src/*.rs` digests were **identical** to the ones recorded
before it, and only the two `Cargo.toml`s and `build-abi.sh` changed. The mechanism is cargo's
dependency-rename: the package is `kotva-core`, the Rust path stays `dmtap_core::`. envoir gets the
same treatment in the other direction, so its `use dmtap_sync::…` did not move either.

Two follow-on edits *did* touch `src/`, and they are separate from the move:

* **Nine clippy findings.** envoir's clippy gate is report-only over ~79 pre-existing findings; this
  repo's is **blocking** at `-D warnings --all-features`, so the debt had to be paid on arrival
  rather than inherited. Eight were mechanical. The ninth was a real latent defect: `snapshot_clone`
  was gated `#[cfg(feature = "abi")]` while its only caller, `mod abi`, is gated
  `#[cfg(all(feature = "abi", not(feature = "js")))]` — so under `--all-features` the method had no
  callers at all. A report-only lint never surfaced it.
* **The WASM artifact was rebuilt** from the corrected sources and `wasm_provenance.json`
  re-recorded, because those edits legitimately changed the bytes. It was not re-recorded to silence
  the guard — the guard fired first, exactly as designed ("the Rust sources this artifact was built
  from are GONE, but this is a repo checkout"), and it fires again on any planted change.

### Publish readiness, measured

Measured, not asserted. The manifest-verification failure the old version of this section quoted
("all dependencies must have a version requirement specified when publishing") is **gone** — that
was the missing `version` alongside the `kotva-core` path dep, and it is fixed. The dry run now gets
all the way through manifest verification and into `Packaging kotva-sync v0.1.0` before hitting the
one remaining wall, which is entirely upstream:

```
$ cargo publish --dry-run --allow-dirty -p kotva-sync
    Updating crates.io index
   Packaging kotva-sync v0.1.0 (/Users/pc/code/vulos/kotva/crates/kotva-sync)
    Updating crates.io index
error: failed to prepare local package for uploading

Caused by:
  no matching package named `kotva-core` found
  location searched: crates.io index
  required by package `kotva-sync v0.1.0 (/Users/pc/code/vulos/kotva/crates/kotva-sync)`
```

Nothing in this crate's own manifest is missing. `kotva-core` has to be published first; the same is
true of `kotva-sync-wasm`, which depends on both. (`--allow-dirty` only because the extraction is
sitting uncommitted in the worktree at the time of writing.)
