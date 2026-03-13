use std::{fs, path::Path, process::Stdio};

use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
    sync::mpsc::error::TryRecvError,
};

use crate::{
    config::{ProcessConfig, RoleAuthConfig},
    logging::{
        AppEvent, AppEventHeader, LogHandle, SeverityText, StructuredFields, SubprocessLineRecord,
        SubprocessStream,
    },
    pginfo::state::render_pg_conninfo,
    state::{JobId, UnixMillis, WorkerError, WorkerStatus},
};

use super::{
    jobs::{
        ActiveJob, ActiveJobKind, ProcessCommandSpec, ProcessEnvValue, ProcessEnvVar, ProcessError,
        ProcessExit, ProcessHandle, ProcessLogIdentity, ProcessOutputLine, ProcessOutputStream,
    },
    state::{
        ActiveRuntime, JobOutcome, ProcessJobKind, ProcessJobRejection, ProcessJobRequest,
        ProcessState, ProcessWorkerCtx,
    },
};

const PROCESS_OUTPUT_READ_CHUNK_BYTES: usize = 8192;
const PROCESS_OUTPUT_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1);
const PROCESS_OUTPUT_DRAIN_MAX_BYTES: usize = 256 * 1024;
const PG_CTL_DEFAULT_WAIT_SECONDS: u64 = 30;

#[derive(Default)]
pub(crate) struct TokioCommandRunner;

#[derive(Clone, Copy, Debug)]
enum ProcessEventKind {
    RunStarted,
    RequestReceived,
    InboxDisconnected,
    BusyReject,
    StartPostgresNoop,
    StartPostgresPreflightFailed,
    BuildCommandFailed,
    SpawnFailed,
    Started,
    OutputDrainFailed,
    Timeout,
    Exited,
    PollFailed,
    OutputEmitFailed,
}

impl ProcessEventKind {
    fn name(self) -> &'static str {
        match self {
            Self::RunStarted => "process.worker.run_started",
            Self::RequestReceived => "process.worker.request_received",
            Self::InboxDisconnected => "process.worker.inbox_disconnected",
            Self::BusyReject => "process.worker.busy_reject",
            Self::StartPostgresNoop => "process.job.start_postgres_noop",
            Self::StartPostgresPreflightFailed => "process.job.start_postgres_preflight_failed",
            Self::BuildCommandFailed => "process.job.build_command_failed",
            Self::SpawnFailed => "process.job.spawn_failed",
            Self::Started => "process.job.started",
            Self::OutputDrainFailed => "process.worker.output_drain_failed",
            Self::Timeout => "process.job.timeout",
            Self::Exited => "process.job.exited",
            Self::PollFailed => "process.job.poll_failed",
            Self::OutputEmitFailed => "process.worker.output_emit_failed",
        }
    }
}

fn process_event(
    kind: ProcessEventKind,
    result: &str,
    severity: SeverityText,
    message: impl Into<String>,
) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(kind.name(), "process", result),
    )
}

fn process_job_fields(job_id: &JobId, job_kind: &str) -> StructuredFields {
    let mut fields = StructuredFields::new();
    fields.insert("job_id", job_id.0.clone());
    fields.insert("job_kind", job_kind.to_string());
    fields
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
    let mut event = process_event(
        ProcessEventKind::RunStarted,
        "ok",
        SeverityText::Debug,
        "process worker run started",
    );
    event
        .fields_mut()
        .insert("capture_subprocess_output", ctx.capture_subprocess_output);
    ctx.log
        .emit_app_event("process_worker::run", event)
        .map_err(|err| {
            WorkerError::Message(format!("process worker start log emit failed: {err}"))
        })?;
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    match ctx.inbox.try_recv() {
        Ok(request) => {
            let mut event = process_event(
                ProcessEventKind::RequestReceived,
                "ok",
                SeverityText::Debug,
                "process job request received",
            );
            event.fields_mut().append_json_map(
                process_job_fields(&request.id, request.kind.label()).into_attributes(),
            );
            ctx.log
                .emit_app_event("process_worker::step_once", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process request log emit failed: {err}"))
                })?;
            start_job(ctx, request).await?;
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            if !ctx.inbox_disconnected_logged {
                ctx.inbox_disconnected_logged = true;
                ctx.log
                    .emit_app_event(
                        "process_worker::step_once",
                        process_event(
                            ProcessEventKind::InboxDisconnected,
                            "failed",
                            SeverityText::Warn,
                            "process worker inbox disconnected",
                        ),
                    )
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process inbox disconnected log emit failed: {err}"
                        ))
                    })?;
            }
        }
    }

    tick_active_job(ctx).await
}

