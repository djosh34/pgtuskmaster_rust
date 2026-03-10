## Task: Add Low-Hanging HA Quorum And Switchover Cucumber Features On The Greenfield Runner <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** After `01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md` lands, extend the greenfield cucumber HA suite with the easy scenarios that fit the same runner primitives and operator surface: low-hanging quorum stories plus planned switchover stories. Add each scenario as its own feature directory with one `.feature` file and one tiny Rust wrapper. Keep the work centered on readable operator-facing feature files, real `pgtuskmaster` nodes, and `pgtm` as the control and observation surface after startup.

**Original user shift / motivation:** The user wants the easy black-box HA stories moved out of the legacy `ha/e2e/half-bdd` world and into the new Docker-based cucumber harness. They explicitly want:
- one scenario per feature
- `pgtm` used as the operator surface for observation and switchover control
- planned switchover added to this task, not deferred and not replacing anything already in scope
- targeted switchover added as a separate feature with similar assertions
- no merging of the normal and targeted switchover stories into one feature
- runner edits limited to small extensions of the task 01 primitives, with the edits documented first rather than silently growing the harness

**Higher-order goal:** Move the easy, high-signal black-box HA stories onto the new greenfield cucumber runner first so the project gains readable end-to-end coverage quickly, keeps the operator model centered on `pgtm`, and leaves harder runner-expanding work to later dedicated tasks.

**Scope:**
- Work only in the greenfield test surface introduced by task 01: `cucumber_tests/ha/features/`, `cucumber_tests/ha/support/...`, `Cargo.toml`, `Makefile` if needed for feature registration, and greenfield HA test docs such as `docs/src/how-to/run-tests.md`.
- Add exactly one new feature directory plus one tiny wrapper for each approved low-hanging scenario:
- `cucumber_tests/ha/features/replica_outage_keeps_primary_stable/`
- `cucumber_tests/ha/features/two_node_outage_one_return_restores_quorum/`
- `cucumber_tests/ha/features/full_cluster_outage_restore_quorum_then_converge/`
- `cucumber_tests/ha/features/replica_flap_keeps_primary_stable/`
- `cucumber_tests/ha/features/planned_switchover_changes_primary_cleanly/`
- `cucumber_tests/ha/features/targeted_switchover_promotes_requested_replica/`
- Keep each feature to one scenario only. Do not combine the normal switchover and targeted switchover stories. Do not bundle multiple outage stories into one feature.
- Keep runner changes limited to small generalizations of the task 01 runner primitives:
- generic node kill/start helpers
- stable-primary and zero-primary polling through `pgtm`
- proof-row SQL helpers driven by `pgtm`-returned DSNs
- primary/replica topology assertions driven by `pgtm primary` and `pgtm replicas`
- switchover control through `pgtm switchover request` and `pgtm switchover request --switchover-to <member>`
- readable timeline/event recording
- Do not add new givens, new Compose topologies, network-partition controls, lag/fault injection, broken-startup wrappers, runtime-only restart machinery, storage/WAL fault machinery, or any other new runner infrastructure in this task.
- Do not duplicate `cucumber_tests/ha/features/primary_crash_rejoin/`; task 01 already covers the “old primary dies, majority fails over, old primary returns as replica” story.
- If execution discovers that any candidate scenario needs more than the allowed small runner edits above, stop and split that work into a new follow-up task in `story-greenfield-cucumber-ha-harness/` instead of expanding this task.

**Context from research:**
- Task 01 defines the greenfield baseline: a separate `cucumber_tests/ha/` tree, static `givens/three_node_plain/`, Docker CLI orchestration, `pgtm`-only observation, `psql` proofing from `pgtm` DSNs, config-derived timeouts, and the first feature `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.feature`.
- The low-hanging quorum scenarios that reuse those primitives are:
- the task-02-style replica outage while the primary stays stable
- the task-02/task-06 style two-node outage with exactly one node returning to restore quorum before full heal
- the task-06 style full-cluster outage followed by two-node restore and final convergence
- the task-09 style repeated replica flap while a healthy majority keeps the same primary
- The easy switchover scenarios that also fit the same greenfield runner are backed by existing old tests:
- `tests/ha_multi_node_failover.rs` `e2e_multi_node_cli_primary_and_replicas_follow_switchover`
- `tests/ha_multi_node_failover.rs` `e2e_multi_node_targeted_switchover_promotes_requested_replica`
- `pgtm` already exposes the required control surface for both:
- `pgtm switchover request`
- `pgtm switchover request --switchover-to <member>`
- Task 07 from the old quorum story is already covered by task 01's primary-crash-and-rejoin feature and must not be reimplemented here.
- The remaining harder quorum-story tasks need runner capabilities that task 01 does not promise:
- full 1:2 network partition control
- storage/WAL fault injection
- deliberate broken-startup or broken-rejoin mechanics
- deterministic lagging/stale-replica setup
- The old story files point at legacy paths such as `tests/ha/support/*.rs`, `tests/ha_*.rs`, and `src/test_harness/ha_e2e/*.rs`. Those path assumptions are not valid implementation targets for this greenfield task and must not be copied into the new runner.
- The approved design choice from the user is to express the easy scenarios as feature-file additions first, one scenario per feature, with `pgtm` used as the post-start operator surface.

