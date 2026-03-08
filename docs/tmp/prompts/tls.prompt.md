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
- docs/src/reference/tls.md

[Page goal]
- Reference the TLS config types, parser validation rules, rustls server-config assembly, runtime wiring, and API accept behavior.

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
- Overview\n- Config types\n- Parser validation surface\n- Rustls builder behavior\n- Runtime wiring\n- API worker TLS behavior\n- Error variants and constants

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

# TLS Reference

Describes the TLS machinery: configuration schema, validation, rustls server-config assembly, runtime wiring, and API connection handling.

## Config Schema

### `InlineOrPath`

Untagged enum for certificate material.

| Variant | Fields | Behavior |
|---|---|---|
| `Path` | `PathBuf` | reads file content bytes |
| `PathConfig` | `{ path: PathBuf }` | reads file content bytes |
| `Inline` | `{ content: String }` | uses UTF-8 bytes of `content` |

### `ApiTlsMode`

Lowercase serde representation.

| Variant | String |
|---|---|
| `Disabled` | `disabled` |
| `Optional` | `optional` |
| `Required` | `required` |

### `TlsServerIdentityConfig`

| Field | Type |
|---|---|
| `cert_chain` | `InlineOrPath` |
| `private_key` | `InlineOrPath` |

### `TlsClientAuthConfig`

| Field | Type |
|---|---|
| `client_ca` | `InlineOrPath` |
| `require_client_cert` | `bool` |

### `TlsServerConfig`

Used by `PostgresConfig.tls` and `ApiSecurityConfig.tls`.

| Field | Type |
|---|---|
| `mode` | `ApiTlsMode` |
| `identity` | `Option<TlsServerIdentityConfig>` |
| `client_auth` | `Option<TlsClientAuthConfig>` |

## Validation

### `validate_tls_server_config`

Returns `Ok(())` when `mode` is `Disabled`.

For `Optional` and `Required` modes:

- `identity` must be present or validation fails with the message `tls identity must be configured when tls.mode is optional or required`
- `cert_chain` and `private_key` are validated with `validate_inline_or_path_non_empty(..., allow_empty_inline = false)`

### `validate_tls_client_auth_config`

Returns `Ok(())` when `client_auth` is absent.

When `client_auth` is present:

- `mode = Disabled` fails with the message `must not be configured when tls.mode is disabled`
- `client_ca` is validated with `validate_inline_or_path_non_empty(..., allow_empty_inline = false)`

### `validate_postgres_conn_identity_ssl_mode_supported`

Rejects PostgreSQL connection `ssl_mode` values `require`, `verify-ca`, and `verify-full` when `postgres.tls.mode` is `Disabled`.

## Rustls Server-Config Assembly

`build_rustls_server_config(&TlsServerConfig)` returns `Result<Option<Arc<rustls::ServerConfig>>, TlsConfigError>`.

### Mode Handling

| `mode` | Returns | Requirement |
|---|---|---|
| `Disabled` | `Ok(None)` | none |
| `Optional` or `Required` | `Ok(Some(config))` | `identity` must be present |

When `identity` is absent for enabled modes, the builder returns `TlsConfigError::InvalidConfig { message }` with the message `tls.identity must be configured when tls.mode is optional or required`.

### PEM Processing

- `load_inline_or_path_bytes` reads files for `Path` and `PathConfig` and returns `content.as_bytes().to_vec()` for `Inline`
- File read failures map to `TlsConfigError::Io` with field-qualified messages
- `parse_pem_cert_chain` uses `rustls_pemfile::certs`, maps parser failures to `TlsConfigError::PemParse`, and rejects empty certificate chains with `no certificates found in PEM input`
- `parse_pem_private_key` uses `rustls_pemfile::private_key`, maps parser failures to `TlsConfigError::PemParse`, and rejects missing keys with `no private key found in PEM input`

### Config Builder

The builder uses `rustls::crypto::ring::default_provider()` and `with_safe_default_protocol_versions()`.

Without `client_auth`:

- `with_no_client_auth()`
- `with_single_cert(cert_chain, key)`

With `client_auth`:

- load `client_ca` via `load_inline_or_path_bytes("tls.client_auth.client_ca", ...)`
- parse the CA certificate chain
- insert each certificate into `RootCertStore`
- build `WebPkiClientVerifier::builder_with_provider(...)`
- call `allow_unauthenticated()` when `require_client_cert` is `false`
- use `with_client_cert_verifier(...)` and `with_single_cert(cert_chain, key)`

Root-store insertion, verifier build, protocol-version build, and certificate attachment failures map to `TlsConfigError::Rustls`.

### `TlsConfigError`

| Variant | Source |
|---|---|
| `InvalidConfig { message }` | invalid builder inputs |
| `Io { message }` | file read failures |
| `PemParse { message }` | PEM parse failure or missing PEM material |
| `Rustls { message }` | rustls builder, root store, verifier, or certificate attachment failure |

## Runtime Wiring

In `src/runtime/node.rs`, `run_workers(...)` applies API TLS in this order:

1. `build_rustls_server_config(&cfg.api.security.tls)`
2. `api_ctx.configure_tls(cfg.api.security.tls.mode, server_tls)`
3. compute `require_client_cert` from `cfg.api.security.tls.client_auth.require_client_cert`, defaulting to `false`
4. `api_ctx.set_require_client_cert(require_client_cert)`

Builder failures map to `RuntimeError::Worker("api tls config build failed: {err}")`.

Configure failures map to `RuntimeError::Worker("api tls configure failed: {err}")`.

## API Worker Accept Behavior

### `ApiWorkerCtx` State

| Field | Type | Purpose |
|---|---|---|
| `tls_mode_override` | `Option<ApiTlsMode>` | runtime mode override |
| `tls_acceptor` | `Option<TlsAcceptor>` | stored acceptor for TLS handshakes |
| `require_client_cert` | `bool` | post-handshake certificate enforcement |

### `configure_tls`

| Mode | `server_config` | Behavior |
|---|---|---|
| `Disabled` | any | stores `Disabled` override and clears the acceptor |
| `Optional` or `Required` | `None` | returns an error |
| `Optional` or `Required` | `Some(...)` | stores the mode override and builds `TlsAcceptor` |

### `effective_tls_mode`

Returns `tls_mode_override` when present. Otherwise returns `cfg.api.security.tls.mode`.

### `require_tls_acceptor`

Returns `WorkerError::Message("tls mode requires a configured tls acceptor")` when the runtime mode requires TLS but the acceptor is missing.

### `accept_connection`

| Mode | Plain TCP | TLS handshake | Post-handshake cert check |
|---|---|---|---|
| `Disabled` | accepted | not performed | not performed |
| `Required` | rejected | performed; on failure emits `api.tls_handshake_failed` with `Warn` severity, peer address, mode, and error, then drops the connection | if `require_client_cert` is `true` and no peer certificate is present, emits `api.tls_client_cert_missing` with `Warn` severity and drops the connection |
| `Optional` | peeks for a TLS ClientHello for up to `API_TLS_CLIENT_HELLO_PEEK_TIMEOUT`; accepts plain connections when no ClientHello is detected | if a ClientHello is detected, performs the handshake; on failure emits `api.tls_handshake_failed` with `Warn` severity, peer address, mode, and error, then drops the connection | applies the same post-handshake client-cert rule as `Required` |

`API_TLS_CLIENT_HELLO_PEEK_TIMEOUT` is `10 ms`.

### `looks_like_tls_client_hello`

- returns `false` on timeout, EOF, and `WouldBlock`
- returns `true` when the first peeked byte is `0x16`
- maps other I/O failures to `WorkerError::Message("api tls peek failed: {err}")`

### `has_peer_client_cert`

Returns `true` only when `peer_certificates()` is present and non-empty.

## Verified Behaviors

Tests in `src/tls.rs` verify:

- optional mode without identity is rejected
- required mode with inline identity and optional client auth builds successfully
- missing certificate path maps to `TlsConfigError::Io`
- invalid certificate chain maps to `TlsConfigError::PemParse`
- invalid private key maps to `TlsConfigError::PemParse`

Tests in `src/config/parser.rs` verify:

- `client_auth` is rejected when `tls.mode` is `disabled`

Tests in `src/api/worker.rs` verify:

- optional TLS accepts both plain TCP HTTP requests and TLS HTTP requests
- required TLS accepts TLS requests and rejects plain TCP requests
- required TLS also works when the server config is built through `build_rustls_server_config`
- required mTLS built through `build_rustls_server_config` accepts a trusted client certificate and rejects missing or untrusted client certificates
- wrong CA, wrong hostname, and expired server certificate handshakes fail

[Repo facts and source excerpts]

--- BEGIN FILE: src/tls.rs ---
use std::{fs, io::Cursor, sync::Arc};

use thiserror::Error;

use crate::config::{ApiTlsMode, InlineOrPath, TlsClientAuthConfig, TlsServerConfig};
use rustls::server::danger::ClientCertVerifier;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum TlsConfigError {
    #[error("invalid config: {message}")]
    InvalidConfig { message: String },
    #[error("io error: {message}")]
    Io { message: String },
    #[error("pem parse error: {message}")]
    PemParse { message: String },
    #[error("rustls error: {message}")]
    Rustls { message: String },
}

pub(crate) fn build_rustls_server_config(
    tls: &TlsServerConfig,
) -> Result<Option<Arc<rustls::ServerConfig>>, TlsConfigError> {
    if matches!(tls.mode, ApiTlsMode::Disabled) {
        return Ok(None);
    }

    let identity = tls
        .identity
        .as_ref()
        .ok_or_else(|| TlsConfigError::InvalidConfig {
            message: "tls.identity must be configured when tls.mode is optional or required"
                .to_string(),
        })?;

    let cert_pem = load_inline_or_path_bytes("tls.identity.cert_chain", &identity.cert_chain)?;
    let key_pem = load_inline_or_path_bytes("tls.identity.private_key", &identity.private_key)?;

    let cert_chain = parse_pem_cert_chain(cert_pem.as_slice())?;
    let key = parse_pem_private_key(key_pem.as_slice())?;

    let provider = rustls::crypto::ring::default_provider();
    let builder = rustls::ServerConfig::builder_with_provider(provider.into())
        .with_safe_default_protocol_versions()
        .map_err(|err| TlsConfigError::Rustls {
            message: format!("build server config failed: {err}"),
        })?;

    let config = match tls.client_auth.as_ref() {
        None => builder
            .with_no_client_auth()
            .with_single_cert(cert_chain, key)
            .map_err(|err| TlsConfigError::Rustls {
                message: format!("configure server cert/key failed: {err}"),
            })?,
        Some(client_auth) => {
            let verifier = build_client_verifier(client_auth)?;
            builder
                .with_client_cert_verifier(verifier)
                .with_single_cert(cert_chain, key)
                .map_err(|err| TlsConfigError::Rustls {
                    message: format!("configure server cert/key failed: {err}"),
                })?
        }
    };

    Ok(Some(Arc::new(config)))
}

fn build_client_verifier(
    client_auth: &TlsClientAuthConfig,
) -> Result<Arc<dyn ClientCertVerifier>, TlsConfigError> {
    let ca_pem = load_inline_or_path_bytes("tls.client_auth.client_ca", &client_auth.client_ca)?;
    let ca_certs = parse_pem_cert_chain(ca_pem.as_slice())?;

    let mut roots = rustls::RootCertStore::empty();
    for cert in ca_certs {
        roots.add(cert).map_err(|err| TlsConfigError::Rustls {
            message: format!("add client ca cert failed: {err}"),
        })?;
    }

    let provider = rustls::crypto::ring::default_provider();
    let mut verifier_builder = rustls::server::WebPkiClientVerifier::builder_with_provider(
        Arc::new(roots),
        provider.into(),
    );
    if !client_auth.require_client_cert {
        verifier_builder = verifier_builder.allow_unauthenticated();
    }

    verifier_builder
        .build()
        .map_err(|err| TlsConfigError::Rustls {
            message: format!("build client cert verifier failed: {err}"),
        })
}

fn parse_pem_cert_chain(
    pem: &[u8],
) -> Result<Vec<rustls::pki_types::CertificateDer<'static>>, TlsConfigError> {
    let mut reader = std::io::BufReader::new(Cursor::new(pem));
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| TlsConfigError::PemParse {
            message: format!("parse certs failed: {err}"),
        })?;
    if certs.is_empty() {
        return Err(TlsConfigError::PemParse {
            message: "no certificates found in PEM input".to_string(),
        });
    }
    Ok(certs)
}

fn parse_pem_private_key(
    pem: &[u8],
) -> Result<rustls::pki_types::PrivateKeyDer<'static>, TlsConfigError> {
    let mut reader = std::io::BufReader::new(Cursor::new(pem));
    let key = rustls_pemfile::private_key(&mut reader)
        .map_err(|err| TlsConfigError::PemParse {
            message: format!("parse private key failed: {err}"),
        })?
        .ok_or_else(|| TlsConfigError::PemParse {
            message: "no private key found in PEM input".to_string(),
        })?;
    Ok(key)
}

fn load_inline_or_path_bytes(
    field: &str,
    source: &InlineOrPath,
) -> Result<Vec<u8>, TlsConfigError> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => {
            fs::read(path).map_err(|err| TlsConfigError::Io {
                message: format!("failed to read `{field}` from {}: {err}", path.display()),
            })
        }
        InlineOrPath::Inline { content } => Ok(content.as_bytes().to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        config::{
            ApiTlsMode, InlineOrPath, TlsClientAuthConfig, TlsServerConfig, TlsServerIdentityConfig,
        },
        test_harness::tls::build_adversarial_tls_fixture,
    };

    use super::build_rustls_server_config;

    #[test]
    fn build_rustls_server_config_rejects_optional_without_identity() {
        let cfg = TlsServerConfig {
            mode: ApiTlsMode::Optional,
            identity: None,
            client_auth: None,
        };
        let result = build_rustls_server_config(&cfg);
        assert!(result.is_err());
    }

    #[test]
    fn build_rustls_server_config_accepts_inline_identity_and_optional_client_auth(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let cfg = TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: fixture.valid_server.key_pem.clone(),
                },
            }),
            client_auth: Some(TlsClientAuthConfig {
                client_ca: InlineOrPath::Inline {
                    content: fixture.trusted_client_ca.cert.cert_pem.clone(),
                },
                require_client_cert: false,
            }),
        };

        let built = build_rustls_server_config(&cfg)?;
        assert!(built.is_some());
        Ok(())
    }

    #[test]
    fn build_rustls_server_config_reports_io_error_when_cert_path_missing() {
        let cfg = TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Path(PathBuf::from(
                    "/tmp/pgtuskmaster-missing-cert-chain.pem",
                )),
                private_key: InlineOrPath::Path(PathBuf::from(
                    "/tmp/pgtuskmaster-missing-private-key.pem",
                )),
            }),
            client_auth: None,
        };

        let result = build_rustls_server_config(&cfg);
        assert!(matches!(result, Err(super::TlsConfigError::Io { .. })));
    }

    #[test]
    fn build_rustls_server_config_reports_pem_error_for_invalid_cert_chain() {
        let cfg = TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: "not-a-cert".to_string(),
                },
                private_key: InlineOrPath::Inline {
                    content: "not-a-key".to_string(),
                },
            }),
            client_auth: None,
        };

        let result = build_rustls_server_config(&cfg);
        assert!(matches!(
            result,
            Err(super::TlsConfigError::PemParse { .. })
        ));
    }

    #[test]
    fn build_rustls_server_config_reports_pem_error_for_invalid_private_key(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let fixture = build_adversarial_tls_fixture()?;
        let cfg = TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: "not-a-private-key".to_string(),
                },
            }),
            client_auth: None,
        };

        let result = build_rustls_server_config(&cfg);
        assert!(matches!(
            result,
            Err(super::TlsConfigError::PemParse { .. })
        ));
        Ok(())
    }
}

--- END FILE: src/tls.rs ---

--- BEGIN FILE: src/config/parser.rs ---
use std::path::{Path, PathBuf};

use thiserror::Error;

