use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use pgtm_log_derive::LoggableEvent;
use serde_json::Value;

use crate::config_v2::RuntimeConfigV2;
use crate::logging::{LogProducer, LogSender, LogSeverity, LogTransport};
use crate::state::WorkerError;

use super::tailer::{DirTailers, FileTailer, StartPosition};

pub(crate) struct PostgresIngestWorkerCtx<'a> {
    pub(crate) cfg: &'a RuntimeConfigV2,
    pub(crate) log: LogSender,
}

#[derive(Clone, Debug, PartialEq, Eq, LoggableEvent)]
#[log_event(producer = "app", transport = "internal", parser = "app")]
pub(crate) enum PostgresIngestLogEvent {
    #[log_event(
        name = "postgres_ingest.step_once_failed",
        severity = "error",
        result = "failed",
        message = "postgres ingest step once failed"
    )]
    StepOnceFailed {
        #[log(key = "postgres_ingest.attempts")]
        attempts: u32,
        #[log(key = "postgres_ingest.suppressed")]
        suppressed: u64,
        cause: String,
    },

    #[log_event(
        name = "postgres_ingest.recovered",
        severity = "info",
        result = "recovered",
        message = "postgres ingest recovered"
    )]
    Recovered {
        #[log(key = "postgres_ingest.attempts")]
        attempts: u32,
    },

    #[log_event(
        name = "postgres_ingest.iteration_summary",
        severity = "debug",
        result = "ok",
        message = "postgres ingest iteration summary"
    )]
    IterationSummary {
        #[log(key = "postgres_ingest.pg_ctl_lines_emitted")]
        pg_ctl_lines_emitted: u64,
        #[log(key = "postgres_ingest.log_dir_files_tailed")]
        log_dir_files_tailed: u64,
        #[log(key = "postgres_ingest.log_dir_lines_emitted")]
        log_dir_lines_emitted: u64,
        #[log(key = "postgres_ingest.dir_tailers")]
        dir_tailers: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PostgresLineSource {
    pub(crate) producer: LogProducer,
    pub(crate) transport: LogTransport,
    pub(crate) path: String,
}

#[derive(Clone, Debug, PartialEq, LoggableEvent)]
pub(crate) enum PostgresLineLogEvent {
    #[log_event(name = "postgres.line_json", meta = "computed")]
    Json {
        #[log(skip)]
        source: PostgresLineSource,
        #[log(skip)]
        severity: LogSeverity,
        #[log(skip)]
        message: String,
        #[log(flatten, prefix = "postgres")]
        payload: BTreeMap<String, Value>,
        #[log(key = "postgres.path")]
        path: String,
    },

    #[log_event(name = "postgres.line_plain", meta = "computed")]
    Plain {
        #[log(skip)]
        source: PostgresLineSource,
        #[log(skip)]
        severity: LogSeverity,
        #[log(skip)]
        message: String,
        #[log(key = "postgres.level_raw")]
        level_raw: String,
        #[log(key = "postgres.path")]
        path: String,
    },

    #[log_event(name = "postgres.line_unparsed", meta = "computed")]
    Unparsed {
        #[log(skip)]
        source: PostgresLineSource,
        #[log(key = "postgres.raw_line")]
        raw_line: String,
        #[log(key = "postgres.path")]
        path: String,
    },
}

const POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS: u64 = 30_000;
const POSTGRES_INGEST_MAX_BYTES_PER_FILE: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
struct IngestIssue {
    stage: &'static str,
    kind: &'static str,
    path: PathBuf,
    cause: String,
}

impl IngestIssue {
    fn new(
        stage: &'static str,
        kind: &'static str,
        path: &Path,
        cause: impl std::fmt::Display,
    ) -> Self {
        Self {
            stage,
            kind,
            path: path.to_path_buf(),
            cause: cause.to_string(),
        }
    }

    fn key(&self) -> IngestErrorKey {
        IngestErrorKey {
            stage: self.stage,
            kind: self.kind,
            path: self.path.clone(),
        }
    }
}

impl std::fmt::Display for IngestIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "stage={} kind={} path={} error={}",
            self.stage,
            self.kind,
            self.path.display(),
            self.cause
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct IngestErrorKey {
    stage: &'static str,
    kind: &'static str,
    path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RateLimitDecision {
    emit: bool,
    suppressed: u64,
}

#[derive(Clone, Debug)]
struct RateLimitState {
    last_emit_ms: u64,
    suppressed: u64,
}

#[derive(Clone, Debug)]
struct IngestErrorRateLimiter {
    window_ms: u64,
    by_key: BTreeMap<IngestErrorKey, RateLimitState>,
}

impl IngestErrorRateLimiter {
    fn new(window_ms: u64) -> Self {
        Self {
            window_ms,
            by_key: BTreeMap::new(),
        }
    }

    fn record(&mut self, key: IngestErrorKey, now_ms: u64) -> RateLimitDecision {
        match self.by_key.get_mut(&key) {
            None => {
                self.by_key.insert(
                    key,
                    RateLimitState {
                        last_emit_ms: now_ms,
                        suppressed: 0,
                    },
                );
                RateLimitDecision {
                    emit: true,
                    suppressed: 0,
                }
            }
            Some(entry) => {
                let elapsed_ms = now_ms.saturating_sub(entry.last_emit_ms);
                if elapsed_ms >= self.window_ms {
                    let suppressed = entry.suppressed;
                    entry.last_emit_ms = now_ms;
                    entry.suppressed = 0;
                    RateLimitDecision {
                        emit: true,
                        suppressed,
                    }
                } else {
                    entry.suppressed = entry.suppressed.saturating_add(1);
                    RateLimitDecision {
                        emit: false,
                        suppressed: 0,
                    }
                }
            }
        }
    }
}

pub(crate) async fn run(ctx: PostgresIngestWorkerCtx<'_>) -> Result<(), WorkerError> {
    let mut state = PostgresIngestWorkerState::new(ctx.cfg);
    let mut limiter = IngestErrorRateLimiter::new(POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS);
    let mut consecutive_failures = 0u32;
    loop {
        if ctx.cfg.logging.postgres.enabled {
            match step_once(&ctx, &mut state).await {
                Ok(()) => {
                    if consecutive_failures > 0 {
                        ctx.log
                            .send(PostgresIngestLogEvent::Recovered {
                                attempts: consecutive_failures,
                            })
                            .map_err(|err| {
                                WorkerError::Message(format!(
                                    "postgres ingest recovered log send failed: {err}"
                                ))
                            })?;
                        consecutive_failures = 0;
                    }
                }
                Err(error) => {
                    consecutive_failures = consecutive_failures.saturating_add(1);
                    let now_ms = crate::logging::system_now_unix_millis();
                    let key = match error.first() {
                        Some(issue) => issue.key(),
                        None => IngestErrorKey {
                            stage: "unknown",
                            kind: "unknown",
                            path: PathBuf::from("."),
                        },
                    };
                    let decision = limiter.record(key, now_ms);
                    if decision.emit {
                        ctx.log
                            .send(PostgresIngestLogEvent::StepOnceFailed {
                                attempts: consecutive_failures,
                                suppressed: decision.suppressed,
                                cause: render_iteration_errors(error.as_slice()),
                            })
                            .map_err(|err| {
                                WorkerError::Message(format!(
                                    "postgres ingest error log send failed: {err}"
                                ))
                            })?;
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(
            u64::try_from(ctx.cfg.logging.postgres.poll_interval.as_millis()).unwrap_or(u64::MAX),
        ))
        .await;
    }
}

struct PostgresIngestWorkerState {
    pg_ctl_log: FileTailer,
    dir_tailers: DirTailers,
}

impl PostgresIngestWorkerState {
    fn new(cfg: &RuntimeConfigV2) -> Self {
        Self {
            pg_ctl_log: FileTailer::new(cfg.postgres.log_file.clone(), StartPosition::Beginning),
            dir_tailers: DirTailers::default(),
        }
    }
}

fn ingestable_postgres_log_start(path: &Path) -> Option<StartPosition> {
    let matches = matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("log") | Some("json")
    );
    if !matches {
        return None;
    }

    Some(match path.file_name().and_then(|name| name.to_str()) {
        Some("postgres.stderr.log") | Some("postgres.stdout.log") => StartPosition::Beginning,
        _ => StartPosition::End,
    })
}

fn render_iteration_errors(issues: &[IngestIssue]) -> String {
    let first = issues
        .first()
        .map(ToString::to_string)
        .unwrap_or_else(|| "stage=unknown kind=unknown path=. error=unknown".to_string());
    let extra = issues
        .iter()
        .skip(1)
        .take(2)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let extra_suffix = if extra.is_empty() {
        String::new()
    } else {
        format!(" extra=[{}]", extra.join(" | "))
    };
    format!(
        "postgres_ingest iteration_errors count={} {}{}",
        issues.len(),
        first,
        extra_suffix
    )
}

impl From<IngestIssue> for WorkerError {
    fn from(value: IngestIssue) -> Self {
        Self::Message(value.to_string())
    }
}

impl From<Vec<IngestIssue>> for WorkerError {
    fn from(value: Vec<IngestIssue>) -> Self {
        Self::Message(render_iteration_errors(value.as_slice()))
    }
}

async fn collect_ingestable_postgres_log_paths(
    dir: &Path,
    stage: &'static str,
) -> Result<Vec<PathBuf>, IngestIssue> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(IngestIssue::new(stage, "read_dir", dir, err)),
    };

    let mut paths = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|err| IngestIssue::new(stage, "read_dir_entry", dir, err))?
    {
        let path = entry.path();
        let is_file = match entry.file_type().await {
            Ok(ft) => ft.is_file(),
            Err(err) => return Err(IngestIssue::new(stage, "file_type", path.as_path(), err)),
        };
        if is_file && ingestable_postgres_log_start(path.as_path()).is_some() {
            paths.push(path);
        }
    }

    Ok(paths)
}

async fn emit_tailer_lines(
    log: &LogSender,
    tailer: &mut FileTailer,
    read_stage: &'static str,
    emit_stage: &'static str,
    max_bytes_per_file: usize,
) -> Result<u64, IngestIssue> {
    let lines = tailer
        .read_new_lines(max_bytes_per_file)
        .await
        .map_err(|err| IngestIssue::new(read_stage, "tailer.read_new_lines", tailer.path(), err))?;

    let mut emitted = 0u64;
    for line in lines {
        log.send(postgres_line_event(
            LogProducer::Postgres,
            LogTransport::FileTail,
            tailer.path(),
            line,
        ))
        .map_err(|err| IngestIssue::new(emit_stage, "log.emit_record", tailer.path(), err))?;
        emitted = emitted.saturating_add(1);
    }

    Ok(emitted)
}

async fn step_once(
    ctx: &PostgresIngestWorkerCtx<'_>,
    state: &mut PostgresIngestWorkerState,
) -> Result<(), Vec<IngestIssue>> {
    let max_bytes_per_file = POSTGRES_INGEST_MAX_BYTES_PER_FILE;
    let mut pg_ctl_lines_emitted: u64 = 0;
    let mut log_dir_lines_emitted: u64 = 0;
    let mut log_dir_files_tailed: u64 = 0;

    let mut issues = Vec::new();

    match emit_tailer_lines(
        &ctx.log,
        &mut state.pg_ctl_log,
        "pg_ctl_log_file.read",
        "pg_ctl_log_file.emit",
        max_bytes_per_file,
    )
    .await
    {
        Ok(emitted) => {
            pg_ctl_lines_emitted = emitted;
        }
        Err(issue) => issues.push(issue),
    }

    if ctx.cfg.logging.postgres.enabled {
        let dir = ctx.cfg.logging.postgres.log_dir.as_path();
        if let Err(err) = discover_log_dir(&mut state.dir_tailers, dir).await {
            issues.push(err);
        }

        for (_, tailer) in state.dir_tailers.iter_mut() {
            log_dir_files_tailed = log_dir_files_tailed.saturating_add(1);
            match emit_tailer_lines(
                &ctx.log,
                tailer,
                "log_dir.read",
                "log_dir.emit",
                max_bytes_per_file,
            )
            .await
            {
                Ok(emitted) => {
                    log_dir_lines_emitted = log_dir_lines_emitted.saturating_add(emitted);
                }
                Err(issue) => {
                    issues.push(issue);
                }
            }
        }

        if ctx.cfg.logging.postgres.cleanup.enabled {
            match cleanup_log_dir(
                dir,
                ctx.cfg.logging.postgres.cleanup.max_files,
                ctx.cfg.logging.postgres.cleanup.max_age,
                ctx.cfg.logging.postgres.cleanup.protect_recent,
                &[state.pg_ctl_log.path()],
                SystemTime::now(),
                &mut issues,
            )
            .await
            {
                Ok(()) => {}
                Err(err) => issues.push(IngestIssue::new(
                    "log_dir.cleanup",
                    "cleanup.fatal",
                    dir,
                    err,
                )),
            }
        }
    }

    if issues.is_empty() {
        ctx.log
            .send(PostgresIngestLogEvent::IterationSummary {
                pg_ctl_lines_emitted,
                log_dir_files_tailed,
                log_dir_lines_emitted,
                dir_tailers: state.dir_tailers.len(),
            })
            .map_err(|err| {
                vec![IngestIssue::new(
                    "iteration_summary.emit",
                    "log.send",
                    state.pg_ctl_log.path(),
                    err,
                )]
            })?;
        return Ok(());
    }

    Err(issues)
}

async fn discover_log_dir(tailers: &mut DirTailers, dir: &Path) -> Result<(), IngestIssue> {
    for path in collect_ingestable_postgres_log_paths(dir, "log_dir.discover").await? {
        let start = match ingestable_postgres_log_start(path.as_path()) {
            Some(start) => start,
            None => continue,
        };
        tailers.ensure_file(path, start);
    }
    Ok(())
}

async fn cleanup_log_dir(
    dir: &Path,
    max_files: u64,
    max_age: Duration,
    protect_recent: Duration,
    protected_paths: &[&Path],
    now: SystemTime,
    issues: &mut Vec<IngestIssue>,
) -> Result<(), IngestIssue> {
    let protected_basenames: [&str; 3] = [
        "postgres.json",
        "postgres.stderr.log",
        "postgres.stdout.log",
    ];

    let mut candidates = Vec::new();
    for path in collect_ingestable_postgres_log_paths(dir, "cleanup.collect").await? {
        let mut protected = protected_paths.contains(&path.as_path());

        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => String::new(),
        };
        if protected_basenames.contains(&file_name.as_str()) {
            protected = true;
        }

        let meta = match tokio::fs::metadata(&path).await {
            Ok(meta) => meta,
            Err(err) => {
                protected = true;
                issues.push(IngestIssue::new(
                    "cleanup.metadata",
                    "metadata",
                    path.as_path(),
                    err,
                ));
                candidates.push((path, None, protected));
                continue;
            }
        };
        let modified = match meta.modified() {
            Ok(modified) => Some(modified),
            Err(err) => {
                protected = true;
                issues.push(IngestIssue::new(
                    "cleanup.modified",
                    "modified",
                    path.as_path(),
                    err,
                ));
                candidates.push((path, None, protected));
                continue;
            }
        };

        if !protected {
            let is_recent = match modified {
                Some(modified) => match now.duration_since(modified) {
                    Ok(age) => age <= protect_recent,
                    Err(err) => {
                        issues.push(IngestIssue::new(
                            "cleanup.age",
                            "duration_since",
                            path.as_path(),
                            err,
                        ));
                        true
                    }
                },
                None => true,
            };
            if is_recent {
                protected = true;
            }
        }

        candidates.push((path, modified, protected));
    }

    let mut eligible = candidates
        .iter()
        .filter_map(|(path, modified, protected)| {
            if *protected {
                return None;
            }
            modified.map(|modified| (path.clone(), modified))
        })
        .collect::<Vec<_>>();

    eligible.sort_by(|a, b| {
        let by_time = a.1.cmp(&b.1);
        if by_time != std::cmp::Ordering::Equal {
            return by_time;
        }
        a.0.cmp(&b.0)
    });

    let mut to_remove: Vec<PathBuf> = Vec::new();

    if max_files > 0 && (eligible.len() as u64) > max_files {
        let remove_count = eligible.len().saturating_sub(max_files as usize);
        for (path, _) in eligible.iter().take(remove_count) {
            to_remove.push(path.clone());
        }
    }

    if !max_age.is_zero() {
        for (path, modified) in eligible {
            match now.duration_since(modified) {
                Ok(age) => {
                    if age > max_age {
                        to_remove.push(path);
                    }
                }
                Err(err) => {
                    issues.push(IngestIssue::new(
                        "cleanup.age",
                        "duration_since",
                        path.as_path(),
                        err,
                    ));
                }
            }
        }
    }

    for path in to_remove {
        match tokio::fs::remove_file(&path).await {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                issues.push(IngestIssue::new(
                    "cleanup.remove_file",
                    "remove_file",
                    path.as_path(),
                    err,
                ));
            }
        }
    }

    Ok(())
}

