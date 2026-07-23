# 12. Gateway

> **Drafting status.** Normative for the bridge property (§12.2), the rendering contract (§12.2a),
> settlement co-location and the facilitator consequence (§12.2b), tax computation as edge policy
> (§12.2c), origin isolation (§12.3), process isolation (§12.4), and subdomain / custom-domain
> binding (§12.4a). One decision is **still scoped** — whether the spec fixes a canonical render so
> byte-for-byte comparison is well-defined, or only requires the underlying objects be exposed for
> independent verification (§12.6). The honest, non-decaying limit of this role is §12.5. The key
> words MUST, MUST NOT, SHOULD, SHOULD NOT and MAY are to be interpreted as in BCP 14 (RFC 2119,
> RFC 8174) when, and only when, in all capitals.
>
> This section adds **no wire bytes.** A gateway is named on the wire only where §16 already names
> it — `Responsible.facilitator` and `EscrowScope.operator` (§16.6, §16.5.4) — and everything a
> gateway serves is a substrate blob or an object §16 already defines. Where a slot would be needed
> and does not exist, it is recorded as an open §16 item (§12.4a), not invented here.

## 12.1 Scope

The storefront role — rendering signed objects for keyless browsers, over domains, across the trust
boundary a gateway introduces — together with its co-located settlement and escrow function, and
the rules that confine a gateway to **authorizing** transactions without being **privileged over**
the content it serves. Escrow mechanics, rail classes and the payment seam are §9; the tax facts a
settling gateway acts on are §11; the operator class itself is defined in §0.4.2, which this section
does not re-derive.

## 12.2 The one role that needs scarce resources, and the bridge property

A gateway is a role of the same software as every other role (§0.4.2), with one difference that
defines it: it is the **only** TRACT function that requires resources not derivable from a keypair —
a domain with TLS and uptime for the storefront, and, where it settles, a payment-provider
relationship, a money float, and jurisdiction-specific licensing. It does two things — **storefront**
rendering (§12.2a) and **settlement + escrow** (§12.2b, §9) — which bundle only because the same
commercial and legal standing underwrites both, never because one implies the other (§15). A gateway
MUST NOT be treated, described, or operated as a marketplace, a registry, an index, a courier, or a
reputation authority; it is none of those (§0.4.2, §0.9).

A gateway is a **deprecatable bridge**, in the same sense as the legacy-egress gateway of the
substrate profile it descends from: it exists to reach the legacy web and keyless shoppers, and to
reach legacy settlement rails. The bridge property is normative:

- A store MUST remain fully functional with **no** gateway. Two TRACT-native parties MUST be able to
  discover, order, and settle without one; a gateway's presence is a convenience for the legacy web
  and for buyers who want recourse, never a precondition of trade (§0.6).
- A gateway MUST NOT be able to delist, suspend, edit, reorder, or otherwise exercise authority over
  a seller's feed. The store **is** the feed (§2), which the gateway merely renders.
- Moving a store between gateways MUST cost no more than a DNS change and MUST NOT require migrating
  any data, because nothing durable about the store lives at the gateway (§12.4a, §0.7).
- A gateway MUST NOT hold, escrow, or have any access to a party's identity keys — ever (§0.4.2,
  §12.4).

## 12.2a The rendering contract — authorizes, but is not privileged over content

Before serving any public object, a gateway MUST verify it exactly as any node must: signature and
content-address per §16.2, and feed-head ordering per §2 (an older signed head is never served over
a newer one already observed). A gateway MUST refuse to serve an object that fails verification; it
MUST NOT "repair", re-sign, or serve it anyway.

A gateway **authorizes** — it may, as its own commercial and legal choice, decline to render a given
store or decline to settle a given order — but authorizing is not authority over the content:

- A gateway MUST NOT modify, reorder, suppress, truncate, or inject into the signed objects it
  renders. It serves what the feed says, or it refuses to serve at all. There is no third option in
  which it serves altered content.
- A gateway's refusal to serve or settle MUST NOT be presentable as a property of the objects
  themselves: the same signed objects remain servable and verifiable by any other node, and a
  disagreement between what a gateway presents and what the seller's feed contains resolves in
  favour of the feed (§0.4.1, §2.6).

