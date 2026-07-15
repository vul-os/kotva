# 3. Naming & Directory (name → key)

DNS names you; your key *is* you; the mesh finds you. This section covers the **stable**
binding: how `abc@def.com` resolves to an identity key, how that binding is made
tamper-evident, and how it degrades safely.

## 3.1 Roles

- **DNS** (we do not run it — registrars/operators do) holds the stable `name → key`
  pointer. It is static and cacheable; it MUST NOT hold location (that is the mesh, §4).
- **Key transparency (KT)** makes the binding *auditable* — since DNS is a trusted third
  party the owner does not control, KT lets a silent key swap be *detected* (§3.5).
- **The mesh DHT** holds the dynamic `key → location` record (§4).

## 3.2 DNS records

For `abc@def.com`, the resolver queries:

```
abc._dmtap.def.com.  IN  TXT  "v=dmtap1; suite=1; ik=<base64url IK>; id=<hash of Identity §1.3>;
                                kt=<KT log URL>; keypkgs=<KeyPackage bundle locator §5.3>"
_dmtap.def.com.      IN  SVCB 1 . ( ... )     ; optional service params, KT anchors
def.com.             IN  MX   ...             ; only if a legacy gateway serves the domain (§7)
```

- `ik` is the identity public key (or a hash of `Identity` that the mesh resolves to the
  full object). `id` pins the current `Identity` version (§1.3).
- Multiple `ik`/suite entries MAY appear during PQ migration (§1.1).
- DNSSEC SHOULD be enabled; it is not sufficient alone (hence KT).

## 3.3 Resolution

```
resolve(name):
  1. DNS TXT/SVCB lookup for name → { iks, id, kt, keypkgs }
  2. (first contact) verify against KT (§3.5): fetch a signed tree head + inclusion proof for
     this identity, and confirm no newer version supersedes it (rollback defense).
  3. fetch full Identity (§1.3) from the mesh by `id`; verify sig chain
  4. PIN (name → iks, id) locally (TOFU); offer out-of-band verification to upgrade the pin
  5. thereafter: route by key via the mesh (§4); DNS is not consulted again unless the
     pinned Identity chain says to (rotation/migration)
```

**Fail-closed on unreachable KT (normative, §12 finding).** If KT is unreachable, partitioned,
or censored at **first contact**, the client MUST NOT silently TOFU-pin an unverified key — it
MUST either refuse to pin (block) or hard-warn and require explicit user acceptance, and MUST
prefer out-of-band verification. Silent downgrade to unverified TOFU is prohibited, because it
enables an attacker to replay an old-but-validly-signed `Identity` (e.g. from before a
legitimate rotation) precisely under the network conditions that make KT unreachable. Once a key
is pinned, later KT unavailability does not block routing (the relationship is already
key-based).

DNS is the **front door, used once**. After first contact the relationship is key-based and
DNS-independent — a later DNS/registrar compromise cannot silently redirect an existing
contact.

## 3.4 Trust on first use (TOFU) + pinning

v0 trust model:

- On first resolution, **pin** `(name → ik, id)`.
- Follow the signed `Identity` hash chain (§1.3–1.6) for rotations and migrations; accept a
  new key only via a valid chain from the pinned key.
- Offer **out-of-band verification** (safety-number / QR comparison of `ik`) to upgrade a
  TOFU pin to a verified pin.
- A key that changes *without* a valid chain MUST raise a security warning, never silently
  update.

**Honest limit:** a MITM at the *very first* contact (before KT is consulted or before OOB
verification) can substitute a key. KT (§3.5) closes this; OOB verification closes it
immediately for high-value contacts.

## 3.5 Key transparency (KT)

**Status: designed-in, v0 ships a minimal form; full CONIKS/Key-Transparency-style logs are
a v1 hardening.**

- The owner's identity events (`Identity`, `RecoveryPolicy`, `KeyRotation`, `MoveRecord`,
  §1) are appended to an **append-only Merkle log**.
- Verifiers obtain **signed tree heads** and **inclusion/consistency proofs**; they gossip
  tree heads to detect **split-view/equivocation** (the log showing different histories to
  different observers).
- The owner's own devices **monitor** the log for the owner's entries and MUST alert on any
  change the owner did not initiate — turning KT into **identity intrusion detection**.
- Logs MAY be federated; a verifier trusts a *set* of logs and requires consistency across
  them. No single log is authoritative.

v0-minimal: a single append-only log with signed heads and inclusion proofs; monitoring and
gossip-based equivocation detection land in v1.

## 3.6 Optional: self-sovereign naming (un-loseable addresses)

DNS names can expire or be seized. For a name that is **un-loseable and un-seizable** as
long as the owner holds `IK`, DMTAP defines a pluggable **name backend** interface. A
conforming backend maps `name → ik` such that only `IK` can update the binding and it
cannot lapse without the owner's action. A name-chain (ENS-style) is one such backend.

This is the **only** place DMTAP admits a blockchain, it is **optional**, and it is confined
to the name layer — nothing else in DMTAP depends on it. Resolution (§3.3) treats a
self-sovereign name identically to a DNS name once `ik` is obtained.

## 3.7 Private lookups

To keep discovery metadata-private, name→key lookups SHOULD be routed **through the mixnet**
(§6), so neither the DNS resolver nor a KT log learns *who* is asking. This replaces
heavier private-contact-discovery schemes for v0; stronger schemes are a v1 option.

## 3.8 Onboarding tiers (day-one setup)

The identity is a **key**; DNS is a generated, auto-managed *projection* of it, never
hand-edited. There are three onboarding tiers. A conformant client SHOULD default to Tier B
and never require a user to author DNS.

### Tier A — no domain (pure DMTAP)

