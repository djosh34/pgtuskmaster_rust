## Plan: Collapse HA Test Config Renderers Onto Config-V2 Test Support

### Why this is the next verified reduction target

The last attached plan is already implemented and committed in `3e28fbc0` and later follow-up slices, so keeping `05-reduce-loop` pointed at it leaves the loop in the wrong state.

The next live boundary problem is still the same class of waste: config-v2 parsing already owns the canonical test-document shape, but the HA harness keeps rebuilding runtime and operator TOML by hand in `tests/ha/support/`.

- `tests/ha/support/givens/mod.rs:208-340` duplicates TOML quoting helpers plus a full runtime config renderer for HA fixture members.
- `tests/ha/support/observer/pgtm.rs:440-487` duplicates a second operator-config renderer plus the same TOML quoting/path helper logic.
- `src/config_v2/parser/load_config.rs:24-188` already owns the reusable runtime test renderer and TOML assembly primitives under `#[cfg(any(test, feature = "internal-test-support"))]`.
- `src/config_v2/parser/load_config.rs:1383-1431` already contains an operator-config TOML renderer inside `mod tests`, which means the parser owns the correct shape but the HA harness cannot reuse it.

### Current overlap verified in code

- `tests/ha/support/givens/mod.rs` repeats:
  - `toml_string`
  - `toml_path_source`
  - `toml_string_secret`
  - a long `render_ha_member_runtime_test_config_toml` document that rebuilds cluster identity, postgres paths, role auth blocks, DCS endpoints, process binary overrides, API config, and `pgtm` sections by raw string assembly.
- `tests/ha/support/observer/pgtm.rs` repeats:
  - `toml_string`
  - `toml_path_source`
  - a hand-written operator document in `build_host_observer_config`.
- `src/config_v2/parser/load_config.rs` splits the parser-owned runtime fixture story across `render_runtime_test_config_toml` and `render_runtime_fixture_toml`, so there is still an opportunity to merge the runtime test renderers before routing the HA harness through them.

### Execution plan

1. Promote parser-owned TOML rendering primitives into internal test support.
   - Move the generic operator-config renderer currently inside `src/config_v2/parser/load_config.rs` test-only code into the main `#[cfg(any(test, feature = "internal-test-support"))]` support section.
   - Expose the small generic TOML helpers that are already duplicated elsewhere: `toml_string`, `toml_path_source`, `toml_string_secret`, and `join_rendered_sections`.
   - Re-export only the helpers actually needed by the HA harness from `src/config_v2/parser/mod.rs` and `src/config_v2/mod.rs`.

2. Collapse the parser-side runtime test renderers into one canonical builder.
   - Merge `render_runtime_fixture_toml` into the existing parser-owned runtime renderer instead of keeping two overlapping runtime-document assembly paths.
   - Extend the canonical runtime renderer to accept the few fields that actually vary across current users:
     - cluster identity
     - postgres path triple
     - DCS endpoint list
     - role usernames/password literals
     - HBA/ident contents
     - appended extra sections
   - Update the existing parser test-support helpers (`runtime_test_config`, `runtime_test_config_with_data_dir`, `managed_postgres_test_config`, `trace_logging_test_config`) to call that one builder.

3. Route HA given runtime documents through the canonical renderer.
   - Rewrite `tests/ha/support/givens/mod.rs::render_ha_member_runtime_test_config_toml` so it builds from the shared config-v2 runtime renderer plus HA-specific extra sections, instead of carrying its own full-document template.
   - Keep only HA-specific values local to the harness:
     - container binary override paths
     - TLS identity/client-auth blocks
     - HA-specific HBA/ident contents
     - logging/API/`pgtm` sections
   - Delete the local duplicated TOML helper functions from the givens module.

4. Route HA observer operator documents through the canonical renderer.
   - Replace `tests/ha/support/observer/pgtm.rs::build_host_observer_config` string assembly with the promoted operator-config renderer and shared TOML helper(s).
   - Delete the local `toml_string` and `toml_path_source` helpers from the observer module.

5. Remove post-render config rewriting where the renderer can own it directly.
   - Rework `validate_runtime_config_for_host` so host validation gets its own parser-backed binary override input instead of rendering the container config and then applying four string replacements.
   - Keep the materialized HA fixture runtime files unchanged for the containers; only the host-side validation path should swap in `/bin/true`.

6. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff is net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not add a second HA-specific config schema or a new intermediate config type.
- Prefer changing the existing shared renderer signatures over introducing a parallel renderer with almost the same fields.
- Keep parser-owned TOML shape knowledge inside `src/config_v2/parser/`; the HA harness should provide values, not rebuild the schema.
- Preserve the current rendered fixture semantics for HA tests: custom role names, TLS paths, DCS endpoint selection, and operator auth/TLS fields must stay equivalent.

### Expected yield

- Delete the duplicated TOML helper functions from `tests/ha/support/givens/mod.rs` and `tests/ha/support/observer/pgtm.rs`.
- Delete the bespoke operator-config renderer from `tests/ha/support/observer/pgtm.rs`.
- Shrink or delete the bespoke runtime document renderer in `tests/ha/support/givens/mod.rs`.
- Merge overlapping parser-side runtime test renderers so config-v2 test support owns a single canonical runtime TOML builder.
- Keep the slice net-negative in lines while improving the boundary: parser test support owns config document assembly, HA harness code supplies fixture-specific values.

NOW EXECUTE
