## Task: Collapse `pginfo/` To Direct `RuntimeConfigV2` Access <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove config-shaped cached state from `src/pginfo/` so the worker owns `cfg: &RuntimeConfigV2` and reads probe connection details and cadence directly from the shared config object. The higher-order goal is to stop treating probe conninfo and polling settings as package-local config materializations.

**Scope:**
- `src/pginfo/startup.rs`
- `src/pginfo/state.rs`
- `src/pginfo/worker.rs`
- any pginfo tests or helpers that still depend on old config types

**Context from research:**
- `src/pginfo/startup.rs` currently materializes `probe_conninfo` and cadence from `RuntimeConfig`
- `PgInfoWorkerCtx` stores both
- the user wants top-level `cfg` plus non-config runtime state only

Representative current code:

```rust
pub(crate) struct PgInfoWorkerCtx {
    pub(crate) identity: NodeIdentity,
    pub(crate) probe_conninfo: PgConnInfo,
    pub(crate) cadence: PgInfoCadence,
    pub(crate) state_channel: PgInfoStateChannel,
    pub(crate) runtime: PgInfoRuntime,
}
```

**Implementation direction:**
- move `cfg` onto the highest pginfo context
- remove cached config-derived fields like `probe_conninfo` and poll interval unless they are proven to be unavoidable runtime state
- do not introduce a `PgInfoConfig` replacement
- keep only runtime publisher/log state and truly dynamic observation state
- add a progress log section to this task as collapses are completed and keep `<passes>false</passes>`
- when this task is fully complete, update the task body with `QUIT IMMEDIATLY`

**Expected outcome:**
- pginfo reads from `RuntimeConfigV2` directly
- package-local config mirror fields disappear
- `src/pginfo/` has zero dependency on `src/config/`

</description>

<acceptance_criteria>
- [ ] `PgInfoWorkerCtx` holds `cfg` at the top level and no package-local config mirror structs are introduced
- [ ] `probe_conninfo` and cadence config duplication are removed or reduced to runtime-only facts with a documented reason
- [ ] `src/pginfo/startup.rs` stops materializing package-local config mirrors from cfg
- [ ] `src/pginfo/` has zero dependency on `src/config/`
- [ ] The task body contains a progress log of removals and still keeps `<passes>false</passes>`
- [ ] The task body is updated to include `QUIT IMMEDIATLY` when done
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
