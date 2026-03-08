Target docs path: docs/src/how-to/perform-switchover.md
Diataxis type: how-to
Why this is the next doc:
- Real-world operational need: Switchover is a core HA operation distinct from health checks
- Closes capability gap: No documentation exists for the `ha switchover` commands despite CLI reference coverage
- Builds on existing tutorials: Users who completed "First HA Cluster" will have a running 3-node cluster ready for switchover practice
- Prerequisite for explanation: Concrete switchover experience will make "Split-brain Prevention" explanation more meaningful

Exact additional information needed:
- file: tests/policy_e2e_api_only.rs
  why: Contains end-to-end test patterns showing switchover request/clear operations and verification steps
- file: tests/ha_multi_node_failover.rs
  why: Demonstrates multi-node HA transitions including leader changes and observer validations
- file: docker/configs/cluster/node-a/runtime.toml
  why: Shows the `api.security.auth` configuration needed for `--admin-token` usage in the commands
- extra info: What specific verification steps (pgtuskmasterctl ha state checks, PostgreSQL connection tests, timeline verification) should be included after switchover completion?
- extra info: Are there any timeout behaviors or lease considerations during switchover that operators must understand?

Optional runtime evidence to generate:
- command: cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 --output json ha switchover request --requested-by node-b
  why: Capture successful switchover request response shape and accepted=true behavior
- command: cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18082 --output text ha state
  why: Show expected state progression on node-b from replica to primary during switchover execution
