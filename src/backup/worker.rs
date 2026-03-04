use crate::{
    config::RuntimeConfig,
    process::{
        jobs::ProcessError,
        state::{ProcessJobKind, ProcessJobRequest},
    },
    state::JobId,
};

use crate::process::jobs::{
    PgBackRestArchiveGetSpec, PgBackRestArchivePushSpec, PgBackRestBackupSpec, PgBackRestCheckSpec,
    PgBackRestInfoSpec, PgBackRestRestoreSpec, PgBackRestVersionSpec,
};

pub(crate) fn pgbackrest_version_job(id: JobId) -> ProcessJobRequest {
    ProcessJobRequest {
        id,
        kind: ProcessJobKind::PgBackRestVersion(PgBackRestVersionSpec {}),
    }
}

pub(crate) fn pgbackrest_info_job(cfg: &RuntimeConfig, id: JobId) -> Result<ProcessJobRequest, ProcessError> {
    let (stanza, repo, options) = pgbackrest_required_inputs(cfg, "info")?;
    Ok(ProcessJobRequest {
        id,
        kind: ProcessJobKind::PgBackRestInfo(PgBackRestInfoSpec {
            stanza,
            repo,
            options,
            timeout_ms: None,
        }),
    })
}

pub(crate) fn pgbackrest_check_job(cfg: &RuntimeConfig, id: JobId) -> Result<ProcessJobRequest, ProcessError> {
    let (stanza, repo, options) = pgbackrest_required_inputs(cfg, "check")?;
    Ok(ProcessJobRequest {
        id,
        kind: ProcessJobKind::PgBackRestCheck(PgBackRestCheckSpec {
            stanza,
            repo,
            options,
            timeout_ms: None,
        }),
    })
}

pub(crate) fn pgbackrest_backup_job(cfg: &RuntimeConfig, id: JobId) -> Result<ProcessJobRequest, ProcessError> {
    let (stanza, repo, options) = pgbackrest_required_inputs(cfg, "backup")?;
    Ok(ProcessJobRequest {
        id,
        kind: ProcessJobKind::PgBackRestBackup(PgBackRestBackupSpec {
            stanza,
            repo,
            options,
            timeout_ms: None,
        }),
    })
}

pub(crate) fn pgbackrest_restore_job(cfg: &RuntimeConfig, id: JobId) -> Result<ProcessJobRequest, ProcessError> {
    let (stanza, repo, options) = pgbackrest_required_inputs(cfg, "restore")?;
    Ok(ProcessJobRequest {
        id,
        kind: ProcessJobKind::PgBackRestRestore(PgBackRestRestoreSpec {
            stanza,
            repo,
            pg1_path: cfg.postgres.data_dir.clone(),
            options,
            timeout_ms: None,
        }),
    })
}

pub(crate) fn pgbackrest_archive_push_job(
    cfg: &RuntimeConfig,
    id: JobId,
    wal_path: String,
) -> Result<ProcessJobRequest, ProcessError> {
    let (stanza, repo, options) = pgbackrest_required_inputs(cfg, "archive_push")?;
    if wal_path.trim().is_empty() {
        return Err(ProcessError::InvalidSpec(
            "pgbackrest archive_push wal_path must not be empty".to_string(),
        ));
    }
    Ok(ProcessJobRequest {
        id,
        kind: ProcessJobKind::PgBackRestArchivePush(PgBackRestArchivePushSpec {
            stanza,
            repo,
            pg1_path: cfg.postgres.data_dir.clone(),
            wal_path,
            options,
            timeout_ms: None,
        }),
    })
}

