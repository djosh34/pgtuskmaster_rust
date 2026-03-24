use std::{net::IpAddr, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PgEndpoint {
    Tcp { host: String, port: u16 },
    UnixSocket { socket_dir: PathBuf, port: u16 },
}

impl PgEndpoint {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgRoute {
    pub endpoint: PgEndpoint,
    pub hostaddr: Option<IpAddr>,
}

impl PgRoute {
    pub fn new(endpoint: PgEndpoint, hostaddr: Option<IpAddr>) -> Self {
        Self { endpoint, hostaddr }
    }

    pub fn tcp(host: String, port: u16) -> Result<Self, String> {
        Self::tcp_hostaddr(host, port, None)
    }

    pub fn tcp_hostaddr(host: String, port: u16, hostaddr: Option<IpAddr>) -> Result<Self, String> {
        Ok(Self::new(PgEndpoint::tcp(host, port)?, hostaddr))
    }

    pub fn unix_socket(socket_dir: PathBuf, port: u16) -> Result<Self, String> {
        Ok(Self::new(PgEndpoint::unix_socket(socket_dir, port)?, None))
    }

    pub fn endpoint(&self) -> &PgEndpoint {
        &self.endpoint
    }

    pub fn host(&self) -> &str {
        self.endpoint.host()
    }

    pub fn socket_dir(&self) -> Option<&PathBuf> {
        self.endpoint.socket_dir()
    }

    pub fn port(&self) -> u16 {
        self.endpoint.port()
    }

    pub fn hostaddr(&self) -> Option<IpAddr> {
        self.hostaddr
    }
}
