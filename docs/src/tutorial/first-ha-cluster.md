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
   tools/docker/cluster.sh up --env-file .env.docker.example
   ```

   This command:

   - Reads environment variables from `.env.docker.example`
   - Builds the `pgtuskmaster:local` image when needed; the first run can take a while because Docker has to build the image
   - Starts `etcd`, `node-a`, `node-b`, and `node-c` as a persistent local stack
   - Waits until the HA API, debug API, PostgreSQL ports, SQL readiness, and `1 primary + 2 replicas` topology are all healthy
   - Prints the API URL, debug URL, PostgreSQL endpoint, leader, and current role for each node
   - Uses the compose-defined bridge network keyed as `pgtm-internal` for internal traffic

2. **Inspect the running stack later without rebuilding it**

   ```bash
   tools/docker/cluster.sh status --env-file .env.docker.example
   ```

   This prints the same endpoint and topology summary for the already-running stack. If you prefer the Makefile wrapper and your `.env.docker` matches the same ports, you can also use:

   ```bash
   make docker-status-cluster
   ```

3. **Check the current leader through node-a**

   ```bash
   cargo run --bin pgtm -- --base-url http://127.0.0.1:18081 --output text status
   ```

   The cluster runtime configs in `docker/configs/cluster/` disable API auth, so this read command does not require `--read-token` or `--admin-token` in the local docker setup.

   The command renders the `/ha/state` response as text lines. Inspect the `leader=` line to see which member currently owns leadership.

4. **Verify that the cluster reports a consistent view**

   Run the same command against node-b and node-c:

   ```bash
   cargo run --bin pgtm -- --base-url http://127.0.0.1:18082 --output text status
   cargo run --bin pgtm -- --base-url http://127.0.0.1:18083 --output text status
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
- A repeatable status command: `tools/docker/cluster.sh status --env-file .env.docker.example`
- A matching teardown command: `tools/docker/cluster.sh down --env-file .env.docker.example`
