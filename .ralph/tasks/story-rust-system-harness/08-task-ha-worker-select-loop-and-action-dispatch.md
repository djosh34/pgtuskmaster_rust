---
## Task: Implement HA worker select loop and action dispatch wiring <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>04-task-pginfo-worker-single-query-and-real-pg-tests,05-task-dcs-worker-trust-cache-watch-member-publish,06-task-process-worker-single-active-job-real-job-exec,07-task-ha-decide-pure-matrix-idempotency-tests</blocked_by>

<description>
**Goal:** Wire the HA runtime loop that reacts to typed watcher changes and periodic ticks, then dispatches actions.

**Scope:**
- Implement `src/ha/worker.rs` with `run`, `step_once`, and `dispatch_actions`.
- Use `tokio::select!` over `pg`, `dcs`, `process`, and `config` `changed()` plus interval tick.
- Ensure no wake-bus/event-enum side channels are introduced.

**Context from research:**
- Plan explicitly removes wake-reason buses in favor of direct watcher changes.

**Expected outcome:**
- HA loop advances from real system state changes and dispatches process/DCS actions safely.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] HA worker consumes `WorldSnapshot` from typed subscribers only.
- [ ] `dispatch_actions` is robust to transient errors and produces typed action errors.
- [ ] Integration tests assert `step_once` and full loop parity for the same inputs.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] Failures must generate `$add-bug` tasks with action traces.
</acceptance_criteria>
