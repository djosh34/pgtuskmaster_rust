use std::time::Duration;

use thiserror::Error;
use tokio_postgres::NoTls;

use crate::config::{resolve_secret_string, RoleAuthConfig, RuntimeConfig};

#[derive(Debug, Error)]
pub(crate) enum RoleProvisionError {
    #[error("resolve bootstrap superuser password failed: {0}")]
    ResolveSuperuserPassword(String),
    #[error("connect local postgres for role provisioning failed: {0}")]
    Connect(String),
    #[error("render required role sql failed: {0}")]
    RenderSql(String),
    #[error("provision required postgres roles failed: {0}")]
    BatchExecute(String),
    #[error("role provisioning connection join failed: {0}")]
    ConnectionJoin(String),
    #[error("role provisioning connection failed: {0}")]
    Connection(String),
}

pub(crate) async fn ensure_required_roles(
    cfg: &RuntimeConfig,
    socket_dir: &std::path::Path,
    postgres_port: u16,
) -> Result<(), RoleProvisionError> {
    let mut config = tokio_postgres::Config::new();
    config.host_path(socket_dir);
    config.port(postgres_port);
    config.user(cfg.postgres.roles.superuser.username.as_str());
    config.dbname(cfg.postgres.local_conn_identity.dbname.as_str());
    config.connect_timeout(Duration::from_secs(cfg.postgres.connect_timeout_s.into()));
    if let RoleAuthConfig::Password { password } = &cfg.postgres.roles.superuser.auth {
        let resolved = resolve_secret_string("postgres.roles.superuser.auth.password", password)
            .map_err(|err| RoleProvisionError::ResolveSuperuserPassword(err.to_string()))?;
        config.password(resolved);
    }

    let (client, connection) = config
        .connect(NoTls)
        .await
        .map_err(|err| RoleProvisionError::Connect(err.to_string()))?;
    let connection_task = tokio::spawn(connection);

    let provision_sql = render_required_role_sql(cfg)?;
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

pub(crate) fn render_required_role_sql(cfg: &RuntimeConfig) -> Result<String, RoleProvisionError> {
    let superuser = render_role_provision_block(
        cfg.postgres.roles.superuser.username.as_str(),
        &cfg.postgres.roles.superuser.auth,
        "LOGIN SUPERUSER NOREPLICATION",
    )?;
    let replicator = render_role_provision_block(
        cfg.postgres.roles.replicator.username.as_str(),
        &cfg.postgres.roles.replicator.auth,
        "LOGIN REPLICATION NOSUPERUSER",
    )?;
    let rewinder = render_role_provision_block(
        cfg.postgres.roles.rewinder.username.as_str(),
        &cfg.postgres.roles.rewinder.auth,
        "LOGIN NOREPLICATION NOSUPERUSER",
    )?;
    let rewinder_grants = render_rewinder_grants_sql(cfg.postgres.roles.rewinder.username.as_str());
    Ok(format!(
        "{superuser}\n{replicator}\n{rewinder}\n{rewinder_grants}"
    ))
}

fn render_role_provision_block(
    username: &str,
    auth: &RoleAuthConfig,
    attributes: &str,
) -> Result<String, RoleProvisionError> {
    let username_literal = sql_literal(username);
    let role_statement = match auth {
        RoleAuthConfig::Tls => {
            format!("format('ALTER ROLE %I WITH {attributes}', {username_literal})")
        }
        RoleAuthConfig::Password { password } => {
            let resolved = resolve_secret_string("runtime role provisioning password", password)
                .map_err(|err| RoleProvisionError::RenderSql(err.to_string()))?;
            let password_literal = sql_literal(resolved.as_str());
            format!(
                "format('ALTER ROLE %I WITH {attributes} PASSWORD %L', {username_literal}, {password_literal})"
            )
        }
    };
    Ok(format!(
        "DO $$\nBEGIN\n  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = {username_literal}) THEN\n    EXECUTE format('CREATE ROLE %I', {username_literal});\n  END IF;\n  EXECUTE {role_statement};\nEND\n$$;"
    ))
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
