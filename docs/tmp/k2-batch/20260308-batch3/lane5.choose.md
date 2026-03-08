**Target docs path:** `docs/src/how-to/perform-switchover.md`

**Diataxis type:** how-to

**Why this is the next doc:**
- Fills the largest gap: only one how-to guide exists vs. multiple reference/explanation docs
- Addresses a core HA operation that operators perform regularly
- Builds directly on existing docs (CLI commands are documented, but no workflow exists)
- Connects architecture concepts (trust model, HA phases) to actionable steps
- The architecture doc mentions switchover as a primary HA transition but provides no operational guidance

**Exact additional information needed:**
- `src/api/controller.rs` - why: to map switchover API endpoints (`/ha/switchover`, `/switchover`) and response payloads
- `src/ha/decide.rs` - why: to understand how `WaitingSwitchoverSuccessor`, `StepDown`, and leadership transfer decisions work
- `tests/ha_multi_node_failover.rs` - why: to extract realistic switchover scenarios, timing expectations, and verification steps
- `docker/configs/cluster/node-a/runtime.toml` - why: to show prerequisites (e.g., auth tokens, member IDs, DCS scope) needed before switchover
- Extra info: What are the failure modes if switchover is requested when cluster is in `FailSafe` or `NotTrusted` state?

**Optional runtime evidence to generate:**
- Command: `cargo run --bin pgtuskmasterctl -- --admin-token dev-token --base-url http://127.0.0.1:18081 ha switchover request --requested-by node-b`
  - Why: Capture actual request/response to document success criteria and error messages
- Command: `cargo run --bin pgtuskmasterctl -- --admin-token dev-token --base-url http://127.0.0.1:18081 ha state`
  - Why: Show pre/post switchover state verification steps from a real cluster
