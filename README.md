# PGTuskMaster

PGTuskMaster is a PostgreSQL high-availability controller that runs alongside PostgreSQL instances, watches local database state, publishes cluster state through etcd, and exposes an HTTP API for observation and operator actions. It combines local PostgreSQL inspection, DCS-backed cluster state, and an explicit HA decision engine to make cluster behavior observable through tutorials, how-to guides, explanations, and reference pages.

## Quickstart

### Run local docs

```bash
make docs-serve
```

The mdBook will be available at `http://127.0.0.1:3000`.

Docs validation uses a pinned Mermaid Node dependency. If `make docs-lint` reports it missing, install it with `./tools/install-docs-node-deps.sh`.

Start with the [docs overview](docs/src/overview.md), then use the chapter entry pages for [tutorials](docs/src/tutorial/overview.md), [how-to guides](docs/src/how-to/overview.md), [explanation](docs/src/explanation/overview.md), and [reference](docs/src/reference/overview.md).

### Run local cluster examples

Three-node HA cluster:

```bash
docker compose -f docker/compose.yml up -d --build
```

This shipped HA stack is TLS-enabled. Use the operator config in `docker/pgtm.toml` or the docs-owned seed configs in `docs/examples/docker-cluster-node-*.toml` when you talk to it with `pgtm`.

Inspect the running stack:

```bash
docker compose -f docker/compose.yml ps
pgtm -c docker/pgtm.toml status
docker compose -f docker/compose.yml down
```

The compose file publishes node APIs on `18081`, `18082`, and `18083`, and PostgreSQL on `15001`, `15002`, and `15003`. The first run can take noticeably longer because Docker needs to build `pgtuskmaster-local:compose`.

For guided walkthroughs, see [First HA Cluster](docs/src/tutorial/first-ha-cluster.md) and [Check Cluster Health](docs/src/how-to/check-cluster-health.md).

## License

All rights reserved 'Joshua Azimullah'.
