**Target docs path**: `docs/src/how-to/monitor-via-metrics.md`

**Diataxis type**: how-to

**Why this is the next doc**:
- Fills a critical production gap: no doc covers systematic metrics collection
- Builds directly on existing `check-cluster-health.md` and `debug-cluster-issues.md` how-tos
- Leverages proven observer patterns from `tests/ha/support/observer.rs`
- The runtime exposes rich metrics via `/ha/state`, `/debug/verbose`, and `/debug/snapshot`, but operators lack a guide to collect and alert on them
- Production deployments need monitoring for DCS trust degradation, split-brain detection, and HA decision trends

**Exact additional information needed**:
- file: `src/api/controller.rs` - why: to list all metric-exposing endpoints and their exact response structures
- file: `src/debug_api/view.rs` - why: to document `verbose` and `snapshot` payload schemas for metric extraction
- file: `src/debug_api/worker.rs` - why: to understand snapshot versioning and `since=` parameter behavior for polling
- file: `tests/ha/support/observer.rs` - why: to replicate tested split-brain detection logic and sampling patterns
- file: `src/ha/decision.rs` - why: to enumerate all decision variants that should trigger alerts
- extra info: Are there any built-in metric export formats (Prometheus, StatsD) or is JSON-only via API?

**Optional runtime evidence to generate**:
- command: `cargo run --bin pgtuskmasterctl -- --output json ha state` - why: to capture CLI output shape for monitoring agent integration
- command: `curl -s "http://127.0.0.1:8080/debug/verbose?since=0" | jq .timeline,.changes` - why: to show how to poll for state transitions and anomalies
