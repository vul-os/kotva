# RTC — the real-time voice / video / calling profile

> **Status:** profile spec (KOTVA family). Normative once ratified. RTC is the **real-time
> modality** of [DIRECTION §7](../DIRECTION.md) — the parallel media plane, distinct from the
> async substrate profiles ([`tract`](tract/README.md), [`wrap`](wrap/README.md)) that ride
> MOTE + PUB + SYNC. It defines **no new bytes**: the wire spec is
> [`../27-realtime-media.md`](../27-realtime-media.md) (DMTAP-RTC), which owns the `rtc_signal`
> object, the SFrame key schedule, the capacity advertisement, the errors and the conformance
> table. This document is the **family-level view** — what RTC composes, how it stays
> scale-invariant, how it degrades offline, and where its guarantees stop. Where this document
> and §27 appear to differ on a wire mechanism, **§27 governs.**

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are BCP 14 (RFC 2119 / RFC 8174).

---

## 1. What this is

RTC is the profile for a **call**: a continuous, latency-bounded media flow between endpoints
that are simultaneously online — 1:1 and multi-party voice, video, screen sharing, and system/tab
audio, at parity with a mainstream conferencing product, **with no proprietary signaling server
and no operator that can read the media.**

A call is the opposite shape from a MOTE. A MOTE is signed, sealed, queued, retried and acked —
store-and-forward (§2). A call is a live stream that cannot be queued and must not be forced
through object delivery. So RTC follows [DIRECTION §7](../DIRECTION.md) exactly: it rides a
**parallel media plane** (WebRTC) that **reuses** the substrate's identity, its group keys, its
infrastructure roles and its signed coordination, but **not** its store-and-forward object model.
Only the media *bytes* are on their own track. That separation is the whole design; everything
below is a consequence of it.

The one thing the media plane still needs from the substrate is a **signaling carrier**: WebRTC's
JSEP is deliberately signaling-agnostic (it says *what* endpoints exchange, never *how*), and every
deployed product fills that hole with a central server. The substrate already is a signed,
mutually-authenticated, end-to-end-encrypted delivery layer, so the hole is already filled — RTC
adds only a message kind, a key label, and a capacity advertisement (§27.1, §27.3).

---

## 2. What it composes — waist capabilities, coordinators, bindings

### 2.1 Waist capabilities (the substrate it reuses)

| Waist capability | How RTC uses it |
|---|---|
| **Identity** ([`../01-identity.md`](../01-identity.md)) | A keypair is the call endpoint. An SFU is identified by its own identity key, disclosed before any media (§27.4.1 keys 9/10). |
| **MOTE** ([`../02-mote.md`](../02-mote.md)) | Signaling is one sealed-MOTE kind, `0x44 rtc_signal`, on the ordinary deliver/ack/retry path (§27.3, §27.4). Media is **not** a MOTE. |
| **SYNC / MLS group** ([`../substrate/SYNC.md`](../substrate/SYNC.md)) | The MLS group that scopes the conversation supplies **membership** (the sole call authorization) and, via its epoch exporter, the **media key root** (§27.5). A 1:1 call is a call in a 2-member group. |
| **Roles & Wake** ([`../substrate/ROLES.md`](../substrate/ROLES.md)) | The SFU is an infrastructure **role** any node may take — your own always-on box or an operator — never a privileged server type (§27.7.2). Wake is content-free push; it is **never relied on for correctness** ([`../substrate/OFFLINE.md`](../substrate/OFFLINE.md) §3). |

RTC does **not** compose the commerce recipe roles
(`OFFER · MATCH/RESERVE · REPUTATION · ESCROW · ORACLE · DISPUTE · PAY`, [DIRECTION §2](../DIRECTION.md))
in its media path — a call
is not a trade. They enter only at the seam where the call is **wrapped in a paid or booked
service**: a metered third-party SFU discloses a signed tariff and issues signed receipts to the
payer ([`../coordinator/CONTRACT.md §6`](../coordinator/CONTRACT.md)); its trust is
**locally measured**, never a published score ([`../primitives/REPUTATION.md`](../primitives/REPUTATION.md));
a scheduled call is an ordinary [`RESERVE`](../primitives/RESERVE.md) hold against a calendar. None
of these is part of the call itself, and RTC hard-wires none of them.