**Render bundles.** A store's presentation — its HTML, CSS, and client script — is seller-supplied
and untrusted code. It is carried as a substrate content-addressed blob referenced from the seller's
feed, not as a TRACT-native object (substrate `FEEDS.md`,
`https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md`); TRACT defines no bytes for it. A
gateway executing or serving a render bundle MUST do so under the origin and process isolation of
§12.3 and §12.4, and MUST NOT grant a bundle any access to identity keys, to another store's data,
or to the object store.

## 12.2b Settlement co-location, and the facilitator consequence

A gateway that settles an order's payment **is** the facilitator of §11.3. This is a grammar-level
consequence, not a labelling convention:

- When a gateway settles an order, that order's `Responsible.facilitator` (§16.6) MUST name the
  gateway's `identity-key`. When no gateway settles — a self-hosted seller taking direct payment for
  their own goods — `facilitator` MUST be absent. Its presence is the marketplace-facilitator hook
  and its absence is equally load-bearing (§16.6, §21.11.2).
- A settling gateway MUST publish an `EscrowScope` (§16.5.4) declaring the buyer, seller and supply
  countries, currencies, rail classes, value ceiling and excluded categories it can lawfully serve,
  and MUST refuse an order that falls outside it. An empty intersection between the buyer's declared
  scope and the gateway's offered scope MUST be **disclosed** to both parties, never silently
  downgraded to an unescrowed trade (§9.4; `ERR_TRACT_ESCROW_SCOPE_EMPTY`, §17). Every release,
  refund or split ruling SHOULD be published as a signed object so a badly-ruling operator
  accumulates a verifiable record — an intended guarantee still gated on the §16 escrow-object gap
  that §9.4.3, §9.5 and §18.5 record as **PROVISIONAL**; this section adds no escrow bytes and
  defers the lifecycle to §9 and §18.5.

**The honest legal consequence, disclosed not computed.** Settlement and escrow are the trigger that
most plausibly makes a gateway a *marketplace facilitator* for tax purposes, and in some
jurisdictions escrow alone suffices; "render-only never touches funds" is not a universal safe
posture, and "the contract is between two keypairs" does not defeat a test that measures economic
influence. None of this is settled law. The reasoning and its substantial caveats are evidence and
live in §11.2a and §21.11; this section states only the wire-shaping conclusion above and MUST NOT
be read as making any deployment compliant (§11.2a).

## 12.2c Tax computation happens here, as edge policy

TRACT carries tax **facts** and never computes, collects, or remits tax (§11.2b, C3). The four
anchors, a treatment **category** (never a rate), and the responsible parties an order names are the
entire tax surface on the wire (§16.6, §5.10).

A settling gateway is the natural **edge** at which that computation happens — exactly the party
§11.2b names alongside the seller. Therefore:

- A gateway that settles MAY compute, collect, invoice, and remit tax as **local policy** over the
  §11 fields, using any rate source, schedule, or commercial service it trusts. The protocol defines
  none of that, and a gateway MUST NOT rely on any TRACT object to carry a tax **rate**, a
  threshold, or a remittance instruction — there is no such field, by design.
- A gateway performing tax computation MUST derive place of supply from the order's `Anchors`
  (§16.6), which is itself derived from the offer's `Fulfilment` (§4, §16.5.2), and MUST NOT
  substitute a party's country for it; resolving place of supply from the parties instead is
  plausibly and consistently wrong about every event or service supplied abroad (§16.5.2, §11.2).

## 12.3 Origin isolation

Merchant render bundles are untrusted code (§12.2a). **Every store MUST get its own origin.** A
gateway that serves two stores from one origin lets one store's bundle read another store's cart and
session; such a gateway is **non-conformant**, not merely ill-advised, and the failure is reported
as `ERR_TRACT_ORIGIN_NOT_ISOLATED` (`0x0B01`, §17), fail-closed-block, and is part of the gateway
profile's auditable fail-closed set (§15.3).

This section states not only *that* origins must differ but *how* they are compared, because the
requirement is otherwise satisfiable on paper and violable in practice. A naive byte comparison
reports `alice.example` and `Alice.example` as distinct origins, while DNS and every browser treat
them as the same one — same storage partition, same cart, same session. A gateway that allocated
both would believe it had isolated two stores and would in fact have handed one merchant's bundle
read access to the other's data, while passing its own conformance check. Therefore a gateway MUST
compare origins **after normalisation**:

- the host label MUST be ASCII-lowercased, and any trailing root-label dot removed, before
  comparison;
- an internationalised name MUST be put through an IDNA pass to its A-label form **before** that
  lowercasing, since a gateway that accepts Unicode hosts without one has the same hole in a less
  obvious form;
- two stores whose hosts compare equal after this normalisation MUST NOT be allocated as distinct
  origins, and a gateway MUST refuse to serve rather than collide them.

## 12.4 Process isolation

A gateway terminates untrusted connections and renders untrusted bundles. It MUST run the rendering
function as a separate process with **no** access to identity keys and **no** access to the object
store. "One binary, several roles" MUST NOT be read as one address space: a role that terminates
untrusted input and a role that holds a key MUST NOT share memory. A gateway that co-locates its
settlement function MUST likewise isolate the key material and float that function requires from the
rendering process.

## 12.4a Subdomains and custom domains

A gateway MAY offer stores a subdomain and MAY accept a store's own custom domain. Both are
**gateway-local configuration with no wire representation**: neither has a production in §16, because
the store's identity is its feed (§2), not its hostname. This is deliberate and is what makes the
portability of §12.2 a DNS change rather than a data migration.

- A gateway offering subdomains MUST allocate each store a **single DNS label** under one parent
  domain, so that a **single wildcard TLS certificate** covers every store, and each store's
  subdomain MUST be its own origin under §12.3. A gateway MUST NOT place two stores at path prefixes
  of one origin (that violates §12.3).
- A gateway accepting a custom domain MUST verify the seller controls that domain before binding it
  (an ACME or DNS challenge; the mechanism is gateway-local and out of scope, like TLS provisioning
  itself). Binding a custom domain MUST NOT alter the store's identity: the store remains the feed,
  and the domain is presentation only.

*(Recorded as an open §16 item, not invented here: a storefront render bundle is treated above as a
substrate content-addressed blob and needs no TRACT grammar. If a future decision requires render
bundles to be TRACT-native signed public objects — for instance to make the byte-for-byte
re-rendering of §12.6 comparable against a fixed shape — §16.5 has no production for one, and adding
it is a MAJOR change under §16, C5.)*

## 12.5 The honest limit — this operator class does not decay

The substrate's legacy gateway self-extinguishes as adoption grows. **TRACT's does not**, and this
section says so rather than letting a reader discover it (§0.4.3):

- The **storefront** function is permanent, because browsers are permanent. A shopper without a
  keypair cannot verify a signature, so they trust that the gateway rendered honestly. That is a
  real trust downgrade, disclosed here as one. It is **mitigated, not removed**, by detectability:
  because the underlying objects are public and content-addressed, any node MAY re-render the same
  store from the same signed objects and be compared against what the gateway served (§12.2a). The
  mitigation is detection, never prevention — a native client needs no gateway, and two native
  parties never need one to transact.
- The **settlement / escrow** function is permanent, because holding money for strangers is a
  licensed activity and physical custody cannot be made trustless (§9.6).

TRACT is therefore **structurally less pure than the substrate it stands on**, and this is stated,
not hidden. What is preserved is that the class is *one*, entered permissionlessly, competed for,
chosen per-order by both parties, replaceable without loss of the store (§12.2), and never in
possession of identity keys (§12.4). This is design reasoning checked for internal consistency; the
trust and legal surfaces a gateway sits on returned nothing verified across the grounding passes and
this section does not claim otherwise (§21.1, §21.11).

## 12.6 Open

- **PROVISIONAL — pending decision.** Whether this specification should fix a **canonical render**
  — a deterministic, reproducible mapping from a store's signed objects to bytes — so that the
  byte-for-byte comparison the §12.5 mitigation relies on is well-defined, **or** should require only
  that the underlying signed objects be exposed for independent re-rendering and leave presentation
  bytes unconstrained. A canonical render makes the mitigation crisp but freezes presentation and is
  fragile against every browser and CSS revision; exposing the objects keeps presentation free but
  makes "compared byte-for-byte" an assertion about the objects, not the rendered page. Until this is
  decided, §12.5's mitigation is stated over the **underlying objects**, and no canonical-render
  requirement is normative. Recorded for the founder-decision list.
