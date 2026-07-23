# REACH — public reachability for arbitrary box services

> **Status:** profile spec (KOTVA family). Normative once ratified. REACH is the **infrastructure
> profile** that gives a sovereign box a stable public name for *any* service it runs — a web app,
> an API, a Git remote, an object bucket, a game server — without a static IP and without the
> coordinator that provides the name ever reading the traffic. It defines **no new wire kinds**: it
> is the generalization of DMTAP [§7.15.2](../07-gateway.md) (the legacy-client *reachability
> ingress*) from "carry one legacy mail client to a mailbox" to "carry any TLS client to any box
> service", and of [§7.10.5](../07-gateway.md) (vanity *local-parts*) from mail local-parts to **DNS
> subdomains**. The party that provides the public name is a **`reachability-adapter`**
> ([coordinator/CONTRACT §5](../coordinator/CONTRACT.md)), fenced by the coordinator contract like
> every other.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are to be interpreted as in BCP 14 (RFC 2119, RFC 8174).

---

## 1. What this is

A sovereign box behind CGNAT or on a dynamic IP has services worth reaching, but ordinary clients
(a browser, `git`, an S3 SDK, a game client) cannot speak the mesh and cannot hole-punch. REACH
gives each service a stable public HTTPS name — `svc.alice.reach.example` — that any such client
dials normally, while the adapter providing that name stays **content-blind**: it routes by TLS SNI
onto a persistent reverse tunnel and **the box terminates TLS**. The adapter forwards ciphertext it
holds no key to read. This is the ngrok/Cloudflare-Tunnel product shape, rebuilt so the tunnel
operator is *hired, not depended on* — accountable, swappable, self-hostable, and blind by
construction ([DIRECTION §0/§4](../DIRECTION.md)).

REACH is three thin things over existing substrate:

1. a **reverse-tunnel stream-mux** — a box→adapter persistent, libp2p-secured (Noise + yamux)
   connection the adapter SNI-demuxes and multiplexes arbitrary inbound client streams onto: the
   ngrok/frp-shaped reachability ingress of [§7.15.2](../07-gateway.md), generalized and inverted
   (the box terminates, not the adapter) — explicitly **not** the mesh's native Circuit Relay v2,
   which relays between two libp2p *peers* and cannot carry a non-libp2p client
   ([ROLES §4](../substrate/ROLES.md) conflation warning);
2. **vanity-alias rules** for the public subdomain namespace ([§7.10.5](../07-gateway.md) generalized
   from mail local-parts to DNS labels); and
3. the **`reachability-adapter`** coordinator that ties them together and is fenced by
   [coordinator/CONTRACT.md](../coordinator/CONTRACT.md).

It adds **reach, not function**: the service runs whether or not any adapter exists. A box with a
public IP + open port needs no adapter at all.

---

## 2. Primitives, coordinators, and bindings it composes

REACH is composition, not new machinery. It reuses:

| Composed with | Role in REACH | Home |
|---|---|---|
| **`reachability-adapter`** coordinator | provides the public name + the ingress; `blind-routing` (SNI-passthrough), assurance `structural` for an own-domain name or `declared` for a bare vanity (REACH-1a). The one load-bearing new binding, fully fenced by the contract. | [CONTRACT §5](../coordinator/CONTRACT.md) |
| **Reachability ingress** pattern | the box↔adapter reverse tunnel + adapter-side SNI-demux is [§7.15.2](../07-gateway.md)'s legacy-client reachability ingress generalized from "carry one legacy mail client to a mailbox" to "carry any TLS client to any box service," and inverted — the box terminates TLS, not the adapter. **Not** the mesh's native Circuit relay role (ROLES §4 explicitly forbids conflating a legacy/non-mesh ingress with the native peer↔peer relay); the box↔adapter leg is secured as an ordinary libp2p transport (Noise + yamux), never a Circuit Relay v2 hop/stop circuit. | [§7.15.2](../07-gateway.md), [ROLES §4](../substrate/ROLES.md) |
| **Announce / Resolve** role | the box publishes a `LocationRecord` whose reachability hints include its adapter-tunnel address; monotonic-`seq` anti-rollback inherited. | [ROLES §2](../substrate/ROLES.md) |
| **Signaling** role | the reachability *ladder* — direct (rung 1) → hole-punch (rung 2) → adapter tunnel (rung 3). REACH is the rung-3 fallback for clients that cannot mesh. | [ROLES §3](../substrate/ROLES.md) |
| **RESERVE** primitive | subdomain allocation is a **single-writer bounded namespace**: the adapter is the only writer of its own DNS zone, so a name collision is a single-writer decision, never a race. | [primitives/RESERVE.md](../primitives/RESERVE.md) |
| **ATTEST** (optional) | an adapter MAY require an accountability attestation (a paid registration, a personhood claim) before granting a vanity name — an authorization input, never a content gate. | [primitives/ATTEST.md](../primitives/ATTEST.md) |

