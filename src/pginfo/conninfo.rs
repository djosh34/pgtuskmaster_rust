use std::{fmt, net::IpAddr, path::PathBuf, str::FromStr};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::state::{PgEndpoint, PgRoute};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PgSslMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

impl PgSslMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Disable => "disable",
            Self::Allow => "allow",
            Self::Prefer => "prefer",
            Self::Require => "require",
            Self::VerifyCa => "verify-ca",
            Self::VerifyFull => "verify-full",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "disable" => Some(Self::Disable),
            "allow" => Some(Self::Allow),
            "prefer" => Some(Self::Prefer),
            "require" => Some(Self::Require),
            "verify-ca" | "verify_ca" => Some(Self::VerifyCa),
            "verify-full" | "verify_full" => Some(Self::VerifyFull),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for PgSslMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(raw.as_str())
            .ok_or_else(|| de::Error::custom(format!("unsupported sslmode `{raw}`")))
    }
}

impl Serialize for PgSslMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgConnInfo {
    pub route: PgRoute,
    pub user: String,
    pub dbname: String,
    pub application_name: Option<String>,
    pub connect_timeout_s: Option<u32>,
    pub options: Option<String>,
    pub tls: PgClientTls,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PgClientTls {
    pub mode: PgSslMode,
    pub root_cert: Option<PathBuf>,
    pub client_cert: Option<PathBuf>,
    pub client_key: Option<PathBuf>,
}

impl fmt::Display for PgConnInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (host, port) = match self.route.endpoint() {
            PgEndpoint::Tcp { host, port } => (host.clone(), *port),
            PgEndpoint::UnixSocket { socket_dir, port } => {
                (socket_dir.display().to_string(), *port)
            }
        };

        write!(f, "host={}", render_conninfo_value(host.as_str()))?;
        if let Some(value) = self.route.hostaddr() {
            write!(f, " hostaddr={value}")?;
        }
        write!(
            f,
            " port={port} user={} dbname={}",
            render_conninfo_value(self.user.as_str()),
            render_conninfo_value(self.dbname.as_str()),
        )?;
        if let Some(value) = &self.application_name {
            write!(
                f,
                " application_name={}",
                render_conninfo_value(value.as_str())
            )?;
        }
        if let Some(value) = self.connect_timeout_s {
            write!(f, " connect_timeout={value}")?;
        }
        write!(f, " sslmode={}", self.tls.mode.as_str())?;
        if let Some(value) = &self.tls.root_cert {
            write!(
                f,
                " sslrootcert={}",
                render_conninfo_value(value.display().to_string().as_str())
            )?;
        }
        if let Some(value) = &self.tls.client_cert {
            write!(
                f,
                " sslcert={}",
                render_conninfo_value(value.display().to_string().as_str())
            )?;
        }
        if let Some(value) = &self.tls.client_key {
            write!(
                f,
                " sslkey={}",
                render_conninfo_value(value.display().to_string().as_str())
            )?;
        }
        if let Some(value) = &self.options {
            write!(f, " options={}", render_conninfo_value(value.as_str()))?;
        }

        Ok(())
    }
}

