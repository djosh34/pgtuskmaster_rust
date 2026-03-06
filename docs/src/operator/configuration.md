# Configuration Guide

This guide describes the current `config_version = "v2"` runtime schema. The backup/pgBackRest configuration surface has been removed. Replica cloning is still supported through `pg_basebackup`.

## Baseline example

```toml
config_version = "v2"

[cluster]
name = "prod-cluster-a"
member_id = "node-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/var/run/pgtuskmaster/sock"
log_file = "/var/log/pgtuskmaster/postgres.log"
rewind_source_host = "10.0.0.10"
rewind_source_port = 5432
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = {
  superuser = { username = "postgres", auth = { type = "tls" } },
  replicator = { username = "replicator", auth = { type = "tls" } },
  rewinder = { username = "rewinder", auth = { type = "tls" } },
}
pg_hba = { source = { content = "local all all trust\nhost replication replicator 10.0.0.0/24 trust\n" } }
pg_ident = { source = { content = "# empty\n" } }

[dcs]
endpoints = ["http://10.0.0.21:2379", "http://10.0.0.22:2379", "http://10.0.0.23:2379"]
scope = "prod-cluster-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000
binaries = {
  postgres = "/usr/pgsql-16/bin/postgres",
  pg_ctl = "/usr/pgsql-16/bin/pg_ctl",
  pg_rewind = "/usr/pgsql-16/bin/pg_rewind",
  initdb = "/usr/pgsql-16/bin/initdb",
  pg_basebackup = "/usr/pgsql-16/bin/pg_basebackup",
  psql = "/usr/pgsql-16/bin/psql",
}

[logging]
level = "info"
capture_subprocess_output = true

[logging.postgres]
enabled = true
poll_interval_ms = 200
cleanup = { enabled = true, max_files = 50, max_age_seconds = 604800, protect_recent_seconds = 300 }

[logging.sinks.stderr]
enabled = true

[logging.sinks.file]
enabled = false
mode = "append"

[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }

[debug]
enabled = false
```

## Why the schema is explicit

The v2 schema is intentionally fail-closed. Startup should fail before the node launches if process binaries, auth identities, TLS material, or DCS settings are incomplete.

## Field groups

### `config_version`

- Must be `v2`.
- `v1` is intentionally unsupported.

### `[cluster]`

- `name` labels the cluster for logs and DCS payloads.
- `member_id` is the stable node identity used in membership and leadership records.

### `[postgres]`

- `data_dir`, `socket_dir`, and `log_file` define local process layout.
- `listen_host` and `listen_port` control local PostgreSQL reachability.
- `rewind_source_host` and `rewind_source_port` are used for rewind and basebackup source connection defaults.
- `local_conn_identity` is used for local control operations.
- `rewind_conn_identity` is used for rewind connectivity.
- `roles.superuser`, `roles.replicator`, and `roles.rewinder` define the PostgreSQL identities used by process jobs.
- `pg_hba.source` and `pg_ident.source` can be inline content or file-backed content.

### `[dcs]`

- `endpoints` must contain at least one reachable etcd endpoint.
- `scope` is the namespace prefix for cluster coordination keys.
- `init` is optional and can seed DCS config during bootstrap when `write_on_bootstrap = true`.

### `[ha]`

- `loop_interval_ms` is the HA decision cadence.
- `lease_ttl_ms` must be greater than `loop_interval_ms`.

### `[process]`

- `pg_rewind_timeout_ms`, `bootstrap_timeout_ms`, and `fencing_timeout_ms` control subprocess deadlines.
- `process.binaries.*` values must be absolute paths.
- The required binaries are:
  - `postgres`
  - `pg_ctl`
  - `pg_rewind`
  - `initdb`
  - `pg_basebackup`
  - `psql`

Replica cloning still depends on `pg_basebackup`, but the removed backup feature does not.

### `[logging]`

- `level` controls application log verbosity.
- `capture_subprocess_output` determines whether subprocess stdout/stderr is captured into process logs.
- `logging.postgres` controls PostgreSQL log tailing and cleanup.
- `logging.sinks.file.path` must not overlap PostgreSQL-owned log inputs.

### `[api]`

- `listen_addr` must be a stable, non-zero listen address.
- `security.tls` controls API transport security.
- `security.auth` controls whether the API is open or token-protected.

### `[debug]`

- `enabled` controls the debug API surface.

## Operational notes

- Initial primary bootstrap uses `initdb`.
- Replica bootstrap uses `pg_basebackup` against a healthy primary when the DCS view shows one.
- There is no restore-bootstrap mode and no pgtuskmaster-owned WAL archive/restore helper path.

## Common misconfigurations

| Symptom | Likely cause | First check |
|---|---|---|
| Startup fails with missing required secure field | incomplete v2 config | required nested blocks and binary paths |
| `pg_basebackup` auth failures | missing replication HBA rules | `pg_hba` replication entries |
| Rewind jobs fail | rewinder auth or privilege mismatch | `postgres.rewind_conn_identity` and rewinder role wiring |
| Node cannot spawn binaries | wrong absolute paths or permissions | `process.binaries.*` |
| Trust degrades repeatedly | unhealthy or inconsistent DCS connectivity | `[dcs] endpoints` and `scope` |

For interface details, see [Node API](../interfaces/node-api.md) and [CLI Workflows](../interfaces/cli.md).
