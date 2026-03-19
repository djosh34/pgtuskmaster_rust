path: /home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/types.rs 304-346

- I found smell 1
since it looks like
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
```

- I found smell 1
since it looks like
```rust
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
