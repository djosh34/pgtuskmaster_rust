use std::process::Stdio;

use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
    sync::mpsc,
    sync::mpsc::error::TryRecvError,
};

use crate::{
    config_v2::RuntimeConfigV2,
    logging::LogSender,
    process::{
        cluster::{prepare_process_launch_from_ctx, ProcessPreparationError},
        postmaster::{start_postgres_preflight, StartPostgresPreflight},
    },
    state::{new_state_channel, StateSubscriber, UnixMillis, WorkerError, WorkerStatus},
};

use super::{
    jobs::{
        ActiveJob, ProcessCommandSpec, ProcessError, ProcessExit, ProcessHandle, ProcessIntent,
        ProcessJobKind, ProcessOutputLine, ProcessOutputStream,
    },
    log_event::{CapturedStream, ProcessLogEvent, SubprocessLogEvent},
    state::{
        ActiveRuntime, JobOutcome, ProcessCadence, ProcessControlPlane, ProcessIntentRequest,
        ProcessJobRejection, ProcessObservedState, ProcessRuntime, ProcessState,
        ProcessStateChannel, ProcessWorkerCtx,
    },
};

const PROCESS_OUTPUT_READ_CHUNK_BYTES: usize = 8192;
const PROCESS_OUTPUT_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1);
const PROCESS_OUTPUT_DRAIN_MAX_BYTES: usize = 256 * 1024;
#[derive(Default)]
pub(crate) struct TokioCommandRunner;

pub(crate) fn bootstrap<'a>(
    cfg: &'a RuntimeConfigV2,
    observed: ProcessObservedState,
    log: LogSender,
) -> (
    ProcessWorkerCtx<'a>,
    StateSubscriber<ProcessState>,
    tokio::sync::mpsc::UnboundedSender<ProcessIntentRequest>,
) {
    bootstrap_with_runtime(
        cfg,
        observed,
        ProcessCadence {
            poll_interval: std::time::Duration::from_millis(10),
            now: Box::new(system_now_unix_millis),
        },
        ProcessRuntime {
            log,
            command_runner: Box::new(TokioCommandRunner),
        },
    )
}

pub(crate) fn bootstrap_with_runtime<'a>(
    cfg: &'a RuntimeConfigV2,
    observed: ProcessObservedState,
    cadence: ProcessCadence,
    runtime: ProcessRuntime,
) -> (
    ProcessWorkerCtx<'a>,
    StateSubscriber<ProcessState>,
    tokio::sync::mpsc::UnboundedSender<ProcessIntentRequest>,
) {
    let initial_state = ProcessState::starting();
    let (publisher, state) = new_state_channel(initial_state.clone());
    let (intents, inbox) = mpsc::unbounded_channel();

    (
        ProcessWorkerCtx {
            cfg,
            cadence,
            observed,
            state_channel: ProcessStateChannel {
                current: initial_state,
                publisher,
                last_rejection: None,
            },
            control: ProcessControlPlane {
                inbox,
                inbox_disconnected_logged: false,
                active_runtime: None,
            },
            runtime,
        },
        state,
        intents,
    )
}

struct TokioProcessHandle {
    child: Child,
    stdout: Option<tokio::process::ChildStdout>,
    stderr: Option<tokio::process::ChildStderr>,
    stdout_pending: Vec<u8>,
    stderr_pending: Vec<u8>,
    stdout_eof: bool,
    stderr_eof: bool,
}

impl ProcessHandle for TokioProcessHandle {
    fn poll_exit(&mut self) -> Result<Option<ProcessExit>, ProcessError> {
        let status = self
            .child
            .try_wait()
            .map_err(|err| ProcessError::SpawnFailure {
                binary: "process-child".to_string(),
                message: err.to_string(),
            })?;

        Ok(status.map(|exit| {
            if exit.success() {
                ProcessExit::Success
            } else {
                ProcessExit::Failure { code: exit.code() }
            }
        }))
    }

    fn drain_output<'a>(
        &'a mut self,
        max_bytes: usize,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<Vec<super::jobs::ProcessOutputLine>, ProcessError>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            if max_bytes == 0 {
                return Ok(Vec::new());
            }

