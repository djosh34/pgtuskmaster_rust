# Extra Context: Minimum Local Command To Start The Three-Node HA Cluster

K2 asked for the minimum docker compose command to start the three-node cluster on a local machine without custom networking. This note answers that question using only source-backed details from this repository.

## What the compose file itself requires

The cluster compose file at `docker/compose/docker-compose.cluster.yml` defines four services:

- `etcd`
- `node-a`
- `node-b`
- `node-c`

It also defines a normal bridge network named `pgtm-internal`, so the cluster does not require the operator to create a custom Docker network up front. The service-to-service hostnames used in the runtime configs are `etcd`, `node-a`, `node-b`, and `node-c`, and those names come from the compose service names.

The compose file depends on environment-variable substitution for:

- `PGTUSKMASTER_IMAGE`
- `ETCD_IMAGE`
- `PGTM_SECRET_SUPERUSER_FILE`
- `PGTM_SECRET_REPLICATOR_FILE`
- `PGTM_SECRET_REWINDER_FILE`
- `PGTM_CLUSTER_NODE_A_API_PORT`
- `PGTM_CLUSTER_NODE_A_PG_PORT`
- `PGTM_CLUSTER_NODE_B_API_PORT`
- `PGTM_CLUSTER_NODE_B_PG_PORT`
- `PGTM_CLUSTER_NODE_C_API_PORT`
- `PGTM_CLUSTER_NODE_C_PG_PORT`

That means a plain `docker compose -f docker/compose/docker-compose.cluster.yml up -d` is not enough on its own unless the caller has already exported all required variables in the shell.

## The repository-provided env file

The repository ships `.env.docker.example`, which defines all of the variables the cluster compose file needs for a local example run.

Important details from `.env.docker.example`:

- `PGTUSKMASTER_IMAGE=pgtuskmaster:local`
- `ETCD_IMAGE=quay.io/coreos/etcd:v3.5.21`
- `PGTM_SECRET_SUPERUSER_FILE=../secrets/postgres-superuser.password.example`
- `PGTM_SECRET_REPLICATOR_FILE=../secrets/replicator.password.example`
- `PGTM_SECRET_REWINDER_FILE=../secrets/rewinder.password.example`
- `PGTM_CLUSTER_NODE_A_API_PORT=18081`
- `PGTM_CLUSTER_NODE_A_PG_PORT=15433`
- `PGTM_CLUSTER_NODE_B_API_PORT=18082`
- `PGTM_CLUSTER_NODE_B_PG_PORT=15434`
- `PGTM_CLUSTER_NODE_C_API_PORT=18083`
- `PGTM_CLUSTER_NODE_C_PG_PORT=15435`

Those secret paths are written relative to the compose file directory, so they resolve to the example password files under `docker/secrets/`.

## The safest documented command to recommend

The most source-backed command to recommend in a tutorial is:

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d --build
```

Reasons this is the safest recommendation:

- It supplies all required variable substitutions through a checked-in env file.
- It uses the checked-in compose file directly.
- It does not require any custom Docker network preparation because the compose file already defines `pgtm-internal`.
- It matches the repository's own smoke flow shape, which starts the cluster with `docker compose ... up -d --build` after generating a compatible env file.

## Strict minimum versus operationally safe minimum

There are two slightly different ways to phrase "minimum":

1. Strict compose invocation minimum when the image is already available locally:

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d
```

2. Safe tutorial command that also covers the common "I have not built the image yet" case:

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d --build
```

For a first-run tutorial, the second command is more defensible because the compose file points each node service at the repo root build context and `docker/Dockerfile.prod`.

## What that command publishes on the host

With `.env.docker.example`, the cluster exposes:

- `node-a` API on `127.0.0.1:18081`
- `node-a` PostgreSQL on `127.0.0.1:15433`
- `node-b` API on `127.0.0.1:18082`
- `node-b` PostgreSQL on `127.0.0.1:15434`
- `node-c` API on `127.0.0.1:18083`
- `node-c` PostgreSQL on `127.0.0.1:15435`

That makes the tutorial concrete because the operator can immediately target node-a's API with `http://127.0.0.1:18081`.

## Evidence sources behind this note

- `docker/compose/docker-compose.cluster.yml`
- `.env.docker.example`
- `tools/docker/smoke-cluster.sh`
- `tools/docker/common.sh`
