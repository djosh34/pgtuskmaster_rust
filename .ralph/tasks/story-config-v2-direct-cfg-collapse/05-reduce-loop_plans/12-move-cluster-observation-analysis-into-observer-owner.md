## Plan: Move Cluster Observation Analysis Into Observer Owner

### Why this reduction target

The next wrong-place hotspot is `tests/ha/support/steps/mod.rs`:

- `tests/ha/support/steps/mod.rs` is still 1246 lines and mixes cucumber step declarations with a long block of cluster-observation analysis helpers: `compatible_primary_from_observation`, `relevant_member_states`, `require_observed_member_state`, `member_observation_failure`, `compatible_authoritative_primary`, `format_observed_authorities`, `replica_members`, `require_visible_members`, `format_warnings`, `format_authority`, and `authoritative_primary`.
- `tests/ha/support/observer/pgtm.rs` already owns `ClusterStateObservation` plus the mechanics for collecting `NodeState` observations from the cluster. The interpretation of those observations is data-owner behavior, not step-definition behavior.
- The current split means every HA step that wants “the compatible primary” or “the ready replicas” has to call back into a large helper block living beside unrelated step text, while the owner of the observation type remains a thin fetch-only wrapper.
- This is both smell 8 and smell 3 from `improve-code-boundaries`: too much in one file and wrong-placeism around a state owner that is missing its obvious analysis behavior.

### Current overlap already verified

- `tests/ha/support/observer/pgtm.rs` defines `ClusterStateObservation` and `PgtmObserver::observe_states`, so the observer layer already owns the raw state map that the helper block analyzes.
- `tests/ha/support/steps/mod.rs` lines 776-982 contain pure interpretation/rendering helpers over that observation data, not cucumber bindings.
- The bottom-of-file step tests already exercise helper-only behavior (`observed_healthy_members...`) separate from actual step wiring, which confirms the file is carrying non-step responsibilities.

### Execution plan

1. Move observation-owned analysis to the observer owner.
   - Replace the `ClusterStateObservation` type alias in `tests/ha/support/observer/pgtm.rs` with the single owning shape needed to host behavior directly.
   - Reuse the existing observation concept; do not introduce a second DTO or parallel wrapper hierarchy.
   - Move the pure observation interpretation/rendering helpers from `tests/ha/support/steps/mod.rs` onto that owner or immediately adjacent observer-owned helpers.

2. Narrow `steps/mod.rs` to step behavior.
   - Rewrite the step polling paths to ask the observer-owned API for the compatible primary, visible-member checks, replica list, and authority/warning rendering instead of manually stitching those helpers together.
   - Keep only the helpers that are genuinely step/harness specific, such as probe execution or step-local logging.

3. Relocate helper tests with the moved owner.
   - Move or rewrite the relevant helper-only tests so they validate the observer-owned behavior instead of the step file.
   - Prefer table-driven coverage over duplicating similar authority/replica assertions across multiple helpers.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4141 -6497 diff: -2356` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Tick the task boxes only after the gates pass.

### Guardrails

- Do not create a second “analysis” module if the existing observer owner can carry the behavior directly.
- If replacing the alias with a new owner shape starts adding pass-through wrapper methods faster than it deletes helper functions, keep the simpler map ownership and switch this plan back to `TO BE VERIFIED`.
- Keep `steps/mod.rs` focused on cucumber bindings, polling orchestration, and harness actions; static observation interpretation should not stay there.

NOW EXECUTE
