Target docs path: docs/src/reference/http-api.md
Diataxis type: reference
Why this is the next doc:
- The CLI reference documents client commands but not the underlying HTTP API surface
- Debug APIs exist in src/debug_api/ but are absent from all published docs
- Operators need a complete API reference to build automation beyond the CLI
- The current reference section lacks documentation for the API layer that controllers and debug workers expose

Exact additional information needed:
- file: src/api/controller.rs
  why: To enumerate public HTTP endpoints, their routes, and handler logic
- file: src/api/worker.rs
  why: To document server initialization, middleware, and base path configuration
- file: src/debug_api/view.rs
  why: To list debug endpoint paths and their response schemas
- file: src/debug_api/snapshot.rs
  why: To capture snapshot data structures returned by debug endpoints
- file: src/debug_api/worker.rs
  why: To understand debug API initialization and conditional exposure
- extra info: Complete list of all API endpoint paths (e.g., /ha/state, /debug/snapshot, /debug/vars)
- extra info: Request/response schemas for each method (GET/POST/DELETE)
- extra info: Authentication requirements per endpoint (read token vs admin token)
- extra info: Error response format and status code conventions

Optional runtime evidence to generate:
- command: cargo run --bin pgtuskmaster -- --config docker/configs/single/node-a/runtime.toml
  why: To launch a documented instance for capturing live API responses
- command: curl -s http://127.0.0.1:8080/ha/state
  why: To record actual JSON response shape and headers
- command: curl -s http://127.0.0.1:8080/debug/snapshot
  why: To document debug output format and available diagnostic fields
