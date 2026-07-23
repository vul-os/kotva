# OFFER — the supply primitive

> **Status:** primitive spec (KOTVA family). Normative once ratified. OFFER is the first of the
> six primitives named in [DIRECTION §2](../DIRECTION.md) (`OFFER · MATCH · RESERVE · REPUTATION ·
> ESCROW · ATTEST`). It is the **supply side** — the public listing that MATCH
> pairs against, that RESERVE holds against, that ESCROW settles — the identical shape also serves
> the demand side as a demand offer / bid (§2). It defines **no new bytes**: a
> listing is a DMTAP-PUB object ([§22](../22-public-objects.md)) carrying a profile-defined shape;
> this document states the primitive-level rules every commerce/gig/classifieds profile inherits.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, **OPTIONAL** are to be interpreted as in BCP 14 (RFC 2119, RFC 8174).

---

## 1. Purpose

An **offer** is a signed, public, content-addressed claim by one identity to supply one thing on
stated terms. It is the atom every discovery-and-trade service is built from: Uber, delivery,
freelance, auctions, bookings, and classifieds all publish offers and differ only in what *consumes*
them (MATCH's assignment rule, or RESERVE's single-owner hold) — [DIRECTION §2](../DIRECTION.md).
OFFER supplies the object — in either direction, supply or demand (§2) — the other primitives
supply the verbs.

OFFER carries the cross-cutting family invariants: it **carries attestations, never funds**; it
**bears no network-wide score and no protocol token**; and it **works offline against local trust**,
a coordinator adding only reach ([DIRECTION §0/§5/§6](../DIRECTION.md)). An offer is **public by
design** — the inverse of sealed sender. The publisher's identity is the point, not a secret
([§22.1](../22-public-objects.md)).

The one structural move OFFER makes is the **Product ≠ Offer split** (§2). Everything else —
availability, fulfilment, pricing — is an axis the split hangs off, owned by profiles and referenced
here, never re-specified.

---

## 2. Objects — the Product ≠ Offer split (four axes)

OFFER defines a **shape**, not a wire kind. On the wire an offer is a DMTAP-PUB
`PubAnnounce` (kind `0x40`, [§22.3](../22-public-objects.md)) whose `meta`/`roots` carry the
profile-defined `Offer` grammar (e.g. tract [`16-wire-format.md §16.5.2`](../profiles/tract/16-wire-format.md));
a product description is a public blob (`PubManifest`, [§22.2](../22-public-objects.md)) or an
announced record ([tract §16.5.1](../profiles/tract/16-wire-format.md)). The primitive owns the
**separation** and the **four axes**; the bytes are DMTAP-PUB's and the profile's.

**The split (normative, [tract §2.2](../profiles/tract/02-catalogue.md)).** A **product record**
describes *what a thing is*; an **offer** is *one seller's claim to supply it*. They are separate
objects. A record belongs to nobody: it names no seller, price, or availability, and two publishers
holding the same bytes converge on the **same content address** by plaintext addressing
([§22.2.2](../22-public-objects.md)) — the global product view is an emergent property of hashing,
not a registry. An offer names its seller (by feed position, §3) and *references* a record by content
address.

A CDDL-ish sketch of the **abstract** primitive shape (the profile freezes the exact bytes):

```cddl
; A product record — belongs to nobody, converges by content address.
;   Carried as a PubManifest blob or an announced record; NO seller, NO price, NO stock.
ProductRef = hash            ; content address of the record (§22.2), or a {group, variant} pair

; An offer — one identity's claim to supply one product on four axes.
;   Carried inside a PubAnnounce (kind 0x40); authenticity is the feed's (§3), not a field here.
Offer = {
  item          : ProductRef,     ; WHAT is supplied — by content address, never inlined
  availability  : Availability,   ; WHEN / HOW MUCH — a published SIGNAL, not a hold (§3, RESERVE owns holds)
  fulfilment    : Fulfilment,     ; HOW it reaches the buyer (ship / pickup / digital / perform)
  consideration : Consideration,  ; WHAT is asked in return — priced in an existing asset, never a token
}
```

