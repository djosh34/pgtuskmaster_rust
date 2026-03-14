use std::{fs, path::Path, process::Stdio};

use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
    sync::mpsc::error::TryRecvError,
};

use crate::{
    config::{PostgresBinaryName, ProcessConfig, RoleAuthConfig, RuntimeConfig},
    dcs::{DcsMemberView, DcsView},
    logging::{
        CapturedStream, InternalEvent, LogEvent, LogHandle, ProcessEvent,
        ProcessExecutionIdentity, ProcessJobIdentity, ProcessJobKind, SeverityText,
        SubprocessLineEvent,
    },
    pginfo::state::render_pg_conninfo,
    postgres_managed::{inspect_managed_recovery_state, materialize_managed_postgres_config},
    postgres_managed_conf::{managed_standby_auth_from_role_auth, ManagedPostgresStartIntent},
    process::postmaster::{
        lookup_managed_postmaster, ManagedPostmasterError, ManagedPostmasterTarget,
    },
    state::{JobId, UnixMillis, WorkerError, WorkerStatus},
};

use super::{
    jobs::{
        ActiveJob, ActiveJobKind, DemoteSpec, PostgresStartIntent, PostgresStartMode,
        ProcessCommandSpec, ProcessEnvValue, ProcessEnvVar, ProcessError, ProcessExit,
        ProcessHandle, ProcessIntent, ProcessLogIdentity, ProcessOutputLine, ProcessOutputStream,
        PromoteSpec, ReplicaProvisionIntent,
    },
    source::{basebackup_source_from_member, rewind_source_from_member},
    state::{
        ActiveRuntime, JobOutcome, ProcessExecutionKind, ProcessExecutionRequest,
        ProcessIntentRequest, ProcessJobRejection, ProcessState, ProcessWorkerCtx,
    },
};

const PROCESS_OUTPUT_READ_CHUNK_BYTES: usize = 8192;
const PROCESS_OUTPUT_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1);
const PROCESS_OUTPUT_DRAIN_MAX_BYTES: usize = 256 * 1024;
const PG_CTL_DEFAULT_WAIT_SECONDS: u64 = 30;

#[derive(Default)]
pub(crate) struct TokioCommandRunner;

fn process_job_identity(job_id: &JobId, job_kind: ProcessJobKind) -> ProcessJobIdentity {
    ProcessJobIdentity {
        job_id: job_id.0.clone(),
        kind: job_kind,
    }
}

fn process_job_kind_from_intent(intent: &ProcessIntent) -> ProcessJobKind {
    match intent {
        ProcessIntent::Bootstrap => ProcessJobKind::Bootstrap,
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { .. }) => {
            ProcessJobKind::BaseBackup
        }
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { .. }) => {
            ProcessJobKind::PgRewind
        }
        ProcessIntent::Start(PostgresStartIntent::Primary) => ProcessJobKind::StartPrimary,
        ProcessIntent::Start(PostgresStartIntent::DetachedStandby) => {
            ProcessJobKind::StartDetachedStandby
        }
        ProcessIntent::Start(PostgresStartIntent::Replica { .. }) => ProcessJobKind::StartReplica,
        ProcessIntent::Promote => ProcessJobKind::Promote,
        ProcessIntent::Demote(_) => ProcessJobKind::Demote,
    }
}

fn process_job_kind_from_execution(kind: &ProcessExecutionKind) -> ProcessJobKind {
    match kind {
        ProcessExecutionKind::Bootstrap(_) => ProcessJobKind::Bootstrap,
        ProcessExecutionKind::BaseBackup(_) => ProcessJobKind::BaseBackup,
        ProcessExecutionKind::PgRewind(_) => ProcessJobKind::PgRewind,
        ProcessExecutionKind::Promote(_) => ProcessJobKind::Promote,
        ProcessExecutionKind::Demote(_) => ProcessJobKind::Demote,
        ProcessExecutionKind::StartPostgres(_) => ProcessJobKind::StartPostgres,
    }
}

fn process_execution_identity(identity: &ProcessLogIdentity) -> ProcessExecutionIdentity {
    ProcessExecutionIdentity {
        job: process_job_identity(&identity.job_id, identity.job_kind),
        binary: identity.binary.clone(),
    }
}

