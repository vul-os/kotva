# DMTAP

**Decentralized Message Transfer & Access Protocol** — an open protocol for sovereign,
metadata-private communication (mail, chat, files) and decentralized identity/login, over a
peer-to-peer mesh, with an optional bridge to legacy email.

DMTAP unifies message **transfer** (SMTP's job — deliver to an offline peer) and message
**access** (IMAP's job — sync to your devices) into one decentralized, encrypted protocol, and
extends the same keypair identity into **decentralized web login** (DMTAP-Auth, §13).

- **Protocol:** DMTAP (this repo — the neutral standard)
- **Message object:** MOTE (signed, encrypted, content-addressed)
- **Reference implementation + apps:** Envoir (open source, MIT)
- **Hosted operator (optional, paid):** Envoir Cloud (private)

DMTAP is to Envoir what Matrix is to Element, or JMAP is to Fastmail: an open standard with an
independent reference implementation and a hosted option — none required to speak the protocol.

---

## The specification

This is the **source of truth** for DMTAP. Independent implementations MUST be buildable from
this text alone, without reading the reference code. Where the reference implementation and this
spec disagree, **the spec wins** (or the discrepancy is a bug to reconcile).

## Building the PDF

The spec is plain Markdown. [`dmtap.pdf`](dmtap.pdf) is generated from it — Markdown → HTML →
PDF via headless Chrome, with **highlight.js** syntax highlighting and **mermaid** diagrams
(no LaTeX). To rebuild:

```sh
cd build
npm install            # markdown-it, highlight.js, mermaid, puppeteer-core
npm run build          # → ../dmtap.pdf
```

It drives the Chrome already on the machine (override with `CHROME_PATH=…`); nothing is
downloaded at render time. Diagrams are authored inline as ` ```mermaid ` fenced blocks, so they
stay in the same Markdown a reader edits. See [`build/`](build/) for the script, cover metadata,
and print stylesheet.

## Sections

| # | File | Status | Contents |
|---|------|--------|----------|
| 0 | [`00-overview.md`](00-overview.md) | detailed | Architecture, components, data flows, threat model summary |
| 1 | [`01-identity.md`](01-identity.md) | detailed | Keys, identity lifecycle, recovery policy + rotation, migration |
| 2 | [`02-mote.md`](02-mote.md) | detailed | The MOTE message object: format, hashing, signing, encryption |
| 3 | [`03-naming.md`](03-naming.md) | draft | Names → keys: DNS records, TOFU + pinning, key transparency |
| 4 | [`04-transport.md`](04-transport.md) | draft | Mesh (libp2p), mixnet, sealed sender, cover traffic, reachability, bulk transfer |
| 5 | [`05-messaging.md`](05-messaging.md) | draft | MLS groups, 1:1, chat, files; KeyPackage async join; the unified substrate |
| 6 | [`06-privacy.md`](06-privacy.md) | draft | Threat model, metadata-privacy guarantees, privacy tiers |
| 7 | [`07-gateway.md`](07-gateway.md) | draft | Legacy SMTP bridge: inbound, outbound, DKIM delegation, attestation |
| 8 | [`08-clients.md`](08-clients.md) | draft | JMAP native; IMAP/POP/SMTP-submission compatibility |
| 9 | [`09-anti-abuse.md`](09-anti-abuse.md) | draft | Postage, anonymous rate-limit tokens, proof-of-work, recipient policy |
| 10 | [`10-conformance.md`](10-conformance.md) | draft | Versioning, capability negotiation, conformance suite, governance |
| 11 | [`11-grounding-and-references.md`](11-grounding-and-references.md) | detailed | Verified standards, corrections applied, honest limits, bibliography |
| 12 | [`12-operators.md`](12-operators.md) | detailed | Deployment modes, the operator seam, the inviolable rule, business/licensing model |
| 13 | [`13-identity-auth.md`](13-identity-auth.md) | detailed | DMTAP-Auth: decentralized OAuth / sovereign web login (your key = your identity everywhere) |
| 14 | [`14-scaling.md`](14-scaling.md) | detailed | Node classes (always-on / mobile), horizontally-scalable gateways, relay/buffer scaling, hosted topology |
| 15 | [`15-references.md`](15-references.md) | detailed | Normative & informative references (RFCs/specs DMTAP profiles: SMTP/IMAP/JMAP/CalDAV/CardDAV/MLS/OAuth/HPKE/…) |
| 16 | [`16-parameters.md`](16-parameters.md) | detailed | Numeric parameters (v0): time/replay windows, KT/DHT, mixnet, file tiers, anti-abuse, suites |
| 17 | [`17-parity.md`](17-parity.md) | detailed | Feature-parity audit: every legacy mail/calendar/contacts feature → DMTAP mechanism, sense-checked + security-reviewed |
| 18 | [`18-wire-format.md`](18-wire-format.md) | detailed | Appendix A — Wire format: CDDL grammar for every object, per-field semantics, canonical signing preimages, collected grammar |
| 19 | [`19-operations.md`](19-operations.md) | detailed | Appendix B — Protocol operations: every op (params/pre/post/errors) + worked traces; JMAP mapping |
| 20 | [`20-state-machines.md`](20-state-machines.md) | detailed | Appendix C — State machines: delivery, validation, resolution, reachability, group/committer, auth session, node lifecycle |
| 21 | [`21-errors-iana.md`](21-errors-iana.md) | detailed | Appendix D — Error/status registry (81 codes) + IANA registries + extension/versioning procedure |

## Conventions

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are to be
interpreted as in RFC 2119 / RFC 8174.

- **Serialization:** deterministic **CBOR** (RFC 8949 core deterministic encoding) for all
  wire objects. JSON appears only at the JMAP client boundary.
- **Hashing:** **BLAKE3** (256-bit) for content addressing unless a field specifies otherwise.
- **Signatures:** **Ed25519** (v0); algorithm identifiers make this negotiable (see §1).
- **Public-key encryption:** **HPKE** (RFC 9180) with X25519 + ChaCha20-Poly1305 (v0).
- **Group / session security:** **MLS** (RFC 9420); async initiation uses MLS-native
  **KeyPackages + external commits** (not a separate PQXDH handshake — see §5.3).
- All multi-byte integers are big-endian. All timestamps are unsigned milliseconds since the
  Unix epoch, transported explicitly (nodes MUST NOT rely on synchronized clocks for
  correctness, only for ordering hints and expiry).

## Crypto-agility

Every signed or encrypted structure carries an **algorithm suite identifier** (`suite`).
v0 defines one classical suite and reserves identifiers for post-quantum hybrids
(Ed25519+ML-DSA for signatures, X25519+ML-KEM for KEM). Implementations MUST reject
unknown suites rather than guess. See §1 and §2.

## Non-goals

- **Real-time media (voice/video).** Different architecture (WebRTC/SFU); out of scope.
- **Perfect anonymity against a global active adversary with unlimited resources.** DMTAP
  targets strong metadata privacy against a **global passive adversary**; see §6 for the
  honest boundary.
- **Consensus / blockchain.** DMTAP uses no chain. Self-sovereign naming (§3) is the *only*
  place a name-chain is offered, as an optional name backend.
