use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    backup::{pgbackrest, ArchiveGetInput, ArchivePushInput, BackupOperation},
    config::{BackupProvider, RuntimeConfig},
};

const ARCHIVE_COMMAND_CONFIG_NAME: &str = "pgtm.pgbackrest.archive.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ArchiveCommandConfig {
    pub(crate) pgbackrest_bin: PathBuf,
    pub(crate) stanza: String,
    pub(crate) repo: String,
    pub(crate) pg1_path: PathBuf,
    pub(crate) archive_push_options: Vec<String>,
    pub(crate) archive_get_options: Vec<String>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ArchiveCommandError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("invalid config: {message}")]
    InvalidConfig { message: String },
    #[error("decode error: {message}")]
    Decode { message: String },
    #[error("pgbackrest render error: {message}")]
    Render { message: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderedCommand {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<String>,
}

pub(crate) fn archive_command_config_path(pgdata: &Path) -> PathBuf {
    pgdata.join(ARCHIVE_COMMAND_CONFIG_NAME)
}

pub(crate) fn materialize_archive_command_config(
    cfg: &RuntimeConfig,
) -> Result<PathBuf, ArchiveCommandError> {
    if !cfg.backup.enabled {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "backup.enabled must be true to materialize archive command config".to_string(),
        });
    }
    if cfg.postgres.data_dir.as_os_str().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "postgres.data_dir must not be empty".to_string(),
        });
    }
    match cfg.backup.provider {
        BackupProvider::Pgbackrest => {}
    }

    let pgbackrest_bin = cfg
        .process
        .binaries
        .pgbackrest
        .clone()
        .ok_or_else(|| ArchiveCommandError::InvalidConfig {
            message: "process.binaries.pgbackrest must be configured when backup.enabled is true"
                .to_string(),
        })?;
    if pgbackrest_bin.as_os_str().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "process.binaries.pgbackrest must not be empty".to_string(),
        });
    }

    let pg_cfg = cfg
        .backup
        .pgbackrest
        .as_ref()
        .ok_or_else(|| ArchiveCommandError::InvalidConfig {
            message: "backup.pgbackrest must be configured when backup.enabled is true".to_string(),
        })?;

    let stanza = pg_cfg
        .stanza
        .clone()
        .ok_or_else(|| ArchiveCommandError::InvalidConfig {
            message: "backup.pgbackrest.stanza must be configured when backup.enabled is true"
                .to_string(),
        })?;
    if stanza.trim().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "backup.pgbackrest.stanza must not be empty".to_string(),
        });
    }

    let repo = pg_cfg
        .repo
        .clone()
        .ok_or_else(|| ArchiveCommandError::InvalidConfig {
            message: "backup.pgbackrest.repo must be configured when backup.enabled is true"
                .to_string(),
        })?;
    if repo.trim().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "backup.pgbackrest.repo must not be empty".to_string(),
        });
    }

    let cfg_path = archive_command_config_path(&cfg.postgres.data_dir);
    let config = ArchiveCommandConfig {
        pgbackrest_bin: absolutize_path(pgbackrest_bin.as_path())?,
        stanza,
        repo,
        pg1_path: absolutize_path(cfg.postgres.data_dir.as_path())?,
        archive_push_options: pg_cfg.options.archive_push.clone(),
        archive_get_options: pg_cfg.options.archive_get.clone(),
    };

    let json = serde_json::to_vec(&config).map_err(|err| ArchiveCommandError::Decode {
        message: format!("failed to serialize archive command config: {err}"),
    })?;
    write_atomic(cfg_path.as_path(), &json, Some(0o644))?;
    Ok(cfg_path)
}

pub(crate) fn render_archive_push_from_pgdata(
    pgdata: &Path,
    wal_path: &str,
) -> Result<RenderedCommand, ArchiveCommandError> {
    if pgdata.as_os_str().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "pgdata must not be empty".to_string(),
        });
    }
    if wal_path.trim().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "wal_path must not be empty".to_string(),
        });
    }
    let config = load_config(pgdata)?;
    let template = pgbackrest::render(BackupOperation::ArchivePush(ArchivePushInput {
        stanza: config.stanza.clone(),
        repo: config.repo.clone(),
        pg1_path: config.pg1_path.clone(),
        wal_path: wal_path.to_string(),
        options: config.archive_push_options.clone(),
    }))
    .map_err(|err| ArchiveCommandError::Render { message: err })?;
    Ok(RenderedCommand {
        program: config.pgbackrest_bin,
        args: template.args,
    })
}

