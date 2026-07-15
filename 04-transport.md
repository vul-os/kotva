# 4. Transport: Mesh, Mixnet, Delivery

The mesh finds you and moves ciphertext to you; the mixnet hides who is talking to whom.
The node **is** the mesh — relay and mix roles are capabilities of the node binary, not
separate services.

## 4.1 Substrate: libp2p

DMTAP builds on **libp2p** rather than reinventing P2P:

- **Kademlia DHT** — peer routing and the `key → location` record store (see §4.2 for its
  security limits — it is the weakest link).
- **Circuit relay v2** — reachability for NAT'd nodes (the "relay" role).
- **AutoNAT v2 + DCUtR (hole punching)** — direct connections where possible.
- **Noise** (or TLS 1.3) — hop-by-hop channel security (in addition to end-to-end MOTE
  encryption; QUIC carries TLS 1.3 natively).
- **QUIC / TCP / WebSocket / WebRTC / WebTransport** transports.

A node dials **outbound** and holds connections, so CGNAT/dynamic-IP nodes are reachable.

**Roaming — honest note:** identity is a persistent keypair, not an IP. When a node's address
changes, roaming is carried primarily by **re-publishing the location record (§4.2) and peers
re-dialing**, not by QUIC connection migration. QUIC migration (RFC 9000) *can* preserve some
live connections, but the Rust QUIC stack (`quinn`) has no multipath and address-change
handling is imperfect; do not rely on it for seamless roaming.

## 4.2 The `key → location` record (DHT)

```
LocationRecord {
  ik:        bytes,        // identity key (DHT key = hash(ik))
  peer_id:   bytes,        // libp2p peer id (may be per-epoch / unlinkable, §6)
  addrs:     [* multiaddr],// current reachability hints (may be relay circuits, mix addrs)
  ttl:       u64,
  ts:        u64,
  sig:       bytes,        // signed by a device key
}
```

The record follows the **IPNS pattern**: a self-certifying **value record** (DHT key =
hash(ik)), signed, with a **monotonic sequence number + EOL/TTL** to defeat rollback/replay,
stored at the **K closest peers**, and **aggressively republished** (DHT record lifetimes are
short — hours — so staleness is a real failure mode). Use value records, not provider records.

### CAUTION — signing does NOT stop eclipse attacks (the DHT is the weakest link)

Signing authenticates record *content*, not *routing*. Because a libp2p PeerId is
`hash(pubkey)`, an attacker can cheaply generate IDs closest (XOR-distance) to a target key,
fill honest routing tables, and control all lookups for that key — returning nothing or an
*old, still-valid signed* record (censorship / rollback) **without forging anything**. This is
a Sybil/eclipse attack at the routing layer, and it is the single most attackable part of
DMTAP. Mitigations DMTAP REQUIRES/RECOMMENDS:

- **S/Kademlia disjoint-path lookups** (parallel, node-disjoint) and **IP-diversity caps** per
  k-bucket.
- **Aggressive republish** + accept only records with a newer sequence number (rollback
  defense).
- **The DHT is one discovery mechanism, not the root of trust.** Root of trust = the user's
  long-term keypair. Resolution order: (1) cached direct addresses, (2) **relay-reservation /
  rendezvous ("home relay") addresses**, (3) DHT as fallback. A fresh contact SHOULD have a
  non-DHT path (rendezvous/introduction) so it is not 100% dependent on a hostile public-DHT
  lookup.
- Closed/organizational deployments SHOULD use a **private DHT** (own protocol prefix) to
  shrink the Sybil surface.

## 4.3 Reachability ladder

Try in order; fall down only as needed:

1. **Direct** — IPv6, or IPv4 with port-forward/UPnP. No relay.
2. **Hole-punch** — DCUtR between two nodes (both online; always true for always-on boxes).
3. **Circuit relay** — a public-IP node relays; ciphertext-only, content-blind.

As IPv6 spreads, rungs 1–2 dominate and the relay role fades. Downtime is covered by
sender-retry (§2.6) and optional **peer buffering** (buddy nodes hold ciphertext during an
outage) — no central buffer required.

