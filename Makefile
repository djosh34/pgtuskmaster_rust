MDBOOK := .tools/mdbook/bin/mdbook
MDBOOK_MERMAID := .tools/mdbook/bin/mdbook-mermaid

.PHONY: check test test-long lint docs-build docs-serve docs-hygiene docs-lint ensure-mdbook ensure-mdbook-mermaid ensure-node ensure-timeout guard-makeflags

SINGLE_DASH_MAKEFLAGS := $(filter -%,$(MAKEFLAGS))
SINGLE_DASH_MAKEFLAGS := $(filter-out --%,$(SINGLE_DASH_MAKEFLAGS))

ifneq ($(filter -n --dry-run --just-print --recon n,$(MAKEFLAGS))$(filter %n%,$(SINGLE_DASH_MAKEFLAGS)),)
$(error Refusing to run Makefile gates with dry-run enabled (MAKEFLAGS contains -n/--dry-run))
endif
ifneq ($(filter -i --ignore-errors i,$(MAKEFLAGS))$(filter %i%,$(SINGLE_DASH_MAKEFLAGS)),)
$(error Refusing to run Makefile gates with ignore-errors enabled (MAKEFLAGS contains -i/--ignore-errors))
endif

ifneq ($(origin ULTRA_LONG_TESTS),undefined)
$(error ULTRA_LONG_TESTS must not be set externally; edit Makefile to change the canonical ultra-long test list)
endif

ULTRA_LONG_TESTS := \
		ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix \
		ha::e2e_multi_node::e2e_multi_node_stress_planned_switchover_concurrent_sql \
		ha::e2e_multi_node::e2e_multi_node_stress_unassisted_failover_concurrent_sql \
		ha::e2e_multi_node::e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity
ULTRA_LONG_SKIP_ARGS := $(foreach t,$(ULTRA_LONG_TESTS),--skip $(t))

ifneq ($(origin ULTRA_LONG_SKIP_ARGS),file)
$(error ULTRA_LONG_SKIP_ARGS must not be set externally; edit Makefile to change the canonical skip list)
endif

# The workspace mount this repo typically lives on can exhibit intermittent
# linker/archive flake with incremental artifacts. Disable incremental builds by
# default for deterministic `make` gates; override with `CARGO_INCREMENTAL=1`
# if you explicitly want it.
CARGO_INCREMENTAL ?= 0

TEST_TIMEOUT_SECS := 120
TEST_TIMEOUT_KILL_AFTER_SECS := 15
TIMEOUT_BIN := $(shell command -v timeout 2>/dev/null || command -v gtimeout 2>/dev/null)

ifneq ($(origin TEST_TIMEOUT_SECS),file)
$(error TEST_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin TEST_TIMEOUT_KILL_AFTER_SECS),file)
$(error TEST_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin TIMEOUT_BIN),file)
$(error TIMEOUT_BIN must not be set externally; edit Makefile to change timeout resolution)
endif

guard-makeflags:
	@set -eu; \
	flags=" $${MAKEFLAGS:-} "; \
	printf '%s\n' "$${flags}" | grep -Eq '(^|[[:space:]])(-n|--just-print|--dry-run|--recon|n)([[:space:]]|$$)' && { \
		echo "Refusing to run gate target with MAKEFLAGS dry-run enabled (-n/--dry-run)." >&2; \
		exit 1; \
	}; \
	printf '%s\n' "$${flags}" | grep -Eq '(^|[[:space:]])(-i|--ignore-errors|i)([[:space:]]|$$)' && { \
		echo "Refusing to run gate target with MAKEFLAGS ignore-errors enabled (-i/--ignore-errors)." >&2; \
		exit 1; \
	}; \
	:

ensure-mdbook:
	@test -x "$(MDBOOK)" || (echo "missing mdBook binary: run ./tools/install-mdbook.sh" >&2; exit 1)

ensure-mdbook-mermaid: ensure-mdbook
	@test -x "$(MDBOOK_MERMAID)" || (echo "missing mdbook-mermaid binary: run ./tools/install-mdbook-mermaid.sh" >&2; exit 1)

ensure-node:
	@command -v node >/dev/null 2>&1 || (echo "missing node binary (required for Mermaid docs lint)" >&2; exit 1)

ensure-timeout:
	@test -n "$(TIMEOUT_BIN)" || (echo "missing timeout binary (install coreutils). Need either 'timeout' (Linux) or 'gtimeout' (macOS)." >&2; exit 1)

check:
	CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" cargo check --all-targets

check: guard-makeflags

