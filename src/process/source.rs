use thiserror::Error;

use crate::{
    dcs::{DcsMemberPostgresView, DcsMemberView},
    pginfo::state::PgConnInfo,
    process::{
        jobs::{ReplicatorSourceConn, RewinderSourceConn},
        state::{ProcessIntentRuntime, RemoteRoleProfile},
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

pub(crate) fn basebackup_source_from_member(
    self_id: &MemberId,
    runtime: &ProcessIntentRuntime,
    member: &DcsMemberView,
) -> Result<ReplicatorSourceConn, SourceMaterializationError> {
    validate_remote_primary_source(self_id, member)?;
    Ok(ReplicatorSourceConn {
        conninfo: remote_conninfo(member, &runtime.remote_source.replicator, runtime),
        auth: runtime.remote_source.replicator.auth.clone(),
    })
}

pub(crate) fn rewind_source_from_member(
    self_id: &MemberId,
    runtime: &ProcessIntentRuntime,
    member: &DcsMemberView,
) -> Result<RewinderSourceConn, SourceMaterializationError> {
    validate_remote_primary_source(self_id, member)?;
    Ok(RewinderSourceConn {
        conninfo: remote_conninfo(member, &runtime.remote_source.rewinder, runtime),
        auth: runtime.remote_source.rewinder.auth.clone(),
    })
}

fn validate_remote_primary_source(
    self_id: &MemberId,
    member: &DcsMemberView,
) -> Result<(), SourceMaterializationError> {
    if &member.member_id == self_id {
        return Err(SourceMaterializationError::SelfTarget {
            member_id: member.member_id.0.clone(),
        });
    }

    if member.routing.postgres.host.trim().is_empty() {
        return Err(SourceMaterializationError::EmptyHost {
            member_id: member.member_id.0.clone(),
        });
    }

    if !matches!(member.postgres, DcsMemberPostgresView::Primary(_)) {
        return Err(SourceMaterializationError::NotHealthyPrimary {
            member_id: member.member_id.0.clone(),
        });
    }

    Ok(())
}

fn remote_conninfo(
    member: &DcsMemberView,
    role: &RemoteRoleProfile,
    runtime: &ProcessIntentRuntime,
) -> PgConnInfo {
    PgConnInfo {
        host: member.routing.postgres.host.clone(),
        port: member.routing.postgres.port,
        user: role.username.clone(),
        dbname: runtime.remote_source.dbname.clone(),
        application_name: None,
        connect_timeout_s: Some(runtime.remote_source.connect_timeout_s),
        ssl_mode: runtime.remote_source.ssl_mode,
        ssl_root_cert: runtime.remote_source.ssl_root_cert.clone(),
        options: None,
    }
}
