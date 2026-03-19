use std::{fmt, net::SocketAddr, str::FromStr};

use reqwest::Url;
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DcsEndpointScheme {
    Http,
    Https,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DcsEndpoint {
    scheme: DcsEndpointScheme,
    host: String,
    port: u16,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DcsEndpointError {
    #[error("endpoint is not a valid URL: {0}")]
    InvalidUrl(String),
    #[error("endpoint scheme must be `http` or `https`")]
    UnsupportedScheme,
    #[error("endpoint must not include username or password")]
    UserInfoNotSupported,
    #[error("endpoint must include a host")]
    MissingHost,
    #[error("endpoint must include an explicit port")]
    MissingPort,
    #[error("endpoint path must be empty")]
    PathNotSupported,
    #[error("endpoint query string is not supported")]
    QueryNotSupported,
    #[error("endpoint fragment is not supported")]
    FragmentNotSupported,
    #[error("endpoint host must be a loopback IP address to derive a proxy socket")]
    LoopbackSocketRequired,
}

impl DcsEndpoint {
    pub fn parse(raw: &str) -> Result<Self, DcsEndpointError> {
        let url = Url::parse(raw).map_err(|err| DcsEndpointError::InvalidUrl(err.to_string()))?;
        Self::from_url(&url)
    }

    pub fn from_socket_addr(socket_addr: SocketAddr) -> Self {
        Self {
            scheme: DcsEndpointScheme::Http,
            host: socket_addr.ip().to_string(),
            port: socket_addr.port(),
        }
    }

    pub fn scheme(&self) -> DcsEndpointScheme {
        self.scheme
    }

    pub fn host(&self) -> &str {
        self.host.as_str()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    fn from_url(url: &Url) -> Result<Self, DcsEndpointError> {
        let scheme = match url.scheme() {
            "http" => DcsEndpointScheme::Http,
            "https" => DcsEndpointScheme::Https,
            _ => return Err(DcsEndpointError::UnsupportedScheme),
        };
        if !url.username().is_empty() || url.password().is_some() {
            return Err(DcsEndpointError::UserInfoNotSupported);
        }
        if url.host_str().is_none() {
            return Err(DcsEndpointError::MissingHost);
        }
        if url.port().is_none() {
            return Err(DcsEndpointError::MissingPort);
        }
        if url.path() != "/" && !url.path().is_empty() {
            return Err(DcsEndpointError::PathNotSupported);
        }
        if url.query().is_some() {
            return Err(DcsEndpointError::QueryNotSupported);
        }
        if url.fragment().is_some() {
            return Err(DcsEndpointError::FragmentNotSupported);
        }
        let host = url
            .host_str()
            .ok_or(DcsEndpointError::MissingHost)?
            .to_string();
        let port = url.port().ok_or(DcsEndpointError::MissingPort)?;

        Ok(Self { scheme, host, port })
    }

    pub fn socket_addr(&self) -> Result<SocketAddr, DcsEndpointError> {
        let socket = SocketAddr::from_str(&format!("{}:{}", self.host, self.port))
            .map_err(|_| DcsEndpointError::LoopbackSocketRequired)?;
        if !socket.ip().is_loopback() {
            return Err(DcsEndpointError::LoopbackSocketRequired);
        }
        Ok(socket)
    }
}

impl fmt::Display for DcsEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scheme = match self.scheme {
            DcsEndpointScheme::Http => "http",
            DcsEndpointScheme::Https => "https",
        };
        write!(f, "{scheme}://")?;
        if self.host.contains(':') && !(self.host.starts_with('[') && self.host.ends_with(']')) {
            write!(f, "[{}]", self.host)?;
        } else {
            f.write_str(self.host.as_str())?;
        }
        write!(f, ":{}", self.port)
    }
}

impl Serialize for DcsEndpoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for DcsEndpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw.as_str()).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::DcsEndpoint;

    #[test]
    fn display_formats_ipv4_endpoint() -> Result<(), String> {
        let endpoint =
            DcsEndpoint::parse("http://127.0.0.1:2379").map_err(|err| err.to_string())?;

        if endpoint.to_string() != "http://127.0.0.1:2379" {
            return Err(format!("unexpected endpoint display: {}", endpoint));
        }

        Ok(())
    }

    #[test]
    fn display_formats_ipv6_endpoint() -> Result<(), String> {
        let endpoint = DcsEndpoint::parse("https://[::1]:2379").map_err(|err| err.to_string())?;

        if endpoint.to_string() != "https://[::1]:2379" {
            return Err(format!("unexpected endpoint display: {}", endpoint));
        }

        Ok(())
    }

    #[test]
    fn serialize_reuses_display_boundary() -> Result<(), String> {
        let endpoint = DcsEndpoint::parse("https://[::1]:2379").map_err(|err| err.to_string())?;
        let rendered = serde_json::to_string(&endpoint).map_err(|err| err.to_string())?;

        if rendered != "\"https://[::1]:2379\"" {
            return Err(format!("unexpected serialized endpoint: {rendered}"));
        }

        Ok(())
    }
}
