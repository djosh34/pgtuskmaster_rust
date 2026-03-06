## Task: Move and split HA e2e tests after the functional rewrite <status>done</status> <passes>true</passes>

<description>
After the HA functional rewrite lands, move and restructure the HA end-to-end tests so they are no longer oversized mixed files living under `src/ha/`.

The agent must explore the rewritten HA tests and current repo structure first, then implement the following fixed product decisions:
- this task happens after the other HA migration tasks in this story
- large HA e2e scenario files should be moved out of `src/ha/` into the appropriate `tests/` structure
- oversized mixed scenario files should be split into clearer files/directories by concern or scenario family
- overlapping fixtures and helpers should be consolidated where that improves clarity
- continuous invariant observer coverage added in the prior HA test task must remain intact after the move/split
- the resulting HA test tree should be easier to understand and navigate than the current `src/ha/e2e_*` layout

This task is about test structure and placement after the architectural rewrite, not about preserving the current mixed layout.

This is the only task in this story that may run `make test-long` or direct long HA cargo-test invocations. Earlier tasks in the story must explicitly skip long-test execution and defer it here.

The agent should use parallel subagents after exploration for file moves/splitting, fixture consolidation, and final test verification.
</description>

<acceptance_criteria>
- [x] HA e2e tests are moved out of `src/ha/` into an appropriate `tests/` layout
- [x] Large mixed HA e2e files are split into clearer files and directories
- [x] Overlapping fixtures/helpers are consolidated where it improves clarity
- [x] Continuous invariant-observer coverage remains intact after the restructure
- [x] The resulting test layout is easier to understand and navigate than the current `src/ha/e2e_*` structure
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly; this final story task owns the deferred long-test validation for the whole story
</acceptance_criteria>

<execution_plan>
## Detailed Execution Plan (Draft 1, 2026-03-06)

1. Reconfirm the actual baseline and the constraints that matter before moving anything
- The current HA real-binary coverage still lives in two oversized unit-test modules under `src/ha/`: `src/ha/e2e_multi_node.rs` and `src/ha/e2e_partition_chaos.rs`.
- `src/ha/mod.rs` still wires both as `#[cfg(test)]` unit-test modules, and `src/ha/test_observer.rs` is also test-only there today.
- The current contributor docs already contain stale path assumptions:
  - `docs/src/contributors/testing-system.md`
  - `docs/src/contributors/ha-pipeline.md`
  - `docs/src/contributors/codebase-map.md`
- The current `Makefile` uses exact `ULTRA_LONG_TESTS` names that point at unit-test paths such as `ha::e2e_multi_node::...` and `ha::e2e_partition_chaos::...`; moving these tests into `tests/` will change those names, so execution must not guess the replacement tokens.
- Continuous invariant observation is a hard requirement, not an optional nice-to-have. The move is only valid if the fail-closed sampling/evidence behavior survives intact.

2. Choose the target test topology up front instead of doing a literal file move
- Do not move the two large files into `tests/` as two new monoliths. The task is specifically to improve structure and navigation.
- The target should be a dedicated HA integration-test area under `tests/` with:
  - small top-level integration test entry files grouped by scenario family
  - a shared `tests/ha/support/` module tree for reusable helpers
- The intended split after execution is:
  - multi-node scenario entry files separated by concern family, not one mixed scenario bucket
  - partition-chaos scenario entry files separated by fault family / recovery family
  - shared support modules for fixtures, observers, SQL workload helpers, artifact writing, and assertion helpers
- Concretely, the execution pass should aim for a structure close to:
  - `tests/ha_multi_node_failover.rs`
  - `tests/ha_multi_node_stress.rs`
  - `tests/ha_partition_isolation.rs`
  - `tests/ha_partition_recovery.rs`
  - `tests/ha/support/...`
- Exact filenames may shift slightly during execution if `cargo test -- --list` naming or module ergonomics make a nearby variant cleaner, but the final shape must clearly improve discoverability over the current two-file `src/ha/e2e_*` layout.

