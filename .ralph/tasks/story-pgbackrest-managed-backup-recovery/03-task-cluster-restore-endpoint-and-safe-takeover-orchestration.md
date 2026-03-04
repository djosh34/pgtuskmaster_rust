---
## Task: Add cluster restore endpoint and safe takeover orchestration across HA/DCS <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Provide an admin API endpoint that forces a full restore takeover on one node and safely converges the entire cluster onto the restored timeline under normal pgtuskmaster HA control.

**Scope:**
- Add restore intent endpoint(s) and status endpoint(s) in API layer with admin auth enforcement.
- Persist restore intents/locks/status in DCS with strict single-flight coordination and explicit ownership.
- Extend HA decision/worker flow to execute restore takeover sequence:
- fence conflicting primaries safely
- execute restore on selected node
- start postgres recovery with managed config takeover
- transition restored node into normal primary leadership
- force remaining nodes to converge via rewind/basebackup as needed
- Ensure restore orchestration works whether the source backup came from a pgtuskmaster-managed cluster or an external/non-pgtuskmaster cluster.

**Context from research:**
- Existing API surface currently supports switchover/fallback but no restore lifecycle endpoint (`src/api/worker.rs`).
- Existing HA loop already coordinates demote/promote/start/rewind/bootstrap actions, so restore should plug into that action model (`src/ha/actions.rs`, `src/ha/worker.rs`, `src/ha/decide.rs`).
- Existing DCS cache/state model already tracks cluster coordination records and can be extended for restore-intent records (`src/dcs/keys.rs`, `src/dcs/state.rs`, `src/dcs/store.rs`).

**Expected outcome:**
- Operators can trigger a restore takeover via API and see explicit progress/failure state.
- Only one restore flow can run at a time cluster-wide; duplicate/competing requests are rejected safely.
- After successful restore, cluster resumes normal HA behavior and non-executor nodes converge automatically.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Extend API routes/controllers in `src/api/controller.rs` and `src/api/worker.rs`:
- [ ] add admin endpoint to request restore takeover (e.g. `POST /ha/restore`)
- [ ] add read endpoint to inspect restore status/progress/errors
- [ ] enforce admin auth role and clear input validation errors
- [ ] Extend DCS model/store in `src/dcs/keys.rs`, `src/dcs/state.rs`, `src/dcs/store.rs`, and adapter(s):
- [ ] add restore intent/status records and canonical key paths
- [ ] implement compare-and-set/single-flight semantics for restore ownership
- [ ] ensure watch/cache refresh logic handles restore records correctly across reconnect/resnapshot
- [ ] Extend HA action model in `src/ha/actions.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, and related state types:
- [ ] define restore orchestration actions/phases and deterministic transition rules
- [ ] enforce fencing-first behavior on conflicting primaries during restore takeover
- [ ] converge non-executor nodes to restored leader via existing rewind/basebackup flows
- [ ] ensure failed restore leaves cluster in explicit safe/diagnosable state
- [ ] Extend debug snapshot/views in `src/debug_api/snapshot.rs` and `src/debug_api/view.rs` to expose restore lifecycle state
- [ ] Add API BDD tests in `tests/bdd_api_http.rs`:
- [ ] auth matrix for restore endpoint(s)
- [ ] bad request and idempotency/conflict responses
- [ ] status endpoint payload checks
- [ ] Add HA integration tests (unit + worker integration) for restore orchestration correctness:
- [ ] competing restore requests cannot run concurrently
- [ ] restore request mid-switchover/failover resolves safely
- [ ] split-brain prevention rules remain intact
- [ ] Add real e2e scenario(s) in HA test suites showing full restore takeover and cluster convergence
- [ ] Document operator restore runbook and rollback guidance in `docs/src/operator/`
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