fn emit_process_event(
    log: &LogHandle,
    origin: &str,
    event: ProcessEvent,
    severity: SeverityText,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    log.emit(origin, LogEvent::Process(InternalEvent::new(severity, event)))
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
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
            log_identity: _,
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

pub(crate) async fn run(mut ctx: ProcessWorkerCtx) -> Result<(), WorkerError> {
    emit_process_event(
        &ctx.runtime.log,
        "process_worker::run",
        ProcessEvent::WorkerRunStarted {
            capture_subprocess_output: ctx.runtime.capture_subprocess_output,
        },
        SeverityText::Debug,
        "process worker start log emit failed",
    )?;
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.cadence.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    match ctx.control.inbox.try_recv() {
        Ok(request) => {
            emit_process_event(
                &ctx.runtime.log,
                "process_worker::step_once",
                ProcessEvent::RequestReceived {
                    job: process_job_identity(
                        &request.id,
                        process_job_kind_from_intent(&request.intent),
                    ),
                },
                SeverityText::Debug,
                "process request log emit failed",
            )?;
            start_job(ctx, request).await?;
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            if !ctx.control.inbox_disconnected_logged {
                ctx.control.inbox_disconnected_logged = true;
                emit_process_event(
                    &ctx.runtime.log,
                    "process_worker::step_once",
                    ProcessEvent::InboxDisconnected,
                    SeverityText::Warn,
                    "process inbox disconnected log emit failed",
                )?;
            }
        }
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
    ctx: &ProcessWorkerCtx,
    intent: &ProcessIntent,
) -> Option<(std::path::PathBuf, std::path::PathBuf, u16)> {
    match intent {
        ProcessIntent::Start(
            PostgresStartIntent::Primary
            | PostgresStartIntent::DetachedStandby
            | PostgresStartIntent::Replica { .. },
        ) => {
            let runtime_config = ctx.observed.runtime_config.latest();
            Some((
                runtime_config.postgres.paths.data_dir.clone(),
                ctx.plan.postgres.paths.socket_dir.clone(),
                ctx.plan.postgres.port,
            ))
        }
        _ => None,
    }
}

pub(crate) async fn start_job(
    ctx: &mut ProcessWorkerCtx,
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
        emit_process_event(
            &ctx.runtime.log,
            "process_worker::start_job",
            ProcessEvent::BusyRejected {
                job: process_job_identity(
                    &rejected_job_id,
                    process_job_kind_from_intent(&request.intent),
                ),
            },
            SeverityText::Warn,
            "process busy reject log emit failed",
        )?;
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
                emit_process_event(
                    &ctx.runtime.log,
                    "process_worker::start_job",
                    ProcessEvent::StartPostgresAlreadyRunning {
                        job: process_job_identity(&request.id, ProcessJobKind::StartPostgres),
                        data_dir: data_dir.display().to_string(),
                    },
                    SeverityText::Info,
                    "process start-postgres noop log emit failed",
                )?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Success {
                        id: request.id,
                        job_kind: active_kind_from_intent(&request.intent),
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
            Ok(false) => {}
            Err(error) => {
                emit_process_event(
                    &ctx.runtime.log,
                    "process_worker::start_job",
                    ProcessEvent::StartPostgresPreflightFailed {
                        job: process_job_identity(&request.id, ProcessJobKind::StartPostgres),
                        error: error.to_string(),
                    },
                    SeverityText::Error,
                    "process start-postgres preflight log emit failed",
                )?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Failure {
                        id: request.id,
                        job_kind: active_kind_from_intent(&request.intent),
                        error,
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
        }
    }

    let execution_request = match materialize_execution_request(ctx, &request) {
        Ok(materialized) => materialized,
        Err(error) => {
            emit_process_event(
                &ctx.runtime.log,
                "process_worker::start_job",
                ProcessEvent::IntentMaterializationFailed {
                    job: process_job_identity(
                        &request.id,
                        process_job_kind_from_intent(&request.intent),
                    ),
                    error: error.to_string(),
                },
                SeverityText::Error,
                "process intent materialization log emit failed",
            )?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: active_kind_from_intent(&request.intent),
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };
    let timeout_ms = timeout_for_kind(&execution_request.kind, &ctx.config);
    let deadline_at = UnixMillis(now.0.saturating_add(timeout_ms));

    let command = match build_command(
        &ctx.config,
        &request.id,
        &execution_request.kind,
        ctx.runtime.capture_subprocess_output,
    ) {
        Ok(command) => command,
        Err(error) => {
            emit_process_event(
                &ctx.runtime.log,
                "process_worker::start_job",
                ProcessEvent::BuildCommandFailed {
                    job: process_job_identity(
                        &request.id,
                        process_job_kind_from_execution(&execution_request.kind),
                    ),
                    error: error.to_string(),
                },
                SeverityText::Error,
                "process build command log emit failed",
            )?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: active_kind(&execution_request.kind),
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };

    let log_identity = command.log_identity.clone();
    let handle = match ctx.runtime.command_runner.spawn(command) {
        Ok(handle) => handle,
        Err(error) => {
            emit_process_event(
                &ctx.runtime.log,
                "process_worker::start_job",
                ProcessEvent::SpawnFailed {
                    job: process_job_identity(
                        &request.id,
                        process_job_kind_from_execution(&execution_request.kind),
                    ),
                    error: error.to_string(),
                },
                SeverityText::Error,
                "process spawn log emit failed",
            )?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: active_kind(&execution_request.kind),
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
        kind: active_kind(&execution_request.kind),
        started_at: now,
        deadline_at,
    };
    let started_execution = process_execution_identity(&log_identity);

    ctx.control.active_runtime = Some(ActiveRuntime {
        request: execution_request,
        deadline_at,
        handle,
        log_identity,
    });
    ctx.state_channel.current = ProcessState::Running {
        worker: WorkerStatus::Running,
        active,
    };
    emit_process_event(
        &ctx.runtime.log,
        "process_worker::start_job",
        ProcessEvent::Started {
            execution: started_execution,
        },
        SeverityText::Info,
        "process job started log emit failed",
    )?;
    publish_state(ctx)
}

pub(crate) async fn tick_active_job(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    let mut runtime = match ctx.control.active_runtime.take() {
        Some(runtime) => runtime,
        None => return Ok(()),
    };

    let now = current_time(ctx)?;
    match runtime
        .handle
        .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
        .await
    {
        Ok(lines) => {
            for line in lines {
                if let Err(err) =
                    emit_subprocess_line(&ctx.runtime.log, &runtime.log_identity, line.clone())
                {
                    emit_process_output_emit_failed(ctx, &runtime.log_identity, &line, &err)?;
                }
            }
        }
        Err(err) => emit_process_output_drain_failed(
            &ctx.runtime.log,
            &runtime.log_identity,
            err.to_string(),
        )?,
    }
    if now.0 >= runtime.deadline_at.0 {
        emit_process_event(
            &ctx.runtime.log,
            "process_worker::tick_active_job",
            ProcessEvent::Timeout {
                execution: process_execution_identity(&runtime.log_identity),
            },
            SeverityText::Warn,
            "process timeout log emit failed",
        )?;
        let cancel_result = runtime.handle.cancel().await;
        match runtime
            .handle
            .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
            .await
        {
            Ok(lines) => {
                for line in lines {
                    if let Err(err) =
                        emit_subprocess_line(&ctx.runtime.log, &runtime.log_identity, line.clone())
                    {
                        emit_process_output_emit_failed(ctx, &runtime.log_identity, &line, &err)?;
                    }
                }
            }
            Err(err) => emit_process_output_drain_failed(
                &ctx.runtime.log,
                &runtime.log_identity,
                err.to_string(),
            )?,
        }
        let outcome = match cancel_result {
            Ok(()) => JobOutcome::Timeout {
                id: runtime.request.id,
                job_kind: active_kind(&runtime.request.kind),
                finished_at: now,
            },
            Err(error) => JobOutcome::Failure {
                id: runtime.request.id,
                job_kind: active_kind(&runtime.request.kind),
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
            match runtime
                .handle
                .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
                .await
            {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = emit_subprocess_line(
                            &ctx.runtime.log,
                            &runtime.log_identity,
                            line.clone(),
                        ) {
                            emit_process_output_emit_failed(
                                ctx,
                                &runtime.log_identity,
                                &line,
                                &err,
                            )?;
                        }
                    }
                }
                Err(err) => emit_process_output_drain_failed(
                    &ctx.runtime.log,
                    &runtime.log_identity,
                    err.to_string(),
                )?,
            }
            let job_id = runtime.request.id.clone();
            let outcome = JobOutcome::Success {
                id: job_id,
                job_kind: active_kind(&runtime.request.kind),
                finished_at: now,
            };
            emit_process_event(
                &ctx.runtime.log,
                "process_worker::tick_active_job",
                ProcessEvent::ExitedSuccessfully {
                    execution: process_execution_identity(&runtime.log_identity),
                },
                SeverityText::Info,
                "process exit log emit failed",
            )?;
            transition_to_idle(ctx, outcome, now)
        }
        Ok(Some(exit)) => {
            match runtime
                .handle
                .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
                .await
            {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = emit_subprocess_line(
                            &ctx.runtime.log,
                            &runtime.log_identity,
                            line.clone(),
                        ) {
                            emit_process_output_emit_failed(
                                ctx,
                                &runtime.log_identity,
                                &line,
                                &err,
                            )?;
                        }
                    }
                }
                Err(err) => emit_process_output_drain_failed(
                    &ctx.runtime.log,
                    &runtime.log_identity,
                    err.to_string(),
                )?,
            }
            let exit_error = ProcessError::from_exit(exit);
            let outcome = JobOutcome::Failure {
                id: runtime.request.id.clone(),
                job_kind: active_kind(&runtime.request.kind),
                error: exit_error.clone(),
                finished_at: now,
            };
            emit_process_event(
                &ctx.runtime.log,
                "process_worker::tick_active_job",
                ProcessEvent::ExitedUnsuccessfully {
                    execution: process_execution_identity(&runtime.log_identity),
                    error: exit_error.to_string(),
                },
                SeverityText::Warn,
                "process exit log emit failed",
            )?;
            transition_to_idle(ctx, outcome, now)
        }
        Err(error) => {
            match runtime
                .handle
                .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
                .await
            {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = emit_subprocess_line(
                            &ctx.runtime.log,
                            &runtime.log_identity,
                            line.clone(),
                        ) {
                            emit_process_output_emit_failed(
                                ctx,
                                &runtime.log_identity,
                                &line,
                                &err,
                            )?;
                        }
                    }
                }
                Err(err) => emit_process_output_drain_failed(
                    &ctx.runtime.log,
                    &runtime.log_identity,
                    err.to_string(),
                )?,
            }
            let outcome = JobOutcome::Failure {
                id: runtime.request.id.clone(),
                job_kind: active_kind(&runtime.request.kind),
                error,
                finished_at: now,
            };
            emit_process_event(
                &ctx.runtime.log,
                "process_worker::tick_active_job",
                ProcessEvent::PollFailed {
                    execution: process_execution_identity(&runtime.log_identity),
                    error: outcome_error_string(&outcome),
                },
                SeverityText::Error,
                "process poll failure log emit failed",
            )?;
            transition_to_idle(ctx, outcome, now)
        }
    }
}

fn emit_process_output_drain_failed(
    log: &LogHandle,
    identity: &ProcessLogIdentity,
    error: String,
) -> Result<(), WorkerError> {
    emit_process_event(
        log,
        "process_worker::tick_active_job",
        ProcessEvent::OutputDrainFailed {
            execution: process_execution_identity(identity),
            error,
        },
        SeverityText::Warn,
        "process output drain log emit failed",
    )
}

fn emit_process_output_emit_failed(
    ctx: &ProcessWorkerCtx,
    identity: &ProcessLogIdentity,
    line: &ProcessOutputLine,
    error: &crate::logging::LogError,
) -> Result<(), WorkerError> {
    emit_process_event(
        &ctx.runtime.log,
        "process_worker::emit_subprocess_line",
        ProcessEvent::OutputEmitFailed {
            execution: process_execution_identity(identity),
            stream: match line.stream {
                ProcessOutputStream::Stdout => CapturedStream::Stdout,
                ProcessOutputStream::Stderr => CapturedStream::Stderr,
            },
            bytes_len: line.bytes.len(),
            error: error.to_string(),
        },
        SeverityText::Warn,
        "process output emit failure log emit failed",
    )
}

fn outcome_error_string(outcome: &JobOutcome) -> String {
    match outcome {
        JobOutcome::Success { .. } => "success".to_string(),
        JobOutcome::Timeout { .. } => "timeout".to_string(),
        JobOutcome::Failure { error, .. } => error.to_string(),
    }
}

fn emit_subprocess_line(
    log: &LogHandle,
    identity: &ProcessLogIdentity,
    line: ProcessOutputLine,
) -> Result<(), crate::logging::LogError> {
    let stream = match line.stream {
        ProcessOutputStream::Stdout => CapturedStream::Stdout,
        ProcessOutputStream::Stderr => CapturedStream::Stderr,
    };

    log.emit(
        "process_worker",
        LogEvent::SubprocessLine(SubprocessLineEvent {
            producer: crate::logging::LogProducer::PgTool,
            stream,
            execution: process_execution_identity(identity),
            origin: "process_worker".to_string(),
            bytes: line.bytes,
        }),
    )
}

fn transition_to_idle(
    ctx: &mut ProcessWorkerCtx,
    outcome: JobOutcome,
    _now: UnixMillis,
) -> Result<(), WorkerError> {
    ctx.state_channel.current = ProcessState::Idle {
        worker: WorkerStatus::Running,
        last_outcome: Some(outcome),
    };
    publish_state(ctx)
}

fn publish_state(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    ctx.state_channel
        .publisher
        .publish(ctx.state_channel.current.clone())
        .map_err(|err| WorkerError::Message(format!("process publish failed: {err}")))?;
    Ok(())
}

fn current_time(ctx: &mut ProcessWorkerCtx) -> Result<UnixMillis, WorkerError> {
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

fn timeout_for_kind(kind: &ProcessExecutionKind, config: &ProcessConfig) -> u64 {
    match kind {
        ProcessExecutionKind::Bootstrap(spec) => {
            spec.timeout_ms.unwrap_or(config.timeouts.bootstrap_ms)
        }
        ProcessExecutionKind::BaseBackup(spec) => {
            spec.timeout_ms.unwrap_or(config.timeouts.bootstrap_ms)
        }
        ProcessExecutionKind::PgRewind(spec) => {
            spec.timeout_ms.unwrap_or(config.timeouts.pg_rewind_ms)
        }
        ProcessExecutionKind::Promote(spec) => {
            spec.timeout_ms.unwrap_or(config.timeouts.bootstrap_ms)
        }
        ProcessExecutionKind::Demote(spec) => spec.timeout_ms.unwrap_or(config.timeouts.fencing_ms),
        ProcessExecutionKind::StartPostgres(spec) => {
            spec.timeout_ms.unwrap_or(config.timeouts.bootstrap_ms)
        }
    }
}

fn active_kind(kind: &ProcessExecutionKind) -> ActiveJobKind {
    match kind {
        ProcessExecutionKind::Bootstrap(_) => ActiveJobKind::Bootstrap,
        ProcessExecutionKind::BaseBackup(_) => ActiveJobKind::BaseBackup,
        ProcessExecutionKind::PgRewind(_) => ActiveJobKind::PgRewind,
        ProcessExecutionKind::Promote(_) => ActiveJobKind::Promote,
        ProcessExecutionKind::Demote(_) => ActiveJobKind::Demote,
        ProcessExecutionKind::StartPostgres(spec) => match spec.mode {
            PostgresStartMode::Primary => ActiveJobKind::StartPrimary,
            PostgresStartMode::DetachedStandby => ActiveJobKind::StartDetachedStandby,
            PostgresStartMode::Replica => ActiveJobKind::StartReplica,
        },
    }
}

fn active_kind_from_intent(intent: &ProcessIntent) -> ActiveJobKind {
    match intent {
        ProcessIntent::Bootstrap => ActiveJobKind::Bootstrap,
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { .. }) => {
            ActiveJobKind::BaseBackup
        }
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { .. }) => {
            ActiveJobKind::PgRewind
        }
        ProcessIntent::Promote => ActiveJobKind::Promote,
        ProcessIntent::Demote(_) => ActiveJobKind::Demote,
        ProcessIntent::Start(PostgresStartIntent::Primary) => ActiveJobKind::StartPrimary,
        ProcessIntent::Start(PostgresStartIntent::DetachedStandby) => {
            ActiveJobKind::StartDetachedStandby
        }
        ProcessIntent::Start(PostgresStartIntent::Replica { .. }) => ActiveJobKind::StartReplica,
    }
}

