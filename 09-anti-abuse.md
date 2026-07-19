# 9. Anti-Abuse & Postage

DMTAP must stop spam/abuse **without** a central filter and **without** breaking sealed-sender
anonymity. The tension: sealed sender hides who sent a message, but the recipient still needs
to rate-limit and block abusers. The resolution is **anonymous but accountable** tokens.

## 9.1 Principles

1. **Authenticated-by-default.** Every native MOTE is authenticated to the recipient
   (payload signature, §2.4); there is no anonymous unauthenticated injection like open SMTP.
2. **Recipient policy is local.** The recipient node decides what to accept — allow known
   contacts, challenge strangers, block per-identity — *before* surfacing to the user (§2.7).
3. **Cost for cold contact.** Reaching a stranger costs something (a token, proof-of-work, or
   a paid stamp), so bulk unsolicited sending is uneconomic. Known contacts are free.
4. **Anonymity preserved.** Abuse control MUST NOT deanonymize the sender or link a sender
   across recipients.

## 9.2 Recipient policy

```
Policy {
  allow:      [* ContactRef],   // known keys / group members → free, always accept
  challenge:  ChallengeSpec,    // unknown senders must satisfy this
  block:      [* Token/KeyRef], // per-identity or per-token blocks
  rate:       RateLimit,        // per-sender-token limits
}
ChallengeSpec = pow(bits) / token(issuer) / stamp(amount) / vouch(guardianSet)
```

A stranger's first MOTE MUST carry a valid challenge response or it is dropped/deferred to a
"requests" area, never delivered to the inbox.

### 9.2a Binding proofs to the message (theft/replay prevention — normative)

Every `ChallengeResponse` is carried in the **cleartext** envelope so it can be checked before
decryption (§2.7). That means an on-path observer sees it. A proof that is valid *in isolation*
could therefore be **stripped from a victim's MOTE and re-attached to the attacker's own MOTE**
(the attacker mints a fresh ephemeral `sender_key`, copies the stolen proof into `challenge`, and
signs — both `sender_sig` and the stolen proof then verify). To close this, **a
`ChallengeResponse` MUST be cryptographically bound to the envelope's ephemeral `sender_key`**
(§2.2, field 12), so a stolen proof is worthless under any other ephemeral key:

- **ArcToken** — the ARC presentation MUST be computed over a request context that includes
  `sender_key` (in addition to `origin`). A verifier MUST reject a presentation whose context
  does not bind the `sender_key` of the carrying envelope. This is what makes the token
  single-holder without deanonymizing it (ARC stays cross-recipient-unlinkable, §9.3).
