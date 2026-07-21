#!/usr/bin/env python3
"""
DMTAP specification linter.

Every internal contradiction found during the 2026-07-21 hardening waves was found
by a human or an agent *reading carefully* — four passes, nine contradictions, and
each pass found ones the previous had missed. That does not scale and it does not
hold: the tenth contradiction arrives with the next edit.

This linter turns those nine catches into invariants. Each check below exists
because a real defect got through review:

  C1  dangling section reference      §2.5 kept citing a ladder that had moved
  C2  error code cited, not registered §9.7a cited ERR_POLICY_BELOW_FLOOR before it existed
  C3  code <-> name disagreement       a registry row and its citations can drift apart
  C4  registered, never cited          dead code points accumulate silently
  C5  conformance counts disagree      §10.3 claimed 157 against a 172-case catalog
  C6  case cites a clause that is gone a case can outlive the clause it tests
  C7  registry Action vs failure class 0x0311 stayed FAIL_CLOSED_BLOCK after §10.7.2
                                       reclassified it FAIL-QUEUED — the registry
                                       contradicting the rule it is supposed to encode
  C8  stale terminology                "directory authority" survived its own deletion
  C10 unclassified MUST section        a section gained MUSTs after conformance/scope.json was
                                       written, so the curated coverage denominator silently
                                       stopped covering the whole spec

Exit status is non-zero if any ERROR-level finding fires, so this belongs in CI.
WARN-level findings are reported but do not fail the build.

Usage:  python3 tools/lint.py [--warn-as-error] [--quiet]
"""

from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# ── what we read ──────────────────────────────────────────────────────────────
# Main spec files are numbered NN-name.md and share ONE section namespace (§1.1,
# §22.4 ...). substrate/*.md are standalone capability documents with their own
# local numbering, so their headings must NOT satisfy a main-spec §ref, but their
# *citations* of main-spec sections must still resolve.
SPEC_FILES = sorted(p for p in ROOT.glob("[0-9][0-9]-*.md"))
SUBSTRATE_FILES = sorted(ROOT.glob("substrate/*.md"))

# ── patterns ──────────────────────────────────────────────────────────────────
# "## 1.1 Title", "### 1.2a ...", "### 4.4.10a ...", "#### 22.4.1 ..."
HEADING_RE = re.compile(r"^#{1,4}\s+(\d+(?:\.\d+[a-z]?)*[a-z]?)\s+\S")
# §4.4.10a, §3.5.2(a), §21.15 — capture the numeric part only
SECREF_RE = re.compile(r"§(\d+(?:\.\d+[a-z]?)*)")
ERRCODE_RE = re.compile(r"`(0x[0-9A-Fa-f]{4})`")
ERRNAME_RE = re.compile(r"`((?:ERR|STATUS)_[A-Z0-9_]+)`")
# registry row: | `0xNNNN` | `ERR_NAME` | context | desc | retry | action |
REGROW_RE = re.compile(
    r"^\|\s*`?~*`?(0x[0-9A-Fa-f]{4})`?~*`?\s*\|\s*`?~*`?((?:ERR|STATUS)_[A-Z0-9_]+)`?~*`?\s*\|(.*)$"
)
# Extension blocks register their own codes outside §21's main table: DMTAP-PUB
# owns 0x0900-0x09FF (§22) and the Sync substrate owns 0x0A00-0x0AFF
# (substrate/SYNC.md). A code defined in its owning document is registered.
EXTENSION_REGISTRIES = [ROOT / "22-public-objects.md", ROOT / "substrate" / "SYNC.md"]
# Conformance case ids: family may contain digits (KTV1, PUB, GWALIAS ...)
CASEID_RE = re.compile(r"\bDMTAP-[A-Z][A-Z0-9]*-\d+\b")

# C8: text that should no longer exist anywhere, with why it was removed.
# Keep this list tight — a false positive here is noise in CI forever.
STALE_TERMS: list[tuple[str, str]] = [
    (r"\{\s*2\s*,\s*8\s*,\s*32\s*,\s*64\s*\}",
     "old 4-rung bucket ladder; the ladder is {16, 64} KiB (§4.4.1, §16.3)"),
    (r"\{8,\s*64\}\s*KiB",
     "8 KiB floor cannot hold a conformant suite-0x02 MOTE (11 967 B min); floor is 16 KiB"),
    (r"two components?\b(?![^.]*signature)(?=[^.]*(?:node|gateway|binary|software))",
     "the node/gateway 'two components' framing; roles are flags on one binary (§0.2)"),
    (r"implements only `?0x01`? and MUST reject",
     "told implementers to reject the suite §1.1 REQUIRES them to originate (§18.2)"),
    (r"post stake/bond|stake is slashed|slashing scheme is specified",
     "stake/slashing was removed — it needs an escrow and an adjudicator (§4.4.8)"),
]