fn build_command(
    config: &ProcessConfig,
    job_id: &JobId,
    kind: &ProcessExecutionKind,
    capture_output: bool,
) -> Result<ProcessCommandSpec, ProcessError> {
    match kind {
        ProcessExecutionKind::Bootstrap(spec) => {
            validate_non_empty_path("bootstrap.data_dir", &spec.data_dir)?;
            if spec.superuser.as_str().trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "bootstrap.superuser must not be empty".to_string(),
                ));
            }
            let program = resolve_process_binary(config, PostgresBinaryName::Initdb)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-A".to_string(),
                    "trust".to_string(),
                    "-U".to_string(),
                    spec.superuser.as_str().to_string(),
                ],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessExecutionKind::BaseBackup(spec) => {
            validate_non_empty_path("basebackup.data_dir", &spec.data_dir)?;
            if spec.source.conninfo.host.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "basebackup.source_conninfo.host must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.user.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "basebackup.source_conninfo.user must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.dbname.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "basebackup.source_conninfo.dbname must not be empty".to_string(),
                ));
            }
            let program = resolve_process_binary(config, PostgresBinaryName::PgBasebackup)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "--dbname".to_string(),
                    render_pg_conninfo(&spec.source.conninfo),
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-Fp".to_string(),
                    "-Xs".to_string(),
                ],
                env: role_auth_env(&spec.source.auth),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessExecutionKind::PgRewind(spec) => {
            validate_non_empty_path("pg_rewind.target_data_dir", &spec.target_data_dir)?;
            if spec.source.conninfo.host.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_rewind.source_conninfo.host must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.user.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_rewind.source_conninfo.user must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.dbname.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_rewind.source_conninfo.dbname must not be empty".to_string(),
                ));
            }
            let program = resolve_process_binary(config, PostgresBinaryName::PgRewind)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "--target-pgdata".to_string(),
                    spec.target_data_dir.display().to_string(),
                    "--source-server".to_string(),
                    render_pg_conninfo(&spec.source.conninfo),
                ],
                env: role_auth_env(&spec.source.auth),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessExecutionKind::Promote(spec) => {
            validate_non_empty_path("promote.data_dir", &spec.data_dir)?;
            let mut args = vec![
                "-D".to_string(),
                spec.data_dir.display().to_string(),
                "promote".to_string(),
                "-w".to_string(),
            ];
            if let Some(wait_seconds) = spec.wait_seconds {
                args.push("-t".to_string());
                args.push(wait_seconds.to_string());
            }
            let program = resolve_process_binary(config, PostgresBinaryName::PgCtl)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args,
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessExecutionKind::Demote(spec) => {
            validate_non_empty_path("demote.data_dir", &spec.data_dir)?;
            let program = resolve_process_binary(config, PostgresBinaryName::PgCtl)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "stop".to_string(),
                    "-m".to_string(),
                    spec.mode.as_pg_ctl_arg().to_string(),
                    "-w".to_string(),
                ],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessExecutionKind::StartPostgres(spec) => {
            validate_non_empty_path("start_postgres.data_dir", &spec.data_dir)?;
            validate_non_empty_path("start_postgres.config_file", &spec.config_file)?;
            validate_non_empty_path("start_postgres.log_file", &spec.log_file)?;
            let wait_seconds = spec.wait_seconds.unwrap_or(PG_CTL_DEFAULT_WAIT_SECONDS);
            let option_tokens = vec![
                "-c".to_string(),
                format!("config_file={}", spec.config_file.display()),
            ];
            let options = render_pg_ctl_option_string(&option_tokens)?;
            let program = resolve_process_binary(config, PostgresBinaryName::PgCtl)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-l".to_string(),
                    spec.log_file.display().to_string(),
                    "-o".to_string(),
                    options,
                    "start".to_string(),
                    "-w".to_string(),
                    "-t".to_string(),
                    wait_seconds.to_string(),
                ],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: process_job_kind_from_execution(kind),
                    binary: binary_label(program.as_path()),
                },
            })
        }
    }
}

