## Plan: Collapse Operator Test-Config String Overlays Onto Existing Auth/TLS Schema

### Why this reduction target

The failed plan-48 HA fixture rewrite proved the local direct-mutation approach can delete the render/parse/merge roundtrip and still lose on total line count. The broader overlap is one layer above that fixture:

- `src/config_v2/parser/private_schema.rs` already owns the operator auth/TLS shape through `TokenAuthConfig`, `RoleTokens`, `OperatorClientTlsInput`, and `build_operator_test_document_value_with_parts(...)`.
- Several callers ignore that owner and reconstruct the same shape as raw TOML strings passed through `render_operator_test_config_toml(..., extra_sections)`.
- The duplicated string overlays are not HA-specific. They recur in `src/cli/config.rs`, `src/config_v2/parser/load_config.rs`, and `tests/ha/support/observer/pgtm.rs`.
- The current dirty worktree also still carries the failed plan-48 exports and HA fixture mutation rewrite, so the next execution needs a slice that first drops that regression and then reuses the existing operator schema owner.

This is a better boundary target because one shared operator-document surface can delete duplicate auth/TLS TOML fragments across multiple files, instead of only moving complexity around inside a single HA fixture.

### Current overlap already verified

- `build_operator_test_document_value_with_parts(...)` already accepts the full operator-test inputs needed by the duplicated callers: routing, auth, API TLS, and Postgres TLS.
- The duplicated raw TOML overlays exist in:
  - `src/cli/config.rs` for role-token auth and client TLS test setup.
  - `src/config_v2/parser/load_config.rs` for the same auth/TLS merge cases and one invalid TLS identity test.
  - `tests/ha/support/observer/pgtm.rs` for the observer config with role tokens plus mirrored API/Postgres TLS.
- The failed plan-48 work added `build_operator_test_document_value` exports and a large `toml::Value` mutation block in `tests/ha/support/givens/mod.rs`, but `bash .ralph/git_diff_lines_since.sh` regressed from the tracked `+8109 -11146 diff: -3037` baseline to `+8213 -11146 diff: -2933`.

### Execution plan

1. Remove the failed plan-48 slice before adding anything new.
   - Drop the dirty-worktree-only public exports added for `build_operator_test_document_value(...)` in `src/config_v2/parser/mod.rs` and `src/config_v2/mod.rs` if the new slice does not need them.
   - Restore `tests/ha/support/givens/mod.rs` to the compact pre-plan-48 implementation rather than carrying forward the direct-mutation rewrite that already failed the reduction metric.
   - Keep plan 48 as historical context only; plan 49 becomes the active execution target.

2. Reuse the existing operator schema as the only owner.
   - In `src/config_v2/parser/private_schema.rs`, expose one cfg-gated operator test-support entrypoint that accepts the already-existing auth/TLS inputs handled by `build_operator_test_document_value_with_parts(...)`.
   - Do not invent new public DTOs if helper constructors over the existing private structs are enough.
   - Prefer a small, explicit surface such as role-token auth and TLS helper constructors plus one richer operator render/document builder, rather than another generic string overlay mechanism.

3. Move the cross-file callers onto that owner.
   - Rewrite the operator-config tests in `src/cli/config.rs` to build auth/TLS through the richer operator test-support API instead of hand-written TOML fragments.
   - Rewrite the matching operator-config tests in `src/config_v2/parser/load_config.rs` the same way.
   - Rewrite `tests/ha/support/observer/pgtm.rs::build_host_observer_config(...)` onto the same API so the observer config no longer formats duplicated `[api.auth]`, `[api.tls]`, and `[postgres.tls]` blocks.

4. Delete only the now-redundant glue.
   - Remove the duplicated format strings and the associated one-off path/token interpolation at the rewritten call sites.
   - If `toml_path_source(...)` and `toml_string_secret(...)` become operator-only dead code after the rewrite, remove them; if runtime-config callers still need them, keep them and do not widen this slice.
   - Keep invalid-shape coverage by constructing the one non-path TLS identity failure case through the smallest still-necessary direct TOML snippet if the typed surface intentionally cannot represent that invalid state.

5. Validate the reduction.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require an improvement beyond the tracked `+8109 -11146 diff: -3037` baseline, not merely recovery from the current regressed dirty state.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not keep both the failed HA direct-mutation rewrite and the new operator-schema slice in flight at the same time; remove the failed slice first.
- Do not add a new generic TOML merge utility, a new operator test DTO hierarchy, or another public string-overlay API.
- Keep `private_schema.rs` as the only owner of operator auth/TLS document shape; test callers should supply inputs, not re-describe the schema in strings.
- If the richer operator test-support surface starts growing into more code than the duplicated call sites it replaces, switch this plan back to `TO BE VERIFIED`.

Failed attempt on 2026-03-24: the cfg-gated `TokenAuthConfig` / `OperatorClientTlsInput` constructor surface plus `render_operator_test_config_toml_with_parts(...)` did remove the duplicated auth/TLS format strings, but `bash .ralph/git_diff_lines_since.sh` regressed from the tracked `+8109 -11146 diff: -3037` baseline to `+8172 -11151 diff: -2979`, with total tracked `src/` + `tests/` lines rising to 35096. The boundary is still right, but this API shape adds more code than it deletes. Next turn should either collapse onto an even smaller helper surface or choose a different higher-yield reduction slice and revert/replace the current dirty implementation before executing again.

TO BE VERIFIED
