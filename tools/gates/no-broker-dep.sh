#!/bin/sh
# no-broker-dep.sh — the R-SOV-1 gate: a product's DEFAULT build and startup path
# MUST NOT acquire a hard dependency on a reachability broker (substrate/SOVEREIGNTY.md).
#
# WHY THIS IS A SCRIPT AND NOT A PARAGRAPH
# The suite has learned twice that a copied template propagates where a specification
# does not: substrate/ADOPTION.md records the same ordered-domain decode defect found
# four times in four languages, each invisible to that repo's own tests. "Do not depend
# on the broker" is exactly that shape of rule — everyone agrees with it, and a single
# `use ephor_client::...` in a startup path violates it silently. So the rule ships as
# something a product can LIFT: copy this file into <product>/tools/gates/, set the two
# environment variables at the top of its CI step, and the rule is enforced on every push
# instead of asserted in a README.
#
# WHAT IT CHECKS (three checks; see substrate/SOVEREIGNTY.md §5 for the shape)
#   C-DEP    structural — the DEFAULT-feature dependency closure must not contain the
#            broker. This is the check that cannot be argued with: it reads the resolved
#            graph the toolchain itself produces, not the source text.
#   C-START  textual — the broker MUST NOT be named anywhere outside a declared seam
#            directory. A default endpoint, a hostname constant, or an import in a startup
#            path is a violation even when the dependency is technically optional, because
#            what R-SOV-1 forbids is the DEFAULT PATH reaching for a broker.
#   C-SEAM   fail-closed bookkeeping — if a seam is declared, it must exist and it must be
#            off by default (an undeclared or default-on seam is not a seam, it is the
#            dependency wearing a hat).
#
# LANGUAGE HONESTY
# C-DEP's mechanics are per-ecosystem and there is no universal spelling. Rust and Go are
# implemented here and are the worked examples; node/python are implemented for the common
# case. An ecosystem this script does not recognise EXITS 2 (fail closed) rather than
# reporting a pass it did not earn — see substrate/SOVEREIGNTY.md §5.2 for the table of
# mechanics to add.
#
# CONFIGURATION (environment)
#   BROKER_RE     POSIX ERE, case-insensitive, naming the broker.
#                 Default: pier-|vul-os/pier|ephor|vulos-relayd
#   SEAM_PATHS    Space-separated repo-relative path prefixes where naming the broker is
#                 permitted — the optional provider implementation and its tests. Empty by
#                 default, which is the strictest and correct setting for a product that has
#                 no broker integration at all. A product that DOES have a seam also lists
#                 its manifest/lockfile here (Cargo.toml, Cargo.lock, go.sum, package.json):
#                 an `optional = true` dependency legitimately names the broker there, and
#                 C-DEP already covers those files structurally, which is the stronger check.
#   SEAM_FLAG     REQUIRED when SEAM_PATHS is non-empty: the cargo feature / go build tag /
#                 npm optional-dependency name that gates the seam. Must not be on by default.
#   .no-broker-dep-self   A repo-ROOT file (not an env var — this must be committed and
#                 reviewable, not something a CI step can quietly export) that scopes a repo
#                 OUT of R-SOV-1 entirely, for exactly one reason: the repo IS the broker
#                 BROKER_RE names. R-SOV-1 asks "does this PRODUCT depend on the broker?" —
#                 applied to the broker's own reference implementation the question is
#                 malformed, not merely satisfied: every crate/module path under a checkout
#                 literally named after the broker (e.g. `.../ephor/crates/admin`) will match
#                 BROKER_RE by construction, and every source file legitimately says "Ephor"
#                 in the sense of naming itself, not depending on anything. That is not 15 or
#                 40 judgment calls, it is ~300 structural false positives from one repo being
#                 what the pattern matches, and no per-file marker scales to that — marking
#                 every file a broker's own source repo touches would either bury the real
#                 escape hatch's accountability under bulk use or (worse) train reviewers to
#                 rubber-stamp a wall of markers, which is the exact failure this whole
#                 mechanism exists to prevent.
#                 The file must contain the marker `no-broker-dep:self-broker` followed by a
#                 stated reason, same accountability shape as `:allow-file` below. A present
#                 file with NO reason is NOT honoured — it is reported via `cannot()` and the
#                 gate proceeds to scan normally, because an unreasoned repo-wide exemption
#                 is not accountable and must not silently widen. When honoured, the gate
#                 prints why and exits 0 having run ZERO of the 3 checks — a deliberate,
#                 narrow, auditable exception to "never exits 0 by doing nothing" (see EXIT
#                 STATUS below), scoped to exactly this one file and nothing else. A repo
#                 that is NOT the broker gains nothing from creating this file dishonestly:
#                 the file is committed, greppable, and the reason is printed on every run,
#                 so a false self-declaration is exactly as visible as an unmarked violation
#                 would have been.
#   DOC_PATHS     Space-separated path prefixes that are prose, not a build or startup path
#                 (docs/, README.md, CHANGELOG.md). Naming the broker there is allowed:
#                 documenting "you may plug a broker in here" is the point. The default set
#                 also carries GOVERNANCE.md, SECURITY.md, CONTRIBUTING.md and ROADMAP.md —
#                 root-level governance/policy prose that, like README/CHANGELOG/LICENSE
#                 already here, structurally cannot BE a dependency edge: no toolchain reads
#                 a manifest out of a GOVERNANCE.md. Naming the broker there to assert its
#                 absence ("no hard dependency on Ephor") is the file doing its job, and the
#                 gate was failing that job — a real correctness bug, not a laxer gate: a
#                 GENUINE dependency is still caught regardless, by C-DEP reading the actual
#                 resolved graph and by C-START reading every other file. This list stays
#                 SMALL and path-based on purpose — everything else (SPEC.md, a whitepaper,
#                 a release template, a Cargo.toml comment, a test fixture, a CI step) still
#                 requires the explicit :allow-file marker below, stated and printed, because
#                 those files are close enough to the build/startup surface that "it's just
#                 prose" is a judgment call a human should make once, not a blanket the gate
#                 assumes forever.
#
# EXIT STATUS — and it never exits 0 by doing nothing, with exactly one stated exception
#   (a repo whose committed `.no-broker-dep-self` truthfully declares it IS the broker BROKER_RE
#   names — see CONFIGURATION above. Nothing else may exit 0 without all 3 checks having run.)
#   0  every check ran and passed
#   1  a violation was found (the default path depends on, or names, the broker)
#   2  the gate COULD NOT CHECK: unknown ecosystem, toolchain missing or failing, nothing
#      scanned, a declared seam that does not exist, or an inert self-control. A gate that
#      cannot check must not be indistinguishable from a gate that passed. `go test` throws
#      a passing package's stderr away without -v, so a "loud skip" is invisible in exactly
#      the run that matters; the only skip this script has is a non-zero exit.
#
# ALL THREE CHECKS ALWAYS RUN, and a violation outranks an unverifiable check (exit 1 beats
# exit 2). An earlier revision of this script exited 2 the moment any check could not run,
# which HID a real C-DEP violation behind an unrelated stale seam path — found by running the
# gate against a Go fixture while writing it. "Cannot check" must never suppress "did check,
# and it is broken".
#
# USAGE
#   tools/gates/no-broker-dep.sh [PRODUCT_ROOT]      # gate a product (default: .)
#   tools/gates/no-broker-dep.sh --selftest          # prove the gate still has teeth

set -u

# The broker was renamed ephor -> pier. `pier` is NOT safe as a bare alternative here: this is
# `grep -Ei` with no word boundaries (see the closure scan and the doc scan below), so a bare
# `pier` would flag "happier", "copier", "occupier" in ordinary prose and the gate would cry wolf
# until someone silenced it. Match the shapes the broker actually takes instead:
#   pier-        every crate is namespaced pier-gateway, pier-relay, pier-billing, ...
#   vul-os/pier  the repo URL and the Go module path github.com/vul-os/pier
#   ephor        RETAINED deliberately: a stale dependency on the pre-rename name is exactly the
#                thing this gate should still catch, and dropping it would quietly narrow the gate
#                at the moment it most needs to be wide.
BROKER_RE=${BROKER_RE:-'pier-|vul-os/pier|ephor|vulos-relayd'}
SEAM_PATHS=${SEAM_PATHS:-}
SEAM_FLAG=${SEAM_FLAG:-}
# `site/` is in the default set because every product in this suite mirrors `docs/` into
# `site/docs/` for its published mini-site. Omitting it made the gate flag a heading like
# "## Exposing via your own Ephor (optional)" — prose describing the PERMITTED optional path —
# as an R-SOV-1 violation. A doc that marks the broker optional is the contract being followed,
# not broken; what R-SOV-1a forbids is prominence in getting-started, which a grep cannot judge
# and a human must.
DOC_PATHS=${DOC_PATHS:-'docs/ site/ README.md CHANGELOG.md LICENSE.md GOVERNANCE.md SECURITY.md CONTRIBUTING.md ROADMAP.md'}

# Split so this script's own source does not contain the literal marker it searches for —
# otherwise the gate would exempt itself by accident rather than by the explicit rule above.
_ALLOW_SUFFIX=':allow-file'

# Build directories and vendored trees are excluded from the TEXT scan only. They are not
# excluded from C-DEP, which reads the resolved graph and is where a vendored broker would
# show up anyway.
# Matched by NAME, at any depth — not by top-level path. A `-path ./node_modules` test only
# ever matches a prune dir sitting in the root, which is not where they mostly live: gitstate
# has web/node_modules, apps/desktop/src-tauri/target and crates/gitstate-sync/target, and a
# path-matched prune walked all three. That cost 28,731 scanned files and ~16 minutes for one
# invocation, and it was a correctness bug before it was a performance one — a vendored package
# or a build fingerprint containing the broker's name would have produced a spurious FAIL
# against a product that does not depend on it.
#
# `dist` and `build` were already here; the fleet's own .gitignores were audited (2026-07-30)
# for the other spellings the same convention actually uses — dist-lib/dist-office/dist-ssr/
# dist-dev/dist-main/dist-demo/dev-dist all appear, gitignored, across multiple products (e.g.
# diwan ships `build:lib`/`build:office` npm scripts whose output is `dist-lib/`/`dist-office/`,
# both gitignored — present on a machine that built them, absent from a clean checkout). A
# minified bundle is DERIVED from source already scanned by this same walk; scanning the
# rebuild too only re-reads the same text through unstable minified names, at the cost of a
# spurious FAIL against a product with no source-level mention at all. A vendored broker
# client would still show up structurally in C-DEP either way, same as node_modules/target.
PRUNE_NAMES='.git target node_modules vendor dist build .venv __pycache__ dist-lib dist-office dist-ssr dist-dev dist-main dist-demo dev-dist'

