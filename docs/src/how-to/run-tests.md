# Run The Test Suite

This guide explains how contributors run the repository's test and validation gates.

The important context is that this project does not treat real-binary HA validation as optional. The test harness and Makefile expect actual etcd and PostgreSQL 16 binaries for substantial parts of the suite.

## Goal

Pick the right validation command for the change you made:

- `make check` for a fast compile gate
- `make test` for the default automated suite
- `make test-long` for the HA nextest suite behind the `ultra-long` profile
- `make test-long TESTS="ha_primary_crash_rejoin"` for focused HA runs
- `make lint` for docs and clippy enforcement

## Prerequisites

Make sure these are available in your environment:

- Rust toolchain with `cargo`
- `cargo-nextest`
- etcd real binary installed through the repository tooling
- PostgreSQL 16 real binaries installed through the repository tooling
- Docker and Docker Compose plugin for the HA validation targets
- permission to access the Docker daemon, not only a running daemon process

The repository itself points to helper installers for the real-binary prerequisites:

```bash
./tools/install-etcd.sh
./tools/install-postgres16.sh
```

## Fast Compile Gate

Run:

```bash
make check
```

This executes `cargo check --all-targets`.

Use it when you want a quick signal that the workspace still compiles before spending time on the longer suites.

## Default Test Suite

Run:

```bash
make test
```

This uses `cargo nextest run --workspace --all-targets --profile default --no-tests fail`.

That is the normal validation path for most code changes. The default profile excludes the long-running greenfield HA binaries selected by the `binary(ha_*)` naming rule, so those Docker-backed scenarios stay out of the default gate.

If you want the flattened per-test log files derived from nextest JUnit output after the run, use:

```bash
make test.convert-logs
```

## Long HA Validation

Run:

```bash
make test-long
```

This target runs the `ultra-long` nextest profile for the HA cucumber suite.

The `ultra-long` nextest profile selects the greenfield HA binaries and still runs them through normal nextest parallel scheduling. Those scenarios are expected to be parallel-safe. If a scenario only passes in serial, that is treated as a test bug rather than an accepted gate workaround.

Use it when your change can affect HA behavior or longer-running operational scenarios.

For a focused HA run, pass one or more `ha_*` test target names through `TESTS`:

```bash
make test-long TESTS="ha_primary_crash_rejoin"
make test-long TESTS="ha_primary_crash_rejoin ha_no_quorum_enters_failsafe"
```

In focused mode, `test-long` appends one `--test <name>` argument per selected target.

The Makefile wrapper itself is intentionally thin:

```bash
make test-long
```

expands to the `cargo nextest run` invocation for the `ultra-long` profile, and:

```bash
make test-long TESTS="ha_primary_crash_rejoin ha_no_quorum_enters_failsafe"
```

adds repeated `--test` selectors to that same invocation.

If you want the flattened per-test log files derived from nextest JUnit output after the run, use:

```bash
make test-long.convert-logs
```

The split targets are also available directly:

```bash
make test.nextest
make test.convert-logs
make test-long.nextest
make test-long.convert-logs
```

## Lint And Docs Validation

Run:

```bash
make lint
```

This is more than a clippy pass. The Makefile runs:

- docs Mermaid linting
- docs no-code guard checks
- silent-error linting
- multiple strict clippy passes, including unwrap/expect/panic/todo restrictions

## What The Tests Cover

The requested source files give a good picture of the suite layout:

- `tests/bdd_api_http.rs`: HTTP API behavior and auth-path tests
- `tests/bdd_state_watch.rs`: state-channel behavior contracts
- `tests/cli_binary.rs`: generic CLI help, debug, and node-config validation contracts
- `tests/nextest_config_contract.rs`: nextest profile routing expectations
- `src/worker_contract_tests.rs`: contract-style runtime and debug API expectations

The repository ships its supported HA end-to-end surface under `cucumber_tests/ha/`. That tree is intentionally independent from the ordinary `tests/` integration-test surface.

The current greenfield entrypoints are the `ha_*` nextest test targets plus `make test-long` as the canonical Makefile wrapper.

