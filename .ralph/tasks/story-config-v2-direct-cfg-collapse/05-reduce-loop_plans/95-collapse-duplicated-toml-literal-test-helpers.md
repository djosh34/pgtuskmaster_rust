## Plan: Collapse Duplicated TOML Literal Test Helpers Onto Config-V2 Test Support

### Why this is the next verified reduction target

Plan `94` is correctly back in `TO BE VERIFIED`: deleting the local runtime-config wrapper helpers would duplicate longer loader boilerplate at many call sites and lose lines overall.

The live overlap is smaller and cleaner:

- `src/config_v2/parser/load_config.rs` still owns cfg-gated TOML literal helpers:
  - `toml_string(...)`
  - `toml_path_source(...)`
  - `toml_string_secret(...)`
- Three test owners reimplement the same logic locally:
  - `tests/ha/support/givens/mod.rs`
  - `tests/ha/support/observer/pgtm.rs`
  - `tests/cli_binary.rs`
- The crate already exposes a clean cfg-gated sharing boundary through `internal-test-support` and `crates/pgtuskmaster_test_support`, so this reuse does not need to widen the normal production API.

This is a better fit for the reduction loop: one owner already exists, four files currently duplicate the same encoding behavior, and the replacement is smaller than the repeated local helpers.

### Current overlap verified in code

1. `src/config_v2/parser/load_config.rs` defines the canonical helpers already used by config-v2 test rendering.
   - `toml_string(...)`
   - `toml_path_source(...)`
   - `toml_string_secret(...)`

2. `tests/ha/support/givens/mod.rs` duplicates all three helpers.
   - `toml_string(...)`
   - `toml_path_source(...)`
   - `toml_string_secret(...)`

3. `tests/ha/support/observer/pgtm.rs` duplicates two of the same helpers.
   - `toml_string(...)`
   - `toml_path_source(...)`

4. `tests/cli_binary.rs` duplicates `toml_string(...)`.

That is nine local helper definitions for three tiny behaviors.

### Execution plan

1. Create one cfg-gated `config_v2` test-support owner for TOML literal encoding.
   - Move the shared implementations into a dedicated module under `src/config_v2/` such as `src/config_v2/test_support.rs`, compiled only for `#[cfg(any(test, feature = "internal-test-support"))]`.
   - Keep the surface narrow:
     - `toml_string(...)`
     - `toml_path_source(...)`
     - `toml_string_secret(...)`
   - Do not add a generic fixture builder, builder struct, or new document type.

2. Route config-v2 parser test support through that owner.
   - Update `src/config_v2/parser/load_config.rs` to import the shared helpers instead of defining them locally.
   - Keep `render_runtime_test_config_toml(...)` and the typed loader helpers where they are; this slice only removes duplicate literal encoders.

3. Route the integration-test owners through the same boundary.
   - Update `tests/ha/support/givens/mod.rs` to use the shared helper module and delete its local copies.
   - Update `tests/ha/support/observer/pgtm.rs` to use the shared helper module and delete its local copies.
   - Update `tests/cli_binary.rs` to use the shared helper module and delete its local `toml_string(...)`.
   - Prefer importing through `pgtuskmaster_test_support::config_v2::test_support::...` where the test is already outside the crate boundary.

4. Keep ownership strict.
   - Do not re-open the old broad public renderer corridor from earlier plans.
   - Keep the helpers cfg-gated to test support only.
   - If any caller needs more than these three literal encoders, stop and re-verify instead of growing another generic rendering API.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_current_lines.sh` and require total lines to fall below the current `34027`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to improve beyond the current baseline.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse existing config-v2 test-support ownership; do not create a second helper module in `tests/`.
- Do not widen the normal production API just to help integration tests.
- Keep the large handwritten fixture renderers local for now unless they fall out naturally from this cleanup; this plan is about deleting repeated literal encoders, not forcing another broad renderer abstraction.
- If the shared helper exports end up costing more lines than the deleted duplicates, switch this plan back to `TO BE VERIFIED`.

### Expected yield

- Delete duplicated `toml_string(...)` helpers from four files down to one owner.
- Delete duplicated `toml_path_source(...)` helpers from three files down to one owner.
- Delete duplicated `toml_string_secret(...)` helpers from two files down to one owner.
- Shrink the cfg-gated config test-support surface without reintroducing the larger stale renderer plans.

NOW EXECUTE
