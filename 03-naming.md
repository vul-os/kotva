# 3. Naming & Directory (name → key)

Your **key is your identity and the only root of trust**; a **`name@domain`** is the everyday,
human-facing pointer to it; the mesh finds you by key. This section covers the **stable**
binding: how `abc@def.com` resolves to an identity key, how that binding is made
tamper-evident, and how it degrades safely.

**The layering (read this first).** Three layers, never conflated:

- **The key is identity and proof.** Authenticity is *always* the key (`IK`, §1.2) — nothing
  else. A name is only a pointer to it.
- **DNS (and any name backend) is *discovery*, never proof.** It tells you which key *claims* a
  name; it does not attest it. A compromised registrar can lie about the pointer, so the pointer
  is never trusted on its own.
- **Key transparency (KT) makes the `name → key` binding tamper-evident**, so a silent key swap
  by DNS/registrar/directory is *detectable* (§3.5).
- **Pinning (TOFU) makes discovery a one-time event.** After first contact you route by the
  pinned key via the mesh (§4); DNS is not consulted again unless the signed identity chain says
  to. A later DNS/registrar compromise cannot redirect an existing relationship.

**Stable anchor, rotatable keys (the real future-proofing).** The human name binds to a
**stable identity anchor — the root identity key `IK` (§1.2)** — *not* to a rotatable
operational key. Day-to-day signing/device keys rotate *under* the identity without changing the
name; even `IK` itself can rotate, and the suite can migrate to post-quantum, without the name
changing — bridged by the signed hash chain (§1.5) and, for a change of the anchor's name, by
aliases + a signed `MoveRecord` (§1.6). So a `name@domain` **survives ordinary key rotation and
PQ migration**: the `name → key` indirection is exactly what lets the key underneath change while
the address people type stays the same. `IK` rotation is the rare migration event, not the
common case.

## 3.1 Roles (who holds what)

- **The key (`IK`)** is the identity and the sole trust root. Everything below only *points* to
  it; nothing below can prove authenticity on its own.
- **DNS** (we do not run it — registrars/operators do) holds the stable `name → key` *pointer*.
  It is discovery: static and cacheable. It MUST NOT hold location (that is the mesh, §4), and it
  is **never** the proof — KT + pinning are.
- **Key transparency (KT)** makes the pointer *auditable* — since DNS is a trusted third party
  the owner does not control, KT lets a silent key swap be *detected* (§3.5).
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
- Offer **out-of-band verification** (safety-number / QR comparison of `ik`, §3.4.1) to upgrade a
  TOFU pin to a verified pin.
- A key that changes *without* a valid chain MUST raise a security warning, never silently
  update.

**Honest limit:** a MITM at the *very first* contact (before KT is consulted or before OOB
verification) can substitute a key. KT (§3.5) closes this; OOB verification closes it
immediately for high-value contacts.

### 3.4.1 Safety numbers (out-of-band key verification)

Out-of-band verification compares the **key**, not the name — it is the strongest trust upgrade
and the one thing that closes a first-contact MITM immediately.

- A **safety number** (Signal-style) is a deterministic function of the parties' `IK`s (the
  fingerprint of the identity keys). Two contacts confirm they see the same value out-of-band —
  in person, over a trusted channel, or by scanning a **QR code**.
- **This is verification, not an address.** A safety number / fingerprint is *never* used to
  route or reach someone; it only confirms that the key you pinned is the key you meant. It does
  not appear in `Identity.names` and cannot be typed at to send mail.
- **Word rendering (optional).** Because digit strings are error-prone to compare aloud, a
  fingerprint MAY be rendered as a **word sequence** for easier human comparison:
  `words(fingerprint, wordlist)` over a curated **~1024-word, language-agnostic** list (short,
  pronounceable across major languages, no homophones/confusables/offensive collisions), 10
  bits/word, with a folded **checksum** so a misheard word fails closed rather than comparing as
  a different key. **Proquints** (pronounceable 5-char syllables, 16 bits each) are an allowed
  language-neutral alternative encoding of the same bits.
- The word encoding exists **only** for this verification role — a comparison aid for confirming
  a key. It is deliberately **not** an address: DMTAP does not name people by their key digits.
  Because the safety number is taken over the full identity key, it carries the key's full
  strength; there is no separate truncated-name address to grind.

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
(a provider-issued `name@domain`) and never require a user to author DNS.

### Tier A — no domain (pure DMTAP)

Identity = key; the human name is a provider directory name or a self-sovereign name backend
(§3.6) that resolves directly to the key. **No DNS, no domain.** DMTAP↔DMTAP only (the legacy
world cannot resolve the name). Setup: generate key → claim a directory name → join mesh. Most
users instead pick Tier B so their address also works for legacy email.

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
legacy setup = none. This is Gmail-grade onboarding: you get an address, not a domain — a
provider-issued `you@gw.example` that is a full, first-class `name@domain` (§3.9.1).

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

## 3.9 Names (one identity, many pointers)

The identity is always the **key**; every name is a memorable pointer to it, and one identity
MAY hold several names at once — all resolving to the same key. DMTAP has **one recommended,
headline form** and a few optional extras. This is deliberately *not* a ladder of equals.

