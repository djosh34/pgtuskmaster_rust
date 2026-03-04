use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

pub(crate) mod postgres_ingest;
pub(crate) mod tailer;
pub(crate) mod archive_wrapper;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SeverityText {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl SeverityText {
    pub(crate) fn number(self) -> u8 {
        // OpenTelemetry severity_number mapping.
        match self {
            Self::Trace => 1,
            Self::Debug => 5,
            Self::Info => 9,
            Self::Warn => 13,
            Self::Error => 17,
            Self::Fatal => 21,
        }
    }
}

impl From<crate::config::LogLevel> for SeverityText {
    fn from(value: crate::config::LogLevel) -> Self {
        match value {
            crate::config::LogLevel::Trace => Self::Trace,
            crate::config::LogLevel::Debug => Self::Debug,
            crate::config::LogLevel::Info => Self::Info,
            crate::config::LogLevel::Warn => Self::Warn,
            crate::config::LogLevel::Error => Self::Error,
            crate::config::LogLevel::Fatal => Self::Fatal,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogProducer {
    App,
    Postgres,
    PostgresArchive,
    PgTool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogTransport {
    Internal,
    FileTail,
    ChildStdout,
    ChildStderr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogParser {
    App,
    PostgresJson,
    PostgresPlain,
    Raw,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LogSource {
    pub(crate) producer: LogProducer,
    pub(crate) transport: LogTransport,
    pub(crate) parser: LogParser,
    pub(crate) origin: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct LogRecord {
    pub(crate) timestamp_ms: u64,
    pub(crate) hostname: String,
    pub(crate) severity_text: SeverityText,
    pub(crate) severity_number: u8,
    pub(crate) message: String,
    pub(crate) source: LogSource,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) attributes: BTreeMap<String, Value>,
}

impl LogRecord {
    pub(crate) fn new(
        timestamp_ms: u64,
        hostname: String,
        severity_text: SeverityText,
        message: String,
        source: LogSource,
    ) -> Self {
        Self {
            timestamp_ms,
            hostname,
            severity_text,
            severity_number: severity_text.number(),
            message,
            source,
            attributes: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum LogError {
    #[error("json serialize failed: {0}")]
    Json(String),
    #[error("sink write failed: {0}")]
    SinkIo(String),
}

#[derive(Debug, Error)]
pub(crate) enum LogBootstrapError {
    #[error("logging misconfigured: {0}")]
    Misconfigured(String),
    #[error("sink init failed: {0}")]
    SinkInit(String),
}

pub(crate) trait LogSink: Send + Sync {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError>;
}

pub(crate) struct JsonlStderrSink {
    stderr: Mutex<std::io::Stderr>,
}

impl JsonlStderrSink {
    pub(crate) fn new() -> Self {
        Self {
            stderr: Mutex::new(std::io::stderr()),
        }
    }
}

impl LogSink for JsonlStderrSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let line = serde_json::to_string(record).map_err(|err| LogError::Json(err.to_string()))?;
        let mut stderr = self
            .stderr
            .lock()
            .map_err(|_| LogError::SinkIo("stderr lock poisoned".to_string()))?;
        stderr
            .write_all(line.as_bytes())
            .and_then(|()| stderr.write_all(b"\n"))
            .map_err(|err| LogError::SinkIo(err.to_string()))?;
        Ok(())
    }
}

struct NullSink;

impl LogSink for NullSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let _ = record;
        Ok(())
    }
}

pub(crate) struct JsonlFileSink {
    path: PathBuf,
    writer: Mutex<LineWriter<File>>,
}

impl JsonlFileSink {
    pub(crate) fn new(path: PathBuf, mode: crate::config::FileSinkMode) -> Result<Self, LogError> {
        if path.as_os_str().is_empty() {
            return Err(LogError::SinkIo("file sink path is empty".to_string()));
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    LogError::SinkIo(format!(
                        "create log directory {} for {} failed: {err}",
                        parent.display(),
                        path.display()
                    ))
                })?;
            }
        }

        let mut options = OpenOptions::new();
        options.create(true).write(true);
        match mode {
            crate::config::FileSinkMode::Append => {
                options.append(true);
            }
            crate::config::FileSinkMode::Truncate => {
                options.truncate(true);
            }
        }

        let file = options
            .open(&path)
            .map_err(|err| LogError::SinkIo(format!("open log file {} failed: {err}", path.display())))?;

        Ok(Self {
            path,
            writer: Mutex::new(LineWriter::new(file)),
        })
    }
}

impl LogSink for JsonlFileSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let line = serde_json::to_string(record).map_err(|err| LogError::Json(err.to_string()))?;
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| LogError::SinkIo("file sink lock poisoned".to_string()))?;
        writer
            .write_all(line.as_bytes())
            .and_then(|()| writer.write_all(b"\n"))
            .map_err(|err| {
                LogError::SinkIo(format!("write to log file {} failed: {err}", self.path.display()))
            })?;
        Ok(())
    }
}

