# Debug API

The debug API is a built-in read-only observability surface. It is available only when debug mode is enabled via `debug.enabled` in the runtime configuration.

## Availability

The debug API runs on the same listener as the main HTTP API and obeys its TLS and authentication settings. When `debug.enabled` is false, all debug endpoints return `404 Not Found`. When true, they are accessible to holders of a valid read token, or to any client if API authentication is disabled.

## Endpoint Summary

| Method | Path | Role | Purpose |
|--------|------|------|---------|
| GET | `/debug/verbose` | read | Stable JSON export of full cluster state |
| GET | `/debug/snapshot` | read | Raw snapshot diagnostic dump |
| GET | `/debug/ui` | read | Browser-based dashboard |

---

### `GET /debug/verbose`

Returns a JSON object that combines current configuration, PostgreSQL state, DCS view, process queue, and HA decision.

#### Query Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `since` | integer (optional) | Minimum sequence number. If supplied, only `changes` and `timeline` entries with `sequence > since` are included. |

#### Response Structure

```
{
  "meta": {...},
  "config": {...},
  "pginfo": {...},
  "dcs": {...},
  "process": {...},
  "ha": {...},
  "api": {...},
  "debug": {...},
  "changes": [...],
  "timeline": [...]
}
```

---

#### `meta`

| Field | Type | Description |
|-------|------|-------------|
| `schema_version` | string | Format version, currently `"v1"` |
| `generated_at_ms` | integer | Clock time at snapshot generation (Unix milliseconds) |
| `channel_updated_at_ms` | integer | Last update time on the internal snapshot state channel |
| `channel_version` | integer | Current snapshot state channel version |
| `app_lifecycle` | string | Application phase (`Starting`, `Running`) |
| `sequence` | integer | Monotonic counter for this snapshot |

---

#### `config`

| Field | Type | Description |
|-------|------|-------------|
| `version` | integer | Config state channel version |
| `updated_at_ms` | integer | Config publish time |
| `cluster_name` | string | Target cluster name |
| `member_id` | string | Self node identifier |
| `scope` | string | DCS scope (prefix for etcd keys) |
| `debug_enabled` | boolean | Whether debug mode is active |
| `tls_enabled` | boolean | Whether HTTPS listener mode is enabled |

---

#### `pginfo`

PostgreSQL state normalized to a common structure.

| Field | Type | Description |
|-------|------|-------------|
| `version` | integer | PostgreSQL state channel version |
| `updated_at_ms` | integer | Last Postgres info refresh time |
| `variant` | string | One of: `Unknown`, `Primary`, `Replica` |
| `worker` | string | Worker status label |
| `sql` | string | SQL status label |
| `readiness` | string | Readiness label |
| `timeline` | integer (null or value) | Current WAL timeline |
| `summary` | string | Human-readable compact summary |

**Worker status labels**: `Starting`, `Running`, `Stopping`, `Stopped`, `Faulted(<error>)`  
**SQL status labels**: `Unknown`, `Healthy`, `Unreachable`  
**Readiness labels**: `Unknown`, `Ready`, `NotReady`

---

#### `dcs`

Distributed consensus store view.

| Field | Type | Description |
|---------------|------|-------------------|
| `version` | integer | DCS state channel version |
| `updated_at_ms` | integer | Last DCS refresh time |
| `worker` | string | Worker status label |
| `trust` | string | Trust level of DCS quorum |
| `member_count` | integer | Number of known members in scope |
| `leader` | string (null or member id) | Current leader member id |
| `has_switchover_request` | boolean | Whether a switchover is queued |

**Trust labels**: `FullQuorum`, `FailSafe`, `NotTrusted`

---

#### `process`

State of the process job queue.

| Field | Type | Description |
|------|------|-------------|
| `version` | integer | Process state channel version |
| `updated_at_ms` | integer | Last process state update time |
| `worker` | string | Worker status label |
| `state` | string | `Idle` (no active job) or `Running` (executing a job) |
| `running_job_id` | string (null or id) | Current job id if state is Running |
| `last_outcome` | string (null or label) | Result of last completed job if idle |

**Outcome labels**: `Success(<jobid>)`, `Failure(<jobid>: <error>)`, `Timeout(<jobid>)`

---

#### `ha`

High-availability decision engine.

| Field | Type | Description |
|------|------|-------------|
| `version` | integer | HA state channel version |
| `updated_at_ms` | integer | Last HA tick publish time |
| `worker` | string | Worker status label |
| `phase` | string | Logical phase of HA state machine |
| `tick` | integer | Tick count since start |
| `decision` | string | Decision label chosen on this tick |
| `decision_detail` | string (null or detail) | Optional extra context |
| `planned_actions` | integer | Number of discrete operations after lowering the decision |

**Phase labels**: `init`, `waiting_postgres_reachable`, `waiting_dcs_trusted`, `waiting_switchover_successor`, `replica`, `candidate_leader`, `primary`, `rewinding`, `bootstrapping`, `fencing`, `fail_safe`  
**Decision labels**: `no_change`, `wait_for_postgres`, `wait_for_dcs_trust`, `attempt_leadership`, `follow_leader`, `become_primary`, `step_down`, `recover_replica`, `fence_node`, `release_leader_lease`, `enter_fail_safe`

---

#### `api`

Static list of available endpoints.

| Field | Type | Description |
|------|------|-------------|
| `endpoints` | array of strings | Supported paths |

Value:

```
[
  "/debug/snapshot",
  "/debug/verbose",
  "/debug/ui",
  "/fallback/cluster",
  "/switchover",
  "/ha/state",
  "/ha/switchover"
]
```

