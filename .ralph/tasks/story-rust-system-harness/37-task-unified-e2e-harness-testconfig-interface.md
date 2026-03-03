---
## Task: Unify HA E2E Harness Behind Stable `TestConfig` Interface <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Design and implement one stable, shared HA e2e harness interface driven by a single `TestConfig` input that initializes the requested cluster topology + pre-test setup, returns a full test handle, and removes duplicated setup/wait/process glue from scenario files.

**Scope:**
- Replace duplicated orchestration in:
  - `src/ha/e2e_multi_node.rs`
  - `src/ha/e2e_partition_chaos.rs`
- Introduce shared test-harness API under `src/test_harness/` for:
  - cluster provisioning from config
  - shared readiness/process/sql/time helpers
  - optional proxy/fault-injection support
  - unified fixture/handle return type(s)
- Keep runtime startup path unchanged (`crate::runtime::run_node_from_config` remains source of truth).
- Preserve post-start e2e policy guard semantics in `tests/policy_e2e_api_only.rs`.

**Context from research (line-anchored):**
- Duplicate startup orchestration blocks exist in both e2e suites:
  - `ClusterFixture::start` in `src/ha/e2e_multi_node.rs:293`
  - `PartitionFixture::start` in `src/ha/e2e_partition_chaos.rs:72`
- Large shared helper duplication (nearly verbatim):
  - `wait_for_node_api_ready_or_task_exit`: multi-node `1615`, partition `903`
  - `read_log_tail`: multi-node `1662`, partition `956`
  - `wait_for_bootstrap_primary`: multi-node `1777`, partition `972`
  - binary resolution helpers: multi-node `1818-1844`, partition `1021-1047`
  - process/sql/time helpers: multi-node `1846-2001`, partition `1049-1204`
  - `run_with_local_set`: multi-node `2159`, partition `1243`
- Existing harness is partially reused but insufficiently centralized:
  - `src/test_harness/pg16.rs:57-132` (`prepare_pgdata_dir`, `spawn_pg16`)
  - `src/test_harness/etcd3.rs:188-273` (`spawn_etcd3_cluster`)
  - `src/test_harness/net_proxy.rs:55-176` (`TcpProxyLink`)
- Policy constraints to preserve:
  - `tests/policy_e2e_api_only.rs:3-35` forbidden/allowed tokens
  - `tests/policy_e2e_api_only.rs:37-89` scans `src/ha/e2e_*.rs` for violations

**Design iteration requirement (must happen first):**
- Before coding, run a focused research/comparison step inside this task and document outcomes in task evidence:
  1. **Interface A: Monolithic scenario runner API**
     - `run_ha_scenario(config, scenario_fn)` abstraction that hides setup/teardown/polling.
  2. **Interface B: Trait-heavy environment abstraction**
     - `trait HaEnv { ... }` with generic scenario logic parameterized by env implementation.
  3. **Interface C (baseline candidate): Config-driven fixture builder + returned handle**
     - `start_cluster(TestConfig) -> TestClusterHandle` where handle exposes operations (`ha_state`, `sql`, `faults`, `waits`, `shutdown`, artifact helpers).
- Cross-check interfaces against:
  - reuse potential across current and future e2e suites
  - readability of scenario tests
  - policy compatibility (`hands-off` post-start controls)
  - migration risk/churn
  - cognitive overhead
- If research finds a superior interface with materially higher reuse/stability than C, adopt it; otherwise proceed with C.

**Proposed stable shared interface baseline (to implement unless superseded by research):**
- New module family under `src/test_harness/ha_e2e/`:
  - `config.rs`: `TestConfig`, `NodeConfig`, `EtcdConfig`, `FaultConfig`, `TimeoutConfig`, `ArtifactConfig`.
  - `cluster.rs`: `TestClusterHandle`, node descriptors, startup/shutdown.
  - `ops.rs`: high-level operations used by tests (`run_sql_on_node`, `wait_for_stable_primary`, etc).
  - `faults.rs`: proxy-backed operations (`partition_node_from_etcd`, `isolate_api_path`, `heal_all_network_faults`) with no-op behavior when proxies disabled.
  - `util.rs`: shared helpers (`wait_for_child_exit_with_timeout`, `run_psql_statement`, `read_log_tail`, `unix_now`, etc).
- `TestConfig` requirements:
  - single entrypoint describing cluster shape + pre-test setup (no ad-hoc 20-function call graph in tests)
  - supports at least:
    - `mode: Plain | PartitionProxy`
    - node count, etcd member count, runtime loop/lease/process settings
    - readiness/scenario timeouts
    - artifact paths
    - security/api defaults
  - optional pre-test actions block (e.g. bootstrap checks, initial schema/sql seeds) executed by harness.
- Return type requirements:
  - one handle struct for scenario execution, containing everything needed:
    - node metadata, API clients, task handles, binary paths, etcd handle, optional proxy maps
    - unified methods for observe/control/fault/wait/assert helpers
    - deterministic `shutdown` + finalize helpers

**Expected outcome:**
- Both e2e suites use one stable shared setup interface and shared helper implementation.
- Startup/process/sql/readiness duplicated code in `src/ha/e2e_multi_node.rs` and `src/ha/e2e_partition_chaos.rs` is removed or reduced to thin scenario-specific wrappers.
- Test scenarios remain black-box and policy-compliant.
- Future e2e additions can be created by editing `TestConfig` and scenario logic, not re-creating setup scaffolding.

