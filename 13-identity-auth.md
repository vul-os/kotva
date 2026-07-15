# 13. DMTAP-Auth — Decentralized Identity & Login

Your DMTAP identity is a keypair (§1) with a human name resolvable to that key (§3). The same
identity that receives your mail can log you in **everywhere**, with **no central identity
provider**. This section specifies DMTAP-Auth: sovereign, decentralized web login built on the
identity and naming layers, plus a bridge so existing "Sign in with OIDC" apps work unchanged.

The governing principle mirrors the rest of DMTAP:

> **Your key is your identity — for mail, for messaging, and for login.** No provider can
> revoke it, surveil it, or lock you out. Your node is your identity provider.

## 13.1 Goals & non-goals

DMTAP-Auth MUST provide:

1. **Decentralized login** — a relying party (RP) authenticates you as `alice@yourdomain` by
   verifying a signature from your key; there is no Google/Okta in the middle.
2. **Phishing resistance** — a signature obtained by a look-alike site MUST NOT authenticate to
   the real site.
3. **No bearer tokens after login** — sessions are bound to your key (proof-of-possession), so
   a stolen token is useless without the key.
4. **Legacy interoperability** — existing OIDC/OAuth relying parties can consume DMTAP-Auth
   through a compatibility bridge.
5. **Recoverable & revocable** — losing a device does not lose your identity (§1.4), and a
   single app/device session can be revoked without rotating your whole identity.

Non-goals: replacing WebAuthn/passkeys (DMTAP-Auth *uses* them); acting as an authorization
server for arbitrary third-party API scopes beyond login + basic profile + delegated
capabilities (§13.5).

## 13.2 Substrate reuse

DMTAP-Auth introduces almost no new cryptography — it reuses:

- **Identity** (§1): root key `IK`, device subkeys, the signed `Identity` object.
- **Naming** (§3): `name → key` via DNS + key transparency — this *is* issuer discovery.
- **Recovery** (§1.4): because your key now guards *all* your logins, recovery is even more
  load-bearing; DMTAP-Auth inherits it directly.
- **Transparency log** (§3.5): device/session authorizations and revocations are logged, so
  unauthorized login grants are detectable (§13.4).

## 13.3 The native login ceremony

```
1. RP shows "Sign in with DMTAP"; user enters  alice@yourdomain
2. RP resolves name → key + auth endpoint      (§3 lookup; DID/OIDC discovery, §13.6)
3. RP creates a Challenge:
     { rp_origin, nonce, issued_at, exp, aud, [scope] }
4. The challenge is presented to a TRUSTED CLIENT on the user's side (browser/OS/app), which
   binds and displays the VERIFIED rp_origin, and runs a WebAuthn/passkey user-verification
   ceremony (§13.3.1)
5. The user's key signs  H(rp_origin ‖ nonce ‖ issued_at ‖ exp ‖ aud)  (canonical, §2)
6. RP verifies the signature against alice's pinned key (§3.4) and that rp_origin == its own
   origin, nonce unused, not expired → authenticated
```

The signed statement is a structured, origin-scoped, nonce-bound challenge (the SIWE/CAIP-122
pattern, hardened — see §13.7). The `aud` field binds the assertion to the intended RP.

### 13.3.1 Origin binding is the load-bearing property (normative)

Phishing resistance comes **entirely** from binding the assertion to the *true* RP origin, and
that binding MUST be injected and enforced by a **trusted client component on the user's side**,
never by the signer trusting a value handed to it by the RP.

- In a browser, this is **WebAuthn** (W3C Rec L2 / CR L3): the browser writes the *observed*
  origin into `clientDataJSON`, the authenticator signs over its hash, and credentials are
  scoped by `rpId`. An assertion produced at `alice-yourdomain.evil.com` cannot validate for
  `yourdomain`. DMTAP-Auth's browser ceremony MUST use WebAuthn for exactly this.
