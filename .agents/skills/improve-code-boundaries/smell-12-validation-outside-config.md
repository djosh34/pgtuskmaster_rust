# Validation Outside config

All config must have been done inside config.
So any 'validate', 'ensure' function outside config is bad and must be refactored.
Validate once and only once. Make sure the full end results is in the validated config
(so no extra absolute path making after config)

Then once validated, NEVER VALIDATE AGAIN
Also NEVER VALIDATE OUTSIDE src/config_v2/parser/, which MUST STAY PRIVATE and NOT pub


Example:


Why is this done outside config?

Wrongs with this:

- Result is not a validated config, instead just result. This means that validation could be forgotten instead of rust's guarantee that it must have been done
- This validation is outside src/config_v2/parser
- Far too much use of format!

```rust
pub(crate) fn ensure_start_paths(cfg: &RuntimeConfigV2) -> Result<(), ProcessError> {
    for (field, path) in [
        (
            "process.binaries.overrides.postgres",
            &cfg.binaries.postgres,
        ),
        ("process.binaries.overrides.pg_ctl", &cfg.binaries.pg_ctl),
        ("process.binaries.overrides.initdb", &cfg.binaries.initdb),
        (
            "process.binaries.overrides.pg_rewind",
            &cfg.binaries.pg_rewind,
        ),
        (
            "process.binaries.overrides.pg_basebackup",
            &cfg.binaries.pg_basebackup,
        ),
        ("process.binaries.overrides.psql", &cfg.binaries.psql),
    ] {
        if !path.is_absolute() {
            return Err(ProcessError::InvalidSpec(format!(
                "{field} must be an absolute path, got `{}`",
                path.display()
            )));
        }
    }

    let data_dir = &cfg.postgres.data_dir;
    if let Some(parent) = data_dir.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            ProcessError::InvalidSpec(format!(
                "failed to create postgres data dir parent `{}`: {err}",
                parent.display()
            ))
        })?;
    }

    fs::create_dir_all(data_dir).map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "failed to create postgres data dir `{}`: {err}",
            data_dir.display()
        ))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700)).map_err(|err| {
            ProcessError::InvalidSpec(format!(
                "failed to set postgres data dir permissions on `{}`: {err}",
                data_dir.display()
            ))
        })?;
    }

    fs::create_dir_all(&cfg.postgres.socket_dir).map_err(|err| {
        ProcessError::InvalidSpec(format!(
            "failed to create postgres socket dir `{}`: {err}",
            cfg.postgres.socket_dir.display()
        ))
    })?;

    if let Some(log_parent) = cfg.postgres.log_file.parent() {
        fs::create_dir_all(log_parent).map_err(|err| {
            ProcessError::InvalidSpec(format!(
                "failed to create postgres log dir `{}`: {err}",
                log_parent.display()
            ))
        })?;
    }

    Ok(())
}

```
