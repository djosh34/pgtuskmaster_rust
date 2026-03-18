## Bug: Refine stringly process errors <status>not_started</status> <passes>false</passes>

<description>
The `src/process` module still uses several stringly error variants and `format!`-built `InvalidSpec` paths where the failure shape is known at the call site. This makes error handling harder to match on and hides typed context in strings.

Inspect `src/process` first, then replace the string payloads with typed variants or dedicated error enums where the data shape is already known.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
