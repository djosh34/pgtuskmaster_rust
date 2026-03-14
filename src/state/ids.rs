use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NonEmptyStringError {
    #[error("{label} must not be empty")]
    Empty { label: &'static str },
}

fn require_non_empty(label: &'static str, value: &str) -> Result<(), NonEmptyStringError> {
    if value.trim().is_empty() {
        return Err(NonEmptyStringError::Empty { label });
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct MemberId(pub(crate) String);

impl MemberId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for MemberId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl AsRef<str> for MemberId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<String> for MemberId {
    type Error = NonEmptyStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        require_non_empty("member_id", value.as_str())?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for MemberId {
    type Error = NonEmptyStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        require_non_empty("member_id", value)?;
        Ok(Self(value.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct ClusterName(pub(crate) String);

impl ClusterName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for ClusterName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl AsRef<str> for ClusterName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<String> for ClusterName {
    type Error = NonEmptyStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        require_non_empty("cluster_name", value.as_str())?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for ClusterName {
    type Error = NonEmptyStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        require_non_empty("cluster_name", value)?;
        Ok(Self(value.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct ScopeName(pub(crate) String);

impl ScopeName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for ScopeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl AsRef<str> for ScopeName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<String> for ScopeName {
    type Error = NonEmptyStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        require_non_empty("scope", value.as_str())?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for ScopeName {
    type Error = NonEmptyStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        require_non_empty("scope", value)?;
        Ok(Self(value.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub cluster_name: ClusterName,
    pub scope: ScopeName,
    pub member_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct SwitchoverRequestId(pub(crate) String);

impl SwitchoverRequestId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for SwitchoverRequestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl AsRef<str> for SwitchoverRequestId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<String> for SwitchoverRequestId {
    type Error = NonEmptyStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        require_non_empty("switchover_request_id", value.as_str())?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for SwitchoverRequestId {
    type Error = NonEmptyStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        require_non_empty("switchover_request_id", value)?;
        Ok(Self(value.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct JobId(pub(crate) String);

impl JobId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl AsRef<str> for JobId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<String> for JobId {
    type Error = NonEmptyStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        require_non_empty("job_id", value.as_str())?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for JobId {
    type Error = NonEmptyStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        require_non_empty("job_id", value)?;
        Ok(Self(value.to_string()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WalLsn(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimelineId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SystemIdentifier(pub u64);
