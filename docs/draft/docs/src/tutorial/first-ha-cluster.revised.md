**What you will accomplish**

You will start a three-node PostgreSQL HA cluster on your local machine and identify the current leader.

**Prerequisites**

- Docker and Docker Compose installed
- `git clone` of the repository completed
- Local shell in repository root directory

**Steps**

 1. **Start the cluster**

    ```bash
    docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d --build
    ```

    This command:
    - Reads environment variables from `.env.docker.example`
    - Builds the `pgtuskmaster:local` image (if not present)
    - Starts etcd and three PostgreSQL nodes (node-a, node-b, node-c)
    - Publishes ports `18081`, `18082`, `18083` for API access
    - Creates a bridge network named `pgtm-internal`
    // todo: avoid promising the literal runtime Docker network name is exactly `pgtm-internal`; source only guarantees the compose network key and driver, while Compose may project-prefix runtime resource names.

 2. **Wait for cluster readiness (approximately 30 seconds)**

    ```bash
    sleep 30
    ```
    // todo: `sleep 30` is not source-backed. Replace this with an actual readiness check loop or a direct verification step without inventing a fixed wait duration.

    Then verify services are running:

    ```bash
    docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml ps
    ```

    **Expected output:**
    ```
    NAME                     COMMAND                  SERVICE   STATUS          PORTS
    pgtuskmaster-etcd-1      "etcd --name=etcd ..."   etcd      running         ...
    pgtuskmaster-node-a-1    "/usr/bin/entrypoint"    node-a    running         0.0.0.0:15433->5432/tcp, 0.0.0.0:18081->8080/tcp
    pgtuskmaster-node-b-1    "/usr/bin/entrypoint"    node-b    running         0.0.0.0:15434->5432/tcp, 0.0.0.0:18082->8080/tcp
    pgtuskmaster-node-c-1    "/usr/bin/entrypoint"    node-c    running         0.0.0.0:15435->5432/tcp, 0.0.0.0:18083->8080/tcp
    ```
    // todo: this `docker compose ps` sample output still uses unsupported concrete container names and exact formatting that depend on the Compose project name and local runtime output shape.

    Verify API reachability:

    ```bash
    curl -sf http://127.0.0.1:18081/ha/state || echo "Waiting for API..."
    ```

 3. **Check the current leader via node-a's API**

    ```bash
    pgtuskmasterctl --base-url http://127.0.0.1:18081 --output text ha state
    ```
    // todo: replace this with a source-backed host command such as `cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 --output text ha state`, or add an explicit build/install prerequisite for the binary.

    **Example output:**
    ```
    self_member_id=node-a
    leader=<current-leader>
    member_count=3
    dcs_trust=full_quorum
    ha_phase=<phase-for-node-a>
    ha_tick=42
    ha_decision=no_change
    snapshot_sequence=5
    ```
    // todo: this example still implies a specific steady-state payload for node-a. Replace it with a smaller, source-backed explanation that tells readers to inspect the `leader=` field, without inventing placeholder values that the CLI will not actually print.

    The `leader=` line shows the current leader. Inspect that line to identify which node is the primary.

 4. **Verify replication topology**

    Repeat the command targeting different nodes to confirm consistent view:

    ```bash
    pgtuskmasterctl --base-url http://127.0.0.1:18082 --output text ha state
    pgtuskmasterctl --base-url http://127.0.0.1:18083 --output text ha state
    ```

    **Expected result for replicas:**
    - `ha_phase` shows `replica` on non-leader nodes
    - `leader` field shows the same leader member ID across all nodes

**What you have now**

- A running 3-node PostgreSQL HA cluster
- One primary node accepting writes (the leader)
- Two replica nodes replicating from the primary
- Etcd as the distributed consensus store
- Exposed APIs at `127.0.0.1:18081`, `18082`, `18083`
- Direct PostgreSQL access at `127.0.0.1:15433`, `15434`, `15435`
