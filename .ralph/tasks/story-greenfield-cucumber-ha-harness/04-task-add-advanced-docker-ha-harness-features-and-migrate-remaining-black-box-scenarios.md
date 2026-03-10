## Task: Add Advanced Docker HA Harness Features And Migrate Remaining Black-Box Scenarios <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Extend the greenfield Docker cucumber HA harness so it can absorb the less-easy-to-migrate black-box scenarios that still qualify for migration. This task must describe, implement, and verify the extra harness features required for those harder scenarios, and it must list the full advanced scenario set to migrate into the Docker-based test path. The key rule is the user's rule: if the behavior can be tested by starting real `pgtuskmaster` binaries and then controlling/observing them from the outside, primarily through `pgtm` and Docker/network control, it belongs here rather than in the legacy HA/E2E/half-BDD harness.

**Original user shift / motivation:** The user wants the migration boundary made explicit. The easy features land in tasks 01 and 02. After that, the repo should not drift back into the legacy harness out of convenience. This task must therefore be the contract for the harder-but-still-black-box stories, and it must say exactly what extra Docker harness capabilities are needed in terms of control, compose topology, network faulting, workload generation, fault wrappers, and other external orchestration. The user also wants the full scenario list written down here so task 03 can remove the corresponding legacy tests completely.

**Higher-order goal:** Finish the migration of black-box behavior testing onto one greenfield architecture: real containers, real binaries, operator-visible control and observation, one-scenario-per-feature cucumber structure, and no hidden dependence on the old harness for scenarios that are externally testable.

**Scope:**
- Extend the greenfield harness under `cucumber_tests/ha/` rather than adding more behavior to the legacy harness.
- Add only the extra harness features needed for externally testable scenarios. Keep true deep-control / unit-contract tests out of scope.
- Update `cucumber_tests/ha/support/...`, `cucumber_tests/ha/givens/...`, `Cargo.toml`, `Makefile`, and greenfield HA docs as required.
- Implement one scenario per feature for the advanced black-box stories listed below.
- Keep `pgtm` as the primary control and observation surface after startup whenever the behavior is exposed there; use Docker/network/fault controls only for fault injection and topology shaping.
- Explicit non-goals for this task:
- `src/worker_contract_tests.rs` style internal worker contract smoke
- `tests/bdd_state_watch.rs` style tiny channel-flow tests
- parser/config/exit-code tests that still belong as unit/CLI contract tests
- any scenario that fundamentally requires white-box internal hooks rather than real runtime orchestration
- The old runtime-only restart / “kill pgtuskmaster only” style scenario is not automatically in scope here. If it cannot be expressed honestly through the agreed greenfield operator model, it must stay out and be called out explicitly rather than half-migrated.

**Context from research:**
- The remaining black-box scenarios currently spread across legacy files include:
- `tests/ha_multi_node_failover.rs`
- `tests/ha_partition_isolation.rs`
- parts of `tests/cli_binary.rs`
- parts of `tests/bdd_api_http.rs`
- The research pass identified the following scenario families as still migratable into the greenfield Docker path once the harness grows the needed features:
- workload-driven switchover and failover
- targeted switchover rejection to an ineligible member
- custom postgres role / recovery config variants
- clone failure and rewind fallback recovery
- repeated leadership changes
- degraded / lagging replica promotion choice
- no-quorum fail-safe and fencing under DCS quorum loss
- full 1:2 partition majority/minority stories
- API-path, postgres-path, and mixed network fault stories
- storage/WAL stall replacement of a wedged primary
- broken returning node and broken replica rejoin stories
- The easy planned-switchover and targeted-switchover accepted-path features are already assigned to task 02 and must not be duplicated here.
- Existing old scenario names that this task is intended to replace or subsume include:
- `e2e_multi_node_stress_planned_switchover_concurrent_sql`
- `e2e_multi_node_stress_unassisted_failover_concurrent_sql`
- `e2e_multi_node_custom_postgres_role_names_survive_bootstrap_and_rewind`
- `e2e_multi_node_clone_failure_recovers_after_fault_removed`
- `e2e_multi_node_rewind_failure_falls_back_to_basebackup`
- `e2e_multi_node_repeated_leadership_changes_preserve_single_primary`
- `e2e_multi_node_degraded_replica_failover_promotes_only_healthy_target`
- `e2e_multi_node_rejects_targeted_switchover_to_ineligible_member`
- `e2e_no_quorum_enters_failsafe_strict_all_nodes`
- `e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity`
- `e2e_partition_minority_isolation_no_split_brain_rejoin`
- `e2e_partition_primary_isolation_failover_no_split_brain`
- `e2e_partition_api_path_isolation_preserves_primary`
- `e2e_partition_primary_postgres_path_blocked_replicas_catch_up_after_heal`
- `e2e_partition_mixed_faults_heal_converges`
- plus the still-open migrated story files from the old quorum task set: storage/WAL stall, broken returning node, lagging/stale replica promotion, minority old primary stale rejoin, broken replica rejoin
- The advanced harness features this task must define are external orchestration features, not internal worker hooks.

