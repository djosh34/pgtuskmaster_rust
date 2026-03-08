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
- docs/src/reference/managed-postgres.md

[Page goal]
- Reference the managed PostgreSQL runtime-file materialization module and readback helpers.

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
- Overview\n- Core types\n- Managed file set\n- Materialization pipeline\n- Standby auth materialization\n- TLS materialization\n- Signal-file behavior\n- Readback and runtime integration boundary

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

# Managed PostgreSQL Runtime Files

The `src/postgres_managed.rs` module materializes and rereads managed PostgreSQL runtime files under `cfg.postgres.data_dir`. It writes managed config files, TLS assets, standby-auth files, and recovery signal files. It does not start PostgreSQL processes.

## Core types

### `ManagedPostgresError`

| Variant | Fields |
|---|---|
| `Io` | `message: String` |
| `InvalidConfig` | `message: String` |
| `InvalidManagedState` | `message: String` |

### `ManagedRecoverySignal`

| Variant |
|---|
| `None` |
| `Standby` |
| `Recovery` |

### `ManagedStandbyAuth`

| Variant | Fields |
|---|---|
| `NoPassword` | none |
| `PasswordPassfile` | `path: PathBuf` |

### `ManagedPostgresTlsConfig`

| Variant | Fields |
|---|---|
| `Disabled` | none |
| `Enabled` | `cert_file: PathBuf`, `key_file: PathBuf`, `ca_file: Option<PathBuf>` |

### `ManagedPostgresStartIntent`

| Variant | Fields |
|---|---|
| `Primary` | none |
| `Replica` | `primary_conninfo: PgConnInfo`, `standby_auth: ManagedStandbyAuth`, `primary_slot_name: Option<String>` |
| `Recovery` | `primary_conninfo: PgConnInfo`, `standby_auth: ManagedStandbyAuth`, `primary_slot_name: Option<String>` |

`ManagedPostgresStartIntent::recovery_signal()` maps variants to `ManagedRecoverySignal`:

| Variant | Signal |
|---|---|
| `Primary` | `None` |
| `Replica` | `Standby` |
| `Recovery` | `Recovery` |

### `ManagedPostgresConfig`

| Field | Type |
|---|---|
| `postgresql_conf_path` | `PathBuf` |
| `hba_path` | `PathBuf` |
| `ident_path` | `PathBuf` |
| `standby_passfile_path` | `Option<PathBuf>` |
| `tls_cert_path` | `Option<PathBuf>` |
| `tls_key_path` | `Option<PathBuf>` |
| `tls_client_ca_path` | `Option<PathBuf>` |
| `standby_signal_path` | `PathBuf` |
| `recovery_signal_path` | `PathBuf` |
| `postgresql_auto_conf_path` | `PathBuf` |
| `quarantined_postgresql_auto_conf_path` | `PathBuf` |

### `ManagedPostgresConf`

Configuration struct rendered into `pgtm.postgresql.conf`.

| Field | Type |
|---|---|
| `listen_addresses` | `String` |
| `port` | `u16` |
| `unix_socket_directories` | `PathBuf` |
| `hba_file` | `PathBuf` |
| `ident_file` | `PathBuf` |
| `tls` | `ManagedPostgresTlsConfig` |
| `start_intent` | `ManagedPostgresStartIntent` |
| `extra_gucs` | `BTreeMap<String, String>` |

## Managed file set

`materialize_managed_postgres_config(cfg, start_intent)` writes the following under `cfg.postgres.data_dir`:

| Filename | Mode | Purpose |
|---|---|---|
| `pgtm.postgresql.conf` | `0644` | Rendered managed PostgreSQL configuration |
| `pgtm.pg_hba.conf` | `0644` | HBA rules from `postgres.pg_hba.source` |
| `pgtm.pg_ident.conf` | `0644` | Ident rules from `postgres.pg_ident.source` |
| `pgtm.standby.passfile` | `0600` | Managed libpq passfile for `PasswordPassfile` auth |
| `pgtm.server.crt` | `0644` | Managed copy of the PostgreSQL TLS server certificate |
| `pgtm.server.key` | `0600` | Managed copy of the PostgreSQL TLS server private key |
| `pgtm.ca.crt` | `0644` | Managed copy of the client CA when client auth is configured |
| `standby.signal` | not written with `write_atomic` | Standby-mode signal file |
| `recovery.signal` | not written with `write_atomic` | Recovery-mode signal file |
| `postgresql.auto.conf` | existing file | Active PostgreSQL auto-config that may be quarantined |
| `pgtm.unmanaged.postgresql.auto.conf` | rename target | Quarantine target for existing auto-config |

Constants:

- `MANAGED_PG_HBA_CONF_NAME`: `"pgtm.pg_hba.conf"`
- `MANAGED_PG_IDENT_CONF_NAME`: `"pgtm.pg_ident.conf"`
- `POSTGRESQL_AUTO_CONF_NAME`: `"postgresql.auto.conf"`
- `QUARANTINED_POSTGRESQL_AUTO_CONF_NAME`: `"pgtm.unmanaged.postgresql.auto.conf"`

## Materialization pipeline

`materialize_managed_postgres_config(cfg, start_intent)` performs:

1. Validates non-empty `cfg.postgres.data_dir`.
2. Writes `pgtm.pg_hba.conf` from `postgres.pg_hba.source`.
3. Writes `pgtm.pg_ident.conf` from `postgres.pg_ident.source`.
4. Materializes TLS files and determines `ManagedPostgresTlsConfig`.
5. Normalizes standby-auth paths to the managed passfile location.
6. Materializes the optional standby passfile for replica or recovery intent.
7. Renders `ManagedPostgresConf` and writes `pgtm.postgresql.conf`.
8. Quarantines existing `postgresql.auto.conf` to `pgtm.unmanaged.postgresql.auto.conf`.
9. Updates `standby.signal` and `recovery.signal` for the selected start intent.
10. Returns `ManagedPostgresConfig` with managed paths.

## Standby auth materialization

`normalize_standby_auth_paths` rewrites `PasswordPassfile` auth to the managed standby passfile path under PGDATA.

`materialize_managed_standby_passfile` behavior:

| Intent And Auth | Action |
|---|---|
| `Primary` | removes stale managed passfile and returns `None` |
| `Replica` or `Recovery` with `NoPassword` | removes the managed passfile |
| `Replica` or `Recovery` with `PasswordPassfile` | resolves the replicator password, writes one libpq passfile entry with mode `0600`, and returns the managed path |

