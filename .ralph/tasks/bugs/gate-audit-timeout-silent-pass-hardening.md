---
## Bug: Harden make gates against hangs and silent passes <status>not_started</status> <passes>false</passes>

<description>
`make test`, `make test-long`, `make lint`, and `make check` currently have uneven timeout behavior and incomplete pass assertions.

Observed issues from audit:
- `make test-long` has no timeout wrapper around `cargo test` executions, so one stalled real-binary test can block forever.
- `make test` has a timeout only around the final `cargo test` run, but not around preflight `cargo test -- --list`.
- `make lint` and `make check` have no timeout bounds for docs scripts / `cargo clippy` / `cargo check`.
- docs no-code guard scans only selected docs subtrees and only fences that begin with ` ``` ` at column 1, which can miss forbidden code blocks and create false confidence.
- Gate evidence logs are mostly raw stdout without normalized per-step start/end timestamps, exit codes, durations, or timeout forensics.

Please explore and research the codebase first, then implement a robust, fail-closed fix set.
</description>

<acceptance_criteria>
- [ ] `make test` bounds both preflight and execution phases with explicit timeouts, and fails with clear diagnostics on timeout.
- [ ] `make test-long` bounds each preflight and per-test execution with explicit timeouts; a stuck ultra-long test cannot hang forever.
- [ ] `make lint` and `make check` run under bounded timeouts (or equivalent watchdog) with deterministic non-zero failure on timeout.
- [ ] docs architecture no-code guard covers intended docs roots and fence patterns without easy bypasses (leading whitespace / moved docs directories).
- [ ] Gate scripts/targets emit structured evidence for each step: command, start/end UTC, duration, exit status, and timeout marker when applicable.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
