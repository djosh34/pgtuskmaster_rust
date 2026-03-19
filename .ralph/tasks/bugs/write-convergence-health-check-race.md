## Bug: Write convergence health check can observe one extra committed write after stopping workers <status>completed</status> <passes>true</passes>

<description>
`make test` exposed a failure in `tests/ha/support/invariants/write_convergence.rs::one_primary_and_two_replicas_are_determined_healthy` where `ensure_healthy()` expected all members to converge to count `3` but observed `4` on every member instead.

This was detected while running the full suite during the HA boundary collapse task. Explore the invariant runner and the surrounding HA/test timing first, then fix the race or behavioral leak so the health check samples a stable committed count.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Boundary diagnosis
- The write-convergence runner already models "a write is currently inside the critical section", but it does so with `pause_write: Arc<RwLock<()>>` owned by worker startup wiring instead of runner-owned shutdown state. `ensure_healthy()` therefore cannot close that boundary before sampling `written_count`.
- That ownership split makes the health check observe an unstable state: a worker can finish one last committed increment after `ensure_healthy()` snapshots `written_count`, producing the observed `expected 3, got 4 everywhere` failure.
- The previous execution attempt also proved the async `tokio::sync::RwLock<()>` is the wrong shape for this boundary. During ultra-long HA cleanup, `ensure_healthy()` runs from synchronous current-thread cleanup code and falls back to a helper thread; waiting on the async lock from that helper runtime starves the original runtime that must poll the worker futures holding the read side, so cleanup deadlocks.
- Repo search did not reveal an existing runtime-independent shutdown gate we can reuse here. The closest shapes are async-only (`tokio::sync::RwLock`) or one-way stopped flags (`primary_count`), so the smallest correct design is a runner-owned synchronous quiesce gate rather than another async helper.
- The real boundary is "no worker may start a new write attempt, and all in-flight attempts must drain before the expected count is sampled". That boundary needs to be synchronous so both normal health checks and cleanup callers can enforce it without depending on a second runtime.
- Re-execution showed the first synchronous-gate design is still wrong. Calling `quiesce()` directly at the start of `ensure_healthy()` blocks the same current-thread runtime that is polling the member worker futures. In `make test-long`, six HA scenarios timed out at 600s and a debugger backtrace on `ha_rejoin_and_restart_recovery::full_database_outage_stays_unhealthy_until_nodes_return` showed the cleanup thread stuck in `WriteGate::quiesce()` while waiting on an in-flight write permit that could only be dropped by a worker task on that starved runtime.
- The deeper boundary problem is therefore bootstrap ownership, not just lock shape: `WriteConvergenceInvariantRunner` exposes synchronous `ensure_healthy()` and synchronous `Drop`, but `MemberWorker` execution currently lives on whichever Tokio runtime happened to call `start()`. A synchronous shutdown API cannot safely quiesce work that is still borrowing the caller's current-thread runtime.
- Repo search also did not reveal an existing detached worker-thread shape to reuse here, so the lowest-complexity fix is to alter `MemberWorker` directly instead of layering more helper wrappers around the borrowed-runtime model.
- Any revised design therefore needs a shutdown boundary that both:
  - prevents new writes from starting before the expected-count snapshot; and
  - allows already-started writes to drain without blocking the runtime that must drive them to completion.
