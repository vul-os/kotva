# DEPOT — managed infrastructure services (the decentralized-cloud profile)

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

A market where any operator — a **gateway** — offers managed infrastructure to a user who holds their
own keys and can leave: a **box** (a managed node), a **bucket** (object storage), an **edge-fn**
(serverless compute), a **database** (Redis / Postgres), a **cdn**. It is the Hetzner / AWS / ngrok
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

Bindings **adopted rather than reinvented** ([bindings/README.md](../bindings/README.md)): **WASI** /
**OCI** (edge-fn runtime), **S3 API** / content-addressing (bucket), **Redis RESP** / **Postgres
wire** (database), **HTTP caching** (cdn), **cloud-init** / any OS (box). DEPOT specifies **no** new
runtime, storage format, or price model.

---

## 3. The service registry (extensible — the future-proof surface)

The specific services are a **registry**, not spec text: a new service (a GPU cluster, a message queue,
a search index) is a **registry row**, never a spec change. Each row fixes the honest defaults the
profile enforces. v0 registry:

| `service` | Adopt (native protocol) | **Honest visibility** (DEPOT-2) | Metering unit (example — operator sets the number) | Portability (DEPOT-4) |
|---|---|---|---|---|
| `bucket` | S3 API / CID content-addressing | **`blind` / structural** — client-encrypted, content-addressed | GB-month + egress-GB | **zero-migration** (re-pin elsewhere) |
| `cdn` | HTTP caching | **`blind-routing`** — serves *public* objects; sees which/when, not private payload | egress-GB + requests | **zero-migration** |
| `edge-fn` | WASI / OCI | **`terminating`** (runs your code, sees I/O) → **`attested`** in a TEE | CPU-ms + invocations | **zero-migration** (redeploy the artefact) |
| `database` | Redis RESP / Postgres wire | **`terminating` / `declared`** — the operator sees plaintext to answer queries | GB-month + ops | **export/import** (a portable dump) |
| `box` | OS + node (cloud-init) | **`terminating` / `declared`** — the operator has root on the host | instance-hour | **export** (keys stay with the user; data dumps out) |
| `queue` | AMQP-class / opaque-payload FIFO | **`blind-routing` / structural** — holds **client-encrypted** payloads; sees depth, size, rate and timing, never content | message-count + GB-month retained | **zero-migration** (drain and re-enqueue elsewhere) |

**Completeness, not catalogue.** This set is deliberately small and is chosen to *span* what a
centralised platform does, not to mirror a product list: run code (`edge-fn`, `box`), store blobs
(`bucket`), store queryable state (`database`), serve at the edge (`cdn`), decouple asynchronously
(`queue`) — composed with public reachability ([REACH](reachability.md)), identity and login (§13),
messaging and wake (§2, §4.9), real-time (§27, §25), and the control plane of DEPOT-11. A capability
that is merely a *product* built from these MUST be a product, not a registry row.

**Worked example — static-site / SPA hosting is a product, not a service.** A static site is already
what PUB ([§22](../22-public-objects.md)) is: signed, content-addressed, self-verifying public objects
servable over plain HTTPS. It composes as **PUB objects in a `bucket`, served through `cdn`, named via
[REACH](reachability.md)** (own domain or vanity, certificates per REACH-2a) — and a deploy is simply
publishing a new content-addressed root plus a signed announcement superseding the previous one, which
makes the switch **atomic** and makes **rollback** a pointer back to a root that is still addressable.
It adds **no** registry row and **no** coordinator kind. What such a site DOES need, purely to stay
portable between providers (DEPOT-4), is one named schema — `kotva-depot/site/v0` — describing serving
behaviour: entry/root object, SPA **fallback** path, redirect rules, and cache policy. Without it each
operator invents its own hosting config and the site stops being swappable; with it, any `cdn` or box
serves the same site identically. It is a **schema over a content-addressed object** — no new wire
object, DS-tag, or error code — exactly like the measurement schema of §5. Deploy pipelines are
authorised by scoped `CapabilityToken`s under DEPOT-11 (a CI credential is strictly narrower than its
parent), so a leaked deploy token can publish a site and **cannot** reach mail or identity.

**Triggers, not more services.** Time-based and event-based invocation are **trigger types on
`edge-fn`**, never separate services: `http` (via REACH ingress), `cron` (a schedule), `queue` (a
`queue` message), and `webhook` (an inbound HTTPS event routed to a box or function through REACH,
buffered in a `queue` when the target is offline). A new trigger is an enum value; it is **not** a
spec change and **not** a new coordinator kind.

Only **`bucket`** (`blind`), **`queue`**, and public-object **`cdn`** (`blind-routing`) keep the
payload cryptographically out of the operator's reach. **`edge-fn`, `database`, and `box` are
`terminating`** — the operator sees your data or computation. This is normal and honest
(Fastmail-tier trust); it is **not** cryptographic blindness, and DEPOT-2 forbids pretending otherwise.

---

## 4. Normative profile rules

