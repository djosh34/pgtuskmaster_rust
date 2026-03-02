---
## Task: Implement e2e multi-node real HA-loop scenario matrix <status>not_started</status> <passes>false</passes> <priority>ultra_high</priority>

<blocked_by>09-task-api-debug-workers-and-snapshot-contracts,10-task-test-harness-namespace-ports-pg-etcd-spawners,12-task-ha-loop-integration-tests-real-watchers-and-step-once</blocked_by>

<description>
**Goal:** Validate real-system HA behavior with all nodes running their own HA loops concurrently.

**Scope:**
- Build e2e tests that boot full clusters using real PG16 + etcd3 + running node processes.
- Cover bootstrap, planned switchover, failover, no-quorum fail-safe, rewind rejoin, fencing-before-promotion, and split-brain prevention.
- Ensure test code only injects faults/inputs; HA actions must be executed by the system itself.

**Context from research:**
- This task encodes the user requirement for real scenarios with multiple pgtuskmaster nodes and autonomous loops.

**Expected outcome:**
- E2E suite proves cluster-level HA behavior under real multi-node conditions.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] E2E suite launches at least 3 nodes with all HA loops running concurrently.
- [ ] Tests verify leader election, promotion, demotion, fencing, and recovery using observed system outputs only.
- [ ] Tests do not call internal HA decision functions to enact transitions.
- [ ] Scenario matrix includes all plan-listed HA paths.
- [ ] Run e2e suite standalone and collect logs/artifacts.
- [ ] Run full suite: `make check`, `make test`, `make lint`, `make test-bdd`.
- [ ] For every failing scenario, use `$add-bug` and create bug task(s) with timeline/log evidence.
</acceptance_criteria>
