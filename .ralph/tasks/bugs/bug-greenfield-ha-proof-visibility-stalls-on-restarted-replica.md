## Bug: Greenfield HA Proof Visibility Stalls On Restarted Replica <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>

<description>
The greenfield cucumber HA feature now gets through bootstrap, failover, proof write on the new primary, and restarted-node rejoin, but the final proof-read from the restarted replica is still broken.

Detection context on March 10, 2026:
- `make test-cucumber-ha-primary-crash-rejoin` reached the final proof-visibility step after the runtime-side role provisioning fixes
- one run failed fast with `ERROR: relation "ha_cucumber_proof" does not exist`, which shows the restarted node can report as a replica before the post-failover write is visible there
- after changing the harness to poll instead of checking once, a later run stalled for minutes instead of converging, which points to an unbounded operator-connection/readiness problem rather than a one-shot timing mistake

The executor should explore the codebase first, then fix the underlying issue rather than adding a flaky harness sleep.

Important code areas already implicated:
- `cucumber_tests/ha/support/steps/mod.rs`
- `cucumber_tests/ha/support/observer/sql.rs`
- `src/cli/connect.rs`
- `src/runtime/node.rs`

The likely product-level causes to investigate are:
- `pgtm` connection DSNs do not carry an explicit `connect_timeout`, so observer-driven `psql` calls can stall when a just-restarted node is not yet ready for clean reads
- the runtime may be advertising/accepting replica state before post-failover data-plane convergence is sufficient for the proof-read contract used by the feature
- the greenfield harness may need a more precise “replica is read-ready for post-failover proof data” condition that still stays within the `pgtm` + `psql` observer model
</description>

<acceptance_criteria>
- [ ] the restarted-replica proof-read path in the greenfield HA cucumber feature becomes bounded and deterministic instead of hanging or failing on a transient missing relation
- [ ] `cucumber_tests/ha/support/steps/mod.rs` and any related observer plumbing use a clear, source-backed readiness/proof-visibility strategy instead of a flaky one-shot read
- [ ] if `pgtm` connection resolution is part of the fix, `src/cli/connect.rs` is updated and covered by focused tests
- [ ] docs are updated if operator-visible `pgtm` connection output or HA proof semantics change
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] if the fix touches ultra-long HA behavior or its selection: `make test-long` — passes cleanly
</acceptance_criteria>
