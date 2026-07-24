# 27. Real-Time Media Profile — voice, video, multi-party calling & screen sharing

DMTAP carries asynchronous, store-and-forward objects: a MOTE is signed, sealed, queued, retried
and acked (§2, §2.6). A **call** is the opposite shape — a continuous, latency-bounded media flow
between endpoints that are simultaneously online. This appendix specifies **DMTAP-RTC**, the
profile that lets DMTAP identities place voice, video, multi-party and screen-sharing calls to one
another **without adding a media stack to DMTAP**.

It does that by adopting the existing real-time media stack wholesale — WebRTC's transport
(SRTP over DTLS), its session description (SDP), its negotiation model (JSEP), its connectivity
establishment (ICE/STUN/TURN) and its end-to-end media protection (SFrame) — and contributing
exactly **three** things that stack deliberately leaves out:

1. **Signaling.** JSEP is explicitly signaling-agnostic (RFC 8829): it defines *what* an endpoint
   must exchange and says nothing about *how*. That omission is the hole every deployed system
   fills with a proprietary, centrally-operated signaling server. DMTAP is already a signed,
   mutually-authenticated, end-to-end-encrypted substrate with delivery, retry and ack semantics
   (§2.6), so the hole is already filled — it just needs a message kind (§27.3, §27.4).
2. **A media key schedule rooted in MLS.** SFrame needs a key source and does not specify one for
   the general case. DMTAP has one: the MLS group that already carries the conversation (§5.1).
   §27.5 derives SFrame's per-call secret from the MLS **epoch** via the RFC 9420 exporter under a
   DMTAP DS-tag, so a call inherits the group's membership, its epoch advancement and its forward
   secrecy rather than running a second, parallel trust model.
3. **Capability advertisement for limits.** A multi-party call has a hard resource ceiling; §27.7
   makes an operator publish that ceiling as a signed, rollback-protected capability *before*
   admission, so a call is refused up front rather than degraded mid-session.

**What this profile does not specify, and will not.** Media transport, codecs, congestion control,
jitter buffering, packetization, RTP header extensions, simulcast/SVC layer selection, and the
internals of an SFU are **out of scope** — they are specified by the documents cited in §27.2 and
implemented by the media stack, and this profile is deliberately implementable against an SFU that
already exists. No part of this appendix requires a new SFU, a new media protocol, or a change to
any of them.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174,
consistent with the rest of this specification. Where this appendix and §18 (wire format) or §5
appear to differ on a shared mechanism, the more specific rule governs; new objects follow the same
integer-keyed CBOR convention as every other DMTAP object (§18.1.2): keys assigned per object type
starting at `1`, keys **≥ 64 reserved** for future/extension fields, and the signed-vs-unsigned
unknown-key discipline unchanged.

## 27.1 Scope & goals

### 27.1.1 Goals

1. **Parity with a mainstream conferencing product** — 1:1 voice and video, multi-party calls,
   screen sharing, mid-call track add/remove, and graceful teardown — with **no proprietary
   signaling server**, because the signaling server is the single component every such product
   requires and DMTAP already renders unnecessary.
2. **Adopt, do not invent.** Every byte on the media path is a byte some other standards body
   already specified. DMTAP's contribution is a carrier, a key schedule, and a limit
   advertisement — three things, enumerated above, and nothing else.
3. **The SFU never sees plaintext media.** SFrame protects the media payload end-to-end under keys
   derived from the MLS group, so a forwarding unit routes ciphertext it cannot read (§27.5). This
   is a materially stronger posture than the legacy gateways of §7/§26, which see plaintext by
   construction, and §27.11 states exactly how much stronger and where it stops.
4. **Limits are stated before admission, not discovered during the call.** §27.7.4.

### 27.1.2 Non-goals

- **Not a media protocol.** See "What this profile does not specify" above. An implementation that
  wants to change how video is encoded, paced or forwarded changes its media stack, not this
  appendix.
- **Not metadata-private.** A real-time call cannot ride the opt-in, research-tier mixnet
  ([docs/research/mixnet.md §4.4](docs/research/mixnet.md)) and this appendix does
  not pretend otherwise: the mixnet's Poisson per-hop delays, 2 KiB cells and cover traffic are
  designed for a latency budget of minutes (§4.6), which is three orders of magnitude past what a
  conversation tolerates. Calls are a `fast`-tier act by construction (§27.4.6) — which is also the
  default tier for everything else — and the honest consequence is stated in §27.11 rather than
  papered over.
- **Not an MCU.** Server-side mixing is out of scope and is incompatible with this profile's
  central guarantee (§27.7.3).
- **Not a recording, transcription or captioning specification.** A participant that records or
  transcribes does so with plaintext it is entitled to hold as a call participant; the protocol has
  no mechanism to prevent it and this appendix claims none (§27.11 item 6). Where a recording is
  *published*, it becomes an ordinary §24 media work and leaves this profile entirely.
- **Not a replacement for §5.** A call is always scoped to an MLS group that already exists or is
  established by §5.3's ordinary async-join machinery. This appendix defines no new way to meet
  someone.

## 27.2 Normative references — what is adopted, and how sure we are of it

Every row below is machinery this profile **adopts unchanged**. The "Status" column records this
document's understanding at the time of writing; where that understanding is not verified against
the published document, the row says so, and §27.13 collects every such caveat. **An implementer
MUST verify a cited document's current status and text before relying on it**; this table is a
map, not a substitute for reading the RFC.

