# Process Worker Reference

## Overview

The process worker manages PostgreSQL operations by mapping requests to subprocess invocations, tracking execution, and publishing state transitions. It operates in a single-threaded event loop, processing one job at a time.

## Module Surface

| Module | File Path | Responsibility |
|--------|-----------|----------------|
| `jobs` | `src/process/jobs.rs` | Job specifications, command building blocks, error types, and runner traits |
| `state` | `src/process/state.rs` | State machine, worker context, and lifecycle types |
| `worker` | `src/process/worker.rs` | Event loop, preflight checks, and Tokio-based command execution |

## Job and State Types

### ProcessState

| Variant | Fields |
|---------|--------|
| `Idle` | `worker: WorkerStatus`, `last_outcome: Option<JobOutcome>` |
| `Running` | `worker: WorkerStatus`, `active: ActiveJob` |

### ProcessJobKind

| Variant | Wrapped Spec |
|---------|--------------|
| `Bootstrap` | `BootstrapSpec` |
| `BaseBackup` | `BaseBackupSpec` |
| `PgRewind` | `PgRewindSpec` |
| `Promote` | `PromoteSpec` |
| `Demote` | `DemoteSpec` |
| `StartPostgres` | `StartPostgresSpec` |
| `Fencing` | `FencingSpec` |

### ProcessJobRequest

| Field | Type |
|-------|------|
| `id` | `JobId` |
| `kind` | `ProcessJobKind` |

### ProcessJobRejection

| Field | Type |
|-------|------|
| `id` | `JobId` |
| `error` | `ProcessError` |
| `rejected_at` | `UnixMillis` |

### ActiveJob

| Field | Type |
|-------|------|
| `id` | `JobId` |
| `kind` | `ActiveJobKind` |
| `started_at` | `UnixMillis` |
| `deadline_at` | `UnixMillis` |

### ActiveJobKind

- `Bootstrap`
- `BaseBackup`
- `PgRewind`
- `Promote`
- `Demote`
- `StartPostgres`
- `Fencing`

### JobOutcome

| Variant | Fields |
|---------|--------|
| `Success` | `id: JobId`, `job_kind: ActiveJobKind`, `finished_at: UnixMillis` |
| `Failure` | `id: JobId`, `job_kind: ActiveJobKind`, `error: ProcessError`, `finished_at: UnixMillis` |
| `Timeout` | `id: JobId`, `job_kind: ActiveJobKind`, `finished_at: UnixMillis` |

### ShutdownMode

| Variant | `as_pg_ctl_arg()` |
|---------|-------------------|
| `Fast` | `fast` |
| `Immediate` | `immediate` |

### ProcessWorkerCtx

| Field | Type |
|-------|------|
| `poll_interval` | `Duration` |
| `config` | `ProcessConfig` |
| `log` | `LogHandle` |
| `capture_subprocess_output` | `bool` |
| `state` | `ProcessState` |
| `publisher` | `StatePublisher<ProcessState>` |
| `inbox` | `UnboundedReceiver<ProcessJobRequest>` |
| `inbox_disconnected_logged` | `bool` |
| `command_runner` | `Box<dyn ProcessCommandRunner>` |
| `active_runtime` | `Option<ActiveRuntime>` |
| `last_rejection` | `Option<ProcessJobRejection>` |
| `now` | `Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>` |

### ActiveRuntime

| Field | Type |
|-------|------|
| `request` | `ProcessJobRequest` |
| `deadline_at` | `UnixMillis` |
| `handle` | `Box<dyn ProcessHandle>` |
| `log_identity` | `ProcessLogIdentity` |

## Command Runner Surface

### ProcessCommandSpec

| Field | Type |
|-------|------|
| `program` | `PathBuf` |
| `args` | `Vec<String>` |
| `env` | `Vec<ProcessEnvVar>` |
| `capture_output` | `bool` |
| `log_identity` | `ProcessLogIdentity` |

### ProcessEnvVar

| Field | Type |
|-------|------|
| `key` | `String` |
| `value` | `ProcessEnvValue` |

### ProcessEnvValue

| Variant | Content |
|---------|---------|
| `Secret` | `SecretSource` |

### ProcessLogIdentity

| Field | Type |
|-------|------|
| `job_id` | `JobId` |
| `job_kind` | `String` |
| `binary` | `String` |

### ProcessOutputStream

- `Stdout`
- `Stderr`

### ProcessOutputLine

| Field | Type |
|-------|------|
| `stream` | `ProcessOutputStream` |
| `bytes` | `Vec<u8>` |

### ProcessExit

| Variant | Content |
|---------|---------|
| `Success` | |
| `Failure` | `code: Option<i32>` |

### ProcessError

| Variant | Fields |
|---------|--------|
| `OperationFailed` | |
| `Busy` | |
| `InvalidSpec` | `String` |
| `EnvSecretResolutionFailed` | `key: String`, `message: String` |
| `SpawnFailure` | `binary: String`, `message: String` |
| `EarlyExit` | `code: Option<i32>` |
| `CancelFailure` | `String` |

### ProcessHandle Trait

