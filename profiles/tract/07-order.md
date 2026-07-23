# 7. Order

> **Drafting status.** Partially normative. The order object's shape and invariants (§7.2), the
> evidence semantics of each signed transition (§7.3), and the no-cross-seller-rollback property
> (§7.8) are now normative and aligned to the frozen `Order` grammar (§16.6). Four surfaces remain
> **PROVISIONAL — pending decision** and are marked inline: order amendment (§7.4), partial
> fulfilment (§7.5), the order-level marker for a partial refund (§7.6), and who bears return
> carriage (§7.7). The lifecycle itself — every transition, trigger and timeout — is normative in
> **§18.3** and is referenced here, never restated (§19.1).

## 7.1 Scope

The sealed order object, its per-object invariants, and what evidence each lifecycle transition
produces. The lifecycle state machine is §18.3's; this section adds the evidence semantics that
sit above it and the personal-data handling the sealed shape makes possible.

## 7.2 One seller, one sealed order

`Order` is a sealed message (§16.6): it MUST be encrypted, per-party, and MUST NOT be published in
the public object family. It is held at the two endpoints, which is where deletion is meaningful
(§0.5, §22.3).

An `Order` MUST name exactly one seller (§16.6 field 2) and MUST carry only that seller's lines
(§16.6 field 3). This is a grammar-level property, not a client convention: there is no field a
cross-seller order could occupy, so a cross-seller order is not expressible (§16.6). A buyer's cart
(§6) is the only place a multi-seller view exists; checkout is where that view stops existing
anywhere else.

A cart spanning N sellers MUST produce N independent sealed `Order`s, one per seller. No `Order`
MUST carry any trace of the others — not their sellers, not their lines, not their totals. Each
seller learns only what §16.6 exposes to it: its own lines, and this buyer's total for those lines.

**The total is exactly one money.** `Order.total` (§16.6 field 4) MUST be a single `money` value.
An order carries exactly one currency; cross-currency arithmetic MUST be refused, never coerced,
and MUST NOT be represented as a float (§16.7). A cart mixing currencies is resolved into
single-currency orders at checkout (§6), not carried into an `Order`.

**Personal data lives only in the sealed family, and is deletable there.** Buyer name, delivery
address and contact details MUST be carried only within the sealed `Order` (§16.6) and MUST have no
production in the public object family at all (§16.4). Because a sealed order exists at exactly two
endpoints, each holding its own copy, an erasure request against it is satisfiable in the ordinary
sense — each endpoint deletes the copy it holds (§22.3, §22.5). This is the structural reason the
protocol carries a buyer's identifying details as sealed order fields and refuses them everywhere
in the public quadrant (§0.5.1); the residual limits that remain even so are §22's subject, not
this section's to overstate.

## 7.3 Who signs, and what it proves

§18.3 has the full machine — every transition, its trigger, its signer, its timeout. This section
MUST NOT keep a second copy of that table, because a fact stated twice is a fact that can disagree
with itself the day only one copy gets edited (§19.1). What it adds instead is what §18.3's table
does not carry: what each signature is evidence *of*, and — for the one transition that genuinely
varies — what "delivered" actually means.

Two transitions carry evidence that is straightforward to state once. `placed` (§18.3) is the
buyer's own signed commitment to specific lines, a specific seller and a specific total, and
nothing more. `accepted` / `declined` / `countered` (§18.3) is the seller's own signed response;
it is signed rather than inferred, which is precisely why §18.3 treats silence as decline rather
than acceptance — an inferred "yes" would be a signature that was never made.

**`delivered` is not one kind of evidence.** The evidence for a `fulfilling → delivered`
transition MUST be whichever the Fulfilment axis (§4) specifies for the variant the line was placed
against; there is no single delivery evidence type, and this section MUST NOT be read as defining
one. §18.4 states that handover evidence exists for the transition; §4 owns the mapping from
Fulfilment variant to what that evidence is. A `ship` line closes on a carrier's proof-of-delivery
signed by the party taking custody (§18.4), not the sender. A `digital-grant` line closes when the
grant is issued, signed by the seller alone, with no carrier in it. A `perform-at-place` line
closes on attendance at the stated time and place. Because `Order.state` (§16.6 field 5) is one
field for the whole message, an `Order` carrying lines under different variants MUST reach
`delivered` only once every line's own evidence exists — which, for a mixed order, means waiting on
the slowest line.

**`closed` is the one transition that can carry no signature at all.** `CONFIRM_TIMEOUT` expiring
into `closed` (§18.3, §19) is deliberate: a buyer who stops responding after delivery must not be
able to strand a seller's money by doing nothing. The evidence behind that specific transition is
elapsed time plus whatever was last signed, not a buyer's word. An implementation MUST NOT present
the timeout path as an implicit buyer confirmation; it is the absence of one, resolved by §18.3's
stated default in the seller's favour.

## 7.4 Amendment

> **PROVISIONAL — pending decision.** Whether an amendment is a new sealed `Order` superseding the
> one it changes, or an entry in an append-only operation log against the original, is open at the
> wire-format level (§16.8), and this section inherits that open question rather than picking a
> side. No normative amendment mechanism is specified until §16 records the shape.

The two options pull toward different parts of the design that already exist. Supersession is the
pattern the rest of the document uses — an offer's withdrawal is a successor object, not an edit
(§18.2); a review's retraction is a superseding tombstone (§10.2) — so amendment-by-supersession
would be one mechanism doing a job this specification asks for repeatedly, rather than a special
case invented for orders. A log is easier to audit for a different reason: the full history of what
changed and when is a linear read, where a chain of superseding objects has to be walked backward
through content addresses to reconstruct the same story. Neither is decided here; §16.8 is where
the choice, once made, belongs.

