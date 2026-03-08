# PGTuskMaster

PGTuskMaster is a PostgreSQL high-availability controller that runs alongside PostgreSQL instances, watches local database state, publishes cluster state through etcd, and exposes an HTTP API for observation and operator actions. It combines local PostgreSQL inspection, DCS-backed cluster state, and an explicit HA decision engine to provide operator-visible runtime that explains its decisions and behavior.

## Quickstart

### Run Local Docs

Serve the documentation locally:

```bash
make docs-serve
```

The mdBook will be available at `http://127.0.0.1:3000`.

View the [full documentation set](docs/src/SUMMARY.md) for tutorials, how-to guides, and reference.

### Run Local Cluster Examples

**Single-node cluster:**

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.single.yml up -d --build
```

API available at `http://127.0.0.1:18080`, PostgreSQL at `127.0.0.1:15432`.

**Three-node HA cluster:**

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d --build
```

API available at ports `18081`, `18082`, `18083`; PostgreSQL at `15433`, `15434`, `15435`.

See [Single-Node Setup](docs/src/tutorial/single-node-setup.md) and [First HA Cluster](docs/src/tutorial/first-ha-cluster.md) for detailed walkthroughs.

## License

All rights reserved 'Joshua Azimullah'.
