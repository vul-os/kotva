# 22. Erasure — the hardest unresolved problem in this design

> **Drafting status.** This section is **normative** for the *structural* resolution of the erasure
> conflict and for the deployment requirements that follow from it, and **honest-but-scoped** for
> every legal question it cannot answer. Carrying RFC 2119 keywords aligned to the frozen
> [`16-wire-format.md`](16-wire-format.md): the prohibition on personal data in the public quadrant
> and its fail-closed enforcement (§22.3, the home of the rule §0.5.1/§11.5/§16.4 also carry);
> orders and everything identifying a person sealed and deletable at the two endpoints (§22.3);
> reviews as the single bounded exception, with cooperative-only retraction and pre-publication
> disclosure (§22.6, referencing §10.4); and the requirements a deployment MUST/SHOULD meet today
> (§22.7). **Still scoped, not normative, and marked PROVISIONAL or Open inline:** every legal
> question — crypto-shredding-as-erasure (§22.4.1), controllership and the household exemption
> (§22.5), and the whole of §22.8 — which need qualified legal advice per jurisdiction, not more
> protocol text (§22.8.8, §21.11). The structural resolution MUST NOT be read as a compliance
> guarantee: GDPR Art 17 against immutable objects remains the most likely hard blocker on the whole
> design and survived three §21 passes unanswered (§21.11). The key words MUST, MUST NOT, REQUIRED,
> SHOULD, SHOULD NOT and MAY are to be interpreted as in BCP 14 (RFC 2119, RFC 8174) where they
> appear.

## 22.1 Scope

What happens when a data-protection erasure right — GDPR Art 17, POPIA section 24, LGPD Art 18 — is
asserted against a TRACT object, or against the fact that one was ever published. This section
**resolves the conflict structurally** — it keeps personal data out of the irrevocable family in
the first place, and that resolution is normative (§22.3) — and then works through the residual that
structure leaves, the mechanisms available for it, and what is still open after three research
passes found nothing (§21.11). The structural resolution is a mitigation, not a proof of compliance;
the difference is stated plainly throughout and MUST NOT be collapsed.

This section owns the erasure-rights analysis for the whole document; §0.5.1 states the rule, §11.5
states its wire consequence, §16.4 enforces it in the grammar, and §10.4 bounds the one exception —
each of those references here.

## 22.2 The conflict, stated precisely

A published TRACT object is content-addressed: its address is a hash of its bytes, and any node
that holds those bytes may serve them to anyone who asks, forever, without asking the publisher
first (§0.4.1, §2.6). Content addressing, feeds and blobs are the substrate's, not TRACT's, and this
section does not re-specify them — see
[`FEEDS.md`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md). That irrevocability is
not an accident of implementation — it is the entire mechanism by which two sellers' catalogues
converge, an index rebuilds itself from nothing, and a cache or a pin keeps a sleeping seller's
storefront answerable (§1.7, §21.5). The same property that makes the network usable without an
operator makes the object impossible to recall once it exists.

"Delete it" is not available against an object like that, for a structural reason rather than a
policy one: there is no single custodian to instruct. A conventional erasure request names a
controller who holds one copy in one system and can remove it. Here, by design, the set of parties
who may hold a copy is unenumerable — any cache, any pin, any index that snapshotted the feed, any
buyer's client that fetched it before a retraction, any node nobody involved in the request has
ever heard of (§0.4.1's cache/pin role requires nothing but willingness to hold bytes; there is no
registry of who has). A right that presupposes a reachable custodian has no target here that can
satisfy it the way the statute contemplates. This is not a claim that the right does not apply; it
is a claim that the object it would be asserted against was built to survive exactly the kind of
removal the right asks for.

## 22.3 The structural resolution (normative)

§0.5.1 does not attempt deletion. It refuses the premise: **no personal data enters the public
quadrant at all.** The resolution is enforced by the §16 grammar, not by reviewer discipline
(§16.4), and this section states it as the normative rule the rest of the document references:

- **No personal data in the public quadrant.** An implementation MUST NOT publish, content-address,
  or serve any public-family object — `ProductRecord`, `Offer`, `RateCard`, `CapacityRecord`,
  `EscrowScope`, `Review` metadata, or a storefront render bundle — that carries a name, street
  address, contact detail, or any other field that identifies or is linkable to a natural person
  (§0.5.1, §16.4). There is no street-address production in the public family at all: a public
  object that references a place carries a country and a coarse locality only (`PlaceRef`,
  `CapacityRecord`; §16.5.2, §16.5.3). A decode or publish that violates this MUST fail closed with
  `ERR_TRACT_PERSONAL_DATA_PUBLIC` (§17, `0x0102`).
