use std::{collections::BTreeMap, process::Stdio};

use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, Command},
    sync::mpsc::error::TryRecvError,
};

use crate::{
    config::{ProcessConfig, RoleAuthConfig},
    logging::{
        EventMeta, LogHandle, LogParser, LogProducer, LogRecord, LogSource, LogTransport,
        SeverityText,
    },
    pginfo::state::render_pg_conninfo,
    state::{JobId, UnixMillis, WorkerError, WorkerStatus},
};

use super::{
    jobs::{
        ActiveJob, ActiveJobKind, ProcessCommandSpec, ProcessEnvValue, ProcessEnvVar,
        ProcessError, ProcessExit, ProcessHandle, ProcessLogIdentity, ProcessOutputLine,
        ProcessOutputStream,
    },
    state::{
        ActiveRuntime, JobOutcome, ProcessJobKind, ProcessJobRejection, ProcessJobRequest,
        ProcessState, ProcessWorkerCtx,
    },
};

#[derive(Default)]
pub(crate) struct TokioCommandRunner;

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
        Box<dyn std::future::Future<Output = Result<Vec<super::jobs::ProcessOutputLine>, ProcessError>> + Send + 'a>,
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

    let mut buf = vec![0u8; 8192];
    loop {
        if *remaining == 0 {
            break;
        }
        let chunk_len = buf.len().min(*remaining);
        let read_result =
            tokio::time::timeout(std::time::Duration::from_millis(1), handle.read(&mut buf[..chunk_len]))
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
                    out.push(super::jobs::ProcessOutputLine { stream, bytes: line });
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

pub(crate) fn can_accept_job(state: &ProcessState) -> bool {
    matches!(state, ProcessState::Idle { .. })
}

pub(crate) async fn run(mut ctx: ProcessWorkerCtx) -> Result<(), WorkerError> {
    let mut attrs = BTreeMap::new();
    attrs.insert(
        "capture_subprocess_output".to_string(),
        serde_json::Value::Bool(ctx.capture_subprocess_output),
    );
    ctx.log
        .emit_event(
            SeverityText::Debug,
            "process worker run started",
            "process_worker::run",
            EventMeta::new("process.worker.run_started", "process", "ok"),
            attrs,
        )
        .map_err(|err| WorkerError::Message(format!("process worker start log emit failed: {err}")))?;
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    match ctx.inbox.try_recv() {
        Ok(request) => {
            let mut attrs = BTreeMap::new();
            attrs.insert(
                "job_id".to_string(),
                serde_json::Value::String(request.id.0.clone()),
            );
            attrs.insert(
                "job_kind".to_string(),
                serde_json::Value::String(request.kind.label().to_string()),
            );
            ctx.log
                .emit_event(
                    SeverityText::Debug,
                    "process job request received",
                    "process_worker::step_once",
                    EventMeta::new("process.worker.request_received", "process", "ok"),
                    attrs,
                )
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
                    .emit_event(
                        SeverityText::Warn,
                        "process worker inbox disconnected",
                        "process_worker::step_once",
                        EventMeta::new("process.worker.inbox_disconnected", "process", "failed"),
                        BTreeMap::new(),
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
        let mut attrs = BTreeMap::new();
        attrs.insert(
            "job_id".to_string(),
            serde_json::Value::String(ctx.last_rejection.as_ref().map(|r| r.id.0.clone()).unwrap_or_else(|| "unknown".to_string())),
        );
        attrs.insert(
            "job_kind".to_string(),
            serde_json::Value::String(request.kind.label().to_string()),
        );
        ctx.log
            .emit_event(
                SeverityText::Warn,
                "process worker busy; rejecting job",
                "process_worker::start_job",
                EventMeta::new("process.worker.busy_reject", "process", "failed"),
                attrs,
            )
            .map_err(|err| {
                WorkerError::Message(format!("process busy reject log emit failed: {err}"))
            })?;
        return Ok(());
    }

    let now = current_time(ctx)?;
    let timeout_ms = timeout_for_kind(&request.kind, &ctx.config);
    let deadline_at = UnixMillis(now.0.saturating_add(timeout_ms));

    let command = match build_command(
        &ctx.config,
        &request.id,
        &request.kind,
        ctx.capture_subprocess_output,
    ) {
        Ok(command) => command,
        Err(error) => {
            let mut attrs = BTreeMap::new();
            attrs.insert(
                "job_id".to_string(),
                serde_json::Value::String(request.id.0.clone()),
            );
            attrs.insert(
                "job_kind".to_string(),
                serde_json::Value::String(request.kind.label().to_string()),
            );
            attrs.insert(
                "error".to_string(),
                serde_json::Value::String(error.to_string()),
            );
            ctx.log
                .emit_event(
                    SeverityText::Error,
                    "process build command failed",
                    "process_worker::start_job",
                    EventMeta::new("process.job.build_command_failed", "process", "failed"),
                    attrs,
                )
                .map_err(|err| {
                    WorkerError::Message(format!("process build command log emit failed: {err}"))
                })?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
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
            let mut attrs = BTreeMap::new();
            attrs.insert(
                "job_id".to_string(),
                serde_json::Value::String(request.id.0.clone()),
            );
            attrs.insert(
                "job_kind".to_string(),
                serde_json::Value::String(request.kind.label().to_string()),
            );
            attrs.insert(
                "error".to_string(),
                serde_json::Value::String(error.to_string()),
            );
            ctx.log
                .emit_event(
                    SeverityText::Error,
                    "process spawn failed",
                    "process_worker::start_job",
                    EventMeta::new("process.job.spawn_failed", "process", "failed"),
                    attrs,
                )
                .map_err(|err| {
                    WorkerError::Message(format!("process spawn log emit failed: {err}"))
                })?;
            transition_to_idle(
                ctx,
                JobOutcome::Failure {
                    id: request.id,
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
    let mut attrs = BTreeMap::new();
    attrs.insert(
        "job_id".to_string(),
        serde_json::Value::String(ctx.active_runtime.as_ref().map(|rt| rt.request.id.0.clone()).unwrap_or_else(|| "unknown".to_string())),
    );
    attrs.insert(
        "job_kind".to_string(),
        serde_json::Value::String(ctx.active_runtime.as_ref().map(|rt| rt.request.kind.label().to_string()).unwrap_or_else(|| "unknown".to_string())),
    );
    attrs.insert(
        "binary".to_string(),
        serde_json::Value::String(ctx.active_runtime.as_ref().map(|rt| rt.log_identity.binary.clone()).unwrap_or_else(|| "unknown".to_string())),
    );
    ctx.log
        .emit_event(
            SeverityText::Info,
            "process job started",
            "process_worker::start_job",
            EventMeta::new("process.job.started", "process", "ok"),
            attrs,
        )
        .map_err(|err| WorkerError::Message(format!("process job started log emit failed: {err}")))?;
    publish_state(ctx, now)
}

pub(crate) async fn tick_active_job(ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    let mut runtime = match ctx.active_runtime.take() {
        Some(runtime) => runtime,
        None => return Ok(()),
    };

    let now = current_time(ctx)?;
    match runtime.handle.drain_output(256 * 1024).await {
        Ok(lines) => {
            for line in lines {
                if let Err(err) = emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone()) {
                    emit_process_output_emit_failed(ctx, &runtime.log_identity, &line, &err)?;
                }
            }
        }
        Err(err) => {
            let mut attrs = process_log_identity_attrs(&runtime.log_identity);
            attrs.insert("error".to_string(), serde_json::Value::String(err.to_string()));
            ctx.log
                .emit_event(
                    SeverityText::Warn,
                    "process output drain failed",
                    "process_worker::tick_active_job",
                    EventMeta::new("process.worker.output_drain_failed", "process", "failed"),
                    attrs,
                )
                .map_err(|emit_err| {
                    WorkerError::Message(format!("process output drain log emit failed: {emit_err}"))
                })?;
        }
    }
    if now.0 >= runtime.deadline_at.0 {
        let mut timeout_attrs = process_log_identity_attrs(&runtime.log_identity);
        timeout_attrs.insert(
            "job_id".to_string(),
            serde_json::Value::String(runtime.request.id.0.clone()),
        );
        ctx.log
            .emit_event(
                SeverityText::Warn,
                "process job timed out; cancelling",
                "process_worker::tick_active_job",
                EventMeta::new("process.job.timeout", "process", "timeout"),
                timeout_attrs,
            )
            .map_err(|err| {
                WorkerError::Message(format!("process timeout log emit failed: {err}"))
            })?;
        let cancel_result = runtime.handle.cancel().await;
        match runtime.handle.drain_output(256 * 1024).await {
            Ok(lines) => {
                for line in lines {
                    if let Err(err) = emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone()) {
                        emit_process_output_emit_failed(ctx, &runtime.log_identity, &line, &err)?;
                    }
                }
            }
            Err(err) => {
                let mut attrs = process_log_identity_attrs(&runtime.log_identity);
                attrs.insert("error".to_string(), serde_json::Value::String(err.to_string()));
                ctx.log
                    .emit_event(
                        SeverityText::Warn,
                        "process output drain failed",
                        "process_worker::tick_active_job",
                        EventMeta::new("process.worker.output_drain_failed", "process", "failed"),
                        attrs,
                    )
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
                finished_at: now,
            },
            Err(error) => JobOutcome::Failure {
                id: runtime.request.id,
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
            match runtime.handle.drain_output(256 * 1024).await {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone()) {
                            emit_process_output_emit_failed(ctx, &runtime.log_identity, &line, &err)?;
                        }
                    }
                }
                Err(err) => {
                    let mut attrs = process_log_identity_attrs(&runtime.log_identity);
                    attrs.insert("error".to_string(), serde_json::Value::String(err.to_string()));
                    ctx.log
                        .emit_event(
                            SeverityText::Warn,
                            "process output drain failed",
                            "process_worker::tick_active_job",
                            EventMeta::new("process.worker.output_drain_failed", "process", "failed"),
                            attrs,
                        )
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
                finished_at: now,
            };
            let mut attrs = process_log_identity_attrs(&runtime.log_identity);
            attrs.insert(
                "job_id".to_string(),
                serde_json::Value::String(job_id.0.clone()),
            );
            ctx.log
                .emit_event(
                    SeverityText::Info,
                    "process job exited successfully",
                    "process_worker::tick_active_job",
                    EventMeta::new("process.job.exited", "process", "ok"),
                    attrs,
                )
                .map_err(|err| {
                    WorkerError::Message(format!("process exit log emit failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
        Ok(Some(exit)) => {
            match runtime.handle.drain_output(256 * 1024).await {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone()) {
                            emit_process_output_emit_failed(ctx, &runtime.log_identity, &line, &err)?;
                        }
                    }
                }
                Err(err) => {
                    let mut attrs = process_log_identity_attrs(&runtime.log_identity);
                    attrs.insert("error".to_string(), serde_json::Value::String(err.to_string()));
                    ctx.log
                        .emit_event(
                            SeverityText::Warn,
                            "process output drain failed",
                            "process_worker::tick_active_job",
                            EventMeta::new("process.worker.output_drain_failed", "process", "failed"),
                            attrs,
                        )
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
                error: exit_error.clone(),
                finished_at: now,
            };
            let mut attrs = process_log_identity_attrs(&runtime.log_identity);
            attrs.insert(
                "job_id".to_string(),
                serde_json::Value::String(job_id.0.clone()),
            );
            attrs.insert(
                "error".to_string(),
                serde_json::Value::String(exit_error.to_string()),
            );
            ctx.log
                .emit_event(
                    SeverityText::Warn,
                    "process job exited unsuccessfully",
                    "process_worker::tick_active_job",
                    EventMeta::new("process.job.exited", "process", "failed"),
                    attrs,
                )
                .map_err(|err| {
                    WorkerError::Message(format!("process exit log emit failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
        Err(error) => {
            match runtime.handle.drain_output(256 * 1024).await {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = emit_subprocess_line(&ctx.log, &runtime.log_identity, line.clone()) {
                            emit_process_output_emit_failed(ctx, &runtime.log_identity, &line, &err)?;
                        }
                    }
                }
                Err(err) => {
                    let mut attrs = process_log_identity_attrs(&runtime.log_identity);
                    attrs.insert("error".to_string(), serde_json::Value::String(err.to_string()));
                    ctx.log
                        .emit_event(
                            SeverityText::Warn,
                            "process output drain failed",
                            "process_worker::tick_active_job",
                            EventMeta::new("process.worker.output_drain_failed", "process", "failed"),
                            attrs,
                        )
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
                error,
                finished_at: now,
            };
            let mut attrs = process_log_identity_attrs(&runtime.log_identity);
            attrs.insert(
                "job_id".to_string(),
                serde_json::Value::String(job_id.0.clone()),
            );
            attrs.insert(
                "error".to_string(),
                serde_json::Value::String(outcome_error_string(&outcome)),
            );
            ctx.log
                .emit_event(
                    SeverityText::Error,
                    "process job poll failed",
                    "process_worker::tick_active_job",
                    EventMeta::new("process.job.poll_failed", "process", "failed"),
                    attrs,
                )
                .map_err(|err| {
                    WorkerError::Message(format!("process poll failure log emit failed: {err}"))
                })?;
            transition_to_idle(ctx, outcome, now)
        }
    }
}

fn process_log_identity_attrs(identity: &ProcessLogIdentity) -> BTreeMap<String, serde_json::Value> {
    let mut attrs = BTreeMap::new();
    attrs.insert(
        "job_id".to_string(),
        serde_json::Value::String(identity.job_id.0.clone()),
    );
    attrs.insert(
        "job_kind".to_string(),
        serde_json::Value::String(identity.job_kind.clone()),
    );
    attrs.insert(
        "binary".to_string(),
        serde_json::Value::String(identity.binary.clone()),
    );
    attrs
}

fn emit_process_output_emit_failed(
    ctx: &ProcessWorkerCtx,
    identity: &ProcessLogIdentity,
    line: &ProcessOutputLine,
    error: &crate::logging::LogError,
) -> Result<(), WorkerError> {
    let mut attrs = process_log_identity_attrs(identity);
    attrs.insert(
        "stream".to_string(),
        serde_json::Value::String(match line.stream {
            ProcessOutputStream::Stdout => "stdout".to_string(),
            ProcessOutputStream::Stderr => "stderr".to_string(),
        }),
    );
    attrs.insert(
        "bytes_len".to_string(),
        serde_json::Value::Number(serde_json::Number::from(line.bytes.len() as u64)),
    );
    attrs.insert(
        "error".to_string(),
        serde_json::Value::String(error.to_string()),
    );
    ctx.log
        .emit_event(
            SeverityText::Warn,
            "process subprocess output emit failed",
            "process_worker::emit_subprocess_line",
            EventMeta::new("process.worker.output_emit_failed", "process", "failed"),
            attrs,
        )
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
    let (transport, severity) = match line.stream {
        ProcessOutputStream::Stdout => (LogTransport::ChildStdout, SeverityText::Info),
        ProcessOutputStream::Stderr => (LogTransport::ChildStderr, SeverityText::Warn),
    };

    let (message, raw_bytes_hex) = match String::from_utf8(line.bytes) {
        Ok(message) => (message, None),
        Err(err) => {
            let bytes = err.into_bytes();
            let hex = hex_encode(bytes.as_slice());
            (format!("non_utf8_bytes_hex={hex}"), Some(hex))
        }
    };

    let mut record = LogRecord::new(
        crate::logging::system_now_unix_millis(),
        log.hostname().to_string(),
        severity,
        message,
        LogSource {
            producer: LogProducer::PgTool,
            transport,
            parser: LogParser::Raw,
            origin: "process_worker".to_string(),
        },
    );

    record.attributes.insert(
        "job_id".to_string(),
        serde_json::Value::String(identity.job_id.0.clone()),
    );
    record.attributes.insert(
        "job_kind".to_string(),
        serde_json::Value::String(identity.job_kind.clone()),
    );
    record.attributes.insert(
        "binary".to_string(),
        serde_json::Value::String(identity.binary.clone()),
    );
    record.attributes.insert(
        "stream".to_string(),
        serde_json::Value::String(match line.stream {
            ProcessOutputStream::Stdout => "stdout".to_string(),
            ProcessOutputStream::Stderr => "stderr".to_string(),
        }),
    );
    if let Some(hex) = raw_bytes_hex {
        record.attributes.insert(
            "raw_bytes_hex".to_string(),
            serde_json::Value::String(hex),
        );
    }

    log.emit_record(&record)
}

fn hex_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for b in bytes {
        out.push(TABLE[(b >> 4) as usize] as char);
        out.push(TABLE[(b & 0x0f) as usize] as char);
    }
    out
}

fn transition_to_idle(
    ctx: &mut ProcessWorkerCtx,
    outcome: JobOutcome,
    now: UnixMillis,
) -> Result<(), WorkerError> {
    ctx.state = ProcessState::Idle {
        worker: WorkerStatus::Running,
        last_outcome: Some(outcome),
    };
    publish_state(ctx, now)
}

fn publish_state(ctx: &mut ProcessWorkerCtx, now: UnixMillis) -> Result<(), WorkerError> {
    ctx.publisher
        .publish(ctx.state.clone(), now)
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

pub(crate) fn timeout_for_kind(kind: &ProcessJobKind, config: &ProcessConfig) -> u64 {
    match kind {
        ProcessJobKind::Bootstrap(spec) => spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms),
        ProcessJobKind::BaseBackup(spec) => spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms),
        ProcessJobKind::PgRewind(spec) => spec.timeout_ms.unwrap_or(config.pg_rewind_timeout_ms),
        ProcessJobKind::Fencing(spec) => spec.timeout_ms.unwrap_or(config.fencing_timeout_ms),
        ProcessJobKind::Promote(spec) => spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms),
        ProcessJobKind::Demote(spec) => spec.timeout_ms.unwrap_or(config.fencing_timeout_ms),
        ProcessJobKind::StartPostgres(spec) => {
            spec.timeout_ms.unwrap_or(config.bootstrap_timeout_ms)
        }
        ProcessJobKind::PgBackRestVersion(_) => config.backup_timeout_ms,
        ProcessJobKind::PgBackRestInfo(spec) => spec.timeout_ms.unwrap_or(config.backup_timeout_ms),
        ProcessJobKind::PgBackRestCheck(spec) => spec.timeout_ms.unwrap_or(config.backup_timeout_ms),
        ProcessJobKind::PgBackRestBackup(spec) => spec.timeout_ms.unwrap_or(config.backup_timeout_ms),
        ProcessJobKind::PgBackRestRestore(spec) => spec.timeout_ms.unwrap_or(config.backup_timeout_ms),
        ProcessJobKind::PgBackRestArchivePush(spec) => {
            spec.timeout_ms.unwrap_or(config.backup_timeout_ms)
        }
        ProcessJobKind::PgBackRestArchiveGet(spec) => {
            spec.timeout_ms.unwrap_or(config.backup_timeout_ms)
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
        ProcessJobKind::Fencing(_) => ActiveJobKind::Fencing,
        ProcessJobKind::PgBackRestVersion(_) => ActiveJobKind::PgBackRestVersion,
        ProcessJobKind::PgBackRestInfo(_) => ActiveJobKind::PgBackRestInfo,
        ProcessJobKind::PgBackRestCheck(_) => ActiveJobKind::PgBackRestCheck,
        ProcessJobKind::PgBackRestBackup(_) => ActiveJobKind::PgBackRestBackup,
        ProcessJobKind::PgBackRestRestore(_) => ActiveJobKind::PgBackRestRestore,
        ProcessJobKind::PgBackRestArchivePush(_) => ActiveJobKind::PgBackRestArchivePush,
        ProcessJobKind::PgBackRestArchiveGet(_) => ActiveJobKind::PgBackRestArchiveGet,
    }
}

pub(crate) fn build_command(
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
            let program = config.binaries.pg_basebackup.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-h".to_string(),
                    spec.source.conninfo.host.clone(),
                    "-p".to_string(),
                    spec.source.conninfo.port.to_string(),
                    "-U".to_string(),
                    spec.source.conninfo.user.clone(),
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-Fp".to_string(),
                    "-Xs".to_string(),
                    "-R".to_string(),
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
            validate_non_empty_path("start_postgres.socket_dir", &spec.socket_dir)?;
            validate_non_empty_path("start_postgres.log_file", &spec.log_file)?;
            if spec.host.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "start_postgres.host must not be empty".to_string(),
                ));
            }
            let wait_seconds = spec.wait_seconds.unwrap_or(30);
            let mut option_tokens = vec![
                "-h".to_string(),
                spec.host.clone(),
                "-p".to_string(),
                spec.port.to_string(),
                "-k".to_string(),
                spec.socket_dir.display().to_string(),
            ];
            for (key, value) in &spec.extra_postgres_settings {
                validate_postgres_setting("start_postgres.extra_postgres_settings", key, value)?;
                option_tokens.push("-c".to_string());
                option_tokens.push(format!("{key}={value}"));
            }
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
        ProcessJobKind::Fencing(spec) => {
            validate_non_empty_path("fencing.data_dir", &spec.data_dir)?;
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
        ProcessJobKind::PgBackRestVersion(_spec) => {
            let program = config.binaries.pgbackrest.clone().ok_or_else(|| {
                ProcessError::InvalidSpec("process.binaries.pgbackrest must be configured".to_string())
            })?;
            validate_non_empty_path("process.binaries.pgbackrest", &program)?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec!["--version".to_string()],
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::PgBackRestInfo(spec) => {
            let program = config.binaries.pgbackrest.clone().ok_or_else(|| {
                ProcessError::InvalidSpec("process.binaries.pgbackrest must be configured".to_string())
            })?;
            validate_non_empty_path("process.binaries.pgbackrest", &program)?;
            let template = crate::backup::pgbackrest::render(
                crate::backup::BackupOperation::Info(crate::backup::InfoInput {
                    stanza: spec.stanza.clone(),
                    repo: spec.repo.clone(),
                    options: spec.options.clone(),
                }),
            )
            .map_err(|err| ProcessError::InvalidSpec(format!("pgbackrest info: {err}")))?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: template.args,
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::PgBackRestCheck(spec) => {
            let program = config.binaries.pgbackrest.clone().ok_or_else(|| {
                ProcessError::InvalidSpec("process.binaries.pgbackrest must be configured".to_string())
            })?;
            validate_non_empty_path("process.binaries.pgbackrest", &program)?;
            let template = crate::backup::pgbackrest::render(
                crate::backup::BackupOperation::Check(crate::backup::CheckInput {
                    stanza: spec.stanza.clone(),
                    repo: spec.repo.clone(),
                    options: spec.options.clone(),
                }),
            )
            .map_err(|err| ProcessError::InvalidSpec(format!("pgbackrest check: {err}")))?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: template.args,
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::PgBackRestBackup(spec) => {
            let program = config.binaries.pgbackrest.clone().ok_or_else(|| {
                ProcessError::InvalidSpec("process.binaries.pgbackrest must be configured".to_string())
            })?;
            validate_non_empty_path("process.binaries.pgbackrest", &program)?;
            let template = crate::backup::pgbackrest::render(
                crate::backup::BackupOperation::Backup(crate::backup::BackupInput {
                    stanza: spec.stanza.clone(),
                    repo: spec.repo.clone(),
                    options: spec.options.clone(),
                }),
            )
            .map_err(|err| ProcessError::InvalidSpec(format!("pgbackrest backup: {err}")))?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: template.args,
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::PgBackRestRestore(spec) => {
            let program = config.binaries.pgbackrest.clone().ok_or_else(|| {
                ProcessError::InvalidSpec("process.binaries.pgbackrest must be configured".to_string())
            })?;
            validate_non_empty_path("process.binaries.pgbackrest", &program)?;
            let template = crate::backup::pgbackrest::render(
                crate::backup::BackupOperation::Restore(crate::backup::RestoreInput {
                    stanza: spec.stanza.clone(),
                    repo: spec.repo.clone(),
                    pg1_path: spec.pg1_path.clone(),
                    options: spec.options.clone(),
                }),
            )
            .map_err(|err| ProcessError::InvalidSpec(format!("pgbackrest restore: {err}")))?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: template.args,
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::PgBackRestArchivePush(spec) => {
            let program = config.binaries.pgbackrest.clone().ok_or_else(|| {
                ProcessError::InvalidSpec("process.binaries.pgbackrest must be configured".to_string())
            })?;
            validate_non_empty_path("process.binaries.pgbackrest", &program)?;
            let template = crate::backup::pgbackrest::render(
                crate::backup::BackupOperation::ArchivePush(crate::backup::ArchivePushInput {
                    stanza: spec.stanza.clone(),
                    repo: spec.repo.clone(),
                    pg1_path: spec.pg1_path.clone(),
                    wal_path: spec.wal_path.clone(),
                    options: spec.options.clone(),
                }),
            )
            .map_err(|err| ProcessError::InvalidSpec(format!("pgbackrest archive-push: {err}")))?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: template.args,
                env: Vec::new(),
                capture_output,
                log_identity: ProcessLogIdentity {
                    job_id: job_id.clone(),
                    job_kind: job_kind_label(kind).to_string(),
                    binary: binary_label(program.as_path()),
                },
            })
        }
        ProcessJobKind::PgBackRestArchiveGet(spec) => {
            let program = config.binaries.pgbackrest.clone().ok_or_else(|| {
                ProcessError::InvalidSpec("process.binaries.pgbackrest must be configured".to_string())
            })?;
            validate_non_empty_path("process.binaries.pgbackrest", &program)?;
            let template = crate::backup::pgbackrest::render(
                crate::backup::BackupOperation::ArchiveGet(crate::backup::ArchiveGetInput {
                    stanza: spec.stanza.clone(),
                    repo: spec.repo.clone(),
                    pg1_path: spec.pg1_path.clone(),
                    wal_segment: spec.wal_segment.clone(),
                    destination_path: spec.destination_path.clone(),
                    options: spec.options.clone(),
                }),
            )
            .map_err(|err| ProcessError::InvalidSpec(format!("pgbackrest archive-get: {err}")))?;
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: template.args,
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
        ProcessJobKind::Fencing(_) => "fencing",
        ProcessJobKind::PgBackRestVersion(_) => "pgbackrest_version",
        ProcessJobKind::PgBackRestInfo(_) => "pgbackrest_info",
        ProcessJobKind::PgBackRestCheck(_) => "pgbackrest_check",
        ProcessJobKind::PgBackRestBackup(_) => "pgbackrest_backup",
        ProcessJobKind::PgBackRestRestore(_) => "pgbackrest_restore",
        ProcessJobKind::PgBackRestArchivePush(_) => "pgbackrest_archive_push",
        ProcessJobKind::PgBackRestArchiveGet(_) => "pgbackrest_archive_get",
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

fn validate_postgres_setting(
    field: &str,
    key: &str,
    value: &str,
) -> Result<(), ProcessError> {
    if key.trim().is_empty() {
        return Err(ProcessError::InvalidSpec(format!(
            "{field} postgres setting key must not be empty"
        )));
    }
    for ch in key.chars() {
        if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '-') {
            return Err(ProcessError::InvalidSpec(format!(
                "{field} invalid postgres setting key `{key}`"
            )));
        }
    }

    if value.contains('\0') || value.contains('\n') || value.contains('\r') {
        return Err(ProcessError::InvalidSpec(format!(
            "{field} postgres setting `{key}` contains invalid characters"
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
    use std::{collections::VecDeque, fs, path::PathBuf, time::Duration};

    use tokio::{
        process::Command,
        sync::mpsc,
        time::{sleep, Instant},
    };

    use crate::{
        config::{BinaryPaths, InlineOrPath, ProcessConfig, RoleAuthConfig, SecretSource},
        logging::{LogHandle, LogSink, SeverityText, TestSink},
        pginfo::state::{PgConnInfo, PgSslMode},
        process::{
            jobs::{
                ActiveJob, BaseBackupSpec, BootstrapSpec, DemoteSpec, FencingSpec,
                NoopCommandRunner, PgRewindSpec, ProcessCommandRunner, ProcessError, ProcessExit,
                ProcessEnvValue, ProcessHandle, PromoteSpec, ReplicatorSourceConn,
                RewinderSourceConn, ShutdownMode, StartPostgresSpec,
            },
            state::{
                JobOutcome, ProcessJobKind, ProcessJobRequest, ProcessState, ProcessWorkerCtx,
            },
            worker::{
                build_command, can_accept_job, start_job, step_once,
                tick_active_job, TokioCommandRunner,
            },
        },
        state::{new_state_channel, JobId, UnixMillis, WorkerError, WorkerStatus},
        test_harness::{
            binaries::require_pg16_process_binaries_for_real_tests, namespace::NamespaceGuard,
            ports::allocate_ports,
        },
    };

    fn test_log_handle() -> (LogHandle, std::sync::Arc<TestSink>) {
        let sink = std::sync::Arc::new(TestSink::default());
        let sink_dyn: std::sync::Arc<dyn LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
    }

    struct FakeHandle {
        polls: VecDeque<Result<Option<ProcessExit>, ProcessError>>,
        cancel_result: Result<(), ProcessError>,
    }

    impl ProcessHandle for FakeHandle {
        fn poll_exit(&mut self) -> Result<Option<ProcessExit>, ProcessError> {
            match self.polls.pop_front() {
                Some(result) => result,
                None => Ok(None),
            }
        }

        fn drain_output<'a>(
            &'a mut self,
            _max_bytes: usize,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<Vec<crate::process::jobs::ProcessOutputLine>, ProcessError>,
                    > + Send
                    + 'a,
            >,
        > {
            Box::pin(async move { Ok(Vec::new()) })
        }

        fn cancel<'a>(
            &'a mut self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), ProcessError>> + Send + 'a>,
        > {
            let result = self.cancel_result.clone();
            Box::pin(async move { result })
        }
    }

    struct FakeRunner {
        spawn_results: VecDeque<Result<FakeHandle, ProcessError>>,
    }

    impl ProcessCommandRunner for FakeRunner {
        fn spawn(
            &mut self,
            _spec: crate::process::jobs::ProcessCommandSpec,
        ) -> Result<Box<dyn ProcessHandle>, ProcessError> {
            match self.spawn_results.pop_front() {
                Some(Ok(handle)) => Ok(Box::new(handle)),
                Some(Err(err)) => Err(err),
                None => Err(ProcessError::InvalidSpec(
                    "fake runner exhausted spawn queue".to_string(),
                )),
            }
        }
    }

    fn sample_config() -> ProcessConfig {
        ProcessConfig {
            pg_rewind_timeout_ms: 1_000,
            bootstrap_timeout_ms: 1_000,
            fencing_timeout_ms: 1_000,
            backup_timeout_ms: 1_000,
            binaries: BinaryPaths {
                postgres: PathBuf::from("/usr/bin/postgres"),
                pg_ctl: PathBuf::from("/usr/bin/pg_ctl"),
                pg_rewind: PathBuf::from("/usr/bin/pg_rewind"),
                initdb: PathBuf::from("/usr/bin/initdb"),
                pg_basebackup: PathBuf::from("/usr/bin/pg_basebackup"),
                psql: PathBuf::from("/usr/bin/psql"),
                pgbackrest: Some(PathBuf::from("/usr/bin/pgbackrest")),
            },
        }
    }

    fn sample_start_spec() -> StartPostgresSpec {
        use std::collections::BTreeMap;

        StartPostgresSpec {
            data_dir: PathBuf::from("/tmp/node/data"),
            host: "127.0.0.1".to_string(),
            port: 5544,
            socket_dir: PathBuf::from("/tmp/node/socket"),
            log_file: PathBuf::from("/tmp/node/postgres.log"),
            extra_postgres_settings: BTreeMap::new(),
            wait_seconds: Some(1),
            timeout_ms: Some(1_000),
        }
    }

    fn sample_rewind_conninfo() -> PgConnInfo {
        PgConnInfo {
            host: "127.0.0.1".to_string(),
            port: 9,
            user: "postgres".to_string(),
            dbname: "postgres".to_string(),
            application_name: None,
            connect_timeout_s: None,
            ssl_mode: PgSslMode::Prefer,
            options: None,
        }
    }

    #[test]
    fn build_command_basebackup_uses_pg_basebackup_binary_and_args() {
        let config = sample_config();
        let command = build_command(
            &config,
            &JobId("job-test".to_string()),
            &ProcessJobKind::BaseBackup(BaseBackupSpec {
                data_dir: PathBuf::from("/tmp/node/data"),
                source: ReplicatorSourceConn {
                    conninfo: PgConnInfo {
                        host: "10.0.0.12".to_string(),
                        port: 5433,
                        user: "replicator".to_string(),
                        dbname: "postgres".to_string(),
                        application_name: None,
                        connect_timeout_s: None,
                        ssl_mode: PgSslMode::Prefer,
                        options: None,
                    },
                    auth: RoleAuthConfig::Tls,
                },
                timeout_ms: Some(30_000),
            }),
            false,
        );

        assert!(command.is_ok());
        if let Ok(spec) = command {
            assert_eq!(spec.program, config.binaries.pg_basebackup);
            assert!(spec.env.is_empty());
            assert_eq!(
                spec.args,
                vec![
                    "-h",
                    "10.0.0.12",
                    "-p",
                    "5433",
                    "-U",
                    "replicator",
                    "-D",
                    "/tmp/node/data",
                    "-Fp",
                    "-Xs",
                    "-R",
                ]
            );
        }
    }

    #[test]
    fn build_command_basebackup_sets_pgpassword_env_for_password_auth() -> Result<(), String> {
        let config = sample_config();
        let spec = build_command(
            &config,
            &JobId("job-test".to_string()),
            &ProcessJobKind::BaseBackup(BaseBackupSpec {
                data_dir: PathBuf::from("/tmp/node/data"),
                source: ReplicatorSourceConn {
                    conninfo: PgConnInfo {
                        host: "10.0.0.12".to_string(),
                        port: 5433,
                        user: "replicator".to_string(),
                        dbname: "postgres".to_string(),
                        application_name: None,
                        connect_timeout_s: None,
                        ssl_mode: PgSslMode::Prefer,
                        options: None,
                    },
                    auth: RoleAuthConfig::Password {
                        password: SecretSource(InlineOrPath::Inline {
                            content: "secret\n".to_string(),
                        }),
                    },
                },
                timeout_ms: Some(30_000),
            }),
            false,
        )
        .map_err(|err| format!("build_command failed: {err}"))?;

        if spec.env.len() != 1 {
            return Err(format!("expected 1 env var, got {}", spec.env.len()));
        }
        if spec.env[0].key.as_str() != "PGPASSWORD" {
            return Err(format!("expected env key PGPASSWORD, got {}", spec.env[0].key));
        }
        match &spec.env[0].value {
            ProcessEnvValue::Secret(secret) => match &secret.0 {
                InlineOrPath::Inline { content } => {
                    if content.as_str() != "secret\n" {
                        return Err(format!("unexpected inline secret content: {content:?}"));
                    }
                }
                other => return Err(format!("expected inline secret source, got: {other:?}")),
            },
        }
        if spec.args.iter().any(|arg| arg.contains("secret")) {
            return Err("password must not appear in args".to_string());
        }
        Ok(())
    }

    #[test]
    fn build_command_bootstrap_uses_configured_superuser_username() -> Result<(), String> {
        let config = sample_config();
        let spec = build_command(
            &config,
            &JobId("job-test".to_string()),
            &ProcessJobKind::Bootstrap(BootstrapSpec {
                data_dir: PathBuf::from("/tmp/node/data"),
                superuser_username: "su_admin".to_string(),
                timeout_ms: Some(30_000),
            }),
            false,
        )
        .map_err(|err| format!("build_command failed: {err}"))?;

        if spec.program != config.binaries.initdb {
            return Err(format!(
                "expected initdb binary, got {}",
                spec.program.display()
            ));
        }
        if spec.args
            != vec![
                "-D", "/tmp/node/data", "-A", "trust", "-U", "su_admin",
            ]
        {
            return Err(format!("unexpected bootstrap args: {:?}", spec.args));
        }
        if !spec.env.is_empty() {
            return Err(format!("expected no env vars, got {:?}", spec.env));
        }
        Ok(())
    }

    #[test]
    fn build_command_start_postgres_includes_extra_settings_deterministically() -> Result<(), String>
    {
        let config = sample_config();
        let mut extra_postgres_settings = std::collections::BTreeMap::new();
        // Insert out of order to ensure deterministic ordering (BTreeMap order).
        extra_postgres_settings.insert(
            "ident_file".to_string(),
            "/tmp/managed/ident.conf".to_string(),
        );
        extra_postgres_settings.insert("hba_file".to_string(), "/tmp/managed/hba.conf".to_string());

        let spec = build_command(
            &config,
            &JobId("job-start".to_string()),
            &ProcessJobKind::StartPostgres(StartPostgresSpec {
                extra_postgres_settings,
                ..sample_start_spec()
            }),
            false,
        )
        .map_err(|err| format!("build_command failed: {err}"))?;

        let expected = vec![
            "-D".to_string(),
            "/tmp/node/data".to_string(),
            "-l".to_string(),
            "/tmp/node/postgres.log".to_string(),
            "-o".to_string(),
            "-h 127.0.0.1 -p 5544 -k /tmp/node/socket -c hba_file=/tmp/managed/hba.conf -c ident_file=/tmp/managed/ident.conf".to_string(),
            "start".to_string(),
            "-w".to_string(),
            "-t".to_string(),
            "1".to_string(),
        ];
        assert_eq!(spec.args, expected);
        Ok(())
    }

    #[test]
    fn build_command_pg_rewind_sets_pgpassword_env_for_password_auth() -> Result<(), String> {
        let config = sample_config();
        let spec = build_command(
            &config,
            &JobId("job-test".to_string()),
            &ProcessJobKind::PgRewind(PgRewindSpec {
                target_data_dir: PathBuf::from("/tmp/node/data"),
                source: RewinderSourceConn {
                    conninfo: PgConnInfo {
                        host: "10.0.0.12".to_string(),
                        port: 5433,
                        user: "rewinder".to_string(),
                        dbname: "postgres".to_string(),
                        application_name: None,
                        connect_timeout_s: None,
                        ssl_mode: PgSslMode::Prefer,
                        options: None,
                    },
                    auth: RoleAuthConfig::Password {
                        password: SecretSource(InlineOrPath::Inline {
                            content: "rewindpass".to_string(),
                        }),
                    },
                },
                timeout_ms: Some(30_000),
            }),
            false,
        )
        .map_err(|err| format!("build_command failed: {err}"))?;

        if spec.env.len() != 1 {
            return Err(format!("expected 1 env var, got {}", spec.env.len()));
        }
        if spec.env[0].key.as_str() != "PGPASSWORD" {
            return Err(format!("expected env key PGPASSWORD, got {}", spec.env[0].key));
        }
        if spec.args.iter().any(|arg| arg.contains("rewindpass")) {
            return Err("password must not appear in args".to_string());
        }
        let idx = spec
            .args
            .iter()
            .position(|arg| arg == "--source-server")
            .ok_or_else(|| "missing --source-server arg".to_string())?;
        let source = spec
            .args
            .get(idx.saturating_add(1))
            .ok_or_else(|| "missing --source-server value".to_string())?;
        if !source.contains("user=rewinder") {
            return Err(format!(
                "expected --source-server to include user=rewinder, got: {source}"
            ));
        }
        Ok(())
    }

    #[test]
    fn build_command_pgbackrest_version_uses_pgbackrest_binary_and_args() -> Result<(), String> {
        let config = sample_config();
        let pg_bin = config
            .binaries
            .pgbackrest
            .clone()
            .ok_or_else(|| "test config missing pgbackrest".to_string())?;
        let command = build_command(
            &config,
            &JobId("job-test".to_string()),
            &ProcessJobKind::PgBackRestVersion(crate::process::jobs::PgBackRestVersionSpec {}),
            true,
        );
        let spec = command.map_err(|err| err.to_string())?;
        assert_eq!(spec.program, pg_bin);
            assert_eq!(spec.args, vec!["--version"]);
        Ok(())
    }

    #[test]
    fn build_command_pgbackrest_renders_all_operations_deterministically() -> Result<(), String> {
        let config = sample_config();
        let pg_bin = config
            .binaries
            .pgbackrest
            .clone()
            .ok_or_else(|| "test config missing pgbackrest".to_string())?;

        let job_id = JobId("job-test".to_string());

        let info = build_command(
            &config,
            &job_id,
            &ProcessJobKind::PgBackRestInfo(crate::process::jobs::PgBackRestInfoSpec {
                stanza: "stanza-a".to_string(),
                repo: "1".to_string(),
                options: vec!["--log-level-console=info".to_string()],
                timeout_ms: None,
            }),
            true,
        )
        .map_err(|err| err.to_string())?;
        assert_eq!(info.program, pg_bin);
        assert_eq!(info.args.last().map(String::as_str), Some("info"));
        assert!(info.args.iter().any(|arg| arg == "--output=json"));

        let check = build_command(
            &config,
            &job_id,
            &ProcessJobKind::PgBackRestCheck(crate::process::jobs::PgBackRestCheckSpec {
                stanza: "stanza-a".to_string(),
                repo: "1".to_string(),
                options: Vec::new(),
                timeout_ms: None,
            }),
            false,
        )
        .map_err(|err| err.to_string())?;
        assert_eq!(check.args.last().map(String::as_str), Some("check"));

        let backup = build_command(
            &config,
            &job_id,
            &ProcessJobKind::PgBackRestBackup(crate::process::jobs::PgBackRestBackupSpec {
                stanza: "stanza-a".to_string(),
                repo: "1".to_string(),
                options: Vec::new(),
                timeout_ms: None,
            }),
            false,
        )
        .map_err(|err| err.to_string())?;
        assert_eq!(backup.args.last().map(String::as_str), Some("backup"));

        let restore = build_command(
            &config,
            &job_id,
            &ProcessJobKind::PgBackRestRestore(crate::process::jobs::PgBackRestRestoreSpec {
                stanza: "stanza-a".to_string(),
                repo: "1".to_string(),
                pg1_path: "/tmp/node/data".into(),
                options: Vec::new(),
                timeout_ms: None,
            }),
            false,
        )
        .map_err(|err| err.to_string())?;
        assert_eq!(restore.args.last().map(String::as_str), Some("restore"));

        let archive_push = build_command(
            &config,
            &job_id,
            &ProcessJobKind::PgBackRestArchivePush(crate::process::jobs::PgBackRestArchivePushSpec {
                stanza: "stanza-a".to_string(),
                repo: "1".to_string(),
                pg1_path: "/tmp/node/data".into(),
                wal_path: "/tmp/000000010000000000000001".to_string(),
                options: Vec::new(),
                timeout_ms: None,
            }),
            false,
        )
        .map_err(|err| err.to_string())?;
        assert_eq!(
            archive_push.args.get(archive_push.args.len().saturating_sub(2)).map(String::as_str),
            Some("archive-push")
        );

        let archive_get = build_command(
            &config,
            &job_id,
            &ProcessJobKind::PgBackRestArchiveGet(crate::process::jobs::PgBackRestArchiveGetSpec {
                stanza: "stanza-a".to_string(),
                repo: "1".to_string(),
                pg1_path: "/tmp/node/data".into(),
                wal_segment: "000000010000000000000001".to_string(),
                destination_path: "/tmp/wal".to_string(),
                options: Vec::new(),
                timeout_ms: None,
            }),
            false,
        )
        .map_err(|err| err.to_string())?;
        assert_eq!(
            archive_get.args.get(archive_get.args.len().saturating_sub(3)).map(String::as_str),
            Some("archive-get")
        );

        Ok(())
    }

    #[test]
    fn timeout_for_kind_uses_process_backup_timeout_ms_for_pgbackrest_ops() {
        let mut config = sample_config();
        config.backup_timeout_ms = 42;
        let kind = ProcessJobKind::PgBackRestInfo(crate::process::jobs::PgBackRestInfoSpec {
            stanza: "stanza-a".to_string(),
            repo: "1".to_string(),
            options: Vec::new(),
            timeout_ms: None,
        });
        assert_eq!(super::timeout_for_kind(&kind, &config), 42);
    }

    fn queued_clock(times: Vec<u64>) -> Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send> {
        let mut queue: VecDeque<u64> = times.into_iter().collect();
        let mut last = 0_u64;
        Box::new(move || {
            if let Some(next) = queue.pop_front() {
                last = next;
                return Ok(UnixMillis(next));
            }
            Ok(UnixMillis(last))
        })
    }

    fn test_ctx(
        runner: Box<dyn ProcessCommandRunner>,
        now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    ) -> (
        ProcessWorkerCtx,
        mpsc::UnboundedSender<ProcessJobRequest>,
        crate::state::StateSubscriber<ProcessState>,
    ) {
        let initial = ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        };
        let (publisher, subscriber) = new_state_channel(initial.clone(), UnixMillis(0));
        let (tx, rx) = mpsc::unbounded_channel();
        (
            ProcessWorkerCtx {
                poll_interval: Duration::from_millis(10),
                config: sample_config(),
                log: crate::logging::LogHandle::null(),
                capture_subprocess_output: false,
                state: initial,
                publisher,
                inbox: rx,
                inbox_disconnected_logged: false,
                command_runner: runner,
                active_runtime: None,
                last_rejection: None,
                now,
            },
            tx,
            subscriber,
        )
    }

    fn test_ctx_with_log(
        runner: Box<dyn ProcessCommandRunner>,
        now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    ) -> (
        ProcessWorkerCtx,
        mpsc::UnboundedSender<ProcessJobRequest>,
        crate::state::StateSubscriber<ProcessState>,
        std::sync::Arc<TestSink>,
    ) {
        let initial = ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        };
        let (publisher, subscriber) = new_state_channel(initial.clone(), UnixMillis(0));
        let (tx, rx) = mpsc::unbounded_channel();
        let (log, sink) = test_log_handle();
        (
            ProcessWorkerCtx {
                poll_interval: Duration::from_millis(10),
                config: sample_config(),
                log,
                capture_subprocess_output: false,
                state: initial,
                publisher,
                inbox: rx,
                inbox_disconnected_logged: false,
                command_runner: runner,
                active_runtime: None,
                last_rejection: None,
                now,
            },
            tx,
            subscriber,
            sink,
        )
    }

    fn running_state_for_acceptance() -> ProcessState {
        ProcessState::Running {
            worker: WorkerStatus::Running,
            active: ActiveJob {
                id: JobId("job-running".to_string()),
                kind: crate::process::jobs::ActiveJobKind::StartPostgres,
                started_at: UnixMillis(1),
                deadline_at: UnixMillis(2),
            },
        }
    }

    fn outcome_id(outcome: &JobOutcome) -> &JobId {
        match outcome {
            JobOutcome::Success { id, .. }
            | JobOutcome::Failure { id, .. }
            | JobOutcome::Timeout { id, .. } => id,
        }
    }

    #[test]
    fn can_accept_job_is_true_only_when_idle() {
        let idle = ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: None,
        };
        assert!(can_accept_job(&idle));
        assert!(!can_accept_job(&running_state_for_acceptance()));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn start_job_when_idle_transitions_to_running_and_publishes() {
        let runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(None)]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut ctx, tx, subscriber) = test_ctx(Box::new(runner), queued_clock(vec![10, 11]));

        let send_result = tx.send(ProcessJobRequest {
            id: JobId("job-1".to_string()),
            kind: ProcessJobKind::StartPostgres(sample_start_spec()),
        });
        assert!(send_result.is_ok());

        let stepped = step_once(&mut ctx).await;
        assert_eq!(stepped, Ok(()));

        assert!(matches!(&ctx.state, ProcessState::Running { .. }));
        if let ProcessState::Running { active, .. } = &ctx.state {
            assert_eq!(active.id, JobId("job-1".to_string()));
        }

        let published = subscriber.latest();
        assert!(matches!(published.value, ProcessState::Running { .. }));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_emits_request_received_and_job_started() -> Result<(), WorkerError> {
        let runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(None)]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut ctx, tx, _subscriber, sink) =
            test_ctx_with_log(Box::new(runner), queued_clock(vec![10, 11]));

        tx.send(ProcessJobRequest {
            id: JobId("job-1".to_string()),
            kind: ProcessJobKind::StartPostgres(sample_start_spec()),
        })
        .map_err(|err| WorkerError::Message(format!("send request failed: {err}")))?;

        step_once(&mut ctx).await?;

        let received = sink
            .collect_matching(|record| {
                matches!(
                    record.attributes.get("event.name"),
                    Some(serde_json::Value::String(name))
                        if name == "process.worker.request_received"
                )
            })
            .map_err(|err| WorkerError::Message(format!("log snapshot failed: {err}")))?;
        if received.is_empty() {
            return Err(WorkerError::Message(
                "expected process.worker.request_received log event".to_string(),
            ));
        }

        let started = sink
            .collect_matching(|record| {
                matches!(
                    record.attributes.get("event.name"),
                    Some(serde_json::Value::String(name)) if name == "process.job.started"
                )
            })
            .map_err(|err| WorkerError::Message(format!("log snapshot failed: {err}")))?;
        if started.is_empty() {
            return Err(WorkerError::Message(
                "expected process.job.started log event".to_string(),
            ));
        }

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rejecting_new_job_while_active_records_deterministic_rejection() {
        let runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(None), Ok(None)]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut ctx, tx, _subscriber) = test_ctx(Box::new(runner), queued_clock(vec![1, 2, 3, 4]));

        assert!(tx
            .send(ProcessJobRequest {
                id: JobId("job-a".to_string()),
                kind: ProcessJobKind::StartPostgres(sample_start_spec()),
            })
            .is_ok());
        assert_eq!(step_once(&mut ctx).await, Ok(()));
        assert!(matches!(&ctx.state, ProcessState::Running { .. }));

        assert!(tx
            .send(ProcessJobRequest {
                id: JobId("job-b".to_string()),
                kind: ProcessJobKind::PgBackRestVersion(crate::process::jobs::PgBackRestVersionSpec {}),
            })
            .is_ok());
        assert_eq!(step_once(&mut ctx).await, Ok(()));

        assert!(matches!(&ctx.state, ProcessState::Running { .. }));
        assert!(ctx.last_rejection.is_some());
        if let Some(rejection) = &ctx.last_rejection {
            assert_eq!(rejection.id, JobId("job-b".to_string()));
            assert_eq!(rejection.error, ProcessError::Busy);
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tick_active_job_maps_success_failure_and_timeout_outcomes() {
        let success_runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(Some(ProcessExit::Success))]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut success_ctx, success_tx, _success_subscriber) =
            test_ctx(Box::new(success_runner), queued_clock(vec![1, 2, 3]));
        assert!(success_tx
            .send(ProcessJobRequest {
                id: JobId("job-success".to_string()),
                kind: ProcessJobKind::StartPostgres(sample_start_spec()),
            })
            .is_ok());
        assert_eq!(step_once(&mut success_ctx).await, Ok(()));
        assert!(matches!(
            &success_ctx.state,
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { .. }),
                ..
            }
        ));
        if let ProcessState::Idle {
            last_outcome: Some(JobOutcome::Success { id, .. }),
            ..
        } = &success_ctx.state
        {
            assert_eq!(id, &JobId("job-success".to_string()));
        }

        let failure_runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(Some(ProcessExit::Failure { code: Some(4) }))]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut failure_ctx, failure_tx, _failure_subscriber) =
            test_ctx(Box::new(failure_runner), queued_clock(vec![10, 11, 12]));
        assert!(failure_tx
            .send(ProcessJobRequest {
                id: JobId("job-failure".to_string()),
                kind: ProcessJobKind::StartPostgres(sample_start_spec()),
            })
            .is_ok());
        assert_eq!(step_once(&mut failure_ctx).await, Ok(()));
        assert!(matches!(
            &failure_ctx.state,
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Failure {
                    error: ProcessError::EarlyExit { .. },
                    ..
                }),
                ..
            }
        ));
        if let ProcessState::Idle {
            last_outcome:
                Some(JobOutcome::Failure {
                    id,
                    error: ProcessError::EarlyExit { code },
                    ..
                }),
            ..
        } = &failure_ctx.state
        {
            assert_eq!(id, &JobId("job-failure".to_string()));
            assert_eq!(*code, Some(4));
        }

        let timeout_runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(None), Ok(None)]),
                cancel_result: Ok(()),
            })]),
        };
        let (mut timeout_ctx, timeout_tx, _timeout_subscriber) =
            test_ctx(Box::new(timeout_runner), queued_clock(vec![20, 40, 41]));
        assert!(timeout_tx
            .send(ProcessJobRequest {
                id: JobId("job-timeout".to_string()),
                kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                    timeout_ms: Some(1),
                    ..sample_start_spec()
                }),
            })
            .is_ok());
        assert_eq!(step_once(&mut timeout_ctx).await, Ok(()));
        assert!(matches!(
            &timeout_ctx.state,
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Timeout { .. }),
                ..
            }
        ));
        if let ProcessState::Idle {
            last_outcome: Some(JobOutcome::Timeout { id, .. }),
            ..
        } = &timeout_ctx.state
        {
            assert_eq!(id, &JobId("job-timeout".to_string()));
        }
    }

    fn pg16_binaries() -> Result<BinaryPaths, WorkerError> {
        require_pg16_process_binaries_for_real_tests()
            .map_err(|err| WorkerError::Message(format!("pg16 binary lookup failed: {err}")))
    }

    fn real_config(binaries: BinaryPaths) -> ProcessConfig {
        ProcessConfig {
            pg_rewind_timeout_ms: 20_000,
            bootstrap_timeout_ms: 40_000,
            fencing_timeout_ms: 20_000,
            backup_timeout_ms: 40_000,
            binaries,
        }
    }

    fn real_ctx(
        config: ProcessConfig,
    ) -> (
        ProcessWorkerCtx,
        mpsc::UnboundedSender<ProcessJobRequest>,
        crate::state::StateSubscriber<ProcessState>,
    ) {
        let initial = ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        };
        let (publisher, subscriber) = new_state_channel(initial.clone(), UnixMillis(0));
        let (tx, rx) = mpsc::unbounded_channel();
        (
            ProcessWorkerCtx {
                poll_interval: Duration::from_millis(50),
                config,
                log: crate::logging::LogHandle::null(),
                capture_subprocess_output: false,
                state: initial,
                publisher,
                inbox: rx,
                inbox_disconnected_logged: false,
                command_runner: Box::new(TokioCommandRunner),
                active_runtime: None,
                last_rejection: None,
                now: Box::new(super::system_now_unix_millis),
            },
            tx,
            subscriber,
        )
    }

    async fn wait_for_outcome(
        ctx: &mut ProcessWorkerCtx,
        expected_job: &JobId,
        timeout: Duration,
    ) -> Result<JobOutcome, WorkerError> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            step_once(ctx).await?;
            if let ProcessState::Idle {
                last_outcome: Some(outcome),
                ..
            } = &ctx.state
            {
                if outcome_id(outcome) == expected_job {
                    return Ok(outcome.clone());
                }
            }
            sleep(Duration::from_millis(50)).await;
        }

        Err(WorkerError::Message(format!(
            "timed out waiting for process outcome for {}",
            expected_job.0
        )))
    }

    async fn submit_job_and_wait(
        tx: &mpsc::UnboundedSender<ProcessJobRequest>,
        ctx: &mut ProcessWorkerCtx,
        id: &str,
        kind: ProcessJobKind,
        timeout: Duration,
    ) -> Result<JobOutcome, WorkerError> {
        tx.send(ProcessJobRequest {
            id: JobId(id.to_string()),
            kind,
        })
        .map_err(|err| WorkerError::Message(format!("send process job failed: {err}")))?;

        wait_for_outcome(ctx, &JobId(id.to_string()), timeout).await
    }

    struct RealProcessFixture {
        ctx: ProcessWorkerCtx,
        tx: mpsc::UnboundedSender<ProcessJobRequest>,
        subscriber: crate::state::StateSubscriber<ProcessState>,
        _guard: NamespaceGuard,
        data_dir: PathBuf,
        socket_dir: PathBuf,
        log_file: PathBuf,
        port: u16,
    }

    impl RealProcessFixture {
        async fn bootstrap_and_start(
            binaries: BinaryPaths,
            ns_name: &str,
        ) -> Result<Self, WorkerError> {
            let guard = NamespaceGuard::new(ns_name)
                .map_err(|err| WorkerError::Message(format!("namespace create failed: {err}")))?;
            let namespace = guard
                .namespace()
                .map_err(|err| WorkerError::Message(format!("namespace lookup failed: {err}")))?
                .clone();

            let reservation = allocate_ports(1)
                .map_err(|err| WorkerError::Message(format!("port allocation failed: {err}")))?;
            let port = reservation.as_slice()[0];

            let data_dir = namespace.child_dir("pgdata");
            let socket_dir = namespace.child_dir("sock");
            let log_dir = namespace.child_dir("log");
            let log_file = log_dir.join("postgres.log");
            fs::create_dir_all(&socket_dir)
                .map_err(|err| WorkerError::Message(format!("socket dir create failed: {err}")))?;
            fs::create_dir_all(&log_dir)
                .map_err(|err| WorkerError::Message(format!("log dir create failed: {err}")))?;

            let (ctx, tx, subscriber) = real_ctx(real_config(binaries));
            let mut fixture = Self {
                ctx,
                tx,
                subscriber,
                _guard: guard,
                data_dir,
                socket_dir,
                log_file,
                port,
            };

            let bootstrap = fixture
                .submit_job_and_wait(
                    "bootstrap",
                    ProcessJobKind::Bootstrap(BootstrapSpec {
                        data_dir: fixture.data_dir.clone(),
                        superuser_username: "postgres".to_string(),
                        timeout_ms: Some(30_000),
                    }),
                    Duration::from_secs(40),
                )
                .await?;
            if !matches!(bootstrap, JobOutcome::Success { .. }) {
                return Err(WorkerError::Message(format!(
                    "bootstrap setup failed: {bootstrap:?}"
                )));
            }

            // Release the reserved port immediately before starting postgres so
            // `pg_ctl` can bind the requested port.
            drop(reservation);
            let mut start_failure: Option<JobOutcome> = None;
            for attempt in 1..=3 {
                if attempt > 1 {
                    let retry_reservation = allocate_ports(1).map_err(|err| {
                        WorkerError::Message(format!("retry port allocation failed: {err}"))
                    })?;
                    fixture.port = retry_reservation.as_slice()[0];
                    drop(retry_reservation);
                }
                let start = fixture
                    .submit_job_and_wait(
                        "start",
                        ProcessJobKind::StartPostgres(StartPostgresSpec {
                            data_dir: fixture.data_dir.clone(),
                            host: "127.0.0.1".to_string(),
                            port: fixture.port,
                            socket_dir: fixture.socket_dir.clone(),
                            log_file: fixture.log_file.clone(),
                            extra_postgres_settings: std::collections::BTreeMap::new(),
                            wait_seconds: Some(45),
                            timeout_ms: Some(70_000),
                        }),
                        Duration::from_secs(80),
                    )
                    .await?;
                if matches!(start, JobOutcome::Success { .. }) {
                    start_failure = None;
                    break;
                }
                if fixture.postgres_ready_probe(Duration::from_secs(5)).await? {
                    start_failure = None;
                    break;
                }
                let cleanup_id = format!("start-cleanup-{attempt}");
                let _ = fixture
                    .submit_job_and_wait(
                        cleanup_id.as_str(),
                        ProcessJobKind::Demote(DemoteSpec {
                            data_dir: fixture.data_dir.clone(),
                            mode: ShutdownMode::Fast,
                            timeout_ms: Some(15_000),
                        }),
                        Duration::from_secs(20),
                    )
                    .await;
                start_failure = Some(start);
            }
            if let Some(start) = start_failure {
                let log_tail = match fs::read_to_string(&fixture.log_file) {
                    Ok(content) => {
                        let lines = content.lines().collect::<Vec<_>>();
                        let tail = if lines.len() > 30 {
                            lines[lines.len().saturating_sub(30)..].to_vec()
                        } else {
                            lines
                        };
                        tail.join(" | ")
                    }
                    Err(err) => format!("log-read-failed: {err}"),
                };
                return Err(WorkerError::Message(format!(
                    "start setup failed after retries: {start:?}; postgres_log_tail={log_tail}"
                )));
            }

            Ok(fixture)
        }

        async fn submit_job_and_wait(
            &mut self,
            id: &str,
            kind: ProcessJobKind,
            timeout: Duration,
        ) -> Result<JobOutcome, WorkerError> {
            let expected_job = JobId(id.to_string());
            self.tx
                .send(ProcessJobRequest {
                    id: expected_job.clone(),
                    kind,
                })
                .map_err(|err| WorkerError::Message(format!("send process job failed: {err}")))?;

            self.wait_for_outcome(&expected_job, timeout).await
        }

        async fn wait_for_outcome(
            &mut self,
            expected_job: &JobId,
            timeout: Duration,
        ) -> Result<JobOutcome, WorkerError> {
            let start = Instant::now();
            let mut last_snapshot = self.subscriber.latest();
            while start.elapsed() < timeout {
                let _before = self.subscriber.latest();
                step_once(&mut self.ctx).await?;
                last_snapshot = self.subscriber.latest();
                if let ProcessState::Idle {
                    last_outcome: Some(outcome),
                    ..
                } = &self.ctx.state
                {
                    if outcome_id(outcome) == expected_job {
                        return Ok(outcome.clone());
                    }
                }
                sleep(Duration::from_millis(50)).await;
            }

            Err(WorkerError::Message(format!(
                "timed out waiting for process outcome for {} (last snapshot: {:?})",
                expected_job.0, last_snapshot
            )))
        }

        async fn postgres_ready_probe(&self, timeout: Duration) -> Result<bool, WorkerError> {
            let mut command = Command::new(&self.ctx.config.binaries.psql);
            command
                .arg("-h")
                .arg("127.0.0.1")
                .arg("-p")
                .arg(self.port.to_string())
                .arg("-U")
                .arg("postgres")
                .arg("-d")
                .arg("postgres")
                .arg("-At")
                .arg("-c")
                .arg("SELECT 1");

            let output = match tokio::time::timeout(timeout, command.output()).await {
                Ok(Ok(output)) => output,
                Ok(Err(err)) => {
                    return Err(WorkerError::Message(format!(
                        "postgres readiness probe spawn failed: {err}"
                    )))
                }
                Err(_) => return Ok(false),
            };
            if !output.status.success() {
                return Ok(false);
            }
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.lines().any(|line| line.trim() == "1"))
        }
    }

    fn assert_success_outcome(label: &str, outcome: &JobOutcome) -> Result<(), WorkerError> {
        match outcome {
            JobOutcome::Success { .. } => Ok(()),
            other => Err(WorkerError::Message(format!(
                "expected {label} to succeed, got: {other:?}"
            ))),
        }
    }

    fn assert_promote_outcome(outcome: &JobOutcome) -> Result<(), WorkerError> {
        match outcome {
            JobOutcome::Success { .. } => Ok(()),
            JobOutcome::Failure {
                error: ProcessError::EarlyExit { code: Some(1) },
                ..
            } => Ok(()),
            JobOutcome::Failure { error, .. } => Err(WorkerError::Message(format!(
                "expected promote success or standby-state early-exit, got failure: {error:?}"
            ))),
            other => Err(WorkerError::Message(format!(
                "expected promote success or standby-state early-exit, got: {other:?}"
            ))),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_bootstrap_job_executes_initdb() -> Result<(), WorkerError> {
        let binaries = pg16_binaries()?;
        let guard = NamespaceGuard::new("process-bootstrap")
            .map_err(|err| WorkerError::Message(format!("namespace setup failed: {err}")))?;
        let namespace = guard
            .namespace()
            .map_err(|err| WorkerError::Message(format!("namespace lookup failed: {err}")))?;

        let data_dir = namespace.child_dir("process/node-a/data");
        let (mut ctx, tx, _sub) = real_ctx(real_config(binaries));
        let outcome = submit_job_and_wait(
            &tx,
            &mut ctx,
            "bootstrap-1",
            ProcessJobKind::Bootstrap(BootstrapSpec {
                data_dir: data_dir.clone(),
                superuser_username: "postgres".to_string(),
                timeout_ms: Some(30_000),
            }),
            Duration::from_secs(40),
        )
        .await;

        match outcome {
            Ok(JobOutcome::Success { .. }) => {
                assert!(data_dir.join("PG_VERSION").exists());
                Ok(())
            }
            Ok(other) => Err(WorkerError::Message(format!(
                "expected bootstrap success, got: {other:?}"
            ))),
            Err(err) => Err(WorkerError::Message(format!("bootstrap job failed: {err}"))),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_pg_rewind_job_executes_binary_path() -> Result<(), WorkerError> {
        let binaries = pg16_binaries()?;

        let guard = NamespaceGuard::new("process-rewind")
            .map_err(|err| WorkerError::Message(format!("namespace setup failed: {err}")))?;
        let namespace = guard
            .namespace()
            .map_err(|err| WorkerError::Message(format!("namespace lookup failed: {err}")))?;

        let data_dir = namespace.child_dir("process/rewind/target");
        fs::create_dir_all(&data_dir)
            .map_err(|err| WorkerError::Message(format!("create rewind data dir failed: {err}")))?;

        let (mut ctx, tx, _sub) = real_ctx(real_config(binaries));
        let outcome = submit_job_and_wait(
            &tx,
            &mut ctx,
            "rewind-1",
            ProcessJobKind::PgRewind(PgRewindSpec {
                target_data_dir: data_dir,
                source: RewinderSourceConn {
                    conninfo: sample_rewind_conninfo(),
                    auth: RoleAuthConfig::Tls,
                },
                timeout_ms: Some(5_000),
            }),
            Duration::from_secs(10),
        )
        .await;

        match outcome {
            Ok(JobOutcome::Failure {
                error: ProcessError::EarlyExit { code: Some(_) },
                ..
            }) => Ok(()),
            Ok(JobOutcome::Failure { error, .. }) => Err(WorkerError::Message(format!(
                "expected rewind early-exit failure for invalid source, got failure: {error:?}"
            ))),
            Ok(other) => Err(WorkerError::Message(format!(
                "expected rewind early-exit failure for invalid source, got: {other:?}"
            ))),
            Err(err) => Err(WorkerError::Message(format!(
                "rewind job wait failed: {err}"
            ))),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_pgbackrest_version_job_executes_binary_path() -> Result<(), WorkerError> {
        let binaries = pg16_binaries()?;

        let (mut ctx, tx, _sub) = real_ctx(real_config(binaries));
        let outcome = submit_job_and_wait(
            &tx,
            &mut ctx,
            "pgbackrest-version-1",
            ProcessJobKind::PgBackRestVersion(crate::process::jobs::PgBackRestVersionSpec {}),
            Duration::from_secs(10),
        )
        .await?;

        match outcome {
            JobOutcome::Success { .. } => Ok(()),
            other => Err(WorkerError::Message(format!(
                "expected pgbackrest version job success, got: {other:?}"
            ))),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_promote_job_executes_binary_path() -> Result<(), WorkerError> {
        let binaries = pg16_binaries()?;
        let mut fixture =
            RealProcessFixture::bootstrap_and_start(binaries, "process-promote").await?;

        let promote = fixture
            .submit_job_and_wait(
                "promote",
                ProcessJobKind::Promote(PromoteSpec {
                    data_dir: fixture.data_dir.clone(),
                    wait_seconds: Some(10),
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        assert_promote_outcome(&promote)?;

        let cleanup = fixture
            .submit_job_and_wait(
                "demote-after-promote",
                ProcessJobKind::Demote(DemoteSpec {
                    data_dir: fixture.data_dir.clone(),
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        assert_success_outcome("demote-after-promote", &cleanup)?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_demote_job_executes_binary_path() -> Result<(), WorkerError> {
        let binaries = pg16_binaries()?;
        let mut fixture =
            RealProcessFixture::bootstrap_and_start(binaries, "process-demote").await?;

        let outcome = fixture
            .submit_job_and_wait(
                "demote",
                ProcessJobKind::Demote(DemoteSpec {
                    data_dir: fixture.data_dir.clone(),
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        assert_success_outcome("demote", &outcome)?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn real_fencing_job_executes_binary_path() -> Result<(), WorkerError> {
        let binaries = pg16_binaries()?;
        let mut fixture =
            RealProcessFixture::bootstrap_and_start(binaries, "process-fencing").await?;

        let outcome = fixture
            .submit_job_and_wait(
                "fence",
                ProcessJobKind::Fencing(FencingSpec {
                    data_dir: fixture.data_dir.clone(),
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(10_000),
                }),
                Duration::from_secs(20),
            )
            .await?;
        assert_success_outcome("fence", &outcome)?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn start_job_returns_channel_closed_when_all_subscribers_are_dropped() {
        let runner = FakeRunner {
            spawn_results: VecDeque::from(vec![Ok(FakeHandle {
                polls: VecDeque::from(vec![Ok(None)]),
                cancel_result: Ok(()),
            })]),
        };
        let initial = ProcessState::Idle {
            worker: WorkerStatus::Starting,
            last_outcome: None,
        };
        let (publisher, subscriber) = new_state_channel(initial.clone(), UnixMillis(0));
        drop(subscriber);
        let (_tx, rx) = mpsc::unbounded_channel();
        let mut ctx = ProcessWorkerCtx {
            poll_interval: Duration::from_millis(10),
            config: sample_config(),
            log: crate::logging::LogHandle::null(),
            capture_subprocess_output: false,
            state: initial,
            publisher,
            inbox: rx,
            inbox_disconnected_logged: false,
            command_runner: Box::new(runner),
            active_runtime: None,
            last_rejection: None,
            now: queued_clock(vec![1, 2]),
        };

        let result = start_job(
            &mut ctx,
            ProcessJobRequest {
                id: JobId("job-no-subscriber".to_string()),
                kind: ProcessJobKind::StartPostgres(sample_start_spec()),
            },
        )
        .await;

        assert!(matches!(
            result,
            Err(WorkerError::Message(message)) if message.contains("state channel is closed")
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tick_active_job_is_safe_when_no_runtime() {
        let (mut ctx, _tx, _sub) = test_ctx(Box::new(NoopCommandRunner), queued_clock(vec![1]));
        assert_eq!(tick_active_job(&mut ctx).await, Ok(()));
    }
}
