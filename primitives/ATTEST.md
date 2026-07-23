# ATTEST — signed claims over the substrate

> **Status:** additive normative primitive spec of the KOTVA family. It defines **no new signed
> wire object and no new signature**: an `Attestation` (§2) is an unsigned deterministic-CBOR map
> carried as opaque bytes **inside** an existing substrate object, authenticated **only** by that
> carrier's own existing signature —
> - **public** — `det_cbor(Attestation)` embedded under a profile-named `meta` key of a
>   `PubAnnounce` (kind `0x40`, §22.3.1 key 5 / [`22-public-objects.md`](../22-public-objects.md)),
>   authenticated by `PubAnnounce.sig` (DS-tag `DMTAP-PUB-v0/announce`); or
> - **private** — carried as the `body` of an ordinary sealed **content** MOTE (kind `0x00` mail /
>   `0x01` chat, §2.3 of [`02-mote.md`](../02-mote.md)), authenticated by that MOTE's `Payload.sig`
>   (DS-tag `DMTAP-v0/payload`, §18.9.2) —
>
> carrying an **EAS attestation or a W3C Verifiable Credential** ([bindings](../bindings/README.md)).
> This document owns only the **mapping** (issuer/subject → substrate identity), the **carrier
> selection**, the **revocation rules**, and the **honest residual**. Where it and a normative-byte
> home (§18, §22, the ratifying binding) disagree, the byte home governs.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are BCP 14 (RFC 2119 / RFC 8174).

---

## 1. Purpose

ATTEST is the primitive of **signed claims**: *identity `I` asserts statement `S` about subject
`X` at time `t`*. It is the substrate under **reputation, credentials, and KYC-optional
identity** — a review, a "licensed plumber" badge, an over-18 proof, a proof-of-personhood, a
"delivery happened" oracle statement are all one shape: an issuer-signed claim, verifiable
offline against the issuer's key, meaning **exactly** *"`I` said this"* and nothing more.

ATTEST invents no credential format ([DIRECTION §3](../DIRECTION.md)). The claim body is EAS or
W3C-VC; the substrate contributes only the binding of issuer and subject to a **keypair identity**
(§1), the choice of **public vs sealed** carrier, and cooperative **revocation**. It mints no
token and carries no funds ([DIRECTION §5](../DIRECTION.md)); it carries no network-wide score
([CONTRACT §2.1](../coordinator/CONTRACT.md)) — a score is *derived* from attestations by
REPUTATION, never stamped into one.

---

## 2. Objects it defines (wire-shape sketch)

No new MOTE kind is allocated and no new signature is minted (subtraction discipline,
[DIRECTION §9](../DIRECTION.md)). An attestation rides one of **two existing carriers** and is
authenticated **only** by that carrier's own signature — the `Attestation` map itself carries none:

- **public** — embedded as `det_cbor(Attestation)` bytes under a profile-named key (e.g.
  `"attestation"`) of a `PubAnnounce`'s `meta` map (kind `0x40`, §22.3.1 key 5, the profile-embedding
  rule of [`22-public-objects.md`](../22-public-objects.md)), signed in the clear, fetched by
  content address, appended to the issuer's author feed. `PubAnnounce.pub` **MUST** equal
  `Attestation.issuer` (below). Use when the claim is meant to be openly verifiable (badges,
  reviews, oracle facts).
- **private** — carried as the `body` of an ordinary sealed **content** MOTE (kind `0x00` mail /
  `0x01` chat, §2.3 of [`02-mote.md`](../02-mote.md)), delivered to the subject, revealed only to
  the recipient. `Payload.from` **MUST** equal `Attestation.issuer`. Use for private identity
  claims (a KYC pass the holder discloses selectively). KOTVA has **no dedicated sealed-attestation
  MOTE kind**: `0x09` is `Identity`/`Move`/`RecoveryPolicy` only (02-mote.md:114) and MUST NOT be
  used to carry a generic attestation claim.

