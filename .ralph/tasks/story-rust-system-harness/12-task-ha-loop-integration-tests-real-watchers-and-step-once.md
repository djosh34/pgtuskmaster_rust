---
## Task: Build HA loop integration tests with real watchers and deterministic stepping <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<blocked_by>08-task-ha-worker-select-loop-and-action-dispatch,10-task-test-harness-namespace-ports-pg-etcd-spawners</blocked_by>

<description>
**Goal:** Verify HA loop correctness when worker states interact together through typed channels.

**Scope:**
- Create integration suites that wire config + pginfo + dcs + process + ha together.
- Use `step_once` for deterministic assertions and `run` for long-loop behavior.
- Ensure tests observe HA decisions from loop outputs; tests must not directly perform HA operations.

**Context from research:**
- User requirement: tests cannot do HA themselves; loops must act autonomously.

**Expected outcome:**
- Inter-worker behavior is validated under realistic state evolution without mock-driven HA shortcuts.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Integration tests cover transitions into `Replica`, `CandidateLeader`, `Primary`, `FailSafe`, `Rewinding`, and `Fencing`.
- [x] Tests assert dispatched actions and resulting process state transitions.
- [x] No test manually mutates HA phase/action lists to force success.
- [x] Run integration suite.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test-bdd`.
- [x] Any failure creates `$add-bug` tasks with snapshot/action traces.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Scope lock and baseline
- [x] Verify dependencies `08` and `10` are already marked done/passing in `.ralph/tasks/story-rust-system-harness/`.
- [x] Capture baseline before edits with `cargo check --all-targets` (fast attribution point if later failures appear).
- [x] Preserve constraints: no `unwrap`/`expect`/`panic` in runtime code; in tests, do not mutate HA internals (`phase`, `pending`, `recent_action_ids`) except initial fixture seed.

2. Place integration tests where crate-private worker contexts are already exercised
- [x] Extend `src/ha/worker.rs` `#[cfg(test)] mod tests` rather than adding a new module file, to avoid visibility churn and duplicate fixture code.
- [x] Keep existing unit tests intact and add a dedicated integration-style fixture section with clear helper names so intent stays separable.

3. Build a full multi-worker in-memory fixture with real watch channels
- [x] Build a fixture that owns publishers/subscribers for config, pginfo, dcs, process, and ha.
- [x] Instantiate real contexts:
- [x] `DcsWorkerCtx` using pg subscriber + dcs publisher.
- [x] `ProcessWorkerCtx` consuming `process_inbox` requests and publishing `ProcessState`.
- [x] `HaWorkerCtx` consuming config/pg/dcs/process subscribers, dispatching into process inbox, and publishing `HaState`.
- [x] Keep all subscriber handles alive for fixture lifetime to prevent channel-close false failures.
- [x] Use deterministic clocks for HA and process workers so IDs/timestamps are reproducible.

4. External-boundary doubles only
- [x] Reuse/extend `RecordingStore` in HA tests to both:
- [x] record leader lease writes/deletes,
- [x] optionally script watch events for DCS worker refresh.
- [x] Add a deterministic process runner/handle in HA tests (local to module) that can return queued `ProcessExit`/error behaviors without real binaries.
- [x] Keep watcher channels real; only DCS store and process-command boundaries are faked.

5. Add deterministic system-step helper (critical skeptical adjustment)
- [x] Add helper that advances workers in realistic order for one logical cycle:
- [x] publish input updates (if any),
- [x] `dcs::worker::step_once`,
- [x] `ha::worker::step_once`,
- [x] `process::worker::step_once`,
- [x] `ha::worker::step_once` again when process outcome is needed for phase progression.
- [x] This avoids hidden ordering assumptions and makes phase assertions explicit per cycle.

6. Required HA phase transition coverage with assertions on outputs only
- [x] Add scenarios covering:
- [x] `WaitingDcsTrusted -> Replica` (external leader present).
- [x] `WaitingDcsTrusted -> CandidateLeader` (leader absent).
- [x] `CandidateLeader -> Primary` (self leader lease visible).
- [x] `* -> FailSafe` (trust degradation).
- [x] `Primary -> Rewinding` (postgres unreachable).
- [x] `Primary -> Fencing` (split-brain: other leader appears while reachable).
- [x] For each scenario assert:
- [x] published `HaState.phase`,
- [x] published `pending` actions,
- [x] side effects observed via process inbox/job kind and DCS writes/deletes.

7. End-to-end process action consumption coverage
- [x] Add integration scenarios where HA-dispatched actions are consumed by process worker and feed back into HA decisions:
- [x] `StartPostgres` dispatch causes process running/idle outcome publication.
- [x] `StartRewind` + process success drives `Rewinding -> Replica`.
- [x] `DemoteToReplica`/`FenceNode` paths enqueue expected `ProcessJobKind` and influence subsequent HA phase.
- [x] Assert only on published states and observed queued jobs, not internal mutable fields.

8. Run-loop coverage with bounded scope (skeptical reduction against flake)
- [x] Keep `run` coverage focused on event-loop wakeups and first-transition parity (already partly present), then add one additional multi-event run assertion with strict timeout.
- [x] Do not attempt full long timeline parity under `run`; rely on deterministic `step_once` integration for deep transition matrix.

9. Traceability and diagnostics
- [x] In new integration tests, record concise local timelines (phase sequence + action IDs + job kinds + DCS ops) to improve failure messages.
- [x] If any required gate fails, create bug task(s) via `$add-bug` with exact repro command and observed trace snapshot.

10. Validation order (sequential to avoid Cargo lock/artifact races)
- [x] Run targeted tests first (new HA integration tests plus related HA/DCS/process worker test modules).
- [x] Run required gates sequentially and rerun after fixes until all pass:
- [x] `make check`
- [x] `make test`
- [x] `make test-bdd`
- [x] `make lint`

11. Task bookkeeping after green gates
- [x] Tick acceptance criteria checkboxes with command evidence.
- [x] Update task header tags to done/passes true only after all gates succeed.
- [x] Set `<passing>true</passing>` at final completion only.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changes (including `.ralph` files) with:
- [x] `task finished 12-task-ha-loop-integration-tests-real-watchers-and-step-once: <summary + gate evidence + implementation notes>`
- [x] Append new learnings/surprises to `AGENTS.md`.
</execution_plan>

NOW EXECUTE
