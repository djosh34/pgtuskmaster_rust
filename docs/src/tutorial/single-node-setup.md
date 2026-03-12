# Single-Node Setup

This tutorial walks through the smallest shipped deployment in the repository: one `pgtuskmaster` node and one `etcd` service. You will use `pgtm` as the normal operator entry point throughout.

## Prerequisites

- Docker and Docker Compose installed
- a checkout of this repository
- a shell in the repository root
- the `pgtm` binary available in your shell

The tutorial uses [`docs/examples/docker-single-node-a.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-single-node-a.toml). That file mirrors the shipped single-node runtime config and adds `[pgtm].api_url = "http://127.0.0.1:18080"` so host-side operator commands stay truthful even though the daemon binds `0.0.0.0:8080` inside the container.

## Step 1: Start the single-node stack

Run the shipped compose file with the example environment:

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.single.yml up -d --build
```

With `.env.docker.example`, the compose file publishes:

- the HTTP API on `127.0.0.1:18080`
- PostgreSQL on `127.0.0.1:15432`

## Step 2: Wait for the operator view to become reachable

Poll `pgtm` until the single-node view is reachable:

```bash
until pgtm -c docs/examples/docker-single-node-a.toml --json status >/dev/null 2>&1; do
  sleep 1
done
```

You can also confirm that both services are running:

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.single.yml ps
```

## Step 3: Read the single-node HA state

Start with the cluster-oriented CLI view:

```bash
pgtm -c docs/examples/docker-single-node-a.toml status -v
```

Focus on:

- `ROLE`
- `TRUST`
- `PHASE`
- `DECISION`
- `DEBUG`

In this topology, the node can still reach trusted state because trust evaluation accepts a one-member cluster when:

- the DCS store is healthy
- the local member record is present
- the local member record is fresh

## Step 4: Inspect the retained debug state

The single-node sample enables the debug surface, so you can inspect it immediately through `pgtm`:

```bash
pgtm -c docs/examples/docker-single-node-a.toml debug verbose
```

That summary is usually enough to answer:

- which member currently leads
- whether DCS trust is healthy
- which HA phase and decision are active
- whether recent `changes` or `timeline` entries show churn

If you want the exact stable payload for automation or later review, save the JSON form:

```bash
pgtm -c docs/examples/docker-single-node-a.toml --json debug verbose > single-node-debug.json
```

## Step 5: Compare the sample to the full cluster tutorial

The single-node sample is simpler to run, but it still uses the normal runtime model:

- DCS-backed trust
- the same HA role/authority loop
- the same PostgreSQL role credentials
- the same daemon configuration shape as the multi-node examples

That makes this tutorial a smaller topology, not a separate product mode.

## What you learned

You now have:

- the repository's smallest shipped deployment running locally
- a truthful operator config for the local mapped API port
- a repeatable `pgtm status -v` path for HA state
- a repeatable `pgtm debug verbose` path for retained debug state
