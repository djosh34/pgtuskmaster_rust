## Plan: Flatten Process Binaries And Delete Overrides Wrapper

### Why this is the next verified reduction target

The `config_v2` binary shape still carries an unnecessary compatibility wrapper:

- `src/config_v2/types.rs:324-359`
  - `ProcessConfig` cannot derive `Deserialize` directly because it detours through:
    - `BinaryResolutionConfig`
    - `ProcessConfigInput`
  - that wrapper exists only to read `[process.binaries.overrides]` and then discard the `overrides` node.

- `src/config_v2/parser/load_config.rs:896-929`
  - `ProcessBinariesConfig::finalize(...)` still hard-codes field names like:
    - `process.binaries.overrides.pg_ctl`
    - `process.binaries.overrides.initdb`
    - `process.binaries.overrides.pg_rewind`
    - `process.binaries.overrides.pg_basebackup`
  - so the parser error surface still exposes the legacy wrapper even though runtime code only reads `cfg.process.binaries.*`.

- The wrapper leaks across docs and fixtures:
  - `docs/src/reference/runtime-configuration.md:287`
  - `docs/src/how-to/bootstrap-cluster.md:89`
  - `docs/src/how-to/add-cluster-node.md:36`
  - `docs/src/how-to/backup-and-restore.md:34`
  - `tests/cli_binary.rs:278`
  - `tests/ha/support/givens/mod.rs:255-274`

This is exactly the kind of boundary smell the reduce loop should remove: config input still uses an adapter-only nesting level that the validated runtime shape does not need.

### Current overlap verified in code

1. The runtime shape already has the real owner.
   - `src/config_v2/types.rs:309-317` keeps one `ProcessConfig` with one `ProcessBinariesConfig`.
   - all runtime readers consume `cfg.process.binaries.{pg_ctl,initdb,pg_rewind,pg_basebackup}` directly:
     - `src/process/cluster.rs:182-278`

2. The parser and docs still act as if an extra intermediate object exists.
   - `BinaryResolutionConfig` / `ProcessConfigInput` are only there to peel away `overrides`.
   - `resolve_binary_path(...)` reports `process.binaries.overrides.*` field names even though the validated type exposes `process.binaries.*`.

3. Fixtures and docs duplicate the legacy nesting.
   - `src/config_v2/parser/load_config.rs` test TOML constants still emit `process.binaries.overrides.*`.
   - HA fixture rendering in `tests/ha/support/givens/mod.rs` emits a whole `[process.binaries.overrides]` block that only exists for that adapter.

### Execution plan

1. Collapse the config shape onto the existing shared owner in `src/config_v2/types.rs`.
   - Delete the manual `Deserialize` impl for `ProcessConfig`.
   - Delete `BinaryResolutionConfig`.
   - Delete `ProcessConfigInput`.
   - Make `ProcessConfig` derive `Deserialize` directly with:
     - `timeouts`
     - `working_root`
     - `binaries: ProcessBinariesConfig`
   - Keep using the existing `ProcessBinariesConfig`; do not create `ProcessConfigV2`, `BinaryOverridesConfig`, or any new adapter DTO.

2. Rewrite binary finalization to use the direct field names only.
   - In `src/config_v2/parser/load_config.rs`, update `ProcessBinariesConfig::finalize(...)` to resolve:
     - `process.binaries.pg_ctl`
     - `process.binaries.initdb`
     - `process.binaries.pg_rewind`
     - `process.binaries.pg_basebackup`
   - Keep the same path normalization and executable validation behavior.
   - If a small helper reduces duplication across those four fields, reuse one helper instead of repeating per-field boilerplate.

3. Delete the wrapper from config renderers, docs, and tests.
   - Update test TOML constants in `src/config_v2/parser/load_config.rs`.
   - Update CLI-facing config fixtures in `tests/cli_binary.rs`.
   - Update HA fixture rendering in `tests/ha/support/givens/mod.rs`.
   - Update docs to present only `[process.binaries]` and direct `process.binaries.*` field paths.
   - Remove any wording that frames these values as “overrides”; after this slice they are simply the binary paths.

4. Keep failure surfaces concrete and current.
   - Validation errors must mention the direct field names that users now configure.
   - Any tests asserting error strings or rendered config snippets must be updated to the new direct paths.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff is net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- No backwards compatibility for `[process.binaries.overrides]`; remove it outright instead of supporting both shapes.
- Reuse `ProcessConfig` and `ProcessBinariesConfig`; do not add a new wrapper or bridge type.
- Preserve current executable discovery behavior when a binary path is omitted.
- Preserve current validation quality for missing or non-executable files.
- Do not touch unrelated binary-discovery systems outside this config-v2/runtime surface unless they are required to keep tests green.

### Expected yield

- Delete the adapter-only `ProcessConfig` deserialization layer from `src/config_v2/types.rs`.
- Shorten `load_config.rs` by removing legacy field-path duplication.
- Shrink docs/tests/helpers by removing repeated `.overrides` nesting.
- Leave one shared binary shape in both parsed config and runtime config: `process.binaries.*`.

NOW EXECUTE
