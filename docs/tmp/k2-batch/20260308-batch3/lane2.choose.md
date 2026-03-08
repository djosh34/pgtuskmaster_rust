Target docs path: docs/src/tutorial/observing-failover.md
Diataxis type: tutorial
Why this is the next doc:
- The existing tutorial covers only basic cluster startup and leader identification
- No tutorial covers HA behavior under failure, which is the core value proposition of pgtuskmaster
- Split-brain prevention and failover recovery are architecturally central (per architecture.md) but not demonstrated in a learning context
- Directly builds on first-ha-cluster.md by using the same Docker environment
- Provides concrete, observable steps showing how the system responds to node failure
- Bridges the gap between initial setup (tutorial) and operational procedures (how-to guides)

Exact additional information needed:
- file: tests/ha_multi_node_failover.rs
  why: Shows expected failover scenarios and success criteria
- file: tests/ha_partition_isolation.rs
  why: Demonstrates split-brain prevention behavior
- file: docker/compose/docker-compose.cluster.yml
  why: Provides exact service definitions and ports for tutorial steps
- file: src/debug_api/mod.rs and src/debug_api/snapshot.rs
  why: Needed to show learners how to observe internal state during failover
- file: src/ha/decision.rs
  why: Lists all possible HA decisions (e.g., FenceNode, RecoverReplica) that should be observed
- file: src/test_harness/ha_e2e/startup.rs and ops.rs
  why: Shows programmatic setup patterns for controlled failure testing
- file: src/dcs/state.rs
  why: Trust evaluation logic (FailSafe, NotTrusted) that learners must understand during failure
- extra info: What specific commands or signals are used in tests to simulate node failure (kill, network partition, etc.)?

Optional runtime evidence to generate:
- command: docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d
  why: Verify exact container names and network setup for tutorial steps
- command: cargo test --test ha_multi_node_failover -- --nocapture
  why: Capture actual log output showing failover sequence and timing
- command: curl http://127.0.0.1:18081/debug/snapshot (if debug API is enabled)
  why: Show internal state snapshot format for learner observation steps
