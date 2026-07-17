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
- **Size padding** — MOTEs are padded to fixed size buckets so the *exact* length does not leak
  (an observer learns only which of the four size buckets, §4.4.1).

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
| `private` | full mixnet + cover | minutes | strong (global passive) — quantified, see §6.4/§4.4.11 | mail, all control MOTEs, **normal-size files** |
| `fast` | direct / few-hop | sub-second | content only | live chat (both online), **large-file bulk** |

- Default is `private`. `fast` is opt-in. Because *choosing* `fast` is itself a signal, clients
  keep `private` as the standing default and cover it with cover traffic.
- **Files:** the control MOTE is always `private`. **Normal-size files route through the
  mixnet like messages (same private-tier metadata privacy as messages — strong vs a global
  passive adversary; the §6.6 residuals apply).** **Large-file bulk** uses `fast` — but MUST
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
   One concrete edge of that residual: **sub-threshold selective dropping** — an adversary dropping
   fewer than the loop-loss detection threshold (< 20%, §16.3) of a target's packets — stays under
   the §4.4.7 detector, so it is **bounded, not eliminated**; the trade is that so little is dropped
   it accomplishes little, and the **High-security profile** (faster loops, §4.4.10) tightens the
   detectable floor further.
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
   committer and keeping application traffic on the mixnet. **Additionally, in v0 the ordered,
   reliable handshake channel is not itself required to ride the mixnet** (§5.1 routes it for
   liveness), so a *network* observer — not only the committer — can see the timing of membership
   changes (Add/Remove Commits, Welcomes), which is membership-graph metadata the mixnet otherwise
   hides. Application traffic stays cover-indistinguishable; the handshake channel does not. Closing
   this fully (carrying handshakes over the `private` tier, realizing "ordered/reliable" as the
   committer's on-arrival log order rather than an in-transit bypass, and bringing any retained
   low-latency committer path under the §4.4.9 no-silent-downgrade rule) is a tracked v0 residual.
