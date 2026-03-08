## Task: Add HA Restart And Leadership Churn E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Add behavioural coverage for HA failures that are not simple one-shot failovers: process restarts, repeated leadership churn, and degraded-cluster transitions that happen across multiple events in sequence. The higher-order goal is to test the system the way real operators break it: not with one clean outage, but with restarts, retries, and more than one leadership event in the same test.

This task should specifically lean into the currently missing failure choreography that came out of research and the user feedback:
- node process restarts
- repeated failovers or quick successive leader changes
- additional HA failure cases beyond the existing etcd/API partition and single-stop failover scenarios

**Scope:**
- Extend the HA e2e suite with new scenarios focused on restart and churn behaviour.
- Prefer scenarios that remain parallel-safe and deterministic under the existing harness isolation model.
- Candidate coverage areas include:
- restarting the HA node process on the primary or replica and verifying stable recovery without role confusion
- inducing two leadership transitions in one scenario and proving the cluster does not accumulate stale authority or dual-primary evidence
- failing over when one replica is already degraded or unavailable
- issuing switchover intent to a node that cannot currently become a healthy target and verifying clear rejection or safe non-action
- Keep each scenario small enough to have one unambiguous failure story, instead of mixing too many fault types at once.

**Context from research:**
- The current suite already covers planned switchover, unassisted failover, no-quorum fail-safe, fencing, and a set of etcd/API partition cases.
- The current suite does not cover process restart behaviour, crash-loop style recovery, or multi-step leadership churn in a single scenario.
- The user explicitly asked for more HA failure tests and does not accept flaky tests as normal.
- The user wants full parallelism preserved, so these scenarios must stay isolated and must not rely on global serialization for correctness.

**Expected outcome:**
- The behavioural suite no longer treats HA as only a single clean transition problem.
- The system is validated against restart and churn patterns that commonly expose stale-leader, rejoin, or state-machine bugs.
- The new tests remain deterministic and parallel-safe.

</description>

<acceptance_criteria>
- [ ] Add one or more e2e scenarios covering HA node/process restart behaviour with explicit assertions about recovered leadership and cluster convergence.
- [ ] Add one or more e2e scenarios covering more than one leadership transition in the same scenario, with strict no-dual-primary assertions across the whole observation window.
- [ ] At least one new scenario covers failover or switchover behaviour when the cluster is already degraded before the transition begins.
- [ ] Each new scenario records a clear timeline artifact and uses unique names/tables/artifact paths so parallel execution remains safe.
- [ ] The scenarios explicitly verify final SQL/data convergence, not only API phase changes.
- [ ] The scenarios avoid ambiguous “something changed” assertions and instead check specific expected leadership and replication outcomes.
- [ ] If a target node is intentionally ineligible or unhealthy, the scenario verifies the system fails safely rather than silently accepting an impossible transition.
- [ ] The implementation reuses existing fixture helpers where possible, but extracts new helpers when that meaningfully reduces duplication and clarifies the scenario intent.
- [ ] The implementation does not touch CLI code.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If the new scenarios belong in the long-running gate: `make test-long` — passes cleanly
</acceptance_criteria>
