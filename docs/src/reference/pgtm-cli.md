# pgtm

## Name

`pgtm` - operator CLI for the pgtuskmaster node API

## Synopsis

```text
pgtm [OPTIONS]
pgtm [OPTIONS] status [--json] [-v|--verbose] [--watch]
pgtm [OPTIONS] primary [--json] [--tls]
pgtm [OPTIONS] replicas [--json] [--tls]
pgtm [OPTIONS] switchover request [--switchover-to <member_id>]
pgtm [OPTIONS] switchover clear
```

## Description

`pgtm` is the operator-facing client for the node API. It resolves API URL, tokens, and API-client TLS settings from the shared runtime config when you pass `-c`.

The CLI now uses one seed node and one read endpoint:

- read path: `GET /state`
- control paths: `POST /switchover` and `DELETE /switchover`

It does not call peer APIs for cluster discovery or debug enrichment.

## Global Options

| Option | Type | Default | Notes |
| --- | --- | --- | --- |
| `-c`, `--config` | path | unset | Shared runtime config path |
| `--base-url` | string | unset | Explicit API URL override |
| `--read-token` | string | unset | Explicit read token override |
| `--admin-token` | string | unset | Explicit admin token override |
| `--timeout-ms` | u64 | `5000` | HTTP timeout in milliseconds |
| `--json` | flag | `false` | Emit machine-readable JSON |
| `-v`, `--verbose` | flag | `false` | Add more per-node detail to `status` |
| `--watch` | flag | `false` | Repeat `status` until interrupted |

Resolution precedence:

1. explicit CLI overrides
2. `pgtm.api.base_url`
3. values derived from `api.listen_addr`
4. config-backed auth and TLS material

## Command Hierarchy

```text
pgtm (= status)
├── primary
├── replicas
└── switchover
    ├── clear
    └── request
```

## Commands

### `status`

Fetches one seed `NodeState` from `GET /state` and renders a cluster-oriented view from the DCS member slots inside that payload.

- HTTP method: `GET`
- Path: `/state`
- Auth role: read, with fallback to admin

Human output is a compact table. `-v` adds more detail from the same payload. `--watch` repeats the same seed read and redraws the result every two seconds.

### `status --json`

Emits the synthesized status view as JSON.

Top-level fields:

- `cluster_name`
- `scope`
- `verbose`
- `queried_via`
- `discovered_member_count`
- `health`
- `warnings`
- `switchover`
- `nodes`

Each `nodes[]` entry records:

- member identity
- whether it is `self`
- published API URL, when present
- role and readiness derived from DCS member slots
- primary leader projection from the seed node's HA publication
- local-only phase, decision, and process fields for the seed node

Warnings are generated from the seed node's trust and authority view. A degraded or leaderless seed response makes the rendered cluster health `degraded`.

### `primary`

Reads one seed `GET /state` payload and resolves the authoritative primary from the seed node's HA publication plus the DCS member slot routing data.

Default output: one libpq keyword/value DSN line

The command fails closed when the seed payload does not currently project a primary or when the routing data is incomplete.

### `replicas`

Reads one seed `GET /state` payload and prints DSNs for the replicas that the seed node currently exposes as ready DCS member slots.

### `switchover request`

Sends:

- method: `POST`
- path: `/switchover`
- auth role: admin

Optional request field:

- `switchover_to`

When omitted, the node API chooses the best eligible target from the current DCS member slots.

### `switchover clear`

Sends:

- method: `DELETE`
- path: `/switchover`
- auth role: admin

## Exit Codes

- `0`: success
- `2`: CLI usage error
- `3`: transport or request-build error
- `4`: API status or state-resolution error
- `5`: decode or output error
- `6`: config resolution error
