use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PgEndpoint {
    Tcp { host: String, port: u16 },
    UnixSocket { socket_dir: PathBuf, port: u16 },
}

impl PgEndpoint {
    pub fn new(host: String, port: u16) -> Result<Self, String> {
        Self::tcp(host, port)
    }

    pub fn tcp(host: String, port: u16) -> Result<Self, String> {
        let trimmed = host.trim().to_string();
        if trimmed.is_empty() {
            return Err("postgres tcp target host must not be empty".to_string());
        }
        if port == 0 {
            return Err("postgres tcp target port must not be zero".to_string());
        }
        Ok(Self::Tcp {
            host: trimmed,
            port,
        })
    }

    pub fn unix_socket(socket_dir: PathBuf, port: u16) -> Result<Self, String> {
        if port == 0 {
            return Err("postgres unix target port must not be zero".to_string());
        }
        Ok(Self::UnixSocket { socket_dir, port })
    }

    pub fn host(&self) -> &str {
        match self {
            Self::Tcp { host, .. } => host.as_str(),
            Self::UnixSocket { .. } => "",
        }
    }

    pub fn socket_dir(&self) -> Option<&PathBuf> {
        match self {
            Self::Tcp { .. } => None,
            Self::UnixSocket { socket_dir, .. } => Some(socket_dir),
        }
    }

    pub fn port(&self) -> u16 {
        match self {
            Self::Tcp { port, .. } | Self::UnixSocket { port, .. } => *port,
        }
    }
}

pub type PgTcpTarget = PgEndpoint;
pub type PgUnixTarget = PgEndpoint;
pub type PgConnectTarget = PgEndpoint;
