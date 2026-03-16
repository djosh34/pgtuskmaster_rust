## Bug: HA stable-primary gate allows non-writable primary <status>not_started</status> <passes>false</passes>

<blocked_by>Full completion of `.ralph/tasks/story-general-architecture-improvement-finding/06-task-move-ha-scenario-execution-into-a-per-scenario-runner-container-and-remove-docker-daemon-polling.md`</blocked_by>

<description>
`make test-long` can pass a stable-primary/recovery wait and then fail the immediately following proof write with `psql: connection refused`, which means the HA harness can report a healthy recovered primary before the cluster is actually healthy enough for writes.

This was observed in at least these ultra-long scenarios:
- `ha_dcs_and_api_faults_then_healed_cluster_converges`
- `ha_primary_loses_local_etcd_on_three_etcd_loses_authority_until_local_dcs_recovers`

In both cases the wait step returned success first, and the next `I insert proof row ...` step failed against the reported primary. Explore and research the harness codebase first, then fix the readiness contract so a reported stable/recovered primary is genuinely writable and not just briefly probeable. Prefer tightening the stable-primary gate and related target-resolution semantics over masking the issue with broad insert retries. Any retry behavior, if still needed, must stay bounded and must not hide a broken readiness gate.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
