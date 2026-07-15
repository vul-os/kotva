# 14. Scaling & Deployment

DMTAP must work for a user with only a phone, a user with an always-on box, and an operator
hosting millions of mailboxes — and the gateway fleet must scale horizontally. This section
specifies the node classes, the reachability/buffer layers, and the scaling patterns, grounded
in how production mail and P2P systems actually operate.

## 14.1 Node classes

| Class | Examples | Role | Durable state? |
|-------|----------|------|:--------------:|
| **Always-on node** | Pi, NAS, home server, VPS | The authority — holds the mailbox, does the work, participates in the mesh | **Yes** |
| **Intermittent node / thin client** | Laptop, phone | A client that syncs to an always-on node it trusts (usually the owner's) | No (cache only) |
| **Relay** | any public-IP node | Reachability hop for NAT'd nodes; content-blind | No |
| **Mix** | staked operator node | A mixnet hop (§6); content-blind | No |
| **Gateway** | accountable operator | Legacy SMTP bridge (§7); the only non-content-blind role | No (short retry queue only) |

A phone is **never** a durable endpoint (§14.3). An always-on node is the durable authority
(§14.4). Relay/mix/gateway are stateless middle roles that scale horizontally (§14.2, §14.5).

## 14.2 Horizontally-scalable gateways (normative patterns)

The gateway (§7) is stateless and MUST scale as a **shared-nothing worker fleet**:

1. **Identical, interchangeable instances.** Every gateway instance runs the same config; there
   is no whole-cluster coordination on the hot path (coordination is a scaling bottleneck —
   the KumoMTA design principle). Add instances to add capacity.
2. **IPs owned by an egress layer, not the instances.** Outbound source IPs are selected by an
   egress-proxy layer (weighted round-robin over **egress pools** of warmed IPs), so worker
   instances stay stateless and IPs are managed independently of instance count.
3. **DKIM keys distributed read-only** to all instances (never generated per-instance), so
   signing is stateless-safe.
4. **Per-ISP warmup state.** An IP is "warm for Gmail, cold for Outlook" independently; pool
   sizing and warmup track **per receiving ISP** (the SES managed-pool model), spilling
   overflow to a shared pool during warmup.
5. **One coordinating tier for the irreducible shared state.** IP reputation and rate/throttle
   counters are the *one* thing a shared-nothing fleet cannot make node-local; hold them in a
   small shared store (Redis-style) feeding automated **health-check pool demotion**
   (Good/Poor/New by bounce & complaint rates), not in the workers.
6. **Inbound MX at scale:** DNS MX priority + equal-preference randomization (RFC 5321 §5.1) for
   coarse load spreading, Anycast receiving IPs, and accept-then-forward MTAs behind a
   source-IP-preserving LB (XCLIENT/PROXY protocol) so downstream filtering still sees the true
   sender.

**Honest limits (MUST disclose):** IP reputation is irreducibly shared mutable state — the one
non-local part of an otherwise shared-nothing fleet. **Warm-IP inventory is a hard scaling
floor:** you cannot grow outbound faster than you can warm IPs (weeks), so burst capacity is
bounded by the pre-warmed pool, not by instance count. DNS MX round-robin is sender-controlled
(you cannot force even distribution). Anycast inbound must be paired with stable routing so a
route flap does not break a live SMTP session.

## 14.3 The mobile-only user (no always-on box)

A phone cannot hold a socket open or be reachable, and platform push is **best-effort, not
guaranteed** on both APNs and FCM. Therefore a phone is a **push-woken thin client that drains
a queue it does not host** — never a durable node. When the user has no always-on box, the
queue is a **hosted, content-blind relay-mailbox** (the Chatmail model):

```
sender → relay-mailbox (E2EE ciphertext, short TTL) → wake-push (content-free, ≤4KB) via
         APNs/FCM through a notification proxy → phone wakes, opens its own authenticated
         connection, DRAINS the queue, decrypts locally, then reconciles on foreground
```

Requirements:

- **Push carries no content** — only a wake signal (the device token is itself encrypted where
  possible). Apple/Google never see plaintext. The design is **wake-and-fetch**, never
  deliver-in-push (APNs payload ≤4KB; silent push is throttled if excessive).
- **Push is a latency optimization, not delivery.** The client MUST still poll/reconcile on
  foreground; DMTAP MUST NOT treat a push as delivery confirmation. Wakes SHOULD be
  coalesced/batched to avoid iOS silent-push throttling.
- **The relay-mailbox is a buffer, not an archive** (§14.5): short TTL (~weeks), content-blind,
  delete-after-inactivity. The durable copy lands on the device once fetched.

This is the *only* architecture that works for a user with no home box, and it matches deployed
practice (Delta Chat/Chatmail, Signal, Matrix/Sygnal — all wake-and-fetch with content-free
push).

## 14.4 The always-on-box user

The Pi/NAS/VPS is the **durable authority**: it holds the mailbox, terminates client protocols
(JMAP/IMAP, §8), and participates in the mesh. The owner's phone/laptop are thin clients that
sync to it (§5.6 device cluster). Reachability (the box is usually NAT'd) comes from the relay
layer (§14.5). Brief box downtime is covered by the same relay-mailbox/gateway short-queue
buffer as §14.3; once the box returns and fetches, the durable copy lives on the box.

## 14.5 Relay & buffer scaling

**Relays (reachability).** Run **many small, independent, stateless relay nodes** — libp2p
circuit-relay v2 needs zero coordination between relays, so the fleet scales by adding cheap
nodes. Relays are a **reachability hop only**, with tight caps (go-libp2p defaults: ~128
reservations, 16 circuits/peer, 2 min / 128 KB per circuit). Discovery SHOULD use an
**operator-run static/rendezvous list**, not sole reliance on the public DHT.

> **Honest limit (MUST disclose):** the public libp2p/IPFS DHT + relay path is designed for
> *brief hole-punch assistance*, not sustained mailbox sync — undialable-peer dial timeouts and
> republish load make it unsuitable as the sole production discovery/relay path. DMTAP
> deployments SHOULD run their own relay fleet with tuned caps and rendezvous discovery, using
> the DHT (if at all) only for opportunistic discovery. Direct connectivity (IPv6, hole-punch)
> is always preferred; relays carry the residual hard-NAT minority (§4.3).

**Buffers (offline holding).** Ciphertext for an offline node is held in a **content-blind
relay-mailbox with a short TTL and delete-after-inactivity** (Chatmail model: ~20-day message
TTL, ~90-day inactive-account purge as reference defaults). This decouples availability from any
single peer's uptime while keeping per-account cost near-zero (no long-term archive). Peer
buffering (§4.3) is an alternative when no hosted buffer is desired, but its durability is tied
to the buffering peer's uptime — weak for mobile-only. The gateway's short queue (§7.4) is only
the legacy-translation hop and MUST NOT become a store.

> **Honest limit:** a relay-mailbox is a **buffer, not an archive** — a node offline past the
> TTL loses undelivered mail. Durability MUST land at the recipient's edge once fetched. Senders
> retry (§2.6) within their own deadline regardless.

**Buffer is not backup (§1.4).** Neither a relay-mailbox nor a peer buffer (§4.3, peer-buffer TTL
§16.6) is a **content backup**: each holds only *undelivered* ciphertext within its TTL, and key
recovery (§1.4) restores onto an **empty store**. Durable content continuity requires a surviving
cluster device (§5.6) or the **portable encrypted backup** of §1.4 — not the buffer.

## 14.6 Hosted multi-tenant topology (the operator)

For an operator hosting many mailboxes (Envoir Cloud, §12), the sensible horizontally-scalable
shape is **stateless routed front + per-tenant sandboxed compute + object-storage persistence +
scale-to-zero**:

- **Stateless front:** hostname-routed proxy on an **Anycast** network, TLS-terminating,
  backhauling to the tenant's cell.
- **Per-tenant isolation:** one logical app / sandbox (microVM, e.g. Firecracker) **per
  customer**, so a compromised tenant cannot read another's secrets or content; equivalently a
  namespace-per-tenant + sandboxed runtime on Kubernetes.
- **Storage:** per-account **encrypted object-storage bucket** (Envoir Cloud uses R2/Tigris,
  free egress); isolation enforced at the app/bucket boundary, not the platform sandbox alone.
- **Scale-to-zero + push-wake:** idle mailboxes cost only storage; a request/push cold-boots the
  cell (~sub-second). This fits mailboxes (idle most of the time) and the §14.3 push model.
- **Multi-region:** place each tenant's cell in its home region behind the Anycast front;
  control-plane metadata (tiny) in a shared control-plane DB (Neon), content in per-tenant
  buckets.

Because there is no long-term archive on the hot path and mailboxes are mostly idle, the
per-mailbox cost floor is near-zero (a documented ~1 GB RAM / 1 CPU serves thousands of light
mailboxes), which is what makes the operator economics in §12 work.

## 14.7 Grounding

Patterns grounded in: KumoMTA (shared-nothing MTA, egress pools), AWS SES (per-ISP managed
warmup), SendGrid/Mailgun (per-stream pools, health-check demotion), RFC 5321 §5.1 (MX
priority), Cloudflare (Anycast inbound), Delta Chat/Chatmail + Signal + Matrix/Sygnal
(wake-and-fetch push, relay-mailbox), APNs/FCM (best-effort, ≤4 KB, throttling), libp2p
circuit-relay v2 + Protocol Labs DHT findings (relay caps, discovery), Fly Machines "one app per
customer" + Kubernetes multi-tenancy (per-tenant sandbox, scale-to-zero). See §11 for the full
bibliography.
