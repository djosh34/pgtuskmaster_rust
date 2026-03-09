## Task: Add Clone And Rewind Failure HA E2E Coverage <status>completed</status> <passes>true</passes>

<priority>low</priority>

<description>
**Goal:** Add explicit HA behavioural coverage for failure paths around `pg_basebackup` clone/startup and `pg_rewind` rejoin. The higher-order goal is to validate not only the happy-path recovery machinery, but also the cases where the cluster cannot successfully clone or rewind a node and must fail safely, surface the problem, and recover cleanly once the fault is removed.

This task should treat clone and rewind failures as first-class operational scenarios, not as rare internal implementation details. These are exactly the places where orchestration often looks correct in clean demos and then breaks under real conditions.

**Scope:**
- Identify the current hooks for making clone and rewind fail in the HA harness or process dispatch layer.
- Add focused behavioural scenarios that force:
- initial clone/basebackup failure for a joining or rejoining replica
- rewind failure for a former primary that needs to rejoin after failover
- eventual recovery once the injected failure is removed, if the scenario is meant to test retry/recovery
- Ensure the tests assert safe cluster behaviour while the clone or rewind step is failing:
- no dual-primary evidence
- no false declaration of a healthy replica when clone/rewind did not complete
- existing primary remains authoritative and writable when expected
- recovered node converges correctly after the fault is cleared
- Keep the scope strictly on node-management/HA paths, not CLI.

**Context from research:**
- The current suite proves that a former primary can rejoin after failover in the happy path.
- There is no current behavioural coverage for `pg_rewind` failure or `pg_basebackup` failure.
- These failure modes were explicitly called out during research as meaningful testing gaps.
- The user wants more HA failure tests and wants them specified precisely.

**Expected outcome:**
- The suite stops assuming clone and rewind always succeed.
- HA orchestration is validated under realistic rejoin failure conditions.
- Recovery logic becomes safer because the failure path is now executable and asserted.

</description>

<acceptance_criteria>
- [x] Identify the exact harness or subprocess hooks needed to inject deterministic `pg_basebackup` and `pg_rewind` failures without relying on flaky timing.
- [x] Add at least one new scenario for clone/basebackup failure and at least one new scenario for rewind failure.
- [x] Each scenario explicitly verifies that the cluster remains safe while the operation is failing and does not falsely report successful convergence.
- [x] Each scenario explicitly verifies no dual-primary evidence during the failure window.
- [x] Where recovery-after-fix is part of the scenario, the test verifies the node eventually rejoins correctly and data converges afterward.
- [x] If a failure is meant to remain unrecovered, the scenario verifies that the cluster stabilizes in the expected degraded state instead of oscillating silently.
- [x] The implementation uses deterministic failure injection rather than probabilistic shell hacks or sleeps that make the test fragile.
- [x] The scenarios remain parallel-safe through isolated namespaces, ports, data dirs, and artifact names.
- [x] Add any needed helper abstractions in the harness or process-dispatch test support so these failure modes are easy to reason about and maintain.
- [x] The implementation does not touch CLI code.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If the new scenarios belong in the long-running gate: `make test-long` — passes cleanly
</acceptance_criteria>

## Execution Plan

### Planning notes locked in before execution

- The closest existing happy-path coverage is `e2e_multi_node_custom_postgres_role_names_survive_bootstrap_and_rewind` in `tests/ha/support/multi_node.rs`. That scenario already proves a former primary can fail over and rejoin successfully, so it is the right template for the new rewind-failure path.
- The HA harness does not currently expose any per-node recovery-binary override seam. `src/test_harness/ha_e2e/config.rs` only configures cluster shape, timeouts, and role overrides, while `src/test_harness/ha_e2e/startup.rs` always loads one verified `BinaryPaths` set and writes those paths into every node runtime config and DCS init payload.
- Skeptical review correction: this task should not introduce a broad per-node replacement for the full `BinaryPaths` struct. The failure scenarios only need to redirect `pg_basebackup` and `pg_rewind`, so the harness seam should stay narrow and merge optional per-node overrides for those recovery binaries onto the existing verified shared binary set.
- The recovery semantics are already defined in production logic and should drive the E2E expectations instead of inventing new behavior:
  - `pg_rewind` failure in `src/ha/decide.rs` transitions from `Rewinding` to `Bootstrapping` with `RecoveryStrategy::BaseBackup`.
  - basebackup failure in `Bootstrapping` transitions to `Fencing`.
  - after fencing, `WaitingDcsTrusted` with a known leader re-enters basebackup recovery.
- That means the cleanest coverage split is:
  - one scenario that forces startup-time or join-time basebackup failure and then proves safe degraded behavior plus eventual recovery after the fault is removed,
  - one scenario that forces rewind failure on a former primary and proves safe failover plus eventual rejoin through the existing basebackup fallback path.
