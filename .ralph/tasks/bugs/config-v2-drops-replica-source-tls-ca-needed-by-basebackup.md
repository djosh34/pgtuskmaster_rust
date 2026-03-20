## Bug: Config v2 drops replica source TLS CA needed by basebackup <status>not_started</status> <passes>false</passes>

<description>
Ultra-long HA scenarios now progress past seed-primary bootstrap, but replica startup still fails because `pg_basebackup` is launched without the configured TLS root CA.

The preserved HA artifacts show repeated replica-side errors like:
`pg_basebackup: error: connection to server at "node-b" ..., port 5432 failed: root certificate file "/var/lib/postgresql/.postgresql/root.crt" does not exist`

This happens because config-v2 parsing currently discards `postgres.rewind.transport.ca_cert`. `RuntimeConfigV2` only keeps the SSL mode on the mandatory role config, and `process::source::source_from_member` therefore materializes source conninfo without the CA path required for basebackup/rewind verification.

Explore and research the codebase first, then fix.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
