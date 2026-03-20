## Task: Collapse `ha/` To Direct `RuntimeConfigV2` Access And Delete Config Observation <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove fake config observation from `src/ha/` and make HA use one shared `cfg: &RuntimeConfigV2` directly. The higher-order goal is to stop HA from treating static config as if it were changing runtime state and to collapse helper signatures that still spray cfg-derived fields.

**Scope:**
- `src/ha/startup.rs`
- `src/ha/state.rs`
- `src/ha/worker.rs`
- `src/ha/decide.rs`
- `src/ha/reconcile.rs`
- any HA tests/helpers still bound to old config types
- `src/postgres_roles.rs` for HA role reconciliation callsites that still explode cfg into extra args

**Context from research:**
- `HaObservedState` currently contains `StateSubscriber<RuntimeConfig>`
- `HaWorkerCadence` carries poll interval even though it is static config
- `ha/worker.rs` clones config-derived values like data dir and lease ttl
- HA still calls role reconciliation with full config plus exploded socket dir plus port

Representative current code:

```rust
pub(crate) struct HaObservedState {
    pub(crate) config: StateSubscriber<RuntimeConfig>,
    pub(crate) pg: StateSubscriber<PgInfoState>,
    pub(crate) dcs: StateSubscriber<DcsSnapshot>,
    pub(crate) process: StateSubscriber<ProcessState>,
}
```

```rust
postgres_roles::reconcile_managed_roles(
    &runtime_config,
    runtime_config.postgres_socket_dir().as_path(),
    runtime_config.postgres.network.listen_port,
)
```

**Implementation direction:**
- delete config observation from HA
- keep `cfg` at the top of `HaRuntimeCtx`
- remove cadence/config duplication that only carries poll interval and similar static values
- collapse role reconciliation signatures so cfg-derived fields are not passed separately
- keep only observed pg/dcs/process state and HA-owned decision/publication state outside cfg
- add a progress log section to this task as collapses are completed and keep `<passes>false</passes>`
- when this task is fully complete, update the task body with `QUIT IMMEDIATLY`

**Expected outcome:**
- HA does not observe config changes
- HA reads static config directly from `cfg`
- cfg-derived helper arguments disappear
- `src/ha/` has zero dependency on `src/config/`

</description>

<acceptance_criteria>
- [ ] `HaObservedState.config` is deleted
- [ ] `HaRuntimeCtx` keeps `cfg` at the highest level and no nested config mirror structs are introduced
- [ ] `HaWorkerCadence.poll_interval` config duplication is deleted or proven runtime-only
- [ ] HA role reconciliation no longer passes cfg-derived socket dir and port as separate arguments
- [ ] `src/ha/` has zero dependency on `src/config/`
- [ ] The task body contains a progress log of removals and still keeps `<passes>false</passes>`
- [ ] The task body is updated to include `QUIT IMMEDIATLY` when done
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
