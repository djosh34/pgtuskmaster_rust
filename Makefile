MDBOOK := .tools/mdbook/bin/mdbook
MDBOOK_MERMAID := .tools/mdbook/bin/mdbook-mermaid

SHELL := /usr/bin/env bash

.PHONY: check test test-long lint lint.no_silent_errors docs-build docs-serve docs-hygiene docs-lint ensure-mdbook ensure-mdbook-mermaid ensure-node ensure-timeout guard-makeflags

SINGLE_DASH_MAKEFLAGS := $(filter -%,$(MAKEFLAGS))
SINGLE_DASH_MAKEFLAGS := $(filter-out --%,$(SINGLE_DASH_MAKEFLAGS))

ifneq ($(filter -n --dry-run --just-print --recon n,$(MAKEFLAGS))$(filter %n%,$(SINGLE_DASH_MAKEFLAGS)),)
$(error Refusing to run Makefile gates with dry-run enabled (MAKEFLAGS contains -n/--dry-run))
endif
ifneq ($(filter -i --ignore-errors i,$(MAKEFLAGS))$(filter %i%,$(SINGLE_DASH_MAKEFLAGS)),)
$(error Refusing to run Makefile gates with ignore-errors enabled (MAKEFLAGS contains -i/--ignore-errors))
endif
ifneq ($(filter -e --environment-overrides e,$(MAKEFLAGS))$(filter %e%,$(SINGLE_DASH_MAKEFLAGS)),)
$(error Refusing to run Makefile gates with environment overrides enabled (MAKEFLAGS contains -e/--environment-overrides))
endif

ifneq ($(origin ULTRA_LONG_TESTS),undefined)
$(error ULTRA_LONG_TESTS must not be set externally; edit Makefile to change the canonical ultra-long test list)
endif

ULTRA_LONG_TESTS := \
		ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix \
		ha::e2e_multi_node::e2e_multi_node_stress_planned_switchover_concurrent_sql \
		ha::e2e_multi_node::e2e_multi_node_stress_unassisted_failover_concurrent_sql \
		ha::e2e_multi_node::e2e_no_quorum_enters_failsafe_strict_all_nodes \
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
CARGO_GATE_TARGET_DIR := /tmp/pgtuskmaster_rust-target

TEST_TIMEOUT_SECS := 120
TEST_TIMEOUT_KILL_AFTER_SECS := 15
ifneq ($(origin TIMEOUT_BIN),undefined)
$(error TIMEOUT_BIN must not be set externally; edit Makefile to change timeout resolution)
endif
TIMEOUT_BIN := $(shell command -v timeout 2>/dev/null || command -v gtimeout 2>/dev/null)

TEST_PREFLIGHT_TIMEOUT_SECS := 180
TEST_PREFLIGHT_TIMEOUT_KILL_AFTER_SECS := 15

TEST_LONG_PER_TEST_TIMEOUT_SECS := 1800
TEST_LONG_PER_TEST_TIMEOUT_KILL_AFTER_SECS := 30

CHECK_TIMEOUT_SECS := 300
CHECK_TIMEOUT_KILL_AFTER_SECS := 15

LINT_DOCS_TIMEOUT_SECS := 120
LINT_DOCS_TIMEOUT_KILL_AFTER_SECS := 15

LINT_CLIPPY_TIMEOUT_SECS := 1200
LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS := 30

GATE_RUN_ID := $(shell date -u +%Y%m%dT%H%M%SZ)-$(shell printf '%s' "$$PPID")-$(shell printf '%s' "$$RANDOM")
GATE_EVIDENCE_DIR := $(CURDIR)/.ralph/evidence/gates/$(GATE_RUN_ID)

GATE_STEP := $(CURDIR)/tools/gate-step.sh

