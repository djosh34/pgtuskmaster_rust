Target docs path: docs/src/reference/debug-api.md  
Diataxis type: reference  
Why this is the next doc:  
- The codebase exposes a debug API (`/debug/verbose`, `/debug/snapshot`, `/debug/ui`) that is not documented.  
- Operators need a reference that lists endpoints, authentication requirements, request/response schemas, and example outputs.  
- The existing HTTP API reference does not cover debug endpoints, leaving a gap in operator tooling.  

Exact additional information needed:  
- file: src/api/controller.rs  
  why: to identify exact debug endpoint routes, HTTP methods, and auth requirements.  
- file: src/config/schema.rs  
  why: to confirm the `debug.enabled` flag and any related configuration knobs.  
- file: src/debug_api/worker.rs  
  why: to understand the internal data structures and snapshot generation logic.  
- file: src/debug_api/view.rs  
  why: to capture the JSON schema of the verbose payload and its nested fields.  
- file: docker/configs/cluster/node-a/runtime.toml  
  why: to extract a sample configuration that enables debug mode.  
- extra info: sample raw JSON output from a live node's `/debug/verbose` and `/debug/snapshot` endpoints (including `since` query usage).  

Optional runtime evidence to generate:  
- command: `curl -s http://127.0.0.1:8080/debug/verbose?since=0`  
  why: to capture the full verbose payload shape for documentation.  
- command: `curl -s http://127.0.0.1:8080/debug/snapshot`  
  why: to provide a snapshot response example.
