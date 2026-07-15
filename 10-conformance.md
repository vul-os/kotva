# 10. Versioning, Conformance, Governance

## 10.1 Protocol versioning

- Every wire object carries a format version (`v`) and algorithm suite (`suite`).
- Unknown `v`/`suite` MUST be rejected (fail closed), never guessed.
- New message kinds use the reserved `0x40–0x7f` range (§2.3); a node ignores unknown kinds it
  is not required to process, but MUST NOT ack a kind it cannot validate.

## 10.2 Capability negotiation (dual-stack)

- A sender discovers a recipient's capabilities via the recipient's `Identity`/DNS record
  (native DMTAP vs legacy-only) and picks the path per-recipient (§3, §7.6).
- `system` MOTEs (kind `0x0a`) carry capability announcements between nodes (supported suites,
  privacy tiers, MLS ciphersuites, extensions).
- **Opportunistic upgrade**: a legacy correspondent is reached via the gateway; if they later
  publish a DMTAP key, native delivery is used automatically. No flag day.

## 10.3 Conformance levels

An implementation is **DMTAP-conformant** at a level if it passes the corresponding suite:

| Level | Requires |
|-------|----------|
| **Core** | Identity (§1), MOTE (§2), naming resolution + TOFU + **v0-minimal KT publish/verify with fail-closed-on-unreachable** (§3), mesh delivery + `deliver`/`ack` (§4), MLS 1:1 (§5), recipient policy incl. cold-sender challenge gating (§9) |
| **Private** | Core + mixnet + sealed sender + cover traffic + privacy tiers (§4, §6) |
| **Groups & Files** | Core + MLS groups + content-addressed file transfer (§5) |
| **Legacy** | Core + gateway inbound/outbound + DKIM delegation (§7) |
| **Clients** | Core + JMAP; IMAP/POP/SMTP-submission compat RECOMMENDED (§8) |
| **Auth** | Core + DMTAP-Auth login ceremony with origin binding + key-bound sessions (§13); OIDC bridge RECOMMENDED (§13.6) |

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
- **v1 hardening:** full CONIKS/KT with monitoring + gossip; onion-routed bulk; anonymous
  tokens at scale; PQ suite `0x02` migration; optional self-sovereign naming backend.
- **Later research:** stronger private contact discovery; scalable private retrieval for
  hostile-buffer scenarios; deniable-group properties; metadata-privacy for very large files.