3. Solve the integration-test visibility problem explicitly before touching scenario bodies
- Integration tests in `tests/` cannot use the current `#[cfg(test)] pub(crate) mod test_harness;` or `#[cfg(test)] pub(crate) mod ha::test_observer;` arrangement as-is.
- Execution must introduce a deliberate support boundary that integration tests can compile against.
- Preferred direction:
  - keep HA observer/scenario helpers in test-side support under `tests/ha/support/`
  - extract the reusable real-binary harness primitives out of `#[cfg(test)]` crate internals into a dedicated dev-only workspace helper crate (`crates/test_harness` or similarly named) that both crate-local tests and `tests/` integration tests can depend on
- The shared harness crate should own only the primitives the HA integration tests actually need:
  - namespace management
  - real binary discovery / provenance checks
  - etcd cluster handles
  - PostgreSQL helpers
  - network proxy helpers
  - HA cluster startup / handle APIs
- `src/test_harness/` in the main crate should either become a thin compatibility layer for remaining crate-local tests during the migration or disappear entirely if all callers move cleanly in the same pass.
- Do not expose the old `src/ha/test_observer.rs` as production-facing HA runtime API just to make the move compile. Observer logic belongs with the tests, not the runtime.
- Only fall back to a doc-hidden public `src/` support surface if the helper-crate extraction proves concretely worse during execution; if that fallback is chosen, the execution notes and docs must justify why widening the library test surface was necessary.

4. Consolidate shared HA e2e support into explicit support modules
- Move the reusable observer logic out of `src/ha/test_observer.rs` into `tests/ha/support/observer.rs` or an equivalently obvious test-only location.
- Preserve the current observer responsibilities:
  - sample tracking
  - API error accounting
  - recent evidence ring
  - leader-change counting
  - fail-safe sample counting
  - fail-closed finalization when evidence is insufficient
  - dual-primary detection from both API samples and SQL-role samples
- Extract shared fixture logic from the current large scenario files into support modules with clear ownership, for example:
  - `tests/ha/support/multi_node_fixture.rs`
  - `tests/ha/support/partition_fixture.rs`
  - `tests/ha/support/sql_workload.rs`
  - `tests/ha/support/artifacts.rs`
- Keep support modules strongly typed and explicit about error propagation. No `unwrap`, `expect`, or panic-driven helpers are allowed during the extraction.

5. Split the multi-node scenarios by scenario family instead of keeping them interleaved
- The current `src/ha/e2e_multi_node.rs` mixes:
  - fixture/bootstrap lifecycle
  - SQL workload machinery
  - artifact writing
  - continuous observation helpers
  - unassisted failover
  - planned switchover stress
  - unassisted failover stress
  - strict no-quorum fail-safe checks
  - fencing/integrity verification
- Execution should move those scenarios into focused integration-test entry files with a shared fixture/support layer.
- The split should keep related scenarios together, for example:
  - failover / continuity scenarios in one entry file
  - stress + concurrent SQL scenarios in one entry file
  - no-quorum / fail-safe / fencing scenarios in one entry file if that reads better than spreading them further
- The key requirement is that a future engineer can find “multi-node stress with workload” vs “strict no-quorum behavior” vs “failover continuity” without scanning a 3000-line file.
- Preserve artifact schemas and timeline-writing behavior unless execution finds a concrete reason to correct them alongside docs.

6. Split the partition-chaos scenarios by fault family and healing behavior
- The current `src/ha/e2e_partition_chaos.rs` mixes:
  - fixture/bootstrap logic
  - proxy fault injection plumbing
  - observation bookkeeping
  - minority isolation behavior
  - primary isolation failover behavior
  - API-path isolation behavior
  - mixed-fault heal-and-converge behavior
- Execution should move these into focused integration-test entry files backed by shared partition fixture support.
- The intended grouping is:
  - isolation-focused scenarios together
  - mixed-fault / heal / convergence scenarios together
- Keep the no-split-brain assertion sourced from the continuous observer window rather than converting the move into final-state-only assertions.
- Preserve the current proxy controls and timeline artifact detail because those are part of the diagnosis story when these tests fail.

