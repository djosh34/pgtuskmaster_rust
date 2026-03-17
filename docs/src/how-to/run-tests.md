# Run the Test Suite

Execute the project's test and validation gates.

## Prerequisites

Install required binaries:

```bash
./tools/install-etcd.sh
./tools/install-postgres16.sh
```

Also required:

- Rust toolchain with cargo
- cargo-nextest
- Docker and Docker Compose plugin
- Permission to access Docker daemon

## Fast compile check

For quick compilation feedback:

```bash
make check
```

## Default test suite

For normal validation of most code changes:

```bash
make test
```

Convert nextest JUnit output to per-test logs:

```bash
make test.convert-logs
```

## HA validation tests

For changes affecting HA behavior:

```bash
make test-long
```

The HA harness now prepares a per-run materialized workspace assembled from shared fixture assets plus rendered topology-specific files, with per-node `faults/` directories bind-mounted into `/var/lib/pgtuskmaster/faults` for explicit fault control.

Stable-primary HA steps now validate a writable PostgreSQL route before recording aliases such as `"initial_primary"` or `"current_primary"`. Follow-up proof writes through those aliases reuse the same validated target instead of resolving a fresh connection path later, so a passed stable-primary wait means the next proof-row insert is checking the same writable endpoint.

Run specific HA scenarios:

```bash
make test-long TESTS="ha_replica_faults_keep_cluster_healthy"
```

Multiple scenarios:

```bash
make test-long TESTS="ha_replica_faults_keep_cluster_healthy ha_primary_faults_fail_over_then_recover ha_operator_switchovers"
```

The current merged HA feature inventory is:

- `ha_replica_faults_keep_cluster_healthy`
- `ha_primary_faults_fail_over_then_recover`
- `ha_quorum_loss_and_dcs_loss`
- `ha_rejoin_and_restart_recovery`
- `ha_operator_switchovers`

Convert HA logs:

```bash
make test-long.convert-logs
```

Individual targets:

- `make test.nextest`
- `make test.convert-logs`
- `make test-long.nextest`
- `make test-long.convert-logs`

## Lint and documentation checks

For documentation changes or full style validation:

```bash
make lint
```

This runs:

- Mermaid diagram linting
- Documentation no-code guard checks
- Silent-error linting
- Strict clippy passes (no unwrap/expect/panic/todo)

## Picking the right command

| Change type | Command |
|-------------|---------|
| Rust code only, quick compile check | `make check` |
| General behavior changes | `make test` |
| HA behavior changes | `make test-long` |
| Specific HA scenarios | `make test-long TESTS="..."` |
| Documentation or full validation | `make lint` |

## Troubleshooting

### `make test` fails before executing scenarios

- Verify `cargo-nextest` is installed and on PATH

### HA scenarios fail because binaries are missing

```bash
./tools/install-etcd.sh
./tools/install-postgres16.sh
```

### `make test-long` fails immediately

Check Docker access:

```bash
docker info
```

Linux permission denied errors mean the current account cannot access the Docker socket. Add user to Docker group or set `DOCKER_HOST` to a reachable daemon endpoint.

### Lint fails on documentation-only work

- Review docs-lint output first
- Documentation validation is required, not optional
