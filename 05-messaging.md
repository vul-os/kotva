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
- **Failover / liveness & deterministic succession:** if the committer is unreachable, members
  hold pending Proposals and initiate a **takeover that does not depend on the incumbent's
  cooperation**. The successor is **deterministic**: among live, non-faulted members the one with
  the **lowest member signing key** (canonical byte order) is the designated next committer
  (keys are unique; earliest join epoch is the fallback tie-break). The successor proposes a
  **takeover Commit** referencing the last agreed log head; it takes effect **only when it carries
  a threshold of member signatures** (a roster quorum, §16), so a partitioned minority cannot
  install a rival committer and there is no split-brain over *who* takes over. (The heavy
  committer-plus-`> n/2`-takeover machinery in this bullet applies only to **groups of n ≥ 3**; a
  **2-member group (a 1:1) uses the lighter symmetric-ordering path of §5.1.1 instead**, because
  the `> n/2` quorum is unsatisfiable when one of the two peers is dead.) The hash-chained
  log makes a **fork detectable**: two Commits at the same log position with the same predecessor
  is proof of committer misbehavior; members MUST halt and alert (analogous to KT equivocation,
  §3.5) and recover per the fork-recovery path below.
- **Censorship is misbehavior — including *selective* censorship:** a committer can *stall* but
  cannot *forge* (every handshake is member-signed). Crucially, a committer that stays live yet
  **withholds ordering of one specific pending, member-signed proposal** (e.g. never orders a
  member's own removal) past the **committer-liveness timeout (§16)** is treated **identically to
  a dead committer** and triggers the deterministic takeover above — total silence is *not*
  required to justify replacement. This defeats the selective-censor-and-block-your-own-
  replacement attack, since the successor rule and member-signature threshold need no consent
  from the incumbent.

**Fork recovery (out of HALT).** A detected fork halts the group (above); recovery is
**out-of-band**. Members compare hash-chained log heads and identify the **last common epoch**
both forks share; an `admin`/`owner` (per §5.8.2 authority) proposes a recovery Commit on top of
that last common epoch, and — **for consistency with committer takeover (§16 roster quorum) and
to deny a single admin unilateral fork-selection power** — the recovery Commit is canonical only
when it carries the same **> n/2 member-signature quorum** as a takeover. This re-establishes a
single canonical head that a strict majority endorsed, not one admin's choice. Members that
applied the losing fork **roll back to the last common epoch** and re-apply from the recovery
Commit; application messages committed only on the abandoned fork are re-submitted by their
senders (sender retry, §2.6). This manual reconciliation is the v0 stopgap; **Decentralized MLS
(`draft-kohbrok-mls-dmls`) is the eventual leaderless fix** that removes the single-orderer fork
surface entirely.

**Metadata exposure (honest, §6.6 item 7).** The committer necessarily sees **all handshake
traffic for its group** — an explicit exception to the "no single node sees both ends" framing.
This is bounded by: the committer sees only *membership-change* metadata (not application
message content or the social graph outside the group), rotating the role spreads exposure, and
application traffic still uses the mixnet. Groups needing stronger metadata protection SHOULD
rotate committer frequently.

Track `draft-kohbrok-mls-dmls` (Decentralized MLS) for a fully leaderless replacement; until it
lands, the committer model above is v0.

### 5.1.1 Lighter ordering for n = 2 (1:1) — symmetric co-committers (normative)

A **1:1 is a 2-member MLS group** (§5.1), and the committer-takeover machinery above **cannot
work for it**: takeover needs a `> n/2` (both-of-two) member-signature quorum, which a live peer
can never assemble when its counterpart is offline or censoring — so a single committer would be
an unremovable single point of failure over the pair's own ordering. DMTAP therefore specifies a
distinct, lighter ordering path for **n = 2** and reserves the committer/quorum machinery for
**n ≥ 3**:

- **Both members are symmetric co-committers.** There is **no single committer** and no takeover:
  each of the two members MAY serialize its own handshake messages (Proposal/Commit/Update). The
  ordered, reliable per-group channel MLS requires (§5.1) is realized by **pairwise handshake
  sequence numbers** rather than one node's log: each member maintains a strictly increasing
  `hs_seq` for the handshakes it originates, and each handshake is chained by `prev` to the last
  handshake that member has applied.
- **Deterministic tie-break on concurrent Commits.** Because either peer may Commit, the two can
  propose **concurrent** Commits against the same epoch. This is resolved without a quorum by a
  fixed rule: the Commit whose **originator has the lower member signing key** (canonical byte
  order — the same total order §5.1 uses for successor selection) is **canonical**; the other
  peer's concurrent Commit is **superseded and re-proposed** on top of the winner (its author
  re-applies it at its next `hs_seq`). Ties on the key are impossible (keys are unique), so the
  order is total and both peers converge on the identical epoch chain with no vote.
