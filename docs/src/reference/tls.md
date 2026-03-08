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
