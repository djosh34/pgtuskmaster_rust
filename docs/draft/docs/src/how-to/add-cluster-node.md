# How to add a node to an existing cluster

This guide shows you how to add a new PostgreSQL node to an existing pgtuskmaster cluster and verify that it joins safely as a replica. The procedure assumes you have a running cluster with at least one healthy primary or replica.

## Prerequisites

- A running pgtuskmaster cluster with a known `cluster.name` and `dcs.scope`
- Network reachability between the new node and existing cluster members on PostgreSQL and DCS ports
- PostgreSQL 16 binaries installed on the new node at known paths
- File system paths prepared: data directory, socket directory, log directory

## Steps

### 1. Prepare a runtime configuration file for the new node

Create a `runtime.toml` file on the new node that matches the existing cluster configuration but with a unique `cluster.member_id` and node-specific listen addresses.

Example template (adapt `node-c` to your node identity and network addresses):

```toml

[cluster]
name = "docker-cluster"
member_id = "node-c"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "node-c"
listen_port = 5432
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
tls = { mode = "disabled" }
pg_hba = { source = { path = "/etc/pgtuskmaster/pg_hba.conf" } }
pg_ident = { source = { path = "/etc/pgtuskmaster/pg_ident.conf" } }

[postgres.roles.superuser]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/postgres-superuser-password" } }

[postgres.roles.replicator]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/replicator-password" } }

[postgres.roles.rewinder]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/rewinder-password" } }

[dcs]
endpoints = ["http://etcd:2379"]
scope = "docker-cluster"

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
```

Key fields you **must** adjust:

- `cluster.member_id`: Unique per node (e.g., `node-c`)
- `postgres.listen_host`: Resolvable hostname or IP for PostgreSQL client connections from other nodes
- `postgres.listen_port`: PostgreSQL port; must be reachable from existing nodes
- `dcs.endpoints`: Must point to the same etcd cluster as existing members

### 2. Verify network connectivity before starting the node

From the new node, confirm you can reach the DCS and the PostgreSQL ports of existing cluster members.

Test DCS connectivity:

```bash
curl http://etcd:2379/v3/cluster/member/list
```

Test PostgreSQL connectivity to each existing member:

```bash
psql -h <existing-node-host> -p 5432 -U postgres -c "SELECT 1"
```

The node will not join correctly if these connections fail.

### 3. Start pgtuskmaster with the prepared configuration

Run the binary with the configuration file path. Use whichever method you normally use to start pgtuskmaster services (systemd, docker, or direct execution).

Example direct execution:

```bash
pgtuskmaster --config /etc/pgtuskmaster/runtime.toml
```

The process will start, initialize its internal state, and attempt to contact the DCS and PostgreSQL.

### 4. Observe the new node progression through HA phases

Poll the node's `/ha/state` HTTP endpoint to watch its progression:

```bash
curl http://<new-node-host>:8080/ha/state
```

The HA state response includes:

- `self_member_id`: Should match your configured `cluster.member_id`
- `ha_phase`: The current high-availability phase
- `leader`: The current cluster leader member ID (if known)
// todo: unsupported field name. `HaStateResponse` does not expose `postgres_sql_status`.
- `postgres_sql_status`: SQL connectivity health

Typical progression for a new replica:

```
Init -> WaitingPostgresReachable -> WaitingDcsTrusted -> Replica
```

If the node instead enters `CandidateLeader` or `Primary`, review the configuration; a new node should normally settle as a `Replica` when an existing primary is healthy.

### 5. Verify the new node appears in cluster membership

// todo: unsupported claim. `GET /ha/state` does not expose a `members` field or per-member records.
Query the DCS directly or poll any existing node's `/ha/state` endpoint. The `members` field should include the new `member_id` along with its `postgres_host` and `postgres_port`.

Example using an existing node's API:

```bash
// todo: unsupported response field. `GET /ha/state` does not expose `.members`.
curl http://<existing-node>:8080/ha/state | jq '.members'
```

// todo: unsupported placement. These member-record fields are real in `src/dcs/state.rs`, but they are not exposed by `/ha/state` in the loaded API sources.
Each member record contains:

