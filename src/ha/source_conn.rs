use thiserror::Error;

use crate::{
    dcs::state::{MemberRecord, MemberRole},
    pginfo::state::PgConnInfo,
    process::jobs::{ReplicatorSourceConn, RewinderSourceConn},
    state::MemberId,
};

use super::state::ProcessDispatchDefaults;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum SourceConnError {
    #[error("remote source member `{member_id}` is self")]
    SelfTarget { member_id: String },
    #[error("remote source member `{member_id}` is not a healthy primary")]
    NotHealthyPrimary { member_id: String },
    #[error("remote source member `{member_id}` has an empty postgres host")]
    EmptyHost { member_id: String },
}

pub(crate) fn basebackup_source_from_member(
    self_id: &MemberId,
    member: &MemberRecord,
    defaults: &ProcessDispatchDefaults,
) -> Result<ReplicatorSourceConn, SourceConnError> {
    validate_remote_source_member(self_id, member)?;
    Ok(ReplicatorSourceConn {
        conninfo: remote_conninfo(member, defaults.replicator_username.as_str(), defaults),
        auth: defaults.replicator_auth.clone(),
    })
}

pub(crate) fn rewind_source_from_member(
    self_id: &MemberId,
    member: &MemberRecord,
    defaults: &ProcessDispatchDefaults,
) -> Result<RewinderSourceConn, SourceConnError> {
    validate_remote_source_member(self_id, member)?;
    Ok(RewinderSourceConn {
        conninfo: remote_conninfo(member, defaults.rewinder_username.as_str(), defaults),
        auth: defaults.rewinder_auth.clone(),
    })
}

fn validate_remote_source_member(
    self_id: &MemberId,
    member: &MemberRecord,
) -> Result<(), SourceConnError> {
    if &member.member_id == self_id {
        return Err(SourceConnError::SelfTarget {
            member_id: member.member_id.0.clone(),
        });
    }

    if member.role != MemberRole::Primary || member.sql != crate::pginfo::state::SqlStatus::Healthy
    {
        return Err(SourceConnError::NotHealthyPrimary {
            member_id: member.member_id.0.clone(),
        });
    }

    if member.postgres_host.trim().is_empty() {
        return Err(SourceConnError::EmptyHost {
            member_id: member.member_id.0.clone(),
        });
    }

    Ok(())
}

fn remote_conninfo(
    member: &MemberRecord,
    user: &str,
    defaults: &ProcessDispatchDefaults,
) -> PgConnInfo {
    PgConnInfo {
        host: member.postgres_host.clone(),
        port: member.postgres_port,
        user: user.to_string(),
        dbname: defaults.remote_dbname.clone(),
        application_name: None,
        connect_timeout_s: Some(defaults.connect_timeout_s),
        ssl_mode: defaults.remote_ssl_mode,
        ssl_root_cert: defaults.remote_ssl_root_cert.clone(),
        options: None,
    }
}
