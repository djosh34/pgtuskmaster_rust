---
## Task: Unify HA E2E Harness Behind Stable `TestConfig` Interface <status>completed</status> <passes>true</passes>

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
0. **Research gate (must be done before code edits)**
- Create an interface-comparison artifact at:
  - `.ralph/evidence/task-37-unified-e2e-harness-testconfig-interface/interface-comparison.md`
- Compare at least 3 interfaces and score them (1–5) across:
  - reuse across future suites
  - readability of scenario code
  - policy compatibility (post-start hands-off)
  - migration churn risk
  - tokio constraints (current_thread + LocalSet + non-Send join handles)
  - type/system complexity (trait-heavy vs concrete handle)
- Candidates to compare (minimum):
  - **A**: monolithic `run_ha_scenario(TestConfig, scenario_fn)` wrapper
  - **B**: trait-heavy `trait HaEnv` / generic scenario layer
  - **C**: config-driven `start_cluster(TestConfig) -> TestClusterHandle` (baseline)
- Include a concrete “mapping table” from today’s fixtures:
  - multi-node startup: `src/ha/e2e_multi_node.rs:293`
  - partition startup: `src/ha/e2e_partition_chaos.rs:72`
- Decision rule: adopt a superior interface if it is materially higher reuse + lower churn than C; otherwise commit to C and treat it as the stable API.

0.5 **Skeptical “policy-safe naming” contract (do before migrating any e2e file)**
- `tests/policy_e2e_api_only.rs` uses raw substring scanning of the full *source text* for `src/ha/e2e_*.rs`, so forbidden tokens can be introduced accidentally via:
  - helper names
  - import paths / aliases
  - comments
  - string literals / URLs
- Establish a hard rule for this task:
  - keep all “risky” names and strings in `src/test_harness/ha_e2e/*`, and expose only neutral wrapper names in `src/ha/e2e_*.rs`
  - preserve existing scenario-facing callsites where possible, especially `.run_sql_on_node(` and `.run_sql_on_node_with_retry(`
- Add a fast feedback loop:
  - after each migration (multi-node first, partition second), run `cargo test --test policy_e2e_api_only` immediately to catch lexical regressions early.

1. **Lock the stable public harness API surface (low churn)**
- Implement interface **C** unless research clearly overturns it:
  - `TestConfig` as the single entrypoint input
  - `TestClusterHandle` as the single returned handle
  - nested config blocks to match existing harness style (`*Spec` + `*Handle`)
- Keep scenario-facing method names close to existing fixture methods to minimize refactors and preserve policy-safe tokens in `src/ha/e2e_*.rs`:
  - `get_ha_state` polling (plus TCP fallback when HTTP transport errors occur)
  - `run_sql_on_node`, `run_sql_on_node_with_retry`
  - `wait_for_bootstrap_primary`, `wait_for_stable_primary`, `assert_no_dual_primary_window`
  - switchover via CLI/API surface (no direct DCS mutation)
  - fault injection only via external process/network operations

2. **Add shared module family under `src/test_harness/ha_e2e/`**
- Export module from `src/test_harness/mod.rs` (near the module list at `src/test_harness/mod.rs:8`).
- Add modules (exact names may be adjusted if research demands, but keep responsibilities stable):
  - `src/test_harness/ha_e2e/mod.rs` (top-level API + reexports)
  - `src/test_harness/ha_e2e/config.rs` (`TestConfig`, `Mode`, defaults, validation)
  - `src/test_harness/ha_e2e/handle.rs` (`TestClusterHandle`, node descriptors, timelines)
  - `src/test_harness/ha_e2e/startup.rs` (build ports, etcd, proxies, runtime tasks)
  - `src/test_harness/ha_e2e/ops.rs` (HA observation, waits, SQL helpers)
  - `src/test_harness/ha_e2e/faults.rs` (proxy-backed faults + no-op/unsupported behavior)
  - `src/test_harness/ha_e2e/util.rs` (shared helpers: log tail, child wait/kill, local-set runner, psql parsing, unix time)
- **Runtime boundary contract (non-negotiable):**
  - shared startup must always run within a `tokio::task::LocalSet` because runtime tasks use `spawn_local`
  - keep `TestClusterHandle` intentionally non-`Send`/non-`Sync` (avoid `tokio::spawn` APIs that would force `Send + 'static`)
  - preserve `TcpProxyLink`’s dedicated-thread runtime model (do not move proxy listeners onto the test’s current-thread runtime)
  - provide a single canonical entry helper (likely `ha_e2e::run_with_local_set`) and make both suites use it.

