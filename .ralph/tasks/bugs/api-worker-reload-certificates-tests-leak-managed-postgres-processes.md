## Bug: Api Worker Reload Certificates Tests Leak Managed Postgres Processes <status>not_started</status> <passes>false</passes>

<description>
`make test` currently exits zero, but nextest reports five leaky tests in `api::worker::tests::reload_certificates_*`:

- `reload_certificates_returns_error_when_postmaster_data_dir_mismatches`
- `reload_certificates_returns_error_when_postmaster_pid_is_stale`
- `reload_certificates_returns_error_when_https_runtime_sees_http_config`
- `reload_certificates_does_not_signal_postgres_when_api_reload_fails`
- `reload_certificates_succeeds_for_https_transport_and_signals_postgres`

That means the reload-certificate test path is leaving background resources alive after test completion, which makes the suite less trustworthy and can hide real lifecycle bugs. Explore and research the api worker reload path and its managed postmaster test setup first, then fix the leak at the owning boundary instead of papering over it in test cleanup.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
