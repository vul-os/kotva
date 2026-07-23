# Coverage matrix — every product shape against the primitives

**Status: non-normative research.** This is the long form of the coverage audit summarised in
[`README.md §1`](README.md). It exists to make one claim checkable: that the six-capability
waist ([DIRECTION §1](../../DIRECTION.md)) plus the six primitives ([DIRECTION §2](../../DIRECTION.md))
plus the one [coordinator contract](../../coordinator/CONTRACT.md) cover the product space —
and to name, without softening, the shapes they cover only partly and the *minimal* addition
that would close each gap. Nothing here is normative; the normative surface is the profiles
([`profiles/`](../../profiles/)), the substrate ([`substrate/`](../../substrate/)), and the
contract. Where a shape reduces to an existing profile, this document points at it rather than
re-specifying it ([DIRECTION §9](../../DIRECTION.md), "simple by subtraction").

The finding, stated first: of the 22 product shapes below, **16 are covered outright**, **4 are
covered with a disclosed ceiling** (anti-Sybil, physical-event oracle, legal issuer, or
editorial governance — the four roots of [`README.md §2`](README.md)), and **2 need a new
seam** — neither a new waist capability nor a new primitive, in every case an existing binding,
a new coordinator *kind*, or an unbuilt thin profile. One shape (surveillance advertising) is
**rejected by design**, not missing.

---

## 1. Legend

**Waist** ([DIRECTION §1](../../DIRECTION.md)) — `ID` identity · `MOTE` sealed object ·
`TRANS` transport · `PUB` public objects · `SYNC` multi-author CRDT · `WAKE` roles + content-free push.

**Primitives** ([DIRECTION §2](../../DIRECTION.md)) — the six are `OFFER` · `MATCH` (assignment:
nearest / best-fit / highest-bid) · `RESERVE` (single-owner calendar, no matcher) · `REP`
reputation · `ESC` escrow · `ATTEST` attestation. The matrix also uses three composite
coordinator/binding roles, not primitives (DIRECTION §2): `ORC` physical-event oracle (⊂
ATTEST) · `DIS` dispute (the arbiter kind) · `PAY` (the stablecoin binding).

**Coordinators** ([CONTRACT §5](../../coordinator/CONTRACT.md)) carry their declared
content-visibility ([CONTRACT §3](../../coordinator/CONTRACT.md)) inline: `blind` /
`blind-routing` / `terminating`. Every coordinator named here is an instance of the four
clauses (accountable · swappable · self-hostable · visibility-declared) and authorizes but
never classifies.

**Status** — `✔` covered · `▲` covered with a disclosed ceiling · `✚` needs a new seam (gap) ·
`✖` rejected by design.

---

## 2. The matrix

