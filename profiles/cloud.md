# DEPOT — managed infrastructure services (the decentralised-cloud profile)

> **Status:** profile spec (KOTVA family), **draft — normative once ratified**. Provisional name
> (**DEPOT** — a supply depot where infrastructure is provisioned and dispensed); the codename is a
> founder call. DEPOT is **thin by construction**: it defines **no new runtime** and **no economics**.
> It reuses the coordinator contract, the [§18.8a](../18-wire-format.md) descriptor / tariff /
> usage-receipt seam, PUB feeds ([§22](../22-public-objects.md)), and the ATTEST primitive, and it
> **adopts** existing engines (WASI, OCI, S3, RESP/Postgres wire, HTTP caching, cloud-init). Its whole
> job is to make managed infrastructure **accountable, swappable, blind where the data allows and
> honestly `declared` where it does not** — so a "gateway" can be a decentralised cloud without
> becoming a captor.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD
NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in
BCP 14 (RFC 2119, RFC 8174) when, and only when, in all capitals.

---

## 1. What this is

> **Terminology — "gateway" here is the colloquial word for a DEPOT operator, not the `gateway`
> coordinator kind.** This profile uses *gateway* in its everyday sense — a provider you can reach that
> offers managed infrastructure — because that is how operators describe themselves. It is **not** the
> `gateway` coordinator kind of [CONTRACT §5](../coordinator/CONTRACT.md) (the legacy-mail bridge: MX,
> DKIM egress, §7). A DEPOT operator is an **`infra-service`** coordinator. One party MAY run both roles,
> but they are distinct kinds with distinct requirements, exactly as [§26](../26-legacy-adapters.md)
> guards the same word for its own adapters.

A market where any operator — a **gateway** — offers managed infrastructure to a user who holds their
own keys and can leave: the four irreducible primitives — a **box** (a managed node), a **bucket**
(object storage, which also serves public objects at the edge — the CDN shape), a **volume** (block
storage) and an **edge-fn** (serverless compute) — plus everything that **composes** from them as a
formula: a **database** (Redis / Postgres), a **queue**, and the rest (§3.2). It is the Hetzner / AWS / ngrok
product shape rebuilt as **fenced coordinators** ([coordinator/CONTRACT.md](../coordinator/CONTRACT.md)):
each service is `accountable`, `swappable`, `self-hostable`, and **never load-bearing** — reach and
convenience, never a gate on the user's function or identity.

DEPOT adds **operations, not protocol**: it mints no new cryptography, no runtime, and no price model.
It is the thin contract over engines that already exist, plus the one honesty rule that keeps
"managed" from quietly meaning "captured" (§4 DEPOT-2).

---

## 2. Primitives, coordinators, and bindings it composes

DEPOT is composition, not new machinery. It reuses:

