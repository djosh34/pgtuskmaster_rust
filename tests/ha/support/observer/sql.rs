use pgtuskmaster_test_support::ha_runner::{RunnerCommand, RunnerResponsePayload};

use crate::support::{
    error::{HarnessError, Result},
    runner::run_contract_command,
};

use crate::support::runner::RunnerSessionHandle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerSqlCommand {
    pub dsn: String,
    pub sql: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunnerSqlContract {
    pub session: RunnerSessionHandle,
}

impl RunnerSqlContract {
    pub fn from_session(session: RunnerSessionHandle) -> Self {
        Self { session }
    }

    pub fn execute(&self, dsn: &str, sql: &str) -> Result<String> {
        self.execute_command(RunnerSqlCommand {
            dsn: dsn.to_string(),
            sql: sql.to_string(),
        })
    }

    pub fn execute_command(&self, command: RunnerSqlCommand) -> Result<String> {
        match run_contract_command(
            &self.session,
            RunnerCommand::ExecuteSql {
                dsn: command.dsn,
                sql: command.sql,
            },
        )? {
            RunnerResponsePayload::SqlRows { rows } => Ok(rows.join("\n")),
            other => Err(HarnessError::message(format!(
                "runner returned unexpected payload `{}` for SQL execution",
                response_kind_label(&other)
            ))),
        }
    }
}

fn response_kind_label(payload: &RunnerResponsePayload) -> &'static str {
    match payload {
        RunnerResponsePayload::Pong => "pong",
        RunnerResponsePayload::State { .. } => "state",
        RunnerResponsePayload::ConnectionView { .. } => "connection_view",
        RunnerResponsePayload::WritablePrimaryTarget { .. } => "writable_primary_target",
        RunnerResponsePayload::Accepted { .. } => "accepted",
        RunnerResponsePayload::SqlRows { .. } => "sql_rows",
        RunnerResponsePayload::Text { .. } => "text",
        RunnerResponsePayload::Error { .. } => "error",
    }
}