| # | Product shape | Waist | Primitives | Coordinators (visibility) | Profile | Status |
|---|---|---|---|---|---|---|
| 1 | **Mail** | MOTE · TRANS · WAKE | — (PAY opt: postage) | gateway (`terminating`, legacy SMTP) · relay (`blind`) · mailbox (`blind-routing`) | DMTAP §5 | ✔ |
| 2 | **Chat / groups** | MOTE(MLS) · SYNC · TRANS | — | relay (`blind`) · mailbox (`blind-routing`) | DMTAP §5 | ✔ |
| 3 | **Voice / video calls** | ID · MLS→SFrame · TRANS(signal) | — | media-relay + SFU (`blind-routing`/structural) | §27 realtime-media | ✔ |
| 4 | **Social feeds** | PUB · SYNC · ID | REP | indexer (`blind`/attested) · labeler (labels-only) | §25 pubsub / §22 | ▲ editorial |
| 5 | **Media — video / music** | PUB · storage · TRANS | PAY (subs, streaming) | indexer · media-relay (live, `blind-routing`) · storage (Walrus/Arweave) | §24 video / §22 | ✔ |
| 6 | **Ride-hailing** | ID · TRANS · SYNC | OFFER · MATCH(nearest) · REP · ESC · ORC · DIS · PAY | matcher (`blind`/attested) · oracle (`terminating`) · arbiter (`terminating`) | WRAP-derived *(unbuilt rides profile)* | ▲ oracle |
| 7 | **Delivery / courier** | ID · TRANS · SYNC | OFFER · MATCH(nearest) · REP · ESC · ORC · DIS · PAY | matcher (`blind`/attested) · oracle · arbiter | WRAP §12 (delivery) + TRACT §8 | ▲ oracle |
| 8 | **Bookings** (lodging / salon / restaurant) | PUB · SYNC · MOTE | OFFER · RESERVE · REP · ESC · PAY | none required (single-writer calendar) · gateway (escrow) opt | TRACT §3 availability | ✔ |
| 9 | **Auctions** | PUB · SYNC · MOTE | OFFER · MATCH(highest-bid) · REP · ESC · DIS · PAY | matcher (`attested`, sealed-bid) · arbiter | TRACT + open bid-book | ✔ (sealed-bid ✚) |
| 10 | **Freelance / labour** | SYNC · MOTE · PUB | OFFER · MATCH/assign · REP · ESC · ORC · DIS · PAY | matcher opt · arbiter · oracle | WRAP | ▲ oracle |
| 11 | **Commerce** | PUB · MOTE · SYNC | OFFER · ESC · REP · DIS · PAY | gateway (`terminating`, storefront + escrow) · indexer · arbiter | TRACT | ▲ editorial (discovery) |
| 12 | **Classifieds** | PUB · MOTE | OFFER · REP · (ESC/PAY opt) | indexer (`blind`/attested) | TRACT (thin) | ✔ |
| 13 | **Search / discovery** | PUB (corpus) | — | indexer (`blind`/attested TEE) | *coordinator, not a profile* | ▲ editorial + Sybil |
| 14 | **Maps** | PUB(tiles) · storage · WAKE(pin) | — | geo-indexer (`blind`/attested) *(geocode / POI)* | *unbuilt maps profile* | ✚ geo-query |
| 15 | **Storage / files** | MOTE(sealed) · PUB · SYNC · storage · WAKE(pin) | PAY (paid durability) | storage-provider (`blind`/`blind-routing`) · pin | DMTAP §5.5 file-share | ✔ |
| 16 | **Calendar / scheduling** | SYNC · MOTE · RESERVE | RESERVE | none | *unbuilt (SYNC + TRACT RESERVE)* | ✔ by composition |
| 17 | **Credentials / education** | attestation(EAS/VC) · PUB · MOTE | REP · PAY | issuer = attester · curation (registry) · TRACT/§24 for delivery | attestation binding + TRACT | ▲ authoritative-issuer |
| 18 | **Healthcare records** | MOTE(sealed, patient-held) · attestation · SYNC | REP · PAY | gateway (`terminating`, legacy-EHR bridge) opt | *unbuilt health profile* | ▲ legal-issuer + regulatory |
| 19 | **Notarization / registries** | PUB(timestamped) · storage(Arweave) · attestation | — | curation (labeler-class) · oracle/attester (real-world fact) | §22 PUB | ▲ editorial + authoritative-issuer |
| 20 | **Advertising** | PUB(sponsored obj) · indexer | OFFER · PAY | indexer (`blind`) — sponsored-listing only | §22 / TRACT | ✔ direct-buy / ✖ surveillance |
| 21 | **IoT / telemetry** | MOTE(light) · PUB · WAKE · SYNC | — | relay (`blind`) · mailbox (`blind-routing`) | *unbuilt constrained-device profile* | ✚ constrained crypto |
| 22 | **Private-AI** | box (self-host) · PUB(weights) · storage | OFFER(compute) · PAY(x402) | **compute** *(provisional kind, CONTRACT §5)* — `terminating` / `attested` (TEE) for blind | TRACT/WRAP for compute-OFFER | self-host ✔ / hosted-blind ✚ |

---

## 3. Why the covered rows collapse into so few mechanisms