- **No forgery, same fork-evidence.** Each handshake is still **member-signed** (MLS, §5.1), so
  neither peer can forge the other's membership actions; a peer that presents two different
  handshakes at the same `(originator_ik, hs_seq)` produces **self-signed fork evidence**, and the
  other member MUST `HALT_ALERT` exactly as for committer equivocation (§5.1, `0x0404`). The
  hash-chain `prev` still detects reordering and gaps.
- **Censorship degrades to unilateral action, not deadlock.** A 2-party peer cannot *stall* the
  other's ordering (there is no committer to withhold service): each member can order its own
  Commits — including the other's **removal** — so the "censoring committer in a 2-party group"
  dead-end is removed (it is no longer resolved by leaving/recreating the group). A genuine
  divergence (a detected fork) recovers via the §5.1 out-of-band path with the trivial 2-party
  quorum (both members, or re-pair).
- **Growth to n ≥ 3.** The first Commit that grows the group beyond two members installs the
  standard **committer** (§5.1 — the adding member is the initial committer) and the group leaves
  the symmetric path; shrinking back to two members reverts to symmetric co-committers. The
  regime is a pure function of roster size, so both peers always agree on which is in force.

## 5.2 Forward secrecy & (non-)deniability

- **Forward secrecy** comes from MLS epoch advancement. Healing (post-compromise security)
  is **per-epoch/per-Commit**, which is coarser than the Signal Double Ratchet's
  per-message healing — using MLS for 1:1 (§5.1) is a deliberate *simplicity-vs-optimality*
  trade, not a strict improvement over Double Ratchet.
- **Deniability — the default path is non-repudiable; an optional deniable 1:1 mode is
  specified.** MLS is **signature-based and therefore non-repudiable**: every LeafNode and every
  FramedContent (handshake *and* application messages) is signed with the member's signature key,
  and RFC 9750 states plainly that *"MLS does not make any claims with regard to deniability."*
  DMTAP's **default MLS path is therefore non-repudiable**, and DMTAP MUST NOT substitute
  shared-secret MACs for MLS's mandatory signatures *inside MLS* (that would break RFC 9420
  conformance). Deniability is **not** achieved by weakening MLS. Instead, DMTAP specifies a
  **normative, OPTIONAL, capability-negotiated deniable 1:1 mode in §5.2.1** that runs a *separate*
  Signal-style channel (X3DH/PQXDH + Double Ratchet with shared-key-MAC authentication) **beside**
  the 2-member MLS group, giving cryptographic **participation and message repudiation** for
  1:1 conversations. Groups (`n ≥ 3`) stay MLS and remain non-repudiable. The residual is disclosed
  honestly in §5.2.1: deniability is *cryptographic repudiation of the transcript*, not protection
  against a compromised endpoint that logs plaintext.

### 5.2.1 Optional deniable 1:1 mode (normative)

