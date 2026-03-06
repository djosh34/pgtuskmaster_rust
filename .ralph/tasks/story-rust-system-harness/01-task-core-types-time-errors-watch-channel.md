## Task: Implement core ids time errors and typed watch channels <status>done</status> <passes>true</passes> <priority>ultra_high</priority>

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
- [x] `src/state/*` exists with the exact core types and channel wrappers from plan signatures.
- [x] `publish()` increments `Version` and stamps `updated_at`.
- [x] `changed()` returns latest `Versioned<T>` and propagates recv errors as typed errors.
- [x] Unit tests verify version increments, initial snapshot, and changed notifications.
- [x] Run targeted tests for state/watch module.
- [x] Run `make check` and capture result.
- [x] Run `make test` and capture result.
- [x] Run `make lint` and capture result.
- [x] Run `make test` and capture result.
- [x] If any command fails, use `$add-bug` skill and create bug task(s) in `.ralph/tasks/bugs/` with repro command and logs.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Baseline and crate discovery
- [x] Identify the Rust crate root where `src/` and `Cargo.toml` live (repo currently has planning/task files only at top level).
- [x] Confirm whether `src/state/` already exists; if partially present, preserve user changes and extend only missing pieces.
- [x] Capture current compile/test baseline with targeted state module tests if available.
- [x] If no crate exists, bootstrap a minimal crate at repo root (`Cargo.toml`, `src/lib.rs`) before implementing `src/state/*`; document this as an explicit prerequisite rather than assuming an existing crate.

2. Create/complete state module files
- [x] Create `src/state/mod.rs` exporting: `ids`, `time`, `errors`, `watch_state`.
- [x] Create `src/state/ids.rs` with newtype wrappers:
  - [x] `MemberId(String)`
  - [x] `ClusterName(String)`
  - [x] `SwitchoverRequestId(String)`
  - [x] `JobId(String)`
  - [x] `WalLsn(u64)`
  - [x] `TimelineId(u32)`
- [x] Create `src/state/time.rs` with:
  - [x] `UnixMillis(u64)`
  - [x] `Version(u64)`
  - [x] `Versioned<T> { version, updated_at, value }`
- [x] Create `src/state/errors.rs` with typed errors used by watch wrapper:
  - [x] `WorkerError` (minimum variant set needed by `WorkerStatus::Faulted`)
  - [x] `StatePublishError` (closed channel / send failure)
  - [x] `StateRecvError` (channel closed / receive failure)
- [x] Create `src/state/watch_state.rs` implementing typed wrapper around `tokio::sync::watch`.

3. Type semantics and trait derivations
- [x] Add derives for ergonomics and testability:
  - [x] IDs and scalar wrappers: `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`
  - [x] Numeric wrappers: also `Copy`, `PartialOrd`, `Ord`
  - [x] `Versioned<T>`: `Clone`, `Debug`, `PartialEq`, `Eq` where bounds allow
- [x] Keep fields private unless cross-module access requires `pub(crate)` methods.
- [x] Add constructor/accessor methods only where needed to avoid overexposing internals.

4. Worker status and versioned state
- [x] Implement `WorkerStatus` enum in `time.rs` or `errors.rs`-adjacent module and re-export from `state::mod`.
- [x] Include variants:
  - [x] `Starting`
  - [x] `Running`
  - [x] `Stopping`
  - [x] `Stopped`
  - [x] `Faulted(WorkerError)`
- [x] Ensure `Versioned<T>` supports:
  - [x] constructing initial snapshot with `Version(0)` and provided `UnixMillis`
  - [x] cloning latest value for publisher/subscriber reads

5. Watch channel wrapper implementation
- [x] `new_state_channel<T: Clone>(initial: T, now: UnixMillis) -> (StatePublisher<T>, StateSubscriber<T>)`
  - [x] initialize `watch::channel` with `Versioned { version: Version(0), updated_at: now, value: initial }`.
- [x] `StatePublisher<T>::publish(&self, next: T, now: UnixMillis) -> Result<Version, StatePublishError>`
  - [x] read current version from `tx.borrow()`
  - [x] increment by exactly 1 (checked/saturating decision documented in code comment)
  - [x] send new `Versioned`
  - [x] return new `Version`
  - [x] map send failure to `StatePublishError`
- [x] `StatePublisher<T>::latest(&self) -> Versioned<T>`
  - [x] clone current `tx.borrow()` snapshot
- [x] `StateSubscriber<T>::latest(&self) -> Versioned<T>`
  - [x] clone current `rx.borrow()` snapshot
- [x] `StateSubscriber<T>::changed(&mut self) -> Result<Versioned<T>, StateRecvError>`
  - [x] await `rx.changed().await`
  - [x] on success return cloned latest snapshot
  - [x] map recv failure to `StateRecvError`

6. Unit test coverage for acceptance criteria
- [x] Add tests in `src/state/watch_state.rs` or `tests/state_watch_state.rs`:
  - [x] initial snapshot has `Version(0)` and provided `updated_at`
  - [x] each `publish()` increments version monotonically by 1
  - [x] `publish()` updates timestamp to provided `now`
  - [x] subscriber `changed()` unblocks after publish and returns latest payload
  - [x] changed/recv error propagation when sender dropped
  - [x] latest() on publisher/subscriber returns same snapshot after publish
- [x] Prefer deterministic single-threaded async tests (`#[tokio::test(flavor = "current_thread")]`).

7. Verification command sequence (strict)
- [x] Run targeted tests first for fast feedback (state/watch module only).
- [x] Verify required commands exist and are wired in this repo (`make check`, `make test`, `make lint`); if missing, treat as failure and file bug task(s) instead of silently skipping.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test`.
- [x] If any failure occurs:
  - [x] collect failing command output
  - [x] use `$add-bug` skill to create bug task file(s) in `.ralph/tasks/bugs/` with repro and logs
  - [x] continue only after documenting each unresolved failure

8. Task-file bookkeeping updates during execution phase
- [x] Tick off checklist items in this task file as each item completes.
- [x] Update `<status>` to reflect progression (`in_progress` then `done`).
- [x] Set `<passes>true</passes>` only when all required commands pass.
- [x] If workflow additionally requires `<passes>true</passes>`, add that tag without removing existing `<passes>` tag.

9. Completion and handoff
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit with message format:
  - [x] `task finished 01-task-core-types-time-errors-watch-channel: <summary with evidence>`
  - [x] include key test/check evidence and any implementation challenge notes
- [x] Append learnings/surprises to `CLAUDE.md`.
- [x] Append diary entry to progress log before quitting.
</execution_plan>

NOW EXECUTE
