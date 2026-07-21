# 15. Normative & Informative References

DMTAP is a **profile that composes existing standards**. This section is the authoritative
reference list. **Normative** references are required to implement DMTAP correctly;
**Informative** references are precedent, rationale, or optional interop. Full text is not
reproduced here — RFCs are permanently available at `https://www.rfc-editor.org/rfc/rfcNNNN`
and W3C/OpenID/other specs at the URLs given. DMTAP cites them by identifier and profiles them;
where DMTAP narrows or extends a referenced spec, the DMTAP text is normative for DMTAP.

## 15.0 Normative — specification conventions & process

| Ref | Title | DMTAP use |
|-----|-------|-----------|
| **RFC 2119** | Key words for use in RFCs to Indicate Requirement Levels | the MUST/SHOULD/MAY vocabulary used throughout (every section) |
| **RFC 8174** | Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words | only **uppercase** keywords carry the RFC 2119 meaning (§0, §21.1) |
| **RFC 8610** | Concise Data Definition Language (CDDL) | the schema notation of every wire grammar in §18 |
| **RFC 8126 (BCP 26)** | Guidelines for Writing an IANA Considerations Section | the allocation-policy vocabulary (Standards Action / Specification Required / FCFS / Private Use) of §21.13, §21.25 |
| **RFC 3463** | Enhanced Mail System Status Codes | the enhanced status codes in the gateway inbound SMTP mapping (§21.9) |

## 15.1 Normative — cryptography & encoding

| Ref | Title | DMTAP use |
|-----|-------|-----------|
| **RFC 9180** | Hybrid Public Key Encryption (HPKE) | seal-to-key encryption; DHKEM(X25519)/HKDF-SHA256/ChaCha20-Poly1305 (§2) |
| **RFC 8032** | EdDSA (Ed25519) | signatures (§1, §2) |
| **RFC 7748** | Elliptic Curves for Security (X25519) | key agreement (§1) |
| **RFC 9420** | The Messaging Layer Security (MLS) Protocol | group/session security for 1:1, groups, files, multi-device (§5) |
| **RFC 9750** | The MLS Architecture | Delivery/Authentication Service roles → mesh/KT mapping (§5) |
| **Signal X3DH** (Marlinspike & Perrin, 2016) | Extended Triple Diffie-Hellman | authenticated key agreement for the optional deniable 1:1 mode (§5.2.1) |
| **Signal PQXDH** (Kret & Schmidt, 2023) | Post-Quantum X3DH (ML-KEM-768) | PQ variant of the deniable-mode handshake, suite 0x02 (§5.2.1) |
| **Signal Double Ratchet** (Perrin & Marlinspike, 2016) | per-message FS + PCS ratchet, shared-key-MAC auth | the deniable 1:1 session channel (§5.2.1) |
| **RFC 6962** | Certificate Transparency (Merkle log, STH, inclusion/consistency proofs) | the profile for key-transparency `SignedTreeHead`/`InclusionProof`/`ConsistencyProof` (§3.5, §18.4.9–§18.4.11) |
| **UCAN v1.0** (Fission et al.) | User-Controlled Authorization Networks — chained, attenuable capability tokens | the profile for the delegated `CapabilityToken` (§13.5, §18.7.3) |
| **XEdDSA / VXEdDSA** (Perrin, 2016) | Ed25519↔X25519 signing/DH from one key | **historical:** earlier drafts derived the deniable X3DH identity DH key from `IK` via XEdDSA; **retired** in favor of a dedicated `IK`-certified X25519 `idk` so `IK` stays sign-only and hardware-buildable (§5.2.1(a)). No longer used by DMTAP. |
| **RFC 8949** | Concise Binary Object Representation (CBOR) | wire serialization; **deterministic (core) encoding, §4.2** (all objects) |
| **FIPS 203** | ML-KEM (Module-Lattice KEM) | PQ KEM (suite 0x02, §1.1) |
| **FIPS 204** | ML-DSA (Module-Lattice signatures) | PQ signatures (suite 0x02, §1.1) |
| **BLAKE3** | BLAKE3 cryptographic hash | content addressing (§2.2), hash-agile prefix; not FIPS/IETF-standardized — SHA-256 fallback |

## 15.2 Normative — transport, mesh & naming

