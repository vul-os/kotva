# Governance

## What this document is

TRACT is a specification, not a product. It is published under CC BY 4.0 so that anyone —
including direct competitors of any implementation — may implement, quote and extend it.

## Implementation neutrality

**No implementation is part of the standard.** [Soko](https://github.com/vul-os/soko) is a
reference implementation and carries no privilege: independent implementations must be buildable
from this specification and the substrate documents alone, without reading any code. Where an
implementation and this document disagree, **this document wins** — and if the document turns out
to be unimplementable, that is a defect in the document.

## Changes

- **Additive by default.** New capability goes in a new section or a profile, negotiated by
  capability token, never by a flag day.
- **A section is non-normative until it says otherwise.** RFC 2119 keywords in capitals are the
  marker, and `tools/lint.py` fails the build if a section marked as drafting carries them.
- **Corrections that change no bytes are PATCH.** Anything that changes what an implementation
  puts on the wire is MAJOR, regardless of how small it looks.

## Honest-limits rule

Every section that has a residual — something the design cannot do, or can only do by trusting
someone — **must state it in that section**, not in a footnote and not only in §21. A limit
discovered by an implementer that was known to an author and left unwritten is treated as a spec
defect of the same severity as a wrong byte.

This is why §21 exists and why it is allowed to contradict the rest of the document.

## Evidence

Claims about what decentralized commerce achieves in the field belong in §21 with their sources and
their coverage caveats. A claim that has not been verified is marked unverified rather than
softened into something that reads as established.

## Scope discipline

Nostr's marketplace specification was abandoned in-repo as "too complicated" and implementers were
redirected to a classified-ad primitive. That is a standing caution: a section that cannot be
implemented by a competent reader in a reasonable time is a section that will not be implemented at
all. Prefer profiling an existing standard over specifying a new mechanism, and prefer omitting a
capability over specifying one nobody will build.
