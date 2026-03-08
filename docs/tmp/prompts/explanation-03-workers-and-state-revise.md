Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise the supplied draft for ASCII punctuation and tighter grounding.

[Output path]
- docs/src/explanation/workers-and-versioned-state.md

[Existing draft to revise]
- docs/tmp/k2_outputs/explanation-03-workers-and-state.md

[Problems to fix]
- Replace all non-ASCII punctuation with ASCII punctuation.
- Avoid overclaiming cross-layer purity beyond the grounded facts. It is fine to say workers consume published state and that debug API and HA use subscribers, but do not claim any stronger framework or total prohibition than the facts support.

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
