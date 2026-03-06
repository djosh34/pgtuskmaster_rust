## Task: Wire HTTP/PG TLS, pg_hba/pg_ident, and DCS init config into startup orchestration <status>done</status> <passes>true</passes>

<description>
**Goal:** Make startup consume the expanded config end-to-end so node boot requires explicit secure config and does not infer missing values.

**Scope:**
- Update runtime/process/startup orchestration to consume new HTTP and PostgreSQL TLS cert/key settings.
- Ensure pg_hba and pg_ident config fields are materialized correctly during bootstrap/start.
- Integrate DCS init config field(s) into initialization logic so bootstrap writes are config-driven.
- Confirm HTTP server has complete explicit config fields (listen, auth, TLS policy) and no hidden fallback.

**Context from research:**
- Some startup behavior currently derives values from defaults and legacy fallbacks.
- Secure deterministic startup requires explicit runtime wiring across process worker, API worker, and DCS bootstrapping paths.

**Expected outcome:**
- A node can only start when complete secure config is present, and all startup side effects follow config directly.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] Startup path consumes explicit HTTP TLS cert/key and auth config with no implicit fallback
- [x] Startup path consumes explicit PostgreSQL TLS/hosting/auth config with no implicit fallback identities
- [x] pg_hba/pg_ident config fields are written/applied deterministically during startup lifecycle
- [x] DCS init config is explicitly sourced from config and used during bootstrap/init writes
- [x] Real-binary and integration tests validate startup with explicit config only
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make test-long` — passes cleanly
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

---

## Research summary (what is missing today)

- Runtime startup (`src/runtime/node.rs`) runs initdb/basebackup + `pg_ctl start`, but does not materialize or apply:
  - `postgres.tls` (server SSL settings + cert/key/ca),
  - `postgres.pg_hba` / `postgres.pg_ident` (files + pointers),
  - `dcs.init` (bootstrap/init writes).
- Runtime workers (`src/runtime/node.rs`) bind the API listener and build `ApiWorkerCtx`, but do not wire:
  - `api.security.tls` into a `rustls::ServerConfig` + `TlsAcceptor`,
  - `api.security.tls.client_auth` / `require_client_cert` into mTLS behavior.
- Process layer (`src/process/jobs.rs`, `src/process/worker.rs`) lacks a way to pass additional postgres `-c key=value` settings through `pg_ctl -o`, so even after materializing files there’s no route to apply them.

The goal of this task is to wire all of those config surfaces end-to-end so startup is deterministic and security posture is not silently ignored.

---

## Full plan (write plan first; then verify; then execute)

### 0) Exhaustive file/module checklist (what to touch + why)

- [x] `src/runtime/node.rs` — startup orchestration
  - [x] Add deterministic “materialize managed config” step for PostgreSQL (TLS material + pg_hba/pg_ident) before any `StartPostgres`.
  - [x] Add deterministic “wire API TLS” step before running `api::worker::run`.
  - [x] Add DCS init/bootstrap write step in the `InitializePrimary` startup path (claim `/<scope>/init` always; seed `/<scope>/config` when `cfg.dcs.init.write_on_bootstrap=true`).
- [x] `src/process/jobs.rs` — job specs
  - [x] Extend `StartPostgresSpec` (and `RestartPostgresSpec` if used in HA) with extra postgres settings/opts.
- [x] `src/process/worker.rs` — command builder
  - [x] Append extra `-c key=value` settings into the `pg_ctl -o` payload for `StartPostgres` and `RestartPostgres`.
  - [x] Add unit tests ensuring the command includes the extra settings deterministically.
- [x] `src/ha/worker.rs` — HA action dispatch
  - [x] Ensure HA-triggered start/restart paths also apply the same managed postgres config (same behavior as runtime startup).
  - [x] Ensure any new fields added to specs/defaults are threaded through HA dispatch without regressions.
- [ ] `src/ha/state.rs` — dispatch defaults (if needed)
  - [ ] Only if required for ergonomics: add defaults for extra postgres settings or keep them computed from `RuntimeConfig` at dispatch-time.
- [x] `src/config/parser.rs` — validation tightening (fail fast + actionable errors)
  - [x] Validate API TLS client-auth (`client_ca`, `require_client_cert`) coherence.
  - [x] Validate Postgres TLS client-auth coherence.
  - [x] Validate `dcs.init` payload requirements.
  - [x] Decide and enforce policy for `postgres.pg_hba.source` and `postgres.pg_ident.source` empty inline content.
- [x] `src/dcs/store.rs` and/or `src/dcs/etcd_store.rs` — DCS init writes
  - [x] Add a small explicit etcd txn primitive (`put-if-absent`) and use it for both `/<scope>/init` and `/<scope>/config` seeding.
  - [x] Avoids one-shot reads by using etcd transactions (`version == 0` compare) to prevent overwrites.
- [x] New small helper module(s) (names to be chosen during implementation; keep minimal)
  - [x] InlineOrPath loader (read bytes/string from `InlineOrPath::Path|PathConfig|Inline` with good errors; never `unwrap`/`expect`/`panic`).
  - [x] Atomic file writer + permissions helper (for TLS private keys).
- [x] Real-binary / integration tests
  - [x] Add production TLS-builder coverage (API TLS + mTLS tests build rustls config via `crate::tls::build_rustls_server_config`).
  - [x] Add real postgres startup checks proving managed `hba_file`/`ident_file` are applied (asserted via `SHOW hba_file`/`SHOW ident_file` in HA e2e harness).
  - [x] Add etcd-backed test proving `dcs.init` writes the configured payload during bootstrap (asserts `/<scope>/init` and `/<scope>/config`).
  - [x] Update fixtures/examples that previously used empty pg_hba/pg_ident content (now rejected by validation).

### 1) API: Wire HTTP TLS + auth into worker startup (no silent ignore)

- [ ] Add production TLS material loading (no test-harness-only helpers) for:
  - [ ] `TlsServerIdentityConfig.cert_chain` (PEM),
  - [ ] `TlsServerIdentityConfig.private_key` (PEM),
  - [ ] optional `TlsClientAuthConfig.client_ca` (PEM).
- [ ] Build `rustls::ServerConfig` from `cfg.api.security.tls`:
  - [ ] `mode=disabled` → do not configure acceptor.
  - [ ] `mode=optional|required` → require identity material; build acceptor; fail fast if build fails.
  - [ ] if `client_auth` is configured:
    - [ ] validate CA is present/non-empty,
    - [ ] configure rustls client verifier appropriately,
    - [ ] wire `require_client_cert` into `ApiWorkerCtx::set_require_client_cert`.
  - [ ] Ensure a rustls crypto provider is installed if required by current rustls version; handle installation errors explicitly.
- [ ] In `src/runtime/node.rs` after creating `api_ctx`:
  - [ ] call `api_ctx.configure_tls(cfg.api.security.tls.mode, server_cfg_opt)` (always set override to match config),
  - [ ] call `api_ctx.set_require_client_cert(...)`.
- [ ] Config validation (`src/config/parser.rs`):
  - [ ] If `cfg.api.security.tls.client_auth` is present, validate `client_ca` via `validate_inline_or_path_non_empty(...)`.
  - [ ] Reject `client_auth` when `tls.mode=disabled` (parsed-but-not-actionable config must be an error).
  - [ ] If `require_client_cert=true`, ensure TLS mode is not disabled and `client_ca` is configured.
  - [ ] Ensure error fields are specific (`api.security.tls.client_auth.client_ca`, etc).
 - [ ] Tests must prove **runtime wiring** (not only `ApiWorkerCtx` unit setup):
  - [ ] Add runtime/startup test coverage that enters via `src/runtime/node.rs` (or a thin worker-runner wrapper) and verifies:
    - [ ] `mode=required` → TLS works and plain HTTP fails.
    - [ ] invalid/missing PEM/path → runtime startup fails loudly with the correct config field in the error.
    - [ ] mTLS required + trusted client cert succeeds; missing/untrusted cert fails.

### 2) Postgres: Materialize + apply TLS, pg_hba, pg_ident deterministically

**Design target:** Postgres must always start with explicit managed paths and TLS posture derived from config, without relying on whatever `initdb`/`pg_basebackup` produced.

- [ ] Introduce a single “managed postgres config” builder used by:
  - [ ] runtime startup (`execute_startup`),
  - [ ] HA start/restart actions (`ha/worker.rs` dispatch).
- [ ] Managed file layout (deterministic, owned by pgtuskmaster):
  - [ ] under `postgres.data_dir`, write:
    - [ ] `pgtm.pg_hba.conf` (from `cfg.postgres.pg_hba.source`),
    - [ ] `pgtm.pg_ident.conf` (from `cfg.postgres.pg_ident.source`),
    - [ ] if TLS enabled:
      - [ ] `pgtm.server.crt` (cert chain),
      - [ ] `pgtm.server.key` (private key; chmod `0600` on unix),
      - [ ] optional `pgtm.ca.crt` (client CA).
  - [ ] use atomic write pattern (`tmp` + rename) so partial writes don’t corrupt startup.
- [ ] Build extra `postgres -c` settings to be applied at start time:
  - [ ] Always set:
    - [ ] `hba_file='<abs path to pgtm.pg_hba.conf>'`
    - [ ] `ident_file='<abs path to pgtm.pg_ident.conf>'`
  - [ ] TLS:
    - [ ] `mode=disabled` → `ssl=off`
    - [ ] `mode=optional|required` → `ssl=on` + `ssl_cert_file='<abs path>'` + `ssl_key_file='<abs path>'`
    - [ ] if `client_auth` configured → `ssl_ca_file='<abs path>'`
  - [ ] `pg_ctl -o` rendering contract (escaping at the correct layer):
    - [ ] Render **argv tokens** first (`-h`, host, `-p`, port, `-k`, socket, repeated `-c`, `key=value`) and then encode into the single `pg_ctl -o` string.
    - [ ] Escape tokens for `pg_ctl` option-string parsing (not SQL/GUC literal rules).
    - [ ] Validate setting keys (`[A-Za-z0-9_.-]+`) to prevent option injection via malformed keys.
    - [ ] Ensure deterministic ordering + duplicate-key policy (recommended: validated `BTreeMap<String,String>`).
- [ ] Plumb start settings through the process layer:
  - [ ] Add `extra_postgres_settings: Vec<(String,String)>` or `Vec<String>` to `StartPostgresSpec`.
  - [ ] Update `build_command` in `src/process/worker.rs` to append settings into the `pg_ctl -o` string as `-c key=value`.
  - [ ] Repeat for `RestartPostgresSpec` if HA uses restart.
- [ ] Apply in runtime startup (`src/runtime/node.rs`):
  - [ ] `InitializePrimary`: after `Bootstrap` job finishes (initdb done), materialize managed files, then `StartPostgres` with extra settings.
  - [ ] `CloneReplica`: after `BaseBackup` job finishes (data dir exists), materialize managed files, then `StartPostgres` with extra settings.
  - [ ] `ResumeExisting`:
    - [ ] if `postmaster.pid` exists, keep current “do nothing” (do not mutate a running instance in this task).
    - [ ] if not running, materialize managed files and start with extra settings.
- [ ] Apply in HA worker (`src/ha/worker.rs`):
  - [ ] Before dispatching Start/Restart actions, call the same materializer and include extra settings in the job spec.
  - [ ] Keep the materializer idempotent so repeated HA ticks don’t create nondeterminism.
- [ ] Config validation (`src/config/parser.rs`):
  - [ ] Postgres TLS: if `client_auth` is present, validate `client_ca` non-empty.
  - [ ] Decide policy for empty inline `pg_hba`/`pg_ident`:
    - [ ] Recommended: reject empty/whitespace inline content (otherwise we can accidentally boot into “no access” state that breaks internal health checks).
    - [ ] If we reject empty, update *all* tests/fixtures/examples that currently use empty strings.

### 3) DCS: Use `dcs.init` during bootstrap/init writes

**Correctness-first contract (must be safe under races):**
- `/init` is the authoritative “cluster has been initialized” marker and must be write-once.
- `dcs.init.payload_json` is used to seed `/<scope>/config` only during the very first bootstrap, and must not overwrite an existing value.

- [ ] Config validation (`src/config/parser.rs`):
  - [ ] If `cfg.dcs.init.write_on_bootstrap=true`:
    - [ ] reject empty/whitespace `payload_json`,
    - [ ] require it to be valid JSON,
    - [ ] and (recommended) require it to deserialize as `RuntimeConfig` to match the existing `DcsKey::Config` decoder.
- [ ] Startup mode selection must honor init lock (`src/runtime/node.rs`):
  - [ ] If DCS cache indicates `init_lock` exists, **must not** choose `InitializePrimary` even if leader is missing.
  - [ ] Add unit tests for `select_startup_mode` to cover `init_lock` scenarios (no timing/async flake).
- [ ] Startup execution (`src/runtime/node.rs`):
  - [ ] Before executing `StartupMode::InitializePrimary`, atomically claim `/<scope>/init` (write-once).
  - [ ] On claim success, proceed with bootstrap.
  - [ ] On claim conflict, abort InitializePrimary and re-plan (or fail with a clear “already initialized” error).
  - [ ] If `write_on_bootstrap=true`, atomically seed `/<scope>/config` **only if absent**.
- [ ] Shared DCS helper / store API (must support CAS/txn):
  - [ ] Add a minimal transactional write-once primitive (e.g. `put_if_absent(path, value) -> wrote_bool`) for etcd-backed store.
  - [ ] Use this primitive for both `/init` and `/config` seeding to avoid race windows.

### 4) Tests (must prove the wiring, not just unit behavior)

#### 4.1 API TLS wiring tests (integration-level)

- [ ] Add a runtime-level test that starts the node workers with:
  - [ ] `api.security.tls.mode=required` and valid cert/key → TLS request succeeds, plain HTTP fails.
  - [ ] `mode=optional` and valid cert/key → both plain and TLS succeed.
  - [ ] mTLS required + trusted client cert → succeeds; missing/untrusted cert → fails.
- [ ] Add negative test: `mode=required` + invalid/missing PEM → runtime startup fails loudly (error mentions the config field).

#### 4.2 Postgres managed config tests (real-binary)

- [ ] In real-binary harness tests, bootstrap/start Postgres and assert:
  - [ ] `pgtm.pg_hba.conf` / `pgtm.pg_ident.conf` are created and contain the provided content.
  - [ ] Postgres was started with the managed `hba_file`/`ident_file` (verify via `SHOW hba_file;` and `SHOW ident_file;`).
  - [ ] TLS mode is applied (verify via `SHOW ssl;` and, for enabled TLS, that connections behave as dictated by the provided HBA).

#### 4.3 DCS init tests (real etcd)

- [ ] Prefer transaction-result assertions over watch/poll timing:
  - [ ] Two clients race to claim `/<scope>/init` → exactly one “claimed”, one “already initialized”.
  - [ ] Seed `/<scope>/config` when absent, and verify it is not overwritten when pre-existing.
  - [ ] Run the `InitializePrimary` startup path with `dcs.init.write_on_bootstrap=true`, then verify via etcd `get` that:
    - [ ] `/<scope>/init` exists and matches the expected JSON shape,
    - [ ] `/<scope>/config` equals `payload_json`.

### 5) Validation gate (definition of done)

- [x] `make check`
- [x] `make test`
- [x] `make test-long`
- [x] `make lint`

NOW EXECUTE
