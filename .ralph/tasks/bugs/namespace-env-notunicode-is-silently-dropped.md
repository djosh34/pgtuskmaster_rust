## Bug: Namespace env var NotUnicode is silently dropped <status>not_started</status> <passes>false</passes>

<description>
`src/dev_support/namespace.rs` treats `std::env::VarError::NotUnicode` from `PGTM_TEST_KEEP_NAMESPACE` as `false`, which silently discards an actual error.

The agent should explore and research the codebase first, then fix this by preserving the failure mode instead of flattening it into a successful default.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
