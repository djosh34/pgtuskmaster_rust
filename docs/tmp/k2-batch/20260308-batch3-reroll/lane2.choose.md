Target docs path: docs/src/how-to/perform-switchover.md
Diataxis type: how-to
Why this is the next doc:
- The project has only one how-to guide; operational tasks need more coverage
- Switchover is a core HA operation referenced in CLI docs but lacks procedural guidance
- It addresses a real-world goal (planned leader migration) distinct from health checks
- It naturally extends the existing tutorial (which covers startup) into maintenance operations
- The architecture doc explains the switchover mechanism but doesn't guide users through performing one

Exact additional information needed:
- file: src/api/controller.rs
  why: To see the exact switchover API endpoint implementations, request/response schemas, and error handling paths
- file: src/ha/decide.rs
  why: To understand the HA phase machine transitions during switchover (WaitingSwitchoverSuccessor → Replica) and decision logic
- file: src/ha/decision.rs
  why: To document the exact switchover-related decision variants (StepDown, BecomePrimary with promote=false, WaitingSwitchoverSuccessor) and their parameters
- file: tests/ha/support/observer.rs
  why: To extract the expected invariants and success criteria for switchover (e.g., no split-brain, single primary observed)
- file: docker/configs/cluster/node-a/runtime.toml
  why: To reference concrete lease TTL values and HA timing parameters that affect switchover duration and behavior
- file: src/dcs/state.rs
  why: To understand how switchover state is stored in DCS and how trust evaluation interacts with switchover requests

Optional runtime evidence to generate:
- command: docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d --build
  why: To create a running three-node cluster for capturing actual switchover command output and phase transitions
- command: cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 --admin-token "admin-secret" ha switchover request --requested-by node-b
  why: To capture the exact success response format and subsequent state changes
- command: timeout 30s bash -c 'while true; do cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 --output text ha state; sleep 2; done'
  why: To observe and document the timeline of HA phase progression during an active switchover operation
