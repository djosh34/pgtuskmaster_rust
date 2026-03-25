## Plan: Collapse Config Test String Renderers Onto Serializable Raw Schema

### Why this is the next verified reduction target

The config-v2 test support still maintains a string-rendering layer even though the parser already owns the source schema:

- `src/config_v2/parser/private_schema.rs`
  - `RuntimeDocument`, `OperatorDocument`, `PathSource`, `SecretSource`, `TokenAuthConfig`, and `ClientTlsInput` already model the raw input shape that tests are trying to create.
- `src/config_v2/parser/load_config.rs`
  - `join_rendered_sections(...)`, `toml_string(...)`, `toml_path_source(...)`, `toml_string_secret(...)`, `render_operator_test_config_toml(...)`, and `render_runtime_test_config_document_toml(...)` rebuild that same schema as handwritten TOML strings.
- `tests/ha/support/givens/mod.rs`
  - `render_ha_member_runtime_test_config_toml(...)` assembles a large array of TOML section strings, then `validate_runtime_config_for_host(...)` reparses the rendered string only to confirm it still matches the existing typed schema.
- `tests/ha/support/observer/pgtm.rs`
  - `build_host_observer_config(...)` does the same operator-side string assembly and reparsing loop.

That is a boundary problem: the raw config owner already exists, but tests are still maintaining a second representation as string fragments plus quoting helpers.

### Current overlap verified in code

1. The raw schema already owns the shapes the fixtures need.
   - `src/config_v2/parser/private_schema.rs`
     - `RuntimeDocument` already owns cluster, postgres, DCS, process, logging, api, and `pgtm`.
     - `OperatorDocument` already owns `api` plus `client_tls`.
     - `PathSource`, `PathOrInline`, `SecretSource`, `TaggedSecretSource`, `TokenAuthConfig`, `RoleTokens`, and `ClientTlsInput` already model the source-level fields used in the test fixtures.

2. The test-support API duplicates that ownership as handwritten TOML.
   - `src/config_v2/parser/load_config.rs`
     - test-only helpers manually quote strings, paths, and secrets even though the raw schema already has typed owners for those fields.
   - `src/config_v2/mod.rs`
   - `src/config_v2/parser/mod.rs`
     - publicly re-export the string helpers into the `internal-test-support` surface.

3. The HA helpers still maintain config content as strings instead of documents.
   - `tests/ha/support/givens/mod.rs`
     - manually formats `[postgres.network]`, `[postgres.rewind.transport]`, `[postgres.tls]`, `[process.binaries]`, `[logging]`, `[api]`, and `[pgtm.*]` sections.
   - `tests/ha/support/observer/pgtm.rs`
     - manually formats `[api.auth]` and `[client_tls]`.

4. The string layer causes redundant parse/validate loops.
   - `tests/ha/support/givens/mod.rs`
     - `validate_runtime_config_for_host(...)` reparses a string produced by the same helper that will be written later.
   - `tests/ha/support/observer/pgtm.rs`
     - rendered observer config is reparsed immediately after assembly to check validity.

### Revised execution plan

The prior execution proved the raw-schema serialization idea is viable, but the public surface was wrong:

- `src/config_v2/parser/mod.rs` exported a new `raw` module.
- `src/config_v2/mod.rs` re-exported `runtime_test_document`, `operator_test_document`, and `render_test_toml_with_sections`.
- `src/config_v2/parser/private_schema.rs` and `src/config_v2/types.rs` widened many owners from crate-private to public only so tests could build raw documents directly.
- `tests/ha/support/givens/mod.rs` and `tests/ha/support/observer/pgtm.rs` then rebuilt new path/secret/logging/auth helpers on top of that raw surface.

That removed one duplication layer and immediately added another, which is why the slice regressed by about `99` total lines.

1. Keep raw serialization private to the parser implementation.
   - Retain only the `Serialize` derives that `src/config_v2/parser/load_config.rs` actually needs for private raw-document serialization.
   - Revert all raw-schema and typed-config visibility widenings that were only introduced for tests.
   - Delete the public `raw` re-export from `src/config_v2/parser/mod.rs`.

2. Restore a narrow public test-support API.
   - Keep external callers on the existing string-returning helpers:
     - `render_runtime_test_config_document_toml(...)`
     - `render_runtime_test_config_toml(...)`
     - `render_operator_test_config_toml(...)`
   - Re-implement those helpers in `src/config_v2/parser/load_config.rs` by constructing private raw documents, serializing them, and merging extra TOML sections internally.
   - Do not expose `runtime_test_document`, `operator_test_document`, or `render_test_toml_with_sections` outside `load_config.rs`.

3. Remove the duplicate raw-builder layer from tests.
   - In `tests/ha/support/givens/mod.rs`, stop constructing `raw::RuntimeDocument` directly and go back to the narrow render helper plus the existing HA-specific extra sections.
   - In `tests/ha/support/observer/pgtm.rs`, stop constructing `raw::OperatorDocument` directly and go back to the narrow operator render helper.
   - Delete the local helper pile that only exists for raw construction:
     - `ha_logging_config(...)`
     - `path_source(...)`
     - `path_secret(...)`
     - `role_tokens_auth(...)`
     - `string_secret(...)`
     - `tls_identity(...)`

4. Keep only reductions that stand on their own.
   - Preserve the `tests/cli_binary.rs` helper consolidation only if it remains net-negative after the API rollback.
   - If `render_test_toml_with_sections(...)` ends up with a single caller after the rollback, inline it or make it private per smell 10 instead of exporting it.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_current_lines.sh` and confirm total lines drop below the current `34,282` and at least beat the prior `34,183` baseline.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+10194 -13987 diff: -3793`.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not introduce any new wrapper documents, builder structs, or parallel `*_test_*` types.
- Raw schema ownership stays in `src/config_v2/parser/private_schema.rs`; tests should not depend on those owners directly.
- Keep the effective parsed configs unchanged:
  - HA runtime fixtures must still validate and materialize the same runtime config shape.
  - observer/operator fixtures must still validate and produce the same operator config semantics.
- If the internal serializer still requires public cross-crate visibility after the raw export rollback, switch this plan back to `TO BE VERIFIED`.

### Expected yield

- Remove the test-only public/raw export surface added by the failed attempt.
- Delete the duplicate HA and observer raw-builder helper layer while keeping the internal serialized-document implementation.
- Preserve any independent reductions such as the CLI failure helper only if the total slice returns to net-negative.

NOW EXECUTE
