# pgtm

## Name

pgtm - HA admin CLI for PGTuskMaster API

## Synopsis

`pgtm [OPTIONS] <COMMAND>`

## Description

`pgtm` is the operator-facing command-line client for the PGTuskMaster HA API. The normal workflow is to point it at the shared runtime config with `-c config.toml`, then let the CLI resolve the API URL, auth tokens, and API-client TLS settings from that config.

## Global Options

| Option | Type | Default | Environment | Notes |
| --- | --- | --- | --- | --- |
| `-c`, `--config` | path | unset | none | Shared runtime config path for config-backed operator context |
| `--base-url` | string | unset | none | Explicit API URL override; takes precedence over config-derived target |
| `--read-token` | string | unset | none | Explicit read token override; otherwise `pgtm` resolves config-backed auth |
| `--admin-token` | string | unset | none | Explicit admin token override; otherwise `pgtm` resolves config-backed auth |
| `--timeout-ms` | u64 | `5000` | none | HTTP client timeout in milliseconds |
| `--output` | `json` or `text` | `json` | none | Output renderer |

`pgtm` resolves operator context with this precedence:

1. `--base-url`, `--read-token`, and `--admin-token` override everything else when provided.
2. `[pgtm].api_url` overrides the API target derived from `api.listen_addr`.
3. Auth tokens come from `api.security.auth` secret sources in the shared config.
4. API-client TLS material comes from `[pgtm.api_client]`.

Read operations use the read token first and fall back to the admin token when no read token is configured. Switchover commands require an admin token when API auth is enabled.

## Command Hierarchy

```text
pgtm
├── status
└── switchover
    ├── clear
    └── request
```

## Commands

### `status`

Fetches the current HA state snapshot.

- HTTP method: `GET`
- Path: `/ha/state`
- Auth role: read, with fallback to admin

JSON output contains these top-level fields:

- `cluster_name`
- `scope`
- `self_member_id`
- `leader`
- `switchover_pending`
- `switchover_to`
- `member_count`
- `dcs_trust`
- `ha_phase`
- `ha_tick`
- `ha_decision`
- `snapshot_sequence`

Text output renders the same state as newline-delimited `key=value` pairs. Missing `leader` and `switchover_to` values are rendered as `<none>`.

### `switchover clear`

Clears the current switchover request.

- HTTP method: `DELETE`
- Path: `/ha/switchover`
- Auth role: admin
- Expected success payload: `{"accepted": true}` or `{"accepted": false}`

In text mode the response is rendered as `accepted=<bool>`.

### `switchover request`

Submits a switchover request.

- HTTP method: `POST`
- Path: `/switchover`
- Auth role: admin
- Request body: `{}` or `{"switchover_to":"<member_id>"}`

Add `--switchover-to <member_id>` to request a specific eligible replica. If you omit it, the command submits a generic planned switchover request and the runtime chooses the successor automatically from observed cluster state.

In text mode the response is rendered as `accepted=<bool>`.

## Output Formats

`json` pretty-prints the decoded API response.

`text` renders:

- `status` as fixed `key=value` lines
- switchover responses as `accepted=<bool>`

The `ha_decision` field is rendered in text as a compact variant string such as `no_change`, `become_primary(promote=true)`, or `step_down(...)`.

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
pgtm -c /etc/pgtuskmaster/config.toml status
pgtm -c /etc/pgtuskmaster/config.toml --output text status
pgtm -c /etc/pgtuskmaster/config.toml switchover request
pgtm -c /etc/pgtuskmaster/config.toml switchover request --switchover-to node-b
pgtm -c /etc/pgtuskmaster/config.toml switchover clear

# Explicit override for troubleshooting
pgtm -c /etc/pgtuskmaster/config.toml --base-url http://127.0.0.1:18081 status
```
