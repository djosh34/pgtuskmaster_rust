## Plan: Collapse Handwritten Test Config TOML

### Why this reduction target

The codebase still carries a large amount of handwritten runtime and operator TOML in tests and HA support even though shared config validation and runtime-config helpers already exist. The biggest remaining examples are:

- `tests/ha/support/world/mod.rs`, where `render_member_runtime_config` keeps a ~100-line runtime document template inline.
- `tests/ha/support/observer/pgtm.rs`, where `build_host_observer_config` hand-renders an operator config with the same TLS/token path concepts.
- `src/config_v2/parser/load_config.rs`, `src/config_v2/parser/load_operator_config.rs`, `src/cli/config.rs`, `tests/cli_binary.rs`, `src/process/cluster.rs`, and `src/pginfo/worker.rs`, where tests keep rewriting the same baseline config sections with only one or two targeted overrides.

That duplication is expensive in both total line count and maintenance cost. It also violates the task's reduction goal because the code already has an existing shared home for runtime-config test support in `src/dev_support/runtime_config.rs` and the `crates/pgtuskmaster_test_support` re-export surface.

### Current overlap already verified

- `src/dev_support/runtime_config.rs` already owns shared config validation helpers:
  - `validate_runtime_config_contents`
  - `validate_operator_config_contents`
  - `RuntimeConfigBuilder`
- `crates/pgtuskmaster_test_support/src/lib.rs` already re-exports that runtime-config support for integration-style tests, so there is no need to invent a second test-support module.
- The raw TOML documents repeat the same stable sections again and again:
  - runtime cluster identity
  - postgres role/password blocks
  - dcs endpoint blocks
  - process binary override blocks
  - API auth/TLS token blocks
- The HA harness and operator observer use the same token and TLS path concepts but currently keep their own string-formatting helpers instead of reusing a shared renderer.

### Execution plan

1. Extend `src/dev_support/runtime_config.rs` with shared test-only config renderers and temp-file helpers.
   - Add one shared runtime-document renderer for the common baseline runtime TOML used by parser, CLI, process, pginfo, and HA tests.
   - Add one shared operator-document renderer for the observer/CLI/operator-config tests.
   - Keep the API small and override-driven so callers specify only the fields that differ from the baseline fixture.
   - Reuse existing helpers and types where possible; do not add a second builder module or a new crate.

2. Collapse the HA harness onto the shared runtime/operator rendering helpers.
   - Replace `tests/ha/support/world/mod.rs::render_member_runtime_config` with a call into the shared runtime renderer plus the few member-specific overrides.
   - Replace `tests/ha/support/observer/pgtm.rs::build_host_observer_config`, `path_source`, and `toml_string` with the shared operator renderer or shared TOML path helper.
   - Keep validation in place after rendering so the generated configs are still parse-checked before being written.

3. Remove the repeated raw runtime TOML literals from focused unit/integration tests.
   - Collapse the repeated runtime config writers in:
     - `src/config_v2/parser/load_config.rs`
     - `src/process/cluster.rs`
     - `src/pginfo/worker.rs`
     - `tests/cli_binary.rs`
   - Each test should start from the shared baseline renderer and apply only its scenario-specific override, rather than pasting a full document.

4. Remove the repeated raw operator TOML literals from operator-facing tests.
   - Collapse the duplicated operator config writers in:
     - `src/config_v2/parser/load_operator_config.rs`
     - `src/cli/config.rs`
     - `tests/ha/support/observer/pgtm.rs`
   - Reuse the same shared operator renderer and path/source formatting helpers so TLS/token sections are defined once.

5. Clean up the now-redundant local helpers and verify the line reduction.
   - Delete obsolete `write_temp_config`, `build_host_observer_config`, `path_source`, `toml_string`, and local raw-config helper functions that become pure pass-throughs.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff moves downward.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not change runtime parsing semantics in this pass; this is a fixture/rendering reduction, not a config format change.
- Do not introduce a new standalone fixture crate or a parallel builder hierarchy. The reduction must merge onto `dev_support::runtime_config` and the existing `pgtuskmaster_test_support` re-export.
- If execution shows the shared renderer API cannot cover both runtime and operator cases without growing into a larger design change, change this plan back to `TO BE VERIFIED`, document the missing shape precisely, and stop immediately.

NOW EXECUTE
