Target docs path: docs/src/reference/ha-decisions.md
Diataxis type: reference
Why this is the next doc:
- HTTP API reference lists HA decision variants but provides no explanation of their meaning or parameters
- Architecture explanation describes the decision flow but not the detailed decision types
- Operators debugging clusters need to understand exactly what each decision variant means and what triggers it
- HA decision engine is the core safety and orchestration component; its decisions should be comprehensively documented as technical reference

Exact additional information needed:
- file: src/ha/decision.rs
  why: Defines the HaDecision enum and all decision variants with their data fields
- file: src/ha/state.rs
  why: Defines the HaPhase enum and phase variants that decisions transition between
- file: src/ha/decide.rs
  why: Contains the decision logic that maps world state to decisions; shows when each decision is emitted
- file: docs/src/reference/http-api.md
  why: Shows current HaDecisionResponse and HaPhaseResponse schemas that need detailed reference documentation
- file: docs/src/explanation/architecture.md
  why: Provides the flow diagram and context for how decisions fit into the overall HA phase machine

Optional runtime evidence to generate:
- command: cargo test --test ha_multi_node_failover -- --nocapture
  why: Captures actual decision sequences during failover scenarios to verify decision emission patterns
