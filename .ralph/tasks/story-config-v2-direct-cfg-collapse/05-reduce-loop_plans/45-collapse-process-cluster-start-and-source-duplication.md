## Plan: Collapse Process Cluster Start And Source Duplication

### Why this reduction target

`src/process/cluster.rs` still keeps one launch-preparation workflow spread across thin duplicate helpers and repeated match branches:

- `prepare_process_launch(...)` repeats the same `prepared_launch(...)` wrapping in every successful branch even though the tracked job kind and timeout already come from `request.intent.job_kind()`.
- The three `ProcessIntent::Start(...)` arms all rebuild the same start-postgres command path; the only real variation is whether a leader-derived source is required and whether existing managed standby state must reject a primary start.
- `basebackup_source_from_leader(...)` and `rewind_source_from_leader(...)` are the same function with only `SourceCredentialKind` changed.
- `build_basebackup_command(...)` and `build_pg_rewind_command(...)` both render a remote-source process command from the same `SourceConn` boundary and the same role-auth environment.

That is boundary duplication, not distinct behavior. The authoritative state is already `ProcessIntent`, `SourceConn`, and `RuntimeConfigV2`; the file keeps re-projecting those facts through near-identical helpers and match arms.

### Current overlap already verified

- `prepare_process_launch(...)` in `src/process/cluster.rs` computes `tracked_job_kind` once, then repeats `prepared_launch(request.id.clone(), tracked_job_kind, default_timeout_ms(cfg, tracked_job_kind), command)` in every success path.
- `ProcessIntent::Start(PostgresStartIntent::Primary)`, `DetachedStandby`, and `Replica { leader }` all end by materializing managed config and calling `build_start_postgres_command(cfg)`.
- `basebackup_source_from_leader(...)` and `rewind_source_from_leader(...)` both call `resolve_source_member(...)` and then `source_from_member(...)`; only the credential kind differs.
- `build_basebackup_command(...)` and `build_pg_rewind_command(...)` both use `source.conninfo.to_string()`, `role_auth_env(&source.auth)`, `cfg.logging.capture_subprocess_output`, and a job-kind-specific program/flag pair.

### Execution plan

1. Collapse leader-source selection onto one helper.
   - Replace `basebackup_source_from_leader(...)` and `rewind_source_from_leader(...)` with one `source_from_leader(...)` that accepts `SourceCredentialKind`.
   - Keep `resolve_source_member(...)` and `source_from_member(...)` as the only source-validation boundaries; do not add a new wrapper type.

2. Collapse the repeated prepared-launch wrapping.
   - Add one thin helper that takes `request`, `cfg`, and the finished `ProcessCommandSpec`, then derives the tracked job kind and timeout once.
   - Rebuild the intent match so each branch only performs the behavior unique to that intent before handing off to the shared launch finisher.

3. Collapse the repeated start-postgres branch.
   - Move the `Primary` / `DetachedStandby` / `Replica` variation into one local start-preparation helper that:
     - rejects primary start when existing managed standby recovery state is present,
     - resolves a replica leader source only for the replica case,
     - materializes managed config once with the appropriate optional `primary_conninfo`,
     - builds the shared start-postgres command once.
   - Preserve current error strings and tracked job kinds.

4. Collapse the duplicated remote-source command shell only if it clearly deletes lines.
   - Reuse `SourceConn` and `role_auth_env(...)`.
   - If a tiny local helper can build both pg_basebackup and pg_rewind commands with less code, use it.
   - If that helper starts adding more scaffolding than it removes, keep the command builders separate and finish the slice with steps 1-3 only.

5. Rebuild focused tests in `src/process/cluster.rs`.
   - Keep coverage for primary-start rejection, replica start materialization, basebackup/rewind source credential selection, and intent/job-kind mapping.
   - Prefer updating existing tests over adding new helper-heavy test scaffolding.

6. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+7700 -10883 diff: -3183` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not introduce a new start DTO, enum, or planner courier just to share code inside `src/process/cluster.rs`.
- Keep `SourceConn` as the source-materialization boundary instead of reintroducing stringly source validation near command rendering.
- If collapsing the start branches forces a helper that needs to thread placeholder values for non-replica cases, switch this plan back to `TO BE VERIFIED`.
- If the optional remote-source command helper is not a net line reduction, skip that substep instead of abstracting for its own sake.

NOW EXECUTE
