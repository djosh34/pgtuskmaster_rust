## Plan: Collapse Config Document Boundaries

### Why this reduction target

The largest remaining reduction seam is the config document boundary itself. The repo already has one raw document model in `src/config_v2/parser/private_schema.rs`, but it still carries multiple parallel layers around that same shape:

- `src/dev_support/runtime_config.rs` hand-renders runtime, HA-member, and operator TOML strings.
- `tests/ha/support/world/mod.rs` adds another runtime-config materialization wrapper on top of those renderers.
- `tests/ha/support/observer/pgtm.rs` manually assembles operator TOML sections again.
- `src/config_v2/parser/load_operator_config.rs` re-parses and re-maps the operator block even though `load_config.rs` already owns adjacent operator mapping and validation logic for runtime documents.

That is a boundary problem, not just a helper problem. The same config/document knowledge is split across parser code, test support, and HA harness materialization, which inflates line count and makes future config changes fan out.

### Current overlap already verified

- `src/config_v2/parser/private_schema.rs` already defines `RuntimeDocument`, `OperatorConfigDocument`, `OperatorDocument`, token/TLS/path source enums, and the default-bearing nested config structs.
- `src/dev_support/runtime_config.rs` re-encodes those raw shapes as TOML strings via `render_runtime_test_config_toml`, `render_ha_member_runtime_config_toml`, `render_operator_test_config_toml`, plus `toml_string`, `toml_path_source`, and `toml_string_secret`.
- `tests/ha/support/world/mod.rs::render_member_runtime_config` is a thin wrapper over the HA runtime renderer and `materialize_runtime_config` is a thin write layer above that.
- `tests/ha/support/observer/pgtm.rs::build_host_observer_config` manually rebuilds the same operator auth/TLS document sections that `render_operator_test_config_toml` already owns.
- `src/config_v2/parser/load_config.rs` already contains `map_operator_api_route`, `map_expected_transport`, token-source helpers, and path/secret resolution used for operator-shaped config inside runtime documents.
- `src/config_v2/parser/load_operator_config.rs` duplicates the operator expected-transport mapping, URL validation, token extraction, and client-TLS merge path for standalone operator documents.

### Execution plan

1. Promote the existing raw config schema into the shared document owner instead of keeping handwritten TOML renderers.
   - Reuse the existing raw structs/enums from `private_schema`; do not create a second document model.
   - Derive or add only the serialization support needed so the raw schema can be rendered back to TOML for tests and harness materialization.
   - If needed, rename/rehome `private_schema` so it is clearly the config document module rather than a parser-private dumping ground, but keep ownership centralized in `config_v2`.

2. Replace handwritten TOML rendering helpers with typed document builders plus serialization.
   - Refactor `src/dev_support/runtime_config.rs` so runtime/operator/HA config helpers build raw document values and serialize them, instead of formatting long TOML strings manually.
   - Collapse `toml_string`, `toml_path_source`, `toml_string_secret`, and the large string-template renderers if the typed document path makes them unnecessary.
   - Keep `write_temp_toml` and validation helpers only if they still serve a purpose after the renderer collapse.

3. Collapse HA harness config materialization onto the same shared document path.
   - Refactor `tests/ha/support/world/mod.rs` to materialize member runtime configs from the shared typed document builders instead of layering another `render_member_runtime_config` wrapper over handwritten strings.
   - Refactor `tests/ha/support/observer/pgtm.rs` to build the observer operator config from that same shared operator-document surface instead of manually assembling duplicate auth/TLS sections.
   - Prefer deleting wrapper/helper functions over moving them sideways.

4. Collapse operator parsing onto one shared mapping path.
   - Extract the shared operator-document-to-`OperatorConfigV2` mapping/validation out of the duplicated runtime/operator loader split.
   - Make `load_runtime_config` and `load_operator_config` both reuse that one operator mapping surface for expected transport, base/advertised URL parsing, auth, and merged client TLS.
   - Reuse existing helper functions and existing config types; do not introduce a generic parser framework.

5. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff moves further downward.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Tick the task boxes only after the gates pass.

### Guardrails

- Do not serialize the public `RuntimeConfigV2`/`OperatorConfigV2` types just to render tests. The existing raw schema is the right reuse point.
- Preserve current config semantics, defaults, and field names exactly; this slice is about ownership and duplication, not a config-format redesign.
- If deriving/annotating serialization on the raw schema creates more compatibility glue than it deletes, switch this plan back to `TO BE VERIFIED`, document the blocking raw-shape mismatch, update the task file to point at that state, and stop immediately.

NOW EXECUTE