Bindings adopted rather than reinvented ([bindings/README.md](../bindings/README.md),
[DIRECTION §3](../DIRECTION.md)): a **libp2p-secured stream** (Noise transport security +
yamux/mplex stream-mux) for the box↔adapter reverse tunnel, on which the adapter runs its own
SNI-demux to plain TLS/TCP clients — **not** libp2p Circuit Relay v2, which relays between two
libp2p *peers* (HOP/STOP) and cannot carry a non-libp2p client (ROLES §4); **TLS SNI** (RFC 6066)
for content-blind stream routing; **ACME TLS-ALPN-01** (RFC 8737) so the *box* obtains its
certificate over the same SNI-passthrough path, with no zone write and no adapter key (REACH-2a);
**ACME DNS-01** (RFC 8555) only where a wildcard is needed, through the box↔adapter challenge seam
REACH-2a defines; **DMTAP-Auth** ([§13](../13-identity-auth.md)) for mutual key-authentication of
the box↔adapter tunnel; DNS wildcard delegation for the adapter's zone. REACH specifies **no** new
cryptography and **no** protocol token.

---

## 3. MOTE kinds and PUB objects it uses

REACH mints no new kinds. It rides:

- **Adapter descriptor** — a signed, **discovery-only, self-asserted** coordinator descriptor
  (kind/policy/signed tariff/region/visibility), the exact shape of the gateway directory descriptor
  ([§7.5](../07-gateway.md), [CONTRACT §2.1](../coordinator/CONTRACT.md)). It carries **no** global
  score, price-rank, or stake field.
- **`LocationRecord`** ([ROLES §2](../substrate/ROLES.md)) — the box's signed `key → location`
  record; REACH adds an adapter-tunnel `multiaddr` as one reachability hint among direct and relay
  hints. Monotonic `seq` + short TTL, resolved in the order *cached-direct → rendezvous/home-adapter
  → DHT* ([ROLES §2](../substrate/ROLES.md)).
- **Subdomain registration** — the adapter's rebuildable operational state binding
  `label → box IK → tunnel`, exactly analogous to the gateway `GatewayAliasMap`
  ([§18.3.12](../18-wire-format.md)) and to a §7.10.5 vanity registration. It is edge-independent
  state the adapter holds *for* the box, never the box's identity or data.
- **`identity` MOTE (`0x09`)** — the box IK the tunnel authenticates to; **`system` MOTE (`0x0a`)** —
  tunnel capability negotiation / control ([§2 kind table](../02-mote.md)).
- **Signed usage receipts** — if the adapter meters bandwidth/connections, receipts delivered
  directly to the paying party ([CONTRACT §6](../coordinator/CONTRACT.md)); auditable, one-directional.

The **auto-derived** name `dmtap1-<base32>.reach.example` — a deterministic encoding of the box IK
(the key-name family, [§3.9.6](../03-naming.md)) — is **conflict-free and always available**,
exactly as the key-derived alias is for mail ([§7.10.5](../07-gateway.md)). It needs no registration
and cannot be squatted.

---

## 4. Normative profile rules

- **REACH-1 — content-blind by SNI-passthrough (the core rule).** An adapter that declares
  `blind-routing` MUST route inbound connections by TLS SNI / stream onto the box's reverse tunnel
  and MUST NOT terminate TLS for that service. The **box terminates TLS and holds the certificate
  private key**; the adapter holds **no key that decrypts the tunneled stream**
  ([§7.15.2](../07-gateway.md) inverted — the box terminates, not the gateway). A `terminating`
  adapter (a TLS-terminating reverse proxy that *does* see plaintext) MUST declare `terminating` and
  MUST NOT advertise blind — **no silent downgrade** ([CONTRACT §3.2](../coordinator/CONTRACT.md)).
