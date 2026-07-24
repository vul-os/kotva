# 20. References

> **Drafting status.** The reference *tables* and the conformance rules over them (§20.1–§20.5) are
> now **normative**: the split between what an implementation must read to interoperate (§20.2) and
> what supplies context only (§20.3–§20.5) is settled, and the RFC 2119 keywords below govern it.
> Four housekeeping questions remain open (§20.7) — version-pinning policy for revisable standards,
> whether the T5 completeness check should extend past RFCs, a "checked as of" date on the legal
> table, and whether the substrate rows should pin a commit. None of them changes which references
> are normative; they change how this list is *maintained*. The key words MUST, MUST NOT, SHOULD,
> SHOULD NOT and MAY are to be interpreted as in BCP 14 (RFC 2119, RFC 8174).

## 20.1 Scope and enforcement

This section lists every standard, specification and source this document profiles or cites, split
by whether an implementation **MUST** read it to be conformant (§20.2) or it supplies context and
mapping targets an implementation does not need to parse to interoperate (§20.3–§20.5). Each entry
carries a "used for" column pointing at the section that relies on it; a reference with no pointer
MUST NOT appear here.

**A conformant implementation MUST satisfy every reference in §20.2.** It **MAY** ignore the
references in §20.3–§20.5 entirely and still interoperate — those are context, mapping targets and
evidence, not wire dependencies.

**Every RFC cited in a normative section of this document MUST appear in §20.2 or §20.3.** This
completeness rule is machine-checked: the linter's **T5** rule extracts every `RFC \d+` citation
from the rest of the document and fails the build if it is absent from this section, so an RFC
cited in body prose but missing here is caught automatically. ISO standards, schema.org terms, GS1
identifiers, and the legal instruments in §20.4 have **no** such check: their completeness here is
manual, and the same drift T5 prevents for RFCs is possible for the rest of these tables without
anyone noticing (§20.7).

## 20.2 Normative

Standards, and the two sibling specifications, an implementation MUST read to be conformant.

### 20.2.1 The DMTAP substrate — the primary dependency

