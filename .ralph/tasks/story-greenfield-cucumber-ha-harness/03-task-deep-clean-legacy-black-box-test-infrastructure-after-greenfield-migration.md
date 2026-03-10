## Task: Deep Clean Legacy Black-Box Test Infrastructure After Greenfield Migration <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Remove the legacy black-box HA/E2E/half-BDD test infrastructure once the greenfield cucumber Docker suite has replacement coverage for the migratable scenarios. This task is not about adding new behavior coverage. It is about deleting the old duplicated paths, collapsing the repo onto one black-box testing architecture, and keeping only genuine deep-control / unit-contract tests outside the new cucumber harness.

**Original user shift / motivation:** The user wants to get rid of the current overcomplicated `ha`, `e2e`, and half-BDD mix. Their rule is explicit: tests that do not require deep unit-test-level control, and that can be tested by running real `pgtuskmaster` binaries and then controlling/observing them through the new Docker-based harness and `pgtm`, are eligible for migration and should be removed from the old infrastructure afterward. This task must therefore perform a full cleanup pass and must not leave duplicated legacy copies lingering by accident.

**Higher-order goal:** Converge the repository on a clear test architecture:
- greenfield Docker cucumber tests for black-box operator-visible behavior
- source/unit/contract tests only for deep implementation seams and fine-grained internal control
- no second legacy black-box harness surviving out of inertia

**Scope:**
- Audit and remove duplicated legacy black-box scenario coverage after tasks 01, 02, and 04 land.
- Remove or drastically shrink the legacy HA E2E harness rooted in:
- `tests/ha/support/`
- `tests/ha_multi_node_failover.rs`
- `tests/ha_partition_isolation.rs`
- `src/test_harness/ha_e2e/`
- `src/test_harness/net_proxy.rs` when no longer needed by retained non-migrated tests
- Remove old “policy around the legacy harness” files once the harness itself is gone, especially `tests/policy_e2e_api_only.rs`.
- Audit mixed black-box test files such as `tests/bdd_api_http.rs` and `tests/cli_binary.rs` and remove any scenarios that have been migrated into the greenfield Docker-based acceptance path. Retain only true unit/contract-scale tests that still need local stubs or non-Docker synthetic setup.
- Update `Cargo.toml`, `Makefile`, `.config/nextest.toml`, `tests/nextest_config_contract.rs`, and docs so the greenfield suite is the documented black-box path and stale legacy references are removed.
- This task must explicitly decide the fate of every old black-box scenario discovered during research. A scenario must end this task in exactly one of these states:
- migrated to greenfield and legacy copy removed
- explicitly retained as non-migratable deep-control coverage, with written justification
- split into a named follow-up if it truly still lacks the needed greenfield support

**Context from research:**
- The legacy black-box HA surface currently lives in:
- `tests/ha/support/multi_node.rs`
- `tests/ha/support/observer.rs`
- `tests/ha/support/partition.rs`
- `tests/ha_multi_node_failover.rs`
- `tests/ha_partition_isolation.rs`
- `src/test_harness/ha_e2e/config.rs`
- `src/test_harness/ha_e2e/handle.rs`
- `src/test_harness/ha_e2e/mod.rs`
- `src/test_harness/ha_e2e/ops.rs`
- `src/test_harness/ha_e2e/startup.rs`
- `src/test_harness/ha_e2e/util.rs`
- The old black-box scenario set includes, among others:
- unassisted failover / SQL continuity
- planned switchover and targeted switchover
- concurrent workload switchover and failover
- clone / rewind failure recovery
- promotion-choice / degraded replica cases
- no-quorum fail-safe and fencing cases
- partition isolation and mixed-fault cases
- The old policy test `tests/policy_e2e_api_only.rs` exists purely to police the old harness boundary and becomes stale once that boundary is deleted.
- Not everything should move into greenfield cucumber. The user explicitly preserved the distinction that tests needing deep unit-test-level control are allowed to remain elsewhere. Current examples that look like retained deep-control / contract coverage are:
- `src/worker_contract_tests.rs`
- `tests/bdd_state_watch.rs`
- parser/config/exit-code slices of `tests/cli_binary.rs`
- any truly internal worker/API contract checks that still rely on stubs rather than real runtime orchestration
- The old runtime-only restart scenario `e2e_multi_node_primary_runtime_restart_recovers_without_split_brain` may or may not be migrated by task 04. This task must not leave it sitting in the old harness by default. It must either be migrated, explicitly retained as deep-control coverage, or split into a named follow-up with justification.

