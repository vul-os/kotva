# 10. Versioning, Conformance, Governance

## 10.1 Protocol versioning

- Every wire object is **version-discriminable**, but not all via an in-band field. `Envelope`
  carries an explicit format-version field (`v`, §18.3.1). The other top-level signed objects
  (`Identity`, `DeviceCert`, `RecoveryPolicy`, `KeyRotation`, `MoveRecord`, `LocationRecord`, and
  the group/auth objects) derive their format version from their **domain-separation tag**
  (`"DMTAP-v0/…"`, §18.9) together with the `v=dmtap<N>` anchor published in DNS (§3.2): the
  DS-tag is part of every signing preimage, so a verifier reconstructs the preimage under a
  definite structural version and a mismatch simply fails signature verification (fail closed) —
  it never guesses. A future structural revision increments **both** the DS-tag (`"DMTAP-v1/…"`)
  and the DNS `v=` anchor in lockstep, giving an unambiguous discriminator without bloating every
  object with a redundant `v` byte.
- Every wire object also carries an algorithm `suite` (or inherits the enclosing object's, per
  §18.1.4).
- Unknown `v`/`suite` MUST be rejected (fail closed), never guessed.
- New message kinds use the reserved `0x40–0x7f` range (§2.3); a node ignores unknown kinds it
  is not required to process, but MUST NOT ack a kind it cannot validate.

## 10.2 Capability negotiation (dual-stack)

- A sender discovers a recipient's capabilities via the recipient's `Identity`/DNS record
  (native DMTAP vs legacy-only) and picks the path per-recipient (§3, §7.6).
- `system` MOTEs (kind `0x0a`) carry capability announcements between nodes (supported suites,
  privacy tiers, MLS ciphersuites, extensions, KT log-types, transport substrates).
- **Anti-rollback: capability announcements are monotonic (normative).** A capability
  announcement is authenticated (it rides inside a `system` MOTE authenticated to the recipient,
  §2.7) but, without a version, a replayed *older* announcement could **suppress a capability the
  peer has since advertised** (e.g. hide that a peer now supports a stronger suite or the KT-v1
  log-type, forcing a downgrade). Every capability announcement MUST therefore carry a
  **monotonic `caps_version` (`u64`)**, and a receiver MUST **reject any announcement whose
  `caps_version` is older-than-or-equal-to** the last one it has accepted from that peer
  (`ERR_CAPABILITY_ANNOUNCE_ROLLBACK`, `0x030A`, §21.5) — the identical rollback-defense rule
  the spec applies to `Identity.version` (§1.3), `LocationRecord.seq` (§4.2, §16.2), and
  `GroupState.version` (§5.8.2). A receiver retains the highest `caps_version` seen per peer;
  capabilities are only ever added or upgraded across increasing versions, never silently rolled
  back by a stale replay. Absence of a recognized capability token in the *current* (highest)
  announcement is still inconclusive, not a negative assertion (§21.22).
- **Structural extension of signed objects rides capability negotiation.** Because a decoder of
  a **signed** object fails closed on any unknown integer key (§18.1.2, so the signing preimage
  stays unambiguous), a sender MUST include a reserved `≥ 64` extension field in a signed object
  **only toward a peer that has advertised support** for that extension in its capability
  announcement — old peers therefore never receive a key they would reject. This is the
  normative path for extending signed objects; the "ignore-unknown" affordance is confined to
  **unsigned** objects and to `Headers.ext` (§18.1.2, §21.20).
- **Opportunistic upgrade**: a legacy correspondent is reached via the gateway; if they later
  publish a DMTAP key, native delivery is used automatically. No flag day.

## 10.3 Conformance levels

An implementation is **DMTAP-conformant** at a level if it passes the corresponding suite:

