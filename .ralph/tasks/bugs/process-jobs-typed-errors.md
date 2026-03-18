## Bug: Process job errors still flatten typed failures into strings <status>not_started</status> <passes>false</passes>

<description>
`src/process/jobs.rs` still uses string payloads for several `ProcessError` variants, and it converts structured errors to `String` via `to_string()` in the secret-resolution path.

This loses machine-readable error data that already exists in nearby code, especially compared to `src/process/postmaster.rs` and `src/config/materialize.rs`, which keep path/pid/env context structured.

Please explore the codebase first, then replace these stringly error payloads with typed fields or source errors where practical.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
