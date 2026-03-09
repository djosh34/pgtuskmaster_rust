use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use thiserror::Error;

use crate::{
    postgres_managed_conf::{MANAGED_RECOVERY_SIGNAL_NAME, MANAGED_STANDBY_SIGNAL_NAME},
    state::{SystemIdentifier, TimelineId, Version, WalLsn},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DataDirKind {
    Missing,
    Empty,
    Initialized,
    InvalidNonEmptyWithoutPgVersion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SignalFileState {
    None,
    Standby,
    Recovery,
    Conflicting,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LocalPhysicalState {
    pub(crate) data_dir_kind: DataDirKind,
    pub(crate) system_identifier: Option<SystemIdentifier>,
    pub(crate) pg_version: Option<Version>,
    pub(crate) control_file_state: Option<String>,
    pub(crate) timeline_id: Option<TimelineId>,
    pub(crate) durable_end_lsn: Option<WalLsn>,
    pub(crate) was_in_recovery: Option<bool>,
    pub(crate) signal_file_state: SignalFileState,
    pub(crate) eligible_for_bootstrap: bool,
    pub(crate) eligible_for_direct_follow: bool,
    pub(crate) eligible_for_rewind: bool,
    pub(crate) eligible_for_basebackup: bool,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum LocalPhysicalStateError {
    #[error("failed to inspect data dir `{path}`: {message}")]
    DataDirIo { path: PathBuf, message: String },
    #[error("pgdata path is not a directory: `{0}`")]
    NotDirectory(PathBuf),
    #[error("invalid PG_VERSION in `{path}`: {message}")]
    InvalidPgVersion { path: PathBuf, message: String },
    #[error("pg_controldata failed for `{data_dir}` using `{binary}`: {message}")]
    PgControlDataCommand {
        binary: PathBuf,
        data_dir: PathBuf,
        message: String,
    },
    #[error("pg_controldata output was invalid for `{data_dir}`: {message}")]
    PgControlDataParse { data_dir: PathBuf, message: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedPgControlData {
    system_identifier: SystemIdentifier,
    control_file_state: String,
    timeline_id: Option<TimelineId>,
    durable_end_lsn: Option<WalLsn>,
    was_in_recovery: bool,
}

pub(crate) fn inspect_local_physical_state(
    data_dir: &Path,
    postgres_binary: &Path,
) -> Result<LocalPhysicalState, LocalPhysicalStateError> {
    let kind = inspect_data_dir_kind(data_dir)?;
    let signal_file_state = inspect_signal_file_state(data_dir)?;

    match kind {
        DataDirKind::Missing => Ok(LocalPhysicalState {
            data_dir_kind: kind,
            system_identifier: None,
            pg_version: None,
            control_file_state: None,
            timeline_id: None,
            durable_end_lsn: None,
            was_in_recovery: None,
            signal_file_state,
            eligible_for_bootstrap: true,
            eligible_for_direct_follow: false,
            eligible_for_rewind: false,
            eligible_for_basebackup: true,
        }),
        DataDirKind::Empty => Ok(LocalPhysicalState {
            data_dir_kind: kind,
            system_identifier: None,
            pg_version: None,
            control_file_state: None,
            timeline_id: None,
            durable_end_lsn: None,
            was_in_recovery: None,
            signal_file_state,
            eligible_for_bootstrap: true,
            eligible_for_direct_follow: false,
            eligible_for_rewind: false,
            eligible_for_basebackup: true,
        }),
        DataDirKind::InvalidNonEmptyWithoutPgVersion => Ok(LocalPhysicalState {
            data_dir_kind: kind,
            system_identifier: None,
            pg_version: None,
            control_file_state: None,
            timeline_id: None,
            durable_end_lsn: None,
            was_in_recovery: None,
            signal_file_state,
            eligible_for_bootstrap: false,
            eligible_for_direct_follow: false,
            eligible_for_rewind: false,
            eligible_for_basebackup: false,
        }),
        DataDirKind::Initialized => {
            let pg_version = read_pg_version(data_dir)?;
            let parsed = run_pg_controldata(data_dir, postgres_binary)?;
            let signals_conflict = signal_file_state == SignalFileState::Conflicting;
            Ok(LocalPhysicalState {
                data_dir_kind: kind,
                system_identifier: Some(parsed.system_identifier),
                pg_version: Some(pg_version),
                control_file_state: Some(parsed.control_file_state),
                timeline_id: parsed.timeline_id,
                durable_end_lsn: parsed.durable_end_lsn,
                was_in_recovery: Some(parsed.was_in_recovery),
                signal_file_state,
                eligible_for_bootstrap: false,
                eligible_for_direct_follow: !signals_conflict,
                eligible_for_rewind: !signals_conflict,
                eligible_for_basebackup: true,
            })
        }
    }
}

fn inspect_data_dir_kind(data_dir: &Path) -> Result<DataDirKind, LocalPhysicalStateError> {
    match fs::metadata(data_dir) {
        Ok(meta) => {
            if !meta.is_dir() {
                return Err(LocalPhysicalStateError::NotDirectory(data_dir.to_path_buf()));
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DataDirKind::Missing);
        }
        Err(err) => {
            return Err(LocalPhysicalStateError::DataDirIo {
                path: data_dir.to_path_buf(),
                message: err.to_string(),
            });
        }
    }

    if data_dir.join("PG_VERSION").exists() {
        return Ok(DataDirKind::Initialized);
    }

    let mut entries =
        fs::read_dir(data_dir).map_err(|err| LocalPhysicalStateError::DataDirIo {
            path: data_dir.to_path_buf(),
            message: err.to_string(),
        })?;
    if entries.next().is_none() {
        Ok(DataDirKind::Empty)
    } else {
        Ok(DataDirKind::InvalidNonEmptyWithoutPgVersion)
    }
}

fn inspect_signal_file_state(data_dir: &Path) -> Result<SignalFileState, LocalPhysicalStateError> {
    let standby_present = file_exists(data_dir.join(MANAGED_STANDBY_SIGNAL_NAME).as_path())?;
    let recovery_present = file_exists(data_dir.join(MANAGED_RECOVERY_SIGNAL_NAME).as_path())?;
    Ok(match (standby_present, recovery_present) {
        (false, false) => SignalFileState::None,
        (true, false) => SignalFileState::Standby,
        (false, true) => SignalFileState::Recovery,
        (true, true) => SignalFileState::Conflicting,
    })
}

fn file_exists(path: &Path) -> Result<bool, LocalPhysicalStateError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(LocalPhysicalStateError::DataDirIo {
            path: path.to_path_buf(),
            message: err.to_string(),
        }),
    }
}

fn read_pg_version(data_dir: &Path) -> Result<Version, LocalPhysicalStateError> {
    let path = data_dir.join("PG_VERSION");
    let raw = fs::read_to_string(&path).map_err(|err| LocalPhysicalStateError::DataDirIo {
        path: path.clone(),
        message: err.to_string(),
    })?;
    let trimmed = raw.trim();
    let parsed = trimmed
        .parse::<u64>()
        .map_err(|err| LocalPhysicalStateError::InvalidPgVersion {
            path,
            message: err.to_string(),
        })?;
    Ok(Version(parsed))
}

fn run_pg_controldata(
    data_dir: &Path,
    postgres_binary: &Path,
) -> Result<ParsedPgControlData, LocalPhysicalStateError> {
    let binary = derived_pg_controldata_path(postgres_binary);
    let output = Command::new(&binary)
        .arg(data_dir)
        .output()
        .map_err(|err| LocalPhysicalStateError::PgControlDataCommand {
            binary: binary.clone(),
            data_dir: data_dir.to_path_buf(),
            message: err.to_string(),
        })?;
    if !output.status.success() {
        return Err(LocalPhysicalStateError::PgControlDataCommand {
            binary,
            data_dir: data_dir.to_path_buf(),
            message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }
    let stdout = String::from_utf8(output.stdout).map_err(|err| {
        LocalPhysicalStateError::PgControlDataParse {
            data_dir: data_dir.to_path_buf(),
            message: err.to_string(),
        }
    })?;
    parse_pg_controldata_output(stdout.as_str()).map_err(|message| {
        LocalPhysicalStateError::PgControlDataParse {
            data_dir: data_dir.to_path_buf(),
            message,
        }
    })
}

fn derived_pg_controldata_path(postgres_binary: &Path) -> PathBuf {
    match postgres_binary.parent() {
        Some(parent) => {
            let sibling = parent.join("pg_controldata");
            if sibling.is_file() {
                sibling
            } else {
                PathBuf::from("pg_controldata")
            }
        }
        None => PathBuf::from("pg_controldata"),
    }
}

fn parse_pg_controldata_output(output: &str) -> Result<ParsedPgControlData, String> {
    let key_values = output
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect::<std::collections::BTreeMap<_, _>>();

    let system_identifier = key_values
        .get("Database system identifier")
        .ok_or_else(|| "missing Database system identifier".to_string())
        .and_then(|value| parse_u64_field(value, "Database system identifier"))
        .map(SystemIdentifier)?;

    let control_file_state = key_values
        .get("Database cluster state")
        .cloned()
        .ok_or_else(|| "missing Database cluster state".to_string())?;

    let checkpoint_timeline = key_values
        .get("Latest checkpoint's TimeLineID")
        .map(|value| parse_u32_field(value, "Latest checkpoint's TimeLineID"))
        .transpose()?
        .map(TimelineId);
    let recovery_timeline = key_values
        .get("Min recovery ending loc's timeline")
        .map(|value| parse_u32_field(value, "Min recovery ending loc's timeline"))
        .transpose()?
        .map(TimelineId);

    let durable_end_lsn = key_values
        .get("Minimum recovery ending location")
        .or_else(|| key_values.get("Latest checkpoint location"))
        .map(|value| parse_wal_lsn(value))
        .transpose()?;

    Ok(ParsedPgControlData {
        system_identifier,
        control_file_state: control_file_state.clone(),
        timeline_id: checkpoint_timeline.or(recovery_timeline),
        durable_end_lsn,
        was_in_recovery: control_file_state.contains("recovery"),
    })
}

fn parse_u64_field(raw: &str, field: &str) -> Result<u64, String> {
    raw.parse::<u64>()
        .map_err(|err| format!("invalid {field}: {err}"))
}

fn parse_u32_field(raw: &str, field: &str) -> Result<u32, String> {
    raw.parse::<u32>()
        .map_err(|err| format!("invalid {field}: {err}"))
}

fn parse_wal_lsn(raw: &str) -> Result<WalLsn, String> {
    let Some((high, low)) = raw.split_once('/') else {
        return Err(format!("invalid WAL LSN `{raw}`"));
    };
    let high = u64::from_str_radix(high, 16)
        .map_err(|err| format!("invalid WAL LSN `{raw}` high bits: {err}"))?;
    let low = u64::from_str_radix(low, 16)
        .map_err(|err| format!("invalid WAL LSN `{raw}` low bits: {err}"))?;
    Ok(WalLsn((high << 32) | low))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        inspect_local_physical_state, parse_pg_controldata_output, DataDirKind, SignalFileState,
    };
    use crate::{
        postgres_managed_conf::{MANAGED_RECOVERY_SIGNAL_NAME, MANAGED_STANDBY_SIGNAL_NAME},
        state::{SystemIdentifier, TimelineId, WalLsn},
    };

    fn temp_dir(label: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!("pgtm-local-physical-{label}-{unique}"));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    #[test]
    fn parses_pg_controldata_output() -> Result<(), Box<dyn std::error::Error>> {
        let parsed = parse_pg_controldata_output(
            "\
Database system identifier:           7374392058184458932
Database cluster state:               shut down in recovery
Latest checkpoint location:           0/16B6C50
Latest checkpoint's TimeLineID:       4
Minimum recovery ending location:     0/16B6D00
Min recovery ending loc's timeline:   4
",
        )?;
        assert_eq!(parsed.system_identifier, SystemIdentifier(7_374_392_058_184_458_932));
        assert_eq!(parsed.timeline_id, Some(TimelineId(4)));
        assert_eq!(parsed.durable_end_lsn, Some(WalLsn(0x00000000_016B6D00)));
        assert!(parsed.was_in_recovery);
        Ok(())
    }

    #[test]
    fn classifies_invalid_non_empty_directory() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir("invalid-non-empty")?;
        fs::write(dir.join("orphan"), "data")?;
        let inspected = inspect_local_physical_state(&dir, Path::new("/usr/bin/postgres"))?;
        assert_eq!(inspected.data_dir_kind, DataDirKind::InvalidNonEmptyWithoutPgVersion);
        assert_eq!(inspected.signal_file_state, SignalFileState::None);
        assert!(!inspected.eligible_for_bootstrap);
        Ok(())
    }

    #[test]
    fn classifies_empty_directory() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir("empty")?;
        let inspected = inspect_local_physical_state(&dir, Path::new("/usr/bin/postgres"))?;
        assert_eq!(inspected.data_dir_kind, DataDirKind::Empty);
        assert!(inspected.eligible_for_bootstrap);
        Ok(())
    }

    #[test]
    fn detects_conflicting_signal_files_without_pg_controldata() -> Result<(), Box<dyn std::error::Error>>
    {
        let dir = temp_dir("conflicting-signals")?;
        fs::write(dir.join("orphan"), "data")?;
        fs::write(dir.join(MANAGED_STANDBY_SIGNAL_NAME), "")?;
        fs::write(dir.join(MANAGED_RECOVERY_SIGNAL_NAME), "")?;
        let inspected = inspect_local_physical_state(&dir, Path::new("/usr/bin/postgres"))?;
        assert_eq!(inspected.signal_file_state, SignalFileState::Conflicting);
        Ok(())
    }

    #[test]
    fn missing_directory_is_bootstrap_eligible() -> Result<(), Box<dyn std::error::Error>> {
        let dir = temp_dir("missing-root")?;
        let missing = dir.join("missing");
        let inspected = inspect_local_physical_state(&missing, Path::new("/usr/bin/postgres"))?;
        assert_eq!(inspected.data_dir_kind, DataDirKind::Missing);
        assert!(inspected.eligible_for_bootstrap);
        assert_eq!(inspected.pg_version, None);
        Ok(())
    }
}
