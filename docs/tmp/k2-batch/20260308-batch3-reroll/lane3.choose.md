Target docs path: docs/src/how-to/perform-switchover.md
Diataxis type: how-to
Why this is the next doc:
- Switchover is a critical manual HA operation not yet documented despite CLI support
- Fills operational gap between "check health" and architecture explanation
- Addresses real-world need for planned leader transitions versus automatic failover
- Builds on existing CLI reference without duplicating content
- Aligns with tested behavior in HA observer tests

Exact additional information needed:
- file: src/api/controller.rs
  why: Exact switchover endpoint handlers (`/ha/switchover` POST/DELETE) and validation logic
- file: src/cli/args.rs
  why: CLI argument parsing for `ha switchover request --requested-by` and `ha switchover clear`
- file: src/ha/decide.rs
  why: HA decision logic when `switchover_requested_by` is present in DCS state
- file: src/dcs/state.rs
  why: How switchover state is stored, retrieved, and merged in DCS cache
- file: tests/ha_multi_node_failover.rs
  why: Test scenarios demonstrating switchover preconditions and success/failure modes
- file: docker/configs/cluster/node-a/runtime.toml
  why: Example config showing member_id values used as `--requested-by` arguments
- extra info: What are the explicit preconditions for switchover acceptance? (e.g., current leader health, replica readiness, DCS trust state requirements)

Optional runtime evidence to generate:
- command: `cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 --output json ha state`
  why: Capture baseline state showing current leader and confirming no active switchover
- command: `cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 ha switchover request --requested-by node-b`
  why: Demonstrate actual switchover request and capture success/failure response
- command: `cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 --output text ha state`
  why: Show `switchover_requested_by=node-b` in text output during pending switchover
- command: `cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 ha switchover clear`
  why: Demonstrate switchover cancellation and capture confirmation response
