## Bug: Test child cleanup ignores process termination errors <status>not_started</status> <passes>false</passes>

<description>
The `Drop` cleanup for the fake postgres child in `src/api/worker.rs` ignores failures from `child.kill()` and `child.wait()` with `let _ = ...`.
This swallows cleanup failures and can hide broken test behavior or leaked subprocesses.
Explore and research the codebase first, then fix the cleanup path so termination failures are surfaced or recorded instead of ignored.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
