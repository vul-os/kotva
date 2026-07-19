# 22. DMTAP-PUB: Public Objects (extension)

DMTAP v0 expresses one quadrant of the confidentiality × authenticity space: **content sealed to
members, authored by a sovereign identity** (§2, §5, §6). This appendix specifies **DMTAP-PUB**,
the additive extension that fills the missing quadrant — **authenticity *without* confidentiality**:
an object **signed by a sovereign identity (§1), readable by anyone, globally deduplicated**. It is
the inverse of sealed sender — here the publisher's identity is the *point*, not a secret.

DMTAP-PUB is **opt-in, additive, and capability-negotiated (§10.2)**. It changes **no** sealed-path
default, bumps **no** top-level `Envelope.v` (§18.1) and **no** DNS `v=` anchor (§3.2), and
introduces **no flag day**. Everything below rides the existing extension machinery: a message kind
in the reserved range (§2.3), a capability token (§10.2, §21.22), distinct domain-separation tags
(§18.1.6), and a fresh error block (§21). A node that does not implement DMTAP-PUB is unaffected: it
never advertises `pub-1`, never serves public objects, and never receives one it must reject.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174,
consistent with the rest of this specification. Where this appendix and §18 (wire format) appear to
differ, §18 governs the byte layout and the intent of this prose governs the semantics (§10.4).

Every DMTAP-PUB wire object is an integer-keyed CBOR map that inherits §18.1.2's conventions
unchanged: keys assigned per object type starting at `1`, keys **≥ 64 reserved** for
future/extension fields, and the signed-vs-unsigned unknown-key discipline (a decoder MUST reject
an unknown key in a **signed** object fail-closed; it MAY ignore unknown keys ≥ 64 in an
**unsigned** object).

## 22.1 Goals & non-goals

### 22.1.1 Goals

1. **Authenticity without confidentiality.** A `pub_announce` (§22.3) is signed **in the clear** by
   the publisher's operational key, chaining to its root identity key `IK` (§1.2). Anyone can fetch
   it and verify — offline, with **zero DNS and zero name-chain** (§3.13) — that identity `IK`
   published exactly these bytes at that time. There is no sealed sender, no payload encryption, no
   per-recipient delivery: publicity is the property being provided.
2. **Global, cross-user deduplication.** A public blob is addressed over its **plaintext** chunks
   (§22.2), so two publishers holding the same bytes compute the **same** content address and the
   swarm stores them once. This is the deliberate inverse of the sealed file model (§5.5), whose
   ciphertext addressing forbids cross-user dedup for privacy — here there is no privacy to protect,
   and dedup is the explicit purpose.
3. **Self-verifying, trustless serving.** Every DMTAP-PUB object — announce, manifest, chunk, feed
   entry — is authenticated by a signature or a content address it carries, so **any** node MAY
   serve **any** object without being trusted (§22.4, §22.5). Indexes built over public objects
   (search, category aggregation) are **derived, rebuildable, and never authoritative** (§22.4.3).
4. **Parity with the public-broadcast world.** DMTAP-PUB deliberately **subsumes** two niches the
   sealed protocol cannot express, using one object model instead of two ecosystems:
   - a **signed public event** stream keyed to a self-sovereign identity (the niche served by
     Nostr-style relays) — realized as an **author feed** (§22.4); and
   - **public content-addressed storage** with global dedup and swarmed retrieval (the niche served
     by IPFS-style pinning) — realized as the **public blob profile** (§22.2).

   The motivating parity uses (§17) are **public mailing-list archives**, **software release
   announcements and newsletters**, and **open-hardware / open-data part libraries**; the CAD
   artifact profile (§23) is the first production application over this substrate.

### 22.1.2 Non-goals

- **Not a CDN or durability contract.** DMTAP-PUB says nothing new about *how long* bytes remain
  available. Durability stays the **§5.5.1 tiered contract** (Inline / Attached / Referenced) and
  the **§5.5.2 durability descriptor** (origin-hold / recipient-pinned / cluster-replicated /
  pinned(term)); a served public blob is available exactly as long as some holder serves it, no
  more (§6.6 item 10). A content address is a name, not a promise (§5.5.1).
- **Not a change to any sealed default.** No sealed manifest becomes plaintext-addressed; §5.5's
  ban on plaintext content addressing for sealed files stands (§22.2.4). DMTAP-PUB is a **distinct,
  DS-tag-separated** address space that a verifier can never confuse with the sealed one
  (§22.2.3) — the carve-out is the extension, not a relaxation of the rule.
- **Not anonymity.** DMTAP-PUB provides the opposite of sealed sender: publisher identity, the set
  of published objects, and their timestamps are public **by design** (§22.9). A user who needs to
  publish *anonymously* is out of scope; that is a different problem than the one solved here.
- **Not moderation infrastructure.** There is **no protocol-level takedown** (§22.6). A holder
  chooses what it serves; the protocol neither compels serving nor compels removal.

## 22.2 Public blob profile

A public blob is a **plaintext-addressed Merkle-DAG manifest** — the structural twin of the sealed
`Manifest` (§5.5, §18.3.8), with three deliberate differences and everything else inherited.

### 22.2.1 `PubManifest`

