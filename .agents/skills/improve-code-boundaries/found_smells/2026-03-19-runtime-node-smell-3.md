path: /home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs 82-145

- I found smell 3
since it looks like
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
    advertised: crate::dcs::startup::DcsAdvertisedEndpoints::from_config(&cfg)?,
    pg_subscriber: pginfo.state.clone(),
    log: log.clone(),
})?;

let process = crate::process::startup::bootstrap(crate::process::startup::ProcessRuntimeRequest {
    identity: identity.clone(),
    runtime_config: cfg_subscriber.clone(),
    dcs_subscriber: dcs.state.clone(),
    plan: process_plan,
    config: cfg.process.clone(),
    capture_subprocess_output: cfg.logging.capture_subprocess_output,
    log: log.clone(),
});
```
