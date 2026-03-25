## Plan: Collapse Trace Logging Test-Config Leaf Overrides

### Why this is the next reduction target

`trace_logging_test_config()` already exists as the owned runtime fixture for logging-focused tests. It sets the trace log level, short postgres log poll interval, and disables postgres log cleanup. But several logging tests still sit one layer above that owner and restate the same logging-specific config edits directly in leaf test bodies.

That boundary is wrong: the logging-focused fixture owner should carry the shared logging-test defaults, while each test should only keep the truly scenario-specific overrides like unique temp paths, real binaries, or per-test ports.

### Current overlap already verified

- `src/config_v2/parser/load_config.rs:228-237` already sets `logging.postgres_log_cleanup_enabled = false` inside `trace_logging_test_config()`.
- `src/logging/postgres_ingest.rs:1494-1500` still rebuilds a trace logging config and then immediately restates:
  - `postgres.pg_hba_contents = "local ... trust\nhost ... trust\n"`
  - `logging.postgres_log_cleanup_enabled = false`
  - a per-test log path override bundle
- `src/logging/postgres_ingest.rs:1570-1598` repeats the same HBA override and the same already-owned cleanup disable before layering on real-test-specific binaries, data dir, port, and log dir.
- `src/logging/postgres_ingest.rs:1830-1834` repeats the same HBA override again for the helper-binary stderr capture test.
- `src/logging/core/runtime.rs:1093-1153` also uses `trace_logging_test_config()`, but only changes sink toggles. That is the correct boundary and should remain untouched except for any fallout from tightening the owner.

The repeated cleanup writes are already dead code today, and the repeated HBA bundle is a shared logging-test concern being reapplied in three leaf tests instead of in the owner.

### Execution plan

1. Tighten the existing logging fixture owner instead of adding a new helper.
   - Update `trace_logging_test_config()` in `src/config_v2/parser/load_config.rs` so it owns the standard logging-test HBA needed by the real logging tests: both local trust and `127.0.0.1/32` host trust.
   - Keep the existing owned logging defaults there: trace level, short poll interval, and disabled cleanup.
   - Do not add another `*_logging_test_config_with_*` helper or builder.

2. Delete leaf-level restatements in `src/logging/postgres_ingest.rs`.
   - Remove the three repeated `cfg.postgres.pg_hba_contents = ...` rewrites.
   - Remove the two repeated `cfg.logging.postgres_log_cleanup_enabled = false` rewrites that are already redundant.
   - Keep only the real per-test differences at each call site: binaries, data dir, socket dir, port, unique log dir, and explicit `extra_gucs`.

3. Collapse sibling path ownership where the test already has one authoritative logging path.
   - In the pg_ctl ingestion test, stop treating `cfg.postgres.log_file` and `cfg.logging.postgres_pg_ctl_log` as separately-authored leaf values when they represent the same file for that corridor.
   - Derive one from the other directly in the test body after choosing the unique per-test path, instead of maintaining two independent literals.

4. Keep the rest of the logging tests on the same owner.
   - Leave `src/logging/core/runtime.rs` using `trace_logging_test_config()` with only sink-specific toggles.
   - Do not widen the fixture to include scenario-specific file sink enablement or unique temp paths that only one test needs.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to stay net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not add a new logging-test config builder, patch struct, or wrapper helper for this slice.
- Keep production behavior unchanged; this is test-fixture ownership cleanup only.
- If moving the HBA default into `trace_logging_test_config()` affects non-logging tests in unrelated modules or forces scenario-specific branching into the owner, switch this plan back to `TO BE VERIFIED` instead of smuggling multiple fixture modes into one helper.

### Expected yield

- Delete repeated logging-test config rewrites from `src/logging/postgres_ingest.rs`.
- Move shared HBA ownership onto the existing `trace_logging_test_config()` owner.
- Remove already-dead cleanup toggles and reduce one more layer of leaf-level test-config churn.

NOW EXECUTE
