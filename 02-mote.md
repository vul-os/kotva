# 2. The MOTE Object

A **MOTE** (the atomic unit of DMTAP) is a signed, encrypted, content-addressed message
object. Mail, chat messages, file-share announcements, group events, and identity
announcements are all MOTEs — one format, rendered differently by clients (§5, §8).

## 2.1 Layered structure

A MOTE has three nested layers, each serving a distinct purpose:

```mermaid
flowchart TD
  subgraph OUTER["Outer — mixnet / sealed sender · §4, §6"]
    direction TB
    O["routing to the recipient node<br/><i>NO sender identity in clear</i>"]
    subgraph ENV["Envelope — signed, per-recipient · §2.2"]
      direction TB
      E["authenticity + integrity<br/><i>content-addressed id</i>"]
      subgraph PAY["Payload — MLS/HPKE ciphertext · §2.4"]
        P["the actual content:<br/>headers + body + attachments"]
      end
    end
  end
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
  epoch:    ?bytes,         // MLS group epoch / group context ref, if group (§5)
  ts:       u64,            // sender timestamp (ms epoch)
  kind:     u8,             // message kind (§2.3)
  keypkg:   ?KeyPackageRef, // present iff this initiates an MLS session (async join, §5.3)
  challenge: ?ChallengeResponse, // anti-abuse proof for cold senders (§2.2b, §9) —
                                 //   verifiable WITHOUT decrypting `ciphertext`
  ciphertext: bytes,        // MLS or HPKE sealed Payload (§2.4)
  sender_key: bytes,        // EPHEMERAL per-message PUBLIC key; the verification key for
                            //   `sender_sig`. Fresh per message ⇒ unlinkable, reveals no identity.
  sender_sig: bytes,        // detached sig by `sender_key`'s secret over the §18.9.1
                            //   preimage; gates abuse, reveals no identity
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
  from a secret established at first contact (`epoch_day` is the **day-counter epoch**, §0.8 —
  distinct from an MLS group epoch or a mix-key epoch), which the recipient's node recognizes but
  which is **unlinkable across time and across observers** to the recipient's persistent key.

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
0x0b  deniable        optional deniable 1:1 transport frame (§5.2.1); real content kind rides inside
0x40–0x7f  reserved for extensions (§10)
```

`0x40 pub_announce` is allocated from the extension range to the DMTAP-PUB extension (§22): a
public signed announcement, plaintext, openly signed by the publisher identity (no sealed
sender). Unlike the kinds above, a `pub_announce` is a **bare signed object, not a MOTE** — it
never rides inside an `Envelope`; it is fetched by content address (§22.3).

**Unknown kinds (normative).** A recipient MUST NOT `ack` a kind it does not implement.
Unknown kinds — unassigned, or assigned but unimplemented — are **ignored** (not surfaced, not
acked, MAY be discarded or held), never rejected as malformed (§21.16, §2.7 unknown-kind gate).

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
  provenance: ?[+ GatewayAttestation],  // sealed gateway-attestation chain (§7.8, §18.3.11);
                            //   present iff gateway-touched, absent ⇒ provably pure-mesh
}

Headers {
  thread:    ?bytes,        // stable thread/conversation id
  subject:   ?tstr,         // mail only
  mime:      ?tstr,         // content type of `body`
  cc:        [* bytes],     // additional recipient keys (fan-out is per-recipient MOTEs)
  ext:       { * tstr => ext-value },  // extension headers (§10); typed, NOT `any` — see §18.3.6
  sensitive: ?bool,         // key 6: no-persist / ephemeral-view (§6.7); §18.3.6 is authoritative
}

