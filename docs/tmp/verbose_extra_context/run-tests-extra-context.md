# Extra context for docs/src/how-to/run-tests.md

This repository has no published contributor testing guide yet, but the local build tooling and harness code provide enough factual material to explain how tests are run.

Canonical entrypoints from `Makefile`:

- `make check` runs `cargo check --all-targets`.
- `make test` runs `cargo nextest run --workspace --all-targets --profile default --no-fail-fast --no-tests fail`.
- `make test-long` runs `cargo nextest run --workspace --all-targets --profile ultra-long --no-fail-fast --no-tests fail`, then Docker Compose validation and two Docker smoke scripts.
- `make lint` includes `docs-lint`, the silent-error guard, and multiple `cargo clippy` passes.

Important prerequisites directly visible in the repo:

- `cargo-nextest` is required for `make test` and `make test-long`.
- `timeout` or `gtimeout` is required for the Makefile gates that use timeouts.
- Docker and the Docker Compose plugin are required for `make test-long` and the docker smoke/config targets.
- The test harness requires real binaries for etcd and PostgreSQL 16. Source paths such as `src/test_harness/binaries.rs`, `src/test_harness/provenance.rs`, and `src/test_harness/ha_e2e/startup.rs` show that real-binary attestations are enforced and that helper install scripts `./tools/install-etcd.sh` and `./tools/install-postgres16.sh` are the expected way to satisfy them.
- The dev image in `docker/Dockerfile.dev` installs `protobuf-compiler`, `pkg-config`, Node, npm, ripgrep, Python, and Rust tooling, which is a good indicator of the local toolchain expected by developers.

What kinds of tests exist, based on files requested by K2:

- `tests/ha_multi_node_failover.rs` is a top-level entry for HA end-to-end scenarios.
- `tests/ha/support/multi_node.rs` contains large multi-node scenarios, convergence waits, stress workloads, and API-driven switchovers.
- `tests/ha/support/observer.rs` contains split-brain detection and HA observation helpers.
- `tests/bdd_api_http.rs` contains behavior-driven HTTP API tests that use a sample runtime config and raw HTTP framing checks.
- `src/worker_contract_tests.rs` contains contract-style tests that assert required state types, debug API responsiveness, and worker coordination behavior.

External dependencies and runtime environment details that are supportable from source:

- Real HA scenarios depend on etcd and PostgreSQL 16 binaries, not mocks only.
- Some scenarios also rely on Docker Compose stacks under `docker/compose/`.
- The harness allocates ports dynamically and creates isolated namespaces, so parallel test execution is expected.
- The Makefile forces a shared target dir under `/tmp/pgtuskmaster_rust-target` and disables incremental builds by default for deterministic gates.

Environment variables and flags visible in source/docs:

- `PGTUSKMASTER_READ_TOKEN` and `PGTUSKMASTER_ADMIN_TOKEN` are CLI env fallbacks, but these are API auth inputs rather than test-runner controls.
- The repository evidence does not show a dedicated test-only environment variable contract for choosing subsets of tests; the primary documented knobs are the Makefile targets and ordinary cargo/nextest filtering.
- A draft should avoid inventing hidden test env vars that are not clearly present in source.

Execution time and resource expectations that can be stated factually:

- `make test-long` is intentionally expensive: the Makefile describes it as running ultra-long HA scenarios plus Docker Compose validation and smoke coverage.
- Timeouts in the Makefile show the intended rough scale:
  - cargo check gate timeout: 300 seconds
  - docs lint timeout: 120 seconds
  - clippy timeout: 1200 seconds
  - docker compose config timeout: 120 seconds
  - docker smoke single timeout: 600 seconds
  - docker smoke cluster timeout: 900 seconds
- HA scenario constants in `tests/ha/support/multi_node.rs` include timeouts up to 180 seconds for loaded failover and 300 seconds for a scenario budget, which further confirms that long-running integration tests are normal.

Guidance boundaries:

- The doc should describe the real prerequisites honestly.
- It should not claim tests are lightweight unit-only checks.
- It should not claim any test is optional when the codebase clearly treats real-binary validation as mandatory.
