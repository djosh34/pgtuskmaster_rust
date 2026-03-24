## Plan: Delete Private-Schema Test Editors And Use Typed Config Fixtures

### Why this is the next reduction target

Plan 52 failed because it added a new edit surface instead of deleting a boundary:

- `src/config_v2/parser/private_schema.rs` now carries `RuntimeTestConfigEditor`, `OperatorTestConfigEditor`, `operator_client_tls_input(...)`, and two `*_with_edits(...)` render helpers.
- `rg` shows those new helpers only serve four in-crate unit-test modules:
  - `src/cli/config.rs`
  - `src/config_v2/parser/load_config.rs`
  - `src/process/cluster.rs`
  - `src/pginfo/worker.rs`
- The non-parser callers do not need TOML at all:
  - `src/process/cluster.rs` already uses `runtime_test_config_with_data_dir(...)` throughout the same test module.
  - `src/pginfo/worker.rs` only needs a loaded `RuntimeConfigV2`.
  - `src/cli/config.rs` already owns `resolve_operator_context_from_config(...)`, which accepts `Option<&OperatorConfigV2>` directly.
- The parser module is the correct place for raw TOML. Moving more schema mutation into `private_schema.rs` grows the wrong boundary.

This slice should therefore delete the failed helper layer and move only the non-parser tests onto typed config fixtures that already exist in the crate.

### Current overlap already verified

- `git diff --stat` shows the failed experiment is dominated by `src/config_v2/parser/private_schema.rs` with roughly `+295` lines.
- `src/process/cluster.rs` changed only one helper, `runtime_config_v2_with_source_ca(...)`, and that helper already returns `RuntimeConfigV2`, so it can mutate a typed config directly after `runtime_test_config_with_data_dir(...)`.
- `src/pginfo/worker.rs` does not need any overlay at all; its single test can use `runtime_test_config_with_data_dir(...)` directly.
- `src/cli/config.rs` currently renders TOML only to parse it straight back into `OperatorConfigV2` before calling `resolve_operator_context_from_config(...)`.
- `src/config_v2/parser/load_config.rs` is the only place in this failed slice where the TOML boundary is justified, because those tests explicitly validate parsing, normalization, and parse-time rejections.

### Execution plan

1. Revert the failed plan-52 helper surface before making new reductions.
   - Remove the crate-private re-exports of `render_runtime_test_config_toml_with_edits(...)` and `render_operator_test_config_toml_with_edits(...)` from:
     - `src/config_v2/mod.rs`
     - `src/config_v2/parser/mod.rs`
   - Delete the added editor-only code from `src/config_v2/parser/private_schema.rs`:
     - `RuntimeTestConfigEditor`
     - `OperatorTestConfigEditor`
     - `operator_client_tls_input(...)`
     - `render_runtime_test_config_toml_with_edits(...)`
     - `render_operator_test_config_toml_with_edits(...)`
   - Fold `build_operator_test_document_value(...)` back onto the simpler pre-editor shape if any split helper remains with only one caller.

2. Move the non-parser runtime tests onto typed `RuntimeConfigV2` fixtures.
   - In `src/process/cluster.rs`, rewrite `runtime_config_v2_with_source_ca(...)` to start from `runtime_test_config_with_data_dir(...)` and then mutate the returned config directly:
     - set `cfg.postgres.source_client_tls`
     - set binary override paths on `cfg.binaries`
   - In `src/pginfo/worker.rs`, replace the render+parse round-trip with `runtime_test_config_with_data_dir(...)`.
   - Prefer struct-field mutation on the already-validated config over introducing any new editor/helper abstraction.

3. Move the non-parser operator tests onto typed `OperatorConfigV2` fixtures.
   - In `src/cli/config.rs`, stop rendering/parsing TOML for tests that only exercise `resolve_base_url(...)` and `resolve_operator_context_from_config(...)`.
   - Add a small local typed fixture helper in the test module if it reduces repetition, but keep it local to `src/cli/config.rs`.
   - Reuse existing types instead of inventing new DTOs:
     - `OperatorConfigV2`
     - `PgClientTls`
     - `Secret`
     - `ApiRoute`
     - `PgtmApiTransportExpectation`
   - Keep the helper minimal: build a baseline `OperatorConfigV2`, then mutate only the specific fields each test needs.

4. Keep parser coverage inside `src/config_v2/parser/load_config.rs`.
   - Replace the failed editor-helper calls with parser-facing inputs again.
   - Use the existing `render_runtime_test_config_toml(...)` / `render_operator_test_config_toml(...)` plus small inline TOML overlays only where the test is specifically about parsing or normalization.
   - If multiple parser tests need the same baseline string tweak, add a tiny local helper in `load_config.rs` instead of reviving a shared schema editor API.

5. Validate reduction and correctness.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require improvement beyond the previous validated baseline `+8107 -11150 diff: -3043`.
   - Run all required checks:
     - `make check`
     - `make lint`
     - `make test`
     - `make test-long`

### Guardrails

- Do not keep any generic test-editor abstraction in `private_schema.rs`; that was the failed design.
- Do not widen raw schema type visibility just to make tests convenient.
- Do not change external integration-test helpers in `tests/cli_binary.rs` or `tests/ha/support/observer/pgtm.rs` in this slice.
- Do not move parser assertions out of `src/config_v2/parser/load_config.rs`; raw TOML belongs there.
- If typed `OperatorConfigV2` construction in `src/cli/config.rs` ends up needing large new helper code, stop and switch this plan back to `TO BE VERIFIED`.

### Expected yield

- Delete roughly the entire `+295` line helper spike from `src/config_v2/parser/private_schema.rs`.
- Delete the extra crate-private re-export glue.
- Replace at least three non-parser render+parse detours with direct typed fixtures, which should improve both code size and boundary clarity.
- Keep the parser module as the only owner of parser-oriented TOML fixtures.

NOW EXECUTE
