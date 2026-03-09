## Bug: Preserved Replica Rejoin Stalls After Runtime Stop Failover <status>completed</status> <passes>true</passes>

<description>
The degraded replica failover scenario exposed a separate recovery bug after the harness stop path was corrected to explicitly stop postgres.

When a replica runtime is stopped, postgres is stopped, the primary fails over to the healthy sibling, and the degraded replica later restarts with its existing data directory preserved, the restarted replica becomes queryable again but can stall without replicating newly inserted rows from the promoted primary. The failing evidence is the long HA scenario `e2e_multi_node_degraded_replica_failover_promotes_only_healthy_target`, which times out waiting for the post-failover proof row on the restarted degraded node unless the test wipes the replica data directory and forces a fresh clone before restart.

Explore and research the restart/rejoin path first, then fix the product behavior rather than papering over it in the test harness. In particular, inspect startup-mode selection, managed recovery config regeneration for preserved replica data dirs, and HA/process interactions after a runtime-stop plus later restart against a new primary.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan

### Current source findings
- The failing degraded-failover scenario in `tests/ha/support/multi_node.rs` still wipes the degraded replica data directory before restart. That is a harness workaround, not product behavior, and it must be removed once the real rejoin path is fixed.
- Runtime startup already has a preserved-PGDATA branch:
  - `src/runtime/node.rs::select_startup_mode(...)` chooses `ResumeExisting` when `PG_VERSION` exists.
  - `src/runtime/node.rs::select_resume_start_intent(...)` inspects managed recovery residue and, when DCS exposes a foreign healthy primary, rebuilds a replica start intent from that leader instead of trusting stale local `postgresql.auto.conf`.
  - `src/runtime/node.rs::run_start_job(...)` materializes managed postgres config and starts postgres with `-c config_file=pgtm.postgresql.conf`, so a cold postgres restart should get an authoritative managed config rewrite.
- Runtime startup already has unit coverage for the preserved-PGDATA authority rule:
  - `src/runtime/node.rs::select_resume_start_intent_prefers_dcs_leader_over_local_auto_conf(...)` proves `ResumeExisting` prefers the DCS leader over stale local `postgresql.auto.conf`.
  - That means startup-intent selection is not the first thing to change; the bigger gap is the live follow path after postgres is already running again.
- Managed config rendering in `src/postgres_managed.rs` and `src/postgres_managed_conf.rs` already quarantines `postgresql.auto.conf`, rewrites `primary_conninfo`, and refreshes managed recovery signal files. That means the bug is likely not just “we forgot to render conninfo at all”.
- PostgreSQL’s documented standby behavior matters here: changing `primary_conninfo` causes the WAL receiver to restart with the new setting after a configuration reload, so the execution plan should prefer an explicit reload-backed retargeting path instead of assuming a full postgres restart is required.
- The HA steady-state follow path is suspicious:
  - `src/ha/decide.rs` emits `HaDecision::FollowLeader { leader_member_id }` whenever a replica sees an active leader.
  - `src/ha/lower.rs` lowers that into `ReplicationEffect::FollowLeader`.
  - `src/ha/process_dispatch.rs` currently treats `HaAction::FollowLeader { .. }` as `Skipped`, so once postgres is already running there is no process-level action that rewrites managed recovery config, reloads conninfo, or restarts postgres toward the authoritative leader.
- The process layer has no reload primitive today. `src/process/jobs.rs` only exposes bootstrap/basebackup/rewind/promote/demote/start/fencing jobs, so execution needs to add an explicit postgres config reload job instead of trying to fake the behavior indirectly.
- There is a closely related open bug at `.ralph/tasks/bugs/runtime-restart-replica-can-stall-before-replaying-post-restart-writes.md`. The likely shared defect is “leader/follow-target changes are observed in HA state, but the product does not actively reconfigure an already-initialized replica toward the new leader”. This task should fix the preserved-rejoin case without regressing that broader class of restart/follow behavior.

