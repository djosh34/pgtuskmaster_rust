## Task: Centralize composable sample runtime-config builders for tests and helpers <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>01-task-introduce-a-typed-managed-postgres-conf-model-and-serializer,02-task-make-pgtm-postgresql-conf-the-only-startup-config-entrypoint,03-task-take-full-ownership-of-replica-recovery-signal-and-auto-conf-state</blocked_by>

<description>
**Goal:** Replace the current duplicated `RuntimeConfig` sample/config-literal sprawl with one composable test-builder layer built from small partial sample functions and explicit overrides.
The higher-order goal is to make tests cheaper to maintain while the managed-config ownership refactor lands, and to stop configuration fixtures from drifting or preserving stale startup semantics in dozens of isolated literals.

**Scope:**
- Create one central test-support module for composable sample runtime config construction.
- Break sample config construction into small typed partial builders so each test can compose only the parts it needs, then override specific fields without copying the full config literal.
- Migrate direct duplicated `RuntimeConfig` literals and duplicated “sample config” helpers in unit tests, integration tests, examples, and helper modules to the shared builder layer where they are testing pgtuskmaster behavior rather than config-parsing itself.
- Preserve intentionally explicit parser/config-fixture tests where the test is about exact user-facing config shape, not general runtime setup.
- Make the builder layer compatible with the authoritative managed-config model so it becomes the standard way to create valid runtime configs in tests after the conf refactor.

**Context from research:**
- Repo search already shows many repeated `RuntimeConfig` literals and repeated `PgHbaConfig` / `PgIdentConfig` / TLS / role blocks across `src/`, `tests/`, and `examples/`.
- This duplication is not wrong in isolation, but it increases drift risk exactly in the area that is being redesigned: startup config ownership.
- The new authoritative managed-config story will otherwise cause repetitive mechanical edits across a large fixture surface.

**Expected outcome:**
- Tests and helper modules construct runtime config from one composable shared layer.
- Configuration changes in the managed-config story can be propagated by updating shared builder parts instead of chasing many full copied literals.
- Parser-shape tests remain explicit where that explicitness is the purpose of the test.

</description>

<acceptance_criteria>
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] Add one central test-support module for composable runtime-config construction
- [x] Shared builder API is composed from partial sample functions or typed builder parts rather than one giant monolithic full-config literal only
- [x] Duplicate sample-config helpers and repeated full `RuntimeConfig` literals are migrated where the test is not specifically about parser input shape
- [x] Parser/config-shape tests that need literal inline TOML or explicit field-by-field configs remain explicit and are not forced through the generic sample builder
- [x] The shared builder layer is aligned with the authoritative managed-config model and does not preserve generic `-c key=value` startup assumptions
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

### Exhaustive checklist (must be treated as part of acceptance)

- [x] Add one shared test-support module, likely under `src/test_harness/` or a dedicated crate-internal test support area
  - [x] Provide small partial sample functions for common config fragments:
  - [x] cluster identity
  - [x] postgres core paths/listen settings
  - [x] postgres roles/auth
  - [x] postgres TLS inputs
  - [x] postgres managed HBA/ident inputs
  - [x] DCS settings
  - [x] API security settings
  - [x] logging/process/debug defaults
  - [x] Provide one easy composition path for a “valid full sample runtime config” plus targeted overrides.
- [x] Migrate duplicated sample config helpers and/or repeated `RuntimeConfig` literals in:
  - [x] `src/api/fallback.rs`
  - [x] `src/api/worker.rs`
  - [x] `src/debug_api/worker.rs`
  - [x] `src/dcs/etcd_store.rs`
  - [x] `src/dcs/state.rs`
  - [x] `src/dcs/store.rs`
  - [x] `src/dcs/worker.rs`
  - [x] `src/ha/decide.rs`
  - [x] `src/ha/events.rs`
  - [x] `src/ha/process_dispatch.rs`
  - [x] `src/ha/worker.rs`
  - [x] `src/logging/mod.rs`
  - [x] `src/logging/postgres_ingest.rs`
  - [x] `src/runtime/node.rs`
  - [x] `src/worker_contract_tests.rs`
  - [x] `tests/bdd_api_http.rs`
  - [x] `examples/debug_ui_smoke_server.rs`
- [x] Review `src/config/parser.rs` tests separately
  - [x] Keep literal TOML fixtures and exact parser-error tests explicit where they are intentionally testing parser shape rather than generic runtime behavior.
- [x] Add focused tests for the shared builder layer
  - [x] Assert it produces a valid baseline config.
  - [x] Assert targeted overrides do not accidentally drop required secure fields.
  - [x] Assert builder parts remain aligned with the authoritative managed-config contract.

## Plan (composable runtime-config builder centralization)

### Research summary and hard constraints

