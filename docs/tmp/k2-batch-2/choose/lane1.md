Target docs path: docs/src/reference/debug-api.md
Diataxis type: reference
Why this is the next doc:
- The debug API is a real, implemented feature surface (`/debug/snapshot`, `/debug/verbose`, `/debug/ui`) that appears in architecture diagrams and operational guides but lacks dedicated reference documentation
- Reference documentation requires authority and dryness; a debug API reference should catalog endpoints, request/response schemas, and configuration requirements without tutorial or explanatory prose
- The existing `docs/src/how-to/debug-cluster-issues.md` demonstrates operator usage but does not formally describe the debug API contract, which belongs in reference
- The `src/debug_api/` module structure, `src/api/controller.rs` registration points, and `src/debug_api/snapshot.rs` state shapes provide authoritative source material for endpoint definitions
- This fills the most obvious gap in the Reference section, which currently documents the main HTTP API, CLIs, and configuration schema, but not the debug surface

Exact additional information needed:
- file: src/debug_api/mod.rs
  why: Shows module exports and public API structure, indicating which debug components are exposed
- file: src/debug_api/view.rs
  why: Contains request handlers for `/debug/snapshot`, `/debug/verbose`, `/debug/ui` endpoints and their authorization checks
- file: src/debug_api/snapshot.rs
  why: Defines the `SystemSnapshot` type and versioned state structures (`config`, `pginfo`, `dcs`, `process`, `ha`) that appear in verbose responses
- file: src/api/controller.rs
  why: Shows exact endpoint paths, HTTP methods, and routing logic for debug endpoints alongside main API routes
- file: src/config/schema.rs
  why: Documents the `[debug]` configuration block and `enabled` toggle that controls debug endpoint availability
- file: src/worker_contract_tests.rs
  why: Contains API contract tests that assert debug endpoint behavior, response shapes, and authorization checks

Optional runtime evidence to generate:
- command: cargo run --bin pgtuskmaster -- --config docker/configs/cluster/node-a/runtime.toml (with `[debug] enabled = true`) and then `curl -v http://127.0.0.1:18081/debug/snapshot`
  why: Provides actual HTTP response samples and confirms endpoint availability when debug mode is enabled
- command: cargo doc --open --no-deps --package pgtuskmaster_rust --document-private-items and inspect `debug_api` module documentation
  why: Reveals internal struct definitions and field-level documentation that should be surfaced in the reference page
