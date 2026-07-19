# 8. Client Access

The node is **native-only**. It exposes exactly one client surface — **JMAP** (§8.1) — as a
*view* over its one MOTE store, reached over the native **mesh** (§4) so no static IP is needed.
The node runs **no legacy protocol server**. Every legacy surface — IMAP, POP3,
SMTP-submission, SMTP MX/relay, CalDAV, and CardDAV — lives **only on the gateway** (§7), which
is the sole home of all legacy protocols and the sole legacy-client reachability ingress. This
draws a clean line: **native client sync = JMAP on the node; legacy client access = the gateway**.

> **Four different "relays" — do not conflate them** (defined in the §0 glossary). (1) The
> **native mesh relay** (Circuit Relay v2 / DCUtR, §4.1, §4.3) is node↔node reachability: it
> **stays on the node** and is native, not legacy. (2) The **legacy-client reachability
> ingress** (the SNI-passthrough / stream routing that accepts a raw IMAP/TLS connection from,
> e.g., an iPhone Mail app and serves its mailbox) lives **only on the gateway** (§7.15.2) and
> exists solely to serve clients that cannot speak the mesh. (3) The **Relay node class**
> (§14.1) is any public-IP node performing (1) as a role. (4) The **relay-mailbox** (§14.3) is
> the hosted, content-blind queue a mobile-only user drains.

## 8.1 JMAP (native — the node's only client surface)

- The node exposes **JMAP** (RFC 8620 / RFC 8621) over a local HTTP endpoint (and, for remote
  devices, over an authenticated mesh stream).
- JMAP is the modern sync surface: `query`, `get`, `set`, `changes`, push. It maps directly
  onto the MOTE store and the device-cluster CRDT (§5.6).
- New DMTAP-native clients SHOULD prefer JMAP + the native MOTE/MLS APIs, which expose DMTAP-only
  features (identity verification, postage, privacy tier, file manifests).

## 8.2 IMAP / POP / SMTP-submission (legacy — served by the gateway, not the node)

Existing mail clients (Apple Mail, Outlook, Thunderbird, mutt) that cannot speak JMAP + the mesh
are served by a **gateway**, never by the node. The node runs **no** IMAP/POP/SMTP-submission
server; the legacy surfaces and their reachability ingress are specified normatively in §7.15:

- The **gateway** runs **IMAP**, **POP3**, and **SMTP-submission** servers, projecting the
  mailbox as folders/flags and accepting outbound submissions on the client's behalf.
- The gateway is reached over the **legacy-client reachability ingress** (SNI-passthrough /
  stream routing, §7.15) — a raw IMAP/TLS connection from the client arrives at the gateway,
  which terminates TLS and speaks the legacy protocol. This is a **gateway** surface; it is
  **distinct** from the node's native mesh relay (§4.3), which never speaks a legacy protocol.
- **Auth = app-passwords**: the gateway issues app-specific passwords mapped to the identity, so
  legacy clients authenticate without touching the keypair.
- **Honest-privacy consequence (normative).** To speak IMAP/POP the gateway MUST **decrypt** the
  mailbox — so a legacy client's mail is **visible to whatever gateway serves it**. A **private**
  gateway (your own, §7.15.4) is a **zero-third-party** arrangement; a **public** gateway is a
  **deliberate trust decision** (like choosing Gmail) in which that operator can read the mail.
  This is unlike the node's native path: JMAP + the mesh (§8.1) remain **zero-access /
  zero-intermediary**. A client MUST NOT present gateway-served legacy access as end-to-end when
  a non-private gateway serves it.
- Outbound submission → the gateway converts to a MOTE (native destination) or bridges to legacy
  (§7.3).

Legacy support is a **gateway** capability that MAY be deprecated over time (like SMTP) as native
JMAP clients mature; it is not required for node conformance, and it is a RECOMMENDED **gateway**
capability for adoption (§10, §7.15).

