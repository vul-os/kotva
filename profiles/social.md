# SOCIAL — public feeds & microblogging without a timeline operator

> **Status:** draft, normative once ratified. A thin KOTVA **profile**
> ([`DIRECTION § 1`](../DIRECTION.md)) over the public substrate. Like every profile it invents no
> cryptography and no wire capability: a post is an ordinary public author-feed object
> ([`substrate/FEEDS.md`](../substrate/FEEDS.md), §22), discovery and anti-Sybil are the REPUTATION
> primitive ([`primitives/REPUTATION.md`](../primitives/REPUTATION.md)) bound to an adopted engine,
> moderation is a market of `labeler` coordinators, and search is a swappable `indexer` — every one
> of them under [`coordinator/CONTRACT.md`](../coordinator/CONTRACT.md). Where this document and a
> normative-byte home (§22, [`REPUTATION`](../primitives/REPUTATION.md)) disagree, the byte home
> governs; this document owns only the social schema on top of `PubAnnounce.meta` and the profile
> rules that bind clients.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are BCP 14 (RFC 2119 / RFC 8174).

---

## 1. What this is

SOCIAL is microblogging and social feeds — posts, replies, reposts, likes, follows, profiles —
**with no timeline operator, no ranking authority, and no token.** A user's posts are a signed feed
the user alone controls; a user's *timeline* is composed **on the reader's own node** from the feeds
it follows; moderation is a set of labelers the reader opted into; discovery is an indexer the reader
hired and can fire. Nothing in the middle owns the timeline, ranks it, or can delete from it.

It occupies the **public quadrant** (§22.1): authenticity without confidentiality. The publisher's
identity is the point, not a secret — the inverse of sealed mail. This is deliberately the shape of
the two live decentralized-social lineages folded into one object model: a **signed public event
stream keyed to a sovereign identity** (the niche served by Nostr-style relays) and **offline-first
signed-log gossip** (the Secure Scuttlebutt property, §22 / [`OFFLINE § 1`](../substrate/OFFLINE.md)),
with **ATProto-style composable, opt-in moderation** layered on as a `labeler` market rather than a
platform trust-and-safety monopoly.

**Non-goals.** SOCIAL is *not* a private-messaging system (direct messages ride the sealed MOTE, §3);
it is *not* an algorithm — it specifies a reverse-chronological baseline and treats every ranking as
disclosed, swappable coordinator policy; and it does *not* attempt anonymous publishing — who posted
what, and when, is exactly the fact it makes verifiable (§22.1.2).

---

## 2. What it composes (primitives · coordinators · bindings)

SOCIAL adds a spine on top of substrate machinery it does not modify.

| # | Substrate / primitive | What SOCIAL uses it for |
|---|---|---|
| ① | **Identity** ([`IDENTITY.md`](../substrate/IDENTITY.md), §1) | The account **is** the keypair `IK`. No platform account; portable across every client, indexer and labeler. Names (DNS / key-name floor) are swappable display pointers, never the identity. |
| ② | **Feeds & Blobs** ([`FEEDS.md`](../substrate/FEEDS.md), §22) | The carrier for **every** public object: posts, follows, likes, reposts, profile cards, and labels are all `PubAnnounce`s in append-only signed feeds; media are `PubManifest` blobs. |
| ③ | **REPUTATION** ([`REPUTATION.md`](../primitives/REPUTATION.md)) | Anti-Sybil discovery weighting and reply/mention notification filtering: the public follow-and-interaction graph is trust-edge input an indexer computes over (EigenTrust), and a reader's web-of-trust is the cold-contact filter (SOC-9). No published score (REP-3). |
| ④ | **MOTE** ([`02-mote.md`](../02-mote.md), §2) | Only for what is genuinely private: DMs ride `chat` (`0x01`); the account's identity/key-rotation announcement is `identity` (`0x09`). SOCIAL allocates no new kind. |

**Coordinators** (each an instance of [`CONTRACT`](../coordinator/CONTRACT.md), all non-load-bearing):

| Kind | Job in SOCIAL | Visibility (§4) |
|---|---|---|
| **`labeler`** | Publishes moderation labels over public posts/accounts; **opt-in and subscribable** — the reader applies only labelers it trusts, client-side (SOC-3, CONTRACT §4). | n/a — labels public objects; sees only what is already public. |
| **`indexer`** | Search, trending, "who replied," follower graphs — **derived and never authoritative** (REP-4, §22.4.3). TEE-preferred so a global-view compute holds no advantage it can forge. | `attested` (TEE) preferred; sees reader **queries** — a metadata leak, declared (§7). |

