# 6. Privacy & Threat Model

DMTAP protects **content, authenticity, and metadata**. This section states the threat model
honestly, defines the guarantees, and marks the boundaries we do *not* claim.

## 6.1 Adversary model

| Adversary | Capability | DMTAP posture |
|-----------|-----------|--------------|
| Network eavesdropper (local) | reads links near a node | defeated (encryption + onion) |
| Malicious relay / mix node | sees ciphertext it forwards | defeated (content-blind, no single hop links ends) |
| Curious directory / DNS / KT log | sees lookups | mitigated (mixnet-routed lookups §3.7); binding tamper-evident (KT §3.5) |
| **Global passive adversary** | observes all links, all timing | **primary target**: social graph + timing hidden by mixnet + cover traffic |
| Global *active* adversary (inject/drop/delay at will, unlimited resources) | shapes traffic to correlate | **not fully defeated** — see §6.6 |
| Compromised endpoint (node seized/keylogged) | reads that node's plaintext | **hardened, then bounded** (§6.6 item 3): hardware-backed non-exportable keys (§1.2a), device-unlock-gated at-rest encryption (§6.7), per-device sealing, and fast revocation heal all cases **except** a device *actively compromised while unlocked and in use* |

DMTAP's headline guarantee is **strong metadata privacy against a global *passive* adversary.**

## 6.2 What is protected, and how

- **Content** — MLS/HPKE end-to-end encryption (§2, §5). Only recipients decrypt.
- **Authenticity** — signed payloads; identity bound to name via KT (§1, §3).
- **Sender identity** — **sealed sender**: the sender's identity and authenticating signature
  live *inside* the encrypted payload; intermediaries see only ciphertext to an opaque
  destination (§2.2). Honest scope: sealed sender hides the sender from *intermediaries*, but
  **does not hide the sender's IP** (the mixnet does that) and is a metadata-*reduction*, not
  elimination — receipt/timing side channels can statistically erode it (Martiny et al.,
  NDSS 2021), which is why cover traffic (§6.3) and mixing are load-bearing, not optional.
- **Recipient identity** — the mixnet delivers by **push to an always-on node** whose network
  identity is decoupled from the human identity; there is **no store-and-poll step to hide**,
  which dissolves the private-retrieval (PIR) problem by architecture (§6.4).
- **Social graph & timing** — **mixnet** (onion routing + Poisson mixing delays) so no node
  sees both ends and timing correlation is defeated; plus **cover traffic** and **size
  padding** (§6.3).
- **Discovery** — name→key lookups routed through the mixnet, hiding *who* looks up *whom*
  (§3.7).

## 6.3 Mixnet, cover traffic, padding

- **Onion packets** (Sphinx-format, constant length) traverse a path of mix nodes; each peels
  one layer.
- **Mixing delay** — each mix holds packets a randomized (Poisson) time and reorders, so input
  and output streams cannot be timing-correlated.
- **Cover traffic** — nodes emit loop and drop cover messages at a steady rate; an observer
  cannot distinguish real activity from cover. Rate is a **tunable knob** (higher rate → more
  privacy, more bandwidth).
- **Size padding** — MOTEs are padded to fixed size buckets so length does not leak.

These are **normative and fully specified** — the Sphinx packet format, mix directory, path
selection, key rotation, cover-traffic rates, active-attack detection, entry guards, operator
diversity, fail-closed no-downgrade, and the high-security profile — in **§4.4.1–§4.4.10** with
parameters in **§16.3**; this subsection is the summary, §4.4 is the buildable specification.

Email's asynchrony is the enabling property: minutes of latency are acceptable for the
`private` tier, and higher latency yields stronger anonymity.

## 6.4 Why we avoid PIR (honest framing)

