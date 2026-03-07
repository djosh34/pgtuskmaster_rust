## Task: Audit and replace magic numbers project-wide <status>done</status> <passes>true</passes> <priority>low</priority>

<description>
Audit the project for unexplained magic numbers and replace them with explicit typed constants, configuration, or otherwise well-justified named values.

The agent must explore the whole codebase first, not only HA, then implement the following fixed product decisions:
- this is a project-wide cleanup, not only an `src/ha/state.rs` cleanup
- unexplained magic numbers should be checked everywhere in runtime code, tests, harness code, and supporting modules
- values that are real product knobs should become explicit config or typed settings where appropriate
- values that are fixed implementation constants should become clearly named constants with obvious ownership
- purely arbitrary numeric literals that remain must be justified by local meaning, not left as unexplained inline numbers

This is intentionally low priority and should not preempt the architectural rewrite stories, but the final codebase should not keep accumulating unexplained numeric literals.

The agent should use parallel subagents after exploration to audit different codebase slices and then apply the cleanup coherently.
</description>

<acceptance_criteria>
- [x] Project-wide audit covers runtime code, tests, harness code, and supporting modules rather than only HA
- [x] Unexplained magic numbers are replaced by config, typed settings, or clearly named constants where appropriate
- [x] Remaining numeric literals are locally justified by obvious meaning rather than unexplained inline usage
- [x] The cleanup does not introduce bogus configurability where fixed constants are the better design
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
</acceptance_criteria>

## Execution plan

### 1. Research summary and current hotspot map

The first audit pass already shows that the cleanup must be broader than `src/ha/state.rs`. The highest-value hotspots are spread across runtime code, shared test helpers, and long-running HA scenario support:

- Runtime implementation hotspots:
  - `src/api/worker.rs`
    - API loop and socket timeouts (`10ms`, `1ms`, `100ms`)
    - request-id truncation cap (`128`)
    - request parsing limits and scratch-buffer sizes (`1024 * 1024`, `16 * 1024`, `4096`)
  - `src/process/worker.rs`
    - inline stream-drain sizes (`8192`, `256 * 1024`)
    - tiny scheduling waits (`Duration::from_millis(1)`)
    - repeated default-ish test timeouts and poll intervals inside in-file tests
  - `src/logging/postgres_ingest.rs`
    - rate-limit window (`30_000`)
    - per-file ingest cap (`256 * 1024`)
    - repeated test waits and harness timeouts in the large in-file test module
  - `src/test_harness/ports.rs`
    - retry budget (`200`)
    - lease TTL (`15 * 60`)
    - short polling sleeps (`5ms`)
  - `src/test_harness/ha_e2e/startup.rs`
    - inline startup and bootstrap durations (`15s`, `30_000ms`)
    - embedded runtime-config JSON literals that duplicate logging and HA timing values

- Shared fixture / builder hotspots:
  - `src/test_harness/runtime_config.rs`
    - repeated sample addresses and ports (`127.0.0.1:8080`, `5432`, `18080`)
    - repeated sample logging/process values (`200`, `1000`, `300`)
  - `examples/debug_ui_smoke_server.rs`
    - duplicated example listen address and retry sleep

- Large scenario-support hotspots:
  - `tests/ha/support/multi_node.rs`
    - many repeated polling sleeps (`75ms`, `100ms`, `150ms`, `200ms`, `500ms`)
    - repeated assertion/fallback windows (`3s`, `5s`, `8s`, `10s`, `20s`, `45s`, `60s`, `90s`, `120s`, `180s`)
    - repeated workload cadence values (`250ms`, `80`, `100`)
  - `tests/ha/support/partition.rs`
    - same class of repeated sleep and timeout values as `multi_node.rs`
    - several scenario-local windows that should become clearly named constants at module scope

The initial audit also shows several literals that are already locally meaningful and should usually remain inline:

- protocol/status values in assertions (`200`, `401`, `404`, `503`) where the assertion is explicitly about the HTTP contract
- PostgreSQL default port `5432` when the fixture is specifically modeling a canonical local PostgreSQL listener
- version/tick/sequence fixture values like `Version(1)`, `UnixMillis(1)`, `tick: 0` when they are simple domain fixtures and not behavior knobs
- arithmetic identities like `0`, `1`, and saturating-add guard values when their local meaning is self-evident

### 2. Classification rules to enforce during execution

During `NOW EXECUTE`, every candidate literal must be classified before changing it:

