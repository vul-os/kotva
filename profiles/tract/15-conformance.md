# 15. Conformance

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 15.1 Scope

Profiles, the auditable fail-closed set, and the conformance vectors.

## 15.2 Planned profiles

A node need not implement everything. Provisional profiles: **catalogue-only** (publish and serve
offers), **transacting** (adds sealed orders), **routing** (adds delivery computation),
**gateway** (adds storefront and/or settlement).

## 15.3 The fail-closed set

Every security-relevant failure is either refused or surfaced as an explicit choice, never a silent
degradation. The table will index every must whose violation must fail closed, with the owning
clause authoritative — in particular scope-intersection failure (§9.4), rail-class substitution
(§9.3), origin isolation (§12.3), and the public-quadrant personal-data prohibition (§0.5.1).

## 15.4 Vectors

Frozen test vectors under `conformance/`, so an independent implementation proves byte-identity
rather than plausible behaviour.