ifneq ($(origin TEST_TIMEOUT_SECS),file)
$(error TEST_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin TEST_TIMEOUT_KILL_AFTER_SECS),file)
$(error TEST_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin TEST_PREFLIGHT_TIMEOUT_SECS),file)
$(error TEST_PREFLIGHT_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin TEST_PREFLIGHT_TIMEOUT_KILL_AFTER_SECS),file)
$(error TEST_PREFLIGHT_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin TEST_LONG_PER_TEST_TIMEOUT_SECS),file)
$(error TEST_LONG_PER_TEST_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin TEST_LONG_PER_TEST_TIMEOUT_KILL_AFTER_SECS),file)
$(error TEST_LONG_PER_TEST_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin CHECK_TIMEOUT_SECS),file)
$(error CHECK_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin CHECK_TIMEOUT_KILL_AFTER_SECS),file)
$(error CHECK_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin LINT_DOCS_TIMEOUT_SECS),file)
$(error LINT_DOCS_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin LINT_DOCS_TIMEOUT_KILL_AFTER_SECS),file)
$(error LINT_DOCS_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin LINT_CLIPPY_TIMEOUT_SECS),file)
$(error LINT_CLIPPY_TIMEOUT_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS),file)
$(error LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS must not be set externally; edit Makefile to change the canonical gate timeout)
endif
ifneq ($(origin TIMEOUT_BIN),file)
$(error TIMEOUT_BIN must not be set externally; edit Makefile to change timeout resolution)
endif
ifneq ($(origin GATE_RUN_ID),file)
$(error GATE_RUN_ID must not be set externally; edit Makefile to change evidence run id generation)
endif
ifneq ($(origin GATE_EVIDENCE_DIR),file)
$(error GATE_EVIDENCE_DIR must not be set externally; edit Makefile to change gate evidence directory)
endif
ifneq ($(origin GATE_STEP),file)
$(error GATE_STEP must not be set externally; edit Makefile to change gate step runner path)
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
	"$(GATE_STEP)" --gate check --step check.cargo_check --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(CHECK_TIMEOUT_SECS)" --kill-after-secs "$(CHECK_TIMEOUT_KILL_AFTER_SECS)" -- env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" CARGO_TARGET_DIR="$(CARGO_GATE_TARGET_DIR)" cargo check --all-targets

check: guard-makeflags ensure-timeout

