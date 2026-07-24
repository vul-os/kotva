# TRACT conformance vectors

## What this is, and how it relates to `conformance/README.md` and `conformance/SUITE.md`

`conformance/README.md` states the rule: **a vector is derived by someone reading the
specification text, never exported from a running implementation.** `conformance/SUITE.md` is
the 39-case catalogue that rule is meant to fill in, and as of this writing it still describes
every one of those 39 cases as PLANNED, because `16-wire-format.md` carries a drafting-status
banner marking it not yet normative.

This directory does not edit either of those two files, and does not declare §16 normative —
that promotion, if and when it lands, is someone else's edit. What it does is show that, reading
`16-wire-format.md`'s CDDL grammar as it stands today (plus the concrete formulas in
`04-fulfilment.md` §4.3 and `09-settlement.md` §9.4 that the grammar's objects feed), a first
slice of `SUITE.md`'s cases is **already mechanically computable** — the byte shapes are written
out in enough detail to check arithmetic against, even while the surrounding prose keeps its
"proposed, not frozen" caveat. When someone does land §16 (and the sections it draws formulas
from) as normative, moving the corresponding `SUITE.md` rows out of PLANNED is a small, separate
edit informed by what is already sitting here.

**21 vector files, covering 8 of the 39 `SUITE.md` case ids** (6 fully, 2 partially — see the
table below). The other 31 ids are listed too, each with the specific reason it is not yet
computable from what has been read for this pass. Nothing here claims more than that.

## The discipline this corpus tries to hold itself to

Every vector in `generate.py` is built as native Python, with a docstring or inline comment
citing the exact §-numbered sentence it encodes or computes from. Nothing is imported from, run
against, or copied out of Soko (`/Users/pc/code/vulos/soko`) — not its crates, not its test
output, not a byte sequence eyeballed from a debugger. Where the cited text is itself silent on a
detail a vector needs (a rounding rule, a tie-break, a numeric interpretation of "intersect"),
`generate.py` says so in the vector's own `description`/`note` fields rather than picking an
answer and presenting it as derived. Two vectors are marked `"coverage": "partial"` for exactly
that reason (`TRACT-CAT-01`, `TRACT-SETTLE-01`) — see their entries below.

One gap surfaced by actually trying to build these vectors, rather than invented to fill space:
**`TRACT-JURIS-02`** asserts an `Order` carries all four jurisdictional anchors (seller
establishment, buyer residence, place of supply, delivery destination) as independent fields —
but §16.6's `Order` CDDL, as currently written, has no fields for any of the four; it carries
only buyer, seller, order lines, total, state and a timestamp. Either those four anchors are
meant to live somewhere §16 doesn't show yet, or `Order` needs new fields before `TRACT-JURIS-02`
is computable. This corpus does not resolve that — it is left as a finding for whoever lands
§16, §11 and this suite row together.

## Regenerating and verifying

```
python3 conformance/vectors/generate.py --write     # (re)write vectors/*.json from generate.py
python3 conformance/vectors/generate.py --verify     # re-derive and diff against the committed JSON
python3 conformance/vectors/generate.py              # defaults to --verify
```

Pure Python 3 standard library — no `pip install`, no third-party CBOR package. The canonical
CBOR encoder (RFC 8949 §4.2's deterministic subset: definite-length only, shortest-form integers
and lengths, map keys sorted by their own encoded bytes) is implemented from scratch in
`generate.py` rather than imported, for the same reason the rest of this corpus avoids a closed
loop: trusting a third-party CBOR library's canonicalisation is trusting *something*'s
implementation instead of the RFC text, which is a smaller version of the exact problem
`conformance/README.md` names.

**What `--verify` actually checks, and what it doesn't.** It re-derives every vector from
`generate.py`'s own definitions and diffs the result against the checked-in JSON — so it catches
a hand-edited JSON file drifting from the generator, or a refactor that silently changed a
formula. It does **not** prove `generate.py` agrees with an independent implementation, because
none has been consulted to build this corpus (by design — see above). That cross-check is what
happens when Soko's own test suite re-derives these same vectors independently and compares,
per `conformance/README.md`'s stated plan (the DMTAP pattern, `verify_pub_vectors.py`) — this
repository does not perform that step, and nothing here should be read as if it had.

## Vector file format

