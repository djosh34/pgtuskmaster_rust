## Task: Add Low-Hanging HA Quorum And Switchover Cucumber Features On The Greenfield Runner <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
Add the next six greenfield Docker HA cucumber features after task 01. This task contains the exact scenario contracts. Each scenario is one feature, one `.feature` file, and one tiny Rust wrapper. Use real `pgtuskmaster` nodes in Docker and use `pgtm` as the operator control and observation surface after startup.

It is explicitly not a requirement that all six scenarios pass against the product before this task is considered complete. The requirement is that all six scenarios are created, wired into the greenfield harness, and can be executed. If a scenario fails, the run must make it clear that the failure is an HA behavior failure in the system under test rather than a harness failure such as broken startup, broken orchestration, bad fixture wiring, missing commands, or unreadable artifacts.
Another explicit requirement, is that the tests must (just like before), be able to succesfully executed in parallel.
Serial execution of tests, is a failure of this task.


**Scenario contracts**

1. `replica_outage_keeps_primary_stable`
- Start `three_node_plain`.
- Wait for exactly one stable primary and record it as `initial_primary`.
- Choose one non-primary node as `stopped_replica`.
- Create a proof table for this feature.
- Insert proof row `1:before-replica-outage` through `initial_primary`.
- Verify all three nodes contain `1:before-replica-outage`.
- Kill `stopped_replica`.
- Verify `pgtm primary` still reports `initial_primary`.
- Verify there is still exactly one primary.
- Verify the remaining online non-primary node is still a replica.
- Insert proof row `2:during-replica-outage` through `initial_primary`.
- Restart `stopped_replica`.
- Wait until `stopped_replica` is visible again as a replica and not a primary.
- Verify all three nodes converge on exactly:
- `1:before-replica-outage`
- `2:during-replica-outage`

2. `two_node_outage_one_return_restores_quorum`
- Start `three_node_plain`.
- Wait for exactly one stable primary and record it as `initial_primary`.
- Choose the two non-primary nodes as `stopped_node_a` and `stopped_node_b`.
- Create a proof table for this feature.
- Insert proof row `1:before-two-node-outage` through `initial_primary`.
- Verify all three nodes contain `1:before-two-node-outage`.
- Kill `stopped_node_a` and `stopped_node_b`.
- Verify there is no operator-visible primary outcome while only one node remains.
- Verify the lone survivor is not treated as a writable primary.
- Restart only `stopped_node_a`.
- Wait until exactly one primary exists across the two running nodes.
- Insert proof row `2:after-quorum-restore-before-full-heal` through the restored primary.
- Verify the cluster is degraded but operational before `stopped_node_b` returns.
- Restart `stopped_node_b`.
- Wait until `stopped_node_b` rejoins as a replica.
- Verify all three nodes converge on exactly:
- `1:before-two-node-outage`
- `2:after-quorum-restore-before-full-heal`

3. `full_cluster_outage_restore_quorum_then_converge`
- Start `three_node_plain`.
- Wait for exactly one stable primary.
- Create a proof table for this feature.
- Insert proof row `1:before-full-cluster-outage`.
- Verify all three nodes contain `1:before-full-cluster-outage`.
- Kill all three node containers.
- Start exactly two fixed nodes first and keep the third node stopped.
- Wait until the two-node subset restores exactly one primary.
- Insert proof row `2:after-two-node-restore-before-final-node` through that primary.
- Verify the third node is still offline during that write.
- Start the final node.
- Wait until the final node rejoins as a replica.
- Verify the final node does not disturb the elected primary.
- Verify all three nodes converge on exactly:
- `1:before-full-cluster-outage`
- `2:after-two-node-restore-before-final-node`

4. `replica_flap_keeps_primary_stable`
- Start `three_node_plain`.
- Wait for exactly one stable primary and record it as `initial_primary`.
- Choose one non-primary node as `flapping_replica`.
- Create a proof table for this feature.
- Insert proof row `1:before-flap`.
- Verify all three nodes contain `1:before-flap`.
- Perform three flap cycles on `flapping_replica`.
- In each cycle:
- kill `flapping_replica`
- verify `initial_primary` is still the only primary
- insert one proof row through `initial_primary`
- restart `flapping_replica`
- wait until `flapping_replica` is again a replica
- Use these exact rows for the three cycles:
- `2:during-flap-cycle-1`
- `3:during-flap-cycle-2`
- `4:during-flap-cycle-3`
- After the third heal, verify all three nodes converge on exactly:
- `1:before-flap`
- `2:during-flap-cycle-1`
- `3:during-flap-cycle-2`
- `4:during-flap-cycle-3`

5. `planned_switchover_changes_primary_cleanly`
- Start `three_node_plain`.
- Wait for exactly one stable primary and record it as `old_primary`.
- Create a proof table for this feature.
- Insert proof row `1:before-planned-switchover` through `old_primary`.
- Verify all three nodes contain `1:before-planned-switchover`.
- Record the initial output of `pgtm primary` and `pgtm replicas`.
- Run `pgtm switchover request`.
- Wait until exactly one different primary stabilizes and record it as `new_primary`.
- Verify `new_primary` is not `old_primary`.
- Verify there is exactly one primary after the switchover.
- Verify `old_primary` remains online and becomes a replica.
- Verify `pgtm primary` points to `new_primary`.
- Verify `pgtm replicas` shows `old_primary` plus the remaining replica and does not show `new_primary`.
- Insert proof row `2:after-planned-switchover` through `new_primary`.
- Verify all three nodes converge on exactly:
- `1:before-planned-switchover`
- `2:after-planned-switchover`

