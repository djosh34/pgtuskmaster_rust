# Configuration Migration (v2)

The runtime config schema is **fail-closed** by design. In particular:

- `config_version = "v2"` is required.
- Security-sensitive sections (roles/auth, connection identities, TLS posture, and `pg_hba`/`pg_ident`) must be explicitly configured.

This page is an operator-facing checklist and a minimal template you can start from.

## Quick checklist (required blocks)

At minimum, a valid v2 runtime config must define:

- `config_version = "v2"`
- `[cluster]` (`name`, `member_id`)
- `[postgres]` (paths, ports, and rewind source)
  - `local_conn_identity` (explicit user/dbname/ssl_mode)
  - `rewind_conn_identity` (explicit user/dbname/ssl_mode)
  - `tls` (explicit mode; identity required when mode != `disabled`)
  - `roles` (superuser/replicator/rewinder with explicit auth)
  - `pg_hba.source` and `pg_ident.source` (inline or path)
- `[dcs]` (`endpoints`, `scope`)
- `[ha]` (`loop_interval_ms`, `lease_ttl_ms`)
- `[process]` (timeouts optional, but `binaries` required)
- `[api]` with `security.tls` and `security.auth`

## Minimal v2 template

This template is intentionally explicit. Replace placeholder paths and secrets.

```toml
config_version = "v2"

[cluster]
name = "example-cluster"
member_id = "node-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/var/run/pgtuskmaster/sock"
log_file = "/var/log/pgtuskmaster/postgres.log"

rewind_source_host = "127.0.0.1"
rewind_source_port = 5432

# Invariants enforced by validation:
# - local_conn_identity.user must match roles.superuser.username
# - rewind_conn_identity.user must match roles.rewinder.username
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }

tls = { mode = "disabled" }

roles = {
  superuser = { username = "postgres", auth = { type = "tls" } },
  replicator = { username = "replicator", auth = { type = "tls" } },
  rewinder = { username = "rewinder", auth = { type = "tls" } },
}

# Remember: replication connections do not match `database=all`.
# Include explicit `host replication <user> ...` rules when you deploy.
pg_hba = { source = { content = "local all all trust\n" } }
pg_ident = { source = { content = "# empty\n" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "example-scope"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = {
  postgres = "/usr/bin/postgres",
  pg_ctl = "/usr/bin/pg_ctl",
  pg_rewind = "/usr/bin/pg_rewind",
  initdb = "/usr/bin/initdb",
  pg_basebackup = "/usr/bin/pg_basebackup",
  psql = "/usr/bin/psql",
}

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
```

## Common errors and what to do

- `invalid config field \`config_version\`: missing required field; set config_version = "v2" ...`
  - Add `config_version = "v2"` at the top-level.
- `invalid config field \`process.binaries\`: missing required secure field for config_version=v2`
  - Add `[process]` and `process.binaries = { ... }` with all required tool paths.
- `invalid config field \`postgres.roles.superuser.auth.password\`: missing required secure field for config_version=v2`
  - If you choose `type = "password"`, you must also configure the secret:
    - `password = { path = "/path/to/secret" }` or `password = { content = "..." }`.

