use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use serde_json::Value;

use crate::config::{LogCleanupConfig, RuntimeConfig};
use crate::logging::{
    LogHandle, LogParser, LogProducer, LogRecord, LogSource, LogTransport, SeverityText,
};
use crate::state::WorkerError;

use super::tailer::{DirTailers, FileTailer, StartPosition};

pub(crate) struct PostgresIngestWorkerCtx {
    pub(crate) cfg: RuntimeConfig,
    pub(crate) log: LogHandle,
}

pub(crate) async fn run(ctx: PostgresIngestWorkerCtx) -> Result<(), WorkerError> {
    if let Ok(Some(path)) = crate::logging::archive_wrapper::ensure_archive_wrapper(&ctx.cfg) {
        let _ = ctx.log.emit(
            SeverityText::Info,
            format!("archive wrapper ready at {}", path.display()),
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "archive_wrapper".to_string(),
            },
        );
    }
    let mut state = PostgresIngestWorkerState::new(&ctx.cfg);
    loop {
        if ctx.cfg.logging.postgres.enabled {
            let _ = step_once(&ctx, &mut state).await;
        }
        tokio::time::sleep(Duration::from_millis(
            ctx.cfg.logging.postgres.poll_interval_ms,
        ))
        .await;
    }
}

struct PostgresIngestWorkerState {
    pg_ctl_log: FileTailer,
    archive_log: Option<FileTailer>,
    dir_tailers: DirTailers,
}

impl PostgresIngestWorkerState {
    fn new(cfg: &RuntimeConfig) -> Self {
        let pg_ctl_log_file = cfg
            .logging
            .postgres
            .pg_ctl_log_file
            .clone()
            .unwrap_or_else(|| cfg.postgres.log_file.clone());
        let archive_log = cfg
            .logging
            .postgres
            .archive_command_log_file
            .clone()
            .map(|path| FileTailer::new(path, StartPosition::Beginning));

        Self {
            pg_ctl_log: FileTailer::new(pg_ctl_log_file, StartPosition::Beginning),
            archive_log,
            dir_tailers: DirTailers::default(),
        }
    }
}

