# DMTAP Conformance

The conformance suite is the **operational definition** of "DMTAP-compatible": an
implementation conforms at a level (Core / Private / Groups&Files / Legacy / Clients / Auth,
see spec §10) if and only if it passes the corresponding vectors here.

Planned:
- `vectors/` — canonical test vectors (MOTE encode/decode, signatures, content addresses,
  HPKE seals, MLS group ops, name→key resolution, DMTAP-Auth challenge/response).
- `harness/` — a language-agnostic runner that drives an implementation against the vectors.

Status: to be authored once the wire format (spec §2) is frozen. Until then the reference
implementation (Envoir) serves as the interop touchstone, but the vectors — not the reference —
are normative once published.