```cddl
PubManifest = {
  1 => hash,            ; id        DS-tagged Merkle root over PLAINTEXT chunk hashes (§22.2.2)
  2 => u64,             ; size      total plaintext size in bytes
  3 => u32,             ; chunk_sz  fixed chunk size (e.g. 1 MiB, §16.4)
  4 => [+ hash],        ; chunks    ordered PLAINTEXT chunk hashes h_i (§22.2.2)
  ; key 5 is FORBIDDEN — a PubManifest has NO key field BY CONSTRUCTION (there is no AEAD,
  ;   no per-file key). Key 5 is reserved-unused so a sealed Manifest mis-served as a public
  ;   one (or a leaky encoder) is detected, never honored — mirrors §18.3.8's key-5 trap.
  6 => suite,           ; suite     hash suite governing each h_i and id (v0 BLAKE3-256); NO AEAD
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `id` | 1 | `hash` | MUST | Content address = the **DS-tag-domain-separated** Merkle root over the ordered `chunks` (§22.2.2). MUST self-verify: a recomputed root MUST equal `id`, else `ERR_PUB_MANIFEST_HASH_MISMATCH` (`0x0909`). |
| `size` | 2 | `u64` | MUST | Total plaintext file size. |
| `chunk_sz` | 3 | `u32` | MUST | Fixed chunk size; every chunk except possibly the last is exactly this many **plaintext** bytes. |
| `chunks` | 4 | `[+ hash]` | MUST | Ordered list of **plaintext** chunk content addresses `h_i = prefix ‖ BLAKE3-256(plaintext_i)` (§22.2.2). At least one. |
| ~~`key`~~ | ~~5~~ | — | **FORBIDDEN** | A `PubManifest` MUST NOT carry a key field; a public blob is unencrypted, so there is no key to carry. A decoder that finds key `5` MUST reject the object (`ERR_PUB_MANIFEST_KEY_PRESENT`, `0x0902`) — either a leaked sealed manifest or a malformation, never used. This mirrors the spirit of `ERR_MANIFEST_KEY_PRESENT` (§5.5, `0x0808`): the sealed manifest forbids key `5` *lest it leak*; the public manifest forbids it *because none exists*. |
| `suite` | 6 | `suite` | MUST | Suite governing the digest of each `h_i` and of `id`. There is **no AEAD selector** — public chunks are not encrypted (§18.1.4). |

Chunking, streaming, resumability, parallel/swarmed fetch, and per-chunk self-verification are all
**inherited unchanged from §5.5**; only the input to the hash and the tree's domain separation
differ. A `PubManifest` is itself a content-addressed blob served from the swarm, exactly like a
sealed `Manifest`.

### 22.2.2 Plaintext addressing (the fixed allocation)

Public chunks are hashed over **plaintext**, with the multihash-style agility prefix (§18.1.5), and
the tree root is domain-separated by the DMTAP-PUB manifest DS-tag so it can never collide with a
sealed root (§18.9.5) over an identical chunk-hash list:

```
; 1. Per-chunk hash — over PLAINTEXT, no AEAD, no key
h_i = 0x1e ‖ BLAKE3-256( plaintext_chunk_i )                 ; v0 prefix 0x1e = BLAKE3-256
                                                             ;   the value stored in PubManifest.chunks

; 2. RFC 6962-style binary Merkle tree over the ORDERED h_0 … h_{n-1},
;    domain-separated by the PUB manifest DS-tag "DMTAP-PUB-v0/manifest" ‖ 0x00
DS = "DMTAP-PUB-v0/manifest" ‖ 0x00
leaf(h_i)          = BLAKE3-256( DS ‖ 0x00 ‖ h_i )           ; 32-byte digest
node(left, right)  = BLAKE3-256( DS ‖ 0x01 ‖ left ‖ right )  ; 32-byte digest

MTH([h_0])                 = leaf(h_0)
MTH(h_0 … h_{n-1}), n > 1  = node( MTH(h_0 … h_{k-1}),       ; k = largest power of 2 < n
                                   MTH(h_k … h_{n-1}) )      ; (RFC 6962 split rule)