| Ref | Title | DMTAP use |
|-----|-------|-----------|
| **libp2p specs** | Kademlia DHT, Circuit Relay v2, DCUtR, AutoNAT v2, Noise | mesh, reachability, NAT traversal (§4, §14) — `github.com/libp2p/specs` |
| **RFC 9000** | QUIC | primary transport; connection IDs / migration (§4) |
| **RFC 1035 / 9460** | DNS / SVCB & HTTPS RRs | `name→key` records; underscore-scoped TXT + SVCB (§3.2) |
| **RFC 6376** | DomainKeys Identified Mail (DKIM) | gateway signs-as-domain via delegated selector (§7.3) |
| **RFC 8461 / 7672** | MTA-STS / DANE (SMTP TLS) | enforce TLS on the legacy leg (§7) |
| **RFC 8030** | Generic Event Delivery Using HTTP Push (Web Push) | delivery transport for the optional wake-signaling layer (§4.9) |
| **RFC 8291** | Message Encryption for Web Push | payload encryption of the content-free `WakePing` token — `aes128gcm` under the device push key + auth secret (§4.9, §18.5.6, §18.9.15) |
| **RFC 8292** | Voluntary Application Server Identification (VAPID) for Web Push | the node authenticates to a Web Push endpoint as the VAPID application server (§4.9.3) |
| **UnifiedPush** | UnifiedPush specification (`unifiedpush.org`) | the user-chosen, self-hostable distributor — the decentralized push provider preferred where the platform allows (§4.9.3) |

## 15.3 Normative — legacy mail interop (the gateway & client edges)

| Ref | Title | DMTAP use |
|-----|-------|-----------|
| **RFC 5321** | Simple Mail Transfer Protocol (SMTP) | gateway ↔ legacy world; MX priority/randomization §5.1 (§7, §14) |
| **RFC 5322** | Internet Message Format | RFC822 ⇄ MOTE translation at the gateway (§7) |
| **RFC 2045–2049** | MIME | attachment/body representation across the legacy boundary (§7) |
| **RFC 9051** | IMAP4rev2 | **gateway** legacy-client compat surface, projected from the MOTE store (§7.15, §8.2) |
| **RFC 3501** | IMAP4rev1 | broader **gateway** legacy-client compat (§7.15, §8.2) |
| **RFC 6409** | Message Submission (port 587) | **gateway** legacy-client outbound submission (§7.15, §8.2) |
| **RFC 1939** | POP3 | optional **gateway** legacy-client compat (§7.15, §8.2) |
| **RFC 8620 / 8621** | JMAP (Core / Mail) | the **node's native** modern client sync surface — the node's only client surface (§8.1) |
| **RFC 4791** | Calendaring Extensions to WebDAV (CalDAV) | **gateway** legacy calendar-client compat, projected from calendar MOTEs (§7.15, §8.4) |
| **RFC 6352** | vCard Extensions to WebDAV (CardDAV) | **gateway** legacy contacts-client compat, projected from contact MOTEs (§7.15, §8.4) |
| **RFC 5545** | iCalendar | calendar object body format (§8.4) |
| **RFC 6350** | vCard 4.0 | contact object body format (§8.4) |
| **RFC 8984 / 9553** | JSCalendar / JSContact | native JSON calendar/contact representation (§8.4) |

## 15.4 Normative — identity, auth & anti-abuse

| Ref | Title | DMTAP use |
|-----|-------|-----------|
| **RFC 6749** | OAuth 2.0 Authorization Framework | base framework for the DMTAP-Auth code/token flows (§13) |
| **RFC 7636** | PKCE | REQUIRED on any DMTAP-Auth code exchange (§13) |
| **RFC 9449** | OAuth 2.0 Demonstrating Proof of Possession (DPoP) | key-bound, sender-constrained sessions — no bearer tokens (§13.4) |
| **RFC 9068** | JWT Profile for OAuth 2.0 Access Tokens | token format for the OIDC bridge (§13.6) |
| **RFC 7009 / 7662** | OAuth Token Revocation / Introspection | session revocation without identity rotation (§13.4) |
| **RFC 8414** | OAuth 2.0 Authorization Server Metadata | `.well-known` discovery for the bridge (§13.6) |
| **RFC 9700** | Best Current Practice for OAuth 2.0 Security | security baseline for §13 |
| **RFC 9576 / 9577 / 9578** | Privacy Pass (Architecture / HTTP / Issuance) | anonymous anti-abuse tokens; ARC profile (§9.3) |
| **WebAuthn L2** | Web Authentication (W3C Rec, L2; L3 CR) | origin-bound login ceremony; PRF extension gates the node key (§13.3) |
| **CTAP2** | Client to Authenticator Protocol (FIDO2) | passkey/authenticator transport (§13.3) |

