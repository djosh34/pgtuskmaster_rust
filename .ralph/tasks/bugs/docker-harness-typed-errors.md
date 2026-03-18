## Bug: Type Docker harness failures instead of matching stderr strings <status>not_started</status> <passes>false</passes>

<description>
The Docker test harness in tests/ha/support/docker still infers control flow from stderr substrings and emits generic HarnessError::message failures for distinct cases like compose network races, missing inspect fields, empty Ryuk container IDs, and unexpected registration acknowledgements.
This makes failures harder to classify and easy to mask. Explore the existing error types first, then replace these stringly paths with typed variants or structured errors where the caller can branch without parsing messages.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
