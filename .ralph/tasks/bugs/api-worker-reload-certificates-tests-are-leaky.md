## Bug: API worker reload-certificates tests are leaky <status>not_started</status> <passes>false</passes>

<description>
`make test` passed on 2026-03-19, but nextest reported 5 leaky tests in `api::worker::tests`:

- `reload_certificates_returns_error_when_postmaster_data_dir_mismatches`
- `reload_certificates_returns_error_when_postmaster_pid_is_stale`
- `reload_certificates_returns_error_when_https_runtime_sees_http_config`
- `reload_certificates_does_not_signal_postgres_when_api_reload_fails`
- `reload_certificates_succeeds_for_https_transport_and_signals_postgres`

Explore and research the codebase first, then fix the underlying resource leak instead of suppressing the signal.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
