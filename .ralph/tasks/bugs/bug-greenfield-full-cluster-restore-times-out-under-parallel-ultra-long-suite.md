## Bug: Greenfield full-cluster restore times out under parallel ultra-long suite <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
`ha_full_cluster_outage_restore_quorum_then_converge` is not currently parallel-safe inside the real `make test-long` ultra-long suite.

Observed on March 10, 2026 from:
- focused wrapper pass: `cargo nextest run --workspace --profile ultra-long --no-fail-fast --no-tests fail --target-dir /tmp/pgtuskmaster_rust-target --config 'build.incremental=false' --failure-output immediate-final --final-status-level slow --status-level slow --test ha_full_cluster_outage_restore_quorum_then_converge`
- failing suite run: `make test-long`

The isolated wrapper passed, but the same scenario failed during the parallel `make test-long` gate with:
- step failure: `Then the node named "node-c" rejoins as a replica`
- observed error: `recovery deadline expired; last observed error: member 'node-c' role is 'unknown' instead of 'replica'`

During the failing suite run, the scenario still successfully:
- bootstrapped `three_node_plain`
- created and verified the proof table
- killed all database nodes
- restarted the fixed two-node subset
- restored exactly one primary across the two running nodes
- inserted `2:after-two-node-restore-before-final-node`
- confirmed the third node stayed offline during that write
- restarted the final node container

This is a trustworthy bug because the failure only appears under the actual parallel ultra-long suite that this story requires to be supported. Even if the underlying root cause overlaps with restarted-node rejoin behavior, the suite currently does not satisfy the contract that the shipped HA wrappers remain executable in parallel.

Explore and research the codebase first, then fix the full-cluster restore path so it remains reliable under the parallel `make test-long` suite rather than only in isolated wrapper execution.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
