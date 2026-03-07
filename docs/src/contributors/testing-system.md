# Testing System Deep Dive

This repo uses several test layers because no single layer can prove HA behavior on its own. Pure logic tests keep refactors cheap, boundary tests prove that workers publish and dispatch the right things, BDD tests protect client-visible contracts, and real-binary scenarios prove that the system survives real Postgres, real etcd, and hostile timing.

If you are trying to find the right place to add coverage, start with this map:

- change in pure HA logic or a small data invariant: start in `src/ha/decide.rs` or the owning module tests
- change in worker behavior or side-effect mapping: start in `src/ha/worker.rs` or `src/worker_contract_tests.rs`
- change in an external HTTP contract: start in `tests/bdd_api_http.rs`
- change in startup, failover, fencing, partitions, or replication timing: start in the harness-backed HA scenarios under `tests/ha/`

The design contract is simple: fast tests should prove local correctness, and slower tests should prove that those decisions still hold once real binaries and asynchronous failures are involved.

## Where the main test layers live

The fastest way to navigate the test tree is to treat it as four different proof styles rather than one large pile of test files.

### Unit tests: prove logic and data-shape invariants

Unit tests are the place to prove exact input-to-output behavior with minimal runtime noise. They should stay cheap enough that you can run them repeatedly while changing one function.

Good starting points:

- `src/ha/decide.rs`: HA phase transitions and decision selection
- `src/ha/lower.rs`: decision-to-effect-plan lowering invariants
- `src/state/watch_state.rs`: latest-snapshot semantics for `StatePublisher` and `StateSubscriber`
- module-local `#[cfg(test)]` blocks in state/config/process code for parsing and type invariants

When you change pure logic, keep the test at this layer unless the behavior depends on actual subprocesses, sockets, or multi-node timing.

### Worker contract tests: prove owned state plus side effects

Worker contract tests answer a different question: given a concrete world snapshot, does the worker publish the right state and request the right side effects?

In this repo, the key boundary is not just "did the function return the right enum?" but also:

- which DCS writes or deletes were attempted
- which process jobs were enqueued
- whether worker status was faulted or remained healthy

Best starting points:

- `src/ha/worker.rs`: step-level dispatch behavior
- `src/worker_contract_tests.rs`: reusable worker-boundary contract coverage
- `src/api/controller.rs`: mapping internal state into API responses

If a change affects `step_once(...)`, dispatch ordering, or published status, keep one test here even if you also add e2e coverage later.

### BDD tests: prove external contracts without a full HA cluster

BDD-style tests under `tests/` treat the system as an external client would: issue a request, observe the response, and confirm the contract that must remain stable.

Key entrypoints:

- `tests/bdd_api_http.rs`: request parsing, auth handling, debug routes, and DCS intent writes
- `tests/bdd_state_watch.rs`: client-visible watch semantics
- `tests/cli_binary.rs`: CLI process-level smoke behavior
- `tests/policy_e2e_api_only.rs`: policy-oriented checks that do not require the full HA harness

Use this layer when the important question is "what does the caller observe?" rather than "which internal action bucket was selected?"

### Real-binary HA scenarios: prove safety under real timing

The most expensive tests use the harness in `src/test_harness/` and the scenario modules under `tests/ha/support/`. These tests are where the repo proves that the control loop, DCS integration, process control, and client-observable state still compose correctly when real binaries are involved.

The narrow entrypoint files are intentionally small:

- `tests/ha_multi_node_failover.rs`
- `tests/ha_partition_recovery.rs`

Those files delegate to the real scenario code in:

- `tests/ha/support/multi_node.rs`
- `tests/ha/support/partition.rs`
- `tests/ha/support/observer.rs`

That split is a navigation hint for contributors. If you want to understand the scenario logic, open the support modules rather than stopping at the top-level test files.

## The harness contract for HA evidence

The most important contributor rule in HA tests is that a final converged state is not enough evidence. The scenarios deliberately use `HaInvariantObserver` from `tests/ha/support/observer.rs` to sample API and SQL observations over a time window and fail closed if there are not enough successful samples to prove the claim being made.

What that means in practice:

- no-dual-primary claims are proven over a sampled window, not inferred from one final poll
- observation gaps are recorded explicitly rather than ignored
- transport failures and blind spots count against confidence instead of being silently tolerated

If you add a new HA scenario, preserve that proof style. Do not replace a continuous invariant window with a single "eventually this node became primary" assertion.

## Shared test fixtures: where runtime config should come from