test: guard-makeflags ensure-timeout
	@set -euo pipefail; \
	if [ "$${RUST_TEST_THREADS:-}" = "1" ]; then echo "RUST_TEST_THREADS=1 is disallowed for make test (parallel must work). Fix parallel flakes instead." >&2; exit 1; fi; \
	list_file="$(GATE_EVIDENCE_DIR)/test/test-list.txt"; \
	mkdir -p "$$(dirname "$${list_file}")"; \
	echo "gate evidence: $(GATE_EVIDENCE_DIR)"; \
	"$(GATE_STEP)" --gate test --step test.preflight_list --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(TEST_PREFLIGHT_TIMEOUT_SECS)" --kill-after-secs "$(TEST_PREFLIGHT_TIMEOUT_KILL_AFTER_SECS)" -- \
		bash -c 'set -euo pipefail; env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" CARGO_TARGET_DIR="$(CARGO_GATE_TARGET_DIR)" cargo test --all-targets -- --list | tee "$$1" >/dev/null' -- "$${list_file}"; \
	"$(GATE_STEP)" --gate test --step test.preflight_validate_ultra_long --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(TEST_PREFLIGHT_TIMEOUT_SECS)" --kill-after-secs "$(TEST_PREFLIGHT_TIMEOUT_KILL_AFTER_SECS)" -- \
		bash -c 'set -euo pipefail; list_file="$$1"; dupes="$$(printf "%s\n" $(ULTRA_LONG_TESTS) | sort | uniq -d)"; if [ -n "$${dupes}" ]; then echo "Ultra-long test list contains duplicates: $${dupes}" >&2; exit 1; fi; for t in $(ULTRA_LONG_TESTS); do if ! awk -F": " '\''$$2=="test"{print $$1}'\'' "$${list_file}" | grep -Fx "$$t" >/dev/null; then echo "Ultra-long test not found (exact): $$t" >&2; exit 1; fi; match_count="$$(awk -F": " '\''$$2=="test"{print $$1}'\'' "$${list_file}" | grep -F "$$t" | wc -l | tr -d " ")"; if [ "$${match_count}" != "1" ]; then echo "Ultra-long skip token is ambiguous (substring matches $${match_count} tests): $$t" >&2; exit 1; fi; done' -- "$${list_file}"; \
	"$(GATE_STEP)" --gate test --step test.preflight_validate_default_nonempty --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(TEST_PREFLIGHT_TIMEOUT_SECS)" --kill-after-secs "$(TEST_PREFLIGHT_TIMEOUT_KILL_AFTER_SECS)" -- \
		bash -c 'set -euo pipefail; list_file="$$1"; all_file="$$(mktemp)"; ultra_file="$$(mktemp)"; trap '\''rm -f "$${all_file}" "$${ultra_file}"'\'' EXIT; awk -F": " '\''$$2=="test"{print $$1}'\'' "$${list_file}" | sort > "$${all_file}"; printf "%s\n" $(ULTRA_LONG_TESTS) | sort > "$${ultra_file}"; non_ultra_count="$$(grep -Fxv -f "$${ultra_file}" "$${all_file}" | wc -l | tr -d " ")"; if [ "$${non_ultra_count}" = "0" ]; then echo "default test suite would execute 0 tests after skipping ULTRA_LONG_TESTS; move at least one test back into make test" >&2; exit 1; fi' -- "$${list_file}"; \
	"$(GATE_STEP)" --gate test --step test.exec_default_suite --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(TEST_TIMEOUT_SECS)" --kill-after-secs "$(TEST_TIMEOUT_KILL_AFTER_SECS)" -- \
		env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" CARGO_TARGET_DIR="$(CARGO_GATE_TARGET_DIR)" cargo test --all-targets -- $(ULTRA_LONG_SKIP_ARGS)

test-long: guard-makeflags ensure-timeout
	@echo "test-long runs only ultra-long tests (evidence-backed passed runtime >= 3 minutes)."
	@echo "If one becomes short enough for regular development cycles, move it back into make test."
	@set -euo pipefail; \
	if [ -z "$(strip $(ULTRA_LONG_TESTS))" ]; then \
		echo "No ultra-long tests configured."; \
		exit 1; \
	fi; \
	list_file="$(GATE_EVIDENCE_DIR)/test-long/test-list.txt"; \
	mkdir -p "$$(dirname "$${list_file}")"; \
	echo "gate evidence: $(GATE_EVIDENCE_DIR)"; \
	"$(GATE_STEP)" --gate test-long --step test_long.preflight_list --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(TEST_PREFLIGHT_TIMEOUT_SECS)" --kill-after-secs "$(TEST_PREFLIGHT_TIMEOUT_KILL_AFTER_SECS)" -- \
		bash -c 'set -euo pipefail; env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" CARGO_TARGET_DIR="$(CARGO_GATE_TARGET_DIR)" cargo test --all-targets -- --list | tee "$$1" >/dev/null' -- "$${list_file}"; \
	"$(GATE_STEP)" --gate test-long --step test_long.preflight_validate_ultra_long --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(TEST_PREFLIGHT_TIMEOUT_SECS)" --kill-after-secs "$(TEST_PREFLIGHT_TIMEOUT_KILL_AFTER_SECS)" -- \
		bash -c 'set -euo pipefail; list_file="$$1"; dupes="$$(printf "%s\n" $(ULTRA_LONG_TESTS) | sort | uniq -d)"; if [ -n "$${dupes}" ]; then echo "Ultra-long test list contains duplicates: $${dupes}" >&2; exit 1; fi; for t in $(ULTRA_LONG_TESTS); do if ! awk -F": " '\''$$2=="test"{print $$1}'\'' "$${list_file}" | grep -Fx "$$t" >/dev/null; then echo "Ultra-long test not found (exact): $$t" >&2; exit 1; fi; match_count="$$(awk -F": " '\''$$2=="test"{print $$1}'\'' "$${list_file}" | grep -F "$$t" | wc -l | tr -d " ")"; if [ "$${match_count}" != "1" ]; then echo "Ultra-long test name is not unique (substring matches $${match_count} tests): $$t" >&2; exit 1; fi; done' -- "$${list_file}"; \
	for t in $(ULTRA_LONG_TESTS); do \
		"$(GATE_STEP)" --gate test-long --step "test_long.exec.$$t" --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(TEST_LONG_PER_TEST_TIMEOUT_SECS)" --kill-after-secs "$(TEST_LONG_PER_TEST_TIMEOUT_KILL_AFTER_SECS)" -- \
			env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" CARGO_TARGET_DIR="$(CARGO_GATE_TARGET_DIR)" cargo test --all-targets "$$t" -- --exact; \
	done

