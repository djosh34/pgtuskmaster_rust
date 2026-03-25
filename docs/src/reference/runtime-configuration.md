# Runtime Configuration Reference

This document describes the current runtime TOML schema for `pgtuskmaster`.

The runtime parser owns validation. It reads TOML into internal schema types, applies defaults, resolves supported path and secret encodings, and returns a fully validated runtime config.

## Top-level sections

Runtime config supports these top-level blocks:

- `cluster`
- `postgres`
- `dcs`
- `ha`
- `process`
- `logging`
- `api`
- `pgtm`
- `debug`

Only `cluster`, `postgres`, and `dcs` are always required. The remaining sections have defaults and may be omitted when the defaults fit your deployment.

## Example

```toml
[cluster]
name = "docker-cluster"
scope = "docker-cluster"
member_id = "node-a"

[postgres.paths]
data_dir = "/var/lib/postgresql/data"

[postgres.network]
listen_host = "node-a"
listen_port = 5432

[postgres.access]
hba = { path = "/etc/pgtuskmaster/pg_hba.conf" }
ident = { path = "/etc/pgtuskmaster/pg_ident.conf" }

[postgres]
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { path = "/run/secrets/postgres-superuser-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { path = "/run/secrets/replicator-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { path = "/run/secrets/rewinder-password" } } } }

[dcs]
endpoints = ["http://etcd:2379"]

[api]
listen_addr = "0.0.0.0:8080"
transport = { transport = "http" }
auth = { type = "disabled" }
```

## `cluster`

```toml
[cluster]
name = "docker-cluster"
scope = "docker-cluster"
member_id = "node-a"
```

| Field | Type | Notes |
|-------|------|-------|
| `name` | string | required, non-empty |
| `scope` | string | required, non-empty; this is the DCS key prefix |
| `member_id` | string | required, non-empty |

`cluster.scope` is the single source of truth for the DCS namespace. The old `dcs.scope` field no longer exists.

## `postgres`

`postgres` is split into smaller typed sub-blocks instead of one large flat table.

### Paths

```toml
[postgres.paths]
data_dir = "/var/lib/postgresql/data"
socket_dir = "/var/lib/pgtuskmaster/socket"
log_file = "/var/log/pgtuskmaster/postgres.log"
```

| Field | Type | Notes |
|-------|------|-------|
| `data_dir` | path | required |
| `socket_dir` | path | optional; defaults under `process.working_root` |
| `log_file` | path | optional; defaults under `process.working_root` |

### Network

```toml
[postgres.network]
listen_host = "node-a"
listen_port = 5432
cluster_advertise = { host = "node-a", port = 5432 }
operator_advertise = { host = "node-a", port = 15432, hostaddr = "127.0.0.1" }
```

| Field | Type | Notes |
|-------|------|-------|
| `listen_host` | string | defaults to `127.0.0.1` |
| `listen_port` | integer | defaults to `5432` |
| `cluster_advertise` | route table | optional; defaults to `listen_host` + `listen_port` |
| `operator_advertise` | route table | optional; preferred by `pgtm primary` and `pgtm replicas` when present |

Each advertise route accepts:

- `host`
- `port`
- optional `hostaddr`

`cluster_advertise` is the intra-cluster PostgreSQL route that HA, replication, and rewind use.

`operator_advertise` is a separate operator-facing PostgreSQL route. Use it when host-side clients must reach PostgreSQL through a different address or port than the cluster's internal HA path.

### Local database and rewind transport

```toml
[postgres]
local_database = "postgres"

[postgres.rewind.transport]
ssl_mode = "verify_full"
ca_cert = { path = "/etc/pgtuskmaster/tls/postgres-ca.pem" }
```

| Field | Type | Notes |
|-------|------|-------|
| `local_database` | string | defaults to `postgres` |
| `rewind.database` | string | defaults to `postgres` |
| `rewind.transport.ssl_mode` | enum | `disable`, `prefer`, `require`, `verify_ca`, `verify_full` |
| `rewind.transport.ca_cert` | inline-or-path | optional; required for `verify_ca` or `verify_full` |

The old `local_conn_identity` and `rewind_conn_identity` config blocks were removed. Local connections use `postgres.roles.mandatory.superuser` plus `postgres.local_database`, and rewind uses `postgres.roles.mandatory.rewinder` plus `postgres.rewind`.

### PostgreSQL TLS server settings

