# First HA Cluster

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

 2. **Wait for cluster readiness (approximately 30 seconds)**

    ```bash
    docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml ps
    ```

    // todo: `docker compose ps` also needs the env file for this compose file, because the compose file contains required variable substitutions.
    // todo: the node services do not define Docker healthchecks in `docker/compose/docker-compose.cluster.yml`, so the docs must not tell readers to wait for `healthy` on all four services.
    // todo: replace this readiness instruction with source-backed checks such as `/ha/state` reachability or the repo's smoke-script-style checks, instead of promising compose health for node-a/node-b/node-c.

    **Expected output:**
    ```
    NAME                     COMMAND                  SERVICE   STATUS              PORTS
    pgtuskmaster-etcd-1      "etcd --name=etcd ..."   etcd      running (healthy)   
    pgtuskmaster-node-a-1    "/usr/bin/entrypoint"    node-a    running (healthy)   0.0.0.0:15433->5432/tcp, 0.0.0.0:18081->8080/tcp
    pgtuskmaster-node-b-1    "/usr/bin/entrypoint"    node-b    running (healthy)   0.0.0.0:15434->5432/tcp, 0.0.0.0:18082->8080/tcp
    pgtuskmaster-node-c-1    "/usr/bin/entrypoint"    node-c    running (healthy)   0.0.0.0:15435->5432/tcp, 0.0.0.0:18083->8080/tcp
    ```
    // todo: this concrete `docker compose ps` output is not source-backed. The node services are not healthchecked, and Docker Compose project/container names are not guaranteed to be these exact values.

 3. **Check the current leader via node-a's API**

    ```bash
    pgtuskmasterctl --base-url http://127.0.0.1:18081 --output text ha state
    ```
    // todo: add a source-backed prerequisite for how the operator gets the `pgtuskmasterctl` binary on the host, because this tutorial currently assumes it is already installed or built.

    **Example output:**
    ```
    self_member_id=node-a
    leader=node-a
    member_count=3
    dcs_trust=full_quorum
    ha_phase=primary
    ha_tick=42
    ha_decision=no_change
    snapshot_sequence=5
    ```
    // todo: the text renderer prints lowercase snake_case values for `dcs_trust`, `ha_phase`, and `ha_decision`, so the example must use those exact forms.
    // todo: do not hard-code `leader=node-a` as the example result unless runtime evidence proves node-a wins leadership. The source only supports "inspect the `leader=` line for the current leader".

    The `leader=` line shows the current leader. In this example, `node-a` is the primary.
    // todo: this sentence also hard-codes node-a as leader without source-backed runtime evidence.

 4. **Verify replication topology**

    Repeat the command targeting different nodes to confirm consistent view:

    ```bash
    pgtuskmasterctl --base-url http://127.0.0.1:18082 --output text ha state
    pgtuskmasterctl --base-url http://127.0.0.1:18083 --output text ha state
    ```

    **Expected result for replicas:**
    - `ha_phase` shows `replica` on non-leader nodes
    - `leader` field shows the same leader member ID across all nodes
    // todo: use lowercase `replica` to match the actual CLI text output.
    // todo: do not claim the shared leader value is specifically `node-a` without runtime evidence.

// todo: replace this placeholder with actual markdown content or remove it. Also avoid claiming the literal Docker network name is always exactly `pgtm-internal`, because Compose commonly project-prefixes runtime network names unless `name:` is set.

**What you have now**

- A running 3-node PostgreSQL HA cluster
- One primary node accepting writes (the leader)
- Two replica nodes replicating from the primary
- Etcd as the distributed consensus store
- Exposed APIs at `127.0.0.1:18081`, `18082`, `18083`
- Direct PostgreSQL access at `127.0.0.1:15433`, `15434`, `15435`