fn parse_postmaster_pid(pid_file: &Path) -> Result<u32, ProcessError> {
    let contents = fs::read_to_string(pid_file).map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "read postmaster.pid {} failed: {err}",
            pid_file.display()
        ))
    })?;
    let first_line = contents.lines().next().ok_or_else(|| {
        ProcessError::InvalidSpec(format!(
            "postmaster.pid {} missing pid line",
            pid_file.display()
        ))
    })?;
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        return Err(ProcessError::InvalidSpec(format!(
            "postmaster.pid {} pid line is empty",
            pid_file.display()
        )));
    }
    trimmed.parse::<u32>().map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "parse postmaster.pid pid '{trimmed}' failed: {err}"
        ))
    })
}

fn postmaster_pid_data_dir_matches(pid_file: &Path, data_dir: &Path) -> Result<bool, ProcessError> {
    let contents = fs::read_to_string(pid_file).map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "read postmaster.pid {} failed: {err}",
            pid_file.display()
        ))
    })?;
    let Some(raw_data_dir) = contents.lines().nth(1) else {
        return Ok(false);
    };
    let trimmed = raw_data_dir.trim();
    if trimmed.is_empty() {
        return Ok(false);
    }
    Ok(Path::new(trimmed) == data_dir)
}

fn pid_exists(pid: u32) -> Result<bool, ProcessError> {
    #[cfg(unix)]
    {
        let pid_i32 = i32::try_from(pid).map_err(|err| {
            ProcessError::InvalidSpec(format!("postmaster pid {pid} i32 conversion failed: {err}"))
        })?;
        let rc = unsafe { libc::kill(pid_i32, 0) };
        if rc == 0 {
            return Ok(true);
        }
        let err = std::io::Error::last_os_error();
        let raw = err.raw_os_error();
        if raw == Some(libc::ESRCH) {
            return Ok(false);
        }
        if raw == Some(libc::EPERM) {
            return Ok(true);
        }
        Err(ProcessError::InvalidSpec(format!(
            "kill(0) failed for pid={pid}: {err}"
        )))
    }
    #[cfg(not(unix))]
    {
        let _pid = pid;
        Ok(true)
    }
}

fn pid_matches_data_dir(pid: u32, data_dir: &Path, pid_file: &Path) -> Result<bool, ProcessError> {
    if !pid_exists(pid)? {
        return Ok(false);
    }

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
        let data_dir_text = data_dir.display().to_string();
        let cmdline_args = cmdline
            .split(|byte| *byte == 0)
            .filter(|arg| !arg.is_empty())
            .map(|arg| String::from_utf8_lossy(arg))
            .collect::<Vec<_>>();
        let has_data_dir = cmdline_args
            .iter()
            .any(|arg| arg.contains(data_dir_text.as_str()));
        let has_postgres_argv = cmdline_args.iter().any(|arg| {
            std::path::Path::new(arg.as_ref())
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| matches!(name, "postgres" | "postmaster"))
                .unwrap_or(false)
        });
        if !has_postgres_argv {
            return Ok(false);
        }
        if has_data_dir {
            return Ok(true);
        }
        postmaster_pid_data_dir_matches(pid_file, data_dir)
    }
    #[cfg(not(unix))]
    {
        let _pid_file = pid_file;
        let _data_dir = data_dir;
        Ok(true)
    }
}

