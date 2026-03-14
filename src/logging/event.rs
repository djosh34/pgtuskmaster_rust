use std::{collections::BTreeMap, path::PathBuf};

use serde_json::Value;

use super::{
    raw_record::EncodedRecord, LogError, LogParser, LogProducer, LogRecord, LogSource,
    LogTransport, SeverityText,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LogEvent {
    Runtime(InternalEvent<RuntimeEvent>),
    Dcs(InternalEvent<DcsEvent>),
    PgInfo(InternalEvent<PgInfoEvent>),
    Process(InternalEvent<ProcessEvent>),
    PostgresIngest(InternalEvent<PostgresIngestEvent>),
    PostgresLine(PostgresLineEvent),
    SubprocessLine(SubprocessLineEvent),
}

impl LogEvent {
    pub(crate) fn severity(&self) -> SeverityText {
        match self {
            Self::Runtime(event) => event.severity,
            Self::Dcs(event) => event.severity,
            Self::PgInfo(event) => event.severity,
            Self::Process(event) => event.severity,
            Self::PostgresIngest(event) => event.severity,
            Self::PostgresLine(event) => event.severity(),
            Self::SubprocessLine(event) => event.severity(),
        }
    }

    pub(crate) fn into_record(
        self,
        timestamp_ms: u64,
        hostname: String,
        origin: impl Into<String>,
    ) -> Result<LogRecord, LogError> {
        let encoded = match self {
            Self::Runtime(event) => encode_runtime_event(origin.into(), event),
            Self::Dcs(event) => encode_dcs_event(origin.into(), event),
            Self::PgInfo(event) => encode_pginfo_event(origin.into(), event),
            Self::Process(event) => encode_process_event(origin.into(), event),
            Self::PostgresIngest(event) => encode_postgres_ingest_event(origin.into(), event),
            Self::PostgresLine(event) => encode_postgres_line_event(event),
            Self::SubprocessLine(event) => encode_subprocess_line_event(event),
        };
        Ok(encoded.into_record(timestamp_ms, hostname))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InternalEvent<T> {
    pub(crate) severity: SeverityText,
    pub(crate) event: T,
}

impl<T> InternalEvent<T> {
    pub(crate) fn new(severity: SeverityText, event: T) -> Self {
        Self { severity, event }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeIdentity {
    pub(crate) scope: String,
    pub(crate) member_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RuntimeEvent {
    StartupEntered {
        identity: RuntimeIdentity,
        startup_run_id: String,
        logging_level: crate::config::LogLevel,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsEventIdentity {
    pub(crate) scope: String,
    pub(crate) member_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsEvent {
    WatchRefreshFailed {
        identity: DcsEventIdentity,
        error: String,
    },
    SnapshotReadFailed {
        identity: DcsEventIdentity,
        error: String,
    },
    CoordinationModeTransition {
        identity: DcsEventIdentity,
        previous: Option<crate::dcs::DcsMode>,
        next: crate::dcs::DcsMode,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PgInfoEvent {
    PollFailed {
        member_id: String,
        error: String,
    },
    SqlTransition {
        member_id: String,
        previous: crate::pginfo::state::SqlStatus,
        next: crate::pginfo::state::SqlStatus,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessJobKind {
    Bootstrap,
    BaseBackup,
    PgRewind,
    Promote,
    Demote,
    StartPostgres,
    StartPrimary,
    StartDetachedStandby,
    StartReplica,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessJobIdentity {
    pub(crate) job_id: String,
    pub(crate) kind: ProcessJobKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessExecutionIdentity {
    pub(crate) job: ProcessJobIdentity,
    pub(crate) binary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CapturedStream {
    Stdout,
    Stderr,
}

impl CapturedStream {
    fn severity(&self) -> SeverityText {
        match self {
            Self::Stdout => SeverityText::Info,
            Self::Stderr => SeverityText::Warn,
        }
    }

    fn transport(&self) -> LogTransport {
        match self {
            Self::Stdout => LogTransport::ChildStdout,
            Self::Stderr => LogTransport::ChildStderr,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProcessEvent {
    WorkerRunStarted {
        capture_subprocess_output: bool,
    },
    RequestReceived {
        job: ProcessJobIdentity,
    },
    InboxDisconnected,
    BusyRejected {
        job: ProcessJobIdentity,
    },
    StartPostgresAlreadyRunning {
        job: ProcessJobIdentity,
        data_dir: String,
    },
    StartPostgresPreflightFailed {
        job: ProcessJobIdentity,
        error: String,
    },
    IntentMaterializationFailed {
        job: ProcessJobIdentity,
        error: String,
    },
    BuildCommandFailed {
        job: ProcessJobIdentity,
        error: String,
    },
    SpawnFailed {
        job: ProcessJobIdentity,
        error: String,
    },
    Started {
        execution: ProcessExecutionIdentity,
    },
    OutputDrainFailed {
        execution: ProcessExecutionIdentity,
        error: String,
    },
    Timeout {
        execution: ProcessExecutionIdentity,
    },
    ExitedSuccessfully {
        execution: ProcessExecutionIdentity,
    },
    ExitedUnsuccessfully {
        execution: ProcessExecutionIdentity,
        error: String,
    },
    PollFailed {
        execution: ProcessExecutionIdentity,
        error: String,
    },
    OutputEmitFailed {
        execution: ProcessExecutionIdentity,
        stream: CapturedStream,
        bytes_len: usize,
        error: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PostgresIngestEvent {
    StepOnceFailed {
        attempts: u32,
        suppressed: u64,
        error: String,
    },
    Recovered {
        attempts: u32,
    },
    IterationSummary {
        pg_ctl_lines_emitted: u64,
        log_dir_files_tailed: u64,
        log_dir_lines_emitted: u64,
        dir_tailers: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PostgresLineSource {
    pub(crate) producer: LogProducer,
    pub(crate) transport: LogTransport,
    pub(crate) origin: String,
    pub(crate) path: PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PostgresLineEvent {
    Json {
        source: PostgresLineSource,
        severity: SeverityText,
        message: String,
        payload: Value,
    },
    Plain {
        source: PostgresLineSource,
        severity: SeverityText,
        message: String,
        level_raw: String,
    },
    Unparsed {
        source: PostgresLineSource,
        decoded_line: String,
    },
}

impl PostgresLineEvent {
    fn severity(&self) -> SeverityText {
        match self {
            Self::Json { severity, .. } | Self::Plain { severity, .. } => *severity,
            Self::Unparsed { .. } => SeverityText::Info,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SubprocessLineEvent {
    pub(crate) producer: LogProducer,
    pub(crate) origin: String,
    pub(crate) execution: ProcessExecutionIdentity,
    pub(crate) stream: CapturedStream,
    pub(crate) bytes: Vec<u8>,
}

impl SubprocessLineEvent {
    fn severity(&self) -> SeverityText {
        self.stream.severity()
    }
}

fn encode_runtime_event(origin: String, event: InternalEvent<RuntimeEvent>) -> EncodedRecord {
    match event.event {
        RuntimeEvent::StartupEntered {
            identity,
            startup_run_id,
            logging_level,
        } => {
            let mut attributes = internal_event_attributes(
                "runtime.startup.entered",
                "runtime",
                "ok",
            );
            insert_runtime_identity(&mut attributes, &identity);
            insert_string(&mut attributes, "startup_run_id", startup_run_id);
            insert_string(
                &mut attributes,
                "logging.level",
                log_level_label(logging_level),
            );
            app_record(
                origin,
                event.severity,
                "runtime starting",
                attributes,
            )
        }
    }
}

fn encode_dcs_event(origin: String, event: InternalEvent<DcsEvent>) -> EncodedRecord {
    match event.event {
        DcsEvent::WatchRefreshFailed { identity, error } => {
            let mut attributes = internal_event_attributes(
                "dcs.watch.refresh_failed",
                "dcs",
                "failed",
            );
            insert_dcs_identity(&mut attributes, &identity);
            insert_string(&mut attributes, "error", error);
            app_record(origin, event.severity, "dcs watch refresh failed", attributes)
        }
        DcsEvent::SnapshotReadFailed { identity, error } => {
            let mut attributes = internal_event_attributes(
                "dcs.snapshot.read_failed",
                "dcs",
                "failed",
            );
            insert_dcs_identity(&mut attributes, &identity);
            insert_string(&mut attributes, "error", error);
            app_record(origin, event.severity, "dcs snapshot read failed", attributes)
        }
        DcsEvent::CoordinationModeTransition {
            identity,
            previous,
            next,
        } => {
            let mut attributes =
                internal_event_attributes("dcs.coordination_mode.transition", "dcs", "ok");
            insert_dcs_identity(&mut attributes, &identity);
            insert_string(
                &mut attributes,
                "mode_prev",
                previous
                    .as_ref()
                    .map(dcs_mode_label)
                    .unwrap_or("unknown"),
            );
            insert_string(&mut attributes, "mode_next", dcs_mode_label(&next));
            app_record(
                origin,
                event.severity,
                "dcs coordination mode transition",
                attributes,
            )
        }
    }
}

fn encode_pginfo_event(origin: String, event: InternalEvent<PgInfoEvent>) -> EncodedRecord {
    match event.event {
        PgInfoEvent::PollFailed { member_id, error } => {
            let mut attributes =
                internal_event_attributes("pginfo.poll_failed", "pginfo", "failed");
            insert_string(&mut attributes, "member_id", member_id);
            insert_string(&mut attributes, "error", error);
            app_record(origin, event.severity, "pginfo poll failed", attributes)
        }
        PgInfoEvent::SqlTransition {
            member_id,
            previous,
            next,
        } => {
            let result = match (&previous, &next) {
                (
                    crate::pginfo::state::SqlStatus::Healthy,
                    crate::pginfo::state::SqlStatus::Unreachable,
                ) => "failed",
                (
                    crate::pginfo::state::SqlStatus::Unreachable,
                    crate::pginfo::state::SqlStatus::Healthy,
                ) => "recovered",
                _ => "ok",
            };
            let mut attributes =
                internal_event_attributes("pginfo.sql_transition", "pginfo", result);
            insert_string(&mut attributes, "member_id", member_id);
            insert_string(&mut attributes, "sql_status_prev", sql_status_label(&previous));
            insert_string(&mut attributes, "sql_status_next", sql_status_label(&next));
            app_record(
                origin,
                event.severity,
                "pginfo sql status transition",
                attributes,
            )
        }
    }
}

fn encode_process_event(origin: String, event: InternalEvent<ProcessEvent>) -> EncodedRecord {
    match event.event {
        ProcessEvent::WorkerRunStarted {
            capture_subprocess_output,
        } => {
            let mut attributes = internal_event_attributes(
                "process.worker.run_started",
                "process",
                "ok",
            );
            insert_bool(
                &mut attributes,
                "capture_subprocess_output",
                capture_subprocess_output,
            );
            app_record(
                origin,
                event.severity,
                "process worker run started",
                attributes,
            )
        }
        ProcessEvent::RequestReceived { job } => {
            let mut attributes = internal_event_attributes(
                "process.worker.request_received",
                "process",
                "ok",
            );
            insert_job_identity(&mut attributes, &job);
            app_record(
                origin,
                event.severity,
                "process job request received",
                attributes,
            )
        }
        ProcessEvent::InboxDisconnected => app_record(
            origin,
            event.severity,
            "process worker inbox disconnected",
            internal_event_attributes(
                "process.worker.inbox_disconnected",
                "process",
                "failed",
            ),
        ),
        ProcessEvent::BusyRejected { job } => {
            let mut attributes = internal_event_attributes(
                "process.worker.busy_reject",
                "process",
                "failed",
            );
            insert_job_identity(&mut attributes, &job);
            app_record(
                origin,
                event.severity,
                "process worker busy; rejecting job",
                attributes,
            )
        }
        ProcessEvent::StartPostgresAlreadyRunning { job, data_dir } => {
            let mut attributes = internal_event_attributes(
                "process.job.start_postgres_noop",
                "process",
                "ok",
            );
            insert_job_identity(&mut attributes, &job);
            insert_string(&mut attributes, "data_dir", data_dir);
            app_record(
                origin,
                event.severity,
                "start postgres preflight: postgres already running",
                attributes,
            )
        }
        ProcessEvent::StartPostgresPreflightFailed { job, error } => {
            let mut attributes = internal_event_attributes(
                "process.job.start_postgres_preflight_failed",
                "process",
                "failed",
            );
            insert_job_identity(&mut attributes, &job);
            insert_string(&mut attributes, "error", error);
            app_record(
                origin,
                event.severity,
                "start postgres preflight failed",
                attributes,
            )
        }
        ProcessEvent::IntentMaterializationFailed { job, error } => {
            let mut attributes = internal_event_attributes(
                "process.worker.intent_materialization_failed",
                "process",
                "failed",
            );
            insert_job_identity(&mut attributes, &job);
            insert_string(&mut attributes, "error", error);
            app_record(
                origin,
                event.severity,
                "process intent materialization failed",
                attributes,
            )
        }
        ProcessEvent::BuildCommandFailed { job, error } => {
            let mut attributes = internal_event_attributes(
                "process.job.build_command_failed",
                "process",
                "failed",
            );
            insert_job_identity(&mut attributes, &job);
            insert_string(&mut attributes, "error", error);
            app_record(
                origin,
                event.severity,
                "process build command failed",
                attributes,
            )
        }
        ProcessEvent::SpawnFailed { job, error } => {
            let mut attributes =
                internal_event_attributes("process.job.spawn_failed", "process", "failed");
            insert_job_identity(&mut attributes, &job);
            insert_string(&mut attributes, "error", error);
            app_record(origin, event.severity, "process spawn failed", attributes)
        }
        ProcessEvent::Started { execution } => {
            let mut attributes = internal_event_attributes("process.job.started", "process", "ok");
            insert_execution_identity(&mut attributes, &execution);
            app_record(origin, event.severity, "process job started", attributes)
        }
        ProcessEvent::OutputDrainFailed { execution, error } => {
            let mut attributes = internal_event_attributes(
                "process.worker.output_drain_failed",
                "process",
                "failed",
            );
            insert_execution_identity(&mut attributes, &execution);
            insert_string(&mut attributes, "error", error);
            app_record(
                origin,
                event.severity,
                "process output drain failed",
                attributes,
            )
        }
        ProcessEvent::Timeout { execution } => {
            let mut attributes =
                internal_event_attributes("process.job.timeout", "process", "timeout");
            insert_execution_identity(&mut attributes, &execution);
            app_record(
                origin,
                event.severity,
                "process job timed out; cancelling",
                attributes,
            )
        }
        ProcessEvent::ExitedSuccessfully { execution } => {
            let mut attributes = internal_event_attributes("process.job.exited", "process", "ok");
            insert_execution_identity(&mut attributes, &execution);
            app_record(
                origin,
                event.severity,
                "process job exited successfully",
                attributes,
            )
        }
        ProcessEvent::ExitedUnsuccessfully { execution, error } => {
            let mut attributes =
                internal_event_attributes("process.job.exited", "process", "failed");
            insert_execution_identity(&mut attributes, &execution);
            insert_string(&mut attributes, "error", error);
            app_record(
                origin,
                event.severity,
                "process job exited unsuccessfully",
                attributes,
            )
        }
        ProcessEvent::PollFailed { execution, error } => {
            let mut attributes =
                internal_event_attributes("process.job.poll_failed", "process", "failed");
            insert_execution_identity(&mut attributes, &execution);
            insert_string(&mut attributes, "error", error);
            app_record(origin, event.severity, "process job poll failed", attributes)
        }
        ProcessEvent::OutputEmitFailed {
            execution,
            stream,
            bytes_len,
            error,
        } => {
            let mut attributes = internal_event_attributes(
                "process.worker.output_emit_failed",
                "process",
                "failed",
            );
            insert_execution_identity(&mut attributes, &execution);
            insert_string(&mut attributes, "stream", captured_stream_label(stream));
            insert_usize(&mut attributes, "bytes_len", bytes_len);
            insert_string(&mut attributes, "error", error);
            app_record(
                origin,
                event.severity,
                "process subprocess output emit failed",
                attributes,
            )
        }
    }
}

fn encode_postgres_ingest_event(
    origin: String,
    event: InternalEvent<PostgresIngestEvent>,
) -> EncodedRecord {
    match event.event {
        PostgresIngestEvent::StepOnceFailed {
            attempts,
            suppressed,
            error,
        } => {
            let mut attributes = internal_event_attributes(
                "postgres_ingest.step_once_failed",
                "postgres_ingest",
                "failed",
            );
            insert_u64(&mut attributes, "attempts", u64::from(attempts));
            insert_u64(&mut attributes, "suppressed", suppressed);
            insert_string(&mut attributes, "error", error);
            app_record(
                origin,
                event.severity,
                "postgres ingest step_once failed",
                attributes,
            )
        }
        PostgresIngestEvent::Recovered { attempts } => {
            let mut attributes = internal_event_attributes(
                "postgres_ingest.recovered",
                "postgres_ingest",
                "recovered",
            );
            insert_u64(&mut attributes, "attempts", u64::from(attempts));
            app_record(origin, event.severity, "postgres ingest recovered", attributes)
        }
        PostgresIngestEvent::IterationSummary {
            pg_ctl_lines_emitted,
            log_dir_files_tailed,
            log_dir_lines_emitted,
            dir_tailers,
        } => {
            let mut attributes = internal_event_attributes(
                "postgres_ingest.iteration",
                "postgres_ingest",
                "ok",
            );
            insert_u64(&mut attributes, "pg_ctl_lines_emitted", pg_ctl_lines_emitted);
            insert_u64(&mut attributes, "log_dir_files_tailed", log_dir_files_tailed);
            insert_u64(&mut attributes, "log_dir_lines_emitted", log_dir_lines_emitted);
            insert_usize(&mut attributes, "dir_tailers", dir_tailers);
            app_record(
                origin,
                event.severity,
                "postgres ingest iteration ok",
                attributes,
            )
        }
    }
}

fn encode_postgres_line_event(event: PostgresLineEvent) -> EncodedRecord {
    match event {
        PostgresLineEvent::Json {
            source,
            severity,
            message,
            payload,
        } => {
            let mut attributes = BTreeMap::new();
            insert_string(
                &mut attributes,
                "log_file.path",
                source.path.display().to_string(),
            );
            attributes.insert("postgres.json".to_string(), payload);
            EncodedRecord {
                severity,
                message,
                source: LogSource {
                    producer: source.producer,
                    transport: source.transport,
                    parser: LogParser::PostgresJson,
                    origin: source.origin,
                },
                attributes,
            }
        }
        PostgresLineEvent::Plain {
            source,
            severity,
            message,
            level_raw,
        } => {
            let mut attributes = BTreeMap::new();
            insert_string(
                &mut attributes,
                "log_file.path",
                source.path.display().to_string(),
            );
            insert_string(&mut attributes, "postgres.level_raw", level_raw);
            EncodedRecord {
                severity,
                message,
                source: LogSource {
                    producer: source.producer,
                    transport: source.transport,
                    parser: LogParser::PostgresPlain,
                    origin: source.origin,
                },
                attributes,
            }
        }
        PostgresLineEvent::Unparsed {
            source,
            decoded_line,
        } => {
            let mut attributes = BTreeMap::new();
            insert_string(
                &mut attributes,
                "log_file.path",
                source.path.display().to_string(),
            );
            insert_bool(&mut attributes, "parse_failed", true);
            insert_string(&mut attributes, "raw_line", decoded_line.clone());
            EncodedRecord {
                severity: SeverityText::Info,
                message: decoded_line,
                source: LogSource {
                    producer: source.producer,
                    transport: source.transport,
                    parser: LogParser::Raw,
                    origin: source.origin,
                },
                attributes,
            }
        }
    }
}

fn encode_subprocess_line_event(event: SubprocessLineEvent) -> EncodedRecord {
    let severity = event.severity();
    let (message, raw_bytes_hex) = decode_subprocess_bytes(event.bytes);
    let mut attributes = BTreeMap::new();
    insert_execution_identity(&mut attributes, &event.execution);
    insert_string(&mut attributes, "stream", captured_stream_label(event.stream));
    if let Some(raw_bytes_hex) = raw_bytes_hex {
        insert_string(&mut attributes, "raw_bytes_hex", raw_bytes_hex);
    }
    EncodedRecord {
        severity,
        message,
        source: LogSource {
            producer: event.producer,
            transport: event.stream.transport(),
            parser: LogParser::Raw,
            origin: event.origin,
        },
        attributes,
    }
}

fn app_record(
    origin: String,
    severity: SeverityText,
    message: impl Into<String>,
    attributes: BTreeMap<String, Value>,
) -> EncodedRecord {
    EncodedRecord {
        severity,
        message: message.into(),
        source: LogSource {
            producer: LogProducer::App,
            transport: LogTransport::Internal,
            parser: LogParser::App,
            origin,
        },
        attributes,
    }
}

fn internal_event_attributes(name: &str, domain: &str, result: &str) -> BTreeMap<String, Value> {
    let mut attributes = BTreeMap::new();
    insert_string(&mut attributes, "event.name", name);
    insert_string(&mut attributes, "event.domain", domain);
    insert_string(&mut attributes, "event.result", result);
    attributes
}

fn insert_runtime_identity(attributes: &mut BTreeMap<String, Value>, identity: &RuntimeIdentity) {
    insert_string(attributes, "scope", identity.scope.as_str());
    insert_string(attributes, "member_id", identity.member_id.as_str());
}

fn insert_dcs_identity(attributes: &mut BTreeMap<String, Value>, identity: &DcsEventIdentity) {
    insert_string(attributes, "scope", identity.scope.as_str());
    insert_string(attributes, "member_id", identity.member_id.as_str());
}

fn insert_job_identity(attributes: &mut BTreeMap<String, Value>, identity: &ProcessJobIdentity) {
    insert_string(attributes, "job_id", identity.job_id.as_str());
    insert_string(attributes, "job_kind", process_job_kind_label(identity.kind));
}

fn insert_execution_identity(
    attributes: &mut BTreeMap<String, Value>,
    identity: &ProcessExecutionIdentity,
) {
    insert_job_identity(attributes, &identity.job);
    insert_string(attributes, "binary", identity.binary.as_str());
}

fn insert_string(
    attributes: &mut BTreeMap<String, Value>,
    key: &str,
    value: impl Into<String>,
) {
    attributes.insert(key.to_string(), Value::String(value.into()));
}

fn insert_bool(attributes: &mut BTreeMap<String, Value>, key: &str, value: bool) {
    attributes.insert(key.to_string(), Value::Bool(value));
}

fn insert_u64(attributes: &mut BTreeMap<String, Value>, key: &str, value: u64) {
    attributes.insert(key.to_string(), Value::Number(value.into()));
}

fn insert_usize(attributes: &mut BTreeMap<String, Value>, key: &str, value: usize) {
    match u64::try_from(value) {
        Ok(value) => insert_u64(attributes, key, value),
        Err(_) => insert_string(attributes, key, value.to_string()),
    }
}

fn log_level_label(level: crate::config::LogLevel) -> &'static str {
    match level {
        crate::config::LogLevel::Trace => "trace",
        crate::config::LogLevel::Debug => "debug",
        crate::config::LogLevel::Info => "info",
        crate::config::LogLevel::Warn => "warn",
        crate::config::LogLevel::Error => "error",
        crate::config::LogLevel::Fatal => "fatal",
    }
}

fn dcs_mode_label(mode: &crate::dcs::DcsMode) -> &'static str {
    match mode {
        crate::dcs::DcsMode::Coordinated => "coordinated",
        crate::dcs::DcsMode::Degraded => "degraded",
        crate::dcs::DcsMode::NotTrusted => "not_trusted",
    }
}

fn sql_status_label(status: &crate::pginfo::state::SqlStatus) -> &'static str {
    match status {
        crate::pginfo::state::SqlStatus::Unknown => "unknown",
        crate::pginfo::state::SqlStatus::Healthy => "healthy",
        crate::pginfo::state::SqlStatus::Unreachable => "unreachable",
    }
}

fn process_job_kind_label(kind: ProcessJobKind) -> &'static str {
    match kind {
        ProcessJobKind::Bootstrap => "bootstrap",
        ProcessJobKind::BaseBackup => "basebackup",
        ProcessJobKind::PgRewind => "pg_rewind",
        ProcessJobKind::Promote => "promote",
        ProcessJobKind::Demote => "demote",
        ProcessJobKind::StartPostgres => "start_postgres",
        ProcessJobKind::StartPrimary => "start_primary",
        ProcessJobKind::StartDetachedStandby => "start_detached_standby",
        ProcessJobKind::StartReplica => "start_replica",
    }
}

fn captured_stream_label(stream: CapturedStream) -> &'static str {
    match stream {
        CapturedStream::Stdout => "stdout",
        CapturedStream::Stderr => "stderr",
    }
}

fn decode_subprocess_bytes(bytes: Vec<u8>) -> (String, Option<String>) {
    match String::from_utf8(bytes) {
        Ok(message) => (message, None),
        Err(err) => {
            let raw_bytes = err.into_bytes();
            let raw_bytes_hex = hex_encode(raw_bytes.as_slice());
            (
                format!("non_utf8_bytes_hex={raw_bytes_hex}"),
                Some(raw_bytes_hex),
            )
        }
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        output.push(TABLE[(byte >> 4) as usize] as char);
        output.push(TABLE[(byte & 0x0f) as usize] as char);
    }
    output
}