3. **Centralize duplicated helper blocks verbatim first (reduce risk)**
- Port duplicated helpers into `ha_e2e::util` with minimal edits (error handling preserved; no `unwrap`/`expect`/`panic` additions).
- Source anchors to consolidate:
  - multi-node helpers: `src/ha/e2e_multi_node.rs:1615` and below
  - partition helpers: `src/ha/e2e_partition_chaos.rs:903` and below
- Canonical helpers to unify:
  - `wait_for_node_api_ready_or_task_exit` + `read_log_tail`
  - `wait_for_bootstrap_primary`
  - `wait_for_child_exit_with_timeout` + pg_ctl stop helpers
  - `run_psql_statement` + parsers (`parse_psql_rows`, `parse_single_u64`)
  - `unix_now`
  - `run_with_local_set` (required because startup uses `spawn_local`)

4. **Implement transactional startup (`TestConfig -> TestClusterHandle`)**
- Add a cross-resource `StartupGuard` (or similarly named) in `ha_e2e::startup` that records owned resources as they come up and guarantees best-effort rollback on error:
  - etcd handle(s)
  - proxy links (if enabled)
  - runtime task join handles / child processes
  - any namespace paths created
- On startup failure: perform reverse-order cleanup and return an error that includes both the startup failure and any cleanup failure details.
- On success: convert guard into `TestClusterHandle` via `into_handle()` to prevent double-shutdown and keep lifecycle explicit.
- Reuse existing harness primitives (no reinvention):
  - etcd: `spawn_etcd3_cluster` (`src/test_harness/etcd3.rs:188`)
  - postgres data dir: `prepare_pgdata_dir` (`src/test_harness/pg16.rs:57`)
  - proxy links: `TcpProxyLink` (`src/test_harness/net_proxy.rs:55`)
  - binary resolution: `require_pg16_bin_for_real_tests`, `require_etcd_bin_for_real_tests` (`src/test_harness/binaries.rs`)
- Support two startup modes via `TestConfig.mode`:
  - `Plain`: direct endpoints and addresses (like current multi-node)
  - `PartitionProxy`: per-node proxy topology (like current partition chaos)
    - rewrite each node’s DCS endpoints to proxy URLs at startup (no runtime config mutation post-boot)
    - maintain `BTreeMap<String, TcpProxyLink>` for `etcd_proxies`, `api_proxies`, `pg_proxies`
- Preserve layered timeout model from current suites (keep defaults identical to existing constants unless explicitly overridden by config).
- Add up-front validation in `TestConfig` (fail early before spawning anything):
  - unique node IDs / unique etcd member names
  - mode-specific requirements fully satisfied
  - artifact/log root path strategy is explicit and deterministic
  - all postgres data dirs go through `prepare_pgdata_dir` (permissions invariants)

5. **Provide high-level operations on `TestClusterHandle`**
- Unify common operations currently duplicated:
  - HA observation + polling:
    - multi-node: `src/ha/e2e_multi_node.rs:1291`
    - partition: `src/ha/e2e_partition_chaos.rs:457`
  - Waits/assertions:
    - `wait_for_stable_primary` / `assert_no_dual_primary_window` (stability-based, not one-shot leader reads)
  - SQL helpers + retry loops:
    - multi-node: `src/ha/e2e_multi_node.rs:529` and `:880`
    - partition: `src/ha/e2e_partition_chaos.rs:614`
- Provide compatibility helpers to avoid scenario code reaching into internal vectors/maps:
  - `node_ids()` / `node_ids_except(primary_id)`
  - `api_addr(node_id)` / `pg_port(node_id)` accessors
- Keep scenario-specific multi-node stress workload logic local unless it becomes a third consumer.

6. **Migrate `src/ha/e2e_multi_node.rs` to shared harness**
- Replace bespoke `ClusterFixture` startup graph with `TestConfig` + `TestClusterHandle`:
  - map `node_count`, timeouts, artifact dir, and runtime defaults into `TestConfig`
  - ensure startup still uses `crate::runtime::run_node_from_config` unchanged
