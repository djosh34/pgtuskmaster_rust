## Bug: Config V2 Binaries Drop Postgres And Psql Needed By HA Fixtures <status>completed</status> <passes>true</passes>

<description>
`src/config_v2/parser/load_config.rs` currently rejects `process.binaries.overrides.postgres` and `process.binaries.overrides.psql` as unsupported, and `config_v2::types::BinariesConfig` has no place to carry them. The ultra-long HA fixtures still depend on those binary overrides, so seed-primary bootstrap dies during config parsing before the cluster can start.

This was detected by `make test-long`, where multiple HA scenarios failed at harness bootstrap with `config error: invalid config field process.binaries.overrides.postgres: is not supported by config_v2`. Explore the current process/tool/runtime usage first, then extend the existing v2 binary shape directly instead of rebuilding an old process config mirror.
</description>

<acceptance_criteria>
- [x] `src/config_v2::types::BinariesConfig` carries every process binary path still legitimately needed by runtime code and HA fixtures
- [x] `src/config_v2/parser/load_config.rs` no longer rejects `process.binaries.overrides.postgres` / `psql` when those binaries are still part of the supported runtime surface
- [x] No helper rebuilds old `ProcessConfig` or introduces a new mirror such as `ProcessConfigV2`
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Boundary diagnosis
- The shared validated v2 type already carries the exact binaries this task names: [src/config_v2/types.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/types.rs) defines one `BinariesConfig` with `postgres`, `pg_ctl`, `initdb`, `pg_rewind`, `pg_basebackup`, and `psql`, and the runtime process boundary still validates `postgres` and `psql` as required absolute paths in [src/process/state.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/state.rs).
- The config-ingestion edges also already preserve those fields instead of rejecting them: the private TOML schema accepts both overrides in [src/config_v2/parser/private_schema.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/parser/private_schema.rs), the runtime loader resolves them into `RuntimeConfigV2` in [src/config_v2/parser/load_config.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/parser/load_config.rs), and the legacy-to-v2 test adapter preserves the same pair in [src/dev_support/runtime_config_v2.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dev_support/runtime_config_v2.rs).
- The HA harness fixtures still depend on those exact overrides and already render them in [tests/ha/support/world/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/world/mod.rs). That means the task description is stale or at least incomplete: the honest remaining work is to prove the current single `BinariesConfig` boundary is sufficient under execution and only patch a real surviving failure if one still exists.
- Do not introduce `ProcessConfigV2`, a second binaries DTO, or any helper that rebuilds old process config. The smallest truthful boundary is still one shared `RuntimeConfigV2.binaries` shape consumed directly by runtime and test support.

### Execution plan
1. Start with verification, not type churn: run the required gates in repo order, `make check`, `make lint`, `make test`, and `make test-long`.
2. If every gate passes, treat this bug as already fixed on the current branch. Tick the acceptance boxes that are already true, set `<passes>true</passes>`, switch the task, commit, and push without inventing extra config shapes.
3. If execution still fails with this bug's symptom, patch the existing ingestion boundary directly instead of adding mirrors:
   - keep `postgres` and `psql` on the existing shared `BinariesConfig`;
   - fix the exact failing edge among the runtime loader, the legacy-to-v2 adapter, or a stale validation/helper path;
   - delete any leftover rejection/helper that implies those fields are unsupported when runtime still requires them.
4. Lock the boundary with focused regression coverage only where execution proves it is missing:
   - a loader test proving `process.binaries.overrides.postgres` and `process.binaries.overrides.psql` survive into `RuntimeConfigV2`;
   - a legacy-to-v2 adapter test proving the same pair survives conversion for HA/dev-support callers;
   - no new mirror types or broad fixture rewrites unless failure evidence requires them.
5. If execution proves the current diagnosis is wrong in a deeper way, switch this task back to `TO BE VERIFIED`, explain the exact missing or duplicated binary boundary in this file, and stop immediately.
6. Only after every required gate passes:
   - tick the acceptance boxes that are truly complete,
   - set `<passes>true</passes>`,
   - run `/bin/bash .ralph/task_switch.sh`,
   - commit all changes, including `.ralph` updates, with `task finished [task name]: ...`,
   - push with `git push`,
   - stop immediately.

NOW EXECUTE
