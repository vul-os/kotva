#!/usr/bin/env python3
"""
DMTAP conformance-suite runner.

`conformance/suite.json` is a 367-case catalog that §10.3 calls "the definition of
compatibility". Before this file, nothing in this repo executed it: `make lint`
checks the catalog against the *prose*, `tests/conformance_vectors.rs` re-derives
`crates/kotva-core/vectors.json` from the reference crate, and the two never meet.
So a case could name a vector that does not exist, an inline reject case could
assert bytes nobody ever decoded, and the published mirror under `conformance/`
could drift from the crate copy the Rust test actually guards — all green.

What this runs, and what it deliberately does not:

  EXECUTED   `self-contained` cases. Their input bytes are inline (`input.cbor_hex`)
             and their expected outcome is a decode decision, so they are runnable
             with no reference implementation. The strict deterministic-CBOR decoder
             below is written from §18.1.1 / RFC 8949 §4.2 directly and does NOT
             import the Rust crate — same rule as verify_pub_vectors.py: a second
             implementation catches what a generator and a naive check would share.

  BOUND      `vectored` cases. The byte-level re-derivation is `cargo test -p
             kotva-core --test conformance_vectors`; what is unchecked anywhere else
             is that each case actually resolves to a committed vector and to the
             `expected` field it claims to read. That binding is checked here.

  SKIPPED    `construction-todo` and `manual-attestation` cases. These are NOT
             runnable — one has no vector generated yet, the other has no wire bytes
             at all. They are printed, counted, and reasoned about out loud, because
             the failure mode this whole file exists to prevent is a runner that
             reports success while executing nothing.

Coverage floors below make a silent under-run impossible from the other direction:
the executable population may only grow, and every member of it must be executed.

Stdlib only, like tools/lint.py — a gate that can be blocked by a supply-chain
problem is not a gate.

Usage:  python3 tools/conformance.py [--quiet]
Exit status is non-zero on any failure, so this belongs in CI.
"""

from __future__ import annotations

import json
import pathlib
import sys
from collections import Counter

ROOT = pathlib.Path(__file__).resolve().parent.parent
SUITE = ROOT / "conformance" / "suite.json"
VECTOR_DIR = ROOT / "conformance" / "vectors"
CRATE_MIRROR = ROOT / "crates" / "kotva-core" / "vectors.json"

# ── Coverage floors ──────────────────────────────────────────────────────────────
# These ratchet upward only. A vector corpus that grows while the number of executed
# cases stays put is exactly the silent under-run this runner exists to catch, and
# the `executed == executable` invariant below catches it without anyone editing
# these numbers. The floors are the second line: they catch the opposite failure —
# cases *disappearing* from the catalog, which `executed == executable` alone would
# happily call a pass.
MIN_CASES = 367
MIN_EXECUTED = 6  # self-contained cases (all det_cbor_decode reject vectors today)
MIN_BOUND = 57  # vectored cases resolved to a committed vector
MIN_VECTORS = 111  # committed vectors across conformance/vectors/*.json
MIN_CBOR_BLOBS = 107  # committed `*cbor_hex` blobs re-decoded for §18.1.1 conformance

STATUS_RUNNABLE = "self-contained"
STATUS_BOUND = "vectored"


# ── Strict deterministic CBOR (§18.1.1, RFC 8949 §4.2) ───────────────────────────
# Written from the spec text, not ported from crates/kotva-core/src/cbor.rs. The
# rule numbers in the rejections match the §18.1.1 sub-rules the cases cite.
class NotDeterministic(Exception):
    pass


def _head(buf: bytes, i: int) -> tuple[int, int, int]:
    if i >= len(buf):
        raise NotDeterministic("truncated: no head byte")
    b0 = buf[i]
    major, ai = b0 >> 5, b0 & 0x1F
    i += 1
    if ai < 24:
        return major, ai, i
    if ai == 31:
        raise NotDeterministic("indefinite-length item (rule 1)")
    if ai in (28, 29, 30):
        raise NotDeterministic(f"reserved additional-info {ai}")
    n = {24: 1, 25: 2, 26: 4, 27: 8}[ai]
    if i + n > len(buf):
        raise NotDeterministic("truncated: argument runs past end")
    val = int.from_bytes(buf[i : i + n], "big")
    i += n
    # Rule 1: preferred (shortest) encoding of the argument.
    shortest = 0 if val < 24 else 1 if val < 2**8 else 2 if val < 2**16 else 4 if val < 2**32 else 8
    if n != shortest:
        raise NotDeterministic(f"non-shortest argument encoding for {val} (rule 1)")
    return major, val, i


