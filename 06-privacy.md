# 6. Privacy & Threat Model

DMTAP protects **content, authenticity, and metadata**. This section states the threat model
honestly, defines the guarantees, and marks the boundaries we do *not* claim.

## 6.1 Adversary model

| Adversary | Capability | DMTAP posture |
|-----------|-----------|--------------|
| Network eavesdropper (local) | reads links near a node | defeated by end-to-end encryption; sealed sender hides the sender from this observer (not from a global one, below) |
| Malicious relay / mix node | sees ciphertext it forwards | defeated (content-blind; on the `fast` tier a relay still sees the endpoints it directly connects, §6.5) |
| Curious directory / DNS / KT log | sees lookups | binding tamper-evident (KT §3.5); lookup privacy beyond that is the opt-in mixnet's routing (§3.7, [docs/research/mixnet.md](docs/research/mixnet.md)) |
| **Global passive adversary** | observes all links, all timing | **default posture: NOT defeated for the social graph.** By default (`fast` tier), a global passive adversary recovers the communication graph via IP + timing + volume correlation; sealed sender denies it the sender's identity/signature but not the graph. Graph/timing privacy against this adversary is available only via the opt-in, research-tier mixnet (§4.4, `private` tier) — see the honest restatement below and §6.9 SP-3 |
| Global *active* adversary (inject/drop/delay at will, unlimited resources) | shapes traffic to correlate | **not defeated by default**; the opt-in mixnet adds detection/response mechanisms discussed in §6.6 item 1, themselves research-tier |
| Compromised endpoint (node seized/keylogged) | reads that node's plaintext | **hardened, then bounded** (§6.6 item 3): hardware-backed non-exportable keys (§1.2a), device-unlock-gated at-rest encryption (§6.7), per-device sealing, and fast revocation heal all cases **except** a device *actively compromised while unlocked and in use* |

**DMTAP's default guarantee is a metadata *reduction*, not global-passive-adversary immunity.**
Sealed sender (§6.2) keeps the sender's identity and authenticating signature inside the
encrypted payload, so every intermediary on the default `fast` tier sees only ciphertext
addressed to an opaque destination — not who sent it. It does **not**, by itself, hide *that a
message moved from IP A to IP B at time T of size S*: a global passive adversary correlating IP,
timing, and volume across links can still recover the communication graph on the default tier.
**Strong metadata privacy against a global passive adversary — hiding the graph and timing, not
just the sender's identity — is the job of the mixnet, and the mixnet is an opt-in, research-tier
layer** (§4.4, [docs/research/mixnet.md](docs/research/mixnet.md)), not a default guarantee. This
is the honest restatement of what was previously stated as DMTAP's headline property; see §6.9
SP-3/SP-4 for the falsifiable claims as they now stand.

## 6.2 What is protected, and how

- **Content** — MLS/HPKE end-to-end encryption (§2, §5). Only recipients decrypt.
- **Authenticity** — signed payloads; identity bound to name via KT (§1, §3).
- **Sender identity, by default (`fast` tier)** — **sealed sender**: the sender's identity and
  authenticating signature live *inside* the encrypted payload; intermediaries see only
  ciphertext to an opaque destination (§2.2). Honest scope: sealed sender hides the sender from
  *intermediaries*, but **does not hide the sender's IP** and is a metadata-*reduction*, not
  elimination — receipt/timing side channels can statistically erode it (Martiny et al.,
  NDSS 2021). This is the default guarantee; it does not depend on the mixnet.
- **Recipient identity** — an always-on node receives by **push**, decoupling its network
  identity from the human identity; this removes the store-and-poll step PIR exists to protect
  (§6.4). This holds on the default `fast` tier as much as on the opt-in mixnet tier — it is a
  property of push delivery, not of onion routing.
- **Social graph & timing — NOT protected by default; opt-in mixnet only.** On the default
  `fast` tier the graph and timing are **observable** to a network-position or global adversary
  (§6.1). Hiding them requires the **opt-in, research-tier mixnet** (onion routing + Poisson
  mixing delays, so no node sees both ends and timing correlation is defeated) plus **cover
  traffic** and **size padding** — see [docs/research/mixnet.md](docs/research/mixnet.md) and the
  honest restatement in §6.1.
- **Discovery** — name→key lookups (§3.7) get the same treatment: private/unlinkable lookup is a
  property of routing them through the opt-in mixnet; on the default tier a lookup is observable
  to whoever the resolution path passes through.

## 6.3 Mixnet, cover traffic, padding (opt-in, research-tier — non-normative)

**This subsection describes the `private` tier's mixnet, which is research-tier and OPT-IN**
(§4.4, [docs/research/mixnet.md](docs/research/mixnet.md)) — it is not part of the default
guarantee (§6.1) and not conformance-tested. It applies only to an implementation that chooses to
offer, and a user who chooses to select, the `private` tier.

- **Onion packets** (Sphinx-format, constant length) traverse a path of mix nodes; each peels
  one layer.
- **Mixing delay** — each mix holds packets a randomized (Poisson) time and reorders, so input
  and output streams cannot be timing-correlated.
- **Cover traffic** — nodes emit loop and drop cover messages at a steady rate; an observer
  cannot distinguish real activity from cover. Rate is a **tunable knob** (higher rate → more
  privacy, more bandwidth).
- **Size padding** — MOTEs are padded to fixed size buckets so the *exact* length does not leak
  (an observer learns only which of the **two** size buckets — [docs/research/mixnet.md
  §4.4.1](docs/research/mixnet.md) — at most one bit per message).

The full mechanism — the Sphinx packet format, mix directory, path selection, key rotation,
cover-traffic rates, active-attack detection, entry guards, operator diversity, fail-closed
no-downgrade, and the high-security profile — is specified **verbatim, in full, as research-tier
non-normative material** in [docs/research/mixnet.md §4.4.1–§4.4.10](docs/research/mixnet.md);
this subsection is a summary of that opt-in layer, not a description of the default.

Email's asynchrony is the enabling property: minutes of latency are acceptable for the
`private` tier, and higher latency yields stronger anonymity.

## 6.4 Push delivery vs. PIR — what this design choice actually buys, and what it does not

PIR (Pung, Talek) exists to hide **which record a client reads from an untrusted shared
mailbox that holds everyone's mail** — a leak that only exists *because* those systems use a
polled central store to support offline recipients. DMTAP makes a different **design choice**
for the case it can: **an always-on recipient node receives by push** — over the default `fast`
tier's direct connections, or over the opt-in mixnet's onion-routed delivery if that tier is in
use (§4.6) — so for *that* node there is no untrusted shared store being queried and thus no
read-access-pattern to hide for its live traffic. This is a genuine architectural difference for
the always-on case, not a trick, and it does **not** depend on the mixnet: it is a property of
push vs. poll, and holds on the default tier as much as the opt-in one. (Note: Loopix itself
*does* use provider store-and-poll to serve offline clients; DMTAP's always-on-push is a
deliberate divergence, not an inherent property of mixnets.)