            let mut out = Vec::new();
            let mut remaining = max_bytes;
            drain_one_stream(
                &mut out,
                &mut remaining,
                super::jobs::ProcessOutputStream::Stdout,
                &mut self.stdout,
                &mut self.stdout_pending,
                &mut self.stdout_eof,
            )
            .await;
            drain_one_stream(
                &mut out,
                &mut remaining,
                super::jobs::ProcessOutputStream::Stderr,
                &mut self.stderr,
                &mut self.stderr_pending,
                &mut self.stderr_eof,
            )
            .await;
            Ok(out)
        })
    }

    fn cancel<'a>(
        &'a mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ProcessError>> + Send + 'a>>
    {
        Box::pin(async move {
            if self
                .child
                .try_wait()
                .map_err(|err| ProcessError::CancelFailure(err.to_string()))?
                .is_some()
            {
                return Ok(());
            }

            self.child
                .start_kill()
                .map_err(|err| ProcessError::CancelFailure(err.to_string()))?;
            self.child
                .wait()
                .await
                .map_err(|err| ProcessError::CancelFailure(err.to_string()))?;
            Ok(())
        })
    }
}

impl super::jobs::ProcessCommandRunner for TokioCommandRunner {
    fn spawn(&mut self, spec: ProcessCommandSpec) -> Result<Box<dyn ProcessHandle>, ProcessError> {
        let ProcessCommandSpec {
            program,
            args,
            env,
            capture_output,
        } = spec;
        let binary = program.display().to_string();
        if !program.is_absolute() {
            return Err(ProcessError::InvalidSpec(format!(
                "program must be an absolute path, got `{}`",
                program.display()
            )));
        }

        let mut command = Command::new(&program);
        command.args(args).stdin(Stdio::null());
        for var in env {
            let value = var.value.resolve_string_for_key(var.key.as_str())?;
            command.env(var.key, value);
        }
        if capture_output {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        } else {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }

        let mut child = command.spawn().map_err(|err| ProcessError::SpawnFailure {
            binary,
            message: err.to_string(),
        })?;

        let stdout = if capture_output {
            child.stdout.take()
        } else {
            None
        };
        let stderr = if capture_output {
            child.stderr.take()
        } else {
            None
        };

        Ok(Box::new(TokioProcessHandle {
            child,
            stdout,
            stderr,
            stdout_pending: Vec::new(),
            stderr_pending: Vec::new(),
            stdout_eof: false,
            stderr_eof: false,
        }))
    }
}

async fn drain_one_stream(
    out: &mut Vec<super::jobs::ProcessOutputLine>,
    remaining: &mut usize,
    stream: super::jobs::ProcessOutputStream,
    handle: &mut Option<impl AsyncRead + Unpin>,
    pending: &mut Vec<u8>,
    eof: &mut bool,
) {
    if *remaining == 0 || *eof {
        return;
    }
    let Some(handle) = handle.as_mut() else {
        *eof = true;
        return;
    };

    let mut buf = vec![0u8; PROCESS_OUTPUT_READ_CHUNK_BYTES];
    loop {
        if *remaining == 0 {
            break;
        }
        let chunk_len = buf.len().min(*remaining);
        let read_result = tokio::time::timeout(
            PROCESS_OUTPUT_READ_TIMEOUT,
            handle.read(&mut buf[..chunk_len]),
        )
        .await;
        let read_outcome = match read_result {
            Ok(Ok(n)) => Ok(n),
            Ok(Err(err)) => Err(err),
            Err(_) => {
                // No data quickly available.
                break;
            }
        };

        match read_outcome {
            Ok(0) => {
                *eof = true;
                if !pending.is_empty() {
                    out.push(super::jobs::ProcessOutputLine {
                        stream,
                        bytes: std::mem::take(pending),
                    });
                }
                break;
            }
            Ok(n) => {
                pending.extend_from_slice(&buf[..n]);
                *remaining = remaining.saturating_sub(n);
                while let Some(pos) = pending.iter().position(|b| *b == b'\n') {
                    let mut line = pending.drain(..=pos).collect::<Vec<u8>>();
                    if let Some(b'\n') = line.last() {
                        line.pop();
                    }
                    if let Some(b'\r') = line.last() {
                        line.pop();
                    }
                    out.push(super::jobs::ProcessOutputLine {
                        stream,
                        bytes: line,
                    });
                }
            }
            Err(err) => {
                *eof = true;
                out.push(super::jobs::ProcessOutputLine {
                    stream,
                    bytes: format!("stdio read error: {err}").into_bytes(),
                });
                break;
            }
        }
    }
}

