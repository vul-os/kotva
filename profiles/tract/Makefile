# TRACT specification build & checks.

.PHONY: lint lint-strict pdf check coverage

## lint — internal-consistency checks over the spec (see tools/lint.py).
## Every check exists because a real contradiction survived human review.
lint:
	@python3 tools/lint.py

## lint-strict — as above, but warnings also fail. Use before a release tag.
lint-strict:
	@python3 tools/lint.py --warn-as-error

## pdf — markdown -> single HTML -> PDF via the installed Chrome. No LaTeX.
## First run: (cd build && npm install)
pdf:
	@cd build && node build.mjs

## coverage — reports how many conformance cases are planned vs. runnable.
## Honest by design: conformance/SUITE.md is a case catalogue with zero vectors
## (§16 is not yet normative — see conformance/README.md), so this counts planned
## rows and reports 0 runnable rather than erroring or inventing a percentage.
coverage:
	@planned=$$(grep -c '^| TRACT-' conformance/SUITE.md 2>/dev/null || echo 0); \
	echo "conformance: $$planned case(s) planned, 0 runnable (no vectors until §16 is normative — see conformance/README.md)"

check: lint
