---
## Task: Migrate parser/defaults/validation to explicit enum-driven config semantics <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Remove hidden config inference by moving defaulting/validation behavior to explicit enum-driven semantics while preserving safe startup requirements.

**Scope:**
- Refactor `src/config/parser.rs` and `src/config/defaults.rs` to stop injecting implicit runtime identities (for example `postgres` user fallback).
- Introduce explicit default policy only where permitted by typed enums and safe documented defaults.
- Ensure parser errors are actionable when required secure fields are missing.
- Update config docs/comments/tests to reflect explicit requirements and no-inference contract.

**Context from research:**
- Current parser/default flow still fills values that should become explicit secure config inputs.
- Safe startup requires deterministic config sources and clear failure when mandatory values are missing.

**Expected outcome:**
- Config load path rejects incomplete secure configs.
- Any defaults that remain are enum-anchored and centrally defined, not scattered magic fallbacks.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] `src/config/parser.rs` has no inferred user/role/TLS identity fallback behavior outside explicit default enums
- [x] `src/config/defaults.rs` is reduced to safe explicit defaults and does not silently synthesize sensitive identities
- [x] Parse/validate error paths clearly identify missing required secure config fields
- [x] Existing fixtures and sample configs are updated or intentionally rejected with explicit migration guidance
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

---

## Plan (parser/defaults/validation migration)

### 0) Non-negotiables / constraints

- No `unwrap()` / `expect()` / `panic!()` anywhere (repo lint policy).
- “Secure explicit config” means: **no code path invents Postgres users/dbnames/roles/auth/TLS posture**.
- All remaining defaults must be:
  - clearly scoped to non-sensitive knobs (timeouts, logging, bind addresses), and
  - centrally defined (no scattered ad-hoc fallbacks).
- Error messages for missing required secure inputs must point to **fully-qualified field paths** (e.g. `postgres.local_conn_identity.user`), not vague “missing field `user`”.

### 1) Decisions to lock early (behavioral contract)

- [x] **Require explicit `config_version` in TOML.**
  - Missing `config_version` must be a `ConfigError::Validation { field: "config_version", ... }`
    with migration guidance: set `config_version = "v2"` (explicit secure schema).
  - Rationale: removes “hidden” behavior selection and prevents accidental reliance on legacy inference.
- [x] **`config_version = "v2"` becomes executable** (this is the main deliverable of task 02).
- [x] **Lock legacy policy for `config_version = "v1"`: Option A (fail-closed, migrate).**
  - Parse v1 shape only to provide a high-quality migration error, but **do not** execute v1.
  - Return `ConfigError::Validation { field: "config_version", ... }` with guidance:
    - “`config_version = "v1"` is no longer supported because it depends on implicit security defaults.”
    - “Migrate to `config_version = "v2"` and provide explicit TLS/auth/identity/role/pg_hba/pg_ident fields.”
  - Rationale: v1 cannot be made “no-inference” without expanding its shape; supporting it keeps the insecure defaulting surface alive.

### 2) Exhaustive file/module checklist (what changes, and why)

- [x] `src/config/parser.rs`
  - Enforce explicit `config_version` (no `unwrap_or(V1)`).
  - Execute `v2` path: parse -> normalize -> validate -> `RuntimeConfig`.
  - If `v1` remains supported: route to compat normalizer (not `defaults.rs`) and ensure behavior is explicit in naming + errors.
  - Add/adjust unit tests to cover:
    - missing `config_version` is rejected with migration guidance
    - v2 config loads successfully
    - missing v2 secure fields produces `ConfigError::Validation` with full field path(s)
- [x] `src/config/schema.rs`
  - **Scope-reduced refactor:** keep `RuntimeConfig` as the normalized runtime target type, but introduce *targeted* v2 input wrappers only where we need precise missing-field errors.
    - Keep “safe” sections as strongly-typed required structs (cluster/dcs/ha/process/logging/debug) unless they are actively painful.
    - Introduce wrappers with `Option<T>` specifically for security-sensitive subtrees:
      - `postgres.local_conn_identity`, `postgres.rewind_conn_identity`
      - `postgres.roles` (+ each role)
      - `postgres.tls`
      - `postgres.pg_hba`, `postgres.pg_ident`
      - `api.security` (auth + tls)
    - Keep `#[serde(deny_unknown_fields)]` on all v2 input structs.
    - Avoid relying on serde “missing field” for security-required values; convert missing values into our own `ConfigError::Validation` with fully-qualified field paths.
