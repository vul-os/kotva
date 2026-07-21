# 7. Order

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 7.1 Scope

The sealed order object, its lifecycle, and the evidence each transition produces.

## 7.2 One seller, one sealed order

`Order` is a sealed message (§16.6): encrypted, per-party, never published, held at the two
endpoints where deletion is meaningful (§0.5). It names exactly one seller and carries only that
seller's lines — not by convention but by the shape of the object itself: there is no field a
cross-seller order could occupy (§16.6). A buyer's cart (§6) is the only place a multi-seller view
exists; checkout is where it stops existing anywhere else. A cart spanning five sellers produces
five independent sealed orders, one per seller, and none of the five carries any trace that the
other four exist — not their sellers, not their lines, not their total. Each seller learns only
what §16.6 gives it: its own lines, and this buyer's total for those lines.

## 7.3 Who signs, and what it proves

§18.3 has the full machine — every transition, its trigger, its timeout. This section does not keep
a second copy of that table, because a fact stated twice is a fact that can disagree with itself
the day only one copy gets edited (§19.1). What it adds instead is the part §18.3's table doesn't
carry: what each signature is evidence *of*, and — for the one transition that genuinely varies —
what "delivered" actually looks like.

Two of the transitions carry evidence that is straightforward to state once. `placed` is the
buyer's own commitment to specific lines, a specific seller and a specific total, nothing more.
`accepted` / `declined` / `countered` is the seller's own response, signed rather than inferred,
which is precisely why §18.3 treats silence as decline rather than acceptance — an inferred "yes"
would be a signature that was never made.

**`delivered` is not one kind of evidence — it is whichever the Fulfilment axis (§4) says it is.**
§18.4 states that handover evidence exists for a `fulfilling → delivered` transition; it does not
say what that evidence is, because the answer depends on which Fulfilment variant the line was
placed against, and §4 is where that mapping belongs. A shipped line closes on a carrier's
proof-of-delivery, signed by the party taking custody (§18.4) — not the sender. A digital-grant
line closes the moment the grant is issued, signed by the seller alone, with no carrier anywhere in
it. A perform-at-place line closes on attendance at the stated time and place. An `Order` can carry
lines under different variants at once, and `Order.state` is one field for the whole message
(§16.6): it only reaches `delivered` once every line's own evidence exists, which for a mixed order
can mean waiting on the slowest one.

**`closed` is the one edge that can carry no signature at all.** `CONFIRM_TIMEOUT` expiring into
`closed` (§18.3, §19.2) is deliberate, not an oversight — §18.3 states why: a buyer who stops
responding must not be able to strand a seller's money by doing nothing. But the evidence behind
that specific transition is elapsed time plus whatever was last signed, not a buyer's word, and
this section should not let anyone read the timeout path as an implicit confirmation.

## 7.4 Amendment

Whether an amendment is a new sealed `Order` superseding the one it changes, or an entry in an
append-only operation log against the original, is open at the wire-format level (§16.8), and this
section inherits the same open question rather than quietly picking a side. The two options pull
toward different parts of the design that already exist elsewhere in this specification.
Supersession is the pattern the rest of the document already uses — an offer's withdrawal is a
successor object, not an edit (§18.2); a review's retraction is a superseding tombstone (§10.2) —
so amendment-by-supersession would be one mechanism doing a job this specification asks for
repeatedly, rather than a special case invented for orders. A log is easier to audit for a
different reason: the full history of what changed and when is a linear read, where a chain of
superseding objects has to be walked backward through content addresses to reconstruct the same
story. Neither is decided here; §16.8 is where the choice, once made, belongs.

## 7.5 Partial fulfilment