- `member_id`: The configured node identifier
- `postgres_host`: Advertised PostgreSQL host
- `postgres_port`: Advertised PostgreSQL port
- `role`: Observed PostgreSQL role (`Primary`, `Replica`, or `Unknown`)
- `sql`: PostgreSQL SQL health (`Healthy`, `Unreachable`)
- `readiness`: Node readiness (`Ready`, `NotReady`)
- `updated_at`: Last refresh timestamp

// todo: unsupported verification path through `/ha/state`. Use a source-backed verification method instead.
Confirm the new node's record shows `role: Replica` and `sql: Healthy` within a few lease intervals.

### 6. Confirm replication is functioning

Connect to the new node's PostgreSQL and verify it is replicating from the primary:

```bash
psql -h <new-node-host> -p 5432 -U postgres -c "SELECT pg_is_in_recovery()"
```

A result of `t` indicates the node is in recovery (replica) mode. If the result is `f`, the node incorrectly promoted itself to primary; investigate configuration and logs immediately.

### 7. Validate no dual-primary condition

Poll cluster state for a short observation window (e.g., 30 seconds) and ensure no two nodes simultaneously report `role: Primary`.

Example script:

```bash
for i in $(seq 1 10); do
  for host in node-a node-b node-c; do
// todo: unsupported response field. `HaStateResponse` does not expose `.role`.
    curl -s http://${host}:8080/ha/state | jq -r '"\(.self_member_id) \(.role)"'
  done
  sleep 3
done | sort | uniq -c
```

You should see only one primary throughout the window. If you observe multiple primaries, stop the new node and review logs.

## Expected steady state

When complete, the new node:

- Runs as a replica following the existing primary
- Publishes a fresh member record to the DCS every lease interval
- Accepts read-only queries if `hot_standby` is enabled in PostgreSQL
- Does not attempt leader election unless the primary fails

## Troubleshooting common issues

**Node stuck in `WaitingPostgresReachable`**  
- Check PostgreSQL logs in `/var/log/pgtuskmaster/postgres.log`
- Verify `postgres.listen_host` and `postgres.listen_port` are not blocked by firewall
- Confirm `process.binaries.postgres` path is correct

**Node enters `FailSafe` immediately**  
- DCS may be unreachable or the node cannot publish its member record
- Verify `dcs.endpoints` and network paths to etcd
- Check etcd cluster health: `etcdctl endpoint health`

**Node becomes `Primary` unexpectedly**  
- Existing primary may have been unreachable during join
- Review logs on all nodes to confirm the previous primary is healthy
- If this was accidental, stop the new node, rewind it using `pg_rewind`, and restart

**Node never appears in `/ha/state` on existing members**  
- Member ID may conflict with an existing node
- Ensure `cluster.member_id` is unique within the `dcs.scope`
- Check the node's runtime logs for DCS write errors

## Diagram: network connectivity required for a new node

// todo: remove placeholder prose and replace it with working mermaid only.
[diagram about network connectivity for new node joining cluster, showing three layers:
- DCS layer (etcd) with arrows from new node to each etcd endpoint
- PostgreSQL replication layer with arrows from new node to existing primary's postgres port and from primary to new node's postgres port
- API observation layer with arrows from operator host to all nodes' API ports (8080)
**more details on diagram**: DCS connectivity is mandatory for membership; PostgreSQL port reachability enables replication; API ports are for observation and control]

## Diagram: HA phase progression for a new replica node

// todo: remove placeholder prose and replace it with working mermaid only.
[diagram about HA phase transition flow from Init to Replica, showing decision points:
- Init -> WaitingPostgresReachable (wait for local PostgreSQL health)
- WaitingPostgresReachable -> WaitingDcsTrusted (wait for DCS trust)
- WaitingDcsTrusted -> Replica (when a primary is visible and trust is established)
- Alternative path to CandidateLeader if no primary detected
**more details on diagram**: Transitions depend on DCS trust evaluation, PostgreSQL reachability, and observed leader state. A healthy existing primary keeps the new node on the replica path.]

## Missing source support

- Command-line flags or environment variable overrides for the join process are not detailed in available sources; use the configuration file approach.
- TLS bootstrapping for node-to-node communication is supported but not explicitly validated in the join flow; refer to [Configure TLS](configure-tls.md) for mode-specific steps.
- Specific `pg_hba.conf` or `pg_ident.conf` rules required for inter-node authentication are not enumerated; ensure existing primary permits replication connections from the new node's PostgreSQL address.