fn resolve_process_binary(
    config: &ProcessConfig,
    binary: PostgresBinaryName,
) -> Result<std::path::PathBuf, ProcessError> {
    config
        .binaries
        .resolve_binary_path(binary)
        .map_err(ProcessError::InvalidSpec)
}

fn role_auth_env(auth: &RoleAuthConfig) -> Vec<ProcessEnvVar> {
    match auth {
        RoleAuthConfig::Password { password } => vec![ProcessEnvVar {
            key: "PGPASSWORD".to_string(),
            value: ProcessEnvValue::Secret(password.clone()),
        }],
    }
}

fn materialize_execution_request(
    ctx: &ProcessWorkerCtx,
    request: &ProcessIntentRequest,
) -> Result<ProcessExecutionRequest, ProcessError> {
    let runtime_config = ctx.observed.runtime_config.latest();
    let dcs = ctx.observed.dcs.latest();
    let kind = match &request.intent {
        ProcessIntent::Bootstrap => {
            wipe_data_dir(runtime_config.postgres.paths.data_dir.as_path())?;
            ProcessExecutionKind::Bootstrap(super::jobs::BootstrapSpec {
                data_dir: runtime_config.postgres.paths.data_dir.clone(),
                superuser: runtime_config
                    .postgres
                    .roles
                    .mandatory
                    .superuser
                    .username
                    .clone(),
                timeout_ms: None,
            })
        }
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }) => {
            wipe_data_dir(runtime_config.postgres.paths.data_dir.as_path())?;
            let source = basebackup_source_from_member(
                &ctx.identity.self_id,
                &ctx.plan,
                resolve_source_member(&dcs, leader)?,
            )
            .map_err(source_materialization_error)?;
            ProcessExecutionKind::BaseBackup(super::jobs::BaseBackupSpec {
                data_dir: runtime_config.postgres.paths.data_dir.clone(),
                source,
                timeout_ms: Some(runtime_config.process.timeouts.bootstrap_ms),
            })
        }
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { leader }) => {
            let source = rewind_source_from_member(
                &ctx.identity.self_id,
                &ctx.plan,
                resolve_source_member(&dcs, leader)?,
            )
            .map_err(source_materialization_error)?;
            ProcessExecutionKind::PgRewind(super::jobs::PgRewindSpec {
                target_data_dir: runtime_config.postgres.paths.data_dir.clone(),
                source,
                timeout_ms: None,
            })
        }
        ProcessIntent::Start(PostgresStartIntent::Primary) => {
            let start_intent = primary_start_intent(&runtime_config)?;
            materialize_start_postgres(
                &runtime_config,
                &ctx.plan,
                PostgresStartMode::Primary,
                &start_intent,
            )?
        }
        ProcessIntent::Start(PostgresStartIntent::DetachedStandby) => materialize_start_postgres(
            &runtime_config,
            &ctx.plan,
            PostgresStartMode::DetachedStandby,
            &ManagedPostgresStartIntent::detached_standby(),
        )?,
        ProcessIntent::Start(PostgresStartIntent::Replica { leader }) => {
            let start_intent = replica_start_intent(ctx, &runtime_config, &dcs, leader)?;
            materialize_start_postgres(
                &runtime_config,
                &ctx.plan,
                PostgresStartMode::Replica,
                &start_intent,
            )?
        }
        ProcessIntent::Promote => ProcessExecutionKind::Promote(PromoteSpec {
            data_dir: runtime_config.postgres.paths.data_dir.clone(),
            wait_seconds: None,
            timeout_ms: None,
        }),
        ProcessIntent::Demote(mode) => ProcessExecutionKind::Demote(DemoteSpec {
            data_dir: runtime_config.postgres.paths.data_dir.clone(),
            mode: mode.clone(),
            timeout_ms: None,
        }),
    };

    Ok(ProcessExecutionRequest {
        id: request.id.clone(),
        kind,
    })
}

