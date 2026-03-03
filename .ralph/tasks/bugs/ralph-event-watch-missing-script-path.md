---
## Bug: ralph-event-watch service restart loop from missing script path <status>not_started</status> <passes>false</passes>

<description>
`ralph-event-watch.service` is in continuous auto-restart with exit code 127 because `ExecStart` points to a non-existent script path:
`/home/joshazimullah.linux/work_mounts/projects/postgres_operator/PGTuskMaster/ElixirPGTuskMaster/.ralph/event_watch.sh`.

Detected on March 3, 2026 while checking Ralph systemd health. `systemctl --user status ralph-event-watch.service` shows `activating (auto-restart)` and `journalctl --user -u ralph-event-watch.service` repeatedly logs `No such file or directory`.

Explore and research the codebase and service configuration first, then fix the broken `ExecStart` target/path so the service can remain stable instead of looping.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