| Composed with | Role in DEPOT | Home |
|---|---|---|
| **`infra-service`** coordinator kind | provides one managed service, fenced by the four clauses; declares `{service, visibility, metering-unit}`. The one load-bearing new binding. | [CONTRACT §5](../coordinator/CONTRACT.md) |
| **`CoordinatorDescriptor` / `Tariff` / `UsageReceipt`** | the signed offer, the operator's own price, and the metered receipt — the accountable, self-asserted, discovery-only seam. | [§18.8a](../18-wire-format.md) |
| **Economics seam** (CONTRACT §6) | settlement over an existing asset, **no token, no published price-rank**; the *numbers* are operator policy. | [CONTRACT §6](../coordinator/CONTRACT.md) |
| **PUB** (feeds & blobs) + **ATTEST** + the **indexer/labeler** role | distributed reputation — signed measurement **claims** (ATTEST attestations, §5) anyone may publish and anyone may aggregate; DEPOT mints no reputation object of its own. | [§22](../22-public-objects.md), [primitives/ATTEST.md](../primitives/ATTEST.md) |
| **Identity** + **`RecoveryPolicy`** | non-custody (the root `IK` never leaves the user's device) + guardian recovery — **not** key escrow. | [§1](../01-identity.md), [§1.4](../01-identity.md) |
| **IK-authenticated Noise transport** | the box↔service control/data channel is a libp2p-Noise `XX` stream keyed to the user's `IK` (as REACH-2), not a bearer token. | [profiles/reachability.md REACH-2](reachability.md) |
| **SYNC** / **RESERVE** | the coordination half of a composed formula (§3.2) — SYNC for a signed CRDT the operator cannot read, RESERVE for a single-writer claim; a `queue` formula's ordering rides these, not a rented broker. | [substrate/SYNC.md](../substrate/SYNC.md), [primitives/RESERVE.md](../primitives/RESERVE.md) |

Bindings **adopted rather than reinvented** ([bindings/README.md](../bindings/README.md)): **WASI** /
**OCI** (edge-fn runtime), **S3 API** / content-addressing **and HTTP caching** (bucket, including
its public-object/edge mode), **Redis RESP** / **Postgres wire** (the `database` formula, §3.2), **cloud-init** / any OS (box). DEPOT specifies **no** new
runtime, storage format, or price model.

---

## 3. The service registry (extensible — the future-proof surface)

The specific **primitives** are a **registry**, not spec text: a genuinely new primitive (say a GPU
fabric that is not just a `box` attribute) is a **registry row**, never a spec change — while anything
that composes from existing primitives (a message queue, a database, a search index) is a **formula**
(§3.2), not a row. Each row fixes the honest defaults the
profile enforces. v0 registry:

| `service` | Adopt (native protocol) | **Honest visibility** (DEPOT-2) | Metering unit (example — operator sets the number) | Portability (DEPOT-4) |
|---|---|---|---|---|
| `bucket` | S3 API / CID content-addressing; **HTTP caching** when serving public objects | **`blind`** — `structural` **only for objects the client actually encrypted** (S3 takes arbitrary bytes, so plaintext handed to it is readable and the declaration stays truthful — CONTRACT §3.3), else `declared`; **`blind-routing`** when it serves *public* objects, seeing which and when but never a private payload | GB-month + egress-GB + requests | **zero-migration** (re-pin elsewhere) |
| `edge-fn` | WASI / OCI | **`terminating`** (runs your code, sees I/O) → **`attested`** in a TEE | CPU-ms + invocations | **zero-migration** (redeploy the artefact) |
| `volume` | Block device (virtio-blk / NVMe-oF / iSCSI); guest-owned filesystem | **`blind`** at `structural` **only where the guest actually encrypted** (LUKS/dm-crypt — the operator then holds ciphertext blocks plus access patterns, never plaintext); an unencrypted volume is **`terminating` / `declared`**, and the operator cannot tell which it was given (CONTRACT §3.3) | GB-month (+ provisioned IOPS) | **export/import** across operators; *detach/reattach* within one operator iff `attachment = detachable` |
| `box` | OS + node (cloud-init) | **`terminating` / `declared`** — the operator has root on the host | instance-hour | **export** (keys stay with the user; data dumps out) |

### 3.1 Declared capacity — how a small operator competes honestly

An `infra-service` descriptor's `policy` field ([§18.8a.1](../18-wire-format.md)) is an **opaque,
kind-specific det_cbor blob**. DEPOT fixes its shape for this kind, so a **new limit is a DEPOT
registry change, never a §18 wire change**. It carries what a client must know *before* committing,
which the after-the-fact measurements of §5 cannot supply:

```cddl
DepotServicePolicy = {          ; det_cbor `policy` blob for kind = "infra-service"
  1 => tstr,                    ; service     a §3 registry value ("bucket", "volume", …)
  ? 2 => Capacity,              ; capacity    declared ceilings — absent means UNDECLARED, never unlimited
  ? 3 => { * tstr => tstr },    ; attributes  service-specific, e.g. volume {persistence, attachment}
}
Capacity = {                    ; every value a uint — no floats (§18.1)
  ? 1 => uint,                  ; total_bytes        storage ceiling
  ? 2 => uint,                  ; max_object_bytes   largest single object/volume
  ? 3 => uint,                  ; egress_bps         sustained throughput ceiling
  ? 4 => uint,                  ; max_concurrent     concurrent streams / instances
  ? 5 => tstr,                  ; class              "cold" / "warm" / "commit-path" (latency tier, §3)
  ? 6 => uint,                  ; uptime_target      per-mille intent (0…1000) — an aim, NOT a promise
  ? 7 => { * tstr => uint },    ; resources          OPEN quantity vocabulary (machine shape, below) — never a closed enum
}
```

**Machine shape (`resources`) — an open vocabulary, not a product list.** A GPU is **not a service**:
a box with an accelerator is still a `box` — same `terminating` visibility, same operator-has-root
posture, same export portability. Making `gpu` a registry row would be exactly the catalogue-thinking
this section forbids. What a client needs is *how much of what*, and that is a **map of
`resource-name → uint quantity`** whose key namespace is **open and registry-extensible**, so a new
accelerator class is a registry name and **never a spec change**:

| Resource key | Unit | Notes |
|---|---|---|
| `cpu-millicores` | thousandths of a core | 1000 = one core; integer, so no float creeps in (§18.1) |
| `mem-bytes` | bytes | |
| `gpu-count` | whole devices | a fractional GPU is a scheduling fiction; count devices |
| `gpu-mem-bytes` | bytes | the axis that actually decides whether a model fits |
| `accel-<class>` | vendor/registry-defined | the extension point — tensor/NPU/FPGA/whatever is next enters here, never as a new service |

**`arch` is an attribute, not a resource — and the distinction is load-bearing.** Architecture
(`x86_64`, `aarch64`, …) belongs in `attributes` (§3.1), because it is a **compatibility predicate**:
you match it or you do not, and there is no such thing as half an ARM. Resources are **quantities**:
more or less. Conflating the two is a common and expensive design error — a client **filters** on
`arch` (and any other predicate), then **compares** on `resources`. Declaring an architecture it
cannot actually serve is the same `capacity-conformance` falsehood as overstating bytes (§5).

Quota separation follows for free: because `resources` is one vocabulary, a declared ceiling and an
enforced per-user quota are the **same keys** — `cpu-millicores` and `gpu-count` are metered and
limited independently without the protocol naming a single number for either (CONTRACT §6).

**This is a declaration, not a promise, and the difference is the whole point.** A home operator with
2 TB and a 50 Mbit uplink can say exactly that and be chosen for the work it can actually carry,
instead of competing on an undifferentiated axis against a datacentre and losing. Three rules keep it
honest:

- **Absent means undeclared, never unlimited.** A client MUST NOT infer a ceiling from an omitted
  field, and an operator MUST NOT read an omission as permission to refuse arbitrarily.
- **`uptime_target` is an intent, not an SLA.** It is the operator's aim; what it *achieved* is the
  §5 `uptime` measurement, published by observers, not by the operator. A consumer MUST NOT treat the
  target as evidence.
- **A declared ceiling is falsifiable.** Because the descriptor is signed and the §5 `metric`
  vocabulary already carries `capacity-conformance`, an operator that advertises 2 TB and refuses at
  1 TB is **detectably** overstating — the same declare-then-measure loop that makes `visibility`
  honest (DEPOT-2). Overstating capacity is non-conformant, not merely rude — but **falsifiable is not
  the same as cheaply falsified**: testing a ceiling costs what the ceiling claims, so this deters a
  careless operator far better than a patient one (§7).

Quotas the operator *enforces* per user at runtime (rate limits, storage caps, `0x070D`, `0x0806`)
remain **operator policy** and are deliberately not fixed here: the protocol standardises the
*declaration* so a client can choose, never the *number*.

**Completeness, not catalogue.** This set is deliberately small and is chosen to *span* what a
centralised platform does, not to mirror a product list: run code (`edge-fn`, `box`), store blobs
and serve them at the edge (`bucket`) — with queryable state (`database`) and asynchronous decoupling
(`queue`) composed as formulas (§3.2), not rows — plus public reachability ([REACH](reachability.md)), identity and login (§13),
messaging and wake (§2, §4.9), real-time (§27, §25), and the control plane of DEPOT-11. A capability
that is merely a *product* built from these MUST be a product, not a registry row.

**Why `volume` is a row and not a `box` attribute — and the three cases it must cover.** A `volume`
is not a slow `bucket`; it is the **hot tier**, and the gap is physical rather than an engineering
debt that will close. A durable commit needs an `fsync` in *tens of microseconds* on local NVMe,
where networked storage is in *milliseconds* and object storage further still — which is why the
most advanced diskless designs keep object storage **off** the commit path entirely and retain a
small low-latency write-ahead tier in front of it. A profile that offered only `bucket` could not
host a correct database at all. Two declared attributes cover the deployment shapes, and a client
MUST NOT assume either:

- **`persistence`** — `ephemeral` (dies with the box; the instance-local disk a `box` already
  includes in its instance-hour, not separately rented) or `durable` (outlives box termination).
- **`attachment`** — `bound` (this box only; cannot move — typically instance-local NVMe, the
  fastest and the commit-path case), `detachable` (datacentre-style network block: may be detached
  and reattached to **another box at the same operator**, so state outlives a dead box), or
  `shared` (multi-attach by several boxes at once — which needs a cluster filesystem or external
  coordination, and whose consistency is the guest's problem, never the operator's promise).

**Honest limit — detachable is not portable.** `detachable` buys resilience *within* one operator: a
box dies, the volume reattaches elsewhere in that operator's fleet. It buys **nothing** against
operator lock-in, because a network volume is operator-local: moving it to a different operator is a
copy, an export/import, exactly as for `box`. The services that genuinely reduce state lock-in are
the content-addressed `bucket` (re-pin anywhere) and [SYNC](../substrate/SYNC.md) (the user's own
devices hold the state). An operator MUST NOT present detachability as though it satisfied DEPOT-4
swappability.

**Why there is no `cdn` row — it folded into `bucket`.** A CDN and an object store are the *same
mechanism*: retain content-addressed bytes and serve them on request. In a content-addressed system
there is no origin-versus-edge distinction to encode — every holder is equivalent and every byte is
verified by hash, so "the CDN" is simply **a copy that is closer**. What genuinely differs is not the
service but the **content class**, and the visibility follows from it: objects the client encrypted
leave the operator `blind`; public objects it must serve on demand leave it `blind-routing`, seeing
which and when but never a private payload. Encoding that as one row with a derived visibility is
honest; encoding it as two rows implied a mechanism split that does not exist. Locality and
edge-placement are an operator's **policy and tariff**, not a separate service.

### 3.2 Formulas — the services that are compositions, not primitives

The four rows above are **irreducible**: bytes cold or hot (`bucket`/`volume`), code stateless or
stateful (`edge-fn`/`box`). Everything a centralised platform sells beyond them — a database, a
managed queue, a Redis, a CDN, a static site — is a **formula**: a recipe that composes the four,
never a new mechanism. This is the profile's own "completeness, not catalogue" rule turned on itself,
and the reason DEPOT does not grow a row per product. A formula is a **signed, content-addressed PUB
object** ([§22](../22-public-objects.md)) anyone may publish, fork and compete on; the protocol never
learns what "Postgres" is.

The two most-asked-for services are worked examples, and folding them here rather than giving each a
row is deliberate — a row would imply a mechanism that is not there:

- **`database` = `box` + `volume` + `bucket`.** A managed database *is* a query engine on a box, its
  WAL and hot pages on a fast `volume` (the commit path physics of §3.1), its archive/PITR on a
  `bucket`. DEPOT defines **no** query language, schema, replication protocol or consistency model —
  it adopts the existing wire protocols (Postgres, RESP) and stops. The visibility is not lost by
  dissolving the row: the `box` in the formula is `terminating`, so "the operator can read your data
  to answer a query" is declared by the box it runs on. **A Redis is the same formula minus the
  bucket** — a process on a box, with an optional `volume` for AOF/RDB persistence, or none at all
  for a pure cache.

- **`queue` = `bucket` + a claim mechanism.** The durable half of a queue is object storage —
  [WarpStream](https://www.warpstream.com/) and AutoMQ run fully Kafka-compatible queues with zero
  local disk, streaming straight to and from S3. What remains is ordering and exactly-once claim,
  which is [SYNC](../substrate/SYNC.md) (a signed CRDT the operator cannot read) or
  [RESERVE](../primitives/RESERVE.md) (a single-writer claim) — **not** a rented broker. The folded
  queue is therefore *capable of being* more private than a broker: the payload sits in a bucket that
  is `blind` **exactly to the extent the client encrypted it** (the same conditional rule as any
  bucket, §3.3 — it is not blind by virtue of being called a queue), and the claim is a CRDT among
  the consumers themselves, needing no trusted operator at all. A broker sees plaintext by
  construction; this fold *lets* a client keep the operator out, it does not do it for them.

**The native alternative comes first, for both.** For application state a user's own devices own, a
local-first signed CRDT ([SYNC](../substrate/SYNC.md)) needs no rented database and no trusted
operator: the box holds the state, SYNC converges it across devices, a `bucket` backs it up under
client-side encryption, and no operator reads it, ever. The `database` formula is the
**compatibility bridge** for software that already speaks SQL or RESP, bought at the price of a
`terminating` box. Reach for SYNC first; compose a `database` when an existing engine is the
requirement.

**A formula's visibility and portability are inherited, not declared (normative).** A formula has no
honesty properties of its own; it takes the **least-blind** visibility and the **least-portable**
class of its parts. The `database` formula is `terminating` because its `box` is, and export/import
because its `box`/`volume` are; the `queue` formula is **at best** `blind`, inheriting the bucket's
*conditional* blindness (`structural` only for what the client encrypted, else `declared` — §3.3),
and its claim CRDT is unreadable. So the DEPOT-2 and DEPOT-4 clauses below enumerate the **four primitives**, and a
formula is bound by the composition of the primitives it names — there is nothing extra to declare and
no way for a formula to be more blind or more portable than its most-exposed, most-stuck part.

**How a formula is actually encoded (normative — it is a named recipe, not a coordinator).** A
formula adds **no** `service` value and **no** coordinator kind: the four `infra-service` primitives
stay exactly four, and their `service` field ([§3.1](#31-declared-capacity-how-a-small-operator-competes-honestly))
stays one of `bucket`/`volume`/`edge-fn`/`box`. A formula is instead a **named schema over a
content-addressed PUB object**, the same shape `DepotSite` uses above — no new wire object, DS-tag or
error code:

```cddl
DepotFormula = {                          ; "kotva-depot/formula/v0"
  1 => tstr,                              ; kind        formula identity, e.g. "postgres" / "redis" / "kafka-queue"
  2 => [+ Part],                          ; parts       the primitive coordinators it composes (>= 1)
  ? 3 => bytes,                           ; recipe      opaque det_cbor provisioning/wiring, engine-defined
  ? 4 => tstr,                            ; consensus   what provides coordination if it scales (see below); absent = single-writer
}
Part = {
  1 => tstr,                              ; service     one of the four §3 primitives
  2 => ik-pub,                            ; provider    the infra-service coordinator supplying this part
  ? 3 => hash,                            ; descriptor  content address of that provider's CoordinatorDescriptor (§18.8a.1)
}
```

This is what makes the inheritance rule *computable* rather than rhetorical: a client reads a
`DepotFormula`, resolves each `Part.provider`'s own signed descriptor, and derives the formula's
visibility and portability as the least-blind and least-portable across the parts — there is a
concrete object to check, not a promise. A formula whose parts span **different operators** is
legitimate and is exactly how a user avoids one operator holding the whole database; a formula all of
whose parts name **one** provider is a single-operator managed offering, and the client can see which
from the `provider` keys. Publishing a `DepotFormula` is permissionless (it is a PUB object); competing
formulas for the same `kind` are the market, and the protocol never learns what "postgres" means.

**Honest limit — a formula composes storage, never consensus (normative for any "scaling" claim).**
`box` + `volume` + `bucket` gives the *ingredients* of a scalable database, never the *coordination*.
Two boxes cannot share one `volume` (attachment is exclusive; `shared` pushes the consistency problem
onto the guest, §3.1); two boxes *can* share a `bucket`, but then which is primary, and what orders
the writes? That is consensus, and no engine gets it from object storage for free — Neon had to build
safekeepers, Aurora a bespoke storage layer. DEPOT cannot supply consensus and does not pretend to. A
formula that advertises "scales across boxes" MUST populate `DepotFormula.consensus` with what
provides the coordination (a consensus protocol, a single-writer lease, an external quorum); an absent
`consensus` field means single-writer, and such a formula MUST NOT advertise horizontal scaling —
absent that, it is a single-writer database with
extra parts, and MUST be described as one. This is the same stateful/stateless asymmetry DEPOT-13
discloses: content-addressed `bucket` bytes replicate freely, single-writer state does not.

**Scaling, autoscaling and load balancing are deliberately out of scope — with one exception.** How
many instances an operator runs, and how it spreads load across its own machines, is **operator
policy** ([CONTRACT §6](../coordinator/CONTRACT.md)): the protocol fixes the *seam*, never the
numbers, and a DEPOT that specified scaling policy would be prescribing an implementation. What *is*
in scope is the decentralised form of the same need — **choosing and failing over between
operators**: signed `CoordinatorDescriptor`s to discover them, published measurements (§5) to
compare them, and DEPOT-4 swappability to leave. That composition, not an elastic-group API, is this
profile's load balancer. **The honest ceiling is the stateless/stateful asymmetry**, not the absence
of an autoscaler: `bucket` and `edge-fn` spread across operators freely *because* they are
zero-migration, while `volume`, `box`, and any stateful formula built on them (a `database`) carry state that must be exported and re-imported to
move. Adding providers is cheap for the former and a migration for the latter, and no protocol rule
changes that.

**Worked example — static-site / SPA hosting is a product, not a service.** A static site is already
what PUB ([§22](../22-public-objects.md)) is: signed, content-addressed, self-verifying public objects
servable over plain HTTPS. It composes as **PUB objects in a public-serving `bucket`, named via
[REACH](reachability.md)** (own domain or vanity, certificates per REACH-2a) — and a deploy is simply
publishing a new content-addressed root plus a signed announcement superseding the previous one, which
makes the switch **atomic** and makes **rollback** a pointer back to a root that is still addressable.
It adds **no** registry row and **no** coordinator kind. What such a site DOES need, purely to stay
portable between providers (DEPOT-4), is one named schema — `kotva-depot/site/v0` — pinning serving
behaviour as a **deterministic-CBOR map** (§18.1) so any provider serves the same site identically:

```cddl
DepotSite = {                                    ; "kotva-depot/site/v0"
  1 => hash,                                     ; root      content address of the site root manifest (§22)
  ? 2 => tstr,                                   ; fallback  SPA fallback path, e.g. "/index.html"
  ? 3 => [* {1 => tstr, 2 => tstr, 3 => uint}],  ; redirects { from, to, status }
  ? 4 => {? 1 => uint, ? 2 => bool},             ; cache     { max_age_s, immutable }
}
```

  A provider MUST serve `root` as the site, apply `redirects` in array order, and — for a path that
  resolves to no object — serve `fallback` when present or return 404 when absent (never a guess). Without it each
operator invents its own hosting config and the site stops being swappable; with it, any `bucket` or box
serves the same site identically. It is a **schema over a content-addressed object** — no new wire
object, DS-tag, or error code — exactly like the measurement schema of §5. Deploy pipelines are
authorised by scoped `CapabilityToken`s under DEPOT-11 (a CI credential is strictly narrower than its
parent), so a leaked deploy token can publish a site and **cannot** reach mail or identity.

**Triggers, not more services.** Time-based and event-based invocation are **trigger types on
`edge-fn`**, never separate services: `http` (via REACH ingress), `cron` (a schedule), `queue` (an item arriving on a `queue` formula,
§3.2), and `webhook` (an inbound HTTPS event routed to a box or function through REACH,
buffered in a `queue` formula when the target is offline). A new trigger is an enum value; it is **not** a
spec change and **not** a new coordinator kind.

Only **`bucket`** and **`volume`** keep the payload cryptographically out of the operator's reach —
**and only for the objects the client actually encrypted** (the `queue` formula inherits this from
the `bucket` it composes, §3.2). Both accept arbitrary bytes, so their blindness is the *client's* property, not the operator's architecture: hand
any of them plaintext and it is readable, while the operator's declaration remains truthful
(CONTRACT §3.3). A deployment wanting unconditional blindness MUST make client-side encryption
non-optional on ingest. **`edge-fn`, `box`, and an
unencrypted `volume` are `terminating`** — the operator sees your data or computation. This is normal and honest
(Fastmail-tier trust); it is **not** cryptographic blindness, and DEPOT-2 forbids pretending otherwise.

---

## 4. Normative profile rules

- **DEPOT-1 — one contract, adopted protocol.** An `infra-service` coordinator MUST publish a signed
  `CoordinatorDescriptor` (§18.8a) carrying `{service` (from §3), `visibility, metering-unit}` and MUST
  speak that service's **adopted native protocol** (§2) over an **`IK`-authenticated, Noise-secured**
  channel (REACH-2 shape — the user's `IK` is the libp2p identity key; **no bearer token**). It mints
  no new runtime, wire object, DS-tag, or error code — reputation reuses the ATTEST claim primitive (§5).
- **DEPOT-2 — honest visibility is load-bearing (the cliff).** Each service MUST declare **exactly**
  the visibility its data model permits (§3): `bucket` `blind` — `structural` **only for what the
  client encrypted**, `declared` otherwise (CONTRACT §3.3) — or public-serving `blind-routing`;
  `volume` `blind` when the guest encrypts it, else `terminating`/`declared`;
  `edge-fn`/`box` `terminating`/`declared`; a **formula** inherits the least-blind of its parts (§3.2). **Advertising a `terminating`
  service as `blind`, `private`, or `sovereign` is non-conformant misrepresentation**
  ([CONTRACT §3.2](../coordinator/CONTRACT.md)), not marketing. A TEE with **verifiable remote
  attestation** MAY raise `edge-fn`/`box` from `declared` to `attested`; the attestation
  MUST be checkable by the client, or the claim reverts to `declared`.
- **DEPOT-3 — non-custody, no key escrow.** The user's **root `IK` is generated and held on the
  user's own device**; a managed `box` receives only a **revocable `DeviceCert` subkey**
  ([§1.2](../01-identity.md)). No `infra-service` MUST ever hold or be able to use the root `IK`, and
  **operator-held key backup is FORBIDDEN** — it makes a swappable coordinator load-bearing (the party
  you may need to leave holds the means to leave). "I forgot my key" is answered by **guardian / social
  recovery** ([§1.4](../01-identity.md)): an operator MAY be **one guardian of a quorum**, never
  sufficient alone.
- **DEPOT-4 — swappable, honest portability.** Leaving or switching an `infra-service` MUST be a
  **config change with zero identity change** ([CONTRACT §2.2](../coordinator/CONTRACT.md)). Each
  service MUST state its **true** portability (§3): content-addressed `bucket` and stateless
  `edge-fn` are **zero-migration**; stateful `volume`/`box` MUST provide a **portable
  export/import**, and MUST NOT be advertised as zero-migration. A `detachable` volume moves between
  boxes of **one** operator and MUST NOT be advertised as zero-migration on that basis (§3).
  **"Portable" means format-portable, and downtime is an acceptable price:** the export MUST be in the
  **adopted standard's own interchange format** for that service — S3-API objects for `bucket`, a
  standard block image or filesystem dump for `volume`, the engine's native dump (`pg_dump`-class,
  RESP) for a `database` formula, an OCI/WASI artefact for `edge-fn`, a standard disk image for `box` — such
  that **any conformant operator of the same service can ingest it without the exporting operator's
  cooperation**. An export only its author's tooling can read is **not** an export and MUST NOT be
  advertised as satisfying this clause. The exit this profile guarantees is *interoperable*, not
  *seamless*: a migration MAY cost real downtime, and that is an acceptable price for being able to
  leave at all — what is never acceptable is a format that makes leaving impossible. A slow or lossy export is a weaker
  exit and MUST be disclosed as such (§7).
- **DEPOT-5 — economics are the operator's; KOTVA specifies only the seam.** Prices, price model
  (per-unit / flat / tiered / spot), billing cycle, free tier, SLA, discounts, and settlement asset are
  **operator policy**, carried in the signed `Tariff`/policy as bytes KOTVA does not inspect. KOTVA
  requires **only**: the tariff is **signed and discoverable** (accountable); usage is metered into
  **signed `UsageReceipt`s delivered to the payer**; settlement is over an **existing asset**; there is
  **no protocol token** and **no published global price-rank** ([CONTRACT §6](../coordinator/CONTRACT.md)).
  Two operators MAY run entirely different economics and both be conformant — KOTVA guarantees a price
  *exists, is signed, and is metered honestly*; it never says what the price *is*.
- **DEPOT-6 — subcontracting stays accountable, never launders visibility.** An operator MAY fulfil a
  service through a third party (a CDN edge, an email relay, a rented cloud) but **remains the sole
  accountable, declaring party**. It MUST NOT launder visibility: a subcontracted `terminating` leg is
  still `terminating` and MUST be declared so — an operator MUST NOT claim `blind` by pointing at a
  subcontractor. The user holds the **declaring operator** accountable, not its supplier.
- **DEPOT-7 — authorise, never classify.** An `infra-service` gates admission on **identity + rate +
  payment** only ([CONTRACT §4](../coordinator/CONTRACT.md)); it MUST NOT admit, refuse, throttle, or
  price on a **content judgement**. Metering measures **resource use**, never content. (A service that
  *must* read content to function — an `edge-fn`, or the `box` inside a `database` formula — does so under its declared `terminating`
  visibility, never as a content gate on delivery.)
- **DEPOT-8 — fail-closed.** An unpaid, expired, unauthenticated, or over-quota request MUST fail
  closed — a clean refusal or connection close ([§21](../21-errors-iana.md) FAIL_CLOSED_BLOCK), never a
  silent best-effort, a partial charge, or a content-based drop.
- **DEPOT-9 — distributed reputation, no authority.** Service quality is a **market of signed
  measurements**, never a single authoritative score — reputation is measured locally by each client
  ([CONTRACT §3.1](../coordinator/CONTRACT.md)). A measurement is an **ATTEST claim** (§5) — a signed,
  timestamped observation about a `(coordinator, service)` — uptime, a conformance-vector pass, an
  honest-visibility audit, latency — published via the ATTEST **public** carrier on a **PUB feed**
  (append-only, signed, content-addressed, [§22](../22-public-objects.md)). A **status page is a REPRODUCIBLE aggregation** of
  such feeds; a client chooses which raters to weight. Automated measurements **SHOULD be reproducible**
  (anyone re-runs the probe or vector), so trust rests on re-checkable evidence, not the rater's word —
  **reproducibility over reputation**. A measurement is **attributed to its signing rater**; a
  **self-measurement** (rater `IK` == the rated coordinator) MUST be presentable as such and weighted
  accordingly. **Any party MAY run a rater** — an operator rating itself or its competitors, or the
  software maintainer running a well-known one — but **none is authoritative** and none MAY be presented
  as *the* score. A rater is the [`labeler`/`indexer`](../coordinator/CONTRACT.md) role; running one
  alongside a gateway is one operator serving two separable, attributable roles.
- **DEPOT-10 — self-host backstop + disclosed scarcity.** Anyone with the resource MAY run any
  `infra-service` for themselves ([CONTRACT §2.3](../coordinator/CONTRACT.md)). The honest exceptions,
  disclosed not papered over, are the same fenced ones: a **reputable public IP / ingress** and **real
  compute, storage, and bandwidth** are resources a host or ISP allocates, not conjured — confined to
  this kind, never a protocol chokepoint (the port-25 / REACH-9 analog, generalised).
- **DEPOT-11 — the control plane is a capability, not an API key.** Provisioning, configuring,
  scaling, and destroying an `infra-service` — the operator's API/CLI — MUST be authorised by a
  **`CapabilityToken`** ([§18.7.3](../18-wire-format.md)): scoped by `resource`/`ability`, attenuable,
  delegable, offline-verifiable, and revocable. It MUST NOT be a bearer API key or an unscoped account
  password, and DEPOT mints no control-plane token of its own. Two consequences are normative: a
  delegated token (a deploy key, a CI credential, a teammate's grant) is **strictly narrower** than its
  parent, because every caveat on **every** link of the chain is evaluated and an unrecognised caveat
  fails closed (§18.7.3); and a capability that can act on the user's **mail or identity** MUST be
  scoped separately from one that acts on infrastructure — provisioning a box MUST NOT implicitly
  grant reading a mailbox.
- **DEPOT-12 — secrets are sealed to the box, never held in operator plaintext.** Configuration
  secrets an `infra-service` stores on a user's behalf (environment values, credentials, connection
  strings) MUST be **encrypted to the box's device key by the client before they leave it**; an
  operator MUST NOT **require** plaintext to operate the service, and one that accepts or stores
  plaintext secrets MUST declare that surface `terminating` (DEPOT-2) rather than implying the
  secrets are protected. Where a service genuinely needs the value in the clear at runtime (an env
  var inside a `terminating` `edge-fn` or `box`), that exposure is bounded by, and disclosed under,
  that service's already-declared visibility — never presented as protected.
  **Honest residual — this is a `declared` property, not a structural one.** Unlike DEPOT-3, where
  the root `IK` is withheld *by construction* (only a revocable subkey ever leaves the device), DEPOT
  defines **no secret-envelope object and no verification step**: services speak their adopted native
  protocols (DEPOT-1), so nothing on the wire distinguishes a sealed blob from a pasted plaintext, and
  an operator that documents "paste your secret here" cannot be refused by the protocol. The
  enforceable half is the **client's** obligation to seal first; the operator's half is **detectable,
  not preventable** — via an ATTEST `visibility-audit` measurement (§5) and the exit (DEPOT-4), the
  same accountability the `declared` assurance level carries everywhere else. Do not read DEPOT-12 as
  a cryptographic guarantee against a dishonest operator.
- **DEPOT-13 — permissionless supply; durability comes from plurality, not from an SLA.** Any node MAY
  offer any `infra-service`, including a single self-hosted box contributing spare capacity: the
  open-role principle of [Roles & Wake](../substrate/ROLES.md) and the self-host clause
  ([CONTRACT §2.3](../coordinator/CONTRACT.md)) apply to this kind unchanged. Joining is **publishing a
  signed descriptor** (§18.8a); standing is **earned through measurement claims** (§5), never granted
  by a gatekeeper. Because no single small provider can match a hyperscaler's availability, a client
  obtains durability and availability by **using several independent providers**, not by trusting one —
  and content-addressed `bucket` bytes replicate freely — including the durable half of a `queue`
  formula, which is a bucket — so plurality is cheap and re-pinning is zero-migration (DEPOT-4).
  **Honest asymmetry:** the stateful primitives `volume` and `box`, and any formula built on them (a
  `database`), do **NOT** replicate freely — they carry single-writer state whose portability is
  an export/import, so for those a client's real protections are that export plus the operator's
  declared visibility, not replication. A `detachable` volume is **not** a counter-example: it moves
  between boxes of one operator, never between operators. A profile MUST NOT present multi-provider replication as though it made
  a stateful service as durable as a content-addressed one.

---

## 5. Measurements are ATTEST claims — no new wire object

Reputation reuses the substrate's generic **signed-claim** primitive: a service measurement is an
**ATTEST** public `Attestation` ([primitives/ATTEST.md](../primitives/ATTEST.md) — whose §1 names "a
rating" as one of its shapes), **not** a bespoke DEPOT object. DEPOT mints **no new wire object, DS-tag,
or signature** for reputation — it defines only a **claim schema** carried inside ATTEST:

- **Carrier** — the ATTEST **public** carrier: `det_cbor(Attestation)` embedded in a PUB feed / manifest
  ([§22](../22-public-objects.md)), so anyone may publish and anyone may aggregate; the carrier's own
  signature authenticates it (ATTEST §2).
- **`issuer`** — the rater's `IK`. A **self-measurement** is exactly `issuer == subject`, surfaced by
  ATTEST and weighted accordingly by consumers.
- **`subject`** — the rated coordinator's `IK`.
- **`schema`** — the DEPOT measurement schema `kotva-depot/measurement/v0`. Its `claim` body is a
  **deterministic-CBOR map** (§18.1 — integer keys, no floats, absent ≠ null), pinned here so two
  raters and two aggregators agree byte-for-byte:

```cddl
DepotMeasurement = {                ; claim body for schema "kotva-depot/measurement/v0"
  1 => tstr,                        ; service      a §3 primitive ("bucket", "volume", "box", "edge-fn")
  2 => tstr,                        ; metric       "uptime" / "conformance" / "visibility-audit" / "latency-ms" / "capacity-conformance" / "export-conformance"
  3 => uint / bool,                 ; value        metric-typed, below — never a float (§18.1)
  4 => tstr,                        ; method       "probe" / "conformance-vector" / "audit" / "self-report"
  5 => ts,                          ; observed_at  ms since the Unix epoch (§18.1)
  ? 6 => { 1 => tstr, 2 => tstr },  ; evidence     { kind: "recipe"/"vector-id"/"transcript", ref }
}
```

  `value` is typed **by `metric`**, with no float anywhere: `uptime` = `uint` **per-mille** availability
  (`0…1000`); `latency-ms` = `uint` milliseconds; `conformance`, `visibility-audit`,
  `capacity-conformance` and `export-conformance` = `bool`. `capacity-conformance` records whether the
  operator honoured the ceilings its own signed `DepotServicePolicy` declared (§3.1);
  `export-conformance` records whether a DEPOT-4 export actually round-tripped into a *different*
  operator of the same service. Both exist so a declaration is falsifiable rather than marketing —
  with the honest limits stated in §7, because both are far more expensive to test than a signature
  is to verify. An
  unrecognised `metric` MUST be ignored by aggregators (never guessed at). Representing the same claim
  as an EAS attestation or W3C VC for consumers outside KOTVA is a **binding-layer mapping**
  ([bindings/README.md](../bindings/README.md)) and is out of scope for `v0`; pinning that external
  mapping is a ratification item, not an interoperability blocker inside the family.

A consumer verifies the ATTEST carrier signature against `issuer` and treats `issuer == subject` as a
self-measurement. Measurements are an **append-only time-series** on the rater's feed (§22.4.2): a newer
observation does **NOT** supersede an older one — the history is exactly what reputation aggregates over
(uptime across a window, a latency distribution), so raw observations MUST NOT be collapsed to a
latest-only value. A rater MAY **revoke** a measurement it retracts (ATTEST `Revoke`, §2.2), and an
issuer that signs two contradictory claims at one feed position is **detectably equivocating** (ATTEST
§4.3), surfaced for dispute, never merged away. A consumer SHOULD **re-run** any `method` =
`probe`/`conformance-vector` whose `evidence` supplies a reproducible recipe rather than trusting the
reported `value`. A malformed
or unverifiable measurement is simply **ignored by aggregators** — never a fail-closed event, and **no
new error code**. New metrics or methods are **new schema versions**, not spec changes — future-proof by
schema, not by wire.

---

## 6. Security + declared content-visibility

Inheriting [THREAT-MODEL.md](../THREAT-MODEL.md) (SEC-1…SEC-9); the DEPOT-specific posture is the
**cliff of §4 DEPOT-2**, restated for clarity:

- **`bucket` and `volume` are structurally private only for what the client encrypted** — and a
  `queue` formula inherits it from its bucket.
  This is the sharpest self-deception risk in the profile: the label reads like an operator guarantee
  and is actually a statement about the client's own discipline, and the failure is silent — a
  misconfigured SDK or a plain `cp` loses the whole protection without the operator lying or anything
  erroring (CONTRACT §3.3). They hold client-encrypted,
  content-addressed data — or, for a `queue` formula, client-encrypted payloads in a bucket whose depth, rate and timing are
  visible but whose content is not (SEC-4, `blind`/`structural` / `blind-routing`) — so the operator
  forwards, holds, or serves ciphertext it has no key to read.
- **`edge-fn`, `box`, and an unencrypted `volume` are `declared`-trust** — and so is any formula built on them, e.g. a `database`. The operator (and any cloud host or
  subcontractor beneath it, DEPOT-6) can read what it must process to serve a query, run a function, or
  host a node. This is a **real, disclosed trust boundary** (SEC-4 `declared`), **not** structurally
  excluded. The durable protections are **DEPOT-3** (the owner-held root key — a breach reads live data
  but cannot *become* the user or survive a device revocation) and **DEPOT-4** (a real exit). A TEE with
  verifiable attestation upgrades these to `attested`.
- **SEC-1 fail-closed / SEC-6 authorise-never-classify / SEC-8 swappable** hold verbatim
  (DEPOT-7/-8/-4).
- **SEC-7 abuse is priced and localised**, never content-classified: a service refuses on identity /
  rate / payment; a poisoned operator is one operator, swappable, and rated down by independent
  measurements (DEPOT-9) rather than removed by a central authority.

---

## 7. Honest residual

- **Managed is not private.** A managed `edge-fn` or `box` — and any `database` formula, whose `box` runs the engine — is `declared` trust: the
  operator, its cloud, and its subcontractors can read what they process. Disclosed, not solved — the
  only durable protections are the **owner-held key** and the **real exit**, never host blindness. TEE
  attestation narrows this; it does not erase the operator's original access to plaintext-in-use.
- **The exit is a property, not a magic one.** Content-addressed services re-pin instantly; stateful
  ones need a genuine export, and a slow, throttled, or lossy export is a **weaker** exit than
  re-pinning. DEPOT requires an export (DEPOT-4); it cannot make a large stateful migration free.
- **Reputation is plural and gameable at the edges.** A market of raters can be astroturfed;
  reproducible measurements bound this (re-run the probe), signatures attribute it, and no single number
  is authoritative — but "distributed and honest" is a *reduction* of the trusted-rating-authority
  problem, not its elimination (DEPOT-9).
- **The re-run-the-probe bound has a hole, and it is the cheapest attack.** `method = "self-report"`
  and `evidence.kind = "transcript"` (§5) are **not reproducible by construction**, so they sit
  entirely outside the mitigation the bullet above leans on. `issuer == subject` catches only a rater
  signing as the operator itself; nothing stops an operator minting fresh pseudonymous keys — no
  anchor or personhood is required of a *rater* anywhere in DEPOT — and publishing praise from each.
  A consumer SHOULD therefore weight a measurement by whether its `method` is re-runnable at all, and
  MUST NOT treat a corpus of `self-report` claims as evidence. This is the general anti-Sybil ceiling
  ([DIRECTION §8](../DIRECTION.md)) arriving in a specific place, not a solved problem.
- **Falsification cost scales with the lie, which inverts the incentive.** Verifying a signature is
  cheap; verifying a *ceiling* is not. Testing a 2 TB `total_bytes` claim means storing ~2 TB, and
  testing an export at real scale means performing the migration — so the more aggressively an
  operator overstates, the more expensive it is to catch, and the routine cheap probes (`uptime`,
  `latency-ms`) are exactly the ones that never catch it. `capacity-conformance` and
  `export-conformance` make these claims falsifiable **in principle**; neither makes them cheap, and
  a profile MUST NOT be read as though publishing the metric had made the lie improbable.
- **The first large customer is unprotected — the measurement that would expose the lie is the harm.**
  For every property whose falsification requires an actual stress event — hitting a capacity ceiling,
  exporting at terabyte scale, discovering plaintext was readable — the observation can only be
  produced by someone already experiencing the harm. A patient operator can farm a clean, cheap,
  frequently-probed history (`uptime`, `latency-ms`) for as long as it likes and defect against the
  first commitment large enough to matter, at the moment its counterparty has least leverage. Plurality
  (DEPOT-13) is the real mitigation and it is a *cost* mitigation, not a detection one: use several
  independent providers so no single defection is total. This is distinct from the whitewashing
  residual in [REPUTATION](../primitives/REPUTATION.md), which concerns fresh keys — here the operator
  is long-lived and well-measured, and defects selectively on the axis nobody has priced.
- **A public IP and real compute are genuinely scarce.** DEPOT-10's self-host backstop is real only for
  a user who has the resource; the user who most needs a managed box is the one who cannot be their own.
  The scarcity is confined to this kind (like port-25 / REACH-9) but does not vanish.
- **Billing is only as honest as the operator, and no metric falsifies over-billing.** A
  `UsageReceipt` ([§18.8a.2](../18-wire-format.md)) is signed by the operator alone and is
  one-directional — it proves an operation occurred, and cannot disconfirm one the operator
  fabricated or silently omitted (disclosed there). §5's `capacity-conformance` catches an operator
  overstating what it *has*; **nothing catches an operator over-reporting what it *did*** — billed
  CPU-ms, invocations or `gpu-count` against delivered. TEE attestation (DEPOT-2) upgrades
  *execution-environment integrity*, a different property from *quantity billed = quantity used*.
  This is the deployed lesson of GPU-market fraud (io.net's 2024 spoofed-capacity incident: ~1.8M
  virtual GPUs farmed for rewards): metered decentralised compute has no cheap third-party proof
  that billed work was done. A client's real protections are the receipt trail as *evidence* for a
  dispute (`arbiter`), metering a workload it can independently bound, and plurality — never a
  protocol guarantee the number is true.
- **DEPOT is a supply-side design in a market whose hard problem is demand.** Every clause here — DEPOT-13
  permissionless supply, plurality-for-durability, "anyone MAY offer a box" — makes *listing* supply
  easy, and treats that as the achievement. The deployed comparables say the achievement is elsewhere:
  Akash and Golem have never lacked *listed* capacity, they lack *paid* utilisation — idle GPUs and thin
  operator economics, not a shortage of providers. DEPOT does not solve this and MUST NOT be read as
  though permissionless supply implied demand. It compounds with the **no-token** stance
  ([DIRECTION §5](../DIRECTION.md)): every deployed decentralised market used token emissions to pay early
  supply *before* paid demand existed, and DEPOT forbids that lever deliberately — so whether
  charge-for-service alone bootstraps and sustains a two-sided market is the **coordinator-funding open
  problem** ([DIRECTION §5, §8](../DIRECTION.md)), unproven, named here rather than left in the
  constitution where a reader of this profile would miss it. The honest position: DEPOT makes an honest
  market *possible* and says nothing about whether one *forms*.
- **Vulos is a participant, never an authority.** The maintainer MAY run the flagship gateway and a
  well-known status page and be one guardian and one rater — because **no token**, **swappable**, **no
  authoritative score**, and **reproducible measurement** structurally deny it a load-bearing position.
  "Run the project and be part of it all" is the model working as intended, not an exception to it.

Every residual traces to a root ceiling ([DIRECTION §8](../DIRECTION.md)): plaintext-in-use for a
query-serving or code-running service is the **compute-must-see-its-inputs** ceiling, disclosed rather
than dressed up as blindness; the scarce public IP is the **scarce-resource** exception; plural
reputation is the **no-global-authority** stance KOTVA takes everywhere. None is a bug in DEPOT; each is
a consequence of not being a single surveilling cloud, disclosed rather than solved.