**This is not the whole picture, and an earlier revision of this section let it read as though it
were.** Push-through-the-mixnet is only a description of what happens while a recipient node is
actually online and receiving. A recipient that is not always reachable — a phone, a laptop,
anything that sleeps, closes, or loses connectivity, which is the **common** case, not the
exception — is served instead by the buffer role (§14.5): a peer or relay holds sealed
ciphertext, and the recipient's device **polls it on return**. A polled shared store that a
third party can observe is *exactly* the untrusted-store-access-pattern problem PIR was built to
solve, restated for a mailbox instead of a database. DMTAP does not make that problem disappear
by choosing push for the online case; it simply does not arise **for that traffic**. The
buffered/offline case is a separate, disclosed exposure, stated honestly at §6.6 item 17 rather
than folded into this section's "no untrusted shared store" claim, which held only for the
always-on path this paragraph actually describes.

**But push delivery is not "recipient anonymity, solved."** Three residual exposures remain, and
on the default `fast` tier the first is what a global observer actually gets for free — there is
no cover traffic to blur it. Item 1's requirement is **normative and owned here** (this clause,
not the honest-limits prose of §6.6, is their home; §6.6 and §6.9 point back to it) and applies
**regardless of tier**; items 2–3 are the opt-in mixnet's mitigations for the same exposures and
apply **only when that research-tier layer is in use**:

1. **Last-hop / receipt visibility (normative, all tiers).** A global observer of the recipient's
   link — the final mix on the opt-in `private` tier, or simply the recipient's link on the
   default `fast` tier — learns that *a packet was delivered to node X*, and on `fast` also learns
   *when* and *how much*: there is no cover traffic on the default tier to blur this. An always-on
   node has a **stable, targetable network presence** — itself a fingerprint — regardless of tier.
   The node's network identity MUST be cryptographically and durably **decoupled** from the human
   identity (pseudonymous peer id, no re-identifying metadata); this is the one mitigation that
   applies by default.
2. **Receipt-timing — opt-in mixnet only.** Without cover on the delivery link, observing the
   node reveals *when* messages arrive; on the default `fast` tier this is simply the disclosed
   residual of item 1, unmitigated. A recipient node that has opted into the research-tier
   `private` mixnet MUST receive a steady **Poisson cover stream** so real receipts are
   indistinguishable from cover (as Loopix does with loop + link cover) —
   [docs/research/mixnet.md §4.4.5](docs/research/mixnet.md).
3. **Long-term intersection / statistical-disclosure attacks — opt-in mixnet only.** Persistent
   presence is exposed to correlation across many rounds on any tier; on the default `fast` tier
   nothing bounds this beyond sealed sender (§6.2). Cover traffic bounds but does not eliminate
   this for a node that has opted into the mixnet (cf. Vuvuzela's differential-privacy noise,
   which degrades over volume) — the bounding mechanisms (entry guards, operator diversity, cover
   rates) are research-tier, non-normative, and specified in [docs/research/mixnet.md
   §4.4.5, §4.4.8](docs/research/mixnet.md).

**Offline buffering — the honest state, not the comfortable one.** If a node is down, a peer/relay
holds sealed ciphertext (§14.5) for later pickup. **A prior revision of this paragraph described
that pickup as happening "via an unlinkable dead-drop token over the mixnet."** That phrase
appeared nowhere else in this specification — no object, no derivation, no verifier, no state
machine — and made a stronger claim than anything actually specified. It is removed. What is
actually specified is a **polled, content-blind shared buffer** (§14.5): the recipient's device
retrieves by its per-recipient delivery tag when it returns online, and the buffer holder (or
anyone observing it) sees that tag polled, at that time, moving that much ciphertext. §6.6 item 17
states this residual in full; DMTAP neither runs PIR against the buffer nor specifies a blinded
retrieval ceremony that would remove it — this is the priced-in cost of not paying for either.

## 6.5 Privacy tiers (and where each product sits)

| Tier | Path | Latency | Graph privacy | Status | Default for |
|------|------|---------|---------------|--------|-------------|
| `fast` | direct / few-hop | sub-second | sealed sender vs. intermediaries; graph observable to a network/global adversary (§6.1) | **normative, default** | mail, all control MOTEs, live chat, **normal-size files**, **large-file bulk** |
| `private` | full mixnet + cover | minutes | additionally hides the graph from a global passive adversary, subject to §4.4.11's honest low-adoption model | **research-tier, OPT-IN** — [docs/research/mixnet.md](docs/research/mixnet.md) | nothing by default; a deliberate, user-surfaced choice |

- **Default is `fast`.** `private` is opt-in, for implementations that choose to offer the
  research-tier mixnet. Because *choosing* `private` is itself a signal, a client that offers it
  SHOULD make the choice deliberate and user-surfaced.
- **Files:** the control MOTE follows the same default-`fast`/opt-in-`private` rule as any other
  control MOTE (§4.6). **Normal-size files route through whichever tier the message itself
  uses** — the default `fast` tier's guarantee, or the opt-in mixnet's private-tier metadata
  privacy if a sender has selected it (the §6.6 residuals apply either way). **Large-file bulk**
  uses `fast` — but MUST
  be **onion-routed (a few hops, Tor-style) + size-padded to buckets + swarmed from multiple
  holders**, accepting bandwidth cost. **This bulk tier is explicitly weaker:** like Tor, it
  provides relationship anonymity against *local/partial* adversaries but is **vulnerable to
  end-to-end traffic correlation** by an adversary observing both endpoints (Murdoch–Danezis,
  2005). Swarming and padding raise cost; they do not guarantee anonymity. The strong
  guarantee is the *messaging* tier; moving large sensitive files inherits Tor's correlation
  exposure. See §5.5, §4.5.

## 6.6 Honest boundaries (what we do NOT claim)