## 8.3 Multi-device

Devices form the owner's personal MLS cluster (§5.6); each runs its own client surface and
syncs the mailbox/flags/labels/file-index via encrypted CRDT over the mesh. The always-on node
anchors receipt while other devices sleep.

## 8.4 Calendar & contacts (first-class, decentralized)

Calendar and contacts are **not** separate central services — they are additional **MOTE kinds**
stored in the same node, end-to-end encrypted, synced across the device cluster (§5.6), and
shared/invited via the same MLS groups (§5) as everything else. They inherit the full
decentralized model: your node holds them, no provider can read them, and — on the native path —
there is no central server.

- **Native (on the node):** calendar events and contacts are represented as **JSCalendar (RFC
  8984)** and **JSContact (RFC 9553)** MOTEs, synced via **JMAP** (Calendars/Contacts) alongside
  mail (§8.1). This is the node's native surface and **stays on the node**. Calendar invitations
  and scheduling (iTIP-style) ride as MOTEs between participants — no central scheduling server;
  free/busy and RSVP are messages, not a server query.
- **Legacy (on the gateway):** **CalDAV (RFC 4791)** and **CardDAV (RFC 6352)** are **gateway**
  surfaces (§7.15), never node surfaces. The gateway projects the calendar/contact MOTE store as
  iCalendar (RFC 5545) and vCard (RFC 6350) so existing clients (Apple Calendar/Contacts,
  Thunderbird, DAVx⁵) work unchanged — reached over the legacy-client ingress with app-passwords,
  exactly like IMAP (§8.2). The same honest-privacy consequence applies: to serve CalDAV/CardDAV
  the gateway MUST decrypt, so a non-private gateway serving DAV can read the calendar/contacts;
  the native JMAP path (above) is zero-access.

The legacy DAV surfaces are **gateway** capabilities that MAY be deprecated over time like the
mail ones; native JMAP + MOTE on the node is the forward path.

## 8.5 The decentralization invariant (all data classes)

Every data class DMTAP carries — **mail, chat, files, calendar, contacts, identity, and login
(§13)** — obeys the same rule on its **native** path: it lives on the **user's node**, is
**end-to-end encrypted**, syncs across the user's **device cluster** via **JMAP** (the node's
only client surface, §8.1), shares via the same **MLS groups**, and routes over the same
**mesh/mixnet** — with **no central server** for any of it. The node runs **native surfaces only
(JMAP + the mesh)** and **no legacy protocol server** of any kind. Legacy protocols
(IMAP/POP/SMTP/CalDAV/CardDAV) live **only on the gateway** (§7.15) as **edge surfaces**, and the
OIDC bridge (§13.6) is likewise an edge-compat surface; none of them is a node surface, and on
the native path none becomes a central store or a required intermediary. A legacy client reached
through a **non-private gateway** is served by an intermediary that can read the mail — that is a
deliberate, disclosed trust choice (§8.2, §7.15.3), not a native property. The node is the
authority for *everything*, uniformly, over its native surfaces. There is no data class that
quietly depends on a central service on the native path.

Silent grants that would **redirect or delegate** that data — **auto-forward rules**, new
**capability delegations** (§13.5), and new **RP-session authorizations** (§13.4) — MUST be
surfaced to the owner's device cluster and logged to KT self-monitoring (§3.5), so a
business-email-compromise-style silent redirect or delegation is owner-visible, not covert.

## 8.6 Surfacing the transport-path (UX guidance)

The node exposes a per-message **`ProvenanceRecord`** (§18.8.1) over the client surface (JMAP,
§8.1; mapped in §19.9), and a client **SHOULD surface the transport-path** so a user can see, for
any message, **how it reached them and which trust boundaries it crossed** (§7.8). This is
implementer UX guidance, not a wire requirement.