def _value(buf: bytes, i: int, depth: int = 0) -> int:
    """Consume one deterministic-CBOR value starting at `i`; return the next offset."""
    if depth > 64:
        raise NotDeterministic("nesting deeper than 64")
    b0 = buf[i] if i < len(buf) else None
    major, arg, i = _head(buf, i)
    if major in (0, 1):
        return i
    if major in (2, 3):
        if i + arg > len(buf):
            raise NotDeterministic("truncated string payload")
        return i + arg
    if major == 4:
        for _ in range(arg):
            i = _value(buf, i, depth + 1)
        return i
    if major == 5:
        prev: bytes | None = None
        for _ in range(arg):
            kstart = i
            i = _value(buf, i, depth + 1)
            key = buf[kstart:i]
            # Rules 2 and 3: keys strictly bytewise-ascending (which subsumes "no duplicates").
            if prev is not None and key <= prev:
                what = "duplicate" if key == prev else "descending"
                raise NotDeterministic(f"map keys not bytewise-ascending: {what} key (rules 2, 3)")
            prev = key
            i = _value(buf, i, depth + 1)
        return i
    if major == 6:
        raise NotDeterministic("tag present (rule 5)")
    # major 7
    if arg in (25, 26, 27) or (b0 is not None and (b0 & 0x1F) in (25, 26, 27)):
        raise NotDeterministic("floating-point value (rule 4)")
    if arg in (20, 21):
        return i  # false / true
    raise NotDeterministic(f"simple value {arg} — undefined/null/unassigned (rule 5)")


def det_cbor_decode(data: bytes) -> None:
    """Raise NotDeterministic unless `data` is exactly one deterministic-CBOR item."""
    end = _value(data, 0)
    if end != len(data):
        raise NotDeterministic(f"{len(data) - end} trailing byte(s) after the top-level item")


# ── Loading ──────────────────────────────────────────────────────────────────────
def load_vectors() -> tuple[dict, dict[str, str], list[str]]:
    """name -> vector, name -> source filename, plus any load-time failures."""
    byname: dict[str, dict] = {}
    source: dict[str, str] = {}
    fails: list[str] = []
    for path in sorted(VECTOR_DIR.glob("*.json")):
        try:
            doc = json.loads(path.read_text())
        except json.JSONDecodeError as e:
            fails.append(f"{path.name}: invalid JSON: {e}")
            continue
        for vec in doc.get("vectors", []):
            name = vec.get("name")
            if name in byname:
                fails.append(f"{path.name}: vector name {name!r} also defined in {source[name]} — "
                             "a case referencing it would resolve ambiguously")
                continue
            byname[name] = vec
            source[name] = path.name
    return byname, source, fails


def as_list(x) -> list:
    if x is None:
        return []
    return x if isinstance(x, list) else [x]


# ── Checks ───────────────────────────────────────────────────────────────────────
def run_self_contained(cases: list[dict]) -> tuple[int, list[str]]:
    """Execute every inline-byte case against the strict decoder above."""
    fails: list[str] = []
    executed = 0
    for c in cases:
        if c.get("status") != STATUS_RUNNABLE:
            continue
        cid = c["id"]
        op = c.get("operation")
        if op != "det_cbor_decode":
            fails.append(f"{cid}: self-contained case has operation {op!r}, which this runner "
                         "cannot execute — teach it the operation or reclassify the case")
            continue
        hexbytes = (c.get("input") or {}).get("cbor_hex")
        if not hexbytes:
            fails.append(f"{cid}: self-contained case has no input.cbor_hex")
            continue
        want = (c.get("expect") or {}).get("outcome")
        try:
            det_cbor_decode(bytes.fromhex(hexbytes))
            got, why = "accept", ""
        except NotDeterministic as e:
            got, why = "reject", str(e)
        executed += 1
        if got != want:
            fails.append(f"{cid}: expected {want}, got {got}"
                         f"{' (' + why + ')' if why else ''} for 0x{hexbytes}")
    return executed, fails