pub(crate) fn render_archive_get_from_pgdata(
    pgdata: &Path,
    wal_segment: &str,
    destination_path: &str,
) -> Result<RenderedCommand, ArchiveCommandError> {
    if pgdata.as_os_str().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "pgdata must not be empty".to_string(),
        });
    }
    if wal_segment.trim().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "wal_segment must not be empty".to_string(),
        });
    }
    if destination_path.trim().is_empty() {
        return Err(ArchiveCommandError::InvalidConfig {
            message: "destination_path must not be empty".to_string(),
        });
    }
    let config = load_config(pgdata)?;
    let template = pgbackrest::render(BackupOperation::ArchiveGet(ArchiveGetInput {
        stanza: config.stanza.clone(),
        repo: config.repo.clone(),
        pg1_path: config.pg1_path.clone(),
        wal_segment: wal_segment.to_string(),
        destination_path: destination_path.to_string(),
        options: config.archive_get_options.clone(),
    }))
    .map_err(|err| ArchiveCommandError::Render { message: err })?;
    Ok(RenderedCommand {
        program: config.pgbackrest_bin,
        args: template.args,
    })
}

fn load_config(pgdata: &Path) -> Result<ArchiveCommandConfig, ArchiveCommandError> {
    let path = archive_command_config_path(pgdata);
    let raw = fs::read(&path).map_err(|err| ArchiveCommandError::Io {
        message: format!("failed to read archive command config {}: {err}", path.display()),
    })?;
    serde_json::from_slice(&raw).map_err(|err| ArchiveCommandError::Decode {
        message: format!(
            "failed to decode archive command config {}: {err}",
            path.display()
        ),
    })
}

fn absolutize_path(path: &Path) -> Result<PathBuf, ArchiveCommandError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().map_err(|err| ArchiveCommandError::Io {
        message: format!("failed to read current_dir: {err}"),
    })?;
    Ok(cwd.join(path))
}

fn now_millis() -> Result<u64, ArchiveCommandError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| ArchiveCommandError::Io {
            message: format!("system clock before unix epoch: {err}"),
        })?;
    u64::try_from(elapsed.as_millis()).map_err(|err| ArchiveCommandError::Io {
        message: format!("millis conversion failed: {err}"),
    })
}

fn write_atomic(path: &Path, contents: &[u8], mode: Option<u32>) -> Result<(), ArchiveCommandError> {
    let parent = path.parent().ok_or_else(|| ArchiveCommandError::Io {
        message: format!("path has no parent: {}", path.display()),
    })?;
    fs::create_dir_all(parent).map_err(|err| ArchiveCommandError::Io {
        message: format!("failed to create dir {}: {err}", parent.display()),
    })?;

    let (pid, millis) = (std::process::id(), now_millis()?);
    let file_name = match path.file_name().and_then(|s| s.to_str()) {
        Some(name) if !name.is_empty() => name,
        _ => "managed",
    };
    let tmp = parent.join(format!(".{file_name}.tmp.{pid}.{millis}"));

    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&tmp)
        .map_err(|err| ArchiveCommandError::Io {
            message: format!("failed to create temp file {}: {err}", tmp.display()),
        })?;

    use std::io::Write;
    file.write_all(contents)
        .map_err(|err| ArchiveCommandError::Io {
            message: format!("failed to write temp file {}: {err}", tmp.display()),
        })?;
    file.sync_all().map_err(|err| ArchiveCommandError::Io {
        message: format!("failed to sync temp file {}: {err}", tmp.display()),
    })?;

    #[cfg(unix)]
    if let Some(mode) = mode {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(mode)).map_err(|err| {
            ArchiveCommandError::Io {
                message: format!("failed to set permissions on {}: {err}", tmp.display()),
            }
        })?;
    }

    fs::rename(&tmp, path).or_else(|err| {
        if path.exists() {
            fs::remove_file(path).map_err(|remove_err| ArchiveCommandError::Io {
                message: format!(
                    "failed to remove existing {} after rename error ({err}): {remove_err}",
                    path.display()
                ),
            })?;
            fs::rename(&tmp, path).map_err(|err| ArchiveCommandError::Io {
                message: format!("failed to rename {} to {}: {err}", tmp.display(), path.display()),
            })
        } else {
            Err(ArchiveCommandError::Io {
                message: format!("failed to rename {} to {}: {err}", tmp.display(), path.display()),
            })
        }
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_is_under_pgdata() {
        let pgdata = PathBuf::from("/tmp/pgdata-a");
        let path = archive_command_config_path(pgdata.as_path());
        assert!(path.ends_with(ARCHIVE_COMMAND_CONFIG_NAME));
    }

    #[test]
    fn render_errors_when_config_missing() {
        let pgdata = std::env::temp_dir().join(format!(
            "pgtuskmaster-archive-cfg-missing-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&pgdata);
        let _ = fs::create_dir_all(&pgdata);

        let result = render_archive_push_from_pgdata(pgdata.as_path(), "/tmp/wal");
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(
                msg.contains(ARCHIVE_COMMAND_CONFIG_NAME),
                "expected error to reference config file name, got: {msg}"
            );
        }

        let _ = fs::remove_dir_all(&pgdata);
    }
}
