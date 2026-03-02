---
## Task: Implement core ids time errors and typed watch channels <status>not_started</status> <passes>false</passes> <priority>ultra_high</priority>

<description>
**Goal:** Build the foundational shared types and state-channel primitives used by every worker.

**Scope:**
- Create `src/state/ids.rs`, `src/state/time.rs`, `src/state/errors.rs`, `src/state/watch_state.rs`, and `src/state/mod.rs`.
- Implement `MemberId`, `ClusterName`, `SwitchoverRequestId`, `JobId`, `WalLsn`, `TimelineId`, `UnixMillis`, `Version`, `WorkerStatus`, and `Versioned<T>`.
- Implement `StatePublisher<T>`, `StateSubscriber<T>`, `new_state_channel`, `publish`, `latest`, and `changed`.

**Context from research:**
- `RUST_SYSTEM_HARNESS_PLAN.md` defines this as Build Order step 1.
- All worker loops depend on typed `watch` channels with versioned updates.

**Expected outcome:**
- Shared primitives compile and are ready for worker wiring and deterministic tests.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] `src/state/*` exists with the exact core types and channel wrappers from plan signatures.
- [ ] `publish()` increments `Version` and stamps `updated_at`.
- [ ] `changed()` returns latest `Versioned<T>` and propagates recv errors as typed errors.
- [ ] Unit tests verify version increments, initial snapshot, and changed notifications.
- [ ] Run targeted tests for state/watch module.
- [ ] Run `make check` and capture result.
- [ ] Run `make test` and capture result.
- [ ] Run `make lint` and capture result.
- [ ] Run `make test-bdd` and capture result.
- [ ] If any command fails, use `$add-bug` skill and create bug task(s) in `.ralph/tasks/bugs/` with repro command and logs.
</acceptance_criteria>