## 7.5 Partial fulfilment

> **PROVISIONAL — pending decision.** An order can be split across time even where it cannot be
> split across sellers — a line back-ordered while the rest ship, or a quantity short-shipped
> against what was accepted. The frozen grammar has nowhere to put per-line progress: `Order`
> carries one `OrderState` for the whole message (§16.6 field 5) and `OrderLine` carries no status
> of its own. No normative representation is specified until the grammar decision is made.

Two shapes could close the gap: a per-line status field added to `OrderLine` (a §16 grammar
change — logged, not invented here), or partial fulfilment expressed as an amendment (§7.4) that
splits one order into a closed successor for what shipped and a still-fulfilling successor for what
did not. Given §16.8's lean toward supersession, the split reads as the more consistent shape, but
nothing here commits to it — this is a decision that has to be made once, at the grammar, rather
than differently by every implementation.

## 7.6 Partial refund

The refund object itself is settled at the grammar level and stated normatively here. A partial
refund MUST be expressed as a second `PaymentAttestation` (§16.6) against the same order, with
`payer` and `payee` reversed relative to the original settlement and `amount` set to the refunded
difference. `PaymentAttestation` already carries a payer, payee, order reference and amount, so
nothing new is needed in the grammar for the refund object; like every `PaymentAttestation` it
MUST carry an opaque external settlement *reference* only, never funds and never card data (§16.6,
§9.2).

> **PROVISIONAL — pending decision.** Whether the order's own state machine (§18.3) needs a
> `refunded` / `partially-refunded` marker, or whether the payment-attestation trail is sufficient
> evidence on its own with nothing said at the order level, is open. The refund object is
> expressible today either way; only the order-level marker is undecided.

## 7.7 Returns and exchanges

A return only has a shape where the Fulfilment axis (§4) gave it a place to go back to. `collect`
and `perform-at-place` have nothing to reverse in the courier sense — undoing them is a refund
(§7.6) or a re-performance, not a shipment. `digital-grant` and `access-grant` have no return leg:
"return" there means revoking a grant already issued, which is a licence-terms question for §4.5
and §5, not a delivery question for this section. Two variants carry an actual carriage leg back —
`ship`, and `return-required` (§16.5.2), whose whole point is that the goods were always going back
to a place the variant already names (§4.2, §4.3).

> **PROVISIONAL — pending decision.** Who bears return carriage on a `ship` or `return-required`
> line by default — buyer, seller, or split by reason for return — is not decided, and neither is
> where that term lives: a default on the offer, a field on the order (a §16 grammar change if so),
> or negotiation left entirely to the parties. Incoterms 2020 (profiled for the forward leg, §4.4,
> §8.3) says nothing about the reverse one. No normative default is set here.

Several consumer-protection regimes assign return-carriage cost by rule for a change-of-mind return
inside a cooling-off window (§11.4 lists which regimes this section must accommodate). That argues
for letting a regime-mandated term override whatever an offer declares by default, rather than the
other way round. Whether that override is expressed here or belongs entirely to §11 is part of the
same open decision.

## 7.8 One seller's decline is not another's rollback

A cart spanning several sellers produces several independent orders (§7.2), and one of them
declining, timing out, or being cancelled MUST have no effect on the others' state. This is a
property, not a limitation apologised for: a distributed transaction across sovereign parties needs
a coordinator with authority over all of them, and that coordinator is precisely what this design
does not have and is not trying to grow. §6.4 states the consequence for the cart; it is stated
again for the order machine because a reader might otherwise expect the order layer — not the cart —
to paper over it with a single "order placed" status.

The evidence is direct rather than an assertion: sagas, the standard pattern for exactly this
multi-party situation, provide **neither atomicity nor isolation** (§21.10.3). Other parties can
observe a partially-completed checkout, and there is no abort cascade that unwinds an accepted
order because a different seller declined. An interface built on this section MUST show per-seller
status; a single aggregate status MUST NOT be presented, because it would report a guarantee the
protocol does not make (§21.10.3).

## 7.9 Standards profiled

`Order` uses the substrate's sealed message object for transport and introduces no transport of its
own (§16.6). Where an order must be exported to a legacy counterparty — an ERP, a 3PL — the mapping
targets are UN/CEFACT and EDIFACT order / despatch / invoice messages. That mapping is
**informative, not required**: a conformant TRACT implementation need not emit or consume it, and
it carries no normative weight over the `Order` grammar, which remains the authority (§16.6).

## 7.10 Open

Collected for the founder-decision list; each is marked PROVISIONAL in its subsection above.

- **Order amendment (§7.4)** — a superseding sealed object or an operation log. Supersession fits
  the pattern the rest of the document uses; a log is easier to audit. Undecided at the wire-format
  level (§16.8).
- **Partial fulfilment (§7.5)** — a per-line status field on `OrderLine` (a §16 grammar change) or
  an amendment that splits one order into two successors. The frozen grammar has no per-line status
  today either way.
- **Partial-refund marker (§7.6)** — whether §18.3 needs a `refunded` / `partially-refunded` state,
  or the `PaymentAttestation` trail suffices with nothing said at the order level. The refund
  object itself is already expressible.
- **Return carriage (§7.7)** — who bears it on a `ship` or `return-required` line by default, where
  that term lives, and whether a regime-mandated term (§11.4) may override an offer default.
