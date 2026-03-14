use std::path::{Path, PathBuf};

use crate::support::{
    error::{HarnessError, Result},
    topology::ClusterMember,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HaGivenId {
    ThreeNodePlain,
    ThreeNodeCustomRoles,
}

impl HaGivenId {
    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "three_node_plain" => Ok(Self::ThreeNodePlain),
            "three_node_custom_roles" => Ok(Self::ThreeNodeCustomRoles),
            _ => Err(HarnessError::message(format!(
                "unsupported HA given `{raw}`"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ThreeNodePlain => "three_node_plain",
            Self::ThreeNodeCustomRoles => "three_node_custom_roles",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HaGivenDefinition {
    pub id: HaGivenId,
    pub topology: HaTopologyFixture,
    pub materialization: FixtureMaterialization,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HaTopologyFixture {
    ThreeNode(ThreeNodeTopologyFixture),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThreeNodeTopologyFixture {
    pub postgres_roles: PostgresRoleMapping,
    pub observer_net_admin: ObserverNetAdmin,
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
pub enum ObserverNetAdmin {
    Enabled,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureMaterialization {
    pub shared_root: PathBuf,
    pub copies: Vec<SharedFixtureEntry>,
    pub renders: Vec<RenderedFixtureFile>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedFixtureFile {
    pub target: FixtureRenderTarget,
    pub template: FixtureTemplate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FixtureRenderTarget {
    ComposeFile,
    MemberRuntimeConfig(ClusterMember),
    ObserverConfig(ClusterMember),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FixtureTemplate {
    Compose(ComposeTemplate),
    Runtime(NodeRuntimeTemplate),
    Observer(ObserverTemplate),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComposeTemplate {
    pub observer_net_admin: ObserverNetAdmin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeRuntimeTemplate {
    pub member: ClusterMember,
    pub postgres_roles: PostgresRoleMapping,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObserverTemplate {
    pub member: ClusterMember,
    pub postgres_roles: PostgresRoleMapping,
}

pub fn resolve_given(repo_root: &Path, given: HaGivenId) -> Result<HaGivenDefinition> {
    let givens_root = repo_root.join("tests/ha/givens");
    let shared_root = givens_root.join("three_node_shared");
    let topology = three_node_topology(given);
    let materialization = FixtureMaterialization {
        shared_root,
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
        renders: three_node_render_plan(topology.clone()),
    };
    Ok(HaGivenDefinition {
        id: given,
        topology: HaTopologyFixture::ThreeNode(topology),
        materialization,
    })
}

fn three_node_topology(given: HaGivenId) -> ThreeNodeTopologyFixture {
    match given {
        HaGivenId::ThreeNodePlain => ThreeNodeTopologyFixture {
            postgres_roles: PostgresRoleMapping {
                replicator: RoleName::new("replicator"),
                rewinder: RoleName::new("rewinder"),
            },
            observer_net_admin: ObserverNetAdmin::Enabled,
        },
        HaGivenId::ThreeNodeCustomRoles => ThreeNodeTopologyFixture {
            postgres_roles: PostgresRoleMapping {
                replicator: RoleName::new("mirrorbot"),
                rewinder: RoleName::new("rewindbot"),
            },
            observer_net_admin: ObserverNetAdmin::Disabled,
        },
    }
}

fn three_node_render_plan(topology: ThreeNodeTopologyFixture) -> Vec<RenderedFixtureFile> {
    std::iter::once(RenderedFixtureFile {
        target: FixtureRenderTarget::ComposeFile,
        template: FixtureTemplate::Compose(ComposeTemplate {
            observer_net_admin: topology.observer_net_admin,
        }),
    })
    .chain(ClusterMember::ALL.into_iter().flat_map(|member| {
        [
            RenderedFixtureFile {
                target: FixtureRenderTarget::MemberRuntimeConfig(member),
                template: FixtureTemplate::Runtime(NodeRuntimeTemplate {
                    member,
                    postgres_roles: topology.postgres_roles.clone(),
                }),
            },
            RenderedFixtureFile {
                target: FixtureRenderTarget::ObserverConfig(member),
                template: FixtureTemplate::Observer(ObserverTemplate {
                    member,
                    postgres_roles: topology.postgres_roles.clone(),
                }),
            },
        ]
    }))
    .collect()
}