def bind_vectored(cases: list[dict], byname: dict[str, dict]) -> tuple[int, list[str]]:
    """Every vectored case must resolve to a committed vector and to the field it reads."""
    fails: list[str] = []
    bound = 0
    for c in cases:
        if c.get("status") != STATUS_BOUND:
            continue
        cid = c["id"]
        names = as_list(c.get("vector")) or as_list(c.get("reuses_vector"))
        if not names:
            fails.append(f"{cid}: status 'vectored' but names neither `vector` nor `reuses_vector`")
            continue
        ok = True
        for name in names:
            vec = byname.get(name)
            if vec is None:
                fails.append(f"{cid}: names vector {name!r}, which is in no conformance/vectors/*.json")
                ok = False
                continue
            expected = vec.get("expected")
            if not isinstance(expected, dict):
                continue
            for field in as_list((c.get("expect") or {}).get("field")):
                if field not in expected:
                    fails.append(f"{cid}: reads expect.field {field!r} from vector {name!r}, "
                                 f"which has no such field (has: {sorted(expected)})")
                    ok = False
        if ok:
            bound += 1
    return bound, fails


def check_committed_cbor(byname: dict[str, dict]) -> tuple[int, list[str]]:
    """Every committed `*cbor_hex` in the corpus MUST itself be deterministic CBOR (§18.1.1).

    Two jobs. It is a real invariant — a vector encoding non-preferred integers would
    hand every implementer bytes the spec tells them to reject. It is also the guard
    that keeps the decoder above honest: a decoder that rejects everything would pass
    all six reject cases and look perfect, and this is what fails it."""
    fails: list[str] = []
    checked = 0
    for name, vec in sorted(byname.items()):
        for holder in (vec.get("input"), vec.get("expected")):
            if not isinstance(holder, dict):
                continue
            for key, val in holder.items():
                if not key.endswith("cbor_hex"):
                    continue
                for idx, item in enumerate(as_list(val)):
                    if not isinstance(item, str):
                        continue
                    checked += 1
                    try:
                        det_cbor_decode(bytes.fromhex(item))
                    except (NotDeterministic, ValueError) as e:
                        where = f"{key}[{idx}]" if isinstance(val, list) else key
                        fails.append(f"vector {name!r} {where} is not deterministic CBOR: {e}")
    return checked, fails


def check_mirror_drift() -> list[str]:
    """conformance/vectors/vectors.json is the published copy of the crate's vectors.json.

    Only the crate copy is guarded by tests/conformance_vectors.rs, so the published
    mirror can go stale against the reference and nothing notices. The two are one
    file maintained in two places (see examples/gen_vectors.rs: "sync this file
    there"), so byte-identity is the invariant."""
    published = VECTOR_DIR / "vectors.json"
    if not published.exists() or not CRATE_MIRROR.exists():
        return [f"missing vector mirror: {published} or {CRATE_MIRROR}"]
    if published.read_bytes() != CRATE_MIRROR.read_bytes():
        return [f"{published.relative_to(ROOT)} differs from {CRATE_MIRROR.relative_to(ROOT)} — "
                "the published mirror is stale against the copy the Rust drift test guards; "
                "re-run `cargo run -p kotva-core --example gen_vectors` and copy it across"]
    return []


def check_declared_counts(suite: dict, cases: list[dict], byname: dict[str, dict]) -> list[str]:
    """suite.json states its own tallies. Hand-maintained numbers drift (lint's C5/C14
    exist for exactly this in the prose); these are the same numbers in the machine
    mirror, and nothing recomputed them."""
    fails: list[str] = []
    published = json.loads((VECTOR_DIR / "vectors.json").read_text())
    n_published = len(published.get("vectors", []))
    if suite.get("vectors_count") != n_published:
        fails.append(f"suite.json vectors_count={suite.get('vectors_count')} but "
                     f"{suite.get('vectors_file')} holds {n_published}")

    # referenced_vectors_count is defined as vectors *in vectors_file* that byte-backed
    # cases drive — vectors living in the sibling pub_/pubsub_/sync_ files do not count.
    in_published = {v["name"] for v in published.get("vectors", [])}
    referenced = set()
    for c in cases:
        referenced.update(as_list(c.get("vector")))
        referenced.update(as_list(c.get("reuses_vector")))
    n_ref = len(referenced & in_published)
    if suite.get("referenced_vectors_count") != n_ref:
        fails.append(f"suite.json referenced_vectors_count={suite.get('referenced_vectors_count')} "
                     f"but {n_ref} of {suite.get('vectors_file')}'s vectors are actually named by a case")

    tiers = Counter(c.get("ratification_tier") for c in cases)
    for tier, declared in (suite.get("ratification_tiers") or {}).items():
        if isinstance(declared, int) and tiers[tier] != declared:
            fails.append(f"suite.json ratification_tiers.{tier}={declared} but {tiers[tier]} cases carry it")
    return fails


