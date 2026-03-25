## Plan: Collapse Operator Client TLS Onto PgClientTls

### Why this reduction target

The operator config and CLI path still keep a client-TLS shadow type that mostly restates an existing one:

- `src/pginfo/conninfo.rs::PgClientTls` already models client-side TLS material as `root_cert`, `client_cert`, and `client_key`.
- `src/config_v2/types.rs::OperatorClientTlsConfig` stores the same facts as `ca_cert` plus optional identity cert/key wrapped in `TlsConfig`.
- `src/config_v2/parser/load_config.rs::resolve_operator_client_tls(...)` builds that operator-only wrapper from `raw::OperatorClientTlsInput`.
- `src/cli/client.rs::apply_tls_config(...)` and `src/cli/connect.rs::build_connection_tls(...)` immediately unpack the wrapper back into CA, cert, and key file paths.

That boundary does not add new semantics. The only extra fact for PostgreSQL connections is the SSL mode, and the CLI already hardcodes that to `verify-full` when TLS emission is requested. Reusing `PgClientTls` lets the parser and CLI share one canonical client-TLS shape instead of translating through `OperatorClientTlsConfig`.

### Current overlap already verified

- `src/pginfo/conninfo.rs` defines `PgClientTls { mode, root_cert, client_cert, client_key }`.
- `src/config_v2/types.rs` defines `OperatorClientTlsConfig { ca_cert, identity: Option<TlsConfig> }`, where `TlsConfig` only contributes `cert` and `key` because `ca_cert` is always `None` for operator client identity.
- `src/config_v2/parser/load_config.rs::resolve_operator_client_tls(...)` maps raw operator TLS input into `OperatorClientTlsConfig`.
- `src/config_v2/parser/load_config.rs::merge_optional_identity(...)` only compares cert/key identity paths, which is the same information `PgClientTls` can already hold directly.
- `src/cli/client.rs::apply_tls_config(...)` reads only `ca_cert`, `identity.cert`, and `identity.key`.
- `src/cli/connect.rs::build_connection_tls(...)` converts `OperatorClientTlsConfig` straight into `PgClientTls` and injects `PgSslMode::VerifyFull`.
- `rg` shows `OperatorClientTlsConfig` is only used in `src/config_v2/types.rs`, `src/config_v2/parser/load_config.rs`, and the CLI modules/tests, so removing it is a bounded refactor.

### Execution plan

1. Replace the operator-only TLS wrapper with the existing client TLS type.
   - Remove `OperatorClientTlsConfig` from `src/config_v2/types.rs`.
   - Change `OperatorConfigV2.client_tls`, `cli::config::OperatorContext.postgres_client_tls`, and `cli::client::CliApiClientConfig.tls` to use `Option<PgClientTls>` or the narrowest equivalent reuse of `PgClientTls`.
   - Import `PgClientTls` from `src/pginfo/conninfo.rs` instead of keeping an operator-specific TLS shape.

2. Collapse operator-config parsing onto `PgClientTls`.
   - Replace `resolve_operator_client_tls(...) -> Result<OperatorClientTlsConfig, _>` with a helper that returns `Result<Option<PgClientTls>, _>`.
   - Map operator CA/cert/key inputs directly onto `root_cert`, `client_cert`, and `client_key`.
   - Replace `merge_optional_identity(...)` and related merge code with direct `PgClientTls` field comparison so the merged operator TLS output is still validated exactly once.

3. Simplify CLI consumers to read the surviving type directly.
   - Update `src/cli/client.rs::apply_tls_config(...)` to read `root_cert`, `client_cert`, and `client_key` from `PgClientTls`.
   - Update `src/cli/connect.rs::build_connection_tls(...)` so it either returns disabled TLS when `emit_tls` is false or reuses the parsed `PgClientTls` paths when TLS emission is requested.
   - Keep current behavior for HTTPS operator API clients with no configured certificate paths: absence of configured files must remain a no-op, not an error.

4. Rebuild tests around the surviving type.
   - Replace `OperatorClientTlsConfig` fixtures in CLI and parser tests with `PgClientTls` or `Option<PgClientTls>`.
   - Keep assertions that operator config merging still preserves CA/cert/key paths, that CLI config still exposes TLS to both API and PostgreSQL clients, and that TLS-disabled connection rendering still returns `sslmode=disable`.
   - Delete fixture code whose only purpose was constructing the removed wrapper.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+7204 -9998 diff: -2794` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long` sequentially.

### Guardrails

- Do not add a replacement operator-only TLS wrapper or a second client identity struct.
- Do not widen `PgClientTls` unless a concrete missing field is required by existing behavior.
- Keep operator config semantics stable: conflicting API/Postgres TLS inputs must still error, and HTTPS client setup without explicit cert paths must still work.
- If `PgClientTls` reuse forces extra adapter code that makes the parser or CLI larger, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
