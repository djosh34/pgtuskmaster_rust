## Task: Collapse `api/` To Direct `RuntimeConfigV2` Access And Path-Based Reload <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove config state/mirror behavior from `src/api/` so the runtime context keeps only `cfg: &RuntimeConfigV2` plus live server/runtime state. The higher-order goal is to make API auth, bind, TLS, and certificate reload use cfg directly without spawning new config-shaped state.

**Scope:**
- `src/api/startup.rs`
- `src/api/worker.rs`
- `src/dev_support/api.rs`
- any API tests that still depend on old config types

**Context from research:**
- `ApiRuntimeCtx` currently contains `StateSubscriber<RuntimeConfig>`
- API startup reads config once, then stores derived `bind`, `auth`, and `transport`
- reload currently depends on runtime config subscriber and additional transport/runtime wrappers
- user decision: reload needs only the paths already in cfg; it should reread those paths directly from cfg and not create additional config state

Representative current code:

```rust
pub(crate) struct ApiRuntimeCtx {
    pub(crate) identity: NodeIdentity,
    pub(crate) observed: ApiObservedState,
    pub(crate) runtime_config: StateSubscriber<RuntimeConfig>,
    pub(crate) dcs_handle: DcsHandle,
    pub(crate) bind: ApiBindConfig,
    pub(crate) auth: TokenAuth,
    pub(crate) transport: ApiServerTransport,
    pub(crate) reload_certificates: ApiReloadCertificatesHandle,
    pub(crate) _log: LogSender,
}
```

**Implementation direction:**
- replace config subscriber with top-level shared cfg
- collapse config-derived `bind`/`auth`/transport state where they are only cached config
- keep only true runtime server objects, such as live rustls reload handles, outside cfg
- make certificate reload reread path-backed material directly from cfg
- do not introduce an `ApiConfigRuntime` replacement
- add a progress log section to this task as collapses are completed and keep `<passes>false</passes>`
- when this task is fully complete, update the task body with `QUIT IMMEDIATLY`

**Expected outcome:**
- API uses `RuntimeConfigV2` directly
- reload logic depends only on runtime server handles plus cfg paths
- `src/api/` has zero dependency on `src/config/`

</description>

<acceptance_criteria>
- [ ] `ApiRuntimeCtx` no longer contains a config subscriber
- [ ] API request auth resolution reads directly from cfg
- [ ] API bind/transport/auth config duplication is removed or reduced to true runtime state only
- [ ] Certificate reload rereads only the paths already stored in cfg and does not depend on reparse or config mirrors
- [ ] `src/api/` has zero dependency on `src/config/`
- [ ] The task body contains a progress log of removals and still keeps `<passes>false</passes>`
- [ ] The task body is updated to include `QUIT IMMEDIATLY` when done
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