Body = tstr / bytes         // text or opaque MIME
```

Only the recipient (or group members) can decrypt `Payload`, so **all sender identity,
subject, recipients, threading, and content are hidden from the network** — this is what
sealed sender + payload encryption buys.

## 2.5 Attachments and files

Small attachments MAY be inlined into `Payload.attach`. Larger files MUST be referenced by a
content-addressed **manifest**; **normal-tier** chunks (≤ 4 MiB) transfer **via the mixnet**
like messages, **large-tier** chunks via the fast/onion bulk path (§4.5). This is what makes
DMTAP a file-share of arbitrary size (no protocol cap).

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
transfer. Only the manifest + `key` travel in the (private) MOTE; the chunks travel per the
size tier below — normal-tier via the mixnet, large-tier direct (§4.5). See §5.5 for the full
file model.

**Metadata-privacy size tiers (normative threshold, reconciles §2.5 / §4.5 / §6.5).** These are
**metadata-privacy tiers** — they fix which *path* the bytes take and what an observer can
learn; the **durability tiers** of §5.5.1 (who holds the bytes, and for how long) are an
orthogonal axis. A file is handled by one of three paths, by size:

| Tier | Size | Path | Metadata privacy |
|------|------|------|------------------|
| **inline** | ≤ v0 **48 KiB** of content — the padded MOTE then rides the top bucket rung, 64 KiB = 32 Sphinx cells (§4.4.1/§16.3); the ≈ 12 kB difference is the PQ envelope | in `Attachment.inline`, inside the MOTE | full (rides the message's tier) |
| **normal** | > inline, ≤ 4 chunks (v0: ≤ 4 MiB) | manifest in MOTE; **chunks also routed via the mixnet** | full (like messages, §6.5) |
| **large** | > normal | manifest in MOTE; **chunks via the fast/onion bulk path** (§4.5) | weaker — Tor-class (§6.5) |

The v0 numeric thresholds (48 KiB / 4 MiB) are parameters (§16.4) and MAY be tuned;
the three-tier model is normative. This removes the earlier binary small/large ambiguity.
**Note on "inline" and the mixnet cell:** an inline payload is **not** a single mix packet — the
Sphinx cell is 2 KiB (§16.3), so a padded inline MOTE is a **whole number of 2 KiB cells** on the
**bucket ladder** {16, 64} KiB (§4.4.1) — i.e. 8 or 32 cells. The inline tier's ceiling is the
**top rung** (64 KiB, 32 cells), not one packet, and the ≤ 48 KiB content cap is that rung less
the envelope; only ladder sizes appear on the wire, so size still leaks nothing.
Note there is **no 2 KiB and no 8 KiB rung**: a conformant PQ envelope (suite `0x02`, §1.1)
carries *two* signatures and *two* public keys plus a KEM ciphertext, and so exceeds **11.9 kB**
before any body at all. §4.4.1 states that arithmetic in bytes; the floor is whatever it forces,
and is 16 KiB in v0.

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
- **Deduplication.** A recipient re-acks only an `id` it has **previously acked** (a stored,
  inbox-delivered MOTE): such a redelivery is acked immediately without re-processing. An `id`
  held **only in the deferred requests area** (§2.7a) is NOT acked on redelivery — re-acking it
  would leak, to an unproven sender probing with a duplicate, exactly the existence confirmation
  the requests-area no-ack rule withholds.
- **Retry deadline when `expires` is absent.** When `expires` is absent, the retry deadline is
  the §16.1 default maximum retry lifetime; retry is always bounded.
- **Ordering.** MOTEs are not globally ordered; `ts` + `refs` + (for groups) the MLS group
  `epoch` provide causal/threading order. Clients MUST tolerate out-of-order and duplicate delivery.

## 2.7 Validation (recipient MUST)

Validation is ordered **cheapest-and-anonymous first**, so a flood of cold junk is rejected
*before* any expensive asymmetric decryption (a decryption-DoS defense). On receipt a node MUST,
in order:

1. Reject unknown `v`/`suite` (fail closed).
2. Verify `id` matches the content address of `ciphertext` (§2.2); drop on mismatch.
3. Verify `sender_sig` over the **§18.9.1 preimage** (DS-tagged; `to` as deterministic CBOR,
   `ts` as u64 big-endian, `kind` as one byte, absent `challenge` as `0xf6`) under
   **`sender_key`** — the ephemeral per-message public key carried in the same envelope (cheap;
   no decryption). Drop on failure.
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
7. Decrypt `ciphertext` (MLS epoch key or HPKE to recipient key); drop on failure. **Deniable
   fork (`kind = 0x0b`, §5.2.1):** if the envelope `kind` is `0x0b`, the `ciphertext` is a
   `DeniableFrame` (§18.3.9), **not** an MLS/HPKE-sealed `Payload`. Decryption is the **Double
   Ratchet** decrypt (a `DeniableInit` first establishes the X3DH/PQXDH session; a `DeniableMessage`
   advances the ratchet); the plaintext is a `DeniablePayload` (§18.3.10). A decrypt/ratchet
   failure is `ERR_DENIABLE_RATCHET_AUTH_FAILED` (`0x040D`, drop/hold-for-resync), never an
   MLS/HPKE decrypt attempt.
8. **Authenticate the sender and bind the envelope context**, as ordered sub-steps:
   - **(a) Payload signature (normal path).** Verify `Payload.sig` under `Payload.from`; **on
     failure, discard silently and do not `ack`** (fail closed, matching steps 1–3).
   - **(b) Envelope-context binding.** The `Payload.sig` preimage **binds the envelope's `kind`,
     `ts`, and `to`** (§18.9.2): recompute it using the received `Envelope`'s `kind`/`ts`/`to`
     and **reject any MOTE whose envelope `kind`/`ts`/`to` differ from the signed context**
     (`ERR_ENVELOPE_CONTEXT_MISMATCH`, `0x0211`). This stops a re-emitter from re-minting the
     anyone-can-mint `sender_sig` (step 3) over an altered `kind`/`ts`/`to` — rewriting the
     timestamp/causal order, or relabeling `kind` to change rendering or force a silent
     decrypt-fail.
   - **(c) Pin check.** Otherwise verify `from` matches the pinned identity for a known contact,
     or TOFU-pin on first contact (§3.4). For a cold sender whose `from` is now revealed,
     re-apply block/allow lists.
   - **(d) Deniable fork (`kind = 0x0b`).** A `DeniablePayload` carries **no** `sig` — the
     substitute authenticator is the **Double-Ratchet AEAD tag** (the shared-key MAC) already
     checked at step 7. The recipient verifies that tag *instead of* `Payload.sig` (sub-steps
     (a)–(b); the deniable path binds the envelope context inside the ratchet AD instead,
     §18.9.10), binds `DeniablePayload.from` to the X3DH-authenticated `IK` (matching the pinned
     identity, §3.4), and **MUST reject any `DeniablePayload` that carries a signature field**
     (`ERR_DENIABLE_SIGNATURE_PRESENT`, `0x040F`) — a present signature would defeat the mode.
   - **(e) Suite-ratchet check (last — requires the now-revealed sender identity).** Verify
     `Envelope.suite` is **not below** this contact's pinned suite high-water-mark (§1.3); a
     below-water-mark suite is a downgrade attempt → reject to the requests area with a security
     warning (`ERR_SUITE_DOWNGRADE`, §21.4), never accept. This check MUST occur here, not at
     step 1, because sealed sender hides the sender until decryption; a recipient that has
     retired a suite for *itself* MAY additionally reject that `Envelope.suite` at step 1 as its
     own floor.

   **Unknown-kind gate (between steps 8 and 9, normative).** If `Envelope.kind` is unassigned,
   or assigned but not implemented by this node, the node MUST NOT `ack` and MUST NOT surface
   the MOTE; it MAY discard it or hold it (e.g. pending an upgrade). A node MUST NOT
   store-and-ack a kind it cannot validate (§21.16 forward-compatibility rule, §10.1) — an ack
   asserts validated delivery, which is impossible for semantics the node cannot check.
9. Apply `expires`, `refs`, `kind` semantics; store; `ack`.

Known contacts MAY skip step 6 (they are pre-authorized). Only known-contact MOTEs reach
decryption on the fast path; unknown senders must pass the anonymous abuse gate first.

**Dedup ordering (normative).** Deduplication by `id` (§2.6) runs **after** classification
(step 5), never before: a duplicate of a previously **acked** `id` is then re-acked immediately
without further processing, while a duplicate of an `id` held only in the deferred requests area
follows §2.7a unchanged — held, not acked. Running dedup earlier would let a duplicate probe
short-circuit the abuse gate and turn the ack itself into an existence oracle.

### 2.7a Outcome of a failed/absent challenge (normative)

To reconcile §2.7 and §9.2, the disposition is by *degree*:

- **Invalid or forged** `challenge`, or failed `sender_sig`/`id` → **discard silently**, no
  user-visible effect, do not `ack` (except a duplicate of a previously **acked** `id`, which is
  re-acked per §2.6; a duplicate of an `id` held only in the requests area is never acked).
- **Absent or below policy threshold** (a cold sender with no/weak proof) → **defer to a
  "requests" area** (not the inbox), rate-limited, never silently dropped and never surfaced as
  a normal message, and **not `ack`ed** (a deferred cold MOTE is durably held but no receipt is
  sent — acking would confirm the recipient's existence to an unproven sender and falsely signal
  *delivered*; the sender's own retry reaches `EXPIRED`, §16.1). Deferred MOTEs are held for the
  **requests-area retention period** (30 days, §16.5), then auto-cleaned. The user MAY promote the
  sender (which pins them as a contact). Ack is owed **only** for inbox delivery (§19.3.1 step 9).

Implementations MUST NOT deliver an unproven cold MOTE to the inbox, and MUST NOT silently
discard a well-formed-but-under-threshold one without the requests-area affordance.

## 2.8 Why this shape

- **Content-addressed** → dedup, integrity, caching, and file chunking all fall out of `id`.
- **Three layers** → clean separation of *routing privacy* (outer), *authenticity*
  (envelope), and *confidentiality* (payload).
- **Sealed sender** → the network sees ciphertext to an opaque destination and nothing else.
- **One object, many kinds** → mail, chat, files, groups, and identity share the format, so
  the transport, store, and crypto are built once (§5).
