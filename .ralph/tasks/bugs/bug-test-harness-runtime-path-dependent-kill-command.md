---
## Bug: Test harness runtime kill command is PATH-dependent and bypasses provenance guarantees <status>not_started</status> <passes>false</passes>

<description>
Real-binary harness paths are intended to be explicit and provenance-controlled, but runtime teardown logic still invokes `kill` by bare name via `Command::new("kill")`.

This creates a PATH-dependent execution path in real-binary e2e runs:
- `src/test_harness/pg16.rs` uses `Command::new("kill")` during postgres child shutdown.
- `src/test_harness/etcd3.rs` uses `Command::new("kill")` during etcd member shutdown.
- `src/test_harness/ha_e2e/util.rs` uses `Command::new("kill")` for postmaster fallback signal and liveness probes.

If PATH is accidentally or maliciously modified in CI/dev shells, teardown and fallback signaling may execute an unexpected binary, violating provenance assumptions and potentially producing false diagnostics.

Please explore and research the codebase first, then implement a fail-closed fix:
- remove PATH-dependent `kill` invocations (prefer direct signal APIs or absolute trusted path resolution),
- add a deterministic regression test/negative control proving PATH-prepended fake `kill` is not executed.
</description>

<acceptance_criteria>
- [ ] Teardown and fallback signaling paths in the test harness no longer execute PATH-resolved `kill` binaries.
- [ ] Add a deterministic test/fixture that prepends a fake `kill` in PATH and proves harness runtime paths do not execute it.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
