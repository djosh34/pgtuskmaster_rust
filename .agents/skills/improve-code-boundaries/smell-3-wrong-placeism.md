# Smell 3: Wrong Place-ism

This smell is about knowledge living in the wrong module.

The classic bad shape is:

- A talks to B
- B talks back to A
- both pass similar request or state types around
- the top-level runtime becomes the courier for everyone else's internals

This is closely related to bad config boundaries. When validation has not been finished, raw or half-validated config tends to spray through runtime, worker startup, and helper functions, and each layer starts compensating in its own way.

The desired direction is:

- `src/config` produces a validated config type
- runtime wires modules together without knowing their internal field soup
- workers consume one shared observed state and the few handles they truly need
- the shared observed state is the source of truth

## Example A: runtime builds every worker's request bundle by hand

From `src/runtime/node.rs`:

```rust
let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone());
let identity = NodeIdentity {
    cluster_name: ClusterName(cfg.cluster.name.clone()),
    scope: ScopeName(cfg.cluster.scope.clone()),
    member_id: MemberId(cfg.cluster.member_id.clone()),
};
let worker_poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);

let pginfo = crate::pginfo::startup::bootstrap(crate::pginfo::startup::PgInfoRuntimeRequest {
    identity: identity.clone(),
    probe: crate::pginfo::state::PgProbeTarget::local_from_config(&cfg, &process_plan),
    poll_interval: worker_poll_interval,
    log: log.clone(),
});

let dcs = crate::dcs::startup::bootstrap(crate::dcs::startup::DcsRuntimeRequest {
    identity: identity.clone(),
    endpoints: cfg.dcs.endpoints.clone(),
    client: cfg.dcs.client.clone(),
    poll_interval: worker_poll_interval,
    member_ttl_ms: cfg.ha.lease_ttl_ms,
    advertised: crate::dcs::startup::DcsAdvertisedEndpoints::from_config(&cfg).map_err(
        |err| RuntimeError::Worker(format!("dcs advertisement build failed: {err}")),
    )?,
    pg_subscriber: pginfo.state.clone(),
    log: log.clone(),
})?;

let process =
    crate::process::startup::bootstrap(crate::process::startup::ProcessRuntimeRequest {
        identity: identity.clone(),
        runtime_config: cfg_subscriber.clone(),
        dcs_subscriber: dcs.state.clone(),
        plan: process_plan,
        config: cfg.process.clone(),
        capture_subprocess_output: cfg.logging.capture_subprocess_output,
        log: log.clone(),
    });

let ha = crate::ha::startup::bootstrap(crate::ha::startup::HaRuntimeRequest {
    identity: identity.clone(),
    poll_interval: worker_poll_interval,
    config_subscriber: cfg_subscriber.clone(),
    pg_subscriber: pginfo.state.clone(),
    dcs_subscriber: dcs.state.clone(),
    process_subscriber: process.state.clone(),
    process_control: process.control.clone(),
    dcs_handle: dcs.handle.clone(),
});

let api = crate::api::startup::bootstrap(crate::api::startup::ApiRuntimeRequest {
    identity,
    runtime_config: cfg_subscriber,
    dcs_handle: dcs.handle.clone(),
    observed_state: crate::api::worker::ApiObservedState::Live {
        pg: pginfo.state.clone(),
        process: process.state.clone(),
        dcs: dcs.state.clone(),
        ha: ha.state.clone(),
    },
    log: log.clone(),
})?;
```

Why this is the smell:

- runtime knows every worker's field list
- runtime knows which subscribers each worker wants
- runtime clones and recombines similar state repeatedly
- worker-private bootstrap concerns are being specified from outside

This is not just wiring. This is runtime owning module internals.

## Example B: HA startup repeats the same observer breakdown

From `src/ha/startup.rs` and `src/ha/state.rs`:

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
```

```rust
pub(crate) struct HaObservedState {
    pub(crate) config: StateSubscriber<RuntimeConfig>,
    pub(crate) pg: StateSubscriber<PgInfoState>,
    pub(crate) dcs: StateSubscriber<DcsView>,
    pub(crate) process: StateSubscriber<ProcessState>,
}

pub(crate) struct HaControlPlane {
    pub(crate) process_intent_inbox: UnboundedSender<ProcessIntentRequest>,
    pub(crate) dcs_handle: DcsHandle,
}
```

This is the same shape again:

- the caller disassembles system state into many subscriber fields
- the callee reassembles them into an observed-state struct
- the boundary is making both sides know too much

## Example C: signature drift shows the runtime is spraying knowledge

From `src/ha/process_dispatch.rs`:

```rust
pub(crate) fn dispatch_process_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &ProcessIntent,
    _runtime_config: &RuntimeConfig,
) -> Result<ProcessDispatchOutcome, ProcessDispatchError> {
```

This is a smaller but useful wrong-place signal:

- config is threaded into a helper
- the helper does not use it
- the signature has become contaminated by higher-level context

That is how bootstrap spaghetti tends to spread.

## How to untangle smell 3

1. Pick one request/bootstrap corridor.
   `src/runtime/node.rs -> src/*/startup.rs -> src/*/state.rs` is a good starting shape.
2. Draw the full A -> B -> A path.
   Include config, subscribers, handles, and request structs.
3. Temporarily comment out the request bundle or startup wrapper and run `make check`.
4. At each failure ask:
   - does the worker really need this field directly?
   - is this field already present in one shared source of truth?
   - is runtime doing derivation that belongs in `src/config` or inside the target module?
5. Move validated config construction back to `src/config`.
6. Introduce or reuse one shared observed-state struct as the source of truth.
7. Pass that shared observer plus the minimal handles a worker truly acts on.
8. Delete request structs that only mirror the caller's knowledge.

## Preferred architecture direction for this repo

For this codebase, the target direction is:

- all workers get one shared observer
- that observer is effectively the combined shared channel view
- DCS, pginfo, and similar producers send updates into the shared state
- every consumer subscribes to the same shared source of truth instead of assembling its own mini observer from several channels
- workers receive handles to other modules only when they must cause actions, not as a way to re-import state knowledge

The key point is not "invent a huge central object." The key point is:

- state is shared once
- subscriptions are shared once
- runtime stops hand-carrying similar request types back and forth

## Decision rule for smell 3

If a startup request struct mostly lists fields that the callee immediately re-bundles into its own context structs, the boundary is in the wrong place.

If module A must understand too many details of module B's internal observer or control plane, the boundary is in the wrong place.

Fix the placement of knowledge first. The code reduction follows from that.

## Relationship to smell 2

Bad config boundaries often cause wrong place-ism.

If `RuntimeConfig` or another half-validated config shape is still leaking through runtime and worker startup, do not treat that as separate noise. It often means:

- finish the config boundary first
- then collapse the bootstrap/request corridor

Follow the full graph until both are flatter.

