# Contributors

This section is implementation-focused. It explains how the code actually works: module ownership, runtime wiring, HA decisions, interface contracts, and the testing/harness system that keeps behavior honest.

Compared with operator sections, contributor chapters go deeper into:

- explicit call paths (who calls whom, when, and why)
- state ownership and update semantics
- failure-path behavior and safety boundaries
- how to extend the system without breaking invariants.

## How to use this section

If you are reading to get oriented, follow the recommended order below.

If you are here with a specific goal, use the “Which chapter answers which question?” map to jump directly to the right page.

## Recommended read order

This order matches the navigation order in `SUMMARY.md` and is designed to build a correct mental model progressively:

1. Codebase Map
2. Worker Wiring and State Flow
3. HA Decision and Action Pipeline
4. API and Debug Contracts
5. Testing System Deep Dive
6. Harness Internals
7. Verification Workflow
8. Docs Authoring

## Which chapter answers which question?

- “Where should this new feature live?” → [Codebase Map](./codebase-map.md)
- “How does state flow between workers?” → [Worker Wiring and State Flow](./worker-wiring.md)
- “Why did HA choose these actions?” → [HA Decision and Action Pipeline](./ha-pipeline.md)
- “How do clients write intent / read state?” → [API and Debug Contracts](./api-debug-contracts.md)
- “What tests should I add for this change?” → [Testing System Deep Dive](./testing-system.md)
- “Why is e2e flaky / where are the artifacts?” → [Harness Internals](./harness-internals.md)
- “How do we keep docs aligned with reality?” → [Verification Workflow](./verification.md)
- “What is the writing standard for contributor docs?” → [Docs Authoring](./docs-style.md)

## A small contributor mental model

If you keep one model in mind while changing code, make it this:

- `pginfo` observes local Postgres and publishes a typed snapshot.
- `dcs` maintains a watch cache + trust view and publishes it.
- `ha` reads pginfo/dcs/process snapshots, decides the next phase, and dispatches side effects.
- `process` executes local side effects and reports outcomes.
- `debug_api` composes a snapshot that the API and debug clients can consume.
- `api` routes requests, writes operator intents into DCS, and serves reads from the composed snapshot.

Once you can trace those edges, you can debug most issues in minutes instead of hours.
