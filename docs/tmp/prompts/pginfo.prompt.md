Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise an existing reference page so it stays strictly in Diataxis reference form.

[Page path]
- docs/src/reference/pginfo.md

[Page goal]
- Reference the pginfo module surface, conninfo parse/render helpers, poll query contract, published state model, and worker loop.

[Audience]
- Operators and contributors who need accurate repo-backed facts while working with pgtuskmaster.

[User need]
- Consult the machinery surface, data model, constraints, constants, and behavior without being taught procedures or background explanations.

[mdBook context]
- This is an mdBook page under docs/src/reference/.
- Keep headings and lists suitable for mdBook.
- Do not add verification notes, scratch notes, or commentary about how the page was produced.

[Diataxis guidance]
- This page must stay in the reference quadrant: cognition plus application.
- Describe and only describe.
- Structure the page to mirror the machinery, not a guessed workflow.
- Use neutral, technical language.
- Examples are allowed only when they illustrate the surface concisely.
- Do not include step-by-step operations, recommendations, rationale, or explanations of why the design exists.
- If action or explanation seems necessary, keep the page neutral and mention the boundary without turning the page into a how-to or explanation article.

[Required structure]
- Overview\n- Module surface\n- Conninfo types and parsing\n- Poll query and decoded payload\n- Published state model\n- Worker loop and emitted events\n- Verified behaviors from direct tests

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

# PostgreSQL Observation Reference

The `pginfo` subsystem parses PostgreSQL conninfo strings, polls PostgreSQL state via SQL, and publishes typed state snapshots.

## Module Surface

| Path | Visibility |
|---|---|
| `src/pginfo/mod.rs` | module definition, public `conninfo` |
| `src/pginfo/conninfo.rs` | public module |
| `src/pginfo/query.rs` | crate-visible |
| `src/pginfo/state.rs` | crate-visible |
| `src/pginfo/worker.rs` | crate-visible |

## Conninfo Parsing And Rendering

### `PgConnInfo`

| Field | Type |
|---|---|
| `host` | `String` |
| `port` | `u16` |
| `user` | `String` |
| `dbname` | `String` |
| `application_name` | `Option<String>` |
| `connect_timeout_s` | `Option<u32>` |
| `ssl_mode` | `PgSslMode` |
| `options` | `Option<String>` |

### `PgSslMode`

| Variant | String |
|---|---|
| `Disable` | `disable` |
| `Allow` | `allow` |
| `Prefer` | `prefer` |
| `Require` | `require` |
| `VerifyCa` | `verify-ca` |
| `VerifyFull` | `verify-full` |

`PgSslMode::parse` returns `None` for any string not listed above. `Deserialize` for `PgSslMode` rejects unsupported values with message `unsupported sslmode \`<value>\``.

### `parse_pg_conninfo`

Accepts keys `host`, `port`, `user`, `dbname`, `application_name`, `connect_timeout`, `sslmode`, and `options`. Requires `host`, `port`, `user`, and `dbname`. Defaults `ssl_mode` to `PgSslMode::Prefer` when omitted. Rejects:

- whitespace before `=`
- empty keys
- duplicate keys
- unsupported keys
- unsupported `sslmode` values
- invalid numeric values

### `render_pg_conninfo`

Always renders `host`, `port`, `user`, `dbname`, and `sslmode`. Conditionally renders `application_name`, `connect_timeout`, and `options` when present. Quotes values that are empty or contain whitespace, `'`, or `\`. Escapes `'` and `\` inside quoted values.

### `ConnInfoParseError`

| Variant | Trigger |
|---|---|
| `Syntax(String)` | whitespace before `=` or empty key |
| `MissingRequiredKey(&'static str)` | missing required key |
| `DuplicateKey(String)` | repeated key |
| `UnsupportedKey(String)` | unsupported key |
| `UnsupportedSslMode(String)` | unsupported `sslmode` value |
| `InvalidValue { key, message }` | invalid numeric value |

## Poll Query And Polling Function

### `PGINFO_POLL_SQL`

| Column | Expression |
|---|---|
| `in_recovery` | `pg_is_in_recovery()` |
| `is_ready` | `TRUE` on primaries; on replicas `TRUE` only when `pg_last_wal_replay_lsn()` is not `NULL` |
| `timeline_id` | `(pg_control_checkpoint()).timeline_id::bigint` |
| `current_wal_lsn` | `pg_current_wal_lsn()::text` when not in recovery, otherwise `NULL` |
| `replay_lsn` | `pg_last_wal_replay_lsn()` |
| `receive_lsn` | `pg_last_wal_receive_lsn()` |
| `slot_names` | `COALESCE(array_remove(array_agg(slot_name ORDER BY slot_name), NULL), '{}'::text[])` |

### `PgPollData`

| Field | Type |
|---|---|
| `in_recovery` | `bool` |
| `is_ready` | `bool` |
| `timeline` | `Option<TimelineId>` |
| `current_wal_lsn` | `Option<WalLsn>` |
| `replay_lsn` | `Option<WalLsn>` |
| `receive_lsn` | `Option<WalLsn>` |
| `slot_names` | `Vec<String>` |

### `parse_wal_lsn`