async fn step_once(
    ctx: &PostgresIngestWorkerCtx,
    state: &mut PostgresIngestWorkerState,
) -> Result<(), WorkerError> {
    let max_bytes_per_file = 256 * 1024;

    let pg_lines = state.pg_ctl_log.read_new_lines(max_bytes_per_file).await?;
    for line in pg_lines {
        emit_postgres_line(
            &ctx.log,
            LogProducer::Postgres,
            LogTransport::FileTail,
            "pg_ctl_log_file",
            state.pg_ctl_log.path(),
            line,
        )?;
    }

    if let Some(tailer) = state.archive_log.as_mut() {
        let archive_lines = tailer.read_new_lines(max_bytes_per_file).await?;
        for line in archive_lines {
            emit_postgres_line(
                &ctx.log,
                LogProducer::PostgresArchive,
                LogTransport::FileTail,
                "archive_command_log_file",
                tailer.path(),
                line,
            )?;
        }
    }

    if let Some(dir) = ctx.cfg.logging.postgres.log_dir.as_ref() {
        discover_log_dir(&mut state.dir_tailers, dir).await?;
        for (path, tailer) in state.dir_tailers.iter_mut() {
            let origin = format!("postgres_log_dir:{}", path.file_name().and_then(|s| s.to_str()).unwrap_or("log"));
            let lines = tailer.read_new_lines(max_bytes_per_file).await?;
            for line in lines {
                emit_postgres_line(
                    &ctx.log,
                    LogProducer::Postgres,
                    LogTransport::FileTail,
                    origin.as_str(),
                    tailer.path(),
                    line,
                )?;
            }
        }

        if ctx.cfg.logging.postgres.cleanup.enabled {
            let _ = cleanup_log_dir(dir, &ctx.cfg.logging.postgres.cleanup, state.pg_ctl_log.path())
                .await;
        }
    }

    Ok(())
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
            Err(_) => false,
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
    protected_path: &Path,
) -> Result<(), WorkerError> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(WorkerError::Message(format!(
                "cleanup read_dir failed for {}: {err}",
                dir.display()
            )));
        }
    };

    let mut files = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|err| WorkerError::Message(format!("cleanup readdir entry failed: {err}")))?
    {
        let path = entry.path();
        if path == protected_path {
            continue;
        }
        let is_file = match entry.file_type().await {
            Ok(ft) => ft.is_file(),
            Err(_) => false,
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

        let meta = match entry.metadata().await {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        let modified = meta.modified().ok();
        files.push((path, modified));
    }

    files.sort_by(|a, b| a.1.cmp(&b.1));

    if cleanup.max_files > 0 && (files.len() as u64) > cleanup.max_files {
        let remove_count = files.len().saturating_sub(cleanup.max_files as usize);
        for (path, _) in files.iter().take(remove_count) {
            let _ = tokio::fs::remove_file(path).await;
        }
    }

    if cleanup.max_age_seconds > 0 {
        let now = std::time::SystemTime::now();
        for (path, modified) in files {
            let Some(modified) = modified else { continue };
            let age = now.duration_since(modified).ok();
            if let Some(age) = age {
                if age.as_secs() > cleanup.max_age_seconds {
                    let _ = tokio::fs::remove_file(path).await;
                }
            }
        }
    }

    Ok(())
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
    let source = LogSource {
        producer,
        transport,
        parser: LogParser::Raw,
        origin: format!("{origin}:{}", path.display()),
    };

    let (record, parser_override) = normalize_postgres_line(log, decoded.as_str(), source);
    let mut final_record = record;
    final_record.source.parser = parser_override;
    log.emit_record(&final_record).map_err(|err| {
        WorkerError::Message(format!("log sink error while ingesting postgres log: {err}"))
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

fn normalize_postgres_line(
    log: &LogHandle,
    line: &str,
    mut source: LogSource,
) -> (LogRecord, LogParser) {
    let mut record = LogRecord::new(
        crate::logging::system_now_unix_millis(),
        log.hostname().to_string(),
        SeverityText::Info,
        line.to_string(),
        source.clone(),
    );

    if let Ok(value) = serde_json::from_str::<Value>(line) {
        if let Some(parsed) = normalize_postgres_json(value) {
            source.parser = LogParser::PostgresJson;
            record.source = source;
            record.severity_text = parsed.severity;
            record.severity_number = parsed.severity.number();
            record.message = parsed.message;
            record.attributes = parsed.attributes;
            return (record, LogParser::PostgresJson);
        }
    }

    if let Some(parsed) = normalize_postgres_plain(line) {
        source.parser = LogParser::PostgresPlain;
        record.source = source;
        record.severity_text = parsed.severity;
        record.severity_number = parsed.severity.number();
        record.message = parsed.message;
        record.attributes = parsed.attributes;
        return (record, LogParser::PostgresPlain);
    }

    record
        .attributes
        .insert("parse_failed".to_string(), Value::Bool(true));
    record.attributes.insert(
        "raw_line".to_string(),
        Value::String(line.to_string()),
    );

    (record, LogParser::Raw)
}

struct ParsedLine {
    severity: SeverityText,
    message: String,
    attributes: BTreeMap<String, Value>,
}

fn normalize_postgres_json(value: Value) -> Option<ParsedLine> {
    let obj = value.as_object()?;
    let message = obj
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    if message.trim().is_empty() {
        return None;
    }

    let severity_raw = obj
        .get("error_severity")
        .and_then(|v| v.as_str())
        .or_else(|| obj.get("severity").and_then(|v| v.as_str()))
        .unwrap_or("INFO");
    let severity = map_pg_severity(severity_raw);

    let mut attributes = BTreeMap::new();
    attributes.insert("postgres.json".to_string(), value);

    Some(ParsedLine {
        severity,
        message,
        attributes,
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
    let mut attributes = BTreeMap::new();
    attributes.insert("postgres.level_raw".to_string(), Value::String(level.to_string()));

    Some(ParsedLine {
        severity,
        message,
        attributes,
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
    use std::sync::Arc;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use crate::config::{
        ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths, ClusterConfig,
        DcsConfig, DebugConfig, HaConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig,
        PgHbaConfig, PgIdentConfig, PostgresConnIdentityConfig, PostgresConfig,
        PostgresLoggingConfig, PostgresRoleConfig, PostgresRolesConfig, ProcessConfig,
        RoleAuthConfig, RuntimeConfig, StderrSinkConfig, TlsServerConfig,
    };
    use crate::logging::{LogHandle, LogParser, LogProducer, LogSource, LogTransport, SeverityText, TestSink};
    use crate::state::WorkerError;
    use crate::pginfo::conninfo::PgSslMode;

    use super::{cleanup_log_dir, normalize_postgres_line, map_pg_severity};

    fn sample_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: "/tmp/pgdata".into(),
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: "/tmp/pgtuskmaster/socket".into(),
                log_file: "/tmp/pgtuskmaster/postgres.log".into(),
                rewind_source_host: "127.0.0.1".to_string(),
                rewind_source_port: 5432,
                local_conn_identity: PostgresConnIdentityConfig {
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                rewind_conn_identity: PostgresConnIdentityConfig {
                    user: "rewinder".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                tls: TlsServerConfig {
                    mode: ApiTlsMode::Disabled,
                    identity: None,
                    client_auth: None,
                },
                roles: PostgresRolesConfig {
                    superuser: PostgresRoleConfig {
                        username: "postgres".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    replicator: PostgresRoleConfig {
                        username: "replicator".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    rewinder: PostgresRoleConfig {
                        username: "rewinder".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                },
                pg_hba: PgHbaConfig {
                    source: InlineOrPath::Inline {
                        content: "local all all trust\n".to_string(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: "# empty\n".to_string(),
                    },
                },
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
                init: None,
            },
            ha: HaConfig {
                loop_interval_ms: 1000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 1000,
                bootstrap_timeout_ms: 1000,
                fencing_timeout_ms: 1000,
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Trace,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    archive_command_log_file: None,
                    poll_interval_ms: 50,
                    cleanup: LogCleanupConfig {
                        enabled: false,
                        max_files: 10,
                        max_age_seconds: 60,
                    },
                },
                sinks: crate::config::LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: crate::config::FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: crate::config::FileSinkMode::Append,
                    },
                },
            },
            api: ApiConfig {
                listen_addr: "127.0.0.1:8080".to_string(),
                security: ApiSecurityConfig {
                    tls: TlsServerConfig {
                        mode: ApiTlsMode::Disabled,
                        identity: None,
                        client_auth: None,
                    },
                    auth: ApiAuthConfig::Disabled,
                },
            },
            debug: DebugConfig { enabled: false },
        }
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
    fn map_pg_severity_maps_known_levels() {
        assert_eq!(map_pg_severity("ERROR"), SeverityText::Error);
        assert_eq!(map_pg_severity("warning"), SeverityText::Warn);
        assert_eq!(map_pg_severity("log"), SeverityText::Info);
    }

    #[test]
    fn normalize_postgres_line_parses_jsonlog() {
        let (log, _sink) = test_log_handle();
        let source = LogSource {
            producer: LogProducer::Postgres,
            transport: LogTransport::FileTail,
            parser: LogParser::Raw,
            origin: "test".to_string(),
        };
        let raw = r#"{"error_severity":"LOG","message":"hello from json"}"#;
        let (record, parser) = normalize_postgres_line(&log, raw, source);
        assert_eq!(parser, LogParser::PostgresJson);
        assert_eq!(record.message, "hello from json");
        assert_eq!(record.severity_text, SeverityText::Info);
        assert_eq!(record.severity_number, SeverityText::Info.number());
        assert_eq!(record.hostname, "host-a");
    }

    #[test]
    fn normalize_postgres_line_parses_plain() {
        let (log, _sink) = test_log_handle();
        let source = LogSource {
            producer: LogProducer::Postgres,
            transport: LogTransport::FileTail,
            parser: LogParser::Raw,
            origin: "test".to_string(),
        };
        let raw = "2026-03-04 01:02:03 UTC [123] ERROR:  something bad";
        let (record, parser) = normalize_postgres_line(&log, raw, source);
        assert_eq!(parser, LogParser::PostgresPlain);
        assert_eq!(record.severity_text, SeverityText::Error);
        assert_eq!(record.message, "something bad");
    }

    #[test]
    fn normalize_postgres_line_preserves_raw_on_failure() {
        let (log, _sink) = test_log_handle();
        let source = LogSource {
            producer: LogProducer::Postgres,
            transport: LogTransport::FileTail,
            parser: LogParser::Raw,
            origin: "test".to_string(),
        };
        let raw = "not a postgres log line";
        let (record, parser) = normalize_postgres_line(&log, raw, source);
        assert_eq!(parser, LogParser::Raw);
        assert_eq!(record.message, raw);
        assert_eq!(record.attributes.get("parse_failed"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(
            record.attributes.get("raw_line"),
            Some(&serde_json::Value::String(raw.to_string()))
        );
    }

    fn temp_dir(label: &str) -> PathBuf {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_millis(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!(
            "pgtuskmaster-logging-cleanup-{label}-{millis}-{}",
            std::process::id()
        ))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cleanup_log_dir_enforces_max_files_and_protects_active_file() -> Result<(), WorkerError>
    {
        let dir = temp_dir("max-files");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|err| WorkerError::Message(err.to_string()))?;

        let protected = dir.join("active.log");
        std::fs::write(&protected, b"active\n").map_err(|err| WorkerError::Message(err.to_string()))?;

        for i in 0..5 {
            let path = dir.join(format!("rotated-{i}.log"));
            std::fs::write(&path, b"x\n").map_err(|err| WorkerError::Message(err.to_string()))?;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        cleanup_log_dir(
            dir.as_path(),
            &LogCleanupConfig {
                enabled: true,
                max_files: 2,
                max_age_seconds: 365 * 24 * 60 * 60,
            },
            protected.as_path(),
        )
        .await?;

        assert!(protected.exists());
        let remaining = std::fs::read_dir(&dir)
            .map_err(|err| WorkerError::Message(err.to_string()))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("log"))
            .count();
        // protected + max_files
        assert!(remaining <= 3);

        let _ = std::fs::remove_dir_all(&dir);
        Ok(())
    }

    mod real_binary {
        use std::path::PathBuf;
        use std::time::Duration;

        use tokio::process::Command;
        use tokio::sync::mpsc;
        use tokio::time::Instant;

        use crate::logging::LogRecord;
        use crate::config::RoleAuthConfig;
        use crate::process::jobs::{
            BaseBackupSpec, BootstrapSpec, ShutdownMode, StartPostgresSpec, StopPostgresSpec,
        };
        use crate::process::state::{ProcessJobKind, ProcessJobRequest, ProcessState, ProcessWorkerCtx};
        use crate::process::worker::{step_once as process_step_once, TokioCommandRunner};
        use crate::state::{new_state_channel, JobId, UnixMillis, WorkerError, WorkerStatus};
        use crate::test_harness::binaries::{
            require_pg16_bin_for_real_tests, require_pg16_process_binaries_for_real_tests,
        };
        use crate::test_harness::namespace::NamespaceGuard;
        use crate::test_harness::pg16::{
            prepare_pgdata_dir, spawn_pg16_with_conf_lines, PgInstanceSpec,
        };
        use crate::test_harness::ports::allocate_ports;

        use super::super::{
            step_once as ingest_step_once, PostgresIngestWorkerCtx, PostgresIngestWorkerState,
        };
        use super::{sample_runtime_config, test_log_handle};

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
                                    format!("\n--- debug tail {} ---\n{debug_tail}", path_display(debug_log_path))
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
            let _ = std::fs::create_dir_all(&log_dir);
            let _ = std::fs::write(&jsonlog_path, b"");

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
            reservation
                .release_port(port)
                .map_err(|err| WorkerError::Message(format!("release reserved port failed: {err}")))?;
            let mut pg = spawn_pg16_with_conf_lines(spec, &conf_lines).await?;

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
                    "psql exited unsuccessfully: {status}"
                )));
            }

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
                tokio::time::sleep(Duration::from_millis(20)).await;
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
            let archive_dir = ns.child_dir("archive/pg16");
            let archive_log = ns.child_dir("logs/archive/archive_command.jsonl");
            std::fs::create_dir_all(&socket_dir).map_err(|err| {
                WorkerError::Message(format!("create socket_dir failed: {err}"))
            })?;
            if let Some(parent) = log_file.parent() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    WorkerError::Message(format!("create log file parent failed: {err}"))
                })?;
            }
            let _ = std::fs::create_dir_all(&log_dir);
            let _ = std::fs::create_dir_all(&archive_dir);
            if let Some(parent) = archive_log.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let jsonlog_path = log_dir.join("postgres.json");
            let _ = std::fs::write(&jsonlog_path, b"");

            let mut cfg = sample_runtime_config();
            cfg.process.binaries = binaries.clone();
            cfg.postgres.data_dir = data_dir.clone();
            cfg.postgres.socket_dir = socket_dir.clone();
            cfg.postgres.listen_port = port;
            cfg.postgres.log_file = log_file.clone();
            cfg.logging.postgres.log_dir = Some(log_dir.clone());
            cfg.logging.postgres.archive_command_log_file = Some(archive_log.clone());
            cfg.logging.postgres.cleanup.enabled = false;

            let (log_handle, sink) = test_log_handle();

            let wrapper = crate::logging::archive_wrapper::ensure_archive_wrapper(&cfg)?
                .ok_or_else(|| WorkerError::Message("archive wrapper was not created".to_string()))?;
            let archive_cmd = format!(
                "{} --archive-dir {} \"%p\" \"%f\"",
                wrapper.display(),
                archive_dir.display()
            );

            let (publisher, _subscriber) = new_state_channel(
                ProcessState::Idle {
                    worker: WorkerStatus::Starting,
                    last_outcome: None,
                },
                UnixMillis(0),
            );
            let (tx, rx) = mpsc::unbounded_channel();
            let mut process_ctx = ProcessWorkerCtx {
                poll_interval: Duration::from_millis(5),
                config: cfg.process.clone(),
                log: log_handle.clone(),
                capture_subprocess_output: true,
                state: ProcessState::Idle {
                    worker: WorkerStatus::Starting,
                    last_outcome: None,
                },
                publisher,
                inbox: rx,
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

            // Configure postgres logging + archiving after initdb (before first start).
            let conf_path = data_dir.join("postgresql.conf");
            let mut conf = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&conf_path)
                .map_err(|err| {
                    WorkerError::Message(format!("open postgresql.conf failed: {err}"))
                })?;
            let conf_lines = vec![
                "logging_collector = on".to_string(),
                "log_destination = 'jsonlog,stderr'".to_string(),
                format!("log_directory = '{}'", log_dir.display()),
                "log_filename = 'postgres.json'".to_string(),
                "log_statement = 'all'".to_string(),
                "archive_mode = on".to_string(),
                format!("archive_command = '{archive_cmd}'"),
            ];
            for line in conf_lines {
                if line.trim().is_empty() {
                    continue;
                }
                std::io::Write::write_all(&mut conf, b"\n").map_err(|err| {
                    WorkerError::Message(format!("write postgresql.conf failed: {err}"))
                })?;
                std::io::Write::write_all(&mut conf, line.as_bytes()).map_err(|err| {
                    WorkerError::Message(format!("write postgresql.conf failed: {err}"))
                })?;
                std::io::Write::write_all(&mut conf, b"\n").map_err(|err| {
                    WorkerError::Message(format!("write postgresql.conf failed: {err}"))
                })?;
            }

            reservation
                .release_port(port)
                .map_err(|err| WorkerError::Message(format!("release reserved port failed: {err}")))?;
            let start_id = JobId("start".to_string());
            tx.send(ProcessJobRequest {
                id: start_id.clone(),
                kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                    data_dir: data_dir.clone(),
                    host: "127.0.0.1".to_string(),
                    port,
                    socket_dir: socket_dir.clone(),
                    log_file: log_file.clone(),
                    extra_postgres_settings: std::collections::BTreeMap::new(),
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
                        crate::process::state::JobOutcome::Success { id, .. } if id == &start_id => {
                            break;
                        }
                        crate::process::state::JobOutcome::Failure { id, error, .. }
                            if id == &start_id =>
                        {
                            let pg_ctl_tail = tail_file_best_effort(&log_file, 120);
                            let postgres_json_tail = tail_file_best_effort(&jsonlog_path, 120);
                            let archive_tail = tail_file_best_effort(&archive_log, 120);
                            let postmaster_pid = tail_file_best_effort(&data_dir.join("postmaster.pid"), 60);

                            let mut pg_tool_lines = Vec::new();
                            for record in &collected_for_debug {
                                if record.source.producer != crate::logging::LogProducer::PgTool {
                                    continue;
                                }
                                let job_kind = record
                                    .attributes
                                    .get("job_kind")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("<none>");
                                let job_id_attr = record
                                    .attributes
                                    .get("job_id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("<none>");
                                if job_kind != "start_postgres" && job_id_attr != start_id.0.as_str() {
                                    continue;
                                }
                                pg_tool_lines.push(format!(
                                    "{:?} {}: {}",
                                    record.source.transport,
                                    record.source.origin,
                                    record.message
                                ));
                            }
                            if pg_tool_lines.len() > 60 {
                                let start = pg_tool_lines.len().saturating_sub(60);
                                pg_tool_lines.drain(0..start);
                            }
                            let pg_tool_debug = if pg_tool_lines.is_empty() {
                                "(no captured pg_tool stdout/stderr lines for start_postgres)".to_string()
                            } else {
                                pg_tool_lines.join("\n")
                            };

                            return Err(WorkerError::Message(format!(
                                "process job {} failed unexpectedly: {error}\n--- pg_ctl log tail {} ---\n{}\n--- postgres jsonlog tail {} ---\n{}\n--- archive_command tail {} ---\n{}\n--- postmaster.pid tail {} ---\n{}\n--- captured pg_tool output (start_postgres) ---\n{}",
                                start_id.0,
                                log_file.display(),
                                pg_ctl_tail,
                                jsonlog_path.display(),
                                postgres_json_tail,
                                archive_log.display(),
                                archive_tail,
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
                .arg("SELECT pg_switch_wal();");
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
                let saw_archive = collected.iter().any(|r| {
                    r.source.producer == crate::logging::LogProducer::PostgresArchive
                        && r.source.parser == crate::logging::LogParser::PostgresJson
                        && r.message.contains("archive_command status=")
                });
                if saw_pg_ctl_log && saw_pg_tool && saw_jsonlog && saw_archive {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }

            let stop_id = JobId("stop".to_string());
            tx.send(ProcessJobRequest {
                id: stop_id.clone(),
                kind: ProcessJobKind::StopPostgres(StopPostgresSpec {
                    data_dir,
                    mode: ShutdownMode::Immediate,
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
            let saw_archive = all_records.iter().any(|r| {
                r.source.producer == crate::logging::LogProducer::PostgresArchive
                    && r.source.parser == crate::logging::LogParser::PostgresJson
                    && r.message.contains("archive_command status=")
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
            if !saw_archive {
                return Err(WorkerError::Message(
                    "missing ingested archive_command wrapper records".to_string(),
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
            let (publisher, _subscriber) = new_state_channel(initial.clone(), UnixMillis(0));
            let (tx, rx) = mpsc::unbounded_channel();
            let mut ctx = ProcessWorkerCtx {
                poll_interval: Duration::from_millis(5),
                config: cfg.process,
                log: log_handle,
                capture_subprocess_output: true,
                state: initial,
                publisher,
                inbox: rx,
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
                            options: None,
                        },
                        auth: RoleAuthConfig::Tls,
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
                        && r.attributes
                            .get("job_kind")
                            .and_then(|v| v.as_str())
                            == Some("basebackup")
                });
                if saw_stderr {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }

            Err(WorkerError::Message(
                "timed out waiting for captured pg_basebackup stderr".to_string(),
            ))
        }
    }
}
