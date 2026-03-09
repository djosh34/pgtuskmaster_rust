# pgtm

## Name

pgtm - operator CLI for the PGTuskMaster HA API

## Synopsis

`pgtm [OPTIONS]`

`pgtm [OPTIONS] status [--json] [-v|--verbose] [--watch]`

`pgtm [OPTIONS] debug verbose [--json] [--since <sequence>]`

`pgtm [OPTIONS] primary [--json] [--tls]`

`pgtm [OPTIONS] replicas [--json] [--tls]`

`pgtm [OPTIONS] switchover request [--switchover-to <member_id>]`

`pgtm [OPTIONS] switchover clear`

## Description

`pgtm` is the operator-facing command-line client for the PGTuskMaster HA API. The normal workflow is to point it at the shared runtime config with `-c config.toml`, then let the CLI resolve the API URL, auth tokens, and API-client TLS settings from that config.

The default operator entry point is cluster status:

- `pgtm` behaves the same as `pgtm status`
- `pgtm status` is the explicit form
- the default presentation is a compact human table
- `--json` switches to the machine-readable cluster view
- `-v` expands the table with deeper node detail, explicit debug availability, and a per-node debug detail block
- `--watch` repeats the same cluster sampling loop on an interval

`pgtm` also exposes shell-oriented connection helpers that resolve the current primary or currently sampled replicas into libpq keyword/value DSNs. These helpers are the supported way to feed the cluster view into `psql`, scripts, or automation without scraping the status table.

For deeper single-node inspection, `pgtm debug verbose` reads the stable `/debug/verbose` payload and renders an operator-oriented summary by default. Add `--json` when you need the raw stable payload for automation or archival capture.

## Global Options

| Option | Type | Default | Environment | Notes |
| --- | --- | --- | --- | --- |
| `-c`, `--config` | path | unset | none | Shared runtime config path for config-backed operator context |
| `--base-url` | string | unset | none | Explicit seed API URL override; takes precedence over config-derived target |
| `--read-token` | string | unset | none | Explicit read token override; otherwise `pgtm` resolves config-backed auth |
| `--admin-token` | string | unset | none | Explicit admin token override; otherwise `pgtm` resolves config-backed auth |
| `--timeout-ms` | u64 | `5000` | none | HTTP client timeout in milliseconds |
| `--json` | flag | `false` | none | Emit machine-readable JSON instead of the default human output |
| `-v`, `--verbose` | flag | `false` | none | Add deeper per-node detail to `status` |
| `--watch` | flag | `false` | none | Repeat `status` sampling and redraw the result |

`pgtm` resolves operator context with this precedence:

1. `--base-url`, `--read-token`, and `--admin-token` override everything else when provided.
2. `[pgtm].api_url` overrides the API target derived from `api.listen_addr`.
3. Auth tokens come from `api.security.auth` secret sources in the shared config.
4. API-client TLS material comes from `[pgtm.api_client]`.

Read operations use the read token first and fall back to the admin token when no read token is configured. Switchover commands require an admin token when API auth is enabled.

## Command Hierarchy

```text
pgtm (= status)
├── debug
│   └── verbose
├── primary
├── replicas
└── switchover
    ├── clear
    └── request
```

## Commands

### `status`

Fetches a cluster-oriented HA view.

- Seed HTTP method: `GET`
- Seed path: `/ha/state`
- Auth role: read, with fallback to admin

`pgtm status` starts from one API target, reads the stable `/ha/state` payload, discovers peer API URLs from the stable member list, and then samples those peers to synthesize a cluster view. That means the JSON emitted by `pgtm status --json` is not a raw `/ha/state` payload from a single node. It is the CLI's aggregated cluster view.

The default human output is intentionally compact:

```text
cluster: prod-eu1  health: healthy
queried via: node-a

NODE    SELF  ROLE     TRUST         PHASE    API
node-a  *     primary  full_quorum   primary  ok
node-b        replica  full_quorum   replica  ok
node-c        replica  full_quorum   replica  ok
```

When `-v` is enabled, the same view grows rather than switching to a different command:

```text
cluster: prod-eu1  health: degraded
queried via: node-a
warning: node-c could not be sampled: transport error: ...

NODE    SELF  ROLE     TRUST      PHASE     LEADER  DECISION     PGINFO   READINESS  PROCESS  DEBUG      API
node-a  *     primary  fail_safe  primary   node-a  no_change    ...      ready      idle     available  ok
node-b        replica  fail_safe  replica   node-a  follow_...   ...      ready      idle     disabled   ok
node-c        unknown  unknown    unknown   ?       ?            ?        ?          ?        ?          down

debug details:
  node-a: debug=available
    dcs: trust=FullQuorum leader=node-a
    ha: phase=Primary decision=NoChange detail=steady
    pginfo: variant=Primary sql=Healthy readiness=Ready summary=primary wal_lsn=7 readiness=Ready
  node-b: debug=disabled
    detail: http 404: debug disabled
    no debug payload
  node-c: debug not requested
```

The `SELF` marker identifies the node used as the initial seed for cluster discovery. The rendered cluster also records `queried via` so operators can see which node started the sample.

The `DEBUG` column tells you whether `/debug/verbose` was available for each sampled node:

- `available`: debug payload fetched successfully
- `disabled`: the node returned `404 Not Found`, which usually means `debug.enabled = false`
- `auth_failed`: the node rejected the debug read with `401` or `403`
- `not_ready`: the node returned `503` while the debug subsystem was not ready
- `transport_failed`, `decode_failed`, or `api_status_failed`: the seed `/ha/state` read succeeded, but the debug read failed for the specific reason shown in the detail block or JSON output

`--watch` repeats the same cluster sampling path. Human mode redraws the table. JSON mode prints one full JSON document per tick.

### `status --json`

Emits the synthesized cluster view instead of the human table.

The top-level JSON shape includes:

- `cluster_name`
- `scope`
- `verbose`
- `queried_via`
- `sampled_member_count`
- `discovered_member_count`
- `health`
- `warnings`
- `switchover`
- `nodes`

`queried_via` always records both the seed member identity and the seed API URL so automation can see which node started the cluster sample.

Each `nodes[]` entry includes:

- `member_id`
- `is_self`
- `sampled`
- `api_url`
- `api_status`
- `role`
- `trust`
- `phase`
- `leader`
- `decision`
- `pginfo`
- `readiness`
- `process`
- `debug`
- `observation_error`

`pginfo`, `readiness`, `process`, and `debug` are populated only when `-v --json` is used. `debug.status` always records whether the CLI fetched `/debug/verbose` successfully, and `debug.payload` contains the full stable verbose payload when it is available.

### `debug verbose`

Fetches the stable `/debug/verbose` payload from the selected API target.

- HTTP method: `GET`
- Path: `/debug/verbose`
- Auth role: read, with fallback to admin

The default output is a compact incident-oriented summary. It keeps the full payload available behind `--json`, but the human rendering is structured for quick operator review:

- identity header with member, cluster, scope, and current sequence
- summary lines for `pginfo`, `dcs`, `ha`, `process`, and debug-retention state
- bounded recent `changes` and `timeline` excerpts

Example default output:

```text
member: node-a  cluster: cluster-a  scope: scope-a
api url: http://127.0.0.1:8080
sequence: 42  schema: v1

pginfo: variant=Primary sql=Healthy readiness=Ready summary=primary wal_lsn=7 readiness=Ready
dcs: trust=FullQuorum leader=node-a members=3 switchover_request=false
ha: phase=Primary decision=NoChange detail=steady planned_actions=0
process: state=Idle worker=Running running_job=none last_outcome=Success(job-1)
debug: history_changes=12 history_timeline=12 last_sequence=42

recent changes:
  - #41 ha decision updated
recent timeline:
  - #42 ha primary steady
```

### `debug verbose --json`

Emits the raw stable `/debug/verbose` payload exactly as returned by the API. This is the machine-readable form to use when you want the complete `changes` and `timeline` arrays without the CLI summary.

### `debug verbose --since <sequence>`

Passes the `since` cursor through to `/debug/verbose?since=<sequence>`.

- only `changes` and `timeline` are filtered
- the other top-level sections still describe the current snapshot
- the default human rendering keeps the same layout while showing only the filtered retained history

### `primary`

Resolves the current primary from the sampled cluster view and prints one libpq keyword/value DSN.

- Sampling path: same cluster-wide peer discovery and sampling flow as `status`
- Auth role: read, with fallback to admin
- Default output contract: exactly one DSN line with no headers or commentary

Example default output:

```text
host=node-a.db.example.com port=5432 user=postgres dbname=postgres
```

`pgtm primary` is intentionally strict. It fails closed when the CLI cannot form an authoritative write-target answer, including:

- no sampled primary
- multiple sampled primaries
- incomplete peer sampling
- leader or membership disagreement across sampled nodes
- missing PostgreSQL host or port metadata