1. **Global active adversary — undefended by default; the irreducible residual *after* the
   mechanisms of the opt-in mixnet.** On the **default `fast` tier there is no active-adversary
   defense at all** — no replay cache, no cover traffic, no entry guards; a global active
   adversary correlates trivially, and this is simply the cost of the default tier's honest scope
   (§6.1). The rest of this item describes what the **opt-in, research-tier mixnet**
   ([docs/research/mixnet.md](docs/research/mixnet.md)) adds for an implementation and user that
   choose it, and none of it is a default guarantee: **per-epoch mix replay caches** (drop
   replayed packets), **tagging-resistant integrity-protected Sphinx headers**, **memoryless
   Poisson mixing** (timing correlation stays hard even under active delay injection),
   **loop-cover active-attack detection** that turns drop/delay/flooding from undetectable into
   **detected → rotate + `HALT_ALERT` + fail-closed**, **entry guards + `≥ 3`-disjoint-operator
   path diversity** bounding long-term intersection, **attested (non-token) mix identities** for
   Sybil resistance, and a **no-silent-downgrade rule**
   ([docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)) so DoSing the mixnet cannot strip
   the tier for a user who selected it. A **user-selectable high-security profile**
   ([docs/research/mixnet.md §4.4.10](docs/research/mixnet.md) — 5 hops, longer delays,
   constant-rate cover, tighter guards/diversity) is the lever to climb toward the bound, for a
   user who has already opted into the mixnet.
   **After all of that**, an irreducible residual remains **for the opt-in tier**: by the
   **Anonymity Trilemma** (Das, Meiser, Mohammadi & Kate, *"Anonymity Trilemma: Strong Anonymity,
   Low Bandwidth Overhead, Low Latency — Choose Two,"* IEEE S&P 2018), strong anonymity provably
   **cannot** be had without paying in latency and/or bandwidth, so a global *active* adversary
   with unlimited resources that controls a large enough fraction of the mix fleet can still mount
   long-term statistical disclosure at *some* latency/cost point. The mixnet **approaches** that
   mathematical floor as the high-security profile's latency/overhead grows; it does **not**
   pretend to defeat an omnipotent adversary at zero cost, and none of this applies at all to the
   default `fast` tier, which makes no such attempt. This is the honest floor *after* the maximal
   opt-in defense, not a substitute for it.
   One concrete edge of that residual: **sub-threshold selective dropping** — an adversary dropping
   fewer than the loop-loss detection threshold (< 20%, §16.3) of a target's packets — stays under
   the [docs/research/mixnet.md §4.4.7](docs/research/mixnet.md) detector, so it is **bounded, not
   eliminated**; the trade is that so little is dropped it accomplishes little, and the
   **High-security profile** (faster loops, [docs/research/mixnet.md
   §4.4.10](docs/research/mixnet.md)) tightens the detectable floor further.
   One claim is prohibited outright (normative, owned here, and binding regardless of tier): an
   implementation or operator MUST NOT claim that raising hop count alone defends against
   colluding entry+exit mixes. Hops buy timing-correlation resistance against *observers*; entry
   guards + attested operator diversity bound *endpoint collusion* (≈ *f*²,
   [docs/research/mixnet.md §4.4.8, §4.4.10](docs/research/mixnet.md)) — and neither substitutes
   for the other. The measured basis is reported informatively, as a research-tier mechanism-model
   simulation, in §6.10.
2. **Large-file bulk metadata.** Onion-routing + padding + swarming makes it *strong*, not
   *free* and not *perfect* — the fact and approximate volume of a large transfer may remain
   partially observable at high adversary capability.
3. **Endpoint compromise — the irreducible residual *after* the mechanisms.** DMTAP does not
   treat endpoint compromise as an unaddressed limit; it specifies concrete, normative defenses
   that shrink the blast radius to a single floor:
   - **Offline seizure is defended.** The local MOTE store is encrypted with a key **released
     only on device unlock** (biometric/PIN, §6.7), so a powered-off / locked stolen phone yields
     **nothing** — the ciphertext is inert without the unlock secret.
   - **Key exfiltration is defended.** `IK`/device keys **MUST** live in a hardware keystore as
     **non-exportable** keys where the platform provides one (Secure Enclave / TPM / StrongBox /
     TEE, §1.2a), and SHOULD use the strongest available key-protection class otherwise, so even
     a software compromise cannot *copy* the key out to sign or decrypt elsewhere — it can only
     *use* it locally while the device is unlocked, and only until revoked.
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
   clients MUST treat a single unaudited log as advisory (§3.5.1, §3.4.1), leaning on OOB
   verification for high-value contacts. This is a real tension with the sovereignty goal and is
   stated as such. For **DMTAP-Auth (§13)** specifically, a split view is a **silent per-RP
   account takeover**, so high-value login RPs MUST require **multi-log consistency or an
   OOB-verified pin even in v0** — the owning requirement is §13.7 item 6.
7. **Group handshake ordering is a metadata concentration point (relevant when the opt-in mixnet
   is used).** The per-group committer/ordering channel (§5.1) necessarily sees all of a group's
   handshake traffic; this is an explicit exception to the "no single node sees both ends"
   framing. For an implementation/user that has opted into the research-tier `private` mixnet,
   this was intended to be bounded by rotating the committer and keeping application traffic on
   the mixnet. **Additionally, in v0 the ordered, reliable handshake channel is not itself required
   to ride the mixnet** (§5.1 routes it for liveness), so a *network* observer — not only the
   committer — can see the timing of membership changes (Add/Remove Commits, Welcomes), which is
   membership-graph metadata the opt-in mixnet otherwise hides *for those who use it*; on the
   default `fast` tier this metadata is observable regardless, consistent with §6.1. Application
   traffic on the mixnet stays cover-indistinguishable; the handshake channel does not. Closing
   this fully (carrying handshakes over the `private` tier, realizing "ordered/reliable" as the
   committer's on-arrival log order rather than an in-transit bypass, and bringing any retained
   low-latency committer path under the no-silent-downgrade rule,
   [docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)) is a tracked residual of that
   opt-in layer.
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
   Because a per-arrival wake is itself an activity-dependent timing signal *outside* any cover
   traffic, it composes with item 1: under the opt-in research-tier **High-security profile**
   ([docs/research/mixnet.md §4.4.10](docs/research/mixnet.md)) wake jitter/batching is therefore
   a **MUST for an implementation offering that profile** (§4.9.1), so the wake path does not
   reintroduce the recipient-arrival timing channel the profile's constant-rate cover closes.
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
    serving sealed chunks does not — hence public-object serving is a **per-operator opt-in**,
    never a default-on behavior of a conformant node; the owning requirement is §22, negotiated
    as the `pub-1` capability (§10.2).
13. **The opt-in mixnet's volunteer infrastructure may simply never materialise — and that is now
    the honest starting assumption, not a fallback case.** DMTAP has no commercial engine behind
    its infrastructure: the KT log set and the rendezvous layer are roles taken reciprocally by
    participants (§0.2.2, §14.1) that the `fast`-tier default depends on, and they are load-bearing
    regardless of the mixnet. The **mixnet fleet** specifically (mix nodes, the research-tier
    `private` path, [docs/research/mixnet.md](docs/research/mixnet.md)) is an *additional*,
    opt-in role with its own default-on proposal for always-on public nodes that choose to
    implement it ([docs/research/mixnet.md §4.4.2a](docs/research/mixnet.md)) — but because that
    layer is research-tier, unshipped, and untested at scale, DMTAP does not assume it exists: the
    default (`fast`) does not depend on it, `private`-tier paths are simply **unbuilt** unless and
    until an implementation ships the mix role and enough operators run it, and a client MUST NOT
    present `private` as available before that is true. If it never reaches useful scale, nothing
    about the default guarantee (§6.1) regresses — it was never conditioned on the mixnet
    materialising. What *is* lost is upside: the honest-low-adoption model
    ([docs/research/mixnet.md §4.4.11](docs/research/mixnet.md)) and the **Bootstrap profile**
    ([docs/research/mixnet.md §4.4.10a](docs/research/mixnet.md), which carries no anonymity
    claim) describe what a small fleet buys *if and when the layer ships*, not a guarantee this
    specification makes about the present. This is the price of having nobody paid to keep an
    optional layer's lights on, stated up front instead of discovered later.
14. **A third-party gateway reads your inbound legacy mail in the clear.** The legacy leg is
    plaintext by construction (§7.10.4), and a gateway serving legacy *clients* must decrypt the
    mailbox to speak IMAP/POP/DAV at all (§7.15.3). Now that "operator" means **whoever chose to
    run the role** (§0.2.3) rather than a vetted commercial provider, this deserves sharpening: the
    party reading that mail is an ordinary volunteer or acquaintance with a reputable IP, subject to
    whatever jurisdiction and whatever operational hygiene they happen to have. DMTAP's mitigations
    are real but bounded — the mode is **declared and enforced** (§7.15.4), gateway-touched mail is
    **marked** to the recipient (§7.8), and the native path never touches a gateway at all (§7.7).
    **Recommendation:** where legacy privacy matters, run the gateway role **yourself** (`private`
    mode, §7.15.4, §7.1a) so the only party who can read it is you. Where you cannot, treat a
    third-party gateway exactly as you would treat any hosted mail provider — because that is what
    it is.
15. **A legacy sender cannot verify that a gateway resolved a chain name honestly.** When mail is
    addressed to `alice.sol@gw.example.net` (§7.10.5a), the gateway performs the SNS/ENS lookup and
    the legacy sender has **no way to check the result**: legacy SMTP carries no cryptography with
    which to bind "the key this name resolved to" to anything the sender can verify, and DMARC/DKIM
    authenticate the *sending* domain, never the *recipient* resolution. A malicious or compromised
    gateway could therefore route mail addressed to a chain name to a key of its choosing, and the
    legacy sender would see a perfectly normal delivery. This is **inherent to legacy having no
    cryptography**, not a defect introduced by the mapping — the same exposure exists for every
    alias form and, in truth, for ordinary MX-based mail routing today. It is bounded on the DMTAP
    side: a *native* sender resolves the chain name itself and verifies the binding bidirectionally
    and against KT (§3.12.5(b), §3.12.3), so the exposure exists only for correspondents who are
    still on legacy — and shrinks as they leave (§7.1c).

16. **A key-name identifies an identity; it does not, on its own, address one.** The key-name is a
    **one-way** digest of the identity key — 80 bits of `BLAKE3-256(ik)` (§18.9.17, §3.9.6). Given
    the key you can compute and confirm the name; **given only the name you cannot recover the
    key**, and the key is what every addressing primitive actually consumes: the HPKE seal (§2.4),
    the `DeliveryTag` (§2.2a), a work proof's `recipient` scope (§9.4), and the DHT lookup key
    `hash(ik)` itself (§4.2). A stranger holding only a spoken or printed key-name therefore holds
    a **checksum, not an address**, and must obtain the key out of band — QR, contact card,
    introduction (§3.13.5).
    The only mechanism that could close this for an identity with *no* domain, *no* chain and *no*
    contacts is a DHT lookup keyed on `hash(ik)`. This specification deliberately relegates the
    DHT to **opportunistic use only** and states that no relationship should depend on it (§4.2.1),
    because §4.2's CAUTION identifies it as the single most attackable surface in the protocol and
    notes that IP-diversity caps are close to worthless under IPv6. So for the cold-start,
    key-name-only user of §3.13.5, **first contact is DHT-dependent and therefore eclipse-deniable**:
    an adversary who eclipses the neighbourhood of `hash(ik)` can make that identity unfindable to
    strangers while it remains perfectly nameable, verifiable, and deliverable-to once found.
    §9.7a's floor guarantees **acceptance**, not **reachability**; §3.13.4's guarantee is therefore
    stated as reachable *by anyone who has your key*, and that qualifier is the honest one. The
    practical mitigation is the one §3.13.5 already recommends for its own reasons: out-of-band
    introduction is the primary first-contact path, and it carries the key rather than only its
    digest.
17. **Offline/buffered recipients are served by the polled shared store PIR exists to remove —
    and DMTAP does not remove it.** §6.4's push-delivery argument holds only for an **always-on**
    recipient node. Any recipient that is not continuously reachable — a phone, a laptop, anything
    that sleeps or loses connectivity, which is the **common** case, not the exception — is served
    instead by the **content-blind relay-mailbox / peer buffer** of §14.5: ciphertext is held there
    for the offline-buffer TTL (§16.6) and **retrieved by the recipient polling it on return**.
    This is, precisely, the untrusted-shared-store exposure PIR was designed to remove, restated
    for a mailbox instead of a database: a party able to observe the buffer learns **which
    per-recipient delivery tag was polled, when, and how much ciphertext moved** — arrival timing
    and volume — every time a buffered device comes back online. Content stays sealed (the buffer
    is content-blind, §14.5) but the **access pattern** is exactly what PIR exists to hide, and
    DMTAP does not hide it: it neither runs PIR against the buffer nor specifies an unlinkable
    retrieval ceremony that would. An earlier revision of §6.4 described retrieval as happening
    "via an unlinkable dead-drop token over the mixnet" — a phrase defined nowhere in this
    specification (no object, no derivation, no state machine) — and that claim is withdrawn as
    false comfort, not merely unspecified. Closing this residual for real is a materially larger
    undertaking than the always-on push path this specification actually builds: either running
    PIR against the buffer (the cost §6.4 declines to pay) or fully specifying a blinded dead-drop
    protocol — its own unlinkable-pickup credential, a defined wire object, and a retrieval state
    machine, none of which exist today. Neither is done here. §14.5 owns the buffer mechanism and
    MUST NOT describe retrieval as anonymous or unlinkable until one of these is actually
    specified; until then, a buffered/offline recipient's arrival-timing and volume metadata is a
    disclosed gap in the metadata-privacy guarantee (§6.1), not a covered case of it.
18. **A vouch (§9.7) discloses three identities in cleartext to buy the strongest cold-contact
    standing in the protocol — a structural exception, not a bug.** Every other cold-contact
    mechanism (ARC, PoW, postage, §9.3–§9.5) is evaluated on the sealed envelope without revealing
    who anyone is. A `Vouch` (§18.3.3) cannot follow that pattern: it *is* the voucher's, the
    subject's (the sender's), and the recipient's identity keys plus a signed introduction, and
    §9.2a requires it to ride the cleartext envelope so the recipient's gate can check it before
    decryption (§2.7 step 6). Every exit mix and any on-path observer of that hop reads all three
    identities on any first-contact MOTE that presents one. This is not the theft/replay hole
    §9.2a and §2.7 step 8(b2) close (a different, narrower defect, since fixed) — it is what the
    mechanism discloses exactly as designed, exactly when the sender presenting it most needed
    protection. §9.1 principle 4 and §9.7's own honest-limit clause state this precisely; §6.9
    SP-11 and SP-5 state its effect on those two claims. Closing it requires restructuring §2.7
    around a decrypt-then-gate path for the vouch case, or replacing the vouch with a blinded
    ARC-style presentation; this specification discloses the exposure rather than choosing between
    those fixes.

