## Task: Send PostgreSQL `SIGHUP` After Certificate Reload <status>not_started</status> <passes>false</passes>

<priority>low</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/08-task-replace-hand-rolled-api-server-with-axum-axum-server-and-tower.md`</blocked_by>

<description>
**Goal:** Extend the certificate reload behavior so that after the cert reload path succeeds, PostgreSQL is also reloaded via `SIGHUP` to the active postmaster. The higher-order goal is to make certificate rotation operationally complete: reloading API/server-side certificate material is not enough if PostgreSQL continues running with old TLS files until some later restart or manual intervention.

This story exists because the currently planned API rewrite task in `.ralph/tasks/story-ctl-operator-experience/08-task-replace-hand-rolled-api-server-with-axum-axum-server-and-tower.md` explicitly introduces `POST /reload/certs` but also explicitly says PostgreSQL reload is out of scope there. Your requested follow-up behavior should therefore land as a separate low-priority story after that route and its runtime plumbing exist.

The required product behavior for this story is:
- after the certificate reload endpoint completes its own cert-reload work successfully, the node also sends `SIGHUP` to the running PostgreSQL postmaster;
- this is a true process signal to the postmaster, not a `pg_ctl reload` wrapper and not SQL `SELECT pg_reload_conf()`;
- the signal must target the currently active postmaster for this node’s managed `postgres.data_dir`;
- the operation must stay safe when no managed postmaster is running: the API should return a clear failure or explicitly documented “not running” response rather than pretending PostgreSQL was reloaded;
- the endpoint must not report success for the PostgreSQL-reload part if the postmaster lookup or signal delivery failed.

Current repo context from research:
- the live hand-rolled API server in `src/api/worker.rs` currently does not even expose `/reload/certs`; the route only exists as planned work in story 08.
- `src/process/worker.rs` already contains careful helpers around `postmaster.pid` parsing and data-dir verification:
  - `parse_postmaster_pid(...)`
  - `postmaster_pid_data_dir_matches(...)`
  - `pid_exists(...)`
  - `pid_matches_data_dir(...)`
- `src/test_harness/signals.rs` already provides a small Unix signal helper for tests.
- The future implementation should reuse or extract the existing postmaster-identification logic rather than re-implementing a second ad hoc PID parser inside the API layer.

This task should deliberately keep the responsibility boundary narrow:
- it is not a broad runtime-config reload feature;
- it is not a reimplementation of the entire cert reload endpoint;
- it is specifically the follow-up behavior that sends `SIGHUP` to PostgreSQL after the cert reload path has succeeded.

Preferred implementation direction:
- factor the postmaster lookup and `SIGHUP` send into a small reusable runtime/process helper instead of burying signal logic in an HTTP handler;
- verify the pid belongs to the managed `postgres.data_dir` before signaling, using the existing safety checks already present in `src/process/worker.rs`;
- keep Unix behavior explicit and return an actionable error on unsupported platforms instead of silently doing nothing;
- keep success semantics strict: if API cert reload succeeded but PostgreSQL `SIGHUP` failed, the endpoint should surface that as a failure, not a silent partial success.

Operational contract that must be documented in code and tests:
- `POST /reload/certs` reloads the configured certificate material and then sends `SIGHUP` to the managed PostgreSQL postmaster.
- The PostgreSQL signal step happens after successful cert reload, not before.
- If cert reload fails, do not send `SIGHUP`.
- If cert reload succeeds but PostgreSQL `SIGHUP` fails, return an error response that makes the partial failure explicit.
- The admin-auth requirement for the reload endpoint remains unchanged.

**Scope:**
- Extend the existing/planned cert reload route from story 08 so it also triggers PostgreSQL reload via `SIGHUP`.
- Extract or reuse postmaster PID discovery/data-dir validation logic from `src/process/worker.rs` in a shared helper reachable from the reload path.
- Add error/reporting behavior for:
  - missing `postmaster.pid`,
  - stale PID files,
  - PID/data-dir mismatch,
  - no running postmaster,
  - signal-delivery failure.
- Update the endpoint response contract and tests to reflect the new PostgreSQL reload step.
- Update any docs that describe the reload endpoint so they mention the PostgreSQL `SIGHUP` behavior and failure cases.

**Context from research:**
- `src/api/worker.rs` current route surface is still only:
  - `GET /state`
  - `POST /switchover`
  - `DELETE /switchover`
- Story 08 is the first place that adds `POST /reload/certs`, so this story must build on top of that completed work.
- The process worker already has the most trustworthy code for locating a real managed postmaster from `postmaster.pid` and validating that the PID belongs to the configured data dir. Reuse that behavior instead of introducing a weaker second implementation.
- The repo already uses raw OS signal handling in test harness code; the production reload path should follow the same direct-signal approach rather than shelling out to `pg_ctl reload`.

**Expected outcome:**
- Certificate reload now also reloads PostgreSQL via `SIGHUP` on the active postmaster.
- The signal targets the correct managed postmaster for this node’s configured data dir.
- Success/failure reporting is honest about partial failures.
- Tests and docs reflect the new operational contract.

</description>

<acceptance_criteria>
- [ ] `.ralph/tasks/story-ctl-operator-experience/08-task-replace-hand-rolled-api-server-with-axum-axum-server-and-tower.md` is fully complete first; this task does not start before `/reload/certs` exists.
- [ ] The cert reload implementation sends PostgreSQL `SIGHUP` only after its own cert-reload step has succeeded.
- [ ] The PostgreSQL reload step uses a direct postmaster signal, not `pg_ctl reload` and not SQL `pg_reload_conf()`.
- [ ] Postmaster discovery validates that the PID belongs to the managed `postgres.data_dir` before signaling, reusing or extracting the existing logic from `src/process/worker.rs` instead of duplicating it loosely in the API layer.
- [ ] Error handling covers missing or stale `postmaster.pid`, PID/data-dir mismatch, no running postmaster, and signal-delivery failures with an explicit API error response.
- [ ] The endpoint does not claim full success when the cert reload succeeded but PostgreSQL `SIGHUP` failed.
- [ ] successful cert reload followed by successful PostgreSQL `SIGHUP`,
- [ ] cert reload failure preventing the signal step,
- [ ] stale or missing postmaster PID cases,
- [ ] PID/data-dir mismatch protection,
- [ ] signal-delivery failure reporting.
- [ ] The reload-endpoint docs and API contract text mention the PostgreSQL `SIGHUP` behavior and its failure semantics.
- [ ] `make check` passes cleanly.
- [ ] `make test` passes cleanly.
- [ ] `make lint` passes cleanly.
</acceptance_criteria>
