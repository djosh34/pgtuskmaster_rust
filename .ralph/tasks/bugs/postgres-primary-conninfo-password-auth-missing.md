## Bug: PostgreSQL replica primary_conninfo password auth is missing <status>completed</status> <passes>true</passes>

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
- [x] Managed replica/recovery startup carries password auth for `primary_conninfo` in a secure, explicit way.
- [x] Real-binary HA coverage proves replicas continue streaming WAL and rewinding under password-authenticated non-default replicator/rewinder role names.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Research Notes

- Confirmed root cause in [src/postgres_managed_conf.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed_conf.rs): `ManagedPostgresStartIntent::{Replica,Recovery}` only carry `PgConnInfo`, and `render_managed_postgres_conf` writes `primary_conninfo` from that non-secret type alone.
- Confirmed subprocess-only password auth in [src/process/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs): `pg_basebackup` and `pg_rewind` use `PGPASSWORD`, which does not help PostgreSQL's walreceiver after the server has started.
- Confirmed the restart/rejoin path in [src/ha/process_dispatch.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/process_dispatch.rs) and [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs): startup reconstructs managed config either from DCS leader info or by rereading previously materialized managed replica state.
- Confirmed the existing real HA scenario in [tests/ha/support/multi_node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/multi_node.rs) already exercises the required symptom: bootstrap with custom password-authenticated replicator/rewinder roles succeeds, failover happens, then the former primary is expected to rewind/rejoin and receive post-failover rows. That scenario is the right end-to-end proof once standby auth is actually wired.
- Confirmed parser constraint in [src/pginfo/conninfo.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/pginfo/conninfo.rs): `parse_pg_conninfo` rejects unknown keys today, so a `passfile=...` token inside `primary_conninfo` cannot be round-tripped unless this area is redesigned.

## Detailed Plan

### 1. Introduce a typed managed standby-auth surface and a dedicated managed primary-conninfo helper

- Keep `PgConnInfo` focused on the non-secret libpq fields already used across the codebase.
- Extend the managed-startup model in [src/postgres_managed_conf.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed_conf.rs) so replica/recovery intents carry both:
  - the existing upstream `PgConnInfo`
  - an explicit managed-auth description for walreceiver startup
- The managed-auth type should distinguish at least:
  - password auth backed by a managed passfile path
  - non-password auth that does not require a passfile
- Do not render plaintext passwords into `primary_conninfo`, task files, logs, or test assertions.
- Add a small managed `primary_conninfo` render/parse helper near the managed-config code rather than extending the generic conninfo parser with secret-bearing or PostgreSQL-specific escape hatches. That helper should own the limited extra token surface needed here, specifically `passfile`, while still delegating the shared non-secret fields to `PgConnInfo`.

### 2. Materialize a managed passfile for standby/recovery starts

- In [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs), add a dedicated managed artifact path under `PGDATA`, owned by pgtuskmaster, for the standby connection passfile.
- When the start intent is replica/recovery and the configured replicator auth is password-based:
  - resolve the password from `RuntimeConfig`
  - write a libpq passfile entry with `write_atomic`
  - enforce `0600` permissions
  - render `primary_conninfo` with `passfile=<managed path>` added explicitly
- When the start intent is primary or the standby auth does not use a password:
  - remove any stale managed standby passfile so old credentials cannot linger and be reused accidentally
- Return the managed passfile path from `materialize_managed_postgres_config` if that makes later assertions or cleanup clearer.

### 3. Make start-intent creation carry standby auth all the way through

- Update start-intent construction sites so they no longer drop auth on the floor:
  - [src/ha/process_dispatch.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/process_dispatch.rs)
  - [src/runtime/node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs)
- Prefer building the managed replica intent from the existing `ReplicatorSourceConn`, not from `source.conninfo.clone()` alone, because that source already owns the correct replicator username and auth mode.
- Keep rewind auth unchanged: `pg_rewind` should continue using rewinder credentials via subprocess env, while steady-state replication uses the replicator role via managed startup artifacts.

### 4. Make existing managed replica state round-trip cleanly through the dedicated helper

- Startup without a DCS leader must still be able to reread previously materialized managed replica state.
- Do not expand the general-purpose `PgConnInfo` parser to understand `passfile` or other managed-only tokens.
- Instead, add a dedicated helper that parses the managed `primary_conninfo` line into:
  - the upstream non-secret conninfo fields
  - the standby-auth mode, including whether a `passfile` token is present and which managed path it points at
  - the optional `primary_slot_name`
