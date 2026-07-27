#!/usr/bin/env python3
"""
TRACT specification linter.

DMTAP's linter exists because four review passes over one restructure found nine internal
contradictions, and each pass found ones the previous had missed. That does not scale: the tenth
arrives with the next edit. This linter is the same idea applied from the start, so TRACT never
accumulates the debt DMTAP had to pay off retroactively.

There are fewer checks than DMTAP's, because TRACT has fewer registries so far. Each one guards an
invariant that is cheap to break and expensive to notice:

  T1  dangling section reference     a §ref pointing at a section that does not exist
  T2  document map disagrees         the §0.8 table vs the files actually on disk
  T3  normative language in a stub   a MUST inside a section still marked non-normative,
                                     which is how a draft silently starts claiming authority
  T4  unmarked stub                  a section with neither the drafting-status note nor any
                                     requirement keyword — a reader cannot tell if it is done
  T5  reference cited, not listed    an RFC cited in the body but missing from §20

Exit status is non-zero if any ERROR fires, so this belongs in CI. WARN findings are reported but
do not fail the build unless --warn-as-error is passed.

Usage:  python3 tools/lint.py [--warn-as-error] [--quiet]
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

SECTION_RE = re.compile(r"^(\d\d)-.*\.md$")
# A TRACT section number is at most two digits (00-21 today, room to 99). A "§" followed by three
# or more digits is a statutory citation — §6041, §1101(e), §151.0242 — not a reference into this
# document, and §21's legal grounding is full of them. The negative lookahead rejects those without
# also rejecting ordinary subsection refs like §11.2, whose next character is a dot rather than a
# digit. Before this, citing California's Revenue & Taxation Code made the linter demand a
# 6041-*.md.
SECTION_REF_RE = re.compile(r"§(\d{1,2})(?!\d)")
RFC_RE = re.compile(r"RFC\s*(\d{3,5})")
REQ_KEYWORDS = ("MUST NOT", "MUST", "SHALL", "SHOULD", "REQUIRED", "MAY")
# A section is a non-normative *stub* only if its status banner explicitly says so. The banner
# itself is universal ("> **Drafting status …**" / "> **Status …**") and is NOT a stub sentinel.
STUB_DECLARATIONS = ("non-normative", "placeholder", "not yet authored", "not normative", " stub")

findings: list[tuple[str, str, str]] = []  # (level, file, message)


def err(f: str, m: str) -> None:
    findings.append(("ERROR", f, m))


def warn(f: str, m: str) -> None:
    findings.append(("WARN", f, m))


def load_sections() -> dict[int, tuple[str, str]]:
    """number -> (filename, text)"""
    out: dict[int, tuple[str, str]] = {}
    for p in sorted(ROOT.glob("*.md")):
        m = SECTION_RE.match(p.name)
        if not m:
            continue
        out[int(m.group(1))] = (p.name, p.read_text(encoding="utf-8"))
    return out


def check_dangling_refs(secs: dict[int, tuple[str, str]]) -> None:
    """T1 — a §ref must name a section that exists."""
    known = set(secs)
    for _num, (name, text) in sorted(secs.items()):
        for ref in sorted({int(m.group(1)) for m in SECTION_REF_RE.finditer(text)}):
            if ref not in known:
                err(name, f"§{ref} referenced but no section {ref:02d}-*.md exists")


def check_document_map(secs: dict[int, tuple[str, str]]) -> None:
    """T2 — the §0.8 map and the files on disk must agree, in both directions."""
    if 0 not in secs:
        err("(repo)", "00-overview.md is missing; there is no document map to check")
        return
    overview = secs[0][1]
    listed = {int(m) for m in re.findall(r"^\|\s*§(\d+)\s*\|", overview, re.M)}
    on_disk = set(secs) - {0}
    for missing in sorted(on_disk - listed):
        err("00-overview.md", f"§{missing} exists on disk but is absent from the §0.8 document map")
    for phantom in sorted(listed - on_disk):
        err("00-overview.md", f"§{phantom} is in the §0.8 document map but has no file")


def _status_banner(text: str) -> str:
    """The section's status-banner line (first blockquote naming a Drafting-status/Status), or ""."""
    for ln in text.splitlines():
        s = ln.lstrip()
        if s.startswith(">") and ("Drafting status" in s or "Status:" in s or "**Status" in s):
            return ln
    return ""


def check_stub_discipline(secs: dict[int, tuple[str, str]]) -> None:
    """T3/T4 — a section is either a marked stub or has requirement language, never both/neither."""
    for num, (name, text) in sorted(secs.items()):
        if num == 0:
            continue  # the overview is normative prose by construction
        # Every TRACT section opens with a "> **Drafting status …**" banner, so banner PRESENCE is not
        # a stub marker — treating it as one T3-errored on every settled section's legitimate MUSTs.
        # A section is a non-normative *stub* only when its banner explicitly declares itself so; a
        # banner claiming normative force — including "partially normative" — is not a stub even
        # though it, too, opens with "Drafting status".
        banner = _status_banner(text)
        low = banner.lower()
        declares_nonnormative = any(m in low for m in STUB_DECLARATIONS)
        declares_normative = "normative" in low.replace("non-normative", "").replace("not normative", "")
        is_stub = declares_nonnormative and not declares_normative
        # Exclude the banner line itself from the keyword scan, so its own wording (e.g. a banner that
        # spells out "The key words MUST, MUST NOT, SHOULD …") is never counted as a requirement hit.
        body = text.replace(banner, "", 1) if banner else text
        hits = [k for k in REQ_KEYWORDS if re.search(rf"\b{re.escape(k)}\b", body)]
        if is_stub and hits:
            err(name, f"carries RFC 2119 keyword(s) {sorted(set(hits))} while still marked "
                      "non-normative — drop the drafting-status note, or soften the language")
        if not is_stub and not hits:
            warn(name, "has neither a drafting-status note nor any requirement keyword — "
                       "a reader cannot tell whether it is finished")


def check_references(secs: dict[int, tuple[str, str]]) -> None:
    """T5 — an RFC cited anywhere must appear in the §20 reference tables."""
    refs = next(((n, t) for n, t in secs.values() if n.startswith("20-")), None)
    if refs is None:
        warn("(repo)", "no 20-references.md; skipping reference-completeness check")
        return
    listed = set(RFC_RE.findall(refs[1]))
    for _num, (name, text) in sorted(secs.items()):
        if name.startswith("20-"):
            continue
        for rfc in sorted(set(RFC_RE.findall(text))):
            if rfc not in listed:
                warn(name, f"cites RFC {rfc} but §20 does not list it")


def main() -> int:
    warn_as_error = "--warn-as-error" in sys.argv
    quiet = "--quiet" in sys.argv

    secs = load_sections()
    if not secs:
        print("lint: no NN-*.md sections found", file=sys.stderr)
        return 1

    check_dangling_refs(secs)
    check_document_map(secs)
    check_stub_discipline(secs)
    check_references(secs)

    errors = [f for f in findings if f[0] == "ERROR"]
    warns = [f for f in findings if f[0] == "WARN"]

    if not quiet or errors:
        for level, f, m in findings:
            print(f"{level:5}  {f}: {m}")

    if not quiet:
        print(f"\nlint: {len(secs)} sections, {len(errors)} error(s), {len(warns)} warning(s)")

    if errors:
        return 1
    if warns and warn_as_error:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
