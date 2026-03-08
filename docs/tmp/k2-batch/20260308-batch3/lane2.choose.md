Target docs path: `docs/src/how-to/perform-switchover.md`
Diataxis type: how-to
Why this is the next doc:
- Builds on existing "Check Cluster Health" how-to with a concrete operational task
- Addresses production need for controlled leader transitions
- CLI reference already documents commands but lacks procedural safety context
- Architecture doc explains switchover mechanics at component level but not operator workflow
- Gap exists between cluster setup (tutorial) and routine operations

Exact additional information needed:
- file: src/api/controller.rs
  why: Exact switchover API endpoints, request/response payloads, and success/failure codes
- file: src/cli/client.rs
  why: CLI client implementation details for `ha switchover request/clear` and error handling
- file: tests/ha/support/observer.rs
  why: Verification patterns for detecting split-brain or incomplete switchover during tests
- file: src/ha/decide.rs
  why: Decision logic for `WaitingSwitchoverSuccessor` phase and transition conditions
- extra info: What DCS trust level and replica lag thresholds make switchover safe to attempt

Optional runtime evidence to generate:
- command: `docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d --build`
  why: Create three-node test cluster for demonstrating switchover procedure
- command: `cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 --output text ha switchover request --requested-by node-b`
  why: Capture exact CLI syntax, auth requirements, and text-mode response format
- command: `while true; do curl -sf http://127.0.0.1:18081/ha/state | jq -r '.ha_decision'; sleep 1; done`
  why: Observe real-time decision progression during switchover execution