| Method | Signature |
|--------|-----------|
| `poll_exit` | `fn poll_exit(&mut self) -> Result<Option<ProcessExit>, ProcessError>` |
| `drain_output` | `fn drain_output<'a>(&'a mut self, max_bytes: usize) -> Pin<Box<dyn Future<Output = Result<Vec<ProcessOutputLine>, ProcessError>> + Send + 'a>>` |
| `cancel` | `fn cancel<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = Result<(), ProcessError>> + Send + 'a>>` |

### ProcessCommandRunner Trait

| Method | Signature |
|--------|-----------|
| `spawn` | `fn spawn(&mut self, spec: ProcessCommandSpec) -> Result<Box<dyn ProcessHandle>, ProcessError>` |

### TokioCommandRunner

Default implementation of `ProcessCommandRunner`. Spawns commands using `tokio::process::Command`.

## Worker Loop and Request Handling

Constants:

- `PROCESS_OUTPUT_READ_CHUNK_BYTES`: 8192
- `PROCESS_OUTPUT_READ_TIMEOUT`: 1ms
- `PROCESS_OUTPUT_DRAIN_MAX_BYTES`: 256 * 1024
- `PG_CTL_DEFAULT_WAIT_SECONDS`: 30

### run

Emits `process.worker.run_started` and loops forever:
1. Call `step_once(&mut ctx).await?`
2. Sleep for `ctx.poll_interval`

### step_once

Checks `ctx.inbox.try_recv()`:

| Result | Behavior |
|--------|----------|
| `Ok(request)` | Emits `process.worker.request_received`, calls `start_job` |
| `Err(TryRecvError::Empty)` | No action |
| `Err(TryRecvError::Disconnected)` | Emits `process.worker.inbox_disconnected` once, continues |

After inbox handling, always calls `tick_active_job(ctx).await`.

### can_accept_job

Returns `true` only for `ProcessState::Idle`.

## Command Building

### build_command

Validates non-empty paths and requires absolute program paths.

| Job Kind | Binary | Arguments |
|----------|--------|-----------|
| `Bootstrap` | `config.binaries.initdb` | `-D <data_dir> -A trust -U <superuser_username>` |
| `BaseBackup` | `config.binaries.pg_basebackup` | `-h <host> -p <port> -U <user> -D <data_dir> -Fp -Xs` |
| `PgRewind` | `config.binaries.pg_rewind` | `--target-pgdata <target_data_dir> --source-server <conninfo>` |
| `Promote` | `config.binaries.pg_ctl` | `-D <data_dir> promote -w [-t <wait_seconds>]` |
| `Demote` | `config.binaries.pg_ctl` | `-D <data_dir> stop -m <mode> -w` |
| `StartPostgres` | `config.binaries.pg_ctl` | `-D <data_dir> -l <log_file> -o <options> start -w -t <wait_seconds>` |
| `Fencing` | `config.binaries.pg_ctl` | `-D <data_dir> stop -m <mode> -w` |

`StartPostgres` requires non-empty `data_dir`, `config_file`, and `log_file`. Renders `-o` as `-c config_file=<path>`.

### role_auth_env

| `RoleAuthConfig` | Environment Variables |
|------------------|------------------------|
| `Tls` | None |
| `Password` | `PGPASSWORD=<secret>` |

### TokioCommandRunner::spawn

- Rejects non-absolute program paths
- Resolves secret environment values
- Sets `stdin` to null
- Pipes `stdout` and `stderr` when `capture_output` is true
- Discards `stdout` and `stderr` when `capture_output` is false

## Timeouts and Preflight Behavior

### timeout_for_kind

| Job Kind | Timeout Source |
|----------|----------------|
| `Bootstrap` | `spec.timeout_ms` or `config.bootstrap_timeout_ms` |
| `BaseBackup` | `spec.timeout_ms` or `config.bootstrap_timeout_ms` |
| `PgRewind` | `spec.timeout_ms` or `config.pg_rewind_timeout_ms` |
| `Promote` | `spec.timeout_ms` or `config.bootstrap_timeout_ms` |
| `Demote` | `spec.timeout_ms` or `config.fencing_timeout_ms` |
| `StartPostgres` | `spec.timeout_ms` or `config.bootstrap_timeout_ms` |
| `Fencing` | `spec.timeout_ms` or `config.fencing_timeout_ms` |

### Fencing Preflight

`fencing_preflight_is_already_stopped(data_dir)`:

- Checks `postmaster.pid`
- Returns `true` if pid file absent
- Returns `false` if pid is live
- Removes stale `postmaster.pid` and `postmaster.opts` before returning `true`

| Result | Behavior |
|--------|----------|
| `Ok(true)` | Emits `process.job.fencing_noop`, transitions to idle with `JobOutcome::Success` |
| `Ok(false)` | Continues command execution |
| `Err(error)` | Emits `process.job.fencing_preflight_failed`, transitions to idle with `JobOutcome::Failure` |

### StartPostgres Preflight

`start_postgres_preflight_is_already_running(data_dir)`:

