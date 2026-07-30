# DMTAP specification build & checks.

.PHONY: lint lint-strict coverage conformance gates pdf check

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

## gates — self-test the reusable product gates this repo SHIPS (tools/gates/).
## The substrate contains no product, so what is verified here is that each gate
## still FAILS on a planted violation and still exits non-zero when it cannot
## check: a copied gate that has gone inert reports a pass nobody earned
## (substrate/SOVEREIGNTY.md §5.1). Needs cargo for the Rust control fixtures —
## and says so with a non-zero exit rather than skipping, which is why it is not
## folded into the toolchain-free `check`.
gates:
	@sh tools/gates/no-broker-dep.sh --selftest

pdf:
	@$(MAKE) -C build 2>/dev/null || echo "see README for the PDF build"

check: lint conformance
