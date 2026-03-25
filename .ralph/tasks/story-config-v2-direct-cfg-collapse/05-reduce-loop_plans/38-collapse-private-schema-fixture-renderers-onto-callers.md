## Plan: Collapse Private Schema Fixture Renderers Onto Callers

### Why this reduction target

`src/config_v2/parser/private_schema.rs` still owns two large fixture-specific renderers even though the generic test-config surfaces already exist and the fixture-specific values belong to their only callers:

- `render_ha_member_runtime_config_toml(...)` / `build_ha_member_runtime_document_value(...)` hand-mutate a large `RuntimeDocument` for the HA cucumber fixture, but `render_runtime_test_config_toml(...)` already owns the baseline runtime config shape and explicitly trims `ha`, `process`, `logging`, `api`, `pgtm`, and `debug` so extra sections can be appended.
- `render_host_observer_operator_config_toml(...)` builds a dedicated operator config variant for exactly one caller, while `render_operator_test_config_toml(...)` already owns the base operator document and its raw schema skips empty auth/TLS sections.
- `tests/ha/support/givens/mod.rs` is the only external caller of `render_ha_member_runtime_config_toml(...)`.
- `tests/ha/support/observer/pgtm.rs` is the only external caller of `render_host_observer_operator_config_toml(...)`.

That is wrong-place fixture ownership. The parser schema module should keep the generic config renderers and reusable TOML helpers, while the HA harness modules own the concrete fixture snippets they alone need.

### Current overlap already verified

- `src/config_v2/parser/private_schema.rs::render_runtime_test_config_toml(...)` already renders the shared runtime baseline and trims the exact sections the HA fixture later repopulates.
- `src/config_v2/parser/private_schema.rs::render_operator_test_config_toml(...)` already accepts the host observer's base URL, expected transport, and resolve-to values; raw operator auth/TLS tables are skipped when empty.
- `tests/ha/support/givens/mod.rs` imports only `render_ha_member_runtime_config_toml(...)` from config test support.
- `tests/ha/support/observer/pgtm.rs` imports only `render_host_observer_operator_config_toml(...)` from config test support, while it already owns the parallel PostgreSQL TLS conninfo rendering locally.
- `src/config_v2/parser/private_schema.rs` is still 1139 lines, so deleting one-off fixture rendering from it is a meaningful reduction target.

### Execution plan

1. Move HA fixture-specific runtime config rendering onto the HA givens caller.
   - In `tests/ha/support/givens/mod.rs`, replace `render_ha_member_runtime_config_toml(...)` with a thin caller-owned wrapper around `render_runtime_test_config_toml(...)`.
   - Build the HA-only TOML sections in that module from the existing fixture constants and member/role inputs.
   - Reuse existing config test-support helpers such as `toml_path_source(...)`; only add one tiny literal helper if a quoting gap is truly unavoidable.

2. Move host-observer operator config rendering onto the observer caller.
   - In `tests/ha/support/observer/pgtm.rs`, replace `render_host_observer_operator_config_toml(...)` with a caller-owned wrapper around `render_operator_test_config_toml(...)`.
   - Pass the base URL, expected transport, and resolve-to through the generic renderer, then append only the observer-specific auth and TLS sections.
   - Keep the rendered config semantically identical: HTTPS, role-token file paths, matching API/Postgres client TLS, and the same resolve-to endpoint.

3. Delete the now-dead specialized private-schema renderers.
   - Remove `build_ha_member_runtime_document_value(...)`, `render_ha_member_runtime_config_toml(...)`, and `render_host_observer_operator_config_toml(...)` from `src/config_v2/parser/private_schema.rs`.
   - Remove any helper that becomes dead with them.
   - Stop re-exporting the deleted surfaces from `src/config_v2/parser/mod.rs` and `src/config_v2/mod.rs`.

4. Rebuild tests around the surviving generic surfaces.
   - Update HA givens and observer tests to validate the new caller-owned renderers still produce parse-valid configs and preserve the expected role/TLS/path behavior.
   - Keep parser/CLI tests using the existing generic config renderers unchanged unless a deleted export forces a minimal import rewrite.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+7247 -10007 diff: -2760` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not add a new builder DSL, a parallel config DTO layer, or more parser-only fixture helpers.
- Prefer caller-owned TOML snippets over rebuilding large `RuntimeDocument` / `OperatorDocument` trees in the parser schema module.
- Keep the generated config behavior identical: if appended sections start producing duplicate-table TOML or larger helper scaffolding than the code removed, switch this plan back to `TO BE VERIFIED`.

### Execution blocker found while starting implementation

`render_runtime_test_config_toml(...)` keeps the full `postgres.roles.mandatory.*` tree, including nested `auth` tables. The HA cucumber custom-role fixture must change the replicator and rewinder usernames and password sources, so re-appending caller-owned TOML sections would redefine already-rendered tables/keys and produce invalid TOML. That means the current design is not executable as written.

TO BE VERIFIED
