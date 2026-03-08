# Debug API Reference

The debug API publishes runtime snapshot data through the debug snapshot worker, three read-only HTTP endpoints, and an embedded HTML view. The worker publishes `SystemSnapshot` values that combine versioned config, PostgreSQL, DCS, process, and HA state with retained change and timeline history.

## Snapshot Model

### `AppLifecycle`

Values: `Starting`, `Running`.

### `SystemSnapshot`

| Field | Type | Description |
|---|---|---|
| `app` | `AppLifecycle` | current application lifecycle |
| `config` | `Versioned<RuntimeConfig>` | versioned runtime configuration snapshot |
| `pg` | `Versioned<PgInfoState>` | versioned PostgreSQL state snapshot |
| `dcs` | `Versioned<DcsState>` | versioned DCS state snapshot |
| `process` | `Versioned<ProcessState>` | versioned process state snapshot |
| `ha` | `Versioned<HaState>` | versioned HA state snapshot |
| `generated_at` | `UnixMillis` | snapshot generation timestamp |
| `sequence` | `u64` | latest recorded change sequence |
| `changes` | `Vec<DebugChangeEvent>` | copied change history |
| `timeline` | `Vec<DebugTimelineEntry>` | copied timeline history |

`build_snapshot` copies the current `DebugSnapshotCtx` plus supplied `changes` and `timeline` slices into the published `SystemSnapshot`.

### `DebugDomain`

Enum variants: `App`, `Config`, `PgInfo`, `Dcs`, `Process`, `Ha`.

Verbose payload labels for these domains are lower-case `app`, `config`, `pginfo`, `dcs`, `process`, and `ha`.

### Change And Timeline Entries

`DebugChangeEvent` fields: `sequence`, `at`, `domain`, `previous_version`, `current_version`, `summary`.

`DebugTimelineEntry` fields: `sequence`, `at`, `domain`, `message`.

## Worker Behavior

`debug_api::worker::run` loops forever: `step_once`, then `sleep(poll_interval)`.

### `DebugApiCtx`

Fields: `app`, `publisher`, `config_subscriber`, `pg_subscriber`, `dcs_subscriber`, `process_subscriber`, `ha_subscriber`, `poll_interval`, `now`, `history_limit`, `sequence`, `last_observed`, `changes`, `timeline`.

`DebugApiCtx::contract_stub` initializes:

| Field | Value |
|---|---|
| `app` | `Starting` |
| `poll_interval` | `10 ms` |
| `now` | `UnixMillis(0)` |
| `history_limit` | `300` |
| `sequence` | `0` |
| `last_observed` | `None` |

The contract stub also starts with empty `changes` and `timeline` deques.

Runtime wiring in `src/runtime/node.rs` sets:

| Field | Runtime value |
|---|---|
| `app` | `Running` |
| `poll_interval` | `Duration::from_millis(cfg.ha.loop_interval_ms)` |
| `now` | `system_now_unix_millis` |

### `step_once`

`step_once`:

1. reads the latest versioned `config`, `pg`, `dcs`, `process`, and `ha` values into `DebugSnapshotCtx`
2. computes summary strings for each domain plus a separate HA signature
3. compares the new observation with `last_observed`
4. records `DebugChangeEvent` and `DebugTimelineEntry` values only for changed domains
5. builds `SystemSnapshot`
6. publishes the snapshot through `ctx.publisher.publish(snapshot, now)`

On the first observation, `step_once` records six entries for `App`, `Config`, `PgInfo`, `Dcs`, `Process`, and `Ha`. The sequence counter starts at `0` and increments once per recorded change with `checked_add(1)`, so the first recorded entry uses sequence `1`. Overflow returns `WorkerError::Message("debug_api sequence overflow")`.

App changes always use `previous_version = None` and `current_version = None`. Config, PostgreSQL, DCS, process, and HA changes include both previous and current versions.

If signatures do not change between steps, `sequence`, `changes`, and `timeline` remain unchanged.

### HA Signature

`summarize_ha` includes worker status, phase, tick, decision label, decision detail, and lowered planned-action count.

`ha_signature` excludes `tick` and planned-action count. It includes worker status, phase, decision label, and decision detail. HA tick-only changes therefore do not create new history entries.

### History Retention

`DEFAULT_HISTORY_LIMIT` is `300`.

`trim_history` removes entries from the front of `changes` and `timeline` until both lengths are less than or equal to `history_limit`.

## HTTP Endpoints

| Method | Path | Success response | Other responses |
|---|---|---|---|
| `GET` | `/debug/snapshot` | `200 OK` text from `format!("{:#?}", snapshot)` | `404 not found` when `cfg.debug.enabled` is false; `503 snapshot unavailable` when no debug snapshot subscriber is configured |
| `GET` | `/debug/verbose` | `200 OK` JSON from `build_verbose_payload` | `404 not found` when `cfg.debug.enabled` is false; `503 snapshot unavailable` when no debug snapshot subscriber is configured; `400 Bad Request` when `since` is invalid |
| `GET` | `/debug/ui` | `200 OK` HTML from `debug_ui_html()` | `404 not found` when `cfg.debug.enabled` is false |

`parse_since_sequence` looks for query parameter `since`, ignores unrelated query pairs, and returns `None` when `since` is absent. Invalid values return `400 Bad Request` with body `invalid since query parameter: <parse error>`.

