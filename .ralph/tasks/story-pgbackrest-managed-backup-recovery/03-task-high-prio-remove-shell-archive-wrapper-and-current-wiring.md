---
## Task: High Prio Remove Shell Archive Wrapper and Current Wiring <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Completely remove the generated shell archive wrapper implementation and all runtime wiring that depends on it.

**Scope:**
- Remove the wrapper module and script generation path in [src/logging/archive_wrapper.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/archive_wrapper.rs).
- Remove managed-postgres wiring that injects wrapper-based `archive_command` / `restore_command` in [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs).
- Remove or update ingest assumptions that rely on `logging.postgres.archive_command_log_file` as wrapper output source in [src/logging/postgres_ingest.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/postgres_ingest.rs).
- Remove obsolete docs describing generated shell wrapper behavior in operator docs.
- Keep backup execution paths functional where possible, but do not keep any legacy wrapper compatibility paths.

**Context from research:**
- Current implementation writes `pgtuskmaster-pgbackrest-wal-wrapper.sh`, then sets Postgres commands to call it.
- Wrapper builds JSON in shell and appends directly to archive log JSONL; this is the behavior being intentionally deleted.
- This removal task is intentionally destructive to clear the slate before reintroducing a Rust-native mechanism.

**Expected outcome:**
- No generated shell wrapper exists in code or docs.
- No runtime path writes or executes `pgtuskmaster-pgbackrest-wal-wrapper.sh`.
- No config/docs promise wrapper-produced archive JSON lines.
- Codebase compiles and test suite passes without this mechanism.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Remove [src/logging/archive_wrapper.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/archive_wrapper.rs) and module exports/imports that reference it.
- [ ] Update [src/logging/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/mod.rs) to remove `archive_wrapper` module exposure and any related dead code.
- [ ] Update [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs) to remove wrapper creation and wrapper-based `archive_command`/`restore_command` insertion.
- [ ] Update [src/logging/postgres_ingest.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/postgres_ingest.rs) to remove dependencies on wrapper-produced archive file semantics or to gate behavior so no stale promises remain.
- [ ] Remove/update config schema/defaults/docs references that specifically require `logging.postgres.archive_command_log_file` because of the shell wrapper.
- [ ] Update operator documentation files that currently describe generated wrapper behavior, including [docs/src/operator/observability.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/operator/observability.md) and [docs/src/operator/configuration.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/operator/configuration.md), so they do not mention shell wrapper ownership.
- [ ] Remove/replace tests that validate shell wrapper generation/execution (currently in deleted module tests), preserving coverage for remaining behavior.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
