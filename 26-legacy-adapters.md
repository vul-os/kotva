# 26. Legacy Adapters — Bridging Beyond SMTP

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHOULD**, **SHOULD NOT**,
**RECOMMENDED**, **MAY**, and **OPTIONAL** are to be interpreted as in RFC 2119 / RFC 8174.

## 26.1 Scope & goals

A DMTAP user's correspondents are not only on legacy email. Some are reachable only by SMS, some
only through WhatsApp, some only through a Telegram bot, a Discord guild, or a Slack workspace.
This document specifies how a DMTAP identity reaches such a correspondent over each of those
rails, with the same honesty this specification applies everywhere else: **who can be contacted,
who sees the plaintext, who pays what, and what survives if the bridge goes away** are stated
plainly rather than left to be discovered the hard way.

**Relationship to §7 (normative boundary).** §7 specifies **one** legacy adapter in full: the
SMTP/mail bridge, the **only** legacy protocol the node ships in-tree and the **only** function in
DMTAP requiring a scarce, non-self-provisionable resource (an IP with sending reputation and
unblocked port 25, §7.1a, §0.2.3). This document does **not** redefine, rename, or extend the
**gateway role** of §7/§0.2.3 — that term keeps its single sense (§0's glossary: the legacy-**mail**
adapter role) unchanged. Instead it generalizes the *pattern* §7 already establishes for mail — a
pluggable bridge to a network DMTAP does not control, with an honest accounting of cost, exposure,
and survivability — to every **other** legacy rail a user's correspondents live on. Nothing in §7.1–
§7.15 is altered, weakened, or superseded by this document; every cross-reference below into §7 is
to text that governs unchanged.

**Terminology (normative).** This document introduces **adapter** as the general noun: a pluggable
bridge between one legacy communication rail and DMTAP MOTEs. The SMTP/mail bridge of §7 is one
adapter — the in-tree, mandatory-pattern-setting one. Every adapter this document specifies (SMS,
WhatsApp, Telegram, Discord, Slack, and any future rail registered under §26.11) is **optional and
out-of-tree** (§26.9). Two further terms are reused **by analogy, not by redefinition**:

- **node mode** and **gateway mode** name a deployment property any adapter can have (§26.2) — the
  same self-host/third-party-operator split §12.1 already draws for the whole protocol, and the
  same private/public split §7.15.4 already draws for legacy mail-client access, generalized here
  to apply uniformly across rails. Writing "gateway mode" never means "is the §7 gateway role";
  an adapter running in gateway mode for, say, WhatsApp is not thereby a gateway in the §0.2.3
  sense, has no scarce resource requirement, and belongs to no new operator class (§26.9, §26.10).
- Every other term this document needs — MOTE, `Payload`, sealed sender, `ProvenanceRecord`,
  `GatewayAuthz`, postage — is the §1–§21 term, used unchanged.

**What this document is not.** It does not add a protocol token (none exists and none is added
here, §12.2, §3.12.5(d)); it does not specify advertising, a payment-split mechanism, or a
payout-fairness scheme (§26.10); and it does not weaken any disclosure this specification makes
elsewhere. Where a platform's own terms (pricing, template rules, API shape) are described below,
they are described **as currently understood** and flagged where verification against the
platform's current terms is the reader's responsibility, not a fact this specification asserts
(§26.4.2, §26.12).

## 26.2 Two deployment modes, one adapter (normative)

Every adapter — in-tree or out-of-tree, SMTP or otherwise — runs in exactly one of two modes, and
the mode is a **property of the deployment**, never of the adapter's code:

| | Node mode | Gateway mode |
|---|---|---|
| Credentials | the user's own (their WABA, their bot token, their modem, their own aggregator account) | the operator's own, serving identities that are not the operator |
| Identities served | exactly one | many |
| Billing | none — nothing to charge, nobody positioned to charge it (§12.3) | MAY be metered where the underlying rail has genuine marginal cost (§26.10) |
| Authorization layer | none — there is only one identity, so there is nothing to authorize between | REQUIRED (§26.2.1) |

**The same adapter code runs both modes.** An implementation MUST NOT require a separate build,
fork, or reimplementation to move an adapter from node mode to gateway mode or back; the two modes
differ only in configuration and in the four additions below. This is the same requirement §7.1b
already states for the mail gateway (one binary, mode selected at deployment) generalized to every
adapter.

**Precedent, not invention.** This two-mode split is not new to DMTAP: §12.1 already draws it for
the whole protocol (self-host vs. someone-else-hosts), and §7.15.4 already draws it specifically
for legacy mail-client access (private vs. public/registered-clients-only). §26 states the general
form once so every rail inherits the same disclosure floor mail already has, instead of each
out-of-tree adapter reinventing — or, worse, omitting — it.