- Checks `postmaster.pid`
- Returns `true` if pid file exists and pid is live
- Removes stale `postmaster.pid` and `postmaster.opts` before returning `false`
- Returns `false` if pid file absent

| Result | Behavior |
|--------|----------|
| `Ok(true)` | Emits `process.job.start_postgres_noop`, transitions to idle with `JobOutcome::Success` |
| `Ok(false)` | Continues command execution |
| `Err(error)` | Emits `process.job.start_postgres_preflight_failed`, transitions to idle with `JobOutcome::Failure` |

## Active Execution and Output Draining

### start_job

When `can_accept_job` returns false:
- Records `last_rejection` with `ProcessError::Busy`
- Emits `process.worker.busy_reject`
- Returns `Ok(())`

After preflight passes:
- Builds command via `build_command`
- Spawns via `ctx.command_runner.spawn`
- Stores `ActiveRuntime`
- Sets state to `Running { worker: WorkerStatus::Running, active }`
- Emits `process.job.started`
- Publishes state

Command-build failure emits `process.job.build_command_failed` and transitions to idle with `JobOutcome::Failure`.

Spawn failure emits `process.job.spawn_failed` and transitions to idle with `JobOutcome::Failure`.

### tick_active_job

Returns immediately if no active runtime.

Drains output up to `PROCESS_OUTPUT_DRAIN_MAX_BYTES`:
- Emits each line as raw subprocess record with `LogProducer::PgTool` and origin `process_worker`
- Emits `process.worker.output_emit_failed` on line emission failure
- Emits `process.worker.output_drain_failed` on drain failure, continues processing

Timeout check:
- If `now >= deadline_at`, emits `process.job.timeout`, cancels handle, drains output
- Transitions to idle with `JobOutcome::Timeout` if cancellation succeeds
- Transitions to idle with `JobOutcome::Failure` if cancellation fails

Exit poll behavior:

| `poll_exit()` Result | Behavior |
|----------------------|----------|
| `Ok(None)` | Restores `active_runtime`, keeps running |
| `Ok(Some(ProcessExit::Success))` | Drains output, emits `process.job.exited`, transitions to idle with `JobOutcome::Success` |
| `Ok(Some(ProcessExit::Failure { code }))` | Drains output, converts to `ProcessError`, emits `process.job.exited`, transitions to idle with `JobOutcome::Failure` |
| `Err(error)` | Drains output, emits `process.job.poll_failed`, transitions to idle with `JobOutcome::Failure` |

### transition_to_idle

Sets `ProcessState::Idle { worker: WorkerStatus::Running, last_outcome: Some(outcome) }` and publishes.

### publish_state

Maps publisher errors to `WorkerError::Message("process publish failed: ...")`.

### system_now_unix_millis

Converts `SystemTime::now()` to `UnixMillis`. Returns `WorkerError::Message` for pre-epoch or conversion failure.
lure` |

**start_postgres_preflight_is_already_running**

- Checks `postmaster.pid`
- Returns `true` when the pid file exists and the pid is live
- Returns `false` when the pid file is absent
- Removes stale `postmaster.pid` and `postmaster.opts` before returning `false`

| Result | Behavior |
|--------|----------|
| `Ok(true)` | Emits `process.job.start_postgres_noop`, transitions directly to idle with `JobOutcome::Success` |
| `Ok(false)` | Continues with command execution |
| `Err(error)` | Emits `process.job.start_postgres_preflight_failed`, transitions to idle with `JobOutcome::Failure` |

### Command mapping

`build_command` validates non-empty required paths and requires absolute program paths.

| Job kind | Binary | Arguments |
|----------|--------|-----------|
| `Bootstrap` | `initdb` | `-D <data_dir> -A trust -U <superuser_username>` |
| `BaseBackup` | `pg_basebackup` | `-h <host> -p <port> -U <user> -D <data_dir> -Fp -Xs` |
| `PgRewind` | `pg_rewind` | `--target-pgdata <target_data_dir> --source-server <rendered conninfo>` |
| `Promote` | `pg_ctl` | `-D <data_dir> promote -w`, plus optional `-t <wait_seconds>` |
| `Demote` | `pg_ctl` | `-D <data_dir> stop -m <mode> -w` |
| `StartPostgres` | `pg_ctl` | `-D <data_dir> -l <log_file> -o <options> start -w -t <wait_seconds>` |
| `Fencing` | `pg_ctl` | `-D <data_dir> stop -m <mode> -w` |

### StartPostgres argument details

- Requires non-empty `data_dir`, `config_file`, and `log_file`
- Defaults `wait_seconds` to `30` when absent
- Renders `-o` from tokens `-c` and `config_file=<path>`

### Environment mapping

| `RoleAuthConfig` | Environment |
|------------------|-------------|
| `Tls` | No env vars |
| `Password` | One secret `PGPASSWORD` env var |

### Spawn behavior

`TokioCommandRunner::spawn`:

- Rejects non-absolute program paths
- Resolves secret environment values using `ProcessEnvValue::resolve_string_for_key`
- Sets stdin to null
- Pipes stdout and stderr when `capture_output` is true
- Discards stdout and stderr when `capture_output` is false
