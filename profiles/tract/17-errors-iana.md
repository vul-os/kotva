# 17. Errors & registries

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 17.1 Scope

The `ERR_TRACT_*` code block, the responder-action vocabulary, and the extension registries.

## 17.2 Conventions

Codes are allocated in a subsystem block, each with: name, operation, meaning, retryability, and a
responder action drawn from a closed vocabulary (fail-closed-block, drop-silent, rotate-retry,
deny-policy, halt-alert).

## 17.3 Registries planned

Item kinds · availability variants · fulfilment variants · consideration variants · rail classes ·
tax treatment categories · excluded-category vocabulary · external-identifier schemes.

## 17.4 The rule that makes registries safe here

Every registry above is **extensible**, and an unrecognised value must not be fatal to a generic
index — it is preserved and surfaced as unknown. But a client that *renders or transacts* against a
variant it does not implement must refuse rather than guess. Tolerant to store, strict to act.