Every file has: `id` (a `TRACT-WIRE-VEC-NN` id, this corpus's own numbering — distinct from
`SUITE.md`'s `TRACT-<CATEGORY>-NN` id space, which is `SUITE.md`'s to extend, not this script's),
`suite_ids` (zero or more `SUITE.md` ids this vector bears on), `coverage` (`null` / `"partial"` /
`"full"` — `null` for vectors with no `SUITE.md` id, i.e. plain §16 object-encoding vectors built
to have *something* to check axis/variant encodings against), `section`, `title`, `kind`
(`cbor-encoding` / `derived-value` / `structural-check`), a `description`, and a `cases` array.

JSON has no integer-keyed objects and no byte-string type, so two conventions apply throughout,
implemented in `generate.py`'s `to_jsonable`/`from_jsonable`:

- every JSON object key is the **decimal string of the CDDL integer key**, e.g. `"1"` means map
  key `1` (an int), never a text key `"1"` — this mirrors §16's own key numbering directly;
- a CBOR byte string (a `content-address` or `identity-key`) is written as
  `{"__bytes__": "<hex>"}`, distinguishing it from a CBOR text string (a plain JSON string).

Every `content-address` and `identity-key` value in this corpus is a **placeholder** — see
`placeholder_bytes()` in `generate.py`. §16.3 types both as plain `bytes`; their real internal
structure (a multihash-style prefix, a real public key encoding) is substrate-defined (§16.2)
and out of scope. The placeholders are ASCII labels zero-padded to a fixed length, never a hash
of anything, specifically so nobody mistakes one for a real address by looking at it.

## Coverage — the honest count

**8 of 39 `SUITE.md` case ids have a supporting vector (6 full, 2 partial). 31 are not yet
computable**, each for a stated reason — mostly because the invariant needs a substrate primitive
(a real hash, a real signature) this corpus is not allowed to invent, or because it is a
runtime/network behaviour rather than a static byte computation, or because the §16 text available
does not yet name the object the case is about.

| SUITE.md id | Status | Vector file | Why |
|---|---|---|---|
| TRACT-PUBSEAL-01 | Not yet computable | — | "Identifies or is linkable to a natural person" is a semantic classification, not a CDDL-checkable shape; and whether these catalogue objects are individually *signed* (so the unknown-key-rejection rule would even apply to a smuggled field) is explicitly left open by §16.8 ("Whether `Offer` carries its own signature..."). Resolving that open question here would be inventing an answer, not deriving one. |
| TRACT-PUBSEAL-02 | Not yet computable | — | The stated non-confusability mechanism is content-address **domain-separation tags** (§16.4) — a substrate hash-construction detail, explicitly out of scope (§16.2: TRACT invents no new address scheme). |
| TRACT-PUBSEAL-03 | **Full** | `review.json` | `Review`'s field set is fully closed in §16.5.5's CDDL (keys 1-6, no `?`-optional beyond key 5), and §16.5.5's own prose confirms `Review` **is** signed (unlike `Offer`) — so §16.2's unknown-key-rejection rule applies unambiguously. Also checks the explicit `score, 0..5` bound. |
| TRACT-PUBSEAL-04 | Not yet computable | — | No storefront render-bundle CDDL shape exists anywhere in `16-wire-format.md` yet. |
| TRACT-CAT-01 | **Partial** | `productrecord-canonical-byte-convergence-and-divergence-tract-cat-01-partial.json` | Covers the precondition — canonical-CBOR-byte equality is independent of a publisher's field-insertion order, and a genuine content difference produces different bytes. The content **address** itself (`multihash prefix ‖ digest`, §16.3) is substrate-defined and not computed here — no hash is invented to fill that gap. |
| TRACT-CAT-02 | Not yet computable | — | Index-vs-feed precedence is a client trust-policy behaviour, not a wire-byte computation. |
| TRACT-CAT-03 | **Full** | `offer-missing-an-axis-is-structurally-rejected-tract-cat-03.json` | §16.5.2's `Offer` CDDL has no `?` on keys 1-4 (Item/Availability/Fulfilment/Consideration) — directly checkable structurally. |
| TRACT-CAT-04 | Not yet computable | — | "Advisory, unverified, never authority" is a client trust-weighting policy; nothing encodes differently on the wire because of it. |
| TRACT-AVAIL-01 | Not yet computable | — | Requires re-evaluating a *live* seller signal at checkout time — a runtime/network behaviour, not a static object computation. |
| TRACT-FULF-01 | **Full** | `place-of-supply-derivation-for-all-7-fulfilment-variants-tract-fulf-01-tract-fulf-02.json` | §4.3's derivation table is transcribed and applied to all 7 `Fulfilment` variants, including the exact forcing example (event venue vs. seller/buyer countries) the case names. |
| TRACT-FULF-02 | **Full** | (same file) | The `ship` case demonstrates the anchor binding only once the buyer's destination choice is recorded, per §4.8. |
| TRACT-CART-01 | Not yet computable | — | Cross-replica bounded-counter inventory is a distributed/CRDT runtime invariant; §16 as read doesn't give the inventory-quota wire shape to encode a static case against. |
| TRACT-CART-02 | Not yet computable | — | Same reason as CART-01 — an operator-visible runtime state, not a decodable object. |
| TRACT-CART-03 | Not yet computable | — | Multi-seller checkout independence is a client behaviour across multiple sealed `Order`s over time, not a single object's bytes. |
| TRACT-ORDER-01 | Not yet computable | — | Needs a real signature scheme; TRACT inherits signatures from the substrate (§16.2) and none is invented here. |
| TRACT-ORDER-02 | Not yet computable | — | The order state machine's timeout-expiry table lives in §18, which was not read for this pass; no computable content to cite yet. |
| TRACT-ORDER-03 | Not yet computable | — | "Never observable by a third party" is a network/deployment property, not a property of one object's bytes. |
| TRACT-DELIV-01 | **Full** | `billable-weight-and-local-rate-card-price-lookup-tract-deliv-01.json` | §16.5.3's `billable = max(actual, L*W*H / dim_divisor)` formula, plus a local bracket-price lookup against a published `RateCard`, computed with no live call — all three cases use exact division so the (genuinely unstated) rounding rule is never guessed at. |
| TRACT-DELIV-02 | Not yet computable | — | "Proof of transfer only, not of recoverability/insurance/delivery" is a semantic scope statement about what an attestation means, not a byte-level check. |
| TRACT-DELIV-03 | **Full** | `route-totals.json` | §16.7's "arithmetic across currencies is refused, never coerced" applied directly to summing route legs: same-currency legs sum; mixed-currency legs are refused and the refusal is disclosed, not converted. |
| TRACT-SETTLE-01 | **Partial** | `escrowscope-checkout-intersection.json` | The five *set-valued* `EscrowScope` fields (§16.5.4) intersect as literal set intersection per §9.4, and an empty result on any of them is refused and disclosed — that part is direct. Marked partial because §9.4 names the two non-set fields (`max_order_value`, `excluded_categories`) as intersected but not what "intersect" means numerically for them; the vector applies a stated, flagged reading (ceiling → min, exclusions → union) rather than presenting it as spec text. |
| TRACT-SETTLE-02 | Not yet computable | — | Rail-class substitution requiring "explicit party agreement" is a process/consent invariant across an order's history, not a single object's bytes. |
| TRACT-SETTLE-03 | Not yet computable | — | "Never the only object standing in for a settlement event" is about what else is or isn't present in a transcript, not decodable from `PaymentAttestation`'s bytes alone (which are already exercised in `paymentattestation.json`). |
| TRACT-SETTLE-04 | Not yet computable | — | Requires a signed escrow-ruling object whose shape is not given in the §16 text read for this pass, plus a real signature. |
| TRACT-TRUST-01 | Not yet computable | — | Index-side acceptance policy for reviews without a purchase attestation; a client/index behaviour, not a wire-byte computation (the `Review`/`PurchaseAttestation` shapes themselves are covered in `review.json`). |
| TRACT-TRUST-02 | Not yet computable | — | "No party computes a single network-wide score" is a prohibition on a computation *not* being performed anywhere in the network — not expressible as an input/output vector. |
| TRACT-TRUST-03 | Not yet computable | — | Supersede-based retraction and tombstone-serving is a feed/network behaviour over time, not a single object. |
| TRACT-JURIS-01 | Not yet computable | — | Needs §11.3's in-region-responsible-person field, not present in the §16 text read for this pass. |
| TRACT-JURIS-02 | Not yet computable | — | **Notable gap, not just an omission**: §16.6's `Order` CDDL as currently written carries no fields at all for the four jurisdictional anchors this case requires — see this file's "discipline" section above. |
| TRACT-GW-01 | Not yet computable | — | Origin isolation between two rendered storefronts is a deployment/runtime property, no wire object to check. |
| TRACT-GW-02 | Not yet computable | — | "No code-path access to identity keys or the raw object store" is a process-boundary property, not a decodable object. |
| TRACT-GW-03 | Not yet computable | — | Byte-for-byte comparability of independent re-renders needs the render-bundle shape (see PUBSEAL-04) plus an actual rendering pipeline. |
| TRACT-ABUSE-01 | Not yet computable | — | Feed-head rollback detection needs the substrate's feed-sequence mechanism (§0.3 capability ②), out of scope here. |
| TRACT-ABUSE-02 | Not yet computable | — | Per-publisher rate/quota enforcement is a holder-side runtime policy, not a wire-byte computation. |
| TRACT-ABUSE-03 | Not yet computable | — | Cold-contact challenge/proof-of-work gating is inherited substrate behaviour triggered at delivery time, not a static object. |
| TRACT-PROFILE-01 | Not yet computable | — | Node advertisement/rejection behaviour at the point of receipt; not a property of a decoded object's bytes. |
| TRACT-PROFILE-02 | Not yet computable | — | Same reason as PROFILE-01 — a node's own behaviour, not an object. |
| TRACT-REG-01 | Not yet computable | — | §16.8 states the extension-key policy for the axis unions is itself **open** ("refuse, or preserve and refuse only on acting... needs stating here in grammar terms"). This corpus cannot cover a case whose grammar-level rule the spec says is still undecided. |
| TRACT-SM-01 | Not yet computable | — | The full per-transition timeout-expiry table is §18 content, not read for this pass. |

## What's next, if this is picked back up

- §18's state machines (order/offer/consignment/escrow) would unlock TRACT-ORDER-02, TRACT-SM-01,
  and sharpen TRACT-SETTLE-02/04.
- §11's jurisdiction fields, once given a wire shape (and once `TRACT-JURIS-02`'s gap above is
  resolved one way or the other), would unlock TRACT-JURIS-01/02.
- Nothing here should be extended to cover TRACT-ORDER-01/PUBSEAL-02/ABUSE-01, or any other case
  needing a real signature or content address, until the DMTAP substrate's own primitives are
  in scope for TRACT to profile — inventing one here to "finish the count" is exactly the failure
  `conformance/README.md` exists to prevent.