; 3. Public-manifest content address
PubManifest.id = 0x1e ‖ MTH(h_0 … h_{n-1})
```

The `h_i = prefix ‖ BLAKE3-256(plaintext_i)` construction is the normative allocation (contrast the
sealed `h_i = prefix ‖ BLAKE3-256(AEAD(key, plaintext_i))` of §18.9.5). Because the DS-tag is folded
into **every** leaf and node, a public root and a sealed root over the *same* chunk-hash list are
different values — the type is bound into the address, not asserted by a boolean flag (§22.2.3). The
prefix (§18.1.5) preserves hash-agility: an implementation may migrate the digest (e.g. to SHA2-256
for FIPS compliance) without changing the address format, and the same prefix is the
interoperability seam with external content-addressed stores (a Git-LFS / sha256 pointer maps onto
`0x12 ‖ SHA2-256(plaintext)`; see §23's appendix).

### 22.2.3 Type-incompatibility with sealed manifests (fail closed)

A verifier is always in exactly one of two modes: it expects a **sealed** `Manifest` (§5.5) or a
**public** `PubManifest`. The two are made **type-incompatible at the content-address level** so a
mix-up fails closed and is never a silent boolean:

- A verifier expecting a **sealed** manifest recomputes the root by the §18.9.5 rules (ciphertext
  chunk hashes, bare `0x00`/`0x01` tree). Fed a `PubManifest`, the recomputation cannot match `id`
  (different chunk-hash inputs *and* a different tree DS) → **reject**,
  `ERR_PUB_MANIFEST_TYPE_MISMATCH` (`0x0903`), FAIL_CLOSED_BLOCK.
- A verifier expecting a **public** manifest recomputes by §22.2.2. Fed a sealed `Manifest` (or one
  carrying the forbidden key field), it likewise mismatches, or trips the key-5 trap
  (`0x0902` / `0x0903`).

Normative: **a public manifest received where a sealed one is required MUST be rejected, and a
sealed manifest received where a public one is required MUST be rejected** — never coerced, never
"tried both ways." The DS-tag is the discriminator; there is no flag a peer could flip to make a
sealed blob masquerade as public (or vice versa).

### 22.2.4 The CAS-confirmation attack, head-on

The sealed file model (§5.5) hashes **ciphertext** precisely to defeat a **content-addressable-
storage confirmation attack**: if manifests were plaintext-addressed, a storage node — or anyone
holding a candidate plaintext — could confirm *which identities hold which file* by comparing
content addresses (a social-graph leak). DMTAP-PUB **plaintext-addresses on purpose**, so that
confirmation is possible — and this is **accepted by design**: the content is *published*, i.e. the
publisher's holding of it is public on purpose. Confirming that a public release announcement's blob
equals a known file leaks nothing the publisher did not already broadcast.

This acceptance has one hard, normative boundary:

> **A `PubManifest` MUST NOT be derived from content the user has not explicitly published.** An
> implementation MUST NOT plaintext-address, announce, or serve any object except as the result of
> an **explicit publish act** (§22.7). The CAS-confirmation property is safe *only* for content the
> user chose to make public; applying plaintext addressing to a user's private files would
> reintroduce exactly the leak §5.5 exists to prevent. The publish act (§22.7) is the sole gate
> from private to public, and it is irrevocable (§22.7, §22.9).

## 22.3 The `pub_announce` object (kind `0x40`)

A `pub_announce` is a **bare, unsealed, signed CBOR object** — not a MOTE. Unlike kinds `0x00`–`0x0b`
(§2.3), which ride *inside* a sealed `Envelope` (sealed sender + `ciphertext`), kind `0x40` names a
**public** object that carries the publisher's identity **in the clear** and is fetched by content
address (§22.5). That is why it lives in the extension kind range (§2.3, §21.16): it deliberately
inverts the envelope's confidentiality guarantees. There is no `Envelope`, no sealed sender, no
`ciphertext` — the publisher's identity is the object's purpose.

### 22.3.1 `PubAnnounce`

```cddl
PubAnnounce = {
  1 => u8,                       ; v          PUB object version, = 0 in v0
  2 => suite,                    ; suite      signature/hash suite (§18.1.4)
  3 => ik-pub,                   ; pub        publisher root identity key IK (§1.2) — the point
  4 => [+ hash],                 ; roots      referenced PubManifest root(s) (§22.2) — the content
  5 => { * tstr => ext-value },  ; meta       structured metadata map (profile-defined, §23; ext-value §18.3.6)
  ? 6 => hash,                   ; supersedes content address of a prior PubAnnounce (revision chain, §22.3.4)
  7 => ts,                       ; ts         publish timestamp (ms epoch)
  8 => ik-pub,                   ; signer     operational key that produced `sig`; a DeviceCert (§1.2) chains it to `pub`
  9 => sig-val,                  ; sig        signer over det_cbor(PubAnnounce ∖ {9}), DS-tag DMTAP-PUB-v0/announce
}
```

| Field | Key | Type | Presence | Meaning & constraints |
|-------|----:|------|----------|-----------------------|
| `v` | 1 | `u8` | MUST | PUB object format version. MUST equal `0` in v0; any other value is rejected fail-closed (`ERR_PUB_UNSUPPORTED_VERSION`, `0x0901`). Distinct from the frozen top-level `Envelope.v` (§18.3.1); DMTAP-PUB evolves through its own registries, not this byte. |
| `suite` | 2 | `suite` | MUST | Algorithm suite for `sig` and for the `roots` digests. Unknown ⇒ reject fail-closed (`0x0901`; the extension analogue of `ERR_UNKNOWN_SUITE` §1.1/`0x0101`). |
| `pub` | 3 | `ik-pub` | MUST | The **publisher's root identity key** `IK`. Carried in the clear — authenticity, not anonymity. A reader binds `pub` to a human name only if it wants to *display* one, via ordinary pinning/KT (§3.4/§3.5); verification itself needs no name (§22.3.3). |
| `roots` | 4 | `[+ hash]` | MUST | One or more `PubManifest.id` content addresses (§22.2). At least one; an announce with an empty `roots` is malformed. Multiple roots let one announce publish a set (e.g. an artifact in several formats, §23). |
| `meta` | 5 | `{* tstr => ext-value}` | MUST (MAY be empty) | Structured metadata (title, kind, license, etc.), text-keyed and restricted to the deterministic-safe `ext-value` subset (§18.3.6) because it rides inside the signed body. Concrete schemas are **profile-defined** (§23); a reader MUST ignore keys it does not recognize (forward-compat, §21.20). A profile MAY carry a compact integer-keyed schema by embedding it as deterministic CBOR (§18.1.1) in a `bytes` value under a single profile-named key (e.g. `"artifact"`, §23.3.1) — opaque to a generic reader, which ignores the key like any other unrecognized one; the embedded bytes are covered by `sig` like every other `meta` value. |
| `supersedes` | 6 | `hash` | OPTIONAL | Content address of a prior `PubAnnounce` this one revises, borrowing `edit` (`0x03`, §2.3/§5.4) semantics: a successor supersedes a predecessor by id, forming a revision chain (§22.3.4). The referenced announce MUST have the **same** `pub`, else `ERR_PUB_SUPERSEDE_INVALID` (`0x090B`). Absent ⇒ this is an original announcement. |
| `ts` | 7 | `ts` | MUST | Publish wall-clock time (ms epoch), subject to clock-skew tolerance (§16.1). Used for display/ordering; feed `seq` (§22.4) is authoritative for order, never `ts`. |
| `signer` | 8 | `ik-pub` | MUST | The **operational (device) key** that produced `sig`. It MUST be authorized by `pub` via a `DeviceCert` (§1.2) that the verifier checks (§22.3.3); `signer` MAY equal `pub` when `IK` signs directly. Keeping `IK` cold (§1.2a) is RECOMMENDED — an operational key signs day-to-day publishes. |
| `sig` | 9 | `sig-val` | MUST | Signature by `signer` over `DMTAP-PUB-v0/announce ‖ 0x00 ‖ det_cbor(PubAnnounce ∖ {9})` (§18.1.6). A failure is `ERR_PUB_ANNOUNCE_SIG_INVALID` (`0x0904`). For `suite = 0x02` the AND-composition rule of §18.1.6 applies (both component signatures MUST verify). |

**Content address.** A `PubAnnounce` has no self-`id` field — a field cannot contain its own hash.
Following the `Identity`-anchor rule (§18.9.4), the announce's content address is derived from the
**fully-signed** object:

```
announce_id = 0x1e ‖ BLAKE3-256( det_cbor(PubAnnounce) )     ; the complete, signed object
```

`announce_id` is what a feed entry (§22.4) and a `supersedes` reference (§22.3.4) point at, and what
a fetcher recomputes and checks on receipt (`ERR_PUB_ANNOUNCE_ID_MISMATCH`, `0x0905`) before trusting
the bytes. *(Rejected alternative: a self-referential `id` field — circular and impossible to
compute; the derived-anchor rule of §18.9.4 is used instead, for consistency with `Identity`.)*

### 22.3.2 Why signed plaintext, not a sealed MOTE

A `pub_announce` is the deliberate inverse of the MOTE (§2.1): the MOTE hides sender identity inside
`ciphertext` (§2.4); the announce **exposes** it in `pub`/`signer`. There is no per-recipient
sealing because there is no recipient — the object is published once and fetched by anyone. This is
also why the anti-abuse `challenge` field (§2.2b) is **absent**: a `challenge` protects a *recipient*
from unsolicited push (§9), and a public announce pushes to no one (§22.6.3).

### 22.3.3 Verification (offline, zero-DNS)

A verifier presented with a `PubAnnounce` MUST, in order:

1. Reject unknown `v`/`suite` (`0x0901`, fail closed) — the extension analogue of §2.7 step 1.
2. Recompute `announce_id = 0x1e ‖ BLAKE3-256(det_cbor(PubAnnounce))` and reject on mismatch against
   the address it was fetched by (`0x0905`).
3. Verify `sig` under `signer` over the DS-tagged preimage (§22.3.1); drop on failure (`0x0904`).
4. Verify `signer` is authorized by `pub` — either `signer == pub`, or a `DeviceCert` (§1.2) signed
   by `pub` (or an `IK`-authorized chain) covers `signer` and is not revoked (§1.5). A broken chain
   is `ERR_PUB_ANNOUNCE_SIG_INVALID` (`0x0904`).
5. If `supersedes` is present, require the referenced announce's `pub` to equal this `pub`
   (`0x090B` on mismatch) — a publisher may only supersede its *own* announcements.

This is **entirely offline-verifiable with zero DNS and zero name-chain**, consistent with §3.13:
authenticity is a property of the keys carried in the object (`pub`, `signer`, and the `DeviceCert`
chain), never of a name lookup. A name is needed only to *display* "who is `pub`" and is an optional
convenience layered over an identity that is already verified (§3.13.2). Replay and ordering are
handled by the feed (§22.4), not by the announce in isolation — a bare announce carries no anti-
rollback state; its position in an author feed does.

### 22.3.4 Revision chains (`supersedes`)

`supersedes` borrows `edit` (`0x03`) semantics (§2.3, §5.4): a later announce names a prior one by
`announce_id`, and a client renders the **latest** announcement in a chain while retaining the
lineage for audit. Because each announce is immutable and content-addressed, a revision is a **new**
object with a **new** `announce_id`, not a mutation of the old one — the predecessor remains fetchable
forever (§22.9). **Deprecation / yank is a successor announcement**, never a deletion: a publisher
marks a revision deprecated in `meta` (schema per §23) and points `supersedes` at the retired one.
This is the only honest model under irrevocability (§22.7): you cannot un-publish, you can only
publish a correction. A chain MUST be single-author (every link's `pub` identical, §22.3.3 step 5);
a fork in a revision chain is a client-display concern, resolved by feed order (§22.4), not a
protocol fault.

## 22.4 Author feeds

An **author feed** is a **per-identity, append-only, signed log** of that identity's announcements —
the DMTAP-PUB analogue of a KT log (§3.5) or the cluster journal (§5.6.3(b)), scoped to one
publisher. It gives ordering, discovery, and anti-rollback that a bare announce cannot.

### 22.4.1 Structure

```cddl
FeedEntry = {
  1 => u64,             ; seq       strictly increasing per author feed, genesis = 0
  2 => hash,            ; announce  content address of the PubAnnounce at this position (§22.3.1)
  ? 3 => hash,          ; prev      content address of the FeedEntry at seq-1; ABSENT iff seq = 0 (genesis)
  4 => ts,              ; ts        entry time (ms epoch)
}