struct FanoutSink {
    sinks: Vec<(String, Arc<dyn LogSink>)>,
}

static FANOUT_DIAGNOSTIC_ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg(test)]
static FANOUT_DIAGNOSTIC_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

struct AtomicResetGuard<'a> {
    value: &'a AtomicBool,
}

impl Drop for AtomicResetGuard<'_> {
    fn drop(&mut self) {
        self.value.store(false, Ordering::SeqCst);
    }
}

impl FanoutSink {
    fn new(sinks: Vec<(String, Arc<dyn LogSink>)>) -> Self {
        Self { sinks }
    }

    fn write_diagnostic(label: &str, err: &LogError) {
        let acquired = FANOUT_DIAGNOSTIC_ACTIVE
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok();
        if !acquired {
            return;
        }

        let _guard = AtomicResetGuard {
            value: &FANOUT_DIAGNOSTIC_ACTIVE,
        };

        #[cfg(test)]
        {
            FANOUT_DIAGNOSTIC_COUNT.fetch_add(1, Ordering::SeqCst);
        }

        let mut stderr = std::io::stderr();
        let _ = stderr.write_all(b"fanout sink failure: ");
        let _ = stderr.write_all(label.as_bytes());
        let _ = stderr.write_all(b": ");
        let _ = stderr.write_all(err.to_string().as_bytes());
        let _ = stderr.write_all(b"\n");
    }
}

impl LogSink for FanoutSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let mut ok_count: u64 = 0;
        let mut failures: Vec<(String, String)> = Vec::new();

        for (label, sink) in &self.sinks {
            match sink.emit(record) {
                Ok(()) => {
                    ok_count += 1;
                }
                Err(err) => {
                    Self::write_diagnostic(label.as_str(), &err);
                    failures.push((label.clone(), err.to_string()));
                }
            }
        }

        if ok_count > 0 {
            return Ok(());
        }

        let mut message = "all sinks failed".to_string();
        if !failures.is_empty() {
            message.push_str(": ");
            for (idx, (label, err)) in failures.iter().enumerate() {
                if idx > 0 {
                    message.push_str("; ");
                }
                message.push_str(label.as_str());
                message.push_str(" => ");
                message.push_str(err.as_str());
            }
        }
        Err(LogError::SinkIo(message))
    }
}

#[derive(Clone)]
pub(crate) struct LogHandle {
    hostname: String,
    sink: Arc<dyn LogSink>,
    min_app_severity_number: u8,
}

impl LogHandle {
    pub(crate) fn new(hostname: String, sink: Arc<dyn LogSink>, min_app_severity: SeverityText) -> Self {
        Self {
            hostname,
            sink,
            min_app_severity_number: min_app_severity.number(),
        }
    }

    #[cfg(test)]
    pub(crate) fn null() -> Self {
        Self {
            hostname: "unknown".to_string(),
            sink: Arc::new(NullSink),
            min_app_severity_number: SeverityText::Trace.number(),
        }
    }

    pub(crate) fn hostname(&self) -> &str {
        self.hostname.as_str()
    }

    pub(crate) fn emit(
        &self,
        severity_text: SeverityText,
        message: impl Into<String>,
        source: LogSource,
    ) -> Result<(), LogError> {
        if severity_text.number() < self.min_app_severity_number {
            return Ok(());
        }
        let record = LogRecord::new(
            system_now_unix_millis(),
            self.hostname.clone(),
            severity_text,
            message.into(),
            source,
        );
        self.sink.emit(&record)
    }

    pub(crate) fn emit_record(&self, record: &LogRecord) -> Result<(), LogError> {
        self.sink.emit(record)
    }
}

pub(crate) fn system_now_unix_millis() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

pub(crate) fn detect_hostname() -> String {
    match std::env::var("HOSTNAME") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => "unknown".to_string(),
    }
}

pub(crate) struct LoggingSystem {
    pub(crate) handle: LogHandle,
}

