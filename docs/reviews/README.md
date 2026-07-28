# Reviews — dated records of adversarial passes over the spec

This directory holds **dated, frozen review records**. It is the counterpart to
[`docs/research/`](../research/README.md): research is quarantined *technical* material that may one
day graduate into the normative spec; a review record is quarantined *process* material that never
graduates at all. Neither is normative, and neither is scanned by `make lint` or referenced by any
conformance case.

## The convention

- **One file per pass, named `YYYY-MM-DD-<slug>.md`.** The date is the date the review was
  performed, not the date it landed here.
- **The record is frozen as written.** The original text is reproduced verbatim under a
  `## Record as written` heading. It is a statement about the spec *on that date*, and rewriting it
  to match today's spec would destroy the only thing it is good for — knowing what was true then,
  and therefore what changed.
- **Triage lives at the top, above the record, and is dated separately.** Each finding gets a
  current status and the evidence that was checked to assign it. A later triage pass appends a new
  dated section; it does not edit an earlier one.
- **Exclude these files from automated rename/refactor sweeps.** A workspace-wide product rename
  ran while `2026-07-21-cross-repo-backlog.md` was being written and silently rewrote a word
  *inside* the frozen record. A sweep that "fixes" a frozen record destroys the record. If a name
  in a record no longer resolves, map it in the triage; do not edit the record.
  The one exception on file: the founder later asked for the retired name to be gone from the
  workspace entirely, so item 6's body was updated by hand and the waiver noted in that file's
  triage table. A deliberate, recorded waiver is fine; a sweep is not.
- **A live finding is not closed by being written down here.** Findings that are still true and
  fixable as spec text are fixed in the spec, and the triage cites the commit. Findings that need a
  protocol decision are carried as an explicit open question with the tradeoff stated — never
  guessed at.

## Distinction from `docs/SPEC-PERFECTION.md` (deleted, `00bb01b`)

Working plan documents that drive an in-flight pass live at `docs/<NAME>.md` and are **deleted when
the pass closes** — they are scaffolding. A review record is the opposite: it is the durable
artifact the scaffolding produced, and it is kept.

## Index

| Record | Date | Scope |
|---|---|---|
| [`2026-07-21-spec-adversarial-review.md`](2026-07-21-spec-adversarial-review.md) | 2026-07-21 | Adversarial read of the DMTAP spec for classes `make lint` cannot see: composition failures, unimplementable requirements, adversary-model gaps, honest-limits lapses. 13 numbered findings + 3 minor. |
| [`2026-07-21-cross-repo-backlog.md`](2026-07-21-cross-repo-backlog.md) | 2026-07-21 | Items found during the same pass that belong to **other** repos, plus a defect class for review checklists. |

## Provenance of these two records

Both were written on 2026-07-21 during the spec restructure and then sat **untracked, in no git
repo at all**, at `/Users/pc/code/vulos/` — outside any version control, next to the repo
directories rather than inside one. They were folded into KOTVA on **2026-07-28**, which is also
the date of the first triage pass in each. Original file names, for anyone matching them against a
shell history or an older note:

- `DMTAP-SPEC-FINDINGS-2026-07-21.md` (264 lines, mtime 2026-07-21 18:55)
- `DMTAP-BACKLOG.md` (114 lines, mtime 2026-07-21 22:26)

The originals were deleted only after the copies were verified byte-identical below their triage
headers.