**Implementation plan (precise, file + line anchors):**
1. **Research gate inside task**
- Add a dedicated “interface comparison” artifact documenting A/B/C scores and decision rationale before code edits.
- Include concrete mapping from current call sites:
  - multi-node startup at `src/ha/e2e_multi_node.rs:293-495`
  - partition startup at `src/ha/e2e_partition_chaos.rs:72-363`

2. **Introduce shared HA e2e harness module**
- Edit `src/test_harness/mod.rs` to export new module(s) (near module list at `8-15`).
- Add:
  - `src/test_harness/ha_e2e/mod.rs`
  - `src/test_harness/ha_e2e/config.rs`
  - `src/test_harness/ha_e2e/cluster.rs`
  - `src/test_harness/ha_e2e/ops.rs`
  - `src/test_harness/ha_e2e/faults.rs`
  - `src/test_harness/ha_e2e/util.rs`

3. **Move duplicated generic helpers into shared util**
- Port from:
  - multi-node `1615-2164`
  - partition `886-1248`
- Ensure one canonical implementation for:
  - API readiness probe
  - bootstrap primary wait
  - process child wait/kill timeout
  - `psql` execution helper and parsers
  - timestamp and scenario utility helpers
  - local-set runner

4. **Implement `TestConfig -> TestClusterHandle` startup path**
- Use existing harness components (no reinvention):
  - `prepare_pgdata_dir` (`src/test_harness/pg16.rs:57`)
  - `spawn_etcd3_cluster` (`src/test_harness/etcd3.rs:188`)
  - `TcpProxyLink` (`src/test_harness/net_proxy.rs:55`)
- Consolidate current duplicated startup logic:
  - from multi-node `293-495`
  - from partition `72-363`
- Provide optional proxy topology driven only by `TestConfig.mode`.

5. **Provide high-level operations on returned handle**
- Unify common operations currently duplicated:
  - HA observation/poll:
    - multi-node `1291-1451`
    - partition `457-612`
  - SQL/waits:
    - multi-node `529-672`, `880-952`
    - partition `614-744`
  - No-split-brain assertions:
    - multi-node `1435-1450`
    - partition `564-586`
- Keep scenario-specific stress-only logic (workload stats/summary) local to multi-node unless reused.

6. **Migrate e2e suites to new shared interface**
- `src/ha/e2e_multi_node.rs`
  - Replace `ClusterFixture` setup internals and duplicated helper block with harness handle usage.
  - Keep scenario behavior and assertions identical.
  - Refactor repeated finalize match at `2317-2343` and `2416-...` to shared finalize helper.
- `src/ha/e2e_partition_chaos.rs`
  - Replace `PartitionFixture` setup internals and duplicated helper block with harness handle usage.
  - Keep fault-injection semantics and scenario timelines unchanged.

7. **Policy compatibility verification**
- Recheck `tests/policy_e2e_api_only.rs:3-35` constraints after migration.
- If helper names in e2e files change, keep allowed controls unchanged:
  - observation via `/ha/state` path
  - switchover via CLI/API client surface
  - SQL via fixture methods
  - external fault injection only
- Update policy token list only if required and justified by equivalent semantics.

8. **Contract tests for new interface**
- Add focused tests under `src/test_harness/ha_e2e/*` for:
  - `TestConfig` validation and defaults
  - startup returns usable handle for plain mode
  - startup returns usable handle for partition-proxy mode
  - deterministic shutdown behavior and resource cleanup
  - major wait/operation helpers error-path quality

9. **Migration safety checks**
- Ensure no production runtime behavior changes in:
  - `src/runtime/node.rs`
- This task is test-harness/e2e orchestration refactor, not runtime logic rewrite.

10. **Evidence and final verification**
- Record interface comparison artifact + final chosen API summary.
- Record before/after duplication reduction (function count/LOC) for moved helpers.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Interface research artifact compares at least 3 candidate interfaces (A/B/C above or better) with explicit scoring and final choice rationale
- [ ] Task honors user requirement: one stable shared interface based on `TestConfig`; if research finds a better higher-reuse interface, that interface is implemented instead
- [ ] New shared module(s) added under `src/test_harness/ha_e2e/` and exported from `src/test_harness/mod.rs`
- [ ] Duplicated helper logic removed from both:
- [ ] `src/ha/e2e_multi_node.rs` helper block (`1615-2164`) migrated to shared harness or reduced to thin wrappers
- [ ] `src/ha/e2e_partition_chaos.rs` helper block (`886-1248`) migrated to shared harness or reduced to thin wrappers
- [ ] Startup interface in both suites now uses the same `TestConfig`-driven entrypoint:
- [ ] `src/ha/e2e_multi_node.rs` no longer owns bespoke startup graph from `293-495`
- [ ] `src/ha/e2e_partition_chaos.rs` no longer owns bespoke startup graph from `72-363`
- [ ] Shared return handle includes everything required by scenario code (node metadata, clients, task handles, etcd/proxy handles where applicable, and unified ops)
- [ ] Fault-injection behavior remains available for partition scenarios via the shared interface (proxy-backed mode)
- [ ] Scenario behavior parity retained for all HA e2e tests:
- [ ] `e2e_multi_node_*` tests preserve semantics and assertions
- [ ] `e2e_partition_*` tests preserve semantics and assertions
- [ ] Policy guard still passes or is updated minimally with justified equivalence:
- [ ] `tests/policy_e2e_api_only.rs` remains semantically aligned with post-start hands-off rules
- [ ] New harness-level tests cover `TestConfig` validation/defaults and handle lifecycle
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
