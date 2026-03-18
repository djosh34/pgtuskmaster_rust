# Smell 6: Raw DTO Once, Shared Type Once

This smell is the non-config version of smell 2.

Raw data from the outside world must be converted once into one shared enum or struct.

Rules:

- raw DTO stays private or `pub(super)`
- conversion happens once
- the shared type is the one other modules use
- do not keep raw strings and only parse them later in another layer
- do not do partial normalization in several separate steps

## Detection checklist

Look for:

- a raw DTO struct that is `pub(crate)` or wider
- raw fields such as `String` that clearly encode a richer type
- a later module parsing one raw field into a typed field
- several helper functions each "finishing" a different part of the raw-to-shared conversion

## Example A: `PgPollData` is raw and still carries raw strings

From `src/pginfo/query.rs`:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgPollData {
    pub(crate) in_recovery: bool,
    pub(crate) is_ready: bool,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) system_identifier: Option<SystemIdentifier>,
    pub(crate) current_wal_lsn: Option<WalLsn>,
    pub(crate) replay_lsn: Option<WalLsn>,
    pub(crate) receive_lsn: Option<WalLsn>,
    pub(crate) primary_conninfo: Option<String>,
    pub(crate) primary_slot_name: Option<String>,
    pub(crate) slot_names: Vec<String>,
}
```

This is clearly a raw query DTO:

- it mirrors query output
- it still carries `primary_conninfo` as `String`
- it is not the final shared state type

The smell is that this DTO is crate-visible and the conversion is split across module boundaries instead of being more tightly contained.

## Example B: conversion finishes later in another module

From `src/pginfo/state.rs`:

```rust
pub(crate) fn to_member_status(
    worker_status: WorkerStatus,
    sql_status: SqlStatus,
    polled_at: UnixMillis,
    poll: Option<PgPollData>,
) -> Result<PgInfoState, WorkerError> {
    let primary_conninfo = poll
        .as_ref()
        .and_then(|value| value.primary_conninfo.as_deref())
        .map(super::conninfo::parse_pg_conninfo)
        .transpose()
        .map_err(|err| WorkerError::Message(format!("primary_conninfo parse failed: {err}")))?;
```

This is the right conversion idea but the wrong boundary shape:

- one raw field is still being parsed later
- the raw DTO escaped the query module
- normalization is not fully complete at the ingestion point

The better direction is:

- keep the raw DTO private to `query`
- convert immediately into one flatter shared poll snapshot or directly into `PgInfoState`

## Example C: contrast with a good shared response boundary

From `src/api/worker.rs`:

```rust
Ok(Json(NodeState {
    identity: state.identity.clone(),
    pg: pg.latest(),
    process: process.latest(),
    dcs: dcs.latest(),
    ha: ha.latest(),
}))
```

This is closer to the correct direction:

- the API handler emits one shared typed state object
- it does not expose several partial raw handler DTOs to callers

That is the shape to prefer after raw data has been normalized.

## How to untangle smell 6

1. Find the raw data boundary.
2. Mark the raw DTO private or `pub(super)`.
3. Comment out downstream uses of the raw DTO and run `make check`.
4. Let the compile failures show what conversion work is still happening later.
5. Move all raw-to-shared conversion to one place.
6. Expose only the shared flat type.
7. Delete helper functions that only finish the old partial conversion.

## Decision rule for smell 6

If a raw field is still a `String` only because "someone later will parse it," the boundary is wrong.

If module A creates a half-processed DTO and module B turns it into the real shared type, flatten that into one conversion step.

