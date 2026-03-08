**Target docs path:** `docs/src/explanation/ha-decision-engine.md`  
**Diataxis type:** explanation  
**Why this is the next doc:**  
- The HA decision engine is the core orchestration logic, but current docs only surface it via brief architecture mentions and a decision reference table  
- Operators need deeper understanding of *why* decisions are made, *how* the phase machine constrains unsafe transitions, and *what* tradeoffs are embodied in the trust-gated design  
- Existing explanation docs cover architecture and failure modes at high level; none unpack the decision engine's philosophy, state evolution, or safety invariants in narrative form  

**Exact additional information needed:**  
- file: `src/ha/decide.rs` - why it is phase-driven, how trust gates the top of `decide_phase`, and how each phase handler chooses the next state  
- file: `src/ha/decision.rs` - the complete taxonomy of decision variants and their payload semantics  
- file: `src/ha/lower.rs` - how decisions become effect plans and why the lowerer folds multiple outcomes into a single safe action set  
- file: `src/dcs/state.rs` - trust evaluation flowchart implementation and freshness window calculations using `ha.lease_ttl_ms`  
- file: `src/ha/process_dispatch.rs` - how the process worker translates effects into concrete PostgreSQL lifecycle jobs  
- extra info: Which HA decisions are idempotent vs. destructive? When does the engine prefer `FailSafe` over attempting recovery?  

**Optional runtime evidence to generate:**  
- command: `cargo run --bin pgtuskmaster -- --config docker/configs/cluster/node-a/runtime.toml` (with debug enabled) then `curl http://127.0.0.1:8080/debug/verbose` during a simulated primary stop  
- why: Capture real `timeline` and `changes` output showing trust degradation, decision evolution, and phase transitions to illustrate the narrative with concrete event sequences
