## Bug: test-long takes far too long with zero tests passing for minutes <status>not_started</status> <passes>false</passes>

<description>
`make test-long` is taking far too long to show any passing HA tests.

Observed behavior:
- after roughly 6 minutes of `make test-long`, zero tests had passed
- this is far slower than expected for the current HA harness behavior
- something is likely wrong in the startup, scheduling, timeout, per-scenario progress path, or the long-test runner itself
- all HA long tests are supposed to be independent and parallel-safe, so we expect tests to start completing without waiting behind one stuck scenario

Explore and research the long-test runner, `nextest`/test-runner behavior, HA harness startup path, scenario scheduling, and timeout model first, then fix the issue so `make test-long` starts producing passing tests much sooner instead of appearing stalled for minutes.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
