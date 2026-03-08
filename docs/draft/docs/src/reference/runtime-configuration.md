# Runtime Configuration Reference

## Overview

The runtime configuration file defines how `pgtuskmaster` operates as a self-managing PostgreSQL node in a high-availability cluster. This reference describes the TOML schema, validation rules, and machinery behavior for version 2 of the configuration format.

[diagram about configuration hierarchy showing top-level sections and nested blocks with field types and cardinalities]

## File Loading

**Entry Point**: `load_runtime_config(path)`  
**Format**: TOML, must be readable as UTF-8 text  
**Required Top-Level Field**: `config_version = "v2"`  

The parser rejects configurations that:
- Omit `config_version` (validation error: `config_version` field missing)
- Declare `config_version = "v1"` (explicit rejection: v1 no longer supported)

The parser normalizes and validates all fields after loading; absent optional fields receive safe defaults.
// todo: Narrow "optional fields" to the specific defaults proven by source. Many secure v2 blocks are required explicitly and do not default.

## Top-Level Structure

`RuntimeConfig` comprises eight mandatory sections and two optional sections:
// todo: This wording is inaccurate. The normalized `RuntimeConfig` has all sections present, while the v2 input schema only makes some sections optional during normalization.

```toml
config_version = "v2"

[cluster]
# cluster identification

[postgres]
# PostgreSQL instance settings

[dcs]
# distributed consensus store endpoints

[ha]
# high-availability timing

[process]
# subprocess execution settings

[logging]                 # optional, defaults applied if absent
# log routing and retention

[api]                     # optional, defaults applied if absent
// todo: `api` is not optional in `RuntimeConfigV2Input`; only `api.listen_addr` defaults if omitted.
# HTTP API binding and security

[debug]                   # optional, defaults applied if absent
# diagnostic features
```

### `cluster`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Logical cluster name; shared by all members of the same cluster |
| `member_id` | string | Yes | Unique identifier for this node within the cluster |

Both fields must be non-empty after whitespace trimming.

### `postgres`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `data_dir` | absolute path | Yes | - | PostgreSQL data directory |
| `connect_timeout_s` | u32 | No | 5 | Seconds to wait for local PostgreSQL connections |
| `listen_host` | string | Yes | - | Bind address for PostgreSQL (`listen_addresses`) |
| `listen_port` | u16 | Yes | - | PostgreSQL port (must be > 0) |
| `socket_dir` | absolute path | Yes | - | Unix socket directory |
| `log_file` | absolute path | Yes | - | PostgreSQL stdout/stderr capture file |
| `local_conn_identity` | inline table | Yes | - | Connection identity for local management queries |
| `rewind_conn_identity` | inline table | Yes | - | Connection identity for `pg_rewind` operations |
| `tls` | inline table | Yes | - | TLS server configuration |
| `roles` | inline table | Yes | - | Role definitions for superuser, replicator, rewinder |
| `pg_hba` | inline table | Yes | - | `pg_hba.conf` content source |
| `pg_ident` | inline table | Yes | - | `pg_ident.conf` content source |
| `extra_gucs` | map of string → string | No | {} | Additional `postgresql.conf` settings |

#### `postgres.local_conn_identity` and `postgres.rewind_conn_identity`

Each identity block defines username, database, and SSL mode for internal connections:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `user` | string | Yes | Connection username |
| `dbname` | string | Yes | Connection database name |
| `ssl_mode` | enum | Yes | SSL requirement: `disable`, `prefer`, `require`, `verify-ca`, `verify-full` |
// todo: The exact list of SSL mode values was not sourced from the reviewed files for this batch. Keep the field as an enum unless the backing type is cited directly.

Validation: `local_conn_identity.user` must equal `postgres.roles.superuser.username`.  
Validation: `rewind_conn_identity.user` must equal `postgres.roles.rewinder.username`.

#### `postgres.tls`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `mode` | enum | Yes | TLS mode: `disabled`, `optional`, `required` |
| `identity` | inline table | Conditional | Certificate and private key when mode ≠ `disabled` |
| `client_auth` | inline table | No | Optional client certificate validation |