---

#### `debug`

Debug-channel metadata.

| Field | Type | Description |
|------|------|-------------|
| `history_changes` | integer | Number of change events retained in memory |
| `history_timeline` | integer | Number of timeline events retained in memory |
| `last_sequence` | integer | Highest sequence number in the history |

---

#### `changes`

Array of change events. Each entry represents a detected state-channel mutation.

| Field | Type | Description |
|------|------|-------------|
| `sequence` | integer | Event order (unique within lifetime) |
| `at_ms` | integer | Event timestamp |
| `domain` | string | Affected subsystem: `config`, `pginfo`, `dcs`, `process`, `ha`, `app` |
| `previous_version` | integer (null or value) | Version before change |
| `current_version` | integer (null or value) | Version after change |
| `summary` | string | Human-readable description |

**Retention**: By default the latest 300 change events are kept per node. Use `since` to poll incrementally.

---

#### `timeline`

Array of timeline entries. Each entry is a log-style event produced during worker ticks.

| Field | Type | Description |
|------|------|-------------|
| `sequence` | integer | Event order |
| `at_ms` | integer | Event timestamp |
| `category` | string | Source domain (same set as `changes`) |
| `message` | string | Free-form message |

**Retention**: By default the latest 300 timeline events are kept.

---

#### Filtering Behavior

When `since` is supplied, both `changes` and `timeline` arrays are trimmed to items where `sequence > since`. The top-level `meta.sequence` remains the current snapshot sequence, allowing a client to use it as the next `since` value on the next poll.

---

### `GET /debug/snapshot`

Returns the raw internal `SystemSnapshot` structure. This endpoint is intended for diagnostic tooling that needs access to the exact state and version metadata. The format is subject to change without notice.

---

### `GET /debug/ui`

Serves a read-only HTML dashboard that calls `/debug/verbose?since=...` and renders a browsable timeline and change table. No additional data endpoint is required.

---

## System Snapshot Composition

The debug worker polls each subsystem state channel and builds a composite snapshot whenever any channel version increases.

```mermaid
graph TD
    subgraph State Channels
        cfg[Config Channel<br/>Versioned&lt;RuntimeConfig&gt;]
        pg[PostgreSQL Channel<br/>Versioned&lt;PgInfoState&gt;]
        dcs[DCS Channel<br/>Versioned&lt;DcsState&gt;]
        proc[Process Channel<br/>Versioned&lt;ProcessState&gt;]
        ha[HA Channel<br/>Versioned&lt;HaState&gt;]
    end

    worker[Debug API Worker<br/>Poll + Deduplicate + Append Events]
    snapshot[System Snapshot<br/>Versioned&lt;SystemSnapshot&gt;]
    verbose[/debug/verbose<br/>JSON Projection]

    cfg --> worker
    pg --> worker
    dcs --> worker
    proc --> worker
    ha --> worker
    worker --> snapshot
    snapshot --> verbose
```

---

## Update Flow

```mermaid
sequenceDiagram
    participant PgWorker
    participant DcsWorker
    participant HaWorker
    participant DebugWorker
    participant Snapshot

    loop Every N ms
        PgWorker->>DebugWorker: publish PgInfoState
        DcsWorker->>DebugWorker: publish DcsState
        HaWorker->>DebugWorker: publish HaState
    end

    DebugWorker->>DebugWorker: Detect changed versions
    DebugWorker->>DebugWorker: Append change/timeline events
    DebugWorker->>Snapshot: Publish Versioned&lt;SystemSnapshot&gt;
```

On each worker poll, the debug layer compares the current version of each channel to its previous version. If changed, a `DebugChangeEvent` is added to the snapshot history with a compact summary. A `DebugTimelineEntry` is also recorded for certain predefined log-worthy transitions. The history queues are trimmed to a fixed ring size (default 300 each) to bound memory usage.

---

## Field Labels and Enums

| Subsystem | Label Set |
|------------|-----------|
| Worker status | `Starting`, `Running`, `Stopping`, `Stopped`, `Faulted(<text>)` |
| SQL status | `Unknown`, `Healthy`, `Unreachable` |
| Readiness | `Unknown`, `Ready`, `NotReady` |
| DCS trust | `FullQuorum`, `FailSafe`, `NotTrusted` |
| HA phase | `init`, `waiting_postgres_reachable`, `waiting_dcs_trusted`, `waiting_switchover_successor`, `replica`, `candidate_leader`, `primary`, `rewinding`, `bootstrapping`, `fencing`, `fail_safe` |
| HA decision | `no_change`, `wait_for_postgres`, `wait_for_dcs_trust`, `attempt_leadership`, `follow_leader`, `become_primary`, `step_down`, `recover_replica`, `fence_node`, `release_leader_lease`, `enter_fail_safe` |

---

## Example

```bash
curl -H "Authorization: Bearer ${READ_TOKEN}" \
  https://node-a.pgtuskmaster.local:8443/debug/verbose?since=0
```

Returns a JSON object as described in the Response Structure section. Client polling pattern:

1. Initial request without `since` (or `since=0`)
2. Extract `meta.sequence`
3. On next poll, use `?since=<previous_sequence>`
4. New entries appear in `changes` and `timeline` if state moved forward

---

## Integration Notes

The debug API reuses the same state channels that feed the `/ha/state` endpoint but augments them with history and compact projections. There is no separate listener or port. Debug data is not persisted to disk; it exists only in memory on each node. If a node restarts, its debug history is reset.

Missing source support: None. All described functionality is present in the supplied source files.