### 2.2 Coordinators (all under the contract)

Every intermediary RTC can place in a path is a coordinator under
[`../coordinator/CONTRACT.md`](../coordinator/CONTRACT.md), declaring exactly one visibility class:

| Coordinator kind | Role in a call | Visibility (declared) |
|---|---|---|
| **media-relay** (the SFU) | Forwards SFrame ciphertext so the host is not the size limit (§27.7.2) | `blind-routing` / `structural` — holds no epoch key, cannot read media; sees routing metadata (participant graph, speaker timing, stream sizes) |
| **relay** (TURN) | Relayed ICE candidate when direct/STUN fails (§27.11 item 3) | `blind` / `structural` — carries SFrame ciphertext |
| **reachability-adapter** | Optional public ingress to reach a self-hosted SFU box | `blind-routing` (SNI-passthrough) preferred |

An SFU/relay **authorizes, never classifies** ([CONTRACT §4](../coordinator/CONTRACT.md)): it may
check *who you are and your rate*, never *what your media contains* — which it structurally cannot,
because it holds no key. None is load-bearing: remove every coordinator and a **mesh** call still
works (§6).

### 2.3 Bindings (what RTC adopts, never reinvents — [DIRECTION §3](../DIRECTION.md))

Every byte on the media path is a byte another standards body specified
([`../bindings/README.md`](../bindings/README.md)):

- **WebRTC** (RFC 8825 stack) — transport, SDP, JSEP, ICE/STUN.
- **SFrame** (RFC 9605), **keyed from the MLS epoch** — end-to-end media protection that makes the
  relay `blind`/structural.
- **TURN** (RFC 8656; coturn) — the NAT-relay served by the `media-relay` / `relay` role.
- **MLS** (RFC 9420) — the group, its epochs, and the exporter the media key is derived through.
- **Distributed SFU** — **LiveKit** (Pion) / **Jitsi Videobridge + Octo** / **mediasoup**:
  multi-provider, cascaded forwarding so no single host bounds the call. **Bound, not built** —
  §27 is implementable against an SFU that does not know KOTVA exists (§27.6.2, §27.7).

---

## 3. The MOTE kinds & objects it uses

RTC allocates the minimum §27 enumerates and nothing more:

| Wire object | Home | Nature |
|---|---|---|
| **`rtc_signal`** (message kind `0x44`) | §27.4.1 | An **ordinary sealed MOTE** — offer/answer/candidate/bye carried under the MLS group's epoch key, with an inner `type` discriminator. Default tier `fast` (§27.4.6). One kind, not four (§27.3.2). |
| **`RtcCapacity`** | §27.7.4 | A signed capacity ceiling (tracks + bitrate, **never headcount**) carried in an ordinary `system` MOTE (`0x0A`) alongside the `rtc-sfu-1` token — no new signed structure; inherits `Payload.sig` and monotonic rollback protection. |
| **`rtc-1`** / **`rtc-sfu-1`** capability tokens | §27.8 | An endpoint's opt-in to calling; an operator's opt-in to the SFU role. |

**No PUB / author-feed object.** A call is ephemeral, not a durable public record — it publishes
nothing to [`../22-public-objects.md`](../22-public-objects.md). (A *recorded* call that a
participant chooses to publish leaves this profile entirely and becomes an ordinary §24 media work.)
**Media bytes are not MOTEs**: SFrame-protected SRTP on the parallel plane, never content-addressed,
never re-relayed as objects.

---

## 4. Normative profile rules

