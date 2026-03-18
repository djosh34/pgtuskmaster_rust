## Bug: test-long takes far too long with zero tests passing for minutes <status>not_started</status> <passes>false</passes>

<description>
`make test-long` is taking far too long to show any passing HA tests.

Observed behavior:
- after roughly 6 minutes of `make test-long`, zero tests had passed
- this is far slower than expected for the current HA harness behavior
- something is likely wrong in the startup, scheduling, timeout, per-scenario progress path, or the long-test runner itself
- all HA long tests are supposed to be independent and parallel-safe, so we expect tests to start completing without waiting behind one stuck scenario

Explore and research the long-test runner, `nextest`/test-runner behavior, HA harness startup path, scenario scheduling, and timeout model first, then fix the issue so `make test-long` starts producing passing tests much sooner instead of appearing stalled for minutes.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Research summary
- `make test-long` is only a thin wrapper around `cargo nextest run --workspace --profile ultra-long --test ha`; there is no serial loop or explicit one-by-one runner in [Makefile](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/Makefile).
- The `ultra-long` profile is intentionally configured for HA fanout, not serialization: it filters only `binary(ha) & test(/^ha_/)`, sets `test-threads = 128`, and documents that HA scenarios are expected to be parallel-safe in [.config/nextest.toml](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.config/nextest.toml).
- On this machine, the existing JUnit artifact at [target/nextest/ultra-long/junit.xml](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/target/nextest/ultra-long/junit.xml) shows the setup script completed in about 3.3s, then all 16 HA tests started together and were still running roughly 366s later when the run was interrupted. That rules out nextest setup as the dominant several-minute delay here.
- The real front-loaded cost is inside the HA harness. Every scenario begins by calling `HarnessShared::initialize(...)` from the first `Given`, so each Rust test pays for full fixture materialization, Docker verification, DCS startup, cluster bootstrap, and background invariant startup before the scenario can make progress in [tests/ha/support/steps/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/steps/mod.rs) and [tests/ha/support/world/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/world/mod.rs).
- That initialization is internally staged and partly serialized: wait for DCS health, start only `node-b`, wait for the seed primary to become authoritative, then start `node-a` and `node-c` in [tests/ha/support/world/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/world/mod.rs).
- The harness also starts both perpetual invariants during initialization, including the heavier write-convergence invariant which opens PostgreSQL connections to all members, initializes the fixture table, and blocks until the fixture is visible everywhere before the scenario even starts in [tests/ha/support/world/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/world/mod.rs) and [tests/ha/support/invariants/write_convergence.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/invariants/write_convergence.rs).
- The current timeout caps are bounded (`startup <= 20s`, `failover <= 20s`, `recovery <= 30s`, `write_convergence <= 15s`) in [tests/ha/support/timeouts/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/timeouts/mod.rs), so the suite-level “minutes before any pass” behavior is coming from repeated per-scenario cold-start phases and invariant startup churn under full parallel fanout, not from a single runaway timeout constant.