The `Attestation` map below is the **payload** carried inside whichever carrier is chosen; it is
an integer-keyed deterministic-CBOR map (§18.1) whose `claim` body is an EAS attestation or a VC —
**not a format defined here**. It has **no signature field of its own**: the carrier's existing
`sig` (`PubAnnounce.sig` or `Payload.sig`, above) covers the embedded bytes end-to-end and is the
**only** signature a verifier checks.

```cddl
Attestation = {
  1 => u8,            ; v         format version (0)
  2 => u8,            ; suite     algorithm suite (§1.1)
  3 => bytes,         ; issuer    issuer root identity key IK (§1.2); MUST equal the carrier's
                      ;           authenticated identity (PubAnnounce.pub / Payload.from, above)
  4 => Subject,       ; subject   who/what the claim is about (§2.1)
  5 => SchemaRef,     ; schema    EAS schema UID or VC type URI — identifies claim semantics
  6 => Claim,         ; claim     the EAS attestation / W3C VC body (bound, NOT invented — §bindings)
  7 => u64,           ; ts        issuance time (ms epoch), display/ordering only
  ? 8 => Revoke,      ; revoke    revocation binding (§2.2); absent ⇒ no status-list/expiry
                      ;           revocation declared for this attestation — supersession for
                      ;           public attestations is via PubAnnounce.supersedes (§4.3), not
                      ;           an Attestation field
  ; key 9 (supersedes) is retired: it duplicated PubAnnounce.supersedes (§22.3.4, same-pub,
  ; ERR_PUB_SUPERSEDE_INVALID) on the public carrier and was inoperative on the sealed carrier
  ; (no feed to detect it on, §9). Public attestations supersede via PubAnnounce (§4.3); the
  ; sealed carrier has no supersession detection at all.
  ; key 10 (sig) is retired: the Attestation carries no signature of its own (§2, above); the
  ; carrier's sig authenticates it.
}

Subject = bytes            ; a subject IK (§1), OR
        / hash             ; a content-address (an OFFER listing, an order, a blob), OR
        / BlindedSubject   ; a hiding commitment for a private subject (below)

BlindedSubject = {
  1 => bytes,        ; salt    32-byte random salt, unique per commitment
  2 => hash,         ; commit  0x1e ‖ BLAKE3-256(salt ‖ subject_IK) — the hiding commitment
}

SchemaRef = tstr           ; EAS schema UID (0x-hex) or a VC @type URI

Revoke = {
  ; key 1 (mode) is retired: the mechanism is self-describing by field presence, no discriminator
  ; needed — {list,index} => status-list; {nbf,exp} => expiry; both pairs MAY coexist on one
  ; Revoke (mode used to force them mutually exclusive). Attestation key 8 absent, above, is
  ; already the supersede-only case.
  ? 2 => tstr,             ; list   status-list locator (W3C Bitstring Status List / EAS registry)
  ? 3 => u64,              ; index  bit index within the status list; present iff list present
  ? 4 => u64,              ; nbf    not-before bound (ms epoch)
  ? 5 => u64,              ; exp    expiry bound (ms epoch) — at least one of nbf/exp REQUIRED
                           ;        when either is present
}
```

