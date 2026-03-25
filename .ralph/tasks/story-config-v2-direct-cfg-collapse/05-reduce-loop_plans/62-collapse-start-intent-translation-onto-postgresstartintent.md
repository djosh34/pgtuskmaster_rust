## Plan: Collapse Start Intent Translation Onto PostgresStartIntent

### Why this is the next reduction target

The process start pipeline still re-encodes the same three PostgreSQL start modes across three modules:

- `src/process/jobs.rs` owns `PostgresStartIntent::{Primary, DetachedStandby, Replica { leader }}` and immediately maps it onto tracked `ProcessJobKind::{StartPrimary, StartDetachedStandby, StartReplica}`.
- `src/process/cluster.rs` then pattern-matches the same intent again in `prepare_start_postgres_launch`, derives the leader/source requirements again, and passes a widened `materialize_managed_postgres_config` API based on `ProcessJobKind` plus optional `primary_conninfo` and `primary_slot_name`.
- `src/postgres_managed.rs` then pattern-matches the start mode a third time by converting `ProcessJobKind` back into `ManagedStartJob`, `ManagedReplicaSource`, and `ManagedRecoverySignal` before rendering the same managed config and signal files.

That is boundary duplication, not domain complexity. The start-mode owner already exists: `PostgresStartIntent`. The managed-config writer should consume that owner directly instead of accepting a broader, partially validated API and rebuilding another DTO layer.

### Current overlap already verified

- `src/process/jobs.rs` still has `PostgresStartIntent::job_kind()` solely to map start variants onto `ProcessJobKind`, while `ProcessIntent::command_job_kind()` maps those same variants back onto the one executable command kind `StartPostgres`.
- `src/process/cluster.rs` matches `PostgresStartIntent` in `prepare_start_postgres_launch`, rejects primary starts against observed managed recovery state, resolves replica sources, and then widens the result into `materialize_start_config(cfg, start_intent.job_kind(), source_conninfo, None)`.
- `src/postgres_managed.rs` still defines `ManagedStartJob` and `ManagedReplicaSource` only to reconstitute start-mode facts that were already known in `PostgresStartIntent`.
- `resolve_managed_start_job` in `src/postgres_managed.rs` repeats the same branching over `ProcessJobKind::{StartPrimary, StartDetachedStandby, StartReplica}`, validates replica-only fields again, and returns `invalid_managed_start_job_kind` for non-start kinds even though the only production caller is the start path in `src/process/cluster.rs`.
- `primary_slot_name` is still threaded through `materialize_managed_postgres_config`, but the production call path in `src/process/cluster.rs` currently passes `None`; that parameter is carrying speculative surface area rather than real behavior.

### Execution plan

1. Make `PostgresStartIntent` the single owner of planned start semantics.
   - Add narrow helpers on the existing `PostgresStartIntent` type for the facts that are intrinsic to the start mode: tracked job kind, whether `hot_standby` should be on, and which managed recovery signal should be materialized.
   - Keep `ProcessJobKind::{StartPrimary, StartDetachedStandby, StartReplica}` only for tracking/outcomes that already depend on those distinct job kinds.
   - Do not introduce a new “validated start plan” wrapper unless direct helpers on `PostgresStartIntent` prove insufficient.

2. Narrow the managed config materialization API to the real start owner.
   - Change `materialize_managed_postgres_config` to accept `&PostgresStartIntent` instead of a generic `ProcessJobKind`.
   - Replace the current `(tracked_job_kind, primary_conninfo, primary_slot_name)` interface with the smallest surface the writer actually needs from callers.
   - If replica starts still need a leader-derived conninfo, pass only that canonical conninfo input and delete the unused slot-name plumbing if no production behavior depends on it.

3. Delete the shadow start DTOs inside `src/postgres_managed.rs`.
   - Remove `ManagedStartJob`, `ManagedReplicaSource`, `resolve_managed_start_job`, `reject_replica_source_fields`, and `invalid_managed_start_job_kind`.
   - Rewrite `render_managed_postgres_conf` and recovery-signal materialization to branch directly on `PostgresStartIntent` plus the already-resolved replica conninfo when applicable.
   - Keep `ManagedRecoverySignal` only where it represents observed on-disk state, not as a second planning API for future starts.

4. Keep leader/source resolution in `src/process/cluster.rs`, but stop remapping start mode there.
   - Let `prepare_start_postgres_launch` continue to own DCS/observed-state checks and source resolution, because those facts live with process preparation.
   - After source resolution, pass `start_intent` directly into the managed-config writer instead of translating it through `ProcessJobKind`.
   - Remove any start-path glue that only exists to satisfy the widened managed-config API.

5. Shrink tests around the collapsed boundary.
   - Update `src/postgres_managed.rs` tests to build around `PostgresStartIntent` directly instead of `ProcessJobKind` plus optional replica fields.
   - Update `src/process/cluster.rs` tests to assert that start preparation still yields the same tracked job kinds and the same command job kind, while no longer depending on the removed translation layer.
   - Remove test helpers that only exist to exercise `resolve_managed_start_job` or the dead slot-name surface if those APIs are deleted.

6. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to stay net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Reuse the existing `PostgresStartIntent` and `ManagedRecoverySignal` types; do not replace one shadow DTO layer with another.
- Preserve the current tracked job-kind distinctions because reconciliation/state logic still keys off `StartPrimary`, `StartDetachedStandby`, and `StartReplica`.
- Keep DCS leader resolution and observed-state rejection in `src/process/cluster.rs`; the reduction target is translation duplication, not moving cluster knowledge into config rendering.
- If replica slot support turns out to matter outside tests, keep the behavior but re-home it on the true start owner rather than preserving dead optional courier parameters.

### Expected yield

- Delete the `ManagedStartJob` / `ManagedReplicaSource` layer from `src/postgres_managed.rs`.
- Remove a full extra round of start-mode pattern matching between `src/process/cluster.rs` and `src/postgres_managed.rs`.
- Narrow `materialize_managed_postgres_config` to the behavior the application actually uses today.
- Reduce duplicated tests and helper surface that only exists to feed the shadow start-translation API.

NOW EXECUTE
