## Bug: Process postmaster reload sighup test times out waiting for signal log <status>not_started</status> <passes>false</passes>

<description>
`make test` currently fails in the default suite at `process::postmaster::tests::reload_managed_postmaster_sends_sighup`.

Observed failure:
- `signal log /tmp/.../signal.log was not written in time`

This was observed while verifying the HA write-convergence DSN bug fix, so it appears unrelated to that invariant path. Explore and research the postmaster reload test and fake postgres helper first, then fix the timeout or signal-delivery behavior so the default suite is reliable again.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