**Expected outcome:**
- The repo no longer carries two parallel black-box HA test architectures.
- Legacy black-box test wrappers, support helpers, and stale policy glue are removed once greenfield replacements exist.
- Mixed files no longer hide migrated black-box behavior next to true contract tests; the remaining tests have a defensible reason to exist outside cucumber.

</description>

<acceptance_criteria>
- [ ] Every black-box scenario migrated by tasks 01, 02, and 04 has its legacy counterpart removed from the old harness; no duplicate black-box scenario remains accidentally in `tests/ha_*`, `tests/ha/support/`, or `src/test_harness/ha_e2e/`.
- [ ] `tests/ha_multi_node_failover.rs`, `tests/ha_partition_isolation.rs`, and `tests/ha/support/` are deleted entirely once their migrated coverage exists, unless a file contains an explicitly justified retained deep-control test that cannot honestly move to the greenfield harness.
- [ ] Obsolete legacy harness code under `src/test_harness/ha_e2e/` is deleted, and repo-wide verification confirms no stale imports or call sites remain.
- [ ] `src/test_harness/net_proxy.rs` is deleted if no retained non-migrated consumer still needs it; if retained, the task documents exactly which non-greenfield test still depends on it and why.
- [ ] `tests/policy_e2e_api_only.rs` is removed because the legacy harness boundary it policed no longer exists after cleanup.
- [ ] `tests/bdd_api_http.rs` and `tests/cli_binary.rs` are audited test-by-test; migrated black-box stories are removed or relocated out of those files, and only true unit/contract-scale tests remain there afterward.
- [ ] The fate of `e2e_multi_node_primary_runtime_restart_recovers_without_split_brain` is explicit: migrated, retained with written deep-control justification, or split into a named follow-up. It must not remain in the old harness by accident.
- [ ] `.config/nextest.toml`, `tests/nextest_config_contract.rs`, `Cargo.toml`, `Makefile`, and docs all reflect the greenfield suite as the supported black-box path and no longer refer to deleted legacy entrypoints.
- [ ] Repo-wide cleanup verification confirms that stale legacy-harness patterns no longer survive outside intentionally retained deep-control boundaries. At minimum, run:
- [ ] `rg -n "tests/ha/support|tests/ha_multi_node_failover|tests/ha_partition_isolation|src/test_harness/ha_e2e|tests/policy_e2e_api_only" .`
- [ ] `rg -n "ha_e2e::|net_proxy::" src tests docs`
- [ ] docs are updated with new/updated/deleted test architecture and stale docs are removed
- [ ] `make check` passes cleanly
- [ ] `make test` passes cleanly
- [ ] `make test-long` passes cleanly
- [ ] `make lint` passes cleanly
- [ ] `<passes>true</passes>` is set only after every acceptance-criteria item and every required implementation-plan checkbox is complete
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Build the migration and deletion matrix
- [ ] Read tasks 01, 02, and 04 in `story-greenfield-cucumber-ha-harness/` and build an explicit matrix of old scenario -> greenfield replacement -> legacy deletion target.
- [ ] The matrix must enumerate at least the old black-box scenarios in `tests/ha_multi_node_failover.rs` and `tests/ha_partition_isolation.rs`, including:
- [ ] unassisted failover continuity
- [ ] planned switchover
- [ ] targeted switchover accepted path
- [ ] targeted switchover rejected path
- [ ] workload switchover / workload failover
- [ ] custom postgres roles
- [ ] clone failure recovery
- [ ] rewind fallback recovery
- [ ] repeated leadership changes
- [ ] degraded / lagging replica promotion choice
- [ ] no-quorum fail-safe and fencing
- [ ] partition and mixed-fault stories
- [ ] For each old scenario, decide whether task 01/02/04 migrates it, whether it is intentionally retained as deep-control coverage, or whether it needs an explicit follow-up. Do not delete before that mapping is written down.