- [x] `src/config/defaults.rs`
  - Remove all synthesis of sensitive identity/auth material:
    - no default Postgres users/dbnames
    - no default role usernames/auth
    - no “empty pg_hba/pg_ident” unless that is explicitly requested via config (i.e. must not be created implicitly here)
    - no API auth/tls inference from legacy fields
  - Retain only “safe defaults” functions/constants used by normalization:
    - timeouts, logging defaults, bind default(s), etc.
  - Update/replace defaults unit tests to assert only safe defaulting behavior.
- [x] `src/config/mod.rs`
  - Export any new normalization helpers/types required by other modules/tests.
  - If a new compat module is introduced, decide whether it is `pub(crate)` and keep surface minimal.
- [x] (Optional / if needed) `src/config/compat_v1.rs` (or similar new module)
  - Only exists if `config_version="v1"` remains executable.
  - Contains all legacy inference explicitly, with names like `normalize_legacy_v1(...)`.
  - Must not leak into `defaults.rs` or v2 normalization.
- [x] Any affected config-producing tests/examples (only if behavior change requires it)
  - `src/config/parser.rs` tests: update the TOML fixtures to include `config_version`.
  - `src/config/defaults.rs` tests: stop asserting default identities/roles that are no longer synthesized.

### 3) Implement v2 normalization (no hidden inference)

- [x] Add `normalize_runtime_config_v2(input: RuntimeConfigV2Input) -> Result<RuntimeConfig, ConfigError>`
  - Apply only safe defaults:
    - `postgres.connect_timeout_s` default allowed
    - logging defaults allowed
    - `api.listen_addr` default allowed
    - debug/logging toggles allowed
  - Require explicit secure fields (no defaulting):
    - `postgres.local_conn_identity` + `postgres.rewind_conn_identity`
    - `postgres.roles.{superuser,replicator,rewinder}` (and each `username`, plus `auth`)
    - `postgres.tls` mode must be explicit; if `optional|required` then identity must be present and valid
    - `postgres.pg_hba.source` and `postgres.pg_ident.source` must be explicitly provided (inline or path)
    - `api.security.tls` + `api.security.auth` must be explicit in v2 (no legacy bridging)
- [x] Add a focused `validate_runtime_config_v2(cfg: &RuntimeConfig) -> Result<(), ConfigError>` *(implemented by hardening `validate_runtime_config` directly, to apply invariants uniformly)*
  - Keep existing non-empty/port/timeout validation.
  - Add semantic checks with field-path errors:
    - TLS mode consistency: identity required when mode != disabled; client_auth consistency
    - `ApiAuthConfig::RoleTokens`: **reject** configs where both tokens are missing/empty; require at least one of `read_token` or `admin_token` to be present and non-empty
    - Role password: reject empty inline password; reject empty path
    - Postgres/server TLS identity: when `mode` is `optional|required`, require `identity.cert_chain` + `identity.private_key` (variant-specific leaf path errors)

### 4) Migration UX: actionable, specific errors

- [x] When rejecting missing v2 secure fields, error messages must:
  - Name the exact missing field path.
  - Include a short “how to fix” hint (example snippet reference is OK; do not paste large TOML).
- [x] When rejecting missing `config_version`, include:
  - “Set `config_version = \"v2\"` to use the explicit secure schema.”
  - If v1 is still supported: “Set `config_version = \"v1\"` to opt into legacy behavior (not recommended).”

### 5) Test plan (update + add coverage)

- [x] `src/config/parser.rs`
  - Update `load_runtime_config_roundtrip_and_defaults` to include an explicit version (v1 or v2, per chosen policy).
  - Replace `load_runtime_config_v2_is_recognized_but_fails_closed` with:
    - a v2 happy-path load test, asserting key fields survive roundtrip
    - v2 missing-secure-field tests that assert `ConfigError::Validation { field: "postgres.local_conn_identity", .. }`-style failures
  - Add `missing_config_version_is_rejected` test.
- [x] `src/config/defaults.rs`
  - Replace tests that assert default `postgres`/`replicator`/`rewinder` identities with:
    - tests for safe defaults only (timeouts/logging/bind defaults)
    - tests proving sensitive fields are not synthesized in v2 normalization (i.e., missing fields must error)

### 6) Gates (must be green before marking passing)

- [x] `make check`
- [x] `make lint`
- [x] `make test`
- [x] `make test-long`