impl FromStr for PgConnInfo {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let entries = parse_conninfo_entries(input)?;
        let host = entries
            .get("host")
            .cloned()
            .ok_or_else(|| "missing required conninfo key `host`".to_string())?;
        let port = entries
            .get("port")
            .ok_or_else(|| "missing required conninfo key `port`".to_string())?
            .parse::<u16>()
            .map_err(|err| format!("invalid conninfo port: {err}"))?;
        let mode = entries
            .get("sslmode")
            .and_then(|value| PgSslMode::parse(value.as_str()))
            .ok_or_else(|| "missing or invalid conninfo key `sslmode`".to_string())?;
        let endpoint = if host.starts_with('/') {
            PgEndpoint::UnixSocket {
                socket_dir: PathBuf::from(host),
                port,
            }
        } else {
            PgEndpoint::tcp(host, port)?
        };
        Ok(Self {
            route: PgRoute::new(
                endpoint,
                entries
                    .get("hostaddr")
                    .map(|value| {
                        value
                            .parse::<IpAddr>()
                            .map_err(|err| format!("invalid conninfo hostaddr: {err}"))
                    })
                    .transpose()?,
            ),
            user: entries
                .get("user")
                .cloned()
                .ok_or_else(|| "missing required conninfo key `user`".to_string())?,
            dbname: entries
                .get("dbname")
                .cloned()
                .ok_or_else(|| "missing required conninfo key `dbname`".to_string())?,
            application_name: entries.get("application_name").cloned(),
            connect_timeout_s: entries
                .get("connect_timeout")
                .map(|value| {
                    value
                        .parse::<u32>()
                        .map_err(|err| format!("invalid conninfo connect_timeout: {err}"))
                })
                .transpose()?,
            options: entries.get("options").cloned(),
            tls: PgClientTls {
                mode,
                root_cert: entries.get("sslrootcert").map(PathBuf::from),
                client_cert: entries.get("sslcert").map(PathBuf::from),
                client_key: entries.get("sslkey").map(PathBuf::from),
            },
        })
    }
}

pub(crate) fn parse_pg_conninfo(input: &str) -> Result<PgConnInfo, String> {
    input.parse()
}

pub fn conninfo_entries(input: &str) -> Result<std::collections::BTreeMap<String, String>, String> {
    parse_conninfo_entries(input)
}

pub fn conninfo_value(input: &str, key: &str) -> Result<Option<String>, String> {
    parse_conninfo_entries(input).map(|entries| entries.get(key).cloned())
}

pub fn render_conninfo_value(value: &str) -> String {
    if value.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_whitespace() || ch == '\'' || ch == '\\')
    {
        let escaped = value
            .chars()
            .map(|ch| match ch {
                '\'' => "\\'".to_string(),
                '\\' => "\\\\".to_string(),
                other => other.to_string(),
            })
            .collect::<String>();
        format!("'{escaped}'")
    } else {
        value.to_string()
    }
}

