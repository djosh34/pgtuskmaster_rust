## Bug: Runtime Verify-Full Conninfo Lacks Explicit CA Path <status>not_started</status> <passes>true</passes>

<description>
Runtime-managed PostgreSQL connections can require `sslmode=verify-full`, but the internal conninfo model does not carry an explicit CA-path field. As a result, the current runtime path falls back to ambient libpq environment such as `PGSSLROOTCERT` instead of rendering a complete source-backed conninfo.

Detection context on March 10, 2026:
- `src/ha/source_conn.rs` builds runtime-managed remote PostgreSQL conninfo through `PgConnInfo`
- `src/pginfo/conninfo.rs` renders `sslmode`, but has no field for `sslrootcert`
- the current shipped fixture still needs ambient `PGSSLROOTCERT` because the runtime path cannot express the CA path directly

The executor should explore the codebase first, then fix the runtime/config design so verify-full connections can carry the CA path explicitly instead of relying on process environment hacks.

Important code areas already implicated:
- `src/pginfo/conninfo.rs`
- `src/ha/source_conn.rs`
- `src/postgres_managed_conf.rs`
- `src/runtime/node.rs`

The intended direction is:
- runtime-managed remote PostgreSQL verify-full connections should carry the CA path through explicit config/rendered conninfo
- internal connection rendering should stop depending on ambient `PGSSLROOTCERT`
- callers and docs should reflect the explicit runtime-configured CA-path contract
</description>

<acceptance_criteria>
- [ ] runtime-managed remote PostgreSQL connections have an explicit, source-backed CA-path configuration path instead of relying on ambient `PGSSLROOTCERT`
- [ ] `src/pginfo/conninfo.rs` and the runtime-managed callers can render/use the CA path for verify-full connections
- [ ] runtime-managed verify-full flows continue to work after removing dependence on ambient `PGSSLROOTCERT`
- [ ] docs/examples are updated anywhere they would otherwise imply the env-hack approach
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] if the fix touches ultra-long HA behavior or its selection: `make test-long` — passes cleanly
</acceptance_criteria>
