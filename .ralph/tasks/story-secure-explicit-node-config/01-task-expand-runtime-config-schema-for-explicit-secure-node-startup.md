## Task: Expand runtime config schema for explicit secure node startup <status>done</status> <passes>true</passes>

<description>
**Goal:** Redesign the runtime config model so every required secure startup setting is explicitly represented (TLS, HTTP, PostgreSQL hosting, roles/auth, pg_hba/pg_ident, and DCS init config).

**Scope:**
- Expand `src/config/mod.rs` and `src/config/schema.rs` with strongly typed fields for:
- PostgreSQL TLS server identity and client auth material.
- HTTP server TLS identity and client-facing auth wiring.
- PostgreSQL hosting/listen/datadir/socket/replication/bootstrap-relevant fields (including any currently inferred fields).
- Role list structure with required role kinds (`superuser`, `replicator`, `rewinder`), each carrying username plus enum auth (`tls` or `password`).
- `pg_hba` and `pg_ident` file-content/path fields as explicit config.
- DCS init config payload field(s) required for bootstrapping cluster defaults.
- Remove implicit fallback semantics from model shape by making inference impossible at type level.

**Context from research:**
- Current config relies on inferred defaults in multiple sections (`defaults.rs`, parser fallback, runtime assumptions like postgres user).
- Roles are not currently modeled as explicit typed list with role-specific auth semantics.
- TLS is currently represented by simple enable/disable and lacks explicit cert/key surfaces for both Postgres and HTTP in one canonical config contract.

**Expected outcome:**
- The config schema can fully describe a safe node startup without hidden runtime assumptions.
- All required secure runtime inputs are represented by explicit typed fields and enums.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] `src/config/mod.rs` includes complete strongly typed runtime config structures for HTTP TLS, Postgres TLS, role list/auth enum, pg_hba/pg_ident, and DCS init config
- [x] `src/config/schema.rs` includes matching partial/serde schema types and compatibility strategy for parsing
- [x] Required roles (`superuser`, `replicator`, `rewinder`) are represented explicitly and unambiguously in the schema
- [x] Role auth is enum-typed (`tls` | `password`) with explicit fields for each mode
- [x] No type-level path remains for implicitly inferred default postgres identity fields
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

---

## Plan (explicit secure runtime config redesign)

### 0) Non-negotiables / constraints (do not violate)

- No panics / unwraps / expects anywhere (repo lint policy).
- Secure-startup-relevant values must flow from config types; runtime must not “invent” credentials or TLS material.
- Keep `#[serde(deny_unknown_fields)]` discipline for each schema epoch; do not weaken parsing globally just to “make v2 fit”.
- This task is the **schema/model + minimal identity plumbing** foundation. Actual side effects are explicitly deferred:
  - Subprocess credential env wiring (`initdb`/`pg_basebackup`/`pg_rewind`) is task `03`.
  - Writing `pg_hba.conf` / `pg_ident.conf` and Postgres TLS server flagging is task `04`.
- Keep `make test-long` stable: do not force real-binary tests to switch away from current `trust` bootstrap or `postgres` identity in this task; instead, preserve legacy behavior in the v1 compatibility path.

### 1) Ground truth (current pipeline + why it matters)

Current parse flow is effectively:

- `TOML -> PartialRuntimeConfig (Options) -> apply_defaults -> validate_runtime_config -> RuntimeConfig`

The important implications for this refactor:

- Most inference is concentrated in `src/config/defaults.rs` (`apply_defaults`).
- Validation happens *after* defaults, so “required fields” can still be optional at parse level for v1.
- Many tests/examples construct `RuntimeConfig` directly (struct literals) and will break as soon as we add required fields to runtime structs.

### 2) Compatibility strategy (must decide first)

**Goal:** add an explicit v2 schema without breaking existing v1 configs/tests, and without silently accepting v2 without semantics.

**Decision:** introduce `ConfigVersion` and version-gated parsing with strict `deny_unknown_fields` per version.

Implementation approach (preferred for correctness, and keeps unknown-key errors as parse errors):

1. Read TOML as a string (already done today).
2. Parse a tiny strict envelope struct from the same string:
   - `ConfigEnvelope { config_version: Option<ConfigVersion> }`
   - default missing `config_version` to v1
3. Parse *again* from the same string into the strict versioned schema:
   - v1 input struct (legacy keys, permissive enough for current configs), or
   - v2 input struct (explicit secure surfaces, strict `deny_unknown_fields`).

This avoids a single mega-struct that accidentally allows v2-only keys in v1, and keeps strictness meaningful.

