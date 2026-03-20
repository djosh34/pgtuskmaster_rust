## Task: Collapse `process/` By Deleting Config-Mirror Plans And Reading `cfg` Directly <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove the deepest config redistribution corridor in the codebase by deleting process-local config mirror structs and making `process/` read static config directly from one shared `cfg: &RuntimeConfigV2`. The higher-order goal is to iteratively collapse copied fields, copied args, copied plan structs, and copied job spec fields until only dynamic process execution state remains outside cfg.

This is expected to be the hardest package task in the story.

**Scope:**
- `src/process/state.rs`
- `src/process/startup.rs`
- `src/process/planner.rs`
- `src/process/session.rs`
- `src/process/tools.rs`
- `src/process/cluster.rs`
- `src/process/worker.rs`
- `src/process/jobs.rs`
- `src/process/source.rs`
- `src/postgres_managed.rs`
- `src/postgres_roles.rs` where process-driven config arguments are still sprayed

**Context from research:**
- `ProcessRuntimePlan`, `ManagedPostgresPaths`, `ManagedPostgresRuntime`, `ReplicaAccessRuntime`, and `MandatoryPostgresRoleCredential` are config mirrors
- `ProcessObservedState.runtime_config` and `ProcessObservedSnapshot.runtime_config` keep the same config object moving through another corridor
- `process/jobs.rs` carries many static config fields such as `data_dir`, `socket_dir`, `log_file`, and `timeout_ms`
- `process/session.rs` still takes full runtime config to materialize managed postgres config
- `process/tools.rs` and `process/planner.rs` still lower static config into more structs before execution

Representative current code:

```rust
pub(crate) struct ProcessRuntimePlan {
    pub(crate) postgres: ManagedPostgresRuntime,
    pub(crate) replica_access: ReplicaAccessRuntime,
}
```

```rust
pub(crate) struct ProcessObservedSnapshot {
    pub(crate) runtime_config: RuntimeConfig,
    pub(crate) dcs: DcsSnapshot,
    pub(crate) managed_recovery_state: ManagedRecoverySignal,
}
```

```rust
pub(crate) struct StartPostgresSpec {
    pub(crate) mode: PostgresStartMode,
    pub(crate) data_dir: PathBuf,
    pub(crate) socket_dir: PathBuf,
    pub(crate) port: u16,
    pub(crate) config_file: PathBuf,
    pub(crate) log_file: PathBuf,
    pub(crate) wait_seconds: Option<u64>,
    pub(crate) timeout_ms: Option<u64>,
}
```

**Implementation direction:**
- delete `ProcessRuntimePlan` rather than porting it to V2
- delete process-local config mirror structs wherever possible
- move `cfg` to the top of the highest process context and keep nested state free of config-like fields
- collapse planner/session/tools/job-spec signatures so only dynamic intent-specific facts survive
- where a helper currently takes cfg plus exploded fields, collapse that helper to accept cfg only or inline it
- do not replace the old corridor with `ProcessConfigV2`, `ProcessRuntimeConfig`, `ProcessRuntimePlanV2`, or any similar new shape
- follow the chain until copied config fields disappear from specs, plans, inputs, and contexts
- add a progress log section to this task as collapses are completed and keep `<passes>false</passes>`
- when this task is fully complete, update the task body with `QUIT IMMEDIATLY`

**Expected outcome:**
- `process/` directly uses `RuntimeConfigV2`
- config-shaped plans/specs/inputs are overwhelmingly deleted
- only dynamic process state remains outside cfg
- `src/process/` has zero dependency on `src/config/`

</description>

<acceptance_criteria>
- [ ] `ProcessRuntimePlan`, `ManagedPostgresPaths`, `ManagedPostgresRuntime`, and `ReplicaAccessRuntime` are removed or proven unnecessary by stronger collapse
- [ ] `ProcessObservedState.runtime_config` and `ProcessObservedSnapshot.runtime_config` are deleted
- [ ] Job/spec structs stop carrying static config fields like data dirs, socket dirs, log files, ports, timeouts, and similar cfg-derived values unless a field is proven to be truly dynamic
- [ ] `process/session.rs`, `process/planner.rs`, and `process/tools.rs` read from `cfg` directly instead of through package-local config mirrors
- [ ] `process/startup.rs` and `process/worker.rs` keep `cfg` only at the highest context level plus non-config runtime state
- [ ] `src/process/` has zero dependency on `src/config/`
- [ ] The task body contains a progress log of removals and still keeps `<passes>false</passes>`
- [ ] The task body is updated to include `QUIT IMMEDIATLY` when done
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
