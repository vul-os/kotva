# 2. The MOTE Object

A **MOTE** (the atomic unit of DMTAP) is a signed, encrypted, content-addressed message
object. Mail, chat messages, file-share announcements, group events, and identity
announcements are all MOTEs — one format, rendered differently by clients (§5, §8).

## 2.1 Layered structure

A MOTE has three nested layers, each serving a distinct purpose:

```
┌─ Outer (mixnet / sealed sender) ─────────────────────────────┐
│  routing to the recipient node; NO sender identity in clear   │  §4, §6
│ ┌─ Envelope (signed, per-recipient) ───────────────────────┐  │
│ │  authenticity + integrity; content-addressed id           │  §2.2
│ │ ┌─ Payload (MLS/HPKE ciphertext) ──────────────────────┐  │  │
│ │ │  the actual content: headers + body + attachments     │  │  §2.4
│ │ └───────────────────────────────────────────────────────┘  │  │
│ └───────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────┘
```

The **outer** layer is what mix nodes and relays see: an onion-wrapped, constant-length
(padded) packet with no clear-text sender (sealed sender, §6). The **envelope** provides
authenticity to the recipient. The **payload** is the end-to-end-encrypted content.

## 2.2 Envelope (CBOR)

```
Envelope {
  v:        u8,             // format version (0)
  suite:    u8,             // algorithm suite (§1.1)
  id:       bytes,          // content address of `ciphertext` (§2.2, hash-agile)
  to:       DeliveryTag,    // routing target: recipient key, group id, or blinded tag (§2.2a)
  epoch:    ?bytes,         // MLS epoch / group context ref, if group (§5)
  ts:       u64,            // sender timestamp (ms epoch)
  kind:     u8,             // message kind (§2.3)
  keypkg:   ?KeyPackageRef, // present iff this initiates an MLS session (async join, §5.3)
  challenge: ?ChallengeResponse, // anti-abuse proof for cold senders (§2.2b, §9) —
                                 //   verifiable WITHOUT decrypting `ciphertext`
  ciphertext: bytes,        // MLS or HPKE sealed Payload (§2.4)
  sender_key: bytes,        // EPHEMERAL per-message PUBLIC key; the verification key for
                            //   `sender_sig`. Fresh per message ⇒ unlinkable, reveals no identity.
  sender_sig: bytes,        // detached sig by `sender_key`'s secret over
                            //   (id‖to‖ts‖kind‖challenge); gates abuse, reveals no identity
}
```

- **`id`** — content address = a 1-byte **hash-algorithm prefix** (multihash-style, for
  agility) followed by the digest. v0 default is **BLAKE3-256** (128-bit collision resistance,
  equal to SHA-256; fast Merkle/XOF structure ideal for chunking). BLAKE3 is cryptographically
  sound but **not FIPS/IETF-standardized** (the IETF draft expired); the agility prefix lets an
  implementation migrate to SHA-256/SHA-3 where compliance requires it without changing the
  address format. Pin the exact BLAKE3 mode + 256-bit output. Content addressing gives
  deduplication, integrity, and cacheability for free; identical ciphertext shares an `id`.
- **Signature placement.** For sealed-sender messages the *authenticating* signature and the
  sender's identity live **inside** the encrypted payload (`Payload.from`, `Payload.sig`),
  so intermediaries never see who signed. `sender_sig` in the envelope is a
  detached signature by an *ephemeral* per-message key (unlinkable) used only for
  spam/abuse gating at the recipient (§9); it does not reveal identity. The matching
  **public** key travels in `sender_key` so the recipient can verify `sender_sig`
  *before decrypting* (§2.7 step 3) — there is no persistent key to look up, and a fresh
  keypair per message means `sender_key` is itself unlinkable. The `challenge` proof
  (§9) MUST be bound to `sender_key` (an ARC token is issued to it; a PoW/stamp commits to
  it — §9.4) so that a captured proof cannot be re-signed onto another envelope with a
  different ephemeral key.
- **`epoch`** ties a message to an MLS group epoch so the recipient selects the right key.

### 2.2a Delivery tag (`to`) and recipient blinding

`to` is a **`DeliveryTag`**, one of:

- the recipient's **identity key** (default, simplest); or
- a **group id** (for MLS group messages, §5); or
- a **blinded delivery tag** — a per-contact value `BT = HKDF(shared_secret, epoch_day)` derived
  from a secret established at first contact, which the recipient's node recognizes but which is
  **unlinkable across time and across observers** to the recipient's persistent key.

