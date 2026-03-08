Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise the supplied draft so it stays within prompt-backed facts and remains an explanation page.

[Output path]
- docs/src/explanation/ha-decisions-and-actions.md

[Existing draft to revise]
- docs/tmp/k2_outputs/explanation-02-ha-decisions.md

[Problems to fix]
- Remove unsupported examples such as "restart" and "reload" because the provided facts only support a generic separation into effect buckets and the apply layer details shown in the prompt.
- Keep the page architectural and explanatory.

[Verified facts that are true]
- The HA worker reacts to changes from pginfo, DCS, process, and config subscribers, plus a poll interval.
- Each HA step builds a world snapshot from subscribers, calls decide, lowers the resulting decision to an effect plan, publishes the next HA state, emits transition events, and then applies effects unless redundant process dispatch should be skipped.
- The lowering pipeline separates decision selection from concrete effects such as Postgres actions, lease changes, switchover cleanup, replication work, and safety actions.
- If effect application returns dispatch errors, HA republishes a faulted state that keeps the chosen phase and decision but marks worker status as faulted.
- There is explicit logic to skip redundant process dispatch when phase and decision are unchanged for certain waiting or recovery decisions.
- The decision layer has a global trust override before phase-specific logic.

[Required structure]
- Introduce the "observe, decide, lower, apply" pipeline.
- Explain what each stage protects or clarifies.
- Weigh the tradeoffs of publishing intent before all side effects finish.
- Close with what this means for debugging and evolution.
