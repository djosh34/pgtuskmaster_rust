MDBOOK := .tools/mdbook/bin/mdbook
MDBOOK_MERMAID := .tools/mdbook/bin/mdbook-mermaid

SHELL := /usr/bin/env bash

.PHONY: check test test.nextest test.convert-logs test-long test-long.nextest test-long.convert-logs lint lint.no_silent_errors docs-build docs-serve docs-hygiene docs-lint ensure-mdbook ensure-mdbook-mermaid ensure-node ensure-docs-node-deps ensure-nextest

# The workspace mount this repo typically lives on can exhibit intermittent
# linker/archive flake with incremental artifacts. Disable incremental builds by
# default for deterministic `make` gates; override with `CARGO_INCREMENTAL=1`
# if you explicitly want it.
CARGO_INCREMENTAL ?= 0
CARGO_GATE_TARGET_DIR := /tmp/pgtuskmaster_rust-target
ifeq ($(CARGO_INCREMENTAL),1)
CARGO_INCREMENTAL_BOOL := true
else
CARGO_INCREMENTAL_BOOL := false
endif

TESTS ?=
TEST_LONG_TARGET_ARGS := --test ha
TEST_LONG_SELECTION_ARGS = $(TEST_LONG_TARGET_ARGS) $(if $(strip $(TESTS)),-- $(strip $(TESTS)) --exact)

ensure-mdbook:
	@test -x "$(MDBOOK)" || (echo "missing mdBook binary: run ./tools/install-mdbook.sh" >&2; exit 1)

ensure-mdbook-mermaid: ensure-mdbook
	@test -x "$(MDBOOK_MERMAID)" || (echo "missing mdbook-mermaid binary: run ./tools/install-mdbook-mermaid.sh" >&2; exit 1)

ensure-node:
	@command -v node >/dev/null 2>&1 || (echo "missing node binary (required for Mermaid docs lint)" >&2; exit 1)

ensure-docs-node-deps: ensure-node
	@test -f "$(CURDIR)/tools/node_modules/mermaid/package.json" || (echo "missing docs Mermaid npm dependency: run ./tools/install-docs-node-deps.sh" >&2; exit 1)

ensure-nextest:
	@command -v cargo-nextest >/dev/null 2>&1 || (echo "missing cargo-nextest binary: run ./tools/install-cargo-nextest.sh" >&2; exit 1)

check:
	cargo check --all-targets --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)"

test: ensure-nextest
	@set -euo pipefail; \
	status=0; \
	$(MAKE) test.nextest || status="$$?"; \
	$(MAKE) test.convert-logs || true; \
	exit "$$status"

test.nextest: ensure-nextest
	cargo nextest run --workspace --all-targets --profile default --no-tests fail --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)"

test.convert-logs:
	python3 ./tools/export-nextest-junit-logs.py ./target/nextest/default/junit.xml ./target/nextest/default/logs

test-long: ensure-nextest
	@set -euo pipefail; \
	echo 'usage: make test-long [TESTS="ha_test_one"|TESTS="ha_test_one ha_test_two"]'; \
	status=0; \
	$(MAKE) test-long.nextest TESTS='$(TESTS)' || status="$$?"; \
	$(MAKE) test-long.convert-logs || true; \
	exit "$$status"

test-long.nextest: ensure-nextest
	cargo nextest run --workspace --profile ultra-long --no-tests fail --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)" $(TEST_LONG_SELECTION_ARGS)

test-long.convert-logs:
	python3 ./tools/export-nextest-junit-logs.py ./target/nextest/ultra-long/junit.xml ./target/nextest/ultra-long/logs

docs-lint: ensure-docs-node-deps
	node ./tools/docs-mermaid-lint.mjs
	./tools/docs-architecture-no-code-guard.sh

lint.no_silent_errors:
	./tools/lint-no-silent-errors.sh

lint: docs-lint lint.no_silent_errors
	cargo clippy --all-targets --all-features --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)" -- -D warnings
	cargo clippy --lib --all-features --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)" -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	cargo clippy --tests --all-features --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)" -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	cargo clippy --all-targets --all-features --target-dir "$(CARGO_GATE_TARGET_DIR)" --config "build.incremental=$(CARGO_INCREMENTAL_BOOL)" -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented

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
