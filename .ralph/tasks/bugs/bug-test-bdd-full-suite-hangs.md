---
## Bug: test full-suite command hangs in real HA e2e <status>done</status> <passes>true</passes>

<description>
After updating `make test` to run `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1 cargo test --all-targets -- --include-ignored`, the verification run did not complete within an extended runtime window (over 15 minutes observed on 2026-03-03).

Detection details:
- `make test` started and progressed through unit + real-binary tests.
- Output stalled at `ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix has been running for over 60 seconds` and never completed during the observed window.
- Active `etcd` and `postgres` child processes for the e2e namespace remained running until manual interruption.

Please explore and research the codebase first to identify whether this is a genuine deadlock/hang or an expected runtime regression, then implement the fix.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft 2, deep skeptical verification: 2026-03-03)

### Skeptical verification deltas from Draft 1
- Altered plan item (mandatory change): add a hard overall timeout to `e2e_multi_node_real_ha_scenario_matrix` itself, not only per-operation wrappers. This ensures the test cannot hang indefinitely even if a new unbounded call is introduced later.
- Scoped down timeline changes: keep major scenario markers and add request-level markers in `send_node_request(...)`, instead of instrumenting every external command call site, to avoid overfitting and preserve readability.
- Kept command timeout hardening and HTTP I/O deadlines because these are concrete unbounded waits in the current implementation.

### Verified findings from deep review
- The 60-second line is a test harness heartbeat, not a failure by itself; prior logs show many successful runs after that line.
- `cargo test --all-targets -- --ignored --list` returns 0 ignored tests, so `--include-ignored` does not add hidden ignored workloads today.
- `src/ha/e2e_multi_node.rs` has known unbounded operations:
- `initialize_pgdata(...)` uses `Command::status().await` with no timeout.
- `pg_ctl_stop_immediate(...)` uses `Command::output().await` with no timeout.
- `send_http_request_with_worker(...)` has unbounded connect/write/read operations.

### Execution plan
1. Add bounded command execution helper(s) for real e2e subprocesses in `src/ha/e2e_multi_node.rs`.
- Introduce a helper that spawns a child process, waits with `tokio::time::timeout`, kills on timeout, and returns a structured `WorkerError` including command label and timeout duration.
- Refactor `initialize_pgdata(...)` and `pg_ctl_stop_immediate(...)` to use this helper.
- Preserve existing "already stopped" behavior for `pg_ctl stop` without panic/unwrap/expect paths.

2. Add bounded HTTP + worker-step timeouts in `send_http_request_with_worker(...)`.
- Wrap TCP connect, request writes, `debug_api::worker::step_once`, `api::worker::step_once`, and `read_to_end` with explicit timeout deadlines.
- Include node/method/path context in timeout errors.

3. Add an overall scenario timeout guard around `e2e_multi_node_real_ha_scenario_matrix`.
- Wrap the scenario body in `tokio::time::timeout` with a conservative upper bound (long enough for real binaries, finite for hangs).
- Ensure timeout errors still flow through existing timeline artifact + shutdown reporting.

4. Improve timeline diagnostics at request granularity.
- In `send_node_request(...)`, emit start/success/failure markers for each API call to improve stall localization without excessive noise.

5. Validate the required gates and task acceptance.
- Run and verify, in order:
- `make check`
- `make test`
- `make test-long`
- `make lint`
- If all pass, update task checklist and `<passes>true</passes>`, then run task switch + commit with required message format.
</execution_plan>