use super::defaults::{
    default_api_listen_addr, default_debug_config, default_logging_config,
    default_postgres_connect_timeout_s, normalize_process_config,
};
use super::schema::{
    ApiConfig, ApiSecurityConfig, ConfigVersion, InlineOrPath, PgHbaConfig, PgIdentConfig,
    PostgresConfig, PostgresConnIdentityConfig, PostgresRoleConfig, PostgresRolesConfig,
    RoleAuthConfig, RoleAuthConfigV2Input, RuntimeConfig, RuntimeConfigV2Input, SecretSource,
    TlsServerConfig, TlsServerIdentityConfig,
};
use crate::postgres_managed_conf::{validate_extra_guc_entry, ManagedPostgresConfError};

const MIN_TIMEOUT_MS: u64 = 1;
const MAX_TIMEOUT_MS: u64 = 86_400_000;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config file {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid config field `{field}`: {message}")]
    Validation {
        field: &'static str,
        message: String,
    },
}

pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfig, ConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.display().to_string(),
        source,
    })?;

    #[derive(serde::Deserialize)]
    struct ConfigEnvelope {
        config_version: Option<ConfigVersion>,
    }

    let envelope: ConfigEnvelope =
        toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.display().to_string(),
            source,
        })?;

    let config_version = envelope.config_version.ok_or_else(|| ConfigError::Validation {
        field: "config_version",
        message: "missing required field; set config_version = \"v2\" to use the explicit secure schema".to_string(),
    })?;

    match config_version {
        ConfigVersion::V1 => {
            probe_legacy_v1_shape_for_diagnostics(&contents);
            Err(ConfigError::Validation {
                field: "config_version",
                message: "config_version = \"v1\" is no longer supported because it depends on implicit security defaults; migrate to config_version = \"v2\""
                    .to_string(),
            })
        }
        ConfigVersion::V2 => {
            let raw: RuntimeConfigV2Input =
                toml::from_str(&contents).map_err(|source| ConfigError::Parse {
                    path: path.display().to_string(),
                    source,
                })?;
            let cfg = normalize_runtime_config_v2(raw)?;
            validate_runtime_config(&cfg)?;
            Ok(cfg)
        }
    }
}

fn probe_legacy_v1_shape_for_diagnostics(contents: &str) {
    // We keep the legacy v1 deserialization surface "alive" to:
    // - avoid unused-schema drift during the transition
    // - allow future improvements that surface rich TOML diagnostics for v1 migrations
    //
    // This must never override the v1 migration guidance with a parse error.
    let parsed: Result<toml::Value, toml::de::Error> = toml::from_str(contents);
    let Ok(mut value) = parsed else {
        return;
    };

    let Some(table) = value.as_table_mut() else {
        return;
    };

    let _ = table.remove("config_version");

    let _: Result<super::schema::PartialRuntimeConfig, toml::de::Error> = value.try_into();
}

fn normalize_runtime_config_v2(input: RuntimeConfigV2Input) -> Result<RuntimeConfig, ConfigError> {
    if !matches!(input.config_version, ConfigVersion::V2) {
        return Err(ConfigError::Validation {
            field: "config_version",
            message: "expected config_version = \"v2\"".to_string(),
        });
    }

    let postgres = normalize_postgres_config_v2(input.postgres)?;
    let process = normalize_process_config(input.process)?;
    let logging = input.logging.unwrap_or_else(default_logging_config);
    let api = normalize_api_config_v2(input.api)?;
    let debug = input.debug.unwrap_or_else(default_debug_config);

    Ok(RuntimeConfig {
        cluster: input.cluster,
        postgres,
        dcs: input.dcs,
        ha: input.ha,
        process,
        logging,
        api,
        debug,
    })
}

fn normalize_postgres_config_v2(
    input: super::schema::PostgresConfigV2Input,
) -> Result<PostgresConfig, ConfigError> {
    let connect_timeout_s = input
        .connect_timeout_s
        .unwrap_or_else(default_postgres_connect_timeout_s);

    let local_conn_identity = normalize_postgres_conn_identity_v2(
        "postgres.local_conn_identity",
        input.local_conn_identity,
    )?;
    let rewind_conn_identity = normalize_postgres_conn_identity_v2(
        "postgres.rewind_conn_identity",
        input.rewind_conn_identity,
    )?;

    let tls = normalize_tls_server_config_v2("postgres.tls", input.tls)?;
    let roles = normalize_postgres_roles_v2(input.roles)?;
    let pg_hba = normalize_pg_hba_v2(input.pg_hba)?;
    let pg_ident = normalize_pg_ident_v2(input.pg_ident)?;

    Ok(PostgresConfig {
        data_dir: input.data_dir,
        connect_timeout_s,
        listen_host: input.listen_host,
        listen_port: input.listen_port,
        socket_dir: input.socket_dir,
        log_file: input.log_file,
        local_conn_identity,
        rewind_conn_identity,
        tls,
        roles,
        pg_hba,
        pg_ident,
        extra_gucs: normalize_postgres_extra_gucs_v2(input.extra_gucs)?,
    })
}

fn normalize_postgres_extra_gucs_v2(
    input: Option<std::collections::BTreeMap<String, String>>,
) -> Result<std::collections::BTreeMap<String, String>, ConfigError> {
    let extra_gucs = input.unwrap_or_default();
    for (key, value) in &extra_gucs {
        validate_extra_guc_for_config(key.as_str(), value.as_str())?;
    }
    Ok(extra_gucs)
}

