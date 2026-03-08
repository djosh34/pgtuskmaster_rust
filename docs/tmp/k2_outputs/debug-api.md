# Debug API Reference

The debug API publishes runtime snapshot data through a snapshot worker, three read-only HTTP endpoints, and an embedded HTML view.

## Module surface

- `src/debug_api/snapshot.rs` – Snapshot model and build logic
- `src/debug_api/view.rs` – Verbose payload construction and view types
- `src/debug_api/worker.rs` – Snapshot worker and observation loop

## Snapshot model

### `AppLifecycle`

| Variant |
|---|
| `Starting` |
| `Running` |

### `SystemSnapshot`

| Field | Type |
|---|---|
| `app` | `AppLifecycle` |
| `config` | `Versioned<RuntimeConfig>` |
| `pg` | `Versioned<PgInfoState>` |
| `dcs` | `Versioned<DcsState>` |
| `process` | `Versioned<ProcessState>` |
| `ha` | `Versioned<HaState>` |
| `generated_at` | `UnixMillis` |
| `sequence` | `u64` |
| `changes` | `Vec<DebugChangeEvent>` |
| `timeline` | `Vec<DebugTimelineEntry>` |

`build_snapshot` copies `DebugSnapshotCtx` plus supplied `changes` and `timeline` slices into a published `SystemSnapshot`.

### `DebugDomain`

| Variant | Verbose label |
|---|---|
| `App` | `app` |
| `Config` | `config` |
| `PgInfo` | `pginfo` |
| `Dcs` | `dcs` |
| `Process` | `process` |
| `Ha` | `ha` |

### `DebugChangeEvent`

| Field | Type |
|---|---|
| `sequence` | `u64` |
| `at` | `UnixMillis` |
| `domain` | `DebugDomain` |
| `previous_version` | `Option<Version>` |
| `current_version` | `Option<Version>` |
| `summary` | `String` |

### `DebugTimelineEntry`

| Field | Type |
|---|---|
| `sequence` | `u64` |
| `at` | `UnixMillis` |
| `domain` | `DebugDomain` |
| `message` | `String` |

## Verbose payload sections

`DebugVerbosePayload` contains the following sections: `meta`, `config`, `pginfo`, `dcs`, `process`, `ha`, `api`, `debug`, `changes`, `timeline`.

### `meta`

| Field | Source |
|---|---|
| `schema_version` | `"v1"` |
| `generated_at_ms` | `snapshot.value.generated_at.0` |
| `channel_updated_at_ms` | `snapshot.updated_at.0` |
| `channel_version` | `snapshot.version.0` |
| `app_lifecycle` | `format!("{:?}", snapshot.value.app)` |
| `sequence` | `snapshot.value.sequence` |

### `config`

| Field | Type or value |
|---|---|
| `version` | `u64` |
| `updated_at_ms` | `u64` |
| `cluster_name` | `String` |
| `member_id` | `String` |
| `scope` | `String` |
| `debug_enabled` | `bool` |
| `tls_enabled` | `true` when `cfg.api.security.tls.mode != ApiTlsMode::Disabled` |

### `pginfo`

| Field | Values |
|---|---|
| `variant` | `Unknown`, `Primary`, `Replica` |
| `worker` | `Starting`, `Running`, `Stopping`, `Stopped`, `Faulted(<error>)` |
| `sql` | `Unknown`, `Healthy`, `Unreachable` |
| `readiness` | `Unknown`, `Ready`, `NotReady` |

Additional fields: `version`, `updated_at_ms`, `timeline`, `summary`.

### `dcs`

| Field | Type or values |
|---|---|
| `version` | `u64` |
| `updated_at_ms` | `u64` |
| `worker` | `String` |
| `trust` | `FullQuorum`, `FailSafe`, `NotTrusted` |
| `member_count` | `usize` |
| `leader` | `Option<String>` |
| `has_switchover_request` | `bool` |

### `process`

| Field | Type or values |
|---|---|
| `version` | `u64` |
| `updated_at_ms` | `u64` |
| `worker` | `String` |
| `state` | `Idle`, `Running` |
| `running_job_id` | `Option<String>` |
| `last_outcome` | `Option<String>` |

### `ha`

| Field | Note |
|---|---|
| `version` | `u64` |
| `updated_at_ms` | `u64` |
| `worker` | `String` |
| `phase` | Rust debug formatting of `HaPhase` |
| `tick` | `u64` |
| `decision` | `HaDecision::label()` |
| `decision_detail` | `HaDecision::detail()` |
| `planned_actions` | `lower_decision(decision).len()` |

### `api`

Contains `endpoints`:

- `/debug/snapshot`
- `/debug/verbose`
- `/debug/ui`
- `/fallback/cluster`
- `/switchover`
- `/ha/state`
- `/ha/switchover`

### `debug`

| Field | Type |
|---|---|
| `history_changes` | `usize` |
| `history_timeline` | `usize` |
| `last_sequence` | `u64` |