docs-lint: guard-makeflags ensure-timeout ensure-node
	@echo "gate evidence: $(GATE_EVIDENCE_DIR)"
	"$(GATE_STEP)" --gate docs-lint --step docs_lint.mermaid --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_DOCS_TIMEOUT_SECS)" --kill-after-secs "$(LINT_DOCS_TIMEOUT_KILL_AFTER_SECS)" -- \
		node ./tools/docs-mermaid-lint.mjs
	"$(GATE_STEP)" --gate docs-lint --step docs_lint.no_code_guard --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_DOCS_TIMEOUT_SECS)" --kill-after-secs "$(LINT_DOCS_TIMEOUT_KILL_AFTER_SECS)" -- \
		./tools/docs-architecture-no-code-guard.sh

lint.no_silent_errors: guard-makeflags ensure-timeout
	"$(GATE_STEP)" --gate lint --step lint.no_silent_errors --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_DOCS_TIMEOUT_SECS)" --kill-after-secs "$(LINT_DOCS_TIMEOUT_KILL_AFTER_SECS)" -- \
		./tools/lint-no-silent-errors.sh

lint: guard-makeflags ensure-timeout docs-lint lint.no_silent_errors
	"$(GATE_STEP)" --gate lint --step lint.clippy.all_targets --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_CLIPPY_TIMEOUT_SECS)" --kill-after-secs "$(LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS)" -- \
		env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" CARGO_TARGET_DIR="$(CARGO_GATE_TARGET_DIR)" cargo clippy --all-targets --all-features -- -D warnings
	# Strict restriction-lint pass for runtime library builds.
	"$(GATE_STEP)" --gate lint --step lint.clippy.lib_restrictions --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_CLIPPY_TIMEOUT_SECS)" --kill-after-secs "$(LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS)" -- \
		env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" CARGO_TARGET_DIR="$(CARGO_GATE_TARGET_DIR)" cargo clippy --lib --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	# Strict restriction-lint pass for test targets.
	"$(GATE_STEP)" --gate lint --step lint.clippy.tests_restrictions --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_CLIPPY_TIMEOUT_SECS)" --kill-after-secs "$(LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS)" -- \
		env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" CARGO_TARGET_DIR="$(CARGO_GATE_TARGET_DIR)" cargo clippy --tests --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	# Skeptical all-target guard so restrictions are enforced uniformly.
	"$(GATE_STEP)" --gate lint --step lint.clippy.all_targets_restrictions --run-id "$(GATE_RUN_ID)" --evidence-dir "$(GATE_EVIDENCE_DIR)" --timeout-bin "$(TIMEOUT_BIN)" --timeout-secs "$(LINT_CLIPPY_TIMEOUT_SECS)" --kill-after-secs "$(LINT_CLIPPY_TIMEOUT_KILL_AFTER_SECS)" -- \
		env CARGO_INCREMENTAL="$(CARGO_INCREMENTAL)" CARGO_TARGET_DIR="$(CARGO_GATE_TARGET_DIR)" cargo clippy --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented

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
