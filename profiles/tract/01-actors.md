# 1. Actors & identity

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 1.1 Scope

Who the parties are, how they are identified, and how they are reached. TRACT adopts substrate
capability ① (Identity) unchanged and adds only the commerce-specific roles and disclosures.

## 1.2 One identity type, five roles

Seller, buyer, courier, distributor and gateway are roles a key takes by what it does, never
separate account kinds a key applies for. The identity underneath every one of them is the
substrate's `IK` plus an optional `DeviceCert` (§1.8) — the same construction a mail identity uses
(§0.3).

| Role | Becomes that role by |
|---|---|
| Seller | publishing a signed catalogue feed (§2) |
| Buyer | holding a cart and sending a sealed order (§6, §7) |
| Courier | publishing a rate card (§8) |
| Distributor | publishing a capacity record (§8) |
| Gateway | the one role needing more than a key — a domain, uptime, and, where it settles, a payment-provider relationship (§0.4.2) |

Because the role is what a message does, and not a field anywhere records, there is no
registration step — there is nothing to register with. No table anywhere maps a key to "seller" or
"buyer"; a key is whichever of these it is currently acting as, and a verifier checking a signed
object never has to ask which. A single key can be all five at once: a seller restocking from a
wholesaler signs the purchase order with the identical key it publishes its own catalogue under,
and nothing in the wire format (§16) distinguishes a "seller key" from a "buyer key" — there is one
`identity-key` type (§16.3). Gateway is the exception only in what it costs to enter, never in what
it is: the same key that runs a storefront can also be a buyer on somebody else's.

## 1.3 Seller disclosure

Several trader-traceability duties assume a seller's identity comes with a minimum disclosure
attached to it, not just a key. TRACT gives an offer the fields those duties ask for; it does not
decide which duties apply to a given trade — that is §11's job, and §11.4 lists the regimes it has
to accommodate.

| Field | What it answers |
|---|---|
| legal or trading name | who the seller claims to be |
| contact route | how a buyer or a regulator reaches them — not necessarily a physical address |
| establishment country | the seller-establishment anchor (§11.2) |
| registration reference, where the seller holds one | a company number, VAT number, or equivalent |

A self-published claim of a trading name carries no more authority than any other field in an
offer the seller alone signs — it is evidence of what the seller asserted, not proof of who they
are. Whether that is sufficient to discharge a legal disclosure duty is exactly the kind of
question §11 exists to answer and largely cannot yet: trader-traceability regimes specifically are
one of the four legal questions a dedicated pass returned nothing verified on, across three
attempts (§21.11). The field's presence in this section is not a compliance claim.

## 1.4 Buyer disclosure tiers

A buyer discloses more of themselves the closer they get to a transaction, and the tiers stay
distinct rather than collapsing into one identity handed over on first contact.

| Tier | What is attached | Seen by |
|---|---|---|
| Browse | nothing — fetching a product record, offer or rate card is an anonymous pull (§0.6) | nobody |
| Grant | the buyer's per-seller pseudonymous subkey (§1.5), enough to be recognised as a repeat visitor without a name | that one seller |
| Order | full detail — name, delivery address, contact — carried inside the sealed `Order` (§16.6) | that one seller, and only for that one order's lines |

Nothing between browse and order is public: the grant tier's subkey is not a new object type, it
is the same key a buyer already uses when returning to that seller's cart or leaving a review
(§1.5), and the order tier's detail never leaves the sealed channel it arrives in (§0.5.1).

## 1.5 Per-store pseudonymous subkeys

A buyer who wants to be recognised by a seller they return to — a repeat-customer discount, a
review with continuity, a support thread that doesn't start from zero — needs *some* linkability.
Full identity would leak it to every seller at once; a fresh key per interaction would leak none,
including to the seller who should have it. TRACT's answer is a subkey scoped to one seller: linked
within that relationship, not across it. It is the same construction `Review.author` already relies
on (§16.5.5) — a per-subject subkey, never the root `IK`.

Two ways to derive it, and the choice is a real trade-off rather than an implementation detail:

- **Deterministic** — the subkey is a function of the buyer's root key material and the seller's
  identity key. Nothing needs storing or backing up; the same subkey falls out again on a new
  device with the same root key. The honest cost: anyone who later learns the buyer's root key — a
  compelled disclosure, a seized device, a leaked backup — can recompute every per-seller subkey the
  buyer ever used and join their purchase history across every seller at once, retroactively. The
  unlinkability was never structural; it was the difficulty of the computation, and that difficulty
  is zero for the one party who holds the root key.
- **Stored random mapping** — a subkey generated independently per seller and kept in the buyer's
  own synced cart state (§6.2, substrate capability ③). Learning the root key reveals nothing extra
  about it. The cost lands on recovery: losing the mapping — not the root key, the mapping itself —
  severs the link to that seller's history, and a buyer restoring from a bare seed phrase gets a
  working identity but a stranger's standing with every seller they had a relationship with.

Either way, two sellers comparing customer lists cannot join them by subkey; only the buyer,
holding the root key, can ever compute or has ever stored the correspondence.

## 1.6 Reachability

A seller, courier or distributor is reached by identity key, not by network address, which is what
makes CGNAT irrelevant rather than a deployment obstacle to route around. The path is the
substrate's own ladder (§0.3 ④): announce/resolve locates a current address for the key; a direct
connection is tried first; where NAT or firewall traversal fails, a circuit relay carries the
session without seeing its content; where the recipient is offline entirely, a short-TTL
content-blind mailbox holds what was sent until it is collected.

That last step matters specifically for orders. A sealed order addressed to a sleeping seller's key
does not fail and does not wait on the sender — it lands in the mailbox, and wake (§0.3 ⑤), a
content-free, sender-blind push, is what can bring the seller's node back online to collect it. The
same primitive substrate mail uses to wake a sleeping inbox wakes a sleeping storefront for exactly
the same reason: neither the wake signal nor the mailbox operator learns anything about what is
waiting.

## 1.7 The liveness asymmetry (§21.5, §21.9)

Reachability by key (§1.6) covers *orders*, not *catalogues*, and this section has to keep that
distinction visible rather than let "reachable by key" imply both. A sleeping node still receives
orders: the sender's node retains the retry queue and the mailbox-plus-wake path of §1.6 brings the
recipient back. A sleeping node does **not** serve its catalogue while asleep — so an offline seller
is not slow, they are invisible, and nobody else is obliged to serve their listings in their
absence. The closest deployed relative of this design measured a ~22-day median listing lifetime
and whole catalogues disappearing when a merchant node departed (§21.3). Third-party caching or
pinning of public objects therefore moves from a convenience to a practical requirement for any
seller who is not always on, and unpaid replication is exactly what that system failed to attract.
Whether pinning needs an incentive, and whether that incentive creates another operator, stays open
(§21.8).

## 1.8 Standards profiled

Ed25519 (RFC 8032), deterministic CBOR (RFC 8949 §4.2), COSE (RFC 9052), and the substrate's
`DeviceCert`, key-transparency and key-name constructions. No new key type is introduced.

## 1.9 Open

- Whether a seller's legal-disclosure block (§1.3) belongs in the offer, the feed head, or both.
- Whether subkey derivation (§1.5) should be deterministic or a stored random mapping — the
  trade-off is stated above; which one the spec should mandate, or whether it leaves the choice to
  implementations, is not decided.
- Whether the grant tier (§1.4) needs its own signed disclosure object, or is fully satisfied by
  the per-seller subkey with no separate protocol object at all.
