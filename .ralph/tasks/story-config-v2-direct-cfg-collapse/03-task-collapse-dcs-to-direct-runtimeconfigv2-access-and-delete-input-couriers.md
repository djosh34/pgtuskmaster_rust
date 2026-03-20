## Task: Collapse `dcs/` To Direct `RuntimeConfigV2` Access And Delete Input Couriers <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove config mirroring from `src/dcs/` so the worker holds `cfg: &RuntimeConfigV2` directly and reads endpoints, auth, TLS, lease timing, scope, and advertised PostgreSQL values from cfg at the use site. The higher-order goal is to finish the smell-2 and smell-1 cleanup in DCS by deleting `Inputs`/courier layers that only restate static config.

**Scope:**
- `src/dcs/mod.rs`
- `src/dcs/worker.rs`
- any DCS tests that currently rely on old `crate::config` types

**Context from research:**
- `src/dcs/mod.rs` currently explodes config into `cfg.dcs.endpoints.clone()`, `cfg.dcs.client.clone()`, derived poll interval, lease TTL, and advertised endpoint
- `src/dcs/worker.rs` stores those values in `DcsWorkerInputs`
- the user explicitly wants stack collapse, including removing `inputs` if it is only a config courier

Representative current code:

```rust
struct DcsWorkerInputs {
    identity: NodeIdentity,
    keys: DcsKeySpace,
    endpoints: Vec<DcsEndpoint>,
    client: DcsClientConfig,
    poll_interval: Duration,
    member_ttl_ms: u64,
    advertised_postgres: PgEndpoint,
    pg: StateSubscriber<PgInfoState>,
    publisher: StatePublisher<DcsSnapshot>,
    command_inbox: DcsCommandInbox,
    log: LogSender,
}
```

**Implementation direction:**
- remove `DcsWorkerInputs` or reduce it until it contains no config-like fields at all
- prefer `self.cfg` over `self.inputs.cfg`; collapse the extra layer if it buys nothing
- remove duplicated config-derived fields such as endpoints, client config, lease ttl, poll interval, and advertised endpoint from worker state
- keep only true runtime state such as subscribers, publishers, command inboxes, sessions, and cluster state
- do not create a DCS-specific runtime config struct as a replacement
- add a progress log section to this task as collapses are completed and keep `<passes>false</passes>`
- when this task is fully complete, update the task body with `QUIT IMMEDIATLY`

**Expected outcome:**
- DCS uses `RuntimeConfigV2` directly
- courier layers that only restate config are deleted
- `src/dcs/` no longer imports anything from `src/config/`

</description>

<acceptance_criteria>
- [ ] `src/dcs/mod.rs` stops exploding cfg into cloned fields before worker construction
- [ ] `src/dcs/worker.rs` no longer stores endpoints, client config, poll interval, lease ttl, or advertised postgres as copied config fields
- [ ] `DcsWorkerInputs` is removed or reduced so it contains no config-like fields whatsoever
- [ ] DCS connect/auth/TLS/lease logic reads directly from `cfg`
- [ ] `src/dcs/` has zero dependency on `src/config/`
- [ ] The task body contains a progress log of removals and still keeps `<passes>false</passes>`
- [ ] The task body is updated to include `QUIT IMMEDIATLY` when done
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
