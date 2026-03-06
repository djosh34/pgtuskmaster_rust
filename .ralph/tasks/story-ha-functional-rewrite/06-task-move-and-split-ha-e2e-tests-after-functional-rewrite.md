---
## Task: Move and split HA e2e tests after the functional rewrite <status>not_started</status> <passes>false</passes>

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
- [ ] HA e2e tests are moved out of `src/ha/` into an appropriate `tests/` layout
- [ ] Large mixed HA e2e files are split into clearer files and directories
- [ ] Overlapping fixtures/helpers are consolidated where it improves clarity
- [ ] Continuous invariant-observer coverage remains intact after the restructure
- [ ] The resulting test layout is easier to understand and navigate than the current `src/ha/e2e_*` structure
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly; this final story task owns the deferred long-test validation for the whole story
</acceptance_criteria>