pub(crate) fn bootstrap(cfg: &crate::config::RuntimeConfig) -> Result<LoggingSystem, LogBootstrapError> {
    let hostname = detect_hostname();
    let mut sinks: Vec<(String, Arc<dyn LogSink>)> = Vec::new();

    if cfg.logging.sinks.stderr.enabled {
        sinks.push((
            "stderr".to_string(),
            Arc::new(JsonlStderrSink::new()) as Arc<dyn LogSink>,
        ));
    }

    if cfg.logging.sinks.file.enabled {
        let path = cfg
            .logging
            .sinks
            .file
            .path
            .clone()
            .ok_or_else(|| {
                LogBootstrapError::Misconfigured(
                    "logging.sinks.file.enabled=true but logging.sinks.file.path is not set".to_string(),
                )
            })?;

        let label = format!("file:{}", path.display());
        let sink = JsonlFileSink::new(path, cfg.logging.sinks.file.mode)
            .map_err(|err| LogBootstrapError::SinkInit(err.to_string()))?;
        sinks.push((label, Arc::new(sink) as Arc<dyn LogSink>));
    }

    let sink: Arc<dyn LogSink> = match sinks.len() {
        0 => Arc::new(NullSink),
        1 => sinks
            .pop()
            .map(|(_label, sink)| sink)
            .ok_or_else(|| LogBootstrapError::SinkInit("unexpected empty sink list".to_string()))?,
        _ => Arc::new(FanoutSink::new(sinks)),
    };

    Ok(LoggingSystem {
        handle: LogHandle::new(hostname, sink, SeverityText::from(cfg.logging.level)),
    })
}

#[cfg(test)]
#[derive(Clone, Default)]
pub(crate) struct TestSink {
    records: Arc<Mutex<Vec<LogRecord>>>,
}

#[cfg(test)]
impl TestSink {
    pub(crate) fn take(&self) -> Vec<LogRecord> {
        let mut locked = match self.records.lock() {
            Ok(locked) => locked,
            Err(poisoned) => poisoned.into_inner(),
        };
        std::mem::take(&mut *locked)
    }
}

#[cfg(test)]
impl LogSink for TestSink {
    fn emit(&self, record: &LogRecord) -> Result<(), LogError> {
        let mut locked = self
            .records
            .lock()
            .map_err(|_| LogError::SinkIo("test sink lock poisoned".to_string()))?;
        locked.push(record.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::{
        ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths, ClusterConfig,
        DcsConfig, DebugConfig, FileSinkConfig, FileSinkMode, HaConfig, InlineOrPath,
        LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig, PgHbaConfig, PgIdentConfig,
        PostgresConnIdentityConfig, PostgresConfig, PostgresLoggingConfig, PostgresRoleConfig,
        PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig, StderrSinkConfig,
        TlsServerConfig,
    };
    use crate::pginfo::conninfo::PgSslMode;

    fn unique_temp_root(label: &str) -> PathBuf {
        let pid = std::process::id();
        let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_nanos(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!("pgtuskmaster-{label}-{pid}-{nanos}"))
    }

    fn sample_record(message: &str) -> LogRecord {
        LogRecord::new(
            1,
            "host-a".to_string(),
            SeverityText::Info,
            message.to_string(),
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )
    }

    fn read_lines(path: &std::path::Path) -> Result<Vec<String>, std::io::Error> {
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8_lossy(&bytes);
        Ok(text
            .lines()
            .map(|line| line.to_string())
            .filter(|line| !line.trim().is_empty())
            .collect())
    }

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
                backup_timeout_ms: 1000,
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                    pgbackrest: None,
                },
            },
            backup: crate::config::BackupConfig::default(),
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
                sinks: LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: FileSinkMode::Append,
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

    #[test]
    fn jsonl_file_sink_creates_parent_dirs_and_writes_jsonl_line() -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-create");
        let _ = std::fs::remove_dir_all(&root);