The matrix's density is the point. Rows 6–12 (rides, delivery, bookings, auctions, freelance,
commerce, classifieds) are **one system wearing seven UIs** — they differ only in MATCH's
assignment rule or in dropping MATCH for RESERVE ([DIRECTION §2](../../DIRECTION.md)). Rows 1–3
are the substrate's two modalities directly: async MOTE (mail, chat) and the parallel real-time
media plane (calls, [DIRECTION §7](../../DIRECTION.md)). Rows 4–5, 13, 19–20 are all **one
object model** — a signed public object plus a derived, rebuildable, never-authoritative index
([§22](../../22-public-objects.md), TRACT §2.6). Building the primitives once builds the space;
that is the substrate claim, and the matrix is its ledger.

Every intermediary above is a coordinator under the same four clauses, so "some centralization"
stays a *checkable property*: each is hired-not-depended-on, swaps with a config change, has a
self-host backstop, and declares its visibility. Remove connectivity and each row degrades to
its local-trust form ([DIRECTION §6](../../DIRECTION.md)) — a local order book instead of a
global matcher, web-of-trust instead of a personhood attester, a known local arbiter instead of
a staked market — and *still works*. That is the apocalypse-proof property, row by row.

---

## 4. The gaps — explicit, with the minimal fix

Only two rows lack any covering mechanism, and three more are covered but partial. In no case
is the fix a new waist capability or a new primitive — each is an existing binding, one new
coordinator *kind*, or an unbuilt thin profile. This is the "future-proof by seams"
discipline ([DIRECTION §9](../../DIRECTION.md)) doing its job: the hard part is always already
a pluggable slot.

### G1 — IoT / constrained devices *(✚ gap; row 21)*
**What's missing.** The waist fits (a device is a keypair; telemetry is PUB; a wake is
content-free), but the full MOTE + MLS group-ratchet stack is too heavy for a microcontroller
class device, and no transport binding targets CoAP/MQTT. This is the "~1 unbuilt" of
[`README.md §1`](README.md).
**Minimal fix.** A **constrained-device profile**: a bounded MOTE envelope (size-capped, no MLS
epoch machinery for one-way telemetry — symmetric or HPKE-only sealing), plus a
[bindings](../../bindings/README.md) row for CoAP/MQTT-over-the-substrate transport. *No new
waist capability, no new primitive, no new coordinator.* A thin profile plus one binding row.

### G2 — Hosted private-AI (blind inference) *(✚ gap; row 22)*
**What's missing.** Private-AI on *your own box* is fully covered — it is compute on hardware
you hold, needing no coordinator. Renting *someone else's* GPU while keeping your prompts
private is a coordinator job (a scarce resource — accelerators — with a global-ish view); **`compute`
is a provisional kind** in [CONTRACT §5](../../coordinator/CONTRACT.md) (`terminating` by default,
`attested`/TEE for blind inference), but no bindings or offer/settlement wiring for it are built
yet. A naive hosted inferencer is `terminating` (it reads your prompt).
**Minimal fix.** Graduate **compute** from provisional to fully-specified in
[CONTRACT §5](../../coordinator/CONTRACT.md) once its TEE
[binding](../../bindings/README.md) and offer/settlement path are worked through end to end. The
*offer* of compute-for-hire rides TRACT/WRAP unchanged; payment rides x402. *No new primitive* —
the kind slot already exists; what remains is filling it in. The honest residual is that the
provisional slot is **disclosed-but-undemonstrated**: the TEE residual already disclosed
([CONTRACT §3.4](../../coordinator/CONTRACT.md)) — `attested` trades operator-trust for
chip-vendor-trust and is never sold as trustless — plus the fact that, unlike `gateway`, this
kind has not yet been worked through as a first fully-specified instance of the contract.

### G3 — Geospatial query / maps *(✚ partial; row 14)*
**What's missing.** Map *data* is covered (tiles are PUB blobs over
[storage](../../bindings/README.md); a box is a local map CDN; routing computes locally over the
data the way TRACT computes delivery routing on the buyer's node). What has no substrate
expression is **"what is near this location?"** — the waist addresses objects by content-hash
and by key, never by coordinate. Rides and delivery (rows 6–7) hide this inside a private
matcher; maps and POI search expose it.
**Minimal fix.** A **geo-index** — the existing indexer kind specialised to spatial queries
(`blind`/attested preferred, TEE for query privacy) — plus an unbuilt maps profile. *No new
waist primitive*; geo-proximity is a query surface on a coordinator, not a new capability. The
same geo-index serves the matchers in rows 6–7, so this is one addition, not three.