8. **`redact`/`expires` are unenforceable** against a non-compliant recipient that already holds
   the plaintext — they are cooperative hints, not guarantees (relevant to any "right to
   erasure" framing).
9. **Push wake-signaling on platforms that mandate a push service.** The wake-signaling layer
   (§4.9) is content-free, sender-blind, node-originated, and RFC 8291-encrypted, and on
   Android / desktop / web it can run **fully open and self-hosted** over UnifiedPush or Web Push
   (§4.9.3) — a push relay there sees nothing but a **self-edge** ("this node woke its own
   device"), never a sender, a correspondent, or any content. **On iOS, however, the OS mandates
   APNs**: a background app cannot be woken except through Apple's push service, so on iOS **Apple
   sees timing metadata** — that *this device* was pinged at time *T* — even though the wake carries
   no sender, no subject, and no content (it is opaque ciphertext, §4.9.1), and even though the node
   MAY jitter/batch wakes to blunt correlation (§4.9.1). This residual is **disclosed and minimized,
   not hidden**: DMTAP strips the payload and the social graph from the wake, but it cannot remove
   Apple from the loop on a platform that forbids any other wake path. It is the push analogue of the
   global-active-adversary residual (item 1) — the maximal open mechanism is applied first, and the
   irreducible **platform-mandated** leak is stated plainly rather than papered over. Where the
   platform permits an open provider (UnifiedPush / Web Push), this residual does not arise at all.
   Because a per-arrival wake is itself an activity-dependent timing signal *outside* the mixnet's
   cover traffic, it composes with item 1: under the **High-security profile** (§4.4.10) wake
   jitter/batching is therefore a **MUST** (§4.9.1), so the wake path does not reintroduce the
   recipient-arrival timing channel the profile's constant-rate cover closes.
10. **Referenced-file availability is a contract, not a property of the hash.** A content address
    stays useful only while **some node still serves the bytes**, so "forever access even after the
    peer drops" is **not free**. DMTAP does not treat this as unaddressed: **Inline** files are
    durable-by-delivery and **Attached** files are pushed into the recipient's own store (§5.5.1),
    so neither depends on the sender staying online; a **Referenced** file carries an explicit
    **durability class** (§5.5.2) and clients **SHOULD auto-pull-and-pin** small referenced files
    to convert best-effort → durable. **After all that**, one residual is irreducible: a Referenced
    file left at the **origin-hold** default (best-effort, no pin, no replica) **MAY become
    permanently unavailable** if the origin node drops before the recipient fetches
    (`ERR_FILE_UNAVAILABLE`, §21) — a **durability**, not a confidentiality, limit. It is closed by
    choosing `recipient-pinned`, `cluster-replicated(N)`, or `pinned(term)` (§5.5.2), each of which
    costs real storage; the default is disclosed as best-effort rather than silently implying
    permanence. Relatedly, **deletion is cooperative-only**: you can stop serving *your* copy but
    **cannot force-delete** bytes another party has pinned (§5.5.4) — the storage analogue of the
    `redact`/`expires` un-share bound (item 8).
11. **Publishing a public object (§22) is irrevocable.** Unlike a sealed file (item 10, item 8),
    a DMTAP-PUB object is content-addressed and swarmed by design — once published, it **cannot
    be unpublished**: any holder may continue serving it indefinitely, and there is no protocol
    mechanism to force its removal from the network. Publication is a deliberate, one-way act
    (§22), not a reversible share.
12. **Holders of public objects are not blind.** A sealed-file holder serves opaque ciphertext
    (item 5.5, §5.5.3) and cannot inspect it; a DMTAP-PUB holder serves **plaintext** it can read.
    Serving public content therefore shifts an operator's moderation/liability posture in a way
    serving sealed chunks does not — hence public-object serving is a **per-operator opt-in**
    (`pub-1`, §10.2, §22), never a default-on behavior of a conformant node.

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
  rotation triggers wholesale); and (4) **deniable-session teardown** — because the pairwise
  Double Ratchet lies **outside** MLS, an MLS Remove does not reach it, so the owner MUST also run
  the §5.2.1(f) flow: withdraw/rotate the deniable prekeys (`idk`/`spk`/`opks`) the device could
  have held, tear down its in-flight ratchets, and **re-establish** any deniable conversation to
  restore post-compromise security. Steps (1)–(2) advance every affected MLS epoch and step (4)
  reboots each deniable ratchet, so the evicted key has **post-compromise** — it decrypts no
  message sent after the eviction. This reuses existing machinery (§1.5, §5.8.2, §13.4, §5.2.1(f));
  no new revocation protocol is introduced.
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

## 6.8 Transport-path provenance does not weaken metadata privacy (normative)

DMTAP lets a recipient learn and verify a message's **transport-path provenance** — the tier it
arrived on, whether it was gateway-touched or pure-mesh, and a coarse hop descriptor (§7.8, wire
objects §18.3.11 / §18.8.1). This is deliberately constrained so it **reveals only trust-boundary
crossings the recipient can already infer, and nothing more** — it MUST NOT weaken sealed sender
(§6.2) or mixnet anonymity (§4.4):

- **Provenance answers "which trust boundaries did this cross?", never "which nodes carried it?"**
  For the `private` tier the recipient learns only the **profile floor** the path satisfied
  (`≥ 3` / `≥ 5` hops, §4.4.10) — **never** a mix-node identity, address, exact hop count, path
  descriptor, or per-hop timing. The private path is, by design, unknown even to the recipient's
  own node (that is the anonymity guarantee, §6.2); a `ProvenanceRecord` MUST NOT contain, and a
  node MUST NOT synthesize, anything from which the path could be reconstructed (§18.8.1 privacy
  invariants). The coarse hop descriptor is a **statement about the profile**, not a trace.
- **The gateway attestation is sealed, so it leaks nothing to intermediaries.** The
  `GatewayAttestation` that marks a message gateway-touched (and names the gateway domain, time,
  and legacy sender) rides **inside the encrypted `Payload`** (§18.3.5), visible only to the
  recipient — a mixnet intermediary sees only ciphertext (§2.2, §6.2). The legacy-sender address it
  carries is the recipient's own inbound mail, which the recipient legitimately sees anyway.
- **The provenance record never leaves the owner's devices.** A `ProvenanceRecord` is node-local,
  served only over the authenticated client surface to the owner's own cluster (§8.1, §8.3,
  §19.9); it is never attached to a forwarded MOTE or exposed to a third party (§18.8.1). It adds
  **no** new on-wire field observable by any intermediary.
- **Gateway-touched vs. pure-mesh exposes only what the boundary already exposed.** A gateway leg
  is *unavoidably* plaintext (§7 opening) — provenance merely lets the recipient **see that this
  already happened**, it does not create new exposure. A pure-mesh message's "no attestation"
  state is the *absence* of a signal, which leaks nothing.

