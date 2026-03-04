MDBOOK := .tools/mdbook/bin/mdbook
MDBOOK_MERMAID := .tools/mdbook/bin/mdbook-mermaid

.PHONY: check test test-long lint docs-build docs-serve docs-hygiene docs-lint ensure-mdbook ensure-mdbook-mermaid ensure-timeout

ULTRA_LONG_TESTS :=
ULTRA_LONG_SKIP_ARGS := $(foreach t,$(ULTRA_LONG_TESTS),--skip $(t))

TEST_TIMEOUT_SECS ?= 120
TEST_TIMEOUT_KILL_AFTER_SECS ?= 15
TIMEOUT_BIN := $(shell command -v timeout 2>/dev/null || command -v gtimeout 2>/dev/null)

ensure-mdbook:
	@test -x "$(MDBOOK)" || (echo "missing mdBook binary: run ./tools/install-mdbook.sh" >&2; exit 1)

ensure-mdbook-mermaid: ensure-mdbook
	@test -x "$(MDBOOK_MERMAID)" || (echo "missing mdbook-mermaid binary: run ./tools/install-mdbook-mermaid.sh" >&2; exit 1)

ensure-timeout:
	@test -n "$(TIMEOUT_BIN)" || (echo "missing timeout binary (install coreutils). Need either 'timeout' (Linux) or 'gtimeout' (macOS)." >&2; exit 1)

check:
	cargo check --all-targets

test: ensure-timeout
	@if [ "$${RUST_TEST_THREADS:-}" = "1" ]; then echo "RUST_TEST_THREADS=1 is disallowed for make test (parallel must work). Fix parallel flakes instead." >&2; exit 1; fi
	"$(TIMEOUT_BIN)" --kill-after="$(TEST_TIMEOUT_KILL_AFTER_SECS)s" "$(TEST_TIMEOUT_SECS)s" cargo test --all-targets -- $(ULTRA_LONG_SKIP_ARGS)

test-long:
	@echo "test-long runs only ultra-long tests (evidence-backed passed runtime >= 3 minutes)."
	@echo "If one becomes short enough for regular development cycles, move it back into make test."
	@set -e; \
	if [ -z "$(strip $(ULTRA_LONG_TESTS))" ]; then \
		echo "No ultra-long tests configured."; \
		exit 0; \
	fi; \
	for t in $(ULTRA_LONG_TESTS); do \
		cargo test --all-targets "$$t"; \
	done

docs-lint:
	./tools/docs-architecture-no-code-guard.sh

lint: docs-lint
	cargo clippy --all-targets --all-features -- -D warnings
	# Strict restriction-lint pass for runtime library builds.
	cargo clippy --lib --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	# Strict restriction-lint pass for test targets.
	cargo clippy --tests --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	# Skeptical all-target guard so restrictions are enforced uniformly.
	cargo clippy --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented

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
