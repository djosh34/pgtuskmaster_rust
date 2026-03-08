# First HA Cluster

**What you will accomplish**

You will start a three-node PostgreSQL HA cluster on your local machine and identify the current leader.

**Prerequisites**

- Docker and Docker Compose installed
- `git clone` of the repository completed
- Local shell in the repository root directory

**Steps**

1. **Start the cluster**

   ```bash
   docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d --build
   ```

   This command:

   - Reads environment variables from `.env.docker.example`
   - Builds the `pgtuskmaster:local` image when needed
   - Starts `etcd`, `node-a`, `node-b`, and `node-c`
   - Publishes the API on `127.0.0.1:18081`, `127.0.0.1:18082`, and `127.0.0.1:18083`
   - Publishes PostgreSQL on `127.0.0.1:15433`, `127.0.0.1:15434`, and `127.0.0.1:15435`
   - Uses the compose-defined bridge network keyed as `pgtm-internal` for internal traffic

2. **Wait for the node-a API to become reachable**

   ```bash
   until curl -sf http://127.0.0.1:18081/ha/state >/dev/null; do
     sleep 1
   done
   ```

   Then list the services:

   ```bash
   docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml ps
   ```

   At that point you should see the four compose services from this stack: `etcd`, `node-a`, `node-b`, and `node-c`.

3. **Check the current leader through node-a**

   ```bash
   cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 --output text ha state
   ```

   The cluster runtime configs in `docker/configs/cluster/` disable API auth, so this read command does not require `--read-token` or `--admin-token` in the local docker setup.

   The command renders the `/ha/state` response as text lines. Inspect the `leader=` line to see which member currently owns leadership.

4. **Verify that the cluster reports a consistent view**

   Run the same command against node-b and node-c:

   ```bash
   cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18082 --output text ha state
   cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18083 --output text ha state
   ```

   In a healthy three-node cluster:

   - All three commands report the same `leader` member ID
   - The cluster reports `member_count=3`
   - Non-leader nodes report `ha_phase=replica`

**What you have now**

- A running three-node PostgreSQL HA cluster
- One leader node serving as the primary
- Two follower nodes reporting `replica` through the HA API
- Etcd backing the cluster's distributed state
- Reachable HA APIs on ports `18081`, `18082`, and `18083`
- Reachable PostgreSQL ports on `15433`, `15434`, and `15435`
