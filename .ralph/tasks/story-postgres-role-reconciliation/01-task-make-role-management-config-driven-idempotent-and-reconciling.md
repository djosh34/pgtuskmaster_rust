## Task: Make PostgreSQL Role Management Config-Driven, Idempotent, And Reconciling <status>not_started</status> <passes>false</passes>

<priority>low</priority>
<blocked_by>Full completion of `.ralph/tasks/story-config-simplification/`</blocked_by>

<description>
**Goal:** Redesign PostgreSQL role management so the running system reconciles the full configured role set into PostgreSQL, not just the current fixed trio hard-coded in `src/postgres_roles.rs`. The higher-order goal is to treat PostgreSQL roles as typed desired state owned by the config and HA reconciliation loop: on startup the node must idempotently converge PostgreSQL onto that desired role set, and whenever the configured role set changes later the same reconciliation must run again and converge without manual SQL.

The required product behavior for this story is:
- the config can declare any number of PostgreSQL roles, not only the three currently modeled roles;
- `superuser`, `replicator`, and `rewinder` remain mandatory system roles and cannot be removed from config or from the managed reconciliation target;
- extra configured roles can be added and removed over time;
- on startup all configured roles are applied idempotently;
- when the configured roles change later, the same reconciliation logic runs again and applies the delta idempotently;
- roles that are no longer configured are removed from PostgreSQL when and only when they are managed extra roles, never by blindly dropping the mandatory system roles.

This task is intentionally blocked on `.ralph/tasks/story-config-simplification/` because the current config schema in `src/config/schema.rs` cannot represent this feature honestly yet. Today `PostgresRolesConfig` is a fixed struct with exactly:
- `superuser`
- `replicator`
- `rewinder`

Current repo context that makes this follow-on story necessary:
- `src/postgres_roles.rs` currently exposes `ensure_required_roles(...)` and `render_required_role_sql(...)`. Both functions are explicitly limited to the required trio and only ever emit create/alter SQL for those three roles plus rewinder grants.
- `src/ha/worker.rs` invokes that logic only from `ReconcileAction::EnsureRequiredRoles`, so the HA layer concept is also still fixed around “required roles” rather than a broader desired role-set reconciliation.
- `src/runtime/node.rs`, `src/ha/state.rs`, `src/ha/source_conn.rs`, and `src/process/jobs.rs` still assume dedicated named mandatory roles for superuser/replicator/rewinder when building conninfo and process specs.
- the config-simplification story already plans to remove the duplicated identity concepts and simplify postgres role config. This story must build on that new config surface rather than inventing a second incompatible shape.

This story is about runtime role reconciliation behavior after the config model can represent it. It is not only a parser change. The implementation should prefer compiler-driven types over loose maps of strings whenever possible. The expected direction is:
- keep a typed mandatory-role struct or equivalent ADT for the three system-owned roles that the runtime needs for bootstrap/replication/rewind;
- add a typed representation for extra managed roles configured by the operator;
- derive one explicit desired role-set value for reconciliation;
- reconcile PostgreSQL against that desired set, including additions, updates, grants, and removals of managed extra roles.

Important safety rules for the reconciliation design:
- do not allow the mandatory roles to disappear from config;
- do not drop or strip the mandatory roles during reconciliation even if the surrounding code is buggy;
- do not issue blind destructive SQL against every role in `pg_roles`;
- reconcile only the roles that pgtuskmaster declares as managed;
- role reconciliation must stay idempotent across repeated startup ticks and repeated config updates;
- if a config update changes only one role, the same reconciliation path should still be safe to run over the whole desired set.

**Scope:**
- Redesign the postgres role config shape created by the config-simplification story so it can represent:
  - the three mandatory system roles,
  - any number of additional managed roles,
  - the role attributes/auth material required for PostgreSQL-side reconciliation.
- Update runtime config validation so:
  - `superuser`, `replicator`, and `rewinder` are always present,
  - extra managed roles cannot shadow those mandatory logical role keys,
  - the resulting type makes the mandatory-vs-extra split explicit.
