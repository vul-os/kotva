# 12. Gateway

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 12.1 Scope

The storefront role: rendering signed objects for browsers, domains, and the trust boundary.

## 12.2 What this section will specify

- The rendering contract: what a gateway must verify before serving, and what it publishes about
  what it served.
- **Render bundles**: seller-supplied presentation, sandboxed.
- **Subdomain** allocation (single-label, one wildcard certificate) and **custom domain** binding.
- Escrow co-location: a gateway that also settles is the facilitator of §11.3.

## 12.3 Origin isolation (will be normative)

Merchant render bundles are untrusted code. **Every store must get its own origin.** A gateway
serving multiple stores from one origin lets one bundle read another store's cart and session; it
is non-conformant, not merely ill-advised.

## 12.4 Process isolation (will be normative)

A gateway terminates untrusted connections and renders untrusted bundles. It must run as a separate
process with no access to identity keys or the object store. "One binary, several roles" never
means one address space.

## 12.5 The honest limit

DMTAP's gateway self-extinguishes as adoption grows. **TRACT's does not**, because browsers are
permanent and a shopper without a keypair cannot verify a signature. The mitigation is
**detectability, not prevention**: any node can re-render the same store from the same signed
objects and be compared byte-for-byte. A native client needs no gateway; two native parties never
need one to transact.

## 12.6 Open

- Whether the spec should define a canonical render so byte-for-byte comparison is meaningful, or
  only require that the underlying objects be exposed for independent verification.