        let path = root.join("a").join("b").join("log.jsonl");
        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append)?;
        sink.emit(&sample_record("hello"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "hello");

        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_append_mode_preserves_existing_content() -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-append");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;

        let path = root.join("log.jsonl");
        std::fs::write(&path, b"{\"pre\":1}\n")?;

        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append)?;
        sink.emit(&sample_record("post"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 2);
        let pre: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(pre["pre"], 1);
        let post: serde_json::Value = serde_json::from_str(lines[1].as_str())?;
        assert_eq!(post["message"], "post");

        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_truncate_mode_replaces_existing_content() -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("file-sink-truncate");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;

        let path = root.join("log.jsonl");
        std::fs::write(&path, b"{\"stale\":true}\n{\"stale\":true}\n")?;

        let sink = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Truncate)?;
        sink.emit(&sample_record("fresh"))?;
        drop(sink);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "fresh");

        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn jsonl_file_sink_errors_when_parent_is_file() {
        let root = unique_temp_root("file-sink-parent-file");
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::create_dir_all(&root);

        let not_a_dir = root.join("not_a_dir");
        let _ = std::fs::remove_file(&not_a_dir);
        let write_res = std::fs::write(&not_a_dir, b"im a file");
        assert!(write_res.is_ok());

        let path = not_a_dir.join("app.jsonl");
        let err = JsonlFileSink::new(path.clone(), crate::config::FileSinkMode::Append);
        assert!(matches!(err, Err(LogError::SinkIo(_))));

        if let Err(LogError::SinkIo(msg)) = err {
            assert!(msg.contains(path.display().to_string().as_str()));
        }

        let _ = std::fs::remove_dir_all(&root);
    }

    #[derive(Clone)]
    struct FailSink;

    impl LogSink for FailSink {
        fn emit(&self, _record: &LogRecord) -> Result<(), LogError> {
            Err(LogError::SinkIo("fail sink".to_string()))
        }
    }

    #[test]
    fn fanout_sink_ok_when_any_sink_succeeds_and_emits_diagnostic() {
        FANOUT_DIAGNOSTIC_COUNT.store(0, Ordering::SeqCst);

        let ok = Arc::new(TestSink::default());
        let ok_dyn: Arc<dyn LogSink> = ok;
        let fail_dyn: Arc<dyn LogSink> = Arc::new(FailSink);

        let sink = FanoutSink::new(vec![
            ("ok".to_string(), ok_dyn),
            ("fail".to_string(), fail_dyn),
        ]);

        assert!(sink.emit(&sample_record("x")).is_ok());
        assert!(FANOUT_DIAGNOSTIC_COUNT.load(Ordering::SeqCst) >= 1);
    }

    #[test]
    fn fanout_sink_err_when_all_sinks_fail() {
        let fail_a: Arc<dyn LogSink> = Arc::new(FailSink);
        let fail_b: Arc<dyn LogSink> = Arc::new(FailSink);

        let sink = FanoutSink::new(vec![
            ("a".to_string(), fail_a),
            ("b".to_string(), fail_b),
        ]);

        let err = sink.emit(&sample_record("x"));
        assert!(matches!(err, Err(LogError::SinkIo(_))));
    }

    #[test]
    fn bootstrap_file_enabled_without_path_returns_misconfigured() {
        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = None;

        let res = bootstrap(&cfg);
        assert!(matches!(res, Err(LogBootstrapError::Misconfigured(_))));
    }

    #[test]
    fn bootstrap_file_enabled_with_path_writes_jsonl() -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("bootstrap-file-enabled");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;

        let path = root.join("app.jsonl");

        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(path.clone());

        let system = bootstrap(&cfg)?;
        system.handle.emit(
            SeverityText::Info,
            "hello",
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )?;
        drop(system);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "hello");
        assert_eq!(v["severity_text"], "info");

        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn bootstrap_with_stderr_and_file_still_writes_file() -> Result<(), Box<dyn std::error::Error>> {
        let root = unique_temp_root("bootstrap-stderr-and-file");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root)?;

        let path = root.join("app.jsonl");

        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = true;
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(path.clone());

        let system = bootstrap(&cfg)?;
        system.handle.emit(
            SeverityText::Info,
            "fanout",
            LogSource {
                producer: LogProducer::App,
                transport: LogTransport::Internal,
                parser: LogParser::App,
                origin: "test".to_string(),
            },
        )?;
        drop(system);

        let lines = read_lines(&path)?;
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(lines[0].as_str())?;
        assert_eq!(v["message"], "fanout");

        let _ = std::fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn bootstrap_with_all_sinks_disabled_is_non_fatal() -> Result<(), LogBootstrapError> {
        let mut cfg = sample_runtime_config();
        cfg.logging.sinks.stderr.enabled = false;
        cfg.logging.sinks.file.enabled = false;

        let system = bootstrap(&cfg)?;
        let record = sample_record("dropped");
        let _ = system.handle.emit_record(&record);
        Ok(())
    }
}
