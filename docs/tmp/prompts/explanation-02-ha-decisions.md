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
- docs/src/explanation/ha-decisions-and-actions.md

[Page title]
- # How HA turns shared state into actions

[Audience]
- Engineers who have seen the HA phase and decision enums in reference docs and want the conceptual model behind them.

[User need]
- Understand why the HA worker is split into world snapshot, decision, lowering, and action dispatch stages.

[mdBook context]
- Link naturally to the HA state machine, process worker, and DCS reference pages.
- Do not restate the full transition matrix.
- Leave diagram placeholders instead of diagrams if useful.

[Diataxis guidance]
- Explanation only: context, reasons, tradeoffs, alternatives, consequences.
- Avoid procedural steps and exhaustive enum dumps.

[Verified facts that are true]
- The HA worker reacts to changes from pginfo, DCS, process, and config subscribers, plus a poll interval.
- Each HA step builds a world snapshot from subscribers, calls decide, lowers the resulting decision to an effect plan, publishes the next HA state, emits transition events, and then applies effects unless redundant process dispatch should be skipped.
- The lowering pipeline separates decision selection from concrete effects such as Postgres actions, lease changes, switchover cleanup, replication work, and safety actions.
- If effect application returns dispatch errors, HA republishes a faulted state that keeps the chosen phase and decision but marks worker status as faulted.
- There is explicit logic to skip redundant process dispatch when phase and decision are unchanged for certain waiting or recovery decisions.
- The decision layer has a global trust override before phase-specific logic.

[Relevant repo grounding]
- src/ha/worker.rs: event loop, step_once sequencing, state publication, redundant dispatch check.
- src/ha/decide.rs: trust gate and phase-to-decision logic.
- src/ha/apply.rs: effect buckets and dispatch ordering.

[Design tensions to explain]
- Why the design avoids a monolithic "do failover" routine.
- Why decision and action dispatch are separated.
- Why the chosen state is published before action dispatch completes.
- Why redundant dispatch skipping exists for some states.

[Required structure]
- Introduce the "observe, decide, lower, apply" pipeline.
- Explain what each stage protects or clarifies.
- Weigh the tradeoffs of publishing intent before all side effects finish.
- Close with what this means for debugging and evolution of the HA system.

[Facts that must not be invented or changed]
- Do not claim every decision produces actions.
- Do not claim effect application is transactional.
- Do not claim the worker ignores process, config, or DCS updates.
