## Bug: Reload certificates tests leak processes <status>not_started</status> <passes>false</passes>

<description>
`make test` passed on 2026-03-19, but `cargo nextest` reported 5 leaky tests in `api::worker::tests::{reload_certificates_returns_error_when_postmaster_data_dir_mismatches,reload_certificates_returns_error_when_postmaster_pid_is_stale,reload_certificates_does_not_signal_postgres_when_api_reload_fails,reload_certificates_returns_error_when_https_runtime_sees_http_config,reload_certificates_succeeds_for_https_transport_and_signals_postgres}`.
Explore and research the codebase first, identify which child process or resource is not being cleaned up in these test paths, then fix the leak so the default test suite is clean.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
