use std::path::PathBuf;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::state::{PgConnectTarget, PgTcpTarget, PgUnixTarget};

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
    pub target: PgConnectTarget,
    pub user: String,
    pub dbname: String,
    pub application_name: Option<String>,
    pub connect_timeout_s: Option<u32>,
    pub ssl_mode: PgSslMode,
    pub ssl_root_cert: Option<PathBuf>,
    pub options: Option<String>,
}

pub(crate) fn parse_pg_conninfo(input: &str) -> Result<PgConnInfo, String> {
    let entries = parse_conninfo_entries(input)?;
    let get_required = |key: &str| {
        entries
            .get(key)
            .cloned()
            .ok_or_else(|| format!("missing required conninfo key `{key}`"))
    };

    let port = get_required("port")?
        .parse::<u16>()
        .map_err(|err| format!("invalid conninfo port: {err}"))?;
    let host = get_required("host")?;
    let ssl_mode_raw = get_required("sslmode")?;
    let ssl_mode = PgSslMode::parse(ssl_mode_raw.as_str())
        .ok_or_else(|| format!("unsupported conninfo sslmode `{ssl_mode_raw}`"))?;
    let connect_timeout_s = entries
        .get("connect_timeout")
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|err| format!("invalid conninfo connect_timeout: {err}"))
        })
        .transpose()?;

    Ok(PgConnInfo {
        target: parse_connect_target(host, port)?,
        user: get_required("user")?,
        dbname: get_required("dbname")?,
        application_name: entries.get("application_name").cloned(),
        connect_timeout_s,
        ssl_mode,
        ssl_root_cert: entries.get("sslrootcert").map(PathBuf::from),
        options: entries.get("options").cloned(),
    })
}

pub(crate) fn render_pg_conninfo(info: &PgConnInfo) -> String {
    let (host, port) = render_connect_target(&info.target);
    let mut pairs = vec![
        ("host".to_string(), host),
        ("port".to_string(), port.to_string()),
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

fn parse_connect_target(host: String, port: u16) -> Result<PgConnectTarget, String> {
    if host.starts_with('/') {
        return Ok(PgConnectTarget::Unix(PgUnixTarget {
            socket_dir: PathBuf::from(host),
        }));
    }
    PgTcpTarget::new(host, port).map(PgConnectTarget::Tcp)
}

fn render_connect_target(target: &PgConnectTarget) -> (String, u16) {
    match target {
        PgConnectTarget::Tcp(target) => (target.host().to_string(), target.port()),
        PgConnectTarget::Unix(target) => (target.socket_dir.display().to_string(), 5432),
    }
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
    use std::path::PathBuf;

    use super::{parse_pg_conninfo, render_pg_conninfo, PgConnInfo, PgSslMode};
    use crate::state::{PgConnectTarget, PgTcpTarget};

    fn sample_conninfo() -> Result<PgConnInfo, String> {
        Ok(PgConnInfo {
            target: PgTcpTarget::new("127.0.0.1".to_string(), 5432).map(PgConnectTarget::Tcp)?,
            user: "postgres".to_string(),
            dbname: "postgres".to_string(),
            application_name: Some("ha worker".to_string()),
            connect_timeout_s: Some(5),
            ssl_mode: PgSslMode::Require,
            ssl_root_cert: Some(PathBuf::from("/etc/pgtm/ca bundle.pem")),
            options: Some("-c search_path=public".to_string()),
        })
    }

    #[test]
    fn render_emits_canonical_key_order() -> Result<(), String> {
        let rendered = render_pg_conninfo(&sample_conninfo()?);
        assert_eq!(
            rendered,
            "host=127.0.0.1 port=5432 user=postgres dbname='postgres' application_name='ha worker' connect_timeout=5 sslmode=require sslrootcert='/etc/pgtm/ca bundle.pem' options='-c search_path=public'"
                .replace("dbname='postgres'", "dbname=postgres")
        );
        Ok(())
    }

    #[test]
    fn parse_accepts_rendered_conninfo_with_extra_keys() -> Result<(), String> {
        let rendered = format!(
            "{} passfile='/var/lib/postgresql/data/pgtm.standby.passfile'",
            render_pg_conninfo(&sample_conninfo()?)
        );

        assert_eq!(parse_pg_conninfo(rendered.as_str()), Ok(sample_conninfo()?));
        Ok(())
    }
}
