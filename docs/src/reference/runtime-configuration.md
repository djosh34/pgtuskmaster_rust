# Runtime Configuration Reference

This document describes the TOML schema and validation rules for pgtuskmaster runtime configuration.

## Config Version

```toml
config_version = "v2"
```

`config_version` is a required top-level field that declares the schema variant. The loader rejects `v1` with a migration message and fails if the field is absent. Only `config_version = "v2"` is accepted.

## Top-Level Sections

The normalized runtime config contains exactly these blocks:

- `cluster` - cluster identity
- `postgres` - PostgreSQL instance settings
- `dcs` - distributed consensus store
- `ha` - high-availability loop timing
- `process` - external binary paths and timeouts
- `logging` - log capture, sinks, and cleanup
- `api` - HTTP control plane
- `debug` - developer features

`logging` and `debug` may be omitted; defaults are applied. All other sections are mandatory.

## `cluster`

```toml
[cluster]
name = "docker-cluster"
member_id = "node-a"
```

| Field | Type | Constraints | Default |
|-------|------|-------------|---------|
| `name` | string | non-empty, trimmed | _required_ |
| `member_id` | string | non-empty, trimmed | _required_ |

Validation fails if either field is empty.

## `postgres`

Defines the managed PostgreSQL instance.

```toml
[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "node-a"
listen_port = 5432
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
tls = { mode = "disabled" }
pg_hba = { source = { path = "/etc/pgtuskmaster/pg_hba.conf" } }
pg_ident = { source = { path = "/etc/pgtuskmaster/pg_ident.conf" } }
```

| Field | Type | Constraints | Default |
|-------|------|-------------|---------|
| `data_dir` | path (absolute) | non-empty, absolute | _required_ |
| `connect_timeout_s` | integer | - | 5 |
| `listen_host` | string | non-empty | _required_ |
| `listen_port` | 16-bit integer | > 0 | _required_ |
| `socket_dir` | path (absolute) | non-empty, absolute | _required_ |
| `log_file` | path (absolute) | non-empty, absolute | _required_ |
| `local_conn_identity` | identity block | see below | _required_ |
| `rewind_conn_identity` | identity block | see below | _required_ |
| `tls` | TLS block | see below | _required_ |
| `roles` | roles block | see below | _required_ |
| `pg_hba` | inline-or-path block | non-empty source | _required_ |
| `pg_ident` | inline-or-path block | non-empty source | _required_ |
| `extra_gucs` | map | optional; validated per entry | empty map |

### Connection Identity

Both `local_conn_identity` and `rewind_conn_identity` must be table literals with three keys:

| Subfield | Type | Constraints |
|----------|------|-------------|
| `user` | string | non-empty |
| `dbname` | string | non-empty |
| `ssl_mode` | enum | `disable`, `prefer`, `require`, `verify_ca`, `verify_full` |

If any subfield is missing or empty, normalization fails with a field-specific validation error listing the exact missing key. If `postgres.tls.mode` is `disabled`, the `ssl_mode` must not require TLS (`require`, `verify_ca`, `verify_full`) or validation fails.

### TLS Server Mode

```toml
tls = { mode = "disabled" }
# or
tls = { mode = "optional", identity = { cert_chain = { path = "/path/to/chain.pem" }, private_key = { path = "/path/to/key.pem" } } }
# or
tls = { mode = "required", identity = { ... } }
```

`mode` accepts three values: `disabled`, `optional`, `required`. When the mode is `optional` or `required`, `identity` must be present and contain non-empty `cert_chain` and `private_key` entries. `client_auth` is optional; if present it requires a non-empty `client_ca` path and a `require_client_cert` boolean.

TLS client authentication is not permitted when `mode = "disabled"`.

### Role Definitions

```toml
[postgres.roles.superuser]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/postgres-superuser-password" } }

[postgres.roles.replicator]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/replicator-password" } }

[postgres.roles.rewinder]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/rewinder-password" } }
```

All three roles (`superuser`, `replicator`, `rewinder`) are required. Each role must provide:

| Subfield | Type | Constraints |
|----------|------|-------------|
| `username` | string | non-empty |
| `auth` | auth block | `type = "password"` with a non-empty secret source |

`type = "tls"` is explicitly rejected at validation with an actionable message directing operators to use password auth. The password source follows the `InlineOrPath` convention:

```toml
password = { content = "inline-secret" }
password = { path = "/run/secrets/file" }
```

Both forms must be non-empty.

**Invariants**:
- `postgres.local_conn_identity.user` must equal `postgres.roles.superuser.username`
- `postgres.rewind_conn_identity.user` must equal `postgres.roles.rewinder.username`

### Authorization Rule Files

`pg_hba` and `pg_ident` each accept a nested `source` key pointing to file or inline content. The source must be non-empty. Example:

```toml
pg_hba = { source = { content = "local all all trust\nhost all all 0.0.0.0/0 md5\n" } }
```

### Extra GUCs

`extra_gucs` is an optional map of PostgreSQL parameter name to string value. Each entry is validated against internal rules; reserved parameter names are rejected. No default parameters are injected automatically.

## `dcs`

Distributed consensus store settings.

```toml
[dcs]
endpoints = ["http://etcd:2379"]
scope = "docker-cluster"
```

| Field | Type | Constraints |
|-------|------|-------------|
| `endpoints` | array of strings | non-empty; each entry non-empty after trim |
| `scope` | string | non-empty after trim |
| `init` | init block (optional) | see below |

At least one endpoint is required; empty endpoint strings are rejected.

### DCS Init Payload

```toml
[dcs.init]
payload_json = "{\"cluster\":{\"name\":\"seed\"}}"
write_on_bootstrap = true
```

| Subfield | Type | Constraints |
|----------|------|-------------|
| `payload_json` | string | valid JSON, must deserialize into a full `RuntimeConfig` object |
| `write_on_bootstrap` | boolean | _required when init is present_ |

If `init` is present, `payload_json` must be non-empty and parseable as both JSON and as a `RuntimeConfig`.

## `ha`

High-availability timing parameters.

```toml
[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000
```

| Field | Type | Constraints |
|-------|------|-------------|
| `loop_interval_ms` | integer | 1 <= value <= 86_400_000 |
| `lease_ttl_ms` | integer | 1 <= value <= 86_400_000 |

Validation requires `lease_ttl_ms` > `loop_interval_ms`.

## `process`

External binary locations and operation timeouts.

```toml
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
```

### Timeouts

| Field | Type | Constraints | Default |
|-------|------|-------------|---------|
| `pg_rewind_timeout_ms` | integer | 1 <= value <= 86_400_000 | 120_000 |
| `bootstrap_timeout_ms` | integer | 1 <= value <= 86_400_000 | 300_000 |
| `fencing_timeout_ms` | integer | 1 <= value <= 86_400_000 | 30_000 |

Timeouts are expressed in milliseconds.

### Binary Paths

All six binary fields are required and must be **absolute** paths. The `binaries` table must be present; missing binaries cause validation errors with the prefix `process.binaries.<name>`. Empty or non-absolute paths are rejected.

| Field | Type | Constraints |
|-------|------|-------------|
| `postgres` | absolute path | non-empty |
| `pg_ctl` | absolute path | non-empty |
| `pg_rewind` | absolute path | non-empty |
| `initdb` | absolute path | non-empty |
| `pg_basebackup` | absolute path | non-empty |
| `psql` | absolute path | non-empty |

## `logging`

Controls log capture, routing, and retention.

```toml
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
```

| Field | Type | Constraints | Default |
|-------|------|-------------|---------|
| `level` | enum | `trace`, `debug`, `info`, `warn`, `error`, `fatal` | `info` |
| `capture_subprocess_output` | boolean | - | `true` |
| `postgres` | postgres-log block | see below | see below |
| `sinks` | sinks block | see below | see below |

### PostgreSQL Log Capture

