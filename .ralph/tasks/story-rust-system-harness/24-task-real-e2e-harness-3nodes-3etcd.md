---
## Task: Upgrade real e2e harness to 3 pgtuskmaster nodes and 3 etcd members <status>done</status> <passes>true</passes> <passing>true</passing>

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
- [x] Full exhaustive checklist completed with concrete module requirements: `src/test_harness/etcd3.rs` (multi-member cluster spawner), `src/test_harness/ports.rs` (port planning for 3 etcd + 3 nodes), `src/ha/e2e_multi_node.rs` or successor e2e fixtures (consume cluster endpoints), e2e teardown/retry logic files updated for clean shutdown
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2 — Skeptically verified)

### Deep skeptical verification tracks (16)
- Track 1: re-read `src/test_harness/etcd3.rs` for API shape and found only single-instance spawn + single fixed data dir helper.
- Track 2: verified `prepare_etcd_data_dir` currently hard-codes `etcd3/data`, which prevents multi-member same-namespace runs.
- Track 3: re-read `src/test_harness/ports.rs` and confirmed no structured topology reservation helper exists yet.
- Track 4: re-read `src/ha/e2e_multi_node.rs` startup path and confirmed `ports_needed = node_count + 2` and exactly one etcd member.
- Track 5: re-read e2e no-quorum path and confirmed it stops only one etcd process; in a 3-member cluster this would not imply quorum loss.
- Track 6: re-read e2e DCS wiring and confirmed all connections use one endpoint string (`vec![endpoint.clone()]`).
- Track 7: re-read e2e teardown and confirmed single `Option<EtcdHandle>` shutdown path only.
- Track 8: scanned `src/dcs/etcd_store.rs` real fixture and found direct dependency on `spawn_etcd3` / `EtcdHandle` single-instance API.
- Track 9: scanned all `allocate_ports(...)` usage to prevent broad signature breakage; generic allocator is used widely.
- Track 10: re-checked `src/test_harness/mod.rs`/error surfaces; no panic-based path is required for cluster orchestration errors.
- Track 11: re-checked readiness call sites and verified only port-open waits exist for etcd currently; cluster-level readiness is absent.
- Track 12: re-checked no-quorum + timeline lines in e2e for artifact semantics; timeline already captures scenario steps and should include which etcd members are stopped.
- Track 13: re-checked `Makefile` gates and confirmed required sequence remains `make check`, `make test`, `make lint`.
- Track 14: checked workspace status and confirmed current task file is mutable and in-progress.
- Track 15: scanned task/story files for acceptance marker behavior (`congratulations`/`evaluation failed`) and captured that marker greps must still be recorded.
- Track 16: re-validated lifecycle protocol for this task file (`TO BE VERIFIED` -> skeptical delta -> `NOW EXECUTE`).

### Mandatory skeptical delta from Draft 1
- **Changed plan item (API compatibility strategy):** keep existing single-member API (`spawn_etcd3`, `EtcdHandle`, and `prepare_etcd_data_dir`) as compatibility wrappers used by existing real-etcd unit/integration tests, and add **new** multi-member APIs (`EtcdClusterMemberSpec`, `EtcdClusterSpec`, `EtcdClusterHandle`, `spawn_etcd3_cluster`, `prepare_etcd_member_data_dir`).
- **Rationale:** Draft 1 implied replacing single-member primitives outright. Deep review showed `src/dcs/etcd_store.rs` real fixture and current harness tests rely on those symbols directly. A hard replacement would create avoidable test churn and increase risk. Wrapper-preserving expansion gives safer migration while still enabling 3x3 e2e topology.
- **Changed plan item (failure semantics):** no-quorum scenario explicitly stops 2/3 members and validates fail-safe, while teardown still attempts shutdown for all members (including already-stopped members) and aggregates outcomes.
- **Rationale:** stopping one etcd member cannot prove quorum-loss behavior in a true 3-member cluster.

### Design goals for implementation
1. Make real e2e default topology exactly 3 pgtuskmaster nodes + 3 etcd members.
2. Keep harness APIs explicit and typed so callers cannot accidentally fall back to single-etcd topology.
3. Add deterministic readiness waiting for multi-member etcd before worker startup.
4. Preserve robust cleanup semantics when some members fail to start or stop.
5. Preserve single-member harness compatibility so existing real-etcd tests continue to compile and run.

