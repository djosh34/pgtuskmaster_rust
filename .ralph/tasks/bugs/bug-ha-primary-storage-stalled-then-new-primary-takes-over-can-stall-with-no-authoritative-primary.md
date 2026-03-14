## Bug: HA storage-stall failover scenario can stall with no authoritative primary <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
`make test-long` is currently not reliably green because `ha_primary_storage_stalled_then_new_primary_takes_over` can fail waiting for a replacement primary.

This bug was detected during validation for `.ralph/tasks/story-general-architecture-improvement-finding/01-task-find-general-architecture-privacy-and-deduplication-improvements-and-create-follow-up-tasks.md`, which only changed markdown task files. `make check`, `make lint`, and `make test` all passed, but `make test-long` failed twice in the same scenario. One targeted rerun of the single scenario passed in between, which suggests the current failure mode may be timing-sensitive or flaky, but it is still a real repo bug because the required long suite is not stable.

Observed failure details from `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha__ha_primary_storage_stalled_then_new_primary_takes_over.log`:
- feature: `tests/ha/features/ha_primary_storage_stalled_then_new_primary_takes_over/ha_primary_storage_stalled_then_new_primary_takes_over.feature`
- failing step: `Then I wait for a different stable primary than "initial_primary" as "final_primary"`
- captured error: `failover deadline expired; last observed error: cluster has no authoritative primary; authority=no_primary(leaseopen) warnings=authority=no_primary(leaseopen)`

The next agent must explore and research the codebase first, then fix. Do not assume the issue is only in the test. Investigate HA decision/state publication, DCS authority handling, and/or harness timing around the wedged-primary path before choosing a solution.
</description>

<acceptance_criteria>
- [ ] Reproduce and understand why `ha_primary_storage_stalled_then_new_primary_takes_over` can end in `authority=no_primary(leaseopen)` instead of converging to a new stable primary.
- [ ] Fix the underlying issue in product code and/or the HA harness only after research shows which layer is actually wrong.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