Blinded tags are RECOMMENDED for the `private` tier. **Honest limit (reconciled with §6.4):**
even a blinded tag does not hide *that a packet was delivered to a particular node* from the
final mix / an observer of the recipient's link — an always-on node has a stable network
presence (§6.4). Blinding removes the *persistent-key* linkage in the envelope; it does not
remove last-hop delivery observability, which §6.4 addresses with node/identity decoupling and
recipient-side cover traffic. Implementations MUST NOT present blinded tags as full recipient
anonymity.

### 2.2b Anti-abuse challenge (`challenge`)

`challenge` is an optional `ChallengeResponse` carrying the sender's proof for cold contact
(§9): an **ARC token**, a **proof-of-work solution**, a **postage stamp**, or a **vouch**. It is
placed in the *envelope* (not the payload) so the recipient can evaluate abuse policy **without
decrypting** — see the validation order in §2.7. Known contacts omit it (their MOTEs are
accepted on the fast path). See §9 for the grammar, issuer-trust rules, and each proof type.

## 2.3 Message kinds

```
0x00  mail            long-form message (email semantics)
0x01  chat            short message (chat semantics)
0x02  reaction        emoji/ack on a referenced MOTE
0x03  edit            supersede a referenced MOTE
0x04  redact          request deletion of a referenced MOTE
0x05  file_offer      manifest + key for a content-addressed file (§5.5)
0x06  group_event     MLS handshake / membership change (§5.3)
0x07  receipt         delivery/read receipt (opt-in; §6)
0x08  presence        ephemeral presence/typing (opt-in, off by default; §6)
0x09  identity        Identity/Move/RecoveryPolicy announcement (§1)
0x0a  system          protocol control (capability negotiation, §10)
0x40–0x7f  reserved for extensions (§10)
```

Kinds `mail` and `chat` differ only in default client rendering and default privacy tier
(§6): `mail` defaults to the `private` tier, `chat` MAY use the `fast` tier when both
parties are online. Both are the same object over the same transport.

## 2.4 Payload (CBOR, encrypted)

The plaintext that is sealed into `Envelope.ciphertext`:

```
Payload {
  from:     bytes,          // sender identity key (IK) — revealed only to the recipient
  sig:      bytes,          // IK (or device key) over the canonical payload hash
  headers:  Headers,
  body:     Body,
  refs:     [* bytes],      // ids of MOTEs this replies to / references (threading)
  attach:   [* Attachment], // §2.5
  expires:  ?u64,           // requested expiry (client-enforced deletion)
  fs_ratchet: ?bytes,       // forward-secrecy ratchet material (§5.2)
}

Headers {
  thread:   ?bytes,         // stable thread/conversation id
  subject:  ?tstr,          // mail only
  mime:     ?tstr,          // content type of `body`
  cc:       [* bytes],      // additional recipient keys (fan-out is per-recipient MOTEs)
  ext:      { * tstr => any },   // extension headers (§10)
}

Body = tstr / bytes         // text or opaque MIME
```

Only the recipient (or group members) can decrypt `Payload`, so **all sender identity,
subject, recipients, threading, and content are hidden from the network** — this is what
sealed sender + payload encryption buys.

## 2.5 Attachments and files

Small attachments MAY be inlined into `Payload.attach`. Larger files MUST be referenced by
a content-addressed **manifest** and transferred out-of-band (direct/fast tier, §4.5),
which is what makes DMTAP a file-share of arbitrary size (no protocol cap).

```
Attachment {
  name:     tstr,
  mime:     tstr,
  size:     u64,
  inline:   ?bytes,         // present iff small
  manifest: ?ManifestRef,   // present iff large (§5.5)
  key:      bytes,          // per-file content key (recipient decrypts chunks)
}

ManifestRef { id: bytes, size: u64, chunks: u32 }   // BLAKE3 Merkle-DAG root (§5.5)
```

The manifest lists chunk hashes (a BLAKE3 Merkle DAG). Chunks are fixed-size, individually
encrypted and content-addressed, enabling **resumable, parallel, swarmed, deduplicated**
transfer. Only the manifest + `key` travel in the (private) MOTE; the bulk chunks travel
direct (§4.5). See §5.5 for the full file model.

**Size tiers (normative threshold, reconciles §2.5 / §4.5 / §6.5).** A file is handled by one of
three paths, by size:

| Tier | Size | Path | Metadata privacy |
|------|------|------|------------------|
| **inline** | ≤ 1 padded packet (v0: ≤ 64 KiB after padding) | in `Attachment.inline`, inside the MOTE | full (rides the message's tier) |
| **normal** | > inline, ≤ 4 chunks (v0: ≤ 4 MiB) | manifest in MOTE; **chunks also routed via the mixnet** | full (like messages, §6.5) |
| **large** | > normal | manifest in MOTE; **chunks via the fast/onion bulk path** (§4.5) | weaker — Tor-class (§6.5) |

The v0 numeric thresholds (64 KiB / 4 MiB) are parameters (§16.4) and MAY be tuned;
the three-tier model is normative. This removes the earlier binary small/large ambiguity.

## 2.6 Delivery semantics

```
deliver(outer_mote)   → recipient node receives, unwraps outer, verifies envelope,
                         decrypts payload, stores, and returns:
ack(id)               → recipient confirms receipt of MOTE `id`.
```

- **`ack(id)` transport.** An `ack` is a **small signed control MOTE** (kind `system`, §2.3) —
  or, on a live direct/`fast` connection, a transport-level ack over the same channel — carrying
  `id` and signed by the recipient's device key, so the sender's retry queue (§4.7) can
  authenticate the confirmation. Acks are not themselves acked (no ack storm).
- **Durability = the sender's node retries** until `ack`, with exponential backoff and an
  `expires`-bounded deadline. The mixnet/relay holds nothing durably.
- **Deduplication.** A recipient that already holds `id` acks immediately without
  re-processing.
- **Ordering.** MOTEs are not globally ordered; `ts` + `refs` + (for groups) MLS `epoch`
  provide causal/threading order. Clients MUST tolerate out-of-order and duplicate delivery.

## 2.7 Validation (recipient MUST)

Validation is ordered **cheapest-and-anonymous first**, so a flood of cold junk is rejected
*before* any expensive asymmetric decryption (a decryption-DoS defense). On receipt a node MUST,
in order:

1. Reject unknown `v`/`suite` (fail closed).
2. Verify `id` matches the content address of `ciphertext` (§2.2); drop on mismatch.
3. Verify `sender_sig` over `(id‖to‖ts‖kind‖challenge)` under **`sender_key`** — the ephemeral
   per-message public key carried in the same envelope (cheap; no decryption). Drop on failure.
   `sender_key` is trusted only as the abuse-gate key for *this* envelope; it asserts no identity
   (identity is authenticated later, inside `ciphertext`, at step 8). The `challenge` at step 6
   MUST be bound to `sender_key` (§9.4), so this step also fixes which ephemeral key the abuse
   proof was minted for.
4. **Resolve `to`** to this node (or a group it belongs to). If `to` does not resolve, drop.
5. **Classify the sender** by `to`/pinning state: a **known contact** (fast path) vs an
   **unknown/cold sender**.
6. **For cold senders, enforce anti-abuse policy (§9) NOW, before decryption**, using the
   `challenge` field (ARC token / PoW / stamp / vouch) — all checkable without decrypting.
   Reject or defer per §9.2 / §2.7a if the challenge is absent or insufficient.
7. Decrypt `ciphertext` (MLS epoch key or HPKE to recipient key); drop on failure.
8. Verify `Payload.sig` under `Payload.from`; **on failure, discard silently and do not `ack`**
   (fail closed, matching steps 1–3). Otherwise verify `from` matches the pinned identity for a
   known contact, or TOFU-pin on first contact (§3.4). For a cold sender whose `from` is now
   revealed, re-apply block/allow lists.
9. Apply `expires`, `refs`, `kind` semantics; store; `ack`.

Known contacts MAY skip step 6 (they are pre-authorized). Only known-contact MOTEs reach
decryption on the fast path; unknown senders must pass the anonymous abuse gate first.

### 2.7a Outcome of a failed/absent challenge (normative)

To reconcile §2.7 and §9.2, the disposition is by *degree*:

- **Invalid or forged** `challenge`, or failed `sender_sig`/`id` → **discard silently**, no
  user-visible effect, do not `ack` (except a duplicate `id`, which is acked).
- **Absent or below policy threshold** (a cold sender with no/weak proof) → **defer to a
  "requests" area** (not the inbox), rate-limited, never silently dropped and never surfaced as
  a normal message. Deferred MOTEs are held for the **requests-area retention period** (30 days,
  §16.5), then auto-cleaned. The user MAY promote the sender (which pins them as a contact).

Implementations MUST NOT deliver an unproven cold MOTE to the inbox, and MUST NOT silently
discard a well-formed-but-under-threshold one without the requests-area affordance.

## 2.8 Why this shape

- **Content-addressed** → dedup, integrity, caching, and file chunking all fall out of `id`.
- **Three layers** → clean separation of *routing privacy* (outer), *authenticity*
  (envelope), and *confidentiality* (payload).
- **Sealed sender** → the network sees ciphertext to an opaque destination and nothing else.
- **One object, many kinds** → mail, chat, files, groups, and identity share the format, so
  the transport, store, and crypto are built once (§5).