- Preserve strict validation:
  - reject malformed quoted values
  - reject unexpected/unsafe managed passfile locations and require the managed standby passfile path to stay under `PGDATA`
  - fail loudly if managed replica state is incomplete or inconsistent

### 5. Strengthen focused tests before running the full gates

- In [src/postgres_managed_conf.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed_conf.rs):
  - add rendering assertions that password-authenticated replica/recovery intents include `passfile=...`
  - add assertions that primary starts do not render standby-only passfile material
- In [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs):
  - verify managed passfile creation, file contents, and `0600` permissions
  - verify stale managed passfile cleanup on primary starts
  - verify `read_existing_replica_start_intent` round-trips password-authenticated replica state
  - extend the real PostgreSQL startup test so it uses password-authenticated replication and proves the replica actually replays WAL after startup by inserting new WAL on the primary after the replica starts and polling for the row on the replica
- In [src/ha/process_dispatch.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/ha/process_dispatch.rs):
  - assert replica start dispatch writes a managed config containing the passfile reference when the replicator role uses password auth
  - assert preserved existing replica state keeps the same standby-auth behavior when DCS leader info is temporarily absent

### 6. Revalidate the real HA scenario with the bug's exact symptom

- Use [tests/ha/support/multi_node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/multi_node.rs) as the principal end-to-end proof.
- Keep the non-default `replicator_custom` / `rewinder_custom` usernames and password auth requirement.
- After the code fix, this scenario should prove all of the following, without changing its intent:
  - clone/bootstrap still works with the custom replicator credentials
  - replicas continue streaming later WAL via the steady-state walreceiver path
  - the former primary rewinds and rejoins using the rewinder role
  - the rejoined former primary receives the post-failover row, proving the managed standby credential path actually works

### 7. Update operator-facing docs

- Update [docs/src/lifecycle/recovery.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/lifecycle/recovery.md) to explain that:
  - rewind auth and steady-state replication auth are distinct runtime paths
  - password-authenticated standby/recovery starts rely on a managed passfile artifact rather than an implicit trust-only HBA assumption
  - stale unmanaged PostgreSQL recovery artifacts are still not authoritative
- If another doc is a better fit once execution starts, update that instead, but some docs change is required if the runtime surface changes.

### 8. Exact execution order for the later `NOW EXECUTE` pass

1. Refactor the managed-startup types so standby intents explicitly carry auth metadata, plus a dedicated managed `primary_conninfo` render/parse helper.
2. Add passfile materialization and stale-file cleanup in `materialize_managed_postgres_config`.
3. Update managed-config rendering/parsing so replica state can be written and reread with the explicit auth surface through that dedicated helper.
4. Update startup and HA dispatch call sites to construct the richer start intent from replicator source connections.
5. Add or update the focused unit/integration tests in `src/postgres_managed_conf.rs`, `src/postgres_managed.rs`, and `src/ha/process_dispatch.rs`.
6. Re-run the relevant targeted tests while iterating until the real startup proof and HA custom-role scenario are green.
7. Update docs.
8. Only then run the full required gates in this order: `make check`, `make test`, `make test-long`, `make lint`.
9. After all gates pass, tick the acceptance boxes, set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit everything including `.ralph` state, and push.

### 9. Required skeptical review targets for the `TO BE VERIFIED` pass

- Challenge whether the managed `primary_conninfo` helper should live in `postgres_managed_conf.rs` or a tiny adjacent module to avoid contaminating generic conninfo parsing.
- Verify that rereading existing managed state does not require the original inline secret and therefore should persist the managed passfile path rather than trying to reconstruct `SecretSource`.
- Verify the passfile format exactly matches what libpq/postgres walreceiver expects for `primary_conninfo`-driven replication.
- Check whether the real startup test in `src/postgres_managed.rs` must insert new WAL after the replica starts; the current test only proves startup/recovery mode, not continued replay.
- Confirm whether any logging or debug surfaces could accidentally expose the managed passfile path or password content and tighten them if needed.
- The `TO BE VERIFIED` pass must change at least one concrete part of this plan before promoting it.

NOW EXECUTE
