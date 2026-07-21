# 20. References

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 20.1 Scope and enforcement

Every standard this document profiles or intends to profile, split by whether an implementation
must read it to be conformant (§20.2) or it supplies context and mapping targets an
implementation does not need to parse to interoperate (§20.3–§20.5). Each entry carries a "used
for" column pointing at the section that relies on it, so a reference with no pointer is a
reference that should not be here.

**This completeness is partly machine-checked.** The linter's T5 rule extracts every `RFC \d+`
citation from the rest of the document and fails the build if it is absent from this section —
an RFC cited in body prose but missing here is caught automatically. ISO standards, schema.org
terms, GS1 identifiers, and the legal instruments in §20.4 have no such check: their completeness
here is manual, and the same drift T5 exists to prevent for RFCs is possible for the rest of this
table without anyone noticing.

## 20.2 Normative

Standards an implementation must read to be conformant.

| Reference | Used for |
|---|---|
| RFC 2119 / RFC 8174 (BCP 14) | requirement-language keywords, interpreted only when written in capitals (§0.9) |
| RFC 8949 §4.2 | deterministic CBOR encoding — the wire format every object in this document is serialised as (§1.3, §16.2) |
| RFC 9052 | COSE structured signatures over CBOR objects (§1.3, §16.3) |
| RFC 8032 | Ed25519 — the one signature scheme this document assumes (§1.3) |
| RFC 5545 | `VAVAILABILITY` / `VFREEBUSY` payloads for time-slot availability, `RRULE` recurrence for capacity intervals and subscription periods (§3.4, §3.5, §5.6, §16.5.2) |
| ISO 3166-1 | country and place codes (§4.9, §11.2, §13.4, §16.3) |
| ISO 4217 | currency codes (§5.3, §16.3) |
| DMTAP substrate | Identity, Feeds & Blobs, Sync, Infrastructure Roles, Wake — the five capabilities TRACT adopts unchanged rather than reinventing (§0.3) |

## 20.3 Informative — commerce and data-model standards

Context and mapping targets. An implementation interoperates without parsing these directly; a
seller or gateway with an existing feed in one of these vocabularies uses them to translate in.

| Reference | Used for |
|---|---|
| schema.org product vocabulary (`Product`, `Offer`, `ProductGroup`, `hasVariant`, `variesBy`) | the §2 data model, so existing merchant feeds map in by translation (§2.4) |
| GS1 GTIN / MPN | external product-identifier claims — advisory, unverified, squattable, never authoritative (§2.4) |
| GS1 SSCC | logistic-unit identification for physical consignments, where the parties already use it (§8) |
| Incoterms 2020 | the risk/cost transfer point on shipped goods, distinct from the place-of-supply anchor it is easily confused with (§4.6, §4.9, §11.2) |
| UN/CEFACT, EDIFACT | legacy order/despatch/invoice mapping targets, for a seller whose counterparty is an ERP or a 3PL rather than another TRACT node (§7) |
| UPU conventions | cross-border postal-leg conventions (§8) |
| RFC 9458 (Oblivious HTTP) | an intended profile — not yet evaluated — for IP-level unlinkability at the transport, ahead of whatever the application-layer analytics grant already withholds (§13.7) |

## 20.4 Informative — legal instruments

Named because §11 and §21.11's legal-grounding pass cite them for specific, narrow propositions.
None of this is legal advice, and §21.11.5 lists the caveats that bound how far each reading
travels — no case law has applied any of them to a permissionless no-operator protocol.

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

## 20.5 Informative — research literature

Cited for one specific, checked claim each — that a mechanism exists and has shipped — not as a
general endorsement of the surrounding paper.

| Reference | Used for |
|---|---|
| O'Neil, "The Escrow Transactional Method" (1986) | the origin of the escrow/demarcation family the bounded-counter inventory mechanism descends from (§6.2a, §21.10.1) |
| Barbará-Millá & Garcia-Molina, "The Demarcation Protocol" (1994) | the demarcation protocol that family passes through on the way to a CRDT (§6.2a, §21.10.1) |
| Balegas et al., SRDS 2015 | `BoundedCounter`, shipped in production as AntidoteDB's `antidote_crdt_counter_b`, and the measured ~200-decrement oversell in a plain eventually-consistent counter at 200 concurrent clients that it corrects (§6.2a, §21.10.1) |

## 20.6 On standards reuse

TRACT invents only where no standard exists. Where a proven specification fits, this document
**profiles** it rather than restating it — and where it profiles, the referenced standard governs
its own bytes.

## 20.7 Open

- Whether the informative tables need version pinning the way RFCs are immutably numbered.
  Incoterms is revised on a roughly ten-year cycle and §4.6 currently names "Incoterms 2020"
  without a stated policy for what happens to a signed offer when Incoterms 2030 exists.
- Whether T5's completeness check should extend beyond RFCs to ISO standards and GS1
  identifiers, which have the identical drift risk and none of the automated protection (§20.1).
- Whether the legal-instrument table (§20.4) needs a "checked as of" date given §21.11.5's own
  caveat that Washington already amended its definition once in 2026 and ViDA will change the EU
  scope — a reference list of statutes is not append-only the way a reference list of RFCs is.
- Whether the DMTAP substrate row needs to name a specific version or commit once the substrate
  specification itself stabilises, rather than a bare directory reference.
