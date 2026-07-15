# 12. Operators, the Seam & Sustainability

DMTAP is deployed in one of two modes, and the boundary between them is a small, explicit
**operator seam**. This section is partly *informative* (business/licensing intent) and partly
*normative* (the inviolable rule and the seam's guarantees).

## 12.1 Two deployment modes

| Mode | Who runs it | Cost | Limits |
|------|-------------|------|--------|
| **Self-host** | The user, on their own box | $0 | none — fully functional, unrestricted |
| **Hosted operator** | A provider (e.g. a commercial control-plane) | paid | plan quotas on *operations* only |

The OSS is a **complete product on its own**. Self-host is not a crippled tier: it has every
protocol, client, and privacy feature. A hosted operator adds *convenience* (someone else runs
the box, warms the IPs, manages the domain), not *capability*.

## 12.2 The operator seam

The seam is the clean boundary the OSS exposes so an operator can add billing and multi-tenant
management **without forking or patching the protocol**. It is four capabilities (reference
crate: `crates/dmtap-seam`, contract: its `CONTRACT.md`):

- **Metering** — the OSS emits usage events at the real cost centers (gateway egress, storage,
  relay bytes, message counts, vanity domains).
- **Provisioning** — create/suspend accounts and `@`-addresses across onboarding tiers A/B/C
  (§3.8).
- **Policy** — per-account quotas/entitlements on operations (storage caps, send caps, domain
  counts, rate limits).
- **GatewayAuthz** — authorize legacy egress with per-identity-token accountability (§9),
  preserving sealed sender.

Each capability has a **self-host default that is unlimited/no-op**, so with no operator the
OSS runs unrestricted. A hosted operator implements the same capabilities — in-process (Rust
traits) or out-of-process (HTTP/events) — to add billing and quotas.

**Fail-open to function, fail-safe on security (MUST):** if the operator is unreachable, the OSS
MUST NOT break user-facing mail/chat/files. Metering queues and retries (usage may under-count
during an outage), and quota **Policy** falls back to allow (a billing concern, not a security
one). **GatewayAuthz is different and MUST NOT fail open to "allow."** GatewayAuthz is a security
control (§9): the accountability it enforces is exactly what prevents unattributable legacy
egress — the open-relay failure mode §7.7 exists to avoid. On operator-unreachable, GatewayAuthz
MUST fall back to a **safe default**: permit legacy egress only to **already-established
contacts** (and to senders carrying valid self-contained proof — postage/PoW verifiable without
the operator), and **deny cold/unproven legacy egress** for the outage window. A generic
"fail-open to allow" for this capability is prohibited.

## 12.3 The inviolable rule (normative)

Privacy, cryptography, metadata privacy, and recovery MUST NEVER be behind the seam or a
paywall. There MUST be no seam hook, quota, or plan gate capable of disabling encryption,
weakening the mixnet, reducing metadata privacy, or denying a user access to their own keys or
mailbox. The seam meters and limits **operations and organizational concerns only**. Premium is
for *running it*, never for *protection*.

A conformant operator implementation MUST NOT expose any control that violates this rule; a
conformant OSS build MUST NOT consult the seam for any privacy/crypto decision.

## 12.4 Business & licensing model (informative)

DMTAP follows an **open-software + paid-operations** model — the GitLab-style split done cleanly,
without a crippled free tier:

- **Everything a user touches, and everything trust depends on, is open** — protocol, spec,
  node, gateway, client, crypto. Shipped under the **MIT license** (Apache-2.0 dual-licensing is
  under consideration for its explicit patent grant, relevant to novel mechanisms such as
  anti-abuse postage/tokens). Libraries other implementations embed are permissively licensed
  to maximize adoption.
- **The paid layer is a thin, private control-plane** (a *hosted operator*) that implements the
  seam contract and bills **operations**: hosted nodes, storage, legacy egress / IP reputation,
  vanity domains, org/SLA. It withholds no protocol, client, or privacy feature.
- **The moat is reputation + network + being the best operator**, not the code. Because openness
  is what lets the network grow large enough to be worth hosting, full openness costs little and
  is itself a trust requirement for a privacy product (a closed client cannot credibly claim
  "we can't read your mail"). Competitors hosting DMTAP grow the network the operator is central
  to.

## 12.5 Honest limits

- **Free-rider economics:** paying (convenience) customers subsidize free self-hosters. Intended
  and viable (the convenience majority pays; the sovereign minority self-hosts), provided the
  paid tier genuinely covers cost + margin.
- **Commoditization:** open software means thin margins on the software itself; margin comes
  from operations and scale, not licenses.
- **No protocol control:** required for adoption, but it means an operator can be out-executed —
  the defense is execution and trust, not a license.
- **The open-core temptation recurs:** pressure to move "just one" feature behind the paywall
  will appear. The inviolable rule (§12.3) is the bright line; drift there is fatal to the
  brand and is prohibited for privacy/crypto features.
