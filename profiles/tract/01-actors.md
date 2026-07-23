# 1. Actors & identity

> **Drafting status.** This section is **normative** for identity and roles. Settled and carrying
> RFC 2119 keywords aligned to the frozen `16-wire-format.md`: the one-identity-type / five-roles
> model (§1.2), the buyer disclosure tiers and their public/sealed placement (§1.4), per-store
> pseudonymous subkeys as a construction (§1.5), reachability (§1.6), and the standards profiled
> (§1.8). **Still scoped, not yet normative and marked PROVISIONAL inline:** *where* a seller's
> legal-disclosure block is carried on the wire (§1.3) — no §16 grammar slot exists for it and the
> offer / feed-head / both question is an open founder call — and *which* subkey-derivation method
> the spec mandates (§1.5). Both are collected in §1.9. The honest-limit subsection (§1.7) states a
> measured constraint and is not weakened by the normative text around it. The key words MUST, MUST
> NOT, REQUIRED, SHOULD, SHOULD NOT and MAY are to be interpreted as in BCP 14 (RFC 2119, RFC 8174)
> where they appear.

## 1.1 Scope

Who the parties are, how they are identified, and how they are reached. TRACT adopts substrate
capability ① (Identity) **unchanged** and adds only the commerce-specific roles and disclosures. It
introduces no new key type, no new signature framing, and no new address scheme; the identity
construction is governed entirely by the substrate
([`IDENTITY.md`](https://github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md)) and this section
MUST NOT be read as re-specifying it.

## 1.2 One identity type, five roles

Seller, buyer, courier, distributor and gateway are **roles a key takes by what it does**, never
separate account kinds a key applies for. Every TRACT party MUST be identified by the substrate's
`IK` keypair; a party MAY authorise additional devices under that identity with a substrate
`DeviceCert` (§1.8) — the same construction a mail identity uses (§0.3). No TRACT object carries an
account-kind or role field, and none MAY be added: the wire format exposes exactly one
`identity-key` type (§16.3), and a decoder MUST NOT expect a value distinguishing a "seller key"
from a "buyer key".

| Role | Becomes that role by | Requires beyond a key |
|---|---|---|
| Seller | publishing a signed catalogue feed (§2) | nothing |
| Buyer | holding a cart and sending a sealed order (§6, §7) | nothing |
| Courier | publishing a rate card (§8) | nothing |
| Distributor | publishing a capacity record (§8) | nothing |
| Gateway | running a storefront and/or settling payments (§0.4.2) | a domain with TLS and uptime, and — where it settles — a payment-provider relationship, a float, and licensing |

Because the role is what a message does and not a field anywhere records, there MUST NOT be a
registration step — there is nothing to register with, and no table anywhere maps a key to "seller"
or "buyer". A key is whichever of these it is currently acting as, and a verifier checking a signed
object never has to ask which. A single key MAY be all five at once: a seller restocking from a
wholesaler signs the purchase order with the identical key it publishes its own catalogue under,
and nothing in the wire format (§16) distinguishes those uses.

**Gateway is the one exception, and only in what it costs to enter, never in what it is (§0.4.2).**
To *act* as a gateway a key MUST additionally control resources no keypair yields: a domain with
TLS and uptime for storefront rendering, and, where it settles, a payment-provider relationship, a
money float, and jurisdiction-specific licensing. This is the only role in TRACT that requires
scarce resources, and confining it is the structural claim of the whole document (§0.4.3). The
same key that runs a storefront MAY also be a buyer on somebody else's; the resource requirement
attaches to the *act of operating*, not to the key.

## 1.3 Seller disclosure

Several trader-traceability duties assume a seller's identity comes with a minimum disclosure
attached to it, not just a key. TRACT's role is to make those facts expressible; it does not decide
which duties apply to a given trade — that is §11's job, and §11.4 lists the regimes it has to
accommodate.

| Field | What it answers |
|---|---|
| legal or trading name | who the seller claims to be |
| contact route | how a buyer or a regulator reaches them — not necessarily a physical address |
| establishment country | the seller-establishment anchor (§11.2), carried on the sealed `Order.Anchors[1]` (§16.6) |
| registration reference, where the seller holds one | a company number, VAT number, or equivalent |

A self-published claim of a trading name carries no more authority than any other field in an offer
the seller alone signs — it is evidence of what the seller asserted, not proof of who they are.
Whether that is sufficient to discharge a legal disclosure duty is exactly the kind of question §11
exists to answer and largely cannot yet: trader-traceability regimes (DSA / INFORM / GPSR)
specifically are one of the four legal questions a dedicated pass returned **nothing verified** on,
across three consecutive attempts (§21.11). The fields' presence in this section is not a
compliance claim.

> **PROVISIONAL — pending decision.** *Where* the seller-disclosure block lives on the wire is not
> settled, and **no §16 production carries it today**: neither `Offer` (§16.5.2) nor `ProductRecord`
> (§16.5.1) has a legal-name / contact-route / registration-reference field, and the establishment
> country is presently derivable only from a sealed order after the fact. Carrying disclosure on the
> public `Offer` makes it browsable before a trade but ties it to every offer revision; carrying it
> on the seller's `FeedHead` states it once per identity but decouples it from the specific supply;
> carrying it on both duplicates. This requires a §16 grammar change **and** a founder call on
> placement (§1.9) — logged as a required §16 open item, not invented here. Until then, an
> implementation MUST NOT synthesise a disclosure field the frozen grammar does not define.

## 1.4 Buyer disclosure tiers

A buyer discloses more of themselves the closer they get to a transaction, and the tiers MUST stay
distinct rather than collapsing into one identity handed over on first contact.

| Tier | What is attached | Seen by |
|---|---|---|
| Browse | nothing — fetching a product record, offer or rate card is an anonymous pull (§0.6) | nobody |
| Grant | the buyer's per-seller pseudonymous subkey (§1.5), enough to be recognised as a repeat visitor without a name | that one seller |
| Order | full detail — name, delivery address, contact — carried inside the sealed `Order` (§16.6) | that one seller, and only for that one order's lines |

These placements are normative and enforced by the grammar, not by convention:

- **Browse.** Retrieval of any public object (`ProductRecord`, `Offer`, `RateCard`,
  `CapacityRecord`) is a content-addressed pull and MUST NOT require or attach buyer identity.
- **Grant.** The grant tier attaches only the per-seller subkey of §1.5; it MUST NOT carry a name,
  address, or contact detail. It is the same key a buyer already uses when returning to that
  seller's cart or leaving a review (§16.5.5) — not a new object type.
- **Order.** Personal detail — buyer name, delivery address, contact — MUST be carried only inside
  the sealed `Order` family (§16.6) and MUST NOT appear in any public object. This is enforced by
  §16.4: there is no name, contact, or street-address production in the public grammar at all, so
  the leak is a grammar change a reviewer would see, not a field a client could add. Order detail
  never leaves the sealed channel it arrives in (§0.5.1), and it is scoped to that one order's lines
  only — a cross-seller order is not expressible (§16.6).

## 1.5 Per-store pseudonymous subkeys

A buyer who wants to be recognised by a seller they return to — a repeat-customer discount, a review
with continuity, a support thread that doesn't start from zero — needs *some* linkability. Full
identity would leak it to every seller at once; a fresh key per interaction would leak none,
including to the seller who should have it. TRACT's answer is a subkey scoped to one seller: linked
within that relationship, not across it.

This is the same construction the wire format already mandates for `Review.author`: a review's
author key MUST be a **per-subject pseudonymous subkey and MUST NOT be the root `IK`** (§16.5.5). A
buyer's grant-tier recognition subkey is derived per seller under the identical rule, so that two
sellers comparing customer lists cannot join them by subkey. Only the buyer, holding the root key,
can ever compute or has ever stored the correspondence.

Two ways to derive it, and the choice is a real trade-off rather than an implementation detail:

- **Deterministic** — the subkey is a function of the buyer's root key material and the seller's
  identity key. Nothing needs storing or backing up; the same subkey falls out again on a new device
  with the same root key. The honest cost: anyone who later learns the buyer's root key — a compelled
  disclosure, a seized device, a leaked backup — can recompute every per-seller subkey the buyer ever
  used and join their purchase history across every seller at once, retroactively. The unlinkability
  was never structural; it was the difficulty of the computation, and that difficulty is zero for the
  one party who holds the root key.
- **Stored random mapping** — a subkey generated independently per seller and kept in the buyer's own
  synced cart state (§6.2, substrate capability ③). Learning the root key reveals nothing extra about
  it. The cost lands on recovery: losing the mapping — not the root key, the mapping itself — severs
  the link to that seller's history, and a buyer restoring from a bare seed phrase gets a working
  identity but a stranger's standing with every seller they had a relationship with.

> **PROVISIONAL — pending decision.** Which derivation the spec mandates — deterministic, stored
> random mapping, or left to the implementation — is not decided; the trade-off above is stated in
> full, but the choice is a founder call (§1.9). Neither method changes any wire shape: `Review`
> already fixes the author as a per-subject subkey (§16.5.5) and the grant-tier key rides the same
> slot, so this decision governs key management, not bytes.

## 1.6 Reachability

A seller, courier or distributor MUST be reachable by identity key, not by network address, which is
what makes CGNAT irrelevant rather than a deployment obstacle to route around. The path is the
substrate's own ladder
([`ROLES.md`](https://github.com/vul-os/dmtap/blob/main/substrate/ROLES.md), §0.3 ④), which TRACT
adopts unchanged and MUST NOT re-specify: announce/resolve locates a current address for the key
(`LocationRecord`); a direct connection is tried first; where NAT or firewall traversal fails, a
circuit relay carries the session **without seeing its content**; where the recipient is offline
entirely, a short-TTL content-blind mailbox holds what was sent until it is collected.

That last step matters specifically for orders. A sealed order addressed to a sleeping seller's key
MUST NOT fail and MUST NOT wait on the sender — it lands in the mailbox, and wake (§0.3 ⑤,
[`ROLES.md §8`](https://github.com/vul-os/dmtap/blob/main/substrate/ROLES.md)), a content-free,
sender-blind push (`PushSubscription` / `WakePing`), is what can bring the seller's node back online
to collect it. The same primitive substrate mail uses to wake a sleeping inbox wakes a sleeping
storefront for exactly the same reason: neither the wake signal nor the mailbox operator learns
anything about what is waiting.

## 1.7 The liveness asymmetry (§21.5, §21.9)

Reachability by key (§1.6) covers *orders*, not *catalogues*, and this section MUST keep that
distinction visible rather than let "reachable by key" imply both. A sleeping node still receives
orders: the sender's node retains the retry queue and the mailbox-plus-wake path of §1.6 brings the
recipient back. A sleeping node does **not** serve its catalogue while asleep — so an offline seller
is not slow, they are **invisible**, and nobody else is obliged to serve their listings in their
absence. Documentation MUST NOT present an intermittently-online seller as fully functional (§21.9).

The closest deployed relative of this design measured a ~22-day median listing lifetime and whole
catalogues disappearing when a merchant node departed (§21.3). Third-party caching or pinning of
public objects therefore moves from a convenience to a **practical requirement** for any seller who
is not always on — and unpaid replication is exactly what that system failed to attract (§21.5).
Whether pinning needs an incentive, and whether that incentive creates another operator, stays open
(§21.8). This subsection cites §21 only for the honesty it obliges; per §21.1 it is not offered as
support for any logistics or trust claim.

## 1.8 Standards profiled

Ed25519 (RFC 8032), deterministic CBOR (RFC 8949 §4.2), COSE (RFC 9052), and the substrate's
`DeviceCert`, key-transparency and 8-word key-name constructions
([`IDENTITY.md`](https://github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md)). **No new key
type is introduced, and an implementation MUST NOT introduce one** — a key construction appearing in
TRACT that is not the substrate's would be a defect (§16.2).

## 1.9 Open

Collected for the founder-decision list:

- **Where the seller legal-disclosure block (§1.3) is carried** — on the `Offer`, the `FeedHead`, or
  both. This needs a §16 grammar slot that does not exist yet (recorded as a required §16 change) and
  a placement call. Recommendation: carry it once on the seller's `FeedHead` (one identity, one
  disclosure), with an optional per-offer override slot for sellers who trade under distinct names,
  so the common case is not duplicated per offer.
- **Whether subkey derivation (§1.5) is deterministic or a stored random mapping.** The trade-off is
  stated in full above. Recommendation: mandate the stored random mapping as the default (structural
  rather than computational unlinkability), and permit deterministic derivation only as an explicit,
  disclosed opt-in for buyers who prioritise seedphrase-only recovery over resistance to root-key
  compromise.
- **Whether the grant tier (§1.4) needs its own signed disclosure object**, or is fully satisfied by
  the per-seller subkey with no separate protocol object at all. Recommendation: no new object — the
  per-seller subkey plus the existing `Review`/cart constructions already carry every grant-tier need,
  and a new object would add wire surface for no capability.
