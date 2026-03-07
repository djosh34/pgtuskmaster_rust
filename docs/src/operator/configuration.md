# Configuration Guide

This guide describes the current `config_version = "v2"` runtime schema. The main rule to keep in mind is that the parser is explicit, not magical: it refuses missing required blocks, but it does not invent credentials, binary paths, or TLS material for you.

## Production baseline

This example keeps PostgreSQL on a private network, uses password-backed PostgreSQL roles, and protects the node API with required TLS plus role tokens.

```toml
config_version = "v2"

[cluster]
name = "prod-cluster-a"
member_id = "node-a"

[postgres]
data_dir = "/var/lib/postgresql/16/data"
listen_host = "10.0.0.41"
listen_port = 5432
socket_dir = "/var/run/pgtuskmaster"
log_file = "/var/log/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = {
  superuser = { username = "postgres", auth = { type = "password", password = { path = "/etc/pgtuskmaster/secrets/postgres-superuser.password" } } },
  replicator = { username = "replicator", auth = { type = "password", password = { path = "/etc/pgtuskmaster/secrets/replicator.password" } } },
  rewinder = { username = "rewinder", auth = { type = "password", password = { path = "/etc/pgtuskmaster/secrets/rewinder.password" } } },
}
pg_hba = { source = { path = "/etc/pgtuskmaster/pg_hba.conf" } }
pg_ident = { source = { content = "# empty\n" } }
extra_gucs = { shared_preload_libraries = "pg_stat_statements" }

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
enabled = true
path = "/var/log/pgtuskmaster/runtime.jsonl"
mode = "append"

[api]
listen_addr = "10.0.0.41:8080"
security = {
  tls = {
    mode = "required",
    identity = {
      cert_chain = { path = "/etc/pgtuskmaster/tls/api-server.crt" },
      private_key = { path = "/etc/pgtuskmaster/tls/api-server.key" }
    }
  },
  auth = { type = "role_tokens", read_token = "REPLACE_WITH_READ_TOKEN", admin_token = "REPLACE_WITH_ADMIN_TOKEN" }
}

[debug]
enabled = false
```

Two important notes about that example:

- `api.security.auth.role_tokens.*` are plain strings in the schema, so generate the config from your secret management system or another protected deployment path instead of committing real tokens.
- `pg_hba` is operator-supplied. If you use password-backed replication or rewind identities, the HBA rules must allow them.

## Local-only development variant

If you are doing a local lab run on one host, it is acceptable to keep the API loopback-only and disable TLS plus auth:

```toml
[api]
listen_addr = "127.0.0.1:8080"
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
```

Use that only for a machine you control locally. Do not copy it into a remotely reachable environment.

## How to think about each section

### `config_version`

`v2` is the only supported version. The parser intentionally rejects missing required blocks instead of inferring defaults for security-sensitive fields.

### `[cluster]`

- `name` is the cluster label that shows up in state and logs.
- `member_id` is the stable node identity written into DCS records and API responses.

### `[postgres]`

This block controls the local PostgreSQL instance that the node manages.

- `data_dir`, `socket_dir`, and `log_file` define the local filesystem layout.
- `listen_host` and `listen_port` are what the node advertises for PostgreSQL reachability.
- `local_conn_identity` is used for local control queries.
- `rewind_conn_identity` is used when the node needs `pg_rewind`.
- `roles.superuser`, `roles.replicator`, and `roles.rewinder` define the credentials passed into subprocess jobs.
- `pg_hba.source` and `pg_ident.source` can be inline content or file-backed content.
- `extra_gucs` is for PostgreSQL settings that `pgtuskmaster` does not model directly.

`pgtuskmaster` owns the managed startup surface inside `PGDATA`. It writes `pgtm.postgresql.conf`, materializes managed `pg_hba` and `pg_ident` files, rebuilds recovery signal files, and quarantines an active `postgresql.auto.conf` out of the live startup path. If PostgreSQL is running from a different `config_file`, you are outside the supported managed contract.

### `[dcs]`

- `endpoints` must contain reachable etcd URLs.
- `scope` is the namespace used for the cluster's keys.
- `init` is optional and only matters if you want bootstrap-time DCS initialization.

Use one consistent `scope` for all members of the same cluster. A scope mismatch looks like a broken cluster because the nodes simply are not talking about the same records.

### `[ha]`

- `loop_interval_ms` is how often the HA decision loop runs.
- `lease_ttl_ms` must be greater than `loop_interval_ms`.

Short intervals make the cluster react faster, but they also make poor etcd or PostgreSQL behavior show up more aggressively.

### `[process]`

The runtime shells out to real PostgreSQL binaries. Every binary path must be absolute and valid on the node. `pg_basebackup` is required because replica cloning uses it directly, and `pg_rewind` is required for recovery paths that can reuse existing data safely.

### `[logging]`

`pgtuskmaster` emits typed application events and can also capture subprocess output.

- `logging.sinks.stderr` is the default structured log path.
- `logging.sinks.file` is optional and needs an explicit `path` when enabled.
- PostgreSQL log ingest is controlled separately under `logging.postgres`.

The currently supported operator-facing backend surface is JSONL to stderr and optional file sinks. OpenTelemetry export is intentionally not part of the configuration contract yet.

### `[api]`

The API defaults to `127.0.0.1:8080` if you omit `listen_addr`. The rest of the block is required:

- `security.tls` controls whether the listener is plain HTTP, optional TLS, or required TLS
- `security.auth` is either `disabled` or `role_tokens`

For operator-facing deployments, keep the API on a dedicated management interface or behind another secured path. The bad example to avoid is `0.0.0.0:8080` with TLS and auth both disabled.

### `[debug]`

`debug.enabled` controls the extra debug routes. Leave it off unless you actively need the debug surfaces.

## Common misconfigurations

| Symptom | Likely cause | First check |
| --- | --- | --- |
| startup fails before the node binds the API | missing required `v2` block or unreadable secret file | parser error and file permissions |
| `pgtuskmasterctl ha state` cannot connect locally | API bound somewhere other than `127.0.0.1:8080` or listener failed to start | `[api].listen_addr` and startup logs |
| replica bootstrap fails | replication credentials or `pg_hba` rules do not match the source primary | `postgres.roles.replicator` and HBA contents |
| rewind jobs fail | rewinder credentials, privileges, or source reachability are wrong | `postgres.rewind_conn_identity`, `postgres.roles.rewinder`, and DCS leader endpoint |
| runtime file sink never appears | file sink enabled without a usable path or permissions | `[logging.sinks.file]` and filesystem ownership |
