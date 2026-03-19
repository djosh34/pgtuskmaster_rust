path: /home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/reconcile.rs 58-203

I found smell 7:


I think this is smell 7 because one observation-timing mini-model is gating many reconcile branches across leader, follower, and failsafe paths. The repeated `waiting_for_fresh_pg_after_*` checks increase branching everywhere, which is a strong sign of overengineering until proven necessary by tests.


code:
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
