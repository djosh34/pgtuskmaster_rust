# PGTuskMaster

PGTuskMaster is a PostgreSQL high-availability controller that runs alongside PostgreSQL instances, watches local database state, publishes cluster state through etcd, and exposes an HTTP API for observation and operator actions. It combines local PostgreSQL inspection, DCS-backed cluster state, and an explicit HA decision engine to make cluster behavior observable through tutorials, how-to guides, explanations, and reference pages.

## Quickstart

### Run local docs

```bash
make docs-serve
```

The mdBook will be available at `http://127.0.0.1:3000`.

Start with the [docs overview](docs/src/overview.md), then use the chapter entry pages for [tutorials](docs/src/tutorial/overview.md), [how-to guides](docs/src/how-to/overview.md), [explanation](docs/src/explanation/overview.md), and [reference](docs/src/reference/overview.md).

### Run local cluster examples

Single-node cluster:

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.single.yml up -d --build
```

API available at `http://127.0.0.1:18080`, PostgreSQL at `127.0.0.1:15432`.

Three-node HA cluster:

```bash
make docker-up-cluster
```

The persistent cluster flow uses `tools/docker/cluster.sh` under the hood. It prints the effective compose project, env file, API URLs, debug URLs, PostgreSQL endpoints, leader, and each node's current HA role once readiness succeeds.

By default the cluster targets read `.env.docker`. If you want the stable example ports from `.env.docker.example`, either copy it to `.env.docker` or run the script directly:

```bash
tools/docker/cluster.sh up --env-file .env.docker.example
tools/docker/cluster.sh status --env-file .env.docker.example
tools/docker/cluster.sh down --env-file .env.docker.example
```

The example env file publishes node APIs on `18081`, `18082`, and `18083`, and PostgreSQL on `15433`, `15434`, and `15435`. The first run can take noticeably longer because Docker needs to build `pgtuskmaster:local`.

For guided walkthroughs, see [Single-Node Setup](docs/src/tutorial/single-node-setup.md), [First HA Cluster](docs/src/tutorial/first-ha-cluster.md), and [Check Cluster Health](docs/src/how-to/check-cluster-health.md).

## License

All rights reserved 'Joshua Azimullah'.
