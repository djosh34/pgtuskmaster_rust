# pgtuskmasterctl

## Name

pgtuskmasterctl - HA admin CLI for PGTuskMaster API

## Synopsis

`pgtuskmasterctl [OPTIONS] ha <COMMAND>`

## Description

`pgtuskmasterctl` is the command-line client for the PGTuskMaster HA API. It queries cluster state and submits switchover operations against a running node's HTTP API.

## Global Options

| Option | Type | Default | Environment | Notes |
| --- | --- | --- | --- | --- |
| `--base-url` | string | `http://127.0.0.1:8080` | none | Parsed after trimming whitespace |
| `--read-token` | string | unset | `PGTUSKMASTER_READ_TOKEN` | Used for read operations when present |
| `--admin-token` | string | unset | `PGTUSKMASTER_ADMIN_TOKEN` | Required for admin operations |
| `--timeout-ms` | u64 | `5000` | none | HTTP client timeout in milliseconds |
| `--output` | `json` or `text` | `json` | none | Output renderer |

Whitespace-only token values are treated as absent. Read operations use `--read-token` first and fall back to `--admin-token` when the read token is missing.

## Command Hierarchy

```text
pgtuskmasterctl
└── ha
    ├── state
    └── switchover
        ├── clear
        └── request
```

## Commands

### `ha state`

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

### `ha switchover clear`

Clears the current switchover request.

- HTTP method: `DELETE`
- Path: `/ha/switchover`
- Auth role: admin
- Expected success payload: `{"accepted": true}` or `{"accepted": false}`

In text mode the response is rendered as `accepted=<bool>`.

### `ha switchover request`

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

- `ha state` as fixed `key=value` lines
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

## Examples

```bash
pgtuskmasterctl ha state
pgtuskmasterctl --base-url http://127.0.0.1:18081 --output text ha state
pgtuskmasterctl --admin-token "$ADMIN_TOKEN" ha switchover request
pgtuskmasterctl --admin-token "$ADMIN_TOKEN" ha switchover request --switchover-to node-b
pgtuskmasterctl --admin-token "$ADMIN_TOKEN" ha switchover clear
```
