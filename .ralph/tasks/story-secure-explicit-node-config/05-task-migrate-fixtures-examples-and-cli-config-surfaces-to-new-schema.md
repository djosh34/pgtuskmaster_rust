## Task: Migrate fixtures/examples/CLI config surfaces to the secure explicit schema <status>done</status> <passes>true</passes>

<description>
**Goal:** Align all config producers/consumers (tests, examples, CLI entrypoints) with the expanded schema and explicit secure requirements.

**Scope:**
- Update test fixtures under `src/` and `tests/` to provide full explicit config values.
- Update `examples/` and any contract fixture builders to compile with new config fields.
- Update CLI/config loading UX and docs to reflect new required fields and migration path.
- Add focused migration tests covering missing/invalid role auth combinations and TLS material.

**Context from research:**
- Existing fixtures rely on old defaults and partial config assumptions.
- `--all-targets` often fails if examples/fixtures are not updated in same patch.

**Expected outcome:**
- All tests/examples/config consumers build and run against one explicit secure config contract.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] All in-repo fixtures/examples compile with new required config fields populated
- [ ] CLI/config load paths provide clear guidance for missing required secure fields
- [ ] Migration tests cover invalid/missing role auth and TLS material combinations
- [ ] No lingering legacy config field usage remains in examples/tests without explicit migration rationale
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Plan (explicit secure config migration)

### Research summary (source of truth)
This repo now **fails-closed** on config security defaults:
- `config_version = "v2"` is required; `v1` is rejected.
- Security-sensitive blocks are required in v2 normalization:
  - `postgres.local_conn_identity`, `postgres.rewind_conn_identity`
  - `postgres.tls` (and `tls.mode`; identity required when mode != disabled)
  - `postgres.roles` (superuser/replicator/rewinder with `username` + `auth`)
  - `postgres.pg_hba.source`, `postgres.pg_ident.source`
  - `api.security` with `api.security.tls` + `api.security.auth`
- Cross-field invariants enforced by `validate_runtime_config`:
  - `postgres.local_conn_identity.user == postgres.roles.superuser.username`
  - `postgres.rewind_conn_identity.user == postgres.roles.rewinder.username`
  - password secrets must be non-empty (path or inline), and TLS client-auth cannot be configured when TLS is disabled.

### Exhaustive checklist (files/modules to touch)
The goal is: every in-repo config producer/consumer either:
1) builds a valid `RuntimeConfig` directly, or
2) loads/validates v2 TOML with **actionable** errors on missing secure fields.

#### RuntimeConfig literal fixtures (must be internally consistent with v2 invariants)
- [ ] `examples/debug_ui_smoke_server.rs`
  - Fix `postgres.rewind_conn_identity.user` to match `postgres.roles.rewinder.username` (currently mismatched).
  - Keep all required security blocks present (`roles`, `tls`, `pg_hba`, `pg_ident`, `api.security`).
- [ ] `tests/bdd_api_http.rs`
  - Ensure fixture remains consistent with invariants (especially conn-identity ↔ role username matching).
- [ ] `src/api/fallback.rs` (unit tests)
  - Ensure fixture remains consistent with invariants; keep this sample aligned with other helpers.
- [ ] `src/api/worker.rs` (unit tests)
  - Ensure fixture remains consistent with invariants; avoid drift vs other `sample_runtime_config` helpers.
- [ ] `src/runtime/node.rs` (tests)
  - Ensure fixture remains consistent with invariants.
- [ ] `src/worker_contract_tests.rs`
  - Ensure fixture remains consistent with invariants.
- [ ] `src/ha/decide.rs` (unit tests)
  - Ensure any directly-constructed `RuntimeConfig` literals remain consistent with invariants.
- [ ] `src/ha/worker.rs` (tests)
  - Ensure fixture remains consistent with invariants.
- [ ] `src/logging/postgres_ingest.rs` (tests)
  - Ensure fixture remains consistent with invariants.
- [ ] `src/dcs/state.rs` (tests)
  - Ensure fixture remains consistent with invariants.
- [ ] `src/dcs/worker.rs` (tests)
  - Ensure fixture remains consistent with invariants.
- [ ] `src/dcs/store.rs` (tests)
  - Ensure fixture remains consistent with invariants.
- [ ] `src/dcs/etcd_store.rs` (tests)
  - Ensure fixture remains consistent with invariants.
- [ ] `src/debug_api/worker.rs` (tests)
  - Ensure fixture remains consistent with invariants.
- [ ] `src/postgres_managed.rs` (tests)
  - Ensure helper `sample_runtime_config(..)` stays aligned with secure requirements and TLS semantics.
