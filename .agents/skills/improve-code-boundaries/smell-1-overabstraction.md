# Smell 1: Useless Overabstraction And Overnesting

This smell is about local program state being projected into other local program state for no real gain.

The bad version is not "there are several types." The bad version is:

- the types mostly restate each other
- the wrapper chain adds nesting but not invariants
- helper functions exist mostly to move the same facts around
- downstream users could have matched on the original type directly

Do not search for `build_` by name and stop there. Study what the functions actually do.

## Detection checklist

Inspect for these concrete signals:

- input type and output type have the same or nearly the same enum cases
- a match sets some field to the same value across multiple arms
- a type collapses several original cases into one coarse case and then another helper re-adds the missing detail
- bools inside enums
- a two-value enum that is really `Option<T>`
- nested `Option` shapes that should be one flatter enum
- more than three forward nestings, that could just work on one original
- request/bootstrap/context wrappers that just repackage fields
- config-like values pushed through signatures instead of owned in one context

## Example A: HA bootstrap wrappers that add no invariant

From `src/ha/startup.rs`:

```rust
pub(crate) struct HaRuntimeRequest {
    pub(crate) identity: NodeIdentity,
    pub(crate) poll_interval: Duration,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsView>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) process_control: ProcessControlHandle,
    pub(crate) dcs_handle: DcsHandle,
}

pub(crate) fn bootstrap(request: HaRuntimeRequest) -> HaRuntimeBundle {
    let initial_state = HaState::initial(crate::state::WorkerStatus::Starting);
    let (publisher, state) = new_state_channel(initial_state.clone());
    let ctx = HaWorkerCtx::new(HaWorkerBootstrap {
        cadence: HaWorkerCadence {
            poll_interval: request.poll_interval,
            now: Box::new(crate::process::worker::system_now_unix_millis),
        },
        state_channel: HaStateChannel {
            current: initial_state,
            publisher,
        },
        observed: HaObservedState {
            config: request.config_subscriber,
            pg: request.pg_subscriber,
            dcs: request.dcs_subscriber,
            process: request.process_subscriber,
        },
        control: HaControlPlane {
            process_intent_inbox: request.process_control.intents,
            dcs_handle: request.dcs_handle,
        },
        identity: request.identity,
    });

    HaRuntimeBundle {
        state,
        worker: HaWorker(ctx),
    }
}
```

From `src/ha/state.rs`:

```rust
pub(crate) struct HaRuntimeCtx {
    pub(crate) cadence: HaWorkerCadence,
    pub(crate) state_channel: HaStateChannel,
    pub(crate) observed: HaObservedState,
    pub(crate) control: HaControlPlane,
    pub(crate) identity: NodeIdentity,
}

pub(crate) type HaWorkerCtx = HaRuntimeCtx;
pub(crate) type HaWorkerBootstrap = HaRuntimeCtx;

impl HaRuntimeCtx {
    pub(crate) fn new(bootstrap: HaWorkerBootstrap) -> Self {
        let HaRuntimeCtx {
            cadence,
            state_channel,
            observed,
            control,
            identity,
        } = bootstrap;
        Self {
            cadence,
            state_channel,
            observed,
            control,
            identity,
        }
    }
}
```

Why this is a smell:

- `HaRuntimeRequest` is only a staging wrapper.
- `HaWorkerBootstrap` is just a type alias to `HaRuntimeCtx`.
- `HaWorkerCtx` is also just a type alias to `HaRuntimeCtx`.
- `HaRuntimeCtx::new` destructures a `HaRuntimeCtx` and reconstructs the same `HaRuntimeCtx`.
- No extra invariant is introduced anywhere in this chain.

This is exactly "type in type in type" without new meaning.

### How to untangle this example

1. Comment out `HaWorkerBootstrap` and `HaRuntimeCtx::new`.
2. Run `make check`.
3. Fix call sites by constructing `HaRuntimeCtx` directly.
4. Reassess whether `HaRuntimeRequest` still buys anything. If it only mirrors `HaRuntimeCtx` inputs, collapse it too.
5. Keep only the one shape that actually represents runtime ownership.
6. Continue on with the structs and functions depend on it, can we merge it further? can we merge it in? please keep iterating until you can absolutely not (even with proper good large refactors) reduce the number of structs further

The winning end state is usually one of:

- runtime calls `HaRuntimeCtx { ... }` directly
- `bootstrap` itself owns the construction details and the caller passes one smaller validated input

Do not keep all names just because they sound different.

## Example B: `PgInfoState` converted into HA-local `PostgresState`

From `src/ha/worker.rs`:

```rust
fn build_postgres_state(pg: &PgInfoState) -> PostgresState {
    match pg {
        PgInfoState::Unknown { common } if common.sql != SqlStatus::Healthy => {
            PostgresState::Offline
        }
        PgInfoState::Unknown { .. } => PostgresState::Offline,
        PgInfoState::Primary {
            common, wal_lsn, ..
        } if common.sql == SqlStatus::Healthy => PostgresState::Primary {
            committed_lsn: wal_lsn.0,
        },
        PgInfoState::Primary { .. } => PostgresState::Offline,
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } if common.sql == SqlStatus::Healthy => PostgresState::Replica {
            upstream: upstream.as_ref().map(|value| value.member_id.clone()),
            replication: build_replication_state(common.timeline, *replay_lsn, *follow_lsn),
        },
        PgInfoState::Replica { .. } => PostgresState::Offline,
    }
}

fn build_local_postgres_state(pg: &PgInfoState, dcs: &DcsView) -> PostgresState {
    match build_postgres_state(pg) {
        PostgresState::Replica {
            upstream,
            replication,
        } => PostgresState::Replica {
            upstream: upstream.or_else(|| resolve_replica_upstream(pg, dcs)),
            replication,
        },
        state => state,
    }
}
```

This is a strong smell for several reasons:

- it converts one local enum into another local enum
- the broad cases stay the same: unknown or primary or replica becomes offline or primary or replica
- `PgInfoState::Unknown` maps to `PostgresState::Offline` in both branches, so the extra condition is already suspect
- one helper strips information out
- another helper adds one missing bit back with DCS fallback

That is the classic shape of a useless intermediate boundary.

### Questions to ask after commenting this out

Temporarily comment out `PostgresState`, `build_postgres_state`, or `build_local_postgres_state`, then run `make check`.

At each failing site, ask:

- Did this caller only need to know primary vs replica vs offline?
- Did it really need a derived WAL position that `PgInfoState` already exposes?
- Did it only need the replica upstream, which can be computed once at the actual DCS join point?
- Is the HA-local enum only compensating for the fact that earlier code threw information away?

### How to untangle this example

1. Start by removing `build_local_postgres_state`.
2. Re-run `make check`.
3. Replace downstream matches with direct matches on `PgInfoState` where possible.
4. If one fact is genuinely missing, add that fact once to the flatter shared type instead of keeping two parallel trees.
5. If HA truly needs a distinct internal enum, create exactly one flatter enum that replaces both the projection helper and the repair helper.
6. Delete every transition helper that now only forwards to the original type.

The important rule is:

- do not keep both `PgInfoState` and `PostgresState` if one can do the job
- if neither is quite right, replace both with one shared flatter type

## Example C: config-like values sprayed through helper signatures

From `src/ha/worker.rs` and `src/ha/process_dispatch.rs`:

```rust
postgres_roles::reconcile_managed_roles(
    &runtime_config,
    runtime_config.postgres_socket_dir().as_path(),
    runtime_config.postgres.network.listen_port,
)
```

```rust
pub(crate) fn dispatch_process_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &ProcessIntent,
    _runtime_config: &RuntimeConfig,
) -> Result<ProcessDispatchOutcome, ProcessDispatchError> {
```

These are signature smells:

- the same config root is already present
- derived config values are still threaded through the call
- `_runtime_config` is passed at all even though this helper does not use it

When you see paths, dirs, ports, timings, durations, TLS settings, or hostnames sprayed through helper functions, stop and ask whether they belong in one module context or one validated shared config instead.

## Decision rule for smell 1

Keep the original type unless the new type provably adds one of:

- a stricter invariant
- a truly smaller set of valid states
- module-owned derived state that cannot be represented once at the original boundary

If it adds none of those, remove it.

## Exceptions

Do not apply this smell mechanically to real-world DTOs and direct external payloads. Raw `config.toml` shapes and direct `PgPollData`-style payloads belong to smell 2. The point here is to remove useless internal projections, not to deny that external inputs exist.