The **four axes** are `item · availability · fulfilment · consideration`. An offer MUST carry all
four, **in either of two directions**: a **supply offer** (the default direction described above:
what is offered, when/how much is available, how it ships, what it costs) or a **demand offer /
bid** — a want-ad, itself an OFFER with the same four fields and the sense inverted: `item` is what
is wanted, `availability` is when/how much is wanted, `fulfilment` is how the requester will take
delivery, and `consideration` is what the requester will pay. A demand offer is still an OFFER; it
is not MATCH's `MatchDemand`, which is a distinct coordination object that *references* a demand
offer by content address, exactly as its `Candidate.offer` references a supply offer ([§4](#4-composition-with-the-other-primitives)).
Their detailed semantics are **owned by profiles and referenced, never duplicated**:
availability/stock bands in [tract §3](../profiles/tract/03-availability.md), fulfilment modes in
tract §4, pricing/tax in tract §5. OFFER fixes only that the four exist, that `item` is a content
reference and never an inlined description, and that `availability` is a *signal that commits no
stock* — the boundary between OFFER (a signal) and RESERVE (the only writer that holds stock) is a
category line the whole primitive is written to keep ([tract §3.1/§3.9](../profiles/tract/03-availability.md)).

---

## 3. Normative rules

- **OFR-1 — separation.** An implementation MUST keep product records and offers as separate
  objects. It MUST NOT fold price, stock, or seller identity into a product record, and MUST NOT
  treat an offer as a shareable description that could converge across sellers. Two sellers supplying
  the identical product publish **one** shared record and **two** distinct offers
  ([tract §2.2](../profiles/tract/02-catalogue.md)).
- **OFR-2 — content-referenced item.** An offer's `item` MUST reference its product by content
  address (§22.2), not inline it. This is what makes convergence and cross-seller price comparison
  fall out of hashing rather than a registry.
- **OFR-3 — authored, addressed by `(IK, content-address)`.** An offer MUST be authored by a
  sovereign identity `IK` ([§1](../01-identity.md)) and self-authenticating: verifiable offline that
  `IK` published exactly these bytes, with zero DNS and zero name-chain
  ([§22.3.3](../22-public-objects.md)). Order and anti-rollback come from the author feed
  ([§22.4](../22-public-objects.md)), never from the bare object.
- **OFR-4 — availability is a signal, not a hold.** An `availability` value MUST NOT be read or
  presented as a reservation; it commits no stock. A hold is RESERVE's single-writer bounded counter
  ([DIRECTION §2](../DIRECTION.md), [tract §3.9](../profiles/tract/03-availability.md)), never OFFER's.
- **OFR-5 — withdrawal is supersede, never delete.** An offer is immutable and irrevocable
  ([§22.7/§22.9](../22-public-objects.md)). Withdrawing, repricing, or marking sold-out MUST be a
  **successor announcement** via `supersedes` (§22.3.4), not a mutation or a deletion. There is no
  protocol takedown; a publisher can only publish a correction.
- **OFR-6 — no personal data in a product record.** A product record is a public object and MUST NOT
  carry any personal data (person name, address, contact) ([tract §2.3](../profiles/tract/02-catalogue.md)).
  An offer names its seller only as an identity key by feed position — never as PII.
- **OFR-7 — indexes are derived, never authoritative.** Any search, category, or price index over
  offers is rebuildable and MUST NOT be treated as an authority. On any disagreement between an index
  and a seller's signed feed, resolution MUST prefer the feed ([§22.4.3](../22-public-objects.md),
  [tract §2.6](../profiles/tract/02-catalogue.md)). No index can revoke a content address or delist a
  seller from the network; it can only decline to list in its own view.
- **OFR-8 — priced in an existing asset, no token.** `consideration` MUST be denominated in an
  existing asset (a stablecoin or fiat); OFFER mints nothing and references no protocol token
  ([DIRECTION §5](../DIRECTION.md)). Settlement is PAY/ESCROW's job on the settlement rail, not a
  field OFFER moves value in.
- **OFR-9 — no global score on the object.** An offer MUST NOT carry a network-wide reputation or
  ranking number. Trust in a seller is REPUTATION's, locally measured or computed over public
  attestation feeds ([bindings](../bindings/README.md), OpenRank) — never a figure baked into the
  listing ([coordinator/CONTRACT §2.1](../coordinator/CONTRACT.md)).
- **OFR-10 — profiles own the axes.** A profile MUST define the concrete `Availability`,
  `Fulfilment`, and `Consideration` grammars; this document MUST NOT be read as freezing them. A
  reader MUST ignore `meta` keys it does not recognise (forward-compat, [§22.3.1](../22-public-objects.md)).

---

## 4. Composition with the other primitives

OFFER is the supply object every other primitive reads or writes against:

| Primitive | Composition with OFFER |
|---|---|
| **MATCH** | Consumes two offers (a supply offer + a demand offer / bid) and emits a signed `Assignment`. The assignment rule (nearest / highest-bid / best-fit) is *matcher policy*, not an OFFER field ([DIRECTION §2](../DIRECTION.md)). |
| **RESERVE** | Holds against an offer's advertised availability. OFFER *signals* stock; RESERVE is the single-writer that *holds* it, so double-booking is structurally impossible. OFR-4 keeps the two from being confused. |
| **REPUTATION** | Attaches to the offer's *author* `IK`, not to the offer. Reviews/attestations are separate signed feed objects keyed to the seller identity; no number rides in the offer (OFR-9). |
| **ESCROW / PAY** | Settle the `consideration` on the rail after a match. OFFER carries the *ask*; the escrow scope and payment attestation are their own objects — attestations on the wire, funds on the rail. |
| **ATTEST / ORACLE** | A product record's identity ladder (manufacturer-signed rung, [tract §2.3c](../profiles/tract/02-catalogue.md)) is an ATTEST claim; "did it arrive?" is an ORACLE/dispute question OFFER neither asks nor answers. |

Substrate composition: an offer **is** a DMTAP-PUB author-feed object ([§22.4](../22-public-objects.md)),
so it inherits PUB's trustless serving, revision chains, and monotonic-`seq` anti-rollback unchanged.
Product blobs inherit the public-blob profile ([§22.2](../22-public-objects.md)) with global dedup.

---

## 5. Binding adopted

Per [DIRECTION §3](../DIRECTION.md) and [`bindings/README.md`](../bindings/README.md), OFFER binds
rather than invents:

- **Listing carrier** — DMTAP-PUB signed public objects + author feeds ([§22](../22-public-objects.md),
  [substrate/FEEDS.md](../substrate/FEEDS.md)). The offer *is* a `PubAnnounce`; the feed gives order,
  discovery, and anti-rollback. No new object model.
- **Product data model** — schema.org `Product`/`Offer`/`ProductGroup` vocabulary and GS1 (GTIN/MPN)
  identifiers as **claims only**, so existing merchant feeds map in by translation
  ([tract §2.4](../profiles/tract/02-catalogue.md)). KOTVA specifies no product vocabulary of its own.
- **Discovery coordinator** — the `indexer` role ([coordinator/CONTRACT §5](../coordinator/CONTRACT.md)):
  corpus `public` (offers are public objects; indexing them reads nothing secret) and query-channel
  `terminating` unless `attested` (TEE-preferred), so a searcher's own query need not be read in the
  clear. An indexer **authorizes, never classifies** (CONTRACT §4): it may rank and filter its *own*
  view, never revoke or delist from the network.
- **Reputation / settlement / dispute** — bound out to REPUTATION (OpenRank), PAY (x402 +
  stablecoins), ESCROW (multisig / HTLC / smart-contract, or a licensed custodial operator),
  DISPUTE (Kleros-class) — all via their own primitives; OFFER holds none of them.

No binding introduces a protocol token, a global score, or a surveillance signal — forbidden by
[DIRECTION §5](../DIRECTION.md) and [`bindings/README.md`](../bindings/README.md).

---

## 6. Scale-invariance — mesh web-of-trust ↔ global coordinator

OFFER is identical at every scale; only the **discovery anchor** slides
([DIRECTION §6](../DIRECTION.md)):

| Function | Small / mesh (offline, no coordinator) | Global (swappable coordinator) |
|---|---|---|
| Discovery | following-graph + local index over feeds you already pull | competing `indexer` coordinators |
| Seller trust | web-of-trust — you know these sellers | REPUTATION over the attestation graph |
| Price view | offers in your local corpus | indexer's cross-seller price aggregation |

The object never changes: the *same* signed offer verifies identically whether it arrived over the
mesh, HTTPS, or an SD card. An indexer adds **reach**, not authority (OFR-7) — remove it and every
offer still verifies and trades against local trust; add it and global discovery becomes *available*,
never *required*. This is the coordinator-optional property applied to supply.

---

## 7. Offline / apocalypse behaviour + reconcile

Per [substrate/OFFLINE.md](../substrate/OFFLINE.md), each OFFER action classifies into exactly one
degradation grade, with **no silent degradation** and **no fabricated completion**:

| Action | Grade | Offline behaviour |
|---|---|---|
| Author / withdraw (supersede) an offer | **`full`** | Local-first: the offer and its feed entry are self-authenticating, created and signed with no network. |
| Distribute an authored/withdrawn offer to peers | **`deferred`** | Signing is `full`, but reach to buyers and indexers is `deferred` until a network path exists ([OFFLINE §3.3](../substrate/OFFLINE.md)) — a sold-out/withdrawal supersede MUST be surfaced as not-yet-reached, never presented as taken down. |
| Read / verify a peer's offer | **`full`** | Verifies from the object alone (`IK` + content address), zero-DNS ([§22.3.3](../22-public-objects.md)). |
| Global discovery (indexer) | **`local-trust`** → **`blocked`** | Degrades to a local index over the following-graph; with no network and no anchor, discovery beyond local corpus is `blocked` and MUST say so, never faked. |

**Reconcile on reconnect** is DMTAP-PUB feed catch-up: a reader fetches `feed_head(pub)`, applies
strict monotonic-`seq` anti-rollback ([§22.4.2](../22-public-objects.md)), and walks the `prev`
chain — idempotent and order-independent. Two offline endpoints converge from the feeds alone, with
no coordinator refereeing. Feed **equivocation** (two heads at one `seq`) surfaces as transferable
`ERR_PUB_FEED_CHAIN_BROKEN` evidence, never swallowed by a clean merge — the R-SYNC-1 rule
([OFFLINE §3](../substrate/OFFLINE.md)) that convergence must not hide a broken invariant.

Because OFFER only *signals* and never *holds* (OFR-4), it has no cross-replica scarcity invariant to
violate offline — the hard offline case (double-booking, offline money) belongs to RESERVE and to
[OFFLINE §5](../substrate/OFFLINE.md), not here. This is why OFFER is `full` where RESERVE is not.

---

## 8. Security MUSTs

Inheriting [THREAT-MODEL.md](../THREAT-MODEL.md) (SEC-1…SEC-9); the OFFER-specific obligations:

- **OFR-S1 (SEC-2, intrinsic authenticity).** Every offer MUST be verified — signature, `DeviceCert`
  chain to `IK`, and content-address recomputation — before use; a serving node is a convenience,
  never a trust root ([§22.5.1](../22-public-objects.md)). Fail closed (SEC-1) on any mismatch.
- **OFR-S2 (SEC-8, replay-inert / downgrade-impossible).** A reader MUST enforce the feed
  origination floor and monotonic-`seq` anti-rollback ([§22.3.3 step 1a, §22.4.2/§22.4.5](../22-public-objects.md)):
  a stale or below-floor `FeedHead` MUST NOT suppress an offer a seller has since published, nor
  displace the genuine feed.
- **OFR-S3 (SEC-6, authorize-never-classify).** An indexer over offers MUST NOT run content
  classification as a network gate; it authorizes from identity + rate only, and any moderation is an
  opt-in labeler the reader chooses ([coordinator/CONTRACT §4](../coordinator/CONTRACT.md)). Its
  visibility class MUST be declared (SEC-4).
- **OFR-S4 (SEC-7, Sybil priced-and-localized).** Spam/Sybil resistance for listings is **index-local**
  — priced at the indexer (postage/rate-limit), never a network-wide authority. There is no global
  offer registry to poison; a poisoned index is one index, swappable (OFR-7).
- **OFR-S5 (privacy scope).** An offer is public by design — publisher, listed set, and timestamps
  are exposed on purpose ([§22.1.2](../22-public-objects.md)). An implementation MUST NOT publish as
  an offer any content the user did not explicitly publish ([§22.2.4](../22-public-objects.md)); the
  publish act is the sole, irrevocable private→public gate.

---

## 9. Honest residual

- **No discovery without an indexer.** A content-addressed substrate offers no global index, so
  discovery is the *first* function to re-centralize: whichever indexer becomes economically dominant
  becomes a de-facto content-policy gatekeeper *regardless of what OFR-7 permits*. Multiple competing
  indexers with verifiable completeness is the candidate answer and has **no deployed precedent**
  ([tract §2.6/§21](../profiles/tract/02-catalogue.md), [DIRECTION §8.4](../DIRECTION.md) editorial
  governance). Marked as the weakest claim, not defended.
- **Cross-publisher product identity is a mechanism carrying an unproven claim.** Plaintext
  content-addressing converges *identical bytes*; making two shops' descriptions of the same shoe
  converge rests on canonicalisation (and near-duplicate merge, which is an open index heuristic).
  No deployed permissionless system achieves this without a licensed registry
  ([tract §2.2a](../profiles/tract/02-catalogue.md)).
- **Spam/Sybil floor is local, not solved.** OFR-S4 confines abuse to an index; it does not raise the
  global anti-Sybil floor, which stays imperfect ([DIRECTION §8.1](../DIRECTION.md)). Local scale
  dissolves it into web-of-trust; global scale prices it, and pricing is bounded by imperfect
  personhood ([bindings](../bindings/README.md), World ID / Human Passport, `adopt (ceiling)`).
- **Irrevocability cuts both ways.** OFR-5's supersede-not-delete is the only honest model under a
  substrate with no takedown, but it means a mistaken or stale offer is permanent history; a reader
  showing the latest revision is a client convention, not a protocol guarantee that the predecessor
  is unreachable ([§22.9](../22-public-objects.md)).

Every residual traces to one of the four root ceilings ([DIRECTION §8](../DIRECTION.md)): the
discovery-gatekeeper and cross-publisher-identity residuals are **editorial governance**; the
Sybil floor is **global anti-Sybil**. None is a bug in OFFER; each is a consequence of not being a
single surveilling company, and is disclosed rather than solved. Maturity/precedent claims are a
2026-07 snapshot ([docs/research/README §6](../docs/research/README.md)).
