## Plan: Delete Public Config Test Renderers

### Why this is the next reduction target

`src/config_v2/parser/private_schema.rs` still owns a wide cfg-gated test-rendering corridor, but the remaining non-parser callers are not generic config consumers. They are three concrete test owners:

- `tests/ha/support/givens/mod.rs` uses `render_ha_member_runtime_test_config_toml(...)` only to materialize HA runtime fixture text, then immediately re-validates that text with `validate_runtime_document_contents(...)`.
- `tests/ha/support/observer/pgtm.rs` uses `render_operator_test_config_toml(...)` plus `toml_path_source(...)` only to write one observer config shape, then immediately re-validates it with `load_operator_config_contents(...)`.
- `tests/cli_binary.rs` uses `render_runtime_test_config_toml(...)` only three times to generate temporary config files for binary integration tests.

That means the parser module is still exporting HA-specific and observer-specific fixture ownership that the harness already owns locally. The public cfg-gated API is now a courier layer between test callers and the real config loaders.

### Current overlap already verified

- `rg -n "render_ha_member_runtime_test_config_toml|render_operator_test_config_toml|toml_path_source|validate_runtime_document_contents|runtime_test_config\(|managed_postgres_test_config\(|trace_logging_test_config\(" src tests` shows the public render helpers are exported from `src/config_v2/mod.rs` and `src/config_v2/parser/mod.rs`, but external uses are limited to:
  - `tests/ha/support/givens/mod.rs`
  - `tests/ha/support/observer/pgtm.rs`
  - `tests/cli_binary.rs`
- `tests/ha/support/givens/mod.rs:158-166` renders HA runtime TOML and immediately runs `validate_runtime_document_contents(...)` on the same string.
- `tests/ha/support/observer/pgtm.rs:317-348` builds observer TOML, then immediately runs `load_operator_config_contents(...)` on that same string before writing the file.
- `src/config_v2/parser/private_schema.rs:763-1123` still carries:
  - `join_rendered_sections(...)`
  - `toml_path_source(...)`
  - `toml_string_secret(...)`
  - `render_ha_member_runtime_test_config_toml(...)`
  - `render_runtime_test_config_toml(...)`
  - `render_operator_test_config_toml(...)`
  - operator-document builder helpers that only exist to support those string renderers
- `src/config_v2/parser/load_config.rs:1035-1400` also depends on those render helpers only inside the parser test module, which means the surface can be narrowed without affecting production code.

### Execution plan

1. Make the HA harness own its runtime fixture text directly.
   - Move the HA member runtime TOML rendering logic out of `src/config_v2/parser/private_schema.rs` and into `tests/ha/support/givens/mod.rs`.
   - Keep the rendered TOML shape the same, but make `givens` own the member-specific endpoint, role-name, TLS-path, and secret-path substitutions itself.
   - Replace `validate_runtime_document_contents(...)` calls in `givens` with `load_runtime_config_contents(...)` so validation follows the canonical runtime loader instead of a second parse-only helper.

2. Make the observer harness own its operator fixture text directly.
   - Move the host-observer config builder logic fully into `tests/ha/support/observer/pgtm.rs`.
   - Replace `render_operator_test_config_toml(...)` and `toml_path_source(...)` usage with a local fixture renderer in that module.
   - Keep validation by parsing through `load_operator_config_contents(...)` in the observer module.

3. Remove the public runtime test renderer from the cross-crate API.
   - Replace the three `tests/cli_binary.rs` calls to `render_runtime_test_config_toml(...)` with one local helper in that test file (or one tiny test-only helper module under `tests/`) that emits exactly the TOML shape those binary tests need.
   - Keep `load_runtime_test_config_from_paths(...)` using an internal runtime renderer if it is still the shortest path for typed config construction, but do not re-export that renderer from `config_v2`.

4. Collapse the parser test-support surface to private helpers only.
   - Delete the public cfg-gated re-exports from `src/config_v2/mod.rs` and `src/config_v2/parser/mod.rs` for:
     - `render_ha_member_runtime_test_config_toml`
     - `render_operator_test_config_toml`
     - `render_runtime_test_config_toml`
     - `toml_path_source`
     - `toml_string_secret`
     - `validate_runtime_document_contents`
   - Keep any remaining string-render helpers private to `load_config.rs` tests or `private_schema.rs` internals only.
   - Delete now-dead helper functions in `private_schema.rs` after the external callers are gone.

5. Rebuild parser tests on the narrowed boundary.
   - Update `src/config_v2/parser/load_config.rs` tests to use private local helpers instead of public `config_v2` re-exports.
   - Keep the existing parser assertions, but stop treating parser-private fixture rendering as part of the crate’s cross-module API.

6. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to stay net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse the existing canonical loaders `load_runtime_config_contents(...)` and `load_operator_config_contents(...)`; do not invent new public DTOs or wrapper config types for tests.
- Keep `managed_postgres_test_config(...)`, `runtime_test_config(...)`, and `trace_logging_test_config(...)` if they still buy real typed-config reuse for in-crate tests; this slice is about deleting the public TOML renderer corridor, not deleting useful typed config constructors.
- Keep the HA and observer rendered TOML semantically identical unless a smaller caller-owned fixture format deletes more code without reducing clarity.
- If the local test helper replacements start growing into another shared renderer layer, switch this plan back to `TO BE VERIFIED` instead of recreating the same public API under a different module.

### Status

This slice is no longer the active execution target.

The public re-export corridor called out above has already been removed from `src/config_v2/mod.rs` and `src/config_v2/parser/mod.rs`, and the HA, observer, and CLI fixture owners now carry their own local TOML rendering helpers. The remaining reduction target is a smaller internal parser boundary, so continuing to execute this plan would be stale.

TO BE VERIFIED