- Replace duplicated helper block (`1615+`) with imports from `crate::test_harness::ha_e2e`.
- Remove direct `fixture.nodes` iteration by using `handle.node_ids_except(...)`.
- Keep scenario behavior and assertions identical.

7. **Migrate `src/ha/e2e_partition_chaos.rs` to shared harness**
- Replace bespoke `PartitionFixture` startup graph with `TestConfig(mode=PartitionProxy)` + `TestClusterHandle`.
- Preserve fault-injection semantics exactly:
  - `partition_node_from_etcd` / `partition_primary_from_etcd`: block etcd link(s)
  - `isolate_api_path`: block only API proxy
  - `heal_all_network_faults`: restore all proxy modes to pass-through
- Replace duplicated helper block (`903+`) with shared harness methods.

8. **Policy compatibility verification**
- Ensure `src/ha/e2e_*.rs` do not gain forbidden lexical tokens from `tests/policy_e2e_api_only.rs:3` (including via comments/strings/import aliases).
- Keep post-start control paths limited to:
  - GET `/ha/state` observation (plus TCP fallback)
  - switchover via CLI/API surface
  - SQL reads/writes for scenario intent
  - external process/network fault injection only
- Update policy lists only if strictly required and justified by semantic equivalence.
- Run `cargo test --test policy_e2e_api_only` immediately after each e2e file migration (before running the full `make` gates).

9. **Harness-level tests (contract + lifecycle)**
- Add new tests under `src/test_harness/ha_e2e/*`:
  - `TestConfig` validation + defaults (pure unit tests)
- Add at least one regression test that exercises the **LocalSet boundary** explicitly:
  - `#[tokio::test(flavor = "current_thread")]` + `ha_e2e::run_with_local_set(...)` + start + shutdown
- Optional if it stays stable and does not duplicate coverage excessively:
  - a minimal partition-proxy-mode start/shutdown smoke test (proxy-heavy tests can be flaky if startup leaks resources; only keep if it remains deterministic under `RUST_TEST_THREADS=1`)
- Tests must not be optional/skipped; missing binaries should fail loudly with actionable errors (use existing `require_*_for_real_tests` helpers).

10. **Evidence + final verification**
- Record artifacts under `.ralph/evidence/task-37-unified-e2e-harness-testconfig-interface/`:
  - interface comparison + final API summary
  - duplication reduction metrics (LOC / function count)
  - logs for `make check`, `make test`, `make lint` (100% green required)

**Execution note:** `start_cluster` must run under a `LocalSet` (startup spawns runtime tasks via `spawn_local`); keep a shared `run_with_local_set` helper in `ha_e2e::util` and ensure both suites use it consistently.
</description>

<acceptance_criteria>
- [x] Interface research artifact compares at least 3 candidate interfaces (A/B/C above or better) with explicit scoring and final choice rationale
- [x] Task honors user requirement: one stable shared interface based on `TestConfig`; if research finds a better higher-reuse interface, that interface is implemented instead
- [x] New shared module(s) added under `src/test_harness/ha_e2e/` and exported from `src/test_harness/mod.rs`
- [x] Duplicated helper logic removed from both:
- [x] `src/ha/e2e_multi_node.rs` helper block (`1615-2164`) migrated to shared harness or reduced to thin wrappers
- [x] `src/ha/e2e_partition_chaos.rs` helper block (`886-1248`) migrated to shared harness or reduced to thin wrappers
- [x] Startup interface in both suites now uses the same `TestConfig`-driven entrypoint:
- [x] `src/ha/e2e_multi_node.rs` no longer owns bespoke startup graph from `293-495`
- [x] `src/ha/e2e_partition_chaos.rs` no longer owns bespoke startup graph from `72-363`
- [x] Shared return handle includes everything required by scenario code (node metadata, clients, task handles, etcd/proxy handles where applicable, and unified ops)
- [x] Fault-injection behavior remains available for partition scenarios via the shared interface (proxy-backed mode)
- [x] Scenario behavior parity retained for all HA e2e tests:
- [x] `e2e_multi_node_*` tests preserve semantics and assertions
- [x] `e2e_partition_*` tests preserve semantics and assertions
- [x] Policy guard still passes or is updated minimally with justified equivalence:
- [x] `tests/policy_e2e_api_only.rs` remains semantically aligned with post-start hands-off rules
- [x] New harness-level tests cover `TestConfig` validation/defaults and handle lifecycle
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

NOW EXECUTE
