## Bug: Preserve typed JSON output errors <status>not_started</status> <passes>false</passes>

<description>
`src/cli/output.rs` converts `serde_json::Error` into a formatted `String` before it reaches the CLI error boundary. That flattens the source error too early and makes it impossible to preserve structured handling in `CliError`.

Explore the output path and error enum first, then refactor the error flow so the lower helper does not build the user-facing message before the top-level error type formats it.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
