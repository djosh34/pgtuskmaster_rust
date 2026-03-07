# pgtuskmaster_rust

`pgtuskmaster_rust` is a local high-availability controller for PostgreSQL. Each node supervises one PostgreSQL instance, watches shared cluster state in etcd, and decides whether the local database should run as primary, replica, recovering follower, or a conservative safety mode such as fail-safe or fencing.

The project is deliberately biased toward safe role changes. When the cluster view is healthy, the node can bootstrap, follow, promote, and handle planned switchover or unplanned failover. When coordination trust drops, it slows down or refuses risky actions instead of guessing.

## What Problem This Solves

Running PostgreSQL in an HA topology is not just about promoting a standby. You need each node to make the same decision from the same shared state, expose enough control-plane information to understand what it is doing, and fail closed when trust in coordination is weak.

This repository provides that control loop plus a checked-in container-first lab path so you can prove the runtime, API surface, PostgreSQL exposure, and docs all agree before you move into deeper operator work.

## Get Started

The supported first path is the checked-in Docker Compose lab.

1. Copy [`.env.docker.example`](.env.docker.example) to `.env.docker`.
2. Point the secret-file variables at three real, readable local secret files.
3. Validate the tracked Compose setup.
4. Bring up the single-node stack.
5. Run the smoke validation flow.

```console
cp .env.docker.example .env.docker
make docker-compose-config
make docker-up
make docker-smoke-single
```

If you want the detailed walkthrough before running commands, start with [Quick Start](docs/src/quick-start/index.md).

## Repo Entry Points

- [Quick Start](docs/src/quick-start/index.md): container-first first run, validation signals, and smoke flow
- [Operator Guide](docs/src/operator/index.md): deployment, configuration, observability, and troubleshooting
- [Start Here](docs/src/start-here/index.md): project mental model and reading paths through the docs
- [Reading Paths And Book Map](docs/src/start-here/docs-map.md): choose the right chapter family quickly
- [Summary](docs/src/SUMMARY.md): full mdBook table of contents
- [`Makefile`](Makefile): canonical commands for checks, tests, docs, and Docker workflows

The most useful early commands are:

- `make check`
- `make test`
- `make lint`
- `make docs-build`
- `make docs-serve`
- `make docker-compose-config`
- `make docker-up`
- `make docker-down`
- `make test-long`

## Learn More

Use the book for depth instead of treating this README as the manual:

- [Quick Start](docs/src/quick-start/index.md) for the shortest supported first run
- [Operator Guide](docs/src/operator/index.md) for day-2 operations and troubleshooting
- [System Lifecycle](docs/src/lifecycle/index.md) for HA phases and transitions
- [Architecture Assurance](docs/src/assurance/index.md) for safety arguments, assumptions, and limits
- [Contributors](docs/src/contributors/index.md) for codebase internals and development workflow

## License

All Rights Reserved Joshua Azimullah