# C7: §10.7 failure classes. A registry Action must not contradict the class the
# spec assigns. Encoded narrowly: codes the spec explicitly reclassified.
FAILCLASS_EXPECT: dict[str, str] = {
    "0x0311": "FAIL-QUEUED",  # stale fleet view — §10.7.2; must not block or downgrade
}

# C9: the normative status of each algorithm suite, per §1.1 (governing). Any
# other suite table that restates a status must agree. This check exists because
# §18.2 labelled 0x01 "v0 REQUIRED" and 0x02 "RESERVED" long after §1.1 inverted
# them, and told implementers to reject the suite they must originate — a
# spec-breaking contradiction that four review passes read past.
SUITE_STATUS: dict[str, str] = {
    "0x01": "legacy",     # verify only, MUST NOT originate
    "0x02": "required",   # the v0 originating suite
    "0x03": "reserved",
    "0x04": "reserved",
}
SUITE_ROW_RE = re.compile(r"^\|\s*`(0x0[1-4])`\s*\|(.+)$")

Finding = tuple[str, str, str]  # (level, location, message)


def read(p: Path) -> str:
    return p.read_text(encoding="utf-8")


def collect_headings() -> dict[str, Path]:
    """Every §-addressable heading in the main spec namespace."""
    out: dict[str, Path] = {}
    for p in SPEC_FILES:
        for line in read(p).splitlines():
            m = HEADING_RE.match(line)
            if m:
                out.setdefault(m.group(1), p)
    return out


def parse_registry() -> tuple[dict[str, str], dict[str, str]]:
    """§21 error registry -> (code->name, code->action-column)."""
    code_to_name: dict[str, str] = {}
    code_to_action: dict[str, str] = {}
    reg = ROOT / "21-errors-iana.md"
    if not reg.exists():
        return code_to_name, code_to_action
    sources = [reg] + [q for q in EXTENSION_REGISTRIES if q.exists()]
    # Registry rows wrap: a long Action column continues on following lines until
    # the next row begins. Rejoin each logical row before parsing, or the Action
    # column is silently truncated and C7 cannot see it.
    logical: list[str] = []
    for raw in "\n".join(read(q) for q in sources).splitlines():
        if REGROW_RE.match(raw):
            logical.append(raw)
        elif logical and raw.strip() and not raw.lstrip().startswith(("|", "#")):
            logical[-1] += " " + raw.strip()
    for line in logical:
        m = REGROW_RE.match(line)
        if m:
            code, name, rest = m.group(1).lower(), m.group(2), m.group(3)
            code_to_name[code] = name
            # Rows end with a trailing "|", so the final split element is blank.
            cols = [c.strip() for c in rest.split("|") if c.strip()]
            code_to_action[code] = cols[-1] if cols else ""
    return code_to_name, code_to_action


def check_secrefs(headings: dict[str, Path]) -> list[Finding]:
    """C1: every §x.y cited in prose resolves to a real heading."""
    out: list[Finding] = []
    # A ref may legitimately name a whole section (§21) or a sub-part that is
    # documented inline rather than as its own heading (§3.5.2(a)). Accept a ref
    # if it, or any prefix of it, is a heading.
    known = set(headings)

    def resolves(ref: str) -> bool:
        parts = ref.split(".")
        for i in range(len(parts), 0, -1):
            if ".".join(parts[:i]) in known:
                return True
        # a bare top-level number is a document (§21 -> 21-errors-iana.md)
        return parts[0].isdigit() and any(
            p.name.startswith(f"{int(parts[0]):02d}-") for p in SPEC_FILES
        )

    for p in SPEC_FILES + SUBSTRATE_FILES:
        for n, line in enumerate(read(p).splitlines(), 1):
            for ref in SECREF_RE.findall(line):
                if not resolves(ref):
                    out.append(("ERROR", f"{p.name}:{n}", f"dangling section ref §{ref}"))
    return out