### Phase 2: Remove the legacy HA black-box harness
- [ ] Delete legacy wrappers in `tests/ha_multi_node_failover.rs` and `tests/ha_partition_isolation.rs` once their migrated replacements exist in greenfield cucumber.
- [ ] Delete `tests/ha/support/multi_node.rs`, `tests/ha/support/observer.rs`, and `tests/ha/support/partition.rs` once no retained test depends on them.
- [ ] Delete obsolete harness modules in `src/test_harness/ha_e2e/`:
- [ ] `config.rs`
- [ ] `handle.rs`
- [ ] `mod.rs`
- [ ] `ops.rs`
- [ ] `startup.rs`
- [ ] `util.rs`
- [ ] Audit `src/test_harness/namespace.rs`, `src/test_harness/binaries.rs`, and adjacent helpers and remove or simplify any code that only existed for the deleted legacy HA harness.
- [ ] Audit `src/test_harness/net_proxy.rs` and delete it if partition scenarios have fully moved to the greenfield Docker network-fault layer.

### Phase 3: Clean mixed black-box files instead of leaving hidden duplicates
- [ ] Audit `tests/policy_e2e_api_only.rs` and delete it once the old harness files it policed are gone.
- [ ] Audit `tests/bdd_api_http.rs` and remove any scenario that now has a greenfield Docker-based replacement. If some tests remain because they are true worker/API contract tests with local stubs, keep only those and update comments/file naming so that boundary is obvious.
- [ ] Audit `tests/cli_binary.rs` and remove any black-box story that is now better covered through the greenfield runtime-based operator path. Keep only CLI contract tests that still need synthetic HTTP fixtures or config parsing boundaries and cannot be replaced cleanly by the Docker suite.
- [ ] If `tests/cli_binary.rs` or `tests/bdd_api_http.rs` remain mixed after deletion, split the retained contract-only cases into clearer files so the repo no longer signals “half-BDD” ambiguity.

### Phase 4: Update tooling, docs, and stale references
- [ ] Update `Cargo.toml` so deleted legacy test targets are gone and greenfield cucumber targets remain the primary black-box suite.
- [ ] Update `.config/nextest.toml` and `tests/nextest_config_contract.rs` so the test runner contract refers to the greenfield suite instead of removed legacy HA binaries.
- [ ] Update `Makefile` so documented HA test entrypoints point at greenfield cucumber targets and no longer mention removed legacy wrappers.
- [ ] Update docs such as `docs/src/how-to/run-tests.md` and any HA/testing architecture pages so they describe one black-box test path and remove stale references to `tests/ha_*`, `tests/ha/support/`, and `src/test_harness/ha_e2e/`.

### Phase 5: Verify cleanup is complete and not lying
- [ ] Run repo-wide stale-pattern verification:
- [ ] `rg -n "tests/ha/support|tests/ha_multi_node_failover|tests/ha_partition_isolation|src/test_harness/ha_e2e|tests/policy_e2e_api_only" .`
- [ ] `rg -n "ha_e2e::|net_proxy::" src tests docs`
- [ ] Confirm every surviving hit is either an intentional retained deep-control boundary or historical task text under `.ralph/tasks/`.
- [ ] Run targeted execution of the greenfield replacements so cleanup is backed by real migrated coverage rather than assumption.
- [ ] Run `make check`
- [ ] Run `make test`
- [ ] Run `make test-long`
- [ ] Run `make lint`
- [ ] Update this task file with completed checkboxes only after the work and every required gate actually pass.
- [ ] Only after all required checkboxes are complete, set `<passes>true</passes>`
- [ ] Run `/bin/bash .ralph/task_switch.sh`
- [ ] Commit all required files, including `.ralph/` updates, with a task-finished commit message that includes verification evidence
- [ ] Push with `git push`

TO BE VERIFIED
