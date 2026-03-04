use super::schema::{
    DebugConfig, FileSinkConfig, FileSinkMode, LogCleanupConfig, LogLevel, LoggingConfig,
    LoggingSinksConfig, PostgresLoggingConfig, ProcessConfig, StderrSinkConfig,
};

// This module is intentionally restricted to *safe* defaults only.
// It must not synthesize security-sensitive material (users/roles/auth, TLS posture, pg_hba/pg_ident).

const DEFAULT_PG_CONNECT_TIMEOUT_S: u32 = 5;
const DEFAULT_PG_REWIND_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_BOOTSTRAP_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_FENCING_TIMEOUT_MS: u64 = 30_000;

const DEFAULT_API_LISTEN_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_DEBUG_ENABLED: bool = false;

const DEFAULT_LOGGING_LEVEL: LogLevel = LogLevel::Info;
const DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT: bool = true;
const DEFAULT_LOGGING_POSTGRES_ENABLED: bool = true;
const DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS: u64 = 200;
const DEFAULT_LOGGING_CLEANUP_ENABLED: bool = true;
const DEFAULT_LOGGING_CLEANUP_MAX_FILES: u64 = 50;
const DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;
const DEFAULT_LOGGING_SINK_STDERR_ENABLED: bool = true;
const DEFAULT_LOGGING_SINK_FILE_ENABLED: bool = false;
const DEFAULT_LOGGING_SINK_FILE_MODE: FileSinkMode = FileSinkMode::Append;

pub(crate) fn default_postgres_connect_timeout_s() -> u32 {
    DEFAULT_PG_CONNECT_TIMEOUT_S
}

pub(crate) fn default_api_listen_addr() -> String {
    DEFAULT_API_LISTEN_ADDR.to_string()
}

pub(crate) fn default_debug_config() -> DebugConfig {
    DebugConfig {
        enabled: DEFAULT_DEBUG_ENABLED,
    }
}

pub(crate) fn default_logging_config() -> LoggingConfig {
    LoggingConfig {
        level: DEFAULT_LOGGING_LEVEL,
        capture_subprocess_output: DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT,
        postgres: PostgresLoggingConfig {
            enabled: DEFAULT_LOGGING_POSTGRES_ENABLED,
            pg_ctl_log_file: None,
            log_dir: None,
            archive_command_log_file: None,
            poll_interval_ms: DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS,
            cleanup: LogCleanupConfig {
                enabled: DEFAULT_LOGGING_CLEANUP_ENABLED,
                max_files: DEFAULT_LOGGING_CLEANUP_MAX_FILES,
                max_age_seconds: DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS,
            },
        },
        sinks: LoggingSinksConfig {
            stderr: StderrSinkConfig {
                enabled: DEFAULT_LOGGING_SINK_STDERR_ENABLED,
            },
            file: FileSinkConfig {
                enabled: DEFAULT_LOGGING_SINK_FILE_ENABLED,
                path: None,
                mode: DEFAULT_LOGGING_SINK_FILE_MODE,
            },
        },
    }
}

pub(crate) fn normalize_process_config(
    input: super::schema::ProcessConfigV2Input,
) -> ProcessConfig {
    ProcessConfig {
        pg_rewind_timeout_ms: input
            .pg_rewind_timeout_ms
            .unwrap_or(DEFAULT_PG_REWIND_TIMEOUT_MS),
        bootstrap_timeout_ms: input
            .bootstrap_timeout_ms
            .unwrap_or(DEFAULT_BOOTSTRAP_TIMEOUT_MS),
        fencing_timeout_ms: input
            .fencing_timeout_ms
            .unwrap_or(DEFAULT_FENCING_TIMEOUT_MS),
        binaries: input.binaries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_logging_config_is_deterministic() {
        let a = default_logging_config();
        let b = default_logging_config();
        assert_eq!(a, b);
    }
}
