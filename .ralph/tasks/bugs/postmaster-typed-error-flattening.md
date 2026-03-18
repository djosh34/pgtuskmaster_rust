## Bug: Preserve typed postmaster errors <status>not_started</status> <passes>false</passes>

<description>
`src/process/postmaster.rs` still converts several known failure shapes into `String` payloads before they reach `ManagedPostmasterError`. That flattens typed sources like `io::Error`, `ParseIntError`, and `TryFromIntError` into formatted text, which makes matching and refactoring harder.

Inspect `src/process/postmaster.rs` first, compare it with the typed error patterns in `src/process/cluster.rs` and `src/state/errors.rs`, then replace the string payloads with typed variants where the source error shape is already known.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
