# Delete process start wrapper types

- Smell: `ManagedStartPlan`, `DesiredManagedPostgresSession`, and `ReplicaFollowPlan` are process-local wrappers around `ManagedPostgresStartIntent`.
- Files: `src/process/planner.rs:27`, `src/process/session.rs:25`, `src/postgres_managed_conf.rs:59`
- Collapse: plan directly in `ManagedPostgresStartIntent` and delete the extra process-only start DTOs.
- Win: removes one whole conversion boundary between planning and managed-config materialization.
