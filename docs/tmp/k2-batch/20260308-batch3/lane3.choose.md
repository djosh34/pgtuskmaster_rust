Target docs path: docs/src/how-to/handle-primary-failure.md
Diataxis type: how-to
Why this is the next doc:
- The HA system has explicit failure modes (Fencing, FailSafe, Rewinding) and recovery paths that operators must understand during incidents
- The current how-tos only cover planned operations (switchover, health checks) but not unplanned failures
- Test files demonstrate multiple failure scenarios with observer patterns that can be translated into operator procedures
- Failure handling is critical for adoption of any HA system; operators need clear incident response guidance
- The architecture doc mentions safety mechanisms but does not provide actionable steps for operators

Exact additional information needed:
- file: tests/ha_multi_node_failover.rs
  why: Contains scenarios of primary failure and replica promotion that show expected HA phase transitions and DCS trust changes
- file: tests/ha_partition_isolation.rs
  why: Demonstrates network partition handling and fencing behavior needed for incident response procedures
- file: tests/ha/support/observer.rs
  why: Shows split-brain detection logic that operators must understand when verifying cluster state post-failure
- file: src/ha/decide.rs
  why: Contains decision logic for failure states (FailSafe, Fencing, RecoverReplica, StepDown) needed to explain recovery phases
- file: src/dcs/state.rs
  why: Defines trust evaluation rules that determine when normal HA operation resumes after failures
- file: docker/configs/cluster/node-a/runtime.toml
  why: Shows ha.lease_ttl_ms and other timing parameters that affect failure detection speed and recovery time
- extra info: What are the exact API responses (JSON output) from `/ha/state` during each failure phase (Fencing, FailSafe, WaitingSwitchoverSuccessor)?
- extra info: What log messages does the HA worker emit at each log level (info, warn, error) during a primary failure and subsequent failover?

Optional runtime evidence to generate:
- command: cargo test ha_multi_node_failover -- --nocapture
  why: Would capture actual HA phase transitions, DCS trust states, and decision logs during simulated primary failure
- command: docker compose logs node-a -f --tail=50 (during docker-compose cluster test)
  why: Would provide real log output showing failure detection and recovery messages for operators to recognize <|tool_calls_section_begin|> <|tool_call_begin|>
