use crate::support::{
    error::{HarnessError, Result},
    faults::DCS_SERVICES,
    topology::{ClusterMember, DcsService},
};

const SHARED_FIXTURE_RELATIVE_PATHS: [&str; 4] = [
    "configs/tls",
    "secrets",
    "configs/pg_hba.conf",
    "configs/pg_ident.conf",
];
const SHARED_ETCD_SERVICES: [DcsService; 1] = [DcsService::SharedEtcd];
const THREE_ETCD_QUORUM_SERVICES: [DcsService; 2] = [DcsService::EtcdA, DcsService::EtcdB];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HaGivenId {
    Plain,
    CustomRoles,
    ThreeEtcd,
}

impl HaGivenId {
    pub fn local_dcs_service_for(self, member: ClusterMember) -> DcsService {
        match self {
            Self::Plain | Self::CustomRoles => DcsService::SharedEtcd,
            Self::ThreeEtcd => member.local_dcs_service(),
        }
    }

    pub fn artifact_service_names(self) -> Vec<&'static str> {
        self.dcs_services()
            .iter()
            .copied()
            .map(DcsService::service_name)
            .chain(
                ClusterMember::ALL
                    .into_iter()
                    .map(ClusterMember::service_name),
            )
            .collect()
    }

    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "three_node_plain" => Ok(Self::Plain),
            "three_node_custom_roles" => Ok(Self::CustomRoles),
            "three_node_three_etcd" => Ok(Self::ThreeEtcd),
            _ => Err(HarnessError::message(format!(
                "unsupported HA given `{raw}`"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "three_node_plain",
            Self::CustomRoles => "three_node_custom_roles",
            Self::ThreeEtcd => "three_node_three_etcd",
        }
    }

    pub fn replicator_role(self) -> &'static str {
        match self {
            Self::Plain | Self::ThreeEtcd => "replicator",
            Self::CustomRoles => "mirrorbot",
        }
    }

    pub fn rewinder_role(self) -> &'static str {
        match self {
            Self::Plain | Self::ThreeEtcd => "rewinder",
            Self::CustomRoles => "rewindbot",
        }
    }

    pub fn dcs_services(self) -> &'static [DcsService] {
        match self {
            Self::Plain | Self::CustomRoles => &SHARED_ETCD_SERVICES,
            Self::ThreeEtcd => &DCS_SERVICES,
        }
    }

    pub fn quorum_majority_dcs_services(self) -> &'static [DcsService] {
        match self {
            Self::Plain | Self::CustomRoles => &SHARED_ETCD_SERVICES,
            Self::ThreeEtcd => &THREE_ETCD_QUORUM_SERVICES,
        }
    }

    pub fn compose_variant_relative_path(self) -> &'static str {
        match self {
            Self::Plain | Self::CustomRoles => "compose/three_node_shared_single.yml",
            Self::ThreeEtcd => "compose/three_node_three_etcd.yml",
        }
    }

    pub fn shared_fixture_relative_paths(self) -> &'static [&'static str] {
        match self {
            Self::Plain | Self::CustomRoles | Self::ThreeEtcd => &SHARED_FIXTURE_RELATIVE_PATHS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HaGivenId;
    use crate::support::topology::{ClusterMember, DcsService};

    #[test]
    fn shared_topology_givens_share_the_same_dcs_layout() {
        for given in [HaGivenId::Plain, HaGivenId::CustomRoles] {
            assert_eq!(given.dcs_services(), &[DcsService::SharedEtcd]);
            assert_eq!(
                given.quorum_majority_dcs_services(),
                &[DcsService::SharedEtcd]
            );
            assert_eq!(
                given.local_dcs_service_for(ClusterMember::NodeA),
                DcsService::SharedEtcd
            );
        }
    }

    #[test]
    fn three_etcd_given_uses_member_local_dcs_routing() {
        assert_eq!(
            HaGivenId::ThreeEtcd.local_dcs_service_for(ClusterMember::NodeA),
            DcsService::EtcdA
        );
        assert_eq!(
            HaGivenId::ThreeEtcd.local_dcs_service_for(ClusterMember::NodeB),
            DcsService::EtcdB
        );
    }
}