## Verbose Payload

`build_verbose_payload` accepts `Versioned<SystemSnapshot>` plus optional `since_sequence`. It filters `changes` and `timeline` to entries whose sequence is greater than `since_sequence.unwrap_or(0)`.

`DebugVerbosePayload` sections: `meta`, `config`, `pginfo`, `dcs`, `process`, `ha`, `api`, `debug`, `changes`, `timeline`.

### `meta`

`DebugMeta` fields: `schema_version`, `generated_at_ms`, `channel_updated_at_ms`, `channel_version`, `app_lifecycle`, `sequence`.

`schema_version` is `v1`.

`generated_at_ms` comes from `snapshot.value.generated_at.0`, `channel_updated_at_ms` from `snapshot.updated_at.0`, and `channel_version` from `snapshot.version.0`.

### `config`

`ConfigSection` fields: `version`, `updated_at_ms`, `cluster_name`, `member_id`, `scope`, `debug_enabled`, `tls_enabled`.

`tls_enabled` is true when `cfg.api.security.tls.mode != ApiTlsMode::Disabled`.

### `pginfo`

`PgInfoSection` fields: `version`, `updated_at_ms`, `variant`, `worker`, `sql`, `readiness`, `timeline`, `summary`.

`variant` values: `Unknown`, `Primary`, `Replica`.

Worker labels are `Starting`, `Running`, `Stopping`, `Stopped`, or `Faulted(<error>)`.

SQL labels are `Unknown`, `Healthy`, `Unreachable`.

Readiness labels are `Unknown`, `Ready`, `NotReady`.

### `dcs`

`DcsSection` fields: `version`, `updated_at_ms`, `worker`, `trust`, `member_count`, `leader`, `has_switchover_request`.

Trust labels are `FullQuorum`, `FailSafe`, `NotTrusted`.

### `process`

`ProcessSection` fields: `version`, `updated_at_ms`, `worker`, `state`, `running_job_id`, `last_outcome`.

`state` values: `Idle`, `Running`.

### `ha`

`HaSection` fields: `version`, `updated_at_ms`, `worker`, `phase`, `tick`, `decision`, `decision_detail`, `planned_actions`.

`phase` uses Rust debug formatting of `HaPhase`, for example `WaitingDcsTrusted`.

`decision` is `HaDecision::label()`.

`decision_detail` comes from `HaDecision::detail()`.

`planned_actions` is `lower_decision(decision).len()`.

### `api`

`ApiSection` contains `endpoints`.

Endpoints listed by `build_verbose_payload`:

| Path |
|---|
| `/debug/snapshot` |
| `/debug/verbose` |
| `/debug/ui` |
| `/fallback/cluster` |
| `/switchover` |
| `/ha/state` |
| `/ha/switchover` |

### `debug`

`DebugSection` fields: `history_changes`, `history_timeline`, `last_sequence`.

### `changes`

`DebugChangeView` fields: `sequence`, `at_ms`, `domain`, `previous_version`, `current_version`, `summary`.

`domain` uses lower-case labels from `debug_domain_label`: `app`, `config`, `pginfo`, `dcs`, `process`, `ha`.

### `timeline`

`DebugTimelineView` fields: `sequence`, `at_ms`, `category`, `message`.

`category` uses the same lower-case domain labels as `DebugChangeView.domain`: `app`, `config`, `pginfo`, `dcs`, `process`, `ha`.

## Embedded UI

`GET /debug/ui` returns HTML titled `PGTuskMaster Debug UI`.

The page contains panels for `Runtime Meta`, `Config`, `PgInfo`, `DCS`, `Process`, `HA`, `Timeline`, and `Changes`.

The embedded script:

- initializes `state.since` to `0`
- fetches `/debug/verbose?since=${state.since}` with `cache: no-store`
- sets the badge to `http-<status>` on non-success HTTP responses
- sets the badge to `offline` on fetch exceptions
- updates `state.since` to the maximum observed `payload.meta.sequence`
- renders the config, pginfo, dcs, process, ha, timeline, and changes panels
- runs once immediately and then polls every `900 ms`

## Verified Behaviors

Tests in `src/debug_api/worker.rs` verify:

- the first `step_once` publishes a snapshot with `app = Running`, `sequence = 6`, and six initial `changes` plus six initial `timeline` entries
- a repeated `step_once` with unchanged versioned inputs leaves `sequence`, `changes`, and `timeline` unchanged
- publishing a changed runtime config records at least one additional `DebugDomain::Config` change event
- HA tick-only changes update the HA snapshot without recording new history entries
- reduced `history_limit` values trim retained `changes` and `timeline` entries from the front

Tests in `src/api/worker.rs` verify:

- `GET /debug/verbose` returns structured JSON and honors the `since` filter
- `GET /debug/snapshot` remains available and returns the formatted text snapshot
- `GET /debug/verbose` returns `404` when debug is disabled
- `GET /debug/verbose` returns `503 snapshot unavailable` when no debug snapshot subscriber is configured
- `GET /debug/ui` returns HTML containing `id="meta-panel"`, `/debug/verbose`, and `id="timeline-panel"`
- debug routes require authorization when role tokens are configured
