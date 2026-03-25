# mixed-responsibilities

- some functions mix responsibilities, and should be untangled


## Example 1

This function does two things:
- Create ManagedPostgresConfig, which should be separate
- Write that config to a file, which should have been an impl method on top of ManagedPostgresConfig

rendering the config, and making the config should be split.

```rust

pub(crate) fn materialize_managed_postgres_config(
  cfg: &RuntimeConfigV2,
  start_intent: &ManagedPostgresStartIntent,
) -> Result<ManagedPostgresConfig, ManagedPostgresError> {
  let data_dir = cfg.postgres.data_dir.as_path();
  if data_dir.as_os_str().is_empty() {
    return Err(ManagedPostgresError::InvalidConfig {
      message: "postgres.data_dir must not be empty".to_string(),
    });
  }

  let managed_hba = absolutize_path(&cfg.postgres.pg_hba_file)?;
  let managed_ident = absolutize_path(&cfg.postgres.pg_ident_file)?;
  let managed_postgresql_conf =
          absolutize_path(&cfg.postgres.data_dir.join(MANAGED_POSTGRESQL_CONF_NAME))?;
  let managed_standby_passfile =
          absolutize_path(&managed_standby_passfile_path(&cfg.postgres.data_dir))?;
  let standby_signal = absolutize_path(&cfg.postgres.data_dir.join(MANAGED_STANDBY_SIGNAL_NAME))?;
  let recovery_signal =
          absolutize_path(&cfg.postgres.data_dir.join(MANAGED_RECOVERY_SIGNAL_NAME))?;
  let postgresql_auto_conf =
          absolutize_path(&cfg.postgres.data_dir.join(POSTGRESQL_AUTO_CONF_NAME))?;
  let quarantined_postgresql_auto_conf = absolutize_path(
    &cfg.postgres
            .data_dir
            .join(QUARANTINED_POSTGRESQL_AUTO_CONF_NAME),
  )?;

  write_atomic(
    &managed_hba,
    cfg.postgres.pg_hba_contents.as_bytes(),
    Some(0o644),
  )?;
  write_atomic(
    &managed_ident,
    cfg.postgres.pg_ident_contents.as_bytes(),
    Some(0o644),
  )?;

  let managed_tls_config = managed_tls_config(cfg)?;
  let standby_passfile_path = materialize_managed_standby_passfile(
    cfg,
    start_intent,
    managed_standby_passfile.as_path(),
  )?;
  let managed_conf = ManagedPostgresConf {
    listen_addresses: cfg.postgres.listen_host.clone(),
    port: cfg.postgres.listen_port,
    unix_socket_directories: cfg.postgres.socket_dir.clone(),
    hba_file: managed_hba.clone(),
    ident_file: managed_ident.clone(),
    tls: managed_tls_config,
    start_intent: start_intent.clone(),
    extra_gucs: cfg.postgres.extra_gucs.clone(),
  };
  let rendered_conf =
          render_managed_postgres_conf(&managed_conf, managed_standby_passfile.as_path())
                  .map_err(map_managed_conf_error)?;
  write_atomic(
    &managed_postgresql_conf,
    rendered_conf.as_bytes(),
    Some(0o644),
  )?;

  quarantine_postgresql_auto_conf(&postgresql_auto_conf, &quarantined_postgresql_auto_conf)?;
  materialize_recovery_signal_files(
    start_intent.recovery_signal(),
    &standby_signal,
    &recovery_signal,
  )?;

  Ok(ManagedPostgresConfig {
    postgresql_conf_path: managed_postgresql_conf,
    hba_path: managed_hba,
    ident_path: managed_ident,
    standby_passfile_path,
    standby_signal_path: standby_signal,
    recovery_signal_path: recovery_signal,
    postgresql_auto_conf_path: postgresql_auto_conf,
    quarantined_postgresql_auto_conf_path: quarantined_postgresql_auto_conf,
  })
}
```

