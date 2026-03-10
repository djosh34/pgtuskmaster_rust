## Task: Add Low-Hanging HA Quorum Cucumber Features On The Greenfield Runner <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** After `01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md` lands, extend the greenfield cucumber HA suite with the quorum-survival scenarios from `story-ha-quorum-survival-under-real-failures` that can be implemented with the same runner primitives and no new infrastructure. Add each scenario as its own feature directory with one `.feature` file and one tiny Rust wrapper, keeping the work centered on readable operator-facing feature files rather than reviving old harness-specific detail.

**Original user shift / motivation:** The user said the existing HA quorum story tasks still carry too much old detail and assume the legacy harness shape. They want one new task in the greenfield cucumber story, created with `add-task-as-agent`, that harvests only the low-hanging HA quorum scenarios that fit the new runner. The user explicitly wants one scenario per feature, only minimal additions on top of the cucumber runner, and any runner edits beyond that small scope must be discussed first instead of silently expanding the harness.

**Higher-order goal:** Move the easy, high-signal HA quorum stories onto the new greenfield cucumber runner first so the project gains readable end-to-end coverage quickly, while keeping partitioning, fault injection, and other runner-expanding work isolated in later follow-up tasks.

**Scope:**
- Work only in the greenfield test surface introduced by task 01: `cucumber_tests/ha/features/`, `cucumber_tests/ha/support/...`, `Cargo.toml`, and the greenfield test docs such as `docs/src/how-to/run-tests.md` or the eventual greenfield HA test entrypoint docs.
- Add exactly one new feature directory plus one tiny wrapper for each approved low-hanging scenario:
- `cucumber_tests/ha/features/replica_outage_keeps_primary_stable/`
- `cucumber_tests/ha/features/two_node_outage_one_return_restores_quorum/`
- `cucumber_tests/ha/features/full_cluster_outage_restore_quorum_then_converge/`
- `cucumber_tests/ha/features/replica_flap_keeps_primary_stable/`
- Keep each feature to one scenario only. Do not bundle multiple stories into one feature file.
- Keep runner changes limited to small generalizations of the task 01 runner primitives: kill/start arbitrary node containers, wait for zero or one primary through `pgtm`, reuse proof-row SQL helpers, remember primary identity across steps, and record readable timeline events.
- Do not add new givens, new Compose topologies, network-partition controls, lag/fault injection, broken-startup wrappers, storage/WAL fault machinery, or any other new runner infrastructure in this task.
- Do not duplicate `cucumber_tests/ha/features/primary_crash_rejoin/`; task 01 already covers the “old primary dies, majority fails over, old primary returns as replica” story.
- If execution discovers that any candidate scenario needs more than the allowed small runner edits above, stop and split that work into a new follow-up task in `story-greenfield-cucumber-ha-harness/` instead of expanding this task.

**Context from research:**
- Task 01 defines the greenfield baseline: a separate `cucumber_tests/ha/` tree, static `givens/three_node_plain/`, Docker CLI orchestration, `pgtm`-only observation, `psql` proofing from `pgtm` DSNs, config-derived timeouts, and the first feature `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.feature`.
- The low-hanging scenarios from `story-ha-quorum-survival-under-real-failures` are the slices that reuse those same primitives:
- task 02: replica whole-node outage while the primary stays stable
- task 02 / task 06: two whole-node outage with exactly one node returning to restore quorum before full heal
- task 06: full-cluster outage followed by two-node restore, then final third-node convergence
- task 09: repeated replica flap while a healthy majority keeps the same primary
- Task 07 is already covered by task 01's primary-crash-and-rejoin feature and must not be reimplemented here.
- The remaining quorum-story tasks need runner capabilities that task 01 does not promise:
- task 03 and task 10 need full 1:2 network partition control
- task 04 needs storage/WAL fault injection
- task 05 and task 11 need deliberate broken-startup or broken-rejoin mechanics
- task 08 needs deterministic lagging/stale-replica setup
- The old story files point at legacy paths such as `tests/ha/support/*.rs`, `tests/ha_*.rs`, and `src/test_harness/ha_e2e/*.rs`. Those path assumptions are not valid implementation targets for this greenfield task and must not be copied into the new runner.
- The approved design choice from the user is to express the easy scenarios as feature-file additions first, one scenario per feature, and defer runner-expanding stories until later.

