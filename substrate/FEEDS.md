# Substrate Capability ② — Feeds & Blobs

> **Status:** additive profile of the core specification. This document **extracts §22 (DMTAP-PUB:
> Public Objects)** as a standalone capability usable **without the mail spec**. It restates no
> normative bytes: `PubManifest`, `PubAnnounce`, `FeedEntry`, `FeedHead`, their CDDL, signing preimages,
> content-address derivations, anti-rollback and fail-closed rules are all defined in **§22 (and §18 for
> the wire layout), which govern.** This document selects that machinery out of the mail spec, states
> what a non-mail product needs, and records the **kerf-pub** reference implementation as the existence
> proof that §22 runs over plain HTTPS with no mesh.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174.

---

## 1. The one idea

Sealed DMTAP (§2, §5, §6) fills one quadrant of the confidentiality × authenticity space: *content
sealed to members, authored by a sovereign identity.* The **Feeds & Blobs** capability fills the
inverse quadrant — **authenticity *without* confidentiality**: an object **signed by a sovereign
identity (§1), readable by anyone, globally deduplicated.** Here the publisher's identity is the
*point*, not a secret.

This is exactly the substrate a product needs when it wants to **publish signed, verifiable, public
data** — a software release feed, a mailing-list archive, an open-hardware part library, a CAD artifact
catalog (§23), a public changelog — without any of the sealed-messaging machinery. It subsumes two
niches that today require separate ecosystems, using **one** object model:

- a **signed public event stream** keyed to a self-sovereign identity — the niche served by Nostr-style
  relays — realized as an **author feed** (§4 below); and
- **public content-addressed storage** with global cross-user dedup and swarmed retrieval — the niche
  served by IPFS-style pinning — realized as the **public blob profile** (§3 below).

Everything is **opt-in, additive, and capability-negotiated** (`pub-1`, §10.2, §22.6.1). A product that
does not implement it never advertises `pub-1`, never serves a public object, and never receives one it
must reject.

---

## 2. Why this is a clean substrate capability

Three properties (all §22.1.1) make Feeds & Blobs adoptable with zero dependency on mail:

1. **Authenticity without confidentiality.** A `PubAnnounce` (§4.2) is signed **in the clear** by the
   publisher's operational key, chaining to its root `IK` (§1.2). Anyone can fetch it and verify —
   **offline, zero DNS, zero name-chain** — that identity `IK` published exactly these bytes at that
   time. No sealed sender, no per-recipient encryption, no recipient at all.
2. **Self-verifying, trustless serving.** Every object — announce, manifest, chunk, feed entry — is
   authenticated by a signature or a content address it carries, so **any node MAY serve any object
   without being trusted** (§22.4.3, §22.5.1). This is what makes the HTTP test (below) pass: the server
   is a convenience, never a trust root.
3. **Global, cross-user dedup.** A public blob is addressed over its **plaintext** chunks (§3.2), so two
   publishers of the same bytes compute the **same** content address and the swarm stores them once —
   the deliberate inverse of the sealed file model (§5.5), which ciphertext-addresses precisely to
   *forbid* cross-user dedup for privacy. Here there is no privacy to protect and dedup is the purpose.

None of the three references the mail path. A product adopts Feeds & Blobs by adopting §22's objects and
the `pub-1` capability; it needs [`IDENTITY.md`](IDENTITY.md) (whose key signs the feed) and nothing
from §2/§5/§7/§8.

---

## 3. Public blobs (profile of §22.2)

A public blob is a **plaintext-addressed Merkle-DAG manifest** — the structural twin of the sealed
`Manifest` (§5.5), with three deliberate differences and everything else inherited.

### 3.1 `PubManifest`

Integer-keyed CBOR (§18.1.2); normative in §22.2.1:

```cddl
PubManifest = {
  1 => hash,       ; id        DS-tagged Merkle root over PLAINTEXT chunk hashes
  2 => u64,        ; size      total plaintext size
  3 => u32,        ; chunk_sz  fixed chunk size (e.g. 1 MiB)
  4 => [+ hash],   ; chunks    ordered PLAINTEXT chunk hashes h_i
  ; key 5 (per-file key) is FORBIDDEN by construction — a public blob has no key
  6 => suite,      ; suite     hash suite (v0 BLAKE3-256); NO AEAD selector
}
```

