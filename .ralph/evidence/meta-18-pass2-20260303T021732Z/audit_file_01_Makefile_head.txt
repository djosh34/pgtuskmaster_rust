check:
	cargo check --all-targets

test:
	cargo test --all-targets

test-real:
	PGTUSKMASTER_REQUIRE_REAL_BINARIES=1 cargo test --all-targets test_harness::pg16::tests::spawn_pg16_requires_binaries_and_spawns
	PGTUSKMASTER_REQUIRE_REAL_BINARIES=1 cargo test --all-targets test_harness::etcd3::tests::spawn_etcd3_requires_binary_and_spawns
	PGTUSKMASTER_REQUIRE_REAL_BINARIES=1 cargo test --all-targets pginfo::worker::tests::step_once_transitions_unreachable_to_primary_and_tracks_wal_and_slots
	PGTUSKMASTER_REQUIRE_REAL_BINARIES=1 cargo test --all-targets pginfo::worker::tests::step_once_maps_replica_when_polling_standby
	PGTUSKMASTER_REQUIRE_REAL_BINARIES=1 cargo test --all-targets process::worker::tests::real_
	PGTUSKMASTER_REQUIRE_REAL_BINARIES=1 cargo test --all-targets ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix

test-bdd:
	cargo test --test bdd_state_watch --test bdd_api_http

lint:
	cargo clippy --all-targets --all-features -- -D warnings
	# Strict restriction-lint pass for runtime library builds.
	cargo clippy --lib --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	# Strict restriction-lint pass for test targets.
	cargo clippy --tests --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
	# Skeptical all-target guard so restrictions are enforced uniformly.
	cargo clippy --all-targets --all-features -- -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::todo -D clippy::unimplemented
