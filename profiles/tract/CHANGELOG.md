# Changelog

All notable changes to the TRACT specification. This project uses semantic versioning for the
document: MAJOR for wire-incompatible changes, MINOR for additive sections, PATCH for corrections
that change no bytes.

## [0.1.0] — 2026-07-21

Initial draft. Nothing here is normative yet except §0.

### Added

- **§0 Overview** — goals, non-goals, the DMTAP substrate adoption, roles, the single operator
  class, the public/sealed quadrant split, where state lives, the document map, and the normative
  glossary.
- **§1–§20** — scoped stubs. Each states what it will specify, which existing standards it
  profiles, and what is still open. They are marked non-normative and the linter enforces that they
  stay that way until deliberately promoted.
- **§21 Grounding** — a July 2026 adversarially-verified literature pass, recorded with its own
  coverage caveats, including the findings that contradict this specification.
- `tools/lint.py` — five checks (dangling section refs, document-map agreement, normative language
  in a stub, unmarked stub, reference cited but not listed).
- `build/` — markdown to PDF via headless Chrome, no LaTeX, inherited from the DMTAP build.

### Decided

- **One operator class**, the gateway (storefront rendering and/or settlement), rather than two.
  Bundling them matches how the standing that underwrites both is actually acquired.
- **Four axes** for every offer, rather than category-specific product types.
- **No personal data in the public quadrant**, because published objects are irrevocable and
  erasure rights cannot be satisfied against them.
- **Jurisdiction as a machine-readable field**, with four separate anchors — seller establishment,
  buyer residence, place of supply, delivery destination — because conflating them is the most
  common commerce-tax error and an event held abroad breaks any two-anchor model.
- **No network-wide published score**, because computing one requires the authority being removed.

### Known weakest points

Recorded rather than deferred, all in §21:

- Content addressing is a sound mechanism for product identity but an **unproven** solution;
  canonicalisation is where the real work is.
- "Any node may build an index" does not prevent one index becoming the de facto gatekeeper. This
  is the weakest load-bearing claim in the document.
- An offline seller is invisible, not merely slow.
- Opt-in escrow's measured failure mode is that bad actors decline it.
