MDBOOK := .tools/mdbook/bin/mdbook
MDBOOK_MERMAID := .tools/mdbook/bin/mdbook-mermaid

.PHONY: check test test-long lint docs-build docs-serve docs-hygiene docs-lint ensure-mdbook ensure-mdbook-mermaid

ULTRA_LONG_TESTS := ha::e2e_multi_node::e2e_multi_node_stress_no_quorum_fencing_with_concurrent_sql
ULTRA_LONG_SKIP_ARGS := $(foreach t,$(ULTRA_LONG_TESTS),--skip $(t))

ensure-mdbook:
	@test -x "$(MDBOOK)" || (echo "missing mdBook binary: run ./tools/install-mdbook.sh" >&2; exit 1)

ensure-mdbook-mermaid: ensure-mdbook
	@test -x "$(MDBOOK_MERMAID)" || (echo "missing mdbook-mermaid binary: run ./tools/install-mdbook-mermaid.sh" >&2; exit 1)

check:
	cargo check --all-targets

test:
	cargo test --all-targets -- $(ULTRA_LONG_SKIP_ARGS)

test-long:
	@echo "test-long runs only ultra-long tests (evidence-backed passed runtime >= 3 minutes)."
	@echo "If one becomes short enough for regular development cycles, move it back into make test."
	@set -e; \
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