## 15.5 Informative — precedent & rationale

| Ref | Title | Relevance |
|-----|-------|-----------|
| **RFC 9635** | Grant Negotiation and Authorization Protocol (GNAP) | key-based, no-pre-registration alternative for §13 sessions |
| **draft-ietf-oauth-v2-1** | OAuth 2.1 | consolidated modern OAuth (draft) |
| **ERC-4361 / CAIP-122** | Sign-In with Ethereum / Sign-In with X | the structured-challenge signing *pattern* (chain stripped, §13.3) |
| **W3C DID Core 1.0** | Decentralized Identifiers | `did:web` (DNS-rooted) / `did:key` expression of DMTAP identity (§13.6) |
| **IndieAuth** (living std) | Identity = a URL/domain you own | discovery ergonomics precedent (§13.6); W3C Note, not a Rec |
| **SIOP v2** (draft) | Self-Issued OpenID Provider | self-issued ID Token shape for the bridge (§13.6) |
| **Solid-OIDC** (CG report) | WebID + DPoP over OIDC | architectural template for the OIDC bridge; trusts *issuers* (§13.6) |
| **Nostr NIP-07 / NIP-98** | keypair browser signing / HTTP auth | deployed keypair-auth precedent; weaker binding than WebAuthn (§13) |
| **UCAN v1.0** | User-Controlled Authorization Networks | capability-delegation model (§13.5) |
| **Sphinx** (Danezis & Goldberg, 2009) | mix packet format | profiled for the Sphinx cell, replay/tagging resistance (§4.4.1, §4.4.6) |
| **Loopix** (USENIX Sec 2017) | mixnet design | Poisson mixing, loop/drop cover + active-attack detection (§4.4.5, §4.4.7) |
| **Nym** | deployed stratified mixnet | stratified topology + operational descendant of Loopix (§4.4.3) |
| **Anonymity Trilemma** (Das et al., IEEE S&P 2018) | anonymity ↔ latency/bandwidth | the fundamental privacy bound; the high-security-profile lever (§6.6, §4.4.10) |
| **CONIKS** (USENIX Sec 2015) / **RFC 6962** | key transparency / Certificate Transparency | auditable `name→key` (§3.5) |
| **Zooko's Triangle** (Wilcox-O'Hearn, 2001) / **Petname Systems** (Stiegler, 2005) | the naming trilemma — a name can be at most two of *human-meaningful*, *globally-unique*, *authority-free* — and the petname resolution of it | the naming-ladder rationale (§3.9, §3.9.3, §3.9.6, §3.12): the key-name takes the global+authority-free corner (not human-chosen), petnames the human+authority-free corner (local), and a name-chain buys the human+global corner via consensus (§3.12.5) |
| **SLIP-0039** / **RFC 9591 (FROST)** | Shamir mnemonic / threshold signatures | recovery (§1.4) |
| **Chatmail / Delta Chat, Matrix/Sygnal, Signal** | minimal mail server, push gateway, sealed sender | mobile push + relay-mailbox precedent (§14); sealed sender (§6) |
| **Raman, Joglekar, De Cristofaro, Sastry & Tyson** (ACM IMC 2019) | *Challenges in the Decentralised Web: The Mastodon Case* — measured volunteer-instance availability (mean downtime 10.95% vs 1.25% for 2007-era Twitter; 21.3% of instances permanently offline within 15 months) and content concentration (removing the 10 most popular instances erased 62.69% of content; replication cut the loss to 2.1%) | the measured basis for making the offline buffer an **`n`-of-`m` role** rather than a hosted service, and for treating buffering as a permanent structural need (§14.3a, §14.5, §14.7) |
| **Liu, Fass, Hong, Sharif, Ravi, Durumeric, Bailey & McCoy** (ACM IMC 2021) | *Who's Got Your Mail? Characterizing Mail Service Provider Usage* — third-party email-security vendors (ProofPoint, Mimecast, Barracuda) among the top-five providers by MX share and growing 2017–2021 despite not being mailbox providers; self-hosting organizations still outsource inbound filtering | the measured basis for the **authorize-never-classify** rule: anti-abuse forms a second, independently-growing centralized tier, so a gateway that classifies content is permanent by construction (§7.11.4, §9.11) |
| **draft-connolly-cfrg-xwing-kem** (`-10`, Independent Submission stream — **not CFRG-adopted**; "no formal standing in the IETF standards process") | X-Wing hybrid KEM | PQ HPKE KEM combiner (§1.1, §1.3); FIPS 203 standardizes **no** combiner and defers to SP 800-227 |
| **draft-ietf-mls-pq-ciphersuites / -combiner** | PQ-MLS | PQ migration for the messaging layer (§5) |
| **draft-kohbrok-mls-dmls** | Decentralized MLS | ongoing work on MLS over a leaderless mesh (§5.1) |
| **draft-yun-privacypass-arc** | Anonymous Rate-Limited Credentials | the ARC token profile (§9.3) |
| **Vatandas, Gennaro, Ithurburn & Krawczyk** (ACNS 2020) | On the Cryptographic Deniability of the Signal Protocol | the honest bound on X3DH offline vs online/interactive deniability (§5.2.1(e)) |
| **Unger & Goldberg** (PETS 2015/2018) | Deniable Authenticated Key Exchange for secure messaging | deniability model precedent for the 1:1 mode (§5.2.1) |
| **Android Key Attestation / Apple Secure Enclave / TPM 2.0 / FIDO** | hardware keystores + key attestation | non-exportable device keys and the `DeviceCert` attestation hook (§1.2a) |
| **Signal Sesame** (2017) | multi-device session management | per-device-pair session fan-out for the deniable mode (§5.2.1(d)) |
| **Shapiro, Preguiça, Baquero & Zawirski** (INRIA RR-7687, 2011) | Conflict-free Replicated Data Types (OR-Set, LWW-Register) | the concrete CRDT types + strong-eventual-consistency model for the device-cluster metadata merge (§5.6.4) |
| **Kulkarni, Demirbas, Madappa, Avva & Leone** (2014) | Logical Physical Clocks / Hybrid Logical Clocks (HLC) | the `{wall, counter, device_id}` clock keying per-field LWW and its deterministic tiebreak (§5.6.4) |
| **Auvolat & Taïani** (DAIS 2019) | Merkle Search Trees: efficient state-based CRDTs in open networks | prior art for the range-based Merkle summary used in device-cluster backfill (§5.6.3) |
| **Meyer** (2023) | Range-Based Set Reconciliation | the drill-divergent-ranges / skip-matching-subtrees reconciliation exchange (§5.6.3) |
| **Minsky & Trachtenberg** (CPISync; IEEE Trans. Inf. Theory 2003) | efficient set reconciliation | precedent for reconciling two sets by their differences, not full transfer (§5.6.3) |
| **RFC 9052** | CBOR Object Signing and Encryption (COSE) | the integer-keyed-map convention DMTAP objects follow (§18.1.2) |
| **RFC 8392** | CBOR Web Token (CWT) | integer-keyed claims precedent for the map keying (§18.1.2) |
| **RFC 7800** | Proof-of-Possession Key Semantics for JWTs | the `cnf` confirmation-key pattern in the auth assertion (§13.3, §18.9.8) |
| **RFC 5546** | iCalendar Transport-Independent Interoperability Protocol (iTIP) | calendar scheduling/invitation semantics behind the calendar object flows (§8.4) |
| **RFC 5228** | Sieve: An Email Filtering Language | the node-local filtering-rule precedent an implementation MAY reuse verbatim (§17, feature-parity item 3) |
| **RFC 7929** | DANE bindings for OpenPGP (DNS-published keys) | precedent for DNS-published key material, informing the `_dmtap` record design (§3.2) |
| **RFC 1918** | Address Allocation for Private Internets | the private-IPv4 range in the avatar-URL SSRF guard (§3.9.5, `0x011B`) |
| **RFC 4193** | Unique Local IPv6 Unicast Addresses | the ULA range in the avatar-URL SSRF guard (§3.9.5, `0x011B`) |

## 15.6 On reproduction

DMTAP does not vendor the full text of referenced standards. RFCs are reproducible under BCP 78
/ the IETF Trust Legal Provisions with their notices intact, but referencing by identifier is
the norm and avoids staleness. Implementers should read the referenced specs directly at the
URLs above; this document defines only the **DMTAP profile** over them.
