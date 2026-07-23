# 12. Profiles

## 12.1. What a profile is

The core (§2–§11) is domain-neutral. A **profile** specialises it for a kind of
work by defining:

- a profile identifier, used in `WorkOrder.profile` (§3.3);
- the meaning of `Place.role` values;
- profile-specific fields in `body` maps, using CBOR keys 32 and above (§4.5);
- OPTIONAL additional object kinds in the range `0x40`–`0x7f`;
- expected fulfilment proof (§10).

A profile MUST NOT redefine core semantics. It cannot change who may assign,
alter the merge rules, or introduce a state outside §6.1. An implementation
that does not recognise a profile MUST still be able to verify, store, relay
and merge its objects — it simply cannot render them meaningfully. This is what
keeps profiles additive rather than forking.

## 12.2. `delivery/v0`

Moving an item between two places, immediately.

| Field | Key | Notes |
|---|---|---|
| `vehicle` | 32 | `"foot"`, `"bicycle"`, `"motorcycle"`, `"car"`, `"van"` |
| `items` | 33 | Array of `{label, qty, note}` |
| `value` | 34 | Declared value, minor units, for insurance and handling |
| `constraints` | 35 | `"chilled"`, `"fragile"`, `"upright"`, `"age-check"` |

- `Place.role` — `"pickup"`, `"dropoff"`.
- `Window.kind` — `0` (immediate) is typical.
- Fulfilment — handoff code (§10.2) RECOMMENDED.
- `Progress` states used: `started` (departed), `in_progress` (collected),
  `completed` (handed over).
- Live location is conveyed as `Progress` objects carrying `at`. These are
  append-only and high-volume; implementations SHOULD rate-limit them and
  SHOULD compact them aggressively after completion (§7.3).

Vocabulary here is deliberately aligned with prior art in the community-owned
delivery space (§16) so that a bridge to those systems is a mapping rather than
a translation.

## 12.3. `trades/v0`

Skilled work at a site, scheduled, often multi-visit, frequently re-quoted
after inspection.

| Field | Key | Notes |
|---|---|---|
| `trade` | 32 | `"plumbing"`, `"electrical"`, `"hvac"`, `"carpentry"`, … |
| `licence` | 33 | Required credential, e.g. `"za:pirb"` |
| `visit` | 34 | `0` quotation, `1` work, `2` follow-up |
| `materials` | 35 | Array of `{label, qty, cost}` |
| `access` | 36 | Site access constraints, key collection, occupant presence |

- `Place.role` — `"site"`.
- `Window.kind` — `1` (scheduled appointment) is typical, and this is the field
  that makes trades expressible at all.
- Fulfilment — beneficiary signature (§10.4) where the client is present;
  `kind = 4` with a note where they are not.
- Multi-visit and re-quotation follow §10.5.

Trades is included as a second profile *in v0 specifically* because designing
against delivery alone produces a core that silently assumes work is immediate,
single-visit and fixed-price. Every one of those assumptions is false for a
plumber, and each would have been baked into the state machine had the second
profile been deferred.

## 12.4. Other applicable domains

Non-normative. Two observations, drawn from surveying adjacent domains
(field service, home care, mutual aid, municipal reporting, agricultural
contracting, roadside assistance, medical courier, inspection, remote
freelance work, event crew calls), shaped the core.

**Geography is optional.** `places` is a MAY field precisely because remote
work — translation, design, code, with no `Place` at all — is a legitimate
domain. An implementation that requires coordinates has narrowed the
protocol for no reason.

**The issuer is not necessarily a business.** A resident reporting a
pothole is an issuer; a municipality is the performer — municipal and
citizen reporting inverts the usual power direction. Nothing in WRAP
privileges commercial issuers, and any profile that assumes one has made an
error.

## 12.5. Registering a profile

Profiles are identified by `"name/vN"` in `WorkOrder.profile`. v0 defines no
registry. Deployments SHOULD use a namespaced identifier for anything not
defined here — `"example.org:towing/v0"` — so that independent profiles do not
collide before a registry exists.