PIR (Pung, Talek) exists to hide **which record a client reads from an untrusted shared
mailbox that holds everyone's mail** — a leak that only exists *because* those systems use a
polled central store to support offline recipients. DMTAP makes a different **design choice**:
the recipient runs an **always-on node that receives by push** through the mixnet, so there
is no untrusted shared store being queried and thus no read-access-pattern to hide. We
therefore avoid PIR's *problem* (and its large cost) — this is a genuine architectural
difference, not a trick. (Note: Loopix itself *does* use provider store-and-poll to serve
offline clients; DMTAP's always-on-push is a deliberate divergence, not an inherent property
of mixnets.)

**But push delivery is not "recipient anonymity, solved."** Three residual exposures remain
and MUST be handled:

1. **Last-hop / receipt visibility.** The final mix and a global observer of the recipient's
   link learn that *a packet was delivered to node X*. An always-on node has a **stable,
   targetable network presence** — itself a fingerprint. The node's network identity MUST be
   cryptographically and durably **decoupled** from the human identity (pseudonymous peer id,
   no re-identifying metadata).
2. **Receipt-timing.** Without cover on the delivery link, observing the node reveals *when*
   messages arrive. The recipient node MUST receive a steady **Poisson cover stream** so real
   receipts are indistinguishable from cover (as Loopix does with loop + link cover).