**Fail-closed policy for v2 in task 01:** until task `02` implements the actual v2 normalization semantics, `config_version = v2` still parses into the v2 schema (so unknown keys are caught), and then returns a clear, actionable validation error (do not start).

### 3) New strongly-typed model surface (v2 types, but used by runtime structs where safe)

This task introduces the *types* for explicit secure startup. Not all of them will be “executed” yet, but the type surface becomes canonical and re-exported.

#### 3.1 Shared primitives (reusable across API + Postgres + secrets)

- `ConfigVersion` enum (e.g. `V1`, `V2`) with serde rename rules.
- `InlineOrPath`:
  - `Path { path: PathBuf }`
  - `Inline { content: String }`
- `SecretSource` (wrapper/newtype around `InlineOrPath` with redacted `Debug`/`Display` behavior; no logs of inline content).

#### 3.2 TLS primitives (API + Postgres)

- **Skeptical adjustment:** do not introduce multiple parallel TLS-mode enums. There is already an `ApiTlsMode` enum in the API worker; move/relocate it into `src/config/*` (so it’s shared), then use it as the canonical TLS mode for API config.
- For Postgres TLS, either reuse the same enum (preferred) or introduce a separate `PostgresTlsMode` only if a clear semantic split emerges later.
- `TlsServerIdentityConfig`: `cert_chain: InlineOrPath`, `private_key: InlineOrPath`
- `TlsClientAuthConfig`: `client_ca: InlineOrPath`, `require_client_cert: bool`
- `TlsServerConfig`:
  - `mode: ApiTlsMode` (or the shared canonical TLS-mode enum)
  - `identity: Option<TlsServerIdentityConfig>`
  - `client_auth: Option<TlsClientAuthConfig>`

Validation rules (added in v2 validation):
- if `mode != disabled`, identity must exist
- if `client_auth.require_client_cert == true`, client_auth must exist

#### 3.3 API config (collapse legacy token split)

New canonical shape:

- `ApiAuthConfig`:
  - `disabled`
  - `role_tokens { read_token, admin_token }`
- `ApiSecurityConfig`: `tls: TlsServerConfig`, `auth: ApiAuthConfig`
- `ApiConfig`: `listen_addr: String`, `security: ApiSecurityConfig`

v1 compatibility mapping (in v1 normalization only):
- legacy `api.read_auth_token` / `api.admin_auth_token` become `ApiAuthConfig::role_tokens`.
- legacy `security.auth_token` (single token) becomes both read+admin tokens (preserve current “one token works everywhere” behavior).
- legacy `security.tls_enabled` maps to `ApiSecurityConfig.tls.mode` (likely `required` when true, `disabled` when false).

#### 3.4 Postgres hosting, identity surfaces, roles/auth, hba/ident, TLS

New structures to add (as v2 schema types, and partially as runtime fields where needed to remove hidden defaults):

- Hosting/bind/paths:
  - Keep current runtime fields for data_dir/listen/socket/log in `PostgresConfig`, but introduce nested groupings in v2 input schema so “explicitness” is possible without changing runtime semantics yet.
- Conninfo identity (this task *does* plumb identity explicitly to remove hidden hardcoded defaults):
  - Reuse existing `PgSslMode` from `src/pginfo/conninfo.rs` (do not create a duplicate enum)
  - `PostgresConnIdentityConfig`: `user: String`, `dbname: String`, `ssl_mode: PgSslMode`
  - Add runtime fields for:
    - local probe DSN identity (replacing `user=postgres dbname=postgres` hardcode)
    - rewind source identity (replacing implicit `contract_stub()` values)
- Roles:
  - `RoleKind`: `superuser | replicator | rewinder`
  - `RoleAuthConfig`: `tls { ... } | password { password: SecretSource }`
  - `PostgresRoleConfig` and `PostgresRolesConfig` (required in v2 input; v1 fills with current `postgres` identity defaults for now).
- `pg_hba` / `pg_ident`:
  - `PgHbaConfig { source: InlineOrPath }`
  - `PgIdentConfig { source: InlineOrPath }`
  - Present in v2 schema; actual file writing deferred to task `04`.
- DCS init payload:
  - `DcsInitConfig { payload_json: String, write_on_bootstrap: bool }` (or similar)
  - Present in v2 schema; actual write semantics likely task `02`/later.

### 4) Parsing, defaults, validation (v1 preserved; v2 fail-closed for now)

#### 4.1 `src/config/schema.rs`

- Add `ConfigVersion` and new v2 schema types described above.
- Split input schema types:
  - `PartialRuntimeConfigV1` = the current `PartialRuntimeConfig` shape (legacy keys).
  - `RuntimeConfigV2Input` (name TBD) = strict explicit config shape.