- The duplication is real and spread across crate-unit tests, integration tests, examples, and harness helpers:
  - `src/api/fallback.rs`
  - `src/api/worker.rs`
  - `src/debug_api/worker.rs`
  - `src/dcs/etcd_store.rs`
  - `src/dcs/state.rs`
  - `src/dcs/store.rs`
  - `src/dcs/worker.rs`
  - `src/ha/decide.rs`
  - `src/ha/events.rs`
  - `src/ha/process_dispatch.rs`
  - `src/ha/worker.rs`
  - `src/logging/mod.rs`
  - `src/logging/postgres_ingest.rs`
  - `src/postgres_managed.rs`
  - `src/runtime/node.rs`
  - `src/test_harness/ha_e2e/startup.rs`
  - `src/worker_contract_tests.rs`
  - `tests/bdd_api_http.rs`
  - `examples/debug_ui_smoke_server.rs`
- `src/lib.rs` already exposes `#[doc(hidden)] pub mod test_harness;`, so the shared fixture layer can live under `src/test_harness/` and still be consumed by unit tests, `tests/`, and `examples/` without creating a new operator-facing API surface.
- The future shared builder must stay aligned with the authoritative managed-config model that already exists after tasks `01`-`03`:
  - explicit `pg_hba` and `pg_ident` sources
  - explicit Postgres TLS inputs
  - explicit Postgres roles and conn identities
  - no lingering generic startup assumptions based on scattered `-c key=value` style fixture thinking
- `src/config/parser.rs` is a special case and must be reviewed separately:
  - keep literal TOML fixtures explicit when the test is about parser input shape, validation field paths, or error wording
  - do not force parser-shape tests through the new generic runtime fixture layer
- The builder API must support targeted overrides without encouraging `mut`-heavy whole-struct rewrites in each test file.

### Concrete implementation shape

- Add a new shared module at `src/test_harness/runtime_config.rs`.
- Export it from `src/test_harness/mod.rs`.
- Keep it always compiled, not `#[cfg(test)]`, because `tests/` and `examples/` need to import it through `pgtuskmaster_rust::test_harness::runtime_config`.
- The module should be side-effect free:
  - no temp dirs
  - no binary discovery
  - no network binding
  - it only constructs typed config values
- Build the module around two layers:
  - small partial sample functions for reusable fragments
  - one immutable consuming builder for ergonomic full-config assembly plus overrides

### Planned builder API

- Partial sample functions:
  - `sample_cluster_config()`
  - `sample_binary_paths()`
  - `sample_postgres_roles_config()`
  - `sample_local_conn_identity()`
  - `sample_rewind_conn_identity()`
  - `sample_postgres_tls_config_disabled()`
  - `sample_pg_hba_config()`
  - `sample_pg_ident_config()`
  - `sample_postgres_logging_config()`
  - `sample_logging_config()`
  - `sample_dcs_config()`
  - `sample_ha_config()`
  - `sample_process_config()`
  - `sample_api_config()`
  - `sample_debug_config()`
  - `sample_postgres_config()` built from the smaller Postgres fragments
- One immutable builder, likely `RuntimeConfigBuilder`, with:
  - `new()` or `Default`
  - `build() -> RuntimeConfig`
  - consuming override methods for top-level sections and common high-signal leaf edits
  - section-transform methods so tests can change one nested branch without re-declaring the full config
- Minimum override surface the execution pass should provide:
  - `with_cluster_name(...)`
  - `with_member_id(...)`
  - `with_dcs_scope(...)`
  - `with_dcs_endpoints(...)`
  - `with_dcs_init(...)`
  - `with_api_listen_addr(...)`
  - `with_api_auth(...)`
  - `with_api_security(...)`
  - `with_postgres_data_dir(...)`
  - `with_postgres_connect_timeout_s(...)`
  - `with_postgres_listen_host(...)`
  - `with_postgres_listen_port(...)`
  - `with_postgres_socket_dir(...)`
  - `with_postgres_log_file(...)`
  - `with_postgres_tls(...)`
  - `with_postgres_extra_gucs(...)`
  - `with_pg_hba(...)`
  - `with_pg_ident(...)`
  - `with_logging(...)`
  - `with_process(...)`
  - `with_cluster(...)`
  - `with_postgres(...)`
  - `with_dcs(...)`
  - `with_ha(...)`
  - `with_api(...)`
  - `with_debug(...)`
- Keep the implementation purely functional:
  - prefer consuming `self` plus struct update syntax
  - avoid interior `mut`-heavy mutation sequences in tests
- Provide one obvious default entrypoint, for example:
  - `sample_runtime_config()`
  - or `RuntimeConfigBuilder::new().build()`
- File-local wrappers may remain when a test needs domain-specific parameters, but those wrappers must become thin composition layers over the shared module instead of re-declaring full `RuntimeConfig` literals.