DMTAP states these boundaries in-product. Honest, disclosed limits beat a false "perfectly
anonymous."

## 6.7 Data at rest

- The node encrypts the mailbox, file blobs, and keys **at rest** under a device/identity key.
- **Unlock-gated store encryption (normative, MUST).** The at-rest key MUST be **released
  only on device unlock** — wrapped by a key the hardware keystore (§1.2a) yields only after a
  successful biometric/PIN authentication, and **evicted from memory on relock/timeout** (§16.9).
  The sole exception is a platform with **no unlock signal** (e.g. a headless always-on box with
  no lock/unlock concept), which MUST instead seal the at-rest key to the strongest available
  boot-time protection (keystore-sealed / full-disk) — the relock eviction it cannot observe
  does not apply, and everything else here does. This distinguishes two threat cases sharply: an
  **offline-seized** device (powered off or locked) yields only inert ciphertext (**defended**);
  a **live, unlocked, in-use** device can read what the user reads (**the residual**, §6.6
  item 3). Implementations MUST NOT keep the at-rest key resident indefinitely across a locked
  device.
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
(§6.2) or the opt-in mixnet's anonymity property when that tier is in use (§4.4,
[docs/research/mixnet.md](docs/research/mixnet.md)):

- **Provenance answers "which trust boundaries did this cross?", never "which nodes carried it?"**
  For the `private` tier the recipient learns only the **profile floor** the path satisfied
  (`≥ 3` / `≥ 5` hops, [docs/research/mixnet.md §4.4.10](docs/research/mixnet.md)) — **never** a
  mix-node identity, address, exact hop count, path
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
adversary reach does not confer, **and this holds on the default `fast` tier, with no mixnet
required.** **The metadata properties (SP-3–SP-5) are, by contrast, tier-dependent.** On the
**default `fast` tier**, sender identity is reduced (sealed sender vs. intermediaries) but the
graph and timing are observable to a global passive adversary — no property here claims otherwise
for that tier. **Strong graph/timing privacy against a global passive adversary, and any bound at
all against a global active one, is available only via the opt-in, research-tier mixnet**
(§4.4, [docs/research/mixnet.md](docs/research/mixnet.md)) and is stated as such in SP-3/SP-4
below, which are themselves marked research-tier, not default guarantees (§6.6 item 1). This
tier-dependence is the honest core of DMTAP's posture.

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
  exposed to a future quantum break until the identity migrates (§5.1). This is distinct from —
  and unconditioned on — the opt-in mixnet's own, separate harvest-now exposure at its routing
  layer ([docs/research/mixnet.md §4.4.12](docs/research/mixnet.md)), which only arises for a
  sender who has opted into the `private` tier.

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

