path: /home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/startup.rs 15-67

- I found smell 1
since it looks like
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
    let ctx = HaRuntimeCtx {
        cadence: HaWorkerCadence {
            poll_interval: request.poll_interval,
            now: Box::new(crate::process::worker::system_now_unix_millis),
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
        state_channel: HaStateChannel { current: initial_state, publisher },
    };
}
```
