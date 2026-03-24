# First HA Cluster

## What you will accomplish

You will start a three-node PostgreSQL HA cluster on your local machine and inspect it through `pgtm`.

## Prerequisites

- Docker and Docker Compose installed
- `git clone` of the repository completed
- a local shell in the repository root directory
- the `pgtm` binary installed from the repo root with `make install-pgtm`

The local docker walkthrough uses the shipped host-side operator config:

- [`docker/pgtm.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docker/pgtm.toml)

Alternate seed configs are still available when you want to prove that another node API reports the same cluster view:

- [`docs/examples/docker-cluster-node-b.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-b.toml)
- [`docs/examples/docker-cluster-node-c.toml`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/examples/docker-cluster-node-c.toml)

## Steps

1. **Start the cluster**

   ```bash
   docker compose up -d --build
   ```

   This command:

   - builds the `pgtuskmaster-local:compose` image when needed
   - starts `etcd`, `node-a`, `node-b`, and `node-c`
   - publishes the HTTPS APIs on ports `18081`, `18082`, and `18083`
   - publishes PostgreSQL on ports `15001`, `15002`, and `15003`

   Wait until the operator view becomes reachable:

   ```bash
   until pgtm -c docker/pgtm.toml status >/dev/null 2>&1; do
     sleep 1
   done
   ```

2. **Inspect the running stack**

   ```bash
   docker compose ps
   ```

3. **Check the current leader through the canonical host config**

   ```bash
   pgtm -c docker/pgtm.toml status
   ```

   `docker/pgtm.toml` points `pgtm` at node-a's host-mapped HTTPS API and carries the shared TLS and token material for the shipped Docker stack. The table is cluster-oriented already, so you can inspect the current topology from one seed node.

4. **Verify that the cluster reports a consistent view**

   Run the same command from the alternate seed configs:

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
- reachable PostgreSQL ports on `15001`, `15002`, and `15003`
- truthful `pgtm -c ... status`, `primary`, and `replicas` examples for the local mapped API and PostgreSQL ports
