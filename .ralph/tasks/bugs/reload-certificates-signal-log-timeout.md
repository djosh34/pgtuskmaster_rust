## Bug: Reload certificates HTTPS test times out waiting for signal log <status>not_started</status> <passes>false</passes>

<description>
`make test` currently fails in `pgtuskmaster_rust::api::worker::tests::reload_certificates_succeeds_for_https_transport_and_signals_postgres`.
The observed error is `signal log /tmp/pgtm-api-worker-reload-success-533849-1773863709231/signal.log was not written in time`.
Explore and research the codebase first, then fix the test or its underlying runtime behavior so the default nextest suite passes cleanly again.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
