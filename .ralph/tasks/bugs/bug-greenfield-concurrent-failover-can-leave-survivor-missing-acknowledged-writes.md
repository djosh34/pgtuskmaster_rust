## Bug: Greenfield concurrent failover can leave survivor missing acknowledged writes <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>

<description>
The advanced greenfield wrapper `ha_stress_failover_concurrent_sql` now reaches a trustworthy data-convergence product failure under concurrent writes and primary loss.

Observed on March 10, 2026 from:
- focused wrapper run: `cargo nextest run --profile ultra-long --target-dir /tmp/pgtuskmaster_rust-target --config build.incremental=false --test ha_stress_failover_concurrent_sql`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for an authoritative stable primary
- created the workload table
- started the concurrent writer and waited until it had recorded at least one committed row
- killed the old primary
- observed exactly one different primary across the 2-node survivor set
- stopped the workload and recorded the committed token set
- inserted the post-failover proof row through the new primary

The scenario then failed on the survivor convergence check:
- step failure: `Then the 2 online nodes contain exactly the recorded proof rows`
- observed evidence on one surviving node: only the earliest workload rows were present, while later committed workload tokens and `post-failover-proof` never converged within the recovery deadline

This is trustworthy product evidence because the harness applied the intended workload-plus-failover choreography and only failed after it had a concrete set of acknowledged writes to verify. Explore and research the failover catch-up and acknowledged-write survival path first, then fix the product so the surviving nodes converge on all committed rows after concurrent-write failover.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