3. **Long-term intersection / statistical-disclosure attacks.** Persistent presence is
   exposed to correlation across many rounds; cover traffic bounds but does not eliminate this
   (cf. Vuvuzela's differential-privacy noise, which degrades over volume). This is inherited,
   not solved.

**Offline buffering:** if a node is down, a peer/relay holds sealed ciphertext, retrieved on
return via an **unlinkable dead-drop token over the mixnet** (the buffer sees an anonymous
pickup, not "user X"). Cheaper than PIR and sufficient for v0; full PIR remains an option for
hostile-buffer scenarios.

## 6.5 Privacy tiers (and where each product sits)

| Tier | Path | Latency | Graph privacy | Default for |
|------|------|---------|---------------|-------------|
| `private` | full mixnet + cover | minutes | full (global passive) | mail, all control MOTEs, **normal-size files** |
| `fast` | direct / few-hop | sub-second | content only | live chat (both online), **large-file bulk** |

- Default is `private`. `fast` is opt-in. Because *choosing* `fast` is itself a signal, clients
  keep `private` as the standing default and cover it with cover traffic.
- **Files:** the control MOTE is always `private`. **Normal-size files route through the
  mixnet like messages (fully metadata-private).** **Large-file bulk** uses `fast` — but MUST
  be **onion-routed (a few hops, Tor-style) + size-padded to buckets + swarmed from multiple
  holders**, accepting bandwidth cost. **This bulk tier is explicitly weaker:** like Tor, it
  provides relationship anonymity against *local/partial* adversaries but is **vulnerable to
  end-to-end traffic correlation** by an adversary observing both endpoints (Murdoch–Danezis,
  2005). Swarming and padding raise cost; they do not guarantee anonymity. The strong
  guarantee is the *messaging* tier; moving large sensitive files inherits Tor's correlation
  exposure. See §5.5, §4.5.

## 6.6 Honest boundaries (what we do NOT claim)

1. **Global active adversary — the irreducible residual *after* the mechanisms.** DMTAP does not
   treat the active adversary as an unaddressed limit: §4.4.6–§4.4.10 specify concrete, normative
   defenses — **per-epoch mix replay caches** (drop replayed packets), **tagging-resistant
   integrity-protected Sphinx headers**, **memoryless Poisson mixing** (timing correlation stays
   hard even under active delay injection), **loop-cover active-attack detection** that turns
   drop/delay/flooding from undetectable into **detected → rotate + `HALT_ALERT` + fail-closed**,
   **entry guards + `≥ 3`-disjoint-operator path diversity** bounding long-term intersection,
   **attested (non-token) mix identities** for Sybil resistance, and a **no-silent-downgrade
   rule** (§4.4.9) so DoSing the mixnet cannot strip the tier. A **user-selectable high-security
   profile** (§4.4.10 — 5 hops, longer delays, constant-rate cover, tighter guards/diversity) is
   the lever to climb toward the bound.
   **After all of that**, an irreducible residual remains: by the **Anonymity Trilemma** (Das,
   Meiser, Mohammadi & Kate, *"Anonymity Trilemma: Strong Anonymity, Low Bandwidth Overhead, Low
   Latency — Choose Two,"* IEEE S&P 2018), strong anonymity provably **cannot** be had without
   paying in latency and/or bandwidth, so a global *active* adversary with unlimited resources that
   controls a large enough fraction of the network can still mount long-term statistical disclosure
   at *some* latency/cost point. DMTAP **approaches** that mathematical floor as the high-security
   profile's latency/overhead grows; it does **not** pretend to defeat an omnipotent adversary at
   zero cost. This is the honest floor *after* the maximal defense, not a substitute for it.
2. **Large-file bulk metadata.** Onion-routing + padding + swarming makes it *strong*, not
   *free* and not *perfect* — the fact and approximate volume of a large transfer may remain
   partially observable at high adversary capability.
3. **Endpoint compromise — the irreducible residual *after* the mechanisms.** DMTAP does not
   treat endpoint compromise as an unaddressed limit; it specifies concrete, normative defenses
   that shrink the blast radius to a single floor:
   - **Offline seizure is defended.** The local MOTE store is encrypted with a key **released
     only on device unlock** (biometric/PIN, §6.7), so a powered-off / locked stolen phone yields
     **nothing** — the ciphertext is inert without the unlock secret.
   - **Key exfiltration is defended.** `IK`/device keys **SHOULD** live in a hardware keystore as
     **non-exportable** keys (Secure Enclave / TPM / StrongBox / TEE, §1.2a), so even a software
     compromise cannot *copy* the key out to sign or decrypt elsewhere — it can only *use* it
     locally while the device is unlocked, and only until revoked.
   - **Single-device compromise heals.** A device is its own MLS leaf (§5.6); removing/rotating it
     (§1.5, §13.4) advances every epoch (**post-compromise security**) so the evicted key decrypts
     nothing further, and the **lost/stolen-device flow** (§6.7) evicts it from the cluster. One
     compromised device does **not** hand over other devices' independent sessions.
   - **Least-persistence options.** OPTIONAL **scoped sync** (recent-N-days on mobile) and the
     **`sensitive` / non-cached message flag** (§6.7 — a message the client MUST NOT persist at
     rest) further shrink what a later seizure can reveal; implementations SHOULD offer both.

   **After all of that**, one residual is irreducible: a device **actively compromised while
   unlocked and in use** sees exactly what the *user* sees — it can read decrypted content and
   exercise the (hardware-held) key in real time. **No protocol can prevent an authorized,
   unlocked endpoint from reading its own screen.** DMTAP shrinks endpoint compromise to precisely
   this floor — live, unlocked, in-use — rather than the old "any seized cluster member leaks the
   whole history." That is the honest boundary *after* the maximal defense, not in place of it.
4. **First-contact MITM.** Before KT/OOB verification, the *first* name→key resolution can be
   MITM'd; DMTAP fails closed if KT is unreachable at first contact (§3.3, §3.4).
5. **Metadata privacy is weakest-link.** One leaky component (a bad lookup, a client bug, a
   timing side-channel) can unravel graph privacy. It is harder to *achieve* than to *design*;
   implementations MUST be conservative and audited.
6. **v0 key transparency is not equivocation-proof.** A v0 single, non-gossiped KT log can show
   different histories to different observers (split-view) until v1 gossip/multi-log auditing
   exists (§3.5). In v0, KT is **tamper-evident-after-the-fact and self-monitorable, but not a
   trusted single source** — a network SHOULD run multiple independent logs even in v0, and
   clients SHOULD treat a single unaudited log as advisory, leaning on OOB verification for
   high-value contacts. This is a real tension with the sovereignty goal and is stated as such.
   For **DMTAP-Auth (§13)** specifically, a split view is a **silent per-RP account takeover**, so
   high-value login RPs MUST require **multi-log consistency or an OOB-verified pin even in v0**
   (§13.7).
7. **Group handshake ordering is a metadata concentration point.** The per-group committer/
   ordering channel (§5.1) necessarily sees all of a group's handshake traffic; this is an
   explicit exception to the "no single node sees both ends" framing, bounded by rotating the
   committer and keeping application traffic on the mixnet.
8. **`redact`/`expires` are unenforceable** against a non-compliant recipient that already holds
   the plaintext — they are cooperative hints, not guarantees (relevant to any "right to
   erasure" framing).

DMTAP states these boundaries in-product. Honest, disclosed limits beat a false "perfectly
anonymous."

## 6.7 Data at rest

- The node encrypts the mailbox, file blobs, and keys **at rest** under a device/identity key.
- **Unlock-gated store encryption (normative, SHOULD).** The at-rest key SHOULD be **released
  only on device unlock** — wrapped by a key the hardware keystore (§1.2a) yields only after a
  successful biometric/PIN authentication, and **evicted from memory on relock/timeout** (§16.9).
  This distinguishes two threat cases sharply: an **offline-seized** device (powered off or
  locked) yields only inert ciphertext (**defended**); a **live, unlocked, in-use** device can
  read what the user reads (**the residual**, §6.6 item 3). Implementations MUST NOT keep the
  at-rest key resident indefinitely across a locked device.
- **`sensitive` / non-cached messages (normative, MAY-send / MUST-honor).** A sender MAY mark a
  message `sensitive` (a `Headers` flag, §18.3.6): the receiving client **MUST NOT persist it at
  rest** — it is held in memory for an ephemeral view and dropped, never written to the durable
  MOTE store — so a *later* seizure reveals nothing of it. Like `redact`/`expires` (§6.6 item 8)
  this is **cooperative** (a non-compliant or compromised recipient can still copy what it can
  read); it is a least-persistence reduction, not a guarantee against a live endpoint.
- **Lost/stolen-device flow (normative).** On suspected device compromise the owner (from any
  surviving cluster device, or after recovery §1.4) performs: (1) an **MLS Remove** of the lost
  device from every group (§5.8.2) and, for a personal cluster, from the device group (§5.6);
  (2) a **device-key rotation** (§1.5) re-keying any identity/recovery material the device held;
  (3) **session revocation** of all that device's auth sessions (§13.4, which a device-key
  rotation triggers wholesale). Steps (1)–(2) advance every affected epoch, so the evicted key
  has **post-compromise** — it decrypts no message sent after the eviction Commit. This reuses
  existing machinery (§1.5, §5.8.2, §13.4); no new revocation protocol is introduced.
- Messages get MLS **forward secrecy**; files use **per-file keys** (blast radius = one file)
  delivered over the forward-secret channel. Persistent, re-readable data cannot ratchet like
  ephemeral messages — this is a property of files, not a DMTAP flaw (§5.5).
- **Access revocation on group-member removal (normative).** MLS removal blocks a removed
  member's *future* messages but does NOT revoke file keys they already hold. Therefore, when a
  member is removed from a shared file folder (§5.5), the node MUST **re-key and re-encrypt**
  every file the removed member had access to **by default**, and redistribute the new keys to
  remaining members. **Skipping the re-key is an explicit, surfaced opt-out** (e.g. low-sensitivity
  shared media where the cost outweighs the benefit), never the silent default; folders marked
  **confidential** MUST always re-key (opt-out forbidden). Without a re-key, a removed member
  retains indefinite read access to already-shared files. Clients MUST surface that pre-removal copies a
  member already downloaded cannot be recalled — deletion/recall is cooperative-only, never a
guarantee (the un-share limit; `redact`/`expires` are unenforceable cooperative hints, §6.6 item 8).
- Recovery custody (§1.4) governs how at-rest keys survive device loss.