- **REACH-1a — cert ownership determines assurance level (normative).** The `structural` assurance
  level ([CONTRACT §3.3](../coordinator/CONTRACT.md)) for `blind-routing` holds only when the box's
  public name resolves under a zone the adapter does not control — an **own-domain** name
  ([§7.10.6](../07-gateway.md) tier 1) — **and** that zone publishes an **RFC 8657**
  `accounturi`-bound CAA record naming the box's own ACME account (SHOULD also pin
  `validationmethods` to exclude `tls-alpn-01`, REACH-2a's default and the in-path-hijackable
  method). A bare **RFC 8659** CAA record restricts only which *CA* may issue, not the validation
  *method* or *account*: because REACH-2a's default TLS-ALPN-01 challenge is forwarded over the
  same passthrough path the adapter routes, an in-path adapter can complete that challenge itself —
  under the same CAA-permitted CA, using its own ACME account — and mint a MITM certificate a bare
  CAA record never excludes. An own-domain box using an adapter MUST publish an RFC 8657
  `accounturi`-bound CAA record for the `structural` claim to hold; a bare RFC 8659 CAA alone is
  insufficient. For a **bare adapter-zone vanity** (REACH-3) the adapter is the zone's sole writer
  (REACH-7) and can complete DNS-01, HTTP-01, or TLS-ALPN-01 validation for that name itself at
  any time, terminate TLS, and re-encrypt to the box — no CAA record helps there, since the adapter
  itself writes the zone. An adapter serving a bare vanity name MUST declare
  `blind-routing` at `declared`, not `structural`, and this MUST be surfaced to the user alongside
  the REACH-4 non-portability label (§7, §8). A KOTVA-aware client SHOULD pin the box's TLS
  certificate to its `IK` via a DANE/TLSA-style hash carried in the signed `LocationRecord`
  ([ROLES §2](../substrate/ROLES.md)), so a substituted adapter-issued certificate is detectable
  even for a vanity name or an own-domain name lacking an RFC 8657 binding, and a box SHOULD
  monitor Certificate Transparency logs for its served names to detect covert issuance. These
  detect a `declared`-level breach; they do not make it `structural`.
- **REACH-2 — the tunnel is key-authenticated; the adapter authorizes, never classifies.** The
  box↔adapter tunnel MUST be mutually authenticated to the box `IK` (DMTAP-Auth, [§13](../13-identity-auth.md)).
  The adapter gates on **identity + rate only** ([CONTRACT §4](../coordinator/CONTRACT.md)); it MUST
  NOT inspect, score, re-rank, drop, or annotate tunneled content. Reachability is not moderation.
- **REACH-2a — certificate issuance: TLS-ALPN-01 over the passthrough path (normative).** The box
  MUST obtain its TLS certificate using **ACME TLS-ALPN-01** (RFC 8737) as the default: the CA
  dials the public name on 443 with `SNI` = the name and ALPN `acme-tls/1`; the adapter forwards
  this handshake over the same SNI-passthrough path as ordinary traffic (REACH-1) and the box
  answers it directly. This requires **no DNS write** in the adapter's zone, so it never makes the
  adapter a co-writer of REACH-7's single-writer namespace, and keeps the adapter blind through
  issuance as well as traffic. A box that needs a **wildcard** certificate for labels under its
  vanity (e.g. `*.alice.reach.example`, for per-service names like `svc.alice.reach.example`) MUST
  use **ACME DNS-01** instead — CA/Browser Forum policy forbids TLS-ALPN-01 for wildcards — and the
  adapter MUST expose a narrowly-scoped `_acme-challenge` provisioning API (or a `CNAME
  _acme-challenge` delegation to a zone the box controls) so the box can place its own challenge
  value without otherwise writing the zone; REACH-7's "sole writer" remains true of the zone's
  physical writes, with the adapter now acting as the box's authorized agent for one named record,
  not a silent co-issuer.
