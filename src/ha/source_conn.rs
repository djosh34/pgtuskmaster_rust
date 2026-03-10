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

pub(crate) fn replica_follow_conninfo_from_member(
    self_id: &MemberId,
    member: &MemberRecord,
    defaults: &ProcessDispatchDefaults,
) -> Result<PgConnInfo, SourceConnError> {
    validate_replica_follow_source_member(self_id, member)?;
    Ok(remote_conninfo(
        member,
        defaults.replicator_username.as_str(),
        defaults,
    ))
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

fn validate_replica_follow_source_member(
    self_id: &MemberId,
    member: &MemberRecord,
) -> Result<(), SourceConnError> {
    if &member.member_id == self_id {
        return Err(SourceConnError::SelfTarget {
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
        options: None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dcs::state::{MemberRecord, MemberRole},
        ha::state::ProcessDispatchDefaults,
        pginfo::state::{Readiness, SqlStatus},
        state::{MemberId, UnixMillis, Version},
    };

    use super::{
        basebackup_source_from_member, replica_follow_conninfo_from_member, SourceConnError,
    };

    fn sample_defaults() -> ProcessDispatchDefaults {
        ProcessDispatchDefaults::contract_stub()
    }

    fn sample_member(member_id: &str) -> MemberRecord {
        MemberRecord {
            member_id: MemberId(member_id.to_string()),
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            api_url: None,
            role: MemberRole::Replica,
            sql: SqlStatus::Unknown,
            readiness: Readiness::Unknown,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
            updated_at: UnixMillis(1),
            pg_version: Version(1),
        }
    }

    #[test]
    fn replica_follow_conninfo_accepts_authoritative_member_before_primary_health(
    ) -> Result<(), SourceConnError> {
        let defaults = sample_defaults();
        let source = sample_member("node-2");

        let conninfo = replica_follow_conninfo_from_member(
            &MemberId("node-1".to_string()),
            &source,
            &defaults,
        )?;

        assert_eq!(conninfo.host, "127.0.0.1");
        assert_eq!(conninfo.port, 5432);
        assert_eq!(conninfo.user, "replicator");
        Ok(())
    }

    #[test]
    fn basebackup_source_still_requires_healthy_primary() {
        let defaults = sample_defaults();
        let source = sample_member("node-2");

        assert_eq!(
            basebackup_source_from_member(&MemberId("node-1".to_string()), &source, &defaults),
            Err(SourceConnError::NotHealthyPrimary {
                member_id: "node-2".to_string(),
            })
        );
    }
}
