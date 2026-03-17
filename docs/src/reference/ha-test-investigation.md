# HA Docker Test Harness Investigation Report

## Scope

This report focuses on the `tests/ha` docker-based HA suite with two goals:

1. Identify opportunities to reduce duplicated state/types and collapse unnecessary conversion/build layers.
2. Identify likely root causes of observed flakiness in the HA runner and support stack.

Primary files reviewed:

- `tests/ha/support/runner/mod.rs`
- `tests/ha/support/world/mod.rs`
- `tests/ha/support/steps/mod.rs`
- `tests/ha/support/docker/{cli,ryuk}.rs`
- `tests/ha/support/{observer,workload,timeouts}/**`

---

## How the HA docker tests work end-to-end

1. **Feature layer (`tests/ha/features/*.feature`)**
   - Cucumber scenarios express failover/switchover and fault-injection behavior.

2. **Runner + world lifecycle (`tests/ha/support/runner/mod.rs`, `world/mod.rs`)**
   - Materializes per-scenario assets and compose files.
   - Boots DCS + DB services with docker compose.
   - Tracks aliases, markers, stopped/wedged members, workload state, and invariant history.
   - Collects artifacts (`compose ps`, logs, timeline snapshots) on teardown.

3. **Step layer (`tests/ha/support/steps/mod.rs`)**
   - Executes high-level Given/When/Then logic.
   - Polls pgtm state until expected authority/publication/member conditions converge.
   - Uses SQL probes and proof-table rows to assert write/read consistency.

4. **Support layer**
   - `observer/pgtm.rs`: pgtm command observer + JSON decode.
   - `docker/cli.rs`: compose/up/down/inspect/exec wrapper.
   - `docker/ryuk.rs`: sidecar cleanup guard.
   - `faults/mod.rs`: iptables and process fault orchestration.
   - `workload/mod.rs`: write workload loop and event capture.

---

## Deduplication and type reuse findings

### 1) Duplicate DSN materialization in tests

`tests/ha/support/steps/mod.rs` previously had a test-local `direct_sql_dsn(...)` string builder that duplicated connection DSN logic already present in `src/command/mod.rs` (`materialize_connection_dsn` + `LocalConnectionMaterialization`).

**Smell:** test helpers drift from production command rendering rules over time.

**Improvement made:** steps now build direct SQL DSNs with source types and source DSN materializer.

### 2) Wrapper-heavy intermediate state remains

`tests/ha/support/world/mod.rs` includes several single-field wrappers (`MemberAlias`, `MarkerName`, `ProofRow`, `ProofTableName`) and a `MemberSet` wrapper around `BTreeSet<ClusterMember>`.

These wrappers provide semantic labeling, but they also increase conversion surfaces and boilerplate methods. This is still a refactor candidate if we decide to aggressively collapse state to fewer canonical structs/enums.

### 3) Multiple target-construction pathways in steps

`steps/mod.rs` still has several paths for constructing SQL execution targets (observer primary/replicas, alias-based direct target, member direct target). Most are justified by scenario semantics, but some could be collapsed into a single state-driven resolver that returns one unified target type.

---

## Flakiness investigation (deep)

### A) Fixed cleanup sleep in Ryuk teardown (high impact)

`tests/ha/support/docker/ryuk.rs` previously paused a hard-coded `2s` before forcing Ryuk removal. Fixed sleeps are timing assumptions and are brittle under slow CI or daemon jitter.

**Observed risk:** nondeterministic cleanup timing between scenarios, especially when daemon load spikes.

**Improvement made:** replaced fixed sleep with bounded retry removal loop (deadline + small retry interval), distinguishing:

- already-completed removal (`no such container`) => success
- transient removal-in-progress => retry
- other errors => fail fast

This shifts cleanup from time-based guessing to state-based completion.

### B) Poll loop stress from repeated container terminal checks

`poll_for_status(...)` invokes terminal container checks during status polling. This is useful for early crash detection but adds inspect pressure and may surface transient Docker lookup errors as hard failures if not handled carefully.

**Current status:** still a valid follow-up tuning area if flakiness persists after cleanup improvements.

### C) High state surface area in `HaWorld`

`ScenarioState` combines aliases, transitions, workload proof, command output, and invariants. The broad mutable surface can magnify timing/order coupling during long scenarios.

**Current status:** not changed in this patch to keep changes surgical, but remains a prime quality target.

---

## Large opportunities for future consolidation

1. **Unify member condition tracking**
   - Replace multiple member sets (`stopped`, `wedged`, `observer_unreachable`, `convergence_blocked`) with one typed member-state map.

2. **Collapse SQL target resolution**
   - Keep one canonical target resolver API for step logic; route all direct/member/alias variants through it.

3. **Reduce conversion layers in proof-table flow**
   - Consider minimizing custom wrapper rows/table-name types where they do not carry invariants.

---

## Changes implemented in this investigation pass

1. **Type reuse from `src` in tests**
   - Removed test-local direct DSN string formatter.
   - Reused `pgtuskmaster_rust::command::{LocalConnectionMaterialization, PathBackedClientTlsDto}` and `materialize_connection_dsn(...)`.

2. **Flakiness fix in teardown**
   - Removed hard-coded `2s` resource cleanup sleep.
   - Added bounded retry teardown behavior in Ryuk close path.
   - Added unit tests for Ryuk removal error classification.

---

## Suggested next steps (prioritized)

1. **Run repeated HA suite in CI soak mode** (same commit, many repetitions) to confirm cleanup-flake reduction.
2. **Refactor member-state tracking to one canonical state model** in `HaWorld`.
3. **Finish SQL target builder consolidation into one path** to reduce helper drift and simplify step assertions (this pass removed duplicate direct-DSN formatting, but multiple target selection paths still remain).