### Planned architecture changes
1. Extend etcd harness with additive cluster API in `src/test_harness/etcd3.rs`.
- Add per-member spec type with explicit `member_name`, `data_dir`, `log_dir`, `client_port`, `peer_port`.
- Add cluster spec type carrying binary path, namespace token, startup timeout, and member specs.
- Add `EtcdClusterHandle` holding member handles + ordered endpoints.
- Add `shutdown_all()` that attempts every member shutdown and returns aggregated error context on partial failures.
- Keep `EtcdHandle` + `spawn_etcd3` as wrapper path for single-member compatibility.

2. Add multi-member bootstrap and cleanup logic in `src/test_harness/etcd3.rs`.
- Build shared `--initial-cluster` across all members (`name=http://127.0.0.1:peer_port`).
- Spawn members, wait per-member client port, then run cluster-level read/write readiness probe via etcd client before returning success.
- On partial-start failure, best-effort shutdown all started members before returning error.

3. Add member-specific data-dir helpers in `src/test_harness/etcd3.rs`.
- Add `prepare_etcd_member_data_dir(namespace, member_name)` using `etcd3/<member_name>/data`.
- Keep `prepare_etcd_data_dir` delegating to member helper (e.g., default member name) for compatibility.

4. Add topology-aware port mapping helper in `src/test_harness/ports.rs`.
- Add typed helper to reserve ports for `etcd_members * 2 + node_count` and return structured mapping.
- Keep existing `allocate_ports` API unchanged.

5. Rework e2e fixture to consume 3-member etcd cluster in `src/ha/e2e_multi_node.rs`.
- Replace single endpoint string with endpoint vector.
- Replace `Option<EtcdHandle>` with `Option<EtcdClusterHandle>`.
- Configure runtime DCS endpoints and all `EtcdDcsStore::connect` calls with full endpoint vector.
- Ensure node startup occurs only after cluster readiness succeeds.

6. Rework e2e no-quorum + teardown semantics in `src/ha/e2e_multi_node.rs`.
- Add helper to stop etcd majority (2 of 3 members) and record member IDs in timeline.
- Assert convergence to `HaPhase::FailSafe`.
- Teardown attempts shutdown for all members and reports aggregated member-level failures.

7. Keep harness tests aligned and add deterministic unit checks.
- Preserve existing single-member spawn test behavior via wrapper API.
- Add deterministic tests for topology port mapping and initial-cluster assembly where possible.

### Planned execution phases (NOW EXECUTE)
1. Harness scaffolding pass.
- Implement additive cluster APIs and helper functions in `src/test_harness/etcd3.rs`.
- Implement topology mapping helper + tests in `src/test_harness/ports.rs`.

2. Fixture migration pass.
- Update `src/ha/e2e_multi_node.rs` fixture fields/startup/no-quorum/teardown to cluster handle and endpoint vector.
- Update control store connect call(s) to multi-endpoint.

3. Compile fallout sweep pass.
- Adjust any impacted harness consumers (`src/dcs/etcd_store.rs` tests etc.) only if needed by signature or behavior changes.
- Keep strict no unwrap/expect/panic additions.

4. Validation pass.
- Run mandatory gates in order:
- `make check`
- `make test`
- `make test-long`
- `make lint`
- Capture marker checks for `make test` and `make lint` outputs (`congratulations` / `evaluation failed`) as required by task acceptance language.

### Parallel execution tracks to apply during implementation (15)
- Track 1: cluster spec/handle definitions.
- Track 2: initial-cluster string assembly + arg wiring.
- Track 3: per-member data-dir helper + compatibility wrapper behavior.
- Track 4: cluster readiness probe.
- Track 5: partial-start cleanup and error aggregation.
- Track 6: shutdown-all member aggregation behavior.
- Track 7: topology port mapping helper + tests.
- Track 8: e2e fixture struct migration.
- Track 9: runtime config endpoint-vector propagation.
- Track 10: e2e control-store and ha-store connect updates.
- Track 11: no-quorum majority-shutdown logic.
- Track 12: teardown all-member shutdown path.
- Track 13: timeline diagnostics enrichment with member IDs.
- Track 14: compile fallout cleanup in other real-etcd fixtures.
- Track 15: full gate run + marker evidence checks.
</execution_plan>

NOW EXECUTE