- **DEPOT-1 — one contract, adopted protocol.** An `infra-service` coordinator MUST publish a signed
  `CoordinatorDescriptor` (§18.8a) carrying `{service` (from §3), `visibility, metering-unit}` and MUST
  speak that service's **adopted native protocol** (§2) over an **`IK`-authenticated, Noise-secured**
  channel (REACH-2 shape — the user's `IK` is the libp2p identity key; **no bearer token**). It mints
  no new runtime, wire object, DS-tag, or error code — reputation reuses the ATTEST claim primitive (§5).
- **DEPOT-2 — honest visibility is load-bearing (the cliff).** Each service MUST declare **exactly**
  the visibility its data model permits (§3): `bucket` `blind`/`structural`; public `cdn`
  `blind-routing`; `edge-fn`/`database`/`box` `terminating`/`declared`. **Advertising a `terminating`
  service as `blind`, `private`, or `sovereign` is non-conformant misrepresentation**
  ([CONTRACT §3.2](../coordinator/CONTRACT.md)), not marketing. A TEE with **verifiable remote
  attestation** MAY raise `edge-fn`/`database`/`box` from `declared` to `attested`; the attestation
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
  service MUST state its **true** portability (§3): content-addressed `bucket`/`cdn` and stateless
  `edge-fn` are **zero-migration**; stateful `database`/`box` MUST provide a **portable
  export/import**, and MUST NOT be advertised as zero-migration. A slow or lossy export is a weaker
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
- **DEPOT-7 — authorize, never classify.** An `infra-service` gates admission on **identity + rate +
  payment** only ([CONTRACT §4](../coordinator/CONTRACT.md)); it MUST NOT admit, refuse, throttle, or
  price on a **content judgement**. Metering measures **resource use**, never content. (A service that
  *must* read content to function — `database`, `edge-fn` — does so under its declared `terminating`
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
  strings) MUST be **encrypted to the box's device key** before they reach the operator; the operator
  stores and serves **ciphertext only** and MUST NOT require plaintext to operate the service. An
  operator holding plaintext secrets is custodial in exactly the sense DEPOT-3 forbids for identity
  keys. Where a service genuinely needs the value in the clear at runtime (an env var inside a
  `terminating` `edge-fn` or `box`), that exposure is bounded by, and disclosed under, that service's
  already-declared visibility (DEPOT-2) — never presented as protected.
- **DEPOT-13 — permissionless supply; durability comes from plurality, not from an SLA.** Any node MAY
  offer any `infra-service`, including a single self-hosted box contributing spare capacity: the
  open-role principle of [Roles & Wake](../substrate/ROLES.md) and the self-host clause
  ([CONTRACT §2.3](../coordinator/CONTRACT.md)) apply to this kind unchanged. Joining is **publishing a
  signed descriptor** (§18.8a); standing is **earned through measurement claims** (§5), never granted
  by a gatekeeper. Because no single small provider can match a hyperscaler's availability, a client
  obtains durability and availability by **using several independent providers**, not by trusting one —
  and content-addressed services (`bucket`, `cdn`, and `queue` payloads) replicate freely, so plurality
  is cheap and re-pinning is zero-migration (DEPOT-4). **Honest asymmetry:** stateful `database` and
  `box` do **NOT** replicate freely — they carry single-writer state whose portability is an
  export/import, so for those a client's real protections are that export plus the operator's declared
  visibility, not replication. A profile MUST NOT present multi-provider replication as though it made
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
- **`schema`** — the DEPOT measurement schema `kotva-depot/measurement/v0` (an EAS schema UID or VC type
  URI), whose `claim` body is `{ service` (§3 registry value)`, metric` ("uptime" / "conformance" /
  "visibility-audit" / "latency-ms")`, value, method` ("probe" / "conformance-vector" / "audit" /
  "self-report")`, observed_at, ? evidence` (a reproducible recipe / vector-id / signed transcript)` }`.

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

- **Only `bucket`, `queue`, and public `cdn` are structurally private.** They hold client-encrypted,
  content-addressed data — or, for `queue`, client-encrypted payloads whose depth, rate and timing are
  visible but whose content is not (SEC-4, `blind`/`structural` / `blind-routing`) — so the operator
  forwards, holds, or serves ciphertext it has no key to read.
- **`database`, `edge-fn`, and `box` are `declared`-trust.** The operator (and any cloud host or
  subcontractor beneath it, DEPOT-6) can read what it must process to serve a query, run a function, or
  host a node. This is a **real, disclosed trust boundary** (SEC-4 `declared`), **not** structurally
  excluded. The durable protections are **DEPOT-3** (the owner-held root key — a breach reads live data
  but cannot *become* the user or survive a device revocation) and **DEPOT-4** (a real exit). A TEE with
  verifiable attestation upgrades these to `attested`.
- **SEC-1 fail-closed / SEC-6 authorize-never-classify / SEC-8 swappable** hold verbatim
  (DEPOT-7/-8/-4).
- **SEC-7 abuse is priced and localised**, never content-classified: a service refuses on identity /
  rate / payment; a poisoned operator is one operator, swappable, and rated down by independent
  measurements (DEPOT-9) rather than removed by a central authority.

---

## 7. Honest residual

- **Managed is not private.** A managed `database`, `edge-fn`, or `box` is `declared` trust: the
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
- **A public IP and real compute are genuinely scarce.** DEPOT-10's self-host backstop is real only for
  a user who has the resource; the user who most needs a managed box is the one who cannot be their own.
  The scarcity is confined to this kind (like port-25 / REACH-9) but does not vanish.
- **Vulos is a participant, never an authority.** The maintainer MAY run the flagship gateway and a
  well-known status page and be one guardian and one rater — because **no token**, **swappable**, **no
  authoritative score**, and **reproducible measurement** structurally deny it a load-bearing position.
  "Run the project and be part of it all" is the model working as intended, not an exception to it.

Every residual traces to a root ceiling ([DIRECTION §8](../DIRECTION.md)): plaintext-in-use for a
query-serving or code-running service is the **compute-must-see-its-inputs** ceiling, disclosed rather
than dressed up as blindness; the scarce public IP is the **scarce-resource** exception; plural
reputation is the **no-global-authority** stance KOTVA takes everywhere. None is a bug in DEPOT; each is
a consequence of not being a single surveilling cloud, disclosed rather than solved.