test: guard-makeflags ensure-timeout
	@set -euo pipefail; \
	if [ "$${RUST_TEST_THREADS:-}" = "1" ]; then echo "RUST_TEST_THREADS=1 is disallowed for make test (parallel must work). Fix parallel flakes instead." >&2; exit 1; fi; \
	list_file="$$(mktemp)"; \
	trap 'rm -f "$${list_file}"' EXIT; \
	echo "Preflight: listing tests for ultra-long skip-token validation..."; \
	env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" cargo test --all-targets -- --list > "$${list_file}"; \
	dupes="$$(printf '%s\n' $(ULTRA_LONG_TESTS) | sort | uniq -d)"; \
	if [ -n "$${dupes}" ]; then \
		echo "Ultra-long test list contains duplicates: $${dupes}" >&2; \
		exit 1; \
	fi; \
	for t in $(ULTRA_LONG_TESTS); do \
		if ! awk -F': ' '{print $$1}' "$${list_file}" | grep -Fx "$$t" >/dev/null; then \
			echo "Ultra-long test not found (exact): $$t" >&2; \
			exit 1; \
		fi; \
		match_count="$$(awk -F': ' '{print $$1}' "$${list_file}" | grep -F "$$t" | wc -l | tr -d ' ')"; \
		if [ "$${match_count}" != "1" ]; then \
			echo "Ultra-long skip token is ambiguous (substring matches $${match_count} tests): $$t" >&2; \
			exit 1; \
		fi; \
	done; \
	"$(TIMEOUT_BIN)" --kill-after="$(TEST_TIMEOUT_KILL_AFTER_SECS)s" "$(TEST_TIMEOUT_SECS)s" env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" cargo test --all-targets -- $(ULTRA_LONG_SKIP_ARGS)

test-long: guard-makeflags
	@echo "test-long runs only ultra-long tests (evidence-backed passed runtime >= 3 minutes)."
	@echo "If one becomes short enough for regular development cycles, move it back into make test."
	@set -e; \
	if [ -z "$(strip $(ULTRA_LONG_TESTS))" ]; then \
		echo "No ultra-long tests configured."; \
		exit 1; \
	fi; \
	list_file="$$(mktemp)"; \
	trap 'rm -f "$${list_file}"' EXIT; \
	echo "Preflight: listing tests for exact-match validation..."; \
	env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" cargo test --all-targets -- --list > "$${list_file}"; \
	dupes="$$(printf '%s\n' $(ULTRA_LONG_TESTS) | sort | uniq -d)"; \
	if [ -n "$${dupes}" ]; then \
		echo "Ultra-long test list contains duplicates: $${dupes}" >&2; \
		exit 1; \
	fi; \
	for t in $(ULTRA_LONG_TESTS); do \
		if ! awk -F': ' '{print $$1}' "$${list_file}" | grep -Fx "$$t" >/dev/null; then \
			echo "Ultra-long test not found (exact): $$t" >&2; \
			exit 1; \
		fi; \
		match_count="$$(awk -F': ' '{print $$1}' "$${list_file}" | grep -F "$$t" | wc -l | tr -d ' ')"; \
		if [ "$${match_count}" != "1" ]; then \
			echo "Ultra-long test name is not unique (substring matches $${match_count} tests): $$t" >&2; \
			exit 1; \
		fi; \
	done; \
	for t in $(ULTRA_LONG_TESTS); do \
		env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" cargo test --all-targets "$$t" -- --exact; \
	done

docs-lint: ensure-node
	node ./tools/docs-mermaid-lint.mjs
	./tools/docs-architecture-no-code-guard.sh

lint: guard-makeflags docs-lint
	env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" cargo clippy --all-targets --all-features -- -D warnings
	# Strict restriction-lint pass for runtime library builds.
	env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" cargo clippy --lib --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	# Strict restriction-lint pass for test targets.
	env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" cargo clippy --tests --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	# Skeptical all-target guard so restrictions are enforced uniformly.
	env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" cargo clippy --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented

docs-build: ensure-mdbook-mermaid
	PATH="$(CURDIR)/.tools/mdbook/bin:$$PATH" "$(MDBOOK)" build docs

docs-serve: ensure-mdbook-mermaid
	PATH="$(CURDIR)/.tools/mdbook/bin:$$PATH" "$(MDBOOK)" serve docs -n 127.0.0.1 -p 3000

docs-hygiene:
	@set -euo pipefail; \
	tracked="$$(git ls-files -- docs/book docs/.mdbook)"; \
	if [[ -n "$${tracked}" ]]; then \
		echo "generated docs output is tracked (must be removed from git index):" >&2; \
		echo "$${tracked}" >&2; \
		exit 1; \
	fi
