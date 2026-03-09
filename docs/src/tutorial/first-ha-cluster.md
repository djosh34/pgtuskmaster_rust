# First HA Cluster

## What you will accomplish

You will start a three-node PostgreSQL HA cluster on your local machine and inspect it through `pgtm`.

## Prerequisites

- Docker and Docker Compose installed
- `git clone` of the repository completed
- a local shell in the repository root directory
- the `pgtm` binary available in your shell

The local docker tutorials use these docs-owned operator configs:

- [`docs/examples/docker-cluster-node-a.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-a.toml)
- [`docs/examples/docker-cluster-node-b.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-b.toml)
- [`docs/examples/docker-cluster-node-c.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-c.toml)

Each file mirrors the shipped runtime config and adds `[pgtm].api_url` for the corresponding host-mapped API port.

## Steps

1. **Start the cluster**

   ```bash
   tools/docker/cluster.sh up --env-file .env.docker.example
   ```

   This command:

   - reads environment variables from `.env.docker.example`
   - builds the `pgtuskmaster:local` image when needed
   - starts `etcd`, `node-a`, `node-b`, and `node-c` as a persistent local stack
   - waits until the HA API, debug API, PostgreSQL ports, SQL readiness, and `1 primary + 2 replicas` topology are all healthy
   - prints the API URL, debug URL, PostgreSQL endpoint, leader, and current role for each node

2. **Inspect the running stack later without rebuilding it**

   ```bash
   tools/docker/cluster.sh status --env-file .env.docker.example
   ```

   If your local `.env.docker` matches the same ports, you can also use:

   ```bash
   make docker-status-cluster
   ```

3. **Check the current leader through node-a**

   ```bash
   pgtm -c docs/examples/docker-cluster-node-a.toml status
   ```

   The local docker runtime config disables API auth, so this read command does not need token flags. The table is cluster-oriented already, so you can inspect the current topology from one seed node.

4. **Verify that the cluster reports a consistent view**

   Run the same command from the other seed configs:

   ```bash
   pgtm -c docs/examples/docker-cluster-node-b.toml status
   pgtm -c docs/examples/docker-cluster-node-c.toml status
   ```

   In a healthy three-node cluster:

   - each command shows one primary and two replicas
   - the cluster stays `healthy`
   - non-leader nodes report `PHASE=replica`

## What you have now

- a running three-node PostgreSQL HA cluster
- one leader node serving as the primary
- two follower nodes reporting `replica`
- reachable HA APIs on ports `18081`, `18082`, and `18083`
- reachable PostgreSQL ports on `15433`, `15434`, and `15435`
- truthful `pgtm -c ... status` examples for the local mapped API ports
