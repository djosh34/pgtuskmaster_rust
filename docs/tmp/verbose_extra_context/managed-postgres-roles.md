# Managed PostgreSQL roles extra context

This task changed the runtime configuration model and the PostgreSQL reconciliation behavior for managed roles.

The old model exposed exactly three role blocks:

- `postgres.roles.superuser`
- `postgres.roles.replicator`
- `postgres.roles.rewinder`

That old shape was a fixed struct and could not represent any operator-managed extra PostgreSQL roles. The runtime provisioning path in `src/postgres_roles.rs` only rendered SQL for those three roles plus the rewinder grants.

The new model makes the managed-role split explicit:

- `postgres.roles.mandatory.superuser`
- `postgres.roles.mandatory.replicator`
- `postgres.roles.mandatory.rewinder`
- `postgres.roles.extra.<logical_role_key>`

Important facts about the new shape:

- The `mandatory` block is required.
- The mandatory roles are still the system roles the runtime depends on for bootstrap, replication, rewind, and local SQL access.
- The `extra` block is optional.
- Extra roles are keyed by a logical managed-role key, not by raw username.
- Extra role keys must not shadow `superuser`, `replicator`, or `rewinder`.
- Every managed PostgreSQL role must have a unique username across both the mandatory and extra role sets.

Extra managed roles currently support explicit privilege intent through a typed ADT rather than loose booleans. At the config and SQL-rendering level this currently supports:

- a `privilege` value that determines whether the role is rendered as a login role or a non-login role
- a `member_of` list that declares target role memberships to grant

The reconciliation behavior also changed materially:

- The runtime no longer treats role provisioning as a fixed "required trio only" operation.
- The HA/runtime reconciliation path now works from a desired managed-role set.
- Startup reconciliation applies the full managed role set idempotently.
- Later config changes reuse the same reconciliation path.
- Mandatory roles are protected and never dropped by the reconciler.
- Extra managed roles that disappear from config are removed from PostgreSQL only when they are recognized as managed extra roles.

The implementation uses a comment marker on managed extra roles so stale extra-role cleanup only targets roles that pgtuskmaster previously created or adopted as managed extras. It does not issue blind destructive SQL across every role in `pg_roles`.

The runtime still derives system behaviors from the mandatory roles:

- local SQL uses the mandatory superuser and `postgres.local_database`
- replication uses the mandatory replicator role
- rewind uses the mandatory rewinder role together with `postgres.rewind`

The examples under `docker/node-*.toml`, CLI/runtime tests, and HA runtime fixture templates were updated to the new `postgres.roles.mandatory.*` shape.
