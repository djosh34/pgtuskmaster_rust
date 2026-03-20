## Task: Collapse `runtime/` To One Shared `cfg: &RuntimeConfigV2` Reference <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Rewrite `src/runtime/` so it holds exactly one shared `cfg: &RuntimeConfigV2` and stops redistributing config through bootstrap layers, state channels, cloned roots, or package-specific startup wrappers. The higher-order goal is to make runtime the composition root that passes the same config object downward without turning it into more config-shaped state.

**Scope:**
- `src/runtime/node.rs`
- `src/runtime/log_event.rs`
- any immediate startup callsites that currently require `RuntimeConfig` clones or config state channels

**Context from research:**
- `src/runtime/node.rs` currently clones config, publishes it into a state channel, and explodes values like cluster identity and poll interval before worker startup
- `src/runtime/log_event.rs` still depends on `crate::config::LogLevel`
- the user explicitly rejected dynamic config and rejected package-local config mirrors

Representative current code:

```rust
let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg.clone());
let identity = NodeIdentity {
    cluster_name: ClusterName(cfg.cluster.name.clone()),
    scope: ScopeName(cfg.cluster.scope.clone()),
    member_id: MemberId(cfg.cluster.member_id.clone()),
};
let worker_poll_interval = Duration::from_millis(cfg.ha.loop_interval_ms);
```

**Implementation direction:**
- remove the config state channel completely
- pass the same shared `cfg` downward instead of cloning `cfg` or cloning static fields from it into local startup variables
- collapse startup wrappers when they only courier cfg-derived values into worker constructors
- switch runtime logging events to V2 enums/types directly, not `crate::config::*`
- prefer fewer startup layers and fewer bundles where the bundle only exists to carry cfg-derived state
- add a progress log section to this task as collapses are completed and keep `<passes>false</passes>`
- when this task is fully complete, update the task body with `QUIT IMMEDIATLY`

**Expected outcome:**
- `runtime/node.rs` no longer creates fake config dynamism
- runtime passes one shared `cfg` object directly
- config-like startup variables stop being fanned out into separate arguments
- `runtime/` no longer imports `crate::config`

</description>

<acceptance_criteria>
- [ ] `src/runtime/node.rs` does not create a config state channel
- [ ] `src/runtime/node.rs` stops cloning the config root for worker startup
- [ ] `src/runtime/log_event.rs` no longer depends on `crate::config::*`
- [ ] Runtime startup wrappers that only courier cfg-derived values are deleted or collapsed
- [ ] `src/runtime/` imports `config_v2` types only, with zero `src/config/` dependency left
- [ ] The task body contains a progress log of removals and still keeps `<passes>false</passes>`
- [ ] The task body is updated to include `QUIT IMMEDIATLY` when done
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