`render_libpq_passfile_entry` rejects newline characters in host, dbname, user, and password fields, escapes `:` and `\`, and renders one trailing newline.

`resolve_role_password` requires password auth for replicator role when managed standby passfile materialization is requested.

## TLS materialization

`materialize_tls_files` returns `ManagedPostgresTlsConfig::Disabled` when `cfg.postgres.tls.mode` is `Disabled`.

When `cfg.postgres.tls.mode` is `Optional` or `Required`:

- `cfg.postgres.tls.identity` must be present
- certificate material is copied to `pgtm.server.crt` with mode `0644`
- key material is copied to `pgtm.server.key` with mode `0600`
- if `cfg.postgres.tls.client_auth` is present, CA material is copied to `pgtm.ca.crt` with mode `0644`

The module copies operator-supplied TLS material and does not generate credentials.

## Signal-file behavior

Recovery signal files are mutually exclusive.

| Start Intent | `standby.signal` | `recovery.signal` |
|---|---|---|
| `Primary` | removed | removed |
| `Replica` | created | removed |
| `Recovery` | removed | created |

## Readback and runtime integration boundary

### `read_existing_replica_start_intent`

`read_existing_replica_start_intent(data_dir)`:

- checks `standby.signal` and `recovery.signal`
- returns `Ok(None)` when neither signal file exists
- returns `InvalidManagedState` if both signal files exist
- reads `pgtm.postgresql.conf`
- requires `primary_conninfo`
- parses `primary_conninfo` through `parse_managed_primary_conninfo`
- reads optional `primary_slot_name`
- reconstructs `Replica` or `Recovery` from the signal file

### Runtime integration

`runtime::run_start_job` materializes managed config via `materialize_managed_postgres_config` and starts `ProcessJobKind::StartPostgres` with `config_file = managed.postgresql_conf_path`.

Related tests:

- `build_command_start_postgres_uses_managed_config_file_override` in `src/process/worker.rs`
- `start_postgres_dispatch_builds_request_with_managed_settings` in `src/ha/process_dispatch.rs`

[Repo facts and source excerpts]

--- BEGIN FILE: src/postgres_managed.rs ---
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

use crate::{
    config::{ApiTlsMode, InlineOrPath, RoleAuthConfig, RuntimeConfig, SecretSource},
    postgres_managed_conf::{
        managed_standby_passfile_path, parse_managed_primary_conninfo,
        render_managed_postgres_conf, ManagedPostgresConf, ManagedPostgresConfError,
        ManagedPostgresStartIntent, ManagedPostgresTlsConfig, ManagedRecoverySignal,
        ManagedStandbyAuth, MANAGED_POSTGRESQL_CONF_NAME, MANAGED_RECOVERY_SIGNAL_NAME,
        MANAGED_STANDBY_SIGNAL_NAME,
    },
};

const MANAGED_PG_HBA_CONF_NAME: &str = "pgtm.pg_hba.conf";
const MANAGED_PG_IDENT_CONF_NAME: &str = "pgtm.pg_ident.conf";
const POSTGRESQL_AUTO_CONF_NAME: &str = "postgresql.auto.conf";
const QUARANTINED_POSTGRESQL_AUTO_CONF_NAME: &str = "pgtm.unmanaged.postgresql.auto.conf";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostgresConfig {
    pub(crate) postgresql_conf_path: PathBuf,
    pub(crate) hba_path: PathBuf,
    pub(crate) ident_path: PathBuf,
    pub(crate) standby_passfile_path: Option<PathBuf>,
    pub(crate) tls_cert_path: Option<PathBuf>,
    pub(crate) tls_key_path: Option<PathBuf>,
    pub(crate) tls_client_ca_path: Option<PathBuf>,
    pub(crate) standby_signal_path: PathBuf,
    pub(crate) recovery_signal_path: PathBuf,
    pub(crate) postgresql_auto_conf_path: PathBuf,
    pub(crate) quarantined_postgresql_auto_conf_path: PathBuf,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ManagedPostgresError {
    #[error("io error: {message}")]
    Io { message: String },
    #[error("invalid config: {message}")]
    InvalidConfig { message: String },
    #[error("invalid managed postgres state: {message}")]
    InvalidManagedState { message: String },
}

pub(crate) fn materialize_managed_postgres_config(
    cfg: &RuntimeConfig,
    start_intent: &ManagedPostgresStartIntent,
) -> Result<ManagedPostgresConfig, ManagedPostgresError> {
    let data_dir = cfg.postgres.data_dir.as_path();
    if data_dir.as_os_str().is_empty() {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "postgres.data_dir must not be empty".to_string(),
        });
    }

    let managed_hba = absolutize_path(&cfg.postgres.data_dir.join(MANAGED_PG_HBA_CONF_NAME))?;
    let managed_ident = absolutize_path(&cfg.postgres.data_dir.join(MANAGED_PG_IDENT_CONF_NAME))?;
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

    let hba_contents =
        load_inline_or_path_string("postgres.pg_hba.source", &cfg.postgres.pg_hba.source)?;
    let ident_contents =
        load_inline_or_path_string("postgres.pg_ident.source", &cfg.postgres.pg_ident.source)?;

    write_atomic(&managed_hba, hba_contents.as_bytes(), Some(0o644))?;
    write_atomic(&managed_ident, ident_contents.as_bytes(), Some(0o644))?;

    let tls_files = materialize_tls_files(cfg)?;
    let normalized_start_intent =
        normalize_standby_auth_paths(start_intent, managed_standby_passfile.as_path());
    let standby_passfile_path = materialize_managed_standby_passfile(
        cfg,
        &normalized_start_intent,
        managed_standby_passfile.as_path(),
    )?;
    let managed_conf = ManagedPostgresConf {
        listen_addresses: cfg.postgres.listen_host.clone(),
        port: cfg.postgres.listen_port,
        unix_socket_directories: cfg.postgres.socket_dir.clone(),
        hba_file: managed_hba.clone(),
        ident_file: managed_ident.clone(),
        tls: tls_files.managed_tls_config.clone(),
        start_intent: normalized_start_intent.clone(),
        extra_gucs: cfg.postgres.extra_gucs.clone(),
    };
    let rendered_conf =
        render_managed_postgres_conf(&managed_conf).map_err(map_managed_conf_error)?;
    write_atomic(
        &managed_postgresql_conf,
        rendered_conf.as_bytes(),
        Some(0o644),
    )?;

    quarantine_postgresql_auto_conf(&postgresql_auto_conf, &quarantined_postgresql_auto_conf)?;
    materialize_recovery_signal_files(
        normalized_start_intent.recovery_signal(),
        &standby_signal,
        &recovery_signal,
    )?;

    Ok(ManagedPostgresConfig {
        postgresql_conf_path: managed_postgresql_conf,
        hba_path: managed_hba,
        ident_path: managed_ident,
        standby_passfile_path,
        tls_cert_path: tls_files.cert_path,
        tls_key_path: tls_files.key_path,
        tls_client_ca_path: tls_files.client_ca_path,
        standby_signal_path: standby_signal,
        recovery_signal_path: recovery_signal,
        postgresql_auto_conf_path: postgresql_auto_conf,
        quarantined_postgresql_auto_conf_path: quarantined_postgresql_auto_conf,
    })
}

pub(crate) fn read_existing_replica_start_intent(
    data_dir: &Path,
) -> Result<Option<ManagedPostgresStartIntent>, ManagedPostgresError> {
    let recovery_signal = existing_recovery_signal(data_dir)?;
    let Some(recovery_signal) = recovery_signal else {
        return Ok(None);
    };

    let managed_conf_path = data_dir.join(MANAGED_POSTGRESQL_CONF_NAME);
    let rendered =
        fs::read_to_string(&managed_conf_path).map_err(|err| ManagedPostgresError::Io {
            message: format!(
                "failed to read existing managed postgres conf {}: {err}",
                managed_conf_path.display()
            ),
        })?;

    let primary_conninfo_raw = parse_managed_string_setting(rendered.as_str(), "primary_conninfo")?
        .ok_or_else(|| ManagedPostgresError::InvalidManagedState {
            message: format!(
                "existing managed replica state at {} is missing primary_conninfo",
                managed_conf_path.display()
            ),
        })?;
    let parsed = parse_managed_primary_conninfo(primary_conninfo_raw.as_str(), data_dir).map_err(
        |err| ManagedPostgresError::InvalidManagedState {
            message: format!(
                "existing managed primary_conninfo at {} is invalid: {err}",
                managed_conf_path.display()
            ),
        },
    )?;
    let primary_slot_name = parse_managed_string_setting(rendered.as_str(), "primary_slot_name")?;

    match recovery_signal {
        ManagedRecoverySignal::Standby => Ok(Some(ManagedPostgresStartIntent::replica(
            parsed.conninfo,
            parsed.standby_auth,
            primary_slot_name,
        ))),
        ManagedRecoverySignal::Recovery => Ok(Some(ManagedPostgresStartIntent::recovery(
            parsed.conninfo,
            parsed.standby_auth,
            primary_slot_name,
        ))),
        ManagedRecoverySignal::None => Ok(None),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MaterializedTlsFiles {
    managed_tls_config: ManagedPostgresTlsConfig,
    cert_path: Option<PathBuf>,
    key_path: Option<PathBuf>,
    client_ca_path: Option<PathBuf>,
}

fn materialize_tls_files(
    cfg: &RuntimeConfig,
) -> Result<MaterializedTlsFiles, ManagedPostgresError> {
    match cfg.postgres.tls.mode {
        ApiTlsMode::Disabled => Ok(MaterializedTlsFiles {
            managed_tls_config: ManagedPostgresTlsConfig::Disabled,
            cert_path: None,
            key_path: None,
            client_ca_path: None,
        }),
        ApiTlsMode::Optional | ApiTlsMode::Required => {
            let identity = cfg.postgres.tls.identity.as_ref().ok_or_else(|| {
                ManagedPostgresError::InvalidConfig {
                    message:
                        "postgres.tls.identity must be configured with user-supplied certificate material when postgres.tls.mode is optional or required"
                            .to_string(),
                }
            })?;

            let cert_pem = load_inline_or_path_bytes(
                "postgres.tls.identity.cert_chain",
                &identity.cert_chain,
            )?;
            let key_pem = load_inline_or_path_bytes(
                "postgres.tls.identity.private_key",
                &identity.private_key,
            )?;

            let managed_cert = absolutize_path(&cfg.postgres.data_dir.join("pgtm.server.crt"))?;
            let managed_key = absolutize_path(&cfg.postgres.data_dir.join("pgtm.server.key"))?;

            // Production TLS credentials are operator-supplied; pgtuskmaster only copies them
            // into managed runtime files under PGDATA before PostgreSQL starts.
            write_atomic(&managed_cert, cert_pem.as_slice(), Some(0o644))?;
            write_atomic(&managed_key, key_pem.as_slice(), Some(0o600))?;

            let client_ca_path = if let Some(client_auth) = cfg.postgres.tls.client_auth.as_ref() {
                let ca_pem = load_inline_or_path_bytes(
                    "postgres.tls.client_auth.client_ca",
                    &client_auth.client_ca,
                )?;
                let managed_ca = absolutize_path(&cfg.postgres.data_dir.join("pgtm.ca.crt"))?;
                write_atomic(&managed_ca, ca_pem.as_slice(), Some(0o644))?;
                Some(managed_ca)
            } else {
                None
            };

            Ok(MaterializedTlsFiles {
                managed_tls_config: ManagedPostgresTlsConfig::Enabled {
                    cert_file: managed_cert.clone(),
                    key_file: managed_key.clone(),
                    ca_file: client_ca_path.clone(),
                },
                cert_path: Some(managed_cert),
                key_path: Some(managed_key),
                client_ca_path,
            })
        }
    }
}

fn normalize_standby_auth_paths(
    start_intent: &ManagedPostgresStartIntent,
    managed_passfile_path: &Path,
) -> ManagedPostgresStartIntent {
    match start_intent {
        ManagedPostgresStartIntent::Primary => ManagedPostgresStartIntent::primary(),
        ManagedPostgresStartIntent::Replica {
            primary_conninfo,
            standby_auth,
            primary_slot_name,
        } => ManagedPostgresStartIntent::replica(
            primary_conninfo.clone(),
            normalize_standby_auth(standby_auth, managed_passfile_path),
            primary_slot_name.clone(),
        ),
        ManagedPostgresStartIntent::Recovery {
            primary_conninfo,
            standby_auth,
            primary_slot_name,
        } => ManagedPostgresStartIntent::recovery(
            primary_conninfo.clone(),
            normalize_standby_auth(standby_auth, managed_passfile_path),
            primary_slot_name.clone(),
        ),
    }
}

fn normalize_standby_auth(
    standby_auth: &ManagedStandbyAuth,
    managed_passfile_path: &Path,
) -> ManagedStandbyAuth {
    match standby_auth {
        ManagedStandbyAuth::NoPassword => ManagedStandbyAuth::NoPassword,
        ManagedStandbyAuth::PasswordPassfile { .. } => ManagedStandbyAuth::PasswordPassfile {
            path: managed_passfile_path.to_path_buf(),
        },
    }
}

fn materialize_managed_standby_passfile(
    cfg: &RuntimeConfig,
    start_intent: &ManagedPostgresStartIntent,
    managed_passfile_path: &Path,
) -> Result<Option<PathBuf>, ManagedPostgresError> {
    let standby_details = match start_intent {
        ManagedPostgresStartIntent::Primary => None,
        ManagedPostgresStartIntent::Replica {
            primary_conninfo,
            standby_auth,
            ..
        }
        | ManagedPostgresStartIntent::Recovery {
            primary_conninfo,
            standby_auth,
            ..
        } => Some((primary_conninfo, standby_auth)),
    };

    let Some((primary_conninfo, standby_auth)) = standby_details else {
        remove_file_if_exists(managed_passfile_path)?;
        return Ok(None);
    };

    match standby_auth {
        ManagedStandbyAuth::NoPassword => {
            remove_file_if_exists(managed_passfile_path)?;
            Ok(None)
        }
        ManagedStandbyAuth::PasswordPassfile { path } => {
            let password = resolve_role_password(
                "postgres.roles.replicator.auth",
                &cfg.postgres.roles.replicator.auth,
            )?;
            let rendered = render_libpq_passfile_entry(primary_conninfo, password.as_str())?;
            write_atomic(path, rendered.as_bytes(), Some(0o600))?;
            Ok(Some(path.clone()))
        }
    }
}

fn resolve_role_password(key: &str, auth: &RoleAuthConfig) -> Result<String, ManagedPostgresError> {
    match auth {
        RoleAuthConfig::Password { password } => resolve_secret_source_string(key, password),
        RoleAuthConfig::Tls => Err(ManagedPostgresError::InvalidConfig {
            message: format!(
                "{key} must use password auth when managed standby passfile materialization is requested"
            ),
        }),
    }
}

fn resolve_secret_source_string(
    key: &str,
    secret: &SecretSource,
) -> Result<String, ManagedPostgresError> {
    let value = match &secret.0 {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => {
            fs::read_to_string(path).map_err(|err| ManagedPostgresError::Io {
                message: format!("failed to read `{key}` from {}: {err}", path.display()),
            })?
        }
        InlineOrPath::Inline { content } => content.clone(),
    };

    Ok(value.trim_end_matches(['\n', '\r']).to_string())
}

fn render_libpq_passfile_entry(
    conninfo: &crate::pginfo::state::PgConnInfo,
    password: &str,
) -> Result<String, ManagedPostgresError> {
    if [conninfo.host.as_str(), conninfo.dbname.as_str(), conninfo.user.as_str(), password]
        .iter()
        .any(|value| value.chars().any(|ch| ch == '\n' || ch == '\r'))
    {
        return Err(ManagedPostgresError::InvalidConfig {
            message: "managed standby passfile fields must not contain newlines".to_string(),
        });
    }

    Ok(format!(
        "{}:{}:{}:{}:{}\n",
        escape_libpq_passfile_field(conninfo.host.as_str()),
        conninfo.port,
        escape_libpq_passfile_field(conninfo.dbname.as_str()),
        escape_libpq_passfile_field(conninfo.user.as_str()),
        escape_libpq_passfile_field(password),
    ))
}

fn escape_libpq_passfile_field(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            ':' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn map_managed_conf_error(err: ManagedPostgresConfError) -> ManagedPostgresError {
    match err {
        ManagedPostgresConfError::InvalidExtraGuc { key, message } => {
            ManagedPostgresError::InvalidConfig {
                message: format!("postgres.extra_gucs entry `{key}` invalid: {message}"),
            }
        }
        ManagedPostgresConfError::ReservedExtraGuc { key } => ManagedPostgresError::InvalidConfig {
            message: format!("postgres.extra_gucs entry `{key}` is reserved by pgtuskmaster"),
        },
        ManagedPostgresConfError::InvalidPrimarySlotName { slot, message } => {
            ManagedPostgresError::InvalidConfig {
                message: format!("managed replica slot `{slot}` invalid: {message}"),
            }
        }
    }
}

fn existing_recovery_signal(
    data_dir: &Path,
) -> Result<Option<ManagedRecoverySignal>, ManagedPostgresError> {
    let standby_signal_path = data_dir.join(MANAGED_STANDBY_SIGNAL_NAME);
    let recovery_signal_path = data_dir.join(MANAGED_RECOVERY_SIGNAL_NAME);
    let standby_present = file_exists(standby_signal_path.as_path())?;
    let recovery_present = file_exists(recovery_signal_path.as_path())?;

    match (standby_present, recovery_present) {
        (false, false) => Ok(None),
        (true, false) => Ok(Some(ManagedRecoverySignal::Standby)),
        (false, true) => Ok(Some(ManagedRecoverySignal::Recovery)),
        (true, true) => Err(ManagedPostgresError::InvalidManagedState {
            message: format!(
                "conflicting managed recovery signal files exist at {} and {}",
                standby_signal_path.display(),
                recovery_signal_path.display()
            ),
        }),
    }
}

fn load_inline_or_path_string(
    field: &str,
    source: &InlineOrPath,
) -> Result<String, ManagedPostgresError> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => fs::read_to_string(path)
            .map_err(|err| ManagedPostgresError::Io {
                message: format!("failed to read `{field}` from {}: {err}", path.display()),
            }),
        InlineOrPath::Inline { content } => Ok(content.clone()),
    }
}

fn load_inline_or_path_bytes(
    field: &str,
    source: &InlineOrPath,
) -> Result<Vec<u8>, ManagedPostgresError> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => {
            fs::read(path).map_err(|err| ManagedPostgresError::Io {
                message: format!("failed to read `{field}` from {}: {err}", path.display()),
            })
        }
        InlineOrPath::Inline { content } => Ok(content.as_bytes().to_vec()),
    }
}

fn absolutize_path(path: &Path) -> Result<PathBuf, ManagedPostgresError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().map_err(|err| ManagedPostgresError::Io {
        message: format!("failed to read current_dir: {err}"),
    })?;
    Ok(cwd.join(path))
}

fn remove_file_if_exists(path: &Path) -> Result<(), ManagedPostgresError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(ManagedPostgresError::Io {
            message: format!("failed to remove {}: {err}", path.display()),
        }),
    }
}

fn file_exists(path: &Path) -> Result<bool, ManagedPostgresError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(ManagedPostgresError::Io {
            message: format!("failed to stat {}: {err}", path.display()),
        }),
    }
}

fn materialize_recovery_signal_files(
    recovery_signal: ManagedRecoverySignal,
    standby_signal: &Path,
    recovery_signal_path: &Path,
) -> Result<(), ManagedPostgresError> {
    match recovery_signal {
        ManagedRecoverySignal::None => {
            remove_file_if_exists(standby_signal)?;
            remove_file_if_exists(recovery_signal_path)?;
        }
        ManagedRecoverySignal::Standby => {
            write_atomic(standby_signal, b"", Some(0o644))?;
            remove_file_if_exists(recovery_signal_path)?;
        }
        ManagedRecoverySignal::Recovery => {
            write_atomic(recovery_signal_path, b"", Some(0o644))?;
            remove_file_if_exists(standby_signal)?;
        }
    }
    Ok(())
}

fn quarantine_postgresql_auto_conf(
    postgresql_auto_conf: &Path,
    quarantined_postgresql_auto_conf: &Path,
) -> Result<(), ManagedPostgresError> {
    match fs::rename(postgresql_auto_conf, quarantined_postgresql_auto_conf) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => {
            if file_exists(quarantined_postgresql_auto_conf)? {
                fs::remove_file(quarantined_postgresql_auto_conf).map_err(|remove_err| {
                    ManagedPostgresError::Io {
                        message: format!(
                            "failed to remove previous quarantined auto conf {} after rename error ({err}): {remove_err}",
                            quarantined_postgresql_auto_conf.display()
                        ),
                    }
                })?;
                fs::rename(postgresql_auto_conf, quarantined_postgresql_auto_conf).map_err(
                    |rename_err| ManagedPostgresError::Io {
                        message: format!(
                            "failed to quarantine {} to {}: {rename_err}",
                            postgresql_auto_conf.display(),
                            quarantined_postgresql_auto_conf.display()
                        ),
                    },
                )
            } else {
                Err(ManagedPostgresError::Io {
                    message: format!(
                        "failed to quarantine {} to {}: {err}",
                        postgresql_auto_conf.display(),
                        quarantined_postgresql_auto_conf.display()
                    ),
                })
            }
        }
    }
}

fn parse_managed_string_setting(
    contents: &str,
    key: &str,
) -> Result<Option<String>, ManagedPostgresError> {
    let prefix = format!("{key} = '");
    for line in contents.lines() {
        if let Some(rest) = line.strip_prefix(prefix.as_str()) {
            let Some(quoted) = rest.strip_suffix('\'') else {
                return Err(ManagedPostgresError::InvalidManagedState {
                    message: format!("managed config setting `{key}` is missing a closing quote"),
                });
            };
            return unescape_managed_string(quoted).map(Some);
        }
    }
    Ok(None)
}

fn unescape_managed_string(value: &str) -> Result<String, ManagedPostgresError> {
    let mut chars = value.chars().peekable();
    let mut out = String::with_capacity(value.len());
    while let Some(ch) = chars.next() {
        match ch {
            '\'' => {
                let Some(next) = chars.next() else {
                    return Err(ManagedPostgresError::InvalidManagedState {
                        message: "managed config string contains an unescaped single quote"
                            .to_string(),
                    });
                };
                if next != '\'' {
                    return Err(ManagedPostgresError::InvalidManagedState {
                        message: "managed config string contains an unescaped single quote"
                            .to_string(),
                    });
                }
                out.push('\'');
            }
            '\\' => {
                let Some(next) = chars.next() else {
                    return Err(ManagedPostgresError::InvalidManagedState {
                        message: "managed config string ends with a trailing backslash".to_string(),
                    });
                };
                out.push(next);
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

fn write_atomic(
    path: &Path,
    contents: &[u8],
    mode: Option<u32>,
) -> Result<(), ManagedPostgresError> {
    let parent = path.parent().ok_or_else(|| ManagedPostgresError::Io {
        message: format!("path has no parent: {}", path.display()),
    })?;
    fs::create_dir_all(parent).map_err(|err| ManagedPostgresError::Io {
        message: format!("failed to create dir {}: {err}", parent.display()),
    })?;

    let pid = std::process::id();
    let millis = now_millis()?;
    let file_name = match path.file_name().and_then(|value| value.to_str()) {
        Some(name) if !name.is_empty() => name,
        _ => "managed",
    };
    let tmp = parent.join(format!(".{file_name}.tmp.{pid}.{millis}"));

    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&tmp)
        .map_err(|err| ManagedPostgresError::Io {
            message: format!("failed to create temp file {}: {err}", tmp.display()),
        })?;

    use std::io::Write;
    file.write_all(contents)
        .map_err(|err| ManagedPostgresError::Io {
            message: format!("failed to write temp file {}: {err}", tmp.display()),
        })?;
    file.sync_all().map_err(|err| ManagedPostgresError::Io {
        message: format!("failed to sync temp file {}: {err}", tmp.display()),
    })?;

    #[cfg(unix)]
    if let Some(mode) = mode {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(mode)).map_err(|err| {
            ManagedPostgresError::Io {
                message: format!("failed to set permissions on {}: {err}", tmp.display()),
            }
        })?;
    }

    fs::rename(&tmp, path).or_else(|err| {
        if file_exists(path)? {
            fs::remove_file(path).map_err(|remove_err| ManagedPostgresError::Io {
                message: format!(
                    "failed to remove existing {} after rename error ({err}): {remove_err}",
                    path.display()
                ),
            })?;
            fs::rename(&tmp, path).map_err(|rename_err| ManagedPostgresError::Io {
                message: format!(
                    "failed to rename {} to {}: {rename_err}",
                    tmp.display(),
                    path.display()
                ),
            })
        } else {
            Err(ManagedPostgresError::Io {
                message: format!(
                    "failed to rename {} to {}: {err}",
                    tmp.display(),
                    path.display()
                ),
            })
        }
    })?;

    Ok(())
}

fn now_millis() -> Result<u128, ManagedPostgresError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| ManagedPostgresError::Io {
            message: format!("clock error: {err}"),
        })?;
    Ok(duration.as_millis())
}

#[cfg(test)]
mod tests {
    use std::{fs, io, path::{Path, PathBuf}, time::Duration};

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use tokio::process::Command;
    use tokio::time::Instant;
    use tokio_postgres::NoTls;

    use crate::{
        config::{
            ApiTlsMode, HaConfig, InlineOrPath, ProcessConfig, RuntimeConfig, TlsServerConfig,
        },
        pginfo::{conninfo::PgSslMode, state::PgConnInfo},
        postgres_managed_conf::{
            managed_standby_passfile_path, ManagedPostgresStartIntent, ManagedStandbyAuth,
            MANAGED_POSTGRESQL_CONF_NAME, MANAGED_RECOVERY_SIGNAL_NAME,
        },
        test_harness::{
            binaries::require_pg16_bin_for_real_tests,
            namespace::NamespaceGuard,
            pg16::{prepare_pgdata_dir, spawn_pg16, PgHandle, PgInstanceSpec},
            ports::allocate_ports,
        },
    };

    use super::{
        materialize_managed_postgres_config, read_existing_replica_start_intent,
        ManagedPostgresError, POSTGRESQL_AUTO_CONF_NAME, QUARANTINED_POSTGRESQL_AUTO_CONF_NAME,
    };

    #[test]
    fn materialize_managed_postgres_config_creates_authoritative_postgresql_conf(
    ) -> Result<(), String> {
        let data_dir = unique_test_data_dir("postgresql-conf");
        let cfg = sample_runtime_config(data_dir.clone());

        let managed =
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
                .map_err(|err| format!("materialize managed config failed: {err}"))?;

        let postgresql_conf = fs::read_to_string(&managed.postgresql_conf_path).map_err(|err| {
            format!(
                "read managed postgresql conf {} failed: {err}",
                managed.postgresql_conf_path.display()
            )
        })?;

        if !postgresql_conf.contains("listen_addresses = '127.0.0.1'") {
            return Err(format!(
                "managed postgresql conf missing listen_addresses: {postgresql_conf}"
            ));
        }
        if !postgresql_conf.contains("hba_file =") || !postgresql_conf.contains("ident_file =") {
            return Err(format!(
                "managed postgresql conf missing managed file paths: {postgresql_conf}"
            ));
        }
        if !postgresql_conf.contains("hot_standby = off") {
            return Err(format!(
                "managed postgresql conf missing primary hot_standby=off: {postgresql_conf}"
            ));
        }
        if postgresql_conf.contains("archive_mode")
            || postgresql_conf.contains("archive_command")
            || postgresql_conf.contains("restore_command")
        {
            return Err(format!(
                "managed postgresql conf unexpectedly contains backup settings: {postgresql_conf}"
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_uses_config_file_path_for_startup() -> Result<(), String>
    {
        let data_dir = unique_test_data_dir("config-file");
        let cfg = sample_runtime_config(data_dir.clone());

        let managed =
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
                .map_err(|err| format!("materialize managed config failed: {err}"))?;

        let expected = data_dir.join(MANAGED_POSTGRESQL_CONF_NAME);
        if managed.postgresql_conf_path != expected {
            return Err(format!(
                "unexpected postgresql_conf_path: expected={} got={}",
                expected.display(),
                managed.postgresql_conf_path.display()
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_creates_and_removes_standby_signal() -> Result<(), String>
    {
        let data_dir = unique_test_data_dir("standby-signal");
        let cfg = sample_runtime_config(data_dir.clone());
        let replica_start = ManagedPostgresStartIntent::replica(
            PgConnInfo {
                host: "leader.internal".to_string(),
                port: 5432,
                user: "replicator".to_string(),
                dbname: "postgres".to_string(),
                application_name: None,
                connect_timeout_s: Some(5),
                ssl_mode: PgSslMode::Prefer,
                options: None,
            },
            sample_password_standby_auth(&data_dir),
            None,
        );

        let managed_replica = materialize_managed_postgres_config(&cfg, &replica_start)
            .map_err(|err| format!("materialize replica config failed: {err}"))?;
        if !managed_replica.standby_signal_path.exists() {
            return Err(format!(
                "expected standby.signal to exist at {}",
                managed_replica.standby_signal_path.display()
            ));
        }
        if managed_replica.recovery_signal_path.exists() {
            return Err(format!(
                "expected recovery.signal to be absent at {}",
                managed_replica.recovery_signal_path.display()
            ));
        }

        let managed_primary =
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
                .map_err(|err| format!("materialize primary config failed: {err}"))?;
        if managed_primary.standby_signal_path.exists() {
            return Err(format!(
                "expected standby.signal to be removed at {}",
                managed_primary.standby_signal_path.display()
            ));
        }
        if managed_primary.recovery_signal_path.exists() {
            return Err(format!(
                "expected recovery.signal to be removed at {}",
                managed_primary.recovery_signal_path.display()
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_creates_recovery_signal_and_cleans_standby_signal(
    ) -> Result<(), String> {
        let data_dir = unique_test_data_dir("recovery-signal");
        let cfg = sample_runtime_config(data_dir.clone());
        let standby_signal = data_dir.join("standby.signal");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create test dir {} failed: {err}", data_dir.display()))?;
        fs::write(&standby_signal, b"").map_err(|err| {
            format!(
                "seed standby.signal {} failed: {err}",
                standby_signal.display()
            )
        })?;

        let managed = materialize_managed_postgres_config(
            &cfg,
            &ManagedPostgresStartIntent::recovery(
                PgConnInfo {
                    host: "leader.internal".to_string(),
                    port: 5432,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: Some(5),
                    ssl_mode: PgSslMode::Prefer,
                    options: None,
                },
                sample_password_standby_auth(&data_dir),
                None,
            ),
        )
        .map_err(|err| format!("materialize recovery config failed: {err}"))?;

        if !managed.recovery_signal_path.exists() {
            return Err(format!(
                "expected recovery.signal to exist at {}",
                managed.recovery_signal_path.display()
            ));
        }
        if managed.standby_signal_path.exists() {
            return Err(format!(
                "expected standby.signal to be removed at {}",
                managed.standby_signal_path.display()
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_writes_managed_standby_passfile() -> Result<(), String> {
        let data_dir = unique_test_data_dir("standby-passfile");
        let cfg = sample_runtime_config(data_dir.clone());

        let managed = materialize_managed_postgres_config(
            &cfg,
            &ManagedPostgresStartIntent::replica(
                sample_replica_conninfo(),
                sample_password_standby_auth(&data_dir),
                None,
            ),
        )
        .map_err(|err| format!("materialize replica config failed: {err}"))?;

        let passfile_path = managed
            .standby_passfile_path
            .ok_or_else(|| "missing standby passfile path".to_string())?;
        let contents = fs::read_to_string(&passfile_path).map_err(|err| {
            format!("read standby passfile {} failed: {err}", passfile_path.display())
        })?;
        if contents != "leader.internal:5432:postgres:replicator:secret-password\n" {
            return Err(format!(
                "unexpected standby passfile contents at {}: {contents:?}",
                passfile_path.display()
            ));
        }

        #[cfg(unix)]
        {
            let mode = fs::metadata(&passfile_path)
                .map_err(|err| format!("stat standby passfile failed: {err}"))?
                .permissions()
                .mode()
                & 0o777;
            if mode != 0o600 {
                return Err(format!(
                    "expected standby passfile mode 0600, got {:o} at {}",
                    mode,
                    passfile_path.display()
                ));
            }
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_removes_stale_standby_passfile_on_primary_start(
    ) -> Result<(), String> {
        let data_dir = unique_test_data_dir("stale-standby-passfile");
        let cfg = sample_runtime_config(data_dir.clone());
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create test dir {} failed: {err}", data_dir.display()))?;
        let stale_path = managed_standby_passfile_path(&data_dir);
        fs::write(&stale_path, "stale-password\n").map_err(|err| {
            format!(
                "write stale standby passfile {} failed: {err}",
                stale_path.display()
            )
        })?;

        let managed =
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
                .map_err(|err| format!("materialize primary config failed: {err}"))?;
        if stale_path.exists() {
            return Err(format!(
                "expected stale standby passfile to be removed at {}",
                stale_path.display()
            ));
        }
        if managed.standby_passfile_path.is_some() {
            return Err("primary start should not report a standby passfile path".to_string());
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_quarantines_postgresql_auto_conf() -> Result<(), String>
    {
        let data_dir = unique_test_data_dir("postgresql-auto-conf");
        let cfg = sample_runtime_config(data_dir.clone());
        let active_auto_conf = data_dir.join(POSTGRESQL_AUTO_CONF_NAME);
        let quarantined_auto_conf = data_dir.join(QUARANTINED_POSTGRESQL_AUTO_CONF_NAME);
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create test dir {} failed: {err}", data_dir.display()))?;
        fs::write(&active_auto_conf, "primary_conninfo = 'stale'\n").map_err(|err| {
            format!(
                "write active auto conf {} failed: {err}",
                active_auto_conf.display()
            )
        })?;
        fs::write(&quarantined_auto_conf, "stale previous quarantine\n").map_err(|err| {
            format!(
                "write quarantined auto conf {} failed: {err}",
                quarantined_auto_conf.display()
            )
        })?;

        let managed =
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
                .map_err(|err| format!("materialize primary config failed: {err}"))?;

        if managed.postgresql_auto_conf_path.exists() {
            return Err(format!(
                "expected active postgresql.auto.conf to be absent at {}",
                managed.postgresql_auto_conf_path.display()
            ));
        }
        let quarantined = fs::read_to_string(&managed.quarantined_postgresql_auto_conf_path)
            .map_err(|err| {
                format!(
                    "read quarantined auto conf {} failed: {err}",
                    managed.quarantined_postgresql_auto_conf_path.display()
                )
            })?;
        if quarantined != "primary_conninfo = 'stale'\n" {
            return Err(format!(
                "unexpected quarantined auto conf contents at {}: {quarantined}",
                managed.quarantined_postgresql_auto_conf_path.display()
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn materialize_managed_postgres_config_rejects_reserved_extra_guc() {
        let data_dir = unique_test_data_dir("reserved-extra");
        let mut cfg = sample_runtime_config(data_dir.clone());
        cfg.postgres
            .extra_gucs
            .insert("config_file".to_string(), "/tmp/override.conf".to_string());

        assert_eq!(
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary()),
            Err(ManagedPostgresError::InvalidConfig {
                message: "postgres.extra_gucs entry `config_file` is reserved by pgtuskmaster"
                    .to_string(),
            })
        );

        let _ = fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn materialize_managed_postgres_config_writes_managed_tls_files_for_user_supplied_identity(
    ) -> Result<(), String> {
        let data_dir = unique_test_data_dir("tls");
        let mut cfg = sample_runtime_config(data_dir.clone());
        cfg.postgres.tls = TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(crate::config::TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: "CERT".to_string(),
                },
                private_key: InlineOrPath::Inline {
                    content: "KEY".to_string(),
                },
            }),
            client_auth: Some(crate::config::TlsClientAuthConfig {
                client_ca: InlineOrPath::Inline {
                    content: "CA".to_string(),
                },
                require_client_cert: true,
            }),
        };

        let managed =
            materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
                .map_err(|err| format!("materialize managed config failed: {err}"))?;

        let cert = managed
            .tls_cert_path
            .ok_or_else(|| "missing managed cert path".to_string())?;
        let key = managed
            .tls_key_path
            .ok_or_else(|| "missing managed key path".to_string())?;
        let ca = managed
            .tls_client_ca_path
            .ok_or_else(|| "missing managed ca path".to_string())?;

        if fs::read_to_string(&cert).map_err(|err| err.to_string())? != "CERT" {
            return Err(format!("unexpected cert contents at {}", cert.display()));
        }
        if fs::read_to_string(&key).map_err(|err| err.to_string())? != "KEY" {
            return Err(format!("unexpected key contents at {}", key.display()));
        }
        if fs::read_to_string(&ca).map_err(|err| err.to_string())? != "CA" {
            return Err(format!("unexpected ca contents at {}", ca.display()));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn materialize_managed_postgres_config_real_clone_start_quarantines_auto_conf_and_stale_signal(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let postgres_bin = require_pg16_bin_for_real_tests("postgres")?;
        let initdb_bin = require_pg16_bin_for_real_tests("initdb")?;
        let basebackup_bin = require_pg16_bin_for_real_tests("pg_basebackup")?;

        let guard = NamespaceGuard::new("managed-config-real-start")?;
        let namespace = guard.namespace()?;

        let primary_data = prepare_pgdata_dir(namespace, "primary")?;
        let primary_socket = namespace.child_dir("run/primary");
        let primary_logs = namespace.child_dir("logs/primary");
        fs::create_dir_all(&primary_socket)?;
        fs::create_dir_all(&primary_logs)?;

        let primary_reservation = allocate_ports(1)?;
        let primary_port = primary_reservation.as_slice()[0];
        drop(primary_reservation);

        let mut primary = spawn_pg16(PgInstanceSpec {
            postgres_bin: postgres_bin.clone(),
            initdb_bin: initdb_bin.clone(),
            data_dir: primary_data.clone(),
            socket_dir: primary_socket,
            log_dir: primary_logs,
            port: primary_port,
            startup_timeout: Duration::from_secs(25),
        })
        .await?;

        let primary_dsn = format!(
            "host=127.0.0.1 port={} user=postgres dbname=postgres",
            primary_port
        );
        let run_result = async {
            wait_for_postgres_ready(&primary_dsn, Duration::from_secs(20)).await?;
            let (primary_client, primary_connection) =
                tokio_postgres::connect(&primary_dsn, NoTls).await?;
            let primary_connection_task = tokio::spawn(primary_connection);
            primary_client
                .batch_execute(
                    concat!(
                        "CREATE ROLE replicator WITH LOGIN REPLICATION PASSWORD 'secret-password';",
                        "CREATE TABLE IF NOT EXISTS public.passfile_replay_test (",
                        "id integer PRIMARY KEY, note text NOT NULL",
                        ");",
                    ),
                )
                .await?;
            append_to_file(
                primary_data.join("pg_hba.conf").as_path(),
                concat!(
                    "\n",
                    "host replication replicator 127.0.0.1/32 scram-sha-256\n",
                ),
            )?;
            let _ = primary_client.query_one("SELECT pg_reload_conf()", &[]).await?;

            let replica_data = namespace.child_dir("pg16/replica/data");
            let replica_parent = replica_data
                .parent()
                .ok_or_else(|| real_test_error("replica data dir has no parent"))?;
            fs::create_dir_all(replica_parent)?;

            let basebackup_output = Command::new(&basebackup_bin)
                .env("PGPASSWORD", "secret-password")
                .arg("-h")
                .arg("127.0.0.1")
                .arg("-p")
                .arg(primary_port.to_string())
                .arg("-D")
                .arg(&replica_data)
                .arg("-U")
                .arg("replicator")
                .arg("-Fp")
                .arg("-Xs")
                .output()
                .await?;
            if !basebackup_output.status.success() {
                return Err(real_test_error(format!(
                    "pg_basebackup failed with status {}",
                    basebackup_output.status
                )));
            }

            fs::write(replica_data.join(POSTGRESQL_AUTO_CONF_NAME), "port = 1\n")?;
            fs::write(replica_data.join(MANAGED_RECOVERY_SIGNAL_NAME), b"")?;

            let replica_socket = namespace.child_dir("run/replica");
            let replica_logs = namespace.child_dir("logs/replica");
            fs::create_dir_all(&replica_socket)?;
            fs::create_dir_all(&replica_logs)?;

            let replica_reservation = allocate_ports(1)?;
            let replica_port = replica_reservation.as_slice()[0];
            drop(replica_reservation);

            let mut runtime_config = sample_runtime_config(replica_data.clone());
            runtime_config.postgres.listen_port = replica_port;
            runtime_config.postgres.socket_dir = replica_socket.clone();
            runtime_config.postgres.log_file = replica_logs.join("managed-postgres.log");
            runtime_config.postgres.pg_hba.source = InlineOrPath::Inline {
                content: concat!(
                    "local all all trust\n",
                    "host all all 127.0.0.1/32 trust\n",
                    "host replication all 127.0.0.1/32 trust\n",
                )
                .to_string(),
            };

            let managed = materialize_managed_postgres_config(
                &runtime_config,
                &ManagedPostgresStartIntent::replica(
                    PgConnInfo {
                        host: "127.0.0.1".to_string(),
                        port: primary_port,
                        user: "replicator".to_string(),
                        dbname: "postgres".to_string(),
                        application_name: None,
                        connect_timeout_s: Some(5),
                        ssl_mode: PgSslMode::Prefer,
                        options: None,
                    },
                    sample_password_standby_auth(&replica_data),
                    None,
                ),
            )
            .map_err(|err| real_test_error(format!("materialize managed config failed: {err}")))?;

            if managed.postgresql_auto_conf_path.exists() {
                return Err(real_test_error(format!(
                    "expected active postgresql.auto.conf to be absent at {}",
                    managed.postgresql_auto_conf_path.display()
                )));
            }
            if !managed.quarantined_postgresql_auto_conf_path.exists() {
                return Err(real_test_error(format!(
                    "expected quarantined postgresql.auto.conf to exist at {}",
                    managed.quarantined_postgresql_auto_conf_path.display()
                )));
            }
            if !managed.standby_signal_path.exists() {
                return Err(real_test_error(format!(
                    "expected standby.signal to exist at {}",
                    managed.standby_signal_path.display()
                )));
            }
            if managed.recovery_signal_path.exists() {
                return Err(real_test_error(format!(
                    "expected recovery.signal to be absent at {}",
                    managed.recovery_signal_path.display()
                )));
            }

            let stdout_file = fs::File::create(replica_logs.join("postgres.stdout.log"))?;
            let stderr_file = fs::File::create(replica_logs.join("postgres.stderr.log"))?;
            let mut replica_child = Command::new(&postgres_bin)
                .arg("-D")
                .arg(&replica_data)
                .arg("-c")
                .arg(format!(
                    "config_file={}",
                    managed.postgresql_conf_path.display()
                ))
                .stdout(stdout_file)
                .stderr(stderr_file)
                .spawn()?;

            let replica_dsn = format!(
                "host=127.0.0.1 port={} user=postgres dbname=postgres",
                replica_port
            );
            let replica_result = async {
                wait_for_postgres_ready(&replica_dsn, Duration::from_secs(25)).await?;
                let (client, connection) = tokio_postgres::connect(&replica_dsn, NoTls).await?;
                let connection_task = tokio::spawn(connection);

                let port = client.query_one("SHOW port", &[]).await?;
                let port_text: String = port.get(0);
                if port_text != replica_port.to_string() {
                    return Err(real_test_error(format!(
                        "expected postgres to listen on managed port {}, got {}",
                        replica_port, port_text
                    )));
                }

                let primary_conninfo = client.query_one("SHOW primary_conninfo", &[]).await?;
                let primary_conninfo_text: String = primary_conninfo.get(0);
                if !primary_conninfo_text.contains(primary_port.to_string().as_str()) {
                    return Err(real_test_error(format!(
                        "expected primary_conninfo to reference primary port {}, got {}",
                        primary_port, primary_conninfo_text
                    )));
                }
                if !primary_conninfo_text.contains("passfile=") {
                    return Err(real_test_error(format!(
                        "expected primary_conninfo to include managed passfile, got {}",
                        primary_conninfo_text
                    )));
                }
                let standby_passfile = managed
                    .standby_passfile_path
                    .clone()
                    .ok_or_else(|| real_test_error("expected managed standby passfile path"))?;
                if !primary_conninfo_text.contains(standby_passfile.display().to_string().as_str()) {
                    return Err(real_test_error(format!(
                        "expected primary_conninfo to reference standby passfile {}, got {}",
                        standby_passfile.display(),
                        primary_conninfo_text
                    )));
                }

                let in_recovery = client.query_one("SELECT pg_is_in_recovery()", &[]).await?;
                let in_recovery_flag: bool = in_recovery.get(0);
                if !in_recovery_flag {
                    return Err(real_test_error(
                        "expected cloned node to start in recovery".to_string(),
                    ));
                }

                let passfile_contents = fs::read_to_string(&standby_passfile).map_err(|err| {
                    real_test_error(format!(
                        "read managed standby passfile {} failed: {err}",
                        standby_passfile.display()
                    ))
                })?;
                if !passfile_contents.contains("replicator:secret-password") {
                    return Err(real_test_error(format!(
                        "expected standby passfile to contain replicator credentials, got {:?}",
                        passfile_contents
                    )));
                }

                primary_client
                    .execute(
                        "INSERT INTO public.passfile_replay_test (id, note) VALUES ($1, $2)",
                        &[&1_i32, &"after-startup"],
                    )
                    .await?;
                wait_for_replica_row(
                    &client,
                    "SELECT note FROM public.passfile_replay_test WHERE id = 1",
                    "after-startup",
                    Duration::from_secs(20),
                )
                .await?;

                drop(client);
                connection_task.await??;
                drop(primary_client);
                primary_connection_task.await??;
                Ok(())
            }
            .await;

            let shutdown_result = shutdown_child("replica", &mut replica_child).await;
            match (replica_result, shutdown_result) {
                (Ok(()), Ok(())) => Ok(()),
                (Err(err), Ok(())) => Err(err),
                (Ok(()), Err(err)) => Err(err),
                (Err(err), Err(clean_err)) => Err(real_test_error(format!("{err}; {clean_err}"))),
            }
        }
        .await;

        let shutdown_primary = shutdown_pg_handle("primary", &mut primary).await;
        match (run_result, shutdown_primary) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(err)) => Err(err),
            (Err(err), Err(clean_err)) => Err(real_test_error(format!("{err}; {clean_err}"))),
        }
    }

    #[test]
    fn read_existing_replica_start_intent_reads_managed_replica_state() -> Result<(), String> {
        let data_dir = unique_test_data_dir("read-existing-replica");
        let cfg = sample_runtime_config(data_dir.clone());
        let expected = ManagedPostgresStartIntent::replica(
            sample_replica_conninfo(),
            sample_password_standby_auth(&data_dir),
            Some("slot_a".to_string()),
        );

        materialize_managed_postgres_config(&cfg, &expected)
            .map_err(|err| format!("materialize managed replica config failed: {err}"))?;

        let actual = read_existing_replica_start_intent(&data_dir)
            .map_err(|err| format!("read existing replica start intent failed: {err}"))?;
        if actual != Some(expected.clone()) {
            return Err(format!(
                "unexpected existing managed replica state: expected={expected:?} actual={actual:?}"
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    #[test]
    fn read_existing_replica_start_intent_rejects_conflicting_signal_files() -> Result<(), String> {
        let data_dir = unique_test_data_dir("conflicting-signals");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create test dir {} failed: {err}", data_dir.display()))?;
        let standby_signal = data_dir.join("standby.signal");
        let recovery_signal = data_dir.join(MANAGED_RECOVERY_SIGNAL_NAME);
        fs::write(&standby_signal, b"").map_err(|err| {
            format!(
                "write standby.signal {} failed: {err}",
                standby_signal.display()
            )
        })?;
        fs::write(&recovery_signal, b"").map_err(|err| {
            format!(
                "write recovery.signal {} failed: {err}",
                recovery_signal.display()
            )
        })?;

        let actual = read_existing_replica_start_intent(&data_dir);
        match actual {
            Err(ManagedPostgresError::InvalidManagedState { message }) => {
                if !message.contains("conflicting managed recovery signal files") {
                    return Err(format!(
                        "unexpected invalid managed state message: {message}"
                    ));
                }
            }
            Ok(value) => {
                return Err(format!(
                    "expected conflicting signal files to fail, got {value:?}"
                ));
            }
            Err(err) => return Err(format!("unexpected error variant: {err}")),
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove temp dir {} failed: {err}", data_dir.display()))?;
        Ok(())
    }

    fn unique_test_data_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "pgtuskmaster-postgres-managed-{label}-{}-{}",
            std::process::id(),
            crate::logging::system_now_unix_millis()
        ))
    }

    fn sample_runtime_config(data_dir: PathBuf) -> RuntimeConfig {
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir)
            .with_dcs_scope("cluster-a")
            .with_ha(HaConfig {
                loop_interval_ms: 500,
                lease_ttl_ms: 5_000,
            })
            .with_process(ProcessConfig {
                pg_rewind_timeout_ms: 30_000,
                bootstrap_timeout_ms: 30_000,
                fencing_timeout_ms: 10_000,
                binaries: crate::test_harness::runtime_config::sample_binary_paths(),
            })
            .build()
    }

    fn sample_replica_conninfo() -> PgConnInfo {
        sample_replica_conninfo_for_port(5432)
    }

    fn sample_replica_conninfo_for_port(port: u16) -> PgConnInfo {
        PgConnInfo {
            host: "leader.internal".to_string(),
            port,
            user: "replicator".to_string(),
            dbname: "postgres".to_string(),
            application_name: None,
            connect_timeout_s: Some(5),
            ssl_mode: PgSslMode::Prefer,
            options: None,
        }
    }

    fn sample_password_standby_auth(data_dir: &Path) -> ManagedStandbyAuth {
        ManagedStandbyAuth::PasswordPassfile {
            path: managed_standby_passfile_path(data_dir),
        }
    }

    fn append_to_file(path: &Path, contents: &str) -> Result<(), io::Error> {
        let mut file = fs::OpenOptions::new().append(true).open(path)?;
        use std::io::Write;
        file.write_all(contents.as_bytes())?;
        file.sync_all()
    }

    async fn wait_for_postgres_ready(
        dsn: &str,
        timeout: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let deadline = Instant::now() + timeout;
        loop {
            match tokio_postgres::connect(dsn, NoTls).await {
                Ok((client, connection)) => {
                    let connection_task = tokio::spawn(connection);
                    client.simple_query("SELECT 1").await?;
                    drop(client);
                    connection_task.await??;
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

    async fn wait_for_replica_row(
        client: &tokio_postgres::Client,
        query: &str,
        expected: &str,
        timeout: Duration,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let deadline = Instant::now() + timeout;

        loop {
            let last_outcome = match client.query_opt(query, &[]).await {
                Ok(Some(row)) => {
                    let actual: String = row.get(0);
                    if actual == expected {
                        return Ok(());
                    }
                    format!("unexpected row value {actual:?}")
                }
                Ok(None) => {
                    "row not replayed yet".to_string()
                }
                Err(err) => {
                    err.to_string()
                }
            };

            if Instant::now() >= deadline {
                return Err(real_test_error(format!(
                    "timed out waiting for replica replay; last outcome: {}",
                    last_outcome
                )));
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    async fn shutdown_pg_handle(
        label: &str,
        handle: &mut PgHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        handle
            .shutdown()
            .await
            .map_err(|err| real_test_error(format!("{label} shutdown failed: {err}")))
    }

    async fn shutdown_child(
        _label: &str,
        child: &mut tokio::process::Child,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if child.try_wait()?.is_none() {
            child.start_kill()?;
            child.wait().await?;
        }
        Ok(())
    }

    fn real_test_error(message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(io::Error::other(message.into()))
    }
}

--- END FILE: src/postgres_managed.rs ---

--- BEGIN FILE: src/runtime/node.rs ---
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use thiserror::Error;
use tokio::{net::TcpListener, sync::mpsc};

use crate::{
    api::worker::ApiWorkerCtx,
    config::{load_runtime_config, validate_runtime_config, ConfigError, RuntimeConfig},
    dcs::{
        etcd_store::EtcdDcsStore,
        state::{DcsCache, DcsState, DcsTrust, DcsWorkerCtx, MemberRole},
        store::{refresh_from_etcd_watch, DcsStore},
    },
    debug_api::{
        snapshot::{build_snapshot, AppLifecycle, DebugSnapshotCtx},
        worker::{DebugApiContractStubInputs, DebugApiCtx},
    },
    ha::source_conn::basebackup_source_from_member,
    ha::state::{
        HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, ProcessDispatchDefaults,
    },
    logging::{
        AppEvent, AppEventHeader, SeverityText, StructuredFields, SubprocessLineRecord,
        SubprocessStream,
    },
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
    postgres_managed_conf::{managed_standby_auth_from_role_auth, ManagedPostgresStartIntent},
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, ProcessCommandRunner, ProcessExit, ReplicatorSourceConn,
            StartPostgresSpec,
        },
        state::{ProcessJobKind, ProcessState, ProcessWorkerCtx},
        worker::{build_command, system_now_unix_millis, timeout_for_kind, TokioCommandRunner},
    },
    state::{new_state_channel, MemberId, UnixMillis, WorkerStatus},
};

const STARTUP_OUTPUT_DRAIN_MAX_BYTES: usize = 256 * 1024;
const STARTUP_JOB_POLL_INTERVAL: Duration = Duration::from_millis(20);
const PROCESS_WORKER_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Debug)]
enum StartupAction {
    ClaimInitLockAndSeedConfig,
    RunJob(Box<ProcessJobKind>),
    StartPostgres(ManagedPostgresStartIntent),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
    #[error("startup planning failed: {0}")]
    StartupPlanning(String),
    #[error("startup execution failed: {0}")]
    StartupExecution(String),
    #[error("api bind failed at `{listen_addr}`: {message}")]
    ApiBind {
        listen_addr: String,
        message: String,
    },
    #[error("worker failed: {0}")]
    Worker(String),
    #[error("time error: {0}")]
    Time(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StartupMode {
    InitializePrimary {
        start_intent: ManagedPostgresStartIntent,
    },
    CloneReplica {
        leader_member_id: MemberId,
        source: ReplicatorSourceConn,
        start_intent: ManagedPostgresStartIntent,
    },
    ResumeExisting {
        start_intent: ManagedPostgresStartIntent,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DataDirState {
    Missing,
    Empty,
    Existing,
}

#[derive(Clone, Copy, Debug)]
enum RuntimeEventKind {
    StartupEntered,
    DataDirInspected,
    DcsCacheProbe,
    ModeSelected,
    ActionsPlanned,
    Action,
    Phase,
    SubprocessLogEmitFailed,
}

impl RuntimeEventKind {
    fn name(self) -> &'static str {
        match self {
            Self::StartupEntered => "runtime.startup.entered",
            Self::DataDirInspected => "runtime.startup.data_dir.inspected",
            Self::DcsCacheProbe => "runtime.startup.dcs_cache_probe",
            Self::ModeSelected => "runtime.startup.mode_selected",
            Self::ActionsPlanned => "runtime.startup.actions_planned",
            Self::Action => "runtime.startup.action",
            Self::Phase => "runtime.startup.phase",
            Self::SubprocessLogEmitFailed => "runtime.startup.subprocess_log_emit_failed",
        }
    }
}

fn runtime_event(
    kind: RuntimeEventKind,
    result: &str,
    severity: SeverityText,
    message: impl Into<String>,
) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(kind.name(), "runtime", result),
    )
}

fn runtime_base_fields(cfg: &RuntimeConfig, startup_run_id: &str) -> StructuredFields {
    let mut fields = StructuredFields::new();
    fields.insert("scope", cfg.dcs.scope.clone());
    fields.insert("member_id", cfg.cluster.member_id.clone());
    fields.insert("startup_run_id", startup_run_id.to_string());
    fields
}

fn startup_mode_label(startup_mode: &StartupMode) -> String {
    format!("{startup_mode:?}").to_lowercase()
}

fn startup_action_kind_label(action: &StartupAction) -> &'static str {
    match action {
        StartupAction::ClaimInitLockAndSeedConfig => "claim_init_lock_and_seed_config",
        StartupAction::RunJob(_) => "run_job",
        StartupAction::StartPostgres(_) => "start_postgres",
    }
}

pub async fn run_node_from_config_path(path: &Path) -> Result<(), RuntimeError> {
    let cfg = load_runtime_config(path)?;
    run_node_from_config(cfg).await
}

pub async fn run_node_from_config(cfg: RuntimeConfig) -> Result<(), RuntimeError> {
    validate_runtime_config(&cfg)?;

    let logging = crate::logging::bootstrap(&cfg).map_err(|err| {
        RuntimeError::StartupExecution(format!("logging bootstrap failed: {err}"))
    })?;
    let log = logging.handle.clone();
    let startup_run_id = format!(
        "{}-{}",
        cfg.cluster.member_id,
        crate::logging::system_now_unix_millis()
    );
    let mut event = runtime_event(
        RuntimeEventKind::StartupEntered,
        "ok",
        SeverityText::Info,
        "runtime starting",
    );
    let fields = event.fields_mut();
    fields.append_json_map(runtime_base_fields(&cfg, startup_run_id.as_str()).into_attributes());
    fields.insert(
        "logging.level",
        format!("{:?}", cfg.logging.level).to_lowercase(),
    );
    log.emit_app_event("runtime::run_node_from_config", event)
        .map_err(|err| {
            RuntimeError::StartupExecution(format!("runtime start log emit failed: {err}"))
        })?;

    let process_defaults = process_defaults_from_config(&cfg);
    let startup_mode = plan_startup(&cfg, &process_defaults, &log, startup_run_id.as_str())?;
    execute_startup(
        &cfg,
        &process_defaults,
        &startup_mode,
        &log,
        startup_run_id.as_str(),
    )
    .await?;

    run_workers(cfg, process_defaults, log).await
}

fn process_defaults_from_config(cfg: &RuntimeConfig) -> ProcessDispatchDefaults {
    ProcessDispatchDefaults {
        postgres_host: cfg.postgres.listen_host.clone(),
        postgres_port: cfg.postgres.listen_port,
        socket_dir: cfg.postgres.socket_dir.clone(),
        log_file: cfg.postgres.log_file.clone(),
        replicator_username: cfg.postgres.roles.replicator.username.clone(),
        replicator_auth: cfg.postgres.roles.replicator.auth.clone(),
        rewinder_username: cfg.postgres.roles.rewinder.username.clone(),
        rewinder_auth: cfg.postgres.roles.rewinder.auth.clone(),
        remote_dbname: cfg.postgres.rewind_conn_identity.dbname.clone(),
        remote_ssl_mode: cfg.postgres.rewind_conn_identity.ssl_mode,
        connect_timeout_s: cfg.postgres.connect_timeout_s,
        shutdown_mode: crate::process::jobs::ShutdownMode::Fast,
    }
}

fn plan_startup(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
) -> Result<StartupMode, RuntimeError> {
    plan_startup_with_probe(cfg, process_defaults, log, startup_run_id, probe_dcs_cache)
}

fn plan_startup_with_probe(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
    probe: impl Fn(&RuntimeConfig) -> Result<DcsCache, RuntimeError>,
) -> Result<StartupMode, RuntimeError> {
    let data_dir_state = match inspect_data_dir(&cfg.postgres.data_dir) {
        Ok(value) => {
            let mut event = runtime_event(
                RuntimeEventKind::DataDirInspected,
                "ok",
                SeverityText::Debug,
                "data dir inspected",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert(
                "postgres.data_dir",
                cfg.postgres.data_dir.display().to_string(),
            );
            fields.insert("data_dir_state", format!("{value:?}").to_lowercase());
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|err| {
                    RuntimeError::StartupPlanning(format!(
                        "data dir inspection log emit failed: {err}"
                    ))
                })?;
            value
        }
        Err(err) => {
            let mut event = runtime_event(
                RuntimeEventKind::DataDirInspected,
                "failed",
                SeverityText::Error,
                "data dir inspection failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert(
                "postgres.data_dir",
                cfg.postgres.data_dir.display().to_string(),
            );
            fields.insert("error", err.to_string());
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|emit_err| {
                    RuntimeError::StartupPlanning(format!(
                        "data dir inspection log emit failed: {emit_err}"
                    ))
                })?;
            return Err(err);
        }
    };

    let cache = match probe(cfg) {
        Ok(cache) => {
            let mut event = runtime_event(
                RuntimeEventKind::DcsCacheProbe,
                "ok",
                SeverityText::Info,
                "startup dcs cache probe ok",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert("dcs_probe_status", "ok");
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|err| {
                    RuntimeError::StartupPlanning(format!("dcs cache probe log emit failed: {err}"))
                })?;
            Some(cache)
        }
        Err(err) => {
            let mut event = runtime_event(
                RuntimeEventKind::DcsCacheProbe,
                "failed",
                SeverityText::Warn,
                "startup dcs cache probe failed; continuing without cache",
            );
            let fields = event.fields_mut();
            fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
            fields.insert("error", err.to_string());
            fields.insert("dcs_probe_status", "failed");
            log.emit_app_event("runtime::plan_startup", event)
                .map_err(|emit_err| {
                    RuntimeError::StartupPlanning(format!(
                        "dcs cache probe log emit failed: {emit_err}"
                    ))
                })?;
            None
        }
    };

    let startup_mode = select_startup_mode(
        data_dir_state,
        cfg.postgres.data_dir.as_path(),
        cache.as_ref(),
        &cfg.cluster.member_id,
        process_defaults,
    )?;

    let mut event = runtime_event(
        RuntimeEventKind::ModeSelected,
        "ok",
        SeverityText::Info,
        "startup mode selected",
    );
    let fields = event.fields_mut();
    fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
    fields.insert("startup_mode", startup_mode_label(&startup_mode));
    log.emit_app_event("runtime::plan_startup", event)
        .map_err(|err| {
            RuntimeError::StartupPlanning(format!("startup mode log emit failed: {err}"))
        })?;

    Ok(startup_mode)
}

fn inspect_data_dir(data_dir: &Path) -> Result<DataDirState, RuntimeError> {
    match fs::metadata(data_dir) {
        Ok(meta) => {
            if !meta.is_dir() {
                return Err(RuntimeError::StartupPlanning(format!(
                    "postgres.data_dir is not a directory: {}",
                    data_dir.display()
                )));
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DataDirState::Missing);
        }
        Err(err) => {
            return Err(RuntimeError::StartupPlanning(format!(
                "failed to inspect data dir {}: {err}",
                data_dir.display()
            )));
        }
    }

    if data_dir.join("PG_VERSION").exists() {
        return Ok(DataDirState::Existing);
    }

    let mut entries = fs::read_dir(data_dir).map_err(|err| {
        RuntimeError::StartupPlanning(format!(
            "failed to read data dir {}: {err}",
            data_dir.display()
        ))
    })?;

    if entries.next().is_none() {
        Ok(DataDirState::Empty)
    } else {
        Err(RuntimeError::StartupPlanning(format!(
            "ambiguous data dir state: `{}` is non-empty but has no PG_VERSION",
            data_dir.display()
        )))
    }
}

fn probe_dcs_cache(cfg: &RuntimeConfig) -> Result<DcsCache, RuntimeError> {
    let mut store =
        EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &cfg.dcs.scope).map_err(|err| {
            RuntimeError::StartupPlanning(format!("failed to connect dcs for startup probe: {err}"))
        })?;

    let events = store.drain_watch_events().map_err(|err| {
        RuntimeError::StartupPlanning(format!("failed to read startup dcs events: {err}"))
    })?;

    let mut cache = DcsCache {
        members: BTreeMap::new(),
        leader: None,
        switchover: None,
        config: cfg.clone(),
        init_lock: None,
    };

    refresh_from_etcd_watch(&cfg.dcs.scope, &mut cache, events).map_err(|err| {
        RuntimeError::StartupPlanning(format!("failed to decode startup dcs snapshot: {err}"))
    })?;

    Ok(cache)
}

fn select_startup_mode(
    data_dir_state: DataDirState,
    data_dir: &Path,
    cache: Option<&DcsCache>,
    self_member_id: &str,
    process_defaults: &ProcessDispatchDefaults,
) -> Result<StartupMode, RuntimeError> {
    match data_dir_state {
        DataDirState::Existing => Ok(StartupMode::ResumeExisting {
            start_intent: select_resume_start_intent(
                data_dir,
                cache,
                self_member_id,
                process_defaults,
            )?,
        }),
        DataDirState::Missing | DataDirState::Empty => {
            let init_lock_present = cache
                .and_then(|snapshot| snapshot.init_lock.as_ref())
                .is_some();
            let self_member_id = MemberId(self_member_id.to_string());

            let leader = leader_from_leader_key(cache, &self_member_id).or_else(|| {
                if init_lock_present {
                    foreign_healthy_primary_member(cache, &self_member_id)
                } else {
                    None
                }
            });

            match leader {
                Some(leader_member) => {
                    let source = basebackup_source_from_member(
                        &self_member_id,
                        &leader_member,
                        process_defaults,
                    )
                    .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
                    Ok(StartupMode::CloneReplica {
                        leader_member_id: leader_member.member_id.clone(),
                        start_intent: replica_start_intent_from_source(&source, data_dir),
                        source,
                    })
                }
                None => {
                    if init_lock_present {
                        Err(RuntimeError::StartupPlanning(
                            "cluster is already initialized (dcs init lock present) but no healthy primary is available for basebackup"
                                .to_string(),
                        ))
                    } else {
                        Ok(StartupMode::InitializePrimary {
                            start_intent: ManagedPostgresStartIntent::primary(),
                        })
                    }
                }
            }
        }
    }
}

fn select_resume_start_intent(
    data_dir: &Path,
    cache: Option<&DcsCache>,
    self_member_id: &str,
    process_defaults: &ProcessDispatchDefaults,
) -> Result<ManagedPostgresStartIntent, RuntimeError> {
    let self_member_id = MemberId(self_member_id.to_string());
    let existing_managed_replica =
        crate::postgres_managed::read_existing_replica_start_intent(data_dir)
            .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;

    let Some(cache) = cache else {
        if existing_managed_replica.is_some() {
            return Err(RuntimeError::StartupPlanning(
                "existing postgres data dir contains managed replica recovery state but startup dcs cache probe was unavailable; cannot rebuild authoritative startup intent"
                    .to_string(),
            ));
        }
        return Ok(ManagedPostgresStartIntent::primary());
    };

    if cache
        .leader
        .as_ref()
        .map(|record| record.member_id == self_member_id)
        .unwrap_or(false)
    {
        return Ok(ManagedPostgresStartIntent::primary());
    }

    if let Some(leader_member) = leader_from_leader_key(Some(cache), &self_member_id)
        .or_else(|| foreign_healthy_primary_member(Some(cache), &self_member_id))
    {
        let source =
            basebackup_source_from_member(&self_member_id, &leader_member, process_defaults)
                .map_err(|err| RuntimeError::StartupPlanning(err.to_string()))?;
        return Ok(replica_start_intent_from_source(&source, data_dir));
    }

    if local_primary_member(cache, &self_member_id).is_some() {
        return Ok(ManagedPostgresStartIntent::primary());
    }

    if existing_managed_replica.is_some() {
        return Err(RuntimeError::StartupPlanning(
            "existing postgres data dir contains managed replica recovery state but no healthy primary is available in DCS to rebuild authoritative managed config"
                .to_string(),
        ));
    }

    Ok(ManagedPostgresStartIntent::primary())
}

fn leader_from_leader_key(
    cache: Option<&DcsCache>,
    self_member_id: &MemberId,
) -> Option<crate::dcs::state::MemberRecord> {
    let snapshot = cache?;
    let leader_record = snapshot.leader.as_ref()?;
    if leader_record.member_id == *self_member_id {
        return None;
    }
    let member = snapshot.members.get(&leader_record.member_id)?;
    let eligible = member.role == MemberRole::Primary && member.sql == SqlStatus::Healthy;
    if eligible {
        Some(member.clone())
    } else {
        None
    }
}

fn foreign_healthy_primary_member(
    cache: Option<&DcsCache>,
    self_member_id: &MemberId,
) -> Option<crate::dcs::state::MemberRecord> {
    cache?
        .members
        .values()
        .find(|member| {
            member.member_id != *self_member_id
                && member.role == MemberRole::Primary
                && member.sql == SqlStatus::Healthy
        })
        .cloned()
}

fn local_primary_member<'a>(
    cache: &'a DcsCache,
    self_member_id: &MemberId,
) -> Option<&'a crate::dcs::state::MemberRecord> {
    cache
        .members
        .get(self_member_id)
        .filter(|member| member.role == MemberRole::Primary && member.sql == SqlStatus::Healthy)
}

fn replica_start_intent_from_source(
    source: &ReplicatorSourceConn,
    data_dir: &Path,
) -> ManagedPostgresStartIntent {
    ManagedPostgresStartIntent::replica(
        source.conninfo.clone(),
        managed_standby_auth_from_role_auth(&source.auth, data_dir),
        None,
    )
}

fn claim_dcs_init_lock_and_seed_config(cfg: &RuntimeConfig) -> Result<(), String> {
    let init_path = format!("/{}/init", cfg.dcs.scope.trim_matches('/'));
    let config_path = format!("/{}/config", cfg.dcs.scope.trim_matches('/'));

    let mut store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &cfg.dcs.scope)
        .map_err(|err| format!("connect failed: {err}"))?;

    let encoded_init = serde_json::to_string(&crate::dcs::state::InitLockRecord {
        holder: MemberId(cfg.cluster.member_id.clone()),
    })
    .map_err(|err| format!("encode init lock record failed: {err}"))?;

    let claimed = store
        .put_path_if_absent(init_path.as_str(), encoded_init)
        .map_err(|err| format!("init lock write failed at `{init_path}`: {err}"))?;
    if !claimed {
        return Err(format!(
            "cluster already initialized (init lock exists at `{init_path}`)"
        ));
    }

    if let Some(init_cfg) = cfg.dcs.init.as_ref() {
        if init_cfg.write_on_bootstrap {
            let _seeded = store
                .put_path_if_absent(config_path.as_str(), init_cfg.payload_json.clone())
                .map_err(|err| format!("seed config failed at `{config_path}`: {err}"))?;
        }
    }

    Ok(())
}

async fn execute_startup(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    startup_mode: &StartupMode,
    log: &crate::logging::LogHandle,
    startup_run_id: &str,
) -> Result<(), RuntimeError> {
    ensure_start_paths(process_defaults, &cfg.postgres.data_dir)?;

    let actions = build_startup_actions(cfg, startup_mode)?;

    let mut planned_event = runtime_event(
        RuntimeEventKind::ActionsPlanned,
        "ok",
        SeverityText::Debug,
        "startup actions planned",
    );
    let fields = planned_event.fields_mut();
    fields.append_json_map(runtime_base_fields(cfg, startup_run_id).into_attributes());
    fields.insert("startup_mode", startup_mode_label(startup_mode));
    fields.insert("startup_actions_total", actions.len());
    log.emit_app_event("runtime::execute_startup", planned_event)
        .map_err(|err| {
            RuntimeError::StartupExecution(format!("startup actions log emit failed: {err}"))
        })?;

    for (action_index, action) in actions.into_iter().enumerate() {
        let action_kind = startup_action_kind_label(&action);
        let mut action_fields = runtime_base_fields(cfg, startup_run_id);
        action_fields.insert("startup_mode", startup_mode_label(startup_mode));
        action_fields.insert("startup_action_index", action_index);
        action_fields.insert("startup_action_kind", action_kind);
        let mut started_event = runtime_event(
            RuntimeEventKind::Action,
            "started",
            SeverityText::Info,
            "startup action started",
        );
        started_event
            .fields_mut()
            .append_json_map(action_fields.clone().into_attributes());
        log.emit_app_event("runtime::execute_startup", started_event)
            .map_err(|err| {
                RuntimeError::StartupExecution(format!("startup action log emit failed: {err}"))
            })?;

        if let StartupAction::StartPostgres(_) = &action {
            emit_startup_phase(log, "start", "start postgres with managed config").map_err(
                |err| {
                    RuntimeError::StartupExecution(format!("startup phase log emit failed: {err}"))
                },
            )?;
        }

        let result = match action {
            StartupAction::ClaimInitLockAndSeedConfig => {
                claim_dcs_init_lock_and_seed_config(cfg).map_err(|err| {
                    RuntimeError::StartupExecution(format!("dcs init lock claim failed: {err}"))
                })?;
                Ok(())
            }
            StartupAction::RunJob(job) => run_startup_job(cfg, *job, log).await,
            StartupAction::StartPostgres(start_intent) => {
                run_start_job(cfg, process_defaults, &start_intent, log).await
            }
        };

        match result {
            Ok(()) => {
                let mut done_event = runtime_event(
                    RuntimeEventKind::Action,
                    "ok",
                    SeverityText::Info,
                    "startup action completed",
                );
                done_event
                    .fields_mut()
                    .append_json_map(action_fields.into_attributes());
                log.emit_app_event("runtime::execute_startup", done_event)
                    .map_err(|err| {
                        RuntimeError::StartupExecution(format!(
                            "startup action log emit failed: {err}"
                        ))
                    })?;
            }
            Err(err) => {
                let mut failed_event = runtime_event(
                    RuntimeEventKind::Action,
                    "failed",
                    SeverityText::Error,
                    "startup action failed",
                );
                let fields = failed_event.fields_mut();
                fields.append_json_map(action_fields.into_attributes());
                fields.insert("error", err.to_string());
                log.emit_app_event("runtime::execute_startup", failed_event)
                    .map_err(|emit_err| {
                        RuntimeError::StartupExecution(format!(
                            "startup action failure log emit failed: {emit_err}"
                        ))
                    })?;
                return Err(err);
            }
        };
    }

    Ok(())
}

fn emit_startup_phase(
    log: &crate::logging::LogHandle,
    phase: &str,
    detail: &str,
) -> Result<(), crate::logging::LogError> {
    let mut event = runtime_event(
        RuntimeEventKind::Phase,
        "ok",
        SeverityText::Info,
        format!("startup phase={phase} ({detail})"),
    );
    let fields = event.fields_mut();
    fields.insert("startup.phase", phase.to_string());
    fields.insert("startup.detail", detail.to_string());
    log.emit_app_event("startup", event)
}

fn build_startup_actions(
    cfg: &RuntimeConfig,
    startup_mode: &StartupMode,
) -> Result<Vec<StartupAction>, RuntimeError> {
    match startup_mode {
        StartupMode::InitializePrimary { start_intent } => Ok(vec![
            StartupAction::ClaimInitLockAndSeedConfig,
            StartupAction::RunJob(Box::new(ProcessJobKind::Bootstrap(BootstrapSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                superuser_username: cfg.postgres.roles.superuser.username.clone(),
                timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
            }))),
            StartupAction::StartPostgres(start_intent.clone()),
        ]),
        StartupMode::CloneReplica {
            source,
            start_intent,
            ..
        } => Ok(vec![
            StartupAction::RunJob(Box::new(ProcessJobKind::BaseBackup(BaseBackupSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                source: source.clone(),
                timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
            }))),
            StartupAction::StartPostgres(start_intent.clone()),
        ]),
        StartupMode::ResumeExisting { start_intent } => {
            if has_postmaster_pid(&cfg.postgres.data_dir) {
                Ok(Vec::new())
            } else {
                Ok(vec![StartupAction::StartPostgres(start_intent.clone())])
            }
        }
    }
}

fn ensure_start_paths(
    process_defaults: &ProcessDispatchDefaults,
    data_dir: &Path,
) -> Result<(), RuntimeError> {
    if let Some(parent) = data_dir.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to create postgres data dir parent `{}`: {err}",
                parent.display()
            ))
        })?;
    }

    fs::create_dir_all(data_dir).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "failed to create postgres data dir `{}`: {err}",
            data_dir.display()
        ))
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700)).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to set postgres data dir permissions on `{}`: {err}",
                data_dir.display()
            ))
        })?;
    }

    fs::create_dir_all(&process_defaults.socket_dir).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "failed to create postgres socket dir `{}`: {err}",
            process_defaults.socket_dir.display()
        ))
    })?;

    if let Some(log_parent) = process_defaults.log_file.parent() {
        fs::create_dir_all(log_parent).map_err(|err| {
            RuntimeError::StartupExecution(format!(
                "failed to create postgres log dir `{}`: {err}",
                log_parent.display()
            ))
        })?;
    }

    Ok(())
}

fn has_postmaster_pid(data_dir: &Path) -> bool {
    data_dir.join("postmaster.pid").exists()
}

async fn run_start_job(
    cfg: &RuntimeConfig,
    process_defaults: &ProcessDispatchDefaults,
    start_intent: &ManagedPostgresStartIntent,
    log: &crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let managed = crate::postgres_managed::materialize_managed_postgres_config(cfg, start_intent)
        .map_err(|err| {
        RuntimeError::StartupExecution(format!("materialize managed postgres config failed: {err}"))
    })?;
    run_startup_job(
        cfg,
        ProcessJobKind::StartPostgres(StartPostgresSpec {
            data_dir: cfg.postgres.data_dir.clone(),
            config_file: managed.postgresql_conf_path,
            log_file: process_defaults.log_file.clone(),
            wait_seconds: Some(30),
            timeout_ms: Some(cfg.process.bootstrap_timeout_ms),
        }),
        log,
    )
    .await
}

async fn run_startup_job(
    cfg: &RuntimeConfig,
    job: ProcessJobKind,
    log: &crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let mut runner = TokioCommandRunner;
    let timeout_ms = timeout_for_kind(&job, &cfg.process);
    let job_id = crate::state::JobId(format!("startup-{}", std::process::id()));
    let command = build_command(
        &cfg.process,
        &job_id,
        &job,
        cfg.logging.capture_subprocess_output,
    )
    .map_err(|err| {
        RuntimeError::StartupExecution(format!("startup command build failed: {err}"))
    })?;
    let log_identity = command.log_identity.clone();
    let command_display = format!("{} {}", command.program.display(), command.args.join(" "));

    let mut handle = runner.spawn(command).map_err(|err| {
        RuntimeError::StartupExecution(format!(
            "startup command spawn failed for `{command_display}`: {err}"
        ))
    })?;

    let started = system_now_unix_millis().map_err(|err| RuntimeError::Time(err.to_string()))?;
    let deadline = started.0.saturating_add(timeout_ms);

    loop {
        let lines = handle
            .drain_output(STARTUP_OUTPUT_DRAIN_MAX_BYTES)
            .await
            .map_err(|err| {
                RuntimeError::StartupExecution(format!(
                    "startup process output drain failed: {err}"
                ))
            })?;
        for line in lines {
            if let Err(err) = emit_startup_subprocess_line(log, &log_identity, line.clone()) {
                let mut event = runtime_event(
                    RuntimeEventKind::SubprocessLogEmitFailed,
                    "failed",
                    SeverityText::Warn,
                    "startup subprocess line emit failed",
                );
                let fields = event.fields_mut();
                fields.insert("job_id", log_identity.job_id.0.clone());
                fields.insert("job_kind", log_identity.job_kind.clone());
                fields.insert("binary", log_identity.binary.clone());
                fields.insert(
                    "stream",
                    match line.stream {
                        crate::process::jobs::ProcessOutputStream::Stdout => "stdout",
                        crate::process::jobs::ProcessOutputStream::Stderr => "stderr",
                    },
                );
                fields.insert("bytes_len", line.bytes.len());
                fields.insert("error", err.to_string());
                log.emit_app_event("runtime::run_startup_job", event)
                    .map_err(|emit_err| {
                        RuntimeError::StartupExecution(format!(
                            "startup subprocess emit failure log emit failed: {emit_err}"
                        ))
                    })?;
            }
        }

        match handle.poll_exit().map_err(|err| {
            RuntimeError::StartupExecution(format!("startup process poll failed: {err}"))
        })? {
            Some(ProcessExit::Success) => return Ok(()),
            Some(ProcessExit::Failure { code }) => {
                let lines = handle
                    .drain_output(STARTUP_OUTPUT_DRAIN_MAX_BYTES)
                    .await
                    .map_err(|err| {
                        RuntimeError::StartupExecution(format!(
                            "startup process output drain failed: {err}"
                        ))
                    })?;
                for line in lines {
                    emit_startup_subprocess_line(log, &log_identity, line).map_err(|err| {
                        RuntimeError::StartupExecution(format!(
                            "startup subprocess line emit failed: {err}"
                        ))
                    })?;
                }
                return Err(RuntimeError::StartupExecution(format!(
                    "startup command `{command_display}` exited unsuccessfully (code: {code:?})"
                )));
            }
            None => {}
        }

        let now = system_now_unix_millis().map_err(|err| RuntimeError::Time(err.to_string()))?;
        if now.0 >= deadline {
            handle.cancel().await.map_err(|err| {
                RuntimeError::StartupExecution(format!(
                    "startup command `{command_display}` timeout cancellation failed: {err}"
                ))
            })?;
            let lines = handle
                .drain_output(STARTUP_OUTPUT_DRAIN_MAX_BYTES)
                .await
                .map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "startup process output drain failed: {err}"
                    ))
                })?;
            for line in lines {
                emit_startup_subprocess_line(log, &log_identity, line).map_err(|err| {
                    RuntimeError::StartupExecution(format!(
                        "startup subprocess line emit failed: {err}"
                    ))
                })?;
            }
            return Err(RuntimeError::StartupExecution(format!(
                "startup command `{command_display}` timed out after {timeout_ms} ms"
            )));
        }

        tokio::time::sleep(STARTUP_JOB_POLL_INTERVAL).await;
    }
}

fn emit_startup_subprocess_line(
    log: &crate::logging::LogHandle,
    identity: &crate::process::jobs::ProcessLogIdentity,
    line: crate::process::jobs::ProcessOutputLine,
) -> Result<(), crate::logging::LogError> {
    let stream = match line.stream {
        crate::process::jobs::ProcessOutputStream::Stdout => SubprocessStream::Stdout,
        crate::process::jobs::ProcessOutputStream::Stderr => SubprocessStream::Stderr,
    };

    log.emit_raw_record(
        SubprocessLineRecord::new(
            crate::logging::LogProducer::PgTool,
            "startup",
            identity.job_id.0.clone(),
            identity.job_kind.clone(),
            identity.binary.clone(),
            stream,
            line.bytes,
        )
        .into_raw_record()?,
    )
}

async fn run_workers(
    cfg: RuntimeConfig,
    process_defaults: ProcessDispatchDefaults,
    log: crate::logging::LogHandle,
) -> Result<(), RuntimeError> {
    let now = now_unix_millis()?;

    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), now);
    let (pg_publisher, pg_subscriber) = new_state_channel(initial_pg_state(), now);

    let initial_dcs = DcsState {
        worker: WorkerStatus::Starting,
        trust: DcsTrust::NotTrusted,
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg.clone(),
            init_lock: None,
        },
        last_refresh_at: None,
    };
    let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, now);

    let initial_process = ProcessState::Idle {
        worker: WorkerStatus::Starting,
        last_outcome: None,
    };
    let (process_publisher, process_subscriber) = new_state_channel(initial_process.clone(), now);

    let initial_ha = HaState {
        worker: WorkerStatus::Starting,
        phase: HaPhase::Init,
        tick: 0,
        decision: crate::ha::decision::HaDecision::NoChange,
    };
    let (ha_publisher, ha_subscriber) = new_state_channel(initial_ha, now);

    let initial_debug_snapshot = build_snapshot(
        &DebugSnapshotCtx {
            app: AppLifecycle::Running,
            config: cfg_subscriber.latest(),
            pg: pg_subscriber.latest(),
            dcs: dcs_subscriber.latest(),
            process: process_subscriber.latest(),
            ha: ha_subscriber.latest(),
        },
        now,
        0,
        &[],
        &[],
    );
    let (debug_publisher, debug_subscriber) = new_state_channel(initial_debug_snapshot, now);

    let self_id = MemberId(cfg.cluster.member_id.clone());
    let scope = cfg.dcs.scope.clone();

    let pg_ctx = crate::pginfo::state::PgInfoWorkerCtx {
        self_id: self_id.clone(),
        postgres_conninfo: local_postgres_conninfo(
            &process_defaults,
            &cfg.postgres.local_conn_identity,
            cfg.postgres.roles.superuser.username.as_str(),
            cfg.postgres.connect_timeout_s,
        ),
        poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
        publisher: pg_publisher,
        log: log.clone(),
        last_emitted_sql_status: None,
    };

    let dcs_store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &scope)
        .map_err(|err| RuntimeError::Worker(format!("dcs store connect failed: {err}")))?;
    let dcs_ctx = DcsWorkerCtx {
        self_id: self_id.clone(),
        scope: scope.clone(),
        poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
        local_postgres_host: cfg.postgres.listen_host.clone(),
        local_postgres_port: cfg.postgres.listen_port,
        pg_subscriber: pg_subscriber.clone(),
        publisher: dcs_publisher,
        store: Box::new(dcs_store),
        log: log.clone(),
        cache: DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg.clone(),
            init_lock: None,
        },
        last_published_pg_version: None,
        last_emitted_store_healthy: None,
        last_emitted_trust: None,
    };

    let (process_inbox_tx, process_inbox_rx) = mpsc::unbounded_channel();
    let process_ctx = ProcessWorkerCtx {
        poll_interval: PROCESS_WORKER_POLL_INTERVAL,
        config: cfg.process.clone(),
        log: log.clone(),
        capture_subprocess_output: cfg.logging.capture_subprocess_output,
        state: initial_process,
        publisher: process_publisher,
        inbox: process_inbox_rx,
        inbox_disconnected_logged: false,
        command_runner: Box::new(TokioCommandRunner),
        active_runtime: None,
        last_rejection: None,
        now: Box::new(system_now_unix_millis),
    };

    let ha_store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &scope)
        .map_err(|err| RuntimeError::Worker(format!("ha store connect failed: {err}")))?;
    let mut ha_ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
        publisher: ha_publisher,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pg_subscriber.clone(),
        dcs_subscriber: dcs_subscriber.clone(),
        process_subscriber: process_subscriber.clone(),
        process_inbox: process_inbox_tx,
        dcs_store: Box::new(ha_store),
        scope: scope.clone(),
        self_id: self_id.clone(),
    });
    ha_ctx.poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);
    ha_ctx.now = Box::new(system_now_unix_millis);
    ha_ctx.process_defaults = process_defaults;
    ha_ctx.log = log.clone();

    let mut debug_ctx = DebugApiCtx::contract_stub(DebugApiContractStubInputs {
        publisher: debug_publisher,
        config_subscriber: cfg_subscriber.clone(),
        pg_subscriber: pg_subscriber.clone(),
        dcs_subscriber: dcs_subscriber.clone(),
        process_subscriber: process_subscriber.clone(),
        ha_subscriber: ha_subscriber.clone(),
    });
    debug_ctx.app = AppLifecycle::Running;
    debug_ctx.poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);
    debug_ctx.now = Box::new(system_now_unix_millis);

    let api_store = EtcdDcsStore::connect(cfg.dcs.endpoints.clone(), &scope)
        .map_err(|err| RuntimeError::Worker(format!("api store connect failed: {err}")))?;
    let listener = TcpListener::bind(cfg.api.listen_addr.as_str())
        .await
        .map_err(|err| RuntimeError::ApiBind {
            listen_addr: cfg.api.listen_addr.clone(),
            message: err.to_string(),
        })?;
    let mut api_ctx = ApiWorkerCtx::new(listener, cfg_subscriber, Box::new(api_store), log.clone());
    api_ctx.set_ha_snapshot_subscriber(debug_subscriber);
    let server_tls = crate::tls::build_rustls_server_config(&cfg.api.security.tls)
        .map_err(|err| RuntimeError::Worker(format!("api tls config build failed: {err}")))?;
    api_ctx
        .configure_tls(cfg.api.security.tls.mode, server_tls)
        .map_err(|err| RuntimeError::Worker(format!("api tls configure failed: {err}")))?;
    let require_client_cert = match cfg.api.security.tls.client_auth.as_ref() {
        Some(auth) => auth.require_client_cert,
        None => false,
    };
    api_ctx.set_require_client_cert(require_client_cert);

    tokio::try_join!(
        crate::pginfo::worker::run(pg_ctx),
        crate::dcs::worker::run(dcs_ctx),
        crate::process::worker::run(process_ctx),
        crate::logging::postgres_ingest::run(crate::logging::postgres_ingest::build_ctx(
            cfg.clone(),
            log.clone(),
        )),
        crate::ha::worker::run(ha_ctx),
        crate::debug_api::worker::run(debug_ctx),
        crate::api::worker::run(api_ctx),
    )
    .map_err(|err| RuntimeError::Worker(err.to_string()))?;

    Ok(())
}

fn local_postgres_conninfo(
    process_defaults: &ProcessDispatchDefaults,
    identity: &crate::config::PostgresConnIdentityConfig,
    superuser_username: &str,
    connect_timeout_s: u32,
) -> crate::pginfo::state::PgConnInfo {
    crate::pginfo::state::PgConnInfo {
        host: process_defaults.socket_dir.display().to_string(),
        port: process_defaults.postgres_port,
        user: superuser_username.to_string(),
        dbname: identity.dbname.clone(),
        application_name: None,
        connect_timeout_s: Some(connect_timeout_s),
        ssl_mode: identity.ssl_mode,
        options: None,
    }
}

fn initial_pg_state() -> PgInfoState {
    PgInfoState::Unknown {
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
                extra: BTreeMap::new(),
            },
            last_refresh_at: None,
        },
    }
}

fn now_unix_millis() -> Result<UnixMillis, RuntimeError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| RuntimeError::Time(format!("system time before epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| RuntimeError::Time(format!("millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs, io,
        path::PathBuf,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::pginfo::conninfo::PgSslMode;
    use crate::{
        config::{PostgresConfig, RuntimeConfig},
        dcs::state::{DcsCache, LeaderRecord, MemberRecord, MemberRole},
        logging::{decode_app_event, LogHandle, LogSink, SeverityText, TestSink},
        pginfo::state::{Readiness, SqlStatus},
        state::{MemberId, UnixMillis, Version},
    };

    use super::{
        inspect_data_dir, plan_startup_with_probe, process_defaults_from_config,
        select_resume_start_intent, select_startup_mode, DataDirState, StartupMode,
    };
    use crate::postgres_managed_conf::{
        managed_standby_auth_from_role_auth, ManagedPostgresStartIntent,
    };

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_postgres_data_dir(PathBuf::from("/tmp/pgtuskmaster-test-data"))
            .build()
    }

    fn temp_path(label: &str) -> PathBuf {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!(
            "pgtuskmaster-runtime-{label}-{millis}-{}",
            std::process::id()
        ))
    }

    fn remove_if_exists(path: &PathBuf) -> Result<(), io::Error> {
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    fn test_log_handle() -> (LogHandle, Arc<TestSink>) {
        let sink = Arc::new(TestSink::default());
        let sink_dyn: Arc<dyn LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
    }

    #[test]
    fn inspect_data_dir_classifies_missing_empty_and_existing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let missing = temp_path("missing");
        remove_if_exists(&missing)?;
        assert_eq!(inspect_data_dir(&missing)?, DataDirState::Missing);

        let empty = temp_path("empty");
        remove_if_exists(&empty)?;
        fs::create_dir_all(&empty)?;
        assert_eq!(inspect_data_dir(&empty)?, DataDirState::Empty);

        let existing = temp_path("existing");
        remove_if_exists(&existing)?;
        fs::create_dir_all(&existing)?;
        fs::write(existing.join("PG_VERSION"), b"16\n")?;
        assert_eq!(inspect_data_dir(&existing)?, DataDirState::Existing);

        remove_if_exists(&empty)?;
        remove_if_exists(&existing)?;
        Ok(())
    }

    #[test]
    fn plan_startup_emits_data_dir_and_mode_events_without_network_probe(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cfg = sample_runtime_config();
        let dir = temp_path("plan-startup-log");
        remove_if_exists(&dir)?;
        cfg.postgres.data_dir = dir.clone();

        let process_defaults = process_defaults_from_config(&cfg);
        let (log, sink) = test_log_handle();

        let _startup_mode =
            plan_startup_with_probe(&cfg, &process_defaults, &log, "run-1", |_cfg| {
                Ok(DcsCache {
                    members: BTreeMap::new(),
                    leader: None,
                    switchover: None,
                    config: cfg.clone(),
                    init_lock: None,
                })
            })?;

        let inspected = sink.collect_matching(|record| {
            decode_app_event(record)
                .map(|event| event.header.name == "runtime.startup.data_dir.inspected")
                .unwrap_or(false)
        })?;
        if inspected.is_empty() {
            return Err(Box::new(io::Error::other(
                "expected runtime.startup.data_dir.inspected event",
            )));
        }

        let probe = sink.collect_matching(|record| {
            decode_app_event(record)
                .map(|event| event.header.name == "runtime.startup.dcs_cache_probe")
                .unwrap_or(false)
        })?;
        if probe.is_empty() {
            return Err(Box::new(io::Error::other(
                "expected runtime.startup.dcs_cache_probe event",
            )));
        }

        let mode_selected = sink.collect_matching(|record| {
            decode_app_event(record)
                .map(|event| event.header.name == "runtime.startup.mode_selected")
                .unwrap_or(false)
        })?;
        if mode_selected.is_empty() {
            return Err(Box::new(io::Error::other(
                "expected runtime.startup.mode_selected event",
            )));
        }

        remove_if_exists(&dir)?;
        Ok(())
    }

    #[test]
    fn inspect_data_dir_rejects_ambiguous_partial_state() -> Result<(), Box<dyn std::error::Error>>
    {
        let ambiguous = temp_path("ambiguous");
        remove_if_exists(&ambiguous)?;
        fs::create_dir_all(&ambiguous)?;
        fs::write(ambiguous.join("postgresql.conf"), b"# test\n")?;

        let result = inspect_data_dir(&ambiguous);
        assert!(result.is_err());

        remove_if_exists(&ambiguous)?;
        Ok(())
    }

    #[test]
    fn select_startup_mode_prefers_clone_when_foreign_healthy_leader_exists(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let leader_id = MemberId("node-b".to_string());
        let mut members = BTreeMap::new();
        members.insert(
            leader_id.clone(),
            MemberRecord {
                member_id: leader_id.clone(),
                postgres_host: "10.0.0.20".to_string(),
                postgres_port: 5440,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );

        let cache = DcsCache {
            members,
            leader: Some(LeaderRecord {
                member_id: leader_id.clone(),
            }),
            switchover: None,
            config: cfg.clone(),
            init_lock: None,
        };

        let data_dir = temp_path("startup-mode-clone");
        remove_if_exists(&data_dir)?;
        let mode = select_startup_mode(
            DataDirState::Empty,
            &data_dir,
            Some(&cache),
            "node-a",
            &defaults,
        )?;

        assert!(matches!(mode, StartupMode::CloneReplica { .. }));
        if let StartupMode::CloneReplica {
            leader_member_id,
            source,
            ..
        } = mode
        {
            assert_eq!(leader_member_id, leader_id);
            assert_eq!(
                source,
                crate::ha::source_conn::basebackup_source_from_member(
                    &MemberId("node-a".to_string()),
                    cache.members.get(&leader_id).ok_or_else(|| {
                        io::Error::other("leader member missing from startup test cache")
                    })?,
                    &defaults,
                )?
            );
        }
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_initialize_when_no_leader_evidence(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();
        let data_dir = temp_path("startup-mode-init");
        remove_if_exists(&data_dir)?;

        let mode = select_startup_mode(DataDirState::Empty, &data_dir, None, "node-a", &defaults)?;

        assert_eq!(
            mode,
            StartupMode::InitializePrimary {
                start_intent: ManagedPostgresStartIntent::primary(),
            }
        );
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_resume_when_pgdata_exists() -> Result<(), Box<dyn std::error::Error>>
    {
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();
        let data_dir = temp_path("startup-mode-resume");
        remove_if_exists(&data_dir)?;
        let mode =
            select_startup_mode(DataDirState::Existing, &data_dir, None, "node-a", &defaults)?;
        assert_eq!(
            mode,
            StartupMode::ResumeExisting {
                start_intent: ManagedPostgresStartIntent::primary(),
            }
        );
        Ok(())
    }

    #[test]
    fn select_resume_start_intent_prefers_dcs_leader_over_local_auto_conf(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = process_defaults_from_config(&cfg);
        let data_dir = temp_path("resume-dcs-authority");
        remove_if_exists(&data_dir)?;
        fs::create_dir_all(&data_dir)?;

        let runtime_config = RuntimeConfig {
            postgres: PostgresConfig {
                data_dir: data_dir.clone(),
                ..cfg.postgres.clone()
            },
            ..cfg.clone()
        };
        crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &ManagedPostgresStartIntent::replica(
                crate::pginfo::state::PgConnInfo {
                    host: "10.0.0.30".to_string(),
                    port: 5439,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: Some(2),
                    ssl_mode: PgSslMode::Prefer,
                    options: None,
                },
                managed_standby_auth_from_role_auth(
                    &runtime_config.postgres.roles.replicator.auth,
                    &data_dir,
                ),
                Some("slot_local".to_string()),
            ),
        )?;
        fs::write(
            data_dir.join("postgresql.auto.conf"),
            "primary_conninfo = 'host=192.0.2.99 port=6543 user=bad dbname=postgres'\n",
        )?;

        let leader_id = MemberId("node-b".to_string());
        let mut members = BTreeMap::new();
        members.insert(
            leader_id.clone(),
            MemberRecord {
                member_id: leader_id.clone(),
                postgres_host: "10.0.0.20".to_string(),
                postgres_port: 5440,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        let cache = DcsCache {
            members,
            leader: Some(LeaderRecord {
                member_id: leader_id.clone(),
            }),
            switchover: None,
            config: runtime_config.clone(),
            init_lock: None,
        };

        let intent = select_resume_start_intent(&data_dir, Some(&cache), "node-a", &defaults)?;
        let expected_source = crate::ha::source_conn::basebackup_source_from_member(
            &MemberId("node-a".to_string()),
            cache
                .members
                .get(&leader_id)
                .ok_or_else(|| io::Error::other("leader missing from test cache"))?,
            &defaults,
        )?;
        assert_eq!(
            intent,
            ManagedPostgresStartIntent::replica(
                expected_source.conninfo,
                managed_standby_auth_from_role_auth(
                    &expected_source.auth,
                    &data_dir,
                ),
                None,
            )
        );

        remove_if_exists(&data_dir)?;
        Ok(())
    }

    #[test]
    fn select_resume_start_intent_rejects_local_replica_state_without_dcs_authority(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = process_defaults_from_config(&cfg);
        let data_dir = temp_path("resume-without-dcs");
        remove_if_exists(&data_dir)?;
        fs::create_dir_all(&data_dir)?;

        let runtime_config = RuntimeConfig {
            postgres: PostgresConfig {
                data_dir: data_dir.clone(),
                ..cfg.postgres.clone()
            },
            ..cfg.clone()
        };
        crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &ManagedPostgresStartIntent::replica(
                crate::pginfo::state::PgConnInfo {
                    host: "10.0.0.30".to_string(),
                    port: 5439,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: Some(2),
                    ssl_mode: PgSslMode::Prefer,
                    options: None,
                },
                managed_standby_auth_from_role_auth(
                    &runtime_config.postgres.roles.replicator.auth,
                    &data_dir,
                ),
                Some("slot_local".to_string()),
            ),
        )?;

        let result = select_resume_start_intent(&data_dir, None, "node-a", &defaults);
        assert!(matches!(
            result,
            Err(super::RuntimeError::StartupPlanning(_))
        ));

        remove_if_exists(&data_dir)?;
        Ok(())
    }

    #[test]
    fn select_startup_mode_rejects_initialize_when_init_lock_present(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let cache = DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg.clone(),
            init_lock: Some(crate::dcs::state::InitLockRecord {
                holder: MemberId("node-other".to_string()),
            }),
        };

        let data_dir = temp_path("startup-mode-init-lock");
        remove_if_exists(&data_dir)?;
        let result = select_startup_mode(
            DataDirState::Empty,
            &data_dir,
            Some(&cache),
            "node-a",
            &defaults,
        );

        assert!(matches!(
            result,
            Err(super::RuntimeError::StartupPlanning(_))
        ));
        Ok(())
    }

    #[test]
    fn select_startup_mode_uses_member_records_when_init_lock_present_and_leader_missing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = sample_runtime_config();
        let defaults = crate::ha::state::ProcessDispatchDefaults::contract_stub();

        let primary_id = MemberId("node-b".to_string());
        let mut members = BTreeMap::new();
        members.insert(
            primary_id.clone(),
            MemberRecord {
                member_id: primary_id.clone(),
                postgres_host: "10.0.0.21".to_string(),
                postgres_port: 5441,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );

        let cache = DcsCache {
            members,
            leader: None,
            switchover: None,
            config: cfg.clone(),
            init_lock: Some(crate::dcs::state::InitLockRecord {
                holder: MemberId("node-init".to_string()),
            }),
        };

        let data_dir = temp_path("startup-mode-member-fallback");
        remove_if_exists(&data_dir)?;
        let mode = select_startup_mode(
            DataDirState::Empty,
            &data_dir,
            Some(&cache),
            "node-a",
            &defaults,
        )?;

        assert!(matches!(mode, StartupMode::CloneReplica { .. }));
        Ok(())
    }

    #[test]
    fn runtime_uses_role_specific_users_for_dsn_clone_and_rewind(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cfg = sample_runtime_config();
        cfg.postgres.roles.superuser.username = "su_admin".to_string();
        cfg.postgres.roles.replicator.username = "repl_user".to_string();
        cfg.postgres.roles.rewinder.username = "rewind_user".to_string();
        cfg.postgres.local_conn_identity.user = "su_admin".to_string();
        cfg.postgres.rewind_conn_identity.user = "rewind_user".to_string();

        let defaults = super::process_defaults_from_config(&cfg);
        assert_eq!(defaults.replicator_username, "repl_user");
        assert_eq!(defaults.rewinder_username, "rewind_user");

        let local_conninfo = super::local_postgres_conninfo(
            &defaults,
            &cfg.postgres.local_conn_identity,
            cfg.postgres.roles.superuser.username.as_str(),
            cfg.postgres.connect_timeout_s,
        );
        assert_eq!(local_conninfo.user, "su_admin");

        let leader_source = crate::ha::source_conn::basebackup_source_from_member(
            &MemberId("node-a".to_string()),
            &MemberRecord {
                member_id: MemberId("node-b".to_string()),
                postgres_host: "10.0.0.30".to_string(),
                postgres_port: 5442,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
            &defaults,
        )?;
        assert_eq!(leader_source.conninfo.user, "repl_user");
        Ok(())
    }
}

--- END FILE: src/runtime/node.rs ---