| Subfield | Type | Constraints | Default |
|----------|------|-------------|---------|
| `enabled` | boolean | - | `true` |
| `pg_ctl_log_file` | absolute path (optional) | must be absolute if present | `None` |
| `log_dir` | absolute path (optional) | must be absolute if present | `None` |
| `poll_interval_ms` | integer | 1 <= value <= 86_400_000 | 200 |
| `cleanup` | cleanup block | see below | see below |

**Cleanup Rules**

| Sub-subfield | Type | Constraints | Default |
|--------------|------|-------------|---------|
| `enabled` | boolean | - | `true` |
| `max_files` | integer | > 0 when enabled | 50 |
| `max_age_seconds` | integer | > 0 when enabled | 604_800 (7 days) |
| `protect_recent_seconds` | integer | > 0 when enabled | 300 |

**Path Invariants**
If the runtime log file sink is enabled, its path must not collide with:

- `postgres.log_file`
- `logging.postgres.pg_ctl_log_file` (if configured)
- any file inside `logging.postgres.log_dir` (to avoid self-ingest)

### Sinks

```toml
[logging.sinks.stderr]
enabled = true

[logging.sinks.file]
enabled = false
path = "/optional/path"
mode = "append"
```

| Subfield | Type | Constraints | Default |
|----------|------|-------------|---------|
| `stderr.enabled` | boolean | - | `true` |
| `file.enabled` | boolean | - | `false` |
| `file.path` | absolute path | required when `file.enabled = true` | `None` |
| `file.mode` | enum | `append`, `truncate` | `append` |

If `file.enabled` is `true` and `file.path` is unset or empty, validation fails. The path must be absolute.

## `api`

HTTP control-plane listener and security.

```toml
[api]
listen_addr = "0.0.0.0:8080"

[api.security]
tls = { mode = "disabled" }
auth = { type = "role_tokens", read_token = "read-secret", admin_token = "admin-secret" }
```

| Field | Type | Constraints | Default |
|-------|------|-------------|---------|
| `listen_addr` | socket address string | non-empty | `"127.0.0.1:8080"` |
| `security` | security block | see below | _required_ |

### Security Block

| Subfield | Type | Constraints |
|----------|------|-------------|
| `tls` | TLS server block | same rules as `postgres.tls` (required) |
| `auth` | auth block | `type = "disabled"` or `type = "role_tokens"` (required) |

### Role Token Authentication

```toml
auth = { type = "role_tokens", read_token = "...", admin_token = "..." }
```

| Subfield | Type | Constraints |
|----------|------|-------------|
| `read_token` | string (optional) | non-empty when configured |
| `admin_token` | string (optional) | non-empty when configured |

At least one token must be configured. Empty tokens are rejected.

## `debug`

Developer instrumentation toggle.

```toml
[debug]
enabled = false
```

| Field | Type | Constraints | Default |
|-------|------|-------------|---------|
| `enabled` | boolean | - | `false` |

## Inline or Path Values

Several fields (`password`, `pg_hba.source`, `pg_ident.source`, TLS identity material) accept a union type serialized as one of:

```toml
# Path forms
key = "/absolute/path"
key = { path = "/absolute/path" }

# Inline form
key = { content = "inline value" }
```

For `SecretSource` values, inline content is redacted in `Debug` output while path-backed forms are shown as paths.

## Validation and Diagnostics

All rules above are enforced by `load_runtime_config`. The parser returns a `Validation` error with:

- `field` - dot-separated path to the invalid key
- `message` - human-readable explanation, often suggesting the correct value or pointing out the constraint

Common patterns:

- **Missing required block** - `missing required secure config block for config_version=v2`
- **Missing required field** - `missing required secure field for config_version=v2`
- **Empty value** - `must not be empty`
- **Relative path** - `must be an absolute path`
- **Interval mismatch** - `must be greater than ha.loop_interval_ms`
- **TLS mismatch** - `must not require server TLS when postgres.tls.mode is disabled`
- **Role auth** - `use type = "password" for now` (TLS auth is not implemented)
