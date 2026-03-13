// todo: remove the escaped heading backslash so this is valid markdown.
// todo: remove invented success-output examples unless they are clearly sourced from real tool output or replace them with generic, source-supported guidance.
\# Run the Test Suite

This guide shows you how to run the pgtuskmaster test suite, from quick validation to full end-to-end validation.

## Prerequisites

You need the following installed and on your PATH:

- Rust toolchain with cargo
- `cargo-nextest` (install with `cargo install cargo-nextest`)
- etcd binary (version compatible with etcd-client 0.14.1)
- PostgreSQL 16 binary
- Docker and Docker Compose plugin (for extended tests)
- `timeout` or `gtimeout` command

Install etcd and PostgreSQL 16 using the provided helpers:

```bash
./tools/install-etcd.sh
./tools/install-postgres16.sh
```

// todo: confirm the install destination for the helper scripts from source before claiming `/usr/local/bin` specifically.
Both scripts install binaries under `/usr/local/bin` by default.

## Quick Validation

Run a fast compilation check without executing tests:

```bash
make check
```

This command runs `cargo check --all-targets` with a 300-second timeout.

## Run Standard Test Suite

Execute the primary test suite:

```bash
make test
```

This runs `cargo nextest run` with:
- Workspace-wide coverage
- All targets (unit, integration, examples)
- Default profile
- No-fail-fast behavior
- Real binaries required (etcd, PostgreSQL 16)

The test harness allocates isolated network namespaces and dynamic ports, enabling parallel execution. Tests include:
- API behavior tests (`tests/bdd_api_http.rs`)
- Multi-node HA failover scenarios (`tests/ha_multi_node_failover.rs`)

## Run Extended Validation

Run comprehensive validation including long HA scenarios and Docker-based smoke tests:

```bash
make test-long
```

This command:
1. Runs ultra-long HA scenarios (timeout: 180-300 seconds per scenario)
2. Validates Docker Compose configurations (timeout: 120 seconds)
3. Executes smoke tests for single-node and cluster deployments (timeout: 600-900 seconds)

The suite requires active Docker daemon and may take 15-30 minutes on modest hardware.

## Run Specific Test Subsets

Use cargo-nextest filters to run specific tests:

```bash
# Run only HA tests
cargo nextest run --test 'ha_*'

# Run only API BDD tests
cargo nextest run --test bdd_api_http

# Run specific scenario by name pattern
cargo nextest run -- 'multi_node_unassisted_failover'
```

## Expected Outcomes

All commands should complete with exit code 0. Test output shows:

- Number of tests run
- Pass/fail counts
- Execution time per test
- Any failures with detailed backtraces

On success, the invoked command exits with status 0 and nextest reports the executed tests plus any failures.

## Clean Up

The build system uses a shared target directory at `/tmp/pgtuskmaster_rust-target` to reduce disk usage. Remove it manually if needed:

```bash
rm -rf /tmp/pgtuskmaster_rust-target
```

[diagram about test execution flow showing the three make targets and their relationships:
- `make check` is fastest, only compilation
- `make test` runs nextest suite against real binaries
- `make test-long` extends test with Docker Compose and smoke tests
- All targets require real etcd and PostgreSQL 16 binaries
- Test harness provides namespace isolation and port allocation]