These are the family-level MUSTs; the authoritative, enumerated conformance set is §27.10
(**RTC-1 … RTC-20**) and the errors are §27.12 (`0x0415`–`0x0417`). This section does not restate
them — it states the rules a profile reader must not miss.

- **R-RTC-1 (media is content-blind, where the operator has committed to it).** Against an SFU that
  publishes `sframe_required=true`, media MUST be SFrame-protected under a key derived from the MLS
  epoch exporter (§27.5.1); no intermediary may hold that key. An endpoint MUST NOT emit or forward
  unprotected media on a call that negotiated SFrame, and MUST NOT accept a renegotiation that
  removes it — protection **ratchets up only** (`ERR_RTC_SFRAME_REQUIRED`, §27.5.2). An operator MAY
  instead publish `sframe_required=false`, declaring it will forward unprotected media if offered;
  that operator is a `terminating` coordinator, not `blind-routing`, and a client MUST disclose it as such
  before joining (§27.7.4, R-RTC-3) rather than treat content-blindness as unconditional.
- **R-RTC-2 (membership is the authorization).** An `rtc_signal` MUST be accepted only from a
  **current member** of the group under whose epoch the MOTE decrypted, evaluated against the
  receiver's own applied state — never a claim in the signal (§27.4.5). This is
  authorize-never-classify at the endpoint: the question is *who*, never *what*.
- **R-RTC-3 (the operator is disclosed before media).** Before a byte of media reaches an SFU, the
  client MUST display *which* operator identity is in the path and whether it has committed to
  `sframe_required` (§27.9 item 4). A topology change (mesh→SFU, or between SFUs) is a
  renegotiation the peer MAY refuse; an implementation MUST NOT silently interpose a forwarder
  (§27.7.2, RTC-18).
- **R-RTC-4 (limits before admission, in the right unit).** An SFU MUST publish its ceiling in
  **tracks and bitrate** before admitting anyone and MUST re-check on **every** renegotiation
  (because renegotiation is how a screen share is added). Headcount MUST NOT be the admission basis
  (§27.7.4, RTC-15/16). A call refused up front is retried in seconds; a call degraded mid-session
  fails in front of its participants with no remedy.
- **R-RTC-5 (calling is an explicit, disclosed act).** Placing a call MUST be an explicit user act,
  and the client MUST surface — before the first signaling MOTE leaves — that a call reveals its
  existence, timing and duration to observers and its endpoint addresses to the peer or SFU
  (§27.4.6, §27.9 item 2). A security-relevant reduction is refused or surfaced, **never silent**.
- **R-RTC-6 (keys die with the call).** Media key material MUST be deleted at the reorder-window
  edge and unconditionally at teardown, and MUST NOT be persisted or backed up (§27.5.2, §27.9
  item 6). Failing to delete is the one way to pass every other rule and still lose forward secrecy.
- **R-RTC-7 (no MCU dressed as E2E).** Server-side mixing (an MCU) requires media plaintext and is
  out of scope; an implementation MUST NOT present a mixed call as end-to-end encrypted (§27.7.3,
  RTC-19).

---

## 5. Scale-invariance — one call, from a mesh to a planet

The primitives never change; only the **trust anchor slides**
([DIRECTION §6](../DIRECTION.md)). The same identity, the same MLS group, the same SFrame keys carry
the call at every scale — only *who forwards the bytes* changes:

| Scale | Forwarder | Coordinator | Ceiling |
|---|---|---|---|
| **2–6, offline-capable** | **mesh** — every peer to every peer | **none** | worst peer's uplink; O(n²) streams (§27.7.1) |
| **Dozens, self-sovereign** | your own always-on **box** as SFU | your box (a role, not a server type) | your box's bandwidth/CPU |
| **Hundreds+, global reach** | a **third-party media-relay pool** (LiveKit / Jitsi-Octo / mediasoup cascade) | swappable operator | the pool's, cascaded |

