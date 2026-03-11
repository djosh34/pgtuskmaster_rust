use std::path::PathBuf;

use serde::{de, Deserialize, Deserializer};

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
            "verify-ca" => Some(Self::VerifyCa),
            "verify-full" => Some(Self::VerifyFull),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgConnInfo {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) user: String,
    pub(crate) dbname: String,
    pub(crate) application_name: Option<String>,
    pub(crate) connect_timeout_s: Option<u32>,
    pub(crate) ssl_mode: PgSslMode,
    pub(crate) ssl_root_cert: Option<PathBuf>,
    pub(crate) options: Option<String>,
}

pub(crate) fn render_pg_conninfo(info: &PgConnInfo) -> String {
    let mut pairs = vec![
        ("host".to_string(), info.host.clone()),
        ("port".to_string(), info.port.to_string()),
        ("user".to_string(), info.user.clone()),
        ("dbname".to_string(), info.dbname.clone()),
    ];

    if let Some(value) = &info.application_name {
        pairs.push(("application_name".to_string(), value.clone()));
    }
    if let Some(value) = info.connect_timeout_s {
        pairs.push(("connect_timeout".to_string(), value.to_string()));
    }
    pairs.push(("sslmode".to_string(), info.ssl_mode.as_str().to_string()));
    if let Some(value) = &info.ssl_root_cert {
        pairs.push(("sslrootcert".to_string(), value.display().to_string()));
    }
    if let Some(value) = &info.options {
        pairs.push(("options".to_string(), value.clone()));
    }

    pairs
        .into_iter()
        .map(|(key, value)| format!("{key}={}", render_value(&value)))
        .collect::<Vec<String>>()
        .join(" ")
}

fn render_value(value: &str) -> String {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{render_pg_conninfo, PgConnInfo, PgSslMode};

    fn sample_conninfo() -> PgConnInfo {
        PgConnInfo {
            host: "127.0.0.1".to_string(),
            port: 5432,
            user: "postgres".to_string(),
            dbname: "postgres".to_string(),
            application_name: Some("ha worker".to_string()),
            connect_timeout_s: Some(5),
            ssl_mode: PgSslMode::Require,
            ssl_root_cert: Some(PathBuf::from("/etc/pgtm/ca bundle.pem")),
            options: Some("-c search_path=public".to_string()),
        }
    }

    #[test]
    fn render_emits_canonical_key_order() {
        let rendered = render_pg_conninfo(&sample_conninfo());
        assert_eq!(
            rendered,
            "host=127.0.0.1 port=5432 user=postgres dbname='postgres' application_name='ha worker' connect_timeout=5 sslmode=require sslrootcert='/etc/pgtm/ca bundle.pem' options='-c search_path=public'"
                .replace("dbname='postgres'", "dbname=postgres")
        );
    }
}