```toml
[postgres]
tls = { mode = "disabled" }
tls = { mode = "enabled", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/postgres-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/postgres-key.pem" } } }
tls = { mode = "enabled", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/postgres-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/postgres-key.pem" } }, client_auth = { client_ca = { path = "/etc/pgtuskmaster/tls/client-ca.pem" }, client_certificate = "required" } }
```

`mode` accepts:

- `disabled`
- `enabled`

When enabled, `identity.cert_chain` and `identity.private_key` are required.

If `client_auth` is present, it uses:

- `client_ca`
- `client_certificate = "optional"` or `client_certificate = "required"`

### Roles

```toml
[postgres.roles.mandatory.superuser]
username = "postgres"
auth = { type = "password", password = { path = "/run/secrets/postgres-superuser-password" } }

[postgres.roles.mandatory.replicator]
username = "replicator"
auth = { type = "password", password = { path = "/run/secrets/replicator-password" } }

[postgres.roles.mandatory.rewinder]
username = "rewinder"
auth = { type = "password", password = { path = "/run/secrets/rewinder-password" } }

[postgres.roles.extra.analytics]
username = "analytics"
auth = { type = "password", password = { path = "/run/secrets/analytics-password" } }
privilege = "login"
member_of = ["readers"]
```

The `mandatory` block is always required and always contains:

- `superuser`
- `replicator`
- `rewinder`

The optional `extra` block adds operator-managed PostgreSQL roles keyed by logical role name. Extra role keys must not shadow `superuser`, `replicator`, or `rewinder`.

Only password auth is supported:

```toml
auth = { type = "password", password = { type = "string", value = "inline-secret" } }
auth = { type = "password", password = { path = "/run/secrets/password-file" } }
auth = { type = "password", password = { type = "env", env = "PASSWORD_ENV_VAR" } }
```

Every managed PostgreSQL role must have a unique username. Validation rejects duplicate usernames across the mandatory and extra role sets.

### Access files

```toml
[postgres.access]
hba = { path = "/etc/pgtuskmaster/pg_hba.conf" }
ident = { path = "/etc/pgtuskmaster/pg_ident.conf" }
```

Both fields are required and accept inline-or-path values.

### Extra GUCs

```toml
[postgres.extra_gucs]
wal_keep_size = "128MB"
```

`extra_gucs` is optional. Reserved keys are rejected. That reserved set includes pgtuskmaster-owned startup and logging settings such as `log_destination` and `logging_collector`, because managed PostgreSQL now always enables:

- `logging_collector = on`
- `log_destination = 'jsonlog,stderr'`

## `dcs`

```toml
[dcs]
endpoints = ["http://etcd:2379"]

[dcs.client]
auth = { type = "disabled" }
tls = { mode = "disabled" }
```

| Field | Type | Notes |
|-------|------|-------|
| `endpoints` | array of typed URLs | required, non-empty |
| `client.auth` | auth block | defaults to disabled |
| `client.tls` | TLS block | defaults to disabled |
| `init` | init block | optional |

### DCS client auth

```toml
[dcs.client]
auth = { type = "basic", username = "etcd-user", password = { path = "/run/secrets/etcd-password" } }
```

### DCS client TLS

```toml
[dcs.client]
tls = { mode = "enabled", ca_cert = { path = "/etc/pgtuskmaster/tls/etcd-ca.pem" } }
tls = { mode = "enabled", ca_cert = { path = "/etc/pgtuskmaster/tls/etcd-ca.pem" }, identity = { cert = { path = "/etc/pgtuskmaster/tls/etcd-client.crt" }, key = { path = "/etc/pgtuskmaster/tls/etcd-client.key" } }, server_name = "etcd.internal" }
```

### DCS bootstrap init

```toml
[dcs.init]
payload_json = "{\"cluster\":{\"name\":\"seed\",\"scope\":\"seed\",\"member_id\":\"node-a\"}}"
write_on_bootstrap = true
```

`payload_json` must be valid JSON and must decode to a full runtime config snapshot.

## `ha`

```toml
[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000
```

Defaults exist for both values. Validation requires `lease_ttl_ms > loop_interval_ms`.

## `process`

`process` separates timeouts, working-root defaults, and optional PostgreSQL binary paths.

```toml
[process]
working_root = "/tmp/pgtuskmaster"

[process.timeouts]
pg_rewind_ms = 120000
bootstrap_ms = 300000
fencing_ms = 30000

[process.binaries]
pg_ctl = "/usr/lib/postgresql/16/bin/pg_ctl"
initdb = "/usr/lib/postgresql/16/bin/initdb"
```