def main() -> int:
    quiet = "--quiet" in sys.argv

    if not SUITE.exists():
        print(f"FAIL: {SUITE} does not exist — the conformance catalog is the thing under test, "
              "so a missing catalog is a failure, not a skip")
        return 1
    suite = json.loads(SUITE.read_text())
    cases = suite["cases"]
    byname, source, fails = load_vectors()

    executed, f = run_self_contained(cases)
    fails += f
    bound, f = bind_vectored(cases, byname)
    fails += f
    blobs, f = check_committed_cbor(byname)
    fails += f
    fails += check_mirror_drift()
    fails += check_declared_counts(suite, cases, byname)

    by_status = Counter(c.get("status") for c in cases)
    runnable = by_status[STATUS_RUNNABLE]
    bindable = by_status[STATUS_BOUND]
    reasons = suite.get("status_values") or {}

    # The invariant that survives a growing corpus: everything classified runnable was run,
    # everything classified vectored was bound. No number below needs editing for that to hold.
    if executed != runnable:
        fails.append(f"{runnable} case(s) are classified '{STATUS_RUNNABLE}' but only {executed} ran")
    if bound != bindable:
        fails.append(f"{bindable} case(s) are classified '{STATUS_BOUND}' but only {bound} bound")

    # Ratchets: catch the catalog *shrinking*.
    for label, got, floor in (("cases", len(cases), MIN_CASES),
                              ("executed", executed, MIN_EXECUTED),
                              ("bound", bound, MIN_BOUND),
                              ("vectors", len(byname), MIN_VECTORS),
                              ("cbor_blobs", blobs, MIN_CBOR_BLOBS)):
        if got < floor:
            fails.append(f"{label}: {got} < floor {floor} — the suite shrank. If that is deliberate, "
                         f"lower MIN_{label.upper()} in this file in the same commit and say why")

    if not quiet:
        print(f"conformance/suite.json — {len(cases)} cases, {len(byname)} committed vectors "
              f"across {len(set(source.values()))} file(s)\n")
        print(f"  EXECUTED  {executed:>3}  self-contained cases decoded here against a from-scratch "
              "§18.1.1 decoder")
        print(f"  BOUND     {bound:>3}  vectored cases resolved to a committed vector "
              "(bytes re-derived by `cargo test -p kotva-core --test conformance_vectors`)")
        print(f"  DECODED   {blobs:>3}  committed cbor_hex blobs re-decoded and accepted as §18.1.1 CBOR")
        # Skips are printed, never implied. A runner that says nothing about what it did
        # not do is indistinguishable from one that did nothing.
        skipped_total = 0
        for status, n in sorted(by_status.items()):
            if status in (STATUS_RUNNABLE, STATUS_BOUND):
                continue
            skipped_total += n
            print(f"\n  SKIPPED   {n:>3}  status '{status}' — NOT executed by any runner.")
            print(f"            reason: {reasons.get(status, '(no reason recorded in suite.json)')}")
            ids = [c["id"] for c in cases if c.get("status") == status]
            print(f"            e.g. {', '.join(ids[:6])}{' …' if len(ids) > 6 else ''}")
        print(f"\n  {executed + bound}/{len(cases)} cases covered; {skipped_total} not runnable "
              f"({100 * skipped_total / len(cases):.0f}% of the catalog)")

    if fails:
        print(f"\nFAIL ({len(fails)})")
        for msg in fails:
            print(f"  {msg}")
        return 1
    print("\nOK — every runnable case ran, every vectored case resolved.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