**Expected outcome:**
- The greenfield Docker cucumber harness can cover the remaining externally testable HA stories without falling back to the legacy harness.
- The repo gains a written and implemented contract for advanced fault orchestration: network partitions, DCS quorum loss, workload generation, recovery-binary blockers, lag/staleness shaping, storage stall injection, and config variants.
- Task 03 can then delete the corresponding legacy test infrastructure with confidence because the advanced migrated scenarios are explicitly enumerated and shipped here.

</description>

<acceptance_criteria>
- [ ] The greenfield harness gains explicit advanced orchestration features under `cucumber_tests/ha/support/...` for all external-control needs this task covers:
- [ ] Docker/network fault control for full 1:2 partitioning and path-specific isolation across etcd, API, and PostgreSQL replication/data paths
- [ ] DCS quorum control through explicit etcd member/service stop-start or equivalent external faulting
- [ ] workload generation and commit/integrity telemetry for concurrent-SQL scenarios
- [ ] deterministic recovery-binary and startup fault wrappers for `pg_basebackup`, `pg_rewind`, and broken rejoin/startup cases
- [ ] lag/staleness / degraded-replica shaping sufficient to test promotion-choice safety
- [ ] storage/WAL stall style fault injection sufficient to wedge a primary without pretending the node vanished
- [ ] additional checked-in given/config variants when required, such as custom postgres role names
- [ ] The advanced scenario set is implemented as one scenario per feature, with tiny wrappers registered in `Cargo.toml`, for all scenarios this task migrates.
- [ ] At minimum, this task ships feature coverage for the following advanced scenario families:
- [ ] concurrent workload planned switchover
- [ ] concurrent workload failover
- [ ] targeted switchover rejection for an ineligible member
- [ ] custom postgres roles survive failover/rejoin
- [ ] clone failure recovers after blocker removal
- [ ] rewind failure falls back to basebackup
- [ ] repeated leadership changes preserve a single primary
- [ ] lagging/degraded replica is not promoted over the healthier candidate
- [ ] no-quorum fail-safe after DCS quorum loss
- [ ] no-quorum fencing blocks post-cutoff commits and preserves integrity
- [ ] full 1:2 partition where minority contains old primary
- [ ] full 1:2 partition where minority contains replica and majority preserves/retains authority
- [ ] minority old primary returns with stale view and is forced to rejoin safely
- [ ] API-path isolation preserves primary
- [ ] postgres-path isolation blocks replica catch-up until heal
- [ ] mixed network faults heal and converge
- [ ] primary storage/WAL stall causes replacement primary election
- [ ] two-node loss with one good return and one broken return still recovers service
- [ ] broken replica rejoin does not block healthy quorum availability
- [ ] Every advanced feature uses the greenfield Docker harness and operator-visible control/observation model; none of them import or call the legacy `tests/ha` or `src/test_harness/ha_e2e` code.
- [ ] The task explicitly records any scenario that still cannot honestly be expressed through the greenfield black-box model and splits it into a follow-up instead of silently keeping it in the old harness.
- [ ] Docs are updated with the advanced harness capabilities, scenario inventory, and operator-control model, and stale docs are removed when relevant.
- [ ] `make check` passes cleanly
- [ ] `make test` passes cleanly
- [ ] `make test-long` passes cleanly
- [ ] `make lint` passes cleanly
- [ ] `<passes>true</passes>` is set only after every acceptance-criteria item and every required implementation-plan checkbox is complete
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Freeze the advanced migration boundary and the explicit non-goals
- [ ] Re-read tasks 01 and 02 and list exactly which advanced stories remain after the low-hanging outage and switchover features are done.
- [ ] Build an explicit “migrate now vs keep deep-control vs split later” table for the remaining old black-box scenarios from `tests/ha_multi_node_failover.rs`, `tests/ha_partition_isolation.rs`, and any black-box cases in `tests/cli_binary.rs` / `tests/bdd_api_http.rs`.
- [ ] Explicitly call out which old scenarios are not part of this task because they still require deep internal control rather than real runtime orchestration. Do not leave that decision implicit.