- **The remote-node hazard (CRITICAL).** If the user's *always-on node* signs challenges that
  arbitrary parties relay to it, origin binding **evaporates** — a phisher relays the real
  challenge, gets the node to sign, and replays it. FIDO2 cross-device auth (CTAP 2.2 "hybrid")
  mitigates the analogous remote-approval risk with **BLE proximity**, which a remote node
  cannot use. Therefore DMTAP-Auth REQUIRES, for any node-signed login:
  1. the challenge carries `rp_origin` and `aud`, and the node signs *over them*;
  2. a trusted approval surface (the node's own authenticated client/app, or a paired
     passkey) **displays the verified `rp_origin`** to the human and requires explicit
     approval per login;
  3. the node MUST reject a challenge whose `rp_origin` it cannot attribute to an
     authenticated request channel, and MUST rate-limit and log approvals (consent-farming
     defense).

  Preferred design: the **passkey/WebAuthn ceremony happens in the user's client**, and the
  node's key is only invoked *after* a successful local user-verification bound to the origin —
  i.e. the passkey gates the node's signature. Concretely, use the **WebAuthn PRF extension**
  (over CTAP2 `hmac-secret`) so the passkey deterministically derives the key that unlocks the
  node's signing key — the identity key never leaves the node and never touches the RP (the
  deployed wwWallet pattern). Nodes MUST NOT offer "approve any challenge" modes.

## 13.4 Sessions: key-bound, not bearer

After login, the RP and client establish a session that is **sender-constrained to the user's
key**, so a leaked token cannot be replayed by a thief:

- Sessions SHOULD use **DPoP (RFC 9449)** — each request carries a fresh proof-of-possession
  JWT signed by a session key bound at login — or **GNAP (RFC 9635)** continuation, which is
  key-based end to end.
- Session keys are **per-RP, per-device** ephemeral keys authorized by a device key (§1.2), not
  `IK` itself. This limits blast radius and enables granular revocation.
- **Revocation:** revoking one app/device session publishes a revocation to the transparency
  log and/or a short-lived status endpoint; it MUST NOT require rotating `IK`. Rotating a
  device key (§1.5) revokes all its sessions at once. Losing `IK` and recovering (§1.4) MUST
  invalidate all prior session authorizations.

## 13.5 Capabilities & delegation