### `primary --json`

Emits a structured connection view instead of a single text line.

The top-level JSON shape includes:

- `cluster_name`
- `scope`
- `kind`
- `tls`
- `sampled_member_count`
- `discovered_member_count`
- `warnings`
- `targets`

Each `targets[]` entry includes:

- `member_id`
- `postgres_host`
- `postgres_port`
- `dsn`

### `primary --tls`

Expands the DSN with PostgreSQL client TLS fields.

- always adds `sslmode=verify-full`
- adds `sslrootcert`, `sslcert`, and `sslkey` only when the effective config resolves to path-backed client material
- uses `[pgtm.postgres_client]` first and falls back to `[pgtm.api_client]` when the PostgreSQL client block is absent
- fails instead of printing misleading partial TLS flags when the effective certificate or key came from inline or env-backed content that has no safe filesystem path to emit

Example:

```text
host=node-a.db.example.com port=5432 user=postgres dbname=postgres sslmode=verify-full sslrootcert=/etc/pgtm/postgres-ca.pem sslcert=/etc/pgtm/postgres.crt sslkey=/run/secrets/postgres.key
```

### `replicas`

Resolves currently sampled replicas from the same cluster-wide sampling path and prints one DSN per line.

Example default output:

```text
host=node-b.db.example.com port=5432 user=postgres dbname=postgres
host=node-c.db.example.com port=5432 user=postgres dbname=postgres
```

`replicas` is less strict than `primary`: it does not require every discovered member to be reachable before returning results. It returns the currently sampled replica targets and omits unsampled or non-replica members. Contradictory sampled views, such as leader or membership disagreement, still fail the command.

`replicas --json` and `replicas --tls` follow the same contracts as `primary --json` and `primary --tls`.

### `switchover clear`

Clears the current switchover request.

- HTTP method: `DELETE`
- Path: `/ha/switchover`
- Auth role: admin
- Expected success payload: `{"accepted": true}` or `{"accepted": false}`

Without `--json`, the response is rendered as `accepted=<bool>`.

### `switchover request`

Submits a switchover request.

- HTTP method: `POST`
- Path: `/switchover`
- Auth role: admin
- Request body: `{}` or `{"switchover_to":"<member_id>"}`

Add `--switchover-to <member_id>` to request a specific eligible replica. If you omit it, the command submits a generic planned switchover request and the runtime chooses the successor automatically from observed cluster state.

Without `--json`, the response is rendered as `accepted=<bool>`.

## Exit Codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `2` | Clap usage failure before command execution |
| `3` | Transport or request-build error |
| `4` | API response status or connection-resolution failure (for example no authoritative primary or no sampled replicas) |
| `5` | Response decode or output serialization error |
| `6` | Config resolution failure (`-c` content, derived API target, env-backed secret, or incompatible auth/TLS settings) |

## Examples

```bash
# Default status path
pgtm -c /etc/pgtuskmaster/config.toml

# Explicit status
pgtm -c /etc/pgtuskmaster/config.toml status

# Machine-readable cluster view
pgtm -c /etc/pgtuskmaster/config.toml --json

# Deeper operator detail
pgtm -c /etc/pgtuskmaster/config.toml status -v

# Repeated observation
pgtm -c /etc/pgtuskmaster/config.toml status --watch

# Inspect one node's stable debug payload through the CLI
pgtm -c /etc/pgtuskmaster/config.toml debug verbose

# Poll only new retained debug history entries after sequence 42
pgtm -c /etc/pgtuskmaster/config.toml debug verbose --since 42

# Export the raw stable verbose payload for automation
pgtm -c /etc/pgtuskmaster/config.toml --json debug verbose

# Connect to the current primary without scraping table output
psql "$(pgtm -c /etc/pgtuskmaster/config.toml primary)"

# Inspect currently sampled replica connection targets
pgtm -c /etc/pgtuskmaster/config.toml replicas

# Export a TLS-expanded DSN contract
pgtm -c /etc/pgtuskmaster/config.toml primary --tls

# Switchover control
pgtm -c /etc/pgtuskmaster/config.toml switchover request
pgtm -c /etc/pgtuskmaster/config.toml switchover request --switchover-to node-b
pgtm -c /etc/pgtuskmaster/config.toml switchover clear

# Explicit override for troubleshooting
pgtm -c /etc/pgtuskmaster/config.toml --base-url http://127.0.0.1:18081 status
```
