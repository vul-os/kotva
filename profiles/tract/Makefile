# TRACT specification build & checks.

.PHONY: lint lint-strict pdf check

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

check: lint
