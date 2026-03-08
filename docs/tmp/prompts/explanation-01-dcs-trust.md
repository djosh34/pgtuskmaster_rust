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
- docs/src/explanation/dcs-trust-and-coordination.md

[Page title]
- # Why DCS trust and coordination shape the cluster

[Audience]
- Engineers and operators who already know the reference surfaces and want to understand why pgtuskmaster ties HA behavior to DCS trust and local member publication.

[User need]
- Understand why DCS is more than a key-value transport, why trust levels exist, and how that changes HA behavior and fail-safe posture.

[mdBook context]
- Link naturally to the DCS, HA state machine, and shared state reference pages.
- Do not turn this into endpoint, type, or key-by-key reference text.
- If a diagram would help, leave a placeholder like [diagram about trust gating and lease ownership].

[Diataxis guidance]
- This page is explanation: cognition + acquisition.
- Provide context, reasons, tradeoffs, alternatives, and consequences.
- Keep instruction and neutral catalog material out of the page.
- The page should answer why the design works this way, not how to call APIs.

[Verified facts that are true]
- The DCS worker publishes the local member record before evaluating trust; if local member publication fails, trust becomes NotTrusted.
- The DCS worker drains watch events, applies them to a cache, and marks the store unhealthy when write, drain, or refresh work fails.
- When the DCS store is unhealthy, the published DcsState uses WorkerStatus::Faulted and forces DcsTrust::NotTrusted.
- The trust model includes FullQuorum, FailSafe, and NotTrusted.
- HA decision logic begins by checking trust. If trust is not FullQuorum and Postgres is primary, the next phase is FailSafe with EnterFailSafe { release_leader_lease: false }. If trust is not FullQuorum and Postgres is not primary, the next phase is FailSafe with NoChange.
- The HTTP API exposes HA state and switchover requests through the DCS-backed model, not through direct leader steering.
- The e2e policy test forbids tests from steering HA through direct DCS writes or internal worker calls after startup, while allowing observation via GET /ha/state and admin switchover requests.

[Relevant repo grounding]
- src/dcs/worker.rs: step_once publishes local member state, drains watch events, refreshes cache, and derives trust from store health and cache.
- src/ha/decide.rs: decide_phase has a global trust override before normal phase handling.
- src/api/controller.rs: switchover requests are written into DCS scope paths and HA state is reported from snapshots.
- tests/policy_e2e_api_only.rs: documents and enforces the post-start hands-off policy.

[Design tensions to explain]
- Why require both local publication success and store health before trusting DCS.
- Why trust is an input to HA decisions instead of a separate advisory metric.
- Why switchover is represented as a DCS request rather than a direct imperative primary switch.
- Why the project prefers a hands-off post-start policy in tests and operators.

[Required structure]
- Start with a short framing section about DCS as shared coordination state, not mere storage.
- Include a section on trust as a precondition for authority.
- Include a section on how trust changes fail-safe behavior.
- Include a section on the consequences for APIs, tests, and operators.

[Facts that must not be invented or changed]
- Do not claim a quorum algorithm beyond the named trust states.
- Do not claim the API can directly assign leadership.
- Do not claim DCS writes are always fatal; the worker logs failures and can continue with unhealthy state.

[Style constraints]
- Keep the tone architectural and explanatory.
- Prefer concrete consequences over abstract slogans.
