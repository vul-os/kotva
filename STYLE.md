# KOTVA house style

The whole family must read as **one professionally-written document**, not twenty voices.
Every spec doc — substrate, primitive, profile, cross-cutting — follows this guide. It is the
standard the perfection pass enforces.

---

## 1. Voice

- **Terse and precise.** Short declarative sentences. No filler, no hedging, no marketing.
- **Normative language is BCP-14** (MUST / MUST NOT / SHOULD / SHOULD NOT / MAY), and *only*
  in all-caps when normative. Never use "must" casually.
- **Intellectually honest.** State what a design does *not* do as plainly as what it does.
  Every doc ends with an **Honest residual** section; an overclaim is a defect.
- **No hype words** ("revolutionary", "seamless", "cutting-edge"). No exclamation marks.
- **Active voice, present tense** for rules ("A gateway MUST…", not "It should be ensured…").

## 2. Document skeletons

**Primitive** (`primitives/*.md`):
`# <NAME> — one-line role` → `Status` line → §1 Purpose → §2 Objects it defines (wire sketch) →
§3 Normative rules → §4 Composition (with other primitives) → §5 Binding adopted → §6
Scale-invariance (mesh ↔ global) → §7 Offline / reconcile → §8 Security MUSTs → §9 Honest residual.

**Profile** (`profiles/*.md`):
`# <NAME> — the <product> profile` → `Status` → §1 What this is → §2 Primitives + coordinators +
bindings it composes → §3 Objects / MOTE kinds used → §4 Normative profile rules → §5
Scale-invariance → §6 Offline → §7 Security + declared content-visibility → §8 Honest residual.

**Cross-cutting** (DIRECTION, CONTRACT, THREAT-MODEL, OFFLINE): keep the doc's own logical
structure, but every one opens with a one-paragraph statement of what it governs and ends with
an Honest residual / disclosed-limits section.

## 3. Headings & numbering

- One `#` H1 per file (the title). Sections are `## N. Title`; subsections `### N.M Title`.
- Number sections so cross-references are stable (`§3.2`), matching the existing DMTAP style.
- A retired/renumbered section leaves a **gap** with a stub explaining where content moved —
  never silently renumber, never reuse a retired number.

## 4. Cross-references

- Reference sibling docs by **root-relative path** in links: `[coordinator/CONTRACT.md](coordinator/CONTRACT.md)`.
- Reference sections as `§7.2` (within the family) or `RFC 9420 §5` (external).
- Every internal reference MUST resolve — a dangling `§X` or dead link is a defect.

## 5. Tables & wire shapes

- Prefer a **table** over prose for: rules-with-columns, kind lists, parameter sets,
  conformance checklists. Keep columns tight; no column wider than its content needs.
- Wire shapes are **CDDL-ish fenced blocks** (```` ```cddl ````), commented. State plainly
  whether the object is *new bytes* or *rides an existing substrate object* (MOTE/PUB/SYNC).
- **Conformance checklists** are a table: `| # | Requirement | Ref |` with a stable ID
  (`OFR-7`, `SEC-6a`, `COORD-4`).

## 6. Family vocabulary (use these exact terms)

- **coordinator** — the umbrella for every hired role (the everything-word).
- **relay** — a *content-blind forwarder of KOTVA's own encrypted traffic* (mesh relay,
  media-relay incl. TURN/coturn, reachability SNI-passthrough). Sees no plaintext.
- **adapter** — a *translator/bridge to a foreign network* (mail, SMS, WhatsApp). Terminating;
  sees plaintext. The **mail adapter is the "gateway"** — keep that name for it, and do **not**
  use "gateway" as the umbrella (that is "coordinator").
- **primitive** — one of the six: OFFER · MATCH · RESERVE · REPUTATION · ESCROW · ATTEST.
  ORACLE (⊂ ATTEST) is the oracle coordinator kind; DISPUTE is the arbiter kind; PAY is the
  stablecoin binding — these are **not** primitives.
- **content-visibility** — every intermediary declares one class (`blind` / `blind-routing` /
  `terminating`) at one assurance level (`structural` / `attested` / `declared`).
- **No token. No global published score.** Ever. Money is an existing stablecoin; trust is
  staked existing value.

## 7. Profile naming

- Canonical profile IDs are **descriptive and uniform**: `kotva-mail`, `kotva-commerce`,
  `kotva-work`, `kotva-social`, `kotva-rtc`, `kotva-media`, `kotva-search`, `kotva-reach`.
- Legacy brand names are **aliases where equity exists**: *KOTVA-Mail (a.k.a. DMTAP)*,
  *KOTVA-Commerce (TRACT)*, *KOTVA-Work (WRAP)*. Do **not** mint a new acronym for a new profile.

## 8. The honesty rules (non-negotiable)

- Disclose every trust boundary, every residual, every ceiling. If a coordinator sees
  plaintext, say so. If a claim is `declared`-level (not verifiable), say so.
- A one-directional audit, a `declared`-vs-`attested` gap, an offline double-spend window — all
  disclosed where they occur, never buried.
- Bind, don't reinvent: if a standard exists, adopt it and cite it; new bytes require a stated
  reason nothing existing suffices.
