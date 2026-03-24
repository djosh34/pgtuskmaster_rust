use crate::{
    config_v2::RuntimeConfigV2,
    postgres_managed::{materialize_managed_postgres_config, ManagedPostgresConfig},
    postgres_managed_conf::ManagedPostgresStartIntent,
    process::{
        planner::{ClusterProcessPlan, DesiredManagedPostgresSession},
        state::ensure_start_paths,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedManagedPostgresSession {
    pub(crate) config: ManagedPostgresConfig,
}

#[derive(Default)]
pub(crate) struct ManagedPostgresSessionMaterializer;

impl ManagedPostgresSessionMaterializer {
    pub(crate) fn materialize(
        &self,
        cfg: &RuntimeConfigV2,
        plan: &ClusterProcessPlan,
    ) -> Result<Option<PreparedManagedPostgresSession>, crate::process::jobs::ProcessError> {
        match plan {
            ClusterProcessPlan::StartManagedPostgres(start) => {
                ensure_start_paths(cfg)?;
                let start_intent = match start.desired_session.clone() {
                    DesiredManagedPostgresSession::Primary => ManagedPostgresStartIntent::primary(),
                    DesiredManagedPostgresSession::DetachedStandby => {
                        ManagedPostgresStartIntent::detached_standby()
                    }
                    DesiredManagedPostgresSession::Follow(plan) => {
                        ManagedPostgresStartIntent::replica(
                            plan.source.conninfo,
                            plan.primary_slot_name,
                        )
                    }
                };
                let config =
                    materialize_managed_postgres_config(cfg, &start_intent).map_err(|err| {
                        crate::process::jobs::ProcessError::InvalidSpec(format!(
                            "materialize managed postgres config failed: {err}"
                        ))
                    })?;
                Ok(Some(PreparedManagedPostgresSession { config }))
            }
            ClusterProcessPlan::Bootstrap(_)
            | ClusterProcessPlan::BaseBackup(_)
            | ClusterProcessPlan::PgRewind(_)
            | ClusterProcessPlan::Promote(_)
            | ClusterProcessPlan::Demote(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        dev_support::{runtime_config::RuntimeConfigBuilder, test_fs::unique_test_dir},
        postgres_managed_conf::{managed_standby_passfile_path, MANAGED_POSTGRESQL_CONF_NAME},
        process::{
            jobs::{MandatoryRoleSourceConn, MandatorySourceRole},
            planner::{
                ClusterProcessPlan, DesiredManagedPostgresSession, ManagedStartPlan,
                ReplicaFollowPlan,
            },
        },
    };

    use super::ManagedPostgresSessionMaterializer;

    fn sample_runtime_config(data_dir: PathBuf) -> crate::config_v2::RuntimeConfigV2 {
        RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir)
            .build()
    }

    #[test]
    fn materialize_follow_session_writes_managed_files_without_tool_lowering() -> Result<(), String>
    {
        let root = unique_test_dir("process-session", "follow")?;
        let data_dir = root.join("data");
        let cfg = sample_runtime_config(data_dir.clone());
        let source = MandatoryRoleSourceConn {
            role: MandatorySourceRole::Replicator,
            conninfo: crate::pginfo::state::PgConnInfo {
                route: crate::state::PgRoute::tcp("10.0.0.10".to_string(), 5432)?,
                user: "replicator".to_string(),
                dbname: "postgres".to_string(),
                application_name: None,
                connect_timeout_s: Some(5),
                options: None,
                tls: crate::pginfo::conninfo::PgClientTls {
                    mode: crate::pginfo::state::PgSslMode::Prefer,
                    root_cert: None,
                    client_cert: None,
                    client_key: None,
                },
            },
            auth: cfg.postgres.replicator.password.clone(),
        };
        let plan = ClusterProcessPlan::StartManagedPostgres(ManagedStartPlan {
            mode: crate::process::jobs::PostgresStartMode::Replica,
            desired_session: DesiredManagedPostgresSession::Follow(Box::new(ReplicaFollowPlan {
                source,
                primary_slot_name: None,
            })),
        });

        let prepared = ManagedPostgresSessionMaterializer
            .materialize(&cfg, &plan)
            .map_err(|err| format!("materialize follow session failed: {err}"))?
            .ok_or_else(|| "expected prepared managed session".to_string())?;

        if !prepared.config.postgresql_conf_path.exists() {
            return Err(format!(
                "managed postgres config file was not written at {}",
                prepared.config.postgresql_conf_path.display()
            ));
        }
        let managed_conf_name = prepared
            .config
            .postgresql_conf_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| "managed postgres conf path is not valid UTF-8".to_string())?;
        if managed_conf_name != MANAGED_POSTGRESQL_CONF_NAME {
            return Err(format!(
                "unexpected managed conf file name: {managed_conf_name}"
            ));
        }
        let passfile_path = managed_standby_passfile_path(&data_dir);
        if !passfile_path.exists() {
            return Err(format!(
                "expected standby passfile to exist at {}",
                passfile_path.display()
            ));
        }

        Ok(())
    }

    #[test]
    fn materialize_skips_non_start_plans() -> Result<(), String> {
        let root = unique_test_dir("process-session", "skip-non-start")?;
        let cfg = sample_runtime_config(root.join("data"));
        let plan = ClusterProcessPlan::Promote(crate::process::jobs::PromoteSpec {
            data_dir: cfg.postgres.data_dir.clone(),
            wait_seconds: None,
            timeout_ms: None,
        });

        let prepared = ManagedPostgresSessionMaterializer
            .materialize(&cfg, &plan)
            .map_err(|err| format!("materialize non-start plan failed: {err}"))?;
        if prepared.is_some() {
            return Err("non-start plan should not materialize managed session".to_string());
        }

        Ok(())
    }
}
