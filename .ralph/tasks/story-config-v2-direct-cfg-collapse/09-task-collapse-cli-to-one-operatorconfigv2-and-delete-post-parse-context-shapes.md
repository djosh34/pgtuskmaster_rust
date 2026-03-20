## Task: Collapse `cli/` To One `OperatorConfigV2` And Delete Post-Parse Context Shapes <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Rewrite `pgtm` so it loads one `OperatorConfigV2` and then uses that cfg directly for all commands. The higher-order goal is to delete dual runtime/operator parsing and to remove operator post-parse context shapes that still rematerialize config.

**Scope:**
- `src/cli/config.rs`
- `src/cli/connect.rs`
- `src/cli/client.rs`
- `src/cli/status.rs`
- `src/cli/switchover.rs`
- `src/cli/mod.rs`
- `src/cli/args.rs`
- any CLI tests or helpers that still depend on old config types

**Context from research:**
- `src/cli/config.rs` currently dual-parses runtime and operator documents
- `OperatorContext` and `OperatorConfigSource` are post-parse config corridors
- `cli/connect.rs` reads primary connection override behavior through those post-parse types
- user decisions:
  - `pgtm` accepts one config file and calls one exposed `config_v2/parser` loader
  - tokens are independently optional
  - primary override port is optional
  - operator TLS should keep validated paths only
  - collapse more, not less

Representative current code:

```rust
struct OperatorConfigSource {
    runtime: Option<RuntimeConfig>,
    operator: Option<PgtmConfig>,
}
```

```rust
pub(crate) struct OperatorContext {
    pub(crate) api_client: CliApiClientConfig,
    pub(crate) postgres_client_tls: CliTlsConfig,
    pub(crate) api_auth_enabled: bool,
}
```

**Implementation direction:**
- delete dual runtime/operator parsing in CLI
- make CLI load one `OperatorConfigV2`
- delete `OperatorConfigSource`
- delete `OperatorContext` if possible; if a tiny execution context survives, it must not duplicate config facts already present in `OperatorConfigV2`
- make `cli/connect.rs` read the primary override directly from `OperatorConfigV2`
- keep only runtime HTTP client objects and command result shaping outside cfg
- add a progress log section to this task as collapses are completed and keep `<passes>false</passes>`
- when this task is fully complete, update the task body with `QUIT IMMEDIATLY`

**Expected outcome:**
- `pgtm` uses one shared `OperatorConfigV2`
- no CLI code reparses or revalidates config
- operator-specific context/config mirror structs are deleted
- `src/cli/` has zero dependency on `src/config/`

</description>

<acceptance_criteria>
- [ ] CLI loads exactly one `OperatorConfigV2` through `config_v2/parser`
- [ ] `OperatorConfigSource` is deleted
- [ ] `OperatorContext` is deleted or reduced so it contains no config facts already present in `OperatorConfigV2`
- [ ] `cli/connect.rs` reads primary override behavior directly from `OperatorConfigV2`
- [ ] `src/cli/` has zero dependency on `src/config/`
- [ ] The task body contains a progress log of removals and still keeps `<passes>false</passes>`
- [ ] The task body is updated to include `QUIT IMMEDIATLY` when done
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
