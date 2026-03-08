Target docs path: docs/src/how-to/handle-network-partition.md
Diataxis type: how-to
Why this is the next doc:
- Network partitions are a critical failure mode for any distributed HA system but lack a dedicated operator guide
- The architecture emphasizes trust gating and fail-safe behavior precisely for partition scenarios, yet operators have no procedural documentation
- Existing failure-modes.md explains theory; a how-to would bridge to practice
- Test suite includes explicit partition isolation tests, proving runtime behavior is defined and testable
- Complements existing handle-primary-failure.md by covering coordination-layer failures rather than single-node failures

Exact additional information needed:
- file: src/dcs/state.rs
  why: Contains trust evaluation logic that determines FullQuorum vs FailSafe vs NotTrusted during connectivity loss
- file: src/ha/decide.rs
  why: Shows HA phase transitions and decision emission when trust degrades or recovers
- file: tests/ha_partition_isolation.rs
  why: Defines test scenarios for partition behavior, showing expected system responses
- file: tests/ha/support/partition.rs
  why: Reveals partition simulation mechanics and fault injection patterns
- file: tests/ha/support/observer.rs
  why: Demonstrates split-brain detection logic that operators must understand
- file: docker/configs/cluster/node-a/runtime.toml
  why: Provides concrete lease_ttl_ms and loop_interval_ms values that bound freshness detection
- file: src/dcs/worker.rs
  why: Shows how member records are published and when they become stale
- file: src/ha/decision.rs
  why: Lists decision variants like EnterFailSafe and FenceNode that appear during partitions

Optional runtime evidence to generate:
- command: docker compose -f docker/compose/docker-compose.cluster.yml up --force-recreate; sleep 5; docker exec -t pgtuskmaster-node-b iptables -A OUTPUT -d $(getent hosts pgtuskmaster-etcd | cut -d' ' -f1) -j DROP; sleep 10; docker exec -t pgtuskmaster-node-b iptables -F
  why: Would produce actual trust degradation timeline showing FailSafe entry, lease timeout, and recovery to FullQuorum
