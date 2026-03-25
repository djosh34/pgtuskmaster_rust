use std::fmt;

use pgtuskmaster_rust::state::MemberId;

use crate::support::error::{HarnessError, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClusterMember {
    NodeA,
    NodeB,
    NodeC,
}

impl ClusterMember {
    pub const ALL: [Self; 3] = [Self::NodeA, Self::NodeB, Self::NodeC];
    pub const SEED_PRIMARY: Self = Self::NodeB;

    pub fn service_name(self) -> &'static str {
        match self {
            Self::NodeA => "node-a",
            Self::NodeB => "node-b",
            Self::NodeC => "node-c",
        }
    }

    pub fn runtime_config_relative_path(self) -> &'static str {
        match self {
            Self::NodeA => "configs/node-a/runtime.toml",
            Self::NodeB => "configs/node-b/runtime.toml",
            Self::NodeC => "configs/node-c/runtime.toml",
        }
    }

    pub fn member_id(self) -> MemberId {
        MemberId(self.service_name().to_string())
    }

    pub fn local_dcs_service(self) -> DcsService {
        match self {
            Self::NodeA => DcsService::EtcdA,
            Self::NodeB => DcsService::EtcdB,
            Self::NodeC => DcsService::EtcdC,
        }
    }

    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "node-a" => Ok(Self::NodeA),
            "node-b" => Ok(Self::NodeB),
            "node-c" => Ok(Self::NodeC),
            _ => Err(HarnessError::message(format!(
                "unknown HA cluster member `{raw}`"
            ))),
        }
    }
}

impl fmt::Display for ClusterMember {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.service_name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DcsService {
    SharedEtcd,
    EtcdA,
    EtcdB,
    EtcdC,
}

impl DcsService {
    pub const COLOCATED_ALL: [Self; 3] = [Self::EtcdA, Self::EtcdB, Self::EtcdC];

    pub fn service_name(self) -> &'static str {
        match self {
            Self::SharedEtcd => "etcd",
            Self::EtcdA => "etcd-a",
            Self::EtcdB => "etcd-b",
            Self::EtcdC => "etcd-c",
        }
    }

    pub fn client_url(self) -> &'static str {
        match self {
            Self::SharedEtcd => "http://etcd:2379",
            Self::EtcdA => "http://etcd-a:2379",
            Self::EtcdB => "http://etcd-b:2379",
            Self::EtcdC => "http://etcd-c:2379",
        }
    }
}

impl fmt::Display for DcsService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.service_name())
    }
}
