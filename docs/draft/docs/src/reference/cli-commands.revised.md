```
# CLI Commands Reference

This page documents the command-line interface for cluster administration.

## Binary

**pgtuskmasterctl**  
Entry point: `src/bin/pgtuskmasterctl.rs`

## Global Flags

| Flag | Type | Default | Environment | Description |
|------|------|---------|-------------|-------------|
| `--base-url` | string | `http://127.0.0.1:8080` | *none* | API server base URL |
| `--read-token` | string | *optional* | `PGTUSKMASTER_READ_TOKEN` | Bearer token for read operations |
| `--admin-token` | string | *optional* | `PGTUSKMASTER_ADMIN_TOKEN` | Bearer token for administrative operations |
| `--timeout-ms` | u64 | `5000` | *none* | HTTP request timeout in milliseconds |
| `--output` | enum | `json` | *none* | Output format: `json` or `text` |

## Command Hierarchy

```
pgtuskmasterctl
└── ha
    ├── state
    └── switchover
        ├── clear
        └── request --requested-by <STRING>
```

## Authentication Rules

Read operations attempt `--read-token` first, then fall back to `--admin-token` if read token is absent. Administrative operations require `--admin-token`. Empty or whitespace-only token values are treated as absent.

## Commands

### `pgtuskmasterctl ha state`

Retrieves the current HA state snapshot from `/ha/state`.

**Required token**: read or admin  
**Method**: GET `/ha/state`

#### JSON Output Schema

```json
{
  "cluster_name": "string",
  "scope": "string",
  "self_member_id": "string",
  "leader": "string|null",
  "switchover_requested_by": "string|null",
  "member_count": "integer",
  "dcs_trust": "full_quorum|fail_safe|not_trusted",
  "ha_phase": "init|waiting_postgres_reachable|waiting_dcs_trusted|waiting_switchover_successor|replica|candidate_leader|primary|rewinding|bootstrapping|fencing|fail_safe",
  "ha_tick": "integer",
  "ha_decision": "{kind: string, ...}",
  "snapshot_sequence": "integer"
}
```

#### Text Output Format

```
cluster_name=<value>
scope=<value>
self_member_id=<value>
leader=<value> or <none>
switchover_requested_by=<value> or <none>
member_count=<value>
dcs_trust=<value>
ha_phase=<value>
ha_tick=<value>
ha_decision=<value>
snapshot_sequence=<value>
```

### `pgtuskmasterctl ha switchover clear`

Clears any pending switchover request from `/ha/switchover`.

**Required token**: admin  
**Method**: DELETE `/ha/switchover`

Response: `{"accepted": true|false}` (JSON) or `accepted=<bool>` (text)

### `pgtuskmasterctl ha switchover request --requested-by <STRING>`

Submits a switchover request.

**Required token**: admin  
**Method**: POST `/switchover` with body `{"requested_by": "<value>"}`

Response: `{"accepted": true|false}` (JSON) or `accepted=<bool>` (text)

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 2 | Invalid CLI usage (clap error) |
| 3 | Transport or network failure |
| 4 | Unexpected API status code |
| 5 | Response decode failure |

## HA Decision Variants

The `ha_decision` field in state responses is a tagged union. The `kind` field determines the variant:

- `no_change`
- `wait_for_postgres`: `start_requested` (bool), `leader_member_id` (string|null)
- `wait_for_dcs_trust`
- `attempt_leadership`
- `follow_leader`: `leader_member_id` (string)
- `become_primary`: `promote` (bool)
- `step_down`: `reason` (enum), `release_leader_lease` (bool), `clear_switchover` (bool), `fence` (bool)
- `recover_replica`: `strategy` (enum with `kind` and fields)
- `fence_node`
- `release_leader_lease`: `reason` (enum)
- `enter_fail_safe`: `release_leader_lease` (bool)

All enum string values use `snake_case`.
```
