use thiserror::Error;

use crate::{
    dcs::DcsMemberState,
    pginfo::{
        conninfo::PgClientTls,
        state::{PgConnInfo, PgInfoState},
    },
    process::{
        jobs::{MandatoryRoleSourceConn, MandatorySourceRole},
        state::ProcessRuntimePlan,
    },
    state::MemberId,
};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum SourceMaterializationError {
    #[error("remote source member `{member_id}` is self")]
    SelfTarget { member_id: String },
    #[error("remote source member `{member_id}` is not a healthy primary")]
    NotHealthyPrimary { member_id: String },
    #[error("remote source member `{member_id}` has an empty postgres host")]
    EmptyHost { member_id: String },
}

pub(crate) fn source_from_member(
    self_id: &MemberId,
    runtime: &ProcessRuntimePlan,
    member_id: &MemberId,
    member: &DcsMemberState,
    role: MandatorySourceRole,
) -> Result<MandatoryRoleSourceConn, SourceMaterializationError> {
    if member_id == self_id {
        return Err(SourceMaterializationError::SelfTarget {
            member_id: member_id.0.clone(),
        });
    }

    if member.postgres_target().host().trim().is_empty() {
        return Err(SourceMaterializationError::EmptyHost {
            member_id: member_id.0.clone(),
        });
    }

    if !matches!(member.postgres(), PgInfoState::Primary { .. }) {
        return Err(SourceMaterializationError::NotHealthyPrimary {
            member_id: member_id.0.clone(),
        });
    }

    let credential = match &role {
        MandatorySourceRole::Replicator => &runtime.replica_access.roles.replicator,
        MandatorySourceRole::Rewinder => &runtime.replica_access.roles.rewinder,
    };
    Ok(MandatoryRoleSourceConn {
        role,
        conninfo: PgConnInfo {
            endpoint: member.postgres_target().clone(),
            hostaddr: None,
            user: credential.username.as_str().to_owned(),
            dbname: runtime.replica_access.dbname.clone(),
            application_name: None,
            connect_timeout_s: Some(runtime.replica_access.connect_timeout_s),
            options: None,
            tls: PgClientTls {
                mode: runtime.replica_access.ssl_mode,
                root_cert: runtime.replica_access.ssl_root_cert.clone(),
                client_cert: None,
                client_key: None,
            },
        },
        auth: credential.auth.clone(),
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        dcs::DcsMemberState,
        dev_support::runtime_config::RuntimeConfigBuilder,
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::{
            jobs::MandatorySourceRole,
            source::{source_from_member, SourceMaterializationError},
            state::ProcessRuntimePlan,
        },
        state::{MemberId, PgEndpoint, SystemIdentifier, TimelineId, UnixMillis, WorkerStatus},
    };

    fn primary_member(host: &str, port: u16) -> Result<DcsMemberState, String> {
        Ok(DcsMemberState {
            postgres_endpoint: PgEndpoint::tcp(host.to_string(), port)?,
            postgres: PgInfoState::Primary {
                common: PgInfoCommon {
                    worker: WorkerStatus::Running,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: Some(TimelineId(1)),
                    system_identifier: Some(SystemIdentifier(7)),
                    pg_config: PgConfig {
                        port: Some(port),
                        hot_standby: Some(false),
                        primary_conninfo: None,
                        primary_slot_name: None,
                        extra: std::collections::BTreeMap::new(),
                    },
                    last_refresh_at: Some(UnixMillis(1)),
                },
                wal_lsn: crate::state::WalLsn(8),
                slots: Vec::new(),
            },
        })
    }

    #[test]
    fn source_from_member_selects_role_specific_credentials() -> Result<(), String> {
        let runtime_config = RuntimeConfigBuilder::new().build();
        let runtime = ProcessRuntimePlan::from_config(&runtime_config);
        let member_id = MemberId("node-b".to_string());
        let member = primary_member("10.0.0.9", 5432)?;

        let replicator = source_from_member(
            &MemberId("node-a".to_string()),
            &runtime,
            &member_id,
            &member,
            MandatorySourceRole::Replicator,
        )
        .map_err(|err| err.to_string())?;
        let rewinder = source_from_member(
            &MemberId("node-a".to_string()),
            &runtime,
            &member_id,
            &member,
            MandatorySourceRole::Rewinder,
        )
        .map_err(|err| err.to_string())?;

        assert_eq!(replicator.role, MandatorySourceRole::Replicator);
        assert_eq!(
            replicator.conninfo.user,
            runtime
                .replica_access
                .roles
                .replicator
                .username
                .as_str()
                .to_string()
        );
        assert_eq!(
            replicator.conninfo.connect_timeout_s,
            Some(runtime.replica_access.connect_timeout_s)
        );
        assert_eq!(
            replicator.auth,
            runtime.replica_access.roles.replicator.auth.clone()
        );

        assert_eq!(rewinder.role, MandatorySourceRole::Rewinder);
        assert_eq!(
            rewinder.conninfo.user,
            runtime
                .replica_access
                .roles
                .rewinder
                .username
                .as_str()
                .to_string()
        );
        assert_eq!(
            rewinder.auth,
            runtime.replica_access.roles.rewinder.auth.clone()
        );
        Ok(())
    }

    #[test]
    fn source_from_member_rejects_self_target() -> Result<(), String> {
        let runtime = ProcessRuntimePlan::from_config(&RuntimeConfigBuilder::new().build());
        let member_id = MemberId("node-a".to_string());
        let member = primary_member("10.0.0.9", 5432)?;

        let err = source_from_member(
            &member_id,
            &runtime,
            &member_id,
            &member,
            MandatorySourceRole::Replicator,
        );

        assert_eq!(
            err,
            Err(SourceMaterializationError::SelfTarget {
                member_id: "node-a".to_string(),
            })
        );
        Ok(())
    }
}
