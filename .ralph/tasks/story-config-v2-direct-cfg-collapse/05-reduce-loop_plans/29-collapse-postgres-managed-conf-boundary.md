## Plan: Collapse Postgres Managed Config Boundary

### Why this reduction target

`src/postgres_managed.rs` already owns managed Postgres file materialization, but it still reconstructs a second config DTO and then hands that DTO to a sibling module that only renders the same data back into files:

- `materialize_managed_postgres_config()` derives managed paths, TLS state, start validation, replica passfile contents, and recovery signal decisions.
- It then rebuilds those same facts as `ManagedPostgresConf`.
- `src/postgres_managed_conf.rs` matches on `ProcessJobKind` again, validates the same start-only fields again, and returns a second error enum.
- `src/postgres_managed.rs` immediately maps that second error enum back into `ManagedPostgresError`.

This is still a courier boundary: one owner module knows everything, serializes it into another local shape, then decodes the same decisions on the other side.

### Current overlap already verified

- `src/postgres_managed.rs` imports `ManagedPostgresConf`, `ManagedPostgresConfError`, `ManagedPostgresTlsConfig`, `ManagedRecoverySignal`, `managed_recovery_signal_for_start_job()`, `managed_standby_passfile_path()`, and rendering constants from `src/postgres_managed_conf.rs`.
- `materialize_managed_postgres_config()` in `src/postgres_managed.rs` builds a `ManagedPostgresConf` instance solely to call `render_managed_postgres_conf(...)`, then converts `ManagedPostgresConfError` back through `map_managed_conf_error()`.
- `src/process/planner.rs`, `src/process/state.rs`, `src/process/worker.rs`, and `src/process/cluster.rs` import managed recovery/config constants from `postgres_managed_conf`, even though those concepts belong to the managed Postgres owner module.
- `src/postgres_managed_conf.rs` test coverage is largely render-order/quoting/recovery behavior that the owning `postgres_managed.rs` module can test directly without a synthetic `ManagedPostgresConf` fixture.

### Execution plan

1. Move the managed Postgres public boundary back into the owner module.
   - Rehome `ManagedRecoverySignal`, `ManagedPostgresTlsConfig`, `MANAGED_POSTGRESQL_CONF_NAME`, `MANAGED_STANDBY_SIGNAL_NAME`, `MANAGED_RECOVERY_SIGNAL_NAME`, and `managed_standby_passfile_path()` into `src/postgres_managed.rs`.
   - Update all imports in process modules and tests to use `crate::postgres_managed::...`.

2. Delete the extra config DTO and second error boundary.
   - Remove `ManagedPostgresConf`.
   - Remove `ManagedPostgresConfError`.
   - Replace `render_managed_postgres_conf(&ManagedPostgresConf, ...)` with an owner-local renderer that consumes the already-owned inputs directly: `&RuntimeConfigV2`, `ProcessJobKind`, `&StartPostgresSpec`, resolved managed TLS config, and managed path references.
   - Fold validation failures onto `ManagedPostgresError` directly so `map_managed_conf_error()` disappears.

3. Collapse recovery-signal and replica rendering decisions into the same owner path.
   - Move `managed_recovery_signal_for_start_job()` into `src/postgres_managed.rs`.
   - Keep `validate_extra_guc_entry()` and slot-name validation private to the owner module unless another call site still genuinely needs them.
   - Make `materialize_managed_postgres_config()` the only place that decides whether a start is primary, detached standby, or replica for managed config output.

4. Rebuild tests around the surviving owner API.
   - Move the deterministic render, setting-order, quoting/escaping, TLS, and recovery-signal assertions into `src/postgres_managed.rs` tests.
   - Replace the synthetic `sample_conf()` fixture with start specs and runtime configs built through existing test helpers.
   - Delete tests whose only purpose was proving the temporary `ManagedPostgresConf` DTO shape or its private error enum.

5. Remove the dead module and validate reduction.
   - Delete `src/postgres_managed_conf.rs`.
   - Remove its module declaration from `src/lib.rs`.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+6115 -8657 diff: -2542` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long` sequentially.

### Guardrails

- Reuse the existing owner inputs (`RuntimeConfigV2`, `ProcessJobKind`, `StartPostgresSpec`, `ManagedPostgresConfig`); do not introduce a replacement DTO just to preserve the deleted boundary.
- Keep the managed-config rendering logic private to `postgres_managed` unless a real external caller still needs a helper after the collapse.
- If deleting `postgres_managed_conf.rs` forces wider public surface area or placeholder data just to satisfy tests, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
