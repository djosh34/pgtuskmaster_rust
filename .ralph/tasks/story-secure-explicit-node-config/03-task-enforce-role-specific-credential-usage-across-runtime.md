---
## Task: Enforce role-specific credential usage across runtime operations <status>done</status> <passes>true</passes>

<description>
**Goal:** Ensure each runtime function uses only its designated role (`superuser`, `replicator`, `rewinder`) and corresponding auth mode from config.

**Scope:**
- Trace and update credential usage in HA/process/pginfo/rewind/postgres control paths.
- Replace shared or hardcoded connection identities with explicit role-based selection.
- Ensure rewinder flows use rewinder role only, replication flows use replicator role only, and admin/system operations use superuser only.
- Add/expand type-safe interfaces preventing accidental cross-role credential use.

**Context from research:**
- Current runtime wiring includes hardcoded/default connection identity assumptions and does not model all role auth pathways explicitly.
- This creates risk of privilege bleed across operations.

**Expected outcome:**
- Runtime operations are least-privilege by construction and enforced in code paths.
- Regression tests fail on any role-misuse or fallback identity usage.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] All PostgreSQL connection builders select credentials via explicit role-kind mapping from config
- [x] Rewind implementation uses rewinder role/auth only
- [x] Replication/follow operations use replicator role/auth only
- [x] Superuser-only actions are isolated and do not fallback to implicit defaults
- [x] Tests cover role misuse prevention and least-privilege mapping behavior
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make test-long` — passes cleanly
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

---

## Research notes (current behavior / problems)

- Roles exist in config (`postgres.roles.{superuser,replicator,rewinder}` + `RoleAuthConfig`) but are not consistently *used* by runtime operations.
- Separate `postgres.{local_conn_identity, rewind_conn_identity}` carry `user/dbname/ssl_mode` and can silently diverge from role usernames.
- `ProcessDispatchDefaults::contract_stub()` hardcodes remote-source conninfo to `user=postgres` / `dbname=postgres` and runtime only overwrites `host/port`:
  - Startup clone/basebackup (`Runtime::StartupMode::CloneReplica`) inherits the stub `user=postgres` through `default_leader_conninfo`.
  - HA rewind (`HaAction::StartRewind`) also inherits the stub `user=postgres`.
- Process commands currently accept raw/untagged `PgConnInfo` and do not prevent role-misuse:
  - `pg_basebackup -U <user>` is taken directly from `BaseBackupSpec.source_conninfo.user`
  - `pg_rewind --source-server <conninfo>` is rendered directly from `PgConnInfo`
- `RoleAuthConfig` is parsed/validated but not consumed by the process runner (no env/secret plumbing), and local SQL polling DSN ignores roles entirely.

## Full plan (detailed)

### 0) Define the runtime contract: which role is used where

- **Local SQL polling (pginfo)**: `superuser` role only.
- **Bootstrap/initdb**: `superuser` role only (`initdb -U <superuser>`; do not hardcode `postgres`).
- **Replica clone / basebackup**: `replicator` role only.
- **Rewind / pg_rewind**: `rewinder` role only.
- **Anything else (start/stop/promote/demote/fencing)**: no DB credentials; keep unchanged, but ensure no accidental conninfo reuse.

### 0.1) Non-obvious but critical dependency: roles must exist on the *source* server

- `pg_basebackup` connects to the source server as the configured `replicator` username; Postgres will reject the connection if that role does not exist on the source, even with `pg_hba` set to `trust`.
- `pg_rewind` connects to the source server as the configured `rewinder` username; the role must exist and have whatever privileges `pg_rewind` needs (likely more than plain `REPLICATION`).
- Production expectation: roles are provisioned out-of-band by an operator.
- Test expectation: HA e2e harness must ensure the `replicator` role exists on the elected primary before attempting to start a clone that runs `pg_basebackup` (otherwise `make test-long` will fail).

### 1) Add role-kind + typed connection specs (prevent accidental cross-role use)

- Introduce an explicit `PostgresRoleKind` enum (`Superuser | Replicator | Rewinder`) in a shared module (likely `src/pginfo/state.rs` or a new small `src/postgres/roles.rs`).
- Add *type-safe wrappers* for operational connection specs (compile-time separation) while avoiding secret leakage in `Debug`:
  - Prefer newtypes around existing primitives (`PgConnInfo`, `String`) rather than redesigning everything:
    - `SuperuserLocalDsn(String)` (never includes secrets)
    - `ReplicatorSourceConn` (host/port/user/dbname/ssl_mode + `RoleAuthConfig`, **keeping secrets as `SecretSource`**)
    - `RewinderSourceConn` (same shape as above)
- Each wrapper must be constructible only via a single builder that takes `&RuntimeConfig` and selects the correct role + auth:
  - `build_superuser_local_dsn(cfg, process_defaults)` (pginfo; uses `roles.superuser.username`)
  - `build_replicator_basebackup_source(cfg, host, port)` (clone; uses `roles.replicator.username`)
  - `build_rewinder_rewind_source(cfg, host, port)` (rewind; uses `roles.rewinder.username`)

### 2) Make runtime defaults explicit (remove hidden “postgres” fallbacks)

**Files/modules checklist (must be exhaustive):**

- `src/ha/state.rs`
  - Split `ProcessDispatchDefaults.rewind_source_conninfo` into two separate remote sources:
    - `basebackup_source_conn` (replicator-only)
    - `rewind_source_conn` (rewinder-only)
  - Keep `ProcessDispatchDefaults::contract_stub()` for tests, but ensure **production code no longer builds defaults by “mutate the stub”** (so stub values can never leak into runtime behavior).

- `src/runtime/node.rs`
  - Replace the “partial overwrite” pattern in `process_defaults_from_config` with **construct-from-scratch** population of:
    - postgres host/port/socket_dir/log_file/shutdown_mode
    - both remote source conn wrappers from Step 1 (no inherited stub fields).
  - Update `default_leader_conninfo` so startup clone/basebackup derives from `basebackup_source_conn` only.
  - Update local pginfo DSN builder so it uses `cfg.postgres.roles.superuser.username` (not `local_conn_identity.user`), while still using `local_conn_identity.{dbname,ssl_mode}`.

- `src/ha/worker.rs`
  - Update `dispatch_actions`:
    - `HaAction::StartRewind` must use `rewind_source_conn` only.
    - `HaAction::RunBootstrap` must supply explicit superuser username into `BootstrapSpec` (no hardcoded `postgres`).

### 3) Enforce config invariants (fail closed on mismatches)

- `src/config/parser.rs` (`validate_runtime_config`)
  - Add cross-field validation preventing identity/role drift:
    - `postgres.local_conn_identity.user` must equal `postgres.roles.superuser.username`
    - `postgres.rewind_conn_identity.user` must equal `postgres.roles.rewinder.username`
  - Even though runtime will start selecting users from roles, keep these validations so misconfigurations are caught early and loudly (and because multiple fixtures currently drift here).

### 4) Plumb auth mode into execution paths (use role’s `RoleAuthConfig`)

- Add a small “resolved auth” helper to turn `RoleAuthConfig` into runtime-ready inputs **without ever storing cleartext secrets in `Debug` structs**:
  - Model env vars as `SecretSource` until the last moment.
  - In `TokioCommandRunner::spawn`, resolve `SecretSource` (inline or file) to a string, apply it via `Command::env`, and then drop it.
  - TLS: currently has no per-role client material in config; treat as “no password env” for now, but keep auth-mode selection explicit in types so future tasks can add cert/key plumbing.

**Process worker changes:**

- `src/process/jobs.rs`
  - Change job specs to carry the typed conn/auth wrapper instead of raw `PgConnInfo`:
    - `BaseBackupSpec.source_*` becomes `ReplicatorSourceConn`
    - `PgRewindSpec.source_*` becomes `RewinderSourceConn`
    - `BootstrapSpec` gains `superuser_username` (or a dedicated wrapper) so `initdb -U` uses the configured superuser.
  - Ensure `PgRewindSpec` validates required source fields (host/user) either via typed constructors or explicit validation (it currently validates only `target_data_dir`).

- `src/process/worker.rs`
  - Extend `ProcessCommandSpec` to carry environment variables **without secret leakage**, e.g. `Vec<ProcessEnvVar>` where the value is `SecretSource` or a redacted wrapper.
  - In `TokioCommandRunner::spawn`, resolve+apply env vars when spawning commands.
  - In `build_command`:
    - `Bootstrap`: use configured superuser username (no hardcoded `postgres`).
    - `BaseBackup`: take user from `ReplicatorSourceConn` only; if password auth, set `PGPASSWORD` env (from `SecretSource`).
    - `PgRewind`: render conninfo from `RewinderSourceConn` only; if password auth, set `PGPASSWORD` env (from `SecretSource`).

### 5) Tests: make role-misuse impossible to miss

- Update fixtures to use **distinct usernames** (never all `postgres`) for any role-mapping test; this makes wrong wiring fail deterministically.

**Targeted unit tests to add/extend:**

- `src/runtime/node.rs` (unit tests)
  - Add a test where:
    - `roles.superuser.username = "su_admin"`
    - `roles.replicator.username = "repl_user"`
    - `roles.rewinder.username = "rewind_user"`
  - Assert:
    - local pginfo DSN uses `su_admin`
    - clone/basebackup source uses `repl_user`
    - rewind source uses `rewind_user`
    - no path falls back to stub `"postgres"`

- `src/ha/worker.rs` (unit tests)
  - Add a `dispatch_actions` test ensuring `HaAction::StartRewind` emits `ProcessJobKind::PgRewind` with rewinder username only.

- `src/process/worker.rs` (unit tests)
  - Extend basebackup command test coverage to ensure the user is exactly the replicator user from the typed wrapper.
  - Add a pg_rewind command test verifying the rendered `--source-server` includes the rewinder username and not others.
  - Add a password-auth test: for password mode, ensure `ProcessCommandSpec` carries `PGPASSWORD` env and the password does **not** appear in args.
  - Add a bootstrap test: `initdb -U` uses `BootstrapSpec.superuser_username`.

**Integration/BDD fixture updates (minimal but necessary):**

- Update **all** in-repo `RuntimeConfig` fixtures that currently violate the new validation (this list is exhaustive from `rg`):
  - `src/api/fallback.rs`
  - `src/api/worker.rs`
  - `src/dcs/etcd_store.rs`
  - `src/dcs/state.rs`
  - `src/dcs/store.rs`
  - `src/dcs/worker.rs`
  - `src/debug_api/worker.rs`
  - `src/ha/decide.rs`
  - `src/ha/worker.rs`
  - `src/logging/postgres_ingest.rs`
  - `src/runtime/node.rs`
  - `src/test_harness/ha_e2e/startup.rs`
  - `src/worker_contract_tests.rs`
  - `tests/bdd_api_http.rs`
  - Ensure identity/role alignment per Step 3.
- HA e2e harness:
  - Update `src/test_harness/ha_e2e/util.rs` so `run_psql_statement` can connect using the configured superuser identity (not hardcoded `postgres/postgres`).
  - Add a harness bootstrap step (in `src/test_harness/ha_e2e/startup.rs` or cluster setup) that ensures the `replicator` role exists on the elected primary (so clone/basebackup with `-U <replicator>` succeeds).
  - Keep distinct usernames primarily in unit tests; keep e2e runtime config conservative if needed to avoid requiring rewinder super-privileges in harness.

### 6) Final verification gates (must be 100% green)

- Run:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`

COMPLETED (2026-03-04)
