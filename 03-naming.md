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
- **`did:web` consistency (normative, for DMTAP-Auth §13.6).** Where the same identity is also
  published as a `did:web` document (`did.json`), that document's key MUST be **byte-consistent
  with this DNS `name → key` binding and its KT entry** (same `IK`, same `Identity` hash). A
  verifier MUST **cross-check the two and pin** (§3.4); a `did.json` is the same discovery-only
  pointer as DNS and never proof on its own.

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

Two profiles are defined, both registered as KT log-types (§21.19) and selected by capability
negotiation (§10.2): **v0-minimal (log-type `0x01`)** is the interoperable default every Core
node MUST implement (§10.3); **v1-hardening (log-type `0x02`)** is specified below and used only
between a verifier and a log set that advertise it. v0 stays the floor; v1 closes the
equivocation gap.

### 3.5.1 v0-minimal — the interoperable default (log-type `0x01`)

The default profile, required at **Core** conformance (§10.3), is a **single append-only Merkle
log** (§21.19 `0x01`): signed tree heads (STHs) + inclusion proofs + rollback defense (§3.3
step 2), self-monitored by the owner's own devices (STH poll, §16.2). It is
**tamper-evident-after-the-fact and self-monitorable, but not equivocation-proof** — a single,
non-gossiped log can present a **split view** (different histories to different observers, §6.6
item 6). v0 therefore fails closed on an unreachable log (§3.3, `0x0106`), and a network SHOULD
run more than one independent log and lean on OOB verification (§3.4.1) for high-value contacts
even in v0. Closing the split-view gap is exactly the job of the v1 profile.

### 3.5.2 v1-hardening — federated, gossiped, equivocation-detecting (log-type `0x02`)

The v1 profile is **specified here and negotiated as a capability** (§10.2 — a KT-log-type token
in the `system` MOTE / the `kt=` DNS anchor, §3.2). It is **not** the interoperable default:
v0-minimal (§3.5.1) remains what a Core node MUST implement, and a verifier uses v1 only with a
counterpart and log set that advertise log-type `0x02` (§21.19). v1 makes **no single log
authoritative** and turns a split view from a deterred-but-undetectable weakness into a
**detected, attributable, and responded-to** event. It has four normative parts (a)–(d); the
detection paths carry codes `0x0107`, `0x0110`–`0x0112` (§21.3).

#### (a) STH gossip — split-view detection

- **Who gossips.** Every v1 verifier (client, monitor, auditor, §3.5.2(c)) that fetches an STH
  MUST re-publish the signed head it saw — the tuple `(log signing key, tree_size, root_hash,
  timestamp, log signature)` — to a set of gossip peers (other verifiers, the owner's monitor
  devices, and any configured auditor) over the mesh (§4). It SHOULD route this gossip through
  the mixnet (§6, §3.7) so auditing does not leak *who audits whom*.
- **Cross-check (the detection step).** On receiving a gossiped STH for a log it also follows, a
  verifier MUST request a **consistency proof** between its own latest STH and the gossiped one
  and verify that the smaller tree is a **prefix of** the larger (a genuine append-only
  extension). Two validly-signed STHs from the same log with the **same `tree_size` but a
  different `root_hash`**, or a consistency proof that cannot be produced between two heads of
  one log, is cryptographic proof of **equivocation** → `ERR_KT_STH_INCONSISTENT` (`0x0110`) and
  `ERR_KT_EQUIVOCATION` (`0x0107`), handled per (d).
- **Freshness (freeze-attack defense).** Gossip MUST occur at least once per **KT gossip
  interval** (§16.2). A v1 log MUST publish a new STH at least every **maximum merge delay**
  (§16.2) and MUST include every accepted entry in an STH within that bound; a binding it
  accepted but did not include within the MMD is evidence of withholding/censorship. A verifier
  MUST treat an STH older than the **STH freshness window** (§16.2) as stale (`ERR_KT_STH_STALE`,
  `0x0112`) and refresh it — closing the **freeze attack** where a log serves an old but
  internally self-consistent head to a targeted observer.

#### (b) Multi-log federation — quorum-audited bindings

- **Pin a set, not a log.** A v1 verifier pins a **set** of logs for a name (the `kt=` DNS/SVCB
  anchor MAY list several, §3.2), and accepts a `name → ik` binding only when it appears, with a
  valid inclusion proof, in a **`> n/2` quorum** of that pinned set — the same strict-majority
  rule the group committer takeover uses (§5.1, §16.8 roster quorum). A **minority of malicious
  or partitioned logs therefore can neither forge nor suppress** a binding, and **no single log
  is authoritative**.