- **Key 5 is the key-5 trap.** A `PubManifest` MUST NOT carry a key field; a decoder that finds key `5`
  MUST reject (`ERR_PUB_MANIFEST_KEY_PRESENT`, `0x0902`) — either a leaked sealed manifest or a
  malformation (§22.2.1). This mirrors §5.5's `ERR_MANIFEST_KEY_PRESENT` (`0x0808`): the sealed manifest
  forbids key 5 *lest it leak*; the public manifest forbids it *because none exists*.
- Chunking, streaming, resumability, and parallel/swarmed fetch are **inherited unchanged from §5.5**;
  only the hash input (plaintext, not ciphertext) and the tree's domain separation differ.

### 3.2 Plaintext addressing (normative allocation, §22.2.2)

```
h_i = 0x1e ‖ BLAKE3-256( plaintext_chunk_i )                 ; 0x1e = BLAKE3-256 multihash prefix
DS  = "DMTAP-PUB-v0/manifest" ‖ 0x00
leaf(h_i)         = BLAKE3-256( DS ‖ 0x00 ‖ h_i )            ; RFC 6962 tree, DS-tagged
node(left,right)  = BLAKE3-256( DS ‖ 0x01 ‖ left ‖ right )
PubManifest.id    = 0x1e ‖ MTH(h_0 … h_{n-1})               ; split at largest power of 2 < n
```

The multihash prefix (§18.1.5) preserves hash-agility and is the **interoperability seam with external
content-addressed stores**: a Git-LFS / sha256 pointer maps onto `0x12 ‖ SHA2-256(plaintext)`, and a
CIDv1 (raw codec) can be derived from a chunk hash for an IPFS fetch-adapter. This is not hypothetical —
the kerf-pub reference (below) ships **SHA2-256 under prefix `0x12`** as its interop digest and derives
CIDv1 for its optional IPFS adapter, changing the digest with "a one-line change" and no flag day.

### 3.3 Type-incompatibility with sealed manifests (fail closed, §22.2.3)

The DS-tag is folded into every leaf and node, so a **public root and a sealed root over the same
chunk-hash list are different values** — the type is bound into the address, not asserted by a boolean.
A public manifest supplied where a sealed one is required (or vice versa) MUST be rejected
(`ERR_PUB_MANIFEST_TYPE_MISMATCH`, `0x0903`, FAIL_CLOSED_BLOCK) — never "tried both ways."

### 3.4 The publish-act boundary (normative, §22.2.4)

Plaintext addressing makes a **CAS-confirmation** possible — anyone holding a candidate plaintext can
confirm a publisher holds the same bytes. This is **accepted by design** for *published* content (the
publisher broadcast it on purpose), with one hard boundary that a Feeds-adopting product MUST enforce:

> **A `PubManifest` MUST NOT be derived from content the user has not explicitly published.** An
> implementation MUST NOT plaintext-address, announce, or serve any object except as the result of an
> **explicit publish act** (§22.7). Applying plaintext addressing to a user's private files would
> reintroduce exactly the leak §5.5 exists to prevent. The publish act is the sole gate from private to
> public, and it is irrevocable (§22.7, §22.9).

---

## 4. Author feeds (profile of §22.3–§22.4)

### 4.1 The two objects that give a feed order and anti-rollback

An **author feed** is a **per-identity, append-only, signed log** of one publisher's announcements — the
DMTAP-PUB analogue of a KT log (§3.5) or the cluster journal (§5.6.3(b)), scoped to one publisher. It
gives the ordering, discovery, and anti-rollback a bare announce cannot. Normative in §22.4.1:

```cddl
FeedEntry = {
  1 => u64,     ; seq       strictly increasing per feed, genesis = 0
  2 => hash,    ; announce  content address of the PubAnnounce at this position
  ? 3 => hash,  ; prev      content address of the FeedEntry at seq-1; ABSENT iff seq = 0
  4 => ts,      ; ts        entry time
}
FeedHead = {
  1 => u8,      ; v = 0
  2 => suite,   3 => ik-pub,  ; pub — the feed's author IK (feed is SINGLE-AUTHOR by construction)
  4 => u64,     ; seq — the tip
  5 => hash,    ; tip — content address of the FeedEntry at seq
  6 => ts,      7 => ik-pub,  ; signer — DeviceCert chains to pub (§1.2)
  8 => sig-val, ; signer over det_cbor(FeedHead ∖ {8}), DS-tag "DMTAP-PUB-v0/feed"
}
```

