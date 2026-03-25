## Plan: Localize Config Test String Support And Delete Export Corridor

### Why this is the next reduction target

`src/config_v2/parser/load_config.rs` still owns a cfg-gated string-rendering corridor that leaks across crate boundaries even though the remaining non-parser callers are concrete test owners:

- `src/config_v2/mod.rs` and `src/config_v2/parser/mod.rs` still re-export:
  - `render_operator_test_config_toml(...)`
  - `render_runtime_test_config_document_toml(...)`
  - `toml_path_source(...)`
  - `toml_string(...)`
  - `toml_string_secret(...)`
- `tests/ha/support/givens/mod.rs` is the only external caller of the runtime renderer plus the TOML quoting helpers.
- `tests/ha/support/observer/pgtm.rs` is the only external caller of the operator renderer plus `toml_path_source(...)`.
- `tests/cli_binary.rs` is the only other external caller of the runtime renderer, and it uses it only to write three temporary invalid-config fixtures.
- Inside `src/config_v2/parser/load_config.rs`, the parser tests also use the operator renderer and TOML quoting helpers, but those uses are local to the parser owner.

That is the wrong boundary: parser-owned fixture rendering and TOML quoting are still being treated as cross-crate test-support API even though the remaining external callers already own the exact fixture shapes they need.

### Current overlap already verified

1. The export corridor is still live today.
   - `src/config_v2/mod.rs`
   - `src/config_v2/parser/mod.rs`
   - both still publicly re-export the cfg-gated renderer and TOML quoting helpers.

2. External usage is narrow and specific, not generic.
   - `tests/ha/support/givens/mod.rs`
     - uses `render_runtime_test_config_document_toml(...)`, `toml_path_source(...)`, `toml_string(...)`, and `toml_string_secret(...)`.
   - `tests/ha/support/observer/pgtm.rs`
     - uses `render_operator_test_config_toml(...)` and `toml_path_source(...)`.
   - `tests/cli_binary.rs`
     - uses `render_runtime_test_config_document_toml(...)` for `assert_node_runtime_config_failure(...)`.

3. The parser owner already has its own local consumers.
   - `src/config_v2/parser/load_config.rs`
     - `load_runtime_test_config_with_hba_and_sections(...)` uses `render_runtime_test_config_document_toml(...)` internally.
   - `src/config_v2/parser/load_config.rs` test module
     - uses `render_operator_test_config_toml(...)`, `toml_path_source(...)`, and `toml_string_secret(...)`.

4. The external callers already own the concrete fixture semantics.
   - `tests/ha/support/givens/mod.rs`
     - already owns the HA-only runtime sections, TLS paths, process binary paths, and API/operator token semantics.
   - `tests/ha/support/observer/pgtm.rs`
     - already owns the observer HTTPS/auth/TLS fixture shape.
   - `tests/cli_binary.rs`
     - only needs small invalid runtime config files with a stable base document and a few extra sections.

### Execution plan

1. Move caller-owned fixture text fully back to the callers.
   - In `tests/ha/support/observer/pgtm.rs`, replace `render_operator_test_config_toml(...)` and `toml_path_source(...)` with a local observer-config renderer.
   - In `tests/cli_binary.rs`, replace `render_runtime_test_config_document_toml(...)` with one local helper that emits the minimal runtime config text those three CLI cases need.
   - In `tests/ha/support/givens/mod.rs`, replace the shared TOML quoting helpers with caller-local quoting helpers and a caller-local runtime fixture renderer, but keep the final validation through `load_runtime_config_contents(...)`.

2. Narrow the parser-owned string helpers to parser-local ownership.
   - Keep `render_runtime_test_config_document_toml(...)` only if `load_runtime_test_config_with_hba_and_sections(...)` still needs it internally; otherwise inline it there.
   - Move `render_operator_test_config_toml(...)`, `toml_path_source(...)`, and `toml_string_secret(...)` into the `load_config.rs` test module if they become parser-test-only after step 1.
   - Delete `toml_string(...)` entirely if no parser code still needs it.

3. Delete the cross-crate export corridor.
   - Remove the cfg-gated re-exports from `src/config_v2/mod.rs` and `src/config_v2/parser/mod.rs` for:
     - `render_operator_test_config_toml`
     - `render_runtime_test_config_document_toml`
     - `toml_path_source`
     - `toml_string`
     - `toml_string_secret`
   - Keep `load_runtime_test_config_with_hba_and_sections(...)`, `runtime_test_config_with_data_dir(...)`, and the config loaders as the surviving shared helpers.

4. Delete any dead parser-side glue exposed only for the old export surface.
   - Remove dead imports and cfg gates in `src/config_v2/parser/load_config.rs`.
   - If a helper now has exactly one remaining internal caller, inline it instead of keeping another thin wrapper.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_current_lines.sh` and require total lines to fall below the current `34,047`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require improvement beyond the current `+10331 -14359 diff: -4028`.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not invent a new shared test-support module for TOML rendering. The point is to move ownership to the existing callers or keep it private to `load_config.rs`.
- Reuse existing loader entrypoints for validation:
  - `load_runtime_config_contents(...)`
  - `load_operator_config_contents(...)`
- Do not widen private-schema visibility again.
- Prefer deleting thin wrappers over moving them elsewhere unchanged.
- If the caller-local runtime fixture renderer in `tests/ha/support/givens/mod.rs` starts duplicating more baseline runtime TOML than the removed export corridor and parser helpers, switch this plan back to `TO BE VERIFIED`.

### Expected yield

- Delete the cfg-gated public re-export corridor for parser-owned string helpers.
- Remove at least one parser-side TOML quoting helper entirely and move the remaining parser-test-only helpers out of the main test-support surface.
- Reduce one more boundary layer between parser internals and HA/observer/CLI fixture ownership.

NOW EXECUTE