- **Independence.** Each log identifies itself by its own signing key (§21.19), runs its own
  append-only tree, and the set SHOULD be operated by **distinct operators** so a quorum does not
  share fate; a verifier SHOULD prefer logs under disjoint operational control (analogous to
  S/Kademlia disjoint paths, §4.2).
- **Fail closed below quorum.** If fewer than the quorum of pinned logs attest the binding
  (logs disagree, or too many are unreachable), resolution MUST fail closed →
  `ERR_KT_LOG_QUORUM_UNMET` (`0x0111`); the verifier MUST NOT pin on a sub-quorum view (the same
  fail-closed discipline as v0, §3.3). Bindings SHOULD be **cross-logged** (submitted to every
  log in the set) so the quorum is normally the whole set and auditors (below) can check each
  member log against the others.

#### (c) Monitor and auditor roles

Two distinct roles, which MAY be co-located on one operator but are separate capabilities:

- **Monitor** — watches a *specific identity's* entries. The owner's own devices MUST monitor
  **every log in the identity's pinned set** for any entry under the owner's name/`IK`, and MUST
  `HALT_ALERT` (§21.2) on any change the owner did not initiate — KT as **identity intrusion
  detection** (§3.5 preamble; the same owner-alert obligation as `0x010B`/`0x010E`). A relying
  party MAY additionally monitor the identities it depends on (e.g. a login RP, §13.7).
- **Auditor** — watches a *whole log's* integrity, name-agnostically. An auditor MUST verify
  every STH's signature and that each new STH is a **consistent append-only extension** of the
  heads it has already seen (no rewrite, no rollback of history), and MUST gossip STHs (part (a))
  so its view is cross-checkable against others. An auditor needs **no knowledge of any user's
  key** and SHOULD run the mixnet-side private path (§3.7) so auditing itself leaks no user
  identities. A conformant v1 deployment SHOULD run **≥ 2 independent auditors per log**.
- **Separation of concerns.** Monitors catch a **targeted** key-swap (one bad entry for one
  name); auditors catch **global** misbehavior (a log rewriting or forking its own tree); a
  **split view** (a log self-consistent but showing different observers different heads) is
  caught by the **gossip cross-check** of part (a), which both roles feed.

#### (d) Equivocation detection and response

- **Detection.** Any of the following is proof — under the log's *own* signature — that a log
  equivocated: (i) two validly-signed STHs of one log with equal `tree_size` but differing
  `root_hash`; (ii) two STHs of one log between which no valid consistency proof exists
  (append-only violation); or (iii) a `name → ik@version` binding attested by some quorum members
  and **contradicted** (a different `ik` for the same name and version) by others.
- **Response (normative).** On any of the above a v1 verifier MUST:
  1. **HALT** — stop treating the offending log as authoritative and MUST NOT pin or update any
     `name → ik` binding on its say-so (analogous to committer fork-detection, §5.1, §19.5.6, and
     to `0x0104`/`0x0209`).
  2. **ALERT** — raise a user-/operator-visible security alert (`HALT_ALERT`, §21.2), emitting
     `ERR_KT_EQUIVOCATION` (`0x0107`) and, for the append-only-violation form,
     `ERR_KT_STH_INCONSISTENT` (`0x0110`).
  3. **Publish evidence.** The two conflicting signed STHs (or the contradicting inclusion
     proofs) are **self-contained, transferable proof of misbehavior** — an equivocating log
     signs its own indictment. The verifier SHOULD gossip this evidence to its peers and auditors
     so the equivocation becomes **globally attributable**, not merely locally observed.
  4. **Recover on the honest quorum.** The identity stays verifiable on the **remaining honest
     quorum** of the pinned set (part (b)): if a `> n/2` quorum still agrees on the binding,
     resolution MAY proceed with the offending log **evicted** from the set and its operator
     treated as a reputation signal (as gateway/postage misbehavior is, §7.5, §9.6); if the
     equivocation breaks quorum, resolution fails closed (`0x0111`) and the verifier falls back
     to OOB verification (§3.4.1). For **DMTAP-Auth**, a split view is a silent per-RP
     account-takeover vector, so a high-value login RP MUST require this multi-log quorum or an
     OOB-verified pin (§6.6 item 6, §13.7).

