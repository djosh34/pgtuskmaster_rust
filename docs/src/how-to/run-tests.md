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

The HA harness now prepares per-run copied inputs with per-node `faults/` directories bind-mounted into `/var/lib/pgtuskmaster/faults` for explicit fault control.

Run specific HA scenarios:

```bash
make test-long TESTS="ha_replica_stopped_primary_stays_primary"
```

Multiple scenarios:

```bash
make test-long TESTS="ha_replica_stopped_primary_stays_primary ha_primary_killed_then_rejoins_as_replica ha_targeted_switchover_to_degraded_replica_is_rejected"
```

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
