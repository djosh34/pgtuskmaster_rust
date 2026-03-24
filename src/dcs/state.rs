use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};

use crate::{
    pginfo::state::PgInfoState,
    state::{ApiRoute, LeaseEpoch, MemberId, PgRoute, SwitchoverState},
};

macro_rules! test_public_method {
    ($(#[$meta:meta])* fn $name:ident($($args:tt)*) -> $ret:ty $body:block) => {
        #[cfg(any(test, feature = "internal-test-support"))]
        $(#[$meta])*
        pub fn $name($($args)*) -> $ret $body

        #[cfg(not(any(test, feature = "internal-test-support")))]
        $(#[$meta])*
        pub(crate) fn $name($($args)*) -> $ret $body
    };
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsMemberState {
    pub cluster_postgres: PgRoute,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_postgres: Option<PgRoute>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_api: Option<ApiRoute>,
    pub postgres: PgInfoState,
}

impl DcsMemberState {
    test_public_method! {
    fn cluster_postgres_target(&self) -> &PgRoute {
        &self.cluster_postgres
    }}

    test_public_method! {
    fn operator_or_cluster_postgres_target(&self) -> &PgRoute {
        self.operator_postgres
            .as_ref()
            .unwrap_or(&self.cluster_postgres)
    }}

    test_public_method! {
    fn operator_api_target(&self) -> Option<&ApiRoute> {
        self.operator_api.as_ref()
    }}

    test_public_method! {
    fn postgres(&self) -> &PgInfoState {
        &self.postgres
    }}
}

#[cfg(any(test, feature = "internal-test-support"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DcsAuthority {
    NoQuorum,
    Quorum,
}

#[cfg(not(any(test, feature = "internal-test-support")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DcsAuthority {
    NoQuorum,
    Quorum,
}

impl fmt::Display for DcsAuthority {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NoQuorum => "no_quorum",
            Self::Quorum => "quorum",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsQuorumState {
    pub leadership: Option<LeaseEpoch>,
    pub switchover: SwitchoverState,
    pub members: BTreeMap<MemberId, DcsMemberState>,
}

impl DcsQuorumState {
    test_public_method! {
    fn members(&self) -> impl Iterator<Item = (&MemberId, &DcsMemberState)> {
        self.members.iter()
    }}

    test_public_method! {
    fn member(&self, member_id: &MemberId) -> Option<&DcsMemberState> {
        self.members.get(member_id)
    }}
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsSnapshot {
    NoQuorum,
    Quorum(DcsQuorumState),
}

impl DcsSnapshot {
    test_public_method! {
    fn starting() -> Self {
        Self::NoQuorum
    }}

    #[cfg(any(test, feature = "internal-test-support"))]
    pub fn quorum(
        leadership: Option<LeaseEpoch>,
        switchover: SwitchoverState,
        members: BTreeMap<MemberId, DcsMemberState>,
    ) -> Self {
        Self::Quorum(DcsQuorumState {
            leadership,
            switchover,
            members,
        })
    }

    test_public_method! {
    fn authority(&self) -> DcsAuthority {
        match self {
            Self::NoQuorum => DcsAuthority::NoQuorum,
            Self::Quorum(_) => DcsAuthority::Quorum,
        }
    }}

    test_public_method! {
    fn is_quorum(&self) -> bool {
        matches!(self, Self::Quorum(_))
    }}

    test_public_method! {
    fn quorum_state(&self) -> Option<&DcsQuorumState> {
        match self {
            Self::Quorum(cluster) => Some(cluster),
            Self::NoQuorum => None,
        }
    }}

    test_public_method! {
    fn members(&self) -> impl Iterator<Item = (&MemberId, &DcsMemberState)> {
        self.members_map()
            .into_iter()
            .flat_map(|members| members.iter())
    }}

    test_public_method! {
    fn member_count(&self) -> usize {
        self.members_map().map(|members| members.len()).unwrap_or(0)
    }}

    test_public_method! {
    fn member(&self, member_id: &MemberId) -> Option<&DcsMemberState> {
        self.members_map()
            .and_then(|members| members.get(member_id))
    }}

    test_public_method! {
    fn switchover(&self) -> Option<&SwitchoverState> {
        match self {
            Self::NoQuorum => None,
            Self::Quorum(cluster) => Some(&cluster.switchover),
        }
    }}

    fn members_map(&self) -> Option<&BTreeMap<MemberId, DcsMemberState>> {
        match self {
            Self::NoQuorum => None,
            Self::Quorum(cluster) => Some(&cluster.members),
        }
    }
}

pub(crate) fn build_local_member_state(
    cluster_postgres: &PgRoute,
    operator_postgres: Option<&PgRoute>,
    operator_api: Option<&ApiRoute>,
    pg_snapshot: &PgInfoState,
) -> DcsMemberState {
    DcsMemberState {
        cluster_postgres: cluster_postgres.clone(),
        operator_postgres: operator_postgres.cloned(),
        operator_api: operator_api.cloned(),
        postgres: pg_snapshot.clone(),
    }
}

pub(crate) fn current_snapshot(
    etcd_reachable: bool,
    _self_id: &MemberId,
    leadership: &Option<LeaseEpoch>,
    switchover: &SwitchoverState,
    members: &BTreeMap<MemberId, DcsMemberState>,
) -> DcsSnapshot {
    if !etcd_reachable {
        return DcsSnapshot::NoQuorum;
    }

    if !has_member_quorum(members) {
        return DcsSnapshot::NoQuorum;
    }

    DcsSnapshot::Quorum(DcsQuorumState {
        leadership: leadership.clone(),
        switchover: switchover.clone(),
        members: members.clone(),
    })
}

fn has_member_quorum(members: &BTreeMap<MemberId, DcsMemberState>) -> bool {
    if members.len() <= 1 {
        members.len() == 1
    } else {
        members.len() >= 2
    }
}