Requires `X/Y` hexadecimal format. Parses each half as hexadecimal, left-shifts the high segment by `32` bits, adds the low segment, and returns `WalLsn`. Errors on malformed format, invalid hex, high-segment overflow, or combined-value overflow.

### `parse_timeline`

Rejects negative `i64` values and converts non-negative values to `TimelineId(u32)`.

### `poll_once`

Renders a DSN from `PgConnInfo`, connects with `tokio_postgres::connect` using `NoTls`, spawns the connection task, runs `query_one(PGINFO_POLL_SQL, &[])`, drops the client, waits for the connection task, decodes columns, parses timeline and LSN values, and returns `PgPollData`. Connect, query, connection task join, post-query connection error, decode, timeline parse, and LSN parse failures map to `WorkerError::Message`.

## Published State Model

### `SqlStatus`

Variants: `Unknown`, `Healthy`, `Unreachable`.

### `Readiness`

Variants: `Unknown`, `Ready`, `NotReady`.

### `PgConfig`

| Field | Type |
|---|---|
| `port` | `Option<u16>` |
| `hot_standby` | `Option<bool>` |
| `primary_conninfo` | `Option<PgConnInfo>` |
| `primary_slot_name` | `Option<String>` |
| `extra` | `BTreeMap<String, String>` |

### `ReplicationSlotInfo`

| Field | Type |
|---|---|
| `name` | `String` |

### `UpstreamInfo`

| Field | Type |
|---|---|
| `member_id` | `MemberId` |

### `PgInfoCommon`

| Field | Type |
|---|---|
| `worker` | `WorkerStatus` |
| `sql` | `SqlStatus` |
| `readiness` | `Readiness` |
| `timeline` | `Option<TimelineId>` |
| `pg_config` | `PgConfig` |
| `last_refresh_at` | `Option<UnixMillis>` |

### `PgInfoState`

| Variant | Fields |
|---|---|
| `Unknown` | `common: PgInfoCommon` |
| `Primary` | `common: PgInfoCommon`, `wal_lsn: WalLsn`, `slots: Vec<ReplicationSlotInfo>` |
| `Replica` | `common: PgInfoCommon`, `replay_lsn: WalLsn`, `follow_lsn: Option<WalLsn>`, `upstream: Option<UpstreamInfo>` |

### `derive_readiness`

Maps `(SqlStatus, is_ready)` to `Readiness`:

| `SqlStatus` | `is_ready` | `Readiness` |
|---|---|---|
| `Healthy` | `true` | `Ready` |
| `Healthy` | `false` | `NotReady` |
| `Unknown` | any | `Unknown` |
| `Unreachable` | any | `NotReady` |

### `to_member_status`

Builds `PgInfoCommon` with the supplied worker status and SQL status, readiness from `derive_readiness`, timeline from the poll result, empty optional `PgConfig` fields plus empty `extra`, and `last_refresh_at` set to the poll timestamp. Returns:

- `Unknown` when poll data is absent
- `Replica` when `in_recovery = true` and `replay_lsn` is present, with `follow_lsn` from `receive_lsn` and `upstream: None`
- `Unknown` when `in_recovery = true` and `replay_lsn` is absent
- `Primary` when `in_recovery = false` and `current_wal_lsn` is present, with slots mapped from `slot_names`
- `Unknown` for all other cases

## Worker Loop And Events

### `PgInfoWorkerCtx`

| Field | Type |
|---|---|
| `self_id` | `MemberId` |
| `postgres_conninfo` | `PgConnInfo` |
| `poll_interval` | `Duration` |
| `publisher` | `StatePublisher<PgInfoState>` |
| `log` | `LogHandle` |
| `last_emitted_sql_status` | `Option<SqlStatus>` |

### `worker::run`

Loops forever, calling `step_once`, then sleeping for `poll_interval`.

### `worker::step_once`

Gets current Unix milliseconds, calls `poll_once`, maps success to `WorkerStatus::Running` and `SqlStatus::Healthy`, maps failure to `WorkerStatus::Running` and `SqlStatus::Unreachable`, and publishes the resulting `PgInfoState`.

On poll failure, emits an app event with severity `Warn`, message `pginfo poll failed`, name `pginfo.poll_failed`, domain `pginfo`, result `failed`, and fields `member_id` and `error`.

When SQL status changes, emits `pginfo.sql_transition`:

| Transition | Severity | Result | Fields |
|---|---|---|---|
| `Healthy -> Unreachable` | `Warn` | `failed` | `member_id`, `sql_status_prev`, `sql_status_next` |
| `Unreachable -> Healthy` | `Info` | `recovered` | `member_id`, `sql_status_prev`, `sql_status_next` |
| other changes | `Debug` | `ok` | `member_id`, `sql_status_prev`, `sql_status_next` |

`step_once` publishes the next state with the current timestamp. Publish failures map to `WorkerError::Message("pginfo publish failed for {:?}: {err}")`.

`now_unix_millis` returns `WorkerError` when the system clock is before the Unix epoch or the millisecond conversion fails.

## Verified Behaviors