**Expected outcome:**
- The greenfield cucumber suite gains a compact batch of readable HA feature files that cover the easy outage, quorum-restore, flap, planned switchover, and targeted switchover stories without waiting for partition or fault-injection infrastructure.
- The new features all run on top of the task 01 primitives, use `pgtm` for control and topology observation, and remain visibly separate from the legacy HA harness.
- Later work is clearer because the low-hanging features are harvested now and every scenario that needs new infrastructure is explicitly left for a follow-up task instead of being smuggled into this one.

</description>

<acceptance_criteria>
- [ ] Task 01 (`01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md`) is complete enough that the actual greenfield file/module layout exists, and this task is implemented against that greenfield layout rather than against legacy `tests/ha` or `src/test_harness/ha_e2e` assumptions.
- [ ] `cucumber_tests/ha/features/replica_outage_keeps_primary_stable/replica_outage_keeps_primary_stable.feature` and its tiny wrapper `.rs` file exist, with exactly one scenario proving that killing a replica leaves the same primary stable, writable, and unique before the replica is restarted and catches up.
- [ ] `cucumber_tests/ha/features/two_node_outage_one_return_restores_quorum/two_node_outage_one_return_restores_quorum.feature` and its tiny wrapper `.rs` file exist, with exactly one scenario proving that after two node containers are down, one surviving node has no primary / fail-safe behavior, and starting exactly one stopped node restores one stable primary and successful proof writes before the third node returns.
- [ ] `cucumber_tests/ha/features/full_cluster_outage_restore_quorum_then_converge/full_cluster_outage_restore_quorum_then_converge.feature` and its tiny wrapper `.rs` file exist, with exactly one scenario proving that after all node containers are down, starting exactly two nodes restores one stable primary and successful proof writes before the final node returns, and the final node then converges as a replica without disturbing the elected primary.
- [ ] `cucumber_tests/ha/features/replica_flap_keeps_primary_stable/replica_flap_keeps_primary_stable.feature` and its tiny wrapper `.rs` file exist, with exactly one scenario proving that repeated kill/start cycles of a replica do not change the primary and do not interrupt proof writes on the stable primary.
- [ ] `cucumber_tests/ha/features/planned_switchover_changes_primary_cleanly/planned_switchover_changes_primary_cleanly.feature` and its tiny wrapper `.rs` file exist, with exactly one scenario proving that a planned `pgtm switchover request` changes leadership to a different node, leaves exactly one primary, demotes the old primary to replica, and updates `pgtm primary` / `pgtm replicas` output to the new topology.
- [ ] `cucumber_tests/ha/features/targeted_switchover_promotes_requested_replica/targeted_switchover_promotes_requested_replica.feature` and its tiny wrapper `.rs` file exist, with exactly one scenario proving that `pgtm switchover request --switchover-to <member>` promotes the requested replica, does not promote the alternate healthy replica, leaves exactly one primary, and preserves proof-row convergence.
- [ ] `Cargo.toml` registers one explicit `[[test]]` target for each new feature wrapper outside `tests/`, following the tiny-wrapper pattern established by task 01.
- [ ] Any runner edits under `cucumber_tests/ha/support/...` are documented up front before implementation starts, and the documented edit list stays limited to small greenfield-only plumbing: generic node kill/start helpers, zero-primary and same-primary polling via `pgtm`, reusable proof-row SQL helpers, `pgtm` primary/replicas topology assertions, `pgtm` switchover request helpers, and timeline/event recording. No new given, no new Compose file, no partition proxy, no lag/fault injector, no broken-node wrapper path, and no legacy harness import is introduced.
- [ ] `cucumber_tests/ha/features/primary_crash_rejoin/` is not duplicated or renamed as part of this task; this task extends the suite beyond the first feature instead of redoing it.
- [ ] If any of the six candidate scenarios turns out to need more than the allowed small runner edits, that scenario is left out of this task and replaced by a newly created follow-up task in `story-greenfield-cucumber-ha-harness/`; this task does not silently grow new infrastructure.
- [ ] Docs are updated so the greenfield HA entrypoint and feature inventory reflect the new feature files and do not imply that the new suite reuses the old HA harness.
- [ ] `make check` passes cleanly
- [ ] `make test` passes cleanly
- [ ] `make test-long` passes cleanly
- [ ] `make lint` passes cleanly
- [ ] `<passes>true</passes>` is set only after every acceptance-criteria item and every required implementation-plan checkbox is complete
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Confirm dependency and freeze the allowed runner scope
- [ ] Wait for task 01 to land, then inspect the actual greenfield files under `cucumber_tests/ha/`, `Cargo.toml`, `Makefile`, and `docs/src/how-to/run-tests.md` so the rest of this task names the real modules and does not guess past task 01's implementation.
- [ ] Before editing code, add a short execution note to this task file or adjacent task notes that lists the exact runner changes needed for these six scenarios and confirms they all fit inside the pre-approved small-edit bucket:
- [ ] kill/start arbitrary node containers through the existing Docker CLI wrapper
- [ ] wait for zero primary, wait for one primary, and assert that the primary identity stayed the same or changed through `pgtm`
- [ ] create/write/read proof rows through existing `psql` helper paths
- [ ] assert topology through `pgtm primary` and `pgtm replicas`
- [ ] request normal and targeted switchover through `pgtm switchover request`
- [ ] capture simple timeline/event messages for outage, restore, switchover, election, and convergence points
- [ ] If any scenario needs a new given, a new Compose topology, a partition mechanism, a lag/fault injector, a broken-startup path, a runtime-only restart path, or any other runner expansion, stop and create a new Ralph follow-up task for that scenario instead of continuing inside this task.