**Bindings** ([`bindings/README.md`](../bindings/README.md), swapped as version bumps, never rebuilt):

- **REPUTATION compute** → OpenRank (EigenTrust, TEE-verified) over the follow/interaction graph.
- **Anti-Sybil anchor** → World ID / Human Passport, or staked existing value — the bot-farm floor,
  disclosed as a ceiling, never a token (§9).
- **Media durability** → Walrus (hot) / Arweave (permanent) via the §22 blob durability descriptor —
  a `PubManifest` root names bytes; how long they live is a bought property (§22.1.2).
- **Identity recovery** → account abstraction (ERC-4337 / passkeys / MPC), so losing a device is not
  losing the account (CONTRACT §2.2 portability).

---

## 3. Objects — social schemas over `PubAnnounce`

Every public social object is a `PubAnnounce` (kind `0x40`, §22.3) in the author's own feed. SOCIAL
carries its type in the profile-named `meta` key `"social"` (the facet pattern of §22.3.1 / §24.18),
opaque to a generic PUB reader, covered by `sig` like every other `meta` value. The shapes below are
a **generic sketch**; a conformant client freezes the exact integer-keyed encoding. All inherit §22's
map conventions, origination floor (§22.3.3 step 1a), and `supersedes` revision chains (§22.3.4).

```cddl
; carried under PubAnnounce.meta["social"] as embedded det-CBOR (§22.3.1)
Post    = { 1 => 0,  2 => tstr,          ; type=post, body (bounded; the natural-person public field)
            ? 3 => hash,                 ; reply_to   announce_id of the post replied to
            ? 4 => [+ hash],             ; embeds     PubManifest roots (images/video/quoted blobs)
            ? 5 => tstr,                 ; lang       BCP-47 tag
            ? 6 => [+ tstr] }            ; self_labels author-declared content warnings (SOC-6)
Repost  = { 1 => 1,  2 => hash,          ; type=repost, target announce_id (pure boost)
            ? 3 => tstr,                 ; quote      OPTIONAL added commentary → a quote-post
            ? 9 => bool }                ; retracted  OPTIONAL; true on a supersede that retracts this repost (SOC-5)
Like    = { 1 => 2,  2 => hash,          ; type=like, target announce_id (a public reaction)
            ? 9 => bool }                ; retracted  OPTIONAL; true on a supersede that retracts this like (SOC-5)
Follow  = { 1 => 3,  2 => ik-pub,        ; type=follow, subject = the followed account's IK
            ? 3 => tstr,                 ; list       OPTIONAL named list/circle (reader-defined)
            ? 9 => bool }                ; retracted  OPTIONAL; true on a supersede that retracts this follow (SOC-5)
Profile = { 1 => 4,  ? 2 => tstr,        ; type=profile (a superseding revision chain), display name
            ? 3 => hash, ? 4 => tstr }   ; avatar PubManifest root, bio
Label   = { 1 => 5,  2 => subject,       ; type=label (issued by a LABELER in its OWN feed)
            3 => tstr, ? 4 => ts }       ; label value (e.g. "spam","nsfw","!hide"), OPTIONAL expiry
subject = hash / ik-pub                  ; a post (announce_id) or an account (IK)
```

- **A post's author is not a field** — it is the feed key that signs the `PubAnnounce` (§22.3),
  displayed via ordinary pinning/KT, verified with none (§22.3.3).
- **`Follow`/`Like`/`Repost` are public by design.** Unfollow / unlike / delete-repost is a
  **supersede** carrying `retracted = true` (key 9, above) over the same subject/target — never a
  deletion (§22.3.4); the graph is public and irrevocable (SOC-5, §9).
- **`Label` lives in the *labeler's* feed**, subject-attached to a post or account. It is the
  ATTEST/Feedback-shaped input of composable moderation (§4).

There is **no shared-timeline object and no like-count object.** A count is a derived aggregate over
public feeds (REP-4, §22.4.3), rebuildable by anyone, authoritative for no one — the same reason
REPUTATION defines no score object (REP-3).

---

## 4. Normative profile rules

- **SOC-1 — The account is the key; there is no platform account.** A user's identity is its
  substrate `IK` (§1). A client MUST NOT require registration with any operator to post, follow, or
  be followed. Moving between clients, indexers, or labelers is a config change with zero data
  migration and zero identity change (CONTRACT §2.2); the feed *is* the account.