fn can_accept_job(state: &ProcessState) -> bool {
    matches!(state, ProcessState::Idle { .. })
}

pub(crate) async fn run(mut ctx: ProcessWorkerCtx<'_>) -> Result<(), WorkerError> {
    ctx.runtime
        .log
        .send(ProcessLogEvent::WorkerRunStarted {
            capture_subprocess_output: ctx.cfg.logging.capture_subprocess_output,
        })
        .map_err(|err| {
            WorkerError::Message(format!("process worker start log send failed: {err}"))
        })?;
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.cadence.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut ProcessWorkerCtx<'_>) -> Result<(), WorkerError> {
    let mut request = None;
    let mut inbox_disconnected = false;
    loop {
        match ctx.control.inbox.try_recv() {
            Ok(next_request) => request = Some(next_request),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                inbox_disconnected = true;
                break;
            }
        }
    }
    if inbox_disconnected && !ctx.control.inbox_disconnected_logged {
        ctx.control.inbox_disconnected_logged = true;
        ctx.runtime
            .log
            .send(ProcessLogEvent::InboxDisconnected)
            .map_err(|err| {
                WorkerError::Message(format!("process inbox disconnected log send failed: {err}"))
            })?;
    }

    if let Some(request) = request {
        ctx.runtime
            .log
            .send(ProcessLogEvent::RequestReceived {
                job_kind: request.intent.job_kind(),
            })
            .map_err(|err| {
                WorkerError::Message(format!("process request log send failed: {err}"))
            })?;
        start_job(ctx, request).await?;
    }

    tick_active_job(ctx).await
}

