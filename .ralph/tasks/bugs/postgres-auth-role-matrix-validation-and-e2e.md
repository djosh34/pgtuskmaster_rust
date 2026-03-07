## Bug: PostgreSQL auth/role matrix validation and e2e coverage <status>done</status> <passes>true</passes>

<description>
Pass-8 of the recurring deep-skeptical meta-task (`.ralph/tasks/story-rust-system-harness/18-task-recurring-meta-deep-skeptical-codebase-review.md`) built an explicit auth/role matrix under `.ralph/evidence/meta-18-pass8-20260307T065112Z/auth-matrix/auth-matrix.csv`.

Key gap: several production-relevant combinations are accepted by the schema/parser but are not mechanically validated and do not have external-interface real-binary e2e coverage.

Concrete examples:

- `postgres.local_conn_identity.ssl_mode = "require"` can currently coexist with `postgres.tls.mode = "disabled"`. This will predictably fail at runtime when the control plane attempts internal SQL connections, but the configuration parser does not reject it.
- `postgres.roles.*.auth.type = "tls"` currently only affects whether `PGPASSWORD` is set for libpq-driven subprocesses (see `role_auth_env` in `src/process/worker.rs`). There is no explicit wiring for libpq TLS client identity material (`PGSSLCERT`, `PGSSLKEY`, etc) in the runtime today, so the operational meaning of `auth.type = "tls"` is unclear and likely misnamed or incomplete.
- Non-`postgres` usernames for `replicator`/`rewinder` roles are supported in config + some unit tests, but there is no external-interface (API/CLI) real-binary e2e test proving that basebackup/rewind workflows succeed with those non-default role names.

This bug should explore the intended meaning of `RoleAuthConfig::Tls` and then either:

- implement the missing TLS client-auth wiring and add real-binary e2e coverage, OR
- rename/reshape the config surface to match the actual behavior (and add fail-closed validation for invalid combinations), OR
- explicitly constrain the surface (reject `auth.type = "tls"` until the implementation exists).

The fix must fail closed: incorrect or contradictory auth/TLS settings should be rejected early with actionable parser errors.
</description>

<acceptance_criteria>
- [x] Config validation rejects contradictory PostgreSQL TLS/auth combinations with actionable error messages.
- [x] Real-binary external-interface e2e coverage exists for at least:
  - [x] a password-auth baseline (already covered by docker smoke) remains green
  - [x] a non-`postgres` replicator/rewinder username configuration
  - [x] the chosen/implemented `auth.type = "tls"` meaning (or the surface is removed/disabled with docs updated)
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan

### Chosen direction

Fail closed by constraining the current surface instead of pretending PostgreSQL role TLS auth exists.

Why this is the right first implementation target:

- `RoleAuthConfig::Tls` in `src/config/schema.rs` carries no client certificate, private key, or CA material, so there is currently no way to express libpq client TLS identity in config.
- `src/process/worker.rs` only interprets `RoleAuthConfig::Tls` as “do not set `PGPASSWORD`”, which is not a meaningful auth implementation.
- The real HA harness in `src/test_harness/ha_e2e/startup.rs` currently uses `auth = { type = "tls" }` while `postgres.tls.mode = "disabled"` and the generated `pg_hba` trusts loopback traffic, which confirms the surface is hollow today.

That means this task should not try to smuggle in a partial client-cert feature. The plan is to reject PostgreSQL role `auth.type = "tls"` with actionable validation, move tests/harness/examples to password auth, and add real-binary proof that non-default role names work.

### Execution plan

1. Add explicit PostgreSQL auth/TLS matrix validation in `src/config/parser.rs`.
   - Add a dedicated helper for PostgreSQL-specific auth/TLS invariants and call it from `validate_runtime_config`.
   - Reject `postgres.roles.superuser.auth = { type = "tls" }`, `postgres.roles.replicator.auth = { type = "tls" }`, and `postgres.roles.rewinder.auth = { type = "tls" }` with field-specific errors that explain PostgreSQL role TLS client auth is not implemented and `password` must be used for now.
   - Reject `postgres.local_conn_identity.ssl_mode` and `postgres.rewind_conn_identity.ssl_mode` when they require server TLS (`require`, `verify-ca`, `verify-full`) but `postgres.tls.mode = "disabled"`.
   - Keep `disable`, `allow`, and `prefer` valid with `postgres.tls.mode = "disabled"` because those modes can still connect without requiring server TLS.