### Phase 2: Add the new feature directories and tiny wrappers
- [ ] Create `cucumber_tests/ha/features/replica_outage_keeps_primary_stable/` with:
- [ ] `replica_outage_keeps_primary_stable.feature`
- [ ] `replica_outage_keeps_primary_stable.rs`
- [ ] Create `cucumber_tests/ha/features/two_node_outage_one_return_restores_quorum/` with:
- [ ] `two_node_outage_one_return_restores_quorum.feature`
- [ ] `two_node_outage_one_return_restores_quorum.rs`
- [ ] Create `cucumber_tests/ha/features/full_cluster_outage_restore_quorum_then_converge/` with:
- [ ] `full_cluster_outage_restore_quorum_then_converge.feature`
- [ ] `full_cluster_outage_restore_quorum_then_converge.rs`
- [ ] Create `cucumber_tests/ha/features/replica_flap_keeps_primary_stable/` with:
- [ ] `replica_flap_keeps_primary_stable.feature`
- [ ] `replica_flap_keeps_primary_stable.rs`
- [ ] Create `cucumber_tests/ha/features/planned_switchover_changes_primary_cleanly/` with:
- [ ] `planned_switchover_changes_primary_cleanly.feature`
- [ ] `planned_switchover_changes_primary_cleanly.rs`
- [ ] Create `cucumber_tests/ha/features/targeted_switchover_promotes_requested_replica/` with:
- [ ] `targeted_switchover_promotes_requested_replica.feature`
- [ ] `targeted_switchover_promotes_requested_replica.rs`
- [ ] Keep every wrapper tiny and nearly identical to the task 01 wrapper: import the shared feature runner only, register the feature path only, and keep scenario logic out of the wrapper file.
- [ ] Update `Cargo.toml` with one `[[test]]` entry per new wrapper.

### Phase 3: Generalize shared greenfield support without adding new infrastructure
- [ ] Update the actual shared greenfield world/runner module(s) under `cucumber_tests/ha/support/...` so steps can target arbitrary node identities instead of only “the current primary”.
- [ ] Extend the existing Docker helper module under `cucumber_tests/ha/support/docker/cli.rs` or its task-01 equivalent so scenarios can kill and restart one or more named node containers using the already-approved container lifecycle primitives.
- [ ] Extend the existing observer helper module under `cucumber_tests/ha/support/observer/pgtm.rs` or its task-01 equivalent with small polling helpers for:
- [ ] “there is no primary yet”
- [ ] “there is exactly one primary”
- [ ] “the primary is still node X”
- [ ] “the primary changed away from node X”
- [ ] “node Y has rejoined as a replica”
- [ ] “`pgtm primary` points at node X”
- [ ] “`pgtm replicas` contains exactly the expected replica set”
- [ ] Extend the existing control helper layer under `cucumber_tests/ha/support/...` so the runner can invoke:
- [ ] `pgtm switchover request`
- [ ] `pgtm switchover request --switchover-to <member>`
- [ ] Extend the existing SQL helper module under `cucumber_tests/ha/support/observer/sql.rs` or its task-01 equivalent so the new features can reuse one proof-table/proof-row flow rather than open-coding SQL in each scenario.
- [ ] Keep all support edits greenfield-only. Do not import anything from `tests/`, `tests/ha/`, `tests/ha_*.rs`, or `src/test_harness/ha_e2e/`.
- [ ] Do not add new infrastructure modules for partitioning, storage faults, lag control, broken startup, runtime-only restart, or custom APIs in this task.

