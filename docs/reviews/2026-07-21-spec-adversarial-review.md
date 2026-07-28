# Review record — DMTAP adversarial spec review, 2026-07-21

**Reviewed:** 2026-07-21 · **Reviewer:** Opus, read-only pass · **Filed into KOTVA:** 2026-07-28
**Original path:** `/Users/pc/code/vulos/DMTAP-SPEC-FINDINGS-2026-07-21.md` (untracked, no repo)

This is a **frozen record** ([`README.md`](README.md)). The text under
[Record as written](#record-as-written) is reproduced verbatim and is a statement about the spec
**as it read on 2026-07-21**. Do not edit it to match today's spec.

---

## Triage — 2026-07-28

Every finding re-checked against the spec text in this repo at commit `585a8b1`. The spec moved
substantially in the intervening week: a **spec-perfection pass** (`00bb01b` and predecessors) ran
over §01–§27, and the **mixnet demotion** moved all of `04-transport.md §4.4` into
[`docs/research/mixnet.md`](../research/mixnet.md) as non-normative, opt-in, research-tier material.
Findings 5, 7, 8 and minor 2 all live in that moved text; their section numbers (`§4.4.x`) are
preserved unchanged inside `mixnet.md`, so the citations below still resolve, but the requirements
they discuss are **no longer conformance-mandatory**. That is a change in *status*, not a fix, and
each is graded on its own text below.

**Status legend:** `FIXED` — the defect is gone from current spec text · `FIXED (demoted)` — fixed
*and* the containing text is now non-normative · `LIVE` — still true today · `OPEN QUESTION` —
still true, and closing it needs a protocol decision rather than an editorial fix.

### Summary

| # | Finding (abbreviated) | Original grade | Status now | Evidence checked |
|---|---|---|---|---|
| 1 | A stolen vouch is fully usable by the thief | SERIOUS ✅ | **FIXED** | `02-mote.md` §2.7 step **8(b2)** now requires `Payload.from == Vouch.subject`, discard + no ack; `09-anti-abuse.md` §9.2a rewritten to state the real residual ("that conclusion was wrong") including the framing-primitive disclosure; `21-errors-iana.md` registers `0x0126 ERR_VOUCH_SUBJECT_MISMATCH` with `DROP_SILENT`. Both halves of the proposed fix landed, including the explicit instruction *not* to bind the vouch to `sender_key` at mint time. |
| 2 | §19 tells implementers to ack deferred cold MOTEs | SERIOUS ✅ | **FIXED** | `19-operations.md`: step 9 now reads "hold it in the **requests area** … but **do NOT `ack`**"; the worked cold-sender example ends `B → C: (no ack)  # a DEFERRED MOTE IS NEVER ACKED`; §19.3.2 **Preconditions** now list only *stored* and *previously-acked dedup* as ack-eligible and add a normative rationale block, *"Why deferred is not ack-eligible"*, that names the existence oracle. The failure-mode table gained a row for a duplicate `id` held only in the requests area. |
| 3 | Cold-path caps keyed on a free-to-mint identifier; overflow is "store it anyway" | SERIOUS ⬜ | **FIXED** | `16-parameters.md` §16.5 now carries three rows that did not exist: the floor is re-keyed as **"a policy constraint, not a per-sender count"** with an explicit *"NOT expressed per `sender_key`"* rationale; a separate **unverified-deferral holding class** (≤ 200 entries, ≤ 16 MiB, ≤ 24 h, non-durable) receives §9.4's over-budget path and MUST NOT count as satisfying the floor. `02-mote.md` §2.7a gained the load-bearing distinction the finding asked to be written down: *"'Never silently dropped' governs VERIFIED input; refusing UNVERIFIED input is a different act"*, permitting outright refusal past the aggregate budget. |
| 4 | §2.7 step 5 cannot be executed for the default `DeliveryTag` | SERIOUS ⬜ | **FIXED** | `02-mote.md` now states normatively that a **`KeyTag` envelope MUST be classified COLD**, that a known contact addresses by `BlindedTag`/`GroupTag`, and that "`KeyTag` is the **first-contact** form, not the steady-state one". The §19.3.1 known-contact example was corrected — it now addresses `to: BT_ab` and carries an inline comment saying a `KeyTag` would have forced the COLD path and made the fast path unreachable. |
| 5 | Mandatory Bootstrap→Standard upgrade forces the guard re-draw §4.4.8 forbids | SERIOUS ⬜ | **FIXED (demoted)** | `docs/research/mixnet.md` §4.4.10a constraint 3 gained *"The upgrade GROWS the guard sample; it MUST NOT re-draw it (MUST)"*: retain existing members, top up toward 20 **spread over ≥ 4 mix-key epochs**, and treat post-admission disappearance as an exhaustion/exposure event (`HALT_ALERT`). The rent-capacity-and-publish-attestations attack is written out in full as the rationale. |
| 6 | Key-name has no pinned preimage; multi-suite `Identity` has no single `ik` to hash | SERIOUS ⬜ | **FIXED** | `18-wire-format.md` §18.9.17 now pins `keyname_digest` with a derivation **version byte**, and `Identity.anchor_suite` (key 12) is documented as *"the sole input selecting the anchor key in the zero-authority key-name derivation `Identity.iks[anchor_suite]`; it MUST resolve to exactly one `iks` entry, which is what stops one identity from yielding two distinct key-names"*. The consequence the finding asked to be stated is stated: rotating the anchor **changes every key-name**, and the change **invalidates the committed `keyname_*` vectors** — said plainly rather than absorbed. |
| 7 | §4.4.3 and §4.4.8 are directly contradictory MUSTs on entry selection | WORTH-FIXING ✅ | **FIXED (demoted)** | `docs/research/mixnet.md` §4.4.3 now reads "**layer 0 from the sender's active entry guards (§4.4.8), NOT uniformly at random**, and layers 1..n−1 uniformly at random within the layer", and the fresh-path bullet is scoped "**for the MIDDLE and EXIT hops only**". A paragraph names the old contradiction and quantifies it (a 64 KiB MOTE would have drawn **32 independent entries**, collapsing `(1−f)^G` into cumulative certainty). This was the one finding that was unimplementable-as-written rather than a matter of taste, and it is closed. |
| 8 | Per-contact Bootstrap ratchet reopens the downgrade attack for every *new* contact | WORTH-FIXING ⬜ | **FIXED (demoted)** | `docs/research/mixnet.md` §4.4.10a constraint 4 gained *"Scope limit — the per-contact ratchet applies ONLY to a node that has never reached Standard (MUST)"*, with a node that has ever run Standard **FAIL-QUEUED** (`0x0310`) rather than Bootstrapped, plus the requested second half: *"Fleet-view shrinkage is an anomaly, not a return to youth (MUST)"* → `HALT_ALERT`. |
| 9 | The PoW epoch beacon is unfetchable for exactly the recipients §9.7a protects | WORTH-FIXING ⬜ | **FIXED** | `09-anti-abuse.md`: a recipient MUST accept a proof scoped to *either* its current published beacon *or* the UTC-date fallback within the §16.1 skew window, and **MUST NOT reject a proof solely because it used the coarser scope** — with the cost (coarser precomputation bound) stated plainly rather than elided. |
| 10 | Key-name reachability rests entirely on the DHT — undisclosed honest-limits gap | WORTH-FIXING ⬜ | **FIXED** | `03-naming.md` §3.9.6 took the *first* of the two options the finding offered, normatively: **"A key-name VERIFIES an identity; it does not, by itself, ADDRESS one"** — a stranger holding only a key-name holds "a **checksum, not an address**" and needs `ik` out of band; an implementation **MUST NOT** present a bare key-name as a sufficient destination. `06-privacy.md` §6.6 gained **item 16** as the disclosed residual. The DHT prefix lookup was *not* specified, which is the consistent choice. |
| 11 | `kind = 0x0b` burns a one-time prekey before authentication | WORTH-FIXING ⬜ | **FIXED** | `05-messaging.md` §5.2.1 now specifies **reserve-then-commit** keyed by `ek_a` (reserved at step 7, committed only after step 8 succeeds) **plus** a cold-sender consumption cap (§19.3.1, §16.5) with deliberate fallback to last-resort. Both arms of the proposed fix landed, and §5.2.1 no longer frames exhaustion as a benign capacity event. |
| 12 | *(no finding 12 — the original numbering skips it)* | — | — | Preserved as written; not a lost finding. |
| 13 | §1.4 rule 2 states an ordering the `Threshold` model does not define | WORTH-FIXING ✅ | **FIXED** | `01-identity.md` gained **§1.4a Table B** — "well-formedness of `rotate_threshold` against `recover_threshold` (rule 2, exhaustive and total)", four rows (B1–B4) applied per `rotate_threshold` clause, with implication defined *within* a kind by count and never across kinds. Rule 2 now says explicitly that it is **not** a numeric comparison of the two `Threshold` objects. This is the decision the finding refused to guess at, made. The related sub-item — `recovery_change_is_weakening()` comparing only against the immediately-prior version — is also closed: `crates/kotva-core/src/identity.rs:1183` adds `recovery_change_is_weakening_vs_history()`, and the older function's doc comment now names its own narrowness. |
| M1 | "four size buckets" survives the two-rung ladder | MINOR ✅ | **FIXED** | `06-privacy.md` §6.1 and `docs/research/mixnet.md` both now say **two** size buckets, "at most one bit of size per message". |
| M2 | §4.4.11's low-adoption outcome lacks the consent gate §4.4.9 requires | MINOR ⬜ | **LIVE → fixed in this pass** | Confirmed still true at `585a8b1`: §4.4.11's last bullet said "`private` is unbuildable and delivery **degrades to the `fast` tier**", while §4.4.9 in the same file says downgrading `private → fast` is "**only ever a deliberate, user-surfaced choice** … **never** an automatic reaction to mix unavailability". Two statements in one document, one of them the downgrade attack. Fixed editorially — see [Fixes made](#fixes-made-in-this-pass) F3. |
| M3 | §13.7 item 6 / §6.6 item 6 hang a MUST on the undefined predicate "high-value login RP" | MINOR ⬜ | **OPEN QUESTION** | Confirmed still true. `13-identity-auth.md` item 6 *"DMTAP-Auth **REQUIRES** — even in v0 — that **high-value** login RPs verify the `name → key` binding against multiple independent KT logs … or an out-of-band-verified pin"*; `06-privacy.md` §6.6 item 6 carries the matching MUST. "high-value" appears 9 times across the spec and is **defined nowhere** — not in §0.8, not in §13. Closing it is a protocol decision, not an edit; see [OQ-1](#oq-1--high-value-login-rp-is-an-undefined-self-assessed-trigger-on-a-must). |

### Findings still live in a MUST-level requirement

**One.** `M3` — and it is live in the strongest possible form: a **REQUIRES** whose trigger is
self-assessed by the party the requirement is meant to constrain. Every other finding in the record
is closed. It is carried as [OQ-1](#oq-1--high-value-login-rp-is-an-undefined-self-assessed-trigger-on-a-must)
below rather than patched, because both available closures change what conformance means.

### New defects found *during* this triage

Neither is in the original record. Both are **editorial** — a wire-type name and a rule restatement
that drifted out of agreement with the clauses they cite — and both were fixed in this pass.

| # | Defect | Where | Status |
|---|---|---|---|
| N1 | **`VouchToken` names a wire type §18 does not define.** The fix for finding 1 introduced the term in three places; §18.3.3 defines the `ChallengeResponse` variant as **`Vouch`** (discriminator `4`, `subject` at key 2). An implementer following §2.7 step 8(b2) to §18 finds no `VouchToken`. | `02-mote.md`, `09-anti-abuse.md`, `21-errors-iana.md` | **Fixed** — F1 |
| N2 | **`0x010C`'s registry row states rule 2 as the numeric comparison §1.4a says it is not.** The row read *"`rotate_threshold` < `recover_threshold`"*; §1.4a states rule 2 "**does not** mean a numeric comparison of the two `Threshold` objects" and defines it as the Table B clause-wise test. The registry is where an implementer looks to decide when to raise the code — which is exactly the ambiguity finding 13 said kept `0x010C` unraised. | `21-errors-iana.md` | **Fixed** — F2 |

### Fixes made in this pass

| id | Change | Rationale |
|---|---|---|
| F1 | `VouchToken.subject` → `Vouch.subject` (3 sites), each with a `§18.3.3` citation added | Resolves N1: makes the step-8(b2) MUST followable to a defined type. No normative change — the field, key, and check are identical. |
| F2 | `0x010C` condition restated as "`rotate_threshold` is **not well-formed** against `recover_threshold` under the §1.4a Table B clause-wise test", with the numeric reading explicitly disclaimed | Resolves N2. Aligns the registry with §1.4a's decision instead of preserving the pre-decision wording. |
| F3 | `docs/research/mixnet.md` §4.4.11 final bullet: "delivery degrades to the `fast` tier" → `private` is simply **not offered/unbuilt**, citing §4.4.9's no-auto-downgrade MUST and §6.6 item 13 | Resolves M2. This is a restatement, not a new rule: §4.4.9 already forbids the automatic demotion, and `06-privacy.md` §6.6 item 13 already frames unbuilt-`private` as "paths are simply **unbuilt** … a client MUST NOT present `private` as available before that is true". The §4.4.11 sentence was a leftover from when `private` was the default tier. |

**Not fixed, deliberately:** M3/OQ-1 (needs a decision). Nothing else in the record is live.

### OQ-1 — "high-value login RP" is an undefined, self-assessed trigger on a MUST

**Where.** `13-identity-auth.md` §13.7 item 6 (the owning clause) and `06-privacy.md` §6.6 item 6
(the residual that defers to it).

**The problem.** The requirement is *"high-value login RPs MUST require multi-log consistency or an
OOB-verified pin even in v0."* Nothing defines "high-value". The RP self-assesses, and an RP that
declines the label is conformant by inspection. A MUST nobody can be found in violation of is a
SHOULD wearing a MUST's clothes — and here the thing it is guarding is a **silent per-RP account
takeover** under a v0 split-view KT log, which §6.6 item 6 concedes v0 cannot rule out.

**Why this is not an editorial fix.** Both closures change the conformance surface:

- **(a) Define the trigger.** Enumerate the classes (payments, admin/root consoles, key custody,
  health/legal records, anything that can authorise a transfer or a further credential). *For:*
  keeps the cheap path for low-stakes logins, so the OIDC bridge (§13.6) stays easy to adopt while
  RP support is still bootstrapping (§13.7 item 4 — adoption is already chicken-and-egg).
  *Against:* the enumeration is a judgement call that will be wrong at the edges, still leaves
  self-assessment at the boundary, and imports a category system the rest of the spec does not have.
- **(b) Make multi-log-or-OOB unconditional for DMTAP-Auth in v0.** *For:* mechanically checkable,
  no self-assessment, and it matches the spec's own stance elsewhere — §3.3, §4.4.9 and §13.4 all
  fail closed rather than hand the decision to the party under attack. *Against:* it makes every
  DMTAP-Auth login depend on a **multi-log** deployment that §6.6 item 6 says v0 networks may not
  have ("a network **SHOULD** run multiple independent logs even in v0"), so the honest v0 fallback
  is an OOB-verified pin per RP — real friction on the exact surface §13.6 exists to make
  frictionless. It would also want an error code and a conformance case, neither of which exists.

**What is not in dispute:** leaving it as-is means the strongest requirement in §13 is
unenforceable, and the reference implementation has no trigger to implement — the same shape as
finding 13's `0x010C`, which sat unraised for exactly this reason until §1.4a decided the rule.

**Recommendation (NOT applied — this needs a founder/protocol call):** (b), scoped — unconditional
for the **bridge** (§13.6), which sees all of a user's bridged logins and is the concentration
point, and (a)-style enumeration for direct RP integrations. That is still a decision, so it is
written here and not in the spec.

### What this triage did NOT check

- The record's own **"NOT reached"** list is unchanged and still unexamined: §5.1 committer
  election/ordering, §5.6 CRDT convergence, §7.2–§7.15 gateway internals, §13.1–§13.6 auth
  ceremony, §14, §17, §20–§24. This pass re-checked the 16 findings; it did not extend the review.
- No finding was re-attacked from scratch. A finding marked FIXED means *the text the finding
  identified as defective now says something else, and that something else answers the attack as
  described*. It does not mean the fixed text was itself adversarially reviewed.
- The "Attacked and found SOUND" list was not re-verified against the moved/rewritten spec. Several
  entries (§4.4.x) now sit in non-normative research, which changes what soundness there buys.

---

## Record as written

*Everything below this line is the 2026-07-21 file, verbatim and unmodified. It describes the spec
as it read on that date. Section citations are to the pre-restructure `04-transport.md §4.4`, which
now lives at [`docs/research/mixnet.md`](../research/mixnet.md) with its numbering preserved.*

---

# DMTAP adversarial spec review — findings, 2026-07-21

Opus read-only review, after `make lint` (12 mechanical checks) was already clean at 0 errors.
Scope was deliberately the classes lint **cannot** see: composition failures, unimplementable
requirements, adversary-model gaps, load-bearing hand-waves, honest-limits lapses.

**Status legend:** ✅ VERIFIED by me directly · ⬜ reported, not yet independently checked.

---

## SERIOUS

### 1. ✅ A stolen vouch is fully usable by the thief — and the stated reason it isn't is wrong
`§9.2a` (Vouch bullet) × `§2.7` step 8 × `§18` `VouchToken`

§9.2a exempts the vouch from the `sender_key` binding that ARC/PoW/postage all carry, arguing a
stolen vouch "still fails identity authentication inside `ciphertext` at §2.7 step 8". **It does
not.** Step 8 verifies `Payload.sig` under `Payload.from` — a field the *attacker* chooses and
signs with their own key. Verified: `18-wire-format.md:351,384` define `VouchToken.subject` and a
signature over `(subject, recipient, exp)`, and **nothing anywhere requires
`Payload.from == VouchToken.subject`.**

Mallory captures Alice's vouch for Carol off the wire (§9.2a's own premise is that it travels in
cleartext), mints a fresh ephemeral key, seals a payload as herself, and is **delivered to the
inbox as a vouched contact**, repeatable until `exp`. Second harm: §9.2a rate-limits per
`subject`, so Mallory's flood is charged against **Carol's** budget — a framing primitive against
the innocent party.

Matters because §9.7 elevates vouch to a *primary* tier and calls it the only mechanism "an
adversary cannot buy with either compute or money". As specified it is the one tier obtainable by
**copying**, and it grants the strongest standing (bypasses VDF/PoW/stamp entirely).

**Fix.** (a) §2.7 step 8 / §19.3.1 step 8 MUST verify `Payload.from == VouchToken.subject`;
mismatch → discard, no ack. (b) Replace §9.2a's "no additional binding required" with the real
residual: the check is necessarily *post-decryption*, so a stolen vouch still buys one decryption
and one charge against `subject`'s budget. Do **not** bind the vouch to `sender_key` at mint time
— the subject cannot know a future ephemeral key, and a cleartext proof-of-possession would break
sealed sender.

### 2. ✅ §19 tells implementers to ack deferred cold MOTEs — the exact existence oracle §2.6 closes
`19-operations.md:895` and `:913–915` × `§2.6`, `§2.7a`, `§19.3.1` step 9, `§20.2`

Verified verbatim: the worked example ends `B → C: ack(id)` with the comment **"deferred MOTEs
ARE acked — only invalid/forged ones are not"**, and §19.3.2's normative **Preconditions** list
**deferred (requests area)** as ack-eligible.

Everywhere else says the opposite, and §2.6 states why: *"re-acking it would leak, to an unproven
sender probing with a duplicate, exactly the existence confirmation the requests-area no-ack rule
withholds."*

An attacker enumerates candidate identity keys with unchallenged cold MOTEs. Non-recipients drop
silently; real recipients defer → **ack** → a liveness/existence oracle over the whole key space,
which §9.7a's mandatory non-zero floor guarantees stays open. With §2.2a's plain `KeyTag` it also
confirms *which node* holds an identity.

§19 is the implementer-facing procedure and claims to reproduce §2.7 *verbatim*. Lint cannot see
this: both readings are well-formed prose.

**Fix.** Example → `B → C: (no ack — deferred, §2.7a)`. Precondition → "**stored** (inbox), or a
dedup of a previously **acked** `id` (§2.6). A **deferred** or **dropped** MOTE MUST NOT be acked."

### 3. ⬜ Every cold-path cap is keyed on a free-to-mint identifier, and the overflow path is "store it anyway"
`§9.7a` + `§16.5` × `§9.4` budget × `§2.7a`

`N_floor` is "≥ 5 cold MOTEs / **sender-key** / day", but §2.2 defines `sender_key` as **ephemeral,
fresh per message** — so the floor bounds nothing, and §9.2's `RateLimit` / §16.4's per-sender
spool cap inherit the vacuity. §9.4's budget is per *delivering connection/relay*, an axis the
attacker also multiplies freely. Beyond budget the recipient MUST **defer without verifying** —
into the requests area, which §2.7a forbids dropping from and retains **30 days**.

Net: saturate the Argon2id budget, then send unlimited cold MOTEs with **garbage** in `challenge`.
They are stored unverified for 30 days. Attacker cost ≈ zero. A conformant node may not refuse.
This converts a CPU DoS into an unbounded **durable-storage** DoS, and §9.11's "native mail needs
no filter" claim rests on a requests area that is an unauthenticated 30-day write channel.

**Fix.** Aggregate (node-wide, per-window) byte+entry budget for the requests area in §16.5, with
§2.7a permitting **drop of unverified** cold MOTEs past it — refusal of unverified input is not
the "silent dropping" the section forbids, and the distinction must be written down. Re-key the
floor as *"a recipient MUST NOT operate a policy under which a valid work proof is never
sufficient"* rather than a per-key count. Route over-budget deferrals to a smaller, shorter-lived,
non-durable holding class so a flood cannot displace verified floor traffic.

### 4. ⬜ §2.7 step 5 cannot be executed for the default `DeliveryTag`
`§2.7` step 5 × `§19.3.1` step 5 example × `§2.2a`

Classification must precede decryption. The only pre-decryption relationship signal is `to` — and
`to` identifies the sender only as a `BlindedTag` or `GroupTag`. For a `KeyTag` (the documented
**default**) `to` is the *recipient's own* key and `sender_key` is ephemeral, so nothing in the
envelope identifies the sender. §19.3.1's known-contact example nonetheless reads *"classify:
alice_ik is a pinned known contact"* — `alice_ik` appears only inside `ciphertext`, two steps later.

Two mutually pinned contacts using `KeyTag` + omitted `challenge` (which §2.2b explicitly permits
for known contacts) → classified cold → deferred → no ack → sender's queue expires at 72 h.
**Established contacts cannot exchange mail.** The alternative reading deletes the cold gate.

**Fix.** Normative: a known contact MUST address by `BlindedTag` (or `GroupTag`); a `KeyTag`
envelope is **always** cold at step 5. Correct the §19.3.1 example; note in §2.2a that `KeyTag` is
the *first-contact* form, not steady state.

### 5. ⬜ The mandatory Bootstrap→Standard upgrade forces the guard re-draw §4.4.8 forbids
`§4.4.10a` constraint 3 × `§16.3` guard-sample rows × `§4.4.8`

Bootstrap's sample is *floor 3*; Standard's is *20*. The mandatory transition therefore requires
enlarging the sample from the current fleet view — an unspecified re-sample, performed
automatically, at a moment an adversary can choose by publishing attested mixes until the target's
derived view crosses the threshold. Sample is then never re-drawn (refresh only on exhaustion), so
adversary guards persist for the node's life. Neither §4.4.9 nor §4.4.10a fires — this is an
*upgrade*.

**Fix.** Specify **growth, not re-sampling**: retain existing members, top up to 20 spread over ≥ k
mix-key epochs so no single epoch's view determines the sample; mass disappearance of freshly
admitted members is an exhaustion/exposure event with `HALT_ALERT`. Add a §16.3 top-up cadence row.

### 6. ⬜ The key-name has no pinned preimage, and multi-suite `Identity` has no single `ik` to hash
`§3.9.6`, `§3.12.4` × `§18.4` `Identity.iks` × `§18.9`

`ik` is a **map** (`{+ u8 => ik-pub}`), 1984 B under `0x02` and 64 B under `0x04`. §3.9.6 says
"`BLAKE3-256(ik)`" without saying which entry, in what encoding, with what domain separation —
and §18.9, which pins a canonical preimage for **every** other hash in the protocol, never
mentions the key-name. Three implementations can produce three different 8-word names for one
identity. This is the load-bearing object for §3.13, §9.7a and §12.3.1 item 6, and it is read
aloud by humans, so ambiguity is a spoofing surface as well as an interop bug.

**Fix.** Add §18.9.x: `keyname_digest = BLAKE3-256(DS-tag ‖ suite_u8 ‖ ik_pub_bytes)` with
`suite = Identity.anchor_suite`, DS-tag distinct from the safety number (§3.4.1) and content
addresses (§18.9.4). Cite from §3.9.6/§3.12.4. State the consequence: an anchor-suite migration
**changes every key-name**, making the `0x04` pivot a network-wide naming event.

---

## WORTH-FIXING

### 7. ✅ §4.4.3 and §4.4.8 are directly contradictory MUSTs on entry selection
Verified: `04-transport.md:529` mandates drawing "one mix **uniformly at random from each layer**"
with an independent path per cell; `:692` mandates "a sender **MUST NOT** choose a fresh entry mix
per packet". §4.4.8 explains at length why §4.4.3's behaviour is the failure it exists to prevent,
but §4.4.3 was never amended — and it is the section an implementer reads for path construction.
A literal reading destroys the `(1−f)^G` bound entirely.

**Fix.** §4.4.3: layer 0 is drawn from the sender's **active entry guards**; layers 1..n−1 uniform
per layer, independent per cell. State that "independent path per cell" means independent
**middle and exit**.

### 8. ⬜ The per-contact Bootstrap ratchet reopens the downgrade attack for every *new* contact
§4.4.10a constraint 3 (node-global) and constraint 4 (per-contact) disagree on scope. An adversary
suppressing the derived fleet view (feasible in v0 — §4.4.2 concedes a single non-gossiped KT log
can split-view the mix set) makes Standard unsatisfiable; established contacts fail closed
correctly, but every **new** contact drops to Bootstrap — 3 hops, best-effort ≥2 ASN, sample floor
3 — a path the adversary may occupy end to end. A small fleet and an eclipsed view are
indistinguishable to the client, so the disclosure reads as "young network", not "under attack".

**Fix.** A node that has *ever* run at Standard MUST NOT build Bootstrap paths for **any** contact;
unreachable-at-Standard is FAIL-QUEUED (`0x0310`). Keep the per-contact ratchet only for nodes that
never reached Standard. Add: a fleet view **shrinking** below a previously observed
Standard-satisfying size is `HALT_ALERT`, not a return to youth.

### 9. ⬜ The PoW epoch beacon is unfetchable for exactly the recipients §9.7a protects
The UTC-date fallback is conditioned on *recipient* behaviour but decided by the *sender*, who
cannot distinguish "no beacon published" from "beacon unreachable" → silent undeliverability. And
a key-name-only identity (`self`, "no lookup, no authority") has **no publication surface** for a
beacon at all, so §3.13.5's cold-start user cannot obtain the floor's mandatory freshness input.

**Fix.** A recipient MUST accept a proof scoped to **either** its current beacon **or** the UTC
fallback within the §16.1 skew window, and MUST NOT reject solely for the coarser scope.

### 10. ⬜ Key-name reachability rests entirely on the DHT — an undisclosed honest-limits gap
The key-name is defined **forward only** (key → name). Addressing needs name → key → location, and
80 bits of `BLAKE3-256(ik)` does not yield `ik` (needed for the HPKE seal, the `DeliveryTag`, the
PoW scope, and the DHT key itself). The only closing mechanism would be a DHT prefix lookup that
is **never specified** — and which §4.2.1 relegates to "opportunistic only", saying no established
relationship depends on it. §3.13.4, whose entire purpose is stating what a free user does *not*
get, lists reachability among the guarantees **without the DHT caveat**.

**Fix.** Either state normatively that a key-name is a *verification* artifact and a stranger needs
the full `ik` out-of-band, **or** specify the DHT prefix lookup with candidate verification. Then
correct §3.9.6's "may be typed at to reach someone" and §3.12.4's "no lookup" to match, and add a
§6.6 residual: *cold contact to a key-name-only identity is DHT-dependent and eclipse-deniable;
the floor guarantees acceptance, not reachability.*

### 11. ⬜ `kind = 0x0b` burns a one-time prekey before authentication
§2.7 step 7 consumes/marks-spent the referenced `opk` (and under PQXDH a one-time KEM key)
**before** step 8's identity check. 100 cold `DeniableInit`s with fresh keys and floor-satisfying
proofs exhaust Bob's bundle; legitimate deniable first contacts then fall back to the reused
signed-prekey / last-resort path — the one §5.2.1 identifies as replayable. This is the answer to
"is there a kind where an expensive operation precedes a cheap rejection".

**Fix.** *Reserve* the `opk` at step 7 and *spend* at step 9, releasing on step-8 failure
(preserves the replay defense — a reservation is per-`ek_a`); and/or cap cold-sender prekey
consumption per window in §16.5 with overflow to last-resort. Disclose in §5.2.1, which currently
frames exhaustion as a benign capacity event.

### 13. ✅ §1.4 rule 2 states an ordering the `Threshold` model does not define — so it is unimplementable
`§1.4` rule 2 × `§18` `Threshold` / `MethodPredicate` × `0x010C ERR_RECOVERY_THRESHOLD_INVALID`

Rule 2 reads *"`rotate_threshold` ≥ `recover_threshold`"*. But a `Threshold` is
`any_of: [MethodPredicate]` — a **disjunction over heterogeneous predicates** (`Phrase`,
`Devices(n)`, `Guardians(n)`, `Ik`). That structure has **no total order**, and "≥" is simply
undefined for the common case of disjoint kinds.

Concretely, `recover = {Phrase}` vs `rotate = {Ik, Guardians(2)}` — a policy shape the reference
implementation's own test treats as valid, and which reads as *correct* under the rule's stated
intent (the phrase-holder can recover but cannot rotate). Neither threshold is "≥" the other under
any natural reading:

- **Reading A** — every way of satisfying `rotate` also satisfies `recover` (rotate is a subset,
  therefore harder). **Rejects the valid policy above.** Verified by implementing it: it fails
  `recovery_policy_and_move_record_sign_verify`.
- **Reading B** — no way of satisfying `recover` also satisfies `rotate` (a recovering factor must
  never suffice to rewrite). Passes the policy above, but **rejects `rotate == recover`**, which
  "≥" must permit.

Both are defensible; they disagree; the spec picks neither. **This is why `0x010C` is registered,
cited from §1.4, and raised nowhere in the reference implementation** — `RecoveryPolicy::verify()`
checks only the degenerate empty-`rotate` case and its doc comment quietly narrows the invariant to
"obviously violated (empty rotate)". The rule is not lazily implemented; it is not mechanically
checkable as written.

**I attempted Reading A, it rejected a valid pre-existing policy, and I reverted it.** Shipping a
guessed comparison here would silently reject legitimate recovery policies — worse than the
current under-enforcement, because it would fail closed on honest users.

**Fix (needs a decision, not a patch).** Define the ordering in §1.4 against the actual data model.
The reading that matches the rule's stated danger — *"a single compromised factor may recover but
MUST NOT rewrite"* — is: **for every predicate `p` in `recover_threshold.any_of`, `p` MUST NOT
imply any predicate in `rotate_threshold.any_of`, unless the two thresholds are equal**, with
implication defined within a kind by count (`Devices(m) ⟹ Devices(n)` iff `m ≥ n`) and never
across kinds. State it in §1.4, give it a worked example of an accepted and a rejected policy, and
only then implement `0x010C`.

*(Related, same area, from the envoir conformance pass and NOT yet verified by me:
`recovery_change_is_weakening()` compares only against the immediately-prior version, so re-adding
a previously-evicted `RecoveryMethod` with the same key material reads as purely additive — an
eviction silently undone with no quorum and no veto window. If real, that defeats §1.4 rules 3–4
directly.)*

---

## MINOR (lint-escaped)

- ✅ **"four size buckets"** survives at `04-transport.md:233` and `06-privacy.md:48`; the ladder is
  now **two** rungs and §4.4.1 argues explicitly that a third would raise the leak from 1 bit to
  log₂3. Same class as the §2.5/§18.2 fixes — **C8/C11 have no rule for the prose phrase.**
- ⬜ §4.4.11's low-adoption outcome ("`private` unbuildable → degrade to `fast`") lacks the consent
  gate that §6.6 item 13 and §4.4.9 both require; otherwise it *is* the downgrade attack.
- ⬜ §13.7 item 6 / §6.6 item 6 hang a MUST on the undefined, self-assessed predicate "**high-value**
  login RP". Define the trigger or make multi-log-or-OOB unconditional for DMTAP-Auth in v0.

---

## Attacked and found SOUND

§9.7a ↔ §9.3.1 composition (the floor *is* the answer to the zero-budget rule) · §4.4.2a/§4.4.11
volunteer provisioning (stated as a **bet, labelled as one**, with §6.6 item 13 disclosing the
failure mode) · §9.2a bindings for ARC, PoW and postage (all three correct — only vouch is wrong) ·
§4.4.6 replay caches incl. the re-onion-wrap corollary · §4.4.2 derived mix directory · §4.4.8's
`(1−f)^(G·r)` analysis and prop-271 citation · §6.6 item 1 + §6.10 · §10.7.0 failure classes
(checked §3.3, §4.4.9, §13.4 — no liveness failure misclassified) · §7.1a/b/c · §9.11 ·
§1.2.0 anchor suite · §2.7a's ack asymmetry as designed in §2/§20 · §4.4.10a's core premise · §12.3.1.

## NOT reached — largest unexamined security-bearing surfaces

§5 beyond deniable/vouch interactions: **§5.1 committer election/ordering** and **§5.6 CRDT
convergence**. Also §7.2–§7.15 gateway internals, §13.1–§13.6 auth ceremony, §14, §17, §20–§24.
The reviewer's own next target would be §5.1's `> n/2` roster quorum under partition healing.
