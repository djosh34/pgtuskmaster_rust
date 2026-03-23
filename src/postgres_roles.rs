use thiserror::Error;
use tokio_postgres::NoTls;

use crate::config_v2::RuntimeConfigV2;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DesiredManagedRoleSet {
    pub(crate) mandatory: [ManagedRoleSpec; 3],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedRoleSpec {
    pub(crate) identity: MandatoryManagedRole,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) grants: Vec<ManagedRoleGrant>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MandatoryManagedRole {
    Superuser,
    Replicator,
    Rewinder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ManagedRoleGrant {
    RewindFunctionExecute,
}

impl DesiredManagedRoleSet {
    fn all_roles(&self) -> impl Iterator<Item = &ManagedRoleSpec> {
        self.mandatory.iter()
    }
}

impl MandatoryManagedRole {
    fn attributes(self) -> &'static str {
        match self {
            Self::Superuser => "LOGIN SUPERUSER NOREPLICATION",
            Self::Replicator => "LOGIN REPLICATION NOSUPERUSER",
            Self::Rewinder => "LOGIN NOSUPERUSER NOREPLICATION",
        }
    }
}

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
    let desired = DesiredManagedRoleSet {
        mandatory: [
            v2_managed_role(
                MandatoryManagedRole::Superuser,
                &cfg.postgres.superuser.username,
                &cfg.postgres.superuser.password,
                Vec::new(),
            ),
            v2_managed_role(
                MandatoryManagedRole::Replicator,
                &cfg.postgres.replicator.username,
                &cfg.postgres.replicator.password,
                Vec::new(),
            ),
            v2_managed_role(
                MandatoryManagedRole::Rewinder,
                &cfg.postgres.rewinder.username,
                &cfg.postgres.rewinder.password,
                vec![ManagedRoleGrant::RewindFunctionExecute],
            ),
        ],
    };
    render_managed_role_reconciliation_sql_for_set(&desired)
}