6. `targeted_switchover_promotes_requested_replica`
- Start `three_node_plain`.
- Wait for exactly one stable primary and record it as `old_primary`.
- Choose one replica as `target_replica`.
- Record the other replica as `other_replica`.
- Create a proof table for this feature.
- Insert proof row `1:before-targeted-switchover` through `old_primary`.
- Verify all three nodes contain `1:before-targeted-switchover`.
- Run `pgtm switchover request --switchover-to <target_replica>`.
- Wait until exactly one primary stabilizes.
- Verify that primary is `target_replica`.
- Verify `other_replica` never becomes primary during the switchover window.
- Verify `old_primary` remains online and becomes a replica.
- Verify `pgtm primary` points to `target_replica`.
- Verify `pgtm replicas` shows `old_primary` and `other_replica` and does not show `target_replica`.
- Insert proof row `2:after-targeted-switchover` through `target_replica`.
- Verify all three nodes converge on exactly:
- `1:before-targeted-switchover`
- `2:after-targeted-switchover`
</description>

<acceptance_criteria>
- [ ] `cucumber_tests/ha/features/replica_outage_keeps_primary_stable/replica_outage_keeps_primary_stable.feature` exists and implements the exact `replica_outage_keeps_primary_stable` scenario contract above.
- [ ] `cucumber_tests/ha/features/two_node_outage_one_return_restores_quorum/two_node_outage_one_return_restores_quorum.feature` exists and implements the exact `two_node_outage_one_return_restores_quorum` scenario contract above.
- [ ] `cucumber_tests/ha/features/full_cluster_outage_restore_quorum_then_converge/full_cluster_outage_restore_quorum_then_converge.feature` exists and implements the exact `full_cluster_outage_restore_quorum_then_converge` scenario contract above.
- [ ] `cucumber_tests/ha/features/replica_flap_keeps_primary_stable/replica_flap_keeps_primary_stable.feature` exists and implements the exact `replica_flap_keeps_primary_stable` scenario contract above.
- [ ] `cucumber_tests/ha/features/planned_switchover_changes_primary_cleanly/planned_switchover_changes_primary_cleanly.feature` exists and implements the exact `planned_switchover_changes_primary_cleanly` scenario contract above.
- [ ] `cucumber_tests/ha/features/targeted_switchover_promotes_requested_replica/targeted_switchover_promotes_requested_replica.feature` exists and implements the exact `targeted_switchover_promotes_requested_replica` scenario contract above.
- [ ] Each of the six features has one tiny wrapper `.rs` file and one explicit `[[test]]` entry in `Cargo.toml`.
- [ ] Runner edits stay limited to the small harness growth listed in this task and do not introduce advanced harness features.
- [ ] The existing `primary_crash_rejoin` feature from task 01 is not reimplemented or duplicated here.
- [ ] All six feature wrappers can be executed on the greenfield harness.
- [ ] Each feature run produces enough evidence to distinguish a harness failure from an HA behavior failure in the system under test.
- [ ] If a scenario fails, the failure is captured as a product or HA failure after the harness has successfully started the cluster, injected the intended action, and recorded the expected topology or proof-row observations up to the failing assertion.
- [ ] `<passes>true</passes>` is set only after every acceptance criterion and required checkbox is complete.
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Add the six feature directories and wrappers
- [ ] Add the six feature directories named in this task.
- [ ] Add one `.feature` file per directory.
- [ ] Add one tiny Rust wrapper per directory.
- [ ] Register one `[[test]]` target per wrapper in `Cargo.toml`.

### Phase 2: Add only the small harness support this task allows
- [ ] Add named node kill and restart helpers.
- [ ] Add `pgtm` polling helpers for zero primary, one primary, same primary, and changed primary.
- [ ] Add topology assertions based on `pgtm primary` and `pgtm replicas`.
- [ ] Add reusable proof-table and proof-row helpers.
- [ ] Add `pgtm` helpers for normal and targeted switchover requests.
- [ ] Add simple timeline recording.

### Phase 3: Implement the exact scenario contracts
- [ ] Implement `replica_outage_keeps_primary_stable` exactly as written.
- [ ] Implement `two_node_outage_one_return_restores_quorum` exactly as written.
- [ ] Implement `full_cluster_outage_restore_quorum_then_converge` exactly as written.
- [ ] Implement `replica_flap_keeps_primary_stable` exactly as written.
- [ ] Implement `planned_switchover_changes_primary_cleanly` exactly as written.
- [ ] Implement `targeted_switchover_promotes_requested_replica` exactly as written.

### Phase 4: Verification and closeout
- [ ] Run targeted execution for each of the six new feature wrappers.
- [ ] For each wrapper run, record whether the result is:
- [ ] harness failure
- [ ] product or HA scenario failure
- [ ] successful scenario pass
- [ ] Fix harness failures until every feature can be executed to a trustworthy outcome.
- [ ] Do not defer feature creation just because one scenario currently exposes a product bug.
- [ ] Update this task file only after the work and verification are actually complete.
- [ ] Only after all required checkboxes are complete, set `<passes>true</passes>`.
- [ ] Run `/bin/bash .ralph/task_switch.sh`.
- [ ] Commit all required files, including `.ralph/` updates, with a task-finished commit message that includes verification evidence.
- [ ] Push with `git push`.

TO BE VERIFIED
