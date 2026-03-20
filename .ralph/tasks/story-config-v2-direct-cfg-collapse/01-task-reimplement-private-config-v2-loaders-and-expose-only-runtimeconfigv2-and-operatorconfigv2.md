## Task: Reimplement Private `config_v2` Loaders And Expose Only `RuntimeConfigV2` And `OperatorConfigV2` <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Replace the old `src/config/` parser entrypoints with one private `config_v2` ingestion boundary that parses exactly once and returns either a fully validated `RuntimeConfigV2` or a fully validated `OperatorConfigV2`. The higher-order goal is to make config ingestion a one-time private concern and to prevent every downstream package from reparsing, revalidating, or materializing config-like fields into new structs.

This task is the root of the story. Every follow-up task depends on this exact rule:
- daemon/runtime code gets one shared `cfg: &RuntimeConfigV2`
- `pgtm`/CLI code gets one shared `cfg: &OperatorConfigV2`
- `src/config/` is transitional only during the story and must be fully removed by the final task
- the parser is private; the exposed surface is only the load function returning a validated V2 type
- post-parse validation anywhere else in the codebase is a bug

**Decisions already made from user discussion:**
- `pgtm` accepts one config file and calls one exposed function inside `config_v2/parser` that returns a fully validated `OperatorConfigV2`
- `RuntimeConfigV2` and `OperatorConfigV2` are the only config objects allowed after parsing
- no package-local config DTOs, runtime plans, inputs, or mirrors may be created as replacements
- `OperatorConfigV2` should collapse more of the operator document, not preserve `advertised_url`, `expected_transport`, or similar split state
- operator tokens are independently optional
- the primary PostgreSQL override port is optional
- operator TLS should keep validated paths, not loaded cert bytes
- `config/` must have zero remaining dependents by the end of the story

**Scope:**
- `src/config_v2/types.rs`
- `src/config_v2/mod.rs`
- `src/config_v2/parser/mod.rs`
- `src/config_v2/parser/load_config.rs`
- add any missing private parser files under `src/config_v2/parser/`
- `src/lib.rs`
- any top-level callsites that need to switch imports from `config` to `config_v2`

**Context from research:**
- `src/config_v2/parser/load_config.rs` currently returns `Not implemented`
- `src/config_v2/types.rs` already defines `RuntimeConfigV2` and `OperatorConfigV2`, but `OperatorConfigV2` still needs shape cleanup
- `src/config/parser.rs` currently exposes:
  - `load_runtime_config(path) -> RuntimeConfig`
  - `load_operator_config(path) -> PgtmConfig`
  - `validate_runtime_config`
  - `validate_operator_config`
- the old parser still accepts dual runtime/operator document parsing for CLI; that dual path must not survive

Representative current code that this task replaces:

```rust
// src/config_v2/parser/load_config.rs
pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    Err(ConfigErrorV2::Validation {
        field: "Not implemented",
        message: "Not implemented".to_string(),
    })
}
```

```rust
// src/config/parser.rs -> and then move to src/config_v2/parser/load_operator_config.rs
pub fn load_operator_config(path: &Path) -> Result<PgtmConfig, ConfigError> {
    let contents = read_config_file(path)?;
    let document: OperatorConfigDocument = toml::from_str(&contents)?;
    let cfg = match document {
        OperatorConfigDocument::Operator(cfg) => *cfg,
        OperatorConfigDocument::Runtime(runtime) => runtime.pgtm.ok_or_else(...)?,
    };
    validate_operator_config(&cfg)?;
    Ok(cfg)
}
```

**Implementation direction:**
- keep raw document DTOs private inside `src/config_v2/parser/`
- parse once, validate once, materialize once
- expose only:
  - runtime loader returning `RuntimeConfigV2`
  - operator loader returning `OperatorConfigV2`
- do not expose raw DTOs or validation helpers
- delete or stop exporting all old `validate_*` entrypoints
- keep impossible states out of the V2 types
- do not preserve old runtime/operator dual parsing for CLI
- add a progress log section to this task as the work proceeds and keep `<passes>false</passes>`
- when this task is fully complete, update the task body with `QUIT IMMEDIATLY`

**Expected outcome:**
- `config_v2` is the only config ingestion boundary still being built on
- a caller can get exactly one validated `RuntimeConfigV2`
- a caller can get exactly one validated `OperatorConfigV2`
- no downstream package needs to call validation or materialization helpers
- the rest of the story can mechanically replace `crate::config::*` usage with direct V2 cfg access

</description>

<acceptance_criteria>
- [ ] `src/config_v2/parser/` has one exposed runtime load path and one exposed operator load path, both returning validated V2 config objects
- [ ] Raw document DTOs used for TOML parsing are private to `src/config_v2/parser/`
- [ ] `OperatorConfigV2` is tightened to the collapsed operator surface decided in this chat
- [ ] No new package-local config mirror structs are introduced in `config_v2`
- [ ] Old `validate_runtime_config` and `validate_operator_config` are no longer part of the intended public runtime path
- [ ] The task body contains a progress log of removed parser/validation corridors and still keeps `<passes>false</passes>`
- [ ] The task body is updated to include `QUIT IMMEDIATLY` when done
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
