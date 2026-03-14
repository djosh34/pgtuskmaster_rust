use thiserror::Error;

use crate::{
    dcs::{ClusterMemberView, MemberPostgresView},
    pginfo::state::PgConnInfo,
    process::{
        jobs::{MandatoryRoleSourceConn, MandatorySourceRole},
        state::{MandatoryPostgresRoleCredential, ProcessRuntimePlan},
    },
    state::{MemberId, PgConnectTarget},
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

pub(crate) fn basebackup_source_from_member(
    self_id: &MemberId,
    runtime: &ProcessRuntimePlan,
    member_id: &MemberId,
    member: &ClusterMemberView,
) -> Result<MandatoryRoleSourceConn, SourceMaterializationError> {
    validate_remote_primary_source(self_id, member_id, member)?;
    Ok(MandatoryRoleSourceConn {
        role: MandatorySourceRole::Replicator,
        conninfo: remote_conninfo(member, &runtime.replica_access.roles.replicator, runtime),
        auth: runtime.replica_access.roles.replicator.auth.clone(),
    })
}

pub(crate) fn rewind_source_from_member(
    self_id: &MemberId,
    runtime: &ProcessRuntimePlan,
    member_id: &MemberId,
    member: &ClusterMemberView,
) -> Result<MandatoryRoleSourceConn, SourceMaterializationError> {
    validate_remote_primary_source(self_id, member_id, member)?;
    Ok(MandatoryRoleSourceConn {
        role: MandatorySourceRole::Rewinder,
        conninfo: remote_conninfo(member, &runtime.replica_access.roles.rewinder, runtime),
        auth: runtime.replica_access.roles.rewinder.auth.clone(),
    })
}

fn validate_remote_primary_source(
    self_id: &MemberId,
    member_id: &MemberId,
    member: &ClusterMemberView,
) -> Result<(), SourceMaterializationError> {
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

    if !matches!(member.postgres(), MemberPostgresView::Primary { .. }) {
        return Err(SourceMaterializationError::NotHealthyPrimary {
            member_id: member_id.0.clone(),
        });
    }

    Ok(())
}

fn remote_conninfo(
    member: &ClusterMemberView,
    role: &MandatoryPostgresRoleCredential,
    runtime: &ProcessRuntimePlan,
) -> PgConnInfo {
    PgConnInfo {
        target: PgConnectTarget::Tcp(member.postgres_target().clone()),
        user: role.username.as_str().to_owned(),
        dbname: runtime.replica_access.dbname.clone(),
        application_name: None,
        connect_timeout_s: Some(runtime.replica_access.connect_timeout_s),
        ssl_mode: runtime.replica_access.ssl_mode,
        ssl_root_cert: runtime.replica_access.ssl_root_cert.clone(),
        options: None,
    }
}