fn primary_start_intent(
    runtime_config: &RuntimeConfig,
) -> Result<ManagedPostgresStartIntent, ProcessError> {
    let managed_recovery_state = inspect_managed_recovery_state(
        runtime_config.postgres.paths.data_dir.as_path(),
    )
    .map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "inspect managed recovery state for primary start failed: {err}"
        ))
    })?;
    if managed_recovery_state != crate::postgres_managed_conf::ManagedRecoverySignal::None {
        return Err(ProcessError::InvalidSpec(
            "existing postgres data dir contains managed replica recovery state but no leader-derived source is available to rebuild authoritative managed config".to_string(),
        ));
    }
    Ok(ManagedPostgresStartIntent::primary())
}

fn replica_start_intent(
    ctx: &ProcessWorkerCtx,
    runtime_config: &RuntimeConfig,
    dcs: &DcsView,
    leader: &crate::state::MemberId,
) -> Result<ManagedPostgresStartIntent, ProcessError> {
    let source = basebackup_source_from_member(
        &ctx.identity.self_id,
        &ctx.plan,
        resolve_source_member(dcs, leader)?,
    )
    .map_err(source_materialization_error)?;
    Ok(ManagedPostgresStartIntent::replica(
        source.conninfo,
        managed_standby_auth_from_role_auth(
            &source.auth,
            runtime_config.postgres.paths.data_dir.as_path(),
        ),
        None,
    ))
}