## 4.4 The mixnet (metadata privacy)

For the `private` tier, MOTEs are sent as **Sphinx-format**, constant-length, onion-wrapped
packets through a sequence of **mix nodes** (Loopix/Nym-style):

- Each mix peels one layer; **no mix sees both sender and recipient**.
- Mixes add **randomized (Poisson) delays** and reorder, defeating timing correlation.
- Nodes emit **cover traffic** (loop + drop messages) at a steady rate so an observer cannot
  tell when real messages are sent/received. Cover rate is a **tunable knob** (§6).
- **Sealed sender** (§6.2): the sender identity is inside the payload, never in the outer
  packet.
- **Size padding**: MOTEs are padded to fixed buckets so size does not leak content.

Mix nodes are the same permissionless, content-blind contributor model as relays; incentive
and Sybil-resistance are covered in §6.4 and §9.

Email's asynchrony is what makes full-strength mixing affordable: minutes of latency are
acceptable for the `private` tier.

## 4.5 Bulk / file transfer

Large blobs (§5.5) MUST NOT traverse the mixnet (impractical bandwidth/latency). Instead:

- The **control MOTE** (`file_offer`: manifest + key) travels the **private** tier.
- The **chunks** transfer **direct** (fast tier), content-encrypted, **swarmed** from any
  holder (BitTorrent-style), resumable per chunk.

Honest tradeoff: the *fact and size* of a bulk transfer between two nodes is observable
(direct), though the content is encrypted and the control message is private. See §6.

## 4.6 Privacy tiers

| Tier | Path | Latency | Metadata privacy | Default for |
|------|------|---------|------------------|-------------|
| `private` | mixnet + cover traffic | minutes | full (global passive) | mail, all control messages |
| `fast` | direct / low-hop mesh | sub-second | content only; graph observable | live chat (both online), bulk chunks |

Default is `private`. `fast` is opt-in per conversation/message. Choosing `fast` is itself
metadata; clients SHOULD keep `private` as the standing default and cover it with cover
traffic (§6).

## 4.7 Delivery state machine (sender)

```
QUEUED → SEALED (sealed sender + onion, §6) → IN_FLIGHT (mixnet/direct)
       → ACKED (done)  |  RETRY (backoff)  |  EXPIRED (drop, notify user)
```

Durability lives entirely in this sender-side queue. The middle is stateless.

## 4.8 Local, isolated, and delay-tolerant networks

DMTAP works in **remote environments with their own networks** — often *more easily than
email*, because email needs an SMTP/IMAP server + MX + DNS, whereas two DMTAP nodes on the same
network need no infrastructure at all.

- **Local discovery (mDNS).** Nodes on the same LAN discover each other via libp2p **mDNS**
  (`_p2p._udp.local`) with **zero configuration, no internet, and no central server**. A ship,
  a remote site, an air-gapped office, or a home LAN can run a fully functional local DMTAP
  mesh with nothing but the nodes. Private networks use a fingerprinted service name so they
  do not cross-discover.
- **Scope of the "easier than email" claim.** True specifically for the **local / same-network
  / isolated** case. Across the open internet, DMTAP still depends on the DHT/relays (§4.2–4.3),
  which have their own fragility.
- **Delay-tolerant store-and-forward is a DMTAP layer, not a libp2p feature.** libp2p is
  connection-oriented and provides no bundle/epidemic DTN routing. DMTAP REQUIRES its own
  store-and-forward: the sender-side retry queue (§4.7), **peer buffering**, and
  **sync-on-reconnect** of the device-cluster CRDT (§5.6). This is what lets an intermittently
  connected site queue locally and reconcile with the wider mesh when connectivity returns.
- **Radio transports (Bluetooth / Wi-Fi Direct / LoRa) are out of scope for v0.** libp2p ships
  no such transport (Briar's offline radio transports are its own Bramble code, not libp2p).
  Supporting them would mean writing a custom libp2p `Transport` — flagged as future work, not
  a v0 claim.
