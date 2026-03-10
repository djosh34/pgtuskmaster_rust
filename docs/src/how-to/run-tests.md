# Run The Test Suite

This guide explains how contributors run the repository's test and validation gates.

The important context is that this project does not treat real-binary HA validation as optional. The test harness and Makefile expect actual etcd and PostgreSQL 16 binaries for substantial parts of the suite.

## Goal

Pick the right validation command for the change you made:

- `make check` for a fast compile gate
- `make test` for the default automated suite
- `make test-long` for the longest HA and Docker validation path
- `make test-cucumber-ha` for the greenfield cucumber Docker HA harness
- `make test-cucumber-ha-primary-crash-rejoin` for the first greenfield cucumber HA feature
- `make lint` for docs and clippy enforcement

## Prerequisites

Make sure these are available in your environment:

- Rust toolchain with `cargo`
- `cargo-nextest`
- `timeout` or `gtimeout`
- etcd real binary installed through the repository tooling
- PostgreSQL 16 real binaries installed through the repository tooling
- Docker and Docker Compose plugin for the longest validation targets

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

This executes `cargo check --all-targets` through the Makefile gate wrapper.

Use it when you want a quick signal that the workspace still compiles before spending time on the longer suites.

## Default Test Suite

Run:

```bash
make test
```

This uses `cargo nextest run --workspace --all-targets --profile default --no-fail-fast --no-tests fail`.

That is the normal validation path for most code changes. The default profile excludes the long-running HA integration-test binaries selected by the `tests/ha_*.rs` layout rule, so new scenarios added to those binaries automatically stay out of the default gate.

## Long HA And Docker Validation

Run:

```bash
make test-long
```

This target is intentionally heavier than `make test`. It runs:

1. the `ultra-long` nextest profile
2. Docker Compose config validation
3. single-node Docker smoke coverage
4. cluster Docker smoke coverage

The `ultra-long` nextest profile selects the `tests/ha_*.rs` integration-test binaries and still runs them through normal nextest parallel scheduling. Those HA scenarios use isolated ports, namespaces, and working directories, so serial-only behavior is treated as a bug in the tests rather than as an accepted workaround in the gate config.

That long HA bucket now includes PostgreSQL data-plane chaos coverage in addition to etcd and API-path partitions. The `ha_partition_isolation` binary exercises a case where the primary's advertised PostgreSQL endpoint is blocked through the harness pg proxy, replicas prove that streaming was interrupted, and the cluster must heal back to one converged primary afterward.

The `ha_multi_node_failover` binary also covers restart and churn paths that are easy to miss in single-transition tests: primary HA-runtime restarts, repeated leadership changes in one scenario, failover with one replica already degraded, safe rejection of targeted switchovers to ineligible members, explicit `pg_basebackup` clone failure with recovery after the blocker is removed, and explicit `pg_rewind` failure where the former primary must fall back to basebackup before it can rejoin.

Use it when your change can affect HA behavior, Docker packaging, or longer-running operational scenarios.

## Lint And Docs Validation

Run:

```bash
make lint
```

This is more than a clippy pass. The Makefile wires in:

- docs Mermaid linting
- docs no-code guard checks
- silent-error linting
- multiple strict clippy passes, including unwrap/expect/panic/todo restrictions

## What The Tests Cover

The requested source files give a good picture of the suite layout:

- `tests/bdd_api_http.rs`: HTTP API behavior and auth-path tests
- `tests/ha_multi_node_failover.rs`: multi-node HA scenario entrypoints
- `tests/ha_partition_isolation.rs`: long-running partition and PostgreSQL-path chaos scenarios
- `tests/ha/support/multi_node.rs`: scenario orchestration, convergence, restart, churn, degraded failover, and switchover flows
- `tests/ha/support/partition.rs`: partition orchestration, pg-proxy fault injection, and convergence assertions
- `tests/ha/support/observer.rs`: split-brain and HA observation checks
- `src/worker_contract_tests.rs`: contract-style runtime and debug API expectations

The repository also now ships a separate greenfield HA end-to-end surface under `cucumber_tests/ha/`. That tree is intentionally independent from the legacy `tests/ha*` harness so the old HA harness can be deleted later without taking the new cucumber runner with it.

The current greenfield entrypoints are:

- `make test-cucumber-ha`
- `make test-cucumber-ha-primary-crash-rejoin`
- `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.feature`
- `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.rs`

Right now the greenfield suite contains one shipped feature, so `make test-cucumber-ha` currently runs the same single wrapper as `make test-cucumber-ha-primary-crash-rejoin`. The targets stay split intentionally: the suite target is the stable suite entrypoint, while the feature target stays pinned to the specific scenario wrapper.

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

The HA harness allocates dynamic ports, builds isolated namespaces, and starts real etcd/PostgreSQL processes. That means you should expect:

- a more realistic environment than unit-only testing
- higher runtime cost than a lightweight mock suite
- failures that can come from system prerequisites, not only from Rust code

The longer runtime is why the HA binaries stay in `make test-long`. It is not a license to serialize them.

## Picking The Right Command

Use this rule of thumb:

- touched only Rust code and want a fast compile signal: `make check`
- changed behavior and need normal validation: `make test`
- touched HA flow, Docker assets, or anything likely to affect system behavior over time: `make test-long`
- changed docs or want the full style and lint gate: `make lint`

## Troubleshooting

### `make test` fails before executing scenarios

Check whether `cargo-nextest` is installed and on `PATH`.

### HA scenarios fail because binaries are missing

Use the repository installer scripts for etcd and PostgreSQL 16, then rerun the target.

### `make test-long` fails immediately

Check Docker availability first:

- Docker daemon reachable
- Docker Compose plugin installed

### Lint fails on documentation-only work

Read the docs-lint output first. The Makefile treats docs validation as a real gate, not as an optional afterthought.