- Keep `#[serde(deny_unknown_fields)]` on the v2 input structs.

#### 4.2 `src/config/parser.rs`

- Change loader to:
  - parse into `toml::Value`
  - read `config_version` (default v1 when missing)
  - deserialize into v1 or v2 input struct accordingly
- v1 path:
  - run existing `apply_defaults` (may become `apply_defaults_v1`)
  - validate with existing validation function (extended for any new runtime fields that become required)
- v2 path (task 01):
  - return a clear `ConfigError::Validation` explaining “v2 schema is recognized but not executable yet; task 02 implements v2 normalization”
  - still add unit tests proving the failure is intentional and message is useful

#### 4.3 `src/config/defaults.rs`

- Keep existing v1 defaults behavior for all current fields.
- When adding new runtime-required fields (like conninfo identity), fill them deterministically in v1:
  - local conn identity defaults to `user=postgres`, `dbname=postgres`, `ssl_mode=prefer` (but now explicit in runtime config)
  - rewind source identity defaults similarly (or derived from existing rewind host/port; identity is separate)
- Keep legacy API token fallback behavior strictly inside v1 normalization.

#### 4.4 Validation additions (no behavior changes beyond stronger consistency checks)

- Add validation for the new runtime-required conninfo identity fields (non-empty user/dbname).
- Add v2-only validation for TLS identity/client-auth consistency and required roles (even if v2 is fail-closed now, keep validator ready so task 02 can call it).

### 5) Minimal runtime plumbing (identity/conninfo only; everything else deferred)

Goal for task 01 runtime: remove *hardcoded* DSN identity defaults and make them come from `RuntimeConfig`.

- `src/runtime/node.rs`
  - `local_postgres_dsn(..)` uses `cfg.postgres.<local_conn_identity>` instead of hardcoding `postgres/postgres`.
- `src/ha/state.rs`
  - `ProcessDispatchDefaults::contract_stub()` must not embed hidden conninfo identity defaults; instead:
    - move defaults into v1 config normalization, and/or
    - require explicit identity fields in the runtime struct.

### 6) Compile-surface checklist (must be exhaustive; update all struct literals)

There are many direct `RuntimeConfig` / `PostgresConfig` / `ApiConfig` struct literals across code/tests/examples. Update these in a deliberate order to reduce churn.

**Priority 0 (schema + defaults + parser unit tests):**
- `src/config/schema.rs` — define new runtime/input types and fields
- `src/config/defaults.rs` — fill new required runtime fields for v1
- `src/config/defaults.rs` — update partial-config literals in defaults tests (`PartialRuntimeConfig`, `PartialPostgresConfig`, `PartialApiConfig`)
- `src/config/parser.rs` — version gate + tests (`base_runtime_config()` literal must be updated)
- `src/config/mod.rs` — re-export new types

**Priority 1 (examples + BDD + harness that compile in `--all-targets`):**
- `examples/debug_ui_smoke_server.rs` — update `RuntimeConfig` literal
- `tests/bdd_api_http.rs` — update `RuntimeConfig` literal and any token/TLS expectations as needed (keep legacy fallback tests intact for v1)
- `src/test_harness/ha_e2e/startup.rs` — update `RuntimeConfig` literal

**Priority 2 (remaining in-crate fixture literals; list from `rg` must be cleared):**
- `src/runtime/node.rs`
- `src/api/worker.rs`
- `src/api/fallback.rs`
- `src/debug_api/worker.rs`
- `src/debug_api/view.rs` (if config field names/shapes change; keep secret values out of debug view)
- `src/dcs/worker.rs`
- `src/dcs/store.rs`
- `src/dcs/state.rs`
- `src/dcs/etcd_store.rs`
- `src/ha/worker.rs`
- `src/ha/decide.rs`
- `src/logging/postgres_ingest.rs`
- `src/worker_contract_tests.rs`

Optional (only if churn is too high): introduce a single shared test fixture builder for `RuntimeConfig` under `src/test_harness/` and/or `#[cfg(test)]` in `src/config/` so tests/examples stop duplicating literals.

### 7) Gate sequence (must be 100% green before marking done)

Run in this order:

1. `make check`
2. `make lint`
3. `make test`
4. `make test-long`

If linker flakes appear on this mount, use the stabilization knobs already documented in AGENTS notes:
- `cargo clean`
- `CARGO_BUILD_JOBS=1`
- `CARGO_INCREMENTAL=0`
- `RUST_TEST_THREADS=1`

---

NOW EXECUTE
