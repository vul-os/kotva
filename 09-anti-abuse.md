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

Where no token issuer is available, a stranger MAY attach a **proof-of-work** (a hashcash-style
puzzle over the MOTE `id` + recipient + date) whose difficulty the recipient policy sets. PoW
imposes cost on bulk senders while remaining trivial for a single legitimate message.

**CAUTION.** PoW is a **last-resort** tier, not primary: there is no live standard, and plain
hashcash asymmetry favors organized spammers (botnets/GPUs) over low-power legitimate senders
(phones). DMTAP PoW MUST use a **memory-hard** function (Argon2/scrypt-style) to flatten that
asymmetry, and difficulty SHOULD be adaptive. Prefer the token/reputation path (§9.3) whenever
available.

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
  serial; issuers reconcile out-of-band (standard interchange). "Trivial" was an overstatement —
  it requires an issuer-side ledger, specified here.
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

Mix nodes are content-blind and cannot filter content. Abuse of the mixnet itself (flooding) is
bounded by: token/PoW admission at the *entry* (a MOTE without valid postage/token is not
accepted into the mixnet by the sender's own entry policy), operator rate limits, and stake for
mix operators (§6.4, §7.5). Cover traffic is rate-bounded per node.