### Baseline config contract the shared layer must enforce

- The baseline full config must already satisfy `validate_runtime_config(...)`.
- The baseline must include every managed-config-relevant input explicitly:
  - Postgres local conn identity
  - Postgres rewind conn identity
  - Postgres TLS config
  - Postgres roles
  - managed `pg_hba`
  - managed `pg_ident`
  - API security config
- The baseline must keep role and conn-identity usernames aligned:
  - `local_conn_identity.user == roles.superuser.username`
  - `rewind_conn_identity.user == roles.rewinder.username`
- The baseline must remain intentionally simple and deterministic:
  - loopback addresses
  - stable `/tmp` or `/usr/bin` style placeholder paths for unit/contract tests
  - no hidden defaults from parser-only code
- The baseline must be compatible with the authoritative managed-config path:
  - it should produce a config that the managed Postgres materialization code accepts
  - it must not encode removed backup-era or generic startup semantics

### Shared builder tests to add before migrations

- Add focused tests in `src/test_harness/runtime_config.rs` for the new builder module itself.
- Required assertions:
  - baseline builder output passes `validate_runtime_config(...)`
  - targeted overrides preserve required secure fields unless the override explicitly replaces them
  - changing data-dir/listen/api/DCS fields via builder methods only touches the intended leaves
  - section-transform methods preserve untouched sibling fields
  - the baseline works with the authoritative managed-config path, ideally by exercising the managed config materialization entrypoint instead of just checking fields structurally
- Keep these tests narrow and fast so they act as the first regression alarm when config requirements evolve again.

### Exhaustive execution checklist by file/module

- [x] `src/test_harness/runtime_config.rs`
  - Add the new partial builders plus immutable `RuntimeConfigBuilder`.
  - Add focused tests for baseline validity, override behavior, and managed-config alignment.
- [x] `src/test_harness/mod.rs`
  - Export the new `runtime_config` module.
- [x] `src/api/fallback.rs`
  - Replace the local full-literal `sample_runtime_config()` with the shared builder.
  - Keep only tiny file-local customization if the test needs a distinct debug flag or cluster/member naming.
- [x] `src/api/worker.rs`
  - Replace the local `sample_runtime_config(auth_token)` implementation with builder composition.
  - Preserve the current auth-token parameterization via shared builder overrides instead of full literal duplication.
- [x] `src/debug_api/worker.rs`
  - Replace the local `sample_runtime_config()` and keep any file-specific `sample_dcs_state(...)` helpers separate.
- [x] `src/dcs/etcd_store.rs`
  - Replace the local `sample_runtime_config(scope)` with builder composition.
  - Keep the `scope` override file-local only if it is still a thin wrapper over the shared builder.
- [x] `src/dcs/state.rs`
  - Replace the local `sample_runtime_config()` with shared builder usage.
- [x] `src/dcs/store.rs`
  - Replace the local `sample_runtime_config()` with shared builder usage.
- [x] `src/dcs/worker.rs`
  - Replace the local `sample_runtime_config()` with shared builder usage.
  - Keep cache-specific helpers separate from config construction.
- [x] `src/ha/decide.rs`
  - Replace the direct duplicated `RuntimeConfig` construction used for general HA behavior tests with builder-based setup.
  - Preserve inline literals only if a given test is explicitly about a specific full-config shape rather than generic runtime behavior.
- [x] `src/ha/events.rs`
  - Replace the local `sample_runtime_config()` with shared builder usage.
- [x] `src/ha/process_dispatch.rs`
  - Replace the local `sample_runtime_config(data_dir)` with builder composition while preserving the custom data-dir override.
  - Keep dispatch-state helpers separate.
- [x] `src/ha/worker.rs`
  - Replace the local `sample_runtime_config()` with shared builder usage.
- [x] `src/logging/mod.rs`
  - Replace the local `sample_runtime_config()` with shared builder usage.
  - Preserve test-specific logging sink overrides via immutable builder methods or struct update from builder output.
- [x] `src/logging/postgres_ingest.rs`
  - Replace the local `sample_runtime_config()` with shared builder usage.
  - Preserve the intentionally custom `pg_hba` or logging tuning only as thin overrides over the shared baseline.
- [x] `src/postgres_managed.rs`
  - Replace the local `sample_runtime_config(data_dir)` with builder composition so managed-config tests also consume the new central baseline.
- [x] `src/runtime/node.rs`
  - Replace the local baseline helper with shared builder usage.
  - Keep one-off inline `RuntimeConfig { ... }` updates only where the test is specifically asserting startup-planning behavior for changed leaves.
- [x] `src/worker_contract_tests.rs`
  - Replace the local baseline helper with shared builder usage.
  - Preserve higher-level DCS/world-state helpers separately.
