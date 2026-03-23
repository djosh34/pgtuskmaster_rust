## Task: Delete `src/config/` And Prove Zero Config Dependencies Remain <status>done</status> <passes>true</passes>

<description>
Migrate the final code to src/config_v2, while not making ANYTHING new public inside src/config_v2/parser
All validation, in ALL code (must verify this), but only be done once, and that must be done only inside src/config_v2/parser.
All other validation functions must go, as you can encode that with non-optional rust types that cannot represent invalid states/configs.

When done you delete src/config, to validate that the full migration is complete.
</description>

<acceptance_criteria>
- [x] `src/config/` is deleted
- [x] `src/config_v2/parser` does NOT export any types
- [x] `src/lib.rs` no longer declares or re-exports old config modules/types
- [x] Repo-wide search finds zero code dependencies on `crate::config` or `pgtuskmaster_rust::config`
- [x] Docs/examples/fixtures no longer describe old config paths or old config type names
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly 
</acceptance_criteria>

<boundary_review>
Smell applied from `improve-code-boundaries`: wrong config-ingestion boundary.

Verified repository facts before execution:
- `src/config_v2/parser` already owns the TOML parsing boundary, and `src/config_v2/parser/mod.rs` only re-exports the two loader functions with `pub(crate)` visibility.
- The old `src/config/` package still exists only because downstream code and tests still depend on its leaf DTOs, helpers, and public exports.
- `src/lib.rs` still exposes `pub mod config;`, which keeps the old runtime/operator config types publicly reachable even though the real parser handoff already moved to config-v2.
- The remaining old-config dependencies are concentrated in:
  - test/dev-support builders and adapters: `src/dev_support/runtime_config.rs`, `src/dev_support/runtime_config_v2.rs`, `src/dev_support/api.rs`, `src/dev_support/binaries.rs`, `src/dev_support/auth.rs`, `crates/pgtuskmaster_test_support/src/lib.rs`
  - old leaf-type reuse inside runtime code: `src/cli/client.rs`, `src/cli/mod.rs`, `src/postgres_roles.rs`, `src/logging/postgres_ingest.rs`, `src/tls.rs`
  - tests and fixtures that still serialize or parse legacy public config structs: `tests/bdd_api_http.rs`, `tests/ha/support/timeouts/mod.rs`, `tests/ha/support/world/mod.rs`, `tests/ha/support/observer/pgtm.rs`
  - docs/generated docs that still name old config Rust types or `crate::config` paths: `docs/src/reference/tls-configuration.md`, `docs/src/reference/runtime-configuration.md`, `docs/src/explanation/process-management.md`, plus tracked generated outputs if they still contain those names

Verified design correction needed before execution:
- Do not make `RuntimeConfigV2`, `OperatorConfigV2`, or parser-private schema types newly public just to keep old tests compiling. That would recreate the same boundary leak under a new name.
- External integration tests cannot depend on `pub(crate)` config-v2 types directly, so their edge should become TOML/render helpers or test-support entry points that stay on the TOML side, not another exported config DTO tree.
- The correct collapse is:
  - one v2-native internal builder/helper surface for crate tests and dev support
  - local runtime/test types for auth, role rendering, TLS fixture setup, cleanup settings, and binary paths where reusing old config DTOs currently leaks ingestion concerns
  - TOML-at-the-edge helpers for external tests that need runtime/operator documents
- Do not introduce new public mirrors such as `RuntimeConfigToml`, `OperatorConfigToml`, `PgtmConfigV2`, or a public `RuntimeConfigV2` wrapper. If execution appears to require that, the design is wrong and this task must go back to `TO BE VERIFIED`.
</boundary_review>

<plan>
1. Replace the legacy test-support corridor with one v2-native helper surface first:
   - rewrite `src/dev_support/runtime_config.rs` to build validated `RuntimeConfigV2` directly
   - merge any still-useful sample logging/binary helpers into that v2-native surface
   - delete `src/dev_support/runtime_config_v2.rs` instead of keeping a legacy-to-v2 adapter layer
2. Remove old leaf-type reuse from runtime code rather than carrying `crate::config` DTOs forward:
   - collapse `src/cli/client.rs` and `src/cli/mod.rs` onto a local auth representation that stores optional token strings instead of `RoleTokens`/`SecretSource`
   - rewrite `src/postgres_roles.rs` so managed-role reconciliation uses local role specs and v2 secrets directly rather than `PostgresRoleName`, `PostgresRolePrivilege`, `PostgresRoleSlots`, `RoleAuthConfig`, and `SecretSource`
   - rewrite `src/logging/postgres_ingest.rs` to use validated cleanup fields or a tiny local cleanup struct instead of `LogCleanupConfig`
   - remove old-config test-only TLS materialization from `src/tls.rs`; keep production on the v2 path-only API
   - move binary test helpers in `src/dev_support/binaries.rs` to `config_v2::types::BinariesConfig` or narrower path helpers
3. Convert crate-internal tests to the new v2-native helper surface:
   - update the process, HA, API, logging, postgres-managed, and other crate tests that still call `RuntimeConfigBuilder::new()` plus `from_legacy_runtime_config(...)`
   - delete all `from_legacy_runtime_config(...)` callsites instead of renaming them
4. Convert external tests and fixtures to TOML-edge helpers rather than public legacy config structs:
   - change `src/dev_support/api.rs` and `crates/pgtuskmaster_test_support/src/lib.rs` so integration tests can build test routers without exposing old config modules
   - rewrite `tests/bdd_api_http.rs` to use the new test-support surface instead of `pgtuskmaster_rust::config::{...}`
   - rewrite `tests/ha/support/timeouts/mod.rs` and `tests/ha/support/world/mod.rs` to validate runtime fixture TOML through current config-v2 semantics rather than `toml::from_str::<RuntimeConfig>`
   - rewrite `tests/ha/support/observer/pgtm.rs` to render current operator TOML directly instead of serializing/deserializing `PgtmConfig`
5. Delete the old public module and the old source tree only after all callers are moved:
   - remove `src/config/`
   - remove `pub mod config;` from `src/lib.rs`
   - keep `src/config_v2/parser` private except for its existing loader re-exports
6. Clean source docs, generated docs, and tracked fixtures so repo-wide searches go to zero:
   - update `docs/src/reference/tls-configuration.md`, `docs/src/reference/runtime-configuration.md`, and `docs/src/explanation/process-management.md`
   - regenerate or remove tracked generated docs/artifacts such as `docs/book/**` and `docs/tmp/**` if they still mention old config module paths or old Rust type names
7. Prove the deletion rather than assuming it:
   - run repo-wide searches for `crate::config`, `pgtuskmaster_rust::config`, and the major old type names before deleting the module and again afterward
8. Run the required gates in repo order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
9. If execution exposes a genuinely missing shared validated shape in `config_v2::types` that downstream code still needs, switch this task back to `TO BE VERIFIED`, explain the precise missing shape here, and stop immediately. Do not reintroduce a public config mirror or keep `src/config/` as a bridge.
</plan>

NOW EXECUTE
