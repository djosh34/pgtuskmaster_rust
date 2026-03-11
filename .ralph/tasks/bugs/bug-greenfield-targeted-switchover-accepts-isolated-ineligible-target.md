## Bug: Greenfield targeted switchover accepts isolated ineligible target <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>

<description>
The advanced greenfield wrapper `ha_targeted_switchover_rejects_ineligible_member` now reaches a trustworthy product failure: a targeted switchover request is accepted even when the requested replica has been fully isolated from the cluster and observer API.

Observed on March 10, 2026 from:
- focused wrapper run: `cargo nextest run --profile ultra-long --target-dir /tmp/pgtuskmaster_rust-target --config build.incremental=false --test ha_targeted_switchover_rejects_ineligible_member`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for an authoritative stable primary
- selected a non-primary member as `ineligible_replica`
- fully isolated that replica from the cluster across DCS, API, and postgres paths and marked it unsampled

The failure then happened on the operator-visible request step:
- step failure: `And I attempt a targeted switchover to "ineligible_replica" and capture the operator-visible error`
- observed output: the request succeeded with `{"accepted": true}`

This is trustworthy product evidence because the scenario no longer relies on a weak lagging approximation; it makes the target clearly ineligible and still receives an accepted targeted switchover response. Explore and research the targeted-switchover validation path first, then fix the product so isolated or otherwise ineligible replicas are rejected with an operator-visible error instead of being accepted.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
