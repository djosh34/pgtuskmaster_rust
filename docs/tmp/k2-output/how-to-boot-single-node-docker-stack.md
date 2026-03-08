# How to boot the single-node docker stack and confirm it is healthy

Run one script to start a complete single-node stack and wait until every component reports healthy.

## Prerequisites

- curl
- Docker with Compose support

## Start the stack

From the repository root run:

```bash
tools/docker/smoke-single.sh
```

The script chooses free host ports, builds the containers, and does not return until the stack passes all health checks.

## What the script verifies

Before the script exits it confirms:

- HTTP 200 from the API endpoints `/ha/state` and `/debug/verbose` on the dynamically chosen host port.
- A TCP listener on the dynamically chosen PostgreSQL host port.
- `psql` can connect via Unix socket inside the `node-a` container and execute `select 1`.
- `etcdctl endpoint health` reports healthy from inside the `etcd` container.

## If the script exits early

The script prints each check. If it fails early, the output shows which step failed. The script always removes the compose project and a temporary directory containing generated files on exit.

## Related reference and explanation

- [Runtime configuration](../reference/runtime-config.md)
- [HTTP API](../reference/http-api.md)
- [pgtuskmaster](../reference/pgtuskmaster.md)
- [Startup versus steady state](../explanation/startup-versus-steady-state.md)