def check_error_codes(code_to_name: dict[str, str]) -> list[Finding]:
    """C2/C3/C4: cited codes exist, names agree, registered codes get used."""
    out: list[Finding] = []
    cited_codes: set[str] = set()
    cited_names: set[str] = set()
    reg_path = ROOT / "21-errors-iana.md"

    for p in SPEC_FILES + SUBSTRATE_FILES:
        if p == reg_path:
            continue  # the registry defines; it does not cite
        text = read(p)
        for n, line in enumerate(text.splitlines(), 1):
            names_here = ERRNAME_RE.findall(line)
            for code in ERRCODE_RE.findall(line):
                c = code.lower()
                cited_codes.add(c)
                if c not in code_to_name and names_here:
                    out.append(("ERROR", f"{p.name}:{n}",
                                f"error code {code} cited but not registered in §21"))
            for name in ERRNAME_RE.findall(line):
                cited_names.add(name)

    registered_names = set(code_to_name.values())
    for name in sorted(cited_names - registered_names):
        out.append(("ERROR", "§21", f"error name {name} cited but not registered"))

    # C4 — registered but never cited anywhere. Informational: a code may be
    # reserved deliberately, so this is a WARN, not a failure.
    for code, name in sorted(code_to_name.items()):
        if code not in cited_codes:
            out.append(("WARN", "§21", f"{code} ({name}) registered but never cited"))
    return out


def check_failclass(code_to_action: dict[str, str]) -> list[Finding]:
    """C7: registry Action must not contradict the §10.7 failure class."""
    out: list[Finding] = []
    for code, expected in FAILCLASS_EXPECT.items():
        action = code_to_action.get(code, "")
        if not action:
            out.append(("WARN", "§21", f"{code} expected to carry a {expected} action; row not found"))
            continue
        if "FAIL_CLOSED_BLOCK" in action and expected == "FAIL-QUEUED":
            out.append(("ERROR", "§21",
                        f"{code} Action says FAIL_CLOSED_BLOCK but §10.7 classifies it {expected} "
                        f"— a liveness failure must queue, never block (§10.7.0)"))
    return out


def check_conformance() -> list[Finding]:
    """C5/C6: catalog, mirror and every stated count agree; cases cite real clauses."""
    out: list[Finding] = []
    suite_md = ROOT / "conformance" / "SUITE.md"
    suite_json = ROOT / "conformance" / "suite.json"
    if not (suite_md.exists() and suite_json.exists()):
        return [("WARN", "conformance/", "SUITE.md or suite.json missing")]

    ids_md = set(CASEID_RE.findall(read(suite_md)))
    try:
        data = json.loads(read(suite_json))
    except json.JSONDecodeError as e:
        return [("ERROR", "conformance/suite.json", f"invalid JSON: {e}")]

    cases = data.get("cases", data if isinstance(data, list) else [])
    ids_json = {c.get("id") for c in cases if isinstance(c, dict) and c.get("id")}

    for missing in sorted(ids_md - ids_json):
        out.append(("ERROR", "conformance/suite.json", f"{missing} in SUITE.md but not mirrored"))
    for missing in sorted(ids_json - ids_md):
        out.append(("ERROR", "conformance/SUITE.md", f"{missing} in suite.json but not catalogued"))

    n = len(ids_json)
    # C5 — every document that states the count must state the same one.
    for p in [ROOT / "10-conformance.md", ROOT / "README.md",
              ROOT / "conformance" / "README.md"]:
        if not p.exists():
            continue
        text = read(p)
        claimed = {int(x) for x in re.findall(r"\b(\d{3})\s+(?:numbered\s+)?cases\b", text)}
        for c in claimed:
            if c != n:
                out.append(("ERROR", p.name,
                            f"states {c} conformance cases; suite.json has {n}"))
    return out


def check_stale_terms() -> list[Finding]:
    """C8: text that should not have survived its own deletion."""
    out: list[Finding] = []
    lint_self = Path(__file__).name
    for p in SPEC_FILES + SUBSTRATE_FILES:
        for n, line in enumerate(read(p).splitlines(), 1):
            for pat, why in STALE_TERMS:
                if re.search(pat, line, re.IGNORECASE):
                    out.append(("ERROR", f"{p.name}:{n}", f"stale: {why}"))
    return out