This keeps provenance consistent with the §12.3 inviolable rule: it is a **transparency** feature
over boundaries the design already makes visible, not a metadata-privacy regression.

## 6.9 Security properties (falsifiable claims → mechanism → adversary → residual)

The prose above states the threat model; this subsection states, in one place, **each security
property DMTAP claims as a falsifiable statement** — a claim an auditor or a formal tool (Tamarin,
ProVerif, a symbolic model) can attempt to **refute**. Every property is mapped to the mechanism
(§) that provides it, the adversary it holds against, and the **disclosed residual** (the §6.6 item
or the mechanism's own honest-limit clause). Nothing here is claimed *beyond* its residual: where a
property has a caveat, the caveat is part of the statement.

Two structural facts frame the whole table. **Confidentiality and authenticity (SP-1, SP-2) hold
against a global *active* adversary** — breaking them requires a signing/decryption key, which
adversary reach does not confer. **The metadata properties (SP-3–SP-5) degrade with adversary
reach**: strong against a global *passive* adversary, *bounded* (not defeated) against a global
*active* one (§6.6 item 1). This asymmetry is the honest core of DMTAP's posture.

**SP-1 — Message confidentiality (end-to-end).**
- *Claim (falsifiable):* No party other than the intended recipient(s) — no relay, mix, gateway,
  directory, KT log, or hosted operator — can recover `Payload` plaintext.
- *Holds by:* MLS group encryption / HPKE sealing of `Payload` into `Envelope.ciphertext` (§2.4,
  §2.1, §5.1); every intermediary is content-blind (§6.1, §6.2).
- *Against:* network eavesdropper, malicious relay/mix, curious directory/KT, and a global
  **passive and active** adversary alike (confidentiality does not degrade with global reach
  *once the name→key binding is established* — a first-contact MITM before KT/OOB is the disclosed
  residual below).
- *Residual:* the endpoint floor — a device compromised while unlocked and in use reads its own
  plaintext (§6.6 item 3); the legacy-gateway leg is plaintext by construction (§7 opening); a
  first-contact MITM before KT/OOB can substitute the encryption key (§6.6 item 4). Harvest-now:
  content is PQ-migratable (suite `0x02`, §1.1) but v0 classical-suite ciphertext recorded today is
  exposed to a future quantum break until the identity migrates (§4.4.12 content note).

**SP-2 — Content authenticity & integrity.**
- *Claim:* A recipient accepts a `Payload` as authored by identity `IK` only if `IK` actually
  signed it, and detects any bit-flip of a MOTE.
- *Holds by:* `Payload.sig` verified under `Payload.from` at §2.7 step 8; content-address `id`
  (BLAKE3-256 of `ciphertext`) checked at §2.7 step 2 (§2.2); canonical signing/hashing preimages
  (§18.9); name→`IK` binding via pinning + KT (§3.4, §3.5).
- *Against:* all adversaries incl. global active (forgery needs the signing key).
- *Residual:* authenticity of the *name→key binding* is only as strong as the KT profile in force
  (SP-9, §6.6 item 6); deniable-mode messages deliberately carry **no** content signature — their
  authenticator is a shared-key MAC (SP-7, §5.2.1(c)).

**SP-3 — Sender anonymity against a GLOBAL PASSIVE adversary (the headline property).**
- *Claim:* A global passive adversary observing all links and all timing cannot learn who sent a
  `private`-tier MOTE to whom, beyond the disclosed residual.
- *Holds by:* sealed sender (identity + authenticating signature sealed inside the payload, §2.2,
  §6.2); Sphinx onion routing + memoryless Poisson mixing (§4.4.1, §4.4.6); mandatory cover traffic
  + size-bucket padding (§4.4.5, §6.3); mixnet-routed lookups (§3.7).
- *Against:* the global **passive** adversary — DMTAP's primary/headline target (§6.1).
- *Residual:* metadata *reduction*, not elimination — sealed sender is statistically eroded by
  receipt/timing side channels (Martiny et al., NDSS 2021), which is why cover traffic is
  load-bearing, not optional (§6.2); last-hop delivery observability remains (§6.4 items 1–3);
  long-term intersection is bounded, not eliminated (§6.6 item 5, §4.4.8); while the fleet is small
  the guarantee is Tor-with-few-relays, not a strong global mixnet (§4.4.11); the v0 onion is not PQ
  — a harvest-now adversary could retroactively deanonymize the recorded social graph (§4.4.12).

**SP-4 — Sender anonymity against a GLOBAL ACTIVE adversary (reduced and disclosed).**
- *Claim:* A global *active* adversary (inject/drop/delay at will) is forced to pay latency and/or
  bandwidth to correlate, and its drop/delay/flooding is **detected and responded to** — but it is
  **not fully defeated**.
- *Holds by:* per-epoch replay caches, tagging-resistant Sphinx (header MAC `γ` + wide-block LIONESS
  payload PRP), memoryless Poisson mixing (§4.4.6); loop-cover active-attack detection →
  rotate + `HALT_ALERT` + fail-closed (§4.4.7); entry guards + **attested** operator-diversity
  (§4.4.8); fail-closed no-downgrade (§4.4.9); the user-selectable High-security profile (§4.4.10).
- *Against:* the global **active** adversary.
- *Residual (bounded, not eliminated):* the Anonymity Trilemma floor (§6.6 item 1, Das et al.,
  IEEE S&P 2018) — strong anonymity provably cannot be free; sub-threshold selective dropping stays
  under the §4.4.7 detector (§6.6 item 1, §16.3); the colluding entry+exit floor is ≈ *f*² and is
  **not** reduced by adding hops (§4.4.10). DMTAP *approaches* the mathematical floor as the
  profile's latency/overhead grows; it does not claim to defeat an omnipotent active adversary. The
  mechanism-model simulation corroborating this shape (chance-floor convergence, 20%-loss detection,
  ≈ *f*² collusion, hops-≠-collusion-defense) is reported — with its honest caveats — in §6.10.

**SP-5 — Recipient unlinkability (blinded delivery tags).**
- *Claim:* An observer cannot link successive `private`-tier deliveries to the same recipient *by
  the routing tag*, nor tie that tag to the recipient's persistent identity key.
- *Holds by:* blinded delivery tag `BT = HKDF(shared_secret, epoch_day)`, recognized by the
  recipient but unlinkable across time and across observers (§2.2a); network/human identity
  decoupling and recipient-side cover (§6.4).
- *Against:* a global passive adversary / final mix, for the *tag-linkage* property.
- *Residual (stated, not overclaimed):* blinding removes the persistent-key linkage in the envelope;
  it does **not** hide *that a packet was delivered to a particular always-on node* — a stable
  network presence is itself a fingerprint (§6.4 item 1, §2.2a). Recipient-side cover (§4.4.5) blurs
  receipt *timing* but does not erase last-hop observability. Implementations MUST NOT present
  blinded tags as full recipient anonymity (§2.2a). This is a *reduction*, not a solved property.

**SP-6 — Forward secrecy & post-compromise security.**
- *Claim:* Compromise of a party's current keys does not expose its past messages (FS), and after a
  Commit/epoch advance an evicted key cannot decrypt future messages (PCS).
- *Holds by:* MLS TreeKEM epoch advancement (§5.1, §5.2); mix-key epoch rotation + deletion at
  `valid_until` gives the mixnet FS against later node seizure (§4.4.4); per-file keys for at-rest
  blobs (§6.7); the optional deniable mode adds *per-message* FS/PCS via the Double Ratchet
  (§5.2.1(b)).
- *Against:* an adversary that later seizes a device or mix (FS), and a bounded-duration endpoint
  compromise that is subsequently revoked (PCS).
- *Residual:* MLS PCS is **per-epoch/per-Commit**, coarser than Double-Ratchet per-message healing —
  a deliberate simplicity trade, not a strict improvement (§5.2); persistent files cannot ratchet
  (§6.7); deniable-mode PCS needs *bidirectional* traffic — a send-only channel retains only FS
  (§5.2.1(b)); PCS heals only *after* the evict/rotate flow runs (§6.7).

**SP-7 — Deniability / repudiation (optional 1:1 mode).**
- *Claim:* In the opt-in deniable 1:1 mode, neither party — nor a third party — can produce a
  *cryptographic* proof, from ciphertext and keys alone, that the *other* party authored a given
  message.
- *Holds by:* a separate pairwise X3DH/PQXDH + Double-Ratchet channel authenticated by a
  **shared-key MAC** (either party could forge it) run beside the 2-member MLS group; every
  long-term signature covers only a *public key*, never content; a `DeniablePayload` carrying a
  signature is rejected (`0x040F`) (§5.2.1(a)–(c)).
- *Against:* a judge given only the cryptographic transcript.
- *Residual (bounds):* the **default MLS path is non-repudiable** — deniability is opt-in, 1:1 only;
  groups (n ≥ 3) stay attributable (§5.2, §5.2.1(d)). It is **not** endpoint protection — a
  logged/coerced endpoint's plaintext still discloses content (§5.2.1(e) item 1 = §6.6 item 3). X3DH
  *online/interactive* deniability is weaker than its offline form (Vatandas et al., ACNS 2020);
  DMTAP claims **no more than Signal's proven guarantees** (§5.2.1(e) item 2).

**SP-8 — Downgrade resistance.**
- *Claim:* An adversary cannot **silently** force a party onto a weaker suite, tier, profile, or
  capability set; every downgrade is either refused (fail-closed) or a deliberate, user-surfaced
  choice.
- *Holds by:* unknown `v`/`suite` rejected fail-closed (§10.1); per-contact suite high-water-mark
  ratchet (§1.3, `0x020F`); fail-closed no `private → fast` and no High-security → Standard
  downgrade (§4.4.9, `0x0310`); monotonic capability announcements (§10.2, `0x030A`); deniable-mode
  no-silent-downgrade (§5.2.1(d), `0x040E`). These rules are collected and audited as a **set** in
  §10.7.
- *Against:* a global active adversary (DoS-to-downgrade) and an on-path MITM.
- *Residual:* enforcement is per-recipient and local, so it is only as good as the implementation —
  "metadata privacy is weakest-link" (§6.6 item 5); a suite high-water-mark lowers **only** through
  an explicit `IK`-authorized retirement the owner performs (§1.3), never via an inbound message.

**SP-9 — Key transparency / equivocation detection (profile-dependent — stated honestly).**
- *Claim:* Under **v1-hardening KT** (log-type `0x02`, §3.5.2) a log that shows different
  `name → ik` histories to different observers (split view) is **detected, attributed, and responded
  to** — HALT_ALERT + evict, fail-closed below a `> n/2` quorum. Under **v0-minimal KT** (log-type
  `0x01`, §3.5.1) equivocation is only **tamper-evident-after-the-fact and deterred — NOT reliably
  detected.**
- *Holds by:* v1 STH gossip + consistency proofs, multi-log `> n/2` quorum, monitor/auditor roles,
  equivocation halt (§3.5.2, `0x0107`/`0x0110`/`0x0111`/`0x0112`); v0 fail-closed-on-unreachable +
  owner self-monitoring (§3.3, §3.5.1, `0x0106`).
- *Against:* a malicious or split-view KT log (v1: detected; v0: only deterred).
- *Residual:* **v0 KT is not equivocation-proof (§6.6 item 6)** — the honest core limit; this is the
  one property whose strength is *gated on the negotiated profile*, and the spec does **not** claim
  v0 delivers it. High-value contacts, and every DMTAP-Auth login RP, MUST require multi-log
  consistency or an OOB-verified pin even in v0 (§3.4.1, §13.7).

**SP-10 — Recoverability from key/device compromise.**
- *Claim:* An owner can recover identity **and all relationships** after losing or having a
  device/factor compromised, and a *partial* compromise cannot silently escalate to an
  owner-evicting takeover.
- *Holds by:* versioned signed `RecoveryPolicy` with distinct `recover_threshold`/`rotate_threshold`
  (§1.4); weakening-needs-quorum-even-under-`IK` + an asymmetric **72 h** veto window (§1.4 rules
  3–4, §16); real revocation by re-key / VSS / redistribution / FROST (§1.4 rule 5); cross-signed
  `IK` rotation chain (§1.5); MLS Remove + device-key rotation + deniable teardown heal the cluster
  forward (§6.7, §1.2a, §5.2.1(f)); recovery invalidates prior sessions (§13.4, `0x050A`).
- *Against:* device loss, single-factor theft, and a stolen-`IK`-alone proactive takeover attempt.
- *Residual:* losing `IK` **and** enough factors simultaneously is unrecoverable; obtaining `IK`
  **plus** a `rotate_threshold` quorum of factors is an owner-unrecoverable takeover — *worse* than
  loss (§1.4 "bottom turtle"). Recovery restores the **key**, not prior **content**, absent a
  surviving cluster device or an encrypted backup (§1.4 backup/restore).

**SP-11 — Anti-abuse without deanonymization.**
- *Claim:* A recipient can rate-limit and block cold senders **without learning who they are** and
  **without linking a sender across recipients**.
- *Holds by:* ARC anonymous, per-origin-scoped, rate-limited tokens (§9.3), PoW fallback (§9.4), and
  postage (§9.5), all evaluated on the sealed envelope **before decryption** (§2.7 step 6), each
  bound to the ephemeral `sender_key` to defeat proof theft/replay (§9.2a); the anonymity-preserved
  principle (§9.1 item 4).
- *Against:* a recipient (and its operator) attempting to deanonymize or cross-link senders.
- *Residual:* repeat abuse is bounded at the **issuer** layer, not the recipient layer —
  recipient-side cross-recipient linking is deliberately unavailable and not claimed (§9.3.2); an
  unvetted/self-issued token carries a **zero** budget (§9.3.1); a hidden-membership list's committer
  holds a disclosed per-delivery vouch-trust power (§9.9). Mix-layer flooding is bounded only by
  content-blind controls (per-connection/operator rate limits + stake, §9.8).

### 6.9.1 The map, in one table

| # | Property | Holds by (§) | Against | Residual (§) |
|---|----------|--------------|---------|--------------|
| SP-1 | Message confidentiality (E2E) | §2.4, §5.1 | passive **+ active** | endpoint floor §6.6 item 3; gateway leg §7; first-contact §6.6 item 4; harvest-now §4.4.12 |
| SP-2 | Content authenticity / integrity | §2.7 step 8, §2.2, §18.9 | passive **+ active** | binding = KT profile SP-9 / §6.6 item 6; deniable = MAC not sig §5.2.1(c) |
| SP-3 | Sender anonymity vs global **passive** | §2.2, §4.4.5–6, §6.3, §3.7 | global passive (headline) | reduction not elimination §6.2, §6.4, §6.6 item 5, §4.4.11–12 |
| SP-4 | Sender anonymity vs global **active** | §4.4.6–§4.4.10 | global active | Trilemma floor §6.6 item 1; sub-threshold drop §16.3 |
| SP-5 | Recipient unlinkability (blinded tags) | §2.2a, §6.4 | passive / final mix | last-hop observability remains §6.4 item 1, §2.2a |
| SP-6 | Forward secrecy + PCS | §5.2, §4.4.4, §6.7, §5.2.1(b) | later seizure / revoked compromise | per-epoch coarser §5.2; send-only = FS only §5.2.1(b) |
| SP-7 | Deniability / repudiation (1:1 opt-in) | §5.2.1 | cryptographic-transcript judge | default non-repudiable §5.2; not endpoint §6.6 item 3; X3DH online bound §5.2.1(e) |
| SP-8 | Downgrade resistance | §1.3, §4.4.9, §10.2, §10.1 | active DoS / MITM | weakest-link §6.6 item 5 (full set §10.7) |
| SP-9 | KT / equivocation detection | §3.5.2 (v1) / §3.5.1 (v0) | malicious KT log | **v0 not equivocation-proof §6.6 item 6** |
| SP-10 | Recoverability from compromise | §1.4, §1.5, §6.7, §13.4 | loss / partial compromise | `IK`+quorum takeover; content needs backup §1.4 |
| SP-11 | Anti-abuse without deanonymization | §9.3–§9.5, §2.7 step 6, §9.2a | deanonymizing recipient/operator | issuer-layer only §9.3.2; committer vouch §9.9 |

An implementation or a formal model that exhibits a counterexample to any SP-*n* **claim line
above** — without invoking its stated residual — has found a spec-level defect, and it MUST be
filed as such (§10.4). That is the point of stating them falsifiably.

**Where the formal models live.** This specification repository ships the **CDDL** wire grammars
(§18) and the conformance vectors (§10.3); the **symbolic/ProVerif protocol models** that exercise
the SP-*n* claims are maintained in the **reference implementation's `formal/` directory** (the
Envoir monorepo), not in this spec repo. They are versioned against the mechanisms cited above and
are the executable counterpart to this falsifiable-claims table; a divergence between a model and
this text is resolved in favor of the **spec** (§10.4).

