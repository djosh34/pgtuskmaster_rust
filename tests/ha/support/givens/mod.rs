use std::path::{Path, PathBuf};

use crate::support::{
    error::{HarnessError, Result},
    faults::DCS_SERVICES,
    topology::{ClusterMember, DcsService},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HaGivenId {
    Plain,
    CustomRoles,
    ThreeEtcd,
}

impl HaGivenId {
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HaGivenDefinition {
    pub id: HaGivenId,
    shared_root: PathBuf,
}

impl HaGivenDefinition {
    pub fn shared_root(&self) -> &Path {
        self.shared_root.as_path()
    }

    pub fn replicator_role(&self) -> &'static str {
        match self.id {
            HaGivenId::Plain | HaGivenId::ThreeEtcd => "replicator",
            HaGivenId::CustomRoles => "mirrorbot",
        }
    }

    pub fn rewinder_role(&self) -> &'static str {
        match self.id {
            HaGivenId::Plain | HaGivenId::ThreeEtcd => "rewinder",
            HaGivenId::CustomRoles => "rewindbot",
        }
    }

    pub fn dcs_services(&self) -> Vec<DcsService> {
        match self.id {
            HaGivenId::Plain | HaGivenId::CustomRoles => vec![DcsService::SharedEtcd],
            HaGivenId::ThreeEtcd => DCS_SERVICES.into_iter().collect(),
        }
    }

    pub fn artifact_service_names(&self) -> Vec<&'static str> {
        self.dcs_services()
            .into_iter()
            .map(DcsService::service_name)
            .chain(
                ClusterMember::ALL
                    .into_iter()
                    .map(ClusterMember::service_name),
            )
            .collect()
    }

    pub fn local_dcs_service_for(&self, member: ClusterMember) -> DcsService {
        match self.id {
            HaGivenId::Plain | HaGivenId::CustomRoles => DcsService::SharedEtcd,
            HaGivenId::ThreeEtcd => member.local_dcs_service(),
        }
    }

    pub fn quorum_majority_dcs_services(&self) -> Vec<DcsService> {
        match self.id {
            HaGivenId::Plain | HaGivenId::CustomRoles => vec![DcsService::SharedEtcd],
            HaGivenId::ThreeEtcd => [DcsService::EtcdA, DcsService::EtcdB].into_iter().collect(),
        }
    }

    pub fn compose_variant_relative_path(&self) -> &'static str {
        match self.id {
            HaGivenId::Plain | HaGivenId::CustomRoles => "compose/three_node_shared_single.yml",
            HaGivenId::ThreeEtcd => "compose/three_node_three_etcd.yml",
        }
    }

    pub fn shared_fixture_relative_paths(&self) -> &'static [&'static str] {
        &[
            "configs/tls",
            "secrets",
            "configs/pg_hba.conf",
            "configs/pg_ident.conf",
        ]
    }
}

pub fn resolve_given(repo_root: &Path, given: HaGivenId) -> Result<HaGivenDefinition> {
    let givens_root = repo_root.join("tests/ha/givens");
    Ok(HaGivenDefinition {
        id: given,
        shared_root: givens_root.join("three_node_shared"),
    })
}
