---
## Task: Implement pginfo worker single-query polling and real PG tests <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>03-task-worker-state-models-and-context-contracts</blocked_by>

<description>
**Goal:** Implement pginfo state derivation with one SQL poll query and verify behavior against real PG16.

**Scope:**
- Implement `src/pginfo/query.rs`, `src/pginfo/state.rs`, `src/pginfo/worker.rs`, `src/pginfo/mod.rs`.
- Implement `PGINFO_POLL_SQL`, `poll_once`, `derive_readiness`, `to_member_status`, `run`, `step_once`.
- Add tests using real postgres instances for primary/replica role transitions and WAL/slot fields.

**Context from research:**
- Plan requires exactly one query per poll cycle for all HA-needed fields.

**Expected outcome:**
- PgInfo publishes correct typed state snapshots from real database behavior.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] `PGINFO_POLL_SQL` returns all required fields for role/readiness/timeline/WAL/slots.
- [ ] `PgInfoState` transitions are verified for unavailable -> primary/replica.
- [ ] Real PG16 tests validate WAL movement and role switch behavior.
- [ ] Run targeted pginfo tests.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] If failures occur, create `$add-bug` task(s) with logs and SQL/result evidence.
</acceptance_criteria>