Under log-type `0x02` the honest v0 limit — "tamper-evident after the fact, self-monitorable, but
not equivocation-proof" (§6.6 item 6) — is **closed**: equivocation is detected by gossip,
bounded by the federation quorum, and responded to by halt-alert-and-evict.

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
- **Security — self-asserted names need forward verification (normative).** `Identity.names` is
  **self-asserted**: an identity can *list* any string, including a **victim's address**. A listed
  name therefore proves **nothing** on its own, and a client MUST trust or display a name in
  `Identity.names` **only after verifying the forward `name → ik` binding** (DNS + KT, §3.3–3.5)
  **also resolves to this same key** — i.e. the name points back. Before accepting a name
  (especially a **legacy alias**), the client MUST require **proof of control**: a DNS challenge
  under that name, or a **KT-anchored per-name record** binding `name → ik`. A name that does not
  verify back MUST be rendered **as unverified**, never shown as an authenticated address. Given
  that check, every *verified* alias resolves to the *same key*, so it cannot be used to
  impersonate — authenticity is always the key, not the name. Adding/removing an alias is a signed
  `Identity` version, audited via the same KT (§3.5). A legacy-alias's inbound mail is marked
  *legacy-origin* (not E2E before the gateway, §7.2), so the user sees which messages came the old
  way.

## 3.10 Organization & domain administration

An organization that controls `@abc.com` needs the things a management console provides — add
users, run a directory, create distribution lists, assign admin roles, offboard people. DMTAP
provides all of them by **composing primitives it already has**, not by bolting on a parallel
system: the domain's DNS + `kt=` anchor (§3.2) is the root of authority for *names*; provisioning
a member is publishing a `name → key` binding (§3.9); the directory is a signed, enumerable
projection of those bindings; org groups are §5.8 group-identities; admin roles are §13.5
capabilities; offboarding is removing a name and revoking capabilities. The sovereignty ethic is
held throughout by one honest invariant: **the org controls names and operations, never a
sovereign member's key** — and where it *does* hold a key, that is disclosed (§3.10.2b).

### 3.10.1 Domain authority (what it means to control `@abc.com`)

Controlling `abc.com` means controlling its DNS zone and the `_dmtap` / `kt=` anchors (§3.2) that
state which key each `name@abc.com` points to. That control is the **root of authority for names
under the domain — and only for names, never for keys.** The authority can say *which key a name
points to* and can add or remove names, but it cannot forge, read, or impersonate anything a key
protects, because members hold their own keys (§3.10.2a) and KT makes every binding auditable
(§3.5).

- **Who holds it.** The domain authority is an ordinary DMTAP identity (§1.3) whose `IK` is
  published at the domain apex (`_dmtap.abc.com`) and anchored in KT. To avoid a single unilateral
  super-admin over the whole namespace, the authority's signing key SHOULD be **threshold-held by
  the domain-owner / domain-admin set**, reusing the group-key custody discipline (§5.8.6,
  FROST-style over the §1.4 machinery): rotating the domain anchor or the directory-signing key is
  a domain-authoritative act requiring a threshold, not one admin (§13.5.1).
- **Relation to onboarding tiers (§3.8).** A **Tier B** provider *is* the domain authority for
  `gw.example` and administers all its users. A **Tier C** org owning `yourbrand.com` becomes its
  own domain authority, self- or provider-operated. **Tier A** has no domain and therefore no
  domain-administration layer — it is an individual identity, not an org.

### 3.10.2 Member provisioning — two honest models

Creating `alice@abc.com` means publishing the binding `alice@abc.com → alice_ik` under the
domain: a `_dmtap` DNS record (§3.2), a KT entry (§3.5), and a directory entry (§3.10.3). Two
models differ **only in who generates and holds `alice_ik`**, and the difference is a §6.6-style
honest limit that MUST be disclosed. The default SHOULD be sovereign.

**(a) Sovereign member (default, SHOULD).** Alice generates her own `IK` on her own device; the
org merely **publishes the `name → key` binding**. The org can add or revoke the *name*, but
**cannot read her mail, cannot impersonate her, and cannot recover her key** — it never held the
key. This is the model that keeps "your key is your identity" (§1) true *inside* an org, and it is
what makes offboarding a name-revocation rather than a mailbox seizure (§3.10.5).

