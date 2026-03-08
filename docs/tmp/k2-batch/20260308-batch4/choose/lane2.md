Target docs path: docs/src/tutorial/debug-api-usage.md

Diataxis type: tutorial

Why this is the next doc:
- The debug API is the primary observability surface for understanding runtime decisions, yet no tutorial exists that teaches users how to use it systematically
- All existing debug API documentation is reference-style (reference/debug-api.md) or embedded in how-to guides; users lack a guided learning path to build mental models of the snapshot structure and incremental polling patterns
- Mastering debug/verbose is a prerequisite for effective troubleshooting, interpreting HA decisions, and validating cluster behavior, making it a foundational skill that unlocks the other documentation
- The tutorial format allows step-by-step discovery of trust evaluation, decision flow, and timeline reconstruction without overwhelming users with exhaustive detail upfront

Exact additional information needed:
- file: src/debug_api/worker.rs
  why: To understand snapshot construction, channel versioning, retention limits, and how changes/timeline events are generated and trimmed
- file: src/api/controller.rs  
  why: To map API endpoint paths to debug worker state and understand how since= parameter filters changes/timeline versus serving full snapshots
- file: tests/ha/support/observer.rs
  why: To see how the test harness polls debug/verbose incrementally and interprets decision changes during failover scenarios
- file: docker/configs/cluster/node-a/runtime.toml
  why: To verify debug.enabled toggle and identify default listen_addr for curl commands in tutorial steps
- extra info: Are there explicit rate limits or client-side thresholds on debug/verbose polling that users should respect in production?

Optional runtime evidence to generate:
- command: cargo run --bin pgtuskmaster -- --config docker/configs/cluster/node-a/runtime.toml & sleep 5 && curl -s http://127.0.0.1:18081/debug/verbose | jq '.meta, .dcs, .ha, .changes, .timeline'
  why: Provides real snapshot output showing schema shape, sequence counter, trust state, HA decision structure, and realistic change/timeline entries for verbatim inclusion
- command: curl -s "http://127.0.0.1:18081/debug/verbose?since=0" | jq '.changes, .timeline'
  why: Demonstrates incremental polling behavior and shows how since= parameter filters history correctly
