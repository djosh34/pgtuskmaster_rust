# Contributors

This section is implementation-focused. It is the shortest path to answering the contributor questions that matter during a real change: where the runtime starts, which worker owns a behavior, how HA decisions turn into side effects, which read model clients consume, and which tests prove the contract you are about to touch.

Compared with operator pages, these chapters go deeper into call paths, publisher/subscriber ownership, failure behavior, and the boundaries that must stay explicit if you want to change the system without creating split-brain or false confidence.

## How to use this section

If you are reading to get oriented, follow the recommended order below.

If you are here with a specific job, use the question map to jump directly to the page that names the owning files and invariants.

## Recommended read order

This order matches `SUMMARY.md` and is designed to build the runtime outward from startup wiring:

1. Codebase Map
2. Worker Wiring and State Flow
3. HA Decision and Effect-Plan Pipeline
4. API and Debug Contracts
5. Testing System Deep Dive
6. Harness Internals
7. Docs Verification and Authoring

## Which chapter answers which question?

- "Where should this new feature or bug fix live?" -> [Codebase Map](./codebase-map.md)
- "Where does startup end and steady-state begin?" -> [Codebase Map](./codebase-map.md) then [Worker Wiring and State Flow](./worker-wiring.md)
- "How does state flow between workers, and who owns the published snapshot?" -> [Worker Wiring and State Flow](./worker-wiring.md)
- "Why did HA choose this phase or side effect?" -> [HA Decision and Effect-Plan Pipeline](./ha-pipeline.md)
- "Which file do I open to change `/switchover`, `/ha/state`, or `/debug/verbose`?" -> [API and Debug Contracts](./api-debug-contracts.md)
- "What tests should I add for this change, and where are the slow HA scenarios?" -> [Testing System Deep Dive](./testing-system.md)
- "Why is a real-binary test flaky, and where are the artifacts?" -> [Harness Internals](./harness-internals.md)
- "How do we keep contributor docs factual and worth reading?" -> [Docs Verification and Authoring](./verification.md)

## Fast starting points in code

When you only have a few minutes, start from these entrypoints:

- `src/bin/pgtuskmaster.rs`: CLI entry into `runtime::node::run_node_from_config_path(...)`.
- `src/runtime/node.rs`: startup planning, startup execution, `run_workers(...)`, and shared channel creation.
- `src/ha/worker.rs`: one HA tick from current world snapshot to published `HaState` plus applied effect plan.
- `src/api/worker.rs`: auth, routing, debug endpoints, and intent writes.
- `src/test_harness/ha_e2e/startup.rs`: how the real-binary cluster fixtures are assembled.

## The contributor mental model

If you keep one model in mind while changing code, make it this:

- `pginfo` observes local Postgres and publishes a typed snapshot.
- `dcs` maintains the scoped watch cache, publishes trust, and writes this node's member record.
- `ha` reads pginfo, dcs, process, and config snapshots, chooses the next phase, publishes `HaState`, and applies the effect plan.
- `process` is the local side-effect boundary for Postgres lifecycle and recovery jobs.
- `debug_api` composes the read model that API reads and debug clients inspect.
- `api` is the external edge: it authorizes requests, writes typed intents, and serves read projections from the composed snapshot.

Once you can trace those edges, you can debug most issues in minutes instead of hours.
