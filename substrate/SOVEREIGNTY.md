<!-- no-broker-dep:allow-file: this document IS the substrate's sovereignty ruling — it names
     Ephor repeatedly because its entire subject is the adoption boundary between the substrate
     and the broker (including the standing "Ephor is not ready" caveat this document states
     itself). It is a spec, not a build or startup path. -->

# Substrate — Product Sovereignty (adopting the substrate without adopting a broker)

> **Status:** additive normative profile of the core specification and the coordinator contract
> ([`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md)). It restates **no wire bytes** and mints no
> object, capability token, or error code — [`IDENTITY.md`](IDENTITY.md), [`FEEDS.md`](FEEDS.md),
> [`SYNC.md`](SYNC.md) and [`ROLES.md`](ROLES.md) remain the sole normative statements of what each
> capability *is*, and where this document and a byte home appear to differ, **the byte home governs**.
> What this document owns is the **adoption contract for a product that is not the reference pair**: the
> five properties such a product demonstrates on its own, and the demonstration that makes each one
> checkable rather than a slogan. It records an owner ruling of **2026-07-30**.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174.

---

## 1. The ruling

**KOTVA is the substrate: this specification plus the published shared libraries.** The libraries are
not a convenience; they are the second half of what "the substrate" means, because determinism *is* the
contract ([`SYNC § 2.2`](SYNC.md#22-determinism-is-the-contract)) and two independent implementations of a
merge algebra agree only **most** of the time — they diverge the first time one of them encodes a float,
sorts a map, or folds a hash differently, and they diverge silently, because each is internally consistent.

**envoir and ephor are the reference implementation pair** of that substrate — envoir the node, ephor the
reachability half. They go hand in hand, and their tight coupling to KOTVA is *correct*: a reference
implementation is supposed to track the spec closely. It is not a template. A product that copies the
pair's shape inherits a dependency the pair is entitled to and the product is not.

**Ephor is not ready.** As of this ruling the substrate does not treat Ephor as an available component.

Therefore: **every product that is not the reference pair takes its decentralisation from the
specification and the shared libraries alone, and MUST additionally be able to deploy itself as a node on
a cloud instance** if its operator wishes. Reachability brokering is a *future convenience for NAT
traversal*, never a prerequisite.

This is not a new principle. It is [`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md) §1's
never-load-bearing invariant and §2.2/§2.3's swappable/self-hostable clauses (COORD-2, COORD-3) applied to
a named case, plus one addition the contract does not make: a coordinator being *hireable* says nothing
about whether the product can **stand up its own node**. §3.4 is that addition.

---

## 2. The distinction that must be unmistakable

**Adopting the substrate is REQUIRED. Adopting Ephor is NOT.**

Those two sentences are not symmetric statements about two optional things. The first is the
[`README § 3`](README.md#3-adoption-rules-normative) rule 2 obligation — *if a product implements a
capability's function it MUST speak that capability's bytes* — sharpened by §3.5 below into *and it MUST
run the shared compiled algebra to produce them*. The second is a coordinator, fenced by a contract that
already forbids it from being load-bearing.

A reader finishing this section should know exactly which side of the line each piece of their product
falls on:

| You **get** this from the substrate — do not rewrite it | You **build** this yourself — the substrate does not provide it |
|---|---|
| The merge algebra: six op kinds, HLC total order, snapshots, range-Merkle reconciliation fingerprints ([`SYNC`](SYNC.md), `kotva-sync`) | **Peer enrolment**: the UX and trust store by which an operator types another node's address and pins its key (§3.2) |
| Identity objects: `IdentityKey`/`Identity`/`DeviceCert`/`KeyRotation`, the 8-word key-name (§1, §3.9.6, [`IDENTITY`](IDENTITY.md), `kotva-core`) | **Transport and session**: the actual listener, the mutual-auth handshake wiring, the retry loop (§3.3) |
| Canonical encoding and signing: deterministic CBOR (§18.1.1), DS-tag domain separation, `COSE_Sign1` op envelopes ([`SYNC § 4.1`](SYNC.md)) | **Storage, durability, backup and restore** (§3.4) |
| Content addressing, public objects and feeds (§22, [`FEEDS`](FEEDS.md), `kotva-core::pubobj`) | **Deployment**: bind address, TLS termination or reverse proxy, artifact, operator documentation (§3.4) |
| The frozen conformance vectors that prove your build is byte-identical, not merely plausible ([`../conformance/vectors/`](../conformance/vectors/)) | **The reachability seam**: your own default provider, with a broker as at most one *optional* implementation behind it (§3.1, §4) |
| The degradation vocabulary and reconcile obligations ([`OFFLINE`](OFFLINE.md)) | **Product semantics**: what your ops mean, which invariants need a single writer (`OFFLINE` R-SYNC-1) |

**Library coordinates (informative).** `kotva-core` 0.2.0 and `kotva-sync` 0.1.0 (Rust,
[`crates/`](../crates/), tag `core-v0.2.0`), `kotva-sync-wasm` 0.1.0 with its `@vul-os/kotva-sync` npm
surface, and the Go module `github.com/vul-os/kotva/bindings/go` at `bindings/go/v0.2.1` — all in **this**
repository, which is the point: an adopter pins the substrate, never a path inside a product
([`BINDINGS`](BINDINGS.md)). Registry publication (crates.io, npm) is asserted by the ruling; what is
verifiable in-tree is the manifests and the version tags above.

---

## 3. The five properties

Each property below states the obligation, then the **demonstration** — what a product does so a reviewer
can *check* the property instead of believing it. A property nobody can test is a slogan, so a product
claiming substrate adoption MUST be able to point at each demonstration.

### 3.1 R-SOV-1 — No dependency on a reachability broker in any default path

A product's **default build** and **default startup path** MUST NOT acquire a hard dependency on a
reachability broker — not at build time, not at process start, not for the product to perform its
function. A product MAY offer a broker as **one reachability provider among others**, and then:

- the provider MUST sit behind a **seam** with a **working default that needs no broker at all** (direct
  address, operator-supplied peer URL, or an optional LAN convenience — §3.2);
- the seam MUST be **off by default** (a cargo feature, a build tag, an optional dependency, a
  configuration provider name that is not the default value);
- **removing the broker MUST degrade to "not reachable from behind NAT" and never to broken.** This is the
  rung ladder of [`ROLES § 3`](ROLES.md) read honestly: rung 1 (direct) and rung 2 (hole-punch) are the
  product's own, rung 3 is the convenience. Losing rung 3 costs *reach*, never *function*
  ([`OFFLINE § 1`](OFFLINE.md), §CONTRACT 1).

**Demonstration.** Run [`../tools/gates/no-broker-dep.sh`](../tools/gates/no-broker-dep.sh) in the
product's CI on every push (§5). Plus one test the gate cannot write for you: a **removal test** that
starts the product with the broker configuration absent and asserts it reaches a serving state, with the
reachability provider reporting `not reachable from behind NAT` — an explicit state, not a silent retry
loop (`OFFLINE` R-GRADE-1 forbids the silent version).

**What a failing product looks like:** a `use ephor_client::…` in a startup path; a default configuration
value naming a broker host; a compose file whose only documented peer is a broker; a build that will not
link without the broker crate present.

### 3.2 R-SOV-2 — Peer-to-peer over an operator-supplied address

Node discovery is **manual**. An operator types the other node's URL, and the product MUST be able to
enrol and sync a peer from that address plus the peer's key alone. Specifically:

- **No central directory.** A product MUST NOT resolve peers through a registry it (or its vendor)
  operates as the default path. [`README § 3`](README.md#3-adoption-rules-normative) rule 6 already forbids
  inverting the naming ladder; this forbids the operational form of the same mistake.
- **No default endpoint.** A shipped default configuration MUST NOT contain a peer, broker, or bootstrap
  address. An unset address is an unset address: the product MUST report it as unconfigured and MUST NOT
  fall back to a vendor host (§10.7 fail-closed; §CONTRACT 3.2 no silent downgrade).
- **No assumption of a LAN.** mDNS / DNS-SD MAY be offered as a convenience; it MUST be disableable and
  MUST NOT be the only path to enrolment. A product whose sync works only on one broadcast domain has not
  adopted the substrate — it has adopted a LAN.
- The address is a **route**, the key is the **identity** (§1.2, §3.9.6). Re-enrolling the same peer at a
  new address MUST NOT change its identity, and a matching address with a mismatching key MUST fail closed.

**Demonstration.** A two-node test — two processes, two data directories, no shared parent — that enrols
node B into node A using **only** a URL and B's public key, syncs a change both ways, and asserts
convergence on the shared state root ([`SYNC § 6.1`](SYNC.md)). Plus an assertion over the *shipped*
default configuration that it contains no host, and a negative test that an unconfigured peer address
yields an explicit unconfigured error.

### 3.3 R-SOV-3 — Authentication that is safe on the open internet

A cloud node is exposed, so the deployment case that matters is the hostile one. Four obligations, each
already grounded in the spec; this rule's contribution is to bind all four to the exposed case at once:

1. **Mutual key-authenticated sync.** Both ends prove possession of their `IK`, bound into the channel.
   [`profiles/reachability.md`](../profiles/reachability.md) REACH-2 is the worked node↔node embodiment —
   a libp2p-Noise `XX` signed-static-key handshake, channel-bound by construction — and it also records
   why the obvious alternative is wrong: a challenge-response run *inside* an unauthenticated tunnel is
   relay-vulnerable, and **§13 DMTAP-Auth MUST NOT be used to authenticate a node↔node leg** (§13.3.1). A
   product terminating TLS itself MAY instead bind the peer's `IK` proof to the TLS exporter/channel, but
   MUST NOT accept an unbound assertion.
   [`SYNC § 5.4`](SYNC.md) offers two transport gates and scopes the bearer-secret arm to *a trusted
   network*: a node bound to a public address is not on one, so such a node MUST take the
   Identity-authenticated arm. This scopes an existing choice; it does not remove it.
2. **Individually signed ops.** Every replicated change carries its own `COSE_Sign1` signature and is
   verified **on its own** ([`SYNC § 4.1`](SYNC.md), and [`SYNC § 9`](SYNC.md): op-signature verification is
   mandatory in every deployment row — the gate differs, the per-op authenticity does not). A product MUST
   NOT accept an op as authentic *because it arrived over an authenticated connection*. The alternative —
   §5.6's **ambient MLS-group membership**, where ops ride unsigned inside an encrypted group
   ([`SYNC § 8`](SYNC.md)) — is authorisation by a shared secret among devices of **one** identity, and
   using that ingest path for ops arriving from other identities over the open internet is exactly the hole
   [`ADOPTION § 2`](ADOPTION.md) records against diwan's grid today.
3. **Replay defence.** Ops MUST be idempotent by op id and the receiver MUST reject a replayed or
   rolled-back op: monotonic HLC per author, and every rollback-defended counter persisted **before** the
   object bearing it is emitted (`OFFLINE` R-ID-1). Decode boundaries MUST reject out-of-domain ordered
   values — the width/sign/`NaN` defect class found four times in four languages
   ([`FEEDS § 4.3`](FEEDS.md), [`ADOPTION § 3`](ADOPTION.md)).
4. **Fail closed on mismatch.** An unknown peer key, a signature that does not verify, a certificate whose
   revocation cannot be checked (§1.5), or a handshake mismatch MUST refuse the exchange (§10.7). Falling
   back to an unauthenticated or bearer-only path when key auth fails is the silent downgrade §CONTRACT 3.2
   forbids.

**Demonstration.** Four negative tests, each asserting the *specific* refusal and not merely "an error":
(a) a peer presenting an unenrolled key is refused at handshake; (b) an op whose signature was tampered
with is rejected while its sibling ops still apply; (c) a captured op replayed a second time changes no
state and a rolled-back counter is refused; (d) with key auth disabled at the client, the server refuses
rather than downgrading. Each test asserts its own count in CI: a suite that runs zero negative tests
reports the same green as one that runs four.

### 3.4 R-SOV-4 — A real cloud-node deployment path

"It is a single binary" is not a deployment story. A product MUST be deployable as a node on a cloud
instance by an operator who has only that instance and the documentation, and the path MUST include all
five of:

1. **A configurable bind address.** Listening on loopback only, with no configuration key to change it, is
   a product that cannot be a node. The default MAY be loopback (a safe default is good); the
   *impossibility* of binding elsewhere is the defect.
2. **An honest TLS story.** Either the product terminates TLS itself (and documents certificate
   provisioning and renewal), or it documents the reverse proxy that does, including what the proxy sees.
   A `terminating` intermediary MUST be declared as such (§CONTRACT 3.1/3.2) — a proxy that reads
   plaintext is a trust boundary, whether or not the operator thought about it.
3. **A deploy artifact.** A container image, a package, or a documented build with pinned dependencies —
   something an operator can place on a host reproducibly.
4. **Data durability and backup.** The node's authoritative state MUST live outside the process
   (§CONTRACT 2.2's "keys and history at the edge" applies to the operator's own node too), and the product
   MUST document a **backup and restore** procedure — and a restore MUST NOT change the node's identity or
   force re-enrolment of its peers.
5. **Operator documentation** that a competent stranger can follow end to end: install, key generation,
   bind address, peer enrolment (§3.2), TLS, backup, upgrade.

**Demonstration.** A restore test: bring a node up, sync, take the documented backup, destroy the data
directory, restore, and assert the node resumes with the same identity and converges with its peer
without re-enrolment. Plus a bind-address test (the listener binds a non-loopback address from
configuration alone), and a documentation check that the operator guide's steps are the ones the artifact
actually needs.

### 3.5 R-SOV-5 — The merge engine is the shared one

A product that syncs structured state MUST run the **shared compiled algebra** — `kotva-sync` natively, or
one of its bindings (`kotva-sync-wasm` / `@vul-os/kotva-sync` / `bindings/go`) — not a private
re-implementation of it. [`README § 3`](README.md#3-adoption-rules-normative) rule 2 requires speaking the
capability's bytes; this rule states how a product *proves* it does. Two nodes of the same product converge
because they run the same compiled algebra, demonstrated by the frozen vectors — never because two
implementations agree in the cases anyone tried.

A product MAY wrap an unrelated engine for a surface the algebra does not yet serve (the
[`BINDINGS § 6`](BINDINGS.md) authenticity-layer path), but MUST NOT describe that surface as adopting
Sync.

**Demonstration.** The product's manifest depends on `kotva-sync` or a named binding of it at a pinned
version; the product's own CI executes the **24 frozen sync vectors**
([`../conformance/vectors/sync_vectors.json`](../conformance/vectors/sync_vectors.json)) and **asserts the
count**, failing closed when the vector file is absent rather than skipping — the failure mode
[`../.github/workflows/ci.yml`](../.github/workflows/ci.yml) already guards against in this repository,
where a sync gate once reported 24/24 having driven zero vectors. Plus a two-node convergence assertion
against the state root, not against a rendered projection.

---

## 4. Ephor's position

The reachability role Ephor fills is **legitimate and self-hostable**: announce/resolve, signalling,
circuit relay and the short-TTL content-blind mailbox — [`ROLES.md`](ROLES.md) sections 2–5, profiling the
core's §4.2, §14.3 and §14.5 — plus the `reachability-adapter` shape of
[`profiles/reachability.md`](../profiles/reachability.md). Anyone with a VPS can run one (REACH-9), which is
exactly why the role is permitted to exist: an open, key-addressed role any node MAY serve is not a
gatekeeper.

**It is not ready, and it MUST NOT be load-bearing.** The second half of that sentence needs no new rule:
[`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md) already requires it of **every** coordinator —
§1 (hired, not depended on; no coordinator is load-bearing, with one disclosed exception that is not this
one), §2.2 swappable with zero migration (COORD-2), §2.3 the self-host backstop with its two disclosed
scarcity classes, of which network reachability is one (COORD-3), and §2.4/§3 declared visibility
(COORD-4). This document adds only the named-case reading:

- **R-SOV-1a.** Ephor is one possible implementation behind a product's reachability seam, never the seam
  itself. A product MUST NOT name Ephor in a default configuration value, a default-on build feature, or
  the minimum-setup path of its documentation. Naming it under "optional: NAT traversal" is correct;
  naming it under "getting started" is a violation of §3.1.
- **R-SOV-1b.** Not-ready is a **status**, and status claims decay. A product MUST NOT record Ephor as
  available, and a document that describes Ephor's state MUST date the claim, so that "not ready" is
  re-checked rather than inherited forever in either direction.

Nothing here demotes the reference pair. envoir and ephor may depend on each other as closely as they
like; that coupling is the reference pair's business and this document constrains only products that are
not it.

---

## 5. The gate (lift this, do not re-derive it)

Specifications do not propagate; copied templates do. [`ADOPTION § 3`](ADOPTION.md) records the same
ordered-domain decode defect found four times in four languages, each invisible to its own repo's tests —
so R-SOV-1 ships as a script a product can copy, not only as prose it can agree with.

[`../tools/gates/no-broker-dep.sh`](../tools/gates/no-broker-dep.sh) — POSIX `sh`, no third-party
dependencies, configured by three environment variables (`BROKER_RE`, `SEAM_PATHS`, `SEAM_FLAG`).

### 5.1 The shape (what any implementation of this gate must do)

| Check | Kind | Question |
|---|---|---|
| **C-DEP** | structural | With **default** features/tags, does the resolved dependency closure contain the broker? Read from the toolchain's own resolver, never from a manifest grep — a manifest cannot see a transitive edge, which is how a broker actually arrives. |
| **C-START** | textual | Is the broker named **anywhere outside a declared seam**? A default endpoint, a hostname constant, a compose file, a systemd unit. An optional dependency that the startup path still reaches for is still a default-path dependency. |
| **C-SEAM** | bookkeeping | If a seam is declared: does it exist, and is it **off by default**? An undeclared or default-on seam is the dependency wearing a hat. A stale seam path is an exemption nobody is reading. |

Four rules bind any implementation:

- **Fail closed, and never exit 0 by doing nothing.** Unknown ecosystem, missing toolchain, empty
  dependency closure, zero files scanned, stale seam path, a declared seam with no flag naming it →
  **exit 2** (*cannot check*), distinct from exit 1 (*violation*) and exit 0 (*checked and clean*). Today's
  lesson, in this repository's own words: `go test` discards a passing package's output entirely without
  `-v`, stderr included, so a "loud skip" is invisible in exactly the run that matters. The only skip a gate
  may have is a non-zero exit.
- **Every check always runs, and a violation outranks an unverifiable check.** Exit 1 beats exit 2. The
  shipped script's first revision exited 2 the moment any check could not run, which **hid a real C-DEP
  violation behind an unrelated stale seam path** — found by running the gate against a Go fixture while
  writing it, and now pinned by a named regression control. "Cannot check" must never suppress "did check,
  and it is broken".
- **Carry a self-control.** `--selftest` runs the gate against hermetic fixtures — clean trees and a
  declared default-off seam that must pass, planted violations that must fail, and unverifiable
  configurations that must exit 2 — so a regex or refactor that renders the gate inert fails the build
  instead of reporting success. This is the discipline [`../tools/lint.py`](../tools/lint.py)'s C12/C15
  exist to enforce on the linter itself, after three of their own revisions shipped silently inert.
- **Be language-honest, and name what you did not check.** One script does not fit every ecosystem, and
  pretending otherwise produces a gate that passes by not understanding the project. The self-control runs
  the fixtures for every toolchain present, exits 2 if that is none, and prints `NOT VERIFIED` for each
  ecosystem it could not exercise — so "the Rust half is fine" is never mistaken for evidence about the Go
  mechanics.

### 5.2 Mechanics per ecosystem

The mechanics differ; the question does not. Rust and Go are the worked cases in the shipped script.

| Ecosystem | C-DEP command (default features/tags) | C-SEAM mechanism |
|---|---|---|
| **Rust** | `cargo tree -e normal --prefix none` — resolves with default features; `-e normal` drops dev/build edges, so a dev-dependency broker client (a test fixture) is correctly not a violation | `optional = true` dependency + a feature absent from `[features] default` |
| **Go** | `go list -deps ./...` — the import closure under **default** build tags; a seam behind `//go:build <tag>` is absent here unless the tag is on by default, which is precisely the property being checked | a `//go:build <tag>` constraint on every seam file |
| **Node** | `npm ls --omit=dev --all --parseable` | `optionalDependencies` / a dynamic `import()` behind configuration |
| **Python** | declared dependencies only — **reduced assurance, and the script says so**: with no resolver assumed present, a transitive broker dependency is not visible | extras (`[project.optional-dependencies]`) |

A product whose ecosystem is not in the table adds its row rather than skipping the check; the shipped
script exits 2 on an unrecognised manifest for that reason.

### 5.3 Lifting it

```sh
cp kotva/tools/gates/no-broker-dep.sh <product>/tools/gates/
# CI step — a product with no broker integration at all needs no configuration:
sh tools/gates/no-broker-dep.sh .
# a product that does have a seam declares it, and the manifest that legitimately names it
# (every declared path must EXIST — a stale entry exits 2 rather than widening the exemption):
BROKER_RE='pier-|vul-os/pier|ephor|vulos-relayd' \
SEAM_PATHS='src/reach/broker Cargo.toml' \
SEAM_FLAG='broker-reach' \
  sh tools/gates/no-broker-dep.sh .

# and the gate's own proof of teeth:
sh tools/gates/no-broker-dep.sh --selftest
```

`--selftest` SHOULD run in the same CI job as the gate itself, because a copied script
drifts: the copy that no longer fails on a planted violation is worse than no gate, since it reports a
pass nobody earned. In this repository it is `make gates`, wired into
[`../.github/workflows/ci.yml`](../.github/workflows/ci.yml) as a blocking job with **both** the Rust and Go
toolchains installed and an asserted control count — 10 controls across 2 ecosystems at 2026-07-30 — so a
selftest that quietly stopped exercising an ecosystem fails the build.

**`BROKER_RE` must track the broker's name, and the self-test must plant the CURRENT one.**
The broker was renamed `ephor` → `pier`, and the default `BROKER_RE` was not updated with it. For a
period every copy of this gate — in this repo and in every product that had lifted it — reported
PASS while matching only a name nothing was called any more: a product could take a hard
`pier-client` dependency and the gate would not see it. The self-test did not catch this because
all of its fixtures were also written in `ephor`, so they could not distinguish "the gate matches
the broker" from "the gate matches the string `ephor`". The rule that follows: when the broker is
renamed, the OLD name stays in `BROKER_RE` (a stale dependency is still a violation) and the new
one is added, and a self-test control must plant the broker under its current name. Those are the
`rs_dep_current_name` / `rs_default_current_name` controls; blank the current name out of
`BROKER_RE` and they go red, which is the property the rest of the suite was missing.

Note also that a bare `pier` is deliberately **not** used: the scans are `grep -Ei` with no word
boundaries, so it would flag "happier"/"copier"/"occupier" in prose until someone silenced the
gate. Match the shapes the broker actually takes (`pier-`, `vul-os/pier`) instead.

---

## 6. Conformance checklist

| # | A product… | Rule |
|---|---|---|
| SOV-1 | acquires no hard broker dependency in its default build or startup path | R-SOV-1 |
| SOV-2 | keeps any broker behind a declared, default-off seam whose removal costs only NAT reachability | R-SOV-1, R-SOV-1a |
| SOV-3 | runs the R-SOV-1 gate in CI on every push, self-control included | §5 |
| SOV-4 | enrols a peer from an operator-supplied address + key, with no central directory and no default endpoint | R-SOV-2 |
| SOV-5 | treats mDNS as an optional convenience, never the only enrolment path | R-SOV-2 |
| SOV-6 | mutually key-authenticates sync, channel-bound, and never authenticates a node leg with §13 | R-SOV-3.1 |
| SOV-7 | verifies every op's own signature rather than trusting the connection it arrived on | R-SOV-3.2 |
| SOV-8 | rejects replayed and rolled-back ops, persisting ordered counters before emitting them | R-SOV-3.3 |
| SOV-9 | fails closed on any auth mismatch — no fallback to bearer-only or unauthenticated | R-SOV-3.4 |
| SOV-10 | can bind a non-loopback address from configuration, with a declared TLS story | R-SOV-4.1/4.2 |
| SOV-11 | ships a deploy artifact, a documented backup/restore that preserves identity, and an operator guide | R-SOV-4.3/4.4/4.5 |
| SOV-12 | depends on the shared merge engine and executes the frozen vectors with an asserted count | R-SOV-5 |
| SOV-13 | dates any claim about a coordinator's readiness rather than inheriting it | R-SOV-1b |

---

## 7. Honest residual

- **A static gate cannot see a runtime-resolved dependency.** C-DEP reads a build graph and C-START reads
  text; a product that fetches a broker address from a vendor API at first run, or plugs in a broker
  through a generic plugin loader, passes both. What closes that gap is the §3.1 removal test and the §3.2
  default-configuration assertion — tests a product writes about its own behaviour, which no gate lifted
  from another repository can write for it.
- **C-START is a heuristic, and deliberately blunt.** It fails on *any* mention outside a declared seam,
  including a comment. That is the trade: a precise version needs a parser per language, and a gate that
  needs a parser per language does not get copied. The seam allowlist is the pressure valve, and a stale
  entry in it exits 2 rather than quietly widening.
- **This repository cannot run the demonstrations.** The substrate has no product in it, so §3's
  demonstrations are checked in each product's CI, and this document's own teeth are the shipped gate and
  its self-control. A checklist row nobody has run against a given product is an obligation, not a result —
  [`ADOPTION.md`](ADOPTION.md) is where per-product reality is recorded, and it is a **snapshot**, not a
  certification.
- **"Not reachable from behind NAT" is a real loss for some users.** R-SOV-1's degradation is honest, not
  free: a product with no broker and no public path is unreachable from outside its network, and the
  scarce-public-address exception is disclosed in §CONTRACT 2.3 and REACH-9 rather than solved here.
- **A cloud node is a new attack surface.** R-SOV-4 asks products to expose themselves on purpose, which
  is why R-SOV-3 is not optional and why the exposed default is the case its tests must cover. The
  substrate reduces what an exposed node must be trusted for; it cannot make exposure risk-free.
- **The shared libraries are young.** `kotva-sync` is 0.1.0 and its Sync capability is the one genuinely
  new normative area in the waist ([`README § 1`](README.md)). Adopting the shared engine buys byte-identical
  behaviour across surfaces; it does not buy maturity that does not exist yet, and the frozen vectors bound
  what has been pinned down, not what has been proven safe.
- **Status claims about another repository are the weakest sentence here.** "Ephor is not ready" is a
  2026-07-30 owner statement recorded by this document, not a property this document can verify on every
  read — hence R-SOV-1b's dating requirement, which is a mitigation and not a fix.

---

## 8. Grounding (informative)

The manual-enrolment shape this document requires is well-precedented, which is the point: it is the
boring option, and it needs no infrastructure.

- **Manual peer configuration with a pinned public key.**
  [WireGuard](https://www.wireguard.com/protocol/) — each peer is a public key plus an optional endpoint;
  there is no directory, and a peer with no endpoint is simply unreachable until it speaks first. This is
  R-SOV-2's model, including its honest failure mode.
- **Trust on first use, pinned thereafter.** [OpenSSH `known_hosts`](https://man.openbsd.org/ssh#FILES) —
  the operator supplies the address, the key is pinned, and a later mismatch is a hard refusal rather than
  a prompt to continue. R-SOV-2's "matching address, mismatching key fails closed" is this behaviour.
- **Device pairing by identity, addresses as hints.**
  [Syncthing](https://docs.syncthing.net/users/security.html) — devices are introduced by device ID
  (a key), discovery is a convenience layered over that, and relaying is optional and disableable. This is
  the whole of §3.1 + §3.2 in a shipped product, including the part where turning relaying off costs reach
  and nothing else.
- **Channel-bound mutual authentication in one pass.** The libp2p-Noise `XX` signed-static-key handshake
  named by [`profiles/reachability.md`](../profiles/reachability.md) REACH-2, and the
  Asokan–Niemi–Nyberg tunnelled-authentication attack it cites as the reason a nonce challenge inside an
  unauthenticated tunnel is not equivalent. R-SOV-3.1 adopts that reasoning rather than restating it.

A 2026-07 snapshot; re-check before relying on any external claim above.
