## Plan: Collapse Config Test-Support Boundary

### Why this reduction target

The previous slice proved that raw-schema serialization itself is not the problem anymore. The remaining inflation is the helper boundary around it:

- `src/config_v2/parser/private_schema.rs` already owns the raw runtime/operator document shapes and the test document builders.
- `src/config_v2/parser/load_config.rs` and `src/config_v2/parser/load_operator_config.rs` already own the real parse and validation semantics.
- `src/dev_support/runtime_config.rs` still adds another layer for test rendering, validation, temp-file creation, TOML string fragment assembly, and tiny wrapper helpers.
- Many tests then add a fourth layer on top of that helper API by writing temp TOML files purely to call the loader again.

That is a boundary problem. The config module already knows how to build, parse, and validate these documents, but tests are routed through `dev_support` and filesystem scaffolding anyway. That duplicates ownership and keeps a large amount of stringly test code alive.

### Current overlap already verified

- `src/config_v2/parser/private_schema.rs` exposes `build_runtime_test_document_value`, `build_ha_member_runtime_document_value`, `build_operator_test_document_value`, and `build_host_observer_operator_document_value`.
- `src/config_v2/parser/load_config.rs` reads file contents and immediately parses `raw::RuntimeDocument` from TOML text before mapping and validating it.
- `src/config_v2/parser/load_operator_config.rs` follows the same pattern for operator config documents and runtime documents with embedded `pgtm`.
- `src/dev_support/runtime_config.rs` still owns `write_temp_toml`, `validate_runtime_config_contents`, `validate_operator_config_contents`, `render_runtime_test_config_toml`, `render_ha_member_runtime_config_toml`, `render_operator_test_config_toml`, `render_host_observer_operator_config_toml`, `join_rendered_sections`, `render_toml_value`, `toml_path_source`, and `toml_string_secret`.
- `src/config_v2/parser/load_operator_config.rs` tests, `src/cli/config.rs` tests, `src/pginfo/worker.rs` tests, and `tests/ha/support/{world,observer/pgtm}.rs` all still depend on that helper layer.
- `tests/cli_binary.rs` is a special case: it really does need on-disk config files for subprocess execution, but it still does not need `dev_support` to own config rendering or validation.

### Execution plan

1. Move test-facing config rendering and in-memory parsing onto the config module itself.
   - Add `config_v2` test-support helpers for `load_runtime_config_contents` and `load_operator_config_contents` that reuse the existing raw document parsing/mapping path after `read_config_file`.
   - Add one shared TOML render helper near the raw document builders so the config module can render raw document values and merge parsed extra TOML fragments without going through `dev_support`.
   - Keep the path-based public loaders as thin file-reading entrypoints that delegate to the same contents-based parsing path.

2. Collapse the `dev_support/runtime_config.rs` config-document helper layer.
   - Remove the rendering, validation, temp-file, and TOML fragment helpers from `src/dev_support/runtime_config.rs`.
   - Keep only typed runtime-config construction support there, unless some of that builder logic can also be merged or reduced naturally while updating call sites.
   - Do not introduce a second “test support” module outside `config_v2`; ownership should move inward, not sideways.

3. Update callers to use the config-owned surface directly.
   - Change in-process parser tests and unit tests (`src/config_v2/parser/load_operator_config.rs`, `src/cli/config.rs`, `src/pginfo/worker.rs`, and similar callers) to parse directly from TOML contents instead of writing temp files first.
   - Keep real file writes only in subprocess-style tests such as `tests/cli_binary.rs`, but have those tests render config through the config-owned helper surface rather than the `dev_support` wrapper layer.
   - Update HA support code (`tests/ha/support/world/mod.rs` and `tests/ha/support/observer/pgtm.rs`) to validate rendered configs through config-owned helpers, deleting their remaining thin wrapper functions if they become redundant.

4. Remove leftover overlap in the operator/runtime loader tests.
   - Replace repeated `write_temp_toml` boilerplate with one small contents-based assertion path wherever the test is verifying config parsing rather than filesystem behavior.
   - Reuse the same helper path for runtime documents containing `pgtm` and standalone operator documents, instead of keeping two testing styles alive.

5. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff moves below the current `+4028 -5876 diff: -1848` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Tick the task boxes only after the gates pass.

### Guardrails

- Reuse the existing raw config structs and existing loader mapping functions. Do not create new runtime/operator document types.
- Keep file-path loader behavior intact for production code; the new contents-based loaders are for shared parsing and test reduction, not a behavioral redesign.
- Prefer deleting call-site wrappers over replacing them with new generic helper abstractions.
- If merging extra TOML fragments into base document values requires more adapter code than it deletes, switch this plan back to `TO BE VERIFIED`, document the mismatch clearly, retarget the task file, and stop immediately.

NOW EXECUTE