fn pid_is_postgres_process(pid: u32) -> Result<bool, ProcessError> {
    if !pid_exists(pid)? {
        return Ok(false);
    }

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
        let pid = parse_postmaster_pid(&pid_file)?;
        if pid_matches_data_dir(pid, data_dir, &pid_file)? {
            return Ok(true);
        }

        remove_file_best_effort(&pid_file)?;
        let opts_file = data_dir.join("postmaster.opts");
        remove_file_best_effort(&opts_file)?;
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

pub(crate) async fn start_job(
    ctx: &mut ProcessWorkerCtx,
    request: ProcessJobRequest,
) -> Result<(), WorkerError> {
    if !can_accept_job(&ctx.state) {
        let now = current_time(ctx)?;
        ctx.last_rejection = Some(ProcessJobRejection {
            id: request.id,
            error: ProcessError::Busy,
            rejected_at: now,
        });
        let mut event = process_event(
            ProcessEventKind::BusyReject,
            "failed",
            SeverityText::Warn,
            "process worker busy; rejecting job",
        );
        let rejected_job_id = ctx
            .last_rejection
            .as_ref()
            .map(|rejection| rejection.id.clone())
            .unwrap_or_else(|| JobId("unknown".to_string()));
        event.fields_mut().append_json_map(
            process_job_fields(&rejected_job_id, request.kind.label()).into_attributes(),
        );
        ctx.log
            .emit_app_event("process_worker::start_job", event)
            .map_err(|err| {
                WorkerError::Message(format!("process busy reject log emit failed: {err}"))
            })?;
        return Ok(());
    }

    let now = current_time(ctx)?;
    let timeout_ms = timeout_for_kind(&request.kind, &ctx.config);
    let deadline_at = UnixMillis(now.0.saturating_add(timeout_ms));

    if let ProcessJobKind::StartPostgres(spec) = &request.kind {
        match start_postgres_preflight_is_already_running(
            spec.data_dir.as_path(),
            spec.socket_dir.as_path(),
            spec.port,
        ) {
            Ok(true) => {
                let mut event = process_event(
                    ProcessEventKind::StartPostgresNoop,
                    "ok",
                    SeverityText::Info,
                    "start postgres preflight: postgres already running",
                );
                let fields = event.fields_mut();
                fields.append_json_map(
                    process_job_fields(&request.id, request.kind.label()).into_attributes(),
                );
                fields.insert("data_dir", spec.data_dir.display().to_string());
                ctx.log
                    .emit_app_event("process_worker::start_job", event)
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process start-postgres noop log emit failed: {err}"
                        ))
                    })?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Success {
                        id: request.id,
                        job_kind: active_kind(&request.kind),
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
            Ok(false) => {}
            Err(error) => {
                let mut event = process_event(
                    ProcessEventKind::StartPostgresPreflightFailed,
                    "failed",
                    SeverityText::Error,
                    "start postgres preflight failed",
                );
                let fields = event.fields_mut();
                fields.append_json_map(
                    process_job_fields(&request.id, request.kind.label()).into_attributes(),
                );
                fields.insert("error", error.to_string());
                ctx.log
                    .emit_app_event("process_worker::start_job", event)
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "process start-postgres preflight log emit failed: {err}"
                        ))
                    })?;
                transition_to_idle(
                    ctx,
                    JobOutcome::Failure {
                        id: request.id,
                        job_kind: active_kind(&request.kind),
                        error,
                        finished_at: now,
                    },
                    now,
                )?;
                return Ok(());
            }
        }
    }

    let command = match build_command(
        &ctx.config,
        &request.id,
        &request.kind,
        ctx.capture_subprocess_output,
    ) {
        Ok(command) => command,
        Err(error) => {
            let mut event = process_event(
                ProcessEventKind::BuildCommandFailed,
                "failed",
                SeverityText::Error,
                "process build command failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(
                process_job_fields(&request.id, request.kind.label()).into_attributes(),
            );
            fields.insert("error", error.to_string());
            ctx.log
                .emit_app_event("process_worker::start_job", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process build command log emit failed: {err}"))
                })?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: active_kind(&request.kind),
                    error,
                    finished_at: now,
                },
                now,
            )?;
            return Ok(());
        }
    };

    let log_identity = command.log_identity.clone();
    let handle = match ctx.command_runner.spawn(command) {
        Ok(handle) => handle,
        Err(error) => {
            let mut event = process_event(
                ProcessEventKind::SpawnFailed,
                "failed",
                SeverityText::Error,
                "process spawn failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(
                process_job_fields(&request.id, request.kind.label()).into_attributes(),
            );
            fields.insert("error", error.to_string());
            ctx.log
                .emit_app_event("process_worker::start_job", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process spawn log emit failed: {err}"))
                })?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
                    job_kind: active_kind(&request.kind),
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
        kind: active_kind(&request.kind),
        started_at: now,
        deadline_at,
    };

    ctx.active_runtime = Some(ActiveRuntime {
        request,
        deadline_at,
        handle,
        log_identity,
    });
    ctx.state = ProcessState::Running {
        worker: WorkerStatus::Running,
        active,
    };
    let mut event = process_event(
        ProcessEventKind::Started,
        "ok",
        SeverityText::Info,
        "process job started",
    );
    let runtime_fields = ctx
        .active_runtime
        .as_ref()
        .map(|runtime| process_log_identity_fields(&runtime.log_identity).into_attributes())
        .unwrap_or_default();
    event.fields_mut().append_json_map(runtime_fields);
    ctx.log
        .emit_app_event("process_worker::start_job", event)
        .map_err(|err| {
            WorkerError::Message(format!("process job started log emit failed: {err}"))
        })?;
    publish_state(ctx)
}

