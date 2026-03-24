## Plan: Collapse HA Runtime Overlay Onto Private Schema Owner

### Why this reduction target

Plan 50 proved the broad "let external callers mutate typed documents directly" design is wrong for this codebase:

- `src/config_v2/parser/private_schema.rs` already owns the real runtime/operator schema.
- External test callers do not need those raw schema structs; widening them just to avoid format strings regressed the reduction baseline and matched the skill's `too public` smell.
- The clearest remaining wrong boundary is narrower: `tests/ha/support/givens/mod.rs` is still a one-off runtime-config assembler that leaves the schema owner, parses TOML back into `toml::Value`, recursively merges tables, then serializes again.

This is the better slice because it attacks a large single-caller overlay layer without exporting more of the raw schema.

### Current overlap already verified

- `tests/ha/support/givens/mod.rs` is the only external caller of `build_runtime_test_document_value(...)`; elsewhere it is only used inside `private_schema.rs`.
- `build_ha_member_runtime_config(...)` currently:
  - builds a base runtime `toml::Value`
  - renders operator TOML as text
  - parses that operator text back into `toml::Value`
  - parses a second large runtime overlay string into `toml::Value`
  - recursively merges the two tables
  - serializes again
- `merge_toml_value(...)` and `merge_toml_tables(...)` exist only for that HA fixture path.
- `private_schema.rs` already owns `RuntimeDocument`, `OperatorDocument`, nested TLS/auth/path structs, and the final TOML serialization boundary.

### Execution plan

1. Revert the failed operator-surface widening before adding anything else.
   - Restore `src/cli/config.rs`, `src/config_v2/mod.rs`, `src/config_v2/parser/load_config.rs`, `src/config_v2/parser/mod.rs`, `src/config_v2/parser/private_schema.rs`, and `tests/ha/support/observer/pgtm.rs` to the last known baseline before the public `TokenAuthConfig` / `OperatorClientTlsInput` helper attempt.
   - Keep `tests/ha/support/givens/mod.rs` only as needed for the new HA-focused rewrite.

2. Move the HA runtime assembly back under the schema owner.
   - In `src/config_v2/parser/private_schema.rs`, extract the current `build_runtime_test_document_value(...)` literal into a private typed builder that returns `RuntimeDocument`.
   - Keep the existing generic render/value helpers as thin wrappers around that typed builder where they still earn their keep.
   - Add one focused test-support helper for the HA fixture that accepts primitive inputs already owned by the fixture (`member_name`, `dcs_endpoint`, `replicator`, `rewinder`) and returns rendered runtime TOML.

3. Build the HA config through typed mutation, not parse-merge.
   - Implement the HA helper by mutating the internal `RuntimeDocument` directly inside `private_schema.rs`, setting the runtime-only fields currently spelled out in the giant overlay string.
   - Reuse the existing internal operator-document builder for the embedded `pgtm` config and assign that serialized value once, without the render-parse-roundtrip in the fixture.
   - Do not expose `RuntimeDocument`, `OperatorDocument`, `TokenAuthConfig`, or `OperatorClientTlsInput` publicly just to make this work.

4. Collapse the HA fixture caller.
   - Rewrite `tests/ha/support/givens/mod.rs` to call the focused HA runtime helper.
   - Delete the raw overlay string, the operator TOML reparsing, and the single-caller `merge_toml_value(...)` / `merge_toml_tables(...)` helpers.

5. Remove no-longer-needed surface area.
   - If the HA fixture no longer needs `build_runtime_test_document_value(...)` outside `private_schema.rs`, stop re-exporting it from `src/config_v2/parser/mod.rs` and `src/config_v2/mod.rs`.
   - Delete any now-unused imports or test-support helpers that existed only for the removed overlay path.

6. Validate the reduction.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require improvement beyond the tracked `+8109 -11146 diff: -3037` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- No new public raw-schema DTOs or helper enums. Reuse the existing private schema types internally.
- No generic second abstraction layer for "patching" runtime documents. This slice should be one focused HA helper plus deleted caller-side glue.
- If the HA helper starts duplicating most of `RuntimeDocument` construction instead of reusing the shared builder, stop and re-plan.
- If reverting the failed operator helper slice reveals that the HA-only collapse is not enough to beat `diff: -3037`, switch this plan back to `TO BE VERIFIED` with the measured failed-yield note before running the full checks.

NOW EXECUTE