TRACT is defined **on** the DMTAP substrate; it is not a standalone protocol. A conformant
implementation MUST use the substrate constructions below for every capability it implements, and
MUST NOT substitute a parallel construction for any of them — this is the substrate's **rule 2**
(*if you implement a capability's function, you speak its bytes*). This section names the
dependency and points at where each capability is consumed; **the substrate documents govern their
own bytes**, and §0.3 / §1 govern TRACT's adoption of them. Neither is restated here, and a
construction from any of these appearing inline in TRACT would be a defect (§16.2).

| Substrate capability | Document | Used for |
|---|---|---|
| **Identity** — `IK` keypair as the identity, `DeviceCert` subkeys, `name → key` over DNS + key transparency, 8-word key-name floor | [`github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md`](https://github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md) | every actor — seller, buyer, courier, distributor, gateway — is an `IK`; keys, signatures and impersonation-resistance (§0.3, §1, §16.3) |
| **Feeds & Blobs** — signed append-only author feeds (`FeedHead` / `FeedEntry` / `PubAnnounce`) and content-addressed plaintext blobs (`PubManifest`) | [`github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md) | catalogues, offers, rate cards, capacity records and reviews are public feed / blob objects; content addressing and anti-rollback (§0.3, §2, §10, §16.5) |
| **Sync** — the signed CRDT op algebra (`SyncOp`, HLC total order, the nine op kinds) | [`github.com/vul-os/dmtap/blob/main/substrate/SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) | carts across a buyer's devices and inventory across a seller's replicas; oversell-bounded counters (§6, §21.10.1) |
| **Infrastructure Roles** — announce / resolve (`LocationRecord`), signaling, circuit relay, short-TTL content-blind mailbox, cache / pin | [`github.com/vul-os/dmtap/blob/main/substrate/ROLES.md`](https://github.com/vul-os/dmtap/blob/main/substrate/ROLES.md) | reachability for NAT'd stores and buyers; the middle holds no durable data (§1.5, §21.5) |
| **Wake** — content-free, sender-blind push (`PushSubscription` / `WakePing`) | [`github.com/vul-os/dmtap/blob/main/substrate/ROLES.md`](https://github.com/vul-os/dmtap/blob/main/substrate/ROLES.md) §8 | waking a sleeping seller node when an order arrives at its mailbox (§1.5, §3.4, §18) |
| Substrate overview and rule 2 | [`github.com/vul-os/dmtap/blob/main/substrate/README.md`](https://github.com/vul-os/dmtap/blob/main/substrate/README.md) | the à-la-carte adoption contract and the abuse-undertaking model (§0.3, §14) |

### 20.2.2 WRAP — the work-coordination sibling

Work-coordination — request → bid → assign, on-demand delivery dispatch, the custody-handoff
lifecycle, and both auction directions — is defined by **WRAP**, on the same substrate and the same
`IK` identity, and **not** by this document. TRACT defines no bidding, dispatch or custody object of
its own. An implementation that participates in on-demand delivery, competitive procurement, forward
auctions, or a custody handoff MUST speak WRAP's objects; **WRAP governs its own bytes**.

| Sibling specification | Used for |
|---|---|
| [`github.com/vul-os/wrap`](https://github.com/vul-os/wrap) | on-demand delivery dispatch (`WorkOrder → Bid → Assignment`, issuer-assigns, §8.9); the custody-handoff lifecycle (`Progress` / `Attestation`, referenced from §8.4 and §18.4); competitive procurement / RFQ (§5.9a); the optional forward-auction profile over `Bid` (§5.9a) |

### 20.2.3 Wire, cryptographic and coded-value standards

The standards §16 profiles for bytes, signatures and the coded-value primitives. An implementation
MUST read each to encode, decode or validate the objects that cite it.

| Reference | Used for |
|---|---|
| RFC 2119 / RFC 8174 (BCP 14) | requirement-language keywords, interpreted only when written in capitals (§0.9) |
| RFC 8949 §4.2 | deterministic CBOR encoding — the wire format every object in this document is serialised as (§1.3, §16.2) |
| RFC 9052 | COSE structured signatures over CBOR objects (§1.3, §16.3) |
| RFC 8032 | Ed25519 — the one signature scheme this document assumes (§1.3) |
| RFC 5545 | `VAVAILABILITY` / `VFREEBUSY` payloads for time-slot availability, `RRULE` recurrence for capacity intervals and subscription periods (§3.4, §3.5, §5.6, §16.5.2) |
| BCP 47 (RFC 5646) | UTF-8 language tags on localized display `name` / `description`, with multi-localization; the content-addressed product **identity is language-neutral** and excludes this text (§2.3a, §0.9). **PROVISIONAL as a wire dependency:** the carrying grammar slot on `ProductRecord` is a logged, not-yet-landed §16 change (§2.3a); until it lands, the frozen v0 `ProductRecord` carries display text with no tag. |
| ISO 3166-1 | country and place codes — the `country` primitive (§4.9, §11.2, §13.4, §16.3) |
| ISO 4217 | currency codes — the `money` primitive; one currency per order, cross-currency arithmetic refused (§5.3, §16.3, §16.7) |

## 20.3 Informative — commerce and data-model standards

Context and mapping targets. An implementation interoperates without parsing these directly; a
seller or gateway with an existing feed in one of these vocabularies uses them to translate in.

| Reference | Used for |
|---|---|
| schema.org product vocabulary (`Product`, `Offer`, `ProductGroup`, `hasVariant`, `variesBy`) | the §2 data model, so existing merchant feeds map in by translation (§2.4) |
| GS1 GTIN / MPN | external product-identifier claims — advisory, unverified, squattable, never authoritative; the `ClaimedExternalRung` an index MUST treat as an advisory join key only (§2.3, §2.4, §16.5.1) |
| GS1 SSCC | logistic-unit identification for physical consignments, where the parties already use it (§8) |
| Incoterms 2020 | the risk / cost transfer point on shipped goods, distinct from the place-of-supply anchor it is easily confused with (§4.6, §4.9, §11.2) |
| UN/CEFACT, EDIFACT | legacy order / despatch / invoice mapping targets, for a seller whose counterparty is an ERP or a 3PL rather than another TRACT node (§7) |
| UPU conventions | cross-border postal-leg conventions (§8) |
| RFC 9458 (Oblivious HTTP) | an intended profile — not yet evaluated — for IP-level unlinkability at the transport, ahead of whatever the application-layer analytics grant already withholds (§13.7) |

## 20.4 Informative — legal instruments

Named because §11 and §21.11's legal-grounding pass cite them for specific, narrow propositions.
None of this is legal advice, and §21.11.5 lists the caveats that bound how far each reading
travels — **no case law has applied any of them to a permissionless no-operator protocol** (§21.11,
brief C6). This table is evidence, not protocol text (C3): nothing in §20.4 is normative, and §11
carries only the wire-shaping conclusions, never the analysis.

| Reference | Used for |
|---|---|
| GDPR Art 4(1) | the working definition of personal data this document uses throughout, including a pseudonymous key linkable to a person (§0.9) |
| GDPR Art 17 (right to erasure) | the conflict between erasure rights and irrevocable content-addressed objects — unresearched after three passes, and the most likely hard blocker on the no-operator design (§0.5.1, §11.5, §21.11) |
| POPIA §1 (South Africa) | personal-data definition (§0.9); South African regime accommodation (§11.4) |
| LGPD Art 5 (Brazil) | personal-data definition (§0.9) |
| Cal. Rev. & Tax. Code §6041(b) | the contract-gated US marketplace-facilitator test — the strongest structural argument for a gateway no seller has contracted with falling outside the definition (§11.2a, §21.11.1) |
| Wash. RCW 82.08.010(15)(a) | the three-prong US facilitator test, all three required simultaneously per the state's own guidance (§11.2a, §21.11.2) |
| N.Y. Tax Law §1101(e)(1)(B) | a facilitator trigger met by contracting with a third party to collect receipts, catching a gateway whose checkout merely routes through its own payment provider (§11.2a, §21.11.2) |
| Tex. Tax Code §151.0242(a)(2) | a facilitator trigger for directly or indirectly processing sales or payments, with no payment-processor carve-out (§11.2a, §21.11.2) |
| Council Directive 2006/112/EC Art 14a (EU VAT Directive) | deemed-supplier scope for imported consignments of intrinsic value ≤ €150 and intra-EU supplies by non-EU-established sellers (§11.2b, §21.11.3) |
| Council Implementing Regulation (EU) 282/2011 Art 5b | the cumulative not-facilitating test and the "economic reality and influence" standard, which the Commission's Explanatory Notes read as rejecting a purely contractual escape (§11.2b, §21.11.3) |

## 20.5 Informative — research literature and deployed prior art

Cited for one specific, checked claim each — that a mechanism exists and has shipped, or that a
deployed system behaved a measured way — not as a general endorsement. **Nothing in this table is
normative, and per §21.1 and brief C6 no section may cite these — or §21 — as *support* for
logistics (§8), analytics (§13), or trust / dispute / tax (§9.6, §10, §11), which returned nothing
verified.** They ground the design's *limits*, not its claims.

### 20.5.1 Mechanism literature

| Reference | Used for |
|---|---|
| O'Neil, "The Escrow Transactional Method" (1986) | the origin of the escrow / demarcation family the bounded-counter inventory mechanism descends from (§6.2a, §21.10.1) |
| Barbará-Millá & Garcia-Molina, "The Demarcation Protocol" (1994) | the demarcation protocol that family passes through on the way to a CRDT (§6.2a, §21.10.1) |
| Balegas et al., SRDS 2015 | `BoundedCounter`, shipped in production as AntidoteDB's `antidote_crdt_counter_b`, and the measured ~200-decrement oversell in a plain eventually-consistent counter at 200 concurrent clients that it corrects (§6.2a, §21.10.1) |

### 20.5.2 Deployed prior art

The systems whose measured outcomes bound what this design can claim. Each is a **postmortem or a
counter-example**, never a live comparison, and each is a load-bearing input to the honesty
requirements of brief C6.

| Reference | Used for |
|---|---|
| OpenBazaar (2016–2021) — the closest deployed relative: signed objects, content-addressed listings, keypair identity, no operator | the postmortem behind the two weakest claims — discovery re-centralising on a default crawler, and availability bounded by publisher liveness (~22-day median listing lifetime) — and the reputation / opt-in-escrow failure modes §10 and §9.6 must state as measured, not hypothetical (§21.3, §21.5, §21.6) |
| Beckn / ONDC — the largest deployed "decentralised commerce" network | the counter-example: it avoids OpenBazaar's failure modes only by a **central approval-gating permissioned registry** — the opposite of this specification's no-operator claim — which is why "any node MAY build an index" is not shown to prevent re-centralisation (§21.4, §21.8) |
| Nostr NIP-15 (marketplace) | the in-repo caution: NIP-15 is marked "unrecommended: too complicated" and moves checkout off the public feed; TRACT's signed-transition order model (§7) is a deliberate departure, but the complexity warning is real and not ignored (§21.7) |

## 20.6 On standards reuse

TRACT invents only where no standard exists. Where a proven specification fits, this document
**profiles** it rather than restating it — and where it profiles, **the referenced standard governs
its own bytes**. This applies most strongly to the two siblings in §20.2.1–§20.2.2: the DMTAP
substrate and WRAP are not "influences" but hard dependencies whose bytes TRACT consumes unchanged
and never re-specifies (C1, C2).

## 20.7 Open

These are maintenance-policy questions about the list, not questions about which references are
normative. Recorded for the founder-decision pass (brief §5).

- **Version-pinning for revisable standards.** Whether the informative tables need version pinning
  the way RFCs are immutably numbered. Incoterms is revised on a roughly ten-year cycle and §4.6
  currently names "Incoterms 2020" without a stated policy for what happens to a signed offer when
  Incoterms 2030 exists. *Recommendation:* pin the version in the offer's own fields (so a signed
  offer commits to the edition it was priced against) and let the reference table name the current
  default only — but this is a §16 / §4.6 shape question, not a §20 one, so it is deferred there.
- **Extending T5 beyond RFCs.** Whether the completeness check should extend to ISO standards,
  schema.org terms and GS1 identifiers, which have the identical drift risk and none of the
  automated protection (§20.1). *Recommendation:* extend it — the marginal linter cost is low and
  the manual-drift surface is the larger half of this section.
- **A "checked as of" date on the legal table (§20.4).** §21.11.5 notes Washington already amended
  its definition once in 2026 and ViDA will change EU scope — a reference list of statutes is not
  append-only the way a list of RFCs is. *Recommendation:* stamp each §20.4 row with the date its
  reading was verified, mirroring §21.11's own dating.
- **Pinning the substrate rows to a commit.** Whether the §20.2.1 rows should name a specific
  substrate version or commit once the substrate specification stabilises, rather than a bare
  `main` reference. *Recommendation:* pin to a tagged substrate release at TRACT's own v1 freeze —
  a floating `main` reference under a frozen wire format (§16) is the same moving-target problem
  freezing was meant to remove — but hold the bare reference until the substrate cuts that tag.