pub(crate) async fn tick_active_job(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    let mut runtime = match ctx.active_runtime.take() {
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
                    emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone())
                {
                    emit_process_output_emit_failed(ctx, &runtime.log_identity, &line, &err)?;
                }
            }
        }
        Err(err) => {
            let mut event = process_event(
                ProcessEventKind::OutputDrainFailed,
                "failed",
                SeverityText::Warn,
                "process output drain failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(
                process_log_identity_fields(&runtime.log_identity).into_attributes(),
            );
            fields.insert("error", err.to_string());
            ctx.log
                .emit_app_event("process_worker::tick_active_job", event)
                .map_err(|emit_err| {
                    WorkerError::Message(format!(
                        "process output drain log emit failed: {emit_err}"
                    ))
                })?;
        }
    }
    if now.0 >= runtime.deadline_at.0 {
        let mut timeout_event = process_event(
            ProcessEventKind::Timeout,
            "timeout",
            SeverityText::Warn,
            "process job timed out; cancelling",
        );
        timeout_event
            .fields_mut()
            .append_json_map(process_log_identity_fields(&runtime.log_identity).into_attributes());
        ctx.log
            .emit_app_event("process_worker::tick_active_job", timeout_event)
            .map_err(|err| {
                WorkerError::Message(format!("process timeout log emit failed: {err}"))
            })?;
        let cancel_result = runtime.handle.cancel().await;
        match runtime
            .handle
            .drain_output(PROCESS_OUTPUT_DRAIN_MAX_BYTES)
            .await
        {
            Ok(lines) => {
                for line in lines {
                    if let Err(err) =
                        emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone())
                    {
                        emit_process_output_emit_failed(ctx, &runtime.log_identity, &line, &err)?;
                    }
                }
            }
            Err(err) => {
                let mut event = process_event(
                    ProcessEventKind::OutputDrainFailed,
                    "failed",
                    SeverityText::Warn,
                    "process output drain failed",
                );
                let fields = event.fields_mut();
                fields.append_json_map(
                    process_log_identity_fields(&runtime.log_identity).into_attributes(),
                );
                fields.insert("error", err.to_string());
                ctx.log
                    .emit_app_event("process_worker::tick_active_job", event)
                    .map_err(|emit_err| {
                        WorkerError::Message(format!(
                            "process output drain log emit failed: {emit_err}"
                        ))
                    })?;
            }
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
            ctx.active_runtime = Some(runtime);
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
                        if let Err(err) =
                            emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone())
                        {
                            emit_process_output_emit_failed(
                                ctx,
                                &runtime.log_identity,
                                &line,
                                &err,
                            )?;
                        }
                    }
                }
                Err(err) => {
                    let mut event = process_event(
                        ProcessEventKind::OutputDrainFailed,
                        "failed",
                        SeverityText::Warn,
                        "process output drain failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(
                        process_log_identity_fields(&runtime.log_identity).into_attributes(),
                    );
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("process_worker::tick_active_job", event)
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "process output drain log emit failed: {emit_err}"
                            ))
                        })?;
                }
            }
            let job_id = runtime.request.id.clone();
            let outcome = JobOutcome::Success {
                id: job_id.clone(),
                job_kind: active_kind(&runtime.request.kind),
                finished_at: now,
            };
            let mut event = process_event(
                ProcessEventKind::Exited,
                "ok",
                SeverityText::Info,
                "process job exited successfully",
            );
            event.fields_mut().append_json_map(
                process_log_identity_fields(&runtime.log_identity).into_attributes(),
            );
            ctx.log
                .emit_app_event("process_worker::tick_active_job", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process exit log emit failed: {err}"))
                })?;
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
                        if let Err(err) =
                            emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone())
                        {
                            emit_process_output_emit_failed(
                                ctx,
                                &runtime.log_identity,
                                &line,
                                &err,
                            )?;
                        }
                    }
                }
                Err(err) => {
                    let mut event = process_event(
                        ProcessEventKind::OutputDrainFailed,
                        "failed",
                        SeverityText::Warn,
                        "process output drain failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(
                        process_log_identity_fields(&runtime.log_identity).into_attributes(),
                    );
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("process_worker::tick_active_job", event)
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "process output drain log emit failed: {emit_err}"
                            ))
                        })?;
                }
            }
            let exit_error = ProcessError::from_exit(exit);
            let job_id = runtime.request.id.clone();
            let outcome = JobOutcome::Failure {
                id: job_id.clone(),
                job_kind: active_kind(&runtime.request.kind),
                error: exit_error.clone(),
                finished_at: now,
            };
            let mut event = process_event(
                ProcessEventKind::Exited,
                "failed",
                SeverityText::Warn,
                "process job exited unsuccessfully",
            );
            let fields = event.fields_mut();
            fields.append_json_map(
                process_log_identity_fields(&runtime.log_identity).into_attributes(),
            );
            fields.insert("error", exit_error.to_string());
            ctx.log
                .emit_app_event("process_worker::tick_active_job", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process exit log emit failed: {err}"))
                })?;
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
                        if let Err(err) =
                            emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone())
                        {
                            emit_process_output_emit_failed(
                                ctx,
                                &runtime.log_identity,
                                &line,
                                &err,
                            )?;
                        }
                    }
                }
                Err(err) => {
                    let mut event = process_event(
                        ProcessEventKind::OutputDrainFailed,
                        "failed",
                        SeverityText::Warn,
                        "process output drain failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(
                        process_log_identity_fields(&runtime.log_identity).into_attributes(),
                    );
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("process_worker::tick_active_job", event)
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "process output drain log emit failed: {emit_err}"
                            ))
                        })?;
                }
            }
            let job_id = runtime.request.id.clone();
            let outcome = JobOutcome::Failure {
                id: job_id.clone(),
                job_kind: active_kind(&runtime.request.kind),
                error,
                finished_at: now,
            };
            let mut event = process_event(
                ProcessEventKind::PollFailed,
                "failed",
                SeverityText::Error,
                "process job poll failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(
                process_log_identity_fields(&runtime.log_identity).into_attributes(),
            );
            fields.insert("error", outcome_error_string(&outcome));
            ctx.log
                .emit_app_event("process_worker::tick_active_job", event)
                .map_err(|err| {
                    WorkerError::Message(format!("process poll failure log emit failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
    }
}

fn process_log_identity_fields(identity: &ProcessLogIdentity) -> StructuredFields {
    let mut fields = process_job_fields(&identity.job_id, identity.job_kind.as_str());
    fields.insert("binary", identity.binary.clone());
    fields
}

fn emit_process_output_emit_failed(
    ctx: &ProcessWorkerCtx,
    identity: &ProcessLogIdentity,
    line: &ProcessOutputLine,
    error: &crate::logging::LogError,
) -> Result<(), WorkerError> {
    let mut event = process_event(
        ProcessEventKind::OutputEmitFailed,
        "failed",
        SeverityText::Warn,
        "process subprocess output emit failed",
    );
    let fields = event.fields_mut();
    fields.append_json_map(process_log_identity_fields(identity).into_attributes());
    fields.insert(
        "stream",
        match line.stream {
            ProcessOutputStream::Stdout => "stdout",
            ProcessOutputStream::Stderr => "stderr",
        },
    );
    fields.insert("bytes_len", line.bytes.len());
    fields.insert("error", error.to_string());
    ctx.log
        .emit_app_event("process_worker::emit_subprocess_line", event)
        .map_err(|emit_err| {
            WorkerError::Message(format!(
                "process output emit failure log emit failed: {emit_err}"
            ))
        })?;
    Ok(())
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
        ProcessOutputStream::Stdout => SubprocessStream::Stdout,
        ProcessOutputStream::Stderr => SubprocessStream::Stderr,
    };

    log.emit_raw_record(
        SubprocessLineRecord::new(
            crate::logging::LogProducer::PgTool,
            "process_worker",
            identity.job_id.0.clone(),
            identity.job_kind.clone(),
            identity.binary.clone(),
            stream,
            line.bytes,
        )
        .into_raw_record()?,
    )
}

fn transition_to_idle(
    ctx: &mut ProcessWorkerCtx,
    outcome: JobOutcome,
    _now: UnixMillis,
) -> Result<(), WorkerError> {
    ctx.state = ProcessState::Idle {
        worker: WorkerStatus::Running,
        last_outcome: Some(outcome),
    };
    publish_state(ctx)
}

fn publish_state(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    ctx.publisher
        .publish(ctx.state.clone())
        .map_err(|err| WorkerError::Message(format!("process publish failed: {err}")))?;
    Ok(())
}

fn current_time(ctx: &mut ProcessWorkerCtx) -> Result<UnixMillis, WorkerError> {
    (ctx.now)()
}

pub(crate) fn system_now_unix_millis() -> Result<UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

fn timeout_for_kind(kind: &ProcessJobKind, config: &ProcessConfig) -> u64 {
    match kind {
        ProcessJobKind::Bootstrap(spec) => spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms),
        ProcessJobKind::BaseBackup(spec) => spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms),
        ProcessJobKind::PgRewind(spec) => spec.timeout_ms.unwrap_or(config.pg_rewind_timeout_ms),
        ProcessJobKind::Promote(spec) => spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms),
        ProcessJobKind::Demote(spec) => spec.timeout_ms.unwrap_or(config.fencing_timeout_ms),
        ProcessJobKind::StartPostgres(spec) => {
            spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms)
        }
    }
}

