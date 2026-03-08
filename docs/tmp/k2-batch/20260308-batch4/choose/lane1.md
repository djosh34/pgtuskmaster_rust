Target docs path: docs/src/tutorial/single-node-setup.md
Diataxis type: tutorial
Why this is the next doc:
- The existing tutorials jump directly to a 3-node HA cluster, which is complex for first-time users
- A single-node tutorial provides a gentler learning path for developers to understand basic concepts before tackling multi-node HA
- The repository already contains single-node Docker configuration (`docker/compose/docker-compose.single.yml`) that can serve as the basis
- Following Diátaxis principles, this would be a practical, concrete, and safe lesson that delivers visible results early
- It addresses the " acquisition + action" quadrant for newcomers who want to learn by doing, but with lower initial complexity

Exact additional information needed:
- file: docker/compose/docker-compose.single.yml
  why: To understand the exact services, ports, and configuration used for single-node deployment
- file: docker/configs/single/node-a/runtime.toml
  why: To see the minimal runtime configuration for a single node and understand which sections can be simplified
- file: src/bin/pgtuskmaster.rs
  why: To understand the daemon startup process and any single-node-specific behavior
- file: src/config/schema.rs
  why: To identify which configuration parameters are optional for single-node setups
- file: tests/policy_e2e_api_only.rs
  why: To understand if there are existing tests that exercise single-node behavior patterns
- extra info: What is the minimum viable PostgreSQL version requirement for single-node mode?
- extra info: Are there any features disabled or behavior changes in single-node vs multi-node?

Optional runtime evidence to generate:
- command: docker compose -f docker/compose/docker-compose.single.yml up -d --build
  why: To verify the single-node setup works and capture exact startup logs
- command: curl -sf http://127.0.0.1:18081/ha/state | jq .
  why: To confirm the API endpoint responds correctly in single-node mode
- command: cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 ha state
  why: To validate the CLI works against a single-node deployment