**Zooko's triangle, stated plainly.** A name cannot be simultaneously *human-meaningful*,
*global*, and *authority-free*. DMTAP's choice: names are human-meaningful and — through DNS
federation — global, and it pays for that with an authority (DNS/registrar). But it
**neutralizes** that authority by keeping the key the sole trust root, KT-auditing the binding
(§3.5), and pinning on first use (§3.4): the authority can help you *discover* a key, it can
never *forge* one undetected. The alternative corner (a flat authority-free name derived from
the key) is not offered as an address — it is unwieldy, and it would name people by a rotatable
artifact rather than the stable identity.

### 3.9.1 `name@domain` — the primary human address

**`name@domain` is the everyday, recommended, headline address.** It comes in two flavours, one
format:

- **Provider-issued** — `you@envoir.org` and the like (Bluesky-style): the majority who don't
  own a domain get a familiar, handle-shaped address for free from a provider, with zero DNS work
  (Tier B, §3.8).
- **Your own domain** — `you@yourbrand.com`, for full sovereignty (Tier C, §3.8).

Why this is the headline:

- **It survives key rotation and PQ migration.** The name binds to the stable anchor `IK`, not
  to a rotatable key; the key underneath can rotate or migrate to a PQ suite and the address is
  unchanged (`name → key` indirection, §1.5–1.6). This is the real future-proofing.
- **It's federated — no global-namespace squatting.** Uniqueness is *per-domain*, so there is no
  single global registry to squat or race: `alice@a.org` and `alice@b.org` are different people,
  and neither had to win a landrush.
- **One familiar format**, already understood by every human and every mail system.
- **It interoperates with legacy email for free** — the shared or owned domain carries
  MX/DKIM/DMARC (§3.8, §7), so `name@domain` is reachable from Gmail/Outlook on day one.

Authenticity is still the key: a `name@domain` is resolved (§3.3), KT-audited (§3.5), and pinned
(§3.4) — the domain only points, it never proves. For a name that cannot expire or be seized,
the same pointer can live in a self-sovereign name backend instead of DNS (§3.6), with resolution
unchanged.

### 3.9.2 `@handle` — optional global registry (opt-in)

A flat, global `@handle` namespace is **optional, not the headline.** For handle-like UX, DMTAP
prefers the **provider-federated** `you@provider` form above, which gives the same short-name
feel *without* a single global namespace to fight over.

A conforming client MAY additionally offer an opt-in `@handle` directory. Because a *chosen,
globally-unique* name requires arbitration (Zooko), the directory is a thin authority that:

- assigns each handle once (first-come-first-served) with an **anti-squat cost** (a small fee or
  proof-of-work, since assignment gives uniqueness but not scarcity);
- publishes `handle → key` to a **key-transparency log** (§3.5), so it is *auditable, not
  trusted* — it cannot silently repoint a handle without detection;
- **cannot hijack existing relationships** — contacts route by the pinned key (§1.6), so the
  handle is only an introduction.

Handles are normalized (NFC, lowercase, collapse consecutive dots; dots allowed as cosmetic
separators, e.g. `@this.is.my.name`) and confusables/homoglyphs are reserved. The directory
SHOULD be a **federated consortium running BFT consensus** (no single owner of the namespace, no
chain) rather than a single operator; a name-chain (§3.6) is a further option.

**Honest limit (why it is not the default):** a single global namespace **reintroduces exactly
the global squatting and impersonation that domain-federation avoids** — every desirable handle
becomes a landrush and a phishing target (`@paypa1` vs `@paypal`). That is the unavoidable price
of a flat global name, and it is why `name@domain` is the recommended form and `@handle` is an
explicit opt-in.

### 3.9.3 Petnames (local)

A user assigns **petnames** locally to contacts ("Mom" → a key). Petnames are human + zero-
authority but *local-scope only* (not global), and never leave the device cluster (they are a
social-graph artifact and MUST be stored encrypted at rest with the mailbox).

### 3.9.4 Aliases & subaddressing (one identity, many addresses)

For feature parity with legacy email, **one identity MAY hold multiple names** — all resolving
to the same key — and support subaddressing:

- **Aliases.** An identity's `Identity.names` (§1.3) is a *list*: a user can hold several
  `name@domain` addresses (provider-issued and/or own-domain) at once, optionally an `@handle`,
  all pointing to the same key. Mail to any of them arrives in the same mailbox.
- **Retaining a legacy address.** Crucially, a **legacy email address you already own can be an
  alias**: point that domain's MX/DKIM at a gateway (§7) and add the address to `Identity.names`,
  so people who only know your old `you@oldprovider.com` still reach you (via the gateway →
  MOTE), while you migrate. This is the practical migration path — you don't lose your old
  address; it becomes one of your DMTAP aliases.
- **Subaddressing (plus-addressing).** `you+tag@domain` (and, for handles, `@you+tag`) is
  supported: the `+tag` is preserved for client-side filtering/labeling but resolves to the same
  key. Catch-all (`*@yourdomain`) is a domain-owner option (tier C).
- **Security:** every alias resolves to the *same key*, so an alias cannot be used to impersonate
  — authenticity is always the key, not the name. Aliases are published/audited via the same KT
  (§3.5); adding/removing an alias is a signed `Identity` version. A legacy-alias's inbound mail
  is marked *legacy-origin* (not E2E before the gateway, §7.2), so the user sees which messages
  came the old way.