### 26.2.1 What gateway mode adds — exactly four things (normative)

Node mode needs no authorization layer because there is only one identity and no one to
distinguish it from. The moment an adapter serves more than one identity, four things become
necessary, and are the **entire** list — gateway mode MUST provide all four and MUST NOT be
represented as providing less:

1. **Authorization scope** — which identity may send as what. This generalizes §7.11.2 step 2 (the
   per-address claim check that closed GW-7's open-relay gap for mail) to any rail: an operator
   running an adapter in gateway mode MUST know, for every outbound message it relays, which of its
   served identities is authorized to present as which remote-facing number/handle/account, and
   MUST refuse an unauthorized claim exactly as §7.11.2 requires for mail. The mechanism is
   `GatewayAuthz` (§12.2, §18.8a.3): a per-rail grant is a `CapabilityToken` (§18.7.3) with
   `Capability.resource = "gw-rail:"+rail+":"+remote_id` and `Capability.ability = "send-as"`,
   referenced from the served identity's `GatewayAuthz.grants` — the same construction §7.11.2
   step 2's per-address grant uses for mail, generalized by the resource string alone. Refusing an
   unauthorized claim fails closed with `ERR_ADAPTER_CREDENTIAL_UNAUTHORIZED` (`0x0B03`, §21.11a).
2. **A signed published tariff** (§26.10) — what this adapter charges, if anything, signed by the
   operator's own key, so a client can compare operators before routing through one.
3. **Signed usage receipts** (§26.10) — so a claimed charge is auditable by the paying user, not
   merely asserted, generalizing the audit model §7.9 already gives mail-gateway usage to every
   metered rail.
4. **A content-visibility disclosure** — which parties see plaintext on this rail, under this
   deployment, disclosed to the user before they rely on it. This generalizes §7.15.3/§7.15.4's
   honest-privacy consequence (which today covers only legacy mail-client access) to every adapter:
   a gateway-mode operator MUST disclose the same class of fact — "here is who can read what you
   send through me" — for whichever rail it bridges.

A deployment that serves more than one identity through an adapter and provides fewer than these
four is non-conformant with this document, regardless of what rail it bridges.

## 26.3 The four fields every adapter declares (normative)

A conformant adapter — in its own documentation if node-mode-only, and additionally in a published
`AdapterDescriptor` (§26.3.1) if it ever runs in gateway mode — MUST declare four fields. These are
what let two adapters, or two operators of the same adapter, be compared **at a glance**; a bare
"platforms supported" list hides all four, which is exactly the comparison a user routing a
contact needs and cannot get from a feature checkbox.

1. **Can it initiate?** The load-bearing field, because it determines whether the adapter can do
   the one thing a legacy bridge exists to do — reach a stranger cold:
   - **freely-initiating** — the adapter can contact a party who has never interacted with this
     identity before, given only an address or number (email, SMS).
   - **inbound-triggered** — the other party MUST start the conversation before this identity may
     send anything at all, or may send only a constrained class of message (WhatsApp, Telegram,
     Discord, Slack) — template restrictions, bots that cannot DM first, shared-guild and
     workspace-install requirements are all instances of this same field, not separate platform
     quirks (§26.4).
2. **Inbound transport class** — how this adapter receives, which determines whether it can run
   behind CGNAT with no public endpoint at all:
   - **hardware-local** — inbound arrives on physical hardware the adapter owns (an attached SMS
     modem); needs no network reachability whatsoever.
   - **outbound-persistent** — the adapter holds an outbound connection open (Telegram long-polling,
     Discord's WebSocket Gateway, Slack Socket Mode) and receives over it; needs no inbound
     reachability, works behind CGNAT.
   - **webhook** — the platform calls the adapter back over HTTPS (WhatsApp Cloud API, most SMS
     aggregators); needs a reachable HTTPS endpoint, exactly the reachability problem §4.3/§7.15.2
     solve for the mesh and the mail gateway respectively.
   - **listener** — the adapter runs its own server the outside world dials (SMTP on port 25);
     needs a public address and, for SMTP specifically, the §7.1a scarce-resource floor.
3. **Price shape** — **metered** (real marginal cost passed through), **flat** (a fixed
   subscription-style fee, if any, unrelated to per-message volume), or **free**.
4. **Exposure** — who sees plaintext on this leg, under this deployment (§26.5, §26.6). Stated per
   rail **and** per mode: node mode and gateway mode do not have the same exposure set, and for a
   platform-mediated rail neither mode reaches "nobody" (§26.5.1).

**Properties are per-direction, not per-platform (normative).** A single platform can have
different answers to field 1 and field 3 depending on which way the message is flowing — WhatsApp
is the clearest case (§26.4.1) — so a conformant declaration MUST be stated per direction where the
rail is asymmetric. Declaring one row for "WhatsApp" when inbound and outbound have different
initiation classes and different price shapes understates what a user is actually agreeing to.

### 26.3.1 The `AdapterDescriptor` (self-signed, discovery-only)

A gateway-mode adapter operator MAY publish a self-signed **`AdapterDescriptor`** under its own
identity, carrying: `{ adapter_ik, rail, mode, initiation_class, inbound_transport_class,
price_shape, exposure, credential_model (§26.8, where applicable), tariff_ref (§26.10), region }`.
This is the same object §7.5 already specifies for the mail gateway (`{ gateway_ik, domain, modes,
operator_mode, region, attestation selector }`), generalized field-for-field to any rail, and it
inherits §7.5's rule **unchanged**: it is **discovery-only, self-asserted, and carries no
reputation field, no price ranking, and no stake** — the same three reasons §7.5 gives for
excluding each (an adjudicator to seize a stake, price being operator policy, reputation being
locally measured, never globally published, §7.5, §9.6) apply without modification to every
adapter's descriptor. §7.5's discovery/reputation model — locally measured, swappable, no
global score — governs adapter descriptors exactly as it governs the mail gateway descriptor; this
section adds no exception to it.

**Wire form (§18.8a, formerly informal).** An adapter is a `gateway`-kind coordinator (CONTRACT §5:
"the legacy `adapter`s (§26)... are the first, fully-worked instances" of the contract, alongside
§7). `AdapterDescriptor` is therefore not a bespoke object but the general `CoordinatorDescriptor`
(§18.8a.1) with `kind = "gateway"`, `identity = adapter_ik`, and `rail`/`mode`/`initiation_class`/
`inbound_transport_class`/`price_shape`/`exposure`/`credential_model`/`region` carried in the
opaque `policy` field (key 4) — the same relationship §7.5's own `{ gateway_ik, domain, modes,
operator_mode, region, attestation selector }` bears to it. `tariff_ref` becomes the descriptor's
OPTIONAL `tariff` field (key 5, a `Tariff`, §18.8a.1). Nothing in this subsection's field list or
disclosure rules changes; §18.8a states the byte layout the fields above now have.

Publication transport is intentionally unspecified, exactly as it is for §7.5's gateway descriptor
today: an operator's own domain, a directory, or — since the descriptor is a small, plaintext, self-
signed fact anyone should be able to discover — an ordinary DMTAP-PUB `pub_announce` (§22.3) under
the operator's own author feed (§22.4) is a natural, already-built transport requiring **zero new
wire mechanism**, following the same reuse §24.18 already makes of §22 for artifact metadata. This is
a MAY, not a MUST: an implementation that publishes an `AdapterDescriptor` some other way is not
thereby non-conformant, provided the object's content and the rules above are honored.

## 26.4 Per-rail properties (the four-field table)

| Adapter | Initiation | Inbound transport class | Price shape | Exposure (plaintext parties, this leg) |
|---|---|---|---|---|
| **Email** (SMTP, §7) | Freely-initiating | listener | Free for native use; operator policy for gateway-mode egress (§7.13) | Third-party gateway operator, iff not self-hosted (§7.15.4) — plus the legacy recipient's own mail provider, a fact inherent to email itself and not a DMTAP party |
| **SMS — hardware** (in-tree reference adapter, §26.9) | Freely-initiating | hardware-local | Free (the user's own carrier plan; no adapter fee) | Nobody but the user (own hardware, own SIM) — plus the user's own mobile carrier, inherent to SMS, not a DMTAP party |
| **SMS — aggregator** | Freely-initiating | webhook | Metered — genuine marginal cost, the one place outside SMTP a real per-message cost exists (§26.10) | The aggregator, always — plus a third-party gateway operator if this identity is not the aggregator-account holder — plus the destination carrier, inherent to SMS |
| **WhatsApp — inbound / in-window reply** | Inbound-triggered (the other party must open the conversation) | webhook | Free, within the service window (§26.4.1) | Meta, always (§26.5.1) — plus a third-party gateway operator, iff not self-hosted |
| **WhatsApp — outbound-initiate, outside the window** | Inbound-triggered overall — this leg cannot originate a conversation at all; restricted to pre-approved templates (§26.4.1) | webhook | Metered and template-restricted — a functional wall, not a price (§26.4.1) | Meta, always — plus a third-party gateway operator, iff not self-hosted |
| **Telegram** | Inbound-triggered — a bot cannot message a user first | outbound-persistent | Free | Telegram, always (§26.5.1) — plus a third-party gateway operator, iff not self-hosted |
| **Discord** | Inbound-triggered — a shared guild is required | outbound-persistent | Free | Discord, always — plus a third-party gateway operator, iff not self-hosted |
| **Slack** | Inbound-triggered — a workspace install is required | outbound-persistent | Free | Slack, always — plus a third-party gateway operator, iff not self-hosted |

### 26.4.1 WhatsApp's asymmetry, stated as a wall, not a footnote (normative)

WhatsApp is not one row. Inbound messages, and replies sent inside the platform's service window
after a user-initiated contact, are free and unlimited. Initiating outside that window is
**metered and restricted to pre-approved message templates** — and the template restriction is
the load-bearing fact, not the price: a template-restricted channel **cannot carry arbitrary human
text at any price**, because the platform will not deliver a freeform message through that path
regardless of what the sender is willing to pay. A conformant client or adapter description MUST
present this as a **functional wall** — a class of message this rail cannot carry outside the
window, full stop — and MUST NOT present it merely as "outbound costs more," which understates it
into an ordinary pricing tier a user might reasonably expect to pay past.

**Pricing verification (informative, explicitly unconfirmed).** WhatsApp Business Platform pricing
has changed repeatedly and this document does not assert a 2026 price list; the window duration,
whether conversation-based or per-template pricing is current, and the exact categories chargeable
outside the window are **the reader's responsibility to verify against Meta's current published
terms** at deployment time (§26.12). What this document asserts normatively is the **shape** —
asymmetric initiation, template-gated outbound-cold — which has held across every pricing revision
to date; it does not assert any specific number.

### 26.4.2 Telegram, Discord, Slack cannot initiate at all (normative)

Unlike WhatsApp, these three rails have no outbound-cold path whatsoever: a Telegram bot cannot
message a user who has not started a chat with it; a Discord bot can only reach a user inside a
guild both are members of; a Slack app can only reach a workspace that installed it. There is no
tier, template, or price that unlocks freely-initiating behavior on any of the three — this is a
platform-imposed ceiling on field 1 (§26.3), not an adapter limitation, and a conformant adapter
MUST NOT imply otherwise (e.g. by offering a "premium outreach" feature that does not exist on
these platforms).

## 26.5 Authenticity model per adapter (normative)

**Email is the only rail with cryptographic legacy authentication.** SPF, DKIM, and DMARC, now
carried in the `AuthResults` map inside the signed `GatewayAttestation` (§7.2c) exactly as GW-2
requires, let a recipient verify — not merely trust — that a message's claimed origin domain
authenticated it before it reached the gateway.

**Every other rail is platform-asserted and cryptographically unverifiable.** When an adapter
bridges WhatsApp, Telegram, Discord, or Slack, the only thing it can honestly convey is "the
platform's API told me this arrived from `+27821234567`" (or a Telegram user id, a Discord
snowflake, a Slack user id) — there is no signature the adapter itself can verify independently of
trusting the platform's own backend, unlike DKIM, which any relay can check without trusting
anyone but the signing domain.

### 26.5.1 The `AuthResults` distinction a client MUST render differently (normative)

Without a distinct value for "platform-asserted," a Discord-bridged message and a DKIM-verified
one look identical to the recipient — exactly the failure §7.2c's client-presentation rule exists
to prevent for `legacy_from`, now generalized:

- The `AuthResults` map (§7.2c) MUST carry a structurally distinct entry for a platform-asserted
  claim — informally, `{ platform_asserted: { rail, claim } }` — never a value that overloads or
  resembles the email verdict shape (`spf`/`dkim`/`dmarc`/`arc`, §7.2c, §7.11.1). The two MUST be
  distinguishable by which key is present, not by a client having to parse a string inside a shared
  field. (The exact CDDL for this addition to `GatewayAttestation`/`AuthResults` is a §18.3.11
  matter, reported separately, §26.11 — the requirement that the two shapes be structurally
  distinct is normative here regardless of where the bytes land.)
- **A client MUST NOT render a `platform_asserted` claim with any visual parity to a message
  carrying `dmarc=pass` alignment.** This is the same rule §7.2c already states for `legacy_from`
  absent DMARC alignment, extended to cover every non-cryptographic rail: a platform's assertion
  that a message came from a given number or handle is evidence of what the platform's backend
  says, never of what a signature proves, and a client's UI MUST NOT collapse that distinction for
  the sake of a uniform "verified sender" badge.
- This holds **regardless of adapter mode.** Node mode does not upgrade a platform-asserted claim
  to a cryptographic one; only the rail's own authentication mechanism (which, for these four
  rails, does not exist) could do that.

## 26.6 Sovereignty disclosure (normative)

**In node mode**, the remote party sees the user's **own** number or handle — the user's own WABA
number, the user's own bot, the user's own modem's SIM. This channel is **durable and portable**:
it survives the user leaving DMTAP entirely, because nothing about it depended on DMTAP in the
first place. It is the legacy-rail analogue of §7.10's "the native address is the anchor": nothing
rented, nothing that disappears if this identity stops running an adapter.

**In gateway mode**, the remote party sees the **gateway's** number or handle. Leave that gateway
and the channel **dies** — and, more sharply than mail's alias residual (§7.10.4, where at least
the alias can be burned without harming a third party), **everyone who knows that number now
reaches someone else**: a phone number or bot handle a gateway operator reassigns to a different
tenant does not go silent, it goes to a stranger. This is a materially worse failure mode than
mail's tier-3 alias residual (§7.10.6) and MUST be disclosed with the same bluntness.

**Normative client rule.** A client MUST be able to tell a user which of the two describes a given
conversation — node-mode (theirs, portable, survives departure) or gateway-mode (borrowed, dies on
departure, and hands the next correspondent to whoever the operator assigns the number to next).
This is the direct generalization of §7.10.6's alias-provenance labelling rule ("legacy alias,
issued by `gw.example.net`, not portable") to every rail: a client that lets a user treat a
gateway-mode WhatsApp number as "their number" has recreated the exact lock-in §7.10.6 exists to
prevent for mail, on a rail where the failure mode is worse.

## 26.7 Reply routing is gateway-mode complexity, not adapter complexity (normative)

**In node mode, everything inbound is the user's** — there is one identity behind the adapter, so
there is no routing decision to make; every inbound message on that rail belongs to that identity
by construction.

**In gateway mode**, the operator holds a mapping from **(rail, remote party, which
number/bot/account)** to a DMTAP identity — state that decides which of the operator's served
identities a given inbound message belongs to. This is exactly the same shape as mail's
`GatewayAliasMap` (§7.10.2, §18.3.12: `alias → native`), generalized to a three-part key because a
gateway-mode operator may run several numbers/bots across several platforms for many identities at
once, where mail's alias map needs only a local-part.

This mapping is:

- **state the operator holds** — it MUST exist for gateway mode to route inbound at all, exactly
  as `GatewayAliasMap` must exist for a random-form mail alias (§7.10.2);
- **state the operator can corrupt** — a misrouted entry delivers a stranger's message to the
  wrong identity, or a user's reply to the wrong correspondent, with no cryptographic check
  independent of the operator's own bookkeeping (the platform-asserted-only authenticity of §26.5
  gives the operator nothing stronger to check the mapping against);
- **state the operator can leak** — the mapping is, in aggregate, the operator's own record of
  which of its users talks to which outside parties over which platform, a strictly larger privacy
  exposure than a single alias row.

**This is specified as a gateway-mode concern, not an adapter concern (normative).** The adapter
code that speaks WhatsApp's or Telegram's wire protocol needs no opinion about this mapping in
node mode, where it does not exist. A conformant gateway-mode deployment MUST maintain this
mapping accountably (rebuildable or re-issuable, never treated as more durable or more trustworthy
than it is, mirroring §7.4's "no message store, only rebuildable operational state" rule) and MUST
disclose that it holds it, as part of the content-visibility disclosure of §26.2.1 item 4.

## 26.8 Credentials (normative)

### 26.8.1 WhatsApp: bring-your-own is the default

The default credential model for a WhatsApp adapter is **bring-your-own (BYO)**: the user's own
WhatsApp Business Account (WABA) and the user's own access token. Under BYO, the gateway operator
is never reselling Meta platform access on the user's behalf, and needs **no** Meta Tech Provider
or Business Solution Provider (BSP) licence to run the adapter for that user — the operator is
relaying traffic the user is already, independently, entitled to send.

A **BSP-backed** option — where the operator holds its own Tech Provider/BSP relationship and
onboards users under it rather than requiring each to hold their own WABA — MAY exist. Where it
does, it MUST be **labelled as such**, distinctly from BYO, as part of the `AdapterDescriptor`'s
`credential_model` field (§26.3.1) and the content-visibility disclosure (§26.2.1 item 4): a user
choosing a WhatsApp adapter is entitled to know whether they are relaying through their own
platform relationship or through the operator's.

### 26.8.2 Ruled out explicitly (normative honest-limit)

Two practices are **ruled out**, not merely discouraged, and a conformant implementation MUST NOT
offer either as a WhatsApp credential path:

- **Unofficial WhatsApp libraries** that speak the consumer app's protocol rather than the Business
  Platform API. These are terms-violating and unreliable by construction — they run against a
  protocol Meta does not publish and can break, detect, and ban against at will, with no recourse
  available to an API integrator.
- **Number rotation to evade bans.** Cycling through numbers to survive being banned turns the very
  channel this adapter exists to provide into a spam vector, and a rail that becomes a spam vector
  loses the deliverability it exists to provide — the identical structural argument §7.7 already
  makes for why an open SMTP relay dies (accepting everything degrades the reputation that makes
  acceptance possible for anyone). The argument is not repeated by analogy here; it is the **same**
  argument, applied to a different rail's version of the same scarce resource — standing with the
  platform, rather than IP reputation.

## 26.9 Adapters are opt-in; hardware SMS is the in-tree reference (normative)

**The node core ships no adapter beyond one.** No modem drivers, no platform SDKs, and no webhook
handling for WhatsApp, Telegram, Discord, Slack, or an SMS aggregator are part of the node's core
release; an implementation MUST NOT bundle them there. Where such an adapter is offered, it MUST
ship and version **independently** of the node core, so a platform's API churn — and every rail
here but hardware SMS is subject to some vendor's API churn — never touches the node's own release
cycle, exactly as §7's optionality already keeps a node with only native clients free of any
legacy machinery at all (§7 preamble, §7.15.5).

**The one exception is hardware SMS**, shipped **in-tree** as the **reference adapter**, and **off
by default** (a user opts in). It earns the exception because it is the only rail that is
simultaneously:

- **freely-initiating** (§26.3 field 1) — it can reach a stranger cold, exactly like email; and
- runnable **entirely on the user's own hardware**, with **no platform in the path at all** —
  no API, no account, no Tech Provider relationship, nothing a vendor can revoke or meter.

No other rail in this document clears both bars: WhatsApp/Telegram/Discord/Slack cannot initiate
freely at all outside WhatsApp's template-gated case (§26.3, §26.4.2), and the SMS-aggregator path
substitutes a vendor API for the modem, reintroducing exactly the platform dependency hardware SMS
exists to avoid. Hardware SMS is therefore the adapter that **proves the model** — a rail with real
reach and zero external dependency — while every other rail visibly declares, via its own §26.3
fields, what it gives up to get its reach (a platform in the plaintext path, an inbound-triggered
initiation ceiling, or both).

## 26.10 Economics (normative)

**Only SMS has genuine marginal cost.** Every other rail specified here is free at the platform
layer (WhatsApp within its window, Telegram, Discord, Slack all charge nothing to send or receive
through their bot/business APIs) or costs only what the user already pays their own carrier
(hardware SMS). SMS-aggregator delivery, and WhatsApp's outside-window/template-gated sends where a
platform fee applies, are the exceptions, and the tariff/receipt machinery below exists **for**
those exceptions — it is not a general billing layer bolted onto rails that do not need one.

**Gateways publish a signed tariff and a signed capability set.** A gateway-mode operator
publishes, under its own key (the same `AdapterDescriptor` mechanism of §26.3.1, or a `tariff_ref`
it points to), a signed **`Tariff`** (§18.8a.1): the price shape and, where metered, the actual
per-message or per-conversation price, for each rail it bridges. A client holding several gateway
relationships can therefore route a given legacy contact to whichever gateway's published tariff
and capability set actually covers that contact's rail, at a price the client can compare
**before** sending — the same comparison-at-a-glance goal the four-field declaration serves
(§26.3), applied specifically to price. A `Tariff` that fails to verify, or is presented past its
validity window, fails closed with `ERR_ADAPTER_TARIFF_INVALID` (`0x0B01`, §21.11a).

**Signed usage receipts make billing auditable, not asserted.** A gateway-mode operator that meters
usage MUST issue the paying identity a signed `UsageReceipt` (§18.8a.2) for each billable
operation, delivered directly to that identity (a `system` MOTE, `kind = 0x0A`, carrying the
receipt — the same transport already used for a correlated bounce/DSN, §7.10.3a — never a publicly
discoverable object, since a receipt is evidence between two parties, not a public claim). A
`UsageReceipt` that fails to verify fails closed with `ERR_ADAPTER_RECEIPT_INVALID` (`0x0B02`,
§21.11a). This generalizes to
every metered adapter the audit model §7.9 already gives mail: "this operation happened because
*this* message used the adapter" becomes checkable against a receipt the user actually holds,
rather than trusted on the operator's invoice alone. The one-directional limit §7.9 already
discloses for mail applies here too and is not weakened by extending the mechanism: a receipt lets
a user confirm a claimed usage was real, never disconfirm a usage the operator fabricated
end-to-end (§7.9's honest residual, generalized without amendment).

**Free adapters still carry the exposure statement.** A tariff of zero does not remove a
content-visible intermediary: Telegram, Discord, and Slack cost nothing and still put the
platform in the plaintext path on every message (§26.5.1, §26.4). A conformant adapter MUST NOT
let "free" read as "private" — the exposure field (§26.3) is populated identically whether or not
the price shape is `free`.

**Settlement is delegated.** This document specifies no payment rail of its own and takes no cut of
any charge it enables — the same posture postage already takes for mail (§9.5: an issuer with its
own real-money float settles independently of the protocol). Whatever fiat or non-custodial crypto
arrangement a gateway operator and its users already trust is theirs to use; DMTAP names none and
brokers none.

**No protocol token; no advertising; no payment-split or payout-fairness mechanism.** DMTAP has no
token and will not gain one here (§12.2, §3.12.5(d)) — nothing in this document's tariff/receipt
machinery is denominated in one, references one, or creates a role for one. This document
similarly specifies no advertising mechanism and no payment-split or payout-fairness scheme between
operators, aggregators, or platforms: these are absent by decision, not by oversight, and adding
any of them is out of scope for this document.

## 26.11 Wire-shape notes and extension registration (informative)

This section records the wire-format status of §26's objects; nothing in this section is itself
normative on the wire (§18/§21 are), and everything normative above holds regardless.

- **Resolved — `GatewayAuthz` per-rail authorization scope** (§26.2.1 item 1). No longer a grant-
  type addition awaiting definition: `GatewayAuthz` (§12.2, §18.8a.3) is defined, and a per-rail
  grant is a `CapabilityToken` (§18.7.3) with `Capability.resource = "gw-rail:"+rail+":"+remote_id`
  — the same construction used for mail's per-address grant (`"gw-addr:"+address`), differing only
  in the resource string.
- **Resolved — `AdapterDescriptor` and `Tariff`** (§26.3.1, §26.10). Formally defined as the
  general `CoordinatorDescriptor`/`Tariff` objects (§18.8a.1, DS-tags `DMTAP-COORD-v0/descriptor`/
  `DMTAP-COORD-v0/tariff`, §21.24h) with `kind = "gateway"`, not a bespoke adapter-only shape; §26.3.1
  records the field mapping. The earlier, permanently-informal option is superseded.
- **Resolved — Usage receipt object.** `UsageReceipt` (§18.8a.2, DS-tag
  `DMTAP-COORD-v0/usage-receipt`) is formally defined and self-certifying.
- **Still open — `AuthResults` platform-asserted entry** (§26.5.1) — a field addition to
  `GatewayAttestation` (§18.3.11), alongside the `AuthResults` map GW-2 already requires, reported
  separately for that section. Needs: a structurally distinct key (not a value inside the existing
  email-verdict shape) and a §21.24a-style discriminator if more than one non-cryptographic rail
  needs its own sub-shape. Out of scope for this pass: it extends `GatewayAttestation` itself, not
  the descriptor/tariff/receipt/authz family this section formalizes.
- **Still open — usage-receipt body-type discriminator.** `UsageReceipt` (above) rides the existing
  `system` kind (`kind = 0x0A`, §21.16), delivered directly to the payer; if a future `system`-kind
  use needs to share the kind with `UsageReceipt`, the two need a body-type discriminator to tell
  them apart. No such second `system`-kind use exists in this specification today, so no
  discriminator is allocated; not a dangling reference (`0x0B02`'s verification does not depend on
  one), flagged here as a forward-compatibility note only.
- **Registered.** §21.24g reserves subsystem byte `0x0B` for this appendix, and §21 defines
  `ERR_ADAPTER_TARIFF_INVALID` (`0x0B01`), `ERR_ADAPTER_RECEIPT_INVALID` (`0x0B02`) and
  `ERR_ADAPTER_CREDENTIAL_UNAUTHORIZED` (`0x0B03`); §21 is authoritative for their codes and
  fail-classes. §21.24h separately registers the `DMTAP-COORD-v0/…` DS-tags the objects above use.
- **Resolved — §12.3.1 cross-check.** §12.3.1 now scopes its closing sentence to "the entire
  chargeable surface of **the in-tree SMTP/mail gateway**" and forward-references this document's
  §26.10 for the second, smaller, credential-gated category of legitimate charge §26 adds; the
  scoping fix flagged here in an earlier pass has landed.

## 26.12 Honest limits (normative disclosure, disclosed rather than implied)

- **Platform-asserted authenticity is a structural ceiling, not an implementation gap.** No adapter
  for WhatsApp, Telegram, Discord, or Slack can ever offer DKIM-class cryptographic sender
  authentication, in any mode, because the platforms themselves do not offer it over these APIs.
  This is disclosed at §26.5 and MUST NOT be described as a limitation a future adapter version
  will lift.
- **The platform is always a plaintext party on these four rails, in every mode.** Node mode
  removes the *gateway operator* as a second intermediary; it cannot remove the platform as the
  first, because these APIs are business/bot interfaces, not the platform's own end-to-end
  consumer protocol. A client MUST NOT present node-mode WhatsApp/Telegram/Discord/Slack as
  zero-intermediary; only email (self-hosted, native-to-native) and hardware SMS in node mode reach
  that bar, and even hardware SMS is only as private as the SMS network itself, which was never
  end-to-end to begin with.
- **Gateway-mode number/handle loss is worse than mail's alias residual.** §26.6 states this
  plainly: losing a gateway-mode WhatsApp number or bot handle does not just orphan the user, it
  redirects every correspondent who still has that number to whoever the operator assigns it to
  next. This is disclosed, not softened.
- **Reply-routing state is a corruptible, leakable operator asset with no cryptographic check
  against it** (§26.7) — the platform-asserted-only authenticity of these rails means a gateway
  operator's own bookkeeping is the only thing standing between an inbound message and the correct
  recipient, and nothing in this document changes that.
- **Meta's WhatsApp Business Platform pricing is explicitly NOT verified against current terms by
  this document** (§26.4.1) — it has changed repeatedly and any specific number a deployment relies
  on MUST be checked against Meta's own current published terms, not against anything stated here.
- **This document does not solve the flat-namespace problem, mint a name, or offer settlement,
  advertising, or payout-fairness mechanics** (§26.1, §26.10) — each omission is a decision, stated
  once here rather than re-litigated per rail.

## 26.13 Conformance checklist (profile-level MUSTs)

| # | Requirement | Ref |
|---|-------------|-----|
| ADAPT-1 | An adapter serving more than one identity (gateway mode) provides all four of: authorization scope, signed tariff, signed usage receipts, content-visibility disclosure — never fewer | §26.2.1 |
| ADAPT-2 | Every adapter declares all four fields (initiation class, inbound transport class, price shape, exposure); an asymmetric rail (WhatsApp) declares them per direction, not as one row | §26.3, §26.4.1 |
| ADAPT-3 | A template-restricted outbound-cold path is presented as a functional wall (cannot carry arbitrary text at any price), never merely as a higher price tier | §26.4.1 |
| ADAPT-4 | `AuthResults` carries a structurally distinct entry for a platform-asserted claim, never overloading the email-verdict shape | §26.5.1 |
| ADAPT-5 | A client does not render a platform-asserted claim with visual parity to a `dmarc=pass`-aligned sender | §26.5.1 |
| ADAPT-6 | A client can state, for any legacy-rail conversation, whether it is node-mode (portable) or gateway-mode (dies on departure, number/handle reassignable to a stranger) | §26.6 |
| ADAPT-7 | A gateway-mode deployment discloses that it holds a (rail, remote party, number/bot) → identity mapping, as part of its content-visibility disclosure | §26.7, §26.2.1 |
| ADAPT-8 | A WhatsApp adapter defaults to bring-your-own credentials; a BSP-backed option, where offered, is labelled distinctly | §26.8.1 |
| ADAPT-9 | No adapter offers unofficial WhatsApp libraries or ban-evasion number rotation as a credential path | §26.8.2 |
| ADAPT-10 | The node core ships no adapter beyond the hardware-SMS reference adapter (off by default); every other adapter ships and versions independently of the node core | §26.9 |
| ADAPT-11 | A free-price-shape adapter still populates the exposure field; "free" is never presented as "private" | §26.10 |
| ADAPT-12 | A metered adapter issues the paying identity a signed usage receipt per billable operation, delivered directly (never published) | §26.10 |

> **Conformance-suite note.** These checklist items are proposed case names for
> `conformance/SUITE.md` / `conformance/suite.json`; they are not added there by this document
> (conformance/ is maintained by its own workstream). ADAPT-1, ADAPT-6, ADAPT-7, and ADAPT-11 are
> manual-attestation shaped (client UX / deployment disclosure, no wire bytes to recompute, in the
> pattern already established for `GWROLE`-family cases); ADAPT-2 through ADAPT-5, ADAPT-8,
> ADAPT-9, ADAPT-10, and ADAPT-12 have a construction recipe available once the §26.11 wire-shape
> notes land.
