# Config not reduced smell

I'll show you the cfg reduce loop.

1) Detect the wrong field
   Have this code:


```rust
pub(crate) struct HaRuntimeCtx<'a> {
    pub(crate) cfg: &'a RuntimeConfigV2,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    pub(crate) state_channel: HaStateChannel,
    pub(crate) observed: HaObservedState,
    pub(crate) control: HaControlPlane,
    pub(crate) identity: NodeIdentity, // Should be in config
}
```

I know for a fact that identity is part of cfg. Many of such variables (think timings, paths, anything config, all config is static), should directly use cfg.



2) Alter name of field without the uses of it
3)
```rust
pub(crate) struct HaRuntimeCtx<'a> {
    pub(crate) cfg: &'a RuntimeConfigV2,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    pub(crate) state_channel: HaStateChannel,
    pub(crate) observed: HaObservedState,
    pub(crate) control: HaControlPlane,
    pub(crate) to_be_deleted_identity: NodeIdentity, // Should be in config
}
```

This will cause `make check` issues. Solve those issues by using cfg directly:



e.g.:

Before:
```rust
pub(crate) fn dispatch_process_action(
    ctx: &mut HaRuntimeCtx,
    ha_tick: u64,
    action_index: usize,
    action: &ProcessIntent,
) -> Result<(), ProcessDispatchError> {
...
            ctx.identity.scope.as_str().trim_matches('/'),
            ctx.identity.member_id.0,
...
}
```

After:
```rust
pub(crate) fn dispatch_process_action(
    ctx: &mut HaRuntimeCtx,
    ha_tick: u64,
    action_index: usize,
    action: &ProcessIntent,
) -> Result<(), ProcessDispatchError> {
...
            ctx.cfg.scope.as_str().trim_matches('/'),
            ctx.cfg.member_id.0,
...
}
```
