## Bug: Greenfield Rewind Fallback Scenario Never Attempts Pg Rewind <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>

<description>
The advanced greenfield HA wrapper `ha_rewind_failure_falls_back_to_basebackup` now executes to a trustworthy product outcome, but the product never attempts `pg_rewind`.

Observed on March 10, 2026 from:
- focused wrapper run: `cargo nextest run --profile ultra-long --no-fail-fast --target-dir /tmp/pgtuskmaster_rust-target --config build.incremental=false --test ha_rewind_failure_falls_back_to_basebackup`
- latest run artifacts:
  - `cucumber_tests/ha/runs/rewind_failure_falls_back_to_basebackup/rewind-failure-falls-back-to-basebackup-1773177630523-102079/artifacts/timeline.json`
  - `cucumber_tests/ha/runs/rewind_failure_falls_back_to_basebackup/rewind-failure-falls-back-to-basebackup-1773177630523-102079/artifacts/compose-logs.txt`

The harness now performs all of the following successfully before the failure:
- boots `three_node_plain`
- waits for a stable initial primary as `old_primary`
- enables the `pg_rewind` blocker on that node
- creates and writes the pre-failure proof row
- fully isolates `old_primary` from peers and observer API
- cuts `old_primary` off from DCS
- observes a different primary across the remaining 2 running nodes
- writes the post-failover proof row through that new primary
- heals the old primary's network faults

After that heal, the step `Then the node named "old_primary" emitted blocker evidence for "pg_rewind"` fails because the compose logs contain no `pg_rewind wrapper` invocation at all. The same log set does show `pg_basebackup wrapper executing real binary` for the initial replica clones, so log capture itself is working. This means the current recovery path either bypasses `pg_rewind` entirely or never reaches the intended rewind-or-fallback branch for a previously isolated old primary.

Explore and research the recovery/rejoin path first, then fix the product so a diverged old primary in this scenario either:
- attempts `pg_rewind` and falls back to basebackup when rewind fails, or
- intentionally chooses a different safe recovery path that is explicit, documented, and reflected in the scenario contract and operator-visible state.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
