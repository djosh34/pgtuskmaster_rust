use std::path::{Path, PathBuf};

use crate::support::{
    error::{HarnessError, Result},
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
            CommandSpec::new(binary.clone(), format!("executing psql from {}", self.materialized_dir.display()))
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
    for candidate in [
        "/usr/lib/postgresql/16/bin/psql",
        "/usr/bin/psql",
        "/usr/local/bin/psql",
    ] {
        let path = Path::new(candidate);
        if path.exists() {
            process::ensure_absolute_executable(path)?;
            return Ok(path.to_path_buf());
        }
    }

    Err(HarnessError::message(
        "psql binary was not found in the expected host locations",
    ))
}
