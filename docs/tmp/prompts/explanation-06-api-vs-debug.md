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
- docs/src/explanation/api-and-debug-boundaries.md

[Page title]
- # Why the control API and debug API are separate

[Audience]
- Engineers and operators who want to understand why the project exposes both a control surface and a richer internal snapshot surface.

[User need]
- Understand why the HTTP API stays narrow while the debug API collects verbose cross-domain snapshots and change history.

[mdBook context]
- Link naturally to the HTTP API, debug API, shared state, and HA state machine reference pages.
- Do not turn this into endpoint-by-endpoint reference.

[Diataxis guidance]
- Explanation only.
- Focus on context and consequences.

[Verified facts that are true]
- The API controller exposes switchover create/delete operations and HA state derived from a debug snapshot.
- The API worker supports auth roles and TLS handling and can carry an optional debug snapshot subscriber.
- The debug API worker builds SystemSnapshot values from config, pginfo, DCS, process, and HA states, tracks a sequence number, records change events by domain, records a bounded timeline, and trims history to a limit.
- The debug API has a history limit default of 300 entries.
- The e2e policy permits observation through GET /ha/state and admin switchover requests while forbidding direct internal steering after startup.

[Relevant repo grounding]
- src/api/controller.rs
- src/api/worker.rs
- src/debug_api/snapshot.rs
- src/debug_api/worker.rs
- tests/policy_e2e_api_only.rs

[Design tensions to explain]
- Why not make the public control API the same thing as the internal debug surface.
- Why the narrow control API fits the hands-off control philosophy.
- Why verbose snapshots and change history are valuable separately.
- Tradeoffs of having two HTTP-adjacent surfaces.

[Required structure]
- Explain the difference between control intent and system introspection.
- Explain why snapshots aggregate cross-worker state.
- Explain how the separation helps policy, debugging, and operational safety.

[Facts that must not be invented or changed]
- Do not claim the debug API is the only source of HA state.
- Do not claim the public API directly mutates HA phase.