**Expected outcome:**
- The greenfield cucumber suite gains a small batch of readable HA quorum feature files that cover the easy, high-value stories without waiting for partition or fault-injection infrastructure.
- The new features all run on top of the task 01 primitives and remain visibly separate from the legacy HA harness.
- Later work is clearer because the low-hanging features are harvested now and every scenario that needs new infrastructure is explicitly left for a follow-up task instead of being smuggled into this one.

</description>

<acceptance_criteria>
- [ ] Task 01 (`01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md`) is complete enough that the actual greenfield file/module layout exists, and this task is implemented against that greenfield layout rather than against legacy `tests/ha` or `src/test_harness/ha_e2e` assumptions.
- [ ] `cucumber_tests/ha/features/replica_outage_keeps_primary_stable/replica_outage_keeps_primary_stable.feature` and its tiny wrapper `.rs` file exist, with exactly one scenario proving that killing a replica leaves the same primary stable, writable, and unique before the replica is restarted and catches up.
- [ ] `cucumber_tests/ha/features/two_node_outage_one_return_restores_quorum/two_node_outage_one_return_restores_quorum.feature` and its tiny wrapper `.rs` file exist, with exactly one scenario proving that after two node containers are down, one surviving node has no primary / fail-safe behavior, and starting exactly one stopped node restores one stable primary and successful proof writes before the third node returns.
- [ ] `cucumber_tests/ha/features/full_cluster_outage_restore_quorum_then_converge/full_cluster_outage_restore_quorum_then_converge.feature` and its tiny wrapper `.rs` file exist, with exactly one scenario proving that after all node containers are down, starting exactly two nodes restores one stable primary and successful proof writes before the final node returns, and the final node then converges as a replica without disturbing the elected primary.
- [ ] `cucumber_tests/ha/features/replica_flap_keeps_primary_stable/replica_flap_keeps_primary_stable.feature` and its tiny wrapper `.rs` file exist, with exactly one scenario proving that repeated kill/start cycles of a replica do not change the primary and do not interrupt proof writes on the stable primary.
- [ ] `Cargo.toml` registers one explicit `[[test]]` target for each new feature wrapper outside `tests/`, following the tiny-wrapper pattern established by task 01.
- [ ] Any runner edits under `cucumber_tests/ha/support/...` are documented up front before implementation starts, and the documented edit list stays limited to small greenfield-only plumbing: generic node kill/start helpers, zero-primary and same-primary polling via `pgtm`, reusable proof-row SQL helpers, and timeline/event recording. No new given, no new Compose file, no partition proxy, no lag/fault injector, no broken-node wrapper path, and no legacy harness import is introduced.
- [ ] `cucumber_tests/ha/features/primary_crash_rejoin/` is not duplicated or renamed as part of this task; this task extends the suite beyond the first feature instead of redoing it.
- [ ] If any of the four candidate scenarios turns out to need more than the allowed small runner edits, that scenario is left out of this task and replaced by a newly created follow-up task in `story-greenfield-cucumber-ha-harness/`; this task does not silently grow new infrastructure.
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
- [ ] Before editing code, add a short execution note to this task file or adjacent task notes that lists the exact runner changes needed for these four scenarios and confirms they all fit inside the pre-approved small-edit bucket:
- [ ] kill/start arbitrary node containers through the existing Docker CLI wrapper
- [ ] wait for zero primary, wait for one primary, and assert that the primary identity stayed the same through `pgtm`
- [ ] create/write/read proof rows through existing `psql` helper paths
- [ ] capture simple timeline/event messages for outage, restore, election, and convergence points
- [ ] If any scenario needs a new given, a new Compose topology, a partition mechanism, a lag/fault injector, a broken-startup path, or any other runner expansion, stop and create a new Ralph follow-up task for that scenario instead of continuing inside this task.

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
- [ ] Keep every wrapper tiny and nearly identical to the task 01 wrapper: import the shared feature runner only, register the feature path only, and keep scenario logic out of the wrapper file.
- [ ] Update `Cargo.toml` with one `[[test]]` entry per new wrapper.

