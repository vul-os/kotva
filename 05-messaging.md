# 5. Messaging & Files (the unified substrate)

Mail, chat, and files are modes over one substrate: **MLS** (RFC 9420) for all session and
group security, MLS **KeyPackages** for async initiation, and **content-addressed chunked
blobs** for files. The same group primitive serves 1:1, group chat, email mailing-lists,
multi-device, and shared file folders.

## 5.1 Why MLS everywhere

DMTAP standardizes on **MLS as the unifying crypto primitive**:

- **1:1** = a 2-member MLS group.
- **Group chat / mailing-list** = an MLS group.
- **Multi-device** = each of the owner's devices is a member (MLS handles this cleanly where
  pairwise ratchets get messy).
- **Shared file folder** = an MLS group over a set of manifests (§5.5).

MLS is **transport- and identity-agnostic** by design (RFC 9420, 2023; architecture in
RFC 9750). Its abstract roles map onto DMTAP:

- **Delivery Service (DS)** → the DMTAP mesh/mixnet (§4).
- **Authentication Service (AS)** → DMTAP identity + KT (§1, §3). KT realizes the AS while
  *reducing* the trust placed in it — a good fit for a decentralized design.

MLS provides group forward secrecy and post-compromise security via the TreeKEM ratchet
tree, with O(log n) member changes. v0 uses ciphersuites `0x0001`
(`MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519`) and `0x0003`
(`…CHACHA20POLY1305…`); PQ migration uses MLS's own PQ ciphersuites
(`draft-ietf-mls-pq-ciphersuites`, ML-KEM) and the MLS combiner
(`draft-ietf-mls-combiner`), **not** a bolted-on external handshake.

### CAUTION — epoch ordering is the hard part on a mesh

MLS trusts the DS for exactly one thing: **a total order on epochs** — Commits must be
applied in an agreed order per group. A leaderless mesh has no natural serialization point,
so **this ordering/consensus of Commits is the real difficulty of "MLS over P2P," not the
crypto.** DMTAP REQUIRES that MLS **handshake** messages (Proposal / Commit / Welcome) travel
over an **ordered, reliable channel** per group, while ordinary **application** messages
(mail/chat/files) MAY travel over the reordering mixnet. Sending handshakes over the mixnet
would stall or fork group state.

**v0 ordered-channel: the group committer (normative).** Each group has a **committer** — the
node that serializes handshake messages into an append-only, hash-chained per-group log:

- **Selection:** the group creator is the initial committer. Committer identity is a signed
  field of the group state; every member knows it.
