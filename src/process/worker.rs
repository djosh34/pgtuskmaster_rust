use std::{fs, path::Path, process::Stdio};

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
        postmaster::{lookup_managed_postmaster, ManagedPostmasterError, ManagedPostmasterTarget},
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
        ActiveRuntime, JobOutcome, ProcessCadence, ProcessControlPlane, ProcessExecutionKind,
        ProcessIntentRequest, ProcessJobRejection, ProcessObservedState, ProcessRuntime,
        ProcessState, ProcessStateChannel, ProcessWorkerCtx,
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
            job_kind: _,
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
                job_kind: request.intent.process_job_kind(),
            })
            .map_err(|err| {
                WorkerError::Message(format!("process request log send failed: {err}"))
            })?;
        start_job(ctx, request).await?;
    }

    tick_active_job(ctx).await
}

fn pid_is_postgres_process(pid: u32) -> Result<bool, ProcessError> {
    #[cfg(unix)]
    {
        let cmdline_path = std::path::PathBuf::from(format!("/proc/{pid}/cmdline"));
        let cmdline = match fs::read(&cmdline_path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(err) => {
                return Err(ProcessError::InvalidSpec(format!(
                    "read {} failed: {err}",
                    cmdline_path.display()
                )));
            }
        };
        Ok(cmdline
            .split(|byte| *byte == 0)
            .filter(|arg| !arg.is_empty())
            .map(|arg| String::from_utf8_lossy(arg))
            .any(|arg| {
                std::path::Path::new(arg.as_ref())
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| matches!(name, "postgres" | "postmaster"))
                    .unwrap_or(false)
            }))
    }
    #[cfg(not(unix))]
    {
        Ok(true)
    }
}

fn remove_file_best_effort(path: &Path) -> Result<(), ProcessError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(ProcessError::InvalidSpec(format!(
            "remove file {} failed: {err}",
            path.display()
        ))),
    }
}

fn postgres_socket_paths(socket_dir: &Path, port: u16) -> (std::path::PathBuf, std::path::PathBuf) {
    let socket_file = socket_dir.join(format!(".s.PGSQL.{port}"));
    let lock_file = socket_dir.join(format!(".s.PGSQL.{port}.lock"));
    (socket_file, lock_file)
}

fn parse_postgres_socket_lock_pid(lock_file: &Path) -> Result<Option<u32>, ProcessError> {
    let contents = match fs::read_to_string(lock_file) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => {
            return Err(ProcessError::InvalidSpec(format!(
                "read postgres socket lock {} failed: {err}",
                lock_file.display()
            )));
        }
    };
    let Some(first_line) = contents.lines().next() else {
        return Ok(None);
    };
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed.parse::<u32>().map(Some).map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "parse postgres socket lock pid '{}' in {} failed: {err}",
            trimmed,
            lock_file.display()
        ))
    })
}

fn cleanup_postgres_socket_files(socket_dir: &Path, port: u16) -> Result<(), ProcessError> {
    let (socket_file, lock_file) = postgres_socket_paths(socket_dir, port);
    remove_file_best_effort(&socket_file)?;
    remove_file_best_effort(&lock_file)?;
    Ok(())
}

fn start_postgres_preflight_is_already_running(
    data_dir: &Path,
    socket_dir: &Path,
    port: u16,
) -> Result<bool, ProcessError> {
    let pid_file = data_dir.join("postmaster.pid");
    if pid_file.exists() {
        let target = ManagedPostmasterTarget::from_data_dir(data_dir.to_path_buf());
        match lookup_managed_postmaster(&target) {
            Ok(_postmaster) => return Ok(true),
            Err(
                ManagedPostmasterError::MissingPidFile { .. }
                | ManagedPostmasterError::PidNotRunning { .. }
                | ManagedPostmasterError::DataDirMismatch { .. },
            ) => {
                remove_file_best_effort(&pid_file)?;
                let opts_file = data_dir.join("postmaster.opts");
                remove_file_best_effort(&opts_file)?;
            }
            Err(err) => {
                return Err(ProcessError::InvalidSpec(format!(
                    "start postgres preflight managed postmaster lookup failed: {err}"
                )));
            }
        }
    }

    let (_, lock_file) = postgres_socket_paths(socket_dir, port);
    if let Some(pid) = parse_postgres_socket_lock_pid(&lock_file)? {
        if pid_is_postgres_process(pid)? {
            return Ok(true);
        }
    }

    cleanup_postgres_socket_files(socket_dir, port)?;
    Ok(false)
}

