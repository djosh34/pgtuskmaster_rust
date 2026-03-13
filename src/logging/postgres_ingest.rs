use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, SystemTime};

use serde_json::Value;

use crate::config::{LogCleanupConfig, RuntimeConfig};
use crate::logging::{
    AppEvent, AppEventHeader, LogHandle, LogParser, LogProducer, LogTransport,
    PostgresLineRecordBuilder, RawRecordBuilder, SeverityText, StructuredFields,
};
use crate::state::WorkerError;

use super::tailer::{DirTailers, FileTailer, StartPosition};

pub(crate) struct PostgresIngestWorkerCtx {
    pub(crate) cfg: RuntimeConfig,
    pub(crate) log: LogHandle,
}

const POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS: u64 = 30_000;
const POSTGRES_INGEST_MAX_BYTES_PER_FILE: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct IngestErrorKey {
    stage: String,
    kind: String,
    path: String,
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

pub(crate) async fn run(ctx: PostgresIngestWorkerCtx) -> Result<(), WorkerError> {
    let mut state = PostgresIngestWorkerState::new(&ctx.cfg);
    let mut limiter = IngestErrorRateLimiter::new(POSTGRES_INGEST_ERROR_RATE_LIMIT_WINDOW_MS);
    let mut consecutive_failures = 0u32;
    loop {
        if ctx.cfg.logging.postgres.enabled {
            match step_once(&ctx, &mut state).await {
                Ok(()) => {
                    if consecutive_failures > 0 {
                        emit_ingest_retry_recovered(&ctx.log, consecutive_failures)?;
                        consecutive_failures = 0;
                    }
                }
                Err(error) => {
                    consecutive_failures = consecutive_failures.saturating_add(1);
                    let now_ms = crate::logging::system_now_unix_millis();
                    let key = ingest_error_key_best_effort(&error);
                    let decision = limiter.record(key, now_ms);
                    if decision.emit {
                        emit_ingest_step_failure(
                            &ctx.log,
                            &error,
                            consecutive_failures,
                            decision.suppressed,
                        )?;
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(
            ctx.cfg.logging.postgres.poll_interval_ms,
        ))
        .await;
    }
}

fn ingest_error_key_best_effort(error: &WorkerError) -> IngestErrorKey {
    let msg = error.to_string();

    let mut stage = "unknown".to_string();
    let mut kind = "unknown".to_string();
    let mut path = "unknown".to_string();

    for token in msg.split_whitespace() {
        if stage == "unknown" {
            if let Some(value) = token.strip_prefix("stage=") {
                stage = value.to_string();
                continue;
            }
        }
        if kind == "unknown" {
            if let Some(value) = token.strip_prefix("kind=") {
                kind = value.to_string();
                continue;
            }
        }
        if path == "unknown" {
            if let Some(value) = token.strip_prefix("path=") {
                path = value.to_string();
                continue;
            }
        }
        if stage != "unknown" && kind != "unknown" && path != "unknown" {
            break;
        }
    }

    IngestErrorKey { stage, kind, path }
}

fn ingest_event(severity: SeverityText, message: &str, name: &str, result: &str) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(name, "postgres_ingest", result),
    )
}

fn emit_ingest_event(
    log: &LogHandle,
    origin: &str,
    event: AppEvent,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    log.emit_app_event(origin, event)
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
}

fn emit_ingest_step_failure(
    log: &LogHandle,
    error: &WorkerError,
    attempts: u32,
    suppressed: u64,
) -> Result<(), WorkerError> {
    let mut event = ingest_event(
        SeverityText::Error,
        "postgres ingest step_once failed",
        "postgres_ingest.step_once_failed",
        "failed",
    );
    let fields = event.fields_mut();
    fields.insert("attempts", attempts);
    fields.insert("suppressed", suppressed);
    fields.insert("error", error.to_string());
    emit_ingest_event(
        log,
        "postgres_ingest::run",
        event,
        "postgres ingest error log emit failed",
    )
}

fn emit_ingest_retry_recovered(log: &LogHandle, attempts: u32) -> Result<(), WorkerError> {
    let mut event = ingest_event(
        SeverityText::Info,
        "postgres ingest recovered",
        "postgres_ingest.recovered",
        "recovered",
    );
    event.fields_mut().insert("attempts", attempts);
    emit_ingest_event(
        log,
        "postgres_ingest::run",
        event,
        "postgres ingest recovered log emit failed",
    )
}

struct PostgresIngestWorkerState {
    pg_ctl_log: FileTailer,
    dir_tailers: DirTailers,
}

impl PostgresIngestWorkerState {
    fn new(cfg: &RuntimeConfig) -> Self {
        let pg_ctl_log_file = match cfg.logging.postgres.pg_ctl_log_file.clone() {
            Some(path) => path,
            None => cfg.postgres.log_file.clone(),
        };

        Self {
            pg_ctl_log: FileTailer::new(pg_ctl_log_file, StartPosition::Beginning),
            dir_tailers: DirTailers::default(),
        }
    }
}

async fn step_once(
    ctx: &PostgresIngestWorkerCtx,
    state: &mut PostgresIngestWorkerState,
) -> Result<(), WorkerError> {
    let max_bytes_per_file = POSTGRES_INGEST_MAX_BYTES_PER_FILE;
    let mut pg_ctl_lines_emitted: u64 = 0;
    let mut log_dir_lines_emitted: u64 = 0;
    let mut log_dir_files_tailed: u64 = 0;

    #[derive(Clone, Debug)]
    struct IterationIssue {
        stage: &'static str,
        kind: &'static str,
        path: String,
        error: String,
    }

    fn encode_path_token(path: &Path) -> String {
        path.display().to_string().replace(' ', "%20")
    }

    fn file_name_best_effort(path: &Path) -> String {
        match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => "log".to_string(),
        }
    }

    fn push_issue(
        issues: &mut Vec<IterationIssue>,
        stage: &'static str,
        kind: &'static str,
        path: &Path,
        error: WorkerError,
    ) {
        issues.push(IterationIssue {
            stage,
            kind,
            path: encode_path_token(path),
            error: error.to_string(),
        });
    }

    let mut issues: Vec<IterationIssue> = Vec::new();

    match state.pg_ctl_log.read_new_lines(max_bytes_per_file).await {
        Ok(pg_lines) => {
            for line in pg_lines {
                if let Err(err) = emit_postgres_line(
                    &ctx.log,
                    LogProducer::Postgres,
                    LogTransport::FileTail,
                    "pg_ctl_log_file",
                    state.pg_ctl_log.path(),
                    line,
                ) {
                    push_issue(
                        &mut issues,
                        "pg_ctl_log_file.emit",
                        "log.emit_record",
                        state.pg_ctl_log.path(),
                        err,
                    );
                } else {
                    pg_ctl_lines_emitted = pg_ctl_lines_emitted.saturating_add(1);
                }
            }
        }
        Err(err) => {
            push_issue(
                &mut issues,
                "pg_ctl_log_file.read",
                "tailer.read_new_lines",
                state.pg_ctl_log.path(),
                err,
            );
        }
    }

    if let Some(dir) = ctx.cfg.logging.postgres.log_dir.as_ref() {
        if let Err(err) = discover_log_dir(&mut state.dir_tailers, dir).await {
            push_issue(&mut issues, "log_dir.discover", "read_dir", dir, err);
        }

        for (path, tailer) in state.dir_tailers.iter_mut() {
            log_dir_files_tailed = log_dir_files_tailed.saturating_add(1);
            let origin = format!("postgres_log_dir:{}", file_name_best_effort(path));
            match tailer.read_new_lines(max_bytes_per_file).await {
                Ok(lines) => {
                    for line in lines {
                        if let Err(err) = emit_postgres_line(
                            &ctx.log,
                            LogProducer::Postgres,
                            LogTransport::FileTail,
                            origin.as_str(),
                            tailer.path(),
                            line,
                        ) {
                            push_issue(
                                &mut issues,
                                "log_dir.emit",
                                "log.emit_record",
                                tailer.path(),
                                err,
                            );
                        } else {
                            log_dir_lines_emitted = log_dir_lines_emitted.saturating_add(1);
                        }
                    }
                }
                Err(err) => {
                    push_issue(
                        &mut issues,
                        "log_dir.read",
                        "tailer.read_new_lines",
                        tailer.path(),
                        err,
                    );
                }
            }
        }

        if ctx.cfg.logging.postgres.cleanup.enabled {
            let protected: Vec<&Path> = vec![state.pg_ctl_log.path()];

            match cleanup_log_dir(
                dir,
                &ctx.cfg.logging.postgres.cleanup,
                protected.as_slice(),
                SystemTime::now(),
            )
            .await
            {
                Ok(report) => {
                    if report.issue_count > 0 {
                        let stage = "log_dir.cleanup";
                        let kind = "cleanup.issues";
                        let error = WorkerError::Message(format!(
                            "cleanup had issues: issue_count={} first={}",
                            report.issue_count, report.first_issue
                        ));
                        push_issue(&mut issues, stage, kind, dir, error);
                    }
                }
                Err(err) => {
                    push_issue(&mut issues, "log_dir.cleanup", "cleanup.fatal", dir, err);
                }
            }
        }
    }

    if issues.is_empty() {
        let mut event = ingest_event(
            SeverityText::Debug,
            "postgres ingest iteration ok",
            "postgres_ingest.iteration",
            "ok",
        );
        let fields = event.fields_mut();
        fields.insert("pg_ctl_lines_emitted", pg_ctl_lines_emitted);
        fields.insert("log_dir_files_tailed", log_dir_files_tailed);
        fields.insert("log_dir_lines_emitted", log_dir_lines_emitted);
        fields.insert("dir_tailers", state.dir_tailers.len());
        emit_ingest_event(
            &ctx.log,
            "postgres_ingest::step_once",
            event,
            "postgres ingest debug log emit failed",
        )?;
        return Ok(());
    }

    let first = match issues.first() {
        Some(first) => format!(
            "stage={} kind={} path={} error={}",
            first.stage, first.kind, first.path, first.error
        ),
        None => "stage=unknown kind=unknown path=unknown error=unknown".to_string(),
    };

    let mut extra = Vec::new();
    for issue in issues.iter().skip(1).take(2) {
        extra.push(format!(
            "stage={} kind={} path={} error={}",
            issue.stage, issue.kind, issue.path, issue.error
        ));
    }
    let extra_suffix = if extra.is_empty() {
        String::new()
    } else {
        format!(" extra=[{}]", extra.join(" | "))
    };

    Err(WorkerError::Message(format!(
        "postgres_ingest iteration_errors count={} {}{}",
        issues.len(),
        first,
        extra_suffix
    )))
}

async fn discover_log_dir(tailers: &mut DirTailers, dir: &Path) -> Result<(), WorkerError> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "read_dir failed for {}: {err}",
                dir.display()
            )));
        }
    };

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|err| WorkerError::Message(format!("read_dir entry failed: {err}")))?
    {
        let path = entry.path();
        let is_file = match entry.file_type().await {
            Ok(ft) => ft.is_file(),
            Err(err) => {
                return Err(WorkerError::Message(format!(
                    "stage=log_dir.discover kind=file_type path={} error={err}",
                    path.display()
                )));
            }
        };
        if !is_file {
            continue;
        }

        let matches = matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("log") | Some("json")
        );
        if !matches {
            continue;
        }

        let start = match path.file_name().and_then(|s| s.to_str()) {
            Some("postgres.stderr.log") | Some("postgres.stdout.log") => StartPosition::Beginning,
            _ => StartPosition::End,
        };
        tailers.ensure_file(path, start);
    }
    Ok(())
}

