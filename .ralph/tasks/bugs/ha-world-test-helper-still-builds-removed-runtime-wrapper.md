## Bug: HA world test helper still builds removed runtime wrapper <status>not_started</status> <passes>false</passes>

<description>
`cargo nextest run support::config::tests::resolve_configured_executable_rejects_relative_path` exposed a compile failure in `tests/ha/support/world/mod.rs`.
The test helper `test_harness_with_write_convergence` still constructs `HarnessShared { runtime: HarnessRuntime { ... } }`, but `HarnessRuntime` is no longer the live shape of `HarnessShared`.
Explore and research the codebase first, then fix the stale test helper so the `ha` test target compiles under `cargo nextest`.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
