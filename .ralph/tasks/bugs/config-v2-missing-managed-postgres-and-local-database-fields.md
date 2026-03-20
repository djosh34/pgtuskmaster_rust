## Bug: Config V2 Drops Managed Postgres Inputs Needed By Process Reduction <status>not_started</status> <passes>false</passes>

<description>
The direct-cfg reduction for the daemon runtime cannot finish correctly because `RuntimeConfigV2` currently drops static config that surviving startup/process helpers still need.

Concrete evidence from execution:
- `src/postgres_roles.rs` needs `postgres.local_database` for managed role reconciliation.
- `src/postgres_managed.rs` still needs the authoritative `postgres.access.hba` and `postgres.access.ident` source data to materialize managed files.
- The raw/private v2 schema still contains at least `postgres.local_database`, but `src/config_v2/types.rs` does not carry it forward, and the managed-postgres source contents are also not available on the v2 type graph.

Explore and research the codebase first, then fix the v2 type design so the process/materialization/runtime-root reduction can stay on config-v2 only without rebuilding `crate::config::RuntimeConfig`, hard-coding defaults, or keeping old-config mirrors alive.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
