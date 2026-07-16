# 17. Feature-Parity Audit — Classic Mail/Calendar/Contacts vs DMTAP

This section is a **sense-check**, not a pitch. For every feature a working email/calendar/
contacts user expects, it states how DMTAP provides it (or doesn't), whether the mapping is
sound in a decentralized/E2E system, and how the security properties compare to legacy. A
"doesn't map / genuinely harder" verdict is treated as a valid, useful outcome — this document
exists to surface exactly those, not to hide them.

## 17.0 Method

Each feature gets four lines:

- **How** — the DMTAP mechanism that provides it, with a `§` back-reference.
- **Sense-check** — does this make sense architecturally in a sealed-sender, E2E, no-central-
  server system, or is it in tension with that model?
- **Security** — better / same / worse than legacy IMAP/SMTP-era email, and why.
- **Open issue** — anything unresolved. "None." if the mapping is complete.

Verdicts used in the summary (§17.5): **Clean** (works cleanly, same or better), **Different**
(provided, but the mechanism or UX genuinely changes), **Harder** (a real capability loss or
added difficulty, disclosed rather than hidden), **N/A** (the legacy feature's premise doesn't
survive translation, and that is correct, not a bug).

52 features are covered: 29 mail, 7 contacts, 11 calendar, 5 cross-cutting.

---

## 17.1 Mail

#### 1. Folders — **Clean**
- **How:** JMAP `Mailbox` semantics over the MOTE store (§8.1); mailbox assignment is part of
  the replicated per-account state kept in sync across the device cluster as an encrypted CRDT
  (§5.6: "the mailbox, read/flag state, labels, and file index are replicated across devices").
- **Sense-check:** Folders are pure client/account-local organization over an already-decrypted
  store; nothing about decentralization touches this.
- **Security:** Same as legacy, but the folder structure itself is encrypted at rest (§6.7) and
  never visible to any intermediary — legacy IMAP folder names/counts are visible to the
  provider by construction; here they are not.
- **Open issue:** None.

#### 2. Labels/tags — **Clean**
- **How:** Same CRDT-replicated state as folders (§5.6); DMTAP's model treats labels and
  folders as the same mechanism (JMAP keywords/mailboxes), not two systems.
- **Sense-check:** Client-local metadata; no tension with the protocol.
- **Security:** Same as legacy in function; better in exposure (never leaves the encrypted
  store/CRDT sync channel).
- **Open issue:** None.

#### 3. Filters/rules (server-side Sieve vs client-side) — **Different**
- **How:** DMTAP has no third-party server that ever sees plaintext, so classic "runs on the
  provider's Sieve server while you're offline" doesn't exist as such. But the **always-on
  node is the user's own authority**, not a third party (§8.2: "the E2E boundary is between
  *parties*, not between the owner and their own device") — it already decrypts every MOTE
  for the owner. Filter rules therefore run **on the owner's own node**, which is functionally
  "server-side" (always-on, runs while the user is offline) without being a *third party's*
  server. This is arguably the correct re-siting of the feature, not a downgrade.
- **Sense-check:** Sound. The property legacy Sieve actually gives users ("rules apply even
  when my client is closed") is fully preserved because the node, not the client, is
  always-on. What's lost is a *provider-hosted* Sieve service independent of any of the user's
  own hardware — that specific shape doesn't survive, correctly, since it would require a
  third party to hold plaintext.
- **Security:** Better: no third party ever evaluates rules against your plaintext. Same
  blast-radius caveat as everything else on the node: a compromised node sees what Sieve
  would have seen anyway.
- **Open issue:** No rule language is specified in v0 (Sieve itself, RFC 5228, MAY simply be
  reused verbatim by an implementation running on the node — nothing prevents it). Left to
  implementations; not a protocol gap since filtering is entirely local to the node.

#### 4. Search (on-device, no server-side index) — **Different**
- **How:** Explicitly a non-goal to centralize (§0.7: "no server-side search"); each node
  indexes its own plaintext mailbox locally. A thin client (phone) either queries its home
  node live or searches its local cache (§14.1: thin clients hold "cache only").
- **Sense-check:** Correct outcome for an E2E system — a searchable *shared* server-side index
  is precisely the kind of central plaintext store DMTAP is designed to not need. The
  interesting nuance: because the "server" (the node) is *yours*, on-device search on the
  always-on node is not actually weaker than a provider's server-side search in capability —
  only a phone's *local* cache-only search is meaningfully limited, and that's a phone-storage
  limit, not a protocol one.
- **Security:** Better — no provider ever builds a searchable index of your mail (a
  historically real target: subpoenas, breaches, ad-targeting scandals all rooted in
  provider-side search indices).
- **Open issue:** No shared/cross-user search corpus exists (see spam classification, item 20)
  — a consequence of the same design choice, not a separate gap.

#### 5. Threading/conversations — **Clean**
- **How:** `Headers.thread` (stable thread id) + `Payload.refs` (reply/reference chain, §2.4);
  clients render either by explicit thread id or reference-graph reconstruction.
- **Sense-check:** Directly maps; MOTEs are inherently unordered on the wire (§2.6) so explicit
  thread/ref fields are the *correct* mechanism (vs. legacy's fragile `In-Reply-To`/`References`
  header heuristics, which routinely break across clients).
- **Security:** Same or better — thread linkage is inside the encrypted payload, invisible to
  the network (legacy threading headers are plaintext SMTP headers, visible to every relay).
- **Open issue:** None.

#### 6. Drafts — **Clean**
- **How:** A draft is a MOTE that has not yet been dispatched — it exists only as local/
  device-cluster CRDT state (§5.6) before a `kind` (§2.3) is ever sent. No protocol object is
  needed because "sent" is the only state transition the wire format cares about.
- **Sense-check:** Sound; drafts are inherently pre-wire and so are naturally client/cluster
  state, syncing across the owner's own devices exactly like read/flag state.
- **Security:** Better — a draft never touches the network, the mixnet, or any intermediary
  until dispatched (legacy webmail drafts are stored server-side in plaintext by the
  provider).
- **Open issue:** None.

#### 7. Attachments (any size) — **Clean**
- **How:** Inline for small, content-addressed chunked manifest for large, with three explicit
  size tiers and their privacy cost (§2.5, §5.5, §6.5, §16.4): inline ≤64 KiB, normal ≤4 MiB
  (full mixnet privacy), large >4 MiB (fast/onion bulk path, weaker metadata privacy).
- **Sense-check:** Clean and in fact a genuine upgrade — no protocol-imposed size cap at all
  (legacy mail is capped ~25 MB by providers); the tradeoff is disclosed rather than hidden
  (privacy weakens as size grows, §6.5).
- **Security:** Content and integrity are strictly better (per-chunk hashes, dedup, resumable
  swarm transfer, §5.5). Metadata privacy is weaker for large files specifically — the spec is
  explicit about this (§6.6.2) rather than pretending otherwise.
- **Open issue:** None beyond the disclosed large-file metadata tradeoff (§6.5), which is a
  known, stated limit, not an unresolved gap.

#### 8. Rich text/HTML — **Clean**
- **How:** `Headers.mime` + `Body` (tstr/bytes, §2.4) carries the content type and body exactly
  as MIME does today; nothing about encryption or decentralization constrains body format.
- **Sense-check:** Non-issue; purely a client-rendering concern.
- **Security:** Same as legacy for content (the usual HTML-mail render-time risks — remote
  image beacons, tracking pixels — are a **client** hygiene concern, unchanged by DMTAP). One
  improvement: since delivery is E2E, a tracking pixel can no longer also be a *provider-side*
  read-confirmation mechanism the way it can when a webmail provider proxies remote images;
  it's still a client-side leak if the client fetches remote content, same as always.
- **Open issue:** None (client responsibility to proxy/strip remote content, as today).

#### 9. Signatures (email sig block) — **Clean**
- **How:** Plain client-local text/HTML prepended to `Body` at compose time. No protocol
  object.
- **Sense-check:** Non-issue.
- **Security:** Unchanged; note this is unrelated to the *cryptographic* `sig` field (§2.4) —
  naming collision with legacy terminology only, worth a footnote in client UX copy so users
  don't conflate "email signature" with "message signing."
- **Open issue:** None.

#### 10. Aliases + plus-addressing — **Clean**
- **How:** `Identity.names` is a list (§1.3); one identity holds many names (key-name, handles,
  `name@domain`) all resolving to the same key, and legacy addresses you already own can be
  folded in as aliases (§3.9.4). Plus-addressing (`you+tag@domain`) is explicitly preserved for
  client-side filtering and resolves to the same key (§3.9.4).
- **Sense-check:** Clean, and stronger than legacy: every alias is cryptographically the same
  identity, so — unlike legacy plus-addressing, which is an unauthenticated convention any
  sender can spoof around — a DMTAP alias cannot be used to impersonate (§3.9.4: "authenticity
  is always the key, not the name").
- **Security:** Better — alias additions/removals are signed `Identity` versions, audited via
  KT (§3.5); a legacy provider can silently disable/redirect an alias, a DMTAP alias change is
  tamper-evident.
- **Open issue:** None.

#### 11. Forwarding (manual) — **Different**
- **How:** Forwarding as *composing a new message that quotes/attaches the original* works
  unchanged — the forwarder decrypts, re-encrypts to the new recipient, done. What does
  **not** carry over is legacy "Redirect"/`Resent-From` semantics — literally re-transmitting
  the original message *as though the original sender sent it*, preserving `From:`. That
  requires forging the original sender's signature, which a DMTAP node cannot do (`Payload.sig`
  is bound to the original sender's key, §2.4).
- **Sense-check:** The impossible half (bit-identical redirect impersonating the original
  sender) is precisely the thing that *should* be impossible in an authenticated system —
  legacy `Resent-From` is a well-known, still-live spoofing vector. Losing it is a feature, not
  a bug.
- **Security:** Better: DMTAP forwarding is always honestly attributed to the forwarder; legacy
  redirect-forwarding is a documented BEC (business email compromise) and phishing amplifier.
- **Open issue:** None — flagged here so implementers don't try to "fix" this by reintroducing
  forgeable redirect.

#### 12. Auto-forward (rule-based, all mail to another address) — **Different**
- **How:** A standing rule on the owner's own node (§8.2 model, same siting as filters, item
  3): decrypt, re-encrypt-and-send-as-new-message to the configured address, for every
  matching incoming MOTE.
- **Sense-check:** Sound mechanically. Worth flagging because silent auto-forward rules are a
  real-world account-takeover technique (attacker adds a hidden Gmail/Exchange forwarding rule
  to exfiltrate mail after a credential compromise) — the same attack is *possible* here if an
  attacker gains node/admin access, since forwarding-rule changes are ordinary local
  configuration, not a signed identity-lifecycle object.
- **Security:** Same residual risk as legacy for this specific attack (it's a node-compromise
  consequence, not a protocol flaw), but DMTAP has the machinery to do *better*: identity and
  recovery-policy changes are logged and self-monitored for owner alerting (§3.5); auto-forward
  rule changes currently are not analogous first-class signed/logged objects.
- **Open issue:** **Gap.** Recommend implementations surface auto-forward rule changes through
  the same device-cluster-notification path used for read/flag sync (§5.6), so a newly added
  silent forwarding rule is visible to the owner's other devices, mirroring the KT
  self-monitoring pattern (§3.5) that already protects identity/recovery changes. This is a
  direct consequence of the all-data-classes decentralization invariant (§8.5): a forwarding
  rule is node-local state and MUST replicate to every device like any other data class, so
  none is blind to it. Not currently specified (tracked as §19.10 gap 5 / §17.6 #5).

#### 13. Vacation / auto-responder / out-of-office — **Different (improved)**
- **How:** A client/node-local automation: reply-once-per-sender-per-window with a template.
  Because DMTAP validates the anti-abuse challenge for cold senders *before* decryption
  (§2.7 step 6), an auto-responder can be scoped to fire only for senders that already cleared
  that gate — i.e. real, accountable senders, not arbitrary inbound traffic.
- **Sense-check:** Fits cleanly, and interacts well with §9's anti-abuse design rather than
  fighting it.
- **Security:** Genuinely better than legacy on one specific, real problem: **auto-responder
  backscatter**. Legacy auto-responders regularly bounce vacation replies to forged `From:`
  addresses used by spammers, spamming innocent third parties. Because DMTAP senders are
  cryptographically authenticated (`Payload.from`, §2.4) rather than a spoofable SMTP
  `MAIL FROM`, an auto-responder always replies to a real party — the backscatter failure mode
  structurally cannot occur.
- **Open issue:** None.

#### 14. Read receipts — **Clean**
- **How:** `kind=0x07 receipt`, explicitly **opt-in**, ephemeral, off by default (§2.3, §5.4,
  §6.2).
- **Sense-check:** Correctly modeled as metadata-sensitive rather than a free default — read
  receipts leak exactly the kind of fine-grained timing/behavior signal the rest of the spec
  works hard to hide (the intersection / statistical-disclosure concern of §6.4 (residual
  exposure 3) and §11.3 applies at the small scale of "did they read it and when" too), so
  opt-in-only is the right call, not an oversight.
- **Security:** Better — legacy read receipts (or worse, silent tracking pixels used as a
  read-receipt substitute) are usually either on-by-default or invisible/non-consensual; here
  the feature only exists when the recipient affirmatively enables it, and it rides inside
  the encrypted payload (not observable by any intermediary either way).
- **Open issue:** None.

#### 15. Scheduled send — **Clean**
- **How:** The originating node holds the MOTE undispatched until the scheduled time, then
  sends normally. Best done from the owner's always-on node (§14.4) so the schedule survives
  a sleeping phone/laptop.
- **Sense-check:** Trivial client/node feature; no protocol change needed.
- **Security:** Same as legacy for the "will it actually fire" guarantee (both depend on
  *something* being up at the scheduled time); better for confidentiality (the pending message
  sits encrypted on the user's own node rather than in a provider's plaintext outbox).
- **Open issue:** A scheduled send fired from a thin client with no always-on node (§14.3's
  mobile-only user) needs the hosted relay-mailbox or the phone itself to be awake at send
  time — same durability caveat as any mobile-only delivery in this model, not specific to
  scheduling.

#### 16. Snooze — **Clean**
- **How:** Pure local/CRDT UI state (hide until time X, then resurface) — no wire format
  involvement at all, same as legacy.
- **Sense-check:** Non-issue.
- **Security:** Unchanged.
- **Open issue:** None.

#### 17. Undo send — **Different**
- **How:** Only meaningful during the **pre-dispatch window** the client itself imposes (hold
  the MOTE N seconds before actually handing it to the outbound queue) — identical in kind to
  how Gmail's "undo send" actually works (a client-side delay, not a true retraction). Once
  the MOTE has left the node, "undo" degrades to the same mechanism as message recall (item
  28): a cooperative `redact`/edit request.
- **Sense-check:** Correct and honestly scoped — true after-the-fact undo (retracting bytes
  already delivered to and possibly displayed by the recipient) is not achievable in *any*
  system, legacy or DMTAP; the useful part of "undo send" (a grace-period buffer) transfers
  perfectly.
- **Security:** Same as legacy in the pre-dispatch window; same *honest limit* as legacy
  post-dispatch (neither system can un-deliver bytes already in a recipient's hands) — DMTAP
  is more upfront about this limit (§6.6.8) than most legacy clients' UI copy is.
- **Open issue:** None — the limit is disclosed, not hidden.

#### 18. Mark read/unread, flag/star — **Clean**
- **How:** Explicitly named in §5.6: "the mailbox, read/flag state, labels, and file index are
  replicated across devices as an encrypted CRDT."
- **Sense-check:** Direct match; this is exactly the multi-device-state problem MLS-based
  device clustering is designed to solve cleanly (no per-pair ratchet mess, §5.1).
- **Security:** Better — flag/star state syncs end-to-end encrypted across your own devices
  rather than living as plaintext provider-side account metadata.
- **Open issue:** None.

#### 19. Archive — **Clean**
- **How:** Same CRDT-replicated mailbox-state mechanism as folders/labels (§5.6); "archive" is
  a mailbox-membership change like any other.
- **Sense-check:** Non-issue.
- **Security:** Same as folders (item 1).
- **Open issue:** None.

#### 20. Spam/junk + blocklists/allowlists — **Different**
- **How:** Recipient-local `Policy{allow, challenge, block, rate}` (§9.2), enforced *before*
  decryption for unknown senders (§2.7 step 6), with a graded outcome (silent-discard vs.
  requests-area defer, §2.7a) rather than a binary accept/reject.
- **Sense-check:** This is the correct re-architecture for a sealed-sender, no-central-scanner
  system: since no third party ever sees plaintext to run ML classification against, the
  defense has to move to a place that doesn't need content — cryptographic cost-to-reach
  (challenge/PoW/postage/vouch, §9.3–9.7) substitutes for content filtering at the point where
  content filtering is architecturally unavailable.
- **Security:** Mixed, disclosed honestly. **Better:** spam that never gets a valid challenge
  is rejected before any provider (or anyone) reads a single byte of it — a strictly stronger
  privacy property than "our ML model read your mail to decide it's spam." **Harder:** there
  is no shared cross-user corpus or network effect the way a single large provider's spam
  model benefits from billions of samples — each recipient's local policy starts cold and
  reputation is scored per-issuer (§9.3.1) rather than per-message-content. A node MAY still
  run local content-based classification on whatever lands in its own requests-area (it holds
  the plaintext of its own mail), but there is no cross-user training signal.
- **Open issue:** No shared abuse-signal exchange between independent nodes/operators is
  specified beyond issuer/gateway reputation scoring (§7.5, §9.3.1) — an acceptable, disclosed
  gap given the metadata-privacy goals, not an oversight.

#### 21. Quotas — **Clean**
- **How:** The protocol itself imposes **no** storage cap (§5.5: "bounded only by the owner's
  storage"). Quotas are a policy/billing layer, entirely outside the protocol, imposed only by
  a *hosted* operator on operations (storage caps, send caps, §12).
- **Sense-check:** Correctly kept out of the wire protocol; quotas are a business-model
  concern, not a messaging concern, and self-hosters have none by construction.
- **Security:** Explicitly protected: §12 states there MUST be no quota/plan gate capable of
  disabling encryption — i.e. a hosted operator cannot use quota enforcement as a backdoor
  into weakening the security model.
- **Open issue:** None.

#### 22. Import/export (mbox/EML) — **Clean**
- **How:** The node's IMAP compatibility surface (§8.2) lets any existing mbox/EML tool work
  against it unchanged, exactly as against a legacy IMAP server; native export is a MOTE-store
  dump via JMAP.
- **Sense-check:** Direct hit — this is precisely what the compatibility surfaces (§8.1, §8.2)
  exist for.
- **Security:** Same as legacy for the exported artifact itself (an mbox/EML file is plaintext
  once exported, same as it always was); better in that the export is pulled by an
  app-password-authenticated client from your own node, not handed to a third party to
  generate on your behalf.
- **Open issue:** None.

#### 23. Delegation / send-on-behalf — **Different**
- **How:** Two real mechanisms, neither identical to legacy "delegate access": (a)
  `DeviceCert.caps` (§1.2) includes a `send` capability — an assistant's device can be issued a
  narrowly-scoped device certificate signed by `IK`, letting it send *as the identity* without
  holding the root key; (b) capability delegation (§13.5, UCAN-style) generalizes further —
  "let this app/person act on my behalf" for a specific, time-bound, attenuable right, which
  extends naturally from login-delegation to mail-send-delegation.
- **Sense-check:** Sound mechanically, but note a real semantic gap from legacy Exchange-style
  delegation: a `send`-capable device signs as the *same identity* indistinguishable from the
  principal — legacy delegate-send shows a distinguishing `Sender:`/"on behalf of" header so
  the recipient knows an assistant actually hit send. DMTAP has no equivalent marker by
  default.
- **Security:** The delegated device is narrowly scoped (only `send`, not `admin`/recovery,
  §1.2) and revocable without rotating the whole identity — tighter blast-radius control than
  legacy delegate access, which often just grants full mailbox access. The one regression:
  the recipient cannot tell a delegate sent it, where legacy could show that distinction.
- **Open issue:** **Gap.** No standard `Headers.ext` field is defined for "sent by delegate
  device X of identity Y" — straightforward to add as an extension header (§10) but not yet
  specified.

#### 24. Shared mailboxes — **Clean**
- **How:** A "team inbox" is explicitly one of the named group models (§5.8.1): a group
  identity with its own address; every member of the MLS group receives posts to the shared
  address.
- **Sense-check:** Direct hit — "a group with an address" (§5.8) is exactly the shared-mailbox
  primitive, generalized.
- **Security:** Better — membership, roles (owner/admin/member/poster/reader, §5.8.2), and
  every add/remove/role change are signed and appear in an auditable hash-chained log
  (§5.8.2), unlike a legacy shared mailbox whose access-grant history is usually just an
  admin-console audit log controlled by the provider.
- **Open issue:** None.

#### 25. Distribution lists — **Clean**
- **How:** The broadcast/list posting model of groups-as-address (§5.8.1): post to the address,
  every member gets a sealed per-member copy; membership is typically hidden (§5.8.3).
- **Sense-check:** This *is* the feature §5.8 was designed to unify — direct, first-class
  mapping, not a stretch.
- **Security:** Better on the specific point legacy mailing lists get wrong by default: hidden
  membership via per-member sealed delivery (§5.8.3) means subscribers don't learn each
  other's identities, whereas legacy list software (majordomo/mailman-era) frequently exposes
  the subscriber list or at least each poster's address to all recipients.
- **Open issue:** Large lists trade cryptographic group-sharing for per-member sealed fan-out
  (§5.8.4) — a stated, deliberate scalability/privacy tradeoff, not an unresolved question.

#### 26. Catch-all — **Clean**
- **How:** Explicitly named as a tier-C domain-owner option (§3.9.4).
- **Sense-check:** Direct hit, no tension — it's just another alias-resolution rule at the
  naming layer.
- **Security:** Same considerations as legacy catch-all (a broader attack surface for
  misdirected/spam mail landing in one inbox); DMTAP's per-alias `Identity` signing (§3.9.4)
  at least makes the set of valid catch-all-eligible names auditable via KT, which legacy
  catch-all configuration is not.
- **Open issue:** None.

#### 27. Priority / importance flag — **N/A** (minor)
- **How:** No dedicated field in v0. Could be carried in `Headers.ext` (§2.4's extension-header
  mechanism, formalized in §10) exactly like any other MIME-era header that isn't
  security-relevant.
- **Sense-check:** Not a decentralization-relevant feature at all — it's cosmetic metadata that
  legacy clients themselves rarely enforce meaningfully (X-Priority is famously ignored/abused
  by spammers to fake urgency).
- **Security:** No change either way; note that letting a *sender* self-declare "urgent" is
  already a weak signal in legacy mail and remains exactly as weak here — not something DMTAP
  needs to strengthen, since the real anti-abuse gate (§9) is what actually matters for
  triage.
- **Open issue:** Minor — a client convention for `ext.priority` is undefined but trivial to
  add; low value, not worth a normative field in v0.

#### 28. Message recall (and why it's only cooperative) — **Harder** (honestly, same as legacy)
- **How:** `kind=0x03 edit` / `kind=0x04 redact` reference a prior MOTE by `id` (§2.3) and ask
  the recipient's client to supersede/delete it.
- **Sense-check:** The honest answer is that recall was **never really possible** in legacy
  email either (Exchange's "recall" famously fails silently once the recipient has opened the
  message) — DMTAP just refuses to pretend otherwise. §6.6 item 8 states it plainly: "`redact`/
  `expires` are unenforceable against a non-compliant recipient that already holds the
  plaintext — they are cooperative hints, not guarantees."
- **Security:** Arguably *more* honest than legacy, which is worse in one specific way: because
  DMTAP is E2E, there is no provider-side copy a "recall" could plausibly reach into even in
  principle (legacy recall's occasional partial success stems from the message often still
  sitting in a shared Exchange store the sender's organization controls) — so DMTAP recall is
  unambiguously cooperative-only, with no false hope of provider-side deletion.
- **Open issue:** None — this is a disclosed, irreducible limit (§6.6.8), not a gap to close.

#### 29. DKIM / SPF / DMARC — **Clean**
- **How:** Entirely the gateway's job (§7): DKIM via delegated selectors so the gateway signs
  `d=<domain>` without ever holding the user's DMTAP key (§7.3); SPF/DMARC records maintained
  once per domain by the gateway operator (Tier B, §3.8) or auto-published by the provider for
  vanity domains (Tier C, §3.8); DMARC alignment follows directly from DKIM delegation.
- **Sense-check:** Correctly scoped as a **legacy-interop** concern only — DMTAP-native
  delivery needs none of this (authenticity comes from the payload signature, §2.4, not
  domain-level email authentication). DKIM/SPF/DMARC only matter on the leg that still speaks
  SMTP.
- **Security:** Same guarantees as legacy for that leg (this is literally legacy DKIM/SPF/
  DMARC, unmodified), with one structural improvement: the gateway never holds the user's
  actual identity key (§7.3) — a compromised or malicious gateway operator can forge outbound
  DKIM-valid legacy mail (same blast radius as any legacy DKIM-holding provider) but can never
  impersonate the user's DMTAP-native identity, which is a strictly separate keyspace.
- **Open issue:** None (this is deliberately unmodified legacy machinery, by design).

---

## 17.2 Contacts

#### 30. Contact cards (vCard/JSContact) — **Clean**
- **How:** Native representation is **JSContact (RFC 9553)** MOTEs, synced via JMAP Contacts
  (§8.1, §8.4); CardDAV (RFC 6352) projects the same store as vCard (RFC 6350) for legacy
  clients (§8.4).
- **Sense-check:** Direct hit; contacts are just another MOTE kind sharing the same node,
  encryption, and sync machinery as mail (§8.4: "not separate central services").
- **Security:** Better — your address book is end-to-end encrypted and synced across your own
  devices; no provider (Google Contacts-style) ever holds a plaintext copy or mines it.
- **Open issue:** None.

#### 31. Contact groups — **Different**
- **How:** Two distinct things get conflated under one legacy name, and DMTAP correctly
  separates them: (a) a *reachable* group with its own address and MLS roster is the
  groups-as-address primitive (§5.8) — full distribution-list semantics; (b) a plain
  *organizational* tag ("Family", "Work") with no address of its own is just local metadata on
  contact MOTEs, needing no group entity at all.
- **Sense-check:** This separation is actually clearer than legacy, which uses "contact group"
  for both an address-able mailing list and a purely-local organizational label under one
  confusing UI concept.
- **Security:** Local-tag groups: no change, purely local. Addressable groups: same
  improvements as distribution lists (item 25) — hidden membership option, auditable
  add/remove.
- **Open issue:** None; flagged only so implementers don't collapse the two into one object.

#### 32. Shared address books — **Different** (extrapolated, not verbatim spec'd)
- **How:** The spec explicitly generalizes the "MLS group over a set of manifests" pattern to
  shared file folders (§5.1, §5.7); a shared contacts collection is the same pattern applied to
  contact MOTEs instead of file manifests — consistent with §8.4's framing that contacts share
  "the same MLS groups (§5) as everything else."
- **Sense-check:** Sound by direct analogy to the spec's own stated pattern, but the spec does
  not literally spell out "shared address book" as its own named construct the way it does for
  shared file folders and team inboxes.
- **Security:** Same benefits as any MLS-group-shared data: encrypted, member-auditable,
  re-keyed on member removal (§6.7's removal/re-key rule applies here too, since a departed
  team member's read access to a shared contact list should not persist any more than to a
  shared file folder).
- **Open issue:** **Minor gap.** Worth an explicit line in a future revision of §8.4 naming
  "shared address book = MLS group over contact MOTEs" the same way §5.5/§5.7 do for files, so
  implementers don't have to infer it.

#### 33. Auto-complete / directory (org-wide GAL equivalent) — **Different** (now first-class)
- **How:** A domain that runs org administration (§3.10) publishes a first-class **`DomainDirectory`
  object (§18.4.7)** — a signed, versioned, KT-logged enumeration of its `name@domain` bindings,
  signed by the domain authority (§3.10.1). Members query it for autocomplete; outsiders resolve a
  single name the ordinary way (§3.3). The global handle directory (§3.9.2) remains available for
  the non-org, flat-namespace case.
- **Sense-check:** A legacy GAL is a provider-hosted, always-populated directory *because* the
  provider already holds everyone's plaintext account data centrally. DMTAP has no such central
  plaintext store, so the directory is an explicitly-published, admin-curated object (§3.10.3)
  rather than a byproduct of central data custody — but it *is* now a named construct with a
  provisioning flow, not a build-it-yourself afterthought.
- **Security:** Better than a legacy GAL: the directory is only a *convenience index* of bindings
  that each independently verify forward via DNS + KT (§3.9.4), so a compromised directory can
  withhold/mislabel names (detectable via KT) but can never forge a `name → key` binding or make
  mail encrypt to a key the member doesn't hold (§3.10.3). Membership can be `public` or
  `members-only` (§3.10.3), a choice a legacy GAL doesn't offer.
- **Open issue:** **Closed** by §3.10.3 (`DomainDirectory`, §18.4.7) and the §3.10 org-admin flow.
  Residual honest difference: unlike a legacy GAL it is not *automatic* — an org must opt to run a
  domain authority and publish the directory (§3.10.1).
  Worth naming this as a needed piece of on-boarding tooling for org/business deployments.

#### 34. Contact photos — **Clean**
- **How:** Rides the vCard `PHOTO` field / JSContact media property, or a small inline
  `Attachment` (§2.5) for larger images.
- **Sense-check:** Non-issue.
- **Security:** Same as any other attachment/inline content — encrypted at rest and in
  transit, unlike legacy contact photos which are often fetched live from a provider's CDN
  (Gravatar-style), leaking a network request per contact-card view.
- **Open issue:** None.

#### 35. Import/export — **Clean**
- **How:** Same CardDAV compatibility surface as item 30 (§8.4) — existing tools (Apple
  Contacts, DAVx⁵) work unchanged.
- **Sense-check:** Direct hit.
- **Security:** Same as mail import/export (item 22): pulled from your own node under
  app-password auth, not generated by a third party.
- **Open issue:** None.

#### 36. Contact key / verification (the DMTAP upgrade) — **Clean, and the headline improvement**
- **How:** Every contact is bound to a real identity key, pinned on first contact (TOFU, §3.4)
  and upgradeable to an out-of-band-verified pin (safety-number/QR comparison, §3.4); the key
  is tracked through key transparency (§3.5) so a silent key swap is detectable, and any
  identity-migration (§1.6) is followed automatically and safely because contacts route by
  key, not name.
- **Sense-check:** This is not something legacy contacts have *any* analog for — a legacy
  contact card is just a display-name/email string with zero cryptographic binding, which is
  precisely why display-name spoofing and lookalike-domain phishing work at all. Adding real
  key identity to the contact model is the single most consequential upgrade in this whole
  audit.
- **Security:** Strictly better, not just "different." A "verified contact" (OOB-checked
  safety number) makes impersonation via display-name or lookalike-address cryptographically
  impossible for that relationship — legacy email has no equivalent ceiling on how good contact
  trust can get.
- **Open issue:** Adoption of OOB verification is, as always with this pattern (Signal/WhatsApp
  safety numbers), a UX problem more than a protocol one — most users will stay at TOFU-pinned
  rather than fully OOB-verified. This is a known, general limit of TOFU (§3.4's honest-limit
  note), not specific to contacts.

---

## 17.3 Calendar

#### 37. Events — **Clean**
- **How:** Native **JSCalendar (RFC 8984)** MOTEs, synced via JMAP Calendars (§8.4); CalDAV
  (RFC 4791) projects the store as iCalendar (RFC 5545) for legacy clients.
- **Sense-check:** Direct hit, same "just another MOTE kind" pattern as mail/contacts (§8.4).
- **Security:** Better — no central CalDAV server ever holds your calendar in plaintext;
  end-to-end encrypted and device-cluster-synced like everything else.
- **Open issue:** None.

#### 38. Recurring events (RRULE) — **Clean**
- **How:** JSCalendar (RFC 8984) has first-class recurrence-rule support carried natively;
  CalDAV/iCalendar projection (§8.4) exposes the legacy RRULE form unchanged for compatible
  clients.
- **Sense-check:** Direct hit; recurrence expansion/exceptions are an ordinary client
  responsibility, unaffected by decentralization.
- **Security:** No change; note RFC 8984 was itself designed specifically to fix long-standing
  `VTIMEZONE`/RRULE ambiguity bugs in iCalendar — an incidental improvement DMTAP inherits by
  choosing JSCalendar as the native form rather than legacy iCalendar.
- **Open issue:** None.

#### 39. Invitations + RSVP + iTIP scheduling (peer-to-peer, no central server) — **Clean** (the cleanest mapping in this document)
- **How:** Explicitly specified (§8.4): "Calendar invitations and scheduling (iTIP-style) ride
  as MOTEs between participants — no central scheduling server; free/busy and RSVP are
  messages, not a server query."
- **Sense-check:** This is arguably not even a stretch — legacy iTIP (RFC 5546, `METHOD:
  REQUEST`/`REPLY`/`CANCEL`) was *already* conceptually a message-passing protocol riding over
  email; DMTAP simply carries the same request/reply pattern over MOTEs instead of MIME email.
  The architecture and the legacy feature were already aligned before DMTAP existed.
- **Security:** Better — invitation/RSVP content is end-to-end encrypted and sender-
  authenticated (payload signature, §2.4), whereas legacy iTIP-over-SMTP invitations are
  exactly as spoofable as any other unauthenticated SMTP mail (a well-known calendar-phishing
  vector: forged meeting invites).
- **Open issue:** None.

#### 40. Free/busy — **Different**
- **How:** Also explicit in §8.4: free/busy is "a message, not a server query" — a live
  request MOTE to the participant, answered by a reply MOTE, gated by the same recipient
  policy (§9) as any other cold contact.
- **Sense-check:** Correct given the no-central-server model, but genuinely changes the
  *availability characteristic* of the feature: legacy free/busy is typically a
  continuously-queryable published resource (a `.ifb` URL, or a CalDAV/EWS free-busy report)
  answerable at any time regardless of whether the person is "present." A message-based
  free/busy needs the recipient's node reachable (or queued for wake, §14.3) to answer, so it
  is not instantaneously queryable during their downtime the way a static published resource
  is.
- **Security:** Better in the common case — a stranger cannot silently scrape your calendar
  availability the way a public `.ifb` URL can be scraped; a free/busy request is gated by the
  same anti-abuse policy (§9) as a message from that sender. Harder for the legitimate
  "public booking page" use case (Calendly-style): there is no first-class, always-answerable
  public-availability resource defined.
- **Open issue:** **Gap.** No spec'd analog for a *publicly queryable* availability resource
  (vs. person-to-person messaged free/busy). Achievable as a broadcast-group address (§5.8)
  with an automated responder (same pattern as resource booking, item 47) but not written up
  as its own construct.

#### 41. Reminders/alarms — **Clean**
- **How:** JSCalendar's alarm component, fired locally by whichever device is awake, using the
  same CRDT sync (§5.6) that already carries read/flag/label state so any device can pick up
  the reminder.
- **Sense-check:** Direct hit; identical mechanism to mail's read-state sync.
- **Security:** Same as items 1/18 — encrypted, device-cluster-synced, never provider-visible.
- **Open issue:** None.

#### 42. Shared calendars — **Different** (same extrapolation as item 32)
- **How:** MLS group over calendar-event MOTEs, the same shared-folder pattern §5.1/§5.7
  establish for files; §8.4 confirms calendar shares "via the same MLS groups (§5) as
  everything else."
- **Sense-check:** Sound by direct analogy; not verbatim named as its own construct any more
  than shared address books are.
- **Security:** Same member-auditable, re-keyed-on-removal properties as any shared-folder
  MLS group (§6.7).
- **Open issue:** Same minor documentation gap as item 32 — worth an explicit naming pass.

#### 43. Calendar delegation (assistant manages a calendar) — **Different**
- **How:** Same two mechanisms as mail delegation (item 23): a narrowly-scoped `DeviceCert`
  capability, or a UCAN-style capability delegation (§13.5) for "create/modify calendar MOTEs
  for identity X, time-bound."
- **Sense-check:** Sound; reuses the general delegation primitive rather than inventing a
  calendar-specific one — consistent with the spec's stated preference for one mechanism over
  many.
- **Security:** Same tradeoff as item 23: tighter scoping than legacy (a calendar-only
  capability, not full mailbox access) but no visible "modified by delegate" marker by default.
- **Open issue:** Same as item 23 — no `Headers.ext`/event-property convention yet defined for
  attributing a delegate's edit.

#### 44. Availability (working-hours preference, distinct from free/busy) — **N/A**
- **How:** Purely a local client preference (what hours you're "available" for booking
  purposes); never needs to leave the device.
- **Sense-check:** Not a protocol-relevant feature at all.
- **Security:** No change.
- **Open issue:** None.

#### 45. Time zones — **Clean** (mild improvement)
- **How:** JSCalendar (RFC 8984) carries IANA-timezone-aware date/time fields natively; CalDAV
  projection maps to legacy `VTIMEZONE`/iCalendar for compatible clients (§8.4).
- **Sense-check:** Non-issue architecturally.
- **Security:** N/A (not a security property); functionally an improvement, since RFC 8984 was
  specifically designed to avoid the recurring `VTIMEZONE`-vs-`RRULE` interaction bugs that
  plague legacy iCalendar across DST transitions and floating times.
- **Open issue:** None.

#### 46. Attachments on events — **Clean**
- **How:** Same `Attachment`/manifest mechanism as mail (§2.5), reused unchanged.
- **Sense-check:** Direct hit; no calendar-specific handling needed.
- **Security:** Better than legacy in the same way mail attachments are (item 7): no
  provider-imposed size ceiling, content-addressed integrity.
- **Open issue:** None.

#### 47. Resource booking (rooms, equipment) — **Clean**
- **How:** A bookable resource is simply **another DMTAP identity** — a keypair with its own
  address (the same "group/identity with an address" pattern as §5.8) whose automation policy
  auto-accepts or conflict-checks incoming invitation MOTEs (item 39) on its own. No special
  protocol primitive is needed: it's an ordinary identity running automation, exactly like a
  "bot" node.
- **Sense-check:** This is a clean, arguably elegant emergent property: legacy resource
  booking requires a special mailbox type and server-side logic maintained by Exchange/Google
  Workspace; here it falls out for free from "identities can run automation over ordinary
  messages," needing zero new machinery.
- **Security:** Better in one respect — the resource's booking policy is enforced by its own
  node under its own key, not by a shared central admin console; auditable the same way any
  identity's activity is (signed messages, KT).
- **Open issue:** **Minor gap.** No reference automation-policy language for such "bot
  identities" (auto-accept rules, conflict resolution) is specified anywhere in the document —
  purely an implementation/client concern, but worth flagging as unwritten territory for a
  future client-behavior appendix.

---

## 17.4 Cross-cutting

#### 48. Multi-device sync — **Clean**
- **How:** The owner's devices form a personal MLS group/cluster (§5.6); mailbox, flags,
  labels, and file index replicate as an encrypted CRDT, converging under out-of-order
  delivery. This is the one genuinely new component (§5.7) that unlocks mail, chat, and files
  together.
- **Sense-check:** This is DMTAP's strongest area — MLS's group-membership model is a better
  fit for "N devices, one identity" than legacy IMAP's per-device independent connections (no
  shared crypto identity across devices) or than pairwise Double-Ratchet fan-out (§5.2 notes
  this explicitly as the reason MLS was chosen).
- **Security:** Better — sync is end-to-end encrypted between the owner's own devices, not
  just between the owner and a central server; legacy IMAP sync trusts the provider to relay
  state between devices in whatever form the provider stores it.
- **Open issue:** Endpoint compromise has cluster-wide blast radius (§6.6.3) — a stolen
  synced device exposes the whole replicated mailbox history, not just a device-local slice.
  Mitigated by optional scoped sync (recent-N-days on mobile) but not eliminated; disclosed
  explicitly in §6.6, not a hidden gap.

#### 49. Offline access — **Clean**
- **How:** The always-on node holds the full plaintext store and is the offline-first
  authority (§14.4); thin clients (§14.1) cache what they've synced, same as any modern IMAP
  client with local cache.
- **Sense-check:** Sound; offline access on the always-on node is unconditional (it's the
  source of truth), and thin-client offline access is bounded by whatever's been cached — the
  same shape modern mail apps already have (a phone's Gmail app also only shows what it's
  cached).
- **Security:** Same or better — the cache on a thin client is itself part of the encrypted-
  at-rest model (§6.7), not a plaintext local SQLite file the way many legacy mail apps cache
  today.
- **Open issue:** None beyond the disclosed cluster-blast-radius point already covered in
  item 48.

#### 50. Push/notifications — **Clean, and a genuine privacy improvement**
- **How:** Wake-and-fetch (§14.3): push carries no content, only a wake signal (≤4 KiB,
  content-free, §16.6), through APNs/FCM via a notification proxy; the device wakes, opens its
  own authenticated connection, and drains/decrypts locally. "Push is a latency optimization,
  not delivery" — the client MUST still reconcile on foreground.
- **Sense-check:** Correctly modeled as the *only* architecture that works for a no-always-on-
  box user (§14.3), matching deployed precedent (Delta Chat/Chatmail, Signal, Matrix/Sygnal).
- **Security:** A real, stated improvement: Apple/Google's push infrastructure never sees
  plaintext or even a content preview — unlike many legacy mail providers whose push payloads
  have historically included message preview text delivered through Apple/Google's own push
  infrastructure (a real, historically-exploited leak surface). Content-free wake push
  structurally cannot leak content through the push channel.
- **Open issue:** iOS silent-push throttling requires wake coalescing/batching (§14.3) — a
  known platform constraint, not a DMTAP-specific gap.

#### 51. Account migration — **Clean, and better than legacy**
- **How:** `MoveRecord` (§1.6) rebinds the human name while preserving the key, distributed via
  mesh/DHT, the transparency log, and a signed push-to-contacts MOTE; because contacts route
  by key (not name) after first contact, they follow the move automatically without being
  redirectable by a forged move.
- **Sense-check:** Correctly frames losing a domain as a change of *name*, not of *identity* —
  the reverse of legacy, where losing your provider or domain *is* losing your identity
  (new address, contacts must be manually re-notified, history often orphaned).
- **Security:** Strictly better: the move is signed and KT-audited (a squatter later
  registering the abandoned domain cannot hijack existing relationships, §1.6), and existing
  contacts survive the migration with zero action — legacy migration has no equivalent
  continuity guarantee at all.
- **Open issue:** Only *new* contacts who know solely the abandoned name are unreachable post-
  move — an unavoidable, disclosed tradeoff (§1.6), not a design flaw.

#### 52. Backup/restore — **Different**
- **How:** Two separate things, and the spec is strong on one and silent on the other. (a)
  **Identity backup** — the recovery policy (§1.4: phrase/device/social-guardian methods,
  threshold-gated, VSS/FROST-hardened) is genuinely best-in-class: the key itself is
  effectively un-loseable given redundant recovery factors. (b) **Mailbox-content backup/DR**
  — i.e., "restore my actual mail/calendar/contacts data after data loss," not just "recover my
  key" — is not separately specified as a DMTAP mechanism. It's implicitly provided by (i)
  multi-device replication (§5.6, each device holds/caches a copy, an incidental N-way backup)
  and (ii) whatever the node operator does at the infra layer — self-hosters own this
  entirely; a hosted hosted-operator (§14.6) presumably backs the per-tenant object-storage
  bucket, but this is an operator/infra choice, not a protocol guarantee.
- **Sense-check:** The identity half is exactly the right thing for a decentralized protocol
  to specify rigorously (§1.4 does). The *content*-backup half is legitimately a node/infra
  concern more than a protocol concern in a self-sovereign model (there is no central server
  whose backup policy the protocol could mandate) — but that also means there is currently no
  documented "export a full encrypted backup archive of everything" primitive the way, e.g.,
  Signal ships a backup file, which a security-conscious self-hoster would reasonably expect.
- **Security:** Identity recovery: strictly better than legacy ("forgot password" flows,
  provider-controlled account recovery). Content backup/DR: no worse than legacy in the
  self-hosted case (you were always responsible for backing up your own server), but legacy
  *hosted* webmail has an implicit "the provider backs up their datacenters" guarantee that
  has no explicit DMTAP-protocol equivalent — it's an operator SLA question, not specified
  here.
- **Open issue:** **Gap.** Recommend a future revision define a client-level "export full
  encrypted backup archive" primitive (mailbox + calendar + contacts + keys-if-desired) at the
  client-access layer (§8), independent of any specific operator's infra practices, so
  self-hosters have a documented, portable disaster-recovery path that doesn't depend on
  reverse-engineering the IMAP/CalDAV/CardDAV export surfaces for a full-fidelity backup.

---

## 17.5 Summary table

| # | Feature | Verdict | One-line note |
|---|---------|:-------:|----------------|
| 1 | Folders | Clean | JMAP mailboxes over CRDT-synced state (§5.6) |
| 2 | Labels/tags | Clean | Same CRDT state as folders (§5.6) |
| 3 | Filters/rules | Different | Run on the owner's own always-on node, not a third party (§8.2) |
| 4 | Search | Different | On-device by design (§0.7); no cross-user index |
| 5 | Threading | Clean | `Headers.thread` + `refs` (§2.4) |
| 6 | Drafts | Clean | Pre-dispatch CRDT state (§5.6) |
| 7 | Attachments (any size) | Clean | Content-addressed manifests, 3 size tiers (§2.5, §5.5, §6.5) |
| 8 | Rich text/HTML | Clean | `mime`/`Body` fields (§2.4) |
| 9 | Signatures (text) | Clean | Client-local, unrelated to crypto `sig` |
| 10 | Aliases + plus-addressing | Clean | `Identity.names` list (§3.9.4); non-spoofable |
| 11 | Forwarding (manual) | Different | Works; forged-`From:` redirect correctly impossible |
| 12 | Auto-forward | Different | Works; rule-change auditing not yet specified (gap) |
| 13 | Vacation/auto-responder | Different | Improved: no backscatter (authenticated senders) |
| 14 | Read receipts | Clean | `kind=0x07`, opt-in, off by default (§2.3, §6) |
| 15 | Scheduled send | Clean | Node holds MOTE until send time |
| 16 | Snooze | Clean | Local UI state only |
| 17 | Undo send | Different | Only within pre-dispatch window, same as legacy in truth |
| 18 | Mark read/unread, flag/star | Clean | Explicitly named in §5.6 |
| 19 | Archive | Clean | Mailbox-membership change (§5.6) |
| 20 | Spam/junk + block/allow lists | Different | Cost-based (§9) replaces central ML; no shared corpus |
| 21 | Quotas | Clean | Not a protocol concept; operator policy only (§12) |
| 22 | Import/export (mbox/EML) | Clean | Via IMAP compat surface (§8.2) |
| 23 | Delegation/send-on-behalf | Different | `DeviceCert.caps`/UCAN (§1.2, §13.5); no "on behalf of" marker yet |
| 24 | Shared mailboxes | Clean | Team-inbox group model (§5.8.1) |
| 25 | Distribution lists | Clean | Groups-as-address, hidden membership (§5.8) |
| 26 | Catch-all | Clean | Named tier-C option (§3.9.4) |
| 27 | Priority/importance | N/A | Cosmetic; undefined ext header, low value |
| 28 | Message recall | Harder | Cooperative-only, disclosed (§6.6.8); legacy recall never really worked either |
| 29 | DKIM/SPF/DMARC | Clean | Gateway-only, delegated selectors (§7, §3.8) |
| 30 | Contact cards | Clean | JSContact/CardDAV (§8.4) |
| 31 | Contact groups | Different | Split cleanly into addressable groups vs. local tags |
| 32 | Shared address books | Different | Extrapolated from shared-folder pattern (§5.1/§5.7); not verbatim named |
| 33 | Auto-complete/directory | Different | First-class `DomainDirectory`/GAL object (§3.10.3, §18.4.7); admin-curated, KT-logged, forward-verified |
| 34 | Contact photos | Clean | vCard field / inline attachment |
| 35 | Import/export | Clean | CardDAV compat surface (§8.4) |
| 36 | Contact key/verification | Clean | **The headline upgrade** — real key binding + KT + OOB verify (§3.4–3.5) |
| 37 | Events | Clean | JSCalendar/CalDAV (§8.4) |
| 38 | Recurring events (RRULE) | Clean | Native in JSCalendar; fixes legacy TZ/RRULE bugs |
| 39 | Invitations+RSVP+iTIP | Clean | Explicit, cleanest mapping in the audit (§8.4) |
| 40 | Free/busy | Different | Message/reply, not an always-queryable resource; no public-availability page yet |
| 41 | Reminders/alarms | Clean | Same CRDT sync as read/flag state |
| 42 | Shared calendars | Different | Extrapolated shared-folder pattern, same as item 32 |
| 43 | Calendar delegation | Different | Same mechanism/gap as mail delegation (item 23) |
| 44 | Availability (working hours) | N/A | Purely local preference |
| 45 | Time zones | Clean | JSCalendar fixes legacy VTIMEZONE issues |
| 46 | Attachments on events | Clean | Same mechanism as mail attachments |
| 47 | Resource booking | Clean | A resource is just an automated identity; no new primitive needed |
| 48 | Multi-device sync | Clean | Core strength of the design (§5.6, §5.7) |
| 49 | Offline access | Clean | Always-on node is offline-first authority (§14.4) |
| 50 | Push/notifications | Clean | Content-free wake-and-fetch (§14.3); real privacy win |
| 51 | Account migration | Clean | `MoveRecord`, contacts follow by key (§1.6) |
| 52 | Backup/restore | Different | Identity recovery is excellent (§1.4); content backup/DR undocumented (gap) |

Tally: **34 Clean, 17 Different, 2 N/A, 1 Harder** (auto-complete/directory, formerly the one
"Harder" feature with no fallback beyond manual setup, is now graded **Different** — §3.10.3 gives
it a first-class `DomainDirectory` object and §3.10 an org-admin provisioning flow, leaving only
the honest "an org must opt in" difference. The remaining strictly-"Harder" grade is message
recall, whose "Harder" is really "same honest limit as legacy," included as Harder only because it
is a capability legacy nominally advertises and DMTAP explicitly refuses to overclaim).

## 17.6 Gaps to resolve

Ranked by how load-bearing the gap is:

1. **Full-fidelity backup/restore export** (item 52) — no client-level "export everything,
   encrypted, portable" primitive is specified. Identity recovery (§1.4) is solved; mailbox/
   calendar/contacts disaster-recovery for self-hosters is not. Recommend a §8 addition.
2. **Auto-complete/organizational directory** (item 33) — **resolved.** §3.10.3 specifies a
   first-class **`DomainDirectory`** (GAL) object (§18.4.7) — admin-curated, domain-authority-signed,
   KT-logged, and forward-verified per entry (§3.9.4) — and §3.10 gives the org-admin provisioning
   and onboarding flow. The only residual difference from a legacy GAL is that it is opt-in
   (an org must run a domain authority, §3.10.1), not automatic.
3. **Public, always-queryable availability/booking page** (item 40) — free/busy-as-message
   covers person-to-person scheduling well but has no analog for a public booking page
   (Calendly-style); worth a short note on composing it from a broadcast group + automated
   responder (same pattern as resource booking, item 47).
4. **Delegate-send/edit attribution marker** (items 23, 43) — delegated `send`-capable devices
   are indistinguishable from the principal; a `Headers.ext` convention (an `ext-value`-typed
   extension header, §18.3.6) for "sent/edited by delegate device X" would close this small but
   real gap versus legacy's `Sender:`/"on behalf of" distinction. The delegation itself is the
   capability model of **§13.5** (`DeviceCert.caps`/UCAN-style scoping), so the marker should
   name the delegated capability from there; this is the same open item tracked as §19.10 gap 5.
5. **Auto-forward rule-change auditing** (item 12) — silent forwarding-rule injection is a
   live real-world account-takeover technique; recommend surfacing rule changes through the
   same device-cluster-notification path already used for identity/KT self-monitoring (§3.5),
   consistent with the all-data-classes decentralization invariant of **§8.5** (a forwarding
   rule is node-local state that MUST replicate to the owner's other devices the same way mail/
   calendar/contact state does, so no device is blind to it).
6. **Explicit naming of "shared address book" / "shared calendar" as named constructs** (items
   32, 42) — the mechanism (MLS group over a data-class's MOTEs) is sound and consistent with
   how §5.5/§5.7 name shared file folders, but §8.4 doesn't spell it out the same way for
   contacts/calendar; a documentation gap, not a mechanism gap.
7. **No shared cross-node abuse-signal exchange** (item 20) — a disclosed, accepted tradeoff of
   the privacy model rather than an oversight, but worth tracking as adoption grows and spam
   patterns emerge that per-recipient-only policy handles less well than a shared corpus would.

None of these gaps require new cryptography or a new core primitive — every one is either a
documentation gap (naming an already-consistent pattern), an extension-header convention
(§10), or a client/onboarding feature to be built on existing primitives (groups, delegation,
CRDT sync).

## 17.7 Net assessment

DMTAP achieves **practical feature parity** with classic email/calendar/contacts, and does so
by mapping the overwhelming majority of legacy features onto existing primitives rather than
inventing per-feature machinery: the MOTE kind system (§2.3), the device-cluster CRDT (§5.6),
groups-as-address (§5.8), and the JMAP/IMAP/CalDAV/CardDAV compatibility surfaces (§8) between
them account for 40+ of the 52 features audited here with no new mechanism required.

Of the remainder: a handful of features are **correctly** not preserved in their legacy form
because that form depended on a central plaintext store or a forgeable identity that DMTAP
deliberately removes (server-side spam ML, forged-`From:` redirect, a central searchable
index, message recall as a real guarantee) — these are disclosed, intentional, and in most
cases a net security improvement, not a parity failure. A smaller number are **genuine,
resolvable gaps** (§17.6) — none load-bearing enough to block adoption, all closeable with
documentation, an extension header, or an onboarding pattern rather than new protocol design.

And a few features are not just preserved but **substantively upgraded** by the decentralized
model: verified contact identity (item 36) eliminates a whole class of impersonation/spoofing
attacks legacy contacts have no defense against; account migration (item 51) gives continuity
of address and relationships that legacy providers cannot; content-free push (item 50) closes
a real historical leak surface; auto-responders (item 13) structurally cannot backscatter; and
aliases (item 10) cannot be used to impersonate, unlike legacy plus-addressing.

**Bottom line:** nothing a classic email/calendar/contacts user relies on daily is missing
without an honest, documented reason — either a clean or different-but-equivalent mapping
exists, or the feature's legacy form depended on a central-plaintext-store property that DMTAP
correctly declines to reintroduce. The open items in §17.6 are real but narrow, and closing
them is client/documentation work on top of the existing primitives, not a redesign.