### Phase 2: Add the advanced harness capabilities
- [ ] Design and implement a greenfield network-fault layer under `cucumber_tests/ha/support/...` that can:
- [ ] isolate one node as the 1-side minority from the 2-side majority
- [ ] partition etcd/control-plane traffic separately from API traffic and postgres/replication traffic
- [ ] create API-only isolation
- [ ] create postgres-path-only isolation
- [ ] compose mixed faults and later heal them deterministically
- [ ] Design and implement explicit DCS quorum fault controls for the greenfield Docker topology:
- [ ] stop/start etcd members or equivalent DCS services
- [ ] prove majority loss and restoration through operator-visible observations
- [ ] Design and implement a reusable workload engine for greenfield cucumber:
- [ ] concurrent writers
- [ ] commit and rejection counting
- [ ] commit timestamp capture or equivalent cutoff analysis
- [ ] integrity verification after failover or fencing
- [ ] artifact capture for workload statistics
- [ ] Design and implement deterministic fault-wrapper support for recovery and startup paths:
- [ ] `pg_basebackup` failure / removal / retry
- [ ] `pg_rewind` failure with fallback path
- [ ] broken rejoin / broken startup blockers
- [ ] Design and implement degraded-replica shaping:
- [ ] replica made stale, lagging, or otherwise ineligible
- [ ] operator-visible evidence that it is the worse candidate
- [ ] promotion-choice assertions after primary loss
- [ ] Design and implement storage/WAL stall injection with externally observable evidence that the primary is wedged rather than cleanly dead.
- [ ] Add any required checked-in given/config variants, for example custom postgres role names, without collapsing back into runtime-generated opaque configs.

### Phase 3: Register the advanced feature directories and wrappers
- [ ] Add explicit feature directories plus tiny wrappers for each advanced migrated scenario. At minimum create feature roots for:
- [ ] `stress_planned_switchover_concurrent_sql`
- [ ] `stress_failover_concurrent_sql`
- [ ] `targeted_switchover_rejects_ineligible_member`
- [ ] `custom_postgres_roles_survive_failover_and_rejoin`
- [ ] `clone_failure_recovers_after_blocker_removed`
- [ ] `rewind_failure_falls_back_to_basebackup`
- [ ] `repeated_leadership_changes_preserve_single_primary`
- [ ] `lagging_replica_is_not_promoted`
- [ ] `no_quorum_enters_failsafe`
- [ ] `no_quorum_fencing_blocks_post_cutoff_commits`
- [ ] `full_partition_majority_survives_old_primary_isolated`
- [ ] `full_partition_majority_survives_old_replica_isolated`
- [ ] `minority_old_primary_rejoins_safely_after_majority_failover`
- [ ] `api_path_isolation_preserves_primary`
- [ ] `postgres_path_isolation_replicas_catch_up_after_heal`
- [ ] `mixed_network_faults_heal_converges`
- [ ] `primary_storage_stall_replaced_by_new_primary`
- [ ] `two_node_loss_one_good_return_one_broken_return_recovers_service`
- [ ] `broken_replica_rejoin_does_not_block_healthy_quorum`
- [ ] Register one `[[test]]` target in `Cargo.toml` per advanced wrapper.