**(b) Org-managed member (convenience/compliance, disclosed).** The org generates and/or
**escrows** the account key (e.g. as a guardian/quorum in the account's `RecoveryPolicy`, §1.4).
This buys admin password-reset, compliance hold, and legal discovery — at the cost that **the org
CAN access the account and CAN impersonate the user.** That capability MUST be **disclosed and
machine-visible**: the directory entry and the member's `Identity` carry `custody = "org-managed"`
(§18.4.7), rendered to the user and to correspondents as an honest limit exactly like a
legacy-origin marker (§3.9.4). A message from an org-managed identity carries less individual
assurance than a sovereign one and MUST NOT be silently presented as equivalent. **Undisclosed
escrow** — an org-managed account presented as sovereign — MUST fail closed
(`ERR_ORG_MANAGED_UNDISCLOSED`, §21).

Org-managed is opt-in per account, chosen deliberately for a stated compliance need, never the
silent default. Even org-managed does not weaken transport crypto or metadata privacy (§12.3): the
org's access is via **holding the key, disclosed**, not via a protocol backdoor.

### 3.10.3 The organization directory (GAL)

A `name@abc.com` autocomplete / global address list is a **`DomainDirectory` object (§18.4.7)**: a
signed, versioned, KT-logged enumeration of the domain's member (and group, §3.10.4/§5.8.7)
bindings. This closes the org-directory gap flagged in §17 (item 33).

- **Where it lives.** Published like any identity object — content-addressed in the mesh, located
  via the domain's `_dmtap` anchor (§3.2, a `dir=` locator), and its root **appended to key
  transparency** (§3.5) so its history is append-only and auditable.
- **Authentication.** Signed by the **domain authority** (§3.10.1), i.e. under the threshold-held
  domain key. A verifier MUST reject a directory not validly signed by the pinned authority
  (`ERR_DOMAIN_DIRECTORY_SIG_INVALID`, §21).
- **The directory cannot forge a binding (load-bearing).** Each `DirEntry`'s `name → ik` is only
  an **index**: a client MUST independently verify it against the **forward DNS + KT binding**
  (§3.3–3.5) before trusting or displaying it — the self-asserted-name rule of §3.9.4 applied to
  the org's assertion. An entry that does not resolve forward to the same key MUST be rendered
  unverified and MUST NOT be used to address mail (`ERR_DIRECTORY_ENTRY_UNVERIFIED`, §21). So the
  GAL is a *convenience enumeration of independently-verifiable bindings*, **not a new root of
  trust**: a compromised directory can withhold or mislabel names (a denial/annoyance, detectable
  via KT) but can never make mail encrypt to a key the member does not hold.
- **Query.** Members query the whole directory (autocomplete) via the domain's node; an outsider
  resolves a single `name@abc.com` the ordinary way (§3.3) and needs no directory access at all.
- **Privacy posture (membership is a choice).** `DomainDirectory.membership_visibility` (§18.4.7)
  is `public` or `members-only`, mirroring group membership visibility (§5.8.3). A `public`
  directory (a company staff page) is world-listable; a `members-only` directory serves entries
  only to authenticated members, so the *membership roster* is not a public artifact even though
  each individual `name@abc.com` remains resolvable if you already know it (the same resolvability
  legacy email already exposes). Which posture applies is an org policy choice the org MUST
  disclose to its members.

### 3.10.4 Admin roles (summary; capability machinery in §13.5.1)

Who may add/remove members, edit the directory, and manage groups is expressed as **§13.5
capabilities** delegated from the domain authority, in four roles: **domain-owner**,
**domain-admin**, **user-admin**, **group-admin** (§13.5.1). Domain-authoritative acts — rotating
the domain anchor, changing the directory-signing key — require the domain's **threshold**
(§3.10.1, §5.8.6), so no single admin is a unilateral super-admin over the namespace. The full
capability grammar, attenuation, revocation, and KT-logging are in §13.5.1. Org distribution lists
and team inboxes are in §5.8.7.

### 3.10.5 Offboarding & lifecycle

Removing `alice@abc.com`:

1. **Remove the name binding.** Publish a `DomainDirectory` version dropping her entry and retire
   the `_dmtap` DNS record for `alice` (§3.2); the removal is KT-logged (§3.5), so "who was
   offboarded when" is auditable.
2. **Revoke org capabilities.** Revoke any admin/role capabilities delegated to her (§13.5.1,
   §13.4 revocation) and remove her from org groups via the normal §5.8.2 Remove (which re-keys
   shared folders, §6.7). Losing the name does not by itself evict her from a group — that is a
   separate §5.8 Remove.
3. **Mailbox disposition — this is where the two models diverge honestly.**
   - **Sovereign member (a):** her **key survives** — the org removed only the *name*. Her
     identity, contacts, and history are hers (§1.6); she can rebind the same key to a new name
     (`MoveRecord`, §1.6) and her existing correspondents follow her by key automatically. The org
     retains nothing of hers it did not already hold, and it cannot read a mailbox it never had the
     key to. Offboarding is a **name revocation, not a mailbox seizure**.
   - **Org-managed member (b):** because the org holds/escrows the key (§3.10.2b), it CAN retain,
     transfer, or archive the mailbox for continuity/compliance — the disclosed cost of that model.
     A conformant client MUST have shown the user this was possible (the `org-managed` marker,
     §18.4.7) at provisioning, so retention is not a surprise.

A domain-wide teardown (the org shuts down or changes providers) is the domain-authority analog of
individual migration (§1.6): sovereign members keep their keys and re-bind their names elsewhere;
only org-managed accounts depend on the org to move them.
