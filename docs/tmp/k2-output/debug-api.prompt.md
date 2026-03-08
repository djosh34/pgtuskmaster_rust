# Debug API Reference

Publishes runtime snapshot data through a snapshot worker, three read-only HTTP endpoints, and an embedded HTML view.

## Module Surface

| Path | Contents |
|---|---|
| `src/debug_api/snapshot.rs` | Snapshot model and build logic |
| `src/debug_api/view.rs` | Verbose payload construction and view types |
| `src/debug_api/worker.rs` | Snapshot worker and observation loop |

## Snapshot Model

### `AppLifecycle`

Discriminant values:

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

`build_snapshot` constructs a `SystemSnapshot` by cloning `DebugSnapshotCtx` fields and copying `changes` and `timeline` slices.

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

## Verbose Payload Sections

`build_verbose_payload` filters `changes` and `timeline` to entries with `sequence > since_sequence.unwrap_or(0)`, then constructs a `DebugVerbosePayload` with the sections below.

### `DebugVerbosePayload`

| Section | Type |
|---|---|
| `meta` | `DebugMeta` |
| `config` | `ConfigSection` |
| `pginfo` | `PgInfoSection` |
| `dcs` | `DcsSection` |
| `process` | `ProcessSection` |
| `ha` | `HaSection` |
| `api` | `ApiSection` |
| `debug` | `DebugSection` |
| `changes` | `Vec<DebugChangeView>` |
| `timeline` | `Vec<DebugTimelineView>` |

### `meta`

| Field | Source |
|---|---|
| `schema_version` | `v1` |
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
| `timeline` | `Option<u64>` |
| `summary` | `String` |

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

`ApiSection` contains `endpoints`: `Vec<&'static str>` with the values:

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

## Published Endpoints

| Method | Path | Success response | Other responses |
|---|---|---|---|
| `GET` | `/debug/snapshot` | `200 OK` text from `format!("{:#?}", snapshot)` | `404 Not Found` when `cfg.debug.enabled` is false; `503 Service Unavailable` when no debug snapshot subscriber is configured |
| `GET` | `/debug/verbose` | `200 OK` JSON from `build_verbose_payload` | `404 Not Found` when `cfg.debug.enabled` is false; `503 Service Unavailable` when no debug snapshot subscriber is configured; `400 Bad Request` when `since` is invalid |
| `GET` | `/debug/ui` | `200 OK` HTML | `404 Not Found` when `cfg.debug.enabled` is false |

`parse_since_sequence` returns `None` when `since` is absent. Invalid values return `400 Bad Request` with body `invalid since query parameter: <parse error>`.

## Worker Loop and History Behavior

### `DebugApiCtx`

| Field | Type |
|---|---|
| `app` | `AppLifecycle` |
| `publisher` | `StatePublisher<SystemSnapshot>` |
| `config_subscriber` | `StateSubscriber<RuntimeConfig>` |
| `pg_subscriber` | `StateSubscriber<PgInfoState>` |
| `dcs_subscriber` | `StateSubscriber<DcsState>` |
| `process_subscriber` | `StateSubscriber<ProcessState>` |
| `ha_subscriber` | `StateSubscriber<HaState>` |
| `poll_interval` | `Duration` |
| `now` | `Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>` |
| `history_limit` | `usize` |
| `sequence` | `u64` |
| `last_observed` | `Option<DebugObservedState>` |
| `changes` | `VecDeque<DebugChangeEvent>` |
| `timeline` | `VecDeque<DebugTimelineEntry>` |

`DebugApiCtx::contract_stub` initializes:

| Field | Value |
|---|---|
| `app` | `Starting` |
| `poll_interval` | 10 ms |
| `now` | `UnixMillis(0)` |
| `history_limit` | `DEFAULT_HISTORY_LIMIT` (300) |
| `sequence` | 0 |
| `last_observed` | `None` |
| `changes` | empty `VecDeque` |
| `timeline` | empty `VecDeque` |

### Execution Loop

`debug_api::worker::run` invokes `step_once` then sleeps for `poll_interval` in a perpetual loop.

### `step_once`

`step_once` performs one observation cycle:

- Reads latest versioned values from all subscribers into a `DebugSnapshotCtx`
- Computes per-domain summary strings and an HA signature
- Compares observations with `last_observed`
- For each domain where a signature differs, calls `record_change` with previous and current versions (App uses `None` for both versions)
- On the first observation (`last_observed` is `None`), records six change and timeline entries for all domains
- Builds a `SystemSnapshot` with current `ctx.sequence`, `changes`, and `timeline`
- Publishes the snapshot

Sequence counter behavior: increments with `checked_add(1)`, starting at `0`. Overflow returns `WorkerError::Message("debug_api sequence overflow")`. First recorded entry uses sequence `1`.

### HA Summary and Signature

`summarize_ha` includes worker status, phase, tick, decision label, decision detail, and planned-action count.

`ha_signature` excludes tick and planned-action count. HA tick-only changes do not create new history entries.

### History Retention

`DEFAULT_HISTORY_LIMIT` is 300.

`trim_history` removes entries from the front of `changes` and `timeline` until both lengths are `<= history_limit`.
