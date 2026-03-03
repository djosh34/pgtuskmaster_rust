MDBOOK := .tools/mdbook/bin/mdbook
MDBOOK_MERMAID := .tools/mdbook/bin/mdbook-mermaid

.PHONY: check test test-real test-bdd lint docs-build docs-serve docs-hygiene docs-lint ensure-mdbook ensure-mdbook-mermaid

ensure-mdbook:
	@test -x "$(MDBOOK)" || (echo "missing mdBook binary: run ./tools/install-mdbook.sh" >&2; exit 1)

ensure-mdbook-mermaid: ensure-mdbook
	@test -x "$(MDBOOK_MERMAID)" || (echo "missing mdbook-mermaid binary: run ./tools/install-mdbook-mermaid.sh" >&2; exit 1)

check:
	cargo check --all-targets

test:
	cargo test --all-targets

test-real:
	cargo test --all-targets test_harness::pg16::tests::spawn_pg16_requires_binaries_and_spawns
	cargo test --all-targets test_harness::etcd3::tests::spawn_etcd3_requires_binary_and_spawns
	cargo test --all-targets pginfo::worker::tests::step_once_transitions_unreachable_to_primary_and_tracks_wal_and_slots
	cargo test --all-targets pginfo::worker::tests::step_once_maps_replica_when_polling_standby
	cargo test --all-targets process::worker::tests::real_
	cargo test --all-targets dcs::etcd_store::tests::etcd_store_round_trips_write_delete_and_events
	cargo test --all-targets dcs::etcd_store::tests::step_once_consumes_real_etcd_watch_path_without_mocking
	cargo test --all-targets dcs::etcd_store::tests::step_once_marks_store_unhealthy_on_real_decode_failure
	cargo test --all-targets ha::e2e_multi_node::e2e_multi_node_unassisted_failover_sql_consistency
	cargo test --all-targets ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix

test-bdd:
	cargo test --all-targets -- --include-ignored

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