- **Rotation:** any member MAY be promoted to committer by a Commit (so committer role is not
  tied to one device's uptime). Committer SHOULD rotate on the current committer going offline
  past a timeout, or on member vote, via a Commit that all members apply.
- **Failover / liveness:** if the committer is unreachable, members hold pending Proposals and
  either wait for it to return or elect a new committer (a Commit referencing the last agreed
  log head). The hash-chained log makes a **fork detectable**: two Commits at the same log
  position with the same predecessor is proof of committer misbehavior, and members MUST halt
  and alert (analogous to KT equivocation, §3.5).
- **Censorship:** a committer can *stall* (withhold ordering) but cannot *forge* — every
  handshake is member-signed. A stalling committer is rotated out.

**Metadata exposure (honest, §6.6 item 7).** The committer necessarily sees **all handshake
traffic for its group** — an explicit exception to the "no single node sees both ends" framing.
This is bounded by: the committer sees only *membership-change* metadata (not application
message content or the social graph outside the group), rotating the role spreads exposure, and
application traffic still uses the mixnet. Groups needing stronger metadata protection SHOULD
rotate committer frequently.

Track `draft-kohbrok-mls-dmls` (Decentralized MLS) for a fully leaderless replacement; until it
lands, the committer model above is v0.

## 5.2 Forward secrecy & (non-)deniability

- **Forward secrecy** comes from MLS epoch advancement. Healing (post-compromise security)
  is **per-epoch/per-Commit**, which is coarser than the Signal Double Ratchet's
  per-message healing — using MLS for 1:1 (§5.1) is a deliberate *simplicity-vs-optimality*
  trade, not a strict improvement over Double Ratchet.
- **Deniability — honest correction.** MLS is **signature-based and therefore
  non-repudiable**: every LeafNode and every FramedContent (handshake *and* application
  messages) is signed with the member's signature key. RFC 9750 states plainly that *"MLS
  does not make any claims with regard to deniability."* DMTAP therefore **does NOT claim
  message deniability** at the MLS layer, and MUST NOT substitute shared-secret MACs for
  MLS's mandatory signatures (that would break RFC 9420 conformance). Deniability is an
  **explicit open design item**: if required, it must be engineered at another layer (e.g.
  exchanging the signature keys themselves over a deniable channel, per RFC 9750's note),
  or scoped to a pairwise/OTR-style sub-channel. Do not represent DMTAP messages as deniable
  in v0.

## 5.3 Async session initiation (MLS-native)

To message an identity whose devices are all offline, DMTAP uses **MLS's own async-join
mechanisms** rather than bolting on a separate PQXDH/X3DH handshake (which would add a second
protocol and a second security proof for no benefit when everything is already MLS):

1. Each device pre-publishes signed **KeyPackages** (located via `Identity.prekeys`, §1.3) to
   the mesh. A KeyPackage is MLS's prekey: identity/signature key, HPKE init key, and (for PQ)
   an ML-KEM init key via the MLS PQ ciphersuite.
2. The initiator **Adds** the offline member (a Commit) and sends a **Welcome** so the new
   member bootstraps group state on return; or the joiner uses an **External Commit** against
   the group's `GroupInfo` to self-join.
3. Post-quantum protection comes from the **MLS PQ ciphersuite / combiner** (§5.1), keeping a
   single key schedule.

KeyPackages are replenished by the owner's node; last-resort KeyPackages avoid exhaustion.
(PQXDH-for-init would only be relevant if the 1:1 layer were Signal Double Ratchet rather than
MLS; DMTAP chooses MLS everywhere, so it is not used.)

## 5.4 Message kinds in context

- `mail` / `chat` — the same MOTE; differ by default tier (§4.6) and client rendering.
- `reaction` / `edit` / `redact` — reference a prior MOTE by `id` via `refs`.
- `group_event` — MLS handshake messages (add/remove/update/commit).
- `receipt` / `presence` — opt-in, ephemeral, off by default (metadata-sensitive, §6).

## 5.5 Files: content-addressed, chunked, any size

A file is a **BLAKE3 Merkle-DAG manifest** over fixed-size encrypted chunks:

```
Manifest {
  id:        bytes,        // BLAKE3 root over chunk hashes (the content address)
  size:      u64,
  chunk_sz:  u32,          // e.g. 1 MiB
  chunks:    [+ bytes],    // ordered chunk hashes
  key:       bytes,        // file content key (delivered inside the MOTE, §2.5)
  suite:     u8,
}
```

Properties:

- **Any size** — no protocol cap; the file is bounded only by the owner's storage.
- **Resumable / parallel / swarmed** — fetch chunks from any holder, restart per chunk,
  BitTorrent-style; a popular file becomes *more* available as more nodes hold it.
- **Deduplicated** — identical chunks (and identical files) stored once by content address.
- **Streamable** — consume in manifest order before full download.
- **Integrity** — every chunk self-verifies against its hash; the manifest self-verifies
  against `id`.

**Availability tiers** (files reintroduce storage economics):

| Tier | Mechanism | Cost |
|------|-----------|------|
| Best-effort | origin node online + swarm from holders | $0 |
| Durable | encrypted peer-cache / replica | peer reciprocity |
| Always-available | paid replica / managed host | real storage cost |

The always-on box gives best-effort/durable for free; "Dropbox-like always available" for
large files is where a paid replica re-enters. Transfer uses the fast/direct tier (§4.5).

## 5.6 Multi-device (the personal cluster)

An owner's devices form an MLS group (`caps` per DeviceCert, §1.2). The mailbox, read/flag
state, labels, and file index are replicated across devices as an **encrypted CRDT** synced
over the mesh, converging under out-of-order delivery. Any device can send/receive; the
always-on box is the anchor that guarantees receipt while other devices sleep.

## 5.7 Three products, shared components

| Component | Mail | Chat | Files |
|-----------|:----:|:----:|:-----:|
| Identity / keys (§1) | ● | ● | ● |
| MOTE object (§2) | ● | ● | ● |
| Mesh + mixnet (§4) | ● | ● | ● (control) |
| **MLS group (§5.1)** | ● (lists) | ● (channels) | ● (folders) |
| MLS KeyPackages (§5.3) | ● | ● | ● |
| Content-addressed blobs (§5.5) | ● (attach) | ● (attach) | ● |
| Fast tier (§4.6) | — | ● (live) | ● (bulk) |
| Device-cluster CRDT (§5.6) | ● | ● | ● |

The one genuinely new component — the **MLS group** — unlocks all three. Real-time
voice/video is **out of scope** (separate WebRTC/SFU architecture).
