## Bug: Legacy RuntimeConfigV2 Test Converter Drops API And DCS TLS <status>not_started</status> <passes>false</passes>

<description>
`src/dev_support/runtime_config_v2.rs` currently converts legacy `RuntimeConfig` into `RuntimeConfigV2` for test and harness code, but it silently degrades TLS settings instead of preserving or rejecting them.

Today the converter maps legacy `api.transport = https` to `ApiTransport::Http`, and it maps legacy `dcs.client.tls = enabled` to `None`. That means a test or harness that asks for HTTPS or DCS TLS can accidentally run with weaker transport than requested while still appearing to succeed.

Explore the existing config-v2 TLS shapes and the dev-support callers first, then make the converter faithful. If a legacy TLS input cannot be represented in `RuntimeConfigV2`, return a clear error instead of silently downgrading it.
</description>

<acceptance_criteria>
- [ ] `src/dev_support/runtime_config_v2.rs` either preserves legacy API/DCS TLS in `RuntimeConfigV2` or returns an explicit error for unsupported legacy TLS inputs
- [ ] No test or harness path silently downgrades HTTPS or DCS TLS to plaintext/disabled transport
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