fn start_postgres_preflight_details(
    ctx: &ProcessWorkerCtx<'_>,
    intent: &ProcessIntent,
) -> Option<(std::path::PathBuf, std::path::PathBuf, u16)> {
    match intent {
        ProcessIntent::Start(_) => Some((
            ctx.cfg.postgres.data_dir.clone(),
            ctx.cfg.postgres.socket_dir.clone(),
            ctx.cfg.postgres.listen_port,
        )),
        _ => None,
    }
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
                job_kind: request.intent.process_job_kind(),
            })
            .map_err(|err| {
                WorkerError::Message(format!("process busy reject log send failed: {err}"))
            })?;
        return Ok(());
    }

    let now = current_time(ctx)?;
    if let Some((data_dir, socket_dir, port)) =
        start_postgres_preflight_details(ctx, &request.intent)
    {
        match start_postgres_preflight_is_already_running(
            data_dir.as_path(),
            socket_dir.as_path(),
            port,
        ) {
            Ok(true) => {
                ctx.runtime
                    .log
                    .send(ProcessLogEvent::StartPostgresAlreadyRunning {
                        data_dir: data_dir.display().to_string(),
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
                        job_kind: request.intent.active_job_kind(),
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
            Ok(false) => {}
            Err(error) => {
                ctx.runtime
                    .log
                    .send(ProcessLogEvent::StartPostgresPreflightFailed {
                        cause: error.to_string(),
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
                        job_kind: request.intent.active_job_kind(),
                        error,
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
                    job_kind: request.intent.active_job_kind(),
                    error: error.into_process_error(),
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };
    let execution_request = prepared_launch.request;
    let timeout_ms = timeout_for_kind(ctx, &execution_request.kind);
    let deadline_at = UnixMillis(now.0.saturating_add(timeout_ms));
    let command = prepared_launch.command;

    let job_kind = command.job_kind;
    let handle = match ctx.runtime.command_runner.spawn(command) {
        Ok(handle) => handle,
        Err(error) => {
            ctx.runtime
                .log
                .send(ProcessLogEvent::SpawnFailed {
                    job_kind: execution_request.kind.process_job_kind(),
                    cause: error.to_string(),
                })
                .map_err(|err| {
                    WorkerError::Message(format!("process spawn log send failed: {err}"))
                })?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: execution_request.kind.active_job_kind(),
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };

    let active = ActiveJob {
        id: request.id.clone(),
        kind: execution_request.kind.active_job_kind(),
        started_at: now,
        deadline_at,
    };
    ctx.control.active_runtime = Some(ActiveRuntime {
        request: execution_request,
        deadline_at,
        handle,
        job_kind,
    });
    ctx.state_channel.current = ProcessState::Running {
        worker: WorkerStatus::Running,
        active,
    };
    ctx.runtime
        .log
        .send(ProcessLogEvent::Started { job_kind })
        .map_err(|err| {
            WorkerError::Message(format!("process job started log send failed: {err}"))
        })?;
    publish_state(ctx)
}

pub(crate) async fn tick_active_job(ctx: &mut ProcessWorkerCtx<'_>) -> Result<(), WorkerError> {
    let mut runtime = match ctx.control.active_runtime.take() {
        Some(runtime) => runtime,
        None => return Ok(()),
    };

    let now = current_time(ctx)?;
    flush_subprocess_output(ctx, runtime.handle.as_mut(), runtime.job_kind).await?;
    if now.0 >= runtime.deadline_at.0 {
        ctx.runtime
            .log
            .send(ProcessLogEvent::Timeout {
                job_kind: runtime.job_kind,
            })
            .map_err(|err| {
                WorkerError::Message(format!("process timeout log send failed: {err}"))
            })?;
        let cancel_result = runtime.handle.cancel().await;
        flush_subprocess_output(ctx, runtime.handle.as_mut(), runtime.job_kind).await?;
        let outcome = match cancel_result {
            Ok(()) => JobOutcome::Timeout {
                id: runtime.request.id,
                job_kind: runtime.request.kind.active_job_kind(),
                finished_at: now,
            },
            Err(error) => JobOutcome::Failure {
                id: runtime.request.id,
                job_kind: runtime.request.kind.active_job_kind(),
                error,
                finished_at: now,
            },
        };
        transition_to_idle(ctx, outcome, now)?;
        return Ok(());
    }

    let poll = runtime.handle.poll_exit();
    match poll {
        Ok(None) => {
            ctx.control.active_runtime = Some(runtime);
            Ok(())
        }
        Ok(Some(ProcessExit::Success)) => {
            flush_subprocess_output(ctx, runtime.handle.as_mut(), runtime.job_kind).await?;
            let job_id = runtime.request.id.clone();
            let outcome = JobOutcome::Success {
                id: job_id,
                job_kind: runtime.request.kind.active_job_kind(),
                finished_at: now,
            };
            ctx.runtime
                .log
                .send(ProcessLogEvent::ExitedSuccessfully {
                    job_kind: runtime.job_kind,
                })
                .map_err(|err| {
                    WorkerError::Message(format!("process exit log send failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
        Ok(Some(exit)) => {
            flush_subprocess_output(ctx, runtime.handle.as_mut(), runtime.job_kind).await?;
            let exit_error = ProcessError::from_exit(exit);
            let outcome = JobOutcome::Failure {
                id: runtime.request.id.clone(),
                job_kind: runtime.request.kind.active_job_kind(),
                error: exit_error.clone(),
                finished_at: now,
            };
            ctx.runtime
                .log
                .send(ProcessLogEvent::ExitedUnsuccessfully {
                    job_kind: runtime.job_kind,
                    cause: exit_error.to_string(),
                })
                .map_err(|err| {
                    WorkerError::Message(format!("process exit log send failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
        Err(error) => {
            flush_subprocess_output(ctx, runtime.handle.as_mut(), runtime.job_kind).await?;
            let outcome = JobOutcome::Failure {
                id: runtime.request.id.clone(),
                job_kind: runtime.request.kind.active_job_kind(),
                error,
                finished_at: now,
            };
            ctx.runtime
                .log
                .send(ProcessLogEvent::PollFailed {
                    job_kind: runtime.job_kind,
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

fn timeout_for_kind(ctx: &ProcessWorkerCtx<'_>, kind: &ProcessExecutionKind) -> u64 {
    match kind {
        ProcessExecutionKind::Bootstrap(spec) => spec
            .timeout_ms
            .unwrap_or(duration_millis_u64(ctx.cfg.timing.bootstrap_timeout)),
        ProcessExecutionKind::BaseBackup(spec) => spec
            .timeout_ms
            .unwrap_or(duration_millis_u64(ctx.cfg.timing.bootstrap_timeout)),
        ProcessExecutionKind::PgRewind(spec) => spec
            .timeout_ms
            .unwrap_or(duration_millis_u64(ctx.cfg.timing.pg_rewind_timeout)),
        ProcessExecutionKind::Promote(spec) => spec
            .timeout_ms
            .unwrap_or(duration_millis_u64(ctx.cfg.timing.bootstrap_timeout)),
        ProcessExecutionKind::Demote(spec) => spec
            .timeout_ms
            .unwrap_or(duration_millis_u64(ctx.cfg.timing.fencing_timeout)),
        ProcessExecutionKind::StartPostgres(_) => {
            duration_millis_u64(ctx.cfg.timing.bootstrap_timeout)
        }
    }
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
                job_kind: request.intent.process_job_kind(),
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
                job_kind: request.intent.process_job_kind(),
                cause: inner.to_string(),
            })
            .map_err(|err| {
                WorkerError::Message(format!("process build command log send failed: {err}"))
            }),
    }
}

fn duration_millis_u64(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process::{Child, Command},
        time::Duration,
    };

    use tokio::sync::mpsc::UnboundedSender;

    use crate::{
        config_v2::types::PostgresConfig,
        config_v2::{managed_postgres_test_config, RuntimeConfigV2},
        dcs::DcsSnapshot,
        dev_support::test_fs::unique_test_dir,
        logging::LogSender,
        postgres_managed_conf::{managed_standby_passfile_path, MANAGED_POSTGRESQL_CONF_NAME},
        process::{
            jobs::{
                ActiveJob, ActiveJobKind, PostgresStartIntent, ProcessCommandRunner,
                ProcessCommandSpec, ProcessIntent, ReplicaProvisionIntent,
            },
            state::{
                ProcessCadence, ProcessIntentRequest, ProcessObservedState, ProcessRuntime,
                ProcessState, ProcessWorkerCtx,
            },
        },
        state::{new_state_channel, JobId, MemberId, StateSubscriber, UnixMillis, WorkerStatus},
    };

    use super::{bootstrap_with_runtime, start_job, step_once};
    use crate::process::postmaster::{lookup_managed_postmaster, ManagedPostmasterTarget};

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

    struct ChildGuard(Option<Child>);

    impl ChildGuard {
        #[cfg(unix)]
        fn spawn_fake_postgres(
            root: &std::path::Path,
            data_dir: &std::path::Path,
        ) -> Result<Self, String> {
            let bin_dir = root.join("bin");
            fs::create_dir_all(&bin_dir).map_err(|err| {
                format!(
                    "create fake postgres bin dir {} failed: {err}",
                    bin_dir.display()
                )
            })?;
            let fake_postgres = bin_dir.join("postgres");
            fs::write(
                &fake_postgres,
                "#!/bin/bash\nexec -a postgres /bin/sleep 30\n",
            )
            .map_err(|err| {
                format!(
                    "write fake postgres script {} failed: {err}",
                    fake_postgres.display()
                )
            })?;
            let mut permissions = fs::metadata(&fake_postgres)
                .map_err(|err| {
                    format!(
                        "read fake postgres metadata {} failed: {err}",
                        fake_postgres.display()
                    )
                })?
                .permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
            fs::set_permissions(&fake_postgres, permissions).map_err(|err| {
                format!(
                    "set fake postgres script permissions {} failed: {err}",
                    fake_postgres.display()
                )
            })?;
            let child = Command::new(&fake_postgres)
                .arg(data_dir.display().to_string())
                .spawn()
                .map_err(|err| {
                    format!(
                        "spawn fake postgres process {} failed: {err}",
                        fake_postgres.display()
                    )
                })?;
            Ok(Self(Some(child)))
        }

        #[cfg(not(unix))]
        fn spawn_fake_postgres(
            _root: &std::path::Path,
            _data_dir: &std::path::Path,
        ) -> Result<Self, String> {
            Err("fake postgres helper is only implemented on unix".to_string())
        }
    }

    impl Drop for ChildGuard {
        fn drop(&mut self) {
            if let Some(child) = self.0.as_mut() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    fn wait_for_fake_postgres_readiness(data_dir: &std::path::Path) -> Result<(), String> {
        let mut attempts = 0_u8;
        while attempts < 50 {
            let target = ManagedPostmasterTarget::from_data_dir(data_dir.to_path_buf());
            if lookup_managed_postmaster(&target).is_ok() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
            attempts = attempts.saturating_add(1);
        }
        Err(format!(
            "fake postgres readiness timed out for data_dir={}",
            data_dir.display()
        ))
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

        let fake_postgres = ChildGuard::spawn_fake_postgres(&root, &data_dir)?;
        let fake_postgres_pid = fake_postgres
            .0
            .as_ref()
            .map(std::process::Child::id)
            .ok_or_else(|| "fake postgres process handle missing child pid".to_string())?;
        let pid_contents = format!("{fake_postgres_pid}\n{}\n", data_dir.display());
        let pid_file = data_dir.join("postmaster.pid");
        fs::write(&pid_file, pid_contents)
            .map_err(|err| format!("write postmaster.pid {} failed: {err}", pid_file.display()))?;
        wait_for_fake_postgres_readiness(&data_dir)?;

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
                if *job_kind != crate::process::jobs::ActiveJobKind::StartDetachedStandby {
                    return Err(format!(
                        "unexpected job kind after noop: expected={:?} actual={job_kind:?}",
                        crate::process::jobs::ActiveJobKind::StartDetachedStandby
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
                kind: ActiveJobKind::Bootstrap,
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
