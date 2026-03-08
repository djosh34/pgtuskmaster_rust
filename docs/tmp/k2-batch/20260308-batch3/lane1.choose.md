Target docs path: docs/src/how-to/bootstrap-cluster.md
Diataxis type: how-to
Why this is the next doc:
- Bootstrapping is a critical operational task not covered by existing docs
- The tutorial covers it in a learning context (Docker Compose), but operators need goal-oriented instructions for production
- Architecture doc mentions bootstrap phases but doesn't provide operational guidance
- Based on code, bootstrap involves DCS init, first node startup, and member join sequence that operators must understand
- Would complete core operational coverage alongside health check and switchover

Exact additional information needed:
- File: src/ha/decide.rs
  why: To understand Bootstrapping phase handler, Init phase transitions, and bootstrap-specific HA decision logic
- File: src/dcs/store.rs
  why: To understand DCS write operations for member registration, leader election, and init lock during bootstrap
- File: src/dcs/state.rs
  why: To understand trust evaluation when DCS has no prior state (Empty vs FailSafe)
- File: src/config/schema.rs
  why: To identify required bootstrap configuration fields (cluster.name, cluster.member_id, dcs.scope, dcs.init)
- File: docker/configs/cluster/node-a/runtime.toml
  why: As concrete working example of bootstrap configuration with dcs.init payload
- File: src/test_harness/ha_e2e/startup.rs
  why: To understand the programmatic bootstrap sequence used in tests and verification patterns
- File: tests/ha/support/observer.rs
  why: To understand how to verify bootstrap success through HA state observation
- Extra info: What are the exact steps to bootstrap a cluster from zero state, including DCS pre-initialization, first node startup, and subsequent node joining patterns?

Optional runtime evidence to generate:
- Command: docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up etcd -d && sleep 2 && docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up node-a -d --no-deps && docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml logs node-a | grep -i bootstrap
  why: To capture bootstrap phase logs and timing from first node initialization
- Command: cargo test --test ha_multi_node_failover -- --nocapture | grep -E "(bootstrap|init|Bootstrapping|Init)" | head -20
  why: To observe bootstrap behavior in integration tests and see the expected sequence