fn normalize_postgres_conn_identity_v2(
    field_prefix: &'static str,
    input: Option<super::schema::PostgresConnIdentityConfigV2Input>,
) -> Result<PostgresConnIdentityConfig, ConfigError> {
    let identity = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let user_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.user",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.user",
        _ => field_prefix,
    };
    let dbname_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.dbname",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.dbname",
        _ => field_prefix,
    };
    let ssl_mode_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.ssl_mode",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.ssl_mode",
        _ => field_prefix,
    };

    let user = identity.user.ok_or_else(|| ConfigError::Validation {
        field: user_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(user_field, user.as_str())?;

    let dbname = identity.dbname.ok_or_else(|| ConfigError::Validation {
        field: dbname_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(dbname_field, dbname.as_str())?;

    let ssl_mode = identity.ssl_mode.ok_or_else(|| ConfigError::Validation {
        field: ssl_mode_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(PostgresConnIdentityConfig {
        user,
        dbname,
        ssl_mode,
    })
}

fn normalize_postgres_roles_v2(
    input: Option<super::schema::PostgresRolesConfigV2Input>,
) -> Result<PostgresRolesConfig, ConfigError> {
    let roles = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.roles",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let superuser = normalize_postgres_role_v2("postgres.roles.superuser", roles.superuser)?;
    let replicator = normalize_postgres_role_v2("postgres.roles.replicator", roles.replicator)?;
    let rewinder = normalize_postgres_role_v2("postgres.roles.rewinder", roles.rewinder)?;

    Ok(PostgresRolesConfig {
        superuser,
        replicator,
        rewinder,
    })
}

fn normalize_postgres_role_v2(
    field_prefix: &'static str,
    input: Option<super::schema::PostgresRoleConfigV2Input>,
) -> Result<PostgresRoleConfig, ConfigError> {
    let role = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let username_field = match field_prefix {
        "postgres.roles.superuser" => "postgres.roles.superuser.username",
        "postgres.roles.replicator" => "postgres.roles.replicator.username",
        "postgres.roles.rewinder" => "postgres.roles.rewinder.username",
        _ => field_prefix,
    };
    let auth_field = match field_prefix {
        "postgres.roles.superuser" => "postgres.roles.superuser.auth",
        "postgres.roles.replicator" => "postgres.roles.replicator.auth",
        "postgres.roles.rewinder" => "postgres.roles.rewinder.auth",
        _ => field_prefix,
    };

    let username = role.username.ok_or_else(|| ConfigError::Validation {
        field: username_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(username_field, username.as_str())?;

    let auth = role.auth.ok_or_else(|| ConfigError::Validation {
        field: auth_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    let auth = normalize_role_auth_config_v2(auth_field, auth)?;

    Ok(PostgresRoleConfig { username, auth })
}

fn normalize_role_auth_config_v2(
    field_prefix: &'static str,
    input: RoleAuthConfigV2Input,
) -> Result<RoleAuthConfig, ConfigError> {
    match input {
        RoleAuthConfigV2Input::Tls => Ok(RoleAuthConfig::Tls),
        RoleAuthConfigV2Input::Password { password } => {
            let password_field = match field_prefix {
                "postgres.roles.superuser.auth" => "postgres.roles.superuser.auth.password",
                "postgres.roles.replicator.auth" => "postgres.roles.replicator.auth.password",
                "postgres.roles.rewinder.auth" => "postgres.roles.rewinder.auth.password",
                _ => field_prefix,
            };

            let password = password.ok_or_else(|| ConfigError::Validation {
                field: password_field,
                message: "missing required secure field for config_version=v2".to_string(),
            })?;

            Ok(RoleAuthConfig::Password { password })
        }
    }
}

fn normalize_pg_hba_v2(
    input: Option<super::schema::PgHbaConfigV2Input>,
) -> Result<PgHbaConfig, ConfigError> {
    let cfg = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_hba",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;
    let source = cfg.source.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_hba.source",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    Ok(PgHbaConfig { source })
}

fn normalize_pg_ident_v2(
    input: Option<super::schema::PgIdentConfigV2Input>,
) -> Result<PgIdentConfig, ConfigError> {
    let cfg = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_ident",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;
    let source = cfg.source.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_ident.source",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    Ok(PgIdentConfig { source })
}

fn normalize_api_config_v2(
    input: super::schema::ApiConfigV2Input,
) -> Result<ApiConfig, ConfigError> {
    let listen_addr = input.listen_addr.unwrap_or_else(default_api_listen_addr);

    let security = input.security.ok_or_else(|| ConfigError::Validation {
        field: "api.security",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let tls = normalize_tls_server_config_v2("api.security.tls", security.tls)?;
    let auth = security.auth.ok_or_else(|| ConfigError::Validation {
        field: "api.security.auth",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(ApiConfig {
        listen_addr,
        security: ApiSecurityConfig { tls, auth },
    })
}

fn normalize_tls_server_config_v2(
    field_prefix: &'static str,
    input: Option<super::schema::TlsServerConfigV2Input>,
) -> Result<TlsServerConfig, ConfigError> {
    let tls = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let mode_field = match field_prefix {
        "postgres.tls" => "postgres.tls.mode",
        "api.security.tls" => "api.security.tls.mode",
        _ => field_prefix,
    };
    let identity_field = match field_prefix {
        "postgres.tls" => "postgres.tls.identity",
        "api.security.tls" => "api.security.tls.identity",
        _ => field_prefix,
    };

    let mode = tls.mode.ok_or_else(|| ConfigError::Validation {
        field: mode_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    let identity = match tls.identity {
        None => None,
        Some(identity) => Some(normalize_tls_server_identity_v2(identity_field, identity)?),
    };

    Ok(TlsServerConfig {
        mode,
        identity,
        client_auth: tls.client_auth,
    })
}

fn normalize_tls_server_identity_v2(
    field_prefix: &'static str,
    input: super::schema::TlsServerIdentityConfigV2Input,
) -> Result<TlsServerIdentityConfig, ConfigError> {
    let cert_chain_field = match field_prefix {
        "postgres.tls.identity" => "postgres.tls.identity.cert_chain",
        "api.security.tls.identity" => "api.security.tls.identity.cert_chain",
        _ => field_prefix,
    };
    let private_key_field = match field_prefix {
        "postgres.tls.identity" => "postgres.tls.identity.private_key",
        "api.security.tls.identity" => "api.security.tls.identity.private_key",
        _ => field_prefix,
    };

    let cert_chain = input.cert_chain.ok_or_else(|| ConfigError::Validation {
        field: cert_chain_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    let private_key = input.private_key.ok_or_else(|| ConfigError::Validation {
        field: private_key_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(TlsServerIdentityConfig {
        cert_chain,
        private_key,
    })
}

fn validate_absolute_path(field: &'static str, path: &Path) -> Result<(), ConfigError> {
    if !path.is_absolute() {
        return Err(ConfigError::Validation {
            field,
            message: "must be an absolute path".to_string(),
        });
    }
    Ok(())
}

fn normalize_path_lexical(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn validate_runtime_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    validate_non_empty_path("postgres.data_dir", &cfg.postgres.data_dir)?;
    validate_non_empty("postgres.listen_host", cfg.postgres.listen_host.as_str())?;
    validate_port("postgres.listen_port", cfg.postgres.listen_port)?;
    validate_non_empty_path("postgres.socket_dir", &cfg.postgres.socket_dir)?;
    validate_non_empty_path("postgres.log_file", &cfg.postgres.log_file)?;

    validate_non_empty(
        "postgres.local_conn_identity.user",
        cfg.postgres.local_conn_identity.user.as_str(),
    )?;
    validate_non_empty(
        "postgres.local_conn_identity.dbname",
        cfg.postgres.local_conn_identity.dbname.as_str(),
    )?;
    validate_non_empty(
        "postgres.rewind_conn_identity.user",
        cfg.postgres.rewind_conn_identity.user.as_str(),
    )?;
    validate_non_empty(
        "postgres.rewind_conn_identity.dbname",
        cfg.postgres.rewind_conn_identity.dbname.as_str(),
    )?;

    validate_non_empty(
        "postgres.roles.superuser.username",
        cfg.postgres.roles.superuser.username.as_str(),
    )?;
    validate_non_empty(
        "postgres.roles.replicator.username",
        cfg.postgres.roles.replicator.username.as_str(),
    )?;
    validate_non_empty(
        "postgres.roles.rewinder.username",
        cfg.postgres.roles.rewinder.username.as_str(),
    )?;

    if cfg.postgres.local_conn_identity.user != cfg.postgres.roles.superuser.username {
        return Err(ConfigError::Validation {
            field: "postgres.local_conn_identity.user",
            message: format!(
                "must match postgres.roles.superuser.username (got `{}`, expected `{}`)",
                cfg.postgres.local_conn_identity.user, cfg.postgres.roles.superuser.username
            ),
        });
    }
    if cfg.postgres.rewind_conn_identity.user != cfg.postgres.roles.rewinder.username {
        return Err(ConfigError::Validation {
            field: "postgres.rewind_conn_identity.user",
            message: format!(
                "must match postgres.roles.rewinder.username (got `{}`, expected `{}`)",
                cfg.postgres.rewind_conn_identity.user, cfg.postgres.roles.rewinder.username
            ),
        });
    }

    validate_postgres_auth_tls_invariants(cfg)?;

    validate_role_auth(
        "postgres.roles.superuser.auth.password.path",
        "postgres.roles.superuser.auth.password.content",
        &cfg.postgres.roles.superuser.auth,
    )?;
    validate_role_auth(
        "postgres.roles.replicator.auth.password.path",
        "postgres.roles.replicator.auth.password.content",
        &cfg.postgres.roles.replicator.auth,
    )?;
    validate_role_auth(
        "postgres.roles.rewinder.auth.password.path",
        "postgres.roles.rewinder.auth.password.content",
        &cfg.postgres.roles.rewinder.auth,
    )?;

    validate_tls_server_config(
        "postgres.tls.identity",
        "postgres.tls.identity.cert_chain",
        "postgres.tls.identity.private_key",
        &cfg.postgres.tls,
    )?;
    validate_tls_client_auth_config(
        "postgres.tls.client_auth",
        "postgres.tls.client_auth.client_ca",
        &cfg.postgres.tls,
    )?;

    validate_inline_or_path_non_empty(
        "postgres.pg_hba.source",
        &cfg.postgres.pg_hba.source,
        false,
    )?;
    validate_inline_or_path_non_empty(
        "postgres.pg_ident.source",
        &cfg.postgres.pg_ident.source,
        false,
    )?;
    for (key, value) in &cfg.postgres.extra_gucs {
        validate_extra_guc_for_config(key.as_str(), value.as_str())?;
    }

    validate_non_empty_path("process.binaries.postgres", &cfg.process.binaries.postgres)?;
    validate_absolute_path("process.binaries.postgres", &cfg.process.binaries.postgres)?;
    validate_non_empty_path("process.binaries.pg_ctl", &cfg.process.binaries.pg_ctl)?;
    validate_absolute_path("process.binaries.pg_ctl", &cfg.process.binaries.pg_ctl)?;
    validate_non_empty_path(
        "process.binaries.pg_rewind",
        &cfg.process.binaries.pg_rewind,
    )?;
    validate_absolute_path(
        "process.binaries.pg_rewind",
        &cfg.process.binaries.pg_rewind,
    )?;
    validate_non_empty_path("process.binaries.initdb", &cfg.process.binaries.initdb)?;
    validate_absolute_path("process.binaries.initdb", &cfg.process.binaries.initdb)?;
    validate_non_empty_path(
        "process.binaries.pg_basebackup",
        &cfg.process.binaries.pg_basebackup,
    )?;
    validate_absolute_path(
        "process.binaries.pg_basebackup",
        &cfg.process.binaries.pg_basebackup,
    )?;
    validate_non_empty_path("process.binaries.psql", &cfg.process.binaries.psql)?;
    validate_absolute_path("process.binaries.psql", &cfg.process.binaries.psql)?;

    validate_timeout(
        "process.pg_rewind_timeout_ms",
        cfg.process.pg_rewind_timeout_ms,
    )?;
    validate_timeout(
        "process.bootstrap_timeout_ms",
        cfg.process.bootstrap_timeout_ms,
    )?;
    validate_timeout("process.fencing_timeout_ms", cfg.process.fencing_timeout_ms)?;

    validate_timeout(
        "logging.postgres.poll_interval_ms",
        cfg.logging.postgres.poll_interval_ms,
    )?;
    if let Some(path) = cfg.logging.postgres.pg_ctl_log_file.as_ref() {
        validate_non_empty_path("logging.postgres.pg_ctl_log_file", path)?;
        validate_absolute_path("logging.postgres.pg_ctl_log_file", path)?;
    }
    if let Some(path) = cfg.logging.postgres.log_dir.as_ref() {
        validate_non_empty_path("logging.postgres.log_dir", path)?;
        validate_absolute_path("logging.postgres.log_dir", path)?;
    }
    if cfg.logging.postgres.cleanup.enabled {
        if cfg.logging.postgres.cleanup.max_files == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.max_files",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
        if cfg.logging.postgres.cleanup.max_age_seconds == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.max_age_seconds",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
        if cfg.logging.postgres.cleanup.protect_recent_seconds == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.protect_recent_seconds",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
    }

    if let Some(path) = cfg.logging.sinks.file.path.as_ref() {
        validate_non_empty_path("logging.sinks.file.path", path)?;
    }

    if cfg.logging.sinks.file.enabled && cfg.logging.sinks.file.path.is_none() {
        return Err(ConfigError::Validation {
            field: "logging.sinks.file.path",
            message: "must be configured when logging.sinks.file.enabled is true".to_string(),
        });
    }

    validate_non_empty_path("postgres.log_file", &cfg.postgres.log_file)?;
    validate_absolute_path("postgres.log_file", &cfg.postgres.log_file)?;

    if cfg.logging.sinks.file.enabled {
        if let Some(path) = cfg.logging.sinks.file.path.as_ref() {
            validate_absolute_path("logging.sinks.file.path", path)?;
        }
    }

    validate_logging_path_ownership_invariants(cfg)?;

    if cfg.dcs.endpoints.is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.endpoints",
            message: "must contain at least one endpoint".to_string(),
        });
    }

    for endpoint in &cfg.dcs.endpoints {
        if endpoint.trim().is_empty() {
            return Err(ConfigError::Validation {
                field: "dcs.endpoints",
                message: "must not contain empty endpoint values".to_string(),
            });
        }
    }

    if cfg.dcs.scope.trim().is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.scope",
            message: "must not be empty".to_string(),
        });
    }

    if cfg.ha.loop_interval_ms == 0 {
        return Err(ConfigError::Validation {
            field: "ha.loop_interval_ms",
            message: "must be greater than zero".to_string(),
        });
    }

    if cfg.ha.lease_ttl_ms == 0 {
        return Err(ConfigError::Validation {
            field: "ha.lease_ttl_ms",
            message: "must be greater than zero".to_string(),
        });
    }

    if cfg.ha.lease_ttl_ms <= cfg.ha.loop_interval_ms {
        return Err(ConfigError::Validation {
            field: "ha.lease_ttl_ms",
            message: "must be greater than ha.loop_interval_ms".to_string(),
        });
    }

    match &cfg.api.security.auth {
        crate::config::ApiAuthConfig::Disabled => {}
        crate::config::ApiAuthConfig::RoleTokens(tokens) => {
            validate_optional_non_empty(
                "api.security.auth.role_tokens.read_token",
                tokens.read_token.as_deref(),
            )?;
            validate_optional_non_empty(
                "api.security.auth.role_tokens.admin_token",
                tokens.admin_token.as_deref(),
            )?;
            if tokens.read_token.is_none() && tokens.admin_token.is_none() {
                return Err(ConfigError::Validation {
                    field: "api.security.auth.role_tokens",
                    message: "at least one of read_token or admin_token must be configured"
                        .to_string(),
                });
            }
        }
    }

    validate_tls_server_config(
        "api.security.tls.identity",
        "api.security.tls.identity.cert_chain",
        "api.security.tls.identity.private_key",
        &cfg.api.security.tls,
    )?;
    validate_tls_client_auth_config(
        "api.security.tls.client_auth",
        "api.security.tls.client_auth.client_ca",
        &cfg.api.security.tls,
    )?;

    validate_dcs_init_config(cfg)?;

    Ok(())
}

fn validate_extra_guc_for_config(key: &str, value: &str) -> Result<(), ConfigError> {
    validate_extra_guc_entry(key, value).map_err(|err| match err {
        ManagedPostgresConfError::InvalidExtraGuc { key, message } => ConfigError::Validation {
            field: "postgres.extra_gucs",
            message: format!("entry `{key}` invalid: {message}"),
        },
        ManagedPostgresConfError::ReservedExtraGuc { key } => ConfigError::Validation {
            field: "postgres.extra_gucs",
            message: format!("entry `{key}` is reserved by pgtuskmaster"),
        },
        ManagedPostgresConfError::InvalidPrimarySlotName { slot, message } => {
            ConfigError::Validation {
                field: "postgres.extra_gucs",
                message: format!(
                    "unexpected replica slot validation while checking extra gucs `{slot}`: {message}"
                ),
            }
        }
    })
}

fn validate_logging_path_ownership_invariants(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    let Some(sink_path) = cfg.logging.sinks.file.path.as_ref() else {
        return Ok(());
    };
    if !cfg.logging.sinks.file.enabled {
        return Ok(());
    }

    let effective_pg_ctl_log_file = match cfg.logging.postgres.pg_ctl_log_file.as_ref() {
        Some(path) => path,
        None => &cfg.postgres.log_file,
    };

    let sink_path = normalize_path_lexical(sink_path);
    let postgres_log_file = normalize_path_lexical(&cfg.postgres.log_file);
    let effective_pg_ctl_log_file = normalize_path_lexical(effective_pg_ctl_log_file);

    let tailed_files: [(&'static str, &PathBuf); 2] = [
        ("postgres.log_file", &postgres_log_file),
        (
            "logging.postgres.pg_ctl_log_file",
            &effective_pg_ctl_log_file,
        ),
    ];

    for (field, path) in tailed_files {
        if &sink_path == path {
            return Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                message: format!("must not equal tailed postgres input {field}"),
            });
        }
    }

    if let Some(log_dir) = cfg.logging.postgres.log_dir.as_ref() {
        let log_dir = normalize_path_lexical(log_dir);
        if sink_path.starts_with(&log_dir) {
            return Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                message: "must not be inside logging.postgres.log_dir (would self-ingest)"
                    .to_string(),
            });
        }
    }

    Ok(())
}

fn validate_non_empty_path(field: &'static str, path: &Path) -> Result<(), ConfigError> {
    if path.as_os_str().is_empty() {
        return Err(ConfigError::Validation {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_timeout(field: &'static str, value: u64) -> Result<(), ConfigError> {
    if !(MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&value) {
        return Err(ConfigError::Validation {
            field,
            message: format!("must be between {MIN_TIMEOUT_MS} and {MAX_TIMEOUT_MS} ms"),
        });
    }
    Ok(())
}

fn validate_port(field: &'static str, value: u16) -> Result<(), ConfigError> {
    if value == 0 {
        return Err(ConfigError::Validation {
            field,
            message: "must be greater than zero".to_string(),
        });
    }
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if value.trim().is_empty() {
        return Err(ConfigError::Validation {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_optional_non_empty(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), ConfigError> {
    if let Some(raw) = value {
        if raw.trim().is_empty() {
            return Err(ConfigError::Validation {
                field,
                message: "must not be empty when configured".to_string(),
            });
        }
    }
    Ok(())
}

fn validate_role_auth(
    password_path_field: &'static str,
    password_content_field: &'static str,
    auth: &RoleAuthConfig,
) -> Result<(), ConfigError> {
    match auth {
        RoleAuthConfig::Tls => Ok(()),
        RoleAuthConfig::Password { password } => {
            validate_secret_source_non_empty(password_path_field, password_content_field, password)
        }
    }
}

fn validate_postgres_auth_tls_invariants(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    validate_postgres_role_auth_supported(
        "postgres.roles.superuser.auth",
        &cfg.postgres.roles.superuser.auth,
    )?;
    validate_postgres_role_auth_supported(
        "postgres.roles.replicator.auth",
        &cfg.postgres.roles.replicator.auth,
    )?;
    validate_postgres_role_auth_supported(
        "postgres.roles.rewinder.auth",
        &cfg.postgres.roles.rewinder.auth,
    )?;

    validate_postgres_conn_identity_ssl_mode_supported(
        "postgres.local_conn_identity.ssl_mode",
        cfg.postgres.local_conn_identity.ssl_mode,
        cfg.postgres.tls.mode,
    )?;
    validate_postgres_conn_identity_ssl_mode_supported(
        "postgres.rewind_conn_identity.ssl_mode",
        cfg.postgres.rewind_conn_identity.ssl_mode,
        cfg.postgres.tls.mode,
    )?;

    Ok(())
}

fn validate_postgres_role_auth_supported(
    field: &'static str,
    auth: &RoleAuthConfig,
) -> Result<(), ConfigError> {
    match auth {
        RoleAuthConfig::Tls => Err(ConfigError::Validation {
            field,
            message:
                "postgresql role TLS client auth is not implemented; use type = \"password\" for now"
                    .to_string(),
        }),
        RoleAuthConfig::Password { .. } => Ok(()),
    }
}

fn validate_postgres_conn_identity_ssl_mode_supported(
    field: &'static str,
    ssl_mode: crate::pginfo::conninfo::PgSslMode,
    tls_mode: crate::config::ApiTlsMode,
) -> Result<(), ConfigError> {
    if matches!(tls_mode, crate::config::ApiTlsMode::Disabled)
        && postgres_ssl_mode_requires_server_tls(ssl_mode)
    {
        return Err(ConfigError::Validation {
            field,
            message: format!(
                "must not require server TLS when postgres.tls.mode is disabled (got `{}`)",
                ssl_mode.as_str()
            ),
        });
    }

    Ok(())
}

fn postgres_ssl_mode_requires_server_tls(ssl_mode: crate::pginfo::conninfo::PgSslMode) -> bool {
    matches!(
        ssl_mode,
        crate::pginfo::conninfo::PgSslMode::Require
            | crate::pginfo::conninfo::PgSslMode::VerifyCa
            | crate::pginfo::conninfo::PgSslMode::VerifyFull
    )
}

fn validate_tls_server_config(
    identity_field: &'static str,
    cert_chain_field: &'static str,
    private_key_field: &'static str,
    cfg: &TlsServerConfig,
) -> Result<(), ConfigError> {
    if matches!(cfg.mode, crate::config::ApiTlsMode::Disabled) {
        return Ok(());
    }

    let identity = cfg
        .identity
        .as_ref()
        .ok_or_else(|| ConfigError::Validation {
            field: identity_field,
            message: "tls identity must be configured when tls.mode is optional or required"
                .to_string(),
        })?;

    validate_inline_or_path_non_empty(cert_chain_field, &identity.cert_chain, false)?;
    validate_inline_or_path_non_empty(private_key_field, &identity.private_key, false)?;
    Ok(())
}

fn validate_tls_client_auth_config(
    client_auth_field: &'static str,
    client_ca_field: &'static str,
    cfg: &TlsServerConfig,
) -> Result<(), ConfigError> {
    let Some(client_auth) = cfg.client_auth.as_ref() else {
        return Ok(());
    };

    if matches!(cfg.mode, crate::config::ApiTlsMode::Disabled) {
        return Err(ConfigError::Validation {
            field: client_auth_field,
            message: "must not be configured when tls.mode is disabled".to_string(),
        });
    }

    validate_inline_or_path_non_empty(client_ca_field, &client_auth.client_ca, false)?;
    Ok(())
}

fn validate_dcs_init_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    let Some(init) = cfg.dcs.init.as_ref() else {
        return Ok(());
    };

    validate_non_empty("dcs.init.payload_json", init.payload_json.as_str())?;

    let _: serde_json::Value = serde_json::from_str(init.payload_json.as_str()).map_err(|err| {
        ConfigError::Validation {
            field: "dcs.init.payload_json",
            message: format!("must be valid JSON: {err}"),
        }
    })?;

    let _: RuntimeConfig = serde_json::from_str(init.payload_json.as_str()).map_err(|err| {
        ConfigError::Validation {
            field: "dcs.init.payload_json",
            message: format!("must decode as a RuntimeConfig JSON document: {err}"),
        }
    })?;

    Ok(())
}

fn validate_secret_source_non_empty(
    path_field: &'static str,
    content_field: &'static str,
    secret: &SecretSource,
) -> Result<(), ConfigError> {
    validate_inline_or_path_non_empty_for_secret(path_field, content_field, &secret.0)
}

fn validate_inline_or_path_non_empty_for_secret(
    path_field: &'static str,
    content_field: &'static str,
    value: &InlineOrPath,
) -> Result<(), ConfigError> {
    match value {
        InlineOrPath::Path(path) => validate_non_empty_path(path_field, path),
        InlineOrPath::PathConfig { path } => validate_non_empty_path(path_field, path),
        InlineOrPath::Inline { content } => validate_non_empty(content_field, content.as_str()),
    }
}

fn validate_inline_or_path_non_empty(
    field: &'static str,
    value: &InlineOrPath,
    allow_empty_inline: bool,
) -> Result<(), ConfigError> {
    match value {
        InlineOrPath::Path(path) => validate_non_empty_path(field, path),
        InlineOrPath::PathConfig { path } => validate_non_empty_path(field, path),
        InlineOrPath::Inline { content } => {
            if allow_empty_inline {
                Ok(())
            } else {
                validate_non_empty(field, content.as_str())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::schema::{
        ApiAuthConfig, ApiConfig, ApiRoleTokensConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths,
        ClusterConfig, DcsConfig, DebugConfig, FileSinkConfig, FileSinkMode, HaConfig,
        InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig, PgHbaConfig,
        PgIdentConfig, PostgresConfig, PostgresConnIdentityConfig, PostgresLoggingConfig,
        PostgresRoleConfig, PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig,
        StderrSinkConfig, TlsServerConfig,
    };
    use crate::pginfo::conninfo::PgSslMode;

    fn sample_password_auth() -> RoleAuthConfig {
        RoleAuthConfig::Password {
            password: crate::config::SecretSource(crate::config::InlineOrPath::Inline {
                content: "secret-password".to_string(),
            }),
        }
    }

    fn expect_validation_error(
        result: Result<(), ConfigError>,
        expected_field: &'static str,
        expected_message_fragment: &str,
    ) -> Result<(), String> {
        match result {
            Err(ConfigError::Validation { field, message }) => {
                if field != expected_field {
                    return Err(format!(
                        "expected validation field {expected_field}, got {field}"
                    ));
                }
                if !message.contains(expected_message_fragment) {
                    return Err(format!(
                        "expected validation message to contain {expected_message_fragment:?}, got {message:?}"
                    ));
                }
                Ok(())
            }
            other => Err(format!(
                "expected validation error for {expected_field}, got {other:?}"
            )),
        }
    }

    fn base_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "member-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: PathBuf::from("/var/lib/postgresql/data"),
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: PathBuf::from("/tmp/pgtuskmaster/socket"),
                log_file: PathBuf::from("/tmp/pgtuskmaster/postgres.log"),
                local_conn_identity: PostgresConnIdentityConfig {
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                rewind_conn_identity: PostgresConnIdentityConfig {
                    user: "rewinder".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                tls: TlsServerConfig {
                    mode: ApiTlsMode::Disabled,
                    identity: None,
                    client_auth: None,
                },
                roles: PostgresRolesConfig {
                    superuser: PostgresRoleConfig {
                        username: "postgres".to_string(),
                        auth: sample_password_auth(),
                    },
                    replicator: PostgresRoleConfig {
                        username: "replicator".to_string(),
                        auth: sample_password_auth(),
                    },
                    rewinder: PostgresRoleConfig {
                        username: "rewinder".to_string(),
                        auth: sample_password_auth(),
                    },
                },
                pg_hba: PgHbaConfig {
                    source: InlineOrPath::Inline {
                        content: "local all all trust\n".to_string(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: "# empty\n".to_string(),
                    },
                },
                extra_gucs: std::collections::BTreeMap::new(),
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
                init: None,
            },
            ha: HaConfig {
                loop_interval_ms: 1_000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 120_000,
                bootstrap_timeout_ms: 300_000,
                fencing_timeout_ms: 30_000,
                binaries: BinaryPaths {
                    postgres: PathBuf::from("/usr/bin/postgres"),
                    pg_ctl: PathBuf::from("/usr/bin/pg_ctl"),
                    pg_rewind: PathBuf::from("/usr/bin/pg_rewind"),
                    initdb: PathBuf::from("/usr/bin/initdb"),
                    pg_basebackup: PathBuf::from("/usr/bin/pg_basebackup"),
                    psql: PathBuf::from("/usr/bin/psql"),
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
                        protect_recent_seconds: 300,
                    },
                },
                sinks: LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: FileSinkMode::Append,
                    },
                },
            },
            api: ApiConfig {
                listen_addr: "127.0.0.1:8080".to_string(),
                security: ApiSecurityConfig {
                    tls: TlsServerConfig {
                        mode: ApiTlsMode::Disabled,
                        identity: None,
                        client_auth: None,
                    },
                    auth: ApiAuthConfig::Disabled,
                },
            },
            debug: DebugConfig { enabled: false },
        }
    }

    #[test]
    fn validate_runtime_config_accepts_valid_config() {
        let cfg = base_runtime_config();
        assert!(validate_runtime_config(&cfg).is_ok());
    }

    #[test]
    fn validate_runtime_config_rejects_postgres_role_tls_auth() -> Result<(), String> {
        let mut superuser_cfg = base_runtime_config();
        superuser_cfg.postgres.roles.superuser.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&superuser_cfg),
            "postgres.roles.superuser.auth",
            "type = \"password\"",
        )?;

        let mut replicator_cfg = base_runtime_config();
        replicator_cfg.postgres.roles.replicator.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&replicator_cfg),
            "postgres.roles.replicator.auth",
            "type = \"password\"",
        )?;

        let mut rewinder_cfg = base_runtime_config();
        rewinder_cfg.postgres.roles.rewinder.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&rewinder_cfg),
            "postgres.roles.rewinder.auth",
            "type = \"password\"",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_local_conn_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), String> {
        let mut cfg = base_runtime_config();
        cfg.postgres.local_conn_identity.ssl_mode = PgSslMode::Require;

        expect_validation_error(
            validate_runtime_config(&cfg),
            "postgres.local_conn_identity.ssl_mode",
            "postgres.tls.mode is disabled",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_rewind_conn_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), String> {
        let mut cfg = base_runtime_config();
        cfg.postgres.rewind_conn_identity.ssl_mode = PgSslMode::VerifyFull;

        expect_validation_error(
            validate_runtime_config(&cfg),
            "postgres.rewind_conn_identity.ssl_mode",
            "postgres.tls.mode is disabled",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_empty_binary_path() {
        let mut cfg = base_runtime_config();
        cfg.process.binaries.pg_ctl = PathBuf::new();

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries.pg_ctl",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_non_absolute_binary_paths() {
        let mut cfg = base_runtime_config();
        cfg.process.binaries.pg_ctl = PathBuf::from("pg_ctl");
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries.pg_ctl",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_bad_timeout() {
        let mut cfg = base_runtime_config();
        cfg.process.bootstrap_timeout_ms = 0;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.bootstrap_timeout_ms",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_invalid_postgres_runtime_fields() {
        let mut cfg = base_runtime_config();
        cfg.postgres.listen_host = " ".to_string();
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.listen_host",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.postgres.listen_port = 0;
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.listen_port",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_missing_dcs_and_ha_invariants() {
        let mut cfg = base_runtime_config();
        cfg.dcs.endpoints.clear();

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "dcs.endpoints",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.ha.lease_ttl_ms = cfg.ha.loop_interval_ms;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "ha.lease_ttl_ms",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_blank_api_tokens() {
        let mut cfg = base_runtime_config();
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: Some(" ".to_string()),
            admin_token: None,
        });

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "api.security.auth.role_tokens.read_token",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: None,
            admin_token: Some("\t".to_string()),
        });

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "api.security.auth.role_tokens.admin_token",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_enabled_without_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = None;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_empty_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::new());

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_accepts_file_sink_enabled_with_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster.jsonl"));

        assert!(validate_runtime_config(&cfg).is_ok());
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_equal_to_tailed_log_via_dot_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/./postgres.log"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_equal_to_tailed_log_via_parent_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/tmp/../postgres.log"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_inside_log_dir_via_dot_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.postgres.log_dir = Some(PathBuf::from("/tmp/pgtuskmaster/log_dir"));
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/log_dir/./out.jsonl"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn load_runtime_config_missing_config_version_is_rejected(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-{unique}.toml"));

        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "config_version",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_config_version_v1_is_rejected() -> Result<(), Box<dyn std::error::Error>>
    {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-invalid-{unique}.toml"));

        let toml = r#"
config_version = "v1"
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "config_version",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_unknown_fields_in_v2() -> Result<(), Box<dyn std::error::Error>>
    {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-invalid-{unique}.toml"));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
connect_timeout_s = 5
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }
unknown = 10

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[logging]
level = "info"
capture_subprocess_output = true
postgres = { enabled = true, poll_interval_ms = 200, cleanup = { enabled = true, max_files = 10, max_age_seconds = 60 } }
sinks = { stderr = { enabled = true }, file = { enabled = false, mode = "append" } }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(err, Err(ConfigError::Parse { .. })));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_happy_path_with_safe_defaults(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-v2-{unique}.toml"));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;
        let cfg = load_runtime_config(&path)?;
        assert_eq!(cfg.postgres.connect_timeout_s, 5);
        assert_eq!(cfg.process.pg_rewind_timeout_ms, 120_000);
        assert_eq!(cfg.process.bootstrap_timeout_ms, 300_000);
        assert_eq!(cfg.process.fencing_timeout_ms, 30_000);
        assert_eq!(cfg.api.listen_addr, "127.0.0.1:8080");
        assert!(!cfg.debug.enabled);

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_secure_fields_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-v2-missing-{unique}.toml"));

        // Intentionally omit `postgres.local_conn_identity`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.local_conn_identity",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_process_binaries_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("runtime-config-v2-missing-binaries-{unique}.toml"));

        // Intentionally omit `process.binaries`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_password_auth_missing_password_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-auth-password-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.superuser.auth.password`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password" } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.superuser.auth.password",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_postgres_roles_block_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("runtime-config-v2-missing-roles-{unique}.toml"));

        // Intentionally omit `postgres.roles`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_replicator_role_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-replicator-role-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_replicator_username_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-replicator-username-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator.username`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator.username",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_replicator_auth_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-missing-replicator-auth-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator.auth`.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator" }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator.auth",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_conn_identity_role_mismatch(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-conn-identity-mismatch-{unique}.toml"
        ));

        // Intentionally set local_conn_identity.user to a different user than roles.superuser.username.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "not-postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.local_conn_identity.user",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_blank_password_secret(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-blank-password-secret-{unique}.toml"
        ));

        // Intentionally set password secret content to empty.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.superuser.auth.password.content",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_tls_required_without_identity(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-required-tls-no-identity-{unique}.toml"
        ));

        // Intentionally omit `postgres.tls.identity` while requiring TLS.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "required" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.tls.identity",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_client_auth_with_tls_disabled(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-client-auth-with-tls-disabled-{unique}.toml"
        ));

        // Intentionally configure client auth while TLS is disabled.
        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled", client_auth = { client_ca = { content = "client-ca" }, require_client_cert = false } }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.tls.client_auth",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_postgres_role_tls_auth_with_actionable_error(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-postgres-role-tls-auth-{unique}.toml"
        ));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "tls" } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        let mapped = match err {
            Err(ConfigError::Validation { field, message }) => {
                if field != "postgres.roles.superuser.auth" {
                    Err(format!(
                        "expected validation field postgres.roles.superuser.auth, got {field}"
                    ))
                } else if !message.contains("type = \"password\"") {
                    Err(format!(
                        "expected validation message to mention password auth, got {message:?}"
                    ))
                } else {
                    Ok(())
                }
            }
            other => Err(format!("expected validation error, got {other:?}")),
        };
        mapped.map_err(std::io::Error::other)?;

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_rejects_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-v2-postgres-ssl-mode-requires-tls-{unique}.toml"
        ));

        let toml = r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "verify-full" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        let mapped = match err {
            Err(ConfigError::Validation { field, message }) => {
                if field != "postgres.local_conn_identity.ssl_mode" {
                    Err(format!(
                        "expected validation field postgres.local_conn_identity.ssl_mode, got {field}"
                    ))
                } else if !message.contains("postgres.tls.mode is disabled") {
                    Err(format!(
                        "expected validation message to mention disabled postgres TLS, got {message:?}"
                    ))
                } else {
                    Ok(())
                }
            }
            other => Err(format!("expected validation error, got {other:?}")),
        };
        mapped.map_err(std::io::Error::other)?;

        std::fs::remove_file(&path)?;
        Ok(())
    }
}

