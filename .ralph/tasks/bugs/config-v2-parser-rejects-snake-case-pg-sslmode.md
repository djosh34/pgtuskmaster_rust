## Bug: Config v2 parser rejects snake_case postgres sslmode values <status>not_started</status> <passes>false</passes>

<description>
`config_v2::parser::load_config::tests::load_runtime_config_preserves_shared_source_client_tls` failed during `make test` because TOML parsing rejected `postgres.rewind.transport.ssl_mode = "verify_full"` with `unsupported sslmode`.

Explore the existing `PgSslMode` parsing boundary first, then fix the shared enum/parser so config-v2 ingestion accepts the supported postgres sslmode spellings without creating a duplicate config-only enum.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