- **PowSolution** — already bound: the `epoch_nonce` scope is `id ‖ recipient ‖ nonce(epoch)`
  (`nonce(epoch)` is the recipient's published epoch beacon, defined in §9.4; parameters §16.5)
  and `id` is the content address of *this* ciphertext, so a stolen PoW only "works"
  on the identical MOTE, which the recipient de-duplicates by `id`. Implementations SHOULD
  additionally fold `sender_key` into the PoW scope for defense in depth.
- **PostageStamp** — theft is bounded by the online single-use redemption check (§9.5.1): the
  first party to redeem `serial` wins and the stamp is spent, so a stolen stamp cannot be
  redeemed twice. The presentation MUST additionally name `sender_key` in the redemption request
  so the issuer binds the spend to the presenting ephemeral key.
- **Vouch** — a stolen vouch lets an attacker past the *abuse gate* only; the message still fails
  identity authentication inside `ciphertext` at §2.7 step 8, so it is discarded. No additional
  binding is required, but a vouch MUST still be rate-limited per `subject` (§9.7).

Because `challenge` is inside the `sender_sig` preimage (§18.9.1) and `sender_sig` is made by
`sender_key`, this binding also proves the proof and the signature came from the *same* ephemeral
key for this exact envelope.

## 9.3 Anonymous rate-limit tokens (the core mechanism)

DMTAP uses **Privacy-Pass anonymous tokens** (RFC 9576 architecture, RFC 9577 HTTP scheme,
RFC 9578 issuance) so a recipient can rate-limit/block a sender **without learning who they
are** and **without the sender being linkable across recipients**.

**CAUTION — pick the right variant.** Vanilla Privacy Pass tokens are issuance-unlinkable but
do **not**, alone, give the exact combination DMTAP needs: *per-recipient rate-limiting* AND
*cross-recipient unlinkability*. That combination requires per-origin-scoped issuance —
**Anonymous Rate-limited Credentials (ARC)**, the Privacy Pass WG work
(`draft-yun-privacypass-arc`). DMTAP tokens MUST use ARC-style per-origin binding, not plain
tokens, for the anonymity + accountability guarantee to hold simultaneously.

- A sender obtains blinded tokens from an **issuer** bound to a *rate budget*.
- The sender attaches a token to a cold MOTE; the recipient verifies it is valid and unspent,
  enforces per-token rate limits, and can **block a token/issuer** if it misbehaves — all
  without identity.
- Tokens are **unlinkable across recipients** (blind signatures), so token use does not build
  a cross-recipient graph.

### 9.3.1 Issuer trust is what gives a token value (normative)

A token is only as trustworthy as its **issuer**, and issuance MUST have a cost or scarcity or
the whole "cost for cold contact" model collapses (a spammer would self-issue unlimited free
tokens). Therefore:

- The recipient's policy scores issuers (parallel to gateway reputation, §7.5). A token's
  granted rate budget is a function of **issuer trust at the recipient**, not of the token
  alone.
- A token from an **unknown/unvetted issuer (including the sender's own node) carries a default
  rate budget of ZERO** — i.e. it counts as *no token*, forcing fallback to PoW (§9.4) or
  postage (§9.5). Self-issuance thus buys anonymity but **no cost relief**.
- Trusted issuers (a recipient's own provider, a staked gateway, a vouched community issuer)
  make issuance costly/rate-limited on their side and stake reputation on not minting for
  abusers; a recipient extends budget to their tokens.

This is the load-bearing rule that makes ARC anti-abuse real rather than bypassable.

### 9.3.2 The unlinkability ↔ collective-defense tension (honest)

ARC's cross-recipient unlinkability is in **direct tension** with identifying a repeat spammer
who hits many recipients: the very property that protects a legitimate sender's social graph
also prevents recipients from cross-linking an abuser. DMTAP resolves this at the **issuer
layer, not the recipient layer**: repeat abuse is bounded by the issuer throttling/revoking its
own issuance to a misbehaving client (who is *not* anonymous to its own issuer), and by
recipients down-scoring issuers that mint for abusers. Recipient-side cross-recipient linking is
deliberately unavailable; DMTAP does not claim it. (Signal solves the analogous problem with
delivery tokens; DMTAP generalizes it to open, issuer-scored anonymous tokens.)

## 9.4 Proof-of-work (fallback)

Where no token issuer is available, a stranger MAY attach a **proof-of-work** — a puzzle whose
scope is `id ‖ recipient ‖ nonce(epoch)` (§16.5) — whose difficulty the recipient policy sets.
PoW imposes cost on bulk senders while remaining trivial for a single legitimate message.

**The freshness source (normative — this clause defines `nonce(epoch)`, referenced by §9.2a and
§16.5).** `nonce(epoch)` is the recipient's **current published epoch beacon**: a value
published in the recipient's directory/`Identity` record and rotated on the §16.5 cadence, so a
cold sender fetches it from the resolution path it already uses (§3) **without contacting the
recipient's node**. A recipient that publishes no beacon falls back to the **current UTC date**
— the coarsest permissible beacon. Either way, the epoch scope is what makes precomputed
puzzles stale.

**CAUTION.** PoW is a **last-resort** tier, not primary: there is no live standard, and plain
hashcash asymmetry favors organized spammers (botnets/GPUs) over low-power legitimate senders
(phones). DMTAP PoW MUST use a **memory-hard** function (Argon2/scrypt-style) to flatten that
asymmetry, and difficulty SHOULD be adaptive. Prefer the token/reputation path (§9.3) whenever
available.

**Bounding verification cost — the verifier's own memory-hard budget (normative).** A memory-hard
Argon2id verification is **symmetric**: checking a solution costs the recipient roughly what
producing it cost the sender. Because the cold-sender gate runs this check **before** any per-source
cap can apply (§2.7 step 6 precedes identity), a flood of **bogus** PoW attachments is itself a
memory/CPU **DoS on the recipient** — the attacker forces expensive verifications for free. A
recipient therefore MUST **bound the number of memory-hard PoW verifications it performs per time
window, per delivering connection/relay** (§16.5). Beyond that budget the recipient MUST **defer the
MOTE to the requests area WITHOUT verifying** its PoW (`DEFER_REQUESTS`, §2.7a) — never spend
unbounded memory-hard work on unauthenticated input, and never fail *open* by accepting it. This is
one more reason PoW is the **rate-capped last resort**: the ARC token / postage paths (§9.3, §9.5)
verify **cheaply** (a signature check), so they impose no symmetric-cost DoS surface and are always
preferred over PoW.

## 9.5 Postage (economic tier + gateway funding)

**Postage** is a signed, prepaid, real-money credit voucher (NOT a cryptocurrency):

- A sender MAY attach postage to a cold MOTE; heavy senders pay, contacts are free.
- **Postage doubles as the gateway fee**: when a MOTE is bridged to legacy (§7), the delivering
  gateway **redeems the stamp**, so outbound legacy sending is paid-for and revenue-neutral.
- Postage is enforced by the recipient/gateway; it is orthogonal to token/PoW anti-spam and
  MAY be combined.

### 9.5.1 Issuance, redemption & double-spend (normative)

Postage is a **bearer instrument for real money**, so it needs a settlement model, not just a
signature. DMTAP specifies the minimum:

- **Issuance:** a postage **issuer** (a provider/gateway with a real-money float) mints a stamp
  = `sign_issuer(serial, amount, expiry, [audience])`. The issuer debits the buyer's account at
  mint time. `audience` MAY scope a stamp to a specific recipient/gateway.
- **Redemption + double-spend prevention:** a stamp is single-use. The **redeeming party
  (recipient node or gateway) MUST check the stamp's `serial` against the issuer's redemption
  endpoint** (online check) or accept the issuer's signed short-lived spent-list; the issuer
  marks the serial spent atomically. A stamp presented twice is rejected on the second redeem.
- **Cross-operator settlement:** the redeemer claims the amount from the issuer against the
  serial; issuers reconcile out-of-band (standard interchange). Settlement requires an
  issuer-side serial ledger; this section specifies it.
- **Failure mode:** if the issuer is unreachable at redemption, the stamp is treated as
  *unverified* (falls back to token/PoW policy), never accepted on faith. No offline bearer
  acceptance of real money.

Postage issuers are subject to the same reputation/trust scoring as token issuers (§9.3.1);
an untrusted issuer's stamps carry no weight.

## 9.6 Gateway-operator accountability

For decentralized legacy gateways (§7):

- **Per-identity accountability**: every outbound handoff carries the sender's signature +
  postage, so a gateway attributes abuse to a *token/identity*, not an IP — making a shared
  reputation pool safe to decentralize.
- **Operator stake/bond**: operators post collateral; poisoning the pool slashes it (Sybil
  resistance + incentive alignment).
- **Reputation routing**: nodes route outbound to operators by measured deliverability;
  bad operators lose traffic.

## 9.7 Vouch / introduction (optional)

A cold sender MAY present a **vouch** — an introduction token signed by a contact the recipient
trusts (a mutual connection) — to bypass PoW/stamp. This bootstraps first contact through the
social graph without a central authority, and MUST itself be rate-limited to prevent vouch
farming.

## 9.8 Mixnet abuse

Mix nodes are content-blind and cannot filter content — and, crucially, an entry mix **cannot**
verify the recipient-facing anti-abuse proof (ARC token / postage / PoW) either, because that
proof is **sealed inside the encrypted `ciphertext`** (§2.2b, §6.2) and bound to the *recipient*,
not to the mix. Mixnet flood-abuse is therefore bounded by mechanisms a **content-blind** node can
actually apply:

- **Per-connection / per-guard-operator rate limiting (MUST).** An entry mix admits Sphinx cells
  under a **per-connection and per-operator rate budget** — it limits how fast any one source (and
  any one upstream operator) may inject cells — **not** by inspecting a sealed recipient proof it
  cannot read. This is the honest entry-admission control: it caps injection rate blind to content.
- **Optional mix-visible anti-flood PoW (MAY).** A mix MAY additionally require a **small
  proof-of-work bound to the mix hop itself** (a puzzle over the cell + epoch the mix *can* check
  without decrypting) as an anti-flood cost knob under load — distinct from the recipient's §9.4
  cold-sender PoW, which the mix cannot see.
- **Operator rate limits and stake (MUST/SHOULD).** Operator-level rate limits and mix-operator
  **stake/bond** (§6.4, §7.5, §9.6) make sustained flooding costly and attributable.

The recipient's sealed token/postage/PoW (§9.3–§9.5) still gates whether the message is *accepted
into the recipient's inbox* — but it is enforced at the **recipient**, after mix delivery, never
mistaken for something an entry mix verifies. Cover traffic is itself **rate-bounded per node**
(§4.4.7) so cover cannot be turned into the flood.

## 9.9 Group-address amplification

A **group address** (§5.8) is a first-class abuse-amplification vector: a single post fans out to
N recipients, and a naïve design evaluates the anti-abuse challenge **once at list ingress** and
then attributes delivery to the **group**, bypassing each recipient's per-sender policy (§9.2) and
**laundering the poster's accountability**. DMTAP forbids this:

- **Origin accountability carries through fan-out — with the right proof per list type.** The
  **poster's** challenge proof MUST be carried on each fanned-out per-member delivery; each
  recipient applies its per-sender policy to the **original poster**, not the group identity.
  *Which* proof depends on the list's membership model, because ARC tokens are per-origin
  (per-recipient) scoped and cross-recipient-unlinkable (§9.3) and therefore **do not compose**
  with fan-out to many recipients:
  - **Member-visible channels** (§5.8.3): the poster knows the members, so it MAY mint a
    per-member ARC token (one per recipient origin). ARC carry-through applies here.
  - **Hidden-membership lists** (§5.8.3): the poster does *not* know the members (that's the
    point), so per-member ARC is impossible without breaking membership privacy. These lists
    MUST use **postage or PoW scoped to the list address** as the poster's proof (a single
    list-scoped proof the committer/relay verifies at ingress and vouches per-delivery), not
    per-member ARC. The committer attests "this fan-out carried a valid list-scoped proof from
    poster P" so recipients still get origin accountability without learning a cross-recipient
    ARC graph. **Honest limit:** this hands the hidden-list committer a trust power recipients
    must accept — it could false-vouch (launder a spammer), misattribute (frame a poster or evade
    a recipient's block on P), or under-size the "commensurate" proof. It is bounded (the hidden
    list's committer *is* the list operator, which already re-seals the fan-out and knows the full
    membership — roughly the mailing-list trust you already accept), and abuse is detectable
    (recipients down-score a committer/list that vouches for spam, §7.5). Member-visible channels
    avoid this power entirely (each recipient verifies ARC independently), so hidden-membership
    trades a committer-trust assumption for its privacy — disclose it, don't hide it.
- **Per-poster fan-out rate-limits.** A list MUST rate-limit fan-out **per poster** and **cap
  amplification** for `open`-join lists (anyone-can-post), whose amplification is otherwise
  unbounded.
- **Cost to post to large lists.** Posting to a large list MUST require **postage or PoW**
  commensurate with the fan-out size, so mass amplification is not free.
- **Legacy fan-out** (§5.8.5) is bound by the same rules; the gateway attributes the origin
  (§9.6).

## 9.10 Gateway bidirectional anti-abuse (the two-way choke point)

§9.1–§9.9 protect **native** recipients. The **legacy gateway** (§7) is the one component that also
faces the open legacy world, so it MUST apply anti-abuse in **both** directions, fail-closed —
specified normatively in §7.11 and restated here because it belongs to the anti-abuse model:

- **Inbound (legacy → mesh).** The gateway MUST authenticate the legacy sender (SPF/DKIM/DMARC,
  §7.11.1) and MUST carry legacy senders through the **cold-sender gate** (§9.2): a legacy stranger is
  a **cold contact**, subject to the recipient's challenge policy, never injected with contact
  standing. This stops a gateway from laundering legacy spam into the accountable mesh.
- **Outbound (mesh → legacy).** The gateway MUST relay **only for authenticated senders** — an
  authorized `GatewayAuthz`/key-registered relationship (§7.12) **or** valid redeemable postage
  (§9.5) — and MUST apply per-sender **rate limits and volume caps** (§9.6). An unauthenticated
  outbound relay attempt is refused fail-closed with `ERR_GATEWAY_SENDER_UNAUTHENTICATED` (`0x0607`,
  §21.8): a valid `sender_sig` proves *who signed*, not *who may relay*, so signature-validity alone
  MUST NOT authorize egress. This is the open-relay floor (§7.7) that per-identity accountability
  (§9.6) makes affordable to hold open.

As everywhere in §9, only the **floor** — that both directions are gated, fail-closed, on these
signals — is in-spec; the **thresholds, caps, and pricing** are operator policy and out of scope
(§7.13).
