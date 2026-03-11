## Bug: Docker socket permission blocks greenfield HA harness runtime validation <status>completed</status> <passes>true</passes> <priority>high</priority>

<description>
Greenfield Docker HA cucumber scenarios cannot start in the current execution environment because `docker info` fails with:

`permission denied while trying to connect to the docker API at unix:///var/run/docker.sock`

This was detected while running `cargo nextest run --profile ultra-long --no-fail-fast --target-dir /tmp/pgtuskmaster_rust-target --config 'build.incremental=false' --test ha_api_path_isolation_preserves_primary` for story task 04. The test compiled and launched, but the first step failed before scenario setup because the harness validates Docker availability during `HarnessShared::initialize`.

Investigate the actual environment-level root cause first, then fix it in the most source-controlled way possible. That may mean correcting the local execution contract, adjusting how the harness discovers and invokes Docker, or documenting/enforcing an explicit prerequisite if the runtime truly cannot self-heal.
</description>

<blocked_by>
.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md
</blocked_by>
<blocked_by>
.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md
</blocked_by>
<blocked_by>
.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md
</blocked_by>
<blocked_by>
.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md
</blocked_by>

<acceptance_criteria>
- [ ] Explore and confirm why this environment cannot use the Docker socket even though the repo's HA harness requires Docker-backed execution.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Because this blocks ultra-long HA wrappers: `make test-long` — passes cleanly
</acceptance_criteria>

## Plan

### Current source and environment findings

- The originally reported environment failure is stale on the current machine state. Fresh verification on 2026-03-11 shows:
  - `docker info` succeeds.
  - the current user is in the `docker` group.
  - `/var/run/docker.sock` is owned by `root:docker` with group read/write permissions.
- The exact focused scenario cited by the bug, `ha_api_path_isolation_preserves_primary`, now passes on current `HEAD` under `cargo nextest run --workspace --profile ultra-long --no-fail-fast --no-tests fail --target-dir /tmp/pgtuskmaster_rust-target --config 'build.incremental=false' --test ha_api_path_isolation_preserves_primary`.
- The cucumber harness already preserves raw Docker stderr through `HarnessError::CommandFailed`, so a permission-denied socket failure is not being silently swallowed.
- The current source-controlled gap is operator diagnostics and explicit prerequisites:
  - `Makefile` preflight collapses every `docker info` failure into the generic message `docker daemon is not reachable`.
  - `docs/src/how-to/run-tests.md` says to check whether the daemon is reachable, but it does not mention lack of permission to access `/var/run/docker.sock` or the expected Docker-group/socket contract.
- Because the runtime environment now works and the harness already surfaces stderr, the most likely durable fix is not a behavior change in HA logic. It is a stale-bug closure plus sharper Docker preflight and documentation so the same environment defect is obvious and actionable the next time it appears.

### Execution strategy

1. Prove the task is stale rather than still-open product work.
   - Re-run the focused `ha_api_path_isolation_preserves_primary` ultra-long wrapper at least one more time through `cargo nextest`.
   - If that fresh pass remains green, treat the original runtime blocker as resolved in the environment, not as an unfixed harness defect.
   - Only pivot to product-side debugging if the focused wrapper fails again on current `HEAD`.

2. Improve source-controlled diagnostics for Docker permission failures.
   - Update the repo preflight path so a failing Docker availability check reports the actual `docker info` stderr instead of replacing it with a generic daemon-unreachable message.
   - Keep the failure strict: missing access to Docker must still fail fast rather than degrade into skipped HA coverage.
   - If useful, include a brief actionable hint when the failure text matches socket-permission denial.

3. Update the contributor contract for long Docker-backed validation.
   - Update the test-running docs to say that contributors need permission to access the Docker daemon, not merely a running daemon.
   - Make the common Linux failure mode explicit: `/var/run/docker.sock` permission denied when the user lacks the expected socket/group access.
   - Use the `k2-docs-loop` skill for the docs update as required by the repository instructions.

4. Re-run the required full gates after the diagnostics/docs changes.
   - Run `make check`.
   - Run `make test`.
   - Run `make test-long`.
   - Run `make lint`.
   - Do not mark the task as passing until all four are green.

5. Close the task only after the full gate and docs state are complete.
   - Tick the acceptance criteria checkboxes with the final evidence.
   - Set `<passes>true</passes>` only after `make check`, `make test`, `make test-long`, and `make lint` pass.
   - Run `/bin/bash .ralph/task_switch.sh`.
   - Commit all tracked changes, including `.ralph`, with a `task finished ...` message that includes what was changed and which gates passed.
   - Push with `git push`.
   - Quit immediately.

### Skeptical review conclusion

- The bug description's central claim, "this environment cannot use the Docker socket," is false on the current 2026-03-11 environment, so execution must not invent a product fix for a non-reproducing failure.
- The correct execution path is to close the stale runtime blocker while improving the repo's diagnostics and contributor-facing Docker prerequisite language, because those are the only durable gaps still visible on current `HEAD`.

NOW EXECUTE