- **Personal data is sealed, and deletable at the edges.** A buyer's name, delivery address, and
  contact details, and the `Order` that carries them, MUST be carried in the sealed family (§16.6)
  and MUST have no production in the public family. A sealed order exists at exactly the two
  endpoints (§22.5), where an implementation MUST be able to delete its own held copy on request —
  deletion at an endpoint is erasure in the ordinary sense, because there, for once, the custodian
  is countable.
- **The two families are non-confusable.** A sealed object presented under a public-object schema,
  or a public object under a sealed one, MUST be rejected with `ERR_TRACT_TYPE_CONFUSION`
  (§17, `0x0101`) and MUST NOT be coerced (§16.4). The families are separated at the content-address
  level by distinct domain-separation tags, not by a droppable `sealed: true` flag.

This is a stronger position than any deletion mechanism could offer, and the reason is worth
stating plainly: a system cannot be asked to erase what it never published. Deletion mechanisms are
always reactive — they reach into a system that already let something escape and try to remove it,
against every copy that may already exist. Structural exclusion is not reactive; there is nothing
that escaped, because the grammar never accepted it. §16.4 exists precisely so this is enforced by
what a decoder can encode, not by a publisher's discipline or a reviewer's attention — an object
that structurally cannot carry a name cannot leak one by someone forgetting a check.

The defence is complete only where it is total, and that is the honest qualifier. §0.5.1's value
depends on there being no exception, because a single field that lets personal data into a public,
irrevocable object is not "mostly" fixed by a later mechanism — the object is irrevocable from the
moment it is published, and a residual mechanism applied afterward inherits every limit described
below. §0.5.1 has exactly one acknowledged exception (§22.6), and this section exists because of it.

> **Honest limit (§21.11, §11.5).** Structural avoidance is a **mitigation, not a compliance
> guarantee.** Keeping personal data out of the irrevocable family removes the largest surface of
> the conflict; it does not prove the conflict resolved for every object a regulator might reach,
> and it says nothing about the legal questions §22.4.1 and §22.5 leave open. GDPR Art 17 against
> immutable published objects is the most likely hard blocker in the whole design and survived three
> §21 passes unanswered. This section MUST NOT be read as asserting compliance by construction.

## 22.4 Candidate mechanisms for the residual

TRACT v0 defines **none** of the three mechanisms below as a protocol mechanism. They are discussed
here to explain why the structural answer of §22.3 — not a residual mechanism bolted on afterward —
is and must remain the primary defence. None of the three is a solution; each is a partial answer to
a narrower question, and each has a limit that does not go away with better engineering.

| Mechanism | What it actually does | What it does not do |
|---|---|---|
| Crypto-shredding | destroys future ability to decrypt, for everyone, once the key is gone | does not remove the ciphertext bytes, and does not fit the public quadrant's plaintext-serving model |
| Tombstone / supersede | tells conformant participants to stop showing the object | binds only participants who look for it and choose to honour it |
| Off-chain payload | removes the content at its source once the source is willing | leaves the address, and the address is not nothing |

### 22.4.1 Crypto-shredding

The idea: publish ciphertext at a content address instead of plaintext, keep the decryption key off
the public feed, and destroy the key when erasure is wanted. Once the last copy of the key is gone,
no holder of the ciphertext — however many independent holders there are — can ever recover the
plaintext again. That is a real and useful property, and it is the only one of the three mechanisms
that can, in principle, make previously-published content permanently unreadable rather than merely
harder to find.

It does not fit cleanly into TRACT as designed. The public quadrant exists to be computed over —
canonicalised, matched, indexed, ranked (§2.2, §2.6) — and a ciphertext blob nobody but the
key-holder can read is not a catalogue entry anyone can index; it defeats the reason objects are
public in the first place. Applying crypto-shredding to product records, offers or rate cards would
mean publishing objects that are structurally public but functionally private, which is a category
neither §16.4's two type families nor §2's canonicalisation rules were built to hold, and which
**would require a §16 grammar change TRACT has not made** — a public ciphertext-blob production does
not exist in the frozen v0 grammar, and inventing one is a MAJOR version change, not a correction
(§16). On the sealed side — orders — the mechanism is not needed: both endpoints already hold their
own copy and can delete it directly (§22.3, §16.6); there is no third party's inaccessible
ciphertext to worry about, because sealed objects were never replicated past two parties to begin
with.