| Adopted | Document | Used for | Status as understood here |
|---|---|---|---|
| SDP | **RFC 8866** | the session description carried in `RtcSignal.sdp` (§27.4.1) | Proposed Standard; obsoletes RFC 4566 |
| JSEP | **RFC 9429** | the offer/answer state machine, `RTCSdpType`, rollback, renegotiation | Proposed Standard; obsoletes RFC 8829 (Apr 2024); **explicitly signaling-agnostic** (preserved in 9429), which is precisely the seam §27.3 fills. Any reliance on `max-bundle` bundle-policy semantics should be checked against RFC 9429's `max-bundle`→`must-bundle` rename — §27 uses RFC 9143 BUNDLE without naming a policy, so likely unaffected |
| ICE | **RFC 8445** | candidate gathering, connectivity checks, nomination | Proposed Standard; updated by RFC 8863 (PAC timer — failure-declaration *timing* only, not wire format; does not affect DMTAP's stated usage) |
| STUN | **RFC 8489** | server-reflexive candidate discovery | Proposed Standard |
| TURN | **RFC 8656** | relayed candidates; the IP-disclosure mitigation of §27.11 item 3 | Proposed Standard |
| Trickle ICE | **RFC 8838** | incremental candidate exchange (`type = candidate`, §27.4.2) | Proposed Standard; updated by RFC 8863 (PAC timer — failure-declaration *timing* only, not wire format; does not affect DMTAP's stated usage) |
| ICE in SDP | **RFC 8839** | the `candidate:` attribute value carried in `IceCandidate.candidate` | Proposed Standard |
| SDP grouping / `a=mid` | **RFC 5888** | the m-section identifier used by `IceCandidate.mid` and by track purpose (§27.6.2) | Proposed Standard |
| BUNDLE | **RFC 9143** | multiplexing all m-sections on one transport | Proposed Standard; obsoletes RFC 8843 |
| SDP `a=content:` | **RFC 4796** | **track purpose** — camera vs. screen (§27.6.2) | Proposed Standard (2007); the attribute is IANA-registered, which is why §27.6.2 prefers it to an invented field |
| Simulcast in SDP | **RFC 8853**, **RFC 8851** | multi-encoding offers an SFU selects from; referenced, never required | Proposed Standard |
| SRTP | **RFC 3711**, keyed by **RFC 5764** (DTLS-SRTP) | hop protection between an endpoint and its peer or the SFU | Proposed Standard; updated by RFC 7983 / RFC 9443 (multiplexing only; does not affect DMTAP's stated usage) |
| WebRTC security architecture | **RFC 8827**, threat model **RFC 8826** | the baseline this profile strengthens with SFrame | Proposed Standard |
| RTP usage in WebRTC | **RFC 8834** | media transport; adopted, not specified here | Proposed Standard |
| **SFrame** | **RFC 9605** | end-to-end media-payload protection across an SFU (§27.5) | Proposed Standard, published August 2024, no successor/obsoleting RFC (§27.13 item 1). This document does **not** restate SFrame's key schedule, cipher suites, KID encoding or header layout in full, but its MLS-binding section (§5.2) and KID/epoch mechanism (§4) have been checked against §27.5.1's derivation — see §27.13 items 1–3 for the confirmed facts and the one remaining gap (a RECOMMENDED `E` value) |
| MLS | **RFC 9420** (architecture **RFC 9750**) | the group, its epochs, and `MLS-Exporter` (§27.5.1) | Proposed Standard; already DMTAP's group primitive (§5.1) |
| Screen capture API | **W3C Screen Capture** (`getDisplayMedia()`) | the source of a screen track (§27.6) | W3C specification, not an IETF document; cited for the API's existence, not for any wire behaviour |

Congestion control is deliberately absent from this table and from this profile. Endpoints and SFUs
apply whatever the media stack implements (the RMCAT problem space, RFC 8836, and RTCP congestion
control feedback, RFC 8888, are the relevant ground); DMTAP neither specifies nor constrains it.

## 27.3 The structural decision — a core message kind, not profile metadata

This is the most consequential choice in this document and it is stated first, with its reasoning,
because it determines everything downstream.

**Decision (normative).** DMTAP-RTC allocates **one genuine core message kind**,
`0x44 rtc_signal`, from the extension range §21.16 reserves (§2.3), continuing the block
`pub_announce` (`0x40`, §21.24b) opened and DMTAP-PUBSUB (`0x41`–`0x43`, §21.24d) continued. It is
an **ordinary sealed MOTE** riding the existing `Envelope`/`Payload` path (§2.4, §18.3.5), exactly
as `feed_hint`/`feed_subscribe`/`feed_unsubscribe` do and unlike `pub_announce`'s bare signed
object. It does **not** ride profile metadata on an existing kind.

### 27.3.1 Why §24's `meta` precedent does not transfer

§24's engineering-artifact and media facets allocate **no** core message kind: each defines metadata maps embedded in
a `pub_announce`'s `meta` (§24.1). That is the correct answer *for those profiles*, and the reason
it is correct is the reason it does not generalise here.

- **§24's facets had a generic carrier; the sealed path does not.** `pub_announce` is a *general-purpose
  signed public record with an open, text-keyed extension map* (§22). A public-object profile can
  therefore be pure application data over an already-general object, and a node implementing only
  §22 stores and serves every §24 object without parsing a byte of it. The sealed path has no
  equivalent. Its extension surface is `Headers.ext` (§21.20), a *header* map on a message whose
  `kind` already determines what the message **is**. There is no sealed-path object that means
  "arbitrary signed thing, interpret by metadata," so riding `meta` here would mean riding some
  *other* kind's semantics — and that is where it breaks.
- **The unknown-kind rule is exactly the behaviour a call needs, and the unknown-header rule is
  exactly the behaviour it must not have.** §2.3 requires a recipient to **ignore, and specifically
  MUST NOT `ack`**, a `kind` it does not implement (`ERR_KIND_UNKNOWN`, `0x020A`, IGNORE_NO_ACK).
  §21.20 requires the opposite disposition for an unrecognized *header key*: ignore the **token**,
  process the **message**. Carrying an offer as, say, a `chat` MOTE with an `ext` header would
  therefore mean a peer without call support **acks the offer as delivered** and renders an empty
  chat message to its user. The caller sees a delivered offer that will never be answered; the
  callee sees a blank message. With a distinct kind the same peer ignores it without acking, the
  caller's delivery state machine (§20.1) reports exactly what happened, and no ghost message is
  ever displayed. This is not a stylistic preference — it is a concrete, user-visible failure that
  only the kind allocation avoids.
- **The privacy tier is a per-kind default, and this kind's default must be pinned regardless of
  what a peer has opted into.** §4.6's default tier for mail and control messages is `fast`
  (direct/low-hop); the opt-in, research-tier `private` mixnet
  ([docs/research/mixnet.md §4.4](docs/research/mixnet.md)) is a deliberate, user-surfaced choice a
  peer may make for its *other* traffic. A signaling exchange whose offer/answer round trip took
  minutes is not a call at all (§27.4.6), so call signaling MUST ride `fast` even for a peer that
  has otherwise opted into `private` — kind is the field DMTAP already uses to pin a message's tier
  independently of what its sender's general preference is; a metadata field cannot carry that
  override, because by the time the metadata is parsed the routing decision has been made.
- **`kind` is bound into both signatures; a metadata field is bound into one.** `Envelope.kind` is
  covered by `sender_sig` (§18.9.1) *and* by `Payload.sig` (§18.9.2, enforced at
  `ERR_ENVELOPE_CONTEXT_MISMATCH`, `0x0211`). A relabel of a signaling message into another kind is
  therefore already detected by machinery that exists. Application metadata inside the ciphertext
  gets the second binding but not the first, so an intermediary's view of "what is this traffic"
  would come from a field the anti-abuse gate (§2.7 step 6) cannot see.
- **Capability gating and rate limiting need a pre-decryption discriminator.** A node that does not
  offer calls, or a user who has turned them off, should refuse signaling traffic without
  decrypting it. `kind` is available at §2.7 step 3; metadata inside `Payload` is not available
  until step 7.

The cost of the decision is honestly one code point out of the 64-point extension range (§21.16),
of which this is the fifth used. That is the entire cost, and §27.3.2 keeps it to one.

### 27.3.2 Why one kind, not four

Signaling has at least five distinguishable acts — offer, answer, ICE candidate, renegotiate,
teardown — and an obvious design gives each a kind. This profile allocates **one**, with an inner
`type` discriminator (§27.4.2), for three reasons:

- **JSEP already owns the discriminator.** `RTCSdpType` (`offer` / `pranswer` / `answer` /
  `rollback`, RFC 8829) is a fact of the adopted stack. Allocating kinds that duplicate it creates
  two places where the same fact is written, and §24.4.2 already settled what this specification
  does about that: *a duplicated fact can disagree with itself, and whichever side is declared the
  winner, the other becomes a lie a sender can tell for free.* A `kind = rtc_answer` MOTE carrying
  SDP whose JSEP type is `offer` would have no defined resolution. With one kind there is one
  discriminator and no disagreement is expressible.
- **The transport treatment is identical for every type.** All five ride the same sealed path, the
  same tier, the same ack/retry, the same capability gate and the same rate budget. Kinds exist to
  separate messages the *transport* treats differently (§2.3); these are not.
- **"Renegotiate" is not a distinct act at all.** In JSEP, renegotiation *is* an ordinary
  offer/answer exchange on an existing session — which is exactly why adding or removing a screen
  track needs no new machinery (§27.6.1). Allocating a `renegotiate` kind, or even a
  `renegotiate` type, would name something the adopted standard does not contain.

### 27.3.3 What this profile does not add to the core

Beyond the one kind, DMTAP-RTC adds **no new signed structure and no new signature DS-tag**. An
`RtcSignal` is authenticated by the existing `Payload.sig` preimage (§18.9.2) and made confidential
by the group's MLS epoch key (§2.4) — the same two mechanisms that protect a chat message. The one
new domain-separated string this profile introduces is a **KDF label**, not a signature context
(§27.5.1), which is why §18.9's signature-preimage inventory gains a *note* rather than a
subsection (§27.8). It allocates **no new error subsystem byte**: its three error codes are new
points inside subsystem `0x04` (Messaging & Group), which §21 already owns, under the same
lighter-weight policy §21.24d used to extend subsystem `0x09` (§27.12).

## 27.4 Signaling

### 27.4.1 The `RtcSignal` object (kind `0x44`)

An `RtcSignal` is carried as the `Payload.body` of a `kind = 0x44` MOTE addressed to the MLS group
that scopes the call (§2.4). A 1:1 call is a call in a 2-member group (§5.1); there is no separate
1:1 path.

```cddl
RtcSignal = {
  1 => bstr .size 16,        ; call_id      per-call identifier, REQUIRED
  2 => u8,                   ; type         signal type (§27.4.2), REQUIRED
  3 => u64,                  ; seq          per-(call_id, sender) counter, REQUIRED
  ? 4 => tstr,               ; sdp          RFC 8866 session description
  ? 5 => [+ IceCandidate],   ; candidates   trickle-ICE candidates (RFC 8838)
  ? 6 => u64,                ; mls_epoch    the MLS epoch the sender held when emitting
  ? 7 => u8,                 ; reason       teardown reason (§27.4.5)
  ? 8 => u8,                 ; topology     1 = mesh, 2 = SFU (§27.7)
  ? 9 => ik-pub,             ; sfu          the SFU's identity key, iff topology = 2
  ? 10 => suite,             ; sfu_suite    the suite governing key 9's length (§18.1.4, §18.2)
}

IceCandidate = {
  1 => tstr,                 ; candidate    the RFC 8839 "candidate:"-prefixed attribute value
  2 => tstr,                 ; mid          the m-section this candidate belongs to (RFC 5888)
  ? 3 => tstr,               ; ufrag        the ICE ufrag it was gathered under (RFC 8839)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `call_id` | 1 | `bstr .size 16` | MUST | 16 bytes from a CSPRNG, chosen by the offerer. Scopes the call within the group; it is **not** a content address and carries no structure. A sender MUST NOT reuse a `call_id` across calls, and MUST NOT derive one from group state — it is an input to the media key schedule (§27.5.1), so a reused value re-uses media keys across calls. |
| `type` | 2 | `u8` | MUST | One of §27.4.2. An unrecognized value MUST cause the `RtcSignal` to be discarded without applying any part of it, and MUST NOT be treated as a fault of the peer (§27.9). |
| `seq` | 3 | `u64` | MUST | Strictly increasing per `(call_id, sender)`. Ordering rules in §27.4.5. |
| `sdp` | 4 | `tstr` | MUST **iff** `type ∈ {offer, pranswer, answer}`; MUST be absent otherwise | The complete RFC 8866 session description as JSEP produced it. DMTAP MUST NOT rewrite, reorder, canonicalize or otherwise normalise it — it is opaque application content to every DMTAP layer, and the only party that parses it is the peer's media stack. |
| `candidates` | 5 | `[+ IceCandidate]` | MUST **iff** `type = candidate`; MUST be absent otherwise | One or more trickle-ICE candidates (RFC 8838). At least one — an empty array is malformed; end-of-candidates is its own type. |
| `mls_epoch` | 6 | `u64` | SHOULD on `offer`/`answer` | The MLS epoch the sender held when emitting. Advisory: it lets a receiver detect that the two sides are keying media from different epochs before media fails, rather than after (§27.5.2). A receiver MUST NOT treat this field as authorisation for anything, and MUST NOT key media from it — the authority is the epoch the receiver itself has applied. |
| `reason` | 7 | `u8` | MUST **iff** `type = bye` | §27.4.5. |
| `topology` | 8 | `u8` | SHOULD on `offer` | `1` = mesh (§27.7.1), `2` = SFU (§27.7.2). Absent ⇒ mesh. A mid-call topology change is a renegotiation the peer may refuse (§27.7.2). |
| `sfu` | 9 | `ik-pub` | MUST **iff** `topology = 2` | The DMTAP identity key of the forwarding unit, so a participant knows *which* operator is in the path before it sends a byte of media. An `ik-pub` carried exactly as §24.3.1 carries one — raw public-key bytes, never a digest — with key 10 governing its length. |
| `sfu_suite` | 10 | `suite` | MUST **iff** key 9 is present | The §18.1.4 suite whose §18.2 row governs key 9's length. Present for the same reason `Rendition.suite` is (§24.4.3): the SFU is by design a third party whose suite need not equal the group's. |

**Forward compatibility (normative).** A client MUST ignore unrecognized integer keys in
`RtcSignal` and `IceCandidate`, MUST NOT treat their presence as fatal, and — because these objects
are **not** re-relayed, re-serialized or content-addressed by any party (each is consumed by its
recipient and discarded) — is under **no** obligation to preserve them. This differs deliberately
from §24.4's byte-retention rule, which exists because a `pub_announce` is a durable,
content-addressed object whose bytes a mirror must not perturb; an `RtcSignal` has neither
property. Keys **≥ 64** are reserved as §18.1.2 reserves them generally; currently-unallocated keys
**< 64** are reserved to this profile's allocation and a client MUST NOT assign one privately.

### 27.4.2 Signal types (`RtcSignal.type`, `u8`)

This table is authoritative for the type space, exactly as §22.10 is authoritative for its error
codes. Values `0x01`–`0x3F` are Specification Required; `0x40`–`0xFE` are Private Use; `0x00` and
`0xFF` are Reserved.

| Value | Name | Carries | Meaning |
|------:|------|---------|---------|
| `0x01` | `offer` | `sdp` | A JSEP offer (`RTCSdpType` `offer`, RFC 8829). The **first** offer for a `call_id` opens the call; a **subsequent** offer on the same `call_id` is a renegotiation — a track added or removed, a codec change, an ICE restart. There is no separate renegotiate type (§27.3.2). |
| `0x02` | `pranswer` | `sdp` | A JSEP provisional answer. OPTIONAL to originate; a receiver that does not implement provisional answers treats it as an unrecognized type and waits for `answer`. |
| `0x03` | `answer` | `sdp` | A JSEP answer, completing the exchange opened by the highest-`seq` `offer` from the peer. |
| `0x04` | `rollback` | — (`sdp` MUST be absent) | Withdraws the sender's own outstanding offer, per JSEP's rollback semantics (RFC 8829). The glare resolution of §27.4.4 is expressed with this type. |
| `0x05` | `candidate` | `candidates` | One or more trickle-ICE candidates (RFC 8838). |
| `0x06` | `end_of_candidates` | — | The sender has finished gathering for this negotiation. A distinct type rather than a sentinel candidate string, so the "no more candidates" fact is not encoded as the absence of a value inside a field whose grammar RFC 8839 owns. |
| `0x07` | `bye` | `reason` | Teardown (§27.4.5). |

### 27.4.3 Trickle ICE candidates

`IceCandidate.candidate` (key 1) carries the RFC 8839 attribute value **including** its
`candidate:` prefix, verbatim as the ICE agent produced it. `mid` (key 2) identifies the m-section
by its RFC 5888 `a=mid` value, **never** by m-line index: an index is invalidated by the very
renegotiations §27.4.2 makes routine, while a MID is stable for the life of the m-section. `ufrag`
(key 3), when present, is the ICE ufrag the candidate was gathered under; a receiver that holds it
MUST discard candidates whose `ufrag` does not match the current negotiation's, which is how a
candidate from a superseded ICE generation is prevented from being applied after an ICE restart.

DMTAP does not parse, validate or rewrite any of these strings. A malformed candidate is a media-
stack error at the receiving endpoint, not a DMTAP wire fault, and MUST NOT tear down the call: the
candidate is dropped and negotiation continues with the remainder (ICE is explicitly tolerant of
individual candidate failures, RFC 8445).

### 27.4.4 Offer/answer, renegotiation and glare (normative)

The offer/answer state machine is JSEP's (RFC 8829), adopted unchanged. Three points where a
signaling-agnostic standard leaves a hole that a *transport* must fill are specified here, and only
these three:

- **Renegotiation is an ordinary `offer` with a higher `seq` on an existing `call_id`.** Adding a
  screen share, removing it, adding a second camera, restarting ICE, and changing simulcast layers
  are all this one operation. An endpoint MUST accept an `offer` on a `call_id` it already holds in
  an established state and MUST process it as a renegotiation, not as a new call.
- **Glare resolution reuses the committer rank; it does not invent a tie-break.** When both peers
  hold an outstanding `offer` on the same `call_id` and neither has answered, the peer with the
  **higher per-epoch rank** — the §18.9.16 committer-rank preimage, computed over the same inputs
  §5.1 uses — is *impolite*: it keeps its own offer and ignores the peer's. The **lower**-ranked
  peer is *polite*: it MUST emit `rollback` (`0x04`), discard its own offer, and apply the peer's.
  This is WebRTC's "perfect negotiation" pattern with DMTAP's existing deterministic ordering
  substituted for an ad-hoc one; because the rank folds in the group id and the epoch, neither peer
  can grind a key to occupy the polite or impolite seat across epochs (§5.1). In a group of more
  than two, glare is resolved pairwise on the same rule.
- **An answer answers exactly one offer.** A receiver MUST apply an `answer` only against the
  highest-`seq` `offer` it has emitted for that `call_id` and that is still outstanding; an
  `answer` arriving for a superseded or already-answered offer MUST be discarded. Without this
  rule, a delayed answer from an earlier negotiation silently reverts a session to a prior media
  configuration — including, concretely, re-enabling a screen share the user had just stopped.

### 27.4.5 Authorization, ordering and teardown (normative)

- **Group membership is the authorisation.** An endpoint MUST accept an `RtcSignal` only if
  `Payload.from` is a **current member** of the MLS group under whose epoch key the MOTE was
  decrypted (§2.7 steps 7–8), evaluated against the receiver's own applied group state, never
  against any claim in the signal. A signal from a non-member — including a former member whose
  Remove the sender has applied but the receiver has not yet, and vice versa — MUST be rejected
  with `ERR_RTC_SIGNAL_UNAUTHORIZED` (`0x0415`, FAIL_CLOSED_BLOCK). Nothing in this profile grants
  a call participant any authority the group does not already grant; in particular, per-participant
  moderation (mute, eject) is **group-role machinery** (§5.8.2, `ERR_GROUP_POLICY_VIOLATION`
  `0x0409`), not a new authority this profile defines.
- **A call is opened only by an `offer`.** An `RtcSignal` naming an unknown `call_id` with any type
  other than `offer` MUST be discarded. This is what prevents a member from injecting candidates or
  a `bye` into a call that does not exist in order to probe or to pre-poison one that is about to.
- **Ordering.** `seq` is strictly increasing per `(call_id, sender)`. A receiver MUST discard an
  `offer`, `pranswer`, `answer`, `rollback` or `bye` whose `seq` is **≤** the highest `seq` it has
  already applied *of those types* from that sender for that call. It MUST NOT apply that rule to
  `candidate`/`end_of_candidates`: ICE is order-insensitive by design (RFC 8838), candidates
  legitimately arrive out of order, and discarding a late candidate costs connectivity for no
  security gain — the `ufrag` check of §27.4.3 is the mechanism that scopes candidates to a
  generation. §2.6's envelope-level dedup handles duplicate delivery of the same MOTE and is
  unaffected.
- **Teardown.** `bye` (`0x07`) ends the sender's participation and MUST carry `reason` (key 7):
  `0` normal, `1` declined, `2` busy, `3` timeout, `4` capacity (§27.7.4), `5` error. An endpoint
  that receives `bye` from the last remaining peer MUST tear down the call and MUST delete the
  media keys of §27.5.2. `reason` is advisory display data and MUST NOT gate any security decision;
  a peer that simply disappears is handled by the media stack's ICE consent-freshness timers, not
  by this field, so a missing `bye` is a normal outcome and not an error.
- **Malformed objects.** An `RtcSignal` that fails to decode against the CDDL of §27.4.1 — a
  missing `sdp` on an SDP-bearing type, an `sdp` on a type that forbids it, an empty `candidates`
  array — is `ERR_MALFORMED_OBJECT` (`0x020D`, DROP_SILENT), the disposition §21.4 already assigns
  to any object that fails its schema.

### 27.4.6 Tier: signaling always rides `fast`; a reduction from an established `private` relationship is disclosed, not silent (normative)

§4.6's default tier for mail and control messages is `fast` (direct/low-hop). The opt-in,
research-tier `private` mixnet ([docs/research/mixnet.md §4.4](docs/research/mixnet.md)) is a
deliberate, user-surfaced choice a peer may make for its *other* traffic, never a default. `rtc_
signal` is a control message and this profile pins its tier to `fast` **unconditionally** —
including for a contact whose other traffic has opted into `private` — because real-time media has
no `private`-tier construction to use instead. That pin, and its one user-visible consequence, are
stated explicitly rather than left to be discovered:

- **A mixnet-carried call is not a call.** The opt-in `private` tier's latency budget is minutes
  (§4.6, §16.3); an offer/answer round trip plus trickled candidates over that path would place
  call setup in the tens of minutes, and media over it is not merely slow but structurally
  impossible ([docs/research/mixnet.md §4.4](docs/research/mixnet.md)'s cell size, padding and
  cover-traffic construction is not a media transport). This is not a preference; there is no
  `private`-tier design for real-time media to downgrade *from*.
- **This is NOT a [docs/research/mixnet.md §4.4.9](docs/research/mixnet.md) downgrade, and MUST
  NOT be implemented as one.** `ERR_PRIVATE_TIER_
  DOWNGRADE_REFUSED` (`0x0310`) polices a *message that could have gone private* being routed
  `fast` instead. A `rtc_signal` has no `private` construction, so nothing is being downgraded.
  What an implementation MUST NOT do is take this clause as licence to route *other* kinds `fast`
  because a call is in progress: the tier default is per-kind, and every other kind keeps its own.
- **Placing a call to a contact whose relationship runs `private` MUST be an explicit user act.**
  Because that contact's other traffic runs on the opt-in `private` tier, a call to them is a
  visible reduction from that established posture, not merely the ordinary `fast` default. It MUST
  NOT be automatic, MUST NOT be a fallback from a failed private send, and the client MUST
  surface — before the first signaling MOTE leaves — that a call reveals the call's existence, its
  timing and its duration to observers, and its endpoint addresses to the peer or the SFU (§27.11).
  Refusing is the default if the user does not consent. This is the same shape as §5.2.1(d)'s
  no-silent-deniability-downgrade and satisfies §10.7.5's governing rule: a security-relevant
  reduction is either refused or surfaced as a deliberate choice, never a silent degradation.

## 27.5 Media protection — SFrame keyed from the MLS epoch

SRTP (RFC 3711, keyed by DTLS-SRTP, RFC 5764) protects each **hop**. In an SFU topology that is not
end-to-end: the SFU terminates DTLS-SRTP and can read every frame it forwards. **SFrame** (RFC
9605) closes that by encrypting the media payload once at the sender, under a key the forwarding
unit does not hold, leaving only the headers an SFU needs for routing in the clear. This profile
adopts SFrame and specifies exactly one thing about it: **where its key comes from.**

### 27.5.1 Deriving the SFrame secret from the MLS epoch (normative)

The call's SFrame secret is derived from the MLS group's epoch via RFC 9420's exporter interface:

```
sframe_epoch_secret =
    MLS-Exporter( Label   = "DMTAP-RTC-v0/sframe",
                  Context = det_cbor([ call_id ]),
                  Length  = Nk )

    ; MLS-Exporter is RFC 9420 §8.5, evaluated against the group's CURRENT epoch:
    ;   MLS-Exporter(Label, Context, Length)
    ;     = ExpandWithLabel( DeriveSecret(epoch_secret, "exporter"),
    ;                        Label, Hash(Context), Length )
    ;   — computed by the implementation's MLS library, never re-implemented here.
    ;
    ; call_id = RtcSignal key 1, the 16 bytes the offerer chose (§27.4.1)
    ; Nk      = the key length of the SFrame cipher suite in use (RFC 9605)
```

Normatively:

- An endpoint MUST obtain `sframe_epoch_secret` through its MLS implementation's **exporter API**
  and MUST NOT reconstruct it from `epoch_secret`, an application-visible copy of the key schedule,
  or any other path. Re-deriving a key schedule by hand is how two conformant implementations stop
  agreeing.
- The `Context` MUST be `det_cbor([ call_id ])` — a **fixed one-element array**, deterministically
  encoded (§18.1.1), not the bare 16 bytes. The array is fixed-arity for the reason §24.4.4 fixes
  its statement's arity: a future revision that needs to bind a second value must be unable to
  produce a context that collides with a v0 one, and an array whose length is part of its encoding
  cannot.
- The secret MUST be **per-call**. Two concurrent calls in one group MUST derive distinct secrets,
  which the `call_id` in the context achieves and which is why §27.4.1 forbids reusing a `call_id`.
- **DMTAP's contribution ends here.** `sframe_epoch_secret` is the shared secret from which
  SFrame's own key schedule (RFC 9605) produces per-sender base keys, salts and key identifiers.
  This profile does **not** restate, profile or vary that schedule, does not define a KID layout,
  and does not select a cipher suite — the media stack does, and an endpoint MUST use SFrame's
  specified derivation from this input rather than any DMTAP-local elaboration of it.
- The sender MUST encode enough of the MLS epoch in the SFrame key identifier for a receiver to
  select the right epoch's key material, using SFrame's own key-identifier mechanism. The exact
  encoding is RFC 9605's, not this profile's; see §27.13 item 3 — confirmed sufficient, but this
  profile does not yet state a RECOMMENDED `E`.
- `sframe_epoch_secret` and everything derived from it are **media keys only**. An implementation
  MUST NOT use them to authenticate signaling, to authorise a participant, or as an input to any
  other DMTAP key schedule.

**What this inherits, for free.** Because the secret is an MLS exporter output at a given epoch, a
call automatically inherits the group's **membership** (only members can derive it), its **epoch
advancement** (an Add/Remove/Update Commit re-keys the media, §27.5.2), and its **forward secrecy**
(a past epoch's media secret is unrecoverable from present state once deleted, §5.2, SP-6 in §6.9).
None of that is a new mechanism; all of it is MLS's, reached through one label.

### 27.5.2 Epoch advance, retention and deletion (normative)

An MLS Commit during a call — someone joins the group, leaves it, or rotates a device key —
advances the epoch and therefore changes `sframe_epoch_secret`. Handling that is where forward
secrecy is either delivered or quietly lost:

- On applying a Commit, a sender MUST begin protecting **new** frames under the new epoch's secret.
  It MUST NOT continue emitting under the old one.
- A receiver MUST retain the **immediately preceding** epoch's derived media keys for a bounded
  reorder window — RECOMMENDED **30 s**, a value §16 should adopt (§27.8) — so that media already
  in flight when the Commit landed still decrypts. It MUST NOT retain more than **two** epochs'
  material (current + previous).
- At the end of the window, and unconditionally at call teardown, an endpoint MUST **delete**
  `sframe_epoch_secret` and every key derived from it. **Failing to delete is the single way an
  implementation can pass every other rule in this section and still not have forward secrecy**:
  the MLS ratchet has already discarded the epoch, so the only surviving copy of the media key is
  the one the application kept.
- A frame whose key identifier resolves to no retained epoch MUST be **discarded**. An endpoint
  MUST NOT request a group re-key per undecryptable frame — that converts a lossy network into a
  Commit storm. Sustained failure (a receiver that decrypts nothing for a call after applying the
  current epoch) is `ERR_EPOCH_MISMATCH` (`0x0406`, HOLD_RESYNC): resync group state once, then
  continue or tear down.
- An endpoint MUST NOT emit or forward unprotected media on a call that negotiated SFrame, and MUST
  NOT accept a renegotiation that removes SFrame protection from an established call. Either is
  `ERR_RTC_SFRAME_REQUIRED` (`0x0417`, FAIL_CLOSED_BLOCK). This is the media-layer analogue of
  §1.3's suite ratchet and §5.1's MLS-ciphersuite high-water-mark (`ERR_MLS_CIPHERSUITE_DOWNGRADE`,
  `0x0414`): protection ratchets up, never down, and never silently.

### 27.5.3 What SFrame covers, and what it does not (normative)

SFrame protects the media **payload**. It does not protect, and this profile does not claim it
protects:

- **RTP/SRTP headers, sequence numbers, timestamps, SSRCs and packet sizes.** These are exactly
  what an SFU needs to forward, and their exposure is the design, not a defect. §27.11 item 2
  quantifies what an observer learns from them.
- **RTCP feedback.** Receiver reports, PLI/FIR, bandwidth estimates and the rest are visible to the
  SFU; that visibility is what lets it do congestion-responsive forwarding at all.
- **The signaling.** Signaling is protected separately and more strongly — it is a sealed MOTE
  (§2.4), so an SFU that is not a group member never sees SDP, candidates or `call_id` unless a
  participant chooses to send them to it out of band as part of establishing the media path.

## 27.6 Screen sharing

### 27.6.1 A track, not a call type (normative)

A screen share is an **additional media track on an existing call**, never a distinct call type,
session, or object. `getDisplayMedia()` yields an ordinary video track; adding it mid-call is a
JSEP renegotiation, which is an `offer` with a higher `seq` on the existing `call_id` (§27.4.2,
§27.4.4). Stopping the share is the same operation in reverse. Consequently this profile defines
**no** screen-share message, **no** screen-share state, and **no** screen-share capability token —
an endpoint that implements `rtc-1` and JSEP renegotiation already implements screen sharing, and
an endpoint that cannot render a second video track simply does not attach it.

### 27.6.2 Track purpose is signalled in SDP, not in DMTAP (normative)

A receiver must be able to tell a camera track from a screen track: they are laid out differently,
sized differently, and a screen track that a client renders as a thumbnail is unreadable while a
camera track rendered full-frame is absurd.

**Decision.** Track purpose is carried by the **SDP `a=content:` attribute (RFC 4796)**, per
m-section. `a=content:main` denotes a camera/primary track; **`a=content:slides` denotes a screen
or presentation track**. DMTAP defines **no** track-purpose field of its own, and an `RtcSignal`
carries none.

Normatively: an endpoint that adds a display-capture track MUST include `a=content:slides` on that
track's m-section in the SDP it signals, and MUST include `a=content:main` on camera m-sections
whenever it includes `a=content:` on any m-section at all. A receiver MUST read purpose from the
remote description's `a=content:` and MUST treat an m-section with no `a=content:` as `main` — the
RFC 4796 default reading, and the correct one for interoperating with an endpoint that predates
this profile.

**Why SDP-native rather than a DMTAP field.**

- **One fact, one place.** A `tracks` map in `RtcSignal` would restate what the SDP already
  describes, and the two can disagree — a manifest declaring one thing while the negotiated media
  says another, with no defined resolution. §24.4.2 rejected a `kind` discriminator on exactly this
  reasoning ("a duplicated fact can disagree with itself"), and the same reasoning applies with
  more force here, because the SDP is the description the media stack actually acts on.
- **It travels with the m-section, atomically.** Purpose changes only through renegotiation, and
  renegotiation replaces the whole session description. There is no window in which the purpose
  field and the track set are out of step, and no ordering rule needed to keep them in step. A
  parallel DMTAP field would need one.
- **It keeps DMTAP thin, which is the whole premise.** An `RtcSignal` stays a carrier for an opaque
  SDP string (§27.4.1). The moment DMTAP describes tracks, it acquires a per-track schema that must
  track the media stack's evolution — simulcast, SVC, multiple encodings per sender — forever.
- **It is IANA-registered ground.** `a=content:` is a registered SDP attribute from a Proposed
  Standard (RFC 4796), with `slides` and `main` among its defined values. Adopting it invents
  nothing; a DMTAP field would invent a vocabulary.
- **An existing SFU can use it without knowing DMTAP exists.** An SFU already parses SDP. Layout
  hints, bitrate allocation and "who is presenting" policy are available to it from the session
  description it already handles. A DMTAP-specific field would be invisible to every SFU not
  written for DMTAP — and §27.1 requires this profile to be implementable against one that was not.

**Disclosed cost, honestly.** No browser API sets `a=content:` from `getDisplayMedia()`, so an
endpoint must insert the attribute into the SDP it generates and read it from the SDP it receives.
JSEP discourages modifying a generated description, and **whether a given engine accepts a local
description carrying an added `a=content:` line is engine-dependent and is not verified in this
document** (§27.13 item 4). Where an engine rejects it, the correct remedy is to fix or work around
the engine — not to add a parallel DMTAP field, which would trade a bounded implementation problem
for a permanent two-sources-of-truth problem.

### 27.6.3 Multiple simultaneous shares

Nothing in this profile limits a call to one screen share, and a conformant implementation MUST NOT
assume there is one. Each share is its own m-section carrying `a=content:slides`, identified by its
RFC 5888 `a=mid` and distinguishable by the sender's identity from the group's own membership; a
receiver lays out as many as it is offered. Two consequences:

- A client MUST distinguish concurrent shares by MID (and by sender), never by "the screen track,"
  and MUST NOT replace an existing share's rendering when a second arrives.
- Concurrent shares are the case that most decisively breaks a headcount-based capacity model,
  which is why §27.7.4 counts tracks and bitrate instead.

### 27.6.4 Tab and system audio capture

`getDisplayMedia()` may, depending on platform and user choice, also yield an **audio** track (tab
audio, application audio, or system audio). It is **in scope** and requires nothing new:

- Captured display audio is an ordinary audio track and MUST be offered as its **own m-section**,
  separate from the microphone's. Mixing it into the microphone track at the sender destroys the
  receiver's ability to control the two independently and is a common, avoidable product defect.
- That m-section SHOULD carry `a=content:slides`, matching the video it accompanies, so a receiver
  can route it with the share rather than with the sharer's voice.
- **Availability is a platform property, not a protocol one.** Tab audio, application audio and
  full system audio have materially different availability across operating systems and browsers,
  and an endpoint MUST treat the absence of a display-audio track as normal rather than as an error
  or as a peer fault.
- **The privacy consequence is the same as the video's and is not smaller.** Shared system audio
  can carry notification sounds, other calls and unrelated media; §27.11 item 5 covers it together
  with the visual surface.

### 27.6.5 SFrame applies identically to screen tracks (normative)

A screen track — and a display-audio track — is protected by **SFrame under the same MLS-derived
keys, by the same rules, as a camera or microphone track** (§27.5). There is no weaker path for
screen content, no separate key, no "presentation mode" that forwards plaintext for the SFU's
convenience, and no exception for a track the SFU is asked to transcode or thumbnail. An SFU
handling a screen track sees the same ciphertext it sees for every other track and MUST NOT be
granted media keys in order to process one.

This is stated explicitly because the assumption runs the other way in practice: screen content is
the stream most often special-cased for server-side processing (thumbnailing, layout compositing,
OCR, "presenter view"), and every one of those special cases requires plaintext. Any of them is a
`ERR_RTC_SFRAME_REQUIRED` (`0x0417`) condition under §27.5.2, not a feature. An operator that wants
server-side processing of screen content is asking for an MCU, which §27.7.3 puts out of scope for
the same reason.

## 27.7 Topologies and limits

Two topologies are in scope. Both carry SFrame-protected media (§27.5); they differ in who forwards
it and therefore in how they scale and what they leak.

### 27.7.1 Mesh (node-hosted)

Every participant sends its media directly to every other participant. For *n* participants each
sending one track, there are **n(n−1)** streams across the call and each participant sustains
**n−1** uplink copies of its own media — uplink cost grows **linearly per participant** and stream
count grows **quadratically across the call**.

- **No third party is in the media path at all.** This is the strongest available posture and the
  reason mesh is the default (`topology` absent ⇒ mesh, §27.4.1): no operator, not even one running
  SFrame-blind, learns the call exists from the media path.
- **The realistic ceiling is 3–6 participants**, and it is set by the *worst* participant's
  hardware and **uplink**, not by anything in this protocol — asymmetric consumer uplinks are
  usually the binding constraint, and a single 720p sender at 1.5 Mb/s needs 7.5 Mb/s of uplink at
  six participants before any screen share is added. An implementation SHOULD refuse to *grow* a
  mesh call past a configured participant count (RECOMMENDED default **6**) and SHOULD offer an SFU
  renegotiation instead; it MUST NOT switch topology unilaterally (§27.7.2).
- **Mesh leaks participants' IP addresses to each other** through ICE, which is the significant
  privacy cost and is disclosed in §27.11 item 3 rather than buried here.

### 27.7.2 SFU (host-run — your own box or an operator)

Each participant sends **one** copy of each of its tracks to a Selective Forwarding Unit, which
forwards to the others. Per-participant cost is **O(n)** downlink and **O(1) per track** uplink;
the call scales to dozens of participants and beyond, bounded by the **host's** bandwidth and
CPU rather than by any participant's. The SFU is **just another node role** (§0.2), not a
privileged server type: the host MAY be a participant's **own always-on box** — the sovereign
default an operator can never silently displace — or a **third-party operator**, identified the
same way either way. (It is an "SFU host," never a "gateway": it forwards SFrame ciphertext, §27.5
— the opposite of the plaintext-handling §7/§26 legacy gateway, and conflating the two words is the
confusion this heading avoids.)

- The SFU is identified in the offer by its DMTAP identity key (`RtcSignal` keys 9/10, §27.4.1), so
  a participant knows **which host** — its own box or a third party — will be in its media path
  **before** it sends media, and can refuse.
- **The SFU never decrypts media** (§27.5). It forwards SFrame ciphertext, selecting streams and
  simulcast layers from the headers and RTCP it can see. This is what makes an SFU a fundamentally
  different kind of intermediary from the legacy-mail gateway of §7/§26, which handles plaintext by
  construction — and §27.11 item 1 states the difference and its limit precisely.
- **A topology change is a renegotiation the peer may refuse.** An endpoint MUST NOT move an
  established call from mesh to SFU (or between SFUs) without an `offer` carrying the new
  `topology`/`sfu`, and a peer MUST be able to decline by responding `bye` with `reason = 5`.
  Silently interposing a forwarding unit mid-call would add an operator to the media path without
  the user's knowledge, which is the one thing a self-hosted substrate must never do.

### 27.7.3 MCU / server-side mixing is out of scope (normative)

An MCU decodes participants' streams, composites or mixes them, and re-encodes a single output.
**It therefore requires media plaintext by construction**, which means holding the SFrame keys,
which means the operator is a member of the confidentiality boundary the whole of §27.5 exists to
draw. This profile does not specify an MCU, and an implementation MUST NOT describe an
MCU-mediated call as end-to-end encrypted. If an operator offers mixing, that is a different
service with a different, weaker guarantee, and it must be presented to the user as such — the same
discipline §24.14 applies to a server attestation being displayed with a weaker badge than an
author signature.

### 27.7.4 `RtcCapacity` — the signed capacity advertisement (normative)

An operator offering the SFU role MUST publish its ceiling **before** admitting anyone. The
ceiling rides existing machinery and adds no new signed structure: it is carried in an ordinary
`system` MOTE (`kind = 0x0A`) alongside the `rtc-sfu-1` capability token (§10.2, §21.22), so it is
covered by `Payload.sig`, and it is rollback-protected by the monotonic `caps_version` rule
(`ERR_CAPABILITY_ANNOUNCE_ROLLBACK`, `0x030A`) that already governs every capability announcement.

```cddl
RtcCapacity = {
  1 => u32,     ; max_tracks                  simultaneous forwarded tracks per call, REQUIRED
  2 => u64,     ; max_aggregate_bps           aggregate forwarded bitrate per call, bits/s, REQUIRED
  3 => u32,     ; max_tracks_per_participant  REQUIRED
  4 => u64,     ; max_bps_per_track           REQUIRED
  ? 5 => u32,   ; advisory_max_participants   DISPLAY ONLY — never the admission basis
  6 => bool,    ; sframe_required             true iff this operator refuses to forward
                ;                             media that is not SFrame-protected, REQUIRED
}
```

**The ceiling is expressed in tracks and bandwidth, not in headcount.** This is the load-bearing
choice in this subsection. A screen share is typically the **highest-bitrate stream in a
conference** — a 1080p30 screen share of moving content can exceed the aggregate of every camera
in a six-person call — so six participants with two shares can cost several times what six
participants on camera cost. A `max_participants` bound is therefore not a bound on the resource
that actually runs out, and an operator that publishes one is publishing a number that is wrong in
both directions: too permissive for a call with shares, too restrictive for an audio-only one.

Normatively:

- Admission MUST be evaluated against keys **1–4**. An operator MUST refuse — with
  `ERR_RTC_CAPACITY_EXCEEDED` (`0x0416`, DENY_POLICY) — any admission or renegotiation whose
  resulting totals would exceed any of them. Because *renegotiation* is what adds a screen share
  (§27.6.1), the check MUST run on every `offer`, not only on join; an operator that checks
  admission and never re-checks has no bound on tracks at all.
- `advisory_max_participants` (key 5) MUST NOT be used as an admission basis by an operator, and
  MUST NOT be relied on by a client for anything but display. It exists because users think in
  people; it is a rendering convenience and it is explicitly not a limit.
- **Auto-detection MAY inform the published values; it MUST NOT replace them.** An operator MAY
  measure live CPU, bandwidth and forwarding load and use those measurements to choose the numbers
  it publishes in its *next* announcement. It MUST NOT admit past a published bound because a live
  measurement suggests headroom, and MUST NOT reduce an effective bound below what it published for
  a call already admitted. **A call that degrades mid-session is worse than one refused up front**:
  the refused call is retried elsewhere in seconds, while the degraded one fails in front of its
  participants with no remedy and no attribution.
- A published bound MUST NOT be lowered with respect to an already-admitted call. Lowering it for
  *future* calls is an ordinary capability announcement with a higher `caps_version`.
- `sframe_required` (key 6) is the operator's declaration that it will not forward unprotected
  media. An operator publishing `true` MUST enforce it (`0x0417`, §27.5.2). An operator publishing
  `false` is declaring that it *will* forward unprotected media if offered, and a client MUST
  surface that to the user before joining — it means the operator's ability to read media depends
  on the client's own configuration rather than on a property the operator has committed to.

## 27.8 Wire allocations & capability negotiation

| Registry | Allocation |
|---|---|
| Message Kinds (§21.16) | `0x44 rtc_signal` — Specification Required, extension range (§2.3, §21.16), the next point after `0x43` (§21.24d). An **ordinary sealed-MOTE kind**, `Payload`-wrapped, riding the existing deliver/ack/retry path (§2.6), unlike `0x40`'s bare signed object. Default tier `fast` (§27.4.6). |
| Capability Tokens (§21.22) | `rtc-1` — Specification Required; an endpoint's opt-in to originating and accepting `rtc_signal` (§27.9). `rtc-sfu-1` — Specification Required; an operator's opt-in to the SFU role (§27.7.2), announced together with an `RtcCapacity` (§27.7.4). An operator MUST advertise `rtc-1` to meaningfully advertise `rtc-sfu-1`. |
| Error/Status Codes (§21.14) | Three new points `0x0415`–`0x0417` **within** the existing subsystem byte `0x04` (Messaging & Group, §21.6), under §21.14's Specification-Required-within-an-existing-subsystem policy rather than the Standards-Action new-subsystem-byte policy — the same lighter path §21.24d took. This appendix registers **no** new subsystem byte. Individual code points are defined in §27.12. |
| KDF labels (§18.9 convention) | `DMTAP-RTC-v0/sframe` — an **MLS exporter label** (§27.5.1), **not** a signature DS-tag. It is reserved in the same namespace so no future extension can allocate the string, but §18.9's signature-preimage inventory gains only a note recording that this profile takes no signature of its own, in the manner §18.9.12 records that `ProvenanceRecord` carries none. |
| Signal types | `RtcSignal.type` (`u8`) values `0x01`–`0x07`, with §27.4.2 authoritative for the space and its allocation policy — the same arrangement §21.24a uses for `GatewayAttestation.disc`. |

A node that does not implement DMTAP-RTC is unaffected in every direction: it never advertises
`rtc-1`, never emits an `rtc_signal`, and ignores one it receives without acking it (§2.3,
`ERR_KIND_UNKNOWN` `0x020A`, IGNORE_NO_ACK) — which surfaces to the caller as an unanswered call
rather than as a delivered message that is never answered (§27.3.1).

## 27.9 Client requirements

A conformant DMTAP-RTC client:

1. MUST NOT originate an `rtc_signal` toward a peer that has not advertised `rtc-1` (§10.2); MUST
   treat the absence of the token as inconclusive rather than as a negative assertion, per §21.22's
   forward-compatibility rule, and MUST surface "cannot be reached for calls" rather than a
   protocol error.
2. MUST make placing a call an explicit user act, with the disclosure of §27.4.6 shown before the
   first signaling MOTE is emitted.
3. MUST show, for every call, **who else is in it**, sourced from the MLS group's own membership
   and not from any list an SFU supplies; and MUST surface a membership change during a call
   (§27.5.2's epoch advance is the same event) rather than only re-keying silently.
4. MUST display, before media is sent to an SFU, which operator identity is in the path
   (`RtcSignal` key 9) and whether it has committed to `sframe_required` (§27.7.4).
5. MUST provide an unambiguous, always-visible indication that a screen share is active, what is
   being shared, and a stop control reachable without returning to the shared surface. This is UX,
   not wire, and it is required here because §27.11 item 5 is the highest-consequence disclosure
   surface in the profile and the protocol cannot mitigate it.
6. MUST delete media key material per §27.5.2 and MUST NOT persist it across calls or into any
   backup.
7. MUST NOT present an MCU-mediated call as end-to-end encrypted (§27.7.3), and MUST NOT present an
   SFU-mediated call as metadata-private (§27.11 item 2).

## 27.10 Conformance & fail-closed table

DMTAP-RTC adds the following invariants to the auditable fail-closed set (§10.7), in the §10.7 /
§25.10 format. A conformant implementation of `rtc-1` enforces every row; a node that never
advertises `rtc-1` is not held to any of them.

| # | Invariant | Clause | Trigger | Behaviour / error on violation |
|---|-----------|--------|---------|-------------------------------|
| RTC-1 | **Signal from a non-member** | §27.4.5 | `Payload.from` is not a current member of the group under whose epoch the MOTE decrypted | reject; `ERR_RTC_SIGNAL_UNAUTHORIZED` `0x0415`, FAIL_CLOSED_BLOCK |
| RTC-2 | **Non-`offer` opens a call** | §27.4.5 | an `RtcSignal` names an unknown `call_id` with `type ≠ offer` | discard; no error surfaced to the sender |
| RTC-3 | **Stale SDP-bearing signal applied** | §27.4.5 | an `offer`/`pranswer`/`answer`/`rollback`/`bye` with `seq` ≤ the highest applied of those types for that `(call_id, sender)` | discard; MUST NOT apply — a late answer must never revert a session (incl. re-enabling a stopped share) |
| RTC-4 | **Candidates ordered like SDP** | §27.4.5 | an implementation applies RTC-3's `seq` rule to `candidate`/`end_of_candidates` | non-conformant; ICE is order-insensitive (RFC 8838) and discarding late candidates costs connectivity for no security gain |
| RTC-5 | **Malformed `RtcSignal`** | §27.4.1 | `sdp` absent on an SDP-bearing type, present on a type forbidding it, or an empty `candidates` array | `ERR_MALFORMED_OBJECT` `0x020D`, DROP_SILENT |
| RTC-6 | **Silent tier choice** | §27.4.6 | a call is placed to a `private`-pinned contact without an explicit user act and the §27.4.6 disclosure | non-conformant; §10.7.5 — refused or surfaced, never silent |
| RTC-7 | **Media secret not from the exporter** | §27.5.1 | `sframe_epoch_secret` derived by any path other than `MLS-Exporter("DMTAP-RTC-v0/sframe", det_cbor([call_id]), Nk)` at the current epoch | non-conformant; two such implementations do not interoperate and neither can prove which is wrong |
| RTC-8 | **`call_id` reuse** | §27.4.1, §27.5.1 | a `call_id` is reused across calls, or derived from group state rather than a CSPRNG | non-conformant; the context is the only per-call separation in the media key schedule |
| RTC-9 | **Media keys retained past the window** | §27.5.2 | `sframe_epoch_secret` or derived keys retained beyond the reorder window, beyond two epochs, or past teardown | non-conformant; this is the one failure that passes every other rule and still loses forward secrecy |
| RTC-10 | **Undecryptable frame triggers re-key** | §27.5.2 | an endpoint requests a group re-key per undecryptable frame | non-conformant; discard the frame. Sustained failure is `ERR_EPOCH_MISMATCH` `0x0406`, HOLD_RESYNC |
| RTC-11 | **SFrame removed or absent** | §27.5.2, §27.6.5 | unprotected media emitted or forwarded on a call that negotiated SFrame, or a renegotiation removing SFrame from an established call | `ERR_RTC_SFRAME_REQUIRED` `0x0417`, FAIL_CLOSED_BLOCK — protection ratchets up only |
| RTC-12 | **Screen track on a weaker path** | §27.6.5 | a screen or display-audio track keyed differently from camera/mic, or media keys granted to an SFU for thumbnailing/compositing/OCR | `ERR_RTC_SFRAME_REQUIRED` `0x0417`, FAIL_CLOSED_BLOCK — the request is an MCU (§27.7.3), not a feature |
| RTC-13 | **Track purpose in a DMTAP field** | §27.6.2 | an implementation carries track purpose anywhere but SDP `a=content:`, or ignores `a=content:` in the remote description | non-conformant; absent `a=content:` reads as `main`, never as an error |
| RTC-14 | **Single-share assumption** | §27.6.3 | a client replaces or suppresses an existing share when a second arrives, or keys layout on "the screen track" rather than on MID+sender | non-conformant |
| RTC-15 | **Admission by headcount** | §27.7.4 | an operator admits or refuses on `advisory_max_participants`, or on any participant count, rather than on keys 1–4 | non-conformant; a screen share is usually the highest-bitrate stream, so headcount does not bound the resource that runs out |
| RTC-16 | **Renegotiation not re-checked** | §27.7.4 | an operator evaluates capacity on join but not on each subsequent `offer` | non-conformant; renegotiation is how tracks are added, so an unchecked renegotiation is an unbounded track count |
| RTC-17 | **Auto-detection overrides the published bound** | §27.7.4 | an operator admits past a published bound on a live measurement, or lowers an effective bound for an already-admitted call | `ERR_RTC_CAPACITY_EXCEEDED` `0x0416`, DENY_POLICY at admission; mid-call reduction is non-conformant |
| RTC-18 | **Topology changed unilaterally** | §27.7.2 | an SFU (or a peer) is interposed on an established call without an `offer` the peer could decline | non-conformant; an operator MUST NOT enter a media path without the user's knowledge |
| RTC-19 | **MCU presented as E2E** | §27.7.3 | a mixed/re-encoded call described to the user as end-to-end encrypted | non-conformant; the operator holds plaintext by construction |
| RTC-20 | **Capability rollback** | §27.7.4, §10.2 | an `RtcCapacity`/token announcement at a `caps_version` ≤ the last accepted from that operator | `ERR_CAPABILITY_ANNOUNCE_ROLLBACK` `0x030A`, FAIL_CLOSED_BLOCK — retain the higher-versioned set |

> **Conformance-suite note.** These invariants are catalogued as the **`RTC`** category in
> [`conformance/SUITE.md`](conformance/SUITE.md) / [`conformance/suite.json`](conformance/suite.json).
> Most are `construction-todo` recipes over the CDDL of §27.4.1/§27.7.4 or `manual-attestation`
> UX/operator reviews (RTC-6, RTC-12, RTC-14, RTC-19, and the §27.9 client requirements), matching
> how the `PUBSUB` and `VIDEO` families are catalogued. **RTC-7 is the exception and is the one case
> that can become a byte-backed KAT**: `MLS-Exporter` with a fixed `exporter_secret`, the fixed label
> `"DMTAP-RTC-v0/sframe"` and a fixed 16-byte `call_id` is fully deterministic and re-derivable with
> an MLS implementation plus HKDF, no DMTAP reference implementation required. That vector MUST fix
> the `det_cbor([call_id])` context byte-for-byte, since the difference between the fixed one-element
> array and a bare 16-byte context is exactly the ambiguity §27.5.1 exists to close and is invisible
> in prose. §27.8's allocations are registry work, not conformance cases.

## 27.11 Security considerations / honest limits

Stated in the falsifiable-claim discipline of §6.9 and the honest-limits governance of §6.6/§22.9:
each item states the claim, the mechanism, and the **residual that is not claimed**. None of these
is a defect awaiting a fix; each is an inherent consequence of the design above.

1. **The SFU never decrypts media — and that is materially stronger than the legacy gateways, which
   see plaintext by construction.** *Claim:* an operator forwarding an SFrame-protected call cannot
   recover media plaintext. *Holds by:* SFrame keyed from the MLS exporter (§27.5.1); the operator
   is not a group member and holds no epoch secret; ratcheting protection (§27.5.2). *Residual:*
   the guarantee is **media payload only** — headers, RTCP and timing are visible by design
   (§27.5.3, and item 2); it evaporates entirely if an operator is *also* admitted to the MLS group
   (then it is a participant, not an intermediary) or if the call is mixed rather than forwarded
   (§27.7.3); and it says nothing about the endpoints, which hold plaintext because they are the
   endpoints (§6.6 item 3). The comparison to §7/§26 is genuine and worth stating: a legacy mail
   gateway handles RFC 5322 plaintext because the legacy protocol has no other shape, whereas an
   SFU handles ciphertext because the media protocol does have one. **The difference is
   structural, not a matter of operator discipline.**
2. **An SFU still learns a great deal, and this profile claims no metadata privacy whatsoever.**
   *Not claimed:* an SFU learns **who is in the call** (it holds the participant list — it must, to
   forward), **when it started and ended, and how long it ran**, **who spoke when** (from packet
   timing and volume-derived RTP feedback), **who was sharing a screen** (a distinct, high-bitrate,
   `a=content:slides`-labelled m-section it routes), **each participant's IP address and rough
   location**, and **per-packet sizes and timing**, from which activity, speech patterns and even
   coarse video content characteristics can be inferred. Traffic analysis of SFrame-protected media
   is a real and active field, and this profile makes no claim of resistance to it. The opt-in,
   research-tier mixnet's protections
   ([docs/research/mixnet.md §4.4](docs/research/mixnet.md)) that bound this class of leakage for
   messaging, for implementations and users that opt into that tier, **do not apply here and
   cannot** (§27.1.2, §27.4.6). A user for whom the *existence* of a call is the sensitive fact
   must not place the call — no protocol setting in this appendix changes that.
3. **Mesh calls disclose participants' IP addresses to each other.** ICE (RFC 8445) discovers and
   exchanges host, server-reflexive and relayed candidates, and a direct mesh connection means each
   participant learns the others' addresses. *Mitigation, partial:* forcing TURN-relayed candidates
   only (RFC 8656) hides the host address behind the relay, at the cost of latency and of putting a
   relay operator in the path — trading item 3 for a bounded amount of item 2. An implementation
   SHOULD offer relay-only as a user-selectable mode and MUST NOT describe a default mesh call as
   hiding participants' addresses. Note the direction of the trade: **mesh is stronger on operator
   exposure and weaker on peer exposure**; SFU is the reverse.
4. **A call's media keys are scoped to the MLS group, not to the set of participants who joined.**
   Because `sframe_epoch_secret` is an exporter output of the group's epoch (§27.5.1), **any current
   group member can derive it**, whether or not it joined the call — including a member who declined
   and one that is present but silent. Excluding a member from a call is therefore a **forwarding
   and UX property, not a cryptographic one**: an SFU can decline to forward to them, and a mesh
   peer can decline to connect, but a member who obtains the packets by any other means can decrypt
   them. An application that needs cryptographic exclusion MUST create an MLS group over exactly the
   intended participants and derive from **that** group; this profile deliberately does not
   auto-create sub-groups, because a silently-created group is a membership change the user did not
   see. This is the residual most likely to be assumed away, so it is stated as a MUST rather than
   as advice.
5. **Screen sharing is the highest-consequence accidental-disclosure surface in conferencing, and
   the protocol cannot mitigate it.** Sharing the wrong window, sharing a full desktop when a window
   was intended, notification banners appearing mid-share, password managers and credentials on
   screen, an open messaging client, a tab strip disclosing unrelated activity, and shared system
   audio carrying a notification or another call — every one is a disclosure that is *cryptographically
   correct*: the content was encrypted, delivered and decrypted exactly as intended, to exactly the
   people the sharer chose. There is no protocol mechanism that helps, and this profile claims none.
   Mitigation is entirely client-side UX, which is why §27.9 item 5 states the minimum as a MUST.
   It is recorded here, in the security section, because a limit whose only remedy is UX is exactly
   the limit an implementation is most likely to leave unowned.
6. **A participant is a plaintext holder and may record.** Every participant necessarily decrypts
   the media, so any participant may record, transcribe, screenshot or re-stream it. This profile
   defines no recording indicator, because an indicator a client can simply not send is a false
   assurance rather than a control. This is the same endpoint floor SP-1 discloses (§6.6 item 3),
   applied to media.
7. **Compromise of an operational signing key extends to call signaling.** A compromised device key
   can emit `rtc_signal` under the identity until it is revoked (§1.5), which within a group means
   it can place calls, add tracks and tear down calls as that member. Keeping `IK` cold and signing
   with a revocable operational key (§1.2) bounds this exactly as it bounds every other kind. Note
   the asymmetry that MLS provides for free: the same compromise yields **media** keys only for the
   epochs the device actually held (§27.5.2), not for the call's history.
8. **Availability is not a claim.** ICE may fail to establish a path; TURN may be unreachable; an
   SFU may refuse on capacity (§27.7.4). A call that cannot be established is a normal outcome, not
   a protocol fault, and this appendix — like §25.1.3 for hints — guarantees nothing about it.

## 27.12 Error registry (`0x0415`–`0x0417`, within subsystem `0x04`)

These codes extend the subsystem byte `0x04` (Messaging & Group — MLS) that §21.6 already owns;
this appendix registers **no** new subsystem (§27.8). Codes follow the §21 conventions and the
responder-action vocabulary of §21.2. The table below is authoritative for this range, exactly as
§25.12 is authoritative for `0x090E`–`0x0913`.

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0415` | `ERR_RTC_SIGNAL_UNAUTHORIZED` | `RtcSignal` intake (§27.4.5) | `Payload.from` is not a current member of the MLS group under whose epoch key the carrying MOTE decrypted — evaluated against the receiver's own applied group state, never a claim in the signal. | Conditional (a pending Add the receiver has not yet applied may resolve it) | FAIL_CLOSED_BLOCK |
| `0x0416` | `ERR_RTC_CAPACITY_EXCEEDED` | SFU admission / renegotiation check (§27.7.4) | Admitting the participant, or accepting the renegotiation, would exceed a published `RtcCapacity` bound (`max_tracks`, `max_aggregate_bps`, `max_tracks_per_participant`, `max_bps_per_track`). A policy deny evaluated against published values, never against a live measurement. | Yes (a later call, a smaller track set, or another operator) | DENY_POLICY |
| `0x0417` | `ERR_RTC_SFRAME_REQUIRED` | Media protection check (§27.5.2, §27.6.5, §27.7.4) | Unprotected media was emitted or forwarded on a call that negotiated SFrame, a renegotiation would remove SFrame from an established call, or media keys were requested in order to process a track server-side. Protection ratchets up only — the media-layer analogue of `0x0414`. | No | FAIL_CLOSED_BLOCK |

## 27.13 Unverified, and deliberately unsettled

Stated separately from §27.11 because these are gaps in *this document's knowledge*, not properties
of the design. Each is a thing an implementer MUST check rather than inherit.

1. **SFrame's document status (verified).** This profile cites SFrame as **RFC 9605**, Proposed
   Standard, published August 2024, with **no successor or obsoleting RFC**. Three verified errata
   exist against it (**8321, 8565, 8703**, as of 2026-01) — none touch §5.2 (the MLS-binding
   section) or the KID/epoch mechanism this profile depends on. The residual risk is not that the
   citation is unverified but that a future erratum against §5.2 or the KID mechanism could still
   land; an implementer SHOULD re-check errata status before implementing §27.5.
2. **SFrame's own MLS binding, and the interoperability consequence of §27.5.1 (verified).** RFC
   9605 §5.2 specifies the exporter label `"SFrame 1.0 Base Key"` with empty context
   (`base_key = MLS-Exporter(label, "", AEAD.Nk)`). This profile deliberately diverges from that:
   §27.5.1 derives the per-call secret using a **DMTAP** exporter label (`"DMTAP-RTC-v0/sframe"`)
   plus `det_cbor([call_id])` as context, for **per-call keying** — a group's single MLS epoch
   would otherwise yield one shared secret across every concurrent call in that group, which
   §27.5.1 needs to avoid. This is now confirmed against RFC 9605 §5.2, not an unverified gap: a
   DMTAP endpoint and a stock SFrame-over-MLS endpoint derive different secrets and **do not
   interoperate at the media layer**, a verified and accepted domain-separation cost, not an
   oversight.
3. **The KID/epoch mapping (confirmed sufficient; one gap remains).** RFC 9605 §4's key identifier
   format is `KID = (context << (S+E)) + (sender_index << E) + (epoch mod 2^E)`: the low-order
   **E bits** carry `epoch mod 2^E`, with `E` an application-chosen parameter sized to the
   receiver's reordering/retention window. This is confirmed sufficient for §27.5.2's window. The
   remaining gap is that this profile does not yet state a concrete `E`: §27.5.2's ~30-second,
   2-epoch retention window needs a **RECOMMENDED** `E` value, and this profile SHOULD state one.
4. **`a=content:` in real engines.** No browser API sets `a=content:` from `getDisplayMedia()`, so
   §27.6.2 requires an endpoint to write and read the attribute in SDP directly. **Whether a given
   engine accepts a local description carrying an added `a=content:` line, rather than rejecting the
   modified description, is engine-dependent and is not verified here.** §27.6.2 states why the
   remedy is to fix the engine path rather than to add a parallel DMTAP field.
5. **The mesh ceiling is a rule of thumb, not a measurement.** The 3–6 figure and the RECOMMENDED
   default of 6 in §27.7.1 are drawn from the O(n²)/O(n) structure and from ordinary asymmetric
   consumer uplinks; this document reports **no** measurement of its own, and an operator with real
   telemetry should trust the telemetry.
6. **No capacity value is recommended.** §27.7.4 defines the *shape* of a ceiling and refuses to
   suggest numbers for it, because a defensible `max_aggregate_bps` depends on hardware, codec mix
   and share prevalence that this document cannot know. An implementation that wants a starting
   point must measure its own.

## 27.14 Change log — normative corrections

This document is pre-1.0 and is corrected in the open, in the same discipline
[`substrate/SYNC.md` §14](substrate/SYNC.md) established and §24.17 / §25.13 follow: a defect found
by an implementation or an adversarial audit is fixed here **and recorded here**, never silently
edited. Each entry states what changed, whether it changes **wire bytes** (a CDDL shape, a
KDF/signature preimage, or a value carried on the wire — a KAT/vector consumer must be updated) or
is a **behavioural rule** (a MUST governing what a conformant implementation does with bytes whose
shape is unchanged), and how it was found.

| # | Change | Class | Found by |
|---|--------|-------|----------|
| **C-01** | **Initial specification.** DMTAP-RTC is introduced allocating one core message kind (`0x44 rtc_signal`, §27.3), one MLS exporter label (`"DMTAP-RTC-v0/sframe"`, §27.5.1), two capability tokens (`rtc-1`, `rtc-sfu-1`), and three error points within the existing subsystem `0x04` (§27.12). It defines **no** new signed structure, **no** new signature DS-tag, and **no** new error subsystem byte (§27.3.3). | **NORMATIVE — new extension.** Nothing pre-existing changes shape: no existing object gains or loses a key, no existing preimage changes, and a node that does not advertise `rtc-1` is unaffected in both directions. **Depends on companion changes to §21 and §16 that have not landed** (§27.8): until §21 registers kind `0x44`, the two tokens, the three codes and the exporter label, every citation of them in this document is a **forward reference** — the same status §24.17 C-06 recorded for §21.24e. | Written to fill the gap §5 leaves: DMTAP has groups, epochs and delivery but no real-time path, so calling required a proprietary signaling server — the one component the substrate makes unnecessary. |
| **C-02** | **The core-kind-vs-metadata decision is recorded with its reasoning rather than asserted.** §24's facets allocate no kind and ride `pub_announce.meta`; the obvious symmetry would have carried signaling on an existing kind's `Headers.ext`. §27.3.1 states why the precedent does not transfer, and the decisive argument is a concrete user-visible failure rather than a taxonomy preference: §21.20 requires an unrecognized *header* to be ignored while the **message is processed and acked**, so a call offer to a peer without call support would be acked as delivered and rendered as an empty chat message. §2.3's unknown-**kind** rule (ignore, MUST NOT ack) produces exactly the right outcome instead. | **Rationale, recorded normatively.** No byte differs from what C-01 already specifies; what is added is the record of why, so a future revision that proposes collapsing the kind back into metadata has to answer the argument rather than rediscover it. | Writing §27.3 against §24's precedent and §21.20's forward-compatibility rule, and following what a non-implementing peer actually does with each of the two carriers. |
| **C-03** | **Track purpose is SDP-native (`a=content:`, RFC 4796) with no DMTAP field, and the implementation cost is disclosed rather than designed around.** An application-level `tracks` map in `RtcSignal` was the obvious alternative and was rejected on §24.4.2's duplicated-fact reasoning, on atomicity with renegotiation, and on the §27.1 requirement that this profile be implementable against an SFU that does not know DMTAP exists. The cost — that no browser API sets the attribute, so endpoints must write and read it in SDP, and that engine tolerance of the added line is unverified (§27.13 item 4) — is stated in §27.6.2 rather than mitigated by adding the field back as a "fallback", which would have reintroduced the two-sources-of-truth problem the decision exists to avoid. | **NORMATIVE — behavioural rule; no DMTAP wire bytes.** `RtcSignal` carries no track-purpose key and none is reserved for one. What is newly non-conformant is an implementation that carries purpose anywhere but SDP, or ignores `a=content:` in a remote description (RTC-13). | Following the mandate to prefer the SDP-native route, then testing it against the case that usually forces an application field: an engine that rejects the munged description. The answer is to fix the engine path, because a subordinate fallback field is still a second source of truth. |
| **C-04** | **The SFU capacity ceiling is expressed in tracks and bitrate, never in participants, and `advisory_max_participants` is explicitly not an admission basis.** A `max_participants` capability is the intuitive design and is wrong for a specific, checkable reason: a screen share is typically the highest-bitrate stream in a conference, so six participants with two shares can cost several times six on camera, and a headcount does not bound the resource that runs out. §27.7.4 makes keys 1–4 the admission basis, keeps key 5 as display-only, requires the check to re-run on **every** renegotiation (because renegotiation is how a share is added, RTC-16), and pins the auto-detection rule: measurement MAY inform the next published announcement, MUST NOT override a published bound, because a call that degrades mid-session is worse than one refused up front. | **NORMATIVE — new object shape (`RtcCapacity`) carried in an existing `system` MOTE.** No new signed structure: the advertisement rides §10.2's capability announcement, so it inherits `Payload.sig` and the monotonic `caps_version` rollback protection (`0x030A`) unchanged. | Working the screen-share case through a headcount bound and finding it wrong in both directions — too permissive for a call with shares, too restrictive for an audio-only one. |
| **C-05** | **§27.11 item 4 is stated as a MUST: SFrame keys are scoped to the MLS group, not to the call's participants.** Because `sframe_epoch_secret` is an exporter output of the **group** epoch, any current member can derive it whether or not they joined the call — so excluding someone from a call is a forwarding/UX property, not a cryptographic one. The remedy (an MLS group over exactly the intended participants) is required of the application rather than automated, because a silently auto-created sub-group is a membership change the user never saw. | **NORMATIVE — behavioural rule; no wire bytes.** No CDDL, label or code changes. What is newly required is that an application needing cryptographic exclusion create the group explicitly, and that no implementation present call-participant exclusion as a confidentiality boundary. | Tracing who can compute the §27.5.1 exporter output, rather than who the SFU forwards to — the two sets differ, and the difference is invisible from the call UI. |

**Standing rule.** A defect between this document and an implementation is resolved by deciding
**which side is right on the merits** and correcting the other in the open, exactly as
[`substrate/SYNC.md`](substrate/SYNC.md) §14 states it. **C-01 and C-04 introduce wire shapes**;
**C-02, C-03 and C-05 change no byte** — each records or sharpens a MUST governing what a
conformant implementation does with bytes whose shape is unchanged. None is classed INFORMATIVE:
every entry here constrains an `rtc-1` conformance requirement, not advice. A future entry adopting
SFrame's own MLS exporter label in place of `"DMTAP-RTC-v0/sframe"` (§27.13 item 2) would be
**NORMATIVE — wire bytes**, invalidating every media key derived under the current label, and MUST
be recorded as such rather than applied as an editorial correction.
