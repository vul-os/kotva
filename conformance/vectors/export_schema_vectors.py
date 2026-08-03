#!/usr/bin/env python3
"""Lift the DEPOT + §18.8a schema vectors out of Rust into language-neutral JSON.

WHY THIS EXISTS. `crates/kotva-depot/tests/vectors.rs` states its own goal: "a second
implementation — in any language — can be checked against the same corpus without running this
crate." That was not true. The hex lived only as Rust `const &str`, so the Python decoder had to
regex-parse Rust source to reach it — fragile, and something every further implementation would
have to re-implement.

WHAT THIS IS NOT. `pub_vectors.json` in this directory is produced by an INDEPENDENT Python
generator and Rust is checked against it; that file is a genuine second opinion. This one is not.
These bytes were hand-derived from the CDDL and live in Rust first, so this file is a TRANSPORT —
it moves an existing corpus across a language boundary and adds no independent evidence. Said
plainly here so nobody cites it as cross-validation. The second opinion is
`conformance/decoders/python/`, which decodes these bytes against the spec.

Run: python3 conformance/vectors/export_schema_vectors.py [--check]
  --check exits non-zero if the JSON is stale, so CI can fail on drift instead of silently
  serving a corpus that no longer matches the Rust.
"""
import json, re, sys, pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "conformance" / "vectors" / "schema_vectors_v0.json"
SOURCES = {
    "kotva-depot": ROOT / "crates" / "kotva-depot" / "tests" / "vectors.rs",
    "kotva-coordinator": ROOT / "crates" / "kotva-coordinator" / "tests" / "vectors.rs",
}
# A vector whose name says it carries a planted defect belongs in corruption_controls: a decoder
# MUST reject those, so mixing them into `vectors` would invert the assertion for any consumer.
CORRUPT = ("CORRUPT", "KIND_COMPUTE", "BAD")


def consts(path):
    s = re.sub(r"//[^\n]*", "", path.read_text())  # strip comments: doc blocks contain hex too
    out = {}
    for m in re.finditer(r"const\s+([A-Z0-9_]+)\s*:\s*&str\s*=\s*(.+?);", s, re.S):
        name, body = m.group(1), m.group(2)
        hx = "".join(re.findall(r'"([0-9a-fA-F]*)"', body)).lower()
        if hx and len(hx) % 2 == 0:
            out[name] = hx
    if not out:
        sys.exit(f"FAIL: extracted 0 constants from {path} — the parser broke, the file did not")
    return out


def build():
    doc = json.loads(OUT.read_text()) if OUT.exists() else {}
    header = {k: doc[k] for k in ("format", "suite", "generated_by", "methodology", "generated_at")
              if k in doc}
    header["vectors"], header["corruption_controls"] = {}, {}
    total = 0
    for src, path in SOURCES.items():
        for name, hx in sorted(consts(path).items()):
            key = "corruption_controls" if any(c in name.upper() for c in CORRUPT) else "vectors"
            header[key][f"{src}/{name}"] = {"hex": hx, "bytes": len(hx) // 2}
            total += 1
    if total < 15:
        sys.exit(f"FAIL: only {total} vectors extracted, expected >= 15 — coverage floor")
    return header


if __name__ == "__main__":
    fresh = json.dumps(build(), indent=2) + "\n"
    if "--check" in sys.argv:
        if not OUT.exists() or OUT.read_text() != fresh:
            sys.exit(f"FAIL: {OUT.name} is stale — re-run without --check")
        print(f"OK: {OUT.name} matches the Rust corpora")
    else:
        OUT.write_text(fresh)
        print(f"wrote {OUT.relative_to(ROOT)}")
