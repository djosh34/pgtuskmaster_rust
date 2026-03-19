path: /home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/types.rs 304-346

I found smell 1:


I think this is smell 1 because `PlannedActions` and `ReconcilePlan` restate the same four optional fields with different names and then copy between each other through `from_plan`. `AuthorityProjectionState` also renames `PublicationState` without changing meaning, so the file is carrying type aliases and mirrors instead of one shared shape.


code:
```rust
pub struct PlannedActions {
    pub publication: Option<PublicationAction>,
    pub coordination: Option<CoordinationAction>,
    pub local: Option<LocalAction>,
    pub process: Option<ProcessIntent>,
}

pub(crate) struct ReconcilePlan {
    pub(crate) publication: Option<PublicationAction>,
    pub(crate) coordination: Option<CoordinationAction>,
    pub(crate) local: Option<LocalAction>,
    pub(crate) process: Option<ProcessIntent>,
}

pub type AuthorityProjectionState = PublicationState;

impl PlannedActions {
    pub(crate) fn from_plan(value: &ReconcilePlan) -> Self {
        Self {
            publication: value.publication.clone(),
            coordination: value.coordination.clone(),
            local: value.local.clone(),
            process: value.process.clone(),
        }
    }
}
```
