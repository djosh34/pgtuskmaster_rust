# Single Node Tutorial

In this tutorial we will start a single PostgreSQL node managed by pgtuskmaster. You will learn how to run the system locally with Docker, verify that Postgres is healthy, and make your first API call to check cluster state.

By the end you will have a running PostgreSQL instance that is observable through both SQL and a REST API, ready for later expansion into a multi-node cluster.

## Prerequisites

- Docker and Docker Compose installed
- `curl` command available
- A terminal with environment variable support

## Step 1: Prepare environment and secrets

// todo: The raw GitHub `curl` secret-download workflow and repository URL in this section were not established by the requested source files. Replace with source-backed instructions based on local repo assets or remove.

```bash
# Create a working directory
mkdir -p pgtuskmaster-tutorial && cd pgtuskmaster-tutorial

# Copy the example secrets from the repository
curl -s https://raw.githubusercontent.com/anomalyco/pgtuskmaster_rust/main/docker/secrets/postgres-superuser.password.example -o postgres-superuser.password
curl -s https://raw.githubusercontent.com/anomalyco/pgtuskmaster_rust/main/docker/secrets/replicator.password.example -o replicator.password
curl -s https://raw.githubusercontent.com/anomalyco/pgtuskmaster_rust/main/docker/secrets/rewinder.password.example -o rewinder.password
```

## Step 2: Create Docker Compose file

// todo: This section inlines a rewritten compose file with hardcoded image names, hardcoded published ports, and inline config content. The requested sources only establish the shipped sample files and env-var based port mapping, not this exact rewritten manifest. Re-anchor this section to the repository sample files.

Create `docker-compose.yml` with the exact content from the single-node example:

```yaml
services:
  etcd:
    image: gcr.io/etcd-development/etcd:v3.5.18
    command:
      - etcd
      - --name=etcd
      - --data-dir=/etcd-data
      - --listen-client-urls=http://0.0.0.0:2379
      - --advertise-client-urls=http://etcd:2379
      - --listen-peer-urls=http://0.0.0.0:2380
      - --initial-advertise-peer-urls=http://etcd:2380
      - --initial-cluster=etcd=http://etcd:2380
      - --initial-cluster-state=new
    healthcheck:
      test: ["CMD", "etcdctl", "--endpoints=http://127.0.0.1:2379", "endpoint", "health"]
      interval: 5s
      timeout: 5s
      retries: 20
    networks:
      - pgtm-internal
    volumes:
      - etcd-single-data:/etcd-data

  node-a:
    image: ghcr.io/anomalyco/pgtuskmaster:latest
    depends_on:
      etcd:
        condition: service_healthy
    restart: unless-stopped
    configs:
      - source: runtime-node-a
        target: /etc/pgtuskmaster/runtime.toml
      - source: common-pg-hba
        target: /etc/pgtuskmaster/pg_hba.conf
      - source: common-pg-ident
        target: /etc/pgtuskmaster/pg_ident.conf
    secrets:
      - source: superuser-password
        target: postgres-superuser-password
      - source: replicator-password
        target: replicator-password
      - source: rewinder-password
        target: rewinder-password
    networks:
      - pgtm-internal
    ports:
      - "8080:8080"
      - "5432:5432"
    volumes:
      - node-a-single-data:/var/lib/postgresql
      - node-a-single-logs:/var/log/pgtuskmaster

configs:
  runtime-node-a:
    content: |
      
      [cluster]
      name = "docker-single"
      member_id = "node-a"
      
      [postgres]
      data_dir = "/var/lib/postgresql/data"
      listen_host = "node-a"
      listen_port = 5432
      socket_dir = "/var/lib/pgtuskmaster/socket"
      log_file = "/var/log/pgtuskmaster/postgres.log"
      local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
      rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
      tls = { mode = "disabled" }
      
      [postgres.roles.superuser]
      username = "postgres"
      auth = { type = "password", password = { path = "/run/secrets/postgres-superuser-password" } }
      
      [postgres.roles.replicator]
      username = "postgres"
      auth = { type = "password", password = { path = "/run/secrets/replicator-password" } }
      
      [postgres.roles.rewinder]
      username = "postgres"
      auth = { type = "password", password = { path = "/run/secrets/rewinder-password" } }
      
      [postgres.pg_hba]
      source = { path = "/etc/pgtuskmaster/pg_hba.conf" }
      
      [postgres.pg_ident]
      source = { path = "/etc/pgtuskmaster/pg_ident.conf" }
      
      [dcs]
      endpoints = ["http://etcd:2379"]
      scope = "docker-single"
      
      [ha]
      loop_interval_ms = 1000
      lease_ttl_ms = 10000
      
      [process]
      pg_rewind_timeout_ms = 120000
      bootstrap_timeout_ms = 300000
      fencing_timeout_ms = 30000
      
      [process.binaries]
      postgres = "/usr/lib/postgresql/16/bin/postgres"
      pg_ctl = "/usr/lib/postgresql/16/bin/pg_ctl"
      pg_rewind = "/usr/lib/postgresql/16/bin/pg_rewind"
      initdb = "/usr/lib/postgresql/16/bin/initdb"
      pg_basebackup = "/usr/lib/postgresql/16/bin/pg_basebackup"
      psql = "/usr/lib/postgresql/16/bin/psql"
      
      [logging]
      level = "info"
      capture_subprocess_output = true
      
      [logging.postgres]
      enabled = true
      poll_interval_ms = 200
      cleanup = { enabled = true, max_files = 20, max_age_seconds = 86400, protect_recent_seconds = 300 }
      
      [logging.sinks.stderr]
      enabled = true
      
      [logging.sinks.file]
      enabled = true
      path = "/var/log/pgtuskmaster/runtime.jsonl"
      mode = "append"
      
      [api]
      listen_addr = "0.0.0.0:8080"
      security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
      
      [debug]
      enabled = true

  common-pg-hba:
    content: |
      # TYPE  DATABASE        USER            ADDRESS                 METHOD
      local   all             postgres                                peer
      host    all             all             0.0.0.0/0               md5
      host    replication     replicator      0.0.0.0/0               md5

  common-pg-ident:
    content: |+
      # Empty ident map

secrets:
  superuser-password:
    file: ./postgres-superuser.password
  replicator-password:
    file: ./replicator.password
  rewinder-password:
    file: ./rewinder.password

networks:
  pgtm-internal:
    driver: bridge

volumes:
  etcd-single-data:
  node-a-single-data:
  node-a-single-logs:
```