- **`issuer` / `signer`.** `issuer` is the root `IK` and **MUST** equal the carrier's
  authenticated identity — `PubAnnounce.pub` (public) or `Payload.from` (private). The carrier's
  `sig` MAY be produced by a `DeviceCert` operational subkey (§1); a verifier MUST confirm
  `signer == issuer` **or** a valid non-revoked `DeviceCert` chain from `signer` to `issuer` before
  accepting (identical to the §22.3.3 `PubAnnounce` rule; the equivalent MOTE rule for the private
  carrier is `Payload.from`'s device-key authorization, §1).
- **`subject`.** Operator-independent by construction — an `IK` or a content address, never a
  registry handle or a coordinator-local row id. This is what makes an attestation **portable**:
  it names the same subject after any coordinator is swapped ([CONTRACT §2.2](../coordinator/CONTRACT.md)).
- **`claim`.** Opaque to the substrate. Its semantics are the referenced EAS schema / VC type;
  the substrate verifies only the *signature over* it, never *its truth* (§7).
- **`BlindedSubject` (open/verify).** `commit` hides `subject_IK` behind a salted digest. A
  verifier holding a claimed `subject_IK` and the matching `salt` **opens** the commitment by
  recomputing `commit' = 0x1e ‖ BLAKE3-256(salt ‖ subject_IK)` and accepting iff `commit' == commit`
  (S-ATT-2's "does not open ⇒ reject"). The opening (`salt`, `subject_IK`) is supplied
  out-of-band by whichever party needs to prove the subject to that verifier — the issuer at
  issuance, or the subject itself later — never inside the `Attestation` (that would defeat the
  hiding). A verifier that never receives an opening treats `subject` as unresolved, not as a
  match.

**Profile instances (normative examples, not re-definitions).** TRACT `PurchaseAttestation`
([§10.2a](../profiles/tract/10-trust.md)) and WRAP work attestations are already this shape,
addressing their subject by content-address or `IK`. A profile MAY define a schema and constrain
fields; it MUST NOT invent a parallel attestation object (the waist adoption rule,
[substrate/README §3.2](../substrate/README.md)).

---

## 3. Normative rules

- **R-ATT-1 (issuer-standing only).** An attestation proves *`issuer` said `claim` about
  `subject` at `ts`* and **MUST NOT** be presented by a client as an established fact beyond the
  issuer's standing (cf. TRACT §10.2a "what attestation does not establish"). What the claim is
  *worth* is the verifier's judgement of the issuer, made on the verifier's device.
- **R-ATT-2 (offline-verifiable).** Verification **MUST** succeed offline, zero-DNS, from the
  object alone: reject unknown `v`/`suite`; verify the **carrier's** `sig` (`PubAnnounce.sig` /
  `Payload.sig`, §2) under `signer`; verify `signer`'s authority chains to `issuer`, and that
  `issuer` equals the carrier's authenticated identity (`PubAnnounce.pub` / `Payload.from`). A name
  is needed only to *display* who `issuer` is, never to verify (§3.13).
- **R-ATT-3 (self-issuance is legal and worthless).** An issuer MAY attest about any subject,
  including itself. The substrate does not forbid self-dealing; it makes it **visible** — a
  self-issued or fresh-key attestation is a genuine signature that a discounting verifier
  (REPUTATION §3 [`REPUTATION.md`](REPUTATION.md), REP-5 — anchoring/discounting) is free to
  weight at zero. Do not confuse *signed* with *credible*.
- **R-ATT-4 (revocation = supersede or status-list, cooperative).** Revoking an attestation is
  either publishing a **superseding** attestation (§4.3) or flipping a bit in the referenced
  **status list**. Both are **cooperatively honoured** (as with review retraction, TRACT §10.2d):
  a conformant verifier that can reach the status/supersede evidence MUST apply it; the protocol
  cannot force a `terminating` intermediary or an offline verifier to un-see bytes it already
  holds (§9).
- **R-ATT-5 (no score, no funds, no token).** An `Attestation` **MUST NOT** carry a network-wide
  reputation number, a price ranking, a stake field, or any protocol-native value
  ([CONTRACT §2.1](../coordinator/CONTRACT.md), [DIRECTION §5](../DIRECTION.md)). Money and stake,
  where a claim references them, are denominated in an existing asset on an existing rail.
- **R-ATT-6 (physical facts go through an oracle).** A claim about a **physical event** ("the
  parcel arrived", "the ride completed") **MUST** be issued by an `oracle`
  ([CONTRACT §5](../coordinator/CONTRACT.md)) — accountable, swappable, self-declared visibility
  `terminating`, disclosed. The oracle *authorizes and attests*; it **MUST NOT** classify content
  ([CONTRACT §4](../coordinator/CONTRACT.md)).
- **R-ATT-7 (personhood is a binding, not our biometrics).** A proof-of-personhood attestation
  **MUST** bind to World ID / Human Passport ([bindings](../bindings/README.md)); the substrate
  defines no biometric of its own. It is the anti-Sybil anchor other primitives consume, and it is
  **imperfect by disclosure** (§9).

---

## 4. Composition with the other primitives

- **REPUTATION** is ATTEST *aggregated*. Reviews are attestations over public feeds; OpenRank
  (EigenTrust, TEE) computes a **derived, rebuildable, per-context** ranking over the attestation
  graph — never a published global score baked into any object ([CONTRACT §2.1](../coordinator/CONTRACT.md)).
  ATTEST supplies the edges; REPUTATION is the (swappable) function over them.
- **OFFER** attaches attestations — an issuer's "licensed", a personhood proof, a
  `PurchaseAttestation` — to a listing, so a buyer weighs the seller by verifiable claims, not by
  a platform's opaque badge. Attestation and listing are both §22 public objects.
- **MATCH / RESERVE** may *authorize* participation on a required attestation (e.g. personhood
  before joining an order book). This is **authorize-never-classify**
  ([CONTRACT §4](../coordinator/CONTRACT.md)): the gate checks *who you are*, never *whether your
  content is wanted*.
- **ESCROW / oracle.** A dispute or release condition consumes an oracle attestation of the
  physical fact; ESCROW carries the funds, ATTEST carries the claim — **attestations on the wire,
  funds on the rail** ([TRACT §9.2](../profiles/tract/09-settlement.md)).
- **Identity (§1).** Every `issuer` and every `subject` is a keypair; a personhood attestation is
  what lets a keypair stand in for "a distinct human" at global scale without the substrate
  minting an identity registry.

### 4.3 Supersession & equivocation (public carrier only)

Supersession for a public attestation rides `PubAnnounce.supersedes` (§22.3.4, same-`pub`,
`ERR_PUB_SUPERSEDE_INVALID`) — a within-issuer edit at the carrier level, not an `Attestation`
field (§2). Because successive announcements live on the issuer's append-only author feed
(§22.4.2), the feed's `prev` hash-chain makes an issuer that signs **two contradictory claims at
one feed position** *detectably* equivocating (`ERR_PUB_FEED_CHAIN_BROKEN`, HALT_ALERT — FEEDS
§4.3). An issuer cannot honestly present two histories; a verifier that sees both holds
transferable evidence. Equivocation is **surfaced for dispute, never merged away** (the SYNC /
OFFLINE invariant, [substrate/OFFLINE.md](../substrate/OFFLINE.md) R-SYNC-1).

The **private sealed carrier has no supersession or equivocation detection**: a sealed content
MOTE (§2) is on no feed, so there is no `prev` chain and no `PubAnnounce.supersedes` to check. A
malicious issuer can hand two recipients contradictory sealed claims and no single verifier ever
holds both to detect it (§9).

---

## 5. Binding adopted

Per [`bindings/README.md`](../bindings/README.md), ATTEST binds and does not reinvent:

| Need | Bind to | Note |
|---|---|---|
| Claim / credential shape | **EAS** + **W3C Verifiable Credentials** | the `claim` body (key 6); mature plumbing, nascent portable-reputation layer |
| Revocation status | **W3C Bitstring Status List** / EAS revocation registry | the `Revoke.list` locator (Revoke key 2, carried under Attestation key 8) |
| Proof-of-personhood | **World ID** / **Human Passport** | the anti-Sybil anchor; imperfect (§9) |
| Selective disclosure | VC selective-disclosure / BBS-class, when the binding matures | a `Claim` profile, not a substrate change |
| Reputation compute | **OpenRank** (EigenTrust, TEE-verified) | consumes the graph; produces no stored score |

When a binding improves, swap the filling; the `Attestation` mapping, the carriers, and the
revocation rules do not change ([DIRECTION §9](../DIRECTION.md), "future-proof by seams").

---

## 6. Scale-invariance

The object is identical at every scale; only the **issuer's standing** slides
([DIRECTION §6](../DIRECTION.md)):

| Function | Mesh / web-of-trust (offline, no coordinator) | Global (swappable coordinator) |
|---|---|---|
| Personhood | a peer you know vouches (`issuer` = a friend's `IK`) | a personhood attester you chose (World ID / Human Passport) |
| Credentials | a local authority you recognise signs | a licensed issuer / registry |
| Physical facts | dual counter-party confirmation | an `oracle` coordinator you hired |
| Aggregation | you weigh issuers you know directly | an `indexer` runs OpenRank over the graph |

Remove connectivity and every attestation still **verifies** and still **means** what its issuer
said; only the reach of *discovering* new issuers and *fetching* live revocation status is lost.
The web-of-trust form is genuinely weaker in coverage than a global attester — disclosed, not
hidden (§9).

---

## 7. Offline / apocalypse behaviour + reconcile

Per the degradation grades of [`substrate/OFFLINE.md`](../substrate/OFFLINE.md):

| Action | Grade | Behaviour offline |
|---|---|---|
| **Issue** an attestation | `full` | Signed locally; the object is self-authenticating the instant it exists. |
| **Verify** signature + issuer chain | `full` | Offline, zero-DNS, from the object alone (R-ATT-2). |
| **Check revocation** (status-list) | `local-trust` → `deferred` | Verifiable **only against the last cached status list**; a fresher status **MUST** be fetched on reconnect. |
| **Check revocation** (supersede) | `full` on held feed / `deferred` for unseen | Applies from the issuer's cached feed; catches up on reconnect. |
| **Issue a physical-fact oracle claim** | `blocked` | Needs the live oracle; **MUST fail closed and say so** — never a fabricated "delivered". |

- **R-ATT-OFF-1 (fail-closed revocation).** An attestation whose revocation status **cannot** be
  checked (status list unreachable, feed not yet caught up) **MUST** be surfaced as
  *revocation-unverified*, never silently treated as valid (OFFLINE R-GRADE-1/2). A safety-critical
  verifier (KYC gate, licence check) **SHOULD** fail closed until status is confirmed, and — where
  it needs *hard* revocation — **MUST** re-fetch live status at decision time (S-ATT-2) rather than
  trust a cached copy; offline, it **MUST** disclose it is trusting stale status.
- **Reconcile on reconnect** is ordinary feed catch-up (§22.4) plus a status-list refresh: pull the
  issuer's `FeedHead`, walk new entries, re-evaluate `supersedes` and status bits. It is
  idempotent and order-independent; a discovered supersession or a flipped status bit simply lowers
  the standing of an already-verified object — no clawback, no coordinator refereeing (OFFLINE §4).
  A discovered **equivocation** is surfaced as evidence (§4.3), not smoothed over.

---

## 8. Security MUSTs

Inheriting [`THREAT-MODEL.md`](../THREAT-MODEL.md):

- **S-ATT-1 (intrinsic authenticity — SEC-2).** Acceptance rests **only** on a valid **carrier**
  signature (`PubAnnounce.sig` / `Payload.sig`, §2) chaining to `issuer` and a self-consistent
  content address, never on transport trust. A `terminating` intermediary can read a public
  attestation but **MUST NOT** be able to forge one.
- **S-ATT-2 (fail-closed — SEC-1).** Unknown `v`/`suite`, a broken `DeviceCert` chain, a revoked
  signer, a subject-commitment that does not open, or unverifiable revocation status ⇒ **reject or
  mark unverified**, never accept-by-default.
- **S-ATT-3 (replay-inert, downgrade-impossible — SEC-8).** The carrier's DS-tagged signing
  preimage (`DMTAP-PUB-v0/announce` / `DMTAP-v0/payload`, §2) covers the entire embedded
  `Attestation` map and so binds `issuer`, `subject`, `schema`, and `ts`; a captured attestation
  cannot be re-bound to a different subject, issuer, or schema. An attestation
  carries no bare re-usable bearer token.
- **S-ATT-4 (issuer-key compromise — SEC-5).** Issuer keys use account-abstraction recovery /
  `DeviceCert` rotation (§1); a compromised operational subkey is revoked via `DeviceCert`
  revocation, and attestations under a revoked-then-superseded key are re-evaluable on reconnect.
  No single device can unilaterally destroy an issuer's standing.
- **S-ATT-5 (sealed-carrier privacy — SEC-9).** A private (`0x00`/`0x01` sealed content MOTE, §2)
  attestation is sealed to the subject; intermediaries see routing metadata only, and MAY use a
  **`BlindedSubject`** commitment so the subject is not in the clear even to the holder's storage.
  Public (`0x40`) attestations are public **by design** (§22.9) — that is the point, not a leak.
- **S-ATT-6 (coordinator visibility declared — SEC-4).** Any `oracle` / `indexer` in an attestation
  path **MUST** declare its visibility class and assurance level and clients **MUST** surface it
  ([CONTRACT §3](../coordinator/CONTRACT.md)); a `declared`-level blind claim is never shown as
  verified.

---

## 9. Honest residual

Every residual here traces to the root ceilings of [DIRECTION §8](../DIRECTION.md); none is
hidden.

- **Issuer trust is exogenous.** ATTEST proves *`I` said this*, never that `I` is trustworthy or
  that the claim is true. An attestation is only as good as the verifier's independent judgement of
  the issuer. The substrate deliberately does not import that judgement (that would be a global
  score — forbidden).
- **The physical-event oracle cannot prove non-fabrication** ([§8.2](../DIRECTION.md)). "Did it
  happen?" reduces to dual-confirm + dispute; an oracle attests *origin-through-itself*, never the
  underlying reality. ESCROW/DISPUTE bound the damage; they do not close the gap.
- **Personhood is imperfect** ([§8.1](../DIRECTION.md)). Every method trades off — biometric +
  operator, or passport-zk that excludes the undocumented. World ID / Human Passport raise the
  Sybil floor; they do not close it. Local scale substitutes web-of-trust, which is weaker in
  coverage and stronger in context.
- **Revocation is cooperative** (R-ATT-4, R-ATT-OFF-1) — no hard clawback of a claim already
  relied upon; a verifier that already cached and acted on an attestation, or a `terminating`
  party that copied it, is outside protocol reach — the same residual as review retraction and
  erasure (TRACT §10.2d, §22.6).
- **The sealed (private) carrier has no cross-recipient equivocation detection.** §4.3's
  equivocation-surfacing property holds only for the **public** feed carrier, because it depends on
  the issuer's append-only author feed (§22.4.2). A private attestation delivered as a sealed
  `0x00`/`0x01` content MOTE (§2) is on no feed: a malicious issuer can hand two recipients
  contradictory sealed claims and no single verifier ever holds both to detect it.
- **`declared`-level blindness is unprovable** ([CONTRACT §3.4](../coordinator/CONTRACT.md)) — a
  `terminating` oracle promising it does not log cannot be cryptographically disproven; only
  `structural`/`attested` are verifiable.
- **Maturity is a 2026-07 snapshot.** EAS/VC plumbing is mature; the portable-reputation and
  selective-disclosure layers over it are nascent ([bindings](../bindings/README.md)). Re-check
  before relying on any binding in production.

These are the disclosed cost of not being a single surveilling authority that simply *declares*
who is real and who is credible. ATTEST gives the mechanism; it discloses the trust.