## 6.10 Measured anonymity evidence (mechanism-model simulation — supporting, not deployment proof)

The §6.9 properties are stated to be **refutable**; this subsection reports what a **mixnet
anonymity simulator** in the reference measured when the §4.4 mechanisms were exercised against a
global adversary. It is offered as **corroborating evidence** for SP-3/SP-4 (and the §6.6 item 1
honest floor), and it is **explicitly caveated**:

> **This is a mechanism-model simulation, NOT the deployed network.** The simulator models the
> §4.4 constructions — Poisson per-hop mixing, loop/drop cover, stratified path selection, entry
> guards, operator diversity, and colluding-mix placement — under an idealized traffic model. It
> is a *sanity check that the mechanisms behave as designed in the abstract*, not a measurement of
> a real fleet, and it is **not a substitute** for the pre-deployment external audit gate
> (§12.8.4). Real-world anonymity depends on the live anonymity set (§4.4.11), implementation
> fidelity (the weakest-link caveat, §6.6 item 5), and side channels a mechanism model does not
> capture. No production claim rests on these numbers.

With that caveat, the simulator's four measured findings each map to a claimed property:

1. **Passive correlation converges toward the 1/N chance floor as cover and hops grow.** Against a
   *global passive* adversary, the simulated probability of correctly linking a target's sender to
   its receiver fell toward **1/N** (N = the anonymity set — indistinguishable senders) as
   cover-traffic rate and hop count increased: with enough cover and mixing, the adversary's
   guess approached a **uniform random guess over the anonymity set**. This is the mechanism-model
   evidence for **SP-3** (sender anonymity vs. global passive) and shows *why* cover traffic is
   load-bearing, not optional (§6.2, §4.4.5) — the convergence is *to* the floor, not *below* it,
   and it is a floor set by N, i.e. by adoption (§4.4.11), which the model idealizes.