fn materialize_start_postgres(
    runtime_config: &RuntimeConfig,
    intent_runtime: &super::state::ProcessRuntimePlan,
    mode: PostgresStartMode,
    start_intent: &ManagedPostgresStartIntent,
) -> Result<ProcessExecutionKind, ProcessError> {
    let managed =
        materialize_managed_postgres_config(runtime_config, start_intent).map_err(|err| {
            ProcessError::InvalidSpec(format!("materialize managed postgres config failed: {err}"))
        })?;
    Ok(ProcessExecutionKind::StartPostgres(
        super::jobs::StartPostgresSpec {
            mode,
            data_dir: runtime_config.postgres.paths.data_dir.clone(),
            socket_dir: intent_runtime.postgres.paths.socket_dir.clone(),
            port: intent_runtime.postgres.port,
            config_file: managed.postgresql_conf_path,
            log_file: intent_runtime.postgres.paths.log_file.clone(),
            wait_seconds: None,
            timeout_ms: None,
        },
    ))
}

fn resolve_source_member<'a>(
    dcs: &'a DcsView,
    leader: &crate::state::MemberId,
) -> Result<&'a DcsMemberView, ProcessError> {
    dcs.members.get(leader).ok_or_else(|| {
        ProcessError::InvalidSpec(format!(
            "target member `{}` not present in DCS view",
            leader.0
        ))
    })
}

