# MEDIA — the Evermesh profile (video & music)

> **Status:** profile spec (KOTVA family). Normative once ratified. MEDIA is the consumer-media
> profile — a YouTube/Spotify/Twitch-shaped surface (uploaded video, music, podcasts, live) built
> as a **thin profile**, not a new stack. It **generalizes** the Published-Artifact media facet
> ([`24-video-profile.md`](../24-video-profile.md), the `VideoManifest`/channel/live model) from a
> publishing convention into a full product, and adds **no new wire bytes** beyond the one signed
> structure §24 already owns. Where §24 defines the objects, this document states the product rules
> — box-as-origin, CDN as a swappable cache tier, live as a media-relay plane, channels-as-feeds,
> discovery as a coordinator, and licensing honesty — and does **not** restate §24's schemas.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are to be interpreted as in BCP 14 (RFC 2119, RFC 8174).

---

## 1. What this is

Evermesh is what you get when the media platform's four welds are cut. A platform joins **who you
are** (an account), **what you publish** (rows + blobs it hosts), **who can find it** (its
recommendation engine), and **how you are paid** (its ad/subscription relationship) — and owns all
four. MEDIA separates them onto the substrate:

| | Platform | Evermesh |
|---|---|---|
| Identity | an account it issues | a keypair you hold ([§1](../01-identity.md)) |
| Catalogue | rows + blobs it hosts | a signed author feed you publish ([§22.4](../22-public-objects.md), [§24.5](../24-video-profile.md)) |
| Origin | its servers | **your box** — the authority; CDN and mirrors are caches (§4) |
| Discovery | its ranking | a derived index any node may rebuild ([§22.4.3](../22-public-objects.md)), served by a swappable `indexer` |
| Payment | ads + its subscription | a settlement seam chosen per work; **no surveillance ads** (§7, [DIRECTION §5](../DIRECTION.md)) |

A **media work** — a video, a song, an album track, a podcast episode, a live set — is exactly a
§24 `VideoManifest`: an original media blob, signed renditions, caption/lyric tracks, a
thumbnail/cover, a license, published as a `pub_announce` on the creator's own feed. Video and audio
are **one object model** (§24.1): an encoding with no video track omits `width`/`height`, and that
absence *is* the audio signal. MEDIA adds the operational envelope around that object model — how it
scales, how it goes live, how it is discovered, gated, and paid for — while keeping the object the
same at every scale.

---

## 2. Which primitives, coordinators and bindings it composes

MEDIA is composition, not new bytes. It sits over the DMTAP-PUB substrate and pulls in:

**Primitives** ([`primitives/`](../primitives/)) — used only where a work is *traded*, never for the
free-publishing common case:

- **OFFER** ([`OFFER.md`](../primitives/OFFER.md)) — a paid work, a members-only tier, a
  pay-per-view stream is an `Offer`: one creator's signed claim to supply access on stated terms,
  referencing the work by content address. Free public media needs no OFFER — it is a bare
  `pub_announce`.
- **REPUTATION** ([`REPUTATION.md`](../primitives/REPUTATION.md)) — creator standing and
  "recommended for you" are **locally measured or OpenRank-computed over the public follow/reaction
  feeds**, keyed to the creator's `IK`. **No published global score** (§6). This is §24.8's
  "aggregates are claims" posture given a primitive name.
- **ATTEST** ([`ATTEST.md`](../primitives/ATTEST.md)) — a `RightsClaim` (§24.11), a verified-creator
  badge, and age-gating are attestations: `{issuer IK, subject, schema, claim}` proving *IK said
  this*, never the fact itself. Age-gating binds to a personhood attester the viewer chose, not to
  the creator's word; the work carries the flag as a signed `meta` text-key (§22.3.1 — inside the
  signed body, not a new wire kind or discriminator, permitted under MED-1) that a client checks
  against the viewer's attestation before rendering.
