use thiserror::Error;

use crate::state::MemberId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsKey {
    Member(MemberId),
    Leader,
    Switchover,
    Config,
    InitLock,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DcsKeyParseError {
    #[error("path `{path}` does not start with scope prefix `{scope_prefix}`")]
    InvalidScopePrefix { path: String, scope_prefix: String },
    #[error("path `{0}` is malformed")]
    MalformedPath(String),
    #[error("member id segment is missing in path `{0}`")]
    MissingMemberId(String),
    #[error("unknown key path `{0}`")]
    UnknownKey(String),
}

pub(crate) fn key_from_path(scope: &str, full_path: &str) -> Result<DcsKey, DcsKeyParseError> {
    let scope = scope.trim_matches('/');
    let expected_prefix = format!("/{scope}/");
    if !full_path.starts_with(&expected_prefix) {
        return Err(DcsKeyParseError::InvalidScopePrefix {
            path: full_path.to_string(),
            scope_prefix: expected_prefix,
        });
    }

    let suffix = &full_path[expected_prefix.len()..];
    let parts: Vec<&str> = suffix.split('/').collect();
    match parts.as_slice() {
        ["leader"] => Ok(DcsKey::Leader),
        ["switchover"] => Ok(DcsKey::Switchover),
        ["config"] => Ok(DcsKey::Config),
        ["init"] => Ok(DcsKey::InitLock),
        ["member", member_id] => {
            if member_id.is_empty() {
                return Err(DcsKeyParseError::MissingMemberId(full_path.to_string()));
            }
            Ok(DcsKey::Member(MemberId((*member_id).to_string())))
        }
        [] | [""] => Err(DcsKeyParseError::MalformedPath(full_path.to_string())),
        ["member"] => Err(DcsKeyParseError::MissingMemberId(full_path.to_string())),
        _ => Err(DcsKeyParseError::UnknownKey(full_path.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::{key_from_path, DcsKey, DcsKeyParseError};
    use crate::state::MemberId;

    #[test]
    fn parses_supported_paths() {
        assert_eq!(
            key_from_path("scope-a", "/scope-a/member/node-a"),
            Ok(DcsKey::Member(MemberId("node-a".to_string())))
        );
        assert_eq!(
            key_from_path("scope-a", "/scope-a/leader"),
            Ok(DcsKey::Leader)
        );
        assert_eq!(
            key_from_path("scope-a", "/scope-a/switchover"),
            Ok(DcsKey::Switchover)
        );
        assert_eq!(
            key_from_path("scope-a", "/scope-a/config"),
            Ok(DcsKey::Config)
        );
        assert_eq!(
            key_from_path("scope-a", "/scope-a/init"),
            Ok(DcsKey::InitLock)
        );
    }

    #[test]
    fn rejects_wrong_scope() {
        let parsed = key_from_path("scope-a", "/scope-b/leader");
        assert!(matches!(
            parsed,
            Err(DcsKeyParseError::InvalidScopePrefix { .. })
        ));
    }

    #[test]
    fn rejects_missing_member_id() {
        let parsed = key_from_path("scope-a", "/scope-a/member/");
        assert_eq!(
            parsed,
            Err(DcsKeyParseError::MissingMemberId(
                "/scope-a/member/".to_string()
            ))
        );
    }

    #[test]
    fn rejects_unknown_and_extra_segments() {
        assert_eq!(
            key_from_path("scope-a", "/scope-a/nope"),
            Err(DcsKeyParseError::UnknownKey("/scope-a/nope".to_string()))
        );
        assert_eq!(
            key_from_path("scope-a", "/scope-a/leader/extra"),
            Err(DcsKeyParseError::UnknownKey(
                "/scope-a/leader/extra".to_string()
            ))
        );
    }
}
