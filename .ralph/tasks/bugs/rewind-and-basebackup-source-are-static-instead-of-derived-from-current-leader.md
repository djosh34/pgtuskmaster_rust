---
## Bug: Rewind and basebackup source stay static instead of deriving from the current leader <status>not_started</status> <passes>false</passes>

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
- [ ] Runtime/HA no longer rely on static `postgres.rewind_source_host` / `postgres.rewind_source_port` for choosing rewind/basebackup source after startup
- [ ] `pg_basebackup` source target is derived from the current leader/member state at action time
- [ ] `pg_rewind` source target is derived from the current leader/member state at action time
- [ ] Role separation remains correct: basebackup uses replicator credentials, rewind uses rewinder credentials
- [ ] Config surface no longer advertises a misleading static rewind-source target if that target is now dynamic
- [ ] Tests prove source selection follows leadership changes and does not stay pinned to stale config
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