FeedHead = {
  1 => u8,              ; v         = 0 in v0
  2 => suite,           ; suite     signature/hash suite
  3 => ik-pub,          ; pub       the feed's author identity key IK
  4 => u64,             ; seq       highest seq in this head — the current tip
  5 => hash,            ; tip       content address of the FeedEntry at `seq` (the log head)
  6 => ts,              ; ts        head publication time
  7 => ik-pub,          ; signer    operational key (DeviceCert chains to `pub`, §1.2)
  8 => sig-val,         ; sig       signer over det_cbor(FeedHead ∖ {8}), DS-tag DMTAP-PUB-v0/feed
}
```

| Object | Field | Key | Type | Presence | Meaning & constraints |
|--------|-------|----:|------|----------|-----------------------|
| `FeedEntry` | `seq` | 1 | `u64` | MUST | Position in the feed; strictly increasing by 1 from the genesis `0`. |
| | `announce` | 2 | `hash` | MUST | The `announce_id` (§22.3.1) published at this position. |
| | `prev` | 3 | `hash` | OPTIONAL | Content address of the `FeedEntry` at `seq-1`, chaining the log. Present for every entry **except** genesis (`seq = 0`); a non-genesis entry lacking `prev`, or a genesis entry carrying one, is malformed (`ERR_PUB_FEED_CHAIN_BROKEN`, `0x0908`). |
| | `ts` | 4 | `ts` | MUST | Entry time (§16.1). |
| `FeedHead` | `v` | 1 | `u8` | MUST | `= 0`; unknown ⇒ `0x0901`. |
| | `suite` | 2 | `suite` | MUST | Unknown ⇒ `0x0901`. |
| | `pub` | 3 | `ik-pub` | MUST | The feed's author `IK`. A feed is **single-author** by construction — one identity, one feed. |
| | `seq` | 4 | `u64` | MUST | The tip's `seq` — the highest position this head commits to. |
| | `tip` | 5 | `hash` | MUST | Content address of the `FeedEntry` at `seq`. Because each entry chains `prev` to its predecessor, `tip` **transitively commits to the entire log** (RFC 6962 discipline); signing the head therefore authenticates every entry reachable from it, so `FeedEntry` needs **no** per-entry signature (as with cluster journal entries, §5.6.3(b), and KT leaves, §3.5). |
| | `ts` | 6 | `ts` | MUST | Head time. |
| | `signer` | 7 | `ik-pub` | MUST | Operational key; authorized by `pub` via `DeviceCert` (§1.2), checked as in §22.3.3 step 4. |
| | `sig` | 8 | `sig-val` | MUST | Signature by `signer` over `DMTAP-PUB-v0/feed ‖ 0x00 ‖ det_cbor(FeedHead ∖ {8})`. Failure ⇒ `ERR_PUB_FEED_SIG_INVALID` (`0x0906`). |

**`FeedEntry` content address (normative).** A `FeedEntry` carries no signature of its own; the
content address that names it — the value carried in a successor's `prev` and in `FeedHead.tip` —
is derived from the complete object by the generic §18.9.4 anchor rule:
`entry_id = 0x1e ‖ BLAKE3-256( det_cbor(FeedEntry) )`, with **no** DS-tag fold (contrast the
signed objects above, whose DS-tags separate *signing preimages*; an unsigned entry's authenticity
flows solely from the signed head's transitive `tip` commitment, §22.4.1). *(Made explicit here
because conformance-vector work showed it was previously only inferable.)*

### 22.4.2 Anti-rollback (the standard monotonic-`seq` rule)

A feed's `seq` obeys the **same anti-rollback rule family** the spec applies everywhere a monotonic
counter guards against stale-replay suppression — `caps_version` (§10.2), `Identity.version` (§1.3),
`LocationRecord.seq` (§4.2), and `GroupState.version` (§5.8.2) — **deliberately relaxed for
pull-fetched heads**: rejection is strict-less-than (`<`) instead of the `≤` those push-delivered
counters use, since a reader legitimately re-fetches the same tip. Specifically:

- A reader **retains the highest `seq` it has accepted per author feed** (keyed by `pub`).
- A reader MUST **reject any `FeedHead` whose `seq` is strictly less than** the last accepted for
  that `pub` (`ERR_PUB_FEED_ROLLBACK`, `0x0907`). Feeds only ever grow; a stale head cannot
  *suppress* announcements a publisher has since made (the public-feed analogue of a capability-
  suppression downgrade, §10.2). An **equal** `seq` is *not* a rollback — a feed head is a
  cacheable object a reader legitimately re-fetches, and re-presenting the current tip must be
  idempotent, not an error. On equal `seq` the reader instead compares `tip`: identical ⇒ accept
  (no-op); different ⇒ two heads claim the same position, which is equivocation, handled as a
  chain fork (`ERR_PUB_FEED_CHAIN_BROKEN`, `0x0908`, below) — never as a rollback. *(This
  deliberately differs from the strict `≤` rejection used by `caps_version`/`Identity.version`,
  which are push-delivered announcements a peer should never replay at the same version; a public
  feed head is pull-fetched and MUST tolerate the reader fetching the same tip twice.)*
- The `prev` hash-chain makes a **fork detectable**: two distinct `FeedEntry`s at the same `seq` with
  the same `prev` — or a `prev` that does not resolve to the entry at `seq-1` — is evidence that the
  author's own log was rewritten or equivocated. This is `ERR_PUB_FEED_CHAIN_BROKEN` (`0x0908`),
  handled `HALT_ALERT`, the same fork-detection posture as a committer fork (`0x0404`) and a cluster-
  journal break (`0x0412`). A publisher cannot honestly present two histories; a reader that sees
  both holds transferable evidence.

Absence of an expected announcement in a *current* (highest-`seq`) head is inconclusive, not a
negative assertion — a publisher may simply not have published it (cf. §21.22's capability-absence
rule).

### 22.4.3 Trustless serving; derived indexes

A feed is **self-verifying**: a fetcher validates the signed `FeedHead`, then walks `prev` from the
`tip`, checking each `FeedEntry` hashes into its successor's `prev` and each `announce` self-addresses
(§22.3.1). Therefore **any node MAY serve any feed** without being trusted — a malicious server can
withhold or stall (detectable: a stale head trips `0x0907`; a gap trips `0x0908`) but cannot **forge**
an entry (that needs `pub`'s key) nor **hide** a published one without the reader noticing a `seq`/
chain discontinuity against another source.

**Indexes are derived data.** Search indexes, category aggregations, "trending," and follower graphs
built *over* feeds are **rebuildable and never authoritative** — the authoritative state is always the
set of signed feeds. Any node MAY build an index; a disagreement between an index and a feed is always
resolved in favour of the feed. This mirrors the cluster rule that indexes over immutable objects are
derived (§5.6.2) and the §23 "a workshop is a set of followed feeds; its category index is derived
data any node can rebuild."

### 22.4.4 Head retrieval and range fetch (transport-independent)

The feed is manipulated through four **abstract operations**, defined here independently of any
transport (§22.5 gives concrete bindings):

- `feed_head(pub) → FeedHead` — the current signed tip for author `pub`. The **only** mutable
  operation; a reader applies §22.4.2 anti-rollback on the result.
- `feed_range(pub, from_seq, to_seq) → [FeedEntry]` — a contiguous slice of the log, each entry
  verified by the `prev` chain up to the signed `tip`. `to_seq` MAY be the head's `seq`.
- `announce(id) → PubAnnounce` — a content-addressed fetch of one announcement, self-verifying by
  `id` (§22.3.1).
- `blob(id) → PubManifest` and `chunk(h) → bytes` — content-addressed fetches of a public manifest
  and a plaintext chunk, self-verifying by `id` / by `h_i` (§22.2).

All five are **read-only, content-addressed, and (for the content-addressed four) immutable**, which
is what makes the HTTP and mesh bindings below cache-friendly and trustless.

## 22.5 Serving

DMTAP-PUB objects are served over two interchangeable transports. Both expose the §22.4.4 operations;
a fetcher verifies identically regardless of where bytes came from, because every object is self-
authenticating. The section is kept abstract enough that **plain HTTPS with no mesh present is a
complete implementation** — the intended first deployment.

### 22.5.1 Gateway HTTP profile

A node advertising `pub-1` (§22.6) MAY expose a **well-known HTTP surface**. Reads are **anonymous**
(no authentication for public reads) and **content-addressed**:

```
GET /.well-known/dmtap-pub/feed/{pub}/head              → FeedHead        (application/cbor)
GET /.well-known/dmtap-pub/feed/{pub}/range?from=&to=   → [FeedEntry]     (application/cbor)
GET /.well-known/dmtap-pub/announce/{id}                → PubAnnounce     (application/cbor)
GET /.well-known/dmtap-pub/manifest/{id}                → PubManifest     (application/cbor)
GET /.well-known/dmtap-pub/chunk/{h}                     → raw plaintext chunk bytes
```

- `{pub}` is the base64url (unpadded) publisher identity key; `{id}` and `{h}` are the base64url of
  the full `hash` (prefix ‖ digest, §18.1.5).
- **Cacheability.** The four content-addressed endpoints return **immutable** objects; a server
  SHOULD send `Cache-Control: public, immutable, max-age=<long>` and a strong `ETag` equal to the
  content address, and MAY be fronted by any ordinary HTTP cache/CDN. `feed/.../head` is **mutable**;
  it SHOULD carry a short TTL / `must-revalidate`. A cache never needs to understand DMTAP — content
  addressing makes correctness a property of the bytes, not of the cache.
- **Verification is the client's job, always.** A fetcher MUST verify signatures and content
  addresses (§22.3.3, §22.4) on every response; an HTTP server (or CDN, or attacker in its place)
  cannot forge an object it did not receive `pub`'s key to sign, and cannot substitute chunk bytes
  under a matching `h_i` (BLAKE3 collision resistance, §5.5.3). A server is a convenience, not a
  trust root.

### 22.5.2 Native mesh fetch

The same five operations are served over the mesh (§4.5 bulk path, §5.5 swarm): a holder advertising
`pub-1` answers content-addressed fetches for announces, manifests, chunks, and serves a feed's head
and ranges. Public objects are **not** routed through the mixnet — there is no metadata to protect
(publisher and content are public), so mesh serving uses the direct/`fast` content-addressed fetch,
not the sealed `private` tier. A mesh holder announces the set of content addresses it serves exactly
as it does for sealed chunks, minus the blindness (§22.6.1).

### 22.5.3 Swarm rules (inherited from §5.5, with global dedup)

Swarm behaviour is **inherited from §5.5.3**: chunks self-verify against their content address before
use; a holder cannot serve wrong-but-accepted bytes (a mismatch is `ERR_PUB_CHUNK_HASH_MISMATCH`,
`0x090A` → rotate to another holder, exactly as `0x0802` does for sealed chunks); a popular object
becomes *more* available as more nodes hold it; poisoning is reduced to wasted bandwidth bounded by
the swarm parallelism cap (§16.4). The **one** difference is the payoff of plaintext addressing:
because two publishers of identical bytes compute the **same** content address, dedup is **global and
cross-user** — the direct inverse of §5.5's intra-key-scope-only dedup. A file published once is,
from then on, one set of chunks in the swarm no matter how many publishers reference it, and a cheap
fork (a derived revision, §23) that changes few chunks shares the rest by construction.

## 22.6 Operator opt-in & anti-abuse

### 22.6.1 `pub-1` capability and the not-blind holder

Serving public objects is **per-operator opt-in**, gated by the **`pub-1` capability** (§10.2,
§21.22): a node advertises `pub-1` iff its operator has chosen to serve public objects, and a node
that never advertises it is never expected to serve them. Opt-in is **mandatory here in a way it is
not for sealed relay**, for one decisive reason:

> **Public-object holders are NOT blind.** A §5.5 sealed-chunk holder relays **ciphertext it cannot
> read** — blind chunk serving (§5.5) means an operator carries bytes without knowing or being
> responsible for their content. A DMTAP-PUB holder serves **plaintext it can read**. Serving public
> content therefore **shifts the operator's moderation and liability posture** in a way blind relay
> does not, so it MUST be an explicit operator choice, never automatic. (§6.6 item 12 records
> this as an honest limit.)

### 22.6.2 Per-holder serve policy; no protocol takedown

Each holder applies its **own** serve policy: it MAY decline to serve any object for any reason
(content, size, publisher, jurisdiction). **Refusal to serve is not a protocol error** against
correctness — it is a `DENY_POLICY` at the holder (`ERR_PUB_NOT_SERVED`, `0x090C`), and a fetcher
responds by rotating to another holder (`ROTATE_RETRY`), exactly as it would for an unavailable
sealed chunk. There is **no protocol-level takedown**: the protocol has no mechanism to compel any
holder to serve or to stop serving, and no holder can force another to drop an object. Availability
is the emergent sum of independent holder choices — the same honest posture as §5.5.4's "you cannot
force-delete bytes others pinned" and §22.9's irrevocability, seen from the serving side.

### 22.6.3 Interaction with §9 — announcements are fee-free (decision)

**Decision: a `pub_announce` requires NO §2.2b anti-abuse challenge; announcements are fee-free at
the protocol level.** Reasoning:

- The entire §9 "cost for cold contact" apparatus exists because a MOTE is **pushed** into a
  recipient's inbox: an unsolicited sender spends the *recipient's* storage and attention, so §9
  makes the sender pay (ARC token / PoW / postage) before that push is honored (§2.7 step 6, §9.1).
- A `pub_announce` is **pulled, never pushed**. It is appended to the **publisher's own** feed
  (§22.4) and fetched only by readers/holders who **chose** to follow or serve that publisher
  (§22.5). No one receives an announcement they did not ask for; there is no cold inbox to protect.
  A sender-paid challenge would therefore protect **no recipient** — it is a push-model defense
  misapplied to a pull model.
- A challenge is also **structurally incompatible** with a public, cached, content-addressed object:
  a §2.2b proof is per-recipient-scoped and ephemeral (bound to an envelope's `sender_key`, §9.2a),
  whereas an announce is a single immutable object served identically to everyone. A per-reader proof
  cannot be baked into a globally-shared content address without breaking dedup and cacheability
  (§22.5.1), and requiring one per fetch would break **anonymous public reads** (§22.5.1).

The abuse surface that *does* exist is a **serving cost**, and it lives at the **holder**, not the
reader — and it is bounded structurally:

- A feed is **single-author** and `seq`/`prev`-chained (§22.4): an attacker can flood only **their
  own** feed, which only *their own* followers/holders pay for and can drop at will (§22.6.2). There
  is no shared feed to spam and no fan-out amplification (contrast §9.9's group-address amplification,
  which DMTAP-PUB structurally lacks).
- A serving node applies its own **admission limits** (object-size ceilings, per-publisher storage
  quota, feed-append rate) as ordinary §9-style resource policy; exceeding them is
  `ERR_PUB_SERVE_QUOTA` (`0x090D`, `DENY_POLICY`), the DMTAP-PUB analogue of `ERR_STORAGE_QUOTA_
  EXCEEDED` (§5.5.5, `0x0806`) — a policy deny, never a security/crypto gate, and never a silent
  hole.

**Rejected alternative: require a §2.2b challenge (ARC / PoW / postage) on every `pub_announce`.**
Rejected because (a) it misapplies a push-model defense to a pull-model object — with no recipient
being force-fed, "cost for cold contact" defends nobody; (b) it would destroy global dedup and
cache-friendliness (a per-recipient, ephemeral proof cannot live inside a shared, immutable content
address, §22.5.1); and (c) it would deanonymize or gate the anonymous public read that is a design
goal (§22.5.1). Anti-abuse for public objects belongs at the **opt-in holder** (per-holder policy +
resource limits), not in a sender-paid stamp on a pulled object.

## 22.7 Client requirements

- **Explicit publish act (MUST).** A client MUST NOT create, plaintext-address, announce, or serve a
  public object except as the direct result of an **explicit user act** to publish that specific
  content. There is no implicit, batch, or background path from a private file to a public blob; the
  publish act is the sole gate (§22.2.4).
- **Irrevocability warning (MUST — normative UX).** Before completing a publish, a client MUST warn
  the user, explicitly and unambiguously, that **publishing is irrevocable**: a content-addressed,
  swarmed public object **cannot be un-published**. This mirrors the spec's other client-MUST
  disclosures (the deniability residual disclosure §5.2.1(e), the `sensitive`/`redact` cooperative-
  only warnings §6.6 item 8, the origin-hold best-effort disclosure §6.6 item 10): an honest, in-
  product statement of a hard limit, surfaced before the irreversible action, not buried after it.
- **Supersede / deprecate, never delete (MUST).** To retract or revise, a client MUST publish a
  **successor** announcement (`supersedes`, §22.3.4) — a correction or a deprecation marker — and MUST
  NOT present deletion as achievable. A client MAY stop serving its *own* copy (which removes it from
  the swarm only insofar as no other holder retains it, §22.6.2), but MUST NOT imply this deletes the
  object for others.

## 22.8 Conformance & fail-closed table

DMTAP-PUB adds the following invariants to the auditable fail-closed set (§10.7). Each is a **MUST**;
the owning clause here is authoritative and this table indexes it, in the §10.7 format. A conformant
implementation of `pub-1` enforces every row.

| Invariant | Clause | Trigger | Behavior / error on violation |
|-----------|--------|---------|-------------------------------|
| **Manifest DS-tag confusion (sealed ↔ public)** | §22.2.3 | a public `PubManifest` supplied where a sealed §5.5 `Manifest` is required, or a sealed manifest where a public one is required | reject — recomputed content address cannot match across the DS-tag/hash-input boundary; `ERR_PUB_MANIFEST_TYPE_MISMATCH` `0x0903`, FAIL_CLOSED_BLOCK; never "try both ways" |
| **Public manifest carries a key field** | §22.2.1 | a `PubManifest` carrying the forbidden key `5` | reject as leaked/malformed; `ERR_PUB_MANIFEST_KEY_PRESENT` `0x0902`, FAIL_CLOSED_BLOCK — mirrors `ERR_MANIFEST_KEY_PRESENT` (`0x0808`) |
| **Public manifest self-verification** | §22.2.2 | recomputed DS-tagged Merkle root ≠ `PubManifest.id` | reject before fetch; `ERR_PUB_MANIFEST_HASH_MISMATCH` `0x0909`, DROP_SILENT |
| **Public chunk self-verification** | §22.2.2, §5.5.3 | a fetched plaintext chunk ≠ its listed `h_i` | rotate to another holder; `ERR_PUB_CHUNK_HASH_MISMATCH` `0x090A`, ROTATE_RETRY |
| **Announce content-address bind** | §22.3.1, §22.3.3 | recomputed `announce_id` ≠ the address it was fetched by | reject; `ERR_PUB_ANNOUNCE_ID_MISMATCH` `0x0905`, DROP_SILENT |
| **Announce signature + IK chain** | §22.3.1, §22.3.3 | `sig` fails under `signer`, or `signer` is not authorized by `pub` (DeviceCert chain, §1.2) | reject; `ERR_PUB_ANNOUNCE_SIG_INVALID` `0x0904`, FAIL_CLOSED_BLOCK |
| **Supersede is same-author** | §22.3.4 | `supersedes` references an announce whose `pub` differs from this one | reject the revision link; `ERR_PUB_SUPERSEDE_INVALID` `0x090B`, FAIL_CLOSED_BLOCK — a publisher may only supersede its own announcements |
| **Feed `seq` anti-rollback** | §22.4.2 | a `FeedHead` with `seq` strictly below the highest accepted for that `pub` | reject the stale head, retain the higher tip; `ERR_PUB_FEED_ROLLBACK` `0x0907`, FAIL_CLOSED_BLOCK. Equal `seq` + identical `tip` ⇒ idempotent re-fetch, accept; equal `seq` + different `tip` ⇒ equivocation, `0x0908` HALT_ALERT — never a rollback |
| **Feed hash-chain integrity (fork)** | §22.4.2 | two `FeedEntry`s at one `seq` with the same `prev`, or a `prev` not resolving to `seq-1` | `ERR_PUB_FEED_CHAIN_BROKEN` `0x0908`, HALT_ALERT — same posture as a committer fork (`0x0404`) / cluster-journal break (`0x0412`); publish the conflicting entries as evidence |
| **Feed head signature** | §22.4.1 | `FeedHead.sig` fails under `signer`/`pub` chain | reject; `ERR_PUB_FEED_SIG_INVALID` `0x0906`, FAIL_CLOSED_BLOCK |
| **Unknown PUB version/suite** | §22.3.1, §22.4.1 | a `PubAnnounce`/`PubManifest`/`FeedHead` carrying a `v`/`suite` the implementation does not support | reject, never guess; `ERR_PUB_UNSUPPORTED_VERSION` `0x0901`, FAIL_CLOSED_BLOCK — the extension analogue of the unknown-suite rule (§1.1, `0x0101`) |
| **Serve refusal is policy, not fault** | §22.6.2 | a holder declines to serve a requested public object per its serve policy | `ERR_PUB_NOT_SERVED` `0x090C`, DENY_POLICY at the holder; the fetcher rotates to another holder — refusal is NOT a correctness error and NEVER a protocol-level takedown |
| **Serving resource limit** | §22.6.3 | a serving node's admission policy (object size / per-publisher quota / append rate) is exceeded | `ERR_PUB_SERVE_QUOTA` `0x090D`, DENY_POLICY — a policy deny, never a security/crypto gate, never a silent hole (cf. `0x0806`) |
| **Publish is explicit + irrevocable (UX)** | §22.7 | a would-be publish that is implicit/background, or a UI that presents deletion as achievable | non-conformant client — the publish act MUST be explicit and MUST warn irrevocability; retraction is supersede-only |

The governing rule of §10.7.5 applies unchanged: a DMTAP-PUB security-relevant failure is either
refused (fail closed) or surfaced as an explicit choice, never a silent degradation. New DMTAP-PUB
downgrade/fail-closed rules MUST be mirrored into §10.7 so the auditable set stays complete.

## 22.9 Security considerations / honest limits

Stated plainly, per the project's honest-limits governance (§6.6, §6.9). None of these is a defect to
be fixed; each is an inherent consequence of the public quadrant, disclosed for what it is.

1. **Irrevocability.** A published object is content-addressed and swarmed; once any independent
   holder retains it, the publisher **cannot** recall it (§22.6.2, §22.7). Deletion is cooperative-
   only and never a guarantee — the storage analogue of the `redact`/`expires` un-share bound (§6.6
   item 8) and the pinned-copy bound (§5.5.4), now made *intentional*. Retraction is a successor
   announcement (§22.3.4), never an erasure. Clients MUST disclose this before the act (§22.7).
2. **Publisher metadata is public by design.** `pub`, the set of `roots`, `meta`, `ts`, and the whole
   feed (§22.4) are **public** — *who published what, and when* is exactly the fact DMTAP-PUB exists
   to make verifiable. This is the deliberate inverse of sealed sender (§2.1, §6.2): the sealed path
   hides the author; the public path certifies them. A publisher for whom the mere fact of publishing
   is sensitive is out of scope (§22.1.2) and should not use DMTAP-PUB.
3. **Dedup reveals content equality across publishers.** Because addressing is over plaintext
   (§22.2.2), an observer can tell that two publishers published the **same bytes** (identical content
   addresses), and can run the CAS-confirmation test against any candidate plaintext (§22.2.4). This
   is **inherent and accepted**: the content is public, so its equality to a known file leaks nothing
   the publisher did not broadcast. The single hard boundary is normative and load-bearing: a public
   manifest MUST NOT be derived from content the user has not explicitly published (§22.2.4) — the
   property is safe only inside the published set, and the explicit publish act (§22.7) is the sole
   gate into it.
4. **Availability is not durability.** Serving is best-effort and opt-in (§22.6); DMTAP-PUB adds no
   durability guarantee (§22.1.2). A public object is available exactly as long as some holder serves
   it, and a fetch that finds none fails like any unavailable content-addressed object (the §5.5.1 /
   §6.6 item 10 residual, re-stated for the public case). Durability, if wanted, is bought the same
   way as for sealed files — pinning/replication (§5.5.2) — and costs real storage.
5. **Signing-key compromise scope.** A `pub_announce`/`FeedHead` is only as authentic as the `signer`
   key and its `DeviceCert` chain to `IK` (§22.3.3). Compromise of an operational signing key lets an
   attacker publish under the identity until the device is revoked (§1.5) and the identity heals
   (§6.6 item 3) — the ordinary DMTAP endpoint residual, with the public twist that anything the
   attacker published in the interval is itself irrevocable (item 1) and must be superseded, not
   deleted, after recovery. Keeping `IK` cold and signing with a revocable operational key (§1.2a,
   §22.3.1) bounds the blast radius.
6. **No metadata privacy for reads — by construction.** Public reads are anonymous to the object
   (no auth, §22.5.1) but not to the transport: an HTTP server or mesh holder sees *which reader
   fetched which public object from it*. DMTAP-PUB does not route public fetches through the mixnet
   (§22.5.2) because the objects carry no secret; a reader who needs to hide *that they read a public
   object* must supply their own transport anonymity (Tor-class), which is outside this extension's
   scope.

## 22.10 Error registry (`ERR_PUB_*`, `0x0900`–`0x09FF`)

DMTAP-PUB occupies subsystem byte **`0x09`** — the §21.1 subsystem table assigns it to this
extension, and §21.24b records the registration under the §21.14 new-subsystem-byte policy. Codes
follow the §21 conventions and responder-action vocabulary (§21.2). The table below is the block's
initial, authoritative contents.

| Code | Name | Operation(s) | Meaning | Retryable | Action |
|------|------|--------------|---------|:---------:|--------|
| `0x0901` | `ERR_PUB_UNSUPPORTED_VERSION` | `PubAnnounce`/`PubManifest`/`FeedHead` parse (§22.3.1, §22.4.1) | Object carries a PUB `v`/`suite` this implementation does not support. | No | FAIL_CLOSED_BLOCK |
| `0x0902` | `ERR_PUB_MANIFEST_KEY_PRESENT` | `PubManifest` decode (§22.2.1) | A public manifest carries the forbidden key field (`5`) — a public blob has no key by construction. | No | FAIL_CLOSED_BLOCK — reject; never use an embedded key |
| `0x0903` | `ERR_PUB_MANIFEST_TYPE_MISMATCH` | Manifest type check (§22.2.3) | A public manifest was supplied where a sealed §5.5 manifest is required, or a sealed manifest where a public one is required — DS-tag/address-space confusion. | No | FAIL_CLOSED_BLOCK |
| `0x0904` | `ERR_PUB_ANNOUNCE_SIG_INVALID` | `PubAnnounce` verification (§22.3.3) | `sig` fails under `signer`, or `signer` is not authorized by `pub` (DeviceCert chain). | No | FAIL_CLOSED_BLOCK |
| `0x0905` | `ERR_PUB_ANNOUNCE_ID_MISMATCH` | `PubAnnounce` fetch (§22.3.1, §22.3.3) | The recomputed `announce_id` does not equal the content address the object was fetched by. | Yes (re-fetch) | DROP_SILENT |
| `0x0906` | `ERR_PUB_FEED_SIG_INVALID` | `FeedHead` verification (§22.4.1) | `FeedHead.sig` fails under `signer`/`pub` chain. | No | FAIL_CLOSED_BLOCK |
| `0x0907` | `ERR_PUB_FEED_ROLLBACK` | Feed head anti-rollback (§22.4.2) | A `FeedHead` presents `seq` strictly below the highest already accepted for that author feed (stale replay / suppression). Equal `seq` is idempotent re-fetch (identical `tip`) or equivocation (`0x0908`), never this error. | No | FAIL_CLOSED_BLOCK — retain the higher tip |
| `0x0908` | `ERR_PUB_FEED_CHAIN_BROKEN` | Feed chain integrity (§22.4.2) | Two `FeedEntry`s at one `seq` with the same `prev`, or a `prev` not resolving to `seq-1` — a fork/rewrite of the author's own log. | No | HALT_ALERT — publish the conflicting entries as evidence |
| `0x0909` | `ERR_PUB_MANIFEST_HASH_MISMATCH` | `PubManifest` integrity (§22.2.2) | The recomputed DS-tagged Merkle root does not equal `PubManifest.id`. | No | DROP_SILENT — do not begin fetch |
| `0x090A` | `ERR_PUB_CHUNK_HASH_MISMATCH` | Public swarm fetch (§22.5.3, §5.5.3) | A fetched plaintext chunk fails to verify against its listed `h_i`. | Yes (re-fetch from another holder) | ROTATE_RETRY |
| `0x090B` | `ERR_PUB_SUPERSEDE_INVALID` | Revision-chain check (§22.3.4) | `supersedes` references an announce whose `pub` differs from this announce's `pub`. | No | FAIL_CLOSED_BLOCK |
| `0x090C` | `ERR_PUB_NOT_SERVED` | Holder serve policy (§22.6.2) | A holder declines to serve a requested public object per its per-holder serve policy. Refusal is a policy decision, not a correctness fault, and never a protocol takedown. | Yes (try another holder) | DENY_POLICY at the holder; ROTATE_RETRY at the fetcher |
| `0x090D` | `ERR_PUB_SERVE_QUOTA` | Serving admission policy (§22.6.3) | A serving node's resource policy (object size / per-publisher storage quota / feed-append rate) is exceeded. A policy deny, never a security gate, never a silent hole. | Yes (after freeing / under a laxer policy) | DENY_POLICY |
