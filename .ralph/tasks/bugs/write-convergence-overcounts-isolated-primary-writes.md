## Bug: Write convergence overcounts isolated-primary writes during failover and cleanup <status>completed</status> <passes>true</passes>

<description>
`make test-long` exposed a second write-convergence failure mode while executing the health-check race refactor.

In `ha_primary_faults_fail_over_then_recover::primary_isolated_from_majority` and `ha_rejoin_and_restart_recovery::rewind_failure_old_primary_rejoins`, the background invariant expected one more committed write than the surviving majority ever converged (`expected 19/21`, observed `18/20` on the surviving nodes and the isolated primary unavailable). In `ha_quorum_loss_and_dcs_loss::lone_survivor_with_only_local_dcs`, the invariant also kept demanding convergence while two members were intentionally gone and not exposing Postgres ports.

Explore the invariant runner and the harness cleanup/background-check path first, then fix the boundary so a locally committed write on a doomed primary is not treated as a cluster-wide durability baseline and unreachable members do not cause false invariant failures in intentionally unhealthy scenarios.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Boundary diagnosis
- This bug is no longer about the health-step member slice alone. The deeper mismatch is that the runner implementation does not match the contract already documented in [docs/src/explanation/failure-modes.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/explanation/failure-modes.md): accepted writes are supposed to be writes routed through the authoritative primary, but [tests/ha/support/invariants/write_convergence.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/invariants/write_convergence.rs) currently creates one independent writer loop per member and treats any local commit as part of the global durability baseline.
- That wrong boundary explains both observed failure classes:
  - in `primary_isolated_from_majority` and `rewind_failure_old_primary_rejoins`, the isolated old primary can still increment its own local row before failover/rewind discards that branch, so the runner overcounts writes the surviving majority never accepted as durable cluster progress;
  - in `lone_survivor_with_only_local_dcs`, cleanup/liveness still behaves as if every per-member writer must remain reconnectable, so intentionally down or unreachable members turn into false invariant failures even though the scenario is supposed to end unhealthy.
- Using the `improve-code-boundaries` skill points to `Smell 7: Stop Overengineering` here. The current member-writer thread set, write gate, and cleanup coupling model too much machinery around the wrong fact. The invariant does not need "one perpetual local writer per node"; it needs "track writes accepted through the authoritative primary, then verify convergence for the selected reachable members when a caller asks for the strong check".
- Existing repo shapes are already sufficient for the right split:
  - `PgtmObserver` plus `authoritative_primary_member(...)` already give the authoritative routing boundary;
  - `WriteConvergenceInvariantRunner::ensure_healthy(...)` versus `ensure_running()` already express the strong-check versus liveness-check caller split;
  - the healthy step already threads a concrete `Vec<ClusterMember>` selected from a successful poll, so execution should reuse that flat slice instead of adding another public wrapper type.
- The codebase search did not reveal a reusable accepted-write ledger type elsewhere, so execution should simplify by rewriting the current runner internals rather than layering another helper tree on top of `MemberWorker` and `WriteGate`.

### Execution plan
1. Collapse the runner onto the real accepted-write boundary. Delete the per-member background writer model from [tests/ha/support/invariants/write_convergence.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/invariants/write_convergence.rs) and replace it with logic that observes cluster authority, writes only through the authoritative primary, and records only those accepted commits as convergence obligations.
2. Keep the caller split that already exists instead of inventing new public state:
   - `ensure_healthy(selected_members)` remains the strong post-quiesce convergence assertion for explicit healthy outcomes;
   - `ensure_running()` remains teardown/liveness-only and must stop the background task without reconnecting to intentionally unhealthy or unreachable members.
3. Reuse existing flat types rather than adding wrappers. Thread the already-derived `Vec<ClusterMember>` healthy-member slice from the successful health poll directly into the strong check, keep `PostgresRoutingTarget` as the routing type, and store any accepted-write bookkeeping in private runner state only if execution proves it is necessary.
4. Simplify the internal state around one authoritative-write loop or equivalent minimal task. The runner should only need enough private state to:
   - pause or stop new accepted-write attempts;
   - wait for any in-flight accepted write to settle before sampling counts; and
   - compare the selected members' observed row counts after quiesce.
   If extra worker structs or helper layers survive without proving value, remove them.
5. Add regression coverage at the boundary that failed:
   - a focused unit/integration test proving a local write on a non-authoritative or doomed member does not become the expected global count;
   - a cleanup/liveness regression proving intentionally down members do not cause teardown to run the strong convergence assertion; and
   - focused HA coverage for `primary_isolated_from_majority`, `rewind_failure_old_primary_rejoins`, and `lone_survivor_with_only_local_dcs` if the narrow tests are not enough.
6. After the boundary rewrite compiles, run the required gates in repo order: `make check`, `make lint`, `make test`, and `make test-long`. If execution shows the accepted-write model is still wrong, switch this task back to `TO BE VERIFIED`, explain the remaining design gap precisely here, and stop immediately.

### Constraints for execution
- Do not add a new public wrapper type just to name the selected members or accepted-write set. Reuse existing `Vec<ClusterMember>`, `PostgresRoutingTarget`, and runner methods.
- Prefer code deletion over helper growth. The likely successful direction is fewer threads, fewer state structs, and fewer faux-ledger concepts.
- Do not let cleanup own strong convergence semantics again.
- Do not run `cargo test`; use the required `make` targets, and only use focused `cargo nextest` if needed during local iteration before the final gates.

NOW EXECUTE
