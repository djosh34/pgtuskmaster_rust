## Plan: Collapse Private Schema Test Document Builders

### Why this reduction target

The parser entrypoint split is gone, but `src/config_v2/parser/private_schema.rs` still carries another large wrong-place duplication seam in its test-support builders:

- `build_runtime_test_document_value(...)` and `build_ha_member_runtime_document_value(...)` both assemble large `RuntimeDocument` trees directly in the same file, with repeated cluster, postgres, DCS, process, logging, API, and `pgtm` structure wiring.
- `build_operator_test_document_value(...)` and `build_host_observer_operator_document_value(...)` do the same for `OperatorDocument`, even though the host-observer variant is just the generic operator document plus concrete auth/TLS defaults.
- The current shape bloats `private_schema.rs` to 1206 lines while keeping the raw schema module responsible for repeated hand-written test document construction instead of one shared owner per document shape.
- Callers only need rendered TOML or a `toml::Value`; they do not benefit from multiple duplicated constructors that restate the same raw config layout.

That is config-not-reduced plus multiple-functions-with-large-overlap. The next reduction should reuse the existing raw schema types directly instead of rebuilding near-identical documents in separate functions.

### Current overlap already verified

- `build_runtime_test_document_value(...)` already provides the generic runtime-test baseline; `render_runtime_test_config_toml(...)` only trims and appends extra sections on top of it.
- `build_ha_member_runtime_document_value(...)` serializes another `RuntimeDocument` by hand even though it is mostly the same raw schema shape with different concrete values.
- `build_operator_test_document_value(...)` already owns the generic operator document layout.
- `build_host_observer_operator_document_value(...)` restates that operator layout with fixed auth/TLS values and repeated `OperatorClientTlsConfig` assembly.

### Execution plan

1. Introduce one shared raw-document constructor per test shape.
   - Add a shared runtime raw-document builder that returns a `RuntimeDocument` baseline instead of jumping directly to `toml::Value`.
   - Add a shared operator raw-document builder that returns an `OperatorDocument` baseline for the generic operator shape.
   - Reuse existing raw schema structs/enums; do not add a new builder DSL or a second helper module.

2. Make specialized builders mutate the shared raw document instead of rebuilding it.
   - Rework `build_ha_member_runtime_document_value(...)` to start from the shared runtime document owner and then overwrite only the fields that differ for HA fixtures.
   - Rework `build_host_observer_operator_document_value(...)` to start from the shared operator document owner and then apply the auth/TLS/resolve-to details needed for host-observer configs.
   - Keep current rendered TOML surfaces unchanged: `render_runtime_test_config_toml`, `render_ha_member_runtime_config_toml`, `render_operator_test_config_toml`, and `render_host_observer_operator_config_toml`.

3. Delete duplicate local construction and keep helpers narrow.
   - Remove repeated inline `RuntimeDocument` / `OperatorDocument` assembly that becomes dead after the shared builders exist.
   - Prefer mutating existing raw structs over adding more trimming or post-serialization string helpers.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+3936 -6304 diff: -2368` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Tick the task boxes only after the gates pass.

### Guardrails

- Keep `private_schema.rs` as the owner of raw config/test document rendering; do not push this duplication into another wrapper module.
- Reuse the existing raw schema structs directly; do not create parallel DTOs or another layer of config builders.
- If the shared-document approach starts adding more mutation scaffolding than the duplicated constructors it replaces, switch this plan back to `TO BE VERIFIED`, document the mismatch, retarget the task, and stop immediately.

NOW EXECUTE