## Step 3: Start the stack

```bash
docker compose up -d
```

Wait for the healthcheck to pass. Check logs:

```bash
docker compose logs -f node-a
```

// todo: The exact startup log lines below were not verified in the requested files.

You should see lines like:

```
INFO starting postgres process
INFO postgres process started
INFO successfully acquired leader lease
```

Press Ctrl+C to exit the log tail.

## Step 4: Verify PostgreSQL is running

Connect with psql:

// todo: The requested sources do not verify that `psql` is available inside the running container in this exact way, nor the exact prompt/version output shown below.

```bash
docker compose exec node-a psql -U postgres -h localhost -p 5432 postgres
```

You will see the psql prompt:

```
psql (16.0)
Type "help" for help.

postgres=#
```

Run a simple query:

```sql
SELECT version();
```

Expected output:

```
                                               version                                               
-----------------------------------------------------------------------------------------------------
 PostgreSQL 16.0 on x86_64-pc-linux-gnu, compiled by gcc (GCC) 12.2.0, 64-bit
(1 row)
```

Exit psql:

```bash
\q
```

## Step 5: Check cluster state via API

// todo: The exact `/ha/state` response example needs source-backed verification before publication.

```bash
curl -s http://localhost:8080/ha/state | jq
```

Expected output:

```json
{
  "leader_lease": {
    "holder_id": "node-a",
    "expires_at": "2026-03-08T10:23:45Z"
  },
  "member_state": {
    "id": "node-a",
    "reachable": true,
    "pg_is_in_recovery": false,
    "pg_timeline": 1,
    "pg_xlogpos": "0/5000090"
  },
  "trust": "FullQuorum"
}
```

Notice that `pg_is_in_recovery` is false, meaning this node is the primary (writable) instance.

## Step 6: Inspect the debug snapshot

Because debug is enabled, you can view internal state:

```bash
curl -s http://localhost:8080/debug/snapshot | jq '.dcs_state.members | keys'
```

Expected output:

```json
[
  "node-a"
]
```

This confirms only one member exists in the cluster.

## What you have built

You now have:

- One etcd instance providing distributed consensus
- One pgtuskmaster node running PostgreSQL 16
- A primary Postgres instance listening on port 5432
- An HTTP API on port 8080 for observability and control
- Debug endpoints enabled for internal inspection

The system uses the same runtime daemon as multi-node clusters, so you can later add replicas without changing the runtime schema.

## Next steps

- Learn how to bootstrap a three-node cluster in [First HA Cluster](first-ha-cluster.md)
- Explore the HTTP API reference at [/reference/http-api.md](reference/http-api.md)
- Read about architecture decisions in [/explanation/architecture.md](explanation/architecture.md)