**FeedEntries are not individually signed.** Because each entry chains `prev` to its predecessor, the
signed head's `tip` **transitively commits to the entire log** (RFC 6962 discipline) — signing the head
authenticates every reachable entry, exactly as with cluster-journal entries (§5.6.3(b)) and KT leaves
(§3.5). This is the load-bearing design: **sign the tip, hash-chain the rest.** The `FeedEntry` content
address is `entry_id = 0x1e ‖ BLAKE3-256(det_cbor(FeedEntry))`, **no DS-tag fold** (an unsigned entry's
authenticity flows solely from the signed head's `tip`; §22.4.1).

### 4.2 The `PubAnnounce` (kind `0x40`, §22.3)

A `PubAnnounce` is a **bare, unsealed, signed CBOR object** — not a MOTE. It carries the publisher's
identity **in the clear** (`pub` = the root `IK`; `signer` = the operational key, chained to `pub` by a
`DeviceCert`) and is fetched by content address. Its `announce_id = 0x1e ‖ BLAKE3-256(det_cbor(PubAnnounce))`
(the fully-signed object). Verification (§22.3.3) is **offline, zero-DNS**: reject unknown `v`/`suite`;
recompute and check `announce_id`; verify `sig` under `signer`; verify `signer` is authorized by `pub`
(`signer == pub`, or a valid non-revoked `DeviceCert` chain); if `supersedes` is present, require the
predecessor's `pub` to match (a publisher may only supersede its own announcements). A name is needed
only to *display* who `pub` is — verification never needs a name lookup (§3.13, and see
[`IDENTITY.md § 6`](IDENTITY.md#the-naming-ladder-is-not-inverted)).

### 4.3 Anti-rollback and equivocation (§22.4.2)

- A reader **retains the highest `seq` accepted per author feed** (keyed by `pub`) and MUST **reject any
  `FeedHead` whose `seq` is strictly less than** the last accepted (`ERR_PUB_FEED_ROLLBACK`, `0x0907`).
  Feeds only grow; a stale head cannot *suppress* announcements. This uses strict `<` (not the `≤` of
  push-delivered counters) because a head is a pull-fetched cacheable object a reader legitimately
  re-fetches: **equal `seq` + identical `tip` is idempotent (accept); equal `seq` + different `tip` is
  equivocation** (`ERR_PUB_FEED_CHAIN_BROKEN`, `0x0908`, HALT_ALERT).
- The `prev` hash-chain makes a **fork detectable**: two distinct entries at one `seq`, or a `prev` that
  does not resolve to `seq-1`, is evidence the author's log was rewritten/equivocated (`0x0908`,
  HALT_ALERT — the same posture as a committer fork `0x0404` and a cluster-journal break `0x0412`). A
  publisher cannot honestly present two histories; a reader that sees both holds transferable evidence.

### 4.4 Retraction is supersede-only; irrevocability (§22.3.4, §22.7, §22.9)

There is **no deletion**. Each announce is immutable and content-addressed, so a revision is a **new**
object with a new `announce_id` naming the old via `supersedes`; the predecessor remains fetchable
forever. **Deprecation / yank is a successor announcement**, never an erasure. A Feeds-adopting client
**MUST** warn the user, before the act, that **publishing is irrevocable** — a swarmed content-addressed
object cannot be un-published (§22.7, normative UX). A client MAY stop serving *its own* copy but MUST
NOT imply that deletes the object for others.

---

## 5. Serving — the HTTP test, head-on (profile of §22.5)

§22.5 is deliberately abstract enough that **plain HTTPS with no mesh present is a complete
implementation** — the intended first deployment, and the substrate's [HTTP test](README.md#42-the-http-test--are-transports-pluggable-with-https-first-class).

### 5.1 The well-known gateway (§22.5.1)

A node advertising `pub-1` MAY expose a well-known HTTP surface; reads are **anonymous** and
**content-addressed**:

```
GET /.well-known/dmtap-pub/feed/{pub}/head              → FeedHead     (application/cbor)  [MUTABLE]
GET /.well-known/dmtap-pub/feed/{pub}/range?from=&to=   → [FeedEntry]  (application/cbor)
GET /.well-known/dmtap-pub/announce/{id}                → PubAnnounce  (application/cbor)  [IMMUTABLE]
GET /.well-known/dmtap-pub/manifest/{id}                → PubManifest  (application/cbor)  [IMMUTABLE]
GET /.well-known/dmtap-pub/chunk/{h}                     → raw plaintext chunk bytes        [IMMUTABLE]
```

The four content-addressed endpoints are **immutable**: a server SHOULD send
`Cache-Control: public, immutable, max-age=<long>` and a strong `ETag` **equal to the content address**,
and MAY be fronted by any ordinary HTTP cache/CDN — a cache never needs to understand DMTAP.
`feed/…/head` is mutable and carries a short TTL / `must-revalidate`. A missing object is a **404**
(`ERR_PUB_NOT_SERVED`, `0x090C` — the holder does not serve it; a fetcher rotates to another holder).

**Verification is the client's job, always.** A fetcher MUST verify signatures and content addresses on
every response (§22.3.3, §22.4); a server (or CDN, or attacker in its place) cannot forge an object it
did not receive `pub`'s key to sign, and cannot substitute chunk bytes under a matching `h_i` (BLAKE3
collision resistance, §5.5.3). **A server is a convenience, not a trust root.**

### 5.2 Mesh is an optional second transport (§22.5.2–§22.5.3)

The same five operations are served over the mesh (§4.5 bulk path, §5.5 swarm). Public objects are **not**
routed through the mixnet — there is no metadata to protect. Swarm behavior is inherited from §5.5.3
(chunks self-verify; a mismatch is `ERR_PUB_CHUNK_HASH_MISMATCH`, `0x090A`, rotate to another holder),
with the one difference being **global cross-user dedup** (§3.2). Mesh serving **adds** swarming and
NAT-traversal on top of HTTPS; it is never a prerequisite to speak the capability.

### 5.3 Optional — chunk-tree range proofs (additive proposal)

> **Status: additive, OPTIONAL.** This subsection proposes **one new, optional gateway endpoint**. It
> allocates no new object, no new signing preimage, and no §21 error code; it is advertised by presence
> (a 404 means "not offered here," a fetcher falls back to whole-manifest verification). It is drawn from
> the *vidmesh* protocol's blob sidecar (its `GET /blob/{id}/proof?chunk=i`), surfaced as waist machinery
> by the convergence work of the [Video/Media profile (§24)](../24-video-profile.md), because it fills a
> real gap for large media (seek + verify a *middle* chunk without downloading the whole chunk list).

Today a fetcher verifies a chunk by holding the whole `PubManifest` (§3.1): it has all `chunks` hashes,
so it can check any `chunk/{h}` byte-for-byte. That is fine for small blobs but wasteful for **large
media** — to verify one 1 MiB chunk in the middle of a multi-gigabyte video, a client must first fetch the
entire (possibly multi-thousand-entry) chunk-hash list. The DS-tagged Merkle tree over the chunk hashes
(§3.2) already supports an **O(log n) inclusion proof**; this endpoint exposes it:

```
GET /.well-known/dmtap-pub/manifest/{id}/proof?chunk=i   → [ i, [ sibling_hashes… ] ]  (application/cbor)  [IMMUTABLE]
```

- The response is a canonically-encoded CBOR array `[chunk_index, [sibling hashes on the path to the
  root]]` — exactly the RFC 6962 audit path for leaf `i` under the tree of §3.2. It is **immutable and
  content-addressed by `(id, i)`** (same `Cache-Control: public, immutable` + CDN-frontable posture as
  the other four content-addressed endpoints, §5.1).
- **Verification stays wholly client-side and trustless.** A fetcher recomputes `leaf(h_i)` from the
  chunk it fetched (`chunk/{h_i}`), folds the returned siblings with the **same `"DMTAP-PUB-v0/manifest"`
  DS-tagged `leaf`/`node` functions** (§3.2), and checks the result equals the `PubManifest.id` it already
  trusts from the signed announce. A lying server cannot forge a proof (BLAKE3 collision resistance) — the
  endpoint is a convenience, never a trust root, identical in spirit to every other §22 read.
- **Purely additive.** A gateway that does not implement it answers 404 and the client falls back to
  whole-manifest verification (§5.1); a client that does not need it never calls it. It changes nothing
  about how objects are signed, addressed, or stored — it only serves a proof the tree already commits to.
  It is the serving substrate under §24's segmented (HLS/DASH) playback: verify a byte-range segment
  against the signed rendition root in O(log n) without the full manifest.

### 5.4 Advisory fetch-hint registry (additive proposal)

> **Status: additive, advisory.** A small profile-local registry of **retrieval hints** — where a holder
> claims to serve a blob — so a fetcher can *find* holders faster. Hints are **advisory and never
> authoritative**: the content address proves integrity regardless of origin (§5.1), so a client MUST NOT
> treat a blob fetched from an unlisted source differently, and MUST ignore unrecognized hint types rather
> than reject. Also drawn from vidmesh (its kernel `Hint` type) via the §24 convergence.

```
Hint = [ hint_type: uint, value: tstr ]
```

| `hint_type` | Name | Value |
|------------:|------|-------|
| `1` | `https` | URL serving the blob with HTTP Range (a §22 `chunk` surface, or any range server) |
| `2` | `torrent-v2` | BitTorrent v2 infohash, lowercase hex |
| `3` | `relay-blob` | base URL of a relay/gateway blob surface (§5.1) |
| `4` | `bundle` | locator of a self-verifying bundle containing the blob |

A publisher MAY carry `hints` alongside an announce or manifest (as the CAD/Video profiles do, §23/§24);
a holder MAY advertise which addresses it serves. New transports are new hint types. Because a hint is
advisory, an implementation that ignores the whole mechanism loses nothing but discovery speed — the
verification gate (§5.1) is unchanged, and the address, not the hint, is the truth.

---

## 6. Reference implementation — kerf-pub proves the HTTP test

**kerf-pub** (`/Users/pc/code/exo/kerf/packages/kerf-pub`) is the reference implementation of §22 (with
the §23 CAD profile layered on top). It is named here only as an existence proof; per the repository's
implementation-neutral stance it is not part of the standard and not required to speak it, and where it
and the spec disagree, **the spec wins.** It proves §22 works over plain HTTPS with no mesh in three
interlocking ways (all observable in its test suite):

1. **The gateway is a plain HTTP server; the client speaks only `GET`.** The five
   `/.well-known/dmtap-pub/*` routes are served dynamically from a content-address-keyed store (in-memory
   or Postgres); the client's entire network layer is `GET` over `{base}/.well-known/dmtap-pub/…` — no
   websockets, no DHT, no peer protocol. Any CDN or `nginx` serving those routes qualifies as a holder.
2. **Total client-side verification makes the server untrusted.** Every fetch re-derives the content
   address and re-checks signatures against the bytes: `PubManifest.verify()` recomputes the DS-tagged
   Merkle root, every chunk is checked with `verify_chunk`, `resolve()` re-verifies the `FeedHead`
   signature, applies the per-author anti-rollback watermark, and walks the full `prev` chain; a lying
   server is rotated away from, never accepted. Trust lives entirely in the Ed25519 signature and the
   hash-chain — nothing for a relay to add, which is precisely why no mesh is needed.
3. **Zero-socket invariant + a five-endpoint round-trip test.** A client with no gateways configured
   never opens a socket (publish/resolve/fetch all work on the local store). The end-to-end test mounts
   **only** the gateway router on a bare app with a plain test client (no mesh, no second peer), GETs all
   five endpoints, and independently re-verifies each result. **A single static server plus a verifying
   client is shown sufficient** — the mesh (§22.5.2) and an IPFS adapter (§22.5.3) are strictly optional
   additional fetch-adapters behind the identical verification gate.

Two honest notes about the reference (not the spec): kerf-pub v1 ships **SHA2-256 (prefix `0x12`)** as
its digest rather than the v0-REQUIRED BLAKE3-256 (`0x1e`) — sanctioned by §23 Appendix A as the
Git-LFS/IPFS interop seam, swappable in one line — and it does **not** yet implement `DeviceCert` chains,
so it enforces `signer == pub` always (§22.9's full-blast-radius residual). Both are implementation
choices the spec permits to be tightened; a conformant v0 implementation SHOULD carry BLAKE3-256 and
SHOULD support the `DeviceCert` chain of §22.3.3 step 4.

---

## 7. Anti-abuse posture (profile of §22.6)

Feeds & Blobs inverts the mail anti-abuse model, and a substrate adopter should understand why it needs
**none** of the §9 cold-contact economics:

- **Announcements are fee-free** (§22.6.3). The §9 "cost for cold contact" apparatus exists because a
  MOTE is *pushed* into a recipient's inbox. A `PubAnnounce` is **pulled, never pushed** — appended to
  the *publisher's own* feed and fetched only by readers/holders who chose to follow it. A sender-paid
  challenge would protect no recipient, and is structurally incompatible with a public, cached,
  content-addressed object (it would break dedup, cacheability, and anonymous reads).
- **The cost is a *serving* cost, at the opt-in holder, and it is bounded.** A feed is single-author and
  `seq`/`prev`-chained, so an attacker can flood only *their own* feed, which only their own
  followers/holders pay for and can drop at will — no shared feed to spam, no fan-out amplification. A
  serving node applies ordinary admission limits (object-size ceiling, per-publisher quota, append rate);
  exceeding them is `ERR_PUB_SERVE_QUOTA` (`0x090D`, `DENY_POLICY`) — a policy deny, never a security
  gate, never a silent hole.
- **Serving is opt-in because holders are NOT blind** (§22.6.1). A sealed-chunk relay carries ciphertext
  it cannot read; a public-object holder serves **plaintext it can read**, which shifts its moderation
  and liability posture. Serving public content MUST therefore be an explicit operator choice (`pub-1`),
  never automatic. There is **no protocol-level takedown** (§22.6.2): a holder chooses what it serves;
  the protocol compels neither serving nor removal.

---

## 8. Security considerations / honest limits (profile of §22.9)

None is a defect to be fixed; each is an inherent consequence of the public quadrant.

1. **Irrevocability.** Once any independent holder retains a published object, the publisher cannot
   recall it. Retraction is a successor announcement (§4.4), never an erasure. Clients MUST disclose this
   before the act (§22.7).
2. **Publisher metadata is public by design.** `pub`, `roots`, `meta`, `ts`, and the whole feed are
   public — *who published what, and when* is exactly the fact this capability makes verifiable. A
   publisher for whom the mere fact of publishing is sensitive is out of scope (§22.1.2).
3. **Dedup reveals content equality across publishers.** Plaintext addressing lets an observer tell that
   two publishers hold the same bytes (§3.4). Inherent and accepted; safe only inside the explicitly
   published set — the publish-act boundary (§3.4) is the sole gate.
4. **Availability is not durability.** Serving is best-effort and opt-in; a public object is available
   only as long as some holder serves it. Durability, if wanted, is bought with pinning/replication
   (§5.5.2) and costs real storage.
5. **Signing-key compromise scope.** An announce/head is only as authentic as the `signer` key and its
   `DeviceCert` chain (§22.3.3). A compromised operational key can publish under the identity until
   revoked — and anything it published is itself irrevocable and must be superseded, not deleted. Keep
   `IK` cold (§1.2a).
6. **No read privacy — by construction.** Reads are anonymous to the object but not to the transport: an
   HTTP server or mesh holder sees *which reader fetched which public object.* A reader who needs to hide
   *that they read* must supply their own transport anonymity (Tor-class), outside this capability's
   scope.

---

## 9. Fail-closed rules this capability contributes

The full auditable set is §22.8 (mirrored into §10.7); the owning clause governs. Summary of the codes a
Feeds-adopting implementation MUST enforce:

| Trigger | Error | Action |
|---------|-------|--------|
| Public manifest carries key 5 | `ERR_PUB_MANIFEST_KEY_PRESENT` `0x0902` | FAIL_CLOSED_BLOCK |
| Sealed↔public manifest DS-tag confusion | `ERR_PUB_MANIFEST_TYPE_MISMATCH` `0x0903` | FAIL_CLOSED_BLOCK |
| Manifest / chunk self-verify fails | `0x0909` / `ERR_PUB_CHUNK_HASH_MISMATCH` `0x090A` | DROP_SILENT / ROTATE_RETRY |
| Announce id / sig-chain invalid | `0x0905` / `ERR_PUB_ANNOUNCE_SIG_INVALID` `0x0904` | DROP_SILENT / FAIL_CLOSED_BLOCK |
| Supersede is cross-author | `ERR_PUB_SUPERSEDE_INVALID` `0x090B` | FAIL_CLOSED_BLOCK |
| Feed `seq` rollback | `ERR_PUB_FEED_ROLLBACK` `0x0907` | FAIL_CLOSED_BLOCK — retain higher tip |
| Feed fork / broken chain | `ERR_PUB_FEED_CHAIN_BROKEN` `0x0908` | HALT_ALERT — publish conflicting entries as evidence |
| Feed head signature invalid | `ERR_PUB_FEED_SIG_INVALID` `0x0906` | FAIL_CLOSED_BLOCK |
| Unknown PUB version/suite | `ERR_PUB_UNSUPPORTED_VERSION` `0x0901` | FAIL_CLOSED_BLOCK — never guess |
| Publish implicit or deletion presented as achievable | (client non-conformance, §22.7) | publish MUST be explicit + warn irrevocability |
