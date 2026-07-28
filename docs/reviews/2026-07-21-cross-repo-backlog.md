# Review record — DMTAP cross-repo backlog, 2026-07-21

**Compiled:** 2026-07-21 · **Filed into KOTVA:** 2026-07-28
**Original path:** `/Users/pc/code/vulos/DMTAP-BACKLOG.md` (untracked, no repo)

Items found during the 2026-07-21 spec work that belong to **other repos**, recorded here because
KOTVA is where the spec work happened and there was nowhere else to put them. This is a **frozen
record** ([`README.md`](README.md)); the text under [Record as written](#record-as-written) is
verbatim.

**KOTVA does not own any of these repos and this file does not dispatch them.** It records what was
found, and — as of the triage below — what is now true. No file outside this repo was edited in
either pass.

---

## Triage — 2026-07-28

Checked by reading the target repos **read-only**: no builds run, no tests run, no files written,
in any repo other than KOTVA. Where that was not enough to settle an item, it says so.

**Repo renames since 2026-07-21** — the record below uses the names as they were on that date. The
mapping needed to follow it today:

| In the record | Today |
|---|---|
| `ofisi` (item 6) | **`diwan`** — `/Users/pc/code/vulos/diwan`; the rename swept KOTVA's own files, and this record was restored to the 2026-07-21 spelling afterwards (see [`README.md`](README.md)) |
| `/Users/pc/code/exo/kerf/packages/kerf-pub` (items 2, 3) | `/Users/pc/code/vulos/kerf/packages/kerf-pub` |
| `dmtap` / `dmtap-core` (items 2, 3) | `kotva` / `kotva-core` — this repo |
| `vulos-relay` (items 3, 5) | **no longer exists** at any path under `/Users/pc/code` |

| # | Item | Target | Status now | Evidence |
|---|---|---|---|---|
| 1 | Cluster snapshot anti-rollback trusts an unauthenticated file | `vulos` · `backend/services/sync/snapshot.go` | **RESOLVED UPSTREAM** (with a residual, below) | See [item 1](#item-1--security--vulos-snapshot-anti-rollback) |
| 2 | kerf-pub Wake diverges from `ROLES.md` §8.1 | `kerf-pub` | **STILL LIVE — and now unblocked** | See [item 2](#item-2--kerf-pub-wake-divergence) |
| 3 | "gateway" terminology follow-up in implementations | spec + `kerf-pub` + `vulos-relay` | **RESOLVED** (spec + kerf-pub); third target no longer exists | See [item 3](#item-3--gateway-terminology) |
| 4 | rustfmt/clippy debt blocks promoting the CI gate | `envoir` | **STILL LIVE** — the gate exists, these two steps are still non-blocking | See [item 4](#item-4--envoir-fmtclippy-debt) |
| 5 | Low-severity, verified non-exploitable (record only) | `vulos-relay`, `whatsacc`, `flowstock` | **PARTLY UNCHECKABLE** — one target no longer exists | See [item 5](#item-5--low-severity-record-only) |
| 6 | New defect class for review checklists (`NaN` comparison) | any repo with a review checklist | **STILL UNADOPTED** — no checklist file exists in KOTVA to carry it | See [item 6](#item-6--the-nan-defect-class) |
| 7 | *(new, found during this triage)* KOTVA has the same fmt debt item 4 describes for envoir | `kotva` (this repo) | **LIVE** | See [item 7](#item-7-new--kotva-carries-the-same-fmt-debt) |

### Item 1 — SECURITY · vulos snapshot anti-rollback

**The task that produced this triage flagged item 1 as "a SECURITY item that has never been
dispatched." That is no longer accurate, and the correction is the point of this entry.**

`vulos` commit **`c3784d64` — "sync/snapshot: authenticate latest.json before trusting its
anti-rollback Version"** closes it. `backend/services/sync/snapshot.go` now carries:

- a package doc comment that states the finding's own reasoning almost word for word — *"the
  anti-rollback check (`version <= existing`) below is only as sound as the authenticity of the
  `existing` value it reads"*, with the concrete harms spelled out (freeze the cluster with an
  absurd `Version`, or point `Key` at a stale/malicious blob to force a rollback on restore);
- `deriveLatestMACKey` / `verifyLatestDoc`, an HMAC-SHA256 over `Version`/`Key`/`CreatedAt` under a
  key derived from the cluster passphrase, domain-separated from the SSE-C key by the constant
  `latestMACDomain = "vulos-sync-latest-json-mac-v1"`;
- a `mac` field in the `LatestDoc` wire format with a `Consumers MUST call verifyLatestDoc before …`
  note, and a `Run()` step-4 comment covering the MAC-fails branch.

The design call the backlog said was needed ("sign the doc? conditional-write? leader lease?") was
made: **MAC the doc under the existing cluster passphrase**, on top of the fencing lease that was
already there.

**Residual, not dispatched (recorded, not fixed — `vulos` is not KOTVA's to edit).**
`CompactorConfig.Passphrase` is documented as *"Leave empty only in tests that don't care about
latest.json authenticity (`existingSnapshotVersion` then falls back to trusting `Version`
unconditionally, matching pre-authenticity behavior)."* An empty passphrase therefore **fails open**
to the exact pre-fix behaviour. `BuildCompactor` is stated to always set it in production, so this
is a test affordance rather than a live hole — but it is a fail-open default on a security check,
which is the shape that becomes a hole the first time a new construction path forgets the field.
Whoever owns cluster-snapshot leader election should decide whether the zero value should be a hard
error instead.

**What was NOT verified:** vulos's own tests were not run and its build was not exercised. This is a
read of the committed source and the commit message, nothing more.

### Item 2 — kerf-pub Wake divergence

**Still live**, but the blocker the 2026-07-21 note identified is now **cleared**, so the item is
dispatchable for the first time.

- The note said the ordering was: implement §25's objects in `dmtap-core` → the §25 conformance
  cases become executable → *then* converge kerf-pub. **Step one is done.**
  `crates/kotva-core/src/pubsub.rs` now defines `Subscription` (§25.4) and `SubscriptionRevoke`
  (§25.5), with the signing-prefix constant `DMTAP-PUB-v0/subscription-revoke\x00` and a
  `verify_for` authorization path. There is now something normative to converge *onto*.
- kerf-pub has **not** converged: `src/kerf_pub/wake.py` is still its own extension, and
  `substrate/ADOPTION.md` still records it as *"Wake — independent. kerf-pub ships its own Wake
  extension (VAPID keys, `router.py`'s … with its own subscription/registration endpoint shapes
  rather than the `PushSubscription`/`WakePing` …)"*.

**Stale path, corrected in this pass.** kerf-pub is no longer at `/Users/pc/code/exo/kerf/packages/kerf-pub`
(the path in the record and in `substrate/ADOPTION.md`); it is at
**`/Users/pc/code/vulos/kerf/packages/kerf-pub`** — `substrate/README.md` already carried the new
path, which is what confirmed the move. The three stale occurrences (two in `substrate/ADOPTION.md`,
one in `substrate/FEEDS.md`) were corrected here in KOTVA: those are KOTVA-owned files, and a wrong
path in an adoption ledger is a wrong fact, not a style choice. Nothing in `kerf` itself was
touched. `substrate/ADOPTION.md:369` still cites the commit as `exo/kerf 66ea6e33`; that is a
historical provenance note about where the commit was made, and was left alone.

**Not verified:** whether the §25 conformance cases (`pub_subscription_decode` / `_verify` /
`_signing_preimage` / `_lifecycle` / `_revoke_verify` / `_revoke_effect`, `pub_subscribe_quota`) are
now *executable*. `make conformance` reports 285 cases at `construction-todo` and does not name them
individually beyond a sample, so this needs a separate check against `conformance/suite.json`.

### Item 3 — "gateway" terminology

**Resolved on the spec side, and the one implementation that mattered has already followed.**

- Spec: `22-public-objects.md` §22.5.1 now opens with an explicit
  *"**Terminology (2026-07 rename, documentation-only).** This surface was previously called the
  'gateway HTTP profile' / 'well-known gateway.' It is **not** a gateway in the sense the rest of
  this specification now uses that word — that term is reserved for the **legacy-mail adapter role**
  (§7, §0.2.3) …"*, and confirms **"The `/.well-known/dmtap-pub/*` paths, methods, media types, and
  error codes are UNCHANGED"** — exactly the constraint the record insisted on.
- `kerf-pub`: already aligned, commit **`53db1c63` — "kerf-pub: align gateway/PUB-server terminology
  with the spec's §22 rename"**. Its remaining `gateway` identifiers are IPFS-gateway ones, and a
  test comment now says so explicitly (*"named for the `gateway_url`/`gateways` identifiers it
  stands in for, not the §7 legacy-mail gateway role"*).
- `vulos-relay`: **the repo no longer exists** at `/Users/pc/code/vulos/vulos-relay`. Its
  `tunnel/pubcache` target cannot be checked from here. Whether it was renamed, absorbed, or
  deleted was not determined.

### Item 4 — envoir fmt/clippy debt

**Still live.** The framing offered to this triage — *"may already be resolved (envoir gained a real
CI gate this session)"* — is **half right and the important half is wrong.**

envoir does have a real CI gate: `.github/workflows/ci.yml` exists and installs `rustfmt, clippy`
(line 115). But the two steps this item is about are **still `continue-on-error: true`**:

```
ci.yml:196   # ~2600 rustfmt diffs and ~79 clippy findings, none introduced by the work
ci.yml:203   # continue-on-error) once the debt is paid down in its own commit, which is
ci.yml:206         continue-on-error: true
ci.yml:207         run: cargo fmt --all -- --check
ci.yml:210         continue-on-error: true
ci.yml:211         run: cargo clippy --all-targets -- -D warnings
```

The comment block at lines 196–203 is still the 2026-07-21 text, verbatim. **Nothing about this item
has moved.** A CI gate that runs a check non-blockingly is exactly the state the item describes.

**What was NOT verified, and why.** The `~2600` / `~79` counts were **not** re-measured. `envoir` is
under concurrent work by another workflow and is explicitly off-limits for writes; `cargo fmt` and
`cargo clippy` both touch `target/` and `Cargo.lock`, so neither was run. The counts should be
treated as stale for a second reason: commit **`620a68c`** carved envoir down to node-only,
dropping the gateway and its conformance/fuzz coverage, so a large fraction of the code the counts
were taken over is no longer in the repo. Someone with write access should re-measure before
planning the paydown commit.

### Item 5 — low-severity, record only

- **`vulos-relay` `client/src/chunkProof.js`** — **uncheckable**: the repo no longer exists at the
  recorded path (see item 3).
- **`whatsacc`** (JCS float64-vs-int64 divergence) and **`flowstock`** (negative-wall rejection test
  passing for the wrong reason) — **not checked**. Both were recorded as verified non-exploitable
  and needing no action, so neither was re-opened; `flowstock` exists at
  `/Users/pc/code/vulos/flowstock`, `whatsacc` was not located under `/Users/pc/code`.

### Item 6 — the `NaN` defect class

**Still unadopted as a checklist item, because KOTVA has no review checklist to adopt it into.** The
repo carries `STYLE.md` (prose/spec style), `tools/lint.py` (12 mechanical spec checks), and
`SECURITY.md` — none of which is a code-review checklist, and the spec-lint checks operate on
markdown, not on implementations.

The class itself is unchanged and correct, and is reproduced in the record below. Restated so it
survives without the record:

> **JavaScript `NaN` comparison semantics.** `parseInt` on a malformed counter yields `NaN`, and
> `NaN < x`, `x < NaN` and `NaN >= x` are **all** `false`. A comparator written the obvious way
> therefore returns "not less than" for garbage input, which callers read as "apply this op." Any
> decode boundary in a language with `NaN` that funnels into a bare `<`/`>` needs to consider
> `NaN`, **not only precision loss**.

It generalises the ordered-domain invariant (`substrate/FEEDS.md` §4.3, `substrate/SYNC.md` §3),
which was confirmed as a real recurring class found independently in five languages. Where it wants
to live is an open question for whoever owns cross-repo review process — this file is a record, not
a checklist, and putting it here is a holding action.

### Item 7 (new) — KOTVA carries the same fmt debt

Found while establishing the verification baseline for this pass, and recorded because it is the
same item as #4 pointed at this repo:

- `cargo clippy --workspace --all-targets -- -D warnings` — **clean**.
- `cargo fmt --all -- --check` — **1009 diff hunks across 47 files**, measured on a detached
  worktree at `HEAD` (`585a8b1`) so the count is of committed code and not of any working-tree
  change. Pre-existing; untouched by this pass, which changed only markdown.

Same reasoning as item 4 applies: this wants its own commit, landed when the tree is quiet, not
mixed into content work.

---

## Record as written

*Everything below this line is the 2026-07-21 file, verbatim and unmodified. Repo paths in it are
as they were on that date; several have since moved — see the triage above.*

---

# DMTAP cross-repo backlog — deferred, found during the 2026-07-21 spec work

Items found while working on the dmtap spec restructure that belong to **other repos**.
Deliberately NOT actioned, to keep focus on the spec. Each is independently dispatchable.

Nothing here is blocking. Ordered by value.

---

## 1. vulos — cluster snapshot anti-rollback trusts an unauthenticated file

**Repo:** `/Users/pc/code/vulos/vulos` · **File:** `backend/services/sync/snapshot.go`
**Found by:** downstream ordered-domain audit (out of that task's scope, correctly not fixed)
**Class:** authenticity-of-write — NOT a decode-width bug

`LatestDoc.Version` is used for an anti-rollback check (`version <= existingVersion`) against a
shared S3 object at `cluster/snapshot/latest.json`. Nothing establishes that `latest.json` is
itself tamper-evident *before* its `Version` is trusted. An anti-rollback rule is only as strong
as the authenticity of the counter it reads — the same reasoning as `substrate/FEEDS.md` §4.3,
one layer up.

Belongs to whoever owns cluster-snapshot leader election. Needs a design call (sign the doc? use
a conditional-write/precondition? leader lease?) before code.

---

## 2. kerf-pub — Wake diverges from `ROLES.md` §8.1

**Repo:** `/Users/pc/code/exo/kerf/packages/kerf-pub`
**Status:** known gap, already recorded in `substrate/ADOPTION.md`

kerf-pub implements its own VAPID extension with non-spec endpoint shapes instead of
`PushSubscription`/`WakePing`.

**Was** deferred until the pub/sub extension landed, because that work could change the
subscription model and converging Wake first would be work done twice.

**That precondition is now met**: `25-pubsub.md` exists (§25.4 `Subscription`, §25.5
`SubscriptionRevoke`, §25.6 the `FeedHint` pull-with-push-hint delivery model). The blocker has
moved rather than cleared, though — `dmtap-core` implements **no** `Subscription` object yet
(`pubobj.rs` covers §22; `push.rs` covers `PushSubscription`/`WakePing` only), so there is still
nothing normative for kerf-pub to converge *onto* in code. The §25 conformance cases
(`pub_subscription_decode` / `_verify` / `_signing_preimage` / `_lifecycle` / `_revoke_verify` /
`_revoke_effect`, `pub_subscribe_quota`) are all unexecuted for exactly this reason.

**So the ordering is:** implement §25's objects in `dmtap-core` → those conformance cases become
executable → *then* converge kerf-pub's Wake onto the real wire types. Doing kerf-pub first would
still be work done twice, now against a spec instead of a guess.

---

## 3. Terminology follow-up in implementations, once the spec settles

The spec restructure narrows "gateway" to exactly one meaning: a node role that adapts DMTAP to
**legacy email**. `§22.5.1` / `FEEDS.md §5.1` currently also call the public-object HTTP profile
a "gateway" — an unrelated concept (HTTP, plaintext, no IP reputation, no SMTP).

The spec agent is resolving this (rename, or a `§0.8` glossary entry). **URL paths
`/.well-known/dmtap-pub/*` do NOT change** — documentation only.

Once resolved, two implementations may want their own docs/identifiers aligned:
- `/Users/pc/code/exo/kerf/packages/kerf-pub` — `router.py` serves that surface
- `/Users/pc/code/vulos/vulos-relay` — `tunnel/pubcache`

Cosmetic. Do not touch until the spec lands.

---

## 4. envoir — rustfmt / clippy debt blocks promoting the CI gate

**Repo:** `/Users/pc/code/vulos/envoir` · **Found:** 2026-07-21, adding CI

`cargo fmt --all -- --check` reports **~2600 diffs**; `cargo clippy --workspace
--all-targets` reports **~79** findings. All pre-existing — none introduced by the
restructure or the postage-seam work.

`.github/workflows/ci.yml` runs both as **non-blocking** (`continue-on-error`) so the
signal is visible without gating. Promoting them to blocking requires paying the debt
down first, and that should be **its own commit, landed when the tree is quiet** — a
2600-diff reformat mixed into feature work buries the real change and makes `git blame`
useless across every crate.

Order: pay down fmt first (mechanical, `cargo fmt --all`), then triage clippy (some
findings will be real). Then drop `continue-on-error` from both steps.

---

## 5. Low-severity, verified non-exploitable (record only, no action needed)

- **vulos-relay** `client/src/chunkProof.js` — `readHead`'s 8-byte path converts `BigInt` →
  `Number` (real precision-loss surface), but every call site bounds the result against a tiny
  ceiling (chunk index ≤ 2^20, path length ≤ 40) before use, so a rounded value still fails those
  bounds. Not exploitable today; would become one if a call site ever drops its bound.
- **whatsacc** — a float64-vs-int64 divergence in the shared JCS canonicalizer, reachable only at
  timestamp magnitudes ~12 orders of magnitude beyond anything IAT/EXP carry.
- **flowstock** — a negative-wall rejection test passes for a subtly different reason than
  assumed; harmless dead code, left alone deliberately.

---

## 6. New defect class to carry forward into review checklists

The ordered-domain invariant (`FEEDS.md` §4.3, `SYNC.md` §3) was confirmed as a **real recurring
class**, found independently in five languages by five different mechanisms. One variant was not
anticipated by the original invariant and should be added to any review checklist:

> **JavaScript `NaN` comparison semantics.** `parseInt` on a malformed counter yields `NaN`, and
> `NaN < x`, `x < NaN` and `NaN >= x` are **all** `false`. A comparator written the obvious way
> therefore returns "not less than" for garbage input, which callers read as "apply this op."
> Any decode boundary in a language with `NaN` that funnels into a bare `<`/`>` needs to consider
> `NaN`, **not only precision loss**.

Found live in ofisi (`grid.js`, `tree.js`) where a single hostile peer could overwrite any cell.
Fixed there; the checklist item is what generalises.
