## Plan: Delete Internal Runtime Test Renderer

### Why this is the next reduction target

The public config test-renderer corridor is already gone, but `src/config_v2/parser/private_schema.rs` still owns an internal TOML renderer that no production loader needs anymore.

- `rg -n "render_runtime_test_config_toml|build_runtime_test_document|trim_runtime_test_document|join_rendered_sections" src` shows the runtime test renderer stack now lives only in:
  - `src/config_v2/parser/private_schema.rs`
  - `src/config_v2/parser/load_config.rs`
- `rg -n "raw::render_runtime_test_config_toml" src/config_v2/parser/load_config.rs` shows every remaining caller is inside `load_config.rs`:
  - `load_runtime_test_config_from_paths(...)`
  - parser test cases inside `#[cfg(test)] mod tests`
- `load_runtime_test_config_from_paths(...)` currently serializes a `RuntimeDocument` to TOML, parses it back through `load_runtime_config_contents(...)`, then mutates the parsed runtime passwords. That is an internal render-then-parse loop inside the same parser module.

That means parser-private schema knowledge is still being used as a courier between parser-owned test helpers and parser-owned typed config constructors. The boundary is wrong: `private_schema.rs` should own parse/normalize shapes, while `load_config.rs` should own typed test configs and its own parser-fixture strings.

### Current overlap already verified

- `src/config_v2/parser/private_schema.rs:719-867` still carries:
  - `trim_runtime_test_document(...)`
  - `join_rendered_sections(...)`
  - `build_runtime_test_document(...)`
  - `render_runtime_test_config_toml(...)`
- `src/config_v2/parser/load_config.rs:258-278` uses that renderer only to build `runtime_test_config*` helpers.
- `src/config_v2/parser/load_config.rs:1035-1443` already owns local TOML helper functions for operator parser tests, so the runtime parser tests can own their runtime fixture rendering there too.

### Execution plan

1. Make `load_config.rs` own typed runtime test defaults directly.
   - Replace `load_runtime_test_config_from_paths(...)` with a direct `RuntimeConfigV2` constructor path instead of serializing TOML and parsing it back.
   - Reuse the existing config_v2 types already returned from the loader; do not introduce new wrapper structs or DTOs.
   - Keep the resulting runtime config semantics the same for the worker, process, runtime, and api tests that call `runtime_test_config*`.

2. Localize runtime parser fixture rendering to `load_config.rs` tests.
   - Add a test-local runtime TOML renderer inside `#[cfg(test)] mod tests`, next to the existing operator fixture helpers.
   - Update parser tests to call that local helper instead of `raw::render_runtime_test_config_toml(...)`.
   - Keep the fixture TOML minimal and owned by the parser tests that actually parse it.

3. Delete the now-dead renderer corridor from `private_schema.rs`.
   - Remove `trim_runtime_test_document(...)`, `join_rendered_sections(...)`, `build_runtime_test_document(...)`, and `render_runtime_test_config_toml(...)`.
   - Remove any imports and helper code that become dead once those functions are gone.

4. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to stay net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not recreate a shared runtime TOML renderer somewhere else in `src/`.
- Keep parser-fixture rendering private to the parser tests and keep typed runtime test defaults private to `load_config.rs`.
- If direct typed config construction starts duplicating too much loader normalization logic, switch this plan back to `TO BE VERIFIED` instead of rebuilding another hidden conversion corridor.

### Expected yield

- Delete the remaining internal runtime test renderer stack from `src/config_v2/parser/private_schema.rs`.
- Stop `load_runtime_test_config_from_paths(...)` from doing an internal render/parse round-trip.
- Keep parser test fixtures owned by the parser tests instead of by raw schema internals.

### Status

This slice has been executed.

- `load_runtime_test_config_from_paths(...)` now builds `RuntimeConfigV2` directly in `load_config.rs`.
- Runtime parser fixture rendering now lives only in `load_config.rs` tests.
- `src/config_v2/parser/private_schema.rs` no longer carries the internal runtime test renderer corridor.
- Validation passed for this slice with:
  - `make check`
  - `make lint`
  - `make test`
  - `make test-long`
- Net line delta remains negative after this slice:
  - `bash .ralph/git_diff_lines_since.sh` => `diff: -3609`
- Repository total is still above the story target:
  - `bash .ralph/git_current_lines.sh` => `total: 34466`

The next reduce-loop turn should pick a new reduction target and replace this plan path.

TO BE VERIFIED