fn source_materialization_error(error: super::source::SourceMaterializationError) -> ProcessError {
    ProcessError::InvalidSpec(error.to_string())
}

fn wipe_data_dir(data_dir: &Path) -> Result<(), ProcessError> {
    if data_dir.as_os_str().is_empty() {
        return Err(ProcessError::InvalidSpec(
            "wipe_data_dir data_dir must not be empty".to_string(),
        ));
    }
    if data_dir.exists() {
        wipe_data_dir_contents(data_dir)?;
    } else {
        fs::create_dir_all(data_dir).map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir create_dir_all failed: {err}"))
        })?;
    }
    set_postgres_data_dir_permissions(data_dir)?;
    Ok(())
}

fn wipe_data_dir_contents(data_dir: &Path) -> Result<(), ProcessError> {
    let entries = fs::read_dir(data_dir).map_err(|err| {
        ProcessError::InvalidSpec(format!("wipe_data_dir read_dir failed: {err}"))
    })?;
    for entry_result in entries {
        let entry = entry_result.map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir read_dir entry failed: {err}"))
        })?;
        let file_type = entry.file_type().map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir file_type failed: {err}"))
        })?;
        let entry_path = entry.path();
        if file_type.is_dir() {
            fs::remove_dir_all(entry_path.as_path()).map_err(|err| {
                ProcessError::InvalidSpec(format!(
                    "wipe_data_dir remove_dir_all failed for {}: {err}",
                    entry_path.display()
                ))
            })?;
        } else {
            fs::remove_file(entry_path.as_path()).map_err(|err| {
                ProcessError::InvalidSpec(format!(
                    "wipe_data_dir remove_file failed for {}: {err}",
                    entry_path.display()
                ))
            })?;
        }
    }
    Ok(())
}

fn set_postgres_data_dir_permissions(data_dir: &Path) -> Result<(), ProcessError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700)).map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir set_permissions failed: {err}"))
        })?;
    }

    #[cfg(not(unix))]
    {
        let _path = data_dir;
    }

    Ok(())
}

fn binary_label(path: &std::path::Path) -> String {
    match path.file_name().and_then(|s| s.to_str()) {
        Some(name) if !name.trim().is_empty() => name.to_string(),
        _ => path.display().to_string(),
    }
}