- [ ] `src/test_harness/ha_e2e/startup.rs`
  - Ensure the harness-built `RuntimeConfig` and any embedded `dcs.init.payload_json` remain decodable as `RuntimeConfig` JSON.
  - Ensure roles/conn identities/tls/pg_hba/pg_ident are all present and consistent.

#### BinaryPaths fixture/helpers (not RuntimeConfig literals, but part of config producers)
- [ ] `src/test_harness/binaries.rs`
  - Ensure `BinaryPaths` / `require_real_binary(..)` helpers cover required binaries (including `psql`) and do not regress `--all-targets`.

#### CLI/config loading UX (must keep errors actionable)
- [ ] `src/config/schema.rs` / `src/config/parser.rs`
  - Remove avoidable serde “missing field” parse errors for secure-v2 inputs by adding/expanding `*V2Input` wrappers with `Option<T>` leaves where appropriate.
  - Specifically target:
    - `auth = { type = "password" }` without `password = { ... }` (should become `ConfigError::Validation` with a stable `field`).
    - missing `process.binaries` (should become `ConfigError::Validation` with a stable `field`).
  - Concrete approach:
    - add a `RoleAuthConfigV2Input` wrapper so missing password materializes as `ConfigError::Validation { field: "postgres.roles.<role>.auth.password", .. }` (or equivalent stable leaf).
    - add a `ProcessConfigV2Input` wrapper with `binaries: Option<BinaryPaths>` so missing `process.binaries` becomes `ConfigError::Validation { field: "process.binaries", .. }`.
  - Keep `ConfigError::Validation { field: &'static str, ... }` actionable (stable path + remediation text).
- [ ] `tests/cli_binary.rs`
  - Add real-binary smoke assertions for config load errors:
    - missing `config_version` prints the explicit v2 migration hint.
    - a selected missing secure block/leaf prints `invalid config field '<path>'` (stable field path).

#### Migration tests (role auth + TLS material)
- [ ] `src/config/parser.rs` (tests)
  - Add **v2 TOML** load tests (not only struct-level validation) for:
    - missing `postgres.roles` block
    - missing `postgres.roles.replicator` / `rewinder`
    - missing per-role `username` / `auth`
    - conn-identity mismatch vs role username invariants
    - role auth password secret empty (inline and path)
    - TLS `mode=optional|required` with missing `identity`
    - `client_auth` configured while TLS is disabled
- [ ] `src/tls.rs` (tests)
  - Add negative tests for malformed PEM and missing identity/key/cert paths to ensure failures are deterministic and field-labelled.
  - Reuse `src/test_harness/tls.rs` fixtures where possible, but keep tests small and explicit.
- [ ] `src/postgres_managed.rs` (tests)
  - Add negative tests for TLS materialization errors (missing identity under required TLS, missing files) to complement existing happy-path tests.

#### Docs (migration path + minimal v2 template)
- [ ] Add a short mdBook page under `docs/src/operations/` describing:
  - required v2 blocks (secure vs safe defaults)
  - minimal example `toml` template (code fence must be labeled `toml`)
  - common error messages and “what to do next”
- [ ] Update `docs/src/SUMMARY.md` to link the new page (under Operations).
- [ ] (Optional) Add a brief link from `docs/src/reading-guide.md` for operator/SRE discoverability.

### Execution steps (order matters)
- [x] 0) Preflight: `make check` to surface current breakage (do not “fix by accident”; keep changes scoped).
- [x] 1) Lock down config-load UX first:
  - Add parser tests for the two known serde-leak cases:
    - missing `process.binaries`
    - password auth without `password = { ... }`
  - Implement `*V2Input` wrappers / normalization so these become `ConfigError::Validation` with stable `field` paths.
- [x] 2) Add targeted migration tests for role auth + TLS combinations (parser + tls + postgres_managed as needed).
- [x] 3) Fix `RuntimeConfig` literal fixtures (examples + unit-test helpers) to match invariants; keep fixtures consistent across modules.
- [x] 4) Add CLI integration tests that assert the **node binary** emits actionable config migration guidance on invalid configs.
- [x] 5) Add docs page + summary link; ensure `make lint` (`docs-lint`) stays green.
- [x] 6) Full verification (must be 100% green):
  - [x] `make check`
  - [x] `make test`
  - [x] `make test-long`
  - [x] `make lint`
- [x] 7) Only once all are green:
  - [x] set `<passes>true</passes>` in this task file
  - [x] `/bin/bash .ralph/task_switch.sh`
  - [x] commit: `task finished 05-task-migrate-fixtures-examples-and-cli-config-surfaces-to-new-schema: <summary + evidence>`
  - [x] `git push`
  - [x] record learnings/surprises in `AGENTS.md`

NOW EXECUTE
