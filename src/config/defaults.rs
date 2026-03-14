use std::{net::SocketAddr, path::PathBuf};

use crate::pginfo::conninfo::PgSslMode;

const DEFAULT_PG_CONNECT_TIMEOUT_S: u32 = 5;
const DEFAULT_POSTGRES_DATABASE: &str = "postgres";
const DEFAULT_POSTGRES_LISTEN_HOST: &str = "127.0.0.1";
const DEFAULT_POSTGRES_LISTEN_PORT: u16 = 5432;
const DEFAULT_PG_SSL_MODE: PgSslMode = PgSslMode::Prefer;

const DEFAULT_HA_LOOP_INTERVAL_MS: u64 = 1_000;
const DEFAULT_HA_LEASE_TTL_MS: u64 = 10_000;

const DEFAULT_PG_REWIND_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_BOOTSTRAP_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_FENCING_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_RUNTIME_WORKING_ROOT: &str = "/tmp/pgtuskmaster";

const DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT: bool = true;
const DEFAULT_LOGGING_POSTGRES_ENABLED: bool = true;
const DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS: u64 = 200;
const DEFAULT_LOGGING_CLEANUP_ENABLED: bool = true;
const DEFAULT_LOGGING_CLEANUP_MAX_FILES: u64 = 50;
const DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;
const DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS: u64 = 300;
const DEFAULT_LOGGING_SINK_STDERR_ENABLED: bool = true;
const DEFAULT_LOGGING_SINK_FILE_ENABLED: bool = false;
const DEFAULT_DEBUG_ENABLED: bool = false;

pub(crate) fn default_postgres_connect_timeout_s() -> u32 {
    DEFAULT_PG_CONNECT_TIMEOUT_S
}

pub(crate) fn default_postgres_database() -> String {
    DEFAULT_POSTGRES_DATABASE.to_string()
}

pub(crate) fn default_postgres_listen_host() -> String {
    DEFAULT_POSTGRES_LISTEN_HOST.to_string()
}

pub(crate) fn default_postgres_listen_port() -> u16 {
    DEFAULT_POSTGRES_LISTEN_PORT
}

pub(crate) fn default_pg_ssl_mode() -> PgSslMode {
    DEFAULT_PG_SSL_MODE
}

pub(crate) fn default_ha_loop_interval_ms() -> u64 {
    DEFAULT_HA_LOOP_INTERVAL_MS
}

pub(crate) fn default_ha_lease_ttl_ms() -> u64 {
    DEFAULT_HA_LEASE_TTL_MS
}

pub(crate) fn default_pg_rewind_timeout_ms() -> u64 {
    DEFAULT_PG_REWIND_TIMEOUT_MS
}

pub(crate) fn default_bootstrap_timeout_ms() -> u64 {
    DEFAULT_BOOTSTRAP_TIMEOUT_MS
}

pub(crate) fn default_fencing_timeout_ms() -> u64 {
    DEFAULT_FENCING_TIMEOUT_MS
}

pub(crate) fn default_runtime_working_root() -> PathBuf {
    PathBuf::from(DEFAULT_RUNTIME_WORKING_ROOT)
}

pub(crate) fn default_api_listen_addr() -> SocketAddr {
    SocketAddr::from((std::net::Ipv4Addr::new(127, 0, 0, 1), 8080))
}

pub(crate) fn default_logging_capture_subprocess_output() -> bool {
    DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT
}

pub(crate) fn default_logging_postgres_enabled() -> bool {
    DEFAULT_LOGGING_POSTGRES_ENABLED
}

pub(crate) fn default_logging_postgres_poll_interval_ms() -> u64 {
    DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS
}

pub(crate) fn default_logging_cleanup_enabled() -> bool {
    DEFAULT_LOGGING_CLEANUP_ENABLED
}

pub(crate) fn default_logging_cleanup_max_files() -> u64 {
    DEFAULT_LOGGING_CLEANUP_MAX_FILES
}

pub(crate) fn default_logging_cleanup_max_age_seconds() -> u64 {
    DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS
}

pub(crate) fn default_logging_cleanup_protect_recent_seconds() -> u64 {
    DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS
}

pub(crate) fn default_logging_sink_stderr_enabled() -> bool {
    DEFAULT_LOGGING_SINK_STDERR_ENABLED
}

pub(crate) fn default_logging_sink_file_enabled() -> bool {
    DEFAULT_LOGGING_SINK_FILE_ENABLED
}

pub(crate) fn default_debug_enabled() -> bool {
    DEFAULT_DEBUG_ENABLED
}
