## Plan: Collapse HA Fixture TOML Roundtrip Onto Generic Document Builders

### Why this reduction target

`tests/ha/support/givens/mod.rs::build_ha_member_runtime_config(...)` still owns a large stringly-TOML roundtrip even though the parser test-support layer already exports a reusable runtime document builder:

- `build_ha_member_runtime_config(...)` starts from `build_runtime_test_document_value(...)`, but then leaves the typed/value-shaped path and switches back into stringly TOML overlays.
- It calls `render_operator_test_config_toml(...)`, immediately parses that rendered string back into `toml::Value`, and inserts it under `pgtm`.
- It builds a second giant `format!(r#"... TOML ..."#)` overlay, parses that string into `toml::Value`, and recursively merges it into the base document through one-off `merge_toml_value(...)` / `merge_toml_tables(...)`.
- The recursive merge helpers exist only for this one caller, and the overlay string exists only because the caller is not mutating the already-owned document directly.

That is the wrong boundary. The HA fixture already owns the exact config fields it wants to override, and the generic parser test-support already owns the baseline document shape. The extra render-parse-merge layer is pure courier work.

### Current overlap already verified

- `src/config_v2/parser/private_schema.rs::build_runtime_test_document_value(...)` already returns the baseline runtime config as `toml::Value`, before serialization freezes the structure into text.
- `tests/ha/support/givens/mod.rs::build_ha_member_runtime_config(...)` is the only caller of the local `merge_toml_value(...)` / `merge_toml_tables(...)` helpers.
- The same function also round-trips `render_operator_test_config_toml(...)` through `toml::from_str::<toml::Value>(...)` purely to obtain a `pgtm` table value.
- The HA overlay only overwrites known tables and fields: postgres network/rewind/TLS/roles/access/GUCs, process binary overrides, logging sink and cleanup settings, API TLS/auth/listen address, and debug enablement.
- `validate_runtime_document_contents(...)` is already called after serialization, so the fixture can keep the same final safety check after switching away from overlay merging.

### Execution plan

1. Reuse generic document builders as the only baseline owner.
   - In `src/config_v2/parser/private_schema.rs`, add a test-support export that builds the base operator test document as `toml::Value` with the same simple inputs currently accepted by `render_operator_test_config_toml(...)`.
   - Re-export that value builder from `src/config_v2/parser/mod.rs` and `src/config_v2/mod.rs`.
   - Keep raw operator structs private; the public test-support surface should remain `toml::Value` plus primitive inputs.

2. Collapse the HA runtime fixture onto direct document mutation.
   - In `tests/ha/support/givens/mod.rs`, keep `build_runtime_test_document_value(...)` as the starting point, then mutate the returned `toml::Value` directly instead of parsing a TOML overlay string.
   - Build the `pgtm` block from the new generic operator document-value helper, mutate its auth/TLS fields in value form, and insert it directly under the runtime document root.
   - Overwrite the HA-owned runtime fields directly in the document tables: listen host, rewind TLS, postgres server TLS, mandatory role usernames/password sources, access file paths, extra GUCs, binary overrides, logging cleanup/file sink settings, API listen/auth/TLS, and debug enablement.
   - Use at most a tiny local nested-table helper if repeated table descent would otherwise bloat the caller.

3. Delete the roundtrip-only glue.
   - Remove the parsed overlay string, the operator render-and-reparse step, and the one-off `merge_toml_value(...)` / `merge_toml_tables(...)` helpers from `tests/ha/support/givens/mod.rs`.
   - Keep `render_operator_test_config_toml(...)` for the other existing callers that still want rendered TOML text; do not create a second fixture-specific renderer.

4. Rebuild tests around the surviving owner.
   - Keep `build_ha_member_runtime_config(...)` returning rendered TOML text and validating it with `validate_runtime_document_contents(...)`.
   - Preserve the HA fixture semantics exactly: same TLS paths, same secret file locations, same custom-role overrides, same logging/process settings, and the same embedded `pgtm` semantics.
   - Update any imports affected by the new operator document-value helper.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+8109 -11146 diff: -3037` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not add new HA-specific config DTO structs or a new generic TOML merge utility.
- Do not move HA fixture knowledge back into `private_schema.rs`; only the generic document builders belong there.
- Prefer direct table mutation over string parsing. If the mutation path grows into more code than the overlay and merge stack it replaces, switch this plan back to `TO BE VERIFIED`.
- Keep the final rendered config semantically identical so the HA harness and parser tests continue to validate the same document shape.

### Execution feedback

- First execution attempt rewrote `tests/ha/support/givens/mod.rs` onto direct `toml::Value` mutation and exported a primitive-input operator document-value builder.
- The direct-mutation version removed the render/parse/merge roundtrip, but it still worsened the tracked reduction baseline from `+8109 -11146 diff: -3037` to `+8213 -11146 diff: -2933`.
- The failure mode is line-count overhead in the HA fixture rewrite itself; the slice needs either a substantially more compact mutation strategy or a different reduction target.

TO BE VERIFIED
