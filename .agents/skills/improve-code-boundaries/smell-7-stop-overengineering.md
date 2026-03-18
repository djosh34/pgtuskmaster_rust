# Smell 7: Stop Overengineering

This smell is intentionally broad.

Sometimes the problem is not only "wrong place" or "wrong type." Sometimes the problem is that the solution is simply too elaborate.

Rules:

- simpler solution is better
- simpler state machine is better
- less remembered state is better
- fewer timing concepts are better
- fewer features are better
- the target question is: what is the minimum needed to keep `test-long` happy?

Do not defend extra machinery just because it sounds robust. Prove it is needed.

## Detection checklist

Look for:

- structs accumulating "maybe useful" fields
- timing or observation fields that only gate other derived helpers
- state-machine branches that look precautionary rather than necessary
- feature-like fields added "just in case"
- debug or bookkeeping data that changes flow control
- repeated `*_success_at` timestamps and "waiting_for_*" helpers

## The required workflow

This smell has a stricter proof method than most:

1. pick one suspicious field, helper, branch, or timing concept
2. remove it, or hardcode the simpler behavior
3. run `make check`
4. run the narrowest relevant `cargo nextest ...`
5. if the area is mainly protected by integration behavior, run `make test-long`
6. interpret the result:
   - if tests still pass, you overengineered it, so keep it removed
   - if tests fail and the behavior really matters, simplify less
   - if the removed behavior should matter but no test failed, add the missing test and then simplify again

The bias is always toward deleting first, not justifying first.

## Example A: HA observation timing is a separate mini-state machine

From `src/ha/types.rs`:

```rust
pub struct LocalKnowledge {
    pub data_dir: DataDirState,
    pub postgres: PostgresState,
    pub process: ProcessAssessment,
    pub storage: StorageState,
    pub managed_roles_reconciled: bool,
    pub publication: PublicationState,
    pub observation: ObservationState,
}

pub struct ObservationState {
    pub pg_observed_at: UnixMillis,
    pub last_start_success_at: Option<UnixMillis>,
    pub last_basebackup_success_at: Option<UnixMillis>,
    pub last_promote_success_at: Option<UnixMillis>,
    pub last_demote_success_at: Option<UnixMillis>,
}
```

And from the same file:

```rust
impl ObservationState {
    pub(crate) fn waiting_for_fresh_pg_after_start(&self) -> bool {
        self.last_start_success_at
            .map(|finished_at| finished_at.0 >= self.pg_observed_at.0)
            .unwrap_or(false)
    }

    pub(crate) fn waiting_for_fresh_pg_after_promote(&self) -> bool {
        self.last_promote_success_at
            .map(|finished_at| finished_at.0 >= self.pg_observed_at.0)
            .unwrap_or(false)
    }

    pub(crate) fn waiting_for_fresh_pg_after_demote(&self) -> bool {
        self.last_demote_success_at
            .map(|finished_at| finished_at.0 >= self.pg_observed_at.0)
            .unwrap_or(false)
    }
}
```

This is a classic overengineering candidate:

- many timestamp fields
- several tiny gating helpers
- a separate local timing mini-model

It may be needed. It may also be compensating for a more complicated design than necessary.

The right question is not "can I imagine why this might help?"

The right question is:

- if I delete one or more of these timing gates, do `make check` and the relevant tests still pass?

## Example B: HA worker reconstructs and preserves extra remembered state

From `src/ha/worker.rs`:

```rust
let observation = ObservationState {
    pg_observed_at: pg.last_refresh_at().unwrap_or(now),
    last_start_success_at: last_start_success_at(&process),
    last_basebackup_success_at: last_success_at(&process, ActiveJobKind::BaseBackup),
    last_promote_success_at: last_success_at(&process, ActiveJobKind::Promote),
    last_demote_success_at: last_success_at(&process, ActiveJobKind::Demote),
};

let local = LocalKnowledge {
    // ...
    managed_roles_reconciled: ctx.state_channel.current.managed_roles_reconciled,
    publication: ctx.state_channel.current.publication.clone(),
    observation,
};
```

And later:

```rust
HaState {
    worker: WorkerStatus::Running,
    tick: current.tick.saturating_add(1),
    managed_roles_reconciled: next_managed_roles_reconciled(current, plan),
    publication: apply_publication_goal(&current.publication, &desired.publication),
    role: desired.role.clone(),
    world: world.clone(),
    clear_switchover: desired.clear_switchover,
    planned_actions: super::types::PlannedActions::from_plan(plan),
}
```

This should trigger the smell immediately:

- how much of this remembered state is truly required?
- how much is just a convenience cache?
- how much is only there because the surrounding design became too indirect?

Fields like `managed_roles_reconciled` and `planned_actions` are not automatically bad. They are bad if deleting them does not break real behavior.

## Example C: reconcile logic repeatedly depends on the timing helpers

From `src/ha/reconcile.rs`:

```rust
(_, _) if world.local.observation.waiting_for_fresh_pg_after_start() => {
    ReconcilePlan::default()
}
(_, _) if world.local.observation.waiting_for_fresh_pg_after_promote() => {
    ReconcilePlan::default()
}
```

And later:

```rust
if world.local.observation.waiting_for_fresh_pg_after_demote() {
    return ReconcilePlan::default();
}
```

This is a strong overengineering smell because:

- one extra state concept now influences many branches
- the complexity multiplier is larger than the original helper looked
- deleting one "feature" can simplify a large amount of downstream logic

## How to use this smell constructively

Do not just say "this feels overengineered."

Do this:

1. identify one field or one helper that looks ornamental, defensive, or too smart
2. remove it
3. propagate the simpler logic through all compile failures
4. run tests
5. decide by evidence, not taste

Good deletion targets are often:

- remembered booleans
- timestamp fields
- cached projections
- "planned actions" that only mirror another plan
- feature data that is not itself the source of truth

## Decision rule

If removing a feature or remembered fact does not break tests, it was probably a mistake.

If removing it should have broken behavior but no test failed, add the missing test and then keep simplifying until the minimal truthful design remains.

The skill is not "make it dumb."

The skill is:

- stop paying complexity rent for behavior the system does not actually need

