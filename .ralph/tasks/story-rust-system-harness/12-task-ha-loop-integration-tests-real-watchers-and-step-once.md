---
## Task: Build HA loop integration tests with real watchers and deterministic stepping <status>not_started</status> <passes>false</passes> <priority>ultra_high</priority>

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
- [ ] Integration tests cover transitions into `Replica`, `CandidateLeader`, `Primary`, `FailSafe`, `Rewinding`, and `Fencing`.
- [ ] Tests assert dispatched actions and resulting process state transitions.
- [ ] No test manually mutates HA phase/action lists to force success.
- [ ] Run integration suite.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] Any failure creates `$add-bug` tasks with snapshot/action traces.
</acceptance_criteria>