# Emits the `find` prune expression, name-matched so it applies at ANY depth.
prune_expr() {
	_e=''
	for _n in $PRUNE_NAMES; do _e="$_e -name $_n -prune -o"; done
	printf '%s' "$_e"
}

# ── file enumeration: prefer git so gitignored, regenerated build output is
# NEVER scanned; PRUNE_NAMES above is kept only as the fallback outside a git
# checkout ──────────────────────────────────────────────────────────────────
# THE DEFECT THIS REPLACES: PRUNE_NAMES is a fixed list of DIRECTORY NAMES, and a
# generated tree that does not happen to be named `dist`/`build`/one of the
# `dist-*` spellings sails straight through it. vulos-static's own
# `scripts/sync-docs.mjs` and `scripts/collect-repo-landings.mjs` regenerate
# `content/docs/`, `public/projects/*/` and `.astro/` — none of those names are in
# PRUNE_NAMES, both scripts' own docstrings say "THE OUTPUT IS NOT COMMITTED", and
# `.gitignore` already lists all three. The gate scanned them anyway: 84
# violations, 100% of them inside those three gitignored trees, in files that
# have never existed in a single commit and do not exist in a fresh clone.
# Marking them is a no-op that makes the bug worse to explain, not better: git
# will not persist a `:allow-file` comment written into an ignored file, so
# there is no annotation that can ever land. This is the SAME shape as DEFECT 1
# above (PRUNE_NAMES was blind to nested `node_modules`/`target`) one level up —
# a fixed name list is blind to *every* spelling nobody thought to enumerate,
# and a product's own build tooling invents new ones over time.
#
# THE FIX: ask git, which already has the authoritative answer for exactly this
# question. `git ls-files --cached --others --exclude-standard` is tracked ∪
# (untracked-but-not-gitignored) — precisely "what a fresh clone would contain,
# plus whatever a human has added but not yet committed", which is the correct
# subject for a gate whose job is "does the PRODUCT depend on the broker", not
# "does whatever happens to be sitting on this disk right now mention it". This
# is the same technique used the same day to fix llmux's check-web-dist.sh.
#
# THIS CANNOT REPLACE PRUNE_NAMES OUTRIGHT: the gate is run against arbitrary
# paths (substrate/SOVEREIGNTY.md §5), including a bare extracted tarball or a
# directory that was never `git init`'d at all, where `git ls-files` has nothing
# to answer from. Falling back SILENTLY there would be the mirror-image of the
# bug just fixed: a directory with no `.gitignore` to consult would have every
# regenerated artifact on disk scanned as if it were source, and the operator
# would have no way to tell "this passed because it's clean" from "this passed
# because git was unavailable and nothing was pruned". So the fallback is
# announced LOUDLY, once, from run_gate — see detect_git_enum() below — with an
# honest "CANNOT use git here" rather than a silent downgrade to the old
# behaviour.
#
# WHAT THIS DOES NOT CHANGE: a file that IS tracked is scanned exactly as
# before, git-enumerated or not — `--cached` lists every tracked file
# regardless of what any `.gitignore` pattern would otherwise match, so a
# tracked file living under a path shaped like `dist/` or `vendor/` is neither
# newly exempted nor newly pruned by this change. Only genuinely gitignored,
# untracked content stops being scanned — which is the entire point: it is not
# in the repository, so it cannot be a repository's dependency.
GIT_ENUM=unknown # yes | no, decided once per run_gate() by detect_git_enum()

detect_git_enum() {
	if ! command -v git >/dev/null 2>&1; then
		GIT_ENUM=no
		say "  file walk: CANNOT use git here — no git on PATH. Falling back to the static"
		say "             PRUNE_NAMES walk (find): this run WILL scan whatever regenerated or"
		say "             gitignored build output happens to sit on disk right now, because without"
		say "             git there is no .gitignore to consult and no tracked/untracked distinction"
		say "             to draw. Install git for the stronger guarantee."
		return 0
	fi
	if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
		GIT_ENUM=no
		say "  file walk: CANNOT use git here — $(pwd) is not inside a git working tree. Falling"
		say "             back to the static PRUNE_NAMES walk (find), with the same caveat: any"
		say "             regenerated/gitignored build output already sitting on disk WILL be"
		say "             scanned, because there is no repository here to ask what is ignored."
		return 0
	fi
	GIT_ENUM=yes
	say "  file walk: git ls-files --cached --others --exclude-standard (tracked + untracked-but-"
	say "             not-ignored; gitignored, regenerated build output is never scanned)"
}

# Emits every file this run should consider, one per line, "./"-prefixed exactly
# like the find-based walk has always produced — so every downstream consumer
# (is_exempt's leading-"./" strip, check_dep_closure's dirname-by-sed) is
# unchanged regardless of which branch fired.
enumerate_files() {
	if [ "$GIT_ENUM" = yes ]; then
		git ls-files -z --cached --others --exclude-standard 2>/dev/null |
			tr '\0' '\n' | sed 's#^#./#'
	else
		# shellcheck disable=SC2086 # word splitting of the prune list is intended
		find . $(prune_expr) -type f -print 2>/dev/null
	fi
}

say() { printf '%s\n' "$*"; }
fail() { printf 'VIOLATION  %s\n' "$*"; violations=$((violations + 1)); }
# `cannot` records an unverifiable check and RETURNS — the remaining checks still run, and the
# final exit code is decided in one place (run_gate) so a violation is never masked.
cannot() { printf 'CANNOT CHECK  %s\n' "$*"; unverifiable=$((unverifiable + 1)); }
# `die` is reserved for a failure that makes the whole run meaningless (a root that does not
# exist, a broken selftest harness).
die() { printf 'CANNOT CHECK  %s\n' "$*" >&2; exit 2; }

