## Plan: Collapse Split Config Parser Entrypoints

### Why this reduction target

The worker-bootstrap slice is already complete, but the config parser still carries one artificial split:

- `src/config_v2/parser/load_config.rs` already owns the real config-mapping logic for both runtime and operator shapes, including `map_operator_document`, operator URL validation, token parsing, and TLS merging.
- `src/config_v2/parser/load_operator_config.rs` is mostly a courier that reuses `read_config_file`, `parse_error`, `validation_error`, `map_operator_document`, and the raw schema from other modules.
- `src/config_v2/parser/mod.rs` and `src/config_v2/mod.rs` then re-export the split entrypoints as if runtime and operator loading were separate subsystems.
- Callers only care about `load_operator_config(...)` and `load_operator_config_contents(...)`; they do not benefit from a second loader file that owns almost none of the parsing behavior.

That is still wrong-placeism plus remove-the-damn-helpers. The operator loader boundary is pretending to be a real module boundary, but the actual ownership already lives in `load_config.rs`.

### Current overlap already verified

- `src/config_v2/parser/load_config.rs` already contains `map_operator_document`, `map_operator_api_route`, `map_expected_transport`, `map_operator_client_tls`, and the shared file/parse/validation helpers used by both loaders.
- `src/config_v2/parser/load_operator_config.rs` only adds the top-level file read, a contents-based test helper, one `OperatorConfigDocument` match, and its local tests.
- `src/cli/config.rs`, `tests/ha/support/observer/pgtm.rs`, and other callers import the operator loader through `config_v2`; none of them depend on `load_operator_config.rs` as a boundary.
- The parser directory is still 2760 lines total, with a 298-line loader file whose main job is forwarding into code owned elsewhere.

### Execution plan

1. Collapse operator loading into the existing parser owner.
   - Move `load_operator_config`, `load_operator_config_contents`, and the `OperatorConfigDocument` dispatch logic into `src/config_v2/parser/load_config.rs`, next to `map_operator_document`.
   - Keep the public function names unchanged so callers stay on the same API surface.
   - Reuse the existing raw schema and mapping functions; do not introduce a generic loader trait or a new shared abstraction.

2. Delete the thin loader file and narrow re-export plumbing.
   - Remove `src/config_v2/parser/load_operator_config.rs`.
   - Update `src/config_v2/parser/mod.rs` so it only exposes `load_config` plus `private_schema`.
   - Keep `src/config_v2/mod.rs` exporting the same config_v2-facing functions, but source them from the collapsed parser owner.

3. Move the operator loader tests to the surviving owner and trim overlap.
   - Fold the relevant `load_operator_config.rs` tests into `load_config.rs` alongside the runtime-loader tests.
   - Remove any test scaffolding that only existed because operator loading lived in a second file.
   - Keep behavior coverage for standalone operator documents and runtime documents with embedded `pgtm`.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+3933 -6236 diff: -2303` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Tick the task boxes only after the gates pass.

### Guardrails

- Keep `load_config.rs` as the single owner of config parsing/mapping logic instead of moving code into another wrapper module.
- Reuse existing raw schema enums/structs and validated config types; do not add new parser DTOs.
- Prefer deleting forwarding helpers and duplicate test scaffolding over renaming files without net reduction.
- If folding the operator entrypoints into `load_config.rs` starts increasing indirection instead of deleting it, switch this plan back to `TO BE VERIFIED`, document the mismatch, retarget the task file, and stop immediately.

NOW EXECUTE