--- END FILE: src/config/parser.rs ---

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

--- BEGIN FILE: src/api/worker.rs ---
use std::{sync::Arc, time::Duration};

use rustls::ServerConfig;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{server::TlsStream, TlsAcceptor};

use crate::{
    api::{
        controller::{delete_switchover, get_ha_state, post_switchover, SwitchoverRequestInput},
        fallback::{get_fallback_cluster, post_fallback_heartbeat, FallbackHeartbeatInput},
        ApiError,
    },
    config::{ApiAuthConfig, ApiTlsMode, RuntimeConfig},
    dcs::store::DcsStore,
    debug_api::{snapshot::SystemSnapshot, view::build_verbose_payload},
    logging::{AppEvent, AppEventHeader, LogHandle, SeverityText, StructuredFields},
    state::{StateSubscriber, WorkerError},
};

const API_LOOP_POLL_INTERVAL: Duration = Duration::from_millis(10);
const API_ACCEPT_TIMEOUT: Duration = Duration::from_millis(1);
const API_REQUEST_READ_TIMEOUT: Duration = Duration::from_millis(100);
const API_TLS_CLIENT_HELLO_PEEK_TIMEOUT: Duration = Duration::from_millis(10);
const API_REQUEST_ID_MAX_LEN: usize = 128;
const HTTP_REQUEST_MAX_BYTES: usize = 1024 * 1024;
const HTTP_REQUEST_HEADER_LIMIT_BYTES: usize = 16 * 1024;
const HTTP_REQUEST_SCRATCH_BUFFER_BYTES: usize = 4096;
const HTTP_REQUEST_HEADER_CAPACITY: usize = 64;

#[derive(Clone, Debug, Default)]
struct ApiRoleTokens {
    read_token: Option<String>,
    admin_token: Option<String>,
}

#[derive(Clone, Copy, Debug)]
enum ApiEventKind {
    StepOnceFailed,
    ConnectionAccepted,
    RequestParseFailed,
    ResponseSent,
    AuthDecision,
    TlsClientCertMissing,
    TlsHandshakeFailed,
}

impl ApiEventKind {
    fn name(self) -> &'static str {
        match self {
            Self::StepOnceFailed => "api.step_once_failed",
            Self::ConnectionAccepted => "api.connection_accepted",
            Self::RequestParseFailed => "api.request_parse_failed",
            Self::ResponseSent => "api.response_sent",
            Self::AuthDecision => "api.auth_decision",
            Self::TlsClientCertMissing => "api.tls_client_cert_missing",
            Self::TlsHandshakeFailed => "api.tls_handshake_failed",
        }
    }
}

fn api_event(
    kind: ApiEventKind,
    result: &str,
    severity: SeverityText,
    message: impl Into<String>,
) -> AppEvent {
    AppEvent::new(
        severity,
        message,
        AppEventHeader::new(kind.name(), "api", result),
    )
}

pub struct ApiWorkerCtx {
    listener: TcpListener,
    poll_interval: Duration,
    scope: String,
    member_id: String,
    config_subscriber: StateSubscriber<RuntimeConfig>,
    dcs_store: Box<dyn DcsStore>,
    debug_snapshot_subscriber: Option<StateSubscriber<SystemSnapshot>>,
    tls_mode_override: Option<ApiTlsMode>,
    tls_acceptor: Option<TlsAcceptor>,
    role_tokens: Option<ApiRoleTokens>,
    require_client_cert: bool,
    log: LogHandle,
}

impl ApiWorkerCtx {
    pub fn contract_stub(
        listener: TcpListener,
        config_subscriber: StateSubscriber<RuntimeConfig>,
        dcs_store: Box<dyn DcsStore>,
    ) -> Self {
        Self::new(
            listener,
            config_subscriber,
            dcs_store,
            LogHandle::disabled(),
        )
    }

    pub(crate) fn new(
        listener: TcpListener,
        config_subscriber: StateSubscriber<RuntimeConfig>,
        dcs_store: Box<dyn DcsStore>,
        log: LogHandle,
    ) -> Self {
        let scope = config_subscriber.latest().value.dcs.scope.clone();
        let member_id = config_subscriber.latest().value.cluster.member_id.clone();
        Self {
            listener,
            poll_interval: API_LOOP_POLL_INTERVAL,
            scope,
            member_id,
            config_subscriber,
            dcs_store,
            debug_snapshot_subscriber: None,
            tls_mode_override: None,
            tls_acceptor: None,
            role_tokens: None,
            require_client_cert: false,
            log,
        }
    }

    pub fn local_addr(&self) -> Result<std::net::SocketAddr, WorkerError> {
        self.listener
            .local_addr()
            .map_err(|err| WorkerError::Message(format!("api local_addr failed: {err}")))
    }

    pub fn configure_tls(
        &mut self,
        mode: ApiTlsMode,
        server_config: Option<Arc<ServerConfig>>,
    ) -> Result<(), WorkerError> {
        match mode {
            ApiTlsMode::Disabled => {
                self.tls_mode_override = Some(ApiTlsMode::Disabled);
                self.tls_acceptor = None;
                Ok(())
            }
            ApiTlsMode::Optional | ApiTlsMode::Required => {
                let cfg = server_config.ok_or_else(|| {
                    WorkerError::Message(
                        "tls mode optional/required requires a server tls config".to_string(),
                    )
                })?;
                self.tls_mode_override = Some(mode);
                self.tls_acceptor = Some(TlsAcceptor::from(cfg));
                Ok(())
            }
        }
    }

    pub fn configure_role_tokens(
        &mut self,
        read_token: Option<String>,
        admin_token: Option<String>,
    ) -> Result<(), WorkerError> {
        let read = normalize_optional_token(read_token)?;
        let admin = normalize_optional_token(admin_token)?;

        if read.is_none() && admin.is_none() {
            self.role_tokens = None;
            return Ok(());
        }

        self.role_tokens = Some(ApiRoleTokens {
            read_token: read,
            admin_token: admin,
        });
        Ok(())
    }

    pub fn set_require_client_cert(&mut self, required: bool) {
        self.require_client_cert = required;
    }

    pub(crate) fn set_ha_snapshot_subscriber(
        &mut self,
        subscriber: StateSubscriber<SystemSnapshot>,
    ) {
        self.debug_snapshot_subscriber = Some(subscriber);
    }
}

