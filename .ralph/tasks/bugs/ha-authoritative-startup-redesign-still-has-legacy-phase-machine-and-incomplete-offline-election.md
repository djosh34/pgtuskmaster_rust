## Bug: HA authoritative startup redesign still has legacy phase machine and incomplete offline election <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
The task `.ralph/tasks/story-managed-start-intent-architecture/task-redesign-ha-startup-bootstrap-and-rejoin-around-authoritative-dcs-reconciliation.md`
is marked completed, but source audit shows major required redesign pieces are still not implemented.

Explore the codebase first, then finish the redesign rather than layering more patches on the old
shape.

Concrete gaps already confirmed:
- `src/ha/state.rs` still centers the HA worker around the legacy `HaPhase` machine
  (`Init`, `WaitingPostgresReachable`, `WaitingDcsTrusted`, `Replica`, `CandidateLeader`,
  `Primary`, `Rewinding`, `Bootstrapping`, `Fencing`, `FailSafe`) instead of the task's required
  compressed `ClusterMode` plus `DesiredNodeState` model.
- `src/ha/decision.rs`, `src/ha/decide.rs`, `src/ha/lower.rs`, `src/ha/apply.rs`,
  `src/ha/worker.rs`, and `src/ha/process_dispatch.rs` still preserve the old overlap between
  `WaitForPostgres`, `FollowLeader`, and start/recovery sequencing that the redesign explicitly
  required removing.
- The redesign task also required explicit behavior for `cluster_initialized = true` after all
  nodes were offline long enough for all leases to expire, plus deterministic offline ranking from
  published physical descriptors. Audit whether the current code truly implements that end to end;
  the current retained phase machine strongly suggests it does not.

Fix the architecture honestly:
- replace the legacy overlap instead of preserving it behind renamed concepts,
- make one authoritative reconcile path for follower convergence,
- make the task file/checklists/docs truthful once the implementation really matches them.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
