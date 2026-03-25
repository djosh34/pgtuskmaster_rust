## Plan: Delete Runtime Test Config Wrapper Helpers

### Why this is the next reduction target

`config_v2` already owns the validated runtime test-config entrypoints:

- `runtime_test_config(...)`
- `runtime_test_config_with_data_dir(...)`
- `managed_postgres_test_config(...)`
- `trace_logging_test_config(...)`

But several modules still sit one layer above those owners and keep local wrapper helpers that only rebuild the same `RuntimeConfigV2` through a small set of direct field rewrites. That boundary is wrong: the shared config owners already exist, while the local wrappers mostly act as pass-through couriers.

### Current overlap already verified

- `src/api/worker.rs:505-566` still carries three near-duplicate helpers:
  - `sample_https_runtime_config(...)`
  - `sample_invalid_https_runtime_config(...)`
  - `sample_http_runtime_config(...)`
  - All three start from `runtime_test_config_with_data_dir(...)`, re-derive the same Postgres path fields, apply the same API auth, and then differ only in transport setup.
- `src/dev_support/api.rs:48-58` still carries `build_test_runtime_config(...)`, which only calls `runtime_test_config(...)` and swaps `api.auth`.
- `src/postgres_managed.rs:697-721` still carries:
  - `sample_managed_config(...)`, a pass-through wrapper over `managed_postgres_test_config(...)`
  - `sample_render_config(...)`, which only applies a small render-specific Postgres field override bundle.
- `src/postgres_roles.rs:125-137` still carries `sample_cfg(...)`, which only calls `runtime_test_config(...)` and swaps the three mandatory role credentials.
- `src/process/cluster.rs:536-555` still carries `runtime_config_v2_with_source_ca(...)`, which only starts from `runtime_test_config_with_data_dir(...)` and then applies source TLS plus `/bin/true` binary overrides for one test corridor.

These wrappers are not new reusable types, schemas, or fixtures. They are local indirection layers above already-owned config entrypoints.

### Execution plan

1. Delete pure pass-through config wrappers first.
   - Remove `sample_managed_config(...)` and replace its call sites with direct `managed_postgres_test_config(...)` usage.
   - Remove `build_test_runtime_config(...)` and apply the same `api.auth` rewrite directly inside the two router builders in `src/dev_support/api.rs`.
   - Remove `sample_cfg(...)` and build the role-adjusted config directly inside the single test that needs it.

2. Collapse the API worker test helpers onto the existing config owner.
   - In `src/api/worker.rs`, stop preserving three separate `sample_*_runtime_config(...)` helpers.
   - Rebuild those tests from `runtime_test_config_with_data_dir(...)` plus direct local rewrites, or keep at most one narrow shared helper if it demonstrably deletes more code than it adds.
   - Do not recreate another shared test-config layer under a different name.

3. Inline one-off mutation wrappers where the call graph is already tiny.
   - Replace `runtime_config_v2_with_source_ca(...)` in `src/process/cluster.rs` with direct setup near the test that uses it.
   - Replace `sample_render_config(...)` in `src/postgres_managed.rs` with direct render-focused config construction at the relevant tests instead of keeping a dedicated wrapper.

4. Keep the remaining shared state in its real owner.
   - Reuse `runtime_test_config(...)`, `runtime_test_config_with_data_dir(...)`, `managed_postgres_test_config(...)`, `trace_logging_test_config(...)`, and `api_auth_from_optional_tokens(...)`.
   - Prefer direct field updates at the final test call site over introducing new builders, patch structs, or helper modules.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to stay net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not invent a new `RuntimeConfigV2` builder, patch type, or helper module for this slice.
- Keep production behavior unchanged; this is a config-fixture ownership cleanup.
- If direct inlining starts repeating the same non-trivial field bundle across multiple touched files, switch this plan back to `TO BE VERIFIED` instead of sneaking in a second shared abstraction layer.

### Expected yield

- Delete multiple local wrapper helpers across `src/api/worker.rs`, `src/dev_support/api.rs`, `src/postgres_managed.rs`, `src/postgres_roles.rs`, and `src/process/cluster.rs`.
- Reduce one more layer of indirection above the config-owned test helpers.
- Shrink repeated `RuntimeConfigV2` wrapper code without widening any config surface.

NOW EXECUTE
