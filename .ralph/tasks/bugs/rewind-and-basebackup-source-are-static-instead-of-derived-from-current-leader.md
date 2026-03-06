## Bug: Rewind and basebackup source stay static instead of deriving from the current leader <status>completed</status> <passes>true</passes>

<blocked_by>01-task-remove-backup-config-and-process-surface,02-task-remove-runtime-restore-bootstrap-and-archive-helper-wiring,04-task-remove-backup-harness-installers-and-gate-selection,05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>

<description>
Runtime and HA process dispatch still build `basebackup_source` and `rewind_source` once from static config fields (`postgres.rewind_source_host` / `postgres.rewind_source_port`) and then reuse those fixed values for clone and rewind operations.

This is the wrong ownership boundary for an HA system. The source node for `pg_basebackup` and `pg_rewind` should be derived from current runtime/DCS leader facts at the moment the action is planned or dispatched, not from a static host/port configured in advance. The current design makes the config surface misleading and can target the wrong source after leadership changes.

The agent must explore the current runtime, HA, config, and test code first, then fix the behavior cleanly rather than layering another partial override on top.

Required investigation/fix shape:
- trace how `src/runtime/node.rs` constructs `ProcessDispatchDefaults.basebackup_source` and `rewind_source`
- trace how `src/ha/process_dispatch.rs` uses those static defaults for `HaAction::StartBaseBackup` and `HaAction::StartRewind`
- decide the correct source-of-truth for leader connection target selection using current DCS/runtime facts and existing leader/member metadata
- remove or redesign the misleading static config surface for rewind source host/port if it is no longer legitimate
- keep role-specific credential selection intact: basebackup must still use replicator credentials and rewind must still use rewinder credentials
- add/adjust tests so leadership changes prove the source host/port is derived dynamically rather than frozen from config

This should be scheduled after the backup-removal story so the runtime startup/restore surface is simpler, but it should not be buried inside the authoritative `pgtm.postgresql.conf` story because it is about HA remote-source selection, not local managed config-file ownership.
</description>

<acceptance_criteria>
- [x] Runtime/HA no longer rely on static `postgres.rewind_source_host` / `postgres.rewind_source_port` for choosing rewind/basebackup source after startup
- [x] `pg_basebackup` source target is derived from the current leader/member state at action time
- [x] `pg_rewind` source target is derived from the current leader/member state at action time
- [x] Role separation remains correct: basebackup uses replicator credentials, rewind uses rewinder credentials
- [x] Config surface no longer advertises a misleading static rewind-source target if that target is now dynamic
- [x] Tests prove source selection follows leadership changes and does not stay pinned to stale config
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Plan

1. Confirm the actual source-of-truth boundary and remove the current misleading one.
   - `src/runtime/node.rs` currently builds `ProcessDispatchDefaults.basebackup_source` and `ProcessDispatchDefaults.rewind_source` from `cfg.postgres.rewind_source_host` / `cfg.postgres.rewind_source_port`.
   - `src/ha/process_dispatch.rs` then reuses those frozen values for `HaAction::StartBaseBackup` and `HaAction::StartRewind`.
   - The clean end-state is:
     - static config keeps only local node settings plus role/auth/identity defaults that are legitimately static
     - remote source host/port comes from the current leader/member record at the moment startup clone or HA process dispatch is planned
   - Do not patch this by layering another override on top of the existing static source fields.

2. Extend DCS member metadata so each member advertises its current PostgreSQL endpoint.
   - Update `src/dcs/state.rs` `MemberRecord` to include the member PostgreSQL endpoint fields needed for remote source selection.
     - use the existing local node values (`listen_host`, `listen_port`) as the published member endpoint
     - update `build_local_member_record` to take the local runtime config endpoint and write it into the member record
   - Update all encode/decode and test fixtures that construct or deserialize `MemberRecord`:
     - `src/dcs/store.rs`
     - `src/dcs/worker.rs`
     - `src/dcs/etcd_store.rs`
     - `src/ha/decide.rs`
     - `src/ha/worker.rs`
     - `src/runtime/node.rs`
     - any other test-only `MemberRecord` builders returned by `rg`
   - Because this is a greenfield project with no backwards-compatibility requirement, do not preserve the old member-record schema.

3. Redesign `ProcessDispatchDefaults` so it keeps only static defaults that still make sense.
   - Update `src/ha/state.rs` so `ProcessDispatchDefaults` no longer stores precomputed remote source connections.
   - Replace the frozen `basebackup_source` / `rewind_source` fields with only the static pieces needed to assemble source conninfo at action time:
     - local postgres host/port/socket/log/shutdown settings
     - replicator auth + replicator username default
     - rewinder auth + rewinder username default
     - shared remote connection identity defaults that still belong in config (`rewind_conn_identity`, connect timeout, ssl mode, dbname)
   - Keep role separation explicit so basebackup still uses replicator credentials and rewind still uses rewinder credentials.

