## Bug: Nextest incremental linking can miss ha test object files <status>not_started</status> <passes>false</passes>

<description>
`cargo nextest run --workspace --all-targets --profile default --no-tests fail` failed during verification while linking test `ha`.

Observed failure:
`/usr/bin/ld: cannot find .../target/aarch64-unknown-linux-gnu/debug/deps/ha-...rcgu.o: No such file or directory`

The same suite passed on immediate retry with `CARGO_INCREMENTAL=0`, so this currently looks like an incremental-build or target-artifact stability problem rather than a Rust compile error in the source tree.

Explore and research the codebase and build setup first, then fix the root cause instead of just papering over the retry path.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Repeated `cargo nextest run --workspace --all-targets --profile default --no-tests fail` runs with incremental enabled do not fail with missing generated object files for `ha`
</acceptance_criteria>