Where the question is not academic is the general one, independent of whether TRACT ever adopts the
mechanism: **does destroying the key satisfy an erasure right, or does it only make the data
permanently inaccessible without deleting it?** Those are not the same claim. A regulator or court
could read "erasure" as requiring the referenced data cease to exist, in which case a permanently
unreadable ciphertext blob sitting at a stable address does not satisfy it, however inaccessible it
is. Or it could accept irrecoverability as functionally equivalent to erasure, on the reasoning that
data nobody can ever read again is not meaningfully different from data that does not exist.

> **PROVISIONAL — pending decision (§22.8.1, §22.8.6).** This document takes no position, because
> none of the three research passes behind §21 found an answer either way, and the question has
> never been tested against a network where the ciphertext itself is replicated beyond the
> publisher's reach by design (§21.11). It is recorded as unresolved, not because it was overlooked,
> but because it was looked for and not found. TRACT v0 therefore defines no crypto-shredding
> mechanism for any object; whether it ever has a place — and if so, only as voluntary
> defence-in-depth on the sealed side, where the problem it would solve does not exist — is an open
> founder-and-counsel call (§22.8.6), not something this section decides.

### 22.4.2 Tombstones and supersede markers

TRACT already uses this pattern twice: an offer that no longer stands is superseded, not deleted
(§18.2), and a review's retraction is a superseding tombstone honoured by conformant clients and
gateways (§10.2d, §10.4). The mechanism is a later signed object that says "the thing at this
address no longer represents the publisher's current position," and every participant that checks
for one before serving or displaying the original object will show the tombstone instead.

**"Cooperative-only" is the honest description, and it is normative:** a tombstone changes what a
conformant implementation shows and MUST NOT be presented — in documentation, in a UI, or to a data
subject — as erasure or as a capability enforced by construction. It changes nothing about what
bytes exist. A holder that serves whatever is at an address without checking the feed head for a
later entry, an index that snapshotted before the supersede was published, a cache that pinned the
object independently of the feed it came from, or a party running software that simply does not
implement retraction — none of these is violating the protocol, because nothing in the wire format
obliges a byte-holder to notice a later message, and nothing can make it notice one. §18.2 already
states this for offers — "withdrawal is not deletion" — and the same sentence applies here without
qualification. Where §22.3's grammar-level prohibition is strong because it never lets the object
into existence, a tombstone is weaker in exactly the way any after-the-fact mechanism is weaker than
never publishing at all.

Whether a supersede/tombstone should ever be given protocol-level *teeth* — a propagation obligation
on indexes and caches — is Open (§22.8.5), and the reason it is open rather than settled is that
enforcing propagation requires exactly the coordinating authority this design refuses to introduce
anywhere else.

### 22.4.3 Off-chain or out-of-band payload

The idea: keep only the address in the public, replicated part of the system, and hold the actual
payload somewhere the publisher controls directly — a private server, a bucket with access control
— so that removing the payload at its source means the address no longer resolves to anything new.
TRACT does not adopt this, for two reasons.

First, it does not sit comfortably inside a content-addressing model, where the address *is* the
hash of the content: moving the payload without changing the address requires a level of indirection
— a pointer object that resolves an address to a location rather than the content itself — that
TRACT does not otherwise have, and introducing one reintroduces exactly the kind of authoritative,
centrally-resolved reference this design avoids everywhere else.

Second, and more importantly, **the address itself is not nothing**, even once the payload behind it
is gone:

- **It proves an object once existed at that address**, and anyone who cached the payload before it
  was withdrawn still holds it — the address does not un-happen, and any independent holder from
  §22.4.2's discussion above may still be serving the bytes regardless of what happens at the
  source.
- **It is a join key wherever it is referenced elsewhere.** A product record's `group` or
  `components` fields, a review's `Subject`, a purchase attestation's order reference (§16.5.1,
  §16.5.5) all carry content addresses forward into other public objects. Removing a payload does
  not remove the addresses that point at it from objects that are themselves still public, so
  anyone holding both the reference and a cached copy of the referenced payload can still correlate
  them.
