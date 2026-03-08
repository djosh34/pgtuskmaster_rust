Target docs path: `docs/src/explanation/architecture.md`

Diataxis type: explanation

Why this is the next doc:
- Fills the missing explanation category in the documentation map
- Complements existing tutorial (learning-oriented action), how-to (work-oriented action), and reference (work-oriented cognition) sections
- The codebase contains complex interactions between HA worker, DCS (etcd), API layer, and PostgreSQL processes that benefit from conceptual explanation
- Users need understanding of HA decision-making, component responsibilities, and system invariants to effectively operate clusters

Exact additional information needed:
- file: `src/lib.rs` - to understand module organization and public API surface
- file: `src/ha/decide.rs` - to understand HA decision logic and state machine
- file: `src/dcs/state.rs` - to understand distributed consensus state representation
- file: `src/api/controller.rs` - to understand API request routing and handler structure
- file: `src/config/schema.rs` - to understand configuration options that drive architectural behavior
- file: `docker/configs/cluster/node-a/runtime.toml` - to see concrete runtime configuration example
- file: `tests/ha/support/observer.rs` - to understand how HA state is monitored for correctness
- extra info: What are the key architectural invariants that prevent split-brain scenarios?
- extra info: How do the HA worker, DCS worker, and API worker coordinate during leader transitions?

Optional runtime evidence to generate:
- command: `cargo doc --no-deps --open` - to inspect generated API documentation for architecture patterns
- command: `docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d && sleep 10 && docker compose logs` - to observe inter-component communication during cluster startup
