---
## Task: Implement process worker single-active-job execution with real job tests <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Implement process worker to run exactly one long-running job at a time and publish deterministic outcomes.

**Scope:**
- Implement `src/process/jobs.rs`, `src/process/state.rs`, `src/process/worker.rs`, `src/process/mod.rs`.
- Implement `can_accept_job`, `start_job`, `tick_active_job`, `cancel_active_job`, `run`, and `step_once`.
- Add real execution tests for `Bootstrap`, `PgRewind`, `Promote`, `Demote`, `StartPostgres`, `StopPostgres`, `RestartPostgres`, and `Fencing`.

**Context from research:**
- Plan requires no queue/history, only single active job + latest outcome.

**Expected outcome:**
- Process worker executes HA-dispatched jobs safely and exposes accurate state for HA decisions.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Worker rejects new jobs while one is active.
- [ ] `JobOutcome` states are emitted for success/failure/timeout/cancelled.
- [ ] Real job execution tests run against actual binaries and harness resources.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] Failures trigger `$add-bug` tasks with failing job kind and repro.
</acceptance_criteria>
