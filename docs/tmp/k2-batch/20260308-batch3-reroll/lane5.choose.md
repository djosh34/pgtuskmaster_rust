Target docs path: docs/src/how-to/perform-switchover.md
Diataxis type: how-to
Why this is the next doc:
- Bridges gap between "first-ha-cluster" tutorial and "check-cluster-health" how-to
- Builds on existing switchover mentions in architecture.md and pgtuskmasterctl-cli.md
- Addresses real-world operational need: planned leadership transfer
- Reuses established CLI patterns from existing reference
- Natural progression of operational competence after cluster bootstrap

Exact additional information needed:
- file: src/api/controller.rs
  why: Shows switchover API endpoints, request/response handling, state updates in DCS
- file: src/ha/decide.rs
  why: Reveals how switchover requests trigger phase transitions (WaitingSwitchoverSuccessor, StepDown)
- file: src/dcs/state.rs
  why: Documents how switchover state is stored, leader lease release mechanics, trust evaluation impact
- file: src/cli/client.rs
  why: Shows exact HTTP client calls for switchover clear/request, error handling, token usage
- extra info: What is the minimum required dcs.lease_ttl_ms and ha.loop_interval_ms for reliable switchover completion? What happens if target replica is lagging?

Optional runtime evidence to generate:
- command: cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 ha switchover request --requested-by node-b
  why: Capture actual API request/response payload and accepted=true/false logic
- command: docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml logs node-a --tail=50 | grep -E "(switchover|phase|decision)"
  why: Show real-time HA phase transitions during switchover execution