### Timeouts

All three timeout fields default to sane values and remain overridable:

- `process.timeouts.pg_rewind_ms`
- `process.timeouts.bootstrap_ms`
- `process.timeouts.fencing_ms`

### Working root

`process.working_root` defaults to `/tmp/pgtuskmaster`. When you do not override the PostgreSQL socket dir or log file, the runtime derives them from that working root.

### Binary resolution

Explicit binary paths are optional.

When a path is not present, the runtime searches:

1. `PATH`
2. conventional PostgreSQL install directories

If autodiscovery fails, the runtime reports which `process.binaries.*` field to set explicitly.

## `logging`

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

Managed PostgreSQL always writes collector output with `logging_collector = on` and `log_destination = 'jsonlog,stderr'`. The postgres ingest worker still follows both JSON log files and plain `.log` output, because helper and archive-command output can still land outside the JSON stream.

`postgres.paths.log_file` is the authoritative `pg_ctl` startup log path. `logging.postgres.log_dir` remains the override for collector-managed postgres log files that the ingest worker tails.

## `api`

`api` no longer nests everything under `security`.

```toml
[api]
listen_addr = "0.0.0.0:8443"
transport = { transport = "http" }
auth = { type = "disabled" }

[api]
listen_addr = "0.0.0.0:8443"
transport = { transport = "https", tls = { identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } }, client_auth = { client_certificate = "required", client_ca = { path = "/etc/pgtuskmaster/tls/client-ca.pem" }, allowed_common_names = ["operator-a"] } } }
auth = { type = "role_tokens", read_token = { type = "env", env = "PGTM_READ_TOKEN" }, admin_token = { path = "/run/secrets/admin-token" } }
```

### Transport

`api.transport` accepts:

- `http`
- `https`

HTTPS requires a `tls.identity` block.

Optional API client auth uses:

- `client_certificate = "optional"`
- `client_certificate = "required"`

The `required` mode may also include `allowed_common_names`.

### API auth

`api.auth` accepts:

- `type = "disabled"`
- `type = "role_tokens"`

Role-token auth can set either or both:

- `read_token`
- `admin_token`

## `pgtm`

The runtime may include an operator-facing `pgtm` block, and `pgtm` can also load a standalone operator-only TOML file.

### Runtime-side `pgtm` block

```toml
[pgtm.api]
base_url = "https://db-a.example.com:8443"
expected_transport = "https"
auth = { type = "role_tokens", read_token = { path = "/run/secrets/api-read-token" }, admin_token = { path = "/run/secrets/api-admin-token" } }

[pgtm.client_tls]
ca_cert = { path = "/etc/pgtm/ca.pem" }
identity = { cert = { path = "/etc/pgtm/client.crt" }, key = { path = "/etc/pgtm/client.key" } }
```

### Meaning

- `pgtm.api.base_url`: operator-reachable API URL
- `pgtm.api.expected_transport`: optional client-side check for `http` or `https`
- `pgtm.api.auth`: operator token config
- `pgtm.client_tls`: shared operator client TLS material used for HTTPS API calls and for `pgtm primary --tls` / `pgtm replicas --tls` output after target selection has already been derived from `GET /state`

## `debug`

```toml
[debug]
enabled = false
```

`debug.enabled` defaults to `false`.

## Inline, path, and secret encodings

Path-backed fields accept:

```toml
key = "/absolute/path"
key = { path = "/absolute/path" }
key = { content = "inline value" }
```

Secret-backed fields accept:

```toml
key = { path = "/run/secrets/value" }
key = { type = "file", path = "/run/secrets/value" }
key = { type = "env", env = "ENV_VAR_NAME" }
key = { type = "string", value = "inline-secret" }
```

Inline secret values are redacted in debug output.

## Validation notes

Common validation patterns include:

- non-empty `cluster.name`, `cluster.scope`, and `cluster.member_id`
- at least one DCS endpoint
- DCS HTTPS endpoints requiring `dcs.client.tls`
- non-empty `dcs.client.auth.username` for basic auth
- `lease_ttl_ms > loop_interval_ms`
- readable path-backed binaries when `process.binaries.*` is set
- TLS client/server combinations that require CA material when verification modes demand it

If validation fails, the loader reports a stable dotted field path such as `cluster.scope` or `process.binaries.initdb`.
