## Bug: CLI client drops file-backed API TLS material <status>not_started</status> <passes>false</passes>

<description>
The CLI API client accepts `CliTlsConfig` with CA/client certificate/client key paths, but `src/cli/client.rs` only consumes the inline PEM byte fields and silently ignores the file-backed fields populated by operator config loading.

This was detected while debugging `make test-long`: the HA harness materialized correct HTTPS observer configs and the node API was listening, but `pgtm status` still failed with `transport error: error sending request for url (...)` because the CLI never loaded the configured CA or client identity from disk.

Explore and research the codebase first, then fix.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
