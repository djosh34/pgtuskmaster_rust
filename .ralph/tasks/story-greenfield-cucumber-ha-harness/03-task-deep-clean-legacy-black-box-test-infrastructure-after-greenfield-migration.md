## Task: Deep Clean Legacy Black-Box Test Infrastructure After Greenfield Migration <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
After tasks 01, 02, and 04 land, aggressively delete the old black-box HA/E2E/half-BDD test infrastructure and any code, config, docs, or helpers that become unused after that deletion.

This task is deletion only. It is not a scenario-definition task. It must remove the old paths completely wherever the greenfield Docker cucumber suite now covers the behavior.

Hard deletion requirements:
- delete `tests/ha_multi_node_failover.rs` once its migrated scenarios exist in greenfield
- delete `tests/ha_partition_isolation.rs` once its migrated scenarios exist in greenfield
- delete `tests/ha/support/` once its migrated scenarios exist in greenfield
- delete `src/test_harness/ha_e2e/` once its migrated scenarios exist in greenfield
- delete `tests/policy_e2e_api_only.rs` once the old harness boundary it enforced is gone
- remove migrated black-box cases from mixed files such as `tests/bdd_api_http.rs` and `tests/cli_binary.rs`
- delete any now-unused helper code, glue code, module exports, nextest config, make targets, and docs references left behind by that removal
- do not leave stale legacy harness code in the repo out of caution or inertia
</description>

<acceptance_criteria>
- [ ] `tests/ha_multi_node_failover.rs` is deleted or reduced to only explicitly justified non-migrated deep-control coverage.
- [ ] `tests/ha_partition_isolation.rs` is deleted or reduced to only explicitly justified non-migrated deep-control coverage.
- [ ] `tests/ha/support/` is deleted once no retained test needs it.
- [ ] `src/test_harness/ha_e2e/` is deleted once no retained test needs it.
- [ ] `src/test_harness/net_proxy.rs` is deleted if it becomes unused after the legacy harness removal.
- [ ] `tests/policy_e2e_api_only.rs` is deleted.
- [ ] Migrated black-box scenarios are removed from `tests/bdd_api_http.rs`.
- [ ] Migrated black-box scenarios are removed from `tests/cli_binary.rs`.
- [ ] Any code, config, docs, comments, or exports that become unused because of these deletions are removed in the same task.
- [ ] `Cargo.toml`, `Makefile`, `.config/nextest.toml`, `tests/nextest_config_contract.rs`, and docs no longer refer to deleted legacy black-box test entrypoints.
- [ ] Repo-wide verification shows no stale legacy-harness references remain outside intentional retained deep-control tests and historical `.ralph/tasks/` text.
- [ ] `make check` passes cleanly.
- [ ] `make test` passes cleanly.
- [ ] `make test-long` passes cleanly.
- [ ] `make lint` passes cleanly.
- [ ] `<passes>true</passes>` is set only after every acceptance criterion and required checkbox is complete.
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Delete the old HA black-box harness
- [ ] Delete migrated legacy wrappers from `tests/ha_multi_node_failover.rs` and `tests/ha_partition_isolation.rs`.
- [ ] Delete `tests/ha/support/`.
- [ ] Delete `src/test_harness/ha_e2e/`.
- [ ] Delete `src/test_harness/net_proxy.rs` if it has no justified remaining consumer.
- [ ] Delete `tests/policy_e2e_api_only.rs`.

### Phase 2: Delete migrated black-box leftovers from mixed files
- [ ] Remove migrated black-box cases from `tests/bdd_api_http.rs`.
- [ ] Remove migrated black-box cases from `tests/cli_binary.rs`.
- [ ] If either file still mixes retained contract tests with migrated black-box remnants, split or simplify it so the black-box leftovers are gone.

### Phase 3: Delete everything made unused by the harness removal
- [ ] Remove dead helper code and dead module exports.
- [ ] Remove stale nextest configuration and stale make targets.
- [ ] Remove stale docs, comments, and references to the deleted harness.

### Phase 4: Verification and closeout
- [ ] Run `rg -n "tests/ha/support|tests/ha_multi_node_failover|tests/ha_partition_isolation|src/test_harness/ha_e2e|tests/policy_e2e_api_only" .`.
- [ ] Run `rg -n "ha_e2e::|net_proxy::" src tests docs`.
- [ ] Confirm every surviving hit is either an intentional retained deep-control boundary or historical `.ralph/tasks/` text.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make test-long`.
- [ ] Run `make lint`.
- [ ] Update this task file only after the work and verification are actually complete.
- [ ] Only after all required checkboxes are complete, set `<passes>true</passes>`.
- [ ] Run `/bin/bash .ralph/task_switch.sh`.
- [ ] Commit all required files, including `.ralph/` updates, with a task-finished commit message that includes verification evidence.
- [ ] Push with `git push`.

TO BE VERIFIED