- **SOC-2 — The timeline is composed at the reader, and reverse-chronological is the baseline.** A
  following-graph feed MUST be constructible on the reader's own node by merging the `FeedHead`s of
  the accounts it follows (§22.4.4), with **strict reverse-chronological order as the REQUIRED
  default and offline fallback**. Feed `seq` (§22.4.1) is per-author and orders entries only
  **within** a single author's feed; it carries no cross-author meaning and MUST NOT be used to order
  a multi-author merge. The merge key across authors is `PubAnnounce.ts` (§22.3.1) — the only
  wall-clock field the merge has. Because `ts` is self-asserted and offline-unverifiable (§22.3.3),
  it is **forgeable**: an author who sets `ts` far in the future pins that post atop every follower's
  timeline indefinitely. A client MUST clamp any `ts` exceeding the reader's local receive time down
  to that receive time before using it to order the merge (the PUB analogue of SYNC's one-sided HLC
  future-skew bound), and MAY fall back to local receive-order where it cannot establish a reliable
  clock. This is a disclosed residual, not a solved one (§8). Any other ordering (relevance, trending,
  "for you") is a **disclosed, swappable indexer policy** the reader hired (SOC-8), never a protocol
  default and never carried by any object. There is no ranking token and no ranking authority
  ([`DIRECTION § 5`](../DIRECTION.md), REP-3).
- **SOC-3 — Moderation is opt-in labelers, applied at the reader.** A client MUST NOT apply a label
  from a `labeler` the user has not explicitly subscribed to. Labels are **advisory and
  reader-applied**: subscribing to multiple labelers is composable, and precedence among them is
  reader policy. There is **no protocol-level takedown** (§22.6.2) — a labeler publishes a judgement;
  it cannot remove, quarantine, or suppress anyone's object.