### G4 — Sealed-bid / second-price auctions *(✚ partial; row 9)*
**What's missing.** Open ascending auctions are covered by a multi-author SYNC bid book (bids
are signed CRDT ops; highest wins; no operator). Sealed-bid and second-price need bids to be
*unreadable until close* and *un-withholdable after commit* — a fairness property SYNC alone
doesn't give.
**Minimal fix.** A **commit-reveal MOTE convention** (commit a hash of the bid before close,
reveal after) plus a clearing step that MAY run in an `attested` matcher for reveal-timing
integrity. *No new primitive* — a convention over existing MOTE and one optional coordinator
mode. Ascending auctions need none of this and stay fully covered.

### G5 — Calendar free/busy negotiation *(✚ partial; row 16)*
**What's missing.** A shared calendar is SYNC; a booking is RESERVE against a single-owner
calendar; an invite is a MOTE. All three exist. What is unspecified is *cross-party mutual-slot
negotiation* ("find a time that works for all of us") — today it composes but has no profile
naming the composition.
**Minimal fix.** A **thin scheduling profile** over SYNC + MOTE invites + TRACT RESERVE. *No
new primitive, no coordinator.* Composition, documented.

### Not a gap — advertising *(✖ by design; row 20)*
Direct-buy / sponsored-listing advertising **is** covered: a sponsored object is a PUB object,
payment is PAY, placement is an indexer surface. **Surveillance-based programmatic ad markets
are rejected by design** ([DIRECTION §8](../../DIRECTION.md)) — there is no cross-user behavioural
profile to sell because there is no party that holds one. This is a deliberate non-goal, listed
so the matrix is not mistaken for incomplete.

---

## 5. Honest residual

- **This is a reduction proof, not a deployment proof.** The matrix shows each shape *reduces
  to* the primitives. It does **not** show that the hard, deployed-scale versions work: global
  cross-publisher product identity (TRACT §21.2), unpaid pinning of a departed node's catalogue
  (TRACT §0.7), and a global matcher at Uber quality are all mechanism-present and
  demonstration-absent. A `✔` means "composes from the primitives," never "shipped and
  measured at scale."
- **The four ceilings recur; they are not per-row bugs.** Every `▲` traces to one of
  [`README.md §2`](README.md)'s roots: **anti-Sybil** (search row 13, reputation everywhere),
  **physical-event oracle** (rides / delivery / freelance rows 6, 7, 10 — a coordinator can
  attest origin-through-itself, never non-fabrication), **legal / authoritative-issuer**
  (credentials, healthcare, registries rows 17–19 — a real accountable party the paid-coordinator
  model *absorbs* but does not dissolve), and **editorial governance** (commerce discovery,
  registries, social rows 4, 11, 13, 19 — "who decides the canonical version"). Naming four
  roots instead of a dozen symptoms is the honest accounting.
- **The two new seams (G1–G2) add a residual, not a guarantee.** The `compute` kind and the
  constrained-device profile are *slots*, not solutions: blind hosted inference is only as blind
  as the TEE it declares (`attested`, disclosed, not trustless), and a constrained MOTE profile
  trades some of the sealed path's ratchet guarantees for footprint — which the profile MUST
  disclose per its own honest-residual section when written.
- **Two shapes are genuinely out of scope and stay so:** surveillance advertising (row 20,
  rejected) and — outside this matrix's 22 but worth stating for completeness —
  coercion-resistant public-election voting, which is harder than anti-Sybil and which KOTVA
  claims *cannot* deliver even in principle ([DIRECTION §8](../../DIRECTION.md)).
- **The matrix dates itself.** Coverage rests on the 2026 [bindings survey](../../bindings/README.md);
  maturity and demand claims are a snapshot ([`README.md §6`](README.md) caveat) and must be
  re-checked before any row's coordinators are relied on in production.