2. **Active drop attacks are detected at the 20%-loss loop threshold.** When the simulated
   adversary dropped packets on a target's paths, the **loop-cover return fraction** (§4.4.7) fell
   below its detection threshold at ≈ **20% loss** (§16.3), triggering the inferred-active-attack
   response — rotate + `HALT_ALERT` + fail-closed (`0x030F`). This is the evidence for **SP-4**'s
   "drop/delay is *detected and responded to*": above-threshold suppression is caught; the honest
   residual is **sub-threshold** dropping (< 20%), which stays under the detector but, dropping so
   little, accomplishes correspondingly little (§6.6 item 1). Faster loops (High-security, §4.4.10)
   tighten this floor.

3. **Anonymity degrades with the compromised fraction f, tracking ≈ f².** As the fraction *f* of
   mixes under adversary control grew, the simulated deanonymization probability rose **≈ f²** —
   the joint probability that **both** the entry **and** the exit of a path are adversarial, the
   placement from which a colluding pair correlates a flow. This is the quantitative shape of the
   §4.4.8 bound and of the §6.6 item 1 residual: entry guards + **attested** operator diversity
   (§4.4.8) are what hold the exponent at ≈ *f*² rather than letting a single Sybil operator faking
   *N* identities collapse it to ≈ *f* (§4.4.8, §10.7.2). It also quantifies **SP-4**'s "bounded,
   not eliminated" — the bound is ≈ *f*², not zero.

4. **More hops defend against timing correlation but NOT against colluding entry+exit mixes.** The
   single most important honesty result: increasing the hop count in the simulation **lowered the
   timing-correlation success of a network *observer*** (more memoryless hops to defeat, finding 1),
   but left the **≈ f² colluding-entry+exit deanonymization essentially unchanged** — adding honest
   middle hops does not reduce the chance that the two *ends* are both adversarial. Hop count and
   operator diversity therefore defend **different** attacks: hops buy timing-correlation resistance
   (against observers), diversity + guards buy entry+exit-collusion resistance (against colluding
   mixes), and **neither substitutes for the other** (§4.4.10). Any claim that "just add more hops"
   defeats a colluding-endpoint adversary is refuted by this measurement and MUST NOT be made.

**Honest reading.** These results *support* the claims exactly as §6.9 states them and *refute* the
overclaims §6.6 disallows: passive correlation is driven to the chance floor (not below), active
dropping is detected (above threshold, not below), and the colluding-endpoint residual is a real
≈ *f*² floor that more hops do not remove. The evidence strengthens the honest posture; it does not
license a stronger claim than the residuals permit, and it does not replace §12.8.4's external
audit.