7. Update module wiring and remove the old `src/ha` e2e placement cleanly
- Remove `mod e2e_multi_node;`, `mod e2e_partition_chaos;`, and the old test-observer wiring from `src/ha/mod.rs` once the new integration test layout is in place.
- Delete the old `src/ha/e2e_multi_node.rs`, `src/ha/e2e_partition_chaos.rs`, and `src/ha/test_observer.rs` files after their logic is relocated and verified.
- Make sure no contributor-facing docs still describe HA real-binary coverage as living under `src/ha/e2e_*`.
- The final codebase should reflect the product decision that these are black-box / real-binary integration tests, not unit tests embedded in the HA runtime module tree.

8. Make the Makefile and test-name plumbing follow the new integration-test reality
- After the new files exist, run `cargo test --all-targets -- --list` and capture the exact discovered test names for all moved ultra-long HA tests.
- Update `Makefile` `ULTRA_LONG_TESTS` to the new exact names emitted by Cargo after the move into `tests/`.
- Re-run the `-- --list` preflight mentally against the Makefile expectations:
  - every ultra-long token must exist exactly once
  - the default suite must remain non-empty after skips
- Do not leave old `ha::e2e_*` tokens in the Makefile after the move; `make test` and `make test-long` are designed to fail closed on stale names.

9. Update docs in the same pass and remove stale references instead of preserving them
- Update at least:
  - `docs/src/contributors/testing-system.md`
  - `docs/src/contributors/ha-pipeline.md`
  - `docs/src/contributors/codebase-map.md`
- Also sweep for stale references outside those three pages, because the repo already contains verification/docs artifacts that still cite `src/ha/e2e_*`:
  - `docs/src/contributors/harness-internals.md`
  - `docs/src/verification/task-33-docs-verification-report.md`
- The docs should describe the new truth:
  - real-binary HA e2e coverage lives under `tests/`
  - shared HA integration support lives under `tests/ha/support/` plus the extracted dev-only harness crate
  - continuous invariant observation remains part of the HA scenario design
- During execution, search for `src/ha/e2e_` and `src/ha/test_observer.rs` across docs and remove or rewrite stale references rather than leaving them behind.
- If the repo requires checked-in generated docs/book artifacts for doc changes, regenerate/update them in the same pass instead of allowing the source/book pair to drift.

10. Use parallel subagents during the execution pass, but only after the support boundary is decided
- Main lane:
  - establish the integration-test-visible support boundary
  - wire/remove `src/ha` module references
  - update `Makefile` exact ultra-long test names
- Parallel worker lane 1:
  - move/split multi-node scenarios and shared multi-node helpers
- Parallel worker lane 2:
  - move/split partition-chaos scenarios and shared partition helpers
- Parallel worker lane 3:
  - perform the doc/reference sweep after the new layout settles
- Final verification should stay centrally coordinated so the exact gate evidence and any failures are interpreted in one place before completion markers are written.

11. Planned verification sequence for the later `NOW EXECUTE` pass
- First run targeted checks while editing:
  - focused `cargo test --all-targets -- --list`
  - focused runs for the new HA integration test entry files
  - focused runs for any synthetic observer/support tests added during the move
- Then run the required repo gates in this exact order:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- Do not tick the acceptance boxes, set `<passes>true</passes>`, run `.ralph/task_switch.sh`, commit, or push until all four gates pass cleanly.

12. Completion protocol to follow once execution actually succeeds
- Tick every acceptance checkbox in this task file.
- Set `<passes>true</passes>` only after `make check`, `make test`, `make test-long`, and `make lint` are all green.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all tracked and generated changes, including `.ralph` updates, with:
  - `task finished 06-task-move-and-split-ha-e2e-tests-after-functional-rewrite: ...`
- Include in the commit message:
  - what moved
  - how the test layout changed
  - the exact verification gates that passed
  - any meaningful implementation challenge such as the integration-test harness visibility change
- Push with `git push`.

</execution_plan>

NOW EXECUTE