**SP-3 — Sender anonymity against a GLOBAL PASSIVE adversary — NON-NORMATIVE, RESEARCH-TIER,
scoped to the opt-in `private` mixnet only. Demoted from its former "headline property" status.**
- *Claim (research-tier, not a default guarantee):* **IF** a sender and recipient have both opted
  into the research-tier `private` mixnet ([docs/research/mixnet.md](docs/research/mixnet.md)),
  **THEN**, subject to the disclosed residuals below, a global passive adversary observing all
  links and all timing is intended not to learn who sent that MOTE to whom. This claim is **not
  conformance-tested**, is **not** part of any default guarantee, and does not hold at all for the
  default `fast` tier — on `fast`, sealed sender (§6.2) reduces what intermediaries learn but a
  global passive adversary recovers the graph via IP + timing + volume correlation (§6.1). **The
  honest default, restated:** sealed sender is a metadata *reduction* against intermediaries, not
  global-passive-adversary immunity; graph/timing privacy against that adversary is the opt-in
  mixnet, not a default property of DMTAP.
- *Would hold by (research-tier mechanism, if implemented and selected):* sealed sender (identity
  + authenticating signature sealed inside the payload, §2.2, §6.2) — this part **is** default and
  normative; plus, only within the opt-in mixnet: Sphinx onion routing + memoryless Poisson mixing
  ([docs/research/mixnet.md §4.4.1, §4.4.6](docs/research/mixnet.md)); mandatory cover traffic +
  size-bucket padding ([docs/research/mixnet.md §4.4.5](docs/research/mixnet.md), §6.3); and
  mixnet-routed lookups (§3.7, opt-in).
- *Against:* the global **passive** adversary, and only within the scope of the opt-in mixnet.
- *Residual:* metadata *reduction*, not elimination, even within the opt-in tier — sealed sender
  is statistically eroded by receipt/timing side channels (Martiny et al., NDSS 2021), which is
  why cover traffic is load-bearing, not optional, *for implementations that offer the mixnet*
  (§6.2); last-hop delivery observability remains (§6.4 items 1–3); long-term intersection is
  bounded, not eliminated (§6.6 item 5, [docs/research/mixnet.md
  §4.4.8](docs/research/mixnet.md)); while the fleet is small the guarantee is
  Tor-with-few-relays, not a strong global mixnet ([docs/research/mixnet.md
  §4.4.11](docs/research/mixnet.md)); the v0 onion is not PQ — a harvest-now adversary could
  retroactively deanonymize the recorded social graph ([docs/research/mixnet.md
  §4.4.12](docs/research/mixnet.md)). **Above all: none of this is available unless an
  implementation ships the mixnet and both parties opt in** — that is the residual that matters
  most, and it did not exist when this property was the stated default.

**SP-4 — Sender anonymity against a GLOBAL ACTIVE adversary — NON-NORMATIVE, RESEARCH-TIER, scoped
to the opt-in `private` mixnet only.**
- *Claim (research-tier, not a default guarantee):* **IF** the opt-in `private` mixnet is in use,
  a global *active* adversary (inject/drop/delay at will) is intended to be forced to pay latency
  and/or bandwidth to correlate, with its drop/delay/flooding **detected and responded to** — but
  **not fully defeated**. On the default `fast` tier there is **no** active-adversary defense of
  any kind (§6.6 item 1) — this claim does not apply there at all.