An order can be split across time even when it cannot be split across sellers: a line back-ordered
while the rest ship, or a quantity short-shipped against what was accepted. The wire format as it
stands has nowhere to put that — `Order` carries one `OrderState` for the whole message (§16.6), and
`OrderLine` carries no status of its own. Two shapes could close that gap: a per-line status field
added to `OrderLine`, or partial fulfilment expressed as an amendment (§7.4) that splits one order
into a closed successor for what shipped and a still-fulfilling successor for what didn't. Given
§16.8's lean toward supersession, the split reads as the more consistent shape, but nothing here
commits to it — this is the kind of decision that has to be made once, at the grammar, rather than
differently by every implementation.

## 7.6 Partial refund

This one has less distance to travel. `PaymentAttestation` (§16.6) already carries a payer, a
payee, an order reference and an amount — a partial refund is a second attestation against the
same order, payer and payee reversed, amount set to the difference. Nothing new needs adding to the
grammar for the object itself; what remains open is only whether the order's own state machine
(§18.3) needs a `refunded` or `partially-refunded` marker, or whether the payment-attestation trail
is sufficient evidence on its own, with nothing said about it at the order level.

## 7.7 Returns and exchanges

A return only has a shape where the Fulfilment axis (§4) gave it a place to go back to. `collect`
and `perform-at-place` have nothing to reverse in the courier sense — undoing them is a refund or a
re-performance, not a shipment. `digital-grant` and `access-grant` don't have a return leg either;
"return" there means revoking a grant already issued, which is a licence-terms question for §4.5
and §5, not a delivery question for this section. Two variants carry an actual carriage leg back:

- **`ship`** — the goods have to travel back to the seller, and nothing in Incoterms 2020 (already
  profiled for the forward leg, §4.4, §8.3) speaks to who pays for the reverse one. Who bears
  return carriage — buyer, seller, or split by reason for return — is a term this section has to
  give a home to, and it isn't decided yet whether that home is a default on the offer, a field on
  the order, or left entirely to the parties to negotiate.
- **`return-required`** (§16.5.2) — the Fulfilment variant already names a place and a term (§4.2,
  §4.3), because the whole point of that variant is that the goods were always going back. Who pays
  to get them there is the same open question as `ship`'s, with the same shape.

Several consumer-protection regimes assign return-carriage cost by rule for a change-of-mind return
inside a cooling-off window (§11.4 lists which regimes this section has to accommodate), which
argues for letting a regime-mandated term override whatever an offer declares by default rather
than the other way round. Whether that override is expressed here or belongs entirely to §11 is
unresolved.

## 7.8 One seller's decline is not another's rollback

A cart spanning several sellers produces several independent orders (§7.2), and one of them
declining, timing out, or being cancelled has no effect on the others' state. This is stated as a
property, not a limitation apologised for: a distributed transaction across sovereign parties needs
a coordinator with authority over all of them, and that coordinator is precisely the thing this
design does not have and is not trying to grow. §6.4 already states the consequence for the cart;
it is repeated here because it is the order machine, not the cart, that a reader might otherwise
expect to paper over it with a single "order placed" status.

The literature backs this up directly rather than leaving it as an assertion: sagas — the standard
pattern for exactly this multi-party situation — provide **neither atomicity nor isolation**
(§21.10.3). Other parties can observe a partially-completed checkout, and there is no abort cascade
that unwinds an accepted order because a different seller declined. An interface built on top of
this section has to show per-seller status, because a single aggregate status would be reporting a
guarantee the protocol does not make.

## 7.9 Standards profiled

The substrate's sealed message object for transport. Where an order must be exported to a legacy
counterparty (an ERP, a 3PL), the mapping targets are UN/CEFACT and EDIFACT order/despatch/invoice
messages; that mapping is informative, not required.

## 7.10 Open

- Whether order amendment (§7.4) is a superseding sealed object or an operation log — supersession
  fits the pattern the rest of the document uses; a log is easier to audit. Undecided at the
  wire-format level (§16.8).
- Whether partial fulfilment (§7.5) needs a per-line status field on `OrderLine`, or is represented
  as an amendment that splits one order into two. The grammar has no per-line status today either
  way.
- Who bears return carriage on a `ship` or `return-required` line by default (§7.7), and whether a
  regime-mandated term is allowed to override it.