### Phase 3: Generalize shared greenfield support without adding new infrastructure
- [ ] Update the actual shared greenfield world/runner module(s) under `cucumber_tests/ha/support/...` so steps can target arbitrary node identities instead of only “the current primary”.
- [ ] Extend the existing Docker helper module under `cucumber_tests/ha/support/docker/cli.rs` or its task-01 equivalent so scenarios can kill and restart one or more named node containers using the already-approved container lifecycle primitives.
- [ ] Extend the existing observer helper module under `cucumber_tests/ha/support/observer/pgtm.rs` or its task-01 equivalent with small polling helpers for:
- [ ] “there is no primary yet”
- [ ] “there is exactly one primary”
- [ ] “the primary is still node X”
- [ ] “node Y has rejoined as a replica”
- [ ] Extend the existing SQL helper module under `cucumber_tests/ha/support/observer/sql.rs` or its task-01 equivalent so the new features can reuse one proof-table/proof-row flow rather than open-coding SQL in each scenario.
- [ ] Keep all support edits greenfield-only. Do not import anything from `tests/`, `tests/ha/`, `tests/ha_*.rs`, or `src/test_harness/ha_e2e/`.
- [ ] Do not add new infrastructure modules for partitioning, storage faults, lag control, broken startup, or custom APIs in this task.

### Phase 4: Implement each low-hanging scenario as one feature
- [ ] Write `replica_outage_keeps_primary_stable.feature` so the operator story is: start `three_node_plain`, identify the stable primary, kill one replica container, prove the same primary remains the only primary and accepts a proof write, restart the killed replica, and prove it catches up as a replica.
- [ ] Implement or wire the shared step definitions for that feature using only current runner primitives and config-derived deadlines.
- [ ] Write `two_node_outage_one_return_restores_quorum.feature` so the operator story is: start `three_node_plain`, create proof state, kill two node containers, prove the lone survivor has no primary / is not writable as a primary, start exactly one stopped node, prove exactly one primary appears and accepts a proof write before the third node returns, then start the third node and prove final convergence.
- [ ] Implement or wire the shared step definitions for that feature using only current runner primitives and config-derived deadlines.
- [ ] Write `full_cluster_outage_restore_quorum_then_converge.feature` so the operator story is: start `three_node_plain`, create proof state, kill all node containers, start exactly two nodes, prove exactly one primary is restored and accepts a new proof write before the final node returns, then start the last node and prove it converges as a replica while the elected primary remains unchanged.
- [ ] Implement or wire the shared step definitions for that feature using only current runner primitives and config-derived deadlines.
- [ ] Write `replica_flap_keeps_primary_stable.feature` so the operator story is: start `three_node_plain`, record the stable primary, repeatedly kill and restart one replica container for a bounded number of cycles, perform proof writes during the flap window, and prove the primary identity never changes and the restarted replica eventually reconverges.
- [ ] Implement or wire the shared step definitions for that feature using only current runner primitives and config-derived deadlines.
- [ ] Keep all four features readable and operator-facing. Do not phrase the steps in terms of legacy helper names or old harness internals.

### Phase 5: Enforce the greenfield boundary and document the feature set
- [ ] Run repo-wide verification such as `rg -n "(tests/ha|tests/ha_|src/test_harness/ha_e2e)" cucumber_tests/ha Cargo.toml docs/src/how-to/run-tests.md` and confirm the new greenfield implementation does not import or depend on the old HA harness.
- [ ] Update `docs/src/how-to/run-tests.md` and any adjacent greenfield HA docs so they list the new feature directories, explain that each feature has one scenario and one tiny wrapper, and keep the greenfield suite separate from the legacy harness story.
- [ ] If `Makefile` or another documented entrypoint enumerates the greenfield feature wrappers explicitly, update it so the new features are runnable through the documented workflow.

### Phase 6: Verification and closeout
- [ ] Run targeted execution of each new greenfield feature so the four scenario flows are exercised independently on top of the task 01 runner.
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