- *Would hold by (research-tier mechanism):* per-epoch replay caches, tagging-resistant Sphinx
  (header MAC `γ` + wide-block LIONESS payload PRP), memoryless Poisson mixing
  ([docs/research/mixnet.md §4.4.6](docs/research/mixnet.md)); loop-cover active-attack detection
  → rotate + `HALT_ALERT` + fail-closed ([docs/research/mixnet.md
  §4.4.7](docs/research/mixnet.md)); entry guards + **attested** operator-diversity
  ([docs/research/mixnet.md §4.4.8](docs/research/mixnet.md)); fail-closed no-downgrade
  ([docs/research/mixnet.md §4.4.9](docs/research/mixnet.md)); the user-selectable High-security
  profile ([docs/research/mixnet.md §4.4.10](docs/research/mixnet.md)) — all of it research-tier
  and opt-in.
- *Against:* the global **active** adversary, only within the scope of the opt-in mixnet.
- *Residual (bounded, not eliminated, and entirely conditional on opting in):* the Anonymity
  Trilemma floor (§6.6 item 1, Das et al., IEEE S&P 2018) — strong anonymity provably cannot be
  free; sub-threshold selective dropping stays under the [docs/research/mixnet.md
  §4.4.7](docs/research/mixnet.md) detector (§6.6 item 1, §16.3); the colluding entry+exit floor
  is ≈ *f*² and is **not** reduced by adding hops ([docs/research/mixnet.md
  §4.4.10](docs/research/mixnet.md)). The mixnet *approaches* the mathematical floor as the
  profile's latency/overhead grows; it does not claim to defeat an omnipotent active adversary, and
  none of it applies to a party that has not opted in. The mechanism-model simulation
  corroborating this shape (chance-floor convergence, 20%-loss detection, ≈ *f*² collusion,
  hops-≠-collusion-defense) is reported — with its honest caveats, as research-tier corroboration,
  not deployment evidence — in §6.10.

**SP-5 — Recipient unlinkability (blinded delivery tags) — scoped to established contact.**
- *Claim:* For an **established contact** (steady-state delivery, `to = BlindedTag`), an observer
  cannot link successive `private`-tier deliveries to the same recipient *by the routing tag*, nor
  tie that tag to the recipient's persistent identity key. **This claim does not extend to a
  first-contact/cold delivery**, which is a distinct case addressed in the residual below, not a
  weaker instance of this one.
- *Holds by:* blinded delivery tag `BT = HKDF(shared_secret, epoch_day)`, recognized by the
  recipient but unlinkable across time and across observers (§2.2a); network/human identity
  decoupling and recipient-side cover (§6.4).
- *Against:* a global passive adversary / final mix, for the *tag-linkage* property, on the
  established-contact path only.
