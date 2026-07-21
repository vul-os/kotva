# 25. DMTAP-PUBSUB: Feed Subscriptions & Push Hints (extension)

DMTAP-PUB (§22) gives an identity an append-only, signed **author feed** any node may serve, and
any reader may pull, without trusting the server. What it does not give is a **protocol object** for
"I follow this feed" — following is a purely client-side act (§23.7's *workshop*), and a system that
wants to notify machines, not humans scrolling a client, has nowhere to plug in. This appendix
specifies **DMTAP-PUBSUB**, an additive extension of §22 that closes that gap with four things: a
signed, revocable **`Subscription`** object; a **topic** dimension so one identity can run several
independent feeds; **push delivery** of new entries as ordinary MOTEs, riding the existing
deliver/ack/retry machinery (§2.6) instead of inventing new reliability plumbing; and an explicit
application of §9.9's fan-out governance to the resulting push traffic.

DMTAP-PUBSUB is **opt-in, additive, and capability-negotiated (§10.2)**, exactly as DMTAP-PUB was
(§22). It changes **no** existing wire object — not `PubAnnounce`, not `FeedEntry`, not `FeedHead`,
not `Envelope`, not `Payload` — bumps no `Envelope.v` and no DNS `v=` anchor, and introduces no flag
day. Everything below rides machinery that already exists: a message kind in the reserved range
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
2. **Topic addressing, at zero wire cost.** One identity, one feed was §22.4's structural
   assumption (`FeedHead.pub` *is* the feed). §25.3 adds a topic dimension entirely at the
   **serving/locator layer** — no field is added to `FeedHead`, `FeedEntry`, or `PubAnnounce` — so an
   identity may run several independent, comparably-scoped feeds (a release feed, a chatter feed, a
   security-advisory feed) without any change to the objects a §22-only peer already knows how to
   verify.
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
- **Not a guarantee that a hint arrives quickly, or at all.** A hint is an optimization over
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
qualification, it means these §22 objects unchanged — consult §22 for their exact wire grammar. This
appendix introduces **three** new object types (`Subscription`, §25.4; `SubscriptionRevoke`, §25.5;
`FeedHint`, §25.6.2) and **zero** changes to any existing one.

## 25.3 Topic addressing

### 25.3.1 A locator dimension, not a wire field

§22.4 assumed one identity, one feed: `FeedHead.pub` names *the* feed, and `feed_head(pub)` (§22.4.4)
is a total function of `pub` alone. Adding a `topic` field to `FeedHead`/`FeedEntry`/`PubAnnounce`
would violate the no-core-wire-changes constraint this appendix holds itself to, and — worse — would
give every *existing* deployed §22 verifier an unknown key to choke on the moment a
DMTAP-PUBSUB-unaware peer tried to decode a topic-bearing signed object (§18.1.2's fail-closed
unknown-key rule on signed objects exists precisely to make that impossible to do safely without
capability negotiation, §10.2).

DMTAP-PUBSUB instead makes topic a **serving/locator-layer** partition: an identity that wants
several independent streams simply maintains **several independent `FeedEntry`/`FeedHead` chains**
under the **same** `pub`, each append-only and internally identical in shape to a §22 feed, and
distinguishes them purely by **which locator serves which chain** — never by any byte inside the
objects themselves. Nothing in a fetched `FeedHead` or `FeedEntry` reveals which topic it belongs to;
that fact lives only in the request that reached it. This is exactly the same move §22 itself made
for public vs. sealed manifests one layer down — bind the distinction into *how the object is
addressed*, never into a flag a peer could misread (§22.2.3) — applied here to *which feed* rather
than *which manifest type*.

A publisher that runs multiple topics is, mechanically, running multiple independent instances of
§22.4's bookkeeping — separate `seq` counters, separate `prev` chains, separate signed heads — under
one identity key, exactly as one person may keep several separate notebooks. `signer` MAY be the
same operational key across every topic (there is no requirement to mint a per-topic delegate); a
publisher choosing to publish two entries to two different topics is simply choosing which chain to
append each `FeedEntry` to, an ordinary local decision with no wire signal.

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

`{topic}` is the percent-encoded (RFC 3986) UTF-8 topic label; the two-segment path (no `topic`
segment) and the three-segment path with `{topic}` equal to the empty string percent-decode to the
**same** feed by definition (§25.3.3) — a server MAY implement either or both, and a topic-unaware
`pub-1` server that only ever serves the original two-segment path remains fully §22-conformant and
needs no code change to keep doing so. The mesh binding (§22.5.2) widens analogously: a holder
advertising `pub-1` MAY additionally serve topic-scoped feeds, discovered by whatever out-of-band
means a topic label is shared (a `pub_announce`'s `meta` map, §22.3.1, is a natural carrier for "here
is my topic list," but this appendix does not standardize one — that is left to a profile, exactly
as §23/§24 layer profile-specific `meta` schemas over §22 without this document's involvement).

**No new capability is required to read a topic-scoped feed.** Fetching
`/feed/{pub}/topic/{topic}/head` is an ordinary anonymous, content-addressed §22 pull — it needs only
`pub-1` (§22.6.1), never `pubsub-1` (§25.8). Topics are a `pub-1`-level convenience; `Subscription` /
`SubscriptionRevoke` / `FeedHint` are the `pubsub-1`-level capability this appendix is really about.

### 25.3.3 Backward compatibility (normative)

> A publisher that already operates a §22 feed and later adopts topics **MUST** continue serving its
> pre-existing `FeedEntry`/`FeedHead` chain, byte-for-byte unchanged, as the `topic = ""` feed. A
> reader that calls `feed_head(pub)` exactly as it did before this appendix existed **MUST** observe
> no discontinuity — the same chain, the same `seq` numbering, the same anti-rollback watermark
> (§22.4.2). Topic adoption is additive per publisher, never a migration.

This is the same discipline as every other extension in this document family: DMTAP-PUB changed no
sealed-path default (§22), the CAD/Video profiles changed no §22 byte (§23, §24), and topic
addressing changes no `FeedHead` byte and orphans no existing subscriber of the default feed.

## 25.4 The `Subscription` object

A `Subscription` is a **signed, self-verifying, bounded-lifetime capability**: a subscriber's
request to receive push hints (§25.6) for one `(pub, topic)` pair. It is the missing protocol object
identified in §25.1.1 — today, "following a feed" leaves no artifact a publisher can point to, audit,
or expire; a `Subscription` is exactly that artifact, modeled on the same self-contained-object
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
| `topic` | 5 | `tstr` | MUST (MAY be empty) | The topic label (§25.3.1). `""` names the default/untopiced feed — the one a pre-DMTAP-PUBSUB §22 deployment already serves (§25.3.3). |
| `issued` | 6 | `ts` | MUST | Creation time (ms epoch, §16.1). |
| `expires` | 7 | `ts` | MUST | Absolute expiry (ms epoch). **There is no indefinite `Subscription`** (§25.4.2) — a `Subscription` with this field absent is malformed and MUST be rejected on decode, not merely treated as non-expiring. |
| `nonce` | 8 | `bytes` | MUST, ≥ 16 B | Source of uniqueness for `subscription_id` (§25.4.1 below), so two `Subscription`s issued by the same subscriber for the same `(feed, topic)` in the same millisecond still content-address distinctly. |
| `signer` | 9 | `ik-pub` | MUST | The operational (device) key that produced `sig`; MUST be authorized by `subscriber` via a `DeviceCert` (§1.2) the verifier checks exactly as §22.3.3 step 4 checks a `PubAnnounce`'s `signer` against its `pub`. `signer` MAY equal `subscriber`. |
| `sig` | 10 | `sig-val` | MUST | Signature by `signer` over `DMTAP-PUB-v0/subscription ‖ 0x00 ‖ det_cbor(Subscription ∖ {10})` (§18.1.6 general rule). Failure is `ERR_PUB_SUBSCRIPTION_SIG_INVALID` (`0x090E`). |

**Content address.** Following the `Identity`/`PubAnnounce` derived-anchor rule (§18.9.4, §22.3.1) —
a field cannot contain its own hash — a `Subscription`'s content address is computed over the
**complete, signed** object:

```
subscription_id = 0x1e ‖ BLAKE3-256( det_cbor(Subscription) )
```

`subscription_id` is what a `SubscriptionRevoke` (§25.5) names, and what any holder recomputes and
checks before honoring the object (`ERR_PUB_SUBSCRIPTION_SIG_INVALID`-adjacent malformation is
caught at decode; a subscription presented under a mismatched address is simply not the object it
claims to be and is rejected the same way a misaddressed `PubAnnounce` is, §22.3.1).

**Why self-signed, when it also rides inside a signed MOTE (§25.4.4).** A `Subscription` is
independently verifiable **without** the `Envelope`/`Payload` that first carried it — exactly the
property that lets a publisher's serving holders (§22.4.3's "any node MAY serve any feed," extended
here to "any holder the publisher delegates to may honor a subscriber list") exchange subscriber
records as portable, self-contained artifacts, and lets a subscriber later *prove* to a third party
"I did subscribe, here is the signed object, here is when it expires" without needing to also
reconstruct the original transport envelope. This mirrors exactly why `PushSubscription` (§18.5.5)
and `PubAnnounce` (§22.3) are self-signed rather than relying solely on an enclosing transport's
authentication.

### 25.4.2 Bounded lifetime is mandatory, not a default (normative)

> A `Subscription` **MUST** carry an `expires` value. A conformant publisher/holder **MUST NOT**
> treat a `Subscription` as active once the current time passes `expires`, and **MUST NOT** push a
> `FeedHint` (§25.6.2) under an expired `Subscription`. Presenting or continuing to honor an expired
> `Subscription` is `ERR_PUB_SUBSCRIPTION_EXPIRED` (`0x090F`).

This is the design's answer to "how does a subscriber list stay bounded, self-pruning edge state
rather than an unbounded durable commitment" (§25.6.1): every entry in a publisher's active-hint list
has a hard expiry baked into the very capability that put it there, so an inactive/abandoned
subscription self-extinguishes even if no revoke is ever sent — the same "TTL, not a promise" posture
the relay-mailbox already applies to buffered ciphertext (§14.3) and the mixnet applies to key
epochs (§4.4.4). Renewal is simply issuing a fresh `Subscription` before the old one lapses; there is
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

A publisher MAY delegate acceptance and hint-pushing to another holder of its feed (exactly as
serving itself is delegable, §22.4.3) by handing that holder the self-signed `Subscription` records
it has accepted — the holder can independently re-verify each one (§25.4.1) without trusting the
publisher's bookkeeping, and without ever needing the publisher's private key.

## 25.5 Revocation — the `SubscriptionRevoke` object

### 25.5.1 `SubscriptionRevoke`

```cddl
SubscriptionRevoke = {
  1 => hash,       ; subscription   content address of the Subscription being revoked (§25.4.1)
  2 => ts,         ; ts             revoke time (ms epoch)
  3 => ik-pub,     ; signer         MUST equal the target Subscription's `subscriber`, or an authorized device thereof
  4 => sig-val,    ; sig            signer over det_cbor(SubscriptionRevoke ∖ {4}), DS-tag DMTAP-PUB-v0/subscription-revoke
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `subscription` | 1 | `hash` | MUST | The `subscription_id` (§25.4.1) of the `Subscription` being revoked. |
| `ts` | 2 | `ts` | MUST | Revocation time. |
| `signer` | 3 | `ik-pub` | MUST | The key that produced `sig`; MUST equal the target `Subscription.subscriber` or be one of its currently-authorized devices (`DeviceCert` chain, §1.2). A revoke signed by anyone else is `ERR_PUB_SUBSCRIPTION_REVOKE_INVALID` (`0x0911`) — only the subscriber who granted a subscription may withdraw it, borrowing the same same-author discipline `supersedes` applies to announces (§22.3.4, §22.3.3 step 5). |
| `sig` | 4 | `sig-val` | MUST | Signature by `signer` over `DMTAP-PUB-v0/subscription-revoke ‖ 0x00 ‖ det_cbor(SubscriptionRevoke ∖ {4})`. Failure is also `ERR_PUB_SUBSCRIPTION_REVOKE_INVALID` (`0x0911`). |

Unlike `Subscription`, a `SubscriptionRevoke` needs no internal content-address derivation of its
own — nothing ever points *at* a revoke — but it is self-signed for the identical portability reason
(§25.4.1): a holder the publisher delegates to (§25.4.3) can honor a revoke it never saw travel
through the original MOTE transport, by verifying the object alone.

**No `v`/`suite` fields, and why that is not an omission (normative).** A `SubscriptionRevoke`
carries neither a version nor a suite discriminator, unlike every other §22/§25 signed object. It
does not need them: a revoke is **never evaluated on its own**. `subscription` (key 1) names a
`subscription_id`, and a verifier cannot check that field at all without already holding the target
`Subscription` — so the target's `v` and `suite` are necessarily in hand, and they govern the
`sig` check. Adding independent discriminators would create a second, separately-negotiable
algorithm choice for a signature that is only ever meaningful relative to an object that has
already fixed one, and a mismatch between the two would be a new failure mode with no useful
resolution.

The consequence is normative and easy to get wrong in the other direction: a verifier MUST NOT
attempt to evaluate a `SubscriptionRevoke` without its target. A revoke naming a `subscription_id`
the holder does not have is not "valid but unmatched" — it is **unevaluable**, and MUST NOT be
recorded as an accepted revocation on the strength of its signature alone (that signature proves
only that *someone* signed some bytes; whether that someone is the subscriber is precisely the
check that requires the target, §25.5.1 `signer`).

### 25.5.2 Effect (normative)

> Once a publisher or any delegated holder (§25.4.3) has accepted a valid `SubscriptionRevoke`
> naming a given `subscription_id`, it **MUST NOT** push any further `FeedHint` under that
> `Subscription`. A `Subscription` presented after its revoke has been accepted — to justify renewed
> hint service, or handed to a *different* holder that has not yet heard the revoke — is
> `ERR_PUB_SUBSCRIPTION_REVOKED` (`0x0910`).

**Honest limit, stated plainly rather than hidden (§25.11 item 3).** Revocation is a request the
*publisher* (or its delegated holders) must honor cooperatively — exactly the same posture §6.6 item
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
| `topic` | 2 | `tstr` | MUST (MAY be empty) | Which topic-scoped chain (§25.3) changed. |
| `seq` | 3 | `u64` | MUST | The publisher's own belief about the new tip `seq`. **Advisory only** (§25.6.2 below) — never a substitute for a verified `feed_head` fetch. |
| `tip` | 4 | `hash` | OPTIONAL | The publisher's own belief about the new `FeedHead.tip`. Advisory, same status as `seq`. |
| `announce` | 5 | `bytes` | OPTIONAL | The complete, deterministically-encoded `PubAnnounce` (§22.3.1) at the hinted position — an inlining optimization (§25.6.3), not a trust shortcut. |

**Normative: advisory status (load-bearing).**

> A `FeedHint`'s `seq` and `tip` fields **MUST NOT** be used to advance a subscriber's accepted-`seq`
> watermark (§22.4.2), and **MUST NOT** be treated as evidence that content has been delivered. A
> conformant subscriber that receives a `FeedHint` **MUST** perform (or schedule) an ordinary,
> independently-verified `feed_head`/`feed_range` fetch (§22.4.4) — or, if `announce` is present,
> independently verify it exactly as a pulled `PubAnnounce` (§22.3.3, §25.6.3) — before accepting any
> change in feed state. A hint is a *reason to check*, never itself a *fact checked*.

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
  ordinary MOTE size discipline (the bucket ladder, §4.4.1/§16.3) — a large announce (many `roots`,
  large `meta`) simply does not fit and MUST NOT be inlined; the publisher falls back to the
  seq/tip-only hint and the subscriber pulls the announce and any referenced manifest/chunks the
  ordinary way (§22.5).
- **It creates no new durable state.** Inlining is a per-hint, stateless choice the publisher's
  client makes at push time (does this announce's encoding fit the bucket the MOTE is already
  being padded to) — it requires no bookkeeping about what any given subscriber has previously
  received, unlike true content-delivery push (§25.6.1).

A subscriber MUST treat the absence of `announce` identically to its presence-but-oversized case:
fetch and verify via the ordinary §22.4.4 pull path. Inlining is purely a latency/round-trip
optimization within an already-bounded MOTE, never a second delivery guarantee alongside the pull
path.

### 25.6.4 Standing: how a hint avoids the cold-sender gate

The act of sending a `Subscription` establishes reciprocal standing, symmetrically to §25.4.3's
observation about the *subscribe* direction: a subscriber that has issued a `Subscription` to a
publisher has, by that act, pre-authorized `FeedHint` MOTEs (kind `0x41` only) from that publisher's
operational key for the `(pub, topic)` pair named, until `expires`. A subscriber's node SHOULD record
this locally and treat a matching, unexpired `FeedHint` the way it treats mail from an established
contact (§2.7 step 5) — no per-hint challenge is needed, because the subscriber already asked for
this, exactly the reasoning §22.6.3 used to exempt `pub_announce` from a challenge (there, because
announces are pulled; here, because the push was solicited).

A `FeedHint` arriving with **no** matching active `Subscription` record is an ordinary unsolicited
push and receives **exactly the disposition §2.7/§9.2 already define for a cold sender** — deferred
to the requests area (§2.7a), rate-limited, never surfaced as a normal notification, and not acked.
No new wire error is defined for this case; it is not a malformed object, merely one this recipient
did not ask for, and DMTAP already has a complete answer for that.

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
> **aggregate** number of active `Subscription`s it honors per feed/topic (and MAY additionally cap
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
| Capability Tokens (§21.22) | `pubsub-1` — Specification Required; node/operator opt-in to originating and/or honoring `Subscription`/`SubscriptionRevoke`/`FeedHint` (§10.2). A node **MUST** advertise `pub-1` (§22.6.1) to meaningfully advertise `pubsub-1` — DMTAP-PUBSUB extends feeds and has no meaning without them. Topic-scoped serving (§25.3) needs **only** `pub-1`, never `pubsub-1` (§25.3.2). |
| Error/Status Codes (§21.14) | Six new code points **within** the existing subsystem byte `0x09` DMTAP-PUB already owns (§21.24b): `0x090E`–`0x0913` (§25.12). This is a Specification-Required addition **within an existing subsystem**, not a new subsystem byte — this appendix extends §22's own extension rather than registering a fresh one (§21.14's lighter-weight allocation policy for `NN` within an existing `SS`). |
| Signature DS-tags (§18.9 convention) | `DMTAP-PUB-v0/subscription` (`Subscription.sig` preimage, §25.4.1), `DMTAP-PUB-v0/subscription-revoke` (`SubscriptionRevoke.sig` preimage, §25.5.1) — reserved, distinct from every `DMTAP-v0/…`, `DMTAP-PUB-v0/…`, and `DMTAP-SYNC-v0/…` DS-tag already registered (§21.24b, §21.24c). `FeedHint` needs no DS-tag of its own — it is ordinary `Payload` content authenticated by the existing `Payload.sig` preimage (§18.9.2), unchanged. |

A peer that has not advertised `pubsub-1` MUST treat kinds `0x41`–`0x43` under the ordinary
forward-compatibility rule already governing unassigned/unimplemented kinds (§21.16, §10.1): it MUST
NOT `ack` a kind it cannot validate, and MAY ignore it. No flag day, no required upgrade.

## 25.9 Client requirements

- **Bounded-lifetime disclosure (MUST — normative UX).** Before a client issues a `Subscription` on
  the user's behalf, it MUST make the bounded, best-effort nature of the relationship visible: that
  the underlying feed content is plaintext and public exactly as any §22 feed is (§22.9 items 1–2,
  unchanged by this appendix), and that a revoke (§25.5.2) is honored cooperatively — a
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

| Invariant | Clause | Trigger | Behavior / error on violation |
|-----------|--------|---------|-------------------------------|
| **`Subscription` unknown version/suite** | §25.4.1 | a `v`/`suite` this implementation does not support | reject, never guess; `ERR_PUB_UNSUPPORTED_VERSION` `0x0901`, FAIL_CLOSED_BLOCK (the same code §22.3.1/§22.4.1 use, scope extended to this appendix's objects). A `SubscriptionRevoke` has **no** `v`/`suite` of its own — it inherits both from the target `Subscription` it names, which a verifier necessarily holds (§25.5.1) — so this rule has no surface there. |
| **`Subscription` missing mandatory `expires`** | §25.4.2 | decode of a `Subscription` lacking key `7` | malformed, reject on decode — no indefinite subscription exists |
| **`Subscription` signature / DeviceCert chain** | §25.4.1 | `sig` fails under `signer`, or `signer` not authorized by `subscriber` | reject; `ERR_PUB_SUBSCRIPTION_SIG_INVALID` `0x090E`, FAIL_CLOSED_BLOCK |
| **Expired `Subscription` honored** | §25.4.2 | current time > `expires`, and the holder still treats it as active / pushes a hint under it | reject/stop; `ERR_PUB_SUBSCRIPTION_EXPIRED` `0x090F`, FAIL_CLOSED_BLOCK |
| **`SubscriptionRevoke` cross-subscriber** | §25.5.1 | `signer` ≠ the target `Subscription.subscriber` (or an authorized device thereof), or `sig` invalid | reject; `ERR_PUB_SUBSCRIPTION_REVOKE_INVALID` `0x0911`, FAIL_CLOSED_BLOCK — only the subscriber who granted a subscription may withdraw it |
| **Revoked `Subscription` honored** | §25.5.2 | a `Subscription` presented/acted on after a valid matching `SubscriptionRevoke` has been accepted | reject; `ERR_PUB_SUBSCRIPTION_REVOKED` `0x0910`, FAIL_CLOSED_BLOCK |
| **Unsolicited `FeedHint`** | §25.6.4 | a `feed_hint` MOTE with no matching active `Subscription` record at the recipient | not a wire fault — ordinary §2.7/§9.2 cold-sender disposition (defer to requests area, §2.7a); no new error code |
| **`FeedHint` treated as authoritative** | §25.6.2 | a client advances its accepted-`seq` watermark, or treats content as delivered, from `FeedHint.seq`/`tip`/`announce` without independent §22.4.2/§22.3.3 verification | non-conformant client; the hint is a reason to check, never a fact checked |
| **Inlined `announce` unverified** | §25.6.3 | an inlined `FeedHint.announce` treated as valid without recomputing `announce_id` / verifying `sig`/`signer` chain | non-conformant client; verify exactly as a pulled `PubAnnounce` (§22.3.3), or reuse `0x0904`/`0x0905` on failure |
| **Publisher subscriber-admission bound** | §25.7.1 | aggregate active-subscription count (or subscribe rate) past a holder's configured bound | `ERR_PUB_SUBSCRIBE_QUOTA` `0x0912`, DENY_POLICY |
| **Subscriber inbound hint-rate bound** | §25.7.2 | inbound `FeedHint` rate from one publisher/topic past the subscriber's own configured budget | `ERR_PUB_HINT_RATE_LIMITED` `0x0913`, DROP_SILENT — excess dropped, never surfaced |
| **Topic backward compatibility** | §25.3.3 | a publisher's pre-existing feed is altered, renumbered, or orphaned upon adopting topics | non-conformant; `topic = ""` MUST remain byte-for-byte the pre-existing chain |

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

## 25.12 Error registry (`ERR_PUB_*`, continued — `0x090E`–`0x0913`)

These codes extend the subsystem byte `0x09` DMTAP-PUB already owns (§21.24b) — this appendix
registers no new subsystem. Codes follow the §21 conventions and responder-action vocabulary
(§21.2); the table below is authoritative for this range, exactly as §22.10 is authoritative for
`0x0901`–`0x090D`.

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x090E` | `ERR_PUB_SUBSCRIPTION_SIG_INVALID` | `Subscription` verification (§25.4.1) | `sig` fails under `signer`, or `signer` is not authorized by `subscriber` (DeviceCert chain). | No | FAIL_CLOSED_BLOCK |
| `0x090F` | `ERR_PUB_SUBSCRIPTION_EXPIRED` | `Subscription` lifecycle check (§25.4.2) | A `Subscription` is presented, or still being honored, past its `expires`. | Yes (subscriber may reissue a fresh `Subscription`) | FAIL_CLOSED_BLOCK |
| `0x0910` | `ERR_PUB_SUBSCRIPTION_REVOKED` | `Subscription` lifecycle check (§25.5.2) | A `Subscription` matching an already-accepted `SubscriptionRevoke` is presented or still being acted on. | No | FAIL_CLOSED_BLOCK |
| `0x0911` | `ERR_PUB_SUBSCRIPTION_REVOKE_INVALID` | `SubscriptionRevoke` verification (§25.5.1) | `sig` fails under `signer`, or `signer` does not match the target `Subscription.subscriber` (or an authorized device thereof). | No | FAIL_CLOSED_BLOCK |
| `0x0912` | `ERR_PUB_SUBSCRIBE_QUOTA` | Subscription admission policy (§25.7.1) | A holder's aggregate subscriber-admission bound (count/rate per feed or topic) is exceeded. A policy deny, never a security/crypto gate. | Yes (after freeing / under a laxer policy) | DENY_POLICY |
| `0x0913` | `ERR_PUB_HINT_RATE_LIMITED` | Subscriber-side inbound rate policy (§25.7.2) | A subscriber's configured per-publisher (or per-topic) inbound `FeedHint` budget is exceeded; excess hints are dropped. | Yes (next budget window) | DROP_SILENT |