- **REACH-3 — subdomain naming is [§7.10.5](../07-gateway.md) generalized to DNS labels.** A **vanity
  subdomain** is a user-chosen label in the adapter's own zone (`alice.reach.example`). It is the
  only REACH name with ownership semantics and is fenced identically: it MUST be **dot-free** (a
  single label), valid **only** fully-qualified under the adapter's zone, **first-come and
  revocable**, and a **bare, un-anchored handle is FORBIDDEN** ([§3.13.1](../03-naming.md), the
  flat-namespace consensus problem KOTVA does not solve). A vanity **MUST yield to, and MUST NOT
  shadow, a resolvable anchored name** (DNS `name@domain` or name-chain).
- **REACH-4 — the name is a pointer, never an identity.** A REACH subdomain MUST NOT be presented as
  the box's identity ([§7.10.6](../07-gateway.md) rules (i)/(ii), generalized). Identity is the
  keypair ([§1.2](../01-identity.md)); the subdomain is a **rotatable, revocable route**. A client's
  share/QR/invite flow MUST surface the box's own domain if it has one, else its chain name, else its
  key-name — and MUST label a bare adapter vanity with its provenance ("reachability alias, issued by
  `reach.example`, not portable").
- **REACH-5 — explicit service allow-policy (default-deny).** The box MUST bind each tunnel to an
  explicit allow-list of local service(s)/port(s) it exposes; anything not listed is refused. The
  **adapter never chooses the backend** — it forwards onto a tunnel the box established for a declared
  service. A tunnel MUST NOT be a general TCP forwarder onto the box's LAN.
- **REACH-6 — anti-SSRF and fail-closed unmapping (normative, both ends).** The **adapter** MUST
  forward only onto an established reverse tunnel keyed to a registered box; it MUST NOT dial an
  arbitrary address on a client's behalf; and it MUST NOT guess or fall back to a name that could
  intercept another (the [§7.10.5a](../07-gateway.md) no-fallback rule). A REACH adapter holds no
  certificate for any name it routes blind (REACH-1), so it can complete no TLS handshake and can
  emit no application-layer (HTTP/SMTP) error or even a TLS alert for a name it will not serve. Its
  **only** fail-closed action — for a ClientHello naming an unregistered/expired name, carrying no
  usable SNI (empty, or an unmapped ECH cover name, §7), or requesting a non-allow-listed
  service/port (REACH-5) — is to **reset or close the TCP connection**: FAIL_CLOSED_BLOCK
  ([§21](../21-errors-iana.md)) realized at the transport layer. This is never
  `ERR_GATEWAY_ALIAS_UNMAPPED` (`0x0605`), whose `550 5.1.1` wire action is specific to the inbound
  legacy-mail alias path ([§7.10.2](../07-gateway.md)) and does not apply to a TLS-transport
  failure with no application protocol in play. The **box** MUST
  refuse to route a tunneled request to loopback, link-local (`169.254.0.0/16`, incl. the
  `169.254.169.254` cloud-metadata endpoint), or private ([RFC 1918](../15-references.md)) addresses
  unless the operator explicitly allow-lists them (default-deny).
- **REACH-7 — single-writer namespace (RESERVE).** Allocation of a label in an adapter's zone MUST be
  single-writer — the adapter is the only writer of its own DNS zone, so two clients cannot both be
  granted the same subdomain ([primitives/RESERVE.md](../primitives/RESERVE.md); double-booking is
  structurally impossible, not raced-away). There is **no** cross-adapter name authority: a name is
  namespaced by its adapter's domain, so distinct adapters share nothing to reconcile.
- **REACH-8 — swappable, zero lock-in.** Switching or dropping an adapter MUST be a **config change**
  with zero data migration and zero identity change ([CONTRACT §2.2](../coordinator/CONTRACT.md)). A
  box keeps its keypair, its services, and — if it uses its own domain — its name and cert; only a
  bare `@reach.example` vanity changes. The [§7.10.6](../07-gateway.md) tiers apply verbatim:
  **own domain = fully portable** (a DNS edit, no name change), **chain name = partly**, **bare
  vanity = not portable** (a disclosed convenience, not the identity).
- **REACH-9 — self-host backstop + the one disclosed scarcity.** A box with a public IP + an open
  port needs **no adapter** (rung 1, direct). Anyone with a VPS MAY run an adapter for themselves
  ([CONTRACT §2.3](../coordinator/CONTRACT.md)). The single honest exception, disclosed not papered
  over, is that a public IP + reachable ingress is a **scarce resource** a NAT'd box cannot conjure —
  the exact analog of the gateway's unblocked port 25 ([§7.15.4](../07-gateway.md), CONTRACT §2.3),
  and confined to this one kind.
- **REACH-10 — declared visibility + assurance, surfaced to users.** An adapter MUST declare exactly
  one visibility class at one assurance level ([CONTRACT §3](../coordinator/CONTRACT.md)) and clients
  MUST surface it. The RECOMMENDED profile is **`blind-routing`** (SNI-passthrough, holds no cert
  key); its assurance level MUST be declared **`structural`** for an own-domain name and MUST be
  declared **`declared`** for a bare adapter-zone vanity (REACH-1a) — the adapter is the vanity
  zone's sole writer and can mint its own certificate for that name, so nothing structurally
  excludes it there. `terminating` is opt-in + disclosed only (REACH-1).
- **REACH-11 — metered receipts, no token.** If an adapter meters bandwidth/connections it MUST issue
  **signed usage receipts** to the payer ([CONTRACT §6](../coordinator/CONTRACT.md)). Prices, quotas,
  and rate limits are operator policy; settlement is an existing stablecoin or fiat; REACH mints **no
  protocol token** and takes no cut ([DIRECTION §5](../DIRECTION.md)).

---

## 5. Scale-invariance — LAN to planet, same box

REACH is identical at every scale; only the **reach anchor** slides
([DIRECTION §6](../DIRECTION.md), [ROLES §3](../substrate/ROLES.md)):

| Function | Small / local (no coordinator) | Global (swappable adapter) |
|---|---|---|
| Reachability | rung 1 direct (own IPv6/forwarded port) or rung 2 DCUtR hole-punch | rung 3 adapter reverse-tunnel for plain clients that can't mesh |
| Naming | mDNS / DNS-SD on the LAN; your own domain | a vanity in a competing adapter's zone, or your own domain over the adapter |
| Discovery | you already know the box | published `LocationRecord`, resolved via rendezvous/home-adapter |

The box, keypair, and services never change. An adapter adds **reach**, not authority: remove it and
the service still runs and is still reachable to anyone on a path that can hole-punch; add it and
public HTTPS reach becomes *available*, never *required*. As IPv6 and port-forwarding spread, rungs
1–2 dominate and adapters **fade** ([ROLES §3](../substrate/ROLES.md)) — REACH lets the network need
*less* infrastructure over time, not more.

---

## 6. Offline / apocalypse behaviour + reconcile

Per [substrate/OFFLINE.md](../substrate/OFFLINE.md), each REACH action classifies into exactly one
grade, with **no silent degradation** and **no fabricated completion**:

| Action | Grade | Behaviour |
|---|---|---|
| Bring up / bind a local service | **`full`** | Local-first; no coordinator was ever in the path. |
| Reach it from the same LAN / mesh | **`full`** | Direct (rung 1) or hole-punched (rung 2); mDNS/DNS-SD discovery. |
| Reach it from the public internet via adapter | **`local-trust` → `blocked`** | On a LAN, degrades to local discovery; with no public path and no adapter, public reach is **`blocked`** and MUST say so — never faked into looking up. |

**Reconcile on reconnect.** The box re-dials its reverse tunnel and re-publishes its `LocationRecord`
(monotonic `seq` anti-rollback, [ROLES §2](../substrate/ROLES.md)); the subdomain registration is
**rebuildable adapter operational state**, re-established from the box's held credential — idempotent
and order-independent. REACH holds **one** cross-replica invariant, and it is discharged by
construction: because each adapter owns a distinct DNS zone (REACH-7), two adapters cannot both
authoritatively grant the same public name — there is nothing to merge, so the R-SYNC-1 hazard
("convergence is not invariant preservation", [OFFLINE §3.4](../substrate/OFFLINE.md)) cannot arise.
The tier-3 non-portability of a bare vanity ([§7.10.6](../07-gateway.md)) *is* this fact, disclosed.

Offline is a **reach property, not a magic one** ([OFFLINE honest-residual](../substrate/OFFLINE.md)):
REACH restores connectivity when a path reappears; it cannot manufacture a public path where none
exists.

---

## 7. Security + declared content-visibility

Inheriting [THREAT-MODEL.md](../THREAT-MODEL.md) (SEC-1…SEC-9); the REACH-specific posture:

- **Declared visibility: `blind-routing`, assurance scoped by cert ownership** (SEC-4,
  [CONTRACT §3](../coordinator/CONTRACT.md)). The adapter sees the SNI hostname, connection
  addresses, byte sizes, and timing; it does **not** see payload, because the box holds the TLS key
  (REACH-1). For an **own-domain** name the box controls the zone **and publishes an RFC 8657
  `accounturi`-bound CAA record** (REACH-1a) — a bare RFC 8659 CAA names only a permitted CA and
  does not bar an in-path adapter forwarding REACH-2a's default TLS-ALPN-01 challenge from
  completing issuance itself — so with that binding blindness is *structural*, provable from key
  placement and account binding. For a **bare adapter-zone vanity** the adapter is the zone's sole
  writer (REACH-7) and can mint its own certificate for the name, so blindness there is `declared`,
  not `structural` (REACH-1a) — a real, disclosed trust residual (§8), mitigated by
  `LocationRecord` TLS-key pinning for KOTVA-aware clients and CT monitoring, never structurally
  excluded for a plain client's connection.
- **Cleartext SNI is a correctness dependency, not only a privacy limit.** Blind routing demuxes on
  the TLS ClientHello's SNI (REACH-1); an ECH-encrypted inner SNI gives a `blind-routing` adapter no
  name to route on, so a blind adapter simply **cannot serve** an ECH client — only a `terminating`
  adapter holding the ECH key could, a disclosed downgrade (REACH-1). An adapter serving a
  `blind-routing` name MUST NOT publish an `ECHConfig` for it. A ClientHello with no usable SNI
  (empty, or an unmapped ECH cover name) fails closed per REACH-6: TCP RST/close, no fallback.
- **SEC-1 fail-closed.** An unregistered/expired name, no usable SNI (above), a failed tunnel auth,
  or a request to a non-allow-listed service/port fails closed (REACH-5, REACH-6), never a guess or
  best-effort.
- **SEC-2 intrinsic authenticity.** The box authenticates to its `IK` (DMTAP-Auth); the client
  authenticates the *service* by its TLS certificate, which the box — not the adapter — presents.
  For an **own-domain** name publishing an RFC 8657 `accounturi`-bound CAA record (REACH-1a), this
  is intrinsic: the adapter cannot mint a competing cert for a zone it does not write, and the CAA
  binding excludes it from completing issuance itself over the passthrough path. Without that
  binding, a bare RFC 8659 CAA does not exclude an in-path adapter from completing REACH-2a's
  default TLS-ALPN-01 challenge under a permitted CA using its own account — the same residual as
  a bare vanity. For a **bare adapter-zone vanity**, the adapter *is* the zone's writer and can
  mint its own certificate and MITM a plain client that does not pin the box's key (REACH-1a) — a
  declared, disclosed residual (§8), not structurally excluded; closed only for a KOTVA-aware client
  that verifies the `LocationRecord` TLS pin.
- **SEC-6 authorize-never-classify + swappable.** REACH-2/REACH-8/REACH-11: the adapter gates on
  identity + rate, is swappable with zero migration, and its audit is one-directional (a signed receipt
  confirms a claimed byte-count, cannot disconfirm a fabricated one — disclosed, CONTRACT §6).
- **SEC-7 abuse priced-and-localized.** Anti-abuse is authorization — authenticated tunnels,
  rate-limit tokens, optional postage for cold registration — never content classification. A poisoned
  adapter is one adapter, swappable; there is no network-wide reachability authority to poison.
- **SEC-8 replay-inert / downgrade-resistant for own-domain names with an RFC 8657 binding.**
  `LocationRecord` monotonic `seq`; DMTAP-Auth challenge-response on the tunnel; TLS with no
  downgrade into `terminating` (REACH-1, REACH-10) is cryptographically enforced when the box owns
  its zone **and** publishes the REACH-1a `accounturi`-bound CAA record — a bare RFC 8659 CAA alone
  does not enforce this, since it does not bind the validation method or account an in-path adapter
  could use. For a bare vanity, or an own-domain zone without that binding, the adapter (or, for a
  vanity, the sole zone writer) can covertly downgrade — mint a cert and terminate, while still
  declaring `blind-routing` — a conformance violation the protocol can detect — via
  `LocationRecord` TLS-key pinning and CT monitoring (REACH-1a) — but cannot structurally prevent; a
  disclosed gap (§8), not a guarantee.

---

## 8. Honest residual

- **`blind-routing` is not `blind` — the destination is exposed by construction.** SNI is cleartext
  on the wire for REACH's blind-routing mode — a correctness dependency, not only a privacy limit
  (§7) — so the adapter and any on-path observer see *which* service is reached, *when*, and *how
  much*. REACH content-blinds the payload; it does **not** hide the routing metadata or the
  connection graph to a box's services. This is declared, not hidden, and traces to the
  **metadata** ceiling — strong graph privacy against a global passive adversary is research-tier and
  non-normative ([THREAT-MODEL SEC-9](../THREAT-MODEL.md), [research/README §5](../docs/research/README.md)).
- **A bare adapter-zone vanity is `declared`, not `structural`, blind — a real MITM residual; an
  own-domain name without an RFC 8657 binding shares it.** The adapter is the sole writer of its
  own zone (REACH-7), so it can complete domain-control validation (DNS-01, HTTP-01, or
  TLS-ALPN-01) and mint a browser-trusted certificate for the vanity name at any time — terminating
  TLS itself instead of passing it through, and re-encrypting to the box. A plain client (browser,
  `git`, an S3 SDK) doing ordinary WebPKI validation does not pin the box's key and cannot detect
  this substitution; it sees no warning. An own-domain name excludes this only when its zone
  publishes an RFC 8657 `accounturi`-bound CAA record (REACH-1a): a **bare RFC 8659 CAA restricts
  only the issuing CA, not the validation method or account**, so without the RFC 8657 binding an
  in-path adapter forwarding REACH-2a's default TLS-ALPN-01 challenge can complete issuance itself
  under the same permitted CA and MITM the box exactly as for a bare vanity. `LocationRecord`
  TLS-key pinning (KOTVA-aware clients only) and CT-log monitoring *detect* a rogue or
  legally-compelled adapter, own-domain or vanity; neither *prevents* it. The specification
  discloses this as a real trust boundary rather than presenting bare-CAA blindness as structural.
- **A public IP is genuinely scarce; a NAT'd box cannot fully self-serve.** REACH-9's backstop is real
  only for a box that already has a public path; the box that most needs an adapter is the one that
  cannot be its own. The scarcity is confined to this one coordinator kind (like port-25) but does not
  vanish — the honest §7.15.4 exception, generalized.
- **Bare vanity names are not portable.** Changing adapters changes a bare public name unless the box
  owns a domain ([§7.10.6](../07-gateway.md) tier 3). The specification says so rather than dressing a
  convenience up as an identity; the durable name is the keypair/key-name, always available.
- **Reach is load-bearing for *reach*, and only for reach.** An adapter is not load-bearing for
  function or data — the service and its store survive any adapter outage — but it *is* the public path
  while it is the only one. A coordinator adds reach; here reach is precisely what it adds, so its
  outage removes exactly reach (public availability), never authenticity, custody, or state. The mitigant
  is plurality — multiple adapters + direct fallback — not a guarantee.
- **Blind means the adapter cannot help by looking.** The same property that makes an adapter
  permissionless and low-liability (it carries ciphertext it cannot read, [ROLES §4](../substrate/ROLES.md))
  means it cannot inspect abusive traffic on a victim's behalf. Abuse response is the box's and the
  recipient's, at the edge — the authorize-never-classify trade ([CONTRACT §4](../coordinator/CONTRACT.md)),
  disclosed as a cost, not solved.

Every residual traces to a root ceiling ([DIRECTION §8](../DIRECTION.md)): SNI/graph exposure is
**metadata**; the scarce public IP is the disclosed **scarce-resource** exception; bare-vanity
non-portability is the **flat-namespace** ceiling KOTVA declines to pretend away (Zooko,
[§3.9](../03-naming.md)). None is a bug in REACH; each is a consequence of not being a single
surveilling tunnel company, and is disclosed rather than solved. Maturity/precedent claims are a
2026-07 snapshot ([docs/research/README §6](../docs/research/README.md)).