- *Residual (stated, not overclaimed):* blinding removes the persistent-key linkage in the envelope;
  it does **not** hide *that a packet was delivered to a particular always-on node* — a stable
  network presence is itself a fingerprint (§6.4 item 1, §2.2a). Recipient-side cover
  ([docs/research/mixnet.md §4.4.5](docs/research/mixnet.md), opt-in mixnet only) blurs receipt
  *timing* for a recipient who has opted in, but does not erase last-hop observability, and on the
  default `fast` tier there is no cover at all (§6.4 item 2). Implementations MUST NOT present
  blinded tags as full recipient anonymity (§2.2a). This is a *reduction*, not a solved property.
  **On the cold/first-contact path, DMTAP makes no recipient-unlinkability claim at all, and the
  gap is total, not partial:** a cold envelope's `to` is `KeyTag` — the recipient's own identity
  key, in cleartext, by construction (§2.2a) — and the anti-abuse `challenge` it carries
  independently names the recipient in cleartext too: `ArcToken.origin` (MUST match the verifying
  node's origin), `PostageStamp.audience`, and `PowSolution`'s recipient-bound epoch scope
  (`id ‖ recipient ‖ nonce(epoch)`, §9.2a, §9.4) each identify the recipient to whichever exit mix
  and on-path observer see the unwrapped envelope. These fields are redundant with the already-
  disclosed `KeyTag` on the cold path today, but they mean that blinding `to` alone would not, by
  itself, buy cold-path recipient unlinkability even if a future revision found a way to avoid the
  `KeyTag` requirement — the challenge fields would still leak the recipient independently. A
  `Vouch` (§9.7) additionally leaks the *sender's* and the voucher's identity on the same cold
  message; that is §6.6 item 18 and §6.9 SP-11's residual, not this claim's.

**SP-6 — Forward secrecy & post-compromise security — scoped to an established session.**
- *Claim:* Once an MLS session (group or 2-party) exists, compromise of a party's current keys
  does not expose its past messages (FS), and after a Commit/epoch advance an evicted key cannot
  decrypt future messages (PCS). **This claim applies only from the point a session's first epoch
  exists.** It does not apply, and is not weakened-but-still-partially-true for, a MOTE sent before
  one does — that case is a distinct, disclosed gap stated in the residual below, not a bounded
  degradation of this claim.
- *Holds by:* MLS TreeKEM epoch advancement (§5.1, §5.2); for a party that has opted into the
  research-tier mixnet, mix-key epoch rotation + deletion at `valid_until` additionally gives
  forward secrecy against later mix-node seizure
  ([docs/research/mixnet.md §4.4.4](docs/research/mixnet.md), opt-in only); per-file keys for
  at-rest blobs (§6.7); the optional deniable mode adds *per-message* FS/PCS via the Double
  Ratchet (§5.2.1(b)).
- *Against:* an adversary that later seizes a device (FS; a mix, additionally, only within the
  opt-in mixnet), and a bounded-duration endpoint
  compromise that is subsequently revoked (PCS) — for traffic carried inside an established
  session.
- *Residual:* MLS PCS is **per-epoch/per-Commit**, coarser than Double-Ratchet per-message healing —
  a deliberate simplicity trade, not a strict improvement (§5.2); persistent files cannot ratchet
  (§6.7); deniable-mode PCS needs *bidirectional* traffic — a send-only channel retains only FS
  (§5.2.1(b)); PCS heals only *after* the evict/rotate flow runs (§6.7).
  **The default 1:1/first-contact path has no forward secrecy at all, stated plainly.** A
  first-contact (or otherwise pre-session) MOTE is HPKE-sealed directly to the recipient's
  identity/KeyPackage key, with `Envelope.epoch` absent (§2.2) — a single HPKE seal, not a
  ratchet. Compromise of that recipient key at **any later time** exposes that message; there is
  no epoch to have advanced past and no key deletion schedule that protects it. This is not a
  weaker version of the residuals above; it is the **entire claim not yet applying**, for exactly
  the messages a cold or new contact sends before any Commit/Welcome establishes a first epoch
  (§5.3). The optional deniable 1:1 mode (SP-7) is the one path that supplies per-message FS/PCS
  from its first message, because its Double Ratchet does not wait on an MLS epoch; a sender who
  needs FS before a session exists and does not want deniability has no other option in this
  specification today. (An earlier revision carried an undefined `fs_ratchet` field that could be
  misread as filling this gap; it supplied no derivation, no verifier, and no semantics anywhere
  in the corpus, and has been removed — see §2.4's note — rather than left to imply a mechanism
  that was never specified.)

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
  ratchet (§1.3, `0x020F`); for a party that has opted into a stronger tier/profile, fail-closed
  no silent `private → fast` and no silent High-security → Standard downgrade
  ([docs/research/mixnet.md §4.4.9](docs/research/mixnet.md), `0x0310`); monotonic capability
  announcements (§10.2, `0x030A`); deniable-mode no-silent-downgrade (§5.2.1(d), `0x040E`). These
  rules are collected and audited as a **set** in §10.7.
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

**SP-11 — Anti-abuse without deanonymization — scoped to ARC, PoW, and postage; the vouch is
excluded, not covered.**
- *Claim:* A recipient can rate-limit and block a cold sender presenting an **ARC token, PoW
  solution, or postage stamp** (§9.3–§9.5) **without learning who they are** and **without linking
  a sender across recipients**. **This claim does not extend to the vouch (§9.7).** Presenting a
  vouch is a deliberate, disclosed exception, not a weaker instance of the same guarantee: it
  reveals the sender's, the voucher's, and the recipient's identity keys in cleartext to the
  mixnet's exit hop and any on-path observer of that hop (§9.2a, §18.3.3) — the opposite of what
  this claim states for the other three mechanisms. An implementation MUST NOT read this claim as
  covering vouch-based cold contact.
- *Holds by (for ARC/PoW/postage):* ARC anonymous, per-origin-scoped, rate-limited tokens (§9.3),
  PoW fallback (§9.4), and postage (§9.5), all evaluated on the sealed envelope **before
  decryption** (§2.7 step 6), each bound to the ephemeral `sender_key` to defeat proof theft/replay
  (§9.2a); the anonymity-preserved principle (§9.1 item 4), which these three mechanisms satisfy
  without exception.
- *Against:* a recipient (and its operator) attempting to deanonymize or cross-link a sender using
  ARC, PoW, or postage.
- *Residual:* repeat abuse is bounded at the **issuer** layer, not the recipient layer —
  recipient-side cross-recipient linking is deliberately unavailable and not claimed (§9.3.2); an
  unvetted/self-issued token carries a **zero** budget (§9.3.1); a hidden-membership list's committer
  holds a disclosed per-delivery vouch-trust power (§9.9). Mix-layer flooding is bounded only by
  content-blind controls (per-connection/operator rate limits + attested operator diversity, §9.8).
  **The vouch's exposure is not a residual of this claim — it is a carve-out from it.** §9.1
  principle 4 states the exception normatively; §9.7's honest-limit clause and §6.6 item 18 state
  the mechanism and its consequence in full: every exit mix and on-path observer of the final hop
  reads the voucher's, subject's, and recipient's identity keys on any first-contact MOTE
  presenting a vouch. Implementations MUST disclose this to a sender before offering vouch as a
  cold-contact option (§9.7) and MUST NOT market or default to vouch as though it carried the same
  anonymity property as ARC, PoW, or postage.

### 6.9.1 The map, in one table

| # | Property | Status | Holds by (§) | Against | Residual (§) |
|---|----------|--------|--------------|---------|--------------|
| SP-1 | Message confidentiality (E2E) | normative, default | §2.4, §5.1 | passive **+ active** | endpoint floor §6.6 item 3; gateway leg §7; first-contact §6.6 item 4; harvest-now §5.1 |
| SP-2 | Content authenticity / integrity | normative, default | §2.7 step 8, §2.2, §18.9 | passive **+ active** | binding = KT profile SP-9 / §6.6 item 6; deniable = MAC not sig §5.2.1(c) |
| SP-3 | Sender anonymity vs global **passive** | **research-tier, OPT-IN mixnet only — not default** | §2.2 (sealed sender, default); [docs/research/mixnet.md §4.4.5–6](docs/research/mixnet.md), §6.3, §3.7 (opt-in) | global passive, **only within the opt-in mixnet** | reduction not elimination §6.2, §6.4, §6.6 item 5, [docs/research/mixnet.md §4.4.11–12](docs/research/mixnet.md); **no claim at all on the default `fast` tier** |
| SP-4 | Sender anonymity vs global **active** | **research-tier, OPT-IN mixnet only — not default** | [docs/research/mixnet.md §4.4.6–§4.4.10](docs/research/mixnet.md) | global active, **only within the opt-in mixnet** | Trilemma floor §6.6 item 1; sub-threshold drop §16.3; **no defense at all on the default `fast` tier** |
| SP-5 | Recipient unlinkability (blinded tags) — **established contact only** | normative, default (mixnet-only mitigation is opt-in) | §2.2a, §6.4 | passive / final mix | last-hop observability remains §6.4 item 1, §2.2a; **cold path: no claim at all** — `to`=`KeyTag` + challenge fields (`origin`/`audience`/`epoch_nonce`) leak recipient §6.6 item 18 |
| SP-6 | Forward secrecy + PCS — **established session only** | normative, default (mix-key FS is opt-in mixnet extra) | §5.2, [docs/research/mixnet.md §4.4.4](docs/research/mixnet.md), §6.7, §5.2.1(b) | later seizure / revoked compromise | per-epoch coarser §5.2; send-only = FS only §5.2.1(b); **pre-session HPKE MOTEs: no FS at all**, `epoch` absent, `fs_ratchet` removed as undefined |
| SP-7 | Deniability / repudiation (1:1 opt-in) | normative, default | §5.2.1 | cryptographic-transcript judge | default non-repudiable §5.2; not endpoint §6.6 item 3; X3DH online bound §5.2.1(e) |
| SP-8 | Downgrade resistance | normative, default | §1.3, [docs/research/mixnet.md §4.4.9](docs/research/mixnet.md), §10.2, §10.1 | active DoS / MITM | weakest-link §6.6 item 5 (full set §10.7) |
| SP-9 | KT / equivocation detection | normative, default | §3.5.2 (v1) / §3.5.1 (v0) | malicious KT log | **v0 not equivocation-proof §6.6 item 6** |
| SP-10 | Recoverability from compromise | normative, default | §1.4, §1.5, §6.7, §13.4 | loss / partial compromise | `IK`+quorum takeover; content needs backup §1.4 |
| SP-11 | Anti-abuse without deanonymization — **ARC/PoW/postage only; vouch excluded** | normative, default | §9.3–§9.5, §2.7 step 6, §9.2a | deanonymizing recipient/operator | issuer-layer only §9.3.2; committer vouch-trust power §9.9; **vouch (§9.7) discloses sender+voucher+recipient in cleartext — a carve-out, not a residual — §6.6 item 18, §9.1 item 4** |

An implementation or a formal model that exhibits a counterexample to any SP-*n* **claim line
above** — without invoking its stated residual — has found a spec-level defect, and it MUST be
filed as such (§10.4). That is the point of stating them falsifiably.

**Where the formal models live.** This specification repository ships the **CDDL** wire grammars
(§18) and the conformance vectors (§10.3); the **symbolic/ProVerif protocol models** that exercise
the SP-*n* claims are maintained in the **reference implementation's `formal/` directory** (the
Envoir monorepo), not in this spec repo. They are versioned against the mechanisms cited above and
are the executable counterpart to this falsifiable-claims table; a divergence between a model and
this text is resolved in favor of the **spec** (§10.4).

