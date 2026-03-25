## Plan: Collapse Config Parser Source Resolution Onto Raw Schema Owners

### Why this is the next verified reduction target

The `config_v2` parser still keeps the wrong owner for several source-variant decisions. The raw schema already defines the real shapes for:

- `src/config_v2/parser/private_schema.rs:74-89`
  - `PathOrInline`
  - `PathSource`
  - `SecretSource`
- `src/config_v2/parser/private_schema.rs:112-119`
  - `TlsServerIdentityConfig`
  - `TlsClientIdentityConfig`
- `src/config_v2/parser/private_schema.rs:428-432`
  - `ClientTlsInput`

But `src/config_v2/parser/load_config.rs` still owns a separate helper layer that switches over those same variants:

- `src/config_v2/parser/load_config.rs:518-608`
  - `resolve_secret_optional(...)`
  - `resolve_secret_required(...)`
  - `resolve_path_only(...)`
  - `resolve_inline_or_path_string(...)`
  - `resolve_optional_path(...)`
- `src/config_v2/parser/load_config.rs:1078-1365`
  - TLS conversion paths for runtime postgres, DCS, API, and operator client TLS
  - duplicate identity-pair resolution through `resolve_path_pair(...)`, `TlsServerIdentityConfig::resolve(...)`, and `TlsClientIdentityConfig::resolve(...)`

That means the loader still carries source-resolution branching that should belong to the raw source and identity owners themselves. The result is duplicated path normalization, duplicated file reading, and duplicated TLS assembly entrypoints across one of the largest remaining files in the tree.

### Current overlap verified in code

1. Raw source variants exist already, but the loader reinterprets them manually.
   - `PathSource` and `PathOrInline` are defined in `src/config_v2/parser/private_schema.rs`, yet `load_config.rs` repeats the branching to resolve path-only values and inline-or-file values.
   - `SecretSource` already owns the file/env/string taxonomy, yet `resolve_secret_required(...)` still decodes those variants as a large free helper.

2. TLS identity resolution is split across multiple helper layers.
   - `raw::TlsServerConfig::into_runtime_tls(...)`
   - `raw::DcsTlsConfig::into_runtime_tls(...)`
   - `raw::ApiTransportConfig::into_runtime_transport(...)`
   - `raw::ClientTlsInput::into_runtime_dcs_tls(...)`
   - `raw::ClientTlsInput::into_pg_client_tls(...)`
   - plus `resolve_path_pair(...)`, `TlsServerIdentityConfig::resolve(...)`, and `TlsClientIdentityConfig::resolve(...)`

3. Existing tests already prove this boundary is important.
   - `src/config_v2/parser/load_config.rs:1644-1674` checks that non-path TLS sources are rejected.
   - `src/config_v2/parser/load_config.rs:1750-1811` checks merged operator client TLS behavior.
   - Those assertions currently sit on the loader even though the raw source owners define the allowed shapes.

### Execution plan

1. Move source resolution onto the raw schema owners in `src/config_v2/parser/private_schema.rs`.
   - Add methods on the existing owners instead of keeping free helpers in `load_config.rs`.
   - Cover these cases on the existing types:
     - `PathSource` resolving a normalized config-relative path
     - `Option<PathSource>` resolving an optional path without a separate helper
     - `PathOrInline` resolving file-or-inline string contents
     - `SecretSource` resolving required and optional secrets, including file/env/string handling
     - `TlsServerIdentityConfig` and `TlsClientIdentityConfig` resolving their path pairs
   - Do not introduce a new wrapper DTO for resolved TLS material.

2. Collapse the loader helper surface in `src/config_v2/parser/load_config.rs`.
   - Delete the helpers that become redundant:
     - `resolve_path_only(...)`
     - `resolve_optional_path(...)`
     - `resolve_inline_or_path_string(...)`
     - `resolve_path_pair(...)`
   - Reduce `resolve_secret_required(...)` / `resolve_secret_optional(...)` to nothing if possible; if one thin adapter remains temporarily, it must delegate straight to the raw owner and not re-switch on variants.

3. Rebuild TLS conversion paths from the owner methods only.
   - Rewrite:
     - `raw::TlsServerConfig::into_runtime_tls(...)`
     - `raw::DcsTlsConfig::into_runtime_tls(...)`
     - `raw::ApiTransportConfig::into_runtime_transport(...)`
     - `raw::ClientTlsInput::into_runtime_dcs_tls(...)`
     - `raw::ClientTlsInput::into_pg_client_tls(...)`
   - Keep `TlsConfig` and `PgClientTls` as the final shapes, but assemble them from the raw-owner methods instead of duplicating path-resolution logic at each callsite.
   - Preserve `merge_operator_client_tls(...)` semantics exactly.

4. Move focused tests to the actual owners.
   - Keep high-level load tests that verify full config behavior end to end.
   - Add focused unit tests around the raw source owners for:
     - path normalization
     - inline vs file content resolution
     - secret file/env/string handling and newline trimming behavior
     - TLS identity path-pair resolution
   - Keep or adapt the existing parse-boundary tests so path-only TLS identities are still enforced.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff is net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse the existing raw schema enums/structs; do not add a new intermediary resolved-source enum or DTO.
- Keep config error field names stable and concrete; do not hide which field or file failed.
- Preserve current newline trimming for secrets read from files.
- Preserve the current parse boundary that rejects inline TLS identity material where only path-based sources are allowed.
- Do not weaken operator TLS merge validation; mismatched API/postgres TLS identity or CA values must still fail.

### Expected yield

- Shrink `src/config_v2/parser/load_config.rs` by deleting its shadow source-resolution helper layer.
- Keep source-variant ownership on `PathSource`, `PathOrInline`, `SecretSource`, and the existing TLS identity structs in `private_schema.rs`.
- Leave the loader focused on composing typed runtime/operator config, not decoding raw source variants.
- Reduce duplicate TLS path assembly across runtime postgres, DCS, API, and operator config loading.

NOW EXECUTE
