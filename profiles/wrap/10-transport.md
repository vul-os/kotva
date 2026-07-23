# 11. Transport

## 11.1. WRAP defines no transport of its own

Because WRAP objects **are** substrate Sync ops and Feed entries (§7.2), they
move on the substrate's transports, unchanged. WRAP specifies no `/wrap/…`
endpoints, no WRAP-specific sync round, and no WRAP-specific request auth. An
earlier draft defined all three; every one of them re-derived a wire the
substrate already standardizes, which is the reinvention rule 2 forbids.

The two surfaces a WRAP deployment uses are the substrate's:

- **Coordination state** — `Offer`, `Bid`, `Assignment`, `Progress` — reconciles
  over the Sync wire ([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §5.2):
  `GET /sync/vector`, `POST /sync/pull`, `POST /sync/ops`, scoped to the work
  order's namespace `wrap:‖id` (§7.2). Sparse sync (SYNC §7) means a node
  exchanges only the jobs it is party to.
- **Attestation feeds** — a subject's reputation history — publish and fetch over
  the Feeds HTTP surface ([`FEEDS.md`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md) §5) at
  `/.well-known/dmtap-pub/*`.

Both are servable over **plain HTTPS with no mesh present** — the substrate's HTTP
test (`README.md` §4.2) — and a fetcher verifies every object identically
regardless of which transport delivered it, because each object is
self-authenticating (§5.1). Where a mesh exists, the same objects ride the
substrate's roles (announce/resolve, circuit relay, mailbox — ROLES §2–§5) for
NAT traversal and offline holding; the mesh is one binding, never *the* binding.

## 11.2. Wire authentication is the substrate's

Transport authentication answers a different question from object signing — "who
is talking to me right now", not "who wrote this object" — and WRAP uses the
substrate's answer for it ([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §5.4), not a
bespoke header scheme. Transport auth is never required for *correctness* of the
data: every object is independently signed and verified (§5), so a compromised
transport cannot forge state. It exists only to control who may consume resources
and to make relaying accountable — and the substrate already defines it.

## 11.3. Offline and out-of-band transports

Because objects are self-authenticating (§5.1), any medium that moves bytes is a
valid transport, including ones with no server at all. The substrate's
content-addressed objects and signed ops travel intact on a synced drive, a NAS,
or a USB stick carried between sites: each node writes only its own ops, no two
nodes ever write the same object id, so a shared folder becomes a conflict-free
transport. This is not a curiosity — it is the transport that works during a
power cut, and for many field deployments it is the only one that works reliably.
A binding is conformant if it delivers intact signed objects and preserves the
§5.4 verification rules; it need not preserve order, deliver exactly once, or
guarantee delivery, because the substrate merge (§7) tolerates reordering,
duplication, and loss by construction.