pub async fn run(mut ctx: ApiWorkerCtx) -> Result<(), WorkerError> {
    loop {
        if let Err(err) = step_once(&mut ctx).await {
            let fatal = is_fatal_api_step_error(&err);
            let mut event = api_event(
                ApiEventKind::StepOnceFailed,
                "failed",
                if fatal {
                    SeverityText::Error
                } else {
                    SeverityText::Warn
                },
                "api step failed",
            );
            let fields = event.fields_mut();
            fields.append_json_map(api_base_fields(&ctx).into_attributes());
            fields.insert("error", err.to_string());
            fields.insert("fatal", fatal);
            ctx.log
                .emit_app_event("api_worker::run", event)
                .map_err(|emit_err| {
                    WorkerError::Message(format!("api step failure log emit failed: {emit_err}"))
                })?;

            if fatal {
                return Err(err);
            }
        }
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub async fn step_once(ctx: &mut ApiWorkerCtx) -> Result<(), WorkerError> {
    let (stream, peer) = match tokio::time::timeout(API_ACCEPT_TIMEOUT, ctx.listener.accept()).await
    {
        Ok(Ok((stream, peer))) => (stream, peer),
        Ok(Err(err)) => {
            return Err(WorkerError::Message(format!("api accept failed: {err}")));
        }
        Err(_elapsed) => return Ok(()),
    };

    let cfg = ctx.config_subscriber.latest().value;
    let mut accept_event = api_event(
        ApiEventKind::ConnectionAccepted,
        "ok",
        SeverityText::Debug,
        "api connection accepted",
    );
    let fields = accept_event.fields_mut();
    fields.append_json_map(api_base_fields(ctx).into_attributes());
    fields.insert("api.peer_addr", peer.to_string());
    fields.insert(
        "api.tls_mode",
        format!("{:?}", effective_tls_mode(ctx, &cfg)).to_lowercase(),
    );
    ctx.log
        .emit_app_event("api_worker::step_once", accept_event)
        .map_err(|err| WorkerError::Message(format!("api accept log emit failed: {err}")))?;

    let mut stream = match accept_connection(ctx, &cfg, peer, stream).await? {
        Some(stream) => stream,
        None => return Ok(()),
    };

    let request =
        match tokio::time::timeout(API_REQUEST_READ_TIMEOUT, stream.read_http_request()).await {
            Ok(Ok(req)) => req,
            Ok(Err(message)) => {
                let mut event = api_event(
                    ApiEventKind::RequestParseFailed,
                    "failed",
                    SeverityText::Warn,
                    "api request parse failed",
                );
                let fields = event.fields_mut();
                fields.append_json_map(api_base_fields(ctx).into_attributes());
                fields.insert("api.peer_addr", peer.to_string());
                fields.insert("error", message.clone());
                ctx.log
                    .emit_app_event("api_worker::step_once", event)
                    .map_err(|err| {
                        WorkerError::Message(format!("api parse failure log emit failed: {err}"))
                    })?;
                let response = HttpResponse::text(400, "Bad Request", message);
                stream.write_http_response(response).await?;
                return Ok(());
            }
            Err(_elapsed) => return Ok(()),
        };

    match authorize_request(ctx, &cfg, &request) {
        AuthDecision::Allowed => {}
        AuthDecision::Unauthorized => {
            emit_api_auth_decision(ctx, peer, &request, "unauthorized")?;
            let response = HttpResponse::text(401, "Unauthorized", "unauthorized");
            stream.write_http_response(response).await?;
            return Ok(());
        }
        AuthDecision::Forbidden => {
            emit_api_auth_decision(ctx, peer, &request, "forbidden")?;
            let response = HttpResponse::text(403, "Forbidden", "forbidden");
            stream.write_http_response(response).await?;
            return Ok(());
        }
    }

    emit_api_auth_decision(ctx, peer, &request, "allowed")?;

    let response = route_request(ctx, &cfg, peer, request);
    let status_code = response.status;
    stream.write_http_response(response).await?;

    let mut event = api_event(
        ApiEventKind::ResponseSent,
        "ok",
        SeverityText::Debug,
        "api response sent",
    );
    let fields = event.fields_mut();
    fields.append_json_map(api_base_fields(ctx).into_attributes());
    fields.insert("api.peer_addr", peer.to_string());
    fields.insert("api.status_code", u64::from(status_code));
    ctx.log
        .emit_app_event("api_worker::step_once", event)
        .map_err(|err| WorkerError::Message(format!("api response log emit failed: {err}")))?;
    Ok(())
}

fn api_base_fields(ctx: &ApiWorkerCtx) -> StructuredFields {
    let mut fields = StructuredFields::new();
    fields.insert("scope", ctx.scope.clone());
    fields.insert("member_id", ctx.member_id.clone());
    fields
}

fn extract_request_id(request: &HttpRequest) -> Option<String> {
    request
        .headers
        .iter()
        .find(|(name, _value)| name.eq_ignore_ascii_case("x-request-id"))
        .map(|(_name, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(|value| {
            if value.len() > API_REQUEST_ID_MAX_LEN {
                value[..API_REQUEST_ID_MAX_LEN].to_string()
            } else {
                value
            }
        })
}

fn auth_header_present(request: &HttpRequest) -> bool {
    request
        .headers
        .iter()
        .any(|(name, _value)| name.eq_ignore_ascii_case("authorization"))
}

fn route_template(request: &HttpRequest) -> String {
    let (path, _query) = split_path_and_query(&request.path);
    format!("{} {}", request.method, path)
}

fn emit_api_auth_decision(
    ctx: &ApiWorkerCtx,
    peer: std::net::SocketAddr,
    request: &HttpRequest,
    decision: &str,
) -> Result<(), WorkerError> {
    let mut event = api_event(
        ApiEventKind::AuthDecision,
        "ok",
        SeverityText::Debug,
        "api auth decision",
    );
    let fields = event.fields_mut();
    fields.append_json_map(api_base_fields(ctx).into_attributes());
    fields.insert("api.peer_addr", peer.to_string());
    fields.insert("api.method", request.method.clone());
    fields.insert("api.route_template", route_template(request));
    fields.insert("api.auth.header_present", auth_header_present(request));
    fields.insert("api.auth.result", decision.to_string());
    fields.insert(
        "api.auth.required_role",
        format!("{:?}", endpoint_role(request)).to_lowercase(),
    );
    if let Some(request_id) = extract_request_id(request) {
        fields.insert("api.request_id", request_id);
    }
    ctx.log
        .emit_app_event("api_worker::authorize_request", event)
        .map_err(|err| WorkerError::Message(format!("api auth log emit failed: {err}")))?;
    Ok(())
}

fn is_fatal_api_step_error(err: &WorkerError) -> bool {
    let message = err.to_string();
    message.contains("api accept failed")
        || message.contains("tls mode requires a configured tls acceptor")
        || message.contains("api local_addr failed")
}

fn route_request(
    ctx: &mut ApiWorkerCtx,
    cfg: &RuntimeConfig,
    _peer: std::net::SocketAddr,
    request: HttpRequest,
) -> HttpResponse {
    let (path, query) = split_path_and_query(&request.path);
    match (request.method.as_str(), path) {
        ("POST", "/switchover") => {
            let input = match serde_json::from_slice::<SwitchoverRequestInput>(&request.body) {
                Ok(parsed) => parsed,
                Err(err) => {
                    return HttpResponse::text(400, "Bad Request", format!("invalid json: {err}"));
                }
            };
            match post_switchover(&ctx.scope, &mut *ctx.dcs_store, input) {
                Ok(value) => HttpResponse::json(202, "Accepted", &value),
                Err(err) => api_error_to_http(err),
            }
        }
        ("DELETE", "/ha/switchover") => match delete_switchover(&ctx.scope, &mut *ctx.dcs_store) {
            Ok(value) => HttpResponse::json(202, "Accepted", &value),
            Err(err) => api_error_to_http(err),
        },
        ("GET", "/ha/state") => {
            let Some(subscriber) = ctx.debug_snapshot_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "snapshot unavailable");
            };
            let snapshot = subscriber.latest();
            let response = get_ha_state(&snapshot);
            HttpResponse::json(200, "OK", &response)
        }
        ("GET", "/fallback/cluster") => {
            let view = get_fallback_cluster(cfg);
            HttpResponse::json(200, "OK", &view)
        }
        ("POST", "/fallback/heartbeat") => {
            let input = match serde_json::from_slice::<FallbackHeartbeatInput>(&request.body) {
                Ok(parsed) => parsed,
                Err(err) => {
                    return HttpResponse::text(400, "Bad Request", format!("invalid json: {err}"));
                }
            };
            match post_fallback_heartbeat(input) {
                Ok(value) => HttpResponse::json(202, "Accepted", &value),
                Err(err) => api_error_to_http(err),
            }
        }
        ("GET", "/debug/snapshot") => {
            if !cfg.debug.enabled {
                return HttpResponse::text(404, "Not Found", "not found");
            }
            let Some(subscriber) = ctx.debug_snapshot_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "snapshot unavailable");
            };
            let snapshot = subscriber.latest();
            HttpResponse::text(200, "OK", format!("{:#?}", snapshot))
        }
        ("GET", "/debug/verbose") => {
            if !cfg.debug.enabled {
                return HttpResponse::text(404, "Not Found", "not found");
            }
            let Some(subscriber) = ctx.debug_snapshot_subscriber.as_ref() else {
                return HttpResponse::text(503, "Service Unavailable", "snapshot unavailable");
            };
            let since_sequence = match parse_since_sequence(query) {
                Ok(value) => value,
                Err(message) => return HttpResponse::text(400, "Bad Request", message),
            };
            let snapshot = subscriber.latest();
            let payload = build_verbose_payload(&snapshot, since_sequence);
            HttpResponse::json(200, "OK", &payload)
        }
        ("GET", "/debug/ui") => {
            if !cfg.debug.enabled {
                return HttpResponse::text(404, "Not Found", "not found");
            }
            HttpResponse::html(200, "OK", debug_ui_html())
        }
        _ => HttpResponse::text(404, "Not Found", "not found"),
    }
}

fn api_error_to_http(err: ApiError) -> HttpResponse {
    match err {
        ApiError::BadRequest(message) => HttpResponse::text(400, "Bad Request", message),
        ApiError::DcsStore(message) => HttpResponse::text(503, "Service Unavailable", message),
        ApiError::Internal(message) => HttpResponse::text(500, "Internal Server Error", message),
    }
}

fn split_path_and_query(path: &str) -> (&str, Option<&str>) {
    match path.split_once('?') {
        Some((head, tail)) => (head, Some(tail)),
        None => (path, None),
    }
}

fn parse_since_sequence(query: Option<&str>) -> Result<Option<u64>, String> {
    let Some(query) = query else {
        return Ok(None);
    };

    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if key == "since" {
            let parsed = value
                .parse::<u64>()
                .map_err(|err| format!("invalid since query parameter: {err}"))?;
            return Ok(Some(parsed));
        }
    }
    Ok(None)
}