### `changes`

| Field | Type |
|---|---|
| `sequence` | `u64` |
| `at_ms` | `u64` |
| `domain` | Lower-case domain label |
| `previous_version` | `Option<u64>` |
| `current_version` | `Option<u64>` |
| `summary` | `String` |

### `timeline`

| Field | Type |
|---|---|
| `sequence` | `u64` |
| `at_ms` | `u64` |
| `category` | Lower-case domain label |
| `message` | `String` |

Domain labels for `changes.domain` and `timeline.category`: `app`, `config`, `pginfo`, `dcs`, `process`, `ha`.

`build_verbose_payload` filters `changes` and `timeline` to entries whose sequence exceeds the optional `since_sequence` parameter.

## Published endpoints

| Method | Path | Success response | Other responses |
|---|---|---|---|
| `GET` | `/debug/snapshot` | `200 OK` text from `format!("{:#?}", snapshot)` | `404 Not Found` when `cfg.debug.enabled` is false; `503 Service Unavailable` when no debug snapshot subscriber is configured |
| `GET` | `/debug/verbose` | `200 OK` JSON from `build_verbose_payload` | `404 Not Found` when `cfg.debug.enabled` is false; `503 Service Unavailable` when no debug snapshot subscriber is configured; `400 Bad Request` when `since` is invalid |
| `GET` | `/debug/ui` | `200 OK` HTML from `debug_ui_html()` | `404 Not Found` when `cfg.debug.enabled` is false |

Query parameter handling: `parse_since_sequence` reads `since`, ignores unrelated pairs, returns `None` when absent; invalid values produce `400 Bad Request`.

## Worker loop and history behavior

### `DebugApiCtx`

| Field |
|---|
| `app` |
| `publisher` |
| `config_subscriber` |
| `pg_subscriber` |
| `dcs_subscriber` |
| `process_subscriber` |
| `ha_subscriber` |
| `poll_interval` |
| `now` |
| `history_limit` |
| `sequence` |
| `last_observed` |
| `changes` |
| `timeline` |

### Initialization

`DebugApiCtx::contract_stub` initializes:

| Field | Value |
|---|---|
| `app` | `Starting` |
| `poll_interval` | `10 ms` |
| `now` | `UnixMillis(0)` |
| `history_limit` | `300` |
| `sequence` | `0` |
| `last_observed` | `None` |

Runtime wiring in `src/runtime/node.rs` sets:

| Field | Runtime value |
|---|---|
| `app` | `Running` |
| `poll_interval` | `Duration::from_millis(cfg.ha.loop_interval_ms)` |
| `now` | `system_now_unix_millis` |

### Execution loop

`debug_api::worker::run` loops forever: `step_once`, then `sleep(poll_interval)`.

`step_once`:

1. Reads latest versioned `config`, `pg`, `dcs`, `process`, and `ha` values into `DebugSnapshotCtx`.
2. Computes summary strings for each domain plus a separate HA signature.
3. Compares the new observation with `last_observed`.
4. Records `DebugChangeEvent` and `DebugTimelineEntry` values only for changed domains.
5. Builds `SystemSnapshot`.
6. Publishes the snapshot with `ctx.publisher.publish(snapshot, now)`.

On the first observation, `step_once` records six entries for `App`, `Config`, `PgInfo`, `Dcs`, `Process`, and `Ha`.

The sequence counter starts at `0` and increments once per recorded change with `checked_add(1)`; the first recorded entry uses sequence `1`. Overflow returns `WorkerError::Message("debug_api sequence overflow")`.

App changes always use `previous_version = None` and `current_version = None`. Config, PostgreSQL, DCS, process, and HA changes include both previous and current versions.

If signatures do not change between steps, `sequence`, `changes`, and `timeline` remain unchanged.

### HA summary and signature

`summarize_ha` includes worker status, phase, tick, decision label, decision detail, and planned-action count.

`ha_signature` excludes tick and planned-action count; it includes worker status, phase, decision label, and decision detail. HA tick-only changes therefore do not create new history entries.

### History retention

`DEFAULT_HISTORY_LIMIT` is `300`.

`trim_history` removes entries from the front of `changes` and `timeline` until both lengths are less than or equal to `history_limit`.

## Embedded UI

`GET /debug/ui` returns HTML titled `PGTuskMaster Debug UI`.

Panels:

- `Runtime Meta`
- `Config`
- `PgInfo`
- `DCS`
- `Process`
- `HA`
- `Timeline`
- `Changes`

The embedded script:

- Initializes `state.since` to `0`.
- Fetches `/debug/verbose?since=${state.since}` with `cache: no-store`.
- Sets the badge to `http-<status>` on non-success HTTP responses.
- Sets the badge to `offline` on fetch exceptions.
- Updates `state.since` to the maximum observed `payload.meta.sequence`.
- Renders the config, pginfo, dcs, process, ha, timeline, and changes panels.
- Runs once immediately.
- Polls every `900 ms`.
