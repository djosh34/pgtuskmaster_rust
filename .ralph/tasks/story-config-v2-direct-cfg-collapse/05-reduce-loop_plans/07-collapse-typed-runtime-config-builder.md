## Plan: Collapse Typed Runtime Config Builder Boundary

### Why this reduction target

The previous slice moved TOML rendering and contents-based parsing back into `config_v2`, but one duplicate config-construction layer still survives:

- `src/config_v2/parser/private_schema.rs` already owns the canonical runtime-document defaults used for test config rendering.
- `src/config_v2/parser/load_config.rs` already owns the typed `RuntimeConfigV2` mapping and validation path.
- `src/dev_support/runtime_config.rs` still re-encodes those same defaults as a second typed `RuntimeConfigV2` builder API (`RuntimeConfigBuilder`, sample config helpers, sample logging/binary defaults, and token-auth glue).
- Tests in `src/process/{cluster,planner,session,source,tools,worker}.rs`, `src/api/worker.rs`, `src/ha/worker.rs`, `src/postgres_managed.rs`, `src/logging/{core/runtime,postgres_ingest}.rs`, and `src/dev_support/api.rs` still depend on that extra builder layer.

That is still the wrong boundary. `config_v2` already owns config documents and the typed validated config, but tests are routed through a second sample-config DSL in `dev_support` anyway. The result is duplicated defaults, duplicated helper tests, and another module that needs to stay in sync with the parser.

### Current overlap already verified

- `src/config_v2/parser/private_schema.rs` exposes `build_runtime_test_document_value` and `render_runtime_test_config_toml`, which already define the baseline runtime test document shape.
- `src/config_v2/parser/load_config.rs` exposes `load_runtime_config_contents`, which already turns those raw defaults back into a validated `RuntimeConfigV2`.
- `src/dev_support/runtime_config.rs` still duplicates the same baseline values in typed form through `sample_postgres_config`, `sample_dcs_config`, `sample_logging_config`, `sample_binary_paths`, and `RuntimeConfigBuilder`.
- The remaining `RuntimeConfigBuilder` call sites mostly apply a small set of overrides: `data_dir`, timing/logging tweaks, API auth, HBA contents, and a few direct field mutations after `build()`.
- `api_auth_from_optional_tokens` is only used by `src/dev_support/api.rs` and `src/api/worker.rs` tests, so its ownership can move with the builder collapse instead of keeping `dev_support/runtime_config.rs` alive on its own.

### Execution plan

1. Move typed runtime test-config ownership into `config_v2` test support.
   - Add one small helper in `config_v2` test support that returns a baseline `RuntimeConfigV2` by reusing the existing runtime test document builders plus `load_runtime_config_contents`.
   - Add targeted override entrypoints only for the cases that are genuinely repeated across callers, instead of recreating a second general-purpose builder type.
   - Reuse existing raw document builders and `RuntimeConfigV2`; do not create new builder structs or new config types.

2. Retarget tests and helpers away from `RuntimeConfigBuilder`.
   - Update process, HA, API, logging, postgres-managed, and dev-support call sites to start from the config-owned baseline helper and then apply local direct mutations where the test is truly specific.
   - Inline tiny one-off adjustments at the call site instead of preserving shared wrapper methods like `with_postgres_data_dir`, `with_pg_hba_contents`, or `for_trace_logging_tests` when those helpers are only hiding plain field assignments.
   - Move the small API token helper to the narrowest remaining owner, or inline it into the few test helpers that actually need it.

3. Delete the duplicate builder layer.
   - Remove `RuntimeConfigBuilder` and the duplicated sample default constructors from `src/dev_support/runtime_config.rs`.
   - Delete any builder-only tests that are just re-testing that the duplicate builder preserves fields, because the parser-owned config helper becomes the only supported construction path.
   - If the file becomes trivial after the collapse, remove the module entirely and drop its export from `src/dev_support/mod.rs`.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4063 -5998 diff: -1935` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Tick the task boxes only after the gates pass.

### Guardrails

- Keep `config_v2` as the single owner of runtime test config defaults and typed loading semantics.
- Prefer direct field mutation in individual tests over introducing a new shared builder abstraction.
- Reuse existing config types and raw document builders; do not invent a third layer.
- If the replacement helper surface starts growing into another builder DSL instead of deleting code, switch this plan back to `TO BE VERIFIED`, document the mismatch clearly, retarget the task file, and stop immediately.

NOW EXECUTE