Identity = key; human name from a provider directory or a self-sovereign name backend (§3.6).
**No DNS, no domain.** DMTAP↔DMTAP only (the legacy world cannot resolve the name). Setup:
generate key → claim a directory name → join mesh.

### Tier B — gateway-owned domain (default; zero user DNS)

The user claims `alice@gw.example`; the **gateway operator owns `gw.example` and maintains all
legacy records once, for all users**:

```
gw.example        MX      → gateway
gw.example        TXT     SPF (gateway sending IPs)
sel._domainkey    TXT     gateway DKIM key
_dmarc.gw.example TXT     DMARC policy
+ *@gw.example name→key directory (service or programmatic/wildcard records)
```

The **user touches zero DNS.** DMTAP-native delivery is key-based; legacy interop works
immediately because the shared domain is pre-configured and its IPs are warmed. Per-user
legacy setup = none. This is Gmail-grade onboarding: you get an address, not a domain.

### Tier C — vanity custom domain (auto-configured)

For `alice@yourbrand.com`, the domain's own DNS MUST carry MX/SPF/DKIM/DMARC (DMARC alignment
is defined on the `From:` domain — unavoidable while legacy exists). But it MUST NOT be
hand-edited: the provider auto-publishes the full record set via a **registrar API / the Domain
Connect standard** or by hosting the domain's DNS. The user approves once; records self-update
on key rotation.

### DKIM delegation / DMARC alignment (all legacy-facing tiers)

The domain publishes the **gateway's DKIM public key** at `sel._domainkey.<domain>`; the
gateway signs outbound with `d=<domain>`, so DKIM passes and **DMARC aligns** — Gmail/Outlook
accept it as ordinary authenticated mail. The gateway holds only a delegated *DKIM* key, never
the user's DMTAP identity key (§7.3). Inbound legacy uses the domain's MX → gateway (§7.2).

### What cannot be skipped

Exactly one thing is irreducible: **some** operator must configure **one** legacy domain
properly and **warm its sending IPs** (weeks). This is a one-time, per-domain operator task,
amortized across all users on that domain — never a per-user task.

## 3.9 Names & the naming ladder

The identity is always the **keypair**; a name is a memorable pointer to it. DMTAP defines a
*ladder* of name forms, from zero-authority to human-chosen, all resolving to the same key. A
user MAY hold several at once.

| Form | Example | Authority | How uniqueness is achieved |
|------|---------|-----------|----------------------------|
| **Key-name** (default, zero-authority) | `maple-heron-otter-cabin-river-slate-amber-quill` | **none** | derived from the key — unique *by construction* |
| **Fingerprint** | `k:7f2a9c1b…` (base32) | none | raw key hash |
| **Handle** | `@thisismyname` | thin directory (§3.9.2) | first-come, KT-audited |
| **name@domain** | `me@yourdomain` | DNS | domain namespacing (§3.2, §3.8) |
| **Self-sovereign name** | `thisismyname` on a name-chain | consensus (§3.6) | chain ordering |

### 3.9.1 Key-name (the zero-authority default)

Every identity has a **key-name** computed deterministically from its identity key, requiring
**no directory, no consensus, and no registration** — it exists the instant a key is generated,
and two different keys yield different names by construction (uniqueness is cryptographic, not
adjudicated). This is DMTAP's answer to "a memorable identifier with no authority."

- **Encoding:** `key-name = words( truncate( hash(IK), 80 bits ), wordlist )` — **8 words**
  from a curated **~1024-word, language-agnostic** list (short, pronounceable across major
  languages, no homophones/confusables/offensive collisions), 10 bits/word = **2⁸⁰** address
  space (~10²⁴; a trillion collision-free identities — unreachable for accidental collision).
- A **checksum** is folded into the last word so a mistyped/misheard name fails closed rather
  than resolving to a different key.
- **Proquints** (pronounceable 5-char syllables, 16 bits each; 5 = 2⁸⁰) are an allowed
  language-neutral alternative encoding of the same bits.
- **Honest limit:** 80 bits resists *single-target* grinding but not cheap *multi-target*
  grinding at massive scale; the key-name is a memorable **pointer/verifier**, and the **key
  remains the security boundary** — contacts pin the key on first contact (§3.4) and high-value
  identities display the full fingerprint/safety-number. Users wanting the name itself to be
  adversary-proof-forever MAY use a **12-word (2¹²⁸)** key-name.

### 3.9.2 Handle (human-chosen, thin directory)

A user MAY additionally register a **chosen** handle `@name` in a DMTAP **directory** (a
namespace). Because a *chosen, globally-unique* name requires arbitration (Zooko), the directory
is a thin authority that:

- assigns each handle once (first-come-first-served) with an **anti-squat cost** (a small fee or
  proof-of-work, since consensus/assignment gives uniqueness but not scarcity);
- publishes `handle → key` to a **key-transparency log** (§3.5), so it is *auditable, not
  trusted* — it cannot silently repoint a handle without detection;
- **cannot hijack existing relationships** — contacts route by the pinned key (§1.6), so the
  handle is only an introduction.

Handles are normalized (NFC, lowercase, collapse consecutive dots; dots allowed as cosmetic
separators, e.g. `@this.is.my.name`) and confusables/homoglyphs are reserved. The directory MAY
be a single operator (simplest), a **federated consortium running BFT consensus** (no single
owner of the namespace, no chain), or a **name-chain** (§3.6) — the same ladder, decreasing
authority for increasing cost.

### 3.9.3 Petnames (local)

A user assigns **petnames** locally to contacts ("Mom" → a key). Petnames are human + zero-
authority but *local-scope only* (not global), and never leave the device cluster (they are a
social-graph artifact and MUST be stored encrypted at rest with the mailbox).