fn render_managed_role_reconciliation_sql_for_set(
    desired: &DesiredManagedRoleSet,
) -> Result<String, RoleProvisionError> {
    let provision_blocks = desired
        .all_roles()
        .map(render_protected_role_provision_block)
        .collect::<Vec<_>>();
    let grant_blocks = desired
        .all_roles()
        .map(render_role_grant_reconciliation_block)
        .collect::<Vec<_>>();

    Ok(provision_blocks
        .into_iter()
        .chain(grant_blocks)
        .filter(|block| !block.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n"))
}

fn v2_managed_role(
    identity: MandatoryManagedRole,
    username: &str,
    password: &crate::config_v2::types::Secret,
    grants: Vec<ManagedRoleGrant>,
) -> ManagedRoleSpec {
    ManagedRoleSpec {
        identity,
        username: username.to_string(),
        password: password.as_str().to_string(),
        grants,
    }
}

fn render_protected_role_provision_block(spec: &ManagedRoleSpec) -> String {
    let username_literal = sql_literal(spec.username.as_str());
    let attributes = spec.identity.attributes();
    let password_literal = sql_literal(spec.password.as_str());
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

fn render_role_grant_reconciliation_block(spec: &ManagedRoleSpec) -> String {
    spec.grants
        .iter()
        .map(|grant| match grant {
            ManagedRoleGrant::RewindFunctionExecute => {
                render_rewinder_grants_sql(spec.username.as_str())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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
    use crate::{
        config_v2::types::{
            BinariesConfig, DcsConfig, FileSinkMode, LogLevel, LoggingConfig, PostgresConfig,
            RoleConfig, RuntimeConfigV2, Secret, TimingConfig,
        },
        pginfo::conninfo::{PgClientTls, PgSslMode},
        postgres_roles::render_managed_role_reconciliation_sql_v2,
        state::{ClusterName, MemberId, ScopeName},
    };

    fn sample_cfg() -> RuntimeConfigV2 {
        RuntimeConfigV2 {
            cluster_name: ClusterName("cluster-a".to_string()),
            scope: ScopeName("scope-a".to_string()),
            member_id: MemberId("node-a".to_string()),
            postgres: PostgresConfig {
                data_dir: "/tmp/pgtm/data".into(),
                socket_dir: "/tmp/pgtm/socket".into(),
                log_file: "/tmp/pgtm/logs/postgres.log".into(),
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                advertise_port: 5432,
                connect_timeout: std::time::Duration::from_secs(5),
                local_database: "postgres".to_string(),
                source_client_tls: PgClientTls {
                    mode: PgSslMode::Prefer,
                    root_cert: None,
                    client_cert: None,
                    client_key: None,
                },
                superuser: role("postgres", "postgres-secret"),
                replicator: role("replicator", "replicator-secret"),
                rewinder: role("rewinder", "rewinder-secret"),
                pg_hba_file: "/tmp/pgtm/data/pgtm.pg_hba.conf".into(),
                pg_ident_file: "/tmp/pgtm/data/pgtm.pg_ident.conf".into(),
                pg_hba_contents: "local all all trust".to_string(),
                pg_ident_contents: String::new(),
                extra_gucs: std::collections::BTreeMap::new(),
                tls: None,
            },
            dcs: DcsConfig {
                endpoints: Vec::new(),
                auth: None,
                tls: None,
            },
            timing: TimingConfig {
                ha_loop_interval: std::time::Duration::from_secs(1),
                ha_lease_ttl: std::time::Duration::from_secs(10),
                bootstrap_timeout: std::time::Duration::from_secs(30),
                pg_rewind_timeout: std::time::Duration::from_secs(30),
                fencing_timeout: std::time::Duration::from_secs(30),
            },
            binaries: BinariesConfig {
                postgres: "/usr/bin/postgres".into(),
                pg_ctl: "/usr/bin/pg_ctl".into(),
                initdb: "/usr/bin/initdb".into(),
                pg_rewind: "/usr/bin/pg_rewind".into(),
                pg_basebackup: "/usr/bin/pg_basebackup".into(),
                psql: "/usr/bin/psql".into(),
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                stderr_enabled: true,
                file_enabled: false,
                file_path: "/tmp/pgtm/runtime.jsonl".into(),
                file_mode: FileSinkMode::Append,
                postgres_logs_enabled: true,
                postgres_log_dir: "/tmp/pgtm/logs/postgres".into(),
                postgres_pg_ctl_log: "/tmp/pgtm/logs/postgres.log".into(),
                postgres_log_poll_interval: std::time::Duration::from_millis(200),
                postgres_log_cleanup_enabled: true,
                postgres_log_cleanup_max_files: 50,
                postgres_log_cleanup_max_age: std::time::Duration::from_secs(3600),
                postgres_log_cleanup_protect_recent: std::time::Duration::from_secs(300),
            },
            api: crate::config_v2::types::ApiConfig {
                listen_addr: std::net::SocketAddr::from(([127, 0, 0, 1], 8080)),
                transport: crate::config_v2::types::ApiTransport::Http,
                auth: crate::config_v2::types::ApiAuth::Disabled,
            },
        }
    }

    fn role(username: &str, password: &str) -> RoleConfig {
        RoleConfig {
            username: username.to_string(),
            password: Secret::new(password.to_string()),
        }
    }

    #[test]
    fn renders_mandatory_role_sql() -> Result<(), String> {
        let sql = render_managed_role_reconciliation_sql_v2(&sample_cfg())
            .map_err(|err| format!("render sql failed: {err}"))?;

        assert!(sql.contains("ALTER ROLE %I WITH LOGIN SUPERUSER NOREPLICATION PASSWORD %L"));
        assert!(sql.contains("ALTER ROLE %I WITH LOGIN REPLICATION NOSUPERUSER PASSWORD %L"));
        assert!(sql.contains("ALTER ROLE %I WITH LOGIN NOSUPERUSER NOREPLICATION PASSWORD %L"));
        assert!(sql.contains("GRANT EXECUTE ON FUNCTION pg_catalog.pg_ls_dir"));
        Ok(())
    }
}