For "let this app act on my behalf" (beyond login), DMTAP-Auth uses **capability delegation**
rather than opaque scopes: a signed, offline-verifiable, attenuable capability token
(**UCAN-style**, informative) delegates a *specific, least-privilege* right (e.g. "read
calendar MOTEs for 24h") from `IK`/device key to an app or another of the user's nodes.
Delegations are attenuable (a device can only sub-delegate a subset), time-bound, and
revocable. This also carries authorization **across the user's own device cluster** (§5.6).

## 13.6 Legacy bridge: OIDC / OAuth compatibility

Existing apps speak "Sign in with Google/OIDC," not DMTAP-Auth. The bridge makes DMTAP identity
consumable by them — the same native-path/legacy-bridge duality as mail (§7):

- **Per-user issuer (native-ish).** OpenID Connect Discovery 1.0 (Final) permits
  WebFinger-based **issuer discovery** from a user identifier, with the issuer at any host. So
  `alice@yourdomain` can advertise her **own node as her OIDC issuer**, self-issuing ID Tokens
  (the **SIOP v2** shape — a draft, not final). Deployed precedent: **Solid-OIDC** (user-controlled
  OPs; note it trusts a WebID-named *issuer*, it does not embed keys) and **IndieAuth** (identity =
  a URL you own; a W3C Note + IndieWeb living standard, not a Recommendation — cite the living
  standard). DMTAP-Auth expresses the binding as **`did:web` rooted at the user's mail domain**
  (`did:web:yourdomain:users:alice` → `did.json`), the DID method that matches §3's DNS name→key.
- **Hosted bridge (compat).** Because mainstream RP libraries assume a *fixed* issuer allowlist
  and rarely implement WebFinger discovery or dynamic client registration, DMTAP-Auth defines a
  **bridge OIDC Provider**: a standard, well-known OP whose backend is DMTAP-Auth. Legacy RPs
  add it like any OIDC provider; the bridge performs the §13.3 ceremony and mints a standard ID
  Token. The bridge is a *convenience operator*, swappable and self-hostable — it sees login
  events but never the user's key (it verifies signatures), and it is content-blind to mail.
- **DID interop.** `alice@yourdomain`'s `name→key` binding is expressible as **`did:web`**
  (DNS/HTTPS-rooted, matching §3) so DID-aware verifiers and Verifiable-Credential ecosystems
  (OID4VP, Final) can consume DMTAP identities. `did:key` expresses a raw-key identity (tier A,
  §3.6).

Adoption grows exactly like mail: native RPs verify signatures directly; legacy RPs use the
bridge; the bridge's importance fades as native support spreads.

## 13.7 Grounding & honest limits

Grounded standards (verified):

| Layer | Standard | Status |
|-------|----------|--------|
| Origin-bound ceremony | WebAuthn (W3C Rec L2, CR L3) + CTAP2 (FIDO2) | Rec / final |
| Challenge pattern | SIWE **ERC-4361** (Final) + **CAIP-122** | final |
| Keypair web-auth precedent | Nostr **NIP-07 / NIP-98** | deployed |
| Key-bound sessions | **DPoP RFC 9449** / **GNAP RFC 9635** | RFC |
| Capability delegation | **UCAN v1.0** | community spec (informative) |
| Issuer discovery / self-issued | **OIDC Discovery 1.0** (Final) + **SIOP v2** (draft) | final / draft |
| User-controlled OP precedent | **Solid-OIDC**, **IndieAuth** (living std) | deployed |
| Identifier interop | **DID Core** (W3C Rec), `did:web`/`did:key` | Rec / community |
| Modern OAuth security | **OAuth 2.1** (draft) + **RFC 9700** BCP | draft / RFC |

Honest limits (stated in-product):

1. **Origin binding depends on a trusted client.** The anti-phishing guarantee holds only where
   a trusted browser/OS/app enforces `rp_origin` (WebAuthn). Raw keypair signing where the
   *user visually verifies* the origin (SIWE/NIP-98 style) is **weaker** and MUST NOT be the
   only mode; DMTAP-Auth improves on it via §13.3.1 but a compromised or absent trusted client
   degrades to user-verified security.
2. **Remote-node consent farming.** A remote always-on node approving relayed challenges cannot
   use proximity; the mitigations in §13.3.1 (origin display, per-login approval, channel
   attribution, rate-limit, log) bound but do not eliminate the risk. Passkey-gated signing is
   the safer default.
3. **Key loss = login loss.** Your key now guards every login, so recovery UX (§1.4) is
   critical; this raises the stakes on getting recovery right, not a new vulnerability.
4. **RP adoption is chicken-and-egg.** The OIDC bridge (§13.6) is what makes DMTAP-Auth useful
   before native RP support exists — the same bootstrap strategy as the mail gateway.
5. **SIOP v2 is a draft** and WebFinger issuer discovery is the least-implemented corner of
   OIDC; the hosted bridge is required precisely for this reason, not optional polish.
6. **No PKI beyond DNS/CA.** `did:web` (and DNS-rooted naming, §3) bottoms out at DNS + TLS/CA —
   there is no independent proof binding domain to key, so a DNS/registrar/CA compromise is an
   identity compromise. Key transparency (§3.5) makes such a substitution *detectable*, not
   impossible; high-value use SHOULD add out-of-band verification.
7. **Login is a deliberate identity disclosure.** Authenticating to a relying party intentionally
   reveals your identity *to that RP*; this is opt-in and per-RP, and MUST NOT be conflated with
   or allowed to weaken the mail/messaging metadata-privacy guarantees (§6). Session keys are
   per-RP (§13.4) so RPs cannot correlate you across sites via the auth layer.