- `src/pginfo/query.rs`: validates `parse_wal_lsn` for valid and invalid formats; verifies `PGINFO_POLL_SQL` selects expected fields with one semicolon
- `src/pginfo/state.rs`: validates readiness mapping; verifies primary and replica state derivation
- `src/pginfo/worker.rs`: real PostgreSQL flows verify unreachable-to-primary transition, WAL and slot tracking on primary, replica convergence, and emitted SQL transition events

[Repo facts and source excerpts]

--- BEGIN FILE: src/pginfo/mod.rs ---
pub mod conninfo;
pub(crate) mod query;
pub(crate) mod state;
pub(crate) mod worker;

--- END FILE: src/pginfo/mod.rs ---

--- BEGIN FILE: src/pginfo/conninfo.rs ---
use serde::{de, Deserialize, Deserializer};
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

struct Cursor<'a> {
    src: &'a str,
    index: usize,
}

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

--- END FILE: src/pginfo/conninfo.rs ---

--- BEGIN FILE: src/pginfo/query.rs ---
use crate::{
    pginfo::state::{render_pg_conninfo, PgConnInfo},
    state::{TimelineId, WalLsn, WorkerError},
};

pub(crate) const PGINFO_POLL_SQL: &str = r#"
SELECT
    s.in_recovery,
    s.is_ready,
    s.timeline_id,
    s.current_wal_lsn,
    s.replay_lsn,
    s.receive_lsn,
    COALESCE(r.slot_names, '{}'::text[]) AS slot_names
FROM (
    SELECT
        pg_is_in_recovery() AS in_recovery,
        CASE
            WHEN pg_is_in_recovery() THEN pg_last_wal_replay_lsn() IS NOT NULL
            ELSE TRUE
        END AS is_ready,
        (pg_control_checkpoint()).timeline_id::bigint AS timeline_id,
        CASE
            WHEN pg_is_in_recovery() THEN NULL
            ELSE pg_current_wal_lsn()::text
        END AS current_wal_lsn,
        pg_last_wal_replay_lsn()::text AS replay_lsn,
        pg_last_wal_receive_lsn()::text AS receive_lsn
) AS s
CROSS JOIN (
    SELECT COALESCE(array_remove(array_agg(slot_name ORDER BY slot_name), NULL), '{}'::text[]) AS slot_names
    FROM pg_replication_slots
) AS r;
"#;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgPollData {
    pub(crate) in_recovery: bool,
    pub(crate) is_ready: bool,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) current_wal_lsn: Option<WalLsn>,
    pub(crate) replay_lsn: Option<WalLsn>,
    pub(crate) receive_lsn: Option<WalLsn>,
    pub(crate) slot_names: Vec<String>,
}

pub(crate) async fn poll_once(postgres_conninfo: &PgConnInfo) -> Result<PgPollData, WorkerError> {
    let postgres_dsn = render_pg_conninfo(postgres_conninfo);
    let (client, connection) = tokio_postgres::connect(&postgres_dsn, tokio_postgres::NoTls)
        .await
        .map_err(|err| WorkerError::Message(format!("postgres connect failed: {err}")))?;

    let connection_task = tokio::spawn(connection);

    let row = client
        .query_one(PGINFO_POLL_SQL, &[])
        .await
        .map_err(|err| WorkerError::Message(format!("pginfo poll query failed: {err}")))?;

    drop(client);

    let connection_result = connection_task.await.map_err(|err| {
        WorkerError::Message(format!("postgres connection task join failed: {err}"))
    })?;
    if let Err(err) = connection_result {
        return Err(WorkerError::Message(format!(
            "postgres connection error after poll: {err}"
        )));
    }

    let timeline_raw: Option<i64> = row
        .try_get("timeline_id")
        .map_err(|err| WorkerError::Message(format!("timeline decode failed: {err}")))?;
    let timeline = parse_timeline(timeline_raw)?;

    let current_wal_lsn =
        parse_optional_lsn(row.try_get("current_wal_lsn").map_err(|err| {
            WorkerError::Message(format!("current_wal_lsn decode failed: {err}"))
        })?)?;
    let replay_lsn = parse_optional_lsn(
        row.try_get("replay_lsn")
            .map_err(|err| WorkerError::Message(format!("replay_lsn decode failed: {err}")))?,
    )?;
    let receive_lsn = parse_optional_lsn(
        row.try_get("receive_lsn")
            .map_err(|err| WorkerError::Message(format!("receive_lsn decode failed: {err}")))?,
    )?;

    let slot_names: Vec<String> = row
        .try_get("slot_names")
        .map_err(|err| WorkerError::Message(format!("slot_names decode failed: {err}")))?;

    let in_recovery: bool = row
        .try_get("in_recovery")
        .map_err(|err| WorkerError::Message(format!("in_recovery decode failed: {err}")))?;
    let is_ready: bool = row
        .try_get("is_ready")
        .map_err(|err| WorkerError::Message(format!("is_ready decode failed: {err}")))?;

    Ok(PgPollData {
        in_recovery,
        is_ready,
        timeline,
        current_wal_lsn,
        replay_lsn,
        receive_lsn,
        slot_names,
    })
}

