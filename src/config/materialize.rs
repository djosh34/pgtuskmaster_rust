use std::path::PathBuf;

use thiserror::Error;

use crate::config::{InlineOrPath, SecretSource};

#[derive(Debug, Error)]
pub enum ConfigMaterializeError {
    #[error("failed to read `{field}` from {path}: {source}")]
    Io {
        field: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("environment variable `{env}` for `{field}` is not set")]
    MissingEnv { field: String, env: String },
    #[error("environment variable `{env}` for `{field}` must not be empty")]
    EmptyEnv { field: String, env: String },
}

pub fn resolve_inline_or_path_string(
    field: &str,
    source: &InlineOrPath,
) -> Result<String, ConfigMaterializeError> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => {
            std::fs::read_to_string(path).map_err(|source| ConfigMaterializeError::Io {
                field: field.to_string(),
                path: path.clone(),
                source,
            })
        }
        InlineOrPath::Inline { content } => Ok(content.clone()),
    }
}

pub fn resolve_inline_or_path_bytes(
    field: &str,
    source: &InlineOrPath,
) -> Result<Vec<u8>, ConfigMaterializeError> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => std::fs::read(path)
            .map_err(|source| ConfigMaterializeError::Io {
                field: field.to_string(),
                path: path.clone(),
                source,
            }),
        InlineOrPath::Inline { content } => Ok(content.as_bytes().to_vec()),
    }
}

pub fn resolve_secret_string(
    field: &str,
    secret: &SecretSource,
) -> Result<String, ConfigMaterializeError> {
    let value = match secret {
        SecretSource::Path(path) | SecretSource::PathConfig { path } => {
            std::fs::read_to_string(path).map_err(|source| ConfigMaterializeError::Io {
                field: field.to_string(),
                path: path.clone(),
                source,
            })?
        }
        SecretSource::Inline { content } => content.clone(),
        SecretSource::Env { env } => {
            let value = std::env::var(env).map_err(|err| match err {
                std::env::VarError::NotPresent => ConfigMaterializeError::MissingEnv {
                    field: field.to_string(),
                    env: env.clone(),
                },
                std::env::VarError::NotUnicode(_) => ConfigMaterializeError::EmptyEnv {
                    field: field.to_string(),
                    env: env.clone(),
                },
            })?;
            if value.trim().is_empty() {
                return Err(ConfigMaterializeError::EmptyEnv {
                    field: field.to_string(),
                    env: env.clone(),
                });
            }
            value
        }
    };

    Ok(value.trim_end_matches(['\n', '\r']).to_string())
}
