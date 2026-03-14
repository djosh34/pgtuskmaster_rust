use std::{collections::BTreeMap, time::Duration};

use thiserror::Error;
use tokio_postgres::NoTls;

use crate::config::{
    resolve_secret_string, ManagedPostgresRoleKey, PostgresRoleName, PostgresRolePrivilege,
    RoleAuthConfig, RuntimeConfig,
};

const MANAGED_EXTRA_ROLE_COMMENT_PREFIX: &str = "pgtuskmaster:managed-extra:";

#[derive(Debug, Error)]
pub(crate) enum RoleProvisionError {
    #[error("resolve bootstrap superuser password failed: {0}")]
    ResolveSuperuserPassword(String),
    #[error("connect local postgres for managed role reconciliation failed: {0}")]
    Connect(String),
    #[error("render managed role reconciliation sql failed: {0}")]
    RenderSql(String),
    #[error("reconcile managed postgres roles failed: {0}")]
    BatchExecute(String),
    #[error("managed role reconciliation connection join failed: {0}")]
    ConnectionJoin(String),
    #[error("managed role reconciliation connection failed: {0}")]
    Connection(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DesiredManagedRoleSet {
    pub(crate) mandatory: MandatoryManagedRoleSet,
    pub(crate) extra: BTreeMap<ManagedPostgresRoleKey, ExtraManagedRoleSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MandatoryManagedRoleSet {
    pub(crate) superuser: ManagedRoleSpec,
    pub(crate) replicator: ManagedRoleSpec,
    pub(crate) rewinder: ManagedRoleSpec,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExtraManagedRoleSpec {
    pub(crate) logical_key: ManagedPostgresRoleKey,
    pub(crate) spec: ManagedRoleSpec,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedRoleSpec {
    pub(crate) identity: ManagedRoleIdentity,
    pub(crate) username: PostgresRoleName,
    pub(crate) privilege: PostgresRolePrivilege,
    pub(crate) auth: RoleAuthConfig,
    pub(crate) grants: Vec<ManagedRoleGrant>,
    pub(crate) drop_policy: ManagedRoleDropPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ManagedRoleIdentity {
    Mandatory(MandatoryManagedRole),
    Extra(ManagedPostgresRoleKey),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MandatoryManagedRole {
    Superuser,
    Replicator,
    Rewinder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ManagedRoleGrant {
    Membership(PostgresRoleName),
    RewindFunctionExecute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManagedRoleDropPolicy {
    Protected,
    DropWhenAbsent,
}

impl DesiredManagedRoleSet {
    fn from_config(cfg: &RuntimeConfig) -> Self {
        let mandatory = MandatoryManagedRoleSet {
            superuser: ManagedRoleSpec {
                identity: ManagedRoleIdentity::Mandatory(MandatoryManagedRole::Superuser),
                username: cfg.postgres.roles.mandatory.superuser.username.clone(),
                privilege: PostgresRolePrivilege::Superuser,
                auth: cfg.postgres.roles.mandatory.superuser.auth.clone(),
                grants: Vec::new(),
                drop_policy: ManagedRoleDropPolicy::Protected,
            },
            replicator: ManagedRoleSpec {
                identity: ManagedRoleIdentity::Mandatory(MandatoryManagedRole::Replicator),
                username: cfg.postgres.roles.mandatory.replicator.username.clone(),
                privilege: PostgresRolePrivilege::Replication,
                auth: cfg.postgres.roles.mandatory.replicator.auth.clone(),
                grants: Vec::new(),
                drop_policy: ManagedRoleDropPolicy::Protected,
            },
            rewinder: ManagedRoleSpec {
                identity: ManagedRoleIdentity::Mandatory(MandatoryManagedRole::Rewinder),
                username: cfg.postgres.roles.mandatory.rewinder.username.clone(),
                privilege: PostgresRolePrivilege::Login,
                auth: cfg.postgres.roles.mandatory.rewinder.auth.clone(),
                grants: vec![ManagedRoleGrant::RewindFunctionExecute],
                drop_policy: ManagedRoleDropPolicy::Protected,
            },
        };
        let extra = cfg
            .postgres
            .roles
            .extra
            .iter()
            .map(|(logical_key, role)| {
                (
                    logical_key.clone(),
                    ExtraManagedRoleSpec {
                        logical_key: logical_key.clone(),
                        spec: ManagedRoleSpec {
                            identity: ManagedRoleIdentity::Extra(logical_key.clone()),
                            username: role.role.username.clone(),
                            privilege: role.privilege,
                            auth: role.role.auth.clone(),
                            grants: role
                                .member_of
                                .iter()
                                .cloned()
                                .map(ManagedRoleGrant::Membership)
                                .collect(),
                            drop_policy: ManagedRoleDropPolicy::DropWhenAbsent,
                        },
                    },
                )
            })
            .collect();
        Self { mandatory, extra }
    }

    fn all_roles(&self) -> impl Iterator<Item = &ManagedRoleSpec> {
        self.mandatory
            .iter()
            .chain(self.extra.values().map(|extra| &extra.spec))
    }

    fn removable_extra_roles(&self) -> impl Iterator<Item = &ExtraManagedRoleSpec> {
        self.extra
            .values()
            .filter(|extra| extra.spec.drop_policy == ManagedRoleDropPolicy::DropWhenAbsent)
    }
}

impl MandatoryManagedRoleSet {
    fn iter(&self) -> impl Iterator<Item = &ManagedRoleSpec> {
        [&self.superuser, &self.replicator, &self.rewinder].into_iter()
    }
}

impl MandatoryManagedRole {
    fn config_key(self) -> &'static str {
        match self {
            Self::Superuser => "superuser",
            Self::Replicator => "replicator",
            Self::Rewinder => "rewinder",
        }
    }
}

pub(crate) async fn reconcile_managed_roles(
    cfg: &RuntimeConfig,
    socket_dir: &std::path::Path,
    postgres_port: u16,
) -> Result<(), RoleProvisionError> {
    let mut config = tokio_postgres::Config::new();
    config.host_path(socket_dir);
    config.port(postgres_port);
    config.user(cfg.postgres.roles.mandatory.superuser.username.as_str());
    config.dbname(cfg.postgres.local_database.as_str());
    config.connect_timeout(Duration::from_secs(cfg.postgres.connect_timeout_s.into()));
    let RoleAuthConfig::Password { password } = &cfg.postgres.roles.mandatory.superuser.auth;
    let resolved = resolve_secret_string("postgres.roles.mandatory.superuser.auth.password", password)
        .map_err(|err| RoleProvisionError::ResolveSuperuserPassword(err.to_string()))?;
    config.password(resolved);

    let (client, connection) = config
        .connect(NoTls)
        .await
        .map_err(|err| RoleProvisionError::Connect(err.to_string()))?;
    let connection_task = tokio::spawn(connection);

    let provision_sql = render_managed_role_reconciliation_sql(cfg)?;
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

pub(crate) fn render_managed_role_reconciliation_sql(
    cfg: &RuntimeConfig,
) -> Result<String, RoleProvisionError> {
    let desired = DesiredManagedRoleSet::from_config(cfg);
    render_managed_role_reconciliation_sql_for_set(&desired)
}

fn render_managed_role_reconciliation_sql_for_set(
    desired: &DesiredManagedRoleSet,
) -> Result<String, RoleProvisionError> {
    let provision_blocks = desired
        .all_roles()
        .map(render_role_provision_block)
        .collect::<Result<Vec<_>, _>>()?;
    let grant_blocks = desired
        .all_roles()
        .map(render_role_grant_reconciliation_block)
        .collect::<Vec<_>>();
    let stale_extra_drop_block = render_drop_stale_extra_roles_block(desired);

    Ok(provision_blocks
        .into_iter()
        .chain(grant_blocks)
        .chain(std::iter::once(stale_extra_drop_block))
        .filter(|block| !block.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n"))
}

fn render_role_provision_block(spec: &ManagedRoleSpec) -> Result<String, RoleProvisionError> {
    match &spec.identity {
        ManagedRoleIdentity::Mandatory(_) => render_protected_role_provision_block(spec),
        ManagedRoleIdentity::Extra(logical_key) => {
            render_extra_role_provision_block(logical_key, spec)
        }
    }
}

fn render_protected_role_provision_block(
    spec: &ManagedRoleSpec,
) -> Result<String, RoleProvisionError> {
    let username_literal = sql_literal(spec.username.as_str());
    let attributes = render_role_attributes(spec.privilege);
    let password_literal = resolve_role_password_literal(spec)?;
    Ok(format!(
        "DO $$\nBEGIN\n  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = {username_literal}) THEN\n    EXECUTE format('CREATE ROLE %I', {username_literal});\n  END IF;\n  EXECUTE format('ALTER ROLE %I WITH {attributes} PASSWORD %L', {username_literal}, {password_literal});\nEND\n$$;"
    ))
}

fn render_extra_role_provision_block(
    logical_key: &ManagedPostgresRoleKey,
    spec: &ManagedRoleSpec,
) -> Result<String, RoleProvisionError> {
    let logical_key_literal = sql_literal(logical_key.as_str());
    let username_literal = sql_literal(spec.username.as_str());
    let marker_literal = sql_literal(managed_extra_role_marker(logical_key).as_str());
    let attributes = render_role_attributes(spec.privilege);
    let password_literal = resolve_role_password_literal(spec)?;
    Ok(format!(
        "DO $$\nDECLARE\n  tracked_role text;\nBEGIN\n  SELECT rolname\n    INTO tracked_role\n    FROM pg_roles\n   WHERE shobj_description(oid, 'pg_authid') = {marker_literal};\n\n  IF tracked_role IS NULL THEN\n    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = {username_literal}) THEN\n      tracked_role := {username_literal};\n    ELSE\n      EXECUTE format('CREATE ROLE %I', {username_literal});\n      tracked_role := {username_literal};\n    END IF;\n  ELSIF tracked_role <> {username_literal} THEN\n    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = {username_literal}) THEN\n      RAISE EXCEPTION 'managed extra role key % cannot rename % to existing role %', {logical_key_literal}, tracked_role, {username_literal};\n    END IF;\n    EXECUTE format('ALTER ROLE %I RENAME TO %I', tracked_role, {username_literal});\n    tracked_role := {username_literal};\n  END IF;\n\n  EXECUTE format('ALTER ROLE %I WITH {attributes} PASSWORD %L', tracked_role, {password_literal});\n  EXECUTE format('COMMENT ON ROLE %I IS %L', tracked_role, {marker_literal});\nEND\n$$;"
    ))
}

fn sql_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn sql_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn render_role_grant_reconciliation_block(spec: &ManagedRoleSpec) -> String {
    let blocks = spec
        .grants
        .iter()
        .filter_map(|grant| match grant {
            ManagedRoleGrant::Membership(_) => None,
            ManagedRoleGrant::RewindFunctionExecute => {
                Some(render_rewinder_grants_sql(spec.username.as_str()))
            }
        })
        .chain(
            matches!(spec.identity, ManagedRoleIdentity::Extra(_))
                .then(|| render_membership_reconciliation_block(spec)),
        )
        .collect::<Vec<_>>();

    blocks.join("\n")
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

fn render_membership_reconciliation_block(spec: &ManagedRoleSpec) -> String {
    let username_literal = sql_literal(spec.username.as_str());
    let desired_memberships = render_text_array_literal(
        spec.grants.iter().filter_map(|grant| match grant {
            ManagedRoleGrant::Membership(role_name) => Some(role_name.as_str()),
            ManagedRoleGrant::RewindFunctionExecute => None,
        }),
    );
    format!(
        "DO $$\nDECLARE\n  desired_memberships text[] := {desired_memberships};\n  existing_parent text;\nBEGIN\n  FOREACH existing_parent IN ARRAY desired_memberships\n  LOOP\n    EXECUTE format('GRANT %I TO %I', existing_parent, {username_literal});\n  END LOOP;\n\n  FOR existing_parent IN\n    SELECT parent.rolname\n      FROM pg_roles AS parent\n      JOIN pg_auth_members AS membership ON membership.roleid = parent.oid\n      JOIN pg_roles AS child ON child.oid = membership.member\n     WHERE child.rolname = {username_literal}\n  LOOP\n    IF NOT (existing_parent = ANY(desired_memberships)) THEN\n      EXECUTE format('REVOKE %I FROM %I', existing_parent, {username_literal});\n    END IF;\n  END LOOP;\nEND\n$$;"
    )
}

fn render_drop_stale_extra_roles_block(desired: &DesiredManagedRoleSet) -> String {
    let desired_markers = render_text_array_literal(
        desired
            .removable_extra_roles()
            .map(|extra| managed_extra_role_marker(&extra.logical_key))
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str),
    );
    let managed_extra_marker_like =
        sql_literal(format!("{MANAGED_EXTRA_ROLE_COMMENT_PREFIX}%").as_str());
    format!(
        "DO $$\nDECLARE\n  desired_markers text[] := {desired_markers};\n  stale_role text;\n  related_role text;\nBEGIN\n  FOR stale_role IN\n    SELECT rolname\n      FROM pg_roles\n     WHERE shobj_description(oid, 'pg_authid') LIKE {managed_extra_marker_like}\n       AND NOT (shobj_description(oid, 'pg_authid') = ANY(desired_markers))\n  LOOP\n    FOR related_role IN\n      SELECT parent.rolname\n        FROM pg_roles AS parent\n        JOIN pg_auth_members AS membership ON membership.roleid = parent.oid\n        JOIN pg_roles AS child ON child.oid = membership.member\n       WHERE child.rolname = stale_role\n    LOOP\n      EXECUTE format('REVOKE %I FROM %I', related_role, stale_role);\n    END LOOP;\n\n    FOR related_role IN\n      SELECT child.rolname\n        FROM pg_roles AS child\n        JOIN pg_auth_members AS membership ON membership.member = child.oid\n        JOIN pg_roles AS parent ON parent.oid = membership.roleid\n       WHERE parent.rolname = stale_role\n    LOOP\n      EXECUTE format('REVOKE %I FROM %I', stale_role, related_role);\n    END LOOP;\n\n    EXECUTE format('DROP ROLE %I', stale_role);\n  END LOOP;\nEND\n$$;"
    )
}

fn resolve_role_password_literal(spec: &ManagedRoleSpec) -> Result<String, RoleProvisionError> {
    let field = match &spec.identity {
        ManagedRoleIdentity::Mandatory(kind) => {
            format!("postgres.roles.mandatory.{}.auth.password", kind.config_key())
        }
        ManagedRoleIdentity::Extra(logical_key) => {
            format!("postgres.roles.extra.{}.auth.password", logical_key.as_str())
        }
    };
    match &spec.auth {
        RoleAuthConfig::Password { password } => resolve_secret_string(field.as_str(), password)
            .map(|resolved| sql_literal(resolved.as_str()))
            .map_err(|err| RoleProvisionError::RenderSql(err.to_string())),
    }
}

fn render_role_attributes(privilege: PostgresRolePrivilege) -> &'static str {
    match privilege {
        PostgresRolePrivilege::Login => "LOGIN NOSUPERUSER NOREPLICATION",
        PostgresRolePrivilege::Replication => "LOGIN REPLICATION NOSUPERUSER",
        PostgresRolePrivilege::Superuser => "LOGIN SUPERUSER NOREPLICATION",
    }
}

fn render_text_array_literal<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let entries = values.map(sql_literal).collect::<Vec<_>>();
    if entries.is_empty() {
        "ARRAY[]::text[]".to_string()
    } else {
        format!("ARRAY[{}]::text[]", entries.join(", "))
    }
}

fn managed_extra_role_marker(logical_key: &ManagedPostgresRoleKey) -> String {
    format!("{MANAGED_EXTRA_ROLE_COMMENT_PREFIX}{}", logical_key.as_str())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::{
            ManagedPostgresRoleKey, PostgresRoleName, PostgresRolePrivilege, RoleAuthConfig,
            SecretSource,
        },
        dev_support::runtime_config::RuntimeConfigBuilder,
    };

    use super::render_managed_role_reconciliation_sql;

    fn inline_password(password: &str) -> SecretSource {
        SecretSource::Inline {
            content: password.to_string(),
        }
    }

    #[test]
    fn renders_only_mandatory_roles_when_no_extra_roles_are_configured() -> Result<(), String> {
        let cfg = RuntimeConfigBuilder::new().build();
        let sql = render_managed_role_reconciliation_sql(&cfg)
            .map_err(|err| format!("render sql failed: {err}"))?;

        assert!(sql.contains("ALTER ROLE %I WITH LOGIN SUPERUSER NOREPLICATION PASSWORD %L"));
        assert!(sql.contains("ALTER ROLE %I WITH LOGIN REPLICATION NOSUPERUSER PASSWORD %L"));
        assert!(sql.contains("ALTER ROLE %I WITH LOGIN NOSUPERUSER NOREPLICATION PASSWORD %L"));
        assert!(sql.contains("GRANT EXECUTE ON FUNCTION pg_catalog.pg_ls_dir"));
        assert!(sql.contains("pgtuskmaster:managed-extra:%"));
        Ok(())
    }

    #[test]
    fn renders_extra_role_provision_membership_and_stale_drop_logic() -> Result<(), String> {
        let cfg = RuntimeConfigBuilder::new()
            .transform_postgres(|postgres| crate::config::PostgresConfig {
                roles: crate::config::PostgresRolesConfig {
                    extra: BTreeMap::from([(
                        ManagedPostgresRoleKey("analytics".to_string()),
                        crate::config::ExtraManagedPostgresRoleConfig {
                            role: crate::config::PostgresRoleConfig {
                                username: PostgresRoleName("analytics_user".to_string()),
                                auth: RoleAuthConfig::Password {
                                    password: inline_password("analytics-secret"),
                                },
                            },
                            privilege: PostgresRolePrivilege::Login,
                            member_of: vec![
                                PostgresRoleName("reporting".to_string()),
                                PostgresRoleName("reader".to_string()),
                            ],
                        },
                    )]),
                    ..postgres.roles
                },
                ..postgres
            })
            .build();

        let sql = render_managed_role_reconciliation_sql(&cfg)
            .map_err(|err| format!("render sql failed: {err}"))?;

        assert!(sql.contains("pgtuskmaster:managed-extra:analytics"));
        assert!(sql.contains("ALTER ROLE %I WITH LOGIN NOSUPERUSER NOREPLICATION PASSWORD %L"));
        assert!(sql.contains("COMMENT ON ROLE %I IS %L"));
        assert!(sql.contains("desired_memberships text[] := ARRAY['reporting', 'reader']::text[]"));
        assert!(sql.contains("GRANT %I TO %I"));
        assert!(sql.contains("REVOKE %I FROM %I"));
        assert!(sql.contains("DROP ROLE %I"));
        Ok(())
    }

    #[test]
    fn renders_drop_block_that_only_targets_tagged_extra_roles() -> Result<(), String> {
        let sql = render_managed_role_reconciliation_sql(&RuntimeConfigBuilder::new().build())
            .map_err(|err| format!("render sql failed: {err}"))?;

        assert!(sql.contains("shobj_description(oid, 'pg_authid') LIKE 'pgtuskmaster:managed-extra:%'"));
        assert!(!sql.contains("DROP ROLE \"postgres\""));
        assert!(!sql.contains("DROP ROLE \"replicator\""));
        assert!(!sql.contains("DROP ROLE \"rewinder\""));
        Ok(())
    }
}