`identity` is mandatory when `mode` is `optional` or `required`; forbidden when `disabled`.
// todo: Recheck the exact validation rule from `validate_tls_server_config`. "forbidden when disabled" may overstate the parser behavior.

##### `postgres.tls.identity`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cert_chain` | `InlineOrPath` | Yes | Server certificate chain file or inline PEM |
| `private_key` | `InlineOrPath` | Yes | Private key file or inline PEM |

`InlineOrPath` accepts three forms:
- Bare path: `"/path/to/file"`
- Path object: `{ path = "/path/to/file" }`
- Inline content: `{ content = "pem-string" }`

For secret material, inline content appears redacted in debug output.

#### `postgres.roles`

Defines three mandatory roles with authentication:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `superuser` | inline table | Yes | PostgreSQL superuser identity |
| `replicator` | inline table | Yes | Replication user (physical replication) |
| `rewinder` | inline table | Yes | Rewind user for `pg_rewind` |

Each role table contains:
- `username` (string, required, non-empty)
- `auth` (tagged table, required)

##### Role `auth` tag

`auth.type` selects authentication method:

| Type | Required Fields | Description |
|------|-----------------|-------------|
| `password` | `password` (SecretSource) | Password authentication (only currently supported method) |
| `tls` | - | **Not implemented**; validation rejects with actionable message asking for `type = "password"` |
// todo: The precise validation wording about `type = "password"` should only appear if quoted from source or tests.

#### `postgres.pg_hba` and `postgres.pg_ident`

Both use the same structure:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `source` | `InlineOrPath` | Yes | File path or inline configuration text for the respective `.conf` |

Content must be non-empty; validation checks lexical content only, not semantics.
// todo: "not semantics" is broader than the reviewed source proves.

#### `postgres.extra_gucs`

A flat map of `postgresql.conf` settings. Keys and values undergo validation by `validate_extra_guc_entry`; reserved keys are forbidden. Validation errors cite the specific key and reason.

### `dcs`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `endpoints` | array of strings | Yes | Etcd or compatible endpoint URLs (≥ 1 entry) |
| `scope` | string | Yes | Key prefix for this cluster's data in the store |
| `init` | inline table | No | Optional bootstrap seed data |

Validation: `endpoints` must contain at least one non-empty URL.  
Validation: `scope` must be non-empty after trimming.
// todo: Confirm the exact endpoints invariant from `validate_runtime_config`; avoid claiming URL semantics unless shown directly.

#### `dcs.init`

Optional seed data written to the store during cluster initialization:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `payload_json` | string | Yes | JSON-encoded `RuntimeConfig` document |
| `write_on_bootstrap` | boolean | Yes | Whether to write seed data on bootstrap |

Validation: `payload_json` must be valid JSON and must decode into a `RuntimeConfig` structure.
// todo: Verify whether this decodes into full `RuntimeConfig` or only checks JSON parse plus selected structure.

### `ha`

High-availability loop timing:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `loop_interval_ms` | u64 | Yes | Milliseconds between HA iteration ticks (must be > 0) |
| `lease_ttl_ms` | u64 | Yes | DCS lease time-to-live milliseconds (must be > 0) |

Validation: `lease_ttl_ms` must be strictly greater than `loop_interval_ms`.
// todo: Reconfirm this exact relational invariant in `validate_runtime_config` before stating it this strongly.

### `process`

Subprocess execution and timeouts:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `pg_rewind_timeout_ms` | u64 | No | 120000 | `pg_rewind` command timeout |
| `bootstrap_timeout_ms` | u64 | No | 300000 | `initdb` / `pg_basebackup` timeout |
| `fencing_timeout_ms` | u64 | No | 30000 | Fencing script timeout |
| `binaries` | inline table | Yes | - | Paths to PostgreSQL executables |

#### `process.binaries`

All paths must be absolute and non-empty:

- `postgres`
- `pg_ctl`
- `pg_rewind`
- `initdb`
- `pg_basebackup`
- `psql`

Validation: absolute path requirement enforced; lexically normalized.
// todo: Absolute-path validation is sourced. "lexically normalized" refers to an internal helper and should not be presented as an external config guarantee.

Timeout values are clamped between 1 ms and 86,400,000 ms (24 hours).

### `logging`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `level` | enum | No | `info` | Global log level |
| `capture_subprocess_output` | boolean | No | `true` | Capture child process stdout/stderr into tracing |
| `postgres` | inline table | No | enabled | PostgreSQL log ingestion |
| `sinks` | inline table | No | stderr on, file off | Output routing |
// todo: Replace these shorthand defaults with explicit values from `default_logging_config()`.

#### `logging.level`

- `trace`
- `debug`
- `info`
- `warn`
- `error`
- `fatal`

#### `logging.postgres`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enabled` | boolean | No | `true` | Poll PostgreSQL log file |
| `pg_ctl_log_file` | absolute path | No | - | Override source for `pg_ctl` logs |
| `log_dir` | absolute path | No | - | Directory for native PostgreSQL log files |
| `poll_interval_ms` | u64 | No | 200 | Poll interval for new log content |
| `cleanup` | inline table | No | enabled | Log file retention policy |

##### `logging.postgres.cleanup`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enabled` | boolean | No | `true` | Rotate and expire PostgreSQL logs |
| `max_files` | u64 | No | 50 | Maximum retained files (must be > 0) |
| `max_age_seconds` | u64 | No | 604800 (7 days) | Age ceiling for retention (must be > 0) |
| `protect_recent_seconds` | u64 | No | 300 | Do not delete files newer than this (must be > 0) |

#### `logging.sinks.stderr`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enabled` | boolean | No | `true` | Write formatted logs to stderr |

#### `logging.sinks.file`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enabled` | boolean | No | `false` | Write JSONL logs to file |
| `path` | absolute path | Conditional | - | Target file path (required when enabled) |
| `mode` | enum | No | `append` | File open mode: `append` or `truncate` |

When `enabled = true`, `path` must be configured and non-empty.  
Validation: `path` must not collide with tailed PostgreSQL input files (`postgres.log_file` or `logging.postgres.pg_ctl_log_file`).  
Validation: `path` must not reside inside `logging.postgres.log_dir` (prevents self-ingestion).

### `api`

HTTP API binding and security:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `listen_addr` | string | No | `127.0.0.1:8080` | Socket address to bind |
| `security` | inline table | Yes | - | TLS and authentication |

#### `api.security`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `tls` | inline table | Yes | TLS server settings |
| `auth` | tagged table | Yes | Authentication mode |

`tls` uses the same `TlsServerConfig` structure as `postgres.tls`; validation rules are identical.

#### `api.security.auth`

`auth.type` selects authentication policy:

| Type | Required Fields | Description |
|------|-----------------|-------------|
| `disabled` | - | No API authentication |
| `role_tokens` | `read_token` and/or `admin_token` | Bearer token authentication |

When `role_tokens` is selected, at least one token must be non-empty. Empty strings are rejected. The parser stores tokens in `ApiRoleTokensConfig`.

