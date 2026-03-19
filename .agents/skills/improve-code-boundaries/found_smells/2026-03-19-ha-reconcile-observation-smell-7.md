path: /home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/reconcile.rs 58-203

- I found smell 7
since it looks like
```rust
fn reconcile_role(world: &WorldView, target: &TargetRole) -> ReconcilePlan {
    match target {
        TargetRole::Leader(_) => match (&world.local.data_dir, &world.local.postgres) {
            (_, _) if world.local.observation.waiting_for_fresh_pg_after_start() => {
                ReconcilePlan::default()
            }
            (_, _) if world.local.observation.waiting_for_fresh_pg_after_promote() => {
                ReconcilePlan::default()
            }
            // ...
        },
        // ...
    }
}

fn reconcile_follow_role(world: &WorldView, goal: &FollowGoal) -> ReconcilePlan {
    match goal.recovery {
        RecoveryPlan::Basebackup => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                ReconcilePlan::default()
            }
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Fast))
            }
            // ...
        },
        RecoveryPlan::Rewind => match &world.local.postgres {
            PostgresState::Primary { .. } | PostgresState::Replica { .. }
                if world.local.observation.waiting_for_fresh_pg_after_demote() =>
            {
                ReconcilePlan::default()
            }
            PostgresState::Primary { .. } | PostgresState::Replica { .. } => {
                ReconcilePlan::process(ProcessIntent::Demote(ShutdownMode::Fast))
            }
            // ...
        },
        RecoveryPlan::StartStreaming => {
            if world.local.observation.waiting_for_fresh_pg_after_start() {
                return ReconcilePlan::default();
            }
            if world.local.observation.waiting_for_fresh_pg_after_demote() {
                return ReconcilePlan::default();
            }
        }
    }
}
```
