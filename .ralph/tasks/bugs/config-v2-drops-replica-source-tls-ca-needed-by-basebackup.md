## Bug: Config v2 drops replica source TLS CA needed by basebackup <status>completed</status> <passes>true</passes>

<description>
Ultra-long HA scenarios now progress past seed-primary bootstrap, but replica startup still fails because `pg_basebackup` is launched without the configured TLS root CA.

The preserved HA artifacts show repeated replica-side errors like:
`pg_basebackup: error: connection to server at "node-b" ..., port 5432 failed: root certificate file "/var/lib/postgresql/.postgresql/root.crt" does not exist`

This happens because config-v2 parsing currently discards `postgres.rewind.transport.ca_cert`. `RuntimeConfigV2` only keeps the SSL mode on the mandatory role config, and `process::source::source_from_member` therefore materializes source conninfo without the CA path required for basebackup/rewind verification.

Explore and research the codebase first, then fix.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Verified design
- The validated runtime shape for this concern already exists and should stay singular: [src/config_v2/types.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/types.rs) carries one shared `postgres.source_client_tls: PgClientTls`, and [src/process/source.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/source.rs) already reuses that exact shape for basebackup, rewind, and replica follow conninfo materialization. Do not add `BasebackupTlsConfig`, `ReplicaSourceTlsConfig`, or any second config-v2 mirror.
- The boundary smell is therefore not "missing a new type"; it is that the real config-v2 ingestion-to-execution path is not truthfully proven. [src/config_v2/parser/load_config.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/parser/load_config.rs) already maps `postgres.rewind.transport` into `postgres.source_client_tls`, the HA fixture runtime TOML in [tests/ha/support/world/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/world/mod.rs) already renders `postgres.rewind.transport.ca_cert`, and [src/process/tools.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/tools.rs) already lowers basebackup with full conninfo via `--dbname`. The task description is at least incomplete, so the next turn must prove which edge still drops the CA instead of assuming the parser alone is wrong.
- Keep the boundary flat: one raw TOML DTO at ingestion, one validated `PgClientTls` in runtime, one `MandatoryRoleSourceConn` materialization path. Reduce helpers if they turn out to obscure that ownership, but do not rebuild legacy config adapters or introduce new role-specific TLS structs.

### Execution plan
1. Reproduce the surviving failure on the direct config-v2 path before changing code:
   - inspect the current failure from `make test-long` and, if needed during development, use a focused `cargo nextest` invocation for the specific config-v2 parser/process/HA test that exposes the drop;
   - do not use `cargo test`;
   - confirm whether the missing `sslrootcert` happens at runtime loading, source materialization, or command lowering.
2. Keep the existing shared runtime type graph unless reproduction proves otherwise:
   - `PostgresConfig.source_client_tls: PgClientTls` remains the only validated source-TLS shape;
   - `source_from_member` remains the only place that turns that shared config into role-specific source connections;
   - if a helper duplicates or re-derives this data, collapse it into the existing ingestion/materialization path instead of creating a new struct.
3. Add the narrowest missing regression coverage that proves the full config-v2 boundary:
   - a loader-level test proving `postgres.rewind.transport.ca_cert` survives into `RuntimeConfigV2.postgres.source_client_tls.root_cert`;
   - a planner/materialization or tool-lowering test that starts from a real config-v2 runtime document, plans a basebackup, and proves the final `pg_basebackup --dbname` conninfo includes `sslrootcert`;
   - if HA-only reproduction exposes a different seam, replace one of the above with the smaller test that directly proves that seam.
4. Fix only the edge that actually drops the CA:
   - if the parser path is wrong, patch [src/config_v2/parser/load_config.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/config_v2/parser/load_config.rs) and keep one flat `PgClientTls` mapping;
   - if dev-support conversion is the only broken path, patch [src/dev_support/runtime_config_v2.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dev_support/runtime_config_v2.rs) without changing production runtime types;
   - if command lowering still strips TLS, patch [src/process/tools.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/tools.rs) and delete any stale helper logic that rebuilds partial conninfo.
5. If execution shows the current diagnosis is still wrong in a deeper way, switch this task back to `TO BE VERIFIED`, explain the exact remaining type or ownership gap in this file, and stop immediately.
6. Run the required validation gates in repo order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
7. Only after every gate passes:
   - tick the acceptance boxes that are truly complete,
   - set `<passes>true</passes>`,
   - run `/bin/bash .ralph/task_switch.sh`,
   - commit all changes, including `.ralph` updates, with `task finished [task name]: ...`,
   - push with `git push`,
   - stop immediately.

### Resolution
- The current production config-v2 path already preserves `postgres.rewind.transport.ca_cert` through runtime loading, source materialization, and `pg_basebackup --dbname` command lowering.
- The missing piece was direct regression coverage for that full boundary, so this task added that proof and re-ran the full validation gates successfully.