2. Add narrow parser coverage for the new fail-closed rules.
   - Extend `src/config/parser.rs` tests with explicit invalid-config fixtures for:
     - `postgres.local_conn_identity.ssl_mode = "require"` plus `postgres.tls.mode = "disabled"`
     - `postgres.rewind_conn_identity.ssl_mode = "verify-full"` plus `postgres.tls.mode = "disabled"`
     - each PostgreSQL role using `auth.type = "tls"`
   - Assert both the stable `field` path and the human-facing error message content.

3. Realign all sample/default/stub configs and harness fixtures so they represent actual supported behavior.
   - Update `src/test_harness/runtime_config.rs` sample runtime config helpers to use explicit non-empty password secrets instead of `RoleAuthConfig::Tls`.
   - Update production-adjacent contract/default fixtures that still encode unsupported TLS auth, especially `src/ha/state.rs`, `src/ha/worker.rs`, and the corresponding HA/process unit tests that currently assert `RoleAuthConfig::Tls`.
   - Sweep nearby sample/test fixtures that construct PostgreSQL role auth directly (for example in process/logging tests) and convert valid fixtures to password auth, leaving only dedicated negative tests for the rejected `tls` surface.
   - Update `src/test_harness/ha_e2e/startup.rs` so generated runtime config and DCS init payload use password auth for PostgreSQL roles.
   - Update the bootstrap SQL in the HA harness to create/alter the `replicator` and `rewinder` roles with passwords matching the configured secrets, instead of relying on trust-only semantics.

4. Extend the HA e2e harness just enough to test non-default PostgreSQL role names.
   - Add a small override surface to `src/test_harness/ha_e2e/config.rs` and `src/test_harness/ha_e2e/startup.rs` so a test can specify custom `replicator` and `rewinder` usernames and matching password material.
   - Keep the write scope narrow: this should be a targeted test harness option, not a broad runtime-config abstraction rewrite.
   - Ensure the harness-generated `pg_hba` entries reference the configured replication username rather than hardcoding `replicator`.

5. Add external-interface tests that prove both the invalid and valid paths.
   - In `tests/cli_binary.rs`, add invalid-config binary tests asserting that:
     - PostgreSQL role `auth.type = "tls"` exits with code `1` and prints the stable field path.
     - contradictory internal `ssl_mode` versus `postgres.tls.mode = "disabled"` exits with code `1` and prints the stable field path.
   - Add a real-binary HA e2e scenario under `tests/ha_multi_node_failover.rs` or a nearby HA test module that starts a cluster with non-default replicator/rewinder usernames and password auth, then proves:
     - replica bootstrap via `pg_basebackup` succeeds using the custom replicator role;
     - a rewind-bearing recovery path succeeds using the custom rewinder role.

6. Remove stale references to unsupported PostgreSQL role TLS auth from docs and examples.
   - Update `docs/src/operator/configuration.md` and any nearby troubleshooting text that currently implies PostgreSQL role auth can be `tls`.
   - Sweep parser fixtures, CLI fixtures, and any harness examples that currently use `auth = { type = "tls" }` as a valid PostgreSQL role configuration and convert them to either:
     - password auth for valid examples, or
     - dedicated negative tests for the newly rejected surface.

7. Verify in increasing cost order, then complete the Ralph flow.
   - Run focused tests for `src/config/parser.rs`, `tests/cli_binary.rs`, and the targeted HA e2e scenario first.
   - Run full required verification:
     - `make check`
     - `make test`
     - `make test-long`
     - `make lint`
   - Update docs if any examples/config snippets changed.
   - Only after all checks are green: tick task boxes, set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all changes including `.ralph`, push, and quit.

### Notes for verifier

The most likely thing to revisit during verification is whether the HA e2e should merely prove custom usernames under trust, or whether it should also enforce password auth end-to-end in `pg_hba`. The plan above intentionally biases toward the stronger version so the test validates both username routing and the only currently supported PostgreSQL role auth mode.

NOW EXECUTE