async fn cleanup_log_dir(
    dir: &Path,
    cleanup: &LogCleanupConfig,
    protected_paths: &[&Path],
    now: SystemTime,
) -> Result<CleanupReport, WorkerError> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(CleanupReport::empty()),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "cleanup read_dir failed for {}: {err}",
                dir.display()
            )));
        }
    };

    let protected_basenames: [&str; 3] = [
        "postgres.json",
        "postgres.stderr.log",
        "postgres.stdout.log",
    ];

    let mut issues: Vec<String> = Vec::new();
    let mut candidates = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|err| WorkerError::Message(format!("cleanup readdir entry failed: {err}")))?
    {
        let path = entry.path();
        let is_file = match entry.file_type().await {
            Ok(ft) => ft.is_file(),
            Err(err) => {
                return Err(WorkerError::Message(format!(
                    "stage=cleanup.file_type kind=file_type path={} error={err}",
                    path.display()
                )));
            }
        };
        if !is_file {
            continue;
        }

        let matches = matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("log") | Some("json")
        );
        if !matches {
            continue;
        }

        let mut protected = false;
        for p in protected_paths {
            if path.as_path() == *p {
                protected = true;
                break;
            }
        }

        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => String::new(),
        };
        if protected_basenames.contains(&file_name.as_str()) {
            protected = true;
        }

        let meta = match entry.metadata().await {
            Ok(meta) => meta,
            Err(err) => {
                protected = true;
                issues.push(format!(
                    "stage=cleanup.metadata kind=metadata path={} error={err}",
                    path.display()
                ));
                candidates.push((path, None, protected));
                continue;
            }
        };
        let modified = match meta.modified() {
            Ok(modified) => Some(modified),
            Err(err) => {
                protected = true;
                issues.push(format!(
                    "stage=cleanup.modified kind=modified path={} error={err}",
                    path.display()
                ));
                candidates.push((path, None, protected));
                continue;
            }
        };

        if !protected {
            let is_recent = match modified {
                Some(modified) => match now.duration_since(modified) {
                    Ok(age) => age.as_secs() <= cleanup.protect_recent_seconds,
                    Err(err) => {
                        issues.push(format!(
                            "stage=cleanup.age kind=duration_since path={} error={err}",
                            path.display()
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

    let mut to_remove: Vec<std::path::PathBuf> = Vec::new();

    if cleanup.max_files > 0 && (eligible.len() as u64) > cleanup.max_files {
        let remove_count = eligible.len().saturating_sub(cleanup.max_files as usize);
        for (path, _) in eligible.iter().take(remove_count) {
            to_remove.push(path.clone());
        }
    }

    if cleanup.max_age_seconds > 0 {
        for (path, modified) in eligible {
            match now.duration_since(modified) {
                Ok(age) => {
                    if age.as_secs() > cleanup.max_age_seconds {
                        to_remove.push(path);
                    }
                }
                Err(err) => {
                    issues.push(format!(
                        "stage=cleanup.age kind=duration_since path={} error={err}",
                        path.display()
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
                issues.push(format!(
                    "stage=cleanup.remove_file kind=remove_file path={} error={err}",
                    path.display()
                ));
            }
        }
    }

    Ok(CleanupReport::from_issues(issues))
}

#[derive(Clone, Debug)]
struct CleanupReport {
    issue_count: usize,
    first_issue: String,
}

impl CleanupReport {
    fn empty() -> Self {
        Self {
            issue_count: 0,
            first_issue: "<none>".to_string(),
        }
    }

    fn from_issues(issues: Vec<String>) -> Self {
        let issue_count = issues.len();
        let first_issue = match issues.first() {
            Some(first) => first.to_string(),
            None => "<none>".to_string(),
        };
        Self {
            issue_count,
            first_issue,
        }
    }
}

fn emit_postgres_line(
    log: &LogHandle,
    producer: LogProducer,
    transport: LogTransport,
    origin: &str,
    path: &Path,
    line: Vec<u8>,
) -> Result<(), WorkerError> {
    let decoded = decode_line(&line);
    let record = normalize_postgres_line(
        decoded.as_str(),
        PostgresLineRecordBuilder::new(producer, transport, format!("{origin}:{}", path.display())),
    );
    log.emit_raw_record(record).map_err(|err| {
        WorkerError::Message(format!(
            "log sink error while ingesting postgres log: {err}"
        ))
    })?;
    Ok(())
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

fn normalize_postgres_line(line: &str, builder: PostgresLineRecordBuilder) -> RawRecordBuilder {
    if let Ok(value) = serde_json::from_str::<Value>(line) {
        if let Some(parsed) = normalize_postgres_json(value) {
            return builder.build(
                LogParser::PostgresJson,
                parsed.severity,
                parsed.message,
                parsed.fields,
            );
        }
    }

    if let Some(parsed) = normalize_postgres_plain(line) {
        return builder.build(
            LogParser::PostgresPlain,
            parsed.severity,
            parsed.message,
            parsed.fields,
        );
    }

    let mut fields = StructuredFields::new();
    fields.insert("parse_failed", true);
    fields.insert("raw_line", line.to_string());
    builder.build(LogParser::Raw, SeverityText::Info, line.to_string(), fields)
}

struct ParsedLine {
    severity: SeverityText,
    message: String,
    fields: StructuredFields,
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

    let mut fields = StructuredFields::new();
    fields.insert("postgres.json", value.clone());

    Some(ParsedLine {
        severity,
        message,
        fields,
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
    let mut fields = StructuredFields::new();
    fields.insert("postgres.level_raw", level.to_string());

    Some(ParsedLine {
        severity,
        message,
        fields,
    })
}

fn map_pg_severity(raw: &str) -> SeverityText {
    match raw.trim().to_ascii_uppercase().as_str() {
        "DEBUG" | "DEBUG1" | "DEBUG2" | "DEBUG3" | "DEBUG4" | "DEBUG5" => SeverityText::Debug,
        "INFO" | "NOTICE" | "LOG" => SeverityText::Info,
        "WARNING" => SeverityText::Warn,
        "ERROR" => SeverityText::Error,
        "FATAL" | "PANIC" => SeverityText::Fatal,
        _ => SeverityText::Info,
    }
}

pub(crate) fn build_ctx(cfg: RuntimeConfig, log: LogHandle) -> PostgresIngestWorkerCtx {
    PostgresIngestWorkerCtx { cfg, log }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};

    use serde_json::Value;

    use crate::config::{
        DebugConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, PgHbaConfig,
        PostgresLoggingConfig, RuntimeConfig,
    };
    use crate::logging::{
        decode_app_event, LogHandle, LogParser, LogProducer, LogTransport,
        PostgresLineRecordBuilder, SeverityText, TestSink,
    };

    use crate::state::WorkerError;

    use super::{
        cleanup_log_dir, decode_line, emit_ingest_step_failure, emit_postgres_line,
        ingest_error_key_best_effort, map_pg_severity, normalize_postgres_line, IngestErrorKey,
        IngestErrorRateLimiter,
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

    fn sample_runtime_config() -> RuntimeConfig {
        let baseline_logging =
            crate::test_harness::runtime_config::sample_postgres_logging_config();
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_pg_hba(PgHbaConfig {
                source: InlineOrPath::Inline {
                    content: concat!("local all all trust\n", "host all all 127.0.0.1/32 trust\n",)
                        .to_string(),
                },
            })
            .with_logging(LoggingConfig {
                level: LogLevel::Trace,
                postgres: PostgresLoggingConfig {
                    poll_interval_ms: 50,
                    cleanup: LogCleanupConfig {
                        enabled: false,
                        ..baseline_logging.cleanup
                    },
                    ..baseline_logging
                },
                ..crate::test_harness::runtime_config::sample_logging_config()
            })
            .with_debug(DebugConfig { enabled: false })
            .build()
    }

    fn test_log_handle() -> (LogHandle, Arc<TestSink>) {
        let sink = Arc::new(TestSink::default());
        let sink_dyn: Arc<dyn crate::logging::LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
    }

    #[test]
    fn ingest_error_rate_limiter_suppresses_and_reemits_with_count() {
        let mut limiter = IngestErrorRateLimiter::new(30_000);
        let key = IngestErrorKey {
            stage: "a".to_string(),
            kind: "b".to_string(),
            path: "c".to_string(),
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
    fn ingest_error_key_parsing_uses_first_stage_kind_path_tokens() {
        let err = WorkerError::Message(
            "postgres_ingest iteration_errors count=2 stage=first kind=k1 path=/a error=x extra=[stage=second kind=k2 path=/b error=y]"
                .to_string(),
        );
        let key = ingest_error_key_best_effort(&err);
        assert_eq!(key.stage, "first");
        assert_eq!(key.kind, "k1");
        assert_eq!(key.path, "/a");
    }

    #[test]
    fn emit_ingest_step_failure_emits_internal_error_record() -> Result<(), WorkerError> {
        let (log, sink) = test_log_handle();
        let err = WorkerError::Message("stage=x kind=y path=/z error=boom".to_string());

        let emitted = emit_ingest_step_failure(&log, &err, 2, 7);
        assert_eq!(emitted, Ok(()));

        let records = sink.take();
        assert_eq!(records.len(), 1);
        let decoded = decode_app_event(&records[0]).map_err(|err| {
            WorkerError::Message(format!("decode postgres ingest event failed: {err}"))
        })?;
        assert_eq!(decoded.severity, SeverityText::Error);
        assert_eq!(decoded.origin, "postgres_ingest::run");
        assert_eq!(
            decoded.header,
            crate::logging::AppEventHeader::new(
                "postgres_ingest.step_once_failed",
                "postgres_ingest",
                "failed",
            )
        );
        assert_eq!(
            decoded.fields.get("attempts"),
            Some(&Value::Number(serde_json::Number::from(2_u64)))
        );
        assert_eq!(
            decoded.fields.get("suppressed"),
            Some(&Value::Number(serde_json::Number::from(7_u64)))
        );
        Ok(())
    }

    #[test]
    fn map_pg_severity_maps_known_levels() {
        assert_eq!(map_pg_severity("ERROR"), SeverityText::Error);
        assert_eq!(map_pg_severity("warning"), SeverityText::Warn);
        assert_eq!(map_pg_severity("log"), SeverityText::Info);
    }

    #[test]
    fn normalize_postgres_line_parses_jsonlog() {
        let raw = r#"{"error_severity":"LOG","message":"hello from json"}"#;
        let record = normalize_postgres_line(
            raw,
            PostgresLineRecordBuilder::new(LogProducer::Postgres, LogTransport::FileTail, "test"),
        )
        .into_record(1, "host-a".to_string());
        assert_eq!(record.source.parser, LogParser::PostgresJson);
        assert_eq!(record.message, "hello from json");
        assert_eq!(record.severity_text, SeverityText::Info);
        assert_eq!(record.severity_number, SeverityText::Info.number());
        assert_eq!(record.hostname, "host-a");
    }

    #[test]
    fn normalize_postgres_line_parses_plain() {
        let raw = "2026-03-04 01:02:03 UTC [123] ERROR:  something bad";
        let record = normalize_postgres_line(
            raw,
            PostgresLineRecordBuilder::new(LogProducer::Postgres, LogTransport::FileTail, "test"),
        )
        .into_record(1, "host-a".to_string());
        assert_eq!(record.source.parser, LogParser::PostgresPlain);
        assert_eq!(record.severity_text, SeverityText::Error);
        assert_eq!(record.message, "something bad");
    }

    #[test]
    fn normalize_postgres_line_preserves_raw_on_failure() {
        let raw = "not a postgres log line";
        let record = normalize_postgres_line(
            raw,
            PostgresLineRecordBuilder::new(LogProducer::Postgres, LogTransport::FileTail, "test"),
        )
        .into_record(1, "host-a".to_string());
        assert_eq!(record.source.parser, LogParser::Raw);
        assert_eq!(record.message, raw);
        assert_eq!(
            record.attributes.get("parse_failed"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            record.attributes.get("raw_line"),
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
        let record = normalize_postgres_line(
            raw.as_str(),
            PostgresLineRecordBuilder::new(LogProducer::Postgres, LogTransport::FileTail, "test"),
        )
        .into_record(1, "host-a".to_string());
        assert_eq!(record.source.parser, LogParser::Raw);
        assert_eq!(record.message, raw);
        assert_eq!(
            record.attributes.get("parse_failed"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            record.attributes.get("raw_line"),
            Some(&Value::String("non_utf8_bytes_hex=ff006180".to_string()))
        );
    }

    #[test]
    fn emit_postgres_line_emits_parse_failed_record_for_non_utf8() -> Result<(), WorkerError> {
        let (log, sink) = test_log_handle();
        let path = PathBuf::from("/tmp/pg.log");
        let bytes = vec![0xff_u8, 0x00, b'a', 0x80];
        emit_postgres_line(
            &log,
            LogProducer::Postgres,
            LogTransport::FileTail,
            "pg_ctl_log_file",
            path.as_path(),
            bytes,
        )?;
        let records = sink.take();
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].attributes.get("parse_failed"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            records[0].attributes.get("raw_line"),
            Some(&Value::String("non_utf8_bytes_hex=ff006180".to_string()))
        );
        Ok(())
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

        let report = cleanup_log_dir(
            dir.as_path(),
            &LogCleanupConfig {
                enabled: true,
                max_files: 2,
                max_age_seconds: 365 * 24 * 60 * 60,
                protect_recent_seconds: 1,
            },
            &[protected.as_path()],
            SystemTime::now() + Duration::from_secs(3600),
        )
        .await?;
        assert_eq!(report.issue_count, 0);

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

        let report = cleanup_log_dir(
            dir.as_path(),
            &LogCleanupConfig {
                enabled: true,
                max_files: 1,
                max_age_seconds: 365 * 24 * 60 * 60,
                protect_recent_seconds: 1,
            },
            &[],
            SystemTime::now() + Duration::from_secs(3600),
        )
        .await?;
        assert_eq!(report.issue_count, 0);

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

        let report = cleanup_log_dir(
            dir.as_path(),
            &LogCleanupConfig {
                enabled: true,
                max_files: 1,
                max_age_seconds: 1,
                protect_recent_seconds: 1,
            },
            &[],
            SystemTime::now() + Duration::from_secs(3600),
        )
        .await?;
        assert!(report.issue_count > 0);
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
        use std::path::PathBuf;
        use std::time::Duration;

        use tokio::process::Command;
        use tokio::sync::mpsc;
        use tokio::time::Instant;

        use crate::config::{RoleAuthConfig, SecretSource};
        use crate::logging::LogRecord;
        use crate::process::jobs::{
            BaseBackupSpec, BootstrapSpec, DemoteSpec, ShutdownMode, StartPostgresSpec,
        };
        use crate::process::state::{
            ProcessJobKind, ProcessJobRequest, ProcessState, ProcessWorkerCtx,
        };
        use crate::process::worker::{step_once as process_step_once, TokioCommandRunner};
        use crate::state::{new_state_channel, JobId, WorkerError, WorkerStatus};
        use crate::test_harness::binaries::{
            require_pg16_bin_for_real_tests, require_pg16_process_binaries_for_real_tests,
        };
        use crate::test_harness::namespace::NamespaceGuard;
        use crate::test_harness::pg16::{
            prepare_pgdata_dir, spawn_pg16_for_vanilla_postgres, PgInstanceSpec,
        };
        use crate::test_harness::ports::allocate_ports;

        use super::super::{
            step_once as ingest_step_once, PostgresIngestWorkerCtx, PostgresIngestWorkerState,
        };
        use super::{
            sample_runtime_config, test_log_handle, REAL_INGEST_RETRY_SLEEP,
            REAL_PROCESS_WORKER_POLL_INTERVAL, REAL_PSQL_RETRY_SLEEP,
        };

        async fn wait_for_process_idle_success(
            ctx: &mut ProcessWorkerCtx,
            job_id: &JobId,
            timeout: Duration,
        ) -> Result<(), WorkerError> {
            wait_for_process_idle_success_with_debug(ctx, job_id, timeout, None).await
        }

        async fn wait_for_process_idle_success_with_debug(
            ctx: &mut ProcessWorkerCtx,
            job_id: &JobId,
            timeout: Duration,
            debug_log_path: Option<&PathBuf>,
        ) -> Result<(), WorkerError> {
            let started = Instant::now();
            while started.elapsed() < timeout {
                process_step_once(ctx).await?;
                if let ProcessState::Idle {
                    last_outcome: Some(outcome),
                    ..
                } = &ctx.state
                {
                    match outcome {
                        crate::process::state::JobOutcome::Success { id, .. } if id == job_id => {
                            return Ok(());
                        }
                        crate::process::state::JobOutcome::Failure { id, error, .. }
                            if id == job_id =>
                        {
                            let debug_tail = match debug_log_path {
                                Some(path) => tail_file_best_effort(path, 60),
                                None => String::new(),
                            };
                            return Err(WorkerError::Message(format!(
                                "process job {} failed unexpectedly: {error}{}",
                                job_id.0,
                                if debug_tail.is_empty() {
                                    "".to_string()
                                } else {
                                    format!(
                                        "\n--- debug tail {} ---\n{debug_tail}",
                                        path_display(debug_log_path)
                                    )
                                }
                            )));
                        }
                        _ => {}
                    }
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(WorkerError::Message(format!(
                "timed out waiting for job {} success",
                job_id.0
            )))
        }

        fn path_display(path: Option<&PathBuf>) -> String {
            match path {
                Some(path) => path.display().to_string(),
                None => "<none>".to_string(),
            }
        }

        fn tail_file_best_effort(path: &PathBuf, max_lines: usize) -> String {
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

            let mut cfg = sample_runtime_config();
            cfg.logging.postgres.log_dir = Some(log_dir);
            cfg.logging.postgres.cleanup.enabled = false;
            cfg.postgres.log_file = ns.child_dir("runtime/pg_ctl.log");

            let (log_handle, sink) = test_log_handle();
            let ctx = PostgresIngestWorkerCtx {
                cfg,
                log: log_handle,
            };
            let mut state = PostgresIngestWorkerState::new(&ctx.cfg);

            // Prime ingestion offsets and then generate logs.
            ingest_step_once(&ctx, &mut state).await?;

            run_psql_query_with_retry(&psql_bin, port, "SELECT 1;", Duration::from_secs(10))
                .await?;

            let deadline = Instant::now() + Duration::from_secs(3);
            let mut collected = Vec::new();
            while Instant::now() < deadline {
                ingest_step_once(&ctx, &mut state).await?;
                collected.extend(sink.take());
                let saw_json = collected
                    .iter()
                    .any(|r| r.source.parser == crate::logging::LogParser::PostgresJson);
                let saw_stderr = collected
                    .iter()
                    .any(|r| r.source.origin.contains("postgres.stderr.log"));
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
            let _ = std::fs::create_dir_all(&log_dir);
            let jsonlog_path = log_dir.join("postgres.json");
            let _ = std::fs::write(&jsonlog_path, b"");

            let mut cfg = sample_runtime_config();
            cfg.process.binaries = binaries.clone();
            cfg.postgres.data_dir = data_dir.clone();
            cfg.postgres.socket_dir = socket_dir.clone();
            cfg.postgres.listen_port = port;
            cfg.postgres.log_file = log_file.clone();
            cfg.postgres
                .extra_gucs
                .insert("log_destination".to_string(), "jsonlog,stderr".to_string());
            cfg.postgres
                .extra_gucs
                .insert("log_filename".to_string(), "postgres.json".to_string());
            cfg.postgres
                .extra_gucs
                .insert("log_directory".to_string(), log_dir.display().to_string());
            cfg.postgres
                .extra_gucs
                .insert("log_statement".to_string(), "all".to_string());
            cfg.postgres
                .extra_gucs
                .insert("logging_collector".to_string(), "on".to_string());
            cfg.logging.postgres.log_dir = Some(log_dir.clone());
            cfg.logging.postgres.cleanup.enabled = false;

            let (log_handle, sink) = test_log_handle();

            let (publisher, _subscriber) = new_state_channel(ProcessState::Idle {
                worker: WorkerStatus::Starting,
                last_outcome: None,
            });
            let (tx, rx) = mpsc::unbounded_channel();
            let mut process_ctx = ProcessWorkerCtx {
                poll_interval: REAL_PROCESS_WORKER_POLL_INTERVAL,
                config: cfg.process.clone(),
                log: log_handle.clone(),
                capture_subprocess_output: true,
                state: ProcessState::Idle {
                    worker: WorkerStatus::Starting,
                    last_outcome: None,
                },
                publisher,
                inbox: rx,
                inbox_disconnected_logged: false,
                command_runner: Box::new(TokioCommandRunner),
                active_runtime: None,
                last_rejection: None,
                now: Box::new(crate::process::worker::system_now_unix_millis),
            };

            let ingest_ctx = PostgresIngestWorkerCtx {
                cfg,
                log: log_handle,
            };
            let mut ingest_state = PostgresIngestWorkerState::new(&ingest_ctx.cfg);

            let bootstrap_id = JobId("bootstrap".to_string());
            tx.send(ProcessJobRequest {
                id: bootstrap_id.clone(),
                kind: ProcessJobKind::Bootstrap(BootstrapSpec {
                    data_dir: data_dir.clone(),
                    superuser_username: ingest_ctx.cfg.postgres.roles.superuser.username.clone(),
                    timeout_ms: Some(30_000),
                }),
            })
            .map_err(|_| WorkerError::Message("send bootstrap job failed".to_string()))?;

            wait_for_process_idle_success(&mut process_ctx, &bootstrap_id, Duration::from_secs(30))
                .await?;
            let managed = crate::postgres_managed::materialize_managed_postgres_config(
                &ingest_ctx.cfg,
                &crate::postgres_managed_conf::ManagedPostgresStartIntent::primary(),
            )
            .map_err(|err| {
                WorkerError::Message(format!("materialize managed postgres config failed: {err}"))
            })?;

            reservation.release_port(port).map_err(|err| {
                WorkerError::Message(format!("release reserved port failed: {err}"))
            })?;
            let start_id = JobId("start".to_string());
            tx.send(ProcessJobRequest {
                id: start_id.clone(),
                kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                    data_dir: data_dir.clone(),
                    socket_dir: ingest_ctx.cfg.postgres.socket_dir.clone(),
                    port,
                    config_file: managed.postgresql_conf_path,
                    log_file: log_file.clone(),
                    wait_seconds: Some(30),
                    timeout_ms: Some(60_000),
                }),
            })
            .map_err(|_| WorkerError::Message("send start job failed".to_string()))?;

            let started = Instant::now();
            let mut collected_for_debug: Vec<LogRecord> = Vec::new();
            while started.elapsed() < Duration::from_secs(60) {
                process_step_once(&mut process_ctx).await?;
                collected_for_debug.extend(sink.take());

                if let ProcessState::Idle {
                    last_outcome: Some(outcome),
                    ..
                } = &process_ctx.state
                {
                    match outcome {
                        crate::process::state::JobOutcome::Success { id, .. }
                            if id == &start_id =>
                        {
                            break;
                        }
                        crate::process::state::JobOutcome::Failure { id, error, .. }
                            if id == &start_id =>
                        {
                            let pg_ctl_tail = tail_file_best_effort(&log_file, 120);
                            let postgres_json_tail = tail_file_best_effort(&jsonlog_path, 120);
                            let postmaster_pid =
                                tail_file_best_effort(&data_dir.join("postmaster.pid"), 60);

                            let mut pg_tool_lines = Vec::new();
                            for record in &collected_for_debug {
                                if record.source.producer != crate::logging::LogProducer::PgTool {
                                    continue;
                                }
                                let job_kind = record
                                    .attributes
                                    .get("job_kind")
                                    .and_then(|v| v.as_str())
                                    .map_or("<none>", |value| value);
                                let job_id_attr = record
                                    .attributes
                                    .get("job_id")
                                    .and_then(|v| v.as_str())
                                    .map_or("<none>", |value| value);
                                if job_kind != "start_postgres"
                                    && job_id_attr != start_id.0.as_str()
                                {
                                    continue;
                                }
                                pg_tool_lines.push(format!(
                                    "{:?} {}: {}",
                                    record.source.transport, record.source.origin, record.message
                                ));
                            }
                            if pg_tool_lines.len() > 60 {
                                let start = pg_tool_lines.len().saturating_sub(60);
                                pg_tool_lines.drain(0..start);
                            }
                            let pg_tool_debug = if pg_tool_lines.is_empty() {
                                "(no captured pg_tool stdout/stderr lines for start_postgres)"
                                    .to_string()
                            } else {
                                pg_tool_lines.join("\n")
                            };

                            return Err(WorkerError::Message(format!(
                                "process job {} failed unexpectedly: {error}\n--- pg_ctl log tail {} ---\n{}\n--- postgres jsonlog tail {} ---\n{}\n--- postmaster.pid tail {} ---\n{}\n--- captured pg_tool output (start_postgres) ---\n{}",
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

                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            if started.elapsed() >= Duration::from_secs(60) {
                return Err(WorkerError::Message(
                    "timed out waiting for start_postgres job success".to_string(),
                ));
            }

            // Pump ingestion a bit to collect pg_ctl log lines.
            let mut cmd = Command::new(binaries.psql.clone());
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
            let mut collected = Vec::new();
            while Instant::now() < deadline {
                ingest_step_once(&ingest_ctx, &mut ingest_state).await?;
                process_step_once(&mut process_ctx).await?;
                collected.extend(sink.take());
                let saw_pg_ctl_log = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::Postgres
                        && r.source.origin.contains("pg_ctl_log_file")
                });
                let saw_pg_tool = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::PgTool
                        && (r.source.transport == crate::logging::LogTransport::ChildStdout
                            || r.source.transport == crate::logging::LogTransport::ChildStderr)
                });
                let saw_jsonlog = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::Postgres
                        && r.source.parser == crate::logging::LogParser::PostgresJson
                });
                if saw_pg_ctl_log && saw_pg_tool && saw_jsonlog {
                    break;
                }
                tokio::time::sleep(REAL_INGEST_RETRY_SLEEP).await;
            }

            let stop_id = JobId("stop".to_string());
            tx.send(ProcessJobRequest {
                id: stop_id.clone(),
                kind: ProcessJobKind::Demote(DemoteSpec {
                    data_dir,
                    mode: ShutdownMode::Fast,
                    timeout_ms: Some(20_000),
                }),
            })
            .map_err(|_| WorkerError::Message("send stop job failed".to_string()))?;
            wait_for_process_idle_success(&mut process_ctx, &stop_id, Duration::from_secs(30))
                .await?;

            // One more ingestion pass after shutdown to catch any final flushes.
            ingest_step_once(&ingest_ctx, &mut ingest_state).await?;

            let mut all_records = collected;
            all_records.extend(sink.take());

            let saw_pg_ctl_log = all_records.iter().any(|r| {
                r.source.producer == crate::logging::LogProducer::Postgres
                    && r.source.origin.contains("pg_ctl_log_file")
            });
            let saw_pg_tool = all_records.iter().any(|r| {
                r.source.producer == crate::logging::LogProducer::PgTool
                    && r.attributes
                        .get("job_kind")
                        .and_then(|v| v.as_str())
                        .is_some()
            });
            let saw_jsonlog = all_records.iter().any(|r| {
                r.source.producer == crate::logging::LogProducer::Postgres
                    && r.source.parser == crate::logging::LogParser::PostgresJson
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
            let _ = std::fs::create_dir_all(&data_dir);

            let mut cfg = sample_runtime_config();
            cfg.process.binaries = binaries;

            let (log_handle, sink) = test_log_handle();

            let initial = ProcessState::Idle {
                worker: WorkerStatus::Starting,
                last_outcome: None,
            };
            let (publisher, _subscriber) = new_state_channel(initial.clone());
            let (tx, rx) = mpsc::unbounded_channel();
            let mut ctx = ProcessWorkerCtx {
                poll_interval: REAL_PROCESS_WORKER_POLL_INTERVAL,
                config: cfg.process,
                log: log_handle,
                capture_subprocess_output: true,
                state: initial,
                publisher,
                inbox: rx,
                inbox_disconnected_logged: false,
                command_runner: Box::new(TokioCommandRunner),
                active_runtime: None,
                last_rejection: None,
                now: Box::new(crate::process::worker::system_now_unix_millis),
            };

            let job_id = JobId("basebackup-fail".to_string());
            tx.send(ProcessJobRequest {
                id: job_id.clone(),
                kind: ProcessJobKind::BaseBackup(BaseBackupSpec {
                    data_dir,
                    source: crate::process::jobs::ReplicatorSourceConn {
                        conninfo: crate::pginfo::state::PgConnInfo {
                            host: "127.0.0.1".to_string(),
                            port: 9,
                            user: "replicator".to_string(),
                            dbname: "postgres".to_string(),
                            application_name: None,
                            connect_timeout_s: Some(1),
                            ssl_mode: crate::pginfo::state::PgSslMode::Prefer,
                            ssl_root_cert: None,
                            options: None,
                        },
                        auth: RoleAuthConfig::Password {
                            password: SecretSource::Inline {
                                content: "secret-password".to_string(),
                            },
                        },
                    },
                    timeout_ms: Some(5_000),
                }),
            })
            .map_err(|_| WorkerError::Message("send basebackup job failed".to_string()))?;

            let deadline = Instant::now() + Duration::from_secs(10);
            let mut collected = Vec::new();
            while Instant::now() < deadline {
                process_step_once(&mut ctx).await?;
                collected.extend(sink.take());
                let saw_stderr = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::PgTool
                        && r.source.transport == crate::logging::LogTransport::ChildStderr
                        && r.attributes.get("job_kind").and_then(|v| v.as_str())
                            == Some("basebackup")
                });
                if saw_stderr {
                    return Ok(());
                }
                tokio::time::sleep(REAL_INGEST_RETRY_SLEEP).await;
            }

            Err(WorkerError::Message(
                "timed out waiting for captured pg_basebackup stderr".to_string(),
            ))
        }
    }
}
