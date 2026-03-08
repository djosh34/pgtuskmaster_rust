# Process Worker Reference

The process worker spans `src/process/state.rs`, `src/process/jobs.rs`, and `src/process/worker.rs`. It accepts one PostgreSQL operation request at a time, maps the request to a subprocess invocation, tracks active execution, and publishes process state transitions.

## State Model

### `ProcessState`

| Variant | Fields |
|---|---|
| `Idle` | `worker: WorkerStatus`, `last_outcome: Option<JobOutcome>` |
| `Running` | `worker: WorkerStatus`, `active: ActiveJob` |

### `ActiveJob`

| Field | Meaning |
|---|---|
| `id` | job identifier |
| `kind` | `ActiveJobKind` |
| `started_at` | job start time |
| `deadline_at` | timeout deadline |

### `ProcessWorkerCtx`

| Field |
|---|
| `poll_interval` |
| `config` |
| `log` |
| `capture_subprocess_output` |
| `state` |
| `publisher` |
| `inbox` |
| `inbox_disconnected_logged` |
| `command_runner` |
| `active_runtime` |
| `last_rejection` |
| `now` |

## Worker Loop

`run(ctx)` emits `process.worker.run_started`, records `capture_subprocess_output`, and then loops forever:

1. call `step_once(&mut ctx).await?`
2. sleep for `ctx.poll_interval`

`step_once` uses `ctx.inbox.try_recv()`:

| Inbox result | Behavior |
|---|---|
| `Ok(request)` | emits `process.worker.request_received`, then calls `start_job` |
| `Err(TryRecvError::Empty)` | no request handling |
| `Err(TryRecvError::Disconnected)` | emits `process.worker.inbox_disconnected` once, then continues running |

After inbox handling, `step_once` always calls `tick_active_job(ctx).await`.

## Job And Supporting Types

### Request And Outcome Types

| Type | Fields |
|---|---|
| `ProcessJobRequest` | `id`, `kind` |
| `ProcessJobRejection` | `id`, `error`, `rejected_at` |
| `JobOutcome::Success` | `id`, `job_kind`, `finished_at` |
| `JobOutcome::Failure` | `id`, `job_kind`, `error`, `finished_at` |
| `JobOutcome::Timeout` | `id`, `job_kind`, `finished_at` |

### Job Kinds

`ActiveJobKind` values:

- `Bootstrap`
- `BaseBackup`
- `PgRewind`
- `Promote`
- `Demote`
- `StartPostgres`
- `Fencing`

`ProcessJobKind::label()` returns:

| Variant | Label |
|---|---|
| `Bootstrap` | `bootstrap` |
| `BaseBackup` | `basebackup` |
| `PgRewind` | `pg_rewind` |
| `Promote` | `promote` |
| `Demote` | `demote` |
| `StartPostgres` | `start_postgres` |
| `Fencing` | `fencing` |

Supporting types:

| Type | Fields |
|---|---|
| `ReplicatorSourceConn` | `conninfo`, `auth` |
| `RewinderSourceConn` | `conninfo`, `auth` |

`ShutdownMode` values are `Fast` and `Immediate`. `as_pg_ctl_arg()` returns `fast` or `immediate`.

## Timeout And Command Resolution

### `timeout_for_kind`

| Job kind | Timeout source |
|---|---|
| `Bootstrap` | `spec.timeout_ms` or `config.bootstrap_timeout_ms` |
| `BaseBackup` | `spec.timeout_ms` or `config.bootstrap_timeout_ms` |
| `PgRewind` | `spec.timeout_ms` or `config.pg_rewind_timeout_ms` |
| `Promote` | `spec.timeout_ms` or `config.bootstrap_timeout_ms` |
| `Demote` | `spec.timeout_ms` or `config.fencing_timeout_ms` |
| `StartPostgres` | `spec.timeout_ms` or `config.bootstrap_timeout_ms` |
| `Fencing` | `spec.timeout_ms` or `config.fencing_timeout_ms` |

### `build_command`

`build_command` validates non-empty required paths and requires the spawned program path to be absolute.

Command mapping:

| Job kind | Binary | Arguments |
|---|---|---|
| `Bootstrap` | `initdb` | `-D <data_dir> -A trust -U <superuser_username>` |
| `BaseBackup` | `pg_basebackup` | `-h <host> -p <port> -U <user> -D <data_dir> -Fp -Xs` |
| `PgRewind` | `pg_rewind` | `--target-pgdata <target_data_dir> --source-server <rendered conninfo>` |
| `Promote` | `pg_ctl` | `-D <data_dir> promote -w`, plus optional `-t <wait_seconds>` |
| `Demote` | `pg_ctl` | `-D <data_dir> stop -m <mode> -w` |
| `StartPostgres` | `pg_ctl` | `-D <data_dir> -l <log_file> -o <options> start -w -t <wait_seconds>` |
| `Fencing` | `pg_ctl` | `-D <data_dir> stop -m <mode> -w` |

`StartPostgres`:

- requires non-empty `data_dir`, `config_file`, and `log_file`
- defaults `wait_seconds` to `30` when absent
- renders `-o` from the tokens `-c` and `config_file=<path>`

Role-auth environment mapping:

| `RoleAuthConfig` | Environment |
|---|---|
| `Tls` | no env vars |
| `Password` | one secret `PGPASSWORD` env var |

`TokioCommandRunner::spawn`:

- rejects non-absolute program paths
- resolves secret environment values
- sets stdin to null
- pipes stdout and stderr when `capture_output` is true
- discards stdout and stderr when `capture_output` is false

## Preflight, Output, And Outcome Behavior

### Request Admission

`can_accept_job` returns true only when state is `Idle`.

When `start_job` receives a request while state is not `Idle`, it:

- records `last_rejection = Some(ProcessJobRejection { error: ProcessError::Busy, ... })`
- emits `process.worker.busy_reject`
- returns `Ok(())`

### Preflight Rules

`fencing_preflight_is_already_stopped`:

- checks `postmaster.pid`
- returns `true` when the pid file is absent
- returns `false` when a live pid exists
- removes stale `postmaster.pid` and `postmaster.opts` before returning `true`

Fencing preflight outcomes:

| Result | Behavior |
|---|---|
| `Ok(true)` | emits `process.job.fencing_noop`, then transitions directly to idle with `JobOutcome::Success` |
| `Ok(false)` | continues with command execution |
| `Err(error)` | emits `process.job.fencing_preflight_failed`, then transitions to idle with `JobOutcome::Failure` |

`start_postgres_preflight_is_already_running`:

- checks `postmaster.pid`
- returns `true` when the pid file exists and the pid is live
- removes stale `postmaster.pid` and `postmaster.opts` before returning `false`
- returns `false` when the pid file is absent

Start-postgres preflight outcomes:

| Result | Behavior |
|---|---|
| `Ok(true)` | emits `process.job.start_postgres_noop`, then transitions directly to idle with `JobOutcome::Success` |
| `Ok(false)` | continues with command execution |
| `Err(error)` | emits `process.job.start_postgres_preflight_failed`, then transitions to idle with `JobOutcome::Failure` |

### Active Execution

After preflight passes, `start_job`:

- builds the command
- spawns it through `ctx.command_runner`
- stores `ActiveRuntime { request, deadline_at, handle, log_identity }`
- sets state to `Running { worker: WorkerStatus::Running, active }`
- emits `process.job.started`
- publishes the running state

Command-build failure emits `process.job.build_command_failed` and transitions to idle with `JobOutcome::Failure`.

Spawn failure emits `process.job.spawn_failed` and transitions to idle with `JobOutcome::Failure`.

### `tick_active_job`

`tick_active_job` returns immediately when no active runtime exists.

Before checking timeout or exit status, it:

- drains subprocess output up to `PROCESS_OUTPUT_DRAIN_MAX_BYTES`
- emits each line as a raw subprocess record with `LogProducer::PgTool` and origin `process_worker`
- emits `process.worker.output_emit_failed` when line emission fails
- emits `process.worker.output_drain_failed` when output drain fails, then continues processing

Timeout behavior:

- emits `process.job.timeout`
- cancels the subprocess handle
- drains output again
- transitions to idle with `JobOutcome::Timeout` when cancellation succeeds
- transitions to idle with `JobOutcome::Failure` when cancellation fails

Exit-poll behavior:

| `poll_exit()` result | Behavior |
|---|---|
| `Ok(None)` | restores `active_runtime` and keeps running |
| `Ok(Some(ProcessExit::Success))` | drains output again, emits `process.job.exited`, transitions to idle with `JobOutcome::Success` |
| `Ok(Some(ProcessExit::Failure { code }))` | drains output again, converts with `ProcessError::from_exit`, emits `process.job.exited`, transitions to idle with `JobOutcome::Failure` |
| `Err(error)` | drains output again, emits `process.job.poll_failed`, transitions to idle with `JobOutcome::Failure` |

`transition_to_idle` sets `ProcessState::Idle { worker: WorkerStatus::Running, last_outcome: Some(outcome) }` and publishes it.

`publish_state` maps publisher errors to `WorkerError::Message("process publish failed: ...")`.

`system_now_unix_millis` converts `SystemTime::now()` to `UnixMillis` and returns `WorkerError::Message` for pre-epoch or conversion failure.

## Core Process Types

| Type | Shape |
|---|---|
| `ProcessCommandSpec` | `program`, `args`, `env`, `capture_output`, `log_identity` |
| `ProcessLogIdentity` | `job_id`, `job_kind`, `binary` |
| `ProcessOutputStream` | `Stdout` or `Stderr` |
| `ProcessOutputLine` | `stream`, `bytes` |
| `ProcessExit` | `Success` or `Failure { code }` |
| `ProcessError` | `OperationFailed`, `Busy`, `InvalidSpec(String)`, `EnvSecretResolutionFailed { key, message }`, `SpawnFailure { binary, message }`, `EarlyExit { code }`, `CancelFailure(String)` |

## Verified Behaviors

Tests in `src/process/worker.rs` verify:

- `build_command` maps `BaseBackup`, `Bootstrap`, `StartPostgres`, and `PgRewind` to the configured binaries and arguments
- password-based role auth is passed through `PGPASSWORD` rather than argv
- `can_accept_job` returns true only for `ProcessState::Idle`
- `start_job` transitions idle state to running state and publishes the transition
- `StartPostgres` preflight returns an immediate success outcome when `postmaster.pid` points at a live process
- `step_once` emits `process.worker.request_received` and `process.job.started`
- a second request received while a job is active records a deterministic `ProcessError::Busy` rejection
- `tick_active_job` maps subprocess success, early-exit failure, and timeout into the corresponding `JobOutcome`
- real-binary tests execute `initdb`, `pg_rewind`, `pg_ctl promote`, `pg_ctl stop` for demote, and `pg_ctl stop` for fencing through the configured PostgreSQL 16 binaries
- the real `pg_rewind` coverage expects an early-exit failure when the source connection is intentionally invalid
- `start_job` returns a channel-closed error when all state subscribers have been dropped
- `tick_active_job` is a no-op when no active runtime is present

- command mapping across job kinds
- password environment injection for basebackup and pg_rewind
- start-postgres managed config override rendering
- request-received and job-started logging
- busy rejection behavior
- success, failure, and timeout outcome mapping
- noop start-postgres behavior when PostgreSQL is already running
- safety when `tick_active_job` has no active runtime
