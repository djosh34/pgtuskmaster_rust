Target docs path: docs/src/how-to/perform-switchover.md
Diataxis type: how-to
Why this is the next doc:
- Only one how-to guide exists ("Check Cluster Health"), leaving a critical gap in operational procedures
- Switchover is a common production operation that builds naturally from health checking
- CLI reference already documents switchover commands but lacks procedural guidance
- Architecture doc mentions switchover concepts but not practical execution
- Users need goal-oriented steps for safe leader transitions

Exact additional information needed:
- file: src/cli/args.rs
  why: Exact CLI flag definitions for switchover request/clear commands
- file: src/api/controller.rs
  why: HTTP endpoint paths, request/response bodies, and success/failure modes
- file: src/ha/decide.rs
  why: Switchover decision logic and phase transitions (WaitingSwitchoverSuccessor, StepDown)
- file: src/dcs/state.rs
  why: DCS switchover state structure and trust requirements
- file: docker/configs/cluster/node-a/runtime.toml
  why: Sample lease_ttl_ms and loop_interval_ms values that affect switchover timing
- file: tests/policy_e2e_api_only.rs
  why: Integration test patterns showing switchover workflow and expected outcomes
- extra info: Are there any guardrails preventing switchover when DCS trust is not FullQuorum?

Optional runtime evidence to generate:
- command: cargo run --bin pgtuskmasterctl -- --help
  why: Verify exact switchover subcommand syntax and options
- command: pgtuskmasterctl ha switchover request --requested-by node-a --base-url http://127.0.0.1:18081
  why: Capture actual switchover request flow and response format (success case)
