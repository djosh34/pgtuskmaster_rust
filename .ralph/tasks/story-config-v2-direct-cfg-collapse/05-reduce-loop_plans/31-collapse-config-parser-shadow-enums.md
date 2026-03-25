## Plan: Collapse Config Parser Shadow Enums

### Why this reduction target

`src/config_v2/parser/private_schema.rs` still carries a small but real shadow type layer for config enums that already exist in `src/config_v2/types.rs`:

- `private_schema::LogLevel` duplicates `types::LogLevel`.
- `private_schema::FileSinkMode` duplicates `types::FileSinkMode`.
- `private_schema::PgtmApiTransportExpectation` duplicates `types::PgtmApiTransportExpectation`.
- `src/config_v2/parser/load_config.rs` immediately remaps those three enums through `map_log_level`, `map_file_sink_mode`, and `map_expected_transport`, even though the variants already match one-for-one.

This is the wrong boundary. The parser document module should only own shapes that differ from the validated runtime types, such as path/secret sources and tagged TLS/auth documents. Keeping identical enums in both places adds lines, mapping glue, and another place for config semantics to drift.

### Current overlap already verified

- `src/config_v2/parser/private_schema.rs` defines `LogLevel`, `FileSinkMode`, and `PgtmApiTransportExpectation`.
- `src/config_v2/types.rs` defines the same three enums with the same variants and names.
- `src/config_v2/parser/load_config.rs` maps those enums directly into their public twins without adding any behavior.
- The surrounding raw document structs only need serde-facing annotations and defaults; they do not require separate enum ownership for these three cases.

### Execution plan

1. Make the public config enums the canonical serde owner.
   - Add the serde derives and default annotations needed on `src/config_v2/types.rs` so `LogLevel`, `FileSinkMode`, and `PgtmApiTransportExpectation` can deserialize directly from the existing TOML spellings.
   - Reuse the current public enums; do not introduce a third shared enum module.

2. Delete the parser-local enum shadows.
   - Remove the three duplicate enum definitions from `src/config_v2/parser/private_schema.rs`.
   - Update the raw document structs in `private_schema.rs` to reference the canonical enums from `types.rs` instead.
   - Keep `private_schema.rs` focused on input-only shapes that still differ from the runtime config graph.

3. Collapse the now-redundant mapping glue.
   - Remove `map_log_level`, `map_file_sink_mode`, and `map_expected_transport` from `src/config_v2/parser/load_config.rs`.
   - Update runtime and operator document mapping to use the canonical enums directly.
   - Preserve current validation and error wording everywhere else.

4. Rebuild the parser test/document surface around the canonical enums.
   - Update any test-document helpers or serialization assertions that referenced the removed parser-local enums.
   - Keep the existing test helper exports stable unless deleting one clearly reduces code without moving the same logic elsewhere.

5. Validate the reduction.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+6925 -9517 diff: -2592` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not deserialize `RuntimeConfigV2` or `OperatorConfigV2` directly from TOML in this slice; only collapse the already-verified enum shadows.
- Do not introduce replacement enums, alias wrappers, or a new shared parser prelude module.
- If serde/default annotations on the public enums force a config spelling change or create wider downstream churn than the shadow enums cost, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