### Phase 4: Implement each low-hanging outage scenario as one feature
- [ ] Write `replica_outage_keeps_primary_stable.feature` so the operator story is: start `three_node_plain`, identify the stable primary, kill one replica container, prove the same primary remains the only primary and accepts a proof write, restart the killed replica, and prove it catches up as a replica.
- [ ] Implement or wire the shared step definitions for that feature using only current runner primitives and config-derived deadlines.
- [ ] Write `two_node_outage_one_return_restores_quorum.feature` so the operator story is: start `three_node_plain`, create proof state, kill two node containers, prove the lone survivor has no primary / is not writable as a primary, start exactly one stopped node, prove exactly one primary appears and accepts a proof write before the third node returns, then start the third node and prove final convergence.
- [ ] Implement or wire the shared step definitions for that feature using only current runner primitives and config-derived deadlines.
- [ ] Write `full_cluster_outage_restore_quorum_then_converge.feature` so the operator story is: start `three_node_plain`, create proof state, kill all node containers, start exactly two nodes, prove exactly one primary is restored and accepts a new proof write before the final node returns, then start the last node and prove it converges as a replica while the elected primary remains unchanged.
- [ ] Implement or wire the shared step definitions for that feature using only current runner primitives and config-derived deadlines.
- [ ] Write `replica_flap_keeps_primary_stable.feature` so the operator story is: start `three_node_plain`, record the stable primary, repeatedly kill and restart one replica container for a bounded number of cycles, perform proof writes during the flap window, and prove the primary identity never changes and the restarted replica eventually reconverges.
- [ ] Implement or wire the shared step definitions for that feature using only current runner primitives and config-derived deadlines.

### Phase 5: Implement the new switchover scenarios as separate features
- [ ] Write `planned_switchover_changes_primary_cleanly.feature` so the operator story is: start `three_node_plain`, record the stable primary, create proof state, run `pgtm switchover request`, prove a different node becomes the only primary, prove the old primary remains online and demotes to replica, prove `pgtm primary` and `pgtm replicas` show the new topology, and prove post-switchover writes replicate to every node.
- [ ] Implement or wire the shared step definitions for the normal switchover feature using only current runner primitives and config-derived deadlines.
- [ ] Write `targeted_switchover_promotes_requested_replica.feature` so the operator story is: start `three_node_plain`, record the stable primary and the two replicas, select one replica as the requested successor, create proof state, run `pgtm switchover request --switchover-to <requested-replica>`, prove the requested replica becomes the only primary, prove the alternate healthy replica does not become primary, prove the old primary demotes to replica, and prove post-switchover writes replicate to every node.
- [ ] Implement or wire the shared step definitions for the targeted switchover feature using only current runner primitives and config-derived deadlines.
- [ ] Keep the planned and targeted switchover stories as separate features. Do not collapse them into one scenario.

### Phase 6: Enforce the greenfield boundary and document the feature set
- [ ] Run repo-wide verification such as `rg -n "(tests/ha|tests/ha_|src/test_harness/ha_e2e)" cucumber_tests/ha Cargo.toml docs/src/how-to/run-tests.md` and confirm the new greenfield implementation does not import or depend on the old HA harness.
- [ ] Update `docs/src/how-to/run-tests.md` and any adjacent greenfield HA docs so they list the new feature directories, explain that each feature has one scenario and one tiny wrapper, describe `pgtm` as the control and observation surface, and keep the greenfield suite separate from the legacy harness story.
- [ ] If `Makefile` or another documented entrypoint enumerates the greenfield feature wrappers explicitly, update it so the new features are runnable through the documented workflow.

### Phase 7: Verification and closeout
- [ ] Run targeted execution of each new greenfield feature so the six scenario flows are exercised independently on top of the task 01 runner.
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
