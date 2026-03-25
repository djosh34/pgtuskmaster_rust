use thiserror::Error;
use tokio_postgres::NoTls;

use crate::config_v2::{types::Secret, RuntimeConfigV2};

#[derive(Debug, Error)]
pub(crate) enum RoleProvisionError {
    #[error("connect local postgres for managed role reconciliation failed: {0}")]
    Connect(String),
    #[error("reconcile managed postgres roles failed: {0}")]
    BatchExecute(String),
    #[error("managed role reconciliation connection join failed: {0}")]
    ConnectionJoin(String),
    #[error("managed role reconciliation connection failed: {0}")]
    Connection(String),
}

const SUPERUSER_ATTRIBUTES: &str = "LOGIN SUPERUSER NOREPLICATION";
const REPLICATOR_ATTRIBUTES: &str = "LOGIN REPLICATION NOSUPERUSER";
const REWINDER_ATTRIBUTES: &str = "LOGIN NOSUPERUSER NOREPLICATION";

pub(crate) async fn reconcile_managed_roles_v2(
    cfg: &RuntimeConfigV2,
) -> Result<(), RoleProvisionError> {
    let mut config = tokio_postgres::Config::new();
    config.host_path(cfg.postgres.socket_dir.as_path());
    config.port(cfg.postgres.listen_port);
    config.user(cfg.postgres.superuser.username.as_str());
    config.dbname(cfg.postgres.local_database.as_str());
    config.connect_timeout(cfg.postgres.connect_timeout);
    config.password(cfg.postgres.superuser.password.as_str());

    let (client, connection) = config
        .connect(NoTls)
        .await
        .map_err(|err| RoleProvisionError::Connect(err.to_string()))?;
    let connection_task = tokio::spawn(connection);

    let provision_sql = render_managed_role_reconciliation_sql_v2(cfg)?;
    client
        .batch_execute(provision_sql.as_str())
        .await
        .map_err(|err| RoleProvisionError::BatchExecute(err.to_string()))?;
    drop(client);

    let connection_result = connection_task
        .await
        .map_err(|err| RoleProvisionError::ConnectionJoin(err.to_string()))?;
    connection_result.map_err(|err| RoleProvisionError::Connection(err.to_string()))
}

pub(crate) fn render_managed_role_reconciliation_sql_v2(
    cfg: &RuntimeConfigV2,
) -> Result<String, RoleProvisionError> {
    Ok([
        render_protected_role_provision_block(
            cfg.postgres.superuser.username.as_str(),
            &cfg.postgres.superuser.password,
            SUPERUSER_ATTRIBUTES,
        ),
        render_protected_role_provision_block(
            cfg.postgres.replicator.username.as_str(),
            &cfg.postgres.replicator.password,
            REPLICATOR_ATTRIBUTES,
        ),
        render_protected_role_provision_block(
            cfg.postgres.rewinder.username.as_str(),
            &cfg.postgres.rewinder.password,
            REWINDER_ATTRIBUTES,
        ),
        render_rewinder_grants_sql(cfg.postgres.rewinder.username.as_str()),
    ]
    .into_iter()
    .filter(|block| !block.trim().is_empty())
    .collect::<Vec<_>>()
    .join("\n"))
}

fn render_protected_role_provision_block(
    username: &str,
    password: &Secret,
    attributes: &str,
) -> String {
    let username_literal = sql_literal(username);
    let password_literal = sql_literal(password.as_str());
    format!(
        "DO $$\nBEGIN\n  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = {username_literal}) THEN\n    EXECUTE format('CREATE ROLE %I', {username_literal});\n  END IF;\n  EXECUTE format('ALTER ROLE %I WITH {attributes} PASSWORD %L', {username_literal}, {password_literal});\nEND\n$$;"
    )
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn sql_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn render_rewinder_grants_sql(username: &str) -> String {
    let role = sql_identifier(username);
    [
        "GRANT EXECUTE ON FUNCTION pg_catalog.pg_ls_dir(text, boolean, boolean) TO ",
        role.as_str(),
        ";",
        "\nGRANT EXECUTE ON FUNCTION pg_catalog.pg_stat_file(text, boolean) TO ",
        role.as_str(),
        ";",
        "\nGRANT EXECUTE ON FUNCTION pg_catalog.pg_read_binary_file(text) TO ",
        role.as_str(),
        ";",
        "\nGRANT EXECUTE ON FUNCTION pg_catalog.pg_read_binary_file(text, bigint, bigint, boolean) TO ",
        role.as_str(),
        ";",
    ]
    .concat()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        config_v2::{runtime_test_config_with_data_dir, types::RoleConfig},
        postgres_roles::render_managed_role_reconciliation_sql_v2,
    };

    fn role(username: &str, password: &str) -> RoleConfig {
        RoleConfig {
            username: username.to_string(),
            password: crate::config_v2::types::Secret::new(password.to_string()),
        }
    }

    #[test]
    fn renders_mandatory_role_sql() -> Result<(), String> {
        let cfg = runtime_test_config_with_data_dir(Path::new("/tmp/pgdata"))
            .map(|cfg| crate::config_v2::RuntimeConfigV2 {
                postgres: crate::config_v2::types::PostgresConfig {
                    superuser: role("postgres", "postgres-secret"),
                    replicator: role("replicator", "replicator-secret"),
                    rewinder: role("rewinder", "rewinder-secret"),
                    ..cfg.postgres
                },
                ..cfg
            })
            .map_err(|err| format!("runtime test config failed: {err}"))?;
        let sql = render_managed_role_reconciliation_sql_v2(&cfg)
            .map_err(|err| format!("render sql failed: {err}"))?;

        assert!(sql.contains("ALTER ROLE %I WITH LOGIN SUPERUSER NOREPLICATION PASSWORD %L"));
        assert!(sql.contains("ALTER ROLE %I WITH LOGIN REPLICATION NOSUPERUSER PASSWORD %L"));
        assert!(sql.contains("ALTER ROLE %I WITH LOGIN NOSUPERUSER NOREPLICATION PASSWORD %L"));
        assert!(sql.contains("GRANT EXECUTE ON FUNCTION pg_catalog.pg_ls_dir"));
        Ok(())
    }
}
