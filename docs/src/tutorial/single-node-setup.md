# Single-Node Setup

This tutorial walks through the smallest shipped deployment in the repository: one `pgtuskmaster` node and one `etcd` service. The runtime still uses the normal DCS and HA machinery, but the topology is easier to inspect than the three-node cluster.

## Prerequisites

- Docker and Docker Compose installed
- a checkout of this repository
- a shell in the repository root

## Step 1: Start the single-node stack

Run the shipped compose file with the example environment:

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.single.yml up -d --build
```

This stack starts:

- one `etcd` service
- one `node-a` service built from the repository image

With `.env.docker.example`, the compose file publishes:

- the HTTP API on `127.0.0.1:18080`
- PostgreSQL on `127.0.0.1:15432`

Internally, the runtime config keeps:

- `cluster.name = "docker-single"`
- `cluster.member_id = "node-a"`
- `dcs.scope = "docker-single"`
- `dcs.endpoints = ["http://etcd:2379"]`
- `api.listen_addr = "0.0.0.0:8080"`
- `debug.enabled = true`

## Step 2: Wait for the API to become reachable

Poll the HA state until the node responds:

```bash
until curl -sf http://127.0.0.1:18080/ha/state >/dev/null; do
  sleep 1
done
```

You can also confirm that both services are running:

```bash
docker compose --env-file .env.docker.example -f docker/compose/docker-compose.single.yml ps
```

## Step 3: Read the single-node HA state

Query the HA API directly:

```bash
curl --fail --silent http://127.0.0.1:18080/ha/state | jq .
```

Or use the CLI:

```bash
cargo run --bin pgtm -- -c ./config.toml
```

Fields worth checking:

- `self_member_id`
- `leader`
- `member_count`
- `dcs_trust`
- `ha_phase`
- `ha_decision`

In this topology, the node can still reach trusted state because trust evaluation accepts a one-member cluster when:

- the DCS store is healthy
- the local member record is present
- the local member record is fresh

## Step 4: Inspect the debug snapshot through `pgtm`

The single-node sample enables the debug surface, so you can inspect it immediately with the CLI:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18080 -v status
cargo run --bin pgtm -- --base-url http://127.0.0.1:18080 debug verbose
```

Focus on:

- `meta.sequence`
- `dcs.trust`
- `dcs.member_count`
- `dcs.leader`
- `ha.phase`
- `ha.decision`

If you want the raw stable payload, switch to JSON:

```bash
cargo run --bin pgtm -- --base-url http://127.0.0.1:18080 --json debug verbose | jq '{meta: .meta, dcs: .dcs, ha: .ha}'
```

Because the debug worker retains recent history, you can also poll incrementally:

```bash
seq=$(cargo run --bin pgtm -- --base-url http://127.0.0.1:18080 --json debug verbose | jq '.meta.sequence')
cargo run --bin pgtm -- --base-url http://127.0.0.1:18080 --json debug verbose --since "${seq}" | jq '{changes: .changes, timeline: .timeline}'
```

If nothing changed between the two requests, the `changes` and `timeline` arrays will be empty.

## Step 5: Compare the single-node sample to the full cluster tutorial

The single-node sample is simpler to run, but it does not disable the normal runtime model. The shipped runtime config still includes:

- `[dcs]`
- `[ha]`
- PostgreSQL role credentials for `superuser`, `replicator`, and `rewinder`
- PostgreSQL 16 binary paths under `/usr/lib/postgresql/16/bin/...`

That means this tutorial is best read as "the same system in a smaller topology," not as a separate standalone mode.

## What you learned

You now have:

- the repository's smallest shipped deployment running locally
- a reachable HA API on `127.0.0.1:18080`
- a reachable debug API on the same listener
- a concrete example of how a one-member cluster still uses DCS-backed trust and HA state
