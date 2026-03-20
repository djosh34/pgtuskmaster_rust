## Bug: Incremental nextest builds drop objects and rlib members <status>not_started</status> <passes>false</passes>

<description>
`make test` and focused `cargo nextest run` invocations can fail during the build phase with linker/archive errors such as `ld: cannot find ... .rcgu.o` or `failed to build archive ... .rlib: failed to open object file`. The failures point at missing members under `target/aarch64-unknown-linux-gnu/debug/deps` while `CARGO_INCREMENTAL=1` is enabled.

This was reproduced on 2026-03-20 after the config_v2 reduction branch rebuilt test targets multiple times. Explore the test build invocation, target-dir behavior, and incremental settings first, then fix the unstable build path in the narrowest way that keeps the required checks reliable.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
