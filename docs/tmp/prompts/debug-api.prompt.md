Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise an existing reference page so it stays strictly in Diataxis reference form.

[Page path]
- docs/src/reference/debug-api.md

[Page goal]
- Reference the debug snapshot machinery, verbose payload shape, UI endpoints, and debug worker publication loop.

[Audience]
- Operators and contributors who need accurate repo-backed facts while working with pgtuskmaster.

[User need]
- Consult the machinery surface, data model, constraints, constants, and behavior without being taught procedures or background explanations.

[mdBook context]
- This is an mdBook page under docs/src/reference/.
- Keep headings and lists suitable for mdBook.
- Do not add verification notes, scratch notes, or commentary about how the page was produced.

[Diataxis guidance]
- This page must stay in the reference quadrant: cognition plus application.
- Describe and only describe.
- Structure the page to mirror the machinery, not a guessed workflow.
- Use neutral, technical language.
- Examples are allowed only when they illustrate the surface concisely.
- Do not include step-by-step operations, recommendations, rationale, or explanations of why the design exists.
- If action or explanation seems necessary, keep the page neutral and mention the boundary without turning the page into a how-to or explanation article.

[Required structure]
- Overview\n- Module surface\n- Snapshot model\n- Verbose payload sections\n- Published endpoints\n- Worker loop and history behavior

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

# Debug API Reference

The debug API publishes runtime snapshot data through a snapshot worker, three read-only HTTP endpoints, and an embedded HTML view.

## Module Surface

- `src/debug_api/snapshot.rs`: snapshot model and build logic
- `src/debug_api/view.rs`: verbose payload construction and view types
- `src/debug_api/worker.rs`: snapshot worker and observation loop

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

- initializes `state.since` to `0`
- fetches `/debug/verbose?since=${state.since}` with `cache: no-store`
- sets the badge to `http-<status>` on non-success HTTP responses
- sets the badge to `offline` on fetch exceptions
- updates `state.since` to the maximum observed `payload.meta.sequence`
- renders the config, pginfo, dcs, process, ha, timeline, and changes panels
- runs once immediately
- polls every `900 ms`

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

[Repo facts and source excerpts]

--- BEGIN FILE: src/debug_api/mod.rs ---
pub(crate) mod snapshot;
pub(crate) mod view;
pub(crate) mod worker;

--- END FILE: src/debug_api/mod.rs ---