- **SOC-4 — A labeler classifies but MUST NOT gate.** A `labeler` is the one coordinator kind whose
  function *is* content classification, and it is permitted **only** because it is not in any
  delivery or feed path (CONTRACT §4's explicit moderation carve-out). It MUST satisfy the four
  clauses — accountable descriptor, swappable, self-hostable, visibility-declared — and the reader
  remains the authority: a client MUST let the user override, ignore, or replace any labeler (mirrors
  REP-4 / REP-7). A labeler that could drop or re-rank delivery would be a classifying relay, which
  §4 forbids.
- **SOC-5 — Social graph and reactions are public, irrevocable, supersede-only.** `Follow`, `Like`,
  `Repost`, `Post`, and `Profile` are public author-feed objects; retraction is a superseding entry
  (§22.3.4), never an erasure. For `Follow`/`Like`/`Repost` the superseding entry carries the same
  subject/target with `retracted = true` (§3); a client MUST treat the latest entry in a `supersedes`
  chain as authoritative and MUST render a `retracted = true` entry as absence (unfollowed / unliked /
  un-reposted). Retraction binds only a reader who checks the author's feed head — nothing obliges an
  independent holder to notice (REPUTATION §2.4). A client MUST disclose irrevocability **before**
  the act (§22.7).
- **SOC-6 — No personal data beyond what the author publishes.** Published objects are irrevocable
  and content-addressed; a right to erasure cannot be satisfied against them (TRACT §0.5.1). A
  `Post.body` and `Profile` fields are the natural-person public surface; a client SHOULD warn on a
  body that appears to carry a third party's personal data, and MUST NOT auto-publish anything the
  user did not explicitly post (§22.2.4 publish-act boundary). An author MAY self-declare content
  warnings via `Post.self_labels` (§3); a client MUST render a post carrying `self_labels` behind the
  declared warning by default, and MUST NOT treat the author's own `self_labels` as `labeler` output
  (SOC-3) — it is a first-person disclosure, not third-party moderation. Private conversation is a
  sealed MOTE (`0x01`), never a public object.
- **SOC-7 — Discovery is derived and non-authoritative.** Any node MAY build a search index, trending
  list, or follower graph over the public feeds; the result is rebuildable and MUST NOT be treated as
  authoritative — the authoritative state is always the set of signed feeds (REP-4, §22.4.3). Two
  indexers disagreeing on "trending" is the design, not a defect.
- **SOC-8 — Ranking and label policy are disclosed and substitutable.** An `indexer` that ranks, and
  a `labeler` that labels, MUST publish its policy in its signed descriptor (CONTRACT §2.1), and a
  reader MUST be able to substitute its own local computation or another provider (REP-7). Neither
  publishes a global score (REP-3).
- **SOC-9 — Reply/mention reach is web-of-trust, never content classification.** A public reply or
  mention cannot be *blocked* — it is a `Post`, an ordinary fee-free, challenge-less `PubAnnounce`
  (§22.3.2, §22.6.3); a §2.2b cold-contact proof is a per-recipient construct that does not apply to
  a pulled public object and is NOT required to publish or fetch one, cold or not. Whether a reply or
  mention *surfaces in a recipient's notifications* is entirely the recipient's own local computation:
  a client MAY scan followed/indexed feeds for a `reply_to`/mention match and rank or suppress the
  result by the recipient's local web-of-trust or a subscribed labeler's advisory label — never by
  gating the underlying `Post`. SOCIAL defines no push-delivered mention/notification object and
  allocates no new MOTE kind (§2); a client wanting live-push notice of a mention runs its own
  polling or indexer subscription, outside this protocol's scope. The filter authorizes on identity
  and rate; it MUST NOT be a coordinator classifying content in a delivery path (CONTRACT §4).

---

## 5. Scale-invariance — village mesh to planet

The objects and rules are identical at every scale; only the **trust anchor** slides
([`DIRECTION § 6`](../DIRECTION.md)).

| Function | Small / mesh (no coordinator) | Global (hired coordinator) |
|---|---|---|
| **Timeline** | reverse-chron merge of the feeds you gossip-replicate (SSB) | same merge, plus an indexer's ranking you opted into |
| **Discovery** | follow-the-follows: web-of-trust over `Follow` edges you hold | an `indexer` running OpenRank/TEE over the global graph |
| **Moderation** | your own blocks + labels from people you know | subscribed `labeler` services, composable |
| **Anti-Sybil** | you know these accounts | a personhood attester you chose (§9) |

The mesh form is the **fallback the design collapses to**, not a lesser mode: unplug the internet and
SOCIAL is exactly the set of followed feeds you can reach, ordered by time, moderated by your own
rules. Add connectivity and an indexer and a labeler become *available* for reach and shared judgement
— never *required* to read the people you already follow.

---

## 6. Offline / apocalypse behaviour

SOCIAL inherits the substrate offline profile ([`OFFLINE.md`](../substrate/OFFLINE.md)); grades:

- **Authoring is `full`.** Composing and signing a `Post`, `Follow`, `Like`, `Repost`, or `Profile`
  and appending it to your own feed needs no network (OFFLINE §3.3). **Distribution** to followers /
  the swarm is `deferred` — a signed intent that gossips out on reconnect, never dropped, never shown
  as delivered before it is (R-GRADE-2).
- **Reading the following-graph timeline is `full`.** Any cached feed entry or media blob verifies
  offline, self-authenticating (§22.5); the reverse-chron merge is a local computation over what you
  hold. This is precisely the Secure Scuttlebutt / Briar case: signed logs replicated by gossip or
  sneakernet, verified identically over mesh, HTTPS, or SD card ([`OFFLINE § 1`](../substrate/OFFLINE.md)).
- **Global discovery and trending are `local-trust`.** An indexer's ranking is a derived, rebuildable
  view whose staleness offline is never a correctness failure (REP-4, OFFLINE §3.3) — you fall back to
  reverse-chron and your own web-of-trust.
- **A *fresh* personhood proof is `blocked`.** Bot-resistance that needs a live attester fails closed
  offline; a personhood/stake ATTEST already held is `full` (OFFLINE §3.1). A client MUST NOT gate an
  offline timeline on a fresh proof it cannot obtain.
- **Labeler subscription updates are `deferred`.** Already-fetched labels apply `full`; new labels
  arrive on reconnect. Moderation therefore degrades to your own rules plus your last-synced labels —
  disclosed, never silently dropped (R-GRADE-1).
- **Reconcile is feed catch-up.** On reconnect a reader fetches entries newer than its last-seen `seq`
  and walks the `prev` chain (§22.4.2), idempotently (content-address dedup, R-REC-1). A **forked
  author feed** — two histories under one key — is a **detectable equivocation surfaced for the reader,
  never swallowed by a merge** (§22.4.2 `HALT_ALERT`, OFFLINE R-REC-2): a poster cannot honestly
  present two histories, and a reader who sees both holds transferable evidence.

SOCIAL carries no money leg, so the offline-money trilemma (OFFLINE §5) does not arise; the sole
`blocked` resources are the fresh personhood proof and live discovery, both marked, never faked.

---

## 7. Security & declared content-visibility

SOCIAL inherits [`THREAT-MODEL.md`](../THREAT-MODEL.md) unchanged; the invariants that bite:

- **SEC-2 (intrinsic authenticity).** Every social object is signed and content-addressed; it verifies
  identically over any transport, trusting no server. A malicious indexer or serving node can
  **withhold or stall** (detectable via `seq`/chain discontinuity, §22.4.3) but can **never forge** a
  post (that needs the author's key) nor **hide** a published one without a reader noticing a gap.
- **SEC-4 (declared content-visibility).** Every intermediary declares what it sees:

| Intermediary | Class (CONTRACT §5) | What it sees |
|---|---|---|
| PUB serving node / CDN | `n/a` (public) | the object (public by design) **and which reader fetched it** — no read privacy (§22.9 item 7) |
| `indexer` | `terminating` (query channel; `attested` preferred) | the public graph **and the reader's queries** — a metadata leak, declared; TEE (`attested`) preferred so it cannot forge results |
| `labeler` | `n/a` (public) | only already-public content; it publishes labels, gates nothing |
| DM carrier (sealed MOTE `0x01`) | **`blind`** / structural | ciphertext only — a DM is E2E-encrypted; the carrier reads nothing (SEC-3) |

- **SEC-6 (authorize, never classify).** The `indexer` ranks and the `labeler` labels only as
  disclosed, swappable, self-hostable policy the reader hired (SOC-4, SOC-8); neither is load-bearing,
  and neither gates delivery on content. Ranking that "finishes" and centralizes is what REP-4 forbids.
- **SEC-7 (abuse priced and localized; anti-Sybil not solved).** A client MUST NOT describe bot /
  trending manipulation as solved. The personhood/stake anchor raises the floor; a reader's local
  web-of-trust notification filter (SOC-9) dissolves it further at local scale — but a funded,
  patient sockpuppet operator defeats both (§9, REPUTATION SEC-7).
- **SEC-9 (irrevocability / metadata).** A client MUST disclose, before publishing, that a post/follow
  is public and irrevocable and that retraction is cooperative-only (SOC-5, §22.7). Reads leak *which
  reader fetched which public object*; a reader who must hide that they read supplies their own
  transport anonymity (Tor-class), outside this profile's scope.

---

## 8. Honest residual

Each is disclosed rather than solved; every one traces to a root ceiling
([`DIRECTION § 8`](../DIRECTION.md)).

- **Timeline ordering is forgeable via self-asserted `ts`.** The SOC-2 future-skew clamp bounds
  but does not eliminate a moderately-skewed forgery sorting ahead of genuinely recent posts.
  Disclosed, not solved.
- **The algorithm is genuinely better UX, and we ship reverse-chron by default.** A tuned engagement
  feed reads better than a chronological one; refusing a surveillance ranking authority costs real
  usability. The fix — a TEE `indexer` you hired — converges toward centralized-grade discovery
  (§4 future-proofing) but trades operator-trust for chip-vendor-trust (`attested`, never trustless).
- **Discovery re-centralizes even though the protocol permits many indexers.** "Any node MAY build an
  index" does not mean many will; whichever indexer becomes economically dominant becomes a de-facto
  content-policy gatekeeper regardless of what this document permits (TRACT §0.4.1 / §21.3). This is
  the **weakest load-bearing claim** here, marked as such rather than defended; competing indexers with
  verifiable completeness proofs are a candidate with no deployed precedent.
- **Moderation is unbundled, not solved.** Composable labelers give the reader sovereignty over what
  they see; they give **no answer to "what is true."** "Which account is a bot, which post is
  disinformation, which label is correct" is the **editorial-governance ceiling**
  ([`DIRECTION § 8.4`](../DIRECTION.md)) — the design offers a *market of judgements*, not a canonical
  one, and different labelers will disagree by construction.
- **Anti-Sybil is imperfect** (root ceiling 1), shared identically with REPUTATION: personhood raises
  the bot-farm floor and every method trades off; it does not close the gap.
- **The social graph is public** — who-follows-whom is exactly what `Follow` edges make verifiable, and
  that is a metadata surface. A *private* follow graph is out of scope (it would forfeit the derived
  discovery and web-of-trust that make the design work).
- **An offline or rarely-online account is invisible, not merely slow.** A feed is available only as
  long as some holder serves it (availability ≠ durability, §22.9 item 4); a poster whose node departs
  disappears from discovery unless a third party caches it — and whether caching needs an incentive,
  and whether that incentive breeds another operator, is open (TRACT §0.7, §21.8).

Trust/discovery research returned nothing verified across passes (REPUTATION §9); the reasoning here
is checked for internal consistency against the anchor docs, not offered as a demonstrated result.
Maturity claims for OpenRank, World ID, Human Passport, Walrus and the ATProto labeler model are a
2026-07 snapshot ([`bindings/README.md`](../bindings/README.md)) and MUST be re-checked before
production reliance.
