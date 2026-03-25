## Plan: Collapse Worker Startup Boundaries

### Why this reduction target

The next clear reduction seam is the bootstrap boundary around runtime workers. Several domains still split construction across thin `startup.rs` wrappers and test-only helper functions even though the owning runtime types already exist elsewhere. That spreads the same initialization knowledge across runtime, tests, and support code, which is both a boundary smell and a direct line-count problem.

This is especially visible in `process` and `ha`, where the exact same state-channel, control-plane, cadence, and runtime wiring is rebuilt in more than one place. `api` and `pginfo` are even thinner: their `startup.rs` modules mostly exist to wrap types that already live in `worker.rs` or `state.rs`.

### Current overlap already verified

- `src/process/startup.rs` constructs `ProcessWorkerCtx`, `ProcessStateChannel`, `ProcessControlPlane`, `ProcessRuntime`, and `ProcessControlHandle`.
- `src/process/worker.rs::build_test_ctx` rebuilds the same `ProcessWorkerCtx` shape with only test-specific runner, paths, and channels changed.
- `src/logging/postgres_ingest.rs::build_process_worker_ctx` rebuilds the same `ProcessWorkerCtx` shape again for integration-style tests.
- `src/ha/startup.rs` constructs `HaRuntimeCtx` and `HaStateChannel`.
- `src/ha/worker.rs::ha_context` rebuilds the same `HaRuntimeCtx` shape for tests.
- `src/pginfo/startup.rs` is a thin wrapper around `PgInfoWorkerCtx`, `PgInfoStateChannel`, and `new_state_channel`.
- `src/api/startup.rs` is a thin wrapper around `ApiRuntimeCtx` construction, and `src/dev_support/api.rs` already proves the real owner is the worker-layer type, not a separate startup boundary.
- `src/runtime/node.rs` is the only production caller for most of these startup wrappers, so the current split is mostly file and helper overhead rather than a meaningful abstraction.

### Execution plan

1. Move bootstrap ownership onto the existing runtime types instead of the thin startup modules.
   - Add domain-owned constructors or bundle builders on the existing `ProcessWorkerCtx` / `ProcessRuntimeBundle`, `HaRuntimeCtx` / `HaRuntimeBundle`, `PgInfoWorkerCtx` / `PgInfoRuntimeBundle`, and `ApiRuntimeCtx`.
   - Keep those constructors in the modules that already own the runtime shapes (`state.rs` or `worker.rs`), not in new helper modules.
   - Reuse existing types; do not introduce a new generic startup framework.

2. Collapse duplicated test builders onto the same constructors.
   - Refactor `src/process/worker.rs::build_test_ctx` and `src/logging/postgres_ingest.rs::build_process_worker_ctx` to call the shared `process` bootstrap surface with injected runtime pieces instead of rebuilding `ProcessWorkerCtx` manually.
   - Refactor `src/ha/worker.rs::ha_context` to call the shared `ha` bootstrap surface with test-specific clock, observed state, and control-plane inputs.
   - Keep test injection explicit, but make the shared constructor own channel creation and initial-state wiring once.

3. Remove now-redundant startup wrappers and module plumbing where the boundary becomes empty.
   - Delete the pass-through `run` wrappers that only forward to `worker::run`.
   - If `api`, `pginfo`, `process`, or `ha` `startup.rs` become pure forwarding shells after the constructor move, delete those files and update `mod.rs`, `runtime/node.rs`, and support/test callers to use the owning modules directly.
   - Prefer deleting files and private helper functions over moving the same logic sideways.

4. Validate the reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff goes further downward.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Tick the task boxes only after the gates pass.

### Guardrails

- Do not genericize worker bootstrap across unrelated domains; that is likely to add traits and more code than it removes.
- Keep production semantics unchanged. This pass is about moving ownership and deleting duplicate construction, not changing worker behavior.
- If the shared constructor surface cannot support both production bootstrap and test injection without introducing a wider design change, switch this plan back to `TO BE VERIFIED`, document the exact missing shape, and stop immediately.

NOW EXECUTE
