---
## Task: Enforce API-only control in e2e and ban direct DCS mutations <status>not_started</status> <passes>false</passes>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>
<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>

<description>
**Goal:** Ensure full e2e tests never write/delete DCS keys directly and only control/read HA behavior through exposed API endpoints.

**Scope:**
- Refactor e2e flows to remove direct `DcsStore` writes/deletes from test logic.
- Replace direct control with API requests (and optional CLI invocation where appropriate) for switchover/failover/admin operations.
- Add a verification gate/test that fails if e2e tests reintroduce direct DCS mutation patterns.
- Keep read-only validation through API responses and allowed SQL probes.

**Context from research:**
- Current scenario in `src/ha/e2e_multi_node.rs` explicitly writes/deletes leader keys through `EtcdDcsStore`.
- Requirement is strict: e2e can read state, but control must be through normal exposed API only.
- We need an explicit regression guard task that proves this policy stays enforced.

**Expected outcome:**
- E2E suites are API-driven and a dedicated policy check prevents future direct DCS interaction regressions.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: `src/ha/e2e_multi_node.rs` and any new e2e files (remove direct `write_path`/`delete_path` DCS control), API helper modules/tests updated for equivalent actions, new policy guard test/script (for example in `tests/` or `scripts/`) that fails on direct DCS control usage inside e2e suites
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