--- BEGIN FILE: src/debug_api/snapshot.rs ---
use crate::{
    config::RuntimeConfig,
    dcs::state::DcsState,
    ha::state::HaState,
    pginfo::state::PgInfoState,
    process::state::ProcessState,
    state::{UnixMillis, Version, Versioned},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AppLifecycle {
    Starting,
    Running,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SystemSnapshot {
    pub(crate) app: AppLifecycle,
    pub(crate) config: Versioned<RuntimeConfig>,
    pub(crate) pg: Versioned<PgInfoState>,
    pub(crate) dcs: Versioned<DcsState>,
    pub(crate) process: Versioned<ProcessState>,
    pub(crate) ha: Versioned<HaState>,
    pub(crate) generated_at: UnixMillis,
    pub(crate) sequence: u64,
    pub(crate) changes: Vec<DebugChangeEvent>,
    pub(crate) timeline: Vec<DebugTimelineEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DebugDomain {
    App,
    Config,
    PgInfo,
    Dcs,
    Process,
    Ha,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DebugChangeEvent {
    pub(crate) sequence: u64,
    pub(crate) at: UnixMillis,
    pub(crate) domain: DebugDomain,
    pub(crate) previous_version: Option<Version>,
    pub(crate) current_version: Option<Version>,
    pub(crate) summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DebugTimelineEntry {
    pub(crate) sequence: u64,
    pub(crate) at: UnixMillis,
    pub(crate) domain: DebugDomain,
    pub(crate) message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DebugSnapshotCtx {
    pub(crate) app: AppLifecycle,
    pub(crate) config: Versioned<RuntimeConfig>,
    pub(crate) pg: Versioned<PgInfoState>,
    pub(crate) dcs: Versioned<DcsState>,
    pub(crate) process: Versioned<ProcessState>,
    pub(crate) ha: Versioned<HaState>,
}

pub(crate) fn build_snapshot(
    ctx: &DebugSnapshotCtx,
    now: UnixMillis,
    sequence: u64,
    changes: &[DebugChangeEvent],
    timeline: &[DebugTimelineEntry],
) -> SystemSnapshot {
    SystemSnapshot {
        app: ctx.app.clone(),
        config: ctx.config.clone(),
        pg: ctx.pg.clone(),
        dcs: ctx.dcs.clone(),
        process: ctx.process.clone(),
        ha: ctx.ha.clone(),
        generated_at: now,
        sequence,
        changes: changes.to_vec(),
        timeline: timeline.to_vec(),
    }
}

--- END FILE: src/debug_api/snapshot.rs ---

--- BEGIN FILE: src/debug_api/view.rs ---
use serde::Serialize;

use crate::{
    config::RuntimeConfig,
    dcs::state::{DcsState, DcsTrust},
    debug_api::snapshot::{DebugChangeEvent, DebugDomain, DebugTimelineEntry, SystemSnapshot},
    ha::{lower::lower_decision, state::HaState},
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    process::state::{JobOutcome, ProcessState},
    state::{Versioned, WorkerStatus},
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugVerbosePayload {
    pub(crate) meta: DebugMeta,
    pub(crate) config: ConfigSection,
    pub(crate) pginfo: PgInfoSection,
    pub(crate) dcs: DcsSection,
    pub(crate) process: ProcessSection,
    pub(crate) ha: HaSection,
    pub(crate) api: ApiSection,
    pub(crate) debug: DebugSection,
    pub(crate) changes: Vec<DebugChangeView>,
    pub(crate) timeline: Vec<DebugTimelineView>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugMeta {
    pub(crate) schema_version: &'static str,
    pub(crate) generated_at_ms: u64,
    pub(crate) channel_updated_at_ms: u64,
    pub(crate) channel_version: u64,
    pub(crate) app_lifecycle: String,
    pub(crate) sequence: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConfigSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) cluster_name: String,
    pub(crate) member_id: String,
    pub(crate) scope: String,
    pub(crate) debug_enabled: bool,
    pub(crate) tls_enabled: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PgInfoSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) variant: &'static str,
    pub(crate) worker: String,
    pub(crate) sql: String,
    pub(crate) readiness: String,
    pub(crate) timeline: Option<u64>,
    pub(crate) summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DcsSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) trust: String,
    pub(crate) member_count: usize,
    pub(crate) leader: Option<String>,
    pub(crate) has_switchover_request: bool,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProcessSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) state: &'static str,
    pub(crate) running_job_id: Option<String>,
    pub(crate) last_outcome: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct HaSection {
    pub(crate) version: u64,
    pub(crate) updated_at_ms: u64,
    pub(crate) worker: String,
    pub(crate) phase: String,
    pub(crate) tick: u64,
    pub(crate) decision: String,
    pub(crate) decision_detail: Option<String>,
    pub(crate) planned_actions: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ApiSection {
    pub(crate) endpoints: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugSection {
    pub(crate) history_changes: usize,
    pub(crate) history_timeline: usize,
    pub(crate) last_sequence: u64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugChangeView {
    pub(crate) sequence: u64,
    pub(crate) at_ms: u64,
    pub(crate) domain: String,
    pub(crate) previous_version: Option<u64>,
    pub(crate) current_version: Option<u64>,
    pub(crate) summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DebugTimelineView {
    pub(crate) sequence: u64,
    pub(crate) at_ms: u64,
    pub(crate) category: String,
    pub(crate) message: String,
}

pub(crate) fn build_verbose_payload(
    snapshot: &Versioned<SystemSnapshot>,
    since_sequence: Option<u64>,
) -> DebugVerbosePayload {
    let cutoff = since_sequence.unwrap_or(0);
    let filtered_changes = snapshot
        .value
        .changes
        .iter()
        .filter(|event| event.sequence > cutoff)
        .map(to_change_view)
        .collect::<Vec<_>>();
    let filtered_timeline = snapshot
        .value
        .timeline
        .iter()
        .filter(|entry| entry.sequence > cutoff)
        .map(to_timeline_view)
        .collect::<Vec<_>>();

    let cfg = &snapshot.value.config;
    let pg = &snapshot.value.pg;
    let dcs = &snapshot.value.dcs;
    let process = &snapshot.value.process;
    let ha = &snapshot.value.ha;

    DebugVerbosePayload {
        meta: DebugMeta {
            schema_version: "v1",
            generated_at_ms: snapshot.value.generated_at.0,
            channel_updated_at_ms: snapshot.updated_at.0,
            channel_version: snapshot.version.0,
            app_lifecycle: format!("{:?}", snapshot.value.app),
            sequence: snapshot.value.sequence,
        },
        config: to_config_section(cfg),
        pginfo: to_pg_section(pg),
        dcs: to_dcs_section(dcs),
        process: to_process_section(process),
        ha: to_ha_section(ha),
        api: ApiSection {
            endpoints: vec![
                "/debug/snapshot",
                "/debug/verbose",
                "/debug/ui",
                "/fallback/cluster",
                "/switchover",
                "/ha/state",
                "/ha/switchover",
            ],
        },
        debug: DebugSection {
            history_changes: snapshot.value.changes.len(),
            history_timeline: snapshot.value.timeline.len(),
            last_sequence: snapshot.value.sequence,
        },
        changes: filtered_changes,
        timeline: filtered_timeline,
    }
}

fn to_config_section(cfg: &Versioned<RuntimeConfig>) -> ConfigSection {
    ConfigSection {
        version: cfg.version.0,
        updated_at_ms: cfg.updated_at.0,
        cluster_name: cfg.value.cluster.name.clone(),
        member_id: cfg.value.cluster.member_id.clone(),
        scope: cfg.value.dcs.scope.clone(),
        debug_enabled: cfg.value.debug.enabled,
        tls_enabled: cfg.value.api.security.tls.mode != crate::config::ApiTlsMode::Disabled,
    }
}

fn to_pg_section(pg: &Versioned<PgInfoState>) -> PgInfoSection {
    match &pg.value {
        PgInfoState::Unknown { common } => PgInfoSection {
            version: pg.version.0,
            updated_at_ms: pg.updated_at.0,
            variant: "Unknown",
            worker: worker_status_label(&common.worker),
            sql: sql_label(&common.sql),
            readiness: readiness_label(&common.readiness),
            timeline: common.timeline.map(|value| u64::from(value.0)),
            summary: format!(
                "unknown worker={} sql={} readiness={}",
                worker_status_label(&common.worker),
                sql_label(&common.sql),
                readiness_label(&common.readiness)
            ),
        },
        PgInfoState::Primary {
            common,
            wal_lsn,
            slots,
        } => PgInfoSection {
            version: pg.version.0,
            updated_at_ms: pg.updated_at.0,
            variant: "Primary",
            worker: worker_status_label(&common.worker),
            sql: sql_label(&common.sql),
            readiness: readiness_label(&common.readiness),
            timeline: common.timeline.map(|value| u64::from(value.0)),
            summary: format!(
                "primary wal_lsn={} slots={} readiness={}",
                wal_lsn.0,
                slots.len(),
                readiness_label(&common.readiness)
            ),
        },
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => PgInfoSection {
            version: pg.version.0,
            updated_at_ms: pg.updated_at.0,
            variant: "Replica",
            worker: worker_status_label(&common.worker),
            sql: sql_label(&common.sql),
            readiness: readiness_label(&common.readiness),
            timeline: common.timeline.map(|value| u64::from(value.0)),
            summary: format!(
                "replica replay_lsn={} follow_lsn={} upstream={}",
                replay_lsn.0,
                follow_lsn
                    .map(|value| value.0)
                    .map_or_else(|| "none".to_string(), |value| value.to_string()),
                upstream
                    .as_ref()
                    .map(|value| value.member_id.0.clone())
                    .unwrap_or_else(|| "none".to_string())
            ),
        },
    }
}

fn to_dcs_section(dcs: &Versioned<DcsState>) -> DcsSection {
    DcsSection {
        version: dcs.version.0,
        updated_at_ms: dcs.updated_at.0,
        worker: worker_status_label(&dcs.value.worker),
        trust: dcs_trust_label(&dcs.value.trust),
        member_count: dcs.value.cache.members.len(),
        leader: dcs
            .value
            .cache
            .leader
            .as_ref()
            .map(|leader| leader.member_id.0.clone()),
        has_switchover_request: dcs.value.cache.switchover.is_some(),
    }
}

fn to_process_section(process: &Versioned<ProcessState>) -> ProcessSection {
    match &process.value {
        ProcessState::Idle {
            worker,
            last_outcome,
        } => ProcessSection {
            version: process.version.0,
            updated_at_ms: process.updated_at.0,
            worker: worker_status_label(worker),
            state: "Idle",
            running_job_id: None,
            last_outcome: last_outcome.as_ref().map(job_outcome_label),
        },
        ProcessState::Running { worker, active } => ProcessSection {
            version: process.version.0,
            updated_at_ms: process.updated_at.0,
            worker: worker_status_label(worker),
            state: "Running",
            running_job_id: Some(active.id.0.clone()),
            last_outcome: None,
        },
    }
}

fn to_ha_section(ha: &Versioned<HaState>) -> HaSection {
    let decision = &ha.value.decision;

    HaSection {
        version: ha.version.0,
        updated_at_ms: ha.updated_at.0,
        worker: worker_status_label(&ha.value.worker),
        phase: format!("{:?}", ha.value.phase),
        tick: ha.value.tick,
        decision: decision.label().to_string(),
        decision_detail: decision.detail(),
        planned_actions: lower_decision(decision).len(),
    }
}

fn to_change_view(event: &DebugChangeEvent) -> DebugChangeView {
    DebugChangeView {
        sequence: event.sequence,
        at_ms: event.at.0,
        domain: debug_domain_label(&event.domain).to_string(),
        previous_version: event.previous_version.map(|value| value.0),
        current_version: event.current_version.map(|value| value.0),
        summary: event.summary.clone(),
    }
}

fn to_timeline_view(entry: &DebugTimelineEntry) -> DebugTimelineView {
    DebugTimelineView {
        sequence: entry.sequence,
        at_ms: entry.at.0,
        category: debug_domain_label(&entry.domain).to_string(),
        message: entry.message.clone(),
    }
}

fn worker_status_label(status: &WorkerStatus) -> String {
    match status {
        WorkerStatus::Starting => "Starting".to_string(),
        WorkerStatus::Running => "Running".to_string(),
        WorkerStatus::Stopping => "Stopping".to_string(),
        WorkerStatus::Stopped => "Stopped".to_string(),
        WorkerStatus::Faulted(error) => format!("Faulted({error})"),
    }
}

fn sql_label(status: &SqlStatus) -> String {
    match status {
        SqlStatus::Unknown => "Unknown".to_string(),
        SqlStatus::Healthy => "Healthy".to_string(),
        SqlStatus::Unreachable => "Unreachable".to_string(),
    }
}

fn readiness_label(readiness: &Readiness) -> String {
    match readiness {
        Readiness::Unknown => "Unknown".to_string(),
        Readiness::Ready => "Ready".to_string(),
        Readiness::NotReady => "NotReady".to_string(),
    }
}

fn dcs_trust_label(trust: &DcsTrust) -> String {
    match trust {
        DcsTrust::FullQuorum => "FullQuorum".to_string(),
        DcsTrust::FailSafe => "FailSafe".to_string(),
        DcsTrust::NotTrusted => "NotTrusted".to_string(),
    }
}

fn debug_domain_label(domain: &DebugDomain) -> &'static str {
    match domain {
        DebugDomain::App => "app",
        DebugDomain::Config => "config",
        DebugDomain::PgInfo => "pginfo",
        DebugDomain::Dcs => "dcs",
        DebugDomain::Process => "process",
        DebugDomain::Ha => "ha",
    }
}

fn job_outcome_label(outcome: &JobOutcome) -> String {
    match outcome {
        JobOutcome::Success { id, .. } => format!("Success({})", id.0),
        JobOutcome::Failure { id, error, .. } => format!("Failure({}: {:?})", id.0, error),
        JobOutcome::Timeout { id, .. } => format!("Timeout({})", id.0),
    }
}

--- END FILE: src/debug_api/view.rs ---

--- BEGIN FILE: src/debug_api/worker.rs ---
use std::{collections::VecDeque, time::Duration};

use crate::{
    config::RuntimeConfig,
    dcs::state::DcsState,
    debug_api::snapshot::{
        build_snapshot, AppLifecycle, DebugChangeEvent, DebugDomain, DebugSnapshotCtx,
        DebugTimelineEntry, SystemSnapshot,
    },
    ha::{lower::lower_decision, state::HaState},
    pginfo::state::PgInfoState,
    process::state::ProcessState,
    state::{StatePublisher, StateSubscriber, UnixMillis, Version, WorkerError},
};

const DEFAULT_HISTORY_LIMIT: usize = 300;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DebugObservedState {
    app: AppLifecycle,
    config_version: Version,
    config_sig: String,
    pg_version: Version,
    pg_sig: String,
    dcs_version: Version,
    dcs_sig: String,
    process_version: Version,
    process_sig: String,
    ha_version: Version,
    ha_sig: String,
}

pub(crate) struct DebugApiCtx {
    pub(crate) app: AppLifecycle,
    pub(crate) publisher: StatePublisher<SystemSnapshot>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) ha_subscriber: StateSubscriber<HaState>,
    pub(crate) poll_interval: Duration,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    pub(crate) history_limit: usize,
    sequence: u64,
    last_observed: Option<DebugObservedState>,
    changes: VecDeque<DebugChangeEvent>,
    timeline: VecDeque<DebugTimelineEntry>,
}

pub(crate) struct DebugApiContractStubInputs {
    pub(crate) publisher: StatePublisher<SystemSnapshot>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsState>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) ha_subscriber: StateSubscriber<HaState>,
}

impl DebugApiCtx {
    pub(crate) fn contract_stub(inputs: DebugApiContractStubInputs) -> Self {
        let DebugApiContractStubInputs {
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        } = inputs;

        Self {
            app: AppLifecycle::Starting,
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
            poll_interval: Duration::from_millis(10),
            now: Box::new(|| Ok(UnixMillis(0))),
            history_limit: DEFAULT_HISTORY_LIMIT,
            sequence: 0,
            last_observed: None,
            changes: VecDeque::new(),
            timeline: VecDeque::new(),
        }
    }

    fn next_sequence(&mut self) -> Result<u64, WorkerError> {
        let next = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| WorkerError::Message("debug_api sequence overflow".to_string()))?;
        self.sequence = next;
        Ok(next)
    }

    fn trim_history(&mut self) {
        while self.changes.len() > self.history_limit {
            let _ = self.changes.pop_front();
        }
        while self.timeline.len() > self.history_limit {
            let _ = self.timeline.pop_front();
        }
    }

    fn record_change(
        &mut self,
        now: UnixMillis,
        domain: DebugDomain,
        previous_version: Option<Version>,
        current_version: Option<Version>,
        summary: String,
    ) -> Result<(), WorkerError> {
        let sequence = self.next_sequence()?;
        self.changes.push_back(DebugChangeEvent {
            sequence,
            at: now,
            domain: domain.clone(),
            previous_version,
            current_version,
            summary: summary.clone(),
        });
        self.timeline.push_back(DebugTimelineEntry {
            sequence,
            at: now,
            domain,
            message: summary,
        });
        self.trim_history();
        Ok(())
    }
}

pub(crate) async fn run(mut ctx: DebugApiCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut DebugApiCtx) -> Result<(), WorkerError> {
    let now = (ctx.now)()?;
    let snapshot_ctx = DebugSnapshotCtx {
        app: ctx.app.clone(),
        config: ctx.config_subscriber.latest(),
        pg: ctx.pg_subscriber.latest(),
        dcs: ctx.dcs_subscriber.latest(),
        process: ctx.process_subscriber.latest(),
        ha: ctx.ha_subscriber.latest(),
    };

    let config_summary = summarize_config(&snapshot_ctx.config.value);
    let pg_summary = summarize_pg(&snapshot_ctx.pg.value);
    let dcs_summary = summarize_dcs(&snapshot_ctx.dcs.value);
    let process_summary = summarize_process(&snapshot_ctx.process.value);
    let ha_summary = summarize_ha(&snapshot_ctx.ha.value);
    let ha_sig = ha_signature(&snapshot_ctx.ha.value);

    let observed = DebugObservedState {
        app: snapshot_ctx.app.clone(),
        config_version: snapshot_ctx.config.version,
        config_sig: config_summary.clone(),
        pg_version: snapshot_ctx.pg.version,
        pg_sig: pg_summary.clone(),
        dcs_version: snapshot_ctx.dcs.version,
        dcs_sig: dcs_summary.clone(),
        process_version: snapshot_ctx.process.version,
        process_sig: process_summary.clone(),
        ha_version: snapshot_ctx.ha.version,
        ha_sig,
    };

    if let Some(previous) = ctx.last_observed.clone() {
        if previous.app != observed.app {
            ctx.record_change(
                now,
                DebugDomain::App,
                None,
                None,
                summarize_app(&observed.app),
            )?;
        }
        if previous.config_sig != observed.config_sig {
            ctx.record_change(
                now,
                DebugDomain::Config,
                Some(previous.config_version),
                Some(observed.config_version),
                config_summary.clone(),
            )?;
        }
        if previous.pg_sig != observed.pg_sig {
            ctx.record_change(
                now,
                DebugDomain::PgInfo,
                Some(previous.pg_version),
                Some(observed.pg_version),
                pg_summary.clone(),
            )?;
        }
        if previous.dcs_sig != observed.dcs_sig {
            ctx.record_change(
                now,
                DebugDomain::Dcs,
                Some(previous.dcs_version),
                Some(observed.dcs_version),
                dcs_summary.clone(),
            )?;
        }
        if previous.process_sig != observed.process_sig {
            ctx.record_change(
                now,
                DebugDomain::Process,
                Some(previous.process_version),
                Some(observed.process_version),
                process_summary.clone(),
            )?;
        }
        if previous.ha_sig != observed.ha_sig {
            ctx.record_change(
                now,
                DebugDomain::Ha,
                Some(previous.ha_version),
                Some(observed.ha_version),
                ha_summary.clone(),
            )?;
        }
    } else {
        ctx.record_change(
            now,
            DebugDomain::App,
            None,
            None,
            summarize_app(&observed.app),
        )?;
        ctx.record_change(
            now,
            DebugDomain::Config,
            None,
            Some(observed.config_version),
            config_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::PgInfo,
            None,
            Some(observed.pg_version),
            pg_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::Dcs,
            None,
            Some(observed.dcs_version),
            dcs_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::Process,
            None,
            Some(observed.process_version),
            process_summary,
        )?;
        ctx.record_change(
            now,
            DebugDomain::Ha,
            None,
            Some(observed.ha_version),
            ha_summary,
        )?;
    }

    ctx.last_observed = Some(observed);

    let changes = ctx.changes.iter().cloned().collect::<Vec<_>>();
    let timeline = ctx.timeline.iter().cloned().collect::<Vec<_>>();
    let snapshot = build_snapshot(&snapshot_ctx, now, ctx.sequence, &changes, &timeline);

    ctx.publisher
        .publish(snapshot, now)
        .map_err(|err| WorkerError::Message(format!("debug_api publish failed: {err}")))?;
    Ok(())
}

fn summarize_app(app: &AppLifecycle) -> String {
    format!("app={app:?}")
}

fn summarize_config(config: &RuntimeConfig) -> String {
    format!(
        "cluster={} member={} scope={} debug_enabled={} tls_enabled={}",
        config.cluster.name,
        config.cluster.member_id,
        config.dcs.scope,
        config.debug.enabled,
        config.api.security.tls.mode != crate::config::ApiTlsMode::Disabled
    )
}

fn summarize_pg(state: &PgInfoState) -> String {
    match state {
        PgInfoState::Unknown { common } => {
            format!(
                "pg=unknown worker={:?} sql={:?} readiness={:?}",
                common.worker, common.sql, common.readiness
            )
        }
        PgInfoState::Primary {
            common,
            wal_lsn,
            slots,
        } => {
            format!(
                "pg=primary worker={:?} wal_lsn={} slots={}",
                common.worker,
                wal_lsn.0,
                slots.len()
            )
        }
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => {
            format!(
                "pg=replica worker={:?} replay_lsn={} follow_lsn={} upstream={}",
                common.worker,
                replay_lsn.0,
                follow_lsn
                    .map(|value| value.0)
                    .map_or_else(|| "none".to_string(), |value| value.to_string()),
                upstream
                    .as_ref()
                    .map(|value| value.member_id.0.clone())
                    .unwrap_or_else(|| "none".to_string())
            )
        }
    }
}

fn summarize_dcs(state: &DcsState) -> String {
    format!(
        "dcs worker={:?} trust={:?} members={} leader={} switchover={}",
        state.worker,
        state.trust,
        state.cache.members.len(),
        state
            .cache
            .leader
            .as_ref()
            .map(|leader| leader.member_id.0.clone())
            .unwrap_or_else(|| "none".to_string()),
        state.cache.switchover.is_some()
    )
}

fn summarize_process(state: &ProcessState) -> String {
    match state {
        ProcessState::Idle {
            worker,
            last_outcome,
        } => {
            format!("process=idle worker={worker:?} last_outcome={last_outcome:?}")
        }
        ProcessState::Running { worker, active } => {
            format!(
                "process=running worker={worker:?} job_id={} kind={:?}",
                active.id.0, active.kind
            )
        }
    }
}

fn summarize_ha(state: &HaState) -> String {
    let decision_detail = state
        .decision
        .detail()
        .unwrap_or_else(|| "<none>".to_string());
    format!(
        "ha worker={:?} phase={:?} tick={} decision={} detail={} planned_actions={}",
        state.worker,
        state.phase,
        state.tick,
        state.decision.label(),
        decision_detail,
        lower_decision(&state.decision).len()
    )
}

fn ha_signature(state: &HaState) -> String {
    let decision_detail = state
        .decision
        .detail()
        .unwrap_or_else(|| "<none>".to_string());
    format!(
        "ha worker={:?} phase={:?} decision={} detail={}",
        state.worker,
        state.phase,
        state.decision.label(),
        decision_detail
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::{ApiTlsMode, RuntimeConfig},
        dcs::state::{DcsCache, DcsState, DcsTrust},
        debug_api::snapshot::{AppLifecycle, DebugDomain, SystemSnapshot},
        ha::decision::HaDecision,
        ha::state::{HaPhase, HaState},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{new_state_channel, UnixMillis, WorkerError, WorkerStatus},
    };

    use super::{DebugApiContractStubInputs, DebugApiCtx};

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_pg_state() -> PgInfoState {
        PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: WorkerStatus::Starting,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: None,
            },
        }
    }

    fn sample_dcs_state(cfg: RuntimeConfig) -> DcsState {
        DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: DcsCache {
                members: BTreeMap::new(),
                leader: None,
                switchover: None,
                config: cfg,
                init_lock: None,
            },
            last_refresh_at: None,
        }
    }

    fn sample_process_state() -> ProcessState {
        ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        }
    }

    fn sample_ha_state() -> HaState {
        HaState {
            worker: WorkerStatus::Starting,
            phase: HaPhase::Init,
            tick: 0,
            decision: HaDecision::EnterFailSafe {
                release_leader_lease: false,
            },
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_publishes_snapshot() -> Result<(), crate::state::WorkerError> {
        let cfg = sample_runtime_config();
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));

        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        let (publisher, subscriber) = new_state_channel(
            SystemSnapshot {
                app: AppLifecycle::Starting,
                config: cfg_subscriber.latest(),
                pg: pg_subscriber.latest(),
                dcs: dcs_subscriber.latest(),
                process: process_subscriber.latest(),
                ha: ha_subscriber.latest(),
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        });
        ctx.now = Box::new(|| Ok(UnixMillis(2)));
        ctx.app = AppLifecycle::Running;

        super::step_once(&mut ctx).await?;
        let latest = subscriber.latest();
        assert_eq!(latest.updated_at, UnixMillis(2));
        assert_eq!(latest.value.app, AppLifecycle::Running);
        assert_eq!(latest.value.sequence, 6);
        assert_eq!(latest.value.changes.len(), 6);
        assert_eq!(latest.value.timeline.len(), 6);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_keeps_history_when_versions_unchanged(
    ) -> Result<(), crate::state::WorkerError> {
        let cfg = sample_runtime_config();
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        let (publisher, subscriber) = new_state_channel(
            SystemSnapshot {
                app: AppLifecycle::Starting,
                config: cfg_subscriber.latest(),
                pg: pg_subscriber.latest(),
                dcs: dcs_subscriber.latest(),
                process: process_subscriber.latest(),
                ha: ha_subscriber.latest(),
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ticks = vec![UnixMillis(2), UnixMillis(3)].into_iter();
        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        });
        ctx.now = Box::new(move || {
            ticks
                .next()
                .ok_or_else(|| WorkerError::Message("clock exhausted".to_string()))
        });

        super::step_once(&mut ctx).await?;
        let first = subscriber.latest();
        super::step_once(&mut ctx).await?;
        let second = subscriber.latest();

        assert_eq!(first.value.sequence, second.value.sequence);
        assert_eq!(first.value.changes.len(), second.value.changes.len());
        assert_eq!(first.value.timeline.len(), second.value.timeline.len());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_records_incremental_version_changes() -> Result<(), crate::state::WorkerError>
    {
        let cfg = sample_runtime_config();
        let (cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        let (publisher, subscriber) = new_state_channel(
            SystemSnapshot {
                app: AppLifecycle::Starting,
                config: cfg_subscriber.latest(),
                pg: pg_subscriber.latest(),
                dcs: dcs_subscriber.latest(),
                process: process_subscriber.latest(),
                ha: ha_subscriber.latest(),
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ticks = vec![UnixMillis(2), UnixMillis(4)].into_iter();
        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        });
        ctx.now = Box::new(move || {
            ticks
                .next()
                .ok_or_else(|| WorkerError::Message("clock exhausted".to_string()))
        });

        super::step_once(&mut ctx).await?;
        let before = subscriber.latest().value.sequence;

        let mut updated_cfg = cfg.clone();
        updated_cfg.api.security.tls.mode = ApiTlsMode::Required;
        cfg_publisher
            .publish(updated_cfg, UnixMillis(3))
            .map_err(|err| WorkerError::Message(format!("cfg publish failed: {err}")))?;

        super::step_once(&mut ctx).await?;
        let latest = subscriber.latest();
        assert!(latest.value.sequence > before);

        let config_events = latest
            .value
            .changes
            .iter()
            .filter(|event| matches!(event.domain, DebugDomain::Config))
            .collect::<Vec<_>>();
        assert!(!config_events.is_empty());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_does_not_record_ha_tick_only_changes(
    ) -> Result<(), crate::state::WorkerError> {
        let cfg = sample_runtime_config();
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));

        let initial_ha = sample_ha_state();
        let (ha_publisher, ha_subscriber) = new_state_channel(initial_ha.clone(), UnixMillis(1));

        let (publisher, subscriber) = new_state_channel(
            SystemSnapshot {
                app: AppLifecycle::Starting,
                config: cfg_subscriber.latest(),
                pg: pg_subscriber.latest(),
                dcs: dcs_subscriber.latest(),
                process: process_subscriber.latest(),
                ha: ha_subscriber.latest(),
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ticks = vec![UnixMillis(2), UnixMillis(3)].into_iter();
        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber: ha_subscriber.clone(),
        });
        ctx.now = Box::new(move || {
            ticks
                .next()
                .ok_or_else(|| WorkerError::Message("clock exhausted".to_string()))
        });

        super::step_once(&mut ctx).await?;
        let before = subscriber.latest();
        let before_timeline_len = before.value.timeline.len();
        let before_sequence = before.value.sequence;

        let mut ha_bumped_tick = initial_ha.clone();
        ha_bumped_tick.tick = ha_bumped_tick.tick.saturating_add(1);
        ha_publisher
            .publish(ha_bumped_tick.clone(), UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;

        super::step_once(&mut ctx).await?;
        let after = subscriber.latest();
        assert_eq!(after.value.timeline.len(), before_timeline_len);
        assert_eq!(after.value.sequence, before_sequence);
        assert_eq!(after.value.ha.value.tick, ha_bumped_tick.tick);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_history_retention_trims_old_entries() -> Result<(), crate::state::WorkerError>
    {
        let cfg = sample_runtime_config();
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        let (publisher, subscriber) = new_state_channel(
            SystemSnapshot {
                app: AppLifecycle::Starting,
                config: cfg_subscriber.latest(),
                pg: pg_subscriber.latest(),
                dcs: dcs_subscriber.latest(),
                process: process_subscriber.latest(),
                ha: ha_subscriber.latest(),
                generated_at: UnixMillis(1),
                sequence: 0,
                changes: Vec::new(),
                timeline: Vec::new(),
            },
            UnixMillis(1),
        );

        let mut ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
            publisher,
            config_subscriber: cfg_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            ha_subscriber,
        });
        ctx.history_limit = 3;
        ctx.now = Box::new(|| Ok(UnixMillis(2)));

        super::step_once(&mut ctx).await?;
        let latest = subscriber.latest();
        assert_eq!(latest.value.changes.len(), 3);
        assert_eq!(latest.value.timeline.len(), 3);
        Ok(())
    }
}

--- END FILE: src/debug_api/worker.rs ---

