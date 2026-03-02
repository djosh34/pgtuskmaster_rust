use super::errors::WorkerError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnixMillis(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Versioned<T> {
    pub version: Version,
    pub updated_at: UnixMillis,
    pub value: T,
}

impl<T> Versioned<T> {
    pub fn new(version: Version, updated_at: UnixMillis, value: T) -> Self {
        Self {
            version,
            updated_at,
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkerStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Faulted(WorkerError),
}
