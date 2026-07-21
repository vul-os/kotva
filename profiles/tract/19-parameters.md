# 19. Parameters

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 19.1 Scope

Every tunable in one table: timeouts, size ceilings, rate limits, retention windows, defaults.

## 19.2 Why this is its own section

Parameters scattered through prose drift out of agreement with each other. Collecting them makes
the linter able to check that a value cited in §N matches the table, and makes an implementer's
configuration surface reviewable in one place.

## 19.3 Categories

Order and escrow timeouts · reservation hold durations · quota rebalance thresholds · object size
ceilings · per-publisher storage quotas · feed append rates · rate-card refresh intervals · grant
lifetimes · clock-skew tolerance.
