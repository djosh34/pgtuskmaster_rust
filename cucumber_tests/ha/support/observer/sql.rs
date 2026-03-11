use std::path::Path;

use crate::support::{docker::cli::DockerCli, error::Result};

const PSQL_BIN: &str = "/usr/lib/postgresql/16/bin/psql";

#[derive(Clone, Debug)]
pub struct SqlObserver {
    docker: DockerCli,
    observer_container: String,
}

impl SqlObserver {
    pub fn new(docker: DockerCli, observer_container: String) -> Self {
        Self {
            docker,
            observer_container,
        }
    }

    pub fn execute(&self, dsn: &str, sql: &str) -> Result<String> {
        self.docker.exec(
            self.observer_container.as_str(),
            Path::new(PSQL_BIN),
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
            ]
            .as_slice(),
        )
    }
}