- **Feed position and timing leak on their own.** A feed's sequence and timestamps are public
  regardless of what the referenced payload said (§16.7), so the fact that some object existed, was
  published by a given key, at a given time, and was later withdrawn, is itself potentially
  identifying information that removing the payload does not touch.

## 22.5 Who is the controller

A sealed order exists at exactly two places: the buyer's node and the seller's node, each typically
a self-hosted box belonging to a different natural person (§0.4.1 — a seller needs nothing rarer
than "a keypair and a box that is up"; identity is an `IK` under
[`IDENTITY.md`](https://github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md), never an
organisational registration). Data-protection regimes built around a business processing customer
records assume a controller with an organisational identity distinct from the data subject. Here
that assumption may not hold, and this document does not know the answer to either question it
raises:

- **Is this joint controllership?** GDPR-style regimes have a concept for two or more parties who
  jointly determine the purposes and means of processing the same data, with obligations that
  follow — a transparent arrangement setting out each party's responsibilities being one of them.
  Whether that is the right frame for two strangers who "have never heard of each other" (§0.1),
  transact once, and each independently decide to create and retain their own copy of a shared
  sealed object, is not something this document asserts either way. It may be the correct legal
  category with no practical way to discharge its own duties between parties who never negotiate
  anything else about the trade; it may be the wrong category entirely. Both readings are guesses
  and are labelled as such rather than adopted.
- **Does a household-style exemption apply to a self-hosted seller?** A regime that exempts purely
  personal or household activity from its scope was not written with a keypair-identified seller
  running their own node for their own trade in mind. Where the boundary of that exemption sits for
  someone who is, in substance, conducting a commercial activity but has no formal business
  registration, no employees, and no infrastructure beyond their own machine, is not answered here.
  It is not answered anywhere this document's research reached (§21.11).

> **PROVISIONAL — pending decision (§22.8.2, §22.8.3).** Neither question was resolved by the pass
> that looked for an answer to who is a "facilitator" (§11.2a, §21.11) — that pass covered the
> gateway's position, not the two ordinary parties to a sealed order, and nobody has separately
> researched this one. These are questions for qualified counsel per jurisdiction (§22.8.8), not
> ones the protocol can answer in bytes.

## 22.6 Reviews — the one bounded exception, worked through

§10.4 already names the residual and carries the normative client requirements; this section works
through why the exception exists and what it does not close. A review is the one object in the public
quadrant that is, by design, signed by a natural person and meant to be recognisably the same person
across their own history with one seller — the entire value of a review is that it is attributable to
a real purchaser, which is in direct tension with keeping the author unidentifiable.

TRACT's mitigation is the per-subject pseudonymous subkey (§1.5, §16.5.5): `Review.author` MUST be a
key derived for, or generated for, that one seller relationship and MUST NOT be the buyer's root
`IK` (§16.5.5). Two sellers comparing their reviews cannot join them by author key; only the buyer,
holding the root key or the stored mapping, can compute the correspondence. Retraction MUST follow
the same supersede pattern as everything else public (§18.2, §10.2d): a later signed tombstone,
honoured by conformant holders, with exactly the cooperative-only limit stated normatively in
§22.4.2 — bytes persist wherever an independent holder kept them, and that residual MUST be disclosed
to the author **before** they publish, not discovered afterward (§10.4).

One point this document's own glossary already forces and is worth making explicit here: **a
pseudonymous key linkable to a person is still personal data** (§0.9). Pseudonymisation lowers risk;
it does not remove the datum from scope. This matters twice over for the review subkey specifically.
First, under deterministic derivation (§1.5), the unlinkability was never structural — it was the
difficulty of a computation that is zero for whoever holds the buyer's root key, which means a
compelled disclosure or a seized device recovers every per-seller subkey the buyer ever used, not
just this one review's author key. Second, even under the stored-random alternative, the subkey
remains, by the glossary's own definition, personal data for as long as the buyer — the only party
who can compute or has stored the correspondence — can be identified by it. Neither derivation
choice turns the review into something outside scope; both only change who can perform the linkage
and how expensively.

> **PROVISIONAL — pending decision.** Which subkey derivation the spec mandates — deterministic or
> stored-random — is an open founder call **owned by §1.5**, not decided here. §22.7 states which way
> this section's own reasoning leans (recoverable unlinkability over recoverable identity) without
> mandating it.

## 22.7 What a deployment MUST and SHOULD do today

None of this waits on a legal answer. Several requirements reduce the residual regardless of how
§22.5 and §22.4.1's open question eventually resolve, and a deployment MUST act on them now:

- **Publish less.** §16.4's grammar already refuses names and addresses in the public family;
  nothing in the free-text fields it cannot constrain — a product description, a review body — has
  to be filled with more than it needs. The grammar cannot stop a person typing an address into
  `Review.body` (§16.5.5 says so plainly); a client SHOULD warn, at the moment of the action, before
  it lets them, and SHOULD discourage placing identifying detail into any free-text field bound for
  the public quadrant.
- **Keep the review body small.** `MAX_REVIEW_BODY` is set at 8 KiB (§19.5), a limit stated there as
  serving privacy as much as resource bounding — the smaller the field, the less room there is to
  put identifying detail into a public, irrevocable object. §19.8 already flags this value as
  probably too large for the privacy purpose it is supposed to serve, chosen instead for
  usefulness, and recommends revisiting it in the other direction. This section agrees with that
  flag; the exact value is §19's call and is Open (§22.8.4), not decided here.
- **Prefer recoverable unlinkability over recoverable identity**, where an implementation has the
  choice §1.5 leaves open: a stored-random subkey mapping costs recovery convenience (losing the
  mapping severs continuity with a seller) but does not hand a compromised root key the ability to
  retroactively join a buyer's entire purchase history; a deterministic subkey costs exactly the
  reverse. Neither is mandated (the mandate is §1.5's PROVISIONAL call); a deployment MAY pick the
  trade-off that matches what it is more afraid of.
- **Disclose irrevocability before the act, not after.** This document already makes that disclosure
  a MUST in two specific places — a non-custodial rail's dispute deadlock disclosed before the trade
  (§18.5), and a review's residual disclosed to its author before publishing (§10.4). Generalising:
  any interface about to publish something into the public quadrant SHOULD state, at the moment of
  the action, that what is about to be published cannot be fully taken back — not surface that fact
  only once someone asks for it to be removed. For reviews specifically this SHOULD is already a MUST
  via §10.4.

None of this closes the gap in §22.5 or resolves the open question in §22.4.1. It shrinks the
residual that those unanswered questions would otherwise be asked to cover.

## 22.8 Open

This is, in the judgement of this document, the most likely hard blocker on the whole design, and
the list below is left long deliberately rather than tidied into false confidence. Each item is
collected for the founder-decision list; none is invented into a normative answer above.

1. **Whether destroying a crypto-shredding key satisfies a statutory erasure right, or only makes
   the referenced data permanently inaccessible without deleting it** (§22.4.1). Unresolved; not
   found by any of three research passes (§21.11).
2. **Whether joint controllership is the right frame for a sealed order held at two independently
   self-hosted nodes**, and if it is, how its arrangement duties could ever be discharged between
   two parties who transact once and have no other relationship (§22.5).
3. **Where, if anywhere, a household-style exemption applies to a self-hosted seller** with no
   formal business registration and no infrastructure beyond their own node (§22.5). Unresearched,
   not merely unsettled.
4. **Whether `MAX_REVIEW_BODY` should shrink further**, trading usefulness for a smaller residual,
   as §19.8 already flags and §22.7 endorses without deciding by how much. Owned by §19.
5. **Whether a supersede/tombstone mechanism should ever be given protocol-level teeth** — a
   propagation obligation on indexes and caches, for instance — or whether that is permanently out
   of reach because enforcing it requires exactly the coordinating authority this design refuses to
   introduce anywhere else (§22.4.2).
6. **Whether crypto-shredding has any legitimate place inside TRACT's model at all**, given that the
   public quadrant exists specifically to be served and computed over in plaintext, or whether it
   only ever applies as voluntary defence-in-depth on the sealed side, where the problem it would
   solve does not exist because both endpoints can already delete their own copy (§22.4.1). Adopting
   it for the public family would require a §16 grammar change (a public ciphertext-blob production
   that does not exist in the frozen v0 grammar).
7. **Whether trader-traceability duties that require identifying a seller** (§11.4) collide with
   minimising what is public, and if they do, which one this document should bend — that tension is
   named, not resolved, here.
8. **This section needs legal advice, not another literature pass.** Three research passes have now
   looked at the GDPR erasure conflict specifically and returned nothing verified each time
   (§21.11). That is not a gap this document can close by writing more carefully about it; it is a
   gap that closes, if it closes, with an opinion from someone qualified to give one in each
   jurisdiction this protocol is deployed into.
