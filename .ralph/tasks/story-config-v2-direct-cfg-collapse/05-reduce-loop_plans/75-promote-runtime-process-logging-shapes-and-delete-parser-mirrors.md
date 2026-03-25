## Plan: Route Runtime Test Support Through The Canonical Parser

### Why this is the next verified reduction target

The parser already owns the canonical runtime defaults and path derivation, but the test-support entrypoints still rebuild that same runtime shape by hand.

- `src/config_v2/parser/load_config.rs:118-175` exposes `runtime_test_config`, `runtime_test_config_with_data_dir`, `managed_postgres_test_config`, and `trace_logging_test_config`.
- `src/config_v2/parser/load_config.rs:193-299` hand-constructs a full `RuntimeConfigV2` in `load_runtime_test_config_from_paths`, repeating cluster defaults, role defaults, working-root-derived paths, HA/process durations, logging defaults, and binary resolution behavior that the parser already knows.
- `src/config_v2/parser/load_config.rs:1299-1475` already contains TOML render helpers inside the unit-test module that express those same runtime defaults once and feed them through `load_runtime_config_contents`.
- `tests/ha/support/timeouts/mod.rs:27-42` depends on `load_runtime_timing_values` precisely because the HA harness must derive timing inputs from a runtime config file without forcing full runtime finalization against host-side binary paths.

The stale mirror-struct plan was wrong; the live boundary problem is duplicated runtime default knowledge in test support plus a second raw runtime-document parse path.

### Current overlap verified in code

- The manual builder in `load_runtime_test_config_from_paths` duplicates parser-owned defaults from `src/config_v2/types.rs` and parser finalize logic from `src/config_v2/parser/load_config.rs`, including:
  - `process.working_root` and derived logging/postgres paths
  - default HA/process timeout durations
  - default logging sink and postgres logging settings
  - default role names and inline password sources
  - binary resolution through `resolve_binary_path`
- The test module already has reusable TOML builders for runtime fixtures:
  - `join_rendered_sections`
  - `toml_string`
  - `render_runtime_test_config_toml`
  - `render_default_runtime_test_config_toml`
- `load_runtime_timing_values` reparses `raw::RuntimeDocument` directly instead of sharing the same parse entry that `load_runtime_config_contents_at` already uses.

### Execution plan

1. Promote the shared runtime test-config render helpers out of `mod tests`.
   - Move the runtime TOML helper functions now living in `src/config_v2/parser/load_config.rs:1310-1475` into private `#[cfg(any(test, feature = "internal-test-support"))]` support code near the public test helper entrypoints.
   - Keep only the operator-specific test helpers inside the unit-test module if they are not needed by runtime test support.
   - Do not create a second hand-built runtime config representation; the helpers should render canonical config text and feed the real parser.

2. Delete the manual `RuntimeConfigV2` builder.
   - Remove `load_runtime_test_config_from_paths`.
   - Reimplement `runtime_test_config` and `runtime_test_config_with_data_dir` by rendering the default runtime fixture TOML and loading it through `load_runtime_config_contents_at` or an equivalent shared parser entry.
   - Preserve the current absolute-path test behavior so relative-path resolution does not become accidental test coupling.

3. Keep specialized test configs as thin parser-backed variants.
   - Rework `managed_postgres_test_config` and `trace_logging_test_config` so they start from rendered config input instead of inheriting parser defaults and then rebuilding nested runtime structs by hand wherever possible.
   - If a tiny post-parse override remains clearer than extra TOML assembly, keep only that thin override layer; the large duplicated base builder must still be deleted.

4. Share runtime-document parsing for timing extraction.
   - Extract a small shared `parse_runtime_document` helper used by both `load_runtime_config_contents_at` and `load_runtime_timing_values`.
   - Keep `load_runtime_timing_values` as the lightweight host-side path for the HA harness, but make it read timing fields from the shared parsed document instead of open-coding another `toml::from_str::<raw::RuntimeDocument>` branch.
   - Do not switch the HA harness to full runtime finalization, because its fixture configs intentionally reference container-side binary paths that are not guaranteed to exist on the host.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff stays net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse existing runtime types and parser behavior; do not add a new normalized config layer.
- Keep config validation and path resolution inside `src/config_v2/parser/`.
- Prefer deleting duplicated defaults over moving them into another helper that still bypasses the parser.
- Leave the `ProcessConfig` custom deserialize helper alone unless it becomes obviously collapsible during execution without adding new schema types.

### Expected yield

- Delete the manual runtime test builder in `src/config_v2/parser/load_config.rs:193-299`.
- Delete or shrink the duplicated default-value knowledge currently spread across the public test helper entrypoints.
- Remove the extra raw runtime-document parse branch from `load_runtime_timing_values` by sharing the parse entrypoint.
- Keep the change net-negative in lines while improving the ownership boundary: parser owns config defaults, test support consumes parser output.

NOW EXECUTE