- Promote to runtime config only if the value is a real product/operator knob already conceptually owned by config.
- Promote to a module constant if the value is a fixed implementation detail that affects behavior but should not be tuned by users.
- Promote to a shared test/support constant if the same value repeats across helpers or many scenarios and names a common testing concept.
- Leave inline if the literal is an obvious domain fixture, protocol contract, or arithmetic identity with immediate local meaning.

Concrete anti-goal: do not create bogus configurability for internal buffer sizes, retry budgets, or test polling sleeps that are not product-facing knobs.

### 3. Runtime code cleanup plan

- Re-audit the runtime modules with the densest magic-number usage and patch only the literals that are genuinely unexplained:
  - `src/api/worker.rs`
  - `src/process/worker.rs`
  - `src/logging/postgres_ingest.rs`
  - `src/ha/worker.rs`
  - `src/ha/process_dispatch.rs`
  - `src/dcs/worker.rs`
  - `src/runtime/node.rs`
- Expected runtime refactors:
  - introduce API-worker-owned constants for internal accept/read timeouts, request-id truncation, request/header size limits, and scratch-buffer sizes unless a value is already a documented external contract
  - introduce module-owned constants for subprocess output read size, per-drain byte budget, and similar fixed implementation thresholds
  - name short internal retry/poll waits where they exist to prevent repeated anonymous `from_millis(...)` calls
  - keep actual product knobs in `src/config/defaults.rs` / schema-owned config rather than inventing parallel constants elsewhere
- Before adding any new config surface, verify whether the value is already represented by:
  - `RuntimeConfig`
  - `ProcessConfig`
  - `LoggingConfig`
  - `HaConfig`
- If a repeated literal only appears in test modules inside these files, prefer test-only constants in the test module rather than widening runtime API surface.

### 4. Test harness and fixture-builder cleanup plan

- Normalize repeated harness/support values in:
  - `src/test_harness/ports.rs`
  - `src/test_harness/runtime_config.rs`
  - `src/test_harness/ha_e2e/startup.rs`
  - `src/test_harness/ha_e2e/util.rs`
  - `src/test_harness/net_proxy.rs`
  - `src/test_harness/etcd3.rs`
  - `examples/debug_ui_smoke_server.rs`
- Expected harness refactors:
  - add named constants for port-allocation retry budget, lease TTL, and short synchronization sleeps
  - centralize sample API listen addresses / PostgreSQL ports / logging poll intervals / cleanup retention values when they are used as canonical fixture defaults rather than scenario-specific data
  - eliminate duplicated inline config literals inside JSON/TOML fixture payloads when the surrounding code can source them from one builder or one named constant
- Keep literals inline when they intentionally differentiate nodes or endpoints in a scenario fixture, for example:
  - distinct member ports
  - distinct IPs
  - explicit endpoint counts tied to topology setup

### 5. HA scenario-support cleanup plan

- Audit the large support modules first, then clean them coherently instead of scattering one-off constants through top-level tests:
  - `tests/ha/support/multi_node.rs`
  - `tests/ha/support/partition.rs`
  - `tests/ha/support/observer.rs`
- Execution approach:
  - group repeated waits into semantic buckets such as poll cadence, state sampling cadence, workload shutdown grace, strict/fallback failover windows, and no-dual-primary observation windows
  - add module-level constants with names that describe the scenario intent, not merely the units
  - keep one-off scenario literals inline when they are part of the scenario itself and not reused
- Likely consolidation targets:
  - repeated `sleep` durations (`75ms`, `100ms`, `150ms`, `200ms`, `500ms`)
  - repeated fallback/strict windows (`25s`, `30s`, `35s`, `45s`, `60s`, `90s`, `120s`)
  - repeated workload cadences and sample-window parameters (`250ms`, `80`, `100`)
- Avoid over-abstracting scenario files into an unreadable constants jungle. A constant should only exist if it clarifies shared intent or removes repetition.

### 6. In-file unit test cleanup plan

- Sweep in-file test modules under `src/` after the main runtime/harness refactors so test constants align with the final ownership choices.
- Priority files for this pass:
  - `src/worker_contract_tests.rs`
  - `src/logging/postgres_ingest.rs`
  - `src/process/worker.rs`
  - `src/ha/worker.rs`
  - `src/ha/process_dispatch.rs`
  - `src/pginfo/conninfo.rs`
