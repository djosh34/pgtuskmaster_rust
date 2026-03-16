## Bug: HA stable-primary gate allows non-writable primary <status>completed</status> <passes>true</passes>

<blocked_by>Full completion of `.ralph/tasks/story-general-architecture-improvement-finding/06-task-move-ha-scenario-execution-into-a-per-scenario-runner-container-and-remove-docker-daemon-polling.md`</blocked_by>

<description>
`make test-long` can pass a stable-primary/recovery wait and then fail the immediately following proof write with `psql: connection refused`, which means the HA harness can report a healthy recovered primary before the cluster is actually healthy enough for writes.

This was observed in at least these ultra-long scenarios:
- `ha_dcs_and_api_faults_then_healed_cluster_converges`
- `ha_primary_loses_local_etcd_on_three_etcd_loses_authority_until_local_dcs_recovers`

In both cases the wait step returned success first, and the next `I insert proof row ...` step failed against the reported primary. Explore and research the harness codebase first, then fix the readiness contract so a reported stable/recovered primary is genuinely writable and not just briefly probeable. Prefer tightening the stable-primary gate and related target-resolution semantics over masking the issue with broad insert retries. Any retry behavior, if still needed, must stay bounded and must not hide a broken readiness gate.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Research summary
- The harness currently treats a “stable primary” as two separately sampled facts in `tests/ha/support/steps/mod.rs`: one authoritative primary in `NodeState`, and one `pgtm primary` target that happens to answer `SELECT 1`.
- That result is collapsed immediately back down to a bare `ClusterMember` alias, so the validated SQL endpoint is discarded as soon as the wait helper returns.
- Later proof writes do not have to reuse the validated endpoint. `insert_proof_row` resolves a fresh SQL path via `sql_target_for_member`, and that helper can fall back from pgtm-discovered routing to a direct member DSN.
- The concurrent workload path has the same lossy contract: `tests/ha/support/workload/mod.rs` returns a raw `(member_id, dsn)` tuple for the primary instead of a typed “validated writable primary target”.
- The observed `connection refused` immediately after a successful stable-primary wait is consistent with this split contract: the wait validated one endpoint at one instant, but the subsequent write was allowed to select a different path without any type-level proof that it was the same writable primary.

### Type design completed in this pass
- `tests/ha/support/world/mod.rs` now introduces a `WritablePrimaryTarget` ADT that binds the authoritative `ClusterMember` to the validated `ConnectionTarget`.
- HA aliases are being reshaped away from `alias -> ClusterMember` into `alias -> ScenarioAlias`, where an alias can be either a plain `Member` or a `WritablePrimary` carrying the validated target.
- The stable-primary wait helpers in `tests/ha/support/steps/mod.rs` now return `WritablePrimaryTarget` instead of a bare `ClusterMember`.
- The workload primary resolver in `tests/ha/support/workload/mod.rs` now returns `WritablePrimaryTarget` instead of an untyped `(member_id, dsn)` tuple.
- This intentionally breaks compilation. The next pass must thread the new alias/target ADTs through the step and workload code instead of recreating lossy primary aliases or tuple-based SQL target resolution.

### Execution plan
1. Finish the alias registry migration in `tests/ha/support/world/mod.rs` and `tests/ha/support/steps/mod.rs` so stable/recovered/current primary aliases retain their `WritablePrimaryTarget` instead of only a member id.
2. Replace the member-only proof-write target resolution with a typed resolver that prefers alias-carried `WritablePrimaryTarget` and makes any direct-member fallback an explicit non-primary path.
3. Tighten the stable-primary gate so `WritablePrimaryTarget` is only constructed after a bounded write probe succeeds against the same target that will be reused for the following proof write.
4. Push the same typed primary-target contract through `tests/ha/support/workload/mod.rs` so bounded concurrent writes use the same validated primary ADT instead of raw `(member_id, dsn)` tuples.
5. Add focused tests around the new alias/target contract and writable-primary gate, including targeted HA reruns for the previously flaky long scenarios if needed during iteration.
6. After the design still proves correct, run the required validation gates in order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
7. Only after all checks pass, update docs if needed using `k2-docs-loop`, then set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit, and push.

### Constraints for execution
- Do not restore the old `alias -> ClusterMember` shape for stable or recovered primary waits.
- Do not mask the bug with broad write retries; any retry logic must remain bounded and must not replace the readiness gate itself.
- If execution shows this ADT boundary is still wrong, switch this task back to `TO BE VERIFIED`, explain the design gap precisely, and stop immediately.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final `make` gates.

NOW EXECUTE

### Completion summary
- Stable-primary and recovery waits now produce a typed `WritablePrimaryTarget` instead of discarding the validated SQL route immediately after the wait step.
- Scenario aliases can now preserve either a plain member or a validated writable-primary route, and proof-row inserts reuse the validated route when writing through stable-primary aliases such as `"initial_primary"`, `"new_primary"`, or `"current_primary"`.
- The runner gained a `WritablePrimaryTls` contract command that verifies the authoritative primary and performs a bounded write probe on that same route before returning it to the HA harness.
- The workload path now uses the same typed writable-primary ADT instead of a raw `(member_id, dsn)` tuple, and unit coverage was added for the alias contract.
- Validation completed successfully with:
  - `make check`
  - `make test`
  - `make lint`
  - `make test-long`