- Re-execution with detached worker threads removed the current-thread deadlock and passed `cargo nextest run write_convergence`, `make check`, `make lint`, and `make test`, but `make test-long` exposed a deeper correctness gap. In `ha_primary_faults_fail_over_then_recover::primary_isolated_from_majority` and `ha_rejoin_and_restart_recovery::rewind_failure_old_primary_rejoins`, the new quiesce path counted one write that committed on the isolated old primary (`expected 19/21`) even though the surviving majority only converged to `18/20`.
- `ha_quorum_loss_and_dcs_loss::lone_survivor_with_only_local_dcs` also failed because the invariant kept demanding convergence while two members were intentionally down and no longer exposed Postgres ports. That means the current "wait for all in-flight writes, snapshot committed_count, then read every member" boundary is still too naive for failover and intentionally unhealthy phases.
- The missing concept is not thread ownership anymore; it is durability semantics. A locally committed write on a doomed or unreachable primary is not automatically a cluster-wide convergence baseline, and cleanup/background checks need a rule for when unreachable members should stop participating in the expected-count assertion.
- The current `WriteGateState::committed_count` is the wrong boundary for that rule. It lives inside the shutdown gate, but it is really acting like a fake durability ledger and increments on any worker-local commit, including isolated primaries whose writes are later discarded during failover/rewind.
- Repo inspection shows the selected-member boundary is already on the right type now: `HaWorld` already owns `online_member_ids()` / `online_expected_count()` derived from `stopped_members` plus `observer_unreachable_members`. The stale smell is that `HarnessShared::cleanup(&[ClusterMember])` still accepts a scenario-member slice only so teardown can call the strong write-convergence assertion.
- `PrimaryCountInvariantRunner` already shows the useful split here: a runner can expose a strong semantic health check and, when needed, a lighter "still running" check. Reusing that shape is lower-risk than inventing another task-specific state enum.
- Re-execution with world-derived online-member selection fixed the detached-runtime race, but `make test-long` still fails in three scenarios because cleanup is still calling the strong convergence assertion unconditionally. In `ha_quorum_loss_and_dcs_loss::lone_survivor_with_only_local_dcs`, the lone survivor scenario correctly ends unhealthy yet cleanup still tries to reconnect to `node-b` and fails with `error communicating with the server`.
- The same wrong caller boundary appears after successful failover/rejoin scenarios. `ha_primary_faults_fail_over_then_recover::primary_isolated_from_majority` and `ha_rejoin_and_restart_recovery::rewind_failure_old_primary_rejoins` now observe the surviving majority converged at `18`/`15`, but cleanup still includes the rejoining old primary and times out on a fresh connection to `node-b`. That proves the remaining bug is not the selected-member set alone; it is that cleanup is a liveness caller, not always a strong post-quiesce convergence caller.
- Search confirms no scenario step currently invokes the strong write-convergence assertion. That means healthy scenarios never prove accepted-write convergence at the moment they declare success, while teardown wrongly owns that responsibility for every scenario, including ones that intentionally end unhealthy.
- Re-execution after moving the strong assertion into `cluster becomes healthy` proved `HaWorld::online_member_ids()` is still the wrong selection boundary for strong convergence. `make test-long` now fails in `ha_rejoin_and_restart_recovery::blocked_basebackup_recovery_recovers_after_unblock` and `ha_primary_faults_fail_over_then_recover::primary_isolated_from_majority` because the healthy outcome step accepts the scenario, but the follow-up strong write-convergence check still includes a member whose Postgres probe is not yet reconnectable (`node-a` / `node-b` connection errors while the surviving members already agree on `11` / `14`).
- The remaining wrong-place boundary is therefore the source of the strong selected-member set. `HaWorld::online_member_ids()` models scenario intent ("not stopped" and "not observer-unreachable"), but accepted-write convergence at healthy time needs the smaller set of members that actually satisfy the successful healthy observation right now.
- The healthy step already has the right bootstrap context to derive that set without adding another public wrapper layer: `wait_for_authoritative_single_primary()` owns the authoritative observation, the verified writable primary, and the observer needed for fresh Postgres probes. The missing behavior is to return or thread through the observed-healthy members from that same successful poll instead of discarding the observation and recomputing members from world state.

### Execution plan
1. Keep cleanup split from strong convergence: `HaWorld::cleanup()` / `HarnessShared::cleanup()` should stay liveness-only and take no member slice. Teardown must not own scenario-outcome membership or strong convergence semantics anymore.
2. Keep `WriteGateState::committed_count` removed and keep the strong write-convergence assertion based on actual post-quiesce observations: after closing the gate and draining in-flight writes, the selected members must all report the same fixture-row count.
3. Keep the `PrimaryCountInvariantRunner`-style runner split: strong post-quiesce convergence for explicit health callers, lighter `ensure_running()` behavior for cleanup so detached workers stop without demanding fresh member reads in intentionally unhealthy or still-rejoining scenarios.
4. Move strong selected-member derivation off `HaWorld` and onto the successful healthy poll itself. Refactor `wait_for_authoritative_single_primary()` or an adjacent helper to return both:
   - the authoritative writable primary; and
   - the observed-healthy members from that same successful poll, derived from the authoritative view (`primary` plus ready replicas) and filtered by fresh observer Postgres probes.
5. Thread only that observed-healthy slice through the explicit strong health-check call path. `cluster becomes healthy` should use the returned slice directly instead of recomputing `world.online_member_ids()`. Other callers that only need the primary should ignore the extra value rather than introducing a new public wrapper struct.
6. Add focused regression coverage for both caller boundaries and the latest ultra-long failures:
   - cleanup/liveness remains green for `ha_quorum_loss_and_dcs_loss::lone_survivor_with_only_local_dcs` and `ha_rejoin_and_restart_recovery::rewind_failure_old_primary_rejoins`;
   - healthy-time strong convergence no longer overselects members in `ha_primary_faults_fail_over_then_recover::primary_isolated_from_majority`; and
   - healthy-time strong convergence no longer overselects members in `ha_rejoin_and_restart_recovery::blocked_basebackup_recovery_recovers_after_unblock`.
7. Add at least one narrow regression around the new healthy boundary itself: the successful health poll provides the strong member slice, and members that are still reported in scenario/world intent but fail a fresh Postgres probe do not participate yet.
8. Once the boundary is redesigned, rerun the narrowest relevant `cargo nextest` coverage, then `make check`, `make lint`, `make test`, and `make test-long`.

### Constraints for execution
- Prefer one small gate/state type over multiple wrapper structs. If execution can express the guard inline next to the runner without adding another public abstraction layer, do that.
- Prefer code reduction over helper growth. If a helper only forwards gate/task state, collapse it instead of adding another layer.
- Do not swallow new errors while refactoring the invariant runner. If a hidden failure mode is uncovered, create a bug immediately.

NOW EXECUTE