fn debug_ui_html() -> &'static str {
    r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>PGTuskMaster Debug UI</title>
  <style>
    :root {
      --bg: radial-gradient(circle at 10% 10%, #162132, #081019 55%, #06090f 100%);
      --panel: rgba(16, 26, 40, 0.92);
      --line: rgba(139, 190, 255, 0.22);
      --text: #d8e6ff;
      --muted: #89a3c4;
      --ok: #4bd18b;
      --warn: #f0bc5e;
      --err: #ff7070;
      --accent: #5ec3ff;
      --font: "JetBrains Mono", "Fira Mono", Menlo, monospace;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: var(--font);
      background: var(--bg);
      color: var(--text);
      min-height: 100vh;
      padding: 14px;
    }
    .layout {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
      gap: 12px;
      max-width: 1300px;
      margin: 0 auto;
    }
    .panel {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 12px;
      padding: 12px;
      box-shadow: inset 0 1px 0 rgba(255,255,255,0.04);
    }
    .panel h2 {
      margin: 0 0 10px 0;
      font-size: 14px;
      letter-spacing: 0.04em;
      color: var(--accent);
      text-transform: uppercase;
    }
    .metrics { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
    .metric {
      border: 1px solid var(--line);
      border-radius: 9px;
      padding: 8px;
      background: rgba(0,0,0,0.2);
    }
    .metric .label { font-size: 11px; color: var(--muted); text-transform: uppercase; }
    .metric .value { margin-top: 6px; font-size: 16px; font-weight: 700; }
    .badge {
      display: inline-flex;
      align-items: center;
      padding: 2px 8px;
      border-radius: 999px;
      font-size: 11px;
      border: 1px solid var(--line);
      margin-left: 8px;
    }
    .badge.ok { color: var(--ok); border-color: color-mix(in oklab, var(--ok), black 40%); }
    .badge.warn { color: var(--warn); border-color: color-mix(in oklab, var(--warn), black 40%); }
    .badge.err { color: var(--err); border-color: color-mix(in oklab, var(--err), black 40%); }
    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 12px;
    }
    th, td {
      text-align: left;
      padding: 6px;
      border-bottom: 1px solid rgba(255,255,255,0.08);
      vertical-align: top;
      word-break: break-word;
    }
    th { color: var(--muted); }
    .timeline { max-height: 260px; overflow: auto; }
    .full { grid-column: 1 / -1; }
    @media (max-width: 760px) {
      body { padding: 8px; }
      .metrics { grid-template-columns: 1fr; }
    }
  </style>
</head>
<body>
  <div class="layout">
    <section class="panel full" id="meta-panel">
      <h2>Runtime Meta <span id="meta-badge" class="badge warn">loading</span></h2>
      <div class="metrics">
        <div class="metric"><div class="label">Lifecycle</div><div class="value" id="m-lifecycle">-</div></div>
        <div class="metric"><div class="label">Sequence</div><div class="value" id="m-seq">-</div></div>
        <div class="metric"><div class="label">Generated (ms)</div><div class="value" id="m-ts">-</div></div>
      </div>
    </section>
    <section class="panel" id="config-panel"><h2>Config</h2><div id="config-body">-</div></section>
    <section class="panel" id="pginfo-panel"><h2>PgInfo</h2><div id="pginfo-body">-</div></section>
    <section class="panel" id="dcs-panel"><h2>DCS</h2><div id="dcs-body">-</div></section>
    <section class="panel" id="process-panel"><h2>Process</h2><div id="process-body">-</div></section>
    <section class="panel" id="ha-panel"><h2>HA</h2><div id="ha-body">-</div></section>
    <section class="panel full timeline" id="timeline-panel">
      <h2>Timeline</h2>
      <table>
        <thead><tr><th>Seq</th><th>At</th><th>Category</th><th>Message</th></tr></thead>
        <tbody id="timeline-body"></tbody>
      </table>
    </section>
    <section class="panel full timeline" id="changes-panel">
      <h2>Changes</h2>
      <table>
        <thead><tr><th>Seq</th><th>At</th><th>Domain</th><th>Versions</th><th>Summary</th></tr></thead>
        <tbody id="changes-body"></tbody>
      </table>
    </section>
  </div>
  <script>
    const state = { since: 0 };
    const byId = (id) => document.getElementById(id);
    const asText = (value) => (value === null || value === undefined ? "-" : String(value));
    const badge = (label, cls) => {
      const el = byId("meta-badge");
      el.textContent = label;
      el.className = `badge ${cls}`;
    };
    function renderKeyValue(id, entries) {
      byId(id).innerHTML = entries
        .map(([k, v]) => `<div><strong>${k}</strong>: ${asText(v)}</div>`)
        .join("");
    }
    function renderRows(id, rows, mapRow) {
      byId(id).innerHTML = rows.map(mapRow).join("");
    }
    function render(payload) {
      byId("m-lifecycle").textContent = asText(payload.meta.app_lifecycle);
      byId("m-seq").textContent = asText(payload.meta.sequence);
      byId("m-ts").textContent = asText(payload.meta.generated_at_ms);
      badge("connected", "ok");

      renderKeyValue("config-body", [
        ["member", payload.config.member_id],
        ["cluster", payload.config.cluster_name],
        ["scope", payload.config.scope],
        ["version", payload.config.version],
        ["debug", payload.config.debug_enabled],
        ["tls", payload.config.tls_enabled]
      ]);
      renderKeyValue("pginfo-body", [
        ["variant", payload.pginfo.variant],
        ["worker", payload.pginfo.worker],
        ["sql", payload.pginfo.sql],
        ["readiness", payload.pginfo.readiness],
        ["summary", payload.pginfo.summary]
      ]);
      renderKeyValue("dcs-body", [
        ["worker", payload.dcs.worker],
        ["trust", payload.dcs.trust],
        ["members", payload.dcs.member_count],
        ["leader", payload.dcs.leader],
        ["switchover", payload.dcs.has_switchover_request]
      ]);
      renderKeyValue("process-body", [
        ["worker", payload.process.worker],
        ["state", payload.process.state],
        ["running_job", payload.process.running_job_id],
        ["last_outcome", payload.process.last_outcome]
      ]);
      renderKeyValue("ha-body", [
        ["worker", payload.ha.worker],
        ["phase", payload.ha.phase],
        ["tick", payload.ha.tick],
        ["decision", payload.ha.decision],
        ["decision_detail", payload.ha.decision_detail ?? "<none>"],
        ["planned_actions", payload.ha.planned_actions]
      ]);

      renderRows("timeline-body", payload.timeline, (row) =>
        `<tr><td>${row.sequence}</td><td>${row.at_ms}</td><td>${row.category}</td><td>${row.message}</td></tr>`
      );
      renderRows("changes-body", payload.changes, (row) =>
        `<tr><td>${row.sequence}</td><td>${row.at_ms}</td><td>${row.domain}</td><td>${asText(row.previous_version)} -> ${asText(row.current_version)}</td><td>${row.summary}</td></tr>`
      );

      if (typeof payload.meta.sequence === "number") {
        state.since = Math.max(state.since, payload.meta.sequence);
      }
    }
    async function tick() {
      try {
        const response = await fetch(`/debug/verbose?since=${state.since}`, { cache: "no-store" });
        if (!response.ok) {
          badge(`http-${response.status}`, "warn");
          return;
        }
        const payload = await response.json();
        render(payload);
      } catch (err) {
        badge("offline", "err");
        console.error("debug ui fetch failed", err);
      }
    }
    tick();
    setInterval(tick, 900);
  </script>
</body>
</html>"#
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EndpointRole {
    Read,
    Admin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthDecision {
    Allowed,
    Unauthorized,
    Forbidden,
}

fn authorize_request(
    ctx: &ApiWorkerCtx,
    cfg: &RuntimeConfig,
    request: &HttpRequest,
) -> AuthDecision {
    let tokens = resolve_role_tokens(ctx, cfg);
    if tokens.read_token.is_none() && tokens.admin_token.is_none() {
        return AuthDecision::Allowed;
    }

    let Some(token) = extract_bearer_token(request) else {
        return AuthDecision::Unauthorized;
    };

    if let Some(expected_admin) = tokens.admin_token.as_deref() {
        if token == expected_admin {
            return AuthDecision::Allowed;
        }
    }

    match endpoint_role(request) {
        EndpointRole::Read => {
            if let Some(expected_read) = tokens.read_token.as_deref() {
                if token == expected_read {
                    return AuthDecision::Allowed;
                }
            }
            AuthDecision::Unauthorized
        }
        EndpointRole::Admin => {
            if let Some(expected_read) = tokens.read_token.as_deref() {
                if token == expected_read {
                    return AuthDecision::Forbidden;
                }
            }
            AuthDecision::Unauthorized
        }
    }
}

fn resolve_role_tokens(ctx: &ApiWorkerCtx, cfg: &RuntimeConfig) -> ApiRoleTokens {
    if let Some(configured) = ctx.role_tokens.as_ref() {
        return configured.clone();
    }

    match &cfg.api.security.auth {
        ApiAuthConfig::Disabled => ApiRoleTokens {
            read_token: None,
            admin_token: None,
        },
        ApiAuthConfig::RoleTokens(tokens) => ApiRoleTokens {
            read_token: normalize_runtime_token(tokens.read_token.clone()),
            admin_token: normalize_runtime_token(tokens.admin_token.clone()),
        },
    }
}

fn endpoint_role(request: &HttpRequest) -> EndpointRole {
    let (path, _query) = split_path_and_query(&request.path);
    match (request.method.as_str(), path) {
        ("POST", "/switchover")
        | ("POST", "/fallback/heartbeat")
        | ("DELETE", "/ha/switchover") => EndpointRole::Admin,
        _ => EndpointRole::Read,
    }
}

fn normalize_optional_token(raw: Option<String>) -> Result<Option<String>, WorkerError> {
    match raw {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Err(WorkerError::Message(
                    "role token must not be empty when configured".to_string(),
                ))
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        None => Ok(None),
    }
}

fn normalize_runtime_token(raw: Option<String>) -> Option<String> {
    match raw {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => None,
    }
}

enum ApiConnection {
    Plain(TcpStream),
    Tls(Box<TlsStream<TcpStream>>),
}

impl ApiConnection {
    async fn write_http_response(&mut self, response: HttpResponse) -> Result<(), WorkerError> {
        match self {
            Self::Plain(stream) => write_http_response(stream, response).await,
            Self::Tls(stream) => write_http_response(stream, response).await,
        }
    }

    async fn read_http_request(&mut self) -> Result<HttpRequest, String> {
        match self {
            Self::Plain(stream) => read_http_request(stream).await,
            Self::Tls(stream) => read_http_request(stream).await,
        }
    }
}

async fn accept_connection(
    ctx: &ApiWorkerCtx,
    cfg: &RuntimeConfig,
    peer: std::net::SocketAddr,
    stream: TcpStream,
) -> Result<Option<ApiConnection>, WorkerError> {
    match effective_tls_mode(ctx, cfg) {
        ApiTlsMode::Disabled => Ok(Some(ApiConnection::Plain(stream))),
        ApiTlsMode::Required => {
            let acceptor = require_tls_acceptor(ctx)?;
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    if ctx.require_client_cert && !has_peer_client_cert(&tls_stream) {
                        let mut event = api_event(
                            ApiEventKind::TlsClientCertMissing,
                            "failed",
                            SeverityText::Warn,
                            "tls client cert missing",
                        );
                        let fields = event.fields_mut();
                        fields.append_json_map(api_base_fields(ctx).into_attributes());
                        fields.insert("api.peer_addr", peer.to_string());
                        fields.insert("api.tls_mode", "required");
                        ctx.log
                            .emit_app_event("api_worker::accept_connection", event)
                            .map_err(|err| {
                                WorkerError::Message(format!(
                                    "api tls missing cert log emit failed: {err}"
                                ))
                            })?;
                        return Ok(None);
                    }
                    Ok(Some(ApiConnection::Tls(Box::new(tls_stream))))
                }
                Err(err) => {
                    let mut event = api_event(
                        ApiEventKind::TlsHandshakeFailed,
                        "failed",
                        SeverityText::Warn,
                        "tls handshake failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(api_base_fields(ctx).into_attributes());
                    fields.insert("api.peer_addr", peer.to_string());
                    fields.insert("api.tls_mode", "required");
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("api_worker::accept_connection", event)
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "api tls handshake log emit failed: {emit_err}"
                            ))
                        })?;
                    Ok(None)
                }
            }
        }
        ApiTlsMode::Optional => {
            if !looks_like_tls_client_hello(&stream).await? {
                return Ok(Some(ApiConnection::Plain(stream)));
            }

            let acceptor = require_tls_acceptor(ctx)?;
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    if ctx.require_client_cert && !has_peer_client_cert(&tls_stream) {
                        let mut event = api_event(
                            ApiEventKind::TlsClientCertMissing,
                            "failed",
                            SeverityText::Warn,
                            "tls client cert missing",
                        );
                        let fields = event.fields_mut();
                        fields.append_json_map(api_base_fields(ctx).into_attributes());
                        fields.insert("api.peer_addr", peer.to_string());
                        fields.insert("api.tls_mode", "optional");
                        ctx.log
                            .emit_app_event("api_worker::accept_connection", event)
                            .map_err(|err| {
                                WorkerError::Message(format!(
                                    "api tls missing cert log emit failed: {err}"
                                ))
                            })?;
                        return Ok(None);
                    }
                    Ok(Some(ApiConnection::Tls(Box::new(tls_stream))))
                }
                Err(err) => {
                    let mut event = api_event(
                        ApiEventKind::TlsHandshakeFailed,
                        "failed",
                        SeverityText::Warn,
                        "tls handshake failed",
                    );
                    let fields = event.fields_mut();
                    fields.append_json_map(api_base_fields(ctx).into_attributes());
                    fields.insert("api.peer_addr", peer.to_string());
                    fields.insert("api.tls_mode", "optional");
                    fields.insert("error", err.to_string());
                    ctx.log
                        .emit_app_event("api_worker::accept_connection", event)
                        .map_err(|emit_err| {
                            WorkerError::Message(format!(
                                "api tls handshake log emit failed: {emit_err}"
                            ))
                        })?;
                    Ok(None)
                }
            }
        }
    }
}

fn effective_tls_mode(ctx: &ApiWorkerCtx, cfg: &RuntimeConfig) -> ApiTlsMode {
    if let Some(mode) = ctx.tls_mode_override {
        return mode;
    }

    cfg.api.security.tls.mode
}

fn require_tls_acceptor(ctx: &ApiWorkerCtx) -> Result<TlsAcceptor, WorkerError> {
    ctx.tls_acceptor.clone().ok_or_else(|| {
        WorkerError::Message("tls mode requires a configured tls acceptor".to_string())
    })
}

fn has_peer_client_cert(stream: &TlsStream<TcpStream>) -> bool {
    let (_, connection) = stream.get_ref();
    connection
        .peer_certificates()
        .map(|certs| !certs.is_empty())
        .unwrap_or(false)
}

async fn looks_like_tls_client_hello(stream: &TcpStream) -> Result<bool, WorkerError> {
    let mut first = [0_u8; 1];
    match tokio::time::timeout(API_TLS_CLIENT_HELLO_PEEK_TIMEOUT, stream.peek(&mut first)).await {
        Err(_) => Ok(false),
        Ok(Ok(0)) => Ok(false),
        Ok(Ok(_)) => Ok(first[0] == 0x16),
        Ok(Err(err)) if err.kind() == std::io::ErrorKind::WouldBlock => Ok(false),
        Ok(Err(err)) => Err(WorkerError::Message(format!("api tls peek failed: {err}"))),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HttpRequest {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HttpResponse {
    status: u16,
    reason: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
}

impl HttpResponse {
    fn text(status: u16, reason: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            reason,
            content_type: "text/plain; charset=utf-8",
            body: body.into().into_bytes(),
        }
    }

    fn json<T: serde::Serialize>(status: u16, reason: &'static str, value: &T) -> Self {
        match serde_json::to_vec(value) {
            Ok(body) => Self {
                status,
                reason,
                content_type: "application/json",
                body,
            },
            Err(err) => Self::text(
                500,
                "Internal Server Error",
                format!("json encode failed: {err}"),
            ),
        }
    }

    fn html(status: u16, reason: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            reason,
            content_type: "text/html; charset=utf-8",
            body: body.into().into_bytes(),
        }
    }
}

async fn write_http_response<S>(stream: &mut S, response: HttpResponse) -> Result<(), WorkerError>
where
    S: AsyncWrite + Unpin,
{
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status,
        response.reason,
        response.content_type,
        response.body.len()
    );
    stream
        .write_all(header.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("api write header failed: {err}")))?;
    stream
        .write_all(&response.body)
        .await
        .map_err(|err| WorkerError::Message(format!("api write body failed: {err}")))?;
    Ok(())
}

async fn read_http_request<S>(stream: &mut S) -> Result<HttpRequest, String>
where
    S: AsyncRead + Unpin,
{
    let mut buffer = Vec::<u8>::new();
    let mut temp = [0u8; HTTP_REQUEST_SCRATCH_BUFFER_BYTES];
    let mut header_end: Option<usize> = None;
    let mut content_length: Option<usize> = None;

    loop {
        if buffer.len() > HTTP_REQUEST_MAX_BYTES {
            return Err("request too large".to_string());
        }

        let n = stream
            .read(&mut temp)
            .await
            .map_err(|err| err.to_string())?;
        if n == 0 {
            return Err("client closed connection".to_string());
        }
        buffer.extend_from_slice(&temp[..n]);

        if header_end.is_none() {
            if let Some(pos) = find_header_end(&buffer) {
                header_end = Some(pos);
            } else if buffer.len() > HTTP_REQUEST_HEADER_LIMIT_BYTES {
                return Err("headers too large".to_string());
            }
        }

        if let Some(end) = header_end {
            if content_length.is_none() {
                content_length = parse_content_length(&buffer).map_err(|err| err.to_string())?;
            }
            let body_len = content_length.unwrap_or(0);
            let required = end.saturating_add(body_len);
            if buffer.len() >= required {
                break;
            }
        }
    }

    let mut headers = [httparse::Header {
        name: "",
        value: &[],
    }; HTTP_REQUEST_HEADER_CAPACITY];
    let mut req = httparse::Request::new(&mut headers);
    let status = req.parse(&buffer).map_err(|err| err.to_string())?;
    let header_bytes = match status {
        httparse::Status::Complete(bytes) => bytes,
        httparse::Status::Partial => return Err("incomplete http request".to_string()),
    };

    let method = req
        .method
        .ok_or_else(|| "missing http method".to_string())?
        .to_string();
    let path = req
        .path
        .ok_or_else(|| "missing http path".to_string())?
        .to_string();

    let mut parsed_headers = Vec::new();
    for header in req.headers.iter() {
        parsed_headers.push((
            header.name.to_string(),
            String::from_utf8_lossy(header.value).to_string(),
        ));
    }

    let body_len = content_length.unwrap_or(0);
    let body_end = header_bytes
        .checked_add(body_len)
        .ok_or_else(|| "content-length overflow".to_string())?;
    if body_end > buffer.len() {
        return Err("incomplete http body".to_string());
    }

    Ok(HttpRequest {
        method,
        path,
        headers: parsed_headers,
        body: buffer[header_bytes..body_end].to_vec(),
    })
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

fn parse_content_length(buffer: &[u8]) -> Result<Option<usize>, String> {
    let mut headers = [httparse::Header {
        name: "",
        value: &[],
    }; 64];
    let mut req = httparse::Request::new(&mut headers);
    let status = req.parse(buffer).map_err(|err| err.to_string())?;
    match status {
        httparse::Status::Complete(_bytes) => {}
        httparse::Status::Partial => return Ok(None),
    }

    for header in req.headers.iter() {
        if header.name.eq_ignore_ascii_case("Content-Length") {
            let raw = String::from_utf8_lossy(header.value);
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(Some(0));
            }
            let parsed = trimmed
                .parse::<usize>()
                .map_err(|err| format!("invalid content-length: {err}"))?;
            return Ok(Some(parsed));
        }
    }
    Ok(Some(0))
}

