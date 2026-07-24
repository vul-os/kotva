# 25. DMTAP-PUBSUB: Feed Subscriptions & Push Hints (extension)

DMTAP-PUB (§22) gives an identity an append-only, signed **author feed** any node may serve, and
any reader may pull, without trusting the server. What it does not give is a **protocol object** for
"I follow this feed" — following is a purely client-side act (§24.18.9's *workshop*), and a system that
wants to notify machines, not humans scrolling a client, has nowhere to plug in. This appendix
specifies **DMTAP-PUBSUB**, an additive extension of §22 that closes that gap with four things: a
signed, revocable **`Subscription`** object; a **topic** dimension so one identity can run several
independent feeds; **push delivery** of new entries as ordinary MOTEs, riding the existing
deliver/ack/retry machinery (§2.6) instead of inventing new reliability plumbing; and an explicit
application of §9.9's fan-out governance to the resulting push traffic.

DMTAP-PUBSUB is **opt-in, additive, and capability-negotiated (§10.2)**, exactly as DMTAP-PUB was
(§22). It reassigns **no** existing key in any existing wire object — not `PubAnnounce`, not
`FeedEntry`, not `FeedHead`, not `Envelope`, not `Payload` — bumps no `Envelope.v` and no DNS `v=`
anchor, and introduces no flag day. It adds exactly **one** field to an existing signed object:
`FeedHead` key `64` (`topic`, §25.3.1), taken from the `≥ 64` range §18.1.2 reserves for precisely
this purpose and carried **only** toward peers that have advertised `pubsub-1` (§25.8), so a peer
that has not opted in never receives a byte it would have to reject. Everything else below rides
machinery that already exists: a message kind in the reserved range
(§2.3, already used once by `pub_announce`), a capability token (§10.2, §21.22), a handful of new
error codes inside the `ERR_PUB_*` block DMTAP-PUB already owns (§21.24b), and the ordinary sealed
`Envelope`/`Payload` path (§2.4, §18.3.5) for every new MOTE kind this appendix defines. A node that
does not implement DMTAP-PUBSUB is unaffected: it never advertises `pubsub-1`, never emits or
accepts a `Subscription`/`SubscriptionRevoke`/`FeedHint`, and continues to serve and pull plain §22
feeds exactly as before.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174,
consistent with the rest of this specification. Where this appendix and §18 (wire format) or §22
appear to differ on a shared mechanism, the more specific rule governs; new objects follow the same
integer-keyed CBOR convention as every §22 object (§18.1.2): keys assigned per object type starting
at `1`, keys **≥ 64 reserved** for future/extension fields, and the signed-vs-unsigned unknown-key
discipline unchanged.

## 25.1 Goals & non-goals

### 25.1.1 The gap, precisely

DMTAP already has three pub/sub-*shaped* mechanisms, and none is aimed at a machine subscriber:

| Mechanism | Confidentiality | Membership | Delivery | Why it doesn't fit |
|-----------|------------------|------------|----------|---------------------|
| Author feed (§22.4) | none (plaintext, signed) | open, unbounded | pull-only | no protocol object for "I follow this"; no push at all |
| MLS group/channel (§5.8) | E2EE | known, closed | push (fan-out MOTEs) | TreeKEM commit churn on every membership change is brutal at large, open, fast-changing subscriber counts (§5.1, §5.8.3) — it is built for *groups*, not *audiences* |
| JMAP push (§8.1) | N/A | one client, one node | push | client-to-**own**-node only; carries no cross-identity subscription concept at all |

The gap is **machine-oriented event distribution with a real subscription**: something a publisher
can grant, a subscriber can hold, audit, and revoke, and that delivers new entries without either
party polling blind or paying TreeKEM's membership-churn cost for an audience that was never meant
to be a cryptographic group.

### 25.1.2 Goals

1. **A subscription is a protocol object, not a client habit.** §22.4.4's `feed_head`/`feed_range`
   already let anyone *pull*; this appendix adds a signed, expiring, revocable `Subscription`
   (§25.4) so "who is allowed to be pushed a hint for this feed" is auditable, not implicit in a
   client's local follow-list.
2. **Topic addressing, at one negotiated field.** One identity, one feed was §22.4's structural
   assumption (`FeedHead.pub` *is* the feed). §25.3 adds a topic dimension so an identity may run
   several independent, comparably-scoped feeds (a release feed, a chatter feed, a
   security-advisory feed). The topic lives in the **locator** *and* in the **signed head** (key
   `64`, §25.3.1): the locator is what a request names, the signed byte is what makes the answer
   checkable. Nothing is added to `FeedEntry` or `PubAnnounce`, and a publisher's pre-existing
   untopiced feed keeps its bytes exactly (§25.3.3).
3. **Push, without inventing delivery.** New entries are pushed to subscribers as **ordinary
   MOTEs** through the existing sealed `Envelope`/`Payload` path (§2.4, §18.3.5), so §2.6's
   deliver/ack/retry gives **at-least-once delivery for free**. No new retry queue, no new ack
   scheme, no new signature preimage for the transport layer.
4. **Fan-out is governed by §9.9, not re-derived.** Pushing a hint to *N* subscribers on one new
   post is structurally the same shape as a group-address post fanning out to *N* members — §25.7
   points at §9.9's existing rules (origin accountability carried through, per-poster rate limits,
   cost commensurate with fan-out) rather than inventing a parallel anti-abuse model.

### 25.1.3 Non-goals

- **Not a broker.** There is no third party in the middle that accepts a publish and redistributes
  it; the publisher's own node (or a holder it delegates to, §25.4.3) is the only thing that ever
  sees a subscriber list, and it is edge state exactly like the retry queue (§0.2.1), never a new
  operator class (§0.2.3).
