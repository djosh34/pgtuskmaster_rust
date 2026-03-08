# Debug API Reference

The debug API publishes runtime snapshot data through a snapshot worker, three read-only HTTP endpoints, and an embedded HTML view.

## Snapshot Model

### `AppLifecycle`

Values:

- `Starting`
- `Running`

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

`build_snapshot` copies the current `DebugSnapshotCtx` plus supplied `changes` and `timeline` slices into the published `SystemSnapshot`.

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

## Worker Behavior

### `DebugApiCtx`

`DebugApiCtx` fields:

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

### Execution Loop

`debug_api::worker::run` loops forever: `step_once`, then `sleep(poll_interval)`.

### `step_once`

`step_once`:

1. reads the latest versioned `config`, `pg`, `dcs`, `process`, and `ha` values into `DebugSnapshotCtx`
2. computes summary strings for each domain plus a separate HA signature
3. compares the new observation with `last_observed`
4. records `DebugChangeEvent` and `DebugTimelineEntry` values only for changed domains
5. builds `SystemSnapshot`
6. publishes the snapshot with `ctx.publisher.publish(snapshot, now)`

On the first observation, `step_once` records six entries for `App`, `Config`, `PgInfo`, `Dcs`, `Process`, and `Ha`.

The sequence counter starts at `0` and increments once per recorded change with `checked_add(1)`, so the first recorded entry uses sequence `1`.

Overflow returns `WorkerError::Message("debug_api sequence overflow")`.

App changes always use `previous_version = None` and `current_version = None`.

Config, PostgreSQL, DCS, process, and HA changes include both previous and current versions.

If signatures do not change between steps, `sequence`, `changes`, and `timeline` remain unchanged.

### HA Summary And Signature

`summarize_ha` includes:

- worker status
- phase
- tick
- decision label
- decision detail
- lowered planned-action count

`ha_signature` excludes tick and planned-action count. It includes:

- worker status
- phase
- decision label
- decision detail

HA tick-only changes therefore do not create new history entries.

### History Retention

`DEFAULT_HISTORY_LIMIT` is `300`.

`trim_history` removes entries from the front of `changes` and `timeline` until both lengths are less than or equal to `history_limit`.

## HTTP Endpoints

| Method | Path | Success response | Other responses |
|---|---|---|---|
| `GET` | `/debug/snapshot` | `200 OK` text from `format!("{:#?}", snapshot)` | `404 Not Found` with body `not found` when `cfg.debug.enabled` is false; `503 Service Unavailable` with body `snapshot unavailable` when no debug snapshot subscriber is configured |
| `GET` | `/debug/verbose` | `200 OK` JSON from `build_verbose_payload` | `404 Not Found` with body `not found` when `cfg.debug.enabled` is false; `503 Service Unavailable` with body `snapshot unavailable` when no debug snapshot subscriber is configured; `400 Bad Request` when `since` is invalid |
| `GET` | `/debug/ui` | `200 OK` HTML from `debug_ui_html()` | `404 Not Found` with body `not found` when `cfg.debug.enabled` is false |

`parse_since_sequence` looks for query parameter `since`, ignores unrelated query pairs, and returns `None` when `since` is absent.

Invalid values return `400 Bad Request` with body `invalid since query parameter: <parse error>`.

## Verbose Payload

`build_verbose_payload` accepts `Versioned<SystemSnapshot>` plus optional `since_sequence`.

It filters `changes` and `timeline` to entries whose sequence is greater than `since_sequence.unwrap_or(0)`.

### `DebugVerbosePayload`

Sections:

- `meta`
- `config`
- `pginfo`
- `dcs`
- `process`
- `ha`
- `api`
- `debug`
- `changes`
- `timeline`

### `meta`

`DebugMeta` fields:

