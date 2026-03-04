# Configuration Guide

This chapter starts with one recommended production profile and then explains each section in detail. The goal is to make field choices meaningful, not only syntactically valid.

## Recommended production profile (baseline)

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
binaries = {
  postgres = "/usr/pgsql-16/bin/postgres",
  pg_ctl = "/usr/pgsql-16/bin/pg_ctl",
  pg_rewind = "/usr/pgsql-16/bin/pg_rewind",
  initdb = "/usr/pgsql-16/bin/initdb",
  pg_basebackup = "/usr/pgsql-16/bin/pg_basebackup",
  psql = "/usr/pgsql-16/bin/psql",
}

[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
```

## Why this exists

The config model is explicit so startup and HA behavior are predictable. The schema is designed to fail closed when required operational or security-sensitive fields are missing.

## Tradeoffs

Explicit configuration is more verbose than permissive auto-discovery. The benefit is deterministic behavior during failover, rewind, and startup planning. The cost is that operators must supply complete, correct field values.

## When this matters in operations

Most severe incidents begin with implicit assumptions about identity, auth, or replication paths. Explicit config makes those assumptions inspectable before failure.

## Field groups and operational effect

### `config_version`

- Purpose: enables strict schema semantics.
- Operational effect: startup fails early on missing v2-required fields.
- PostgreSQL implication: prevents launching with incomplete auth or process wiring that would fail later.

### `[cluster]`

- `name`: cluster label for operational context.
- `member_id`: stable node identity in DCS membership records.
- Operational effect: unstable IDs can cause role confusion and stale membership records.

### `[postgres]` core paths and listen settings

- `data_dir`, `socket_dir`, `log_file` define local process layout.
- `listen_host`, `listen_port` define local PostgreSQL network presence.
- PostgreSQL implication: invalid directory permissions or path length issues can prevent startup; incorrect listen settings can block probes and replication access.

### PostgreSQL identity blocks

- `local_conn_identity` is used for local SQL/control interactions.
- `rewind_conn_identity` is used for rewind-related connectivity.
- Operational effect: mismatched user identities will fail validation or later job execution.
- PostgreSQL implication: rewind user privileges and auth paths must support `pg_rewind` safely.

### PostgreSQL roles and auth

- `roles.superuser`, `roles.replicator`, `roles.rewinder` define control identities.
- Operational effect: missing replication-compatible auth in `pg_hba` causes basebackup and replication connection failures.
- PostgreSQL implication: replication connections require explicit replication rules and do not match generic database rules.

### `[dcs]`

- `endpoints`: etcd cluster URLs.
- `scope`: namespace prefix for coordination keys.
- Operational effect: scope mismatch isolates node views; endpoint instability degrades trust posture.

### `[ha]`

- `loop_interval_ms`: decision cadence.
- `lease_ttl_ms`: leader lease freshness budget.
- Operational effect: shorter loops detect change faster but increase control-plane activity; TTL influences sensitivity to lease expiration and failover timing.

### `[process]`

- `binaries`: explicit tool paths used for local PostgreSQL and recovery actions.
- Operational effect: missing or wrong paths fail startup or action execution.
- PostgreSQL implication: rewind/bootstrap capability depends directly on correct binary wiring.

### `[api]`

- `listen_addr`: operator access endpoint.
- `security.tls` and `security.auth`: transport and action protection model.
- Operational effect: mismatched auth policy can block control actions or expose unsafe surfaces.

## Misconfiguration symptoms and likely causes

| Symptom or log pattern | Likely cause | First check |
|---|---|---|
| Startup fails with missing required secure field | incomplete v2 config | top-level and required nested blocks |
| `pg_basebackup` auth failures | missing replication HBA rules | `pg_hba` replication entries |
| Rewind jobs fail on permissions/auth | rewinder identity mismatch or privileges | `rewind_conn_identity` and role grants |
| Node cannot find binaries | invalid `process.binaries` paths | absolute binary paths |
| Trust drops repeatedly despite healthy PostgreSQL | etcd endpoint or scope mismatch | `[dcs]` endpoints and `scope` |

For interface-level details, see [Interfaces / Node API](../interfaces/node-api.md) and [Interfaces / CLI Workflows](../interfaces/cli.md).