pub(crate) fn parse_wal_lsn(raw: &str) -> Result<WalLsn, WorkerError> {
    let trimmed = raw.trim();
    let Some((left, right)) = trimmed.split_once('/') else {
        return Err(WorkerError::Message(format!(
            "invalid LSN '{trimmed}': expected X/Y format"
        )));
    };

    let left_num = u64::from_str_radix(left, 16).map_err(|err| {
        WorkerError::Message(format!(
            "invalid LSN '{trimmed}': high segment parse failed: {err}"
        ))
    })?;
    let right_num = u64::from_str_radix(right, 16).map_err(|err| {
        WorkerError::Message(format!(
            "invalid LSN '{trimmed}': low segment parse failed: {err}"
        ))
    })?;

    let shifted = left_num.checked_shl(32).ok_or_else(|| {
        WorkerError::Message(format!("invalid LSN '{trimmed}': high segment overflow"))
    })?;
    let combined = shifted.checked_add(right_num).ok_or_else(|| {
        WorkerError::Message(format!("invalid LSN '{trimmed}': combined value overflow"))
    })?;
    Ok(WalLsn(combined))
}

fn parse_optional_lsn(raw: Option<String>) -> Result<Option<WalLsn>, WorkerError> {
    match raw {
        Some(value) => parse_wal_lsn(&value).map(Some),
        None => Ok(None),
    }
}