When a test needs a valid `RuntimeConfig`, prefer `src/test_harness/runtime_config.rs`. `RuntimeConfigBuilder::new()` gives you a valid baseline and lets you override only the fields that matter for the scenario.

That builder is the right tool for:

- worker contract tests
- harness-backed integration tests
- examples or helpers that need a syntactically valid managed config

It is not the right tool when the test is specifically about parser input shape or error messages. Those tests should keep the TOML fixture inline in the parser/config tests so the user-visible input remains obvious.

## Real binaries are mandatory, not optional

Harness-backed tests intentionally fail closed when required binaries are missing or untrusted. The public entrypoints in `src/test_harness/binaries.rs` delegate to provenance enforcement in `src/test_harness/provenance.rs`, which verifies:

- the repo-tracked policy in `tools/real-binaries-policy.json`
- the local attestation manifest in `.tools/real-binaries-attestation.json`
- executable/file properties, path constraints, hashes, sizes, and expected versions

If a real-binary test fails because a prerequisite is missing, the fix is to install or refresh the binaries:

```bash
./tools/install-etcd.sh
./tools/install-postgres16.sh
```

If you suspect the wrong binary is being executed, collect evidence with:

```bash
./tools/trace-real-binary-execve.sh
```

## What `make test` and `make test-long` actually prove

The split between `make test` and `make test-long` is defined by the nextest profiles in `.config/nextest.toml` and by extra Docker validation in the `Makefile`.

- `make test` runs the default nextest profile across the workspace.
- The default profile explicitly excludes the most expensive HA scenarios such as `e2e_multi_node_unassisted_failover_sql_consistency` and `e2e_partition_mixed_faults_heal_converges`.
- `make test-long` runs the `ultra-long` nextest profile, which is serialized (`test-threads = 1`) and contains those focused HA scenarios.
- After the ultra-long profile passes, `make test-long` also runs Docker Compose config validation plus the single-node and cluster smoke scripts.

The important nuance is that `make test-long` is not "all real-binary tests" in the abstract. It is the home for the most expensive HA scenarios and the container-smoke verification owned by the repo. Keep that distinction when moving tests between gates.

## How to choose the right new test

Use this decision rule when adding coverage:

- pure transformation or state-machine rule: add a unit test in the owning module
- worker chooses different DCS/process effects: add a worker contract test
- HTTP or CLI contract changes: add a BDD test under `tests/`
- timing-sensitive startup/failover/partition behavior: add or extend a harness-backed HA scenario

When a change is safety-relevant, add one narrow fast test and one realistic system test if both layers cover different failure modes.

## Safe ways to change the test system

The test stack has a few design contracts that are easy to break accidentally:

- Do not silently skip real-binary tests. Missing prerequisites are an environment failure, not a green result.
- Do not bypass the observer-based evidence model in HA scenarios. If the claim is about split-brain or fail-safe behavior, the scenario must collect enough samples to prove it.
- Do not hide topology-specific details inside a generic fixture builder. Scenario wiring belongs in the harness and scenario code.
- Do not move a slow scenario into `make test-long` just because it is inconvenient. Move it only when the runtime cost is consistently too high for the default developer loop.

## Failure triage: where to look first

When a test fails, classify the failure before changing code.

- unit failure: inspect the exact input snapshot and expected output in the module test
- worker contract failure: inspect the published state and recorded DCS/process side effects
- BDD failure: inspect the request/response pair and any store writes the test captures
- harness-backed failure: inspect the namespace logs, observer summary, and the scenario support module that drove the fault

That sequence keeps you from debugging a low-level harness issue as though it were an HA decision bug, or vice versa.

## Adjacent subsystem connections

Testing is only useful if you understand the code paths being exercised:

- Read [Harness Internals](./harness-internals.md) for `start_cluster(...)`, namespaces, port reservations, proxies, and cleanup behavior.
- Read [HA Decision and Effect-Plan Pipeline](./ha-pipeline.md) before changing the assertions in HA scenario support modules.
- Read [API and Debug Contracts](./api-debug-contracts.md) when you are deciding whether a behavior belongs in BDD coverage or in internal-only contract tests.

## Evidence pointers

If you want to verify the claims in this chapter directly, start here:

- `src/ha/decide.rs`
- `src/ha/worker.rs`
- `src/worker_contract_tests.rs`
- `src/test_harness/runtime_config.rs`
- `.config/nextest.toml`
- `Makefile`
- `tests/bdd_api_http.rs`
- `tests/ha/support/observer.rs`
- `tests/ha/support/multi_node.rs`
- `tests/ha/support/partition.rs`