- Expected changes:
  - extract repeated test-only durations, ports, and buffer caps into local `const`s near the test module
  - keep fixture-domain literals inline where they improve readability more than a named constant would
  - reuse existing builders from `src/test_harness/runtime_config.rs` instead of cloning anonymous numbers into each test

### 7. Planned parallel split for the later `NOW EXECUTE` pass

Once this task reaches `NOW EXECUTE`, use parallel subagents only after the main agent has re-opened this file and confirmed the execution order below.

- Subagent A: runtime audit and cleanup
  - scope: `src/process/*`, `src/logging/*`, `src/runtime/*`, `src/dcs/*`, `src/ha/*`
- Subagent B: harness/builders/examples
  - scope: `src/test_harness/*`, `examples/*`
- Subagent C: HA/e2e support and broad in-file test cleanup
  - scope: `tests/ha/support/*`, top-level `tests/*.rs`, in-file unit tests under `src/`

The main agent keeps ownership of:

- final classification decisions for borderline literals
- any config-surface changes
- docs updates
- task-file bookkeeping and gate verification

### 8. Files most likely to change during execution

- Runtime and supporting modules:
  - `src/api/worker.rs`
  - `src/process/worker.rs`
  - `src/logging/postgres_ingest.rs`
  - `src/ha/worker.rs`
  - `src/ha/process_dispatch.rs`
  - `src/dcs/worker.rs`
  - `src/runtime/node.rs`
- Harness / fixture modules:
  - `src/test_harness/ports.rs`
  - `src/test_harness/runtime_config.rs`
  - `src/test_harness/ha_e2e/startup.rs`
  - `src/test_harness/ha_e2e/util.rs`
  - `src/test_harness/net_proxy.rs`
  - `examples/debug_ui_smoke_server.rs`
- Scenario support / tests:
  - `tests/ha/support/multi_node.rs`
  - `tests/ha/support/partition.rs`
  - `tests/bdd_api_http.rs`
  - other `src/*` test modules touched while removing duplicated literals
- Docs and bookkeeping if config or canonical fixture values change:
  - `src/config/defaults.rs`
  - relevant pages under `docs/src/`
  - this task file
  - normal `.ralph` bookkeeping files

### 9. Exact execution order for the later `NOW EXECUTE` pass

- Re-open this task file and confirm the terminal marker says `NOW EXECUTE`.
- Re-run the focused numeric audit only to gather the exact files named above; do not do a broad exploratory restart.
- Land runtime constant/config ownership first, with `src/api/worker.rs` and any `src/config/defaults.rs` ownership decisions resolved before delegating the rest, so HTTP helpers and tests do not duplicate the final names.
- Execute the parallel split on the three codebase slices after the main agent has enough context to assign bounded work against those fixed owners.
- Land harness and fixture-builder cleanup second, consolidating shared sample values and harness timing constants.
- Land HA scenario-support cleanup third, naming repeated polling and timeout windows without hiding scenario intent.
- Sweep in-file unit tests last so they align with the shared constants/builders introduced earlier.
- Update docs if the cleanup changes user-visible config knobs, canonical sample config/default values, or contributor guidance about which module owns shared constants and fixture defaults.
- Tick off acceptance criteria only after the full cleanup is done and the remaining inline literals have been consciously classified.

### 10. Verification and closeout sequence

- Run the required gates in full:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- If any gate fails, fix the underlying issue and re-run that gate until it is clean.
- Only after every required gate passes:
  - update the acceptance boxes in this task file
  - set `<passes>true</passes>` in this task file
  - run `/bin/bash .ralph/task_switch.sh`
  - commit all modified files, including `.ralph` bookkeeping, with `task finished [task name]: ...`
  - `git push`
- Stop immediately after the required push.

### 11. Required skeptical-review targets for the `TO BE VERIFIED` pass

- Challenge whether the current plan over-focuses on the noisiest files and misses smaller but more important runtime literals elsewhere.
- Challenge whether any value proposed for a constant should instead stay inline because its meaning is already obvious.
- Challenge whether any proposed constant belongs in an existing config/defaults module rather than inside a worker/test file.
- Challenge whether the planned HA scenario-support consolidation is too aggressive and would reduce scenario readability.
- Challenge whether the docs-update condition is too narrow if canonical fixture values move in ways contributors/operators need to know about.
- The `TO BE VERIFIED` pass must alter at least one concrete part of this plan before replacing the marker below.

NOW EXECUTE
