use serde::{de, Deserialize, Deserializer};
#[cfg(test)]
use thiserror::Error;

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
    pub(crate) options: Option<String>,
}

#[cfg(test)]
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ConnInfoParseError {
    #[error("conninfo syntax error: {0}")]
    Syntax(String),
    #[error("missing required conninfo key `{0}`")]
    MissingRequiredKey(&'static str),
    #[error("duplicate conninfo key `{0}`")]
    DuplicateKey(String),
    #[error("unsupported conninfo key `{0}`")]
    UnsupportedKey(String),
    #[error("unsupported conninfo sslmode `{0}`")]
    UnsupportedSslMode(String),
    #[error("invalid conninfo value for `{key}`: {message}")]
    InvalidValue { key: &'static str, message: String },
}

#[cfg(test)]
pub(crate) fn parse_pg_conninfo(input: &str) -> Result<PgConnInfo, ConnInfoParseError> {
    let mut cursor = Cursor::new(input);

    let mut host = None;
    let mut port = None;
    let mut user = None;
    let mut dbname = None;
    let mut application_name = None;
    let mut connect_timeout_s = None;
    let mut ssl_mode = None;
    let mut options = None;

    while cursor.skip_whitespace() {
        let key = cursor.parse_key()?;
        cursor.expect_equals()?;
        let value = cursor.parse_value()?;
        cursor.require_token_boundary()?;

        match key.as_str() {
            "host" => assign_once(&mut host, key, value)?,
            "port" => {
                if port.is_some() {
                    return Err(ConnInfoParseError::DuplicateKey(key));
                }
                let parsed =
                    value
                        .parse::<u16>()
                        .map_err(|err| ConnInfoParseError::InvalidValue {
                            key: "port",
                            message: err.to_string(),
                        })?;
                port = Some(parsed);
            }
            "user" => assign_once(&mut user, key, value)?,
            "dbname" => assign_once(&mut dbname, key, value)?,
            "application_name" => assign_once(&mut application_name, key, value)?,
            "connect_timeout" => {
                if connect_timeout_s.is_some() {
                    return Err(ConnInfoParseError::DuplicateKey(key));
                }
                let parsed =
                    value
                        .parse::<u32>()
                        .map_err(|err| ConnInfoParseError::InvalidValue {
                            key: "connect_timeout",
                            message: err.to_string(),
                        })?;
                connect_timeout_s = Some(parsed);
            }
            "sslmode" => {
                if ssl_mode.is_some() {
                    return Err(ConnInfoParseError::DuplicateKey(key));
                }
                let parsed = PgSslMode::parse(&value)
                    .ok_or_else(|| ConnInfoParseError::UnsupportedSslMode(value.clone()))?;
                ssl_mode = Some(parsed);
            }
            "options" => assign_once(&mut options, key, value)?,
            _ => return Err(ConnInfoParseError::UnsupportedKey(key)),
        }
    }

    let host = host.ok_or(ConnInfoParseError::MissingRequiredKey("host"))?;
    let port = port.ok_or(ConnInfoParseError::MissingRequiredKey("port"))?;
    let user = user.ok_or(ConnInfoParseError::MissingRequiredKey("user"))?;
    let dbname = dbname.ok_or(ConnInfoParseError::MissingRequiredKey("dbname"))?;

    Ok(PgConnInfo {
        host,
        port,
        user,
        dbname,
        application_name,
        connect_timeout_s,
        ssl_mode: ssl_mode.unwrap_or(PgSslMode::Prefer),
        options,
    })
}

#[cfg(test)]
fn assign_once(
    slot: &mut Option<String>,
    key: String,
    value: String,
) -> Result<(), ConnInfoParseError> {
    if slot.is_some() {
        return Err(ConnInfoParseError::DuplicateKey(key));
    }
    *slot = Some(value);
    Ok(())
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
struct Cursor<'a> {
    src: &'a str,
    index: usize,
}

#[cfg(test)]
impl<'a> Cursor<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, index: 0 }
    }

    fn skip_whitespace(&mut self) -> bool {
        while let Some(ch) = self.peek_char() {
            if !ch.is_whitespace() {
                break;
            }
            let _ = self.next_char();
        }
        self.index < self.src.len()
    }

    fn parse_key(&mut self) -> Result<String, ConnInfoParseError> {
        let mut key = String::new();
        while let Some(ch) = self.peek_char() {
            if ch == '=' {
                break;
            }
            if ch.is_whitespace() {
                return Err(ConnInfoParseError::Syntax(
                    "whitespace before '=' in key/value pair".to_string(),
                ));
            }
            key.push(ch);
            let _ = self.next_char();
        }

        if key.is_empty() {
            return Err(ConnInfoParseError::Syntax(
                "empty conninfo key is not allowed".to_string(),
            ));
        }

        Ok(key)
    }

    fn expect_equals(&mut self) -> Result<(), ConnInfoParseError> {
        match self.next_char() {
            Some('=') => Ok(()),
            _ => Err(ConnInfoParseError::Syntax(
                "expected '=' after conninfo key".to_string(),
            )),
        }
    }

    fn parse_value(&mut self) -> Result<String, ConnInfoParseError> {
        match self.peek_char() {
            Some('\'') => self.parse_quoted_value(),
            Some(_) => self.parse_unquoted_value(),
            None => Err(ConnInfoParseError::Syntax(
                "missing conninfo value after '='".to_string(),
            )),
        }
    }

    fn parse_unquoted_value(&mut self) -> Result<String, ConnInfoParseError> {
        let mut value = String::new();
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                break;
            }
            if ch == '\\' {
                let _ = self.next_char();
                let escaped = self.next_char().ok_or_else(|| {
                    ConnInfoParseError::Syntax("dangling backslash in conninfo value".to_string())
                })?;
                value.push(escaped);
            } else {
                value.push(ch);
                let _ = self.next_char();
            }
        }
        Ok(value)
    }

    fn parse_quoted_value(&mut self) -> Result<String, ConnInfoParseError> {
        let quote = self.next_char();
        if quote != Some('\'') {
            return Err(ConnInfoParseError::Syntax(
                "quoted conninfo value must start with single quote".to_string(),
            ));
        }

        let mut value = String::new();
        loop {
            match self.next_char() {
                Some('\'') => return Ok(value),
                Some('\\') => {
                    let escaped = self.next_char().ok_or_else(|| {
                        ConnInfoParseError::Syntax(
                            "dangling backslash in quoted conninfo value".to_string(),
                        )
                    })?;
                    value.push(escaped);
                }
                Some(ch) => value.push(ch),
                None => {
                    return Err(ConnInfoParseError::Syntax(
                        "unterminated quoted conninfo value".to_string(),
                    ))
                }
            }
        }
    }

    fn require_token_boundary(&self) -> Result<(), ConnInfoParseError> {
        if let Some(ch) = self.peek_char() {
            if !ch.is_whitespace() {
                return Err(ConnInfoParseError::Syntax(
                    "expected whitespace between conninfo key/value pairs".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn peek_char(&self) -> Option<char> {
        self.src[self.index..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let next = self.peek_char()?;
        self.index += next.len_utf8();
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_pg_conninfo, render_pg_conninfo, ConnInfoParseError, PgConnInfo, PgSslMode};

    fn sample_conninfo() -> PgConnInfo {
        PgConnInfo {
            host: "127.0.0.1".to_string(),
            port: 5432,
            user: "postgres".to_string(),
            dbname: "postgres".to_string(),
            application_name: Some("ha worker".to_string()),
            connect_timeout_s: Some(5),
            ssl_mode: PgSslMode::Require,
            options: Some("-c search_path=public".to_string()),
        }
    }

    #[test]
    fn parse_accepts_minimal_valid_input() {
        let parsed = parse_pg_conninfo("host=127.0.0.1 port=5432 user=postgres dbname=postgres");
        assert_eq!(
            parsed,
            Ok(PgConnInfo {
                host: "127.0.0.1".to_string(),
                port: 5432,
                user: "postgres".to_string(),
                dbname: "postgres".to_string(),
                application_name: None,
                connect_timeout_s: None,
                ssl_mode: PgSslMode::Prefer,
                options: None,
            })
        );
    }

    #[test]
    fn parse_accepts_quoted_values_with_escapes() {
        let parsed = parse_pg_conninfo(
            "host='127.0.0.1' port=5432 user=postgres dbname='app db' application_name='ha \\'node\\'' options='-c search_path=public'",
        );
        assert!(parsed.is_ok());
        if let Ok(info) = parsed {
            assert_eq!(info.dbname, "app db");
            assert_eq!(info.application_name, Some("ha 'node'".to_string()));
            assert_eq!(info.options, Some("-c search_path=public".to_string()));
        }
    }

    #[test]
    fn parse_rejects_invalid_syntax() {
        assert!(matches!(
            parse_pg_conninfo("host 127.0.0.1 port=5432 user=postgres dbname=postgres"),
            Err(ConnInfoParseError::Syntax(_))
        ));
        assert!(matches!(
            parse_pg_conninfo("host='127.0.0.1 port=5432 user=postgres dbname=postgres"),
            Err(ConnInfoParseError::Syntax(_))
        ));
        assert!(matches!(
            parse_pg_conninfo("host='127.0.0.1'port=5432 user=postgres dbname=postgres"),
            Err(ConnInfoParseError::Syntax(_))
        ));
    }

    #[test]
    fn parse_rejects_missing_required_keys() {
        assert_eq!(
            parse_pg_conninfo("host=127.0.0.1 user=postgres dbname=postgres"),
            Err(ConnInfoParseError::MissingRequiredKey("port"))
        );
    }

    #[test]
    fn parse_rejects_duplicate_keys() {
        assert_eq!(
            parse_pg_conninfo(
                "host=127.0.0.1 host=127.0.0.2 port=5432 user=postgres dbname=postgres"
            ),
            Err(ConnInfoParseError::DuplicateKey("host".to_string()))
        );
    }

    #[test]
    fn parse_rejects_unknown_key_typos() {
        assert_eq!(
            parse_pg_conninfo(
                "host=127.0.0.1 port=5432 user=postgres dbname=postgres sslmdoe=require"
            ),
            Err(ConnInfoParseError::UnsupportedKey("sslmdoe".to_string()))
        );
    }

    #[test]
    fn parse_rejects_unsupported_sslmode() {
        assert_eq!(
            parse_pg_conninfo(
                "host=127.0.0.1 port=5432 user=postgres dbname=postgres sslmode=invalid"
            ),
            Err(ConnInfoParseError::UnsupportedSslMode(
                "invalid".to_string()
            ))
        );
    }

    #[test]
    fn parse_rejects_invalid_numeric_ranges() {
        assert!(matches!(
            parse_pg_conninfo("host=127.0.0.1 port=70000 user=postgres dbname=postgres"),
            Err(ConnInfoParseError::InvalidValue { key: "port", .. })
        ));
        assert!(matches!(
            parse_pg_conninfo(
                "host=127.0.0.1 port=5432 user=postgres dbname=postgres connect_timeout=-1"
            ),
            Err(ConnInfoParseError::InvalidValue {
                key: "connect_timeout",
                ..
            })
        ));
    }

    #[test]
    fn render_emits_canonical_key_order() {
        let rendered = render_pg_conninfo(&sample_conninfo());
        assert_eq!(
            rendered,
            "host=127.0.0.1 port=5432 user=postgres dbname='postgres' application_name='ha worker' connect_timeout=5 sslmode=require options='-c search_path=public'"
                .replace("dbname='postgres'", "dbname=postgres")
        );
    }

    #[test]
    fn parse_render_roundtrip_is_stable() {
        let original = sample_conninfo();
        let rendered = render_pg_conninfo(&original);
        let reparsed = parse_pg_conninfo(&rendered);
        assert_eq!(reparsed, Ok(original));
    }
}