fn parse_timeline(raw: Option<i64>) -> Result<Option<TimelineId>, WorkerError> {
    match raw {
        Some(value) => {
            if value < 0 {
                return Err(WorkerError::Message(format!(
                    "timeline must be non-negative, got {value}"
                )));
            }
            let as_u32 = u32::try_from(value)
                .map_err(|err| WorkerError::Message(format!("timeline out of range: {err}")))?;
            Ok(Some(TimelineId(as_u32)))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_wal_lsn, PGINFO_POLL_SQL};

    #[test]
    fn parse_wal_lsn_accepts_valid_hex_format() {
        let parsed = parse_wal_lsn("16/B374D848");
        assert!(parsed.is_ok());
        if let Ok(lsn) = parsed {
            assert_eq!(lsn.0, 0x16_0000_0000 + 0xB374D848);
        }
    }

    #[test]
    fn parse_wal_lsn_rejects_invalid_formats() {
        assert!(parse_wal_lsn("16").is_err());
        assert!(parse_wal_lsn("G/10").is_err());
        assert!(parse_wal_lsn("10/XYZ").is_err());
    }

    #[test]
    fn poll_sql_selects_expected_fields() {
        assert!(PGINFO_POLL_SQL.contains("in_recovery"));
        assert!(PGINFO_POLL_SQL.contains("timeline_id"));
        assert!(PGINFO_POLL_SQL.contains("current_wal_lsn"));
        assert!(PGINFO_POLL_SQL.contains("replay_lsn"));
        assert!(PGINFO_POLL_SQL.contains("receive_lsn"));
        assert!(PGINFO_POLL_SQL.contains("slot_names"));
        assert_eq!(PGINFO_POLL_SQL.matches(';').count(), 1);
    }
}

--- END FILE: src/pginfo/query.rs ---

--- BEGIN FILE: src/pginfo/state.rs ---
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub(crate) use super::conninfo::{render_pg_conninfo, PgConnInfo, PgSslMode};
use super::query::PgPollData;
use crate::logging::LogHandle;
use crate::state::StatePublisher;
use crate::state::{MemberId, TimelineId, UnixMillis, WalLsn, WorkerStatus};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SqlStatus {
    Unknown,
    Healthy,
    Unreachable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Readiness {
    Unknown,
    Ready,
    NotReady,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgConfig {
    pub(crate) port: Option<u16>,
    pub(crate) hot_standby: Option<bool>,
    pub(crate) primary_conninfo: Option<PgConnInfo>,
    pub(crate) primary_slot_name: Option<String>,
    pub(crate) extra: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplicationSlotInfo {
    pub(crate) name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UpstreamInfo {
    pub(crate) member_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgInfoCommon {
    pub(crate) worker: WorkerStatus,
    pub(crate) sql: SqlStatus,
    pub(crate) readiness: Readiness,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) pg_config: PgConfig,
    pub(crate) last_refresh_at: Option<UnixMillis>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PgInfoState {
    Unknown {
        common: PgInfoCommon,
    },
    Primary {
        common: PgInfoCommon,
        wal_lsn: WalLsn,
        slots: Vec<ReplicationSlotInfo>,
    },
    Replica {
        common: PgInfoCommon,
        replay_lsn: WalLsn,
        follow_lsn: Option<WalLsn>,
        upstream: Option<UpstreamInfo>,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct PgInfoWorkerCtx {
    pub(crate) self_id: MemberId,
    pub(crate) postgres_conninfo: PgConnInfo,
    pub(crate) poll_interval: Duration,
    pub(crate) publisher: StatePublisher<PgInfoState>,
    pub(crate) log: LogHandle,
    pub(crate) last_emitted_sql_status: Option<SqlStatus>,
}

pub(crate) fn derive_readiness(sql: &SqlStatus, is_ready: bool) -> Readiness {
    match sql {
        SqlStatus::Healthy => {
            if is_ready {
                Readiness::Ready
            } else {
                Readiness::NotReady
            }
        }
        SqlStatus::Unknown => Readiness::Unknown,
        SqlStatus::Unreachable => Readiness::NotReady,
    }
}

pub(crate) fn to_member_status(
    worker_status: WorkerStatus,
    sql_status: SqlStatus,
    polled_at: UnixMillis,
    poll: Option<PgPollData>,
) -> PgInfoState {
    let readiness_signal = poll.as_ref().map(|value| value.is_ready).unwrap_or(false);
    let timeline = poll.as_ref().and_then(|value| value.timeline);
    let common = PgInfoCommon {
        worker: worker_status,
        sql: sql_status.clone(),
        readiness: derive_readiness(&sql_status, readiness_signal),
        timeline,
        pg_config: PgConfig {
            port: None,
            hot_standby: None,
            primary_conninfo: None,
            primary_slot_name: None,
            extra: std::collections::BTreeMap::new(),
        },
        last_refresh_at: Some(polled_at),
    };

    let Some(polled) = poll else {
        return PgInfoState::Unknown { common };
    };

    if polled.in_recovery {
        if let Some(replay_lsn) = polled.replay_lsn {
            return PgInfoState::Replica {
                common,
                replay_lsn,
                follow_lsn: polled.receive_lsn,
                upstream: None,
            };
        }
        return PgInfoState::Unknown { common };
    }

    if let Some(wal_lsn) = polled.current_wal_lsn {
        return PgInfoState::Primary {
            common,
            wal_lsn,
            slots: polled
                .slot_names
                .into_iter()
                .map(|name| ReplicationSlotInfo { name })
                .collect(),
        };
    }

    PgInfoState::Unknown { common }
}

#[cfg(test)]
mod tests {
    use crate::state::{UnixMillis, WalLsn, WorkerStatus};

    use super::{derive_readiness, to_member_status, PgInfoState, Readiness, SqlStatus};
    use crate::pginfo::query::PgPollData;
    use crate::state::TimelineId;

    #[test]
    fn derive_readiness_maps_sql_and_signal() {
        assert_eq!(
            derive_readiness(&SqlStatus::Unknown, false),
            Readiness::Unknown
        );
        assert_eq!(
            derive_readiness(&SqlStatus::Unreachable, true),
            Readiness::NotReady
        );
        assert_eq!(
            derive_readiness(&SqlStatus::Healthy, true),
            Readiness::Ready
        );
        assert_eq!(
            derive_readiness(&SqlStatus::Healthy, false),
            Readiness::NotReady
        );
    }

    #[test]
    fn to_member_status_maps_primary_snapshot() {
        let poll = PgPollData {
            in_recovery: false,
            is_ready: true,
            timeline: Some(TimelineId(3)),
            current_wal_lsn: Some(WalLsn(42)),
            replay_lsn: None,
            receive_lsn: None,
            slot_names: vec!["slot_a".to_string(), "slot_b".to_string()],
        };
        let state = to_member_status(
            WorkerStatus::Running,
            SqlStatus::Healthy,
            UnixMillis(100),
            Some(poll),
        );
        assert!(matches!(&state, PgInfoState::Primary { .. }));
        if let PgInfoState::Primary {
            wal_lsn,
            slots,
            common,
            ..
        } = &state
        {
            assert_eq!(*wal_lsn, WalLsn(42));
            assert_eq!(slots.len(), 2);
            assert_eq!(common.readiness, Readiness::Ready);
        }
    }

    #[test]
    fn to_member_status_maps_replica_snapshot() {
        let poll = PgPollData {
            in_recovery: true,
            is_ready: true,
            timeline: Some(TimelineId(8)),
            current_wal_lsn: None,
            replay_lsn: Some(WalLsn(11)),
            receive_lsn: Some(WalLsn(12)),
            slot_names: Vec::new(),
        };
        let state = to_member_status(
            WorkerStatus::Running,
            SqlStatus::Healthy,
            UnixMillis(100),
            Some(poll),
        );
        assert!(matches!(&state, PgInfoState::Replica { .. }));
        if let PgInfoState::Replica {
            replay_lsn,
            follow_lsn,
            common,
            ..
        } = &state
        {
            assert_eq!(*replay_lsn, WalLsn(11));
            assert_eq!(*follow_lsn, Some(WalLsn(12)));
            assert_eq!(common.readiness, Readiness::Ready);
        }
    }
}

--- END FILE: src/pginfo/state.rs ---

--- BEGIN FILE: src/pginfo/worker.rs ---
use crate::state::{UnixMillis, WorkerStatus};
use crate::{
    logging::{AppEvent, AppEventHeader, SeverityText, StructuredFields},
    state::WorkerError,
};

use super::query::poll_once;
use super::state::{to_member_status, PgInfoState, PgInfoWorkerCtx, SqlStatus};

fn pginfo_append_base_fields(fields: &mut StructuredFields, ctx: &PgInfoWorkerCtx) {
    fields.insert("member_id", ctx.self_id.0.clone());
}

fn pginfo_event(severity: SeverityText, message: &str, name: &str, result: &str) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(name, "pginfo", result),
    )
}

fn emit_pginfo_event(
    ctx: &PgInfoWorkerCtx,
    origin: &str,
    event: AppEvent,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    ctx.log
        .emit_app_event(origin, event)
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
}

fn sql_status_label(status: &SqlStatus) -> String {
    format!("{status:?}").to_lowercase()
}

pub(crate) async fn run(mut ctx: PgInfoWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut PgInfoWorkerCtx) -> Result<(), WorkerError> {
    let now = now_unix_millis()?;
    let poll = poll_once(&ctx.postgres_conninfo).await;
    let next_state = match poll {
        Ok(polled) => {
            to_member_status(WorkerStatus::Running, SqlStatus::Healthy, now, Some(polled))
        }
        Err(ref err) => {
            let mut event = pginfo_event(
                SeverityText::Warn,
                "pginfo poll failed",
                "pginfo.poll_failed",
                "failed",
            );
            let fields = event.fields_mut();
            pginfo_append_base_fields(fields, ctx);
            fields.insert("error", err.to_string());
            emit_pginfo_event(
                ctx,
                "pginfo_worker::step_once",
                event,
                "pginfo poll failure log emit failed",
            )?;
            to_member_status(WorkerStatus::Running, SqlStatus::Unreachable, now, None)
        }
    };

    let next_sql = pginfo_sql_status(&next_state);
    let prev_sql = ctx
        .last_emitted_sql_status
        .clone()
        .unwrap_or(SqlStatus::Unknown);
    if prev_sql != next_sql {
        let (severity, result) = match (prev_sql.clone(), next_sql.clone()) {
            (SqlStatus::Healthy, SqlStatus::Unreachable) => (SeverityText::Warn, "failed"),
            (SqlStatus::Unreachable, SqlStatus::Healthy) => (SeverityText::Info, "recovered"),
            _ => (SeverityText::Debug, "ok"),
        };
        let mut event = pginfo_event(
            severity,
            "pginfo sql status transition",
            "pginfo.sql_transition",
            result,
        );
        let fields = event.fields_mut();
        pginfo_append_base_fields(fields, ctx);
        fields.insert("sql_status_prev", sql_status_label(&prev_sql));
        fields.insert("sql_status_next", sql_status_label(&next_sql));
        emit_pginfo_event(
            ctx,
            "pginfo_worker::step_once",
            event,
            "pginfo sql transition log emit failed",
        )?;
        ctx.last_emitted_sql_status = Some(next_sql.clone());
    }

    ctx.publisher.publish(next_state, now).map_err(|err| {
        WorkerError::Message(format!(
            "pginfo publish failed for {:?}: {err}",
            ctx.self_id
        ))
    })?;
    Ok(())
}

fn pginfo_sql_status(state: &PgInfoState) -> SqlStatus {
    match state {
        PgInfoState::Unknown { common } => common.sql.clone(),
        PgInfoState::Primary { common, .. } => common.sql.clone(),
        PgInfoState::Replica { common, .. } => common.sql.clone(),
    }
}

fn now_unix_millis() -> Result<UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use std::time::Duration;

    use std::sync::Arc;

    use tokio::time::Instant;
    use tokio_postgres::NoTls;

    use crate::logging::{decode_app_event, LogHandle, LogSink, SeverityText, TestSink};
    use crate::pginfo::state::{PgConfig, PgInfoCommon};
    use crate::state::{new_state_channel, MemberId, UnixMillis, WorkerStatus};
    use crate::test_harness::binaries::require_pg16_bin_for_real_tests;
    use crate::test_harness::namespace::NamespaceGuard;
    use crate::test_harness::pg16::{
        prepare_pgdata_dir, spawn_pg16, spawn_pg16_for_vanilla_postgres, PgHandle, PgInstanceSpec,
    };
    use crate::test_harness::ports::allocate_ports;

    use super::{step_once, PgInfoWorkerCtx, SqlStatus};
    use crate::pginfo::state::{PgConnInfo, PgInfoState, PgSslMode, Readiness};

    type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn test_error(message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(io::Error::other(message.into()))
    }

    fn test_log_handle() -> (LogHandle, Arc<TestSink>) {
        let sink = Arc::new(TestSink::default());
        let sink_dyn: Arc<dyn LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
    }

    fn local_test_conninfo(port: u16) -> PgConnInfo {
        PgConnInfo {
            host: "127.0.0.1".to_string(),
            port,
            user: "postgres".to_string(),
            dbname: "postgres".to_string(),
            application_name: None,
            connect_timeout_s: None,
            ssl_mode: PgSslMode::Prefer,
            options: None,
        }
    }

    async fn wait_for_postgres_ready(dsn: &str, timeout: Duration) -> TestResult {
        let deadline = Instant::now() + timeout;
        loop {
            match tokio_postgres::connect(dsn, NoTls).await {
                Ok((client, connection)) => {
                    let conn_task = tokio::spawn(connection);
                    client.simple_query("SELECT 1;").await?;
                    drop(client);
                    conn_task.await??;
                    return Ok(());
                }
                Err(err) => {
                    if Instant::now() >= deadline {
                        return Err(Box::new(err));
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn shutdown_with_context(label: &str, handle: &mut PgHandle) -> TestResult {
        handle
            .shutdown()
            .await
            .map_err(|err| test_error(format!("{label} shutdown failed: {err}")))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_transitions_unreachable_to_primary_and_tracks_wal_and_slots() -> TestResult {
        let postgres_bin = require_pg16_bin_for_real_tests("postgres")?;
        let initdb_bin = require_pg16_bin_for_real_tests("initdb")?;

        let guard = NamespaceGuard::new("pginfo-primary-flow")?;
        let namespace = guard.namespace()?;

        let reservation = allocate_ports(1)?;
        let port = reservation.as_slice()[0];

        let data_dir = prepare_pgdata_dir(namespace, "primary")?;
        let socket_dir = namespace.child_dir("run/primary");
        let log_dir = namespace.child_dir("logs/primary");
        fs::create_dir_all(&socket_dir)?;
        fs::create_dir_all(&log_dir)?;

        let spec = PgInstanceSpec {
            postgres_bin,
            initdb_bin,
            data_dir: data_dir.clone(),
            socket_dir: socket_dir.clone(),
            log_dir,
            port,
            startup_timeout: Duration::from_secs(20),
        };

        // Release the reserved port immediately before spawning postgres so the
        // child can bind the same port.
        drop(reservation);
        let mut handle = spawn_pg16(spec).await?;

        let conninfo = local_test_conninfo(port);

        let unknown = PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: WorkerStatus::Starting,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: std::collections::BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
        };
        let (publisher, subscriber) = new_state_channel(unknown, UnixMillis(1));
        let (log, sink) = test_log_handle();
        let mut ctx = PgInfoWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            postgres_conninfo: conninfo.clone(),
            poll_interval: Duration::from_millis(25),
            publisher,
            log,
            last_emitted_sql_status: None,
        };

        let run_result: TestResult = async {
            let dsn = crate::pginfo::state::render_pg_conninfo(&conninfo);
            wait_for_postgres_ready(&dsn, Duration::from_secs(10)).await?;
            step_once(&mut ctx).await?;

            let transitions = sink
                .take()
                .into_iter()
                .filter_map(|record| decode_app_event(&record).ok())
                .filter(|event| event.header.name == "pginfo.sql_transition")
                .collect::<Vec<_>>();
            if transitions.is_empty() {
                return Err(test_error(
                    "expected pginfo.sql_transition event on first poll".to_string(),
                ));
            }
            if transitions[0].header.result != "ok" {
                return Err(test_error(format!(
                    "expected initial pginfo.sql_transition result ok, got {:?}",
                    transitions[0].header.result
                )));
            }

            let first = subscriber.latest().value;
            let first_wal = match first {
                PgInfoState::Primary { wal_lsn, .. } => wal_lsn,
                other => {
                    return Err(test_error(format!(
                        "expected primary state after first poll, got: {other:?}"
                    )));
                }
            };

            let (client, connection) = tokio_postgres::connect(&dsn, NoTls).await?;
            let conn_task = tokio::spawn(connection);

            client
                .batch_execute(
                    "CREATE TABLE IF NOT EXISTS t_pginfo(id integer);
                     INSERT INTO t_pginfo(id) VALUES (1);
                     SELECT pg_create_physical_replication_slot('slot_pginfo_worker_test');",
                )
                .await?;
            drop(client);
            conn_task.await??;

            step_once(&mut ctx).await?;

            let second = subscriber.latest().value;
            match second {
                PgInfoState::Primary {
                    wal_lsn,
                    slots,
                    common,
                } => {
                    assert!(wal_lsn >= first_wal);
                    assert!(slots
                        .iter()
                        .any(|slot| slot.name == "slot_pginfo_worker_test"));
                    assert_eq!(common.sql, SqlStatus::Healthy);
                    assert_eq!(common.readiness, Readiness::Ready);
                }
                other => {
                    return Err(test_error(format!(
                        "expected primary after writes, got: {other:?}"
                    )));
                }
            }
            Ok(())
        }
        .await;

        let shutdown_result = shutdown_with_context("postgres", &mut handle).await;
        match (run_result, shutdown_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(err)) => Err(err),
            (Err(err), Err(clean_err)) => Err(test_error(format!("{err}; {clean_err}"))),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_maps_replica_when_polling_standby() -> TestResult {
        let postgres_bin = require_pg16_bin_for_real_tests("postgres")?;
        let initdb_bin = require_pg16_bin_for_real_tests("initdb")?;
        let basebackup_bin = require_pg16_bin_for_real_tests("pg_basebackup")?;

        let guard = NamespaceGuard::new("pginfo-replica-flow")?;
        let ns = guard.namespace()?;

        let primary_data = prepare_pgdata_dir(ns, "primary")?;
        let primary_socket = ns.child_dir("run/primary");
        let primary_logs = ns.child_dir("logs/primary");
        fs::create_dir_all(&primary_socket)?;
        fs::create_dir_all(&primary_logs)?;

        let primary_reservation = allocate_ports(1)?;
        let primary_port = primary_reservation.as_slice()[0];
        drop(primary_reservation);

        let mut primary = spawn_pg16(PgInstanceSpec {
            postgres_bin: postgres_bin.clone(),
            initdb_bin: initdb_bin.clone(),
            data_dir: primary_data.clone(),
            socket_dir: primary_socket.clone(),
            log_dir: primary_logs.clone(),
            port: primary_port,
            startup_timeout: Duration::from_secs(25),
        })
        .await?;

        let primary_dsn = format!(
            "host=127.0.0.1 port={} user=postgres dbname=postgres",
            primary_port
        );
        let mut replica: Option<PgHandle> = None;
        let run_result: TestResult = async {
            wait_for_postgres_ready(&primary_dsn, Duration::from_secs(20)).await?;

            let replica_data = ns.child_dir("pg16/replica/data");
            let replica_parent = replica_data
                .parent()
                .ok_or_else(|| test_error("replica data dir has no parent"))?;
            fs::create_dir_all(replica_parent)?;

            let output = tokio::process::Command::new(&basebackup_bin)
                .arg("-h")
                .arg("127.0.0.1")
                .arg("-p")
                .arg(primary_port.to_string())
                .arg("-D")
                .arg(&replica_data)
                .arg("-U")
                .arg("postgres")
                .arg("-Fp")
                .arg("-Xs")
                .output()
                .await?;
            if !output.status.success() {
                return Err(test_error(format!(
                    "pg_basebackup failed with status {}",
                    output.status
                )));
            }
            fs::write(replica_data.join("standby.signal"), b"")?;

            let replica_socket = ns.child_dir("run/replica");
            let replica_logs = ns.child_dir("logs/replica");
            fs::create_dir_all(&replica_socket)?;
            fs::create_dir_all(&replica_logs)?;

            let replica_reservation = allocate_ports(1)?;
            let replica_port = replica_reservation.as_slice()[0];
            drop(replica_reservation);

            let replica_spec = PgInstanceSpec {
                postgres_bin: postgres_bin.clone(),
                initdb_bin: initdb_bin.clone(),
                data_dir: replica_data.clone(),
                socket_dir: replica_socket,
                log_dir: replica_logs,
                port: replica_port,
                startup_timeout: Duration::from_secs(30),
            };
            let replica_conf_lines = vec![format!(
                "primary_conninfo = 'host=127.0.0.1 port={} user=postgres dbname=postgres'",
                primary_port
            )];
            // This test exercises PostgreSQL standby polling after pg_basebackup, so
            // it uses the explicit vanilla-Postgres exception path instead of the
            // pgtuskmaster-managed config flow.
            let replica_handle =
                spawn_pg16_for_vanilla_postgres(replica_spec, &replica_conf_lines).await?;
            replica = Some(replica_handle);

            let initial = PgInfoState::Unknown {
                common: PgInfoCommon {
                    worker: WorkerStatus::Starting,
                    sql: SqlStatus::Unknown,
                    readiness: Readiness::Unknown,
                    timeline: None,
                    pg_config: PgConfig {
                        port: None,
                        hot_standby: None,
                        primary_conninfo: None,
                        primary_slot_name: None,
                        extra: std::collections::BTreeMap::new(),
                    },
                    last_refresh_at: Some(UnixMillis(1)),
                },
            };
            let (publisher, subscriber) = new_state_channel(initial, UnixMillis(1));
            let (log, sink) = test_log_handle();
            let mut ctx = PgInfoWorkerCtx {
                self_id: MemberId("node-b".to_string()),
                postgres_conninfo: local_test_conninfo(replica_port),
                poll_interval: Duration::from_millis(50),
                publisher,
                log,
                last_emitted_sql_status: None,
            };

            let deadline = Instant::now() + Duration::from_secs(20);
            let snapshot = loop {
                step_once(&mut ctx).await?;

                let polled = subscriber.latest().value;
                if matches!(polled, PgInfoState::Replica { .. }) {
                    let transitions = sink
                        .take()
                        .into_iter()
                        .filter_map(|record| decode_app_event(&record).ok())
                        .filter(|event| event.header.name == "pginfo.sql_transition")
                        .collect::<Vec<_>>();
                    if transitions.is_empty() {
                        return Err(test_error(
                            "expected pginfo.sql_transition event during replica convergence"
                                .to_string(),
                        ));
                    }
                    break polled;
                }

                if Instant::now() >= deadline {
                    return Err(test_error(format!(
                        "timed out waiting for replica state, got: {polled:?}"
                    )));
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            };

            match snapshot {
                PgInfoState::Replica { common, .. } => {
                    assert_eq!(common.sql, SqlStatus::Healthy);
                    assert_eq!(common.readiness, Readiness::Ready);
                }
                other => {
                    return Err(test_error(format!(
                        "expected replica state, got: {other:?}"
                    )));
                }
            }
            Ok(())
        }
        .await;

        let mut cleanup_errors = Vec::new();
        if let Some(handle) = replica.as_mut() {
            if let Err(err) = shutdown_with_context("replica postgres", handle).await {
                cleanup_errors.push(err.to_string());
            }
        }
        if let Err(err) = shutdown_with_context("primary postgres", &mut primary).await {
            cleanup_errors.push(err.to_string());
        }

        if let Err(err) = run_result {
            if cleanup_errors.is_empty() {
                return Err(err);
            }
            return Err(test_error(format!(
                "{err}; cleanup errors: {}",
                cleanup_errors.join("; ")
            )));
        }

        if cleanup_errors.is_empty() {
            Ok(())
        } else {
            Err(test_error(format!(
                "cleanup errors: {}",
                cleanup_errors.join("; ")
            )))
        }
    }
}

--- END FILE: src/pginfo/worker.rs ---

