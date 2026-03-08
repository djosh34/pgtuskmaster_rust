Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Draft a new explanation page from scratch.

[Output path]
- docs/src/explanation/workers-and-versioned-state.md

[Page title]
- # Why the runtime is built from workers and versioned state

[Audience]
- Readers who want to understand the architectural reason for separate workers and watch-based state channels.

[User need]
- Understand why pgtuskmaster composes the system from publishers, subscribers, and narrow worker responsibilities.

[Diataxis guidance]
- Explanation only.
- Focus on context, reasons, tradeoffs, and consequences.

[Verified facts that are true]
- Shared state uses tokio watch channels carrying Versioned<T> snapshots with strict +1 version increments.
- Publishers attach updated_at timestamps and subscribers can read latest() or await changed().
- The node runtime wires separate workers for pginfo, DCS, process, HA, API, and debug API.
- The debug API assembles snapshots from config, pginfo, DCS, process, and HA versioned states and records change and timeline history.
- The HA worker and debug API worker both consume multiple subscribers rather than reading each other's internals directly.

[Required structure]
- Frame the architecture as a state-publishing topology.
- Explain the benefit of versioned snapshots.
- Explain why the debug API depends on this model.
- Discuss tradeoffs such as stale views and asynchronous lag.
