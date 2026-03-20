## Task: Collapse `logging/` To Direct `RuntimeConfigV2` Access <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove logging config mirrors so `logging/` reads static config directly from one shared `cfg: &RuntimeConfigV2` and keeps only true runtime sink/tailer state outside it. The higher-order goal is to stop packaging log context, sink settings, postgres log paths, and cleanup settings into extra config-like corridors.

**Scope:**
- `src/logging/core/runtime.rs`
- `src/logging/postgres_ingest.rs`
- `src/logging/core/queued_record.rs`
- any logging tests still tied to old config types

**Context from research:**
- `logging/core/runtime.rs` still depends on `crate::config::LogLevel`, `FileSinkMode`, and full runtime config
- `logging/postgres_ingest.rs` stores `cfg: RuntimeConfig` and repeatedly reads config-like fields from that copy
- user decision: even apparently harmless config copies should be treated as collapse targets if lifetimes permit

Representative current code:

```rust
pub(crate) struct PostgresIngestWorkerCtx {
    pub(crate) cfg: RuntimeConfig,
    pub(crate) log: LogSender,
}
```

**Implementation direction:**
- move to V2 logging enums and types directly
- keep cfg borrowed/shared, not cloned
- remove any logging package-local config runtime wrappers that only restate cfg
- for postgres ingest, keep runtime tailers/rate limiters/state only; read static paths and cleanup settings from cfg
- add a progress log section to this task as collapses are completed and keep `<passes>false</passes>`
- when this task is fully complete, update the task body with `QUIT IMMEDIATLY`

**Expected outcome:**
- logging no longer imports `crate::config::*`
- postgres ingest does not own a copied config root
- `src/logging/` has zero dependency on `src/config/`

</description>

<acceptance_criteria>
- [ ] `logging/core/runtime.rs` uses V2 enums/types directly instead of `crate::config::*`
- [ ] `PostgresIngestWorkerCtx` no longer owns a copied config root
- [ ] Any logging package-local config mirror layers are removed
- [ ] `src/logging/` has zero dependency on `src/config/`
- [ ] The task body contains a progress log of removals and still keeps `<passes>false</passes>`
- [ ] The task body is updated to include `QUIT IMMEDIATLY` when done
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
