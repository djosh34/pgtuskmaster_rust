use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgTcpTarget {
    host: String,
    port: u16,
}

impl PgTcpTarget {
    pub fn new(host: String, port: u16) -> Result<Self, String> {
        let trimmed = host.trim().to_string();
        if trimmed.is_empty() {
            return Err("postgres tcp target host must not be empty".to_string());
        }
        if port == 0 {
            return Err("postgres tcp target port must not be zero".to_string());
        }
        Ok(Self {
            host: trimmed,
            port,
        })
    }

    pub fn host(&self) -> &str {
        self.host.as_str()
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgUnixTarget {
    pub socket_dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PgConnectTarget {
    Tcp(PgTcpTarget),
    Unix(PgUnixTarget),
}
