# Configuration Guide

This chapter starts with a complete configuration example and then explains each section in detail. The goal is to make field choices meaningful, not only syntactically valid.

## Configuration example (baseline)

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
# Example paths: adjust to your PostgreSQL installation.
binaries = {
  postgres = "/usr/pgsql-16/bin/postgres",
  pg_ctl = "/usr/pgsql-16/bin/pg_ctl",
  pg_rewind = "/usr/pgsql-16/bin/pg_rewind",
  initdb = "/usr/pgsql-16/bin/initdb",
  pg_basebackup = "/usr/pgsql-16/bin/pg_basebackup",
  psql = "/usr/pgsql-16/bin/psql",
  # Only required when backup.enabled = true (see [backup] below).
  pgbackrest = "/usr/bin/pgbackrest",
}

[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }

[backup]
enabled = false
provider = "pgbackrest"

[backup.pgbackrest]
# Required when backup.enabled = true.
stanza = "prod-cluster-a"
repo = "1"

[backup.pgbackrest.options]
# Extra pgBackRest CLI options, per operation. These are appended to the rendered command.
# For safety, options must not override managed fields (no `--stanza` / `--repo` / `--pg1-path` tokens).
backup = []
info = ["--log-level-console=info"]
check = []
restore = []
archive_push = []
archive_get = []
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
- Operational effect: mismatched user identities can fail validation or later job execution.
- PostgreSQL implication: the rewinder role must be provisioned with enough privileges and correct auth for `pg_rewind`; privilege mistakes surface as runtime command/auth failures (not as a separate privilege validator in this codebase).

### PostgreSQL roles and auth

- `roles.superuser`, `roles.replicator`, `roles.rewinder` define control identities.
- Operational effect: a common failure mode is missing replication-compatible auth in `pg_hba`, which causes basebackup and replication connection failures.
- PostgreSQL implication: replication connections require explicit replication rules and do not match generic database rules.

### `[dcs]`

- `endpoints`: etcd cluster URLs.
- `scope`: namespace prefix for coordination keys.
- Operational effect: scope mismatch isolates node views; endpoint instability degrades trust posture.

### `[ha]`

- `loop_interval_ms`: decision cadence.
- `lease_ttl_ms`: validated guardrail (must be greater than `loop_interval_ms`).
- Operational effect: shorter loops detect change faster but increase control-plane activity; `lease_ttl_ms` is currently enforced as a configuration constraint rather than a separately tunable lease-renewal loop.

### `[process]`

- `binaries`: explicit tool paths used for local PostgreSQL and recovery actions.
- Operational effect: missing or wrong paths fail startup or action execution.
- PostgreSQL implication: rewind/bootstrap capability depends directly on correct binary wiring.

### `[backup]`

Backup/restore operations are provider-driven and executed as process jobs so that:

- subprocess execution is centralized (timeouts, cancellation, output capture),
- orchestration code can stay provider-agnostic, and
- the provider interface can evolve without leaking pgBackRest CLI strings across the codebase.

Fields:

- `enabled`:
  - when `false` (default), backup settings are inert and the node does not require pgBackRest wiring.
  - when `true`, the node requires:
    - `process.binaries.pgbackrest` to be set to a valid executable path
    - `[backup.pgbackrest] stanza` and `repo` to be set and non-empty.
- `provider`: currently only `pgbackrest` is supported.
- `[backup.pgbackrest.options]`: extra pgBackRest CLI options per operation (arrays of strings).
  - For safety and determinism, these option tokens must not override managed fields (no `--stanza` / `--repo` / `--pg1-path`).

#### `backup.bootstrap` (restore bootstrap and config takeover)

`backup.bootstrap` controls whether an *uninitialized* node (a node whose `postgres.data_dir` is `Missing|Empty`) is allowed to restore itself from a backup instead of running `initdb`.

When enabled, pgtuskmaster:

- selects a `RestoreBootstrap` startup mode (instead of `InitializePrimary`) when the cluster is uninitialized,
- runs `pgbackrest restore` to materialize `PGDATA`,
- performs a deterministic takeover of backup-era config artifacts *before PostgreSQL starts*, and
- starts PostgreSQL using a pgtuskmaster-owned config file via `-c config_file=...` so backup-era `postgresql.conf` does not apply.

Example:

```toml
[backup]
enabled = true
provider = "pgbackrest"

[backup.bootstrap]
enabled = true
takeover_policy = "quarantine" # or "delete"
recovery_mode = "default"

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

[backup.pgbackrest]
stanza = "prod-cluster-a"
repo = "1"

[backup.pgbackrest.options]
# You typically need to supply repository configuration here (example: repo1-path).
restore = ["--repo1-path=/var/lib/pgbackrest"]
archive_push = ["--repo1-path=/var/lib/pgbackrest"]
archive_get = ["--repo1-path=/var/lib/pgbackrest"]
```

Postgres log ingestion and cleanup notes:

- `logging.postgres.log_dir` (optional) is scanned for `*.log` and `*.json` and tailed for additional observability signals (for example `postgres.json` when you use `jsonlog`).
- `logging.postgres.cleanup` applies only to `logging.postgres.log_dir` and is designed to be safe-by-default:
  - it never deletes `postgres.json`, `postgres.stderr.log`, or `postgres.stdout.log`,
  - it does not delete files modified within `protect_recent_seconds`, and
  - it treats missing/failed metadata reads conservatively (the file is kept and the cleanup issue is surfaced via structured ingest events like `postgres_ingest.step_once_failed` plus per-iteration debug breadcrumbs like `postgres_ingest.iteration`).
- Path ownership guardrails:
  - if `logging.sinks.file.enabled = true`, `logging.sinks.file.path` must not overlap tailed Postgres inputs and must not be inside `logging.postgres.log_dir` (to avoid self-ingestion loops),
  - `logging.postgres.log_dir` should remain reserved for Postgres-owned log outputs to keep ingestion and cleanup behavior deterministic.

Takeover policy details:

- `quarantine`: moves conflicting files into a timestamped `PGDATA/pgtm.quarantine.*` directory.
- `delete`: removes conflicting files.

Conflicting artifacts removed/quarantined during takeover include:

- `postgresql.conf` and `postgresql.auto.conf`
- `pg_hba.conf` and `pg_ident.conf`
- `recovery.signal` and `standby.signal` (never inherited)
- any stale `pgtm.*` managed artifacts

### `[api]`

- `listen_addr`: operator access endpoint.
- `security.tls` and `security.auth`: transport and action protection model.
- Operational effect: mismatched auth policy can block control actions or expose unsafe surfaces.

## Misconfiguration symptoms and likely causes

| Symptom or log pattern | Likely cause | First check |
|---|---|---|
| Startup fails with missing required secure field | incomplete v2 config | top-level and required nested blocks |
| `pg_basebackup` auth failures | missing replication HBA rules | `pg_hba` replication entries |
| Rewind jobs fail on permissions/auth | rewinder identity mismatch, missing auth, or insufficient privileges | `rewind_conn_identity` wiring, rewinder auth, and DB grants (privilege failures surface at runtime) |
| Node cannot find binaries | invalid `process.binaries` paths | `process.binaries` values (path correctness/executability) |
| Backup enabled fails validation | missing `process.binaries.pgbackrest` or `[backup.pgbackrest] stanza/repo` | `process.binaries.pgbackrest` and `[backup]` configuration |
| Trust drops repeatedly despite healthy PostgreSQL | DCS store is unhealthy (NotTrusted) or membership/leader invariants are inconsistent (FailSafe) | `[dcs]` endpoints and `scope`, plus DCS membership/leader consistency |

For interface-level details, see [Interfaces / Node API](../interfaces/node-api.md) and [Interfaces / CLI Workflows](../interfaces/cli.md).
