## Plan: Collapse Local Runtime Test Config Wrapper Stack Onto Config Owners

### Why this is the next verified reduction target

Plan `93` targeted the parser-exported TOML helper corridor, but that corridor is already gone in the current tree. The live boundary smell is now smaller and more local:

- `src/config_v2/parser/load_config.rs` still owns the two surviving typed test-config owners:
  - `load_runtime_test_config_with_hba_and_sections(...)`
  - `runtime_test_config_with_data_dir(...)`
- Several unrelated module test suites still add one more local wrapper layer on top of those owners, even though the wrappers only hard-code a tiny bundle of arguments and the call sites immediately mutate the returned `RuntimeConfigV2`.

That is the wrong boundary. The config owner should stay in `config_v2`; module-local pass-through wrappers above it are just courier code.

### Current overlap already verified

1. `src/postgres_managed.rs` still keeps a pure preset wrapper.
   - `managed_test_config(data_dir)` only calls `load_runtime_test_config_with_hba_and_sections(...)` with one fixed scope, one fixed HBA string, and one fixed extra-section bundle.
   - Many call sites immediately mutate the returned config further, for example reserved-GUC, TLS-path, and render-focused tests.

2. `src/logging/postgres_ingest.rs` still keeps a local trace wrapper.
   - `trace_test_config()` only calls `load_runtime_test_config_with_hba_and_sections(...)` with fixed logging-oriented HBA and section constants.
   - Its real-test callers immediately overwrite log directories, runtime log file paths, process binaries, timeouts, and port-related Postgres fields.

3. `src/logging/core/runtime.rs` keeps another one-file variant of the same wrapper shape.
   - `trace_test_config()` again only exists to inject one HBA string and one section bundle.
   - The three callers then only toggle sink booleans and one file path, proving the helper is just an extra preset layer.

4. `src/process/cluster.rs` still keeps thin runtime-test couriers.
   - `test_cfg(data_dir)` only forwards to `runtime_test_config_with_data_dir(...)`.
   - `test_cfg_for_case(case)` only adds `unique_test_dir(...).join("data")` on top of that wrapper.
   - These helpers add names, not ownership.

### Execution plan

1. Delete pure pass-through wrapper functions in the touched test modules.
   - Inline `managed_test_config(...)` call sites in `src/postgres_managed.rs` onto direct `load_runtime_test_config_with_hba_and_sections(...)` usage.
   - Inline the `trace_test_config()` helpers in:
     - `src/logging/postgres_ingest.rs`
     - `src/logging/core/runtime.rs`
   - Inline `test_cfg(...)` and `test_cfg_for_case(...)` in `src/process/cluster.rs` where that removes more code than it adds.

2. Keep the real shared owners and only the real shared owners.
   - Keep `load_runtime_test_config_with_hba_and_sections(...)`.
   - Keep `runtime_test_config_with_data_dir(...)` if, after step 1, it still clearly buys net reduction across the crate.
   - Do not introduce a new builder, preset enum, patch struct, or replacement helper module.

3. Keep shared literals as data, not wrappers.
   - Reuse the existing `*_HBA` and `*_SECTIONS` constants where they still remove duplication.
   - Delete helper functions that only pipe those constants into existing config-v2 owners.

4. Remove any now-dead imports and cfg-gated re-exports.
   - If this cleanup makes a `config_v2` re-export unused, remove it.
   - Otherwise keep the public surface unchanged in this slice.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_current_lines.sh` and require total lines to fall below the current `34027`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require improvement beyond the current `+10312 -14360 diff: -4048`.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not recreate a second layer of local runtime-config wrappers under new names.
- Keep production behavior unchanged; this is a test-fixture ownership cleanup.
- If direct inlining starts repeating the same non-trivial field rewrite bundle more than the deleted wrappers saved, switch this plan back to `TO BE VERIFIED` instead of forcing a lossy collapse.
- Reuse existing config types and loader entrypoints; do not add new structs or enums for this slice.

### Expected yield

- Delete several one-file runtime-config wrapper functions.
- Reduce local indirection above the surviving `config_v2` test-config owners.
- Improve the story’s net line reduction without widening config boundaries again.

### Verification note after opening the call sites

This plan is not safe to execute as written:

- `managed_test_config(...)` in `src/postgres_managed.rs` fans out to 9 call sites.
- `trace_test_config()` in `src/logging/postgres_ingest.rs` fans out to 3 call sites.
- `trace_test_config()` in `src/logging/core/runtime.rs` fans out to 3 call sites.
- `test_cfg(...)` / `test_cfg_for_case(...)` in `src/process/cluster.rs` fan out to 7 call sites.

Inlining those helpers would remove small wrapper bodies, but it would duplicate the longer shared-loader invocation and repeated `map_err(...to_string())` conversion across many callers. That is the opposite of this story's line-reduction goal, so this design needs a new verified reduction target before execution resumes.

TO BE VERIFIED
