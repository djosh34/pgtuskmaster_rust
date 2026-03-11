# Evidence inventory

## Current gate evidence from this execution

Current gate logs for this planning review:

- `make check`: [make-check.log](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/make-check.log)
- `make test`: [make-test.log](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/make-test.log)
- `make test-long`: [make-test-long.log](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/make-test-long.log)
- `make lint`: [make-lint.log](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/make-lint.log)

Observed in this execution:

- `make check` passed.
- `make test` passed.
- `make lint` passed.
- `make test-long` failed with `26 tests run: 11 passed, 15 failed, 0 skipped`.

Important truth boundary:

- This execution now has both current live failures and preserved historical bug-task evidence.
- The historical bug tasks are still useful because they capture additional symptom detail and earlier reproductions, but they are no longer the only source of evidence.

## Current live failing set from `make test-long`

The current ultra-long HA run failed in these 15 scenarios:

- `ha_full_cluster_outage_restore_quorum_then_converge`
- `ha_mixed_network_faults_heal_converges`
- `ha_no_quorum_enters_failsafe`
- `ha_no_quorum_fencing_blocks_post_cutoff_commits`
- `ha_planned_switchover_changes_primary_cleanly`
- `ha_broken_replica_rejoin_does_not_block_healthy_quorum`
- `ha_full_partition_majority_survives_old_primary_isolated`
- `ha_primary_storage_stall_replaced_by_new_primary`
- `ha_minority_old_primary_rejoins_safely_after_majority_failover`
- `ha_rewind_failure_falls_back_to_basebackup`
- `ha_targeted_switchover_rejects_ineligible_member`
- `ha_two_node_loss_one_good_return_one_broken_return_recovers_service`
- `ha_two_node_outage_one_return_restores_quorum`
- `ha_repeated_leadership_changes_preserve_single_primary`
- `ha_stress_failover_concurrent_sql`

Representative live symptoms from this execution:

- quorum loss still exposes operator-visible primary targets
- no-quorum fencing flow still fails to produce the expected fail-safe-visible state
- healthy two-node majorities still fail to elect a survivor in minority-primary partitions
- mixed DCS/API faults can keep the old primary authoritative or fail to converge after heal
- planned switchover can still produce an incomplete replica view after leadership moves
- broken rejoin and full-cluster restore cases can still report availability or authority before queryable convergence
- targeted switchover still accepts a fully isolated ineligible target
- recovery can bypass the expected `pg_rewind` path entirely
- repeated failovers can still stall on stale leader state
- concurrent failover can still lose acknowledged rows on surviving nodes

This live run confirms that the design-review stop is still current, not merely historical.

## Greenfield scenario classes reviewed

The greenfield feature set under [`cucumber_tests/ha/features`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/cucumber_tests/ha/features) was inspected to map scenario intent, especially:

- [no_quorum_enters_failsafe.feature](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/cucumber_tests/ha/features/no_quorum_enters_failsafe/no_quorum_enters_failsafe.feature)
- [full_partition_majority_survives_old_primary_isolated.feature](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/cucumber_tests/ha/features/full_partition_majority_survives_old_primary_isolated/full_partition_majority_survives_old_primary_isolated.feature)
- [lagging_replica_is_not_promoted.feature](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/cucumber_tests/ha/features/lagging_replica_is_not_promoted/lagging_replica_is_not_promoted.feature)
- [targeted_switchover_rejects_ineligible_member.feature](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/cucumber_tests/ha/features/targeted_switchover_rejects_ineligible_member/targeted_switchover_rejects_ineligible_member.feature)
- [mixed_network_faults_heal_converges.feature](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/cucumber_tests/ha/features/mixed_network_faults_heal_converges/mixed_network_faults_heal_converges.feature)
- [primary_storage_stall_replaced_by_new_primary.feature](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/cucumber_tests/ha/features/primary_storage_stall_replaced_by_new_primary/primary_storage_stall_replaced_by_new_primary.feature)
- [repeated_leadership_changes_preserve_single_primary.feature](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/cucumber_tests/ha/features/repeated_leadership_changes_preserve_single_primary/repeated_leadership_changes_preserve_single_primary.feature)

Those scenarios are coherent. They are not asking for isolated patches. Collectively they require one authority model with these properties:

- quorum as majority of configured cluster membership
- deterministic and durability-aware successor selection
- switchover eligibility matching failover eligibility
- startup authority matching steady-state authority
- clearer distinction between uncertainty, ineligibility, and must-stop safety states
- rejoin and recovery paths that do not claim success before the node is actually queryable and integrated

## Preserved failure evidence from bug tasks

The following existing bug tasks were inspected and used as preserved evidence inputs:

- [bug-greenfield-lone-survivor-remains-primary-after-quorum-loss.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/bug-greenfield-lone-survivor-remains-primary-after-quorum-loss.md)
- [bug-greenfield-no-quorum-fencing-can-miss-fail-safe-state.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/bug-greenfield-no-quorum-fencing-can-miss-fail-safe-state.md)
- [bug-greenfield-majority-partition-can-lose-primary-without-electing-survivor.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/bug-greenfield-majority-partition-can-lose-primary-without-electing-survivor.md)
- [bug-greenfield-lagging-replica-can-still-win-failover.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/bug-greenfield-lagging-replica-can-still-win-failover.md)
- [bug-greenfield-targeted-switchover-accepts-isolated-ineligible-target.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/bug-greenfield-targeted-switchover-accepts-isolated-ineligible-target.md)
- [bug-greenfield-mixed-network-fault-can-leave-dcs-cut-primary-authoritative.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/bug-greenfield-mixed-network-fault-can-leave-dcs-cut-primary-authoritative.md)
- [bug-greenfield-storage-stall-does-not-trigger-primary-replacement.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/bug-greenfield-storage-stall-does-not-trigger-primary-replacement.md)
- [bug-greenfield-old-primary-stays-unknown-after-planned-switchover.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/bug-greenfield-old-primary-stays-unknown-after-planned-switchover.md)
- [bug-greenfield-clone-failure-can-report-rejoined-replica-before-it-is-queryable.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/bug-greenfield-clone-failure-can-report-rejoined-replica-before-it-is-queryable.md)
- [bug-greenfield-broken-rejoin-can-stay-offline-after-blocker-removal.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/bug-greenfield-broken-rejoin-can-stay-offline-after-blocker-removal.md)
- [greenfield-repeated-leadership-churn-can-stall-on-stale-leader-lease.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/greenfield-repeated-leadership-churn-can-stall-on-stale-leader-lease.md)
- [rapid-repeated-failovers-can-drop-intermediate-writes.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/bugs/rapid-repeated-failovers-can-drop-intermediate-writes.md)

Representative preserved symptoms:

- quorum loss can still leave a lone survivor operator-visible as primary
- quorum loss can withdraw primary visibility but still fail to drive every node into explicit `fail_safe`
- a healthy observable two-node majority can fail to elect exactly one primary
- a lagging or isolated replica can still be considered eligible for leadership or targeted switchover
- mixed DCS/API faults can leave a DCS-cut primary authoritative
- a wedged primary can stay authoritative instead of being replaced
- a successful switchover can leave the old primary stuck as `unknown`
- healed rejoin / clone paths can be reported as complete before the node is queryable
- repeated failovers can stall on stale leader lease state or lose acknowledged writes

## Implicated code surfaces

The source review concentrated on:

- [src/dcs/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs)
- [src/ha/decision.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decision.rs)
- [src/ha/decide.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs)
- [src/ha/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs)
- [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs)
- [src/ha/lower.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/lower.rs)
- [src/ha/apply.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/apply.rs)

Key findings:

- Quorum evaluation in [src/dcs/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs) still uses the observed-member shortcut of `1` when the view size is `1`, otherwise `>= 2`, instead of majority-of-configured-membership.
- [src/ha/decide.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decide.rs) routes every non-`FullQuorum` trust state straight into `FailSafe`, which is too blunt for the scenario set.
- [src/ha/decision.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/decision.rs) has stronger leader predicates than startup, but leadership and switchover eligibility are still mostly freshness plus health based, not durability-ranked.
- [src/ha/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/worker.rs) skips effect application through a worker-local dedup heuristic keyed only by `ActiveJobKind`, which means side effects are not driven purely by authoritative state.
- [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs) chooses startup role intent from local disk state plus ad hoc leader/member checks and can default to primary when DCS information is partial or unavailable.

## Cross-check conclusion

The feature files, bug tasks, and source files were cross-checked together. The failure inventory is consistent with the code review. These are not separate one-off bugs. They all trace back to the same larger architectural gaps:

- missing configured-membership quorum semantics
- authority/trust states that are too coarse
- election and switchover eligibility that are not unified around durability and readiness
- startup using a different authority model from the steady-state HA loop
- recovery and rejoin state that can become operator-visible before full convergence
- worker-local dedup bypasses that keep the HA state loop from being the single source of truth

## Gate summary for task truthfulness

Final gate status for this planning stop:

- `make check`: pass
- `make test`: pass
- `make lint`: pass
- `make test-long`: fail

Because `make test-long` is not green, this task cannot honestly claim `<passes>true</passes>`. The artifacts are complete, but the repo is not green.
