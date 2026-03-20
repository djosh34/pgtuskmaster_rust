## Bug: Config V2 DCS HTTPS Endpoints Miss Required TLS Validation <status>not_started</status> <passes>false</passes>

<description>
`src/config_v2/parser/load_config.rs` currently accepts `dcs.endpoints` entries with the `https://` scheme even when `dcs.client.tls` is absent. The legacy parser rejected that configuration with a stable `dcs.client.tls` validation error, but the config-v2 path now lets startup proceed until it fails later in unrelated runtime preparation.

This regression was detected by `tests/cli_binary.rs::node_rejects_https_dcs_without_tls_config`, which now reports a filesystem permission error instead of the expected stable field path. Explore the existing config-v2 DCS validation path first, then restore the missing invariant without rebuilding legacy config shapes.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
