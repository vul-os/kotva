# DMTAP specification build & checks.

.PHONY: lint lint-strict coverage conformance pdf check

## lint — internal-consistency checks over the spec (see tools/lint.py).
## Every check exists because a real contradiction survived human review.
lint:
	@python3 tools/lint.py

## lint-strict — as above, but warnings also fail. Use before a release tag.
lint-strict:
	@python3 tools/lint.py --warn-as-error

## coverage — normative coverage: MUSTs with no conformance case citing their
## clause. A MUST nothing tests reads as a requirement and behaves as a
## suggestion (§10.3: the suite IS the definition of compatibility).
coverage:
	@python3 tools/lint.py --coverage

## conformance — execute the suite catalog (see tools/conformance.py): runs the
## self-contained cases, binds every vectored case to a committed vector, and
## names out loud every case class it did NOT run. `lint` checks the catalog
## against the prose; this checks it against the bytes.
conformance:
	@python3 tools/conformance.py

pdf:
	@$(MAKE) -C build 2>/dev/null || echo "see README for the PDF build"

check: lint conformance