4. Split leader selection from connection assembly so startup and HA do not accidentally drift into the same policy.
   - Implement a small shared helper that only assembles a remote source connection from:
     - a chosen `MemberRecord` that already contains the leader PostgreSQL endpoint
     - the static role/identity defaults from `ProcessDispatchDefaults`
   - The shared helper must:
     - reject self as a remote source
     - return a `ReplicatorSourceConn` for basebackup and a `RewinderSourceConn` for rewind with:
       - host/port from the chosen member record
       - user/auth from the correct role
       - dbname/ssl/connect-timeout from the configured connection identity defaults
   - Do not hide leader selection inside this helper. Startup and HA have different selection inputs and should stay explicit.

5. Update startup clone planning to use the leader member endpoint instead of a frozen config target.
   - In `src/runtime/node.rs`, change startup-mode selection so `StartupMode::CloneReplica` is built from the selected leader member record endpoint, not from `default_leader_source(process_defaults, ...)`.
   - The startup planner should continue to:
     - use the leader key when present
     - fall back to a healthy primary member when the leader key is absent but init-lock/member evidence shows the cluster is already initialized
   - Keep the startup-only selector explicit in `runtime/node.rs`; it should return the chosen leader member record (or member id plus record), then feed that into the shared connection-assembly helper from step 4.
   - Add or update unit tests in `src/runtime/node.rs` so clone startup proves:
     - the chosen source host/port comes from the leader member record
     - changing the selected leader/member changes the derived source
     - role-specific user selection remains correct for startup basebackup

6. Preserve the chosen HA recovery target through lowering/apply and resolve that exact member at dispatch time.
   - Extend the HA action surface so `HaAction::StartBaseBackup` and `HaAction::StartRewind` carry the `leader_member_id` already chosen in `RecoveryStrategy`.
   - Update `src/ha/apply.rs` to preserve that `leader_member_id` when converting the effect plan into concrete actions.
   - In `src/ha/process_dispatch.rs`, stop using `ctx.process_defaults.basebackup_source` and `ctx.process_defaults.rewind_source`.
   - Build the `BaseBackupSpec` / `PgRewindSpec` source by:
     - reading `ctx.dcs_subscriber.latest()`
     - resolving the action's `leader_member_id` to a current member record
     - feeding that member record into the shared connection-assembly helper from step 4
   - If the targeted member is missing, unhealthy, self-referential, or lacks endpoint data, return an explicit `ProcessDispatchError` instead of silently dispatching a bad job.
   - Add dispatch tests that prove:
     - basebackup uses the targeted leader member endpoint and replicator credentials
     - rewind uses the targeted leader member endpoint and rewinder credentials
     - the dispatched job follows the `leader_member_id` selected by HA recovery, not an unrelated cache leader value
     - changing the targeted member record between dispatches changes the emitted job source host/port
     - missing, invalid, or self-referential target members are rejected with a clear error

7. Remove the misleading static config surface for rewind/basebackup source targeting.
   - Delete `rewind_source_host` and `rewind_source_port` from:
     - `src/config/schema.rs`
     - `src/config/parser.rs`
     - v2/v1 input structs and validation code
     - config examples, parser tests, CLI tests, and any runtime-config fixtures created only to populate those fields
   - Update all sample/test config constructors across the repo to stop providing those fields.
   - Because this project intentionally does not support legacy compatibility, remove the fields outright instead of deprecating them.

8. Update docs to describe the new behavior and remove stale operator guidance.
   - Update `docs/src/operator/configuration.md` to remove the old fields from examples and field descriptions.
   - Update `docs/src/operator/troubleshooting.md` so rewind/bootstrap troubleshooting points operators at leader/member endpoint visibility and replication/rewinder auth, not a static `rewind_source_host:rewind_source_port`.
   - Remove any other stale documentation/examples found by `rg`.

9. Verify the full repo after implementation.
   - Run and fix forward until all pass:
     - `make check`
     - `make test`
     - `make test-long`
     - `make lint`
   - Tick the acceptance criteria boxes and set `<passes>true</passes>` only after all commands pass cleanly.

## Expected Code Shape After Execution

- `MemberRecord` is the cluster source-of-truth for each member's remote PostgreSQL endpoint.
- Runtime startup clone planning and HA dispatch both derive remote source host/port from current DCS facts instead of config-time literals.
- Static config continues to own credentials/identity defaults, but not remote leader selection.
- `postgres.rewind_source_host` / `postgres.rewind_source_port` no longer exist anywhere in schema, parser, docs, or tests.

NOW EXECUTE
