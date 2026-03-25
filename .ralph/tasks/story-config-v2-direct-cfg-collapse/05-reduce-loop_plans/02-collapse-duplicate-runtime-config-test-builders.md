## Plan: Collapse Duplicate Runtime Config Test Builders

### Why this reduction target

The codebase already has a reusable validated fixture builder in [`src/dev_support/runtime_config.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dev_support/runtime_config.rs), but several test modules still keep local wrappers that rebuild the same `RuntimeConfigV2` shape:

- [`src/postgres_managed.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs)
- [`src/process/planner.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/planner.rs)
- [`src/process/cluster.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/cluster.rs)
- [`src/process/tools.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/tools.rs)
- [`src/process/session.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/session.rs)
- [`src/logging/postgres_ingest.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/postgres_ingest.rs)
- [`src/logging/core/runtime.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/core/runtime.rs)

Some of those wrappers are only a few lines, but together they keep duplicating the same sample-config knowledge in multiple places. That makes future config refactors fan out across unrelated test modules and works directly against the reduction task's requirement to reuse existing shapes instead of rebuilding them.

### Current overlap already verified

- `RuntimeConfigBuilder` already provides the overrides these tests need:
  - `with_postgres_data_dir`
  - `with_dcs_scope`
  - `with_timing`
  - `with_binaries`
  - `with_pg_hba_contents`
  - `with_logging`
- The logging tests already reuse `sample_logging_config`, which means there is no missing shared fixture type to invent first.
- The process tests in planner/cluster/tools/session all currently wrap the exact same `RuntimeConfigBuilder::new().with_postgres_data_dir(...).build()` pattern.

### Execution plan

1. Delete the local `sample_runtime_config` or `sample_runtime` wrappers that are pure pass-throughs to `RuntimeConfigBuilder`.
   - In planner/cluster/tools/session, replace each helper callsite with direct `RuntimeConfigBuilder` usage.
   - Keep the imports local and minimal; do not add new helper structs or new fixture modules for this pass.

2. Inline or directly reuse the richer builder chain in the larger test modules.
   - In `postgres_managed` tests, replace the local helper with direct `RuntimeConfigBuilder` construction at the callsites that need the timing, scope, binary, and data-dir overrides.
   - In `logging/postgres_ingest` and `logging/core/runtime`, remove the local runtime-config helper and construct the config directly from `RuntimeConfigBuilder` plus `sample_logging_config`.

3. Prefer reuse over new abstraction.
   - Only extend `RuntimeConfigBuilder` if execution exposes a repeated override pattern that appears in at least two touched modules and cannot be expressed clearly with the existing methods.
   - Do not create a second builder, a fixture DTO, or a `sample_test_runtime_config` module. The existing builder is the shared state to reuse.

4. Clean up imports after the helper removals.
   - Drop now-unused `PathBuf`, `Duration`, or builder imports where the local helper disappears.
   - Keep the touched test modules compiling without extra dead-code allowances.

5. Validate the reduction and the repo gates.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the net diff moved downward after the helper collapse.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not touch production config parsing or runtime behavior in this pass; this is a fixture and test-support reduction only.
- Do not invent a new generic fixture layer. The whole point is to collapse onto the existing `RuntimeConfigBuilder`.
- If execution shows a touched test needs a config override that the builder cannot express cleanly, update this plan back to `TO BE VERIFIED`, document the exact missing override, and stop immediately.

NOW EXECUTE