# ── path classification ───────────────────────────────────────────────────────────────
# A file is exempt from the TEXT scan if it sits under a declared seam or a declared doc
# path, OR carries an explicit :allow-file marker. A file can be exempt via MORE THAN ONE of
# these at once — e.g. a file under docs/ that also carries its own marker for a distinct,
# more specific reason — and the mechanisms carry different semantics: SEAM_PATHS/DOC_PATHS
# is a coarse, pre-declared, configuration-level exemption; the marker is a deliberate,
# per-file, human-authored justification that this gate promises to print (see below). AN
# EARLIER REVISION short-circuited on whichever matched first: if a SEAM_PATHS/DOC_PATHS
# prefix matched, the function returned before ever reaching the marker check, so a file's
# specific, human-written reason was silently swallowed whenever a broader path rule already
# covered it. The boolean verdict (exempt or not) never changed — SEAM_PATHS/DOC_PATHS
# membership alone is sufficient — but the marker's accountability promise ("every exemption
# via this mechanism is printed with its reason on every run") was broken for exactly the
# files where a human bothered to write one. Proven by fixture: the identical marker on the
# identical prose printed nothing under docs/ and printed its reason in full the moment the
# same file was moved out from under DOC_PATHS. The marker is now checked UNCONDITIONALLY,
# never short-circuited by the coarser path match, so its reason is always surfaced when
# present — a pure superset of what used to print, not a change to which files are exempt.
is_exempt() {
	_p=${1#./}
	# This gate quotes the broker in its own explanatory comments, so it matches itself once
	# copied into a product. A gate that fails every adopter on its own documentation is worse
	# than no gate: the first fix anyone reaches for is deleting the gate.
	case $1 in */no-broker-dep.sh | no-broker-dep.sh) return 0 ;; esac

	_exempt=1

	# A file may name the broker in order to ASSERT ITS ABSENCE. gitstate's CI carries a step
	# called "Assert no Ephor dependency anywhere" whose grep pattern necessarily spells the
	# broker's name — and this gate flagged it, which is the gate failing a repo for enforcing
	# the very rule the gate exists to enforce. The available workarounds were both bad:
	# exempting all of .github/ would blind the gate to a workflow that genuinely deploys a
	# broker, and deleting the assertion would remove a real check to satisfy a false one.
	#
	# So a file may opt out explicitly, and the cost of doing so is that it must say why in the
	# same breath. Every exemption via this marker is PRINTED with its reason on every run it
	# is present, unconditionally — see the header comment above for why this can no longer be
	# short-circuited by a SEAM_PATHS/DOC_PATHS match reached first. Grep-hostile spelling of
	# the marker keeps this very comment from matching it.
	if grep -Iq -e "no-broker-dep""$_ALLOW_SUFFIX" "$1" 2>/dev/null; then
		_reason=$(grep -Ihm1 -e "no-broker-dep""$_ALLOW_SUFFIX" "$1" 2>/dev/null |
			sed "s/.*no-broker-dep$_ALLOW_SUFFIX[: ]*//" | cut -c1-100)
		[ -n "$_reason" ] || _reason='(NO REASON GIVEN — this exemption is not accountable)'
		printf '  EXEMPT-FILE  %s — %s\n' "$_p" "$_reason"
		_file_exemptions=$((_file_exemptions + 1))
		_exempt=0
	fi

	for _pre in $SEAM_PATHS $DOC_PATHS; do
		case $_p in "${_pre%/}"/* | "$_pre") _exempt=0 ;; esac
	done

	return $_exempt
}

# ── C-DEP: the default-feature dependency closure ─────────────────────────────────────
# Each ecosystem answers ONE question: "with default features/tags, what does the build
# actually pull in?" The command must be the toolchain's own resolver — a manifest grep
# cannot see a transitive dependency, which is precisely how a broker arrives.
#
# C-DEP runs once PER MANIFEST, not once per repo. The earlier if/elif chain checked whichever
# ecosystem happened to own the root manifest and silently ignored every other one — and said
# nothing, so the output read as a clean pass. pango is the case that exposed it: package.json
# at the root, go.mod in backend/, and the entire Go closure — the actual product — was never
# read. A gate that quietly declines to look is worse than one that fails, because the operator
# has no way to tell the difference.
#
# The SAME bug survived one level down: check_dep_closure_one() (below) used to be its own
# if/elif chain OVER A SINGLE DIRECTORY — Cargo.toml, else go.mod, else package.json, else
# pyproject.toml/requirements.txt — so a directory holding MORE THAN ONE manifest only ever
# had the first one examined, silently. diwan is the live case (go.mod AND package.json at
# the repo root — only Go was ever read); wede, molao, cackle and evermesh all carry more
# than one manifest at a checked directory too, and a Rust-crate-with-a-JS-binding or
# Go-service-with-a-JS-frontend shape is ordinary, not rare. check_dep_closure_one() now
# checks every manifest present in its directory independently — see its own header comment.
check_dep_closure() {
	_root=$(pwd)
	_manifests=$(enumerate_files |
		grep -E '/(Cargo\.toml|go\.mod|package\.json|pyproject\.toml|requirements\.txt)$' |
		sed 's|/[^/]*$||' | sort -u)

	if [ -z "$_manifests" ]; then
		cannot "C-DEP  no recognised manifest (Cargo.toml / go.mod / package.json / pyproject.toml) anywhere under $_root — add this ecosystem's mechanics per substrate/SOVEREIGNTY.md §5.2 instead of skipping"
		return 0
	fi

	# A Cargo workspace member resolves through its workspace root, so checking it separately
	# re-reads the same graph. Skip members; the root already covers them.
	_rust_ws=no
	if [ -f Cargo.toml ] && grep -q '^\[workspace\]' Cargo.toml 2>/dev/null; then _rust_ws=yes; fi

	_oldifs=$IFS
	IFS='
'
	for _d in $_manifests; do
		IFS=$_oldifs
		# A workspace MEMBER's Cargo.toml resolves through the workspace root, so re-running
		# cargo tree here would just re-read the same graph — skip only the rust half of this
		# directory's check, never the whole directory. Skipping the whole directory was the
		# earlier bug: a workspace member that ALSO carries a go.mod or package.json (a Rust
		# crate with a JS/Go binding is an ordinary shape in this suite) had that other
		# ecosystem's closure silently never read, because it happened to share a directory
		# with a Cargo.toml the outer loop chose to skip entirely.
		_skip_rust=no
		if [ "$_rust_ws" = yes ] && [ "$_d" != "." ] && [ -f "$_d/Cargo.toml" ] &&
			! grep -q '^\[workspace\]' "$_d/Cargo.toml" 2>/dev/null; then
			_skip_rust=yes
		fi
		# NOT a subshell: the violation counter must survive back into run_gate.
		cd "$_d" || {
			cannot "C-DEP  cannot enter $_d — its dependency closure was NOT read"
			IFS='
'
			continue
		}
		check_dep_closure_one "$_d" "$_skip_rust"
		cd "$_root" || die "lost the repo root while walking manifests"
		IFS='
'
	done
	IFS=$_oldifs
}

# Reads ONE manifest directory's closure — EVERY ecosystem manifest present there, not just
# the first one found. THE DEFECT THIS REPLACES: this function used to be its own if/elif
# chain (Cargo.toml, else go.mod, else package.json, else pyproject.toml/requirements.txt),
# so a directory holding MORE THAN ONE manifest only ever got the first one checked — no
# violation reported for the rest, and no cannot-check either, so the run read as a clean
# pass. This is the SAME bug check_dep_closure()'s header comment already describes fixing
# once (root-manifest-only -> per-directory); the elif simply moved down one level, from
# "once per repo" to "once per directory". The fix is the same shape at the smaller scope:
# every ecosystem manifest present in $_where is checked independently and reported under
# its own name — a hit in any one is a violation, a closure that cannot be read in any one is
# its own cannot-check, and neither may be silently absorbed by whichever ecosystem happened
# to be tested first.
#
# $2 (_skip_rust) is "yes" only when $_where is a Cargo workspace MEMBER whose root already
# resolved this graph (see check_dep_closure) — that skip is legitimate (re-reading the same
# graph twice proves nothing new), but it must skip only the rust half of this directory's
# check, never the other manifests that might sit beside that Cargo.toml. Skipping the whole
# directory was an instance of the same bug wearing a different hat.
check_dep_closure_one() {
	_where=$1
	_skip_rust=${2:-no}
	_dep_found=0

	if [ -f Cargo.toml ] && [ "$_skip_rust" = yes ]; then
		_dep_found=1
		say "  C-DEP    rust ($_where): workspace member — already covered via the workspace root's cargo tree"
	elif [ -f Cargo.toml ]; then
		_dep_found=1
		check_dep_rust "$_where"
	fi

	if [ -f go.mod ]; then
		_dep_found=1
		check_dep_go "$_where"
	fi

	if [ -f package.json ]; then
		_dep_found=1
		check_dep_node "$_where"
	fi

	if [ -f pyproject.toml ] || [ -f requirements.txt ]; then
		_dep_found=1
		check_dep_python "$_where"
	fi

	if [ "$_dep_found" = 0 ]; then
		cannot "C-DEP  no recognised manifest (Cargo.toml / go.mod / package.json / pyproject.toml / requirements.txt) in $(pwd) — add this ecosystem's mechanics per substrate/SOVEREIGNTY.md §5.2 instead of skipping"
	fi
}

# ── one resolver function per ecosystem, each reporting through report_dep_closure ────────
# `cargo tree` resolves with DEFAULT features, which is exactly R-SOV-1's subject. -e normal
# drops build/dev edges: a dev-dependency on a broker client is a test fixture, not a default
# runtime path.
check_dep_rust() {
	_where=$1
	if ! command -v cargo >/dev/null 2>&1; then
		cannot "C-DEP  rust ($_where): Cargo.toml present but no cargo on PATH — the dependency graph was NOT read"
		return 0
	fi
	_closure=$(cargo tree -e normal --prefix none --offline 2>/dev/null) ||
		_closure=$(cargo tree -e normal --prefix none 2>/dev/null) || {
		cannot "C-DEP  rust ($_where): cargo tree failed — the dependency graph was NOT read (do not read this as a pass)"
		return 0
	}
	report_dep_closure rust "$_where" "$_closure"
}

# `go list -deps ./...` is the import closure for the DEFAULT build tags. A seam behind
# `//go:build broker` is absent here unless it is on by default, which is the property
# R-SOV-1 wants and the reason this is the right command.
check_dep_go() {
	_where=$1
	if ! command -v go >/dev/null 2>&1; then
		cannot "C-DEP  go ($_where): go.mod present but no go on PATH — the import closure was NOT read"
		return 0
	fi
	_closure=$(go list -deps ./... 2>/dev/null) || {
		cannot "C-DEP  go ($_where): go list -deps failed — the import closure was NOT read"
		return 0
	}
	report_dep_closure go "$_where" "$_closure"
}

check_dep_node() {
	_where=$1
	if ! command -v npm >/dev/null 2>&1; then
		cannot "C-DEP  node ($_where): package.json present but no npm on PATH — the dependency graph was NOT read"
		return 0
	fi
	_closure=$(npm ls --omit=dev --all --parseable 2>/dev/null) || {
		cannot "C-DEP  node ($_where): npm ls failed — the dependency graph was NOT read (do not read this as a pass; a pre-existing broken/extraneous node_modules reports this way, which is an environment problem, not evidence of a clean tree)"
		return 0
	}
	report_dep_closure node "$_where" "$_closure"
}

check_dep_python() {
	_where=$1
	# No resolver is assumed present, so this is the declared set, not the closure —
	# stated as reduced assurance rather than quietly passed off as equivalent.
	_closure=$(cat pyproject.toml requirements.txt 2>/dev/null)
	say "  note: python C-DEP reads DECLARED dependencies, not a resolved closure ($_where) —"
	say "        a transitive broker dependency is NOT visible to this check."
	report_dep_closure python "$_where" "$_closure"
}

# Shared tail for every ecosystem above: an empty closure is cannot-check, a hit is a
# violation, otherwise the entries-examined count C-DEP has always printed.
report_dep_closure() { # $1=ecosystem $2=where $3=closure text
	_eco=$1
	_where=$2
	_closure=$3

	if [ -z "$_closure" ]; then
		cannot "C-DEP  $_eco ($_where): dependency closure came back EMPTY — the command ran but produced nothing to check"
		return 0
	fi

	_hits=$(printf '%s\n' "$_closure" | grep -Ei -- "$BROKER_RE" || true)
	_n=$(printf '%s' "$_closure" | grep -c '' || true)
	if [ -n "$_hits" ]; then
		printf '%s\n' "$_hits" | while IFS= read -r h; do
			[ -n "$h" ] && fail "C-DEP  $_eco ($_where): default-feature dependency closure contains the broker: $h"
		done
		# The subshell above (the pipe into `while read`) cannot increment the parent's
		# counter; record the fact here.
		violations=$((violations + 1))
	fi
	say "  C-DEP    $_eco ($_where): examined $_n entries of the default-feature dependency closure"
}

# ── C-START: the broker may not be named outside a declared seam ───────────────────────
check_startup_text() {
	files=$(enumerate_files)
	if [ -z "$files" ]; then
		cannot "C-START  scanned NOTHING — the file walk found no files under $(pwd)"
		return 0
	fi

	scanned=0
	exempted=0
	# Split on NEWLINE only, so a path containing a space is one path and not two — a
	# gate that silently stops scanning "src/my app/" is a gate with a hole in it.
	_oldifs=$IFS
	IFS='
'
	for f in $files; do
		IFS=$_oldifs
		if is_exempt "$f"; then
			exempted=$((exempted + 1))
			IFS='
'
			continue
		fi
		scanned=$((scanned + 1))
		# -I skips binaries; a match in a source file, a default config, a Dockerfile or a
		# systemd unit is a violation, because all four are the startup path.
		hit=$(grep -Iin -Ee "$BROKER_RE" "$f" 2>/dev/null | head -3 || true)
		if [ -n "$hit" ]; then
			printf '%s\n' "$hit" | while IFS= read -r line; do
				[ -n "$line" ] && fail "C-START  ${f#./}:$line"
			done
			violations=$((violations + 1))
		fi
		IFS='
'
	done
	IFS=$_oldifs
	if [ "$scanned" -eq 0 ]; then
		cannot "C-START  scanned 0 non-exempt files — every path is exempt, so this check verified nothing"
		return 0
	fi
	say "  C-START  scanned $scanned files ($exempted exempt under SEAM_PATHS/DOC_PATHS)"
}

# ── C-SEAM: a declared seam must exist and must be off by default ─────────────────────
# THE DEFECT THIS REPLACES: this function used to be an if/elif over the REPO ROOT's own
# manifest presence (Cargo.toml, else go.mod, else "cannot verify") — the exact shape
# check_dep_closure_one() was fixed for, one call site over, and proven wrong by fixture in
# BOTH directions:
#   (a) FALSE NEGATIVE — a genuinely correct, off-by-default Go build-tag seam
#       (SEAM_PATHS="reach go.mod") sitting beside an unrelated Cargo.toml at root is never
#       examined: the Cargo.toml branch fires first, greps Cargo.toml's [features] for a flag
#       name that was never a cargo feature, and prints an identical "clean" message whether
#       the real Go seam is correct or was mutated to compile the broker unconditionally.
#   (b) FALSE POSITIVE, worse — a legitimate npm-based seam (SEAM_PATHS="package.json") is
#       rejected outright when an UNRELATED go.mod also happens to sit at root: go.mod wins
#       the elif, and its mechanic (grep for a `//go:build` tag) is applied to package.json,
#       which structurally cannot contain that syntax, so a seam that was never broken FAILS
#       the gate. Proven with C-DEP(go) and C-DEP(node) BOTH clean (the optional dependency
#       was never even installed) — check_seam() was the SOLE source of the violation.
# A Tauri app (Rust core + npm frontend) or a Go service with an npm binding is an ordinary
# shape in this suite, so a polyglot root is not an edge case.
#
# THE FIX: classify by what SEAM_PATHS itself NAMES, not by what else happens to live at the
# repo root. SEAM_PATHS is documented above to include the seam's own manifest/lockfile
# (Cargo.toml/Cargo.lock, go.mod/go.sum, package.json/lockfiles) — that is the actual subject.
# Every ecosystem the declaration names is checked independently (never elif), which also
# answers the plurality question directly: yes, a repo may declare a seam implemented in more
# than one ecosystem at once (a Rust core AND an npm binding gated by the same logical
# provider, say), and each is verified on its own terms.
check_seam() {
	if [ -z "$SEAM_PATHS" ]; then
		say "  C-SEAM   no seam declared — the strictest posture: the broker is named nowhere"
		return 0
	fi
	if [ -z "$SEAM_FLAG" ]; then
		cannot "C-SEAM   SEAM_PATHS is set but SEAM_FLAG is not: an ungated seam is not a seam. Name the cargo feature / build tag / optional dependency that turns it on."
		return 0
	fi
	_stale=0
	for p in $SEAM_PATHS; do
		if [ ! -e "$p" ]; then
			cannot "C-SEAM   declared seam path '$p' does not exist — a stale allowlist silently widens the exemption"
			_stale=1
		fi
	done
	[ "$_stale" = 0 ] || return 0

	_seam_rust=no
	_seam_go=no
	_seam_npm=no
	_seam_py=no
	for p in $SEAM_PATHS; do
		case $p in *Cargo.toml | *Cargo.lock) _seam_rust=yes ;; esac
		case $p in *go.mod | *go.sum) _seam_go=yes ;; esac
		case $p in *package.json | *package-lock.json | *yarn.lock | *pnpm-lock.yaml) _seam_npm=yes ;; esac
		case $p in *pyproject.toml | *requirements.txt) _seam_py=yes ;; esac
	done
	_seam_matched=no

	# Rust: the seam must not be in the default feature set. C-DEP already covers this
	# structurally (cargo tree resolves default features), so this is a confirmatory,
	# human-readable second check, not the only one.
	if [ "$_seam_rust" = yes ]; then
		_seam_matched=yes
		if [ -f Cargo.toml ]; then
			defaults=$(awk '/^\[features\]/{f=1} f&&/^default *=/{print;exit}' Cargo.toml)
			case $defaults in
			*"$SEAM_FLAG"*) fail "C-SEAM   rust: feature '$SEAM_FLAG' gates the broker seam but is ON by default: $defaults" ;;
			*) say "  C-SEAM   rust: seam '$SEAM_FLAG' is not in Cargo.toml's default feature set" ;;
			esac
		else
			cannot "C-SEAM   rust: SEAM_PATHS names a Cargo manifest/lockfile but no Cargo.toml exists at $(pwd) — the seam declaration does not match the tree"
		fi
	fi

	# Go: a Go seam is a build tag; absence from the default `go list -deps` closure was
	# already proven by C-DEP, so the remaining check is that the declared tag is real.
	if [ "$_seam_go" = yes ]; then
		_seam_matched=yes
		if [ -f go.mod ]; then
			# shellcheck disable=SC2086 # SEAM_PATHS is a space-separated LIST of paths by design
			if grep -rIlE "^//go:build( .*)?\\b$SEAM_FLAG\\b" $SEAM_PATHS >/dev/null 2>&1; then
				say "  C-SEAM   go: seam files carry the '//go:build $SEAM_FLAG' constraint"
			else
				fail "C-SEAM   go: no file under '$SEAM_PATHS' carries a '//go:build $SEAM_FLAG' constraint — the seam is compiled unconditionally"
			fi
		else
			cannot "C-SEAM   go: SEAM_PATHS names a Go manifest/sum but no go.mod exists at $(pwd) — the seam declaration does not match the tree"
		fi
	fi

	# npm/Python: no structural off-by-default mechanic is implemented for either — an
	# `optionalDependencies` entry being present in node_modules (or a declared Python
	# requirement) does not by itself prove the code path is default-off, the same reduced-
	# assurance reasoning check_dep_python already states out loud for C-DEP. Reporting a
	# pass here would be exactly the "reports success it did not earn" failure this whole
	# script exists to avoid — say so and exit unverifiable instead of guessing.
	if [ "$_seam_npm" = yes ]; then
		_seam_matched=yes
		cannot "C-SEAM   npm: SEAM_PATHS names an npm manifest/lockfile for '$SEAM_FLAG' but no structural off-by-default mechanic is implemented — an optionalDependency's presence in node_modules does not by itself prove the code path is default-off; verify by hand per substrate/SOVEREIGNTY.md §5.2 instead of assuming a pass"
	fi
	if [ "$_seam_py" = yes ]; then
		_seam_matched=yes
		cannot "C-SEAM   python: SEAM_PATHS names a Python manifest for '$SEAM_FLAG' but no structural off-by-default mechanic is implemented — verify by hand per substrate/SOVEREIGNTY.md §5.2 instead of assuming a pass"
	fi

	if [ "$_seam_matched" = no ]; then
		cannot "C-SEAM   SEAM_PATHS ($SEAM_PATHS) names no recognised manifest/lockfile (Cargo.toml/.lock, go.mod/.sum, package.json/lockfiles, pyproject.toml/requirements.txt) — cannot identify which ecosystem's off-by-default mechanics to verify"
	fi
}

run_gate() {
	root=${1:-.}
	cd "$root" 2>/dev/null || die "cannot enter '$root'"
	violations=0
	unverifiable=0
	_file_exemptions=0
	say "R-SOV-1 gate (no-broker-dep) — $(pwd)"
	say "  broker pattern: $BROKER_RE"

	# ── repo-level self-declaration: "this repo IS the broker" ──────────────────────────
	# Checked BEFORE any scan runs, and only ever short-circuits when a stated reason is
	# present — see the `.no-broker-dep-self` entry in CONFIGURATION above for why this
	# exists and why it cannot be an env var. An unreasoned file is deliberately NOT
	# honoured: it is surfaced as an unverifiable control and the normal scan still runs,
	# so an empty exemption file cannot silently widen into a real one.
	_self_decl=.no-broker-dep-self
	if [ -f "$_self_decl" ] && grep -q "no-broker-dep:self-broker" "$_self_decl" 2>/dev/null; then
		_self_reason=$(grep -h "no-broker-dep:self-broker" "$_self_decl" 2>/dev/null |
			head -1 | sed "s/.*no-broker-dep:self-broker[: ]*//" | cut -c1-200)
		if [ -n "$_self_reason" ]; then
			say "  OUT OF SCOPE  $_self_decl declares this repo IS the broker named by BROKER_RE."
			say "                R-SOV-1 asks whether a PRODUCT depends on the broker; applied to the"
			say "                broker's own repository the question is malformed, not merely satisfied"
			say "                — every crate/module path here matches BROKER_RE by construction."
			say "                stated reason: $_self_reason"
			say ""
			say "PASS (out of scope)  0 of 3 checks ran, DELIBERATELY: see $_self_decl. This is a narrow,"
			say "      committed, greppable exception to 'never exits 0 by doing nothing', good for"
			say "      exactly one repo-self-reference case, not a template for anything else."
			return 0
		fi
		cannot "REPO-SELF  $_self_decl carries the self-broker marker with NO stated reason — an unreasoned repo-wide exemption is not accountable, so it is IGNORED and this run proceeds as a normal scan"
	fi

	# Decided once per run, before any check reads a file: see the "file enumeration"
	# header comment above prune_expr() for why git is preferred and how the fallback is
	# scoped.
	detect_git_enum

	# All three always run: a check that cannot run must not suppress a check that found
	# something. The exit code is decided once, here.
	check_dep_closure
	check_startup_text
	check_seam
	say ""
	if [ "$violations" -gt 0 ]; then
		say "FAIL  $violations violation(s) found: the default build or startup path depends"
		say "      on, or names, the broker. R-SOV-1 (substrate/SOVEREIGNTY.md §3.1) requires the"
		say "      default path to work with no broker present; a broker may only sit behind a"
		say "      declared, default-off seam whose removal costs reachability from behind NAT and"
		say "      nothing else."
		[ "$unverifiable" -gt 0 ] &&
			say "      ($unverifiable further check(s) could not run — fix those too; they verified nothing.)"
		return 1
	fi
	if [ "$unverifiable" -gt 0 ]; then
		say "NOT VERIFIED  $unverifiable of 3 checks could not run, so this is NOT a pass. Exit 2 is"
		say "      deliberately distinct from exit 0: the checks that did run found nothing, and the"
		say "      ones above found out nothing at all."
		return 2
	fi
	say "PASS  all 3 checks ran: default build and startup path carry no hard broker dependency."
	return 0
}