fn validate_non_empty_path(field: &str, value: &std::path::Path) -> Result<(), ProcessError> {
    if value.as_os_str().is_empty() {
        return Err(ProcessError::InvalidSpec(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

fn render_pg_ctl_option_string(tokens: &[String]) -> Result<String, ProcessError> {
    let mut out = String::new();
    for (index, raw) in tokens.iter().enumerate() {
        let escaped = escape_pg_ctl_option_token(raw.as_str())?;
        if index > 0 {
            out.push(' ');
        }
        out.push_str(escaped.as_str());
    }
    Ok(out)
}

fn escape_pg_ctl_option_token(token: &str) -> Result<String, ProcessError> {
    if token.is_empty() {
        return Err(ProcessError::InvalidSpec(
            "pg_ctl option token must not be empty".to_string(),
        ));
    }
    if token.contains('\0') || token.contains('\n') || token.contains('\r') {
        return Err(ProcessError::InvalidSpec(
            "pg_ctl option token contains invalid characters".to_string(),
        ));
    }

    let needs_quotes = token.chars().any(|ch| ch.is_ascii_whitespace());
    if !needs_quotes {
        return Ok(token.to_string());
    }

    let mut out = String::with_capacity(token.len().saturating_add(2));
    out.push('"');
    for ch in token.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            other => out.push(other),
        }
    }
    out.push('"');
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process::{Child, Command},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use tokio::sync::mpsc::unbounded_channel;

    use crate::{
        config::{HaConfig, ProcessTimeoutsConfig},
        dcs::DcsView,
        dev_support::runtime_config::{sample_binary_paths, RuntimeConfigBuilder},
        logging::LogHandle,
        postgres_managed_conf::{managed_standby_passfile_path, MANAGED_POSTGRESQL_CONF_NAME},
        process::{
            jobs::{PostgresStartIntent, ProcessCommandRunner, ProcessCommandSpec, ProcessIntent},
            state::{
                ProcessCadence, ProcessControlPlane, ProcessIntentRequest, ProcessNodeIdentity,
                ProcessObservedState, ProcessRuntime, ProcessRuntimePlan, ProcessState,
                ProcessStateChannel, ProcessWorkerBootstrap, ProcessWorkerCtx,
            },
        },
        state::{new_state_channel, JobId, StateSubscriber, WorkerStatus},
    };

    use super::start_job;
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

    fn unique_test_dir(label: &str) -> Result<PathBuf, String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("clock error for test dir: {err}"))?
            .as_millis();
        let dir = std::env::temp_dir().join(format!(
            "pgtm-process-worker-{label}-{}-{millis}",
            std::process::id()
        ));
        fs::create_dir_all(&dir)
            .map_err(|err| format!("create test dir {} failed: {err}", dir.display()))?;
        Ok(dir)
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
    ) -> Result<(ProcessWorkerCtx, StateSubscriber<ProcessState>), String> {
        let cfg = RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir.clone())
            .transform_postgres(move |postgres| crate::config::PostgresConfig {
                paths: crate::config::PostgresPathsConfig {
                    data_dir: postgres.paths.data_dir.clone(),
                    socket_dir: Some(socket_dir.clone()),
                    log_file: Some(log_file.clone()),
                },
                ..postgres
            })
            .with_dcs_scope("cluster-a")
            .with_ha(HaConfig {
                loop_interval_ms: 500,
                lease_ttl_ms: 5_000,
            })
            .with_process(crate::config::ProcessConfig {
                timeouts: ProcessTimeoutsConfig {
                    pg_rewind_ms: 30_000,
                    bootstrap_ms: 30_000,
                    fencing_ms: 10_000,
                },
                working_root: std::path::PathBuf::from("/tmp/pgtuskmaster"),
                binaries: sample_binary_paths(),
            })
            .build();
        let initial = ProcessState::starting();
        let (publisher, subscriber) = new_state_channel(initial.clone());
        let (_cfg_publisher, runtime_config) = new_state_channel(cfg.clone());
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(DcsView::empty(WorkerStatus::Running));
        let (_tx, inbox) = unbounded_channel();

        Ok((
            ProcessWorkerCtx::new(ProcessWorkerBootstrap {
                cadence: ProcessCadence {
                    poll_interval: Duration::from_millis(10),
                    now: Box::new(super::system_now_unix_millis),
                },
                config: cfg.process.clone(),
                identity: ProcessNodeIdentity {
                    self_id: cfg.cluster.member_id.clone(),
                },
                observed: ProcessObservedState {
                    runtime_config,
                    dcs: dcs_subscriber,
                },
                plan: ProcessRuntimePlan::from_config(&cfg),
                state_channel: ProcessStateChannel {
                    current: initial,
                    publisher,
                    last_rejection: None,
                },
                control: ProcessControlPlane {
                    inbox,
                    inbox_disconnected_logged: false,
                    active_runtime: None,
                },
                runtime: ProcessRuntime {
                    log: LogHandle::disabled(),
                    capture_subprocess_output: true,
                    command_runner: Box::new(UnexpectedSpawnRunner),
                },
            }),
            subscriber,
        ))
    }

    #[tokio::test]
    async fn start_postgres_noop_preserves_existing_standby_passfile() -> Result<(), String> {
        let root = unique_test_dir("noop-passfile")?;
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
        let (mut ctx, _state_subscriber) = build_test_ctx(data_dir.clone(), socket_dir, log_file)?;
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
}
