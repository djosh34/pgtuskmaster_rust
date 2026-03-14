# Process Management and Execution Domain

Process management is the execution boundary between the HA reconciler and the operating system. The HA side decides what should happen next. The process domain turns that decision into concrete PostgreSQL subprocess work, records the outcome, and publishes state for the rest of the node.

## Startup Boundary

The narrowed startup rewrite moved process-specific startup policy behind `ProcessRuntimePlan` and `process::startup::bootstrap(...)`.

`ProcessRuntimePlan::from_config(...)` projects the parts of `RuntimeConfig` that the process and pginfo domains need repeatedly:

- managed PostgreSQL paths and listen port
- replication-source defaults for replicator and rewinder jobs
- connection defaults such as dbname, SSL mode, CA path, and connect timeout

`ProcessRuntimePlan::ensure_start_paths()` also moved out of `runtime/node.rs`. It creates the data-dir parent, data dir, socket dir, and log parent before workers start. On Unix it additionally sets `0o700` permissions on the data directory.

At runtime composition level, `src/runtime/node.rs` now creates the plan once, prepares the paths once, and passes the typed plan into the owning startup modules instead of rebuilding loose strings and paths across domains.

## Worker Context Shape

`ProcessWorkerCtx` is no longer a flat startup bag. It groups concerns into narrower ADTs:

- `cadence`: worker poll interval and time source
- `config`: process-level timeout and binary configuration
- `identity`: the local `MemberId`
- `observed`: live `RuntimeConfig` and `DcsView` subscribers
- `plan`: the stable `ProcessRuntimePlan`
- `state_channel`: current `ProcessState`, publisher, and last rejection
- `control`: the inbox plus the optional active runtime
- `runtime`: logging, subprocess-output capture flag, and command runner

That split keeps the startup boundary smaller and makes cross-domain dependencies more explicit. The worker reads local identity and long-lived runtime defaults from typed bundles instead of from many unrelated top-level fields.

## Intent Flow

The HA reconciler never spawns a subprocess directly. It emits `ProcessIntent` values such as:

- `Bootstrap`
- `ProvisionReplica(BaseBackup | PgRewind)`
- `Start(Primary | DetachedStandby | Replica)`
- `Promote`
- `Demote(Fast | Immediate)`

`src/ha/process_dispatch.rs` converts each intent into a `ProcessIntentRequest` with a deterministic `JobId` built from scope, member id, HA tick, action index, and intent label. That request is sent through the process worker inbox.

```mermaid
flowchart LR
    A[HA reconcile] --> B[ProcessIntent]
    B --> C[process_dispatch]
    C --> D[ProcessIntentRequest<br/>deterministic JobId]
    D --> E[Process worker inbox]
    E --> F[Materialize execution request]
    F --> G[Build command spec]
    G --> H[Spawn PostgreSQL tool process]
    H --> I[Drain output and poll exit]
    I --> J[Publish ProcessState and JobOutcome]
```

If the worker is already busy, the new request is rejected without starting a second job. That rejection is recorded in `state_channel.last_rejection` and logged as a worker event.

## Materialization and Validation

The process worker turns `ProcessIntentRequest` into a concrete `ProcessExecutionRequest` inside `materialize_execution_request(...)`.

For replica-provisioning paths, materialization reads the latest DCS view and validates the chosen leader before building conninfo:

- the source member must not be `self`
- the advertised PostgreSQL host must be non-empty
- the source member must currently present as a primary in DCS

Those checks live in `src/process/source.rs` and use the typed replication-source defaults stored in `ProcessRuntimePlan`. That keeps replication-source policy in the process domain instead of leaving it spread across HA and runtime startup code.

The same materialization step also converts start intents into concrete PostgreSQL start specs, including detached-standby and replica-start managed configuration.

## Job Lifecycle and Timeouts

`ProcessState` exposes two high-level states:

- `Idle { worker, last_outcome }`
- `Running { worker, active }`

Internally, `ActiveRuntime` holds the execution request, deadline, process handle, and structured log identity for the running job.

Timeouts are enforced by deadline checks inside `tick_active_job(...)`. Different execution kinds resolve to different timeout defaults from `ProcessConfig`:

- bootstrap, basebackup, promote, and start-postgres use the bootstrap timeout unless the spec overrides it
- pg_rewind uses the pg_rewind timeout unless overridden
- demote uses the fencing timeout unless overridden

When the deadline is exceeded, the worker logs a timeout event, calls the process handle cancellation path, drains any remaining output, and transitions back to idle. In the current implementation that cancellation path is kill-based: `TokioProcessHandle::cancel()` uses `start_kill()` followed by `wait()`. A successful cancellation produces `JobOutcome::Timeout`; a cancellation failure becomes `JobOutcome::Failure`.

Subprocess output is drained during execution and again during shutdown paths. When `logging.capture_subprocess_output` is enabled, the process startup bundle projects that setting into `ProcessRuntime.capture_subprocess_output`, and stdout/stderr lines are emitted as structured subprocess log records tagged with the job identity.

## PostgreSQL Preflight Safety

The start-postgres path does extra preflight work before spawning `pg_ctl start`.

- It checks `postmaster.pid` in the configured data directory.
- It verifies whether that PID still exists and, on Unix, whether `/proc/<pid>/cmdline` looks like a PostgreSQL postmaster for the same data directory.
- It checks the PostgreSQL socket lock file for the configured port.
- If the PID or socket-lock evidence is stale, it removes the stale files before continuing.
- If PostgreSQL already appears to be running for that data directory or port, the start job becomes a no-op success instead of spawning another process.

This keeps the start path crash-tolerant and reduces false-positive "already running" failures after unclean shutdowns.

## Integration with PgInfo and API

The pginfo domain now shares the same `ProcessRuntimePlan` at startup rather than rebuilding its local socket target in `runtime/node.rs`. `PgProbeTarget::local_from_config(...)` derives the local probe conninfo from the runtime config plus the process plan, so the process and pginfo domains agree on the managed socket directory and port.

The API domain no longer reaches into process startup details either. It consumes published process state through its live observed-state bundle. During startup, the API can stay in `ApiObservedState::Unavailable` until the full live subscriber set is ready, which avoids pretending that partially wired state is already live.

## Why This Boundary Is Better

The rewrite makes `src/runtime/node.rs` a smaller composition root:

- runtime validates top-level config and boots global services
- process startup owns process-specific path preparation and runtime projection
- pginfo startup owns its local probe target
- HA sends typed intents instead of process commands
- API consumes published state instead of process internals

That boundary reduces startup duplication, shrinks the number of raw fields runtime must know about, and keeps process execution policy close to the code that actually launches and supervises PostgreSQL subprocesses.
