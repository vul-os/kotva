# 0. Overview & Architecture

## 0.1 Goals

DMTAP is a protocol for authenticated, encrypted, metadata-private messaging between
self-sovereign identities, with async (store-and-forward) delivery, no required central
party, and an optional bridge to legacy email. One substrate carries **mail, chat, files**,
and **decentralized identity/login** — the same keypair that receives your mail logs you in
across the web (§13), with no central identity provider.

Concretely, DMTAP MUST provide:

1. **Sovereign identity** — a keypair you own; no account with any provider is required to
   *be* a DMTAP identity. The same identity serves mail, messaging, files, and web login
   (§13).
2. **Reachability without a static IP** — a node behind CGNAT, on a dynamic IP, is
   reachable by its key.
3. **Content, authenticity, and metadata privacy** — messages are end-to-end encrypted and
   signed, and the social graph (who talks to whom, when, how much) is hidden from a global
   passive observer.
4. **Continuity** — you never lose access to your identity (redundant, rotatable recovery)
   and can migrate your human name without losing existing contacts.
5. **Legacy interoperability** — you can exchange mail with the existing SMTP world, and
   existing OIDC apps can consume DMTAP login through a bridge (§13.6).
6. **Scales across device classes** — always-on nodes (Pi/NAS/VPS) hold the mailbox;
   intermittent devices (laptops, phones) participate as thin clients or push-woken nodes;
   gateways scale horizontally (§14).
7. **Future-proofing** — crypto-agility, transport independence, standards reuse; the
   system gets *simpler* as IPv6 spreads and legacy fades.

## 0.2 The two components

DMTAP is, deliberately, only two pieces of software plus DNS (which we do not build):

### The Node (`node/`)

One binary, installed on any box that runs most of the time. It holds **all durable state**
and does **all the real work**:

- Identity: the root keypair, device subkeys, recovery policy (§1).
- Store: the mailbox and file blobs (encrypted MOTEs + content-addressed chunks) (§2, §5).
- Mesh participation: peer discovery (DHT), relaying for others, delivery (§4).
- Mixnet client: onion-wrapping, cover traffic, sealed sender (§4, §6).
- Messaging: MLS groups for 1:1, chat, and file folders; MLS KeyPackages (§5.3).
- Client access: JMAP native, plus IMAP/POP/SMTP-submission compatibility (§8).
- The outbound **retry queue** — durability lives here, not in the middle.

A node MAY additionally run in **relay mode** (help NAT'd peers, if it has a public
address) or **mix mode** (be a mixnet hop). These are capabilities of the same binary, not
separate programs.

### The Gateway (`gateway/`) — optional

The **only** component that speaks SMTP and the **only** one that is not content-blind
(the legacy leg is unavoidably plaintext). It:

- receives inbound legacy mail (acts as MX), wraps it into a MOTE, attests it, and delivers
  into the mesh; returns SMTP `4xx` if the recipient is offline so the *sending* server
  retries;
- sends outbound legacy mail, DKIM-signing as the user's domain via delegated selectors;
- carries the operational weight the system cannot avoid: **IP reputation**.

A node without legacy correspondents never invokes a gateway. At full adoption, the
gateway is unnecessary. The gateway MAY be the node binary run in `--gateway` mode by an
operator with a reputable IP and a domain.

### DNS (not built here)

The naming substrate that maps a human name to a key. We publish and read records; we do
not run DNS. See §3. DNS holds the **stable** binding (name → key); the mesh holds the
**dynamic** binding (key → current location).

## 0.3 The three layers of indirection

The core trick that frees an address from any IP:

```
abc@def.com  ──DNS/§3──▶  public key   ──mesh/§4──▶  current location
   NAME                    IDENTITY                    (IP/relay/mix path)
   (human, stable)         (durable, portable)         (ephemeral, self-updated)
```

- **Name → key** is stable and lives in DNS (+ key transparency). It changes only when you
  migrate names.
- **Key → location** is dynamic and lives in the mesh (a signed, TTL'd DHT record the node
  republishes as its address changes).
- The **key is the identity.** Existing contacts route by key via the mesh and never need
  DNS again after first contact; a lost domain is a change of *name*, not of identity (§1.6).

## 0.4 Message-flow summary

### DMTAP → DMTAP (the common path)

```
1. resolve  abc@def.com → recipient key K            (§3; cached/pinned after first contact)
2. fetch    K's KeyPackages + current location           (§4 DHT, §5.3 KeyPackages)
3. build    a MOTE: sealed-sender, MLS/HPKE-encrypted to K, signed  (§2, §5, §6)
4. send     through the mixnet (private tier) or direct (fast tier) (§4, §6)
5. recipient node receives, verifies, decrypts, stores; acks        (§2, §4)
   (sender's node retries until ack — durability at the edge)
```

No gateway, no SMTP, no plaintext outside the endpoints.

### Legacy → DMTAP (inbound)

```
gmail ──SMTP──▶ Gateway ──wrap+attest+encrypt to K──▶ mesh ──▶ recipient node
                (if node offline: return SMTP 4xx; Gmail retries)
```

### DMTAP → Legacy (outbound)

```
node ──MOTE(legacy)──▶ Gateway ──SMTP + delegated DKIM──▶ gmail
                       (node retries on failure)
```

## 0.5 Where state lives

| State | Location | Notes |
|-------|----------|-------|
| Keys, mailbox, files, retry queue | **Node** (the edge) | All durable state |
| Name → key | **DNS** + key-transparency log | Stable; small |
| Key → location | **Mesh DHT** | Dynamic; signed; TTL'd; self-republished |
| In-flight ciphertext | **Mixnet / relay** | Held only until delivered; content-blind |
| Legacy reputation | **Gateway** | The only non-trivial operational cost |

The middle (mesh, mixnet, gateway) holds **no durable user data**. Durability is always
punted to an edge: the sender's node retries; inbound legacy leans on the sending server's
SMTP retry.

## 0.6 Privacy posture (summary; full model in §6)

- **Content & authenticity:** end-to-end encrypted (MLS/HPKE) and signed. Always.
- **Sender metadata:** hidden via **sealed sender** — intermediaries never learn the sender.
- **Social graph & timing:** hidden via a **mixnet** (onion routing + mixing delays) plus
  **cover traffic** and **size padding**. Email's asynchrony is what makes full-strength
  mixing affordable.
- **Recipient retrieval:** the **always-on node receives by push** through the mixnet, so
  there is *no store-and-poll step to hide* — the hardest metadata problem is dissolved by
  architecture, not by expensive PIR.
- **Discovery:** name→key lookups are routed *through* the mixnet, so the directory does not
  learn who is looking up whom.
- **Privacy tiers:** messages may choose `private` (full mixnet, minutes of latency) or
  `fast` (direct/low-hop, seconds, less metadata protection). Default is `private`; bulk
  file transfer uses `fast` for the payload and `private` for its control message.

**Honest boundary:** DMTAP targets a **global passive adversary**. Perfect resistance to a
global *active* adversary with unlimited resources is not claimed; see §6.

## 0.7 Non-goals

- Real-time voice/video (separate WebRTC/SFU architecture).
- Blockchain/consensus (except optional self-sovereign naming in §3).
- Server-side search or server-side spam ML (search is on-device; anti-abuse is §9).