fn extract_bearer_token(request: &HttpRequest) -> Option<String> {
    let header = request
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("Authorization"))
        .map(|(_, value)| value.as_str())?;

    let trimmed = header.trim();
    let prefix = "Bearer ";
    if !trimmed.starts_with(prefix) {
        return None;
    }
    Some(trimmed[prefix.len()..].trim().to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use rustls::{pki_types::ServerName, ClientConfig};
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio_rustls::TlsConnector;

    use crate::logging::{decode_app_event, LogHandle, LogSink, SeverityText, TestSink};

    use crate::{
        api::worker::{
            step_once, ApiWorkerCtx, HTTP_REQUEST_HEADER_LIMIT_BYTES,
            HTTP_REQUEST_SCRATCH_BUFFER_BYTES,
        },
        config::{ApiAuthConfig, ApiRoleTokensConfig, ApiTlsMode, InlineOrPath, RuntimeConfig},
        dcs::state::{DcsCache, DcsState, DcsTrust},
        dcs::store::{DcsStore, DcsStoreError, WatchEvent},
        debug_api::snapshot::{
            AppLifecycle, DebugChangeEvent, DebugDomain, DebugTimelineEntry, SystemSnapshot,
        },
        ha::{
            decision::HaDecision,
            state::{HaPhase, HaState},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{new_state_channel, UnixMillis, WorkerError},
        test_harness::{
            auth::ApiRoleTokens,
            namespace::NamespaceGuard,
            tls::{
                build_adversarial_tls_fixture, build_client_config, build_server_config,
                build_server_config_with_client_auth, write_tls_material,
            },
        },
    };

    #[derive(Clone, Default)]
    struct RecordingStore {
        writes: Arc<Mutex<Vec<(String, String)>>>,
        deletes: Arc<Mutex<Vec<String>>>,
    }

    impl RecordingStore {
        fn write_count(&self) -> Result<usize, WorkerError> {
            let guard = self
                .writes
                .lock()
                .map_err(|_| WorkerError::Message("writes lock poisoned".to_string()))?;
            Ok(guard.len())
        }

        fn delete_count(&self) -> Result<usize, WorkerError> {
            let guard = self
                .deletes
                .lock()
                .map_err(|_| WorkerError::Message("deletes lock poisoned".to_string()))?;
            Ok(guard.len())
        }

        fn deletes(&self) -> Result<Vec<String>, WorkerError> {
            let guard = self
                .deletes
                .lock()
                .map_err(|_| WorkerError::Message("deletes lock poisoned".to_string()))?;
            Ok(guard.clone())
        }
    }

    impl DcsStore for RecordingStore {
        fn healthy(&self) -> bool {
            true
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(())
        }

        fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(true)
        }

        fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
            let mut guard = self
                .deletes
                .lock()
                .map_err(|_| DcsStoreError::Io("deletes lock poisoned".to_string()))?;
            guard.push(path.to_string());
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            Ok(Vec::new())
        }
    }

    fn sample_runtime_config(auth_token: Option<String>) -> RuntimeConfig {
        let auth = match auth_token {
            Some(token) => ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
                read_token: Some(token.clone()),
                admin_token: Some(token),
            }),
            None => ApiAuthConfig::Disabled,
        };

        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_api_listen_addr("127.0.0.1:0")
            .with_api_auth(auth)
            .build()
    }

    fn sample_pg_state() -> PgInfoState {
        PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: crate::state::WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                pg_config: PgConfig {
                    port: Some(5432),
                    hot_standby: Some(false),
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
        }
    }

    fn sample_dcs_state(cfg: RuntimeConfig) -> DcsState {
        DcsState {
            worker: crate::state::WorkerStatus::Running,
            trust: DcsTrust::FullQuorum,
            cache: DcsCache {
                members: BTreeMap::new(),
                leader: None,
                switchover: None,
                config: cfg,
                init_lock: None,
            },
            last_refresh_at: Some(UnixMillis(1)),
        }
    }

    fn sample_process_state() -> ProcessState {
        ProcessState::Idle {
            worker: crate::state::WorkerStatus::Running,
            last_outcome: None,
        }
    }

    fn sample_ha_state() -> HaState {
        HaState {
            worker: crate::state::WorkerStatus::Running,
            phase: HaPhase::Replica,
            tick: 7,
            decision: HaDecision::EnterFailSafe {
                release_leader_lease: false,
            },
        }
    }

    fn sample_debug_snapshot(auth_token: Option<String>) -> SystemSnapshot {
        let cfg = sample_runtime_config(auth_token);
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone(), UnixMillis(1));
        let (_pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (_dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(cfg.clone()), UnixMillis(1));
        let (_process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (_ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));

        SystemSnapshot {
            app: AppLifecycle::Running,
            config: cfg_subscriber.latest(),
            pg: pg_subscriber.latest(),
            dcs: dcs_subscriber.latest(),
            process: process_subscriber.latest(),
            ha: ha_subscriber.latest(),
            generated_at: UnixMillis(1),
            sequence: 2,
            changes: vec![DebugChangeEvent {
                sequence: 1,
                at: UnixMillis(1),
                domain: DebugDomain::Config,
                previous_version: None,
                current_version: Some(cfg_subscriber.latest().version),
                summary: "config initialized".to_string(),
            }],
            timeline: vec![DebugTimelineEntry {
                sequence: 2,
                at: UnixMillis(1),
                domain: DebugDomain::Ha,
                message: "ha reached replica".to_string(),
            }],
        }
    }

    fn test_log_handle() -> (LogHandle, Arc<TestSink>) {
        let sink = Arc::new(TestSink::default());
        let sink_dyn: Arc<dyn LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
    }

    async fn build_ctx_with_config(
        cfg: RuntimeConfig,
    ) -> Result<(ApiWorkerCtx, RecordingStore), WorkerError> {
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

        let store = RecordingStore::default();
        let ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store.clone()));
        Ok((ctx, store))
    }

    async fn build_ctx_with_config_and_log(
        cfg: RuntimeConfig,
    ) -> Result<(ApiWorkerCtx, RecordingStore, Arc<TestSink>), WorkerError> {
        let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

        let store = RecordingStore::default();
        let (log, sink) = test_log_handle();
        let ctx = ApiWorkerCtx::new(listener, cfg_subscriber, Box::new(store.clone()), log);
        Ok((ctx, store, sink))
    }

    async fn build_ctx(
        auth_token: Option<String>,
    ) -> Result<(ApiWorkerCtx, RecordingStore), WorkerError> {
        build_ctx_with_config(sample_runtime_config(auth_token)).await
    }

    const MAX_BODY_BYTES: usize = 256 * 1024;
    const MAX_RESPONSE_BYTES: usize = HTTP_REQUEST_HEADER_LIMIT_BYTES + MAX_BODY_BYTES;
    const IO_TIMEOUT: Duration = Duration::from_secs(2);

    #[derive(Debug)]
    struct TestHttpResponse {
        status_code: u16,
        body: Vec<u8>,
    }

    #[derive(Debug)]
    struct ParsedHttpHead {
        status_code: u16,
        content_length: usize,
        body_start: usize,
    }

    fn parse_http_response_head(
        raw: &[u8],
        header_end: usize,
    ) -> Result<ParsedHttpHead, WorkerError> {
        let head = raw.get(..header_end).ok_or_else(|| {
            WorkerError::Message("response header end offset out of bounds".to_string())
        })?;

        let status_line_end = head
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or_else(|| WorkerError::Message("response missing status line".to_string()))?;

        let status_line_bytes = head.get(..status_line_end).ok_or_else(|| {
            WorkerError::Message("response status line offset out of bounds".to_string())
        })?;
        let status_line = std::str::from_utf8(status_line_bytes)
            .map_err(|err| WorkerError::Message(format!("response status line not utf8: {err}")))?;

        let mut status_parts = status_line.split_whitespace();
        let http_version = status_parts.next().ok_or_else(|| {
            WorkerError::Message("response status line missing http version".to_string())
        })?;
        if http_version != "HTTP/1.1" {
            return Err(WorkerError::Message(format!(
                "unexpected http version in response: {http_version}"
            )));
        }
        let status_str = status_parts.next().ok_or_else(|| {
            WorkerError::Message("response status line missing status code".to_string())
        })?;
        if status_str.len() != 3 || !status_str.bytes().all(|b| b.is_ascii_digit()) {
            return Err(WorkerError::Message(format!(
                "response status code must be 3 digits, got: {status_str}"
            )));
        }
        let status_code = status_str.parse::<u16>().map_err(|err| {
            WorkerError::Message(format!("response status code parse failed: {err}"))
        })?;
        if !(100..=599).contains(&status_code) {
            return Err(WorkerError::Message(format!(
                "response status code out of range: {status_code}"
            )));
        }

        let header_text = head.get(status_line_end + 2..).ok_or_else(|| {
            WorkerError::Message("response header offset out of bounds".to_string())
        })?;
        let header_text = std::str::from_utf8(header_text)
            .map_err(|err| WorkerError::Message(format!("response headers not utf8: {err}")))?;

        let mut content_length: Option<usize> = None;
        for line in header_text.split("\r\n") {
            if line.is_empty() {
                continue;
            }
            let (name, value) = line.split_once(':').ok_or_else(|| {
                WorkerError::Message(format!(
                    "invalid response header line (missing ':'): {line}"
                ))
            })?;
            if name.trim().eq_ignore_ascii_case("Content-Length") {
                if content_length.is_some() {
                    return Err(WorkerError::Message(
                        "response contains multiple Content-Length headers".to_string(),
                    ));
                }
                let parsed = value.trim().parse::<usize>().map_err(|err| {
                    WorkerError::Message(format!("response Content-Length parse failed: {err}"))
                })?;
                content_length = Some(parsed);
            }
        }

        let content_length = content_length.ok_or_else(|| {
            WorkerError::Message("response missing Content-Length header".to_string())
        })?;

        let body_start = header_end
            .checked_add(4)
            .ok_or_else(|| WorkerError::Message("response body offset overflow".to_string()))?;

        Ok(ParsedHttpHead {
            status_code,
            content_length,
            body_start,
        })
    }

    async fn read_http_response_framed(
        stream: &mut (impl AsyncRead + Unpin),
        timeout: Duration,
    ) -> Result<TestHttpResponse, WorkerError> {
        let response = tokio::time::timeout(timeout, async {
            let mut raw: Vec<u8> = Vec::new();
            let mut scratch = [0u8; HTTP_REQUEST_SCRATCH_BUFFER_BYTES];

            let mut parsed_head: Option<ParsedHttpHead> = None;
            let mut expected_total_len: Option<usize> = None;

            loop {
                if let Some(expected) = expected_total_len {
                    if raw.len() == expected {
                        let parsed = parsed_head.ok_or_else(|| {
                            WorkerError::Message("response framing parsed without header".to_string())
                        })?;
                        let body = raw
                            .get(parsed.body_start..expected)
                            .ok_or_else(|| {
                                WorkerError::Message(
                                    "response body slice out of bounds after framing".to_string(),
                                )
                            })?
                            .to_vec();
                        return Ok(TestHttpResponse {
                            status_code: parsed.status_code,
                            body,
                        });
                    }
                    if raw.len() > expected {
                        return Err(WorkerError::Message(format!(
                            "response exceeded expected length (expected {expected} bytes, got {})",
                            raw.len()
                        )));
                    }
                } else {
                    if raw.len() > HTTP_REQUEST_HEADER_LIMIT_BYTES {
                        return Err(WorkerError::Message(format!(
                            "response headers exceeded limit of {HTTP_REQUEST_HEADER_LIMIT_BYTES} bytes"
                        )));
                    }

                    if let Some(header_end) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                        let head = parse_http_response_head(&raw, header_end)?;
                        if head.content_length > MAX_BODY_BYTES {
                            return Err(WorkerError::Message(format!(
                                "response body exceeded limit of {MAX_BODY_BYTES} bytes (Content-Length={})",
                                head.content_length
                            )));
                        }
                        let expected =
                            head.body_start.checked_add(head.content_length).ok_or_else(|| {
                                WorkerError::Message("response total length overflow".to_string())
                            })?;
                        if expected > MAX_RESPONSE_BYTES {
                            return Err(WorkerError::Message(format!(
                                "response exceeded limit of {MAX_RESPONSE_BYTES} bytes (expected {expected})"
                            )));
                        }
                        parsed_head = Some(head);
                        expected_total_len = Some(expected);
                        continue;
                    }
                }

                let n = stream.read(&mut scratch).await.map_err(|err| {
                    WorkerError::Message(format!("client read failed: {err}"))
                })?;
                if n == 0 {
                    return Err(WorkerError::Message(format!(
                        "unexpected eof while reading response (read {} bytes so far)",
                        raw.len()
                    )));
                }

                let new_len = raw.len().checked_add(n).ok_or_else(|| {
                    WorkerError::Message("response length overflow while reading".to_string())
                })?;
                if new_len > MAX_RESPONSE_BYTES {
                    return Err(WorkerError::Message(format!(
                        "response exceeded limit of {MAX_RESPONSE_BYTES} bytes while reading (would reach {new_len})"
                    )));
                }
                raw.extend_from_slice(&scratch[..n]);
            }
        })
        .await;

        match response {
            Ok(inner) => inner,
            Err(_) => Err(WorkerError::Message(format!(
                "timed out reading framed http response after {}s",
                timeout.as_secs()
            ))),
        }
    }

    async fn send_plain_request(
        ctx: &mut ApiWorkerCtx,
        request_head: String,
        body: Option<Vec<u8>>,
    ) -> Result<TestHttpResponse, WorkerError> {
        let addr = ctx.local_addr()?;
        let mut client = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;

        client
            .write_all(request_head.as_bytes())
            .await
            .map_err(|err| WorkerError::Message(format!("client write header failed: {err}")))?;

        if let Some(body) = body {
            client
                .write_all(&body)
                .await
                .map_err(|err| WorkerError::Message(format!("client write body failed: {err}")))?;
        }

        step_once(ctx).await?;
        read_http_response_framed(&mut client, IO_TIMEOUT).await
    }

    async fn send_tls_request(
        ctx: &mut ApiWorkerCtx,
        client_config: Arc<ClientConfig>,
        server_name: &str,
        request_head: String,
        body: Option<Vec<u8>>,
    ) -> Result<TestHttpResponse, WorkerError> {
        let addr = ctx.local_addr()?;
        let tcp = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;

        let connector = TlsConnector::from(client_config);
        let server_name = ServerName::try_from(server_name.to_string()).map_err(|err| {
            WorkerError::Message(format!("invalid server name {server_name}: {err}"))
        })?;

        let client = async move {
            let mut tls = connector
                .connect(server_name, tcp)
                .await
                .map_err(|err| WorkerError::Message(format!("tls connect failed: {err}")))?;
            tls.write_all(request_head.as_bytes())
                .await
                .map_err(|err| WorkerError::Message(format!("tls write header failed: {err}")))?;
            if let Some(body) = body {
                tls.write_all(&body)
                    .await
                    .map_err(|err| WorkerError::Message(format!("tls write body failed: {err}")))?;
            }
            read_http_response_framed(&mut tls, IO_TIMEOUT).await
        };

        let (step_result, client_result) = tokio::join!(step_once(ctx), client);
        step_result?;
        client_result
    }

    async fn expect_tls_handshake_failure(
        ctx: &mut ApiWorkerCtx,
        client_config: Arc<ClientConfig>,
        server_name: &str,
    ) -> Result<(), WorkerError> {
        let addr = ctx.local_addr()?;
        let tcp = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;

        let connector = TlsConnector::from(client_config);
        let server_name = ServerName::try_from(server_name.to_string()).map_err(|err| {
            WorkerError::Message(format!("invalid server name {server_name}: {err}"))
        })?;

        let client = async move { connector.connect(server_name, tcp).await };
        let (step_result, client_result) = tokio::join!(step_once(ctx), client);
        step_result?;
        if client_result.is_ok() {
            return Err(WorkerError::Message(
                "expected tls handshake failure, but handshake succeeded".to_string(),
            ));
        }
        Ok(())
    }

    async fn expect_tls_request_rejected(
        ctx: &mut ApiWorkerCtx,
        client_config: Arc<ClientConfig>,
        server_name: &str,
    ) -> Result<(), WorkerError> {
        let result = send_tls_request(
            ctx,
            client_config,
            server_name,
            format_get("/fallback/cluster", None),
            None,
        )
        .await;

        match result {
            Ok(response) => {
                if response.status_code == 200 {
                    Err(WorkerError::Message(format!(
                        "expected tls request rejection, got status {}",
                        response.status_code
                    )))
                } else {
                    Ok(())
                }
            }
            Err(_) => Ok(()),
        }
    }

    fn format_get(path: &str, auth: Option<&str>) -> String {
        match auth {
            Some(auth_header) => format!(
                "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: {auth_header}\r\n\r\n"
            ),
            None => format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
        }
    }

    fn format_post(path: &str, auth: Option<&str>, body: &[u8]) -> String {
        match auth {
            Some(auth_header) => format!(
                "POST {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: {auth_header}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                body.len()
            ),
            None => format!(
                "POST {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
                body.len()
            ),
        }
    }

    fn format_delete(path: &str, auth: Option<&str>) -> String {
        match auth {
            Some(auth_header) => format!(
                "DELETE {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: {auth_header}\r\n\r\n"
            ),
            None => format!("DELETE {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_role_permissions_allow_read_deny_admin() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-role-read-deny")?;

        let (mut ctx, store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let response = send_plain_request(
            &mut ctx,
            format_get("/fallback/cluster", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let post_body = br#"{"requested_by":"node-a"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/switchover",
                Some(&roles.read_bearer_header()),
                post_body.as_slice(),
            ),
            Some(post_body),
        )
        .await?;
        assert_eq!(response.status_code, 403);
        assert_eq!(store.write_count()?, 0);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn auth_decision_logs_do_not_leak_bearer_token() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-auth-redaction")?;

        let (mut ctx, _store, sink) =
            build_ctx_with_config_and_log(sample_runtime_config(None)).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let secret = "super-secret-token-value";
        let auth_header = format!("Bearer {secret}");
        let response = send_plain_request(
            &mut ctx,
            format_get("/fallback/cluster", Some(auth_header.as_str())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 401);

        let records = sink
            .snapshot()
            .map_err(|err| WorkerError::Message(format!("log snapshot failed: {err}")))?;

        let auth_decision_present = records.iter().any(|record| {
            decode_app_event(record)
                .map(|event| event.header.name == "api.auth_decision")
                .unwrap_or(false)
        });
        if !auth_decision_present {
            return Err(WorkerError::Message(
                "expected api.auth_decision log event, but it was not emitted".to_string(),
            ));
        }

        for record in records {
            let encoded = serde_json::to_string(&record)
                .map_err(|err| WorkerError::Message(format!("encode log record failed: {err}")))?;
            if encoded.contains(secret) {
                return Err(WorkerError::Message(
                    "bearer token leaked into structured logs".to_string(),
                ));
            }
        }

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_role_permissions_allow_admin() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-role-admin-allow")?;

        let (mut ctx, store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let post_body = br#"{"requested_by":"node-a"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/switchover",
                Some(&roles.admin_bearer_header()),
                post_body.as_slice(),
            ),
            Some(post_body),
        )
        .await?;
        assert_eq!(response.status_code, 202);
        assert_eq!(store.write_count()?, 1);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ha_state_route_returns_typed_json_even_when_debug_disabled() -> Result<(), WorkerError>
    {
        let _guard = NamespaceGuard::new("api-ha-state-json")?;
        let mut cfg = sample_runtime_config(None);
        cfg.debug.enabled = false;
        let (mut ctx, _store) = build_ctx_with_config(cfg).await?;
        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response = send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert_eq!(response.status_code, 200);
        let decoded: serde_json::Value = serde_json::from_slice(&response.body)
            .map_err(|err| WorkerError::Message(format!("decode ha state json failed: {err}")))?;
        assert_eq!(decoded["cluster_name"], "cluster-a");
        assert_eq!(decoded["scope"], "scope-a");
        assert_eq!(decoded["self_member_id"], "node-a");
        assert_eq!(decoded["leader"], serde_json::Value::Null);
        assert_eq!(decoded["switchover_requested_by"], serde_json::Value::Null);
        assert_eq!(decoded["member_count"], 0);
        assert_eq!(decoded["dcs_trust"], "full_quorum");
        assert_eq!(decoded["ha_phase"], "replica");
        assert_eq!(decoded["ha_tick"], 7);
        assert_eq!(decoded["ha_decision"]["kind"], "enter_fail_safe");
        assert_eq!(decoded["ha_decision"]["release_leader_lease"], false);
        assert_eq!(decoded["snapshot_sequence"], 2);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ha_state_route_returns_503_without_subscriber() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-state-missing-subscriber")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let response = send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert_eq!(response.status_code, 503);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn ha_leader_routes_are_not_found_and_do_not_mutate_dcs_keys() -> Result<(), WorkerError>
    {
        let _guard = NamespaceGuard::new("api-ha-leader-routes-removed")?;
        let (mut ctx, store) = build_ctx(None).await?;

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post("/ha/leader", None, body.as_slice()),
            Some(body),
        )
        .await?;
        assert_eq!(response.status_code, 404);

        let response =
            send_plain_request(&mut ctx, format_delete("/ha/leader", None), None).await?;
        assert_eq!(response.status_code, 404);

        let response =
            send_plain_request(&mut ctx, format_delete("/ha/switchover", None), None).await?;
        assert_eq!(response.status_code, 202);

        assert_eq!(store.write_count()?, 0);

        assert_eq!(store.delete_count()?, 1);
        let deletes = store.deletes()?;
        assert_eq!(deletes, vec!["/scope-a/switchover"]);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_role_permissions_handle_removed_ha_leader_routes() -> Result<(), WorkerError>
    {
        let _guard = NamespaceGuard::new("api-ha-authz-removed-leader-routes")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/ha/leader",
                Some(&roles.read_bearer_header()),
                body.as_slice(),
            ),
            Some(body),
        )
        .await?;
        assert_eq!(response.status_code, 404);

        let body = br#"{"member_id":"node-b"}"#.to_vec();
        let response = send_plain_request(
            &mut ctx,
            format_post(
                "/ha/leader",
                Some(&roles.admin_bearer_header()),
                body.as_slice(),
            ),
            Some(body),
        )
        .await?;
        assert_eq!(response.status_code, 404);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_legacy_auth_token_fallback_protects_ha_routes() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-authz-legacy-fallback")?;
        let (mut ctx, _store) = build_ctx(Some("legacy-token".to_string())).await?;
        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response = send_plain_request(&mut ctx, format_get("/ha/state", None), None).await?;
        assert_eq!(response.status_code, 401);

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer legacy-token")),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_api_tokens_override_legacy_token() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-ha-authz-api-precedence")?;
        let mut cfg = sample_runtime_config(Some("legacy-token".to_string()));
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: Some("read-token".to_string()),
            admin_token: Some("admin-token".to_string()),
        });
        let (mut ctx, _store) = build_ctx_with_config(cfg).await?;
        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer legacy-token")),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 401);

        let response = send_plain_request(
            &mut ctx,
            format_get("/ha/state", Some("Bearer read-token")),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_verbose_route_returns_structured_json_and_since_filter(
    ) -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-verbose-json")?;
        let (mut ctx, _store) = build_ctx(None).await?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose?since=1", None), None).await?;
        assert_eq!(response.status_code, 200);

        let decoded: serde_json::Value = serde_json::from_slice(&response.body).map_err(|err| {
            WorkerError::Message(format!("decode debug verbose json failed: {err}"))
        })?;
        assert_eq!(decoded["meta"]["schema_version"], "v1");
        assert_eq!(decoded["meta"]["sequence"], 2);
        assert!(decoded["timeline"].is_array());
        assert!(decoded["changes"].is_array());
        assert_eq!(
            decoded["changes"].as_array().map(|value| value.len()),
            Some(0)
        );
        let endpoints = decoded["api"]["endpoints"].as_array().ok_or_else(|| {
            WorkerError::Message("debug verbose payload missing api.endpoints".to_string())
        })?;
        let contains_restore_route = endpoints.iter().any(|value| {
            value
                .as_str()
                .map(|route| route.contains("restore"))
                .unwrap_or(false)
        });
        assert!(!contains_restore_route);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_snapshot_route_is_kept_for_backward_compatibility() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-snapshot-compat")?;
        let (mut ctx, _store) = build_ctx(None).await?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response =
            send_plain_request(&mut ctx, format_get("/debug/snapshot", None), None).await?;
        assert_eq!(response.status_code, 200);
        let body_text = String::from_utf8(response.body)
            .map_err(|err| WorkerError::Message(format!("snapshot body not utf8: {err}")))?;
        assert!(body_text.contains("SystemSnapshot"));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_verbose_route_404_when_debug_disabled() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-disabled-404")?;
        let mut cfg = sample_runtime_config(None);
        cfg.debug.enabled = false;
        let (mut ctx, _store) = build_ctx_with_config(cfg).await?;
        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert_eq!(response.status_code, 404);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_verbose_route_503_without_subscriber() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-missing-subscriber")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert_eq!(response.status_code, 503);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_ui_route_returns_html_scaffold() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-ui-html")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let response = send_plain_request(&mut ctx, format_get("/debug/ui", None), None).await?;
        assert_eq!(response.status_code, 200);
        let html = String::from_utf8(response.body)
            .map_err(|err| WorkerError::Message(format!("ui body not utf8: {err}")))?;
        assert!(html.contains("id=\"meta-panel\""));
        assert!(html.contains("/debug/verbose"));
        assert!(html.contains("id=\"timeline-panel\""));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn debug_routes_require_auth_when_tokens_set() -> Result<(), WorkerError> {
        let _guard = NamespaceGuard::new("api-debug-authz")?;
        let (mut ctx, _store) = build_ctx(None).await?;
        let roles = ApiRoleTokens::new("read-token", "admin-token")?;
        ctx.configure_role_tokens(
            Some(roles.read_token.clone()),
            Some(roles.admin_token.clone()),
        )?;

        let snapshot = sample_debug_snapshot(None);
        let (_debug_publisher, debug_subscriber) = new_state_channel(snapshot, UnixMillis(1));
        ctx.set_ha_snapshot_subscriber(debug_subscriber);

        let response =
            send_plain_request(&mut ctx, format_get("/debug/verbose", None), None).await?;
        assert_eq!(response.status_code, 401);

        let response = send_plain_request(
            &mut ctx,
            format_get("/debug/verbose", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let response = send_plain_request(
            &mut ctx,
            format_get("/debug/ui", Some(&roles.read_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let response = send_plain_request(
            &mut ctx,
            format_get("/debug/verbose", Some(&roles.admin_bearer_header())),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_disabled_accepts_plain_rejects_tls() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-disabled")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "disabled",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;

        let response =
            send_plain_request(&mut ctx, format_get("/fallback/cluster", None), None).await?;
        assert_eq!(response.status_code, 200);

        let trusted_client = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx, trusted_client, "localhost").await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_optional_accepts_plain_and_tls() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-optional")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "optional",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Optional,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;

        let response =
            send_plain_request(&mut ctx, format_get("/fallback/cluster", None), None).await?;
        assert_eq!(response.status_code, 200);

        let client_cfg = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        let response = send_tls_request(
            &mut ctx,
            client_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_required_accepts_tls_rejects_plain() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-required")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "required",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;

        let client_cfg = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        let response = send_tls_request(
            &mut ctx,
            client_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let addr = ctx.local_addr()?;
        let mut plain = TcpStream::connect(addr)
            .await
            .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
        plain
            .write_all(format_get("/fallback/cluster", None).as_bytes())
            .await
            .map_err(|err| WorkerError::Message(format!("plain write failed: {err}")))?;
        step_once(&mut ctx).await?;
        let plain_result = read_http_response_framed(&mut plain, IO_TIMEOUT).await;
        if let Ok(plain_response) = plain_result {
            assert_ne!(plain_response.status_code, 200);
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_required_accepts_tls_with_production_tls_builder(
    ) -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-required-prod-builder")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material = write_tls_material(
            namespace,
            "required-prod-builder",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;

        let tls_cfg = crate::config::TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(crate::config::TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: fixture.valid_server.key_pem.clone(),
                },
            }),
            client_auth: None,
        };

        let server_cfg = crate::tls::build_rustls_server_config(&tls_cfg).map_err(|err| {
            WorkerError::Message(format!(
                "build production rustls server config failed: {err}"
            ))
        })?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(ApiTlsMode::Required, server_cfg)?;

        let client_cfg = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        let response = send_tls_request(
            &mut ctx,
            client_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_mtls_required_works_with_production_tls_builder() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-mtls-required-prod-builder")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material_server = write_tls_material(
            namespace,
            "mtls-server-prod-builder",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;
        let _material_trusted = write_tls_material(
            namespace,
            "mtls-trusted-client-prod-builder",
            Some(fixture.trusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.trusted_client.cert_pem.as_bytes()),
            Some(fixture.trusted_client.key_pem.as_bytes()),
        )?;
        let _material_untrusted = write_tls_material(
            namespace,
            "mtls-untrusted-client-prod-builder",
            Some(fixture.untrusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.key_pem.as_bytes()),
        )?;

        let tls_cfg = crate::config::TlsServerConfig {
            mode: ApiTlsMode::Required,
            identity: Some(crate::config::TlsServerIdentityConfig {
                cert_chain: InlineOrPath::Inline {
                    content: fixture.valid_server.cert_pem.clone(),
                },
                private_key: InlineOrPath::Inline {
                    content: fixture.valid_server.key_pem.clone(),
                },
            }),
            client_auth: Some(crate::config::TlsClientAuthConfig {
                client_ca: InlineOrPath::Inline {
                    content: fixture.trusted_client_ca.cert.cert_pem.clone(),
                },
                require_client_cert: true,
            }),
        };

        let server_cfg = crate::tls::build_rustls_server_config(&tls_cfg).map_err(|err| {
            WorkerError::Message(format!(
                "build production rustls server config failed: {err}"
            ))
        })?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(ApiTlsMode::Required, server_cfg)?;
        ctx.set_require_client_cert(true);

        let trusted_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.trusted_client),
            Some(&fixture.trusted_client_ca.cert),
        )?;
        let response = send_tls_request(
            &mut ctx,
            trusted_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let missing_client_cert_cfg =
            build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_request_rejected(&mut ctx, missing_client_cert_cfg, "localhost").await?;

        let untrusted_client_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.untrusted_client),
            Some(&fixture.untrusted_client_ca.cert),
        )?;
        expect_tls_request_rejected(&mut ctx, untrusted_client_cfg, "localhost").await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_tls_wrong_ca_and_hostname_and_expiry_failures() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-tls-failures")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material_valid = write_tls_material(
            namespace,
            "valid-server",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;
        let _material_expired = write_tls_material(
            namespace,
            "expired-server",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.expired_server.cert_pem.as_bytes()),
            Some(fixture.expired_server.key_pem.as_bytes()),
        )?;

        let (mut ctx_wrong_ca, _store) = build_ctx(None).await?;
        ctx_wrong_ca.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;
        let client_wrong_ca = build_client_config(&fixture.wrong_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_wrong_ca, client_wrong_ca, "localhost").await?;

        let (mut ctx_hostname, _store) = build_ctx(None).await?;
        ctx_hostname.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;
        let client_hostname = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_hostname, client_hostname, "not-localhost").await?;

        let (mut ctx_expired, _store) = build_ctx(None).await?;
        ctx_expired.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config(
                &fixture.expired_server,
                &fixture.valid_server_ca.cert,
            )?),
        )?;
        let client_expired = build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_handshake_failure(&mut ctx_expired, client_expired, "localhost").await?;

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_mtls_node_auth_allows_trusted_client_only() -> Result<(), WorkerError> {
        let guard = NamespaceGuard::new("api-mtls-node-auth")?;
        let namespace = guard.namespace()?;
        let fixture = build_adversarial_tls_fixture()?;

        let _material_server = write_tls_material(
            namespace,
            "mtls-server",
            Some(fixture.valid_server_ca.cert.cert_pem.as_bytes()),
            Some(fixture.valid_server.cert_pem.as_bytes()),
            Some(fixture.valid_server.key_pem.as_bytes()),
        )?;
        let _material_trusted = write_tls_material(
            namespace,
            "mtls-trusted-client",
            Some(fixture.trusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.trusted_client.cert_pem.as_bytes()),
            Some(fixture.trusted_client.key_pem.as_bytes()),
        )?;
        let _material_untrusted = write_tls_material(
            namespace,
            "mtls-untrusted-client",
            Some(fixture.untrusted_client_ca.cert.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.cert_pem.as_bytes()),
            Some(fixture.untrusted_client.key_pem.as_bytes()),
        )?;

        let (mut ctx, _store) = build_ctx(None).await?;
        ctx.configure_tls(
            ApiTlsMode::Required,
            Some(build_server_config_with_client_auth(
                &fixture.valid_server,
                &fixture.valid_server_ca.cert,
                &fixture.trusted_client_ca.cert,
            )?),
        )?;
        ctx.set_require_client_cert(true);

        let trusted_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.trusted_client),
            Some(&fixture.trusted_client_ca.cert),
        )?;
        let response = send_tls_request(
            &mut ctx,
            trusted_cfg,
            "localhost",
            format_get("/fallback/cluster", None),
            None,
        )
        .await?;
        assert_eq!(response.status_code, 200);

        let missing_client_cert_cfg =
            build_client_config(&fixture.valid_server_ca.cert, None, None)?;
        expect_tls_request_rejected(&mut ctx, missing_client_cert_cfg, "localhost").await?;

        let untrusted_client_cfg = build_client_config(
            &fixture.valid_server_ca.cert,
            Some(&fixture.untrusted_client),
            Some(&fixture.untrusted_client_ca.cert),
        )?;
        expect_tls_request_rejected(&mut ctx, untrusted_client_cfg, "localhost").await?;

        Ok(())
    }
}

--- END FILE: src/api/worker.rs ---