fn parse_conninfo_entries(
    input: &str,
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let chars = input.chars().collect::<Vec<_>>();
    let mut entries = std::collections::BTreeMap::new();
    let mut index = 0usize;

    while index < chars.len() {
        while index < chars.len() && chars[index].is_whitespace() {
            index = index.saturating_add(1);
        }
        if index >= chars.len() {
            break;
        }

        let key_start = index;
        while index < chars.len() && chars[index] != '=' && !chars[index].is_whitespace() {
            index = index.saturating_add(1);
        }
        if index == key_start || index >= chars.len() || chars[index] != '=' {
            return Err("invalid conninfo key/value pair".to_string());
        }
        let key = chars[key_start..index].iter().collect::<String>();
        index = index.saturating_add(1);
        if index >= chars.len() {
            return Err(format!("missing value for conninfo key `{key}`"));
        }

        let value = if chars[index] == '\'' {
            index = index.saturating_add(1);
            let mut value = String::new();
            let mut closed = false;
            while index < chars.len() {
                match chars[index] {
                    '\'' => {
                        index = index.saturating_add(1);
                        closed = true;
                        break;
                    }
                    '\\' => {
                        index = index.saturating_add(1);
                        let escaped = chars.get(index).ok_or_else(|| {
                            format!("unterminated escape sequence for conninfo key `{key}`")
                        })?;
                        value.push(*escaped);
                        index = index.saturating_add(1);
                    }
                    value_char => {
                        value.push(value_char);
                        index = index.saturating_add(1);
                    }
                }
            }
            if !closed {
                return Err(format!(
                    "unterminated quoted value for conninfo key `{key}`"
                ));
            }
            value
        } else {
            let value_start = index;
            while index < chars.len() && !chars[index].is_whitespace() {
                index = index.saturating_add(1);
            }
            chars[value_start..index].iter().collect::<String>()
        };

        entries.insert(key, value);
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        path::PathBuf,
    };

    use super::{
        conninfo_entries, conninfo_value, parse_pg_conninfo, render_conninfo_value, PgClientTls,
        PgConnInfo, PgSslMode,
    };
    use crate::state::PgRoute;

    fn sample_conninfo() -> Result<PgConnInfo, String> {
        Ok(PgConnInfo {
            route: PgRoute::tcp("127.0.0.1".to_string(), 5432)?,
            user: "postgres".to_string(),
            dbname: "postgres".to_string(),
            application_name: Some("ha worker".to_string()),
            connect_timeout_s: Some(5),
            options: Some("-c search_path=public".to_string()),
            tls: PgClientTls {
                mode: PgSslMode::Require,
                root_cert: Some(PathBuf::from("/etc/pgtm/ca bundle.pem")),
                client_cert: Some(PathBuf::from("/etc/pgtm/client cert.pem")),
                client_key: Some(PathBuf::from("/etc/pgtm/client key.pem")),
            },
        })
    }

    #[test]
    fn render_emits_canonical_key_order() -> Result<(), String> {
        let rendered = sample_conninfo()?.to_string();
        assert_eq!(
            rendered,
            "host=127.0.0.1 port=5432 user=postgres dbname='postgres' application_name='ha worker' connect_timeout=5 sslmode=require sslrootcert='/etc/pgtm/ca bundle.pem' sslcert='/etc/pgtm/client cert.pem' sslkey='/etc/pgtm/client key.pem' options='-c search_path=public'"
                .replace("dbname='postgres'", "dbname=postgres")
        );
        Ok(())
    }

    #[test]
    fn parse_accepts_rendered_conninfo_with_extra_keys() -> Result<(), String> {
        let rendered = format!(
            "{} passfile='/var/lib/postgresql/data/pgtm.standby.passfile'",
            sample_conninfo()?
        );

        assert_eq!(parse_pg_conninfo(rendered.as_str()), Ok(sample_conninfo()?));
        Ok(())
    }

    #[test]
    fn parse_and_render_round_trip_hostaddr() -> Result<(), String> {
        let conninfo = PgConnInfo {
            route: PgRoute::tcp_hostaddr(
                "127.0.0.1".to_string(),
                5432,
                Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            )?,
            ..sample_conninfo()?
        };

        let rendered = conninfo.to_string();

        assert!(rendered.contains("hostaddr=127.0.0.1"));
        assert_eq!(parse_pg_conninfo(rendered.as_str()), Ok(conninfo));
        Ok(())
    }

    #[test]
    fn conninfo_value_reads_quoted_late_field() -> Result<(), String> {
        let rendered = concat!(
            "host=127.0.0.1 ",
            "port=5432 ",
            "user=postgres ",
            "dbname=postgres ",
            "sslmode=verify-full ",
            "sslrootcert='/etc/pgtm/ca bundle.pem'"
        );

        assert_eq!(
            conninfo_value(rendered, "sslrootcert")?,
            Some("/etc/pgtm/ca bundle.pem".to_string())
        );
        Ok(())
    }

    #[test]
    fn conninfo_entries_parse_multiple_quoted_tls_fields() -> Result<(), String> {
        let entries = conninfo_entries(
            "host=node-a sslrootcert='/tmp/ca bundle.pem' sslcert='/tmp/client cert.pem'",
        )?;

        assert_eq!(entries.get("host"), Some(&"node-a".to_string()));
        assert_eq!(
            entries.get("sslrootcert"),
            Some(&"/tmp/ca bundle.pem".to_string())
        );
        assert_eq!(
            entries.get("sslcert"),
            Some(&"/tmp/client cert.pem".to_string())
        );
        Ok(())
    }

    #[test]
    fn render_conninfo_value_quotes_whitespace_and_backslashes() {
        assert_eq!(
            render_conninfo_value("/etc/pgtm/ca bundle\\with'space.pem"),
            r"'/etc/pgtm/ca bundle\\with\'space.pem'"
        );
    }

    #[test]
    fn pg_ssl_mode_parse_accepts_snake_case_aliases() {
        assert_eq!(PgSslMode::parse("verify_ca"), Some(PgSslMode::VerifyCa));
        assert_eq!(PgSslMode::parse("verify_full"), Some(PgSslMode::VerifyFull));
    }
}