# ── selftest: positive and negative controls, hermetic ────────────────────────────────
# A check that cannot fail reports success it did not earn (the lesson tools/lint.py's C12
# and C15 were written for, three inert revisions each). So the gate proves, on demand, that
# it still trips on PLANTED violations, still passes a clean tree, and still refuses to call
# an unverifiable configuration a pass.
#
# Controls are per-ecosystem and run for every toolchain PRESENT. At least one must run — a
# selftest with nothing to run exits 2 — and any ecosystem NOT exercised is named out loud as
# NOT VERIFIED, because "the Rust half is fine" is not evidence about the Go mechanics.
selftest() {
	tmp=$(mktemp -d) || die "mktemp failed"
	trap 'rm -rf "$tmp"' EXIT
	self=$(cd "$(dirname "$0")" && pwd)/$(basename "$0")
	rc=0
	ran=0
	controls=0
	expected_controls=0
	unexercised=''

	expect() { # $1 = fixture dir, $2 = expected exit, $3 = what it proves, $4.. = env
		f=$1 want=$2 what=$3
		shift 3
		controls=$((controls + 1))
		out=$(env "$@" sh "$self" "$tmp/$f" 2>&1)
		got=$?
		if [ "$got" != "$want" ]; then
			say "SELFTEST FAIL  '$f': expected exit $want, got $got — $what"
			printf '%s\n' "$out" | sed 's/^/    | /'
			rc=1
		else
			say "SELFTEST ok    '$f' -> exit $got ($what)"
		fi
	}

	# Like expect(), but for a defect whose fix does NOT change the exit code — a reporting
	# hole, not a verdict flip — so the only way to prove it is to inspect the OUTPUT for text
	# that must (or must not) appear. Exit code is still captured and printed on failure for
	# context, but is deliberately NOT asserted here; that is expect()'s job.
	expect_output() { # $1 = fixture dir, $2 = mode(has|lacks), $3 = needle, $4 = what it proves, $5.. = env
		f=$1 mode=$2 needle=$3 what=$4
		shift 4
		controls=$((controls + 1))
		out=$(env "$@" sh "$self" "$tmp/$f" 2>&1)
		case $mode in
		has)
			case $out in
			*"$needle"*)
				say "SELFTEST ok    '$f' -> output contains expected text ($what)"
				;;
			*)
				say "SELFTEST FAIL  '$f': expected output to CONTAIN: $needle — $what"
				printf '%s\n' "$out" | sed 's/^/    | /'
				rc=1
				;;
			esac
			;;
		lacks)
			case $out in
			*"$needle"*)
				say "SELFTEST FAIL  '$f': expected output to NOT contain: $needle — $what"
				printf '%s\n' "$out" | sed 's/^/    | /'
				rc=1
				;;
			*)
				say "SELFTEST ok    '$f' -> output correctly omits text ($what)"
				;;
			esac
			;;
		esac
	}

	# ---- Rust controls -----------------------------------------------------------------
	if command -v cargo >/dev/null 2>&1; then
		ran=$((ran + 1))
		# 8 original + 2 CURRENT-NAME rename controls (see POSITIVE D/E below).
		# 12, not 10: the last two omit BROKER_RE so the compiled-in DEFAULT is exercised.
		expected_controls=$((expected_controls + 12))
		# a vendored "broker client" the fixtures can depend on with no network
		mkdir -p "$tmp/ephor-client/src"
		cat >"$tmp/ephor-client/Cargo.toml" <<-'EOF'
			[package]
			name = "ephor-client"
			version = "0.0.0"
			edition = "2021"
		EOF
		echo 'pub fn dial() {}' >"$tmp/ephor-client/src/lib.rs"

		mkrust() { # $1 = fixture name
			mkdir -p "$tmp/$1/src"
			cat >"$tmp/$1/Cargo.toml" <<-EOF
				[package]
				name = "$1"
				version = "0.0.0"
				edition = "2021"

				[dependencies]
			EOF
			echo 'fn main() {}' >"$tmp/$1/src/main.rs"
		}

		# NEGATIVE control — a clean product.
		mkrust rs_clean
		printf 'const DEFAULT_PEER: Option<&str> = None;\nfn main() {}\n' >"$tmp/rs_clean/src/main.rs"

		# POSITIVE A — a hard dependency in the default feature set.
		mkrust rs_dep
		printf '\nephor-client = { path = "../ephor-client" }\n' >>"$tmp/rs_dep/Cargo.toml"
		printf 'fn main() { ephor_client::dial(); }\n' >"$tmp/rs_dep/src/main.rs"

		# POSITIVE B — no dependency at all, but a default broker endpoint in the startup
		# path. This is the variant a dependency-graph check alone cannot see.
		mkrust rs_default
		printf 'const BROKER: &str = "https://rendezvous.ephor.example";\nfn main() { let _ = BROKER; }\n' \
			>"$tmp/rs_default/src/main.rs"

		# POSITIVE C — a seam that exists but is ON by default. With the manifest declared
		# as part of the seam, C-DEP and C-START are both clean, so this control isolates
		# C-SEAM.
		mkrust rs_seam_on
		mkdir -p "$tmp/rs_seam_on/src/reach"
		printf 'pub fn dial_ephor() {}\n' >"$tmp/rs_seam_on/src/reach/broker.rs"
		printf '\n[features]\ndefault = ["ephor-reach"]\nephor-reach = []\n' >>"$tmp/rs_seam_on/Cargo.toml"

		# NEGATIVE control — the same seam, off by default. Must PASS: an optional provider
		# behind a default-off flag is exactly what R-SOV-1 permits.
		mkrust rs_seam_off
		mkdir -p "$tmp/rs_seam_off/src/reach"
		printf 'pub fn dial_ephor() {}\n' >"$tmp/rs_seam_off/src/reach/broker.rs"
		printf '\n[features]\ndefault = []\nephor-reach = []\n' >>"$tmp/rs_seam_off/Cargo.toml"

		# POSITIVE D — the broker under its CURRENT name. Every control above is written in
		# `ephor`, the PRE-rename name. That is why the ephor -> pier rename was able to blind
		# this gate suite-wide while its own self-test stayed green: the fixtures could not
		# tell the difference between "the gate matches the broker" and "the gate matches the
		# string ephor". This control plants a `pier-` dependency and a `vul-os/pier` module
		# path, so a BROKER_RE that has lost the current name FAILS here instead of passing.
		mkdir -p "$tmp/pier-client/src"
		cat >"$tmp/pier-client/Cargo.toml" <<-'EOF'
			[package]
			name = "pier-client"
			version = "0.0.0"
			edition = "2021"
		EOF
		echo 'pub fn dial() {}' >"$tmp/pier-client/src/lib.rs"
		mkrust rs_dep_current_name
		printf '\npier-client = { path = "../pier-client" }\n' >>"$tmp/rs_dep_current_name/Cargo.toml"
		printf 'fn main() { pier_client::dial(); }\n' >"$tmp/rs_dep_current_name/src/main.rs"

		# POSITIVE E — the repo/module path shape, with NO dependency, so this isolates
		# C-START on `vul-os/pier` specifically.
		mkrust rs_default_current_name
		printf 'const BROKER: &str = "https://github.com/vul-os/pier";\nfn main() { let _ = BROKER; }\n' \
			>"$tmp/rs_default_current_name/src/main.rs"

		E='BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'
		expect rs_clean 0 "a clean tree passes with all 3 checks run" "$E" SEAM_PATHS= SEAM_FLAG=
		expect rs_dep 1 "C-DEP catches a default-feature dependency" "$E" SEAM_PATHS= SEAM_FLAG=
		expect rs_default 1 "C-START catches a default broker endpoint with no dependency" "$E" SEAM_PATHS= SEAM_FLAG=
		expect rs_seam_on 1 "C-SEAM catches a seam that is ON by default" "$E" \
			SEAM_PATHS="src/reach Cargo.toml" SEAM_FLAG="ephor-reach"
		expect rs_seam_off 0 "a declared, default-off seam is permitted" "$E" \
			SEAM_PATHS="src/reach Cargo.toml" SEAM_FLAG="ephor-reach"
		# A seam declared without naming the flag that gates it verifies nothing: exit 2.
		expect rs_seam_off 2 "an ungated seam is UNVERIFIABLE, never a pass" "$E" \
			SEAM_PATHS="src/reach Cargo.toml" SEAM_FLAG=
		# A stale seam path is an exemption nobody is reading: exit 2.
		expect rs_clean 2 "a stale seam path is UNVERIFIABLE, never a pass" "$E" \
			SEAM_PATHS="src/gone" SEAM_FLAG="ephor-reach"
		# REGRESSION control. An earlier revision exited 2 the moment any check could not
		# run, masking a real C-DEP violation behind an unrelated stale seam path. A
		# violation must outrank an unverifiable check.
		expect rs_dep 1 "a violation outranks an unverifiable check (regression)" "$E" \
			SEAM_PATHS="src/gone" SEAM_FLAG="ephor-reach"
		# RENAME controls — the gate must catch the broker under the name it has TODAY, not
		# only the name it had when these fixtures were written.
		expect rs_dep_current_name 1 "C-DEP catches the broker under its CURRENT name (pier-)" "$E" \
			SEAM_PATHS= SEAM_FLAG=
		expect rs_default_current_name 1 "C-START catches the current repo path (vul-os/pier)" "$E" \
			SEAM_PATHS= SEAM_FLAG=

		# THE DEFAULT ITSELF. Every control above passes BROKER_RE explicitly, and the script
		# reads ${BROKER_RE:-...}, so an explicit value ALWAYS wins and the compiled-in default
		# on the BROKER_RE= line is never exercised by any of them. That is precisely why the
		# ephor -> pier rename blinded this gate without turning the self-test red: the controls
		# proved the MATCHING LOGIC handled whatever name they handed it, never that the name
		# the gate reaches for BY DEFAULT is the broker's current one. These two omit BROKER_RE
		# entirely, so they fail the moment the default stops naming the live broker.
		expect rs_dep_current_name 1 \
			"the DEFAULT BROKER_RE (no override) still names the live broker — C-DEP" \
			SEAM_PATHS= SEAM_FLAG=
		expect rs_default_current_name 1 \
			"the DEFAULT BROKER_RE (no override) still names the live repo path — C-START" \
			SEAM_PATHS= SEAM_FLAG=
	else
		unexercised="$unexercised rust(no-cargo)"
	fi

	# ---- Go controls -------------------------------------------------------------------
	# Go's mechanics are genuinely different: the seam is a BUILD TAG, not a feature, and
	# `go list -deps ./...` is what proves a tagged file is outside the default build. The
	# claim that this works is worth a control of its own.
	if command -v go >/dev/null 2>&1; then
		ran=$((ran + 1))
		expected_controls=$((expected_controls + 2))
		mkdir -p "$tmp/ephorclient"
		cat >"$tmp/ephorclient/go.mod" <<-'EOF'
			module example.test/ephorclient

			go 1.21
		EOF
		printf 'package ephorclient\n\nfunc Dial() string { return "broker" }\n' \
			>"$tmp/ephorclient/dial.go"

		mkgo() { # $1 = fixture name; creates a module whose reach/ package has two builds
			mkdir -p "$tmp/$1/reach"
			cat >"$tmp/$1/go.mod" <<-EOF
				module example.test/$1

				go 1.21

				require example.test/ephorclient v0.0.0

				replace example.test/ephorclient => ../ephorclient
			EOF
			printf 'package main\n\nimport "example.test/%s/reach"\n\nfunc main() { println(reach.Provider()) }\n' \
				"$1" >"$tmp/$1/main.go"
			printf '//go:build broker\n\npackage reach\n\nimport "example.test/ephorclient"\n\nfunc Provider() string { return ephorclient.Dial() }\n' \
				>"$tmp/$1/reach/broker.go"
		}

		# NEGATIVE control — the seam is behind `//go:build broker`, so the default build
		# never imports it.
		mkgo go_seam_off
		printf '//go:build !broker\n\npackage reach\n\nfunc Provider() string { return "direct" }\n' \
			>"$tmp/go_seam_off/reach/default.go"

		# POSITIVE control — the broker is imported by the DEFAULT build file.
		mkgo go_planted
		printf '//go:build !broker\n\npackage reach\n\nimport "example.test/ephorclient"\n\nfunc Provider() string { return ephorclient.Dial() }\n' \
			>"$tmp/go_planted/reach/default.go"

		E='BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'
		expect go_seam_off 0 "go: a build-tag seam is outside the default import closure" "$E" \
			SEAM_PATHS="reach go.mod" SEAM_FLAG="broker"
		expect go_planted 1 "go: C-DEP catches a broker imported by the default build" "$E" \
			SEAM_PATHS="reach go.mod" SEAM_FLAG="broker"
	else
		unexercised="$unexercised go(no-go-toolchain)"
	fi

	# ---- Structural controls: three defects found by running this gate across the suite ----
	# Each of these shipped as a clean-looking pass while the gate was wrong. They are controls
	# now so they cannot come back silently.
	if command -v go >/dev/null 2>&1; then
		ran=$((ran + 1))
		expected_controls=$((expected_controls + 4))

		# DEFECT 1 — nested prune. A prune list matched by top-level PATH walks every nested
		# node_modules/ and target/, so a vendored file naming the broker fails a product that
		# does not depend on it. The planted file is deep enough that only a name-match prunes it.
		mkdir -p "$tmp/prune_nested/web/node_modules/pkg" "$tmp/prune_nested/crates/x/target/debug"
		printf 'module example.test/prune_nested\n\ngo 1.21\n' >"$tmp/prune_nested/go.mod"
		printf 'package main\n\nfunc main() {}\n' >"$tmp/prune_nested/main.go"
		printf 'const url = "https://ephor.example";\n' \
			>"$tmp/prune_nested/web/node_modules/pkg/index.js"
		printf 'ephor build fingerprint\n' >"$tmp/prune_nested/crates/x/target/debug/build.log"
		expect prune_nested 0 "nested node_modules/ and target/ are pruned at any depth, not just the root" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

		# DEFECT 2 — monorepo blind spot. package.json at the root, go.mod in backend/. The
		# if/elif chain read the npm closure and never looked at Go, reporting a clean pass over
		# an unexamined product. The planted broker import is reachable ONLY through the Go side.
		mkdir -p "$tmp/multi_mod/backend"
		printf '{"name":"multi","version":"0.0.0","private":true}\n' >"$tmp/multi_mod/package.json"
		cat >"$tmp/multi_mod/backend/go.mod" <<-'EOF'
			module example.test/multi/backend

			go 1.21

			require example.test/ephorclient v0.0.0

			replace example.test/ephorclient => ../../ephorclient
		EOF
		printf 'package main\n\nimport "example.test/ephorclient"\n\nfunc main() { println(ephorclient.Dial()) }\n' \
			>"$tmp/multi_mod/backend/main.go"
		expect multi_mod 1 "a second manifest below the root is CHECKED, not silently skipped" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

		# DEFECT 3 — the gate flagged a CI step whose purpose was asserting the broker's absence.
		# The marker exempts that file and prints the stated reason; without it this same tree
		# fails, which is what makes the control meaningful rather than decorative.
		mkdir -p "$tmp/assert_absence/.github/workflows"
		printf 'module example.test/assert_absence\n\ngo 1.21\n' >"$tmp/assert_absence/go.mod"
		printf 'package main\n\nfunc main() {}\n' >"$tmp/assert_absence/main.go"
		printf 'name: ci\njobs:\n  x:\n    steps:\n      - run: grep -rE "ephor" . && exit 1\n' \
			>"$tmp/assert_absence/.github/workflows/ci.yml"
		expect assert_absence 1 "an unmarked file naming the broker still FAILS (the marker is doing the work)" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'
		printf '# no-broker-dep%s asserts this broker is absent; the grep must spell its name\n' \
			"$_ALLOW_SUFFIX" >>"$tmp/assert_absence/.github/workflows/ci.yml"
		expect assert_absence 0 "a file may name the broker to assert its ABSENCE, with a printed reason" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'
	else
		unexercised="$unexercised structural(no-go-toolchain)"
	fi

	# ---- DEFECT 4 — a THIRD instance of the same bug class: the gate failed a repo for
	# stating, in prose, that it does NOT depend on the broker (molao/GOVERNANCE.md, cackle/
	# CONTRIBUTING.md — both real, both fixed by the DOC_PATHS additions above). The danger in
	# fixing that is fixing it TOO WIDE: a disclaimer must never be able to launder an actual
	# dependency. These four controls pin both directions at once — a disclaimer-only repo
	# passes, and a disclaimer sitting next to a REAL dependency (manifest closure, or a
	# hardcoded default in the startup path) still fails, exactly as loud as it would without
	# the disclaimer. A fourth control proves the companion prune-list addition (dist-lib and
	# siblings, generated output confirmed via .gitignore across the fleet) is pruned the same
	# way node_modules/target already are — DEFECT 1 above, recurring under a different name.
	if command -v go >/dev/null 2>&1; then
		ran=$((ran + 1))
		expected_controls=$((expected_controls + 4))

		disclaimer_doc() { # $1 = fixture dir — writes the shared disclaimer prose
			printf '# Governance\n\nNo hard dependency on Ephor; vulos-relayd is not part of this\nproject'"'"'s default path.\n' \
				>"$tmp/$1/GOVERNANCE.md"
		}

		# CONTROL A — the disclaimer alone. Nothing else in the tree names the broker.
		mkdir -p "$tmp/disclaimer_only"
		printf 'module example.test/disclaimer_only\n\ngo 1.21\n' >"$tmp/disclaimer_only/go.mod"
		printf 'package main\n\nfunc main() {}\n' >"$tmp/disclaimer_only/main.go"
		disclaimer_doc disclaimer_only
		expect disclaimer_only 0 "a GOVERNANCE.md disclaimer with NO other mention of the broker PASSES" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

		# CONTROL B — the SAME disclaimer, but the default build now really imports the broker
		# (a real manifest dependency, C-DEP's subject). The disclaimer must not launder it.
		mkdir -p "$tmp/disclaimer_plus_dep"
		cat >"$tmp/disclaimer_plus_dep/go.mod" <<-'EOF'
			module example.test/disclaimer_plus_dep

			go 1.21

			require example.test/ephorclient v0.0.0

			replace example.test/ephorclient => ../ephorclient
		EOF
		printf 'package main\n\nimport "example.test/ephorclient"\n\nfunc main() { println(ephorclient.Dial()) }\n' \
			>"$tmp/disclaimer_plus_dep/main.go"
		disclaimer_doc disclaimer_plus_dep
		expect disclaimer_plus_dep 1 "a real manifest dependency still FAILS behind a disclaimer doc" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

		# CONTROL C — the SAME disclaimer, no manifest dependency at all, but a hardcoded
		# default broker endpoint in the startup path (C-START's subject, not C-DEP's).
		mkdir -p "$tmp/disclaimer_plus_default"
		printf 'module example.test/disclaimer_plus_default\n\ngo 1.21\n' >"$tmp/disclaimer_plus_default/go.mod"
		printf 'const DefaultBroker = "https://rendezvous.ephor.example"\n\nfunc main() { _ = DefaultBroker }\n' \
			>"$tmp/disclaimer_plus_default/main.go"
		disclaimer_doc disclaimer_plus_default
		expect disclaimer_plus_default 1 "a real default-endpoint in the startup path still FAILS behind a disclaimer doc" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

		# CONTROL D — the prune-list companion fix. A broker string sitting only inside a
		# generated dist-lib/ (gitignored everywhere it was found) must not fail a product
		# whose authored source never mentions the broker at all.
		mkdir -p "$tmp/prune_dist_lib/dist-lib"
		printf 'module example.test/prune_dist_lib\n\ngo 1.21\n' >"$tmp/prune_dist_lib/go.mod"
		printf 'package main\n\nfunc main() {}\n' >"$tmp/prune_dist_lib/main.go"
		printf 'const u="https://ephor.example";export{u};\n' >"$tmp/prune_dist_lib/dist-lib/bundle.js"
		expect prune_dist_lib 0 "a generated dist-lib/ bundle naming the broker is pruned, like node_modules/target" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'
	else
		unexercised="$unexercised docbug(no-go-toolchain)"
	fi

	# ---- DEFECT 5 (this fix) — a FOURTH instance of the same bug class, one level down from
	# DEFECT 2 above. DEFECT 2 was "package.json at the root, go.mod in backend/ — the whole
	# Go product was never read"; this is "go.mod AND package.json in the SAME directory —
	# only the first one in the if/elif's fixed priority order was ever read". diwan is the
	# live case (go.mod + package.json at the repo root, only Go's closure was ever
	# examined); wede, molao, cackle and evermesh all carry more than one manifest at a
	# checked directory too. check_dep_closure_one() now checks every manifest present
	# independently instead of stopping at the first — these controls pin exactly the
	# failure mode whose absence let the bug survive three prior fixes.
	if command -v go >/dev/null 2>&1 && command -v npm >/dev/null 2>&1; then
		ran=$((ran + 1))
		expected_controls=$((expected_controls + 3))

		# CONTROL A — the broker is declared ONLY in the SECOND manifest (package.json) of a
		# directory whose FIRST manifest (go.mod, clean) an if/elif chain would have checked
		# and stopped at. node_modules is hand-built (no `npm install`, no network) so the
		# fixture stays hermetic like every other control here.
		mkdir -p "$tmp/polyglot_second_only/node_modules/ephor-client"
		printf 'module example.test/polyglot_second_only\n\ngo 1.21\n' >"$tmp/polyglot_second_only/go.mod"
		printf 'package main\n\nfunc main() {}\n' >"$tmp/polyglot_second_only/main.go"
		printf '{"name":"ephor-client","version":"0.0.0"}\n' \
			>"$tmp/polyglot_second_only/node_modules/ephor-client/package.json"
		cat >"$tmp/polyglot_second_only/package.json" <<-'EOF'
			{
			  "name": "polyglot-second-only",
			  "version": "0.0.0",
			  "private": true,
			  "dependencies": { "ephor-client": "0.0.0" }
			}
		EOF
		expect polyglot_second_only 1 \
			"a broker declared ONLY in the SECOND manifest of a directory (go.mod clean, package.json names it) still FAILS — the exact control whose absence let this bug survive" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

		# CONTROL B — same shape, but the SECOND manifest's closure cannot be read at all (a
		# malformed package.json here; a broken/extraneous node_modules is kerf's real-world
		# version of the same failure). Must be CANNOT-CHECK, never a silent pass just
		# because the first manifest in the directory happened to be clean.
		mkdir -p "$tmp/polyglot_second_unreadable"
		printf 'module example.test/polyglot_second_unreadable\n\ngo 1.21\n' \
			>"$tmp/polyglot_second_unreadable/go.mod"
		printf 'package main\n\nfunc main() {}\n' >"$tmp/polyglot_second_unreadable/main.go"
		printf '{ this is not valid json' >"$tmp/polyglot_second_unreadable/package.json"
		expect polyglot_second_unreadable 2 \
			"a second manifest whose closure cannot be read is CANNOT-CHECK, not a silent pass riding on the first manifest's clean result" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

		# CONTROL C — the negative: BOTH manifests in the same directory are clean. Proves
		# checking every manifest present does not itself manufacture a false FAIL.
		mkdir -p "$tmp/polyglot_clean"
		printf 'module example.test/polyglot_clean\n\ngo 1.21\n' >"$tmp/polyglot_clean/go.mod"
		printf 'package main\n\nfunc main() {}\n' >"$tmp/polyglot_clean/main.go"
		printf '{"name":"polyglot-clean","version":"0.0.0","private":true}\n' \
			>"$tmp/polyglot_clean/package.json"
		expect polyglot_clean 0 \
			"a polyglot directory where EVERY manifest is clean still PASSES (checking more manifests is not itself a false positive)" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'
	else
		unexercised="$unexercised polyglot(no-go-or-npm-toolchain)"
	fi

	# ---- DEFECT 6 (this fix) — the SAME bug class as DEFECT 2/3/5, one function over:
	# check_seam() used to be an if/elif over the REPO ROOT's own manifest existence
	# (Cargo.toml, else go.mod, else "cannot verify"), so a declared seam's OWN ecosystem was
	# never what got checked — whichever manifest happened to ALSO exist at root won,
	# regardless of what SEAM_PATHS actually named. Proven by fixture in BOTH directions:
	# a correct, off-by-default Go build-tag seam sitting beside an unrelated Cargo.toml at
	# root was silently never examined (Cargo.toml won the elif, checked its own [features]
	# for a flag name that was never a cargo feature, and printed the IDENTICAL "clean"
	# message whether the real Go seam was correct or mutated to compile the broker
	# unconditionally — isolating check_seam() alone from check_dep_closure() proved this).
	# Worse: a legitimate npm-based seam that was never even installed (genuinely off by
	# default — no node_modules/ at all) was REJECTED by the gate when an unrelated go.mod
	# also sat at root, because the Go branch's mechanic (grep for a `//go:build` tag) was
	# applied to package.json, which structurally cannot contain that syntax — a false FAIL
	# with C-DEP(go) and C-DEP(node) both independently clean, so check_seam() alone was the
	# sole source of the violation. check_seam() now classifies by what SEAM_PATHS itself
	# names (Cargo.toml/.lock, go.mod/.sum, package.json/lockfiles), independently per
	# ecosystem, never elif — see its own header comment for the full account.
	if command -v cargo >/dev/null 2>&1 && command -v go >/dev/null 2>&1; then
		ran=$((ran + 1))
		expected_controls=$((expected_controls + 2))

		# CONTROL A — a genuinely correct, off-by-default Go build-tag seam declared via
		# SEAM_PATHS="reach go.mod", with an UNRELATED Cargo.toml (no [features] section at
		# all) also at root. An if/elif over root manifest existence takes the Cargo.toml
		# branch first and never looks at the declared Go seam at all; the fix must verify it.
		mkdir -p "$tmp/seam_go_root_has_cargo/src" "$tmp/seam_go_root_has_cargo/reach"
		cat >"$tmp/seam_go_root_has_cargo/Cargo.toml" <<-'EOF'
			[package]
			name = "seam_go_root_has_cargo"
			version = "0.0.0"
			edition = "2021"
		EOF
		echo 'pub fn stub() {}' >"$tmp/seam_go_root_has_cargo/src/lib.rs"
		cat >"$tmp/seam_go_root_has_cargo/go.mod" <<-'EOF'
			module example.test/seam_go_root_has_cargo

			go 1.21

			require example.test/ephorclient v0.0.0

			replace example.test/ephorclient => ../ephorclient
		EOF
		printf 'package main\n\nimport "example.test/seam_go_root_has_cargo/reach"\n\nfunc main() { println(reach.Provider()) }\n' \
			>"$tmp/seam_go_root_has_cargo/main.go"
		printf '//go:build !broker\n\npackage reach\n\nfunc Provider() string { return "direct" }\n' \
			>"$tmp/seam_go_root_has_cargo/reach/default.go"
		printf '//go:build broker\n\npackage reach\n\nimport "example.test/ephorclient"\n\nfunc Provider() string { return ephorclient.Dial() }\n' \
			>"$tmp/seam_go_root_has_cargo/reach/broker.go"
		expect seam_go_root_has_cargo 0 \
			"a correct, off-by-default Go build-tag seam is genuinely verified even when an UNRELATED Cargo.toml also sits at root (not silently skipped because Cargo.toml won an elif)" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd' SEAM_PATHS='reach go.mod' SEAM_FLAG=broker

		# CONTROL B — the SAME fixture, but the Go seam is now BROKEN (the build tag removed,
		# so the broker compiles unconditionally). check_seam() itself must name the
		# violation, not merely ride on C-DEP's independent catch of the same underlying break.
		mkdir -p "$tmp/seam_go_root_has_cargo_broken/src" "$tmp/seam_go_root_has_cargo_broken/reach"
		cp "$tmp/seam_go_root_has_cargo/Cargo.toml" "$tmp/seam_go_root_has_cargo_broken/Cargo.toml"
		cp "$tmp/seam_go_root_has_cargo/src/lib.rs" "$tmp/seam_go_root_has_cargo_broken/src/lib.rs"
		cp "$tmp/seam_go_root_has_cargo/go.mod" "$tmp/seam_go_root_has_cargo_broken/go.mod"
		cp "$tmp/seam_go_root_has_cargo/main.go" "$tmp/seam_go_root_has_cargo_broken/main.go"
		printf 'package reach\n\nimport "example.test/ephorclient"\n\nfunc Provider() string { return ephorclient.Dial() }\n' \
			>"$tmp/seam_go_root_has_cargo_broken/reach/broker.go"
		expect seam_go_root_has_cargo_broken 1 \
			"the SAME fixture with the Go build tag removed (broker compiled unconditionally) still FAILS, with C-SEAM itself now able to name the violation, not only C-DEP" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd' SEAM_PATHS='reach go.mod' SEAM_FLAG=broker
	else
		unexercised="$unexercised seam-ecosystem-mismatch(no-cargo-or-go)"
	fi

	if command -v go >/dev/null 2>&1 && command -v npm >/dev/null 2>&1; then
		ran=$((ran + 1))
		expected_controls=$((expected_controls + 1))

		# CONTROL C — the sharper proof of the false-POSITIVE direction: a legitimate
		# npm-based seam that was NEVER INSTALLED at all (genuinely off by default — no
		# node_modules/, nothing for `npm ls` to find), with an UNRELATED go.mod also at
		# root. The if/elif this replaces took the go.mod branch and applied its
		# `//go:build` grep to package.json, which cannot contain that syntax, producing a
		# FALSE VIOLATION against a product with no real dependency anywhere: C-DEP(go) and
		# C-DEP(node) are both independently clean here, so a FAIL could only have come from
		# C-SEAM misjudging the wrong ecosystem. The fix must report this as unverifiable —
		# npm has no structural off-by-default mechanic here — never a false FAIL and never
		# a false PASS either.
		mkdir -p "$tmp/seam_npm_root_has_go"
		printf 'module example.test/seam_npm_root_has_go\n\ngo 1.21\n' >"$tmp/seam_npm_root_has_go/go.mod"
		printf 'package main\n\nfunc main() {}\n' >"$tmp/seam_npm_root_has_go/main.go"
		cat >"$tmp/seam_npm_root_has_go/package.json" <<-'EOF'
			{
			  "name": "seam-npm-root-has-go",
			  "version": "0.0.0",
			  "private": true,
			  "optionalDependencies": { "ephor-client": "0.0.0" }
			}
		EOF
		expect seam_npm_root_has_go 2 \
			"a genuinely off-by-default npm seam (never installed) beside an UNRELATED go.mod at root is CANNOT-CHECK, never a false FAIL from applying Go build-tag mechanics to package.json" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd' SEAM_PATHS=package.json SEAM_FLAG=ephor-client
	else
		unexercised="$unexercised seam-ecosystem-mismatch-npm(no-go-or-npm)"
	fi

	# ---- DEFECT 7 (this fix) — is_exempt() dropped a reason, not a verdict. A file CAN be
	# exempt via more than one mechanism at once (a DOC_PATHS/SEAM_PATHS prefix match AND its
	# own :allow-file marker), and an earlier revision short-circuited on whichever matched
	# first: if the coarse path prefix matched, the function returned before ever reaching the
	# marker, so a human-authored, more specific reason was silently swallowed whenever a
	# broader rule already covered the same file. The boolean verdict never changed (either
	# mechanism alone already exempts the file), so this is NOT the exit-code-flipping shape
	# check_seam()/check_dep_closure() were fixed for — it is the adjacent one: "one mechanism
	# prints a reason and another doesn't", a reporting hole even though the answer stayed
	# right. Needs no toolchain at all: is_exempt() is pure path/text logic. Unconditional.
	ran=$((ran + 1))
	expected_controls=$((expected_controls + 2))

	mkdir -p "$tmp/dual_exempt_reason/docs"
	printf 'Some docs prose about the Ephor broker.\nno-broker-dep%s this paragraph quotes a THIRD PARTY notice verbatim, unrelated to why docs/ is exempt at all\n' \
		"$_ALLOW_SUFFIX" >"$tmp/dual_exempt_reason/docs/mentions.md"
	expect_output dual_exempt_reason has \
		'EXEMPT-FILE  docs/mentions.md — this paragraph quotes a THIRD PARTY notice verbatim' \
		"a file exempt via BOTH a DOC_PATHS prefix match AND its own :allow-file marker still prints the marker's specific reason — not silently swallowed by the broader path rule reached first" \
		'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

	# NEGATIVE control — the same DOC_PATHS-exempt directory, but a file with NO marker at
	# all. Must stay silent about that file: the fix is a pure superset of what used to print,
	# not new noise on the common case (a docs/ tree with no per-file marker at all).
	printf 'Just plain documentation prose about Ephor, no marker.\n' >"$tmp/dual_exempt_reason/docs/plain.md"
	expect_output dual_exempt_reason lacks 'plain.md' \
		"a DOC_PATHS-exempt file with NO marker prints nothing extra — the fix adds no noise to the common case" \
		'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

	# ---- Repo-self-declaration controls (`.no-broker-dep-self`) — unconditional: this
	# mechanism short-circuits before check_dep_closure/check_startup_text ever run, so it
	# needs no toolchain at all. The fixture tree is otherwise EMPTY (no manifest, no source),
	# which makes the pair maximally sharp: without the declaration an empty tree is
	# UNVERIFIABLE (C-DEP finds no manifest, exit 2), so a reasoned declaration flipping that
	# to exit 0 proves the short-circuit actually fired rather than the tree coincidentally
	# passing on its own.
	ran=$((ran + 1))
	expected_controls=$((expected_controls + 2))

	mkdir -p "$tmp/self_broker_reasoned"
	printf 'no-broker-dep:self-broker: this fixture pretends to be the broker itself\n' \
		>"$tmp/self_broker_reasoned/.no-broker-dep-self"
	expect self_broker_reasoned 0 \
		"a reasoned .no-broker-dep-self exits 0 with ZERO checks run, on an otherwise-unverifiable empty tree" \
		'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

	mkdir -p "$tmp/self_broker_unreasoned"
	printf 'no-broker-dep:self-broker\n' >"$tmp/self_broker_unreasoned/.no-broker-dep-self"
	expect self_broker_unreasoned 2 \
		"an UNREASONED .no-broker-dep-self is NOT honoured — falls through to a normal (here, unverifiable) scan" \
		'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

	# ---- DEFECT 8 (this fix) — the walk scanned gitignored, regenerated build output. ----
	# vulos-static's own docs-ingestion and product-landing collectors (`scripts/sync-docs.mjs`,
	# `scripts/collect-repo-landings.mjs`) regenerate `content/docs/`, `.astro/` and
	# `public/projects/*/` on every build; both scripts' own docstrings say so ("THE OUTPUT IS
	# NOT COMMITTED") and `.gitignore` already lists all three. None of those three names are in
	# PRUNE_NAMES (the fixed list only knows `dist`/`build`/the `dist-*` spellings audited so
	# far), so the gate walked them anyway: 84 violations, 100% inside those three trees, none of
	# which exist in a single commit or a fresh clone. Marking them was never an option — git will
	# not persist a comment written into a file it is told to ignore.
	#
	# THE FIX: enumerate_files() now asks git (`ls-files --cached --others --exclude-standard`)
	# for the tracked-plus-untracked-but-not-ignored set instead of walking the disk with a name
	# list, falling back to the old PRUNE_NAMES walk only when there is no git to ask (checked
	# by the fallback-announced control below, which needs no toolchain and so runs
	# unconditionally alongside the other self-declaration controls above).
	#
	# The two controls below are the ones that actually matter, proven the same way the fix
	# itself was proven against a real fresh clone of vulos-static: a broker mention sitting in a
	# GITIGNORED file must be invisible to the gate (control A), and the identical mention sitting
	# in a file git actually TRACKS must still fail exactly as loudly as before (control B) — the
	# control that tells a correct git-aware prune apart from a scan that got quietly gutted.
	# Needs no toolchain beyond git+go (go, only so C-DEP has a trivially clean closure to read,
	# keeping the exit code an unambiguous 0/1 rather than folding in an unrelated cannot-check).
	if command -v git >/dev/null 2>&1 && command -v go >/dev/null 2>&1; then
		ran=$((ran + 1))
		expected_controls=$((expected_controls + 2))

		# CONTROL A — the broker is named ONLY inside a file the fixture's OWN .gitignore
		# excludes, and that file is never `git add`ed either — exactly vulos-static's
		# content/docs/.astro/public/projects shape (untracked, ignored, regenerated). A gate
		# that still walks the disk with a name-based prune list would find it; one that asks
		# git for tracked-plus-untracked-but-not-ignored never sees it at all.
		mkdir -p "$tmp/git_walk_ignored"
		(
			cd "$tmp/git_walk_ignored" || exit 1
			git init -q
			printf 'module example.test/git_walk_ignored\n\ngo 1.21\n' >go.mod
			printf 'package main\n\nfunc main() {}\n' >main.go
			printf 'ignored-broker.md\n' >.gitignore
			printf '# regenerated docs mirror — not committed\nDefaults to https://rendezvous.ephor.example\n' \
				>ignored-broker.md
		) >/dev/null 2>&1
		expect git_walk_ignored 0 \
			"a broker mention confined to a GITIGNORED, untracked file (vulos-static's exact content/docs/.astro/public-projects shape) is never scanned at all — the fix, not a laxer gate" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

		# CONTROL B — the decisive discriminator. Identical tree, identical broker text, but
		# the file is `git add`ed (staged into the index — a commit is not required for
		# `--cached` to see it) instead of gitignored. If the fix had gutted the scan instead
		# of correctly following git, this would go quiet along with control A; it must not.
		mkdir -p "$tmp/git_walk_tracked"
		(
			cd "$tmp/git_walk_tracked" || exit 1
			git init -q
			printf 'module example.test/git_walk_tracked\n\ngo 1.21\n' >go.mod
			printf 'package main\n\nfunc main() {}\n' >main.go
			printf 'ignored-broker.md\n' >.gitignore
			printf '# tracked source, not a build artifact\nDefaults to https://rendezvous.ephor.example\n' \
				>tracked-broker.md
			git add go.mod main.go .gitignore tracked-broker.md
		) >/dev/null 2>&1
		expect git_walk_tracked 1 \
			"the SAME broker text in a file git actually tracks (staged, no commit needed) still FAILS — proves the fix narrowed nothing for tracked content" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'
	else
		unexercised="$unexercised gitwalk(no-git-or-go)"
	fi

	# CONTROL C — the non-git fallback is ANNOUNCED, never silent. Every fixture in this
	# selftest already runs in a plain mktemp directory (not a git repo), so this is already
	# exercised implicitly by all 28+ controls above and below — but "implicitly" is exactly the
	# kind of evidence this whole script exists to distrust (see the file's own rule: assert
	# coverage, don't infer it). This control names the requirement directly: rule 2 of the task
	# that produced this fix requires the fallback be loud, not a silent downgrade to the old
	# behaviour. Reuses the self_broker_unreasoned fixture built above (still a plain, non-git
	# directory) rather than minting a new one — running the gate again against it is free and
	# mutates nothing. Needs no toolchain: pure path logic, unconditional.
	ran=$((ran + 1))
	expected_controls=$((expected_controls + 1))
	expect_output self_broker_unreasoned has 'CANNOT use git here' \
		"outside a git working tree, the fallback to the static PRUNE_NAMES walk is printed loudly, not silently taken" \
		'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'

	# CONTROL D — the git path is likewise announced when it DOES fire, so a run's own output is
	# sufficient to tell which mechanism verified it without inferring that from the violation
	# count alone. Reuses control A's fixture (git_walk_ignored) — same reasoning as control C.
	if command -v git >/dev/null 2>&1 && command -v go >/dev/null 2>&1; then
		ran=$((ran + 1))
		expected_controls=$((expected_controls + 1))
		expect_output git_walk_ignored has 'git ls-files --cached --others --exclude-standard' \
			"inside a git working tree, the gate announces it is using git-based enumeration, not left to be inferred" \
			'BROKER_RE=pier-|vul-os/pier|ephor|vulos-relayd'
	else
		unexercised="$unexercised gitwalk-announce(no-git-or-go)"
	fi

	# The control COUNT is itself asserted, not just printed — a control quietly deleted (an
	# `expect` line dropped in a future edit, or a whole section commented out) still leaves
	# `rc` at 0 if every remaining `expect` happens to pass, which is exactly "a gate that
	# passes everything gates nothing" one level up: a selftest that silently shrank. Each
	# gated section above declares its own control count as it runs, so this total is the sum
	# of only the sections that actually ran on THIS machine — a toolchain genuinely missing is
	# still reported via $unexercised, not folded into a mismatch.
	if [ "$controls" != "$expected_controls" ]; then
		say "SELFTEST FAIL  ran $controls control(s) but the sections that executed declare $expected_controls —"
		say "               a control was added or removed without updating its section's count. Do not"
		say "               trust this as a shrink-proof run until the two numbers agree."
		rc=1
	fi

	say ""
	[ -n "$unexercised" ] &&
		say "NOT VERIFIED   ecosystems not exercised in this run:$unexercised — their mechanics are unproven here."
	if [ "$ran" = 0 ]; then
		say "SELFTEST CANNOT RUN  no toolchain present for any control fixture. This is exit 2, not a"
		say "                     pass: nothing was verified."
		return 2
	fi
	if [ "$rc" = 0 ]; then
		say "SELFTEST PASS  $controls controls across $ran ecosystem(s), count asserted: clean trees and"
		say "               default-off seams pass, every planted violation fails, unverifiable"
		say "               configurations exit 2, a violation outranks an unverifiable check, a"
		say "               disclaimer-only doc passes while a real dependency behind one still fails, a"
		say "               generated dist-lib/ bundle is pruned like node_modules/target, a broker named"
		say "               ONLY in the second of two manifests sharing a directory still fails and an"
		say "               unreadable second closure is cannot-check not a pass, a correct Go"
		say "               build-tag seam is genuinely verified (not silently skipped) when an"
		say "               unrelated Cargo.toml also sits at root, a genuinely off-by-default npm"
		say "               seam beside an unrelated go.mod is cannot-check rather than a false FAIL, a"
		say "               file exempt via a path rule AND its own marker still prints the marker's"
		say "               reason (never silently swallowed), a reasoned .no-broker-dep-self exits 0"
		say "               having run nothing, an unreasoned one is ignored outright, a broker mention"
		say "               confined to a gitignored file is never scanned while the identical mention"
		say "               in a tracked file still fails (the git-based walk narrows nothing tracked),"
		say "               and the fallback to the static prune list outside a git checkout is always"
		say "               announced, never silent. The gate is not inert."
	else
		say "SELFTEST FAIL  a control did not behave as specified — do NOT trust a clean run of this gate"
		say "               until it does."
	fi
	return $rc
}

case ${1:-} in
--selftest) selftest ;;
-h | --help) sed -n '2,66p' "$0" ;;
*) run_gate "${1:-.}" ;;
esac
