# 8. Client Access

The node is a multi-protocol front-end over one MOTE store. Every protocol is a *view* of the
same mailbox. Client-facing protocols terminate **on the node**; the node is reached through
the mesh (SNI/stream routing over the relay/mesh) so no static IP is needed.

## 8.1 JMAP (native)

- The node exposes **JMAP** (RFC 8620 / RFC 8621) over a local HTTP endpoint (and, for remote
  devices, over an authenticated mesh stream).
- JMAP is the modern sync surface: `query`, `get`, `set`, `changes`, push. It maps directly
  onto the MOTE store and the device-cluster CRDT (§5.6).
- New DMTAP-native clients SHOULD prefer JMAP + the native MOTE/MLS APIs, which expose DMTAP-only
  features (identity verification, postage, privacy tier, file manifests).

## 8.2 IMAP / POP / SMTP-submission (compatibility)

To let existing mail clients (Apple Mail, Outlook, Thunderbird, mutt) work unchanged:

- The node runs **IMAP**, **POP3**, and **SMTP-submission** servers locally, projecting the
  MOTE store as folders/flags and accepting outbound submissions.
- Reached through the mesh (SNI-passthrough / stream routing); the node terminates TLS and
  speaks the legacy protocol; the relay/mesh never decrypts.
- **Auth = app-passwords**: the node issues app-specific passwords mapped to the identity, so
  legacy clients authenticate without touching the keypair.
- **Encryption is transparent to the owner's own client**: the node decrypts MOTEs and presents
  normal RFC 5322/MIME to the authenticated client. The E2E boundary is between *parties*, not
  between the owner and their own device.
- Outbound submission → the node converts to a MOTE (native) or routes to a gateway (legacy
  destination).

Compatibility support MAY be deprecated over time (like SMTP) as native clients mature; it is
not required for conformance, but is RECOMMENDED for adoption.

## 8.3 Multi-device

Devices form the owner's personal MLS cluster (§5.6); each runs its own client surface and
syncs the mailbox/flags/labels/file-index via encrypted CRDT over the mesh. The always-on node
anchors receipt while other devices sleep.

## 8.4 Calendar & contacts (first-class, decentralized)

Calendar and contacts are **not** separate central services — they are additional **MOTE kinds**
stored in the same node, end-to-end encrypted, synced across the device cluster (§5.6), and
shared/invited via the same MLS groups (§5) as everything else. They inherit the full
decentralized model: your node holds them, no provider can read them, and there is no central
CalDAV/CardDAV server.

- **Native:** calendar events and contacts are represented as **JSCalendar (RFC 8984)** and
  **JSContact (RFC 9553)** MOTEs, synced via **JMAP** (Calendars/Contacts) alongside mail (§8.1).
  Calendar invitations and scheduling (iTIP-style) ride as MOTEs between participants — no
  central scheduling server; free/busy and RSVP are messages, not a server query.
- **Compatibility:** the node exposes **CalDAV (RFC 4791)** and **CardDAV (RFC 6352)** servers on
  `localhost`/over the mesh, projecting the calendar/contact MOTE store as iCalendar (RFC 5545)
  and vCard (RFC 6350) so existing clients (Apple Calendar/Contacts, Thunderbird, DAVx⁵) work
  unchanged — reached through the mesh with app-passwords, exactly like IMAP (§8.2).

Compatibility DAV surfaces MAY be deprecated over time like the mail ones; native JMAP + MOTE is
the forward path.

## 8.5 The decentralization invariant (all data classes)

Every data class DMTAP carries — **mail, chat, files, calendar, contacts, identity, and login
(§13)** — obeys the same rule: it lives on the **user's node**, is **end-to-end encrypted**,
syncs across the user's **device cluster**, shares via the same **MLS groups**, and routes over
the same **mesh/mixnet** — with **no central server** for any of it. Legacy protocols
(IMAP/POP/SMTP/CalDAV/CardDAV) and the OIDC bridge (§13.6) are **edge-compat surfaces only**;
they never become a central store or a required intermediary. The node is the authority for
*everything*, uniformly. There is no data class that quietly depends on a central service.