| Level | Requires |
|-------|----------|
| **Core** | Identity (§1), MOTE (§2), naming resolution + TOFU + **v0-minimal KT publish/verify with fail-closed-on-unreachable** (§3), mesh delivery + `deliver`/`ack` (§4), MLS 1:1 (§5), recipient policy incl. cold-sender challenge gating (§9) |
| **Private** | Core + mixnet (Sphinx packet + directory + 3-hop stratified paths + key-epoch rotation, §4.4.1–§4.4.4) + sealed sender + cover traffic + the **anti-active-adversary mechanisms** (per-epoch replay caches, Poisson mixing, loop-cover attack detection, entry guards + operator diversity, and **fail-closed no-downgrade**, §4.4.6–§4.4.9) + privacy tiers (§4, §6). The user-selectable **high-security profile** (§4.4.10) and **PQ-Sphinx** (§4.4.12) are OPTIONAL. |
| **Groups & Files** | Core + MLS groups + content-addressed file transfer (§5) |
| **Legacy** | Core + gateway inbound/outbound + DKIM delegation (§7) |
| **Clients** | Core + JMAP; IMAP/POP/SMTP-submission compat RECOMMENDED (§8) |
| **Auth** | Core + DMTAP-Auth login ceremony with origin binding + key-bound sessions (§13); OIDC bridge RECOMMENDED (§13.6) |

**Core is an interoperability floor, not a production target.** Because the standing default
privacy tier for mail (and for every control MOTE) is `private` (§4.6), a Core-only node cannot
operate at the protocol's own default without also implementing **Private**. Therefore a
**production** node that carries user mail **MUST** implement **Private** as well as Core;
`fast`-tier-only operation is a deliberate, user-surfaced downgrade (§4.6, §6), never the silent
default. Core exists so a minimal or special-purpose implementation (e.g. a gateway, a test
harness) can interoperate on the message spine, not so a shipping mail client can skip metadata
privacy. Privacy is a first-class requirement of the protocol, not an optional add-on level — the
Private level is the *normal* baseline, and the mixnet it depends on is fully specified in §4.4 /
§6.3 (Sphinx/Loopix, with the parameters of §16.3) rather than left to implementations.

The **conformance test suite** (in `conformance/`, TBD) is the *operational definition* of
compatibility. "DMTAP-compatible" means "passes the suite," not "resembles the reference." This
is the primary defense against fragmentation.

## 10.4 The spec is authoritative

Independent implementations MUST be buildable from this specification alone. The Rust reference
in `node/` and `gateway/` is a proof and a set of libraries, **not** normative. Where reference
and spec disagree, the spec governs (or the discrepancy is filed as a bug).

## 10.5 Governance & licensing (intent)

- **Standards track:** pursue an **IETF Internet-Draft** for the wire protocol and object
  formats, aiming for RFC status (as JMAP and MLS did). Neutral governance is what lets
  competitors adopt without fearing capture.
- **Licensing:** the **specification** and the **reference implementation** (node, gateway,
  client, libraries) ship under the **MIT license** (Apache-2.0 dual-licensing under
  consideration for its explicit patent grant). Everything a user touches and everything trust
  depends on is open; permissive licensing maximizes adoption by any party, competitors
  included.
- **Open software + paid operations:** commercial sustainability comes from a thin, **private
  control-plane** (a hosted operator) that implements the **operator seam** (`crates/dmtap-seam`)
  to bill *operations* — never gating any protocol or privacy feature. See §12 for the full
  model and the inviolable rule (privacy/crypto are never behind the seam).
- **Reuse over reinvention:** DMTAP deliberately composes existing standards — MLS (RFC 9420),
  HPKE (RFC 9180), JMAP (RFC 8620/8621), libp2p, DNS/DNSSEC/SVCB, key transparency, Privacy
  Pass. The novelty is the *composition and transport*, not new cryptography.

## 10.6 Roadmap markers (non-normative)

- **v0:** Core + Private (minimal KT via TOFU+pinning) + Groups & Files + Legacy gateway.
- **v1 hardening:** **KT-v1 is specified and capability-negotiated** — the full CONIKS/
  key-transparency profile (federated multi-log with `> n/2` quorum-audited bindings, STH
  gossip, monitor/auditor roles, and equivocation halt-alert) is normatively defined as log-type
  `0x02` (§3.5.2, §21.19) and selected per §10.2, with v0-minimal (§3.5.1) remaining the
  interoperable Core default. Also: onion-routed bulk; anonymous tokens at scale; PQ suite `0x02`
  migration; optional self-sovereign naming backend.
- **Later research:** stronger private contact discovery; scalable private retrieval for
  hostile-buffer scenarios; deniable-group properties; metadata-privacy for very large files.
