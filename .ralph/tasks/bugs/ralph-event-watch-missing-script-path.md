---
## Bug: ralph-event-watch service restart loop from missing script path <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`ralph-event-watch.service` is in continuous auto-restart with exit code 127 because `ExecStart` points to a non-existent script path:
`/home/joshazimullah.linux/work_mounts/projects/postgres_operator/PGTuskMaster/ElixirPGTuskMaster/.ralph/event_watch.sh`.

Detected on March 3, 2026 while checking Ralph systemd health. `systemctl --user status ralph-event-watch.service` shows `activating (auto-restart)` and `journalctl --user -u ralph-event-watch.service` repeatedly logs `No such file or directory`.

Explore and research the codebase and service configuration first, then fix the broken `ExecStart` target/path so the service can remain stable instead of looping.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make test-bdd` — all BDD features pass
</acceptance_criteria>

<plan>
## Plan (researched on March 3, 2026)

### Baseline findings (parallel exploration summary)
- `ralph-event-watch.service` is enabled and currently in restart loop (`status=127`) with `No such file or directory`.
- Live unit file and repo template both point `ExecStart` to `/home/joshazimullah.linux/work_mounts/projects/postgres_operator/PGTuskMaster/ElixirPGTuskMaster/.ralph/event_watch.sh`.
- That `event_watch.sh` file does not exist in the old Elixir workspace or in this Rust workspace.
- This Rust workspace currently has no `.ralph/event-processor/` implementation either, and Ralph runtime scripts do not depend on `ralph-event-watch.service` for core loop execution.
- Unit files under `~/.config/systemd/user` are static copies/symlinks and are vulnerable to stale absolute paths.
- `opencode-ralph.service` is also still pinned to the old Elixir working directory, but this bug scope is the restart loop caused by missing `event_watch.sh`.
- Exploration used parallel probes across: live systemd state, journal, unit templates, repo-wide path references, historical logs, legacy workspace content, and script wiring.

### Full execution plan
1. Preserve baseline evidence for this bug.
- Capture and store pre-fix outputs for:
  - `systemctl --user status ralph-event-watch.service --no-pager`
  - `journalctl --user -u ralph-event-watch.service -n 120 --no-pager`
  - `systemctl --user cat ralph-event-watch.service`

2. Introduce a canonical executable target in this repo.
- Add `.ralph/event_watch.sh` as the explicit `ExecStart` target so the unit no longer references a non-existent file.
- Script requirements:
  - strict shell settings (`set -euo pipefail`),
  - deterministic startup logging,
  - long-running stable behavior (no immediate exit loop) with predictable heartbeat + trap logs,
  - clear comment that this is compatibility/legacy event-watch entrypoint and where to evolve it.

3. Fix service template path in source control.
- Update `.ralph/systemd/ralph-event-watch.service` so `ExecStart` points to the new script path in the current repo.
- Use systemd home specifiers (`%h/...`) for `ExecStart` and `WorkingDirectory` to avoid user-specific absolute path drift.
- Keep `Restart=always` only if the script is genuinely long-running; otherwise change restart policy to avoid restart loops.
- Ensure `WorkingDirectory` matches the same repository root as the script target.

4. Reconcile deployed user unit with template.
- Rewrite `~/.config/systemd/user/ralph-event-watch.service` from the repo template (copy/sync), then verify rendered config with `systemctl --user cat`.
- Run `systemctl --user daemon-reload`.
- Restart and verify: `systemctl --user restart ralph-event-watch.service`.

5. Validate runtime stability.
- Confirm unit is not in `activating (auto-restart)` loop.
- Validate:
  - `systemctl --user status ralph-event-watch.service --no-pager` shows stable state,
  - `journalctl --user -u ralph-event-watch.service -n 120 --no-pager` has no missing-script failures after restart.

6. Add regression guard for stale absolute path drift.
- Add a lightweight check in Ralph tooling/docs (for example in `.ralph/ralph-status.sh` or task notes) that flags if user unit `ExecStart` points to missing file.
- Keep this guard non-invasive (diagnostic only) to avoid side effects.

7. Update task bookkeeping during execution.
- Mark checklist boxes progressively while executing (not during planning).
- Update `<status>` and `<passes>` only after all required gates pass.

8. Run full required gates (no skipping).
- `make check`
- `make test`
- `make test-bdd`
- `make lint`
- Use known stabilizers from workspace learnings when running heavy suites:
  - `CARGO_BUILD_JOBS=1`
  - `CARGO_INCREMENTAL=0`
  - `RUST_TEST_THREADS=1`
- Record logs/evidence and grep markers where required by task criteria.

9. Finalization sequence (only after all gates pass).
- Set `<passing>true</passing>` in this task file.
- Run `/bin/bash .ralph/task_switch.sh`.
- Commit all changed files (including `.ralph` artifacts) using:
  - `task finished ralph-event-watch-missing-script-path: <summary + gate evidence + implementation notes>`
- `git push`.
- Append any new cross-task learning to `AGENTS.md`.
</plan>

NOW EXECUTE
