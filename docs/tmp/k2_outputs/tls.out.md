# TLS

TLS configuration types, validation, rustls server-config assembly, and error surface.

## Overview

`TlsServerConfig` controls TLS behavior for the API server. The configuration is validated during parser normalization and converted into a `rustls::ServerConfig` at runtime.

## Config types

### `InlineOrPath`

Untagged enum for certificate material.

| Variant | Fields |
|---------|--------|
| `Path` | `PathBuf` |
| `PathConfig` | `{ path: PathBuf }` |
| `Inline` | `{ content: String }` |

Behavior: `Path` and `PathConfig` read file content bytes; `Inline` uses UTF-8 bytes of `content`.

### `ApiTlsMode`

Serde representation: lowercase strings.

| Variant | String |
|---------|--------|
| `Disabled` | `disabled` |
| `Optional` | `optional` |
| `Required` | `required` |

### `TlsServerIdentityConfig`

| Field | Type |
|-------|------|
| `cert_chain` | `InlineOrPath` |
| `private_key` | `InlineOrPath` |

### `TlsClientAuthConfig`

| Field | Type |
|-------|------|
| `client_ca` | `InlineOrPath` |
| `require_client_cert` | `bool` |

### `TlsServerConfig`

Used by `PostgresConfig.tls` and `ApiSecurityConfig.tls`.

| Field | Type |
|-------|------|
| `mode` | `ApiTlsMode` |
| `identity` | `Option<TlsServerIdentityConfig>` |
| `client_auth` | `Option<TlsClientAuthConfig>` |

## Parser validation surface

`validate_tls_server_config` returns `Ok(())` when `mode` is `Disabled`. For `Optional` and `Required`:

- `identity` must be present or validation fails with message `tls identity must be configured when tls.mode is optional or required`
- `cert_chain` and `private_key` are validated with `validate_inline_or_path_non_empty(..., allow_empty_inline = false)`

`validate_tls_client_auth_config` returns `Ok(())` when `client_auth` is absent. When present:

- `mode = Disabled` fails with message `must not be configured when tls.mode is disabled`
- `client_ca` is validated with `validate_inline_or_path_non_empty(..., allow_empty_inline = false)`

`validate_postgres_conn_identity_ssl_mode_supported` rejects PostgreSQL connection `ssl_mode` values `require`, `verify-ca`, and `verify-full` when `postgres.tls.mode` is `Disabled`.

## Rustls builder behavior

`build_rustls_server_config(&TlsServerConfig) -> Result<Option<Arc<rustls::ServerConfig>>, TlsConfigError>`

### Mode handling

| `mode` | Returns | Requirement |
|--------|---------|-------------|
| `Disabled` | `Ok(None)` | none |
| `Optional` or `Required` | `Ok(Some(config))` | `identity` must be present |

When `identity` is absent for enabled modes, returns `TlsConfigError::InvalidConfig { message }` with message `tls.identity must be configured when tls.mode is optional or required`.

### PEM processing

- `load_inline_or_path_bytes` reads files for `Path` and `PathConfig`; returns `content.as_bytes().to_vec()` for `Inline`
- File read failures map to `TlsConfigError::Io` with field-qualified messages
- `parse_pem_cert_chain` uses `rustls_pemfile::certs`, maps parser failures to `TlsConfigError::PemParse`, and rejects empty chains with `no certificates found in PEM input`
- `parse_pem_private_key` uses `rustls_pemfile::private_key`, maps parser failures to `TlsConfigError::PemParse`, and rejects missing keys with `no private key found in PEM input`

### Config builder

Builder uses `rustls::crypto::ring::default_provider()` and `with_safe_default_protocol_versions()`.

**Without `client_auth`:**
- `with_no_client_auth()`
- `with_single_cert(cert_chain, key)`

**With `client_auth`:**
- Load `client_ca` via `load_inline_or_path_bytes("tls.client_auth.client_ca", ...)`
- Parse CA certificate chain
- Insert each certificate into `RootCertStore`
- Build `WebPkiClientVerifier::builder_with_provider(...)`
- Call `allow_unauthenticated()` when `require_client_cert` is `false`
- Use `with_client_cert_verifier(...)` and `with_single_cert(cert_chain, key)`

Root-store insertion, verifier build, protocol-version build, and certificate attachment failures map to `TlsConfigError::Rustls`.

## Error variants and constants

`TlsConfigError` enumeration:

| Variant | Source |
|---------|--------|
| `InvalidConfig { message }` | Invalid builder inputs |
| `Io { message }` | File read failures |
| `PemParse { message }` | PEM parse failure or missing PEM material |
| `Rustls { message }` | rustls builder, root store, verifier, or certificate attachment failure |

---

Verified in `src/tls.rs` tests:

- Optional mode without identity is rejected
- Required mode with inline identity and optional client auth builds successfully
- Missing certificate path maps to `TlsConfigError::Io`
- Invalid certificate chain maps to `TlsConfigError::PemParse`
- Invalid private key maps to `TlsConfigError::PemParse`

Verified in `src/config/parser.rs` tests:

- `client_auth` is rejected when `tls.mode` is `disabled`