- [x] `tests/bdd_api_http.rs`
  - Replace the duplicated sample runtime config helper with builder composition.
  - Preserve auth-token parameterization through shared overrides.
- [x] `examples/debug_ui_smoke_server.rs`
  - Replace the example-local full literal with the shared builder.
  - Keep example-specific listen address and any debug toggles as explicit overrides.
- [x] `src/test_harness/ha_e2e/startup.rs`
  - Review the real-binary harness config construction and migrate repeated generic runtime-config assembly to the new shared builder where it is not tightly coupled to harness-only topology facts.
  - Keep harness-only node/port/path wiring explicit, layered on top of builder output, not hidden inside the generic baseline.
- [x] `src/config/parser.rs`
  - Review only; do not mechanically migrate parser-shape tests.
  - Keep explicit TOML fixtures and explicit parser-error tests inline where that explicitness is the point of the test.
- [x] `docs/src/contributors/testing-system.md`
  - Add a short note telling contributors to start generic runtime-based tests from the shared runtime-config builder rather than copying full literals.
- [x] `docs/src/contributors/harness-internals.md` or `docs/src/contributors/codebase-map.md`
  - Document where the shared builder module lives and when to use it versus explicit parser fixtures or harness-specific topology setup.

### Migration boundaries: what must stay explicit

- Keep explicit parser TOML in `src/config/parser.rs` when testing:
  - missing fields
  - unknown fields
  - validation field names
  - user-facing config shape
- Keep explicit harness topology wiring in `src/test_harness/ha_e2e/startup.rs` when testing:
  - node-specific ports
  - generated data dirs
  - etcd member topology
  - proxy or namespace wiring
- Keep explicit one-off struct updates in a test when the changed leaf is itself the subject of the test.
- Do not preserve duplicated whole-config literals just because a file already has them; only preserve explicitness when the explicit full shape is itself the evidence.

### Planned execution order for the later `NOW EXECUTE` pass

1. Add `src/test_harness/runtime_config.rs` and export it from `src/test_harness/mod.rs`.
2. Add builder self-tests first so later migrations have a stable shared contract.
3. Migrate the managed-config proof point first:
   - `src/postgres_managed.rs`
4. Migrate crate-internal unit/contract test helpers:
   - `src/api/fallback.rs`
   - `src/api/worker.rs`
   - `src/debug_api/worker.rs`
   - `src/dcs/etcd_store.rs`
   - `src/dcs/state.rs`
   - `src/dcs/store.rs`
   - `src/dcs/worker.rs`
   - `src/ha/events.rs`
   - `src/ha/process_dispatch.rs`
   - `src/ha/worker.rs`
   - `src/logging/mod.rs`
   - `src/logging/postgres_ingest.rs`
   - `src/runtime/node.rs`
   - `src/worker_contract_tests.rs`
5. Migrate the remaining special-call-site consumers:
   - `src/ha/decide.rs`
   - `tests/bdd_api_http.rs`
   - `examples/debug_ui_smoke_server.rs`
   - `src/test_harness/ha_e2e/startup.rs` only after the rest are stable, and only by layering harness-only topology values over shared fragments or section-transform methods
6. Review `src/config/parser.rs` and intentionally leave parser-shape fixtures explicit.
7. Update contributor docs so future tests do not reintroduce literal sprawl.
8. Run targeted tests around the new shared builder and representative migrated modules.
9. Run the required full gates in this order:
   - `make check`
   - `make test`
   - `make test-long`
   - `make lint`
10. Only after every required gate passes:
   - set `<passes>true</passes>`
   - run `/bin/bash .ralph/task_switch.sh`
   - commit all changes including `.ralph` metadata
   - `git push`

### Targeted verification sequence to use during execution

- First, narrow regression checks while editing:
  - new builder module tests
  - representative API worker test
  - representative HA worker/process-dispatch test
  - representative logging test
  - `tests/bdd_api_http.rs`
- Then run the full required gates exactly as the task demands.
- If `make test-long` fails in a harness path, fix the shared builder or the harness-specific override layer rather than adding a special-case duplicate fixture back.

### Risks the mandatory `TO BE VERIFIED` pass must scrutinize

- Whether `src/test_harness/runtime_config.rs` is the best final location versus a more specific submodule name under `src/test_harness/`.
- The override API must expose general section-transform methods in addition to named leaf helpers, to avoid another wave of local wrappers.
- `src/test_harness/ha_e2e/startup.rs` should reuse shared fragments or section-transform methods while keeping topology wiring explicit instead of forcing the generic baseline builder to own harness topology concerns.
- `src/postgres_managed.rs` should be in the first migration wave because it is the best place to prove managed-config alignment before broader fixture migration.
- Whether any additional docs page beyond the contributor docs needs an update once the exact public import path is known.
- The verification pass must change at least one concrete part of this plan before replacing the marker below.

NOW EXECUTE