| Field | Source |
|---|---|
| `schema_version` | `v1` |
| `generated_at_ms` | `snapshot.value.generated_at.0` |
| `channel_updated_at_ms` | `snapshot.updated_at.0` |
| `channel_version` | `snapshot.version.0` |
| `app_lifecycle` | `format!("{:?}", snapshot.value.app)` |
| `sequence` | `snapshot.value.sequence` |

### `config`

`ConfigSection` fields:

| Field | Type or value |
|---|---|
| `version` | `u64` |
| `updated_at_ms` | `u64` |
| `cluster_name` | `String` |
| `member_id` | `String` |
| `scope` | `String` |
| `debug_enabled` | `bool` |
| `tls_enabled` | true when `cfg.api.security.tls.mode != ApiTlsMode::Disabled` |

### `pginfo`

`PgInfoSection` fields:

| Field | Values |
|---|---|
| `variant` | `Unknown`, `Primary`, `Replica` |
| `worker` | `Starting`, `Running`, `Stopping`, `Stopped`, `Faulted(<error>)` |
| `sql` | `Unknown`, `Healthy`, `Unreachable` |
| `readiness` | `Unknown`, `Ready`, `NotReady` |

Additional fields present in the section:

- `version`
- `updated_at_ms`
- `timeline`
- `summary`

### `dcs`

`DcsSection` fields:

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

`ProcessSection` fields:

| Field | Type or values |
|---|---|
| `version` | `u64` |
| `updated_at_ms` | `u64` |
| `worker` | `String` |
| `state` | `Idle`, `Running` |
| `running_job_id` | `Option<String>` |
| `last_outcome` | `Option<String>` |

### `ha`

`HaSection` fields:

| Field | Note |
|---|---|
| `version` | `u64` |
| `updated_at_ms` | `u64` |
| `worker` | `String` |
| `phase` | Rust debug formatting of `HaPhase`, for example `WaitingDcsTrusted` |
| `tick` | `u64` |
| `decision` | `HaDecision::label()` |
| `decision_detail` | `HaDecision::detail()` |
| `planned_actions` | `lower_decision(decision).len()` |

### `api`

`ApiSection` contains `endpoints`.

Endpoints listed by `build_verbose_payload`:

- `/debug/snapshot`
- `/debug/verbose`
- `/debug/ui`
- `/fallback/cluster`
- `/switchover`
- `/ha/state`
- `/ha/switchover`

### `debug`

`DebugSection` fields:

| Field | Type |
|---|---|
| `history_changes` | `usize` |
| `history_timeline` | `usize` |
| `last_sequence` | `u64` |

### `changes`

`DebugChangeView` fields:

| Field | Type |
|---|---|
| `sequence` | `u64` |
| `at_ms` | `u64` |
| `domain` | lower-case domain label |
| `previous_version` | `Option<u64>` |
| `current_version` | `Option<u64>` |
| `summary` | `String` |

### `timeline`

`DebugTimelineView` fields:

| Field | Type |
|---|---|
| `sequence` | `u64` |
| `at_ms` | `u64` |
| `category` | lower-case domain label |
| `message` | `String` |

Domain labels for `changes.domain` and `timeline.category`:

- `app`
- `config`
- `pginfo`
- `dcs`
- `process`
- `ha`

## Embedded UI

`GET /debug/ui` returns HTML titled `PGTuskMaster Debug UI`.

The page contains panels for:

- `Runtime Meta`
- `Config`
- `PgInfo`
- `DCS`
- `Process`
- `HA`
- `Timeline`
- `Changes`

The embedded script:

- initializes `state.since` to `0`
- fetches `/debug/verbose?since=${state.since}` with `cache: no-store`
- sets the badge to `http-<status>` on non-success HTTP responses
- sets the badge to `offline` on fetch exceptions
- updates `state.since` to the maximum observed `payload.meta.sequence`
- renders the config, pginfo, dcs, process, ha, timeline, and changes panels
- runs once immediately
- polls every `900 ms`

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
