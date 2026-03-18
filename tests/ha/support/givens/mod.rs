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

#[derive(Clone, Copy, Debug)]
pub(crate) struct HaGivenDefinition {
    pub name: &'static str,
    pub replicator_role: &'static str,
    pub rewinder_role: &'static str,
    pub dcs_services: &'static [DcsService],
    pub quorum_majority_dcs_services: &'static [DcsService],
    pub compose_variant_relative_path: &'static str,
    pub shared_fixture_relative_paths: &'static [&'static str],
    shared_dcs_service: Option<DcsService>,
}

impl HaGivenDefinition {
    pub fn local_dcs_service_for(self, member: ClusterMember) -> DcsService {
        self.shared_dcs_service
            .unwrap_or_else(|| member.local_dcs_service())
    }

    pub fn artifact_service_names(self) -> Vec<&'static str> {
        self.dcs_services
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
}

const PLAIN_GIVEN: HaGivenDefinition = HaGivenDefinition {
    name: "three_node_plain",
    replicator_role: "replicator",
    rewinder_role: "rewinder",
    dcs_services: &SHARED_ETCD_SERVICES,
    quorum_majority_dcs_services: &SHARED_ETCD_SERVICES,
    compose_variant_relative_path: "compose/three_node_shared_single.yml",
    shared_fixture_relative_paths: &SHARED_FIXTURE_RELATIVE_PATHS,
    shared_dcs_service: Some(DcsService::SharedEtcd),
};

const CUSTOM_ROLES_GIVEN: HaGivenDefinition = HaGivenDefinition {
    name: "three_node_custom_roles",
    replicator_role: "mirrorbot",
    rewinder_role: "rewindbot",
    dcs_services: &SHARED_ETCD_SERVICES,
    quorum_majority_dcs_services: &SHARED_ETCD_SERVICES,
    compose_variant_relative_path: "compose/three_node_shared_single.yml",
    shared_fixture_relative_paths: &SHARED_FIXTURE_RELATIVE_PATHS,
    shared_dcs_service: Some(DcsService::SharedEtcd),
};

const THREE_ETCD_GIVEN: HaGivenDefinition = HaGivenDefinition {
    name: "three_node_three_etcd",
    replicator_role: "replicator",
    rewinder_role: "rewinder",
    dcs_services: &DCS_SERVICES,
    quorum_majority_dcs_services: &THREE_ETCD_QUORUM_SERVICES,
    compose_variant_relative_path: "compose/three_node_three_etcd.yml",
    shared_fixture_relative_paths: &SHARED_FIXTURE_RELATIVE_PATHS,
    shared_dcs_service: None,
};

impl HaGivenId {
    pub(crate) fn definition(self) -> HaGivenDefinition {
        match self {
            Self::Plain => PLAIN_GIVEN,
            Self::CustomRoles => CUSTOM_ROLES_GIVEN,
            Self::ThreeEtcd => THREE_ETCD_GIVEN,
        }
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
        self.definition().name
    }
}

#[cfg(test)]
mod tests {
    use super::HaGivenId;
    use crate::support::topology::{ClusterMember, DcsService};

    #[test]
    fn shared_topology_givens_share_the_same_dcs_layout() {
        for given in [HaGivenId::Plain, HaGivenId::CustomRoles] {
            let definition = given.definition();
            assert_eq!(definition.dcs_services, &[DcsService::SharedEtcd]);
            assert_eq!(
                definition.quorum_majority_dcs_services,
                &[DcsService::SharedEtcd]
            );
            assert_eq!(
                definition.local_dcs_service_for(ClusterMember::NodeA),
                DcsService::SharedEtcd
            );
        }
    }

    #[test]
    fn three_etcd_given_uses_member_local_dcs_routing() {
        let definition = HaGivenId::ThreeEtcd.definition();
        assert_eq!(
            definition.local_dcs_service_for(ClusterMember::NodeA),
            DcsService::EtcdA
        );
        assert_eq!(
            definition.local_dcs_service_for(ClusterMember::NodeB),
            DcsService::EtcdB
        );
    }
}
