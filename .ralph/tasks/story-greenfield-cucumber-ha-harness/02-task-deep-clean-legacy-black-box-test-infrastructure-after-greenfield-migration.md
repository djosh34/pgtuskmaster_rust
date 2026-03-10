## Task: Deep Clean Legacy Black-Box Test Infrastructure After Greenfield Migration <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
Task 02 is no longer a narrow overlap cleanup. The legacy HA/E2E surface is now considered actively harmful and must be removed decisively before the greenfield cucumber story continues. The repo already has an independent greenfield HA harness under `cucumber_tests/ha/`, and the user explicitly wants the old HA/E2E harness gone rather than preserved until later migration tasks finish.

This task therefore fully replaces the earlier “keep the old harness for now” direction. Delete the legacy HA/E2E integration-test surface in `tests/` and the hidden legacy HA/E2E support surface in `src/test_harness/` that exists only to prop those tests up. Do not keep harmful legacy coverage around just because tasks 03 and 04 have not yet recreated every scenario. Those later tasks should build the desired greenfield features from their scenario contracts, not from a half-dead legacy harness that keeps interfering with the repo.

Aggressive cleanup means the default stance is deletion, not preservation. The executor should assume the whole `tests/` directory is deletable unless specific surviving tests can be defended as small, clearly valuable, non-legacy contract checks that do not rely on the deleted HA/E2E harness or keep stale routing alive. Likewise, the executor should assume most of `src/test_harness/` is suspect once the old HA/E2E path is removed, and only keep helpers that are still demonstrably needed by specific surviving tests.

The decision boundary must be test-by-test, not file-by-file. For integration tests, inspect individual tests and keep only the ones that are still worth having. For helper modules such as `src/test_harness/pg16.rs`, `src/test_harness/etcd3.rs`, `src/test_harness/runtime_config.rs`, `src/test_harness/tls.rs`, and similar, do not preserve a whole helper file just because some part of it might still be useful in theory. Keep helper code only when a concrete surviving test still calls it. If a file mixes keeper helpers and dead helpers, shrink the file instead of treating it as all-or-nothing.

The required delete set for this task is at least:
- `tests/ha_multi_node_failover.rs`
- `tests/ha_partition_isolation.rs`
- `tests/ha/support/multi_node.rs`
- `tests/ha/support/partition.rs`
- `tests/ha/support/observer.rs`
- `tests/policy_e2e_api_only.rs`
- `src/test_harness/ha_e2e/`
- `src/test_harness/net_proxy.rs`
- any direct module exports, helpers, comments, docs, nextest rules, or task text that still preserve or route through that deleted legacy HA/E2E surface

The executor must also review the rest of `tests/` and `src/test_harness/` with a delete-first mindset:
- if an integration test is not clearly worth keeping, delete it
- if a `src/test_harness/*` module existed mainly to support the deleted integration/E2E story, delete it
- only keep a module when there is a direct surviving caller and a strong reason to keep it

The keep set is intentionally tiny and must be defended explicitly:
- keep unit tests under `src/` unless they only exist for deleted legacy HA/E2E modules
- do not assume any integration test in `tests/` survives by default
- only keep a specific integration test if it is both:
- independent from the deleted HA/E2E harness and routing
- obviously valuable as a small contract test for a live product surface
- the most plausible candidates are:
- `tests/bdd_state_watch.rs`
- `tests/bdd_api_http.rs`
- `tests/cli_binary.rs`
- `tests/nextest_config_contract.rs` is not automatically a keeper; if deleting `tests/ha_*` makes its assertions stale or low-value, delete or rewrite it rather than preserving old layout assumptions
- if any supposedly kept integration test still depends on deleted HA/E2E helpers or assumptions, either refactor it away from that dependency in this task or delete it too; do not keep accidental legacy coupling

This task must make the deletion boundary fully unambiguous before any more greenfield feature growth. After this task, there should be no repo ambiguity about whether the legacy HA/E2E harness is still a supported path. It is not.
</description>

<acceptance_criteria>
- [ ] Task 02 explicitly supersedes the earlier “retain unreplaced legacy HA coverage” plan and instead treats full legacy HA/E2E removal as mandatory before continuing the greenfield story.
- [ ] `tests/ha_multi_node_failover.rs` is deleted.
- [ ] `tests/ha_partition_isolation.rs` is deleted.
- [ ] `tests/ha/support/multi_node.rs` is deleted.
- [ ] `tests/ha/support/partition.rs` is deleted.
- [ ] `tests/ha/support/observer.rs` is deleted.
- [ ] `tests/policy_e2e_api_only.rs` is deleted.
- [ ] `src/test_harness/ha_e2e/` is deleted.
- [ ] `src/test_harness/net_proxy.rs` is deleted.
- [ ] Any newly dead `src/test_harness` exports or modules caused by those deletions are removed in the same task instead of being left as tombstones.
- [ ] Docs, task text, and gate/config references no longer claim that `tests/ha_*`, `tests/ha/support/*`, `src/test_harness/ha_e2e`, `src/test_harness/net_proxy.rs`, or any other removed legacy helpers are still part of the supported verification story.
- [ ] The keep set is explicit and minimal: unit tests remain, and only a tiny number of non-legacy integration tests with clear contract value remain, if any.
- [ ] No kept test target or helper still imports, references, or textually depends on the deleted legacy HA/E2E harness.
- [ ] Surviving integration tests are chosen on a test-by-test basis rather than by preserving whole files out of convenience.
- [ ] Surviving helper code under `src/test_harness/` is chosen on a test-by-test caller basis rather than by preserving whole helper files out of convenience.
- [ ] Tasks 03 and 04 are left clearly positioned as greenfield feature-construction tasks, not as reasons to keep the old harness alive.
- [ ] `<passes>true</passes>` is set only after every acceptance criterion and required checkbox is complete.
</acceptance_criteria>

