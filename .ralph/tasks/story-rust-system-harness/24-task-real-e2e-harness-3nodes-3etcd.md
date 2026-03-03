---
## Task: Upgrade real e2e harness to 3 pgtuskmaster nodes and 3 etcd members <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Make the real e2e environment represent a true 3-node HA control plane with a 3-member etcd cluster instead of a single etcd instance.

**Scope:**
- Extend harness support for multi-member etcd cluster bootstrap and lifecycle management.
- Update e2e fixture setup to always launch 3 pgtuskmaster nodes wired to 3 etcd members.
- Ensure all real e2e suites consume this topology by default.
- Add clear readiness waits and teardown handling so cluster tests remain stable and deterministic.

**Context from research:**
- Current `src/ha/e2e_multi_node.rs` starts 3 nodes but only one etcd instance.
- New requirements explicitly demand 3 pgtuskmaster + 3 etcd in e2e tests.
- Existing harness modules (`src/test_harness/etcd3.rs`, `src/test_harness/ports.rs`, `src/test_harness/namespace.rs`) are the right extension points.

**Expected outcome:**
- Real e2e tests run against a 3x3 topology and no longer rely on a single-etcd simplification.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: `src/test_harness/etcd3.rs` (multi-member cluster spawner), `src/test_harness/ports.rs` (port planning for 3 etcd + 3 nodes), `src/ha/e2e_multi_node.rs` or successor e2e fixtures (consume cluster endpoints), e2e teardown/retry logic files updated for clean shutdown
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
