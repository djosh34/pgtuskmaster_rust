## Task: Enforce API-only control in e2e and ban direct DCS mutations <status>done</status> <passes>true</passes>

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
- [x] Full exhaustive checklist completed with concrete module requirements: `src/ha/e2e_multi_node.rs` and any new e2e files (remove direct `write_path`/`delete_path` DCS control), API helper modules/tests updated for equivalent actions, new policy guard test/script (for example in `tests/` or `scripts/`) that fails on direct DCS control usage inside e2e suites
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2 — Deep Skeptical Verification)

### Deep verification tracks completed (16+)
- Track 1: re-read `src/ha/e2e_multi_node.rs` and reconfirmed direct mutation calls remain (`write_path` / `delete_path`) inside real e2e scenario flow.
- Track 2: reconfirmed direct controller mutation usage (`post_switchover`) bypasses HTTP routing in e2e.
- Track 3: revalidated API routes and methods in `src/api/worker.rs` for all needed actions (`POST /switchover`, `POST /ha/leader`, `DELETE /ha/leader`, `DELETE /ha/switchover`, `GET /ha/state`).
- Track 4: verified `GET /ha/state` depends on `set_ha_snapshot_subscriber`, otherwise returns `503`.
- Track 5: checked `tests/bdd_api_http.rs` transport helper style for raw HTTP over `TcpStream` that can be reused.
- Track 6: checked `src/worker_contract_tests.rs` for debug snapshot wiring shape (`DebugApiCtx` plus subscriber handoff).
- Track 7: verified e2e fixture currently has no API worker per node and no API address plumbing.
- Track 8: verified e2e convergence checks still rely on internal subscribers (HA/DCS/process) and not API payloads.
- Track 9: verified no existing policy test blocks future direct DCS mutation in e2e modules.
- Track 10: revalidated mandatory gate order from `Makefile`: `make check` -> `make test` -> `make test-long` -> `make lint`.
- Track 11: checked for broad false-positive risk in guard scope because `tests/bdd_api_http.rs` intentionally implements a `DcsStore` test double with `write_path`/`delete_path` methods.
- Track 12: checked that real e2e already uses 3-node/3-etcd fixture from prior task and should be minimally disrupted.
- Track 13: rechecked current task file acceptance criteria to ensure guard can be implemented as Rust test in `tests/`.
- Track 14: reviewed current worktree and confirmed unrelated dirty files exist, so edits must be scoped tightly to this task’s files only.
- Track 15: verified no-unwrap/no-expect/no-panic requirement should remain enforced in all new helper code.
- Track 16: revalidated lifecycle requirement: after skeptical delta, switch marker from `TO BE VERIFIED` to `NOW EXECUTE`.

### Mandatory skeptical deltas applied to Draft 1
- Changed plan item (guard scope hardening): limit policy guard scanning to e2e modules under `src/ha/` (for example `src/ha/e2e_*.rs`) instead of all tests, to avoid false positives from intentionally mutating DCS test doubles in non-e2e suites.
- Changed plan item (control/read split): keep API-only requirement strict for control actions, but retain internal process-subscriber probes for rewind/fencing observability where `/ha/state` does not expose job-kind detail yet.
- Changed plan item (HTTP validation robustness): require response status verification for each control action (expect `202`) and parse `/ha/state` JSON contract fields before using results in convergence waits.

### Refined architecture changes
1. Add API worker runtime to each e2e node fixture.
- Spawn one API worker listener (`127.0.0.1:0`) per node.
- Persist each node API address in `NodeFixture`.

2. Wire per-node snapshot feed for `/ha/state`.
- Spawn one debug snapshot worker per node and connect API worker via `set_ha_snapshot_subscriber(...)`.
- Keep worker cancellation/teardown consistent with existing `tasks` lifecycle.

3. Replace direct DCS control and controller mutations with endpoint calls.
- Remove `EtcdDcsStore` control mutations and direct `api::controller` calls from e2e scenario code.
- Add e2e-local HTTP helpers with explicit error propagation.
- Scenario mapping:
- planned switchover -> `POST /switchover`.
- conflicting leader injection -> `POST /ha/leader`.
- leader clear -> `DELETE /ha/leader`.
- optional cleanup -> `DELETE /ha/switchover` only when scenario consistency needs it.

4. Shift read-side convergence to `/ha/state` where feasible.
- Add polling helpers that call `GET /ha/state` on all nodes and derive current leader/fail-safe conditions from response payload.
- Keep process subscriber checks for rewind/fencing path until API exposes equivalent granularity.

5. Add regression guard test.
- Add `tests/policy_e2e_api_only.rs` to scan e2e source files and fail on forbidden direct mutation/control patterns.
- Initial forbidden patterns:
- `.write_path(`
- `.delete_path(`
- `api::controller::`
- `post_switchover(` from controller context in e2e files.
- Report exact offending file + pattern in failure message.

6. Keep strict error handling.
- All new async helpers return `Result<_, WorkerError>`.
- No unwrap/expect/panic/todo/unimplemented additions.

### Execution phases (NOW EXECUTE)
1. Fixture/API plumbing.
- Add API and debug worker startup + subscriber wiring + node API address storage.

2. Scenario migration.
- Convert switchover/fencing/failover control actions to HTTP requests.
- Validate expected 202 responses.

3. API-read assertions.
- Add `/ha/state` polling-based waits for leader target and fail-safe convergence.
- Retain internal-only checks where API lacks equivalent fidelity.

4. Regression guard.
- Add policy test targeting e2e modules only.

5. Validation and evidence.
- Run gates in required order:
- `make check`
- `make test`
- `make test-long`
- `make lint`
- Capture `congratulations` / `evaluation failed` markers for `make test` and `make lint`.
</execution_plan>

NOW EXECUTE
