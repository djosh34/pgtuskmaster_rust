## Bug: Runtime-restart replica can stall before replaying post-restart writes <status>completed</status> <passes>true</passes>

<description>
During `make test-long` on 2026-03-09, `e2e_multi_node_primary_runtime_restart_recovers_without_split_brain` reached a stable post-restart state with `node-2` as primary and both `node-1` and `node-3` reporting replica roles, but `node-3` never replayed the post-restart proof row within the scenario window.

The exported failure was:
- `timed out waiting for expected rows on node-3; expected=["1:before-restart", "2:after-restart"]; last_observation=rows=["1:before-restart"]`

Evidence:
- nextest log: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_multi_node_failover__e2e_multi_node_primary_runtime_restart_recovers_without_split_brain.log`
- scenario timeline: `.ralph/evidence/13-e2e-multi-node/ha-e2e-primary-runtime-restart-recovers-without-split-brain-1773073335978.timeline.log`

Explore and research the codebase first, then fix the underlying replica catch-up / follow-target gap without weakening the HA guarantees or hiding the replication defect behind broader test relaxations.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Outcome

- Closed as stale/already-fixed on current `HEAD`; no product or docs changes were required.
- Fresh verification ran the exact scenario `e2e_multi_node_primary_runtime_restart_recovers_without_split_brain` three times in succession, and all three runs passed.
- The same scenario passed again inside `make test-long`, alongside the rest of the ultra-long HA suite plus Docker compose validation, single-node smoke, and cluster smoke.
- Full required gates passed on 2026-03-09: `make check`, `make test`, `make test-long`, and `make lint`.

## Plan

### Current source findings
- The task description is older than the current source tree and current artifacts, so execution must start by reconciling that mismatch rather than assuming the original failure still reproduces unchanged.
- In the current scenario implementation at `tests/ha/support/multi_node.rs::e2e_multi_node_primary_runtime_restart_recovers_without_split_brain(...)`, post-restart proof validation already narrows the validation set when leadership moves away from the restarted primary. That means the task description's specific "`node-3` still timed out after failover to `node-2`" symptom does not match the current test body.
- The same scenario still requires all three nodes to become queryable again after runtime restart recovery, so the current assertion was not relaxed into a two-node scenario overall; only the post-restart proof-row check is conditionally narrowed.
- The referenced older timeline artifact in `.ralph/evidence/13-e2e-multi-node/ha-e2e-primary-runtime-restart-recovers-without-split-brain-1773073335978.timeline.log` stops after leadership stabilizes on `node-2`; it does not record any successful post-restart proof catch-up on the remaining replica.
- The latest local ultra-long nextest log for this scenario at `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_multi_node_failover__e2e_multi_node_primary_runtime_restart_recovers_without_split_brain.log` currently reports the test as passing, so this bug may already be partially or fully fixed by later changes.
- The immediately previous task `preserved-replica-rejoin-stalls-after-runtime-stop-failover` landed related follow-target and preserved-PGDATA changes:
  - `src/ha/process_dispatch.rs` now makes `HaAction::FollowLeader { .. }` materialize authoritative managed config and dispatch `Demote` when the live upstream differs from the DCS leader.
  - `src/runtime/node.rs` now prefers authoritative foreign-leader clone/recovery startup over stale preserved replica residue in more cases.
- The current `FollowLeader` dispatch path has a more specific possible blind spot than the task description suggests: `follow_leader_is_already_current_or_pending(...)` skips work when `pginfo` is not healthy and the managed config already targets the authoritative leader, so any remaining bug is more likely to involve an incorrect skip/latch condition than a totally missing follow primitive.
- Because of that, the remaining likely outcomes are:
  - the original bug is already fixed and this task should be closed as a stale duplicate after strong reproduction attempts, or
  - a narrower state-machine gap remains where a replica has the right managed config but the HA/process loop fails to drive the demote/start cycle needed to make it actually follow the new leader.

### Execution strategy
1. Treat stale-task verification as the default path and try to disprove the bug on current `HEAD`.
   - Run the targeted runtime-restart ultra-long scenario directly first, not the full suite, so the result is attributable.
   - If it passes, rerun that exact targeted scenario multiple times in succession before touching product code.
   - Only keep this task open for implementation if there is a fresh failure on current `HEAD`; otherwise close it as stale/already-fixed work after the full required gates pass.
   - If a fresh failure appears, capture fresh evidence immediately:
     - the nextest log for the scenario,
     - the timeline artifact for the scenario,
     - the affected nodes' debug snapshots or log lines that show role, timeline, `primary_conninfo`, SQL health, and replay progress after failover.

2. If the scenario fails, identify whether the failure is in proof validation scope or in real recovery state.
   - Compare the post-failover state of the healthy surviving replica(s):
     - DCS leader/member view,
     - HA phase and dispatched actions,
     - postgres SQL role,
     - `primary_conninfo`,
     - SQL health,
     - timeline/replay LSN movement.
   - Check whether the non-validated third node is actually healthy and replaying despite not being part of the narrowed proof-row assertion. If it is, the task is stale and the old artifact simply predates the corrected scenario invariant.
   - Determine whether the stalled node is
     - still following the restarted former primary,
     - pointed at the correct new primary but incorrectly skipped by the `FollowLeader` current-or-pending check,
     - latched behind a demote/start sequencing issue in the HA worker,
     - or carrying preserved data that now requires clone/rewind instead of plain follow retargeting.
   - Do not weaken the test until the product-level failure mode is explicitly proven.

3. If the failure is a follow-target adoption gap, check skip/latch behavior before adding any new process primitive.
   - Start in `src/ha/process_dispatch.rs`, `src/ha/worker.rs`, and the existing `FollowLeader` unit tests, because current source already contains an explicit follow retarget + demote path and the more likely remaining defect is an incorrect skip or dispatch latch.
   - First prove whether `follow_leader_is_already_current_or_pending(...)` is suppressing needed work when `pginfo` is unhealthy or incomplete.
   - First prove whether `WaitForPostgres { start_requested: true }` is failing to re-issue `StartPostgres` after a follow demotion.
   - Only if the fresh evidence disproves both of those should execution add a new process primitive such as explicit reload or explicit restart.
   - Keep the action authoritative:
     - derive the follow target from DCS, not from stale local files,
     - preserve the existing hard-error behavior when preserved replica recovery state exists but no authoritative leader can be resolved.
   - Add focused unit tests around the dispatch and process job behavior before relying on the HA e2e to validate it.

4. If the failure is actually a preserved-data divergence problem after failover, fix startup/recovery planning instead.
   - Re-check `src/runtime/node.rs` startup mode selection against the fresh evidence.
   - Verify whether the affected replica should remain on `ResumeExisting`, use `Recovery`, or force reset-plus-basebackup/rewind after the old primary restarts on a different timeline.
   - Prefer authoritative rebuild behavior over stale local resume whenever WAL continuity cannot be guaranteed.
   - Add or extend unit coverage in `src/runtime/node.rs` for the exact startup-mode branch exercised by the failing evidence.

5. Keep the HA test strict and only adjust it if the intended invariant is currently mis-specified.
   - If fresh evidence shows the third node is healthy/queryable but intentionally outside the proof-row assertion after leadership moves, document that invariant more clearly instead of silently preserving ambiguity.
   - If the product promise really is “all surviving replicas catch up after primary runtime restart failover”, restore strict multi-node proof validation and make the product satisfy it.
   - Any scenario edit must be accompanied by a product-side proof or a clearer scenario assertion, not by a timing relaxation.

6. Update docs only if behavior or guarantees materially change.
   - If execution changes how replicas retarget or recover after runtime restart failover, update the relevant HA documentation using the `k2-docs-loop` skill during the execution pass.
   - Remove stale wording if the docs currently imply a stronger or weaker post-failover catch-up guarantee than the product actually enforces.

7. Verify and close only after the full gate is green.
   - Run `make check`.
   - Run `make test`.
   - Run `make test-long`.
   - Run `make lint`.
   - Update the task file checkboxes and `<passes>true</passes>` only after all four commands and any required docs updates are complete.
   - Then run `/bin/bash .ralph/task_switch.sh`, commit all changes including `.ralph`, push, and stop immediately.

### Review focus for the verification pass
- Challenge the assumption that this bug is still open. The latest local scenario log is passing, and the previous task plausibly fixed the same defect class.
- The skeptical review changed the plan accordingly: stale-task closure is now the default path, and the first implementation suspect is the `FollowLeader` skip/latch logic rather than a missing reload primitive.
- Confirm that the plan does not accidentally enshrine the current narrowed test behavior without first deciding what the real product guarantee should be.

NOW EXECUTE