def check_suite_status() -> list[Finding]:
    """C9: no suite table may contradict §1.1 on a suite's normative status."""
    out: list[Finding] = []
    for p in SPEC_FILES:
        for n, line in enumerate(read(p).splitlines(), 1):
            m = SUITE_ROW_RE.match(line)
            if not m:
                continue
            code, rest = m.group(1), m.group(2).lower()
            # only rows that actually assert a status
            if not any(k in rest for k in ("required", "reserved", "legacy")):
                continue
            want = SUITE_STATUS[code]
            if want == "required" and "required" not in rest:
                out.append(("ERROR", f"{p.name}:{n}",
                            f"{code} is the v0 REQUIRED originating suite (§1.1); row says otherwise"))
            elif want == "legacy" and "required" in rest:
                out.append(("ERROR", f"{p.name}:{n}",
                            f"{code} is LEGACY verify-only (§1.1); row marks it required"))
            elif want == "reserved" and "required" in rest:
                out.append(("ERROR", f"{p.name}:{n}",
                            f"{code} is RESERVED (§1.1); row marks it required"))
    return out


def _must_sections() -> dict[str, tuple[int, Path]]:
    """MUST/MUST NOT count per §-section, attributing each line to the most recent
    heading. "MUST" only counts in caps (BCP 14, per §0.8)."""
    must_re = re.compile(r"\bMUST(?:\s+NOT)?\b")
    per_section: dict[str, tuple[int, Path]] = {}
    for p in SPEC_FILES:
        current = None
        for line in read(p).splitlines():
            m = HEADING_RE.match(line)
            if m:
                current = m.group(1)
                continue
            if current and must_re.search(line):
                n, _ = per_section.get(current, (0, p))
                per_section[current] = (n + 1, p)
    return per_section


def load_scope() -> tuple[dict[str, str], list[Finding]]:
    """conformance/scope.json — the curated denominator: section -> class.

    Returns the map plus findings for any drift between the spec and the file.
    A curated denominator is only honest if the curation is auditable, so a
    section that gained MUSTs since the file was written, or an entry that no
    longer names a real section, is reported rather than silently ignored.
    """
    out: list[Finding] = []
    path = ROOT / "conformance" / "scope.json"
    if not path.exists():
        return {}, [("WARN", "conformance/scope.json", "missing; coverage falls back to the raw denominator")]
    try:
        doc = json.loads(read(path))
    except json.JSONDecodeError as e:
        return {}, [("ERROR", "conformance/scope.json", f"invalid JSON: {e}")]
    classes = set(doc.get("classes", {}))
    per_section = _must_sections()
    cls: dict[str, str] = {}
    for e in doc.get("sections", []):
        sec = e.get("section")
        if sec in cls:
            out.append(("ERROR", "conformance/scope.json", f"§{sec} classified twice"))
        if e.get("class") not in classes:
            out.append(("ERROR", "conformance/scope.json", f"§{sec}: unknown class {e.get('class')!r}"))
        if not (e.get("reason") or "").strip():
            out.append(("ERROR", "conformance/scope.json", f"§{sec}: classified with no reason"))
        if sec not in per_section:
            out.append(("ERROR", "conformance/scope.json", f"§{sec} classified but has no MUSTs in the spec"))
        cls[sec] = e.get("class")
    for sec in sorted(per_section):
        if sec not in cls:
            out.append(("ERROR", "conformance/scope.json",
                        f"§{sec} carries MUSTs but is unclassified — classify it (default IMPL) or the "
                        f"headline coverage number is measuring an incomplete denominator"))
    return cls, out