- Replace the current “required roles only” provisioning path in `src/postgres_roles.rs` with a desired-role reconciliation module that can:
  - create missing managed roles,
  - alter existing managed roles to match config,
  - apply required grants for mandatory and extra roles,
  - remove managed extra roles that were deleted from config,
  - never remove the mandatory system roles.
- Update HA/runtime triggering so startup reconciliation still happens automatically and later config role changes also retrigger reconciliation.
- Keep the runtime paths that build bootstrap/replication/rewind conninfo using the typed mandatory roles, even after extra roles exist.
- Update test fixtures, HA givens, runtime-config samples, and docs that still assume the old fixed-only role surface.

**Context from research:**
- `src/config/schema.rs` currently defines `PostgresRolesConfig` as a fixed struct with `superuser`, `replicator`, and `rewinder`. That is the primary reason this story cannot start before the config rewrite.
- `src/postgres_roles.rs` currently:
  - connects locally using the configured superuser;
  - renders SQL only for `superuser`, `replicator`, and `rewinder`;
  - applies rewinder-specific grants through `render_rewinder_grants_sql(...)`;
  - does not model extra roles or removals at all.
- `src/ha/worker.rs` currently sets `ctx.state.required_roles_ready = true` after `ReconcileAction::EnsureRequiredRoles`. That naming and state meaning should be revisited so the HA state reflects broader managed-role reconciliation rather than a hard-coded trio.
- `src/runtime/node.rs` currently derives `ProcessDispatchDefaults` from the mandatory `replicator` and `rewinder` usernames/auth. That remains valid, but it should clearly consume the mandatory-role ADT instead of depending on a broader raw config blob.
- The config-simplification story already decided that role config should get simpler and that the runtime should derive internal identities from the configured roles. This story must stay aligned with that direction instead of reintroducing duplicate user-facing identity blocks.

**Expected outcome:**
- The config surface can express the mandatory system roles plus arbitrary additional managed PostgreSQL roles.
- Startup role reconciliation converges all configured managed roles idempotently.
- Later role-set config changes trigger the same reconciliation logic and converge PostgreSQL again.
- Removing an extra managed role from config removes it from PostgreSQL.
- Mandatory `superuser`, `replicator`, and `rewinder` remain protected and always present in the desired role set.
- Runtime bootstrap, replication, and rewind code still use the typed mandatory roles cleanly after the broader reconciliation redesign.

</description>

<acceptance_criteria>
- [ ] `.ralph/tasks/story-config-simplification/` is fully complete first; this task does not start on top of the old fixed-role schema.
- [ ] `src/config/schema.rs`, `src/config/parser.rs`, `src/config/defaults.rs`, and any replacement config-validation modules represent PostgreSQL roles as a typed desired set with mandatory `superuser` / `replicator` / `rewinder` plus arbitrary additional managed roles.
- [ ] Config validation rejects any config that omits one of the mandatory system roles or tries to redefine an extra managed role with one of those reserved logical names.
- [ ] `src/postgres_roles.rs` no longer hard-codes a three-role-only SQL renderer; it reconciles the full managed desired role set, including create/alter/remove behavior for managed extra roles and mandatory-role protection.
- [ ] The runtime still derives bootstrap, replication, and rewind behavior from the typed mandatory-role values in `src/runtime/node.rs`, `src/ha/state.rs`, `src/ha/source_conn.rs`, `src/ha/process_dispatch.rs`, and `src/process/jobs.rs`.
- [ ] HA/runtime triggering re-runs role reconciliation both on startup and after role-set config changes, with idempotent behavior across repeated runs.
- [ ] startup reconciliation with only the mandatory roles,
- [ ] startup reconciliation with additional managed roles,
- [ ] idempotent reapplication with no changes,
- [ ] updating an existing managed role,
- [ ] removing a previously configured extra managed role,
- [ ] protection against removing mandatory system roles.
- [ ] HA/runtime fixtures and examples under `tests/ha/givens/**`, `tests/cli_binary.rs`, docs examples, and any shipped runtime-config samples are updated to the new role model.
- [ ] `make check` passes cleanly.
- [ ] `make test` passes cleanly.
- [ ] `make lint` passes cleanly.
</acceptance_criteria>
