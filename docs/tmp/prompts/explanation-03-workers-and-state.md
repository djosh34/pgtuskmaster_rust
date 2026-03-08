Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Draft a new explanation page.

[Output path]
- docs/src/explanation/workers-and-versioned-state.md

[Page title]
- # Why the runtime is built from workers and versioned state

[Audience]
- Readers who want to understand the architectural reason for separate workers and watch-based state channels.

[User need]
- Understand why pgtuskmaster composes the system from publishers, subscribers, and narrow worker responsibilities.

[mdBook context]
- Link naturally to the node runtime, shared state, pginfo, DCS, process worker, HA, and debug API references.
- Avoid listing every field from each state type.

[Diataxis guidance]
- Explanation page only.
- Focus on reasons and consequences.

[Verified facts that are true]
- Shared state uses tokio watch channels carrying Versioned<T> snapshots with strict +1 version increments.
- Publishers attach updated_at timestamps and subscribers can read latest() or await changed().
- The node runtime wires separate workers for pginfo, DCS, process, HA, API, and debug API.
- The debug API assembles snapshots from config, pginfo, DCS, process, and HA versioned states and records change and timeline history.
- The HA worker and debug API worker both consume multiple subscribers rather than reading each other's internals directly.

[Relevant repo grounding]
- src/state/watch_state.rs: new_state_channel, publish, latest, changed.
- src/runtime/node.rs: worker wiring and initial state creation.
- src/debug_api/snapshot.rs and src/debug_api/worker.rs: composed system snapshots and history retention.

[Design tensions to explain]
- Why use explicit versioned snapshots instead of direct shared mutable state.
- Why separate workers communicate through published state instead of calling across layers.
- What this buys for observability and testability.
- The cost of eventual consistency between workers.

[Required structure]
- Frame the architecture as a state-publishing topology.
- Explain the benefit of versioned snapshots.
- Explain why the debug API depends on this model.
- Discuss tradeoffs such as stale views and asynchronous lag.

[Facts that must not be invented or changed]
- Do not claim the channels buffer a full history; the debug API keeps its own bounded history.
- Do not claim the architecture is actor-framework based if that term is not grounded here.
