## Plan: Collapse Worker Bootstrap Boundaries

### Why this reduction target

The last slice removed a duplicate runtime-config builder, but another wrong boundary still survives in the worker startup path:

- `src/runtime/node.rs` already owns the full node wiring order and is the only production caller that assembles all worker dependencies.
- `src/pginfo/state.rs`, `src/process/state.rs`, and `src/ha/state.rs` still mix pure state types with worker bootstrap constructors that allocate channels, choose clocks, and build runtime/control-plane pieces.
- `src/dcs/mod.rs` is mostly a startup courier that allocates a state channel, creates a command inbox, and forwards everything into `worker::DcsWorker::new`.
- `src/api/worker.rs` has a separate `bootstrap` helper even though it is another one-caller runtime assembly step.

That ownership is split the wrong way around. State modules should describe state, not worker startup assembly. The current shape spreads one workflow across `runtime/node.rs`, several `state.rs` files, one thin `mod.rs`, and a few test-only bootstrap variants.

### Current overlap already verified

- `src/runtime/node.rs` is the only production place that calls `PgInfoWorkerCtx::bootstrap`, `dcs::bootstrap`, `ProcessWorkerCtx::bootstrap`, `HaRuntimeCtx::bootstrap`, and `api::worker::bootstrap`.
- `src/process/state.rs` owns both state definitions and two assembly helpers (`bootstrap`, `bootstrap_with_runtime`) even though the latter exists mainly to serve `src/process/worker.rs` tests.
- `src/ha/state.rs` has the same split with `bootstrap` and `bootstrap_with_now`, and `src/ha/worker.rs` tests are the only extra caller for the custom clock variant.
- `src/pginfo/state.rs` owns a one-caller `bootstrap` that only allocates a state channel and wraps `LogSender`.
- `src/dcs/mod.rs` is a thin bootstrap wrapper that allocates the starting snapshot plus command channel and then immediately constructs `worker::DcsWorker`.

This is the "remove the damn helpers" and "wrong place-ism" smell together: setup knowledge lives outside the worker owners, and several helpers only exist to hide local struct construction.

### Execution plan

1. Move startup assembly next to the worker owners.
   - Add worker-owned constructors in `src/{pginfo,process,ha,api}/worker.rs` and `src/dcs/worker.rs` for the runtime/control/channel assembly that is currently split across `state.rs` and `mod.rs`.
   - Keep the existing state types; do not add new startup DTOs or replacement wrapper layers.

2. Reduce `state.rs` modules back to state ownership only.
   - Delete `PgInfoWorkerCtx::bootstrap` from `src/pginfo/state.rs`.
   - Delete `ProcessWorkerCtx::{bootstrap,bootstrap_with_runtime}` from `src/process/state.rs`.
   - Delete `HaRuntimeCtx::{bootstrap,bootstrap_with_now}` from `src/ha/state.rs`.
   - Leave those files focused on state enums/structs plus genuinely state-local helpers.

3. Retarget runtime wiring and tests directly onto the worker-owned constructors.
   - Update `src/runtime/node.rs` to call the new worker-owned constructors.
   - Update `src/process/worker.rs` and `src/ha/worker.rs` tests to use the worker-owned test/runtime constructors or direct local struct construction instead of reaching back into `state.rs`.
   - Collapse `src/dcs/mod.rs` if it becomes a trivial re-export shell after removing its bootstrap wrapper.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+3917 -6209 diff: -2292` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Tick the task boxes only after the gates pass.

### Guardrails

- Prefer deleting bootstrap helpers over moving them unchanged.
- Keep worker runtime assembly with the worker that owns the runtime context.
- Keep `state.rs` focused on durable state shapes and state-local behavior, not clocks, channels, or command wiring.
- Reuse the existing context/state types; do not introduce new builder structs, startup enums, or parallel helper layers.
- If the refactor starts growing replacement bootstrap wrappers instead of deleting them, switch this plan back to `TO BE VERIFIED`, document the mismatch, retarget the task file, and stop immediately.

NOW EXECUTE
