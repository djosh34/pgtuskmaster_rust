---
## Task: Implement security auth TLS validation tests in real cluster runs <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>10-task-test-harness-namespace-ports-pg-etcd-spawners,13-task-e2e-multi-node-real-ha-loops-scenario-matrix</blocked_by>

<description>
**Goal:** Verify auth and TLS behavior under real deployment conditions.

**Scope:**
- Add test scenarios for node-to-node auth, client auth, TLS required/optional/disabled modes, invalid/expired certs, wrong CA, and endpoint role permissions.
- Ensure failures are explicit and do not silently downgrade security.

**Context from research:**
- Plan includes dedicated security/auth/TLS matrix as mandatory quality gate.

**Expected outcome:**
- Security behavior is proven by real-system tests and regressions are detectable.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Positive and negative auth/TLS cases are covered with real certificates and network connections.
- [ ] API role permissions are tested for allow/deny behavior.
- [ ] Security tests run in parallel-safe harness namespaces.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] On any failure, create `$add-bug` tasks including cert/config artifacts and reproductions.
</acceptance_criteria>
