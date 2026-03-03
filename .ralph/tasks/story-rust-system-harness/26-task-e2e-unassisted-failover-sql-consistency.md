---
## Task: Add unassisted failover e2e with before/after SQL consistency proof <status>done</status> <passes>true</passes> <passing>true</passing>

<blocked_by>24-task-real-e2e-harness-3nodes-3etcd</blocked_by>
<blocked_by>25-task-enforce-e2e-api-only-control-no-direct-dcs</blocked_by>

<description>
**Goal:** Create a skeptical e2e test proving full failover completes after killing one postgres instance, with no further interventions beyond API reads and SQL validation.

**Scope:**
- Add a dedicated e2e scenario that writes SQL before failure, kills the active primary postgres process, then performs no control actions.
- Poll only exposed API status endpoints to detect health/recovery completion.
- After recovery, write new SQL and read back both pre-failure and post-failure data to validate continuity and correctness.
- Assert demotion/promotion/fencing outcomes from observable API state transitions (not direct binary hooks).

**Context from research:**
- Existing e2e matrix currently steers failover by direct DCS writes, which is no longer acceptable for strict proof.
- Requested acceptance bar is explicit: kill one pg instance, do nothing else, wait for API to report healthy, then validate SQL read/write behavior.
- Existing postgres harness helpers and process controls can be reused for the single injected failure action.

**Expected outcome:**
- A high-confidence failover proof exists showing autonomous HA recovery and preserved write/read correctness across failure.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist completed with concrete module requirements: dedicated unassisted scenario + SQL helpers + API-only convergence + before/after data assertions implemented in `src/ha/e2e_multi_node.rs`.
- [x] `make check` — passes cleanly (`.ralph/evidence/26-task-e2e-unassisted-failover-sql-consistency/make-check.log`)
- [x] `make test` — passes cleanly (`.ralph/evidence/26-task-e2e-unassisted-failover-sql-consistency/make-test.log`); grep for `congratulations|evaluation failed` returned no matches.
- [x] `make lint` — passes cleanly (`.ralph/evidence/26-task-e2e-unassisted-failover-sql-consistency/make-lint.log`); grep for `congratulations|evaluation failed` returned no matches.
- [x] `make test` — all BDD features pass (`.ralph/evidence/26-task-e2e-unassisted-failover-sql-consistency/make-test.log`)
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2 - Skeptically Verified)

### Deep skeptical verification tracks completed (16)
- Track 1: re-read `src/ha/e2e_multi_node.rs` fixture startup/wiring and confirmed API worker is already present per node with `/ha/state` request helper.
- Track 2: re-read current scenario matrix and confirmed failover currently depends on control actions beyond failure injection (`DELETE /ha/leader` and `POST /ha/leader`) after killing postgres.
- Track 3: verified current scenario still includes manual leader/switchover steering, so it does not satisfy unassisted failover proof requirements.
- Track 4: verified `/ha/state` contract in `src/api/mod.rs` and `src/api/controller.rs`; it exposes `self_member_id`, `leader`, `ha_phase`, trust/ticks/actions, and is suitable for status-only convergence checks.
- Track 5: verified there is no existing SQL execution helper in e2e harness paths; `psql` binary exists in resolved real-test binaries but is currently unused in scenario assertions.
- Track 6: verified real e2e already has timeout hardening (`E2E_COMMAND_TIMEOUT`, `E2E_HTTP_STEP_TIMEOUT`, `E2E_SCENARIO_TIMEOUT`) and timeline artifact support that should be reused.
- Track 7: verified current fencing/rewind assertions partially rely on internal `process_subscriber` job-kind observations, which must be replaced by API-visible phase transitions for this task.
- Track 8: verified `ClusterFixture` can already identify node-by-id and map ID/index; extending it with port-aware SQL helpers will be straightforward.
- Track 9: verified no reusable helper in `src/test_harness/pg16.rs` for running SQL against a port.
- Track 10: verified task acceptance requires explicit before/after SQL write-read continuity proof and no post-failure control interventions beyond API reads.
- Track 11: verified required gate order from `Makefile` remains `make check` -> `make test` -> `make test` -> `make lint`.
- Track 12: verified existing timeline artifact directory pattern under `.ralph/evidence/13-e2e-multi-node` should be reused or extended for this scenario’s forensic trace.
- Track 13: verified `ClusterFixture` currently has no node postgres port in `NodeFixture`; SQL against elected primary cannot be implemented without adding this field.
- Track 14: verified API-only path can still prove demotion/promotion by capturing per-node `ha_phase` history and asserting concrete transitions over time.
- Track 15: verified current test function name/artifact file are matrix-specific; dedicated unassisted proof should use dedicated test and artifact naming for traceability.
- Track 16: verified preserving existing matrix scenario is lower-risk than rewriting it, so dedicated scenario should be additive and isolated.

