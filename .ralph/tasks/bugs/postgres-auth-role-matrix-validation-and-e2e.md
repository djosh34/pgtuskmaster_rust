## Bug: PostgreSQL auth/role matrix validation and e2e coverage <status>not_started</status> <passes>false</passes>

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
- [ ] Config validation rejects contradictory PostgreSQL TLS/auth combinations with actionable error messages.
- [ ] Real-binary external-interface e2e coverage exists for at least:
  - [ ] a password-auth baseline (already covered by docker smoke) remains green
  - [ ] a non-`postgres` replicator/rewinder username configuration
  - [ ] the chosen/implemented `auth.type = "tls"` meaning (or the surface is removed/disabled with docs updated)
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
