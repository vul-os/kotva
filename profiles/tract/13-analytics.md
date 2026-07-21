# 13. Analytics

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 13.1 Scope

Measurement for sellers that does not require per-visitor surveillance.

## 13.2 Posture

Tiered and buyer-granted, with an aggregate floor: anonymous browse by default; scoped signed
disclosure the buyer's node chooses to attach; full detail at order, which the seller needs to
fulfil anyway; aggregate telemetry carrying no per-visitor record.

## 13.3 What this section will specify

- The grant object: what a buyer's node may disclose, to whom, for how long, revocably.
- The aggregate telemetry object and its privacy parameters.
- Anonymous rate-limiting credentials for bot and abuse signals without identity.

## 13.4 Standards profiled

Privacy-preserving aggregation and attribution work from the browser vendors; anonymous credential
schemes for un-attributed rate limiting; Oblivious HTTP (RFC 9458) where IP-level unlinkability is
wanted.

## 13.5 What a seller loses, stated plainly

No cross-site retargeting; campaign-level rather than person-level attribution; no individual
session replay; coarser fraud signals on non-reversible rails. Aggregation needs volume, so
small sellers get noisier numbers. A section claiming parity with hosted analytics would be false.
