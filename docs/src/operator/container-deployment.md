# Container Deployment

This is the primary operator path for the repository. The checked-in Docker assets are meant to be the canonical first deployment shape, not a side example.

## What ships

The repo includes:

- `docker/Dockerfile.prod`: minimal runtime image for `pgtuskmaster` nodes on top of `postgres:16-bookworm`
- `docker/Dockerfile.dev`: development image with Rust, cargo, Node, and local iteration tooling
- `docker/compose/docker-compose.single.yml`: `etcd` + `node-a`
- `docker/compose/docker-compose.cluster.yml`: `etcd` + `node-a` + `node-b` + `node-c`
- `docker/configs/**`: tracked runtime TOML and PostgreSQL config inputs
- `docker/secrets/*.example`: placeholder secret files only

The important point is that these assets are operational, not illustrative. The quick-start path, smoke flows, and long test gate all depend on them staying truthful.

## Prepare the local environment

1. create `.env.docker` from `.env.docker.example`
2. replace the example secret file paths with local non-example files
3. write local values into those files

For the checked-in lab stacks, readable non-empty secrets are sufficient because PostgreSQL network auth inside the private bridge stays trust-based. For anything beyond the lab, generate strong real passwords and align `pg_hba` with the identities in the runtime config.

The env file controls:

- `PGTUSKMASTER_IMAGE`
- `ETCD_IMAGE`
- `PGTM_SECRET_SUPERUSER_FILE`
- `PGTM_SECRET_REPLICATOR_FILE`
- `PGTM_SECRET_REWINDER_FILE`
- the single-node published ports
- the per-node published ports for the three-node cluster

## Bring up the single-node stack

```console
make docker-up
```

Useful checks after startup:

```console
curl --silent --show-error http://127.0.0.1:${PGTM_SINGLE_API_PORT}/ha/state
curl --silent --show-error http://127.0.0.1:${PGTM_SINGLE_API_PORT}/debug/verbose
```

What this proves:

- the image can build from current source
- etcd becomes healthy enough for the node dependency
- `node-a` binds the API listener and PostgreSQL listener described by the config
- debug routes are available on the same listener in the lab posture

Teardown:

```console
make docker-down
```

Remember that `make docker-down` removes named volumes. Use raw `docker compose stop/start` if you intentionally want to preserve data between short lab runs.

## Bring up the three-node cluster

```console
make docker-up-cluster
```

Validation:

```console
make docker-smoke-cluster
```

Teardown:

```console
make docker-down-cluster
```

This cluster path is where you start proving replication reachability, multi-node API exposure, and coordinated behavior across members instead of only one node's startup story.

## Security posture of the starter stacks

The checked-in lab stacks prioritize inspectability over hardening:

- API TLS is disabled
- API token auth is disabled
- debug routes are enabled
- PostgreSQL host and replication access inside the Compose bridge network use trust-based `pg_hba` rules

That choice is deliberate. The runtime schema currently carries role tokens as plain strings, so the safe first-run story is "keep the lab obviously local" rather than pretending committed configs are already a secret-management system.

The password files are still mounted and validated because the runtime contract is file-backed. Keeping that contract visible in the starter stack avoids a misleading split between "lab config" and "real config" semantics.

## Makefile and smoke helpers

The repo-owned commands are:

- `make docker-compose-config`
- `make docker-up`
- `make docker-down`
- `make docker-up-cluster`
- `make docker-down-cluster`
- `make docker-smoke-single`
- `make docker-smoke-cluster`

`make test-long` runs the Compose config validation and both smoke flows after the ultra-long HA profile. That means container deployment drift is part of the required verification story, not a manual afterthought.
