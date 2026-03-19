## Bug: make test nextest build fails with missing rlib archive <status>not_started</status> <passes>false</passes>

<description>
`make test` currently fails during the `cargo nextest run --workspace --all-targets --profile default --no-tests fail` build step before any tests execute.

Observed on March 19, 2026 while validating a conninfo boundary refactor:

`failed to build archive at target/aarch64-unknown-linux-gnu/debug/deps/libpgtuskmaster_rust-80c89a10b912d071.rlib: failed to open object file: No such file or directory (os error 2)`

The target directory contained matching `libpgtuskmaster_rust-*.rmeta` files but no corresponding `.rlib` for the hash in the failure. Explore and research the codebase and build configuration first, then fix the underlying archive/build issue instead of papering over it.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
