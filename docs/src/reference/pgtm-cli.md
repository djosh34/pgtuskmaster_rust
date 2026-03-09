# pgtm

## Name

pgtm - operator CLI for the PGTuskMaster HA API

## Synopsis

`pgtm [OPTIONS]`

`pgtm [OPTIONS] status [--json] [-v|--verbose] [--watch]`

`pgtm [OPTIONS] switchover request [--switchover-to <member_id>]`

`pgtm [OPTIONS] switchover clear`

## Description

`pgtm` is the operator-facing command-line client for the PGTuskMaster HA API. The normal workflow is to point it at the shared runtime config with `-c config.toml`, then let the CLI resolve the API URL, auth tokens, and API-client TLS settings from that config.

The default operator entry point is cluster status:

- `pgtm` behaves the same as `pgtm status`
- `pgtm status` is the explicit form
- the default presentation is a compact human table
- `--json` switches to the machine-readable cluster view
- `-v` expands the table with deeper node detail
- `--watch` repeats the same cluster sampling loop on an interval

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

NODE    SELF  ROLE     TRUST      PHASE     LEADER  DECISION     PGINFO   READINESS  PROCESS  API
node-a  *     primary  fail_safe  primary   node-a  no_change    ...      ready      idle     ok
node-b        replica  fail_safe  replica   node-a  follow_...   ...      ready      idle     ok
node-c        unknown  unknown    unknown   ?       ?            ?        ?          ?        down
```

The `SELF` marker identifies the node used as the initial seed for cluster discovery. The rendered cluster also records `queried via` so operators can see which node started the sample.

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
- `observation_error`

`pginfo`, `readiness`, and `process` are populated only when `-v --json` is used and debug detail is available from the sampled node.

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
| `4` | API response status did not match the expected success status |
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

# Switchover control
pgtm -c /etc/pgtuskmaster/config.toml switchover request
pgtm -c /etc/pgtuskmaster/config.toml switchover request --switchover-to node-b
pgtm -c /etc/pgtuskmaster/config.toml switchover clear

# Explicit override for troubleshooting
pgtm -c /etc/pgtuskmaster/config.toml --base-url http://127.0.0.1:18081 status
```