pub(crate) async fn start_job(
    ctx: &mut ProcessWorkerCtx<'_>,
    request: ProcessIntentRequest,
) -> Result<(), WorkerError> {
    if !can_accept_job(&ctx.state_channel.current) {
        let now = current_time(ctx)?;
        let rejected_job_id = request.id.clone();
        ctx.state_channel.last_rejection = Some(ProcessJobRejection {
            id: rejected_job_id.clone(),
            error: ProcessError::Busy,
            rejected_at: now,
        });
        ctx.runtime
            .log
            .send(ProcessLogEvent::BusyRejected {
                job_kind: request.intent.job_kind(),
            })
            .map_err(|err| {
                WorkerError::Message(format!("process busy reject log send failed: {err}"))
            })?;
        return Ok(());
    }

    let now = current_time(ctx)?;
    if matches!(&request.intent, ProcessIntent::Start(_)) {
        match start_postgres_preflight(
            ctx.cfg.postgres.data_dir.as_path(),
            ctx.cfg.postgres.socket_dir.as_path(),
            ctx.cfg.postgres.listen_port,
        ) {
            Ok(StartPostgresPreflight::AlreadyRunning) => {
                ctx.runtime
                    .log
                    .send(ProcessLogEvent::StartPostgresAlreadyRunning {
                        data_dir: ctx.cfg.postgres.data_dir.display().to_string(),
                    })
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process start-postgres noop log send failed: {err}"
                        ))
                    })?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Success {
                        id: request.id,
                        job_kind: request.intent.job_kind(),
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
            Ok(StartPostgresPreflight::SafeToStart) => {}
            Err(error) => {
                let cause = format!("start postgres preflight failed: {error}");
                ctx.runtime
                    .log
                    .send(ProcessLogEvent::StartPostgresPreflightFailed {
                        cause: cause.clone(),
                    })
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process start-postgres preflight log send failed: {err}"
                        ))
                    })?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Failure {
                        id: request.id,
                        job_kind: request.intent.job_kind(),
                        error: ProcessError::InvalidSpec(cause),
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
        }
    }

    let prepared_launch = match prepare_process_launch_from_ctx(ctx, &request) {
        Ok(prepared) => prepared,
        Err(error) => {
            log_prepare_failure(ctx, &request, &error)?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: request.intent.job_kind(),
                    error: error.into_process_error(),
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };
    let active = ActiveJob {
        id: request.id.clone(),
        kind: request.intent.job_kind(),
        started_at: now,
        deadline_at: UnixMillis(now.0.saturating_add(request.intent.timeout_ms(ctx.cfg))),
    };
    let command = prepared_launch;
    let handle = match ctx.runtime.command_runner.spawn(command) {
        Ok(handle) => handle,
        Err(error) => {
            ctx.runtime
                .log
                .send(ProcessLogEvent::SpawnFailed {
                    job_kind: active.kind,
                    cause: error.to_string(),
                })
                .map_err(|err| {
                    WorkerError::Message(format!("process spawn log send failed: {err}"))
                })?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: active.kind,
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };

    ctx.control.active_runtime = Some(ActiveRuntime { handle });
    ctx.state_channel.current = ProcessState::Running {
        worker: WorkerStatus::Running,
        active,
    };
    ctx.runtime
        .log
        .send(ProcessLogEvent::Started {
            job_kind: request.intent.job_kind(),
        })
        .map_err(|err| {
            WorkerError::Message(format!("process job started log send failed: {err}"))
        })?;
    publish_state(ctx)
}

pub(crate) async fn tick_active_job(ctx: &mut ProcessWorkerCtx<'_>) -> Result<(), WorkerError> {
    let ActiveRuntime { mut handle } = match ctx.control.active_runtime.take() {
        Some(runtime) => runtime,
        None => return Ok(()),
    };
    let active = ctx.state_channel.current.active().cloned().ok_or_else(|| {
        WorkerError::Message(
            "process runtime exists without a running active job in state".to_string(),
        )
    })?;
    let job_kind = active.kind;

    let now = current_time(ctx)?;
    flush_subprocess_output(ctx, handle.as_mut(), job_kind).await?;
    if now.0 >= active.deadline_at.0 {
        ctx.runtime
            .log
            .send(ProcessLogEvent::Timeout { job_kind })
            .map_err(|err| {
                WorkerError::Message(format!("process timeout log send failed: {err}"))
            })?;
        let cancel_result = handle.cancel().await;
        flush_subprocess_output(ctx, handle.as_mut(), job_kind).await?;
        let outcome = match cancel_result {
            Ok(()) => JobOutcome::Timeout {
                id: active.id.clone(),
                job_kind: active.kind,
                finished_at: now,
            },
            Err(error) => JobOutcome::Failure {
                id: active.id.clone(),
                job_kind: active.kind,
                error,
                finished_at: now,
            },
        };
        transition_to_idle(ctx, outcome, now)?;
        return Ok(());
    }

    let poll = handle.poll_exit();
    match poll {
        Ok(None) => {
            ctx.control.active_runtime = Some(ActiveRuntime { handle });
            Ok(())
        }
        Ok(Some(ProcessExit::Success)) => {
            flush_subprocess_output(ctx, handle.as_mut(), job_kind).await?;
            let outcome = JobOutcome::Success {
                id: active.id.clone(),
                job_kind: active.kind,
                finished_at: now,
            };
            ctx.runtime
                .log
                .send(ProcessLogEvent::ExitedSuccessfully { job_kind })
                .map_err(|err| {
                    WorkerError::Message(format!("process exit log send failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
        Ok(Some(exit)) => {
            flush_subprocess_output(ctx, handle.as_mut(), job_kind).await?;
            let exit_error = ProcessError::from_exit(exit);
            let outcome = JobOutcome::Failure {
                id: active.id.clone(),
                job_kind: active.kind,
                error: exit_error.clone(),
                finished_at: now,
            };
            ctx.runtime
                .log
                .send(ProcessLogEvent::ExitedUnsuccessfully {
                    job_kind,
                    cause: exit_error.to_string(),
                })
                .map_err(|err| {
                    WorkerError::Message(format!("process exit log send failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
        Err(error) => {
            flush_subprocess_output(ctx, handle.as_mut(), job_kind).await?;
            let outcome = JobOutcome::Failure {
                id: active.id,
                job_kind: active.kind,
                error,
                finished_at: now,
            };
            ctx.runtime
                .log
                .send(ProcessLogEvent::PollFailed {
                    job_kind,
                    cause: outcome_error_string(&outcome),
                })
                .map_err(|err| {
                    WorkerError::Message(format!("process poll failure log send failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
    }
}

fn outcome_error_string(outcome: &JobOutcome) -> String {
    match outcome {
        JobOutcome::Success { .. } => "success".to_string(),
        JobOutcome::Timeout { .. } => "timeout".to_string(),
        JobOutcome::Failure { error, .. } => error.to_string(),
    }
}

fn captured_stream(stream: ProcessOutputStream) -> CapturedStream {
    match stream {
        ProcessOutputStream::Stdout => CapturedStream::Stdout,
        ProcessOutputStream::Stderr => CapturedStream::Stderr,
    }
}

async fn flush_subprocess_output(
    ctx: &mut ProcessWorkerCtx<'_>,
    handle: &mut dyn ProcessHandle,
    job_kind: ProcessJobKind,
) -> Result<(), WorkerError> {
    match handle.drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES).await {
        Ok(lines) => emit_subprocess_output(ctx, job_kind, lines),
        Err(err) => ctx
            .runtime
            .log
            .send(ProcessLogEvent::OutputDrainFailed {
                job_kind,
                cause: err.to_string(),
            })
            .map_err(|send_err| {
                WorkerError::Message(format!("process output drain log send failed: {send_err}"))
            }),
    }
}

fn emit_subprocess_output(
    ctx: &mut ProcessWorkerCtx<'_>,
    job_kind: ProcessJobKind,
    lines: Vec<ProcessOutputLine>,
) -> Result<(), WorkerError> {
    for line in lines {
        if let Err(err) = ctx
            .runtime
            .log
            .send(subprocess_log_event(job_kind, line.clone()))
        {
            ctx.runtime
                .log
                .send(ProcessLogEvent::OutputEmitFailed {
                    job_kind,
                    stream: captured_stream(line.stream),
                    cause: err.to_string(),
                })
                .map_err(|send_err| {
                    WorkerError::Message(format!(
                        "process output emit failure log send failed: {send_err}"
                    ))
                })?;
        }
    }

    Ok(())
}

fn subprocess_log_event(job_kind: ProcessJobKind, line: ProcessOutputLine) -> SubprocessLogEvent {
    SubprocessLogEvent::Line {
        job_kind,
        stream: captured_stream(line.stream),
        line: match String::from_utf8(line.bytes) {
            Ok(line) => line,
            Err(err) => String::from_utf8_lossy(err.as_bytes()).into_owned(),
        },
    }
}

fn transition_to_idle(
    ctx: &mut ProcessWorkerCtx<'_>,
    outcome: JobOutcome,
    _now: UnixMillis,
) -> Result<(), WorkerError> {
    ctx.state_channel.current = ProcessState::Idle {
        worker: WorkerStatus::Running,
        last_outcome: Some(outcome),
    };
    publish_state(ctx)
}

fn publish_state(ctx: &mut ProcessWorkerCtx<'_>) -> Result<(), WorkerError> {
    ctx.state_channel
        .publisher
        .publish(ctx.state_channel.current.clone())
        .map_err(|err| WorkerError::Message(format!("process publish failed: {err}")))?;
    Ok(())
}

fn current_time(ctx: &mut ProcessWorkerCtx<'_>) -> Result<UnixMillis, WorkerError> {
    (ctx.cadence.now)()
}

pub(crate) fn system_now_unix_millis() -> Result<UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

fn log_prepare_failure(
    ctx: &mut ProcessWorkerCtx<'_>,
    request: &ProcessIntentRequest,
    error: &ProcessPreparationError,
) -> Result<(), WorkerError> {
    match error {
        ProcessPreparationError::Snapshot(inner)
        | ProcessPreparationError::IntentMaterialization(inner) => ctx
            .runtime
            .log
            .send(ProcessLogEvent::IntentMaterializationFailed {
                job_kind: request.intent.job_kind(),
                cause: inner.to_string(),
            })
            .map_err(|err| {
                WorkerError::Message(format!(
                    "process intent materialization log send failed: {err}"
                ))
            }),
        ProcessPreparationError::BuildCommand(inner) => ctx
            .runtime
            .log
            .send(ProcessLogEvent::BuildCommandFailed {
                job_kind: request.intent.job_kind(),
                cause: inner.to_string(),
            })
            .map_err(|err| {
                WorkerError::Message(format!("process build command log send failed: {err}"))
            }),
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, time::Duration};

    use tokio::sync::mpsc::UnboundedSender;

    use crate::{
        config_v2::types::PostgresConfig,
        config_v2::{managed_postgres_test_config, RuntimeConfigV2},
        dcs::DcsSnapshot,
        dev_support::test_fs::unique_test_dir,
        logging::LogSender,
        postgres_managed::{managed_standby_passfile_path, MANAGED_POSTGRESQL_CONF_NAME},
        process::{
            jobs::{
                ActiveJob, PostgresStartIntent, ProcessCommandRunner, ProcessCommandSpec,
                ProcessIntent, ProcessJobKind, ReplicaProvisionIntent,
            },
            state::{
                ProcessCadence, ProcessIntentRequest, ProcessObservedState, ProcessRuntime,
                ProcessState, ProcessWorkerCtx,
            },
            test_support::{
                spawn_fake_managed_postmaster, wait_for_lookup_ready, write_postmaster_pid,
            },
        },
        state::{new_state_channel, JobId, MemberId, StateSubscriber, UnixMillis, WorkerStatus},
    };

    use super::{bootstrap_with_runtime, start_job, step_once};

    struct UnexpectedSpawnRunner;

    impl ProcessCommandRunner for UnexpectedSpawnRunner {
        fn spawn(
            &mut self,
            _spec: ProcessCommandSpec,
        ) -> Result<Box<dyn crate::process::jobs::ProcessHandle>, crate::process::jobs::ProcessError>
        {
            Err(crate::process::jobs::ProcessError::SpawnFailure {
                binary: "unexpected-spawn".to_string(),
                message: "spawn should not be called for start-postgres noop".to_string(),
            })
        }
    }

    fn build_test_ctx(
        data_dir: PathBuf,
        socket_dir: PathBuf,
        log_file: PathBuf,
    ) -> Result<
        (
            ProcessWorkerCtx<'static>,
            StateSubscriber<ProcessState>,
            UnboundedSender<ProcessIntentRequest>,
        ),
        String,
    > {
        let config =
            managed_postgres_test_config(data_dir.clone()).map_err(|err| err.to_string())?;
        let cfg = RuntimeConfigV2 {
            postgres: PostgresConfig {
                socket_dir: socket_dir.clone(),
                log_file: log_file.clone(),
                ..config.postgres
            },
            ..config
        };
        let cfg = Box::leak(Box::new(cfg));
        let (_dcs_publisher, dcs_subscriber) = new_state_channel(DcsSnapshot::starting());
        Ok(bootstrap_with_runtime(
            cfg,
            ProcessObservedState {
                dcs: dcs_subscriber,
            },
            ProcessCadence {
                poll_interval: Duration::from_millis(10),
                now: Box::new(super::system_now_unix_millis),
            },
            ProcessRuntime {
                log: LogSender::disabled(),
                command_runner: Box::new(UnexpectedSpawnRunner),
            },
        ))
    }

    #[tokio::test]
    async fn start_postgres_noop_preserves_existing_standby_passfile() -> Result<(), String> {
        let root = unique_test_dir("process-worker", "noop-passfile")?;
        let data_dir = root.join("data");
        let socket_dir = root.join("socket");
        let log_file = root.join("logs/postgres.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        fs::create_dir_all(&socket_dir)
            .map_err(|err| format!("create socket dir {} failed: {err}", socket_dir.display()))?;

        let passfile_path = managed_standby_passfile_path(&data_dir);
        let original_passfile = "node-b:5432:replication:replicator:secret-password\n";
        fs::write(&passfile_path, original_passfile).map_err(|err| {
            format!(
                "write standby passfile {} failed: {err}",
                passfile_path.display()
            )
        })?;

        let fake_postgres = spawn_fake_managed_postmaster(&root, &data_dir, None)?;
        let fake_postgres_pid = fake_postgres.pid();
        write_postmaster_pid(&data_dir, fake_postgres_pid, &data_dir)?;
        wait_for_lookup_ready(
            &crate::process::postmaster::ManagedPostmasterTarget::from_data_dir(data_dir.clone()),
        )?;

        let _fake_postgres = fake_postgres;
        let (mut ctx, _state_subscriber, _tx) =
            build_test_ctx(data_dir.clone(), socket_dir, log_file)?;
        let request = ProcessIntentRequest {
            id: JobId("job-start-detached-standby-noop".to_string()),
            intent: ProcessIntent::Start(PostgresStartIntent::DetachedStandby),
        };

        start_job(&mut ctx, request.clone())
            .await
            .map_err(|err| format!("start_job failed: {err}"))?;

        match &ctx.state_channel.current {
            ProcessState::Idle {
                last_outcome: Some(crate::process::state::JobOutcome::Success { id, job_kind, .. }),
                ..
            } => {
                if *id != request.id {
                    return Err(format!(
                        "unexpected job id after noop: expected={} actual={}",
                        request.id.0, id.0
                    ));
                }
                if *job_kind != crate::process::jobs::ProcessJobKind::StartDetachedStandby {
                    return Err(format!(
                        "unexpected job kind after noop: expected={:?} actual={job_kind:?}",
                        crate::process::jobs::ProcessJobKind::StartDetachedStandby
                    ));
                }
            }
            other => {
                return Err(format!(
                    "expected idle success after start noop, observed {other:?}"
                ));
            }
        }

        let preserved = fs::read_to_string(&passfile_path).map_err(|err| {
            format!(
                "read standby passfile {} failed: {err}",
                passfile_path.display()
            )
        })?;
        if preserved != original_passfile {
            return Err(format!(
                "standby passfile changed during noop: expected={original_passfile:?} actual={preserved:?}"
            ));
        }

        let managed_conf = data_dir.join(MANAGED_POSTGRESQL_CONF_NAME);
        if managed_conf.exists() {
            return Err(format!(
                "managed postgres conf should not be materialized for noop start at {}",
                managed_conf.display()
            ));
        }

        Ok(())
    }

    #[tokio::test]
    async fn step_once_rejects_only_latest_queued_request_when_busy() -> Result<(), String> {
        let root = unique_test_dir("process-worker", "busy-drain-latest")?;
        let data_dir = root.join("data");
        let socket_dir = root.join("socket");
        let log_file = root.join("logs/postgres.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        fs::create_dir_all(&socket_dir)
            .map_err(|err| format!("create socket dir {} failed: {err}", socket_dir.display()))?;

        let (mut ctx, _state_subscriber, tx) = build_test_ctx(data_dir, socket_dir, log_file)?;
        ctx.state_channel.current = ProcessState::Running {
            worker: WorkerStatus::Running,
            active: ActiveJob {
                id: JobId("active-job".to_string()),
                kind: ProcessJobKind::Bootstrap,
                started_at: UnixMillis(10),
                deadline_at: UnixMillis(20),
            },
        };

        let first = ProcessIntentRequest {
            id: JobId("job-first".to_string()),
            intent: ProcessIntent::Start(PostgresStartIntent::Primary),
        };
        let second = ProcessIntentRequest {
            id: JobId("job-second".to_string()),
            intent: ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup {
                leader: MemberId("node-a".to_string()),
            }),
        };

        assert!(tx.send(first).is_ok(), "failed to send first request");
        assert!(
            tx.send(second.clone()).is_ok(),
            "failed to send second request"
        );

        step_once(&mut ctx)
            .await
            .map_err(|err| format!("step_once failed: {err}"))?;

        assert_eq!(
            ctx.state_channel
                .last_rejection
                .as_ref()
                .map(|rejection| &rejection.id),
            Some(&second.id)
        );
        Ok(())
    }
}
