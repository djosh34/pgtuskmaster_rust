## Task: Rebuild `dev_support/` And Tests Around V2 Config Only <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove test/dev preservation of the old config tree by rebuilding helpers, builders, harnesses, and fixtures around `RuntimeConfigV2` and `OperatorConfigV2` only. The higher-order goal is to prevent tests from keeping `src/config/` alive after production code migrates.

**Scope:**
- `src/dev_support/runtime_config.rs`
- `src/dev_support/api.rs`
- `src/dev_support/auth.rs`
- `src/dev_support/binaries.rs`
- `tests/cli_binary.rs`
- `tests/ha/support/**`
- `tests/bdd_api_http.rs`
- `tests/bdd_state_watch.rs`
- any remaining tests or support modules that parse or construct old config types

**Context from research:**
- `src/dev_support/runtime_config.rs` currently entrenches the old config tree with `RuntimeConfigBuilder`
- `src/dev_support/api.rs` still takes `config::RuntimeConfig`
- several HA helpers still parse TOML into `pgtuskmaster_rust::config::RuntimeConfig`
- tests are a major preservation corridor for the old types

**Implementation direction:**
- replace old builders with V2 builders/constructors only
- keep test ergonomics, but do not preserve package-local config mirrors
- switch tests that parse TOML to use the one V2 loader or direct V2 constructors
- remove all `pgtuskmaster_rust::config::RuntimeConfig` / old operator-config usage from tests and helpers
- add a progress log section to this task as collapses are completed and keep `<passes>false</passes>`
- when this task is fully complete, update the task body with `QUIT IMMEDIATLY`

**Expected outcome:**
- tests no longer keep `src/config/` alive
- dev support uses V2 config only
- fixtures and builders match the new direct-cfg architecture

</description>

<acceptance_criteria>
- [ ] `src/dev_support/runtime_config.rs` no longer builds old `config::RuntimeConfig`
- [ ] `src/dev_support/api.rs` no longer takes old config types
- [ ] Tests do not parse or construct `pgtuskmaster_rust::config::RuntimeConfig`
- [ ] CLI and HA support code use `OperatorConfigV2` / `RuntimeConfigV2` only
- [ ] The task body contains a progress log of removals and still keeps `<passes>false</passes>`
- [ ] The task body is updated to include `QUIT IMMEDIATLY` when done
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