pub(crate) fn pgbackrest_archive_get_job(
    cfg: &RuntimeConfig,
    id: JobId,
    wal_segment: String,
    destination_path: String,
) -> Result<ProcessJobRequest, ProcessError> {
    let (stanza, repo, options) = pgbackrest_required_inputs(cfg, "archive_get")?;
    if wal_segment.trim().is_empty() {
        return Err(ProcessError::InvalidSpec(
            "pgbackrest archive_get wal_segment must not be empty".to_string(),
        ));
    }
    if destination_path.trim().is_empty() {
        return Err(ProcessError::InvalidSpec(
            "pgbackrest archive_get destination_path must not be empty".to_string(),
        ));
    }
    Ok(ProcessJobRequest {
        id,
        kind: ProcessJobKind::PgBackRestArchiveGet(PgBackRestArchiveGetSpec {
            stanza,
            repo,
            pg1_path: cfg.postgres.data_dir.clone(),
            wal_segment,
            destination_path,
            options,
            timeout_ms: None,
        }),
    })
}

fn pgbackrest_required_inputs(
    cfg: &RuntimeConfig,
    op: &'static str,
) -> Result<(String, String, Vec<String>), ProcessError> {
    let pg_cfg = cfg.backup.pgbackrest.as_ref().ok_or_else(|| {
        ProcessError::InvalidSpec("backup.pgbackrest config block is required".to_string())
    })?;

    let stanza = pg_cfg.stanza.clone().ok_or_else(|| {
        ProcessError::InvalidSpec("backup.pgbackrest.stanza is required".to_string())
    })?;
    if stanza.trim().is_empty() {
        return Err(ProcessError::InvalidSpec(
            "backup.pgbackrest.stanza must not be empty".to_string(),
        ));
    }

    let repo = pg_cfg.repo.clone().ok_or_else(|| {
        ProcessError::InvalidSpec("backup.pgbackrest.repo is required".to_string())
    })?;
    if repo.trim().is_empty() {
        return Err(ProcessError::InvalidSpec(
            "backup.pgbackrest.repo must not be empty".to_string(),
        ));
    }

    let options = match op {
        "backup" => pg_cfg.options.backup.clone(),
        "info" => pg_cfg.options.info.clone(),
        "check" => pg_cfg.options.check.clone(),
        "restore" => pg_cfg.options.restore.clone(),
        "archive_push" => pg_cfg.options.archive_push.clone(),
        "archive_get" => pg_cfg.options.archive_get.clone(),
        _ => {
            return Err(ProcessError::InvalidSpec(format!(
                "unknown pgbackrest operation kind: {op}"
            )))
        }
    };

    Ok((stanza, repo, options))
}

pub(crate) fn validate_pgbackrest_enabled_config(cfg: &RuntimeConfig) -> Result<(), ProcessError> {
    let job_id = JobId("pgbackrest-config-validate".to_string());

    let version = pgbackrest_version_job(job_id.clone());
    let _ = crate::process::worker::build_command(
        &cfg.process,
        &version.id,
        &version.kind,
        true,
    )?;

    let info = pgbackrest_info_job(cfg, job_id.clone())?;
    let _ = crate::process::worker::build_command(&cfg.process, &info.id, &info.kind, true)?;

    let check = pgbackrest_check_job(cfg, job_id.clone())?;
    let _ = crate::process::worker::build_command(&cfg.process, &check.id, &check.kind, true)?;

    let backup = pgbackrest_backup_job(cfg, job_id.clone())?;
    let _ = crate::process::worker::build_command(&cfg.process, &backup.id, &backup.kind, true)?;

    let restore = pgbackrest_restore_job(cfg, job_id.clone())?;
    let _ = crate::process::worker::build_command(&cfg.process, &restore.id, &restore.kind, true)?;

    let archive_push = pgbackrest_archive_push_job(
        cfg,
        job_id.clone(),
        "/tmp/000000010000000000000001".to_string(),
    )?;
    let _ = crate::process::worker::build_command(
        &cfg.process,
        &archive_push.id,
        &archive_push.kind,
        true,
    )?;

    let archive_get = pgbackrest_archive_get_job(
        cfg,
        job_id,
        "000000010000000000000001".to_string(),
        "/tmp/wal".to_string(),
    )?;
    let _ = crate::process::worker::build_command(
        &cfg.process,
        &archive_get.id,
        &archive_get.kind,
        true,
    )?;

    Ok(())
}
