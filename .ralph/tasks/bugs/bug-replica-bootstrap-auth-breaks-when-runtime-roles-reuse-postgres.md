## Bug: Replica Bootstrap Auth Breaks When Runtime Roles Reuse `postgres` <status>not_started</status> <passes>false</passes>

<description>
The new `cucumber_tests/ha` Docker HA harness now reaches a real primary on `node-b`, but replica bootstrap currently fails during `pg_basebackup` when `node-a` and `node-c` try to clone from that primary.

This was detected while running the greenfield cucumber feature `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.feature` through the new harness. The harness stages startup as `etcd -> observer -> node-b -> node-a/node-c`, uses `pgtm status --json` for readiness, and now fails the startup story quickly with a concrete terminal error instead of timing out.

Concrete observations from the failing run:

- `pgtm status --json` from the observer shows `node-b` sampled and primary.
- `node-a` and `node-c` exit immediately during startup clone mode.
- Their logs show `pg_basebackup` against `node-b` failing with:
- `FATAL: password authentication failed for user "postgres"`
- `FATAL: pg_hba.conf rejects replication connection ... no encryption`
- Inspecting the live primary over its local socket shows only one login role, `postgres`, and `select rolpassword is not null from pg_authid where rolname='postgres'` returns `false`.

That means the configured runtime role/auth contract is not being materialized coherently for this case. The current configs and examples use:

- `[postgres.roles.superuser] username = "postgres"`
- `[postgres.roles.replicator] username = "postgres"`
- `[postgres.roles.rewinder] username = "postgres"`

But the live primary still ends up with no password set on `postgres`, so replica bootstrap cannot authenticate to the primary at all. Explore and research the runtime code first, then fix the runtime/auth behavior rather than papering over it in the harness.
</description>

<acceptance_criteria>
- [ ] Reproduce the failure from the new cucumber HA harness and identify the runtime code path that materializes PostgreSQL role credentials for bootstrap primary startup and replica clone startup
- [ ] The runtime behaves coherently when the configured superuser/replicator/rewinder identities reuse the same PostgreSQL role name `postgres`, or the runtime rejects that configuration explicitly with a clear validation/runtime error instead of silently starting a primary with unusable auth
- [ ] `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.feature` no longer fails at initial replica clone bootstrap because of missing/incorrect primary auth material
- [ ] If the correct fix is to forbid duplicate PostgreSQL usernames across configured roles, validation/docs/examples are updated consistently and stale misleading examples are removed
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
