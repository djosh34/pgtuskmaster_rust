## Bug: Reduce verbose error handling to typed boundaries <status>not_started</status> <passes>false</passes>

<description>
The codebase still has several early-error-flattening patterns:

- `map_err(|err| err.to_string())`
- `map_err(|err| format!(...))`
- `Err(format!(...))`
- `Result<_, String>`
- `Message(String)` / other catch-all string buckets

These appear in production code and test helpers. They hide the real cause too early and force string parsing where typed branching should work. Explore the codebase first, identify the worst offenders, and reduce them to typed enums using `thiserror` and `#[from]` so call sites can use `?` directly.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