### Implementation strategy
1. Lock the product regression in tests before changing behavior.
   - First add a focused real-binary regression in `src/postgres_managed.rs` for a running standby whose managed config is retargeted from old primary A to new primary B with preserved PGDATA.
   - That proof must materialize new managed config, issue a postgres config reload, and then assert both `SHOW primary_conninfo` and replay progress move to the new leader. This is the source-backed guardrail for the rest of the implementation.
   - Then update `tests/ha/support/multi_node.rs::e2e_multi_node_degraded_replica_failover_promotes_only_healthy_target(...)` so the degraded replica restarts with its preserved data directory instead of `wipe_node_data_dir(...)`.
   - Keep the SQL proof strict: after the healthy sibling is promoted and the degraded node restarts, the preserved replica must replay the post-failover proof row from the promoted primary.

2. Introduce an explicit postgres reload primitive in the process layer.
   - Extend `src/process/jobs.rs`, `src/process/state.rs`, and `src/process/worker.rs` with a dedicated reload job for postgres configuration, using the existing binary/process framework rather than shelling out ad hoc.
   - Add command-building and worker tests proving the reload job targets the right data dir and uses the supported postgres control path.
   - Keep the job narrow: this is for “managed config changed on an already running postgres instance”, not for a full restart.

3. Make `FollowLeader` a real authoritative retarget action.
   - Add a shared helper in `src/ha/process_dispatch.rs` for “derive replica start intent from current DCS leader and local data dir”, so both `StartPostgres` and `FollowLeader` use the same leader-derived managed-config logic.
   - Change `dispatch_process_action(...)` so `HaAction::FollowLeader { leader_member_id }` materializes managed config for that leader and dispatches the new reload job instead of returning `Skipped`.
   - Add focused dispatch tests asserting that `FollowLeader` rewrites managed config, enqueues the reload job, and preserves the existing hard-error behavior when preserved replica residue exists but no authoritative leader can be resolved.

4. Revisit startup intent semantics only if the reload-backed proof still fails.
   - Do not change `select_resume_start_intent(...)` first; existing unit tests already show it prefers the DCS leader over stale local config.
   - If the new real retarget/reload proof or the preserved-PGDATA HA scenario still fail after `FollowLeader` becomes real, then inspect whether preserved rejoin truly requires `ManagedPostgresStartIntent::recovery(...)` rather than `ManagedPostgresStartIntent::replica(...)`.
   - Keep the existing safety rule: preserved managed replica residue without an authoritative DCS leader remains a hard error, not a best-effort local resume.

5. Update docs as part of execution with the `k2-docs-loop` skill.
   - Refresh the HA and recovery docs that describe replica rejoin behavior, at minimum:
     - `docs/src/reference/ha-decisions.md`
     - `docs/src/explanation/ha-decision-engine.md`
     - `docs/src/how-to/handle-primary-failure.md`
   - Remove any stale wording that implies a restarted replica automatically catches up just because it becomes queryable again. The docs should reflect that follower retargeting is driven by authoritative managed-config rewrite plus postgres reload/restart behavior, whichever the final implementation proves correct.

6. Execute and verify in this order.
   - Implement the real standby retarget-and-reload proof first.
   - Add the new reload job and process-worker tests.
   - Wire the shared leader-derived helper and make `FollowLeader` dispatch reload-backed retargeting.
   - Remove the degraded-failover data-dir wipe workaround only after the product path passes without it.
   - Revisit `Replica` vs `Recovery` preserved-startup intent only if the previous steps still leave the scenario failing.
   - Run `make check`, `make test`, `make test-long`, and `make lint`.
   - Only after all gates pass, tick the acceptance criteria, set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all changes including `.ralph`, push, and stop.

### Review focus for the next pass
- Treat the live standby retarget proof as the deciding signal: if reload-backed retargeting fails in the real test, revise the execution plan before touching more HA logic.
- Verify whether the true root cause is the `FollowLeader` no-op alone, or whether preserved replica cold start also requires a `Recovery` start intent change after the reload path exists.
- Keep the implementation aligned with existing process primitives unless a new primitive is required and explicitly tested, which in this case it is.

NOW EXECUTE
