## Bug: Config V2 Drops Managed Postgres Inputs Needed By Process Reduction <status>completed</status> <passes>true</passes>

<description>
The direct-cfg reduction for the daemon runtime cannot finish correctly because `RuntimeConfigV2` currently drops static config that surviving startup/process helpers still need.

Concrete evidence from execution:
- `src/postgres_roles.rs` needs `postgres.local_database` for managed role reconciliation.
- `src/postgres_managed.rs` still needs the authoritative `postgres.access.hba` and `postgres.access.ident` source data to materialize managed files.
- The raw/private v2 schema still contains at least `postgres.local_database`, but `src/config_v2/types.rs` does not carry it forward, and the managed-postgres source contents are also not available on the v2 type graph.

Explore and research the codebase first, then fix the v2 type design so the process/materialization/runtime-root reduction can stay on config-v2 only without rebuilding `crate::config::RuntimeConfig`, hard-coding defaults, or keeping old-config mirrors alive.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Boundary diagnosis
- The current validated v2 type graph already carries the data this bug describes: [src/config_v2/types.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/types.rs) has `postgres.local_database` plus the managed HBA/ident file paths and resolved contents, [src/config_v2/parser/load_config.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/parser/load_config.rs) populates them from the private TOML schema, and [src/dev_support/runtime_config_v2.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dev_support/runtime_config_v2.rs) preserves the same fields for test/harness conversion.
- The known v2 consumers also already read those values directly from `RuntimeConfigV2`: [src/postgres_roles.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_roles.rs) and [src/process/source.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/source.rs) use `cfg.postgres.local_database`, and [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs) materializes HBA/ident files from the resolved v2 contents without rebuilding `crate::config::RuntimeConfig`.
- That means the task description is at least partially stale. The remaining honest boundary work is not "invent another config-v2 mirror", but to prove the current shared type is sufficient, remove any leftover stale expectations, and only patch a real regression if execution finds one.
- Do not add a new `ProcessConfigV2`, `ManagedPostgresConfigV2`, or another nested access DTO unless execution proves the current `PostgresConfig` shape still leaks duplication. The current evidence says the existing flat validated fields are already the smallest truthful runtime surface.

### Execution plan
1. Start with verification before refactoring: run the required gates in repo order, `make check`, `make lint`, `make test`, and `make test-long`.
2. If every gate passes, treat this bug as already fixed by the current branch. Tick the acceptance boxes, set `<passes>true</passes>`, switch the task, commit, and push without inventing extra type churn.
3. If a gate fails with this bug's symptom, repair the existing v2 boundary directly instead of introducing a new mirror:
   - keep the authoritative data on `RuntimeConfigV2.postgres`;
   - patch whichever ingestion edge is actually missing it (`load_runtime_config` or `dev_support::runtime_config_v2`);
   - thread the existing field into the failing consumer and delete any stale helper or legacy rebuild exposed by the failure.
4. Lock the boundary with focused regression coverage if execution has to touch code:
   - a loader/converter test proving non-default `postgres.local_database` survives into `RuntimeConfigV2`;
   - a loader/converter or managed-postgres test proving configured HBA/ident source data survives as resolved v2 contents and is what gets written to managed files;
   - consumer assertions only where needed to show roles/source/session code still stays on `RuntimeConfigV2`.
5. If execution shows the diagnosis above is wrong in a deeper way, switch this task back to `TO BE VERIFIED`, explain the exact missing v2 field or leftover legacy reconstruction path in this file, and stop immediately.
6. Only after every required gate passes:
   - tick the acceptance boxes that are truly complete,
   - set `<passes>true</passes>`,
   - run `/bin/bash .ralph/task_switch.sh`,
   - commit all changes, including `.ralph` updates, with `task finished [task name]: ...`,
   - push with `git push`,
   - stop immediately.

NOW EXECUTE
