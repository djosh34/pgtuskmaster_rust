---
## Task: Build parallel-safe real-system test harness for PG16 and etcd3 <status>not_started</status> <passes>false</passes> <priority>ultra_high</priority>

<blocked_by>02-task-runtime-config-schema-defaults-parse-validate,03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Provide deterministic, parallel-safe infrastructure for real integration and e2e tests.

**Scope:**
- Implement `src/test_harness/namespace.rs`, `ports.rs`, `pg16.rs`, `etcd3.rs`, `tls.rs`, `auth.rs`, and `mod.rs`.
- Implement `create_namespace`, `cleanup_namespace`, `allocate_ports`, `prepare_pgdata_dir`, `prepare_etcd_data_dir`, `spawn_pg16`, and `spawn_etcd3`.
- Ensure no shared dirs/sockets/ports across tests.

**Context from research:**
- Plan requires real-system heavy testing and parallel-safe isolation.

**Expected outcome:**
- Tests can run multiple nodes and infra instances concurrently without cross-test interference.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Every test instance uses unique namespace + unique dirs + unique ports.
- [ ] Harness cleanup executes on success and failure paths.
- [ ] Harness tests verify concurrent test runs do not conflict.
- [ ] Run targeted harness tests.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] If failures occur, add `$add-bug` tasks with namespace/port collision logs.
</acceptance_criteria>
