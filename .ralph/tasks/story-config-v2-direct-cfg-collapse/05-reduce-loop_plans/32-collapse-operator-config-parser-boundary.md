## Plan: Collapse Operator Config Parser Boundary

### Why this reduction target

`config_v2` still carries a dedicated operator document/mapping boundary even though the validated operator runtime shape is already small and canonical in `src/config_v2/types.rs`:

- `src/config_v2/parser/private_schema.rs` owns `OperatorConfigDocument`, `OperatorDocument`, `OperatorApiConfig`, `OperatorPostgresConfig`, and a parser-local `OperatorClientTlsConfig`.
- `src/config_v2/parser/load_config.rs` immediately remaps that raw operator stack through `map_operator_document`, `map_operator_auth`, `map_operator_client_tls`, `merge_operator_client_tls`, and URL/transport helpers into `OperatorConfigV2`.
- The operator test renderers in `private_schema.rs` build the same raw operator document layer only to serialize TOML for tests and harness helpers.

That is a wrong-place boundary. The repo already has canonical validated operator types, but standalone operator loading and operator test documents still bounce through a second owner layer before ending up there.

### Current overlap already verified

- `src/config_v2/types.rs` already owns the final validated operator semantics in `OperatorConfigV2`, `OperatorClientTlsConfig`, and `PgtmApiTransportExpectation`.
- `src/config_v2/parser/private_schema.rs` uses a second operator stack only for TOML-facing structure: base URL, advertised URL, expected transport, resolve-to, auth, and client TLS are all restated there.
- `src/config_v2/parser/load_config.rs` uses the same operator mapping path both when loading a standalone operator config and when extracting the embedded runtime `pgtm` block.
- The operator tests in `src/config_v2/parser/load_config.rs` prove the behavior that must survive: expected-transport preservation, resolve-to preservation, advertised URL preservation, auth flattening, merged client TLS, and parse-boundary rejection for non-path TLS identity sources.
- `src/cli/config.rs` already consumes only `OperatorConfigV2`; it does not need the raw operator document layer.

### Execution plan

1. Make the validated operator types the only operator owner.
   - Reuse `OperatorConfigV2` and `types::OperatorClientTlsConfig` as the canonical operator surface.
   - Keep only the truly input-only leaf shapes that still differ from runtime values, such as path/secret sources and TLS identity source parsing.
   - Delete the raw operator wrapper structs that only courier `[api]` and `[postgres]` sections into the same validated fields.

2. Collapse standalone and embedded operator loading onto one smaller parse path.
   - Remove `OperatorConfigDocument` and the dedicated standalone operator wrapper match in `load_operator_config_contents_at`.
   - Rebuild the operator parse path so both standalone operator files and embedded `pgtm` blocks flow through the same canonical operator construction logic.
   - Preserve existing URL normalization, expected-transport validation, token flattening, and API/postgres TLS merge behavior exactly.

3. Shrink operator-specific helper sprawl in `load_config.rs`.
   - Inline or consolidate operator-only helpers that exist only to translate one raw operator type into its validated twin.
   - Keep only the operator helpers that still serve more than one real call site after the raw wrapper layer is gone.
   - Avoid creating a replacement adapter family; the point is to delete the fake boundary, not move it.

4. Rebuild operator test document rendering around the surviving input-only surface.
   - Replace `operator_test_document`, `build_operator_test_document_value`, and adjacent operator-only builder glue with a smaller rendering path that targets the surviving operator input surface directly.
   - Keep `render_operator_test_config_toml` and `render_host_observer_operator_config_toml` behavior stable for their existing callers.
   - Preserve the parse-boundary guarantee that TLS client identity sources must stay path-based.

5. Validate the reduction.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+6928 -9565 diff: -2637` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Reuse the existing operator runtime types; do not introduce a replacement operator DTO or a second shared adapter module.
- Do not widen this slice back into the older broad config-document migration. Runtime config ownership outside the operator path is out of scope here.
- Keep `load_operator_config_contents(...)` and runtime `pgtm` extraction behavior identical at the API boundary.
- If making the canonical operator types own too much deserialization forces a large custom visitor or pushes the same wrapper layer into tests under a new name, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
