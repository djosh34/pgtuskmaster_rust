## Bug: PostgreSQL replica primary_conninfo password auth is missing <status>not_started</status> <passes>false</passes>

<description>
Real-binary HA work for `.ralph/tasks/bugs/postgres-auth-role-matrix-validation-and-e2e.md` exposed that password auth is only wired for libpq subprocesses like `pg_basebackup`/`pg_rewind`, not for steady-state standby streaming after bootstrap.

Evidence:

- The HA harness was updated to use non-default `replicator` / `rewinder` usernames with password auth and `pg_hba` entries that require `scram-sha-256`.
- `pg_basebackup` succeeded with the custom replicator username, proving subprocess password auth works.
- Replicas never replayed later WAL, and the former primary never rejoined after failover, because managed standby config still renders `primary_conninfo` from `PgConnInfo` only.
- `src/postgres_managed_conf.rs` shows `ManagedPostgresStartIntent::{Replica,Recovery}` carry only `PgConnInfo`, and `render_managed_postgres_conf` writes `primary_conninfo = '...'` without password or passfile material.

This means the current runtime accepts PostgreSQL role password auth in config, but the long-lived replication path still depends on trust-style HBA or some external password side channel. Explore and research the codebase first, then fix it properly. Likely solutions include a managed passfile surface for standby/recovery connections or another explicit, auditable way to supply password material to PostgreSQL's walreceiver path.
</description>

<acceptance_criteria>
- [ ] Managed replica/recovery startup carries password auth for `primary_conninfo` in a secure, explicit way.
- [ ] Real-binary HA coverage proves replicas continue streaming WAL and rewinding under password-authenticated non-default replicator/rewinder role names.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