fn postgres_line_event(
    producer: LogProducer,
    transport: LogTransport,
    path: &Path,
    line: Vec<u8>,
) -> PostgresLineLogEvent {
    let decoded = decode_line(&line);
    normalize_postgres_line(
        decoded.as_str(),
        PostgresLineSource {
            producer,
            transport,
            path: path.display().to_string(),
        },
    )
}

fn decode_line(line: &[u8]) -> String {
    match String::from_utf8(line.to_vec()) {
        Ok(s) => s,
        Err(err) => {
            let bytes = err.into_bytes();
            format!("non_utf8_bytes_hex={}", hex_encode(bytes.as_slice()))
        }
    }
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

fn normalize_postgres_line(line: &str, source: PostgresLineSource) -> PostgresLineLogEvent {
    if let Ok(value) = serde_json::from_str::<Value>(line) {
        if let Some(parsed) = normalize_postgres_json(value) {
            let path = source.path.clone();
            return PostgresLineLogEvent::Json {
                source,
                severity: parsed.severity,
                message: parsed.message,
                payload: parsed.payload,
                path,
            };
        }
    }

    if let Some(parsed) = normalize_postgres_plain(line) {
        let path = source.path.clone();
        return PostgresLineLogEvent::Plain {
            source,
            severity: parsed.severity,
            message: parsed.message,
            level_raw: parsed.level_raw,
            path,
        };
    }

    let path = source.path.clone();
    PostgresLineLogEvent::Unparsed {
        source,
        raw_line: line.to_string(),
        path,
    }
}

struct ParsedLine {
    severity: LogSeverity,
    message: String,
    payload: BTreeMap<String, Value>,
    level_raw: String,
}

fn normalize_postgres_json(value: Value) -> Option<ParsedLine> {
    let obj = value.as_object()?;
    let message = match obj.get("message").and_then(|v| v.as_str()) {
        Some(message) => message.to_string(),
        None => String::new(),
    };
    if message.trim().is_empty() {
        return None;
    }

    let severity_raw = obj
        .get("error_severity")
        .and_then(|v| v.as_str())
        .or_else(|| obj.get("severity").and_then(|v| v.as_str()));
    let severity_raw = severity_raw.map_or("INFO", |severity| severity);
    let severity = map_pg_severity(severity_raw);

    Some(ParsedLine {
        severity,
        message,
        payload: obj
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
        level_raw: String::new(),
    })
}

fn normalize_postgres_plain(line: &str) -> Option<ParsedLine> {
    // Example:
    // 2026-01-01 12:34:56.789 UTC [123] LOG:  message
    let bracket = line.find('[')?;
    let after_bracket = line[bracket..].find(']')?;
    let rest = line[bracket + after_bracket + 1..].trim_start();

    let (level, message) = rest.split_once(':')?;
    let level = level.trim();
    let message = message.trim_start().to_string();
    if level.is_empty() || message.is_empty() {
        return None;
    }
    let severity = map_pg_severity(level);

    Some(ParsedLine {
        severity,
        message,
        payload: BTreeMap::new(),
        level_raw: level.to_string(),
    })
}

fn map_pg_severity(raw: &str) -> LogSeverity {
    match raw.trim().to_ascii_uppercase().as_str() {
        "DEBUG" | "DEBUG1" | "DEBUG2" | "DEBUG3" | "DEBUG4" | "DEBUG5" => LogSeverity::Debug,
        "INFO" | "NOTICE" | "LOG" => LogSeverity::Info,
        "WARNING" => LogSeverity::Warn,
        "ERROR" => LogSeverity::Error,
        "FATAL" | "PANIC" => LogSeverity::Fatal,
        _ => LogSeverity::Info,
    }
}

pub(crate) fn build_ctx<'a>(
    cfg: &'a RuntimeConfigV2,
    log: LogSender,
) -> PostgresIngestWorkerCtx<'a> {
    PostgresIngestWorkerCtx { cfg, log }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use serde_json::Value;
    use tokio::task::JoinHandle;

    use crate::logging::{
        LogContext, LogParser, LogProducer, LogSender, LogSeverity, LogSink, LogTransport, TestSink,
    };

    use crate::state::WorkerError;

    use super::{
        cleanup_log_dir, decode_line, map_pg_severity, normalize_postgres_line, IngestErrorKey,
        IngestErrorRateLimiter, IngestIssue, PostgresIngestLogEvent,
    };

    const REAL_INGEST_RETRY_SLEEP: Duration = Duration::from_millis(20);
    const REAL_PROCESS_WORKER_POLL_INTERVAL: Duration = Duration::from_millis(5);
    const REAL_PSQL_RETRY_SLEEP: Duration = Duration::from_millis(50);

    fn remove_dir_all_if_exists(path: &std::path::Path) -> Result<(), WorkerError> {
        match std::fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(WorkerError::Message(err.to_string())),
        }
    }

    struct RunningTestLog {
        sender: LogSender,
        sink: Arc<TestSink>,
        worker_task: JoinHandle<()>,
    }

    impl RunningTestLog {
        fn start() -> Self {
            let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
            let sink = Arc::new(TestSink::default());
            let sink_dyn: Arc<dyn LogSink> = sink.clone();
            let worker = super::super::LogWorker::from_sink(receiver, sink_dyn);
            Self {
                sender: LogSender::new(sample_context(), sender, LogSeverity::Trace),
                sink,
                worker_task: tokio::spawn(worker.run()),
            }
        }

        fn sender(&self) -> LogSender {
            self.sender.clone()
        }

        async fn take(&self) -> Vec<crate::logging::LogRecord> {
            tokio::task::yield_now().await;
            self.sink.take()
        }
    }

    impl Drop for RunningTestLog {
        fn drop(&mut self) {
            self.worker_task.abort();
        }
    }

    fn materialize_record<E>(event: E) -> crate::logging::LogRecord
    where
        E: crate::logging::LoggableEvent,
    {
        crate::logging::core::QueuedRecord::from_event(1, sample_context(), event.into_log_event())
            .into_record()
    }

    fn sample_context() -> LogContext {
        LogContext {
            hostname: "host-a".to_string(),
            cluster_name: "cluster-a".to_string(),
            scope: "scope-a".to_string(),
            member_id: "member-a".to_string(),
        }
    }

    fn sample_postgres_line_source() -> super::PostgresLineSource {
        super::PostgresLineSource {
            producer: LogProducer::Postgres,
            transport: LogTransport::FileTail,
            path: "/tmp/postgres.log".to_string(),
        }
    }

    fn normalized_postgres_record(raw: &str) -> crate::logging::LogRecord {
        materialize_record(normalize_postgres_line(raw, sample_postgres_line_source()))
    }

    fn start_test_log() -> RunningTestLog {
        RunningTestLog::start()
    }

    fn sample_postgres_ingest_failure_event(cause: &str) -> PostgresIngestLogEvent {
        PostgresIngestLogEvent::StepOnceFailed {
            attempts: 2,
            suppressed: 7,
            cause: cause.to_string(),
        }
    }

    fn sample_non_utf8_postgres_line_event(path: &std::path::Path) -> super::PostgresLineLogEvent {
        super::postgres_line_event(
            LogProducer::Postgres,
            LogTransport::FileTail,
            path,
            vec![0xff_u8, 0x00, b'a', 0x80],
        )
    }

    #[test]
    fn ingest_error_rate_limiter_suppresses_and_reemits_with_count() {
        let mut limiter = IngestErrorRateLimiter::new(30_000);
        let key = IngestErrorKey {
            stage: "a",
            kind: "b",
            path: PathBuf::from("c"),
        };

        let first = limiter.record(key.clone(), 1_000);
        assert_eq!(
            first,
            super::RateLimitDecision {
                emit: true,
                suppressed: 0
            }
        );

        let suppressed = limiter.record(key.clone(), 2_000);
        assert_eq!(
            suppressed,
            super::RateLimitDecision {
                emit: false,
                suppressed: 0
            }
        );

        let reemit = limiter.record(key, 31_000);
        assert_eq!(
            reemit,
            super::RateLimitDecision {
                emit: true,
                suppressed: 1
            }
        );
    }

    #[test]
    fn ingest_issue_key_uses_stage_kind_and_path() {
        let issue = IngestIssue::new("first", "k1", PathBuf::from("/a").as_path(), "x");
        let key = issue.key();
        assert_eq!(key.stage, "first");
        assert_eq!(key.kind, "k1");
        assert_eq!(key.path, PathBuf::from("/a"));
    }

    #[test]
    fn render_iteration_errors_uses_first_issue_and_caps_extras() {
        let rendered = super::render_iteration_errors(&[
            IngestIssue::new("first", "k1", PathBuf::from("/a").as_path(), "x"),
            IngestIssue::new("second", "k2", PathBuf::from("/b").as_path(), "y"),
            IngestIssue::new("third", "k3", PathBuf::from("/c").as_path(), "z"),
            IngestIssue::new("fourth", "k4", PathBuf::from("/d").as_path(), "w"),
        ]);
        assert!(rendered.contains("count=4 stage=first kind=k1 path=/a error=x"));
        assert!(rendered.contains("stage=second kind=k2 path=/b error=y"));
        assert!(rendered.contains("stage=third kind=k3 path=/c error=z"));
        assert!(!rendered.contains("stage=fourth kind=k4 path=/d error=w"));
    }

    #[test]
    fn step_failure_event_encodes_internal_error_record() {
        let record = materialize_record(sample_postgres_ingest_failure_event(
            "stage=x kind=y path=/z error=boom",
        ));

        assert_eq!(record.severity_text, LogSeverity::Error);
        assert_eq!(record.event_name, "postgres_ingest.step_once_failed");
        assert_eq!(record.event_result, crate::logging::LogEventResult::Failed);
        assert_eq!(
            record.attributes.get("postgres_ingest.attempts"),
            Some(&Value::Number(serde_json::Number::from(2_u64)))
        );
        assert_eq!(
            record.attributes.get("postgres_ingest.suppressed"),
            Some(&Value::Number(serde_json::Number::from(7_u64)))
        );
    }

    #[test]
    fn map_pg_severity_maps_known_levels() {
        assert_eq!(map_pg_severity("ERROR"), LogSeverity::Error);
        assert_eq!(map_pg_severity("warning"), LogSeverity::Warn);
        assert_eq!(map_pg_severity("log"), LogSeverity::Info);
    }

    #[test]
    fn normalize_postgres_line_parses_jsonlog() {
        let raw = r#"{"error_severity":"LOG","message":"hello from json"}"#;
        let record = normalized_postgres_record(raw);
        assert_eq!(record.parser, LogParser::PostgresJson);
        assert_eq!(record.message, "hello from json");
        assert_eq!(record.severity_text, LogSeverity::Info);
        assert_eq!(record.severity_number, LogSeverity::Info.number());
        assert_eq!(record.hostname, "host-a");
    }

    #[test]
    fn normalize_postgres_line_parses_plain() {
        let raw = "2026-03-04 01:02:03 UTC [123] ERROR:  something bad";
        let record = normalized_postgres_record(raw);
        assert_eq!(record.parser, LogParser::PostgresPlain);
        assert_eq!(record.severity_text, LogSeverity::Error);
        assert_eq!(record.message, "something bad");
    }

    #[test]
    fn normalize_postgres_line_preserves_raw_on_failure() {
        let raw = "not a postgres log line";
        let record = normalized_postgres_record(raw);
        assert_eq!(record.parser, LogParser::Raw);
        assert_eq!(record.message, raw);
        assert_eq!(
            record.attributes.get("postgres.raw_line"),
            Some(&serde_json::Value::String(raw.to_string()))
        );
    }

    #[test]
    fn decode_line_encodes_non_utf8_bytes_as_hex() {
        let bytes = [0xff_u8, 0x00, b'a', 0x80];
        assert_eq!(decode_line(bytes.as_slice()), "non_utf8_bytes_hex=ff006180");
    }

    #[test]
    fn normalize_postgres_line_preserves_raw_on_non_utf8_failure() {
        let bytes = [0xff_u8, 0x00, b'a', 0x80];
        let raw = decode_line(bytes.as_slice());
        let record = normalized_postgres_record(raw.as_str());
        assert_eq!(record.parser, LogParser::Raw);
        assert_eq!(record.message, raw);
        assert_eq!(
            record.attributes.get("postgres.raw_line"),
            Some(&Value::String("non_utf8_bytes_hex=ff006180".to_string()))
        );
    }

    #[test]
    fn postgres_line_event_preserves_parse_failure_for_non_utf8() {
        let path = PathBuf::from("/tmp/pg.log");
        let record = materialize_record(sample_non_utf8_postgres_line_event(path.as_path()));
        assert_eq!(
            record.attributes.get("postgres.raw_line"),
            Some(&Value::String("non_utf8_bytes_hex=ff006180".to_string()))
        );
    }

    fn temp_dir(label: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pgtuskmaster-logging-cleanup-{label}-{}-{unique}",
            std::process::id()
        ))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cleanup_log_dir_enforces_max_files_and_protects_active_file() -> Result<(), WorkerError>
    {
        let dir = temp_dir("max-files");
        remove_dir_all_if_exists(&dir)?;
        std::fs::create_dir_all(&dir).map_err(|err| WorkerError::Message(err.to_string()))?;

        let protected = dir.join("active.log");
        std::fs::write(&protected, b"active\n")
            .map_err(|err| WorkerError::Message(err.to_string()))?;

        for i in 0..5 {
            let path = dir.join(format!("rotated-{i}.log"));
            std::fs::write(&path, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        }

        let mut issues = Vec::new();
        cleanup_log_dir(
            dir.as_path(),
            2,
            Duration::from_secs(365 * 24 * 60 * 60),
            Duration::from_secs(1),
            &[protected.as_path()],
            SystemTime::now() + Duration::from_secs(3600),
            &mut issues,
        )
        .await?;
        assert!(issues.is_empty());

        assert!(protected.exists());
        let mut remaining = 0usize;
        for entry in std::fs::read_dir(&dir).map_err(|err| WorkerError::Message(err.to_string()))? {
            let entry = entry.map_err(|err| WorkerError::Message(err.to_string()))?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("log") {
                remaining = remaining.saturating_add(1);
            }
        }
        // protected + max_files
        assert!(remaining <= 3);

        remove_dir_all_if_exists(&dir)?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cleanup_log_dir_never_deletes_known_active_signals() -> Result<(), WorkerError> {
        let dir = temp_dir("protected-basenames");
        remove_dir_all_if_exists(&dir)?;
        std::fs::create_dir_all(&dir).map_err(|err| WorkerError::Message(err.to_string()))?;

        let json = dir.join("postgres.json");
        let stderr = dir.join("postgres.stderr.log");
        let stdout = dir.join("postgres.stdout.log");
        std::fs::write(&json, b"{}\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        std::fs::write(&stderr, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        std::fs::write(&stdout, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;

        for i in 0..10 {
            let path = dir.join(format!("rotated-{i}.log"));
            std::fs::write(&path, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;
        }

        let mut issues = Vec::new();
        cleanup_log_dir(
            dir.as_path(),
            1,
            Duration::from_secs(365 * 24 * 60 * 60),
            Duration::from_secs(1),
            &[],
            SystemTime::now() + Duration::from_secs(3600),
            &mut issues,
        )
        .await?;
        assert!(issues.is_empty());

        assert!(json.exists());
        assert!(stderr.exists());
        assert!(stdout.exists());

        remove_dir_all_if_exists(&dir)?;
        Ok(())
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn cleanup_log_dir_surfaces_remove_failures() -> Result<(), WorkerError> {
        use std::os::unix::fs::PermissionsExt;

        let dir = temp_dir("remove-failure");
        remove_dir_all_if_exists(&dir)?;
        std::fs::create_dir_all(&dir).map_err(|err| WorkerError::Message(err.to_string()))?;

        let old = dir.join("old.log");
        std::fs::write(&old, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;

        let mut perms = std::fs::metadata(&dir)
            .map_err(|err| WorkerError::Message(err.to_string()))?
            .permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(&dir, perms)
            .map_err(|err| WorkerError::Message(err.to_string()))?;

        let mut issues = Vec::new();
        cleanup_log_dir(
            dir.as_path(),
            1,
            Duration::from_secs(1),
            Duration::from_secs(1),
            &[],
            SystemTime::now() + Duration::from_secs(3600),
            &mut issues,
        )
        .await?;
        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|issue| issue.stage == "cleanup.remove_file" && issue.kind == "remove_file"));
        assert!(old.exists());

        let mut perms = std::fs::metadata(&dir)
            .map_err(|err| WorkerError::Message(err.to_string()))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dir, perms)
            .map_err(|err| WorkerError::Message(err.to_string()))?;

        remove_dir_all_if_exists(&dir)?;
        Ok(())
    }

    mod real_binary {
        use std::path::{Path, PathBuf};
        use std::time::Duration;

        use tokio::process::Command;
        use tokio::time::Instant;

        use crate::dcs::{DcsMemberState, DcsSnapshot};
        use crate::dev_support::binaries::{
            require_pg16_bin_for_real_tests, require_pg16_process_binaries_for_real_tests,
        };
        use crate::dev_support::namespace::NamespaceGuard;
        use crate::dev_support::pg16::{
            prepare_pgdata_dir, spawn_pg16_for_vanilla_postgres, PgInstanceSpec,
        };
        use crate::dev_support::ports::allocate_ports;
        use crate::logging::LogRecord;
        use crate::pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus};
        use crate::process::jobs::{
            PostgresStartIntent, ProcessIntent, ReplicaProvisionIntent, ShutdownMode,
        };
        use crate::process::state::{
            ProcessCadence, ProcessIntentRequest, ProcessObservedState, ProcessRuntime,
            ProcessState, ProcessWorkerCtx,
        };
        use crate::process::worker::{step_once as process_step_once, TokioCommandRunner};
        use crate::state::{
            new_state_channel, JobId, MemberId, TimelineId, WalLsn, WorkerError, WorkerStatus,
        };

        use super::super::{
            step_once as ingest_step_once, PostgresIngestWorkerCtx, PostgresIngestWorkerState,
        };
        use super::{
            start_test_log, RunningTestLog, REAL_INGEST_RETRY_SLEEP,
            REAL_PROCESS_WORKER_POLL_INTERVAL, REAL_PSQL_RETRY_SLEEP,
        };

        async fn wait_for_process_idle_success(
            ctx: &mut ProcessWorkerCtx<'_>,
            job_id: &JobId,
            timeout: Duration,
        ) -> Result<(), WorkerError> {
            wait_for_process_idle_success_with_debug(ctx, job_id, timeout, None).await
        }

        async fn wait_for_process_idle_success_with_debug(
            ctx: &mut ProcessWorkerCtx<'_>,
            job_id: &JobId,
            timeout: Duration,
            debug_log_path: Option<&PathBuf>,
        ) -> Result<(), WorkerError> {
            if !wait_for_process_condition(
                ctx,
                None,
                timeout,
                Duration::from_millis(10),
                |ctx, _| {
                    if let ProcessState::Idle {
                        last_outcome: Some(outcome),
                        ..
                    } = &ctx.state_channel.current
                    {
                        match outcome {
                            crate::process::state::JobOutcome::Success { id, .. }
                                if id == job_id =>
                            {
                                return Ok(true);
                            }
                            crate::process::state::JobOutcome::Failure { id, error, .. }
                                if id == job_id =>
                            {
                                return Err(WorkerError::Message(format!(
                                    "process job {} failed unexpectedly: {error}{}",
                                    job_id.0,
                                    debug_tail_suffix(debug_log_path)
                                )));
                            }
                            crate::process::state::JobOutcome::Timeout { id, .. }
                                if id == job_id =>
                            {
                                return Err(WorkerError::Message(format!(
                                    "process job {} timed out unexpectedly{}",
                                    job_id.0,
                                    debug_tail_suffix(debug_log_path)
                                )));
                            }
                            _ => {}
                        }
                    }
                    Ok(false)
                },
            )
            .await?
            {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for job {} success",
                    job_id.0
                )));
            }
            Ok(())
        }

        async fn wait_for_process_condition<Done>(
            ctx: &mut ProcessWorkerCtx<'_>,
            test_log: Option<&RunningTestLog>,
            timeout: Duration,
            poll_interval: Duration,
            mut done: Done,
        ) -> Result<bool, WorkerError>
        where
            Done: FnMut(&ProcessWorkerCtx<'_>, &[LogRecord]) -> Result<bool, WorkerError>,
        {
            let started = Instant::now();
            let mut collected: Vec<LogRecord> = Vec::new();
            loop {
                process_step_once(ctx).await?;
                if let Some(test_log) = test_log {
                    collected.extend(test_log.take().await);
                }
                if done(ctx, &collected)? {
                    return Ok(true);
                }
                if started.elapsed() >= timeout {
                    return Ok(false);
                }
                tokio::time::sleep(poll_interval).await;
            }
        }

        fn debug_tail_suffix(debug_log_path: Option<&PathBuf>) -> String {
            debug_log_path.map_or_else(String::new, |path| {
                format!(
                    "\n--- debug tail {} ---\n{}",
                    path.display(),
                    tail_file_best_effort(path, 60)
                )
            })
        }

        fn tail_file_best_effort(path: &Path, max_lines: usize) -> String {
            let contents = match std::fs::read_to_string(path) {
                Ok(contents) => contents,
                Err(err) => return format!("(failed to read {}: {err})", path.display()),
            };
            let mut lines = contents.lines().collect::<Vec<_>>();
            if lines.len() > max_lines {
                let start = lines.len().saturating_sub(max_lines);
                lines.drain(0..start);
            }
            lines.join("\n")
        }

        fn build_process_worker_ctx(
            cfg: &crate::config_v2::RuntimeConfigV2,
            log: crate::logging::LogSender,
            dcs: DcsSnapshot,
        ) -> Result<
            (
                ProcessWorkerCtx<'static>,
                crate::state::StateSubscriber<ProcessState>,
                tokio::sync::mpsc::UnboundedSender<ProcessIntentRequest>,
            ),
            WorkerError,
        > {
            let cfg = Box::leak(Box::new(cfg.clone()));
            let (_dcs_publisher, dcs_subscriber) = new_state_channel(dcs);
            Ok(crate::process::worker::bootstrap_with_runtime(
                cfg,
                ProcessObservedState {
                    dcs: dcs_subscriber,
                },
                ProcessCadence {
                    poll_interval: REAL_PROCESS_WORKER_POLL_INTERVAL,
                    now: Box::new(crate::process::worker::system_now_unix_millis),
                },
                ProcessRuntime {
                    log,
                    command_runner: Box::new(TokioCommandRunner),
                },
            ))
        }

        fn is_transient_psql_failure(stderr: &str) -> bool {
            let normalized = stderr.to_ascii_lowercase();
            normalized.contains("the database system is starting up")
                || normalized.contains("the database system is shutting down")
                || normalized.contains("not yet accepting connections")
                || normalized.contains("could not connect to server")
                || normalized.contains("connection refused")
        }

        async fn run_psql_query_with_retry(
            psql_bin: &PathBuf,
            port: u16,
            query: &str,
            timeout: Duration,
        ) -> Result<(), WorkerError> {
            let deadline = Instant::now() + timeout;
            let mut last_stderr = String::new();
            let mut last_stdout = String::new();

            while Instant::now() < deadline {
                let mut cmd = Command::new(psql_bin);
                cmd.arg("-h")
                    .arg("127.0.0.1")
                    .arg("-p")
                    .arg(port.to_string())
                    .arg("-U")
                    .arg("postgres")
                    .arg("-d")
                    .arg("postgres")
                    .arg("-c")
                    .arg(query);

                let output = cmd
                    .output()
                    .await
                    .map_err(|err| WorkerError::Message(format!("psql spawn failed: {err}")))?;

                if output.status.success() {
                    return Ok(());
                }

                last_stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                last_stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

                if !is_transient_psql_failure(&last_stderr) {
                    return Err(WorkerError::Message(format!(
                        "psql exited unsuccessfully: {} (non-transient)\n--- stdout ---\n{}\n--- stderr ---\n{}",
                        output.status,
                        last_stdout,
                        last_stderr
                    )));
                }

                tokio::time::sleep(REAL_PSQL_RETRY_SLEEP).await;
            }

            Err(WorkerError::Message(format!(
                "timed out waiting for psql readiness after {:?}\n--- last stdout ---\n{}\n--- last stderr ---\n{}",
                timeout, last_stdout, last_stderr
            )))
        }

        #[tokio::test(flavor = "current_thread")]
        async fn ingests_jsonlog_and_stderr_files_from_real_postgres() -> Result<(), WorkerError> {
            let postgres_bin = require_pg16_bin_for_real_tests("postgres")?;
            let initdb_bin = require_pg16_bin_for_real_tests("initdb")?;
            let psql_bin = require_pg16_bin_for_real_tests("psql")?;

            let guard = NamespaceGuard::new("log-jsonlog-stderr")?;
            let ns = guard.namespace()?;

            let data_dir = prepare_pgdata_dir(ns, "node-a")?;
            let mut reservation = allocate_ports(1)?;
            let port = reservation.as_slice()[0];
            let socket_dir = ns.child_dir("pg16/node-a/socket");
            let log_dir = ns.child_dir("logs/pg16-node-a");

            let jsonlog_path = log_dir.join("postgres.json");
            std::fs::create_dir_all(&log_dir).map_err(|err| {
                WorkerError::Message(format!(
                    "create postgres ingest log dir {} failed: {err}",
                    log_dir.display()
                ))
            })?;
            std::fs::write(&jsonlog_path, b"").map_err(|err| {
                WorkerError::Message(format!(
                    "seed postgres ingest jsonlog file {} failed: {err}",
                    jsonlog_path.display()
                ))
            })?;

            let conf_lines = vec![
                "logging_collector = on".to_string(),
                "log_destination = 'jsonlog,stderr'".to_string(),
                format!("log_directory = '{}'", log_dir.display()),
                "log_filename = 'postgres.json'".to_string(),
                "log_statement = 'all'".to_string(),
            ];

            let spec = PgInstanceSpec {
                postgres_bin,
                initdb_bin,
                data_dir,
                socket_dir,
                log_dir: log_dir.clone(),
                port,
                startup_timeout: Duration::from_secs(10),
            };
            reservation.release_port(port).map_err(|err| {
                WorkerError::Message(format!("release reserved port failed: {err}"))
            })?;
            // This test validates raw PostgreSQL log emission and ingest parsing, not
            // pgtuskmaster-managed startup ownership, so it uses the explicit
            // vanilla-Postgres config exception path.
            let mut pg = spawn_pg16_for_vanilla_postgres(spec, &conf_lines).await?;

            let mut cfg = crate::config_v2::trace_logging_test_config()
                .map_err(|err| WorkerError::Message(err.to_string()))?;
            cfg.logging.postgres.log_dir = log_dir;
            cfg.postgres.log_file = ns.child_dir("runtime/pg_ctl.log");

            let test_log = start_test_log();
            let ctx = PostgresIngestWorkerCtx {
                cfg: &cfg,
                log: test_log.sender(),
            };
            let mut state = PostgresIngestWorkerState::new(ctx.cfg);

            // Prime ingestion offsets and then generate logs.
            ingest_step_once(&ctx, &mut state).await?;

            run_psql_query_with_retry(&psql_bin, port, "SELECT 1;", Duration::from_secs(10))
                .await?;

            let deadline = Instant::now() + Duration::from_secs(3);
            let mut collected = Vec::new();
            while Instant::now() < deadline {
                ingest_step_once(&ctx, &mut state).await?;
                collected.extend(test_log.take().await);
                let saw_json = collected
                    .iter()
                    .any(|r| r.parser == crate::logging::LogParser::PostgresJson);
                let saw_stderr = collected.iter().any(|r| {
                    r.attributes
                        .get("postgres.path")
                        .and_then(|value| value.as_str())
                        .is_some_and(|path| path.contains("postgres.stderr.log"))
                });
                if saw_json && saw_stderr {
                    pg.shutdown().await?;
                    return Ok(());
                }
                tokio::time::sleep(REAL_INGEST_RETRY_SLEEP).await;
            }

            pg.shutdown().await?;
            drop(reservation);
            Err(WorkerError::Message(
                "timed out waiting for jsonlog+stderr ingestion".to_string(),
            ))
        }

        #[tokio::test(flavor = "current_thread")]
        async fn ingests_pg_ctl_log_file_and_captures_pg_tool_output() -> Result<(), WorkerError> {
            let binaries = require_pg16_process_binaries_for_real_tests()?;

            let guard = NamespaceGuard::new("log-pgctl")?;
            let ns = guard.namespace()?;

            let mut reservation = allocate_ports(1)?;
            let port = reservation.as_slice()[0];

            let data_dir = prepare_pgdata_dir(ns, "node-a")?;
            let socket_dir = ns.child_dir("sock");
            let log_file = ns.child_dir("runtime/pg_ctl.log");
            let log_dir = ns.child_dir("logs/pg16-node-a");
            std::fs::create_dir_all(&socket_dir)
                .map_err(|err| WorkerError::Message(format!("create socket_dir failed: {err}")))?;
            if let Some(parent) = log_file.parent() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    WorkerError::Message(format!("create log file parent failed: {err}"))
                })?;
            }
            std::fs::create_dir_all(&log_dir)
                .map_err(|err| WorkerError::Message(format!("create log_dir failed: {err}")))?;
            let jsonlog_path = log_dir.join("postgres.json");
            std::fs::write(&jsonlog_path, b"")
                .map_err(|err| WorkerError::Message(format!("seed jsonlog failed: {err}")))?;

            let mut cfg = crate::config_v2::trace_logging_test_config()
                .map_err(|err| WorkerError::Message(err.to_string()))?;
            cfg.process.binaries = binaries.clone();
            cfg.process.timeouts.bootstrap = Duration::from_secs(30);
            cfg.process.timeouts.fencing = Duration::from_secs(30);
            cfg.postgres.data_dir = data_dir.clone();
            cfg.postgres.pg_hba_file = data_dir.join("pgtm.pg_hba.conf");
            cfg.postgres.pg_ident_file = data_dir.join("pgtm.pg_ident.conf");
            cfg.postgres.socket_dir = socket_dir.clone();
            cfg.postgres.listen_port = port;
            cfg.postgres.cluster_advertise =
                crate::state::PgRoute::tcp(cfg.postgres.listen_host.clone(), port).map_err(
                    |err| WorkerError::Message(format!("test advertise route failed: {err}")),
                )?;
            cfg.postgres.log_file = log_file.clone();
            cfg.postgres
                .extra_gucs
                .insert("log_filename".to_string(), "postgres.json".to_string());
            cfg.postgres
                .extra_gucs
                .insert("log_directory".to_string(), log_dir.display().to_string());
            cfg.postgres
                .extra_gucs
                .insert("log_statement".to_string(), "all".to_string());
            cfg.logging.postgres.log_dir = log_dir.clone();

            let test_log = start_test_log();

            let (mut process_ctx, _process_state_subscriber, tx) =
                build_process_worker_ctx(&cfg, test_log.sender(), DcsSnapshot::starting())?;

            let ingest_ctx = PostgresIngestWorkerCtx {
                cfg: &cfg,
                log: test_log.sender(),
            };
            let mut ingest_state = PostgresIngestWorkerState::new(ingest_ctx.cfg);

            let bootstrap_id = JobId("bootstrap".to_string());
            tx.send(ProcessIntentRequest {
                id: bootstrap_id.clone(),
                intent: ProcessIntent::Bootstrap,
            })
            .map_err(|_| WorkerError::Message("send bootstrap job failed".to_string()))?;

            wait_for_process_idle_success(&mut process_ctx, &bootstrap_id, Duration::from_secs(30))
                .await?;

            reservation.release_port(port).map_err(|err| {
                WorkerError::Message(format!("release reserved port failed: {err}"))
            })?;
            let start_id = JobId("start".to_string());
            tx.send(ProcessIntentRequest {
                id: start_id.clone(),
                intent: ProcessIntent::Start(PostgresStartIntent::Primary),
            })
            .map_err(|_| WorkerError::Message("send start job failed".to_string()))?;

            if !wait_for_process_condition(
                &mut process_ctx,
                Some(&test_log),
                Duration::from_secs(60),
                Duration::from_millis(10),
                |process_ctx, collected_for_debug| {
                    if let ProcessState::Idle {
                        last_outcome: Some(outcome),
                        ..
                    } = &process_ctx.state_channel.current
                    {
                        match outcome {
                            crate::process::state::JobOutcome::Success { id, .. }
                                if *id == start_id =>
                            {
                                return Ok(true);
                            }
                            crate::process::state::JobOutcome::Failure { id, error, .. }
                                if *id == start_id =>
                            {
                                let pg_ctl_tail = tail_file_best_effort(&log_file, 120);
                                let postgres_json_tail = tail_file_best_effort(&jsonlog_path, 120);
                                let postmaster_pid =
                                    tail_file_best_effort(&data_dir.join("postmaster.pid"), 60);

                                let mut pg_tool_lines = collected_for_debug
                                    .iter()
                                    .filter(|record| {
                                        record.producer == crate::logging::LogProducer::PgTool
                                            && record
                                                .attributes
                                                .get("job.kind")
                                                .and_then(|v| v.as_str())
                                                == Some("start_primary")
                                    })
                                    .map(|record| {
                                        format!(
                                            "{:?} {}: {}",
                                            record.transport,
                                            record
                                                .attributes
                                                .get("postgres.path")
                                                .and_then(|value| value.as_str())
                                                .map_or("<none>", |value| value),
                                            record.message
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                if pg_tool_lines.len() > 60 {
                                    let start = pg_tool_lines.len().saturating_sub(60);
                                    pg_tool_lines.drain(0..start);
                                }
                                let pg_tool_debug = if pg_tool_lines.is_empty() {
                                    "(no captured pg_tool stdout/stderr lines for start_primary)"
                                        .to_string()
                                } else {
                                    pg_tool_lines.join("\n")
                                };

                                return Err(WorkerError::Message(format!(
                                    "process job {} failed unexpectedly: {error}\n--- pg_ctl log tail {} ---\n{}\n--- postgres jsonlog tail {} ---\n{}\n--- postmaster.pid tail {} ---\n{}\n--- captured pg_tool output (start_primary) ---\n{}",
                                    start_id.0,
                                    log_file.display(),
                                    pg_ctl_tail,
                                    jsonlog_path.display(),
                                    postgres_json_tail,
                                    data_dir.join("postmaster.pid").display(),
                                    postmaster_pid,
                                    pg_tool_debug
                                )));
                            }
                            _ => {}
                        }
                    }
                    Ok(false)
                },
            )
            .await?
            {
                return Err(WorkerError::Message(
                    "timed out waiting for start_primary job success".to_string(),
                ));
            }

            // Pump ingestion a bit to collect pg_ctl log lines.
            let psql_bin = require_pg16_bin_for_real_tests("psql")?;
            let mut cmd = Command::new(psql_bin);
            cmd.arg("-h")
                .arg("127.0.0.1")
                .arg("-p")
                .arg(port.to_string())
                .arg("-U")
                .arg("postgres")
                .arg("-d")
                .arg("postgres")
                .arg("-c")
                .arg("SELECT 1;");
            let status = cmd
                .status()
                .await
                .map_err(|err| WorkerError::Message(format!("psql spawn failed: {err}")))?;
            if !status.success() {
                return Err(WorkerError::Message(format!(
                    "psql pg_switch_wal exited unsuccessfully: {status}"
                )));
            }

            let deadline = Instant::now() + Duration::from_secs(10);
            let mut collected: Vec<LogRecord> = Vec::new();
            while Instant::now() < deadline {
                ingest_step_once(&ingest_ctx, &mut ingest_state).await?;
                process_step_once(&mut process_ctx).await?;
                collected.extend(test_log.take().await);
                let saw_pg_ctl_log = collected.iter().any(|r| {
                    r.producer == crate::logging::LogProducer::Postgres
                        && r.attributes
                            .get("postgres.path")
                            .and_then(|value| value.as_str())
                            .is_some_and(|path| path.contains("pg_ctl.log"))
                });
                let saw_pg_tool = collected.iter().any(|r| {
                    r.producer == crate::logging::LogProducer::PgTool
                        && (r.transport == crate::logging::LogTransport::ChildStdout
                            || r.transport == crate::logging::LogTransport::ChildStderr)
                });
                let saw_jsonlog = collected.iter().any(|r| {
                    r.producer == crate::logging::LogProducer::Postgres
                        && r.parser == crate::logging::LogParser::PostgresJson
                });
                if saw_pg_ctl_log && saw_pg_tool && saw_jsonlog {
                    break;
                }
                tokio::time::sleep(REAL_INGEST_RETRY_SLEEP).await;
            }

            let stop_id = JobId("stop".to_string());
            tx.send(ProcessIntentRequest {
                id: stop_id.clone(),
                intent: ProcessIntent::Demote(ShutdownMode::Fast),
            })
            .map_err(|_| WorkerError::Message("send stop job failed".to_string()))?;
            wait_for_process_idle_success(&mut process_ctx, &stop_id, Duration::from_secs(30))
                .await?;

            // One more ingestion pass after shutdown to catch any final flushes.
            ingest_step_once(&ingest_ctx, &mut ingest_state).await?;

            let mut all_records = collected;
            all_records.extend(test_log.take().await);

            let saw_pg_ctl_log = all_records.iter().any(|r| {
                r.producer == crate::logging::LogProducer::Postgres
                    && r.attributes
                        .get("postgres.path")
                        .and_then(|value| value.as_str())
                        .is_some_and(|path| path.contains("pg_ctl.log"))
            });
            let saw_pg_tool = all_records.iter().any(|r| {
                r.producer == crate::logging::LogProducer::PgTool
                    && r.attributes
                        .get("job.kind")
                        .and_then(|v| v.as_str())
                        .is_some()
            });
            let saw_jsonlog = all_records.iter().any(|r| {
                r.producer == crate::logging::LogProducer::Postgres
                    && r.parser == crate::logging::LogParser::PostgresJson
            });
            if !saw_pg_ctl_log {
                return Err(WorkerError::Message(
                    "missing ingested pg_ctl log file records".to_string(),
                ));
            }
            if !saw_pg_tool {
                return Err(WorkerError::Message(
                    "missing captured pg tool stdout/stderr records".to_string(),
                ));
            }
            if !saw_jsonlog {
                return Err(WorkerError::Message(
                    "missing ingested postgres jsonlog records".to_string(),
                ));
            }

            drop(reservation);
            Ok(())
        }

        #[tokio::test(flavor = "current_thread")]
        async fn captures_helper_binary_stdout_stderr_on_failure() -> Result<(), WorkerError> {
            let binaries = require_pg16_process_binaries_for_real_tests()?;

            let guard = NamespaceGuard::new("log-pgtool")?;
            let ns = guard.namespace()?;

            let data_dir = ns.child_dir("pg_basebackup/out");
            std::fs::create_dir_all(&data_dir)
                .map_err(|err| WorkerError::Message(format!("create data_dir failed: {err}")))?;

            let mut cfg = crate::config_v2::trace_logging_test_config()
                .map_err(|err| WorkerError::Message(err.to_string()))?;
            cfg.process.binaries = binaries;

            let test_log = start_test_log();

            let dcs = DcsSnapshot::quorum(
                None,
                crate::state::SwitchoverState::None,
                std::collections::BTreeMap::from([(
                    MemberId("node-b".to_string()),
                    DcsMemberState {
                        cluster_postgres: crate::state::PgRoute::tcp("127.0.0.1".to_string(), 9)
                            .map_err(|err| {
                                WorkerError::Message(format!("test dcs target failed: {err}"))
                            })?,
                        operator_postgres: None,
                        operator_api: None,
                        postgres: PgInfoState::Primary {
                            common: PgInfoCommon {
                                worker: WorkerStatus::Running,
                                sql: SqlStatus::Healthy,
                                readiness: Readiness::Ready,
                                timeline: Some(TimelineId(1)),
                                system_identifier: None,
                                pg_config: PgConfig {
                                    port: Some(9),
                                    hot_standby: None,
                                    primary_conninfo: None,
                                    primary_slot_name: None,
                                    extra: std::collections::BTreeMap::new(),
                                },
                                last_refresh_at: None,
                            },
                            wal_lsn: WalLsn(0),
                            slots: Vec::new(),
                        },
                    },
                )]),
            );
            let (mut ctx, _process_state_subscriber, tx) =
                build_process_worker_ctx(&cfg, test_log.sender(), dcs)?;

            let job_id = JobId("basebackup-fail".to_string());
            tx.send(ProcessIntentRequest {
                id: job_id.clone(),
                intent: ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup {
                    leader: MemberId("node-b".to_string()),
                }),
            })
            .map_err(|_| WorkerError::Message("send basebackup job failed".to_string()))?;

            if wait_for_process_condition(
                &mut ctx,
                Some(&test_log),
                Duration::from_secs(10),
                REAL_INGEST_RETRY_SLEEP,
                |_, collected| {
                    Ok(collected.iter().any(|r| {
                        r.producer == crate::logging::LogProducer::PgTool
                            && r.transport == crate::logging::LogTransport::ChildStderr
                            && r.attributes.get("job.kind").and_then(|v| v.as_str())
                                == Some("base_backup")
                    }))
                },
            )
            .await?
            {
                return Ok(());
            }

            Err(WorkerError::Message(
                "timed out waiting for captured pg_basebackup stderr".to_string(),
            ))
        }
    }
}