fn active_kind(kind: &ProcessJobKind) -> ActiveJobKind {
    match kind {
        ProcessJobKind::Bootstrap(_) => ActiveJobKind::Bootstrap,
        ProcessJobKind::BaseBackup(_) => ActiveJobKind::BaseBackup,
        ProcessJobKind::PgRewind(_) => ActiveJobKind::PgRewind,
        ProcessJobKind::Promote(_) => ActiveJobKind::Promote,
        ProcessJobKind::Demote(_) => ActiveJobKind::Demote,
        ProcessJobKind::StartPostgres(_) => ActiveJobKind::StartPostgres,
    }
}

fn build_command(
    config: &ProcessConfig,
    job_id: &JobId,
    kind: &ProcessJobKind,
    capture_output: bool,
) -> Result<ProcessCommandSpec, ProcessError> {
    match kind {
        ProcessJobKind::Bootstrap(spec) => {
            validate_non_empty_path("bootstrap.data_dir", &spec.data_dir)?;
            if spec.superuser_username.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "bootstrap.superuser_username must not be empty".to_string(),
                ));
            }
            let program = config.binaries.initdb.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-A".to_string(),
                    "trust".to_string(),
                    "-U".to_string(),
                    spec.superuser_username.clone(),
                ],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::BaseBackup(spec) => {
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
            let program = config.binaries.pg_basebackup.clone();
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
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::PgRewind(spec) => {
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
            let program = config.binaries.pg_rewind.clone();
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
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::Promote(spec) => {
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
            let program = config.binaries.pg_ctl.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args,
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::Demote(spec) => {
            validate_non_empty_path("demote.data_dir", &spec.data_dir)?;
            let program = config.binaries.pg_ctl.clone();
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
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::StartPostgres(spec) => {
            validate_non_empty_path("start_postgres.data_dir", &spec.data_dir)?;
            validate_non_empty_path("start_postgres.config_file", &spec.config_file)?;
            validate_non_empty_path("start_postgres.log_file", &spec.log_file)?;
            let wait_seconds = spec.wait_seconds.unwrap_or(PG_CTL_DEFAULT_WAIT_SECONDS);
            let option_tokens = vec![
                "-c".to_string(),
                format!("config_file={}", spec.config_file.display()),
            ];
            let options = render_pg_ctl_option_string(&option_tokens)?;
            let program = config.binaries.pg_ctl.clone();
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
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
    }
}

fn role_auth_env(auth: &RoleAuthConfig) -> Vec<ProcessEnvVar> {
    match auth {
        RoleAuthConfig::Tls => Vec::new(),
        RoleAuthConfig::Password { password } => vec![ProcessEnvVar {
            key: "PGPASSWORD".to_string(),
            value: ProcessEnvValue::Secret(password.clone()),
        }],
    }
}

fn job_kind_label(kind: &ProcessJobKind) -> &'static str {
    match kind {
        ProcessJobKind::Bootstrap(_) => "bootstrap",
        ProcessJobKind::BaseBackup(_) => "basebackup",
        ProcessJobKind::PgRewind(_) => "pg_rewind",
        ProcessJobKind::Promote(_) => "promote",
        ProcessJobKind::Demote(_) => "demote",
        ProcessJobKind::StartPostgres(_) => "start_postgres",
    }
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