The move from left to right adds **reach**, never **function**: the SFU is a `media-relay`
coordinator anyone can provide, content-blind by construction, so **the host's hardware is not the
size limit** ([DIRECTION §7](../DIRECTION.md)). Because the operator is disclosed and refusable
(R-RTC-3) and holds no key (R-RTC-1), sliding the anchor outward never surrenders sovereignty or
confidentiality — it trades a bounded, disclosed metadata exposure (§7) for scale. The default is
the leftmost cell that works: `topology` absent ⇒ mesh (§27.4.1).

---

## 6. Offline / apocalypse behavior

RTC is measured against the substrate offline grades
([`../substrate/OFFLINE.md`](../substrate/OFFLINE.md) §2). A call is honestly the hardest case for
apocalypse-proofing, because it is intrinsically an **online-both-endpoints** act — and RTC says so
rather than pretending otherwise:

- **On a local mesh with no internet and no coordinator: `full`.** A mesh call is pure peer-to-peer
  — no SFU, no TURN, no cloud. The MLS group and its exporter are local; SFrame keys derive locally;
  ICE finds LAN candidates. Calling **survives a total coordinator outage on a shared segment**,
  which is precisely DIRECTION §6's coordinator-optional property. This is the apocalypse guarantee,
  and it is real.
- **Reaching an offline peer for a *live* call: `blocked`, disclosed.** You cannot place a
  synchronous call to a peer who is not online; no substrate makes simultaneity optional. RTC fails
  **closed and says why** (the peer is unreachable for calls, §27.9 item 1) and MUST NOT fake a
  connection. The async answer to "I called and you were out" is a **voicemail** — an ordinary
  sealed MOTE on the store-and-forward plane, a *different profile*, not a degraded call. RTC does
  not blur the two.
- **Signaling reconciles; §27 defines no offer-freshness bound.** `rtc_signal` MOTEs reconcile on
  reconnect like any MOTE (mailbox drain, §27's ordinary path). An opening `offer` carries a fresh
  `call_id` (§27.4.1), and RTC-2/RTC-3 reject only a replay against an already-known `(call_id,
  sender)` or a non-offer on an unknown one — neither rejects an offer merely for arriving late. A
  `rtc_signal` exchange that reconciles minutes after it was sent will therefore still open and ring
  the call rather than cleanly surface as a **missed call**; RTC does not claim otherwise. A bounded
  offer-freshness check that gives late-reconciled signaling a missed-call disposition is
  unspecified here and left to a future revision.
- **No offline-money case.** A call moves media, not funds. Where a *metered* SFU is used, its usage
  receipts settle on reconnect under the payer's rail (OFFLINE §5 strategy C, settle-on-reconnect,
  the honest default) — the call bytes complete while settlement finality stays deferred. No token
  is minted, ever ([DIRECTION §5](../DIRECTION.md)).

---

## 7. Security & declared content-visibility

RTC inherits the family posture of [`../THREAT-MODEL.md`](../THREAT-MODEL.md); the falsifiable
per-item statements are §27.11. The load-bearing claims:

- **Content-blind media where the operator commits to it (SEC-3).** SFrame keyed from the MLS
  exporter means an SFU/relay that publishes `sframe_required=true` forwards ciphertext it *cannot*
  read — `blind-routing` at `structural` assurance, the strongest level (no key, provable, though
  routing metadata stays visible), not a promise. This is **materially stronger than the legacy mail
  gateway** (§7), which handles plaintext by construction; where the operator has made that
  commitment, the difference is structural, not operator discipline (§27.11 item 1). An operator that
  instead publishes `sframe_required=false` makes no such commitment and is `terminating`, not
  `blind-routing` — see the table below.