- Deterministic failure injection should use explicit wrapper binaries and marker files, not sleeps or timing races. The wrapper scripts should live under the test namespace so they remain parallel-safe and disposable.
- The task must stay in HA/test-harness scope only. No CLI production code should change.
- There is no `update-docs` skill available in this session. Execution must therefore update the relevant docs directly in-repo.

### Phase 1: Add a deterministic per-node recovery-binary override seam to the HA E2E harness

- [x] Extend `src/test_harness/ha_e2e/config.rs` so `TestConfig` can optionally carry per-node recovery-binary overrides for just `pg_basebackup` and `pg_rewind`, instead of forcing one shared immutable recovery-binary selection for the whole cluster.
- [x] Keep the default path unchanged when no overrides are provided:
  - [x] continue to load the verified real PostgreSQL binaries through the existing harness tooling,
  - [x] only substitute overridden recovery-binary paths for the specific node ids named by the test,
  - [x] leave `postgres`, `pg_ctl`, `initdb`, and `psql` on the verified shared defaults.
- [x] Validate any override paths up front so bad test inputs fail clearly:
  - [x] the override path must be non-empty,
  - [x] the override path must be absolute,
  - [x] the override path must point to an executable file.
- [x] Reuse `src/test_harness/binaries.rs` executable validation helpers instead of duplicating file-mode checks in the HA config layer.
- [x] Thread the chosen per-node `BinaryPaths` through `src/test_harness/ha_e2e/startup.rs` in both places that matter:
  - [x] the runtime `ProcessConfig` passed into `run_node_from_config(...)`,
  - [x] the DCS init payload JSON written for that node.
- [x] Preserve the existing restart behavior in `src/test_harness/ha_e2e/handle.rs` by ensuring the stored per-node runtime config already contains the overridden paths, so restarting a node keeps the same wrapper-binary behavior without extra special cases.

### Phase 2: Add reusable deterministic failure-wrapper helpers

- [x] Add a small helper in the HA test support layer that can create executable wrapper scripts inside the namespace-owned working directory for a specific node and binary kind.
- [x] The helper should support a switchable failure mode driven by marker files:
  - [x] while a `fail-enabled` marker exists, the wrapper exits non-zero and records that it ran,
  - [x] once the marker is removed, the wrapper `exec`s the real binary path,
  - [x] each invocation appends a small trace line or touches a counter marker so the scenario can prove the failure path actually executed.
- [x] Keep the wrappers targeted and absolute-path based:
  - [x] override `pg_basebackup` only for the designated clone-failure node,
  - [x] override `pg_rewind` only for the designated rewind-failure node,
  - [x] do not rely on `PATH` shadowing.
- [x] Reuse existing executable-file helpers where possible so the new wrapper creation path does not duplicate low-level permission logic more than necessary.

### Phase 3: Extend the multi-node fixture with failure-aware helpers instead of open-coding scenario plumbing

- [x] Extend `tests/ha/support/multi_node.rs` with a small set of helpers dedicated to these scenarios.
- [x] Add fixture support for building a `TestConfig` that includes node-specific binary overrides and scenario-specific artifact names.
- [x] Add helper(s) to manage the wrapper markers during a scenario:
  - [x] enable a failure,
  - [x] disable a failure,
  - [x] assert that the wrapper was actually invoked at least once.
- [x] Add helper(s) to assert safe degraded behavior while a node is failing clone or rewind:
  - [x] exactly one SQL-visible primary remains writable,
  - [x] the failing node is not treated as a healthy converged replica,
  - [x] the healthy leader path continues to accept proof writes when expected,
  - [x] no dual-primary evidence appears over an observation window.
- [x] Add helper(s) to wait for a recovering node to reach the intended intermediate and final states:
  - [x] for rewind failure, wait until the former primary stops being primary and eventually rejoins as a replica after fallback,
  - [x] for basebackup failure, wait until the broken node remains non-queryable or non-converged during the fault window, then eventually becomes queryable and catches up after the blocker is removed.
- [x] Reuse existing helpers wherever possible:
  - [x] stable-primary wait helpers,
  - [x] proof-table helpers,
  - [x] `assert_former_primary_demoted_or_unreachable_after_transition(...)`,
  - [x] no-dual-primary observation helpers from `tests/ha/support/observer.rs`.

### Phase 4: Add a clone/basebackup failure scenario with recovery after fault removal

- [x] Add a new multi-node E2E scenario in `tests/ha/support/multi_node.rs` and register it in `tests/ha_multi_node_failover.rs`.
- [x] Use a 3-node plain-mode cluster so the scenario can distinguish:
  - [x] one stable primary,
  - [x] one healthy replica that still converges normally,
  - [x] one replica whose `pg_basebackup` path is deliberately failing.