## 6.10 Measured anonymity evidence (mechanism-model simulation — research-tier, non-normative, supporting the opt-in mixnet only)

**This entire subsection is research-tier and non-normative**, exactly like the SP-3/SP-4 claims
it corroborates ([docs/research/mixnet.md](docs/research/mixnet.md)) — it is evidence about the
opt-in `private` mixnet, not about DMTAP's default (`fast`-tier) guarantee, and it is retained here
(rather than moved) because it is evidence *about a claim*, stated in falsifiable form in §6.9,
not machinery a future graduation needs to restore byte-for-byte. The §6.9 properties are stated
to be **refutable**; this subsection reports what a **mixnet anonymity simulator** in the
reference measured when the opt-in mixnet's mechanisms
([docs/research/mixnet.md §4.4](docs/research/mixnet.md)) were exercised against a global
adversary. It is offered as **corroborating evidence** for SP-3/SP-4 (and the §6.6 item 1 honest
floor) — properties that hold, if at all, only for a party that has opted into that research-tier
layer — and it is **explicitly caveated**:

> **This is a mechanism-model simulation, NOT the deployed network, and it says nothing about the
> default `fast` tier.** The simulator models the opt-in mixnet's constructions
> ([docs/research/mixnet.md §4.4](docs/research/mixnet.md)) — Poisson per-hop mixing, loop/drop
> cover, stratified path selection, entry guards, operator diversity, and colluding-mix placement
> — under an idealized traffic model. It is a *sanity check that the mechanisms behave as designed
> in the abstract*, not a measurement of a real fleet (none has shipped), and it is **not a
> substitute** for the pre-deployment external audit gate (§12.8.4). Real-world anonymity depends
> on the live anonymity set ([docs/research/mixnet.md §4.4.11](docs/research/mixnet.md)),
> implementation fidelity (the weakest-link caveat, §6.6 item 5), and side channels a mechanism
> model does not capture. No production claim rests on these numbers, and no claim here extends to
> a party that has not opted into the mixnet.

With that caveat, the simulator's four measured findings each map to a claimed property:

1. **Passive correlation converges toward the 1/N chance floor as cover and hops grow.** Against a
   *global passive* adversary, the simulated probability of correctly linking a target's sender to
   its receiver fell toward **1/N** (N = the anonymity set — indistinguishable senders) as
   cover-traffic rate and hop count increased: with enough cover and mixing, the adversary's
   guess approached a **uniform random guess over the anonymity set**. This is the mechanism-model
   evidence for **SP-3** (sender anonymity vs. global passive) and shows *why* cover traffic is
   load-bearing, not optional (§6.2, [docs/research/mixnet.md §4.4.5](docs/research/mixnet.md)) — the convergence is *to* the floor, not *below* it,
   and it is a floor set by N, i.e. by adoption ([docs/research/mixnet.md §4.4.11](docs/research/mixnet.md)), which the model idealizes.

2. **Active drop attacks are detected at the 20%-loss loop threshold.** When the simulated
   adversary dropped packets on a target's paths, the **loop-cover return fraction**
   ([docs/research/mixnet.md §4.4.7](docs/research/mixnet.md)) fell below its detection threshold
   at ≈ **20% loss** (§16.3), triggering the inferred-active-attack response — rotate +
   `HALT_ALERT` + fail-closed (`0x030F`). This is the evidence for **SP-4**'s "drop/delay is
   *detected and responded to*": above-threshold suppression is caught; the honest residual is
   **sub-threshold** dropping (< 20%), which stays under the detector but, dropping so little,
   accomplishes correspondingly little (§6.6 item 1). Faster loops (High-security,
   [docs/research/mixnet.md §4.4.10](docs/research/mixnet.md)) tighten this floor.

3. **Anonymity degrades with the compromised fraction f, tracking ≈ f².** As the fraction *f* of
   mixes under adversary control grew, the simulated deanonymization probability rose **≈ f²** —
   the joint probability that **both** the entry **and** the exit of a path are adversarial, the
   placement from which a colluding pair correlates a flow. This is the quantitative shape of the
   [docs/research/mixnet.md §4.4.8](docs/research/mixnet.md) bound and of the §6.6 item 1
   residual: entry guards + **attested** operator diversity ([docs/research/mixnet.md
   §4.4.8](docs/research/mixnet.md)) are what hold the exponent at ≈ *f*² rather than letting a
   single Sybil operator faking *N* identities collapse it to ≈ *f* ([docs/research/mixnet.md
   §4.4.8](docs/research/mixnet.md), §10.7.2). It also quantifies **SP-4**'s "bounded,
   not eliminated" — the bound is ≈ *f*², not zero.

4. **More hops defend against timing correlation but NOT against colluding entry+exit mixes.** The
   single most important honesty result: increasing the hop count in the simulation **lowered the
   timing-correlation success of a network *observer*** (more memoryless hops to defeat, finding 1),
   but left the **≈ f² colluding-entry+exit deanonymization essentially unchanged** — adding honest
   middle hops does not reduce the chance that the two *ends* are both adversarial. Hop count and
   operator diversity therefore defend **different** attacks: hops buy timing-correlation resistance
   (against observers), diversity + guards buy entry+exit-collusion resistance (against colluding
   mixes), and **neither substitutes for the other** ([docs/research/mixnet.md
   §4.4.10](docs/research/mixnet.md)). Any claim that "just add more hops" defeats a
   colluding-endpoint adversary is refuted by this measurement; the normative prohibition on
   making that claim is owned by §6.6 item 1 — this section stays informative.

**Honest reading.** These results *support* the claims exactly as §6.9 states them and *refute* the
overclaims §6.6 disallows: passive correlation is driven to the chance floor (not below), active
dropping is detected (above threshold, not below), and the colluding-endpoint residual is a real
≈ *f*² floor that more hops do not remove. The evidence strengthens the honest posture; it does not
license a stronger claim than the residuals permit, and it does not replace §12.8.4's external
audit.