### Revised design direction
- The rejected plan was wrong because it made invariant behavior explicit in scenario state and feature flow. That must not continue.
- Both invariants must be real independent runners, not explicit feature or step requirements. Scenario steps must not activate them, sequence them, or contain any direct success requirement for them.
- Write-convergence startup should only await invariant start in the narrow sense of connectivity: once a node is connected enough to succeed with `SELECT 1`, the invariant is considered started and good to run.
- After that startup point, write-convergence must be fully independent. Features and steps must not alter it, coordinate it, or declare when it should begin doing real work.
- The write-convergence runner must always try to add `1` into the shared table from each node at the same time. Failures to write are not test failures by themselves; they are retry conditions.
- Disconnects are also retry conditions for write-convergence. The runner must reconnect and retry forever instead of failing on disconnect or temporary write errors.
- Write-convergence may fail the test only in this situation: `ensure_health` is called and, after the relevant timeout expires, the count reported by each node is not equal to the atomic written count maintained by the invariant.
- Primary-count must also remain always-on and independent from test scenarios. It should keep polling forever through normal runner behavior instead of being treated as a per-step gate.
- Primary-count observation must stay simple and consistent: if checking a node errors, treat that node as not-primary for that observation; if checking succeeds, count it as primary only if the node reports itself as primary.
- If the total reported primary count is ever greater than `1`, fail the test immediately with no ensure-health grace period.
- Legitimate transitional `0`-primary windows must not fail general scenario progress on their own. They matter only if the health/ensure window expires without returning to exactly one primary.
- `ensure_health` for the primary invariant must use the same primary-detection function as the always-running invariant. Its job is only to fail on timeout when the cluster does not reach exactly one primary; it should not use separate logic.
- Invariant ownership belongs inside invariant variants and runner state, not in feature wording or scenario-step contracts. The feature should describe cluster behavior; the invariant implementation should own how it keeps trying, reconnects, and reports failure.
- The suite must keep HA scenarios independent and parallel-safe. The harness design should let tests run while invariant runners operate independently beside them.

### Revised execution plan
1. Revert the explicit write-convergence activation plan and any design that makes invariant readiness a scenario-step concern.
2. Refactor the harness so scenario execution and invariant runners are separate independent concerns, with invariant lifecycle owned by the invariant runner state itself.
3. Make write-convergence startup wait only for invariant connectivity readiness (`SELECT 1`-level availability), then leave it fully independent from features for the rest of the run.
4. Make write-convergence loop forever: attempt the concurrent per-node increments, treat disconnects and write failures as retry-only conditions, and reserve test failure for ensure-health timeout where node-visible counts do not match the invariant's atomic written count.
5. Make primary-count an always-independent polling runner using one shared primary-detection function for both background observation and ensure-health checks.
6. Fail immediately on observed split-brain (`>1` primary), but otherwise only let ensure-health fail on timeout when the cluster does not return to exactly one primary.
7. Validate with the required repo gates in order:
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`
8. Only after all checks pass, set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit, and push.

### Constraints for execution
- Do not serialize the HA suite or weaken the parallel-safe contract.
- Do not mask the issue with larger timeout ceilings or blanket retries at the scenario level.
- Do not make any feature or scenario step explicitly activate, await, or declare success for primary-count or write-convergence beyond the narrow write-convergence startup readiness of "connected enough for `SELECT 1`".
- Do not treat legitimate `0`-primary transition windows as automatic failure outside timeout-backed ensure-health logic.
- Do not let write-convergence become best-effort or optional; it must remain a real invariant runner that keeps trying independently forever.
- Do not fail write-convergence on temporary disconnects, failed writes, or other retryable errors; only the ensure-health timeout mismatch between node-visible counts and the atomic written count should fail that invariant.
- Do not use separate logic for primary detection in ensure-health versus the primary invariant loop.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if needed before the final `make` gates.
- If execution discovers a new design gap, switch back to redesign notes again and stop rather than forcing through another invalid plan.

### Reverted execution feedback
- The prior execution attempt should not continue because the design moved invariant policy into scenario flow.
- Treating primary-count as a blanket per-step readiness gate was incorrect because it punished legitimate `0`-primary transitions and split primary detection away from ensure-health semantics.
- Treating write-convergence as an explicit activation boundary was also incorrect because the invariant must become independent after narrow connectivity startup and then keep reconnecting/retrying rather than being feature-driven.
- The next valid execution pass must preserve invariant independence while allowing HA scenarios to run in parallel without waiting on explicit invariant activation steps.
- The current type direction is now explicit in code: the world-side write-convergence lifecycle is being flattened to invariant-owned `NotStarted` / `Starting` / `Running` / `Failed` state, with no scenario `InvariantRequirement` boundary.
- The next execution pass should finish the flattening inside `write_convergence.rs` by replacing the one-shot `Client + fatal_error + connection_task` member model with reconnectable member workers, then repair the compile fallout and run the required gates.

NOW EXECUTE
