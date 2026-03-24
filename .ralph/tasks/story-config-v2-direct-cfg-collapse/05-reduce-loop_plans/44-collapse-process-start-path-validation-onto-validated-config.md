## Plan: Collapse Process Start Path Validation Onto Validated Config

### Why this reduction target

`src/process/cluster.rs`, `src/process/state.rs`, and `src/runtime/node.rs` still carry startup validation and path-preparation logic that duplicates guarantees already established by `config_v2` and the local process source-materialization boundary:

- `resolve_binary_path(...)` in `src/config_v2/parser/load_config.rs` already guarantees every runtime binary path points to a real file, yet `ensure_start_paths(...)` in `src/process/state.rs` re-checks the same six binary paths and is called from both `src/runtime/node.rs` and `src/process/cluster.rs`.
- `resolve_path_only(...)` and the runtime config mapping already reject empty path inputs for path-backed fields, yet `build_bootstrap_command(...)`, `build_basebackup_command(...)`, `build_pg_rewind_command(...)`, `build_promote_command(...)`, `build_demote_command(...)`, and `build_start_postgres_command(...)` all repeat `validate_non_empty_path(...)` against `cfg.postgres.data_dir`, `cfg.postgres.log_file`, and the derived managed config path.
- `source_from_member(...)` already rejects self-targets, non-primary members, and empty postgres hosts before constructing `SourceConn`, while `config_v2` already guarantees non-empty role usernames and `postgres.local_database`; `validate_source_conninfo(...)` then re-validates the same `SourceConn` shape right before rendering `pg_basebackup` and `pg_rewind` commands.
- `ensure_start_paths(...)` lives in the process state module even though it is neither state nor command planning; it mixes binary-path validation, directory creation, and unix permission setup into a public helper that exists only to defend against already-validated config.

This is the exact boundary smell from `improve-code-boundaries`: validated config and typed source materialization already own these invariants, but the process layer keeps re-checking them instead of trusting one authoritative shape.

### Current overlap already verified

- `src/runtime/node.rs` calls `ensure_start_paths(cfg)` during startup.
- `src/process/cluster.rs` calls `ensure_start_paths(cfg)` again from `materialize_start_config(...)`.
- `src/process/cluster.rs` also repeats `validate_non_empty_path(...)` seven times across bootstrap, basebackup, pg_rewind, promote, demote, and start-postgres command builders.
- `src/process/cluster.rs` repeats `validate_source_conninfo(...)` in both `build_basebackup_command(...)` and `build_pg_rewind_command(...)`.
- `src/process/cluster.rs::source_from_member(...)` already constructs `SourceConn` from validated config fields and rejects the only source-member shape problem that `validate_source_conninfo(...)` still checks locally: an empty host.
- `src/config_v2/parser/load_config.rs` already validates non-empty `postgres.local_database`, role usernames, path-only fields, and resolves each runtime binary path once.

### Execution plan

1. Delete the stale process-state startup validation helper.
   - Remove `ensure_start_paths(...)` from `src/process/state.rs`.
   - Stop importing it from `src/process/cluster.rs` and `src/runtime/node.rs`.
   - Keep startup directory creation in exactly one runtime-start path instead of re-running it from both runtime bootstrap and managed-start config materialization.

2. Collapse command builders onto validated config and typed source inputs.
   - Delete `validate_non_empty_path(...)` and remove its call sites from the process command builders.
   - Delete `validate_source_conninfo(...)` and its private endpoint helper once `SourceConn` becomes the authoritative boundary for basebackup/rewind source materialization.
   - Preserve current user-facing command rendering and error behavior for real runtime failures; only remove redundant spec checks that defend against states config parsing and `source_from_member(...)` already forbid.

3. Keep only one filesystem-preparation boundary.
   - Either inline the existing directory creation into `run_node_from_config(...)` or extract a private runtime-start helper there.
   - Preserve creation of the data dir parent, the data dir itself, unix `0o700` permissions for the data dir, the socket dir, and the postgres log parent directory.
   - Do not reintroduce another public courier/helper in `process` just to move the same `cfg.postgres.*` paths around.

4. Rewrite tests around the remaining authoritative boundaries.
   - Update or add focused tests in `src/process/cluster.rs` and `src/runtime/node.rs` for the surviving runtime-start path preparation behavior and for command construction from `SourceConn`.
   - Remove tests that only exist to exercise deleted redundant validators, unless they still prove observable behavior at the new boundary.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+7621 -10782 diff: -3161` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Reuse `RuntimeConfigV2`, `PostgresConfig`, `BinariesConfig`, and `SourceConn`; do not add a new validated path DTO or source-conn wrapper just to preserve the old layering.
- If one of the deleted checks is still guarding a state that can be constructed without going through `config_v2` or `source_from_member(...)`, switch this plan back to `TO BE VERIFIED` instead of silently weakening the runtime contract.
- Do not move validation logic into random call sites; the reduction should remove duplicate guards, not scatter them.

NOW EXECUTE
