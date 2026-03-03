---
## Task: Add HA stress e2e suites with concurrent SQL workloads during role changes <status>not_started</status> <passes>false</passes>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>

<description>
**Goal:** Build stress-oriented e2e tests that continuously read/write/query SQL while HA switchover and failover paths execute, and verify safe demotion/promotion/fencing behavior.

**Scope:**
- Add workload generators that run concurrent SQL read/write traffic through the active cluster endpoint during HA transitions.
- Cover planned switchover, unplanned failover, and fencing-sensitive windows with measurable assertions.
- Assert no split-brain writes, no dual-primary windows, and expected role convergence via API-visible state.
- Add timeline artifacts/metrics to aid deterministic debugging when stress tests fail.

**Context from research:**
- Existing e2e scenario is primarily a reaction matrix and does not sustain high SQL activity during transitions.
- Requirement calls for many more skeptical stress tests while HA operations are in flight.
- Current worker state channels and timeline artifacts can be extended for richer observability.

**Expected outcome:**
- The suite includes multiple high-load HA transition scenarios proving safety and data correctness under stress.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: new/updated stress e2e files (`src/ha/e2e_*` and/or `tests/e2e_*`), SQL workload helper module(s), API-state assertion utilities, artifact logging paths under `.ralph/evidence` for stress timelines and summary stats
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