## Detailed implementation plan

### Phase 0: Rewrite the deletion boundary so it is explicit
- [ ] Update this task text and any directly adjacent story wording so the repo no longer says task 02 must preserve the legacy HA/E2E harness.
- [ ] Make it explicit that the old HA/E2E surface is being removed because it is actively harming the repo, not because it has been perfectly feature-for-feature replaced already.
- [ ] Make it explicit that unit tests stay, and that only narrowly justified non-legacy integration tests stay.

### Phase 1: Delete the legacy HA/E2E integration tests in `tests/`
- [ ] Delete `tests/ha_multi_node_failover.rs`.
- [ ] Delete `tests/ha_partition_isolation.rs`.
- [ ] Delete `tests/ha/support/multi_node.rs`.
- [ ] Delete `tests/ha/support/partition.rs`.
- [ ] Delete `tests/ha/support/observer.rs`.
- [ ] Delete `tests/policy_e2e_api_only.rs`.
- [ ] Remove the now-empty `tests/ha/` directory if it becomes empty.
- [ ] Inventory every remaining test in `tests/` with a delete-first mindset instead of assuming a file survives as a unit.
- [ ] Delete any remaining `tests/*.rs` file that has no keeper tests left after the per-test review.
- [ ] If a remaining file mixes keeper tests and dead tests, delete only the dead tests and keep the file only if the surviving tests still justify it.

### Phase 2: Delete the hidden legacy HA/E2E support surface in `src/test_harness/`
- [ ] Delete the entire `src/test_harness/ha_e2e/` directory.
- [ ] Delete `src/test_harness/net_proxy.rs`.
- [ ] Remove `pub mod ha_e2e;`, `pub mod net_proxy;`, and any other newly dead exports or references from `src/test_harness/mod.rs` or elsewhere.
- [ ] Review the remaining `src/test_harness/*` modules helper-by-helper with a delete-first mindset.
- [ ] If deletion exposes additional `src/test_harness` modules that were only there for the old HA/E2E path, remove them too in this task; do not preserve dead support code.
- [ ] Only keep helper code under `src/test_harness/*` when there is a direct surviving test caller and a clear reason to keep it.
- [ ] If a helper file mixes keeper helpers and dead helpers, delete the dead helpers instead of preserving the whole file by default.

### Phase 3: Preserve only a tiny defensible keep set
- [ ] Keep unit tests in `src/**` unless they only test deleted legacy HA/E2E code.
- [ ] Treat `tests/bdd_state_watch.rs`, `tests/bdd_api_http.rs`, `tests/cli_binary.rs`, and `tests/nextest_config_contract.rs` as candidate containers only, not automatic keepers; inspect their individual tests.
- [ ] Delete any candidate keeper that is stale, low-value, or still coupled to the deleted harness/routing.
- [ ] For helpers such as `pg16`, `etcd3`, `runtime_config`, `tls`, `ports`, `namespace`, `binaries`, `auth`, `provenance`, and `signals`, inspect which surviving tests still call them before deciding whether any part of those helpers stays.
- [ ] Do not keep any test or helper merely because it once provided coverage.

### Phase 4: Remove stale references and routing
- [ ] Update `.config/nextest.toml` so it no longer routes or documents deleted `tests/ha_*` binaries as the long-test boundary.
- [ ] Update `Makefile` targets or comments that still imply the old HA binaries are the `test-long` payload.
- [ ] Update `docs/src/how-to/run-tests.md` and any directly stale docs so they no longer describe the deleted legacy HA/E2E harness as present.
- [ ] Update any nearby Ralph task text that still depends on the old “keep the legacy harness for later migration” assumption if that wording would now mislead the next executor.

### Phase 5: Verification and closeout
- [ ] Run `rg -n "tests/ha_|tests/ha/support|policy_e2e_api_only|src/test_harness/ha_e2e|src/test_harness/net_proxy" src tests docs .config Makefile .ralph`.
- [ ] Confirm any remaining matches are intentional historical/task references only, or remove/update them if they still describe live code paths.
- [ ] Verify the remaining integration-test surface is limited to the explicitly kept non-legacy tests plus the greenfield cucumber surface.
- [ ] Verify unit tests remain intact except for tests that necessarily disappear with deleted legacy modules.
- [ ] For every helper left in `src/test_harness/`, verify there is still at least one surviving concrete test that needs it.
- [ ] Update this task file only after the work and verification are actually complete.
- [ ] Only after all required checkboxes are complete, set `<passes>true</passes>`.
- [ ] Run `/bin/bash .ralph/task_switch.sh`.
- [ ] Commit all required files, including `.ralph/` updates, with a task-finished commit message that includes verification evidence.
- [ ] Push with `git push`.

NOW EXECUTE
