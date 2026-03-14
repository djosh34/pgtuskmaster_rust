use std::fmt;

use pgtuskmaster_rust::state::MemberId;

use crate::support::{
    error::{HarnessError, Result},
    faults::OBSERVER_SERVICE_NAME,
};

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

    pub fn observer_config_path(self) -> &'static str {
        match self {
            Self::NodeA => "/etc/pgtuskmaster/observer/node-a.toml",
            Self::NodeB => "/etc/pgtuskmaster/observer/node-b.toml",
            Self::NodeC => "/etc/pgtuskmaster/observer/node-c.toml",
        }
    }

    pub fn observer_config_relative_path(self) -> &'static str {
        match self {
            Self::NodeA => "configs/observer/node-a.toml",
            Self::NodeB => "configs/observer/node-b.toml",
            Self::NodeC => "configs/observer/node-c.toml",
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

    pub fn as_str(self) -> &'static str {
        self.service_name()
    }

    pub fn local_dcs_member(self) -> DcsMember {
        match self {
            Self::NodeA => DcsMember::EtcdA,
            Self::NodeB => DcsMember::EtcdB,
            Self::NodeC => DcsMember::EtcdC,
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
pub enum DcsMember {
    EtcdA,
    EtcdB,
    EtcdC,
}

impl DcsMember {
    pub const ALL: [Self; 3] = [Self::EtcdA, Self::EtcdB, Self::EtcdC];

    pub fn service_name(self) -> &'static str {
        match self {
            Self::EtcdA => "etcd-a",
            Self::EtcdB => "etcd-b",
            Self::EtcdC => "etcd-c",
        }
    }

    pub fn client_url(self) -> &'static str {
        match self {
            Self::EtcdA => "http://etcd-a:2379",
            Self::EtcdB => "http://etcd-b:2379",
            Self::EtcdC => "http://etcd-c:2379",
        }
    }

    pub fn peer_url(self) -> &'static str {
        match self {
            Self::EtcdA => "http://etcd-a:2380",
            Self::EtcdB => "http://etcd-b:2380",
            Self::EtcdC => "http://etcd-c:2380",
        }
    }

    pub fn volume_name(self) -> &'static str {
        match self {
            Self::EtcdA => "etcd-a-data",
            Self::EtcdB => "etcd-b-data",
            Self::EtcdC => "etcd-c-data",
        }
    }
}

impl fmt::Display for DcsMember {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.service_name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DcsService {
    SharedEtcd,
    Member(DcsMember),
}

impl DcsService {
    pub fn service_name(self) -> &'static str {
        match self {
            Self::SharedEtcd => "etcd",
            Self::Member(member) => member.service_name(),
        }
    }

    pub fn client_url(self) -> &'static str {
        match self {
            Self::SharedEtcd => "http://etcd:2379",
            Self::Member(member) => member.client_url(),
        }
    }
}

impl fmt::Display for DcsService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.service_name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComposeService {
    Member(ClusterMember),
    Observer,
    Dcs(DcsService),
}

impl ComposeService {
    pub fn service_name(self) -> &'static str {
        match self {
            Self::Member(member) => member.service_name(),
            Self::Observer => OBSERVER_SERVICE_NAME,
            Self::Dcs(service) => service.service_name(),
        }
    }
}

impl fmt::Display for ComposeService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.service_name())
    }
}

impl From<ClusterMember> for ComposeService {
    fn from(value: ClusterMember) -> Self {
        Self::Member(value)
    }
}

impl From<DcsService> for ComposeService {
    fn from(value: DcsService) -> Self {
        Self::Dcs(value)
    }
}
