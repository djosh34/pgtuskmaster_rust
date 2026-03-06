## Task: Centralize composable sample runtime-config builders for tests and helpers <status>not_started</status> <passes>false</passes> <priority>high</priority>

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
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Add one central test-support module for composable runtime-config construction
- [ ] Shared builder API is composed from partial sample functions or typed builder parts rather than one giant monolithic full-config literal only
- [ ] Duplicate sample-config helpers and repeated full `RuntimeConfig` literals are migrated where the test is not specifically about parser input shape
- [ ] Parser/config-shape tests that need literal inline TOML or explicit field-by-field configs remain explicit and are not forced through the generic sample builder
- [ ] The shared builder layer is aligned with the authoritative managed-config model and does not preserve generic `-c key=value` startup assumptions
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

### Exhaustive checklist (must be treated as part of acceptance)

- [ ] Add one shared test-support module, likely under `src/test_harness/` or a dedicated crate-internal test support area
  - [ ] Provide small partial sample functions for common config fragments:
  - [ ] cluster identity
  - [ ] postgres core paths/listen settings
  - [ ] postgres roles/auth
  - [ ] postgres TLS inputs
  - [ ] postgres managed HBA/ident inputs
  - [ ] DCS settings
  - [ ] API security settings
  - [ ] logging/process/debug defaults
  - [ ] Provide one easy composition path for a “valid full sample runtime config” plus targeted overrides.
- [ ] Migrate duplicated sample config helpers and/or repeated `RuntimeConfig` literals in:
  - [ ] `src/api/fallback.rs`
  - [ ] `src/api/worker.rs`
  - [ ] `src/debug_api/worker.rs`
  - [ ] `src/dcs/etcd_store.rs`
  - [ ] `src/dcs/state.rs`
  - [ ] `src/dcs/store.rs`
  - [ ] `src/dcs/worker.rs`
  - [ ] `src/ha/decide.rs`
  - [ ] `src/ha/events.rs`
  - [ ] `src/ha/process_dispatch.rs`
  - [ ] `src/ha/worker.rs`
  - [ ] `src/logging/mod.rs`
  - [ ] `src/logging/postgres_ingest.rs`
  - [ ] `src/runtime/node.rs`
  - [ ] `src/worker_contract_tests.rs`
  - [ ] `tests/bdd_api_http.rs`
  - [ ] `examples/debug_ui_smoke_server.rs`
- [ ] Review `src/config/parser.rs` tests separately
  - [ ] Keep literal TOML fixtures and exact parser-error tests explicit where they are intentionally testing parser shape rather than generic runtime behavior.
- [ ] Add focused tests for the shared builder layer
  - [ ] Assert it produces a valid baseline config.
  - [ ] Assert targeted overrides do not accidentally drop required secure fields.
  - [ ] Assert builder parts remain aligned with the authoritative managed-config contract.
