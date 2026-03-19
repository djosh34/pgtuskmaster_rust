## Bug: API worker reload certificate tests leak resources under nextest <status>not_started</status> <passes>false</passes>

<description>
`make test` currently passes but nextest reports leaky tests in `pgtuskmaster_rust::api::worker::tests`:
`reload_certificates_returns_error_when_postmaster_pid_is_stale`,
`reload_certificates_returns_error_when_postmaster_data_dir_mismatches`,
`reload_certificates_returns_error_when_https_runtime_sees_http_config`,
`reload_certificates_does_not_signal_postgres_when_api_reload_fails`,
and `reload_certificates_succeeds_for_https_transport_and_signals_postgres`.
Explore and research the codebase first, then fix the test harness or underlying runtime cleanup so the default nextest suite passes cleanly without leak reports.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