- **ESCROW / PAY** ([`ESCROW.md`](../primitives/ESCROW.md)) — settlement for a paid work rides an
  existing stablecoin rail; nothing is escrowed for a completed download, and a `RailClass` carries
  the recourse weight.

**Coordinators** — every one obeys [`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md): hired,
swappable, self-hostable, never load-bearing, visibility declared:

- **`indexer`** — search, category, "trending," recommendation over a corpus that is public
  plaintext — nothing to be blind about. Its query channel is `terminating` unless `attested`
  (TEE), so a reader's search terms are a disclosed metadata leak, not a payload-visibility one.
  Derived, rebuildable, never authoritative (§6).
- **`media-relay`** — forwards SFrame-encrypted **gated/members-only live** media, MLS-keyed to the
  entitled group, so the streamer's uplink is not the audience-size limit. `blind-routing` /
  `structural` for that case (§5) — SFrame seals the media payload, but the SFU reads routing
  metadata (participant graph, speaker timing, stream sizes) to forward frames (CONTRACT §3.1).
  Public live has no SFrame keying and does not ride this coordinator (§4 MED-4).
- **`relay`** — mesh reachability so a box behind CGNAT can still serve its feed. `blind` /
  `structural`.
- **`reachability-adapter`** — a public subdomain for a box's PUB HTTP surface (§22.5.1).
  `blind-routing` (SNI-passthrough) preferred.
- **`labeler`** — opt-in moderation labels over public works; you subscribe to the ones you trust
  (§10).
- **`arbiter` / `oracle`** — rights disputes and physical-fact attestation, disclosed, not
  protocol takedown (§8).

**Bindings** ([`bindings/README.md`](../bindings/README.md)) — adopted, never rebuilt:

- **Walrus** (hot, CDN-like) for scaled retrieval of public blobs — the Evermesh CDN answer;
  **Arweave** for permanent archival of a work; **Filecoin** for bulk durability.
- **WebRTC + SFrame (RFC 9605) + TURN + a distributed SFU** (LiveKit/Jitsi/mediasoup) for the live
  real-time plane.
- **MLS (RFC 9420)** to key sealed, members-only media (§7).
- **x402 + stablecoins** for purchase; **payment channels / Superfluid-class streams** for
  recurring subscriptions.
- **OpenRank** for reputation/recommendation compute; **EAS / W3C VC** for rights and creator
  attestation; **World ID / Human Passport** for the age-gate personhood anchor; **Kleros-class**
  arbitration for rights disputes.

---

## 3. MOTE kinds and PUB objects

MEDIA is a **PUB profile**: authenticity without confidentiality is the default, because published
media is public by design ([§22.1](../22-public-objects.md)). It allocates **no core message kind
and no §21 error block** — every object is §24's, and §24 in turn embeds unsigned application maps in
an already-signed `pub_announce.meta` (§22.3.1). The objects, all `pub_announce` (kind `0x40`) on the
author's own feed:

| Object | §24 source | Role in Evermesh |
|---|---|---|
| `VideoManifest` (`meta["video"]`) | §24.4 | the media work — video or audio-only |
| `Rendition` + derivation statement | §24.4.3–4 | signed transcodes; the **one** profile-local signed structure the family owns |
| `Channel` (`meta["channel"]`) | §24.5 | a creator's channel = their author feed (§22.4) |
| `Comment` / `Reaction` / `Follow` / `Playlist` | §24.6 | social graph; `Playlist` **is** the album/EP/release |
| `LiveManifest` / `LiveChat` (`vid-live-1`) | §24.10 | rolling signed segment batches; the durable record of a live stream |
| `Mirror` | §24.7 | a serving assertion → the cache/pin role ([`ROLES.md`](../substrate/ROLES.md)) |
| `RightsClaim` (`meta["claim"]`) | §24.11 | a self-asserted rights claim, presented with provenance, never as truth |
| `Delegate` grant | §24.4.6 | authorizes a third-party transcoder |

**MOTE (the sealed object) is used in exactly two places, and only there:**

1. **Gated media (§7).** A members-only or paid work whose *bytes* are confidential is a sealed file
   ([§5.5](../05-messaging.md)) keyed via MLS to the entitled group; the entitlement/key is delivered to
   each buyer as a MOTE. The public `pub_announce` still exists as the *listing* (an OFFER, §2);
   only the media payload is sealed.
2. **Live-start notification.** A content-free **Wake** ([`ROLES.md`](../substrate/ROLES.md)) tells a
   subscriber's box to reconnect and fetch the new `LiveManifest` — wake-and-fetch, never
   deliver-in-push.

Nothing else in Evermesh is a MOTE. The real-time live *bytes* are on the WebRTC media plane
([DIRECTION §7](../DIRECTION.md)), not the store-and-forward object model — correctly, since
real-time must not be forced through MOTE delivery.

---

## 4. Normative profile rules

- **MED-1 — No new bytes.** A conforming Evermesh publisher emits §24 objects over §22 bytes and
  nothing else. The profile MUST NOT define a new wire kind, a new signed structure, a new hash
  construction, or a media-kind discriminator field (§24.4.2). A §22-only node with no §24 awareness
  already stores, serves and swarms every Evermesh object.

- **MED-2 — The box is the origin and the authority.** The creator's box holds the signing identity,
  the author feed, and the canonical `original` blob (§24.4.3). Every CDN copy, mirror, rendition and
  index is **derived**; on any disagreement the signed feed and the `original` win (§22.4.3,
  §24.8). A consumer that needs the highest-fidelity source MUST be able to reach `original.blob`,
  never only a lossy transcode.

- **MED-3 — The CDN is a swappable cache tier, never load-bearing.** A Walrus/CDN/mirror location is
  carried as an advisory `Hint` (§24.4.5) or a `Mirror` assertion (§24.7); it is **not** the address.
  Every blob self-verifies against its §22 content address regardless of where the bytes came from
  (§22.5.1), so a client MUST verify on fetch and MUST be able to fall back to origin, mesh, or
  another mirror with no identity change and no re-announce. Switching CDN is a config change
  ([CONTRACT §2.2](../coordinator/CONTRACT.md)). A CDN that could withhold or alter a work without
  detection would be a conformance violation; content addressing makes that impossible.

- **MED-4 — Gated live is a media-relay plane; public live is plaintext segment serving; both keep a
  verifiable durable shadow.** A **gated/members-only** low-latency stream rides the WebRTC + SFrame
  plane, keyed to the bounded MLS group, through a pool of `media-relay` coordinators
  (`blind-routing`/`structural`, because SFrame E2E-encrypts the media payload to that group — the
  relay holds no key but reads routing metadata: participant graph, speaker timing, stream sizes),
  coordinated by a distributed SFU so the host's uplink is not the size limit
  ([DIRECTION §7](../DIRECTION.md)). §27 defines no anonymous-audience SFrame keying, so a **public**,
  open-audience stream MUST NOT be presented as riding this blind-routing plane: it is instead §24.10 rolling
  `LiveManifest` segments served as ordinary public plaintext (§24.13 — a holder serves plaintext it
  can read), fronted by the same CDN/mirror tier as VOD (MED-3), never a `media-relay`. In both cases
  the streamer MUST publish the §24.10 rolling `LiveManifest` chain on its feed, so the stream has the
  **same integrity as VOD** and closes into an ordinary `VideoManifest` for archive. The two planes
  are separate: MOTE delivery MUST NOT be on the real-time path.

- **MED-5 — Channels are feeds; the social graph is portable.** A channel is an author feed (§24.5);
  a `Follow` names the followed identity's `IK`, not a platform's user-id (§24.6.3). Subscribing,
  unsubscribing, and moving between apps MUST NOT require re-telling followers or re-establishing
  identity — the graph lives at the edge, in signed feeds ([CONTRACT §2.2](../coordinator/CONTRACT.md)).

- **MED-6 — Discovery is a coordinator that authorizes, never classifies.** Search, trending, and
  recommendation are `indexer` output: **derived, rebuildable, never authoritative** (§22.4.3), TEE
  (`attested`) preferred. An indexer MAY rank or re-rank content within its own derived,
  non-authoritative view — that ranking *is* its function; it MUST NOT drop, gate, quarantine, or
  re-rank on a *delivery* or *canonical/authoritative* path, and MUST NOT publish a global reputation
  score ([CONTRACT §4](../coordinator/CONTRACT.md), [REPUTATION.md](../primitives/REPUTATION.md)).
  Multiple indexers competing over the same feeds is the design, not a fault; a client MAY use several
  or its own local index.

- **MED-7 — Paid and gated media use existing rails; no token, no surveillance ads.** Access to a
  paid work is an OFFER (§2) settled on an existing stablecoin rail (x402) or a payment-channel
  stream for subscriptions; the media payload, where confidential, is MLS-sealed and the entitlement
  delivered as a MOTE (§3). The profile MUST NOT mint a protocol token, MUST NOT operate a
  surveillance-based ad market ([DIRECTION §5](../DIRECTION.md)), and MUST NOT make free tiers,
  subsidies, or ads a *protocol* requirement — they are operator policy, escapable by self-hosting.

- **MED-8 — Licensing is honest: a claim, never truth.** A `license` field (§24.4.1 key 9) is an
  SPDX expression or a profile consent token; a `RightsClaim` (§24.11) is a *self-asserted*
  attestation presented with provenance. There is **no protocol-level takedown** (§22.6): a holder
  chooses what it serves; the protocol compels neither serving nor removal. Rights disputes route to
  an `arbiter`/`oracle` coordinator (§8), disclosed — not adjudicated by the substrate.

- **MED-9 — Every intermediary declares its content-visibility.** Per [CONTRACT §3](../coordinator/CONTRACT.md),
  each MUST declare exactly one class at one assurance level, surfaced to the user: a **public-blob
  CDN/mirror** is **not blind** — the cache/pin role ([ROLES.md §6](../substrate/ROLES.md)) that
  holds and serves plaintext a holder can read; payload confidentiality is n/a because the content is
  public by design ([§22.1](../22-public-objects.md)). A **sealed-media CDN** and a gated-stream
  **`media-relay`** are `blind-routing`/`structural` (CONTRACT §3.1): neither holds a key, but each
  sees routing metadata — which reader fetched which sealed object (SEC-9), or the SFU's per-frame
  participant graph, speaker timing, and stream sizes. An **`indexer`**'s corpus is public plaintext,
  nothing to be `blind` about ([CONTRACT §5](../coordinator/CONTRACT.md)); its query channel is
  `terminating` unless `attested` (TEE), so a reader's search terms are a disclosed metadata leak. No
  coordinator here may silently downgrade from a class it can run `blind` or `blind-routing` at
  (CONTRACT §3.2).

- **MED-10 — Moderation is edge-selected and opt-in.** A viewer's client decides what it renders,
  against `labeler` subscriptions it chose (§24.13); a labeler is itself a coordinator under the
  contract (it labels; you subscribe; you can leave). Anti-abuse allowed is authorization-level only —
  authenticated identity, rate limits, optional postage for cold contact — never central content
  classification ([CONTRACT §4](../coordinator/CONTRACT.md)).

---

## 5. Scale-invariance

The objects are identical from a village to a planet; only the **trust and reach anchors** slide
([DIRECTION §6](../DIRECTION.md)):

| Function | Small / mesh (no coordinator) | Global (swappable coordinator) |
|---|---|---|
| Serving | box serves its feed directly to followers over LAN/mesh | Walrus CDN + mirror pool front the same content addresses |
| Live | one box → a handful of peers over WebRTC direct/STUN | `media-relay` pool + distributed SFU cascade |
| Discovery | following-graph + local index | competing `indexer`s |
| Recommendation | web-of-trust, "what people I follow reacted to" | OpenRank over the attestation graph |
| Reputation | direct and local | `indexer`-derived, still no global score |
| Age-gate | web-of-trust (you know these viewers) | a personhood attester the viewer chose |
| Payment | direct stablecoin transfer | x402 / streaming subscription |

The same `VideoManifest`, `Channel`, `Follow` and `LiveManifest` carry every rung of that ladder.
Adding a coordinator makes global reach *available*; it never becomes *required*, and removing one
collapses the function to its local-trust form without changing an object.

---

## 6. Offline / apocalypse behavior

Graded per [`substrate/OFFLINE.md`](../substrate/OFFLINE.md); the **Sneakernet test** holds because
every Evermesh object is self-authenticating and verifies identically over HTTPS, mesh, or an SD card
— a creator's entire channel can move on physical media and be verified with zero DNS.

- **Authoring — `full`.** Appending a new work, rendition, comment, reaction or feed head to the
  local signed feed is fully offline (§22.4, OFFLINE §3.3).
- **Playback of cached works — `full`.** A cached `VideoManifest`, its blobs and captions
  self-verify offline (§22.5, §22.2); no server is a trust root.
- **Distribution — `deferred`.** Pushing new works and feed heads to the swarm/CDN settles on
  reconnect; the intent is captured, never dropped (OFFLINE §3.3).
- **Discovery / recommendation — `local-trust`.** A stale local index is a disclosed reduced-assurance
  view, never presented as authoritative (§22.4.3, OFFLINE §2).
- **Age-gate — `blocked` (fresh proof) / `full` or `local-trust` (cached).** Checking the personhood
  attestation against a fresh attester needs connectivity and is `blocked` offline; a cached VC the
  viewer already holds lets the check proceed with no round trip — `full`/cryptographic for a
  gated work via the MLS entitlement, `local-trust`/advisory for a public work (§7, §8).
- **Live — `blocked` (disclosed honestly).** Real-time delivery needs a live media path and a
  `media-relay`; it cannot be deferred, because deferred real-time is not real-time. Evermesh MUST
  fail this closed and say why — never fake a live session offline (OFFLINE §2 R-GRADE-2). The
  §24.10 chain that *records* the stream is still `full`/`deferred` like any other feed content; only
  the low-latency plane is `blocked`.
- **Paid settlement — `deferred` / `blocked`.** A purchase intent is captured offline and settles on
  reconnect; final on-rail settlement is `blocked` until connectivity, and MUST NOT be shown as paid
  before it is (OFFLINE §5, `RailClass` finality `blocked`).

Aggregate counters (views, reactions) obey **R-SYNC-1** (OFFLINE §3.4): convergence is not invariant
preservation. A view count is a derived claim reassembled from feeds, not a protected balance — over-
or under-counting on a partition is surfaced as the honest state, never a merge that invents a total.

---

## 7. Security and declared content-visibility

Inherits [`THREAT-MODEL.md`](../THREAT-MODEL.md) whole; the profile-specific posture:

- **SEC-2 (intrinsic authenticity).** Every work, channel head, comment and rendition is
  self-authenticating — a signature or a content address it carries — so any node MAY serve any
  object without being trusted (§22.3.3, §22.4.3). A CDN or attacker in its place cannot forge or
  alter a work.
- **SEC-3 (confidentiality, where bought).** Public media has none by design (§22.1). A gated work's
  payload is MLS-sealed end-to-end (§7); the sealing coordinator/CDN holds no key and is
  `blind-routing`/`structural` — it cannot read the payload, but it sees which reader fetched which
  sealed object (SEC-9).
- **SEC-4 (declared visibility).** Every intermediary declares one class (MED-9); a client surfaces
  the trust boundary a path crosses ([CONTRACT §3](../coordinator/CONTRACT.md)).
- **SEC-7 (abuse priced, not filtered).** View/reaction inflation is Sybil-bounded, handled as
  aggregates-are-claims (§6) plus cold-contact cost and local web-of-trust — never central content
  filtering. Recommendation Sybil-resistance is REPUTATION's ceiling, disclosed, not solved.
- **SEC-9 (metadata).** Public reads are anonymous content-addressed fetches (§22.5), but a CDN or
  indexer sees *what is fetched and roughly by whom*. This exposure is reduced, not eliminated, and
  is disclosed here rather than hidden.

Rendition trust is **accountability, not fidelity** (§24.4.4): a signed derivation proves *who*
transcoded, and a malicious transcoder can still sign an unfaithful rendition — the remedy is
revocation plus edge reputation, never a protocol guarantee of transcode correctness.

---

## 8. Honest residual

MEDIA covers the mechanism of a media platform; it discloses, rather than solves, the following. Each
traces to a root ceiling in [DIRECTION §8](../DIRECTION.md).

- **Live is not apocalypse-proof.** Real-time needs a live path; it is `blocked` offline (§6). The
  recorded stream survives; the *liveness* does not. This is honest, not a gap to close.
- **Age-gating is enforceable only for gated works.** For a public (non-sealed) work the age flag is
  a client-side rendering choice: a non-cooperating client can ignore it, exactly like a moderation
  label (MED-10) — it is advisory, not a protocol guarantee. It is cryptographically enforced only for
  a sealed/gated work, where the MLS entitlement is itself the gate (§7): the personhood attestation
  must be presented to obtain the decryption key, not merely checked by a compliant renderer.
- **No takedown.** Under §22.6 irrevocability, illegal or abusive content persists on any willing
  holder. Moderation is edge-selected + opt-in labelers (MED-10) — genuinely weaker than a platform's
  unilateral delete, and disclosed as such. The safety floor is a *labeler market*, not a switch.
- **Copyright/rights is a claim, not a proof.** A `RightsClaim` proves *who said what*, never *who
  owns what*. Real ownership disputes hit the **legal/authoritative-issuer** ceiling
  ([DIRECTION §8.3](../DIRECTION.md)): an `arbiter`/`oracle` (a licensed, paid coordinator) can
  adjudicate, but the burden moves — it does not vanish — and that operator is structurally permanent
  and disclosed ([ESCROW.md](../primitives/ESCROW.md) honest residual).
- **View counts and "trending" are gameable.** They are derived claims over a Sybil-imperfect graph
  (§6, REPUTATION ceiling). Local scale dissolves this into web-of-trust; global scale raises the
  Sybil floor with a personhood anchor but never closes it.
- **The CDN is a CDN, not an archive.** Walrus behaves like hot cache; a content address is a name,
  not a durability promise ([§22.1.2](../22-public-objects.md)). Permanence costs — Arweave, or an
  origin-hold the creator's box keeps serving. A work whose every holder stops serving it is gone,
  and the profile does not pretend otherwise.
- **Recommendation without surveillance is weaker.** No behavioral surveillance engine means the feed
  is less personalized than an ad-funded platform's — the one thing genuinely surrendered by not
  being a company that owns you ([research §3](../docs/research/README.md)), and surrendered on
  purpose.
- **A non-attested indexer's query privacy is a promise, not a proof.** Per
  [CONTRACT §3.4](../coordinator/CONTRACT.md), only `structural` and `attested` levels are
  verifiable. An indexer's query channel is `terminating` unless it runs a TEE (`attested`, MED-9);
  absent that, the operator's promise not to log or correlate a reader's search queries is
  `declared`-level trust, not proof. A client MUST NOT present a non-attested indexer's query
  handling as verified.