def coverage_report() -> int:
    """Which normative MUSTs have no conformance case citing their clause?

    The suite is the operational definition of compatibility (§10.3), so a MUST
    with no case is unenforceable — it reads as a requirement and behaves as a
    suggestion. This turns "52 of 194 byte-runnable" into a prioritised worklist:
    the sections below are the ones carrying normative weight that nothing tests.

    This is a FLOOR, and deliberately generous: a section counts as covered if any
    case cites it, not if every MUST in it is exercised. Read it as "at least this
    much is untested", never as a pass mark.

    The headline number is measured against the **IMPL** denominator of
    conformance/scope.json, because not every capitalised MUST is an
    implementation requirement a runner could check: the §21 registry's Action
    column and the §19/§20 appendices restate rules other clauses own, IANA
    allocation policies bind future registrants, and a handful of capitalised
    MUSTs are list headers or counterfactuals and are not requirements at all.
    A curated denominator is only honest if the curation is auditable, so the
    raw number is printed alongside it and every classification in scope.json
    carries a stated reason. Inclusion is the default there: arguable means IMPL.
    """
    suite_md = ROOT / "conformance" / "SUITE.md"
    if not suite_md.exists():
        print("no conformance/SUITE.md")
        return 1

    # Clauses any case cites, plus every prefix (a case on §9.7a covers §9.7a;
    # a case citing §4.4.10a also evidences §4.4).
    cited: set[str] = set()
    for ref in SECREF_RE.findall(read(suite_md)):
        parts = ref.split(".")
        for i in range(len(parts), 0, -1):
            cited.add(".".join(parts[:i]))

    per_section = _must_sections()
    cls, scope_findings = load_scope()
    for level, loc, msg in scope_findings:
        print(f"  {level} {loc}: {msg}")
    if scope_findings:
        print()

    impl = {s: v for s, v in per_section.items() if cls.get(s, "IMPL") == "IMPL"}
    uncovered = {sec: v for sec, v in impl.items() if sec not in cited}

    print("NORMATIVE COVERAGE — IMPL sections with MUSTs and no conformance case\n")
    for sec, (n, p) in sorted(uncovered.items(), key=lambda kv: -kv[1][0])[:25]:
        print(f"  §{sec:<12} {n:>3} MUST(s)   {p.name}")
    if len(uncovered) > 25:
        print(f"  ... and {len(uncovered) - 25} more sections")

    impl_musts = sum(n for n, _ in impl.values())
    unc_musts = sum(n for n, _ in uncovered.values())
    pct = 100 * (impl_musts - unc_musts) / impl_musts if impl_musts else 0
    print(f"\n  IMPL sections       : {len(impl)}, of which uncovered {len(uncovered)}")
    print(f"  IMPL MUSTs          : {impl_musts} total, {unc_musts} in uncovered sections")
    print(f"  NORMATIVE COVERAGE  : {pct:.0f}% of IMPL MUSTs sit in a section some case cites")

    # The full picture, so the curated denominator can be checked against the raw one.
    print("\n  denominator breakdown (conformance/scope.json)")
    order = ["IMPL", "ENCODING", "RESTATEMENT", "PROCEDURE", "NARRATIVE"]
    tot_m = tot_c = 0
    for k in order + sorted(set(cls.values()) - set(order)):
        secs = {s: v for s, v in per_section.items() if cls.get(s, "IMPL") == k}
        if not secs:
            continue
        m = sum(n for n, _ in secs.values())
        cov = sum(n for s, (n, _) in secs.items() if s in cited)
        tot_m += m
        tot_c += cov
        print(f"    {k:<12} {len(secs):>3} sections {m:>5} MUSTs   {100*cov/m if m else 0:>3.0f}% cited")
    print(f"    {'RAW TOTAL':<12} {len(per_section):>3} sections {tot_m:>5} MUSTs   "
          f"{100*tot_c/tot_m if tot_m else 0:>3.0f}% cited")
    return 0


def main() -> int:
    if "--coverage" in sys.argv:
        return coverage_report()

    warn_as_error = "--warn-as-error" in sys.argv
    quiet = "--quiet" in sys.argv

    headings = collect_headings()
    code_to_name, code_to_action = parse_registry()

    findings: list[Finding] = []
    findings += check_secrefs(headings)
    findings += check_error_codes(code_to_name)
    findings += check_failclass(code_to_action)
    findings += check_conformance()
    findings += check_stale_terms()
    findings += check_suite_status()
    findings += load_scope()[1]

    errors = [f for f in findings if f[0] == "ERROR"]
    warns = [f for f in findings if f[0] == "WARN"]

    by_level: dict[str, list[Finding]] = defaultdict(list)
    for f in findings:
        by_level[f[0]].append(f)

    for level in ("ERROR", "WARN"):
        items = by_level.get(level, [])
        if not items or (quiet and level == "WARN"):
            continue
        print(f"\n{level} ({len(items)})")
        for _, loc, msg in items:
            print(f"  {loc}: {msg}")

    print(f"\nscanned {len(SPEC_FILES)} spec + {len(SUBSTRATE_FILES)} substrate files, "
          f"{len(headings)} headings, {len(code_to_name)} registered error codes")
    print(f"{len(errors)} error(s), {len(warns)} warning(s)")

    return 1 if errors or (warn_as_error and warns) else 0


if __name__ == "__main__":
    sys.exit(main())
