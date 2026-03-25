## Plan: Collapse Managed Postgres TLS Shadow Type

### Why this reduction target

`src/postgres_managed.rs` still owns a small but pure shadow type layer for Postgres TLS:

- `config_v2::types::TlsConfig` already stores the runtime TLS file paths as `cert`, `key`, and optional `ca_cert`.
- `postgres_managed.rs::managed_tls_config(...)` immediately remaps that into a second local shape, `ManagedPostgresTlsConfig::{Disabled, Enabled { cert_file, key_file, ca_file }}`.
- `render_managed_postgres_conf(...)` then matches that shadow enum only to emit the same `ssl`, `ssl_cert_file`, `ssl_key_file`, and `ssl_ca_file` settings.
- The tests in `src/postgres_managed.rs` also fabricate `ManagedPostgresTlsConfig` directly just to drive the renderer.

That boundary adds lines without adding semantics. The managed Postgres owner only needs one owner-local validation pass that confirms the configured files exist and are absolute, then it can keep using the existing `TlsConfig`.

### Current overlap already verified

- `src/config_v2/types.rs` defines `TlsConfig { cert, key, ca_cert }`.
- `src/postgres_managed.rs` defines `ManagedPostgresTlsConfig::Disabled` and `ManagedPostgresTlsConfig::Enabled { cert_file, key_file, ca_file }`, which is the same state with renamed fields plus `None` encoded as a local enum variant.
- `managed_tls_config(...)` rebuilds the runtime config TLS state into that shadow enum through `resolve_existing_configured_file(...)`.
- `render_managed_postgres_conf(...)` consumes the shadow enum immediately and does not expose it outside the module.
- `rg` shows `ManagedPostgresTlsConfig` is only used inside `src/postgres_managed.rs` and its tests, so deleting it does not require a new shared abstraction.

### Execution plan

1. Reuse the canonical runtime TLS shape inside `src/postgres_managed.rs`.
   - Remove `ManagedPostgresTlsConfig`.
   - Import and reuse `config_v2::types::TlsConfig`.
   - Replace the shadow `Disabled` case with `None` and the enabled case with `Some(TlsConfig)`.

2. Keep owner-local validation while returning the surviving type.
   - Replace `managed_tls_config(...)` with a helper that validates `cfg.postgres.tls` and returns `Result<Option<TlsConfig>, ManagedPostgresError>`.
   - Preserve the current field-specific error messages from `resolve_existing_configured_file(...)`.
   - Keep path normalization and existence checks in `postgres_managed.rs`; do not push them into the parser or another module.

3. Collapse rendering onto the surviving type.
   - Change `render_managed_postgres_conf(...)` to consume `Option<&TlsConfig>` or an equivalent borrow of the validated TLS config.
   - Match directly on `None` versus `Some(tls)` when writing `ssl`, `ssl_cert_file`, `ssl_key_file`, and `ssl_ca_file`.
   - Keep rendered config output and error wording stable.

4. Rebuild tests around the surviving config type.
   - Replace `sample_render_tls_config()` and any direct `ManagedPostgresTlsConfig` fixtures with ordinary `TlsConfig` values.
   - Keep the existing assertions about deterministic rendering, quoting, TLS path reuse, and managed file materialization.
   - Delete tests or fixture code whose only purpose was constructing the removed shadow enum.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+7216 -9996 diff: -2780` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long` sequentially.

### Guardrails

- Do not add a replacement wrapper enum or managed-only TLS struct.
- Do not widen `TlsConfig` just to preserve the deleted boundary.
- Keep validation and rendering in `src/postgres_managed.rs`; this slice is about reusing the existing type, not moving ownership elsewhere.
- If reusing `TlsConfig` forces extra cloning or adapter code that makes the module larger, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
