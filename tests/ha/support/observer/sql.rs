use std::{path::Path, time::Duration};

use crate::support::{config::harness_settings, error::Result, process};

pub fn execute(materialized_dir: &Path, dsn: &str, sql: &str, timeout: Duration) -> Result<String> {
    let connect_timeout_s = timeout.as_secs().max(1).to_string();
    let statement_timeout_ms = timeout.as_millis().max(1);
    let env = vec![
        ("PGCONNECT_TIMEOUT".to_string(), connect_timeout_s),
        (
            "PGOPTIONS".to_string(),
            format!(
                "-c statement_timeout={statement_timeout_ms} -c lock_timeout={statement_timeout_ms}"
            ),
        ),
        ("PATH".to_string(), String::new()),
    ];
    process::run(
        harness_settings()?.psql_executable(),
        None,
        format!("executing psql from {}", materialized_dir.display()),
        [
            "--no-psqlrc",
            "--quiet",
            "--tuples-only",
            "--no-align",
            "--set",
            "ON_ERROR_STOP=1",
            "--dbname",
            dsn,
            "--command",
            sql,
        ],
        env,
    )
}