### Phase 4: Implement the workload and recovery scenario features
- [ ] Implement the concurrent-workload planned switchover scenario: bootstrap stable primary, start workload, request switchover through `pgtm`, verify no dual-primary evidence, verify new primary and table integrity.
- [ ] Implement the concurrent-workload failover scenario: bootstrap stable primary, start workload, inject failover fault, verify no split-brain write evidence, verify new primary and table integrity.
- [ ] Implement the custom-postgres-role scenario with a checked-in config variant and prove rejoin after failover still works.
- [ ] Implement the clone-failure scenario with an externally controlled `pg_basebackup` blocker and prove the node rejoins after the blocker is removed.
- [ ] Implement the rewind-failure scenario with an externally controlled `pg_rewind` blocker and prove fallback recovery still rejoins the old primary safely.
- [ ] Implement the repeated-leadership-change scenario and prove the primary sequence changes without any dual-primary window.
- [ ] Implement the broken-returning-node and broken-rejoin scenarios so a bad node does not block healthy quorum service.
- [ ] Implement the storage/WAL stall scenario and prove a wedged primary is replaced cleanly by a healthy successor.

### Phase 5: Implement the promotion-choice, no-quorum, and fencing scenarios
- [ ] Implement the lagging/degraded replica scenario and prove the healthier candidate is the only promoted node after primary loss.
- [ ] Implement the targeted-switchover rejection scenario and prove the ineligible target is rejected without disturbing the current primary.
- [ ] Implement the no-quorum fail-safe scenario with explicit DCS majority loss and prove no node remains primary.
- [ ] Implement the no-quorum fencing scenario with active workload, explicit cutoff analysis, and integrity verification after quorum restoration.

### Phase 6: Implement the partition and mixed-fault scenarios
- [ ] Implement the full 1:2 partition scenario where the minority side contains the old primary and the majority side elects the only authoritative primary before heal.
- [ ] Implement the full 1:2 partition scenario where the minority side contains a replica and the majority side preserves or converges to one primary before heal.
- [ ] Implement the minority-old-primary stale-return scenario and prove safe demotion/rejoin after partition repair.
- [ ] Implement the API-path isolation scenario and prove API visibility loss alone does not force unnecessary leadership movement.
- [ ] Implement the postgres-path isolation scenario and prove replicas stop catching up during the fault and then converge after heal.
- [ ] Implement the mixed-fault scenario and prove convergence after healing the composed network faults.

### Phase 7: Enforce the greenfield boundary and prepare for task 03 cleanup
- [ ] Run repo-wide verification such as `rg -n "(tests/ha|tests/ha_|src/test_harness/ha_e2e)" cucumber_tests/ha Cargo.toml docs/src/how-to/run-tests.md` and confirm the advanced greenfield implementation does not depend on the legacy harness.
- [ ] In this task file, list the old test names and files that task 03 must delete once this migration lands, so cleanup is a mechanical follow-through rather than a fresh rediscovery exercise.
- [ ] Update docs and developer entrypoints so the advanced greenfield suite is discoverable and the old harness is clearly marked as pending removal by task 03.

### Phase 8: Verification and closeout
- [ ] Run targeted execution of each new advanced feature or scenario family so every advanced migrated story is exercised on top of the greenfield Docker harness.
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
