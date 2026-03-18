use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::support::{
    config::harness_settings,
    error::Result,
    process::{self, CommandSpec},
};

pub fn execute(materialized_dir: &Path, dsn: &str, sql: &str, timeout: Duration) -> Result<String> {
    let binary = resolve_psql_binary()?;
    let connect_timeout_s = timeout.as_secs().max(1).to_string();
    let statement_timeout_ms = timeout.as_millis().max(1);
    process::run(
        CommandSpec::new(
            binary.clone(),
            format!("executing psql from {}", materialized_dir.display()),
        )
        .env("PGCONNECT_TIMEOUT", connect_timeout_s)
        .env(
            "PGOPTIONS",
            format!(
                "-c statement_timeout={statement_timeout_ms} -c lock_timeout={statement_timeout_ms}"
            ),
        )
        .env("PATH", "")
        .args([
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
        ]),
    )
}

fn resolve_psql_binary() -> Result<PathBuf> {
    Ok(Path::new(harness_settings()?.psql_executable()).to_path_buf())
}
