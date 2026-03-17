use std::path::{Path, PathBuf};

use crate::support::{
    config::{configured_executable, harness_settings},
    error::Result,
    process::{self, CommandSpec},
};

#[derive(Clone, Debug)]
pub struct SqlObserver {
    materialized_dir: PathBuf,
}

impl SqlObserver {
    pub fn new(materialized_dir: PathBuf) -> Self {
        Self { materialized_dir }
    }

    pub fn execute(&self, dsn: &str, sql: &str) -> Result<String> {
        let binary = resolve_psql_binary()?;
        process::run(
            CommandSpec::new(
                binary.clone(),
                format!("executing psql from {}", self.materialized_dir.display()),
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
        )?
        .stdout_text("decoding psql stdout")
    }
}

fn resolve_psql_binary() -> Result<PathBuf> {
    let settings = harness_settings()?;
    let candidate = configured_executable(
        settings.psql.executable_candidates.as_slice(),
        "psql.executable_candidates",
        "psql",
    )?;
    process::ensure_absolute_executable(candidate.as_path())?;
    Ok(Path::new(candidate.as_path()).to_path_buf())
}
