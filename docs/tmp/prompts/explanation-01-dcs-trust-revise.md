Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Draft the page from scratch. The earlier attempt returned no content.

[Output path]
- docs/src/explanation/dcs-trust-and-coordination.md

[Page title]
- # Why DCS trust and coordination shape the cluster

[Audience]
- Engineers and operators who already know the reference surfaces and want to understand why pgtuskmaster ties HA behavior to DCS trust and local member publication.

[User need]
- Understand why DCS is more than a key-value transport, why trust levels exist, and how that changes HA behavior and fail-safe posture.

[Diataxis guidance]
- This page is explanation: cognition + acquisition.
- Provide context, reasons, tradeoffs, alternatives, and consequences.
- Keep instruction and neutral catalog material out of the page.

[Verified facts that are true]
- The DCS worker publishes the local member record before evaluating trust; if local member publication fails, trust becomes NotTrusted.
- The DCS worker drains watch events, applies them to a cache, and marks the store unhealthy when write, drain, or refresh work fails.
- When the DCS store is unhealthy, the published DcsState uses WorkerStatus::Faulted and forces DcsTrust::NotTrusted.
- The trust model includes FullQuorum, FailSafe, and NotTrusted.
- HA decision logic begins by checking trust. If trust is not FullQuorum and Postgres is primary, the next phase is FailSafe with EnterFailSafe { release_leader_lease: false }. If trust is not FullQuorum and Postgres is not primary, the next phase is FailSafe with NoChange.
- The API controller writes switchover requests into DCS-backed scope paths rather than directly assigning leadership.
- The policy test allows post-start observation through GET /ha/state and admin switchover requests while forbidding direct DCS writes, internal worker calls, and other internal steering after startup.

[Relevant repo grounding]
- src/dcs/worker.rs
- src/ha/decide.rs
- src/api/controller.rs
- tests/policy_e2e_api_only.rs

[Required structure]
- Start with a framing section that explains DCS as shared coordination state.
- Include a section on trust as a precondition for authority.
- Include a section on how trust changes fail-safe behavior.
- Include a section on consequences for APIs, tests, and operators.

[Facts that must not be invented or changed]
- Do not claim a specific quorum algorithm beyond the named trust states.
- Do not claim the API can directly assign leadership.
- Do not claim every DCS error kills the process immediately.
