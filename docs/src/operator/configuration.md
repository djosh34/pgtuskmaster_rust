# Configuration Guide

This guide describes the current `config_version = "v2"` runtime schema. The parser is explicit on purpose: it refuses missing security-sensitive fields instead of inventing passwords, token material, or binary paths for you.

The quickest way to see a working configuration is the checked-in container set under `docker/configs/**`. Those files are real runtime configs, not pseudocode. Use them as the default reference point before you translate the runtime into another deployment system.

## Container-first baseline

This is the shape the Compose stacks use today:

```toml
config_version = "v2"

[cluster]
name = "docker-cluster"
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
roles = {
  superuser = { username = "postgres", auth = { type = "password", password = { path = "/run/secrets/postgres-superuser-password" } } },
  replicator = { username = "postgres", auth = { type = "password", password = { path = "/run/secrets/replicator-password" } } },
  rewinder = { username = "postgres", auth = { type = "password", password = { path = "/run/secrets/rewinder-password" } } },
}
pg_hba = { source = { path = "/etc/pgtuskmaster/pg_hba.conf" } }
pg_ident = { source = { path = "/etc/pgtuskmaster/pg_ident.conf" } }

[dcs]
endpoints = ["http://etcd:2379"]
scope = "docker-cluster"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = {
  postgres = "/usr/lib/postgresql/16/bin/postgres",
  pg_ctl = "/usr/lib/postgresql/16/bin/pg_ctl",
  pg_rewind = "/usr/lib/postgresql/16/bin/pg_rewind",
  initdb = "/usr/lib/postgresql/16/bin/initdb",
  pg_basebackup = "/usr/lib/postgresql/16/bin/pg_basebackup",
  psql = "/usr/lib/postgresql/16/bin/psql",
}

[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }

[debug]
enabled = true
```

That baseline is intentionally lab-oriented:

- passwords are still file-backed and mounted through Docker secrets
- PostgreSQL host and replication access inside the Compose bridge use trust-based `pg_hba`
- debug routes are enabled
- the API shares one port for operational and debug routes
- API TLS and tokens are disabled for local-only use

## Hardened operator baseline

For a real operator-facing deployment, keep the same file-backed secret pattern but harden the API surface:

```toml
[api]
listen_addr = "10.0.0.41:8080"
security = {
  tls = {
    mode = "required",
    identity = {
      cert_chain = { path = "/run/secrets/api-server.crt" },
      private_key = { path = "/run/secrets/api-server.key" }
    }
  },
  auth = { type = "role_tokens", read_token = "REPLACE_WITH_READ_TOKEN", admin_token = "REPLACE_WITH_ADMIN_TOKEN" }
}

[debug]
enabled = false
```

Two important caveats still apply:

- `api.security.auth.role_tokens.*` are plain strings in the schema, so render the final TOML from a protected deployment path instead of committing real tokens
- `pg_hba` is still operator-owned. If you use password-backed replication or rewind identities, the HBA rules must allow them

## How to think about each section

### `config_version`

`v2` is the only supported version.

### `[cluster]`

- `name` is the cluster label
- `member_id` is the stable node identity

### `[postgres]`

- `data_dir`, `socket_dir`, and `log_file` define the local filesystem layout
- `listen_host` and `listen_port` are what the node advertises for PostgreSQL reachability
- `roles.*.auth.password.path` is the file-backed password source for each PostgreSQL role
- `pg_hba.source` and `pg_ident.source` can be inline content or file-backed content

### `[dcs]`

- `endpoints` must contain reachable etcd URLs
- `scope` must be shared by every member in the same cluster

### `[ha]`

- `loop_interval_ms` controls the HA loop cadence
- `lease_ttl_ms` must remain greater than `loop_interval_ms`

### `[process]`

Every PostgreSQL binary path must be absolute. The container images in this repo use `/usr/lib/postgresql/16/bin/*`.

### `[api]`

The runtime has one API listener. When `debug.enabled = true`, `/debug/*` routes ride that same listener. Do not invent a second debug port in your deployment configs.

### `[debug]`

Enable it only when you actively need the extra routes. The quick-start stacks turn it on for observability; hardened deployments usually turn it off.

## Common misconfigurations

| Symptom | Likely cause | First check |
| --- | --- | --- |
| startup fails before the node binds the API | missing required `v2` block or unreadable secret file | parser error and file permissions |
| `docker compose config` fails | `.env.docker` points at missing secret files or malformed image/port values | `.env.docker` and the referenced secret files |
| `/debug/verbose` returns `404` unexpectedly | `debug.enabled` is false | the `[debug]` block |
| replica bootstrap fails | replication credentials or `pg_hba` rules do not match the source primary | `postgres.roles.replicator` and HBA contents |
| rewind jobs fail | rewinder credentials, privileges, or source reachability are wrong | `postgres.rewind_conn_identity`, `postgres.roles.rewinder`, and DCS leader endpoint |
