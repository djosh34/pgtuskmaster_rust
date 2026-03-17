use std::path::{Path, PathBuf};

use crate::support::{
    error::{HarnessError, Result},
    faults::DCS_MEMBERS,
    topology::{ClusterMember, ComposeService, DcsMember, DcsService},
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
    pub topology: HaTopologyFixture,
    pub materialization: FixtureMaterialization,
}

impl HaGivenDefinition {
    pub fn dcs_services(&self) -> Vec<DcsService> {
        match &self.topology {
            HaTopologyFixture::ThreeNode(topology) => topology.dcs_layout.dcs_services(),
        }
    }

    pub fn support_services(&self) -> Vec<ComposeService> {
        self.dcs_services()
            .into_iter()
            .map(ComposeService::from)
            .collect()
    }

    pub fn artifact_services(&self) -> Vec<ComposeService> {
        self.support_services()
            .into_iter()
            .chain(ClusterMember::ALL.into_iter().map(ComposeService::from))
            .collect()
    }

    pub fn member_binding(&self, member: ClusterMember) -> MemberDcsBinding {
        match &self.topology {
            HaTopologyFixture::ThreeNode(topology) => topology.member_binding(member),
        }
    }

    pub fn local_dcs_service_for(&self, member: ClusterMember) -> DcsService {
        self.member_binding(member).dcs_service
    }

    pub fn quorum_majority_dcs_services(&self) -> Vec<DcsService> {
        match &self.topology {
            HaTopologyFixture::ThreeNode(topology) => {
                topology.dcs_layout.quorum_majority_services()
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HaTopologyFixture {
    ThreeNode(ThreeNodeTopologyFixture),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThreeNodeTopologyFixture {
    pub postgres_roles: PostgresRoleMapping,
    pub dcs_layout: ThreeNodeDcsLayout,
}

impl ThreeNodeTopologyFixture {
    pub fn member_binding(&self, member: ClusterMember) -> MemberDcsBinding {
        MemberDcsBinding {
            member,
            dcs_service: self.dcs_layout.service_for(member),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresRoleMapping {
    pub replicator: RoleName,
    pub rewinder: RoleName,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RoleName(String);

impl RoleName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreeNodeDcsLayout {
    SharedSingle,
    ColocatedThreeMember,
}

impl ThreeNodeDcsLayout {
    pub fn dcs_services(self) -> Vec<DcsService> {
        match self {
            Self::SharedSingle => vec![DcsService::SharedEtcd],
            Self::ColocatedThreeMember => DCS_MEMBERS.into_iter().map(DcsService::Member).collect(),
        }
    }

    pub fn service_for(self, member: ClusterMember) -> DcsService {
        match self {
            Self::SharedSingle => DcsService::SharedEtcd,
            Self::ColocatedThreeMember => DcsService::Member(member.local_dcs_member()),
        }
    }

    pub fn quorum_majority_services(self) -> Vec<DcsService> {
        match self {
            Self::SharedSingle => vec![DcsService::SharedEtcd],
            Self::ColocatedThreeMember => [DcsMember::EtcdA, DcsMember::EtcdB]
                .into_iter()
                .map(DcsService::Member)
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemberDcsBinding {
    pub member: ClusterMember,
    pub dcs_service: DcsService,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureMaterialization {
    pub shared_root: PathBuf,
    pub compose_variant: ComposeVariant,
    pub copies: Vec<SharedFixtureEntry>,
    pub runtime_configs: Vec<MemberRuntimeConfigMaterialization>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SharedFixtureEntry {
    Directory {
        source_relative_path: PathBuf,
        target_relative_path: PathBuf,
    },
    File {
        source_relative_path: PathBuf,
        target_relative_path: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComposeVariant {
    SharedSingleDcs,
    ColocatedThreeMemberDcs,
}

impl ComposeVariant {
    pub fn relative_path(self) -> &'static str {
        match self {
            Self::SharedSingleDcs => "compose/three_node_shared_single.yml",
            Self::ColocatedThreeMemberDcs => "compose/three_node_three_etcd.yml",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeRuntimeTemplate {
    pub binding: MemberDcsBinding,
    pub postgres_roles: PostgresRoleMapping,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberRuntimeConfigMaterialization {
    pub member: ClusterMember,
    pub template: NodeRuntimeTemplate,
}

pub fn resolve_given(repo_root: &Path, given: HaGivenId) -> Result<HaGivenDefinition> {
    let givens_root = repo_root.join("tests/ha/givens");
    let shared_root = givens_root.join("three_node_shared");
    let topology = three_node_topology(given);
    let materialization = FixtureMaterialization {
        shared_root,
        compose_variant: compose_variant(topology.dcs_layout),
        copies: vec![
            SharedFixtureEntry::Directory {
                source_relative_path: PathBuf::from("configs/tls"),
                target_relative_path: PathBuf::from("configs/tls"),
            },
            SharedFixtureEntry::Directory {
                source_relative_path: PathBuf::from("secrets"),
                target_relative_path: PathBuf::from("secrets"),
            },
            SharedFixtureEntry::File {
                source_relative_path: PathBuf::from("configs/pg_hba.conf"),
                target_relative_path: PathBuf::from("configs/pg_hba.conf"),
            },
            SharedFixtureEntry::File {
                source_relative_path: PathBuf::from("configs/pg_ident.conf"),
                target_relative_path: PathBuf::from("configs/pg_ident.conf"),
            },
        ],
        runtime_configs: three_node_runtime_configs(topology.clone()),
    };
    Ok(HaGivenDefinition {
        id: given,
        topology: HaTopologyFixture::ThreeNode(topology),
        materialization,
    })
}

fn three_node_topology(given: HaGivenId) -> ThreeNodeTopologyFixture {
    match given {
        HaGivenId::Plain => ThreeNodeTopologyFixture {
            postgres_roles: PostgresRoleMapping {
                replicator: RoleName::new("replicator"),
                rewinder: RoleName::new("rewinder"),
            },
            dcs_layout: ThreeNodeDcsLayout::SharedSingle,
        },
        HaGivenId::CustomRoles => ThreeNodeTopologyFixture {
            postgres_roles: PostgresRoleMapping {
                replicator: RoleName::new("mirrorbot"),
                rewinder: RoleName::new("rewindbot"),
            },
            dcs_layout: ThreeNodeDcsLayout::SharedSingle,
        },
        HaGivenId::ThreeEtcd => ThreeNodeTopologyFixture {
            postgres_roles: PostgresRoleMapping {
                replicator: RoleName::new("replicator"),
                rewinder: RoleName::new("rewinder"),
            },
            dcs_layout: ThreeNodeDcsLayout::ColocatedThreeMember,
        },
    }
}

fn compose_variant(layout: ThreeNodeDcsLayout) -> ComposeVariant {
    match layout {
        ThreeNodeDcsLayout::SharedSingle => ComposeVariant::SharedSingleDcs,
        ThreeNodeDcsLayout::ColocatedThreeMember => ComposeVariant::ColocatedThreeMemberDcs,
    }
}

fn three_node_runtime_configs(
    topology: ThreeNodeTopologyFixture,
) -> Vec<MemberRuntimeConfigMaterialization> {
    ClusterMember::ALL
        .into_iter()
        .map(|member| MemberRuntimeConfigMaterialization {
            member,
            template: NodeRuntimeTemplate {
                binding: topology.member_binding(member),
                postgres_roles: topology.postgres_roles.clone(),
            },
        })
        .collect()
}