- **Not encrypted broadcast to an open audience.** §25.11 states this plainly rather than papering
  over it: DMTAP-PUBSUB inherits §22's plaintext posture unchanged. A publisher who needs
  confidentiality with a *known, bounded* membership already has MLS channels (§5.8); wanting
  confidentiality **and** millions of anonymous subscribers **and** open join, all at once, is out
  of scope (§25.11 item 1, mirroring §6.6's honest-limits style).
- **Not a guarantee that a hint arrives quickly, or at all.** A hint is an optimisation over
  polling, never a substitute for it (§25.6.1); a subscriber that never receives a single hint for
  its entire subscription lifetime and instead only ever polls `feed_head` directly is still fully
  conformant and loses nothing but latency.
- **Not a replacement for `pub-1`.** Every object this appendix defines is either served by the
  existing §22.5 surfaces unchanged (topic-scoped feeds, §25.3.2) or delivered over the ordinary
  MOTE path (§25.4–§25.6); DMTAP-PUBSUB adds no new serving surface of its own.

## 25.2 Relationship to §22 (informative recap)

This appendix is built entirely on §22 primitives it does not redefine: **author feeds**
(`FeedEntry`/`FeedHead`, §22.4, per-identity append-only monotonic-`seq` logs with the standard
anti-rollback and fork-detection rules, §22.4.2); **`pub_announce`** (kind `0x40`, §22.3, a signed
plaintext CBOR announcement, `announce_id` the derived content address, §22.3.1); **public-object
serving** (§22.5, plain-HTTPS feed/announce/manifest/chunk endpoints, extended additively in
§25.3.2); and the ordinary sealed **MOTE** path this appendix newly puts to use for feeds (`Envelope`
/ `Payload`, §2, §18.3). Where this document says "the feed" or "the announce" without
qualification, it means these §22 objects — consult §22 for their exact wire grammar. This
appendix introduces **three** new object types (`Subscription`, §25.4; `SubscriptionRevoke`, §25.5;
`FeedHint`, §25.6.2) and exactly **one** capability-negotiated extension field on an existing one
(`FeedHead` key `64`, §25.3.1); no existing key changes meaning, and no object outside `FeedHead`
changes at all.

## 25.3 Topic addressing

### 25.3.1 A locator dimension, bound into the signed head

§22.4 assumed one identity, one feed: `FeedHead.pub` names *the* feed, and `feed_head(pub)` (§22.4.4)
is a total function of `pub` alone. DMTAP-PUBSUB widens that to `(pub, topic)`: an identity that
wants several independent streams maintains **several independent `FeedEntry`/`FeedHead` chains**
under the **same** `pub`, each append-only and internally identical in shape to a §22 feed. A
publisher that runs multiple topics is, mechanically, running multiple independent instances of
§22.4's bookkeeping — separate `seq` counters, separate `prev` chains, separate signed heads — under
one identity key, exactly as one person may keep several separate notebooks. `signer` MAY be the
same operational key across every topic (there is no requirement to mint a per-topic delegate).

Two things are needed to make that safe, and an earlier revision of this section supplied only the
first: a **locator** saying which chain a request is for (§25.3.2), and a **signed byte** saying
which chain a served head belongs to.

**The topic is inside the signature (normative).** A `FeedHead` served for a non-empty topic carries
the topic label in the reserved extension key `64`:

```cddl
; FeedHead (§22.4.1) as extended by DMTAP-PUBSUB. Keys 1-8 are §22's, unchanged in
; number, type and meaning; only key 64 is added.
FeedHead-pubsub = {
  1 => u8, 2 => suite, 3 => ik-pub, 4 => u64, 5 => hash, 6 => ts, 7 => ik-pub, 8 => sig-val,
  ? 64 => tstr,        ; topic   NFC UTF-8 topic label (§25.3.4); ABSENT iff topic = ""
}
```

Key `64` sits in the **`≥ 64` range §18.1.2 reserves** for exactly this case — a structural extension
of a *signed* object, added by a document other than the one that defined the object, and carried
only toward peers that advertise the paired capability token (`pubsub-1`, §25.8; §21.22's
signed-object `≥ 64` extension-field rule). Because it is an ordinary map key it is inside
`det_cbor(FeedHead ∖ {8})`, so `FeedHead.sig` covers it and the `DMTAP-PUB-v0/feed` preimage
(§22.4.1) binds the topic to the head exactly as it binds `pub`, `seq` and `tip`.

> The following are all **MUST**:
>
> 1. Key `64` is **absent** iff the head is the `topic = ""` (default/untopiced) feed; a head
>    carrying key `64` with an empty string is malformed and MUST be rejected on decode. There is
>    exactly one encoding of every topic, including the empty one.
> 2. A server **MUST NOT** return a head carrying key `64` on the two-segment §22.5.1 path, and
>    **MUST NOT** send one to a peer that has not advertised `pubsub-1` (§18.1.2, §10.2). On the
>    HTTP surface the three-segment path (§25.3.2) *is* the request-side declaration: a client that
>    requests it has by construction declared it understands key `64`.
> 3. A server **MUST** include key `64`, carrying the exact NFC UTF-8 label (§25.3.4), in every
>    `FeedHead` it serves at a three-segment locator with a non-empty topic.
> 4. A reader **MUST** reject a `FeedHead` whose key `64` (absent ⇒ `""`) is not byte-equal to the
>    topic it requested: `ERR_PUB_FEED_TOPIC_MISMATCH` (`0x0915`), FAIL_CLOSED_BLOCK. The check is on
>    the verified, signed value, never on the locator alone.
> 5. A reader **MUST NOT** admit a `FeedEntry` into the chain for a given `(pub, topic)` unless that
>    entry is reachable by the `prev` chain from the `tip` of a `FeedHead` it has verified for that
>    same `(pub, topic)`. `FeedEntry` gains no field: an entry's topic scoping is the transitive
>    commitment of the signed head that names it (§22.4.1), and nothing else.

**What this costs, stated plainly.** A `pub-1`-only verifier handed a topic-bearing head rejects the
whole object fail-closed (§18.1.2). That is the *intended* behaviour and the reason the capability
gate above is a MUST rather than a SHOULD: a topic chain silently consumed as the publisher's
default feed is precisely the confusion key `64` exists to prevent. Topic-scoped serving is
therefore a `pubsub-1` surface, not the `pub-1` convenience an earlier revision claimed (§25.3.2,
§25.13 C-01).

**Why leaving the topic out of every signed byte was wrong (rejected alternative, and the false
claim it rested on).** An earlier revision of this section put the topic *only* in the locator and
defended it by analogy to §22.2.3 — "bind the distinction into how the object is addressed, never
into a flag a peer could misread." The analogy was false, and the difference is the whole security
argument: §22.2.3 binds the manifest *type* into the **hash/signature preimage**, folding the DS-tag
into every leaf and node of the Merkle tree so that a sealed root and a public root over the same
chunk-hash list are **different values**. The type there is not asserted, it is *computed*. A
locator is not a preimage; nothing was folded anywhere, and both consequences were real:

1. **Self-inflicted equivocation.** §22.4.2 keys the anti-rollback watermark and fork detection on
   `pub` alone. Several topics under one `pub` produce several signed heads at overlapping `seq`
   values with different `tip`s, so an honest publisher trips `ERR_PUB_FEED_ROLLBACK` (`0x0907`) and
   `ERR_PUB_FEED_CHAIN_BROKEN` (`0x0908`, HALT_ALERT) against itself — and worse, hands any third
   party two genuinely-signed claims about the same `(pub, seq)`, which §22.4.2 calls transferable
   evidence that the author equivocated.
2. **Silent topic substitution.** A hostile holder could serve topic A's chain at topic B's locator
   and pass **every** §22 check: valid head signature, intact `prev` chain, advancing `seq`. A
   subscriber to `security-advisories` would see a signed, advancing, verifiable feed that simply
   never contains the advisory — the suppression attack signed feeds exist to prevent,
   reintroduced one layer above them.

Both are closed by putting the label in the preimage, and neither is closed by any amount of care at
the locator layer. The residual cost — a negotiated field, and a `pub-1`-only peer that cannot read
a topic-scoped feed at all — is the honest price, and it is paid where it is visible.

### 25.3.2 Serving-layer locators (additive to §22.5)

The abstract §22.4.4 operations widen with an optional `topic` parameter that **defaults to the
empty string**, which names the untopiced feed — i.e. exactly the feed a §22-only peer already
knows:

- `feed_head(pub, topic = "") → FeedHead`
- `feed_range(pub, topic = "", from_seq, to_seq) → [FeedEntry]`

The HTTP binding (§22.5.1) is extended the same way §5.3/§5.4 of `substrate/FEEDS.md` extended it
for range proofs and fetch hints — a **new, additive path**, with the existing two-segment path left
byte-for-byte as specified in §22.5.1:

```
GET /.well-known/dmtap-pub/feed/{pub}/head                          → FeedHead    [UNCHANGED, §22.5.1]
GET /.well-known/dmtap-pub/feed/{pub}/range?from=&to=               → [FeedEntry] [UNCHANGED, §22.5.1]
GET /.well-known/dmtap-pub/feed/{pub}/topic/{topic}/head            → FeedHead    [NEW, additive]
GET /.well-known/dmtap-pub/feed/{pub}/topic/{topic}/range?from=&to= → [FeedEntry] [NEW, additive]
```

`{topic}` is the percent-encoded (RFC 3986) UTF-8 topic label, subject to §25.3.4. A client **MUST**
percent-encode the NFC UTF-8 bytes of the label; a server **MUST** percent-decode and then apply
§25.3.4's rules to the decoded result, rejecting a non-conforming label rather than repairing it.
The **empty topic has exactly one locator spelling** — the two-segment path: a server **MUST NOT**
emit, and a client **MUST** reject, a three-segment path whose `{topic}` segment decodes to the empty
string (§25.3.4 rule 5), because an empty path segment is precisely the thing an intermediary,
proxy, or normalizing router is apt to collapse, turning one feed's locator into another's. A
topic-unaware `pub-1` server that only ever serves the original two-segment path remains fully
§22-conformant and needs no code change to keep doing so. The mesh binding (§22.5.2) widens
analogously: a holder advertising **`pubsub-1`** MAY additionally serve topic-scoped feeds,
discovered by whatever out-of-band means a topic label is shared (a `pub_announce`'s `meta` map,
§22.3.1, is a natural carrier for "here is my topic list," but this appendix does not standardise
one — that is left to a profile, exactly as §24's media and engineering-artifact facets layer profile-specific `meta` schemas over §22
without this document's involvement).

**Reading a topic-scoped feed requires `pubsub-1` (normative).** A topic-scoped `FeedHead` carries
the reserved extension key `64` (§25.3.1), and §18.1.2 permits a sender to place a `≥ 64` key in a
*signed* object **only** toward a peer that has advertised the paired capability token. `pubsub-1`
is that token. Concretely: a server **MUST NOT** serve a topic-scoped feed to a peer that has not
advertised `pubsub-1` (on the HTTP surface, requesting the three-segment path is that
advertisement — §25.3.1 rule 2), and a client that cannot decode key `64` **MUST NOT** request one.
The reads themselves remain anonymous and content-addressed exactly as §22.5.1 specifies;
`pubsub-1` gates *format understanding*, not identity. An earlier revision of this subsection
asserted the opposite — "no new capability is required to read a topic-scoped feed" — which was
true only of the unsigned-topic design §25.3.1 rejects (§25.13 C-01).

### 25.3.3 Backward compatibility (normative)

> A publisher that already operates a §22 feed and later adopts topics **MUST** continue serving its
> pre-existing `FeedEntry`/`FeedHead` chain, byte-for-byte unchanged, as the `topic = ""` feed. A
> reader that calls `feed_head(pub)` exactly as it did before this appendix existed **MUST** observe
> no discontinuity — the same chain, the same `seq` numbering, the same anti-rollback watermark
> (§22.4.2). Topic adoption is additive per publisher, never a migration.

This is the same discipline as every other extension in this document family: DMTAP-PUB changed no
sealed-path default (§22), and the Published-Artifact profile changed no §22 byte (§24). Topic
addressing changes no byte of the **default** feed — key `64` is absent from it by construction
(§25.3.1 rule 1), so its `det_cbor` encoding, its signature preimage and its `tip` are what they
always were — and orphans no existing subscriber of it.

### 25.3.4 Topic labels (normative)

A **topic label** is the Unicode string carried in `Subscription.topic` (key 5, §25.4.1),
`FeedHint.topic` (key 2, §25.6.2), `FeedHead` key `64` (§25.3.1), and the `{topic}` locator segment
(§25.3.2). One label, one feed — so the label needs a single, mechanically checkable spelling.

> A topic label **MUST** satisfy all of the following. A producer **MUST NOT** emit a label that
> violates any of them, and a decoder **MUST** reject the containing object rather than repair the
> label.
>
> 1. **NFC only.** The label MUST be in Unicode Normalisation Form C (UAX #15). A decoder that
>    receives a label that is not already NFC MUST reject it; it **MUST NOT** normalise the label
>    and proceed.
> 2. **Bounded length.** The UTF-8 encoding MUST be **≤ 128 bytes**. The empty label `""` is
>    permitted only as the default-feed label, and only where this appendix says so.
> 3. **Forbidden code points.** The label MUST NOT contain U+0000–U+001F (C0 controls), **U+002F
>    (`/`)**, or U+007F (DEL).
> 4. **Comparison is byte equality.** Two labels name the same topic **iff** their NFC UTF-8
>    encodings are byte-identical. No case folding, no width folding, no Unicode collation, no
>    locale-dependent comparison, at any layer — locator, `Subscription`, `FeedHint`, or `FeedHead`.
> 5. **One locator spelling for the empty topic.** The default feed is addressed by the two-segment
>    §22.5.1 path only; the three-segment path with an empty `{topic}` segment MUST NOT be emitted
>    and MUST be rejected (§25.3.2).

**Why each rule is load-bearing.** Without rule 1, `café` in NFC and `café` in NFD are two different
feeds that render identically in every UI a user or an operator would inspect — a subscriber can be
moved onto a feed that *looks* like the one it asked for. Rules 1 and 4 have to be stated together:
normalizing on decode would silently merge two chains that were signed as distinct, so the only
consistent pairing is reject-on-decode plus byte comparison. Without rule 3, `%2F` percent-decodes
into a path separator and a label becomes path structure — the classic locator-confusion bug, and
one an intermediary can exploit without touching a single signed byte. Without rule 2 a label is
unbounded and appears inside a signed head that every subscriber fetches. Without rule 5 the empty
topic has two spellings, one of which contains an empty path segment that proxies and normalizing
routers are entitled to collapse. These five rules did not exist in an earlier revision, which left
the label unconstrained (§25.13 C-07).

### 25.3.5 Per-feed reader state is keyed by `(pub, topic)` (normative)

> A reader implementing `pubsub-1` **MUST** key every piece of §22.4.2 per-feed state — the
> highest-accepted `seq` watermark, the retained `tip`, and the fork-detection record — by the pair
> **`(pub, topic)`**, where `topic` is the value of `FeedHead` key `64` (absent ⇒ `""`, §25.3.1). A
> `FeedHead` for one `(pub, topic)` **MUST NOT** be compared against, nor allowed to advance or roll
> back, the state of any other `(pub, topic)`. `ERR_PUB_FEED_ROLLBACK` (`0x0907`) and
> `ERR_PUB_FEED_CHAIN_BROKEN` (`0x0908`) are raised only **within** one such pair. Two heads under
> one `pub` bearing different `topic` values are **not** equivocation: a reader **MUST NOT** treat
> them as a fork, MUST NOT raise `0x0908`, and **MUST NOT** publish or forward them as evidence that
> the publisher equivocated (§25.13 C-02).

A `pub-1`-only reader is unaffected and needs no change: it never receives key `64` (§25.3.1 rule
2), holds exactly one chain per `pub`, and §22.4.2 applies to it verbatim — for it, `(pub, "")` and
`pub` are the same key by construction.

## 25.4 The `Subscription` object

A `Subscription` is a **signed, self-verifying, bounded-lifetime capability**: a subscriber's
request to receive push hints (§25.6) for one `(pub, topic)` pair. It is the missing protocol object
identified in §25.1.1 — today, "following a feed" leaves no artifact a publisher can point to, audit,
or expire; a `Subscription` is exactly that artifact, modelled on the same self-contained-object
discipline as `PubAnnounce`/`FeedHead` (§22.3, §22.4) and on `PushSubscription`'s device-registration
pattern (§4.9.1, §18.5.5), applied cross-identity instead of device-to-own-node.

### 25.4.1 `Subscription`

```cddl
Subscription = {
  1  => u8,        ; v            PUBSUB object version, = 0 in v0
  2  => suite,     ; suite        signature/hash suite (§18.1.4)
  3  => ik-pub,    ; subscriber   the subscriber's root identity key IK (§1.2)
  4  => ik-pub,    ; feed         the feed author's IK — FeedHead.pub (§22.4.1) — being subscribed to
  5  => tstr,      ; topic        topic label (§25.3); "" = the untopiced/default feed
  6  => ts,        ; issued       ms epoch
  7  => ts,        ; expires      ms epoch — MUST be present (§25.4.2); bounds the subscription's life
  8  => bytes,     ; nonce        ≥ 16 bytes; uniqueness / anti-replay source for `subscription_id`
  9  => ik-pub,    ; signer       operational key that produced `sig`; a DeviceCert (§1.2) chains it to `subscriber`
  10 => sig-val,   ; sig          signer over det_cbor(Subscription ∖ {10}), DS-tag DMTAP-PUB-v0/subscription
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `v` | 1 | `u8` | MUST | PUBSUB object format version. MUST equal `0` in v0; any other value is rejected fail-closed (`ERR_PUB_UNSUPPORTED_VERSION`, `0x0901` — the same code §22.3.1/§22.4.1 already use for this rule, extended in scope to this appendix's objects, §25.12). |
| `suite` | 2 | `suite` | MUST | Algorithm suite for `sig`. Unknown ⇒ reject fail-closed (`0x0901`). |
| `subscriber` | 3 | `ik-pub` | MUST | The subscriber's root identity key. This is the identity a publisher (or a delegated holder, §25.4.3) pushes future `FeedHint`s to (§25.6.2) — an ordinary mesh delivery target, exactly like any MOTE recipient (§4). |
| `feed` | 4 | `ik-pub` | MUST | The publisher identity being subscribed to — the value that appears as `FeedHead.pub` (§22.4.1) for the feed in question. |
| `topic` | 5 | `tstr` | MUST (MAY be empty) | The topic label (§25.3.1), which MUST satisfy §25.3.4 (NFC, ≤ 128 B, no `/` or C0/DEL) — a `Subscription` carrying a non-conforming label is malformed and MUST be rejected on decode. `""` names the default/untopiced feed — the one a pre-DMTAP-PUBSUB §22 deployment already serves (§25.3.3). |
| `issued` | 6 | `ts` | MUST | Creation time (ms epoch, §16.1). |
| `expires` | 7 | `ts` | MUST | Absolute expiry (ms epoch). **There is no indefinite `Subscription`** (§25.4.2) — a `Subscription` with this field absent is malformed and MUST be rejected on decode, not merely treated as non-expiring. |
| `nonce` | 8 | `bytes` | MUST, ≥ 16 B | Source of uniqueness for `subscription_id` (§25.4.1 below), so two `Subscription`s issued by the same subscriber for the same `(feed, topic)` in the same millisecond still identify distinctly. Because the identifier is derived from the **body** and not the signature, `nonce` is the *only* source of that distinctness — a producer MUST draw it from a CSPRNG and MUST NOT reuse one across subscriptions. |
| `signer` | 9 | `ik-pub` | MUST | The operational (device) key that produced `sig`; MUST be authorised by `subscriber` via a `DeviceCert` (§1.2) the verifier checks exactly as §22.3.3 step 4 checks a `PubAnnounce`'s `signer` against its `pub`. `signer` MAY equal `subscriber`. |
| `sig` | 10 | `sig-val` | MUST | Signature by `signer` over `DMTAP-PUB-v0/subscription ‖ 0x00 ‖ det_cbor(Subscription ∖ {10})` (§18.1.6 general rule). Failure is `ERR_PUB_SUBSCRIPTION_SIG_INVALID` (`0x090E`). |

**Identifier (normative).** A `Subscription`'s identifier is derived from its **body — the signed
content — and never from its signature**, under a DS-tag of its own (§25.8):

```
subscription_id = 0x1e ‖ BLAKE3-256( "DMTAP-PUB-v0/subscription-id" ‖ 0x00 ‖ det_cbor(Subscription ∖ {10}) )
```

> A holder **MUST** compute `subscription_id` over `det_cbor(Subscription ∖ {10})` under the DS-tag
> above, and **MUST NOT** include key `10` (`sig`) in the preimage. Two `Subscription`s whose bodies
> are byte-identical are **one subscription**, however their `sig` bytes differ: a holder **MUST**
> treat the second arrival as a duplicate of the first — not a second subscription, not a second
> entry against the aggregate bound (§25.7.1), and not a fresh grant of standing (§25.6.4) — and a
> `SubscriptionRevoke` naming that id **MUST** be honoured against every copy the holder retains
> (§25.5.2), whatever signature bytes each copy carries.

`subscription_id` is what a `SubscriptionRevoke` (§25.5) names. It is a **local binding**, not a
fetch address: a `Subscription` travels inside a sealed MOTE (kind `0x42`, §25.8) and is never
fetched by address, so no address ever accompanies it to compare against. Whoever holds the object
computes the id from the body it has already verified; §25.5 is the identifier's only consumer.

**Why not over the complete signed object (normative rationale, and a correction).** §1.3 forbids
it, in terms that name this construction exactly: *"no identifier, dedup key, or replay-cache key in
this protocol is derived from a signature … An implementation MUST NOT introduce a construction that
depends on signature uniqueness or non-malleability."* Hybrid AND-composition buys **EUF-CMA**, not
**SUF-CMA** (§1.3), so a valid `sig` may be maulable into a *different* valid signature over the
same body. An earlier revision of this subsection derived `subscription_id` over
`det_cbor(Subscription)` — the complete, signed object — by analogy to `announce_id` (§22.3.1). The
analogy did not survive the difference in what the identifier is *used for*:

- **Revocation bypass (the severe one).** Revocation is keyed on the id. A holder that mauls the
  signature of a `Subscription` it has received — or that simply retains a mauled copy handed to it
  by another custodian (§25.4.3) — stores an object whose id is `id(S′) ≠ id(S)`. A later
  `SubscriptionRevoke` naming `id(S)` then matches nothing that holder has, and by §25.5.1's own
  rule an unmatched revoke is **unevaluable**, not "valid but pending". Hint service continues to
  `expires` with the subscriber having done everything the protocol asks of it. The party best
  placed to perform the mauling is the party revocation is aimed at.
- **Quota and standing evasion.** The same subscription re-presented with a mauled signature
  identified distinctly, so it counted twice against §25.7.1's aggregate bound and re-granted
  §25.6.4 standing on replay.
- **A dangling referent.** The old text called `subscription_id` "what any holder recomputes and
  checks before honoring the object" and likened a mismatch to a misaddressed `PubAnnounce`. There
  was nothing to check it against, for the reason given above: a `Subscription` is delivered, not
  fetched.

DS-tagging the preimage keeps the identifier out of every other hash space in this family (§18.1.6's
domain-separation rule, applied to a hash rather than a signature), so a `subscription_id` can never
collide with an `announce_id`, a `PubManifest.id`, or a `FeedEntry` address computed over related
bytes (§25.13 C-03).

**Why self-signed, when it also rides inside a signed MOTE (§25.8).** A `Subscription` is
independently verifiable **without** the `Envelope`/`Payload` that first carried it — exactly the
property that lets a publisher's serving holders (§22.4.3's "any node MAY serve any feed," extended
here to "any holder the publisher delegates to may honour a subscriber list") exchange subscriber
records as portable, self-contained artifacts, and lets a subscriber later *prove* to a third party
"I did subscribe, here is the signed object, here is when it expires" without needing to also
reconstruct the original transport envelope. This mirrors exactly why `PushSubscription` (§18.5.5)
and `PubAnnounce` (§22.3) are self-signed rather than relying solely on an enclosing transport's
authentication.

### 25.4.2 Bounded lifetime is mandatory, not a default (normative)

> A `Subscription` **MUST** carry an `expires` value. A conformant publisher/holder **MUST NOT**
> treat a `Subscription` as active once the current time passes `expires`, and **MUST NOT** push a
> `FeedHint` (§25.6.2) under an expired `Subscription`. Presenting or continuing to honour an expired
> `Subscription` is `ERR_PUB_SUBSCRIPTION_EXPIRED` (`0x090F`).

This is the design's answer to "how does a subscriber list stay bounded, self-pruning edge state
rather than an unbounded durable commitment" (§25.6.1): every entry in a publisher's active-hint list
has a hard expiry baked into the very capability that put it there, so an inactive/abandoned
subscription self-extinguishes even if no revoke is ever sent — the same "TTL, not a promise" posture
the relay-mailbox already applies to buffered ciphertext (§14.3) and the opt-in, research-tier
mixnet applies to key epochs
([docs/research/mixnet.md §4.4.4](docs/research/mixnet.md)). Renewal is simply issuing a fresh
`Subscription` before the old one lapses; there is
no in-place mutation (every §22-family object is immutable and content-addressed, §22.3.4's
`supersedes` precedent applies by analogy but is not required here — a lapsed-and-reissued
subscription is two independent objects, not a revision chain, since there is no "current version of
a subscription" concept to resolve).

### 25.4.3 Admission is the publisher's own policy

A `feed_subscribe` MOTE (kind `0x42`, §25.8) carrying a `Subscription` is, mechanically, an ordinary
MOTE arriving at the publisher's node — a **push to a specific recipient** (the publisher), not a
pull. It is therefore already subject to the recipient's own §9 cold-sender policy exactly as a first
contact from a stranger is (§2.7 steps 5–6): a publisher who has never heard from this subscriber MAY
require a challenge (ARC/PoW/postage/vouch, §9.2) before accepting the `Subscription` at all. No new
anti-abuse mechanism is needed for the subscribe request itself — this is the one place DMTAP-PUBSUB
gets a first layer of admission control **for free**, simply by virtue of being an ordinary MOTE
rather than a bespoke registration call.

Passing the cold-sender gate once does not entitle a subscriber to indefinite standing, and does not
bound the **aggregate** number of subscribers a publisher accumulates over time (a popular feed may
clear the cold-sender gate thousands of times over). §25.7.1 adds the aggregate bound this
per-message gate cannot provide.

A publisher MAY delegate **acceptance and custody** of subscriber records to another holder of its
feed (exactly as serving itself is delegable, §22.4.3) by handing that holder the self-signed
`Subscription` records it has accepted — the holder can independently re-verify each one (§25.4.1)
without trusting the publisher's bookkeeping, and without ever needing the publisher's private key.
Delegated custody covers admission (the cold-sender gate above), the aggregate bound (§25.7.1),
audit, and failover.

> **Delegation does not extend to pushing hints (normative).** A delegated holder **MUST NOT**
> originate a `FeedHint` (§25.6.2) under a `Subscription` in its custody. Every `FeedHint` **MUST**
> be signed by a key authorised by the feed identity it names — the carrying `Payload.from`
> (§18.9.2) equal to `FeedHint.pub`, or chained to it by an unrevoked `DeviceCert` (§1.2) — and a
> subscriber **MUST** verify that binding before treating the hint as solicited (§25.6.4). A holder
> that wishes to push on a publisher's behalf does so **as** that publisher, holding a key the
> publisher's `DeviceCert` chain authorises; there is no third identity a subscriber is obliged to
> accept.

**Why not a delegated-pusher grant (rejected alternative, and the contradiction that forced the
choice).** An earlier revision let a publisher delegate hint-pushing, while §25.6.4 granted standing
only to "that publisher's operational key" and §25.7.3 asserted there is "no second identity to
launder through." The three cannot hold at once, because a delegated pusher **is** a second
identity. Either subscribers reject its hints — delegation dead on arrival — or they accept any
sender able to produce a matching `Subscription`, and a `Subscription` is a portable, self-contained
artifact every custodian holds a copy of, so possession of a copy would become sufficient to push
into every subscriber's inbox with pre-authorised standing (§25.6.4) and pre-authorised fetches
(§25.6.2). A signed `HintDelegation` object with its own DS-tag would close the gap at the cost of a
new signed object, a new registry allocation, and a second revocation lifecycle to keep consistent
with the first. This appendix takes the smaller answer: hints come from the publisher, §25.7.3's
claim becomes true rather than aspirational, and custody delegation — the part carrying the
operational weight — is untouched (§25.13 C-06).

## 25.5 Revocation — the `SubscriptionRevoke` object

### 25.5.1 `SubscriptionRevoke`

```cddl
SubscriptionRevoke = {
  1 => hash,        ; subscription   subscription_id of the Subscription being revoked (§25.4.1)
  2 => ts,          ; ts             revoke time (ms epoch)
  3 => ik-pub,      ; signer         MUST equal the target Subscription's `subscriber`, or an authorized device thereof
  4 => sig-val,     ; sig            signer over det_cbor(SubscriptionRevoke ∖ {4}), DS-tag DMTAP-PUB-v0/subscription-revoke
  5 => u8,          ; v              PUBSUB object version, = 0 in v0 — governs THIS object only
  6 => suite,       ; suite          signature suite for `sig` (§18.1.4) — governs THIS object only
  ? 7 => DeviceCert, ; device_cert   OPTIONAL (§18.4.2): the cert chaining `signer` to the target's `subscriber`
}
```

Keys `1`–`4` keep the number, type and meaning they were first assigned; `v`/`suite` are appended as
`5`/`6` rather than renumbered in, because §18.1.2 forbids reusing a key with a different meaning
across versions of an object. A signature key that is not the last key has precedent in the core
grammar (`DeviceCert`, §18.4.2, signs at key `8` and carries optional keys `9`/`10` after it);
deterministic CBOR orders map keys numerically regardless, so nothing about the encoding depends on
where `sig` sits.

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `subscription` | 1 | `hash` | MUST | The `subscription_id` (§25.4.1) of the `Subscription` being revoked — derived from that object's **body**, so a mauled signature cannot put the target out of reach of this field (§25.4.1). |
| `ts` | 2 | `ts` | MUST | Revocation time. |
| `signer` | 3 | `ik-pub` | MUST | The key that produced `sig`; MUST equal the target `Subscription.subscriber` or be one of its currently-authorised devices (`DeviceCert` chain, §1.2). A revoke signed by anyone else is `ERR_PUB_SUBSCRIPTION_REVOKE_INVALID` (`0x0911`) — only the subscriber who granted a subscription may withdraw it, borrowing the same same-author discipline `supersedes` applies to announces (§22.3.4, §22.3.3 step 5). |
| `sig` | 4 | `sig-val` | MUST | Signature by `signer` over `DMTAP-PUB-v0/subscription-revoke ‖ 0x00 ‖ det_cbor(SubscriptionRevoke ∖ {4})`, under **this object's own** `suite` (key 6) and §18.1.6's representative for it. Failure is also `ERR_PUB_SUBSCRIPTION_REVOKE_INVALID` (`0x0911`). |
| `v` | 5 | `u8` | MUST | PUBSUB object format version of **this revoke**. MUST equal `0` in v0; any other value is rejected fail-closed (`ERR_PUB_UNSUPPORTED_VERSION`, `0x0901`). Independent of the target `Subscription`'s `v`. |
| `suite` | 6 | `suite` | MUST | Algorithm suite for **this revoke's** `sig` (§18.1.4). Unknown ⇒ reject fail-closed (`0x0901`). It governs nothing else, and it need not equal the target `Subscription.suite`. |
| `device_cert` | 7 | `DeviceCert` | OPTIONAL | The `DeviceCert` (§18.4.2) authorising `signer` under the target's `subscriber`, carried inline so an offline holder can complete the §1.2 chain check without directory access. A verifier that uses it MUST check it fully — `ik` equal to the target's `subscriber`, `device_key` equal to this revoke's `signer`, `sig` valid under `ik`, unexpired, unrevoked (§1.5) — and MUST NOT treat mere presence as authorisation. Its absence is not a fault: a verifier that can obtain the chain by its ordinary means proceeds normally. |

Unlike `Subscription`, a `SubscriptionRevoke` needs no internal content-address derivation of its
own — nothing ever points *at* a revoke — but it is self-signed for the identical portability reason
(§25.4.1): a holder the publisher delegates to (§25.4.3) can honour a revoke it never saw travel
through the original MOTE transport, by verifying the object alone.

**A revoke carries its own `v`/`suite`, and why inheritance was wrong (normative).** An earlier
revision omitted both and had a revoke inherit them from the target `Subscription`, reasoning that a
revoke is never evaluated without its target, so the target's discriminators are necessarily in hand
and a second, separately-negotiable algorithm choice would be redundant. The premise is true and is
retained below; the conclusion did not follow, because **`signer` need not be the device that signed
the target**. §25.5.1 admits any currently-authorised device of `subscriber`, and §18.1.6's message
representative is **suite-dependent** — a device holding no key at the target's suite cannot produce
the signature an inherited value demands. A subscriber whose original device was lost, retired, or
rotated onto a different suite would then be unable to revoke at all: an unproducible revoke, for
the one operation whose entire purpose is to work when circumstances have changed. A revoke's
`suite` governs exactly one thing — verification of its own `sig` — and the target's choice has no
claim on it. Unknown `v`/`suite` on a revoke is `ERR_PUB_UNSUPPORTED_VERSION` (`0x0901`),
fail-closed, exactly as for every other object in this family (§25.13 C-04).

The retained premise is normative and easy to get wrong in the other direction: a verifier MUST NOT
attempt to evaluate a `SubscriptionRevoke` without its target. A revoke naming a `subscription_id`
the holder does not have is not "valid but unmatched" — it is **unevaluable**, and MUST NOT be
recorded as an accepted revocation on the strength of its signature alone (that signature proves
only that *someone* signed some bytes; whether that someone is the subscriber is precisely the
check that requires the target, §25.5.1 `signer`).

**Retention duty — a subscriber that loses its own state cannot revoke (normative).**

> A subscriber **MUST** retain, for every `Subscription` it has issued that has neither expired nor
> been revoked, either the exact `det_cbor(Subscription ∖ {10})` bytes or enough state to reproduce
> them byte-for-byte, so that it can compute `subscription_id` (§25.4.1) and issue this object. A
> client **MUST** carry that state through whatever backup, restore, and device-migration path it
> offers (§25.9).

There is no way to name a subscription except by the identifier its body derives (§25.4.1), and no
holder accepts a revoke naming anything else (`0x0911`). A subscriber that discards the state before
`expires` therefore has **no protocol means of revoking at all** and is left with the bounded-lifetime
backstop alone (§25.4.2) — which is exactly why that backstop is a MUST, but it is a poor substitute
for the operation the user actually asked for. This duty is stated because it is otherwise easy to
miss: nothing else in this appendix places a retention obligation on the *subscriber*, and a client
that treats its subscription list as disposable cache satisfies every other rule here while
quietly making revocation impossible (§25.13 C-08).

### 25.5.2 Effect (normative)

> Once a publisher — or any holder with custody of that record (§25.4.3) — has accepted a valid
> `SubscriptionRevoke` naming a given `subscription_id`, it **MUST NOT** push any further `FeedHint`
> under that `Subscription`, **MUST** drop the record from any subscriber list it maintains, and
> **MUST NOT** hand that record on to another holder. A `Subscription` presented after its revoke
> has been accepted — to justify renewed hint service, or handed to a *different* holder that has not
> yet heard the revoke — is `ERR_PUB_SUBSCRIPTION_REVOKED` (`0x0910`). The rule binds **every copy
> whose body hashes to the named `subscription_id`**, whatever signature bytes each copy carries
> (§25.4.1).

**Honest limit, stated plainly rather than hidden (§25.11 item 3).** Revocation is a request the
*publisher* (or its delegated holders) must honour cooperatively — exactly the same posture §6.6 item
8 already discloses for `redact`/`expires` and §22.6.2 discloses for serve refusal. A holder that
never learns of a revoke (network partition, a delegated holder the publisher forgot to notify) may
keep pushing hints under a nominally-revoked `Subscription` until its **mandatory `expires`**
(§25.4.2) finally lapses — which is exactly why `expires` is a MUST and not a SHOULD: it is the
backstop that bounds this residual even when cooperative revocation fails. A subscriber that no
longer wants hints from a non-cooperating holder MAY simply stop honoring them (ordinary local
policy, no protocol obligation) — extra unwanted `FeedHint` MOTEs are wasted bandwidth, never a
confidentiality or integrity breach (§25.6.2's advisory-only status means an unwanted hint asserts
nothing the subscriber must act on).

## 25.6 Push delivery: pull-with-push-hint

### 25.6.1 Why not true push (the stateless-publisher argument)

**True push** — the publisher tracking, for every subscriber, what content that subscriber has
received and retrying the *actual bytes* until every subscriber has every entry — would require the
publisher to hold **durable, growing, per-subscriber content-delivery state** for as long as the
subscription lives. That is precisely the shape of state §0.5's architecture exists to keep out of
the middle and off any party's permanent books: §0.2.1 gives the node a retry queue for its own
outbound MOTEs, bounded by an expiry (§16.1); a would-be feed-push system that must remember
per-subscriber read-position, forever, for a potentially unbounded audience, is a different and much
heavier commitment — it is, in effect, reinventing mailing-list delivery bookkeeping (§5.8) as a
side effect of a feed extension, and inheriting all of §9.9's amplification concerns as a **content**
delivery guarantee rather than as a bounded advisory signal.

DMTAP-PUBSUB instead follows the same move §4.9 (Wake) already made for sleeping devices: a
**content-free-**ish, cheap, best-effort **hint** that says "check now," after which the *existing*
pull machinery (§22.4.4, unchanged) does the actual, verified content transfer. The publisher's only
durable state is the bounded, self-expiring `Subscription` list (§25.4.2) — a set of "who to nudge,"
never "what they have." If every hint for a subscriber's entire subscription lifetime were lost in
transit, the subscriber loses nothing but latency: it can always `feed_head`/`feed_range` on its own
schedule (§22.4.4) and observe the same state a hint would have advertised.

This is not the same mechanism as §4.9's `WakePing` — that wake is **own-node-to-own-device**,
carries **zero** content because the woken device already knows to sync with its own node, and is
sealed under a device push key via a platform push provider. A feed hint is **cross-identity**
(publisher's node to subscriber's identity) and needs, at minimum, to say *which* feed changed — so
it cannot be reduced to Wake's opaque nonce. What DMTAP-PUBSUB borrows from Wake is the **design
principle** (a thin, advisory, non-authoritative nudge that keeps the party in the middle — or, here,
the sender — stateless about content), not the wire object.

### 25.6.2 The `FeedHint` object (kind `0x41`) — advisory, never authoritative

A `FeedHint` is carried as ordinary `Payload.body` content (§18.3.5, §18.3.6) inside a normal sealed
`Envelope` addressed to the subscriber, discriminated by `Envelope.kind = 0x41` exactly as `mail`
(`0x00`) or `chat` (`0x01`) content is discriminated by their own kind values (§21.16) — **not** a
bare unsealed object like `PubAnnounce` (§22.3.2). This is the deliberate choice that gives it
deliver/ack/retry for free (§25.1.2 goal 3): `Payload.from`/`Payload.sig` (§18.9.2) authenticate the
publisher's operational key exactly as they authenticate any sender, sealed sender (§2.2) hides the
publisher's identity from mix/relay intermediaries carrying the hint, and §2.6's retry-until-ack
applies unmodified. Nothing new is invented at the transport layer.

```cddl
FeedHint = {
  1 => ik-pub,     ; pub        the feed author identity (FeedHead.pub, §22.4.1) this hint concerns
  2 => tstr,       ; topic      topic label (§25.3); "" = the default feed
  3 => u64,        ; seq        ADVISORY — the seq the publisher believes is now current; NEVER authoritative
  ? 4 => hash,     ; tip        ADVISORY — a FeedHead.tip hint; NEVER authoritative
  ? 5 => bytes,    ; announce   OPTIONAL: det_cbor(PubAnnounce) for the entry at `seq` — a bounded convenience (§25.6.3)
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `pub` | 1 | `ik-pub` | MUST | Which feed changed. |
| `topic` | 2 | `tstr` | MUST (MAY be empty) | Which topic-scoped chain (§25.3) changed. MUST satisfy §25.3.4; a hint carrying a non-conforming label is malformed and MUST be rejected on decode, never normalised. |
| `seq` | 3 | `u64` | MUST | The publisher's own belief about the new tip `seq`. **Advisory only** (§25.6.2 below) — never a substitute for a verified `feed_head` fetch. |
| `tip` | 4 | `hash` | OPTIONAL | The publisher's own belief about the new `FeedHead.tip`. Advisory, same status as `seq`. |
| `announce` | 5 | `bytes` | OPTIONAL | The complete, deterministically-encoded `PubAnnounce` (§22.3.1) at the hinted position — an inlining optimisation (§25.6.3), not a trust shortcut. |

**Normative: advisory status (load-bearing).**

> A `FeedHint`'s `seq` and `tip` fields **MUST NOT** be used to advance a subscriber's accepted-`seq`
> watermark (§22.4.2), and **MUST NOT** be treated as evidence that content has been delivered. A
> conformant subscriber that receives a `FeedHint` **MUST** perform (or schedule) an ordinary,
> independently-verified `feed_head`/`feed_range` fetch (§22.4.4) — or, if `announce` is present,
> independently verify it exactly as a pulled `PubAnnounce` (§22.3.3, §25.6.3) — before accepting any
> change in feed state. A hint is a *reason to check*, never itself a *fact checked*.
>
> A subscriber **MUST NOT** perform, or schedule, any fetch on the strength of a `FeedHint` that
> fails the standing test of §25.6.4 — including the identity binding that requires the carrying
> `Payload.from` to be `FeedHint.pub` or a device it authorised (§25.4.3). "A reason to check" is a
> reason granted by a subscription the subscriber itself issued, and by nothing else.

This is the same non-authoritative posture `substrate/FEEDS.md` §5.4 already establishes for its
advisory fetch-hint registry ("a client MUST NOT treat a blob fetched from an unlisted source
differently" — here, a client MUST NOT treat a *hinted* `seq` differently from one it discovered by
blind polling): the **content address and the signed `FeedHead`/`FeedEntry` chain are the only
authority**; the hint only changes *when* a subscriber decides to check, never *what* it accepts once
it does.

### 25.6.3 Bounded inline delivery (a bounded form of true push)

The design brief for this appendix asks explicitly whether *true* push — delivering the actual
content, not just a nudge — is ever warranted, and requires bounding it if so. It is, in exactly one
narrow case: **the `announce` field MAY carry the complete, already-signed `PubAnnounce` bytes for
the hinted entry**, saving the subscriber a round trip when the announce is small enough to travel
inline.

This is bounded in the ways that matter:

- **It changes no trust model.** A subscriber **MUST** independently verify an inlined `announce`
  exactly as it would verify one fetched by pull (recompute `announce_id`, verify `sig`/`signer`
  chain, §22.3.3) before treating it as valid. Presence of inline bytes is never a shortcut around
  verification — a lying intermediary that tampered with the inlined bytes is caught the identical
  way a lying PUB server is caught (§22.5.1's "verification is the client's job, always").
- **It changes no size ceiling.** The inlined bytes ride inside `Payload.attach`/`body` under the
  ordinary MOTE size discipline (the bucket ladder,
  [docs/research/mixnet.md §4.4.1](docs/research/mixnet.md)/§16.3) — a large announce (many `roots`,
  large `meta`) simply does not fit and MUST NOT be inlined; the publisher falls back to the
  seq/tip-only hint and the subscriber pulls the announce and any referenced manifest/chunks the
  ordinary way (§22.5).
- **It creates no new durable state.** Inlining is a per-hint, stateless choice the publisher's
  client makes at push time (does this announce's encoding fit the bucket the MOTE is already
  being padded to) — it requires no bookkeeping about what any given subscriber has previously
  received, unlike true content-delivery push (§25.6.1).

A subscriber MUST treat the absence of `announce` identically to its presence-but-oversized case:
fetch and verify via the ordinary §22.4.4 pull path. Inlining is purely a latency/round-trip
optimisation within an already-bounded MOTE, never a second delivery guarantee alongside the pull
path.

### 25.6.4 Standing: how a hint avoids the cold-sender gate

The act of sending a `Subscription` establishes reciprocal standing, symmetrically to §25.4.3's
observation about the *subscribe* direction: a subscriber that has issued a `Subscription` to a
publisher has, by that act, pre-authorised `FeedHint` MOTEs (kind `0x41` only) from that publisher's
operational key for the `(pub, topic)` pair named, until `expires`. A subscriber's node SHOULD record
this locally and treat a matching, unexpired `FeedHint` the way it treats mail from an established
contact (§2.7 step 5) — no per-hint challenge is needed, because the subscriber already asked for
this, exactly the reasoning §22.6.3 used to exempt `pub_announce` from a challenge (there, because
announces are pulled; here, because the push was solicited).

**Standing is scoped, and a hint outside it is discarded before any fetch (normative).**

> A subscriber **MUST** discard a `FeedHint` — **without performing or scheduling any fetch it would
> otherwise trigger** (§25.6.2) — unless **all three** of the following hold:
>
> 1. the subscriber itself issued a `Subscription` (§25.4) that is currently **active**: not expired
>    (§25.4.2) and not revoked (§25.5.2);
> 2. that `Subscription`'s `feed` equals the hint's `pub`, **and** its `topic` is byte-equal to the
>    hint's `topic` under §25.3.4's comparison rule; and
> 3. the carrying `Payload.from` (§18.9.2) is the identity named by `FeedHint.pub`, or a device
>    authorised by it through an unrevoked `DeviceCert` chain (§1.2, §25.4.3).
>
> A hint failing (3) is an ordinary unsolicited push and takes the cold-sender disposition below. A
> hint failing (1) or (2) **MUST** be discarded and **MUST NOT** be acted on, even where the sender
> is an otherwise established contact and even where the hint is perfectly well-formed.

**Why the fetch has to be gated, and not merely the notification (normative rationale).** §25.7's
fan-out analysis counts only the deliveries a publisher's own node pays for. Nothing there — and
nothing in an earlier revision of this subsection — required a hint's `(pub, topic)` to be a feed the
recipient had actually subscribed to. A publisher with 50 000 subscribers could therefore emit hints
naming a **third party's** `(pub, topic)`, and §25.6's own "check now" rule converted one signed act
into 50 000 independently-verified fetches aimed at an identity that had never published to any of
them and never accepted a subscription from any of them: **fan-in amplification**, with the
amplification factor set by the attacker's own subscriber count and pointed outward. The
subscriber-side rate bound of §25.7.2 does not reach it — each subscriber sees exactly one hint, well
inside its budget, and it is the *victim*, who holds no relationship with any of them, that absorbs
the aggregate. Matching every hint against a subscription the subscriber itself issued is what bounds
fan-in by the *victim's own* subscriber count rather than by the attacker's (§25.13 C-05).

A `FeedHint` failing test (3) — an ordinary unsolicited push — receives **exactly the disposition
§2.7/§9.2 already define for a cold sender**: deferred to the requests area (§2.7a), rate-limited,
never surfaced as a normal notification, and not acked. No new wire error is defined for this case;
it is not a malformed object, merely one this recipient did not ask for, and DMTAP already has a
complete answer for that.

### 25.6.5 Composition with Wake (§4.9) for sleeping devices

If the subscriber's device is asleep when a `FeedHint` arrives at the subscriber's own node, that is
an **entirely separate, already-solved problem**: the subscriber's own node applies §4.9 unchanged —
it wakes the sleeping device with a content-free `WakePing` exactly as it would for any other
inbound MOTE. DMTAP-PUBSUB needs no awareness of device sleep state and defines no interaction with
Wake beyond "it composes for free," because a `FeedHint` is, from the receiving node's perspective, an
ordinary MOTE like any other.

## 25.7 Fan-out & anti-abuse (§9.9 governance)

Posting a new entry and pushing a `FeedHint` to every active subscriber is structurally the same
shape as §5.8's group-address fan-out — one act, *N* deliveries — so §9.9's governing rules apply
directly; this section states how, without re-deriving them.

### 25.7.1 Publisher-side: admission bound

> A publisher's node (or delegated holder, §25.4.3) **MUST** apply an admission policy bounding the
> **aggregate** number of active `Subscription`s it honours per feed/topic (and MAY additionally cap
> per-subscriber pending-subscription counts or subscribe rate). Exceeding the configured bound is
> `ERR_PUB_SUBSCRIBE_QUOTA` (`0x0912`, DENY_POLICY) — a policy deny at the holder, never a
> security/crypto gate, the DMTAP-PUBSUB analogue of `ERR_PUB_SERVE_QUOTA` (§22.6.3, `0x090D`).

This is the aggregate bound §25.4.3 notes the per-message cold-sender gate cannot itself provide: a
gate stops any *one* stranger from imposing itself for free, but says nothing about the total size
the resulting list is allowed to grow to.

### 25.7.2 Subscriber-side: dual-ended rate bound (mirrors §4.9.4)

> A subscriber's own node **MUST** enforce a bounded inbound `FeedHint` rate **per publisher (or per
> `(pub, topic)`)**, independent of whatever limiter the publisher's own node applies, as a
> fail-closed backstop. Hints beyond the budget are dropped: `ERR_PUB_HINT_RATE_LIMITED` (`0x0913`,
> DROP_SILENT).

This is the identical **dual-ended** discipline §4.9.4 already applies to Wake ("rate-limited at both
ends... so a misbehaving relay that replays/floods cannot exceed the budget") — here applied to a
compromised or simply misconfigured publisher key that starts posting (and therefore hinting) at a
damaging rate. Subscribing once does not entitle a publisher to an unbounded claim on a subscriber's
battery or bandwidth any more than a contact relationship entitles a sender to unlimited mail (§9).

### 25.7.3 Origin accountability, and why this case is simpler than §9.9's general one

§9.9 distinguishes **member-visible channels**, where the poster knows the membership and can mint a
per-member proof, from **hidden-membership lists**, where a separate list-operator/committer must be
trusted to vouch for fan-out without revealing membership to the poster. DMTAP-PUBSUB's shape
collapses that distinction: **the publisher is simultaneously the poster and the entire membership
registry** — there is no separate committer, no third-party list operator, and no hidden-membership
trust problem to disclose, because a publisher's own `Subscription` list is, by construction, exactly
as visible to the publisher as a mailing-list operator's roster already is to that operator (§9.9's
own baseline comparison). Origin accountability is therefore immediate and structural, not a proof
carried through an intermediary: every `FeedHint` a recipient receives already names, and is signed
by, the same feed identity a recipient's ordinary per-sender policy (§9.2) would apply to any other
MOTE from that identity — there is no laundering vector because there is no second identity to
launder through.

## 25.8 Wire allocations & capability negotiation

| Registry | Allocation |
|---|---|
| Message Kinds (§21.16) | `0x41 feed_hint` — an ordinary sealed-MOTE kind (§25.6.2), Payload-wrapped, riding the existing deliver/ack/retry path; `0x42 feed_subscribe` — an ordinary sealed-MOTE kind carrying a `Subscription` (§25.4) as `Payload.body`; `0x43 feed_unsubscribe` — an ordinary sealed-MOTE kind carrying a `SubscriptionRevoke` (§25.5) as `Payload.body`. All three Specification Required, extension range (§2.3, §21.16), continuing the block `pub_announce` (`0x40`) opened. |
| Capability Tokens (§21.22) | `pubsub-1` — Specification Required; node/operator opt-in to originating and/or honoring `Subscription`/`SubscriptionRevoke`/`FeedHint`, **and** to serving or reading a topic-scoped `FeedHead` (§10.2). A node **MUST** advertise `pub-1` (§22.6.1) to meaningfully advertise `pubsub-1` — DMTAP-PUBSUB extends feeds and has no meaning without them. Topic-scoped serving/reading (§25.3) needs `pubsub-1`, not `pub-1` alone: a topic-scoped `FeedHead` carries the reserved key `64` (§25.3.1), and §18.1.2's signed-object extension rule requires the paired capability before that key may be sent (§25.3.2; an earlier revision of this row claimed the opposite, §25.13 C-01). The **default** (`topic = ""`) feed is unaffected and remains a `pub-1`-only surface. |
| Error/Status Codes (§21.14) | Seven new code points **within** the existing subsystem byte `0x09` DMTAP-PUB already owns (§21.24b): `0x090E`–`0x0913` and `0x0915` (§25.12). `0x0914` (`ERR_PUB_SUITE_BELOW_FLOOR`) falls between the two and is **not** one of this appendix's allocations — it is §22's own (§22.10, §21.24b) — so the DMTAP-PUBSUB block is non-contiguous by one code, not a numbering error. This is a Specification-Required addition **within an existing subsystem**, not a new subsystem byte — this appendix extends §22's own extension rather than registering a fresh one (§21.14's lighter-weight allocation policy for `NN` within an existing `SS`). |
| Signature DS-tags (§18.9 convention) | `DMTAP-PUB-v0/subscription` (`Subscription.sig` preimage, §25.4.1), `DMTAP-PUB-v0/subscription-revoke` (`SubscriptionRevoke.sig` preimage, §25.5.1) — reserved, distinct from every `DMTAP-v0/…`, `DMTAP-PUB-v0/…`, and `DMTAP-SYNC-v0/…` DS-tag already registered (§21.24b, §21.24c). `FeedHint` needs no DS-tag of its own — it is ordinary `Payload` content authenticated by the existing `Payload.sig` preimage (§18.9.2), unchanged. |
| Hash domain-separation tags (§18.1.6's DS rule, applied to a hash rather than a signature) | `DMTAP-PUB-v0/subscription-id` (`subscription_id`'s preimage tag, §25.4.1) — reserved and distinct from every signature DS-tag in the row above and from `DMTAP-PUB-v0/feed`'s hash domain (§22.4.1), so a `subscription_id` cannot collide with an `announce_id`, a `PubManifest.id`, a `FeedEntry` address, or a signature preimage computed over related bytes. |

A peer that has not advertised `pubsub-1` MUST treat kinds `0x41`–`0x43` under the ordinary
forward-compatibility rule already governing unassigned/unimplemented kinds (§21.16, §10.1): it MUST
NOT `ack` a kind it cannot validate, and MAY ignore it. No flag day, no required upgrade.

## 25.9 Client requirements

- **Bounded-lifetime disclosure (MUST — normative UX).** Before a client issues a `Subscription` on
  the user's behalf, it MUST make the bounded, best-effort nature of the relationship visible: that
  the underlying feed content is plaintext and public exactly as any §22 feed is (§22.9 items 1–2,
  unchanged by this appendix), and that a revoke (§25.5.2) is honoured cooperatively — a
  non-cooperating or partitioned holder MAY continue pushing hints until the subscription's own
  `expires`, never indefinitely, but not necessarily the instant a revoke is sent. This mirrors the
  spec's existing pattern of disclosing a hard limit before the user relies on it (§22.7's
  irrevocability warning, §6.6 item 8's cooperative-only `redact`/`expires`).
- **Verify before acting on a hint (MUST).** Per §25.6.2's advisory-status rule: a client MUST NOT
  surface, badge, or otherwise act on a `FeedHint`'s `seq`/`tip`/inlined `announce` until it has been
  independently verified through the ordinary §22 pull/verification path.
- **Revoke, never silently stop honoring (SHOULD).** When a user unsubscribes, a client SHOULD emit a
  `SubscriptionRevoke` (§25.5) promptly, rather than relying solely on the bounded `expires` to clean
  up — the same "ask nicely first, bounded fallback second" posture the whole design leans on.

## 25.10 Conformance & fail-closed table

DMTAP-PUBSUB adds the following invariants to the auditable fail-closed set (§10.7), in the §10.7 /
§22.8 format. A conformant implementation of `pubsub-1` enforces every row; a node that never
advertises `pubsub-1` is not held to any of them.

| Invariant | Clause | Trigger | Behaviour / error on violation |
|-----------|--------|---------|-------------------------------|
| **`Subscription` unknown version/suite** | §25.4.1 | a `v`/`suite` this implementation does not support | reject, never guess; `ERR_PUB_UNSUPPORTED_VERSION` `0x0901`, FAIL_CLOSED_BLOCK (the same code §22.3.1/§22.4.1 use, scope extended to this appendix's objects). |
| **`Subscription` missing mandatory `expires`** | §25.4.2 | decode of a `Subscription` lacking key `7` | malformed, reject on decode — no indefinite subscription exists |
| **`Subscription` signature / DeviceCert chain** | §25.4.1 | `sig` fails under `signer`, or `signer` not authorised by `subscriber` | reject; `ERR_PUB_SUBSCRIPTION_SIG_INVALID` `0x090E`, FAIL_CLOSED_BLOCK |
| **`subscription_id` computed over `sig`, or two body-identical `Subscription`s treated as distinct** | §25.4.1 | an implementation includes key `10` in the `subscription_id` preimage, or stores a mauled-signature duplicate as a second subscription | non-conformant; MUST derive from `det_cbor(Subscription ∖ {10})` under the `subscription-id` DS-tag and MUST treat body-identical copies as one subscription for revocation, quota and standing purposes alike (§25.13 C-03) |
| **Expired `Subscription` honoured** | §25.4.2 | current time > `expires`, and the holder still treats it as active / pushes a hint under it | reject/stop; `ERR_PUB_SUBSCRIPTION_EXPIRED` `0x090F`, FAIL_CLOSED_BLOCK |
| **Delegated holder originates a `FeedHint`** | §25.4.3 | a holder with `Subscription`-record custody, but not authorised under the feed's own `DeviceCert` chain, signs and sends a `FeedHint` | the hint fails the standing test's identity binding (§25.6.4 test 3) and is disposed of as an unsolicited push; a subscriber MUST NOT treat custody of a `Subscription` copy as authorisation to originate hints (§25.13 C-06) |
| **`SubscriptionRevoke` cross-subscriber** | §25.5.1 | `signer` ≠ the target `Subscription.subscriber` (or an authorised device thereof), or `sig` invalid | reject; `ERR_PUB_SUBSCRIPTION_REVOKE_INVALID` `0x0911`, FAIL_CLOSED_BLOCK — only the subscriber who granted a subscription may withdraw it |
| **`SubscriptionRevoke` unknown version/suite** | §25.5.1 | a `SubscriptionRevoke` carrying a `v`/`suite` (its own, keys 5/6 — independent of the target `Subscription`'s) this implementation does not support | reject, never guess; `ERR_PUB_UNSUPPORTED_VERSION` `0x0901`, FAIL_CLOSED_BLOCK (§25.13 C-04) |
| **Revoked `Subscription` honoured** | §25.5.2 | a `Subscription` presented/acted on after a valid matching `SubscriptionRevoke` has been accepted, for *any* copy whose body hashes to the named `subscription_id` | reject; `ERR_PUB_SUBSCRIPTION_REVOKED` `0x0910`, FAIL_CLOSED_BLOCK |
| **Subscriber discards state needed to revoke** | §25.5.1 | a subscriber fails to retain `det_cbor(Subscription ∖ {10})` (or reproducing state) for an active `Subscription` through backup/restore/migration | non-conformant; the subscriber is left with only the bounded-`expires` backstop and has no protocol means of revoking (§25.13 C-08) |
| **Unsolicited `FeedHint`** | §25.6.4 | a `feed_hint` MOTE whose carrying `Payload.from` is not the identity named by `FeedHint.pub` (or an authorised device) | not a wire fault — ordinary §2.7/§9.2 cold-sender disposition (defer to requests area, §2.7a); no new error code |
| **`FeedHint` outside subscriber's own standing** | §25.6.4 | a `FeedHint` whose `(pub, topic)` does not match a `Subscription` the recipient itself issued and that is currently active | MUST be discarded before any fetch is performed or scheduled — never merely un-notified; closes fan-in amplification against an uninvolved third party (§25.13 C-05) |
| **`FeedHint` treated as authoritative** | §25.6.2 | a client advances its accepted-`seq` watermark, or treats content as delivered, from `FeedHint.seq`/`tip`/`announce` without independent §22.4.2/§22.3.3 verification | non-conformant client; the hint is a reason to check, never a fact checked |
| **Inlined `announce` unverified** | §25.6.3 | an inlined `FeedHint.announce` treated as valid without recomputing `announce_id` / verifying `sig`/`signer` chain | non-conformant client; verify exactly as a pulled `PubAnnounce` (§22.3.3), or reuse `0x0904`/`0x0905` on failure |
| **Publisher subscriber-admission bound** | §25.7.1 | aggregate active-subscription count (or subscribe rate) past a holder's configured bound | `ERR_PUB_SUBSCRIBE_QUOTA` `0x0912`, DENY_POLICY |
| **Subscriber inbound hint-rate bound** | §25.7.2 | inbound `FeedHint` rate from one publisher/topic past the subscriber's own configured budget | `ERR_PUB_HINT_RATE_LIMITED` `0x0913`, DROP_SILENT — excess dropped, never surfaced |
| **Topic backward compatibility** | §25.3.3 | a publisher's pre-existing feed is altered, renumbered, or orphaned upon adopting topics | non-conformant; `topic = ""` MUST remain byte-for-byte the pre-existing chain |
| **`FeedHead` topic-mismatch / capability leak** | §25.3.1, §25.3.2 | a served `FeedHead`'s key `64` (absent ⇒ `""`) does not byte-equal the requested topic, or key `64` is sent to a peer that has not advertised `pubsub-1` | reader: reject, `ERR_PUB_FEED_TOPIC_MISMATCH` `0x0915`, FAIL_CLOSED_BLOCK; server: MUST NOT send, protocol violation (§25.13 C-01) |
| **Topic-labelled reader state cross-contaminated** | §25.3.5 | §22.4.2 anti-rollback watermark, retained `tip`, or fork record compared/advanced across two different `(pub, topic)` pairs | non-conformant; state MUST be keyed by `(pub, topic)`; two heads differing only in `topic` MUST NOT be raised as `0x0907`/`0x0908` or published as equivocation evidence (§25.13 C-02) |
| **Non-conforming topic label accepted or normalised** | §25.3.4 | a `Subscription.topic`, `FeedHint.topic`, `FeedHead` key `64`, or `{topic}` locator segment fails NFC-only, the ≤ 128 B bound, the forbidden-code-point set, or the one-spelling-for-empty rule | reject the containing object/request on decode; MUST NOT normalise and proceed (§25.13 C-07) |

The governing rule of §10.7.5 applies unchanged: a DMTAP-PUBSUB security-relevant failure is either
refused (fail closed) or surfaced as an explicit choice, never a silent degradation.

## 25.11 Security considerations / honest limits

Stated plainly, per the project's honest-limits governance (§6.6, §6.9, §22.9's precedent for this
extension family). None of these is a defect to be fixed; each is an inherent consequence of the
design this appendix makes, disclosed for what it is.

1. **Encrypted broadcast to a large, open subscriber set is an unsolved problem, not an oversight.**
   MLS gives confidentiality with **known** membership (§5.8); §22/§25 give scale and open join with
   **plaintext**. Wanting millions of subscribers, end-to-end encryption, *and* open join, all at
   once, is out of scope for this appendix and for v1 of DMTAP generally. For the overwhelming
   majority of machine-to-machine cases this appendix targets — release feeds, status/event streams,
   changelogs, security advisories — **authenticated-but-plaintext is exactly what a webhook already
   is**, and DMTAP-PUBSUB's guarantee (signed, content-addressed, verifiable without trusting the
   transport) already exceeds a bare webhook's. A publisher whose audience is genuinely bounded and
   whose content must be confidential already has the right tool: an MLS channel (§5.8), not this
   extension.
2. **Hint delivery is best-effort; only the pull path is guaranteed by the protocol.** §2.6's
   deliver/ack/retry gives at-least-once delivery of the *hint MOTE itself* once it is sent, but
   nothing compels a publisher to send one for every entry, at every subscriber, promptly — a
   publisher that never emits a hint at all remains conformant (§25.1.3); the pull path is the only
   thing a subscriber may rely on for correctness.
3. **Revocation is cooperative, exactly like every other un-share bound in this spec.** §25.5.2's
   residual — a holder that never learns of a revoke may keep pushing hints until natural `expires`
   — is the same shape as §6.6 item 8's `redact`/`expires` bound and §22.6.2's "you cannot compel a
   holder to stop serving," now applied to push rather than serve. The mandatory bounded lifetime
   (§25.4.2) is what keeps the residual finite rather than open-ended.
4. **A `Subscription`'s existence is itself metadata.** A publisher (and any holder it delegates to,
   §25.4.3) necessarily learns *who* subscribed and *when* — there is no mechanism in this appendix
   for anonymous subscription, mirroring §22.9 item 2's disclosure that publisher-side metadata
   (`pub`, `roots`, `ts`, the whole feed) is public by design. A subscriber for whom the mere fact of
   following a given feed is sensitive should not use a `Subscription` at all and should instead pull
   anonymously (§22.5.1's anonymous, unauthenticated reads remain available regardless of whether
   this appendix is implemented).
5. **Compromise of an operational signing key extends to push standing.** Exactly as §22.9 item 5
   discloses for `PubAnnounce`/`FeedHead`, a compromised `signer` key can mint `FeedHint`s (and, for
   the subscriber side, `Subscription`/`SubscriptionRevoke` objects) under the identity until the
   device is revoked (§1.5). Keeping `IK` cold and signing with a revocable operational key (§1.2a)
   bounds this exactly as it already bounds the base extension.

## 25.12 Error registry (`ERR_PUB_*`, continued — `0x090E`–`0x0913` and `0x0915`)

These codes extend the subsystem byte `0x09` DMTAP-PUB already owns (§21.24b) — this appendix
registers no new subsystem. Codes follow the §21 conventions and responder-action vocabulary
(§21.2); the table below is authoritative for this range, exactly as §22.10 is authoritative for
`0x0901`–`0x090D`.

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x090E` | `ERR_PUB_SUBSCRIPTION_SIG_INVALID` | `Subscription` verification (§25.4.1) | `sig` fails under `signer`, or `signer` is not authorised by `subscriber` (DeviceCert chain). | No | FAIL_CLOSED_BLOCK |
| `0x090F` | `ERR_PUB_SUBSCRIPTION_EXPIRED` | `Subscription` lifecycle check (§25.4.2) | A `Subscription` is presented, or still being honoured, past its `expires`. | Yes (subscriber may reissue a fresh `Subscription`) | FAIL_CLOSED_BLOCK |
| `0x0910` | `ERR_PUB_SUBSCRIPTION_REVOKED` | `Subscription` lifecycle check (§25.5.2) | A `Subscription` matching an already-accepted `SubscriptionRevoke` is presented or still being acted on. | No | FAIL_CLOSED_BLOCK |
| `0x0911` | `ERR_PUB_SUBSCRIPTION_REVOKE_INVALID` | `SubscriptionRevoke` verification (§25.5.1) | `sig` fails under `signer`, or `signer` does not match the target `Subscription.subscriber` (or an authorised device thereof). | No | FAIL_CLOSED_BLOCK |
| `0x0912` | `ERR_PUB_SUBSCRIBE_QUOTA` | Subscription admission policy (§25.7.1) | A holder's aggregate subscriber-admission bound (count/rate per feed or topic) is exceeded. A policy deny, never a security/crypto gate. | Yes (after freeing / under a laxer policy) | DENY_POLICY |
| `0x0913` | `ERR_PUB_HINT_RATE_LIMITED` | Subscriber-side inbound rate policy (§25.7.2) | A subscriber's configured per-publisher (or per-topic) inbound `FeedHint` budget is exceeded; excess hints are dropped. | Yes (next budget window) | DROP_SILENT |
| `0x0915` | `ERR_PUB_FEED_TOPIC_MISMATCH` | `FeedHead` verification, topic-scoped fetch (§25.3.1) | A verified `FeedHead`'s key `64` (absent ⇒ `""`) is not byte-equal to the topic the reader requested. Not allocated contiguously with the block above: `0x0914` (`ERR_PUB_SUITE_BELOW_FLOOR`) belongs to §22 (§22.10), allocated between this appendix's two registration passes. | No | FAIL_CLOSED_BLOCK |

## 25.13 Change log — normative corrections

This document is pre-1.0 and is corrected in the open, in the same discipline
[`substrate/SYNC.md` §14](substrate/SYNC.md) established: a defect found by an adversarial protocol
audit is fixed here **and recorded here**, never silently edited. Each entry states what changed,
whether it changes **wire bytes** (a CDDL shape, a DS-tag/hash-domain preimage, or a value carried
on the wire — a KAT/vector consumer must be updated) or is a **behavioural rule** (a MUST governing
what a conformant implementation does with bytes whose shape is unchanged), and how it was found.

| # | Change | Class | Found by |
|---|--------|-------|----------|
| **C-01** | §25.3.1/§25.3.2 bind the topic into the signed `FeedHead` (new key `64`), gated behind `pubsub-1`; replaces the false §22.2.3-parity claim. See §25.3.1. | **NORMATIVE — wire bytes.** `FeedHead` gains one OPTIONAL key (strictly additive: absent on every pre-existing default-feed head, so no previously valid object or signature changes, §25.3.3); one new error code `0x0915` (§25.12); the capability-token row (§25.8) is corrected, not merely reworded. A `pub-1`-only verifier handed a topic-bearing head rejects it fail-closed under §18.1.2 — the intended behaviour, not a regression, since a topic chain silently consumed as the default feed is exactly the confusion key `64` exists to prevent. | Adversarial protocol audit (item PS-1/PS-2): keying `feed_head`/fork-detection state on `pub` alone while topic lived only in the locator was found to make the two consequences above unavoidable, and the §22.2.3 "parity" claim was found to rest on a hash-preimage property the locator design never had. |
| **C-02** | §25.3.5 is new: `pubsub-1` reader state is keyed by `(pub, topic)`; two heads differing only in `topic` are not equivocation. See §25.3.5. | **NORMATIVE — behavioural rule; no additional wire bytes** (the field it keys on is C-01's). Un-implementing this rule while implementing C-01 reintroduces the self-inflicted-equivocation failure C-01 was partly written to close. | Adversarial protocol audit (item PS-1/PS-2), same investigation as C-01 — found by tracing what §22.4.2's existing state-keying does once a second topic exists under one `pub`. |
| **C-03** | §25.4.1's `subscription_id` is re-derived from the `Subscription` body under a dedicated DS-tag, `sig` excluded, closing a signature-malleability revocation bypass. See §25.4.1. | **NORMATIVE — wire bytes.** The `subscription_id` preimage and value change for every `Subscription` (a new hash domain-separation tag is reserved, §25.8); `SubscriptionRevoke.subscription` (key `1`) carries the new value. No CDDL shape changes on `Subscription` itself. | Adversarial protocol audit (item PS-3), applying §1.3's signature-derived-identifier prohibition to §25.4.1 by the same reasoning being applied in parallel to `announce_id`/`Identity_id` in §18/§22 — kept consistent with that fix's derivation style (DS-tagged hash over the body, `sig` excluded). |
| **C-04** | §25.5.1's `SubscriptionRevoke` gains its own `v`/`suite` (keys `5`/`6`) instead of inheriting the target `Subscription`'s, so a device at a different suite can still revoke. See §25.5.1. | **NORMATIVE — wire bytes.** `SubscriptionRevoke` gains three keys (`5`, `6`, REQUIRED; `7`, OPTIONAL); every previously-describable revoke lacked a `v`/`suite` of its own, so this is a strict, additive widening, not a break of a previously-conformant object (there was none, since the prior text specified no encoding for `v`/`suite` at all). | Adversarial protocol audit (item PS-4): found by asking which device produces a revoke's signature when the original signing device is gone, and observing that §18.1.6's suite-dependent representative makes "inherit the target's suite" sometimes unproducible. |
| **C-05** | §25.6.4 requires discarding, before any fetch, a `FeedHint` whose `(pub, topic)` the recipient holds no active `Subscription` for, closing a fan-in amplification vector. See §25.6.4. | **NORMATIVE — behavioural rule; no wire bytes.** No CDDL, DS-tag or error-code change; the fix is a MUST on when a subscriber may act on a `FeedHint` it has already decoded and authenticated. | Adversarial protocol audit (item PS-5): found by tracing who pays for the fetches a `FeedHint` triggers and observing that §25.6.4's identity check constrains only the *sender*, never the *target* the hint names. |
| **C-06** | §25.4.3 states that delegating `Subscription` custody does not delegate the authority to originate `FeedHint`s. See §25.4.3. | **NORMATIVE — behavioural rule; no wire bytes.** No new object, key or error code; the existing §25.6.4 identity-binding test (test 3) already enforces this once stated, so a delegated holder's hint fails standing and is disposed of as an ordinary unsolicited push. | Adversarial protocol audit (item PS-6): found by checking §25.4.3's delegation grant against §25.6.4's standing rule and §25.7.3's accountability claim and finding the three mutually inconsistent. |
| **C-07** | §25.3.4 is new: a normative topic-label grammar (NFC-only, ≤ 128 B, forbidden code points, byte-equality comparison). See §25.3.4. | **NORMATIVE — new constraint on an existing field's valid values; no CDDL shape change.** No key is added, retyped or removed; what is newly non-conformant is a producer or decoder that accepted, emitted, or silently normalised a label failing any of the five rules. `Subscription.topic`/`FeedHint.topic`/`FeedHead` key `64`'s table rows are updated to cite this section. | Adversarial protocol audit (item PS-7): found by asking what a topic label is compared *against*, and observing that neither NFC-equivalence nor path-separator characters had a stated rule. |
| **C-08** | §25.5.1 is new: a subscriber MUST retain the state to compute `subscription_id` for every unexpired, unrevoked `Subscription` it issued, through backup/restore/device-migration. See §25.5.1. | **NORMATIVE — behavioural rule; no wire bytes.** No CDDL, DS-tag or error-code change; a client-side retention MUST, parallel to the existing backup/restore/migration expectations §25.9 already states for other client obligations. | Adversarial protocol audit (item PS-8): found by asking what happens when a subscriber that issued a `Subscription` has discarded its own copy before `expires`, and observing that no protocol path lets it name the object it meant to revoke. |

**Standing rule.** A defect between this document and an implementation is resolved by deciding
**which side is right on the merits** and correcting the other in the open, exactly as
[`substrate/SYNC.md`](substrate/SYNC.md) §14 states it. **C-01, C-02, C-03 and C-04 change wire
bytes** (a CDDL shape, a hash-domain preimage, or a value carried in an existing field) and are
classed NORMATIVE — wire bytes accordingly; **C-05, C-06, C-07 and C-08 change no byte** — each adds
or sharpens a MUST governing what a conformant implementation does with, or requires of, bytes whose
shape is unchanged — and are classed NORMATIVE — behavioural rule. None is classed INFORMATIVE: unlike
§24.14's non-normative migration guidance, every entry in this table corrects a `pubsub-1`
conformance requirement, not advice.
