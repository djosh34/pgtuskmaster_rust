## Plan: Collapse Managed Postgres Start Branching And Path Couriers

### Why this reduction target

`src/postgres_managed.rs` still spends a lot of lines restating one local startup workflow through extra couriers and repeated `ProcessJobKind` matching:

- `ManagedRenderPaths` exists only inside this file to shuttle three already-known paths from `materialize_managed_postgres_config(...)` into `render_managed_postgres_conf(...)` and test literals.
- `materialize_managed_standby_passfile(...)` returns `Option<PathBuf>`, but its caller ignores that value completely; the real contract is only the side effect on disk.
- `materialize_managed_standby_passfile(...)`, `render_managed_postgres_conf(...)`, and `managed_recovery_signal_for_start_job(...)` each rematch `StartPrimary` / `StartDetachedStandby` / `StartReplica`, re-check replica-source requirements, and reformat the same invalid start-job error.
- The tests rebuild the same `ManagedRenderPaths { ... }` literals and near-identical `render_managed_postgres_conf(...)` calls for replica, primary, and detached-standby scenarios.

That is the exact overabstraction smell for this loop: one owner already has the real facts, but it keeps projecting them into extra local wrappers and duplicate branch trees instead of running a single direct managed-start flow.

### Current overlap already verified

- `ManagedRenderPaths` is declared once in `src/postgres_managed.rs` and only used by this file plus its local tests.
- `materialize_managed_standby_passfile(...)` is only called from `materialize_managed_postgres_config(...)`, and the returned `Option<PathBuf>` is never consumed.
- The message `managed postgres config requires a start job kind, got ...` appears in three separate start-kind helpers.
- The replica-only `primary_conninfo` requirement is checked in both passfile materialization and config rendering.
- Local tests construct `ManagedRenderPaths { ... }` repeatedly even though the paths are deterministic from `cfg.postgres` and `data_dir`.

### Execution plan

1. Delete the local path courier and ignored return value.
   - Remove `ManagedRenderPaths`.
   - Change the managed render/materialize flow to pass only direct paths or derive them from `cfg.postgres` / `data_dir` where needed.
   - Change `materialize_managed_standby_passfile(...)` to return `Result<(), ManagedPostgresError>` instead of an unused `Option<PathBuf>`.

2. Collapse `ProcessJobKind` branching onto one authoritative managed-start path.
   - Compute the managed-start facts once: recovery signal, whether `hot_standby` should be on, and whether replica source fields are required or forbidden.
   - Reuse that single decision path for passfile materialization, `primary_conninfo` rendering, and recovery-signal file materialization.
   - Preserve the current observable behavior for primary, detached standby, and replica starts.

3. Rewrite tests around observable start modes instead of local couriers.
   - Replace repeated `ManagedRenderPaths { ... }` literals and near-identical render calls with one shared test helper per rendered start mode.
   - Keep coverage for replica `primary_conninfo` / `primary_slot_name`, primary and detached-standby omission of replica-only fields, and recovery-signal behavior.

4. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+7618 -10772 diff: -3154` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not replace `ManagedRenderPaths` or the ignored `Option<PathBuf>` with another renamed local courier.
- Reuse `ProcessJobKind` and the existing `ManagedRecoverySignal` semantics instead of inventing a new start-mode enum unless the code reduction clearly justifies it.
- If collapsing the start-job logic requires exporting a new cross-module wrapper type just to move the same facts around, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
