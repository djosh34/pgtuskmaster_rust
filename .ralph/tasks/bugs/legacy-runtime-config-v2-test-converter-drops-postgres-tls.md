## Bug: Legacy RuntimeConfigV2 Test Converter Drops Postgres TLS <status>not_started</status> <passes>false</passes>

<description>
`src/dev_support/runtime_config_v2.rs` currently converts legacy `RuntimeConfig` into `RuntimeConfigV2` for test and harness code, but it silently drops `postgres.tls` by always setting the v2 field to `None`.

This was detected while updating managed-postgres tests: a test that configures legacy Postgres TLS through the builder now loses that identity/client-auth material during conversion, so managed TLS files are never materialized.

Explore the existing config-v2 postgres TLS shape and the dev-support callers first, then make the converter faithful. If a legacy Postgres TLS input cannot be represented in `RuntimeConfigV2`, return a clear error instead of silently discarding it.
</description>

<acceptance_criteria>
- [ ] `src/dev_support/runtime_config_v2.rs` either preserves legacy Postgres TLS in `RuntimeConfigV2` or returns an explicit error for unsupported legacy TLS inputs
- [ ] No test or harness path silently drops configured Postgres TLS
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