- [x] Planned scenario flow:
  - [x] create a wrapper for one replica's `pg_basebackup` binary before cluster startup and start the cluster with that node-specific override enabled,
  - [x] wait for bootstrap primary election and for the non-failing replica to become queryable,
  - [x] create a proof table and verify writes still replicate across the healthy nodes,
  - [x] assert during the failure window that the cluster stays safe:
    - [x] no dual-primary evidence,
    - [x] only one writable primary,
    - [x] the broken replica is not falsely treated as a converged healthy replica,
    - [x] the cluster does not silently oscillate into an unsafe state,
  - [x] prove the wrapper was invoked so the scenario is not accidentally passing on the happy path,
  - [x] remove the `pg_basebackup` failure marker,
  - [x] wait for the failed replica to retry recovery through the existing HA flow and eventually rejoin,
  - [x] write a post-recovery proof row on the current primary and verify final row convergence across all nodes.
- [x] Prefer assertions based on concrete data convergence and queryability rather than only phase names, because transient phase names can vary while the safety requirements remain stable.
- [x] If the actual recovery path observed during execution shows a required intermediate fencing cycle, keep that explicit in the final assertions instead of hiding it.

### Phase 5: Add a rewind failure scenario that proves safe failover and basebackup fallback

- [x] Add a second multi-node E2E scenario in `tests/ha/support/multi_node.rs` and register it in `tests/ha_multi_node_failover.rs`.
- [x] Override `pg_rewind` only on `node-1`, because the current harness startup order currently waits for `node-1` to become bootstrap primary before bringing up later nodes.
- [x] Planned scenario flow:
  - [x] start a 3-node cluster with `node-1` using a failing `pg_rewind` wrapper and all other binaries left real,
  - [x] wait for a stable bootstrap primary and explicitly assert it is still `node-1`; if that invariant ever stops holding, fail the scenario immediately instead of silently testing the wrong node,
  - [x] seed a proof table replicated to the healthy replicas,
  - [x] stop PostgreSQL on the bootstrap primary to force failover,
  - [x] wait for a new stable primary that differs from the original leader,
  - [x] assert no dual-primary evidence through the failover and failed-rewind window,
  - [x] assert the former primary is demoted or unreachable and is not falsely reported as a healthy replica while rewind is still failing,
  - [x] prove the `pg_rewind` wrapper was invoked,
  - [x] wait for the system to follow its existing designed fallback from rewind failure into basebackup recovery,
  - [x] wait for the former primary to become queryable again and catch up with post-failover proof rows,
  - [x] verify final convergence across all nodes.
- [x] Do not design this scenario around a rewind retry after the fault is cleared, because the current decision engine already models rewind failure as a transition to basebackup, not as a rewind retry loop.
- [x] Make the assertions strong enough to prove the failure path was real:
  - [x] former primary does not immediately rejoin on the happy rewind path,
  - [x] wrapper evidence shows `pg_rewind` was attempted and failed,
  - [x] eventual recovery happens through the cluster's defined fallback behavior, not through accidental timing luck.

### Phase 6: Keep the new coverage parallel-safe and easy to diagnose

- [x] Give each scenario unique names, wrapper paths, marker files, proof tables, and timeline artifact names derived from the existing per-test token utilities.
- [x] Keep all wrapper scripts and marker files inside the namespace-owned directories so parallel runs cannot collide.
- [x] Record timeline entries for:
  - [x] when the failure wrapper is enabled,
  - [x] when the cluster first shows the degraded state,
  - [x] when the failure marker is removed for the basebackup-recovery scenario,
  - [x] when fallback or final rejoin is observed.
- [x] If execution reveals any repetitive assertion boilerplate, extract helpers in `tests/ha/support/multi_node.rs` instead of duplicating long polling loops inside the scenarios.

### Phase 7: Docs and verification

- [x] Update the docs to reflect that the long HA suite now exercises explicit clone/basebackup and rewind failure paths.
- [x] Minimum expected docs touch:
  - [x] `docs/src/how-to/run-tests.md` so it names the new HA failure coverage in the `make test-long` description,
  - [x] if needed after implementation, `docs/src/how-to/handle-primary-failure.md` or another HA operations page only where the new fallback/failure-path wording would otherwise stay stale.
- [x] Run the full required verification sequence after implementation:
  - [x] `make check`
  - [x] `make test`
  - [x] `make test-long`
  - [x] `make lint`
- [x] Only after all gates pass:
  - [x] mark the acceptance criteria and execution-plan checkboxes complete,
  - [x] set `<passes>true</passes>`,
  - [x] run `/bin/bash .ralph/task_switch.sh`,
  - [x] commit all tracked changes including `.ralph` state with the required `task finished [task name]: ...` message and explicit test evidence,
  - [x] push with `git push`.

EXECUTED
