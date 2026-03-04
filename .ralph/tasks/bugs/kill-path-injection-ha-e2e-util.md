---
## Bug: HA e2e util executes PATH-resolved kill for process control <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
`src/test_harness/ha_e2e/util.rs` uses `tokio::process::Command::new("kill")` both to send signals and to probe liveness (`kill -0`).

This allows a PATH-prepended fake `kill` binary/script to be executed by tests or harness code, causing incorrect behavior and creating command-injection surface in test environments.

Investigate and replace shell-command `kill` usage with direct syscall-based signaling/liveness checks (or another non-PATH-resolved mechanism), then add regression tests that prove PATH-prepended fake `kill` is never executed.

Explore the surrounding harness usage first and then implement the fix.

Resolved via `bug-test-harness-runtime-path-dependent-kill-command` (replaced all harness `kill` invocations with syscall-based helpers and added PATH-fake regression coverage).
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