- `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.feature`
- `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.rs`
- `cucumber_tests/ha/features/replica_outage_keeps_primary_stable/replica_outage_keeps_primary_stable.feature`
- `cucumber_tests/ha/features/replica_outage_keeps_primary_stable/replica_outage_keeps_primary_stable.rs`
- `cucumber_tests/ha/features/two_node_outage_one_return_restores_quorum/two_node_outage_one_return_restores_quorum.feature`
- `cucumber_tests/ha/features/two_node_outage_one_return_restores_quorum/two_node_outage_one_return_restores_quorum.rs`
- `cucumber_tests/ha/features/full_cluster_outage_restore_quorum_then_converge/full_cluster_outage_restore_quorum_then_converge.feature`
- `cucumber_tests/ha/features/full_cluster_outage_restore_quorum_then_converge/full_cluster_outage_restore_quorum_then_converge.rs`
- `cucumber_tests/ha/features/replica_flap_keeps_primary_stable/replica_flap_keeps_primary_stable.feature`
- `cucumber_tests/ha/features/replica_flap_keeps_primary_stable/replica_flap_keeps_primary_stable.rs`
- `cucumber_tests/ha/features/planned_switchover_changes_primary_cleanly/planned_switchover_changes_primary_cleanly.feature`
- `cucumber_tests/ha/features/planned_switchover_changes_primary_cleanly/planned_switchover_changes_primary_cleanly.rs`
- `cucumber_tests/ha/features/targeted_switchover_promotes_requested_replica/targeted_switchover_promotes_requested_replica.feature`
- `cucumber_tests/ha/features/targeted_switchover_promotes_requested_replica/targeted_switchover_promotes_requested_replica.rs`

The shipped greenfield surface now covers primary crash and rejoin, replica outage, two-node outage with quorum restore, full-cluster outage with staged restore, repeated replica flap cycles, planned switchover, and targeted switchover. The old legacy HA/E2E harness has been removed, so new HA end-to-end coverage belongs in the cucumber runner.

The greenfield harness layout is:

- `cucumber_tests/ha/features/` for one feature directory plus tiny wrapper `.rs` per scenario file
- `cucumber_tests/ha/givens/three_node_plain/` for the static Docker compose fixture, static configs, static secrets, and Dockerfiles
- `cucumber_tests/ha/support/` for the independent runner, world, Docker CLI, Ryuk, `pgtm`, and `psql` plumbing
- `cucumber_tests/ha/runs/` for copied per-run input snapshots and captured artifacts
- `cucumber_tests/ha/harness.toml` for checked-in harness-local settings such as Docker binary discovery

That harness uses:

- Docker CLI orchestration with one unique Compose project per feature run
- Ryuk ownership keyed by the Compose project label
- `pgtm` as the cluster observer path
- `psql --dbname <conninfo>` with conninfo resolved by `pgtm`
- repo-local copied run workspaces so every feature run preserves its exact input files and artifacts
- Docker Compose `configs:` and `secrets:` entries for checked-in fixture material instead of bind-mounting host config/secret paths into the containers

Inside `three_node_plain`, the checked-in config layout is intentionally explicit rather than directory-noisy:

- `configs/node-a|node-b|node-c/runtime.toml` for node runtime configs
- `configs/observer/node-a.toml|node-b.toml|node-c.toml` for observer seed configs
- `configs/pg_hba.conf`, `configs/pg_ident.conf`, and `configs/tls/*` for shared material
- `secrets/*` for the checked-in test-only secret files consumed through Compose `secrets:`

## Real-Binary Expectations

The HA and Docker validation paths exercise real binaries and containers rather than pure mocks. That means you should expect:

- a more realistic environment than unit-only testing
- higher runtime cost than a lightweight mock suite
- failures that can come from system prerequisites, not only from Rust code

The longer runtime is why the HA binaries stay in `make test-long`. It is not a license to serialize them.

## Picking The Right Command

Use this rule of thumb:

- touched only Rust code and want a fast compile signal: `make check`
- changed behavior and need normal validation: `make test`
- touched HA flow or anything likely to affect longer-running HA behavior: `make test-long`
- want one or a few HA scenarios: `make test-long TESTS="ha_primary_crash_rejoin [ha_other_target ...]"`
- changed docs or want the full style and lint gate: `make lint`

## Troubleshooting

### `make test` fails before executing scenarios

Check whether `cargo-nextest` is installed and on `PATH`.

### HA scenarios fail because binaries are missing

Use the repository installer scripts for etcd and PostgreSQL 16, then rerun the target.

### `make test-long` fails immediately

Check Docker availability first:

- Docker daemon reachable from your current account
- permission to access `/var/run/docker.sock` or another configured Docker endpoint
- Docker Compose plugin installed

If the daemon is running but your account cannot reach it, `make test-long` now prints the raw `docker info` failure before exiting. On Linux, the common failure is:

```text
permission denied while trying to connect to the docker API at unix:///var/run/docker.sock
```

That means the current account cannot access the socket. Fix the account-to-daemon access first, for example by using the expected Docker group membership or by pointing `DOCKER_HOST` at a reachable daemon, then rerun the target.

### Lint fails on documentation-only work

Read the docs-lint output first. The Makefile treats docs validation as part of normal validation, not as an optional afterthought.
