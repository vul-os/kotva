# 20. References

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 20.1 Normative

Standards an implementation must read to be conformant.

| Reference | Used for |
|---|---|
| RFC 2119 / RFC 8174 | requirement language |
| RFC 8949 | deterministic CBOR encoding |
| RFC 9052 | COSE structured signatures |
| RFC 8032 | Ed25519 |
| RFC 5545 | availability, recurrence (§3, §5) |
| ISO 3166-1 | country codes (§8, §9, §11) |
| ISO 4217 | currency codes (§5, §9) |
| DMTAP substrate | Identity, Feeds & Blobs, Sync, Roles, Wake (§0.3) |

## 20.2 Informative

| Reference | Relevance |
|---|---|
| schema.org product vocabulary | the §2 data model and existing-feed mapping |
| GS1 GTIN / SSCC | external identifier claims (§2), logistic units (§8) |
| Incoterms 2020 | risk/cost transfer (§4, §8) |
| UN/CEFACT, EDIFACT | legacy order/despatch/invoice mapping (§7) |
| RFC 9458 (Oblivious HTTP) | IP-level unlinkability (§13) |
| UPU conventions | cross-border postal legs (§8) |

## 20.3 On standards reuse

TRACT invents only where no standard exists. Where a proven specification fits, this document
**profiles** it rather than restating it — and where it profiles, the referenced standard governs
its own bytes.