### Mandatory plan changes from Draft 1 (skeptical corrections)
1. Keep SQL helper local to `src/ha/e2e_multi_node.rs` instead of touching shared harness module now; this isolates risk and avoids cross-target breakage from shared harness API churn.
2. Remove all process-worker introspection for this scenario (no `process_subscriber`, no `ActiveJobKind`) and replace with explicit API phase-history assertions only.
3. Require stabilization windows using consecutive polls for leadership/primary convergence (not single-sample success) to avoid transient false positives.
4. Add dedicated artifact filename and timeline markers for this scenario to separate evidence from the existing matrix run.

### Design decisions for implementation
1. Add a dedicated e2e scenario function in `src/ha/e2e_multi_node.rs` focused only on unassisted failover + SQL continuity proof, instead of overloading the broad scenario matrix.
2. Keep failure injection limited to one explicit action: killing the active primary postgres process via existing fixture method.
3. After failure injection, perform no API control mutations (`POST /switchover`, `POST /ha/leader`, `DELETE /ha/leader`, `DELETE /ha/switchover`) and rely exclusively on `GET /ha/state` polling for convergence.
4. Validate demotion/promotion/fencing outcomes from observable API state transitions over time (phase history across nodes), not internal process worker subscribers.
5. Execute SQL through explicit helper(s) that return rich errors and bounded timeouts; avoid any unwrap/expect/panic paths.

### Planned code changes
1. Add API-state transition capture helpers in `src/ha/e2e_multi_node.rs`.
- Introduce helpers that collect cluster `/ha/state` snapshots plus per-node phase history.
- Add convergence helper requiring consecutive successful polls (for example, 5 consecutive samples) where:
- exactly one primary exists,
- all reachable nodes agree on `leader`,
- primary is the expected member id.
- Add helper that asserts former primary was observed as `Primary` before failure and observed in at least one non-`Primary` phase after failure.
- Keep timeline evidence entries for each important transition.

2. Add SQL helper utilities local to `src/ha/e2e_multi_node.rs`.
- Add helper to execute `psql` against `host=127.0.0.1 port=<node_port> user=postgres dbname=postgres` with bounded timeout and captured stdout/stderr.
- Add helper to execute query output in stable `-AXqt` format and parse rows.
- Add retry helper for post-failover SQL readiness on the elected primary.

3. Extend fixture node metadata and mapping.
- Add postgres port field to `NodeFixture`.
- Provide helper to resolve node id -> postgres port and fail with rich error if missing.
- Keep all mapping logic deterministic through existing node list ordering + ids.

4. Add dedicated unassisted failover SQL consistency scenario.
- Bootstrap: wait for stable primary via API-only polling.
- Pre-failure SQL:
- create scenario table,
- insert pre-failure row with deterministic key/value,
- read and assert row exists.
- Failure injection: kill only the current primary postgres process.
- Unassisted recovery:
- poll only `GET /ha/state` until new stable primary differs from failed node,
- assert no dual-primary during observation window,
- assert former primary transitions away from `Primary`.
- Post-failure SQL:
- insert post-failure row on new primary,
- query ordered rows and assert both pre/post records are present and correct.

5. Keep existing broad matrix scenario unchanged.
- Additive test only; no behavior changes to matrix scenario in this task.
- Dedicated scenario gets dedicated timeline artifact file naming.

6. Enforce no post-failure control actions in dedicated scenario.
- After failure injection, call only helpers that issue `GET /ha/state` plus SQL helpers.
- Do not invoke `post_switchover_via_api`, `post_set_leader_via_api`, `delete_leader_via_api`, or `delete_switchover` path from this new test.

### Validation plan
1. Run targeted test first:
- `cargo test ha::e2e_multi_node::e2e_multi_node_unassisted_failover_sql_consistency -- --nocapture`
2. Run required full gates in exact order:
- `make check`
- `make test`
- `make test`
- `make lint`
3. Persist logs under task-specific evidence directory and grep `make test`/`make lint` logs for `congratulations` and `evaluation failed`.
4. Update task checklist and tags only after all gates pass.
</execution_plan>

NOW EXECUTE