### `debug`

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enabled` | boolean | No | `false` | Enable debug-only endpoints |

## Validation Rules Summary

The parser enforces these invariants after normalization:

**Path Constraints**
- All binary paths must be absolute.
- All file paths must be non-empty; absolute where required.
- `logging.sinks.file.path` must not overlap with tailed PostgreSQL input files.
- `logging.sinks.file.path` must not be contained within `logging.postgres.log_dir`.

**Identity Matching**
- `postgres.local_conn_identity.user` must equal `postgres.roles.superuser.username`.
- `postgres.rewind_conn_identity.user` must equal `postgres.roles.rewinder.username`.

**TLS Coherence**
- PostgreSQL role auth must use `type = "password"`; `tls` is not implemented.
- When `postgres.tls.mode` is `disabled`, `local_conn_identity.ssl_mode` and `rewind_conn_identity.ssl_mode` may not be `require`, `verify-ca`, or `verify-full`.
- TLS `identity` is mandatory when mode is `optional` or `required`.
- TLS `client_auth` must not be configured when mode is `disabled`.

**Timeouts and Numeric Limits**
- Timeouts must be in `[1, 86_400_000]` ms.
- `ha.loop_interval_ms` must be > 0.
- `ha.lease_ttl_ms` must be > `ha.loop_interval_ms`.
- `logging.postgres.cleanup.*` fields must be > 0 when cleanup is enabled.

**Secret Sources**
- Password `InlineOrPath` must be non-empty: file paths must exist and be non-empty; inline content must be non-empty.
- `pg_hba.source` and `pg_ident.source` must be non-empty.

**DCS and Scope**
- `dcs.endpoints` must contain ≥ 1 non-empty URL.
- `dcs.scope` must be non-empty.
- `dcs.init.payload_json` must be valid JSON and decode to `RuntimeConfig`.

**API Tokens**
- If `api.security.auth` is `role_tokens`, at least one of `read_token` or `admin_token` must be non-empty; blanks are rejected.

**Extra GUCs**
- Keys and values validated by PostgreSQL's parser; reserved keys are forbidden.

## Example Configurations

### Local development (single-node, no TLS, stderr logging only)

```toml
config_version = "v2"

[cluster]
name = "dev-single"
member_id = "localhost"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }

[postgres.roles.superuser]
username = "postgres"
auth = { type = "password", password = { content = "dev-superuser-pass" } }

[postgres.roles.replicator]
username = "replicator"
auth = { type = "password", password = { content = "dev-replicator-pass" } }

[postgres.roles.rewinder]
username = "rewinder"
auth = { type = "password", password = { content = "dev-rewinder-pass" } }

[postgres.pg_hba]
source = { content = "local all all trust" }

[postgres.pg_ident]
source = { content = "# empty" }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "dev"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[logging]
level = "debug"
capture_subprocess_output = true

[logging.sinks.stderr]
enabled = true

[logging.sinks.file]
enabled = false

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }

[debug]
enabled = true
```

### Production cluster (TLS enabled, file logging, secret files)

```toml
config_version = "v2"

[cluster]
name = "prod-cluster"
member_id = "node-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "node-a.internal"
listen_port = 5432
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "verify-full" }
rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "verify-full" }

[postgres.tls]
mode = "required"
identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/server.crt" }, private_key = { path = "/etc/pgtuskmaster/tls/server.key" } }

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
endpoints = ["https://etcd-0.internal:2379", "https://etcd-1.internal:2379", "https://etcd-2.internal:2379"]
scope = "prod"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/lib/postgresql/16/bin/postgres", pg_ctl = "/usr/lib/postgresql/16/bin/pg_ctl", pg_rewind = "/usr/lib/postgresql/16/bin/pg_rewind", initdb = "/usr/lib/postgresql/16/bin/initdb", pg_basebackup = "/usr/lib/postgresql/16/bin/pg_basebackup", psql = "/usr/lib/postgresql/16/bin/psql" }

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

[api.security.tls]
mode = "required"
identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api.crt" }, private_key = { path = "/etc/pgtuskmaster/tls/api.key" } }

[api.security.auth]
type = "role_tokens"
read_token = "READ_TOKEN_PLACEHOLDER"
admin_token = "ADMIN_TOKEN_PLACEHOLDER"

[debug]
enabled = false
```

## Reserved and Forbidden Names

The configuration parser reserves certain keys in `postgres.extra_gucs` that conflict with generated settings managed by `pgtuskmaster`. Attempting to set these via `extra_gucs` results in a validation error citing the reserved key. The exact list of reserved GUCs is determined by the PostgreSQL configuration integration layer; consult source comments in `src/postgres_managed_conf.rs` for the current set.
