## Task: Deep Clean Legacy Black-Box Test Infrastructure After Greenfield Migration <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
After task 01, the only shipped greenfield HA feature is `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.feature`. A skeptical repo review shows that tasks 03 and 04 still own migration of the rest of the legacy HA coverage, so this task must not delete the full old HA harness yet. Doing so now would remove unreplaced failover, switchover, quorum, partition, and chaos coverage.

This task therefore narrows cleanup to the legacy boundary that task 01 actually replaced: the old unassisted primary-failover-and-rejoin path in `tests/ha_multi_node_failover.rs` and `tests/ha/support/multi_node.rs`. The job is to delete that migrated legacy scenario and then remove any direct stale references that specifically describe the old long-suite binary as still owning that exact boundary.

Do not delete `tests/ha_partition_isolation.rs`, `tests/ha/support/partition.rs`, `tests/ha/support/observer.rs`, `src/test_harness/ha_e2e/`, `src/test_harness/net_proxy.rs`, `tests/policy_e2e_api_only.rs`, or the remaining `ha_multi_node_failover` scenarios in this task. Those still back unreplaced coverage that tasks 03 and 04 explicitly plan to migrate later.

This task is still deletion-first, but only within the currently migrated boundary. If rerunning the greenfield `primary_crash_rejoin` feature or the repo gates exposes a trustworthy HA or product failure rather than a harness failure, create a bug task via add-bug and add `<blocked_by>` tags for every task in `story-greenfield-cucumber-ha-harness`.
</description>

<acceptance_criteria>
- [ ] Task 02 is treated as the mandatory cleanup step immediately after task 01 and before tasks 03 and 04 expand the greenfield scenario set.
- [ ] The obviously wrong or misleading legacy HA/E2E tests are explicitly removed first so further greenfield scenario work starts from a clean slate.
- [ ] `tests/ha_multi_node_failover.rs` no longer exposes the old unassisted failover/rejoin scenario that task 01 replaced.
- [ ] `tests/ha/support/multi_node.rs` no longer defines the old unassisted failover/rejoin scenario implementation or dead keepalive references for it.
- [ ] Remaining legacy HA binaries and support modules are left in place when their scenario coverage is not yet migrated to greenfield.
- [ ] Any docs or comments that still imply the legacy long HA binary owns the primary-crash/rejoin boundary are updated in the same task.
- [ ] Repo-wide verification shows the deleted legacy primary-crash overlap is gone while unreplaced legacy HA coverage remains intentionally present.
- [ ] Cleanup happens only for the replacement greenfield feature that already exists and can be executed to a trustworthy harness-backed outcome, even if that outcome is a product-failure result.
- [ ] Any trustworthy HA or product failure found while rerunning migrated greenfield features after cleanup creates a bug task with add-bug with `<blocked_by>` tags for every task in this story.
- [ ] `<passes>true</passes>` is set only after every acceptance criterion and required checkbox is complete.
</acceptance_criteria>

## Detailed implementation plan

### Phase 0: Reset the slate before adding more greenfield scenarios
- [ ] Treat this task as the required bridge between task 01 and tasks 03/04, not as optional follow-up cleanup.
- [ ] Narrow the cleanup to the migrated boundary task 01 actually replaced: legacy primary-crash failover/rejoin coverage.
- [ ] Do not delete unreplaced long HA harness surfaces that tasks 03 and 04 still need to migrate.

### Phase 1: Delete the migrated legacy primary-crash overlap
- [ ] Remove the legacy unassisted failover/rejoin test entrypoint from `tests/ha_multi_node_failover.rs`.
- [ ] Remove the matching scenario implementation and dead keepalive reference from `tests/ha/support/multi_node.rs`.

### Phase 2: Delete direct stale references to the removed overlap
- [ ] Update docs or comments that still describe the old long HA binary as owning the primary-crash/rejoin boundary now covered by `primary_crash_rejoin`.
- [ ] Keep docs accurate about the remaining unreplaced legacy HA coverage instead of pretending the whole harness is already gone.

### Phase 3: Verification and closeout
- [ ] Run `rg -n "e2e_multi_node_unassisted_failover_sql_consistency|ha-e2e-unassisted-failover-sql-consistency|ha_unassisted_failover_proof" tests docs src`.
- [ ] Run `make test-cucumber-ha-primary-crash-rejoin` if a focused rerun is needed while developing, and treat any trustworthy product failure as an add-bug trigger rather than a reason to restore the deleted legacy path.
- [ ] Confirm every surviving long HA reference is still tied to intentionally retained, unreplaced legacy coverage.
- [ ] Update this task file only after the work and verification are actually complete.
- [ ] Only after all required checkboxes are complete, set `<passes>true</passes>`.
- [ ] Run `/bin/bash .ralph/task_switch.sh`.
- [ ] Commit all required files, including `.ralph/` updates, with a task-finished commit message that includes verification evidence.
- [ ] Push with `git push`.

NOW EXECUTE