MLS-everywhere buys simplicity and post-compromise security at the cost of **non-repudiation** —
inherent to any signature-authenticated protocol. For users who need **repudiation** (a
whistleblower, a source, a lawyer's off-record channel), DMTAP specifies an OPTIONAL **deniable
1:1 mode**. It does **not** modify MLS; it is a **separate pairwise channel** selected per
conversation, reusing the proven Signal design rather than inventing new cryptography.

**Why not "MLS with MACs."** Deniability cannot be retrofitted onto MLS by dropping its
signatures — that breaks RFC 9420 conformance and the group ratchet's security proof. The clean
construction is a **distinct 1:1 protocol whose authentication is a shared-key MAC**, so *either*
party could have produced any transcript ⇒ neither can prove the other authored it. That protocol
already exists and is deployed at scale: **Signal's X3DH/PQXDH + Double Ratchet**.

#### (a) Handshake — X3DH (classical) / PQXDH (PQ), by reference

Session setup reuses **X3DH** (Marlinspike & Perrin, 2016) for `suite = 0x01` and **PQXDH**
(Kret & Schmidt, 2023) for `suite = 0x02` (ML-KEM-768, matching the suite's X-Wing KEM, §16.7)
— **not reimplemented here**. DMTAP supplies only the binding to its own identity and prekeys:

- **Long-term identity DH key from `IK`.** X3DH mixes the parties' *long-term identity DH keys*.
  DMTAP's `IK` is an **Ed25519 signing** key, so both parties derive the corresponding
  X25519 identity DH key from `IK` via **XEdDSA** (Perrin, 2016) — **no new long-term key is
  provisioned**, and the `IK ↔ name` binding (KT, §3.5) still authenticates the counterpart.
- **Published deniable prekeys.** An identity that offers the mode publishes a
  **`DeniablePrekeyBundle`** (§18.4.8) located via `Identity.deniable_prekeys` (§1.3, §18.4.1):
  a **signed prekey** `spk` (X25519) with its signature, a set of **one-time prekeys** `opks`,
  and — under `suite = 0x02` — a signed **last-resort ML-KEM key** plus one-time KEM keys
  (PQXDH). The bundle is replenished by the owner's node exactly like KeyPackages (§5.3);
  exhaustion falls back to the signed prekey / last-resort key, rate-limited (§16.9).
- **What is signed vs MAC'd (the crux).** The **only** signature in the whole mode is the
  **signed-prekey signature** (`spk_sig`), which attests that the *prekey was published* — it
  does **not** sign any message, transcript, or the fact that a conversation occurred. This is
  exactly the standard X3DH signed prekey and is what preserves deniability: the long-term
  signature covers a public prekey, never content. **Every message is authenticated by a
  shared-key MAC** (the Double Ratchet AEAD tag under a per-message key derived from the shared
  secret), which **both** parties can compute ⇒ **participation + message repudiation**.

#### (b) Session — Double Ratchet, by reference

After the handshake yields a root secret, messages travel over the **Double Ratchet**
(Perrin & Marlinspike, 2016): a DH ratchet + symmetric-key chains giving **per-message forward
secrecy and per-message post-compromise security** — *finer* healing than MLS's per-epoch Commit
(§5.2), a bonus of the mode, not a regression. Message keys authenticate via AEAD (a shared-key
MAC), never a signature.

#### (c) Wire framing

- A deniable MOTE uses **`kind = 0x0b` (`deniable`)** (§21.16). Its `Envelope.ciphertext`
  (§18.3.1) carries a **`DeniableFrame`** (§18.3.9) — a tagged choice of **`DeniableInit`** (the
  X3DH/PQXDH first message, embedding the first ratchet message) or **`DeniableMessage`** (a
  subsequent Double-Ratchet message). Neither carries a `sig-val`.
- The inner plaintext, after ratchet decryption, is a **`DeniablePayload`** (§18.3.10) — the
  structural twin of `Payload` (§2.4) **with the signature field removed** and the real content
  `kind` carried inside. A `DeniablePayload` that carries a signature MUST be rejected
  (`ERR_DENIABLE_SIGNATURE_PRESENT`, `0x040F`); the missing signature is the point.
- The **envelope layer stays deniable-preserving**: `Envelope.sender_sig` (§18.3.1) is a
  **fresh, identity-free, per-message ephemeral** signature over routing metadata only (§18.9.1).
  It binds no long-term key and asserts no identity, so it does **not** make the transcript
  attributable — it only gates abuse (§9). Sealed-sender still applies (the sender's `IK` lives
  inside the ratchet-encrypted `DeniablePayload`, visible only to the recipient).

#### (d) Negotiation, scope, and composition (normative)

- **Capability-negotiated.** The mode is used only when **both** peers advertise the
  **`deniable-1:1`** capability token (§10.2, §21.22) and the initiating **user selects it** —
  it is a user/client-selectable mode, never a silent default. A peer that has not advertised
  it is handled per `ERR_DENIABLE_MODE_UNAVAILABLE` (`0x040E`): the client MUST surface the
  choice (fall back to the non-deniable MLS 1:1, or do not send), never silently downgrade the
  user's *expectation* of deniability.
- **1:1 only; pairwise per device.** Deniability is inherently pairwise. The mode replaces the
  **2-member MLS group** (§5.1.1) as the transport for that 1:1's *content*; it does not apply to
  groups of `n ≥ 3`, which stay MLS and remain non-repudiable. Across a personal cluster (§5.6)
  the mode runs as **one Double-Ratchet session per device-pair** (Signal-Sesame style),
  fanning out to each of the recipient's devices — trading MLS's efficient single-tree cluster
  sync for pairwise deniability. This cost is disclosed; clients SHOULD show that deniable
  threads sync per-device rather than through the shared MLS tree.
- **Identity verification unchanged.** The counterpart's `IK` is still bound to its name via
  KT / OOB safety numbers (§3.4–§3.5); deniability repudiates *authorship of messages*, not the
  *key agreement*. Downgrade defense (§1.3 suite ratchet) and cold-sender anti-abuse (§9) apply
  as for any MOTE.

#### (e) The residual (honest, after the mechanism)

The mode delivers **cryptographic repudiation of the transcript**: given only ciphertext and
keys, neither party (nor a third party) can produce a cryptographic proof that the *other* party
authored a given message, because the authenticator is a shared-key MAC either could have forged.
Two residuals survive and are stated plainly:

1. **Deniability is not endpoint protection.** A compromised or coerced endpoint that **logs the
   plaintext as it is displayed** trivially "proves" content — no repudiation protocol prevents a
   party from keeping and disclosing its own plaintext (this is the same floor as §6.6 item 3).
   Cryptographic deniability defeats a *cryptographic* transcript proof, not a screenshot.
2. **X3DH deniability has known theoretical bounds.** Formal analyses (Vatandas, Gennaro,
   Ithurburn & Krawczyk, ACNS 2020; Unger & Goldberg) show X3DH gives strong **offline**
   deniability but that **online/interactive** deniability against a judge colluding in real time
   with a participant is weaker; the signed prekey and prekey-signing key are long-lived. DMTAP
   inherits these bounds — it does not claim to exceed Signal's proven guarantees.

This is honest deniability: *repudiation of the cryptographic transcript*, disclosed for what it
is, not a promise that a determined endpoint compromise can be undone.

## 5.3 Async session initiation (MLS-native)

To message an identity whose devices are all offline, DMTAP uses **MLS's own async-join
mechanisms** rather than bolting on a separate PQXDH/X3DH handshake (which would add a second
protocol and a second security proof for no benefit when everything is already MLS):

1. Each device pre-publishes signed **KeyPackages** (located via `Identity.keypkgs`, §1.3) to
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
  suite:     u8,
  // NO key field. The content key is NOT part of the manifest — see below.
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
- **Key is never in the manifest (confidentiality).** The manifest is itself a
  content-addressed blob a fetcher pulls from the swarm to learn the chunk list, so any node
  serving it would also learn an embedded key and could decrypt the whole file. The file content
  key therefore travels **only inside the sealed MOTE** (`Attachment.key`, §2.5/§18.3.7), never
  in the manifest and never alongside the chunks. Chunks are served **blind** — a holder can
  relay encrypted chunks and the manifest without ever being able to read the file. A manifest
  received with a key embedded MUST be rejected as a leak (`ERR_MANIFEST_KEY_PRESENT`, §21), not
  used.

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

## 5.8 Groups as addressable identities

A group is an identity that has members. It has its **own keypair** and therefore its **own
name** on the full naming ladder (§3.9): a group key-name, an `@handle` (e.g. `@team`), or a
domain address (`team@company.com`). Sending to a group address delivers to all current members;
membership is the group's MLS roster (§5.1). This single mechanism unifies mailing lists, chat
channels, shared folders, and team inboxes — "a group with an address."

### 5.8.1 Two posting models

| Model | Behavior | Membership visibility |
|-------|----------|-----------------------|
| **Broadcast / list** | post to the address → every member receives a copy (distribution list, announce, team inbox) | typically hidden (subscribers don't see each other) |
| **Collaborative / channel** | shared, ordered conversation + shared state (chat channel, shared folder) | typically member-visible |

Both are MLS groups; they differ only in **posting policy** and **membership-visibility policy**,
which are fields of the group state. A group MAY switch models by policy change (a Commit).

### 5.8.2 Roles & management

Group state carries **roles**: `owner` (≥1), `admin`, `member`, and optional `poster` (may send)
vs `reader`. Management operations are **MLS Commits** ordered by the group committer (§5.1),
authorized by role:

- **Add / remove member** — Add uses the invitee's KeyPackage (§5.3) + Welcome; Remove triggers
  file-key rotation for shared folders (§6.7). Requires `admin`.
- **Role change / transfer ownership** — requires `admin`/`owner`.
- **Policy change** (posting model, membership visibility, join policy) — requires `admin`.
- **Join policy:** `closed` (invite only), `request` (request → admin approval; a request with no
  admin response auto-expires after the join-request expiry, 30 days, §16.8), `open` (anyone
  with the address may join, rate-limited + anti-abuse §9), or `vouch` (a member introduces).

All membership/role/policy changes are signed and appear in the group's hash-chained handshake
log (§5.1), so a member can audit "who added/removed whom" — the group analog of identity
key-transparency (§3.5). A malicious/coerced committer can stall but not forge (every change is
member-signed); **selective non-ordering of a pending signed management proposal past the
committer-liveness timeout is misbehavior that triggers the deterministic committer takeover
(§5.1)**, and forks are detectable and recoverable (§5.1).

### 5.8.3 Membership privacy (subscriber-list)

Broadcast lists MUST support **hidden membership**: members receive via per-member sealed
delivery (§6) and do not learn the other members' keys. MLS's tree exposes members to *each
other* by default, so hidden-membership lists use a **relay/committer fan-out** where the
list identity re-seals to each member individually rather than a shared member-visible tree —
at the cost of the shared-group efficiency. Channels (member-visible) use the normal MLS tree.
This resolves the earlier mailing-list-privacy gap: choose the model per group, and disclose it.

### 5.8.4 Delivery & scale

- Small groups: standard MLS group message, fan-out to members over the mesh/mixnet.
- Large lists: the committer's ordered log is the source of truth; delivery is **per-member**
  (sealed to each), pull-or-push, so a 10k-member list is 10k individual sealed deliveries, not a
  10k-member MLS tree. This trades cryptographic group-sharing for scalability and hidden
  membership — the right trade for announce lists.
- **Anti-abuse through fan-out (normative, §9.9).** Fan-out is an **amplification vector**: one
  post becomes N deliveries, so the recipient's per-sender policy (§9) MUST apply to the
  **original poster**, not to the list identity. The poster's **ARC token / postage / PoW** (§9)
  MUST be **carried through the fan-out** and presented on each per-member delivery, and each
  recipient evaluates it against the *poster* — no accountability laundering behind the group
  address. Fan-out MUST be **rate-limited per poster**, amplification for `open` (anyone-can-post)
  lists MUST be **capped**, and posting to a **large list** MUST require **postage or PoW**
  commensurate with the fan-out size.

### 5.8.5 Legacy interop

A group MAY have a **legacy address** (`team@company.com`) served by the gateway (§7): inbound
legacy mail to the address is fanned out to members as MOTEs; outbound from the list to legacy
subscribers goes via the gateway. So a DMTAP group is reachable as an ordinary mailing-list
address from the old world, while native members get the full encrypted/group experience.
Legacy fan-out is bound by the **same per-poster anti-abuse carry-through and fan-out rate-limits**
(§5.8.4, §9.9); the gateway attributes the origin (§9.6), so list fan-out cannot launder a
spammer's accountability.

### 5.8.6 Group key custody, recovery & threshold (normative)

A group is an addressable identity (§5.8) with its **own keypair**, so it needs the same custody
discipline as a personal identity (§1) — otherwise a single admin or committer holding the group
key can unilaterally hijack `team@company.com`.

- **Threshold-held signing key.** The group's identity signing key MUST be **threshold-held by
  the group's `owner`/`admin` set** (FROST-style, reusing the §1.4 recovery machinery), so no
  single admin — and no committer — can sign as the group alone. Group-authoritative acts (below)
  require a threshold of admins, not one.
- **Group `RecoveryPolicy` + `rotate_threshold`.** A group has its own `RecoveryPolicy` (§1.4);
  changes to the group `Identity`, its recovery methods, or its key MUST satisfy the group's
  `rotate_threshold` (the weakening-quorum + veto-window rules of §1.4 apply), so a compromised
  admin device cannot rewrite group recovery or evict co-owners.
- **KT-logged group key events.** Group `Identity` / `KeyRotation` / `RecoveryPolicy` events MUST
  appear in key transparency (§3.5) exactly like personal ones, so members and the wider network
  detect an unauthorized group-key change. The committer (§5.1) orders *handshakes*; it is
  **not** authorized to change the group's identity key — that is a threshold act above the
  committer role.

### 5.8.7 Organization groups & distribution lists (normative)

An org's `team@abc.com`, `all@abc.com`, `support@abc.com` are ordinary **group identities (§5.8)**
whose group *name* is a domain address under the org's domain authority (§3.10.1). Nothing new is
needed; the org-administration layer only adds **who may provision and administer them**:

- **Provisioning.** A `group-admin` or `domain-admin` capability (§13.5.1, §3.10.4) authorizes
  creating a group identity and binding its `team@abc.com` name under the domain (§3.2 record +
  directory entry, §3.10.3) — exactly as a member is provisioned (§3.10.2). A group is "a member
  that has members" (§5.8).
- **Group custody stays threshold-held.** Because the group is domain-addressed and org-critical,
  its signing key MUST be threshold-held by the group's `owner`/`admin` set (§5.8.6), so neither a
  single group-admin nor the committer (§5.1) can unilaterally hijack `team@abc.com`. Org
  provisioning does not override §5.8.6: the domain authority grants the *name*, the group's own
  threshold governs the *key*.
- **Membership maps to the directory.** An org group's roster (§5.8.2) is populated from
  `DomainDirectory` entries (§3.10.3): `all@abc.com` is the group whose members are the directory's
  members, and admin tooling keeps them in sync (adding a person to the org SHOULD offer to add
  them to standing groups). The mapping is convenience only — the **roster remains the group's own
  MLS roster** (the authority), and every add/remove is member-signed and KT-audited (§5.8.2), so
  the org cannot silently inject a listener into a team inbox: a directory-driven add is still a
  signed group Commit, visible in the group's handshake log.
- **Posting model & privacy per group.** `all@abc.com` (announce) SHOULD be a broadcast list with
  hidden membership (§5.8.1, §5.8.3); `team@abc.com` (collaboration) a member-visible channel — the
  org picks per group and discloses it (§5.8.1). Per-poster anti-abuse carry-through and fan-out
  rate-limits (§5.8.4, §9.9) apply unchanged.
- **Offboarding.** Removing a person from the org (§3.10.5) SHOULD remove them from org groups via
  the normal §5.8.2 Remove (re-keying shared state, §6.7), so a departed member loses group access
  even though their **sovereign key survives** (§3.10.2a) — the group evicts the *member*, it does
  not touch their *identity*.
