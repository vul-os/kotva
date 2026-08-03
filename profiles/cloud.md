# DEPOT — the cloud elementals (managed-infrastructure profile)

> **Status:** profile spec (KOTVA family), **draft — a deferred extension, NOT part of Core v1**
> ([SPEC.md ratification tiers](../SPEC.md)). Normative once ratified. Provisional name (**DEPOT** — a
> supply depot where infrastructure is provisioned and dispensed); the codename is a founder call.

DEPOT specifies **the elementals of a cloud and the control plane that provisions them** — nothing
else. It defines **no runtime, no storage format, no query language, no economics, and no cloud**. It
names four irreducible resources, fixes what an operator must **declare** about each, and fixes the
**verbs** by which any client provisions and manages them. Everything a platform sells beyond that —
a database, a queue, a CDN, a registry, a static site, a hosted model — **composes** from the four
and is a *product*, never a spec change.

The point is that a **gateway brings its own implementation**. Back `bucket` with Tigris, MinIO, or
Ceph; back `box` with Hetzner, Vultr, Fly, or a rack in a basement; back `edge-fn` with any
WASI or OCI runtime — open source or not. DEPOT does not care, and deliberately cannot tell. What it
requires is that the operator **declare honestly** what it can see (§6 DEPOT-2), **speak the adopted
standard** (§6 DEPOT-1), **accept the same control verbs** (§5), and **let the user leave with their
bytes in a format someone else can read** (§6 DEPOT-4). A full open-source cloud built to satisfy
this profile is a **separate project**, not part of this spec, and DEPOT is written so that it never
needs to be the only one.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD
NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in
BCP 14 (RFC 2119, RFC 8174) when, and only when, in all capitals.

---

## 1. What this is

> **Terminology — "gateway" here is the colloquial word for a DEPOT operator, not the `gateway`
> coordinator kind.** This profile uses *gateway* in its everyday sense — a provider you can reach
> that offers managed infrastructure — because that is how operators describe themselves. It is
> **not** the `gateway` coordinator kind of [CONTRACT §5](../coordinator/CONTRACT.md) (the legacy-mail
> bridge: MX, DKIM egress, §7). A DEPOT operator is an **`infra-service`** coordinator. One party MAY
> run both roles, but they are distinct kinds with distinct requirements, exactly as
> [§26](../26-legacy-adapters.md) guards the same word for its own adapters.

A market where any operator offers managed infrastructure to a user who holds their own keys and can
leave. Each service is a **fenced coordinator**
([coordinator/CONTRACT.md](../coordinator/CONTRACT.md)): `accountable`, `swappable`,
`self-hostable`, and **never load-bearing** — reach and convenience, never a gate on the user's
function or identity.

### 1.1 Scope, stated as an exclusion

DEPOT is three things and no more:

1. **The elementals** (§3) — four resources, and what an operator MUST declare about each.
2. **The artefacts** (§4) — how images, snapshots and observability data are named and moved,
   because a control plane that cannot say *what boots* or *what happened* is not a control plane.
3. **The control plane** (§5) — the resource/ability vocabulary by which provisioning and
   management are authorised and requested.

**Where this sits in the family — DEPOT is the stateful intermediary, and that is the whole
difference.** It is *not* unusual in being `terminating`: most of the family is. Of the eleven
coordinator kinds ([CONTRACT §5](../coordinator/CONTRACT.md)), only `relay` and
`reachability-adapter` are unconditionally blind (and `media-relay` only where it published
`sframe_required`); `gateway` reads legacy mail in plaintext, `matcher` is terminating *always*, and
`arbiter`, `oracle`, `indexer`'s query channel and `custodial-escrow` all are too. Disclosed trust
is the family's majority position, not DEPOT's exception.

What genuinely separates DEPOT is that every other coordinator sees your data **in flight**, while
DEPOT holds it **at rest**. Leaving a `matcher` or an `arbiter` costs nothing — reconnect elsewhere;
the data was never theirs to keep. Leaving a `box` or a `volume` costs a **migration**. That one
distinction is load-bearing across the whole profile, and it is why:

- **DEPOT-4** carries an export obligation no other profile needs;
- **swappability** does the work here that cryptography does elsewhere — the exit *is* the guarantee;
- **DEPOT-13** leans on plurality for durability, which no other kind does — and so is the one place
  a thin market genuinely degrades the protections (§8);
- **DEPOT-8** collides hardest with legal compulsion, because a durable host is a servable target in
  a way a transient one is not.

A reader who imports the family's "intermediaries cannot betray you" thesis will misread this
profile — but so will one who assumes DEPOT is uniquely exposed. It is ordinary in *what the
operator sees* and singular in *what the operator keeps*.

*A future extraction seam, noted rather than acted on:* §3–§5 (the elementals, artefacts and control
plane) are adoptable by a cloud that has never heard of KOTVA — that half is closer to an interop
standard such as the S3 API or OCI than to a profile — while §6–§8 (visibility declaration,
reputation, residuals) are pure coordinator-contract application. If DEPOT is ever adopted outside
the family, that is where it splits. With no implementations yet, drawing the boundary now would
freeze it before anything has pushed on it.

It is explicitly **not**: a cloud, an orchestrator, a scheduler, an autoscaler, a price model, a
billing system, a marketplace, or a reference implementation. Where an existing standard covers a
job, DEPOT **adopts it and stops** ([bindings/README.md](../bindings/README.md)): **S3 API** and
content-addressing plus **HTTP caching** (`bucket`), **virtio-blk / NVMe-oF / iSCSI** (`volume`),
**cloud-init** and any OS (`box`), **WASI / OCI** (`edge-fn`), **OTLP** (observability, §4.3),
**Redis RESP / Postgres wire** (the `database` formula, §3.6).

### 1.2 What a gateway supplies, and the three backing modes (normative)

An `infra-service` coordinator is a **declaration and a control plane over an implementation it
chooses**. Who owns that implementation is a first-class, declared fact, because it changes both the
trust story and the exit — and a client MUST be able to tell the three apart before committing:

| `backing` | Who owns the underlying resource | Who is billed by it | Exit |
|---|---|---|---|
| `operator` | The gateway (its own hardware, or a cloud it rents and resells — §6 DEPOT-6) | The gateway bills the user (§6 DEPOT-5) | Export/import, or re-pin (§6 DEPOT-4) |
| `customer` | **The user**, at a provider the user holds an account with | The underlying provider bills the **user directly**; the gateway bills only for management | **Revoke the delegated credential.** The resource never left the user's account |
| `mixed` | Some parts each; the `DepotFormula` `Part` keys say which (§3.6) | Per part | Per part — and a formula is only as portable as its least-portable part |

`backing = customer` is the **bring-your-own** mode: the user keeps their own Hetzner, Vultr, Fly,
Tigris or S3 account, and the gateway operates it under a **delegated, attenuated, unilaterally
revocable** credential. It buys the **best exit in the profile** — structural rather than
contractual, because the bytes were never in the gateway's possession, so leaving is a credential
revocation and not a migration.

**It does not follow that it is the safest mode, and the trade runs the other way on containment.**
Under `operator` backing the gateway holds the resources it provisioned. Under `customer` backing it
holds **credentials into the user's own cloud account**, and its reach is then bounded by that
provider's IAM granularity rather than by DEPOT — which for providers offering only account-wide keys
means a compromised gateway can act on resources it never provisioned and bill the user for them.
So `customer` backing **improves the exit and can widen the blast radius**, and an operator MUST NOT
present it as though it improved both. The gateway's declared visibility is unchanged in *class* — it
sees exactly what the credential lets it see and MUST declare that (§6 DEPOT-2); it is never `blind`
by virtue of who owns the account.

A user who addresses their own S3 endpoint **directly**, with no coordinator in the path at all, is
outside this profile entirely — that is the self-host backstop
([CONTRACT §2.3](../coordinator/CONTRACT.md)), and it is always available.

---

## 2. What it composes

DEPOT is composition, not new machinery. It reuses:

| Composed with | Role in DEPOT | Home |
|---|---|---|
| **`infra-service`** coordinator kind | provides one managed service, fenced by the four clauses; declares `{service, backing, visibility, metering-unit}`. The one load-bearing new binding. | [CONTRACT §5](../coordinator/CONTRACT.md) |
| **`CoordinatorDescriptor` / `Tariff` / `UsageReceipt`** | the signed offer, the operator's own price, and the metered receipt — the accountable, self-asserted, discovery-only seam. | [§18.8a](../18-wire-format.md) |
| **`CapabilityToken`** | the control plane (§5) — scoped, attenuable, offline-verifiable, revocable. Never a bearer API key. | [§18.7.3](../18-wire-format.md) |
| **Economics seam** (CONTRACT §6) | settlement over any existing rail the operator chooses, **no *protocol* token, no published price-rank**; the *numbers* are operator policy. | [CONTRACT §6](../coordinator/CONTRACT.md) |
| **PUB** (feeds & blobs) | images, snapshots, formulas, site roots and measurements are all signed, content-addressed public objects — DEPOT mints no wire object of its own. | [§22](../22-public-objects.md) |
| **ATTEST** + the **indexer/labeler** role | distributed reputation and image provenance — signed claims anyone may publish and anyone may aggregate. | [primitives/ATTEST.md](../primitives/ATTEST.md) |
| **Identity** + **`RecoveryPolicy`** | non-custody (the root `IK` never leaves the user's device) + guardian recovery — **not** key escrow. | [§1](../01-identity.md), [§1.4](../01-identity.md) |
| **IK-authenticated Noise transport** | the client↔coordinator control/data channel is a libp2p-Noise `XX` stream keyed to the user's `IK` (as REACH-2), not a bearer token. | [profiles/reachability.md REACH-2](reachability.md) |
| **REACH** | ingress, naming, DNS, certificates and public reachability — DEPOT adds none of these. | [profiles/reachability.md](reachability.md) |
| **SYNC** / **RESERVE** | the coordination half of a composed formula (§3.6) — SYNC for a signed CRDT the operator cannot read, RESERVE for a single-writer claim. | [substrate/SYNC.md](../substrate/SYNC.md), [primitives/RESERVE.md](../primitives/RESERVE.md) |

---

## 3. The elementals

The registry below is **four rows and is meant to stay four**. A genuinely new *mechanism* is a
registry row and never a spec change; anything that composes from existing rows is a **formula**
(§3.6) and not a row at all. The split that generates exactly these four is **bytes vs code × cold
vs hot**:

|  | **stateless / cold** | **stateful / hot** |
|---|---|---|
| **bytes** | `bucket` | `volume` |
| **code** | `edge-fn` | `box` |

| `service` | Adopts | **Honest visibility** (DEPOT-2) | Metering unit (example — operator sets the number) | Portability (DEPOT-4) |
|---|---|---|---|---|
| `bucket` | S3 API / CID content-addressing; **HTTP caching** when serving public objects | **`blind`** — `structural` **only for objects the client actually encrypted** (S3 takes arbitrary bytes, so plaintext handed to it is readable and the declaration stays truthful — CONTRACT §3.3), else `declared`; **`blind-routing`** when it serves *public* objects, seeing which and when but never a private payload | GB-month + egress-GB + requests | **zero-migration** (re-pin elsewhere) |
| `volume` | Block device (virtio-blk / NVMe-oF / iSCSI); guest-owned filesystem | **`blind`** at `structural` **only where the guest actually encrypted** (LUKS/dm-crypt — the operator then holds ciphertext blocks plus access patterns, never plaintext); an unencrypted volume is **`terminating` / `declared`**, and the operator cannot tell which it was given (CONTRACT §3.3) | GB-month (+ provisioned IOPS) | **export/import** across operators; *detach/reattach* within one operator iff `attachment = detachable` |
| `edge-fn` | WASI / OCI | **`terminating`** (runs code, sees I/O) → **`attested`** in a TEE | CPU-ms + invocations | **zero-migration** (redeploy the artefact) |
| `box` | OS + node (cloud-init) | **`terminating` / `declared`** — the operator has root on the host | instance-hour | **export** (keys stay with the user; data dumps out) |

Only **`bucket`** and **`volume`** can keep a payload cryptographically out of the operator's reach —
**and only for what the client actually encrypted**. Both accept arbitrary bytes, so their blindness
is the *client's* property, not the operator's architecture: hand either one plaintext and it is
readable while the operator's declaration remains truthful (CONTRACT §3.3). A deployment wanting
unconditional blindness MUST make client-side encryption non-optional on ingest. **`edge-fn`, `box`,
and an unencrypted `volume` are `terminating`** — the operator sees the data or the computation. This
is normal and honest (Fastmail-tier trust); it is **not** cryptographic blindness, and DEPOT-2
forbids pretending otherwise.

### 3.1 Resources — quantities, an open vocabulary

What a client needs from a machine is *how much of what*, so `resources` is a map of
**`resource-name → uint quantity`** whose key namespace is **open and registry-extensible**. A new
accelerator class is a registry name and **never a spec change**:

| Resource key | Unit | Notes |
|---|---|---|
| `cpu-millicores` | thousandths of a core | 1000 = one core; integer, so no float creeps in (§18.1) |
| `mem-bytes` | bytes | |
| `gpu-count` | whole devices | a fractional GPU is a scheduling fiction; count devices |
| `gpu-mem-bytes` | bytes | the axis that actually decides whether a model fits |
| `ipv4-count` | whole addresses | a routable public IPv4 is genuinely scarce, transferable, and separately priced — a quantity a box holds, not a property of it (§3.1.1) |
| `accel-<class>` | vendor/registry-defined | the extension point — TPU, NPU, FPGA, whatever is next enters here, **never** as a new service |

#### 3.1.1 IPv4 is a resource; IPv6 is an attribute — and the asymmetry is economic, not stylistic

These are the two address families, and modelling them the same way would be wrong in both
directions. **IPv4 is scarce, transferable property with a real secondary-market price**: the RIR
free pools are exhausted, addresses trade at a per-address price, and an operator handing you one is
passing through a genuine recurring cost it cannot conjure. It is therefore a **quantity** —
countable, meterable, and legitimately billed per unit. **IPv6 is abundant and effectively free**: an
operator allocates a `/64` (or a `/56`, or a `/48`) and the count of addresses inside it is a number
with no economic meaning. Metering it would be theatre, and a `resources` key whose quantity nobody
can price is a key that invites invented scarcity.

So IPv6 is an **attribute** (§3.2): the client's question is *which prefix size do I get, if any* — a
predicate it filters on — not *how many addresses*, which is never the constraint.

| | family | vocabulary | why |
|---|---|---|---|
| `ipv4-count` | IPv4 | `resources` (quantity) | scarce, transferable, market-priced; a per-unit charge is an honest pass-through |
| `ipv6` | IPv6 | `attributes`, values `none` / `/64` / `/56` / `/48` | abundant; the prefix *size* is a capability predicate, the address *count* is meaningless |

**The honest consequence, stated because it cuts against this profile's own thesis.** DEPOT-13 says
permissionless supply and plurality are how a small operator competes. IPv4 is the sharpest place
that claim strains: a home operator typically holds **exactly one** routable IPv4, so the elemental
that most needs plurality is the one a small operator can least supply, and the cost gap against a
datacentre holding a `/22` is structural rather than a matter of efficiency. DEPOT does not fix this
and MUST NOT be read as though declaring `ipv4-count` had. What it does is make the constraint
**visible before commitment** (a declared ceiling, §3.3) instead of a surprise at provisioning time,
and make the IPv6-only path a **first-class, filterable option** rather than a degraded one — which
is the only lever a protocol actually has here. An operator MUST NOT charge for IPv6 addresses as
though they were scarce while declaring the same `class` of service; that is a `capacity-conformance`
falsehood dressed as a tariff (§6 DEPOT-5 fixes no prices, but §6 DEPOT-2 forbids misrepresenting
what is scarce).

**A GPU is not a service.** A box with an accelerator is still a `box`: same `terminating`
visibility, same operator-has-root posture, same export portability. Making `gpu` a registry row
would be exactly the catalogue-thinking §3 forbids. The same holds for a TPU (`accel-tpu`) — you
receive a machine with a device attached.

**A QPU is not a box, and this is the one accelerator that does not fit.** Deployed quantum services
are *submit a circuit, queue, collect results*: there is no OS, no root, no cloud-init, and nothing
to hold across invocations. That is the **`edge-fn`** shape with an operator-supplied runtime and a
circuit artefact (`format = "qir"` / `"qasm"`, §4.1) — not `accel-quantum` on a machine nobody gets.
Modelling it as a box would declare a `terminating` host that does not exist.

Because `resources` is one vocabulary, a declared ceiling and an enforced per-user quota are the
**same keys** — `cpu-millicores` and `gpu-count` are metered and limited independently without the
protocol naming a single number for either (CONTRACT §6).

### 3.2 Attributes — predicates, a registry

`resources` are **quantities**: more or less, and a client **compares** them. `attributes` are
**compatibility predicates**: you match one or you do not, and a client **filters** on them. Half an
ARM does not exist. Conflating the two is a common and expensive design error, so they are separate
fields with separate rules.

Attribute keys are a **registry with defined value spaces**, exactly as `resources` is. Without
that, two operators coin different keys for the same predicate and a client cannot filter portably —
which would leave the profile generic in theory and unusable in practice. Unrecognised keys are
ignored (§21.20 forward-compat); a *recognised* key with an unrecognised value MUST NOT be treated
as a match.

| Attribute | Applies to | Values | Notes |
|---|---|---|---|
| `arch` | `box`, `edge-fn` | `x86_64`, `aarch64`, `riscv64`, … | registry-extensible. Declaring an architecture the operator cannot serve is the same `capacity-conformance` falsehood as overstating bytes (§7) |
| `virt` | `box` | `vm`, `container`, `metal` | **an isolation boundary first and a compatibility predicate second** — see below |
| `ipv6` | `box` | `none`, `/64`, `/56`, `/48` | the prefix a box receives. A predicate, not a quantity — see §3.1.1 for why this is the opposite vocabulary from `ipv4-count` |
| `region` | all four | operator-chosen label, e.g. `eu-central`, `za-jhb` | **opaque to the protocol** — it is an equality predicate, not a geography KOTVA adjudicates. See below |
| `jurisdiction` | all four | ISO 3166-1 alpha-2, e.g. `DE`, `ZA` | the legal-venue predicate, distinct from `region`: two regions may share a jurisdiction. **Self-asserted and not network-falsifiable** — see below |
| `persistence` | `volume` | `ephemeral`, `durable` | `ephemeral` dies with the box (the instance-local disk a `box` already includes in its instance-hour); `durable` outlives box termination |
| `attachment` | `volume` | `bound`, `detachable`, `shared` | see §3.3 |
| `artifact-source` | `edge-fn` | `client`, `operator` | whether the client supplies the artefact or invokes one the operator supplies. **This is what a hosted-inference endpoint is** — an `edge-fn` whose code you did not write — and it is an attribute, not a kind |
| `sframe` / TEE class | `edge-fn`, `box` | per [§27](../27-realtime-media.md) / attestation binding | an `attested` claim MUST carry the attribute a client can check, or it reverts to `declared` (DEPOT-2) |

**`virt` is a tenancy boundary, and reading it as a mere compatibility flag is a security error
(normative).** It does double duty — nested virtualisation, kernel modules and device passthrough do
depend on it — but its **first** meaning is *what separates you from the operator's other customers*:
`container` is a shared kernel, `vm` is a shared hypervisor, `metal` is dedicated hardware. That is
the largest isolation difference an operator can offer and the axis on which a cross-tenant escape
either does or does not reach you. Three rules follow, and DEPOT deliberately adds **no isolation
taxonomy of its own** — the three existing values already carry the distinction:

- An operator MUST declare `virt` truthfully; declaring `vm` while scheduling shared-kernel
  containers is a DEPOT-2 misrepresentation, not a packaging detail.
- An operator MUST NOT present `container` isolation as equivalent to `vm`, nor `vm` as equivalent to
  `metal`, in any surface a user chooses from.
- **Side channels are not addressed and are not claimable.** Shared CPU, cache and memory bandwidth
  leak across tenants on all three of these below `metal`, and no declaration changes that. An
  operator MUST NOT advertise resistance to cross-tenant side channels on the strength of `virt`
  alone; a TEE claim is the `attested` path (DEPOT-2) and carries its own checkability requirement.

**Why `region` and `jurisdiction` are required and not "operator policy".** Placement is not a price
axis. It decides whether a CDN copy is actually closer, whether a commit-path `volume` is in the same
building as its `box`, and whether the deployment is lawful. A profile that folded CDN into `bucket`
(§3.7) on the grounds that "the CDN is simply a copy that is closer" **must** give a client a way to
say *closer to what* — otherwise the fold is sound and the result is unusable. DEPOT therefore fixes
the **key and its predicate semantics** and fixes **nothing about the value**: it never adjudicates
what `eu-central` means, never maintains a region list, and never ranks locations. `jurisdiction` is
separated from `region` precisely because the legal question is the one a user cannot afford to infer
from a marketing label.

**Honest limit — neither key is verifiable from the wire.** A latency probe weakly bounds *geography*
and nothing bounds *legal venue*: incorporation, ownership and the reach of a compelled-disclosure
order are facts about companies and courts, not about packets, and no measurement in §7 can
contradict a false `jurisdiction`. These two keys are therefore **declarations a user may rely on
contractually and MUST NOT treat as attested**, and DEPOT deliberately mints no `region-conformance`
metric because it would imply a check the network cannot perform. What the keys buy is that the claim
is **signed, specific and attributable** — an operator that misstates its jurisdiction has
misrepresented under DEPOT-2 and left evidence, which is a legal remedy rather than a protocol one.
For a user whose threat model is the operator's own jurisdiction, the protective mechanism is
client-side encryption (§3), never this field.

### 3.3 Declared capacity

An `infra-service` descriptor's `policy` field ([§18.8a.1](../18-wire-format.md)) is an **opaque,
kind-specific det_cbor blob**. DEPOT fixes its shape for this kind, so a **new limit is a DEPOT
registry change, never a §18 wire change**. It carries what a client must know *before* committing,
which the after-the-fact measurements of §7 cannot supply:

```cddl
DepotServicePolicy = {          ; det_cbor `policy` blob for kind = "infra-service"
  1 => tstr,                    ; service     a §3 registry value ("bucket", "volume", "box", "edge-fn")
  2 => tstr,                    ; backing     "operator" / "customer" / "mixed" (§1.2) — CLOSED
  ? 3 => Capacity,              ; capacity    declared ceilings — absent means UNDECLARED, never unlimited
  ? 4 => { * tstr => tstr },    ; attributes  §3.2 registry keys
  ? 5 => [* tstr],              ; abilities   §5.2 verbs this coordinator accepts; absent = the common set
}
Capacity = {                    ; every value a uint — no floats (§18.1)
  ? 1 => uint,                  ; total_bytes        storage ceiling
  ? 2 => uint,                  ; max_object_bytes   largest single object/volume
  ? 3 => uint,                  ; egress_bps         sustained throughput ceiling
  ? 4 => uint,                  ; max_concurrent     concurrent streams / instances
  ? 5 => tstr,                  ; class              "cold" / "warm" / "commit-path" (latency tier)
  ? 6 => uint,                  ; uptime_target      per-mille intent (0…1000) — an aim, NOT a promise
  ? 7 => { * tstr => uint },    ; resources          §3.1 OPEN quantity vocabulary — never a closed enum
}
```

**This is a declaration, not a promise, and the difference is the whole point.** A home operator with
2 TB and a 50 Mbit uplink can say exactly that and be chosen for the work it can actually carry,
instead of competing on an undifferentiated axis against a datacentre and losing. Three rules keep it
honest:

- **Absent means undeclared, never unlimited.** A client MUST NOT infer a ceiling from an omitted
  field, and an operator MUST NOT read an omission as permission to refuse arbitrarily.
- **`uptime_target` is an intent, not an SLA.** It is the operator's aim; what it *achieved* is the
  §7 `uptime` measurement, published by observers, not by the operator. A consumer MUST NOT treat the
  target as evidence.
- **A declared ceiling is falsifiable.** The descriptor is signed and the §7 `metric` vocabulary
  carries `capacity-conformance`, so an operator advertising 2 TB and refusing at 1 TB is
  **detectably** overstating — the same declare-then-measure loop that makes `visibility` honest.
  **Falsifiable is not cheaply falsified**, and §8 states the limit rather than papering it.

Quotas the operator *enforces* per user at runtime (rate limits, storage caps, `0x070D`, `0x0806`)
remain **operator policy**: the protocol standardises the *declaration* so a client can choose, never
the *number*.

**Why `volume` is a row and not a `box` attribute.** A `volume` is not a slow `bucket`; it is the
**hot tier**, and the gap is physical rather than an engineering debt that will close. A durable
commit needs an `fsync` in *tens of microseconds* on local NVMe, where networked storage is in
*milliseconds* and object storage further still — which is why the most advanced diskless designs
keep object storage **off** the commit path entirely and retain a small low-latency write-ahead tier
in front of it. A profile that offered only `bucket` could not host a correct database at all. The
`attachment` attribute (§3.2) covers the three deployment shapes, and a client MUST NOT assume any of
them: `bound` (this box only — typically instance-local NVMe, the fastest and the commit-path case),
`detachable` (datacentre-style network block: may be reattached to **another box at the same
operator**, so state outlives a dead box), or `shared` (multi-attach by several boxes at once, which
needs a cluster filesystem or external coordination, and whose consistency is the guest's problem,
never the operator's promise).

**Honest limit — detachable is not portable.** `detachable` buys resilience *within* one operator. It
buys **nothing** against operator lock-in, because a network volume is operator-local: moving it
elsewhere is an export/import, exactly as for `box`. The services that genuinely reduce state
lock-in are the content-addressed `bucket` (re-pin anywhere) and
[SYNC](../substrate/SYNC.md) (the user's own devices hold the state). An operator MUST NOT present
detachability as though it satisfied DEPOT-4 swappability.

### 3.4 Networking — REACH provides it, and the overlay *is* the private network

DEPOT adds **no** networking primitives, and the omission is deliberate rather than pending:

- **Ingress, naming, DNS, vanity domains and certificates** are [REACH](reachability.md). A DEPOT
  service that needs to be publicly reachable composes with REACH; it does not grow an ingress row.
  **A gateway MUST NOT require delegation of a user's DNS zone (normative).** The commodity pattern —
  "point your nameservers at us and we will handle everything" — makes the naming layer a captor: it
  is the single hardest thing to reverse, it hands the operator the ability to obtain certificates
  for every name you own, and it converts a swappable coordinator into a load-bearing one
  ([CONTRACT §2.2](../coordinator/CONTRACT.md)). REACH already fixes the alternative and DEPOT
  inherits it unchanged: a user's **own domain** stays in a zone the operator does not control, and
  the **box** obtains its own certificate over the passthrough path with **no zone write by the
  operator** (REACH-2a, ACME TLS-ALPN-01). An RFC 8657 `accounturi`-bound **CAA** record in the
  user's zone raises the bar further — but read REACH-1a before relying on it: a bare RFC 8659 CAA
  restricts only *which CA* may issue, not the validation *method* or *account*, and even the RFC
  8657 binding "restrains only a CA that implements RFC 8657". It is **necessary and not
  sufficient**, its precondition is verifiable by the **zone owner and never by the connecting
  client**, and a deployment that has not established its CA's `accounturi` enforcement MUST declare
  `declared`, not `structural`. The single case needing a DNS write — a wildcard certificate, which
  CA/Browser Forum policy bars from TLS-ALPN-01 —
  is served by delegating **one record** (`CNAME _acme-challenge` to a zone the box controls), never
  a zone.

  **Convenience is not the problem; compulsion and misdeclaration are.** Gateway-managed naming is
  good product design and this profile encourages it — an operator SHOULD make the zero-configuration
  path the default, because a user who must hand-edit DNS to get started mostly does not get started.
  What DEPOT fixes is that each tier **declares what it costs**, and that a user can always move down
  the table without changing identity (DEPOT-4):

  | Tier | User's DNS work | Who holds the TLS key | Visibility |
  |---|---|---|---|
  | **Operator vanity** — `you.gateway.example` | none | operator | `declared` — operator is sole writer of its own zone (REACH-3/-7) and can mint a certificate for that name at any time; the MITM residual is real and disclosed |
  | **Own domain, operator-driven ACME** | one `CNAME` | operator | `declared` — the convenient custom-domain path, and the operator terminates TLS |
  | **Own domain, box-held key** | one `CNAME` (+ one `CNAME _acme-challenge` for a wildcard) | **the box** | `blind-routing`, `structural` **iff** REACH-1a's CAA precondition is genuinely met |
  | **Operator-hosted zone** (NS delegation) | delegate the zone | operator | `declared`; MUST be optional, MUST NOT gate any other service |

  Two rules make the table load-bearing rather than decorative. An operator MUST NOT make any tier a
  **precondition** of another service — naming is convenience, never a gate (CONTRACT §2.2). And the
  middle two tiers differ **only in who holds the key, not in how much the user types**: a
  `CNAME _acme-challenge` delegation lets the operator *drive* issuance end-to-end while the **box**
  holds the private key, so the one-command experience costs nothing in assurance. An operator
  offering managed certificates SHOULD implement that variant and MUST NOT present a tier where it
  holds the key as though it were the tier where the box does.

  A third rule the table cannot show: **these tiers rank power over the *name and the certificate*,
  and the bottom two collapse where the naming operator also hosts the `box`.** "The box holds the
  key" is a real distinction when the party driving ACME is not the party with root on the machine —
  a self-hosted box, or a `box` rented from a *different* operator. Under `backing = operator` with
  one gateway doing both, the machine holding the key is a machine that gateway has root on (§3,
  `terminating`), so those rows differ in *who is expected to touch the key*, not in *who can*. The
  `structural` in that row is a claim about the **naming path** — that the operator is not the sole
  writer of a zone from which it can mint any name it likes at any time — and never a claim that TLS
  terminating on a rented box is out of the renting operator's reach. A user who wants that row to
  mean what it looks like has to put the box where the naming operator does not have root.
- **A routable public IPv4** is a *quantity* (`ipv4-count`, §3.1), not a service.
- **Private networking between boxes has no VPC row because KOTVA already provides a better one.**
  Boxes address each other over **IK-authenticated Noise** (REACH-2): mutually authenticated,
  end-to-end encrypted, and — unlike a cloud VPC — **not trusting the operator's switch fabric,
  hypervisor vNIC, or control plane**. A VPC is a perimeter drawn *by the party you are trying to
  bound*; the overlay is a cryptographic boundary that survives a hostile operator, and it works
  unchanged when the two boxes sit at *different* operators, which no VPC does. An implementation
  MUST NOT present operator-supplied network isolation as equivalent, and an operator offering a
  private-network product MUST declare it `terminating` for traffic it can observe.
  **The boundary is around the *path*, not around the *endpoints* — and on a managed `box` the
  operator owns the endpoints.** What the overlay removes from the trust surface is the operator's
  switch fabric, its hypervisor vNIC, its control plane, its other tenants, and any *transit* or
  *third* operator in between; that is a real improvement on a VPC and it is what "survives a
  hostile operator" is entitled to mean. It does **not** survive the operator hosting either end. A
  `box` is `terminating` (§3): the operator has root, and root reaches the Noise keys and the
  plaintext on both sides of the tunnel. Box-to-box encryption is therefore protection against
  everyone except the party you rented the boxes from, and it is only as strong as the weaker end's
  host — where both ends sit at one operator it buys tenant isolation and nothing against that
  operator, and where they sit at different operators each end is exposed to its own host alone.
- **Load balancing** is REACH ingress plus a health policy — a formula (§3.6), not a row.

### 3.5 The stateless/stateful asymmetry (normative for any scaling claim)

How many instances an operator runs, and how it spreads load across its own machines, is **operator
policy** ([CONTRACT §6](../coordinator/CONTRACT.md)); a DEPOT that specified scaling policy would be
prescribing an implementation. What *is* in scope is the decentralised form of the same need —
**choosing and failing over between operators**: signed `CoordinatorDescriptor`s to discover them,
published measurements (§7) to compare them, and DEPOT-4 swappability to leave. That composition, not
an elastic-group API, is this profile's load balancer.

**The honest ceiling is not the absent autoscaler, it is the asymmetry:** `bucket` and `edge-fn`
spread across operators freely *because* they are zero-migration, while `volume`, `box`, and anything
stateful built on them carry state that must be exported and re-imported to move. Adding providers is
cheap for the former and a migration for the latter, and no protocol rule changes that.

### 3.6 Formulas — everything else

Everything a centralised platform sells beyond the four is a **formula**: a recipe that composes
them, never a new mechanism. A formula is a **signed, content-addressed PUB object**
([§22](../22-public-objects.md)) anyone may publish, fork and compete on; the protocol never learns
what "Postgres" is.

```cddl
DepotFormula = {                          ; "kotva-depot/formula/v0"
  1 => tstr,                              ; kind        formula identity, e.g. "postgres" / "redis" / "kafka-queue"
  2 => [+ Part],                          ; parts       the primitive coordinators it composes (>= 1)
  ? 3 => bytes,                           ; recipe      opaque det_cbor provisioning/wiring, engine-defined
  ? 4 => tstr,                            ; consensus   what provides coordination if it scales; absent = single-writer
}
Part = {
  1 => tstr,                              ; service     one of the four §3 elementals
  2 => ik-pub,                            ; provider    the infra-service coordinator supplying this part
  ? 3 => hash,                            ; descriptor  content address of that provider's CoordinatorDescriptor (§18.8a.1)
}
```

**A formula's visibility and portability are inherited, not declared (normative).** A formula has no
honesty properties of its own; it takes the **least-blind** visibility and the **least-portable**
class of its parts. This is *computable*, not rhetorical: a client reads a `DepotFormula`, resolves
each `Part.provider`'s signed descriptor, and derives both. So DEPOT-2 and DEPOT-4 enumerate the four
elementals, and a formula is bound by the composition of what it names — there is no way for a
formula to be more blind or more portable than its most-exposed, most-stuck part.

A formula whose parts span **different operators** is legitimate and is exactly how a user avoids one
operator holding the whole database; a formula all of whose parts name **one** provider is a
single-operator managed offering, and the client can see which from the `provider` keys.

**A formula composes storage, never consensus.** `box` + `volume` + `bucket` gives the *ingredients*
of a scalable database, never the *coordination*. Two boxes cannot share one `volume`
(`attachment = shared` pushes consistency onto the guest, §3.3); two boxes *can* share a `bucket`,
but then which is primary, and what orders the writes? No engine gets that from object storage for
free — Neon built safekeepers, Aurora a bespoke storage layer. A formula advertising "scales across
boxes" MUST populate `consensus` with what provides the coordination (a consensus protocol, a
single-writer lease, an external quorum); an absent `consensus` means single-writer, and such a
formula MUST NOT advertise horizontal scaling.

### 3.7 Worked folds — what is deliberately not a row

Each of these is a product a real cloud sells as a service, and each is here to be *closed*: a future
reader proposing it as a fifth elemental should find it already answered.

- **`database` = `box` + `volume` + `bucket`.** A query engine on a box, its WAL and hot pages on a
  fast `volume` (§3.3), its archive/PITR on a `bucket`. DEPOT defines **no** query language, schema,
  replication protocol or consistency model — it adopts the existing wire protocols (Postgres, RESP)
  and stops. Visibility is not lost by dissolving the row: the `box` is `terminating`, so "the
  operator can read your data to answer a query" is declared by the box it runs on. **A Redis is the
  same formula minus the bucket.** *The native alternative comes first:* for state a user's own
  devices own, [SYNC](../substrate/SYNC.md) needs no rented database and no trusted operator. The
  `database` formula is the **compatibility bridge** for software that already speaks SQL or RESP,
  bought at the price of a `terminating` box.
- **`queue` = `bucket` + a claim mechanism.** The durable half of a queue is object storage —
  WarpStream and AutoMQ run Kafka-compatible queues with zero local disk, straight to and from S3.
  What remains is ordering and exactly-once claim, which is [SYNC](../substrate/SYNC.md) or
  [RESERVE](../primitives/RESERVE.md) — **not** a rented broker. *That half is the unproven one:*
  exactly-once claim among mutually-distrusting consumers is a real distributed-systems problem, and
  this fold names where it is solved rather than demonstrating that it is. The fold is therefore *capable of
  being* more private than a broker: the payload sits in a bucket that is `blind` exactly to the
  extent the client encrypted it, and the claim is a CRDT among the consumers themselves. A broker
  sees plaintext by construction; this fold *lets* a client keep the operator out, it does not do it
  for them.
- **`cdn` folded into `bucket`.** A CDN and an object store are the *same mechanism*: retain
  content-addressed bytes and serve them on request. In a content-addressed system there is no
  origin-versus-edge distinction to encode — every holder is equivalent and every byte is verified by
  hash, so "the CDN" is simply **a copy that is closer**. What differs is the **content class**, and
  visibility follows from it: client-encrypted objects leave the operator `blind`; public objects it
  must serve on demand leave it `blind-routing`. *Closer to what* is `region` (§3.2); how many copies
  and where is tariff.
- **`registry` (container/image) = `bucket` + `DepotImage` + a PUB tag.** An OCI registry is a
  content-addressed blob store with a manifest convention on top, which is a **schema**, not a
  service (§4.1).
- **`static site` / SPA hosting = PUB objects in a public-serving `bucket`, named via REACH.** A
  deploy is publishing a new content-addressed root plus a signed announcement superseding the
  previous one — which makes the switch **atomic** and **rollback** a pointer back to a root that is
  still addressable. It needs one schema, purely to stay portable between providers:

```cddl
DepotSite = {                                    ; "kotva-depot/site/v0"
  1 => hash,                                     ; root      content address of the site root manifest (§22)
  ? 2 => tstr,                                   ; fallback  SPA fallback path, e.g. "/index.html"
  ? 3 => [* {1 => tstr, 2 => tstr, 3 => uint}],  ; redirects { from, to, status }
  ? 4 => {? 1 => uint, ? 2 => bool},             ; cache     { max_age_s, immutable }
}
```

  A provider MUST serve `root` as the site, apply `redirects` in array order, and — for a path
  resolving to no object — serve `fallback` when present or return 404 when absent (never a guess).

- **`hosted inference` / rented-GPU AI = `edge-fn` with `artifact-source = operator`, on a box class
  declaring `gpu-count`.** Renting someone's model endpoint is invoking code you did not write on
  hardware you do not hold: the same `terminating` visibility, the same TEE path to `attested`, the
  same metering seam. It is an **attribute**, not a coordinator kind — which is why the provisional
  `compute` kind folds into `infra-service` ([CONTRACT §5](../coordinator/CONTRACT.md)).
- **`shared filesystem` (NFS/EFS-class) = `box` + `volume`, exporting a filesystem.** Distinct from
  `attachment = shared`, which is shared *block* and explicitly leaves consistency to the guest; a
  filesystem service adds server-side consistency, and that server is a `box` someone runs.
- **`load balancer` = REACH ingress + a health policy** (§3.4).
- **`cron` / `webhook` / `queue-trigger` are trigger types on `edge-fn`**, never services: `http`
  (via REACH ingress), `cron` (a schedule), `queue` (an item arriving on a `queue` formula), and
  `webhook` (an inbound HTTPS event routed through REACH, buffered in a `queue` formula when the
  target is offline). A new trigger is an enum value.

**Encoding (normative — grounding, not new bytes).** DEPOT invents no PUB object. Its schemas are
integer-keyed deterministic-CBOR maps carried under a **named `meta` key** on a `PubAnnounce`
([§22.3](../22-public-objects.md)), embedded as `bytes` exactly as §24 carries `meta["artifact"]`:
`DepotFormula` under `meta["depot-formula"]`, `DepotSite` under `meta["depot-site"]`, `DepotImage`
under `meta["depot-image"]`. `DepotMeasurement` (§7) is **not** a PUB payload but an ATTEST claim
body. A reader that does not implement DEPOT ignores the unknown `meta` key like any other (§21.20
forward-compat), so publishing a formula never breaks a generic §22 node.

---

## 4. Artefacts — what boots, what is kept, what happened

A control plane that cannot name *what boots*, *what was kept*, or *what happened* is not a control
plane. These three are the minimum artefact surface, and all three reduce to existing machinery.

### 4.1 Images and snapshots — one schema, three targets

An image is **immutable content-addressed bytes in a `bucket`** plus a manifest — the same shape as a
site root, and not a fifth elemental. A **mutable tag** (`myapp:latest` → digest), the one thing a
bucket does not supply, is a **signed PUB announcement superseding the previous one**: atomic,
attributable, and better than a registry tag because the superseded digest stays addressable, so
rollback is a pointer rather than a rebuild.

```cddl
DepotImage = {                    ; "kotva-depot/image/v0"
  1 => tstr,                      ; target    what it instantiates: "box" / "edge-fn" / "volume" — CLOSED
  2 => tstr,                      ; format    CLOSED registry: "raw"/"qcow2"/"oci"/"wasm"/"qir"/"qasm"/"fs-dump"
  3 => hash,                      ; digest    content address of the artefact (in a bucket)
  4 => uint,                      ; bytes     artefact size
  ? 5 => tstr,                    ; arch      compatibility predicate, as attributes.arch (§3.2)
  ? 6 => { * tstr => tstr },      ; boot      engine hints — cloud-init dataset ref, OCI entrypoint, …
  ? 7 => hash,                    ; parent    content address of the image this derives from
}
```

One schema deliberately covers what every cloud keeps in three separate systems — **VM images**
(`target = "box"`, `format = "qcow2"`/`"raw"`), **function artefacts** (`target = "edge-fn"`,
`format = "oci"`/`"wasm"`), and **snapshots** (`target = "volume"`, `format = "fs-dump"`/`"raw"`) —
because all three are an immutable artefact plus a manifest plus a signed tag. An unrecognised
`format` MUST be refused, never guessed at.

**A snapshot is not an export, and conflating them loses one of the two.** A DEPOT-4 **export**
exists so a user can *leave*: it MUST be in the adopted standard's own interchange format and
ingestible by a different operator without the exporting operator's cooperation. A **snapshot**
exists so a user can *operate*: rollback, clone, backup — frequent, incremental, and legitimately
operator-optimised. A snapshot MAY be an export when its `format` is an interchange format; an
operator MUST NOT offer only a proprietary snapshot and call the DEPOT-4 obligation met.

**Provenance is an ATTEST claim about a `digest`**, not new machinery: who built this image, from
what source, with what toolchain. DEPOT defines no build system and does not require one.

### 4.2 Where an image comes from

`provision` (§5.2) on a `box` or `edge-fn` MUST name a `DepotImage`, and because that image is
content-addressed in a `bucket`, **the identical artefact is retrievable and verifiable at any
operator** — a Hetzner-backed and a Vultr-backed gateway fetch the same bytes and can prove it by
hash. That is what makes DEPOT-4 portability concrete for `box` rather than aspirational: the exit is
not only a data dump, it is a named artefact the next operator can ingest.

**Content-addressing makes the bytes identical; it does not make them boot.** A `box` image carries
assumptions about firmware (UEFI versus BIOS), virtio or NVMe driver availability, the cloud-init
datasource it expects, bootloader layout and CPU features — and an operator whose platform differs on
any of these will fetch the correct artefact and fail to start it. `arch` (§3.2) filters the coarsest
case and no attribute covers the rest. So the portable unit is the **artefact and its manifest**, and
bootability across operators is a property of the image's own construction, which DEPOT does not
specify and MUST NOT be read as guaranteeing. `edge-fn` is genuinely stronger here: a WASI or OCI
artefact targets a defined runtime rather than a machine.

### 4.3 Observability — a required seam, a composed store

**You cannot operate infrastructure you cannot see, and this is the one thing a cloud's users need
every day that no signed descriptor can supply.** §7 measurements are third-party claims *about an
operator*; they are not this. This is your own box's logs.

- **Normative seam (DEPOT-14).** A `box` and an `edge-fn` MUST expose logs and metrics to the holder
  of the `observe` ability (§5.2) as an append-only stream in **OTLP** (OpenTelemetry Protocol —
  adopted, not invented; it carries logs, metrics and traces in one vendor-neutral wire format). An
  operator MUST NOT make observability available *only* through a proprietary console, because a
  console is not portable and is not scriptable, and a client that cannot read its own telemetry
  cannot compare operators or leave one.
- **Retention and query are a formula, not a service.** Ship OTLP to a `bucket`; query with an
  `edge-fn`. A log store is `bucket` + `edge-fn` and needs no row, exactly as a database does not.
- **The cost of this requirement, priced rather than assumed.** DEPOT-13 celebrates the operator
  running spare capacity from a basement, and DEPOT-14 hands that operator a dependency it did not
  choose. The burden is real and is stated here rather than discovered at implementation time — but
  the floor is lower than OTLP's reputation suggests: the **HTTP/protobuf** binding is a plain
  `POST` of a length-delimited message to one endpoint, with no gRPC stack, no streaming, and no
  collector required, and an operator MAY implement exactly that and nothing more. The alternative
  considered and rejected was making the format advisory; it recreates the proprietary-console
  problem DEPOT-14 exists to prevent, since telemetry nobody else can parse is telemetry that
  cannot be used to compare operators or to leave one. This is the same adopt-a-standard trade as
  S3 for `bucket` or virtio for `volume`, and it is not free in any of those cases either.
- **The stream is authorised and protected exactly like the control plane.** Telemetry MUST be
  served over the same **`IK`-authenticated, Noise-secured** channel as every other DEPOT request
  (DEPOT-1) and authorised by a `CapabilityToken` carrying `observe` for that resource (§5) — never
  by a log-shipping bearer token, a signed URL, or an unauthenticated collector endpoint. OTLP is
  adopted for its *wire format*, not for its default transport posture: an operator MUST NOT expose a
  plaintext or anonymous OTLP receiver for a user's telemetry.
- **Honest visibility.** Telemetry from a `terminating` `box` is already visible to the operator, so
  this seam adds no exposure there. It **does** add exposure when telemetry is shipped *elsewhere*: a
  third-party observability provider is its own `infra-service` with its own declaration, and logs
  are frequently the most sensitive plaintext a system emits. An operator MUST NOT present forwarded
  telemetry as covered by the source service's visibility declaration.

---

## 5. The control plane

DEPOT-11 requires the control plane to be a capability rather than an API key. That is necessary and
not sufficient: [`Capability`](../18-wire-format.md) is `{resource: tstr, ability: tstr}` with
**free-text** members, so two conformant gateways could name the same operation differently and fail
to interoperate **silently** — unlike an unrecognised *caveat*, which fails closed by construction
(§18.7.3). This section closes that hole. It adds **no wire object**: it fixes the vocabulary that
goes into the existing `CapabilityToken`, and requests ride each service's adopted native protocol.

### 5.1 Resource grammar

```
depot:<service>/<instance-id>     one instance, e.g.  depot:box/7f3a91c2
depot:<service>/*                 every instance of one service at this coordinator
depot:*                           every DEPOT resource at this coordinator
```

`<service>` is a §3 elemental. `<instance-id>` is operator-assigned and opaque. Attenuation follows
§18.7.3 unchanged: `depot:box/7f3a91c2` is narrower than `depot:box/*`, which is narrower than
`depot:*`, and a child token can only narrow.

**A resource string names no operator, so the token MUST (normative).** Instance ids are
operator-assigned and opaque, so `depot:box/7f3a91c2` at one coordinator and at another are the same
string denoting different machines — and `CapabilityToken.aud` binds the **delegatee**, not the
target. Without a target binding, a credential delegated to something that talks to two gateways (a
CI runner, a teammate's laptop) is presentable at both: a **confused deputy**. Every DEPOT capability
MUST therefore carry the caveat `depot:coordinator` whose value is the intended coordinator's `IK`,
and a coordinator MUST reject a token whose `depot:coordinator` is absent or is not its own key. This adds
**no new wire object**, and most of the enforcement is free: §18.7.3 already evaluates every caveat
on every link and **fails closed on any it does not recognise**, so a non-DEPOT coordinator rejects
the token automatically. It is **not** entirely free, and the difference matters — fail-closed covers
a caveat that is *present and unrecognised*, not one that is *absent*. Requiring presence is a real
additional check a DEPOT coordinator MUST perform: a token carrying no `depot:coordinator` caveat is
well-formed under §18.7.3 and valid everywhere, which is exactly the defect being closed.

### 5.2 Ability registry (CLOSED — extended by registry addition, never by coinage)

**Common lifecycle**, applicable to all four elementals:

| Ability | Meaning |
|---|---|
| `provision` | create an instance from a declared shape (and, for `box`/`edge-fn`, a `DepotImage`, §4.2) |
| `inspect` | read one instance's configuration and state |
| `list` | enumerate instances in scope |
| `reconfigure` | mutate declared configuration of an existing instance |
| `observe` | read logs and metrics (§4.3) |
| `export` | obtain the DEPOT-4 portable export |
| `destroy` | delete an instance and release its resources |

**Per-elemental:**

| `service` | Additional abilities |
|---|---|
| `box` | `start`, `stop`, `restart`, `snapshot`, `console` |
| `volume` | `attach`, `detach`, `resize`, `snapshot` |
| `bucket` | `read`, `write`, `delete`, `serve` (toggle public serving, §3.7) |
| `edge-fn` | `deploy`, `invoke`, `rollback` |

Three rules make this vocabulary load-bearing rather than decorative:

- **Unknown ability fails closed.** A coordinator receiving an ability outside this registry MUST
  refuse (`FAIL_CLOSED_BLOCK`, [§21](../21-errors-iana.md)) and MUST NOT map it onto a
  similar-sounding one. An operator MUST NOT coin `terminate` for `destroy`; a new verb is a registry
  addition. This is what makes one open-source client able to drive any conformant gateway.
- **`console` is the privilege cliff, and MUST be separately delegable.** Interactive access to a
  `box` subsumes nearly every other ability — it can read secrets, alter state, and forge the
  evidence of having done so. A token granting `console` MUST NOT be issued as an implicit
  consequence of granting `provision` or `reconfigure`.
- **An operator MUST NOT offer `destroy` while withholding `export`.** This binds the operator's
  *product*, not any individual grant: a service where the holder of the account can delete an
  instance but can never extract it has made the exit weaker than the loss (DEPOT-4). It is
  emphatically **not** a rule that every token carrying `destroy` must also carry `export` — that
  would make a cleanup credential (a CI job reaping preview environments) into a **data-exfiltration
  credential**, which is the opposite of least privilege. Delegated grants narrow freely and
  independently; the obligation is on what the coordinator offers, checked against its declared
  `abilities` (§3.3).

### 5.3 Semantics

- **Idempotency.** `provision` with identical parameters and nonce yields the same instance, not a
  second one; `destroy` on an absent instance succeeds. Retries are safe.
- **`destroy` MUST make the data irrecoverable to the next tenant (normative).** Releasing a
  `volume` or a `box`'s local disk back into an operator's pool without rendering the previous
  tenant's bytes unreadable is a **cross-tenant disclosure**, and it is the failure that has recurred
  in deployed clouds. Before reallocating storage an operator MUST either erase it or ensure the
  bytes were never readable in the first place — a `volume` the guest encrypted (§3) satisfies this
  by construction once the key is discarded, which is the cheapest correct implementation and the one
  an operator SHOULD prefer. `destroy` MUST NOT return success while the bytes remain recoverable by
  a subsequent tenant. Backups and snapshots taken under DEPOT-4 or §4.1 are **not** covered by a
  `destroy` on the instance: an operator MUST state their retention and deletion separately, because
  a user who deletes a box and keeps paying for its snapshots has deleted nothing.
- **Authorisation is the token, never the transport.** Possession of a network path grants nothing:
  every request carries a `CapabilityToken` chain verified offline to a root the coordinator trusts
  (§18.7.3), over the IK-authenticated Noise channel of DEPOT-1.
- **Scope separation is normative.** A capability that acts on **infrastructure** MUST be scoped
  separately from one that acts on **mail or identity** (DEPOT-11). Provisioning a box MUST NOT
  implicitly grant reading a mailbox, and a leaked deploy token MUST NOT reach either.

---

## 6. Normative profile rules

- **DEPOT-1 — one contract, adopted protocol, and the implementation is the operator's business.** An
  `infra-service` coordinator MUST publish a signed `CoordinatorDescriptor` (§18.8a) carrying
  `{service, backing, visibility, metering-unit}` and MUST speak that service's **adopted native
  protocol** (§1.1) over an **`IK`-authenticated, Noise-secured** channel (REACH-2 shape — **no
  bearer token**), accepting the §5 control vocabulary. It mints no new runtime, wire object, DS-tag,
  or error code.
  **The Noise requirement binds the coordinator, not its backing.** The coordinator terminates the
  authenticated channel; what it speaks *behind* that boundary — the HTTPS API of a commodity cloud,
  a hypervisor socket, a local runtime — is its own business and is **expected** to be some existing
  provider's protocol. Nothing else would let a gateway bring its own implementation. What the
  operator MUST NOT do is let that backing leak into the contract: not the auth model (no bearer key
  reaches the KOTVA client), not the vocabulary (§5.2), and not the visibility (DEPOT-6).
- **DEPOT-2 — honest visibility is load-bearing (the cliff).** Each service MUST declare a visibility
  **no more blind than** its data model permits (§3) — declaring *less* blindness than permitted is
  always conformant, since an operator may hold itself to a stricter account than the protocol
  requires; declaring *more* is the violation: `bucket` `blind` — `structural` **only for what the
  client encrypted**, `declared` otherwise (CONTRACT §3.3) — or `blind-routing` when public-serving;
  `volume` `blind` when the guest encrypts it, else `terminating`/`declared`; `edge-fn`/`box`
  `terminating`/`declared`; a **formula** inherits the least-blind of its parts (§3.6). **Advertising
  a `terminating` service as `blind`, `private`, or `sovereign` is non-conformant misrepresentation**
  ([CONTRACT §3.2](../coordinator/CONTRACT.md)), not marketing. A TEE with **verifiable remote
  attestation** MAY raise `edge-fn`/`box` from `declared` to `attested`; the attestation MUST be
  checkable by the client, or the claim reverts to `declared`. **This clause is currently inert and
  says so deliberately:** no TEE attestation [binding](../bindings/README.md) exists yet, so there is
  no interoperable way for a client to perform that check, and until one is ratified **every**
  `edge-fn`/`box` is `declared` in practice. The rule degrades safely — the default is the honest
  answer — but an operator MUST NOT advertise `attested` on the strength of a TEE it merely operates.
- **DEPOT-3 — non-custody, no key escrow.** The user's **root `IK` is generated and held on the
  user's own device**; a managed `box` receives only a **revocable `DeviceCert` subkey**
  ([§1.2](../01-identity.md)). An `infra-service` **MUST NOT** hold, escrow, or be able to use the root `IK`, and
  **operator-held key backup is FORBIDDEN** — it makes a swappable coordinator load-bearing (the
  party you may need to leave holds the means to leave). "I forgot my key" is answered by **guardian
  / social recovery** ([§1.4](../01-identity.md)): an operator MAY be **one guardian of a quorum**,
  never sufficient alone.
  **What the subkey still is, since "only a subkey" reads as a reduced identity.** A `DeviceCert` is
  a working credential: the box may do whatever its `caps` allow ([§1.2](../01-identity.md)), and on
  a `terminating` box the operator can do it too, because it has root on the machine holding the
  key — and the hardware-keystore hardening of [§1.2a](../01-identity.md) is unavailable here, since
  the keystore and its attestation would both be the operator's own. Non-custody bounds the
  **aftermath and the duration** of that — the identity itself never moved, the operator cannot
  rotate or re-issue it, cannot change the `RecoveryPolicy` alone (§1.4 makes even an `admin` device
  one factor), cannot follow the user to the next operator, and loses everything on revocation. It
  does not bound the **live** capability, so the limit on the damage is detection time. See §8.
- **DEPOT-4 — swappable, honest portability.** Leaving or switching an `infra-service` MUST be a
  **config change with zero identity change** ([CONTRACT §2.2](../coordinator/CONTRACT.md)). Each
  service MUST state its **true** portability (§3): content-addressed `bucket` and stateless
  `edge-fn` are **zero-migration**; stateful `volume`/`box` MUST provide a **portable
  export/import**, and MUST NOT be advertised as zero-migration. A `detachable` volume moves between
  boxes of **one** operator and MUST NOT be advertised as zero-migration on that basis (§3.3).
  **"Portable" means format-portable, and downtime is an acceptable price:** the export MUST be in
  the **adopted standard's own interchange format** — S3-API objects for `bucket`, a standard block
  image or filesystem dump for `volume`, the engine's native dump (`pg_dump`-class, RESP) for a
  `database` formula, an OCI/WASI artefact for `edge-fn`, a standard disk image for `box` — such that
  **any conformant operator of the same service can ingest it without the exporting operator's
  cooperation**. An export only its author's tooling can read is **not** an export. The exit this
  profile guarantees is *interoperable*, not *seamless*: a migration MAY cost real downtime, and that
  is an acceptable price for being able to leave at all; what is never acceptable is a format that
  makes leaving impossible. A slow or lossy export is a weaker exit and MUST be disclosed as such
  (§8).
- **DEPOT-5 — economics are the operator's; KOTVA specifies only the seam.** Prices, price model,
  billing cycle, free tier, SLA, discounts, and settlement asset are **operator policy**, carried in
  the signed `Tariff`/policy as bytes KOTVA does not inspect. KOTVA requires **only**: the tariff is
  **signed and discoverable**; usage is metered into **signed `UsageReceipt`s delivered to the
  payer**; settlement is over an **existing rail the operator chooses** ([DIRECTION
  §5](../DIRECTION.md)); there is **no *protocol* token** and **no published global price-rank**
  ([CONTRACT §6](../coordinator/CONTRACT.md)). Two operators MAY run entirely different economics and
  both be conformant.
- **DEPOT-6 — bring your own implementation; subcontracting stays accountable and never launders
  visibility.** An operator MAY fulfil a service through **any** implementation it chooses — its own
  hardware, an open-source stack, or a commodity cloud it rents and resells — and this is the
  expected case, not an exception (§1.2). It **remains the sole accountable, declaring party**: a
  subcontracted `terminating` leg is still `terminating` and MUST be declared so, and an operator
  MUST NOT claim `blind` by pointing at a subcontractor. The user holds the **declaring operator**
  accountable, not its supplier. An operator MUST declare `backing` (§3.3) truthfully; misdeclaring
  it is a DEPOT-2 misrepresentation, because it misstates who can be compelled and who holds the
  bytes.
  **The invariant that separates subcontracting from lock-in:** a third party the *operator* uses
  MUST NOT become a party the *user* must trust or cannot replace. Applied to the recurring cases —
  DNS hosting (§3.4: never a zone delegation), the ACME CA (an operator MUST NOT be its own CA for a
  user's own-domain names), a TEE attestation root (client-checkable or the claim reverts to
  `declared`, DEPOT-2), a settlement rail (DEPOT-5), an observability sink (its own `infra-service`
  with its own declaration, DEPOT-14), and the commodity cloud underneath (declare it, do not launder
  it). An operator may depend on whatever it likes; the user must be able to leave all of it by
  leaving the operator.
  **What the invariant does not buy, because three natural readings of it are wrong.** It is a rule
  about **accountability and replaceability**, not about exposure. (i) A subcontractor carrying a
  `terminating` leg **is** in the user's confidentiality surface — it reads the same plaintext the
  declaring operator does and answers to its own jurisdiction, whatever the user's contract with the
  declaring operator says. "MUST NOT become a party the user must trust" means the user need hold no
  *relationship* with it and depend on no *promise* of its; it never means the subcontractor cannot
  read. (ii) Leaving is **forward-looking**. It ends future exposure and does nothing about bytes,
  backups or logs the subcontractor already holds. (iii) **`backing` does not name the
  subcontractor.** It is three values (§1.2), and `operator` covers a rack in a basement and a
  resold hyperscaler equally. So "declare it, do not launder it" forbids *claiming a blindness you
  do not have*; it does not tell a client which cloud is underneath, and DEPOT mints no field that
  does. A user who needs to know the supplier must ask off-wire and MUST NOT expect a descriptor to
  answer.
- **DEPOT-7 — customer-supplied backing is delegated, attenuated, and revocable.** Where
  `backing = customer` (§1.2) — the user's own Hetzner, Vultr, Fly, Tigris or S3 account, operated by
  the gateway — the credential the gateway holds MUST be (a) **created by the user in their own
  account and handed over deliberately** — the gateway MUST NOT mint it for them, and MUST NOT hold
  any credential capable of minting further credentials; (b) **attenuated at the underlying
  provider** to the narrowest scope that provider supports; (c) **unilaterally revocable by the user
  without the gateway's cooperation**; and (d) **encrypted in transit and at rest** in the gateway's
  own store. The gateway MUST declare the visibility that credential actually grants it — a
  credential with read access makes the gateway `terminating` regardless of who owns the account.
  **What (d) does and does not buy, since DEPOT-12 does not apply here.** DEPOT-12 seals secrets to
  *the box's* device key, so that the operator's control plane, database and backups never hold them
  in the clear (with the further bound DEPOT-12 states on its own reach). That mechanism is unavailable
  for a backing credential, because the party that must **use** it is the gateway's own control
  plane: it necessarily holds the plaintext at the moment of use. So (d) protects against a stolen
  database, a leaked backup, and a compromised third party — **never against the gateway itself**.
  A user under `backing = customer` is trusting the gateway with account credentials, and no
  encryption clause changes that; (b) and (c) bound the damage, not (d).
  **The bearer-key carve-out, stated rather than hidden.** DEPOT-11 forbids bearer API keys **in the
  KOTVA control plane**. A commodity cloud that has never heard of KOTVA will issue a bearer
  credential and nothing in this profile can change that. The honest position is that the bearer
  token stops at the boundary: it authorises the *gateway* at the *third party*, it is never what
  authorises the *user* at the *gateway*, and its blast radius is bounded by (b) and its lifetime by
  (c). This is a real residual (§8), not a solved problem.
- **DEPOT-8 — authorise, never classify, scoped as [CONTRACT §4](../coordinator/CONTRACT.md) scopes
  it.** An `infra-service` gates admission on **identity + rate + payment**, and **MUST NOT run
  content classification** — spam scoring, ML filters, keyword or URL reputation — as a gate on a
  **delivery path or a canonical/authoritative path**. Metering measures **resource use**, never
  content. A service that must read content to function (an `edge-fn`, the `box` inside a `database`
  formula) does so under its declared `terminating` visibility, never as a content gate on delivery.
  **The scoping is the rule, not a softening of it.** §4's prohibition is aimed at a specific
  failure — a systematic classifier that improves with corpus size, therefore centralises, therefore
  never finishes, forming a second centralised tier. It is scoped to content that *reaches, or is
  withheld from, a recipient by default*. Three consequences follow, and an earlier draft of this
  clause got them wrong by stating the prohibition flatly, which made DEPOT stricter than the family
  rule it cites:
  - **Storing or running a user's own resources is not a delivery path.** A private `bucket`,
    `volume`, `box`, or an `edge-fn` the user deployed has no recipient being protected from the
    operator's judgement. §4 does not reach it, and DEPOT does not extend it there.
  - **Executing a specific, lawful, third-party order is not classification.** A takedown notice or
    court order names particular content and supplies someone else's judgement; the operator runs no
    classifier and builds no corpus. Likewise **sanctions screening is an identity gate**, which this
    clause has always permitted. Neither is an exception to §4 — both fall outside what it prohibits.
  - **Refusing a customer is an identity gate, and this profile does not forbid it.** The protection
    against deplatforming in this family is **DEPOT-4 swappability plus the DEPOT-15 self-host
    backstop**, not §4 — and at one operator that protection is weak (§8). Stated plainly rather than
    left to be inferred from silence.

  **The one surface where the conflict is real: a public-serving `bucket`.** Serving public objects
  *is* a delivery path, and such a bucket serves plaintext to arbitrary readers by definition, so it
  can never be `blind` (§3). That is precisely where proactive scanning for categories an operator is
  legally obliged to act on — CSAM above all — is demanded in practice, and it **is** systematic
  classification of the kind §4 prohibits. DEPOT does not pretend this resolves. An operator that
  performs such scanning on a **public-serving surface** is conformant **only** if it (a) confines it
  to that surface and never to private storage or compute, (b) confines it to categories it is
  legally obliged to act on rather than discretionary content policy, and (c) **discloses that it
  does so**, alongside the `jurisdiction` attribute (§3.2) under whose law it acts — so a user
  filtering on jurisdiction is choosing this knowingly. Extending it to general moderation is
  non-conformant.
  **The user's lever is client-side encryption, with an honest bound.** An operator holding
  ciphertext cannot be compelled to judge plaintext it does not have, which makes §4 hold
  *structurally* for a `blind`/`structural` service rather than as a promise. It does **not** make the
  operator untouchable: it can still be compelled to preserve data, hand over ciphertext, and
  identify the account holder, and some jurisdictions are actively legislating against the position.
  Encryption narrows what can be demanded; it does not exempt anyone.
- **DEPOT-9 — fail-closed.** An unpaid, expired, unauthenticated, over-quota,
  unrecognised-ability, or unbound request — the last being a capability lacking a valid
  `depot:coordinator` caveat (§5.1) — MUST fail closed — a clean refusal or connection close
  ([§21](../21-errors-iana.md) `FAIL_CLOSED_BLOCK`), never a silent best-effort, a partial charge, or
  a content-based drop.
- **DEPOT-10 — distributed reputation, no authority.** Service quality is a **market of signed
  measurements**, never a single authoritative score — reputation is measured locally by each client
  ([CONTRACT §3.1](../coordinator/CONTRACT.md)). A measurement is an **ATTEST claim** (§7). A
  **status page is a REPRODUCIBLE aggregation** of such feeds; a client chooses which raters to
  weight. Automated measurements **SHOULD be reproducible** — **reproducibility over reputation**. A
  measurement is **attributed to its signing rater**; a **self-measurement** (rater `IK` == the rated
  coordinator) MUST be presentable as such. **Any party MAY run a rater**, and **none is
  authoritative**. A rater is the [`labeler`/`indexer`](../coordinator/CONTRACT.md) role; running one
  alongside a gateway is one operator serving two separable, attributable roles.
- **DEPOT-11 — the control plane is a capability with a fixed vocabulary.** Provisioning,
  configuring and destroying an `infra-service` MUST be authorised by a **`CapabilityToken`**
  ([§18.7.3](../18-wire-format.md)) scoped by the §5 `resource`/`ability` grammar: attenuable,
  delegable, offline-verifiable, revocable. It MUST NOT be a bearer API key or an unscoped account
  password, and DEPOT mints no control-plane token of its own. A delegated token (a deploy key, a CI
  credential, a teammate's grant) is **strictly narrower** than its parent, and this rests on **two**
  independent mechanisms of §18.7.3, not one: the **attenuation invariant** bounds `resource` and
  `ability` to same-or-narrower at every link, and **caveat evaluation** makes a parent's caveats
  non-droppable because every link's caveats are checked and an unrecognised one fails closed. A
  capability that can act on the user's **mail or identity** MUST be scoped separately from one that
  acts on infrastructure.
- **DEPOT-12 — secrets are sealed to the box, never held in operator plaintext.** Configuration
  secrets an `infra-service` stores on a user's behalf (environment values, credentials, connection
  strings, and the DEPOT-7 backing credential) MUST be **encrypted to the box's device key by the
  client before they leave it**; an operator MUST NOT **require** plaintext to operate the service,
  and one that accepts or stores plaintext secrets MUST declare that surface `terminating` rather
  than implying the secrets are protected. Where a service genuinely needs the value in the clear at
  runtime (an env var inside a `terminating` `edge-fn` or `box`), that exposure is bounded by, and
  disclosed under, that service's already-declared visibility.
  **Honest residual — this is a `declared` property, not a structural one.** Unlike DEPOT-3, where
  the root `IK` is withheld *by construction*, DEPOT defines **no secret-envelope object and no
  verification step**: services speak adopted native protocols (DEPOT-1), so nothing on the wire
  distinguishes a sealed blob from a pasted plaintext. The enforceable half is the **client's**
  obligation to seal first; the operator's half is **detectable, not preventable** — via an ATTEST
  `visibility-audit` measurement (§7) and the exit (DEPOT-4).
  **Second honest residual — the sealing target is hardware the operator runs.** A secret sealed to
  a `box`'s device key is out of reach of the operator's **control plane, its store and its
  backups**, which is a real reduction and the one this clause buys. It is not out of reach of the
  **operator**: the box is `terminating`, the operator has root on it, and root reaches the key that
  opens the envelope. `DeviceCert.key_protection` ([§1.2a](../01-identity.md)) does not rescue this
  on a managed box, because the keystore that would make the key non-exportable is the operator's
  own hardware and its attestation is the operator's own to produce. The mechanism that would close
  the gap is a client-checkable TEE, which DEPOT-2 records as **inert** until a binding exists. So
  DEPOT-12 raises the cost of a stolen dump, a leaked backup and a curious employee; it does not put
  a secret beyond the party running the machine, and an operator MUST NOT present it as though it
  did.
- **DEPOT-13 — permissionless supply; durability comes from plurality, not from an SLA.** Any node
  MAY offer any `infra-service`, including a single self-hosted box contributing spare capacity: the
  open-role principle of [Roles & Wake](../substrate/ROLES.md) and the self-host clause
  ([CONTRACT §2.3](../coordinator/CONTRACT.md)) apply unchanged. Joining is **publishing a signed
  descriptor**; standing is **earned through measurement claims** (§7), never granted by a
  gatekeeper. Because no single small provider can match a hyperscaler's availability, a client
  obtains durability by **using several independent providers** — and content-addressed `bucket`
  bytes replicate freely, so plurality is cheap and re-pinning is zero-migration.
  **"Independent" is doing real work in that sentence, and it means independent *ownership*, not
  independent *code*.** Providers running the **same implementation** are independent for ownership,
  jurisdiction, business failure and seizure — and are **not independent for faults**: one defect,
  one bad release, one CVE fails all of them at the same moment, and no number of providers changes
  that. Since a family with one widely-adopted implementation is the normal early state, a client
  MUST NOT read "several providers" as fault diversity unless the providers actually differ in
  software, and an operator MUST NOT advertise it as such. Plurality bounds *operator* risk; only
  implementation diversity bounds *code* risk, and DEPOT can require neither.
  **Nor is independent ownership the same as independent infrastructure, and this one is invisible
  to the protocol.** Under `backing = operator` a gateway may be reselling a commodity cloud (§1.2,
  DEPOT-6), so two separately-owned, separately-keyed gateways reselling the **same** hyperscaler in
  the same region share precisely the outage, the jurisdiction and the seizure the client chose two
  providers to diversify — while looking, on the wire, exactly like two independent ones. No field in
  a descriptor names the supplier beneath (DEPOT-6), so unlike the implementation-diversity gap
  above — which a client can at least ask about and an operator can answer — this correlation cannot
  be detected from anything DEPOT carries. What plurality verifiably delivers is independence of
  **keys**; independence of ownership, jurisdiction and infrastructure are inferences a client draws
  from off-wire knowledge, and a client that has not done that work holds fewer of them than the
  word "independent" implies.
  **Honest asymmetry:** `volume`, `box`, and any formula built on them do **NOT** replicate freely
  (§3.5). A `detachable` volume is not a counter-example: it moves between boxes of one operator,
  never between operators. A profile MUST NOT present multi-provider replication as though it made a
  stateful service as durable as a content-addressed one.
- **DEPOT-14 — observability is a right of the resource holder, not a product tier.** A `box` and an
  `edge-fn` MUST expose logs and metrics in **OTLP** to the holder of the `observe` ability (§4.3),
  and MUST NOT make them available only through a proprietary console. Telemetry forwarded to a third
  party is that party's own `infra-service` with its own declaration.
- **DEPOT-15 — self-host backstop + disclosed scarcity.** Anyone with the resource MAY run any
  `infra-service` for themselves ([CONTRACT §2.3](../coordinator/CONTRACT.md)). The honest
  exceptions, disclosed not papered over, are the fenced ones: a **reputable public IP / ingress**
  and **real compute, storage, and bandwidth** are resources a host or ISP allocates, not conjured —
  confined to this kind, never a protocol chokepoint (the port-25 / REACH-9 analog, generalised).

---

## 7. Measurements are ATTEST claims — no new wire object

A service measurement is an **ATTEST** public `Attestation`
([primitives/ATTEST.md](../primitives/ATTEST.md)), **not** a bespoke DEPOT object. DEPOT mints **no
new wire object, DS-tag, or signature** for reputation — only a **claim schema**: carrier is the
ATTEST public carrier on a PUB feed; `issuer` is the rater's `IK`; `subject` the rated coordinator's;
a self-measurement is exactly `issuer == subject`.

```cddl
DepotMeasurement = {                ; claim body for schema "kotva-depot/measurement/v0"
  1 => tstr,                        ; service      a §3 elemental
  2 => tstr,                        ; metric       "uptime" / "conformance" / "visibility-audit" / "latency-ms"
                                    ;              / "capacity-conformance" / "export-conformance" / "ability-conformance"
  3 => uint / bool,                 ; value        metric-typed, below — never a float (§18.1)
  4 => tstr,                        ; method       CLOSED: "probe"/"conformance-vector"/"audit"/"self-report"
  5 => ts,                          ; observed_at  ms since the Unix epoch (§18.1)
  ? 6 => { 1 => tstr, 2 => tstr },  ; evidence     { kind: CLOSED "recipe"/"vector-id"/"transcript", ref }
}
```

`value` is typed **by `metric`**: `uptime` = `uint` per-mille (`0…1000`); `latency-ms` = `uint` ms;
`conformance`, `visibility-audit`, `capacity-conformance`, `export-conformance` and
`ability-conformance` = `bool`. `capacity-conformance` records whether the operator honoured its own
declared ceilings (§3.3); `export-conformance` whether a DEPOT-4 export actually round-tripped into a
*different* operator; `ability-conformance` whether the coordinator accepts the §5.2 vocabulary
without coinage or aliasing — the cheapest of the three to test, and the one that makes the control
plane's interoperability falsifiable rather than assumed. **With one implementation in existence it
is vacuous, and an aggregator MUST NOT treat it as evidence of interoperability in that state:** its
whole purpose is catching a gateway that diverged from the vocabulary, and where every gateway runs
the same code there is nothing to diverge from, so it passes for reasons unrelated to conformance.
Below two independent implementations the binding check is the **schema vector corpus**
(`conformance/SUITE.md`), which is derived from this document by hand rather than from any
implementation and therefore still disagrees with a wrong one.

`metric`, `method` and evidence `kind` are **closed value sets**: a rater MUST NOT coin a value and
an aggregator MUST ignore an unknown one, never guess. Measurements are an **append-only time-series**
(§22.4.2): a newer observation does **NOT** supersede an older one — the history is what reputation
aggregates over, so raw observations MUST NOT be collapsed to a latest-only value. A rater MAY
**revoke** a measurement (ATTEST `Revoke`); an issuer signing two contradictory claims at one feed
position is **detectably equivocating**, surfaced for dispute, never merged away. A consumer SHOULD
**re-run** any `probe`/`conformance-vector` whose `evidence` supplies a reproducible recipe rather
than trusting the reported `value`. A malformed or unverifiable measurement is **ignored**, never a
fail-closed event and no new error code. New metrics or methods are **new schema versions**.

---

## 8. Security and honest residual

Inheriting [THREAT-MODEL.md](../THREAT-MODEL.md) (SEC-1…SEC-9). SEC-1 fail-closed, SEC-6
authorise-never-classify and SEC-8 swappable hold verbatim (DEPOT-9/-8/-4); SEC-7 abuse is priced and
localised, never content-classified.

- **Managed is not private.** A managed `edge-fn` or `box` — and any formula containing one — is
  `declared` trust: the operator, its cloud, and its subcontractors can read what they process. This
  is the **compute-must-see-its-inputs** ceiling ([DIRECTION §8](../DIRECTION.md)), disclosed rather
  than dressed up as blindness. The durable protections are **DEPOT-3** (the owner-held root key) and
  **DEPOT-4** (a real exit). TEE attestation narrows this; it does not erase the operator's original
  access to plaintext-in-use. **DEPOT-3's protection is narrower than "the operator cannot become
  you":** a managed box holds a live `DeviceCert` subkey with its own `caps`, so an operator with
  root on that box can **act as that authorised device** — send and receive under the identity, and
  read what the device's cluster membership decrypts — for as long as the cert stands. What the
  owner-held root structurally denies is different and still worth having: no rotation, no re-issue,
  no unilateral `RecoveryPolicy` change (§1.4), nothing that follows the user to the next operator,
  and nothing at all after revocation. The bound on the damage is therefore **detection time**, not
  cryptography, and non-custody MUST NOT be read as impersonation-resistance while the device is
  live.
- **`bucket` and `volume` blindness is the client's discipline, not the operator's architecture.**
  This is the sharpest self-deception risk in the profile: the label reads like an operator guarantee
  and is a statement about the client's own habits, and the failure is **silent** — a misconfigured
  SDK or a plain `cp` loses the whole protection without the operator lying or anything erroring.
- **A terminating service is a compellable service, and that is a consequence of §3 rather than a
  gap in it.** Everything an operator can read, it can be ordered to produce, preserve, or remove —
  so `box`, `edge-fn`, and any unencrypted `bucket` or `volume` are exposed to legal process by the
  same property that makes them useful. DEPOT-8 now scopes the classification prohibition where
  CONTRACT §4 scopes it and admits the one surface where the conflict is irreducible — a
  **public-serving `bucket`**, which serves plaintext to arbitrary readers and is therefore both a
  delivery path and unencryptable. There, an operator legally obliged to scan will scan, and the
  profile's answer is disclosure plus a declared `jurisdiction` (§3.2), not prevention. Client-side
  encryption removes the operator's *ability* to judge, which is the only structural answer available
  — and it does not stop preservation orders, ciphertext production, or account-holder
  identification. A user whose threat model is legal process against their host should read §3's
  visibility column as a **compulsion surface**, not merely a privacy one.
- **The exit is a property, not a magic one.** Content-addressed services re-pin instantly; stateful
  ones need a genuine export, and a slow, throttled, or lossy export is a **weaker** exit than
  re-pinning. DEPOT requires an export; it cannot make a large stateful migration free.
- **Customer-supplied backing moves the trust, it does not remove it (DEPOT-7).** The gateway still
  holds a credential to a third party that has never heard of KOTVA, and that credential is a bearer
  token by the third party's design. Attenuation depends entirely on how granular that provider's
  own IAM is — some are excellent, some offer one account-wide key — and DEPOT can require narrowing
  but cannot create a scope the provider does not implement. What the mode genuinely buys is a
  **unilateral, instantaneous exit** and the removal of the gateway from the *ownership* of the
  resource. It does not make the gateway blind, and the honest failure mode is a user who believes it
  does.
- **Reputation is plural and gameable at the edges.** A market of raters can be astroturfed;
  reproducible measurements bound this, signatures attribute it, and no single number is
  authoritative — but "distributed and honest" is a *reduction* of the trusted-rating-authority
  problem, not its elimination.
- **The re-run-the-probe bound has a hole, and it is the cheapest attack.** `method = "self-report"`
  and `evidence.kind = "transcript"` are **not reproducible by construction**. `issuer == subject`
  catches only a rater signing as the operator itself; nothing stops an operator minting fresh
  pseudonymous keys — no anchor or personhood is required of a *rater* anywhere in DEPOT — and
  publishing praise from each. A consumer SHOULD weight a measurement by whether its `method` is
  re-runnable at all, and MUST NOT treat a corpus of `self-report` claims as evidence.
- **Falsification cost scales with the lie, which inverts the incentive.** Verifying a signature is
  cheap; verifying a *ceiling* is not. Testing a 2 TB `total_bytes` claim means storing ~2 TB, and
  testing an export at real scale means performing the migration — so the more aggressively an
  operator overstates, the more expensive it is to catch, and the routine cheap probes (`uptime`,
  `latency-ms`, `ability-conformance`) are exactly the ones that never catch it.
- **The first large customer is unprotected.** For every property whose falsification requires an
  actual stress event — hitting a capacity ceiling, exporting at terabyte scale, discovering
  plaintext was readable — the observation can only be produced by someone already experiencing the
  harm. A patient operator can farm a clean, cheaply-probed history and defect against the first
  commitment large enough to matter, at the moment its counterparty has least leverage. Plurality
  (DEPOT-13) is the real mitigation and it is a *cost* mitigation, not a detection one.
- **Billing is only as honest as the operator, and no metric falsifies over-billing.** A
  `UsageReceipt` ([§18.8a.2](../18-wire-format.md)) is signed by the operator alone and is
  one-directional — it proves an operation occurred, and cannot disconfirm one the operator
  fabricated or silently omitted. `capacity-conformance` catches an operator overstating what it
  *has*; **nothing catches an operator over-reporting what it *did*** — billed CPU-ms, invocations or
  `gpu-count` against delivered. TEE attestation upgrades *execution-environment integrity*, a
  different property from *quantity billed = quantity used*. This is the deployed lesson of
  GPU-market fraud (io.net's 2024 spoofed-capacity incident: ~1.8M virtual GPUs farmed for rewards).
  A client's real protections are the receipt trail as *evidence* for a dispute (`arbiter`), metering
  a workload it can independently bound, and plurality — never a protocol guarantee the number is
  true.
- **Offline verification buys availability and costs revocation latency.** A `CapabilityToken`
  verifies without contacting a server (§18.7.3) — which is why the control plane keeps working when
  a coordinator is unreachable — but a revocation is a **separately published object** a verifier has
  to have seen. Between revoking a token and the coordinator observing that revocation, the token
  still verifies. The bound is the token's own `exp` (mandatory, no eternal capability), so the
  practical mitigation is **short lifetimes on high-privilege grants** — `console` above all (§5.2) —
  rather than a protocol guarantee of immediate effect. An operator SHOULD poll revocations at a
  cadence it discloses; a client SHOULD assume a revoked token remains usable for that interval.
- **A public IP and real compute are genuinely scarce.** DEPOT-15's self-host backstop is real only
  for a user who has the resource; the user who most needs a managed box is the one who cannot be
  their own. The scarcity is confined to this kind (like port-25 / REACH-9) but does not vanish.
- **DEPOT is a supply-side design in a market whose hard problem is demand.** Every clause here makes
  *listing* supply easy. Akash and Golem have never lacked *listed* capacity; they lack *paid*
  utilisation. DEPOT does not solve this and MUST NOT be read as though permissionless supply implied
  demand — it compounds with the no-token stance, since every deployed decentralised market used
  token emissions to pay early supply *before* paid demand existed. Whether charge-for-service alone
  bootstraps a two-sided market is the **coordinator-funding open problem**
  ([DIRECTION §5, §8](../DIRECTION.md)), unproven. DEPOT makes an honest market *possible* and says
  nothing about whether one *forms*.
- **DEPOT's market protections are conditional on a market existing, and the likely early state is
  one gateway.** DEPOT-10 (plural reputation) and DEPOT-13 (plurality for durability) are the
  profile's answers to a bad operator, and **both are inert at one operator**: reputation informs a
  choice that does not exist, and "use several independent providers" is unavailable. The bullets
  above that lean on plurality — the first-large-customer defection, billing you cannot falsify, a
  capacity ceiling too expensive to test — therefore lose their stated mitigation entirely in that
  state, and the honest statement is that they are *unmitigated*, not merely weakened. What still
  holds at one operator is exactly the set that never depended on the market: the **owner-held root
  key** (DEPOT-3), **client-side encryption** of `bucket`/`volume` (§3), **content-addressing** (a
  bucket's bytes can be re-pinned by anyone, including the user, without the operator's
  cooperation), and the **self-host backstop** (DEPOT-15) — whose own limit is already disclosed
  above: the user who most needs a managed box is the one who cannot be their own. The protocol
  guarantees **permissionless entry**, never **actual plurality**; a reader MUST NOT treat DEPOT-13
  as though it delivered the latter.
- **One implementation is a different failure from one operator, and the profile's protections sort
  the opposite way.** At **one operator**, the market clauses (DEPOT-10, DEPOT-13) go inert while the
  cryptographic ones hold. At **one implementation with many operators**, the market clauses work —
  plurality is about ownership, and an exit to another operator is real — but **spec correctness
  fails**: "conformant" degenerates into "behaves like the reference implementation", a defect in
  that implementation becomes a defect in the protocol, and no measurement in §7 can reveal it
  because every measured party shares the bug. `ability-conformance` is explicitly vacuous there
  (§7). The mitigation is not a rule this profile can impose — it is the **hand-derived schema
  vector corpus**, which is written from this document rather than emitted by any implementation and
  so remains a second opinion when no second implementation exists. That corpus is load-bearing in a
  way its size suggests it is not, and the honest ranking of what would retire this residual is: an
  independent **decoder** in another language reading the same vectors (cheap, and the rung actually
  worth buying), then a genuinely independent implementation (expensive, and its absence should be
  stated rather than assumed away).
- **Vulos is a participant, never an authority — but that claim is strong for stateless kinds and
  weak for this one.** The maintainer MAY run the flagship gateway and a well-known status page and
  be one guardian and one rater. For `relay`, `labeler` or `indexer` the structural denials are
  real, because Vulos holds nothing: **no token**, **no authoritative score**, and **reproducible
  measurement** leave it nothing to withhold. For **`infra-service` they are much weaker**, because
  a stateful host *does* hold something (§1.1). No token does not stop it holding a volume; no
  authoritative score does not either; reproducible measurement reports a problem and supplies no
  remedy. Of the four denials only **swappable** does real work here — and swappability requires a
  destination, which is the previous bullet. The maintainer running the only gateway is therefore
  the family's least-defended configuration, and it is named here rather than left to be inferred.

Every residual traces to a root ceiling ([DIRECTION §8](../DIRECTION.md)): plaintext-in-use for a
code-running service is the **compute-must-see-its-inputs** ceiling; the scarce public IP is the
**scarce-resource** exception; plural reputation is the **no-global-authority** stance KOTVA takes
everywhere. None is a bug in DEPOT; each is a consequence of not being a single surveilling cloud,
disclosed rather than solved.
