use std::fmt;

use pgtuskmaster_rust::state::MemberId;

use crate::support::{
    error::{HarnessError, Result},
    faults::{ETCD_SERVICE_NAME, OBSERVER_SERVICE_NAME},
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
pub enum SupportService {
    Observer,
    Etcd,
}

impl SupportService {
    pub fn service_name(self) -> &'static str {
        match self {
            Self::Observer => OBSERVER_SERVICE_NAME,
            Self::Etcd => ETCD_SERVICE_NAME,
        }
    }

}

impl fmt::Display for SupportService {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.service_name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComposeService {
    Member(ClusterMember),
    Support(SupportService),
}

impl ComposeService {
    pub fn service_name(self) -> &'static str {
        match self {
            Self::Member(member) => member.service_name(),
            Self::Support(service) => service.service_name(),
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

impl From<SupportService> for ComposeService {
    fn from(value: SupportService) -> Self {
        Self::Support(value)
    }
}
