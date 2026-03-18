## Bug: process postmaster reload SIGHUP test times out with no signal log <status>not_started</status> <passes>false</passes>

<description>
`make test` currently fails in `process::postmaster::tests::reload_managed_postmaster_sends_sighup`.

Observed behavior:
- the test reproduces in isolation with `cargo nextest run --workspace reload_managed_postmaster_sends_sighup`
- the fake managed postmaster process never writes the expected signal log entry within the test timeout
- this blocks the required default test gate for unrelated tasks

Explore and research the fake postmaster signal harness first, then fix the signal delivery or observation path so the reload test becomes deterministic and `make test` passes cleanly again.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