- **Every intermediary's visibility is declared (SEC-4, [CONTRACT §3](../coordinator/CONTRACT.md)).**

  | Party | Class | Sees |
  |---|---|---|
  | SFU / media-relay (`sframe_required=true`) | `blind-routing` / structural | ciphertext + routing metadata (participants, timing, IPs, packet sizes) |
  | SFU / media-relay (`sframe_required=false`) | `terminating` | plaintext, if the client offers unprotected media — disclosed to the client before joining (R-RTC-3, §27.7.4) |
  | TURN relay | `blind` / structural | ciphertext + the two hops' addresses |
  | mesh peer | *endpoint*, not intermediary | plaintext (it is a participant) |
  | MCU | `terminating` | plaintext — **out of scope**, never called E2E |

- **Fail-closed, no downgrade (SEC-1, SEC-8).** SFrame ratchets up only; a superseded ICE
  generation is scoped out by `ufrag`; a stale answer MUST NOT revert a session (RTC-3). A member's
  Remove takes media authority with it at the next epoch.
- **Authorize-never-classify (SEC-6).** The only gate is MLS membership. No coordinator inspects,
  scores, re-ranks or drops on a content basis — it holds no key to do so with.

**No metadata privacy is claimed (SEC-9 residual).** A real-time call cannot ride the mixnet — its
per-hop delays are built for a latency budget of minutes (§27.1.2, §27.4.6). The family-normative
guarantee is content-blindness + declared visibility, **not** graph-privacy against a global passive
observer; that is research-tier and non-normative ([`../docs/research/README.md §5`](../docs/research/README.md)).

---

## 8. Honest residual — what RTC genuinely cannot do

Each is an inherent consequence of the design, disclosed rather than solved (§27.11):

1. **The call's existence is not hidden.** An SFU learns who is in the call, when it started and
   ended, how long it ran, who spoke when, who shared a screen, and every participant's IP and rough
   location. If the *existence* of a call is the sensitive fact, do not place the call — no setting
   in this profile changes that (§27.11 item 2).
2. **SFrame keys are scoped to the MLS group, not to who joined.** Any current group member can
   derive the media key whether or not they joined — so excluding someone from a call is a
   forwarding/UX property, **not** a cryptographic one. Cryptographic exclusion requires a group over
   exactly the intended participants; RTC does not auto-create sub-groups, because a silent group is
   a membership change the user never saw (§27.11 item 4, RTC's most-assumed-away limit).
3. **Screen sharing is an unmitigable disclosure surface.** Sharing the wrong window, a notification
   banner, credentials on screen, system audio carrying another call — every one is
   *cryptographically correct* and delivered exactly as intended. There is no protocol mechanism;
   the only remedy is client UX (§27.11 item 5, §27.9 item 5).
4. **Every participant is a plaintext holder and may record.** RTC defines no recording indicator,
   because an indicator a client can simply not send is a false assurance, not a control
   (§27.11 item 6).
5. **Reaching an offline peer live is impossible** — the honest floor of §6. The async substitute is
   a voicemail MOTE, a different profile.
6. **Availability is not a claim.** ICE may fail, TURN may be unreachable, an SFU may refuse on
   capacity. A call that cannot be established is a normal outcome, not a fault (§27.11 item 8).
7. **The mesh↔SFU privacy trade is real, not eliminated.** Mesh leaks peer IPs to each other but
   admits no operator; an SFU hides peer IPs from each other but adds an operator that sees the
   metadata of item 1. RTC discloses both and lets the user choose; it does not pretend a costless
   option exists (§27.11 item 3).
8. **An operator MAY decline content-blindness altogether.** `sframe_required=false` is a
   conformant `RtcCapacity` value (§27.7.4): that SFU is `terminating`, not `blind-routing`, and will
   forward — and can read — unprotected media if the client offers it. Content-blindness is
   therefore an operator commitment a client must check (R-RTC-1, R-RTC-3), not a structural
   property of every SFU.

Every residual traces to the design's premises, not to a missing feature: real-time cannot be
mixnet-private, endpoints hold their own plaintext, and coordinators add reach without ever becoming
load-bearing. Coordinator-optional holds — a mesh call needs none of them — and that is the property
that makes RTC apocalypse-proof at the one scale where it can be.
