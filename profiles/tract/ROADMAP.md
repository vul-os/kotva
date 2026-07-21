# Roadmap

Sequence, not dates.

## 0.1 — architecture (current)

§0 written; §1–§20 scoped; §21 grounded in verified evidence; build and linter working.

## 0.2 — the catalogue half

The sections that must be right before anything else can be, because everything references them:

- **§2 Catalogue** — the object shapes, and the **canonicalisation rules**, which §21.2 identifies
  as the load-bearing part rather than the addressing.
- **§3–§5 The four axes** — availability, fulfilment, consideration.
- **§16 Wire format** for the above, with conformance vectors.

Blocking question carried from §21.8: whether any middle ground exists between a licensed registry
and a nominal string for cross-publisher product identity. A dedicated pass on entity resolution
under **adversarial** publishers is needed before §2 can claim more than a mechanism.

## 0.3 — the transaction half

- **§6–§7** cart, bounded-counter inventory, sealed orders, the state machine.
- **§8** delivery: rate cards, legs, consolidation.
- **§18** the state machines, including every timeout's expiry behaviour.

## 0.4 — the institutional half

- **§9** settlement seam, rail classes, escrow scope, fail-closed intersection.
- **§10–§11** trust and jurisdiction.
- **§12–§13** gateway and analytics.

Blocking research from §21.8: the achievable Sybil-cost floor for reputation on a signed-feed
substrate, and how erasure rights and trader-traceability mandates interact with immutable
replicated objects. §11 currently answers by assertion.

## 0.5 — conformance

- **§15** profiles and the auditable fail-closed set.
- **§17** error registry.
- **§19** parameters.
- Frozen vectors, and a second independent implementation proving byte-identity.

## The question that decides whether this is worth finishing

From §21.8, and it is not rhetorical:

> Where does a spec put the operator-shaped functions it cannot eliminate — indexer, registrar,
> arbiter, pinner? ONDC answers "one central approval-gating registry". OpenBazaar answered
> "nowhere" and got a gatekeeping default search engine plus liveness-bounded availability. Is
> there a middle design — multiple competing indexers with verifiable completeness or censorship
> proofs, federated or rotating registrars, paid pinning markets — with any deployed precedent and
> measured outcomes?

TRACT's current answer is "one operator class, entered permissionlessly, chosen per order". That is
a claim about the middle. If it cannot be supported, the honest outcome is to say so rather than to
ship a specification whose central premise is unevidenced.
