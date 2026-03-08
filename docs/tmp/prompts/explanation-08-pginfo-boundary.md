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
- docs/src/explanation/pginfo-observation-boundary.md

[Page title]
- # Why pginfo observes PostgreSQL instead of controlling it

[Audience]
- Engineers who see pginfo feeding HA decisions and want to understand why its role stops at observation.

[User need]
- Understand the value of a dedicated observation worker and why control work is left to HA and process layers.

[mdBook context]
- Link naturally to pginfo, HA state machine, process worker, and shared state references.

[Diataxis guidance]
- Explanation only.

[Verified facts that are true]
- The pginfo worker polls PostgreSQL using poll_once against postgres_conninfo.
- On successful poll it publishes a state derived with WorkerStatus::Running and SqlStatus::Healthy.
- On poll failure it emits a warning event and publishes a state with WorkerStatus::Running and SqlStatus::Unreachable.
- The worker emits SQL status transition events when status changes.
- Real tests in src/pginfo/worker.rs verify transitions from unreachable to primary and track WAL and slots.
- The HA worker consumes pginfo state from a subscriber as one input among several when deciding next steps.

[Relevant repo grounding]
- src/pginfo/worker.rs
- src/ha/worker.rs

[Design tensions to explain]
- Why a polling observer is valuable even when HA ultimately needs to act.
- Why pginfo publishes degraded observations instead of throwing fatal control errors.
- Why keeping observation separate from process control improves reasoning.

[Required structure]
- Explain pginfo as a sensor, not a controller.
- Explain how degraded observations still let the system reason.
- Explain the tradeoffs of polling and asynchronous observation.

[Facts that must not be invented or changed]
- Do not claim pginfo restarts or promotes PostgreSQL.
- Do not claim unreachable always means the node is fenced or failed over.