- **Render a transport-path graph:** `sender → tier (hops) → gateway? → recipient`. Draw the
  arrival **tier** (`private`/mixnet vs `fast`/direct), the **coarse hop descriptor** (the profile
  floor, e.g. "≥ 3 mix hops" — **never** individual mix nodes, which the node does not know and
  MUST NOT invent, §6.8), and the **gateway leg** if one exists.
- **Make pure-mesh vs. gateway-touched visually unmistakable.** A **pure-mesh** message (no
  attestation, `origin = 0`) SHOULD be shown as **end-to-end, never plaintext at a gateway**; a
  **gateway-touched / legacy-origin** message (`origin = 1`) SHOULD be shown as having crossed a
  named gateway (`GatewayAttestation.domain`, its receipt time, and — for inbound — the legacy
  sender), clearly marked **not E2E before the gateway** (§7.2a). Chained gateways (§7.8.3) render
  as ordered hops, with any unverified-domain hop flagged as such.
- **Do not over-claim on `private`.** While the mix fleet is small the client MUST honour the
  §6.6/§4.4.11 disclosure and MUST NOT present `private` as absolute anonymity; the graph shows the
  **boundary crossings**, not a node-by-node trace.
- **Tie it to billing transparency.** For a gateway-touched message the client MAY let the user
  confirm the attestation against the operator's metered legacy operation (§7.9, §12.7) — "this
  message used the gateway, which is why it was billed" — and MUST NOT show a pure-mesh message as
  a billable gateway operation.

## 8.7 Deniable mode, org administration & device attestation (UX guidance)

The hardening mechanisms of §5.2.1, §3.10 and §1.2a each have a **client surface** a conforming
node SHOULD expose. This is implementer UX guidance, not a wire requirement, but the **fail-closed
choices are normative** where cited.

- **Deniable-mode selection (§5.2.1).** A client offering the deniable 1:1 mode MUST present it as
  an **explicit, per-conversation user choice**, never a silent default, and MUST show that a
  deniable thread **repudiates authorship** (no transferable proof) while still authenticating the
  key agreement. When the counterpart has **not** advertised the `deniable-1:1` capability
  (`ERR_DENIABLE_MODE_UNAVAILABLE`, `0x040E`), the client MUST **surface the choice** — send
  non-deniable (MLS 1:1) or don't send — and MUST NOT silently downgrade the user's *expectation of
  deniability*. Clients SHOULD indicate that deniable threads **sync per-device** (Sesame fan-out,
  §5.2.1(d)) rather than through the shared MLS tree, that **one-way** threads do not self-heal
  (PCS needs a reply, §5.2.1(b)), and — on device loss — that the session was **re-established**
  (§5.2.1(f)).
- **Org-admin console (§3.10, §13.5.1).** For a domain admin, the client SHOULD surface a
  member/directory console driving `provision-member` / `publish-directory` / `query-directory` /
  `offboard` (§19.1.5) and capability `delegate`/`revoke` (§19.6.6). It MUST render an
  **`org-managed` (escrowed-key) account honestly** — visibly distinct from a sovereign one, never
  presented as equivalent (`ERR_ORG_MANAGED_UNDISCLOSED`, `0x0115`) — and MUST show every
  grant/revocation as an **owner/authority-visible, KT-logged event** (the BEC-defense
  self-monitoring path, §13.5), so a silently installed admin grant or auto-forward rule is
  alertable.
- **Attestation enrollment prompts (§1.2a).** During device enrollment the client SHOULD prompt to
  generate the device key in a **hardware keystore** and attach attestation evidence
  (`attest-enroll`, §19.1.6), and SHOULD surface when a relying context **requires** attestation and
  the device lacks it (`0x0116`) or its evidence has **expired / the root retired** (`0x0118`,
  prompt to **re-attest**). The client MUST make clear attestation is **advisory hardening** — it
  never overrides the owner's §1.4 authorization — and that verifying it trusts a **vendor
  attestation root** (a disclosed TTP, §1.2a).
